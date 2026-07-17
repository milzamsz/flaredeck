# Phase 7 desktop workspace UX

The desktop now has a lazy-loaded **Workspaces** route and primary navigation entry. A non-persisted Zustand store owns registered workspace, trust, session, log, and audit display state; browser persistence remains limited to theme and active profile.

The list/detail flow distinguishes manifest validation, trust approval, runtime ownership, tunnel ownership, readiness/session state, routes, and public URLs. The trust dialog uses the existing focus-trapped shadcn dialog and displays the canonical path, executable and separate arguments, working directory, environment names and committed non-secret literals, readiness, profile, origins, lifecycle flags, capabilities, and current fingerprint. Changed fingerprints show an explicit warning. There is no implicit approval or unredacted view.

Session controls call thin Tauri adapters over the shared session service. The page includes a text-and-icon pipeline, ownership labels, copy feedback, bounded keyboard-focusable redacted logs, health observations, protected persistent routes, and bounded audit events. Responsive grids stack at narrow widths and the existing collapsible sidebar remains intact.

New Tauri commands: `workspace_list`, `workspace_session_start`, `workspace_session_status`, `workspace_session_stop`, `workspace_session_logs`, and `workspace_audit`. Their response types live in `src-tauri/src/types.rs`, registrations in `lib.rs`, TypeScript types/wrappers in `src/lib/tauriApi.ts`, and callers in `src/store/workspace-store.ts`/`src/pages/Workspaces.tsx`.

Verification: `npm run lint`, `npm run build`, and `bash scripts/verify.sh` pass with 44 Rust/contract tests. Obscura loaded the built `/workspaces` route and confirmed its navigation, main landmark content, controls, live status text, and empty-state copy. Obscura has no layout engine, so pixel-level narrow-window rendering and the native Tauri dialog/session flow remain desktop spot-check items; responsive behavior is covered by the implemented Tailwind breakpoints. No frontend test runner exists, and none was added solely for this phase.

The review also corrected lifecycle parity: `stopRuntimeOnSessionStop: false` now leaves the owned runtime untouched and persisted for safe reuse instead of displaying a promise the service ignored.
