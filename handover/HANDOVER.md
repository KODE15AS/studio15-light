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

## Åpne spørsmål (avklares før bygging, jf. erfaringsoverføringen kap. 10)

- [ ] Zoo Code-verifisering: utvidelses-ID, distribusjon (Open VSX?),
      `.roomodes`-format, announcement-hack, ripgrep-fellen.
- [ ] HTTP eller HTTPS på tailnettet (clipboard/secure context) — velg én gang.
- [ ] Trengs moduser/begrensninger for to likeverdige utviklere?
- [ ] Persistens: containere alltid oppe, eller dvale/vekke-mønster?
- [ ] Kostnadsinnsyn: felles master-nøkkel eller én virtuell nøkkel per person?

## Leveranser

- [x] Repo opprettet på GitHub (KODE15AS/studio15-light) og klonet til
      `~/dev/studio15-light` på raven.
- [x] Stack på plass: compose + plassholder-container på
      http://100.65.19.39:8100 (kun Tailscale).
- [ ] Åpne spørsmål over avklart med Jørn.
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
