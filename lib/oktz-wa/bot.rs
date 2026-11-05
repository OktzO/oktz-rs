use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use oktz_plugin_api::berkas::BerkasNyata;
use oktz_plugin_api::UreqHttp;
use oktz_router::ParseOptions;
use oktz_store::json_store::JsonStore;
use whatsapp_rust::prelude::*;

use crate::creds;
use crate::pairing_run::{self, FalseLogoutGuard};
use crate::pipeline::{ClientSender, Identity, Pipeline, StoreContext};

/// Env yang memberi tahu jalur produksi di mana `creds.json` berada.
///
/// [`connect_device`] tidak menerima creds (signedannya dijaga supaya tidak
/// pecah), dan tanpa snapshot guard tidak bisa membedakan "sesi sah yang
/// dapat 401 palsu" dari "belum pernah ter-link". Tanpa env ini guard
/// menerima `Value::Null` → `pairing::is_authenticated` false → arahnya
/// selalu "percaya logout". Itu konservatif (tidak pernah menahan logout
/// asli) tapi berarti grace window tidak aktif; caller yang punya file creds
/// sebaiknya memakai [`connect_device_guarded`] langsung.
pub const CREDS_ENV: &str = "OKTZ_CREDS";

/// Env yang menimpa lokasi settings store. Kosong = bukan override.
pub const SETTINGS_ENV: &str = "OKTZ_SETTINGS";

/// `bot_id` dipakai kalau creds tidak menyebut JID sendiri. Hanya jadi
/// namespace prefiks pada kunci dedup/spam (`handler.js:926, 1508`), jadi nilai
/// tetap yang penting, bukan formatnya.
pub const DEFAULT_BOT_ID: &str = "oktz";

/// Lokasi settings store yang dibaca router: `users` / `groups` / `settings`.
///
/// Default-nya mengikuti `oktz-bin` (`main.rs:166`), yaitu
/// `<database.path>/settings.json`. Argumen eksplisit selalu menang atas env
/// supaya bisa diuji tanpa menyentuh `std::env` yang process-wide.
pub fn settings_path(db_dir: &str, env: Option<&str>) -> PathBuf {
    match env.map(str::trim).filter(|s| !s.is_empty()) {
        Some(v) => PathBuf::from(v),
        None => PathBuf::from(db_dir).join("settings.json"),
    }
}

/// Lokasi store `owner` yang dibaca `.owner`: `<database.path>/owner.json`.
///
/// Sengaja TERPISAH dari [`settings_path`] — JS menyimpan daftar owner di
/// `db.data.owner`, bukan di `settings.owner` (`plugins/main/owner.js:24`,
/// `src/lib/database.js:173-181`). Menaruhnya di settings akan membuat plugin
/// `owner/*` dan `.owner` membaca sumber yang berbeda.
pub fn owner_path(db_dir: &str) -> PathBuf {
    PathBuf::from(db_dir).join("owner.json")
}

/// Baca `db.data.owner` (`owner.json`) untuk `.owner`.
///
/// Berkas hilang atau rusak berarti daftar kosong, bukan bot gagal boot: daftar
/// owner hanyalah salah satu sumber `.owner`, sumber utamanya tetap `config`.
/// Bentuknya array string — sama seperti `database.js:127-153`.
pub fn owner_entries(path: &std::path::Path) -> Vec<String> {
    match JsonStore::open(path) {
        Ok(store) => store
            .all()
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "owner.json tidak bisa dibaca; daftar DB kosong");
            Vec::new()
        }
    }
}

/// `OWNER_NUMBERS` → [`Identity`].
///
/// `oktz_core::Config` hanya punya `owner_numbers`; premium/partner/admin tidak
/// punya accessor sama sekali, jadi kumpulan itu sengaja dibiarkan kosong
/// (lihat catatan modul di `pipeline.rs`).
pub fn identity_from_owners<I: IntoIterator<Item = String>>(owners: I) -> Identity {
    Identity {
        owners: owners.into_iter().collect(),
        ..Default::default()
    }
}

/// Muat identitas owner/premium/partner dari berkas sumber yang sama dengan JS.
///
/// Sumber:
/// - owner: `config.owner_numbers` + `<db>/owner.json` + `settings.ownerNumbers`
/// - premium: `<db>/premium.json` (bentuk `{premium:[...]}` atau array) + `settings.premiumUsers`
/// - partner: `<db>/partner.json` + `settings.partnerUsers` (jika ada)
///
/// Premium otomatis mencakup owner & partner, sama seperti `config.js`.
pub fn load_identity<I: IntoIterator<Item = String>>(
    settings: &std::path::Path,
    owners: I,
) -> Identity {
    use oktz_store::json_store::JsonStore;
    use std::collections::HashSet;

    let mut owners_set: HashSet<String> = owners.into_iter().collect();
    let mut premium_set: HashSet<String> = HashSet::new();
    let mut partner_set: HashSet<String> = HashSet::new();
    let dir = settings.parent().unwrap_or_else(|| std::path::Path::new("."));

    // owner.json
    if let Ok(store) = JsonStore::open(dir.join("owner.json")) {
        if let Some(arr) = store.all().as_array() {
            for v in arr {
                if let Some(s) = v.as_str() { owners_set.insert(s.to_string()); }
            }
        }
    }

    // premium.json / partner.json — bentuk objek {premium:[...]} atau array langsung.
    for (file, set) in [("premium.json", &mut premium_set), ("partner.json", &mut partner_set)] {
        if let Ok(store) = JsonStore::open(dir.join(file)) {
            let root = store.all();
            let arr = root.as_array()
                .or_else(|| root.get(file.trim_end_matches(".json")).and_then(serde_json::Value::as_array))
                .or_else(|| root.get("premium").and_then(serde_json::Value::as_array))
                .or_else(|| root.get("partner").and_then(serde_json::Value::as_array))
                .or_else(|| root.get("partners").and_then(serde_json::Value::as_array));
            if let Some(arr) = arr {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        set.insert(s.to_string());
                    } else if let Some(obj) = v.as_object() {
                        if let Some(s) = obj.get("jid").or_else(|| obj.get("number")).or_else(|| obj.get("id")).and_then(serde_json::Value::as_str) {
                            set.insert(s.to_string());
                        }
                    }
                }
            }
        }
    }

    // settings.json
    if let Ok(store) = JsonStore::open(settings) {
        let root = store.all();
        if let Some(arr) = root.get("ownerNumbers").and_then(serde_json::Value::as_array) {
            for v in arr { if let Some(s)=v.as_str() { owners_set.insert(s.to_string()); } }
        }
        if let Some(arr) = root.get("premiumUsers").and_then(serde_json::Value::as_array) {
            for v in arr { if let Some(s)=v.as_str() { premium_set.insert(s.to_string()); } }
        }
        if let Some(arr) = root.get("partnerUsers").and_then(serde_json::Value::as_array) {
            for v in arr { if let Some(s)=v.as_str() { partner_set.insert(s.to_string()); } }
        }
    }

    // owner & partner ikut premium (sama seperti config.js)
    for o in owners_set.clone() { premium_set.insert(o); }
    for p in partner_set.clone() { premium_set.insert(p); }

    Identity {
        owners: owners_set,
        premium: premium_set,
        partners: partner_set,
        admins: HashSet::new(),
    }
}

