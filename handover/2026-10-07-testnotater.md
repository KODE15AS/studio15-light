# 2026-10-07 — Jørns fortløpende testnotater (kveld)

Nytt nybegynner-prosjekt med PC13 + Cadify102 13 (to maskiner).
Punktene kom fortløpende i chatten og ble fikset og utrullet én for én.
Prosjektet slettes etter økten; ny test i morgen med ferske deltagere.

| # | Funn | Fix (commit) |
|---|------|--------------|
| 1 | Starttekst på websiden viste gammel layout («feltet under websiden») | Ny tekst: «Snakk med hjelperen til venstre …» (`dc2e2b3`) |
| 2 | Hjelperen limte inn webadressen ved testing — unødvendig med live webside til høyre | Regel: henvis til «websiden til høyre», URL deles KUN når spiller 2 skal bli med fra annen maskin (`ff8b7bf`) |
| 3 | Svarknappene manglet fremdrift — ingen «ta oss videre»-knapp | Obligatorisk fremdriftsknapp FØRST i forslagene («Trinn 1 er fullført — ta oss til trinn 2!») = bekreftelse + avkryssing + neste trinn (`39c18b5`) |
| 4 | Trinn 2 må opplyse at det kreves minst to deltagere | Inn i spilleplan-teksten OG hjelper-regel: informer først, bygg gjerne, men test/avkryss aldri før spiller 2 er med (`466afd2`) |
| 5 | Deltager 2 fikk delt editor med smal Zoo-fane + «Chat»-kolonne | Rotårsak: VS Codes SEKUNDÆRE sidestolpe åpner seg i ferske nettlesere; oppstart-utvidelsen lukket bare primær+panel. Lukker nå alle (`7964b11`) |
| 6 | Run/Deny-dialog for `npm install` forvirrer nybegynnere | `alwaysAllowExecute` + åpen kommandoliste i nybegynner-malen (trygt: innelåst container, skjult terminal). NB: zoo-importfila skrives nå ved HVER oppstart i nybegynner — innstillingsfikser rulles ut med container-restart (`13389d7`) |
| 8 | Spillet rotet med tur-identitet (farger + feil navn) | Deltagernavnet skrives i `.roo/rules/01-webside.md` (fra PARTICIPANT) + regel: navn er identiteten i turindikator/poeng, farger kun støtte, spør alltid om medspillernavn (`0be1f1d`) |

(Punkt 7 finnes ikke i notatene — Jørn hoppet fra 6 til 8.)

## Utrullingsmønster ved mal-/instruksfikser (brukt hele kvelden)

- `lobby/prosjektmal/nybegynner/*` → bak inn i lobby-imaget
  (`docker compose build lobby && up -d`) for NYE prosjekter.
- Kjørende flater: `docker cp` av AGENTS.md/spilleplan rett inn i
  containeren — hjelperen leser instruksene på nytt ved hver melding.
- Zoo-innstillinger i kjørende flater: skriv `/home/coder/zoo-settings.json`
  i containeren og be deltageren laste siden på nytt (importen kjører ved
  hver aktivering i nybegynner-malen).
- `arbeidsflate/*` (entrypoint/oppstart-utvidelse) → bygg
  `studio15-light-arbeidsflate:v2` på nytt for NYE flater; kjørende
  flater kan hot-patches med docker cp + restart.

## Tidligere samme dag

- Tavle-PC (Raspberry Pi 5) bestilt — `docs/tavle-pi-spec.md`;
  omkobling utsatt til chat 3.
- GNOME-krasjskjermen ved HDMI-omkobling varig fikset —
  `2026-10-07-gnome-krasjskjerm.md`.
