// Protokoll-smoketest for presence-laget (se/peke/ta over) — à la
// Skjermsamlings 16-punkts ws-test. Kjøres av presence-test.sh, som sørger
// for testfixturer (prosjekt + arbeidsflate «alfa») og kort reconnect-
// vindu (PRESENCE_TIMEOUT_SECS=2) før dette skriptet starter.
//
// Bruk: node presence-test.mjs <BASE> <PROGRAM> <PROSJEKT> <FLATE-KORTNAVN>

const [BASE, PROGRAM, PROSJEKT, FLATE] = process.argv.slice(2)
if (!FLATE) {
  console.error('Bruk: node presence-test.mjs <BASE> <PROGRAM> <PROSJEKT> <FLATE>')
  process.exit(2)
}
const WS_BASE = BASE.replace(/^http/, 'ws') + `/api/presence/${PROGRAM}/${PROSJEKT}/ws`

let punkt = 0
const ok = (navn) => console.log(`  ✓ ${++punkt}. ${navn}`)
const feil = (melding) => {
  console.error(`  ✗ FEIL (punkt ${punkt + 1}): ${melding}`)
  process.exit(1)
}
const vent = (ms) => new Promise((r) => setTimeout(r, ms))

// En testklient: samler meldinger, lar testen vente på bestemte typer.
function klient(navnILogg, { watch = false } = {}) {
  const ws = new WebSocket(WS_BASE + (watch ? '?watch=1' : ''))
  const k = {
    ws,
    meldinger: [],
    lukket: false,
    send: (o) => ws.send(JSON.stringify(o)),
    /** Vent på neste melding av gitt type (inkl. allerede mottatte uleste). */
    async neste(type, frist = 4000) {
      const start = Date.now()
      while (Date.now() - start < frist) {
        const i = k.meldinger.findIndex((m) => m.type === type)
        if (i >= 0) return k.meldinger.splice(i, 1)[0]
        await vent(20)
      }
      feil(`${navnILogg}: fikk aldri «${type}» innen ${frist} ms`)
    },
    /** Siste roster (venter på minst én). */
    async roster(frist) {
      let r = await k.neste('roster', frist)
      // tøm køen for nyere rostere
      let i
      while ((i = k.meldinger.findIndex((m) => m.type === 'roster')) >= 0) {
        r = k.meldinger.splice(i, 1)[0]
      }
      return r
    },
    tomKo: () => (k.meldinger.length = 0),
  }
  ws.onmessage = (ev) => k.meldinger.push(JSON.parse(ev.data))
  ws.onclose = () => (k.lukket = true)
  return new Promise((res, rej) => {
    ws.onopen = () => res(k)
    ws.onerror = (e) => rej(new Error(`${navnILogg}: ws-feil ${e.message ?? ''}`))
  })
}

console.log(`== Presence-smoketest mot ${WS_BASE} ==`)

// 1–2: join → velkommen + roster-snapshot direkte
const a = await klient('A')
a.send({ type: 'join', navn: 'Alfa' })
const velkommenA = await a.neste('velkommen')
if (!velkommenA.session || !velkommenA.deg?.id || !velkommenA.deg?.farge)
  feil('velkommen mangler session/id/farge')
if (velkommenA.deg.slug !== 'alfa') feil(`ventet slug alfa, fikk ${velkommenA.deg.slug}`)
ok('join → velkommen med session, id og farge')
const snapshotA = await a.neste('roster', 1000)
if (!snapshotA.deltagere.some((d) => d.id === velkommenA.deg.id))
  feil('roster-snapshotet mangler deltageren selv')
ok('roster-snapshot direkte etter velkommen (race-vaksinen)')

// 3: B joiner → A får roster med 2
const b = await klient('B')
b.send({ type: 'join', navn: 'Beta' })
const velkommenB = await b.neste('velkommen')
await b.neste('roster')
const rosterA2 = await a.roster()
if (rosterA2.deltagere.length !== 2) feil(`A ser ${rosterA2.deltagere.length} deltagere, ventet 2`)
if (velkommenB.deg.farge === velkommenA.deg.farge) feil('A og B fikk samme farge')
ok('B joiner → roster med 2 deltagere og ulike farger')

// 4: cursor fra A → B mottar med navn/farge/tile
a.send({ type: 'cursor', tile: `${FLATE}:editor`, x: 0.25, y: 0.75 })
const cursorHosB = await b.neste('cursor')
if (cursorHosB.tile !== `${FLATE}:editor` || cursorHosB.x !== 0.25 || cursorHosB.y !== 0.75)
  feil(`feil cursor hos B: ${JSON.stringify(cursorHosB)}`)
if (cursorHosB.navn !== 'Alfa' || cursorHosB.farge !== velkommenA.deg.farge)
  feil('cursor mangler riktig navn/farge')
ok('cursor fra A → B mottar tile-relativ posisjon med navn og farge')

// 5: cursor-koordinater clampes til [0,1]
a.send({ type: 'cursor', tile: `${FLATE}:web`, x: 5, y: -3 })
const clamped = await b.neste('cursor')
if (clamped.x !== 1 || clamped.y !== 0) feil(`koordinater ikke clampet: ${clamped.x},${clamped.y}`)
ok('cursor-koordinater clampes til [0,1]')

