# Phase 5 MCP transport

`flaredeck-mcp` uses the existing MIT-licensed `serde_json` and Tokio dependencies for local stdio JSON-RPC. No MCP SDK dependency was added, so Phase 5 adds no dependency, license, maintenance, or binary-size burden. Reconsider an SDK only if later protocol features cannot remain small and testable here.

The adapter follows the stable MCP `2025-11-25` initialization, tools, stdio, and cancellation contracts while accepting earlier supported protocol versions during negotiation. Messages are newline-delimited JSON-RPC. The process never opens a network listener. Stdout contains protocol responses only; child output is read through bounded redacted log storage and never forwarded directly. Diagnostics may use stderr, although the current implementation emits none.

The approved tool surface is `workspace_list`, `workspace_status`, `session_start`, `session_status`, `session_stop`, `public_url_get`, `logs_read`, `health_check`, and `doctor`. Workspace operations resolve only desktop-registered IDs or unique names. MCP cannot approve trust and cannot supply a path, executable, arguments, environment values, route, token, process ID, or Cloudflare operation. Safe projections omit canonical paths, executable/log paths, fingerprints, and PIDs.

All inputs reject unknown properties and enforce documented bounds. Tool failures include a stable code, retryability, required action when applicable, and correlation ID. Session start is limited to 120 seconds for MCP and handles `notifications/cancelled`; cancellation or timeout stops a runtime spawned before readiness completes.

## Acceptance evidence

`cargo test --manifest-path src-tauri/Cargo.toml --test mcp_protocol` drives the built stdio binary through initialization, discovery, registered workspace status, trusted external-runtime session start, URL retrieval, health, redacted logs, ownership-safe stop, cancellation, changed-fingerprint rejection, arbitrary-property/path rejection, and JSON-only protocol responses. A synthetic `CANARY_SECRET` inserted at the persisted-log boundary is redacted again on read.

`bash scripts/verify.sh` passes frontend lint/build, Rust format/Clippy/tests, JSON schema validation, package validation, and diff checks. Phase 5 security-review decision: **Approve**. No secret-returning capability, trust bypass, arbitrary execution/path input, network listener, or unowned cleanup path was found.

References: [MCP stdio transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [tool contract](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), and [cancellation](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation).
