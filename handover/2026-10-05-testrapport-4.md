# 05.10.2026 ettermiddag — svar på testrapport 4

Jørns funn (2026-10-05-Test-Report-4.docx) gjennomgått og levert. Alt er
deployet og verifisert i nettleser + på tavla. Merk: Jørns funn «fikk ikke
se agentens dialog i trinn 1» skyldtes at Zoo åpnet kodefaner/diff-visning
OVER chat-webviewen — løst under punkt 5.

## 1. Deltagervelger (`Deltagervelger.svelte`)

- Chip-en sier nå «Deltager: PC10», og uten valg står det
  «Deltager: ingen valgt» — konteksten Jørn savnet.
- Panelet lukkes med ×-knapp øverst til høyre OG ved klikk utenfor
  (`<svelte:window onpointerdown>` + `rot.contains(e.target)`).
- Verifisert i nettleser: åpne/lukk med ×, klikk-utenfor, valg lukker.

## 2. Slett prosjekt med én knapp (`Lobby.svelte`)

- Avskrift av prosjektnavn utgikk. «Slett prosjektet» (disclosure) →
  «Bekreft sletting av prosjektet» (én knapp). Frontend sender slug-en
  selv i DELETE-kallet.
- Verifisert ved å slette testprosjektet zoo-rydding-test den veien.

## 3+4. Nybegynnerskjermen i KODE15-webprofil

- `Samling.svelte`: `.stage.lys`-variant (kun `mal=nybegynner`) med
  `--k15-*`-tokens: varm off-white bakgrunn, hvite kort, Montserrat på
  overskrifter. Standard prosjektskjerm beholder mørk scene.
- Kicker «Samling» → «Prosjekt:» (alle samlingsvisninger).
- Flisen «Hjelper» heter «Editor» igjen. `FlateTile` fikk `lys`-prop
  (lyst flis-hode).

## 3.3 Zoo-chatten ryddet (kun nybegynner, i `entrypoint.sh`)

Tre mekanismer, alle markør-gardet og idempotente per container-start:

1. **Innstillinger som finnes**: `zoo-settings.json` får
   `enableCheckpoints: false` + `autoCloseZooOpenedFiles/NewFiles/
   AfterUserEdited: true` (Zoo lukker selv filene den åpner — deltageren
   ser aldri kode, bare dialogen).
2. **CSS i webview-bygget** (`webview-ui/build/assets/index.css`):
   introblokken (sebra-hero, om-tekst, versjonsnummer), nederste
   kontrollinje (modus/profil/auto-approve/ikoner) og token-/kontekst-
   info skjules; promptfeltet får alltid synlig ramme
   (`--vscode-focusBorder`). Selektorer via `data-testid` som ligger i
   produksjonsbygget.
3. **Strengpatch i webview-js**: placeholder på norsk («Skriv oppgaven
   din her …» / «Skriv en melding …», norm «språk») og @-/kommando-hintet
   fjernet (det var del av placeholder-teksten, ikke et element).
   Zoo er versjonspinnet (3.87.100557), så strengene er stabile —
   ettersees ved bevisst Zoo-oppgradering.

- Tittellinjen («Zoo Code - project - code-server») kan IKKE skrus av i
  web-workbenchen (`window.customTitleBarVisibility` er desktop-only).
  Løst med inline `<style>` i `workbench.html` (gjøres skrivbar for
  coder i Dockerfile). Inline fordi `workbench.css` caches hardt i
  nettleserne (URL uten innholdshash) — HTML-en genereres per request.

## 5. Agent-støy — verifisert med ekte agentkjøring

Sendte en reell oppgave i chatten på en fersk nybegynner-flate:
dialogen var synlig hele veien (les filer → rediger `App.svelte` →
«Task Completed» på norsk), ingen kodefane/diff stjal bildet, ingen
checkpoints, ingen tokenteller. Det lille kontekst-%-merket øverst står
igjen (lavt støynivå, lot det ligge).

## 6. Tavla — rotårsak funnet og selvhelende vakt

- **Rotårsak**: GNOME-utvidelsen `tiling-assistant@ubuntu.com` flisla
  kioskvinduet til venstre halvdel (1905×2140 med 10px-gap) — fullskjerm-
  flagget sto på, men geometrien var flis. Høyre halvdel på Jørns foto
  var et annet vindu/spøkelsesbilde. Utvidelsen er skrudd av på raven.
- **Vakta** (`vegg-kiosk.sh`) håndhever nå fullskjerm hver runde (3s):
  sammenligner VINDUETS geometri med skjermens (flagget er ikke til å
  stole på) og kjører remove,fullscreen → resize → add,fullscreen ved
  avvik. Verifisert innenfra med CDP: viewport 3840×2160.
- **Bifunn fikset**: kiosk-chromium arvet vaktas flock-fd — en død vakt
  etterlot låsen hos kiosken så ny vakt aldri kom opp. Nå `9>&-` ved
  start av nettleseren.
- **Feilsøkingshjelp**: kiosken kjører med `--remote-debugging-port=9223`
  (kun 127.0.0.1). Minimal CDP-klient uten avhengigheter ligger som
  mønster i dette notatet sitt opphav (ren stdlib-websocket) — praktisk
  for å måle viewport/ta skjermbilde av tavla maskinelt.
- Autostart-fila peker på repo-skriptet, så ny innlogging får ny versjon.

## Driftsnotater

- Jørns prosjekt `2026-10-05-joeern-2-slettmeg` fikk GitHub-repo — altså
  er GitHub-appen nå installert på KODE15AS-Nybegynner (ventepunktet fra
  testrapport 3 er løst). Flate-containerne/volumene hans er slettet
  (arbeidet ligger pushet), så neste start får alle nybegynner-patchene.
  Prosjektet kan slettes i lobbyen når Jørn er ferdig.
- Eldre nybegynner-flater (volumer fra før i dag) får webview-CSS-en
  patchet ved neste start (entrypoint), men IKKE zoo-settings-endringene
  (import skjer kun ved første start). Slett/gjenskap flata om
  checkpoints fortsatt vises.
- Agentens svar lenker til `https://startside.studio15.cloud/web/…` —
  det er `PUBLIC_BASE` fra compose, bevisst konfigurert, ikke en feil.
