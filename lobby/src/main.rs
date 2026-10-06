// Lobbyen for Studio 15 LIGHT: programvelger, prosjektvelger og
// arbeidsflate-styring. Ingen database — programregisteret er YAML i
// repoet, kjøretilstand leses fra Docker (beslutning 03.10).
//
// Controller-mønsteret fra Skjermsamling: dette er den ENESTE komponenten
// som starter/stopper containere, og kun gjennom faste operasjoner
// (ensure/start/stop/remove). Frontend kan aldri sende containerparametre.

mod driver;
mod presence;
mod register;
mod vaktmester;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

use driver::{Driver, WsSpec};
use register::{Program, Prosjekt, Register};

pub struct Cfg {
    pub public_base: String,
    pub ws_image: String,
    pub ws_network: String,
    pub repos_volume: String,
    pub proxy_key: String,
    pub register_path: String,
    pub access_log: String,
    pub idle_minutes: u64,
    pub reaper_ignore_ips: Vec<String>,
    pub prosjektmal: String,
    pub repos_dir: String,
    pub webdir: String,
    /// Deltagerregisteret (04.10): åpen tabell, samme katalog som programmer.
    pub deltager_path: String,
    /// Hemmelighet for utledning av per-arbeidsflate-tokens (git-token-stien).
    pub token_secret: String,
    pub ice_servers: serde_json::Value,
}

pub struct App {
    pub cfg: Cfg,
    pub register: Mutex<Register>,
    pub driver: Driver,
    /// Siste aktivitet per arbeidsflate (kortnavn → unix-sekunder).
    pub activity: Mutex<HashMap<String, f64>>,
    /// GitHub App-klienten — None når appen ikke er konfigurert.
    pub vaktmester: Option<vaktmester::Vaktmester>,
    /// Presence-huben (se/peke/ta over, V2) — ett rom per prosjekt.
    pub presence: presence::Presence,
    /// Deltagerregisteret (04.10): åpen tabell — identitetsvalg, ikke
    /// innlogging. Fast farge tildeles ved registrering.
    pub deltagere: Mutex<register::Deltagere>,
    /// Fjern-restart av veggen (Jørn 04.10): unix-tidsstempel for siste
    /// forespørsel. Kiosk-vakta på raven poller og restarter Chromium
    /// friskt når stempelet er nyere enn det den har sett — virker også
    /// når selve veggsiden er frossen eller krasjet.
    pub vegg_restart: Mutex<u64>,
}

async fn vegg_restart_sett(State(app): State<Arc<App>>) -> Response {
    let naa = now_unix() as u64;
    *app.vegg_restart.lock().await = naa;
    Json(json!({ "restart": naa })).into_response()
}

async fn vegg_restart_les(State(app): State<Arc<App>>) -> Response {
    let sist = *app.vegg_restart.lock().await;
    Json(json!({ "sist": sist })).into_response()
}

/// Per-arbeidsflate-hemmelighet for /api/git-token: HMAC(token_secret,
/// kortnavn). Stateless — lobbyen kan alltid regne den ut på nytt, og den
/// overlever både lobby-restart og gjenskapte arbeidsflater.
pub fn flate_hemmelighet(token_secret: &str, kortnavn: &str) -> String {
    use hmac::Mac;
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(token_secret.as_bytes())
        .expect("HMAC tar nøkkel av vilkårlig lengde");
    mac.update(kortnavn.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Sammenligning i konstant tid (hemmeligheter skal aldri time-lekkes).
fn lik_konstant_tid(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn now_unix() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
}

/// Slugify til [a-z0-9-]: norske tegn translittereres, resten blir bindestrek.
pub fn slugify(s: &str) -> String {
    let mut ut = String::new();
    let mut forrige_strek = true;
    for c in s.to_lowercase().chars() {
        let mapped: Option<&str> = match c {
            'æ' => Some("ae"),
            'ø' => Some("oe"),
            'å' => Some("aa"),
            'a'..='z' | '0'..='9' => None,
            _ => Some("-"),
        };
        match mapped {
            None => {
                ut.push(c);
                forrige_strek = false;
            }
            Some("-") => {
                if !forrige_strek {
                    ut.push('-');
                    forrige_strek = true;
                }
            }
            Some(t) => {
                ut.push_str(t);
                forrige_strek = false;
            }
        }
    }
    ut.trim_matches('-').to_string()
}

fn feil(status: StatusCode, melding: &str) -> Response {
    (status, Json(json!({ "feil": melding }))).into_response()
}

// ---------- Deltagere ----------

#[derive(Deserialize)]
struct NyDeltager {
    navn: String,
}

async fn deltagere_liste(State(app): State<Arc<App>>) -> Response {
    let d = app.deltagere.lock().await.clone();
    Json(json!({ "deltagere": d.deltagere })).into_response()
}

async fn deltager_registrer(State(app): State<Arc<App>>, Json(b): Json<NyDeltager>) -> Response {
    let navn = b.navn.trim().to_string();
    let slug = slugify(&navn);
    if slug.is_empty() {
        return feil(StatusCode::BAD_REQUEST, "navnet gir ingen gyldig deltager");
    }
    let mut tabell = app.deltagere.lock().await;
    if tabell.deltagere.iter().any(|d| d.slug == slug) {
        return feil(StatusCode::CONFLICT, "deltageren er allerede registrert — velg den i listen");
    }
    // Fast farge ved registrering (Jørn 04.10): første ledige fra paletten;
    // ved flere deltagere enn farger gjenbrukes paletten rundt.
    let i_bruk: Vec<&str> = tabell.deltagere.iter().map(|d| d.farge.as_str()).collect();
    let farge = presence::FARGER
        .iter()
        .find(|f| !i_bruk.contains(*f))
        .copied()
        .unwrap_or(presence::FARGER[tabell.deltagere.len() % presence::FARGER.len()])
        .to_string();
    let deltager = register::Deltager {
        slug,
        navn,
        farge,
        registrert: chrono::Utc::now().format("%Y-%m-%d").to_string(),
    };
    tabell.deltagere.push(deltager.clone());
    if let Err(e) = tabell.save(&app.cfg.deltager_path) {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("lagring: {e}"));
    }
    Json(json!({ "deltager": deltager })).into_response()
}

/// Sletter en deltager fra registeret (Jørn 06.10, rapport 1 pkt. 1):
/// én bekreftelsesknapp i frontenden, ingen avskrift av navn. Åpen
/// tabell uten credentials — sletting er like åpen som registrering.
/// Eksisterende arbeidsflater beholder deltagernavnet sitt (kortnavnet
/// bærer det); kun valgmuligheten i velgeren forsvinner.
async fn deltager_slett(State(app): State<Arc<App>>, Path(slug): Path<String>) -> Response {
    let mut tabell = app.deltagere.lock().await;
    let foer = tabell.deltagere.len();
    tabell.deltagere.retain(|d| d.slug != slug);
    if tabell.deltagere.len() == foer {
        return feil(StatusCode::NOT_FOUND, "deltageren finnes ikke");
    }
    if let Err(e) = tabell.save(&app.cfg.deltager_path) {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("lagring: {e}"));
    }
    Json(json!({ "slettet": slug })).into_response()
}

