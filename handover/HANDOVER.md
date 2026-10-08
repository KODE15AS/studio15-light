# HANDOVER — Studio 15 LIGHT

Levende spesifikasjon (norm «handover»). Punktene krysses av etter hvert som
de leveres. Chat 0 startet 2026-10-03. Chat 1 leverte V1 samme dag. Chat 2
(samme dag) fullførte V1 (vaktmester-integrasjonen) og bygget V2 maskintestet
grønt — gjenstår kun mennesketestene, resept i
`2026-10-03-mennesketest-v2.md`. Natt til 05.10 leverte chat 2 endringene
fra Jørns testrapport — se `2026-10-04-nattarbeid-testrapport.md`
(navneskifter, startside.studio15.cloud, deltagerregister, brødsmuler,
sync-fiksen på tavla). 05.10 formiddag: testrapport 1 og 2 besvart — se
`2026-10-05-testrapport-1.md` (WebRTC-piloten «Del til tavla» + coturn,
tavle-layout speiler deltagerskjermen, zombie-vekking fikset,
template-designnotat). 05.10 midt på dagen: testrapport 3 — se
`2026-10-05-testrapport-3.md` (prosjektmaler full/nybegynner,
nybegynnerskjerm med spilleplan i 4 trinn, ny org KODE15AS-Nybegynner —
app-installasjonen bekreftet i orden 05.10 ettermiddag). 05.10
ettermiddag: testrapport 4 — se `2026-10-05-testrapport-4.md`
(deltagervelger med kontekst og lukking, én-klikks prosjektsletting,
nybegynnerskjerm i lys KODE15-webprofil med ryddet Zoo-chat på norsk,
agent-dialogen alltid synlig, tavla: tiling-assistant var rotårsaken +
selvhelende fullskjerm-vakt). 05.10 kveld: nybegynner-iterasjon 2 — se
`2026-10-05-nybegynner-iterasjon-2.md` (to kolonner ⅓ chat / ⅔ webside
med dragbart skille, hjelperen eier spilleplanen med agentisk todo-liste
og klikkbare svarforslag, stort promptfelt via minRows-patch, +15 %
skrift, varsler skjult, tavla dynamisk 1/2/2×2; nettbrett utgått).
06.10 morgen: testrapport 1 — se `2026-10-06-testrapport-1.md`
(slette deltager med én bekreftelse, dialog-speil på tavla i stedet for
tom editor-økt, webrtc-blinket hvert 5. s fikset, kablet-nett-analysen:
Fortigate-policy er veien — løst samme kveld med hosts + statisk rute
på de tre PC-ene). Samme morgen:
`docs/pedagogiske-mekanismer-i-zoo.md` — kartlegging av Zoo-mekanismene
for pedagogiske oppgavepakker (.roomodes, skills, commands, MCP).
07.10: **TAVLE-PC bestilt** — tavla flyttes fra ravens HDMI til egen
Raspberry Pi 5 (8 GB, Argon ONE V5, High Endurance-SD) bak 70-tommeren
i klasserommet; raven beholder skjermen sin og av/på-bryter-behovet
bortfaller. Spec, design-review (ingen pikselstreaming — Pi-en rendrer
webappen selv) og ordre: `docs/tavle-pi-spec.md`. Oppsett gjøres når
delene ankommer (Ubuntu Server 24.04, X11-kiosk, vakt som systemd).
07.10 kveld: «Oh no! Something has gone wrong» på tavla ved HDMI-
omkobling — rotårsak funnet (mutter-segfault i EDID-lesing ved hotplug)
og varig fikset (gnome-session-failed masket på raven, kontrollert
krasjtestet) — se `2026-10-07-gnome-krasjskjerm.md`. Samme kveld:
to-deltager-test med sju fortløpende funn, alle fikset og utrullet —
se `2026-10-07-testnotater.md` (starttekst, ingen URL-deling på egen
skjerm, fremdriftsknapp i svarforslagene, to-deltager-krav i trinn 2,
sekundær sidestolpe lukkes, kommandoer auto-godkjennes, deltagernavn
som spillidentitet).
08.10: flere deltagere i SAMME prosjekt er PARKERT (Jørns beslutning) —
én deltager per prosjekt er normen; se
`2026-10-08-flerdeltager-parkert.md` (TODO: designe løsning eller
blokkere muligheten; tavla viser i dag ett prosjekt om gangen).

