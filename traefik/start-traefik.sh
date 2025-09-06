#!/usr/bin/env bash
set -euo pipefail

# -----------------------
# Config
# -----------------------
PORT="${PORT:-8080}"
ROOT="$(cd "$(dirname "$0")" && pwd)"

TRA_BIN="$ROOT/traefik"
STATIC_CFG="$ROOT/traefik.yml"
TEMPLATE="$ROOT/dynamic.yml.template"
DYNAMIC_CFG="$ROOT/dynamic.yml"
TARBALL_URL="https://github.com/traefik/traefik/releases/download/v3.5.1/traefik_v3.5.1_linux_amd64.tar.gz"
TMP_TGZ="$ROOT/traefik.tgz"

echo "start-traefik.sh: ROOT=$ROOT, PORT=$PORT"
ls -la "$ROOT" || true

# -----------------------
# Ensure Traefik binary
# -----------------------
if [ ! -x "$TRA_BIN" ]; then
  echo "Traefik binary not found — downloading..."
  curl -sSL --fail "$TARBALL_URL" -o "$TMP_TGZ"
  echo "Extracting binary..."
  tar -xzf "$TMP_TGZ" traefik
  chmod +x traefik
  rm -f "$TMP_TGZ"
fi

# -----------------------
# Render dynamic.yml from template
# -----------------------
if [ ! -f "$TEMPLATE" ]; then
  echo "ERROR: $TEMPLATE not found"
  exit 1
fi

echo "Rendering $DYNAMIC_CFG from $TEMPLATE ..."
if command -v envsubst >/dev/null 2>&1; then
  envsubst < "$TEMPLATE" > "$DYNAMIC_CFG"
else
  python3 - <<'PY' > "$DYNAMIC_CFG"
import os, re, sys
tmpl = open("traefik/dynamic.yml.template","r",encoding="utf-8").read()
def repl(m):
    var = m.group('var')
    default = m.group('def') or ''
    return os.environ.get(var, default)
pattern = re.compile(r'\$\{(?P<var>[A-Za-z_][A-Za-z0-9_]*)' +
                     r'(?:\:-(?P<def>[^}]*))?\}')
sys.stdout.write(pattern.sub(repl, tmpl))
PY
fi

echo "Rendered $DYNAMIC_CFG (first 20 lines):"
head -n 20 "$DYNAMIC_CFG"

# -----------------------
# Start Traefik
# -----------------------
exec "$TRA_BIN" \
  --configFile="$STATIC_CFG" \
  --providers.file.filename="$DYNAMIC_CFG" \
  --providers.file.watch=true \
  --entryPoints.web.address=":${PORT}" \
  --log.level=DEBUG