# Agentic Kanban - Instruction

You are working with the **Agentic Kanban** extension.

IMPORTANT: Follow these workspace structure, file format, and workflow rules strictly.

Agentic Kanban structures AI-assisted delivery around fixed workflow profiles. Conversation between you (the agent) and the user happens in task files. Use the chat window only for summaries and lightweight coordination. Planning decisions, reviews, and implementation notes belong in the task file so the workflow remains auditable.

IMPORTANT: Always respond in the task file, not the chat window. Stay in the assigned task file until a new one is given.

## Task Directory Structure

```text
.agentkanban/
  .gitignore          # Auto-generated - ignores logs/
  board.yaml          # Workflow profile, lanes, and policy config
  goals/
    <goal-slug>/
      goal.md              # goal artifact (objective, acceptance criteria, metrics, decomposition)
  specs/
    <capability>/spec.md   # capability contract — one per capability, shared across tasks
  changes/
    <task-slug>/
      proposal.md
      design.md
      tasks.md
    archive/<task-slug>/   # archived change folders (via @kanban /archive)
  memory.md           # Persistent memory across tasks (reset via command)
  INSTRUCTION.md      # This file - managed by the extension
  tasks/
    task_<id>_<slug>.md    # Task files (lane stored in frontmatter)
    todo_<id>_<slug>.md    # Checklist artifact for the task
    archive/               # Archived tasks (hidden from board)
  logs/               # Diagnostic logs (gitignored)
```

## Task File Format

IMPORTANT: The task lane is managed by the user and extension via frontmatter. Do not change the lane implicitly. Use explicit transitions.

Each task is a markdown file with YAML frontmatter. Conversation flows under `### user` and `### agent` headings. The user may add inline comments `[comment: <text>]` on your responses. Check for these before continuing.

```markdown
---
title: <Task Title>
lane: <lane-slug>
created: <ISO 8601>
updated: <ISO 8601>
description: <Brief description>
labels: [blocked, blocked-by:<slug>]    # optional blocker labels
dependsOn: [<slug>]                     # task dependencies (synced with blocked-by labels)
evidence:                               # optional — recorded via @kanban /evidence
  lint: {ran: true, passed: true}
  test: {ran: true, passed: true}
  build: {ran: true, passed: true}
  behavior: {ran: true, passed: true}
goal: .agentkanban/goals/<slug>         # optional — goal artifact dir (set on epic tasks by /goal new)
parent: <goal-slug>                     # optional — parent/goal slug (set on child tasks by /goal)
superseeds: [<slug>]                    # optional — slugs of tasks this supersedes
blockerResolved: true                   # optional — set when a blocker is cleared
---

## Conversation

### user

<message>

### agent

<response> [comment: <inline comment by the user>]

### user
```

Rules:

- Append new entries at the end. Never modify or delete existing conversation entries.
- Start each message with `### user` or `### agent` on its own line, with a blank line between messages.
- After your response, add `### user` on a new line for the user's next entry.
- Honor inline `[comment: <text>]` annotations from the user.
- Re-read this `INSTRUCTION.md` at the start of every action.
- Confirm which task file you are working in at the start of each response. If none is in context, ask the user to select one with `@kanban /task`.
- If no task file reference is found, ask the user to run `@kanban /task` or `@kanban /refresh`.

## Checklist Artifact

Track execution details with `- [ ]` / `- [x]` checkboxes in the corresponding `todo_*.md` file. This is a checklist artifact, not a workflow lane.

```markdown
# Iteration <number>

- [ ] Uncompleted item
- [x] Completed item
```

Rules:

- Create or update the checklist during planning or implementation when work needs explicit steps.
- Add new checklist items to the bottom of the current iteration.
- Mark completed items during or immediately after implementation.
- For spec-driven tasks linked through `change: .agentkanban/changes/<task-slug>`, use the change `tasks.md` as the authoritative checklist instead of the sibling `todo_*.md`.

## Definition of Done Checklist

When the board policy `requireDoneChecklistForDone` is on (standard profile default), the `review -> done` transition requires a `## Definition of Done` section in the task body. This section is separate from the planning checklist (`todo_*.md` / `tasks.md`).

Rules:

- The section heading must be exactly `## Definition of Done`.
- Items use `- [ ]` / `- [x]` syntax, same as the planning checklist.
- Each item may carry an `(agent)` or `(human)` owner tag: `- [x] (agent) Tests pass` or `- [ ] (human) Release sign-off`.
- Untagged items default to `agent` owner.
- Human-owned items require a human actor to clear (actor must be in `enforcement.overrides.actors` as `human`).
- All items must be checked for the task to move to `done`.
- The board webview renders a `DoD` progress badge on cards with this section.

