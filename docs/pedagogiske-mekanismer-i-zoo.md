# Pedagogiske mekanismer i Zoo Code — grunnlag for «oppgavepakker»

Kartlagt 06.10.2026 ved å lese Zoo Code 3.87.100557 (vårt pinnede bygg)
fra innsiden. Målet: finne mekanismene vi kan bruke for å definere et
«prosjekt» som en spesialtilpasset, pedagogisk oppgave — der hjelperen
leder deltageren forsiktig i riktig retning. Gjelder Studio 15 LIGHT i
dag og fullversjonen av Studio 15 som pedagogisk verktøy.

Alt under er verifisert til stede i bygget (ikke lest fra dokumentasjon).
Alt er FILER I PROSJEKTREPOET — altså nøyaktig det prosjektmal-mekanismen
vår allerede seeder. En «oppgavepakke» = en prosjektmal-mappe.

## 1. Instruksjonslagene (bruker vi delvis i dag)

| Mekanisme | Plassering | Hva den gir |
|---|---|---|
| `AGENTS.md` | repo-roten | Hovedinstruksen; vi bruker den til spilleplan-regi i dag. Undermapper kan ha egne (innstilling `enableSubfolderRules`). |
| `.roo/rules/` | repo | Regelfiler som alltid leses — vi legger webside-adressen her. |
| `.roo/rules-{modus}/` | repo | Regler som KUN gjelder én modus — f.eks. strengere regler for kode-modus enn for spørre-modus. |

## 2. Egendefinerte moduser — `.roomodes` (ubrukt, størst potensial)

YAML-fil i repo-roten. Hver modus har:

- `slug`, `name`, `roleDefinition` — hvem hjelperen ER i denne modusen
  («Du er en tålmodig spillcoach …»).
- `whenToUse` — når orkestratoren/modellen skal velge modusen.
- `customInstructions` — modusens egne regler.
- `groups` — HVILKE VERKTØY modusen har: read / edit / command / mcp /
  browser. `edit` kan begrenses med `fileRegex`!

Det siste er pedagogisk gull: en «Spillcoach»-modus kan f.eks. bare få
redigere `src/App.svelte` (`fileRegex: "src/App\\.svelte$"`), aldri
config-filer, og ikke kjøre kommandoer i det hele tatt. Da KAN ikke
hjelperen rote seg bort i ting deltageren aldri skulle se — innrammingen
ligger i verktøystilgangen, ikke bare i formaninger.

Moduser kan også eksporteres/importeres som YAML — dvs. en kurator kan
bygge et bibliotek av moduser for ulike oppgavetyper.

## 3. Svarknapper som bytter modus (ubrukt finesse)

`ask_followup_question`-forslagene er `{ text, mode }` — en knapp kan
altså SKIFTE MODUS. Eksempel på gradert hjelp:

- «Bare fiks det for oss» → kode-modus (hjelperen gjør det)
- «Forklar hva som skjer først» → spørre-modus (kan ikke redigere —
  ren samtale, deltageren må selv be om endringen etterpå)

Dette er «forsiktig pushing» satt i system: valget mellom å FÅ løsningen
og å FORSTÅ den blir et klikk, og modusen håndhever forskjellen.

## 4. Oppgaveløpet: todo-liste, deloppgaver, orkestrator

- `update_todo_list` — bruker vi fra i går (trinnene 1–4 i chatten).
- `new_task` kan STARTE EN NY DELOPPGAVE i en valgt modus, med egen
  todo-liste; innstillingen `newTaskRequireTodos` tvinger alle nye
  oppgaver til å ha definerte todos.
- Innebygd `orchestrator`-modus delegerer til andre moduser per
  deloppgave. For fullversjonen: en «Kursleder»-orkestrator som åpner
  trinn 1 som deloppgave i Spillcoach-modus, trinn 5 i quiz-modus, osv.

## 5. Slash-kommandoer — `.roo/commands/*.md` (ubrukt)

Markdown-filer i repoet = kommandoer som kan trigges i chatten (verktøyet
`run_slash_command`). Dette er «lærerens makroer», definert per
oppgavepakke:

