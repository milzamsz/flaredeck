use std::path::Path;

use chrono::Utc;
use flaredeck_lib::application::{
    context::{ActorKind, OperationContext},
    profile_service,
    session_service::{session_logs, session_status, start_session, stop_session, SessionRecord},
    state_paths::{
        default_state_dir, session_store_path, trust_store_path, workspace_registry_path,
    },
    temporary_route_service,
    trust_service::{fingerprint, is_approved},
    tunnel_service, workspace_registry,
    workspace_service::discover,
};
use flaredeck_lib::cloudflared;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Envelope<T: Serialize> {
    ok: bool,
    data: Option<T>,
    warnings: Vec<String>,
    error: Option<CliError>,
    meta: Meta,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CliError {
    code: &'static str,
    message: String,
    retryable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    schema_version: &'static str,
    correlation_id: String,
    timestamp: chrono::DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    let (json_output, command) = parse_args(std::env::args().skip(1).collect());
    let context = OperationContext::new(ActorKind::CliUser);
    let result = run(&command, &context).await;
    match result {
        Ok((data, human)) => emit_success(json_output, data, human, &context),
        Err(message) => emit_failure(json_output, message, &context),
    }
}

async fn run(command: &[String], context: &OperationContext) -> Result<(Value, String), String> {
    let command = command.iter().map(String::as_str).collect::<Vec<_>>();
    match command.as_slice() {
        ["version"] => Ok((json!(env!("CARGO_PKG_VERSION")), format!("FlareDeck {}", env!("CARGO_PKG_VERSION")))),
        ["profile", "list"] => {
            let index = profile_service::list().await.map_err(|error| error.to_string())?;
            let human = format!("{} profiles", index.profiles.len());
            Ok((serde_json::to_value(index).map_err(|error| error.to_string())?, human))
        }
        ["tunnel", "status", profile] => {
            let installed = cloudflared::resolve_cloudflared_path().is_some();
            let observed = if installed {
                tunnel_service::observe(profile)
                    .await
                    .map_err(|error| error.to_string())?
            } else {
                None
            };
            Ok((json!({
                "profileId": profile,
                "state": if observed.is_some() { "running" } else if installed { "stopped" } else { "unavailable" },
                "ownedByCli": false,
                "correlationId": context.correlation_id
            }), if observed.is_some() { "tunnel running".into() } else { "tunnel stopped".into() }))
        }
        ["doctor"] => doctor(context).await,
        ["workspace", "list"] => workspace_list().await,
        ["workspace", "discover", path]
        | ["workspace", "inspect", path]
        | ["workspace", "validate", path]
        | ["workspace", "trust-status", path] => workspace_inspect(path).await,
        ["session", "start", workspace] => {
            let record = start_session(
                Path::new(workspace),
                &trust_store_path().map_err(|error| error.to_string())?,
                &session_store_path().map_err(|error| error.to_string())?,
            )
            .await
            .map_err(|error| error.to_string())?;
            Ok((safe_session(&record, context), "session started".into()))
        }
        ["session", "status", selector] => {
            let record = required_session(selector).await?;
            Ok((safe_session(&record, context), "session found".into()))
        }
        ["session", "stop", selector] => {
            let record = stop_session(
                &session_store_path().map_err(|error| error.to_string())?,
                selector,
            )
            .await
            .map_err(|error| error.to_string())?;
            Ok((safe_session(&record, context), "session stopped".into()))
        }
        ["session", "logs", selector] => {
            let lines = session_logs(
                &session_store_path().map_err(|error| error.to_string())?,
                selector,
                100,
            )
            .await
            .map_err(|error| error.to_string())?;
            Ok((json!({ "entries": lines, "truncated": false }), "runtime logs".into()))
        }
        ["route", "list", selector] => {
            let record = required_session(selector).await?;
            let temporary = temporary_route_service::list_routes(
                &temporary_route_service::route_store_path(
                    &session_store_path().map_err(|error| error.to_string())?,
                ),
                Some(&record.id),
            )
            .await
            .map_err(|error| error.to_string())?;
            let routes = record
                .public_urls
                .into_iter()
                .map(|url| {
                    let hostname = url
                        .strip_prefix("https://")
                        .unwrap_or(&url)
                        .split('/')
                        .next()
                        .unwrap_or_default();
                    if let Some(route) = temporary
                        .iter()
                        .find(|route| route.hostname.eq_ignore_ascii_case(hostname))
                    {
                        json!({
                            "url": url,
                            "ownership": "temporary",
                            "state": route.state,
                            "expiresAt": route.expires_at
                        })
                    } else {
                        json!({ "url": url, "ownership": "persistent" })
                    }
                })
                .collect::<Vec<_>>();
            Ok((json!({ "routes": routes }), "session routes".into()))
        }
        ["health", "check", selector] => {
            let record = required_session(selector).await?;
            Ok((json!({ "sessionId": record.id, "state": record.state, "checkedAt": Utc::now() }), "health checked".into()))
        }
        _ => Err("usage: flaredeck-cli [--output json] version | profile list | tunnel status <profile> | doctor | workspace list | workspace discover|inspect|validate|trust-status <path> | session start|status|stop|logs <selector> | route list <selector> | health check <selector>".into()),
    }
}

async fn doctor(context: &OperationContext) -> Result<(Value, String), String> {
    let cloudflared = cloudflared::resolve_cloudflared_path();
    let state = default_state_dir().map_err(|error| error.to_string())?;
    let profiles_readable = profile_service::list().await.is_ok();
    let reconciled =
        temporary_route_service::reconcile_expired(&temporary_route_service::route_store_path(
            &session_store_path().map_err(|error| error.to_string())?,
        ))
        .await
        .map_err(|error| error.to_string())?;
    Ok((
        json!({
            "cloudflaredInstalled": cloudflared.is_some(),
            "profileIndexReadable": profiles_readable,
            "stateDirectory": if state.exists() { "available" } else { "not_created" },
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "version": env!("CARGO_PKG_VERSION"),
            "schemaVersion": "1",
            "temporaryRoutesReconciled": reconciled.len(),
            "correlationId": context.correlation_id
        }),
        format!(
            "cloudflared: {}",
            if cloudflared.is_some() {
                "found"
            } else {
                "missing"
            }
        ),
    ))
}

async fn workspace_list() -> Result<(Value, String), String> {
    let paths =
        workspace_registry::list(&workspace_registry_path().map_err(|error| error.to_string())?)
            .await
            .map_err(|error| error.to_string())?;
    let mut workspaces = Vec::new();
    for path in paths {
        if let Ok((_, manifest)) = discover(Path::new(&path)).await {
            workspaces.push(json!({
                "id": manifest.project.id.unwrap_or_else(|| manifest.project.name.clone()),
                "name": manifest.project.name,
                "profile": manifest.profile.id.or(manifest.profile.name),
                "path": path
            }));
        }
    }
    let human = format!("{} workspaces", workspaces.len());
    Ok((json!({ "workspaces": workspaces }), human))
}

async fn workspace_inspect(path: &str) -> Result<(Value, String), String> {
    let (root, manifest) = discover(Path::new(path))
        .await
        .map_err(|error| error.to_string())?;
    let raw = tokio::fs::read_to_string(root.join(".flaredeck/project.yaml"))
        .await
        .map_err(|error| error.to_string())?;
    let digest = fingerprint(&raw).map_err(|error| error.to_string())?;
    let trusted = match trust_store_path() {
        Ok(path) => is_approved(&path, &root, &digest).await,
        Err(_) => false,
    };
    Ok((
        json!({
            "workspaceId": manifest.project.id.unwrap_or(manifest.project.name),
            "profile": manifest.profile.id.or(manifest.profile.name),
            "fingerprint": digest,
            "trusted": trusted,
            "valid": true
        }),
        if trusted {
            "trusted".into()
        } else {
            "approval required".into()
        },
    ))
}

async fn required_session(selector: &str) -> Result<SessionRecord, String> {
    session_status(
        &session_store_path().map_err(|error| error.to_string())?,
        selector,
    )
    .await
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "session not found".into())
}