// ---------- Dialog-speilet (tavla) ----------

/// Tavlas dialog-speil (Jørn 06.10, rapport 1 pkt. 2): tavlas egen
/// code-server-økt viser alltid en fersk, TOM Zoo-chat — deltagerens
/// samtale bor i en annen nettleserøkt. Selve samtalen ligger som fil i
/// containeren (ui_messages.json), så lobbyen leser den derfra og koker
/// den ned til det tavla skal vise: tekstmeldinger, spørsmål med
/// svarforslag og små verktøylinjer. Store felt (diffs, filinnhold)
/// filtreres bort her slik at tavla aldri henter dem over nettet.
async fn flate_dialog(State(app): State<Arc<App>>, Path(kortnavn): Path<String>) -> Response {
    let raa = match app.driver.les_zoo_dialog(&kortnavn).await {
        Ok(r) => r,
        Err(e) => return feil(StatusCode::BAD_GATEWAY, &format!("dialog: {e}")),
    };
    let meldinger: Vec<serde_json::Value> = serde_json::from_str(&raa).unwrap_or_default();
    let mut ut = Vec::new();
    for (i, m) in meldinger.iter().enumerate() {
        let typ = m["type"].as_str().unwrap_or("");
        let ts = m["ts"].as_i64().unwrap_or(0);
        let tekst = m["text"].as_str().unwrap_or("");
        match (typ, m["say"].as_str().unwrap_or(""), m["ask"].as_str().unwrap_or("")) {
            ("say", "task", _) | ("say", "user_feedback", _) => {
                if !tekst.is_empty() {
                    ut.push(json!({ "hvem": "deltager", "tekst": tekst, "ts": ts }));
                }
            }
            ("say", "text", _) | ("say", "completion_result", _) => {
                // Første melding i samtalen er deltagerens oppgave;
                // system-sanitering er støy.
                if tekst.is_empty() || tekst.starts_with("[System:") {
                    continue;
                }
                let hvem = if i == 0 { "deltager" } else { "hjelper" };
                ut.push(json!({ "hvem": hvem, "tekst": tekst, "ts": ts }));
            }
            ("ask", _, "followup") => {
                let f: serde_json::Value = serde_json::from_str(tekst).unwrap_or_default();
                let forslag: Vec<&str> = f["suggest"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|s| s["answer"].as_str()).collect())
                    .unwrap_or_default();
                ut.push(json!({
                    "hvem": "hjelper",
                    "tekst": f["question"].as_str().unwrap_or(tekst),
                    "forslag": forslag,
                    "ts": ts,
                }));
            }
            ("ask", _, "tool") => {
                let v: serde_json::Value = serde_json::from_str(tekst).unwrap_or_default();
                let verb = match v["tool"].as_str().unwrap_or("") {
                    "editedExistingFile" | "appliedDiff" | "newFileCreated" => "skriver i",
                    "readFile" => "leser",
                    "searchFiles" | "listFilesTopLevel" | "listFilesRecursive" => "leter i",
                    _ => continue, // andre verktøy er støy på tavla
                };
                if let Some(sti) = v["path"].as_str() {
                    ut.push(json!({ "hvem": "verktoy", "tekst": format!("{verb} {sti}"), "ts": ts }));
                }
            }
            _ => {}
        }
    }
    // «Hjelperen jobber»-indikatoren: siste råmelding er et API-kall.
    let aktiv = meldinger
        .last()
        .map(|m| m["say"].as_str() == Some("api_req_started"))
        .unwrap_or(false);
    Json(json!({ "meldinger": ut, "aktiv": aktiv })).into_response()
}

// ---------- Tilstand ----------

async fn tilstand(State(app): State<Arc<App>>) -> Response {
    let reg = app.register.lock().await.clone();
    let deltagerliste = app.deltagere.lock().await.clone().deltagere;
    let farge_for = |slug: &str| {
        deltagerliste
            .iter()
            .find(|d| d.slug == slug)
            .map(|d| d.farge.clone())
    };
    let flater = match app.driver.list().await {
        Ok(f) => f,
        Err(e) => return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}")),
    };
    let programmer: Vec<_> = reg
        .programmer
        .iter()
        .map(|p| {
            let prosjekter: Vec<_> = p
                .prosjekter
                .iter()
                .map(|pr| {
                    let arbeidsflater: Vec<_> = flater
                        .iter()
                        .filter(|w| w.program == p.slug && w.prosjekt == pr.slug)
                        .map(|w| {
                            json!({
                                "deltager": w.deltager,
                                // Registerfargen (04.10): konsistent eierfarge
                                // også når deltageren ikke er tilkoblet presence.
                                "farge": farge_for(&w.deltager),
                                "kortnavn": w.kortnavn,
                                "kjorer": w.running,
                                // Relative lenker: fungerer både via ts.net-inngangen og
                                // LAN-inngangen (studio-wifi) — nettleseren beholder sin origin.
                                "editor_url": format!("/w/{}/", w.kortnavn),
                                "web_url": format!("/web/{}/", w.kortnavn),
                            })
                        })
                        .collect();
                    json!({
                        "slug": pr.slug, "navn": pr.navn, "repo": pr.repo,
                        "mal": pr.mal,
                        "arbeidsflater": arbeidsflater,
                    })
                })
                .collect();
            json!({
                "slug": p.slug, "navn": p.navn, "github_org": p.github_org,
                "prosjekter": prosjekter,
            })
        })
        .collect();
    Json(json!({
        "programmer": programmer,
        "deltagere": deltagerliste,
        "base": app.cfg.public_base,
        "ice": app.cfg.ice_servers,
    }))
    .into_response()
}

