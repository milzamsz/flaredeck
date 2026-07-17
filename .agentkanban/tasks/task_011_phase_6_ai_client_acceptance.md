---
title: Prove AI client integration and acceptance
lane: done
created: 2026-07-17T18:45:00+07:00
updated: 2026-07-17T19:00:00+07:00
description: Current OpenCode and VS Code clients must discover the local MCP server and reproduce the trusted session workflow without secrets.
priority: high
labels: [phase-6, mcp, testing, security]
dependsOn: [task_010_phase_5_mcp]
evidence:
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
---

## Conversation

### user

Approved by the request to complete every roadmap prompt autonomously.

### agent

Working in `task_011_phase_6_ai_client_acceptance.md`. Moved directly to implementation under the user's blanket phase approval. Current official OpenCode and VS Code configuration references were checked. OpenCode 1.18.1 successfully loaded the example and connected to the built stdio server; its deprecated `tools` policy was replaced by current global/per-agent `permission` rules. VS Code 1.129.0 matches the checked-in stdio configuration shape. A headless smoke pack and explicit UI/platform gaps remain.

### user

## Definition of Done

- [x] (agent) Current OpenCode and VS Code configurations are validated.
- [x] (agent) A non-secret fixture and full lifecycle smoke scenario are reproducible.
- [x] (agent) Trust invalidation and prompt-injection/secret denial evidence is recorded.
- [x] (agent) Client/version/platform results and explicit gaps are documented.
- [x] (agent) Aggregate verification passes.

### agent

Phase 6 review decision: Approve. OpenCode 1.18.1 connected to the built server using the current permission syntax; VS Code 1.129.0 configuration matches its official stdio schema. The checked-in non-secret fixture, `scripts/ai-integration-smoke.sh`, and MCP protocol suite reproduce discovery and the complete safe lifecycle, including trust invalidation, prompt-injection denial, canary redaction, cancellation, and cleanup. `bash scripts/verify.sh` passes with 43 Rust/contract tests. macOS, Windows, WSL, and the VS Code first-run trust dialog are explicitly recorded as environment gaps rather than claimed passes.

### user
