# Handover chat 3 → 4 (07.11.2026)

Chat 3 gikk fra 03.10 til 07.11 — mye lenger enn planlagt, fordi samme
Cursor-chat fortsatte etter midtveis-dokumentet
(`2026-10-15-chat-3-midtveis.md`, som dekker 03.10–20.10). Dette
dokumentet dekker **20.10–07.11** og er den reelle handoveren.
**Chat 4 starter med seksjonen «Chat 3 avsluttet — start her» nederst.**

Perioden kulminerte 07.11 i en strategisk pivot: **kode15.studio** —
generell webtilgang, demokratisk tavle, rebranding i nytt repo, og
MCAD som ultimat mål. Beslutningene står i egen seksjon under; chat 4
starter rett på dem.

## Drift 29.10 — raven-reboot (fikset, dokumentert)

Feilmelding på 70-tommeren ved gjenopptatt testing: GNOME-krasj (kjent
mutter-hotplug-bug) + NVIDIA-driver-mismatch (unattended-upgrades hadde
oppgradert userspace uten reboot) → reboot → caddy døde på
boot-rekkefølge-fella (kode15-wifi hadde autoconnect av, 10.10.0.33
fantes ikke da Docker startet). Alt fikset og verifisert; tre lærdommer
i `2026-10-29-reboot-grafikk-caddy.md`.

## Smartenheter og DEFA (29.10–06.11)

- Enhetene funnet og verifisert på kontor-wifi: **2× DEFA Power**
  (10.10.0.16 / 10.10.0.21, Modbus TCP port 502, slave-ID 255), **Mill
  panelovn** (10.10.0.22, HTTP), **Shelly Pro 3EM** (10.10.0.14, kun
  lesing). DHCP-reservasjoner ligger i fortigate-drengens planlagte
  ordre — **fortsatt ikke utført**.
- **Modbus TCP valgt** som DEFA-integrasjonsvei. Fasit og registerkart i
  `kodelader/handover/2026-10-29-defa-power-modbus.md`;
  referanseskript `kodelader/verktoy/les_defa_status.py` (én
  forbindelse + blokklesing 290–306 — laderne straffer
  tilkoblings-churn). **DEFA-integrasjonen eies av kodelader-drengen**;
  her i studio15-light er laderne bare enhetsfakta i malene.
- **Content-filter-fella** (05.11 + gjentatt 06.11): nettverksskanning i
  Zoo-prompter gir tomme modellsvar (`finish_reason: content_filter`) →
  Zoo Code retryer evig (`MODEL_NO_ASSISTANT_MESSAGES`). Botemiddel:
  enhetsfakta + skanneforbud i malenes AGENTS.md (se klassetimen under)
  — og på sikt `enhetsregister`-forslaget som ligger i
  sikkerhetssjef-drengens HANDOVER.md.
- Display-skriving på DEFA: kun via OCPP 2.0.1 CSMS (godkjent backend —
  DEFA må whiteliste vår URL; den saken bor i kodelader-drengen).

## Vaktmester-vakta for prosjektgrupper (06.11, commit 21936fd)

Jørn bommet to ganger på org-registrering (GitHub-URL i navnefeltet,
app ikke installert i org-en). Nå validerer lobbyen FØR lagring:
org-feltet normaliseres (tåler limte URL-er), vaktmesteren sjekker
ferskt at appen er installert i org-en (404 → feilmelding med
installasjonslenke), og navnefeltet avviser URL-er. Frontenden har eget
org-felt og klikkbar installasjonslenke i veiledningen. E2E-testet;
`Studio-DEFA`-org-en satt opp riktig med denne flyten.

## Klassetime 1 med Øystein (06.11) — tre funn, tre fikser

Første «skoleklasse» (Jørn + Øystein, Studio DEFA-gruppen). Alle funn
fikset samme dag, alt pushet:

1. **Filter-løkka slo til igjen:** «finn enheter på wifi» ga
   content-filter-løkke i BEGGE nye ekspert-prosjekter — fordi
   smartenhets-avsnittet fra 05.11 bare lå i pretest-prosjektet, ikke i
   malen. Fiks: avsnittet inn i begge prosjektene + permanent i
   `STUDIO15-Maler/mal-ekspert` (commit `6b814a1`). **Lærdom (viktig):
   maler er fasit — en fiks i et enkeltprosjekt SKAL samtidig inn i
   malen, ellers arver neste prosjekt feilen.**
