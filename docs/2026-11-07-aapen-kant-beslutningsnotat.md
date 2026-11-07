# Beslutningsnotat: åpen kant for kode15.studio (07.11.2026)

Etappe 1 i chat 4 (`docs/2026-11-07-plan-chat-4.md`). Erstatter den
først foreslåtte «trusselmodellen» — rammen ble korrigert av Jørn
samme dag: raven ER webserveren, FortiGate er kontrollpunktet, og
risikobildet skal være nøkternt, ikke paranoid. Vi er et lite
oppstartsfirma med full kontroll over egen infrastruktur — det er en
fordel, ikke et problem.

## Faktagrunnlag (verifisert på raven 07.11)

- Raven har ingen offentlig IP. Tailscale Funnel på 443 eksponerer
  allerede kodelader (`/`) og deploi-dns-hook — andre prosjekter,
  urørt av dette notatet.
- Caddy for Light lytter kun på tailnettet (100.65.19.39) og
  kontor-wifi (10.10.0.33) + kablet LAN (10.5.0.22). Coturn lytter på
  wifi + tailnett, ikke offentlig.
- Arbeidsflatene har fri utgående internett (npm m.m.), når ikke
  lobbyen (nettskille verifisert i konfetti-testen), og har ikke
  Docker-socketen (kun lobbyen har den).
- LLM-nøkler bor i proxy-leddet på raven, aldri i flatene.
- WordPress for kode15.no (`KODE15AS/wordpress-kode15-no`) kjører
  allerede offentlig på raven via erbium.no (Caddy-ingress i elduro).
  Det offentlige sporet eksisterer altså i dag.

## To risikodomener

Påstanden «et lite firma uten innlogging tiltrekker seg ikke
målrettede angrep» gjelder bare halve bildet. Riktig deling:

**kode15.no — WordPress + (planlagt) WooCommerce.** Har innlogging
(wp-admin) og får kundedata og betaling. WordPress er den mest
automatisk angrepne programvaren på internett. Svaret er standard,
kjent herding, og det bor i wordpress-drengen: auto-oppdatering av
core/plugins, 2FA på wp-admin, vurder å begrense wp-admin til
tailnettet i Caddy, få og velprøvde plugins. GDPR løses der dataene
bor: WordPress/WooCommerce har innebygd eksport og sletting per
person, og regnskapsdata bor i Fiken (selv databehandler).

**kode15.studio — plattformen.** Ingen innlogging, ingen
brukerdatabase, ingen persondata. Identitet er signerte
invitasjonslenker per prosjektgruppe. Den sterkeste GDPR-posisjonen
er å ikke ha dataene — det vedtaket står. Trusselbildet er
internett-bakgrunnsstøy (skannere mot åpen 443) pluss det ene som er
spesifikt for oss: en lekket invitasjonslenke som brenner LLM-budsjett
eller bruker en arbeidsflate som gratis regnekraft.

## Arkitektur: raven er webserveren

- **Kanten er FortiGate → caddy på raven.** Eksponeringen gjøres som
  FortiGate-bestilling etter norm «bestilling» (committet ordre,
  godkjenning i web-UI) — en eksplisitt, versjonert beslutning.
  Fortigate-drengen har ordreflyten.
- **DNS:** `kode15.studio` pekes på den offentlige IP-en når ordren er
  godkjent. Sertifikat via samme DNS-01-flyt som i dag (bevist).
  `studio15.cloud` redirecter til nytt navn i måneder.
- **Utbyggingstrapp, ikke kantflytting:** raven alene først; deretter
  container swarm med flere servere når behovet er målt. Høstingen
  (etappe 3) skal ikke bake sesjonsstate, presence eller volumer fast
  i «én maskin» på en måte som gjør swarm-steget dyrt — men swarm
  designes ikke nå.
- **Hybrid LLM:** lokale RTX 5080/5090 ved siden av Opper. Lokal
  servering trenger et OpenAI-kompatibelt endepunkt (f.eks. vLLM) som
  Zoo Code-moduser kan peke på per modell. Plassen reserveres i
  arkitekturen; designes når GPU-ene tas i bruk.
- **TURN:** coturn får offentlig lytter via samme FortiGate-regime når
  fjerndeltagere trenger det. I HIØ-rommet går WebRTC direkte på
  rommets LAN.

## Produktegenskaper (ikke paranoia)

1. **Signerte invitasjonslenker per prosjektgruppe.** HMAC, samme
   stateless mønster som `flate_hemmelighet` i dagens lobby. Lenken
   bærer gruppe + utløp; lobbyen verifiserer uten database.
   Revokering = roter gruppens nøkkel.
2. **Kill-switch per gruppe.** Én bryter i registeret: lobbyen avviser
   gruppen, containerne stoppes, invitasjonsnøkkelen roteres. Grovere
   nødbrems finnes alltid i FortiGate (policyen av).
3. **Budsjett-tak per gruppe via Opper.** Forbruks-merkelapper per
   kall og programmatisk nøkkelforvaltning er innebygd — tak blir
   konfigurasjon. Nådd tak varsler og kan utløse kill-switchen.
4. **Hygiene i flatene:** CPU/minne/pids-grenser per container og et
   enkelt egress-tak (rate, ikke allowlist — npm skal virke). Vanlig
   husholdning, ikke frykt.

## Innlogging: utredes som kjøpsflyt, ikke brukerdatabase

Betalende kunder trenger identitet — men den bor i
WordPress/WooCommerce (som har kontoer, GDPR-verktøy og betalingsflyt),
ikke i studio. Modellen som utredes: kjøp/tildeling i WooCommerce
utsteder en signert gruppenøkkel/invitasjonslenke til kode15.studio.
Fakturaflyt WooCommerce → Fiken via regnskaps-drengen. Studio lagrer
fortsatt null persondata. Utredningen bor i wordpress-drengen;
studio-siden trenger bare et endepunkt som utsteder lenker mot
gruppenøkkel.

## Beslutningspunkter til Jørn

1. **Kanten:** bekreft FortiGate-port-forward til caddy på raven som
   eneste offentlige inngang for kode15.studio (ingen Funnel for
   produktet).
2. **Invitasjonslevetid:** forslag 14 dager standard, fornybar av
   gruppeeier.
3. **LLM-budsjett:** forslag at tak per gruppe settes ved opprettelse
   med et fornuftig standardnivå (tall når Opper-kontoen er oppe).
4. **Innloggingsmodellen:** bekreft WooCommerce-kjøp → signert
   gruppenøkkel som utredningsretning (utredes i wordpress-drengen).
5. **HUD-budsjettmål:** forslag tastetrykk→skjerm < 50 ms i rommet,
   < 120 ms hjemmefra — fryses først når HUD-en måler.
