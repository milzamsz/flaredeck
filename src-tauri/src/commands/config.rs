use std::path::{Path, PathBuf};

use crate::cloudflared::{cloudflared_dir, flaredeck_index_path};
use crate::error::{AppError, AppResult};
use crate::types::{CloudflaredConfig, ConfigPayload, IngressRule};

async fn resolve_config_path(profile_id: &str) -> AppResult<PathBuf> {
    let index_path = flaredeck_index_path()?;
    if index_path.exists() {
        let raw = tokio::fs::read_to_string(&index_path).await?;
        if let Ok(index) = serde_json::from_str::<crate::types::ProfileIndex>(&raw) {
            if let Some(profile) = index.profiles.iter().find(|p| p.id == profile_id) {
                return Ok(PathBuf::from(&profile.config_path));
            }
        }
    }
    Ok(cloudflared_dir()?.join(format!("{profile_id}.yml")))
}

#[tauri::command]
pub async fn config_get(profile_id: String) -> AppResult<ConfigPayload> {
    let path = resolve_config_path(&profile_id).await?;
    if !path.exists() {
        return Ok(ConfigPayload {
            path: path.to_string_lossy().to_string(),
            raw: String::new(),
            parsed: None,
        });
    }
    let raw = tokio::fs::read_to_string(&path).await?;
    let parsed = if raw.trim().is_empty() {
        None
    } else {
        serde_yaml::from_str::<CloudflaredConfig>(&raw).ok()
    };
    Ok(ConfigPayload {
        path: path.to_string_lossy().to_string(),
        raw,
        parsed,
    })
}

#[tauri::command]
pub async fn config_save(profile_id: String, raw: String) -> AppResult<ConfigPayload> {
    let path = resolve_config_path(&profile_id).await?;
    let dir = path
        .parent()
        .ok_or_else(|| AppError::Other("config path has no parent".to_string()))?;
    tokio::fs::create_dir_all(dir).await?;

    if path.exists() {
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
        let backup_name = format!(
            "{}.bak.{}",
            path.file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "config".into()),
            ts
        );
        let backup_path = dir.join(backup_name);
        tokio::fs::copy(&path, &backup_path).await?;
        prune_old_backups(dir, &path).await?;
    }

    tokio::fs::write(&path, &raw).await?;
    let parsed = if raw.trim().is_empty() {
        None
    } else {
        serde_yaml::from_str::<CloudflaredConfig>(&raw).ok()
    };
    Ok(ConfigPayload {
        path: path.to_string_lossy().to_string(),
        raw,
        parsed,
    })
}

pub async fn write_initial_config(
    path: &Path,
    uuid: &str,
    credentials_file: &str,
) -> AppResult<()> {
    let dir = path
        .parent()
        .ok_or_else(|| AppError::Other("config path has no parent".to_string()))?;
    tokio::fs::create_dir_all(dir).await?;

    let config = CloudflaredConfig {
        tunnel: Some(uuid.to_string()),
        credentials_file: Some(credentials_file.to_string()),
        ingress: Some(vec![IngressRule {
            hostname: None,
            path: None,
            service: "http_status:404".to_string(),
            origin_request: None,
        }]),
        extras: serde_yaml::Mapping::new(),
    };
    let raw = serde_yaml::to_string(&config)?;
    tokio::fs::write(path, raw).await?;
    Ok(())
}

async fn prune_old_backups(dir: &std::path::Path, source: &std::path::Path) -> AppResult<()> {
    let source_name = source
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let prefix = format!("{source_name}.bak.");

    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut backups: Vec<(String, PathBuf)> = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) {
            backups.push((name, entry.path()));
        }
    }

    backups.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, path) in backups.into_iter().skip(10) {
        let _ = tokio::fs::remove_file(path).await;
    }
    Ok(())
}
