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
pub struct AuthStatus {
    pub authenticated: bool,
    pub cert_path: Option<String>,
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
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePatch {
    pub name: Option<String>,
    pub wsl_host: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatedTunnel {
    pub uuid: String,
    pub name: String,
    pub credentials_file: String,
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
pub struct TunnelListEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
