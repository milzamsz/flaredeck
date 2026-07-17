# Workspace Manifest Specification

## Location

```text
<repository-root>/.flaredeck/project.yaml
```

## Design goals

- version-controlled;
- human-readable;
- deterministic;
- contains no secrets;
- directly validatable;
- stable input to the trust fingerprint;
- framework-neutral.

## Example

```yaml
version: 1

project:
  name: fluxbill
  id: fluxbill-local
  labels:
    product: fluxbill

profile:
  name: ocloud-development

runtime:
  executable: bun
  args: [run, dev]
  workingDirectory: .
  startupTimeoutSeconds: 60
  stopTimeoutSeconds: 10
  crashloop:
    maxFailures: 3
    windowSeconds: 30

ready:
  type: http
  url: http://127.0.0.1:5173/health
  expectedStatus: [200, 299]
  intervalMilliseconds: 500
  timeoutSeconds: 60

exposure:
  routes:
    - hostname: fluxbill-dev.ocloud.pro
      service: http://127.0.0.1:5173

lifecycle:
  startRuntime: true
  ensureTunnel: true
  stopRuntimeOnSessionStop: true
  stopTunnelIfStartedBySession: true
  removeTemporaryRoutes: true

environment:
  passthrough:
    - NODE_ENV
    - RUST_LOG
  values:
    APP_ENV: development
```

## Field rules

### `version`

Required integer. Unsupported versions fail validation.

### `project`

- `name`: required display name;
- `id`: optional stable repository identifier using a restricted character set;
- `labels`: optional display metadata, excluded from trust unless behavior depends on it.

### `profile`

References an existing FlareDeck profile by stable ID when available. Name lookup may be supported for convenience but must resolve unambiguously and persist the selected ID locally.

### `runtime`

- `executable`: required, no shell expression;
- `args`: optional string array;
- `workingDirectory`: relative path beneath workspace root;
- timeout and crashloop limits use bounded positive values.

A repository script is allowed:

```yaml
runtime:
  executable: bash
  args: [scripts/dev.sh]
```

The script itself is repository code and remains visible during trust review. Direct shell mode such as `bash -c` should be rejected by policy unless a future ADR permits it.

### `ready`

MVP types:

- `tcp` with host and port;
- `http` with local URL and expected status range.

Targets are local by default. External readiness targets require explicit future policy.

### `exposure.routes`

Each route contains:

- hostname;
- local service URL;
- optional path pattern;
- optional mode `persistent` or `temporary`, with `persistent` as MVP default until temporary ownership is implemented.

Phase 4 verifies persistent routes against the selected profile's ingress configuration before starting a session. Temporary route creation and deletion remain unavailable until the session can record and safely clean up Cloudflare route ownership.

Hostnames must belong to the selected profile’s zone or an approved subdomain policy.

### `lifecycle`

Controls intent but cannot grant capabilities wider than local policy.

### `environment`

- `passthrough`: environment names whose values may be inherited without being read or returned;
- `values`: committed non-secret literals.

Sensitive-looking names may be prohibited from `values`.

## Prohibited content

- Cloudflare API tokens;
- tunnel credential JSON;
- private keys;
- passwords;
- bearer tokens;
- secret `.env` values;
- arbitrary post-start shell hooks;
- unrestricted filesystem mounts;
- remote command execution configuration.

## Validation output

Validation should return field-oriented errors:

```json
{
  "valid": false,
  "errors": [
    {
      "path": "runtime.workingDirectory",
      "code": "PATH_OUTSIDE_WORKSPACE",
      "message": "The working directory resolves outside the workspace root."
    }
  ]
}
```
