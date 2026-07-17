# Phased Implementation Plan: FlareDeck AI Development Integration

## 1. Planning principles

- Deliver a usable outcome at the end of every phase.
- Do not begin MCP implementation before the shared application services and CLI contracts are stable.
- Separate behavior extraction from large file moves.
- Treat security, tests, migration, cleanup, and documentation as phase deliverables.
- Convert each phase into small Agentic Kanban tasks with explicit dependencies and blockers.
- Block implementation when product, security, compatibility, or migration decisions remain unresolved.

## 2. Delivery tiers

### MVP

Phases 0–6.

### Production-quality desktop integration

Phase 7.

### Advanced integration tooling

Phase 8.

### Release and ecosystem hardening

Phase 9.

## Phase 0: Discovery and baseline protection

### Objective

Confirm current behavior, repository structure, tests, release assumptions, and security invariants before changing architecture.

### Deliverables

- repository map verified against current source;
- current command surface inventory;
- profile, tunnel, route, secret, WSL, updater, and release regression checklist;
- baseline test report;
- gap analysis between current code and target docs;
- decisions or corrections recorded as ADR changes;
- implementation task backlog for Phase 1.

### Required analysis

- trace profile creation from React to Tauri to Cloudflare API and disk;
- trace tunnel start/stop and crashloop behavior;
- trace DNS route API and CLI fallback;
- inspect token storage and redaction paths;
- inspect release workflow and updater coupling;
- identify existing tests and untested critical paths;
- confirm package manager and Rust toolchain assumptions.

### Acceptance criteria

- no unresolved contradiction between docs and current behavior;
- all current invariants have named regression checks;
- no implementation changes mixed into the baseline commit except test harness corrections explicitly approved;
- Phase 1 tasks have dependencies and verification commands.

### Rollback

Documentation-only phase. Revert the baseline commit if it misrepresents the repository.

## Phase 1: Repository readiness and verification baseline

### Objective

Make AI-assisted changes reliably verifiable before changing runtime architecture.

### Deliverables

- aggregate `scripts/verify.sh` or equivalent;
- pull-request CI for frontend lint/build and Rust fmt/clippy/test;
- documentation and JSON schema validation;
- test fixtures and temporary-directory conventions;
- contribution guidance linked from `AGENTS.md`;
- stable strategy for tests requiring Cloudflare credentials;
- optional pre-commit hooks only if they remain lightweight.

### Tasks

1. Add aggregate verification script.
2. Add CI without changing release workflow behavior.
3. Expand unit-test discovery beyond only selected modules.
4. Add fixture utilities for filesystem and process tests.
5. Add documentation validation for this enhancement pack.
6. Record baseline CI duration and known platform differences.

### Acceptance criteria

- clean checkout passes documented verification;
- no real Cloudflare credential is needed for standard CI;
- warnings fail CI where currently intended;
- generated and build artifacts remain excluded;
- release workflow still behaves as before.

### Risks

- Tauri Linux dependencies in CI;
- platform-specific tests accidentally run on unsupported runners;
- existing hidden warnings become blockers.

### Exit condition

A coding agent can make a small change and receive one unambiguous verification result.

## Phase 2: Domain and application-service extraction

### Objective

Create shared Rust services and ports without changing observable behavior.

### Deliverables

- domain IDs and error categories;
- application operation context and correlation ID;
- profile service wrapping existing profile behavior;
- tunnel supervisor port wrapping existing tunnel commands;
- route and DNS port wrapping current API/CLI fallback;
- adapters for existing storage and process state;
- thin Tauri handlers calling services;
- regression tests proving behavior parity.

### Tasks

1. Define module boundaries with minimal public surface.
2. Extract one vertical slice, preferably tunnel status/start/stop, behind a service.
3. Preserve log streaming and crashloop behavior.
4. Extract profile read operations.
5. Extract route/DNS operations while preserving `hint_for` scope behavior.
6. Move remaining orchestration incrementally.
7. Keep TypeScript wrappers compatible.

### Acceptance criteria

- desktop behavior remains unchanged;
- Tauri handlers contain translation and no substantial orchestration;
- services can be tested with fake ports;
- no token handling bypass;
- current five-place Tauri rule remains satisfied;
- no premature Cargo workspace split.

### Risks

- accidental state duplication;
- process ownership regressions;
- over-generalized abstractions that hide required platform behavior.

### Rollback

