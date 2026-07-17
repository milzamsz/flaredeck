use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::application::{
    audit_service::audit_path,
    runtime_service::RuntimeLogLine,
    session_service::{self, SessionRecord, SessionState},
    temporary_route_service::{self, TemporaryRouteRecord, TemporaryRouteState},
    trust_service::{fingerprint, has_approval, is_approved, save_desktop_approval},
    webhook_service::{self, WebhookEvent},
    workspace_registry,
    workspace_service::{discover, Readiness, WorkspaceManifest},
};
use crate::error::{AppError, AppResult};
use crate::types::{
    TemporaryRouteView, WebhookEventView, WorkspaceAuditEventView, WorkspaceEnvironmentLiteralView,
    WorkspaceListItemView, WorkspaceRouteView, WorkspaceSessionView, WorkspaceTrustView,
};

fn state_path(app: &AppHandle, name: &str) -> AppResult<PathBuf> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::Other(error.to_string()))?
        .join(name))
}

async fn inspect(path: &Path) -> AppResult<(PathBuf, WorkspaceManifest, String)> {
    let (root, manifest) = discover(path).await?;
    let raw = tokio::fs::read_to_string(root.join(".flaredeck/project.yaml")).await?;
    Ok((root, manifest, fingerprint(&raw)?))
}

