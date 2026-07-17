# MCP Tool Specification

## 1. Transport and process contract

- executable: `flaredeck-mcp`;
- transport: stdio;
- stdout: MCP protocol only;
- stderr: diagnostics;
- no network listener in the MVP;
- tools call shared application services;
- all results are bounded and redacted.

## 2. Tool design rules

- protocol tool names are concise (`workspace_list`, `session_start`, and similar); MCP clients may namespace them with the configured server name. For example, OpenCode exposes tools from a server named `flaredeck` as `flaredeck_workspace_list`, `flaredeck_session_start`, and similar;
- input schemas reject unnecessary properties;
- workspace selectors use registered IDs or unique names, not arbitrary filesystem paths;
- no tool approves trust;
- no tool accepts arbitrary command, environment, secret, route, or Cloudflare endpoint input;
- mutation tools return ownership and cleanup information;
- errors include stable FlareDeck error codes in structured content.

## 3. Approved MVP tools

The raw MCP protocol names below intentionally omit the server prefix. Configure the server name as `flaredeck` in clients that namespace tools.

## `workspace_list`

Purpose: list registered workspaces and their safe status.

Input:

```json
{
  "type": "object",
  "properties": {
    "state": {
      "type": "string",
      "enum": ["all", "trusted", "approval_required", "running", "invalid"]
    },
    "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
  },
  "additionalProperties": false
}
```

Output fields:

- workspace ID and name;
- trust and validation state;
- selected profile summary;
- active session summary;
- safe path display according to MCP output policy.

## `workspace_status`

Purpose: inspect one registered workspace.

Input:

```json
{
  "type": "object",
  "required": ["workspace"],
  "properties": {
    "workspace": { "type": "string", "minLength": 1, "maxLength": 100 }
  },
  "additionalProperties": false
}
```

Output:

- manifest version;
- safe runtime summary;
- route summaries;
- readiness summary;
- trust state and required human action;
- active session state.

## `session_start`

Purpose: start a trusted workspace session.

Input:

```json
{
  "type": "object",
  "required": ["workspace"],
  "properties": {
    "workspace": { "type": "string", "minLength": 1, "maxLength": 100 },
    "waitForHealthy": { "type": "boolean", "default": true }
  },
  "additionalProperties": false
}
```

Controls:

- denies untrusted or changed workspace;
- cannot override runtime, profile, route, environment, or timeout above policy;
- returns existing active session when idempotency rules allow.

## `session_status`

Input selector: session ID or workspace ID/name.

Output:

- session state;
- stage statuses;
- runtime and tunnel ownership;
- public URLs;
- health;
- timestamps;
- warnings;
- correlation ID.

## `session_stop`

Purpose: stop and clean session-owned resources.

Input:

```json
{
  "type": "object",
  "required": ["session"],
  "properties": {
    "session": { "type": "string", "minLength": 1, "maxLength": 100 }
  },
  "additionalProperties": false
}
```

No force flag is exposed in MVP. Cleanup wider than normal ownership requires a human desktop action.

## `public_url_get`

Purpose: retrieve public URLs for an active session without returning unrelated configuration.

Input: session or workspace selector.

Output:

- URL;
- hostname;
- route health;
- route ownership type;
- optional expiration.

## `logs_read`

Input:

```json
{
  "type": "object",
  "required": ["session"],
  "properties": {
    "session": { "type": "string", "minLength": 1, "maxLength": 100 },
    "source": { "type": "string", "enum": ["all", "runtime", "tunnel", "system"] },
    "tail": { "type": "integer", "minimum": 1, "maximum": 200, "default": 50 }
  },
  "additionalProperties": false
}
```

Output:

- redacted log entries;
- truncation indicator;
- earliest and latest returned timestamps;
- correlation IDs when available.

No follow/stream mode is required for MVP MCP.

## `health_check`

Purpose: execute bounded checks for one workspace or session.

Output:

- aggregated state;
- individual runtime, readiness, tunnel, DNS, origin, and route observations;
- safe failure messages;
- latency and checked time.

## `doctor`

Purpose: read-only environment diagnostics.

Output excludes tokens and secret values. It may report whether secure storage is available and whether required configuration is missing.

## `webhook_event_list`

Purpose: read at most 100 pre-storage-redacted captures for an owned temporary route.

Input requires a route ID and accepts an optional `limit` from 1 through 100. The output includes safe request metadata, redacted headers/body, response status, body storage state, timestamp, and redaction version.

## `webhook_event_get`

Purpose: read one pre-storage-redacted capture by owned temporary route ID and event ID.

Both selectors are bounded strings and unknown properties are rejected. MCP deliberately has no capture clear, replay, target override, or raw-event tool. Replay remains an explicit per-event desktop action restricted to the manifest-approved original loopback origin.

## 4. Structured tool error

Tool results should include a machine-readable error object in the content or SDK-supported error structure:

```json
{
  "code": "WORKSPACE_NOT_TRUSTED",
  "message": "The workspace requires local approval in FlareDeck.",
  "retryable": false,
  "requiredAction": "open_flaredeck_and_review_workspace",
  "correlationId": "corr_01J..."
}
```

## 5. Explicitly prohibited tools

- `flaredeck_run_shell_command`;
- `flaredeck_read_env`;
- `flaredeck_get_token`;
- `flaredeck_write_token`;
- unrestricted file read/write;
- arbitrary DNS mutation;
- arbitrary Cloudflare API request;
- arbitrary process kill;
- trust approval;
- webhook replay or capture mutation;
- permanent profile deletion.

## 6. Response limits

Suggested defaults:

- workspace list: 50;
- log entries: 50 default, 200 maximum;
- per log entry: bounded bytes;
- total tool result: bounded by an implementation policy;
- health response bodies: not returned;
- audit detail: not part of MVP MCP unless a later read-only tool is approved.
- webhook events: 50 default, 100 maximum per call and per route.

## 7. Client permission guidance

Clients should enable FlareDeck tools only for agents that perform integration testing or local exposure tasks. Ordinary implementation agents do not need session mutation tools by default.
