# ADR-003: Use Shared Rust Application Services for Desktop, CLI, and MCP

- Status: Accepted

## Context

Adding CLI and MCP directly beside existing Tauri commands could create three implementations of validation, process ownership, cleanup, error handling, and redaction.

## Decision

Extract domain behavior and orchestration into Rust application services. Tauri handlers, CLI commands, and MCP tools are thin adapters that construct operation context, call services, and serialize results.

## Consequences

- behavior is testable without the desktop shell;
- interface parity becomes enforceable;
- initial extraction work is required;
- adapters must not leak protocol-specific types inward.

## Rejected alternatives

- call Tauri commands from CLI;
- have MCP shell out to the CLI as the final architecture;
- duplicate current commands in each binary.

## Transitional allowance

MCP or CLI may temporarily call a stable CLI during an exploratory prototype only. Such a prototype may not be accepted as the production architecture.
