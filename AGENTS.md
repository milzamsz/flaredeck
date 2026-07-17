# AGENTS.md

This file applies to the entire `flaredeck` repository. It is the always-on operating contract for coding agents and human contributors using AI-assisted development.

## 1. Mission

FlareDeck is a local-first desktop control panel for Cloudflare Tunnel development. The enhancement roadmap adds a trusted local development control plane that can be accessed through the desktop UI, a headless CLI, and a local MCP server.

FlareDeck is **not** an embedded AI application, a hosted orchestration service, a general-purpose shell runner, or a production deployment platform.

## 2. Source-of-truth hierarchy

When information conflicts, use this order:

1. `PRODUCT-SCOPE.md`
2. Approved ADRs under `docs/adr/`
3. `DOMAIN-MODEL.md`
4. `ARCHITECTURE.md`
5. `TECHNICAL.md`
6. Detailed specifications under `docs/specs/`
7. `PLAN.md`
8. Active task specification
9. Existing implementation

The existing implementation is evidence of current behavior. It does not automatically override approved architecture. Report conflicts before implementing a material behavior change.

## 3. Hard invariants

These rules may not be changed without an explicit product decision and approved ADR:

1. One profile equals one Cloudflare Tunnel and one API-token identity.
2. API tokens are never stored in plaintext and never returned through UI, CLI, MCP, logs, or audit events.
3. The existing keychain and encrypted fallback remain the only sanctioned token-write paths.
4. Profile creation must verify required access before mutating local or Cloudflare state.
5. Ingress configuration preserves the final catch-all `http_status:404` rule.
6. WSL loopback-origin rewriting remains supported.
7. Process lifecycle remains platform-aware, including process-tree termination and crashloop protection.
8. Desktop, CLI, and MCP interfaces use shared Rust application services.
9. The MCP server uses local stdio for the MVP and does not bind a network listener.
10. No CLI or MCP operation accepts an arbitrary shell command.
11. A workspace runtime may start only from a validated manifest with an active local trust approval for the current fingerprint.
12. MCP callers cannot approve workspace trust.
13. Stop and cleanup operations are idempotent and ownership-aware.
14. New Cloudflare endpoints require scope analysis and actionable error hints.
15. Existing updater signing assumptions must not be changed casually.

## 4. Current project shape

FlareDeck is a hybrid client:

- control-plane operations use the Cloudflare REST API with a per-profile token;
- data-plane traffic uses the local `cloudflared` child process;
- React, Zustand, and Tauri provide the desktop interface;
- Rust manages Cloudflare operations, local files, secrets, networking, DNS checks, WSL behavior, and child processes.

### Existing frontend areas

```text
src/
├── store/app-store.ts
├── lib/
│   ├── tauriApi.ts
│   └── yaml-helpers.ts
├── components/
│   ├── ui/
│   ├── app-sidebar.tsx
│   ├── proxy-table.tsx
│   └── proxy-form-dialog.tsx
└── pages/
    ├── Dashboard.tsx
    ├── Config.tsx
    └── Settings.tsx
```

### Existing Rust areas

```text
src-tauri/src/
├── lib.rs
├── cf_api.rs
├── cloudflared.rs
├── secrets.rs
├── state.rs
├── error.rs
├── types.rs
└── commands/
    ├── cf.rs
    ├── config.rs
    ├── dns.rs
    ├── network.rs
    ├── prefs.rs
    ├── profiles.rs
    ├── shell.rs
    ├── tunnel.rs
    └── wsl.rs
```

### Target enhancement areas

```text
src-tauri/src/
├── application/
├── domain/
├── ports/
├── adapters/
├── interfaces/
└── bin/
```

Do not create all target folders in a single rename-only change. Extract one tested vertical slice at a time.

## 5. On-disk data

Existing data under `~/.cloudflared/` includes:

- profile index;
- per-profile YAML;
- tunnel credentials JSON;
- optional `cert.pem`;
- encrypted token fallback when keychain access is unavailable.

New workspace, trust, session, and audit state must live in the FlareDeck application-data directory, not in the repository and not in browser storage. The repository may contain only the non-secret `.flaredeck/project.yaml` manifest and documentation.

Generated output that must not be edited or committed unless explicitly requested:

- `node_modules/`
- `dist/`
- `dist-windows/`
- `src-tauri/target/`
- `src-tauri/gen/`

## 6. Required task workflow

For every non-trivial task:

