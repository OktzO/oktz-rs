//! Plugin `.clearsessiondb` — bersihkan session basi saat membeludak:

use oktz_plugin_api::{Ctx, Reply, plugin};

fn help(prefix: &str) -> String {
    format!(
        "╭┈┈⬡「 🧹 *ᴄʟᴇᴀʀ sᴇssɪᴏɴ* 」\n┃\n┃ Hapus session basi: karantina lokal,\n┃ prune tabel kedaluwarsa, backup/legacy\n┃ Turso. Prekey & session aktif TIDAK disentuh.\n┃\n┃ Contoh: `{prefix}clearsessiondb`\n╰┈┈⬡",
        prefix = prefix
    )
}

/// Folder db session mengikuti binary (`src/main.rs::run`): `OKTZ_DB_DIR`
/// atau `database/wa`. Bisa relatif (cwd launcher) atau absolut.
fn dir_db_session() -> String {
    std::env::var("OKTZ_DB_DIR")
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "database/wa".to_string())
}

/// Hapus SEMUA karantina `<base>.broken-*` di `db_dir`. Akun aktif (nama tanpa
/// `.broken-`) tidak cocok prefiks → aman. `Ok(nama yang gagal dihapus)`.
fn bersih_karantina(db_dir: &str, base: &str) -> (usize, Vec<String>) {
    let prefiks = format!("{base}.broken-");
    let Ok(rd) = std::fs::read_dir(db_dir) else {
        return (0, Vec::new());
    };
    let mut n = 0usize;
    let mut gagal = Vec::new();
    for e in rd.flatten() {
        let nama = e.file_name().to_string_lossy().to_string();
        if !nama.starts_with(&prefiks) {
            continue;
        }
        let p = std::path::Path::new(db_dir).join(&nama);
        let ok = if p.is_dir() {
            std::fs::remove_dir_all(&p).is_ok()
        } else {
            std::fs::remove_file(&p).is_ok()
        };
        if ok {
            n += 1;
        } else {
            gagal.push(nama);
        }
    }
    (n, gagal)
}

/// Prune SQLite lokal di thread + runtime sendiri, bukan `block_in_place`.
fn prune_lokal(db_path: &str) -> Result<oktz_store::session_db::PruneReport, String> {
    let now = crate::common::now_ms() / 1000;
    std::thread::scope(|s| {
        s.spawn(
            move || -> Result<oktz_store::session_db::PruneReport, String> {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("runtime: {e}"))?;
                rt.block_on(async move {
                    oktz_store::session_db::prune_expired(db_path, now)
                        .await
                        .map(|r| r.unwrap_or_default())
                        .map_err(|e| e.to_string())
                })
            },
        )
        .join()
    })
    .map_err(|_| "thread prune panic".to_string())?
}

fn bersih_lokal(
    db_dir: &str,
    base: &str,
) -> (
    usize,
    Vec<String>,
    Result<oktz_store::session_db::PruneReport, String>,
) {
    let (n, gagal) = bersih_karantina(db_dir, base);
    let path = std::path::Path::new(db_dir).join(base);
    (n, gagal, prune_lokal(&path.to_string_lossy()))
}

/// Hapus snapshot basi di Turso. `Ok(None)` = Turso tidak dikonfigurasi.
///
/// Turso remote butuh async, plugin sinkron. `block_in_place` TIDAK cocok:
/// runtime `current_thread` (main.rs memakainya untuk 1 core) panic di situ.
/// Jadi jalankan runtime sendiri di thread terpisah — aman di kedua flavor.
fn turso_bersih() -> Result<Option<(usize, bool)>, String> {
    let url = oktz_core::env::Env::get("TURSO_URL");
    let token = oktz_core::env::Env::get("TURSO_AUTH_TOKEN");
    let (Some(url), Some(token)) = (url, token) else {
        return Ok(None);
    };
    let hasil = std::thread::scope(|s| {
        s.spawn(move || -> Result<Option<(usize, bool)>, String> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("runtime: {e}"))?;
            rt.block_on(async move {
                let t = oktz_store::turso::TursoStore::bare(url, token);
                let conn = t.connect().await.map_err(|e| e.to_string())?;
                t.clear_session_backups(&conn)
                    .await
                    .map_err(|e| e.to_string())
                    .map(Some)
            })
        })
        .join()
    });
    hasil.map_err(|_| "thread Turso panic".to_string())?
}

