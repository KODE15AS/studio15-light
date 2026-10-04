# Nattarbeid 04.10 → 05.10 — endringer fra Jørns testrapport

Grunnlag: «2026-10-04-Test-Rapport-endringer-som-nskes.docx» (Dropbox) +
Jørns svar på oppfølgingsspørsmålene (kveld 04.10). Alt under er bygget,
rullet ut og verifisert i natt (nettleser + presence- og konfetti-testen
GRØNNE). Klart for mennesketest 05.10 morgen.

## Navneskifter (synlig tekst — interne navn i kode/API beholdt, scope
avklart med Jørn: ren tekst-/URL-jobb, ikke omdøping av rust-symboler,
containere eller API-ruter)

| Gammelt | Nytt | Merknad |
|---|---|---|
| Lobby / lobbyen | **Startside** | Begrepet «lobby» er UTGÅTT utad |
| Program | **Prosjektgruppe** | «Velg prosjektgruppe», «Lag ny prosjektgruppe» |
| Vegg | **Tavle** | Raven 70". Rute `/tavle`; `/vegg` lever som alias |
| Arbeidsflate/flate | **Skjerm / deltagerskjerm** | «Åpne min skjerm», «Min skjerm» |

- Ny hovedadresse: **https://startside.studio15.cloud** (A-post i
  deploi-dns, nytt Let's Encrypt-sertifikat som dekker begge navn).
  `lobby.studio15.cloud` → evig 308-redirect. PUBLIC_BASE oppdatert.
- Kiosken på raven bruker nå `/tavle` (alias `/vegg` virker fortsatt).

## Ny informasjonsarkitektur

- `/` = startsiden: deltagervelger øverst til høyre, «Lag ny
  prosjektgruppe»-knapp på overskriftslinjen (UT av rutenettet),
  gruppefliser med inntil 4 prosjektnavn (skroller ved flere).
- `/gruppe/<slug>/` = prosjektsiden (NY URL — gruppene er bokmerkbare):
  «Prosjektgruppe: <navn>», «Lag nytt prosjekt»-knapp på
  overskriftslinjen etter samme mønster.
- **Brødsmuler i GitHub-stil** på begge sider («Startside / <gruppe>»).
  Tatt inn i web-profilen som default på sider med hierarki:
  `webprofil-kode15/kode15-profil.md` + `.k15-smuler` i kode15.css
  (committet og pushet i raven-platform).

## Deltagerregisteret (bevisst unntak fra grunnlagsdokumentene)

- `register/deltagere.yaml` — åpen tabell i repoet: slug, navn, fast
  farge, registreringsdato. INGEN credentials/passord: dette er
  identitetsVALG, ikke innlogging (grunnprinsippet «ingen innlogging»
  står — unntaket er avklart med Jørn 04.10).
- API: `GET/POST /api/deltagere` (POST tildeler første ledige farge fra
  presence-paletten; duplikat-slug avvises). Tabellen følger også med i
  `/api/tilstand`.
- UI: velger øverst til høyre på startsiden (velg eksisterende eller
  registrer ny); valget huskes i nettleseren (localStorage `s15l.deltager`)
  og brukes ved «Åpne min skjerm».
- Fargen er FAST overalt: presence-join slår opp registerfargen
  server-side (klientens ønske overstyres), og `/api/tilstand` dekorerer
  flatene med eierfarge så samling/tavle viser riktig farge også når
  eieren er frakoblet.
- Tabellen står TOM ved overlevering (testdeltageren fra verifiseringen
  er fjernet) — Jørn registrerer ekte deltagere i morgentesten.

## Sync-granskningen (Jørns siste kommentar 04.10)

Funn: skjermen hos deltageren og tilsvarende flis på tavla ER samme
kilde (samme container, samme filer, samme Vite-server). Det som IKKE
deles er code-servers arbeidsflate-TILSTAND: åpne faner/layout bor per
nettlesertilkobling (VS Code web lagrer workbench-state i nettleseren),
så tavlas editor-flis var en fersk tilkobling som viste velkomstskjermen
mens deltageren jobbet. Websiden deler kilde, men klient-state (f.eks.
klikkteller) er naturlig per nettleser.

Fiks (bygget i natt): tavlas og samlingens FREMMEDE editor-fliser åpner
nå prosjektets hovedfil eksplisitt
(`?folder=/home/coder/project&payload=[["openFile",…/src/App.svelte]]`) —
flisen viser koden og oppdateres ved hver lagring (filvokteren). Egen
flis røres ikke (egen økt med egne faner). Verifisert i nettleser:
tavla viser begge deltagernes App.svelte live. Merk: payload-stien
forutsetter prosjektmalens `src/App.svelte`; mangler fila viser
code-server bare en stille feilmelding.

## Teknisk smått

- presence-protokollen: `join` tar nå valgfri `farge` (register-fargen
  vinner server-side). Testene uendret GRØNNE (16/16).
- Caddy: `@spahtml` utvidet med `/gruppe/*` og `/tavle`; :8102 bruker
  startside-sertifikatet og redirecter host `lobby.studio15.cloud`;
  :8103 redirecter til startside.
- `skript/lobby-sertifikat.sh` utsteder nå ett sertifikat for begge
  navn (LEGO_DOMAINS) — cron-fornyelsen uendret.
- deploi-dns: `startside` A-post lagt til i `zones/studio15.cloud.yaml`.
