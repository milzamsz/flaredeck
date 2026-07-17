# Phase 2 Prompt: Domain and Application-Service Extraction

## Objective

Extract shared Rust application services and ports while preserving existing desktop behavior.

## Required approach

Use tested vertical slices. Start with one coherent flow, preferably tunnel status/start/stop, then continue only through separate tasks.

For each slice:

1. Trace current handler, types, state, process/API adapter, TypeScript wrapper, store action, and UI caller.
2. Define the smallest domain types and port needed.
3. Add fake-port application-service tests.
4. Wrap existing implementation as a concrete adapter.
5. Convert the Tauri handler into a thin translation layer.
6. Preserve public command and response compatibility.
7. Run cross-boundary verification.
8. Update architecture and technical docs if the approved boundary changes.

## Required preservation

- token storage path;
- Cloudflare preflight ordering;
- `hint_for` behavior;
- route API/CLI fallback;
- YAML catch-all;
- WSL rewriting;
- concurrent profile process state;
- log streaming;
- crashloop threshold;
- platform-specific termination;
- updater initialization and release behavior.

## Prohibited work

- no workspace runtime yet unless a dedicated later Phase 4 task;
- no MCP dependency;
- no broad Cargo workspace split;
- no rename-only migration mixed with behavior changes;
- no protocol types inside domain services.

## Required evidence

- before/after flow map;
- behavior-parity tests;
- thin-handler proof;
- commands and results;
- migration and rollback note.

## Exit criteria

Core behavior can be invoked and tested through application services without launching the desktop UI.
