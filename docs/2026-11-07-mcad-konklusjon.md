# Konklusjon: MCAD i KODE15.studio — agentarkitektur (07.11.2026)

Konklusjon på Jørns MCAD-strategidokumenter («2026 11 07 Kommentar
studio 1» og «zoocode dev for cadify pro critical friends», begge i
Jørns Dropbox), skrevet i kode15.studio-kontekst. Selvstendig dokument —
kan tas med i andre sammenhenger.

**Verifisert funn som lukker siste infrastrukturspørsmål:** Opper
eksponerer et fullt OpenAI-kompatibelt endepunkt
(`https://api.opper.ai/v3/compat`, med `/chat/completions`,
SSE-streaming og OpenAI-kompatible embeddings). OpenAI-SDK-er fungerer
uendret mot base-URL-en med Opper-nøkkel som API-nøkkel. Dette er
nøyaktig protokollen Zoo Code snakker — Zoo Code kan altså pekes
**direkte** mot Opper, uten mellomlag. Dermed er kjeden
Zoo Code → Opper → 700+ modeller teknisk bekreftet kompatibel i alle
ledd, dokumentert per november 2026
(kilde: docs.opper.ai, verifisert 07.11.2026).

## Hva dette betyr for arkitekturen

1. **LiteLLM-sporet avsluttes.** Den eneste rollen LiteLLM kunne hatt
   (OpenAI-kompatibel oversettelse foran gatewayen) er overflødig —
   Opper leverer den selv. Verktøydiskusjonen Zoo Code vs. LiteLLM er
   dermed endelig avgjort: de var aldri konkurrenter; Zoo Code er
   agentlaget, Opper er modellaget.
2. **Kostnads- og nivåmodellen (subsidiert → betalt) får infrastruktur
   gratis.** Opper har innebygd forbruks-attribusjon per kall
   (merkelapper per gruppe/prosjekt, grupperbar fakturaanalyse) og
   programmatisk nøkkelforvaltning. Forretningsmodellens budsjettnivåer
   blir konfigurasjon, ikke utvikling.
3. **GDPR/skoleverk-kravet ivaretas i modellaget** uten å binde
   agentlaget: modellvalg (inkl. EU-hostede og raske
   inference-leverandører) gjøres per modus i Zoo Code, mens
   databehandleravtale og tilgangsstyring bor hos én gateway-leverandør.
4. **Også Zoo Codes semantiske indeksering** (embeddings + Qdrant) kan
   gå gjennom samme endepunkt — hele agentplattformens modelltrafikk
   samles bak én nøkkel, én avtale, ett målepunkt.

## Samlet arbeidshypotese — nå uten åpne infrastrukturforbehold

| Lag | Valg |
|---|---|
| Agentramme og regi | Zoo Code (domenespesifikke moduser) |
| Modelltilgang, kost, GDPR | Opper (OpenAI-kompatibelt endepunkt) |
| Strukturerte geometriendringer | Zoo KCL/AST + parser/LSP |
| Autoritativ sannhet | Product Domain-modell + dependency-graf (aldri vektordatabasen) |
| Geometri og eksport | Zoo Engine (versjonslåst per godkjent konstruksjon) |
| Gyldighet | Deterministiske engineering-verktøy og -tester |

Sentrale designprinsipper fra critical-friends-vurderingen som fryses:
agentene foreslår **strukturerte engineering-operasjoner** (aldri
vilkårlig tekstredigering), vektordatabasen er aldri autoritativ kilde
til engineering-relasjoner, geometrimotor-versjonen låses og
dokumenteres per godkjent konstruksjon, og latency måles tredelt
(LLM-, orkestrerings- og engineering-latency) fra dag én.

## Anbefalt neste steg

Uendret fra critical-friend-vurderingen, men kan nå startes uten
forbehold: en **Proof of Architecture** som kjører Zoo Code mot Opper i
et KCL-repo med Product Domain-modell og engine-eksport. Første
arbeidsdag i PoA-en bør inkludere den empiriske røyktesten (Zoo Code
konfigurert mot Opper-endepunktet, alle moduser) — dokumentasjonen er
bekreftet, driften skal bekreftes.

Plattformsiden er allerede de-risket: KODE15.studio (i dag Studio 15
Light) kjører Zoo Code produksjonsmessig bak et rent chat-grensesnitt
uten synlig IDE, med malstyrt agentregi — samme rigg som MCAD-agentene
vil stå på. Driftserfaringen (retry-løkker ved tomme modellsvar,
checkpoint-atferd, `.roomodes`/modusprofiler, malstyring) arves av
MCAD-prototypen gratis.
