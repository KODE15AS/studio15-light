# HANDOVER — Studio 15 LIGHT

Levende spesifikasjon (norm «handover»). Punktene krysses av etter hvert som
de leveres. Chat 0 startet 2026-10-03.

## Grunnlag — les disse først

- «Erfaringsoverforing Studio15 LIGHT.docx» i `~/dev/studio15/` — hele
  rammen, fellene fra Studio 15 og rådene inn i Zoo Code.
- Følgedokument: `docs/ERFARINGSOVERFORING-ZOO-CODE.md` i
  `raven-skjermsamling` (per 03.10 på umerget gren
  `cursor/skjermsamling-light-brief-24b6`) — samarbeidslaget (70"-veggen).

## Målbilde

To personer på hver sin PC over Tailscale (wifi), hver med sin arbeidsflate:
VS Code i nettleseren (code-server) + Zoo Code koblet mot LiteLLM-proxy +
Vite-dev-server som viser websiden levende. Ravens 70"-skjerm viser begge
flatene 50/50. Ingen innlogging, ingen database, ingen offentlig eksponering.

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

## Åpne punkter (agentarbeid, første byggetrinn)

- [ ] Zoo Code-verifisering: utvidelses-ID, distribusjon (Open VSX?),
      `.roomodes`-format, announcement-hack, ripgrep-fellen.
- [ ] Hente følgedokumentet `docs/ERFARINGSOVERFORING-ZOO-CODE.md` fra
      umerget gren `cursor/skjermsamling-light-brief-24b6` i
      `raven-skjermsamling`.

## Leveranser

- [x] Repo opprettet på GitHub (KODE15AS/studio15-light) og klonet til
      `~/dev/studio15-light` på raven.
- [x] Stack på plass: compose + plassholder-container på
      http://100.65.19.39:8100 (kun Tailscale).
- [x] Åpne spørsmål fra erfaringsoverføringen kap. 10 avklart med Jørn
      (se Beslutninger over).
- [ ] HTTPS på plass via `tailscale cert` for arbeidsflate-URL-ene.
- [ ] Dvale/vekke: idle-reaper + 503-vekkeside for de to setene.
- [ ] Workspace-image: code-server (pinnet versjon) + Node LTS + Vite +
      Zoo Code, med alle image-fiksene fra Studio 15 (chown på volum,
      Copilot fjernet, trust/velkomst av, ripgrep-symlink, autolagring).
- [ ] LiteLLM-proxy i enkleste form: master-nøkkel, Fable 5 primær med
      fallback til Opus 5, `drop_params: true`. Nøkkel kun i proxyen.
- [ ] Docker-nett: workspace-containere deler aldri nett med styrende
      tjenester (`internal: true`-mønsteret).
- [ ] Konfetti-testen: modus → proxy → modell → filredigering → synlig på
      levende webside.
- [ ] To seter verifisert fra wifi med Tailscale.
- [ ] 70"-visningen 50/50 (etter følgedokumentet fra raven-skjermsamling).
