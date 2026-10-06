<script>
  // Dialog-speilet på tavla (Jørn 06.10, rapport 1 pkt. 2): viser
  // deltagerens Zoo-samtale, lest fra containeren via lobbyen
  // (/api/arbeidsflater/<kortnavn>/dialog). Tavlas egen code-server-økt
  // kunne aldri vise den — samtalen bor i deltagerens nettleserøkt.
  // Read-only som alt annet på tavla; ruller selv til nyeste melding.
  let { kortnavn, tittel, eierFarge = '#77838C' } = $props()

  let meldinger = $state([])
  let aktiv = $state(false)
  let boks = $state(null)

  async function hent() {
    try {
      const r = await fetch(`/api/arbeidsflater/${kortnavn}/dialog`)
      if (!r.ok) return
      const data = await r.json()
      meldinger = data.meldinger ?? []
      aktiv = data.aktiv ?? false
    } catch {}
  }
  hent()
  const timer = setInterval(hent, 3000)
  $effect(() => () => clearInterval(timer))

  // Rull til bunnen når det kommer nytt — tavla skal alltid vise siste.
  $effect(() => {
    meldinger.length
    aktiv
    if (boks) boks.scrollTop = boks.scrollHeight
  })

  // Minimal markdown → HTML for hjelperens meldinger: fet, kode og
  // lenker (lenketekst beholdes, mål droppes — tavla er read-only).
  function formater(tekst) {
    const esc = tekst
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
    return esc
      .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replaceAll('\n', '<br>')
  }
</script>

<div class="tile" style="--eier: {eierFarge}">
  <div class="hode">
    <span class="prikk" style="background: {eierFarge}"></span>
    <span class="tittel">{tittel}</span>
    {#if aktiv}<span class="jobber">hjelperen jobber …</span>{/if}
  </div>
  <div class="dialog" bind:this={boks}>
    {#if meldinger.length === 0}
      <p class="tom">Ingen samtale ennå — dialogen dukker opp her.</p>
    {:else}
      {#each meldinger as m}
        {#if m.hvem === 'verktoy'}
          <div class="verktoy">⚙ {m.tekst}</div>
        {:else}
          <div class="boble {m.hvem}">
            <!-- eslint-disable-next-line svelte/no-at-html-tags — kilden
                 escapes i formater() før mønstrene legges på -->
            {@html formater(m.tekst)}
            {#if m.forslag?.length}
              <div class="forslag">
                {#each m.forslag as f}
                  <span class="pille">{f}</span>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      {/each}
      {#if aktiv}
        <div class="verktoy puls">⏳ hjelperen skriver …</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    border-radius: 10px;
    border: 2px solid var(--eier);
    background: #f4f1ec;
    overflow: hidden;
  }
  .hode {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px;
    font-size: 12px;
    color: #c7cdd2;
    background: #1d262e;
    flex: none;
  }
  .prikk {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex: none;
  }
  .tittel {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .jobber {
    margin-left: auto;
    color: #8a949c;
    font-style: italic;
  }
  .dialog {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 16px;
    line-height: 1.45;
    color: #3d3a36;
  }
  .tom {
    margin: auto;
    color: #8a949c;
  }
  .boble {
    max-width: 88%;
    padding: 9px 13px;
    border-radius: 12px;
    overflow-wrap: break-word;
  }
  .boble.hjelper {
    align-self: flex-start;
    background: #fff;
    border: 1px solid #e3ded6;
    border-bottom-left-radius: 4px;
  }
  .boble.deltager {
    align-self: flex-end;
    background: color-mix(in srgb, var(--eier) 18%, #fff);
    border: 1px solid color-mix(in srgb, var(--eier) 40%, #fff);
    border-bottom-right-radius: 4px;
  }
  .boble :global(code) {
    background: #eee9e1;
    border-radius: 4px;
    padding: 1px 5px;
    font-size: 0.92em;
  }
  .forslag {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }
  .pille {
    border: 1px solid #cdc6ba;
    background: #faf8f4;
    border-radius: 999px;
    padding: 3px 11px;
    font-size: 0.86em;
    color: #5a554e;
  }
  .verktoy {
    align-self: flex-start;
    font-size: 0.82em;
    color: #8a949c;
    font-style: italic;
    padding: 0 4px;
  }
  .puls {
    animation: puls 1.6s ease-in-out infinite;
  }
  @keyframes puls {
    50% {
      opacity: 0.35;
    }
  }
</style>
