# Webhook Inspector Threat Model

## Scope and boundary

This model covers Phase 8 temporary routes, the loopback capture proxy, redacted event storage, desktop replay, read-only MCP event access, expiration, and cleanup. The inspector is a local development aid, not a production gateway, generic reverse proxy, traffic recorder, or API client.

The public sender is untrusted. Repository manifests are untrusted until the current fingerprint is approved. MCP callers are untrusted for approval, route selection, replay, filesystem paths, and secrets. The selected profile tunnel and its persistent routes are protected assets.

## Assets

- API tokens and tunnel credentials, which never enter inspector state;
- inbound authorization/cookie headers and personal webhook payload fields;
- local application origin and captured response metadata;
- persistent DNS/ingress routes;
- the ability to replay a request with side effects.

## Threats and controls

### Unowned route deletion or conflict

- temporary creation rejects any existing DNS name or ingress hostname/path;
- ownership stores the exact Cloudflare DNS record ID, profile, session, hostname, ingress rule, creation time, and expiry;
- cleanup deletes only that record ID and only an exact matching temporary ingress rule;
- changed/missing ownership converges to `cleanup_incomplete`, never broad deletion;
- the final `http_status:404` rule remains last;
- persistent routes are never selected by temporary cleanup.

### Secret or personal-data capture

- route opt-in is explicit in the trusted manifest;
- the proxy accepts only bounded HTTP/1.1 requests for its owned route;
- `Authorization`, `Proxy-Authorization`, `Cookie`, `Set-Cookie`, and configured sensitive headers are replaced before persistence;
- configured JSON field names and common secret names are recursively redacted before persistence;
- unsupported/ambiguous content types store metadata and a redaction marker, not raw bytes;
- raw environment, token, credential, TLS, and Cloudflare headers are never stored or returned.

### Storage exhaustion and denial of service

- request headers are limited to 16 KiB, bodies to 64 KiB, and events to 100 per route;
- retention is at most 24 hours and never later than route expiry;
- response bodies are not persisted;
- list/read results are bounded; no MCP follow stream exists;
- timeouts bound origin forwarding and replay.

### SSRF and arbitrary replay

- capture forwards only to the trusted manifest’s loopback HTTP origin;
- replay is desktop-only, explicitly confirmed for one selected event, and targets only that same recorded loopback origin/path;
- callers cannot supply a replay URL, host, headers, body, command, or environment map;
- redirects are not followed.

### Prompt injection

- MCP exposes read-only bounded event list/get operations;
- event text is untrusted display data and cannot approve trust, create routes, replay, reveal redacted values, or widen retention;
- tool schemas reject paths, targets, raw headers, and arbitrary properties.

### Proxy/process ownership

- the capture proxy is a directly spawned owned child, not a shell command;
- PID start time and executable identity are persisted;
- stop/reconciliation never kills a mismatched process;
- crashes are bounded by the existing lifecycle policy; no automatic infinite restart.

## Acceptance tests

- existing ingress/DNS conflict blocks before mutation;
- cleanup removes only an exact owned temporary route and is idempotent;
- persistent and final catch-all rules survive cleanup;
- expired/orphaned ownership reconciles safely;
- oversized/chunked/unsupported requests are rejected or metadata-only;
- header and nested JSON canaries are absent from storage, desktop, CLI, MCP, logs, and audit;
- replay cannot change origin and requires desktop approval;
- cancellation/forwarding timeout leaves no unbounded operation;
- corrupt ownership state fails closed.

## Deferred threats

TLS termination, multipart/file inspection, arbitrary binary bodies, remote inspector administration, production retention, multi-user isolation, and generic response mocking require new product and threat-model decisions.
