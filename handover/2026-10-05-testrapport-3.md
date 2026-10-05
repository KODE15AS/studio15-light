# 2026-10-05 — Testrapport 3: prosjektmaler og nybegynnerspillet

Svar på Jørns testrapport 3 (05.10 formiddag). Kjørt autonomt etter
avklaring i chat; alt maskintestet i nettleser mot
startside.studio15.cloud.

## Prosjektmaler (rapportens hovedoppgave)

- To maler ved «Lag nytt prosjekt» (navnene er Jørns):
  - **«Full Zoo Code UI og minimal konfetti-webside»** — den gamle malen,
    uendret (nå i `lobby/prosjektmal/full/`).
  - **«Nybegynner for enkle spill og samarbeide»** — ny
    (`lobby/prosjektmal/nybegynner/`).
- Malen velges med kort i skjemaet, lagres per prosjekt i
  `register/programmer.yaml` (`mal:`-felt, default `full` for gamle
  prosjekter) og styrer både repo-seeding og skjermlayout.
  Arbeidsflate-containere får `S15L_MAL` i miljøet.
- Valideringstekst ved prosjektnavnet (rapportens pkt. 1): live
  forhåndsvisning «Adressen blir `<slug>` (x av maks y tegn)» med samme
  regler som lobbyen (æ/ø/å → ae/oe/aa, 47-tegnsgrensen mot DNS-fella);
  Opprett-knappen sperres når navnet er ugyldig/for langt.

## Nybegynnerskjermen (Jørns skisse)

- Stablet layout i samlingsvisningen når prosjektets mal er nybegynner:
  1. **Webside** øverst (ca. 50 %) — vanlig flis med deltagerfarge.
  2. **Oppgave** i midten: «Trinn X av 4 — tittel» + instruks, med
     Forrige/Neste-knapper. Trinnvalget huskes per prosjekt
     (localStorage).
  3. **Hjelper** nederst: Zoo-chatten som HELE editorflaten.
- Strekkbare skiller mellom radene (pointer capture virker over
  iframene); andelene huskes per prosjekt. BEVISST AVVIK fra skissen:
  skissens felt 3 (chat fra agent) og 4 (deltager-input) er ett og samme
  Zoo-webview (svar over, skrivefelt under) og kan ikke deles i to
  rammer — det er derfor to strekkbare skiller, ikke tre.
- Ryddet editor-UI (all «støy» vekk): settings i entrypoint
  (aktivitetslinje, statuslinje, faner, menylinje, command center av) +
  oppstart-utvidelsen v1.1.1 som lukker editorer, åpner
  `zoo-code.openInNewTab`, tvinger ÉN editorgruppe
  (`workbench.action.editorLayoutSingle` — openInNewTab legger chatten i
  delt kolonne ellers) og lukker sidestolpe/panel. Kjøres to ganger
  (1,5 s + 5 s) fordi første kjøring kan komme før workbenchen er
  ferdig; idempotent via fane-sjekk.

## Informasjonsdesignet mot agenten (rapportens spørsmål 2)

ÉN kilde for spillet: `spilleplan.yaml` i nybegynnermalen (4 trinn med
tittel + instruks, Jørns tekster lett omformulert). To lesere:

1. Oppgavefeltet på deltagerskjermen henter den via
   `GET /api/spilleplan/nybegynner` (lobbyen serverer malens fil).
2. Hjelperen leser samme fil i prosjektrepoet (seedet fra malen);
   `AGENTS.md` i malen gir coach-rollen: les spilleplanen, merk alle
   svar «Trinn X av 4», start på trinn 1, gå videre først når deltageren
   bekrefter at forrige trinn virker, enkelt bokmål, ett spørsmål om
   gangen. Flerspiller-hint: alle åpner SAMME webside-adresse;
   delt tilstand via liten WebSocket-tjener som Vite-plugin.

Merk: trinnet deltakeren står på (oppgavefeltet) og trinnet agenten tror
på er ikke hardkoblet — agenten resonnerer ut fra samtalen og koden.
Vurder kobling senere hvis mennesketester viser forvirring.

## Ny org KODE15AS-Nybegynner (rapportens pkt. 3)

- Prosjektgruppen **Nybegynner** er registrert mot org-en
  `KODE15AS-Nybegynner` (samme app-ID 5176150 som vår vaktmester).
- VENTER PÅ JØRN: vaktmester-appen er IKKE installert i org-en ennå
  (GitHub svarte 404 på installasjonsoppslaget). Ett klikk:
  github.com/apps/studio15-light-vaktmester → Install → velg
  KODE15AS-Nybegynner. Før det feiler prosjektoppretting i gruppen med
  tydelig melding.

## Testet (maskintest 05.10)

- Nybegynnerprosjekt «Spilltest» i saturday-test-2 (ekte GitHub-repo):
  seeding fra riktig mal, S15L_MAL satt, stablet layout, trinnpanel med
  Forrige/Neste, skille-drag (595→690 px, huskes), Zoo-chatten som hele
  hjelperflaten uten editor-støy. Prosjektet står igjen som demo.
- Regresjon: standardprosjekt har fortsatt to kolonner, «Del til tavla»
  virker (deltest), startside/gruppeside OK.
- Kjent kosmetisk (gammelt, ikke nytt): SPA-ruter (/tavle, /gruppe/…)
  serveres med HTTP 404-status men korrekt index.html — nettlesere viser
  siden fint. Kan ryddes ved anledning.
