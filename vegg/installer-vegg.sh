#!/usr/bin/env bash
# Installerer vegg-kiosken i Ravens autostart. Kjøres på raven (én gang).
#
# VIKTIG: Skjermsamlings wall-watcher bruker samme 70"-skjerm (samme
# EDID-match) — to kiosker kan ikke dele den. Skriptet flytter derfor
# Skjermsamlings autostart-fil til side (reversibelt) etter bekreftelse.
set -euo pipefail
cd "$(dirname "$0")"

AUTOSTART="$HOME/.config/autostart"
SKJERMSAMLING="$AUTOSTART/skjermsamling-wall-watcher.desktop"

# Klassisk felle: ~/.config/autostart må være en MAPPE (en fil med samme
# navn dreper all autostart stille).
if [ -e "$AUTOSTART" ] && [ ! -d "$AUTOSTART" ]; then
  echo "FEIL: $AUTOSTART finnes, men er ikke en mappe — fiks det først."
  exit 1
fi
mkdir -p "$AUTOSTART"

if [ -f "$SKJERMSAMLING" ]; then
  echo "Skjermsamlings wall-watcher er i autostart og deler 70\"-skjermen."
  read -r -p "Flytte den til side (${SKJERMSAMLING##*/}.avslaatt)? [j/N] " SVAR
  if [ "${SVAR,,}" = "j" ]; then
    mv "$SKJERMSAMLING" "$SKJERMSAMLING.avslaatt"
    pkill -f skjermsamling-wall-watcher 2>/dev/null || true
    pkill -f "skjermsamling-wall-profile" 2>/dev/null || true
    echo "Flyttet. (Angres med: mv '$SKJERMSAMLING.avslaatt' '$SKJERMSAMLING')"
  else
    echo "Avbrutt — to kiosker kan ikke dele skjermen."
    exit 1
  fi
fi

chmod +x vegg-kiosk.sh
cp vegg-kiosk.desktop "$AUTOSTART/"
echo "Installert: $AUTOSTART/vegg-kiosk.desktop"
echo
echo "Start nå (fra en terminal PÅ Ravens desktop, ikke SSH):"
echo "  ./vegg-kiosk.sh &"
echo "— eller logg ut/inn på desktopen, så starter den selv."
