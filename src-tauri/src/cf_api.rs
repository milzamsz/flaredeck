use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::secrets;
use crate::types::{Profile, TokenInfo};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Identifies which Cloudflare API call we're making so error messages
/// can name the operation and recommend the right scope to add.
#[derive(Debug, Clone, Copy)]
enum ApiCall {
    VerifyToken,
    ZoneLookup,
    CreateTunnel,
    DnsRoute,
    /// Cheap read used to probe whether a token has Cloudflare Tunnel
    /// scope on a given account before we mutate any state.
    PreflightTunnelScope,
}

impl ApiCall {
    fn label(&self) -> &'static str {
        match self {
            ApiCall::VerifyToken => "verifying token",
            ApiCall::ZoneLookup => "looking up zone",
            ApiCall::CreateTunnel => "creating Cloudflare Tunnel",
            ApiCall::DnsRoute => "creating DNS record",
            ApiCall::PreflightTunnelScope => "checking Cloudflare Tunnel scope",
        }
    }
}

pub struct CfClient {
    http: reqwest::Client,
    token: String,
    account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CfEnvelope<T> {
    success: bool,
    #[serde(default)]
    errors: Vec<CfError>,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct CfError {
    #[serde(default)]
    code: i64,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct CfTokenVerify {
    id: String,
    status: String,
    #[serde(default)]
    expires_on: Option<String>,
}

#[derive(Debug, Serialize)]
struct DnsRecordBody<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    name: &'a str,
    content: &'a str,
    proxied: bool,
    comment: &'a str,
}

#[derive(Debug, Deserialize)]
struct CfDnsRecord {
    id: String,
}

#[derive(Debug, Deserialize)]
struct CfZone {
    id: String,
    name: String,
    #[serde(default)]
    account: Option<CfZoneAccount>,
}

#[derive(Debug, Deserialize)]
struct CfZoneAccount {
    #[serde(default)]
    id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZoneLookup {
    pub zone_id: String,
    pub zone_name: String,
    pub account_id: String,
    pub account_name: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatedTunnelApi {
    pub id: String,
    pub name: String,
    /// Base64-encoded random 32-byte secret used to author the
    /// credentials JSON file that cloudflared loads at run time.
    pub secret_b64: String,
}

#[derive(Debug, Deserialize)]
struct CfCreatedTunnel {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct CreateTunnelBody<'a> {
    name: &'a str,
    tunnel_secret: &'a str,
    config_src: &'a str,
}

fn http_client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!("flaredeck/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::Http(e.to_string()))
}

impl CfClient {
    pub fn from_token(token: String, account_id: Option<String>) -> AppResult<Self> {
        Ok(Self {
            http: http_client()?,
            token,
            account_id,
        })
    }

    pub fn for_profile(profile: &Profile) -> AppResult<Self> {
        let token = secrets::load_token(&profile.id)?
            .ok_or_else(|| AppError::NoApiToken(profile.id.clone()))?;
        Self::from_token(token, profile.account_id.clone())
    }

    fn account_id(&self) -> AppResult<&str> {
        self.account_id
            .as_deref()
            .ok_or(AppError::MissingProfileField("accountId"))
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        ctx: ApiCall,
    ) -> AppResult<T> {
        let resp = self
            .http
            .get(format!("{API_BASE}{path}"))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| AppError::Http(e.to_string()))?;
        parse_envelope(resp, ctx).await
    }

    async fn post_json<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
        ctx: ApiCall,
    ) -> AppResult<T> {
        let resp = self
            .http
            .post(format!("{API_BASE}{path}"))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::Http(e.to_string()))?;
        parse_envelope(resp, ctx).await
    }

    pub async fn verify_token(&self) -> AppResult<TokenInfo> {
        let v: CfTokenVerify = self
            .get_json("/user/tokens/verify", ApiCall::VerifyToken)
            .await?;
        Ok(TokenInfo {
            valid: v.status.eq_ignore_ascii_case("active"),
            status: Some(v.status),
            id: Some(v.id),
            expires_on: v.expires_on,
        })
    }

    /// Cheapest readable call on the `cfd_tunnel` namespace. Succeeds
    /// iff the token has `Account → Cloudflare Tunnel: Read` (or Edit,
    /// which implies Read). Used as a pre-flight gate before the
    /// wizard generates a UUID and writes files.
    pub async fn preflight_cfd_tunnel_scope(&self) -> AppResult<()> {
        let account = self.account_id()?;
        let _: serde_json::Value = self
            .get_json(
                &format!("/accounts/{account}/cfd_tunnel?per_page=1"),
                ApiCall::PreflightTunnelScope,
            )
            .await?;
        Ok(())
    }

    /// Resolve a domain to its Cloudflare zone + owning account.
    /// Walks subdomain → apex (e.g. `app.example.com` → `example.com`)
    /// until Cloudflare returns a match, so users can paste any hostname
    /// they have in mind, not just the apex.
    pub async fn lookup_zone_by_domain(&self, domain: &str) -> AppResult<ZoneLookup> {
        let cleaned = normalise_domain(domain)
            .ok_or_else(|| AppError::Cloudflare(format!("not a valid domain: {domain:?}")))?;

        for candidate in apex_candidates(&cleaned) {
            let resp = self
                .http
                .get(format!("{API_BASE}/zones"))
                .query(&[("name", candidate.as_str()), ("per_page", "1")])
                .bearer_auth(&self.token)
                .send()
                .await
                .map_err(|e| AppError::Http(e.to_string()))?;
            let zones: Vec<CfZone> = parse_envelope(resp, ApiCall::ZoneLookup).await?;
            if let Some(z) = zones.into_iter().next() {
                let account_id = z
                    .account
                    .and_then(|a| a.id)
                    .ok_or_else(|| {
                        AppError::Cloudflare(
                            "zone response did not include an account id".into(),
                        )
                    })?;
                return Ok(ZoneLookup {
                    zone_id: z.id,
                    zone_name: z.name,
                    account_id,
                    account_name: None,
                });
            }
        }

        Err(AppError::Cloudflare(format!(
            "no Cloudflare zone matches {cleaned:?} — check the domain is in your account and the token has Zone:Read scope"
        )))
    }

    /// Create a Cloudflare Tunnel via the API, generating a fresh
    /// 32-byte secret on the client. The same secret is sent in the
    /// POST (base64-encoded as `tunnel_secret`) and returned to the
    /// caller so it can be written into the local credentials JSON.
    pub async fn create_tunnel(&self, name: &str) -> AppResult<CreatedTunnelApi> {
        let account = self.account_id()?;
        let mut secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret);
        let secret_b64 = B64.encode(secret);

        let body = CreateTunnelBody {
            name,
            tunnel_secret: &secret_b64,
            config_src: "local",
        };
        let resp = self
            .http
            .post(format!("{API_BASE}/accounts/{account}/cfd_tunnel"))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Http(e.to_string()))?;
        let created: CfCreatedTunnel = parse_envelope(resp, ApiCall::CreateTunnel).await?;
        Ok(CreatedTunnelApi {
            id: created.id,
            name: created.name,
            secret_b64,
        })
    }

    pub async fn upsert_dns_route(
        &self,
        zone_id: &str,
        hostname: &str,
        tunnel_id: &str,
    ) -> AppResult<String> {
        let target = format!("{tunnel_id}.cfargotunnel.com");
        let body = DnsRecordBody {
            kind: "CNAME",
            name: hostname,
            content: &target,
            proxied: true,
            comment: "Managed by FlareDeck",
        };
        let rec: CfDnsRecord = self
            .post_json(
                &format!("/zones/{zone_id}/dns_records"),
                &body,
                ApiCall::DnsRoute,
            )
            .await?;
        Ok(rec.id)
    }
}

