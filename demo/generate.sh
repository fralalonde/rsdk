#!/usr/bin/env bash
# Deterministically generate docs/demo.gif from demo/demo.tape against the
# local mock SDKMAN API (demo/mock_api.py) and a throwaway install home.
# Run via `just demo` (or `sh demo/generate.sh`).
set -euo pipefail
cd "$(dirname "$0")/.."

# Pick a free local port (a fixed port collides with stray dev servers).
PORT="${RSDK_DEMO_PORT:-$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')}"

cargo build --release

# Start the mock API and wait until it answers.
python3 demo/mock_api.py "$PORT" >/dev/null 2>&1 &
MOCK_PID=$!
trap 'kill "$MOCK_PID" 2>/dev/null || true; rm -rf "$FIXTURE"' EXIT
for _ in $(seq 1 50); do
    curl -fsS "http://127.0.0.1:$PORT/candidates/all" >/dev/null 2>&1 && break
    sleep 0.1
done

# Build a throwaway install home: java has two installed versions, 21.0.6-tem
# current and 17.0.9-tem default, so the TUI shows starred tools and the
# full Use / Set default / Remove action modal.
FIXTURE="$(mktemp -d)"
mkdir -p "$FIXTURE/.rsdk/tools/java/21.0.6-tem/bin" \
         "$FIXTURE/.rsdk/tools/java/17.0.9-tem/bin"
printf '#!/bin/sh\necho "openjdk 21.0.6"\n' > "$FIXTURE/.rsdk/tools/java/21.0.6-tem/bin/java"
printf '#!/bin/sh\necho "openjdk 17.0.9"\n' > "$FIXTURE/.rsdk/tools/java/17.0.9-tem/bin/java"
chmod +x "$FIXTURE/.rsdk/tools/java/21.0.6-tem/bin/java" \
         "$FIXTURE/.rsdk/tools/java/17.0.9-tem/bin/java"
ln -sfn 21.0.6-tem "$FIXTURE/.rsdk/tools/java/current"
ln -sfn 17.0.9-tem "$FIXTURE/.rsdk/tools/java/default"

# Keep vhs's bundled-Chromium cache in a stable location (it would otherwise
# land in $FIXTURE and be re-downloaded on every run).
REAL_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}"

PATH="$PWD/target/release:$PATH" \
    HOME="$FIXTURE" \
    XDG_CACHE_HOME="$REAL_CACHE" \
    RSDK_API_BASE_URL="http://127.0.0.1:$PORT" \
    vhs demo/demo.tape