Each extracted vertical slice remains independently revertible.

## Phase 3: Headless CLI foundation

### Objective

Provide a stable machine-readable interface over shared services before adding workspace runtime orchestration or MCP.

### Deliverables

- `flaredeck` binary entry point;
- human and JSON output modes;
- response envelope and error mapping;
- workspace-independent profile/tunnel diagnostic commands as an initial proof;
- `doctor` command;
- CLI snapshot and exit-code tests;
- packaging decision for development builds.

### Initial command scope

- profile list or safe status where useful;
- tunnel status by profile;
- health or doctor information;
- version and schema information.

Workspace and session commands may be added fully in Phase 4.

### Acceptance criteria

- CLI and Tauri return equivalent results for shared operations;
- JSON output contains no ANSI control sequences;
- stderr and stdout behavior is documented;
- token and credential redaction tests pass;
- stable exit-code categories exist;
- CLI can run without launching a desktop window.

### Risks

- desktop crate startup assumptions in a CLI process;
- updater or Tauri plugin initialization leaking into headless mode;
- output contracts changing before MCP.

### Exit condition

Shared services are proven usable without Tauri UI.

## Phase 4: Workspace, trust, runtime, and session orchestration

### Objective

Implement the core local development control-plane behavior.

### Deliverables

- manifest parser and schema validation;
- canonical path policy;
- workspace registry;
- trust fingerprint and local approval storage;
- runtime supervisor;
- TCP and HTTP readiness probes;
- session aggregate and orchestrator;
- session persistence and recovery policy;
- route verification and public URL calculation;
- CLI workspace/session commands;
- audit events and redaction.

### Task groups

#### Workspace and manifest

- discover `.flaredeck/project.yaml`;
- parse and validate;
- normalize executable, args, paths, routes, and probes;
- produce safe workspace view.

#### Trust

- canonical security projection;
- fingerprint test vectors;
- trust approval repository;
- desktop approval adapter or minimal trust review UX;
- invalidate on relevant changes.

#### Runtime

- direct process spawn;
- platform-aware process-tree termination;
- bounded logs;
- independent crashloop policy;
- cancellation.

#### Session

- state machine;
- workspace lock;
- runtime and tunnel ownership;
- compensation and cleanup;
- idempotent start/stop/status;
- audit events.

### Acceptance criteria

- an approved fixture workspace starts and reaches readiness;
- an untrusted workspace is blocked;
- path traversal is blocked;
- changing runtime args invalidates trust;
- failed readiness cleans up the owned runtime;
- pre-existing tunnel is not stopped by session cleanup;
- repeated stop is successful and harmless;
- CLI returns public URLs and bounded logs;
- no secrets appear in manifest, state, logs, or output.

### Risks

- unsafe shell assumptions;
- Windows process-tree leaks;
- trust UX becoming a bypassable checkbox;
- session recovery after application crash.

### Exit condition

A human can run the complete development exposure lifecycle through CLI and minimal desktop approval.

## Phase 5: Local MCP server

### Objective

Expose the approved workspace/session capabilities to local AI clients through a small MCP stdio server.

### Deliverables

- `flaredeck-mcp` binary;
- MCP SDK decision and dependency review;
- stdio initialization and tool discovery;
- typed tools from `docs/specs/mcp-tools.md`;
- operation context for MCP actor;
- bounded results and structured errors;
- stdout/stderr discipline tests;
- cancellation and timeout behavior;
- example client configuration.

### Acceptance criteria

- server is launched as a child process by an MCP client;
- stdout contains only protocol messages;
- tool schemas reject unknown or unsafe inputs;
- tool calls use shared application services;
- untrusted workspace start remains blocked;
- no MCP tool can approve trust;
- logs and paths follow MCP output policy;
- tool list remains within approved scope;
- protocol and redaction tests pass.

### Risks

- SDK maturity or version churn;
- accidental diagnostic output on stdout;
- context bloat from excessive tools or verbose results;
- long-running readiness request cancellation.

### Exit condition

MCP Inspector or an equivalent client can execute the complete safe session lifecycle.

## Phase 6: AI client integration and acceptance evidence

### Objective

Prove the local workflow in OpenCode and VS Code and make it usable in controlled AI development tasks.

### Deliverables

