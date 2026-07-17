---
title: Extract tunnel lifecycle application service
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T00:00:00Z
description: Tunnel start, stop, and restart must delegate to shared application logic while preserving platform termination, logs, crashloop behavior, and the desktop API.
priority: high
labels: [phase-2, process-lifecycle, security]
dependsOn: [task_005_phase_2_tunnel_status]
---

## Conversation

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Queued after the status slice. It must introduce only the ports needed to test process ownership and crashloop behavior; no shell command interface is permitted.

Planning review: start currently combines profile-index reads, direct cloudflared spawning, Tauri log emission, early-exit handling, and crashloop updates; stop owns the platform process-tree command. The slice will move lifecycle orchestration into `application::tunnel_service`, retain concrete spawning/termination in a cloudflared adapter, and provide a narrow log sink so Tauri is only an adapter. Acceptance tests must cover already-running rejection, crashloop rejection, early exit, successful start, repeated stop, and no token serialization. No caller-supplied executable, arguments, or environment can be introduced.

Moved start, stop, and restart orchestration into `application::tunnel_service`. The Tauri handlers now pass only managed state, profile ID, and a log-event callback. The command remains fixed to `cloudflared tunnel --no-autoupdate [--config] run <profile tunnel>` with `TUNNEL_ORIGIN_CERT`; no caller-controlled command or environment was added. `cargo check --manifest-path src-tauri/Cargo.toml` passes. Remaining work: add lifecycle tests and run the aggregate verifier.

Added application-level tests for crashloop rejection before spawning and idempotent stop without an owned child. Evidence: `cargo test --manifest-path src-tauri/Cargo.toml application::tunnel_service::tests` passed (3 tests) and `bash scripts/verify.sh` passed. Security review: no token crosses the service; process arguments remain fixed; only the owned child map is removed on stop. Review: approve.

## Definition of Done

- [x] (agent) Lifecycle orchestration is shared and Tauri is limited to event adaptation.
- [x] (agent) Fixed direct-spawn command, crashloop policy, and platform stop behavior remain unchanged.
- [x] (agent) Crashloop and idempotent-stop tests pass.
- [x] (agent) Aggregate verification passes.

### user

### user

### user

### user
