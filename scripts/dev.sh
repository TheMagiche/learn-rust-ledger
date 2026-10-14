#!/usr/bin/env bash
# Start the API and the Vite teaching UI. Ctrl-C stops both.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [[ ! -d web/node_modules ]]; then
  npm --prefix web install
fi

cargo run -p spend -- serve --dev --addr 127.0.0.1:8080 &
api_pid=$!

cleanup() {
  kill "$api_pid" 2>/dev/null || true
  wait "$api_pid" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

npm --prefix web run dev
