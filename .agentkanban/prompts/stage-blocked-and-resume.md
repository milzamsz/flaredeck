# Prompt — block / unblock

Block a task that hits a real blocker, or sweep blocked tasks to resume those whose blocker cleared.

---

```markdown
# BLOCK / UNBLOCK

"Real blocker" = something you cannot resolve with available tools/info (external dependency, a
decision only the user can make, broken env, upstream bug, or an unfinished task this one depends on).
If you can resolve it yourself, do — don't park the task.

## Action: `block` (single task)
1. `@kanban /task <name>`; read current state.
2. Under `### agent`, record precisely: what, why, what clears it, who/what must act, what you tried.
3. Waits on another task -> `dependsOn: [<slug>]` + `blocked-by:<slug>` label. External -> plain
   `blocked` label. Keep the task in its working lane.
4. Surface to the user and wait.

## Sweep: resume cleared tasks
1. Worklist: all tasks carrying `blocked` / `blocked-by:<slug>`.
2. Cleared when: every `dependsOn` is `done` (dependency blocker), or the recorded condition is
   resolved (confirm it — don't assume; e.g. "Temporal now reachable", "templates/ added").
3. Cleared -> remove the satisfied labels, note how it cleared, continue via the matching stage driver
   (usually the autonomous `planning -> review`).
4. Still blocked -> leave labels + a one-line status. Watch for dependency cycles; surface them.

Common blockers: a dependency task not yet `done`, an unavailable env (e.g. Temporal/agent/templates),
an upstream bug, or a missing migration. Emit a `resumed` / `still blocked` summary.
```

```
