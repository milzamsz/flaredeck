# Development Session Lifecycle Specification

## 1. Session purpose

A development session is a bounded, auditable attempt to run one trusted workspace and make its approved local routes available through the selected FlareDeck profile.

## 2. Ownership model

The session records whether it created or merely observed each resource:

| Resource | Possible ownership |
|---|---|
| Runtime process | owned by session or externally running if future policy permits |
| Tunnel process | started by session or pre-existing |
| Persistent route | pre-existing and verified |
| Temporary route | created and owned by session |
| DNS record | pre-existing, updated by approved operation, or created by session |

Cleanup may affect only resources owned by the session unless a user explicitly approves wider action.

## 3. Start stages

1. Acquire workspace-level session lock.
2. Resolve workspace and manifest.
3. Validate schema and security policy.
4. Verify current trust fingerprint.
5. Create session and audit request.
6. Start runtime if configured.
7. Wait for readiness.
8. Inspect selected profile tunnel.
9. Start tunnel if required and not already running.
10. Verify persistent routes or create approved temporary routes.
11. Run health aggregation.
12. Persist state and return public URLs.

## 4. Failure compensation

| Failure | Required compensation |
|---|---|
| Runtime spawn | mark failed, no process cleanup if spawn never succeeded |
| Readiness timeout | stop owned runtime |
| Tunnel start | stop owned runtime; preserve pre-existing routes |
| Route creation | remove only routes created by this attempt; stop owned runtime and owned tunnel according to policy |
| State persistence after resource creation | attempt safe cleanup and report both original and cleanup errors |

## 5. Idempotency

### Start

- if a healthy active session exists for the workspace, return it with `reused: true`;
- if a start is already in progress, return conflict or join according to the final concurrency design;
- do not spawn a second runtime accidentally.

### Stop

- stopped session returns success with `alreadyStopped: true`;
- missing owned process is treated as converged state with a warning;
- temporary route already absent is converged state;
- cleanup metadata is updated consistently.

## 6. Tunnel stop policy

A session stops a tunnel only when all are true:

- it started the tunnel;
- manifest lifecycle allows stopping it;
- no other active session depends on it;
- no manual ownership marker protects it;
- stop is part of normal cleanup or explicit user action.

## 7. Crash recovery

On FlareDeck startup or doctor command:

1. read recoverable active-session metadata;
2. inspect recorded process IDs carefully, guarding against PID reuse;
3. inspect tunnel state through authoritative process state or command;
4. classify session as running, orphaned, stopped, or recovery-required;
5. do not kill uncertain processes automatically;
6. offer safe reconciliation through desktop UX or explicit CLI action.

## 8. Concurrency

MVP default: one active session per workspace.

Different workspaces may run concurrently if:

- runtime ports do not conflict;
- selected routes do not conflict;
- process and log state are keyed correctly;
- shared profile tunnel ownership is reference-aware.

## 9. Health states

- `unknown`: no observation;
- `starting`: startup in progress;
- `healthy`: required checks pass;
- `degraded`: session works partially or non-critical checks fail;
- `failed`: required service unavailable;
- `stopped`: no active session resources;
- `cleanup_incomplete`: stop completed partially.

## 10. Session result requirements

Return:

- session and workspace IDs;
- state;
- stage statuses;
- public URLs;
- runtime/tunnel ownership;
- health summary;
- warnings;
- correlation ID;
- cleanup status when stopped.

Never return process environment, tokens, credentials, or unbounded logs.
