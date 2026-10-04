#!/usr/bin/env bash
# Henter/fornyer Let's Encrypt-sertifikatet for lobby.studio15.cloud
# (wifi-inngangen) via DNS-01 og GitOps-hooken deploi-dns01-hook.sh.
# Kjøres månedlig fra cron (fornyer kun når < 30 dager gjenstår) — og
# manuelt første gang. Kopierer inn i certs/ og restarter caddy ved endring.
set -euo pipefail

ROT="$(cd "$(dirname "$0")/.." && pwd)"
LEGO="${LEGO:-$HOME/.local/bin/lego}"
EPOST="37360334+Watvedt@users.noreply.github.com"
DOMENE="lobby.studio15.cloud"
LEGO_PATH="$ROT/certs/lego"

export EXEC_PATH="$ROT/skript/deploi-dns01-hook.sh"
export EXEC_PROPAGATION_TIMEOUT=600
export EXEC_POLLING_INTERVAL=10

# lego v5: flaggene hører til subkommandoen — env-variantene virker for begge.
export LEGO_ACCEPT_TOS=true LEGO_EMAIL="$EPOST" LEGO_DNS=exec \
       LEGO_DOMAINS="$DOMENE" LEGO_PATH="$LEGO_PATH"
# netims navnetjenere er anycast: propageringssjekken kan passere mot én node
# mens LE treffer en annen (NXDOMAIN 04.10). Fast ventetid i stedet for sjekk.
export LEGO_DNS_PROPAGATION_WAIT=300s

if [ -f "$LEGO_PATH/certificates/$DOMENE.crt" ]; then
  "$LEGO" renew
else
  "$LEGO" run
fi

# Kopier inn kun ved endring; restart caddy så nytt sertifikat tas i bruk.
if ! cmp -s "$LEGO_PATH/certificates/$DOMENE.crt" "$ROT/certs/$DOMENE.crt" 2>/dev/null; then
  cp "$LEGO_PATH/certificates/$DOMENE.crt" "$ROT/certs/$DOMENE.crt"
  cp "$LEGO_PATH/certificates/$DOMENE.key" "$ROT/certs/$DOMENE.key"
  chmod 644 "$ROT/certs/$DOMENE.crt"
  chmod 600 "$ROT/certs/$DOMENE.key"
  docker restart s15l-caddy >/dev/null
  echo "nytt sertifikat installert og caddy restartet"
else
  echo "sertifikatet er uendret"
fi
