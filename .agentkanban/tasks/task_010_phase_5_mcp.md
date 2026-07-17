---
title: Add local MCP stdio adapter
lane: done
created: 2026-07-17T00:00:00Z
updated: 2026-07-17T18:00:00+07:00
description: A flaredeck-mcp child process must expose only the approved bounded workspace/session tools over stdio through accepted shared services.
priority: critical
labels: [phase-5, mcp, security]
dependsOn: [task_009_phase_4_workspace_sessions]
evidence:
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
---

## Conversation

### user

### agent

Phase 5 implementation is ready for review. The current stable MCP protocol was checked against the official stdio, tools, and cancellation specifications. The adapter negotiates supported versions through `2025-11-25`, accepts omitted arguments for zero-input tools, handles concurrent cancellation notifications, caps MCP session startup at 120 seconds with cleanup, and re-redacts persisted logs at read time. The protocol test proves the complete registered workspace lifecycle plus cancellation, changed-fingerprint denial, arbitrary path/property rejection, and canary-secret containment. `bash scripts/verify.sh` passes with frontend lint/build, warnings-denied Rust checks, 36 tests, schemas, package validation, and diff checks. Focused security review decision: Approve; no blocker found.

## Definition of Done

- [x] (agent) Approved MCP tools are typed, bounded, and registry-scoped.
- [x] (agent) Trust approval, arbitrary command/path, secret, and network capabilities remain absent.
- [x] (agent) Timeout, cancellation, cleanup, and redaction paths are tested.
- [x] (agent) Protocol lifecycle evidence and implementation documentation are current.
- [x] (agent) Aggregate lint, test, build, schema, and package verification passes.

### user

### agent

Working in `task_010_phase_5_mcp.md`. The user explicitly approved autonomous execution through every phase, so this task moved to `in-progress` while the completed Phase 4 evidence is normalized. Implemented the full approved nine-tool surface over stdio, registry-only workspace selection, stable structured tool errors, strict property and bound validation, safe MCP projections that exclude internal paths/PIDs/fingerprints, atomic fail-closed workspace registry writes, and a protocol-level full lifecycle test. The test exposed and fixed an external-runtime session bug: `startRuntime: false` no longer requires the unspawned executable to resolve on PATH. Focused MCP unit tests, the stdio integration suite, and warnings-denied Clippy pass. Cancellation and final end-to-end evidence remain.

### user

Approved by the Phase 0-to-5 implementation request.

### agent

Queued after trusted session behavior is accepted. It must not add a network listener, trust approval tool, arbitrary command, secret output, or unbounded log stream.

### user
