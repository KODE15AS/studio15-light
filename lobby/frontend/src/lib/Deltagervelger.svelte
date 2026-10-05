<script>
  // Deltagervelgeren (Jørn 04.10): øverst på startsiden. IdentitetsVALG,
  // ikke innlogging — åpen tabell uten credentials (bevisst unntak fra
  // grunnlagsdokumentene, se handover 04.10). Valget huskes i nettleseren
  // og brukes når skjermer åpnes; fargen ble tildelt ved registrering og
  // er fast overalt (presence, tavle, startside).
  //
  // deltagere: listen fra /api/tilstand (lobbyen poller hvert 5. sekund).
  // valgt (bindable): valgt deltager-objekt eller null.
  let { deltagere = [], valgt = $bindable(null) } = $props()

  const LAGER = 's15l.deltager'

  let aapen = $state(false)
  let rot = $state(null) // wrapperen — klikk utenfor den lukker panelet
  let nyttNavn = $state('')
  let feil = $state('')
  let opptatt = $state(false)

  // Gjenopprett valget fra nettleseren når listen er på plass; hold
  // navn/farge i sync med registeret (sluggen er nøkkelen).
  $effect(() => {
    const slug = localStorage.getItem(LAGER)
    if (!slug) return
    const d = deltagere.find((d) => d.slug === slug)
    if (d) valgt = d
  })

  function velg(d) {
    valgt = d
    localStorage.setItem(LAGER, d.slug)
    aapen = false
    feil = ''
  }

  async function registrer() {
    const navn = nyttNavn.trim()
    if (!navn) return
    opptatt = true
    try {
      const r = await fetch('/api/deltagere', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ navn }),
      })
      const data = await r.json().catch(() => ({}))
      if (!r.ok) throw new Error(data.feil ?? `${r.status}`)
      nyttNavn = ''
      velg(data.deltager)
    } catch (e) {
      feil = e.message ?? 'Noe gikk galt — prøv igjen.'
    } finally {
      opptatt = false
    }
  }
</script>

<svelte:window
  onpointerdown={(e) => {
    // Klikk utenfor lukker panelet (Jørn 05.10, rapport 4 pkt. 1).
    if (aapen && rot && !rot.contains(e.target)) aapen = false
  }}
/>

<div class="velger" bind:this={rot}>
  <!-- Tydelig kontekst (Jørn 05.10): etiketten «Deltager:» + «ingen
       valgt» når ingen er valgt. -->
  {#if valgt}
    <button class="chip" onclick={() => (aapen = !aapen)} title="Bytt deltager">
      <span class="etikett">Deltager:</span>
      <span class="prikk" style="background: {valgt.farge}"></span>
      {valgt.navn}
      <span class="bytt">bytt</span>
    </button>
  {:else}
    <button class="k15-btn k15-btn-primary" onclick={() => (aapen = !aapen)}>
      Deltager: ingen valgt
    </button>
  {/if}

  {#if aapen}
    <div class="panel k15-card">
      <button class="lukk" title="Lukk" onclick={() => (aapen = false)}>×</button>
      <span class="k15-kicker">Hvem er du?</span>
      {#if deltagere.length > 0}
        <ul>
          {#each deltagere as d (d.slug)}
            <li>
              <button class="valg" class:aktiv={valgt?.slug === d.slug} onclick={() => velg(d)}>
                <span class="prikk" style="background: {d.farge}"></span>
                {d.navn}
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="tom">Ingen deltagere registrert ennå — registrer deg under.</p>
      {/if}

      <form
        onsubmit={(e) => {
          e.preventDefault()
          registrer()
        }}
      >
        <input placeholder="Registrer ny deltager" bind:value={nyttNavn} />
        <button class="k15-btn k15-btn-secondary" disabled={opptatt || !nyttNavn.trim()}>
          Registrer
        </button>
      </form>
      {#if feil}<p class="feil">{feil}</p>{/if}
    </div>
  {/if}
</div>

<style>
  .velger {
    position: relative;
    margin-left: auto;
  }
  .chip {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--k15-hvit);
    border: 1px solid var(--k15-linje);
    border-radius: 999px;
    padding: 7px 14px;
    font: inherit;
    font-weight: 600;
    color: var(--k15-tekst);
    cursor: pointer;
  }
  .chip .etikett {
    font-weight: 400;
    color: var(--k15-noytralgraa);
  }
  .lukk {
    position: absolute;
    top: 8px;
    right: 10px;
    background: none;
    border: none;
    font-size: 20px;
    line-height: 1;
    color: var(--k15-noytralgraa);
    cursor: pointer;
    padding: 4px;
  }
  .lukk:hover {
    color: var(--k15-tekst);
  }
  .chip .bytt {
    font-size: 11.5px;
    font-weight: 400;
    color: var(--k15-noytralgraa);
    text-decoration: underline;
  }
  .prikk {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    flex: none;
  }
  .panel {
    position: absolute;
    right: 0;
    top: calc(100% + 8px);
    width: min(320px, 86vw);
    z-index: 30;
    box-shadow: 0 14px 40px rgba(78, 71, 64, 0.18);
  }
  .panel ul {
    list-style: none;
    margin: 10px 0 14px;
    padding: 0;
    max-height: 240px;
    overflow-y: auto;
  }
  .valg {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    background: none;
    border: none;
    border-radius: 8px;
    padding: 8px 10px;
    font: inherit;
    color: var(--k15-tekst);
    cursor: pointer;
    text-align: left;
  }
  .valg:hover {
    background: var(--k15-flate);
  }
  .valg.aktiv {
    background: var(--k15-flate);
    font-weight: 600;
  }
  .panel form {
    display: flex;
    gap: 8px;
  }
  .panel input {
    flex: 1;
    min-width: 0;
    border: 1px solid var(--k15-linje);
    border-radius: 8px;
    padding: 8px 10px;
    font: inherit;
    background: var(--k15-hvit);
    color: var(--k15-tekst);
  }
  .tom {
    font-size: 13px;
    color: var(--k15-noytralgraa);
  }
  .feil {
    color: #a33d5e;
    font-size: 13px;
    margin: 8px 0 0;
  }
</style>