## Grunnlag — les disse først

- «Erfaringsoverforing Studio15 LIGHT.docx» i `~/dev/studio15/` — hele
  rammen, fellene fra Studio 15 og rådene inn i Zoo Code.
- «Erfaringsoverføring: Skjermsamling → Studio 15 Light» (levert 03.10,
  erstatter dokumentet på den umergede grenen i `raven-skjermsamling`) —
  samarbeidslaget: 70"-veggen 50/50, se/peke/ta over, presence-protokollen,
  kiosk-oppsettet (`wall/`), controller-mønsteret (`workspace.rs`, `hub.rs`).

## Målbilde

To personer på hver sin PC over Tailscale (wifi), hver med sin arbeidsflate:
VS Code i nettleseren (code-server) + Zoo Code koblet mot LiteLLM-proxy +
Vite-dev-server som viser websiden levende. Ravens 70"-skjerm viser begge
flatene 50/50. Ingen innlogging, ingen database, ingen offentlig eksponering.

## Begreper (avklart 03.10 — brukes konsekvent)

- **Program** = GitHub-org. Overbygningen som dekker et antall prosjekter
  (erstatter Studio 15s «klasse»).
- **Prosjekt** = container/repo-settet: GitHub-repoet pluss containerne som
  kjører det som bygges (f.eks. Vite-serveren). Persistent, gjenopptakbart,
  slettes som helhet.
- **Arbeidsflate** (kode: workspace) = en editor-container (code-server +
  Zoo Code) koblet til et prosjekt. Opprettes ved behov.
- **To personer kan dele ett prosjekt** på to måter: hver med sin egen
  arbeidsflate mot samme repo (normalen), eller med én **felles
  arbeidsflate** — code-server tillater flere samtidige tilkoblinger til
  samme instans, så begge åpner samme URL og jobber i samme flate. Denne
  egenskapen er også grunnlaget for «ta over» i V2.

I synlig tekst (UI, docs) brukes «arbeidsflate» (norm «språk»);
«workspace» kun i kode og tekniske identifikatorer.

OMDØPT UTAD 04.10 (testrapporten; interne navn i kode/API beholdt —
scope avklart med Jørn): «program» → **prosjektgruppe**, «lobby» →
**startside**, «vegg» → **tavle**, «arbeidsflate» → **skjerm /
deltagerskjerm». Detaljer i `2026-10-04-nattarbeid-testrapport.md`.

## Beslutninger (Jørn, 2026-10-03)

Fra VeloStack-runden (notatet «README about Studio 15 Light med VeloStack»):

- Begrepet «VeloStack» droppes — stacken fra erfaringsoverføringen er fasit.
- Lokalt og enkelt: Tailscale uten ingress, ingen offentlig eksponering.
- NVMe med navngitte volumer er godt nok — ingen tmpfs/RAM-disk.
- Latency-arbeid kun der det merkes (UI-ekko, Vite HMR); mål først,
  optimaliser etterpå. Ingen kunstig kompleksitet (HTTP/3, Unix-socket-MCP).

Fra kap. 10-avklaringene (erfaringsoverføringen):

- **HTTPS via Tailscale-sertifikat** (`tailscale cert`) — secure context for
  clipboard/streaming uten offentlig CA eller ingress.
- **Full tilgang for begge utviklere**: redigering + terminal, men
  terminal-autokjøring AV som standard. Det skal være enkelt og intuitivt
  for brukeren selv å endre dette i Zoo Code-innstillingene.
- **Dvale/vekke-mønsteret** gjenbrukes (idle-reaper: stopp etter N min,
  503-vekkeside med auto-reload, oppe igjen på ~10 s).
- **Felles master-nøkkel** i LLM-proxyen — kostnadsinnsyn per person
  legges eventuelt til senere med to virtuelle nøkler.

Fra skjermsamling-avklaringene (03.10):

