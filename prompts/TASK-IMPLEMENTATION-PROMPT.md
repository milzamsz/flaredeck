# Task Implementation Prompt

Implement the active FlareDeck Agentic Kanban task.

## Inputs to resolve

- Task ID and title:
- Parent phase:
- Required outcome:
- Dependencies:
- Blockers:
- In scope:
- Explicitly out of scope:
- Acceptance criteria:
- Required evidence:

## Procedure

1. Read `AGENTS.md` and all source-of-truth documents referenced by the task.
2. Inspect affected code, tests, schemas, and prior behavior.
3. Confirm the task is implementation-ready. If a product, security, migration, or architecture decision is unresolved, stop and create a blocking decision item instead of guessing.
4. Produce a small file-level implementation plan.
5. Implement the minimum coherent change.
6. Add tests for success, failure, cleanup, and security behavior relevant to the task.
7. Run required verification.
8. Inspect the diff for unrelated changes, secret leakage, generated files, and contract drift.
9. Update documentation and examples that became inaccurate.

## Required constraints

- preserve all hard invariants in `AGENTS.md`;
- no arbitrary shell execution;
- no secret output;
- no duplicated Tauri/CLI/MCP orchestration;
- no wider Cloudflare mutation than approved;
- no task completion without verification evidence.

## Final report

- Outcome
- Files changed
- Behavior changes
- Tests added/updated
- Verification commands and results
- Security checks
- Compatibility/migration impact
- Documentation changes
- Remaining risks/blockers
- Rollback
