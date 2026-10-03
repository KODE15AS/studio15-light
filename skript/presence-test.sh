#!/usr/bin/env bash
# Presence-smoketesten (V2): kjører 16-punkts protokolltesten i
# presence-test.mjs mot den kjørende stacken, med kort reconnect-vindu
# (PRESENCE_TIMEOUT_SECS=2) og et engangsprosjekt med én arbeidsflate.
# Idempotent: rydder selv, og setter lobbyen tilbake til normalt vindu.
# Krever node >= 22 (global WebSocket) på maskinen som kjører testen.
set -euo pipefail
cd "$(dirname "$0")/.."

BASE="${S15L_BASE:-https://cadify104raven.tail14de1b.ts.net:8100}"
PROGRAM="maskintest-presence"
PROSJEKT="presence"
DELTAGER="alfa"
KORT="$PROGRAM-$PROSJEKT-$DELTAGER"

api() {
  local metode="$1" sti="$2" kropp="${3:-}"
  if [ -n "$kropp" ]; then
    curl -fsS -X "$metode" "$BASE$sti" -H "Content-Type: application/json" -d "$kropp"
  else
    curl -fsS -X "$metode" "$BASE$sti"
  fi
}

rydd() {
  api DELETE "/api/prosjekter/$PROGRAM/$PROSJEKT" "{\"bekreft\": \"$PROSJEKT\"}" >/dev/null 2>&1 || true
  api DELETE "/api/programmer/$PROGRAM" "{\"bekreft\": \"$PROGRAM\"}" >/dev/null 2>&1 || true
}

vent_lobby() {
  for _ in $(seq 1 20); do
    curl -fsS "$BASE/healthz" >/dev/null 2>&1 && return 0
    sleep 1
  done
  echo "FEIL: lobbyen kom ikke opp"; exit 1
}

echo "== Forbereder: lobby med PRESENCE_TIMEOUT_SECS=2 + testfixturer =="
PRESENCE_TIMEOUT_SECS=2 docker compose up -d lobby >/dev/null 2>&1
vent_lobby
rydd
api POST /api/programmer '{"navn": "Maskintest Presence"}' >/dev/null 2>&1 || true
api POST /api/prosjekter "{\"program\": \"$PROGRAM\", \"navn\": \"Presence\"}" >/dev/null
api POST /api/arbeidsflater \
  "{\"program\": \"$PROGRAM\", \"prosjekt\": \"$PROSJEKT\", \"deltager\": \"$DELTAGER\"}" >/dev/null

STATUS=0
node skript/presence-test.mjs "$BASE" "$PROGRAM" "$PROSJEKT" "$KORT" || STATUS=$?

echo "== Rydder: fixturer bort, lobby tilbake til normalt vindu =="
rydd
docker compose up -d lobby >/dev/null 2>&1
vent_lobby
exit $STATUS
