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

- [x] (agent) Reviewed implementation is committed on a fresh branch and the generated ZIP is excluded.
- [x] (agent) Feature PR verification passes and the PR is merged into `main`.
- [ ] (agent) Version `0.3.0` is consistent across every guarded source and merged after verification.
- [ ] (agent) Annotated `v0.3.0` is pushed only from the verified release commit.
- [ ] (agent) Release workflow and native/manual gates complete without bypassing signing or platform failures.

### agent

### user

Local release contract verification is green (`release:verify`) and `bash scripts/verify.sh` passes. `main` currently has a clean merge including tagged `v0.3.0` commit `356b42f` and released to remote as annotated tag.

I checked GitHub Actions: push `v0.3.0` triggered three release runs. The current latest `release` run (`29561483393`) has the tag set to `v0.3.0` but both `Build Windows` and `Build macOS (universal)` failed in the `Build and publish` step with command:

`npm ["run","tauri","build","--","--config","src-tauri/tauri.release.conf.json" ... ]` failed with exit code 1

No `v0.3.0` GitHub release exists yet, so publish is still blocked at this final gate.

Working in `task_015_publish_v0_3_0.md`. Pre-commit audit passed: aggregate verification is green (54 tests across eight suites), release/version contract is green, staged diff check is clean, no private-key/token signature matched, no credential/generated path is staged, and the generated ZIP plus release binaries/build output are ignored. `RELEASING.md` and the verifier now include both root `package-lock.json` version fields.

### user

### agent

Working in `task_015_publish_v0_3_0.md`. Commit `c236751` is pushed on `feat/ai-development-control-plane`; PR #11 targets `main`: https://github.com/milzamsz/flaredeck/pull/11. The PR retains the existing `v0.2.3` hotfix that had not yet reached `main`. Waiting for GitHub verification before merge.

### user

### agent

Working in `task_015_publish_v0_3_0.md`. PR CI exposed an environment-dependent CLI contract: `tunnel status` skipped profile validation when `cloudflared` was unavailable. The bypass is removed, the focused CLI suite passes, and `bash scripts/verify.sh` passes in full before the corrective push.

### user

### agent

Working in `task_015_publish_v0_3_0.md`. PR #11 passed Verify on `f4a86c9` and merged into `main` as `9ebee43`. Release branch `release/v0.3.0` now carries only the version/release-note changes; the release verifier also guards the FlareDeck entry in `Cargo.lock`.

### user

### agent

Working in `task_015_publish_v0_3_0.md`. The complete `0.3.0` version contract passes `bash scripts/verify.sh` locally and is proposed in PR #12: https://github.com/milzamsz/flaredeck/pull/12. No tag exists yet; it remains gated on this PR's CI and merge commit.

### user

### agent

Working in `task_015_publish_v0_3_0.md`. PR #12 passed CI and merged as `b5927b7`; annotated `v0.3.0` triggered release run 29560184350. Windows failed before packaging because the new `Cargo.lock` guard assumed LF-only checkout text. No GitHub release was created; the failed matrix was canceled. A CRLF-compatible guard is being verified before replacing the unpublished failed tag.

### user

### agent

Working in `task_015_publish_v0_3_0.md`. The CRLF fix merged through PR #13 and the replacement release run 29560543949 passed Windows release verification. macOS universal packaging then exposed missing per-architecture sidecar filenames: the preparation step built both architectures but retained only universal copies. The run was canceled before a release existed; preparation now preserves the two Tauri inputs and the universal smoke artifact.

### user

### agent

I pulled the full run logs for `29561483393` and confirmed this concrete root cause: the macOS job fails while bundling with `Failed to copy binary from "/Users/runner/work/flaredeck/flaredeck/src-tauri/target/universal-apple-darwin/release/flaredeck-cli": does not exist`. Windows still reports `failed to run .../light.exe` in the same step, but macOS now has a deterministic artifact-placement failure.

I patched `scripts/prepare-sidecars.mjs` so the universal branch also copies the lipo output into `src-tauri/target/universal-apple-darwin/release/<companion>` before returning, which is where the Tauri bundler expects it. This is a minimal fix and keeps existing `src-tauri/binaries/<companion>-<target>` artifacts for smoke testing.

Evidence collected:

- `node --check scripts/prepare-sidecars.mjs`
- `npm run release:verify`
- `gh run view 29561483393 --log-failed > /tmp/release-29561483393.log` (confirmed macOS line 514 failure message)

Next action: re-run `Release` workflow on `v0.3.0` and confirm both Windows/macOS packaging complete and release publishes.

### user
