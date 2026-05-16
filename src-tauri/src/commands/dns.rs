use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::error::ResolveErrorKind;

use crate::error::{AppError, AppResult};
use crate::types::DnsLookupResult;

#[tauri::command]
pub async fn dns_check(hostname: String) -> AppResult<DnsLookupResult> {
    let mut opts = ResolverOpts::default();
    opts.timeout = std::time::Duration::from_secs(3);
    opts.attempts = 1;

    let resolver = TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), opts);
    let lookup = match resolver.lookup_ip(&hostname).await {
        Ok(l) => l,
        Err(e) => {
            if matches!(e.kind(), ResolveErrorKind::NoRecordsFound { .. }) {
                return Ok(DnsLookupResult {
                    resolved: false,
                    addresses: Vec::new(),
                });
            }
            return Err(AppError::Dns(e.to_string()));
        }
    };

    let addresses: Vec<String> = lookup.iter().map(|ip| ip.to_string()).collect();
    Ok(DnsLookupResult {
        resolved: !addresses.is_empty(),
        addresses,
    })
}
