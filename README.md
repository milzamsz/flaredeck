# FlareDeck

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24c8db.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61dafb.svg)](https://react.dev/)
[![Cloudflare Tunnel](https://img.shields.io/badge/Cloudflare-Tunnel-f38020.svg)](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/)

FlareDeck is a desktop control panel for local Cloudflare Tunnel development.
It gives you a focused GUI for creating tunnel profiles, editing ingress rules,
routing DNS, starting and stopping `cloudflared`, and watching live tunnel logs.

FlareDeck is built with Tauri v2, Rust, React 19, Vite, Tailwind CSS v4,
shadcn/ui, and Zustand. It is intentionally local-first: it talks to the
`cloudflared` CLI and files under `~/.cloudflared/`, not the Cloudflare API.

## Screenshots

![FlareDeck dashboard showing a running tunnel, ingress rules, DNS status, and live logs](docs/screenshots/dashboard.png)

![FlareDeck new profile dialog with tunnel creation and WSL service options](docs/screenshots/new-profile.png)

## Why FlareDeck?

Cloudflare Tunnel is excellent for publishing local services, but day-to-day
development usually means jumping between YAML, shell commands, DNS routing, and
log output. FlareDeck puts that workflow in one desktop app while keeping
Cloudflare's own `cloudflared` CLI as the source of truth.

Use it when you want to:

- Run one or more named tunnels without memorizing every command.
- Keep separate profiles for different local projects.
- Edit ingress routes without hand-writing every YAML rule.
- Route hostnames with `cloudflared tunnel route dns`.
- Check whether local origins and DNS records are reachable.
- Bridge Windows desktop development to services running inside WSL Ubuntu.
- Keep live `cloudflared` logs visible while you iterate.

## Features

- Start, stop, and restart `cloudflared tunnel run` per profile.
- Create named tunnels from the new-profile flow and seed YAML with the tunnel
  UUID and credentials path.
- Manage ingress rules in a typed form; FlareDeck appends the catch-all rule.
- Edit the raw `~/.cloudflared/<profile-id>.yml` config in a CodeMirror YAML
  editor.
- Run `cloudflared tunnel login` and detect `cert.pem` authentication state.
- Route DNS through `cloudflared tunnel route dns -f`.
- Check local origin ports with `tokio::net::TcpStream`.
- Check DNS through `hickory-resolver`.
- Run multiple named tunnel profiles concurrently.
- Stream `cloudflared` stdout and stderr into the in-app log viewer.
- Guard against fast tunnel crash loops.
- Support WSL-hosted local services by rewriting loopback origins to the WSL
  host IP per profile.
- Support light, dark, and system themes.
- English UI locale.
- Keep running in the system tray when configured.
- Enforce a single desktop instance and focus the existing window on relaunch.

## How It Works

FlareDeck is not a Cloudflare API client. It shells out to the local
`cloudflared` binary and reads or writes the same local files you would use by
hand.

```text
~/.cloudflared/
|-- cert.pem                  # written by cloudflared tunnel login
|-- flaredeck.json            # profile index: profiles[] + activeProfileId
|-- <profile-id>.yml          # one cloudflared config per FlareDeck profile
|-- <profile-id>.yml.bak.*    # automatic backups, last 10 kept on save
`-- <tunnel-uuid>.json        # cloudflared tunnel credentials
```

That design keeps FlareDeck easy to inspect and easy to leave: your tunnel
configuration remains normal `cloudflared` configuration.

## Quick Start

### Prerequisites

- Node.js 22 LTS, recommended through [nvm](https://github.com/nvm-sh/nvm).
- Rust and Cargo from [rustup](https://rustup.rs/).
- `cloudflared` on `PATH`.

Install `cloudflared` with one of the official methods:

- macOS: `brew install cloudflare/cloudflare/cloudflared`
- Linux: download from the
  [Cloudflare downloads page](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/)
  and place the binary on `PATH`.
- Windows: install from the same downloads page. FlareDeck also checks common
  install locations under `%ProgramFiles%` and `%LOCALAPPDATA%`.

### Web Preview

The web preview is useful for UI work. Tauri commands are unavailable outside
the desktop shell, so this mode exercises only the frontend shell.

```bash
npm install
npm run dev -- --host 0.0.0.0 --port 5173
```

Open <http://localhost:5173>.

### Desktop App

```bash
npm install
npm run desktop
```

This runs `tauri dev`, starts the Vite frontend, and launches the desktop shell.

## First-Run Flow

1. Launch FlareDeck.
2. Check the sidebar footer for `cloudflared` and auth status.
3. If `cloudflared` is missing, open Settings and use the install action to
   visit Cloudflare's downloads page.
4. Sign in from Settings. FlareDeck runs `cloudflared tunnel login` and polls
   for `cert.pem`.
5. Create a profile from Dashboard.
6. Add ingress routes for your local services.
7. Start the tunnel and watch the live logs.

## Windows + WSL Development

This repository was developed with the project on a WSL share and the desktop
app running on Windows. In that setup, Windows npm may not resolve Linux
`node_modules/.bin` symlinks. Use the provided helper:

```powershell
pwsh scripts/desktop-dev.ps1
```

or from Git Bash:

```bash
./scripts/desktop-dev.sh
```

The helper starts Vite inside WSL Ubuntu with Node 22 from nvm, then launches
Tauri from Windows against the MSVC toolchain. Install the Tauri CLI once if
needed:

```bash
npm install -g @tauri-apps/cli@^2.11
```

## Build

```bash
npm run build
npm run desktop:build
```

The desktop build writes platform bundles under
`src-tauri/target/release/bundle/`.

## Architecture

```text
src/
|-- main.tsx                  # React entrypoint
|-- router.tsx                # Dashboard, Config, Settings routes
|-- store/app-store.ts        # Zustand state and async app workflows
|-- lib/
|   |-- tauriApi.ts           # typed wrappers around Tauri commands
|   |-- yaml-helpers.ts       # ingress parsing, serialization, WSL rewrite
|   |-- i18n.ts               # English locale setup
|   |-- migrations.ts
|   `-- utils.ts              # shadcn cn()
|-- components/               # app shell, tables, dialogs, logs, shadcn UI
`-- pages/                    # Dashboard, Config, Settings

src-tauri/
|-- Cargo.toml
|-- tauri.conf.json
|-- capabilities/default.json
`-- src/
    |-- lib.rs                # app setup and Tauri command registration
    |-- main.rs
    |-- cloudflared.rs        # cloudflared discovery and version checks
    |-- error.rs              # AppError + serialization
    |-- state.rs              # runtime child-process state
    |-- types.rs              # shared serde payloads
    `-- commands/             # tunnel, auth, config, DNS, profiles, prefs
```

The frontend talks to the backend through typed wrappers in
`src/lib/tauriApi.ts`. When a Tauri command changes, update the Rust command,
shared serde shape, command registration, frontend wrapper, and any Zustand
actions together.

## Verification

Common checks:

```bash
npm run lint
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

For Windows + WSL hybrid work, run checks through the appropriate shell. This
repo's agent instructions require `rtk` prefixes for shell commands, so agentic
workflows should use the command forms documented in `AGENTS.md`.

## Privacy and Security

- FlareDeck reads and writes local `cloudflared` config files.
- It starts local `cloudflared` child processes and streams their logs.
- It does not store Cloudflare API tokens.
- It does not call the Cloudflare API directly.
- Authentication is delegated to `cloudflared tunnel login` and the resulting
  local `cert.pem`.

Treat tunnel credentials and YAML files under `~/.cloudflared/` as sensitive
local machine data.

## Contributing

Contributions are welcome. Before opening a pull request:

- Keep changes focused and easy to review.
- Prefer existing React, shadcn/ui, Zustand, and Tauri patterns.
- Keep TypeScript and Rust command payloads in sync.
- Run the relevant frontend and Rust verification commands.
- Do not commit generated build output such as `dist/`, `dist-windows/`, or
  `src-tauri/target/`.

For more detailed coding-agent guidance, see `AGENTS.md`.

## License

FlareDeck is open source under the [MIT License](LICENSE).
