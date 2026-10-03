<script>
  // Samlingsvisningen (V2): deltagerens vindu inn i prosjektet.
  // Egen flate = editor + levende webside side om side. Kollegaens flate:
  // se/peke via overlay, dobbeltklikk (eller knapp) for å ta over —
  // overtagelsen ER code-servers flerbrukertilkobling (avklart 03.10),
  // presence-laget broadcaster bare hvem som kontrollerer hva.
  import FlateTile from '../lib/FlateTile.svelte'
  import { presence, join, ta, slipp } from '../lib/presence.svelte.js'

  let { program, prosjekt } = $props()

  const deltagerNavn = new URLSearchParams(location.search).get('deltager') ?? ''

  let prosjektNavn = $state(prosjekt)
  let flater = $state([])
  let valgt = $state(null) // kortnavn på flaten som vises

  async function hent() {
    try {
      const r = await fetch('/api/tilstand')
      if (!r.ok) return
      const data = await r.json()
      for (const p of data.programmer) {
        if (p.slug !== program) continue
        for (const pr of p.prosjekter) {
          if (pr.slug !== prosjekt) continue
          prosjektNavn = pr.navn
          flater = pr.arbeidsflater
        }
      }
    } catch {}
  }

  // Eierfarge per flate: deltager-sluggen kobler flate ↔ presence-deltager.
  const farge = (flate) =>
    presence.deltagere.find((d) => d.slug === flate.deltager)?.farge ?? '#77838C'

  const minFlate = $derived(flater.find((f) => presence.deg && f.deltager === presence.deg.slug))
  const visteFlate = $derived(flater.find((f) => f.kortnavn === valgt) ?? minFlate ?? flater[0])
  const kontrollerer = $derived(
    visteFlate &&
      presence.deg &&
      presence.kontroll[visteFlate.kortnavn]?.id === presence.deg.id
  )
  const erMin = $derived(visteFlate && presence.deg && visteFlate.deltager === presence.deg.slug)
  const modus = $derived(erMin ? 'egen' : kontrollerer ? 'kontroll' : 'se')

  function velg(kortnavn) {
    if (valgt !== kortnavn) slipp() // bytte av flate slipper kontrollen
    valgt = kortnavn
  }

  function taOver() {
    if (visteFlate && !erMin) ta(visteFlate.kortnavn)
  }

  function tastetrykk(e) {
    if (e.key === 'Escape' && kontrollerer) slipp()
  }

  if (deltagerNavn) join(program, prosjekt, deltagerNavn)
  hent()
  setInterval(hent, 5000)
</script>

<svelte:window onkeydown={tastetrykk} />

<div class="stage">
  <header>
    <a class="hjem" href="/">← Lobby</a>
    <span class="kicker">Samling</span>
    <strong class="prosjekt">{prosjektNavn}</strong>

    <nav class="flatevalg">
      {#each flater as f (f.kortnavn)}
        <button
          class:aktiv={visteFlate?.kortnavn === f.kortnavn}
          style="--farge: {farge(f)}"
          onclick={() => velg(f.kortnavn)}
        >
          <span class="prikk"></span>
          {presence.deg && f.deltager === presence.deg.slug ? 'Min flate' : f.deltager}
          {#if presence.kontroll[f.kortnavn]}
            <span class="mini-badge" style="background: {presence.kontroll[f.kortnavn].farge}"
              >{presence.kontroll[f.kortnavn].navn}</span
            >
          {/if}
        </button>
      {/each}
    </nav>

    <div class="roster">
      {#each presence.deltagere as d (d.id)}
        <span class="deltager" class:borte={!d.tilkoblet} style="--farge: {d.farge}">
          <span class="prikk"></span>{d.navn}
        </span>
      {/each}
      {#if !presence.tilkoblet}
        <span class="frakoblet">kobler til …</span>
      {/if}
    </div>

    {#if modus === 'se'}
      <button class="handling" onclick={taOver}>Ta over</button>
    {:else if kontrollerer}
      <button class="handling slipp" onclick={() => slipp()}>Slipp (Esc)</button>
    {/if}
  </header>

  {#if presence.feil}
    <div class="feil">{presence.feil}</div>
  {/if}

  {#if !deltagerNavn}
    <div class="melding">
      Mangler deltagernavn — gå til <a href="/">lobbyen</a> og åpne arbeidsflaten derfra.
    </div>
  {:else if !visteFlate}
    <div class="melding">
      Ingen arbeidsflater i prosjektet ennå — opprett en i <a href="/">lobbyen</a>.
    </div>
  {:else}
    {#key visteFlate.kortnavn}
      <main>
        <FlateTile
          tittel="Editor — {visteFlate.deltager}"
          url={visteFlate.editor_url}
          tileId="{visteFlate.kortnavn}:editor"
          {modus}
          eierFarge={farge(visteFlate)}
          kontroll={presence.kontroll[visteFlate.kortnavn] ?? null}
          onta={taOver}
        />
        <FlateTile
          tittel="Webside — {visteFlate.deltager}"
          url={visteFlate.web_url}
          tileId="{visteFlate.kortnavn}:web"
          {modus}
          eierFarge={farge(visteFlate)}
          kontroll={presence.kontroll[visteFlate.kortnavn] ?? null}
          onta={taOver}
        />
      </main>
    {/key}
  {/if}
</div>

<style>
  /* Mørk «stage» rundt levende flater (Skjermsamling F). */
  .stage {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #0c1116;
    color: #c7cdd2;
    font-family: -apple-system, 'Segoe UI', Roboto, sans-serif;
    font-size: 14px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 14px;
    background: #141b22;
    flex: none;
    flex-wrap: wrap;
  }
  .hjem {
    color: #8a949c;
    text-decoration: none;
  }
  .kicker {
    font-size: 11px;
    letter-spacing: 2px;
    text-transform: uppercase;
    color: #8a949c;
  }
  .prosjekt {
    color: #fff;
    font-weight: 500;
  }
  .flatevalg {
    display: flex;
    gap: 8px;
  }
  .flatevalg button {
    display: flex;
    align-items: center;
    gap: 7px;
    background: none;
    border: 1px solid #2a343d;
    color: #c7cdd2;
    border-radius: 999px;
    padding: 4px 14px;
    font: inherit;
    cursor: pointer;
  }
  .flatevalg button.aktiv {
    border-color: var(--farge);
    background: #1d262e;
  }
  .prikk {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--farge);
    flex: none;
  }
  .mini-badge {
    color: #233038;
    font-size: 10.5px;
    font-weight: 700;
    padding: 0 7px;
    border-radius: 999px;
  }
  .roster {
    display: flex;
    gap: 12px;
    margin-left: auto;
    align-items: center;
  }
  .deltager {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .deltager.borte {
    opacity: 0.45;
  }
  .frakoblet {
    color: #e8a33d;
    font-size: 12.5px;
  }
  .handling {
    background: #bbad9a;
    border: none;
    color: #233038;
    font: inherit;
    font-weight: 600;
    border-radius: 999px;
    padding: 5px 16px;
    cursor: pointer;
  }
  .handling.slipp {
    background: #e55381;
    color: #fff;
  }
  .feil {
    background: #3a1d26;
    color: #ffb3c6;
    padding: 6px 14px;
    flex: none;
  }
  .melding {
    margin: auto;
    background: #141b22;
    border: 1px solid #2a343d;
    border-radius: 14px;
    padding: 26px 34px;
  }
  .melding a {
    color: #bbad9a;
  }
  main {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    padding: 10px;
    min-height: 0;
  }
</style>
