# Example Agentic Kanban Task

## ID

FD-AI-4.7

## Title

Implement trusted workspace session start orchestration

## Parent phase

Phase 4: Workspace, Trust, Runtime, and Session Orchestration

## Outcome

A validated and trusted workspace can start one development session through `SessionApplicationService`, reaching runtime readiness and safely observing or starting the selected profile tunnel.

## Dependencies

- workspace manifest parser accepted;
- workspace path validation accepted;
- trust repository and fingerprint accepted;
- runtime supervisor accepted;
- existing tunnel supervisor exposed through a port;
- audit service available.

## Scope

- create session record and workspace lock;
- validate trust and capabilities;
- start approved runtime;
- wait for readiness;
- inspect/start selected tunnel;
- record ownership;
- return session state and safe public-route placeholders;
- compensate runtime/tunnel resources on failure;
- emit audit events.

## Exclusions

- MCP adapter;
- temporary route creation;
- webhook inspector;
- polished desktop session UI;
- multiple concurrent sessions per workspace;
- arbitrary runtime overrides.

## Acceptance criteria

1. Untrusted workspace returns `WORKSPACE_NOT_TRUSTED` before process spawn.
2. Trusted fixture workspace starts one runtime.
3. Readiness timeout stops the owned runtime.
4. Pre-existing tunnel is observed and not marked as session-owned.
5. Tunnel started by the session is marked as owned.
6. Repeated start returns the active session or a documented conflict without spawning another runtime.
7. Failure emits safe audit events.
8. Canary secret does not appear in returned errors or logs.
9. Unit/application tests pass without real Cloudflare credentials.

## Likely files

- `src-tauri/src/application/session_service.rs`
- `src-tauri/src/domain/session.rs`
- `src-tauri/src/ports/runtime_supervisor.rs`
- `src-tauri/src/ports/tunnel_supervisor.rs`
- relevant fake adapters and tests

## Verification

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml session
```

## Evidence

- state transition test results;
- process-spawn count proving idempotency;
- ownership assertions;
- failure compensation assertions;
- canary-secret assertion;
- diff summary and rollback note.
