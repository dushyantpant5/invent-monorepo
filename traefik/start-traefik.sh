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
echo "Listing files in $ROOT:"
ls -la "$ROOT" || true

# -----------------------
# Ensure Traefik binary
# -----------------------
if [ ! -x "$TRA_BIN" ]; then
  echo "Traefik binary not found at $TRA_BIN — downloading..."
  if ! curl -sSL --fail "$TARBALL_URL" -o "$TMP_TGZ"; then
    echo "ERROR: failed to download $TARBALL_URL"
    exit 1
  fi

  echo "Tarball contents (diagnostic):"
  tar -tzf "$TMP_TGZ" || true

  echo "Extracting binary to $TRA_BIN ..."
  # extract into $ROOT and ensure the traefik file ends up at $TRA_BIN
  tar -xzf "$TMP_TGZ" -C "$ROOT" traefik || {
    echo "ERROR: tar extraction failed or 'traefik' not present in archive"
    rm -f "$TMP_TGZ"
    exit 1
  }

  chmod +x "$TRA_BIN" || true
  rm -f "$TMP_TGZ"
  echo "Traefik binary is ready at $TRA_BIN"
else
  echo "Traefik binary already present and executable at $TRA_BIN"
fi

# -----------------------
# Render dynamic.yml from template
# -----------------------
if [ ! -f "$TEMPLATE" ]; then
  echo "ERROR: $TEMPLATE not found in $ROOT"
  ls -la "$ROOT"
  exit 1
fi

echo "Rendering $DYNAMIC_CFG from $TEMPLATE ..."
if command -v envsubst >/dev/null 2>&1; then
  envsubst < "$TEMPLATE" > "$DYNAMIC_CFG"
else
  # Python fallback - uses $TEMPLATE variable
  python3 - <<PY > "$DYNAMIC_CFG"
import os, re, sys
tmpl = open(os.path.expanduser("$TEMPLATE"), "r", encoding="utf-8").read()
def repl(m):
    var = m.group('var')
    default = m.group('def') if m.group('def') is not None else ''
    return os.environ.get(var, default)
pattern = re.compile(r'\$\{(?P<var>[A-Za-z_][A-Za-z0-9_]*)' +
                     r'(?:\:-(?P<def>[^}]*))?\}')
sys.stdout.write(pattern.sub(repl, tmpl))
PY
fi

echo "Rendered $DYNAMIC_CFG (first 40 lines):"
head -n 40 "$DYNAMIC_CFG" || true

# -----------------------
# Final checks & Start Traefik
# -----------------------
if [ ! -x "$TRA_BIN" ]; then
  echo "ERROR: traefik binary not executable at $TRA_BIN"
  ls -la "$ROOT"
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

echo "Starting Traefik..."
exec "$TRA_BIN" \
  --configFile="$STATIC_CFG" \
  --providers.file.filename="$DYNAMIC_CFG" \
  --providers.file.watch=true \
  --entryPoints.web.address=":${PORT}" \
  --log.level=DEBUG