2. **Flerspill-identiteten knakk i nybegynner-malen** (trinn 2,
   tre-på-rad): spillet viste feil navn og «ingen». Rotårsak:
   regelfila i flata presenterte deltageren som SLUG («oeystein») mens
   `?spiller=`-parameteren og medspiller-API-et bruker VISNINGSNAVN
   («Øystein») — pluss skjør navnelogikk i spillet og en agent som
   maste «har Jørn åpnet lenken?» uten å sjekke tilkoblinger selv.
   Plattformen ble frikjent ved live-test (iframe får `?spiller=`
   korrekt). Fiks i tre lag (commits `d855bf2` + `1bc31c1`):
   - Lobbyen sender `DELTAGER_NAVN` (visningsnavn) til flatene.
   - `01-webside.md` i nye flater bærer visningsnavnet og forklarer
     `?spiller=`-mekanikken eksplisitt.
   - `mal-nybegynner` herdet: sammenlign kun visningsnavn, vis
     navnevelger når parameter mangler (aldri «ingen»), og **se i
     tilkoblingsloggen før du spør** deltageren.
   E2E-verifisert med prøvedeltager («Prøvekanin Æøå» — æøå intakt);
   testavfall slettet. NB: gjelder NYE prosjekter/flater — gamle har
   gammel mal.
3. **DEFA 2-simulatortesten:** bilsimulator + installatør-app ga null i
   alle dynamiske Modbus-registre. Grundig feilsøkt fra raven:
   kommunikasjonen frikjent (lesing OK, skriving OK, elektrikerens
   innstillinger bekreftet: eMS 6 A / fallback 6 A / timeout 100 s);
   DEFAs dokumentasjon bekrefter at 0 er default-verdi. Konklusjon:
   **flatt batteri i bilsimulatoren** — laderen fikk aldri CP-signal.
   Retest gjenstår med ladet batteri; lesesløyfe-oppskriften ligger i
   kodelader-skriptet, og registrene skal vise status ≠ 0 (f.eks.
   0x4332 = «C2») i det CP-signalet kommer.

## STRATEGIDAGEN 07.11 — pivoten til kode15.studio

Jørn la frem fire tema + to strategidokumenter (MCAD). Beslutningene
(Jørns kommentardokument, 07.11) og vurderingene er fasit for chat 4:

### Besluttet (fryses)

- **Pivot:** domenet `kode15.studio` er registrert. KODE15.studio samler
  erfaringen fra skjermsamling + Studio 15 + Light, generaliserer
  brukerprofilen, beholder utdanningsfokus. Dette REVERSERER
  grunnlagsvalget «ingen offentlig eksponering» — bevisst, med egen
  trusselmodell som første oppgave i chat 4.
- **Forretningsmodell:** nivåer fra subsidiert (nye interessenter) til
  betalt (fritt modellvalg). **Opper** blir modell-gateway (GDPR/
  skoleverk). Verifisert 07.11: Opper er OpenAI-kompatibel
  (`api.opper.ai/v3/compat`) — Zoo Code kan peke direkte på den;
  LiteLLM-sporet avsluttes. Se `docs/2026-11-07-mcad-konklusjon.md`.
- **Identitet uten innlogging består:** signerte invitasjonslenker per
  prosjektgruppe — ingen brukerdatabase.
- **Utrullingstrapp:** først fremmed wifi (HIØ Halden/Remmen-scenarioet:
  4 studenter + 70-tommer i samme rom), deretter åpen web (elev
  hjemmefra).
- **«Bli tavle»-knappen bygges tidlig:** enhver nettleser i fullskjerm
  blir tavle. Beste-vei i Chromium: Window Management API
  (`getScreenDetails()` + `window.open` med skjermkoordinater +
  fullskjerm-delegering til barnevinduet — vennens oppskrift, vurdert
  realiserbar; krever HTTPS + ett samtykke, KUN Chromium). Fallback:
  åpne URL + F11. HIØ-rommet er latency-gunstig: WebRTC går direkte på
  rommets LAN; bare presence/signalering trenger lobbyen.
- **Flere deltagere på samme prosjekt av-parkeres** (beslutning 08.10
  reverseres): 4 studenter skal kunne jobbe samtidig. Krever designnotat
  (kortnavn-skjemaet bærer én deltager, git-identitet, dvale-regnskap,
  presence-regi) FØR koding.
- **Rebranding = nytt repo, høsting ikke nyskriving:** Light bærer 22
  testfunn og en måneds lærepenger — modne komponenter porteres bevisst
  inn i nytt repo med ren historikk, resten legges igjen.
