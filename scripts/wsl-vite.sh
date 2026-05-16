#!/usr/bin/env bash
# Launch Vite for FlareDeck inside WSL.
# Invoked by scripts/desktop-dev.{ps1,sh} via `wsl.exe -- bash <this-file>`.
# Keeps the window open on failure so errors are readable.

set -u
cd /home/milzam/flaredeck || { echo "FATAL: /home/milzam/flaredeck missing"; read -r -p "press enter "; exit 1; }
export PATH="/home/milzam/.nvm/versions/node/v22.22.2/bin:/usr/local/bin:/usr/bin:/bin"

npm run dev
status=$?
if [ "$status" -ne 0 ]; then
  echo
  echo "VITE EXITED $status"
  read -r -p "press enter to close "
fi
