# AGENTS.md — studio15-light

- Følg RAVEN-normene i `raven-platform/README.md` (handover, cursor, kontoer,
  tbd, container, stack, lenker, web-profil, bestilling, språk).
- Start hver chat med å lese `handover/HANDOVER.md` og siste daterte
  handover — «les handover» er nok som første melding.
- Grunnlagsdokumentene (erfaringsoverføringene fra Studio 15 og
  raven-skjermsamling) er fasit for hva som bevisst er droppet: ingen
  innlogging, ingen database, ingen offentlig eksponering. Ikke gjeninnfør
  dette «for sikkerhets skyld».
- Secrets: `.env` per stack (chmod 600, aldri i git). API-nøkler bor i
  proxyen, aldri i workspace-containere.
- Pin alle image- og utvidelsesversjoner; oppgrader bevisst ved rebuild.
