# 05.10 — svar på «2026-10-05-Test-Rapport-1» (autonom kjøring)

Alt bygget, rullet ut og verifisert 05.10 formiddag. Regresjon: presence
16/16 GRØNN, konfetti med GitHub-variant GRØNN.

## 1) WebRTC-pilot: «Del til tavla» (webside-strøm, énveis)

BESLUTNING (Jørn 05.10, revisjon av erfaringsoverføringens «ingen
streaming»): websiden kan streames énveis deltager → tavle. Editor-flisen
beholder lagrings-syncen (video gjør kodetekst uskarp). Nettbrett er
NØDLØSNING for deling (skjermkoding gir varme/batteritrekk) — PC er
normalen.

Slik virker det:

- Samlingens header har «▶ Del til tavla» (kun synlig når du har egen
  skjerm i prosjektet). Klikk → nettleserens delingsdialog (egen fane er
  forhåndsvalgt — trykk «Del»; dette KAN ikke automatiseres bort, krav i
  nettleseren). Strømmen beskjæres automatisk til webside-flisen
  (Region Capture, Chromium). Knappen blir «■ Deler til tavla» (rosa).
- Tavlas webside-flis bytter fra iframe til video med «● direkte»-merke.
  Faller strømmen bort (stopp, lukket fane, nettverksfeil) kommer den
  levende iframen tilbake av seg selv — piloten kan aldri gjøre tavla
  dårligere enn før.
- Bytter du til kollegaens skjerm mens du deler, stoppes delingen ryddig
  (beskjæringsmålet er din egen webside-flis).
- Signalering går over presence-WebSocketen (`strom`-meldinger: søk/
  tilbud/svar/is-kandidat — serveren er kun postbud; media går ALDRI via
  serveren). Tavla (watch) får egen flyktig adresse (`watch_velkommen`)
  og er fortsatt read-only for alt annet.
- Maskintest-krok: `?deltest=1` på samlings-URL-en bytter skjermfangsten
  med en canvas-strøm uten tillatelsesdialog — hele kjeden E2E-testet
  maskinelt (video oppe på tavla, readyState 4, fallback etter stopp).
  GJENSTÅR menneskelig: ekte getDisplayMedia-flyt med dialogen, og
  Tailscale-PC → raven (testes i neste mennesketest).

Varig ICE-robusthet (Jørns kommentar): **coturn på raven** (pinnet
4.7.0-alpine, host-nett, lytter KUN på 10.10.0.22 og 100.65.19.39, UDP
3478 + reléporter 49160–49200). Klientene prøver direkte først; reléet
er garantert vei PC↔raven over Tailscale. Hemmelighet: `TURN_PASSORD` i
`.env`; klientene får ICE-listen fra `/api/tilstand` (`ICE_SERVERS` i
compose). Ingen offentlig eksponering (raven har ikke offentlig IP;
Funnel rører ikke portene).

## 2a) Tavle-layout speiler deltagerskjermen

Hver halvdel viser nå editor til VENSTRE og webside til HØYRE — samme
plassering som samlingsvisningen (før: editor øverst, webside nederst).

## 2b) «4 kolonner»-mysteriet — rotårsak funnet og fikset

Hendelsesforløpet (fra caddy-loggene): dvale-vakten virket, men (1) en
gjenglemt fane på en deltager-enhet (10.10.0.24) holdt pc-6 våken til
kl. 01 med ~1000 vellykkede kall/time, og (2) kl. 08:08 vekket
GJENÅPNEDE nettleserfaner (Chrome-restore) zombie-flatene len-6/pc-6 —
vekkesiden auto-vekket også fra bakgrunnsfaner. Sammen med de nye
len10/pc10 ga det 4 kolonner; de gamle sovnet igjen 08:55 (45 min).

Fikser:

- **Synlighetsvakt på vekkesiden**: vekking og polling skjer kun når
  fanen faktisk er synlig (`visibilityState`). En gjenopprettet
  bakgrunnsfane vekker ingenting og holder ingenting våkent — flaten
  våkner først når brukeren bytter til fanen.
- **REAPER_IGNORE_IPS** (compose): trafikk fra ravens egne adresser
  (tavla/kiosken via ts.net, localhost, wifi-IP) teller aldri som
  aktivitet — en tavle-restart forlenger ikke flatenes liv.

Zoo-forskjellen på bildet (to «ulike» Zoo-instanser): de høyre var
FERSKE containere som viste Zoos velkomstkort i chatten (normal
tom-tilstand før første oppgave); de venstre hadde oppgavehistorikk.
Samme kilde, ulik alder — kosmetisk, ingen fiks.

## 2c) Fremtidige templates — designnotat (bygges IKKE nå)

Ingenting i 04–05.10-endringene låser dette. Når flere prosjektmaler
kommer:

- **Mal-register** (YAML i repoet, som programmer/deltagere): navn,
  beskrivelse, katalog. Prosjektoppretting får malvelger
  (`PROSJEKTMAL` er allerede en utskiftbar katalog — i dag én mal).
- **Oppsett som metadata per mal**: flisplassering (webside over editor,
  faner, …), hvilken fil tavla/se-modus skal åpne (dagens hardkodede
  `src/App.svelte` i payload-URL-ene flyttes hit), og hvilke
  Zoo-/code-server-innstillinger som strippes (nybegynner-mal med tre
  felt: prompt, agent-svar, oppgave).
- Deltager-UI-et (samling/tavle) leser oppsettet fra prosjektet — ikke
  omvendt. Dermed reduserer ikke dagens faste layout fremtidig frihet.
