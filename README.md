# studio15-light

Forenklet 2-personers versjon av Studio 15: to utviklere på hver sin PC
over Tailscale, hver med sin **arbeidsflate** (VS Code i nettleseren med
Zoo Code koblet mot LLM-proxy) og prosjektets **levende webside** (Vite).
Bygget fra scratch — ingen kode gjenbrukes fra Studio 15, bare erfaringene
(se `handover/HANDOVER.md`). Ingen innlogging, ingen database, ingen
offentlig eksponering — med vilje.

## Lenker

| Hva | Tilgang | URL |
|---|---|---|
| Lobbyen (program → prosjekt → arbeidsflate) | 🔒 Kun Tailscale | https://cadify104raven.tail14de1b.ts.net:8100 |
| Samlingen (editor + webside + se/peke/ta over) | 🔒 Kun Tailscale | https://cadify104raven.tail14de1b.ts.net:8100/samling/\<program\>/\<prosjekt\>/ |
| Veggen (70", read-only watch-modus) | 🔒 Kun Tailscale | https://cadify104raven.tail14de1b.ts.net:8100/vegg |
| Arbeidsflate (per deltager) | 🔒 Kun Tailscale | https://cadify104raven.tail14de1b.ts.net:8100/w/\<flate\>/ |
| Levende webside (per arbeidsflate) | 🔒 Kun Tailscale | https://cadify104raven.tail14de1b.ts.net:8100/web/\<flate\>/ |
| Healthcheck (lobbyen) | 🔒 Kun Tailscale | https://cadify104raven.tail14de1b.ts.net:8100/healthz |
| Dette repoet på GitHub | 🌐 Offentlig (krever innlogging) | https://github.com/KODE15AS/studio15-light |

🌐 Offentlig — kan nås fra internett. 🔒 Kun Tailscale — kun internt.

## Begreper

- **Program** = GitHub-org: overbygningen for en samling prosjekter.
- **Prosjekt** = repo + containerne som kjører det som bygges. Persistent,
  gjenopptakbart, slettes som helhet.
- **Arbeidsflate** = editor-container (code-server + Zoo Code + Vite) per
  deltager. To deltagere deler et prosjekt med hver sin arbeidsflate — eller
  jobber i samme flate ved å åpne samme adresse (code-server tåler flere
  samtidige tilkoblinger).

## Arkitektur

Én inngang på tailnettet: **caddy** på port 8100 (HTTPS via
Tailscale-sertifikat) ruter `/` til **lobbyen**, `/w/<flate>/` til
arbeidsflatens editor og `/web/<flate>/` til arbeidsflatens webside — alt
samme origin (forberedt for se/peke-laget i V2).

- **Lobbyen** (Rust/axum + Svelte): programregister (`register/programmer.yaml`
  — YAML i git, ingen database), prosjektoppretting fra mal, arbeidsflater
  ved behov, dvale/vekke og sletteregimet. Eneste komponent som styrer
  containere, kun gjennom faste operasjoner.
- **LLM-proxyen** (LiteLLM, pinnet): Fable 5 primær med automatisk fallback
  til Opus 5, `drop_params: true`, felles master-nøkkel. API-nøkkelen bor
  KUN her — aldri i arbeidsflatene.
- **Arbeidsflate-imaget** (`arbeidsflate/`): code-server 4.140.0 + Node 22 +
  Zoo Code 3.87.100557 (Open VSX, pinnet) med alle image-fiksene fra
  Studio 15 (chown på volum, Copilot fjernet, trust/velkomst av,
  ripgrep-symlink, autolagring, announcement-hack).
- **Presence-laget (V2)**: se/peke/ta over per prosjekt. Samlingen
  (`/samling/<program>/<prosjekt>/`) viser egen flate interaktivt og
  kollegaens med peke-overlay (ghost-cursors med navn og eierfarge);
  dobbeltklikk eller «Ta over» gir kontroll — realisert med code-servers
  flerbrukertilkobling (avklart 03.10), IKKE input-streaming. Kun én
  ekstern controller per flate; ta/slipp broadcastes. Veggen (`/vegg`)
  er alltid read-only (watch-modus: joiner aldri, sender aldri input) og
  viser begge halvdeler med editor + levende webside, rammer og badges.
- **Nettskille** (lærdom betalt én gang): lobbyen ligger på et internt nett
  arbeidsflatene ikke kan rute til, og caddy avviser all trafikk fra
  arbeidsflate-subnettet. Arbeidsflater får aldri docker-socket, host-nett
  eller GPU.
- **Prosjektrepoer** — typen velges ved opprettelse og kan ikke endres:
  har programmet GitHub-org, oppretter vaktmesteren (GitHub App, se
  `docs/vaktmester-klikkeliste.md`) et privat repo i org-en; uten org bor
  repoet som bare-repo på volumet `s15l-repos` (`file:///repos/<slug>.git`).
  Begge arbeidsflater kloner fra og pusher til samme repo; autosave
  committer og pusher hvert 2. minutt. GitHub-push autentiseres med FERSKE
  repo-scopede installasjonstokens (1 times levetid) som arbeidsflaten
  henter via `POST /api/git-token` — det ene, dokumenterte unntaket i
  caddy-vakten (autentisert med per-arbeidsflate-hemmelighet; utsteder kun
  token til flatens eget repo).

## Kjøring

```sh
# Forutsetninger (én gang): .env fra .env.example (chmod 600) og sertifikat:
skript/hent-sertifikat.sh

docker build -t studio15-light-arbeidsflate:v2 arbeidsflate/
docker compose up -d --build
```

Arbeidsflate-imaget bygges separat (det er ikke en compose-tjeneste —
lobbyen oppretter containerne dynamisk).

## Dvale/vekke

Arbeidsflater uten aktivitet i 45 minutter (`IDLE_MINUTES` i compose)
stoppes av lobbyen. Adressen svarer da med en vekkeside (503) som starter
containeren og laster siden på nytt — oppe igjen på ~10 sekunder. Lobbyen
er alltid oppe.

## Sletting

Sletting er sletting: `skript/slett-prosjekt.sh <program> <prosjekt>`
(eller lobbyens UI) fjerner arbeidsflate-containere, volumer, prosjektrepo
og registeroppføring i én operasjon etter eksplisitt bekreftelse.
GitHub-repo/org-steget er manuelt til vaktmester-appen finnes.

## Veggen (70"-skjermen på raven)

Kiosk-oppsettet bor i `vegg/`: `installer-vegg.sh` legger `vegg-kiosk.sh`
i Ravens autostart (og flytter Skjermsamlings wall-watcher til side — to
kiosker kan ikke dele skjermen). Kiosken er helse-gatet mot `/healthz`,
starter når 70"-skjermen (EDID-match) er tilkoblet og lukker seg når
skjermen eller tjenesten forsvinner. Veggen vekker aldri sovende flater
(`?watch=1` på vekkesiden).

## Testing

Maskinelle tester (trygge å kjøre når som helst — engangsprosjektene
slettes sporløst):

```sh
skript/konfetti-test.sh    # hele kjeden, inkl. GitHub-varianten (vaktmesteren)
skript/presence-test.sh    # 16-punkts protokolltest av se/peke/ta over
```

Mennesketesten (full Zoo Code-kjede fra nettleser, begge seter over
wifi/Tailscale) står i `handover/HANDOVER.md` under Leveranser, og
testresepten leveres i chatten ved hver milepæl.

## Utvikling uten Docker

Lobbyen har mock-driver: `WORKSPACE_DRIVER=mock cargo run` i `lobby/`,
og `npm run dev` i `lobby/frontend/` (proxyer API-kall til :8200).
