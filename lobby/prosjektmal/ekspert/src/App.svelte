<script lang="ts">
  // Klikktelleren er bevisst med (Jørn 09.10): en enkel, ufarlig test på
  // at websiden er levende og oppdaterer seg.
  let antall = $state(0)
</script>

<main>
  <div class="kort">
    <p class="kicker">KODE15 · Studio 15 Light</p>
    <h1>__PROSJEKTNAVN__</h1>
    <p class="tekst">
      Denne websiden er levende: den viser alltid den nyeste utgaven av
      det som bygges i prosjektet. Fortell Agenten i dialogen hva du vil
      lage — endringene dukker opp her med én gang.
    </p>
    <button onclick={() => (antall += 1)}>
      Du har klikket {antall} gang{antall === 1 ? '' : 'er'}
    </button>
    <p class="hint">
      Knappen er bare en liten test på at siden lever — klikk i vei.
    </p>
  </div>
</main>

<style>
  /* KODE15-webprofilen (norm «web-profil», fasit raven-platform/web-profil):
     designtokens herfra — byttes gjerne ut når prosjektet får eget uttrykk. */
  @import url('https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500&display=swap');

  :global(body) {
    margin: 0;
    background: #f7f4ef; /* k15-bg */
  }
  main {
    min-height: 100vh;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 10vh 24px 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    font-size: 15px;
    line-height: 1.6;
    color: #959593; /* k15-noytralgraa */
    box-sizing: border-box;
  }
  .kort {
    background: #ffffff;
    border: 1px solid #dbdbdb; /* k15-linje */
    border-radius: 14px;
    padding: 28px 34px 30px;
    max-width: 560px;
  }
  .kicker {
    font-family: 'Montserrat', 'Segoe UI', system-ui, sans-serif;
    font-size: 11px;
    letter-spacing: 2px;
    text-transform: uppercase;
    color: #959593;
    margin: 0 0 6px;
  }
  h1 {
    font-family: 'Montserrat', 'Segoe UI', system-ui, sans-serif;
    font-weight: 400;
    color: #525d65; /* k15-blaagraa-mork */
    line-height: 1.2;
    margin: 0 0 14px;
  }
  .tekst {
    color: #233038; /* k15-skifer */
    margin: 0 0 20px;
  }
  button {
    border: none;
    border-radius: 999px;
    padding: 10px 22px;
    background: #bbad9a; /* k15-beige */
    color: #ffffff;
    font-family: 'Montserrat', 'Segoe UI', system-ui, sans-serif;
    font-size: 12px;
    letter-spacing: 0.4px;
    text-transform: uppercase;
    cursor: pointer;
  }
  button:hover {
    filter: brightness(0.95);
  }
  .hint {
    font-size: 12.5px;
    margin: 12px 0 0;
  }
</style>
