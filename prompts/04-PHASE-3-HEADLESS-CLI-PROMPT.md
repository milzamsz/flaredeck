# Phase 3 Prompt: Headless CLI Foundation

## Objective

Implement the `flaredeck` headless CLI over shared application services and stabilize machine-readable contracts before MCP.

## Required work

1. Confirm desktop-only initialization is separated from reusable application construction.
2. Add the CLI binary and argument parser using an approved dependency or existing capability.
3. Implement global human/JSON output policy.
4. Implement the response envelope, error codes, exit-code mapping, correlation IDs, and stderr discipline.
5. Begin with safe profile/tunnel diagnostics and `doctor` to prove parity.
6. Add workspace/session commands only when Phase 4 services exist.
7. Add JSON snapshot or structured contract tests.
8. Add canary-secret tests covering stdout and stderr.
9. Document installation and invocation for development builds.

## Constraints

- no desktop window;
- no arbitrary command or raw path escape;
- no token output;
- no ANSI in JSON;
- no separate orchestration implementation;
- no unstable field names left undocumented at phase exit.

## Acceptance criteria

- CLI and Tauri produce equivalent safe results for shared operations;
- `doctor --output json` is valid JSON;
- error paths use stable codes;
- exit codes match documented categories;
- secret canaries never appear;
- headless invocation works without desktop plugin failures.

## Exit criteria

The shared service architecture is proven through a stable local automation interface.
