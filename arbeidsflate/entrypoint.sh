#!/usr/bin/env bash
# Oppstart for arbeidsflaten (Studio 15 LIGHT):
#   1) klon prosjektrepoet hvis prosjektmappen er tom
#   2) render Zoo Code-config med proxyadresse (kun ved fersk container)
#   3) skriv regelfil med webside-adressen (aldri localhost — testfunn 9)
#   4) start Vite (den levende websiden) + autosave-commit + code-server
#
# Miljøvariabler satt av lobbyen ved opprettelse:
#   PROJECT_REPO    - git-URL til prosjektrepoet (file:///repos/<slug>.git
#                     til vaktmester-appen gir GitHub-repoer)
#   PARTICIPANT     - deltagerens navn (git-identitet)
#   WEB_BASE        - sti-prefiks for websiden, f.eks. /web/konfetti-jorn/
#   WEB_URL         - full adresse til websiden (https, via caddy)
#   EDITOR_URL      - full adresse til arbeidsflaten (til regelfila)
#   LLM_PROXY_BASE  - default http://s15l-litellm:4000/v1
#   LLM_PROXY_KEY   - felles master-nøkkel mot proxyen (beslutning 03.10);
#                     den ekte API-nøkkelen bor KUN i proxyen
set -euo pipefail

PROJECT_DIR="${PROJECT_DIR:-/home/coder/project}"
mkdir -p "$PROJECT_DIR"

git config --global user.name "${PARTICIPANT:-deltager}"
git config --global user.email "${PARTICIPANT:-deltager}@studio15-light.lokal"
git config --global init.defaultBranch main
# Prosjektrepoene på /repos deles mellom lobby og arbeidsflater; lukket
# system, så git sin eierskapssjekk («dubious ownership») er bare i veien.
git config --global --add safe.directory '*'

if [ -n "${PROJECT_REPO:-}" ] && [ -z "$(ls -A "$PROJECT_DIR")" ]; then
  git clone "$PROJECT_REPO" "$PROJECT_DIR"
fi

# Zoo Code: provider-profil (proxy + nøkkel) auto-importeres ved oppstart via
# zoo-code.autoImportSettingsPath. Importen overskriver brukerens egne valg
# (f.eks. terminal-autokjøring PÅ), så den kjøres kun når containeren er
# fersk (ingen globalStorage ennå). Restart/reload beholder dermed brukerens
# tilpasninger; en gjenskapt container starter på standardoppsettet.
ZOO_STORAGE=/home/coder/.local/share/code-server/User/globalStorage/zoocodeorganization.zoo-code
if [ ! -d "$ZOO_STORAGE" ]; then
  sed -e "s|__PROXY_BASE__|${LLM_PROXY_BASE:-http://s15l-litellm:4000/v1}|" \
      -e "s|__PROXY_KEY__|${LLM_PROXY_KEY:-}|" \
      /opt/s15l/zoo-settings.template.json > /home/coder/zoo-settings.json
else
  rm -f /home/coder/zoo-settings.json
fi

# Regelfil med webside-adressen — .roo/rules/ leses av Zoo Code (verifisert
# 03.10) og er gitignorert i prosjektmalen, så hver arbeidsflate kan ha sin
# egen adresse uten git-konflikter. localhost-fellen: testfunn 9.
mkdir -p "$PROJECT_DIR/.roo/rules"
cat > "$PROJECT_DIR/.roo/rules/01-webside.md" <<EOF
# Adresser i denne arbeidsflaten

- Prosjektets levende webside: ${WEB_URL:-ukjent — spør i lobbyen}
- Oppgi ALLTID denne adressen når brukeren spør hvor websiden er.
- ALDRI henvis til localhost eller 127.0.0.1 — det virker bare inne i
  containeren, ikke på brukerens maskin.
- Vite-utviklingsserveren kjører allerede og oppdaterer websiden automatisk;
  du trenger aldri starte eller installere noe for at siden skal vises.
EOF

# Fornuftige editor-innstillinger (barnesykdommene fra Studio 15):
# ingen trust-dialog (funn 2), ingen velkomstside, autolagring (avgjørende:
# endringer når websiden uten Ctrl+S), Copilot/chat helt av (funn 13).
SETTINGS_DIR=/home/coder/.local/share/code-server/User
mkdir -p "$SETTINGS_DIR"
if [ ! -f "$SETTINGS_DIR/settings.json" ]; then
  cat > "$SETTINGS_DIR/settings.json" <<'EOF'
{
  "security.workspace.trust.enabled": false,
  "workbench.startupEditor": "none",
  "files.autoSave": "afterDelay",
  "telemetry.telemetryLevel": "off",
  "update.mode": "none",
  "extensions.ignoreRecommendations": true,
  "chat.commandCenter.enabled": false,
  "chat.agent.enabled": false,
  "chat.disableAIFeatures": true,
  "github.gitAuthentication": false,
  "zoo-code.autoImportSettingsPath": "/home/coder/zoo-settings.json"
}
EOF
fi

# Den levende websiden (Vite leser WEB_BASE i vite.config)
if [ -f "$PROJECT_DIR/package.json" ]; then
  (cd "$PROJECT_DIR" && npm install && exec npm run dev -- --host 0.0.0.0 --port 5173) &
fi

# Autosave-commit + push hvert 2. minutt — arbeidet er alltid trygt i
# prosjektrepoet. Push kan feile ved samtidig arbeid (ikke fast-forward);
# det tolereres stille, deltagerne kan git selv.
(
  while true; do
    sleep 120
    if [ -d "$PROJECT_DIR/.git" ]; then
      cd "$PROJECT_DIR"
      git add -A
      git diff --cached --quiet || git commit -m "autosave" >/dev/null
      git push >/dev/null 2>&1 || true
    fi
  done
) &

# --auth none er greit på rent tailnett SÅ LENGE arbeidsflatene aldri deler
# docker-nett med styrende tjenester (normen fra testfunn 22).
exec code-server --bind-addr 0.0.0.0:8080 --auth none "$PROJECT_DIR"
