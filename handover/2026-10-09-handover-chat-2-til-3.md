# 2026-10-09 — Handover chat 2 → chat 3

Chat 2 sier seg ferdig med bondesjakk/nybegynner-testingen (Jørn 09.10).
Chat 3 skal lage **prosjektmal for mer profesjonelle oppgaver**.

## Status ved overlevering

- Nybegynnerløpet er gjennomtestet over flere dager (trinn 1–2 med ekte
  flerspill på tvers av maskiner). Alle funn fikset og utrullet — siste
  runde: `2026-10-08-flerspill-laaste-spill.md` (låste spill var døde
  WebSockets, ikke spillogikk; vern i tre lag er på plass).
- Testprosjektene (Lenovo 1 nr 2, 112CAD 1 nybegynner 2, 102CAD 1 nr 2)
  er FERDIGTESTET og kan slettes av Jørn fra startsiden når som helst.
  All midlertidig feilsøking er ryddet ut av containerne.
- Alt er committet og pushet i dette repoet (git log 08–09.10).

## Det chat 3 trenger å vite om prosjektmaler

- Malene bor i `lobby/prosjektmal/<navn>/` — i dag `nybegynner` og
  `full`. En ny mal = ny mappe her.
- Malen bakes inn i lobby-imaget (`lobby/Dockerfile`:
  `COPY prosjektmal /opt/prosjektmal`). Utrulling:
  `docker compose build lobby && docker compose up -d lobby`.
- Malvalget lagres per prosjekt i `register/programmer.yaml`
  (`mal: nybegynner`); lobbyen tilbyr valget ved prosjektoppretting.
- `nybegynner`-malen er referansen for virkemidlene: `AGENTS.md`
  (agentens regi: trinnstyring, invitasjonsregi, flerspill-normer),
  `spilleplan.yaml` (todo-trinnene agenten driver), pluss
  Svelte 5 + TS + Vite-grunnoppsettet. `full`-malen er det rå
  grunnoppsettet uten pedagogisk regi.
- Normer fra 08.10 som bør gjelde ALLE maler med flerspill/websider:
  vaktbikkje (hent hvert 3. s + selv-reload etter 10 s stillhet),
  tilbakemelding ved ugyldige klikk, identitet via `?spiller=`-param
  (aldri spørre om navn), invitasjonslenker med gjestens navn. Se
  flerspill-seksjonen i `nybegynner/AGENTS.md` — gjenbruk ordlyden.
- Per-flate-regler genereres av lobbyen i `.roo/rules/01-webside.md`
  (adresser, deltagernavn, medspiller-kommandoen) — uavhengig av mal.

## Åpne punkter (ikke blokkerende)

- Tavla reloades ikke når en deltager Ctrl F5-er sin skjerm (Jørn
  08.10) — tavlas iframe lever sitt eget liv. Egen sak.
- Rotårsaken bak de døde WebSocketene er sannsynliggjort (HTTP/3-veien,
  nå avskrudd i caddy), ikke 100 % bevist. Vaktbikkja gjør spillene
  robuste uansett.
