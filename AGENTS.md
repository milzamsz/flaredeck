# AGENTS.md

This file applies to the whole `flaredeck` repository. It is written for coding
agents (LLM-driven dev assistants) working in this project.

## Project Shape

FlareDeck is a desktop GUI for managing Cloudflare Tunnels. It is a **hybrid
client**: the control plane (zone lookup, tunnel creation, DNS routing) goes
through the **Cloudflare REST API** with a per-profile token; the data plane
(actually running the tunnel) is the local `cloudflared` CLI invoked as a
child process.

**Hard rule:** 1 profile = 1 Cloudflare Tunnel = 1 API token. Multi-tunnel
profiles are not supported and that's deliberate.

### Layout

```
src/                       React 19 + Vite + Tailwind v4 + shadcn/ui + Zustand
├── store/app-store.ts     Zustand store + async workflows
├── lib/
│   ├── tauriApi.ts        Typed boundary to Rust commands; helpers (routeDnsForProfile, normaliseDomainInput); CF_TOKEN_CREATE_URL
│   └── yaml-helpers.ts    Ingress parsing, serialization, catch-all rule, WSL host rewriting
├── components/
│   ├── ui/                shadcn primitives — prefer these over new UI libs
│   ├── app-sidebar.tsx    Sidebar with profile list, "+" trigger, API status badge
│   ├── proxy-table.tsx    Ingress rules table
│   └── proxy-form-dialog.tsx   Add/edit route, with Advanced (path) disclosure
└── pages/
    ├── Dashboard.tsx      Profile detail + NewProfileDialog (Name + Token + Domain)
    ├── Config.tsx         CodeMirror YAML editor
    └── Settings.tsx       cloudflared install/version, WSL, profile list, CredentialsCard, window, theme

src-tauri/                 Tauri v2 + Rust
├── Cargo.toml             keyring, reqwest, chacha20poly1305, base64, sha2, rand, tokio, sysinfo, hickory-resolver, …
├── tauri.conf.json
└── src/
    ├── lib.rs             App setup, tray, single-instance, tauri::generate_handler! registrations
    ├── cf_api.rs          CfClient — Cloudflare REST surface (verify, lookup, create_tunnel, preflight, upsert_dns_route) + scope-aware error hints
    ├── cloudflared.rs     cloudflared binary discovery + cert.pem path resolution
    ├── secrets.rs         Per-profile API tokens — primary: OS keychain; fallback: machine-key-encrypted file (ChaCha20-Poly1305)
    ├── state.rs           Runtime child-process state per profile
    ├── error.rs           AppError + serde Serialize for cross-boundary errors
    ├── types.rs           Shared serde payloads (Profile, ProfileIndex, ProfilePatch, …)
    └── commands/          tauri::command handlers
        ├── cf.rs          cf_route_dns, cf_lookup_zone, create_tunnel_with_files (internal)
        ├── config.rs      config_get, config_save, write_initial_config
        ├── dns.rs         hickory-resolver wrapper
        ├── network.rs     TCP probe for origin port checks
        ├── prefs.rs       App prefs (minimize-to-tray, etc.)
        ├── profiles.rs    profiles_list/update/delete/set_active/set_token/clear_token/verify_token/create_simple
        ├── shell.rs       shell_open_external, shell_open_path (defense-in-depth wrappers)
        ├── tunnel.rs      cloudflared_check, tunnel_status/start/stop/restart, tunnel_route_dns (CLI fallback)
        └── wsl.rs         Detects WSL VM IP for the host-rewrite feature
```

### On-disk runtime data

- `~/.cloudflared/flaredeck.json` — the profile index (no secrets)
- `~/.cloudflared/<profile-id>.yml` — cloudflared config per profile
- `~/.cloudflared/<tunnel-uuid>.json` — cloudflared credentials per tunnel
- `~/.cloudflared/cert.pem` — global cert from `cloudflared tunnel login` (only used by profiles created via the legacy CLI path or when `TUNNEL_ORIGIN_CERT` is read by cloudflared at run time)
- `~/.cloudflared/flaredeck.secrets` — encrypted-file fallback for API tokens (only created when keychain is unavailable, e.g. WSL without `gnome-keyring`)
- **API tokens themselves never live on disk in plaintext.** They live in the OS keychain (service `flaredeck`, account = profile UUID) or in the AEAD-encrypted fallback file.

Generated/build output to leave alone unless the user explicitly asks:
`node_modules/`, `dist/`, `dist-windows/`, `src-tauri/target/`, `src-tauri/gen/`.

## Tauri Command Surface

