#!/usr/bin/env bash
set -euo pipefail

# macOS-friendly run-all: start product + traefik, tail logs, detect exit of either process.

# load .env if present
[ -f .env ] && . .env || true

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOGDIR="$ROOT/tmp"
mkdir -p "$LOGDIR"

# Start product
echo "Starting product service..."
(
  cd "$ROOT/services/product"
  # run product, send stdout/stderr to logfile
  cargo run -p product > "$LOGDIR/product.log" 2>&1 &
  echo $! > "$LOGDIR/product.pid"
)
sleep 0.6

# Start traefik
echo "Starting traefik..."
(
  cd "$ROOT/traefik"
  ./start-traefik.sh > "$LOGDIR/traefik.log" 2>&1 &
  echo $! > "$LOGDIR/traefik.pid"
)

echo "Logs: $LOGDIR/product.log  $LOGDIR/traefik.log"
echo "Tailing logs — press Ctrl-C to stop (processes keep running)."

# tail logs in background
tail -n +1 -F "$LOGDIR/product.log" "$LOGDIR/traefik.log" &
TAIL_PID=$!

# Helper to read PID files
get_pid() {
  local f="$1"
  [ -f "$f" ] || return 1
  read -r pid < "$f"
  printf "%s" "$pid"
}

# Monitor loop (portable)
PRODUCT_PID=$(get_pid "$LOGDIR/product.pid" || echo "")
TRAEFIK_PID=$(get_pid "$LOGDIR/traefik.pid" || echo "")

# If either pid empty, fail early
if [ -z "$PRODUCT_PID" ] || [ -z "$TRAEFIK_PID" ]; then
  echo "ERROR: missing pid file(s). product=$PRODUCT_PID traefik=$TRAEFIK_PID"
  kill "$TAIL_PID" 2>/dev/null || true
  exit 1
fi

# Poll loop: check every 0.5s whether either pid is dead
while true; do
  if ! kill -0 "$PRODUCT_PID" 2>/dev/null; then
    echo "Product process $PRODUCT_PID has exited."
    break
  fi
  if ! kill -0 "$TRAEFIK_PID" 2>/dev/null; then
    echo "Traefik process $TRAEFIK_PID has exited."
    break
  fi
  sleep 0.5
done

# One of them exited — show recent logs
echo "One of the services exited. Showing last 200 lines of logs:"
sleep 0.2
echo "== PRODUCT LOG (last 200) =="
tail -n 200 "$LOGDIR/product.log" || true
echo "== TRAEFIK LOG (last 200) =="
tail -n 200 "$LOGDIR/traefik.log" || true

# cleanup
kill "$TAIL_PID" 2>/dev/null || true
exit 1
