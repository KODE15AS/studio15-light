<script lang="ts">
  let antall = $state(0)
</script>

<main>
  <h1>__PROSJEKTNAVN__</h1>
  <p>
    Websiden din lever: alt du og assistenten endrer i <code>src/</code>
    vises her med én gang.
  </p>
  <button onclick={() => (antall += 1)}>
    Du har klikket {antall} gang{antall === 1 ? '' : 'er'}
  </button>
</main>

<style>
  main {
    font-family: -apple-system, 'Segoe UI', Roboto, sans-serif;
    max-width: 640px;
    margin: 10vh auto 0;
    padding: 0 24px;
    color: #233038;
  }
  h1 {
    font-weight: 400;
    color: #525d65;
  }
  button {
    border: none;
    border-radius: 999px;
    padding: 10px 22px;
    background: #bbad9a;
    color: #fff;
    font-size: 14px;
    cursor: pointer;
  }
  button:hover {
    filter: brightness(0.95);
  }
  code {
    background: #f4f5f7;
    padding: 1px 6px;
    border-radius: 4px;
  }
</style>