async fn parse_envelope<T: for<'de> Deserialize<'de>>(
    resp: reqwest::Response,
    ctx: ApiCall,
) -> AppResult<T> {
    let status = resp.status();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;
    let env: CfEnvelope<T> = serde_json::from_slice(&bytes).map_err(|e| {
        AppError::Cloudflare(format!(
            "unparseable response while {} ({status}): {e}: {}",
            ctx.label(),
            String::from_utf8_lossy(&bytes)
        ))
    })?;
    if !env.success {
        let body = if env.errors.is_empty() {
            format!("request failed with status {status}")
        } else {
            env.errors
                .iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let hint = hint_for(ctx, &env.errors);
        let msg = if hint.is_empty() {
            format!("while {}: {body}", ctx.label())
        } else {
            format!("while {}: {body} — {hint}", ctx.label())
        };
        return Err(AppError::Cloudflare(msg));
    }
    env.result
        .ok_or_else(|| AppError::Cloudflare("empty result".into()))
}

/// Site-specific hints. Cloudflare reuses `10000: Authentication error`
/// to mean both "token rejected" and "token can't touch this resource
/// type", which is unactionable on its own. The combination of code +
/// API call lets us name the specific scope to add.
fn hint_for(ctx: ApiCall, errors: &[CfError]) -> String {
    let has_auth_failure = errors
        .iter()
        .any(|e| matches!(e.code, 10000 | 9109 | 9106 | 9000));
    if !has_auth_failure {
        return String::new();
    }
    match ctx {
        ApiCall::VerifyToken => "Token was rejected. It may have been deleted, expired, \
or copied incomplete. Re-copy or create a new token.".into(),

        ApiCall::ZoneLookup => "Token cannot read zones. \
Add Zone → Zone: Read to its scopes (Zone Resources can be specific zones or \"All zones\").".into(),

        ApiCall::PreflightTunnelScope | ApiCall::CreateTunnel =>
            "Token is missing Account → Cloudflare Tunnel: Edit. \
Edit the token in the Cloudflare dashboard, add that permission, and try again.".into(),

        ApiCall::DnsRoute => "Token cannot write DNS records on this zone. \
Add Zone → DNS: Edit to its scopes.".into(),
    }
}

/// Strip scheme, path, port, leading `www.`, and trailing dot.
/// Returns `None` if nothing usable remains or the result has no dot.
fn normalise_domain(input: &str) -> Option<String> {
    let s = input.trim().to_ascii_lowercase();
    let s = s.trim_start_matches("https://").trim_start_matches("http://");
    let s = s.split('/').next().unwrap_or("");
    let s = s.split('?').next().unwrap_or("");
    let s = s.split(':').next().unwrap_or("");
    let s = s.trim_end_matches('.');
    let s = s.strip_prefix("www.").unwrap_or(s);
    if s.is_empty() || !s.contains('.') {
        return None;
    }
    // bare sanity check — Cloudflare will reject anything truly weird
    if s.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '.' || c == '-')) {
        return None;
    }
    Some(s.to_string())
}

