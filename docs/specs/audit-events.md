# Audit Event Specification

## Event envelope

```json
{
  "schemaVersion": 1,
  "eventId": "evt_01J...",
  "timestamp": "2026-07-16T10:00:00Z",
  "actor": {
    "kind": "mcp_client",
    "name": "opencode"
  },
  "operation": "session.start",
  "result": "success",
  "workspaceId": "ws_01J...",
  "sessionId": "ses_01J...",
  "profileId": "profile-uuid",
  "correlationId": "corr_01J...",
  "errorCode": null,
  "metadata": {
    "tunnelStartedBySession": false,
    "publicUrlCount": 1
  },
  "redactionVersion": 1
}
```

## Required operation families

- workspace discovered, registered, validation failed;
- trust approved, revoked, invalidated;
- session start requested, started, failed;
- runtime started, exited, crashloop paused, stopped;
- tunnel observed, started, failed, stopped;
- route verified, created, failed, removed;
- health check completed;
- session stop requested, stopped, cleanup incomplete;
- state migration and recovery actions;
- MCP server startup and unrecoverable protocol failure without logging request content.

## Security requirements

Audit metadata must not contain:

- API tokens or credential content;
- Authorization/Cookie headers;
- environment values;
- full webhook bodies;
- unrestricted command strings outside the approved safe runtime summary;
- raw stack traces;
- sensitive URL query values.

## Storage

Use append-oriented, rotating local storage with atomicity appropriate to the selected format. Bound retention by age and size. Audit is local product evidence, not enterprise compliance logging.

## Query behavior

Desktop and CLI may query by workspace, session, operation, result, time range, and correlation ID. Results are paginated or bounded.