fn safe_session(record: &SessionRecord, context: &OperationContext) -> Value {
    json!({
        "sessionId": record.id,
        "workspaceId": record.workspace_id,
        "profileId": record.profile_id,
        "state": record.state,
        "runtime": { "ownedBySession": record.runtime_owned },
        "tunnel": { "startedBySession": record.tunnel_started_by_session },
        "publicUrls": record.public_urls,
        "startedAt": record.started_at,
        "correlationId": context.correlation_id
    })
}

fn parse_args(args: Vec<String>) -> (bool, Vec<String>) {
    let mut json_output = false;
    let mut command = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--output=json" {
            json_output = true;
        } else if arg == "--output" {
            if args.next().as_deref() == Some("json") {
                json_output = true;
            } else {
                command.push("--invalid-output".into());
            }
        } else {
            command.push(arg);
        }
    }
    (json_output, command)
}

fn emit_success(json_output: bool, data: Value, human: String, context: &OperationContext) {
    if json_output {
        emit_json(&Envelope {
            ok: true,
            data: Some(data),
            warnings: Vec::new(),
            error: None,
            meta: meta(context),
        });
    } else {
        println!("{human}");
    }
}

fn emit_failure(json_output: bool, message: String, context: &OperationContext) -> ! {
    let (code, exit_code, retryable) = classify_error(&message);
    if json_output {
        emit_json(&Envelope::<Value> {
            ok: false,
            data: None,
            warnings: Vec::new(),
            error: Some(CliError {
                code,
                message,
                retryable,
            }),
            meta: meta(context),
        });
    } else {
        eprintln!("{code}: {message}");
    }
    std::process::exit(exit_code)
}

