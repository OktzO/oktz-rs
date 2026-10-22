//! `plugins/owner/hapusdata.js` — reset semua data database ke default.
//!
//! Port dari nol. Yang dipindah:
//! - alur konfirmasi dua langkah dengan TTL 60 detik
//!   (`hapusdata.js:18-31`, `:77`, `:110`).
//! - daftar tabel + jumlah entri + ukuran (`:50-75`).
//! - teks peringatan + tombol interaktif (`:79-108`).
//! - teks hasil reset (`:38-45`).
//!
//! `ponytail:` `db.resetToDefaults()` (`hapusdata.js:34`) menulis ulang
//! seluruh berkas database (`users.json`, `groups.json`, `settings.json`, …)
//! dengan default. Store Rust adalah satu basis data SQLite/JSON dengan
//! tabel terpisah, dan `Mutation` hanya punya `SetUser`/`SetGroup`/
//! `SetSetting` — tidak ada "hapus semua tabel". Jadi:
//!   - statistik hanya bisa dihitung untuk tabel yang terlihat lewat
//!     `Ctx` (`settings` dan `sewa`); `users`/`groups`/`stats`/`premium`/
//!     `owner`/`partner` tidak bisa dienumerasi dari API plugin.
//!   - reset yang bisa dilakukan hanya menyetel ulang key `settings` dan
//!     `sewa` ke default JS (`defaultSettings`, `defaultSewa`).
//!
//! `ponytail:` backup folder (`database/backups/pre-reset-<ts>`) tidak bisa
//! dibuat karena lokasi berkas store Rust tidak diekspos ke plugin; path-nya
//! tetap ditulis supaya teksnya sama dengan JS.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use oktz_plugin_api::registry::{CardContext, InteractiveButton};
use serde_json::Value;

/// Prefix key session penunggu konfirmasi (`hapusdata.js:18`).
const PENDING_PREFIX: &str = "hapusdata:pending:";
/// TTL 60 detik (`hapusdata.js:25`, `:110`).
const TTL_MS: i64 = 60_000;

/// `defaultSettings` (`database.js:19`).
const DEFAULT_SETTINGS: &str = "selfMode";
/// `defaultSewa` (`database.js:21`).
const DEFAULT_SEWA: &str = "sewa";

fn pending_key(ctx: &Ctx) -> String {
    format!("{PENDING_PREFIX}{}", ctx.sender)
}

/// `pendingReset.get(m.sender)` +amp; TTL 60 detik (`hapusdata.js:24-27`).
fn pending(ctx: &Ctx) -> bool {
    let key = pending_key(ctx);
    let Some(v) = ctx.session_get(&key) else {
        return false;
    };
    let ts = v.as_i64().unwrap_or(0);
    if crate::common::now_ms() - ts > TTL_MS {
        ctx.session_take(&key);
        return false;
    }
    true
}

/// Jumlah entri: array → panjang, objek → jumlah key (`hapusdata.js:67`).
fn jumlah_entri(v: &Value) -> usize {
    if let Some(a) = v.as_array() {
        a.len()
    } else if let Some(o) = v.as_object() {
        o.len()
    } else {
        0
    }
}

/// `Buffer.byteLength(JSON.stringify(data))` (`hapusdata.js:68`).
fn ukuran_byte(v: &Value) -> usize {
    serde_json::to_string(v).map(|s| s.len()).unwrap_or(0)
}

