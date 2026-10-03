# Handover 1 → 2 (2026-10-03)

Chat 1 («Handover 1») bygget og testet hele V1. `HANDOVER.md` er målbildet —
denne fila er tilstanden ved overgangen og byggeoppdraget til chat 2.

## Tilstand: V1 er levert og testet

Hele stacken kjører på raven (`docker compose up -d` i dette repoet pluss
arbeidsflate-imaget `studio15-light-arbeidsflate:v1`):

- **Caddy** (:8100, HTTPS via tailscale-sertifikat i `certs/`, fornyes med
  `skript/hent-sertifikat.sh`): én inngang, samme origin for lobby
  (`/`), editorer (`/w/<flate>/`) og websider (`/web/<flate>/`).
- **Lobby** (Rust/axum + Svelte): programregister i
  `register/programmer.yaml`, prosjekter fra mal, arbeidsflater per
  deltager, dvale/vekke (idle-reaper 45 min + vekkeside), sletteregime.
  Mock-driver: `WORKSPACE_DRIVER=mock`.
- **LiteLLM** v1.103.2: `standard` = Fable 5, fallback `reserve` = Opus 5,
  `drop_params`, master-nøkkel i `.env`.
- **Arbeidsflate-imaget**: code-server 4.140.0 + Node 22 + Zoo Code
  3.87.100557 (alle fellefiksene — se Dockerfile-kommentarene).

Verifisert 03.10: `skript/konfetti-test.sh` GRØNN (maskinell akseptansetest,
idempotent — kjør den etter hver endring). Jørn har i mennesketest bestått
konfetti-testen fra sete 1 (Zoo-chat → levende webside), org-flyten,
sletteflyten og prosjektoppretting. **Vaktmester-appen er opprettet,
installert og E2E-verifisert** (JWT → installasjonstoken → repo opprettet og
slettet i `KODE15-saturday-test-2`). App ID 5176150, nøkkel i
`certs/vaktmester.pem`, `GITHUB_APP_ID`/`GITHUB_APP_KEY_FILE` i `.env`.

I registeret står programmet «Saturday test 2» (org
`KODE15-saturday-test-2`) med to testprosjekter på lokale bare-repoer, og
et tomt «Demo»-program.

## Chat 2 starter her — oppdrag: bygg og test alt gjenstående autonomt

Døp chatten «Handover 2». Oppdraget fra Jørn (03.10): **bygg og maskintest
alt gjenstående i én autonom prosess**; når alt er grønt kjøres omfattende
mennesketester. Rekkefølge:

### 1. Vaktmester-integrasjonen i lobbyen (fullfører V1)

- Nytt prosjekt i et program med `github_org` satt → opprett repo i org-en
  via installasjonstoken (seed fra prosjektmalen som i dag) i stedet for
  bare-repo. Programmer uten org fortsetter på bare-repoer.
- Arbeidsflatene må få push-tilgang: Studio 15-mønsteret er en git
  credential helper som henter FERSKT repo-scopet installasjonstoken ved
  hver push (tokens lever 1 time — aldri bak i miljøvariabler).
  **Designpunkt som må løses:** arbeidsflatene når ikke lobbyen (bevisst,
  testfunn 22). Foreslått løsning: ett eksplisitt unntak i caddy-vakten for
  én token-sti (f.eks. `POST /api/git-token`), autentisert med
  per-arbeidsflate-hemmelighet satt som env ved opprettelse; endepunktet
  utsteder kun token scopet til flatens eget repo. Hold unntaket minimalt
  og dokumentér det i Caddyfile.
- Sletteregimet sletter GitHub-repoet i samme operasjon (Administration RW
  er verifisert). Org-sletting forblir manuell.
- Eksisterende bare-repo-prosjekter: la dem leve videre som lokale
  prosjekter; dokumentér i README at repo-typen velges ved opprettelse.
- Utvid `skript/konfetti-test.sh` med en GitHub-variant (eget testrepo i
  `KODE15-saturday-test-2`, slettes sporløst).

### 2. V2-avklaringen (før presence bygges)

Undersøk om «ta over» kan realiseres med code-servers flerbruker-tilkobling
(begge åpner samme `/w/<flate>/`-URL — allerede støttet) i stedet for
input-streaming. Hvis ja: presence-laget trenger bare se/peke (cursors),
ikke kontrollprotokollens input-videresending. Samme metode som
Zoo-verifiseringen: undersøk først, bygg etterpå.

### 3. Presence-laget (V2)