1. Read the active task and its parent phase in `PLAN.md`.
2. Read relevant source-of-truth documents and ADRs.
3. Inspect existing code before proposing edits.
4. State any conflict, ambiguity, migration risk, security risk, or blocker.
5. Define a small implementation plan tied to acceptance criteria.
6. Implement only the approved task scope.
7. Add or update tests before declaring completion.
8. Run the required verification commands.
9. Review the diff for unrelated churn and secret exposure.
10. Update affected docs, schemas, and examples.
11. Produce evidence: changed files, tests, commands, results, risks, and unresolved items.

Do not re-enter broad planning after implementation begins unless the scope materially changes.

## 7. Tauri command rule

Every Tauri command must be represented consistently in five places:

1. Rust handler.
2. Shared serde request/response type.
3. Registration in `lib.rs`.
4. TypeScript wrapper and type in `src/lib/tauriApi.ts`.
5. Caller in Zustand or a component.

New handlers must be thin adapters. Business orchestration belongs in application services.

## 8. Cloudflare integration rules

- Preserve preflight-before-mutation behavior.
- New endpoints require a named operation type and a corresponding user-facing error hint.
- Do not widen token scopes without updating product scope, security docs, UI guidance, and an ADR.
- Route creation and cleanup must be ownership-aware.
- Persistent profile routes are not deleted as part of ordinary session cleanup.
- Preserve API and CLI fallback behavior until an approved phase removes it.
- Avoid real Cloudflare calls in standard CI.

## 9. Workspace manifest rules

The canonical project manifest is `.flaredeck/project.yaml`.

The manifest:

- contains no secrets;
- uses executable and argument arrays rather than an opaque shell command;
- resolves paths beneath the workspace root;
- selects an existing profile;
- declares readiness and exposure routes;
- declares environment names explicitly;
- is validated against `docs/specs/workspace.schema.json`;
- contributes security-relevant fields to the trust fingerprint.

Never add an MCP or CLI option that bypasses the manifest command with a caller-supplied command.

## 10. Trust and authorization rules

- Trust is local, explicit, revocable, and fingerprint-based.
- A path alone is not trusted.
- Manifest changes that affect execution, routes, environment, profile, readiness, or cleanup invalidate trust.
- Display-only metadata may be excluded from the fingerprint.
- AI tools may read trust status but may not create approval.
- A test may use a fake trust repository; production code may not use a hidden auto-trust fallback.
- Approval UI must show executable, arguments, working directory, environment names, readiness target, routes, profile, and lifecycle behavior.

## 11. Process lifecycle rules

- Spawn processes directly without a shell in the MVP.
- Use platform-aware path handling.
- Track process ownership by session.
- Stop the entire owned process tree.
- Bound stdout and stderr buffers.
- Apply crashloop protection independently to development runtimes and tunnels.
- Cancel readiness checks when a session stops.
- A session may stop only the tunnel it started, unless an explicit user action authorizes wider impact.
- Never simplify the existing tunnel process code without demonstrating equivalent cross-platform behavior.

## 12. MCP rules

- Use stdio for the MVP.
- stdout contains only valid MCP protocol messages.
- diagnostics go to stderr.
- Keep the tool surface small and typed.
- Reject unknown properties where practical.
- Bound every log, list, and status response.
- Redact paths more strictly than the desktop UI where appropriate.
- Never expose token values, tunnel credential contents, raw environment values, unrestricted file access, or arbitrary Cloudflare API calls.
- Do not add tools solely because an internal method exists.
- Long-running tools must respect cancellation and timeouts.

## 13. CLI rules

- Support human and JSON output.
- JSON mode has no ANSI escapes or decorative output.
- Use stable response envelopes and exit-code categories.
- Print data to stdout and diagnostics to stderr.
- Never print secret values.
- CLI behavior must be testable through snapshots or structured assertions.
- Trust approval remains desktop-only until an ADR approves a secure interactive CLI flow.

## 14. Error handling

- Use `AppError` and `AppResult` or their approved successor.
- No `panic!` in interface or application paths.
- Errors crossing interfaces have stable codes and safe details.
- Internal causes may be logged after redaction.
- Retryability must be explicit where returned to CLI or MCP.
- Cleanup errors must not replace the original failure; report both safely.

## 15. State management

- Zustand holds UI state and safe display models.
- Pure form state may remain local to components.
- Persisted browser state stays narrow and must never contain tokens, logs, environment values, profile lists, or authoritative session state.
- Rust application services own authoritative workspace/session behavior.
- Avoid a global mutable singleton when dependency injection or managed state can keep tests isolated.

