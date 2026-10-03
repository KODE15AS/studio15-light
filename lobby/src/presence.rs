// Presence-laget (V2): se/peke/ta over per prosjekt.
//
// Designet fra Skjermsamling (erfaringsoverføring C), gjenbrukt uendret:
//   - abonner på broadcast FØR join, roster-snapshot direkte etter welcome
//   - reconnect med session-token + generasjonsteller på disconnect-timeout
//   - tile-relative cursor-koordinater (tile-id + normalisert x/y)
//   - veggen har watch-modus: joiner aldri, sender aldri input
//   - kun ÉN ekstern controller per arbeidsflate; ta/slipp broadcastes
//
// Avklart 03.10 (chat 2): «ta over» realiseres med code-servers
// flerbrukertilkobling (samme /w/<flate>/-URL), IKKE input-streaming.
// Kontroll her er derfor ren TILSTAND (rammer + badges på vegg og hos
// deltagerne) — selve overtagelsen er at klienten slår av peke-overlayet
// og slipper input inn i kollegaens iframe.
//
// Ingen database: alt bor i minnet. Lobby-restart nullstiller presence —
// klientene reconnecter og joiner på nytt (session-tokenet avvises stille).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

/// Eierfargene fra webprofil-kode15/Skjermsamling: faste, lesbare på mørk
/// stage. Stabil per navn innenfor rommets levetid (fargeminne).
const FARGER: &[&str] = &[
    "#E8A33D", // rav
    "#5FA8D3", // lyseblå
    "#9BC53D", // grønn
    "#E55381", // rosa
    "#B07CC6", // lilla
    "#5AC8B0", // turkis
];

#[derive(Clone, Serialize)]
pub struct Deltager {
    pub id: Uuid,
    pub navn: String,
    /// slugify(navn) — kobler deltageren til arbeidsflaten med samme
    /// deltager-slug (kortnavnets siste ledd).
    pub slug: String,
    pub farge: String,
    pub tilkoblet: bool,
    #[serde(skip)]
    pub disconnect_gen: u64,
}

#[derive(Clone, Serialize)]
pub struct Kontroll {
    pub id: Uuid,
    pub navn: String,
    pub farge: String,
}

/// Klient → server
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum KlientMelding {
    Join {
        navn: String,
        /// Session-token fra tidligere besøk — reconnect uten ny identitet.
        session: Option<Uuid>,
    },
    Cursor {
        /// Tile-id, f.eks. "<kortnavn>:editor" — opak for serveren.
        tile: String,
        x: f64,
        y: f64,
    },
    Ta {
        /// Arbeidsflatens kortnavn.
        flate: String,
    },
    Slipp,
    Forlat,
}

/// Server → klient
#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ServerMelding<'a> {
    Velkommen {
        deg: &'a Deltager,
        session: Uuid,
    },
    Roster {
        deltagere: Vec<Deltager>,
        /// kortnavn → hvem som kontrollerer flaten utenfra.
        kontroll: HashMap<String, Kontroll>,
    },
    Cursor {
        id: Uuid,
        navn: &'a str,
        farge: &'a str,
        tile: &'a str,
        x: f64,
        y: f64,
    },
    Feil {
        melding: String,
    },
}

struct RomInner {
    deltagere: HashMap<Uuid, Deltager>,
    sessions: HashMap<Uuid, Uuid>,
    /// kortnavn → ekstern controller (deltager-id). Kun ÉN per flate.
    kontroll: HashMap<String, Uuid>,
    /// navn → farge, så samme navn får samme farge gjennom rommets levetid.
    fargeminne: HashMap<String, String>,
}

/// Ett rom per prosjekt (program/prosjekt).
pub struct Rom {
    pub tx: broadcast::Sender<String>,
    inner: Mutex<RomInner>,
}

impl Rom {
    fn ny() -> Arc<Rom> {
        let (tx, _) = broadcast::channel(512);
        Arc::new(Rom {
            tx,
            inner: Mutex::new(RomInner {
                deltagere: HashMap::new(),
                sessions: HashMap::new(),
                kontroll: HashMap::new(),
                fargeminne: HashMap::new(),
            }),
        })
    }

    fn broadcast(&self, melding: String) {
        let _ = self.tx.send(melding);
    }

    pub fn roster_json(&self) -> String {
        let inner = self.inner.lock().unwrap();
        let mut deltagere: Vec<Deltager> = inner.deltagere.values().cloned().collect();
        deltagere.sort_by(|a, b| a.navn.cmp(&b.navn));
        let kontroll = inner
            .kontroll
            .iter()
            .filter_map(|(flate, id)| {
                inner.deltagere.get(id).map(|d| {
                    (
                        flate.clone(),
                        Kontroll {
                            id: d.id,
                            navn: d.navn.clone(),
                            farge: d.farge.clone(),
                        },
                    )
                })
            })
            .collect();
        serde_json::to_string(&ServerMelding::Roster { deltagere, kontroll }).unwrap()
    }

    pub fn broadcast_roster(&self) {
        let melding = self.roster_json();
        self.broadcast(melding);
    }

