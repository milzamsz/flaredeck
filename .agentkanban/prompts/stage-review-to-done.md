# Prompt — `review` -> `done` (human gate)

Finalize every task whose implementation review passed. This is a human gate.

---

```markdown
# REVIEW -> DONE

Read AGENTS.md (Definition of Done) + .agentkanban/INSTRUCTION.md first.

## Scope
- Lane: `review` — finalize tasks with an **approved** verdict. Never skip review. Per board
  reviewPolicy, high/critical need independent (+ human for critical) sign-off recorded in the task.

## Per approved task
1. **Production-readiness gate (required):** run [production-readiness-audit.md](production-readiness-audit.md)
   and paste the PASS/FAIL report into the task. Any unresolved FAIL on correctness / security /
   reliability blocks `done` — fix, or `block` with a reason. Mark untested checks `not-run`.
2. **Definition of Done checklist (required if board policy `requireDoneChecklistForDone` is on):**
   The task body must contain a `## Definition of Done` section with all items checked.
   Items tagged `(human)` require a human actor to move to `done` (actor must be in
   `enforcement.overrides.actors` as `human`).
3. **Update honest-state docs** (this is part of done, not optional): update any README / architecture /
   spec notes, and any status banners that the change made real. The board must stop overclaiming.
4. **Spec-driven:** confirm the capability `spec.md` reflects the shipped behavior. Archive the change
   folder with `@kanban /archive <slug>` (capability spec stays). The change folder stays as the
   record.
5. **Release/handover — only what the user asks:** commit/tag/deploy only when instructed; if on the
   default branch, branch first. Summarize what shipped, how verified, follow-ups.
6. **Finalize:** set `lane: done`. Moving to `done` unblocks downstream tasks whose `dependsOn` now
   clear — flag them for the next pass.

State plainly what shipped, verified, skipped. Discovered work -> `backlog`. Summary at end.
```

