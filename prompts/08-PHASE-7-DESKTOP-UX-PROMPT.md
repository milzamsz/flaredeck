# Phase 7 Prompt: Desktop Workspace and Session UX

## Objective

Implement the approved desktop design over stable application services. Do not redesign backend contracts to accommodate arbitrary UI convenience.

## Required work

1. Read `DESIGN.md` and inspect current FlareDeck visual patterns.
2. Define safe Tauri display models and TypeScript types.
3. Add workspace navigation and list/detail views.
4. Add trust review with behavior-oriented diff.
5. Add session pipeline, controls, public URLs, health, logs, and audit views.
6. Keep profile management and existing dashboard behavior usable.
7. Use Zustand for shared UI state and local component state only for drafts/disclosures.
8. Add accessibility and narrow-window behavior.
9. Add frontend tests where infrastructure exists and desktop manual scenarios.

## Constraints

- no new UI library;
- no secret display or unredacted mode;
- no generic “connected” status for distinct systems;
- no duplicated session orchestration in frontend;
- no implicit trust approval;
- no automatic persistent route deletion.

## Acceptance criteria

Use all criteria in `DESIGN.md`, plus regression checks for current profile, configuration, settings, updater, and tunnel controls.
