use std::path::PathBuf;

use crate::error::{AppError, AppResult};

pub fn default_state_dir() -> AppResult<PathBuf> {
    dirs::config_dir()
        .map(|path| path.join("dev.flaredeck.desktop"))
        .ok_or(AppError::NoHomeDir)
}

pub fn trust_store_path() -> AppResult<PathBuf> {
    Ok(default_state_dir()?.join("trust-approvals.json"))
}

pub fn session_store_path() -> AppResult<PathBuf> {
    Ok(default_state_dir()?.join("active-sessions.json"))
}

pub fn workspace_registry_path() -> AppResult<PathBuf> {
    Ok(default_state_dir()?.join("workspaces.json"))
}
