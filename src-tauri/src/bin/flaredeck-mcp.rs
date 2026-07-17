use std::collections::HashMap;
use std::time::Duration;

use flaredeck_lib::application::{
    session_service::{
        session_logs, session_status, start_session_cancellable, stop_session, SessionRecord,
    },
    state_paths::{session_store_path, trust_store_path, workspace_registry_path},
    temporary_route_service,
    trust_service::{fingerprint, is_approved},
    webhook_service, workspace_registry,
    workspace_service::{discover, Readiness, WorkspaceManifest},
};
use serde::Serialize;
use serde_json::{json, Map, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, watch};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolError {
    code: &'static str,
    message: String,
    retryable: bool,
    required_action: Option<&'static str>,
    correlation_id: String,
}

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let (responses, mut response_queue) = mpsc::unbounded_channel::<Value>();
    let writer = tokio::spawn(async move {
        let mut stdout = tokio::io::BufWriter::new(tokio::io::stdout());
        while let Some(response) = response_queue.recv().await {
            if stdout
                .write_all(response.to_string().as_bytes())
                .await
                .is_err()
                || stdout.write_all(b"\n").await.is_err()
                || stdout.flush().await.is_err()
            {
                break;
            }
        }
    });
    let (completed, mut completions) = mpsc::unbounded_channel::<String>();
    let mut pending = HashMap::<String, watch::Sender<bool>>::new();
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    loop {
        let line = tokio::select! {
            line = lines.next_line() => match line {
                Ok(Some(line)) => line,
                _ => break,
            },
            Some(key) = completions.recv() => {
                pending.remove(&key);
                continue;
            }
        };
        let request = match serde_json::from_str::<Value>(&line) {
            Ok(request) => request,
            Err(_) => {
                let _ = responses.send(error_response(&Value::Null, -32700, "parse error"));
                continue;
            }
        };
        if request.get("method").and_then(Value::as_str) == Some("notifications/cancelled") {
            if let Some(request_id) = request.pointer("/params/requestId") {
                if let Some(sender) = pending.get(&request_id.to_string()) {
                    let _ = sender.send(true);
                }
            }
            continue;
        }
        let Some(id) = request.get("id") else {
            continue;
        };
        let key = id.to_string();
        let (cancel, cancellation) = watch::channel(false);
        pending.insert(key.clone(), cancel);
        let responses = responses.clone();
        let completed = completed.clone();
        tokio::spawn(async move {
            let response = process_request(request, cancellation).await;
            let _ = responses.send(response);
            let _ = completed.send(key);
        });
    }
    drop(responses);
    let _ = writer.await;
}

async fn process_request(request: Value, cancellation: watch::Receiver<bool>) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("initialize") => result_response(
            &id,
            json!({
                "protocolVersion": negotiated_protocol_version(&request),
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "flaredeck", "version": env!("CARGO_PKG_VERSION") }
            }),
        ),
        Some("tools/list") => result_response(&id, json!({ "tools": tools() })),
        Some("tools/call") => {
            let correlation_id = format!("corr_{}", uuid::Uuid::new_v4());
            match call_tool(request.get("params"), &correlation_id, cancellation).await {
                Ok(content) => result_response(
                    &id,
                    json!({ "content": [{ "type": "text", "text": content.to_string() }] }),
                ),
                Err(error) => result_response(
                    &id,
                    json!({
                        "content": [{ "type": "text", "text": serde_json::to_string(&error).unwrap_or_else(|_| "{\"code\":\"INTERNAL_ERROR\"}".into()) }],
                        "isError": true
                    }),
                ),
            }
        }
        _ => error_response(&id, -32601, "method not found"),
    }
}

async fn call_tool(
    params: Option<&Value>,
    correlation_id: &str,
    mut cancellation: watch::Receiver<bool>,
) -> Result<Value, ToolError> {
    let result = call_tool_inner(params, correlation_id, &mut cancellation).await;
    result.map_err(|message| classify_error(message, correlation_id))
}

