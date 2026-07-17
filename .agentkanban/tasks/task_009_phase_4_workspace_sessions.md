---
title: Implement trusted workspace sessions
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T18:30:00+07:00
description: Trusted manifests must safely start, observe, and stop owned local runtimes and tunnel sessions through desktop and CLI services.
priority: critical
labels: [phase-4, trust, runtime, sessions, security]
dependsOn: [task_008_phase_3_cli]
evidence:
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
---

## Conversation

### agent

Completed the Phase 4 persistent-route boundary. Session startup verifies every persistent manifest route against the selected profile ingress before tunnel startup; it does not create or delete persistent routes. Temporary route mutation remains deliberately unavailable until Cloudflare ownership-safe cleanup is implemented, and that limitation is documented in the manifest specification. `bash scripts/verify.sh` passes with 32 Rust tests, frontend lint/build, clippy, schema, package, and diff checks.

Implemented tunnel ownership composition through the existing shared tunnel service. A session first observes a `cloudflared` process tied to the selected tunnel name; an observed tunnel is recorded but never stopped. If absent and the manifest requests it, the shared service starts the tunnel and persists PID/start-time/executable ownership; cleanup stops it only when identity still matches and lifecycle policy permits. Runtime and external-session reuse remain idempotent. `bash scripts/verify.sh` passes with 31 Rust tests and all frontend/schema/package checks. Persistent-route verification and temporary-route ownership are the remaining Phase 4 scope.

Completed lifecycle ownership handling for external runtimes: `startRuntime: false` waits for the local readiness target but creates no child process and records no runtime ownership, so normal cleanup cannot stop a process it did not create. Confirmed that desktop and CLI use the identical Tauri configuration path for approvals and session state. `bash scripts/verify.sh` passes with 30 Rust tests, lint/build, clippy, schema, package, and diff checks. Tunnel/route ownership is still required before Phase 4 exit.

Completed bounded CLI runtime-log retrieval. Runtime output is capped in memory and persisted per session as capped JSONL; lines with secret-like material are replaced before either storage path. `flaredeck-cli session logs <workspace-or-session>` returns no more than 100 safe structured entries. Added tests for log redaction and 200-entry bounded retrieval. Focused Rust tests, clippy, frontend lint, and frontend build pass. Tunnel/route ownership remains the final major Phase 4 gap.

Added the first durable session vertical slice. `active-sessions.json` stores a schema-versioned session record with workspace/profile binding, fingerprint, runtime ownership, PID start time, executable identity, public URLs, and state. Start is idempotent for a matching live owned runtime; stop checks PID start time and executable identity before terminating the owned process tree, otherwise reports `cleanup_incomplete` without killing an uncertain PID. Session start/status/stop and workspace inspect/trust-status are available in `flaredeck-cli`; approval remains desktop-only. Bounded local audit events are emitted without commands, environment values, tokens, logs, or sensitive request content. `bash scripts/verify.sh` passed with 27 Rust tests, frontend lint/build, clippy, schema, package, and diff checks. Tunnel/route ownership and CLI log retrieval still remain before Phase 4 can exit.

Added the desktop-only trust review path. The trust store now scopes fingerprints by canonical workspace, so approvals for multiple workspaces do not collide and a manifest change still invalidates only the changed workspace. New thin Tauri commands expose a safe manifest projection and approval action; Settings renders executable/args, directory, readiness, routes, profile, and environment names before approval. There is intentionally no CLI/MCP approval command. Runtime supervision also now clears inherited environment, applies only manifest-declared values, and drains stdout/stderr into a capped 200-line buffer. Rust tests (23), clippy, frontend lint/build, and diff checks pass. Durable session ownership/recovery, tunnel/route composition, audit events, and CLI start/status/stop remain.

Implemented the next Phase 4 vertical slice: readiness configuration now has bounded defaults and schema-aligned limits; TCP probes retry to their deadline; HTTP probes are local-only, bodyless, bounded per attempt, and do not follow redirects. `start_approved_runtime` now authorizes before spawning, waits for readiness, and terminates the session-owned runtime on readiness failure. Tests cover local HTTP status, retry timeout, untrusted spawn denial, and failed-readiness compensation. `bash scripts/verify.sh` passed: frontend lint/build, Rust format/clippy/tests (23 passed), schema, package, and diff checks. Session persistence, owned tunnel/route orchestration, bounded surfaced logs, desktop trust UX, and CLI session commands remain outstanding.

### user

### agent

Working in `task_009_phase_4_workspace_sessions.md`. Final review closed the remaining external-runtime defect: `startRuntime: false` no longer resolves or requires an executable that FlareDeck does not spawn. Protocol-level lifecycle tests now exercise this ownership mode, idempotent stop, cancellation, and trust invalidation. The approved Phase 4 boundary remains deliberately limited to verifying persistent routes; temporary mutation proceeds only under the Phase 8 threat model and ownership design. `bash scripts/verify.sh` passes.

## Definition of Done

- [x] (agent) Manifests, local trust, readiness, runtime, session persistence, logs, and audit state are implemented.
- [x] (agent) Runtime/tunnel lifecycle is direct, bounded, idempotent, and ownership-aware.
- [x] (agent) Desktop alone can approve trust; CLI/MCP cannot bypass it.
- [x] (agent) Persistent routes are verified and temporary mutation remains excluded until Phase 8.
- [x] (agent) Success, failure, cancellation, cleanup, changed trust, and secret redaction are tested.

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Queued after the CLI proves shared service construction. Implementation must be decomposed into manifest, trust, runtime, readiness, and session slices before code changes begin.

Plan approved by the user’s phase authorization: (1) parse and validate `.flaredeck/project.yaml`, including local-only readiness, bounded values, and path containment; (2) calculate and persist a fail-closed trust fingerprint outside the repository; (3) directly spawn only the approved executable/args with bounded logs and platform-aware termination; (4) add bounded TCP/HTTP readiness probes; (5) compose ownership-aware, idempotent session start/stop and expose it through desktop/CLI. MCP remains excluded until every preceding slice is verified.

Implemented and tested the first two boundaries: canonical manifest discovery; strict typed validation for shell mode, traversal, local readiness/routes, and secret-like environment values; SHA-256 fingerprints; fail-closed local approvals; and bounded loopback TCP readiness. No approval is exposed through CLI, and no runtime is spawned yet. `bash scripts/verify.sh` passed with 15 Rust tests before the latest discovery-only test; focused discovery tests pass. Runtime/session work remains pending.

### user

### user
