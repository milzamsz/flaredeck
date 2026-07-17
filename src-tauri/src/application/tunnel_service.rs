use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::System;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::cloudflared::{
    cloudflared_dir, effective_cert_path, ensure_cloudflared, flaredeck_index_path,
};
use crate::error::{AppError, AppResult};
use crate::state::RuntimeState;
use crate::types::{ProfileIndex, TunnelStatus};

const FAILURE_WINDOW_SECS: u64 = 30;
const FAILURE_LIMIT: usize = 3;
const EARLY_EXIT_WAIT_MS: u64 = 600;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelLogEvent {
    pub profile_id: String,
    pub stream: &'static str,
    pub line: String,
}

#[derive(Debug, Clone)]
pub struct TunnelObservation {
    pub pid: u32,
    pub started_at_seconds: u64,
    pub executable: String,
}

type LogSink = Arc<dyn Fn(TunnelLogEvent) + Send + Sync>;

fn record_failure(state: &RuntimeState, profile_id: &str) -> usize {
    let mut guard = match state.recent_failures.lock() {
        Ok(g) => g,
        Err(_) => return 0,
    };
    let now = Instant::now();
    let window = Duration::from_secs(FAILURE_WINDOW_SECS);
    let entry = guard.entry(profile_id.to_string()).or_default();
    entry.retain(|t| now.duration_since(*t) <= window);
    entry.push(now);
    entry.len()
}

fn clear_failures(state: &RuntimeState, profile_id: &str) {
    if let Ok(mut guard) = state.recent_failures.lock() {
        guard.remove(profile_id);
    }
}

fn spawn_log_reader<R>(sink: LogSink, profile_id: String, stream: &'static str, reader: R)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            sink(TunnelLogEvent {
                profile_id: profile_id.clone(),
                stream,
                line,
            });
        }
    });
}

pub fn status(state: &RuntimeState, profile_id: String) -> AppResult<TunnelStatus> {
    let mut guard = state
        .children
        .lock()
        .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
    let (running, pid) = match guard.get_mut(&profile_id) {
        Some(child) => match child.try_wait() {
            Ok(Some(_)) => (false, None),
            Ok(None) => (true, child.id()),
            Err(_) => (false, None),
        },
        None => (false, None),
    };
    if !running {
        guard.remove(&profile_id);
    }
    Ok(TunnelStatus {
        profile_id,
        running,
        pid,
    })
}

async fn resolve_profile(profile_id: &str) -> AppResult<crate::types::Profile> {
    let path = flaredeck_index_path()?;
    if !path.exists() {
        return Err(AppError::ProfileNotFound(profile_id.to_string()));
    }
    let raw = tokio::fs::read_to_string(&path).await?;
    serde_json::from_str::<ProfileIndex>(&raw)
        .unwrap_or_default()
        .profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.to_string()))
}

/// Observes only a process whose command line includes this profile's tunnel
/// name. A PID alone is never considered ownership evidence.
pub async fn observe(profile_id: &str) -> AppResult<Option<TunnelObservation>> {
    let profile = resolve_profile(profile_id).await?;
    let system = System::new_all();
    Ok(system.processes().iter().find_map(|(pid, process)| {
        let is_cloudflared = process
            .exe()
            .and_then(|path| path.file_stem())
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("cloudflared"));
        let has_tunnel_name = process
            .cmd()
            .iter()
            .any(|argument| argument.to_string_lossy() == profile.tunnel_name);
        if is_cloudflared && has_tunnel_name {
            process.exe().map(|executable| TunnelObservation {
                pid: pid.as_u32(),
                started_at_seconds: process.start_time(),
                executable: executable.to_string_lossy().into_owned(),
            })
        } else {
            None
        }
    }))
}

