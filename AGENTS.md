# AGENTS.md

This file applies to the whole `flaredeck` repository. It is written for coding
agents working in this project.

## Mandatory Shell Rule

Always prefix shell commands with `rtk`.

Examples:

```bash
rtk git status
rtk npm run lint
rtk cargo check --manifest-path src-tauri/Cargo.toml
```

`rtk` wraps executable programs, not PowerShell cmdlets directly. When you need
PowerShell cmdlets, run PowerShell through `rtk`:

```powershell
rtk powershell -NoProfile -Command "Get-Content -LiteralPath 'package.json' -Raw"
rtk powershell -NoProfile -Command "Get-ChildItem -Force"
```

The project commonly lives on a WSL UNC path such as
`\\wsl.localhost\Ubuntu\home\milzam\flaredeck`. Avoid `cmd.exe` from that UNC
working directory because it falls back to the Windows directory and loses the
project cwd. Prefer `rtk powershell ...` for Windows-side commands and the
provided scripts for hybrid Windows/WSL development.

## Project Shape

FlareDeck is a desktop GUI for managing Cloudflare Tunnels through the local
`cloudflared` CLI. It is not a Cloudflare API client.

- Frontend: `src/`
  - React 19, Vite, Tailwind v4, shadcn/ui, Zustand, React Router.
  - `src/store/app-store.ts` owns app state and async workflows.
  - `src/lib/tauriApi.ts` is the typed frontend boundary for Tauri commands.
  - `src/lib/yaml-helpers.ts` handles cloudflared YAML parsing, ingress
    serialization, catch-all rules, and WSL host rewriting.
  - `src/components/ui/` contains shadcn primitives; prefer these over adding
    unrelated UI libraries.
- Backend: `src-tauri/`
  - Tauri v2 with Rust command handlers.
  - Commands live under `src-tauri/src/commands/`.
  - Shared serde payloads live in `src-tauri/src/types.rs`.
  - Tauri command registration lives in `src-tauri/src/lib.rs`.
  - Runtime process state lives in `src-tauri/src/state.rs`.
  - User-facing backend failures should flow through `src-tauri/src/error.rs`
    and `AppError`.
- Runtime data:
  - `~/.cloudflared/cert.pem`
  - `~/.cloudflared/flaredeck.json`
  - `~/.cloudflared/<profile-id>.yml`
  - `~/.cloudflared/<profile-id>.yml.bak.*`
  - `~/.cloudflared/<tunnel-uuid>.json`

Do not edit generated or build output unless the user explicitly asks:
`node_modules/`, `dist/`, `dist-windows/`, `src-tauri/target/`, and
`src-tauri/gen/`.

## Implementation Guidance

- Keep TypeScript and Rust command contracts in sync. If you add or change a
  Tauri command, update all relevant places:
  - Rust command handler in `src-tauri/src/commands/`
  - serde types in `src-tauri/src/types.rs`
  - command registration in `src-tauri/src/lib.rs`
  - frontend wrapper/types in `src/lib/tauriApi.ts`
  - Zustand actions or UI call sites as needed
- Preserve the local-CLI architecture. The app should shell out to
  `cloudflared` and read/write local config files; do not introduce Cloudflare
  API-client behavior unless the user explicitly requests it.
- Preserve YAML semantics. Ingress edits should keep the generated catch-all
  rule, retain supported config fields, and use the helpers in
  `src/lib/yaml-helpers.ts` instead of ad hoc string manipulation.
- Preserve WSL behavior. When a profile has `wslHost` enabled, loopback
  services are rewritten to the WSL host IP where the current code expects it.
- Use existing UI patterns. Prefer shadcn primitives from `src/components/ui/`,
  lucide icons, Tailwind utility classes, and `cn()` from `src/lib/utils.ts`.
- Keep persisted frontend state narrow. `zustand` persistence currently stores
  only the theme and active profile id.
- Keep backend errors structured with `AppError` and `AppResult`. Avoid
  returning raw strings from new Rust internals when a typed error fits.
- Be careful with process management. Tunnel lifecycle code supports concurrent
  profile processes, log streaming, restart retries, crashloop protection, and
  platform-specific process termination.
- Be careful with cross-platform paths. Windows, WSL, Linux, and macOS path
  behavior all matter in this repo.

## Common Commands

Use these commands from the repo root, always with `rtk`.

```bash
rtk npm install
rtk npm run dev
rtk npm run lint
rtk npm run build
rtk npm run preview
rtk npm run desktop
rtk npm run desktop:build
rtk cargo check --manifest-path src-tauri/Cargo.toml
```

For Windows + WSL hybrid desktop development:

```powershell
rtk powershell -NoProfile -ExecutionPolicy Bypass -File scripts/desktop-dev.ps1
```

The helper starts Vite inside WSL and Tauri from Windows so the project can use
Linux `node_modules` with the Windows MSVC Rust toolchain.

## Verification Expectations

- Documentation-only changes: verify the file is present and readable.
- Frontend changes: run `rtk npm run lint`; run `rtk npm run build` when types,
  routing, bundling, or shared frontend behavior changed.
- Rust/Tauri backend changes: run
  `rtk cargo check --manifest-path src-tauri/Cargo.toml`; run the desktop flow
  when command wiring, process behavior, tray/window behavior, or filesystem
  interactions changed.
- Cross-boundary changes: run both frontend and Rust checks.
- UI changes: inspect the local app in a browser or desktop shell when feasible,
  especially for responsive layout, dialogs, and shadcn component changes.

If a verification command cannot be run in the current environment, state that
clearly in the final response and explain what blocked it.

## Working Practices

- Read the relevant files before editing. This repo has tight coupling between
  command payloads, store actions, and UI state.
- Keep changes scoped to the user request. Avoid unrelated refactors and build
  artifact churn.
- Do not revert user changes unless the user explicitly asks.
- Prefer `rg` for search:

```bash
rtk rg "tunnel_start" src src-tauri
rtk rg --files -g "!node_modules" -g "!dist" -g "!src-tauri/target"
```

- Use `package-lock.json` with npm. Do not switch package managers unless the
  user asks.
- Keep files ASCII unless an existing file or user-facing copy already requires
  Unicode.
- When adding dependencies, justify the dependency and prefer existing local
  libraries first.
