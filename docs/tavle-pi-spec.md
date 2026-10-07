# Tavle-PC på Raspberry Pi — spec for maksimal respons

Bestillings- og oppsettspec (07.10.2026) for å flytte tavla fra ravens
HDMI-port til et eget apparat bak 70-tommeren i klasserommet. Målet er
respons på nivå med dagens direkte HDMI: tavla rendrer webappen selv —
kun data krysser wifi, aldri piksler (se design-review i chat 07.10 og
handover 06.10).

## Maskinvare (bestillingsliste)

| # | Del | Hvorfor respons-kritisk |
|---|-----|------------------------|
| 1 | **Raspberry Pi 5, 8 GB** (ikke Pi 4) | 2–3× CPU/GPU mot Pi 4; WebRTC-dekoding skjer i programvare, og tavla kan vise 4 strømmer samtidig. 8 GB gir Chromium rom til code-server-iframes uten swapping. |
| 2 | **Offisiell aktiv kjøler (Active Cooler)** | Pi 5 throttler uten kjøling — throttling er den vanligste «mystiske tregheten» i kioskdrift. |
| 3 | **Offisiell 27 W USB-C-strømforsyning** | Under 27 W nedskaleres ytelsen og NVMe/USB får ikke full effekt. |
| 4 | **M.2 HAT+ (offisiell) + 256 GB NVMe SSD** (f.eks. Kingston NV2/WD SN350) | Boot fra NVMe: 5–10× raskere I/O enn microSD, Chromium-cache og oppstart flyr — og eliminerer SD-kort-korrupsjon, Pi-ens klassiske svakhet i 24/7-drift. |
| 5 | Kabinett med plass til HAT + kjøler (offisielt Pi 5-kabinett eller Argon NEO 5 M.2) | Termikk + fysisk beskyttelse bak TV-en. |
| 6 | **Micro-HDMI → HDMI-kabel, 2 m, HDMI 2.0-sertifisert** | Billige kabler gir signaldropp på lange trekk; 2.0 holder til 1080p60 med god margin (og 4K60 om vi noen gang vil). |
| 7 | (Reserve) 64 GB microSD A2 (SanDisk Extreme) | Kun til førstegangs-flash/berging; drift skjer fra NVMe. |

Typiske norske kilder: Kjell & Company, Multicom, RS/Farnell, Digital
Impuls. Kjøp delene enkeltvis — «starter kit»-ene inneholder som regel
SD-kort og spill-kabinett vi ikke vil ha, og mangler NVMe.

## Programvare (Jørns preferanse: Ubuntu 24.04)

- **Ubuntu Server 24.04 LTS arm64** (offisielt Pi 5-støttet av
  Canonical) — *Server*, ikke Desktop: GNOME-skrivebordet er den
  største enkeltstående responstyven på en Pi. Tavla trenger bare:
- **Minimal X11-kiosk**: `xorg` + `openbox` + autologin på en
  kioskbruker som starter tavlevakta. X11 (ikke Wayland) fordi
  `vegg-kiosk.sh`-mønstrene våre (wmctrl-fullskjermvakt, PID-sporing)
  gjenbrukes uendret, og Chromiums kiosk-modus er mest forutsigbar der.
- **Chromium (snap, pinnet versjon)** i kiosk mot
  `https://startside.studio15.cloud/tavle` — med **GPU-akselerasjon PÅ**
  (motsatt av raven: Pi-en har én enkel GPU, ingen hybrid-Intel/NVIDIA-
  garble; verifiser i `chrome://gpu` at rasterisering og kompositering
  er «Hardware accelerated»).
- **1920×1080@60 ut** — ikke 4K. Halverer render-arbeidet; TV-en
  skalerer, og på 70" fra klasseromsavstand er forskjellen usynlig.
  (Tavle-CSS-en vår er flytende og bryr seg ikke om oppløsningen.)
- `cpufreq`-governor **performance** (ikke ondemand) — jevn respons
  uten oppspinningsforsinkelse.
- journald flyktig + ingen swap-fil — NVMe-en lever lenge, og en
  kiosk har ingenting å logge lokalt.
- Vakta porteres fra `vegg/vegg-kiosk.sh`: samme helse-gating mot
  lobbyen, samme fjern-restart-polling («↻ Tavle» fra setene virker
  uendret), men som **systemd-tjeneste** i stedet for GNOME-autostart,
  og uten EDID-sjekken (apparatet har én jobb). SSH på for fjernstell
  fra raven.

## Nett

- Samme wifi som raven (10.10.0.x) — tavla når
  `startside.studio15.cloud` direkte, ingen hosts/ruter-triks (navnet
  peker på ravens wifi-IP).
- «Del til tavla» (WebRTC) går P2P deltager-PC → Pi over wifi; coturn
  på raven (10.10.0.22) er nåbar som relé-reserve. Ingen endringer.
- Fast DHCP-lease på Pi-ens MAC anbefales (gjenfinnbar for SSH).

## Gevinst utover flyttingen

Raven får skjermen sin tilbake permanent — tavle-av/på-bryteren vi
har diskutert bortfaller; tavla blir et apparat med én jobb.
