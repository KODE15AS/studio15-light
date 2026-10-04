<script lang="ts">
  // Ruting uten router-avhengighet: fire visninger på samme SPA.
  //   /                          startsiden (velg prosjektgruppe)
  //   /gruppe/<slug>/            prosjektsiden for én prosjektgruppe
  //   /samling/<gruppe>/<prosjekt>/   deltagerens samlingsvisning (V2)
  //   /tavle                     70"-tavla (watch-modus, read-only)
  //                              (/vegg lever som alias — gammelt navn)
  import Lobby from './views/Lobby.svelte'
  import Samling from './views/Samling.svelte'
  import Vegg from './views/Vegg.svelte'

  const sti = location.pathname
  const samling = sti.match(/^\/samling\/([a-z0-9-]+)\/([a-z0-9-]+)\/?$/)
  const gruppe = sti.match(/^\/gruppe\/([a-z0-9-]+)\/?$/)
  const erTavle = /^\/(tavle|vegg)\/?$/.test(sti)
</script>

{#if samling}
  <Samling program={samling[1]} prosjekt={samling[2]} />
{:else if erTavle}
  <Vegg />
{:else if gruppe}
  <Lobby gruppe={gruppe[1]} />
{:else}
  <Lobby />
{/if}