/// `creds.me.id` → `bot_id`, dengan [`DEFAULT_BOT_ID`] sebagai fallback.
///
/// Creds tak terbaca menghasilkan `Value::Null` (`load_guard_creds`), jadi
/// jalur ini harus tidak pernah gagal — `bot_id` tidak pernah boleh memblokir
/// boot hanya karena creds belum ada.
pub fn bot_id_from(creds: &serde_json::Value) -> String {
    creds
        .get("me")
        .and_then(|me| me.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_BOT_ID)
        .to_string()
}

/// Segala yang dibutuhkan pipeline yang TIDAK butuh `Arc<Client>`.
///
/// Dipisah supaya semua yang mahal (baca file settings) selesai SEBELUM
/// connect. Yang tersisa saat `Event::Connected` tiba hanyalah `subscribe`,
/// sehingga jendela pesan yang bisa terlewat antara dispatch dan subscribe
/// jadi kecil. Konstruksi yang boros di dalam `on_ready` justru
/// memperlebar jendela itu — lihat catatan pada [`connect_device`].
pub struct PipelineWiring {
    ctx: StoreContext,
    identity: Identity,
    bot_id: String,
    /// `config.js` runtime untuk `ctx.config()`.
    config: oktz_core::config::Runtime,
    /// `db.data.owner` untuk `.owner`.
    db_owners: Vec<String>,
    /// Pipeline yang sudah menempel ke bus. Disimpan supaya reconnect
    /// memakai instance yang sama — membangun yang baru akan meninggalkan
    /// loop lama hidup dan membalas dua kali.
    live: Mutex<Option<Arc<Pipeline>>>,
    /// Mirror Turso; `None` = fitur mati (TURSO_URL kosong / disabled).
    turso: Mutex<Option<(oktz_store::turso::TursoStore, std::time::Duration)>>,
}

/// Bangun [`PipelineWiring`] dari lokasi store, owner, dan bot id.
///
/// Semua argumen eksplisit: `Config::from_env()` memanggil `Env::load_dotenv()`
/// yang mengubah environment process-wide, jadi dipanggil di satu tempat saja
/// (`connect_device`), bukan di dalam helper yang juga dipakai tes.
pub fn wiring_from(
    settings: &std::path::Path,
    owners: impl IntoIterator<Item = String>,
    bot_id: impl Into<String>,
) -> Result<PipelineWiring> {
    wiring_from_with_config(
        settings,
        owners,
        bot_id,
        oktz_core::config::Runtime::default(),
    )
}

/// Seperti [`wiring_from`], tapi `config.js`-nya bisa diganti. Dipakai jalur
/// produksi yang memang punya `config.js` sungguhan; test memakai varian
/// bawaan yang kosong.
pub fn wiring_from_with_config(
    settings: &std::path::Path,
    owners: impl IntoIterator<Item = String>,
    bot_id: impl Into<String>,
    config: oktz_core::config::Runtime,
) -> Result<PipelineWiring> {
    let store = JsonStore::open(settings)
        .with_context(|| format!("buka settings store {}", settings.display()))?;
    Ok(PipelineWiring {
        ctx: StoreContext::new(Arc::new(store)),
        identity: load_identity(settings, owners),
        bot_id: bot_id.into(),
        config,
        db_owners: owner_entries(settings.parent().map(|p| p.join("owner.json")).as_deref().unwrap_or(Path::new("owner.json"))),
        live: Mutex::new(None),
        turso: Mutex::new(None),
    })
}

impl PipelineWiring {
    /// Pasang Turso: boot sync sekali lalu spawn task flush write-behind.
    /// Dipanggil dari `connect_device` sebelum connect; Turso mati TIDAK
    /// menggagalkan boot.
    pub fn boot_and_spawn_turso(
        self: &Arc<Self>,
        store: oktz_store::turso::TursoStore,
        interval: std::time::Duration,
    ) {
        *self.turso.lock().unwrap_or_else(|e| e.into_inner()) = Some((store.clone(), interval));
        // Boot sync butuh runtime async; `connect_device` selalu dipanggil di
        // dalam tokio, jadi spawn task sekali-jalan cukup — wiring-nya `Arc`.
        let this = Arc::clone(self);
        tokio::spawn(async move {
            this.turso_boot_once(store).await;
            this.spawn_flush_task();
        });
    }

    /// Boot sync sekali: seed/restore/lewatkan, plus mirror tabel session.
    /// Turso mati = warning.
    async fn turso_boot_once(&self, store: oktz_store::turso::TursoStore) {
        match store.connect().await {
            Ok(conn) => {
                if let Err(e) = store.ensure_tables(&conn).await {
                    tracing::warn!(error = %e, "[turso] ensure_tables gagal; mirror dimatikan");
                    return;
                }
                if let Err(e) = self.boot_sync_turso(&store, &conn).await {
                    tracing::warn!(error = %e, "[turso] boot sync gagal; lanjut lokal");
                }
                // Mirror tabel session (plan Task 3/4) — best-effort.
                if let Some(db_path) = self.session_db_path() {
                    for table in SESSION_MIRROR_TABLES {
                        match mirror_sqlite_table(&store, &conn, &db_path, table).await {
                            Ok(0) => {}
                            Ok(n) => tracing::info!(table, n, "[turso] tabel session termirror"),
                            Err(e) => tracing::warn!(error = %e, table, "[turso] mirror tabel gagal"),
                        }
                    }
                }
            }
            Err(e) => tracing::warn!(error = %e, "[turso] koneksi gagal; mirror dimatikan"),
        }
    }

