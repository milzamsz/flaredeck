# Repository Migration Guide

## Goal

Move from command-centric Tauri handlers to shared application services without a disruptive rewrite.

## Step 1: Establish tests around existing behavior

Prioritize:

- tunnel status/start/stop;
- crashloop threshold;
- route API and CLI fallback;
- profile creation preflight;
- token redaction;
- WSL host rewriting;
- YAML catch-all preservation.

## Step 2: Introduce operation context and ports

Add types with no behavior change. Keep existing modules as concrete adapters.

## Step 3: Extract one vertical slice

Recommended first slice: tunnel status/start/stop.

Status extraction is complete: `application::tunnel_service::status` owns
status calculation and stale-child cleanup while the Tauri command remains a
thin adapter.

The lifecycle slice now also lives in `application::tunnel_service`: Tauri
supplies only its existing log-event callback. The service keeps the current
fixed cloudflared invocation, early-exit and crashloop policy, and
platform-specific process-tree termination unchanged.

Profile-index reads now use `application::profile_service::list`; this returns
the existing safe profile display model and does not read or serialize token
values.

- define `TunnelSupervisor` port;
- wrap current process logic;
- add `ProfileApplicationService` or `TunnelApplicationService`;
- make current Tauri handlers delegate;
- verify desktop behavior.

## Step 4: Extract route and profile operations

Preserve Cloudflare error-hint behavior and preflight ordering.

## Step 5: Add headless binary initialization

Construct the same adapters without desktop-only plugin initialization. Confirm no window is created.

## Step 6: Add workspace/session modules

New behavior is built on the stable service structure rather than inside existing commands.

## Migration rules

- one behavioral change per vertical slice;
- no mass rename plus feature implementation;
- old public Tauri command names remain until frontend migration is complete;
- persistence changes require versioning and migration tests;
- data is copied or upgraded atomically;
- rollback must leave existing profiles usable.

## Cargo workspace decision

Do not split crates merely because there are now multiple modules. Reassess after CLI and MCP dependencies and release packaging are known.
