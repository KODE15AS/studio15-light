# HANDOVER — Studio 15 LIGHT

Levende spesifikasjon (norm «handover»). Punktene krysses av etter hvert som
de leveres. Chat 0 startet 2026-10-03.

## Grunnlag — les disse først

- «Erfaringsoverforing Studio15 LIGHT.docx» i `~/dev/studio15/` — hele
  rammen, fellene fra Studio 15 og rådene inn i Zoo Code.
- «Erfaringsoverføring: Skjermsamling → Studio 15 Light» (levert 03.10,
  erstatter dokumentet på den umergede grenen i `raven-skjermsamling`) —
  samarbeidslaget: 70"-veggen 50/50, se/peke/ta over, presence-protokollen,
  kiosk-oppsettet (`wall/`), controller-mønsteret (`workspace.rs`, `hub.rs`).

## Målbilde

To personer på hver sin PC over Tailscale (wifi), hver med sin arbeidsflate:
VS Code i nettleseren (code-server) + Zoo Code koblet mot LiteLLM-proxy +
Vite-dev-server som viser websiden levende. Ravens 70"-skjerm viser begge
flatene 50/50. Ingen innlogging, ingen database, ingen offentlig eksponering.

Strukturen over flatene: et **program** (= GitHub-org) dekker et antall
**prosjekter** (= eget container/repo-sett, persistent). Lobbyen lar en
opprette program og prosjekter og starte/gjenoppta dem; workspaces opprettes
per deltager ved behov.

## Beslutninger (Jørn, 2026-10-03)

Fra VeloStack-runden (notatet «README about Studio 15 Light med VeloStack»):

- Begrepet «VeloStack» droppes — stacken fra erfaringsoverføringen er fasit.
- Lokalt og enkelt: Tailscale uten ingress, ingen offentlig eksponering.
- NVMe med navngitte volumer er godt nok — ingen tmpfs/RAM-disk.
- Latency-arbeid kun der det merkes (UI-ekko, Vite HMR); mål først,
  optimaliser etterpå. Ingen kunstig kompleksitet (HTTP/3, Unix-socket-MCP).

Fra kap. 10-avklaringene (erfaringsoverføringen):

- **HTTPS via Tailscale-sertifikat** (`tailscale cert`) — secure context for
  clipboard/streaming uten offentlig CA eller ingress.
- **Full tilgang for begge utviklere**: redigering + terminal, men
  terminal-autokjøring AV som standard. Det skal være enkelt og intuitivt
  for brukeren selv å endre dette i Zoo Code-innstillingene.
- **Dvale/vekke-mønsteret** gjenbrukes (idle-reaper: stopp etter N min,
  503-vekkeside med auto-reload, oppe igjen på ~10 s).
- **Felles master-nøkkel** i LLM-proxyen — kostnadsinnsyn per person
  legges eventuelt til senere med to virtuelle nøkler.

Fra skjermsamling-avklaringene (03.10):

- **Veggen viser begge per halvdel**: arbeidsflate (editor + Zoo Code-chat)
  og den levende websiden side om side. Veggen er alltid read-only
  (watch-modus), Ravens tastatur/mus brukes aldri.
- **Prosjektvelger i lobbyen**: ett prosjekt = eget container/repo-sett,
  persistent, startes/gjenopptas med klikk. Lobbyen er også vekkeside for
  dvale/vekke-mønsteret.
- **Sletting er sletting**: ett skript sletter containere, volumer OG
  GitHub-repoet (etter eksplisitt bekreftelse) — ingen dangling repos.
  GitHub-tilgangen løses av vaktmester-appen (se Hovedstruktur under);
  org-sletting er GitHubs ene manuelle unntak.
- Se/peke/ta over videreføres uendret fra Skjermsamling (eierfarge, ghost-
  cursors med navn, én ekstern controller, take/release broadcastes).
- Designgrunnlag: `webprofil-kode15` (norm «web-profil»), med Skjermsamlings
  `app.css` som referanse. Arbeid rett på `main` (norm «tbd»), ikke PR-flyt.

Hovedstruktur (Handover 0-grillingen, 03.10):

- **Program = ekte GitHub-org.** Et program er overbygningen som dekker et
  antall prosjekter (erstatter Studio 15s «klasse»). Valgt med åpne øyne:
  GitHub har ikke API for org-oppretting eller org-sletting, så UI-et blir
  en guidet flyt med manuelle GitHub-steg (som Studio 15 løste det), og
  sletteregimet får et manuelt org-steg til slutt — resten automatiseres.