/// Spilleplanen for en mal (nybegynner): YAML-fila i malmappen servert som
/// JSON. ÉN kilde — samme fil seedes inn i prosjektrepoet for hjelperen,
/// og vises i instruksfeltet på deltagerskjermen.
async fn spilleplan(State(app): State<Arc<App>>, Path(mal): Path<String>) -> Response {
    if !MALER.contains(&mal.as_str()) {
        return feil(StatusCode::NOT_FOUND, "ukjent prosjektmal");
    }
    let sti = format!("{}/{mal}/spilleplan.yaml", app.cfg.prosjektmal);
    let Ok(tekst) = std::fs::read_to_string(&sti) else {
        return feil(StatusCode::NOT_FOUND, "malen har ingen spilleplan");
    };
    match serde_yaml::from_str::<serde_json::Value>(&tekst) {
        Ok(v) => Json(v).into_response(),
        Err(e) => feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("spilleplan: {e}")),
    }
}

// ---------- Programmer ----------

#[derive(Deserialize)]
struct NyttProgram {
    navn: String,
    github_org: Option<String>,
}

async fn nytt_program(State(app): State<Arc<App>>, Json(b): Json<NyttProgram>) -> Response {
    let slug = slugify(&b.navn);
    if slug.is_empty() {
        return feil(StatusCode::BAD_REQUEST, "programnavnet gir ingen gyldig slug");
    }
    let mut reg = app.register.lock().await;
    if reg.programmer.iter().any(|p| p.slug == slug) {
        return feil(StatusCode::CONFLICT, "programmet finnes allerede");
    }
    reg.programmer.push(Program {
        slug: slug.clone(),
        navn: b.navn,
        github_org: b.github_org.filter(|o| !o.trim().is_empty()),
        prosjekter: vec![],
    });
    if let Err(e) = reg.save(&app.cfg.register_path) {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("register: {e}"));
    }
    Json(json!({ "slug": slug })).into_response()
}

/// Sletter et program fra registeret — kun når det er tomt (ingen
/// prosjekter). Org-sletting på GitHub er alltid manuell.
async fn slett_program(
    State(app): State<Arc<App>>,
    Path(program): Path<String>,
    Json(b): Json<Bekreftelse>,
) -> Response {
    if b.bekreft != program {
        return feil(StatusCode::BAD_REQUEST, "bekreftelsen må være programmets slug");
    }
    let mut reg = app.register.lock().await;
    let Some(p) = reg.programmer.iter().find(|p| p.slug == program) else {
        return feil(StatusCode::NOT_FOUND, "ukjent program");
    };
    if !p.prosjekter.is_empty() {
        return feil(
            StatusCode::CONFLICT,
            "programmet har prosjekter — slett dem først",
        );
    }
    reg.programmer.retain(|p| p.slug != program);
    if let Err(e) = reg.save(&app.cfg.register_path) {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("register: {e}"));
    }
    Json(json!({ "slettet": program })).into_response()
}

// ---------- Prosjekter ----------

#[derive(Deserialize)]
struct NyttProsjekt {
    program: String,
    navn: String,
    /// Prosjektmal (Jørn 05.10): «full» eller «nybegynner». Utelatt = full.
    mal: Option<String>,
}

/// Gyldige prosjektmaler = undermapper av PROSJEKTMAL. Listen er også
/// vakta mot sti-triksing i mal-parameteren (ingen «..» e.l.).
const MALER: &[&str] = &["full", "nybegynner"];

