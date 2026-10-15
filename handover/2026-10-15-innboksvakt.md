# 15.10 — Innboks-vakten: Agenten våkner selv ved dokumentopplasting

## Testfunn (Jørn, DEFA-pretest)

Jørn lastet opp DEFA-handoveren i sitt nye ekspert-prosjekt — opplastingen
fungerte, men så skjedde ingenting. Forventningen var å «se en dialog
agenten har med seg selv». Rotårsak: Zoo (som alle LLM-agenter) handler
bare på en tur — ingen chatmelding, ingen handling. Dokumentet ble
liggende i `innboks/` til noen skrev i chatten.

## Løsning: innboks-vakt i oppstartsutvidelsen

`arbeidsflate/oppstart-extension/extension.js` (aktiv når
`S15L_MAL=ekspert`):

- Poller `innboks/` hvert 3. s; kjente filer huskes i
  `/home/coder/.local/state/s15l-innboks-sett.json` (mtime+størrelse).
  Første kjøring uten tilstandsfil noterer eksisterende filer UTEN å
  varsle (malens `les-meg.md` og gamle opplastinger skal ikke fyre).
- Ny fil → 4 s ro-periode (flere filer i én opplasting blir ÉN beskjed)
  → automatisk Zoo-oppgave med norsk instruks: les, oppsummer for
  Deltageren, strukturer inn i prosjektet per AGENTS.md, foreslå neste
  steg.
- Trigger via Zoo-utvidelsens eksporterte API (Roo-arv, verifisert i
  dist): `startNewTask({text, newTab: true})`. **newTab er avgjørende**
  — uten den havner oppgaven i den lukkede sidestolpen (smal stripe,
  verifisert i test); med den gjør Zoo selv closeAllEditors + ny
  fullskjermfane, samme utseende som blankt UI ellers. Etterfulgt av
  layout-opprydding (editorLayoutSingle, closeSidebar, closeAuxiliaryBar).
- Ro-vakt: `getCurrentTaskStack` ser bare sidestolpe-provideren, så
  aktivitet måles i stedet på mtime av `ui_messages.json` under Zoo-
  oppgavelageret. Starter aldri ny oppgave hvis siste melding er <90 s
  gammel (newTab lukker alle faner — skal ikke rive vekk en pågående
  samtale). Tålmodighet: opptil ~30 min, så gir den seg (AGENTS.md
  instruerer uansett agenten om innboksen).

Følgeendringer:

- Kvittering på skjermen: «… Agenten begynner på det i chatten om et
  øyeblikk» (før: «be Agenten ta det inn»). `Samling.svelte`, lobby
  bygget og rullet ut.
- Malens AGENTS.md: agenten forklares at systemmeldingen ved opplasting
  ikke er skrevet av deltageren, men skal følges.

## Verifisert maskinelt (15.10)

Full E2E i egen nettleserøkt mot DEFA-pretesten: fil i innboks →
automatisk oppgave i fullskjermfane → agenten pakket ut docx-en, laget
todo-liste, skrev `docs/prosjektbrief-defa-kaldstart.md` og oppdaterte
`handover/HANDOVER.md` i prosjektet — synlig selvdialog hele veien.
Vaktens tilstandsfil deretter nullstilt (`{}`) så Jørns egen økt
utløser dialogen på nytt når han åpner skjermen.

## Utrulling

- Nytt arbeidsflate-image (`studio15-light-arbeidsflate:v2`); begge
  ekspert-flatene gjenskapt på nytt image (prosjektvolumene urørt).
- Lobby bygget og restartet (ny kvitteringstekst).

## Runde 2 samme kveld (Jørns korrigeringer etter test)

1. **Innboks-linjen:** grønn statuslinje på deltagerskjermen som står så
   lenge det ligger dokumenter i `innboks/` — med «Tøm innboksen»-knapp
   (angre). Lobby: `GET/POST /api/arbeidsflater/{kortnavn}/innboks[/tom]`
   (docker exec via ny `exec_sh`-hjelper i driveren); skjermen poller
   hvert 8. s og kvitterer ulikt for «Agenten har tømt» vs. «du tømte».
   Vakten i containeren sjekker at filene fortsatt finnes før den vekker
   agenten (angre-vinduet). E2E-testet: opplast → liste → tøm → tom.
2. **Samtalehistorikk:** «Ny samtale — prosjektet beholdes» SLETTER ikke
   den gamle samtalen — den ligger i Zoo-historikken, som lå gjemt i
   «…»-menyen i et usynlig fanehode (`showTabs: "none"` kollapser raden).
   Ekspert-malen setter nå `showTabs: "single"`: fanehodet vises med to
   knapper — ✏️ «Ny samtale» og 🕘 «Samtalehistorikk» (norske tooltips
   via package.nls.json-patch). Settings/marketplace/popout fjernet fra
   fanehodet (package.json-patch), split/close/mer skjult med CSS
   (ikonklasser verifisert i DOM). Historikk-klikk åpner Zoo-historikken
   med søk og gjenopptak — verifisert i nettleser.

## Funn til senere

- **Zoo-oppgavehistorikken bor i container-fs**, ikke i prosjektvolumet
  (`.local/share/code-server/User/globalStorage/...`): gjenskapes en
  container, mistes chat-historikken (prosjektfilene består). Vurder
  volum/flytt hvis historikk blir viktig.
- Hver nettleserøkt har egen extension host med egen vakt; tilstandsfila
  på disk deles, så første økt som ser filen «vinner» triggeren. Med én
  deltager per flate (dagens modell) er dette riktig oppførsel.
