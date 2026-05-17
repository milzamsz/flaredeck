//! Per-profile Cloudflare API token storage.
//!
//! Primary backend: OS keychain via `keyring`. Falls back to an encrypted
//! file at `~/.cloudflared/flaredeck.secrets` when the keychain is
//! unavailable — typical on WSL distros without `gnome-keyring`.
//! The fallback key is derived from a stable per-machine identifier
//! (`/etc/machine-id`, IOPlatformUUID, MachineGuid), so the file does not
//! survive being copied to another machine.

use std::collections::BTreeMap;
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cloudflared::cloudflared_dir;
use crate::error::{AppError, AppResult};

const SERVICE: &str = "flaredeck";
const KEY_CONTEXT: &[u8] = b"flaredeck-secrets-v1";
const FILE_VERSION: u32 = 1;

fn entry(profile_id: &str) -> AppResult<Entry> {
    Entry::new(SERVICE, profile_id).map_err(|e| AppError::Secret(e.to_string()))
}

pub fn store_token(profile_id: &str, token: &str) -> AppResult<()> {
    match entry(profile_id).and_then(|e| {
        e.set_password(token)
            .map_err(|e| AppError::Secret(e.to_string()))
    }) {
        Ok(()) => {
            // best-effort: clear any prior fallback entry so we don't read stale data
            let _ = file_delete(profile_id);
            Ok(())
        }
        Err(keychain_err) => match file_store(profile_id, token) {
            Ok(()) => Ok(()),
            Err(file_err) => Err(AppError::Secret(format!(
                "keychain failed ({keychain_err}); encrypted-file fallback also failed ({file_err})"
            ))),
        },
    }
}

pub fn load_token(profile_id: &str) -> AppResult<Option<String>> {
    match entry(profile_id).and_then(|e| match e.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Secret(e.to_string())),
    }) {
        Ok(Some(t)) => Ok(Some(t)),
        Ok(None) => file_load(profile_id),
        // keychain itself blew up (e.g. no Secret Service); try file fallback
        Err(_) => file_load(profile_id),
    }
}

pub fn delete_token(profile_id: &str) -> AppResult<()> {
    // best-effort on both layers; we only fail if both report a hard error
    let key_err = entry(profile_id)
        .and_then(|e| match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Secret(e.to_string())),
        })
        .err();
    let file_err = file_delete(profile_id).err();
    match (key_err, file_err) {
        (None, _) | (_, None) => Ok(()),
        (Some(a), Some(b)) => Err(AppError::Secret(format!(
            "keychain delete failed ({a}); file fallback delete failed ({b})"
        ))),
    }
}

// ---------------- encrypted-file fallback ----------------

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecretsFile {
    v: u32,
    entries: BTreeMap<String, String>, // profile_id -> base64(nonce || ciphertext)
}

fn secrets_file_path() -> AppResult<PathBuf> {
    Ok(cloudflared_dir()?.join("flaredeck.secrets"))
}

fn derive_key() -> AppResult<[u8; 32]> {
    let id = machine_id()?;
    let mut h = Sha256::new();
    h.update(id.as_bytes());
    h.update(KEY_CONTEXT);
    Ok(h.finalize().into())
}

fn cipher() -> AppResult<ChaCha20Poly1305> {
    let key = derive_key()?;
    Ok(ChaCha20Poly1305::new(Key::from_slice(&key)))
}

fn read_file() -> AppResult<SecretsFile> {
    let path = secrets_file_path()?;
    if !path.exists() {
        return Ok(SecretsFile {
            v: FILE_VERSION,
            entries: BTreeMap::new(),
        });
    }
    let raw = std::fs::read(&path).map_err(AppError::from)?;
    let f: SecretsFile = serde_json::from_slice(&raw)
        .map_err(|e| AppError::Secret(format!("corrupt secrets file: {e}")))?;
    if f.v != FILE_VERSION {
        return Err(AppError::Secret(format!(
            "unsupported secrets file version: {}",
            f.v
        )));
    }
    Ok(f)
}

fn write_file(f: &SecretsFile) -> AppResult<()> {
    let dir = cloudflared_dir()?;
    std::fs::create_dir_all(&dir).map_err(AppError::from)?;
    let path = secrets_file_path()?;
    let raw = serde_json::to_vec_pretty(f).map_err(AppError::from)?;
    std::fs::write(&path, raw).map_err(AppError::from)?;
    set_owner_only_perms(&path)?;
    Ok(())
}

#[cfg(unix)]
fn set_owner_only_perms(path: &std::path::Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).map_err(AppError::from)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms).map_err(AppError::from)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only_perms(_path: &std::path::Path) -> AppResult<()> {
    // Windows file ACLs default to the current user's profile dir; rely on that.
    Ok(())
}

fn file_store(profile_id: &str, token: &str) -> AppResult<()> {
    let cipher = cipher()?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, token.as_bytes())
        .map_err(|e| AppError::Secret(format!("encrypt failed: {e}")))?;
    let mut blob = Vec::with_capacity(12 + ct.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ct);

    let mut f = read_file()?;
    f.v = FILE_VERSION;
    f.entries
        .insert(profile_id.to_string(), B64.encode(&blob));
    write_file(&f)
}

fn file_load(profile_id: &str) -> AppResult<Option<String>> {
    let f = match read_file() {
        Ok(f) => f,
        // if the fallback file is unreadable that's not a "no token" signal,
        // but for the load path we treat it as "no token" rather than break
        // every UI surface — the user can re-set the token to recover.
        Err(_) => return Ok(None),
    };
    let Some(b64) = f.entries.get(profile_id) else {
        return Ok(None);
    };
    let blob = B64
        .decode(b64)
        .map_err(|e| AppError::Secret(format!("base64 decode failed: {e}")))?;
    if blob.len() < 13 {
        return Err(AppError::Secret("ciphertext too short".into()));
    }
    let (nonce_bytes, ct) = blob.split_at(12);
    let cipher = cipher()?;
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ct)
        .map_err(|e| AppError::Secret(format!("decrypt failed: {e}")))?;
    Ok(Some(String::from_utf8(pt).map_err(|e| {
        AppError::Secret(format!("token is not utf-8: {e}"))
    })?))
}

fn file_delete(profile_id: &str) -> AppResult<()> {
    let path = secrets_file_path()?;
    if !path.exists() {
        return Ok(());
    }
    let mut f = read_file()?;
    if f.entries.remove(profile_id).is_some() {
        if f.entries.is_empty() {
            std::fs::remove_file(&path).map_err(AppError::from)?;
        } else {
            write_file(&f)?;
        }
    }
    Ok(())
}

// ---------------- machine id helpers ----------------

fn machine_id() -> AppResult<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
            .map_err(AppError::from)?;
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            if let Some(rest) = line.split("IOPlatformUUID").nth(1) {
                if let Some(start) = rest.find('"') {
                    if let Some(end) = rest[start + 1..].find('"') {
                        return Ok(rest[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        // Read MachineGuid from registry without pulling in a crate
        let out = std::process::Command::new("reg")
            .args([
                "query",
                "HKLM\\SOFTWARE\\Microsoft\\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
            .map_err(AppError::from)?;
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            if line.contains("MachineGuid") {
                if let Some(value) = line.split_whitespace().last() {
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }
        }
    }
    Err(AppError::Secret(
        "could not derive a stable machine id for secrets fallback".into(),
    ))
}
