use std::path::PathBuf;

use uuid::Uuid;

use crate::cf_api::CfClient;
use crate::cloudflared::{cloudflared_dir, flaredeck_index_path};
use crate::commands::cf::create_tunnel_with_files;
use crate::error::{AppError, AppResult};
use crate::secrets;
use crate::types::{Profile, ProfileIndex, ProfilePatch, TokenInfo};

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

pub async fn get_profile(id: &str) -> AppResult<Profile> {
    let index = load_index().await?;
    index
        .profiles
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::ProfileNotFound(id.to_string()))
}

async fn mutate_profile<F>(id: &str, mutate: F) -> AppResult<Profile>
where
    F: FnOnce(&mut Profile),
{
    let mut index = load_index().await?;
    let profile = index
        .profiles
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::ProfileNotFound(id.to_string()))?;
    mutate(profile);
    let updated = profile.clone();
    save_index(&index).await?;
    Ok(updated)
}

#[tauri::command]
pub async fn profiles_list() -> AppResult<ProfileIndex> {
    load_index().await
}

#[tauri::command]
pub async fn profiles_update(id: String, patch: ProfilePatch) -> AppResult<Profile> {
    mutate_profile(&id, |profile| {
        if let Some(name) = patch.name {
            profile.name = name;
        }
        if let Some(wsl_host) = patch.wsl_host {
            profile.wsl_host = wsl_host;
        }
        if let Some(account_id) = patch.account_id {
            profile.account_id = if account_id.trim().is_empty() {
                None
            } else {
                Some(account_id.trim().to_string())
            };
        }
        if let Some(zone_id) = patch.zone_id {
            profile.zone_id = if zone_id.trim().is_empty() {
                None
            } else {
                Some(zone_id.trim().to_string())
            };
        }
        if let Some(zone_name) = patch.zone_name {
            profile.zone_name = if zone_name.trim().is_empty() {
                None
            } else {
                Some(zone_name.trim().to_string())
            };
        }
    })
    .await
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
    // best-effort: keep the index update authoritative even if the keychain bombs
    let _ = secrets::delete_token(&id);
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

#[tauri::command]
pub async fn profiles_set_token(id: String, token: String) -> AppResult<Profile> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err(AppError::Other("token is empty".into()));
    }
    // ensure the profile exists before touching the keychain
    get_profile(&id).await?;
    secrets::store_token(&id, trimmed)?;
    mutate_profile(&id, |p| p.has_api_token = true).await
}

#[tauri::command]
pub async fn profiles_clear_token(id: String) -> AppResult<Profile> {
    get_profile(&id).await?;
    secrets::delete_token(&id)?;
    mutate_profile(&id, |p| p.has_api_token = false).await
}

#[tauri::command]
pub async fn profiles_verify_token(id: String) -> AppResult<TokenInfo> {
    let profile = get_profile(&id).await?;
    let client = CfClient::for_profile(&profile)?;
    client.verify_token().await
}

/// One-shot wizard: paste a token, type a domain, get a fully working
/// profile with its Cloudflare Tunnel created and DNS-ready.
///
/// If `token` is empty, looks up an existing keychain token by
/// `reuse_token_from_profile_id`. This is how the "Use token from
/// <other profile>" dropdown in the New Profile dialog works without
/// the user re-pasting the secret.
#[tauri::command]
pub async fn profiles_create_simple(
    name: String,
    token: String,
    reuse_token_from_profile_id: Option<String>,
    domain: String,
    wsl_host: Option<bool>,
) -> AppResult<Profile> {
    let name = name.trim().to_string();
    let domain = domain.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Other("display name is empty".into()));
    }
    if domain.is_empty() {
        return Err(AppError::Other("domain is empty".into()));
    }

    let token = if !token.trim().is_empty() {
        token.trim().to_string()
    } else if let Some(src_id) = reuse_token_from_profile_id.as_deref() {
        secrets::load_token(src_id)?.ok_or_else(|| AppError::NoApiToken(src_id.to_string()))?
    } else {
        return Err(AppError::Other(
            "no token provided and no source profile to reuse from".into(),
        ));
    };

    // 1) Resolve the domain to a zone+account via Cloudflare API.
    let lookup_client = CfClient::from_token(token.clone(), None)?;
    let zone = lookup_client.lookup_zone_by_domain(&domain).await?;

    // 1b) Pre-flight: confirm the token can actually touch the
    //     `cfd_tunnel` namespace before we mutate any state.
    //     Without this we'd write a UUID config and credentials file,
    //     stash the token in the keychain, and only THEN discover the
    //     token lacks Cloudflare Tunnel: Edit — leaving cruft behind.
    let preflight = CfClient::from_token(token.clone(), Some(zone.account_id.clone()))?;
    preflight.preflight_cfd_tunnel_scope().await?;

    // 2) Create the cloudflared tunnel via the API. Tunnel name is
    //    derived from the zone so it's stable and readable in the
    //    Cloudflare dashboard.
    let id = Uuid::new_v4().to_string();
    let short = &id[..8];
    let tunnel_name = format!(
        "flaredeck-{}-{short}",
        zone.zone_name.replace('.', "-")
    );
    let config_path: PathBuf = cloudflared_dir()?.join(format!("{id}.yml"));
    let config_path_str = config_path.to_string_lossy().to_string();
    let (_tunnel_uuid, created_name, _cred_path) = create_tunnel_with_files(
        token.clone(),
        zone.account_id.clone(),
        &tunnel_name,
        &config_path,
    )
    .await?;

    // 3) Persist token in the keychain under this new profile id.
    secrets::store_token(&id, &token)?;

    // 4) Save the profile to the index.
    let profile = Profile {
        id: id.clone(),
        name,
        tunnel_name: created_name,
        config_path: config_path_str,
        wsl_host: wsl_host.unwrap_or(false),
        account_id: Some(zone.account_id),
        zone_id: Some(zone.zone_id),
        zone_name: Some(zone.zone_name),
        cert_path: None,
        has_api_token: true,
    };
    let mut index = load_index().await?;
    index.profiles.push(profile.clone());
    if index.active_profile_id.is_none() {
        index.active_profile_id = Some(id);
    }
    save_index(&index).await?;
    Ok(profile)
}
