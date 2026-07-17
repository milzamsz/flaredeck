# Phase 8 Prompt: Temporary Routes and Webhook Inspector

## Objective

Add task-specific route lifecycle and safe webhook inspection as a separate bounded context.

## Mandatory prerequisite

Before implementation, create or approve:

- webhook product scope;
- separate threat-model update;
- data-retention and redaction policy;
- request replay policy;
- route ownership and conflict design.

## Required work

1. Implement temporary routes on an existing profile tunnel.
2. Store creation ownership, expiration, and cleanup state.
3. Add reconciliation for expired or orphaned routes.
4. Implement a bounded local capture proxy.
5. Redact headers and configured payload fields before storage.
6. Limit body size, event count, retention, and content types.
7. Add request replay only with explicit approval and restricted targets.
8. Expose read-only MCP event tools by default.
9. Keep persistent routes protected.

## Prohibited work

- one tunnel per task by default;
- capture all traffic without route opt-in;
- store unredacted credentials;
- arbitrary replay target;
- unlimited body storage;
- presenting the feature as a production gateway.

## Exit criteria

Temporary exposure is ownership-safe and webhook data is bounded, redacted, inspectable, and cleanly removable.
