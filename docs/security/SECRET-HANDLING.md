# Secret Handling Policy

## 1. Existing secret model

FlareDeck API tokens remain stored through:

1. the operating-system keychain; or
2. the existing machine-bound encrypted fallback when keychain access is unavailable.

Tunnel credential JSON files remain sensitive operational credentials managed by the existing profile/tunnel flow.

## 2. Rules

- never store API tokens in `flaredeck.json`, workspace manifests, trust approvals, sessions, audit events, Zustand persistence, CLI config, MCP config, fixtures, screenshots, or documentation;
- never return token values through Tauri, CLI, or MCP;
- never log request authorization headers;
- never add a “show token” feature for agent convenience;
- secret writes use only the sanctioned secret module;
- delete or rotation operations require explicit user intent;
- errors report token missing/invalid/under-scoped without echoing the value.

## 3. Workspace environment

FlareDeck does not become a secret manager in the MVP.

Allowed behavior:

- child framework loads its own `.env` file;
- manifest lists environment names to pass through;
- FlareDeck reports whether a name appears present when this can be done without returning a value;
- committed non-secret literals may be set.

Prohibited behavior:

- parsing and returning `.env` values;
- sending secrets to MCP;
- putting secret literals in the manifest;
- automatically copying secrets between projects;
- accepting an environment map from an AI tool call.

## 4. Redaction

Central redaction should cover:

- known token formats where practical;
- values loaded by the secret subsystem;
- Authorization, Proxy-Authorization, Cookie, and Set-Cookie headers;
- configured sensitive environment names;
- tunnel secret and credential fields;
- URL query parameters commonly used for tokens;
- webhook fields configured as sensitive in Phase 8.

Redaction output should use stable markers such as `[REDACTED]` and record a redaction version in audit metadata.

## 5. Test strategy

Use synthetic canary values and assert absence in:

- success JSON;
- error JSON;
- human output;
- stdout and stderr;
- runtime and tunnel log views;
- audit records;
- persisted workspace/session state;
- MCP tool results and protocol diagnostics.

## 6. Incident response

If a secret is exposed:

1. stop affected sessions;
2. revoke or rotate the Cloudflare token or tunnel credential;
3. remove leaked artifacts from current files and history as appropriate;
4. identify every output boundary affected;
5. add regression tests using a canary secret;
6. document root cause and corrective action;
7. publish a security release when users may be affected.
