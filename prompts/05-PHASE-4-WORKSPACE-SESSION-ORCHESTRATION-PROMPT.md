# Phase 4 Prompt: Workspace, Trust, Runtime, and Session Orchestration

## Objective

Implement the core local development exposure lifecycle.

## Workstreams

### A. Manifest and workspace

- implement `.flaredeck/project.yaml` discovery;
- validate against schema and security policy;
- canonicalize workspace and working-directory paths;
- resolve selected profile;
- return safe workspace views.

### B. Trust

- implement canonical security projection and fingerprint;
- add test vectors;
- persist local approval separately from repository;
- invalidate approval when relevant fields change;
- add a minimal human desktop approval flow;
- prohibit MCP approval.

### C. Runtime supervisor

- spawn executable and args directly;
- restrict environment inheritance;
- capture bounded logs;
- implement platform-aware process-tree termination;
- add crashloop and cancellation;
- test fixture processes.

### D. Readiness and health

- implement local TCP and HTTP probes;
- enforce timeout, interval, response, redirect, and target limits;
- aggregate safe observations.

### E. Session orchestrator

- implement the documented state machine;
- enforce one active session per workspace;
- track runtime/tunnel/route ownership;
- implement compensation and idempotent cleanup;
- persist recoverable metadata;
- emit audit events;
- expose CLI commands.

## Mandatory security tests

- untrusted start denied;
- command change invalidates trust;
- path traversal and symlink policy;
- shell metacharacters not interpreted;
- environment values not returned;
- readiness timeout cleans owned runtime;
- pre-existing tunnel remains running;
- repeated stop is safe;
- corrupt trust store fails closed;
- canary secret absent from every output.

## Prohibited shortcuts

- hidden auto-trust;
- `bash -c` as generic escape hatch;
- route or profile override supplied by CLI caller;
- automatic deletion of persistent routes;
- killing uncertain recovered PIDs;
- implementing MCP before this phase is accepted.

## Exit criteria

A human can register and approve a workspace, then use the CLI to start, inspect, and stop a complete development session safely.