## 16. UI rules

- Reuse shadcn primitives, Lucide icons, `cn()`, Tailwind, and existing theme patterns.
- Do not add another UI library without justification.
- Use text plus icons for status; color alone is insufficient.
- Distinguish profile API status, tunnel status, runtime status, readiness, DNS, and public-route health.
- Trust screens must show meaningful configuration, not a generic confirmation.
- Never offer “show unredacted” behavior.
- Follow `DESIGN.md` when Phase 7 begins.

## 17. Testing expectations

### Documentation-only

- validate links and JSON files where tooling exists;
- run `bash scripts/validate-package.sh` for the enhancement pack.

### Frontend

- `npm run lint`;
- `npm run build` when types, imports, routes, or bundling behavior change;
- component or state tests added when infrastructure exists;
- desktop spot-check for Tauri-gated flows.

### Rust

- format check;
- clippy with warnings denied;
- relevant unit and integration tests;
- fake adapters for Cloudflare, filesystem, process, trust, and session behavior.

### Cross-boundary

- verify Rust types, Tauri registrations, TypeScript wrappers, Zustand actions, and UI caller;
- verify response casing and optional fields;
- verify no secret appears in serialized results.

### Security-sensitive

Test normal, under-scoped, untrusted, changed-fingerprint, path-escape, readiness-timeout, crashloop, cancellation, repeated-stop, and cleanup-failure paths.

If a command cannot run in the current environment, state that explicitly. Do not pretend that reading code is equivalent to a passing test suite, a habit software has somehow survived for decades.

## 18. Common commands

Current baseline:

```bash
npm install
npm run dev
npm run desktop
npm run lint
npm run build
npm run desktop:build

cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

The roadmap should add one aggregate verification command, preferably:

```bash
bash scripts/verify.sh
```

Use `npm` while `package-lock.json` is authoritative. Do not switch package managers casually.

## 19. Dependency policy

Before adding a dependency:

- identify the exact capability gap;
- check existing dependencies;
- assess maintenance, license, platform support, binary size, security, and release impact;
- prefer a small proven crate or no dependency when implementation is straightforward;
- isolate MCP SDK dependencies so a later crate split remains possible;
- do not add Redis, queues, databases, daemons, or hosted services without a product requirement.

## 20. Documentation update rules

Update documents when changing:

- product boundary or non-goals: `PRODUCT-SCOPE.md`;
- entities or invariants: `DOMAIN-MODEL.md`;
- components or dependency direction: `ARCHITECTURE.md`;
- commands, schemas, errors, files, or tests: `TECHNICAL.md` and `docs/specs/`;
- a material decision: new or superseding ADR;
- phase scope or dependencies: `PLAN.md`;
- user interaction: `DESIGN.md`;
- security control: `docs/security/`;
- agent workflow: this file, skills, or prompts.

Do not rewrite unrelated documents merely to harmonize wording.

## 21. Definition of done

A non-trivial task is complete only when:

- acceptance criteria are met;
- exclusions remain excluded;
- tests cover success, failure, and cleanup;
- required verification passes;
- relevant docs and schemas are current;
- no secret or sensitive fixture is committed;
- compatibility and migration impacts are stated;
- rollback is understood;
- reviewer findings are resolved or explicitly accepted;
- final evidence is concise and reproducible.

## 22. Prohibited shortcuts

Do not:

- bypass trust for “local convenience”;
- add `run_shell`, `execute_command`, or equivalent MCP tools;
- read and return `.env` values;
- put tokens in manifests or test fixtures;
- duplicate orchestration in CLI or MCP;
- stop tunnels not owned by the session without approval;
- delete persistent routes during normal cleanup;
- add HTTP MCP before a new threat model;
- refactor all modules and add features in the same task;
- modify core updater signing keys;
- claim a task is verified when commands were not run.

<!-- BEGIN AGENTIC KANBAN — DO NOT EDIT THIS SECTION -->
## Agentic Kanban

Read `.agentkanban/INSTRUCTION.md` for task workflow rules.
Read `.agentkanban/memory.md` for project context.

Enforcement mode: `warn`

Load these project skills before working: `flaredeck-implementation`, `flaredeck-release`, `flaredeck-security-review`, `flaredeck-verification`.

If a task file (`.agentkanban/tasks/**/*.md`) was referenced earlier in this conversation, re-read it before responding and always respond in and at the end the task file.
<!-- END AGENTIC KANBAN -->
