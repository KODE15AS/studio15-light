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

# GitHub-prosjekter: credential helper som henter FERSKT repo-scopet
# installasjonstoken fra lobbyen ved hver push/pull (Studio 15-mønsteret —
# tokens lever 1 time og ligger aldri fast i miljøet). Autentisering:
# per-arbeidsflate-hemmelighet (GIT_TOKEN_SECRET) satt av lobbyen ved
# opprettelse. Kallet går gjennom caddys ENE vakt-unntak (/api/git-token);
# -k fordi sertifikatet er utstedt for tailnett-navnet, ikke containernavnet
# (lukket docker-nett).
case "${PROJECT_REPO:-}" in
  https://github.com/*)
    mkdir -p /home/coder/.local/bin
    cat > /home/coder/.local/bin/git-credential-s15l <<'HELPER'
#!/bin/sh
[ "$1" = "get" ] || exit 0
svar=$(curl -fsSk -m 15 -X POST "$GIT_TOKEN_URL" \
  -H "Content-Type: application/json" \
  -d "{\"kortnavn\":\"$WS_KORTNAVN\",\"hemmelighet\":\"$GIT_TOKEN_SECRET\"}") || {
  echo "git-credential-s15l: fikk ikke token fra lobbyen" >&2; exit 1; }
token=$(printf '%s' "$svar" | sed -n 's/.*"token" *: *"\([^"]*\)".*/\1/p')
[ -n "$token" ] || { echo "git-credential-s15l: tomt token i svaret" >&2; exit 1; }
echo "username=x-access-token"
echo "password=$token"
HELPER
    chmod 700 /home/coder/.local/bin/git-credential-s15l
    git config --global credential.helper /home/coder/.local/bin/git-credential-s15l
    ;;
esac

if [ -n "${PROJECT_REPO:-}" ] && [ -z "$(ls -A "$PROJECT_DIR")" ]; then
  git clone "$PROJECT_REPO" "$PROJECT_DIR"
fi

# Zoo Code: provider-profil (proxy + nøkkel) auto-importeres ved oppstart via
# zoo-code.autoImportSettingsPath. Importen overskriver brukerens egne valg
# (f.eks. terminal-autokjøring PÅ), så den kjøres kun når containeren er
# fersk (ingen globalStorage ennå). Restart/reload beholder dermed brukerens
# tilpasninger; en gjenskapt container starter på standardoppsettet.
# UNNTAK (07.10, bug 6): i nybegynner-malen skrives importfila ved HVER
# oppstart — deltagerne justerer aldri innstillinger selv, og oppsettet
# vårt skal alltid gjelde (importen kjører ved hver aktivering av Zoo så
# lenge fila finnes).
ZOO_STORAGE=/home/coder/.local/share/code-server/User/globalStorage/zoocodeorganization.zoo-code
if [ ! -d "$ZOO_STORAGE" ] || [ "${S15L_MAL:-full}" = "nybegynner" ]; then
  sed -e "s|__PROXY_BASE__|${LLM_PROXY_BASE:-http://s15l-litellm:4000/v1}|" \
      -e "s|__PROXY_KEY__|${LLM_PROXY_KEY:-}|" \
      /opt/s15l/zoo-settings.template.json > /home/coder/zoo-settings.json
  # Nybegynner-malen (Jørn 05.10, rapport 4 pkt. 5): ingen checkpoints i
  # chatten, og Zoo lukker selv filene den åpner — deltageren skal aldri
  # se kode, bare agentens dialog.
  if [ "${S15L_MAL:-full}" = "nybegynner" ]; then
    python3 - /home/coder/zoo-settings.json <<'PY'
import json, sys
sti = sys.argv[1]
d = json.load(open(sti))
d["globalSettings"].update({
    "enableCheckpoints": False,
    "autoCloseZooOpenedFiles": True,
    "autoCloseZooOpenedNewFiles": True,
    "autoCloseZooOpenedFilesAfterUserEdited": True,
    # Jørn 05.10 kveld: +15 % skrift (standard 13), todo-listen på, og
    # svarforslag skal VENTE på deltagerne — aldri auto-svares.
    "chatFontSize": 15,
    "todoListEnabled": True,
    "alwaysAllowFollowupQuestions": False,
    # Bug 6 (Jørn 07.10): Run/Deny-dialog for npm install er bare
    # forvirrende for nybegynnere — kommandoer auto-godkjennes.
    # Trygt her: containeren er innelåst (eget nett, ingen styrende
    # tjenester, ingen docker-socket), og terminalen er skjult.
    "alwaysAllowExecute": True,
    "allowedCommands": ["*"],
})
json.dump(d, open(sti, "w"), indent=2)
PY
  fi
