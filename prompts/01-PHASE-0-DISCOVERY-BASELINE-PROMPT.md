# Phase 0 Prompt: Discovery and Baseline Protection

Use this prompt only for Phase 0.

## Objective

Produce an evidence-based baseline of the current FlareDeck repository before architecture changes. Do not implement enhancement features.

## Required work

1. Read all source-of-truth documents and current `AGENTS.md`.
2. Map current frontend and Rust modules.
3. Trace these flows end to end:
   - profile creation and token storage;
   - zone lookup and tunnel-scope preflight;
   - named tunnel creation and credential-file writing;
   - route creation through Cloudflare API and CLI fallback;
   - YAML parsing, catch-all preservation, and WSL rewriting;
   - tunnel start, stop, restart, log streaming, and crashloop handling;
   - updater and release workflow.
4. Inventory Tauri commands and TypeScript wrappers.
5. Inventory tests, CI, release workflows, and platform-specific scripts.
6. Run all currently documented verification commands that the environment supports.
7. Compare actual behavior with `PRODUCT-SCOPE.md`, ADRs, domain, architecture, and technical specifications.
8. Classify differences as:
   - documentation correction;
   - existing bug;
   - required refactor;
   - blocked decision;
   - future scope.
9. Produce Phase 1 Agentic Kanban tasks with dependencies, acceptance criteria, and verification.

## Prohibited work

- no MCP implementation;
- no CLI implementation;
- no workspace/session features;
- no broad module moves;
- no dependency additions;
- no behavior changes unless needed to make baseline verification run and separately justified.

## Deliverables

- repository baseline report;
- current command and type map;
- regression checklist;
- test and CI gap analysis;
- conflict list;
- proposed ADR corrections if needed;
- prioritized Phase 1 task backlog.

## Exit criteria

Phase 0 is complete only when current critical behavior is understood and every material enhancement assumption is confirmed, corrected, or blocked.
