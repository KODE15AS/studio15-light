<script>
  // Én levende tile (editor ELLER webside) med eierfarge-ramme, eventuell
  // controller-ramme + badge, ghost-cursors og peke-overlay.
  //
  // Modus:
  //   egen     – interaktiv (din egen flate; eieren har alltid kontroll)
  //   se       – overlay fanger pekeren: se/peke + dobbeltklikk for å ta over
  //   kontroll – interaktiv (du har tatt over; code-servers flerbruker-
  //              tilkobling gjør resten — ingen input-streaming)
  //   vegg     – read-only: all input er slått av (watch-modus)
  import Cursors from './Cursors.svelte'
  import { sendCursor } from './presence.svelte.js'

  let { tittel, url, tileId, modus, eierFarge = '#77838C', kontroll = null, onta = () => {} } = $props()

  function pek(e) {
    const r = e.currentTarget.getBoundingClientRect()
    sendCursor(tileId, (e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height)
  }
</script>

<div
  class="tile"
  class:vegg={modus === 'vegg'}
  style="--eier: {eierFarge}; --ctrl: {kontroll?.farge ?? 'transparent'}"
>
  <div class="hode">
    <span class="prikk" style="background: {eierFarge}"></span>
    <span class="tittel">{tittel}</span>
    {#if kontroll}
      <span class="badge" style="background: {kontroll.farge}">{kontroll.navn} kontrollerer</span>
    {/if}
  </div>
  <div class="flate" class:kontrollert={kontroll}>
    <iframe src={url} title={tittel} class:dod={modus === 'se' || modus === 'vegg'}></iframe>
    {#if modus === 'se'}
      <!-- Peke-overlay: fanger pekeren (se/peke), dobbeltklikk tar over -->
      <div
        class="overlay"
        role="button"
        tabindex="-1"
        onpointermove={pek}
        ondblclick={onta}
        title="Dobbeltklikk for å ta over"
      ></div>
    {/if}
    <Cursors {tileId} />
  </div>
</div>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    border-radius: 10px;
    /* Eierfarge som permanent ramme + controllerfarge som ytre ramme —
       umiddelbar lesbarhet (Skjermsamling F). */
    border: 2px solid var(--eier);
    box-shadow: 0 0 0 3px var(--ctrl);
    background: #161d23;
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
  .badge {
    margin-left: auto;
    color: #233038;
    font-weight: 700;
    font-size: 11px;
    padding: 1px 9px;
    border-radius: 999px;
    white-space: nowrap;
  }
  .flate {
    position: relative;
    flex: 1;
    min-height: 0;
  }
  iframe {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: 0;
    background: #fff;
  }
  /* «Død» iframe: ingen input slipper inn (se-modus og veggen). */
  iframe.dod {
    pointer-events: none;
  }
  .overlay {
    position: absolute;
    inset: 0;
    cursor: crosshair;
    z-index: 40;
  }
  .vegg iframe {
    pointer-events: none;
  }
</style>
