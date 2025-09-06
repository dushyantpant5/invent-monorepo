#!/usr/bin/env bash
set -euo pipefail

PORT="${PORT:-8080}"

ROOT="$(cd "$(dirname "$0")" && pwd)"

TRA_BIN="$ROOT/traefik"
STATIC_CFG="$ROOT/traefik.yml"
TEMPLATE="$ROOT/dynamic.yml.template"
DYNAMIC_CFG="$ROOT/dynamic.yml"

echo "start-traefik.sh: ROOT=$ROOT, PORT=$PORT"
ls -la "$ROOT" || true

# Ensure template exists
if [ ! -f "$TEMPLATE" ]; then
  echo "ERROR: dynamic template not found at $TEMPLATE"
  echo "Place dynamic.yml.template in the same folder as this script."
  exit 1
fi

# Render template -> dynamic.yml (expand env vars)
echo "Rendering dynamic config from template..."
if command -v envsubst >/dev/null 2>&1; then
  envsubst < "$TEMPLATE" > "$DYNAMIC_CFG"
else
  # Fallback to python (handles ${VAR} and ${VAR:-default})
  python3 - <<'PY' > "$DYNAMIC_CFG"
import os, re, sys
tmpl = open("""$TEMPLATE""".replace('"""','\\"\"\"'), "r", encoding="utf-8").read()
def repl(m):
    var = m.group('var')
    default = m.group('def') if m.group('def') is not None else ''
    return os.environ.get(var, default)
pattern = re.compile(r'\$\{(?P<var>[A-Za-z_][A-Za-z0-9_]*)' +
                     r'(?:\:-(?P<def>[^}]*))?\}')
out = pattern.sub(repl, tmpl)
sys.stdout.write(out)
PY
fi

# show rendered head for debugging
echo "Rendered dynamic.yml (first 40 lines):"
head -n 40 "$DYNAMIC_CFG" || true

# sanity checks
if [ ! -x "$TRA_BIN" ]; then
  echo "ERROR: traefik binary not found or not executable at $TRA_BIN"
  ls -la "$ROOT" || true
  exit 1
fi
if [ ! -f "$STATIC_CFG" ]; then
  echo "ERROR: static config not found at $STATIC_CFG"
  exit 1
fi
if [ ! -f "$DYNAMIC_CFG" ]; then
  echo "ERROR: rendered dynamic config not created at $DYNAMIC_CFG"
  exit 1
fi

# start Traefik using the rendered dynamic file
exec "$TRA_BIN" \
  --configFile="$STATIC_CFG" \
  --providers.file.filename="$DYNAMIC_CFG" \
  --providers.file.watch=true \
  --entryPoints.web.address=":${PORT}" \
  --log.level=DEBUG