# Technical Specification: FlareDeck AI Development Integration

## 1. Purpose

This document translates the product, domain, and architecture decisions into implementable contracts. It defines repository boundaries, data formats, commands, error codes, process behavior, testing, and compatibility requirements.

## 2. Implementation principles

1. Reuse application services across Tauri, CLI, and MCP.
2. Preserve current profile, tunnel, secret, YAML, DNS, WSL, and release behavior.
3. Add behavior incrementally and keep each phase reviewable.
4. Prefer explicit typed structures over maps and opaque JSON.
5. Do not execute an untrusted or caller-supplied command.
6. Keep secret values inside the existing secret subsystem.
7. Make start, stop, route verification, and cleanup idempotent.
8. Return stable machine-readable errors.
9. Test domain behavior without Tauri or a real Cloudflare account.
10. Avoid new dependencies unless they remove material implementation risk.

## 3. Proposed Rust module layout

Initial implementation within `src-tauri`:

```text
src-tauri/src/
├── application/
│   ├── mod.rs
│   ├── context.rs
│   ├── workspace_service.rs
│   ├── session_service.rs
│   ├── health_service.rs
│   └── audit_service.rs
├── domain/
│   ├── mod.rs
│   ├── ids.rs
│   ├── workspace.rs
│   ├── manifest.rs
│   ├── trust.rs
│   ├── runtime.rs
│   ├── session.rs
│   ├── route.rs
│   ├── health.rs
│   └── audit.rs
├── ports/
│   ├── mod.rs
│   ├── profile_repository.rs
│   ├── workspace_repository.rs
│   ├── trust_repository.rs
│   ├── session_repository.rs
│   ├── runtime_supervisor.rs
│   ├── tunnel_supervisor.rs
│   ├── route_service.rs
│   ├── probe_service.rs
│   └── audit_repository.rs
├── adapters/
│   ├── filesystem/
│   ├── process/
│   ├── cloudflare/
│   ├── cloudflared/
│   └── probes/
├── interfaces/
│   ├── tauri/
│   ├── cli/
│   └── mcp/
├── bin/
│   ├── flaredeck.rs
│   └── flaredeck-mcp.rs
├── cf_api.rs
├── cloudflared.rs
├── secrets.rs
├── state.rs
├── error.rs
├── types.rs
└── lib.rs
```

Existing files may remain in place while adapters are extracted. Avoid a large rename-only change mixed with behavior changes.

## 4. Application context

Every operation receives an immutable context:

```rust
pub struct OperationContext {
    pub actor: Actor,
    pub correlation_id: CorrelationId,
    pub requested_at: DateTime<Utc>,
    pub output_policy: OutputPolicy,
}
```

`Actor` includes safe origin information:

```rust
pub enum ActorKind {
    DesktopUser,
    CliUser,
    McpClient,
    SystemRecovery,
    TestHarness,
}
```

Do not treat actor metadata as authentication. Authorization is enforced through trust and capability policy.

## 5. Workspace manifest

Canonical location:

```text
<repository-root>/.flaredeck/project.yaml
```

The schema is defined in `docs/specs/workspace.schema.json` and explained in `docs/specs/workspace-manifest.md`.

Recommended command representation:

```yaml
runtime:
  executable: bun
  args: [run, dev]
```

Avoid:

```yaml
runtime:
  command: "bun run dev && curl ..."
```

Shell mode is not part of the MVP.

## 6. Canonical path policy

1. Resolve the workspace root with platform-aware canonicalization.
2. Resolve the manifest path beneath that root.
3. Resolve the working directory relative to the root.
4. Reject traversal outside the root.
5. Handle Windows drive-letter normalization and UNC paths.
6. Preserve WSL-specific origin logic but do not confuse WSL paths with Windows process paths.
7. Do not accept a workspace root directly from MCP unless it is already registered or discovered from the current approved repository context.

## 7. Trust fingerprint algorithm

The exact hash algorithm may use an existing approved cryptographic dependency. SHA-256 is suitable and already conceptually consistent with the project’s dependencies.

Procedure:

1. Validate manifest.
2. Convert security-relevant fields to a canonical structure.
3. Normalize paths, hostnames, URLs, executable, arguments, and environment names.
4. Serialize with stable field order.
5. Include a fingerprint format version.
6. Hash bytes.
7. Encode as lowercase hex with prefix, for example `fdtrust:v1:<digest>`.

Test vectors must prove:

- formatting-only YAML changes do not change the fingerprint;
- command argument changes do change it;
- working-directory changes do change it;
- route changes do change it;
- display-label changes do not change it;
- schema-version changes do change it.

## 8. Runtime process spawn

Use direct process execution:

