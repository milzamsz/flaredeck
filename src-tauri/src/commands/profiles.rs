use std::path::PathBuf;

use uuid::Uuid;

use crate::cloudflared::{cloudflared_dir, flaredeck_index_path};
use crate::commands::config::write_initial_config;
use crate::commands::tunnel::tunnel_create;
use crate::error::{AppError, AppResult};
use crate::types::{Profile, ProfileIndex, ProfilePatch};

async fn load_index() -> AppResult<ProfileIndex> {
    let path = flaredeck_index_path()?;
    if !path.exists() {
        return Ok(ProfileIndex::default());
    }
    let raw = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

async fn save_index(index: &ProfileIndex) -> AppResult<()> {
    let dir = cloudflared_dir()?;
    tokio::fs::create_dir_all(&dir).await?;
    let path = flaredeck_index_path()?;
    let raw = serde_json::to_string_pretty(index)?;
    tokio::fs::write(&path, raw).await?;
    Ok(())
}

#[tauri::command]
pub async fn profiles_list() -> AppResult<ProfileIndex> {
    load_index().await
}

#[tauri::command]
pub async fn profiles_create(
    name: String,
    tunnel_name: String,
    wsl_host: Option<bool>,
    create_tunnel: Option<bool>,
) -> AppResult<Profile> {
    let mut index = load_index().await?;
    let id = Uuid::new_v4().to_string();
    let config_path: PathBuf = cloudflared_dir()?.join(format!("{id}.yml"));
    let config_path_str = config_path.to_string_lossy().to_string();

    let resolved_tunnel_name = if create_tunnel.unwrap_or(false) {
        let created = tunnel_create(tunnel_name.clone()).await?;
        write_initial_config(&config_path, &created.uuid, &created.credentials_file).await?;
        created.name
    } else {
        tunnel_name
    };

    let profile = Profile {
        id: id.clone(),
        name,
        tunnel_name: resolved_tunnel_name,
        config_path: config_path_str,
        wsl_host: wsl_host.unwrap_or(false),
    };
    index.profiles.push(profile.clone());
    if index.active_profile_id.is_none() {
        index.active_profile_id = Some(id);
    }
    save_index(&index).await?;
    Ok(profile)
}

#[tauri::command]
pub async fn profiles_update(id: String, patch: ProfilePatch) -> AppResult<Profile> {
    let mut index = load_index().await?;
    let profile = index
        .profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::ProfileNotFound(id.clone()))?;
    if let Some(name) = patch.name {
        profile.name = name;
    }
    if let Some(wsl_host) = patch.wsl_host {
        profile.wsl_host = wsl_host;
    }
    let updated = profile.clone();
    save_index(&index).await?;
    Ok(updated)
}

#[tauri::command]
pub async fn profiles_delete(id: String) -> AppResult<ProfileIndex> {
    let mut index = load_index().await?;
    let before = index.profiles.len();
    index.profiles.retain(|p| p.id != id);
    if index.profiles.len() == before {
        return Err(AppError::ProfileNotFound(id));
    }
    if index.active_profile_id.as_deref() == Some(&id) {
        index.active_profile_id = index.profiles.first().map(|p| p.id.clone());
    }
    save_index(&index).await?;
    Ok(index)
}

#[tauri::command]
pub async fn profiles_set_active(id: String) -> AppResult<ProfileIndex> {
    let mut index = load_index().await?;
    if !index.profiles.iter().any(|p| p.id == id) {
        return Err(AppError::ProfileNotFound(id));
    }
    index.active_profile_id = Some(id);
    save_index(&index).await?;
    Ok(index)
}
