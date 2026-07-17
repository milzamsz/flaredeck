use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CloudflaredInfo {
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub profile_id: String,
    pub running: bool,
    pub pid: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OriginRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "noTLSVerify")]
    pub no_tls_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_host_header: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngressRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "originRequest")]
    pub origin_request: Option<OriginRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CloudflaredConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel: Option<String>,
    #[serde(rename = "credentials-file", skip_serializing_if = "Option::is_none")]
    pub credentials_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ingress: Option<Vec<IngressRule>>,
    #[serde(flatten)]
    pub extras: serde_yaml::Mapping,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPayload {
    pub path: String,
    pub raw: String,
    pub parsed: Option<CloudflaredConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub tunnel_name: String,
    pub config_path: String,
    #[serde(default)]
    pub wsl_host: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert_path: Option<String>,
    #[serde(default)]
    pub has_api_token: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePatch {
    pub name: Option<String>,
    pub wsl_host: Option<bool>,
    pub account_id: Option<String>,
    pub zone_id: Option<String>,
    pub zone_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub valid: bool,
    pub status: Option<String>,
    pub id: Option<String>,
    pub expires_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfileIndex {
    pub profiles: Vec<Profile>,
    pub active_profile_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupResult {
    pub resolved: bool,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRouteView {
    pub hostname: String,
    pub origin: String,
    pub path: Option<String>,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEnvironmentLiteralView {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTrustView {
    pub root: String,
    pub workspace_id: String,
    pub project_name: String,
    pub profile: String,
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: String,
    pub readiness: String,
    pub routes: Vec<WorkspaceRouteView>,
    pub environment_names: Vec<String>,
    pub environment_values: Vec<WorkspaceEnvironmentLiteralView>,
    pub lifecycle: Vec<String>,
    pub capabilities: Vec<String>,
    pub fingerprint: String,
    pub approval_state: String,
    pub trusted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSessionView {
    pub id: String,
    pub workspace_id: String,
    pub profile_id: String,
    pub state: String,
    pub runtime_ownership: String,
    pub tunnel_ownership: String,
    pub public_urls: Vec<String>,
    pub started_at: String,
    pub cleanup_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListItemView {
    pub root: String,
    pub workspace_id: String,
    pub project_name: String,
    pub profile: String,
    pub validation_state: String,
    pub approval_state: String,
    pub trusted: bool,
    pub active_session: Option<WorkspaceSessionView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAuditEventView {
    pub timestamp: String,
    pub operation: String,
    pub result: String,
    pub session_id: String,
    pub correlation_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryRouteView {
    pub id: String,
    pub session_id: String,
    pub hostname: String,
    pub path: Option<String>,
    pub origin: String,
    pub state: String,
    pub created_at: String,
    pub expires_at: String,
    pub cleanup_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEventView {
    pub id: String,
    pub route_id: String,
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub headers: std::collections::BTreeMap<String, String>,
    pub content_type: Option<String>,
    pub body: Option<String>,
    pub body_state: String,
    pub response_status: Option<u16>,
    pub redaction_version: u8,
}
