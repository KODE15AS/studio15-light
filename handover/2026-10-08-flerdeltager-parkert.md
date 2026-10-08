# 2026-10-08 — Flere deltagere i SAMME prosjekt: parkert (Jørn)

Beslutning (Jørn, 08.10 morgen): muligheten for flere deltagere i samme
prosjekt HVILER inntil videre. Testing og videre utvikling skjer med én
deltager per prosjekt (i dag: 2 deltagere på 2 forskjellige prosjekter).

## Bakgrunn

Ideen oppsto underveis (agentens forslag, ikke Jørns opprinnelige plan)
og 07.10-testen avdekket at den skaper gordiske knuter i nybegynner-
sporet:

- To arbeidsflater i samme prosjekt = to containere med hver sin klone
  og hver sin webside — hvem sin webside er «sannheten» for spillet og
  for tavla?
- Delt spilltilstand krever at alle spiller mot ÉN flates webside
  (WebSocket-pluginen bor per container).
- Tavlas eksakte speiling (webrtc) og dialog-speilet er per flate —
  med flere flater i samme prosjekt blir tavledisponeringen og
  «hvilken dialog er prosjektets dialog?» uavklart.

## AVKLART samme dag (Jørn 08.10 ~09:00): regelen er bygget

Jørn valgte å blokkere muligheten og gjøre regelen eksplisitt:

1. **Ett prosjekt = én deltagerskjerm.** `POST /api/arbeidsflater`
   avviser (409) en annen deltager når prosjektet allerede har en
   skjerm; startsiden viser «Prosjektet tilhører X — bli med via
   websiden» i stedet for åpne-knappen. Spiller 2 er alltid gjest via
   webside-adressen.
2. **Én deltager = ett aktivt prosjekt.** Deltageren har en
   `aktiv`-peker (kortnavn) i `register/deltagere.yaml`. Å åpne skjerm
   i et annet prosjekt flytter pekeren dit — INGENTING slettes eller
   stoppes: forrige container kjører videre (til vanlig dvale), så
   veksling tilbake er umiddelbar (målt: ~30 ms når containeren står).
3. **Tavla er deltagerSTYRT, på tvers av prosjekter.** Nytt
   `GET /api/tavle`: én flis per deltager med aktiv, kjørende flate.
   Veggen holder én presence-watch-tilkobling PER prosjektrom
   (`kobleVeggRom` i presence.svelte.js) — webrtc-speiling og
   dialog-speil virker uavhengig per flis. `?program=&prosjekt=`
   begrenser fortsatt til ett prosjekt om ønskelig.

Prosjektsletting nullstiller aktiv-pekere som pekte inn i prosjektet.
E2E-verifisert 08.10: 409-vakt, veksling a→c→a med alle containere
urørt, tavle med to fliser fra to prosjekter (skjermbilde ok), og
full opprydding.

Merk: de to gamle konfetti-flatene i demo (joern-paa-pc +
joern-paa-lenovo) er fra før regelen og bryter den — de ligger i dvale
og er ufarlige, men prosjektet kan bare «eies» av én av dem nå.
