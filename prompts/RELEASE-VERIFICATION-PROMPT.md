# Release Verification Prompt

Verify a FlareDeck release candidate containing desktop, CLI, or MCP changes.

## Required checks

- version consistency;
- changelog and release notes;
- updater metadata and signing remain valid;
- desktop artifact launch;
- CLI `version` and `doctor`;
- MCP initialization and tool discovery;
- existing profile migration and tunnel operation;
- workspace/trust/session state migration;
- no secrets in artifacts or logs;
- platform artifact names and stable download links;
- checksums/signatures;
- rollback/downgrade documentation;
- license and dependency changes.

## Output

- Release decision: Ready, Ready with known limitations, or Blocked
- Artifact matrix
- Test matrix by platform
- Migration results
- Security results
- Known issues
- Required release-note warnings