else
  rm -f /home/coder/zoo-settings.json
fi

# Nybegynner-malen: rydd Zoo-webviewen (Jørn 05.10, rapport 4 pkt. 3.3).
# Introblokken, bunnlinjen med modus/profil/ikoner og teknisk info kan
# ikke styres med innstillinger — de overstyres med CSS rett i utvidelsens
# webview-bygg, og placeholder-teksten patches til norsk uten @/⁠/-hintet.
# Zoo er versjonspinnet i Dockerfile, så selektorene/strengene er stabile;
# ettersees ved bevisst Zoo-oppgradering. Markørvakt gjør blokken idempotent.
if [ "${S15L_MAL:-full}" = "nybegynner" ]; then
  # Tittellinjen («Zoo Code - project - code-server») kan ikke skrus av med
  # innstillinger i web-workbenchen (window.customTitleBarVisibility er
  # desktop-only) — den gjøres usynlig med en inline <style> i workbench.html.
  # Inline fordi workbench.css kan ligge cachet i nettleserne (URL-en har
  # ingen innholdshash); HTML-en genereres per request og er aldri cachet.
  # Layouten beholder stripen, men den er blank og lys, så flaten ser ren ut.
  WBHTML=/usr/lib/code-server/lib/vscode/out/vs/code/browser/workbench/workbench.html
  if [ -w "$WBHTML" ] && ! grep -q "S15L-NYBEGYNNER" "$WBHTML"; then
    # python, ikke sed -i: katalogen er root-eid, kun selve fila er skrivbar.
    python3 - "$WBHTML" <<'PY'
import sys
sti = sys.argv[1]
t = open(sti, encoding="utf-8").read()
stil = ("<style>/* S15L-NYBEGYNNER (Jørn 05.10, rapport 4 pkt. 3.3-B1/B2) */ "
        ".monaco-workbench .part.titlebar { opacity: 0 !important; "
        "pointer-events: none !important; } "
        "/* 05.10 kveld: VS Code-varsler (port 5173, settings-import) er "
        "bare støy for nybegynnere — og skjemmer tavla. */ "
        ".monaco-workbench .notifications-toasts { display: none !important; }"
        "</style>")
open(sti, "w", encoding="utf-8").write(t.replace("</head>", stil + "</head>"))
PY
  fi
  for BUILD in /home/coder/.local/share/code-server/extensions/zoocodeorganization.zoo-code-*/webview-ui/build; do
    [ -d "$BUILD" ] || continue
    if ! grep -q "S15L-NYBEGYNNER" "$BUILD/assets/index.css" 2>/dev/null; then
      cat >> "$BUILD/assets/index.css" <<'CSS'
/* S15L-NYBEGYNNER (Jørn 05.10, rapport 4): all «støy» vekk for nybegynnere. */
/* 3.3-B3: introblokken (zebra-hero, om-tekst, tips, versjon) skjules */
div.flex.flex-col.h-full.p-6.min-h-0.overflow-y-auto.gap-4.relative { display: none !important; }
/* 3.3-B8: nederste kontrollinje (modus, profil, auto-approve, ikoner) */
div:has(> [data-testid="mode-selector-root"]),
div:has(> div > [data-testid="mode-selector-root"]),
div:has(> div > [data-testid="mode-selector-trigger"]) { display: none !important; }
/* 5: teknisk info (tokens, kontekstvindu, kost) i oppgavehodet */
[data-testid="context-tokens-count"],
[data-testid="context-window-size"],
[data-testid="context-window-label"],
[data-testid="cost-footer-compact"],
[data-testid="checkpoint-menu-container"] { display: none !important; }
/* 3.3-B4: tydelig, alltid synlig ramme rundt promptfeltet */
div:has(> [data-testid="highlight-layer"]) {
  border: 1px solid var(--vscode-focusBorder) !important;
  border-radius: 4px;
}
/* 05.10 kveld: skriftstørrelsen i selve promptfeltet følger chatten
   (+15 %). Høyden eies av Zoos autosize (minRows/maxRows patches i
   index.js — CSS taper mot komponentens inline !important). */
div:has(> [data-testid="highlight-layer"]) textarea {
  font-size: 15px !important;
}
CSS
    fi
    J="$BUILD/assets/index.js"
    if [ -f "$J" ] && ! grep -q "Skriv oppgaven din her" "$J"; then
      python3 - "$J" <<'PY'
