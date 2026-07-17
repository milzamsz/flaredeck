# Local AI Development Integration Guide

## Recommended flow

1. Install FlareDeck desktop with its version-matched CLI and MCP companions.
2. Add `.flaredeck/project.yaml` to the application repository.
3. Open FlareDeck and register/review the workspace.
4. Approve the current fingerprint.
5. Configure the local MCP client to launch `flaredeck-mcp` over stdio.
6. Enable mutation tools only for an integration-testing agent.
7. Use the task prompt and Agent Skills from this package.
8. Require start, test, logs/health, and stop evidence.

## OpenCode

Use `examples/opencode.jsonc` as a starting point. Project configuration contains no Cloudflare token. Current OpenCode uses `permission` rules; the example denies `flaredeck_*` globally and allows them only for the dedicated `integration-test` agent.

## VS Code

Use `examples/.vscode/mcp.json`. VS Code asks the user to trust local MCP servers and can keep workspace configuration in source control. Review any client-side auto-approval separately; FlareDeck still enforces workspace trust.

## Other clients

Use the client’s current official documentation for a local stdio server. Configure:

- command: the installed `flaredeck-mcp` companion path (absolute when the installer directory is not on `PATH`);
- arguments: usually none or `--stdio` if retained by implementation;
- working directory: repository root when supported;
- no secret environment variables required for FlareDeck itself.

Do not commit unverified client syntax to the repository merely because another tool used a similar JSON shape.

## Reproducible smoke check

Run `bash scripts/ai-integration-smoke.sh`. It builds compatible CLI/MCP binaries, runs their binary-level contract suites, validates CLI JSON, performs MCP initialization/tool discovery, parses the VS Code configuration, and—when OpenCode is installed—starts the server through the checked-in OpenCode configuration.

`examples/fixture-workspace` is a non-secret loopback HTTP fixture. Its manifest disables tunnel startup so standard smoke tests never mutate Cloudflare state. For a live exposure test, copy the fixture, select an existing profile, configure a persistent route already owned by that profile, enable tunnel startup, review the resulting behavior in the desktop trust screen, and approve the new fingerprint.

## Agent workflow contract

An agent should:

- inspect workspace status first;
- stop and report when approval is required;
- start one session;
- wait for healthy or return structured failure;
- use public URLs only for the active task;
- inspect bounded logs and health;
- stop the session;
- include session and correlation IDs in evidence;
- never ask for token values.

## Phase 6 acceptance evidence (2026-07-17)

- Platform: Linux x86_64; FlareDeck `0.2.3` development binaries.
- OpenCode: `1.18.1`; `opencode mcp list --pure` loaded `examples/opencode.jsonc`, launched `flaredeck-mcp`, and reported `flaredeck connected`.
- VS Code: `1.129.0`; `examples/.vscode/mcp.json` matches the current stdio configuration reference and parses as JSON. The first-run MCP trust dialog was not clicked in this headless run; it remains an intentional human gate.
- Protocol client: MCP `2025-11-25`; discovered all eleven approved tools, including read-only webhook event list/get, and completed workspace status, session start, public URL, health, redacted logs, session stop, and cleanup using a generated non-secret fixture. Session/correlation IDs are generated per run and asserted structurally rather than committed.
- Security: a changed executable invalidated trust; an unknown `get_token` prompt requesting approval and a synthetic canary returned only `INVALID_REQUEST`; canary content was absent; arbitrary paths/properties were rejected; MCP never writes child logs to protocol stdout.
- Cleanup: external fixture runtime was never owned or killed; repeated session stop returned `stopped`; cancellation returned `CANCELLED` without leaving an owned runtime.
- Cross-platform gaps: macOS, Windows, and WSL client UI smoke runs were unavailable on this Linux host. No support claim is inferred from Linux evidence; Phase 9 CI/package jobs must supply those results.

Official syntax checked against [OpenCode local MCP configuration](https://opencode.ai/docs/mcp-servers/), [OpenCode agent permissions](https://opencode.ai/docs/agents/), and the [VS Code MCP configuration reference](https://code.visualstudio.com/docs/agents/reference/mcp-configuration).
