# Webhook Data, Redaction, and Replay Policy

## Capture limits

- one capture stream per owned temporary route;
- 100 newest events per route;
- 16 KiB aggregate request-header limit;
- 64 KiB request-body limit;
- 24-hour maximum retention, shortened to the route expiration;
- accepted stored bodies: UTF-8 `application/json`, `application/*+json`, `text/*`, and `application/x-www-form-urlencoded`;
- unsupported, invalid UTF-8, chunked, or oversized bodies are not stored; safe metadata records the reason;
- response bodies are forwarded but never persisted.

State lives under the FlareDeck application-data directory. It is excluded from repositories, browser storage, audit details, screenshots, and client configuration.

## Redaction before persistence

Header names are case-insensitive. Always redact `authorization`, `proxy-authorization`, `cookie`, `set-cookie`, `x-api-key`, and headers whose names contain `token`, `secret`, `password`, or `key`.

For JSON bodies, recursively replace values whose field names contain `token`, `secret`, `password`, `authorization`, `cookie`, `private_key`, or `api_key`. A future manifest extension may add field names but cannot disable the defaults. Query parameter values for the same names are replaced in stored paths. Stable marker: `[REDACTED]`.

Redaction occurs in memory before append. There is no “show unredacted” action and no raw shadow file.

## Replay policy

Replay is disabled for MCP and CLI in Phase 8. Desktop replay requires an explicit per-event confirmation describing method, original local target, and side-effect warning.

Replay reconstructs only the redacted stored request and sends it to the event’s original trusted loopback origin/path. It does not follow redirects, restore redacted values, accept caller-supplied headers/body/URL, target the public hostname, or replay an expired/deleted route. Because redaction may make authentication fail, developers should prefer asking the webhook provider to resend when credentials are required.

## Deletion and expiry

Stopping the owning session removes the route and its event file when manifest cleanup policy allows. Reconciliation removes expired event files even when route cleanup is incomplete. Manual “clear events” deletes only the selected route’s capture file; audit events remain.
