# Phase 9 Prompt: Release and Ecosystem Hardening

## Objective

Package desktop, CLI, and MCP capabilities safely across supported platforms and preserve updater compatibility.

## Required work

1. Inspect current release workflow, artifact names, updater metadata, signing, and download-page integration.
2. Decide whether CLI/MCP are bundled, installed beside the desktop binary, or released separately.
3. Define version compatibility and schema compatibility checks.
4. Add release artifact checksums and signing where supported.
5. Add smoke tests for Windows, macOS, Linux, and WSL-relevant workflows.
6. Test data migration from existing profile-only versions.
7. Test rollback boundaries and document unsupported downgrade cases.
8. Verify updater cannot leave mismatched companion binaries.
9. Update release notes and installation documentation.

## Constraints

- do not rotate updater signing keys;
- do not break stable release download URLs without migration;
- do not publish untested companion binaries;
- do not claim platform support without smoke-test evidence.

## Exit criteria

A user can install or upgrade FlareDeck and receive compatible desktop, CLI, and MCP components without losing existing profiles or secrets.
