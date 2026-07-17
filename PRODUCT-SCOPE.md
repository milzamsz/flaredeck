# Product Scope: FlareDeck AI Development Integration

## 1. Executive summary

FlareDeck currently provides a desktop control panel for creating and operating local Cloudflare Tunnels. The enhancement described here turns FlareDeck into a **local development exposure control plane** that can be used consistently by its desktop UI, a headless CLI, and AI development tools through a local MCP server.

The business and developer value is not “adding AI to FlareDeck.” The value is making local development exposure deterministic, observable, scriptable, and safe enough for controlled use by coding agents.

A developer or agent should be able to identify a trusted project, start its configured local runtime, verify readiness, start or reuse its FlareDeck tunnel, obtain a public URL, read bounded logs and health information, and stop or clean up the session. Secrets remain inside FlareDeck’s existing protected storage and are never returned to the agent.

## 2. Product problem

Local integration development commonly requires several disconnected actions:

- start a framework-specific development command;
- determine which local port is serving;
- maintain Cloudflare Tunnel ingress configuration;
- create or verify DNS records;
- start `cloudflared`;
- wait for application readiness;
- share a public callback URL with a payment gateway, webhook provider, OAuth service, mobile device, or external tester;
- inspect application and tunnel logs;
- remove temporary exposure after the task.

Humans can perform these steps manually, but AI agents need typed, deterministic interfaces. Giving an agent unrestricted terminal access to recreate the workflow is unsafe and produces inconsistent state. FlareDeck already owns the tunnel, DNS, secret, and process-management concerns, making it the correct control plane.

## 3. Product vision

> FlareDeck is the trusted local control plane for exposing development services through Cloudflare Tunnel, usable interactively by developers and programmatically by approved local tools and AI agents.

## 4. Goals

### 4.1 Primary goals

1. Provide one shared implementation for desktop, CLI, and MCP operations.
2. Introduce a project-level workspace manifest with no committed secrets.
3. Model a bounded development session covering runtime, tunnel, routes, health, and logs.
4. Provide machine-readable CLI commands with stable error contracts.
5. Provide a local MCP server over stdio with a small, typed tool surface.
6. Preserve the current invariant: one FlareDeck profile equals one Cloudflare Tunnel and one API token.
7. Protect users from arbitrary command execution, secret disclosure, path traversal, and unintended Cloudflare mutations.
8. Generate sufficient audit evidence for a developer to understand what an agent changed or started.

### 4.2 Secondary goals

- make the same workflow usable from OpenCode and VS Code;
- support deterministic integration and webhook testing;
- provide an evolution path to temporary task routes and a webhook inspector;
- improve repository structure, tests, and documentation for AI-assisted development.

## 5. Non-goals

The following are explicitly outside the MVP:

- embedding an LLM, chat interface, or coding agent in FlareDeck;
- exposing a general-purpose shell execution tool;
- returning API tokens, tunnel credentials, `.env` contents, or process environment values;
- remote MCP hosting or public MCP endpoints;
- multi-user collaboration, RBAC, organizations, or hosted control planes;
- replacing Cloudflare Access, production deployment, or production service discovery;
- creating one Cloudflare Tunnel per task by default;
- Kubernetes, Redis, message brokers, or distributed orchestration;
- automatic code modification by FlareDeck;
- autonomous route or DNS deletion without an approved session policy;
- supporting every development framework through framework-specific code.

## 6. Target users

### 6.1 Primary user: local application developer

Needs to expose a development application or callback endpoint without manually managing YAML, DNS, tunnel commands, and process logs.

### 6.2 Primary user: AI-assisted developer

Uses OpenCode, VS Code, Cursor-compatible MCP clients, or another local agent and needs a safe tool contract for starting and inspecting a development exposure session.

### 6.3 Secondary user: integration engineer

Tests payment callbacks, OAuth redirects, Odoo integrations, GitHub webhooks, n8n workflows, SaaS APIs, or mobile clients against a local service.