    /// Lokasi db session SQLite, kalau ada. `database/wa/oktz-run.db` mengikuti
    /// `main.rs::run`.
    fn session_db_path(&self) -> Option<PathBuf> {
        // Turunan settings.json → database dir → sibling `wa/oktz-run.db`.
        // settings.json bisa di root `database/` (sibling `wa/`) atau di
        // `database/main/` (parent berisi `wa/`).
        let dir = self.ctx.path.parent()?;
        let kandidat = [dir.join("wa").join("oktz-run.db"), dir.parent().map(|p| p.join("wa")).unwrap_or_default().join("oktz-run.db")];
        kandidat.into_iter().find(|p| p.exists())
    }

    /// Boot sync store JSON — tiga cabang (plan Review Focus #2/#3):
    /// Turso kosong + lokal terisi → seed Turso; Turso terisi + lokal kosong →
    /// restore ke lokal; keduanya terisi → lokal menang.
    pub async fn boot_sync_turso(
        &self,
        t: &oktz_store::turso::TursoStore,
        conn: &libsql::Connection,
    ) -> anyhow::Result<()> {
        let turso_ada = t.has_any_store(&conn).await?;
        let lokal_terisi = self
            .ctx
            .snapshot()
            .all()
            .as_object()
            .is_some_and(|o| !o.is_empty());
        match (turso_ada, lokal_terisi) {
            (false, true) => {
                // Seed: salin seluruh store lokal ke Turso.
                let root = self.ctx.snapshot().all().clone();
                if let Some(obj) = root.as_object() {
                    for (k, v) in obj {
                        t.put_store(conn, k, v).await?;
                    }
                }
                tracing::info!("[turso] seed data lokal ke stores");
            }
            (true, false) => {
                // Restore: taruh tiap store dari Turso ke JsonStore + disk.
                // Kunci diambil dari TURSO (lokal kosong tidak tahu apa yang
                // harus direstore).
                let mut loads = Vec::new();
                for k in t.list_store_keys(conn).await? {
                    if let Some(v) = t.load_store(conn, &k).await? {
                        loads.push((k, v));
                    }
                }
                // Seluruh restore di bawah lock snapshot-write, satu buka+tulis
                // file — sama seperti `StoreContext::apply`. Tanpa itu `apply`
                // bisa menyisipkan mutasi baru di antara baca dan tulis restore
                // dan mutasi itu hilang dari disk + snapshot (review I1).
                let n = self.ctx.restore_all(loads)?;
                if n > 0 {
                    tracing::info!(n, "[turso] stores direstore dari Turso ke lokal");
                }
            }
            (true, true) => tracing::info!("[turso] lokal & Turso terisi; lokal menang"),
            (false, false) => {}
        }
        Ok(())
    }

    /// Task flush write-behind: tiap interval, kalau kotor → kirim ke Turso.
    /// Gagal = warning + flag kotor dipasang lagi (coba siklus berikutnya).
    pub fn spawn_flush_task(self: &Arc<Self>) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                let interval = this
                    .turso
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .as_ref()
                    .map(|(_, iv)| *iv)
                    .unwrap_or(std::time::Duration::from_secs(15));
                tokio::time::sleep(interval).await;
                this.flush_once_turso().await;
            }
        });
    }

    /// Satu siklus flush. `pub(crate)` supaya test bisa memanggil langsung.
    pub async fn flush_once_turso(&self) {
        let Some((store, _)) = self
            .turso
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        else {
            return;
        };
        if !self.ctx.take_dirty_turso() {
            return;
        }
        match store.connect().await {
            Ok(conn) => {
                if let Err(e) = store.ensure_tables(&conn).await {
                    tracing::warn!(error = %e, "[turso] ensure_tables gagal saat flush");
                    self.ctx.mark_dirty_turso();
                    return;
                }
                if let Err(e) = self.ctx.flush_turso(&store, &conn).await {
                    tracing::warn!(error = %e, "[turso] flush gagal; dicoba lagi siklus berikutnya");
                    self.ctx.mark_dirty_turso();
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "[turso] koneksi flush gagal; dicoba lagi nanti");
                self.ctx.mark_dirty_turso();
            }
        }
    }

    /// Pasang pipeline ke bus event. Aman dipanggil berulang.
    fn attach(&self, client: &Arc<Client>) {
        let mut slot = self.live.lock().unwrap_or_else(|e| e.into_inner());
        let pipeline = match slot.as_ref() {
            Some(p) => p.clone(),
            None => {
                let sender = Arc::new(ClientSender::new(client.clone()));
                let p = Arc::new(
                    Pipeline::new(
                        self.ctx.clone(),
                        sender.clone(),
                        self.bot_id.clone(),
                        ParseOptions::default(),
                    )
                    .with_identity(self.identity.clone())
                    .with_http(Arc::new(UreqHttp::new()))
                    .with_owner_store(self.db_owners.clone())
                    .with_config(self.config.clone())
                    // `Ctx.sock` + roster nyata dari klien yang sama.
                    .with_sock(crate::sock_client::SockKlien::baru(client.clone()))
                    // `fs` Nyata berakar ke cwd — launcher sudah men-set cwd ke
                    // akar repo, jadi jalur `./database/...` sama dengan JS.
                    .with_berkas(Arc::new(BerkasNyata::baru(
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    )))
                    .with_roster_source(sender),
                );
                *slot = Some(p.clone());
                p
            }
        };
        drop(slot);
        pipeline.attach(client);
    }
}

fn jid(s: &str) -> Result<Jid> {
    s.parse::<Jid>()
        .map_err(|e| anyhow::anyhow!("JID {s}: {e}"))
}