fn kjor(cmd: &mut std::process::Command) -> anyhow::Result<()> {
    let out = cmd.output()?;
    if !out.status.success() {
        anyhow::bail!(
            "{:?}: {}",
            cmd.get_program(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

/// Seeder et tomt remote-repo (bare-repo ELLER GitHub) fra prosjektmalen:
/// git init → malen inn med navnefletting → commit → push HEAD:main.
fn seed_fra_mal(app: &App, repo_slug: &str, navn: &str, mal: &str, remote: &str) -> anyhow::Result<()> {
    use std::process::Command;
    let tmp = format!("/tmp/seed-{repo_slug}");
    let _ = std::fs::remove_dir_all(&tmp);
    kjor(Command::new("git").args(["init", "-b", "main", &tmp]))?;
    kjor(Command::new("cp").args(["-rT", &format!("{}/{mal}", app.cfg.prosjektmal), &tmp]))?;
    // Flett prosjektnavnet inn i malen
    for fil in ["package.json", "index.html", "src/App.svelte"] {
        let sti = format!("{tmp}/{fil}");
        if let Ok(innhold) = std::fs::read_to_string(&sti) {
            let nytt = innhold
                .replace("__PROSJEKTSLUG__", repo_slug)
                .replace("__PROSJEKTNAVN__", navn);
            std::fs::write(&sti, nytt)?;
        }
    }
    kjor(Command::new("git").args(["-C", &tmp, "add", "-A"]))?;
    kjor(Command::new("git").args([
        "-C", &tmp,
        "-c", "user.name=Studio 15 LIGHT",
        "-c", "user.email=lobby@studio15-light.lokal",
        "commit", "-m", "Prosjekt opprettet fra malen",
    ]))?;
    kjor(Command::new("git").args(["-C", &tmp, "push", remote, "HEAD:main"]))?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}

/// Oppretter prosjektets bare-repo på repos-volumet og seeder det fra
/// prosjektmalen. Brukes av programmer UTEN GitHub-org (lokale prosjekter).
fn seed_bare_repo(app: &App, repo_slug: &str, navn: &str, mal: &str) -> anyhow::Result<String> {
    use std::process::Command;
    let bare = format!("{}/{}.git", app.cfg.repos_dir, repo_slug);
    let url = format!("file://{bare}");
    if std::path::Path::new(&bare).exists() {
        return Ok(url); // idempotent: «prøv igjen» er alltid trygt
    }
    kjor(Command::new("git").args(["init", "--bare", "-b", "main", &bare]))?;
    seed_fra_mal(app, repo_slug, navn, mal, &url)?;
    // Arbeidsflatene kjører som coder (uid 1000) og skal både klone fra og
    // pushe til repoet — lobbyen kjører som root, så eierskapet må over
    // (samme klasse felle som testfunn 10: root-eide volumer).
    kjor(Command::new("chown").args(["-R", "1000:1000", &bare]))?;
    Ok(url)
}

/// Oppretter prosjektrepoet i programmets GitHub-org via vaktmesteren og
/// seeder det fra malen (push med ferskt installasjonstoken — tokenet
/// lever 1 time og havner aldri i registeret eller miljøvariabler).
async fn seed_github_repo(
    app: &App,
    org: &str,
    repo_slug: &str,
    navn: &str,
    mal: &str,
) -> anyhow::Result<String> {
    let vm = app
        .vaktmester
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "programmet har GitHub-org, men vaktmester-appen er ikke konfigurert \
             (GITHUB_APP_ID/GITHUB_APP_KEY_FILE i .env)"
        ))?;
    let url = vm.opprett_repo(org, repo_slug).await?;
    let tok = vm.repo_token(org, repo_slug).await?;
    let auth = format!("https://x-access-token:{}@github.com/{org}/{repo_slug}.git", tok.token);
    // Tokenet må aldri lekke i feilmeldinger (git siterer gjerne URL-en).
    seed_fra_mal(app, repo_slug, navn, mal, &auth)
        .map_err(|e| anyhow::anyhow!("{}", e.to_string().replace(&tok.token, "***")))?;
    Ok(url)
}

async fn nytt_prosjekt(State(app): State<Arc<App>>, Json(b): Json<NyttProsjekt>) -> Response {
    let slug = slugify(&b.navn);
    if slug.is_empty() {
        return feil(StatusCode::BAD_REQUEST, "prosjektnavnet gir ingen gyldig slug");
    }
    let mal = b.mal.unwrap_or_else(register::mal_standard);
    if !MALER.contains(&mal.as_str()) {
        return feil(StatusCode::BAD_REQUEST, "ukjent prosjektmal");
    }
    let mut reg = app.register.lock().await;
    let Some(prog) = reg.programmer.iter_mut().find(|p| p.slug == b.program) else {
        return feil(StatusCode::NOT_FOUND, "ukjent program");
    };
    if prog.prosjekter.iter().any(|p| p.slug == slug) {
        return feil(StatusCode::CONFLICT, "prosjektet finnes allerede");
    }
    // Forhåndsvakt mot DNS-fella (se ny_arbeidsflate): det må være plass
    // til minst et kort deltagernavn (8 tegn) innenfor 55-grensen.
    if prog.slug.len() + 1 + slug.len() > 47 {
        return feil(
            StatusCode::BAD_REQUEST,
            &format!(
                "program- og prosjektnavnet blir for langt sammen ({} av maks \
                 47 tegn) — da blir arbeidsflate-adressene ugyldige. \
                 Velg et kortere prosjektnavn.",
                prog.slug.len() + 1 + slug.len()
            ),
        );
    }
    // Repo-typen velges ved opprettelse: GitHub-repo i program-org-en når
    // org finnes (vaktmesteren), ellers bare-repo på repos-volumet.
    let repo_slug = format!("{}-{}", prog.slug, slug);
    let github_org = prog.github_org.clone();
    let repo = if app.driver.is_mock() {
        format!("file:///repos/{repo_slug}.git")
    } else if let Some(org) = &github_org {
        // Repo-navnet i org-en er prosjekt-sluggen (org-en ER programmet).
        match seed_github_repo(&app, org, &slug, &b.navn, &mal).await {
            Ok(u) => u,
            Err(e) => return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("github: {e}")),
        }
    } else {
        match seed_bare_repo(&app, &repo_slug, &b.navn, &mal) {
            Ok(u) => u,
            Err(e) => return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("repo: {e}")),
        }
    };
    prog.prosjekter.push(Prosjekt {
        slug: slug.clone(),
        navn: b.navn,
        repo,
        mal,
    });
    if let Err(e) = reg.save(&app.cfg.register_path) {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("register: {e}"));
    }
    Json(json!({ "slug": slug })).into_response()
}

#[derive(Deserialize)]
struct Bekreftelse {
    bekreft: String,
}

/// Sletting er sletting (beslutning 03.10): containere + volumer + repo i
/// én operasjon etter eksplisitt bekreftelse. GitHub-repo/org-steget er
/// manuelt til vaktmester-appen finnes.
async fn slett_prosjekt(
    State(app): State<Arc<App>>,
    Path((program, prosjekt)): Path<(String, String)>,
    Json(b): Json<Bekreftelse>,
) -> Response {
    if b.bekreft != prosjekt {
        return feil(
            StatusCode::BAD_REQUEST,
            "bekreftelsen må være prosjektets slug — sletting er sletting",
        );
    }
    let mut reg = app.register.lock().await;
    let Some(prog) = reg.programmer.iter_mut().find(|p| p.slug == program) else {
        return feil(StatusCode::NOT_FOUND, "ukjent program");
    };
    let Some(pr) = prog.prosjekter.iter().find(|p| p.slug == prosjekt) else {
        return feil(StatusCode::NOT_FOUND, "ukjent prosjekt");
    };
    let repo_url = pr.repo.clone();
    // 1) alle arbeidsflater (containere + volumer)
    let flater = match app.driver.list().await {
        Ok(f) => f,
        Err(e) => return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}")),
    };
    for w in flater.iter().filter(|w| w.program == program && w.prosjekt == prosjekt) {
        if let Err(e) = app.driver.remove_workspace(&w.kortnavn).await {
            return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("sletting: {e}"));
        }
    }
    // 2) prosjektrepoet — GitHub-repo via vaktmesteren, ellers bare-repoet
    //    på volumet. Feiler GitHub-sletting, blir prosjektet stående i
    //    registeret så slettingen kan prøves igjen (sletting er sletting —
    //    ingen dangling repos).
    let mut github_slettet = false;
    if let Some((org, repo)) = vaktmester::parse_github_url(&repo_url) {
        let Some(vm) = &app.vaktmester else {
            return feil(
                StatusCode::INTERNAL_SERVER_ERROR,
                "prosjektet har GitHub-repo, men vaktmesteren er ikke konfigurert",
            );
        };
        if let Err(e) = vm.slett_repo(&org, &repo).await {
            return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("github: {e}"));
        }
        github_slettet = true;
    } else if !app.driver.is_mock() {
        let bare = format!("{}/{}-{}.git", app.cfg.repos_dir, program, prosjekt);
        let _ = std::fs::remove_dir_all(&bare);
    }
    // 3) registeret
    prog.prosjekter.retain(|p| p.slug != prosjekt);
    if let Err(e) = reg.save(&app.cfg.register_path) {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("register: {e}"));
    }
    Json(json!({
        "slettet": prosjekt,
        "github_repo_slettet": github_slettet,
        "manuelt": if github_slettet {
            "Ingenting — GitHub-repoet er slettet av vaktmesteren. \
             (Org-sletting er fortsatt manuell hvis hele programmet legges ned.)"
        } else {
            "Lokalt prosjekt — ingenting å gjøre på GitHub."
        }
    }))
    .into_response()
}