- **Veggen viser begge per halvdel**: arbeidsflate (editor + Zoo Code-chat)
  og den levende websiden side om side. Veggen er alltid read-only
  (watch-modus), Ravens tastatur/mus brukes aldri.
- **Prosjektvelger i lobbyen**: ett prosjekt = eget container/repo-sett,
  persistent, startes/gjenopptas med klikk. Lobbyen er også vekkeside for
  dvale/vekke-mønsteret.
- **Sletting er sletting**: ett skript sletter containere, volumer OG
  GitHub-repoet (etter eksplisitt bekreftelse) — ingen dangling repos.
  GitHub-tilgangen løses av vaktmester-appen (se Hovedstruktur under);
  org-sletting er GitHubs ene manuelle unntak.
- Se/peke/ta over videreføres uendret fra Skjermsamling (eierfarge, ghost-
  cursors med navn, én ekstern controller, take/release broadcastes).
- Designgrunnlag: `webprofil-kode15` (norm «web-profil»), med Skjermsamlings
  `app.css` som referanse. Arbeid rett på `main` (norm «tbd»), ikke PR-flyt.

Beslutninger (Jørn, 2026-10-04, under mennesketesten):

- **Wifi-inngang uten Tailscale** (den gang `https://lobby.studio15.cloud`,
  fra 04.10-natten `https://startside.studio15.cloud` — gammelt navn
  redirecter for alltid):
  Tailscale-oppsett per enhet var for tungt for sete 2 (Android-nettbrett).
  Caddy fikk en egen inngang for studio-wifien; lobbyen returnerer relative
  flate-lenker så alle innganger beholder sin origin. Fortsatt ingen
  offentlig eksponering: DNS-navnet peker på PRIVAT IP (ravens wifi-kort).
  Tre feller betalt på veien (alle verifisert 04.10):
  1. Topologi: studio-wifien KODE15 (10.10.0.0/24) og ravens kablede nett
     (10.5.0.0/24) er TO SEPARATE nett med hver sin internettlinje (samme
     ruterboks, ingen ruting imellom) — derfor gikk sete 1 alltid via
     Tailscale-relé. Løst: raven på wifi-en med eget wifi-kort
     (nmcli-tilkobling «kode15-wifi» på wlo1, uten standardrute/DNS —
     kablet nett og Tailscale urørt). Wifi-IP-en er DHCP-tildelt (vurder
     reservasjon i ruteren); endres den må compose OG A-posten i
     deploi-dns oppdateres.
  2. Ren http holdt ikke: Zoo-chatten er webview → krever secure context →
     BLANKT panel. Ekte sertifikat var nødvendig, ikke kjekt-å-ha.
  3. Sertifikat uten offentlig eksponering: A-post `lobby.studio15.cloud →
     10.10.0.22` i deploi-dns (privat IP i offentlig DNS er ok — Telenor-
     resolverne filtrerer ikke) + Let's Encrypt via DNS-01 der TXT-posten
     settes gjennom deploi-dns-GitOps-flyten (skript/deploi-dns01-hook.sh).
     netims navnetjenere er anycast: propageringsSJEKK er ikke nok, fast
     ventetid 300 s (skript/lobby-sertifikat.sh). Fornyelse: cron på raven
     (måned, dag 3, 04:17 → ~/.local/state/lobby-sertifikat.log).
     PUBLIC_BASE peker nå på wifi-inngangen (brukes i Zoo-regelfilene).
- **Funnel vurderes i neste etappe**: Jørn ønsker å se på Tailscale Funnel
  (offentlig eksponering) som alternativ inngang. IKKE avgjort — strider
  mot grunnprinsippet «ingen offentlig eksponering», så det må være en
  eksplisitt beslutning med egen risikovurdering. Merk: raven har allerede
  Funnel på 443 for ANDRE prosjekter (kodelader, deploi-dns-hook) — urørt.

Hovedstruktur (Handover 0-grillingen, 03.10):