#[plugin(
    name = "clearsessiondb",
    alias = ["clearsesidb", "bersihkansessiondb"],
    category = "owner",
    description = "Bersihkan session basi (karantina lokal + backup/legacy Turso), session aktif utuh",
    usage = ".clearsessiondb",
    example = ".clearsessiondb",
    cooldown = 60,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn clearsessiondb(ctx: Ctx) -> Reply {
    if ctx.args.first().is_some_and(|a| a == "help" || a == "-h") {
        return Reply::text(help(&ctx.prefix));
    }

    let db_dir = dir_db_session();
    let (lokal, gagal, prune) = bersih_lokal(&db_dir, "oktz-run.db");

    let mut laporan = vec![format!("┃ 📁 Karantina lokal dihapus: *{lokal}*")];
    match prune {
        Ok(r) => {
            laporan.push(format!("┃ 🗃️ msg_secrets: *{}*", r.msg_secrets));
            laporan.push(format!("┃ 🗃️ tc_tokens: *{}*", r.tc_tokens));
            laporan.push(format!("┃ 🗃️ sent_messages: *{}*", r.sent_messages));
            laporan.push(format!("┃ 🗃️ pending_inbound: *{}*", r.pending_inbound));
        }
        Err(e) => laporan.push(format!("┃ ⚠️ Prune lokal: {e}")),
    }
    if !gagal.is_empty() {
        laporan.push(format!("┃ ⚠️ Gagal hapus: {}", gagal.join(", ")));
    }
    match turso_bersih() {
        Ok(Some((backup, legacy))) => laporan.push(format!(
            "┃ ☁️ Turso: *{backup}* backup dihapus{}",
            if legacy { " + legacy dibersihkan" } else { "" }
        )),
        Ok(None) => laporan.push("┃ ☁️ Turso: tidak aktif (TURSO_URL kosong)".to_string()),
        Err(e) => laporan.push(format!("┃ ⚠️ Turso: {e}")),
    }

    Reply::text(format!(
        "╭┈┈⬡「 🧹 *sᴇssɪᴏɴ ᴄʟᴇᴀɴᴇᴅ* 」\n┃\n{}\n┃\n┃ ✅ Session aktif utuh — bot jalan terus\n╰┈┈⬡",
        laporan.join("\n")
    ))
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `OKTZ_DB_DIR` process-wide: test yang menyetelnya harus serial.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn ctx_of() -> Ctx {
        Ctx {
            text: String::new(),
            args: Vec::new(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "clearsessiondb".into(),
            is_owner: true,
            ..Ctx::default()
        }
    }

    fn scratch(nama: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("oktz-clear-{}-{nama}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn karantina_dihapus_db_dan_marker_aktif_utuh() {
        let d = scratch("karantina");
        std::fs::write(d.join("oktz-run.db"), b"aktif").unwrap();
        std::fs::write(d.join("oktz-run.ident"), b"s").unwrap();
        std::fs::create_dir_all(d.join("oktz-run.db.broken-1700000000000")).unwrap();
        std::fs::write(d.join("oktz-run.db.broken-1700000000000/oktz-run.db"), b"x").unwrap();
        std::fs::write(d.join("oktz-run.db.broken-1700000000001"), b"x").unwrap();

        let (n, gagal) = bersih_karantina(&d.to_string_lossy(), "oktz-run.db");
        assert_eq!(n, 2, "kedua karantina (dir + file) terhapus");
        assert!(gagal.is_empty(), "{gagal:?}");
        assert!(d.join("oktz-run.db").exists(), "db aktif utuh");
        assert!(d.join("oktz-run.ident").exists(), "marker utuh");
        assert!(!d.join("oktz-run.db.broken-1700000000000").exists());
        assert!(!d.join("oktz-run.db.broken-1700000000001").exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn dir_db_default_relatif_kalau_env_kosong() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::remove_var("OKTZ_DB_DIR") };
        assert_eq!(dir_db_session(), "database/wa");
        unsafe { std::env::set_var("OKTZ_DB_DIR", "  /x/wa/  ") };
        assert_eq!(dir_db_session(), "/x/wa", "trim + buang slash akhir");
        unsafe { std::env::remove_var("OKTZ_DB_DIR") };
    }

    #[test]
    fn balasan_menyebut_karantina_dan_session_utuh() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let d = scratch("balasan");
        std::fs::write(d.join("oktz-run.db.broken-1"), b"x").unwrap();
        unsafe { std::env::set_var("OKTZ_DB_DIR", &d) };
        let r = clearsessiondb(ctx_of());
        unsafe { std::env::remove_var("OKTZ_DB_DIR") };
        let t = r.text_body().unwrap();
        assert!(t.contains("sᴇssɪᴏɴ ᴄʟᴇᴀɴᴇᴅ"));
        assert!(t.contains("Karantina lokal dihapus: *1*"), "{t}");
        assert!(t.contains("Turso: tidak aktif"));
        assert!(t.contains("Session aktif utuh"));
        assert!(!d.join("oktz-run.db.broken-1").exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn prune_lokal_dipanggil_untuk_db_yang_ada() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let d = scratch("prune-lokal");
        std::fs::write(d.join("oktz-run.db"), b"").unwrap();

        let (lokal, gagal, prune) = bersih_lokal(&d.to_string_lossy(), "oktz-run.db");
        assert_eq!(lokal, 0, "tidak ada karantina");
        assert!(gagal.is_empty(), "{gagal:?}");
        let r = prune.expect("db kosong tetap harus diproses");
        assert_eq!(r, oktz_store::session_db::PruneReport::default());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn help_dibalas_tanpa_menyentuh_disk() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let d = scratch("help");
        std::fs::write(d.join("oktz-run.db.broken-1"), b"x").unwrap();
        unsafe { std::env::set_var("OKTZ_DB_DIR", &d) };
        let mut c = ctx_of();
        c.args = vec!["help".into()];
        let r = clearsessiondb(c);
        unsafe { std::env::remove_var("OKTZ_DB_DIR") };
        assert!(r.text_body().unwrap().contains("ᴄʟᴇᴀʀ sᴇssɪᴏɴ"));
        assert!(
            d.join("oktz-run.db.broken-1").exists(),
            "help tidak menghapus"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    /// Runtime `current_thread` (main.rs memakainya untuk 1 core) TIDAK boleh
    /// bikin plugin panic: `block_in_place` panic di flavor itu, jadi Turso
    /// dijalankan lewat runtime sendiri di thread terpisah.
    #[tokio::test(flavor = "current_thread")]
    async fn turso_tidak_terjangkau_di_runtime_current_thread_tidak_panic() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let d = scratch("current-thread");
        unsafe {
            std::env::set_var("OKTZ_DB_DIR", &d);
            std::env::set_var("TURSO_URL", "http://127.0.0.1:1");
            std::env::set_var("TURSO_AUTH_TOKEN", "tok");
        }
        let r = clearsessiondb(ctx_of());
        unsafe {
            std::env::remove_var("OKTZ_DB_DIR");
            std::env::remove_var("TURSO_URL");
            std::env::remove_var("TURSO_AUTH_TOKEN");
        }
        let t = r.text_body().unwrap();
        assert!(t.contains("Session aktif utuh"), "{t}");
        assert!(t.contains("Turso:"), "{t}");
        std::fs::remove_dir_all(&d).ok();
    }
}
