# ADR-004: Use Local MCP over stdio for the MVP

- Status: Accepted

## Context

The intended clients are local development tools. A network MCP server would require listener security, authentication, origin validation, lifecycle management, and a larger attack surface.

## Decision

Implement `flaredeck-mcp` as a local process using MCP stdio transport. The client launches the server. Protocol output uses stdout only; diagnostics use stderr.

## Consequences

- no local network listener;
- client lifecycle naturally controls the process;
- simpler installation and threat model;
- each client launches its own server process unless later coordination is introduced;
- remote agents are outside scope.

## Rejected alternatives

- unauthenticated localhost HTTP;
- public Streamable HTTP;
- exposing Tauri IPC directly to external clients.

## Change trigger

Remote MCP requires a separate ADR, authentication design, origin policy, session isolation, and updated threat model.
