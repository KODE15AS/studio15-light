// Studio 15 LIGHT: åpner Zoo Code-chatten automatisk ved oppstart
// (lærdom fra Studio 15: brukerne fant ikke assistent-ikonet selv).
// View-ID verifisert 03.10: zoo-code.SidebarProvider → focus-kommandoen
// genereres av VS Code som "<view-id>.focus".
//
// Nybegynner-malen (Jørn 05.10, testrapport 3): absolutt all «støy» vekk —
// chatten åpnes som HELE editorflaten (zoo-code.openInNewTab, verifisert
// i utvidelsens package.json 05.10) og sidestolpen lukkes. Malen kommer
// som S15L_MAL i containermiljøet; extension-hosten arver det.
const vscode = require("vscode");
const fs = require("fs");
const path = require("path");

async function ryddNybegynner() {
  // Idempotent (kjøres flere ganger — første kjøring kan komme før
  // workbenchen er ferdig gjenoppbygd): åpne chatten bare én gang,
  // og rydd layouten hver gang.
  const harZooFane = vscode.window.tabGroups.all.some((g) =>
    g.tabs.some((t) => (t.label || "").toLowerCase().includes("zoo"))
  );
  if (!harZooFane) {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await vscode.commands.executeCommand("zoo-code.openInNewTab");
  }
  // openInNewTab legger chatten i en DELT kolonne (verifisert 05.10) —
  // én editorgruppe gjør chatten til hele flaten.
  await vscode.commands.executeCommand("workbench.action.editorLayoutSingle");
  await vscode.commands.executeCommand("workbench.action.closeSidebar");
  await vscode.commands.executeCommand("workbench.action.closePanel");
  // Bug 5 (Jørn 07.10): den SEKUNDÆRE sidestolpen («Chat» — VS Codes
  // innebygde chatvisning) åpner seg i ferske nettlesere og klemte
  // Zoo-fanen til en smal stripe. Egen kommando — closeSidebar tar den
  // ikke. (Så aldri feilen i testing: brukte nettlesere har stolpen
  // lagret lukket fra før.)
  await vscode.commands.executeCommand("workbench.action.closeAuxiliaryBar");
}

// --- Innboks-vakt (ekspert-malen, Jørn 15.10) -------------------------
// Testfunn: Deltageren lastet opp et dokument og forventet at Agenten
// «snakket med seg selv» — men ingenting skjedde før noen skrev i chatten.
// Vakten poller innboks/ og starter automatisk en Zoo-oppgave (eller
// sender melding inn i pågående samtale) når en ny fil lander, slik at
// Agenten synlig begynner å jobbe med dokumentet med en gang.
//
// Zoo-utvidelsen eksporterer et Roo-arvet API (verifisert i dist 15.10):
// startNewTask({text}), sendMessage(text), getCurrentTaskStack().

const INNBOKS = "/home/coder/project/innboks";
const VAKT_TILSTAND = "/home/coder/.local/state/s15l-innboks-sett.json";

function lesSett() {
  try {
    return JSON.parse(fs.readFileSync(VAKT_TILSTAND, "utf8"));
  } catch (_) {
    return null;
  }
}

function skrivSett(sett) {
  try {
    fs.mkdirSync(path.dirname(VAKT_TILSTAND), { recursive: true });
    fs.writeFileSync(VAKT_TILSTAND, JSON.stringify(sett));
  } catch (_) {
    /* uten tilstandsfil varsles i verste fall samme fil to ganger */
  }
}

function skannInnboks() {
  const funn = {};
  let navneliste;
  try {
    navneliste = fs.readdirSync(INNBOKS);
  } catch (_) {
    return funn; // mappen finnes ikke (ennå)
  }
  for (const navn of navneliste) {
    if (navn.startsWith(".") || navn === "les-meg.md") continue;
    try {
      const st = fs.statSync(path.join(INNBOKS, navn));
      if (st.isFile()) funn[navn] = Math.floor(st.mtimeMs) + ":" + st.size;
    } catch (_) {}
  }
  return funn;
}

async function zooApi() {
  const ext = vscode.extensions.all.find((e) => /\.zoo-code$/i.test(e.id));
  if (!ext) return null;
  if (!ext.isActive) {
    try {
      await ext.activate();
    } catch (_) {
      return null;
    }
  }
  return ext.exports || null;
}

// Nyeste aktivitet på tvers av ALLE Zoo-oppgaver (ui_messages.json får
// ny mtime ved hver melding). Trengs fordi API-ets getCurrentTaskStack
// kun ser sidestolpe-provideren — i blankt UI kjører samtalen i en FANE
// med egen provider, usynlig for API-et.
const ZOO_TASKS =
  "/home/coder/.local/share/code-server/User/globalStorage/" +
  "zoocodeorganization.zoo-code/tasks";

