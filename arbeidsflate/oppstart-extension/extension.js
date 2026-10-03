// Studio 15 LIGHT: åpner Zoo Code-chatten automatisk ved oppstart
// (lærdom fra Studio 15: brukerne fant ikke assistent-ikonet selv).
// View-ID verifisert 03.10: zoo-code.SidebarProvider → focus-kommandoen
// genereres av VS Code som "<view-id>.focus".
const vscode = require("vscode");

function activate() {
  setTimeout(() => {
    vscode.commands
      .executeCommand("zoo-code.SidebarProvider.focus")
      .then(undefined, () => {});
  }, 1000);
}

function deactivate() {}

module.exports = { activate, deactivate };
