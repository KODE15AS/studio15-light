<script>
  // 70"-veggen (V2): alltid read-only (watch-modus — joiner aldri, sender
  // aldri input; Ravens tastatur/mus brukes aldri). Viser begge halvdeler:
  // én kolonne per arbeidsflate med editor + levende webside, eierfarge-
  // rammer, «X kontrollerer»-badges og ghost-cursors.
  //
  // Prosjektvalg: ?program=&prosjekt= i URL-en, ellers automatisk det
  // prosjektet som har flest kjørende arbeidsflater.
  import FlateTile from '../lib/FlateTile.svelte'
  import { presence, watch, forlat } from '../lib/presence.svelte.js'

  const params = new URLSearchParams(location.search)
  const fastProgram = params.get('program')
  const fastProsjekt = params.get('prosjekt')

  let rom = $state(null) // { program, prosjekt, navn }
  let flater = $state([])

  function velgRom(data) {
    // Fast valg fra URL hvis satt, ellers prosjektet med flest kjørende flater.
    let beste = null
    for (const p of data.programmer) {
      for (const pr of p.prosjekter) {
        // Kun KJØRENDE flater på veggen (04.10): sovende testflater ga en
        // skog av smale «sover»-kolonner. Vekkesideteksten lover allerede
        // at en flate «dukker opp her av seg selv» når den våkner — og det
        // gjør den, via 5-sekunders-pollingen.
        const kjorende = pr.arbeidsflater.filter((f) => f.kjorer)
        if (fastProgram && fastProsjekt) {
          if (p.slug === fastProgram && pr.slug === fastProsjekt) {
            return { program: p.slug, prosjekt: pr.slug, navn: pr.navn, flater: kjorende }
          }
          continue
        }
        if (kjorende.length > 0 && (!beste || kjorende.length > beste.kjorende)) {
          beste = { program: p.slug, prosjekt: pr.slug, navn: pr.navn, flater: kjorende, kjorende: kjorende.length }
        }
      }
    }
    return beste
  }

  async function hent() {
    try {
      const r = await fetch('/api/tilstand')
      if (!r.ok) return
      const data = await r.json()
      const valgt = velgRom(data)
      if (!valgt) {
        // Ingen aktive prosjekter: koble fra og vent.
        if (rom) forlat()
        rom = null
        flater = []
        return
      }
      flater = valgt.flater
      if (!rom || rom.program !== valgt.program || rom.prosjekt !== valgt.prosjekt) {
        if (rom) forlat()
        rom = { program: valgt.program, prosjekt: valgt.prosjekt, navn: valgt.navn }
        watch(rom.program, rom.prosjekt)
      }
    } catch {}
  }

  // Tavlas iframes sover ikke skjermene våkne: ?watch=1 på URL-ene får
  // vekkesiden til å vise dvale uten å vekke (se vekk_side i lobbyen).
  const veggUrl = (url) => url + (url.includes('?') ? '&' : '?') + 'watch=1'

  // Editor-flisen på tavla er en EGEN code-server-tilkobling: åpne faner
  // bor per nettleser, så uten dette viste tavla bare velkomstskjermen
  // mens deltageren jobbet (Jørns funn 04.10). Payload-en ber code-server
  // åpne prosjektets hovedfil — den oppdateres ved hver lagring
  // (filvokteren), så tavla følger koden. Websiden er uansett levende.
  const PAYLOAD = encodeURIComponent(
    '[["openFile","vscode-remote:///home/coder/project/src/App.svelte"]]'
  )
  const editorUrl = (url) =>
    veggUrl(`${url}?folder=/home/coder/project&payload=${PAYLOAD}`)

  hent()
  setInterval(hent, 5000)
</script>

<div class="vegg">
  {#if !rom}
    <div class="venter">
      <span class="kicker">Studio 15 LIGHT</span>
      <h1>Ingen aktive prosjekter</h1>
      <p>Tavla våkner når noen åpner en skjerm fra startsiden.</p>
    </div>
  {:else}
    <header>
      <span class="kicker">Studio 15 LIGHT · tavle</span>
      <strong>{rom.navn}</strong>
      <div class="roster">
        {#each presence.deltagere as d (d.id)}
          <span class="deltager" class:borte={!d.tilkoblet} style="--farge: {d.farge}">
            <span class="prikk"></span>{d.navn}
          </span>
        {/each}
      </div>
    </header>
    <!-- Alltid minst to kolonner (Jørn 04.10): første flate tar én
         halvdel, den andre halvdelen står tom til neste flate kommer. -->
    <main style="--kolonner: {Math.max(flater.length, 2)}">
      {#each flater as f (f.kortnavn)}
        {@const eierFarge =
          presence.deltagere.find((d) => d.slug === f.deltager)?.farge ?? f.farge ?? '#77838C'}
        <section>
          <FlateTile
            tittel="Editor — {f.deltager}"
            url={editorUrl(f.editor_url)}
            tileId="{f.kortnavn}:editor"
            modus="vegg"
            {eierFarge}
            kontroll={presence.kontroll[f.kortnavn] ?? null}
          />
          <FlateTile
            tittel="Webside — {f.deltager}"
            url={veggUrl(f.web_url)}
            tileId="{f.kortnavn}:web"
            modus="vegg"
            {eierFarge}
            kontroll={presence.kontroll[f.kortnavn] ?? null}
          />
        </section>
      {/each}
    </main>
  {/if}
</div>

<style>
  .vegg {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #0c1116;
    color: #c7cdd2;
    font-family: -apple-system, 'Segoe UI', Roboto, sans-serif;
    cursor: none; /* veggen har aldri egen peker */
  }
  .kicker {
    font-size: 11px;
    letter-spacing: 2px;
    text-transform: uppercase;
    color: #8a949c;
  }
  header {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 6px 16px;
    background: #141b22;
    flex: none;
  }
  header strong {
    color: #fff;
    font-weight: 500;
  }
  .roster {
    display: flex;
    gap: 14px;
    margin-left: auto;
  }
  .deltager {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .deltager.borte {
    opacity: 0.45;
  }
  .prikk {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--farge);
  }
  .venter {
    margin: auto;
    text-align: center;
  }
  .venter h1 {
    font-weight: 400;
    color: #fff;
  }
  main {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(var(--kolonner), 1fr);
    gap: 10px;
    padding: 10px;
    min-height: 0;
    /* Veggen tar ALDRI input (Jørn 04.10): klikk med ravens mus nådde
       iframene og fikk fliser til å krasje. Musepekeren skjules også. */
    pointer-events: none;
    cursor: none;
  }
  /* Hver halvdel: editor øverst (60 %), levende webside under (40 %). */
  section {
    display: grid;
    grid-template-rows: 3fr 2fr;
    gap: 10px;
    min-height: 0;
  }
</style>