// ---------- Git-tokens til arbeidsflatene ----------

#[derive(Deserialize)]
struct GitTokenReq {
    kortnavn: String,
    hemmelighet: String,
}

/// DET ENE unntaket i caddy-vakten (se Caddyfile): arbeidsflater kan POSTe
/// hit for å få et FERSKT installasjonstoken scopet til sitt eget repo
/// (contents: write, 1 times levetid). Autentisering: per-arbeidsflate-
/// hemmelighet (HMAC av kortnavnet) satt som env ved opprettelse.
/// Arbeidsflaten kan aldri få token til andre repoer enn sitt eget.
async fn git_token(State(app): State<Arc<App>>, Json(b): Json<GitTokenReq>) -> Response {
    if app.cfg.token_secret.is_empty() {
        return feil(StatusCode::SERVICE_UNAVAILABLE, "token-tjenesten er ikke konfigurert");
    }
    let riktig = flate_hemmelighet(&app.cfg.token_secret, &b.kortnavn);
    if !lik_konstant_tid(&riktig, &b.hemmelighet) {
        return feil(StatusCode::FORBIDDEN, "ugyldig arbeidsflate-hemmelighet");
    }
    // Arbeidsflaten må finnes — kortnavnet gir program/prosjekt via labels.
    let flater = match app.driver.list().await {
        Ok(f) => f,
        Err(e) => return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}")),
    };
    let Some(w) = flater.into_iter().find(|w| w.kortnavn == b.kortnavn) else {
        return feil(StatusCode::NOT_FOUND, "ukjent arbeidsflate");
    };
    let repo_url = {
        let reg = app.register.lock().await;
        let Some(pr) = reg
            .programmer
            .iter()
            .find(|p| p.slug == w.program)
            .and_then(|p| p.prosjekter.iter().find(|pr| pr.slug == w.prosjekt))
        else {
            return feil(StatusCode::NOT_FOUND, "arbeidsflaten hører ikke til noe prosjekt");
        };
        pr.repo.clone()
    };
    let Some((org, repo)) = vaktmester::parse_github_url(&repo_url) else {
        return feil(StatusCode::BAD_REQUEST, "prosjektet bruker lokalt repo — trenger ikke token");
    };
    let Some(vm) = &app.vaktmester else {
        return feil(StatusCode::SERVICE_UNAVAILABLE, "vaktmesteren er ikke konfigurert");
    };
    match vm.repo_token(&org, &repo).await {
        Ok(t) => Json(json!({ "token": t.token, "utloper": t.expires_at })).into_response(),
        Err(e) => feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("github: {e}")),
    }
}

// ---------- Arbeidsflater ----------

#[derive(Deserialize)]
struct NyArbeidsflate {
    program: String,
    prosjekt: String,
    deltager: String,
}

async fn ny_arbeidsflate(State(app): State<Arc<App>>, Json(b): Json<NyArbeidsflate>) -> Response {
    let deltager = slugify(&b.deltager);
    if deltager.is_empty() {
        return feil(StatusCode::BAD_REQUEST, "deltagernavnet gir ingen gyldig slug");
    }
    let (repo, mal) = {
        let reg = app.register.lock().await;
        let Some(pr) = reg
            .programmer
            .iter()
            .find(|p| p.slug == b.program)
            .and_then(|p| p.prosjekter.iter().find(|pr| pr.slug == b.prosjekt))
        else {
            return feil(StatusCode::NOT_FOUND, "ukjent program/prosjekt");
        };
        (pr.repo.clone(), pr.mal.clone())
    };
    let kortnavn = format!("{}-{}-{}", b.program, b.prosjekt, deltager);
    // DNS-fella (betalt 04.10): containernavnet «s15l-ws-<kortnavn>» er
    // også DNS-navnet caddy ruter på, og DNS-etiketter er maks 63 tegn.
    // Lengre navn gjør flaten UOPPNÅELIG (dockers DNS svarer «Message too
    // large») — derfor hard vakt her med tydelig melding.
    if kortnavn.len() > 55 {
        return feil(
            StatusCode::BAD_REQUEST,
            &format!(
                "navnet blir for langt ({} av maks 55 tegn: program + prosjekt \
                 + deltager). Bruk kortere prosjekt- eller deltagernavn.",
                kortnavn.len()
            ),
        );
    }
    // GitHub-prosjekter: arbeidsflaten får en per-flate-hemmelighet og
    // henter ferske repo-scopede tokens via /api/git-token ved hver
    // push/pull (tokens lever 1 time — aldri fast i miljøet).
    let mut git_env = vec![];
    if vaktmester::parse_github_url(&repo).is_some() && !app.cfg.token_secret.is_empty() {
        git_env.push(format!("WS_KORTNAVN={kortnavn}"));
        git_env.push(format!(
            "GIT_TOKEN_SECRET={}",
            flate_hemmelighet(&app.cfg.token_secret, &kortnavn)
        ));
        git_env.push("GIT_TOKEN_URL=https://s15l-caddy:8100/api/git-token".to_string());
    }
    let spec = WsSpec {
        kortnavn: kortnavn.clone(),
        program: b.program.clone(),
        prosjekt: b.prosjekt.clone(),
        deltager: deltager.clone(),
        image: app.cfg.ws_image.clone(),
        network: app.cfg.ws_network.clone(),
        repos_volume: app.cfg.repos_volume.clone(),
        env: vec![
            format!("PROJECT_REPO={repo}"),
            format!("PARTICIPANT={deltager}"),
            format!("WEB_BASE=/web/{kortnavn}/"),
            format!("WEB_URL={}/web/{}/", app.cfg.public_base, kortnavn),
            format!("EDITOR_URL={}/w/{}/", app.cfg.public_base, kortnavn),
            "LLM_PROXY_BASE=http://s15l-litellm:4000/v1".to_string(),
            format!("LLM_PROXY_KEY={}", app.cfg.proxy_key),
            // Malen styrer editor-UI-et (nybegynner = ryddet Zoo-skjerm);
            // extension-hosten i code-server leser den fra miljøet.
            format!("S15L_MAL={mal}"),
        ]
        .into_iter()
        .chain(git_env)
        .collect(),
    };
    if let Err(e) = app.driver.ensure_workspace(&spec).await {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}"));
    }
    app.activity.lock().await.insert(kortnavn.clone(), now_unix());
    Json(json!({
        "kortnavn": kortnavn,
        "editor_url": format!("/w/{kortnavn}/"),
        "web_url": format!("/web/{kortnavn}/"),
    }))
    .into_response()
}

