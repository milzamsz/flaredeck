# Prompt — new task intake

Turn a raw idea / feature request / bug into a well-formed task in `backlog`. Opinionated:
state the outcome as observable behavior that must RUN, not a status/row write.

---

```markdown
# NEW TASK INTAKE

## Input (paste the raw request)
`<idea / feature request / bug report, as-is>`

## Steps
1. `@kanban /new "<concise imperative title>"` (e.g. "Add OAuth2 login", not "auth").
2. Fill the task file:
   - **Description** — 1-3 sentences: the outcome, not the solution.
   - **Context** — where it came from, affected area, code refs as `path:line`.
   - **Bug?** repro steps, expected vs actual, environment.
   - **Scope hints** — obvious in/out, known constraints.
   - **Open questions** — anything needing user input before planning.
3. Frontmatter: `priority` (critical/high/medium/low), `labels`, `dependsOn:[<slug>]` if it waits on
   another task (+ a `blocked-by:<slug>` label for board visibility).
4. **Spec-driven?** If it touches a capability with a spec, run `@kanban /spec <capability>` to scaffold
   `changes/<slug>/{proposal,design,tasks}.md` + link the capability `spec`. Add `change:`/`spec:`
   frontmatter to the task.
5. Leave it in `backlog`. Do NOT plan or implement yet.

## OCloud framing (apply at intake)
- State the outcome as observable behavior that must RUN, not a status/row write (the Definition of
  Done will hold the task to this).
- Note org-scoping, audit, and secret-handling implications if the task mutates state.
- Respect frozen routes + public interfaces — new work re-implements behind them.

## Discovered mid-work
Spotted while doing another task -> create it here with label `discovered` +
`Discovered-from: <originating-slug>`, leave in `backlog`, continue current work. Don't pull it in.

## Checklist
- [ ] Title imperative + specific
- [ ] Outcome stated as runnable behavior
- [ ] **Gate awareness:** the `review -> done` gate runs the full production-readiness audit
  ([production-readiness-audit.md](production-readiness-audit.md)). If this task touches queries,
  mutations, secrets, agent commands, or billable resources, the audit will check org-scoping,
  audit events, signed commands, and quota enforcement. Plan for them now.
- [ ] Bug: repro + expected/actual + env
- [ ] Frontmatter + spec link where useful; left in `backlog`

## Next
Standard profile: [stage-backlog-to-planning.md](stage-backlog-to-planning.md).
Lite profile: [stage-backlog-to-inprogress.md](stage-backlog-to-inprogress.md).
```
```
