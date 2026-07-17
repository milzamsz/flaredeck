# ADR-001: Preserve One Profile, One Tunnel, One Token Identity

- Status: Accepted
- Decision owners: FlareDeck maintainers
- Applies from: current architecture and all enhancement phases

## Context

FlareDeck profiles currently provide a clear operational and security boundary: one local profile selects one Cloudflare named tunnel and one API-token storage account. The AI development enhancement introduces workspaces and sessions, creating pressure to combine multiple tunnels, accounts, or token identities inside one profile.

## Decision

Preserve the invariant:

```text
1 Profile = 1 Cloudflare Tunnel = 1 API-token identity
```

A workspace references a profile. A development session references both the workspace and its selected profile. Neither workspace nor session duplicates, replaces, or expands the profile’s tunnel/token cardinality.

## Consequences

### Positive

- clear secret ownership;
- simpler Cloudflare account and zone reasoning;
- predictable process supervision;
- easier error hints and token-scope diagnosis;
- reduced migration risk for existing users.

### Negative

- users needing multiple tunnels create multiple profiles;
- workspace routing must layer above profiles;
- multi-account workflows require explicit profile selection.

## Rejected alternatives

- one profile with many tunnels;
- global API token shared implicitly by every profile;
- workspace-owned tunnel credentials;
- task-created tunnel by default.

## Change trigger

Reconsider only if a validated product requirement proves the current cardinality causes material user harm and a complete secret, migration, process, and UI model is approved.