- **Program = ekte GitHub-org.** Et program er overbygningen som dekker et
  antall prosjekter (erstatter Studio 15s «klasse»). Valgt med åpne øyne:
  GitHub har ikke API for org-oppretting eller org-sletting på gratisplanen,
  så UI-et blir en guidet flyt med manuelle GitHub-steg (som Studio 15
  løste det), og sletteregimet får et manuelt org-steg til slutt — resten
  automatiseres. (Presisert 03.10: API-et finnes, men kun for
  enterprise-kontoer — GitHub Enterprise Cloud, $21/bruker/mnd. Vurdert og
  avvist som ikke verdt det nå; detaljer i handover 1→2.)
- **Vaktmester-mønsteret gjenopplives**: GitHub App for repo-automatikk i
  program-orgene (lærdommene fra Studio 15 om deploy keys og app-tokens
  gjelder). Erstatter merknaden om ny PAT.
- **Alt bor i git — ingen database.** Programregister som YAML i dette
  repoet, prosjektkode i prosjektrepoene, secrets i `.env` utenfor git,
  kjøretilstand leses fra Docker. Kun temp-filer på raven, slik at en
  container alltid kan gjenoppbygges fra repo.
- **Arbeidsflater opprettes per deltager ved behov** — solo-prosjekter støttes;
  ikke fast to seter.
- **V1 = tynn E2E-skive**: lobby (program + prosjekt) → arbeidsflate med
  Zoo Code → proxy → konfetti-test. Vegg + se/peke/ta over er v2.
- Web-design: `webprofil-kode15` (norm «web-profil») — bekreftet på nytt.

## Åpne punkter (agentarbeid, første byggetrinn)

- [x] Zoo Code-verifisering (chat 1, 03.10 — vsix-en inspisert direkte):
      ID `ZooCodeOrganization.zoo-code`, alle nøkler/kommandoer har prefiks
      `zoo-code` (ikke `roo-cline`); distribusjon Open VSX (verifisert
      utgiver, pinnet 3.87.100557); `.roomodes`-formatet og import-skjemaet
      (providerProfiles/modeApiConfigs) uendret fra Roo; announcement-hacken
      trengs fortsatt (ID `oct-2026-v3.86.0-models-aborts-tool-streaming`);
      ripgrep-fellen er FIKSET oppstrøms (Zoo leter selv i
      ripgrep-universal) — symlinken beholdt som forsikring; ren webapp
      bekreftet → ingen GUI-streaming. NYTT funn: Zoo viser en
      telemetri-dialog ved første oppstart — kveles med
      `telemetrySetting: "disabled"` i settings-malen.
- [x] Se/peke-laget over iframes (V2) — AVKLART og BYGGET (chat 2, 03.10):
      «ta over» realiseres med code-servers flerbrukertilkobling (to
      tilkoblinger til samme /w/<flate>/-URL har begge full kontroll —
      maskinelt verifisert i nettleser, endring fra tilkobling B landet på
      disk og var umiddelbart synlig i A). Ingen input-streaming. Presence-
      laget broadcaster bare tilstand (hvem ser/peker/kontrollerer). Bonus:
      code-server fungerer fint i same-origin-iframe.

## Leveranser

Gjort:

- [x] Repo opprettet på GitHub (KODE15AS/studio15-light) og klonet til
      `~/dev/studio15-light` på raven.
- [x] Stack på plass: compose + plassholder-container på
      http://100.65.19.39:8100 (kun Tailscale).
- [x] Alle avklaringer tatt med Jørn (se Beslutninger over).

V1 — tynn E2E-skive (trinnvis, hvert trinn E2E-verifisert maskinelt;
alt under levert av chat 1, 03.10):

- [x] Zoo Code-verifisering (se Åpne punkter) — først, alt annet avhenger
      av den.
- [x] Arbeidsflate-image (`arbeidsflate/`): code-server 4.140.0-debian
      (pinnet) + Node 22 + Vite + Zoo Code 3.87.100557, med alle
      image-fiksene fra Studio 15 (chown på volum, Copilot fjernet med
      byggvakt, trust/velkomst av, ripgrep-symlink, autolagring,
      announcement-hack, telemetri av). Oppstart-utvidelse åpner
      Zoo-chatten automatisk. NY felle betalt: bare-repoer seedet av
      lobbyen (root) må chownes til uid 1000 + `safe.directory` i
      entrypoint, ellers nekter git («dubious ownership»).
