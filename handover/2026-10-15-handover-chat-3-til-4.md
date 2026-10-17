# Handover chat 3 → 4 (15.10.2026, kveld)

Chat 3 leverte ekspert-malen og gjorde den moden gjennom Jørns DEFA-
pretest. Denne handoveren samler kveldens leveranser, beslutningene som
venter, og ferdig undersøkte designforslag for chat 4.

## Levert og verifisert i chat 3 (alt committet og pushet)

- **Ekspert-malen** (09.10): raven-fakta i stedet for regi, blankt UI +
  autokjøring, dokumentopplasting → `innboks/`. Se
  `2026-10-09-ekspertmal.md`.
- **Innboks-vakten** (15.10): opplasting vekker Agenten automatisk i
  fullskjermfane — les, oppsummer, strukturer. Ro-vakt mot pågående
  samtaler, filsjekk mot «Tøm innboksen»-angring. Se
  `2026-10-15-innboksvakt.md`.
- **Innboks-linjen** (15.10): grønn statuslinje på deltagerskjermen står
  til innboksen er tømt; «Tøm innboksen»-knapp; lobby-API
  `GET/POST /api/arbeidsflater/{kortnavn}/innboks[/tom]`.
- **Samtalehistorikk** (15.10): «Ny samtale — prosjektet beholdes»
  sletter aldri noe; fanehodet viser ✏️ Ny samtale og 🕘
  Samtalehistorikk (showTabs single, norske tooltips, støyknapper
  fjernet). Jørn-testet i DEFA.

## Venter på Jørns beslutning

1. **Mal-org:** innholdsmaler som egne repoer i ny org. Forslag:
   `STUDIO15-Maler` (Jørns utgangspunkt var «STUDIO15-templates»).
   Vaktmester-appen må installeres i orgen når den opprettes.
2. **Samtalehistorikk til repo:** design klart (under), valg av variant
   gjenstår.

## Utsatt til senere chat (Jørns beslutning 15.10)

- **Zoo-oppgradering 3.87.100557 → 3.88.0.** Undersøkt og klargjort:
  3.88.0 er stabil utgivelse (3.89.x er pre-release og hoppes over).
  Relevant for oss: pålitligere lagring av oppgavemeldinger (leses av
  dialogspeil + innboks-vakt), ryddigere oppgave-avbrytelse, raskere
  filskriving, bedre håndtering av egendefinerte modellnavn mot
  OpenAI-kompatible gatewayer (= vår LiteLLM). **Alle patch-ankre i
  entrypoint er maskinelt verifisert til stede i 3.88.0-vsix-en**
  (norske tekster, knappeskjuling, fanehode-meny, API-et vakten bruker).
  Lav risiko; kjøres som egen kontrollert runde: bump i Dockerfile,
  rebuild, verifiser på en testflate før deltagerflater friskes opp.

## Designforslag A: samtalehistorikk til prosjektrepoet

Dagens svakhet: Zoo-lageret bor i container-fs → historikken dør når en
flate friskes opp (sett 15.10). Jørn vil ha samtalene **i repoet**.

Undersøkt (Zoo 3.87-dist + reelt lager fra DEFA-flaten):

- `zoo-code.customStoragePath` flytter HELE lageret: `tasks/` (per
  samtale: `ui_messages.json`, `api_conversation_history.json`,
  metadata), `settings/` (`mcp_settings.json`, `custom_modes.yaml`) og
  `cache/`. API-nøkler/profiler bor i VS Codes secret storage og følger
  IKKE med — ingen hemmeligheter i lageret (verifisert på kveldens
  samtaler).
- Størrelse: to DEFA-samtaler = 108 K tasks + 572 K modellkatalog-cache.
  Cache bør gitignores; tasks er git-vennlige (JSON).
- Viktig felle: sjekkpunkter er git-repoer (shadow git) under
  `tasks/<id>/checkpoints/` — nestede .git-kataloger blir gitlinks og
  kan IKKE committes meningsfullt. Ikke observert aktive hos oss, men
  må gitignores uansett.
- Merk: `/home/coder/project` er BÅDE prosjektvolumet og git-arbeidstreet
  — «til volumet» og «til repoet» er samme sti; forskjellen er bare om
  stien gitignores eller committes.

Anbefalt variant: `customStoragePath = /home/coder/project/.zoo` satt i
entrypoint (ekspert-malen), med `.gitignore` i malen for
`.zoo/cache/` og `.zoo/tasks/*/checkpoints/`. Autosaven (hvert 2. min)
committer da samtalene fortløpende til GitHub-repoet. Restrisiko å
akseptere: samtaletekst kan sitere miljøverdier hvis agenten skriver dem
ut (repoene er private). Premium-variant (senere): eksportere lesbare
markdown-transkripter til `handover/samtaler/` i stedet for rå JSON.

## Designforslag B: Zoo-moduser som del av malregimet

Undersøkt modus-systemet (Roo-arv, verifisert i dist):

