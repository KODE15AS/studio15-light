# Regler for agenten i dette prosjektet (ekspert-malen)

Dette er et PROFESJONELT prosjekt i Studio 15 Light. Hva som skal bygges
styres av deltageren og prosjektdokumentene — ikke av denne malen. Du får
utfolde deg: ingen trinnregi, ingen pedagogisk ramme. Dette dokumentet
gir deg FAKTA om miljøet ditt og konvensjonene som gjelder, slik at du
slipper å gjette og aldri går på de samme fellene to ganger.

## Skjermen og samtalen

- Deltageren ser BARE samtalen (venstre) og den levende websiden (høyre)
  — aldri editoren eller terminalen. Alt du gjør i filer og terminal er
  usynlig for deltageren til du forteller om det: oppsummer viktige
  handlinger og funn i chatten.
- Kommandoer auto-godkjennes. Med den friheten følger ansvar: vær nøktern
  med hva du kjører, og forklar inngripende operasjoner før du gjør dem.
- Bruk todo-listen (verktøyet) ved flerstegs arbeid så fremdriften er
  synlig, og still oppfølgingsspørsmål (verktøyet, gjerne med svarforslag)
  når briefen er uklar eller et veivalg er deltagerens. Profesjonell tone,
  norsk bokmål.

## Miljøfakta — arbeidsflaten (utviklingscockpiten)

- Debian-container (code-server), bruker `coder` (uid 1000). INGEN
  root/sudo/apt — systempakker kan ikke legges til her.
- Installert: Node 22 (npm/npx), git, curl, wget, tar, unzip, jq,
  sqlite3, python3 (pip kun inne i venv: `python3 -m venv`), gcc/make
  (build-essential) og pkg-config — så rustup, native npm-moduler og
  pip-pakker som kompilerer fungerer.
- Mer verktøy installerer du selv i hjemmemappen (rustup, venv,
  npm-pakker). Det overlever restart og dvale, men IKKE gjenskaping av
  containeren — legg derfor oppsettsstegene i et skript i repoet (f.eks.
  `oppsett.sh`) slik at miljøet alltid kan gjenskapes fra repo.
- Prosjektet bor i `/home/coder/project` og ER git-repoet. En
  autosave-jobb committer og pusher hvert 2. minutt — arbeid er alltid
  trygt, men husk at også halvferdige filer blir committet.
- Nettverk UT: fritt (npm, crates.io, pypi, dokumentasjon). Nettverk INN:
  kun editoren (8080) og websiden (5173) rutes til brukerne via proxyen.
  Alt annet du starter er kun tilgjengelig INNE i containeren — utmerket
  til utvikling og tester (tjener + klient/simulator i samme container),
  men deltageren og eksterne enheter når det aldri.
- Websiden: Vite-dev-serveren på :5173 kjører alltid (containeren starter
  den) og viser `src/App.svelte` levende. Backend-logikk som websiden
  skal nå i utviklingsfasen må gå gjennom Vite: plugin/middleware eller
  WebSocket på samme port.
- Henvis ALDRI til localhost eller 127.0.0.1 — det virker bare inne i
  containeren. Adressene som gjelder står i `.roo/rules/01-webside.md`.
- Containeren er bevisst avmektig: du når ikke docker, lobbyen eller
  raven-verten. Ikke let etter veier rundt dette.

## Kjøring som tjeneste — normen «ett repo, én container»

- Når prosjektet skal KJØRE som tjeneste (ikke bare utvikles), skjer det
  ikke i arbeidsflaten: hvert repo blir sin egen Docker-stack på raven
  under `~/dev/<repo>`, med compose-fil i repoet. Din oppgave er å holde
  `docker-compose.yml` og en deploy-seksjon i README klar og oppdatert —
  Jørn deployer på raven.
- Veien inn for eksterne enheter (porter, ingress, TLS, DNS) besluttes av
  Jørn per prosjekt. Foreslå konkret i README/handover — aldri anta.
- raven-fakta: Ubuntu 24.04-server, Docker, Tailscale (tailnett), Caddy
  som ingress, DNS styres via deploi-dns (GitOps). Ingen offentlig
  eksponering uten eksplisitt beslutning med egen risikovurdering.

## Lagring

- Foretrukket: SQLite som fil i prosjektet (CLI-en `sqlite3` finnes;
  bruk en SQLite-driver i språket ditt for applikasjonen).
- Større databasebehov: egen MariaDB-container på raven (repoet
  `KODE15AS/MariaDB`) — avtal med Jørn og dokumenter tilkoblingen i
  README.
- Plattformen har bevisst ingen database og ingen innlogging — gjeninnfør
  aldri dette «for sikkerhets skyld» på plattformens vegne.

## Hemmeligheter

- ALDRI i git — heller ikke midlertidig, heller ikke i logger eller
  eksempelfiler med ekte verdier. Alle secrets bor i Bitwarden (KODE15);
  i kjøretid leses de fra `.env` (chmod 600) utenfor repoet.
- Denne containeren har ingen API-nøkler og skal ikke få noen i repoet.
  Trenger prosjektet en nøkkel: be deltageren/Jørn og dokumenter i README
  hvor den bor og hvordan den settes.

## Dokumentinnboksen

- Deltageren laster opp dokumenter fra skjermen — de lander i `innboks/`
  i prosjektet. Sjekk mappen når deltageren nevner en opplasting.
- Din jobb: les dokumentene (docx er en zip med `word/document.xml`;
  for PDF kan du installere verktøy selv), strukturer innholdet inn i
  repoet (docs/, README, handover), flytt eller slett originalen etter
  avtale, og oppsummer i chatten hva du fant og gjorde.
- Deltageren kan også lime inn LENKER i chatten (Dropbox, nettsider,
  GitHub) — hent dem selv med curl. Dropbox: bytt `dl=0` med `dl=1` i
  lenken for å få selve fila. Legg nedlastede dokumenter i `innboks/`
  og behandle dem som over.

## Konvensjoner (KODE15)

- `handover/HANDOVER.md` er levende spesifikasjon: oppdater den i samme
  endring som funksjonelle leveranser; daterte notater i `handover/` ved
  faseskifter.
- README.md har en «Lenker»-tabell øverst — hold den oppdatert når
  adresser og endepunkter endres.
- Norsk bokmål i alt: UI-tekst, dokumentasjon, kodekommentarer og
  commit-meldinger (commits uten æøå — bruk ae/oe/aa).
- Web-UI følger KODE15-webprofilen med mindre prosjektet krever noe
  annet.
- Pin versjoner (npm-pakker, images); oppgrader bevisst og dokumentert.
- Grunnoppsettet for websiden er Svelte 5 + TypeScript + Vite — bruk
  Svelte 5-runer (`$state`, `$derived`, `$effect`, `$props`), aldri
  Svelte 4-syntaks. Prosjektbriefen styrer alle andre stackvalg
  (backend-språk, biblioteker, arkitektur).
