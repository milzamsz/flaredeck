# Review and Verification Prompt

Act as an independent reviewer. Do not modify code unless the review task explicitly authorizes fixes.

## Review sources

Read:

- active task and acceptance criteria;
- `AGENTS.md`;
- relevant product, ADR, domain, architecture, technical, security, and specification documents;
- the complete diff;
- affected existing code and tests.

## Review dimensions

1. Scope correctness
2. Product and domain consistency
3. Architecture and dependency direction
4. Security and secret handling
5. Trust and authorization
6. Process ownership and cleanup
7. Cross-platform behavior
8. Error and output contracts
9. CLI/MCP/Tauri parity
10. Test adequacy
11. Migration and backward compatibility
12. Documentation consistency
13. Unrelated churn and dependency justification

## Required verification

Run the relevant commands. Add focused exploratory checks for the risk area. Do not treat a successful compilation as proof of lifecycle correctness.

## Finding severity

- Blocker: security violation, data/credential risk, destructive behavior, invalid architecture decision, or acceptance criteria not met.
- High: likely regression, cleanup leak, cross-platform failure, or contract incompatibility.
- Medium: incomplete tests, ambiguous errors, maintainability issue, or documentation mismatch.
- Low: minor clarity, naming, or non-blocking improvement.

## Output

- Review decision: Approve, Approve with follow-up, or Block
- Findings ordered by severity, each with evidence and required correction
- Acceptance-criteria matrix
- Commands and results
- Security review summary
- Compatibility summary
- Residual risks

Do not approve based on intent. Approve based on code, tests, and evidence.