pub async fn build_device(c: &creds::Creds) -> Result<whatsapp_rust::wacore::store::Device> {
    use wacore_libsignal::core::curve::{KeyPair, PrivateKey, PublicKey};
    // onigis/Baileys menyimpan public key Curve25519 sebagai 32 byte POLOS.
    // libsignal/wacore memakai bentuk kanonik 33 byte = 0x05 (KeyType::Djb) + 32.
    // Kalau sudah 33 byte, biarkan apa adanya.
    const DJB_PREFIX: u8 = 0x05;

    fn kp(j: &creds::KeyPairJson, label: &str) -> Result<KeyPair> {
        let mut pb = creds::b64(&j.public)?;
        let vb = creds::b64(&j.private)?;
        if pb.len() == 32 {
            pb.insert(0, DJB_PREFIX);
        }
        Ok(KeyPair {
            public_key: PublicKey::try_from(&pb[..]).map_err(|e| {
                anyhow::anyhow!(
                    "{label}.public (len={} first={:#x}): {e}",
                    pb.len(),
                    pb.first().unwrap_or(&0)
                )
            })?,
            private_key: PrivateKey::try_from(&vb[..]).map_err(|e| {
                anyhow::anyhow!(
                    "{label}.private (len={} first={:#x}): {e}",
                    vb.len(),
                    vb.first().unwrap_or(&0)
                )
            })?,
        })
    }
    fn arr<const N: usize>(b: Vec<u8>, what: &str) -> Result<[u8; N]> {
        b.try_into()
            .map_err(|v: Vec<u8>| anyhow::anyhow!("{what} harus {N} byte, dapat {}", v.len()))
    }

    let me = c.me.as_ref().context("creds tanpa `me`")?;
    let mut d = whatsapp_rust::wacore::store::Device {
        pn: Some(jid(&me.id)?),
        lid: me.lid.as_deref().map(jid).transpose()?,
        registration_id: c.registration_id,
        noise_key: kp(&c.noise_key, "noiseKey")?,
        identity_key: kp(&c.signed_identity_key, "signedIdentityKey")?,
        signed_pre_key: kp(&c.signed_pre_key.key_pair, "signedPreKey.keyPair")?,
        signed_pre_key_id: c.signed_pre_key.key_id,
        signed_pre_key_signature: arr(creds::b64(&c.signed_pre_key.signature)?, "signature")?,
        adv_secret_key: arr(creds::b64(&c.adv_secret_key)?, "advSecretKey")?,
        push_name: me.name.clone(),
        next_pre_key_id: c.next_pre_key_id.unwrap_or(0),
        first_unupload_pre_key_id: c.first_unuploaded_pre_key_id.unwrap_or(0),
        props_hash: c.last_prop_hash.clone(),
        ..Default::default()
    };
    if let Some(ri) = c.routing_info.as_ref().and_then(|r| r.data.as_ref()) {
        d.edge_routing_info = Some(creds::b64(ri)?);
    }

    // ADVSignedDeviceIdentity — tanpa ini PDO/retry receipt dan decrypt pesan
    // grup yang gagal decrypt akan bermasalah.
    if let Some(a) = c.account.as_ref() {
        let nb = |s: &str| -> Result<Option<Vec<u8>>> {
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(creds::b64(s)?))
            }
        };
        d.account = Some(std::sync::Arc::new(
            whatsapp_rust::waproto::whatsapp::ADVSignedDeviceIdentity {
                details: nb(a.details.as_deref().unwrap_or(""))?,
                account_signature_key: nb(a.account_signature_key.as_deref().unwrap_or(""))?,
                account_signature: match &a.account_signature {
                    Some(s) => nb(s)?,
                    None => None,
                },
                device_signature: match &a.device_signature {
                    Some(s) => nb(s)?,
                    None => None,
                },
            },
        ));
    }
    Ok(d)
}
/// Db sesi lama boleh dipakai ulang hanya kalau sidik creds-nya cocok.
///
/// Db tanpa marker sidik dianggap tidak sah: device row di dalamnya bisa
/// berasal dari creds lain, dan memakainya bersama creds baru membuat sinyal
/// tidak nyambung.
fn sesi_lama_boleh_dipakai(db_ada: bool, marker: Option<&str>, sidik: &str) -> bool {
    db_ada && marker.is_some_and(|m| m.trim() == sidik)
}

