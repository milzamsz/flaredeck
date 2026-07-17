---
name: flaredeck-security-review
description: Perform a focused security review of FlareDeck workspace, runtime, CLI, MCP, Cloudflare, route, logging, secret, or release changes. Use when a task touches execution, trust, credentials, public exposure, process control, or protocol boundaries.
argument-hint: "[change or threat area]"
context: fork
---

# FlareDeck Security Review

## Required references

- `docs/security/THREAT-MODEL.md`
- `docs/security/SECRET-HANDLING.md`
- `docs/security/TRUST-AND-APPROVAL.md`
- relevant ADRs and specifications

## Procedure

1. Map attacker-controlled inputs and trust boundaries.
2. Trace data and control flow from interface to infrastructure.
3. Check secret containment and redaction at every output boundary.
4. Check trust approval, fingerprint invalidation, and fail-closed behavior.
5. Check command, argument, path, environment, readiness, and route validation.
6. Check process ownership, process-tree termination, PID recovery, cancellation, crashloops, and cleanup.
7. Check MCP schema, stdout discipline, result bounds, and prohibited capabilities.
8. Check public-exposure and DNS mutation risk.
9. Run canary-secret and malicious-input tests where possible.
10. Report threats, controls, findings, and decision.

## Block immediately when

- an interface can return a token, credential, or environment value;
- an agent can approve trust;
- a caller can supply an arbitrary command;
- a session can stop or delete unowned resources;
- remote MCP is introduced without a new threat model;
- state corruption causes automatic trust or fail-open behavior.
