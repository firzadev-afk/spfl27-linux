#!/usr/bin/env bash
# Launch the native Playtools app (iced). Builds when missing or stale.
set -euo pipefail
cd "$(dirname "$0")"

BIN="target/release/spfl27"

needs_build=false
if [[ ! -x "$BIN" ]]; then
  needs_build=true
elif [[ -n "$(find src assets Cargo.toml Cargo.lock -newer "$BIN" -print -quit 2>/dev/null)" ]]; then
  echo "Sources changed since last build — rebuilding..."
  needs_build=true
fi

if [[ "$needs_build" == true ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "ERROR: binary missing/stale and 'cargo' not found." >&2
    echo "Build once manually: cd native && cargo build --release" >&2
    exit 1
  fi
  echo "Building (takes a few minutes on first run)..."
  cargo build --release
fi

exec "$BIN" "$@"
