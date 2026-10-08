# 2026-10-08 — Forensik: «låste» flerspill-spill, og vernet mot dem

## Symptomene (Jørns test, trinn 2)

- Klikk på opptatt rute «låste spillet» (bondesjakk, Lenovo 1 mot
  102CAD 1).
- Begge spillere så «Venter på <den andre>» samtidig — hver skjerm sin
  egen virkelighet (også i prosjekt nr 3, 112CAD 1 mot Lenovo 1).
- Agent/skjerm byttet om identiteter (112CAD 1 vist som Lenovo 1).

## Hva forensikken viste

1. **Spillogikken var korrekt hele tiden.** Både klient og tjener
   avviser opptatt rute og trekk utenfor tur; full spillsekvens med to
   nettleserfaner + tjenerlogging bekreftet at trekk, broadcasts og
   avvisninger virker feilfritt (ingen trekk-hendelse sendes engang ved
   klikk på opptatt rute).
2. **Den egentlige feilen: WebSocket-forbindelser som dør stille.**
   Vite-klienten kobler til én gang ved sidelast; feiler det (eller blir
   forbindelsen halvdød — mottar, men får ikke sendt), finnes INGEN
   selvhelbredelse. Siden ser normal ut, men er frosset/enveis. Uten
   tilbakemelding ved klikk ser det ut som «låst spill». Reprodusert med
   en fane som var beviselig desynket i minutter uten tilhørende
   TCP-WebSocket i caddy-loggen, mens en manuell WebSocket fra samme
   side koblet på 27 ms (8/8 vellykkede håndtrykk i stabil tilstand).
3. **Identitetsbyttet**: spillenes kode mapper riktig
   (`?spiller=<navn>` → egen rolle, naken URL → gjest). Byttet oppstår
   når noen åpner en naken adresse (f.eks. eieren åpner
   invitasjonslenken selv) — da antas gjesterollen.

## Vernet (tre lag, utrullet 08.10 kveld)

1. **Caddy: HTTP/3 skrudd av** (`servers { protocols h1 h2 }`).
   Alt-Svc-annonsen ga Chrome en alternativ QUIC-vei som er
   hovedmistenkt for de halvdøde forbindelsene (de friske lastene gikk
   h2/h1; de døde etterlot ingen loggspor). h3 gir oss ingenting på
   LAN/tailnett.
2. **Vaktbikkje i spillene** (begge levende spill patchet: bondesjakk
   nr 2 Lenovo, pengebinge nr 3 112CAD): klienten sender hent hvert
   3. sekund og kjører `location.reload()` hvis det ikke kom svar på
   10 sekunder mens siden er synlig. Døde/halvdøde sider helbreder seg
   selv. Verifisert: ingen falske reload-er på frisk side.
3. **Tilbakemelding ved ugyldig klikk** (bondesjakk): «🙅 Den ruten er
   opptatt — velg en ledig rute!» / «⏳ Vent litt — det er X sin tur!»
   (pengebinge har knapper med disabled-tilstand og trenger det ikke).

## Malen (`lobby/prosjektmal/nybegynner/AGENTS.md`, lobby rebygd)

- Vaktbikkje PÅBUDT i alle flerspill (mønster med window-globaler så
  HMR ikke stabler intervaller).
- Ugyldige klikk skal alltid gi vennlig melding; tjeneren skal uansett
  avvise selv.
- Invitasjonslenken skal alltid bære gjestens navn
  (`?spiller=<gjest, URL-kodet>`), og agenten skal presisere at lenken
  er til gjesten — eieren ser websiden til høyre og åpner den aldri
  selv. AGENTS.md hot-kopiert inn i de tre levende prosjektene.

## Småplukk / gjenstår

- MIDLERTIDIG tjenerlogging ligger i `vite.config.js` i
  bondesjakk-containeren (nr 2 Lenovo) — `[bondesjakk]`-linjer i docker
  logs. Nyttig for kveldens testing; fjernes når testingen er ferdig.
- Jørns kommentar: «Ctrl F5 nullstilte Editor på min PC men ikke på
  tavla» — tavlas iframe lever sitt eget liv og reloades ikke av
  deltagerens Ctrl F5. Ikke rørt i kveld; vurder egen sak (f.eks.
  tavle-reload når flaten bytter innhold).
- Rotårsaken bak selve tilkoblingssvikten er sannsynliggjort (h3), ikke
  100 % bevist — men vaktbikkja gjør spillene robuste uansett transport.
