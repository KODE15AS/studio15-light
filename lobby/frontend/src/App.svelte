<script lang="ts">
  // Ruting uten router-avhengighet: tre visninger på samme SPA.
  //   /                          lobbyen (program → prosjekt → arbeidsflate)
  //   /samling/<program>/<prosjekt>/   deltagerens samlingsvisning (V2)
  //   /vegg                      70"-veggen (watch-modus, read-only)
  import Lobby from './views/Lobby.svelte'
  import Samling from './views/Samling.svelte'
  import Vegg from './views/Vegg.svelte'

  const sti = location.pathname
  const samling = sti.match(/^\/samling\/([a-z0-9-]+)\/([a-z0-9-]+)\/?$/)
  const erVegg = /^\/vegg\/?$/.test(sti)
</script>

{#if samling}
  <Samling program={samling[1]} prosjekt={samling[2]} />
{:else if erVegg}
  <Vegg />
{:else}
  <Lobby />
{/if}