## Spec-Driven Development

Some tasks opt into spec-driven development. They carry two frontmatter keys: a `change` pointing at
`.agentkanban/changes/<task-slug>` (the per-task proposal/design/tasks) and a `spec` pointing at the
shared capability contract `.agentkanban/specs/<capability>/spec.md`.

For those tasks:

- Read the capability `spec.md` (behavior + acceptance criteria) and `proposal.md` before planning or implementation.
- In the Standard profile, also read `design.md`.
- Treat the change `tasks.md` as the authoritative checklist (not the sibling `todo_*.md`).
- The capability spec lives once under `.agentkanban/specs/` and is referenced, not duplicated per change.
- Preserve the task-to-change and task-to-spec links in frontmatter.
- A task is `done` only when its behavior is proven to run and the spec's acceptance criteria are met.
- When the task reaches `done`, archive the change with `@kanban /archive [slug]`; the capability spec stays in `specs/`. Archiving and validation remain agent-driven.

## Memory

`.agentkanban/memory.md` persists across tasks. Read it at the start of each task. Update it with project conventions, key decisions, and useful context.

## Technical Document

Maintain `TECHNICAL.md` at workspace root with implementation details for agents and humans. Update the relevant section when behavior or workflow rules change.

## Multi-Root Workspaces

When multiple workspace folders are open in VS Code:

- treat each folder as an independent project with its own `.agentkanban/` state
- operate on the active project selected in the board UI
- do not create `.agentkanban/` in an uninitialised folder unless that specific folder is initialised
- keep each project's chosen profile and board policy isolated to that folder

## Workflow Profiles

### Lite profile

Lane flow:

`backlog -> in-progress -> done`

Guidance:

- `backlog` is for rough work items that still need lightweight clarification.
- `in-progress` is where implementation happens.
- `done` is for completed work.
- `/loop [lane]` is a lane-flow prompt driver for Lite too. Default lane is `backlog`. Lane-to-prompt mapping: `backlog` -> `stage-backlog-to-inprogress`, `in-progress` -> `stage-inprogress-to-done`.

### Standard profile

Lane flow:

`backlog -> planning -> in-progress -> review -> done`

Guidance:

- `backlog`: the task is still broad and not ready for detailed execution.
- `planning`: refine scope, write the implementation plan, identify risks, and update the checklist artifact as needed. This is where the plan is **approved**. To transition a task from `planning` to `in-progress`, it must have a checklist with at least one item, and if it is spec-driven, it must also have a valid spec file and change folder.
- `in-progress`: implement the approved plan. Entering `in-progress` is **not** a separate human gate — an agent may carry an approved task from `planning` straight through implementation to `review` in one pass (the autonomous `planning -> review` flow). When running this flow, the task must be moved to `in-progress` before starting work, so the board reflects the current progress state. Worktrees are optional unless the board policy requires them.
- `review`: implementation review. Return to `in-progress` for revisions, or move to `done` when approved.
- The two human gates are **plan approval** (in `planning`) and **`review -> done`**. Everything between can run hands-off. Before moving to `done`, all four evidence checks (lint, test, build, behavior) must be recorded as passing via `@kanban /evidence`. When `requireDoneChecklistForDone` is on, the task body must also contain a `## Definition of Done` section with all items checked; items tagged `(human)` require a human actor.
- `/loop [lane]` is a **lane-flow prompt driver**: it emits the stage-driver prompt for the selected lane into chat. Click the **"Send prompt to chat"** button in the response to inject it directly into the chat input (clipboard copy as fallback). Default lane is `backlog`. Lane-to-prompt mapping: `backlog` -> `stage-backlog-to-planning`, `planning`/`in-progress` -> `stage-planning-to-review`, `review` -> `stage-review-to-done`. The agent driven by that prompt performs the actual work and board moves; gates are enforced when the agent moves a task via the board UI. `/prompts` is the general manual prompt picker.
- Real blockers → keep the task in its lane. Use the task's `dependsOn` frontmatter array or `blocked-by:<slug>` labels to document task dependencies, and the `blocked` label for external blockers. The board webview and backend sync these fields bidirectionally. Record what clears the blocker and never force past a real blocker.
- One active implementation task at a time (WIP) unless the board allows otherwise.

## Action Vocabulary

The user may instruct you with these action words:

| Action | Meaning |
| --- | --- |
| `plan` | Clarify requirements, refine scope, and write or update the implementation plan |
| `checklist` | Create or update the TODO checklist artifact |
| `implement` | Carry out the approved implementation work |
| `review` | Perform or prepare an implementation review |
| `block` | Record blockers on the task and add `blocked` or `blocked-by:<slug>` labels |
| `unblock` | Remove blocker labels once the task can continue |

Treat `TODO` as the checklist artifact only. Do not treat `todo` as a workflow lane.

## Chat Commands Reference

| Command | Usage | Purpose |
| --- | --- | --- |
| `/new` | `@kanban /new <title>` | Create task in backlog |
| `/task` | `@kanban /task <name>` | Select task, inject context |
| `/refresh` | `@kanban /refresh` | Re-inject workflow context |
| `/spec` | `@kanban /spec [capability]` | Scaffold spec-driven artifacts |
| `/worktree` | `@kanban /worktree` | Create git worktree for task |
| `/archive` | `@kanban /archive [slug]` | Archive completed change folder |
| `/prompts` | `@kanban /prompts` | Write/refresh bundled stage prompts |
| `/loop` | `@kanban /loop [lane]` | Emit stage-driver prompt for a lane into chat; default lane: `backlog` |
| `/goal new` | `@kanban /goal new <objective>` | Define a goal: epic card + artifact + decompose prompt |
| `/goal` | `@kanban /goal` | Goal progress dashboard |
| `/goal show` | `@kanban /goal show <slug>` | Detail view for a specific goal |
| `/doctor` | `@kanban /doctor` | Run workflow diagnostics |
| `/work` | `@kanban /work [task]` | Copy task work prompt to clipboard |
| `/evidence` | `@kanban /evidence [task] [check] [pass\|fail]` | View or record task evidence |

## Execution Rules

- Do not implement changes unless the task is in `in-progress`. When running the autonomous `planning -> review` flow on an approved task, you must move the task to `in-progress` before starting work.
- Do not move a Standard-profile task to `done` without implementation review.
- Respect WIP limits from `board.yaml` (`wipLimits.in-progress: 1` default for Standard). A move into a full lane is blocked (strict mode) or warned (warn mode).
- Preserve blocker context in the task file when adding or clearing blocker labels.
- If the user wants to override the workflow, note the reason in the task file.
- Do not add or commit to version control unless specifically instructed.

## Goals

`@kanban /goal new <objective>` creates a goal epic task (with `goal` and `epic` labels) plus a goal artifact at `.agentkanban/goals/<slug>/goal.md`. It copies a decompose prompt to clipboard so you can break the goal into child tasks.

Child tasks carry `parent: <goalSlug>` in frontmatter. The epic carries `goal: .agentkanban/goals/<slug>`.

Goal tier: **goal (epic) > capability spec > change > task**. In Standard profile, complex child tasks should use `/spec` to attach a capability spec. In Lite, children use lightweight proposal + tasks only.

After decomposition run `@kanban /loop backlog` to get the backlog-to-planning driver prompt for child tasks. Use `/loop <lane>` as you advance through each lane stage.

`@kanban /goal` shows a progress dashboard. `@kanban /goal show <slug>` shows next-ready and blocked children.

## Worktree Guidance

In a worktree workspace, `AGENTS.md` permanently contains the task reference, so `/task` and `/refresh` are usually unnecessary.

When the profile or policy requires worktrees for implementation:

- ensure the task has worktree metadata before implementation starts
- keep implementation activity inside the worktree
- preserve the task file and checklist file as the workflow record

## Board Policy

The active board policy is injected through `AGENTS.md`.

- `enforcement.mode: strict` means transition failures block unless a human override is used.
- `enforcement.mode: warn` means transition failures are surfaced as warnings and the move can continue.
- `reviewPolicy` tells the agent who should review planning and implementation work for each task priority.
- `self-agent` means the active coding agent can perform that review.
- `independent-agent` means a different agent should review that stage.
- `independent-agent+human` means a different agent review plus human review is required.

## Project Skills

Project skills are loaded from the workspace first and then from machine-level installations:

- **Project-local active skills:** `project/.agents/skills/`, `project/skills/`, and `project/.claude/skills/`
- **Machine-installed skills:** `~/.agents/skills/`, `~/.codex/skills/`, `~/.claude/skills/`, `~/.antigravity/skills/`, plus `agentKanban.skillsDirs`
- **Prompt variables:** `{{skills}}` is resolved from the active project skills and injected into scaffolded prompts and the managed `AGENTS.md` sentinel

Checking a machine-installed skill in Settings links it into `project/.agents/skills/` so the project owns the active skill set.
