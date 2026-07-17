---
title: Add the Phase 1 aggregate verifier
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T00:00:00Z
description: A clean checkout must produce one unambiguous frontend, Rust, schema, and documentation verification result without Cloudflare credentials.
priority: high
labels: [phase-1, verification]
dependsOn: [task_001_phase_0_baseline]
---

## Conversation

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Pending Phase 0 completion. Scope: add `scripts/verify.sh`, use existing npm/cargo commands, separate platform-sensitive checks, and document actual prerequisites. Do not alter release behavior or add dependencies.

Plan: run frontend lint/build, Rust format/clippy/test, and package validation in one fail-fast script rooted at the repository. Apply only mechanical Rust formatting required for the documented format check. Prove the script succeeds without Cloudflare credentials.

Implemented `scripts/verify.sh` with the existing npm, Cargo, and package-validation commands. Applied `cargo fmt --all`, affecting only baseline formatting in five Rust files. Evidence: `bash scripts/verify.sh` passed (lint, build, fmt, clippy with warnings denied, 3 Rust tests, JSON schema, and package structure). Review: approve; no dependency, release-workflow, credential, or runtime-behavior change.

## Definition of Done

- [x] (agent) One fail-fast aggregate verifier covers the documented standard checks.
- [x] (agent) The verifier passes without Cloudflare credentials.
- [x] (agent) Verification documentation names the aggregate command.
- [x] (agent) Diff contains no release-workflow or secret-handling change.

### user

### user