### 6.4 Secondary user: maintainer

Needs predictable architecture, regression tests, explicit security invariants, and cross-platform behavior.

## 7. Core user journeys

### Journey A: developer starts a configured project

1. The developer creates `.flaredeck/project.yaml` without secrets.
2. FlareDeck validates the manifest and displays the command, root, routes, and profile binding.
3. The developer explicitly trusts the workspace.
4. FlareDeck starts the development runtime.
5. FlareDeck waits for the configured readiness probe.
6. FlareDeck starts or reuses the profile tunnel.
7. FlareDeck reports local and public URLs.
8. The developer inspects health and logs.
9. The developer stops the session.

### Journey B: AI agent runs an integration task

1. The agent reads repository instructions and the task acceptance criteria.
2. The agent calls `workspace_status` or the equivalent CLI command.
3. If the workspace is not trusted or requires a changed command, the operation is blocked for human approval.
4. The agent starts the session using a typed tool.
5. The agent receives only bounded status information and public URLs.
6. The agent runs its own application tests against the URL.
7. The agent reads redacted runtime and tunnel logs.
8. The agent stops the session and records acceptance evidence.

### Journey C: workspace command changes

1. A committed manifest changes `bun run dev` to another command.
2. FlareDeck calculates a new trust fingerprint.
3. Existing approval becomes invalid.
4. Automated start is denied.
5. The UI shows the exact reviewed difference and requests approval.

## 8. MVP functional scope

### 8.1 Workspace management

- discover a manifest at `.flaredeck/project.yaml`;
- validate against the published schema;
- resolve the canonical workspace root;
- bind a workspace to an existing FlareDeck profile;
- expose parsed configuration without secret values;
- calculate a stable trust fingerprint;
- store local trust approval separately from the repository;
- detect manifest or command changes.

### 8.2 Runtime orchestration

- execute only the approved manifest command;
- set the configured working directory;
- allow only declared environment names or values explicitly permitted by policy;
- stream bounded stdout and stderr;
- check readiness using TCP or HTTP probes;
- stop the child process and its process tree correctly on supported platforms;
- apply crashloop protection independently from tunnel crashloop protection.

### 8.3 Session orchestration

- create one active session per workspace by default;
- start runtime, readiness, tunnel, and route verification in a defined order;
- return a stable session identifier;
- support status, logs, health, and stop operations;
- support idempotent start and stop behavior;
- persist minimal session metadata required for recovery;
- record audit events without secrets.

### 8.4 Headless CLI

- support human-readable and JSON output;
- provide stable exit codes and structured errors;
- never print secrets;
- support workspace, session, route, logs, health, and doctor commands;
- use the same application services as Tauri commands.

### 8.5 MCP server

- run locally using stdio;
- expose a deliberately small tool set;
- use schemas with `additionalProperties: false` where practical;
- write protocol messages only to stdout and diagnostics only to stderr;
- call the same application services as CLI and desktop;
- require human approval through FlareDeck state for trust-sensitive operations;
- return structured, bounded, redacted data.

### 8.6 Audit and observability

- record actor type, operation, workspace, session, result, timestamps, and safe metadata;
- keep secrets and raw environment values out of logs;
- provide correlation IDs across CLI, MCP, runtime, tunnel, and Cloudflare operations;
- cap in-memory and returned log volume.

## 9. Post-MVP scope

### 9.1 Desktop workspace and session UI

- workspace list and detail;
- trust approval and change comparison;
- runtime and tunnel status;
- public URL copy actions;
- combined log filtering;
- session history and cleanup controls.

### 9.2 Temporary task routes

- create a task-specific hostname under an approved zone;
- reuse a persistent profile tunnel;
- apply expiration and cleanup policy;
- remove only routes created by the session.

### 9.3 Webhook inspector