// 6: watch-socket får roster-snapshot uten å joine
const vegg = await klient('vegg', { watch: true })
const veggSnapshot = await vegg.neste('roster')
if (veggSnapshot.deltagere.length !== 2) feil('veggen fikk ikke roster-snapshot')
ok('watch-tilkobling får roster-snapshot uten join')

// Hjelper: ferskt roster via en engangs watch-tilkobling (snapshot).
async function ferskRoster() {
  const ekstra = await klient('snapshot', { watch: true })
  const r = await ekstra.neste('roster')
  ekstra.ws.close()
  return r
}

// 7: watch kan ikke joine
vegg.send({ type: 'join', navn: 'Snikjoiner' })
await vent(400)
if (vegg.meldinger.some((m) => m.type === 'velkommen')) feil('veggen fikk velkommen!')
if ((await ferskRoster()).deltagere.length !== 2) feil('watch-join endret roster')
ok('watch-tilkobling kan aldri joine (veggen er read-only)')

// 8: watch-cursor ignoreres
b.tomKo()
vegg.send({ type: 'cursor', tile: `${FLATE}:editor`, x: 0.5, y: 0.5 })
await vent(400)
if (b.meldinger.some((m) => m.type === 'cursor')) feil('veggens cursor ble broadcastet!')
ok('watch-cursor ignoreres')

// 9: B tar kontroll over As flate
b.send({ type: 'ta', flate: FLATE })
const rosterTa = await b.roster()
if (rosterTa.kontroll[FLATE]?.id !== velkommenB.deg.id) feil('kontrollen ble ikke registrert')
ok(`B tok kontroll over ${FLATE} — broadcastet i roster`)

// 10: konflikt — Gamma kan ikke ta samme flate
const c = await klient('C')
c.send({ type: 'join', navn: 'Gamma' })
await c.neste('velkommen')
c.send({ type: 'ta', flate: FLATE })
const feilC = await c.neste('feil')
if (!feilC.melding.includes('Beta')) feil(`uventet feilmelding: ${feilC.melding}`)
ok('kun ÉN ekstern controller: Gamma avvises mens Beta kontrollerer')

// 11: eieren kan ikke «ta» sin egen flate
a.send({ type: 'ta', flate: FLATE })
const feilA = await a.neste('feil')
if (!feilA.melding.includes('egen')) feil(`uventet feilmelding: ${feilA.melding}`)
ok('eieren avvises (har alltid kontroll selv)')

// 12: ukjent flate avvises
b.send({ type: 'ta', flate: 'finnes-ikke' })
const feilUkjent = await b.neste('feil')
if (!feilUkjent.melding.includes('ukjent')) feil(`uventet feilmelding: ${feilUkjent.melding}`)
ok('ta på ukjent flate avvises (flaten må finnes i prosjektet)')

// 13: slipp broadcastes
a.tomKo()
b.send({ type: 'slipp' })
const rosterSlipp = await a.neste('roster')
if (rosterSlipp.kontroll[FLATE]) feil('kontrollen henger igjen etter slipp')
ok('slipp → kontrollen borte fra roster hos alle')

// 14: reconnect med session-token → samme identitet, ingen duplikat
a.ws.close()
await vent(300)
const a2 = await klient('A2')
a2.send({ type: 'join', navn: 'Alfa', session: velkommenA.session })
const velkommenA2 = await a2.neste('velkommen')
if (velkommenA2.deg.id !== velkommenA.deg.id) feil('reconnect ga ny identitet')
if (velkommenA2.deg.farge !== velkommenA.deg.farge) feil('reconnect mistet fargen')
ok('reconnect med session-token → samme id og farge')

// 15: generasjonstelleren avbryter oppryddingen (timeout er 2 s i testen)
await vent(3000)
const rosterEtterVindu = await ferskRoster()
if (!rosterEtterVindu.deltagere.some((d) => d.id === velkommenA.deg.id && d.tilkoblet))
  feil('A ble ryddet bort tross reconnect — generasjonstelleren virker ikke')
if (rosterEtterVindu.deltagere.length !== 3) feil(`ventet 3 deltagere, så ${rosterEtterVindu.deltagere.length}`)
ok('rask reconnect avbryter oppryddingen (generasjonsteller)')

// 16: brå frakobling uten reconnect → fjernes etter timeout, kontroll slippes
b.send({ type: 'ta', flate: FLATE })
await b.roster()
b.ws.close()
await vent(3500) // timeout 2 s + margin
const rosterEtterTimeout = await ferskRoster()
if (rosterEtterTimeout.deltagere.some((d) => d.navn === 'Beta'))
  feil('Beta henger igjen etter disconnect-timeout')
if (rosterEtterTimeout.kontroll[FLATE]) feil('Betas kontroll ble ikke sluppet ved opprydding')
ok('disconnect-timeout rydder deltager OG slipper kontrollen')

console.log('\nPRESENCE-SMOKETESTEN ER GRØNN (16 punkter) 🎉')
process.exit(0)
