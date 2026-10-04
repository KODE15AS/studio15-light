<script lang="ts">
  // Startsiden (tidl. «Lobby» — begrepet utgikk 04.10, Jørn):
  // prosjektgrupper (GitHub-orger) → prosjekter → deltagerskjermer.
  // Design: webprofil-kode15 (norm «web-profil»), brødsmuler i GitHub-stil.
  // To «sider» i samme komponent, skilt på URL:
  //   /                 startsiden (velg prosjektgruppe + deltagervelger)
  //   /gruppe/<slug>/   prosjektsiden for én gruppe
  import Smuler from '../lib/Smuler.svelte'
  import Deltagervelger from '../lib/Deltagervelger.svelte'

  type Arbeidsflate = {
    deltager: string
    farge: string | null
    kortnavn: string
    kjorer: boolean
    editor_url: string
    web_url: string
  }
  type Prosjekt = { slug: string; navn: string; repo: string; arbeidsflater: Arbeidsflate[] }
  type Gruppe = { slug: string; navn: string; github_org: string | null; prosjekter: Prosjekt[] }
  type Deltager = { slug: string; navn: string; farge: string; registrert: string }

  let { gruppe = null }: { gruppe?: string | null } = $props()

  let grupper: Gruppe[] = $state([])
  let deltagere: Deltager[] = $state([])
  let valgtDeltager: Deltager | null = $state(null)
  let feilmelding = $state('')
  let opptatt = $state(false)
  let lastet = $state(false)

  // Skjemafelter
  let nyGruppeNavn = $state('')
  let visOrgFlyt = $state(false)
  let nyttProsjektNavn = $state('')
  let visNyttProsjekt = $state(false)
  let slettBekreft: Record<string, string> = $state({})

  const gruppen = $derived(grupper.find((g) => g.slug === gruppe) ?? null)

  async function hent() {
    try {
      const r = await fetch('/api/tilstand')
      if (!r.ok) throw new Error(await r.text())
      const data = await r.json()
      grupper = data.programmer
      deltagere = data.deltagere ?? []
      feilmelding = ''
    } catch (e) {
      feilmelding = 'Får ikke kontakt med startsiden — last siden på nytt for å prøve igjen.'
    } finally {
      lastet = true
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

  async function lagGruppe() {
    if (!nyGruppeNavn.trim()) return
    await kall('POST', '/api/programmer', { navn: nyGruppeNavn })
    nyGruppeNavn = ''
    visOrgFlyt = false
  }

  async function lagProsjekt() {
    if (!gruppen || !nyttProsjektNavn.trim()) return
    await kall('POST', '/api/prosjekter', { program: gruppen.slug, navn: nyttProsjektNavn })
    nyttProsjektNavn = ''
    visNyttProsjekt = false
  }

  async function aapneSkjerm(prosjekt: Prosjekt) {
    if (!gruppen || !valgtDeltager) return
    await kall('POST', '/api/arbeidsflater', {
      program: gruppen.slug,
      prosjekt: prosjekt.slug,
      deltager: valgtDeltager.navn,
    })
    // Samlingsvisningen (V2): editor + webside + se/peke/ta over.
    window.open(
      `/samling/${gruppen.slug}/${prosjekt.slug}/?deltager=${encodeURIComponent(valgtDeltager.navn)}`,
      '_blank'
    )
  }

  async function slettProsjekt(prosjekt: Prosjekt) {
    if (!gruppen) return
    await kall('DELETE', `/api/prosjekter/${gruppen.slug}/${prosjekt.slug}`, {
      bekreft: slettBekreft[prosjekt.slug] ?? '',
    })
    slettBekreft[prosjekt.slug] = ''
  }

  hent()
  setInterval(hent, 5000)
</script>

<div class="k15-page side">
  <header class="k15-header">
    <!-- Logoen er alltid veien hjem; på startsiden virker den som forenklet
         F5 (nettbrett mangler lett tilgjengelig reload — Jørn 04.10). -->
    <a
      class="logolenke"
      href="/"
      title="Oppdater startsiden"
      onclick={(e) => {
        if (location.pathname === '/') {
          e.preventDefault()
          location.reload()
        }
      }}
    >
      <img class="k15-logo" src="/kode15-logo.png" alt="KODE15 — til startsiden" />
    </a>
    <div>
      <span class="k15-kicker">Studio 15 LIGHT</span>
      <h1 class="tittel">{gruppe ? 'Prosjekter' : 'Startside'}</h1>
    </div>
    <!-- Deltagervelgeren (04.10): identitetsvalg for hele økten — åpen
         tabell, ingen innlogging. -->
    <Deltagervelger {deltagere} bind:valgt={valgtDeltager} />
  </header>

  <main class="innhold">
    <Smuler
      deler={gruppen
        ? [{ navn: 'Startside', href: '/' }, { navn: gruppen.navn }]
        : [{ navn: 'Startside' }]}
    />

    {#if feilmelding}
      <div class="k15-card varsel">{feilmelding}</div>
    {/if}

    {#if !gruppe}
      <section>
        <span class="k15-kicker">Prosjektgrupper</span>
        <div class="topplinje">
          <h2>Velg prosjektgruppe</h2>
          <button class="k15-btn k15-btn-primary" onclick={() => (visOrgFlyt = !visOrgFlyt)}>
            Lag ny prosjektgruppe
          </button>
        </div>
        <p>
          En prosjektgruppe samler prosjekter som naturlig hører sammen, og
          opprettes på GitHub som en «organisasjon». Prosjektgruppenavnet bør
          være beskrivende for alle prosjektene i gruppen.
        </p>

        {#if visOrgFlyt}
          <div class="k15-card orgflyt">
            <span class="k15-kicker">Guidet flyt</span>
            <h3>Ny prosjektgruppe (GitHub-organisasjon)</h3>
            <p>
              GitHub har ikke API for å opprette organisasjoner, så selve
              org-en lages manuelt — resten håndterer startsiden:
            </p>
            <ol>
              <li>
                Gå til <strong>github.com → ikonet øverst til høyre →
                Settings → Organizations → New organization</strong> (Free).
              </li>
              <li>Org-navn: bruk gruppenavnet med KODE15-prefiks, f.eks. <code>KODE15-&lt;gruppe&gt;</code>.</li>
              <li>Eier: KODE15-kontoen. Ikke inviter medlemmer.</li>
              <li>
                Installer vaktmester-appen på org-en (se
                <code>docs/vaktmester-klikkeliste.md</code>) — da oppretter og
                sletter lobbytjenesten prosjektrepoene i org-en automatisk.
              </li>
            </ol>
            <p>
              Registrer prosjektgruppen her — org-navnet kan legges til i
              <code>register/programmer.yaml</code> når org-en er laget:
            </p>
            <form
              onsubmit={(e) => {
                e.preventDefault()
                lagGruppe()
              }}
            >
              <input placeholder="Navn på prosjektgruppen" bind:value={nyGruppeNavn} />
              <button class="k15-btn k15-btn-primary" disabled={opptatt}>Registrer</button>
            </form>
          </div>
        {/if}

        <div class="k15-ruter">
          {#each grupper as g, i}
            <a class="k15-rute" href="/gruppe/{g.slug}/">
              <span class="k15-nummer">{String(i + 1).padStart(2, '0')}</span>
              <h3>{g.navn}</h3>
              {#if g.prosjekter.length > 0}
                <!-- Prosjektnavnene direkte på flisen (04.10): inntil 4
                     synlige, skroller ved flere. Kun visning — veien inn
                     går via prosjektsiden. -->
                <ul class="prosjektliste" class:skroller={g.prosjekter.length > 4}>
                  {#each g.prosjekter as pr}
                    <li>{pr.navn}</li>
                  {/each}
                </ul>
              {:else}
                <p>Ingen prosjekter ennå</p>
              {/if}
              {#if !g.github_org}
                <p class="mangel">GitHub-org mangler</p>
              {/if}
            </a>
          {/each}
        </div>
      </section>
    {:else if gruppen}
      <section>
        <span class="k15-kicker">Prosjektgruppe</span>
        <div class="topplinje">
          <h2>Prosjektgruppe: {gruppen.navn}</h2>
          <button class="k15-btn k15-btn-primary" onclick={() => (visNyttProsjekt = !visNyttProsjekt)}>
            Lag nytt prosjekt
          </button>
        </div>
        {#if !gruppen.github_org}
          <p class="hint">
            Prosjektgruppen har ingen GitHub-org — nye prosjekter får lokale
            git-repoer på raven. (Repo-typen velges ved opprettelse; med org
            lager vaktmesteren private GitHub-repoer automatisk.)
          </p>
        {/if}

        {#if visNyttProsjekt}
          <div class="k15-card orgflyt">
            <span class="k15-kicker">Nytt prosjekt</span>
            <h3>Lag nytt prosjekt</h3>
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
        {/if}

        <div class="prosjekter">
          {#each gruppen.prosjekter as prosjekt}
            <div class="k15-card prosjekt">
              <span class="k15-nummer">PROSJEKT</span>
              <h3>{prosjekt.navn}</h3>

              {#if prosjekt.arbeidsflater.length > 0}
                <ul class="flater">
                  {#each prosjekt.arbeidsflater as flate}
                    <li>
                      <span
                        class="status"
                        class:paa={flate.kjorer}
                        style={flate.farge && flate.kjorer ? `background: ${flate.farge}` : ''}
                      ></span>
                      <strong>{flate.deltager}</strong>
                      {#if flate.kjorer}
                        <a
                          href="/samling/{gruppen.slug}/{prosjekt.slug}/?deltager={encodeURIComponent(
                            flate.deltager
                          )}"
                          target="_blank">Samling</a
                        >
                        <a href={flate.editor_url} target="_blank">Editor</a>
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

              {#if valgtDeltager}
                <button
                  class="k15-btn k15-btn-primary"
                  disabled={opptatt}
                  onclick={() => aapneSkjerm(prosjekt)}
                >
                  Åpne min skjerm ({valgtDeltager.navn})
                </button>
              {:else}
                <p class="hint">Velg deltager øverst til høyre for å åpne din skjerm.</p>
              {/if}
              <p class="hint">
                To deltagere kan dele prosjektet med hver sin skjerm — eller
                jobbe på samme skjerm ved å åpne samme adresse.
              </p>

              <details class="slett">
                <summary>Slett prosjektet</summary>
                <p>
                  Sletter skjermene, volumene og prosjektrepoet i én
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
        </div>
      </section>
    {:else if lastet}
      <div class="k15-card varsel">
        Fant ingen prosjektgruppe med adressen «{gruppe}» —
        <a href="/">tilbake til startsiden</a>.
      </div>
    {/if}
  </main>

  <footer class="k15-footer-bottom">
    Studio 15 LIGHT · KODE15 · <a class="tavlelenke" href="/tavle">Tavla</a>
  </footer>
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
    padding: 24px 0 48px;
  }
  .innhold :global(.k15-smuler) {
    margin-bottom: 18px;
  }
  .varsel {
    border-color: #bbad9a;
    margin-bottom: 20px;
  }
  .topplinje {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }
  .topplinje h2 {
    margin: 0;
  }
  .prosjektliste {
    list-style: none;
    margin: 0;
    padding: 0;
    font-size: 13.5px;
  }
  .prosjektliste li {
    padding: 2px 0;
    transition: color 0.25s;
  }
  /* Flere enn 4 prosjekter: fast høyde på 4 rader + skroller (04.10). */
  .prosjektliste.skroller {
    max-height: 96px;
    overflow-y: auto;
    padding-right: 6px;
  }
  /* Flisens hover gjør bakgrunnen mørk — prosjektlisten må følge med.
     (hover: hover): aldri hover-styling på berøringsskjermer, funn 04.10. */
  @media (hover: hover) {
    .k15-rute:hover .prosjektliste li {
      color: var(--k15-hvit);
    }
  }
  .mangel {
    font-size: 12px;
    margin: 8px 0 0;
  }
  .orgflyt {
    margin: 18px 0 6px;
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
  .prosjekter {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 20px;
    margin-top: 18px;
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
  .tavlelenke {
    color: inherit;
  }
</style>
