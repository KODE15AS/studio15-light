#!/usr/bin/env bash
# Konfetti-testen (maskinell del) — V1s akseptansetest, kjøres på raven.
# Verifiserer hele kjeden med et engangsprosjekt som slettes sporløst:
#
#   1. lobby:    program + prosjekt + arbeidsflate via API
#   2. editor:   code-server svarer gjennom caddy (HTTPS, sti-prefiks)
#   3. webside:  Vite svarer gjennom caddy med malens innhold
#   4. proxy:    chat-kall fra ARBEIDSFLATENS nett med arbeidsflatens
#                nøkkel → modellsvar (Fable 5, drop_params-kjeden)
#   5. levende:  filredigering i containeren → endringen synlig på websiden
#   6. sletting: prosjektet fjernes — containere, volumer og repo borte
#
# Zoo Code-leddet (modus → proxy fra selve chatten) testes i nettleser —
# se testreseptene i README. Idempotent: kan kjøres igjen ved feil.
set -euo pipefail

BASE="${S15L_BASE:-https://cadify104raven.tail14de1b.ts.net:8100}"
PROGRAM="maskintest"
PROSJEKT="konfetti"
DELTAGER="robot"
KORT="$PROGRAM-$PROSJEKT-$DELTAGER"
CONTAINER="s15l-ws-$KORT"

rod()   { printf '\033[31m%s\033[0m\n' "$*"; }
gronn() { printf '\033[32m%s\033[0m\n' "$*"; }
feil()  { rod "FEIL: $*"; exit 1; }

api() { # api METODE STI [JSON]
  local metode="$1" sti="$2" kropp="${3:-}"
  if [ -n "$kropp" ]; then
    curl -fsS -X "$metode" "$BASE$sti" -H "Content-Type: application/json" -d "$kropp"
  else
    curl -fsS -X "$metode" "$BASE$sti"
  fi
}

vent_paa() { # vent_paa BESKRIVELSE SEKUNDER KOMMANDO...
  local hva="$1" frist="$2"; shift 2
  local start=$SECONDS
  until "$@" >/dev/null 2>&1; do
    if [ $((SECONDS - start)) -gt "$frist" ]; then
      feil "$hva kom ikke opp innen ${frist}s"
    fi
    sleep 3
  done
  gronn "  ✓ $hva ($((SECONDS - start))s)"
}

rydd() {
  api DELETE "/api/prosjekter/$PROGRAM/$PROSJEKT" "{\"bekreft\": \"$PROSJEKT\"}" >/dev/null 2>&1 || true
  api DELETE "/api/programmer/$PROGRAM" "{\"bekreft\": \"$PROGRAM\"}" >/dev/null 2>&1 || true
}

echo "== Konfetti-testen (maskinell) mot $BASE =="

# 0) Lobbyen må svare
vent_paa "lobbyen (/healthz)" 15 curl -fsS "$BASE/healthz"

# Rydd rester fra en tidligere avbrutt kjøring
rydd

# 1) Program + prosjekt + arbeidsflate
api POST /api/programmer '{"navn": "Maskintest"}' >/dev/null 2>&1 \
  || true # finnes kanskje fra før — idempotent
api POST /api/prosjekter "{\"program\": \"$PROGRAM\", \"navn\": \"Konfetti\"}" >/dev/null
gronn "  ✓ program + prosjekt opprettet (repo seedet fra malen)"
api POST /api/arbeidsflater \
  "{\"program\": \"$PROGRAM\", \"prosjekt\": \"$PROSJEKT\", \"deltager\": \"$DELTAGER\"}" >/dev/null
gronn "  ✓ arbeidsflate opprettet"

# 2) Editoren gjennom caddy
vent_paa "code-server (/w/$KORT/healthz)" 90 curl -fsS "$BASE/w/$KORT/healthz"

# 3) Websiden gjennom caddy (npm install tar tid første gang)
sjekk_webside() {
  curl -fsS "$BASE/web/$KORT/" | grep -q "Konfetti"
}
vent_paa "websiden (/web/$KORT/)" 300 sjekk_webside

# 4) Proxy → modell fra arbeidsflatens nett, med arbeidsflatens nøkkel
SVAR=$(docker exec "$CONTAINER" sh -c '
  curl -fsS "$LLM_PROXY_BASE/chat/completions" \
    -H "Authorization: Bearer $LLM_PROXY_KEY" \
    -H "Content-Type: application/json" \
    -d "{\"model\": \"standard\", \"temperature\": 0, \"messages\": [{\"role\": \"user\", \"content\": \"Svar med kun ett ord: KONFETTI\"}]}"
')
echo "$SVAR" | grep -qi "konfetti" || feil "modellsvaret inneholdt ikke KONFETTI: $SVAR"
gronn "  ✓ proxy → Fable 5 svarte (temperature=0 droppet av drop_params)"

# 5) Filredigering → synlig på den levende websiden
MERKE="KONFETTI-$(date +%s)"
docker exec -u coder "$CONTAINER" sh -c \
  "sed -i 's|Websiden din lever|$MERKE|' /home/coder/project/src/App.svelte"
sjekk_live() {
  curl -fsS "$BASE/web/$KORT/src/App.svelte" | grep -q "$MERKE"
}
vent_paa "endringen på websiden ($MERKE)" 30 sjekk_live

# 6) Sikkerhetsvaktene (testfunn 22): arbeidsflaten skal IKKE nå lobbyen
if docker exec "$CONTAINER" sh -c 'curl -fsS -m 5 -k https://s15l-caddy:8100/api/tilstand' >/dev/null 2>&1; then
  feil "arbeidsflaten når lobby-API-et gjennom caddy — vakten er nede!"
fi
gronn "  ✓ caddy avviser arbeidsflate-nettet (403-vakten)"
if docker exec "$CONTAINER" sh -c 'curl -fsS -m 5 http://s15l-lobby:8200/healthz' >/dev/null 2>&1; then
  feil "arbeidsflaten når lobbyen direkte — nettskillet er brutt!"
fi
gronn "  ✓ arbeidsflaten har ingen rute til lobbyen (internt nett)"

# 7) Sletting er sletting
api DELETE "/api/prosjekter/$PROGRAM/$PROSJEKT" "{\"bekreft\": \"$PROSJEKT\"}" >/dev/null
docker ps -a --format '{{.Names}}' | grep -q "^$CONTAINER$" && feil "containeren finnes fortsatt"
docker volume ls --format '{{.Name}}' | grep -q "^$CONTAINER$" && feil "volumet finnes fortsatt"
api DELETE "/api/programmer/$PROGRAM" "{\"bekreft\": \"$PROGRAM\"}" >/dev/null
gronn "  ✓ sletting fjernet container, volum, repo og testprogrammet"

echo
gronn "KONFETTI-TESTEN (maskinell del) ER GRØNN 🎉"
echo "Gjenstår i nettleser: Zoo Code-kjeden (modus → proxy fra chatten) —"
echo "se testreseptene i README."
