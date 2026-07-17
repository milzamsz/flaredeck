---
title: Publish the v0.3.0 release candidate
lane: in-progress
created: 2026-07-18T00:20:00+07:00
updated: 2026-07-18T00:20:00+07:00
description: Review and publish the completed Phase 0-9 implementation through PR verification, merge, consistent version bump, tag, and guarded cross-platform release workflow.
priority: critical
labels: [release, security]
dependsOn: [task_014_phase_9_release_hardening]
---

## Conversation

### user

Approved committing the completed work, pushing a branch, opening and merging verified PRs, bumping the version, tagging, and running the cross-platform release workflow.

### agent

Working in `task_015_publish_v0_3_0.md`. The implementation is based on the existing `v0.2.3` hotfix branch while `main` remains at `v0.2.2`; the feature PR will retain that hotfix and target `main`. The generated enhancement ZIP remains excluded. After merge, the release version will be `0.3.0`, reflecting the new CLI/MCP/workspace/webhook feature set.

### user

## Definition of Done

- [ ] (agent) Reviewed implementation is committed on a fresh branch and the generated ZIP is excluded.
- [ ] (agent) Feature PR verification passes and the PR is merged into `main`.
- [ ] (agent) Version `0.3.0` is consistent across every guarded source and merged after verification.
- [ ] (agent) Annotated `v0.3.0` is pushed only from the verified release commit.
- [ ] (agent) Release workflow and native/manual gates complete without bypassing signing or platform failures.

### agent

Working in `task_015_publish_v0_3_0.md`. Pre-commit audit passed: aggregate verification is green (54 tests across eight suites), release/version contract is green, staged diff check is clean, no private-key/token signature matched, no credential/generated path is staged, and the generated ZIP plus release binaries/build output are ignored. `RELEASING.md` and the verifier now include both root `package-lock.json` version fields.

### user
