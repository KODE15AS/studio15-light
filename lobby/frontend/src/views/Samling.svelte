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

  // Eierfarge per skjerm: presence først, ellers registerfargen fra
  // /api/tilstand (04.10) — konsistent farge også når eieren er frakoblet.
  const farge = (flate) =>
    presence.deltagere.find((d) => d.slug === flate.deltager)?.farge ??
    flate.farge ??
    '#77838C'

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

  // Fjern-restart av tavla (Jørn 04.10): setter tidsstempelet som
  // kiosk-vakta på raven poller — virker også når tavlesiden er frossen.
  let tavleStartes = $state(false)
  async function restartTavle() {
    try {
      await fetch('/api/vegg/restart', { method: 'POST' })
      tavleStartes = true
      setTimeout(() => (tavleStartes = false), 6000)
    } catch {}
  }

  // Andres editor-flis er en EGEN code-server-tilkobling uten åpne faner
  // (velkomstskjermen, 04.10) — be den åpne prosjektets hovedfil, så
  // se/ta over faktisk viser koden. Egen flis røres ikke (egen økt).
  const PAYLOAD = encodeURIComponent(
    '[["openFile","vscode-remote:///home/coder/project/src/App.svelte"]]'
  )
  // Samme slugify som lobbyen (statisk — ellers reloader iframen når
  // presence kobler til og «min flate» avklares).
  const slugifyNavn = (s) =>
    s
      .toLowerCase()
      .replaceAll('æ', 'ae')
      .replaceAll('ø', 'oe')
      .replaceAll('å', 'aa')
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
  const minSlug = deltagerNavn ? slugifyNavn(deltagerNavn) : ''
  const editorUrl = (flate) =>
    flate.deltager === minSlug
      ? flate.editor_url
      : `${flate.editor_url}?folder=/home/coder/project&payload=${PAYLOAD}`

  if (deltagerNavn) join(program, prosjekt, deltagerNavn)
  hent()
  setInterval(hent, 5000)
</script>

<svelte:window onkeydown={tastetrykk} />

<div class="stage">
  <header>
    <!-- Logoen er alltid veien hjem (Jørn 04.10) -->
    <a class="hjem" href="/" title="Til startsiden">
      <img src="/kode15-logo.png" alt="KODE15 — til startsiden" />
    </a>
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
          {presence.deg && f.deltager === presence.deg.slug ? 'Min skjerm' : f.deltager}
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

    <button
      class="veggknapp"
      title="Start tavla (70-tommeren) på nytt hvis den henger"
      disabled={tavleStartes}
      onclick={restartTavle}
    >
      {tavleStartes ? 'Tavla startes …' : '↻ Tavle'}
    </button>
  </header>

  {#if presence.feil}
    <div class="feil">{presence.feil}</div>
  {/if}

  {#if !deltagerNavn}
    <div class="melding">
      Mangler deltagernavn — gå til <a href="/">startsiden</a> og åpne skjermen derfra.
    </div>
  {:else if !visteFlate}
    <div class="melding">
      Ingen skjermer i prosjektet ennå — opprett en fra <a href="/">startsiden</a>.
    </div>
  {:else}
    {#key visteFlate.kortnavn}
      <main>
        <FlateTile
          tittel="Editor — {visteFlate.deltager}"
          url={editorUrl(visteFlate)}
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
  /* Logoen har mørk tekst — hvit brikke gjør den lesbar på mørk header. */
  .hjem {
    display: flex;
    align-items: center;
    background: #fff;
    border-radius: 6px;
    padding: 3px 7px;
  }
  .hjem img {
    height: 20px;
    display: block;
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
  /* Diskret nødknapp ytterst til høyre (roster har margin-left:auto). */
  .veggknapp {
    background: none;
    border: 1px solid #3a4652;
    color: #8a949c;
    font: inherit;
    font-size: 12px;
    border-radius: 999px;
    padding: 4px 12px;
    cursor: pointer;
    white-space: nowrap;
  }
  .veggknapp:disabled {
    opacity: 0.6;
    cursor: default;
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
