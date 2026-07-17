# ADR-006: Reuse Persistent Profile Tunnels for Temporary Task Routes

- Status: Accepted for Phase 8 implementation

## Context

Task-specific public hostnames are useful for integration testing. Creating a named tunnel and credentials for every task increases Cloudflare mutations, orphan risk, cleanup complexity, and secret lifecycle burden.

## Decision

Reuse the workspace-selected persistent profile tunnel. Temporary task exposure, when implemented, creates owned ingress and DNS route entries with expiration and cleanup metadata.

## Consequences

- fewer tunnel credentials;
- simpler cleanup;
- one tunnel may serve multiple approved routes;
- route ownership and conflict detection become mandatory;
- persistent routes must be protected from session cleanup.
- exact DNS/ingress conflict checks happen before mutation;
- ownership and expiry live in application data;
- webhook capture/replay follow the separate Phase 8 threat model and data policy.

## Rejected alternatives

- one tunnel per task;
- random quick tunnels as the default;
- deleting all workspace routes on stop.
