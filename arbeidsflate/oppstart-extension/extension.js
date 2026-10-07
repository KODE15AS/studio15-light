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

function activate() {
  const nybegynner = process.env.S15L_MAL === "nybegynner";
  const start = async () => {
    try {
      if (nybegynner) {
        await ryddNybegynner();
      } else {
        await vscode.commands.executeCommand("zoo-code.SidebarProvider.focus");
      }
    } catch (_) {
      /* chatten får heller åpnes manuelt enn at oppstarten feiler */
    }
  };
  setTimeout(start, 1500);
  if (nybegynner) setTimeout(start, 5000); // andre runde når alt er lastet
}

function deactivate() {}

module.exports = { activate, deactivate };
