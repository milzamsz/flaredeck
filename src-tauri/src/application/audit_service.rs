use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use serde::Serialize;
use tokio::io::AsyncWriteExt;

use crate::error::AppResult;

const MAX_AUDIT_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent<'a> {
    pub schema_version: u8,
    pub event_id: String,
    pub timestamp: String,
    pub operation: &'a str,
    pub result: &'a str,
    pub workspace_id: &'a str,
    pub session_id: &'a str,
    pub profile_id: &'a str,
    pub correlation_id: String,
    pub redaction_version: u8,
}

pub fn audit_path(state_path: &Path) -> PathBuf {
    let now = Utc::now();
    state_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("logs")
        .join(format!("audit-{:04}-{:02}.jsonl", now.year(), now.month()))
}

pub async fn append(path: &Path, event: AuditEvent<'_>) -> AppResult<()> {
    if tokio::fs::metadata(path)
        .await
        .map(|metadata| metadata.len() >= MAX_AUDIT_BYTES)
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
    file.write_all(&serde_json::to_vec(&event)?).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

pub fn session_event<'a>(
    operation: &'a str,
    result: &'a str,
    workspace_id: &'a str,
    session_id: &'a str,
    profile_id: &'a str,
) -> AuditEvent<'a> {
    AuditEvent {
        schema_version: 1,
        event_id: format!("evt_{}", uuid::Uuid::new_v4()),
        timestamp: Utc::now().to_rfc3339(),
        operation,
        result,
        workspace_id,
        session_id,
        profile_id,
        correlation_id: format!("corr_{}", uuid::Uuid::new_v4()),
        redaction_version: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{append, session_event};

    #[tokio::test]
    async fn appends_only_the_safe_event_shape() {
        let path =
            std::env::temp_dir().join(format!("flaredeck-audit-{}.jsonl", uuid::Uuid::new_v4()));
        append(
            &path,
            session_event(
                "session.start",
                "success",
                "workspace",
                "session",
                "profile",
            ),
        )
        .await
        .unwrap();
        let raw = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(raw.contains("session.start"));
        assert!(!raw.contains("TOKEN"));
        let _ = tokio::fs::remove_file(path).await;
    }
}
