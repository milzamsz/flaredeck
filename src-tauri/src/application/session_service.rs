use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

use crate::application::audit_service::{append as append_audit, audit_path, session_event};
use crate::application::runtime_service::{
    spawn, spawn_with_log_path, stop as stop_runtime, RuntimeLogLine, RuntimeProcess,
};
use crate::application::trust_service::fingerprint;
use crate::application::tunnel_service::{observe as observe_tunnel, start as start_tunnel};
use crate::application::workspace_service::authorize_start;
use crate::error::{AppError, AppResult};
use crate::state::RuntimeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Stopped,
    Starting,
    Healthy,
    Failed,
    Stopping,
    CleanupIncomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: String,
    pub workspace_root: String,
    pub workspace_id: String,
    pub profile_id: String,
    pub fingerprint: String,
    pub state: SessionState,
    pub runtime_pid: Option<u32>,
    pub runtime_started_at_seconds: Option<u64>,
    pub runtime_executable: String,
    pub runtime_log_path: Option<String>,
    pub runtime_owned: bool,
    #[serde(default = "default_true")]
    pub stop_runtime_on_session_stop: bool,
    pub tunnel_started_by_session: bool,
    #[serde(default)]
    pub tunnel_pid: Option<u32>,
    #[serde(default)]
    pub tunnel_started_at_seconds: Option<u64>,
    #[serde(default)]
    pub tunnel_executable: Option<String>,
    #[serde(default = "default_true")]
    pub stop_tunnel_on_session_stop: bool,
    #[serde(default = "default_true")]
    pub remove_temporary_routes: bool,
    pub public_urls: Vec<String>,
    pub started_at: DateTime<Utc>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionStore {
    #[serde(default = "session_schema_version")]
    schema_version: u8,
    #[serde(default)]
    sessions: Vec<SessionRecord>,
}

fn session_schema_version() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

pub fn start(state: SessionState) -> Result<SessionState, &'static str> {
    match state {
        SessionState::Stopped | SessionState::Failed => Ok(SessionState::Starting),
        SessionState::Healthy => Ok(SessionState::Healthy),
        _ => Err("session start already in progress"),
    }
}
pub fn stop(state: SessionState) -> SessionState {
    match state {
        SessionState::Stopped => SessionState::Stopped,
        _ => SessionState::Stopping,
    }
}

pub async fn start_approved_runtime(
    workspace_root: &Path,
    approval_path: &Path,
) -> AppResult<RuntimeProcess> {
    let (root, manifest) = authorize_start(workspace_root, approval_path).await?;
    let mut process = spawn(&root, &manifest).await?;
    if let Err(error) =
        crate::application::health_service::wait_for_readiness(&manifest.ready).await
    {
        stop_runtime(&mut process).await;
        return Err(error);
    }
    Ok(process)
}

pub async fn start_session(
    workspace_root: &Path,
    approval_path: &Path,
    store_path: &Path,
) -> AppResult<SessionRecord> {
    start_session_inner(workspace_root, approval_path, store_path, None).await
}

pub async fn start_session_cancellable(
    workspace_root: &Path,
    approval_path: &Path,
    store_path: &Path,
    cancellation: &mut tokio::sync::watch::Receiver<bool>,
    maximum_wait: std::time::Duration,
) -> AppResult<SessionRecord> {
    start_session_inner(
        workspace_root,
        approval_path,
        store_path,
        Some((cancellation, maximum_wait)),
    )
    .await
}

