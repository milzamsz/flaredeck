# ADR-002: Introduce Workspace and Development Session Above Profile

- Status: Accepted

## Context

A profile models Cloudflare infrastructure, not a repository, development command, readiness rule, or bounded integration-test activity. Reusing `Profile` for these concerns would couple project execution to Cloudflare credentials and produce ambiguous lifecycle ownership.

## Decision

Introduce:

- `Workspace`: a validated repository plus non-secret manifest and trust state;
- `DevelopmentSession`: a bounded runtime/tunnel/route/health activity for one workspace.

A workspace selects an existing profile. A session coordinates the workspace runtime with the selected profile tunnel.

## Consequences

- profile remains backward compatible;
- one workspace can create many historical sessions;
- session cleanup can track resource ownership;
- trust applies to executable behavior, not Cloudflare credentials;
- additional local state repositories are required.

## Rejected alternatives

- add command and repository fields directly to `Profile`;
- make every route a workspace;
- use only stateless CLI commands with no session model.
