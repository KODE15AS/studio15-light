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
  type Prosjekt = {
    slug: string
    navn: string
    repo: string
    mal: string
    arbeidsflater: Arbeidsflate[]
  }
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
  let nyttProsjektMal = $state('full')
  let visNyttProsjekt = $state(false)

  // Prosjektmalene (Jørn 05.10, testrapport 3) — navnene er Jørns.
  const maler = [
    {
      id: 'full',
      navn: 'Full Zoo Code UI og minimal konfetti-webside',
      hjelp: 'Svelte + Vite med levende webside og hele editoren synlig.',
    },
    {
      id: 'nybegynner',
      navn: 'Nybegynner for enkle spill og samarbeide',
      hjelp: 'Ryddet skjerm: websiden øverst, spilloppgave i fire trinn og hjelperen under.',
    },
    {
      id: 'ekspert',
      navn: 'Ekspert for profesjonelle prosjekter',
      hjelp:
        'Ryddet skjerm uten føringer: agenten kjenner raven-miljøet, prosjektbriefen styrer. Dokumenter lastes opp fra skjermen.',
    },
  ]

  // Samme slugify som lobbyen (validering vises live — Jørn 05.10):
  // navnet blir en adresse, så æ/ø/å og tegn oversettes.
  const slugifyNavn = (s: string) =>
    s
      .toLowerCase()
      .replaceAll('æ', 'ae')
      .replaceAll('ø', 'oe')
      .replaceAll('å', 'aa')
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
  const nySlug = $derived(slugifyNavn(nyttProsjektNavn))
  // Vakta i lobbyen: gruppe + prosjekt ≤ 47 tegn (DNS-grensen med plass
  // til deltagernavn). Speiles her så feilen vises FØR man trykker.
  const maksSlug = $derived(gruppen ? 47 - gruppen.slug.length - 1 : 46)

  const gruppen = $derived(grupper.find((g) => g.slug === gruppe) ?? null)

  // Ett prosjekt = én deltagerskjerm (Jørn 08.10): prosjektets «eier» er
  // deltageren bak den (eneste) eksisterende skjermen.
  const eierAv = (prosjekt: Prosjekt) => prosjekt.arbeidsflater[0]?.deltager ?? null

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
    await kall('POST', '/api/prosjekter', {
      program: gruppen.slug,
      navn: nyttProsjektNavn,
      mal: nyttProsjektMal,
    })
    nyttProsjektNavn = ''
    nyttProsjektMal = 'full'
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
    // Én bekreftelsesknapp (Jørn 05.10, rapport 4 pkt. 2) — avskrift av
    // sluggen utgikk; API-ets bekreft-felt fylles av frontenden.
    await kall('DELETE', `/api/prosjekter/${gruppen.slug}/${prosjekt.slug}`, {
      bekreft: prosjekt.slug,
    })
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
            <!-- Malvelger (Jørn 05.10, testrapport 3) -->
            <div class="malvalg">
              {#each maler as m}
                <label class="mal" class:valgt={nyttProsjektMal === m.id}>
                  <input type="radio" name="mal" value={m.id} bind:group={nyttProsjektMal} />
                  <strong>{m.navn}</strong>
                  <span>{m.hjelp}</span>
                </label>
              {/each}
            </div>
            <form
              class="rad"
              onsubmit={(e) => {
                e.preventDefault()
                lagProsjekt()
              }}
            >
              <input placeholder="Prosjektnavn" bind:value={nyttProsjektNavn} />
              <button
                class="k15-btn k15-btn-primary"
                disabled={opptatt || !nySlug || nySlug.length > maksSlug}>Opprett</button
              >
            </form>
            <!-- Valideringstekst (Jørn 05.10): navnet blir en adresse -->
            <p class="hint" class:ugyldig={nySlug.length > maksSlug}>
              Bokstaver, tall og mellomrom er lov — i adressen blir æ/ø/å til
              ae/oe/aa og andre tegn til bindestrek.
              {#if nySlug}
                Adressen blir <code>{nySlug}</code> ({nySlug.length} av maks
                {maksSlug} tegn{nySlug.length > maksSlug
                  ? ' — for langt, velg et kortere navn'
                  : ''}).
              {:else}
                Maks {maksSlug} tegn i adressen.
              {/if}
            </p>
          </div>
        {/if}

        <div class="prosjekter">
          {#each gruppen.prosjekter as prosjekt}
            <div class="k15-card prosjekt">
              <span class="k15-nummer"
                >PROSJEKT{prosjekt.mal && prosjekt.mal !== 'full'
                    ? ` · ${prosjekt.mal.toUpperCase()}`
                    : ''}</span
              >
              <h3>{prosjekt.navn}</h3>

              {#if prosjekt.arbeidsflater.length > 0}
                <ul class="flater">
                  <!-- Eierstyring (Jørn 08.10): kun eieren ser Samling/
                       Editor/Stopp/Vekk — andre kan ALDRI starte eller
                       styre en annens skjerm. Websiden (gjest) er åpen
                       for alle når flaten kjører (regel 0: URL = adgang). -->
                  {#each prosjekt.arbeidsflater as flate}
                    {@const erMin = valgtDeltager?.slug === flate.deltager}
                    <li>
                      <span
                        class="status"
                        class:paa={flate.kjorer}
                        style={flate.farge && flate.kjorer ? `background: ${flate.farge}` : ''}
                      ></span>
                      <strong>{flate.deltager}</strong>
                      {#if flate.kjorer}
                        {#if erMin}
                          <a
                            href="/samling/{gruppen.slug}/{prosjekt.slug}/?deltager={encodeURIComponent(
                              flate.deltager
                            )}"
                            target="_blank">Samling</a
                          >
                          <a href={flate.editor_url} target="_blank">Editor</a>
                        {/if}
                        <!-- Eieren får spiller-parameteren med (08.10):
                             uten den tror spillet at eieren er gjest.
                             Andre får ren gjeste-URL (regel 0). -->
                        <a
                          href={erMin
                            ? `${flate.web_url}?spiller=${encodeURIComponent(valgtDeltager.navn)}`
                            : flate.web_url}
                          target="_blank">Webside</a
                        >
                        {#if erMin}
                          <button
                            class="lenkeknapp"
                            onclick={() => kall('POST', `/api/arbeidsflater/${flate.kortnavn}/stopp`)}
                            >Stopp</button
                          >
                        {/if}
                      {:else}
                        <span class="sover">i dvale</span>
                        {#if erMin}
                          <button
                            class="lenkeknapp"
                            onclick={() => kall('POST', `/api/arbeidsflater/${flate.kortnavn}/vekk`)}
                            >Vekk</button
                          >
                        {/if}
                      {/if}
                    </li>
                  {/each}
                </ul>
              {/if}

              <!-- Ett prosjekt = én deltagerskjerm (Jørn 08.10): tilhører
                   prosjektet en annen, vises veien inn via websiden i
                   stedet for åpne-knappen. API-et håndhever det samme. -->
              {#if valgtDeltager && eierAv(prosjekt) && eierAv(prosjekt) !== valgtDeltager.slug}
                <p class="hint">
                  Prosjektet tilhører <strong>{eierAv(prosjekt)}</strong> — bli
                  med via websiden over, eller lag et eget prosjekt.
                </p>
              {:else if valgtDeltager}
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
                Ett prosjekt har én deltagerskjerm — og en deltager er i ett
                prosjekt om gangen. Åpner du skjermen din her, flytter du
                deg (og tavla) hit; det forrige prosjektet ditt står urørt
                og kan åpnes igjen når som helst.
              </p>

              <!-- Sletting er eierens (Jørn 08.10): synlig kun for eieren
                   — eller for alle når prosjektet ikke har noen skjerm
                   ennå (tomt skall). Admin går via API/terminal. -->
              {#if !eierAv(prosjekt) || valgtDeltager?.slug === eierAv(prosjekt)}
              <details class="slett">
                <summary>Slett prosjektet</summary>
                <p>
                  Sletter skjermene, volumene og prosjektrepoet i én
                  operasjon — dette kan ikke angres.
                </p>
                <button
                  class="k15-btn k15-btn-secondary"
                  disabled={opptatt}
                  onclick={() => slettProsjekt(prosjekt)}
                >
                  Bekreft sletting av prosjektet
                </button>
              </details>
              {/if}
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
  .hint.ugyldig {
    color: #b3443f;
  }
  .malvalg {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 10px;
    margin-top: 12px;
  }
  .mal {
    display: flex;
    flex-direction: column;
    gap: 4px;
    border: 1px solid var(--k15-linje);
    border-radius: 10px;
    padding: 10px 14px;
    cursor: pointer;
    background: var(--k15-hvit);
  }
  .mal.valgt {
    border-color: var(--k15-blaagraa-mork);
    box-shadow: 0 0 0 1px var(--k15-blaagraa-mork);
  }
  .mal input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .mal strong {
    font-size: 13.5px;
  }
  .mal span {
    font-size: 12px;
    color: var(--k15-noytralgraa);
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