async fn start_session_inner(
    workspace_root: &Path,
    approval_path: &Path,
    store_path: &Path,
    mut cancellation: Option<(&mut tokio::sync::watch::Receiver<bool>, std::time::Duration)>,
) -> AppResult<SessionRecord> {
    let (root, manifest) = authorize_start(workspace_root, approval_path).await?;
    let raw = tokio::fs::read_to_string(root.join(".flaredeck/project.yaml")).await?;
    let fingerprint = fingerprint(&raw)?;
    let mut store = load_store(store_path).await?;
    if let Some(record) = store
        .sessions
        .iter()
        .find(|record| {
            record.workspace_root == root.to_string_lossy() && record.state == SessionState::Healthy
        })
        .cloned()
    {
        if (!record.runtime_owned || runtime_matches(&record))
            && (!record.tunnel_started_by_session || tunnel_matches(&record))
        {
            return Ok(record);
        }
    }

    let retained_runtime = store
        .sessions
        .iter()
        .find(|record| {
            record.workspace_root == root.to_string_lossy()
                && record.state == SessionState::Stopped
                && record.runtime_owned
                && !record.stop_runtime_on_session_stop
                && runtime_matches(record)
        })
        .cloned();

    let session_id = format!("ses_{}", uuid::Uuid::new_v4());
    let runtime_log_path = retained_runtime
        .as_ref()
        .and_then(|record| record.runtime_log_path.as_ref().map(PathBuf::from))
        .or_else(|| {
            store_path.parent().map(|parent| {
                parent
                    .join("logs")
                    .join(format!("runtime-{session_id}.jsonl"))
            })
        });
    let start_runtime = manifest
        .lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.start_runtime)
        .unwrap_or(true);
    let executable = if let Some(record) = &retained_runtime {
        record.runtime_executable.clone()
    } else if start_runtime {
        executable_identity(&root, &manifest.runtime.executable)?
            .to_string_lossy()
            .into_owned()
    } else {
        String::new()
    };
    let mut process = if start_runtime && retained_runtime.is_none() {
        Some(spawn_with_log_path(&root, &manifest, runtime_log_path.clone()).await?)
    } else {
        None
    };
    let readiness = crate::application::health_service::wait_for_readiness(&manifest.ready);
    let readiness_result = match cancellation.as_mut() {
        Some((cancelled, maximum_wait)) => tokio::select! {
            result = readiness => result,
            _ = cancelled.changed() => Err(AppError::Other("operation cancelled".into())),
            _ = tokio::time::sleep(*maximum_wait) => Err(AppError::Other("operation timed out".into())),
        },
        None => readiness.await,
    };
    if let Err(error) = readiness_result {
        if let Some(process) = process.as_mut() {
            stop_runtime(process).await;
        }
        return Err(error);
    }
    let pid = process
        .as_ref()
        .and_then(|process| process.child.id())
        .or_else(|| {
            retained_runtime
                .as_ref()
                .and_then(|record| record.runtime_pid)
        });
    let runtime_started_at_seconds = retained_runtime
        .as_ref()
        .and_then(|record| record.runtime_started_at_seconds)
        .or_else(|| pid.and_then(process_start_time));
    let ensure_tunnel = manifest
        .lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.ensure_tunnel)
        .unwrap_or(true);
    let stop_tunnel_on_session_stop = manifest
        .lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.stop_tunnel_if_started_by_session)
        .unwrap_or(true);
    let stop_runtime_on_session_stop = manifest
        .lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.stop_runtime_on_session_stop)
        .unwrap_or(true);
    let remove_temporary_routes = manifest
        .lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.remove_temporary_routes)
        .unwrap_or(true);
    let profile_id = manifest
        .profile
        .id
        .clone()
        .or(manifest.profile.name.clone())
        .unwrap_or_default();
    let (tunnel_started_by_session, tunnel_pid, tunnel_started_at_seconds, tunnel_executable) =
        if ensure_tunnel {
            if let Err(error) = crate::application::route_service::verify_persistent_routes(
                &profile_id,
                &manifest.exposure.routes,
            )
            .await
            {
                if let Some(process) = process.as_mut() {
                    stop_runtime(process).await;
                }
                return Err(error);
            }
            match observe_tunnel(&profile_id).await? {
                Some(tunnel) => (
                    false,
                    Some(tunnel.pid),
                    Some(tunnel.started_at_seconds),
                    Some(tunnel.executable),
                ),
                None => {
                    let state = RuntimeState::new();
                    let status = match start_tunnel(&state, profile_id.clone(), |_| {}).await {
                        Ok(status) => status,
                        Err(error) => {
                            if let Some(process) = process.as_mut() {
                                stop_runtime(process).await;
                            }
                            return Err(error);
                        }
                    };
                    let tunnel_pid = status.pid;
                    (
                        true,
                        tunnel_pid,
                        tunnel_pid.and_then(process_start_time),
                        Some(
                            crate::cloudflared::ensure_cloudflared()?
                                .to_string_lossy()
                                .into_owned(),
                        ),
                    )
                }
            }
        } else {
            (false, None, None, None)
        };
    if ensure_tunnel {
        if let Err(error) = crate::application::temporary_route_service::create_for_session(
            &root,
            approval_path,
            store_path,
            &session_id,
        )
        .await
        {
            if let Some(process) = process.as_mut() {
                stop_runtime(process).await;
            }
            if tunnel_started_by_session {
                if let Some(pid) = tunnel_pid {
                    stop_owned_process_tree(pid);
                }
            }
            return Err(error);
        }
    }
    let record = SessionRecord {
        id: session_id,
        workspace_root: root.to_string_lossy().into_owned(),
        workspace_id: manifest.project.id.unwrap_or(manifest.project.name),
        profile_id,
        fingerprint,
        state: SessionState::Healthy,
        runtime_pid: pid,
        runtime_started_at_seconds,
        runtime_executable: executable,
        runtime_log_path: runtime_log_path.map(|path| path.to_string_lossy().into_owned()),
        runtime_owned: start_runtime,
        stop_runtime_on_session_stop,
        tunnel_started_by_session,
        tunnel_pid,
        tunnel_started_at_seconds,
        tunnel_executable,
        stop_tunnel_on_session_stop,
        remove_temporary_routes,
        public_urls: manifest
            .exposure
            .routes
            .into_iter()
            .map(|route| format!("https://{}", route.hostname))
            .collect(),
        started_at: Utc::now(),
    };
    store
        .sessions
        .retain(|entry| entry.workspace_root != record.workspace_root);
    store.sessions.push(record.clone());
    if let Err(error) = save_store(store_path, &store).await {
        if let Some(process) = process.as_mut() {
            stop_runtime(process).await;
        }
        return Err(error);
    }
    if let Err(error) = append_audit(
        &audit_path(store_path),
        session_event(
            "session.start",
            "success",
            &record.workspace_id,
            &record.id,
            &record.profile_id,
        ),
    )
    .await
    {
        if let Some(process) = process.as_mut() {
            stop_runtime(process).await;
        }
        if let Some(persisted) = store
            .sessions
            .iter_mut()
            .find(|entry| entry.id == record.id)
        {
            persisted.state = SessionState::Failed;
        }
        let _ = save_store(store_path, &store).await;
        return Err(error);
    }
    Ok(record)
}

