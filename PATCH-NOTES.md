# Implementation and Release Notes

## Replacement file

`AGENTS.md` is intended to replace the current repository-root file after a maintainer reviews it against newer changes. It preserves the existing operational rules and adds the AI-development source-of-truth hierarchy, workspace/session rules, CLI/MCP rules, and definition of done.

## New root documents

- `PRODUCT-SCOPE.md`
- `DOMAIN-MODEL.md`
- `ARCHITECTURE.md`
- `TECHNICAL.md`
- `DESIGN.md`
- `PLAN.md`

## New directories

- `docs/adr/`
- `docs/specs/`
- `docs/security/`
- `docs/implementation/`
- `.agents/skills/`
- `prompts/`
- `templates/`
- `examples/`

## Release-sensitive files

- `src-tauri/tauri.conf.json` preserves the existing updater identity/key/endpoint and adds version-matched external binaries.
- `.github/workflows/release.yml` builds platform companions, keeps incomplete releases as drafts, emits checksums, and publishes only after every target passes.
- `docs/specs/release-compatibility.json` makes versions, schemas, updater identity, and stable artifact names machine-checkable.

## Recommended commit sequence

1. documentation and ADR baseline;
2. Agent Skills, prompts, and templates;
3. Phase 1 verification/CI implementation;
4. each later phase as separate task-oriented commits.
