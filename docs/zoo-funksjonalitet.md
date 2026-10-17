# Zoo-funksjonalitet å utforske (påminnelsesliste)

Kryssjekket mot Zoo 3.87-dist 17.10.2026 (alle punktene er verifisert i
koden). Kort status per punkt; detaljene bor i handover-notatene
(designforslag A/B/C i `handover/2026-10-15-handover-chat-3-til-4.md`).
Rollefordelingen Cursor/Zoo er norm «verktøy» i `raven-platform/README.md`.

| Funksjon | Hva | Status |
|---|---|---|
| Egendefinerte moduser | `.roomodes` i reporoten + `.roo/rules-<modus>/`: innholdsmalen kan bære egne «fag-agenter» (f.eks. DEFA-maskinvaremodus, dokumentasjonsmodus) med egne regler og verktøyrettigheter. Globale moduser (`custom_modes.yaml` i Zoo-lageret) eies av UI-regimet. `modeApiConfigs` gir egen modellprofil per modus. | Klar til design (forslag B) |
| MCP-verktøy | Globalt (`mcp_settings.json` i Zoo-lageret) og per prosjekt/mal (`.roo/mcp.json`). Aktuelt senere: Bitwarden-oppslag, deploy på raven. Passer proxy-agent-planen. | Senere |
| customStoragePath | Flytter hele Zoo-lageret (tasks/settings/cache) til `/home/coder/project/.zoo` → samtalene overlever oppfriskede flater og autosaves til repoet. Gitignore `cache/` og `tasks/*/checkpoints/` (shadow-git). Hemmeligheter følger IKKE med (verifisert). | **Anbefalt — gjøres først** (forslag A) |
| Todo-disiplin | `newTaskRequireTodos` (krev todo-liste i nye oppgaver) + `preventCompletionWithOpenTodos` (nekt «ferdig» med åpne punkter). Gratis kvalitetsheving for ekspert. | Klar til å slås på |
| Followup-knapper | `ask_followup_question`: agenten stiller spørsmål med klikkbare svarforslag `{ text, mode? }` — et svar kan bære et modusbytte. Muliggjør gradert hjelp og «forsiktig pushing» satt i system. | Klar til design (med modusene) |
| `.rooignore` | Holder agenten ute av filer — inn i malene for å verne `.env`. | Gjøres ved neste malrunde |
| Sjekkpunkter | Zoo tar snapshot per endring (shadow-git) og kan rulle tilbake. Skjult i blankt UI i dag; vurderes synlig for ekspert. | Vurderes for ekspert |
| Orkestrator / deloppgaver | Innebygd Orchestrator-modus delegerer til underoppgaver (`new_task`). Nyttig i store ekspertprosjekter (DEFA). | Senere (med modusene) |
| Kodebase-indeksering | Semantisk søk; krever embeddings-modell + Qdrant-vektordatabase. RTX 5080 kan senere hoste embeddings. | Utsatt |
| Bildegenerering | Finnes (`generateImage`); krever ekstern bildemodell og proxy-støtte. | Utsatt |
