#!/usr/bin/env bash
# Studio 15 LIGHT — vegg-kiosk for Ravens 70"-skjerm.
#
# Kjøres i Ravens desktop-session (autostart eller terminal på desktopen —
# ALDRI over SSH). Overvåker DRM-tilkoblingene og starter Chromium i
# kiosk-modus på /vegg når 70"-skjermen (EDID-match) er tilkoblet OG
# tjenesten svarer på /healthz. Skjerm borte eller tjeneste nede → kiosken
# lukkes. Veggen er alltid read-only (watch-modus i appen).
#
# Alle kiosk-fellene fra Skjermsamling (erfaringsoverføring D) er med:
#   - Chromium ignorerer --kiosk på Wayland: --ozone-platform=x11 +
#     fullskjerm tvinges med wmctrl, matchet på PID (aldri tittel/klasse)
#   - DISPLAY-vakt med tydelig feilmelding (ellers spinner det stille)
#   - frisk --user-data-dir per start (gammel vindusstørrelse gjenbrukes aldri)
#   - fast skjerm, ingen hotplug; VEGG_POSISJON (xrandr-koordinat) styrer
#     hvilken skjerm kiosken tar når flere er tilkoblet
#
# Konfigurasjon (miljøvariabler):
#   VEGG_MATCH      Tekst i skjermens EDID (default "SAMSUNG")
#   VEGG_URL        URL som vises (default Studio 15 LIGHT /vegg)
#   VEGG_HEALTH_URL Helsesjekk (default /healthz samme sted)
#   VEGG_BROWSER    Nettleser-kommando (default: første chromium/chrome)
#   VEGG_POLL_SECS  Sjekkintervall i sekunder (default 3)
#   VEGG_POSISJON   Valgfri "X,Y" vindusposisjon (xrandr --query)

set -u

BASE="${S15L_BASE:-https://cadify104raven.tail14de1b.ts.net:8100}"
MATCH="${VEGG_MATCH:-SAMSUNG}"
URL="${VEGG_URL:-$BASE/vegg}"
HEALTH_URL="${VEGG_HEALTH_URL:-$BASE/healthz}"
POLL="${VEGG_POLL_SECS:-3}"
POSISJON="${VEGG_POSISJON:-}"
PROFIL="${XDG_RUNTIME_DIR:-/tmp}/s15l-vegg-profil"

log() { echo "[vegg-kiosk] $(date '+%H:%M:%S') $*"; }

finn_nettleser() {
  if [ -n "${VEGG_BROWSER:-}" ]; then
    echo "$VEGG_BROWSER"
    return
  fi
  for b in chromium chromium-browser google-chrome; do
    if command -v "$b" >/dev/null 2>&1; then
      echo "$b"
      return
    fi
  done
  echo ""
}

# DISPLAY-vakten: uten grafisk session kan Chromium aldri nå skjermen.
if [ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then
  log "FEIL: ingen grafisk session (DISPLAY/WAYLAND_DISPLAY er tomme)."
  log "Kiosken må startes fra Ravens desktop — ikke over SSH."
  log "(Ved innlogging på desktopen starter den automatisk via autostart.)"
  exit 1
fi

NETTLESER="$(finn_nettleser)"
if [ -z "$NETTLESER" ]; then
  log "FEIL: fant ingen chromium/chrome i PATH. Installer: sudo snap install chromium"
  exit 1
fi

tjeneste_oppe() {
  curl -fsS --max-time 2 -o /dev/null "$HEALTH_URL" 2>/dev/null
}

# 70"-skjermen gjenkjennes på EDID-innholdet (produsentnavn som ASCII).
vegg_skjerm_tilkoblet() {
  local st dir
  for st in /sys/class/drm/card*-*/status; do
    [ -e "$st" ] || continue
    [ "$(cat "$st" 2>/dev/null)" = "connected" ] || continue
    dir="$(dirname "$st")"
    if strings "$dir/edid" 2>/dev/null | grep -qi -- "$MATCH"; then
      return 0
    fi
  done
  return 1
}

kiosk_kjorer() {
  pgrep -f -- "user-data-dir=$PROFIL" >/dev/null 2>&1
}

# Fullskjerm tvinges på vindusbehandler-nivå: match vinduet på PID (aldri
# tittel/klasse — da risikerer man andres vinduer).
tving_fullskjerm() {
  local i pid wid
  for i in $(seq 1 30); do
    sleep 1
    pid="$(pgrep -of -- "user-data-dir=$PROFIL" 2>/dev/null)"
    [ -n "$pid" ] || continue
    if command -v wmctrl >/dev/null 2>&1; then
      wid="$(wmctrl -lp 2>/dev/null | awk -v p="$pid" '$3==p {print $1; exit}')"
      [ -n "$wid" ] || continue
      wmctrl -i -r "$wid" -b add,fullscreen 2>/dev/null
      log "Tvang kiosk-vinduet til fullskjerm (wmctrl)"
      return 0
    elif command -v xdotool >/dev/null 2>&1; then
      wid="$(xdotool search --pid "$pid" --onlyvisible 2>/dev/null | head -1)"
      [ -n "$wid" ] || continue
      xdotool key --window "$wid" F11 2>/dev/null
      log "Tvang kiosk-vinduet til fullskjerm (xdotool)"
      return 0
    else
      log "MERK: verken wmctrl eller xdotool finnes — installer: sudo apt install -y wmctrl"
      return 1
    fi
  done
  log "MERK: fant aldri kiosk-vinduet — fullskjerm ble ikke tvunget"
  return 1
}

start_kiosk() {
  # --ozone-platform=x11: kiosk-modus fungerer pålitelig OG vinduet blir
  # synlig for wmctrl (XWayland). Harmløst på ren X11.
  local args=(
    --ozone-platform=x11
    --kiosk
    --start-fullscreen
    --noerrdialogs
    --disable-session-crashed-bubble
    # Ingen «oversett siden?»-bar over veggen (funn 04.10)
    --disable-features=Translate
    --user-data-dir="$PROFIL"
  )
  if [ -n "$POSISJON" ]; then
    args+=(--window-position="$POSISJON")
  fi
  args+=("$URL")
  rm -rf "$PROFIL" # frisk profil hver gang
  log "70\"-skjerm oppdaget (match: $MATCH) — starter kiosk: $NETTLESER"
  nohup "$NETTLESER" "${args[@]}" >/dev/null 2>&1 &
  tving_fullskjerm &
}

stopp_kiosk() {
  log "Skjermen er koblet fra eller tjenesten er nede — lukker kiosken"
  pkill -f -- "user-data-dir=$PROFIL" 2>/dev/null
}

log "Starter. Ser etter skjerm med EDID-match «$MATCH», intervall ${POLL}s."
log "Kiosken er helse-gatet mot $HEALTH_URL."

while true; do
  if tjeneste_oppe && vegg_skjerm_tilkoblet; then
    if ! kiosk_kjorer; then
      sleep 2 # gi desktopen et øyeblikk til å aktivere skjermen
      start_kiosk
    fi
  else
    if kiosk_kjorer; then
      stopp_kiosk
    fi
  fi
  sleep "$POLL"
done