async fn stopp_arbeidsflate(State(app): State<Arc<App>>, Path(kortnavn): Path<String>) -> Response {
    match app.driver.stop(&kortnavn).await {
        Ok(_) => Json(json!({ "stoppet": kortnavn })).into_response(),
        Err(e) => feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}")),
    }
}

async fn vekk_arbeidsflate(State(app): State<Arc<App>>, Path(kortnavn): Path<String>) -> Response {
    match app.driver.start(&kortnavn).await {
        Ok(_) => {
            app.activity.lock().await.insert(kortnavn.clone(), now_unix());
            Json(json!({ "vekket": kortnavn })).into_response()
        }
        Err(e) => feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}")),
    }
}

// ---------- Presence (se/peke/ta over, V2) ----------

#[derive(Deserialize)]
struct PresenceParams {
    watch: Option<String>,
}

async fn presence_ws(
    State(app): State<Arc<App>>,
    Path((program, prosjekt)): Path<(String, String)>,
    Query(q): Query<PresenceParams>,
    ws: WebSocketUpgrade,
) -> Response {
    // Veggens watch-modus: joiner aldri, kan aldri påvirke økten.
    let watch = q.watch.as_deref() == Some("1");
    ws.on_upgrade(move |socket| presence_socket(app, program, prosjekt, watch, socket))
}

async fn presence_socket(
    app: Arc<App>,
    program: String,
    prosjekt: String,
    watch: bool,
    socket: WebSocket,
) {
    let rom = app.presence.rom(&program, &prosjekt);
    // Abonner på broadcast FØR join — ellers race der ny tilkobling mister
    // roster-oppdateringer (Skjermsamling-lærdom C).
    let mut rx = rom.tx.subscribe();
    let (mut ut, mut inn) = socket.split();

    // Watch-tilkoblinger (tavla) joiner aldri, men trenger en adresse for
    // WebRTC-signaleringen (pilot 05.10) — flyktig id per tilkobling.
    let watch_id = uuid::Uuid::new_v4();

    // Watch-tilkoblinger får roster-snapshot og signaleringsadressen sin.
    if watch {
        let _ = ut.send(Message::Text(rom.roster_json().into())).await;
        let hilsen = serde_json::to_string(&presence::ServerMelding::WatchVelkommen {
            id: watch_id,
        })
        .unwrap();
        let _ = ut.send(Message::Text(hilsen.into())).await;
    }

    let mut meg: Option<uuid::Uuid> = None;
    loop {
        tokio::select! {
            b = rx.recv() => {
                match b {
                    Ok(m) => {
                        if ut.send(Message::Text(m.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Hengt etter: send ferskt roster i stedet for å dø.
                        let _ = ut.send(Message::Text(rom.roster_json().into())).await;
                    }
                    Err(_) => break,
                }
            }
            m = inn.next() => {
                let Some(Ok(m)) = m else { break };
                let Message::Text(tekst) = m else { continue };
                let Ok(melding) = serde_json::from_str::<presence::KlientMelding>(&tekst) else {
                    continue;
                };
                // Watch (tavla) er read-only for ØKTEN, men strøm-signalering
                // (WebRTC-pilot 05.10) må gå begge veier: tavla svarer på
                // tilbud for å motta webside-strømmen. Media går aldri
                // gjennom serveren — kun små signal-meldinger.
                if watch {
                    if let presence::KlientMelding::Strom { flate, til, signal } = melding {
                        rom.strom(watch_id, &flate, til, signal);
                    }
                    continue; // alle andre meldinger ignoreres
                }
                use presence::KlientMelding::*;
                match melding {
                    Join { navn, session, farge } => {
                        let navn = navn.trim().to_string();
                        let slug = slugify(&navn);
                        if slug.is_empty() {
                            let feil = serde_json::to_string(&presence::ServerMelding::Feil {
                                melding: "navnet gir ingen gyldig deltager".into(),
                            }).unwrap();
                            let _ = ut.send(Message::Text(feil.into())).await;
                            continue;
                        }
                        // Registrert deltager → alltid registerfargen,
                        // uansett hva klienten oppga.
                        let registrert_farge = app
                            .deltagere
                            .lock()
                            .await
                            .deltagere
                            .iter()
                            .find(|d| d.slug == slug)
                            .map(|d| d.farge.clone());
                        let (id, _tok, welcome) =
                            rom.join(navn, slug, session, registrert_farge.or(farge));
                        meg = Some(id);
                        // Welcome + roster-snapshot DIREKTE til ny socket,
                        // deretter roster til alle.
                        let _ = ut.send(Message::Text(welcome.into())).await;
                        let _ = ut.send(Message::Text(rom.roster_json().into())).await;
                        rom.broadcast_roster();
                    }
                    Cursor { tile, x, y } => {
                        if let Some(id) = meg {
                            rom.broadcast_cursor(id, &tile, x, y);
                        }
                    }
                    Ta { flate } => {
                        let Some(id) = meg else { continue };
                        // Flaten må finnes i DETTE prosjektet; eier-sluggen
                        // avgjør om det er ens egen flate.
                        let eier = app
                            .driver
                            .list()
                            .await
                            .ok()
                            .and_then(|f| {
                                f.into_iter().find(|w| {
                                    w.kortnavn == flate
                                        && w.program == program
                                        && w.prosjekt == prosjekt
                                })
                            })
                            .map(|w| w.deltager);
                        let resultat = match eier {
                            Some(eier_slug) => rom.ta(id, &flate, &eier_slug),
                            None => Err("ukjent arbeidsflate i dette prosjektet".into()),
                        };
                        if let Err(e) = resultat {
                            let feil = serde_json::to_string(&presence::ServerMelding::Feil {
                                melding: e,
                            }).unwrap();
                            let _ = ut.send(Message::Text(feil.into())).await;
                        }
                    }
                    Slipp => {
                        if let Some(id) = meg {
                            rom.slipp(id);
                        }
                    }
                    Forlat => {
                        if let Some(id) = meg.take() {
                            rom.fjern(id);
                        }
                        break;
                    }
                    Strom { flate, til, signal } => {
                        // WebRTC-signalering (pilot 05.10): postbud-relay.
                        if let Some(id) = meg {
                            rom.strom(id, &flate, til, signal);
                        }
                    }
                }
            }
        }
    }

    // Frakobling: behold identiteten en stund (reconnect-vinduet), rydd så
    // opp — generasjonstelleren avbryter utdaterte timeout-tasks.
    if let Some(id) = meg {
        if let Some(gen) = rom.marker_frakoblet(id) {
            let secs = app.presence.timeout_secs;
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                if rom.fortsatt_frakoblet(id, gen) {
                    rom.fjern(id);
                }
            });
        }
    }
}

