# 2026-10-07 — «Oh no! Something has gone wrong» på tavla (løst)

Gjentagende problem ved omkobling av 70-tommeren til raven: TV-en viste
GNOME-krasjskjermen i stedet for tavla.

## Rotårsak (fra coredump 18:06, verifisert)

1. Ved HDMI-hotplug av Samsung-en segfaulter gnome-shell/mutter i
   `meta_monitor_get_edid_checksum_md5` (EDID-lesing, kjent sårbar
   kodevei i GNOME 46 på Ubuntu 24.04).
2. systemd restarter skallet av seg selv på sekunder — det går bra, og
   kiosk-vinduet overlever (X11: appene dør ikke med vindushåndtereren).
3. MEN systemd starter i tillegg `gnome-session-failed.service` — en
   «lockdown»-krasjskjerm som blir liggende OVER alt, selv når skallet
   lever igjen. Det var den TV-en viste.

## Felle avdekket underveis

Å stoppe `gnome-session-failed.service` manuelt river HELE økten
(utlogging, X-serveren dør, kiosken med den). Ikke gjør det.

## Varig fix (hostnivå på raven)

```
systemctl --user mask gnome-session-failed.service
```

(symlink `~/.config/systemd/user/gnome-session-failed.service → /dev/null`)

Skallet restartes fortsatt automatisk ved krasj; lockdown-skjermen kan
aldri mer legge seg over tavla. Skulle GNOME en dag være ekte ødelagt,
ser man det på skjermen uansett — og raven har SSH.

**Testet kontrollert 07.10 18:12**: SIGSEGV sendt til gnome-shell →
skallet restartet (active), krasjskjerm uteble (inactive), økten
overlevde, kiosk-vinduet urørt (samme PID), tavla tilbake av seg selv.
Jørn bekreftet visuelt fra TV-en.

## Perspektiv

Hele problemklassen (hotplug mot ravens hybrid-GPU/GNOME) bortfaller
når tavla flyttes til egen Raspberry Pi 5 (bestilt — se
`docs/tavle-pi-spec.md`, oppsett i chat 3).