async fn view(
    trust_path: &Path,
    root: PathBuf,
    manifest: WorkspaceManifest,
    fingerprint: String,
) -> WorkspaceTrustView {
    let trusted = is_approved(trust_path, &root, &fingerprint).await;
    let approval_state = if trusted {
        "trusted"
    } else if has_approval(trust_path, &root).await {
        "changed"
    } else {
        "approval_required"
    };
    let readiness = match manifest.ready {
        Readiness::Tcp { host, port, .. } => format!("tcp://{host}:{port}"),
        Readiness::Http { url, .. } => url,
    };
    let mut environment_names = manifest
        .environment
        .as_ref()
        .and_then(|environment| environment.passthrough.clone())
        .unwrap_or_default();
    let environment_values = manifest
        .environment
        .as_ref()
        .and_then(|environment| environment.values.as_ref())
        .map(|values| {
            values
                .iter()
                .map(|(name, value)| WorkspaceEnvironmentLiteralView {
                    name: name.clone(),
                    value: value.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    environment_names.extend(environment_values.iter().map(|entry| entry.name.clone()));
    environment_names.sort();
    environment_names.dedup();
    let lifecycle = manifest.lifecycle.as_ref();
    let ensure_tunnel = lifecycle
        .and_then(|value| value.ensure_tunnel)
        .unwrap_or(true);
    let start_runtime = lifecycle
        .and_then(|value| value.start_runtime)
        .unwrap_or(true);
    WorkspaceTrustView {
        root: root.to_string_lossy().into_owned(),
        workspace_id: manifest
            .project
            .id
            .clone()
            .unwrap_or_else(|| manifest.project.name.clone()),
        project_name: manifest.project.name,
        profile: manifest
            .profile
            .id
            .or(manifest.profile.name)
            .unwrap_or_default(),
        executable: manifest.runtime.executable,
        args: manifest.runtime.args,
        working_directory: manifest.runtime.working_directory,
        readiness,
        routes: manifest
            .exposure
            .routes
            .into_iter()
            .map(|route| WorkspaceRouteView {
                hostname: route.hostname,
                origin: route.service,
                path: route.path,
                mode: route.mode.unwrap_or_else(|| "persistent".into()),
            })
            .collect(),
        environment_names,
        environment_values,
        lifecycle: vec![
            format!("Start runtime: {start_runtime}"),
            format!("Ensure tunnel: {ensure_tunnel}"),
            format!(
                "Stop owned runtime: {}",
                lifecycle
                    .and_then(|value| value.stop_runtime_on_session_stop)
                    .unwrap_or(true)
            ),
            format!(
                "Stop session-started tunnel: {}",
                lifecycle
                    .and_then(|value| value.stop_tunnel_if_started_by_session)
                    .unwrap_or(true)
            ),
            format!(
                "Remove temporary routes: {}",
                lifecycle
                    .and_then(|value| value.remove_temporary_routes)
                    .unwrap_or(true)
            ),
        ],
        capabilities: [
            start_runtime.then_some("Execute declared runtime"),
            ensure_tunnel.then_some("Ensure selected tunnel"),
            Some("Verify declared routes"),
            Some("Stop session-owned resources"),
            Some("Read redacted logs"),
        ]
        .into_iter()
        .flatten()
        .map(str::to_owned)
        .collect(),
        fingerprint,
        approval_state: approval_state.into(),
        trusted,
    }
}

fn session_view(record: &SessionRecord) -> WorkspaceSessionView {
    let state = match record.state {
        SessionState::Stopped => "stopped",
        SessionState::Starting => "starting",
        SessionState::Healthy => "healthy",
        SessionState::Failed => "failed",
        SessionState::Stopping => "stopping",
        SessionState::CleanupIncomplete => "cleanup_incomplete",
    };
    WorkspaceSessionView {
        id: record.id.clone(),
        workspace_id: record.workspace_id.clone(),
        profile_id: record.profile_id.clone(),
        state: state.into(),
        runtime_ownership: if record.runtime_owned {
            "session"
        } else {
            "external"
        }
        .into(),
        tunnel_ownership: if record.tunnel_started_by_session {
            "session"
        } else {
            "external_or_disabled"
        }
        .into(),
        public_urls: record.public_urls.clone(),
        started_at: record.started_at.to_rfc3339(),
        cleanup_required: record.state == SessionState::CleanupIncomplete,
    }
}

fn temporary_route_view(record: TemporaryRouteRecord) -> TemporaryRouteView {
    let state = match record.state {
        TemporaryRouteState::Creating => "creating",
        TemporaryRouteState::Active => "active",
        TemporaryRouteState::CleanupIncomplete => "cleanup_incomplete",
        TemporaryRouteState::Cleaned => "cleaned",
    };
    TemporaryRouteView {
        id: record.id,
        session_id: record.session_id,
        hostname: record.hostname,
        path: record.path,
        origin: record.origin,
        state: state.into(),
        created_at: record.created_at.to_rfc3339(),
        expires_at: record.expires_at.to_rfc3339(),
        cleanup_error: record.cleanup_error,
    }
}

fn webhook_event_view(event: WebhookEvent) -> WebhookEventView {
    WebhookEventView {
        id: event.id,
        route_id: event.route_id,
        timestamp: event.timestamp.to_rfc3339(),
        method: event.method,
        path: event.path,
        headers: event.headers,
        content_type: event.content_type,
        body: event.body,
        body_state: event.body_state,
        response_status: event.response_status,
        redaction_version: event.redaction_version,
    }
}

#[tauri::command]
pub async fn workspace_inspect(app: AppHandle, path: String) -> AppResult<WorkspaceTrustView> {
    let (root, manifest, fingerprint) = inspect(Path::new(&path)).await?;
    Ok(view(
        &state_path(&app, "trust-approvals.json")?,
        root,
        manifest,
        fingerprint,
    )
    .await)
}

#[tauri::command]
pub async fn workspace_approve(app: AppHandle, path: String) -> AppResult<WorkspaceTrustView> {
    let (root, manifest, fingerprint) = inspect(Path::new(&path)).await?;
    let trust = state_path(&app, "trust-approvals.json")?;
    save_desktop_approval(&trust, &root, fingerprint.clone()).await?;
    workspace_registry::register(&root, &state_path(&app, "workspaces.json")?).await?;
    Ok(view(&trust, root, manifest, fingerprint).await)
}

#[tauri::command]
pub async fn workspace_list(app: AppHandle) -> AppResult<Vec<WorkspaceListItemView>> {
    let trust = state_path(&app, "trust-approvals.json")?;
    let sessions = state_path(&app, "active-sessions.json")?;
    temporary_route_service::reconcile_expired(&temporary_route_service::route_store_path(
        &sessions,
    ))
    .await?;
    let mut result = Vec::new();
    for path in workspace_registry::list(&state_path(&app, "workspaces.json")?).await? {
        let root = PathBuf::from(&path);
        match inspect(&root).await {
            Ok((root, manifest, digest)) => {
                let detail = view(&trust, root.clone(), manifest, digest).await;
                let active_session =
                    session_service::session_status(&sessions, &detail.workspace_id)
                        .await?
                        .as_ref()
                        .map(session_view);
                result.push(WorkspaceListItemView {
                    root: root.to_string_lossy().into_owned(),
                    workspace_id: detail.workspace_id,
                    project_name: detail.project_name,
                    profile: detail.profile,
                    validation_state: "valid".into(),
                    approval_state: detail.approval_state,
                    trusted: detail.trusted,
                    active_session,
                });
            }
            Err(_) => result.push(WorkspaceListItemView {
                root: path.clone(),
                workspace_id: path.clone(),
                project_name: root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Invalid workspace")
                    .into(),
                profile: String::new(),
                validation_state: "invalid".into(),
                approval_state: "approval_required".into(),
                trusted: false,
                active_session: None,
            }),
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn workspace_session_start(
    app: AppHandle,
    workspace_id: String,
) -> AppResult<WorkspaceSessionView> {
    let root =
        workspace_registry::resolve(&state_path(&app, "workspaces.json")?, &workspace_id).await?;
    let record = session_service::start_session(
        &root,
        &state_path(&app, "trust-approvals.json")?,
        &state_path(&app, "active-sessions.json")?,
    )
    .await?;
    Ok(session_view(&record))
}

#[tauri::command]
pub async fn workspace_session_status(
    app: AppHandle,
    workspace_id: String,
) -> AppResult<Option<WorkspaceSessionView>> {
    Ok(
        session_service::session_status(&state_path(&app, "active-sessions.json")?, &workspace_id)
            .await?
            .as_ref()
            .map(session_view),
    )
}

#[tauri::command]
pub async fn workspace_session_stop(
    app: AppHandle,
    session_id: String,
) -> AppResult<WorkspaceSessionView> {
    Ok(session_view(
        &session_service::stop_session(&state_path(&app, "active-sessions.json")?, &session_id)
            .await?,
    ))
}

#[tauri::command]
pub async fn workspace_session_logs(
    app: AppHandle,
    session_id: String,
    tail: usize,
) -> AppResult<Vec<RuntimeLogLine>> {
    session_service::session_logs(
        &state_path(&app, "active-sessions.json")?,
        &session_id,
        tail.min(200),
    )
    .await
}

#[tauri::command]
pub async fn workspace_temporary_routes(
    app: AppHandle,
    session_id: String,
) -> AppResult<Vec<TemporaryRouteView>> {
    let session_store = state_path(&app, "active-sessions.json")?;
    temporary_route_service::list_routes(
        &temporary_route_service::route_store_path(&session_store),
        Some(&session_id),
    )
    .await
    .map(|routes| routes.into_iter().map(temporary_route_view).collect())
}

#[tauri::command]
pub async fn workspace_temporary_routes_reconcile(
    app: AppHandle,
) -> AppResult<Vec<TemporaryRouteView>> {
    let session_store = state_path(&app, "active-sessions.json")?;
    temporary_route_service::reconcile_expired(&temporary_route_service::route_store_path(
        &session_store,
    ))
    .await
    .map(|routes| routes.into_iter().map(temporary_route_view).collect())
}

#[tauri::command]
pub async fn workspace_webhook_events(
    app: AppHandle,
    route_id: String,
    limit: usize,
) -> AppResult<Vec<WebhookEventView>> {
    let session_store = state_path(&app, "active-sessions.json")?;
    let route_store = temporary_route_service::route_store_path(&session_store);
    if !temporary_route_service::list_routes(&route_store, None)
        .await?
        .iter()
        .any(|route| route.id == route_id)
    {
        return Err(AppError::Other("temporary route not found".into()));
    }
    webhook_service::list_events(
        &temporary_route_service::event_store_path(&route_store, &route_id)?,
        &route_id,
        limit.min(100),
    )
    .await
    .map(|events| events.into_iter().map(webhook_event_view).collect())
}

#[tauri::command]
pub async fn workspace_webhook_replay(
    app: AppHandle,
    route_id: String,
    event_id: String,
) -> AppResult<u16> {
    let session_store = state_path(&app, "active-sessions.json")?;
    let route_store = temporary_route_service::route_store_path(&session_store);
    let route = temporary_route_service::list_routes(&route_store, None)
        .await?
        .into_iter()
        .find(|route| route.id == route_id)
        .ok_or_else(|| AppError::Other("temporary route not found".into()))?;
    if route.state != TemporaryRouteState::Active || route.expires_at <= chrono::Utc::now() {
        return Err(AppError::Other(
            "temporary route is not active for replay".into(),
        ));
    }
    let active_session = session_service::session_status(&session_store, &route.session_id).await?;
    if active_session.is_none_or(|session| {
        matches!(
            session.state,
            SessionState::Stopped | SessionState::Failed | SessionState::CleanupIncomplete
        )
    }) {
        return Err(AppError::Other(
            "temporary route session is not active for replay".into(),
        ));
    }
    webhook_service::replay_event(
        &temporary_route_service::event_store_path(&route_store, &route.id)?,
        &route.id,
        &event_id,
        &route.origin,
    )
    .await
}

#[tauri::command]
pub async fn workspace_audit(
    app: AppHandle,
    workspace_id: String,
) -> AppResult<Vec<WorkspaceAuditEventView>> {
    let raw =
        match tokio::fs::read_to_string(audit_path(&state_path(&app, "active-sessions.json")?))
            .await
        {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
    Ok(raw
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| {
            event.get("workspaceId").and_then(serde_json::Value::as_str) == Some(&workspace_id)
        })
        .take(50)
        .map(|event| WorkspaceAuditEventView {
            timestamp: event
                .get("timestamp")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
            operation: event
                .get("operation")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
            result: event
                .get("result")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
            session_id: event
                .get("sessionId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
            correlation_id: event
                .get("correlationId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .into(),
        })
        .collect::<Vec<_>>())
}
