<script lang="ts">
  // Lobbyen: programmer (GitHub-orger) → prosjekter → arbeidsflater.
  // Design: webprofil-kode15 (norm «web-profil»).

  type Arbeidsflate = {
    deltager: string
    kortnavn: string
    kjorer: boolean
    editor_url: string
    web_url: string
  }
  type Prosjekt = { slug: string; navn: string; repo: string; arbeidsflater: Arbeidsflate[] }
  type Program = { slug: string; navn: string; github_org: string | null; prosjekter: Prosjekt[] }

  let programmer: Program[] = $state([])
  let valgtProgram: string | null = $state(null)
  let feilmelding = $state('')
  let opptatt = $state(false)

  // Skjemafelter
  let nyttProgramNavn = $state('')
  let visOrgFlyt = $state(false)
  let nyttProsjektNavn = $state('')
  let deltagerNavn: Record<string, string> = $state({})
  let slettBekreft: Record<string, string> = $state({})

  const programmet = $derived(programmer.find((p) => p.slug === valgtProgram) ?? null)

  async function hent() {
    try {
      const r = await fetch('/api/tilstand')
      if (!r.ok) throw new Error(await r.text())
      const data = await r.json()
      programmer = data.programmer
      feilmelding = ''
    } catch (e) {
      feilmelding = 'Får ikke kontakt med lobbyen — last siden på nytt for å prøve igjen.'
    }
  }

  async function kall(metode: string, sti: string, kropp?: unknown): Promise<any> {
    opptatt = true
    try {
      const r = await fetch(sti, {
        method: metode,
        headers: { 'Content-Type': 'application/json' },
        body: kropp === undefined ? undefined : JSON.stringify(kropp),
      })
      const data = await r.json().catch(() => ({}))
      if (!r.ok) throw new Error(data.feil ?? `${r.status}`)
      feilmelding = ''
      await hent()
      return data
    } catch (e: any) {
      feilmelding = e.message ?? 'Noe gikk galt — prøv igjen.'
      throw e
    } finally {
      opptatt = false
    }
  }

  async function lagProgram() {
    if (!nyttProgramNavn.trim()) return
    await kall('POST', '/api/programmer', { navn: nyttProgramNavn })
    nyttProgramNavn = ''
    visOrgFlyt = false
  }

  async function lagProsjekt() {
    if (!programmet || !nyttProsjektNavn.trim()) return
    await kall('POST', '/api/prosjekter', { program: programmet.slug, navn: nyttProsjektNavn })
    nyttProsjektNavn = ''
  }

  async function aapneArbeidsflate(prosjekt: Prosjekt) {
    const navn = (deltagerNavn[prosjekt.slug] ?? '').trim()
    if (!programmet || !navn) return
    const svar = await kall('POST', '/api/arbeidsflater', {
      program: programmet.slug,
      prosjekt: prosjekt.slug,
      deltager: navn,
    })
    window.open(svar.editor_url, '_blank')
  }

  async function slettProsjekt(prosjekt: Prosjekt) {
    if (!programmet) return
    await kall('DELETE', `/api/prosjekter/${programmet.slug}/${prosjekt.slug}`, {
      bekreft: slettBekreft[prosjekt.slug] ?? '',
    })
    slettBekreft[prosjekt.slug] = ''
  }

  hent()
  setInterval(hent, 5000)
</script>