// ---------- Vekkesiden (dvale/vekke-mønsteret) ----------

/// Caddy ruter hit ved upstream-feil: svarer 503 med en side som laster
/// originaladressen på nytt når flaten svarer. Tre tilfeller med hver sin
/// tydelige melding (Skjermsamling F: aldri evig spinner):
///   - kjørende, men ikke klar ennå → første oppstart («gjør seg klar»)
///   - stoppet → dvale: vekkes automatisk, oppe på ~10 s
///   - ukjent → 404 med vei tilbake til lobbyen
async fn vekk_side(
    State(app): State<Arc<App>>,
    Path(sti): Path<String>,
    Query(q): Query<PresenceParams>,
) -> Response {
    // Veggen (watch-modus) skal ALDRI vekke sovende flater — ellers holder
    // en vegg som står på hele natten alle flatene kunstig våkne.
    let watch = q.watch.as_deref() == Some("1");
    // sti er originalstien uten ledende skråstrek, f.eks. "w/demo-x-jorn/..."
    let deler: Vec<&str> = sti.splitn(3, '/').collect();
    let kortnavn = match deler.as_slice() {
        ["w" | "web", navn, ..] => navn.to_string(),
        _ => String::new(),
    };
    let flate = if kortnavn.is_empty() {
        None
    } else {
        app.driver
            .list()
            .await
            .ok()
            .and_then(|f| f.into_iter().find(|w| w.kortnavn == kortnavn))
    };
    let Some(flate) = flate else {
        return (
            StatusCode::NOT_FOUND,
            Html(vekk_html(
                "Ukjent skjerm",
                "Denne adressen peker ikke på noen kjent deltagerskjerm. \
                 Gå til startsiden og start prosjektet derfra.",
                None,
            )),
        )
            .into_response();
    };
    // Behold watch-parameteren i reload-adressen — ellers mister veggens
    // iframe watch-modusen etter første vekking og begynner å vekke selv.
    let orig = if watch {
        format!("/{sti}?watch=1")
    } else {
        format!("/{sti}")
    };
    let (tittel, melding, vekk) = if flate.running {
        (
            "Skjermen gjør seg klar …",
            "Prosjektet starter opp — første gang tar det gjerne et minutt \
             mens pakkene installeres. Siden laster automatisk på nytt når \
             alt er klart.",
            None, // kjører allerede — ingenting å vekke
        )
    } else if watch {
        (
            "Skjermen sover",
            "Skjermen er i dvale etter inaktivitet. Tavla vekker den ikke — \
             den våkner når eieren åpner den, og dukker da opp her av seg selv.",
            None, // watch vekker aldri, men poller til flaten er oppe
        )
    } else {
        (
            "Skjermen vekkes …",
            "Skjermen har sovet (dvale etter inaktivitet) og startes nå. \
             Siden laster automatisk på nytt — det tar normalt rundt 10 sekunder.",
            Some(kortnavn.as_str()),
        )
    };
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Html(vekk_html(tittel, melding, Some((vekk, &orig)))),
    )
        .into_response()
}

