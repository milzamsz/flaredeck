# Phase 0 Baseline Report

Date: 2026-07-17

## Current implementation

FlareDeck is a Tauri desktop application. React calls typed wrappers in
`src/lib/tauriApi.ts`; Rust Tauri handlers in `src-tauri/src/commands/` own
profile, tunnel, configuration, DNS, WSL, preferences, and Cloudflare work.
There is no application-service layer, CLI binary, workspace state, trust
store, runtime supervisor, session model, or MCP server yet.

Critical flows confirmed:

- profile creation verifies the token and tunnel scope before creating the
  tunnel, writes credentials and initial YAML, then stores the token through
  `secrets.rs`;
- routes use the Cloudflare API when a profile token and zone are available,
  otherwise use the existing `cloudflared tunnel route dns` fallback;
- initial YAML includes the final `http_status:404` ingress rule; frontend
  YAML helpers preserve that rule and perform WSL origin rewriting;
- tunnel processes are tracked by profile, stream output to Tauri events,
  apply a three-failure/30-second crashloop guard, and use platform-specific
  stop commands;
- release packaging is handled by `.github/workflows/release.yml`; no PR CI
  workflow exists.

## Command surface

`lib.rs` registers 27 Tauri commands, including profile/token management,
Cloudflare API operations, tunnel lifecycle, config, DNS/network checks, WSL,
preferences, and external opener actions. The TypeScript wrapper maps the
same desktop surface. No headless interface exists.

## Regression checklist

- one profile remains one tunnel and one token identity;
- tokens use only keychain/encrypted fallback and never cross an interface;
- profile preflight occurs before Cloudflare or local mutation;
- DNS API path keeps scope hints and CLI fallback;
- ingress ends with `http_status:404` and WSL origin rewriting remains;
- concurrent tunnel state, logs, crashloop handling, and platform-aware stop
  behavior remain intact;
- updater configuration and release workflow remain unchanged;
- normal CI must not use Cloudflare credentials.

## Verification evidence

| Command | Result |
| --- | --- |
| `npm run lint` | passed |
| `npm run build` | passed |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | failed: existing formatting differences in `cf_api.rs`, `commands/cf.rs`, `commands/dns.rs`, `commands/profiles.rs`, and `secrets.rs` |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | not run because format check failed in the chained baseline command |
| `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` | passed: 3 tests |
| `bash scripts/validate-package.sh` | passed |

## Gaps and classification

- Required refactor: handlers contain orchestration and cannot yet be reused
  by CLI or MCP. Phase 2 extracts vertical slices without behavior changes.
- Required verification work: no aggregate verifier, PR CI, test fixtures, or
  meaningful Rust coverage beyond three unit tests. Phase 1 owns this.
- Future scope: workspace, trust, runtime, session, CLI, and MCP contracts
  exist only as approved documentation. Their implementation begins only in
  their respective phases.
- Existing issue: Rust formatting is not clean at baseline. Phase 1 must make
  it deterministic without formatting or changing unrelated user work.

Existing uncommitted changes add cloudflared installation and preference UI
work. They are outside this enhancement task and must remain untouched.

## Phase 1 backlog

1. `task_002_phase_1_verification` — aggregate verification and documentation
   validation; first because every later task depends on a reliable signal.
2. `task_003_phase_1_ci` — PR CI using the aggregate verifier while preserving
   the release workflow.
3. `task_004_phase_1_fixtures` — temporary-directory and fake-process testing
   conventions after the baseline command is stable.

No ADR correction is required: the current desktop-only behavior is the
expected pre-enhancement state, not a contradiction of the approved target.
