---
title: Implement owned temporary routes and webhook inspector
lane: done
created: 2026-07-17T19:30:00+07:00
updated: 2026-07-17T23:25:00+07:00
description: Trusted temporary routes must be ownership-safe, expiring, captured with pre-storage redaction, and replayable only through explicit restricted desktop approval.
priority: critical
labels: [phase-8, security, routes, webhook, retention]
dependsOn: [task_012_phase_7_desktop_workspace_ux]
evidence:
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
---

## Conversation

### user

Approved by the request to complete every roadmap prompt autonomously.

### agent

Working in `task_013_phase_8_temporary_routes_webhooks.md`. Completed the mandatory design gate before code: product boundary, separate webhook threat model, bounded retention/redaction/replay policy, and exact route ownership/conflict/compensation contract are now explicit. The implementation may proceed only within those controls.

### user

## Definition of Done

- [x] (agent) Temporary route create/cleanup/reconciliation is exact, owned, expiring, and persistent-route safe.
- [x] (agent) Capture proxy is loopback-only, bounded, and redacts before persistence.
- [x] (agent) Replay is desktop-confirmed and cannot target an arbitrary host.
- [x] (agent) MCP event access is read-only, typed, bounded, and redacted.
- [x] (agent) Security tests and aggregate verification pass with current docs.

### agent

Implemented the Phase 8 vertical slice: exact temporary DNS/ingress/proxy ownership with expiry and compensation; loopback bounded capture with pre-storage redaction; safe desktop route/event projections and per-event replay confirmation; temporary ownership in CLI/MCP URL output; and read-only bounded MCP list/get tools with no replay surface. Focused Rust tests, clippy, frontend lint, and frontend build pass. Aggregate verification and final security review remain before moving this task to done.

### agent

Working in `task_013_phase_8_temporary_routes_webhooks.md`. Implementation review found and resolved three boundary issues: per-route event files now avoid cross-proxy writer collisions; percent-encoded sensitive query names and duplicate headers fail safe; and event files are deleted even when infrastructure cleanup remains incomplete. Replay now shows the original loopback target plus a side-effect warning. `bash scripts/verify.sh` passes: lint, production build, warnings-denied Rust checks, 52 Rust tests across eight suites, JSON/schema validation, skill validation, and package validation. Diff check is clean and the canary scan found only intentional tests/documentation. Moving to review.

### user

### agent

Working in `task_013_phase_8_temporary_routes_webhooks.md`. Self-review against the Phase 8 acceptance criteria, threat model, hard invariants, and recorded evidence has no unresolved finding. Persistent routes and the final catch-all remain protected; capture/replay are bounded and loopback-only; MCP remains read-only; retention cleanup survives partial infrastructure failure; and no token scope was widened. Moving the reviewed task to done.

### user
