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

## Begreper (avklart 03.10 — brukes konsekvent)

- **Program** = GitHub-org. Overbygningen som dekker et antall prosjekter
  (erstatter Studio 15s «klasse»).
- **Prosjekt** = container/repo-settet: GitHub-repoet pluss containerne som
  kjører det som bygges (f.eks. Vite-serveren). Persistent, gjenopptakbart,
  slettes som helhet.
- **Arbeidsflate** (kode: workspace) = en editor-container (code-server +
  Zoo Code) koblet til et prosjekt. Opprettes ved behov.
- **To personer kan dele ett prosjekt** på to måter: hver med sin egen
  arbeidsflate mot samme repo (normalen), eller med én **felles
  arbeidsflate** — code-server tillater flere samtidige tilkoblinger til
  samme instans, så begge åpner samme URL og jobber i samme flate. Denne
  egenskapen er også grunnlaget for «ta over» i V2.

I synlig tekst (UI, docs) brukes «arbeidsflate» (norm «språk»);
«workspace» kun i kode og tekniske identifikatorer.

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
- **Arbeidsflater opprettes per deltager ved behov** — solo-prosjekter støttes;
  ikke fast to seter.
- **V1 = tynn E2E-skive**: lobby (program + prosjekt) → arbeidsflate med
  Zoo Code → proxy → konfetti-test. Vegg + se/peke/ta over er v2.
- Web-design: `webprofil-kode15` (norm «web-profil») — bekreftet på nytt.

## Åpne punkter (agentarbeid, første byggetrinn)

- [x] Zoo Code-verifisering (chat 1, 03.10 — vsix-en inspisert direkte):
      ID `ZooCodeOrganization.zoo-code`, alle nøkler/kommandoer har prefiks
      `zoo-code` (ikke `roo-cline`); distribusjon Open VSX (verifisert
      utgiver, pinnet 3.87.100557); `.roomodes`-formatet og import-skjemaet
      (providerProfiles/modeApiConfigs) uendret fra Roo; announcement-hacken
      trengs fortsatt (ID `oct-2026-v3.86.0-models-aborts-tool-streaming`);
      ripgrep-fellen er FIKSET oppstrøms (Zoo leter selv i
      ripgrep-universal) — symlinken beholdt som forsikring; ren webapp
      bekreftet → ingen GUI-streaming. NYTT funn: Zoo viser en
      telemetri-dialog ved første oppstart — kveles med
      `telemetrySetting: "disabled"` i settings-malen.
- [ ] Se/peke-laget over iframes (V2): same-origin-grunnlaget er alt på
      plass — caddy serverer lobby, editorer og websider fra samme origin.
      Gjenstår: undersøke om «ta over» kan realiseres med code-servers
      flerbruker-tilkobling i stedet for input-streaming.

## Leveranser

Gjort:

- [x] Repo opprettet på GitHub (KODE15AS/studio15-light) og klonet til
      `~/dev/studio15-light` på raven.
- [x] Stack på plass: compose + plassholder-container på
      http://100.65.19.39:8100 (kun Tailscale).
- [x] Alle avklaringer tatt med Jørn (se Beslutninger over).

V1 — tynn E2E-skive (trinnvis, hvert trinn E2E-verifisert maskinelt;
alt under levert av chat 1, 03.10):

- [x] Zoo Code-verifisering (se Åpne punkter) — først, alt annet avhenger
      av den.
- [x] Arbeidsflate-image (`arbeidsflate/`): code-server 4.140.0-debian
      (pinnet) + Node 22 + Vite + Zoo Code 3.87.100557, med alle
      image-fiksene fra Studio 15 (chown på volum, Copilot fjernet med
      byggvakt, trust/velkomst av, ripgrep-symlink, autolagring,
      announcement-hack, telemetri av). Oppstart-utvidelse åpner
      Zoo-chatten automatisk. NY felle betalt: bare-repoer seedet av
      lobbyen (root) må chownes til uid 1000 + `safe.directory` i
      entrypoint, ellers nekter git («dubious ownership»).