async fn call_tool_inner(
    params: Option<&Value>,
    correlation_id: &str,
    cancellation: &mut watch::Receiver<bool>,
) -> Result<Value, String> {
    let params = params.ok_or("missing tool parameters")?;
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("missing tool name")?;
    let empty = Map::new();
    let arguments = match params.get("arguments") {
        Some(value) => value.as_object().ok_or("invalid arguments")?,
        None => &empty,
    };
    validate_arguments(name, arguments)?;

    match name {
        "doctor" => Ok(json!({
            "cloudflaredInstalled": flaredeck_lib::cloudflared::resolve_cloudflared_path().is_some(),
            "version": env!("CARGO_PKG_VERSION"),
            "transport": "stdio",
            "temporaryRoutesReconciled": temporary_route_service::reconcile_expired(
                &temporary_route_service::route_store_path(
                    &session_store_path().map_err(|error| error.to_string())?
                )
            ).await.map_err(|error| error.to_string())?.len(),
            "correlationId": correlation_id
        })),
        "workspace_list" => workspace_list(arguments, correlation_id).await,
        "workspace_status" => {
            let selector = selector(arguments, "workspace")?;
            let root = resolve_workspace(selector).await?;
            workspace_status(&root, correlation_id).await
        }
        "session_start" => {
            let selector = selector(arguments, "workspace")?;
            if arguments
                .get("waitForHealthy")
                .is_some_and(|value| value != &Value::Bool(true))
            {
                return Err("waitForHealthy must be true".into());
            }
            let root = resolve_workspace(selector).await?;
            let record = start_session_cancellable(
                &root,
                &trust_store_path().map_err(|error| error.to_string())?,
                &session_store_path().map_err(|error| error.to_string())?,
                cancellation,
                Duration::from_secs(120),
            )
            .await
            .map_err(|error| error.to_string())?;
            Ok(safe_session(&record, correlation_id))
        }
        "session_status" | "public_url_get" | "logs_read" | "session_stop" | "health_check" => {
            session_tool(name, arguments, correlation_id).await
        }
        "webhook_event_list" | "webhook_event_get" => {
            webhook_tool(name, arguments, correlation_id).await
        }
        _ => Err("tool is not available for this request".into()),
    }
}

fn validate_arguments(name: &str, arguments: &Map<String, Value>) -> Result<(), String> {
    let allowed: &[&str] = match name {
        "doctor" => &[],
        "workspace_list" => &["state", "limit"],
        "workspace_status" => &["workspace"],
        "session_start" => &["workspace", "waitForHealthy"],
        "session_status" | "session_stop" | "public_url_get" | "health_check" => &["session"],
        "logs_read" => &["session", "source", "tail"],
        "webhook_event_list" => &["route", "limit"],
        "webhook_event_get" => &["route", "event"],
        _ => return Err("tool is not available for this request".into()),
    };
    if arguments.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("unknown tool property".into());
    }
    Ok(())
}

