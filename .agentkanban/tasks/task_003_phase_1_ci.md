---
title: Add pull-request verification CI
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T00:00:00Z
description: Pull requests must run the approved aggregate verification without credentials and without changing the release workflow.
priority: high
labels: [phase-1, ci]
dependsOn: [task_002_phase_1_verification]
---

## Conversation

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Pending the stable local verifier. Scope: add a PR-only workflow that invokes it and keeps `.github/workflows/release.yml` behavior unchanged.

Plan: add one Ubuntu pull-request job with the same Node, Rust, and Linux desktop dependencies used by the release workflow, then run `bash scripts/verify.sh`. It has read-only contents permission and no secrets.

Implemented `.github/workflows/verify.yml`. It runs only for pull requests, installs the release workflow's Linux prerequisites, and invokes the aggregate verifier with read-only repository access. Evidence: `git diff --check` passed; `git diff --exit-code -- .github/workflows/release.yml` confirmed the release workflow was untouched; `bash scripts/verify.sh` passed locally. GitHub Actions execution remains pending on the next pull request.

## Definition of Done

- [x] (agent) PR workflow has no credentials or release permissions.
- [x] (agent) CI invokes the same aggregate verifier as local development.
- [x] (agent) Existing release workflow is unchanged.
- [x] (agent) Local verifier passes.

### user

### user
