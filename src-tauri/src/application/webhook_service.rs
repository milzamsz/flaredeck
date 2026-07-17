use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{AppError, AppResult};

const MAX_HEADERS: usize = 16 * 1024;
const MAX_BODY: usize = 64 * 1024;
const MAX_RESPONSE_BODY: u64 = 64 * 1024;
const MAX_EVENTS_PER_ROUTE: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEvent {
    pub id: String,
    pub route_id: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub content_type: Option<String>,
    pub body: Option<String>,
    pub body_state: String,
    pub response_status: Option<u16>,
    pub redaction_version: u8,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventStore {
    #[serde(default = "schema_version")]
    schema_version: u8,
    #[serde(default)]
    events: Vec<WebhookEvent>,
}

fn schema_version() -> u8 {
    1
}

pub async fn serve(
    listen: &str,
    route_id: String,
    origin: String,
    store_path: std::path::PathBuf,
) -> AppResult<()> {
    if !is_loopback_http(&origin) || !is_loopback_socket(listen) {
        return Err(AppError::Other(
            "webhook proxy targets must be loopback".into(),
        ));
    }
    let listener = tokio::net::TcpListener::bind(listen).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        if let Err(error) = handle_connection(stream, &route_id, &origin, &store_path).await {
            eprintln!("webhook proxy request rejected: {error}");
        }
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    route_id: &str,
    origin: &str,
    store_path: &Path,
) -> AppResult<()> {
    let mut raw = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 4096];
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            return Err(AppError::Other("incomplete webhook request".into()));
        }
        raw.extend_from_slice(&chunk[..read]);
        if let Some(end) = find_header_end(&raw) {
            break end;
        }
        if raw.len() > MAX_HEADERS {
            write_status(&mut stream, 431).await?;
            return Err(AppError::Other("webhook headers exceed limit".into()));
        }
    };
    if header_end > MAX_HEADERS {
        write_status(&mut stream, 431).await?;
        return Err(AppError::Other("webhook headers exceed limit".into()));
    }
    let header_text = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| AppError::Other("webhook headers must be UTF-8".into()))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default().to_owned();
    let path = request_parts.next().unwrap_or_default().to_owned();
    let version = request_parts.next().unwrap_or_default().to_owned();
    if !version.starts_with("HTTP/1.")
        || !path.starts_with('/')
        || path.starts_with("//")
        || path.len() > 2048
        || matches!(method.as_str(), "CONNECT" | "TRACE")
    {
        write_status(&mut stream, 400).await?;
        return Err(AppError::Other("invalid webhook request line".into()));
    }
    let mut headers = BTreeMap::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| AppError::Other("invalid webhook header".into()))?;
        let name = name.trim().to_ascii_lowercase();
        if headers.insert(name, value.trim().to_owned()).is_some() {
            write_status(&mut stream, 400).await?;
            return Err(AppError::Other(
                "duplicate webhook headers are not supported".into(),
            ));
        }
    }
    if headers
        .get("transfer-encoding")
        .is_some_and(|value| !value.eq_ignore_ascii_case("identity"))
    {
        write_status(&mut stream, 400).await?;
        return Err(AppError::Other(
            "chunked webhook bodies are not supported".into(),
        ));
    }
    let content_length = headers
        .get("content-length")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| AppError::Other("invalid webhook content length".into()))?
        .unwrap_or(0);
    if content_length > MAX_BODY {
        write_status(&mut stream, 413).await?;
        return Err(AppError::Other("webhook body exceeds limit".into()));
    }
    let body_start = header_end + 4;
    while raw.len().saturating_sub(body_start) < content_length {
        let mut chunk = [0u8; 4096];
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            write_status(&mut stream, 400).await?;
            return Err(AppError::Other("incomplete webhook body".into()));
        }
        raw.extend_from_slice(&chunk[..read]);
    }
    let body = &raw[body_start..body_start + content_length];
    let target = format!("{}{}", origin.trim_end_matches('/'), path);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| AppError::Http(error.to_string()))?;
    let request_method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|_| AppError::Other("unsupported webhook method".into()))?;
    let mut request = client.request(request_method, target).body(body.to_vec());
    for (name, value) in &headers {
        if !matches!(
            name.as_str(),
            "host" | "connection" | "content-length" | "transfer-encoding"
        ) {
            request = request.header(name, value);
        }
    }
    let response = request.send().await;
    let (status, response_body) = match response {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = if response
                .content_length()
                .is_some_and(|length| length <= MAX_RESPONSE_BODY)
            {
                response
                    .bytes()
                    .await
                    .map(|bytes| bytes.to_vec())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            (status, body)
        }
        Err(_) => (502, Vec::new()),
    };
    record_event(
        store_path,
        route_id,
        &method,
        &path,
        &headers,
        body,
        Some(status),
    )
    .await?;
    write_response(&mut stream, status, &response_body).await?;
    Ok(())
}