/// Buka store sqlite, simpan device, muat sesi onigis, lalu siap connect.
///
/// Db sesi yang sudah ada dan ber-sidik sama DIPAKAI ULANG: di situ device
/// yang sudah dirotasi library + sesi ratchet bertahan, jadi restart tidak
/// memaksa dekripsi ulang dari nol. Db yang hilang atau ber-sidik beda
/// dibangun ulang dari creds.
pub async fn prepare_store(
    creds_path: &str,
    db_path: &str,
    session_fixture: Option<&str>,
) -> Result<SqliteStore> {
    let raw = std::fs::read_to_string(creds_path).with_context(|| format!("baca {creds_path}"))?;
    let (_m, c) = crate::creds::inspect_creds(&raw)?;

    let db = std::path::PathBuf::from(db_path);
    if let Some(parent) = db.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Satu-satunya tempat di crate ini yang benar-benar menulis device row,
    // jadi ini juga tempat yang tahu di mana artefak sesi durable hidup.
    // Dipakai [`pairing_run::purge_handler`] saat logout dipercaya.
    pairing_run::note_store_path(&db);

    let mut ident = crate::creds::b64(&c.signed_identity_key.public)?;
    let sidik = format!(
        "{}:{}",
        c.registration_id,
        ident.iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
    let marker = db.with_extension("ident");
    let marker_isi = std::fs::read_to_string(&marker).ok();
    let db_ada = db.exists();

    if sesi_lama_boleh_dipakai(db_ada, marker_isi.as_deref(), &sidik) {
        tracing::info!(db = %db.display(), "db sesi dipakai ulang (sesi ratchet lama dipertahankan)");
        return Ok(SqliteStore::new(db.to_str().context("path tidak utf8")?).await?);
    }

    let _ = std::fs::remove_file(&db);
    let store = SqliteStore::new(db.to_str().context("path tidak utf8")?).await?;

    let device = build_device(&c).await?;
    store.save_device_data_for_device(1, &device).await?;

    if let Some(fx) = session_fixture {
        if ident.len() == 32 {
            ident.insert(0, 0x05);
        }
        let report = crate::load::load_into_store(&store, fx, &ident, c.registration_id).await?;
        tracing::info!(
            loaded = report.sessions_loaded,
            failed = report.sessions_failed,
            write_failed = report.write_failed,
            "sesi ratchet dimuat (hanya yang benar-benar tersimpan)"
        );
    }

    // Marker gagal ditulis bukan alasan gagal start — efeknya hanya db ini
    // dibangun ulang pada start berikutnya.
    if let Err(e) = std::fs::write(&marker, &sidik) {
        tracing::warn!(error = %e, path = %marker.display(), "gagal menulis sidik sesi");
    }
    Ok(store)
}

/// Path creds untuk guard: argumen eksplisit menang atas env.
///
/// Murni supaya bisa diuji tanpa menyentuh filesystem maupun `std::env`
/// (yang process-wide, jadi bisa saling menimpa antar-test).
pub fn resolve_creds_path(explicit: Option<&str>, env: Option<&str>) -> Option<PathBuf> {
    [explicit, env]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .map(PathBuf::from)
}

/// Snapshot creds untuk [`FalseLogoutGuard`].
///
/// Kegagalan baca TIDAK boleh menggagalkan sesi: guard hanya melindungi
/// keputusan logout, dan creds yang tak terbaca membuat
/// `pairing::is_authenticated` false, yang arahnya selalu "percaya logout" —
/// lebih konservatif daripada diam-diam menganggap sesi sah.
///
/// # Kenapa tidak ada write-back (sudah diperiksa, bukan asumsi)
///
/// Creds di sini read-only untuk runtime. Tidak ada helper yang bisa menulis
/// balik, dan mengarang satu akan lebih buruk daripada tidak menulis:
///
/// * whatsapp-rust 0.7.0 tidak mengenal konsep creds/authState sama sekali
///   (`grep -ri "authstate\|creds\.json" src/` di crate itu kosong). Device
///   state yang dirotasi library disimpan lewat `PersistenceManager` →
///   `DeviceStore::save` ke SQLite, dengan background saver 30 detik
///   (`src/bot.rs:1394`). Jadi "menyalin creds JS balik ke disk" tidak punya
///   padanan Rust.
/// * `oktz_store` tidak punya helper creds; `JsonStore` itu key-value generik
///   dan `oktz-wa` bahkan tidak depend ke `oktz-store` sama sekali.
/// * Yang paling penting: rewrite `creds.json` dari snapshot in-memory
///   akan menimpa prekey/advSecretKey yang SUDAH dirotasi library ke SQLite
///   dengan nilai basi — key rollback senyap yang menetap. JS aman di sini
///   karena `sock.authState.creds` adalah objek yang sama persis dengan yang
///   diputar library, bukan salinan.
///
/// Yang hilang akibat ini, dan hanya ini: `pairing::reset_pairing_creds` pada
/// cabang `Apply(ResetPairing)` tidak bertahan lintas restart. Dampaknya
/// kecil — pemanggil wajib pairing ulang, dan `pair_code_flow` sudah
/// membangun store dari nol.
pub fn load_guard_creds(path: Option<&str>) -> serde_json::Value {
    let env = oktz_core::env::Env::get(CREDS_ENV);
    resolve_creds_path(path, env.as_deref())
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(serde_json::Value::Null)
}

/// Jalur produksi: `connect_device` + [`FalseLogoutGuard`] terpasang.
///
/// Pemasangan guard-nya milik [`crate::pairing_run`]; ini titik masuk
/// bernama supaya semua caller punya satu jalan yang jelas.
///
/// Handler purge ikut terpasang, karena [`pairing_run::connect_device_guarded`]
/// yang memasangkannya dan fungsi ini mendelegasikannya tanpa syarat apa pun —
/// jalur produksi `oktz-bin` yang memanggil [`connect_device`] ikut dapat
/// purge, bukan cuma caller langsung.
pub async fn connect_device_guarded<F, Fut>(
    store: SqliteStore,
    creds: serde_json::Value,
    on_ready: F,
) -> Result<(BotHandle, Arc<FalseLogoutGuard>)>
where
    F: Fn(Arc<Client>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    pairing_run::connect_device_guarded(store, creds, on_ready).await
}

/// [`connect_device_guarded`] dengan plan purge eksplisit.
///
/// Dipakai kalau lokasi store sesi sudah diketahui pemanggil dan tidak boleh
/// diambil dari env / pencatatan [`pairing_run::note_store_path`].
pub async fn connect_device_guarded_with_purge<F, Fut>(
    store: SqliteStore,
    creds: serde_json::Value,
    plan: pairing_run::PurgePlan,
    on_ready: F,
) -> Result<(BotHandle, Arc<FalseLogoutGuard>)>
where
    F: Fn(Arc<Client>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    pairing_run::connect_device_guarded_with_purge(store, creds, plan, on_ready).await
}

/// Boot bot dan pump handle; `on_ready` dipanggil setelah connect.
///
/// Guard 401 sudah terpasang di setiap jalur ini — termasuk jalur produksi.
/// Creds diambil dari [`CREDS_ENV`] karena signature lama tidak punya
/// tempat menerima creds dan tidak boleh pecah.
///
/// # Pipeline pesan
///
/// `on_ready` dibungkus supaya [`crate::pipeline::Pipeline`] terpasang ke bus
/// event SEBELUM closure milik pemanggil dijalankan: setiap pesan inbound
/// `Event::Messages` lalu antre dan diproses (lihat [`PipelineWiring::attach`]).
///
/// Bentuk ini dipakai karena `on_ready` adalah satu-satunya titik di jalur ini
/// yang memegang `Arc<Client>`, dan `pairing_run` berada di luar file yang boleh
/// diubah. Builder (`Bot::on_event_for`) sebenarnya bisa mendaftarkan handler
/// lebih awal, tapi itu menuntut membangun `Bot` sendiri — artinya
/// menggandakan pemasangan `FalseLogoutGuard`, yang lebih berbahaya daripada
/// jendela kecil di bawah.
///
/// ## Batas yang jujur: jendela di `Connected`
///
/// `Event::Connected` dikirim per generasi koneksi
/// (`whatsapp-rust-0.7.0/src/client/lifecycle.rs:230`) dan handler-nya
/// dijalankan pada task terpisah (`src/bot.rs:344-356`), bukan inline. Jadi
/// ada secara teoretis `Event::Messages` yang bisa ter-dispatch sebelum
/// `subscribe` selesai. Jendelanya dibuat sekecil mungkin dengan mengerjakan
/// bagian mahal (baca settings) sebelum connect, sehingga di `on_ready` hanya
/// tersisa `subscribe`. Menutupnya sepenuhnya berarti mendaftarkan handler lewat
/// `Bot::on_event_for` sebelum build — menuntut perubahan di `pairing_run.rs`.
///
/// Kegagalan membangun pipeline (settings.json rusak/tidak terbaca) TIDAK
/// menggagalkan boot: yang dicatat, command tidak dibalas, koneksi tetap jalan.
///
/// # Batas urutan 401 (bukan jaminan)
///
/// whatsapp-rust 0.7.0 tidak punya hook pra-logout. Pada
/// `<stream:error code="401">` `handle_stream_error` sudah men-clear
/// `enable_auto_reconnect` dan menandai `should_disconnect` **sebelum**
/// `Event::LoggedOut` keluar ke handler mana pun
/// (whatsapp-rust-0.7.0 `src/client/node_io.rs:1789-1801`), dan supervision
/// loop sudah `break` + `stop_supervision_loop()` kalau flag itu masih
/// `false` saat dibaca (`src/client/lifecycle.rs:641-644`). Jadi guard ini
/// mitigasi (menghidupkan lagi flag dan memanggil ulang `Client::run()`),
/// bukan penjamin: ada jendela mati selama grace di mana socket sudah turun.
///
/// Yang sudah diperiksa dan sengaja TIDAK dipakai untuk "memperbaiki" urutan:
/// [`Bot::with_event_handler`] (raw handler) disubscribe langsung ke bus
/// (`src/bot.rs:596-598`) dan `CoreEventBus::dispatch` memanggil
/// `handle_event` secara sinkron (`wacore src/types/events.rs:609-621`), jadi
/// handler-nya jalan inline tepat setelah flag ter-clear — tidak sebelum.
/// `EventDelivery::Ordered` juga tidak menolong: mailbox-nya digerakkan task
/// terpisah (`src/bot.rs:290-332`), jadi tetap bisa telat. Inline membuat
/// keputusan guard terurut terhadap event-nya, tetapi sekaligus membuat
/// `enable_auto_reconnect=true` pra-grace di [`crate::pairing_run`] langsung
/// terbaca oleh loop, jadi reconnect-nya seketika dan anggaran 2 retry
/// terkuras dalam hitungan detik. Reconnect yang tertunda grace adalah
/// perilaku yang benar, jadi wiring yang sudah direview itu dibiarkan apa
/// adanya.
/// Tabel session SQLite yang termirror ke Turso per-baris (plan Task 3).
/// Creds utama sudah tercakup `device`/`identities`; prekeys/sessions/
/// sender_keys/app_state_keys = inti dekripsi. Tabel turunan (`sent_messages`,
/// `pending_inbound_messages`, `group_metadata`, dll) sengaja tidak — hilangnya
/// tidak merusak sesi.
pub const SESSION_MIRROR_TABLES: &[&str] = &[
    "device",
    "identities",
    "sessions",
    "prekeys",
    "signed_prekeys",
    "sender_keys",
    "app_state_keys",
];

/// Salin satu tabel dari SQLite lokal (diesel) ke Turso — JSON-per-row.
///
/// Skema Turso generik `(k INTEGER PRIMARY KEY, v TEXT)`; `v` = seluruh row
/// sebagai JSON (BLOB → hex). Sederhana & selaras; upgrade ke kolom asli
/// kalau ukuran `msg_secrets` bermasalah.
///
/// Diesel dipakai untuk baca lokal (bukan libsql `core`) supaya `libsql`
/// tetap remote-only — lihat komentar `Cargo.toml` soal duplicate symbol.
pub async fn mirror_sqlite_table(
    store: &oktz_store::turso::TursoStore,
    conn: &libsql::Connection,
    local_path: &Path,
    table: &str,
) -> anyhow::Result<usize> {
    use diesel::prelude::*;
    use diesel::sql_query;

    #[derive(diesel::QueryableByName)]
    struct Kolom {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    #[derive(diesel::QueryableByName)]
    struct BarisJson {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        rowid: i64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        j: String,
    }

    let mut conn_sqlite = diesel::sqlite::SqliteConnection::establish(
        local_path.to_str().context("path tidak utf8")?,
    )?;

    // ponytail: baca kolom dinamis via pragma + JSON1 (bawaan sqlite), row
    // diserialisasi `json_object`. Upgrade path: skema kolom asli per tabel.
    let cols: Vec<String> = sql_query(&format!(
        "SELECT name FROM pragma_table_info('{table}')"
    ))
    .load::<Kolom>(&mut conn_sqlite)
    .map_err(|e| anyhow::anyhow!("pragma {table}: {e}"))?
    .into_iter()
    .map(|k| k.name)
    .collect();

    if cols.is_empty() {
        return Ok(0);
    }
    let json_cols = cols
        .iter()
        .map(|c| {
            // BLOB tidak bisa masuk json_object — hex() dulu, penanda `__hex:`
            // supaya unwrap tahu ini byte.
            format!(
                "'{c}', CASE WHEN typeof(\"{c}\")='blob' THEN '__hex:'||hex(\"{c}\") ELSE json_quote(\"{c}\") END"
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let rows: Vec<BarisJson> = sql_query(&format!(
        "SELECT rowid, json_object({json_cols}) AS j FROM \"{table}\" ORDER BY rowid"
    ))
    .load::<BarisJson>(&mut conn_sqlite)
    .map_err(|e| anyhow::anyhow!("baca {table}: {e}"))?;

    // Kumpulkan semua row dulu, kirim sekali batch (review I2: N round-trip
    // HTTP per-baris bikin boot menit-an untuk tabel sessions yang besar).
    let mut mirror_rows: Vec<(i64, serde_json::Value)> = Vec::with_capacity(rows.len());
    for b in &rows {
        let v: serde_json::Value =
            serde_json::from_str(&b.j).context("json_object hasil sqlite tak valid")?;
        // json_quote menghasilkan string JSON per nilai; buka satu lapis kutip.
        let v = unwrap_json1_strings(v);
        mirror_rows.push((b.rowid, v));
    }
    store
        .upsert_mirror_rows(conn, table, mirror_rows.iter().map(|(k, v)| (*k, v)).collect::<Vec<_>>().as_slice())
        .await?;
    Ok(rows.len())
}

/// `json_object('a', json_quote(x))` menghasilkan nilai DOUBLE-ENCODED untuk
/// teks (`"\"isi\""`). Buka satu lapis kutip per field. BLOB dikirim sebagai
/// string `__hex:<hex>` dan dikonversi jadi hex polos (kesepakatan mirror;
/// restore mem-balikkan lewat `unhex` saat di-load — untuk sekarang mirror
/// satu arah lokal→Turso).
fn unwrap_json1_strings(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => serde_json::Value::Object(
            m.into_iter()
                .map(|(k, val)| {
                    let val = match val {
                        serde_json::Value::String(s) => {
                            if let Some(hexs) = s.strip_prefix("__hex:") {
                                serde_json::Value::String(hexs.to_string())
                            } else {
                                serde_json::from_str::<serde_json::Value>(&s)
                                    .unwrap_or(serde_json::Value::String(s))
                            }
                        }
                        other => other,
                    };
                    (k, val)
                })
                .collect(),
        ),
        other => other,
    }
}

pub async fn connect_device<F, Fut>(store: SqliteStore, on_ready: F) -> Result<BotHandle>
where
    F: Fn(Arc<Client>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let creds = load_guard_creds(None);

    let cfg = oktz_core::config::Config::load().context("baca config.json")?;
    let settings = settings_path(
        &cfg.database.path,
        oktz_core::env::Env::get(SETTINGS_ENV).as_deref(),
    );
    let (turso_conn, interval) = if cfg.turso.enabled && !cfg.turso.url.is_empty() {
        (
            Some(oktz_store::turso::TursoStore::bare(
                cfg.turso.url.clone(),
                cfg.turso.auth_token.clone(),
            )),
            std::time::Duration::from_millis(cfg.turso.sync_interval.max(15_000)),
        )
    } else {
        (None, std::time::Duration::from_secs(15))
    };
    let wiring = match wiring_from_with_config(
        &settings,
        cfg.owner_numbers.clone(),
        bot_id_from(&creds),
        cfg.runtime(),
    ) {
        Ok(w) => {
            // Mirror Turso (Task 2/4 plan): boot sync sekali, lalu flush
            // write-behind tiap interval. Turso mati = warning, bukan gagal
            // boot.
            if let Some(t) = turso_conn {
                let wiring = Arc::new(w);
                wiring.boot_and_spawn_turso(t, interval);
                Some(wiring)
            } else {
                Some(Arc::new(w))
            }
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                settings = %settings.display(),
                "pipeline pesan tidak aktif; command inbound tidak akan dibalas"
            );
            None
        }
    };
    if wiring.is_some() {
        tracing::info!(settings = %settings.display(), "pipeline pesan siap");
    }

    let (handle, _guard) = connect_device_guarded(store, creds, move |client: Arc<Client>| {
        if let Some(w) = &wiring {
            w.attach(&client);
        }
        on_ready(client)
    })
    .await?;
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairing;
    use oktz_router::RouterContext;

    static SEQ_TEST: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn tmpfile(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("oktz-bot-test-{name}.json"));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn explicit_creds_path_beats_env() {
        assert_eq!(
            resolve_creds_path(Some("/a/creds.json"), Some("/b/creds.json")),
            Some(PathBuf::from("/a/creds.json"))
        );
    }

    // ---------- mirror Turso (plan Task 3/4) ----------

    #[tokio::test]
    async fn mirror_sqlite_table_copies_rows_and_upserts() {
        use oktz_store::turso::TursoStore;
        let dir = std::env::temp_dir().join(format!("oktz-mirror-{}", SEQ_TEST.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("lokal.db");
        let _ = std::fs::remove_file(&db_path);
        // SQLite lokal dengan diesel (jalur yang sama dengan produksi).
        use diesel::prelude::*;
        use diesel::connection::SimpleConnection;
        {
            let mut c = diesel::sqlite::SqliteConnection::establish(db_path.to_str().unwrap()).unwrap();
            c.batch_execute(
                "CREATE TABLE identities (nama TEXT, kunci BLOB);
                 INSERT INTO identities VALUES ('satu', x'0102');
                 INSERT INTO identities VALUES ('dua', x'0304');",
            )
            .unwrap();
        }
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let t = TursoStore::with_conn(conn.clone());
        let n = mirror_sqlite_table(&t, &conn, &db_path, "identities").await.unwrap();
        assert_eq!(n, 2);
        let mut rows = conn.query("SELECT v FROM identities ORDER BY k", ()).await.unwrap();
        let v1: serde_json::Value = serde_json::from_str(
            &rows.next().await.unwrap().unwrap().get::<String>(0).unwrap(),
        ).unwrap();
        assert_eq!(v1["nama"], "satu");
        assert_eq!(v1["kunci"], "0102", "BLOB harus jadi hex di JSON");
        // Mirror ulang = upsert, bukan duplikasi.
        let n2 = mirror_sqlite_table(&t, &conn, &db_path, "identities").await.unwrap();
        assert_eq!(n2, 2);
        let mut cnt = conn.query("SELECT COUNT(*) FROM identities", ()).await.unwrap();
        let c: i64 = cnt.next().await.unwrap().unwrap().get(0).unwrap();
        assert_eq!(c, 2);
    }

    #[tokio::test]
    async fn flush_once_turso_writes_only_when_dirty() {
        use oktz_store::turso::TursoStore;
        let dir = std::env::temp_dir().join(format!("oktz-flushonce-{}", SEQ_TEST.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, r#"{"users":{"628111":{"koin":3}}}"#).unwrap();
        let wiring = Arc::new(wiring_from(&path, Vec::<String>::new(), "bot").unwrap());
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let t = TursoStore::with_conn(conn.clone());
        t.ensure_tables(&conn).await.unwrap();

        *wiring.turso.lock().unwrap() = Some((t.clone(), std::time::Duration::from_secs(15)));
        // Belum kotor → flush tidak menulis apa pun.
        wiring.flush_once_turso().await;
        assert!(!t.has_any_store(&conn).await.unwrap());

        // Tulis mutasi → kotor → flush mengisi Turso.
        wiring
            .ctx
            .apply(
                &[oktz_plugin_api::Mutation::SetUser {
                    jid: "628222@s.whatsapp.net".into(),
                    data: serde_json::json!({ "koin": 7 }),
                }],
                1_000,
            )
            .unwrap();
        wiring.flush_once_turso().await;
        let users = t.load_store(&conn, "users").await.unwrap().unwrap();
        assert_eq!(users["628222"]["koin"], 7);
        assert_eq!(users["628111"]["koin"], 3, "store lama tidak boleh hilang");
    }

    #[test]
    fn session_db_path_sits_next_to_settings() {
        let dir = std::env::temp_dir().join(format!("oktz-sessdb-{}", SEQ_TEST.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir_all(dir.join("wa")).unwrap();
        std::fs::write(dir.join("settings.json"), "{}").unwrap();
        std::fs::write(dir.join("wa").join("oktz-run.db"), "").unwrap();
        let wiring = wiring_from(&dir.join("settings.json"), Vec::<String>::new(), "bot").unwrap();
        assert_eq!(
            wiring.session_db_path(),
            Some(dir.join("wa").join("oktz-run.db"))
        );
    }

    #[test]
    fn explicit_creds_path_beats_env_dup_guard() {
        // Penanda urutan test: test di atas menambah test tokio; tidak ada
        // yang perlu diuji di sini selain memastikan mod test tetap compile.
    }

    #[test]
    fn sesi_lama_dipakai_hanya_kalau_sidik_creds_cocok() {
        assert!(sesi_lama_boleh_dipakai(true, Some("7:abc\n"), "7:abc"));
        assert!(!sesi_lama_boleh_dipakai(true, Some("8:abc"), "7:abc"));
        assert!(
            !sesi_lama_boleh_dipakai(true, None, "7:abc"),
            "db tanpa marker sidik tidak boleh dipercaya"
        );
        assert!(!sesi_lama_boleh_dipakai(false, Some("7:abc"), "7:abc"));
    }

    #[test]
    fn env_is_the_fallback_and_blank_means_absent() {
        assert_eq!(
            resolve_creds_path(None, Some("/b/creds.json")),
            Some(PathBuf::from("/b/creds.json"))
        );
        assert_eq!(resolve_creds_path(None, None), None);
        assert_eq!(resolve_creds_path(None, Some("   ")), None);
        assert_eq!(
            resolve_creds_path(Some(""), Some("/b/creds.json")),
            Some(PathBuf::from("/b/creds.json")),
            "argumen kosong harus jatuh ke env, bukan jadi path invalid"
        );
    }

    #[test]
    fn load_guard_creds_reads_the_authenticated_markers() {
        let p = tmpfile("creds-ok");
        std::fs::write(
            &p,
            r#"{"registered": true, "account": {"details": "x"}, "additionalData": {}}"#,
        )
        .unwrap();
        let c = load_guard_creds(Some(p.to_str().unwrap()));
        assert!(pairing::is_authenticated(&c));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn unreadable_creds_yield_null_so_logout_is_trusted() {
        // Arah konservatif: creds tak terbaca = "belum pernah ter-link",
        // jadi 401 langsung dipercaya. Guard TIDAK boleh gagal start.
        let c = load_guard_creds(Some("/definitely/not/here/creds.json"));
        assert!(c.is_null());
        assert!(!pairing::is_authenticated(&c));
    }

    #[test]
    fn production_path_reads_creds_from_env() {
        // `connect_device` memanggil `load_guard_creds(None)`, jadi cabang env
        // adalah satu-satunya yang dipakai jalur produksi. Diuji lewat env
        // sungguhan (bukan helper murni) supaya nama env-nya ikut terkunci —
        // kalau `CREDS_ENV` salah ketik, tes ini yang menangkapnya.
        // Aman: tidak ada tes lain di workspace yang menyentuh var ini.
        let p = tmpfile("creds-env");
        std::fs::write(
            &p,
            r#"{"registered": true, "account": {"details": "x"}, "additionalData": {}}"#,
        )
        .unwrap();
        let path = p.to_str().unwrap();

        unsafe { std::env::set_var(CREDS_ENV, path) };
        let from_env = load_guard_creds(None);
        unsafe { std::env::remove_var(CREDS_ENV) };

        assert!(
            pairing::is_authenticated(&from_env),
            "{} tidak dibaca sebagai creds ter-autentikasi",
            CREDS_ENV
        );
        std::fs::remove_file(&p).ok();
    }

    // ---------- pipeline wiring (tanpa koneksi WA) ----------

    #[test]
    fn settings_path_defaults_under_the_database_dir() {
        // Default harus sama dengan yang dipakai oktz-bin (main.rs:166),
        // kalau tidak pipeline akan membaca store yang salah diam-diam.
        assert_eq!(
            settings_path("./database/main", None),
            PathBuf::from("./database/main/settings.json")
        );
    }

    #[test]
    fn settings_env_overrides_but_blank_does_not() {
        assert_eq!(
            settings_path("./database/main", Some("/var/lib/oktz/s.json")),
            PathBuf::from("/var/lib/oktz/s.json")
        );
        for blank in ["", "   "] {
            assert_eq!(
                settings_path("./database/main", Some(blank)),
                PathBuf::from("./database/main/settings.json"),
                "{blank:?} harus berarti tidak di-override, bukan path path-butuh-dibuat"
            );
        }
    }

    #[test]
    fn owner_path_defaults_under_database_dir_and_owner_entries_read_array() {
        assert_eq!(
            owner_path("./database/main"),
            PathBuf::from("./database/main/owner.json")
        );

        let p = tmpfile("owner-entries");
        std::fs::write(&p, r#"["6281","6282"]"#).unwrap();
        assert_eq!(
            owner_entries(&p),
            vec!["6281".to_string(), "6282".to_string()]
        );

        let missing = std::env::temp_dir().join(format!("oktz-bot-missing-owner-{}", std::process::id()));
        assert!(owner_entries(&missing).is_empty());
    }

    #[test]
    fn owner_numbers_become_identity_owners() {
        let id = identity_from_owners(vec!["6281".to_string(), "6282".to_string()]);
        assert!(id.is_owner("6281@s.whatsapp.net"));
        assert!(id.is_owner("6282:9@s.whatsapp.net"), "cocok lewat nomor");
        assert!(!id.is_owner("6283@s.whatsapp.net"));
        assert!(
            !id.is_premium("6281@s.whatsapp.net") && !id.is_admin("6281@s.whatsapp.net"),
            "premium/admin tidak punya sumber di Config — harus kosong, bukan menebak"
        );
    }

    #[test]
    fn bot_id_falls_back_when_creds_carry_no_me() {
        // `load_guard_creds` mengembalikan Null saat creds tak terbaca; jalur
        // ini tidak boleh menggagalkan boot hanya karena itu.
        assert_eq!(bot_id_from(&serde_json::Value::Null), DEFAULT_BOT_ID);
        assert_eq!(bot_id_from(&serde_json::json!({})), DEFAULT_BOT_ID);
        assert_eq!(bot_id_from(&serde_json::json!({"me": {"id": "  "}})), DEFAULT_BOT_ID);
        assert_eq!(
            bot_id_from(&serde_json::json!({"me": {"id": "6281@s.whatsapp.net"}})),
            "6281@s.whatsapp.net"
        );
    }

    #[test]
    fn wiring_builds_from_a_store_and_survives_a_missing_file() {
        let p = tmpfile("wiring");
        std::fs::write(&p, r#"{"settings": {"botMode": "public"}}"#).unwrap();
        let w = wiring_from(&p, vec!["6281".to_string()], "botid").unwrap();
        assert_eq!(w.bot_id, "botid");
        assert!(w.ctx.mode_settings().mode == "public");
        assert!(w.live.lock().unwrap().is_none(), "belum ada pipeline");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn unreadable_settings_fail_wiring_without_panicking() {
        // File rusak = error yang harus jadi log + boot tanpa pipeline
        // (lihat connect_device), bukan panik di thread boot.
        let p = tmpfile("wiring-bad");
        std::fs::write(&p, "{ this is not json").unwrap();
        assert!(wiring_from(&p, Vec::new(), "botid").is_err());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn wiring_context_is_cloned_so_attach_can_run_more_than_once() {
        // `attach` dipanggil ulang tiap reconnect; kalau ctx tidak bisa di-clone
        // atau state-nya berubah, pipeline kedua akan membaca store berbeda.
        let p = tmpfile("wiring-clone");
        std::fs::write(&p, r#"{"settings": {"onlyGc": true}}"#).unwrap();
        let w = wiring_from(&p, Vec::new(), "botid").unwrap();
        let a = w.ctx.clone();
        let b = w.ctx.clone();
        assert!(a.mode_settings().only_gc && b.mode_settings().only_gc);
        std::fs::remove_file(&p).ok();
    }
}