pub async fn session_status(store_path: &Path, selector: &str) -> AppResult<Option<SessionRecord>> {
    Ok(load_store(store_path)
        .await?
        .sessions
        .into_iter()
        .find(|record| record.id == selector || record.workspace_id == selector))
}

pub async fn session_logs(
    store_path: &Path,
    selector: &str,
    tail: usize,
) -> AppResult<Vec<RuntimeLogLine>> {
    let Some(record) = session_status(store_path, selector).await? else {
        return Err(AppError::Other("session not found".into()));
    };
    let Some(path) = record.runtime_log_path else {
        return Ok(Vec::new());
    };
    let raw = match tokio::fs::read_to_string(path).await {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    Ok(raw
        .lines()
        .rev()
        .take(tail.min(200))
        .filter_map(|line| serde_json::from_str::<RuntimeLogLine>(line).ok())
        .map(|mut entry| {
            entry.line = crate::application::runtime_service::redact_log_line(entry.line);
            entry
        })
        .collect::<Vec<RuntimeLogLine>>()
        .into_iter()
        .rev()
        .collect())
}

pub async fn stop_session(store_path: &Path, selector: &str) -> AppResult<SessionRecord> {
    let mut store = load_store(store_path).await?;
    let record = store
        .sessions
        .iter_mut()
        .find(|record| record.id == selector || record.workspace_id == selector)
        .ok_or_else(|| AppError::Other("session not found".into()))?;
    if record.state == SessionState::Stopped {
        return Ok(record.clone());
    }
    if (record.runtime_owned && record.stop_runtime_on_session_stop && !runtime_matches(record))
        || (record.tunnel_started_by_session
            && record.stop_tunnel_on_session_stop
            && !tunnel_matches(record))
    {
        record.state = SessionState::CleanupIncomplete;
        let result = record.clone();
        save_store(store_path, &store).await?;
        append_audit(
            &audit_path(store_path),
            session_event(
                "session.stop",
                "cleanup_incomplete",
                &result.workspace_id,
                &result.id,
                &result.profile_id,
            ),
        )
        .await?;
        return Ok(result);
    }
    let temporary_cleanup = if record.remove_temporary_routes {
        crate::application::temporary_route_service::cleanup_session(
            &crate::application::temporary_route_service::route_store_path(store_path),
            &record.id,
        )
        .await
    } else {
        Ok(Vec::new())
    };
    let temporary_cleanup_incomplete = match &temporary_cleanup {
        Ok(routes) => routes.iter().any(|route| {
            route.state
                == crate::application::temporary_route_service::TemporaryRouteState::CleanupIncomplete
        }),
        Err(_) => true,
    };
    if record.stop_runtime_on_session_stop {
        if let Some(pid) = record.runtime_pid {
            stop_owned_process_tree(pid);
        }
        record.runtime_pid = None;
    }
    if record.tunnel_started_by_session && record.stop_tunnel_on_session_stop {
        if let Some(pid) = record.tunnel_pid {
            stop_owned_process_tree(pid);
        }
    }
    record.tunnel_pid = None;
    record.state = if temporary_cleanup_incomplete {
        SessionState::CleanupIncomplete
    } else {
        SessionState::Stopped
    };
    let result = record.clone();
    save_store(store_path, &store).await?;
    append_audit(
        &audit_path(store_path),
        session_event(
            "session.stop",
            if temporary_cleanup_incomplete {
                "cleanup_incomplete"
            } else {
                "success"
            },
            &result.workspace_id,
            &result.id,
            &result.profile_id,
        ),
    )
    .await?;
    Ok(result)
}

async fn load_store(path: &Path) -> AppResult<SessionStore> {
    match tokio::fs::read_to_string(path).await {
        Ok(raw) => Ok(serde_json::from_str(&raw)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(SessionStore::default()),
        Err(error) => Err(error.into()),
    }
}

async fn save_store(path: &Path, store: &SessionStore) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Other("session path has no parent".into()))?;
    tokio::fs::create_dir_all(parent).await?;
    // ponytail: one JSON file is sufficient for the MVP; add a database only if concurrent writers appear.
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, serde_json::to_vec(store)?).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