```rust
Command::new(&runtime.executable)
    .args(&runtime.args)
    .current_dir(&runtime.working_directory)
```

Requirements:

- no shell interpolation;
- clear or tightly control inherited environment;
- set only declared safe literals and approved passthrough names;
- capture stdout and stderr;
- create an appropriate process group or Windows job/process tree strategy;
- track ownership by session;
- cap buffered logs;
- terminate the process tree on stop;
- wait for exit and record code;
- avoid blocking Tauri’s async runtime.

## 9. Runtime environment policy

Manifest may declare:

```yaml
environment:
  passthrough:
    - NODE_ENV
    - RUST_LOG
  values:
    APP_ENV: development
```

Rules:

- names are validated;
- values in the manifest are non-secret literals;
- sensitive-looking keys may be rejected or require explicit policy;
- passthrough reports only present/missing state, never values;
- automatic loading of `.env` by the child framework is permitted because FlareDeck does not read or return it;
- FlareDeck must not parse `.env` files in the MVP.

## 10. Readiness probes

### TCP

```yaml
ready:
  type: tcp
  host: 127.0.0.1
  port: 5173
  timeoutSeconds: 60
  intervalMilliseconds: 500
```

### HTTP

```yaml
ready:
  type: http
  url: http://127.0.0.1:5173/health
  expectedStatus: [200, 299]
  timeoutSeconds: 60
  intervalMilliseconds: 500
```

Technical controls:

- local targets by default;
- no redirects to unapproved external hosts;
- response body ignored or capped;
- per-attempt timeout;
- cancellation when the session stops;
- observations stored with latency and safe error summaries.

## 11. Session start algorithm

Pseudocode:

```text
start_session(workspace):
  acquire workspace session lock
  return existing active session if idempotency policy allows
  resolve workspace
  validate manifest
  verify trust fingerprint and required capabilities
  create session record in Created state
  audit start requested

  start runtime
  mark runtime ownership
  wait for readiness

  inspect tunnel status
  start tunnel only if not already running
  record tunnel ownership

  verify or create routes allowed by policy
  calculate public URLs
  run initial health aggregation

  mark Healthy or Degraded
  persist session
  audit success
  return safe result

on failure:
  mark Failed
  compensate only resources created by this attempt
  persist cleanup outcome
  audit failure
  return structured error
```

## 12. Session stop algorithm

1. Resolve session.
2. If already stopped, return successful idempotent result.
3. Mark `Stopping`.
4. Cancel readiness and health tasks.
5. Stop runtime process owned by the session.
6. Remove temporary routes owned by the session when policy allows.
7. Stop tunnel only if the session started it and no other active consumer requires it.
8. Mark cleanup result.
9. Persist and audit.

## 13. CLI contract

Binary name:

```text
flaredeck
```

During Phase 3 development the headless binary is `flaredeck-cli`; the desktop
bundle already owns `flaredeck`. Renaming or packaging the desktop binary is a
release compatibility decision deferred to Phase 9. The CLI contract name
remains `flaredeck` for the eventual released artifact.

Global options:

```text
--output human|json
--correlation-id <id>
--no-color
--config-dir <path>   # local human use only; not exposed by MCP
```

Suggested commands:

```text
flaredeck workspace discover [path]
flaredeck workspace list
flaredeck workspace inspect <workspace>
flaredeck workspace validate <workspace>
flaredeck workspace trust-status <workspace>
flaredeck workspace revoke-trust <workspace>

flaredeck session start <workspace>
flaredeck session status <workspace-or-session>
flaredeck session stop <workspace-or-session>
flaredeck session logs <workspace-or-session>

flaredeck route list <workspace-or-session>
flaredeck health check <workspace-or-session>
flaredeck doctor
```

Trust approval should initially remain a desktop-only action. A CLI approval command may be added only with explicit interactive confirmation and an ADR.

Detailed output is defined in `docs/specs/cli-contract.md`.

## 14. Standard response envelope

```json
{
  "ok": true,
  "data": {},
  "warnings": [],
  "error": null,
  "meta": {
    "schemaVersion": "1",
    "correlationId": "corr_...",
    "timestamp": "2026-07-16T10:00:00Z"
  }
}
```

Error form:

```json
{
  "ok": false,
  "data": null,
  "warnings": [],
  "error": {
    "code": "WORKSPACE_NOT_TRUSTED",
    "message": "The workspace configuration requires local approval.",
    "retryable": false,
    "details": {
      "workspaceId": "ws_...",
      "requiredAction": "open_flaredeck_and_review_workspace"
    }
  },
  "meta": {
    "schemaVersion": "1",
    "correlationId": "corr_...",
    "timestamp": "2026-07-16T10:00:00Z"
  }
}
```

