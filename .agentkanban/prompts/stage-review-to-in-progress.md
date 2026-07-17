# Prompt — `review` -> `in-progress` (revise)

Return every review-rejected task back to active work to apply the requested fixes.

---

```markdown
# SWEEP REVIEW -> IN-PROGRESS

Read AGENTS.md + the task file's review verdict + spec + change `tasks.md` first.

## Scope (fill first)
- Lane: `review` — return all tasks with verdict **revise** (or explicit user request to resume).
- Required Skills: 

## Per revise task (parallel where independent) — action: `implement`
1. `@kanban /task <name>`; set `lane: in-progress`.
2. Confirm the review findings are captured as concrete checklist items in change `tasks.md`.
3. TDD loop on the first unresolved finding: failing test -> make it pass -> refactor. Match
   surrounding code style; never hardcode to the test — implement the general solution.
4. Re-run the verify gate (`` · `` · `` + smoke) before re-review.
5. Mark items done with a one-line note under `### agent` in the task file.

Apply ONLY the requested fixes. If a fix forces a material scope change, note it and return to
`planning` rather than widening silently. Discovered work -> `backlog`. Summary at end.
```

```
