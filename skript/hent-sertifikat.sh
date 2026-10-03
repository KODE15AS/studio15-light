#!/usr/bin/env bash
# Henter/fornyer Tailscale-sertifikatet for raven og legger det i certs/
# (gitignorert), der caddy leser det. HTTPS-beslutningen fra 03.10:
# secure context for clipboard/streaming uten offentlig CA eller ingress.
#
# Kjøres på raven ved oppsett og når sertifikatet nærmer seg utløp
# (tailscale cert fornyer automatisk når det trengs).
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p certs

VERT="cadify104raven.tail14de1b.ts.net"

# tailscale cert krever root eller operator-rettigheter
if tailscale cert --cert-file certs/raven.crt --key-file certs/raven.key "$VERT" 2>/dev/null; then
  :
else
  sudo tailscale cert --cert-file certs/raven.crt --key-file certs/raven.key "$VERT"
  sudo chown "$(id -u):$(id -g)" certs/raven.crt certs/raven.key
fi

chmod 600 certs/raven.key
echo "Sertifikat for $VERT ligger i certs/ — restart caddy ved fornyelse:"
echo "  docker compose restart caddy"