async fn workspace_list(
    arguments: &Map<String, Value>,
    correlation_id: &str,
) -> Result<Value, String> {
    let limit = bounded_integer(arguments, "limit", 50, 1, 100)? as usize;
    let filter = arguments
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("all");
    if !["all", "trusted", "approval_required", "running", "invalid"].contains(&filter) {
        return Err("invalid workspace state filter".into());
    }
    let registry = workspace_registry_path().map_err(|error| error.to_string())?;
    let trust = trust_store_path().map_err(|error| error.to_string())?;
    let sessions = session_store_path().map_err(|error| error.to_string())?;
    let paths = workspace_registry::list(&registry)
        .await
        .map_err(|error| error.to_string())?;
    let mut workspaces = Vec::new();
    for path in paths {
        let root = std::path::Path::new(&path);
        let entry = match discover(root).await {
            Ok((root, manifest)) => {
                let raw = tokio::fs::read_to_string(root.join(".flaredeck/project.yaml"))
                    .await
                    .map_err(|error| error.to_string())?;
                let digest = fingerprint(&raw).map_err(|error| error.to_string())?;
                let trusted = is_approved(&trust, &root, &digest).await;
                let id = manifest
                    .project
                    .id
                    .clone()
                    .unwrap_or_else(|| manifest.project.name.clone());
                let active = session_status(&sessions, &id)
                    .await
                    .map_err(|error| error.to_string())?;
                let state = if active.as_ref().is_some_and(|session| {
                    format!("{:?}", session.state).eq_ignore_ascii_case("healthy")
                }) {
                    "running"
                } else if trusted {
                    "trusted"
                } else {
                    "approval_required"
                };
                json!({
                    "id": id,
                    "name": manifest.project.name,
                    "state": state,
                    "trusted": trusted,
                    "profile": manifest.profile.id.or(manifest.profile.name),
                    "activeSession": active.as_ref().map(|session| safe_session(session, correlation_id)),
                    "path": "registered"
                })
            }
            Err(_) => json!({ "state": "invalid", "path": "registered" }),
        };
        if filter == "all" || entry.get("state").and_then(Value::as_str) == Some(filter) {
            workspaces.push(entry);
        }
        if workspaces.len() == limit {
            break;
        }
    }
    Ok(json!({ "workspaces": workspaces, "correlationId": correlation_id }))
}