fn executable_identity(root: &Path, executable: &str) -> AppResult<PathBuf> {
    let executable = Path::new(executable);
    let resolved = if executable.is_absolute() {
        executable.to_path_buf()
    } else if executable.components().count() > 1 {
        root.join(executable)
    } else {
        which::which(executable).map_err(|error| AppError::Other(error.to_string()))?
    };
    Ok(resolved.canonicalize()?)
}

pub(crate) fn process_start_time(pid: u32) -> Option<u64> {
    let system = System::new_all();
    system
        .process(Pid::from_u32(pid))
        .map(|process| process.start_time())
}

fn runtime_matches(record: &SessionRecord) -> bool {
    matches_process(
        record.runtime_pid,
        record.runtime_started_at_seconds,
        Some(&record.runtime_executable),
    )
}

fn tunnel_matches(record: &SessionRecord) -> bool {
    matches_process(
        record.tunnel_pid,
        record.tunnel_started_at_seconds,
        record.tunnel_executable.as_deref(),
    )
}

pub(crate) fn matches_process(
    pid: Option<u32>,
    started_at: Option<u64>,
    executable: Option<&str>,
) -> bool {
    let (Some(pid), Some(started_at), Some(executable)) = (pid, started_at, executable) else {
        return false;
    };
    let system = System::new_all();
    let Some(process) = system.process(Pid::from_u32(pid)) else {
        return false;
    };
    process.start_time() == started_at
        && process
            .exe()
            .is_some_and(|path| path == Path::new(executable))
}

