# Prompt — single-task work driver

Carry one chosen task to **done** without touching other tasks. Uses the `autonomous` workflow:
`backlog → planning → in-progress → review → done`. After reading, implement, verify, and advance per instructions below.

---

```markdown
# SINGLE-TASK WORK DRIVER

Source-of-truth order: AGENTS.md (custom rules + DoD) -> code -> TECHNICAL.md -> .agentkanban/INSTRUCTION.md.
Read AGENTS.md, .agentkanban/memory.md, and the task file + linked artifacts before touching code.
Required Skills: 

## Task

****

Task file: ``

## Workflow

Profile: `autonomous`
Lanes: `backlog → planning → in-progress → review → done`

Autonomous workflow:
- `in-progress` → implementation + verification.
- Set `lane: review` after verification, then continue when all gates and evidence pass.
- `review → done` requires recorded agent reason, passing evidence, and no human-owned blocker.
- Worktree per board policy.

## Instructions

### 1. Read context

Read the task file (``), its linked `change/` artifacts (proposal.md, design.md, tasks.md),
its capability spec, the todo checklist, and `.agentkanban/INSTRUCTION.md`.
Re-read the task's `dependsOn` list. Do not start if a dependency is not in `done`.

### 2. Implement only this task's scope

Move the task to `in-progress`. Implement strictly the approved scope from `tasks.md`.
Reuse existing repository patterns. Honor the required skills (``).

### 3. Verify gate

Run the verification commands:


Check off `tasks.md` items only after verification. Paste real output, not assertions.

### 4. Advance

Autonomous workflow:
- `in-progress` → implementation + verification.
- Set `lane: review` after verification, then continue when all gates and evidence pass.
- `review → done` requires recorded agent reason, passing evidence, and no human-owned blocker.
- Worktree per board policy.

Do not modify or advance other tasks. When done, stop.
```

---

Continue in the task file. Use explicit lane transitions. Record evidence. Never claim pass without output.
