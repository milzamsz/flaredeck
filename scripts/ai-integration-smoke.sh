#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="$ROOT/src-tauri/target/debug"

cargo build --manifest-path "$ROOT/src-tauri/Cargo.toml" --bin flaredeck-cli --bin flaredeck-mcp
cargo test --manifest-path "$ROOT/src-tauri/Cargo.toml" --test cli_contract --test mcp_protocol

doctor="$($TARGET/flaredeck-cli --output=json doctor)"
node -e 'const value=JSON.parse(process.argv[1]); if (!value.ok || value.meta.schemaVersion !== "1") process.exit(1)' "$doctor"

protocol="$(printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"smoke","version":"1"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | "$TARGET/flaredeck-mcp")"
node -e 'const lines=process.argv[1].trim().split("\n").map(JSON.parse); if (lines.length !== 2 || lines[1].result.tools.length !== 11) process.exit(1)' "$protocol"

node -e 'JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"))' "$ROOT/examples/.vscode/mcp.json"

if command -v opencode >/dev/null 2>&1; then
  config="$(mktemp -d)"
  trap 'rm -rf "$config"' EXIT
  PATH="$TARGET:$PATH" XDG_CONFIG_HOME="$config" OPENCODE_CONFIG="$ROOT/examples/opencode.jsonc" \
    opencode mcp list --pure | rg 'flaredeck.*connected' >/dev/null
fi

echo "AI integration smoke checks passed."
