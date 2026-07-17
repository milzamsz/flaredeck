---
title: Harden desktop and companion releases
lane: done
created: 2026-07-17T23:30:00+07:00
updated: 2026-07-18T00:10:00+07:00
description: Package version-matched CLI, MCP, and webhook companions with the desktop release while preserving updater identity, signing, migration, rollback, and stable download behavior.
priority: critical
labels: [phase-9, release, security]
dependsOn: [task_013_phase_8_temporary_routes_webhooks]
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

Working in `task_014_phase_9_release_hardening.md`. Release design decision: ship CLI, MCP, and webhook proxy as version-matched adjacent companions in every desktop installer/updater artifact. A single updater bundle is the atomic compatibility unit; standalone, independently updated companions remain unsupported. Preserve product identity, updater public key, endpoint, installer names, and existing profile/secret formats.

### user

## Definition of Done

- [x] (agent) Desktop and all companions are packaged as one version-compatible release unit.
- [x] (agent) Release workflow verifies versions, artifacts, smoke behavior, checksums, and updater metadata on every target.
- [x] (agent) Existing profile/secret and application-state migration plus rollback boundaries are tested and documented.
- [x] (agent) Stable download names, signing assumptions, licenses, and release notes are current.
- [x] (agent) Full verification and the release-candidate audit pass with honest platform evidence.

### agent

Working in `task_014_phase_9_release_hardening.md`. Release implementation and audit are complete. The release-only Tauri overlay requires exact-version CLI/MCP/proxy sidecars; target preparation handles Linux/Windows and macOS universal; release jobs smoke CLI version/doctor, MCP version/init/11 tools, and proxy version before an all-platform draft may publish. The workflow guards tag/version, preserves updater identity/key/endpoint, signs the complete unit, generates SHA-256 checksums, and fails closed on partial builds. Additive migration and corrupt-state tests pass. Linux release evidence includes an actual `.deb`, identical pre/post-package sidecar hashes, extracted companion execution, isolated doctor, no artifact canary, and checksum. Aggregate verification passes 54 tests across eight suites. Cross-platform native GUI/updater/WSL execution is explicitly retained as the tagged RC manual gate; the decision is Ready with known limitations, not an unsupported platform claim. Moving to review.

### user

### agent

Working in `task_014_phase_9_release_hardening.md`. Final self-review against the Phase 9 prompt and release-verification prompt found no unresolved source or packaging defect. A final release-contract check, Linux sidecar smoke, diff check, secret scan, workflow parse, migration suite, and aggregate verifier pass. Platform-native launch, OS signing/notarization, live updater installation, and WSL remain explicit pre-publication RC checks enforced by the draft workflow and documented as limitations. Moving the reviewed implementation task to done.

### user
