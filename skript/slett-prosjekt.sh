#!/usr/bin/env bash
# Sletteregimet («sletting er sletting», beslutning 03.10): sletter et
# prosjekts arbeidsflater (containere), volumer og prosjektrepo i ÉN
# operasjon, etter eksplisitt bekreftelse. Går via lobby-API-et slik at
# det bare finnes én slettevei.
#
# Bruk: skript/slett-prosjekt.sh <program-slug> <prosjekt-slug>
set -euo pipefail

PROGRAM="${1:?Bruk: slett-prosjekt.sh <program-slug> <prosjekt-slug>}"
PROSJEKT="${2:?Bruk: slett-prosjekt.sh <program-slug> <prosjekt-slug>}"
BASE="${S15L_BASE:-https://cadify104raven.tail14de1b.ts.net:8100}"

echo "Dette sletter prosjektet «$PROSJEKT» i programmet «$PROGRAM» FOR ALLTID:"
echo "  - alle arbeidsflate-containere og -volumer"
echo "  - prosjektrepoet på repos-volumet"
echo "  - oppføringen i register/programmer.yaml"
echo
read -r -p "Skriv prosjektets slug for å bekrefte: " BEKREFT
if [ "$BEKREFT" != "$PROSJEKT" ]; then
  echo "Avbrutt — bekreftelsen stemte ikke."
  exit 1
fi

curl -fsS -X DELETE "$BASE/api/prosjekter/$PROGRAM/$PROSJEKT" \
  -H "Content-Type: application/json" \
  -d "{\"bekreft\": \"$PROSJEKT\"}"
echo
echo "Manuelt gjenstår (til vaktmester-appen finnes): slett eventuelt"
echo "GitHub-repo, og GitHub-org-en hvis hele programmet legges ned"
echo "(org-sletting er GitHubs ene manuelle unntak)."
