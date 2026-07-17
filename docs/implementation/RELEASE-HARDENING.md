# Release Hardening

## Packaging decision

FlareDeck desktop, `flaredeck-cli`, `flaredeck-mcp`, and `flaredeck-webhook-proxy` ship in one Tauri installer/updater artifact. The companions are Tauri external binaries beside the desktop executable (inside the application bundle on macOS). Their versions must exactly equal the desktop version. Standalone companion releases are intentionally unsupported because a desktop update could otherwise leave incompatible executables behind.

The updater identifier `dev.flaredeck.desktop`, public key, endpoint, product name, and stable installer filenames are unchanged. Tauri signs the complete updater bundle, including its sidecars; the workflow does not rotate or export the private key. Each successful release also publishes `SHA256SUMS.txt` for transport/inventory verification. The updater signature remains the authenticity control.

The release begins as a draft. Linux x86_64, Windows x86_64, and macOS universal jobs must all build the three target-specific companions and pass CLI/MCP/proxy smoke checks before the draft can be published or the hosted updater manifest updated. Partial platform releases fail closed.

## Installation and discovery

- Windows installer: companions are in the FlareDeck installation directory. Configure AI clients with the absolute `flaredeck-mcp.exe` path when that directory is not on `PATH`.
- macOS DMG: companions are under `/Applications/FlareDeck.app/Contents/MacOS/`; configure the full path.
- Linux `.deb`: companions install under `/usr/bin`. The AppImage is suitable for the desktop, but its mounted internal path is not stable for long-lived MCP client configuration; prefer `.deb` for AI integration.

The desktop updater replaces the containing bundle atomically. Copying a companion elsewhere opts out of compatibility guarantees; delete the copy and use the newly installed companion after an upgrade.

## Compatibility and migration

`docs/specs/release-compatibility.json` is the machine-checked contract. Package, Cargo, Tauri, and companion versions are exact. State schemas remain version 1 and changes are additive:

- existing `~/.cloudflared/` profiles, keychain entries, encrypted fallback tokens, YAML, credentials, and catch-all ingress are not migrated or rewritten by installation;
- missing application-data workspace, trust, session, temporary-route, event, audit, and registry files load as empty state;
- session fields added after the initial schema use safe defaults, covered by migration tests;
- corrupted ownership/trust state fails closed rather than mutating Cloudflare or starting a process.

No new dependency, license, token-write path, Cloudflare scope, updater key, or updater endpoint is introduced by packaging.

## Rollback and downgrade

Before downgrading, stop every session and use desktop cleanup reconciliation. Downgrading to a pre-workspace release preserves existing profiles and secrets because those versions ignore the separate FlareDeck application-data state. They cannot manage active runtimes, temporary routes, or captures created by a newer version. Do not downgrade while cleanup is incomplete; reinstall the current version and reconcile first.

Downgrades within schema version 1 retain additive JSON fields, but exact companion compatibility still requires using all binaries from the same installer. Automatic updater rollback is not enabled. Manual rollback requires a previously signed installer compatible with the unchanged updater identity; restoring application-data backups across a schema-major boundary is unsupported.

## Verification and evidence

Local Linux evidence: aggregate lint/build/Rust/package verification, release-contract validation, release-mode companion build, CLI version/doctor, MCP initialize/tool discovery, proxy version, and generated bundle inspection. Cross-platform workflow jobs provide the same companion smoke checks and installer creation on Windows, macOS universal, and Linux. Native GUI launch, OS code-signing/notarization UI, WSL invocation, and live updater installation remain manual release-candidate checks and must not be claimed from this Linux development host.

Official packaging assumptions follow Tauri's external-binary target-triple contract and updater signature requirement: https://v2.tauri.app/develop/sidecar/ and https://v2.tauri.app/plugin/updater/.

### Artifact matrix

| Target | Desktop artifact | Included companions | Evidence |
| --- | --- | --- | --- |
| Windows x86_64 | stable `x64-setup.exe` | CLI, MCP, proxy `.exe` | release workflow configured; runner execution pending next tag |
| macOS universal | stable universal DMG | universal CLI, MCP, proxy | dual-arch build plus `lipo` configured; runner execution pending next tag |
| Linux x86_64 | stable AppImage and `.deb` | CLI, MCP, proxy | `.deb` built and inspected locally; all companions executed at `0.2.3` |

### Release-candidate decision

**Ready with known limitations.** Source, Linux release-mode companion, and Linux `.deb` evidence pass. Publication is fail-closed behind the Windows, macOS, and Linux workflow matrix, updater manifest generation, and checksum creation. The next tag is not “Ready” for public release until those remote jobs and the manual native checks below pass.

| Check | Linux local | Windows runner | macOS runner | Manual RC |
| --- | --- | --- | --- | --- |
| release contract/version | passed | configured | configured | n/a |
| CLI version/doctor | passed from extracted `.deb` | configured version smoke | configured version smoke | inspect diagnostics |
| MCP initialize/11 tools | passed | configured | configured | client trust UX |
| proxy version | passed | configured | configured | temporary-route live flow |
| desktop package creation | `.deb` passed | configured | configured | native launch required |
| updater signature/install | private key unavailable locally | build gate | build gate | live upgrade required |
| WSL invocation | unavailable | n/a | n/a | required on Windows RC host |

Linux `.deb` SHA-256 for this non-published local build: `884157a24ec0fd5001350ce27da503e8e0fd3aabcb4dea92bf72bf12a5ac405c`. Reproducible byte equality is not claimed because package timestamps and signing vary.
