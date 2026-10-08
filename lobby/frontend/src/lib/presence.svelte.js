// Presence-klienten (se/peke/ta over) — mønsteret fra Skjermsamling:
// session-token i localStorage for reconnect, roster som sannhetskilde,
// tile-relative cursor-koordinater, watch-modus for veggen (joiner aldri).
import { SvelteMap } from 'svelte/reactivity'

export const presence = $state({
  tilkoblet: false,
  deg: null, // egen deltager fra velkommen
  deltagere: [],
  kontroll: {}, // kortnavn → { id, navn, farge }
  feil: null,
  // Flyktig adresse for watch-tilkoblinger (tavla) — kun til
  // WebRTC-signaleringen (pilot 05.10).
  watchId: null,
})

// id → { tile, x, y, navn, farge, ts } — normalisert innenfor tilen.
export const cursors = new SvelteMap()

let ws = null
let reconnectTimer = null
let watchModus = false
let joinNavn = null
let joinFarge = null // fast farge fra deltagerregisteret (04.10)
let rommet = null // { program, prosjekt }

const sessionKey = () => `s15l.presence.${rommet.program}/${rommet.prosjekt}.session`

function wsUrl() {
  const proto = location.protocol === 'https:' ? 'wss' : 'ws'
  const watch = watchModus ? '?watch=1' : ''
  return `${proto}://${location.host}/api/presence/${rommet.program}/${rommet.prosjekt}/ws${watch}`
}

function haandterMelding(ev) {
  let m
  try {
    m = JSON.parse(ev.data)
  } catch {
    return
  }
  switch (m.type) {
    case 'velkommen':
      presence.deg = m.deg
      localStorage.setItem(sessionKey(), m.session)
      break
    case 'roster':
      presence.deltagere = m.deltagere
      presence.kontroll = m.kontroll
      if (presence.deg) {
        const oppdatert = m.deltagere.find((d) => d.id === presence.deg.id)
        if (oppdatert) presence.deg = oppdatert
      }
      break
    case 'cursor':
      if (presence.deg && m.id === presence.deg.id) break // egen ghost vises ikke
      cursors.set(m.id, { tile: m.tile, x: m.x, y: m.y, navn: m.navn, farge: m.farge, ts: Date.now() })
      break
    case 'feil':
      presence.feil = m.melding
      setTimeout(() => (presence.feil = null), 5000)
      break
    case 'watch_velkommen':
      presence.watchId = m.id
      break
    case 'strom':
      // WebRTC-signalering (pilot 05.10): relayet melding — visningene
      // (Samling/Vegg) registrerer handler og filtrerer selv på `til`.
      stromHandler?.(m)
      break
  }
}

function aapne() {
  ws = new WebSocket(wsUrl())
  ws.onopen = () => {
    presence.tilkoblet = true
    if (!watchModus && joinNavn) {
      const session = localStorage.getItem(sessionKey()) || undefined
      ws.send(
        JSON.stringify({ type: 'join', navn: joinNavn, session, farge: joinFarge ?? undefined })
      )
    }
  }
  ws.onmessage = haandterMelding
  ws.onclose = () => {
    presence.tilkoblet = false
    // Automatisk reconnect — identiteten lever på serveren til timeout.
    reconnectTimer = setTimeout(aapne, 1500)
  }
  ws.onerror = () => ws && ws.close()
}

/** Koble til som deltager og join med navn (+ ev. fast registerfarge). */
export function join(program, prosjekt, navn, farge = null) {
  rommet = { program, prosjekt }
  watchModus = false
  joinNavn = navn
  joinFarge = farge
  aapne()
}

/** Koble til read-only (veggen): joiner aldri, sender aldri noe. */
export function watch(program, prosjekt) {
  rommet = { program, prosjekt }
  watchModus = true
  joinNavn = null
  aapne()
}

function send(obj) {
  if (!watchModus && ws && ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify(obj))
}

let sistSendt = 0
/** Egen peker, normalisert innenfor tilen (tile = "<kortnavn>:editor|web"). */
export function sendCursor(tile, x, y) {
  const naa = performance.now()
  if (naa - sistSendt < 33) return // ~30 Hz
  sistSendt = naa
  send({ type: 'cursor', tile, x, y })
}

export const ta = (flate) => send({ type: 'ta', flate })
export const slipp = () => send({ type: 'slipp' })

// --- WebRTC-signalering for webside-strømmen (pilot 05.10) ---
// Går UTENOM send(): tavla (watch) må også kunne sende strøm-signal —
// serveren slipper kun `strom`-meldinger gjennom fra watch-tilkoblinger.
let stromHandler = null
export function onStrom(fn) {
  stromHandler = fn
}
export function sendStrom(flate, til, signal) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type: 'strom', flate, til: til ?? undefined, signal }))
  }
}

export function forlat() {
  send({ type: 'forlat' })
  if (rommet) localStorage.removeItem(sessionKey())
  presence.deg = null
  if (reconnectTimer) clearTimeout(reconnectTimer)
  if (ws) {
    ws.onclose = null
    ws.close()
    ws = null
  }
  presence.tilkoblet = false
}

// --- Flerroms-watch for tavla (08.10) ---
// Tavla er deltagerstyrt og kan vise flere prosjekter samtidig — da
// trengs én watch-tilkobling PER prosjektrom (WebRTC-signaleringen bor i
// rommet). Frittstående fabrikk, uavhengig av singleton-tilstanden over
// (Samling bruker fortsatt den).
export function kobleVeggRom(program, prosjekt) {
  const st = $state({ watchId: null, deltagere: [], kontroll: {} })
  let sock = null
  let timer = null
  let handler = null
  let lukket = false
  const proto = location.protocol === 'https:' ? 'wss' : 'ws'
  const url = `${proto}://${location.host}/api/presence/${program}/${prosjekt}/ws?watch=1`

  function aapneRom() {
    if (lukket) return
    sock = new WebSocket(url)
    sock.onmessage = (ev) => {
      let m
      try {
        m = JSON.parse(ev.data)
      } catch {
        return
      }
      if (m.type === 'roster') {
        st.deltagere = m.deltagere
        st.kontroll = m.kontroll
      } else if (m.type === 'watch_velkommen') {
        st.watchId = m.id
      } else if (m.type === 'strom') {
        handler?.(m)
      }
    }
    sock.onclose = () => {
      st.watchId = null
      if (!lukket) timer = setTimeout(aapneRom, 1500)
    }
    sock.onerror = () => sock && sock.close()
  }
  aapneRom()

  return {
    st,
    onStrom: (fn) => (handler = fn),
    sendStrom: (flate, til, signal) => {
      if (sock && sock.readyState === WebSocket.OPEN) {
        sock.send(JSON.stringify({ type: 'strom', flate, til: til ?? undefined, signal }))
      }
    },
    lukk: () => {
      lukket = true
      if (timer) clearTimeout(timer)
      if (sock) {
        sock.onclose = null
        sock.close()
        sock = null
      }
      st.watchId = null
    },
  }
}

// Ghost-cursors som har stått stille lenge ryddes bort.
setInterval(() => {
  const naa = Date.now()
  for (const [id, c] of cursors) {
    if (naa - c.ts > 8000) cursors.delete(id)
  }
}, 2000)