pub async fn record_event(
    store_path: &Path,
    route_id: &str,
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    body: &[u8],
    response_status: Option<u16>,
) -> AppResult<WebhookEvent> {
    let content_type = headers.get("content-type").cloned();
    let (body, body_state) = redact_body(content_type.as_deref(), body);
    let event = WebhookEvent {
        id: format!("wh_{}", uuid::Uuid::new_v4()),
        route_id: route_id.into(),
        timestamp: Utc::now(),
        method: method.into(),
        path: redact_query(path),
        headers: headers
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    if sensitive_name(name) {
                        "[REDACTED]".into()
                    } else {
                        value.clone()
                    },
                )
            })
            .collect(),
        content_type,
        body,
        body_state,
        response_status,
        redaction_version: 1,
    };
    let mut store = load_store(store_path).await?;
    store.events.push(event.clone());
    while store
        .events
        .iter()
        .filter(|entry| entry.route_id == route_id)
        .count()
        > MAX_EVENTS_PER_ROUTE
    {
        if let Some(index) = store
            .events
            .iter()
            .position(|entry| entry.route_id == route_id)
        {
            store.events.remove(index);
        }
    }
    save_store(store_path, &store).await?;
    Ok(event)
}

pub async fn list_events(
    store_path: &Path,
    route_id: &str,
    limit: usize,
) -> AppResult<Vec<WebhookEvent>> {
    Ok(load_store(store_path)
        .await?
        .events
        .into_iter()
        .rev()
        .filter(|event| event.route_id == route_id)
        .take(limit.min(MAX_EVENTS_PER_ROUTE))
        .collect())
}

pub async fn get_event(
    store_path: &Path,
    route_id: &str,
    event_id: &str,
) -> AppResult<WebhookEvent> {
    load_store(store_path)
        .await?
        .events
        .into_iter()
        .find(|event| event.route_id == route_id && event.id == event_id)
        .ok_or_else(|| AppError::Other("webhook event not found".into()))
}

pub async fn clear_route(store_path: &Path, route_id: &str) -> AppResult<()> {
    let mut store = load_store(store_path).await?;
    store.events.retain(|event| event.route_id != route_id);
    if store.events.is_empty() {
        match tokio::fs::remove_file(store_path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    } else {
        save_store(store_path, &store).await
    }
}

pub async fn replay_event(
    store_path: &Path,
    route_id: &str,
    event_id: &str,
    origin: &str,
) -> AppResult<u16> {
    if !is_loopback_http(origin) {
        return Err(AppError::Other(
            "webhook replay target must be loopback".into(),
        ));
    }
    let event = get_event(store_path, route_id, event_id).await?;
    let method = reqwest::Method::from_bytes(event.method.as_bytes())
        .map_err(|_| AppError::Other("stored webhook method is invalid".into()))?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| AppError::Http(error.to_string()))?;
    let mut request = client
        .request(
            method,
            format!("{}{}", origin.trim_end_matches('/'), event.path),
        )
        .body(event.body.unwrap_or_default());
    for (name, value) in event.headers {
        if value != "[REDACTED]"
            && !matches!(name.as_str(), "host" | "connection" | "content-length")
        {
            request = request.header(name, value);
        }
    }
    Ok(request
        .send()
        .await
        .map_err(|error| AppError::Http(error.to_string()))?
        .status()
        .as_u16())
}

fn redact_body(content_type: Option<&str>, body: &[u8]) -> (Option<String>, String) {
    if body.is_empty() {
        return (None, "empty".into());
    }
    let Some(content_type) =
        content_type.map(|value| value.split(';').next().unwrap_or(value).trim())
    else {
        return (None, "unsupported_content_type".into());
    };
    let Ok(text) = std::str::from_utf8(body) else {
        return (None, "invalid_utf8".into());
    };
    if content_type == "application/json" || content_type.ends_with("+json") {
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(text) else {
            return (None, "invalid_json".into());
        };
        redact_json(&mut value);
        return (Some(value.to_string()), "stored_redacted".into());
    }
    if content_type.starts_with("text/") || content_type == "application/x-www-form-urlencoded" {
        if [
            "TOKEN",
            "SECRET",
            "PASSWORD",
            "AUTHORIZATION",
            "COOKIE",
            "API_KEY",
        ]
        .iter()
        .any(|name| text.to_ascii_uppercase().contains(name))
        {
            return (Some("[REDACTED]".into()), "stored_redacted".into());
        }
        return (Some(text.into()), "stored".into());
    }
    (None, "unsupported_content_type".into())
}