- [x] LiteLLM-proxy (`proxy/`, pinnet v1.103.2, UTEN database):
      master-nøkkel, `standard` (Fable 5) med fallback til `reserve`
      (Opus 5), `drop_params: true`, `num_retries: 2`. Nøkkel kun i
      proxyen; arbeidsflatene får bare master-nøkkelen mot proxyen.
- [x] Vaktmester-appen OPPRETTET og VERIFISERT (Jørn + chat 1, 03.10):
      GitHub App `studio15-light-vaktmester` (App ID 5176150) under
      @KODE15AS (som er en personlig konto, ikke org — klikkelisten
      rettet), installert i `KODE15-saturday-test-2`. E2E-verifisert:
      JWT → installasjonstoken → repo opprettet i org → repo slettet.
      Nøkkel i `certs/vaktmester.pem` + Bitwarden.
- [x] Vaktmester-INTEGRASJONEN i lobbyen (chat 2, 03.10): program med org →
      privat GitHub-repo opprettet/seedet i org-en; arbeidsflater henter
      FERSKE repo-scopede tokens via POST /api/git-token (det ene
      caddy-unntaket, per-flate-hemmelighet, HMAC — stateless); sletteregimet
      sletter GitHub-repoet i samme operasjon. Programmer uten org bruker
      fortsatt bare-repoer (repo-typen velges ved opprettelse). Lobbyen fikk
      egress-nettet `s15l-ut` (s15l-front er internal). Konfetti-testen har
      GitHub-variant — GRØNN 03.10.
- [x] Lobby (webprofil-kode15, Rust/axum + Svelte 5): programregister
      (YAML i dette repoet), guidet org-opprettingsflyt med manuelle
      GitHub-steg, prosjektoppretting fra mal (Svelte 5 + Vite, pinnet),
      prosjektvelger, arbeidsflate per deltager ved behov, sletting med
      bekreftelse. Mock-driver (`WORKSPACE_DRIVER=mock`) for utvikling
      uten Docker. Controller-mønsteret: lobbyen er eneste komponent som
      styrer containere, kun faste operasjoner.
- [x] Docker-nett: lobbyen på internt nett (`s15l-front`, internal: true)
      som arbeidsflatene ikke kan rute til; caddy avviser trafikk fra
      arbeidsflate-subnettet (client_ip-vakt, DNAT-lærdommen). Begge
      vaktene verifiseres i konfetti-testen.
- [x] HTTPS via `tailscale cert` for alle URL-er — én inngang
      (caddy :8100), alt samme origin (V2-forberedelse for se/peke).
      `skript/hent-sertifikat.sh` henter/fornyer.
- [x] Wifi-inngang uten Tailscale (04.10, se Beslutninger): caddy-snippeten
      «ruting» deles av ts.net-inngangen (:8100, tailscale-sertifikat) og
      wifi-inngangen (:8102, Let's Encrypt for lobby.studio15.cloud, bundet
      til 10.10.0.22:443 + 80-redirect). Relative flate-lenker fra lobbyen.
      Verifisert maskinelt: healthz/vekking/editor/webside gjennom
      inngangen, gyldig sertifikat, secure context og Zoo-chat-panelet
      rendrer (nettleser-verifisert fra wifi-siden).
- [x] Dvale/vekke: idle-reaper i lobbyen (45 min, fra caddys tilgangslogg)
      + 503-vekkeside med autovekking og reload. Verifisert maskinelt med
      1-minutts grense.
- [x] Sletteskript (`skript/slett-prosjekt.sh` + lobby-UI): containere +
      volumer + prosjektrepo + registeroppføring i én operasjon etter
      eksplisitt bekreftelse; GitHub-repo/org som dokumentert manuelt steg
      til vaktmesteren finnes.