Se/peke/ta over per skjermsamling-lærdommene (C i erfaringsoverføringen):
roster-snapshot etter welcome, reconnect med session-token +
generasjonsteller, tile-relative koordinater, eierfarge + controllerfarge,
én ekstern controller, take/release broadcastes. Same-origin-grunnlaget er
på plass i caddy. Bygg mot mock-driveren og lag protokoll-smoketest
(à la 16-punkts ws-testen).

### 4. 70"-veggen (V2)

Kiosk på raven, helse-gatet mot `/healthz`, watch-modus (`?watch=1` —
joiner aldri, sender aldri input), begge halvdeler med arbeidsflate +
levende webside. Alle kiosk-fellene står i skjermsamling-erfaringsover-
føringen (D): wmctrl på PID, DISPLAY-vakt, autostart-MAPPE, fast skjerm,
frisk --user-data-dir. Fysisk verifisering på 70"-en krever et menneske —
forbered, og ta det som del av slutt-testen.

### 5. Omfattende mennesketest (med Jørn, til slutt)

Full resept på nytt: konfetti fra begge seter (sete 2 gjenstår fra V1!),
samtidig arbeid i samme prosjekt, felles arbeidsflate (samme URL), se/peke/
ta over, veggen, dvale/vekke i praksis, terminal-autokjøring av/på,
sletteregime med GitHub-repo.

## Avklart: automatisk org-oppretting via API (Jørns spørsmål 03.10)

Undersøkt 03.10: GitHubs API kan KUN opprette org-er under en
**enterprise-konto** (GitHub Enterprise Cloud): GraphQL-mutasjonene
`createEnterpriseOrganization` / `removeEnterpriseOrganization` (sistnevnte
ville også automatisert org-sletting). Siden juli 2025 kan GitHub Apps få
enterprise-rettigheter til dette, pluss API for å installere apper i
org-ene — hele program-livssyklusen kunne altså automatiseres.

**Pris:** GitHub Enterprise Cloud koster $21 per bruker/måned (samme bruker
i flere org-er telles én gang). For to utviklere ≈ $42/mnd ≈ 5 000 kr/år.
Ingen egen API-lisens utover abonnementet. **Anbefaling:** ikke verdt det
nå — manuell org-oppretting tar to minutter og skjer sjelden; den guidede
flyten i lobbyen dekker behovet. Revurder hvis programmer opprettes ofte.
Pengebruk er uansett Jørns beslutning (norm «bestilling»).

## Feller betalt i chat 1 (ikke betal igjen)

- Bare-repoer seedet av lobbyen (root) må `chown 1000:1000` og
  arbeidsflaten trenger `git config --global safe.directory '*'` —
  ellers «dubious ownership» og klonen feiler stille i restart-loop.
- Zoo Code viser telemetri-dialog ved første oppstart — kvalt med
  `telemetrySetting: "disabled"` i settings-malen.
- Zoo-settings auto-importeres KUN ved fersk container (globalStorage-
  sjekk i entrypoint) så brukerens egne valg (f.eks. terminal-autokjøring)
  overlever restart/reload. Gjenskapt container = standardoppsett.
- Faste docker-subnett kolliderer lett: `docker network inspect` før valg
  (s15l-workspaces = 172.31.15.0/24; caddy-vakten refererer samme subnett).
- Volum-sletting rett etter force-remove av container gir 409 et øyeblikk —
  driveren prøver igjen (maks 5 × 500 ms).
- Vekkesiden må skille «første oppstart» (kjørende, ikke klar — npm tar
  ~1 min) fra «dvale» (stoppet) — Jørn-funn, fikset.
- @KODE15AS er en PERSONLIG konto, ikke org — GitHub Apps lages under
  kontoens Developer settings. Klikkelisten er rettet.
- Zoo-chatten (webview-iframe) kan ikke fjernstyres av nettleserverktøy —
  Zoo-leddet må mennesketestes; alt rundt (proxy, modell, filer, HMR)
  maskintestes i konfetti-skriptet.

## Praktisk

- Konfetti-test: `skript/konfetti-test.sh` (idempotent, rydder selv).
- Sertifikat: `skript/hent-sertifikat.sh`; sletting:
  `skript/slett-prosjekt.sh`.
- Arbeidsflate-imaget bygges med
  `docker build -t studio15-light-arbeidsflate:v1 arbeidsflate/` —
  det er IKKE en compose-tjeneste.
- Dropbox-lenkene Jørn brukte til pem-nøkkel og skjermbilder bør slettes
  (nøkkelen kan alternativt roteres — ett klikk på appsiden).
- Gamle Studio 15-testorger (`jorn-slettmeg`, `kjernepraksis-tilkobling-
  slett`, `test-ny-klasse-slettmeg`) venter på manuell sletting ved
  anledning.
