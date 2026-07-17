# Prompt — autonomous `planning` -> `review` (default driver)

One launch carries every **approved** planning task through implementation to `review`, hands-off.
`in-progress` is automatic — no human touch between plan approval and the review gate. Real blockers
are labeled and parked. Serial: one task at a time (WIP = 1).

Human gates: **plan approval** (in `planning`, before you launch) and **`review -> done`** (after).

---

```markdown
# AUTONOMOUS PLANNING -> REVIEW

Source-of-truth order: AGENTS.md (custom rules + DoD) -> code -> TECHNICAL.md -> .agentkanban/INSTRUCTION.md.
Read AGENTS.md, .agentkanban/memory.md, and per task its capability spec + change `design.md`/`tasks.md`
before touching code.
Required Skills: 

## Scope (fill first)
- Lane: `planning` — process approved + ready tasks only.
- Approved = a human go is recorded in the task (or you name the tasks when launching). If a task has
  no approval and you weren't told to run it, skip it.
- Ready = every `dependsOn` slug is `done`. Otherwise it stays (guardrail).
- WIP = 1, **serial**: fully finish or park one task before starting the next. Topo-sort by `dependsOn`.

## Per approved+ready task (serial) — carries it from planning to review
1. **Enter implementation (automatic).** Set `lane: in-progress` before starting work, so the board reflects the current progress state. Confirm toolchain green first (`` · `` · ``). No human approval here — launching this driver is the authority.
2. **Re-read** the capability spec (`spec:` frontmatter) + `changes/<slug>/{design,tasks}.md`. If the
   design materially diverges from current code, set `lane: planning`, add a `blocked` label with the reason, **park**, note why (do not widen scope silently), and move to the next task.
3. **Plan-review gate (high/critical only).** If the task has no independent planning-review verdict
   recorded, spawn an **independent reviewer agent** to vet the plan against the code.
   - `revise` -> write the findings into the task, set `lane: planning`, add a `blocked` label with the reason, **park**, next task.
   - `approve` -> continue. (low/medium: skip — self-review is enough.)
4. **Implement** TDD down `changes/<slug>/tasks.md`: failing test -> pass -> refactor; check items off
   with a one-line `### agent` note each. Match surrounding code style; never hardcode to a test.
   Honor the guardrails: org-scoped query + audit + tx on mutations; no mock fallback (fail closed);
   lifecycle/provisioning must dispatch real work; secrets reference-backed + never logged;
   agent commands typed/allowlisted/signed; frozen routes + interfaces intact.
5. **Verify gate (paste real output):**

   Plus **evidence the behavior RUNS** (the spec's Verification proof — a real workflow/job id,
   an agent command that executed, an HTTP 429 on quota breach, an S3 object), not a DB row.
6. **Blocker handling.** A real blocker = something you cannot resolve with available tools/info
   (missing dependency task not `done`, env unavailable, upstream bug, or a decision only the user
   can make). On a real blocker: add `blocked` (external) or `blocked-by:<slug>` (task dep), record
   what blocks + what clears it in the task, **park**, move to the next task. Do not force or fake
   progress. If you can resolve it yourself, do — it's not a blocker.
7. **Success -> review.** Set `lane: review`. Write the implementation-review verdict + the pasted
   evidence in the task file. **STOP here** — `review -> done` is the human gate (critical also needs
   independent-agent + human per board policy). Never push a task to `done` yourself.

## End-of-run summary (in chat)
Per task: `advanced-to-review` / `parked-blocked:<reason>` / `revise-parked:<reason>` / `skipped:<reason>`.
Discovered work -> a new `backlog` task (label `discovered` + `Discovered-from:<slug>`); never pull it into
   the current pass.

## Notes
- This driver runs the `planning -> in-progress -> review` steps **inline** — there are no separate
  prompts for those middle transitions.
- Blockers use [stage-blocked-and-resume.md](stage-blocked-and-resume.md) (label + park; resume when cleared).
- A review rejection is sent back to implementation via [stage-review-to-in-progress.md](stage-review-to-in-progress.md).
- `review -> done` is run separately by a human via [stage-review-to-done.md](stage-review-to-done.md).
```
