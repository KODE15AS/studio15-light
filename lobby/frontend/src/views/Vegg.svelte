<script>
  // 70"-veggen (V2): alltid read-only (watch-modus — joiner aldri, sender
  // aldri input; Ravens tastatur/mus brukes aldri). Viser begge halvdeler:
  // én kolonne per deltager med editor/dialog + levende webside, eierfarge-
  // rammer, «X kontrollerer»-badges og ghost-cursors.
  //
  // DeltagerSTYRT (Jørn 08.10): én deltager er i ett prosjekt om gangen
  // (aktiv-pekeren i deltagerregisteret), så tavla viser én flis per aktiv
  // deltager — på tvers av prosjekter. Hvert prosjekt har sitt eget
  // presence-rom, så tavla holder én watch-tilkobling per rom (WebRTC-
  // signaleringen bor i rommet). ?program=&prosjekt= i URL-en begrenser
  // fortsatt til ett prosjekt om ønskelig.
  import { SvelteMap } from 'svelte/reactivity'
  import FlateTile from '../lib/FlateTile.svelte'
  import DialogTile from '../lib/DialogTile.svelte'
  import { kobleVeggRom } from '../lib/presence.svelte.js'

  const params = new URLSearchParams(location.search)
  const fastProgram = params.get('program')
  const fastProsjekt = params.get('prosjekt')

  let fliser = $state([]) // fra /api/tavle: én per aktiv deltager
  let iceServers = []
  const strommer = new SvelteMap() // kortnavn → MediaStream
  // romnøkkel "program/prosjekt" → { kobling, pcs, lukk }
  const rom = new SvelteMap()

  const romKey = (f) => `${f.program}/${f.prosjekt}`

  // --- Webside-strøm fra deltageren (WebRTC-pilot 05.10) ---
  // Tavla søker jevnlig («soek») i hvert rom; deltagere som deler svarer
  // med tilbud. Kommer en strøm opp, viser webside-flisen video i stedet
  // for iframe — ekte speiling av det deltageren ser. Faller strømmen
  // bort, kommer iframen tilbake av seg selv.
  function lagRom(key) {
    const [program, prosjekt] = key.split('/')
    const kobling = kobleVeggRom(program, prosjekt)
    const pcs = new Map() // streamer-id → { pc, kortnavn }

    function rydd(fra) {
      const r = pcs.get(fra)
      if (!r) return
      r.pc.close()
      strommer.delete(r.kortnavn)
      pcs.delete(fra)
    }

    kobling.onStrom(async (m) => {
      const meg = kobling.st.watchId
      if (!meg) return
      try {
        if (m.signal.type === 'tilbud' && m.til === meg) {
          rydd(m.fra)
          const pc = new RTCPeerConnection({ iceServers })
          pcs.set(m.fra, { pc, kortnavn: m.flate })
          pc.ontrack = (e) => strommer.set(m.flate, e.streams[0])
          pc.onconnectionstatechange = () => {
            if (['failed', 'closed', 'disconnected'].includes(pc.connectionState)) rydd(m.fra)
          }
          pc.onicecandidate = (e) => {
            if (e.candidate) kobling.sendStrom(m.flate, m.fra, { type: 'is', kandidat: e.candidate })
          }
          await pc.setRemoteDescription(m.signal.sdp)
          const svar = await pc.createAnswer()
          await pc.setLocalDescription(svar)
          kobling.sendStrom(m.flate, m.fra, { type: 'svar', sdp: pc.localDescription })
        } else if (m.signal.type === 'is' && m.til === meg) {
          await pcs.get(m.fra)?.pc.addIceCandidate(m.signal.kandidat)
        } else if (m.signal.type === 'slutt') {
          rydd(m.fra)
        } else if (m.signal.type === 'starter') {
          // Deltager begynte å dele mens tavla sto på — be om tilbud straks.
          kobling.sendStrom('*', null, { type: 'soek' })
        }
      } catch {}
    })

    // Jevnlig søk fanger tavle-restarter, nye delinger og tapte signaler.
    const soek = setInterval(() => {
      if (kobling.st.watchId) kobling.sendStrom('*', null, { type: 'soek' })
    }, 5000)

    return {
      kobling,
      lukk() {
        clearInterval(soek)
        for (const fra of [...pcs.keys()]) rydd(fra)
        kobling.lukk()
      },
    }
  }

  async function hent() {
    try {
      const r = await fetch('/api/tavle')
      if (!r.ok) return
      const data = await r.json()
      iceServers = data.ice ?? []
      // Kun KJØRENDE flater på veggen (04.10): sovende flater ga en skog
      // av smale «sover»-kolonner. Vekkesideteksten lover allerede at en
      // flate «dukker opp her av seg selv» når den våkner.
      let alle = (data.fliser ?? []).filter((f) => f.kjorer)
      if (fastProgram && fastProsjekt) {
        alle = alle.filter((f) => f.program === fastProgram && f.prosjekt === fastProsjekt)
      }
      fliser = alle
      // Ferske fliser får nådeperioden fra første observasjon — deltageren
      // rekker å joine presence før flisen eventuelt skjules.
      for (const f of fliser) {
        if (!sistSett.has(f.deltager)) sistSett.set(f.deltager, Date.now())
      }
      // Presence-rommene synkes mot flisenes prosjekter.
      const trengs = new Set(fliser.map(romKey))
      for (const [key, r] of rom) {
        if (!trengs.has(key)) {
          r.lukk()
          rom.delete(key)
        }
      }
      for (const key of trengs) {
        if (!rom.has(key)) rom.set(key, lagRom(key))
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

  // Presence-tilstand for en flis: tilkoblet-prikk og kontroll-badge
  // hentes fra flisens EGET prosjektrom.
  const romSt = (f) => rom.get(romKey(f))?.kobling.st
  const erTilkoblet = (f) =>
    romSt(f)?.deltagere.some((d) => d.slug === f.deltager && d.tilkoblet) ?? false

  // Deltager borte = flis av tavla (Jørn 08.10): presence styrer flisene.
  // En deltager som lukker skjermen sin forsvinner fra tavla etter en
  // kort nådeperiode (overlever F5/reconnect og oppstartsvinduet), og
  // flisen kommer tilbake i det øyeblikket skjermen åpnes igjen.
  // Rom-tilkoblingene følger fortsatt ALLE fliser, så returen oppdages.
  const NAADE_MS = 15000
  const sistSett = new SvelteMap() // deltager-slug → sist sett tilkoblet
  let naa = $state(Date.now())
  setInterval(() => {
    for (const f of fliser) {
      if (erTilkoblet(f)) sistSett.set(f.deltager, Date.now())
    }
    naa = Date.now()
  }, 3000)
  const synlige = $derived(
    fliser.filter((f) => erTilkoblet(f) || naa - (sistSett.get(f.deltager) ?? 0) < NAADE_MS)
  )

  hent()
  setInterval(hent, 5000)
</script>

<div class="vegg">
  {#if synlige.length === 0}
    <div class="venter">
      <span class="kicker">Studio 15 LIGHT</span>
      <h1>Ingen aktive prosjekter</h1>
      <p>Tavla våkner når noen åpner en skjerm fra startsiden.</p>
    </div>
  {:else}
    <header>
      <span class="kicker">Studio 15 LIGHT · tavle</span>
      <strong>{[...new Set(synlige.map((f) => f.prosjekt_navn))].join(' · ')}</strong>
      <div class="roster">
        {#each synlige as f (f.kortnavn)}
          <span class="deltager" class:borte={!erTilkoblet(f)} style="--farge: {f.farge}">
            <span class="prikk"></span>{f.navn}
          </span>
        {/each}
      </div>
    </header>
    <!-- Dynamisk disponering (Jørn 05.10 kveld): 1–2 deltagere deler
         bredden (minst to kolonner, 04.10: tom halvdel = plass til
         nestemann), 3–4 gir 2×2-rutenett — hver rute 1920×1080 på
         70-tommeren. Flere enn 4: flere kolonner i to rader. -->
    <main
      style="--kolonner: {synlige.length <= 2 ? 2 : Math.ceil(synlige.length / 2)};
             --rader: {synlige.length <= 2 ? 1 : 2}"
    >
      {#each synlige as f (f.kortnavn)}
        {@const kontroll = romSt(f)?.kontroll[f.kortnavn] ?? null}
        {@const blankUi = f.mal === 'nybegynner' || f.mal === 'ekspert'}
        <section class:nybegynner={blankUi}>
          {#if blankUi}
            <!-- Dialog-speilet (06.10, rapport 1 pkt. 2): tavlas egen
                 code-server-økt viser bare en fersk, tom Zoo-chat —
                 samtalen hentes i stedet fra containeren via lobbyen.
                 Gjelder begge blank-UI-malene (ekspert 09.10): deltageren
                 ser aldri editoren, så tavla speiler dialogen. -->
            <DialogTile
              tittel="{f.mal === 'ekspert' ? 'Agenten' : 'Hjelperen'} — {f.navn} · {f.prosjekt_navn}"
              kortnavn={f.kortnavn}
              eierFarge={f.farge}
            />
          {:else}
            <FlateTile
              tittel="Editor — {f.navn} · {f.prosjekt_navn}"
              url={editorUrl(f.editor_url)}
              tileId="{f.kortnavn}:editor"
              modus="vegg"
              eierFarge={f.farge}
              {kontroll}
            />
          {/if}
          <FlateTile
            tittel="Webside — {f.navn}"
            url={veggUrl(f.web_url)}
            tileId="{f.kortnavn}:web"
            modus="vegg"
            eierFarge={f.farge}
            strom={strommer.get(f.kortnavn) ?? null}
            {kontroll}
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
    grid-template-rows: repeat(var(--rader, 1), 1fr);
    gap: 10px;
    padding: 10px;
    min-height: 0;
    /* Veggen tar ALDRI input (Jørn 04.10): klikk med ravens mus nådde
       iframene og fikk fliser til å krasje. Musepekeren skjules også. */
    pointer-events: none;
    cursor: none;
  }
  /* Hver rute speiler deltagerskjermen (Jørn 05.10): editor til
     venstre, levende webside til høyre — samme plassering som i
     samlingsvisningen. Nybegynner speiler ⅓/⅔-delingen. */
  section {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    min-height: 0;
    min-width: 0;
  }
  section.nybegynner {
    grid-template-columns: 1fr 2fr;
  }
</style>
