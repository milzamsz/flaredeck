# Temporary Route Ownership and Conflict Contract

## Creation input

Temporary routes originate only from a currently trusted `.flaredeck/project.yaml` entry with `mode: temporary`. The workspace chooses an existing profile and loopback origin. Interfaces cannot override hostname, path, origin, profile, expiry, or command.

Default expiry is one hour; maximum expiry is 24 hours. One profile tunnel is reused. No new tunnel or credential is created.

## Pre-mutation checks

1. Re-authorize the current manifest fingerprint.
2. Resolve the selected profile, zone, tunnel, and configuration.
3. Verify the origin is loopback HTTP and the route is temporary.
4. Reject an ingress rule with the same hostname/path, including an identical persistent rule.
5. Query Cloudflare for the exact hostname and reject any existing DNS record.
6. Verify token scope errors return the named DNS read/edit hint.
7. Verify the ownership store is readable and has no active record for the same key.

No mutation occurs before every check succeeds.

## Commit and compensation

1. Persist a `creating` ownership record atomically.
2. Create the exact DNS CNAME and record its returned ID.
3. Insert the temporary ingress rule immediately before the final catch-all and save YAML atomically.
4. Start the owned loopback capture proxy.
5. Mark ownership `active` with DNS ID, exact ingress rule, proxy PID identity, timestamps, and expiry.

Failure compensates in reverse order. Cleanup errors are recorded alongside the original failure and result in `cleanup_incomplete`.

## Cleanup and reconciliation

- cleanup is idempotent;
- stop only the matching owned proxy process;
- remove ingress only when hostname, path, and service exactly match ownership;
- delete DNS only by the recorded zone/record ID after confirming the record still represents the owned hostname/target;
- never delete a persistent route or the final catch-all;
- missing resources count as converged cleanup;
- mismatched resources remain untouched and become `cleanup_incomplete`;
- startup, doctor, session stop, and an explicit desktop retry reconcile expired or incomplete records.

## Ownership schema

The schema-versioned application-data record includes route ID, workspace/session/profile IDs, zone ID, tunnel ID, hostname, path, original loopback origin, proxy loopback service, DNS record ID, ingress fingerprint, proxy PID/start-time/executable, created/expiry timestamps, and cleanup state. It contains no token, credential, raw request, or environment value.