pub async fn start<F>(state: &RuntimeState, profile_id: String, sink: F) -> AppResult<TunnelStatus>
where
    F: Fn(TunnelLogEvent) + Send + Sync + 'static,
{
    if state
        .children
        .lock()
        .map_err(|_| AppError::Other("runtime lock poisoned".into()))?
        .contains_key(&profile_id)
    {
        return Err(AppError::ProfileAlreadyRunning(profile_id));
    }
    {
        let failures = state
            .recent_failures
            .lock()
            .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
        if failures
            .get(&profile_id)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|t| t.elapsed() <= Duration::from_secs(FAILURE_WINDOW_SECS))
                    .count()
            })
            .unwrap_or(0)
            >= FAILURE_LIMIT
        {
            return Err(AppError::Other(format!("tunnel has crashed {FAILURE_LIMIT} times in the last {FAILURE_WINDOW_SECS}s; check the logs and try again later")));
        }
    }
    let binary = ensure_cloudflared()?;
    let profile = resolve_profile(&profile_id).await?;
    let cert = effective_cert_path(&profile)?;
    tokio::fs::create_dir_all(cloudflared_dir()?).await?;
    let mut command = tokio::process::Command::new(binary);
    command.arg("tunnel").arg("--no-autoupdate");
    if std::path::Path::new(&profile.config_path).exists() {
        command.arg("--config").arg(&profile.config_path);
    }
    command
        .arg("run")
        .arg(&profile.tunnel_name)
        .env("TUNNEL_ORIGIN_CERT", cert)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let pid = child.id();
    let sink: LogSink = Arc::new(sink);
    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(sink.clone(), profile_id.clone(), "stdout", stdout);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(sink, profile_id.clone(), "stderr", stderr);
    }
    tokio::time::sleep(Duration::from_millis(EARLY_EXIT_WAIT_MS)).await;
    match child.try_wait() {
        Ok(Some(status)) => {
            let count = record_failure(state, &profile_id);
            return Err(AppError::Other(format!("cloudflared exited immediately (code {:?}, {count}/{FAILURE_LIMIT} recent failures); see logs", status.code())));
        }
        Ok(None) => {}
        Err(error) => return Err(AppError::Other(format!("failed to poll child: {error}"))),
    }
    clear_failures(state, &profile_id);
    state
        .children
        .lock()
        .map_err(|_| AppError::Other("runtime lock poisoned".into()))?
        .insert(profile_id.clone(), child);
    Ok(TunnelStatus {
        profile_id,
        running: true,
        pid,
    })
}

pub async fn stop(state: &RuntimeState, profile_id: String) -> AppResult<TunnelStatus> {
    let mut child = state
        .children
        .lock()
        .map_err(|_| AppError::Other("runtime lock poisoned".into()))?
        .remove(&profile_id);
    if let Some(child) = child.as_mut() {
        kill_process(child, child.id()).await;
        let _ = child.wait().await;
    }
    Ok(TunnelStatus {
        profile_id,
        running: false,
        pid: None,
    })
}

pub async fn restart<F>(
    state: &RuntimeState,
    profile_id: String,
    sink: F,
) -> AppResult<TunnelStatus>
where
    F: Fn(TunnelLogEvent) + Send + Sync + 'static,
{
    stop(state, profile_id.clone()).await?;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let sink = Arc::new(sink);
    let mut last_error = None;
    for attempt in 0..3 {
        let sink = sink.clone();
        match start(state, profile_id.clone(), move |event| sink(event)).await {
            Ok(status) => return Ok(status),
            Err(error) => {
                last_error = Some(error);
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| AppError::Other("restart failed".into())))
}

#[cfg(unix)]
async fn kill_process(child: &mut tokio::process::Child, _pid: Option<u32>) {
    use std::process::Command;
    if let Some(pid) = child.id() {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status();
    } else {
        let _ = child.start_kill();
    }
}

#[cfg(windows)]
async fn kill_process(child: &mut tokio::process::Child, pid: Option<u32>) {
    use std::process::Command;
    if let Some(pid) = pid.or_else(|| child.id()) {
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .status();
    } else {
        let _ = child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::{start, status, stop, FAILURE_LIMIT};
    use crate::state::RuntimeState;
    use std::time::Instant;

    #[test]
    fn unknown_profile_is_stopped() {
        assert!(
            !status(&RuntimeState::new(), "profile-1".into())
                .unwrap()
                .running
        );
    }

    #[tokio::test]
    async fn stop_without_owned_child_is_idempotent() {
        let status = stop(&RuntimeState::new(), "profile-1".into())
            .await
            .unwrap();
        assert!(!status.running);
        assert_eq!(status.pid, None);
    }

    #[tokio::test]
    async fn crashloop_rejection_happens_before_spawn() {
        let state = RuntimeState::new();
        state
            .recent_failures
            .lock()
            .unwrap()
            .insert("profile-1".into(), vec![Instant::now(); FAILURE_LIMIT]);
        let error = start(&state, "profile-1".into(), |_| {}).await.unwrap_err();
        assert!(error.to_string().contains("crashed"));
    }
}
