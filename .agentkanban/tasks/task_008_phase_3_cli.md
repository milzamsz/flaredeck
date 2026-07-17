---
title: Add headless diagnostic CLI
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T18:30:00+07:00
description: A headless flaredeck binary must expose safe profile and tunnel diagnostics plus doctor through shared services with human and JSON output.
priority: high
labels: [phase-3, cli]
dependsOn: [task_007_phase_2_profile_route_services]
evidence:
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
---

## Conversation

### user

### agent

Working in `task_008_phase_3_cli.md`. Completed the stable headless contract: versioned JSON envelopes, warnings/error/meta fields, correlation IDs, timestamps, documented exit categories, stderr discipline, safe session projections, bounded redacted logs, doctor checks, and observational tunnel diagnostics. Added binary-level contract tests for valid doctor JSON, no ANSI/stderr contamination, usage error code/exit, canary non-disclosure, and safe tunnel errors. The CLI constructs no Tauri application and duplicates no orchestration. `bash scripts/verify.sh` passes.

## Definition of Done

- [x] (agent) CLI invokes shared application services without desktop initialization.
- [x] (agent) Human and JSON output contracts, stable errors, correlations, and exit codes are implemented.
- [x] (agent) Safe profile/tunnel/doctor and workspace/session commands are available.
- [x] (agent) Canary, stdout/stderr, and structured contract tests pass.
- [x] (agent) Development invocation and Phase 9 naming decision are documented.

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Queued after shared service contracts are stable. No workspace/session command or arbitrary command override belongs in this task.

User approval covers the phase sequence; this initial diagnostic slice proceeds while the remaining Phase 2 fake-port coverage is tracked separately. Implemented `flaredeck-cli` as a development binary because the desktop updater currently owns the `flaredeck` binary name. It supports only `version`, `profile list`, and `doctor` with human or JSON output; it starts no desktop window and exposes no secrets. `cargo run --manifest-path src-tauri/Cargo.toml --bin flaredeck-cli -- --output json version` and `doctor` both returned valid JSON.

Evidence: JSON `profile list` returned the shared `ProfileIndex` safe display model without token values or credential contents; an invalid command returned a JSON envelope and exit code 2. `bash scripts/verify.sh` passed. Remaining Phase 3 work is stable metadata/error envelopes and tunnel diagnostics that can observe desktop-owned state without duplicating lifecycle orchestration.

### user

### user
