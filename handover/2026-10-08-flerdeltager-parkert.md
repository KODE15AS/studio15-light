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

## TODO (påminnelse — må besluttes senere)

- ENTEN designe en ordentlig løsning for flere deltagere i samme
  prosjekt (én «vertsflate» for spillet + gjestevisninger? felles
  dialog?), ELLER blokkere muligheten i lobbyen (ett prosjekt = én
  deltagerskjerm) så ingen kan gå seg vill i den.

## Kjent begrensning relevant for dagens test (2 prosjekter)

Tavla viser I DAG ett prosjekt om gangen: uten ?program=&prosjekt= i
URL-en velger den prosjektet med flest kjørende flater. Med to
prosjekter à én flate vises bare det ene. Vurderes sammen med punktet
over (tavle på tvers av prosjekter — naturlig når én deltager = ett
prosjekt blir normen).
