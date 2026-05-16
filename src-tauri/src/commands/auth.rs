use std::process::Stdio;

use crate::cloudflared::{cert_path, ensure_cloudflared};
use crate::error::{AppError, AppResult};
use crate::types::AuthStatus;

#[tauri::command]
pub async fn auth_check() -> AppResult<AuthStatus> {
    let cert = cert_path()?;
    let authenticated = cert.exists();
    Ok(AuthStatus {
        authenticated,
        cert_path: authenticated.then(|| cert.to_string_lossy().to_string()),
    })
}

#[tauri::command]
pub async fn auth_login() -> AppResult<()> {
    let binary = ensure_cloudflared()?;
    let mut child = tokio::process::Command::new(&binary)
        .args(["tunnel", "login"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let _ = child.id();
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(())
}

#[tauri::command]
pub async fn auth_logout() -> AppResult<()> {
    let cert = cert_path()?;
    if cert.exists() {
        tokio::fs::remove_file(&cert)
            .await
            .map_err(AppError::from)?;
    }
    Ok(())
}
