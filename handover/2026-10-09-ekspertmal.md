# 2026-10-09 — Ekspert-malen (prosjektmal for profesjonelle oppgaver)

Chat 3-leveransen. Bestillingen (Jørn 09.10): en mal med FÆRREST mulig
føringer — åpen for alle profesjoner, agenten skal få utfolde seg — der
virkemiddelet er presis kunnskap om ravens muligheter og begrensninger.
Kompleksitetseksempelet (skal IKKE bygges): DEFA-kaldstart-briefen
(OCPP 2.0.1-CSMS mot fysisk DEFA Power Up; Dropbox-docx 09.10) og
`KODE15AS/kodelader` (Shelly-forløperen). Fremtid: maler mellom
nybegynner og ekspert må påregnes.

## Beslutninger (Jørn 09.10, etter grilling)

- **1:1-opprydding (norm «container») — trappetrinnet valgt:** utvikling
  skjer i S15L-arbeidsflaten (cockpit); når et prosjekt skal KJØRE som
  tjeneste blir det egen Docker-stack på raven under `~/dev/<repo>` med
  compose-fil i repoet (kodelader-mønsteret). Full ombygging (lobbyen som
  compose-orkestrator med vern) ble vurdert og UTSATT — konsekvensene er
  dokumentert i chat 3: orkestrering av agent-skrevet compose krever nytt
  vern rundt docker-socketen, formidlings-API for deploy, reaper/sletting
  som skiller verktøy- fra tjenestecontainere, og omlagt tavle-ruting.
  Vurderes på nytt etter proxy-agenten.
- **Vei inn for eksterne enheter** (f.eks. lader → wss-endepunkt): IKKE
  scope nå — avventer proxy-agenten. Malen dokumenterer begrensningen.
- **Verktøy:** dagens arbeidsflate-image beholdes; agenten installerer
  selv i hjemmemappen (rustup, venv, npm). MERK: imaget fikk et minimalt
  verktøylag (build-essential, pkg-config, unzip, jq, sqlite3) fordi
  rustup/native npm-moduler/pip-bygg ikke LENKER uten C-verktøykjede —
  «agenten installerer selv» var ellers umulig for annet enn ren JS.
- **Secrets:** alle i Bitwarden (KODE15), aldri i git; `.env` utenfor
  repo i kjøretid.
- **Lagring:** SQLite som fil i prosjektet foretrekkes; større behov →
  egen MariaDB-container (repo `KODE15AS/MariaDB`) med README-referanse.
  «Ingen database»-regelen gjelder plattformen, ikke prosjektinnholdet.
- **Skjerm:** blankt UI + kommando-autokjøring som nybegynner, og
  filopplasting for prosjektdokumenter fra start.
- **Repo-hjem:** ny org = ny prosjektgruppe «Ekspert» (opprettes av Jørn
  via den guidede org-flyten — GJENSTÅR).

## Levert (bygget, utrullet og maskintestet 09.10)

- **Malen `lobby/prosjektmal/ekspert/`**: AGENTS.md med raven-fakta
  (miljø, nettgrenser, 1:1-tjenestekjøring, lagring, secrets,
  dokumentinnboks, KODE15-konvensjoner — fakta, ikke regi),
  README-skjelett med Lenker-tabell, handover/HANDOVER.md-skjelett,
  `innboks/les-meg.md`, pluss Svelte 5 + TS + Vite-grunnoppsettet.
  Navnefletting utvidet til README.md og handover/HANDOVER.md.
- **Blank UI generalisert**: entrypoint og oppstart-utvidelsen styrer nå
  på `S15L_BLANK_UI` (nybegynner + ekspert) — samme rydding, autokjøring
  og dialogspeil på tavla; medspiller-regien er fortsatt kun nybegynner.
- **Dokumentopplasting**: knapp på ekspert-skjermen («Last opp dokument»,
  flervalg) → `POST /api/arbeidsflater/{kortnavn}/opplast?navn=…` (rå
  kropp, 50 MB-grense) → `driver.last_opp` skriver tar-arkiv rett inn i
  containeren (PUT archive, eierskap coder) → `innboks/` i prosjektet.
  Agenten er instruert i malens AGENTS.md om å strukturere innholdet inn
  i repoet. Filnavn saniteres (kun siste sti-komponent, ingen punktum-
  prefiks); kortnavn valideres som slug.
- **Arbeidsflate-imaget** (v2, rebuildet): verktøylaget over.

Verifisert: E2E med ekte ekspert-prosjekt (seed med navnefletting, blank
UI-innstillinger i containeren, opplasting inkl. sti-trikse-forsøk som
nøytraliseres, verktøyene til stede, webside 200, sletting sporløs) +
presence-testen GRØNN (16/16) + konfetti-testen med GitHub-variant GRØNN.

## Gjenstår

- Jørn: opprette org/prosjektgruppen «Ekspert» (guidet flyt på
  startsiden) og veto/godkjenne verktøylaget i imaget.
- Mennesketest: ekte økt der DEFA-briefen lastes opp som dokument og
  agenten strukturerer den inn i repoet.
- Senere etapper (egne beslutninger): deploy-formidling fra startsiden,
  vei inn for eksterne enheter (etter proxy-agenten), mellommaler.