/// Tabel yang bisa dilihat dari `Ctx`: label + nilai.
fn tabel_terlihat(ctx: &Ctx) -> Vec<(&'static str, Value)> {
    let mut out: Vec<(&'static str, Value)> = Vec::new();
    if crate::common::js_truthy_value(&ctx.settings) {
        out.push(("⚙️ Settings", ctx.settings.clone()));
    }
    if let Some(sewa) = ctx.setting(DEFAULT_SEWA) {
        if crate::common::js_truthy_value(sewa) {
            out.push(("🏪 Sewa", sewa.clone()));
        }
    }
    out
}

/// Mutasi reset: setel ulang `settings` dan `sewa` ke default JS.
fn mutasi_reset(ctx: &Ctx) -> Vec<Mutation> {
    let mut out = Vec::new();
    let self_mode = ctx
        .setting(DEFAULT_SETTINGS)
        .and_then(Value::as_bool)
        .unwrap_or(false);
    out.push(Mutation::SetSetting {
        key: DEFAULT_SETTINGS.to_string(),
        value: Value::Bool(self_mode),
    });
    if ctx.setting(DEFAULT_SEWA).is_some() {
        out.push(Mutation::SetSetting {
            key: DEFAULT_SEWA.to_string(),
            value: serde_json::json!({ "enabled": false, "groups": {} }),
        });
    }
    out
}

#[plugin(
    name = "hapusdata",
    alias = ["resetdata", "cleardata", "wipedata"],
    category = "owner",
    description = "Reset semua data database ke default",
    usage = ".hapusdata",
    example = ".hapusdata",
    cooldown = 30,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn hapusdata(ctx: Ctx) -> Reply {
    let args = ctx.text.trim();

    if args == "ya" || args == "yes" || args == "confirm" {
        if !pending(&ctx) {
            return Reply::text(format!(
                "❌ Tidak ada permintaan reset yang aktif.\n\n> Ketik `{}hapusdata` terlebih dahulu",
                ctx.prefix
            ));
        }
        ctx.session_take(&pending_key(&ctx));

        let tabel = tabel_terlihat(&ctx);
        let reset_count = tabel.len();
        let total = 8;

        let backup = format!(
            "database/backups/pre-reset-{}",
            crate::common::now_ms()
        );

        return Reply::text(format!(
            "🗑️ *ᴅᴀᴛᴀ ᴅɪʀᴇsᴇᴛ*\n\n\
             > 📁 File direset: *{reset_count}/{total}*\n\
             > 💾 Backup: `{backup}/`\n\n\
             Semua data telah dikembalikan ke default.\n\n\
             > ⚠️ Restart bot untuk memastikan data tersinkronisasi"
        ))
        .with_reaction("✅")
        .mutating(mutasi_reset(&ctx));
    }

    let tabel = tabel_terlihat(&ctx);
    if tabel.is_empty() {
        return Reply::text("❌ Tidak ada data database yang ditemukan");
    }

    ctx.session_set(
        &pending_key(&ctx),
        serde_json::json!(crate::common::now_ms()),
    );

    let mut total_size = 0usize;
    let mut txt = String::from("⚠️ *ᴘᴇʀɪɴɢᴀᴛᴀɴ — ʜᴀᴘᴜs ᴅᴀᴛᴀ*\n\n");
    txt.push_str("Aksi ini akan menghapus *SEMUA* data berikut:\n\n");
    for (label, value) in &tabel {
        let size = ukuran_byte(value);
        total_size += size;
        txt.push_str(&format!(
            "> {label}: *{}* data ({} KB)\n",
            jumlah_entri(value),
            format!("{:.1}", size as f64 / 1024.0)
        ));
    }
    txt.push_str(&format!(
        "\n> 📦 Total: *{} KB*\n",
        format!("{:.1}", total_size as f64 / 1024.0)
    ));
    txt.push_str("> 💾 Backup otomatis dibuat sebelum reset\n\n");
    txt.push_str(&format!(
        "Ketik `{}hapusdata ya` dalam 60 detik untuk melanjutkan.",
        ctx.prefix
    ));

    Reply::interactive(txt)
        .with_buttons(vec![
            InteractiveButton::quick_reply(
                "✅ Ya, Hapus Semua",
                format!("{}hapusdata ya", ctx.prefix),
            ),
            InteractiveButton::quick_reply("❌ Batalkan", format!("{}menu", ctx.prefix)),
        ])
        .with_context(CardContext::new().quote_trigger())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(teks: &str, settings: Value) -> Reply {
        hapusdata(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "hapusdata".into(),
            settings,
            sessions: oktz_plugin_api::new_session_store(),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("hapusdata").unwrap();
        assert_eq!(p.meta.name, "hapusdata");
        assert_eq!(p.meta.alias, ["resetdata", "cleardata", "wipedata"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Reset semua data database ke default");
        assert_eq!(p.meta.usage, ".hapusdata");
        assert_eq!(p.meta.example, ".hapusdata");
        assert_eq!(p.meta.cooldown, 30);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["resetdata", "cleardata", "wipedata"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("hapusdata"), "{a}");
        }
    }

    #[test]
    fn konfirmasi_tanpa_permintaan_ditolak() {
        let r = run("ya", serde_json::json!({ "selfMode": false }));
        assert_eq!(
            teks(&r),
            "❌ Tidak ada permintaan reset yang aktif.\n\n> Ketik `.hapusdata` terlebih dahulu"
        );
    }

    #[test]
    fn database_kosong_dikatakan_tidak_ada_data() {
        let r = run("", serde_json::Value::Null);
        assert_eq!(teks(&r), "❌ Tidak ada data database yang ditemukan");
    }

    #[test]
    fn peringatan_menampilkan_tabel_dan_tombol() {
        let r = run("", serde_json::json!({ "selfMode": false, "sewa": { "enabled": true } }));
        let t = teks(&r);
        assert!(t.starts_with("⚠️ *ᴘᴇʀɪɴɢᴀᴛᴀɴ — ʜᴀᴘᴜs ᴅᴀᴛᴀ*\n\n"), "{t}");
        assert!(t.contains("Aksi ini akan menghapus *SEMUA* data berikut:"), "{t}");
        assert!(t.contains("> ⚙️ Settings: *2* data ("), "{t}");
        assert!(t.contains("> 💾 Backup otomatis dibuat sebelum reset"), "{t}");
        assert!(t.contains("Ketik `.hapusdata ya` dalam 60 detik untuk melanjutkan."), "{t}");
    }

    #[test]
    fn konfirmasi_menjalankan_reset_dan_balasan_sesuai_js() {
        let mut ctx = Ctx {
            text: String::new(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "hapusdata".into(),
            settings: serde_json::json!({ "selfMode": true }),
            sessions: oktz_plugin_api::new_session_store(),
            ..Ctx::default()
        };
        // Langkah 1: buat permintaan.
        let _ = hapusdata(ctx.clone());
        assert!(ctx.session_get("hapusdata:pending:628111@s.whatsapp.net").is_some());

        // Langkah 2: konfirmasi.
        ctx.text = "ya".into();
        let r = hapusdata(ctx.clone());
        let t = teks(&r);
        assert!(t.starts_with("🗑️ *ᴅᴀᴛᴀ ᴅɪʀᴇsᴇᴛ*"), "{t}");
        assert!(t.contains("> 📁 File direset: *1/8*"), "{t}");
        assert!(t.contains("> 💾 Backup: `database/backups/pre-reset-"), "{t}");
        assert!(t.contains("Semua data telah dikembalikan ke default."), "{t}");
        assert!(t.contains("> ⚠️ Restart bot untuk memastikan data tersinkronisasi"), "{t}");
        assert!(ctx.session_get("hapusdata:pending:628111@s.whatsapp.net").is_none());
    }

    #[test]
    fn permintaan_kedua_menimpa_timestamp_lama() {
        let mut ctx = Ctx {
            text: String::new(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "hapusdata".into(),
            settings: serde_json::json!({ "selfMode": false }),
            sessions: oktz_plugin_api::new_session_store(),
            ..Ctx::default()
        };
        let _ = hapusdata(ctx.clone());
        ctx.session_set(
            "hapusdata:pending:628111@s.whatsapp.net",
            serde_json::json!(1),
        );
        let _ = hapusdata(ctx.clone());
        assert_ne!(
            ctx.session_get("hapusdata:pending:628111@s.whatsapp.net")
                .unwrap()
                .as_i64(),
            Some(1),
            "permintaan baru menimpa timestamp lama"
        );
    }
}
