<script>
  // Samlingsvisningen (V2): deltagerens vindu inn i prosjektet.
  // Egen flate = editor + levende webside side om side. Kollegaens flate:
  // se/peke via overlay, dobbeltklikk (eller knapp) for å ta over —
  // overtagelsen ER code-servers flerbrukertilkobling (avklart 03.10),
  // presence-laget broadcaster bare hvem som kontrollerer hva.
  import FlateTile from '../lib/FlateTile.svelte'
  import { presence, join, ta, slipp, onStrom, sendStrom } from '../lib/presence.svelte.js'

  let { program, prosjekt } = $props()

  const deltagerNavn = new URLSearchParams(location.search).get('deltager') ?? ''

  let prosjektNavn = $state(prosjekt)
  let prosjektMal = $state('full')
  let flater = $state([])
  let valgt = $state(null) // kortnavn på flaten som vises
  let iceServers = [] // STUN/TURN (coturn på raven) fra /api/tilstand

  async function hent() {
    try {
      const r = await fetch('/api/tilstand')
      if (!r.ok) return
      const data = await r.json()
      iceServers = data.ice ?? []
      for (const p of data.programmer) {
        if (p.slug !== program) continue
        for (const pr of p.prosjekter) {
          if (pr.slug !== prosjekt) continue
          prosjektNavn = pr.navn
          flater = pr.arbeidsflater
          if (pr.mal) prosjektMal = pr.mal
        }
      }
    } catch {}
  }

  // --- Nybegynner-malen (Jørn 05.10 kveld, iterasjon etter rapport 4) ---
  // To kolonner: hjelperen (Zoo-chatten) til venstre ≈ ⅓, websiden til
  // høyre ≈ ⅔. Oppgavefeltet utgikk — spilleplanen eies nå av hjelperen
  // selv (todo-liste + svarknapper i chatten, instruert i AGENTS.md).
  // Skillet kan dras i bredden og huskes per prosjekt.
  const kolonneNokkel = `s15l-kolonner-${program}-${prosjekt}`
  let kolonner = $state(
    JSON.parse(localStorage.getItem(kolonneNokkel) ?? 'null') ?? [1 / 3, 2 / 3]
  )
  let stabelEl = $state(null)
  function startDra(e) {
    // Pointer capture: skillet beholder pekeren selv over iframene.
    e.preventDefault()
    const el = e.currentTarget
    const bredde = stabelEl.getBoundingClientRect().width
    const startX = e.clientX
    const start = [...kolonner]
    try {
      el.setPointerCapture(e.pointerId)
    } catch {} // enkelte pekere (test/berøring) mangler capture — draget virker likevel
    const flytt = (ev) => {
      const d = (ev.clientX - startX) / bredde
      const a = Math.max(0.15, Math.min(start[0] + d, 0.85))
      kolonner = [a, 1 - a]
    }
    const slippDra = () => {
      el.removeEventListener('pointermove', flytt)
      el.removeEventListener('pointerup', slippDra)
      localStorage.setItem(kolonneNokkel, JSON.stringify(kolonner))
    }
    el.addEventListener('pointermove', flytt)
    el.addEventListener('pointerup', slippDra)
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
    // Deling beskjærer til DIN webside-flis — forlater du egen skjerm,
    // forsvinner beskjæringsmålet, så delingen stoppes ryddig.
    if (deler && minFlate && kortnavn !== minFlate.kortnavn) {
      stoppDeling()
      deleMelding = 'Deling til tavla stoppet — du forlot din egen skjerm.'
      setTimeout(() => (deleMelding = ''), 6000)
    }
    valgt = kortnavn
  }

  // --- «Del til tavla» (WebRTC-pilot 05.10) ---
  // Deltagerens webside-flis fanges som fane-strøm (Region Capture:
  // beskåret til flisen) og sendes P2P til tavla. Signalering går over
  // presence-WebSocketen; tavla søker («soek»), vi svarer med tilbud.
  // Tavla faller tilbake til sin levende iframe uten strøm — piloten kan
  // aldri gjøre tavla dårligere. ?deltest=1 bytter fangsten med en
  // canvas-strøm uten tillatelsesdialog (maskintest-krok).
  let deler = $state(false)
  let deleMelding = $state('')
  let websideBoks = $state(null) // wrapper rundt webside-flisen (crop-mål)
  // Enheter uten skjermfangst (nettbrett = nødløsning) får ingen død
  // knapp — tavla viser uansett websiden fra serveren (fallback).
  const delingStottes =
    !!navigator.mediaDevices?.getDisplayMedia ||
    new URLSearchParams(location.search).get('deltest') === '1'
  let delStrom = null // MediaStream
  const pcs = new Map() // watcher-id → RTCPeerConnection
  let testTimer = null

  async function startDeling() {
    if (!minFlate) return
    try {
      let strom
      if (new URLSearchParams(location.search).get('deltest') === '1') {
        const c = document.createElement('canvas')
        c.width = 320
        c.height = 180
        const ctx = c.getContext('2d')
        testTimer = setInterval(() => {
          ctx.fillStyle = `hsl(${(Date.now() / 20) % 360} 70% 60%)`
          ctx.fillRect(0, 0, 320, 180)
        }, 200)
        strom = c.captureStream(5)
      } else {
        strom = await navigator.mediaDevices.getDisplayMedia({
          video: { frameRate: 15 },
          audio: false,
          preferCurrentTab: true,
          selfBrowserSurface: 'include',
        })
        const [spor] = strom.getVideoTracks()
        // Beskjær til INNHOLDET i webside-flisen (Region Capture,
        // Chromium ≥104) — ikke hele flisen: da ligger Chromes blå
        // fangst-indikator innenfor vår oransje ID-ramme i stedet for
        // oppå den (Jørn 05.10), og strømmen slipper flisens topplinje
        // (tavla har sin egen).
        const innhold = websideBoks?.querySelector('.flate') ?? websideBoks
        if (window.CropTarget && spor.cropTo && innhold) {
          const maal = await window.CropTarget.fromElement(innhold)
          await spor.cropTo(maal)
        }
        spor.addEventListener('ended', stoppDeling) // «Stopp deling» i nettleseren
      }
      delStrom = strom
      deler = true
      // Si fra til tavler som allerede står på at strømmen finnes.
      sendStrom(minFlate.kortnavn, null, { type: 'starter' })
    } catch (e) {
      // Skill «avbrutt av deg» og «støttes ikke» (nettbrett = nødløsning)
      // fra ekte feil (Jørn 05.10, rapport 2 pkt. 3).
      deleMelding =
        e?.name === 'NotAllowedError'
          ? 'Deling avbrutt.'
          : e?.name === 'NotSupportedError' || e instanceof TypeError
            ? 'Deling støttes ikke i denne nettleseren — tavla viser websiden fra serveren som før.'
            : 'Fikk ikke startet deling — prøv igjen.'
      setTimeout(() => (deleMelding = ''), 6000)
    }
  }

  function stoppDeling() {
    if (minFlate) sendStrom(minFlate.kortnavn, null, { type: 'slutt' })
    for (const pc of pcs.values()) pc.close()
    pcs.clear()
    delStrom?.getTracks().forEach((t) => t.stop())
    delStrom = null
    if (testTimer) clearInterval(testTimer)
    testTimer = null
    deler = false
  }

  async function tilbyTil(watcherId) {
    if (!deler || !delStrom || !minFlate) return
    // Tavla søker («soek») hvert 5. sekund for å fange restarter — men en
    // LEVENDE forbindelse skal aldri re-forhandles: hvert nye tilbud rev
    // ned strømmen på tavla og ga et svart blink hvert 5. sekund (Jørn
    // 06.10, rapport 1 pkt. 3). Nytt tilbud kun når forbindelsen er død,
    // eller når en oppkobling har stått fast i over 15 sekunder.
    const eksisterende = pcs.get(watcherId)
    if (eksisterende) {
      const alder = Date.now() - (eksisterende.s15lOpprettet ?? 0)
      if (eksisterende.connectionState === 'connected' || alder < 15000) return
    }
    pcs.get(watcherId)?.close()
    const pc = new RTCPeerConnection({ iceServers })
    pc.s15lOpprettet = Date.now() // for fastlåst-vakten over
    pcs.set(watcherId, pc)
    for (const spor of delStrom.getTracks()) pc.addTrack(spor, delStrom)
    pc.onicecandidate = (e) => {
      if (e.candidate) {
        sendStrom(minFlate.kortnavn, watcherId, { type: 'is', kandidat: e.candidate })
      }
    }
    const tilbud = await pc.createOffer()
    await pc.setLocalDescription(tilbud)
    sendStrom(minFlate.kortnavn, watcherId, { type: 'tilbud', sdp: pc.localDescription })
  }

  onStrom(async (m) => {
    const meg = presence.deg?.id
    if (!meg) return
    try {
      if (m.signal.type === 'soek' && deler) {
        tilbyTil(m.fra)
      } else if (m.signal.type === 'svar' && m.til === meg) {
        await pcs.get(m.fra)?.setRemoteDescription(m.signal.sdp)
      } else if (m.signal.type === 'is' && m.til === meg) {
        await pcs.get(m.fra)?.addIceCandidate(m.signal.kandidat)
      }
    } catch {}
  })

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