fn redact_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(values) => {
            for (name, value) in values {
                if sensitive_name(name) {
                    *value = serde_json::Value::String("[REDACTED]".into());
                } else {
                    redact_json(value);
                }
            }
        }
        serde_json::Value::Array(values) => values.iter_mut().for_each(redact_json),
        _ => {}
    }
}

fn redact_query(path: &str) -> String {
    let Some((base, query)) = path.split_once('?') else {
        return path.into();
    };
    let query = query
        .split('&')
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            if sensitive_name(&percent_decode_name(name)) {
                format!("{name}=[REDACTED]")
            } else {
                format!("{name}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&");
    format!("{base}?{query}")
}

fn percent_decode_name(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                decoded.push(high * 16 + low);
                index += 3;
                continue;
            }
        }
        decoded.push(if bytes[index] == b'+' {
            b' '
        } else {
            bytes[index]
        });
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn sensitive_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [
        "authorization",
        "cookie",
        "token",
        "secret",
        "password",
        "private_key",
        "api_key",
        "x-api-key",
    ]
    .iter()
    .any(|needle| name.contains(needle))
}

fn is_loopback_http(value: &str) -> bool {
    value.starts_with("http://127.0.0.1:")
        || value.starts_with("http://localhost:")
        || value.starts_with("http://[::1]:")
}

fn is_loopback_socket(value: &str) -> bool {
    value.starts_with("127.0.0.1:") || value.starts_with("[::1]:")
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

async fn write_status(stream: &mut tokio::net::TcpStream, status: u16) -> AppResult<()> {
    write_response(stream, status, &[]).await
}

async fn write_response(
    stream: &mut tokio::net::TcpStream,
    status: u16,
    body: &[u8],
) -> AppResult<()> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        502 => "Bad Gateway",
        _ => "Response",
    };
    stream
        .write_all(
            format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .as_bytes(),
        )
        .await?;
    stream.write_all(body).await?;
    Ok(())
}

async fn load_store(path: &Path) -> AppResult<EventStore> {
    match tokio::fs::read_to_string(path).await {
        Ok(raw) => Ok(serde_json::from_str(&raw)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(EventStore::default()),
        Err(error) => Err(error.into()),
    }
}

async fn save_store(path: &Path, store: &EventStore) -> AppResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Other("webhook event path has no parent".into()))?;
    tokio::fs::create_dir_all(parent).await?;
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, serde_json::to_vec(store)?).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn redacts_headers_nested_json_and_query_before_storage() {
        let path =
            std::env::temp_dir().join(format!("flaredeck-webhooks-{}.json", uuid::Uuid::new_v4()));
        let headers = BTreeMap::from([
            ("authorization".into(), "Bearer CANARY_SECRET".into()),
            ("content-type".into(), "application/json".into()),
        ]);
        let event = super::record_event(
            &path,
            "route",
            "POST",
            "/hook?token=CANARY_SECRET&%74oken=CANARY_SECRET&safe=yes",
            &headers,
            br#"{"customer":{"password":"CANARY_SECRET"},"safe":"ok"}"#,
            Some(204),
        )
        .await
        .unwrap();
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.contains("CANARY_SECRET"));
        assert!(serialized.contains("[REDACTED]"));
        assert!(event
            .body
            .as_deref()
            .unwrap_or_default()
            .contains("\"safe\":\"ok\""));
        let raw = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(!raw.contains("CANARY_SECRET"));
        let _ = tokio::fs::remove_file(path).await;
    }

    #[tokio::test]
    async fn event_results_are_bounded() {
        let path =
            std::env::temp_dir().join(format!("flaredeck-webhooks-{}.json", uuid::Uuid::new_v4()));
        for _ in 0..105 {
            super::record_event(
                &path,
                "route",
                "POST",
                "/",
                &BTreeMap::new(),
                b"",
                Some(204),
            )
            .await
            .unwrap();
        }
        assert_eq!(
            super::list_events(&path, "route", usize::MAX)
                .await
                .unwrap()
                .len(),
            100
        );
        let _ = tokio::fs::remove_file(path).await;
    }
}