- Innebygde moduser: Architect, Code, Ask, Debug, **Orchestrator**
  (deloppgave-delegering via `new_task`).
- Egendefinerte moduser på to nivåer — nøyaktig skillet vi ønsker:
  - **Globalt** (`settings/custom_modes.yaml` i Zoo-lageret): eies av
    UI-regimet/plattformen — kan settes fra entrypoint per mal.
  - **Per prosjekt** (`.roomodes`-fil i reporoten): eies av
    innholdsmalen — følger malrepoet.
- En modus definerer: slug, navn, `roleDefinition` (systemprompt),
  `whenToUse`, `customInstructions`, `groups` = verktøyrettigheter
  (read/edit/command/mcp, med fil-regex-begrensning, f.eks. «kun
  *.md»), `allowedMcpServers`, og regler i `.roo/rules-<slug>/`.
- `modeApiConfigs`: egen modellprofil PER modus — f.eks. tunge moduser
  på claude, lette/lokale moduser på fremtidig RTX 5080-backend.

Skisse: ekspert-UI-regimet skiper 2–3 norske moduser (f.eks.
«Prosjektleder» (orchestrator-variant), «Utvikler», «Dokumentar» med
md-begrensning); en innholdsmal som DEFA legger fagmoduser i `.roomodes`
+ `.roo/rules-<slug>/`. Blankt UI skjuler modusvelgeren i dag — må få
en enkel, norsk velger når modusene kommer.

## Designforslag C: RTX 5080 i Studio 15 (forberedende tanker)

Spørsmål fra Jørn: lokal RTX 5080 (egen container/repo, norm
«container») som skal brukes lokalt OG fra andre servere på samme
10G-nett — container-til-container, MCP eller annet?

Anbefaling: **ingen av delene — bruk HTTP/OpenAI-kompatibelt API bak
LiteLLM-proxyen.**

- GPU-maskinen kjører en inferens-server (vLLM anbefalt; ev.
  llama.cpp/Ollama) som egen 1:1-stack, eksponert som
  OpenAI-kompatibelt endepunkt på 10G-nettet.
- Registreres som ny modell i `proxy/litellm-config.yaml` (én blokk:
  `model_name: lokal` → `api_base: http://<gpu-vert>:8000/v1`).
  Dermed får ALT i Studio 15 (Zoo, fremtidige tjenester) tilgang uten
  endringer — og fallback-regimet (standard/reserve) gjelder også
  lokalmodellen.
- Hvorfor ikke MCP: MCP er verktøykanal for agenter, ikke
  modell-inferens — feil lag. (MCP blir derimot riktig når GPU-en skal
  tilby TJENESTER, f.eks. bildegenerering som verktøy.)
- Hvorfor ikke rå container-til-container: på tvers av fysiske maskiner
  er det uansett HTTP over nettet; OpenAI-dialekten gir gratis
  kompatibilitet med hele økosystemet.
- Kobling til modus-designet: `modeApiConfigs` kan rute lette moduser
  til lokalmodellen og tunge til claude — kostnadsstyring per modus.

## Maler: org, vaktmester og «Lagre som mal» (17.10)

- Ny org `STUDIO15-Maler` for innholdsmaler; vaktmester-appen verifisert
  installert der (administration/contents: write, alle repoer).
  Org-navnet kan overstyres med `S15L_MAL_ORG` i `.env` (default
  `STUDIO15-Maler`).
- Jørns beslutning: **mal er mal** — ikke skille UI-maler og
  innholdsmaler. UI-regimet er feltet `ui:` i malens `mal.yaml`.
- Bygget «Lagre som mal»: `POST /api/prosjekter/{program}/{prosjekt}/
  lagre-som-mal` + skjema i prosjektkortet i lobbyen. Renset kopi
  (uten .git-historikk, innboks-innhold, dist, node_modules, .zoo)
  pushes som `mal-<slug>`; force-push gjør samme navn = oppdater malen.
- E2E-verifisert 17.10: nybegynner-prosjekt → `mal-nybegynner-standard`
  i STUDIO15-Maler med korrekt `mal.yaml` (ui: nybegynner). Malen står
  igjen som eksempel; testprosjektet er slettet.
- Runde 2 (17.10, etter Jørns test): komplett malsirkel bygget og
  E2E-verifisert.
  - «Lagre som mal» har to varianter (Jørns beslutning): MED innhold
    (webside + programfunksjonalitet, aldri chatten — standard,
    avkrysningsboks i skjemaet) eller REN mal (purged skjelett for samme
    ui-regime). Før innholds-lagring kjøres git-flush (commit+push) i
    prosjektets kjørende containere — autosaven går bare hvert
    2. minutt, så uten flush manglet de siste minuttenes arbeid (det
    var dette Jørn opplevde som «sletter alle brukerdata»).
  - Plassholder-reversering: ved lagring flettes kildeprosjektets
    navn/slug TILBAKE til __PROSJEKTNAVN__/__PROSJEKTSLUG__ i
    flettefilene, så neste prosjekt fra malen får sitt eget navn.
  - Malvelgeren i «Nytt prosjekt» henter nå GET /api/maler = `mal-*` i
    STUDIO15-Maler (mal.yaml gir navn/ui/beskrivelse); innebygde
    skjeletter er reserve når org-en er utilgjengelig. De tre innebygde
    malene er flyttet inn i org-en (mal-full, mal-nybegynner,
    mal-ekspert, med plassholdere intakt).
  - Seeding fra org-mal: grunn klon → mal.yaml ut → navnefletting →
    push. E2E: prosjekt fra mal → endring → lagret som mal med innhold
    → nytt prosjekt fra den malen arvet innholdet med riktig navn.
  - Kopi av tidligere prosjekt = «Lagre som mal» + nytt prosjekt fra
    malen (bevisst: én mekanisme, ikke to).

