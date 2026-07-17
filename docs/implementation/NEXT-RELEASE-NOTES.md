# Next FlareDeck Release Notes

## Highlights

- shared Rust application services across desktop, CLI, and local stdio MCP;
- trusted `.flaredeck/project.yaml` workspaces with fingerprint invalidation;
- owned runtime/tunnel sessions, readiness, bounded redacted logs, and idempotent cleanup;
- desktop workspace/session/trust experience;
- expiring owned temporary routes with bounded pre-storage-redacted webhook capture;
- desktop-only confirmed webhook replay to the original loopback target;
- version-matched CLI, MCP, and webhook proxy companions inside every installer/updater.

## Security and compatibility

No token scope or token storage path changes. MCP still has no network listener, trust approval, arbitrary command, arbitrary route, token, or webhook replay tool. Existing profiles and secrets remain in their original locations and formats. The updater identifier, public key, endpoint, and stable installer filenames are unchanged.

Before a manual downgrade, stop sessions and reconcile temporary cleanup. The AppImage desktop does not provide a stable internal MCP executable path; Linux AI-integration users should prefer the `.deb` package. Windows/macOS signing and native launch, WSL invocation, and live updater installation require the release-candidate checklist on their respective systems.
