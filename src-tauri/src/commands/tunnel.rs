use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::state::RuntimeState;
use crate::types::{CloudflaredInfo, TunnelStatus};

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
pub async fn cloudflared_install() -> AppResult<CloudflaredInfo> {
    let path = crate::cloudflared::install_path()?;
    let asset = crate::cloudflared::release_asset()?;
    let url = format!("https://github.com/cloudflare/cloudflared/releases/latest/download/{asset}");
    let response = reqwest::get(url)
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;
    if !response.status().is_success() {
        return Err(AppError::Http(format!(
            "cloudflared download failed with HTTP {}",
            response.status()
        )));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temp = path.with_extension(if asset.ends_with(".tgz") {
        "tgz.download"
    } else {
        "download"
    });
    tokio::fs::write(&temp, bytes).await?;
    if asset.ends_with(".tgz") {
        let status = tokio::process::Command::new("tar")
            .args([
                "-xzf",
                &temp.to_string_lossy(),
                "-C",
                &parent_string(&path)?,
            ])
            .status()
            .await?;
        if !status.success() {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(AppError::Other(
                "failed to extract cloudflared archive".into(),
            ));
        }
        tokio::fs::remove_file(&temp).await?;
    } else {
        tokio::fs::rename(&temp, &path).await?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).await?;
    }

    let version = crate::cloudflared::cloudflared_version(&path).await;
    Ok(CloudflaredInfo {
        installed: version.is_some(),
        path: Some(path.to_string_lossy().to_string()),
        version,
    })
}

fn parent_string(path: &std::path::Path) -> AppResult<String> {
    path.parent()
        .map(|p| p.to_string_lossy().into_owned())
        .ok_or_else(|| AppError::Other("cloudflared install path has no parent".into()))
}

#[tauri::command]
pub async fn tunnel_status(
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    crate::application::tunnel_service::status(&state, profile_id)
}

#[tauri::command]
pub async fn tunnel_start(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    crate::application::tunnel_service::start(&state, profile_id, move |event| {
        let _ = app.emit("tunnel:log", event);
    })
    .await
}

#[tauri::command]
pub async fn tunnel_stop(
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    crate::application::tunnel_service::stop(&state, profile_id).await
}

#[tauri::command]
pub async fn tunnel_restart(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    profile_id: String,
) -> AppResult<TunnelStatus> {
    crate::application::tunnel_service::restart(&state, profile_id, move |event| {
        let _ = app.emit("tunnel:log", event);
    })
    .await
}

/// CLI fallback when a profile doesn't have a Cloudflare API token
/// configured. The API-token path goes through `cf_route_dns` in
/// `commands/cf.rs`.
#[tauri::command]
pub async fn tunnel_route_dns(tunnel_name: String, hostname: String) -> AppResult<()> {
    crate::application::route_service::route_dns_with_cloudflared(tunnel_name, hostname).await
}
