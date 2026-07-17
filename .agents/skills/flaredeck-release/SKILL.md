---
name: flaredeck-release
description: Prepare and verify FlareDeck desktop, CLI, and MCP release artifacts while preserving updater signing, version compatibility, migrations, platform packaging, checksums, and release evidence. Use for Phase 9 or an approved release task.
argument-hint: "[release version or release candidate]"
disable-model-invocation: true
---

# FlareDeck Release

## Preconditions

- implementation and verification tasks are complete;
- release scope and version are approved;
- signing configuration is available through secure CI or local release environment;
- no signing key rotation is planned inside the release task.

## Procedure

1. Read `RELEASING.md`, current workflow, Phase 9 plan, and release checklist.
2. Confirm desktop, CLI, MCP, schema, and data-migration version compatibility.
3. Run full verification and release builds on supported targets.
4. Smoke-test existing profile migration, tunnel operation, CLI doctor, and MCP discovery.
5. Verify artifact names, stable links, updater metadata, signatures, and checksums.
6. Inspect artifacts and logs for secrets.
7. Prepare release notes with migration and known limitations.
8. Record rollback and unsupported downgrade boundaries.

## Output

- artifact matrix;
- platform test matrix;
- migration results;
- signature/checksum results;
- release decision;
- release notes;
- known issues and rollback.
