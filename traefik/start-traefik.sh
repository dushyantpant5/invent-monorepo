#!/usr/bin/env bash
set -euo pipefail

PORT="${PORT:-8080}"

ROOT="$(cd "$(dirname "$0")" && pwd)"

TRA_BIN="$ROOT/traefik"
STATIC_CFG="$ROOT/traefik.yml"
DYNAMIC_CFG="$ROOT/dynamic.yml"

echo "start-traefik.sh: ROOT=$ROOT, PORT=$PORT"
ls -la "$ROOT" || true

if [ ! -x "$TRA_BIN" ]; then
  echo "ERROR: traefik binary not found or not executable at $TRA_BIN"
  exit 1
fi
if [ ! -f "$STATIC_CFG" ]; then
  echo "ERROR: static config not found at $STATIC_CFG"
  exit 1
fi
if [ ! -f "$DYNAMIC_CFG" ]; then
  echo "ERROR: dynamic config not found at $DYNAMIC_CFG"
  exit 1
fi

exec "$TRA_BIN" \
  --configFile="$STATIC_CFG" \
  --providers.file.filename="$DYNAMIC_CFG" \
  --providers.file.watch=true \
  --entryPoints.web.address=":${PORT}" \
  --log.level=DEBUG
