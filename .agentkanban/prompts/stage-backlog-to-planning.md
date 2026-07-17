# Prompt — sweep `backlog` -> `planning`

Clarify + plan EVERY ready task in `backlog`, ending in `planning`. Spec-driven where a capability
spec applies.

---

```markdown
# SWEEP BACKLOG -> PLANNING

Read first (in repo): AGENTS.md (custom rules + DoD), .agentkanban/INSTRUCTION.md (workflow + SDD),
.agentkanban/memory.md. For each task touching a capability, read its
`.agentkanban/specs/<cap>/spec.md` — it is the acceptance contract.

## Scope (fill first)
- Lane: `backlog` — process all ready tasks (a task is ready only when every `dependsOn` is `done`).
- Required Skills: 

## Per ready task (parallel where independent) — action: `plan` (read/think only, NO code)
1. **Discovery** (brainstorming skill): problem & outcome; actors; scope in/out; testable acceptance
   criteria; constraints (org-scoping, security, secrets, perf, ADRs); affected code as `path:line`.
   Unanswerable blocking question -> ask the user, don't guess. Waits on another task -> `dependsOn` +
   `blocked-by:<slug>` -> treat as not-ready.
2. **Implementation plan** (stack skill): approach + key decisions (+ rejected alternatives); data
   model / migration; interface methods + repos touched; security model; **how the behavior is
   proven to RUN** (the verify path); risks + mitigations.
   - Must honor: frozen routes + interface shape; org-scoped queries + audit + tx; no mock fallback
     (fail closed); secrets as references; agent commands typed/allowlisted/signed.
3. **Spec-driven:** create `.agentkanban/changes/<slug>/{proposal,design,tasks}.md`. design.md records
   the chosen approach + decisions + risks (grounded in real `path:line`); tasks.md is the
   authoritative checklist. Add `change:`/`spec:` frontmatter to the task.

**Production-readiness gate:** include in `design.md` how each audit category will be satisfied
   (org-scoping, audit events, secret references, signed commands, quotas, migration idempotency,
   runbooks). The `review -> done` sweep will run the full audit; planning-time coverage prevents
   late failures.
4. **Transition:** set `lane: planning`. Per board policy, high/critical need an **independent-agent**
   planning review (critical = + human) before `planning -> in-progress`.

Ground every claimed fact against code (don't trust prior docs). Record discovered work + end-of-pass
summary. Do not implement.
```
