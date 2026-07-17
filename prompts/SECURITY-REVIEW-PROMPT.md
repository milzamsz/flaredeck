# Security Review Prompt

Review the proposed FlareDeck change as a security-focused engineer.

## Mandatory threat areas

- secret storage and output;
- workspace trust and fingerprint invalidation;
- arbitrary command or shell escape;
- path traversal, symlinks, Windows/UNC/WSL paths;
- environment leakage;
- process ownership, PID reuse, and process-tree termination;
- DNS and route mutation ownership;
- MCP schema abuse and stdout protocol integrity;
- unbounded logs, payloads, timeouts, and denial of service;
- public development exposure;
- local state tampering and fail-open behavior;
- dependency and release implications.

## Required method

1. Map data and control flow.
2. Identify trust boundaries and attacker-controlled inputs.
3. Compare controls against `docs/security/THREAT-MODEL.md`.
4. Inspect tests for each applicable threat.
5. Run canary-secret and malicious-input tests where possible.
6. Classify findings by exploitability and impact.

## Output

- Threats introduced or changed
- Existing controls reused
- Missing controls
- Test gaps
- Findings with severity and remediation
- Decision: Accept, Accept with conditions, or Block
