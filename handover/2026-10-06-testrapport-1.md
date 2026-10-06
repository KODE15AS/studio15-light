# 2026-10-06 — Testrapport 1 (morgen)

Jørns testrapport etter nybegynner-iterasjon 2. Fire punkter; 1–3 fikset
og E2E-verifisert, punkt 4 er nettverk/Fortigate (Jørns bane, analyse
under).

## 1. Slette deltager — én bekreftelse ✅

Fantes ikke før (kun registrering). Nå: ×-knapp per rad i
deltagervelgeren → inline «Vil du virkelig slette <navn>?» med
«Ja, slett»/«Avbryt». Ingen avskrift av navn (lærdom fra slett
prosjekt, rapport 4). Backend: `DELETE /api/deltagere/{slug}`.
Arbeidsflater beholder deltagernavnet sitt (kortnavnet bærer det);
var deltageren valgt i nettleseren, nullstilles valget.

## 2. Editor-flisen på tavla var tom → dialog-speil ✅

Rotårsak (reprodusert headless): tavlas editor-flis er en EGEN
code-server-økt, og med nybegynner-oppsettet åpner den en FERSK, TOM
Zoo-chat («Skriv oppgaven din her …») — deltagerens samtale bor i
deltagerens nettleserøkt og kan aldri vises via en ny økt.

Løsning: samtalen ligger som fil i containeren
(`…/globalStorage/zoocodeorganization.zoo-code/tasks/<nyeste>/ui_messages.json`).
Lobbyen leser den med docker exec (`Driver::les_zoo_dialog`) og koker
den ned i `GET /api/arbeidsflater/{kortnavn}/dialog`: tekstmeldinger
(deltager/hjelper), followup-spørsmål med svarforslag, små
verktøylinjer («skriver i src/App.svelte») og «hjelperen jobber»-flagg.
Store felt (diffs, filinnhold) filtreres bort server-side.

Ny `DialogTile.svelte` på tavla (kun nybegynner-malen; full-malen har
editor-flisen som før): bobler i eierfarge/hvitt, svarforslag som
piller, 3 s polling, ruller selv til nyeste. Minimal markdown
(fet/kode/lenketekst), escaping før mønstrene legges på.

## 3. Svart blink hvert 5. sekund på webrtc-delingen ✅

Rotårsak: tavla sender «soek» hvert 5. s (for å fange restarter) —
setet svarte med NYTT tilbud hver gang, og tavla river eksisterende
forbindelse ved tilbud → strøm ned → iframe → strøm opp = svart blink.

Fix i setet (`tilbyTil`): levende forbindelse re-forhandles aldri —
nytt tilbud kun når forbindelsen er død, eller en oppkobling har stått
fast i > 15 s (selvhelbreding når signal går tapt). Verifisert E2E:
video-elementet på tavla identisk og strømmen live gjennom 20+ s
(fire søkerunder) med `?deltest=1`-setet.

## 4. Kablede PC-er når ikke startsiden — Fortigate, ikke raven

Kartlagt: `startside.studio15.cloud` → **10.10.0.22 = ravens
wifi-IP** (wlo1). Vår caddy binder kun wifi-IP-en (+ tailnettet).
Ravens kablede IP er **10.5.0.22** (enp5s0), og portene 80/443 der er
opptatt av elduro-stacken — vi kan ikke bare binde samme URL der.

De tre kablede PC-ene (10.5.0.0/24) må altså krysse Fortigaten over
til wifi-subnettet for å nå 10.10.0.22 — og det stopper i dag på
soneskillet/policy. Anbefaling (Jørns bane):

- **Én Fortigate-policy**: tillat kablet subnett → 10.10.0.22 på
  TCP 80+443. Ingen endringer på raven, samme URL overalt.
- Alternativ: Tailscale på de tre PC-ene (fungerer, men oppsett per
  PC og ts.net-URL med :8100 i stedet for startside-navnet).
- Diagnose på én av PC-ene hvis policy finnes allerede:
  `nslookup startside.studio15.cloud` (skal gi 10.10.0.22) og
  `curl -v https://10.10.0.22/` (timeout = policy/rute mangler).

## Annet

- Tavle av/på-bryter («ut av tavlemodus») er foreslått (flaggfil i
  vakta + knapp i lobbyen) — venter på Jørns valg.
- Zoo-mekanismekartlegging for pedagogiske oppgavepakker:
  `docs/pedagogiske-mekanismer-i-zoo.md`.
