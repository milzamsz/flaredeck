---
title: Extract tunnel status application service
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T00:00:00Z
description: The desktop tunnel-status command must preserve its response while delegating state inspection and stale-child cleanup to a reusable Rust application service.
priority: high
labels: [phase-2, application-service, tunnel]
dependsOn: [task_002_phase_1_verification, task_004_phase_1_fixtures]
---

## Conversation

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Working in `task_005_phase_2_tunnel_status.md`. Plan: create a minimal application module that owns status calculation and stale-child cleanup, leave the Tauri command as a request/response adapter, and add an application-level test for an unknown profile. Start/stop, log streaming, and crashloop behavior remain untouched for the next vertical slice.

Implemented the status extraction. `commands::tunnel::tunnel_status` now only translates Tauri state and delegates to `application::tunnel_service::status`; the TypeScript wrapper and serialized `TunnelStatus` are unchanged. Security review: no new process creation, command input, secret flow, or output boundary. Evidence: focused application test passed and `bash scripts/verify.sh` passed (4 Rust tests). Review: approve.

## Definition of Done

- [x] (agent) Status calculation and stale-child cleanup live in an application module.
- [x] (agent) Tauri and TypeScript tunnel-status contracts remain unchanged.
- [x] (agent) Application-level stopped-status regression test passes.
- [x] (agent) Full aggregate verification passes with no secret or process-lifecycle expansion.

### user

### user