# Placeholder på norsk (norm «språk») og uten det tekniske @//-hintet
# (rapport 4 pkt. 3.3-B7). Eksakte strenger fra Zoo 3.87.100557.
import sys
sti = sys.argv[1]
t = open(sti, encoding="utf-8").read()
hint = "Xe=`\\n(${b(`chat:addContext`)}${c?`, ${b(`chat:dragFiles`)}`:`, ${b(`chat:dragFilesImages`)}`})`"
t = t.replace(hint, "Xe=``")
t = t.replace("`Type your task here...`", "`Skriv oppgaven din her \u2026`")
t = t.replace("`Type a message...`", "`Skriv en melding \u2026`")
# Jørn 05.10 kveld: romslig promptfelt — nedre del av chatkolonnen.
# Høyden styres av autosize-komponentens radgrenser (inline !important
# slår all CSS), så grensene patches her: 8 rader i ro (~¼ kolonne),
# vokser til 28 (~halv kolonne) når deltagerne skriver langt.
t = t.replace("minRows:3,maxRows:15", "minRows:8,maxRows:28")
open(sti, "w", encoding="utf-8").write(t)
PY
    fi
  done
fi

# Regelfil med webside-adressen — .roo/rules/ leses av Zoo Code (verifisert
# 03.10) og er gitignorert i prosjektmalen, så hver arbeidsflate kan ha sin
# egen adresse uten git-konflikter. localhost-fellen: testfunn 9.
mkdir -p "$PROJECT_DIR/.roo/rules"
cat > "$PROJECT_DIR/.roo/rules/01-webside.md" <<EOF
# Adresser i denne arbeidsflaten

- Denne skjermen tilhører deltageren: ${PARTICIPANT:-ukjent}
  (bruk navnet når du snakker om hvem som gjør hva)
- Prosjektets levende webside: ${WEB_URL:-ukjent — spør i lobbyen}
- Oppgi ALLTID denne adressen når brukeren spør hvor websiden er.
- ALDRI henvis til localhost eller 127.0.0.1 — det virker bare inne i
  containeren, ikke på brukerens maskin.
- Vite-utviklingsserveren kjører allerede og oppdaterer websiden automatisk;
  du trenger aldri starte eller installere noe for at siden skal vises.
EOF

# Medspiller-kommandoen (Jørn 08.10): hjelperen skal selv kunne se hvem
# som er aktive i SAMME prosjektgruppe når flerspill skal testes.
# Kortnavnet identifiserer flaten; lobbyen filtrerer til gruppen.
if [ "${S15L_MAL:-full}" = "nybegynner" ] && [ -n "${WS_KORTNAVN:-}" ]; then
  cat >> "$PROJECT_DIR/.roo/rules/01-webside.md" <<EOF

## Medspillere (trinn med flere spillere)

- Hvem i prosjektgruppen som er aktive AKKURAT NÅ ser du med kommandoen:
  \`curl -sk "https://s15l-caddy:8100/api/medspillere?flate=${WS_KORTNAVN}"\`
  Svaret: «deg» er deltageren din, «aktive» er mulige medspillere
  (navn + prosjekt). Kjør den hver gang flerspill skal testes — listen
  endrer seg når deltagere kommer og går.
EOF
fi

# Fornuftige editor-innstillinger (barnesykdommene fra Studio 15):
# ingen trust-dialog (funn 2), ingen velkomstside, autolagring (avgjørende:
# endringer når websiden uten Ctrl+S), Copilot/chat helt av (funn 13).
SETTINGS_DIR=/home/coder/.local/share/code-server/User
mkdir -p "$SETTINGS_DIR"
if [ ! -f "$SETTINGS_DIR/settings.json" ]; then
  # Nybegynner-malen (Jørn 05.10, testrapport 3): absolutt all «støy» i
  # editor-UI-et vekk — deltageren skal bare se Zoo-chatten (oppstart-
  # utvidelsen åpner den som hele editorflaten og lukker sidestolpen).
  EKSTRA=""
  if [ "${S15L_MAL:-full}" = "nybegynner" ]; then
    EKSTRA='
  "workbench.activityBar.location": "hidden",
  "workbench.statusBar.visible": false,
  "workbench.editor.showTabs": "none",
  "window.menuBarVisibility": "hidden",
  "window.customTitleBarVisibility": "never",
  "workbench.tips.enabled": false,
  "workbench.layoutControl.enabled": false,
  "window.commandCenter": false,'
  fi
  cat > "$SETTINGS_DIR/settings.json" <<EOF
{$EKSTRA
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