<!-- Nybegynnerskjermen følger KODE15-webprofilen (Jørn 05.10, rapport 4
     pkt. 3.1) — standardskjermen beholder den mørke scenen. -->
<div class="stage" class:lys={prosjektMal === 'nybegynner'}>
  <header>
    <!-- Logoen er alltid veien hjem (Jørn 04.10) -->
    <a class="hjem" href="/" title="Til startsiden">
      <img src="/kode15-logo.png" alt="KODE15 — til startsiden" />
    </a>
    <!-- «Samling» utgikk (Jørn 05.10, rapport 4 pkt. 4) -->
    <span class="kicker">Prosjekt:</span>
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

    {#if minFlate && delingStottes}
      <button
        class="veggknapp"
        class:deler
        title={deler
          ? 'Stopp webside-strømmen til tavla'
          : 'Send websiden din som direkte strøm til tavla (velg «Del» i dialogen)'}
        onclick={() => (deler ? stoppDeling() : startDeling())}
      >
        {deler ? '■ Deler til tavla' : '▶ Del til tavla'}
      </button>
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

  {#if deleMelding}
    <div class="feil">{deleMelding}</div>
  {/if}

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
      {#if prosjektMal === 'nybegynner'}
        <!-- Nybegynner (Jørn 05.10 kveld): hjelperen ⅓ til venstre,
             websiden ⅔ til høyre — skillet kan dras i bredden. -->
        <main class="nybegynner" bind:this={stabelEl}>
          <div class="kol" style="flex-grow: {kolonner[0]}">
            <FlateTile
              tittel="Editor — {visteFlate.deltager}"
              url={editorUrl(visteFlate)}
              tileId="{visteFlate.kortnavn}:editor"
              {modus}
              lys
              eierFarge={farge(visteFlate)}
              kontroll={presence.kontroll[visteFlate.kortnavn] ?? null}
              onta={taOver}
            />
          </div>
          <div
            class="skille"
            title="Dra for å endre bredden"
            onpointerdown={startDra}
          ></div>
          <div class="kol deleboks" style="flex-grow: {kolonner[1]}" bind:this={websideBoks}>
            <FlateTile
              tittel="Webside — {visteFlate.deltager}"
              url={visteFlate.web_url}
              tileId="{visteFlate.kortnavn}:web"
              {modus}
              lys
              eierFarge={farge(visteFlate)}
              kontroll={presence.kontroll[visteFlate.kortnavn] ?? null}
              onta={taOver}
            />
          </div>
        </main>
      {:else}
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
          <!-- Wrapper = beskjæringsmål for «Del til tavla» (Region Capture) -->
          <div class="deleboks" bind:this={websideBoks}>
            <FlateTile
              tittel="Webside — {visteFlate.deltager}"
              url={visteFlate.web_url}
              tileId="{visteFlate.kortnavn}:web"
              {modus}
              eierFarge={farge(visteFlate)}
              kontroll={presence.kontroll[visteFlate.kortnavn] ?? null}
              onta={taOver}
            />
          </div>
        </main>
      {/if}
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
  /* Aktiv deling: rosa «direkte»-markering (samme som tavlas live-merke) */
  .veggknapp.deler {
    border-color: #e55381;
    color: #e55381;
  }
  .deleboks {
    display: flex;
    min-width: 0;
    min-height: 0;
  }
  .deleboks > :global(.tile) {
    flex: 1;
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
  /* Nybegynner-malen: to kolonner med strekkbart skille — all flate er
     brukbar (ingen pynteareal, Jørn 05.10 kveld). */
  main.nybegynner {
    display: flex;
    flex-direction: row;
    gap: 0;
    padding: 6px;
  }
  .kol {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-basis: 0;
    flex-shrink: 1;
  }
  .kol > :global(.tile) {
    flex: 1;
  }
  .skille {
    flex: none;
    width: 10px;
    margin: 0 1px;
    cursor: col-resize;
    border-radius: 5px;
    background: #1d262e;
    touch-action: none;
  }
  .skille:hover {
    background: #2a343d;
  }
  /* --- KODE15-webprofil på nybegynnerskjermen (Jørn 05.10, pkt. 3.1) --- */
  .stage.lys {
    background: var(--k15-bg);
    color: var(--k15-skifer);
    font-family: var(--k15-font-body);
  }
  .stage.lys header {
    background: var(--k15-hvit);
    border-bottom: 1px solid var(--k15-linje);
  }
  .stage.lys .hjem {
    border: 1px solid var(--k15-linje);
  }
  .stage.lys .kicker {
    color: var(--k15-noytralgraa);
    font-family: var(--k15-font-heading);
  }
  .stage.lys .prosjekt {
    color: var(--k15-blaagraa-mork);
    font-family: var(--k15-font-heading);
    font-weight: 500;
  }
  .stage.lys .flatevalg button {
    border-color: var(--k15-linje);
    color: var(--k15-skifer);
    background: var(--k15-hvit);
  }
  .stage.lys .flatevalg button.aktiv {
    background: var(--k15-flate);
    border-color: var(--farge);
  }
  .stage.lys .deltager {
    color: var(--k15-skifer);
  }
  .stage.lys .veggknapp {
    border-color: var(--k15-linje);
    color: var(--k15-blaagraa-mork);
  }
  .stage.lys .veggknapp.deler {
    border-color: #e55381;
    color: #e55381;
  }
  .stage.lys .skille {
    background: var(--k15-linje);
  }
  .stage.lys .skille:hover {
    background: var(--k15-blaagraa-lys);
  }
  .stage.lys .melding {
    background: var(--k15-hvit);
    border-color: var(--k15-linje);
  }
  .stage.lys .feil {
    background: #f6e3e7;
    color: #a33d5e;
  }
</style>