- [x] LiteLLM-proxy (`proxy/`, pinnet v1.103.2, UTEN database):
      master-nøkkel, `standard` (Fable 5) med fallback til `reserve`
      (Opus 5), `drop_params: true`, `num_retries: 2`. Nøkkel kun i
      proxyen; arbeidsflatene får bare master-nøkkelen mot proxyen.
- [ ] Vaktmester-appen: GitHub App for repo-automatikk i program-orgene.
      Klikkelisten til Jørn er KLAR (`docs/vaktmester-klikkeliste.md`) —
      venter på app-oppretting/installasjon. Til da bruker prosjektene
      bare-repoer på volumet `s15l-repos` (`file:///repos/<slug>.git`),
      som begge arbeidsflater kloner fra og pusher til.
- [x] Lobby (webprofil-kode15, Rust/axum + Svelte 5): programregister
      (YAML i dette repoet), guidet org-opprettingsflyt med manuelle
      GitHub-steg, prosjektoppretting fra mal (Svelte 5 + Vite, pinnet),
      prosjektvelger, arbeidsflate per deltager ved behov, sletting med
      bekreftelse. Mock-driver (`WORKSPACE_DRIVER=mock`) for utvikling
      uten Docker. Controller-mønsteret: lobbyen er eneste komponent som
      styrer containere, kun faste operasjoner.
- [x] Docker-nett: lobbyen på internt nett (`s15l-front`, internal: true)
      som arbeidsflatene ikke kan rute til; caddy avviser trafikk fra
      arbeidsflate-subnettet (client_ip-vakt, DNAT-lærdommen). Begge
      vaktene verifiseres i konfetti-testen.
- [x] HTTPS via `tailscale cert` for alle URL-er — én inngang
      (caddy :8100), alt samme origin (V2-forberedelse for se/peke).
      `skript/hent-sertifikat.sh` henter/fornyer.
- [x] Dvale/vekke: idle-reaper i lobbyen (45 min, fra caddys tilgangslogg)
      + 503-vekkeside med autovekking og reload. Verifisert maskinelt med
      1-minutts grense.
- [x] Sletteskript (`skript/slett-prosjekt.sh` + lobby-UI): containere +
      volumer + prosjektrepo + registeroppføring i én operasjon etter
      eksplisitt bekreftelse; GitHub-repo/org som dokumentert manuelt steg
      til vaktmesteren finnes.
- [x] Konfetti-testen (`skript/konfetti-test.sh`), maskinell del GRØNN
      03.10: lobby → repo-seed → arbeidsflate → code-server og Vite
      gjennom HTTPS-proxyen → proxy → Fable 5-svar (temperature droppet)
      → filredigering synlig live (HMR verifisert i nettleser) →
      nettvakter → sletting. Zoo-chat-leddet kan ikke fjernstyres maskinelt
      (webview-iframe) — det er første punkt i mennesketesten.
- [ ] Begge deltagere verifisert fra wifi med Tailscale. Demo-prosjektet
      («Demo» → «Konfetti») står klart i lobbyen; testresept levert i
      chat 1. Sete 1 (Jørns PC) har alt vist lobby/editor/webside over
      Tailscale med gyldig HTTPS under nettlesertesten.

V2 — samarbeidslaget:

- [ ] Presence-laget: roster-snapshot etter welcome, reconnect med
      session-token + generasjonsteller, tile-relative cursor-koordinater,
      se/peke/ta over.
- [ ] 70"-veggen: kiosk helse-gatet mot /healthz, watch-modus (?watch=1),
      begge halvdeler med arbeidsflate + levende webside
      (kiosk-fellene fra `wall/`-skriptene: wmctrl på PID, DISPLAY-vakt,
      autostart-mappe, fast skjerm).