function sisteZooAktivitet() {
  let nyest = 0;
  try {
    for (const id of fs.readdirSync(ZOO_TASKS)) {
      try {
        const st = fs.statSync(path.join(ZOO_TASKS, id, "ui_messages.json"));
        if (st.mtimeMs > nyest) nyest = st.mtimeMs;
      } catch (_) {}
    }
  } catch (_) {}
  return nyest;
}

async function varsleAgenten(nyeNavn) {
  // Tålmodig løkke: API-et kan være uferdig rett etter oppstart, og en
  // PÅGÅENDE samtale skal ikke rives vekk (newTab lukker alle faner).
  // Vent til det har vært ro i 90 s, maks ~30 min, før oppgaven startes.
  for (let forsok = 0; forsok < 120; forsok++) {
    try {
      // «Tøm innboksen»-knappen (Jørn 15.10): deltageren kan ha
      // ombestemt seg mens vi ventet — varsle kun om filer som
      // fortsatt ligger der.
      nyeNavn = nyeNavn.filter((n) => {
        try {
          return fs.statSync(path.join(INNBOKS, n)).isFile();
        } catch (_) {
          return false;
        }
      });
      if (nyeNavn.length === 0) return;
      if (Date.now() - sisteZooAktivitet() > 90_000) {
        const en = nyeNavn.length === 1;
        const liste = nyeNavn.map((n) => `«${n}»`).join(", ");
        const tekst =
          `Deltageren har nettopp lastet opp ${en ? "et dokument" : "dokumenter"} ` +
          `i innboks/: ${liste}. Les ${en ? "det" : "dem"} nå, gi Deltageren en ` +
          `kort oppsummering på norsk av innholdet, strukturer det inn i ` +
          `prosjektet slik AGENTS.md beskriver, og avslutt med forslag til ` +
          `neste steg.`;
        const api = await zooApi();
        if (api && api.startNewTask) {
          // newTab: Zoo lukker selv alle editorfaner og åpner oppgaven
          // som egen fane — samme fullskjermsutseende som blankt UI.
          // (Uten newTab havner oppgaven i den LUKKEDE sidestolpen.)
          await api.startNewTask({ text: tekst, newTab: true });
          // Zoo legger fanen i delt kolonne — samme opprydding som ved
          // oppstart gjør den til hele flaten igjen.
          await new Promise((r) => setTimeout(r, 1000));
          await vscode.commands.executeCommand(
            "workbench.action.editorLayoutSingle"
          );
          await vscode.commands.executeCommand("workbench.action.closeSidebar");
          await vscode.commands.executeCommand(
            "workbench.action.closeAuxiliaryBar"
          );
          return;
        }
      }
    } catch (_) {}
    await new Promise((r) => setTimeout(r, 15000));
  }
}

function startInnboksvakt() {
  // Første kjøring uten tilstandsfil: noter det som ligger der (malens
  // egne filer / tidligere opplastinger) uten å varsle.
  let kjent = lesSett();
  if (kjent === null) {
    kjent = skannInnboks();
    skrivSett(kjent);
  }
  const ventende = new Set();
  let sistNytt = 0;
  let varsler = false;
  setInterval(async () => {
    const naa = skannInnboks();
    for (const [navn, stempel] of Object.entries(naa)) {
      if (kjent[navn] !== stempel) {
        kjent[navn] = stempel;
        ventende.add(navn);
        sistNytt = Date.now();
        skrivSett(kjent);
      }
    }
    // Liten ro-periode så en opplasting av flere filer blir ÉN beskjed.
    if (ventende.size > 0 && !varsler && Date.now() - sistNytt > 4000) {
      varsler = true;
      const navn = [...ventende];
      ventende.clear();
      await varsleAgenten(navn);
      varsler = false;
    }
  }, 3000);
}

function activate() {
  // Blankt UI: nybegynner (05.10) og ekspert (09.10) — chatten er hele
  // flaten, all editor-støy ryddes.
  const blankUi = ["nybegynner", "ekspert"].includes(process.env.S15L_MAL);
  const start = async () => {
    try {
      if (blankUi) {
        await ryddNybegynner();
      } else {
        await vscode.commands.executeCommand("zoo-code.SidebarProvider.focus");
      }
    } catch (_) {
      /* chatten får heller åpnes manuelt enn at oppstarten feiler */
    }
  };
  setTimeout(start, 1500);
  if (blankUi) setTimeout(start, 5000); // andre runde når alt er lastet
  // Innboks-vakten gjelder malene med dokumentopplasting (i dag: ekspert).
  if (process.env.S15L_MAL === "ekspert") startInnboksvakt();
}

function deactivate() {}

module.exports = { activate, deactivate };
