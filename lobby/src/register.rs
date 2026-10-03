// Programregisteret: YAML i repoet (beslutning 03.10 — alt bor i git,
// ingen database). Lobbyen leser og skriver fila; committing skjer som
// vanlig arbeid i repoet (norm «tbd»).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Prosjekt {
    pub slug: String,
    pub navn: String,
    /// git-URL arbeidsflatene kloner fra. file:///repos/<slug>.git til
    /// vaktmester-appen gir ekte GitHub-repoer.
    pub repo: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Program {
    pub slug: String,
    pub navn: String,
    /// GitHub-org-navnet når org-en finnes (manuell oppretting — GitHub
    /// har ikke API for dette; lobbyen viser den guidede flyten).
    pub github_org: Option<String>,
    #[serde(default)]
    pub prosjekter: Vec<Prosjekt>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Register {
    #[serde(default)]
    pub programmer: Vec<Program>,
}

const HODE: &str = "\
# Programregister for Studio 15 LIGHT (beslutning 03.10: alt bor i git,
# ingen database). Et program er en GitHub-org; prosjektene er
# container/repo-settene under programmet. Lobbyen leser og oppdaterer
# denne fila; endringer committes som vanlig (norm «tbd»).
#
# github_org er null til org-en finnes (GitHub har ikke API for
# org-oppretting — lobbyen viser den guidede manuelle flyten).
";

impl Register {
    pub fn load(path: &str) -> anyhow::Result<Register> {
        match std::fs::read_to_string(path) {
            Ok(tekst) => Ok(serde_yaml::from_str(&tekst)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Register::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, format!("{HODE}{yaml}"))?;
        Ok(())
    }
}
