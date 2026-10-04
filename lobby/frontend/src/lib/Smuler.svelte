<script>
  // Brødsmuler i GitHub-stil (Jørn 04.10): sti-ledd adskilt av «/», siste
  // ledd er nåværende side (halvfet, ingen lenke). Default-mønster i
  // KODE15s web-profil — se webprofil-kode15/kode15-profil.md.
  //
  // deler: [{ navn, href }] — siste ledd trenger ikke href.
  let { deler = [] } = $props()
</script>

<nav class="k15-smuler" aria-label="Brødsmuler">
  {#each deler as d, i}
    {#if i > 0}<span class="skille" aria-hidden="true">/</span>{/if}
    {#if i < deler.length - 1 && d.href}
      <a href={d.href}>{d.navn}</a>
    {:else}
      <span class="naa" aria-current="page">{d.navn}</span>
    {/if}
  {/each}
</nav>

<style>
  .k15-smuler {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    flex-wrap: wrap;
  }
  .k15-smuler a {
    color: var(--k15-blaagraa-mork);
    text-decoration: none;
  }
  .k15-smuler a:hover {
    text-decoration: underline;
  }
  .skille {
    color: var(--k15-noytralgraa);
  }
  .naa {
    color: var(--k15-tekst);
    font-weight: 600;
  }
</style>
