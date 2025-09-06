#!/usr/bin/env bash
set -euo pipefail

# Render provides $PORT for incoming traffic
PORT="${PORT:-8080}"

# SCRIPT_DIR -> absolute path to the traefik/ folder (where this script lives)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

TRAefik_BIN="$SCRIPT_DIR/traefik"       # binary will be at traefik/traefik
STATIC_CFG="$SCRIPT_DIR/traefik.yml"    # static config at traefik/traefik.yml
DYNAMIC_CFG="$SCRIPT_DIR/dynamic.yml"   # dynamic config at traefik/dynamic.yml

# Diagnostics (helpful in Render logs)
echo "SCRIPT_DIR=$SCRIPT_DIR"
echo "TRAefik_BIN=$TRAefik_BIN"
echo "STATIC_CFG=$STATIC_CFG"
echo "DYNAMIC_CFG=$DYNAMIC_CFG"
echo "PORT=$PORT"

# Sanity checks
if [ ! -x "$TRAefik_BIN" ]; then
  echo "ERROR: traefik binary not found or not executable at $TRAefik_BIN"
  ls -la "$SCRIPT_DIR" || true
  exit 1
fi
if [ ! -f "$STATIC_CFG" ]; then
  echo "ERROR: static config not found at $STATIC_CFG"
  ls -la "$SCRIPT_DIR" || true
  exit 1
fi
if [ ! -f "$DYNAMIC_CFG" ]; then
  echo "ERROR: dynamic config not found at $DYNAMIC_CFG"
  ls -la "$SCRIPT_DIR" || true
  exit 1
fi

# Exec Traefik with explicit dynamic file + entrypoint bound to Render $PORT
exec "$TRAefik_BIN" \
  --configFile="$STATIC_CFG" \
  --providers.file.filename="$DYNAMIC_CFG" \
  --providers.file.watch=true \
  --entryPoints.web.address=":${PORT}" \
  --log.level=DEBUG