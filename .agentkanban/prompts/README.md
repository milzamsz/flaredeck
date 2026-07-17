# Kanban prompts

Paste-ready stage drivers for AI-assisted development on this board. Opinionated, not stack-agnostic -
tailor the `` / `` / `` / `` placeholders for your project and they
become a repeatable delivery pipeline.

Shared rules live in the repo: `AGENTS.md` (the managed Agentic Kanban block + your custom rules),
`.agentkanban/INSTRUCTION.md` (workflow, lanes, action vocabulary, SDD), and per capability
`.agentkanban/specs/<capability>/spec.md`.

## Flow

`in-progress` is **automatic** - not a human gate. Humans touch only at **plan approval**
(`planning`) and **`review -> done`**. The default driver carries an approved plan straight to `review`.

```
intake -> backlog --[plan]-> planning ==[planning->review: AUTO]==> review --[audit]-> done
                    [GATE:human]   |  in-progress (auto, no touch)       [GATE:human]
                                   +-- blocker? -> label + park ----------+ (stays, resume later)
```

## Default (autonomous)

| Prompt | When |
|---|---|
| [stage-planning-to-review.md](stage-planning-to-review.md) | **one launch** - carry approved planning tasks through `in-progress` to `review`, hands-off; serial (WIP=1); blockers labeled + parked. Runs plan->implement->verify inline. |

## Entry + gates + recovery

| Prompt | When |
|---|---|
| [new-task-intake.md](new-task-intake.md) | raw idea/bug -> well-formed task in `backlog` |
| [stage-backlog-to-planning.md](stage-backlog-to-planning.md) | clarify + plan all ready backlog tasks (spec-driven); ending in `planning` |
| [stage-backlog-to-inprogress.md](stage-backlog-to-inprogress.md) | Lite sweep: clarify + lightly plan all ready backlog tasks, then start them in `in-progress` |
| [stage-review-to-done.md](stage-review-to-done.md) | finalize approved tasks (human gate; runs the production-readiness audit) |
| [stage-review-to-in-progress.md](stage-review-to-in-progress.md) | revise: send a rejected task back to implementation |
| [stage-inprogress-to-done.md](stage-inprogress-to-done.md) | Lite sweep: review, verify, and close every task in `in-progress` |
| [stage-blocked-and-resume.md](stage-blocked-and-resume.md) | block one / sweep blocked to resume cleared |
| [production-readiness-audit.md](production-readiness-audit.md) | gate run by `review->done` (evidence the behavior RUNS, not a status write) |
| [work-on-task.md](work-on-task.md) | single-task driver; used by `/work` (pick a not-done task, copy interpolated prompt to clipboard) |
| [goal-decompose.md](goal-decompose.md) | goal decomposition driver; used by `/goal new` (copy to clipboard after epic + artifact are scaffolded) |

> The middle steps (`planning->in-progress`, `in-progress->review`) are not separate prompts -- the
> autonomous driver does them inline. Use the granular flow only if you split it out later.

## Project verify gate (every driver reuses)

Every implementation driver runs this gate before advancing. Paste **real output**, not assertions.

```bash
   # e.g. go vet, eslint, ruff
   # e.g. go test, npm test
  # e.g. go build, tsc, vite build
# plus any project-specific smoke checks
```

"Done" = behavior proven to RUN (test output / real run / workflow or job id), not a DB row. Respond
IN the task file; explicit lane transitions only; record evidence; never claim pass without output.

These files are bundled by the extension and (re)written by `@kanban /prompts`. Edit freely - your
copies are preserved on init; `/prompts` overwrites to the latest bundled versions.

`@kanban /loop [lane]` emits the stage-driver prompt for the selected lane into chat. A **"Send prompt to chat"**
button in the response injects it directly into the chat input (one click, press Enter to run). Clipboard copy
included as fallback. Default lane is `backlog`. Lane-to-prompt mapping (Standard): `backlog` -> `stage-backlog-to-planning`,
`planning`/`in-progress` -> `stage-planning-to-review`, `review` -> `stage-review-to-done`; (Lite): `backlog` ->
`stage-backlog-to-inprogress`, `in-progress` -> `stage-inprogress-to-done`. Use `@kanban /prompts` to pick a prompt manually.
