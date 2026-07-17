---
title: Extract profile and route read services
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T18:30:00+07:00
description: Safe profile reads and route/DNS operations must become shared services without changing token storage, preflight, scope hints, or API/CLI fallback.
priority: high
labels: [phase-2, profiles, routes, security]
dependsOn: [task_006_phase_2_tunnel_lifecycle]
evidence:
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
---

## Conversation

### user

### agent

Working in `task_007_phase_2_profile_route_services.md`. The user authorized autonomous phase completion, so the previously implemented slice was reviewed and closed directly. Added the missing fake `RoutePort` test seam: route orchestration is now proven without Cloudflare while production continues through `CfClient`, preserving token containment, scope hints, preflight behavior, and CLI fallback. `bash scripts/verify.sh` passes with frontend lint/build and 42 Rust/contract tests.

## Definition of Done

- [x] (agent) Profile and route operations are callable through shared services.
- [x] (agent) Tauri adapters remain thin and existing public contracts are preserved.
- [x] (agent) Route orchestration has fake-port coverage without a real Cloudflare call.
- [x] (agent) Token, fallback, ingress, WSL, process, and updater invariants remain unchanged.
- [x] (agent) Verification and migration/rollback evidence are documented.

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Queued after lifecycle extraction to avoid mixing process and Cloudflare behavior in one review.

Profile read extraction is complete: `application::profile_service::list` now owns index loading and parsing, and the desktop handler reuses it. Invalid index data preserves existing empty-index fallback behavior. Route/DNS extraction remains pending because it must preserve Cloudflare scope hints and API/CLI fallback as a separate security-sensitive slice.

Route extraction is now complete: Tauri handlers delegate API route creation and zone lookup to `application::route_service`, and the no-token `cloudflared tunnel route dns -f` fallback delegates there too. `CfClient` remains the only token consumer, preserving its scope hints; no real Cloudflare request was made in verification. `bash scripts/verify.sh` passes. Remaining Phase 2 work is fake-port route coverage and operation context, so this task stays active.

### user

### user
