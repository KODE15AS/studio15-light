# Mennesketest V1+V2 (resept, 2026-10-03)

Alt maskintestbart er GRØNT (chat 2): konfetti-testen med GitHub-variant,
16-punkts presence-test, UI-test av samling og vegg i nettleser, og
vekkesidens watch-modus. Denne resepten dekker det bare mennesker kan
teste. Utstyr: Jørns PC (sete 1), en PC/nettbrett til (sete 2), begge på
studio-wifien. Raven med 70"-skjermen.

Base-URL (velg den som passer enheten):

- Med Tailscale: https://cadify104raven.tail14de1b.ts.net:8100
- Uten Tailscale, på studio-wifien KODE15 (lagt til 04.10 for
  sete 2-nettbrettet): http://10.10.0.22:8100 — skriv adressen i
  nettleserens ADRESSEFELT (ikke søke-/AI-feltet). NB: wifi-en og ravens
  kablede nett er to separate nett — raven står på wifi-en med eget
  wifi-kort (wlo1).

  VIKTIG over http (funn 04.10): Zoo Code-chatten er en webview og krever
  secure context — over http blir panelet BLANKT (verifisert). Engangsfiks
  per enhet i Chrome: `chrome://flags` → «Insecure origins treated as
  secure» → Enabled med verdien `http://10.10.0.22:8100` → Relaunch. Da
  virker også clipboard-API-et. Varig løsning (https med gyldig sertifikat
  på wifi-inngangen via DNS-navn) hører til neste etappe, sammen med
  Funnel-vurderingen.

## 1. Konfetti fra begge seter (sete 2 gjenstår fra V1!)

1. Sete 2: åpne lobbyen → programmet «Demo» → nytt prosjekt «Konfetti 2».
2. Sete 2: skriv deltagernavn, «Åpne min arbeidsflate» — samlingen åpnes
   med editor + webside. Vent ut første oppstart (~1 min).
3. Sete 2: i Zoo-chatten: «Legg konfetti på siden» → endringen skal dukke
   opp levende på websiden (høyre tile).
4. Sjekk underveis: gyldig HTTPS (hengelås), ingen innlogging noe sted.

## 2. GitHub-prosjekt (vaktmesteren, ny i chat 2)

1. Sete 1: velg programmet «Saturday test 2» (har org) → nytt prosjekt.
2. Verifiser på github.com at det private repoet dukket opp i
   `KODE15-saturday-test-2`, seedet fra malen.
3. Åpne arbeidsflaten, gjør en endring, vent ~2 min (autosave-push) —
   verifiser commiten på GitHub.
4. Slett prosjektet fra lobbyen (skriv sluggen) — verifiser at repoet er
   borte fra GitHub.

## 3. Samtidig arbeid i samme prosjekt

1. Begge seter: hver sin arbeidsflate i SAMME prosjekt (f.eks. «Demo»).
2. Begge redigerer hver sine filer samtidig; autosave-push hvert 2. min —
   verifiser at begge endringene ender i repoet (ingen tapte skrivinger
   ved vanlig arbeidsdeling).
3. Felles arbeidsflate: sete 2 åpner sete 1s editor-URL direkte (eller
   flatevelgeren i samlingen + «Ta over») — begge skriver i samme fil.

## 4. Se / peke / ta over (V2)

1. Begge seter åpner samlingen (`Samling`-lenken i lobbyen).
2. Sete 2: velg sete 1s flate i flatevelgeren. Beveg musen over tilen —
   sete 1 skal se sete 2s fargede ghost-cursor med navn (over både editor
   og webside). PEK på noe konkret («den knappen der») — treffer pekeren?
3. Sete 2: «Ta over» (eller dobbeltklikk på tilen) → rammen skifter til
   sete 2s farge, badge «<navn> kontrollerer» hos begge + på veggen.
   Sete 2 redigerer i sete 1s editor. Eieren (sete 1) skal kunne jobbe
   SAMTIDIG — begge har kontroll.
4. Sete 1: prøv «Ta over» på sete 2s flate mens sete 2 kontrollerer
   sete 1s — skal gå fint (uavhengige flater). Prøv deretter å ta en flate
   noen allerede kontrollerer fra et tredje vindu — skal avvises med navn.
5. «Slipp (Esc)» → badge og ramme borte hos alle.
6. Reconnect: last samlingssiden på nytt midt i en økt — samme farge og
   identitet skal komme tilbake (session-token), ingen duplikat i rosteren.

## 5. Veggen (fysisk, på Ravens desktop — IKKE over SSH)

1. På Ravens desktop: `cd ~/dev/studio15-light/vegg && ./installer-vegg.sh`
   (svarer «j» til å flytte Skjermsamlings wall-watcher til side).
2. `./vegg-kiosk.sh &` fra desktop-terminalen (eller logg ut/inn).
3. Verifiser: kiosken åpner /vegg i fullskjerm på 70"-en, begge halvdeler
   med editor + levende webside, eierfarger, roster øverst.
4. Ghost-cursors og «kontrollerer»-badges skal være synlige på veggen når
   setene peker/tar over (pkt. 4).
5. Veggen er read-only: Ravens mus/tastatur skal ikke kunne påvirke noe
   (watch-modus joiner aldri).
6. Stopp stacken (`docker compose stop caddy`) → kiosken skal lukke seg
   innen ~10 s. Start igjen (`docker compose start caddy`) → kiosken
   kommer tilbake av seg selv.

## 6. Dvale/vekke i praksis

1. La en arbeidsflate stå urørt til dvale (45 min — eller test med
   `IDLE_MINUTES=1 docker compose up -d lobby`, husk å sette tilbake).
2. Veggen skal vise «Arbeidsflaten sover» UTEN å vekke den (ny i chat 2).
3. Deltageren åpner sin flate → vekkesiden starter den automatisk,
   oppe på ~10 s.

## 7. Terminal-autokjøring av/på

1. I arbeidsflaten: Zoo Code-innstillinger → verifiser at autokjøring av
   terminalkommandoer er AV som standard, og at det er enkelt å slå på.
2. Slå på, kjør noe, reload siden — innstillingen skal overleve (kun
   gjenskapt container nullstiller).

## Etterarbeid når alt er grønt

- Kryss av punktene i `HANDOVER.md` (sete 2-punktet under V1 og de to
  siste under V2).
- Rydd eventuelle testprosjekter.
- Husk (fra handover 1→2): slett Dropbox-lenkene til pem-nøkkelen
  (eller rotér nøkkelen), og de gamle Studio 15-testorgene venter på
  manuell sletting.
