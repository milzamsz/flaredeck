# Phase 1 Prompt: Repository Readiness and Verification Baseline

## Objective

Create one dependable verification path for AI-assisted development without altering FlareDeck product behavior.

## Required work

1. Inspect current package scripts, Cargo configuration, tests, GitHub workflows, and platform prerequisites.
2. Add or refine an aggregate verification script that runs the correct frontend and Rust checks.
3. Add pull-request CI while preserving the existing release workflow.
4. Ensure standard CI does not require real Cloudflare credentials.
5. Add fixture conventions for temporary files, fake processes, and mocked HTTP.
6. Add validation for JSON schemas and documentation-package structure.
7. Expand tests only where necessary to protect existing critical behavior before refactoring.
8. Update `AGENTS.md` and contributor documentation if actual commands differ.

## Constraints

- do not switch package managers;
- do not combine release workflow redesign;
- do not introduce enhancement behavior;
- do not weaken warnings or skip failing checks to make CI green;
- isolate platform-specific tests correctly.

## Required tests

- clean-checkout verification;
- frontend lint/build;
- Rust format/clippy/test;
- documentation/schema validation;
- confirmation that secret-required tests are opt-in.

## Deliverables

- CI workflow;
- aggregate verify script;
- fixture/test utilities where approved;
- updated verification documentation;
- evidence of local and CI-equivalent results.

## Exit criteria

A subsequent agent can make a scoped change and receive one unambiguous pass/fail verification result.