- verified OpenCode project configuration;
- verified VS Code `.vscode/mcp.json` example;
- Agent Skills installed under `.agents/skills/`;
- phased and task prompts validated against the implementation;
- fixture application for end-to-end tests;
- example Agentic Kanban task;
- acceptance evidence format;
- cross-platform smoke-test matrix;
- operational guide for developers.

### End-to-end scenario

1. Agent reads `AGENTS.md` and active task.
2. Agent checks workspace status.
3. Human approves workspace if required.
4. Agent starts session.
5. Agent receives public URL.
6. Agent runs external integration or browser test.
7. Agent reads health and redacted logs.
8. Agent stops session.
9. Agent attaches command results and test evidence to the task.

### Acceptance criteria

- OpenCode discovers and executes tools;
- VS Code discovers and executes tools after trust confirmation;
- no project API key is stored in MCP config;
- an agent cannot bypass trust through prompt wording;
- a complete start/test/stop trace is reproducible;
- generated evidence includes correlation and session IDs but no secret;
- docs accurately match real commands and schemas.

### Risks

- client-specific MCP behavior differences;
- auto-approval settings wider than FlareDeck’s intended policy;
- stale example configurations.

### MVP exit condition

FlareDeck is a secure local AI-development exposure control plane with a tested human and agent workflow.

## Phase 7: Desktop workspace and session UX

### Objective

Replace minimal approval and CLI-centric management with a polished desktop experience.

### Deliverables

- workspace navigation and detail;
- trust review and configuration diff;
- session dashboard and pipeline;
- public URL controls;
- combined logs and health;
- audit history;
- accessibility and responsive validation.

### Acceptance criteria

Defined in `DESIGN.md`.

### Risks

- duplicating domain state in frontend;
- ambiguous status labels;
- exposing safe internal details too broadly to MCP because the desktop needs them.

## Phase 8: Temporary routes and webhook inspector

### Objective

Support task-specific exposure and safe webhook debugging without changing the core tunnel-per-profile invariant.

### Deliverables

- temporary session route ownership and expiration;
- cleanup reconciliation;
- request capture proxy;
- redaction configuration;
- request replay with approval;
- read-only MCP event tools by default;
- storage retention policy;
- separate webhook threat model.

### Acceptance criteria

- temporary route cleanup is ownership-safe and idempotent;
- persistent routes are never removed by temporary cleanup;
- sensitive headers and configured fields are redacted before storage;
- replay cannot target arbitrary hosts;
- event volume and body size are bounded.

### Risks

- capturing credentials or personal data;
- replay side effects;
- becoming a general reverse proxy or Postman replacement.

## Phase 9: Release and ecosystem hardening

### Objective

Package and support CLI/MCP capabilities across FlareDeck release targets.

### Deliverables

- binary packaging strategy;
- version compatibility checks;
- updater behavior for companion binaries;
- release artifact signing and checksums;
- migration and rollback documentation;
- smoke tests for release artifacts;
- optional templates for other MCP clients after official syntax verification.

### Acceptance criteria

- desktop, CLI, and MCP versions are compatible and discoverable;
- release artifacts pass platform smoke tests;
- upgrades migrate safe state and preserve existing profiles;
- rollback behavior is documented;
- no signing or updater regression.

## 3. Agentic Kanban task rules

Every implementation task must include:

- outcome;
- phase and parent deliverable;
- dependencies;
- scope and exclusions;
- source-of-truth references;
- files likely affected;
- acceptance criteria;
- tests and verification;
- security and migration considerations;
- rollback note;
- evidence required.

Tasks that combine domain extraction, CLI, MCP, UI, and release changes must be split. Humanity has already run enough “small refactors” that modify forty-seven files.

## 4. Blocking decisions

Implementation must stop and request an ADR when any task proposes:

- weakening one-profile/one-tunnel/one-token;
- arbitrary shell mode;
- MCP trust approval;
- remote MCP or an HTTP listener;
- wider Cloudflare scopes;
- secrets in manifests or outputs;
- multiple sessions per workspace;
- automatic persistent route deletion;
- database or daemon introduction;
- core process-lifecycle simplification;
- modification of updater signing assumptions.

## 5. Recommended implementation order inside phases

Use vertical slices:

1. domain type;
2. port;
3. fake adapter and tests;
4. concrete adapter;
5. application service;
6. CLI proof;
7. Tauri adapter;
8. MCP adapter when Phase 5 begins;
9. documentation and acceptance evidence.

This order reduces the delightful industry practice of designing an API around code that cannot be tested.
