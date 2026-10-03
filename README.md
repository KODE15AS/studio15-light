# studio15-light

Forenklet 2-bruker versjon av Studio 15 med Zoo Code for agentstyring.
Bygges fra scratch — ingen kode gjenbrukes fra Studio 15, bare erfaringene
(se `handover/HANDOVER.md`).

## Lenker

| Hva | Tilgang | URL |
|---|---|---|
| Statusside (plassholder) | 🔒 Kun Tailscale | http://100.65.19.39:8100 |
| Dette repoet på GitHub | 🌐 Offentlig (krever innlogging) | https://github.com/KODE15AS/studio15-light |

🌐 Offentlig — kan nås fra internett. 🔒 Kun Tailscale — kun internt.

## Status

Repoet er nyopprettet. Containeren som kjører nå er en plassholder som kun
serverer en statusside, slik at stacken (compose, port, Tailscale-binding)
er på plass fra dag én. Selve arbeidsflaten (code-server + Zoo Code + Vite)
bygges trinnvis etter `handover/HANDOVER.md`.

## Kjøring

```sh
docker compose up -d --build
```

Containeren binder kun til Tailscale-adressen (100.65.19.39) — ingen
offentlig eksponering, i tråd med erfaringsoverføringen.
