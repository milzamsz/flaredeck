use std::process::Stdio;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::cloudflared::{cloudflared_dir, ensure_cloudflared, flaredeck_index_path};
use crate::error::{AppError, AppResult};
use crate::state::RuntimeState;
use crate::types::{CloudflaredInfo, CreatedTunnel, ProfileIndex, TunnelListEntry, TunnelStatus};

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

fn record_failure(state: &State<'_, RuntimeState>, profile_id: &str) -> usize {
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

fn clear_failures(state: &State<'_, RuntimeState>, profile_id: &str) {
    if let Ok(mut guard) = state.recent_failures.lock() {
        guard.remove(profile_id);
    }
}

fn spawn_log_reader<R>(app: AppHandle, profile_id: String, stream: &'static str, reader: R)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app.emit(
                "tunnel:log",
                TunnelLogEvent {
                    profile_id: profile_id.clone(),
                    stream,
                    line,
                },
            );
        }
    });
}

#[tauri::command]
pub async fn cloudflared_check() -> AppResult<CloudflaredInfo> {
    let Some(path) = crate::cloudflared::resolve_cloudflared_path() else {
        return Ok(CloudflaredInfo {
            installed: false,
            path: None,
            version: None,
        });
    };
    let version = crate::cloudflared::cloudflared_version(&path).await;
    Ok(CloudflaredInfo {
        installed: true,
        path: Some(path.to_string_lossy().to_string()),
        version,
    })
}

#[tauri::command]
pub async fn tunnel_status(
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    let mut guard = state
        .children
        .lock()
        .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
    let entry = guard.get_mut(&profile_id);
    let (running, pid) = match entry {
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
    let index: ProfileIndex = serde_json::from_str(&raw).unwrap_or_default();
    index
        .profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| AppError::ProfileNotFound(profile_id.to_string()))
}

#[tauri::command]
pub async fn tunnel_start(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    {
        let guard = state
            .children
            .lock()
            .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
        if guard.contains_key(&profile_id) {
            return Err(AppError::ProfileAlreadyRunning(profile_id));
        }
    }

    {
        let guard = state
            .recent_failures
            .lock()
            .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
        let now = Instant::now();
        let window = Duration::from_secs(FAILURE_WINDOW_SECS);
        let count = guard
            .get(&profile_id)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|t| now.duration_since(**t) <= window)
                    .count()
            })
            .unwrap_or(0);
        if count >= FAILURE_LIMIT {
            return Err(AppError::Other(format!(
                "tunnel has crashed {count} times in the last {FAILURE_WINDOW_SECS}s; check the logs and try again later"
            )));
        }
    }

    let binary = ensure_cloudflared()?;
    let profile = resolve_profile(&profile_id).await?;
    let config_path = profile.config_path.clone();
    let tunnel_name = profile.tunnel_name.clone();

    tokio::fs::create_dir_all(cloudflared_dir()?).await?;

    let mut cmd = tokio::process::Command::new(&binary);
    cmd.arg("tunnel").arg("--no-autoupdate");
    if std::path::Path::new(&config_path).exists() {
        cmd.arg("--config").arg(&config_path);
    }
    cmd.arg("run").arg(&tunnel_name);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let pid = child.id();

    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(app.clone(), profile_id.clone(), "stdout", stdout);
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(app.clone(), profile_id.clone(), "stderr", stderr);
    }

    tokio::time::sleep(Duration::from_millis(EARLY_EXIT_WAIT_MS)).await;

    match child.try_wait() {
        Ok(Some(status)) => {
            let count = record_failure(&state, &profile_id);
            return Err(AppError::Other(format!(
                "cloudflared exited immediately (code {:?}, {count}/{FAILURE_LIMIT} recent failures); see logs",
                status.code()
            )));
        }
        Ok(None) => {}
        Err(e) => {
            return Err(AppError::Other(format!("failed to poll child: {e}")));
        }
    }

    clear_failures(&state, &profile_id);

    let mut guard = state
        .children
        .lock()
        .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
    guard.insert(profile_id.clone(), child);

    Ok(TunnelStatus {
        profile_id,
        running: true,
        pid,
    })
}

