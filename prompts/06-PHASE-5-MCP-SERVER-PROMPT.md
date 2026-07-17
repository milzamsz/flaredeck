# Phase 5 Prompt: Local MCP Server

## Objective

Implement `flaredeck-mcp` as a local stdio adapter over accepted workspace and session services.

## Required work

1. Review the current MCP specification and selected Rust SDK.
2. Document dependency, license, version, and maintenance implications.
3. Implement stdio initialization and clean process startup.
4. Reserve stdout exclusively for protocol messages and stderr for diagnostics.
5. Implement only the approved tools in `docs/specs/mcp-tools.md`.
6. Map MCP calls to operation context and shared application services.
7. Enforce schema validation, result bounds, redaction, timeout, and cancellation.
8. Add protocol-level tests that capture stdout and stderr.
9. Test untrusted, changed, missing, conflict, failure, and cleanup paths.
10. Validate with MCP Inspector or an equivalent protocol client.

## Explicit prohibitions

- no HTTP listener;
- no arbitrary command;
- no arbitrary filesystem path;
- no token or environment-value tool;
- no trust-approval tool;
- no arbitrary DNS or Cloudflare API tool;
- no direct child-process logs written to protocol stdout;
- no excessive tool splitting.

## Required acceptance evidence

- initialization and tool discovery transcript;
- tool schemas;
- complete start/status/log/health/URL/stop scenario;
- untrusted-workspace rejection;
- canary-secret redaction result;
- cancellation result;
- proof stdout contains protocol only.

## Exit criteria

A local MCP client can safely execute the complete approved development-session lifecycle.
