# FlareDeck

FlareDeck is a local-first desktop control panel for Cloudflare Tunnel development. The desktop, headless CLI, and local stdio MCP server share Rust application services for profiles, trusted workspaces, owned development sessions, health/log observations, temporary routes, and bounded webhook inspection.

## Intended outcome

FlareDeck remains a local-first desktop application. The enhancement adds a shared Rust application-service layer, a machine-readable CLI, workspace and session orchestration, and a local MCP server over stdio. AI agents may then start a trusted development workspace, expose it through an existing Cloudflare Tunnel, retrieve the public URL, inspect health and logs, and stop the session.

The design deliberately excludes an embedded LLM, a general-purpose shell tool, remote multi-user orchestration, and autonomous secret access.

## Package contents

- `PRODUCT-SCOPE.md`: product vision, users, scope, non-goals, outcomes, and acceptance criteria.
- `DOMAIN-MODEL.md`: domain entities, aggregates, invariants, lifecycle, and persistence boundaries.
- `ARCHITECTURE.md`: target architecture, dependency direction, runtime topology, and evolution path.
- `TECHNICAL.md`: implementation contracts for Rust, Tauri, CLI, MCP, manifests, errors, logging, and testing.
- `DESIGN.md`: desktop UX design for workspace, session, trust, health, and logs.
- `PLAN.md`: phased delivery plan with dependencies, exit criteria, risks, and rollback guidance.
- `AGENTS.md`: refined repository-wide instructions for coding agents.
- `docs/adr/`: architecture decisions that must not be silently reversed.
- `docs/specs/`: detailed machine and lifecycle contracts.
- `docs/security/`: threat model, secret handling, and trust/approval rules.
- `docs/implementation/`: migration, testing, observability, and local AI integration guidance.
- `.agents/skills/`: portable Agent Skills following the open `SKILL.md` convention.
- `prompts/`: master, phased, task, review, bug-fix, security, ADR, and release prompts.
- `templates/`: task, feature, review, release, and workspace-manifest templates.
- `examples/`: OpenCode and VS Code local MCP configuration examples.
- `scripts/validate-package.sh`: structural validation for this documentation overlay.

## Development

Install dependencies with `npm ci`. Run the aggregate verification gate with:

Example:

```bash
bash scripts/verify.sh
```

Build a local desktop installer with version-matched companions using `npm run desktop:build`. Release packaging, migration, rollback, and platform evidence are documented in `docs/implementation/RELEASE-HARDENING.md`.

## Source-of-truth order

When documents disagree, use this order:

1. `PRODUCT-SCOPE.md`
2. Approved ADRs in `docs/adr/`
3. `DOMAIN-MODEL.md`
4. `ARCHITECTURE.md`
5. `TECHNICAL.md`
6. Detailed specifications under `docs/specs/`
7. `PLAN.md`
8. Active task specification
9. Existing implementation

The existing implementation is evidence of current behavior, not automatic permission to preserve an architectural mistake.

## Implemented boundary

The repository includes:

- repository verification baseline;
- shared Rust application services;
- headless JSON CLI;
- workspace manifest and trust model;
- development runtime and tunnel session lifecycle;
- local MCP server over stdio;
- OpenCode and VS Code integration;
- desktop workspace/trust/session UX;
- expiring owned temporary routes and bounded redacted webhook inspection;
- version-matched companion packaging and release verification.

## Important security boundary

FlareDeck may execute only a previously reviewed command declared by a trusted workspace manifest. The CLI and MCP interfaces must never accept an arbitrary shell command, raw token, secret value, unrestricted filesystem path, or unrestricted environment dump.
