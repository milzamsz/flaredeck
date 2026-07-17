use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::application::trust_service::{fingerprint, is_approved};
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceManifest {
    pub version: u8,
    pub project: Project,
    pub profile: ProfileRef,
    pub runtime: Runtime,
    pub ready: Readiness,
    pub exposure: Exposure,
    pub lifecycle: Option<Lifecycle>,
    pub environment: Option<Environment>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    pub id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileRef {
    pub id: Option<String>,
    pub name: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_working_directory", rename = "workingDirectory")]
    pub working_directory: String,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Readiness {
    Tcp {
        host: String,
        port: u16,
        #[serde(
            default = "default_readiness_interval",
            rename = "intervalMilliseconds"
        )]
        interval_milliseconds: u64,
        #[serde(default = "default_readiness_timeout", rename = "timeoutSeconds")]
        timeout_seconds: u64,
    },
    Http {
        url: String,
        #[serde(default = "default_expected_status", rename = "expectedStatus")]
        expected_status: [u16; 2],
        #[serde(
            default = "default_readiness_interval",
            rename = "intervalMilliseconds"
        )]
        interval_milliseconds: u64,
        #[serde(default = "default_readiness_timeout", rename = "timeoutSeconds")]
        timeout_seconds: u64,
    },
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exposure {
    pub routes: Vec<Route>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub hostname: String,
    pub service: String,
    pub path: Option<String>,
    pub mode: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    #[serde(rename = "startRuntime")]
    pub start_runtime: Option<bool>,
    #[serde(rename = "ensureTunnel")]
    pub ensure_tunnel: Option<bool>,
    #[serde(rename = "stopRuntimeOnSessionStop")]
    pub stop_runtime_on_session_stop: Option<bool>,
    #[serde(rename = "stopTunnelIfStartedBySession")]
    pub stop_tunnel_if_started_by_session: Option<bool>,
    #[serde(rename = "removeTemporaryRoutes")]
    pub remove_temporary_routes: Option<bool>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub passthrough: Option<Vec<String>>,
    pub values: Option<std::collections::BTreeMap<String, String>>,
}

fn default_working_directory() -> String {
    ".".into()
}

fn default_readiness_interval() -> u64 {
    500
}

fn default_readiness_timeout() -> u64 {
    60
}

fn default_expected_status() -> [u16; 2] {
    [200, 299]
}

pub fn parse_manifest(raw: &str) -> AppResult<WorkspaceManifest> {
    let manifest: WorkspaceManifest = serde_yaml::from_str(raw)?;
    validate(&manifest)?;
    Ok(manifest)
}

pub async fn discover(root: &Path) -> AppResult<(PathBuf, WorkspaceManifest)> {
    let root = root.canonicalize()?;
    let manifest_path = root.join(".flaredeck/project.yaml");
    let raw = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest = parse_manifest(&raw)?;
    resolve_working_directory(&root, &manifest.runtime.working_directory)?;
    Ok((root, manifest))
}

pub async fn authorize_start(
    root: &Path,
    approval_path: &Path,
) -> AppResult<(PathBuf, WorkspaceManifest)> {
    let (root, manifest) = discover(root).await?;
    let raw = tokio::fs::read_to_string(root.join(".flaredeck/project.yaml")).await?;
    let fingerprint = fingerprint(&raw)?;
    if !is_approved(approval_path, &root, &fingerprint).await {
        return Err(AppError::Other(
            "workspace requires local desktop approval".into(),
        ));
    }
    Ok((root, manifest))
}

pub fn resolve_working_directory(root: &Path, relative: &str) -> AppResult<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(AppError::Other(
            "working directory must stay inside the workspace".into(),
        ));
    }
    let root = root.canonicalize()?;
    let resolved = root.join(path);
    if resolved.exists() {
        let canonical = resolved.canonicalize()?;
        if !canonical.starts_with(&root) {
            return Err(AppError::Other(
                "working directory must stay inside the workspace".into(),
            ));
        }
        return Ok(canonical);
    }
    Ok(resolved)
}