async fn workspace_status(root: &std::path::Path, correlation_id: &str) -> Result<Value, String> {
    let (root, manifest) = discover(root).await.map_err(|error| error.to_string())?;
    let raw = tokio::fs::read_to_string(root.join(".flaredeck/project.yaml"))
        .await
        .map_err(|error| error.to_string())?;
    let digest = fingerprint(&raw).map_err(|error| error.to_string())?;
    let trusted = is_approved(
        &trust_store_path().map_err(|error| error.to_string())?,
        &root,
        &digest,
    )
    .await;
    let id = manifest
        .project
        .id
        .clone()
        .unwrap_or_else(|| manifest.project.name.clone());
    let active = session_status(
        &session_store_path().map_err(|error| error.to_string())?,
        &id,
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(safe_workspace_status(
        manifest,
        trusted,
        active.as_ref(),
        correlation_id,
    ))
}

async fn session_tool(
    name: &str,
    arguments: &Map<String, Value>,
    correlation_id: &str,
) -> Result<Value, String> {
    let session = selector(arguments, "session")?;
    let store = session_store_path().map_err(|error| error.to_string())?;
    match name {
        "session_status" => session_status(&store, session)
            .await
            .map_err(|error| error.to_string())?
            .as_ref()
            .map(|record| safe_session(record, correlation_id))
            .ok_or("session not found".into()),
        "public_url_get" => session_status(&store, session)
            .await
            .map_err(|error| error.to_string())?
            .map(|record| async move {
                let route_store = temporary_route_service::route_store_path(&store);
                let temporary = temporary_route_service::list_routes(
                    &route_store,
                    Some(&record.id),
                )
                .await
                .map_err(|error| error.to_string())?;
                let urls = record.public_urls.into_iter().filter_map(|url| {
                    let hostname = url.strip_prefix("https://")?.split('/').next()?;
                    let route = temporary
                        .iter()
                        .find(|route| route.hostname.eq_ignore_ascii_case(hostname));
                    Some(json!({
                        "url": url,
                        "hostname": hostname,
                        "ownership": if route.is_some() { "temporary" } else { "persistent" },
                        "health": "configured",
                        "expiresAt": route.map(|route| route.expires_at)
                    }))
                }).collect::<Vec<_>>();
                Ok(json!({ "urls": urls, "correlationId": correlation_id }))
            })
            .ok_or("session not found")?
            .await,
        "health_check" => session_status(&store, session)
            .await
            .map_err(|error| error.to_string())?
            .map(|record| json!({
                "state": record.state,
                "checks": {
                    "runtime": if record.runtime_owned { "owned" } else { "external" },
                    "tunnel": if record.tunnel_started_by_session { "owned" } else { "observed_or_disabled" },
                    "routes": "configured"
                },
                "checkedAt": chrono::Utc::now(),
                "correlationId": correlation_id
            }))
            .ok_or("session not found".into()),
        "logs_read" => {
            if let Some(source) = arguments.get("source").and_then(Value::as_str) {
                if !["all", "runtime"].contains(&source) {
                    return Err("requested log source is not available".into());
                }
            }
            let tail = bounded_integer(arguments, "tail", 50, 1, 200)? as usize;
            let lines = session_logs(&store, session, tail)
                .await
                .map_err(|error| error.to_string())?;
            Ok(json!({
                "entries": lines,
                "truncated": false,
                "correlationId": correlation_id
            }))
        }
        "session_stop" => {
            let record = stop_session(&store, session)
                .await
                .map_err(|error| error.to_string())?;
            Ok(safe_session(&record, correlation_id))
        }
        _ => unreachable!(),
    }
}

async fn webhook_tool(
    name: &str,
    arguments: &Map<String, Value>,
    correlation_id: &str,
) -> Result<Value, String> {
    let route_id = selector(arguments, "route")?;
    let session_store = session_store_path().map_err(|error| error.to_string())?;
    let route_store = temporary_route_service::route_store_path(&session_store);
    let route_exists = temporary_route_service::list_routes(&route_store, None)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .any(|route| route.id == route_id);
    if !route_exists {
        return Err("temporary route not found".into());
    }
    let event_store = temporary_route_service::event_store_path(&route_store, route_id)
        .map_err(|error| error.to_string())?;
    match name {
        "webhook_event_list" => {
            let limit = bounded_integer(arguments, "limit", 50, 1, 100)? as usize;
            let events = webhook_service::list_events(&event_store, route_id, limit)
                .await
                .map_err(|error| error.to_string())?;
            Ok(json!({ "events": events, "correlationId": correlation_id }))
        }
        "webhook_event_get" => {
            let event_id = selector(arguments, "event")?;
            let event = webhook_service::get_event(&event_store, route_id, event_id)
                .await
                .map_err(|error| error.to_string())?;
            Ok(json!({ "event": event, "correlationId": correlation_id }))
        }
        _ => unreachable!(),
    }
}

async fn resolve_workspace(selector: &str) -> Result<std::path::PathBuf, String> {
    workspace_registry::resolve(
        &workspace_registry_path().map_err(|error| error.to_string())?,
        selector,
    )
    .await
    .map_err(|error| error.to_string())
}

fn safe_session(record: &SessionRecord, correlation_id: &str) -> Value {
    json!({
        "id": record.id,
        "workspaceId": record.workspace_id,
        "profileId": record.profile_id,
        "state": record.state,
        "runtimeOwnership": if record.runtime_owned { "session" } else { "external" },
        "tunnelOwnership": if record.tunnel_started_by_session { "session" } else { "external_or_disabled" },
        "publicUrls": record.public_urls,
        "startedAt": record.started_at,
        "cleanupRequired": format!("{:?}", record.state).eq_ignore_ascii_case("cleanupincomplete"),
        "correlationId": correlation_id
    })
}

fn safe_workspace_status(
    manifest: WorkspaceManifest,
    trusted: bool,
    session: Option<&SessionRecord>,
    correlation_id: &str,
) -> Value {
    let readiness = match manifest.ready {
        Readiness::Tcp { host, port, .. } => {
            json!({ "type": "tcp", "target": format!("{host}:{port}") })
        }
        Readiness::Http { url, .. } => json!({ "type": "http", "target": url }),
    };
    let executable = std::path::Path::new(&manifest.runtime.executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("configured");
    json!({
        "id": manifest.project.id.unwrap_or_else(|| manifest.project.name.clone()),
        "name": manifest.project.name,
        "manifestVersion": manifest.version,
        "profile": manifest.profile.id.or(manifest.profile.name),
        "runtime": { "executable": executable, "argumentCount": manifest.runtime.args.len() },
        "readiness": readiness,
        "routes": manifest.exposure.routes.into_iter().map(|route| json!({ "hostname": route.hostname, "path": route.path, "mode": route.mode.unwrap_or_else(|| "persistent".into()) })).collect::<Vec<_>>(),
        "trusted": trusted,
        "requiredAction": if trusted { Value::Null } else { Value::String("open_flaredeck_and_review_workspace".into()) },
        "activeSession": session.map(|record| safe_session(record, correlation_id)),
        "path": "registered",
        "correlationId": correlation_id
    })
}

fn selector<'a>(arguments: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 100)
        .ok_or_else(|| format!("invalid {key} selector"))
}

fn bounded_integer(
    arguments: &Map<String, Value>,
    key: &str,
    default: u64,
    minimum: u64,
    maximum: u64,
) -> Result<u64, String> {
    match arguments.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_u64()
            .filter(|value| (minimum..=maximum).contains(value))
            .ok_or_else(|| format!("invalid {key}")),
    }
}