/// Candidate apexes to try in order: the input itself, then progressively
/// shorter forms. For `a.b.example.com` yields `["a.b.example.com",
/// "b.example.com", "example.com"]`. Stops at 2-label minimum.
fn apex_candidates(domain: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: &str = domain;
    loop {
        if cur.matches('.').count() < 1 {
            break;
        }
        out.push(cur.to_string());
        match cur.find('.') {
            Some(idx) if cur[idx + 1..].contains('.') => cur = &cur[idx + 1..],
            _ => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_common_inputs() {
        assert_eq!(normalise_domain("Example.COM").as_deref(), Some("example.com"));
        assert_eq!(normalise_domain("https://www.example.com/path").as_deref(), Some("example.com"));
        assert_eq!(normalise_domain("api.example.com:8080").as_deref(), Some("api.example.com"));
        assert_eq!(normalise_domain("example.com.").as_deref(), Some("example.com"));
        assert_eq!(normalise_domain("notadomain"), None);
        assert_eq!(normalise_domain(""), None);
        assert_eq!(normalise_domain("bad space.com"), None);
    }

    #[test]
    fn walks_apex_candidates() {
        assert_eq!(apex_candidates("example.com"), vec!["example.com"]);
        assert_eq!(
            apex_candidates("api.example.com"),
            vec!["api.example.com", "example.com"]
        );
        assert_eq!(
            apex_candidates("a.b.example.com"),
            vec!["a.b.example.com", "b.example.com", "example.com"]
        );
        // 2-label minimum: doesn't try the empty TLD
        let v = apex_candidates("example.com");
        assert_eq!(v.last().unwrap(), "example.com");
    }
}
