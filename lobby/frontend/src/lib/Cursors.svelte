<script>
  // Fargede ghost-cursors for ÉN tile: HTML-overlay (ikke OS-pekere) med
  // koordinater normalisert innenfor tilen — pekeren treffer riktig uansett
  // hvilken visning mottakeren ser (egen flate, kollega eller vegg).
  import { cursors } from './presence.svelte.js'

  let { tileId } = $props()

  const synlige = $derived([...cursors].filter(([, c]) => c.tile === tileId))
</script>

<div class="cursors" aria-hidden="true">
  {#each synlige as [id, c] (id)}
    <div class="cursor" style="left: {c.x * 100}%; top: {c.y * 100}%">
      <svg width="22" height="22" viewBox="0 0 24 24">
        <path d="M4 2 L20 12 L12 13.5 L8.5 21 Z" fill={c.farge} stroke="#233038" stroke-width="1.5" />
      </svg>
      <span class="navn" style="background: {c.farge}">{c.navn}</span>
    </div>
  {/each}
</div>

<style>
  .cursors {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    z-index: 50;
  }
  .cursor {
    position: absolute;
    transform: translate(-2px, -2px);
    transition: left 0.05s linear, top 0.05s linear;
    display: flex;
    align-items: flex-start;
  }
  .navn {
    margin-top: 14px;
    margin-left: -4px;
    color: #233038;
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.1rem 0.5rem;
    border-radius: 999px;
    white-space: nowrap;
  }
</style>