- **Vaktmester-mønsteret gjenopplives**: GitHub App for repo-automatikk i
  program-orgene (lærdommene fra Studio 15 om deploy keys og app-tokens
  gjelder). Erstatter merknaden om ny PAT.
- **Alt bor i git — ingen database.** Programregister som YAML i dette
  repoet, prosjektkode i prosjektrepoene, secrets i `.env` utenfor git,
  kjøretilstand leses fra Docker. Kun temp-filer på raven, slik at en
  container alltid kan gjenoppbygges fra repo.
- **Workspaces opprettes per deltager ved behov** — solo-prosjekter støttes;
  ikke fast to seter.
- **V1 = tynn E2E-skive**: lobby (program + prosjekt) → workspace med
  Zoo Code → proxy → konfetti-test. Vegg + se/peke/ta over er v2.
- Web-design: `webprofil-kode15` (norm «web-profil») — bekreftet på nytt.

## Åpne punkter (agentarbeid, første byggetrinn)

- [ ] Zoo Code-verifisering: utvidelses-ID, distribusjon (Open VSX?),
      `.roomodes`-format, announcement-hack, ripgrep-fellen. Bekreft at
      arbeidsflaten er ren webapp (code-server) → ingen GUI-streaming.
- [ ] Se/peke-laget over iframes: flatene må trolig serveres same-origin
      via en intern reverse proxy i stacken (ikke offentlig ingress) for at
      ghost-cursors skal kunne fanges i kollegaens flate. Undersøk også om
      «ta over» kan realiseres med code-servers flerbruker-tilkobling i
      stedet for input-streaming — før streaming-veien eventuelt velges.

## Leveranser

Gjort:

- [x] Repo opprettet på GitHub (KODE15AS/studio15-light) og klonet til
      `~/dev/studio15-light` på raven.
- [x] Stack på plass: compose + plassholder-container på
      http://100.65.19.39:8100 (kun Tailscale).
- [x] Alle avklaringer tatt med Jørn (se Beslutninger over).

V1 — tynn E2E-skive (trinnvis, hvert trinn E2E-verifiseres maskinelt):

- [ ] Zoo Code-verifisering (se Åpne punkter) — først, alt annet avhenger
      av den.
- [ ] Workspace-image: code-server (pinnet versjon) + Node LTS + Vite +
      Zoo Code, med alle image-fiksene fra Studio 15 (chown på volum,
      Copilot fjernet, trust/velkomst av, ripgrep-symlink, autolagring).
- [ ] LiteLLM-proxy i enkleste form: master-nøkkel, Fable 5 primær med
      fallback til Opus 5, `drop_params: true`. Nøkkel kun i proxyen.
- [ ] Vaktmester-appen: GitHub App for repo-automatikk i program-orgene.
      Krever manuelle GitHub-steg fra Jørn (app-oppretting/installasjon).
- [ ] Lobby (webprofil-kode15): programregister (YAML i dette repoet),
      guidet org-opprettingsflyt med manuelle GitHub-steg, prosjektvelger
      som starter/gjenopptar prosjekt, workspace per deltager ved behov.
      Mock-driver (`WORKSPACE_DRIVER=mock`) for utvikling uten Docker.
- [ ] Docker-nett: workspace-containere deler aldri nett med styrende
      tjenester (`internal: true`-mønsteret).
- [ ] HTTPS via `tailscale cert` for alle URL-er.
- [ ] Dvale/vekke: idle-reaper + 503-vekkeside (lobbyen er vekkesiden).
- [ ] Sletteskript: workspaces + volumer + GitHub-repo i én operasjon etter
      eksplisitt bekreftelse; org-sletting som dokumentert manuelt steg.
- [ ] Konfetti-testen: modus → proxy → modell → filredigering → synlig på
      levende webside.
- [ ] Begge deltagere verifisert fra wifi med Tailscale.

V2 — samarbeidslaget:

- [ ] Presence-laget: roster-snapshot etter welcome, reconnect med
      session-token + generasjonsteller, tile-relative cursor-koordinater,
      se/peke/ta over.
- [ ] 70"-veggen: kiosk helse-gatet mot /healthz, watch-modus (?watch=1),
      begge halvdeler med arbeidsflate + levende webside
      (kiosk-fellene fra `wall/`-skriptene: wmctrl på PID, DISPLAY-vakt,
      autostart-mappe, fast skjerm).
