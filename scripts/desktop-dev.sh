#!/usr/bin/env bash
# Launch FlareDeck in dev mode (Git Bash on Windows).
# See scripts/desktop-dev.ps1 for architecture notes.

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="$USERPROFILE\\.cargo\\target-flaredeck"
OVERRIDE_PATH="${TMP:-/tmp}/flaredeck-tauri-override.json"

printf '{"build":{"beforeDevCommand":""}}' > "$OVERRIDE_PATH"

echo "==> Project root: $PROJECT_ROOT"
echo "==> Starting Vite in a separate WSL window..."
MSYS_NO_PATHCONV=1 cmd.exe /c start "FlareDeck Vite" wsl.exe -d Ubuntu -- \
  bash /home/milzam/flaredeck/scripts/wsl-vite.sh

echo "==> Waiting for Vite on http://localhost:5173 ..."
ready=0
for _ in $(seq 1 60); do
  if curl -sf -o /dev/null http://localhost:5173/; then
    ready=1
    break
  fi
  sleep 1
done
if [ "$ready" -ne 1 ]; then
  echo "Vite did not become ready on :5173 within 60s. Check the WSL window for errors." >&2
  exit 1
fi
echo "==> Vite ready."

export CARGO_TARGET_DIR="$TARGET_DIR"
export CARGO_INCREMENTAL=0
echo "==> Launching tauri dev (CARGO_TARGET_DIR=$CARGO_TARGET_DIR)"
cd "$PROJECT_ROOT"
exec tauri dev --no-dev-server-wait -c "$OVERRIDE_PATH"
