# Test Strategy

## Test pyramid

### Domain unit tests

Fast tests for:

- manifest normalization;
- trust fingerprint vectors;
- workspace invariants;
- session state transitions;
- ownership rules;
- redaction;
- error mapping.

### Application service tests

Use fake ports for:

- trusted and untrusted starts;
- runtime failure;
- readiness timeout;
- tunnel pre-existing versus session-started;
- route failure and compensation;
- repeated stop;
- audit failure policy.

### Adapter tests

- atomic file repositories in temporary directories;
- process supervisor fixture scripts;
- HTTP/TCP probe fixture servers;
- mocked Cloudflare API;
- CLI JSON snapshots;
- MCP stdio protocol capture.

### End-to-end tests

Use a tiny fixture application that:

- starts on a selectable local port;
- exposes `/health`;
- writes predictable stdout/stderr;
- handles termination signals;
- can simulate startup delay and crash.

Real Cloudflare tests are opt-in and use dedicated non-production zones and credentials.

## Cross-platform matrix

- Linux: primary CI for domain, CLI, MCP, and fixture process tests;
- Windows: process tree, path, Credential Manager, and WSL hybrid smoke tests;
- macOS: process, keychain, packaging, and updater smoke tests;
- WSL/headless Linux: encrypted fallback and host rewriting.

## Security regression suite

Use synthetic canary secrets and malicious manifests:

- path traversal;
- shell metacharacters;
- `bash -c` policy rejection;
- environment leakage;
- oversized logs;
- external readiness redirect;
- changed fingerprint;
- corrupt trust file;
- MCP unknown properties;
- pre-existing tunnel cleanup protection.

## CI policy

Standard PR CI must not require:

- a Cloudflare account;
- real tokens;
- public DNS changes;
- desktop interaction;
- network access beyond dependency installation and controlled test fixtures.

## Fixture conventions

- use `std::env::temp_dir()` plus a unique `uuid` directory and remove it in
  the test cleanup path;
- keep fixture scripts and binaries local to the test's temporary directory;
- use loopback-only TCP/HTTP fixtures with a selected free port;
- fake Cloudflare, process, trust, and session ports in service tests rather
  than calling a real account or spawning `cloudflared`;
- introduce a shared fixture helper only after two tests need the same setup.