fn validate(manifest: &WorkspaceManifest) -> AppResult<()> {
    if manifest.version != 1
        || manifest.project.name.trim().is_empty()
        || manifest.runtime.executable.trim().is_empty()
        || manifest.exposure.routes.is_empty()
    {
        return Err(AppError::Other("invalid workspace manifest".into()));
    }
    if manifest.profile.id.as_deref().unwrap_or("").is_empty()
        && manifest.profile.name.as_deref().unwrap_or("").is_empty()
    {
        return Err(AppError::Other(
            "workspace manifest must select a profile".into(),
        ));
    }
    if matches!(
        manifest.runtime.executable.as_str(),
        "sh" | "bash" | "cmd" | "powershell"
    ) && manifest
        .runtime
        .args
        .first()
        .is_some_and(|arg| arg == "-c" || arg == "/c" || arg == "-Command")
    {
        return Err(AppError::Other("shell command mode is not allowed".into()));
    }
    match &manifest.ready {
        Readiness::Tcp {
            host,
            interval_milliseconds,
            timeout_seconds,
            ..
        } if matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1")
            && is_valid_readiness_timing(*interval_milliseconds, *timeout_seconds) => {}
        Readiness::Http {
            url,
            expected_status,
            interval_milliseconds,
            timeout_seconds,
        } if is_local_http_url(url)
            && expected_status[0] <= expected_status[1]
            && is_valid_readiness_timing(*interval_milliseconds, *timeout_seconds) => {}
        _ => return Err(AppError::Other("readiness target must be local".into())),
    }
    if manifest
        .exposure
        .routes
        .iter()
        .any(|route| !is_local_http_url(&route.service))
    {
        return Err(AppError::Other(
            "route service must be a local HTTP URL".into(),
        ));
    }
    if manifest
        .exposure
        .routes
        .iter()
        .any(|route| route.mode.as_deref() == Some("temporary"))
        && manifest
            .lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.ensure_tunnel)
            == Some(false)
    {
        return Err(AppError::Other(
            "temporary routes require lifecycle.ensureTunnel".into(),
        ));
    }
    if manifest
        .environment
        .as_ref()
        .and_then(|environment| environment.values.as_ref())
        .is_some_and(|values| values.keys().any(|key| is_sensitive_name(key)))
    {
        return Err(AppError::Other(
            "manifest environment values must not contain secrets".into(),
        ));
    }
    resolve_working_directory(Path::new("."), &manifest.runtime.working_directory)?;
    Ok(())
}

fn is_valid_readiness_timing(interval_milliseconds: u64, timeout_seconds: u64) -> bool {
    (100..=10_000).contains(&interval_milliseconds) && (1..=900).contains(&timeout_seconds)
}

fn is_local_http_url(value: &str) -> bool {
    value.starts_with("http://127.0.0.1:")
        || value.starts_with("http://localhost:")
        || value.starts_with("http://[::1]:")
}

fn is_sensitive_name(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    ["TOKEN", "SECRET", "PASSWORD", "PRIVATE_KEY", "API_KEY"]
        .iter()
        .any(|needle| name.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{authorize_start, discover, parse_manifest, resolve_working_directory};
    use crate::application::trust_service::{fingerprint, save_desktop_approval};
    use std::path::Path;
    const VALID: &str = "version: 1\nproject: { name: app }\nprofile: { id: profile-1 }\nruntime: { executable: cargo, args: [run] }\nready: { type: tcp, host: 127.0.0.1, port: 3000 }\nexposure: { routes: [{ hostname: app.example.com, service: http://127.0.0.1:3000 }] }\n";
    #[test]
    fn parses_valid_manifest() {
        assert_eq!(parse_manifest(VALID).unwrap().runtime.args, ["run"]);
    }
    #[test]
    fn checked_in_integration_fixture_is_valid_and_secret_free() {
        let raw = include_str!("../../../examples/fixture-workspace/.flaredeck/project.yaml");
        let manifest = parse_manifest(raw).unwrap();
        assert_eq!(
            manifest.project.id.as_deref(),
            Some("flaredeck-integration-fixture")
        );
        assert!(!raw.to_ascii_uppercase().contains("TOKEN"));
    }
    #[test]
    fn rejects_shell_mode_and_traversal() {
        assert!(parse_manifest(&VALID.replace(
            "executable: cargo, args: [run]",
            "executable: bash, args: [-c, whoami]"
        ))
        .is_err());
        assert!(resolve_working_directory(Path::new("/workspace"), "../escape").is_err());
    }
    #[test]
    fn rejects_non_local_targets_and_secret_values() {
        assert!(parse_manifest(&VALID.replace("127.0.0.1", "example.com")).is_err());
        assert!(parse_manifest(
            &(VALID.to_owned() + "environment: { values: { API_TOKEN: nope } }\n")
        )
        .is_err());
    }
    #[tokio::test]
    async fn discovers_only_the_canonical_manifest_path() {
        let root =
            std::env::temp_dir().join(format!("flaredeck-workspace-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(root.join(".flaredeck"))
            .await
            .unwrap();
        tokio::fs::write(root.join(".flaredeck/project.yaml"), VALID)
            .await
            .unwrap();
        let (canonical, manifest) = discover(&root).await.unwrap();
        assert!(canonical.is_absolute());
        assert_eq!(manifest.project.name, "app");
        let _ = tokio::fs::remove_dir_all(root).await;
    }
    #[tokio::test]
    async fn start_authorization_fails_closed_without_current_approval() {
        let root =
            std::env::temp_dir().join(format!("flaredeck-authorize-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(root.join(".flaredeck"))
            .await
            .unwrap();
        let manifest = root.join(".flaredeck/project.yaml");
        tokio::fs::write(&manifest, VALID).await.unwrap();
        let approval = root.join("approval.json");
        assert!(authorize_start(&root, &approval).await.is_err());
        save_desktop_approval(&approval, &root, fingerprint(VALID).unwrap())
            .await
            .unwrap();
        assert!(authorize_start(&root, &approval).await.is_ok());
        let _ = tokio::fs::remove_dir_all(root).await;
    }
}
