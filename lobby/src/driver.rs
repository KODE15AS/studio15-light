// Arbeidsflate-driveren: faste operasjoner mot Docker (controller-mønsteret
// fra Skjermsamling — frontend kan aldri sende containerparametre), pluss
// en mock-driver (WORKSPACE_DRIVER=mock) så lobbyen kan utvikles og testes
// uten Docker.
//
// Arbeidsflate-containere får ALDRI: docker-socket, privileged, host-
// filsystem, host-nettverk eller GPU — kun prosjektvolumet, repos-volumet
// og arbeidsflate-nettet.

use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures_util::StreamExt;
use bollard::models::{HostConfig, RestartPolicy, RestartPolicyNameEnum};
use bollard::volume::{CreateVolumeOptions, RemoveVolumeOptions};
use bollard::Docker;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct WsSpec {
    pub kortnavn: String,
    pub program: String,
    pub prosjekt: String,
    pub deltager: String,
    pub image: String,
    pub network: String,
    pub repos_volume: String,
    pub env: Vec<String>,
}

#[derive(Clone)]
pub struct WsInfo {
    pub kortnavn: String,
    pub program: String,
    pub prosjekt: String,
    pub deltager: String,
    pub running: bool,
    pub started_at_unix: Option<f64>,
}

pub enum Driver {
    Docker(Docker),
    Mock(Mutex<HashMap<String, WsInfo>>),
}

fn container_name(kortnavn: &str) -> String {
    format!("s15l-ws-{kortnavn}")
}

/// 404/409 fra Docker tolereres der operasjonen er idempotent
/// («prøv igjen» er alltid trygt).
fn tolerer(e: bollard::errors::Error, koder: &[u16]) -> anyhow::Result<()> {
    if let bollard::errors::Error::DockerResponseServerError { status_code, .. } = &e {
        if koder.contains(status_code) {
            return Ok(());
        }
    }
    Err(e.into())
}

impl Driver {
    pub fn new_docker() -> anyhow::Result<Driver> {
        Ok(Driver::Docker(Docker::connect_with_unix_defaults()?))
    }

    pub fn new_mock() -> Driver {
        Driver::Mock(Mutex::new(HashMap::new()))
    }

    pub fn is_mock(&self) -> bool {
        matches!(self, Driver::Mock(_))
    }

    pub async fn list(&self) -> anyhow::Result<Vec<WsInfo>> {
        match self {
            Driver::Mock(m) => Ok(m.lock().unwrap().values().cloned().collect()),
            Driver::Docker(d) => {
                let mut filters = HashMap::new();
                filters.insert("label".to_string(), vec!["s15l.role=arbeidsflate".to_string()]);
                let liste = d
                    .list_containers(Some(ListContainersOptions {
                        all: true,
                        filters,
                        ..Default::default()
                    }))
                    .await?;
                let mut ut = vec![];
                for c in liste {
                    let labels = c.labels.unwrap_or_default();
                    let navn = c
                        .names
                        .unwrap_or_default()
                        .first()
                        .map(|n| n.trim_start_matches('/').to_string())
                        .unwrap_or_default();
                    let kortnavn = navn.strip_prefix("s15l-ws-").unwrap_or(&navn).to_string();
                    let running = c.state.as_deref() == Some("running");
                    // started_at for reaper-gulvet — kun for kjørende
                    let started_at_unix = if running {
                        d.inspect_container(&navn, None)
                            .await
                            .ok()
                            .and_then(|i| i.state)
                            .and_then(|s| s.started_at)
                            .and_then(|t| {
                                chrono::DateTime::parse_from_rfc3339(&t)
                                    .ok()
                                    .map(|dt| dt.timestamp() as f64)
                            })
                    } else {
                        None
                    };
                    ut.push(WsInfo {
                        kortnavn,
                        program: labels.get("s15l.program").cloned().unwrap_or_default(),
                        prosjekt: labels.get("s15l.prosjekt").cloned().unwrap_or_default(),
                        deltager: labels.get("s15l.deltager").cloned().unwrap_or_default(),
                        running,
                        started_at_unix,
                    });
                }
                Ok(ut)
            }
        }
    }