fn emit_json<T: Serialize>(value: &Envelope<T>) {
    match serde_json::to_string(value) {
        Ok(value) => println!("{value}"),
        Err(_) => {
            eprintln!("INTERNAL_ERROR: failed to serialize response");
            std::process::exit(50);
        }
    }
}

fn meta(context: &OperationContext) -> Meta {
    Meta {
        schema_version: "1",
        correlation_id: context.correlation_id.clone(),
        timestamp: Utc::now(),
    }
}

fn classify_error(message: &str) -> (&'static str, i32, bool) {
    let message = message.to_ascii_lowercase();
    if message.starts_with("usage:") || message.contains("invalid output") {
        ("USAGE_ERROR", 2, false)
    } else if message.contains("approval") || message.contains("trusted") {
        ("WORKSPACE_NOT_TRUSTED", 11, false)
    } else if message.contains("already") || message.contains("conflict") {
        ("STATE_CONFLICT", 12, false)
    } else if message.contains("readiness") || message.contains("timed out") {
        ("READINESS_FAILED", 21, true)
    } else if message.contains("tunnel") || message.contains("route") || message.contains("dns") {
        ("TUNNEL_OPERATION_FAILED", 30, true)
    } else if message.contains("not found")
        || message.contains("manifest")
        || message.contains("workspace")
        || message.contains("invalid")
    {
        ("VALIDATION_FAILED", 10, false)
    } else if message.contains("persist")
        || message.contains("audit")
        || message.contains("json")
        || message.contains("yaml")
    {
        ("PERSISTENCE_FAILED", 40, false)
    } else if message.contains("process") || message.contains("runtime") {
        ("RUNTIME_FAILED", 20, true)
    } else {
        ("INTERNAL_ERROR", 50, false)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_output_mode_without_consuming_command_values() {
        let (json, command) =
            super::parse_args(vec!["--output".into(), "json".into(), "doctor".into()]);
        assert!(json);
        assert_eq!(command, ["doctor"]);
    }

    #[test]
    fn stable_exit_categories_cover_usage_trust_and_readiness() {
        assert_eq!(super::classify_error("usage: flaredeck").1, 2);
        assert_eq!(super::classify_error("workspace requires approval").1, 11);
        assert_eq!(super::classify_error("readiness timed out").1, 21);
    }
}
