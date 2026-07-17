use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

use crate::application::profile_service;
use crate::application::session_service::{
    matches_process, process_start_time, stop_owned_process_tree,
};
use crate::application::webhook_service;
use crate::application::workspace_service::{authorize_start, Route};
use crate::cf_api::{CfClient, CfDnsRecord};
use crate::error::{AppError, AppResult};
use crate::types::{CloudflaredConfig, IngressRule, Profile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryRouteState {
    Creating,
    Active,
    CleanupIncomplete,
    Cleaned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryRouteRecord {
    pub id: String,
    pub workspace_id: String,
    pub session_id: String,
    pub profile_id: String,
    pub zone_id: String,
    pub tunnel_id: String,
    pub hostname: String,
    pub path: Option<String>,
    pub origin: String,
    pub proxy_service: String,
    pub dns_record_id: Option<String>,
    pub proxy_pid: Option<u32>,
    pub proxy_started_at_seconds: Option<u64>,
    pub proxy_executable: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub state: TemporaryRouteState,
    pub cleanup_error: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemporaryRouteStore {
    #[serde(default = "schema_version")]
    schema_version: u8,
    #[serde(default)]
    routes: Vec<TemporaryRouteRecord>,
}

fn schema_version() -> u8 {
    1
}

trait TemporaryDnsPort {
    async fn find(
        &self,
        profile: &Profile,
        zone_id: &str,
        hostname: &str,
    ) -> AppResult<Option<CfDnsRecord>>;
    async fn create(
        &self,
        profile: &Profile,
        zone_id: &str,
        hostname: &str,
        tunnel_id: &str,
    ) -> AppResult<String>;
    async fn delete(&self, profile: &Profile, zone_id: &str, record_id: &str) -> AppResult<()>;
}

struct CloudflareTemporaryDnsPort;

impl TemporaryDnsPort for CloudflareTemporaryDnsPort {
    async fn find(
        &self,
        profile: &Profile,
        zone_id: &str,
        hostname: &str,
    ) -> AppResult<Option<CfDnsRecord>> {
        CfClient::for_profile(profile)?
            .find_dns_route(zone_id, hostname)
            .await
    }

    async fn create(
        &self,
        profile: &Profile,
        zone_id: &str,
        hostname: &str,
        tunnel_id: &str,
    ) -> AppResult<String> {
        CfClient::for_profile(profile)?
            .upsert_dns_route(zone_id, hostname, tunnel_id)
            .await
    }

    async fn delete(&self, profile: &Profile, zone_id: &str, record_id: &str) -> AppResult<()> {
        CfClient::for_profile(profile)?
            .delete_dns_route(zone_id, record_id)
            .await
    }
}

pub fn route_store_path(session_store_path: &Path) -> PathBuf {
    session_store_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("temporary-routes.json")
}

pub fn event_store_path(route_store_path: &Path, route_id: &str) -> AppResult<PathBuf> {
    if route_id.is_empty()
        || !route_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(AppError::Other("temporary route id is invalid".into()));
    }
    Ok(route_store_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("webhook-events")
        .join(format!("{route_id}.json")))
}

pub async fn list_routes(
    store_path: &Path,
    session_id: Option<&str>,
) -> AppResult<Vec<TemporaryRouteRecord>> {
    Ok(load_store(store_path)
        .await?
        .routes
        .into_iter()
        .filter(|route| session_id.is_none_or(|session_id| route.session_id == session_id))
        .collect())
}

pub async fn create_for_session(
    workspace_root: &Path,
    approval_path: &Path,
    session_store_path: &Path,
    session_id: &str,
) -> AppResult<Vec<TemporaryRouteRecord>> {
    let (_root, manifest) = authorize_start(workspace_root, approval_path).await?;
    let temporary = manifest
        .exposure
        .routes
        .iter()
        .filter(|route| route.mode.as_deref() == Some("temporary"))
        .cloned()
        .collect::<Vec<_>>();
    if temporary.is_empty() {
        return Ok(Vec::new());
    }
    let profile_selector = manifest
        .profile
        .id
        .or(manifest.profile.name)
        .unwrap_or_default();
    let profile = resolve_profile(&profile_selector).await?;
    let zone_id = profile
        .zone_id
        .clone()
        .ok_or(AppError::MissingProfileField("zoneId"))?;
    let store_path = route_store_path(session_store_path);
    reconcile_expired(&store_path).await?;
    let mut created = Vec::new();
    for route in temporary {
        let record = match create_route(
            &CloudflareTemporaryDnsPort,
            &store_path,
            &manifest
                .project
                .id
                .clone()
                .unwrap_or_else(|| manifest.project.name.clone()),
            session_id,
            &profile,
            &zone_id,
            &route,
        )
        .await
        {
            Ok(record) => record,
            Err(error) => {
                let cleanup = cleanup_session(&store_path, session_id).await;
                return match cleanup {
                    Ok(_) => Err(error),
                    Err(cleanup) => Err(AppError::Other(format!(
                        "{error}; previous temporary route cleanup also failed: {cleanup}"
                    ))),
                };
            }
        };
        created.push(record);
    }
    Ok(created)
}

async fn create_route(
    dns: &impl TemporaryDnsPort,
    store_path: &Path,
    workspace_id: &str,
    session_id: &str,
    profile: &Profile,
    zone_id: &str,
    route: &Route,
) -> AppResult<TemporaryRouteRecord> {
    validate_hostname(&route.hostname)?;
    let mut config = load_config(&profile.config_path).await?;
    let tunnel_id = config
        .tunnel
        .clone()
        .ok_or_else(|| AppError::Other("profile configuration has no tunnel id".into()))?;
    preflight_ingress(&config, route)?;
    ensure_no_dns_conflict(dns, profile, zone_id, &route.hostname).await?;
    let mut store = load_store(store_path).await?;
    if store.routes.iter().any(|entry| {
        entry.state != TemporaryRouteState::Cleaned
            && entry.hostname.eq_ignore_ascii_case(&route.hostname)
            && entry.path == route.path
    }) {
        return Err(AppError::Other(
            "temporary route already has active ownership".into(),
        ));
    }
    let port = available_port()?;
    let id = format!("route_{}", uuid::Uuid::new_v4());
    let proxy_service = format!("http://127.0.0.1:{port}");
    let mut record = TemporaryRouteRecord {
        id: id.clone(),
        workspace_id: workspace_id.into(),
        session_id: session_id.into(),
        profile_id: profile.id.clone(),
        zone_id: zone_id.into(),
        tunnel_id: tunnel_id.clone(),
        hostname: route.hostname.clone(),
        path: route.path.clone(),
        origin: route.service.clone(),
        proxy_service: proxy_service.clone(),
        dns_record_id: None,
        proxy_pid: None,
        proxy_started_at_seconds: None,
        proxy_executable: None,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
        state: TemporaryRouteState::Creating,
        cleanup_error: None,
    };
    store.routes.push(record.clone());
    save_store(store_path, &store).await?;

    let result = async {
        record.dns_record_id = Some(
            dns.create(profile, zone_id, &route.hostname, &tunnel_id)
                .await?,
        );
        update_record(store_path, &record).await?;
        insert_temporary_ingress(&mut config, route, &proxy_service)?;
        save_config(&profile.config_path, &config).await?;
        let executable = proxy_executable()?;
        let child = tokio::process::Command::new(&executable)
            .args([
                format!("127.0.0.1:{port}"),
                id.clone(),
                route.service.clone(),
                event_store_path(store_path, &id)?
                    .to_string_lossy()
                    .into_owned(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| AppError::Other("webhook proxy did not return a PID".into()))?;
        record.proxy_pid = Some(pid);
        record.proxy_started_at_seconds = process_start_time(pid);
        record.proxy_executable = Some(executable.to_string_lossy().into_owned());
        update_record(store_path, &record).await?;
        wait_for_proxy(port).await?;
        record.state = TemporaryRouteState::Active;
        update_record(store_path, &record).await?;
        AppResult::Ok(())
    }
    .await;
    if let Err(error) = result {
        let cleanup = cleanup_record(dns, store_path, profile, &mut record).await;
        return match cleanup {
            Ok(()) => Err(error),
            Err(cleanup) => Err(AppError::Other(format!(
                "{error}; cleanup also failed: {cleanup}"
            ))),
        };
    }
    Ok(record)
}

pub async fn cleanup_session(
    store_path: &Path,
    session_id: &str,
) -> AppResult<Vec<TemporaryRouteRecord>> {
    let records = list_routes(store_path, Some(session_id)).await?;
    let profiles = profile_service::list().await?;
    let mut results = Vec::new();
    for mut record in records {
        if record.state == TemporaryRouteState::Cleaned {
            results.push(record);
            continue;
        }
        let Some(profile) = profiles
            .profiles
            .iter()
            .find(|profile| profile.id == record.profile_id)
        else {
            record.state = TemporaryRouteState::CleanupIncomplete;
            record.cleanup_error = Some("owned profile no longer exists".into());
            update_record(store_path, &record).await?;
            results.push(record);
            continue;
        };
        if let Err(error) = cleanup_record(
            &CloudflareTemporaryDnsPort,
            store_path,
            profile,
            &mut record,
        )
        .await
        {
            record.state = TemporaryRouteState::CleanupIncomplete;
            let mut message = error.to_string();
            if let Err(clear_error) =
                webhook_service::clear_route(&event_store_path(store_path, &record.id)?, &record.id)
                    .await
            {
                message.push_str(&format!("; event cleanup also failed: {clear_error}"));
            }
            record.cleanup_error = Some(message);
            update_record(store_path, &record).await?;
        }
        results.push(record);
    }
    Ok(results)
}

pub async fn reconcile_expired(store_path: &Path) -> AppResult<Vec<TemporaryRouteRecord>> {
    let expired_sessions = load_store(store_path)
        .await?
        .routes
        .into_iter()
        .filter(|route| {
            route.state == TemporaryRouteState::CleanupIncomplete
                || (route.state != TemporaryRouteState::Cleaned && route.expires_at <= Utc::now())
        })
        .map(|route| route.session_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut reconciled = Vec::new();
    for session in expired_sessions {
        reconciled.extend(cleanup_session(store_path, &session).await?);
    }
    Ok(reconciled)
}

async fn cleanup_record(
    dns: &impl TemporaryDnsPort,
    store_path: &Path,
    profile: &Profile,
    record: &mut TemporaryRouteRecord,
) -> AppResult<()> {
    if record.proxy_pid.is_some() {
        if matches_process(
            record.proxy_pid,
            record.proxy_started_at_seconds,
            record.proxy_executable.as_deref(),
        ) {
            stop_owned_process_tree(record.proxy_pid.unwrap_or_default());
        } else if record.proxy_pid.is_some_and(process_exists) {
            return Err(AppError::Other(
                "owned webhook proxy identity no longer matches".into(),
            ));
        }
    }
    let mut config = load_config(&profile.config_path).await?;
    remove_exact_ingress(&mut config, record)?;
    save_config(&profile.config_path, &config).await?;
    if let Some(existing) = dns.find(profile, &record.zone_id, &record.hostname).await? {
        let expected = format!("{}.cfargotunnel.com", record.tunnel_id);
        if record.dns_record_id.as_deref() != Some(existing.id.as_str())
            || existing.content != expected
        {
            return Err(AppError::Other(
                "owned DNS record identity no longer matches".into(),
            ));
        }
        dns.delete(profile, &record.zone_id, &existing.id).await?;
    }
    webhook_service::clear_route(&event_store_path(store_path, &record.id)?, &record.id).await?;
    record.proxy_pid = None;
    record.state = TemporaryRouteState::Cleaned;
    record.cleanup_error = None;
    update_record(store_path, record).await?;
    Ok(())
}

fn preflight_ingress(config: &CloudflaredConfig, route: &Route) -> AppResult<()> {
    let ingress = config
        .ingress
        .as_ref()
        .ok_or_else(|| AppError::Other("profile configuration has no ingress rules".into()))?;
    if !ingress
        .last()
        .is_some_and(|rule| rule.hostname.is_none() && rule.service == "http_status:404")
    {
        return Err(AppError::Other(
            "profile ingress must end with http_status:404".into(),
        ));
    }
    if ingress.iter().any(|rule| {
        rule.hostname
            .as_deref()
            .is_some_and(|hostname| hostname.eq_ignore_ascii_case(&route.hostname))
            && rule.path.as_deref() == route.path.as_deref()
    }) {
        return Err(AppError::Other(
            "temporary route conflicts with existing ingress".into(),
        ));
    }
    Ok(())
}

async fn ensure_no_dns_conflict(
    dns: &impl TemporaryDnsPort,
    profile: &Profile,
    zone_id: &str,
    hostname: &str,
) -> AppResult<()> {
    if dns.find(profile, zone_id, hostname).await?.is_some() {
        return Err(AppError::Other(format!(
            "temporary route conflicts with existing DNS record {hostname}"
        )));
    }
    Ok(())
}

fn insert_temporary_ingress(
    config: &mut CloudflaredConfig,
    route: &Route,
    proxy_service: &str,
) -> AppResult<()> {
    preflight_ingress(config, route)?;
    let ingress = config.ingress.as_mut().unwrap();
    ingress.insert(
        ingress.len() - 1,
        IngressRule {
            hostname: Some(route.hostname.clone()),
            path: route.path.clone(),
            service: proxy_service.into(),
            origin_request: None,
        },
    );
    Ok(())
}

fn remove_exact_ingress(
    config: &mut CloudflaredConfig,
    record: &TemporaryRouteRecord,
) -> AppResult<()> {
    let ingress = config
        .ingress
        .as_mut()
        .ok_or_else(|| AppError::Other("profile configuration has no ingress rules".into()))?;
    if let Some(index) = ingress.iter().position(|rule| {
        rule.hostname
            .as_deref()
            .is_some_and(|hostname| hostname.eq_ignore_ascii_case(&record.hostname))
            && rule.path == record.path
    }) {
        if ingress[index].service != record.proxy_service {
            return Err(AppError::Other(
                "owned ingress identity no longer matches".into(),
            ));
        }
        ingress.remove(index);
    }
    if !ingress
        .last()
        .is_some_and(|rule| rule.hostname.is_none() && rule.service == "http_status:404")
    {
        return Err(AppError::Other(
            "cleanup would violate final catch-all ingress".into(),
        ));
    }
    Ok(())
}

async fn resolve_profile(selector: &str) -> AppResult<Profile> {
    profile_service::list()
        .await?
        .profiles
        .into_iter()
        .find(|profile| profile.id == selector || profile.name == selector)
        .ok_or_else(|| AppError::ProfileNotFound(selector.into()))
}

async fn load_config(path: &str) -> AppResult<CloudflaredConfig> {
    Ok(serde_yaml::from_str(
        &tokio::fs::read_to_string(path).await?,
    )?)
}

async fn save_config(path: &str, config: &CloudflaredConfig) -> AppResult<()> {
    let path = Path::new(path);
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, serde_yaml::to_string(config)?).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

fn proxy_executable() -> AppResult<PathBuf> {
    let name = if cfg!(windows) {
        "flaredeck-webhook-proxy.exe"
    } else {
        "flaredeck-webhook-proxy"
    };
    let adjacent = std::env::current_exe()?
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(name);
    if adjacent.exists() {
        return Ok(adjacent);
    }
    which::which(name)
        .map_err(|error| AppError::Other(format!("webhook proxy companion not found: {error}")))
}

fn available_port() -> AppResult<u16> {
    Ok(std::net::TcpListener::bind("127.0.0.1:0")?
        .local_addr()?
        .port())
}

fn process_exists(pid: u32) -> bool {
    System::new_all().process(Pid::from_u32(pid)).is_some()
}

async fn wait_for_proxy(port: u16) -> AppResult<()> {
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(AppError::Other("webhook proxy readiness timed out".into()))
}

fn validate_hostname(hostname: &str) -> AppResult<()> {
    if hostname.len() > 253
        || hostname.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err(AppError::Other(
            "temporary route hostname is invalid".into(),
        ));
    }
    Ok(())
}

async fn load_store(path: &Path) -> AppResult<TemporaryRouteStore> {
    match tokio::fs::read_to_string(path).await {
        Ok(raw) => Ok(serde_json::from_str(&raw)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(TemporaryRouteStore::default())
        }
        Err(error) => Err(error.into()),
    }
}

async fn save_store(path: &Path, store: &TemporaryRouteStore) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Other("temporary route path has no parent".into()))?;
    tokio::fs::create_dir_all(parent).await?;
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, serde_json::to_vec(store)?).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

async fn update_record(path: &Path, record: &TemporaryRouteRecord) -> AppResult<()> {
    let mut store = load_store(path).await?;
    let current = store
        .routes
        .iter_mut()
        .find(|current| current.id == record.id)
        .ok_or_else(|| AppError::Other("temporary route ownership record disappeared".into()))?;
    *current = record.clone();
    save_store(path, &store).await
}

#[cfg(test)]
mod tests {
    use super::{
        preflight_ingress, remove_exact_ingress, TemporaryRouteRecord, TemporaryRouteState,
    };
    use crate::application::workspace_service::Route;
    use crate::types::{CloudflaredConfig, IngressRule};
    use chrono::Utc;

    #[test]
    fn temporary_ingress_stays_before_catchall_and_cleanup_is_exact() {
        let route = Route {
            hostname: "hook.example.test".into(),
            service: "http://127.0.0.1:3000".into(),
            path: Some("/hook".into()),
            mode: Some("temporary".into()),
        };
        let mut config = config();
        super::insert_temporary_ingress(&mut config, &route, "http://127.0.0.1:4000").unwrap();
        let ingress = config.ingress.as_ref().unwrap();
        assert_eq!(ingress.last().unwrap().service, "http_status:404");
        let record = record();
        let mut mismatched = record.clone();
        mismatched.proxy_service = "http://127.0.0.1:other".into();
        assert!(remove_exact_ingress(&mut config, &mismatched).is_err());
        remove_exact_ingress(&mut config, &record).unwrap();
        assert_eq!(config.ingress.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn persistent_conflict_blocks_before_mutation() {
        let config = config();
        let route = Route {
            hostname: "persistent.example.test".into(),
            service: "http://127.0.0.1:3000".into(),
            path: None,
            mode: Some("temporary".into()),
        };
        assert!(preflight_ingress(&config, &route).is_err());
        assert_eq!(config.ingress.unwrap().len(), 2);
    }

    struct ConflictDns;

    impl super::TemporaryDnsPort for ConflictDns {
        async fn find(
            &self,
            _profile: &crate::types::Profile,
            _zone_id: &str,
            hostname: &str,
        ) -> crate::error::AppResult<Option<crate::cf_api::CfDnsRecord>> {
            Ok(Some(crate::cf_api::CfDnsRecord {
                id: "existing".into(),
                name: hostname.into(),
                content: "persistent.example".into(),
            }))
        }

        async fn create(
            &self,
            _profile: &crate::types::Profile,
            _zone_id: &str,
            _hostname: &str,
            _tunnel_id: &str,
        ) -> crate::error::AppResult<String> {
            panic!("create must not run after conflict")
        }

        async fn delete(
            &self,
            _profile: &crate::types::Profile,
            _zone_id: &str,
            _record_id: &str,
        ) -> crate::error::AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn dns_conflict_is_detected_before_creation() {
        let profile = crate::types::Profile {
            id: "profile".into(),
            name: "Profile".into(),
            tunnel_name: "tunnel".into(),
            config_path: "config".into(),
            wsl_host: false,
            account_id: None,
            zone_id: Some("zone".into()),
            zone_name: None,
            cert_path: None,
            has_api_token: true,
        };
        assert!(
            super::ensure_no_dns_conflict(&ConflictDns, &profile, "zone", "hook.example.test")
                .await
                .is_err()
        );
    }

    struct OwnedDns(std::sync::Mutex<Option<crate::cf_api::CfDnsRecord>>);

    impl super::TemporaryDnsPort for OwnedDns {
        async fn find(
            &self,
            _profile: &crate::types::Profile,
            _zone_id: &str,
            _hostname: &str,
        ) -> crate::error::AppResult<Option<crate::cf_api::CfDnsRecord>> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .as_ref()
                .map(|record| crate::cf_api::CfDnsRecord {
                    id: record.id.clone(),
                    name: record.name.clone(),
                    content: record.content.clone(),
                }))
        }

        async fn create(
            &self,
            _profile: &crate::types::Profile,
            _zone_id: &str,
            _hostname: &str,
            _tunnel_id: &str,
        ) -> crate::error::AppResult<String> {
            unreachable!()
        }

        async fn delete(
            &self,
            _profile: &crate::types::Profile,
            _zone_id: &str,
            record_id: &str,
        ) -> crate::error::AppResult<()> {
            let mut record = self.0.lock().unwrap();
            assert_eq!(
                record.as_ref().map(|record| record.id.as_str()),
                Some(record_id)
            );
            *record = None;
            Ok(())
        }
    }

    #[tokio::test]
    async fn cleanup_is_idempotent_and_preserves_persistent_and_catchall_rules() {
        let root =
            std::env::temp_dir().join(format!("flaredeck-route-cleanup-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&root).await.unwrap();
        let config_path = root.join("config.yml");
        let store_path = root.join("temporary-routes.json");
        let route = Route {
            hostname: "hook.example.test".into(),
            service: "http://127.0.0.1:3000".into(),
            path: Some("/hook".into()),
            mode: Some("temporary".into()),
        };
        let mut config = config();
        super::insert_temporary_ingress(&mut config, &route, "http://127.0.0.1:4000").unwrap();
        super::save_config(&config_path.to_string_lossy(), &config)
            .await
            .unwrap();
        let mut owned = record();
        owned.dns_record_id = Some("dns-owned".into());
        super::save_store(
            &store_path,
            &super::TemporaryRouteStore {
                schema_version: 1,
                routes: vec![owned.clone()],
            },
        )
        .await
        .unwrap();
        let profile = crate::types::Profile {
            id: "profile".into(),
            name: "Profile".into(),
            tunnel_name: "tunnel".into(),
            config_path: config_path.to_string_lossy().into_owned(),
            wsl_host: false,
            account_id: None,
            zone_id: Some("zone".into()),
            zone_name: None,
            cert_path: None,
            has_api_token: true,
        };
        let dns = OwnedDns(std::sync::Mutex::new(Some(crate::cf_api::CfDnsRecord {
            id: "dns-owned".into(),
            name: "hook.example.test".into(),
            content: "tunnel.cfargotunnel.com".into(),
        })));
        super::cleanup_record(&dns, &store_path, &profile, &mut owned)
            .await
            .unwrap();
        super::cleanup_record(&dns, &store_path, &profile, &mut owned)
            .await
            .unwrap();
        assert_eq!(owned.state, TemporaryRouteState::Cleaned);
        let config = super::load_config(&profile.config_path).await.unwrap();
        let ingress = config.ingress.unwrap();
        assert_eq!(ingress.len(), 2);
        assert_eq!(
            ingress[0].hostname.as_deref(),
            Some("persistent.example.test")
        );
        assert_eq!(ingress[1].service, "http_status:404");
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    #[tokio::test]
    async fn corrupt_ownership_state_fails_closed() {
        let root =
            std::env::temp_dir().join(format!("flaredeck-route-corrupt-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&root).await.unwrap();
        let store = root.join("temporary-routes.json");
        tokio::fs::write(&store, b"not valid ownership state")
            .await
            .unwrap();
        assert!(super::list_routes(&store, None).await.is_err());
        let _ = tokio::fs::remove_dir_all(root).await;
    }

    fn config() -> CloudflaredConfig {
        CloudflaredConfig {
            tunnel: Some("tunnel".into()),
            credentials_file: None,
            ingress: Some(vec![
                IngressRule {
                    hostname: Some("persistent.example.test".into()),
                    path: None,
                    service: "http://127.0.0.1:3000".into(),
                    origin_request: None,
                },
                IngressRule {
                    hostname: None,
                    path: None,
                    service: "http_status:404".into(),
                    origin_request: None,
                },
            ]),
            extras: serde_yaml::Mapping::new(),
        }
    }

    fn record() -> TemporaryRouteRecord {
        TemporaryRouteRecord {
            id: "route".into(),
            workspace_id: "workspace".into(),
            session_id: "session".into(),
            profile_id: "profile".into(),
            zone_id: "zone".into(),
            tunnel_id: "tunnel".into(),
            hostname: "hook.example.test".into(),
            path: Some("/hook".into()),
            origin: "http://127.0.0.1:3000".into(),
            proxy_service: "http://127.0.0.1:4000".into(),
            dns_record_id: None,
            proxy_pid: None,
            proxy_started_at_seconds: None,
            proxy_executable: None,
            created_at: Utc::now(),
            expires_at: Utc::now(),
            state: TemporaryRouteState::Active,
            cleanup_error: None,
        }
    }
}
