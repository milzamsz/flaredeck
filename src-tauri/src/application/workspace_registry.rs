use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::application::workspace_service::discover;
use crate::error::{AppError, AppResult};

#[derive(Default, Serialize, Deserialize)]
struct Registry {
    #[serde(default)]
    workspaces: Vec<String>,
}

pub async fn register(path: &Path, registry_path: &Path) -> AppResult<()> {
    let mut registry = load(registry_path).await?;
    let path = path.to_string_lossy().into_owned();
    if !registry.workspaces.contains(&path) {
        registry.workspaces.push(path);
        registry.workspaces.sort();
    }
    if let Some(parent) = registry_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temporary = registry_path.with_extension("tmp");
    tokio::fs::write(&temporary, serde_json::to_vec(&registry)?).await?;
    tokio::fs::rename(temporary, registry_path).await?;
    Ok(())
}

pub async fn list(registry_path: &Path) -> AppResult<Vec<String>> {
    Ok(load(registry_path).await?.workspaces)
}

pub async fn resolve(registry_path: &Path, selector: &str) -> AppResult<PathBuf> {
    let mut matched = None;
    for path in list(registry_path).await? {
        let path = PathBuf::from(path);
        let Ok((root, manifest)) = discover(&path).await else {
            continue;
        };
        let id = manifest.project.id.as_deref();
        if id == Some(selector) || manifest.project.name == selector {
            if matched.is_some() {
                return Err(AppError::Other("workspace selector is ambiguous".into()));
            }
            matched = Some(root);
        }
    }
    matched.ok_or_else(|| AppError::Other("registered workspace not found".into()))
}

async fn load(path: &Path) -> AppResult<Registry> {
    match tokio::fs::read_to_string(path).await {
        Ok(raw) => Ok(serde_json::from_str(&raw)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Registry::default()),
        Err(error) => Err(error.into()),
    }
}
