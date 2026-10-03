// Vaktmester-klienten: GitHub App-automatikk for program-orgene
// (App `studio15-light-vaktmester`, App ID i .env, nøkkel i certs/).
//
// Mønsteret fra Studio 15: app-JWT → installasjonstoken → repo-operasjoner.
// Installasjonstokens lever 1 time og hentes FERSKT ved hver operasjon —
// de lagres aldri. Arbeidsflatene får kun tokens scopet til sitt eget repo
// (contents: write) via /api/git-token; org-brede tokens brukes bare av
// lobbyen selv (repo-oppretting/-sletting og seeding).

use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;

const API: &str = "https://api.github.com";

pub struct Vaktmester {
    app_id: String,
    key: jsonwebtoken::EncodingKey,
    http: reqwest::Client,
    /// org → installasjons-id (stabil så lenge appen er installert).
    installasjoner: Mutex<HashMap<String, u64>>,
}

#[derive(Deserialize)]
pub struct Token {
    pub token: String,
    pub expires_at: String,
}

/// Plukker (org, repo) ut av en https://github.com/<org>/<repo>.git-URL.
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("https://github.com/")?;
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut deler = rest.splitn(2, '/');
    let org = deler.next()?.to_string();
    let repo = deler.next()?.to_string();
    if org.is_empty() || repo.is_empty() || repo.contains('/') {
        return None;
    }
    Some((org, repo))
}

impl Vaktmester {
    /// None når appen ikke er konfigurert (da finnes bare lokale repoer).
    pub fn fra_env() -> Option<Vaktmester> {
        let app_id = std::env::var("GITHUB_APP_ID").ok()?;
        let key_file = std::env::var("GITHUB_APP_KEY_FILE").ok()?;
        let pem = match std::fs::read(&key_file) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[vaktmester] kan ikke lese {key_file}: {e} — GitHub-automatikk er AV");
                return None;
            }
        };
        let key = match jsonwebtoken::EncodingKey::from_rsa_pem(&pem) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("[vaktmester] ugyldig nøkkel i {key_file}: {e} — GitHub-automatikk er AV");
                return None;
            }
        };
        let http = reqwest::Client::builder()
            .user_agent("studio15-light-vaktmester")
            .build()
            .ok()?;
        eprintln!("[vaktmester] klar (App ID {app_id})");
        Some(Vaktmester {
            app_id,
            key,
            http,
            installasjoner: Mutex::new(HashMap::new()),
        })
    }

    /// Kortlevd app-JWT (9 min) — kun til å hente installasjonstokens.
    fn jwt(&self) -> anyhow::Result<String> {
        let naa = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let claims = json!({
            // 60 s slingring bakover mot klokkeskjevhet (GitHubs anbefaling)
            "iat": naa - 60,
            "exp": naa + 540,
            "iss": self.app_id,
        });
        Ok(jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &self.key,
        )?)
    }

    async fn github(
        &self,
        metode: reqwest::Method,
        sti: &str,
        bearer: &str,
        kropp: Option<serde_json::Value>,
    ) -> anyhow::Result<(reqwest::StatusCode, serde_json::Value)> {
        let mut req = self
            .http
            .request(metode, format!("{API}{sti}"))
            .bearer_auth(bearer)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(k) = kropp {
            req = req.json(&k);
        }
        let svar = req.send().await?;
        let status = svar.status();
        let body: serde_json::Value = svar.json().await.unwrap_or(serde_json::Value::Null);
        Ok((status, body))
    }

    async fn installasjons_id(&self, org: &str) -> anyhow::Result<u64> {
        if let Some(id) = self.installasjoner.lock().unwrap().get(org) {
            return Ok(*id);
        }
        let jwt = self.jwt()?;
        let (status, body) = self
            .github(reqwest::Method::GET, &format!("/orgs/{org}/installation"), &jwt, None)
            .await?;
        if !status.is_success() {
            anyhow::bail!(
                "vaktmesteren er ikke installert i org-en «{org}» ({status}) — \
                 se docs/vaktmester-klikkeliste.md"
            );
        }
        let id = body["id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("uventet svar fra GitHub: {body}"))?;
        self.installasjoner.lock().unwrap().insert(org.to_string(), id);
        Ok(id)
    }

    /// Ferskt installasjonstoken. `repo_scope`: Some((repo, kun contents:rw))
    /// for arbeidsflatene, None = org-bredt (kun lobbyens egne operasjoner).
    async fn token(&self, org: &str, repo_scope: Option<&str>) -> anyhow::Result<Token> {
        let id = self.installasjons_id(org).await?;
        let jwt = self.jwt()?;
        let kropp = repo_scope.map(|repo| {
            json!({
                "repositories": [repo],
                "permissions": { "contents": "write" },
            })
        });
        let (status, body) = self
            .github(
                reqwest::Method::POST,
                &format!("/app/installations/{id}/access_tokens"),
                &jwt,
                kropp,
            )
            .await?;
        if !status.is_success() {
            anyhow::bail!("fikk ikke installasjonstoken for «{org}» ({status}): {body}");
        }
        Ok(serde_json::from_value(body)?)
    }

    /// Repo-scopet token (contents: write) til en arbeidsflates push/pull.
    pub async fn repo_token(&self, org: &str, repo: &str) -> anyhow::Result<Token> {
        self.token(org, Some(repo)).await
    }

    /// Oppretter privat repo i program-org-en. Idempotent: finnes repoet
    /// allerede, er det greit («prøv igjen» er alltid trygt).
    /// Returnerer klone-URL (https, uten auth).
    pub async fn opprett_repo(&self, org: &str, repo: &str) -> anyhow::Result<String> {
        let tok = self.token(org, None).await?;
        let (status, body) = self
            .github(
                reqwest::Method::POST,
                &format!("/orgs/{org}/repos"),
                &tok.token,
                Some(json!({
                    "name": repo,
                    "private": true,
                    "has_issues": false,
                    "has_projects": false,
                    "has_wiki": false,
                })),
            )
            .await?;
        // 422 med «name already exists» = idempotent gjenbruk
        let finnes_alt = status.as_u16() == 422
            && body["errors"]
                .as_array()
                .map(|e| {
                    e.iter().any(|x| {
                        x["message"]
                            .as_str()
                            .is_some_and(|m| m.contains("already exists"))
                    })
                })
                .unwrap_or(false);
        if !status.is_success() && !finnes_alt {
            anyhow::bail!("repo-oppretting i «{org}» feilet ({status}): {body}");
        }
        Ok(format!("https://github.com/{org}/{repo}.git"))
    }

    /// Sletter repoet (Administration RW — E2E-verifisert 03.10).
    /// 404 tolereres: borte er borte.
    pub async fn slett_repo(&self, org: &str, repo: &str) -> anyhow::Result<()> {
        let tok = self.token(org, None).await?;
        let (status, body) = self
            .github(
                reqwest::Method::DELETE,
                &format!("/repos/{org}/{repo}"),
                &tok.token,
                None,
            )
            .await?;
        if !status.is_success() && status.as_u16() != 404 {
            anyhow::bail!("repo-sletting av {org}/{repo} feilet ({status}): {body}");
        }
        Ok(())
    }
}
