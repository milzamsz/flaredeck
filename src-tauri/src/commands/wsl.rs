use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::AppResult;

const CACHE_TTL: Duration = Duration::from_secs(60);

static CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

#[tauri::command]
pub async fn wsl_host_ip() -> AppResult<Option<String>> {
    {
        let guard = CACHE.lock().ok();
        if let Some(state) = guard.as_ref().and_then(|g| g.as_ref()) {
            if state.1.elapsed() < CACHE_TTL {
                return Ok(Some(state.0.clone()));
            }
        }
    }

    let ip = detect_ip().await;
    if let Some(ip) = ip.as_ref() {
        if let Ok(mut guard) = CACHE.lock() {
            *guard = Some((ip.clone(), Instant::now()));
        }
    }
    Ok(ip)
}

#[cfg(target_os = "windows")]
async fn detect_ip() -> Option<String> {
    let output = tokio::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "hostname", "-I"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .find(|tok| {
            let parts: Vec<&str> = tok.split('.').collect();
            parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok())
        })
        .map(|s| s.to_string())
}

#[cfg(not(target_os = "windows"))]
async fn detect_ip() -> Option<String> {
    None
}
