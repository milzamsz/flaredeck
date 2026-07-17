use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::application::workspace_service::{
    resolve_working_directory, Environment, WorkspaceManifest,
};
use crate::error::AppResult;
use std::path::Path;
use std::process::Stdio;

const MAX_LOG_LINES: usize = 200;
const MAX_LOG_LINE_BYTES: usize = 4 * 1024;
const MAX_LOG_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeLogLine {
    pub stream: String,
    pub line: String,
}

#[derive(Default)]
struct RuntimeLogs {
    lines: VecDeque<RuntimeLogLine>,
}

pub struct RuntimeProcess {
    pub child: tokio::process::Child,
    logs: Arc<Mutex<RuntimeLogs>>,
}

pub async fn spawn(root: &Path, manifest: &WorkspaceManifest) -> AppResult<RuntimeProcess> {
    spawn_with_log_path(root, manifest, None).await
}

pub async fn spawn_with_log_path(
    root: &Path,
    manifest: &WorkspaceManifest,
    log_path: Option<PathBuf>,
) -> AppResult<RuntimeProcess> {
    let runtime = &manifest.runtime;
    let cwd = resolve_working_directory(root, &runtime.working_directory)?;
    let mut command = tokio::process::Command::new(&runtime.executable);
    command
        .args(&runtime.args)
        .current_dir(cwd)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_environment(&mut command, manifest.environment.as_ref());
    let mut child = command.spawn()?;
    let logs = Arc::new(Mutex::new(RuntimeLogs::default()));
    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(logs.clone(), "stdout", stdout, log_path.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(logs.clone(), "stderr", stderr, log_path);
    }
    Ok(RuntimeProcess { child, logs })
}

pub fn logs(process: &RuntimeProcess, tail: usize) -> Vec<RuntimeLogLine> {
    let Ok(logs) = process.logs.lock() else {
        return Vec::new();
    };
    let skip = logs.lines.len().saturating_sub(tail.min(MAX_LOG_LINES));
    logs.lines.iter().skip(skip).cloned().collect()
}

fn apply_environment(command: &mut tokio::process::Command, environment: Option<&Environment>) {
    let Some(environment) = environment else {
        return;
    };
    if let Some(names) = &environment.passthrough {
        for name in names {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
    }
    if let Some(values) = &environment.values {
        command.envs(values);
    }
}

fn spawn_log_reader<R>(
    logs: Arc<Mutex<RuntimeLogs>>,
    stream: &'static str,
    reader: R,
    log_path: Option<PathBuf>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(mut line)) = lines.next_line().await {
            line.truncate(MAX_LOG_LINE_BYTES);
            let line = redact_log_line(line);
            let event = RuntimeLogLine {
                stream: stream.into(),
                line,
            };
            {
                let Ok(mut logs) = logs.lock() else {
                    return;
                };
                if logs.lines.len() == MAX_LOG_LINES {
                    logs.lines.pop_front();
                }
                logs.lines.push_back(event.clone());
            }
            if let Some(path) = &log_path {
                let _ = append_log(path, &event).await;
            }
        }
    });
}

async fn append_log(path: &std::path::Path, line: &RuntimeLogLine) -> AppResult<()> {
    if tokio::fs::metadata(path)
        .await
        .map(|metadata| metadata.len() >= MAX_LOG_FILE_BYTES)
        .unwrap_or(false)
    {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(&serde_json::to_vec(line)?).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

pub(crate) fn redact_log_line(line: String) -> String {
    let upper = line.to_ascii_uppercase();
    if [
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "PRIVATE_KEY",
        "API_KEY",
        "AUTHORIZATION",
        "COOKIE",
    ]
    .iter()
    .any(|needle| upper.contains(needle))
    {
        "[redacted runtime output]".into()
    } else {
        line
    }
}
pub async fn stop(process: &mut RuntimeProcess) {
    #[cfg(unix)]
    if let Some(pid) = process.child.id() {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
    #[cfg(windows)]
    if let Some(pid) = process.child.id() {
        let _ = std::process::Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .status();
    }
    let _ = process.child.start_kill();
    let _ = process.child.wait().await;
}

#[cfg(test)]
mod tests {
    use super::{logs, spawn, stop};
    use crate::application::workspace_service::{
        Exposure, ProfileRef, Project, Readiness, Route, Runtime, WorkspaceManifest,
    };

    #[tokio::test]
    async fn spawns_and_stops_directly() {
        let manifest = manifest();
        let mut process = spawn(std::path::Path::new("."), &manifest).await.unwrap();
        stop(&mut process).await;
        assert!(logs(&process, usize::MAX).len() <= super::MAX_LOG_LINES);
    }

    #[test]
    fn redacts_secret_like_output() {
        assert_eq!(
            super::redact_log_line("CANARY_SECRET=value".into()),
            "[redacted runtime output]"
        );
    }

    fn manifest() -> WorkspaceManifest {
        WorkspaceManifest {
            version: 1,
            project: Project {
                name: "test".into(),
                id: None,
            },
            profile: ProfileRef {
                id: Some("profile".into()),
                name: None,
            },
            runtime: Runtime {
                executable: std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                args: vec!["--help".into()],
                working_directory: ".".into(),
            },
            ready: Readiness::Tcp {
                host: "127.0.0.1".into(),
                port: 1,
                interval_milliseconds: 100,
                timeout_seconds: 1,
            },
            exposure: Exposure {
                routes: vec![Route {
                    hostname: "test.example.com".into(),
                    service: "http://127.0.0.1:1".into(),
                    path: None,
                    mode: None,
                }],
            },
            lifecycle: None,
            environment: None,
        }
    }
}