    /// Join eller reconnect. Returnerer (id, session-token, welcome-json).
    pub fn join(&self, navn: String, slug: String, session: Option<Uuid>) -> (Uuid, Uuid, String) {
        let mut inner = self.inner.lock().unwrap();

        // Reconnect: kjent session-token → samme identitet, avbrutt opprydding.
        if let Some(tok) = session {
            if let Some(&id) = inner.sessions.get(&tok) {
                if let Some(d) = inner.deltagere.get_mut(&id) {
                    d.tilkoblet = true;
                    d.disconnect_gen += 1; // avbryter ventende timeout-task
                    d.navn = navn;
                    d.slug = slug;
                    let welcome =
                        serde_json::to_string(&ServerMelding::Velkommen { deg: d, session: tok })
                            .unwrap();
                    return (id, tok, welcome);
                }
            }
        }

        // Ny deltager: stabil farge per navn, ellers første ledige.
        let i_bruk: Vec<String> = inner.deltagere.values().map(|d| d.farge.clone()).collect();
        let farge = inner
            .fargeminne
            .get(&navn)
            .cloned()
            .filter(|f| !i_bruk.contains(f))
            .or_else(|| {
                FARGER
                    .iter()
                    .map(|f| f.to_string())
                    .find(|f| !i_bruk.contains(f))
            })
            .unwrap_or_else(|| FARGER[inner.deltagere.len() % FARGER.len()].to_string());
        inner.fargeminne.insert(navn.clone(), farge.clone());

        let id = Uuid::new_v4();
        let tok = Uuid::new_v4();
        let d = Deltager {
            id,
            navn,
            slug,
            farge,
            tilkoblet: true,
            disconnect_gen: 0,
        };
        let welcome =
            serde_json::to_string(&ServerMelding::Velkommen { deg: &d, session: tok }).unwrap();
        inner.deltagere.insert(id, d);
        inner.sessions.insert(tok, id);
        (id, tok, welcome)
    }

    pub fn broadcast_cursor(&self, id: Uuid, tile: &str, x: f64, y: f64) {
        let inner = self.inner.lock().unwrap();
        let Some(d) = inner.deltagere.get(&id) else { return };
        let melding = serde_json::to_string(&ServerMelding::Cursor {
            id,
            navn: &d.navn,
            farge: &d.farge,
            tile,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        })
        .unwrap();
        drop(inner);
        self.broadcast(melding);
    }

    /// Ta kontroll over en flate. `eier_slug` er flatens deltager-ledd —
    /// handleren har validert at flaten finnes i prosjektet.
    pub fn ta(&self, hvem: Uuid, flate: &str, eier_slug: &str) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap();
        let Some(d) = inner.deltagere.get(&hvem) else {
            return Err("du har ikke joinet".into());
        };
        if d.slug == eier_slug {
            return Err("det er din egen arbeidsflate — du har alltid kontroll".into());
        }
        match inner.kontroll.get(flate) {
            Some(andre) if *andre != hvem => {
                let navn = inner
                    .deltagere
                    .get(andre)
                    .map(|a| a.navn.clone())
                    .unwrap_or_else(|| "noen".into());
                return Err(format!("{navn} kontrollerer denne flaten allerede"));
            }
            _ => {}
        }
        // Én kontroll per deltager: slipp eventuell forrige flate.
        inner.kontroll.retain(|_, id| *id != hvem);
        inner.kontroll.insert(flate.to_string(), hvem);
        drop(inner);
        self.broadcast_roster();
        Ok(())
    }

    pub fn slipp(&self, hvem: Uuid) {
        let mut inner = self.inner.lock().unwrap();
        let hadde = inner.kontroll.values().any(|id| *id == hvem);
        inner.kontroll.retain(|_, id| *id != hvem);
        drop(inner);
        if hadde {
            self.broadcast_roster();
        }
    }

    /// Markér frakoblet; returnerer generasjonsnummeret timeout-tasken
    /// skal sjekke mot (reconnect øker telleren og ugyldiggjør tasken).
    pub fn marker_frakoblet(&self, id: Uuid) -> Option<u64> {
        let mut inner = self.inner.lock().unwrap();
        let d = inner.deltagere.get_mut(&id)?;
        d.tilkoblet = false;
        d.disconnect_gen += 1;
        let gen = d.disconnect_gen;
        drop(inner);
        self.broadcast_roster();
        Some(gen)
    }

    pub fn fortsatt_frakoblet(&self, id: Uuid, gen: u64) -> bool {
        let inner = self.inner.lock().unwrap();
        matches!(
            inner.deltagere.get(&id),
            Some(d) if !d.tilkoblet && d.disconnect_gen == gen
        )
    }

    /// Fjern deltageren helt (timeout eller eksplisitt forlat) og slipp
    /// all kontroll vedkommende hadde.
    pub fn fjern(&self, id: Uuid) {
        let mut inner = self.inner.lock().unwrap();
        inner.deltagere.remove(&id);
        inner.sessions.retain(|_, v| *v != id);
        inner.kontroll.retain(|_, v| *v != id);
        drop(inner);
        self.broadcast_roster();
    }
}

/// Presence-huben: ett rom per prosjekt, opprettet ved behov.
pub struct Presence {
    rom: Mutex<HashMap<String, Arc<Rom>>>,
    /// Sekunder en frakoblet deltager beholdes før opprydding.
    pub timeout_secs: u64,
}

impl Presence {
    pub fn ny() -> Presence {
        let timeout_secs = std::env::var("PRESENCE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);
        Presence {
            rom: Mutex::new(HashMap::new()),
            timeout_secs,
        }
    }

    pub fn rom(&self, program: &str, prosjekt: &str) -> Arc<Rom> {
        let nokkel = format!("{program}/{prosjekt}");
        self.rom
            .lock()
            .unwrap()
            .entry(nokkel)
            .or_insert_with(Rom::ny)
            .clone()
    }
}
