use crate::application::workspace_service::Route;
use crate::cf_api::{CfClient, ZoneLookup};
use crate::cloudflared::ensure_cloudflared;
use crate::commands::profiles::get_profile;
use crate::error::{AppError, AppResult};
use crate::types::CloudflaredConfig;

trait RoutePort {
    async fn upsert_dns_route(
        &self,
        profile_id: String,
        hostname: String,
        tunnel_id: String,
    ) -> AppResult<String>;
    async fn lookup_zone(&self, profile_id: String, domain: String) -> AppResult<ZoneLookup>;
}

struct CloudflareRoutePort;

impl RoutePort for CloudflareRoutePort {
    async fn upsert_dns_route(
        &self,
        profile_id: String,
        hostname: String,
        tunnel_id: String,
    ) -> AppResult<String> {
        let profile = get_profile(&profile_id).await?;
        let zone_id = profile
            .zone_id
            .clone()
            .ok_or(AppError::MissingProfileField("zoneId"))?;
        CfClient::for_profile(&profile)?
            .upsert_dns_route(&zone_id, hostname.trim(), tunnel_id.trim())
            .await
    }

    async fn lookup_zone(&self, profile_id: String, domain: String) -> AppResult<ZoneLookup> {
        let profile = get_profile(&profile_id).await?;
        CfClient::for_profile(&profile)?
            .lookup_zone_by_domain(domain.trim())
            .await
    }
}

/// Verifies only persistent routes already owned by the profile configuration.
/// Temporary-route creation waits for ownership-safe deletion support.
pub async fn verify_persistent_routes(profile_id: &str, routes: &[Route]) -> AppResult<()> {
    let profile = get_profile(profile_id).await?;
    let raw = tokio::fs::read_to_string(&profile.config_path).await?;
    let config: CloudflaredConfig = serde_yaml::from_str(&raw)?;
    if persistent_routes_present(&config, routes) {
        return Ok(());
    }
    for route in routes
        .iter()
        .filter(|route| route.mode.as_deref() != Some("temporary"))
    {
        if !config.ingress.as_ref().is_some_and(|ingress| {
            ingress.iter().any(|rule| {
                rule.hostname.as_deref() == Some(route.hostname.as_str())
                    && rule.path.as_deref() == route.path.as_deref()
                    && rule.service == route.service
            })
        }) {
            return Err(AppError::Other(format!(
                "persistent route {} is not present in the selected profile configuration",
                route.hostname
            )));
        }
    }
    Ok(())
}

fn persistent_routes_present(config: &CloudflaredConfig, routes: &[Route]) -> bool {
    routes
        .iter()
        .filter(|route| route.mode.as_deref() != Some("temporary"))
        .all(|route| {
            config.ingress.as_ref().is_some_and(|ingress| {
                ingress.iter().any(|rule| {
                    rule.hostname.as_deref() == Some(route.hostname.as_str())
                        && rule.path.as_deref() == route.path.as_deref()
                        && rule.service == route.service
                })
            })
        })
}

/// Shared Cloudflare route operation; token access remains inside CfClient.
pub async fn upsert_dns_route(
    profile_id: String,
    hostname: String,
    tunnel_id: String,
) -> AppResult<String> {
    upsert_dns_route_with(&CloudflareRoutePort, profile_id, hostname, tunnel_id).await
}

/// Shared safe zone lookup; scope failures retain CfClient's actionable hint.
pub async fn lookup_zone(profile_id: String, domain: String) -> AppResult<ZoneLookup> {
    lookup_zone_with(&CloudflareRoutePort, profile_id, domain).await
}

async fn upsert_dns_route_with(
    port: &impl RoutePort,
    profile_id: String,
    hostname: String,
    tunnel_id: String,
) -> AppResult<String> {
    port.upsert_dns_route(profile_id, hostname, tunnel_id).await
}

async fn lookup_zone_with(
    port: &impl RoutePort,
    profile_id: String,
    domain: String,
) -> AppResult<ZoneLookup> {
    port.lookup_zone(profile_id, domain).await
}

/// Existing no-token fallback; its command and argument order remain fixed.
pub async fn route_dns_with_cloudflared(tunnel_name: String, hostname: String) -> AppResult<()> {
    let output = tokio::process::Command::new(ensure_cloudflared()?)
        .args(["tunnel", "route", "dns", "-f", &tunnel_name, &hostname])
        .output()
        .await?;
    if !output.status.success() {
        return Err(AppError::CloudflaredFailed(
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::application::workspace_service::Route;
    use crate::cf_api::ZoneLookup;
    use crate::error::{AppError, AppResult};
    use crate::types::{CloudflaredConfig, IngressRule};

    #[test]
    fn persistent_route_matching_is_exact() {
        let config = CloudflaredConfig {
            tunnel: None,
            credentials_file: None,
            ingress: Some(vec![IngressRule {
                hostname: Some("app.example.com".into()),
                path: None,
                service: "http://127.0.0.1:3000".into(),
                origin_request: None,
            }]),
            extras: serde_yaml::Mapping::new(),
        };
        let route = Route {
            hostname: "app.example.com".into(),
            service: "http://127.0.0.1:3000".into(),
            path: None,
            mode: None,
        };
        assert!(super::persistent_routes_present(&config, &[route]));
    }

    struct FakeRoutePort;

    impl super::RoutePort for FakeRoutePort {
        async fn upsert_dns_route(
            &self,
            profile_id: String,
            hostname: String,
            tunnel_id: String,
        ) -> AppResult<String> {
            if profile_id == "profile" && hostname == "app.example.test" && tunnel_id == "tunnel" {
                Ok("route-id".into())
            } else {
                Err(AppError::Other("unexpected route input".into()))
            }
        }

        async fn lookup_zone(&self, _profile_id: String, _domain: String) -> AppResult<ZoneLookup> {
            Err(AppError::Other("not used".into()))
        }
    }

    #[tokio::test]
    async fn route_orchestration_is_testable_without_cloudflare() {
        let id = super::upsert_dns_route_with(
            &FakeRoutePort,
            "profile".into(),
            "app.example.test".into(),
            "tunnel".into(),
        )
        .await
        .unwrap();
        assert_eq!(id, "route-id");
    }
}
