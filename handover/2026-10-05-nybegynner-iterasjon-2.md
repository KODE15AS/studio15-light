# 05.10.2026 kveld — nybegynner-iterasjon 2 (Jørns konklusjon etter rapport 4)

Jørns beslutninger: nettbrett-testing avsluttet (kun PC-er videre),
side-ved-side-layout, oppgavefeltet utgikk (hjelperen eier spilleplanen),
agentisk todo + klikkbare svarforslag skal utforskes, skrift maks +10–20 %,
tavla må håndtere 1–4 deltagere dynamisk. Alt levert og verifisert.

## Deltagerskjermen (nybegynner): to kolonner

- `Samling.svelte`: hjelper-chatten ⅓ til venstre, websiden ⅔ til høyre.
  Skillet dras i bredden (pointer capture, 15–85 %-klemme) og huskes per
  prosjekt (`s15l-kolonner-…`). Oppgavefeltet, trinnknappene og
  spilleplan-hentingen i frontenden er fjernet — ingen pynteareal.
  (/api/spilleplan-endepunktet står igjen ubrukt i backend.)
- Verifisert i nettleser: standard 1:2 eksakt (724/1448 px), dra virker
  og lagres.

## Venstresidens indre deling (respons/prompt) — viktig læring

Prompt-feltet bor INNE i Zoo-webviewen. Zoos autosize-komponent setter
høyden med inline `!important` hvert tastetrykk — all CSS taper. Løsning:
radgrensene patches i webview-bygget: `minRows:3,maxRows:15` →
`minRows:8,maxRows:28`. Promptfeltet er dermed stort i ro (~¼ kolonne)
og vokser selv til ~halv kolonne når deltagerne skriver langt. Manuelt
drag av den indre delingen er IKKE levert — det ville sloss mot
komponenten; autovekst dekker behovet i praksis.

## Hjelperen eier spilleplanen (AGENTS.md i nybegynner-malen)

- **Todo-liste**: hjelperen oppretter huskeliste med de fire trinnene i
  første svar og holder den oppdatert hver tur (gjeldende trinn aktivt,
  ferdige avkrysset). Zoo-innstilling `todoListEnabled: true`.
- **Trinn-ferdig-kriterier** (formulert i AGENTS.md): (1) endringen er
  levende på websiden, (2) deltagerne har PRØVD den der, (3) deltagerne
  har bekreftet med svarknapp/melding. Aldri videre uten punkt 3.
- **Svarknapper**: hvert svar som trenger noe fra deltageren avsluttes
  med oppfølgingsspørsmål med 2–4 korte norske forslag.
  `alwaysAllowFollowupQuestions: false` — spørsmål auto-besvares ALDRI.
- **Verifisert med ekte agentkjøring**: hjelperen opprettet huskelisten
  (0/4, trinn 1 aktivt), spurte «Hvordan vil dere at brettet skal se ut?»
  med fire klikkbare forslag; klikk på «Klassisk med X og O» ga ferdig
  bondesjakk live på websiden + nytt spørsmål «Har dere prøvd spillet på
  websiden?» med bekreftelsesknapper. Nøyaktig flyten Jørn bestilte.

## Skrift og støy

- `chatFontSize: 15` (standard 13 ≈ +15 %, innenfor Jørns tak). CSS gir
  promptfeltets textarea samme størrelse (Zoo lar den stå på 13 ellers).
- VS Code-varsler (toasts: «port 5173 available», settings-import)
  skjules i nybegynner-flater (inline-stilen i workbench.html) — de
  støyet både hos deltageren og på tavla.

## Tavla: dynamisk disponering 1–4 (+)

- `Vegg.svelte`: 1–2 flater = 2 kolonner i én rad (04.10-regelen om tom
  halvdel «plass til nestemann» beholdt), 3–4 = 2×2-rutenett (hver rute
  1920×1080 på 70-tommeren), >4 = to rader med flere kolonner.
- Hver rute speiler deltagerskjermen — nybegynnerprosjekter speiler også
  ⅓/⅔-delingen (`section.nybegynner`, mal følger med fra /api/tilstand).
- Verifisert på raven via kioskens CDP-port: 4 flater → 2×2 (skjermbilde),
  2 → to kolonner én rad, 1 → to kolonner med én tom.

## Feller møtt underveis (for ettertiden)

- Python-heredoc i entrypoint: en edit ødela innrykket → containeren
  restartet i løkke med IndentationError. `bash -n` + `ast.parse` på
  PY-blokkene er nå billig forsikring før image-bygg.
- Zoo-webview-patcher gjelder PER CONTAINER-OPPSTART med markørvakter;
  endringer i workbench.html-stilen krever image-rebuild + gjenskaping
  av flater (volum-slett) for å nå eksisterende flater.
