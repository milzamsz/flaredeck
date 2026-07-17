---
name: flaredeck-verification
description: Independently review and verify a FlareDeck change against task acceptance criteria, architecture, security, process ownership, cross-platform behavior, tests, migration, and documentation. Use after implementation or before merging.
argument-hint: "[task, branch, commit, or diff to review]"
context: fork
---

# FlareDeck Verification

## Review process

1. Read the active task and relevant source-of-truth documents.
2. Inspect the complete diff and affected existing code.
3. Build an acceptance-criteria matrix.
4. Review product scope, domain invariants, dependency direction, interface parity, trust, secrets, process lifecycle, routes, error contracts, and migration.
5. Run relevant verification commands and focused risk tests.
6. Classify findings by severity.
7. Return a merge decision based on evidence.

## Mandatory review areas

- unrelated scope expansion;
- duplicated orchestration in Tauri, CLI, or MCP;
- secret leakage in success, error, logs, audit, fixtures, or snapshots;
- trust bypass or incorrect fingerprint fields;
- arbitrary command or path escape;
- ownership and idempotency of stop/cleanup;
- pre-existing tunnel protection;
- cross-platform process behavior;
- schema and error compatibility;
- tests for failure and cleanup, not only happy path;
- documentation and examples matching implementation.

## Decision values

- `Approve`
- `Approve with follow-up`
- `Block`

Include findings ordered by severity and the exact evidence required to resolve each blocked item.