- capture approved inbound requests;
- redact configured headers and JSON fields;
- show request and response metadata;
- replay a selected request with approval;
- expose read-only event tools to AI agents by default.

Phase 8 implements this only for explicitly trusted temporary routes on an existing
profile tunnel. Capture is a bounded loopback development proxy: 16 KiB headers,
64 KiB bodies, 100 events per route, and at most 24-hour retention. Replay is
desktop-only, per-event confirmed, and restricted to the original trusted loopback
origin. Binary/file capture, arbitrary targets, public administration, production
traffic, and unredacted storage remain non-goals.

## 10. Product requirements

### PR-001 Shared behavior

Desktop, CLI, and MCP must call shared application services. No interface may implement a separate tunnel, workspace, or session lifecycle.

### PR-002 Existing profile invariant

One profile remains bound to exactly one Cloudflare Tunnel and one API token. Workspace and session concepts are layered above profiles and must not weaken this invariant.

### PR-003 Local-first security

The MCP server is local stdio for the MVP. It must not bind an HTTP listener.

### PR-004 No arbitrary shell

No CLI or MCP operation accepts an arbitrary command string. The only executable runtime command is the reviewed command declared by the workspace manifest and approved by the local trust store.

### PR-005 Secret containment

Secret values remain within the existing FlareDeck secret subsystem. Interfaces may report readiness or missing secret names but never secret values.

### PR-006 Idempotent lifecycle

Repeated start, stop, route verification, and cleanup calls must converge on the same safe state.

### PR-007 Cross-platform lifecycle

Runtime process management must preserve Windows, macOS, Linux, and WSL considerations already present in tunnel management.

### PR-008 Evidence

Every mutating operation produces an audit event and a structured result suitable for acceptance evidence.

## 11. Product acceptance criteria

The MVP is accepted when all of the following are true:

1. A trusted example workspace can be started from the desktop service layer and CLI using the same core implementation.
2. The CLI returns valid JSON for start, status, logs, health, and stop.
3. An untrusted workspace cannot start a runtime through CLI or MCP.
4. Changing the approved command invalidates trust.
5. The MCP server starts over stdio without non-protocol output on stdout.
6. OpenCode and VS Code can discover the MCP tools from documented local configuration.
7. An agent can start a session, receive a public URL, read redacted logs, and stop it without receiving secret data.
8. Repeated stop and cleanup calls are safe.
9. Unit and integration tests cover manifest validation, trust invalidation, session state transitions, command restrictions, output redaction, and MCP tool schemas.
10. Existing profile creation, DNS routing, tunnel start/stop, WSL rewriting, and secret storage behavior remain functional.

## 12. Success metrics

- median time from trusted workspace selection to healthy public URL;
- percentage of session starts completed without manual tunnel or DNS intervention;
- number of orphan routes, child processes, or credentials after failed starts;
- number of security approvals correctly triggered by manifest changes;
- CLI/MCP operation success rate;
- regression count in existing FlareDeck profile and tunnel behavior;
- percentage of AI integration tasks that produce complete start/test/stop evidence.

## 13. Constraints and assumptions

- FlareDeck continues to use Tauri v2, Rust, React, Zustand, and the local `cloudflared` binary.
- Existing Cloudflare API scopes remain the baseline unless a later approved feature requires more.
- The workspace manifest is versioned and contains no secrets.
- Human approval occurs through FlareDeck local state or desktop UX, not through a model assertion.
- The first implementation should remain a single repository and may remain a single Cargo package until shared dependencies justify a workspace split.
- The product is a development tool, not a production runtime manager.

## 14. Product decisions

1. Build CLI before MCP.
2. Extract shared services before adding another interface.
3. Use local stdio MCP for the MVP.
4. Prefer a persistent named tunnel per profile and temporary ingress/routes later, rather than a tunnel per task.
5. Keep AI outside the product boundary.
6. Make trust revocable and content-based.
7. Treat observability and cleanup as product requirements, not implementation polish.