pub(crate) fn stop_owned_process_tree(pid: u32) {
    let system = System::new_all();
    let mut pending = vec![Pid::from_u32(pid)];
    let mut descendants = Vec::new();
    while let Some(parent) = pending.pop() {
        for (child_pid, child) in system.processes() {
            if child.parent() == Some(parent) {
                pending.push(*child_pid);
                descendants.push(*child_pid);
            }
        }
    }
    #[cfg(unix)]
    for target in descendants
        .into_iter()
        .rev()
        .chain(std::iter::once(Pid::from_u32(pid)))
    {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &target.as_u32().to_string()])
            .status();
    }
    #[cfg(windows)]
    let _ = std::process::Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .status();
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::trust_service::{fingerprint, save_desktop_approval};

    #[test]
    fn start_and_stop_are_idempotent() {
        assert_eq!(start(SessionState::Healthy), Ok(SessionState::Healthy));
        assert_eq!(stop(SessionState::Stopped), SessionState::Stopped);
    }

    #[tokio::test]
    async fn untrusted_workspace_cannot_reach_runtime_spawn() {
        let root = std::env::temp_dir().join(format!("flaredeck-session-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(root.join(".flaredeck"))
            .await
            .unwrap();
        let raw = manifest("missing-flaredeck-runtime", 1);
        tokio::fs::write(root.join(".flaredeck/project.yaml"), &raw)
            .await
            .unwrap();
        let approval = root.join("approval.json");
        let error = match start_approved_runtime(&root, &approval).await {
            Ok(_) => panic!("untrusted workspace started a runtime"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("desktop approval"));
        save_desktop_approval(&approval, &root, fingerprint(&raw).unwrap())
            .await
            .unwrap();
        assert!(start_approved_runtime(&root, &approval).await.is_err());
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn approved_runtime_is_stopped_when_readiness_fails() {
        let root = std::env::temp_dir().join(format!("flaredeck-session-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(root.join(".flaredeck"))
            .await
            .unwrap();
        let raw = manifest(&std::env::current_exe().unwrap().to_string_lossy(), 1);
        tokio::fs::write(root.join(".flaredeck/project.yaml"), &raw)
            .await
            .unwrap();
        let approval = root.join("approval.json");
        save_desktop_approval(&approval, &root, fingerprint(&raw).unwrap())
            .await
            .unwrap();
        assert!(start_approved_runtime(&root, &approval).await.is_err());
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn repeated_stop_is_idempotent() {
        let path = session_store_path();
        save_store(
            &path,
            &SessionStore {
                schema_version: 1,
                sessions: vec![record(SessionState::Stopped, None)],
            },
        )
        .await
        .unwrap();
        let stopped = stop_session(&path, "ses_test").await.unwrap();
        assert_eq!(stopped.state, SessionState::Stopped);
        let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
    }

    #[tokio::test]
    async fn uncertain_pid_is_not_killed() {
        let path = session_store_path();
        save_store(
            &path,
            &SessionStore {
                schema_version: 1,
                sessions: vec![record(SessionState::Healthy, Some(u32::MAX))],
            },
        )
        .await
        .unwrap();
        let stopped = stop_session(&path, "ses_test").await.unwrap();
        assert_eq!(stopped.state, SessionState::CleanupIncomplete);
        let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
    }

    #[tokio::test]
    async fn lifecycle_can_leave_an_owned_runtime_running() {
        let path = session_store_path();
        let mut session = record(SessionState::Healthy, Some(u32::MAX));
        session.stop_runtime_on_session_stop = false;
        save_store(
            &path,
            &SessionStore {
                schema_version: 1,
                sessions: vec![session],
            },
        )
        .await
        .unwrap();
        let stopped = stop_session(&path, "ses_test").await.unwrap();
        assert_eq!(stopped.state, SessionState::Stopped);
        assert_eq!(stopped.runtime_pid, Some(u32::MAX));
        let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
    }

    #[tokio::test]
    async fn observed_tunnel_is_never_stopped_by_session_cleanup() {
        let path = session_store_path();
        let mut session = record(SessionState::Healthy, None);
        session.runtime_owned = false;
        session.tunnel_pid = Some(u32::MAX);
        session.tunnel_started_at_seconds = Some(1);
        session.tunnel_executable = Some("/missing".into());
        session.tunnel_started_by_session = false;
        save_store(
            &path,
            &SessionStore {
                schema_version: 1,
                sessions: vec![session],
            },
        )
        .await
        .unwrap();
        let stopped = stop_session(&path, "ses_test").await.unwrap();
        assert_eq!(stopped.state, SessionState::Stopped);
        let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
    }

    #[tokio::test]
    async fn logs_are_bounded_and_loaded_from_the_session_record() {
        let path = session_store_path();
        let log_path = path.parent().unwrap().join("logs/runtime.jsonl");
        tokio::fs::create_dir_all(log_path.parent().unwrap())
            .await
            .unwrap();
        let raw = (0..201)
            .map(|index| format!("{{\"stream\":\"stdout\",\"line\":\"{index}\"}}"))
            .collect::<Vec<_>>()
            .join("\n");
        tokio::fs::write(&log_path, raw).await.unwrap();
        let mut record = record(SessionState::Healthy, None);
        record.runtime_log_path = Some(log_path.to_string_lossy().into_owned());
        save_store(
            &path,
            &SessionStore {
                schema_version: 1,
                sessions: vec![record],
            },
        )
        .await
        .unwrap();
        let logs = session_logs(&path, "ses_test", usize::MAX).await.unwrap();
        assert_eq!(logs.len(), 200);
        assert_eq!(logs.first().unwrap().line, "1");
        let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
    }

    #[tokio::test]
    async fn approved_workspace_start_persists_a_session() {
        let root = std::env::temp_dir().join(format!("flaredeck-session-{}", uuid::Uuid::new_v4()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::fs::create_dir_all(root.join(".flaredeck"))
            .await
            .unwrap();
        let raw = manifest_with_port(&std::env::current_exe().unwrap().to_string_lossy(), port);
        tokio::fs::write(root.join(".flaredeck/project.yaml"), &raw)
            .await
            .unwrap();
        let approval = root.join("trust.json");
        let store = root.join("sessions.json");
        save_desktop_approval(&approval, &root, fingerprint(&raw).unwrap())
            .await
            .unwrap();
        let session = start_session(&root, &approval, &store).await.unwrap();
        let loaded = session_status(&store, &session.id).await.unwrap().unwrap();
        assert_eq!(loaded.id, session.id);
        drop(listener);
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn external_runtime_is_observed_but_not_owned() {
        let root = std::env::temp_dir().join(format!("flaredeck-session-{}", uuid::Uuid::new_v4()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::fs::create_dir_all(root.join(".flaredeck"))
            .await
            .unwrap();
        let raw = manifest_with_port(&std::env::current_exe().unwrap().to_string_lossy(), port)
            .replace(
                "lifecycle: { ensureTunnel: false }",
                "lifecycle: { startRuntime: false, ensureTunnel: false }",
            );
        tokio::fs::write(root.join(".flaredeck/project.yaml"), &raw)
            .await
            .unwrap();
        let approval = root.join("trust.json");
        let store = root.join("sessions.json");
        save_desktop_approval(&approval, &root, fingerprint(&raw).unwrap())
            .await
            .unwrap();
        let session = start_session(&root, &approval, &store).await.unwrap();
        assert!(!session.runtime_owned);
        assert_eq!(session.runtime_pid, None);
        let repeated = start_session(&root, &approval, &store).await.unwrap();
        assert_eq!(repeated.id, session.id);
        drop(listener);
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    fn session_store_path() -> std::path::PathBuf {
        std::env::temp_dir()
            .join(format!("flaredeck-session-store-{}", uuid::Uuid::new_v4()))
            .join("active-sessions.json")
    }

    fn record(state: SessionState, pid: Option<u32>) -> SessionRecord {
        SessionRecord {
            id: "ses_test".into(),
            workspace_root: "/workspace".into(),
            workspace_id: "workspace".into(),
            profile_id: "profile".into(),
            fingerprint: "sha256:test".into(),
            state,
            runtime_pid: pid,
            runtime_started_at_seconds: Some(1),
            runtime_executable: "/missing".into(),
            runtime_log_path: None,
            runtime_owned: true,
            stop_runtime_on_session_stop: true,
            tunnel_started_by_session: false,
            tunnel_pid: None,
            tunnel_started_at_seconds: None,
            tunnel_executable: None,
            stop_tunnel_on_session_stop: true,
            remove_temporary_routes: true,
            public_urls: vec![],
            started_at: Utc::now(),
        }
    }

    #[test]
    fn additive_session_fields_migrate_from_profile_only_release_state() {
        let old = serde_json::json!({
            "id": "ses_old",
            "workspaceRoot": "/workspace",
            "workspaceId": "workspace",
            "profileId": "profile",
            "fingerprint": "sha256:test",
            "state": "healthy",
            "runtimePid": null,
            "runtimeStartedAtSeconds": null,
            "runtimeExecutable": "",
            "runtimeLogPath": null,
            "runtimeOwned": false,
            "tunnelStartedBySession": false,
            "publicUrls": [],
            "startedAt": "2026-01-01T00:00:00Z"
        });
        let migrated: SessionRecord = serde_json::from_value(old).unwrap();
        assert!(migrated.stop_runtime_on_session_stop);
        assert!(migrated.stop_tunnel_on_session_stop);
        assert!(migrated.remove_temporary_routes);
        assert!(migrated.tunnel_pid.is_none());
    }

    fn manifest(executable: &str, timeout_seconds: u64) -> String {
        format!(
            "version: 1\nproject: {{ name: app }}\nprofile: {{ id: profile-1 }}\nruntime: {{ executable: {executable:?}, args: [--help] }}\nready: {{ type: tcp, host: 127.0.0.1, port: 1, intervalMilliseconds: 100, timeoutSeconds: {timeout_seconds} }}\nexposure: {{ routes: [{{ hostname: app.example.com, service: http://127.0.0.1:3000 }}] }}\n"
        )
    }

    fn manifest_with_port(executable: &str, port: u16) -> String {
        format!(
            "version: 1\nproject: {{ name: app }}\nprofile: {{ id: profile-1 }}\nruntime: {{ executable: {executable:?}, args: [--help] }}\nready: {{ type: tcp, host: 127.0.0.1, port: {port}, intervalMilliseconds: 100, timeoutSeconds: 1 }}\nexposure: {{ routes: [{{ hostname: app.example.com, service: http://127.0.0.1:{port} }}] }}\nlifecycle: {{ ensureTunnel: false }}\n"
        )
    }
}
