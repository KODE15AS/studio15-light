// Lobbyen for Studio 15 LIGHT: programvelger, prosjektvelger og
// arbeidsflate-styring. Ingen database — programregisteret er YAML i
// repoet, kjøretilstand leses fra Docker (beslutning 03.10).
//
// Controller-mønsteret fra Skjermsamling: dette er den ENESTE komponenten
// som starter/stopper containere, og kun gjennom faste operasjoner
// (ensure/start/stop/remove). Frontend kan aldri sende containerparametre.

mod driver;
mod register;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
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
    pub prosjektmal: String,
    pub repos_dir: String,
    pub webdir: String,
}

pub struct App {
    pub cfg: Cfg,
    pub register: Mutex<Register>,
    pub driver: Driver,
    /// Siste aktivitet per arbeidsflate (kortnavn → unix-sekunder).
    pub activity: Mutex<HashMap<String, f64>>,
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

// ---------- Tilstand ----------

async fn tilstand(State(app): State<Arc<App>>) -> Response {
    let reg = app.register.lock().await.clone();
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
                                "kortnavn": w.kortnavn,
                                "kjorer": w.running,
                                "editor_url": format!("{}/w/{}/", app.cfg.public_base, w.kortnavn),
                                "web_url": format!("{}/web/{}/", app.cfg.public_base, w.kortnavn),
                            })
                        })
                        .collect();
                    json!({
                        "slug": pr.slug, "navn": pr.navn, "repo": pr.repo,
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
    Json(json!({ "programmer": programmer, "base": app.cfg.public_base })).into_response()
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
}

/// Oppretter prosjektets bare-repo på repos-volumet og seeder det fra
/// prosjektmalen. Stand-in til vaktmester-appen gir ekte GitHub-repoer.
fn seed_bare_repo(app: &App, repo_slug: &str, navn: &str) -> anyhow::Result<String> {
    use std::process::Command;
    let bare = format!("{}/{}.git", app.cfg.repos_dir, repo_slug);
    let url = format!("file://{bare}");
    if std::path::Path::new(&bare).exists() {
        return Ok(url); // idempotent: «prøv igjen» er alltid trygt
    }
    let kjor = |cmd: &mut Command| -> anyhow::Result<()> {
        let out = cmd.output()?;
        if !out.status.success() {
            anyhow::bail!(
                "{:?}: {}",
                cmd.get_program(),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    };
    kjor(Command::new("git").args(["init", "--bare", "-b", "main", &bare]))?;
    let tmp = format!("/tmp/seed-{repo_slug}");
    let _ = std::fs::remove_dir_all(&tmp);
    kjor(Command::new("git").args(["clone", &bare, &tmp]))?;
    kjor(Command::new("cp").args(["-rT", &app.cfg.prosjektmal, &tmp]))?;
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
    kjor(Command::new("git").args(["-C", &tmp, "push", "origin", "HEAD:main"]))?;
    let _ = std::fs::remove_dir_all(&tmp);
    // Arbeidsflatene kjører som coder (uid 1000) og skal både klone fra og
    // pushe til repoet — lobbyen kjører som root, så eierskapet må over
    // (samme klasse felle som testfunn 10: root-eide volumer).
    kjor(Command::new("chown").args(["-R", "1000:1000", &bare]))?;
    Ok(url)
}

async fn nytt_prosjekt(State(app): State<Arc<App>>, Json(b): Json<NyttProsjekt>) -> Response {
    let slug = slugify(&b.navn);
    if slug.is_empty() {
        return feil(StatusCode::BAD_REQUEST, "prosjektnavnet gir ingen gyldig slug");
    }
    let mut reg = app.register.lock().await;
    let Some(prog) = reg.programmer.iter_mut().find(|p| p.slug == b.program) else {
        return feil(StatusCode::NOT_FOUND, "ukjent program");
    };
    if prog.prosjekter.iter().any(|p| p.slug == slug) {
        return feil(StatusCode::CONFLICT, "prosjektet finnes allerede");
    }
    let repo_slug = format!("{}-{}", prog.slug, slug);
    let repo = if app.driver.is_mock() {
        format!("file:///repos/{repo_slug}.git")
    } else {
        match seed_bare_repo(&app, &repo_slug, &b.navn) {
            Ok(u) => u,
            Err(e) => return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("repo: {e}")),
        }
    };
    prog.prosjekter.push(Prosjekt {
        slug: slug.clone(),
        navn: b.navn,
        repo,
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
    if !prog.prosjekter.iter().any(|p| p.slug == prosjekt) {
        return feil(StatusCode::NOT_FOUND, "ukjent prosjekt");
    }
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
    // 2) prosjektrepoet på volumet
    if !app.driver.is_mock() {
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
        "manuelt": "Finnes prosjektet også som GitHub-repo, slettes det manuelt \
                    (vaktmester-appen automatiserer dette senere)."
    }))
    .into_response()
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
    let (repo, _navn) = {
        let reg = app.register.lock().await;
        let Some(pr) = reg
            .programmer
            .iter()
            .find(|p| p.slug == b.program)
            .and_then(|p| p.prosjekter.iter().find(|pr| pr.slug == b.prosjekt))
        else {
            return feil(StatusCode::NOT_FOUND, "ukjent program/prosjekt");
        };
        (pr.repo.clone(), pr.navn.clone())
    };
    let kortnavn = format!("{}-{}-{}", b.program, b.prosjekt, deltager);
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
        ],
    };
    if let Err(e) = app.driver.ensure_workspace(&spec).await {
        return feil(StatusCode::INTERNAL_SERVER_ERROR, &format!("docker: {e}"));
    }
    app.activity.lock().await.insert(kortnavn.clone(), now_unix());
    Json(json!({
        "kortnavn": kortnavn,
        "editor_url": format!("{}/w/{}/", app.cfg.public_base, kortnavn),
        "web_url": format!("{}/web/{}/", app.cfg.public_base, kortnavn),
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

// ---------- Vekkesiden (dvale/vekke-mønsteret) ----------

/// Caddy ruter hit ved upstream-feil: svarer 503 med en side som laster
/// originaladressen på nytt når flaten svarer. Tre tilfeller med hver sin
/// tydelige melding (Skjermsamling F: aldri evig spinner):
///   - kjørende, men ikke klar ennå → første oppstart («gjør seg klar»)
///   - stoppet → dvale: vekkes automatisk, oppe på ~10 s
///   - ukjent → 404 med vei tilbake til lobbyen
async fn vekk_side(State(app): State<Arc<App>>, Path(sti): Path<String>) -> Response {
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
                "Ukjent arbeidsflate",
                "Denne adressen peker ikke på noen kjent arbeidsflate. \
                 Gå til lobbyen og start prosjektet derfra.",
                None,
            )),
        )
            .into_response();
    };
    let orig = format!("/{sti}");
    let (tittel, melding, vekk) = if flate.running {
        (
            "Arbeidsflaten gjør seg klar …",
            "Prosjektet starter opp — første gang tar det gjerne et minutt \
             mens pakkene installeres. Siden laster automatisk på nytt når \
             alt er klart.",
            None, // kjører allerede — ingenting å vekke
        )
    } else {
        (
            "Arbeidsflaten vekkes …",
            "Arbeidsflaten har sovet (dvale etter inaktivitet) og startes nå. \
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
{vekk_kall}
setInterval(async () => {{
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
<p><a href="/">Til lobbyen</a></p></div>
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
        prosjektmal: env_or("PROSJEKTMAL", "/opt/prosjektmal"),
        repos_dir: env_or("REPOS_DIR", "/repos"),
        webdir: env_or("WEBDIR", "/opt/lobby/web"),
    };
    let driver = if env_or("WORKSPACE_DRIVER", "docker") == "mock" {
        Driver::new_mock()
    } else {
        Driver::new_docker()?
    };
    let register = Register::load(&cfg.register_path)?;
    let app = Arc::new(App {
        cfg,
        register: Mutex::new(register),
        driver,
        activity: Mutex::new(HashMap::new()),
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
        .route("/api/prosjekter/{program}/{prosjekt}", delete(slett_prosjekt))
        .route("/api/arbeidsflater", post(ny_arbeidsflate))
        .route("/api/arbeidsflater/{kortnavn}/stopp", post(stopp_arbeidsflate))
        .route("/api/arbeidsflater/{kortnavn}/vekk", post(vekk_arbeidsflate))
        .route("/vekk/{*sti}", get(vekk_side))
        .fallback_service(statisk)
        .with_state(app);

    let lytter = tokio::net::TcpListener::bind("0.0.0.0:8200").await?;
    eprintln!("[lobby] lytter på :8200");
    axum::serve(lytter, router).await?;
    Ok(())
}