- [x] Konfetti-testen (`skript/konfetti-test.sh`), maskinell del GRØNN
      03.10: lobby → repo-seed → arbeidsflate → code-server og Vite
      gjennom HTTPS-proxyen → proxy → Fable 5-svar (temperature droppet)
      → filredigering synlig live (HMR verifisert i nettleser) →
      nettvakter → sletting. Zoo-chat-leddet kan ikke fjernstyres maskinelt
      (webview-iframe) — BESTÅTT av Jørn i mennesketest 03.10 (sete 1,
      wifi/Tailscale): prompt i Zoo-chatten → endring levende på websiden.
- [ ] Begge deltagere verifisert fra wifi med Tailscale. Demo-prosjektet
      («Demo» → «Konfetti») står klart i lobbyen; testresept levert i
      chat 1. Sete 1 (Jørns PC) har alt vist lobby/editor/webside over
      Tailscale med gyldig HTTPS under nettlesertesten.

V2 — samarbeidslaget (bygget og maskintestet av chat 2, 03.10):

- [x] Presence-laget (`lobby/src/presence.rs` + `/samling/<program>/
      <prosjekt>/`): roster-snapshot etter welcome, reconnect med
      session-token + generasjonsteller, tile-relative cursor-koordinater,
      se/peke (overlay + ghost-cursors), ta over (code-servers flerbruker-
      tilkobling; kun én ekstern controller, ta/slipp broadcastes).
      16-punkts protokolltest `skript/presence-test.sh` — GRØNN 03.10.
      UI-verifisert i nettleser (ta over/slipp, badges, rammer).
- [x] 70"-veggen, programvaren: `/vegg` (watch-modus — joiner aldri, sender
      aldri input; auto-valg av prosjekt; begge halvdeler med editor +
      levende webside, eierfarge-rammer, controller-badges, ghost-cursors).
      Veggen vekker aldri sovende flater (?watch=1 på vekkesiden —
      verifisert maskinelt). Kiosk-skriptene i `vegg/` med alle fellene
      (wmctrl på PID, DISPLAY-vakt, autostart-MAPPE, helse-gate, frisk
      profil, VEGG_POSISJON).
- [x] Fysisk verifisering på 70"-en (04.10): kiosken installert i autostart
      (Skjermsamlings wall-watcher flyttet til side), fullskjerm via wmctrl,
      live prosjekt vist på veggen. To nye feller betalt underveis: GNOME
      krasjer ved skjerm-hotplug (kjent fra erfaringsoverføringen — fast
      skjerm!), og kortnavn over DNS-grensen gjorde flater uoppnåelige
      (lengdevakt lagt inn). Se/peke/ta over PÅ veggen testes i
      mennesketesten.
- [x] UI-justeringer fra mennesketesten (Jørn, 04.10): KODE15-logoen er
      hjem-knapp på alle sider (hvit brikke på samlingens mørke header);
      på lobbyen virker den som forenklet F5 (location.reload — nettbrett
      mangler lett tilgjengelig reload). Nettleser-verifisert begge veier.
- [x] Deploy-cache-fellen tettet (04.10): SPA-HTML serveres nå med
      Cache-Control no-cache i caddy — bufret index.html pekte på utgåtte
      asset-hasher etter utrulling og ga «tom lobby» med kun
      Nytt program-kortet. Assets (hash i navnet) caches som før.
- [x] Vegg-lærdom (04.10): kiosk-vakta oppdager død prosess, men ikke
      FROSSEN side — veggsiden overlevde dagens caddy-restarter i fryst
      tilstand og måtte skytes manuelt (pkill på profil-stien; vakta
      starter friskt). Vurder watchdog på /api/tilstand-polling senere.
- [x] Fjern-restart av veggen fra setene (04.10, Jørns ønske): «↻ Vegg»-
      knappen ytterst til høyre i samlingens header → POST /api/vegg/restart
      (tidsstempel i lobbyen) → kiosk-vakta poller og starter Chromium
      friskt. Virker også når veggsiden er frossen/krasjet. E2E-verifisert
      (knappetrykk → ny kiosk på 5 s).
