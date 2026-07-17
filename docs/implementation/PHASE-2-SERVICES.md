# Phase 2 shared-service evidence

Tunnel status/start/stop, profile reads, route/DNS operations, and later workspace/session interfaces enter through `application` services. Tauri handlers remain translation adapters and retain their existing command names and response types. `route_service` keeps token access inside `CfClient`, preserves scope hints and the fixed `cloudflared tunnel route dns -f` fallback, and now has fake-port coverage without making a Cloudflare request.

Rollback is local: handlers can delegate back to their prior implementation without changing stored profile, YAML, token, tunnel credential, or updater formats. No dependency, schema migration, token-write path, WSL behavior, ingress catch-all handling, or platform termination behavior changed in this extraction.

Evidence: `bash scripts/verify.sh` passes, including application-service tests for tunnel status/lifecycle/crashloops, safe profile parsing, exact persistent route matching, and fake route orchestration.
