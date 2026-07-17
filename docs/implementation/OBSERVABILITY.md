# Observability and Diagnostics

## Objectives

- explain session startup and failure stages;
- correlate desktop, CLI, MCP, runtime, tunnel, route, and health events;
- support debugging without exposing secrets;
- bound memory, disk, and model context usage.

## Correlation

Generate one correlation ID per top-level operation. A session has a stable session ID and may have many operation correlation IDs.

Propagate correlation metadata through:

- application service calls;
- process events;
- health checks;
- Cloudflare operations;
- audit events;
- returned CLI/MCP results.

## Log sources

- `system`: application and orchestration;
- `runtime`: development child process;
- `tunnel`: `cloudflared` child process;
- `health`: readiness and route observations;
- `audit`: immutable safe action summaries.

## Bounds

- in-memory ring buffers per process;
- maximum line length;
- maximum returned entries and bytes;
- rotating local audit files;
- no unbounded MCP streams in MVP.

## Doctor command

Report:

- versions;
- platform;
- data-directory accessibility;
- `cloudflared` discovery;
- secure-store availability;
- profile/workspace state readability;
- orphan or recovery-required sessions;
- schema compatibility;
- safe remediation guidance.