- `/hint` — gi ett lite hint på gjeldende trinn, aldri hele løsningen
- `/status` — oppsummer hvor langt dere er kommet, på norsk
- `/nestetrinn` — avslutt trinnet ryddig og presenter neste

Instruktøren (eller AGENTS.md) kan henvise til dem, og de er like for
alle prosjekter fra samme mal.

## 6. Skills — `.roo/skills/<navn>/SKILL.md` (ubrukt)

Gjenbrukbare «oppskrifter» med frontmatter (`name`, `description`) som
agenten henter NÅR DE TRENGS (verktøyet `skill`), uten å belaste
konteksten ellers. Finnes også per modus (`skills-{modus}/`). For oss:

- «flerspiller-websocket» — vår fasit-oppskrift for trinn 2 (Vite-plugin,
  delt tilstand), slik at hjelperen alltid velger samme robuste løsning
  i stedet for å improvisere.
- «datarobot» — oppskriften for trinn 3–4.

Pedagogisk poeng: skills gjør hjelperens LØSNINGER forutsigbare på tvers
av grupper — læreren vet hva som kommer til å skje.

## 7. Egne verktøy via MCP — `.roo/mcp.json` (ubrukt, fullversjon-spor)

Prosjektrepoet kan definere MCP-servere hjelperen får bruke. Det åpner
for VÅRE pedagogiske verktøy, f.eks.:

- `registrer_trinn_fullfort` — melder progresjon til lobbyen/læreren
  (dashbord over alle gruppers trinn — tavla kan vise fremdrift!)
- `hent_fasit` — gir hjelperen (ikke deltageren) tilgang til
  oppgavens fasit/vurderingskriterier
- `sporreundersokelse` — samle refleksjonssvar etter økten

Serveren kan kjøre i containeren (stdio) — ingen nettverksendringer.
Verktøybruk kan auto-godkjennes per server/verktøy.

## 8. Andre funn verdt å kjenne til

- `generate_image` — bildegenerering som verktøy (krever bildemodell i
  proxyen). Kan la deltagerne lage spillgrafikk med hjelperen.
- `codebase_search` — semantisk søk, krever indeksering (embeddings-
  modell i proxyen); uinteressant for små nybegynner-repoer.
- Modus→modell-kobling (`modeApiConfigs`): hver modus kan ha sin egen
  modell/temperatur via proxyen — billig modell til spørre-modus, sterk
  modell til kode-modus. Styres i zoo-settings-importen vår.
- Followup har auto-svar-timeout — vi har allerede skrudd den AV i
  nybegynner-malen (spørsmål skal vente på deltagerne).

## Anbefalt arkitektur: «oppgavepakke» = prosjektmal-mappe

```
lobby/prosjektmal/<pakkenavn>/
├── AGENTS.md            ← pedagogikken: rolle, trinnregi, todo/knapper
├── spilleplan.yaml      ← innholdet: trinn, mål, ferdig-kriterier
├── .roomodes            ← Spillcoach-modus (begrensede verktøy/filer)
├── .roo/
│   ├── rules/           ← faste fakta (webside-adresse — fylles av oss)
│   ├── commands/        ← /hint, /status, /nestetrinn
│   ├── skills/          ← tekniske fasit-oppskrifter per trinn
│   └── mcp.json         ← (fullversjon) progresjon/fasit-verktøy
└── src/ …               ← startkoden deltagerne ser
```

Dagens nybegynner-mal bruker rad 1–2. Resten er neste trappetrinn, i
stigende innsats: commands (billig) → .roomodes (middels) → skills
(middels) → MCP-verktøy (størst, men åpner lærer-dashbordet).

## Forslag til første eksperiment (LIGHT)

1. `/hint`-kommando + «Gi oss et hint»-svarknapp i AGENTS.md-regien.
2. Spillcoach-modus i `.roomodes` med fileRegex-lås til `src/` og
   spørre-variant uten redigeringsverktøy — og modusbyttende knapper
   («Fiks det» vs «Forklar først»).
3. Én skill: flerspiller-oppskriften for trinn 2 (der improvisasjon
   svir mest).