- **Org-struktur:** GitHub har IKKE sub-organisasjoner. Anbefalt (ikke
  endelig besluttet): **én org (`KODE15studio`), kjernerepo `studio`**,
  prosjektgrupper som prefiks/team i samme org — vaktmesteren
  installeres da ÉN gang (fjerner hele install-bommen fra 06.11 by
  design). Deltagerne har ikke GitHub-kontoer, så isolasjonsargumentet
  for separate org-er er svakt.
- **Latency-prinsippet fredes:** dagens gode latency skyldes DOM-ikke-
  piksler + LAN. Måle-HUD bygges FØR arkitekturendringer; budsjettmål
  (forslag: tastetrykk→skjerm < 50 ms i rommet, < 120 ms remote); kant
  i Norge ved webåpning; TURN offentlig kun som fallback.

### MCAD — det ultimate målet

Jørns to dokumenter (kommentarer + critical friends) konkluderer: **Zoo
Code som agentramme, Opper som modellag, KCL/AST som kildekode
(HW-as-SW), Product Domain-modellen som autoritet, deterministiske
verktøy som dommer.** Full konklusjon med verifisert Opper-funn og
frosne designprinsipper: `docs/2026-11-07-mcad-konklusjon.md`.
Neste steg der: **Proof of Architecture** (Zoo Code + Opper + KCL/AST +
Zoo Engine + Product Domain + assembly-solver) — bor naturlig som egen
drengr i den nye org-en. Plattform og MCAD-applikasjon er to produkter:
MCAD blir på sikt en prosjektmal («mcad-mal») på plattformen, samme
mekanikk som nybegynner/ekspert.

## Venter / småplukk (arv til chat 4)

- **Slette «slettmeg»-prosjekter:** `nybegynner/oeystein-defa-6-november`,
  `nybegynner/joern-6-nov-slettmeg`, `studio-defa/joern-6-nov-sletmeg`,
  `ekspert/prosjekt-defa-kaldstart-pretest-slettes` (referansekoden
  derfra er berget til kodelader).
- **DEFA 2-retest** med ladet simulatorbatteri (kodelader-drengen eier,
  men testes gjerne herfra).
- **Zoo 3.88-oppgraderingen** — fortsatt bevisst utsatt (pinnet 3.87).
- **FortiGate DHCP-reservasjonsordren** (DEFA 1+2, Mill, Shelly, raven)
  — planlagt, ikke utført.
- **Tavle-Pi** (bestilt 07.10) — settes opp når delene er her; NB:
  «Bli tavle»-arbeidet kan endre/forenkle Pi-ens rolle.
- **Nybegynner trinn 2 på nytt** i et FERSKT prosjekt (verifisere
  mal-herdingen i praksis).

## Chat 3 avsluttet — start her (chat 4)

Rekkefølgen er bevisst — beslutningsnotatene først, så bygging:

1. **Les:** `HANDOVER.md`, dette dokumentet og
   `docs/2026-11-07-mcad-konklusjon.md`. Jørns to strategidokumenter
   (07.11) er oppsummert i seksjonene over; originalene i Jørns Dropbox.
2. **Trusselmodell/beslutningsnotat for åpen kant** (nivå «inviterte
   grupper»): misbruksflater (vilkårlig kodekjøring, LLM-kost, egress),
   invitasjonslenke-design, kill-switch per gruppe, kant-arkitektur
   (VPS i Norge / tunnel) med backhaul til raven.
3. **Designnotat: flere deltagere på samme prosjekt** (av-parkert
   08.10-beslutning) — kortnavn, git-identitet, dvale, presence.
4. **Repo-fødselen:** opprett org + kjernerepo (anbefaling over; Jørn
   bekrefter navnene), skriv høstingsplanen (hva porteres, hva legges
   igjen), Handover 0 i nytt repo. Vurder om chat 4 selv bor i det nye
   repoet etter fødselen.
5. **DNS:** `kode15.studio` inn i deploi-dns (GitOps), sertifikat via
   samme DNS-01-flyt, dobbeltkjøring med studio15.cloud (gammel
   redirecter til ny — i måneder).
6. **«Bli tavle»-knappen** + offentlig nåbar TURN (coturn lytter i dag
   kun på kontor-wifi + tailnett) + fjerne raven-særbehandlingen i
   dvale-vakta.
7. **Måle-HUD for latency** (tastetrykk→skjerm, WebRTC-vei,
   presence-RTT) — FØR kantendringer, så degradering måles i tall.
8. Småplukk-listen over etter kapasitet.
