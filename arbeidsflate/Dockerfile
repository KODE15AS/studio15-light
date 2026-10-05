# Arbeidsflate-image for Studio 15 LIGHT: VS Code i nettleser (code-server)
# + Zoo Code + Node/Vite. Én container per arbeidsflate (per deltager).
# Ingen ekte API-nøkler her inne — Zoo Code snakker kun med LLM-proxyen.
#
# Alle versjoner er pinnet (norm: oppgrader bevisst ved rebuild):
#   code-server 4.140.0 (VS Code >= 1.104, Zoo Code krever >= 1.100)
#   Node 22 LTS, Zoo Code 3.87.100557 (Open VSX, verifisert utgiver)

FROM codercom/code-server:4.140.0-debian

USER root

# Node LTS for Vite-utviklingsserveren (prosjektets levende webside)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
  && apt-get install -y --no-install-recommends nodejs \
  && rm -rf /var/lib/apt/lists/*

# Ripgrep-fellen fra Studio 15 (testfunn 6): Roo lette kun etter
# @vscode/ripgrep, code-server leverer @vscode/ripgrep-universal.
# Zoo Code 3.87 leter selv i ripgrep-universal/bin/<platform>-<arch>
# (verifisert i dist/extension.js 03.10) — symlinken beholdes som billig
# forsikring, og testen under feiler bygget høyt hvis code-server-layouten
# endrer seg ved en oppgradering.
RUN test -x /usr/lib/code-server/lib/vscode/node_modules/@vscode/ripgrep-universal/bin/linux-x64/rg \
  && mkdir -p /usr/lib/code-server/lib/vscode/node_modules/@vscode/ripgrep/bin \
  && ln -sf ../../ripgrep-universal/bin/linux-x64/rg \
       /usr/lib/code-server/lib/vscode/node_modules/@vscode/ripgrep/bin/rg

# code-server leverer Copilot/GitHub-innlogging innebygd (testfunn 13):
# Copilot-chatten åpner seg selv og lokker brukeren gjennom GitHub
# device-kode + 2FA. Fjernes — og bygget feiler hvis noe GitHub-/Copilot-
# aktig fortsatt ligger igjen (mappenavnene kan endres ved oppgradering).
RUN rm -rf /usr/lib/code-server/lib/vscode/extensions/copilot \
           /usr/lib/code-server/lib/vscode/extensions/github \
           /usr/lib/code-server/lib/vscode/extensions/github-authentication \
  && ! ls /usr/lib/code-server/lib/vscode/extensions/ | grep -Ei 'copilot|github'

# Workbench-HTML-en må være skrivbar for coder: entrypoint legger inn en
# inline <style> som gjemmer tittellinjen i nybegynner-malen (Jørn 05.10,
# rapport 4) — web-workbenchen har ingen innstilling som skjuler den, og
# workbench.css kan ligge cachet i nettleserne (URL uten innholdshash).
RUN chown coder:coder /usr/lib/code-server/lib/vscode/out/vs/code/browser/workbench/workbench.html

# Prosjektmappen må finnes med riktig eier i imaget: navngitte volumer
# arver eierskapet herfra ved første montering — ellers blir volumet
# root-eid og git clone feiler med Permission denied (testfunn 10).
RUN mkdir -p /home/coder/project && chown coder:coder /home/coder/project

USER coder

# Zoo Code fra Open VSX — pinnet versjon, verifisert utgiver
# (ZooCodeOrganization). Utvidelses-ID: ZooCodeOrganization.zoo-code.
RUN curl -fsSL -o /tmp/zoo.vsix \
      "https://open-vsx.org/api/ZooCodeOrganization/zoo-code/3.87.100557/file/ZooCodeOrganization.zoo-code-3.87.100557.vsix" \
  && code-server --install-extension /tmp/zoo.vsix \
  && rm /tmp/zoo.vsix

# Bitteliten lokal utvidelse som åpner Zoo Code-chatten automatisk ved
# oppstart (lærdom fra Studio 15: brukerne fant ikke ikonet selv). Må pakkes
# som vsix og installeres ordentlig — code-server stoler på extensions.json
# og ignorerer mapper som bare kopieres inn (testfunn 16).
COPY --chown=coder:coder oppstart-extension/ /tmp/oppstart-extension/
RUN cd /tmp/oppstart-extension \
  && npx --yes @vscode/vsce package -o /tmp/oppstart.vsix \
  && code-server --install-extension /tmp/oppstart.vsix \
  && rm -rf /tmp/oppstart-extension /tmp/oppstart.vsix ~/.npm

COPY --chown=coder:coder zoo-settings.template.json /opt/s15l/
COPY --chown=coder:coder entrypoint.sh /opt/s15l/entrypoint.sh

EXPOSE 8080 5173
ENTRYPOINT ["/opt/s15l/entrypoint.sh"]
