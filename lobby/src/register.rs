// Programregisteret: YAML i repoet (beslutning 03.10 — alt bor i git,
// ingen database). Lobbyen leser og skriver fila; committing skjer som
// vanlig arbeid i repoet (norm «tbd»).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Prosjekt {
    pub slug: String,
    pub navn: String,
    /// git-URL arbeidsflatene kloner fra. Typen velges ved opprettelse:
    /// https://github.com/<org>/<slug>.git (program med org, vaktmesteren)
    /// eller file:///repos/<slug>.git (lokalt bare-repo).
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

// ---------- Deltagere ----------

/// Registrert deltager (Jørn 04.10): en enkel, helt åpen tabell i repoet —
/// ingen credentials, hvem som helst kan velge hvilken som helst deltager.
/// Bevisst unntak fra grunnlagsdokumentene («ingen innlogging»): dette er
/// identitetsVALG, ikke autentisering. Fargen tildeles ved registrering og
/// er fast for alltid (samme palett som presence-laget).
#[derive(Serialize, Deserialize, Clone)]
pub struct Deltager {
    pub slug: String,
    pub navn: String,
    pub farge: String,
    pub registrert: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Deltagere {
    #[serde(default)]
    pub deltagere: Vec<Deltager>,
}

const DELTAGER_HODE: &str = "\
# Deltagerregister for Studio 15 LIGHT (Jørn 04.10): åpen tabell i git —
# ingen database, ingen credentials. Identitetsvalg, ikke innlogging
# (bevisst unntak fra grunnlagsdokumentene, se handover 04.10).
# Fargen tildeles ved registrering og er fast.
";

impl Deltagere {
    pub fn load(path: &str) -> anyhow::Result<Deltagere> {
        match std::fs::read_to_string(path) {
            Ok(tekst) => Ok(serde_yaml::from_str(&tekst)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Deltagere::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, format!("{DELTAGER_HODE}{yaml}"))?;
        Ok(())
    }
}
