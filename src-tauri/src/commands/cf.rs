use serde::Serialize;

use crate::cf_api::{CfClient, ZoneLookup};
use crate::cloudflared::cloudflared_dir;
use crate::commands::config::write_initial_config;
use crate::error::{AppError, AppResult};

/// Credentials JSON cloudflared expects at `~/.cloudflared/<uuid>.json`.
/// PascalCase keys to match cloudflared's existing on-disk format.
#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct CredentialsFile<'a> {
    AccountTag: &'a str,
    TunnelID: &'a str,
    TunnelName: &'a str,
    TunnelSecret: &'a str,
}

/// Create a tunnel via the API for the given account/token, write the
/// credentials JSON file cloudflared expects, and seed a profile YAML.
/// Returns the credentials-file path so the caller can record it.
pub(crate) async fn create_tunnel_with_files(
    token: String,
    account_id: String,
    tunnel_name: &str,
    config_path: &std::path::Path,
) -> AppResult<(String, String, String)> {
    let client = CfClient::from_token(token, Some(account_id.clone()))?;
    let created = client.create_tunnel(tunnel_name).await?;

    let dir = cloudflared_dir()?;
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(AppError::from)?;
    let cred_path = dir.join(format!("{}.json", created.id));
    let cred = CredentialsFile {
        AccountTag: &account_id,
        TunnelID: &created.id,
        TunnelName: &created.name,
        TunnelSecret: &created.secret_b64,
    };
    let raw = serde_json::to_vec_pretty(&cred)?;
    tokio::fs::write(&cred_path, raw)
        .await
        .map_err(AppError::from)?;
    let cred_path_str = cred_path.to_string_lossy().to_string();

    write_initial_config(config_path, &created.id, &cred_path_str).await?;

    Ok((created.id, created.name, cred_path_str))
}

#[tauri::command]
pub async fn cf_route_dns(
    profile_id: String,
    hostname: String,
    tunnel_id: String,
) -> AppResult<String> {
    crate::application::route_service::upsert_dns_route(profile_id, hostname, tunnel_id).await
}

#[tauri::command]
pub async fn cf_lookup_zone(profile_id: String, domain: String) -> AppResult<ZoneLookup> {
    crate::application::route_service::lookup_zone(profile_id, domain).await
}
