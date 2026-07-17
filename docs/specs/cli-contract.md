# CLI Contract

## Binary

`flaredeck`

## Output modes

- `human`: concise text and tables for developers;
- `json`: stable response envelope with no ANSI sequences.

Data goes to stdout. Diagnostics and debug logs go to stderr.

## Response envelope

```json
{
  "ok": true,
  "data": {},
  "warnings": [],
  "error": null,
  "meta": {
    "schemaVersion": "1",
    "correlationId": "corr_123",
    "timestamp": "2026-07-16T10:00:00Z"
  }
}
```

## Command reference

### `workspace discover [path]`

Discovers `.flaredeck/project.yaml`, validates safe paths, and returns a workspace candidate. It does not approve trust or start a process.

### `workspace list`

Returns registered workspaces with validation, trust, and active-session summaries.

### `workspace inspect <workspace>`

Returns a safe manifest projection, profile binding, fingerprint, trust status, and validation errors.

### `workspace validate <workspace>`

Re-runs schema and policy validation.

### `workspace trust-status <workspace>`

Reports active approval, fingerprint mismatch, revocation, or absence. It never creates approval.

### `workspace revoke-trust <workspace>`

Human CLI operation only. Requires an interactive confirmation unless `--yes` is explicitly supplied by a human-operated terminal policy. It must not be exposed through MCP.

### `session start <workspace>`

Starts or returns the active session according to idempotency policy.

Options should be narrow:

- `--wait` or `--no-wait` only if both semantics are defined;
- `--timeout` may reduce the manifest maximum but may not widen it;
- no command, path, environment, route, token, or profile override.

### `session status <workspace-or-session>`

Returns state, ownership, health, public URLs, safe process summary, and timestamps.

### `session stop <workspace-or-session>`

Stops session-owned resources and performs cleanup. Repeated calls succeed with an idempotent result.

### `session logs <workspace-or-session>`

Options:

- `--source runtime|tunnel|system|all`;
- `--tail <n>` bounded by policy;
- `--since <timestamp>`;
- `--follow` for human use, not required by MCP.

### `route list <workspace-or-session>`

Returns persistent and temporary route status with ownership.

### `health check <workspace-or-session>`

Runs bounded checks and returns observations.

### `doctor`

Checks:

- FlareDeck data directory;
- `cloudflared` discovery and version;
- profile index readability;
- keychain or fallback availability without reading token values;
- workspace-state readability;
- supported platform details;
- CLI and schema version.

## Safe session-start example

```json
{
  "ok": true,
  "data": {
    "sessionId": "ses_01J...",
    "workspaceId": "ws_01J...",
    "state": "healthy",
    "runtime": {
      "state": "running",
      "origin": "http://127.0.0.1:5173"
    },
    "tunnel": {
      "state": "running",
      "startedBySession": false
    },
    "publicUrls": ["https://fluxbill-dev.ocloud.pro"],
    "health": { "state": "healthy" }
  },
  "warnings": [],
  "error": null,
  "meta": {
    "schemaVersion": "1",
    "correlationId": "corr_01J...",
    "timestamp": "2026-07-16T10:00:00Z"
  }
}
```

## Output restrictions

Never return:

- token values;
- tunnel credential content;
- `.env` values;
- complete inherited environment;
- raw Authorization, Cookie, or secret headers;
- unrestricted home-directory paths in MCP-oriented output;
- unbounded logs;
- stack traces by default.

## Exit-code categories

- `0` success;
- `2` usage error;
- `10` validation;
- `11` trust or capability;
- `12` state conflict;
- `20` runtime;
- `21` readiness;
- `30` tunnel, route, or DNS;
- `40` persistence or audit;
- `50` internal.

## Compatibility

- adding optional fields is allowed within schema version 1;
- renaming or changing meaning requires a new schema version;
- error codes are stable after Phase 3 acceptance;
- human wording may improve without changing machine semantics.