fn classify_error(message: String, correlation_id: &str) -> ToolError {
    let lower = message.to_ascii_lowercase();
    let (code, retryable, required_action) = if lower.contains("cancelled") {
        ("CANCELLED", true, None)
    } else if lower.contains("approval") || lower.contains("trusted") {
        (
            "WORKSPACE_NOT_TRUSTED",
            false,
            Some("open_flaredeck_and_review_workspace"),
        )
    } else if lower.contains("not found") {
        ("NOT_FOUND", false, None)
    } else if lower.contains("timeout") || lower.contains("timed out") {
        (
            "READINESS_TIMEOUT",
            true,
            Some("check_local_runtime_and_retry"),
        )
    } else if lower.contains("ambiguous") || lower.contains("already") || lower.contains("conflict")
    {
        ("CONFLICT", false, None)
    } else if lower.contains("invalid")
        || lower.contains("unknown")
        || lower.contains("missing")
        || lower.contains("not available")
    {
        ("INVALID_REQUEST", false, None)
    } else {
        ("OPERATION_FAILED", false, None)
    };
    ToolError {
        code,
        message,
        retryable,
        required_action,
        correlation_id: correlation_id.into(),
    }
}

fn negotiated_protocol_version(request: &Value) -> &str {
    match request
        .pointer("/params/protocolVersion")
        .and_then(Value::as_str)
    {
        Some(version @ ("2024-11-05" | "2025-03-26" | "2025-06-18" | "2025-11-25")) => version,
        _ => "2025-11-25",
    }
}

