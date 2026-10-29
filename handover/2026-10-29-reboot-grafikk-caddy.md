# 29.10: Feilmelding på 70-tommeren → reboot → caddy-boot-fella

Jørn gjenopptok testing (DEFA-pretest via studio15.cloud) og fikk
feilmelding på 70-tommeren. Diagnose, fiks og tre lærdommer.

## Hendelsesforløpet

1. TV-hotplug kl. 10:00 utløste den KJENTE mutter-buggen (segfault i
   `meta_monitor_get_edid_checksum_md5`, samme som 07.10) — GNOME Shell
   krasjet.
2. Shell klarte ikke restarte: unattended-upgrades hadde oppgradert
   NVIDIA-driveren på disk (595.84 → 595.99.02 den 11.10) mens den gamle
   kjernemodulen fortsatt var lastet (siste boot 12.09) → «NVRM: API
   mismatch» → «no available drivers found». Systemet flagget selv
   «restart required» (to nyere kjerner ventet også).
3. Jørn rebootet raven. Driver og kjerne matcher nå (595.99.02 på
   7.0.0-38), GNOME og kiosk kom opp.
4. MEN: s15l-caddy døde ved boot — `failed to bind host port
   10.10.0.33:443: cannot assign requested address`. Rotårsak:
   **kode15-wifi hadde autoconnect AV** og kom aldri opp, så wifi-IP-en
   fantes ikke da Docker startet containerne.
5. Felle i fella: `docker start`/`docker restart` av den feilede
   containeren ga «Up» men **publiserte ingen porter** (alle
   `NetworkSettings.Ports` null, sbJoin uten gateway-endepunkt) — helt
   stille. Kun `docker compose up -d --force-recreate caddy` (etter at
   wifi-en var oppe) ga portene tilbake.

## Fikset og verifisert 10:23

- kode15-wifi: oppe med 10.10.0.33 og `autoconnect yes` (Jørn, sudo).
- caddy gjenskapt: tailnet- og wifi-inngangene svarer 200, kiosken
  rendrer tavla med DEFA-prosjektet, flaten svarer.

## Lærdommer

1. **Etter driver-oppgraderinger fra unattended-upgrades må raven
   rebootes snart** — ellers er neste GNOME-krasj (f.eks. HDMI-hotplug)
   uopprettelig uten omstart. Sjekk: `cat /var/run/reboot-required`.
2. **Boot-rekkefølge-fella:** caddy binder til wifi-IP-en; kommer wifi-en
   etter Docker, dør caddy stille. Autoconnect er nå på, men wifi tar
   uansett sekunder — vurder varig vern: `net.ipv4.ip_nonlocal_bind=1` i
   `/etc/sysctl.d/` (lar binds til ennå-fraværende IP-er lykkes), eller
   flytt tavla til Tavle-Pi-en og wifi-avhengigheten til VLAN-trunken
   (WifiOmada fase 2).
3. **Stol aldri på «Up» etter en feilet containerstart med IP-spesifikke
   porter:** sjekk `docker port <navn>` — er den tom, må containeren
   gjenskapes med compose, ikke restartes.

Mutter-hotplug-buggen består; varig løsning er fortsatt Tavle-Pi-en
(bestilt 07.10, `docs/tavle-pi-spec.md`).
