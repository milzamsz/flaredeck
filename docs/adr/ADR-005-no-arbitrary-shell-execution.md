# ADR-005: Prohibit Arbitrary Shell Execution

- Status: Accepted

## Context

An AI agent needs to start a project runtime. A generic shell tool is convenient but turns FlareDeck into an unrestricted code-execution interface with unclear trust, escaping, environment, and cleanup behavior.

## Decision

The runtime is declared in `.flaredeck/project.yaml` as executable plus arguments. FlareDeck validates it, includes it in a trust fingerprint, and executes it directly without a shell. CLI and MCP cannot override it with a caller-supplied command.

## Consequences

- predictable process ownership and argument handling;
- no pipelines, redirects, interpolation, or compound shell commands in MVP;
- complex projects should call a committed repository script through an approved executable and argument list;
- command changes require renewed trust.

## Rejected alternatives

- `flaredeck_run_shell_command` MCP tool;
- accepting one opaque command string;
- trusting commands based only on repository path.