fn tools() -> Vec<Value> {
    let selector = || json!({ "type": "string", "minLength": 1, "maxLength": 100 });
    vec![
        json!({ "name": "workspace_list", "description": "List desktop-registered workspaces.", "inputSchema": { "type": "object", "properties": { "state": { "type": "string", "enum": ["all", "trusted", "approval_required", "running", "invalid"] }, "limit": { "type": "integer", "minimum": 1, "maximum": 100 } }, "additionalProperties": false } }),
        json!({ "name": "workspace_status", "description": "Inspect one registered workspace.", "inputSchema": { "type": "object", "required": ["workspace"], "properties": { "workspace": selector() }, "additionalProperties": false } }),
        json!({ "name": "session_start", "description": "Start an approved registered workspace session.", "inputSchema": { "type": "object", "required": ["workspace"], "properties": { "workspace": selector(), "waitForHealthy": { "type": "boolean", "default": true } }, "additionalProperties": false } }),
        json!({ "name": "session_status", "description": "Read safe session status.", "inputSchema": { "type": "object", "required": ["session"], "properties": { "session": selector() }, "additionalProperties": false } }),
        json!({ "name": "session_stop", "description": "Stop resources owned by a session.", "inputSchema": { "type": "object", "required": ["session"], "properties": { "session": selector() }, "additionalProperties": false } }),
        json!({ "name": "public_url_get", "description": "Read session public URLs.", "inputSchema": { "type": "object", "required": ["session"], "properties": { "session": selector() }, "additionalProperties": false } }),
        json!({ "name": "health_check", "description": "Read bounded session health observations.", "inputSchema": { "type": "object", "required": ["session"], "properties": { "session": selector() }, "additionalProperties": false } }),
        json!({ "name": "logs_read", "description": "Read bounded redacted logs.", "inputSchema": { "type": "object", "required": ["session"], "properties": { "session": selector(), "source": { "type": "string", "enum": ["all", "runtime", "tunnel", "system"] }, "tail": { "type": "integer", "minimum": 1, "maximum": 200 } }, "additionalProperties": false } }),
        json!({ "name": "webhook_event_list", "description": "Read bounded redacted events for an owned temporary route.", "inputSchema": { "type": "object", "required": ["route"], "properties": { "route": selector(), "limit": { "type": "integer", "minimum": 1, "maximum": 100 } }, "additionalProperties": false } }),
        json!({ "name": "webhook_event_get", "description": "Read one redacted event for an owned temporary route.", "inputSchema": { "type": "object", "required": ["route", "event"], "properties": { "route": selector(), "event": selector() }, "additionalProperties": false } }),
        json!({ "name": "doctor", "description": "Read safe local diagnostics.", "inputSchema": { "type": "object", "additionalProperties": false } }),
    ]
}

fn result_response(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn tool_surface_matches_approved_names_and_excludes_privileged_access() {
        let names = super::tools()
            .into_iter()
            .filter_map(|tool| {
                tool.get("name")
                    .and_then(|name| name.as_str())
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        for required in [
            "workspace_list",
            "workspace_status",
            "session_start",
            "session_status",
            "session_stop",
            "public_url_get",
            "logs_read",
            "health_check",
            "doctor",
            "webhook_event_list",
            "webhook_event_get",
        ] {
            assert!(names.contains(&required.into()));
        }
        assert!(!names.iter().any(|name| name.contains("approve")
            || name.contains("shell")
            || name.contains("token")
            || name.contains("replay")));
    }

    #[test]
    fn safe_session_never_serializes_internal_paths_or_fingerprint() {
        let record = flaredeck_lib::application::session_service::SessionRecord {
            id: "ses_1".into(),
            workspace_root: "/secret/workspace".into(),
            workspace_id: "app".into(),
            profile_id: "profile".into(),
            fingerprint: "sha256:secret".into(),
            state: flaredeck_lib::application::session_service::SessionState::Healthy,
            runtime_pid: Some(1),
            runtime_started_at_seconds: Some(2),
            runtime_executable: "/secret/bin".into(),
            runtime_log_path: Some("/secret/log".into()),
            runtime_owned: true,
            stop_runtime_on_session_stop: true,
            tunnel_started_by_session: false,
            tunnel_pid: None,
            tunnel_started_at_seconds: None,
            tunnel_executable: None,
            stop_tunnel_on_session_stop: true,
            remove_temporary_routes: true,
            public_urls: vec!["https://app.example.com".into()],
            started_at: chrono::Utc::now(),
        };
        let value = super::safe_session(&record, "corr_test").to_string();
        for forbidden in ["/secret", "sha256", "runtimePid", "logPath"] {
            assert!(!value.contains(forbidden));
        }
        assert_eq!(
            json!("app"),
            super::safe_session(&record, "corr_test")["workspaceId"]
        );
    }
}
