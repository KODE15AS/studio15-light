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

// Ghost-cursors som har stått stille lenge ryddes bort.
setInterval(() => {
  const naa = Date.now()
  for (const [id, c] of cursors) {
    if (naa - c.ts > 8000) cursors.delete(id)
  }
}, 2000)
