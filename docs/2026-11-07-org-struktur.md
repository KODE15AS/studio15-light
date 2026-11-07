# Beslutning: én GitHub-org for kode15.studio (07.11.2026)

Frosset i chat 4. Anbefalingen fra handover chat 3 → 4 er vedtatt.
Gjelder det nye produktet. Studio 15 LIGHT beholder dagens
org-per-prosjektgruppe til høstingen; denne fila endrer ikke
`register/programmer.yaml` eller lobby-flyten der.

## Vedtak

Én GitHub-organisasjon. Prosjektgruppe er et registerbegrep, ikke en
org. Deltagerne har ingen GitHub-kontoer: vaktmesteren gjør alt
git-arbeid, og arbeidsflatene får bare tokens scopet til sitt eget
repo (`contents: write` via `/api/git-token`). Org-veggen isolerer
derfor ingen som bruker produktet. Den grensen som betyr noe, er
repo-scopet token — den består.

Én installasjon av vaktmesteren, med «All repositories», varer for
alle grupper. Per-gruppe-sjekken som ble bygget 06.11
(`sjekk_installasjon` før lagring, installasjonslenke i den guidede
flyten) hører hjemme i Light og bygges ikke på nytt.

## Navn

| Ting | Navn |
|---|---|
| GitHub-org | `KODE15studio` (visning; GitHub skiller ikke store/små) |
| Eierkonto | `KODE15AS` (personlig konto, som i dag). Ingen medlemmer inviteres. |
| Plattformrepo | `studio` — rot for selve studio-applikasjonen |
| Grunnmal | `mal-studio` — grunnleggende funksjonalitet som andre maler bygger på |
| Andre maler | `mal-<navn>` |
| Prosjektrepo | `<gruppe>-<prosjekt>` (slugs) |

Org-navnet er uten bindestrek, slik anbefalingen var skrevet. Org-en
opprettes manuelt — GitHub har ikke API for det på denne planen.

## Hva som er reservert

Repo-navnet er identiteten. Prefikset gjør at to grupper kan ha samme
prosjektnavn uten å kollidere.

- `studio` er plattformrepoet. Ingen prosjektgruppe får slug `studio`.
- `mal-studio` er grunnmalen. Andre maler heter `mal-<navn>` og bygger
  på den. Ingen prosjektgruppe får slug `mal`.
- GitHub Teams opprettes ikke nå. Uten GitHub-brukere er et team bare
  en etikett, og det krever organisasjonsrettigheter på appen som
  dagens vaktmester ikke har. Repo-navnet bærer gruppetilhørigheten.
  Teams kan legges på senere uten å døpe om repoer.

Kortnavn-budsjettet for arbeidsflate-adresser (gruppe + prosjekt innen
47 tegn) er uendret. Det er en DNS-grense, ikke en GitHub-grense.

## Hva lobbyen i det nye repoet gjør

- Ny prosjektgruppe er navn i registeret. Ingen org-felt, ingen «lag
  organisasjon på GitHub», ingen installasjonslenke.
- Nytt prosjekt opprettes som privat repo `<gruppe>-<prosjekt>` i
  `KODE15studio`.
- Vaktmesteren sjekker installasjonen én gang, mot den faste org-en,
  ikke per gruppe.
- `mal-studio` er grunnlaget. Andre maler leses fra `mal-<navn>` i
  samme org, så appen ikke må installeres i en mal-org ved siden av.

## Status 07.11

Org-en finnes: https://github.com/KODE15studio (id `353726039`).
Vaktmesteren er installert og verifisert samme dag: JWT →
`GET /orgs/KODE15studio/installation` → installasjonstoken →
`GET /orgs/KODE15studio` svarer 200.

- App: `studio15-light-vaktmester` (App ID `5176150`)
- Installasjon: `179558664`
- Repository access: `all`
- Rettigheter: administration write, contents write, metadata read

Jørn opprettet de to første repoene samme dag. Begge er private,
tomme (ingen commits) og synlige for vaktmesteren:

- https://github.com/KODE15studio/studio — «Rot repo for selve studio
  applikasjonen»
- https://github.com/KODE15studio/mal-studio — «Maler for grunnleggende
  funksjonalitet som også alle andre maler bygger på»

Appen gjenbrukes til det nye repoet er hjemmet. Nytt nøkkkelpar og
eventuelt nytt app-navn tas i høstingen, ikke i dette vedtaket.