#[tauri::command]
pub async fn tunnel_stop(
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    let mut child = {
        let mut guard = state
            .children
            .lock()
            .map_err(|_| AppError::Other("runtime lock poisoned".into()))?;
        guard.remove(&profile_id)
    };

    if let Some(ref mut c) = child {
        let pid = c.id();
        kill_process(c, pid).await;
        let _ = c.wait().await;
    }

    Ok(TunnelStatus {
        profile_id,
        running: false,
        pid: None,
    })
}

#[tauri::command]
pub async fn tunnel_restart(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    let _ = tunnel_stop(state.clone(), profile_id.clone()).await?;
    tokio::time::sleep(Duration::from_millis(1500)).await;

    let mut last_err: Option<AppError> = None;
    for attempt in 0..3 {
        match tunnel_start(app.clone(), state.clone(), profile_id.clone()).await {
            Ok(status) => return Ok(status),
            Err(e) => {
                last_err = Some(e);
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| AppError::Other("restart failed".into())))
}

#[derive(Debug, Deserialize)]
struct CloudflaredListEntry {
    id: String,
    name: String,
    #[serde(default)]
    created_at: Option<String>,
}

#[tauri::command]
pub async fn tunnel_list() -> AppResult<Vec<TunnelListEntry>> {
    let binary = ensure_cloudflared()?;
    let output = tokio::process::Command::new(&binary)
        .args(["tunnel", "list", "--output", "json"])
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::CloudflaredFailed(
            output.status.code().unwrap_or(-1),
            stderr,
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<CloudflaredListEntry> = serde_json::from_str(&stdout).unwrap_or_default();
    Ok(entries
        .into_iter()
        .map(|e| TunnelListEntry {
            id: e.id,
            name: e.name,
            created_at: e.created_at,
        })
        .collect())
}

#[tauri::command]
pub async fn tunnel_route_dns(tunnel_name: String, hostname: String) -> AppResult<()> {
    let binary = ensure_cloudflared()?;
    let output = tokio::process::Command::new(&binary)
        .args(["tunnel", "route", "dns", "-f", &tunnel_name, &hostname])
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::CloudflaredFailed(
            output.status.code().unwrap_or(-1),
            stderr,
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CloudflaredCreatedTunnel {
    id: String,
    name: String,
}

#[tauri::command]
pub async fn tunnel_create(name: String) -> AppResult<CreatedTunnel> {
    let binary = ensure_cloudflared()?;
    let output = tokio::process::Command::new(&binary)
        .args(["tunnel", "create", "--output", "json", &name])
        .output()
        .await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::CloudflaredFailed(
            output.status.code().unwrap_or(-1),
            stderr,
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let (uuid, parsed_name) = if let Ok(parsed) =
        serde_json::from_str::<CloudflaredCreatedTunnel>(stdout.trim())
    {
        (parsed.id, parsed.name)
    } else {
        let combined = format!("{stdout}\n{stderr}");
        let uuid = find_uuid(&combined).ok_or_else(|| {
            AppError::Other("could not parse tunnel uuid from cloudflared output".into())
        })?;
        (uuid, name.clone())
    };

    let credentials_file = cloudflared_dir()?
        .join(format!("{uuid}.json"))
        .to_string_lossy()
        .to_string();

    Ok(CreatedTunnel {
        uuid,
        name: parsed_name,
        credentials_file,
    })
}

fn find_uuid(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    // Pattern: 8-4-4-4-12 hex chars separated by '-' (36 chars).
    for i in 0..bytes.len().saturating_sub(35) {
        let candidate = &bytes[i..i + 36];
        if is_uuid(candidate) {
            return Some(
                std::str::from_utf8(candidate)
                    .ok()?
                    .to_ascii_lowercase(),
            );
        }
    }
    None
}

fn is_uuid(b: &[u8]) -> bool {
    const DASHES: [usize; 4] = [8, 13, 18, 23];
    if b.len() != 36 {
        return false;
    }
    for (idx, byte) in b.iter().enumerate() {
        if DASHES.contains(&idx) {
            if *byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

#[cfg(unix)]
async fn kill_process(child: &mut tokio::process::Child, _pid: Option<u32>) {
    use std::process::Command;
    if let Some(pid) = child.id() {
        let _ = Command::new("kill").arg("-TERM").arg(pid.to_string()).status();
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
