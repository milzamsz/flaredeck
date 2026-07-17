use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::application::workspace_service::parse_manifest;
use crate::error::AppResult;

/// A validated manifest's exact bytes are the Phase 4 trust input.
/// Canonical serialization replaces this conservative form only when every
/// security-relevant default has an explicit normalized representation.
pub fn fingerprint(raw: &str) -> AppResult<String> {
    parse_manifest(raw)?;
    let digest = Sha256::digest(raw.as_bytes());
    Ok(format!("sha256:{digest:x}"))
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrustStore {
    #[serde(default = "trust_schema_version")]
    schema_version: u8,
    #[serde(default)]
    approvals: BTreeMap<String, String>,
}

fn trust_schema_version() -> u8 {
    1
}

/// Fail closed: missing, corrupt, or mismatched approval is never trusted.
pub async fn is_approved(path: &Path, workspace_root: &Path, fingerprint: &str) -> bool {
    tokio::fs::read_to_string(path)
        .await
        .ok()
        .and_then(|raw| serde_json::from_str::<TrustStore>(&raw).ok())
        .and_then(|store| store.approvals.get(&workspace_key(workspace_root)).cloned())
        .is_some_and(|approved| approved == fingerprint)
}

pub async fn has_approval(path: &Path, workspace_root: &Path) -> bool {
    tokio::fs::read_to_string(path)
        .await
        .ok()
        .and_then(|raw| serde_json::from_str::<TrustStore>(&raw).ok())
        .is_some_and(|store| store.approvals.contains_key(&workspace_key(workspace_root)))
}

/// Desktop approval adapter only; CLI and MCP must never call this function.
pub async fn save_desktop_approval(
    path: &Path,
    workspace_root: &Path,
    fingerprint: String,
) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| crate::error::AppError::Other("trust path has no parent".into()))?;
    tokio::fs::create_dir_all(parent).await?;
    let mut store = tokio::fs::read_to_string(path)
        .await
        .ok()
        .and_then(|raw| serde_json::from_str::<TrustStore>(&raw).ok())
        .unwrap_or_default();
    store
        .approvals
        .insert(workspace_key(workspace_root), fingerprint);
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, serde_json::to_vec(&store)?).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

fn workspace_key(workspace_root: &Path) -> String {
    workspace_root.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::{fingerprint, is_approved, save_desktop_approval};
    const MANIFEST: &str = "version: 1\nproject: { name: app }\nprofile: { id: profile-1 }\nruntime: { executable: cargo, args: [run] }\nready: { type: tcp, host: 127.0.0.1, port: 3000 }\nexposure: { routes: [{ hostname: app.example.com, service: http://127.0.0.1:3000 }] }\n";
    #[test]
    fn changes_when_runtime_changes() {
        assert_ne!(
            fingerprint(MANIFEST).unwrap(),
            fingerprint(&MANIFEST.replace("[run]", "[test]")).unwrap()
        );
    }
    #[tokio::test]
    async fn corrupt_or_mismatched_approval_is_untrusted() {
        let path =
            std::env::temp_dir().join(format!("flaredeck-trust-{}.json", uuid::Uuid::new_v4()));
        let root =
            std::env::temp_dir().join(format!("flaredeck-trust-root-{}", uuid::Uuid::new_v4()));
        let fingerprint = fingerprint(MANIFEST).unwrap();
        assert!(!is_approved(&path, &root, &fingerprint).await);
        tokio::fs::write(&path, "not-json").await.unwrap();
        assert!(!is_approved(&path, &root, &fingerprint).await);
        save_desktop_approval(&path, &root, fingerprint.clone())
            .await
            .unwrap();
        assert!(is_approved(&path, &root, &fingerprint).await);
        assert!(!is_approved(&path, &root, "sha256:other").await);
        let _ = tokio::fs::remove_file(path).await;
    }
}
