# Designnotat: flere deltakere på samme prosjekt (E2, 07.11.2026)

Reverserer 08.10-blokken («ett prosjekt = én deltagerskjerm») for
kode15.studio, slik strategidagen besluttet. Rammene fra E1 gjelder:
deltakeren er enheten for budsjett, nøkkel og invitasjon. Grillet med
Jørn 07.11 — beslutningene hans er merket.

## Hva som allerede bærer (verifisert i Light-koden)

Mesteparten av 08.10-problemet er løst av ting som er bygget siden:

- **Adresseskjemaet:** kortnavn er alt `<gruppe>-<prosjekt>-<deltager>`
  (55-tegnsvakt). Flere deltakere i samme prosjekt gir unike navn uten
  endring. Det eneste som blokkerer er 409-vakta.
- **Git-identitet:** hver flate committer som deltakeren og pusher med
  ferske repo-scopede tokens. Flere committere i samme repo er vanlig
  git.
- **Samtaler og budsjett:** Zoo-lageret (`.zoo`) bor på flatens eget
  volum — hver deltaker har sin agent og sin dialog. Det er også
  forutsetningen for budsjett per deltaker (E1): forbruket kan bare
  attribueres når agenten er deltakerens.
- **Presence:** rommene er per prosjekt og rosteret bærer flere
  identiteter (bygget for to seter i V2).
- **Tavla:** `GET /api/tavle` gir én flis per deltaker allerede.
- **Dvale:** reaperen virker per flate. Flater i samme prosjekt sover
  og vekkes uavhengig; repoet er fjernlager, så ingen deler
  kjøretilstand.

## Beslutning 1 (Jørn): arbeidsflyt-innstilling per prosjekt

Valget mellom Trunk Based Development og GitHub Flow er
prosjektinnstillingen som styrer deling:

- **`tbd`** — én deltaker, rett på main (dagens modell). Prosjektet
  TILLATER IKKE flere skjermer: 409-vakta består, men betinget av
  innstillingen i stedet for globalt. Flerspill skjer fortsatt via
  gjestemodellen (medspiller på websiden med `?spiller=`) — den
  berøres ikke.
- **`github-flow`** — flere deltakere som faktisk koder sammen. Da kan
  ikke TBD velges: hver deltaker arbeider på egen gren, og alt som
  deles går gjennom merge.

Innstillingen settes ved prosjektopprettelse og bor i registeret.
Maler kan ha standardverdi (nybegynner: `tbd`; samarbeidsmaler:
`github-flow`).

## Beslutning 2 (Jørn): merge-autoritet

I `github-flow`-prosjekter pekes ÉN merge-autoritet ut per prosjekt:

- **en utpekt deltaker** — merge skjer i dennes flate, med agentens
  hjelp (agenten henter grenen, merger, løser konflikter i dialog med
  deltakeren, pusher main), eller
- **en utpekt LLM** — en merge-agent uten menneske i løkka.

V1 bygger deltaker-varianten. LLM-varianten reserveres: den trenger en
agentkjøring utenfor en deltakerskjerm (samme mønster som norm «hugr»)
og designes når behovet er reelt. Innstillingen (`merger:
<deltaker-slug> | llm`) legges i registeret fra start så skjemaet ikke
må endres.

**GitHub Flow uten GitHub-UI:** deltakerne har ikke GitHub-kontoer, så
mønsteret (gren → gjennomgang → merge) realiseres i produktet.
Grenene bor i prosjektrepoet (`deltaker/<slug>`), og merge er en
git-operasjon i merge-autoritetens flate — ikke en pull request på
github.com. Vaktmester-tokens (contents: write) dekker dette; ingen
nye app-rettigheter trengs.

## Beslutning 3 (Jørn): deling ved milepæler

Agenten deler deltakerens arbeid ved milepæler og henter de andres
før nye oppgaver — deltakeren merker det ikke. Konkret i
`github-flow`:

- Milepæl nådd → agenten committer og pusher deltakerens gren, og
  melder merge-autoriteten via presence (merke i rosteret: «klar til
  merge»).
- Ny oppgave startes → agenten henter main først (fast regel i
  malenes regelfiler, samme mekanikk som «se i tilkoblingsloggen før
  du spør» fra 06.11).

Ingen kontinuerlig autosynk — det gir flest konflikter og minst
forståelse.

## Beslutning 4 (Jørn): adgang følger gruppen

Alle deltakere i prosjektgruppen kan åpne egen skjerm i et
`github-flow`-prosjekt — invitasjonslenken (per deltaker, 14 dager,
E1) ER adgangen. Ingen ny invitasjonsmekanikk per prosjekt.
409-vakta i `tbd`-prosjekter gir fortsatt klar beskjed om hvem
prosjektet tilhører.

## Beslutning 5 (Jørn): tavla viser inntil 4, demokratisk

Maks 4 fliser på tavla, én per deltaker som i dag (4 = 2×2).
Prosessen for HVEM som vises når flere enn 4 kandidater finnes skal
være demokratisk — detaljene fastsettes eksperimentelt (som
HUD-målene). Startforslag som bygges først: de 4 sist aktive vises,
og en deltaker kan be om tavleplass («vis meg») som bytter ut den
eldste. Stemmegivning og finere regi vurderes når klasserommet har
prøvd det enkle.

## Hva som IKKE endres

- Gjestemodellen for flerspill (invitasjonsregien, `?spiller=`,
  medspiller-API-et) — uavhengig av arbeidsflyt-innstillingen.
- Kortnavn, dvale-regnskap, presence-protokollen, én-deltaker =
  ett-aktivt-prosjekt (aktiv-pekeren).
- Light: ingenting av dette bygges i studio15-light. Designet
  realiseres i høstingen (E3) i `KODE15studio/studio`.

## Konsekvenser for E3 (høstingen)

1. Registerskjemaet får `arbeidsflyt: tbd | github-flow` og `merger`
   per prosjekt.
2. 409-vakta porteres som betinget (kun `tbd`).
3. Entrypoint/regelfiler får gren-regimet (gren per deltaker, hent
   main før ny oppgave, push ved milepæl) — kun i
   `github-flow`-flater.
4. Presence får «klar til merge»-merket i rosteret.
5. Tavla får 4-taket og «vis meg»-byttet.
