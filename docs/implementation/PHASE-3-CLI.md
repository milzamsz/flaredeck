# Phase 3 headless CLI evidence

`flaredeck-cli` constructs no Tauri application or window. It calls the same profile, tunnel, workspace, session, health, route, and log services as desktop/MCP adapters. `--output json` emits the versioned envelope documented in `docs/specs/cli-contract.md`, including stable error code, warnings, correlation ID, timestamp, and categorized exit code. JSON output contains no ANSI escapes; stdout carries data and stderr carries human diagnostics.

Safe commands include version, profile list, observational tunnel status, doctor, workspace list/discover/inspect/validate/trust status, session start/status/stop/logs, route list, and health check. There is no trust approval, arbitrary command, route/profile/environment override, token read, or unrestricted file operation.

The development binary remains named `flaredeck-cli` because the Tauri desktop target already owns `flaredeck`; Phase 9 decides companion packaging without renaming the desktop binary or changing updater identity.

Evidence: `tests/cli_contract.rs` verifies the doctor JSON envelope and stdout discipline, stable usage exit/code, canary non-disclosure, and observational tunnel error behavior. `bash scripts/verify.sh` passes.
