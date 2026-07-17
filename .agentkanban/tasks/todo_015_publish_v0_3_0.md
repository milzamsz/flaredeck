# Iteration 1

- [x] Audit untracked files, secrets, diff, and release prerequisites.
- [x] Commit, push, open the feature PR, and monitor verification.
- [x] Merge the feature PR and create the `0.3.0` version-bump PR.
- [x] Merge the verified version bump and push annotated `v0.3.0`.
- [ ] Monitor release artifacts, signatures, checksums, updater metadata, and platform checks.

# Iteration 2

- [x] Restore universal macOS companion binaries at the tauri target bundle path expected by the Tauri builder.
- [x] Re-run the release workflow and verify universal macOS publishes successfully.
- [ ] Fix Windows packaging blocker (`tauri-action` `light.exe`) and complete all-platform publish.
  - [x] Implemented a Windows-only release workflow workaround to skip WiX and build only NSIS (`-b nsis`) in
    `Build and publish` while preserving updater JSON preference for NSIS artifacts.
  - [ ] Validate via new workflow rerun that Windows publish now succeeds and the release can publish.
