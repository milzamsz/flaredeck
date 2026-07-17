# Bug Fix Prompt

Fix the reported FlareDeck bug without using the bug as permission for unrelated refactoring.

## Procedure

1. Reproduce or establish the failure from logs/tests/code evidence.
2. Classify the affected layer: UI, Tauri adapter, application service, domain, infrastructure adapter, persistence, process lifecycle, Cloudflare API, CLI, or MCP.
3. Identify root cause, not only the visible symptom.
4. Add a failing regression test when technically feasible.
5. Implement the smallest safe fix.
6. Test adjacent failure and cleanup paths.
7. Verify cross-interface impact where shared services are involved.
8. Update docs only if documented behavior changes or was wrong.

## Special checks

- secret output;
- process or route ownership;
- repeated calls and idempotency;
- Windows/macOS/Linux/WSL differences;
- schema and migration compatibility;
- Cloudflare scope hints;
- Tauri five-place consistency.

## Final report

Include reproduction, root cause, fix, regression test, commands/results, risk, and rollback.