- [x] ROTÅRSAK garble/krasj på 70"-en funnet (04.10): vakta sporet kiosken
      med pgrep -f på profilstien, men snap-Chromium bytter argv underveis
      i oppstarten og forker (launcher-PID dør) → falsk «død» → DOBBELT-
      start, og start nr. 2 slettet profilen under den kjørende. Fiks:
      vakta venter nå til wmctrl ser kiosk-VINDUET og sporer eier-PID-en
      (`kill -0`). Vakta må dessuten startes med sesjonens faktiske miljø —
      på raven er det DISPLAY=:1 og XAUTHORITY=/run/user/1000/gdm/
      Xauthority (hent fra gnome-shells /proc/PID/environ, anta aldri :0).
- [x] Veggen viser kun KJØRENDE flater (04.10): sovende testflater ga en
      skog av smale «sover»-kolonner; de dukker opp av seg selv når de
      våkner (5 s-polling). GNOME-varsler dempet på raven (veggmaskin):
      update-notifier + alle banner-varsler av via gsettings.
- [x] Vegg-herding etter Jørns kveldsrunde (04.10): kiosken kjører
      --disable-gpu (raven er hybrid Intel-iGPU/NVIDIA — GPU-kompositoren
      ga garble, hvite felter, spøkelsesrammer og krasj; NB: NVIDIA skal
      ALDRI drive skjerm på raven, kun iGPU-en) og --lang=en-US (ellers
      legger code-server norsk-språkpakke-varsel oppå editor-flisene).
      Veggen tar aldri input (pointer-events: none + skjult peker), og
      viser alltid minst to kolonner (én flate = én halvdel, resten tom).
      Vakta har flock-lås — relogin/autostart kan aldri gi to vakter.
- [x] Regresjonskjøring etter dagens endringer (04.10): presence-testen
      GRØNN (16/16) og konfetti-testen med GitHub-variant GRØNN — relative
      flate-lenker, caddy-snippeten, cache-regelen og ny PUBLIC_BASE
      verifisert maskinelt mot ts.net-inngangen.
- [x] Testrapport-endringene (natt til 05.10, hele pakken i
      `2026-10-04-nattarbeid-testrapport.md`): startside.studio15.cloud
      med evig lobby-redirect; navneskiftene i all synlig tekst
      (prosjektgruppe/startside/tavle/skjerm, `/tavle`-rute); brødsmuler i
      GitHub-stil på startside + prosjektside (og inn i web-profilen);
      `/gruppe/<slug>/`-URL-er; «Lag ny prosjektgruppe»/«Lag nytt
      prosjekt» ut av rutenettet; gruppefliser med inntil 4 prosjektnavn +
      skroller; deltagerregister (åpen tabell i repoet, fast farge, velger
      på startsiden — bevisst unntak fra «ingen innlogging», avklart);
      sync-granskningen besvart + fiks (tavlas/samlingens fremmede
      editor-fliser åpner prosjektets hovedfil og følger lagringer).
      Verifisert i nettleser + begge regresjonstestene GRØNNE på nytt.
- [x] Testrapport 1 (05.10, hele pakken i `2026-10-05-testrapport-1.md`):
      WebRTC-piloten «Del til tavla» (énveis webside-strøm deltager→tavle,
      signalering over presence, video med iframe-fallback, ● direkte-
      merke, ?deltest=1-krok — E2E maskintestet); coturn på raven som
      varig STUN/TURN (Tailscale-robusthet); tavle-halvdelene speiler
      deltagerskjermen (editor venstre, webside høyre); zombie-vekking
      fikset (synlighetsvakt på vekkesiden + REAPER_IGNORE_IPS);
      template-designnotat (2c). BESLUTNING ført: revisjon av
      erfaringsoverføringen — énveis streaming KUN for websiden;
      nettbrett er nødløsning for deling. Gjenstår menneskelig: ekte
      delingsdialog + Tailscale-PC→raven-strøm (neste mennesketest).
- [ ] Omfattende mennesketest med Jørn — full resept i
      `2026-10-03-mennesketest-v2.md` (inkl. sete 2 som gjenstår fra V1).
      Status 04.10: sete 2 (nettbrett) er inne via wifi-inngangen; punkt 1
      (konfetti fra sete 2) avbrutt av nettbrett-frys — gjenopptas.