fn vekk_html(tittel: &str, melding: &str, vekk: Option<(Option<&str>, &str)>) -> String {
    // Synlighetsvakt (funn 05.10): gjenåpnede/gjenopprettede BAKGRUNNS-faner
    // vekket zombie-flater om morgenen og holdt flater kunstig våkne hele
    // natten. Vekking og polling skjer kun når fanen faktisk er synlig —
    // en gjenopprettet fane vekker først når brukeren bytter til den.
    let script = match vekk {
        Some((kortnavn, orig)) => {
            let vekk_kall = match kortnavn {
                Some(navn) => format!(
                    "fetch('/api/arbeidsflater/{navn}/vekk', {{method: 'POST'}}).catch(() => {{}});"
                ),
                None => String::new(),
            };
            format!(
                r#"<script>
const synlig = () => document.visibilityState === 'visible';
let vekket = false;
function vekk() {{
  if (vekket || !synlig()) return;
  vekket = true;
  {vekk_kall}
}}
vekk();
document.addEventListener('visibilitychange', vekk);
setInterval(async () => {{
  if (!synlig()) return;
  try {{
    const r = await fetch({orig:?}, {{cache: 'no-store'}});
    if (r.ok) location.href = {orig:?};
  }} catch (e) {{}}
}}, 2000);
</script>"#
            )
        }
        None => String::new(),
    };
    format!(
        r#"<!DOCTYPE html><html lang="nb"><head><meta charset="utf-8">
<title>{tittel} — Studio 15 LIGHT</title>
<style>
  body {{ background: #F7F4EF; color: #959593; font-family: -apple-system, 'Segoe UI', Roboto, sans-serif;
         display: grid; place-items: center; min-height: 100vh; margin: 0; }}
  .kort {{ background: #fff; border: 1px solid #DBDBDB; border-radius: 14px; padding: 32px 40px;
          max-width: 480px; text-align: center; }}
  h1 {{ font-size: 22px; font-weight: 400; color: #525D65; margin: 0 0 12px; }}
  .puls {{ width: 14px; height: 14px; border-radius: 50%; background: #BBAD9A; margin: 0 auto 18px;
          animation: puls 1.2s ease-in-out infinite; }}
  @keyframes puls {{ 50% {{ transform: scale(1.6); opacity: 0.4; }} }}
  a {{ color: #525D65; }}
</style></head><body>
<div class="kort"><div class="puls"></div><h1>{tittel}</h1><p>{melding}</p>
<p><a href="/">Til startsiden</a></p></div>
{script}
</body></html>"#
    )
}

// ---------- Idle-reaper (dvale) ----------

/// Leser caddys JSON-tilgangslogg og stopper arbeidsflater uten aktivitet
/// på IDLE_MINUTES minutter. Vekking skjer via vekkesiden over.
async fn reaper(app: Arc<App>) {
    let mut offset: u64 = 0;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        // 1) ny aktivitet fra loggen
        if let Ok(data) = tokio::fs::read(&app.cfg.access_log).await {
            if (data.len() as u64) < offset {
                offset = 0; // loggen er rullert
            }
            let nye = &data[offset as usize..];
            offset = data.len() as u64;
            let mut akt = app.activity.lock().await;
            for linje in nye.split(|b| *b == b'\n') {
                let Ok(v) = serde_json::from_slice::<serde_json::Value>(linje) else {
                    continue;
                };
                let (Some(ts), Some(uri)) = (
                    v.get("ts").and_then(|t| t.as_f64()),
                    v.pointer("/request/uri").and_then(|u| u.as_str()),
                ) else {
                    continue;
                };
                // Kun VELLYKKEDE oppslag teller som aktivitet. Vekkesiden
                // (og veggen) poller hvert 2. sekund og treffer 503 — uten
                // dette filteret holder en åpen fane/vegg flaten kunstig
                // våken for alltid (funn 04.10).
                if v.get("status").and_then(|s| s.as_u64()).unwrap_or(599) >= 500 {
                    continue;
                }
                // Ravens egen trafikk (tavla) teller ikke som aktivitet.
                let klient = v
                    .pointer("/request/client_ip")
                    .or_else(|| v.pointer("/request/remote_ip"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                if app.cfg.reaper_ignore_ips.iter().any(|ip| ip == klient) {
                    continue;
                }
                let mut deler = uri.trim_start_matches('/').splitn(3, '/');
                if let (Some("w") | Some("web"), Some(navn)) = (deler.next(), deler.next()) {
                    let e = akt.entry(navn.to_string()).or_insert(0.0);
                    if ts > *e {
                        *e = ts;
                    }
                }
            }
        }
        // 2) stopp sovende arbeidsflater
        let grense = app.cfg.idle_minutes as f64 * 60.0;
        let naa = now_unix();
        let Ok(flater) = app.driver.list().await else { continue };
        let akt = app.activity.lock().await.clone();
        for w in flater.iter().filter(|w| w.running) {
            let start = w.started_at_unix.unwrap_or(0.0);
            let siste = akt.get(&w.kortnavn).copied().unwrap_or(0.0).max(start);
            if naa - siste > grense {
                eprintln!("[reaper] {} har sovnet ({} min inaktiv) — stopper", w.kortnavn, app.cfg.idle_minutes);
                let _ = app.driver.stop(&w.kortnavn).await;
            }
        }
    }
}

// ---------- Oppstart ----------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Cfg {
        public_base: env_or("PUBLIC_BASE", "https://cadify104raven.tail14de1b.ts.net:8100"),
        ws_image: env_or("WORKSPACE_IMAGE", "studio15-light-arbeidsflate:v1"),
        ws_network: env_or("WORKSPACE_NETWORK", "s15l-workspaces"),
        repos_volume: env_or("REPOS_VOLUME", "s15l-repos"),
        proxy_key: env_or("LITELLM_MASTER_KEY", ""),
        register_path: env_or("REGISTER_PATH", "/data/register/programmer.yaml"),
        access_log: env_or("ACCESS_LOG", "/logs/access.log"),
        idle_minutes: env_or("IDLE_MINUTES", "45").parse().unwrap_or(45),
        // Ravens egne adresser (tavla/kiosken, lokal testing): trafikk herfra
        // teller ALDRI som aktivitet — ellers nullstiller hver tavle-restart
        // dvaleklokka for alle fliser den viser (funn 05.10).
        reaper_ignore_ips: env_or("REAPER_IGNORE_IPS", "")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        prosjektmal: env_or("PROSJEKTMAL", "/opt/prosjektmal"),
        repos_dir: env_or("REPOS_DIR", "/repos"),
        webdir: env_or("WEBDIR", "/opt/lobby/web"),
        deltager_path: env_or("DELTAGER_PATH", "/data/register/deltagere.yaml"),
        token_secret: env_or("S15L_TOKEN_SECRET", ""),
        // RTCIceServers-liste (JSON) for webside-strømmen (pilot 05.10):
        // coturn på raven gir garantert vei også PC↔raven over Tailscale.
        ice_servers: serde_json::from_str(&env_or("ICE_SERVERS", "[]"))
            .unwrap_or_else(|_| serde_json::json!([])),
    };
    let driver = if env_or("WORKSPACE_DRIVER", "docker") == "mock" {
        Driver::new_mock()
    } else {
        Driver::new_docker()?
    };
    let register = Register::load(&cfg.register_path)?;
    let deltagere = register::Deltagere::load(&cfg.deltager_path)?;
    let app = Arc::new(App {
        cfg,
        register: Mutex::new(register),
        deltagere: Mutex::new(deltagere),
        driver,
        activity: Mutex::new(HashMap::new()),
        vaktmester: vaktmester::Vaktmester::fra_env(),
        presence: presence::Presence::ny(),
        vegg_restart: Mutex::new(0),
    });

    tokio::spawn(reaper(app.clone()));

    let statisk = ServeDir::new(&app.cfg.webdir)
        .not_found_service(ServeFile::new(format!("{}/index.html", app.cfg.webdir)));

    let router = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/tilstand", get(tilstand))
        .route("/api/programmer", post(nytt_program))
        .route("/api/programmer/{program}", delete(slett_program))
        .route("/api/prosjekter", post(nytt_prosjekt))
        .route("/api/spilleplan/{mal}", get(spilleplan))
        .route("/api/prosjekter/{program}/{prosjekt}", delete(slett_prosjekt))
        .route("/api/arbeidsflater", post(ny_arbeidsflate))
        .route("/api/deltagere", get(deltagere_liste).post(deltager_registrer))
        .route("/api/deltagere/{slug}", delete(deltager_slett))
        .route("/api/arbeidsflater/{kortnavn}/dialog", get(flate_dialog))
        .route("/api/git-token", post(git_token))
        .route("/api/presence/{program}/{prosjekt}/ws", get(presence_ws))
        .route("/api/arbeidsflater/{kortnavn}/stopp", post(stopp_arbeidsflate))
        .route("/api/arbeidsflater/{kortnavn}/vekk", post(vekk_arbeidsflate))
        .route("/api/vegg/restart", post(vegg_restart_sett).get(vegg_restart_les))
        .route("/vekk/{*sti}", get(vekk_side))
        .fallback_service(statisk)
        .with_state(app);

    let lytter = tokio::net::TcpListener::bind("0.0.0.0:8200").await?;
    eprintln!("[lobby] lytter på :8200");
    axum::serve(lytter, router).await?;
    Ok(())
}
