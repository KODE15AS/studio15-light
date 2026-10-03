# Handover 0 → 1 (2026-10-03)

Chat 0 («Handover 0») etablerte repoet og hele beslutningsgrunnlaget.
`HANDOVER.md` er målbildet — denne fila er tilstanden ved overgangen.

## Tilstand

- Repo klonet til `~/dev/studio15-light` på raven (git@github-kode15).
- Plassholder-container kjører: http://100.65.19.39:8100 (kun Tailscale,
  pinnet busybox, `docker compose up -d --build`).
- Begge erfaringsoverføringene er lest og flettet inn i `HANDOVER.md`:
  Studio 15 → LIGHT (arbeidsflaten, alle fellene, Zoo Code-rådene) og
  Skjermsamling → LIGHT (veggen, presence, kiosk, controller-mønsteret).
  Kildene bor i `~/dev/studio15/` og `~/dev/raven-skjermsamling/`.
- VeloStack-notatet er behandlet og begrepet droppet — alle beslutninger
  fra den runden står i `HANDOVER.md` under «Beslutninger».
- Alle åpne spørsmål er avklart med Jørn. Ingen beslutninger gjenstår for
  å starte V1.

## Viktigste beslutninger (detaljer i HANDOVER.md)

- Program = ekte GitHub-org; prosjekter = container/repo-sett under
  programmet; workspaces per deltager ved behov.
- Vaktmester-appen (GitHub App) gjenopplives for repo-automatikk.
- Alt bor i git — ingen database. Programregister som YAML i dette repoet.
- HTTPS via `tailscale cert`; dvale/vekke med lobbyen som vekkeside;
  felles master-nøkkel i LiteLLM-proxyen; full tilgang med
  terminal-autokjøring av; sletting er sletting (org-steget manuelt).
- V1 = tynn E2E-skive; vegg + se/peke/ta over er V2.
- Web-design: `webprofil-kode15`. Norm «tbd»: rett på `main`.

## Chat 1 starter her — oppdrag: bygg V1 og test den

Døp chatten «Handover 1» (prosjektet navngir chattene slik). Oppdraget fra
Jørn (03.10): **start byggingen av V1 og test den ut.** Ikke vent på flere
avklaringer — alle beslutninger er tatt (se HANDOVER.md).

1. **Zoo Code-verifiseringen** (HANDOVER.md «Åpne punkter») — alt annet
   avhenger av den: utvidelses-ID, distribusjon (Open VSX? nedlastings-URL
   til Dockerfile), `.roomodes`-format, announcement-hack, ripgrep-fellen,
   og bekreft ren webapp (ingen GUI-streaming).
2. Bygg deretter etter V1-listen i HANDOVER.md: arbeidsflate-image →
   LiteLLM-proxy → lobby. Hvert trinn E2E-verifiseres maskinelt før
   mennesketest; idempotente operasjoner; engangsskript utenfor git.
3. Test: konfetti-testen (modus → proxy → modell → filredigering → synlig
   på levende webside) er V1s akseptansetest — kjør den maskinelt, og meld
   fra til Jørn når den er grønn og begge sete-URL-ene er klare for
   mennesketest fra wifi/Tailscale.
4. Vaktmester-appen krever manuelle GitHub-steg fra Jørn — forbered
   nøyaktig klikkeliste før han involveres. Begynn med det som ikke
   trenger ham.
5. Merk begrepene i HANDOVER.md («Begreper»): program / prosjekt /
   arbeidsflate — to personer kan dele ett prosjekt, også med én felles
   arbeidsflate (flere tilkoblinger til samme code-server).

## Feller å huske (betalt én gang)

- PowerShell/UTF-8: aldri pipe æøå gjennom PowerShell til ssh — men
  agenter jobber uansett direkte på raven (norm «cursor»).
- PAT-en på raven (`~/.config/github/token`) dekker IKKE KODE15AS —
  repo-automatikk går via vaktmester-appen når den finnes.
- Workspace-containere deler aldri docker-nett med styrende tjenester.
- API-nøkler bor i proxyen, aldri i workspace-containere.
- Pin alle versjoner (basisimage, utvidelse); oppgrader kun bevisst.