Every command registered in `lib.rs:invoke_handler!` MUST have a matching wrapper in `src/lib/tauriApi.ts`. The shape and naming must agree (Rust snake_case ↔ JS camelCase via serde's `rename_all = "camelCase"`).

When adding or modifying a command, update **all five** in the same change:
1. Rust handler in `src-tauri/src/commands/`.
2. Shared serde type in `src-tauri/src/types.rs` (or local to the command file if it's only used there).
3. Registration in `src-tauri/src/lib.rs:invoke_handler!`.
4. Frontend wrapper + type in `src/lib/tauriApi.ts`.
5. Caller — Zustand action in `src/store/app-store.ts` or a component.

Skipping any of these will surface as either a frontend-only TS error or a silent Tauri "command not found" at runtime.

## Cloudflare Integration

### The flow

1. User creates a profile via `profiles_create_simple(name, token, domain, …)`.
2. Backend stores the token in the keychain, calls `CfClient::lookup_zone_by_domain` to resolve account+zone, runs `CfClient::preflight_cfd_tunnel_scope` to verify the token has `Cloudflare Tunnel: Edit` before mutating any state, then calls `CfClient::create_tunnel` (POST `/accounts/{id}/cfd_tunnel`). The 32-byte tunnel secret is generated client-side and written into `<uuid>.json` so cloudflared can run the tunnel.
3. Profile entry lands in `flaredeck.json` with `account_id`, `zone_id`, `zone_name`, `has_api_token: true`.
4. Adding routes goes through `routeDnsForProfile` (in `tauriApi.ts`), which dispatches to `cf_route_dns` (API) when the profile has a token + zone, or `tunnel_route_dns` (CLI) otherwise.
5. Starting a tunnel runs `cloudflared tunnel run` with `TUNNEL_ORIGIN_CERT` set to the profile's effective cert.

### Token scopes

A working FlareDeck token needs three permission groups. The template URL `CF_TOKEN_CREATE_URL` pre-ticks them, but Cloudflare's dashboard sometimes drops the prefill — the Settings credentials card and New Profile dialog list them inline as well:

- `Account → Cloudflare Tunnel: Edit`
- `Zone → Zone: Read`
- `Zone → DNS: Edit`

If you change which Cloudflare endpoints the app calls, audit `hint_for(ApiCall, errors)` in `cf_api.rs` — it produces user-facing hints by `ApiCall` variant, and those hints name specific scopes.

### CLI fallback

Some commands still exist for legacy / fallback use:
- `tunnel_route_dns` — used by `routeDnsForProfile` when the profile has no API token; needs `cert.pem`.
- `effective_cert_path(&Profile)` — resolves which cert.pem `cloudflared` should use when spawning the tunnel. The `Profile.cert_path` field exists but isn't surfaced in the UI; if set, it wins.

There is no "global login" UI any more. We don't expose `auth_check/auth_login/auth_logout`; if you need to add login UX back, you'd have to re-add those commands.

## Implementation Guidance

- **Don't introduce new Cloudflare API endpoints without a `hint_for` branch.** Auth errors look identical across endpoints (Cloudflare returns 10000 for "missing scope" *and* "bad token"); the only thing that disambiguates them is which call was being made.
- **Don't mutate disk before pre-flight on new wizard-like flows.** `profiles_create_simple` is the model: zone lookup, then pre-flight, then state mutation. Otherwise a bad token leaves orphan files.
- **Preserve YAML semantics.** Ingress edits go through `src/lib/yaml-helpers.ts`. The catch-all rule (`service: http_status:404`) is appended automatically; keep it.
- **Preserve WSL host rewriting.** When `Profile.wslHost` is true, loopback service URLs get rewritten to the WSL VM IP on save. See `yaml-helpers.ts` and `commands/wsl.rs`.
- **Errors:** use `AppError` / `AppResult`. Never `panic!` in a Tauri command — `AppError::serialize` produces a string the frontend toasts.
- **State management:** the Zustand store is keyed off `activeProfileId`; switching profiles reloads config + tunnel status. New state generally goes in the store, not React component state, unless it's purely local UI (form drafts, disclosure open/close).
- **Persisted state stays narrow.** Zustand `persist` middleware currently only saves `theme` and `activeProfileId`. Don't add tokens, profile lists, or other server state here.
- **UI patterns:** shadcn from `src/components/ui/`, lucide-react icons, `cn()` from `src/lib/utils.ts`, Tailwind utilities. Don't add new UI libraries casually.
- **Cross-platform paths matter.** This runs on Windows, macOS, Linux, and WSL. Use `std::path::PathBuf` and `dirs::home_dir()`; don't hardcode `/`.
- **Process lifecycle is careful for a reason.** `commands/tunnel.rs` handles concurrent profile processes, log streaming, crashloop detection (3 fails in 30s), platform-specific kill (`taskkill /T /F` on Windows, `kill -TERM` on Unix). Don't simplify these without understanding why.

## Releases & Updates

- **Release pipeline**: `.github/workflows/release.yml` builds Windows /
  macOS / Linux on `git tag v*`, uploads to GitHub Releases, and pushes a
  merged `latest.json` into the `flaredeck-web` repo's `public/`. See
  `RELEASING.md` for the end-to-end procedure including secret setup.
- **In-app updater**: `tauri-plugin-updater` + `tauri-plugin-process` are
  wired in `lib.rs`. The hook in `src/lib/updater.ts` exposes
  `useUpdater()` to the UI (used by `UpdateCard` in `Settings.tsx`). The
  signing pubkey lives in `tauri-tauri.conf.json` under
  `plugins.updater.pubkey`; never rotate it without breaking installed users.
- **Download page**: reference React component + plain-HTML version at
  `docs/website/`. They link to GitHub Releases via stable
  `/releases/latest/download/<asset>` URLs and read version from
  `/latest.json` on the same origin.

## Common Commands

From the repo root:

```bash
npm install
npm run dev                  # web preview (most Tauri commands are no-ops here)
npm run desktop              # tauri dev (real backend, real GUI)
npm run lint                 # eslint
npm run build                # tsc -b + vite build
npm run desktop:build        # tauri build (produces native bundle)

cargo check   --manifest-path src-tauri/Cargo.toml
cargo clippy  --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test    --manifest-path src-tauri/Cargo.toml --lib cf_api::
```

### Windows .exe / installer via cross-compile from WSL

Requires (one-time): `sudo apt-get install -y lld clang nsis`,
`cargo install cargo-xwin --locked`,
`rustup target add x86_64-pc-windows-msvc`.

```bash
PATH="/usr/lib/llvm-18/bin:$PATH" \
  pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
```

Artifacts at:
- `src-tauri/target/x86_64-pc-windows-msvc/release/flaredeck.exe`
- `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/FlareDeck_<version>_x64-setup.exe`

The `PATH` prefix is required so `cc-rs` can find `llvm-lib` and `llvm-rc` by their unsuffixed names.

### Windows + WSL hybrid dev

When the project lives on a WSL UNC path (`\\wsl.localhost\Ubuntu\…`) and you want to run the desktop shell from Windows:

```powershell
pwsh scripts/desktop-dev.ps1
```

Vite runs inside WSL with the project's Linux `node_modules`; Tauri builds from Windows with the MSVC toolchain. Install the CLI once if missing: `npm install -g @tauri-apps/cli@^2.11`.

## Verification Expectations

- **Documentation-only changes:** verify the file is present and readable. No build needed.
- **Frontend-only changes:** `npm run lint` always; `npx tsc -b` when types changed; `npm run build` when bundling-affecting changes (router, lazy imports, env).
- **Rust/Tauri changes:** `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (we keep warnings clean); `cargo test` if you touched `cf_api.rs` (the domain normaliser + apex walker are unit-tested).
- **Cross-boundary changes:** run both. Specifically check that a new field added to `Profile` in Rust has the matching field in `src/store/app-store.ts:Profile` and `src/lib/tauriApi.ts:ProfilePatch`.
- **UI changes:** spot-check in a browser (`npm run dev`) where you can. Tauri-gated invocations short-circuit there via `isTauri()`; full flow needs `npm run desktop`.
- **Wizard / token flow:** if you touched `profiles_create_simple`, `cf_api.rs`, or `hint_for`, test with both a fully-scoped token (happy path) and an under-scoped token (error path). The pre-flight should fail cleanly without orphaning files on disk; verify `ls ~/.cloudflared/ | wc -l` before and after.

If a verification command can't run in your environment, say so explicitly in your final response.

## Working Practices

- **Read before editing.** The store / commands / UI are tightly coupled. Skim the related slice + wrapper + handler before making a change.
- **Scope changes to the request.** Don't refactor unrelated code on the side. Don't churn `dist/` or `src-tauri/target/`.
- **Don't revert user changes** unless the user explicitly asks.
- **Search with `rg`** rather than `grep -r` — faster, respects `.gitignore`:
  ```bash
  rg "profiles_create_simple" src src-tauri
  rg --files -g "!node_modules" -g "!dist" -g "!src-tauri/target"
  ```
- **npm with `package-lock.json`** — don't switch to pnpm/yarn without permission. (The cross-compile command above uses `pnpm` only because that's what's installed in the dev container; `npm run tauri build …` works equivalently if you have npm.)
- **ASCII by default** for code and comments. Existing user-facing copy may use Unicode (e.g. "→" in scope hints); keep it consistent.
- **New deps need justification.** Check what's already in `package.json` / `Cargo.toml`. We pulled in `chacha20poly1305` for a real reason (secrets fallback) — don't add crypto for cosmetic reasons.
- **Don't bypass the keychain.** `secrets::store_token` is the only sanctioned token-write path. If it fails, the fallback file is the *next* layer, not a parallel option to mix and match.
- **Plan mode is for design choices, not execution.** Once you have a plan approved, execute it; don't re-enter plan mode unless the scope materially changed.