<div class="k15-page side">
  <header class="k15-header">
    <img class="k15-logo" src="/kode15-logo.png" alt="KODE15" />
    <div>
      <span class="k15-kicker">Studio 15 LIGHT</span>
      <h1 class="tittel">Lobby</h1>
    </div>
  </header>

  <main class="innhold">
    {#if feilmelding}
      <div class="k15-card varsel">{feilmelding}</div>
    {/if}

    {#if !programmet}
      <section>
        <span class="k15-kicker">Programmer</span>
        <h2>Velg program</h2>
        <p>
          Et program er overbygningen for en samling prosjekter — en egen
          GitHub-organisasjon. Prosjektene under et program deles av begge
          deltagerne.
        </p>
        <div class="k15-ruter">
          {#each programmer as p, i}
            <a
              class="k15-rute"
              href="#{p.slug}"
              onclick={(e) => {
                e.preventDefault()
                valgtProgram = p.slug
              }}
            >
              <span class="k15-nummer">{String(i + 1).padStart(2, '0')}</span>
              <h3>{p.navn}</h3>
              <p>
                {p.prosjekter.length} prosjekt{p.prosjekter.length === 1 ? '' : 'er'}
                {#if !p.github_org}· GitHub-org mangler{/if}
              </p>
            </a>
          {/each}
          <button class="k15-rute ny" onclick={() => (visOrgFlyt = !visOrgFlyt)}>
            <span class="k15-nummer">+</span>
            <h3>Nytt program</h3>
            <p>Guidet oppretting med manuelle GitHub-steg</p>
          </button>
        </div>

        {#if visOrgFlyt}
          <div class="k15-card orgflyt">
            <span class="k15-kicker">Guidet flyt</span>
            <h3>Nytt program (GitHub-org)</h3>
            <p>
              GitHub har ikke API for å opprette organisasjoner, så selve
              org-en lages manuelt — resten håndterer lobbyen:
            </p>
            <ol>
              <li>
                Gå til <strong>github.com → ikonet øverst til høyre →
                Settings → Organizations → New organization</strong> (Free).
              </li>
              <li>Org-navn: bruk programnavnet med KODE15-prefiks, f.eks. <code>KODE15-&lt;program&gt;</code>.</li>
              <li>Eier: KODE15-kontoen. Ikke inviter medlemmer.</li>
              <li>
                Installer vaktmester-appen på org-en (se
                <code>docs/vaktmester-klikkeliste.md</code>) — da oppretter og
                sletter lobbyen prosjektrepoene i org-en automatisk.
              </li>
            </ol>
            <p>
              Registrer programmet her — org-navnet kan legges til i
              <code>register/programmer.yaml</code> når org-en er laget:
            </p>
            <form
              onsubmit={(e) => {
                e.preventDefault()
                lagProgram()
              }}
            >
              <input placeholder="Programnavn" bind:value={nyttProgramNavn} />
              <button class="k15-btn k15-btn-primary" disabled={opptatt}>Registrer program</button>
            </form>
          </div>
        {/if}
      </section>
    {:else}
      <section>
        <button class="k15-btn k15-btn-secondary tilbake" onclick={() => (valgtProgram = null)}>
          ← Alle programmer
        </button>
        <span class="k15-kicker">Program</span>
        <h2>{programmet.navn}</h2>
        {#if !programmet.github_org}
          <p class="hint">
            Programmet har ingen GitHub-org — nye prosjekter får lokale
            git-repoer på raven. (Repo-typen velges ved opprettelse; med org
            lager vaktmesteren private GitHub-repoer automatisk.)
          </p>
        {/if}

        <div class="prosjekter">
          {#each programmet.prosjekter as prosjekt}
            <div class="k15-card prosjekt">
              <span class="k15-nummer">PROSJEKT</span>
              <h3>{prosjekt.navn}</h3>

              {#if prosjekt.arbeidsflater.length > 0}
                <ul class="flater">
                  {#each prosjekt.arbeidsflater as flate}
                    <li>
                      <span class="status" class:paa={flate.kjorer}></span>
                      <strong>{flate.deltager}</strong>
                      {#if flate.kjorer}
                        <a href={flate.editor_url} target="_blank">Arbeidsflate</a>
                        <a href={flate.web_url} target="_blank">Webside</a>
                        <button
                          class="lenkeknapp"
                          onclick={() => kall('POST', `/api/arbeidsflater/${flate.kortnavn}/stopp`)}
                          >Stopp</button
                        >
                      {:else}
                        <span class="sover">i dvale</span>
                        <button
                          class="lenkeknapp"
                          onclick={() => kall('POST', `/api/arbeidsflater/${flate.kortnavn}/vekk`)}
                          >Vekk</button
                        >
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}

              <form
                class="rad"
                onsubmit={(e) => {
                  e.preventDefault()
                  aapneArbeidsflate(prosjekt)
                }}
              >
                <input placeholder="Deltagernavn" bind:value={deltagerNavn[prosjekt.slug]} />
                <button class="k15-btn k15-btn-primary" disabled={opptatt}>
                  Åpne min arbeidsflate
                </button>
              </form>
              <p class="hint">
                To deltagere kan dele prosjektet med hver sin arbeidsflate —
                eller jobbe i samme flate ved å åpne samme adresse.
              </p>

              <details class="slett">
                <summary>Slett prosjektet</summary>
                <p>
                  Sletter arbeidsflatene, volumene og prosjektrepoet i én
                  operasjon. Skriv prosjektets slug
                  (<code>{prosjekt.slug}</code>) for å bekrefte:
                </p>
                <form
                  class="rad"
                  onsubmit={(e) => {
                    e.preventDefault()
                    slettProsjekt(prosjekt)
                  }}
                >
                  <input placeholder={prosjekt.slug} bind:value={slettBekreft[prosjekt.slug]} />
                  <button
                    class="k15-btn k15-btn-secondary"
                    disabled={opptatt || slettBekreft[prosjekt.slug] !== prosjekt.slug}
                  >
                    Slett for alltid
                  </button>
                </form>
              </details>
            </div>
          {/each}

          <div class="k15-card prosjekt nytt">
            <span class="k15-nummer">+</span>
            <h3>Nytt prosjekt</h3>
            <p>Starter fra malen: Svelte + Vite med levende webside og Zoo Code.</p>
            <form
              class="rad"
              onsubmit={(e) => {
                e.preventDefault()
                lagProsjekt()
              }}
            >
              <input placeholder="Prosjektnavn" bind:value={nyttProsjektNavn} />
              <button class="k15-btn k15-btn-primary" disabled={opptatt}>Opprett</button>
            </form>
          </div>
        </div>
      </section>
    {/if}
  </main>

  <footer class="k15-footer-bottom">Studio 15 LIGHT · KODE15 · kun Tailscale</footer>
</div>

<style>
  .side {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
  .tittel {
    margin: 0;
    font-size: 22px;
  }
  .innhold {
    flex: 1;
    width: min(1040px, 92vw);
    margin: 0 auto;
    padding: 32px 0 48px;
  }
  .varsel {
    border-color: #bbad9a;
    margin-bottom: 20px;
  }
  .k15-rute.ny {
    text-align: left;
    font: inherit;
    cursor: pointer;
    border-style: dashed;
  }
  .orgflyt {
    margin-top: 24px;
  }
  .orgflyt form,
  .rad {
    display: flex;
    gap: 10px;
    margin-top: 12px;
    flex-wrap: wrap;
  }
  input {
    flex: 1;
    min-width: 160px;
    border: 1px solid var(--k15-linje);
    border-radius: 8px;
    padding: 9px 12px;
    font: inherit;
    background: var(--k15-hvit);
    color: var(--k15-tekst);
  }
  .tilbake {
    margin-bottom: 18px;
  }
  .prosjekter {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 20px;
    margin-top: 18px;
  }
  .prosjekt.nytt {
    border-style: dashed;
  }
  .flater {
    list-style: none;
    padding: 0;
    margin: 10px 0;
  }
  .flater li {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 6px 0;
    border-bottom: 1px solid var(--k15-flate);
  }
  .status {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--k15-linje);
    align-self: center;
  }
  .status.paa {
    background: #7ca982;
  }
  .sover {
    font-size: 12px;
    color: var(--k15-noytralgraa);
  }
  .lenkeknapp {
    background: none;
    border: none;
    color: var(--k15-blaagraa-mork);
    text-decoration: underline;
    cursor: pointer;
    font: inherit;
    font-size: 13px;
    padding: 0;
  }
  .hint {
    font-size: 12.5px;
    color: var(--k15-noytralgraa);
    margin: 8px 0 0;
  }
  .slett {
    margin-top: 14px;
    font-size: 13px;
  }
  .slett summary {
    cursor: pointer;
    color: var(--k15-blaagraa-lys);
  }
</style>