`details` must be safe for an AI client.

## 15. Error codes

### Workspace and trust

- `WORKSPACE_NOT_FOUND`
- `WORKSPACE_MANIFEST_NOT_FOUND`
- `WORKSPACE_MANIFEST_INVALID`
- `WORKSPACE_PATH_OUTSIDE_ROOT`
- `WORKSPACE_NOT_TRUSTED`
- `WORKSPACE_TRUST_CHANGED`
- `WORKSPACE_CAPABILITY_DENIED`
- `PROFILE_NOT_FOUND`
- `PROFILE_NOT_READY`

### Session and runtime

- `SESSION_ALREADY_RUNNING`
- `SESSION_NOT_FOUND`
- `SESSION_STATE_CONFLICT`
- `RUNTIME_START_FAILED`
- `RUNTIME_NOT_READY`
- `RUNTIME_CRASHLOOP_PAUSED`
- `RUNTIME_STOP_FAILED`

### Tunnel and routes

- `TUNNEL_START_FAILED`
- `TUNNEL_STATUS_UNKNOWN`
- `TUNNEL_STOP_FAILED`
- `ROUTE_VALIDATION_FAILED`
- `ROUTE_CREATION_FAILED`
- `DNS_VERIFICATION_FAILED`

### Persistence and protocol

- `STATE_READ_FAILED`
- `STATE_WRITE_FAILED`
- `AUDIT_WRITE_FAILED`
- `OUTPUT_SERIALIZATION_FAILED`
- `MCP_PROTOCOL_ERROR`
- `INTERNAL_INVARIANT_VIOLATION`

## 16. Exit codes

Suggested mapping:

- `0`: success;
- `2`: invalid CLI usage;
- `10`: validation failure;
- `11`: trust or capability denied;
- `12`: conflict or already running;
- `20`: runtime failure;
- `21`: readiness failure;
- `30`: tunnel or route failure;
- `40`: persistence failure;
- `50`: internal failure.

Exact values become stable after Phase 3 acceptance.

## 17. MCP tool surface

MVP tools:

- `workspace_list`
- `workspace_status`
- `session_start`
- `session_status`
- `session_stop`
- `public_url_get`
- `logs_read`
- `health_check`
- `doctor`

Nine tools are acceptable if each has a clear boundary. Do not split every status field into another tool.

No tool accepts:

- arbitrary shell command;
- arbitrary environment map;
- token or credential;
- unrestricted path;
- arbitrary Cloudflare API endpoint;
- raw DNS mutation parameters outside an approved workspace route.

Detailed schemas are defined in `docs/specs/mcp-tools.md`.

## 18. MCP transport requirements

- stdio only for MVP;
- newline-delimited JSON-RPC according to the supported MCP SDK;
- stdout reserved for protocol messages;
- stderr for logs;
- process exits non-zero on unrecoverable initialization failure;
- log level controlled by environment or CLI flag without emitting to stdout;
- protocol version negotiated through the SDK;
- tool results are bounded and redacted;
- cancellation propagates to long-running readiness and health checks where supported.

## 19. Tauri command integration

New Tauri commands must follow the existing five-place rule:

1. Rust handler.
2. Serde request/response type.
3. Registration in `lib.rs`.
4. TypeScript wrapper and type.
5. Zustand action or component caller.

Suggested command groups:

- `workspace_*`;
- `session_*`;
- `health_*`;
- `audit_*`.

Implemented desktop workspace commands are `workspace_inspect`, `workspace_approve`,
`workspace_list`, `workspace_session_start`, `workspace_session_status`,
`workspace_session_stop`, `workspace_session_logs`, and `workspace_audit`.
Only `workspace_approve` writes trust, and it is registered only by the desktop target.

Tauri handlers remain thin adapters and do not contain orchestration logic.

## 20. Frontend state

Recommended state shape:

```ts
interface WorkspaceState {
  workspaces: WorkspaceSummary[];
  activeWorkspaceId: string | null;
  validation: Record<string, WorkspaceValidation>;
  trust: Record<string, WorkspaceTrustStatus>;
}

interface SessionState {
  sessions: Record<string, DevelopmentSessionView>;
  logs: Record<string, SafeLogEntry[]>;
  health: Record<string, HealthSummary>;
}
```

Persist only safe preferences and identifiers. Do not persist tokens, environment values, log bodies beyond approved limits, or stale session ownership in browser storage.

## 21. Logging and redaction

Every structured log record should support:

- timestamp;
- level;
- component;
- operation;
- correlation ID;
- workspace ID;
- session ID;
- safe message;
- safe fields.

Redact:

- API tokens;
- Authorization and Cookie headers;
- tunnel credentials;
- known secret environment names and values;
- URLs with sensitive query parameters;
- filesystem paths when exposed to MCP unless required and approved;
- request bodies by default.

Return no more than a configured number of log lines and bytes per operation.

## 22. Persistence and migrations

Each new persisted document includes:

```json
{
  "schemaVersion": 1,
  "updatedAt": "...",
  "data": {}
}
```

Requirements:

- atomic write via temporary file and rename;
- backup last known valid version where appropriate;
- migration function per version;
- corrupted-state error with recovery guidance;
- lock strategy to prevent concurrent writers;
- tests for upgrade and rollback boundaries.

## 23. Tests required by phase

### Phase 1

- aggregate verification script;
- CI for frontend and Rust checks;
- documentation and JSON schema validation.

### Phase 2

- application service tests with fake ports;
- regression tests around existing tunnel and profile behavior.

### Phase 3

- CLI parser tests;
- JSON snapshots;
- stable exit code tests;
- no-secret output tests.

### Phase 4

- manifest validation;
- path escape rejection;
- trust fingerprint vectors;
- process start/stop and process-tree cleanup;
- readiness timeout and cancellation;
- session idempotency and compensation.

### Phase 5

- MCP initialize and tool discovery;
- tool input validation;
- stdout protocol discipline;
- cancellation;
- output bounds and redaction.

### Phase 6

- OpenCode and VS Code smoke tests;
- end-to-end fixture application;
- agent acceptance evidence workflow;
- cross-platform test matrix.

## 24. Verification commands

The project should converge on one aggregate script, for example:

```bash
bash scripts/verify.sh
```

The aggregate verifier runs the current frontend, Rust, and package checks:

```bash
bash scripts/verify.sh
```

Tests that need real Cloudflare credentials must be opt-in and excluded from standard CI.

## 25. Compatibility rules

- Existing profile files remain readable.
- Existing secret references remain valid.
- Existing route YAML preserves catch-all semantics.
- Existing WSL origin rewriting remains intact.
- Existing Tauri frontend behavior remains functional during extraction.
- A new interface may not require wider Cloudflare API scopes without an explicit product and ADR update.
- CLI and MCP schema versions are reported and versioned.

## 26. Definition of done for non-trivial implementation tasks

A task is complete only when:

- requirements and non-goals are met;
- domain and architectural boundaries are respected;
- tests cover normal, error, and cleanup behavior;
- no secret appears in output or fixtures;
- relevant docs and schemas are updated;
- lint, build, and Rust checks pass;
- manual checks are documented when automation is unavailable;
- rollback or compatibility impact is stated;
- reviewer findings are resolved or explicitly accepted.

## 27. Phase 8 temporary routes and webhook capture

- `temporary_route_service` owns exact DNS/ingress/proxy identity, one-hour expiry, reverse compensation, idempotent cleanup, and expired-route reconciliation in application data.
- `flaredeck-webhook-proxy` is an adjacent loopback-only companion. It accepts HTTP/1 requests with at most 16 KiB of headers and 64 KiB of body, forwards only to the manifest-approved loopback origin, and retains at most 100 redacted events per route.
- route state and captures use schema version 1 JSON written through temporary-file rename. They contain no token, tunnel credential, raw environment value, or unredacted known secret field.
- desktop commands return safe route/event projections. Replay accepts only route/event IDs, requires an active unexpired owned session route, and resolves the stored loopback origin internally.
- CLI route output distinguishes persistent and temporary ownership. MCP adds only `webhook_event_list` and `webhook_event_get`; replay and mutation are absent.
- the existing Cloudflare DNS Edit token requirement is unchanged. DNS read/delete failures use named operation hints; no wider scope is introduced.

## 28. Phase 9 release compatibility

Release builds merge `src-tauri/tauri.release.conf.json` into the unchanged base Tauri configuration. `scripts/prepare-sidecars.mjs` produces target-triple CLI/MCP/proxy external binaries; macOS universal binaries are formed from separately compiled x86_64 and arm64 outputs. `scripts/smoke-sidecars.mjs` verifies exact versions and MCP discovery before packaging. `scripts/verify-release.mjs` fails version, updater identity/key/endpoint, companion list, schema contract, or stable-download-name drift.

The GitHub release remains a draft until all platform jobs pass. Only then are updater manifests merged, SHA-256 checksums uploaded, the hosted manifest updated, and the draft published. Tauri's existing updater private-key secrets sign the complete desktop/sidecar update unit; private keys never enter repository files or logs. See `docs/implementation/RELEASE-HARDENING.md` for install paths, migration, rollback, artifact evidence, and the release decision.
