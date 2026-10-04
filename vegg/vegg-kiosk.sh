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

# Aldri mer enn ÉN vakt (funn 04.10): relogin starter autostarten på nytt
# mens en eksisterende vakt kan leve videre — to vakter slåss om kiosken.
# flock holder låsen så lenge vakta lever; nykommere avslutter stille.
LAAS="${XDG_RUNTIME_DIR:-/tmp}/s15l-vegg-kiosk.laas"
exec 9>"$LAAS"
if ! flock -n 9; then
  log "En annen vegg-kiosk-vakt kjører allerede — avslutter."
  exit 0
fi

tjeneste_oppe() {
  curl -fsS --max-time 2 -o /dev/null "$HEALTH_URL" 2>/dev/null
}

# Fjern-restart fra setene (Jørn 04.10): lobbyen holder et tidsstempel som
# settes av «↻ Vegg»-knappen i samlingen. Nyere stempel enn sist sett →
# kiosken skytes og vakta starter den friskt. Dette er eneste vei når
# veggsiden er frossen eller renderprosessen har krasjet (vakta ser ellers
# bare om PROSESSEN lever). Initialiseres ved oppstart så gamle trykk
# ikke gir restart ved boot.
RESTART_URL="${VEGG_RESTART_URL:-$BASE/api/vegg/restart}"
restart_stempel() {
  curl -fsS --max-time 2 "$RESTART_URL" 2>/dev/null \
    | sed -n 's/.*"sist":\([0-9]*\).*/\1/p'
}
RESTART_SETT="$(restart_stempel)"
RESTART_SETT="${RESTART_SETT:-0}"

restart_forespurt() {
  local naa
  naa="$(restart_stempel)"
  [ -n "$naa" ] && [ "$naa" -gt "$RESTART_SETT" ] || return 1
  RESTART_SETT="$naa"
  return 0
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

# Kiosken spores via PID-en til prosessen som EIER KIOSK-VINDUET (wmctrl).
# Ingenting annet er til å stole på: snap-kjeden forker underveis (så $!
# fra launcheren dør), og argv skifter under oppstart (så pgrep -f på
# profilstien flagrer falsk negativt). Begge deler fikk vakta til å
# dobbeltstarte kiosken, som igjen slettet profilen under den kjørende —
# garble/krasj på 70-tommeren (funn 04.10).
KIOSK_PID=""
kiosk_kjorer() {
  [ -n "$KIOSK_PID" ] && kill -0 "$KIOSK_PID" 2>/dev/null
}

# Vent til kiosk-vinduet finnes (match på PID med vår profil — aldri
# tittel/klasse), noter eier-PID-en og tving fullskjerm. Blokkerer vakta
# til vinduet er der; det er poenget — før vinduet finnes VET vi ikke at
# kiosken lever, og da skal det heller ikke startes flere.
vent_paa_kiosk() {
  local i pid wid
  if ! command -v wmctrl >/dev/null 2>&1; then
    log "MERK: wmctrl mangler — installer: sudo apt install -y wmctrl"
    return 1
  fi
  for i in $(seq 1 40); do
    sleep 1
    for pid in $(pgrep -f -- "user-data-dir=$PROFIL" 2>/dev/null); do
      wid="$(wmctrl -lp 2>/dev/null | awk -v p="$pid" '$3==p {print $1; exit}')"
      [ -n "$wid" ] || continue
      KIOSK_PID="$pid"
      wmctrl -i -r "$wid" -b add,fullscreen 2>/dev/null
      log "Kiosken er oppe (PID $pid, vindu $wid) — fullskjerm tvunget"
      return 0
    done
  done
  log "MERK: fant aldri kiosk-vinduet etter start"
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
    # Programvare-rendering (funn 04.10): raven er hybrid Intel-iGPU
    # (driver TV-en) + NVIDIA — GPU-kompositoren ga garble, hvite felter,
    # gjenliggende spøkelsesrammer («lagvise instanser») og krasj ved
    # klikk. Veggen er statisk visning; CPU-raster er stabilt og raskt nok.
    --disable-gpu
    # Engelsk nettleserlokale: ellers anbefaler code-server norsk språk-
    # pakke med en varsling oppå hver editor-flis (funn 04.10).
    --lang=en-US
    --user-data-dir="$PROFIL"
  )
  if [ -n "$POSISJON" ]; then
    args+=(--window-position="$POSISJON")
  fi
  args+=("$URL")
  # Rydd eventuelle foreldreløse kiosker (f.eks. etter vakt-restart) FØR
  # profilen slettes — aldri rm under en kjørende instans.
  pkill -f -- "user-data-dir=$PROFIL" 2>/dev/null && sleep 1
  rm -rf "$PROFIL" # frisk profil hver gang
  log "70\"-skjerm oppdaget (match: $MATCH) — starter kiosk: $NETTLESER"
  nohup "$NETTLESER" "${args[@]}" >/dev/null 2>&1 &
  vent_paa_kiosk
}

stopp_kiosk() {
  log "${1:-Skjermen er koblet fra eller tjenesten er nede} — lukker kiosken"
  [ -n "$KIOSK_PID" ] && kill "$KIOSK_PID" 2>/dev/null
  pkill -f -- "user-data-dir=$PROFIL" 2>/dev/null
  KIOSK_PID=""
}

log "Starter. Ser etter skjerm med EDID-match «$MATCH», intervall ${POLL}s."
log "Kiosken er helse-gatet mot $HEALTH_URL."

while true; do
  if tjeneste_oppe && vegg_skjerm_tilkoblet; then
    if kiosk_kjorer && restart_forespurt; then
      stopp_kiosk "Fjern-restart forespurt fra et sete"
      sleep 1
    fi
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