## Verktøyroller og Zoo-funksjonsliste (17.10)

- Ny norm «verktøy» i `raven-platform/README.md` (Jørns beslutning):
  Cursor = backend (utviklingsarbeid på plattformen, Jørn), Zoo Code =
  frontend (sluttbrukere/deltakere i produktene; Jørn kun testing).
  Motiv: full tilpassbarhet + forberedt hybrid modellkjøring (RTX 5080 /
  Zoo-økosystemets modeller); claude fable 5 i dag er bekvemmelighet.
- Jørns påminnelsesliste over Zoo-funksjonalitet er kryssjekket mot
  3.87-dist-en (alt verifisert, inkl. followup-svar med modusbytte og
  `.roo/mcp.json` per prosjekt) og bor i `docs/zoo-funksjonalitet.md`.
- Norm «tbd» presisert (Jørn 17.10): TBD gjelder backend; frontend
  (deltakernes prosjektrepoer) kjører TBD nå, men skal kunne flyttes til
  GitHub Flow (branch → PR → senior-review → merge) senere — ikke bygg
  frontend-flyt som antar at alt skjer på `main`. Berører på sikt
  autosaven (pusher til main i dag), «Lagre som mal»-flushen og
  git-flyten i arbeidsflatene.

## Designforslag A GJENNOMFØRT (17.10): samtaler på prosjektvolumet

Jørns beslutning: chatten skal IKKE i repo (for tungt) og IKKE i maler —
men den skal overleve oppfriskinger. Løsning: volum-varianten (gitignorert):

- `zoo-code.customStoragePath = /home/coder/project/.zoo` (entrypoint,
  alle maler) — lageret bor i arbeidstreet på prosjektvolumet.
- `.zoo/` gitignoreres: linje i alle tre malene (lokalt + org-repoene) og
  `.git/info/exclude` fra entrypoint som dekker eldre prosjektrepoer.
- «Fersk container»-sjekken for settings-importen ser nå på BEGGE lagre
  (gjenskapt container med levende volum-lager beholder brukeroppsettet).
- Leserne av lagerstien oppdatert med kandidatstier (ny først, gammel
  reserve): `les_zoo_dialog` i driveren og innboks-vaktens ro-sjekk.
- E2E-verifisert: .zoo opprettes på volumet, git ser den ikke
  (check-ignore), gammel sti tom, og lageret OVERLEVDE simulert
  oppfrisking (docker rm + gjenskaping). Eksisterende flater får flyttet
  lager ved neste oppfrisking (gamle samtaler i container-fs følger ikke
  med — de dør som før, én siste gang).

## Småjusteringer etter volum-flyttingen (17.10, ettermiddag)

- DEFA-pretestflaten migrert manuelt (engangs): gammelt chatlager kopiert
  til `.zoo` på volumet før oppfriskning — Jørn verifiserte at samtalene
  kom tilbake. Nye flater trenger ingen migrering.
- Workspace-filteret i samtalehistorikken («Workspace: Current/All»)
  patches bort i webview-bygget (entrypoint): irrelevant hos oss — én
  flate = ett workspace, lageret er låst til volumet. Patch-ankeret er
  verifisert uendret i 3.88-vsix-en.
- Redigerbare samtale-titler finnes IKKE i Zoo (verifisert 3.87 OG 3.88:
  ingen renameTask/taskTitle) — tittelen i historikken er samtalens
  første melding. Ettersees ved fremtidige Zoo-oppgraderinger; ekte
  løsning krever endring oppstrøms eller egen fork.

## Annet verdt å vite (fra 15.10)

- Zoo-funksjoner kartlagt og tilgjengelige ved behov: `.rooignore`
  (verne .env — bør inn i ekspert-malen uansett), todo-disiplin
  (`newTaskRequireTodos`, `preventCompletionWithOpenTodos`),
  sjekkpunkter (skjult i blankt UI i dag), kodebase-indeksering (krever
  embeddings + vektordb — utsatt), bildegenerering (krever bildemodell
  — utsatt).
- Overvåkningsmønster brukt i kveld: docker exec-polling av innboks/
  oppgaver/filer med varsling — gjenbrukbart ved senere deltagertester.
