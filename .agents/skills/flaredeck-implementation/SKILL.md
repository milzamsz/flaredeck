---
name: flaredeck-implementation
description: Implement scoped FlareDeck features or refactors using the repository source-of-truth documents, shared Rust service architecture, tests, security invariants, and Agentic Kanban acceptance criteria. Use for non-trivial code changes in FlareDeck.
argument-hint: "[task ID or implementation objective]"
---

# FlareDeck Implementation

## When to use

Use this skill for a specific implementation-ready task. Do not use it to make unresolved architecture or product decisions.

## Required references

Read, in order:

1. repository `AGENTS.md`;
2. active task;
3. relevant phase in `PLAN.md`;
4. relevant product, ADR, domain, architecture, technical, spec, and security docs.

## Procedure

1. Inspect current code and tests in every affected boundary.
2. Confirm dependencies and blockers are resolved.
3. State a small implementation plan mapped to acceptance criteria.
4. Implement one coherent vertical slice.
5. Keep Tauri, CLI, and MCP adapters thin and use shared application services.
6. Add tests for success, failure, cleanup, idempotency, and security behavior relevant to the task.
7. Run the repository verification commands.
8. Review the diff for unrelated churn, generated files, dependency creep, and secrets.
9. Update affected docs and schemas.
10. Produce completion evidence.

## Hard checks

- one profile remains one tunnel and one token identity;
- no token or secret value crosses an interface;
- no arbitrary shell execution;
- no MCP trust approval;
- process and route cleanup are ownership-aware;
- current WSL, YAML, scope-hint, and updater behavior is preserved unless the task explicitly changes it through an approved decision.

## Expected output

- outcome;
- files changed;
- tests;
- commands and results;
- security and compatibility notes;
- documentation changes;
- residual risk and rollback.
