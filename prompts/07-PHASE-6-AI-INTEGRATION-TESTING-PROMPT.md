# Phase 6 Prompt: AI Client Integration and Acceptance Evidence

## Objective

Prove that FlareDeck works as a controlled local AI-development tool in OpenCode and VS Code.

## Required work

1. Build/install compatible desktop, CLI, and MCP binaries.
2. Create or use a non-secret fixture workspace manifest.
3. Verify desktop trust approval and invalidation.
4. Configure OpenCode from `examples/opencode.jsonc` using current official syntax.
5. Configure VS Code from `examples/.vscode/mcp.json` using current official syntax.
6. Verify client trust prompts and tool discovery.
7. Enable mutation tools only for an integration-testing agent where supported.
8. Execute a complete scenario:
   - workspace status;
   - session start;
   - public URL retrieval;
   - test request or browser flow;
   - health and logs;
   - session stop;
   - cleanup verification.
9. Test that prompt injection cannot obtain trust or secrets.
10. Update examples, skills, and prompts to match real commands.
11. Produce cross-platform smoke-test results or explicit gaps.

## Evidence format

- client and version;
- FlareDeck version;
- workspace and session IDs;
- correlation IDs;
- tools called;
- safe results;
- application tests;
- cleanup result;
- no-secret verification;
- deviations from docs.

## Exit criteria

The workflow is reproducible by another developer using repository instructions, with no secret in client configuration and no manual tunnel/DNS shell work.