    /// Oppretter volum + container hvis de mangler, og starter containeren.
    /// Idempotent: finnes alt fra før, startes den bare.
    pub async fn ensure_workspace(&self, spec: &WsSpec) -> anyhow::Result<()> {
        match self {
            Driver::Mock(m) => {
                let mut l = m.lock().unwrap();
                l.insert(
                    spec.kortnavn.clone(),
                    WsInfo {
                        kortnavn: spec.kortnavn.clone(),
                        program: spec.program.clone(),
                        prosjekt: spec.prosjekt.clone(),
                        deltager: spec.deltager.clone(),
                        running: true,
                        started_at_unix: None,
                    },
                );
                Ok(())
            }
            Driver::Docker(d) => {
                let navn = container_name(&spec.kortnavn);
                // Volumet må finnes før containeren (eierskap arves fra
                // imaget ved første montering — testfunn 10).
                d.create_volume(CreateVolumeOptions {
                    name: navn.clone(),
                    ..Default::default()
                })
                .await?; // volum-create er idempotent i Docker
                let mut labels = HashMap::new();
                labels.insert("s15l.role".to_string(), "arbeidsflate".to_string());
                labels.insert("s15l.program".to_string(), spec.program.clone());
                labels.insert("s15l.prosjekt".to_string(), spec.prosjekt.clone());
                labels.insert("s15l.deltager".to_string(), spec.deltager.clone());
                let res = d
                    .create_container(
                        Some(CreateContainerOptions {
                            name: navn.clone(),
                            platform: None,
                        }),
                        Config {
                            image: Some(spec.image.clone()),
                            env: Some(spec.env.clone()),
                            labels: Some(labels),
                            host_config: Some(HostConfig {
                                binds: Some(vec![
                                    format!("{navn}:/home/coder/project"),
                                    format!("{}:/repos", spec.repos_volume),
                                ]),
                                network_mode: Some(spec.network.clone()),
                                restart_policy: Some(RestartPolicy {
                                    name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )
                    .await;
                if let Err(e) = res {
                    tolerer(e, &[409])?; // finnes allerede — greit
                }
                self.start(&spec.kortnavn).await
            }
        }
    }

    pub async fn start(&self, kortnavn: &str) -> anyhow::Result<()> {
        match self {
            Driver::Mock(m) => {
                if let Some(w) = m.lock().unwrap().get_mut(kortnavn) {
                    w.running = true;
                }
                Ok(())
            }
            Driver::Docker(d) => {
                match d
                    .start_container(&container_name(kortnavn), None::<StartContainerOptions<String>>)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => tolerer(e, &[304]), // 304: kjører allerede
                }
            }
        }
    }

    pub async fn stop(&self, kortnavn: &str) -> anyhow::Result<()> {
        match self {
            Driver::Mock(m) => {
                if let Some(w) = m.lock().unwrap().get_mut(kortnavn) {
                    w.running = false;
                }
                Ok(())
            }
            Driver::Docker(d) => {
                match d
                    .stop_container(&container_name(kortnavn), Some(StopContainerOptions { t: 10 }))
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => tolerer(e, &[304]), // 304: allerede stoppet
                }
            }
        }
    }

    /// Leser Zoo-dialogens meldingsfil (siste task) fra arbeidsflaten —
    /// grunnlaget for dialog-speilet på tavla (Jørn 06.10, rapport 1
    /// pkt. 2): tavlas egen code-server-økt viser en FERSK, tom Zoo-chat,
    /// aldri deltagerens samtale. Selve samtalen bor som fil i
    /// containeren, så lobbyen henter den derfra (docker exec).
    pub async fn les_zoo_dialog(&self, kortnavn: &str) -> anyhow::Result<String> {
        match self {
            Driver::Mock(_) => Ok(String::new()),
            Driver::Docker(d) => {
                let exec = d
                    .create_exec(
                        &container_name(kortnavn),
                        CreateExecOptions {
                            cmd: Some(vec![
                                "sh",
                                "-c",
                                // Nyeste task-mappe = samtalen som pågår.
                                "cat \"$(ls -td /home/coder/.local/share/code-server/User/globalStorage/zoocodeorganization.zoo-code/tasks/*/ 2>/dev/null | head -1)ui_messages.json\" 2>/dev/null",
                            ]),
                            attach_stdout: Some(true),
                            attach_stderr: Some(false),
                            ..Default::default()
                        },
                    )
                    .await?;
                let mut ut = Vec::new();
                if let StartExecResults::Attached { mut output, .. } =
                    d.start_exec(&exec.id, None).await?
                {
                    while let Some(melding) = output.next().await {
                        if let Ok(bollard::container::LogOutput::StdOut { message }) = melding {
                            ut.extend_from_slice(&message);
                        }
                    }
                }
                Ok(String::from_utf8_lossy(&ut).into_owned())
            }
        }
    }

    /// Legger en opplastet fil i prosjektets `innboks/` (ekspert-malen,
    /// 09.10): deltageren laster opp prosjektdokumenter fra skjermen, og
    /// agenten strukturerer dem inn i repoet. Skrives som tar-arkiv rett
    /// inn i containeren (PUT /containers/…/archive) — ingen shell-quoting,
    /// og eierskapet settes til coder (uid 1000) i tar-headeren.
    pub async fn last_opp(&self, kortnavn: &str, filnavn: &str, data: &[u8]) -> anyhow::Result<()> {
        match self {
            Driver::Mock(_) => Ok(()),
            Driver::Docker(d) => {
                let mut arkiv = tar::Builder::new(Vec::new());
                let mut hode = tar::Header::new_gnu();
                hode.set_size(data.len() as u64);
                hode.set_mode(0o644);
                hode.set_uid(1000);
                hode.set_gid(1000);
                hode.set_mtime(chrono::Utc::now().timestamp() as u64);
                arkiv.append_data(&mut hode, format!("innboks/{filnavn}"), data)?;
                let bytes = arkiv.into_inner()?;
                d.upload_to_container(
                    &container_name(kortnavn),
                    Some(bollard::container::UploadToContainerOptions {
                        path: "/home/coder/project",
                        ..Default::default()
                    }),
                    bytes.into(),
                )
                .await?;
                Ok(())
            }
        }
    }

    /// Fjerner container + prosjektvolum. Del av sletteregimet
    /// («sletting er sletting»).
    pub async fn remove_workspace(&self, kortnavn: &str) -> anyhow::Result<()> {
        match self {
            Driver::Mock(m) => {
                m.lock().unwrap().remove(kortnavn);
                Ok(())
            }
            Driver::Docker(d) => {
                let navn = container_name(kortnavn);
                if let Err(e) = d
                    .remove_container(
                        &navn,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await
                {
                    tolerer(e, &[404])?;
                }
                // Volumet kan være «in use» (409) et øyeblikk etter at
                // containeren force-fjernes — prøv igjen noen ganger.
                let mut forsok = 0;
                loop {
                    match d
                        .remove_volume(&navn, Some(RemoveVolumeOptions { force: true }))
                        .await
                    {
                        Ok(_) => break,
                        Err(e) => {
                            if let bollard::errors::Error::DockerResponseServerError {
                                status_code, ..
                            } = &e
                            {
                                if *status_code == 404 {
                                    break;
                                }
                                if *status_code == 409 && forsok < 5 {
                                    forsok += 1;
                                    tokio::time::sleep(std::time::Duration::from_millis(500))
                                        .await;
                                    continue;
                                }
                            }
                            return Err(e.into());
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
