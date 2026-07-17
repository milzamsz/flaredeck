use std::path::PathBuf;

use crate::error::{AppError, AppResult};

pub fn home_dir() -> AppResult<PathBuf> {
    dirs::home_dir().ok_or(AppError::NoHomeDir)
}

pub fn cloudflared_dir() -> AppResult<PathBuf> {
    Ok(home_dir()?.join(".cloudflared"))
}

pub fn cert_path() -> AppResult<PathBuf> {
    Ok(cloudflared_dir()?.join("cert.pem"))
}

/// Cert to use for a profile. Today it's always the global cert at
/// `~/.cloudflared/cert.pem` (created via `cloudflared tunnel login`).
/// The `Profile.cert_path` field is reserved for a future per-profile
/// override but isn't surfaced in the UI; if set, it wins.
pub fn effective_cert_path(profile: &crate::types::Profile) -> AppResult<PathBuf> {
    if let Some(p) = profile
        .cert_path
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        let per = PathBuf::from(p);
        if per.exists() {
            return Ok(per);
        }
    }
    cert_path()
}

pub fn flaredeck_index_path() -> AppResult<PathBuf> {
    Ok(cloudflared_dir()?.join("flaredeck.json"))
}

fn fallback_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".local/bin/cloudflared"));
        #[cfg(target_os = "windows")]
        paths.push(home.join("AppData/Local/cloudflared/cloudflared.exe"));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/opt/homebrew/bin/cloudflared"));
        paths.push(PathBuf::from("/usr/local/bin/cloudflared"));
        paths.push(PathBuf::from("/opt/local/bin/cloudflared"));
    }

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/usr/local/bin/cloudflared"));
        paths.push(PathBuf::from("/usr/bin/cloudflared"));
        paths.push(PathBuf::from("/snap/bin/cloudflared"));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            paths.push(PathBuf::from(program_files).join("cloudflared/cloudflared.exe"));
        }
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            paths.push(PathBuf::from(program_files_x86).join("cloudflared/cloudflared.exe"));
        }
    }

    paths
}

pub fn resolve_cloudflared_path() -> Option<PathBuf> {
    if let Ok(path) = which::which("cloudflared") {
        return Some(path);
    }
    fallback_paths().into_iter().find(|p| p.exists())
}

pub async fn cloudflared_version(path: &std::path::Path) -> Option<String> {
    let output = tokio::process::Command::new(path)
        .arg("--version")
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().next().unwrap_or("").trim().to_string())
}

pub fn ensure_cloudflared() -> AppResult<PathBuf> {
    resolve_cloudflared_path().ok_or(AppError::CloudflaredMissing)
}

pub fn install_path() -> AppResult<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|p| p.join("AppData/Local")))
            .ok_or(AppError::NoHomeDir)?;
        return Ok(base.join("cloudflared/cloudflared.exe"));
    }

    Ok(home_dir()?.join(".local/bin/cloudflared"))
}

pub fn release_asset() -> AppResult<&'static str> {
    let asset = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "cloudflared-windows-amd64.exe",
        ("macos", "x86_64") => "cloudflared-darwin-amd64.tgz",
        ("macos", "aarch64") => "cloudflared-darwin-arm64.tgz",
        ("linux", "x86_64") => "cloudflared-linux-amd64",
        ("linux", "aarch64") => "cloudflared-linux-arm64",
        _ => {
            return Err(AppError::Other(
                "unsupported platform for cloudflared installer".into(),
            ))
        }
    };
    Ok(asset)
}

#[cfg(test)]
mod tests {
    use super::release_asset;

    #[test]
    fn release_asset_is_official_cloudflare_binary() {
        assert!(release_asset().unwrap().starts_with("cloudflared-"));
    }
}
