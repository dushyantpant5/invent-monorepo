set -euo pipefail
PORT="${PORT:-8080}"
if [ ! -f "./traefik" ]; then
  echo "Downloading Traefik..."
  curl -sSL https://github.com/traefik/traefik/releases/download/v3.5.1/traefik_v3.5.1_linux_amd64.tar.gz -o traefik.tgz
  tar -xzf traefik.tgz traefik
  chmod +x traefik
fi
exec ./traefik \
  --configFile=./traefik.yml \
  --entryPoints.web.address=":${PORT}" \
  --log.level=DEBUG