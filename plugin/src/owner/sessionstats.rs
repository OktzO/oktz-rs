//! `plugins/owner/sessionstats.js` — menampilkan ukuran session_keys per
//! kategori.
//!
//! Port dari nol. Yang dipindah:
//! - `fmtSize` (`sessionstats.js:22-27`) — B / KB / MB.
//! - cabang "tidak ada key" (`:40-48`) dengan alasan
//!   "Turso tidak aktif atau belum ada key.".
//!
//! `ponytail:` `sessionKeyStats()` (`src/lib/turso-session.js`) membaca tabel
//! `session_keys` Turso — tabel auth state Baileys. `Ctx` hanya membawa
//! `sessions` (peta in-memory untuk state antarm plugin), bukan auth state,
//! dan tidak ada API Turso di `oktz-plugin-api`. Jadi cabang "ada key" tidak
//! punya padanan dan yang bisa ditampilkan adalah cabang "tidak ada key".

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `fmtSize(bytes)` (`sessionstats.js:22-27`).
pub(crate) fn fmt_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
}

/// Teks "tidak ada key" (`sessionstats.js:41-46`).
fn tanpa_key() -> String {
    "╭┈┈⬡「 📊 *sᴇssɪᴏɴ sᴛᴀᴛs* 」\n┃\n┃ ɴᴏ ᴋᴇʏ ᴛᴇʀsɪᴍᴘᴀɴ\n> _Turso tidak aktif atau belum ada key._\n╰┈┈⬡".to_string()
}

#[plugin(
    name = "sessionstats",
    alias = ["sessionstat", "statkeys"],
    category = "owner",
    description = "Menampilkan ukuran session_keys per kategori",
    usage = ".sessionstats",
    example = ".sessionstats",
    cooldown = 30,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn sessionstats(_ctx: Ctx) -> Reply {
    Reply::text(tanpa_key()).with_reaction("📊")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("sessionstats").unwrap();
        assert_eq!(p.meta.name, "sessionstats");
        assert_eq!(p.meta.alias, ["sessionstat", "statkeys"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Menampilkan ukuran session_keys per kategori"
        );
        assert_eq!(p.meta.usage, ".sessionstats");
        assert_eq!(p.meta.example, ".sessionstats");
        assert_eq!(p.meta.cooldown, 30);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["sessionstat", "statkeys"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("sessionstats"), "{a}");
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        let r = sessionstats(Ctx {
            command: "sessionstats".into(),
            ..Ctx::default()
        });
        assert_eq!(
            r.without_mutations().text_body(),
            Some("╭┈┈⬡「 📊 *sᴇssɪᴏɴ sᴛᴀᴛs* 」\n┃\n┃ ɴᴏ ᴋᴇʏ ᴛᴇʀsɪᴍᴘᴀɴ\n> _Turso tidak aktif atau belum ada key._\n╰┈┈⬡")
        );
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "📊"),
            other => panic!("harus Reaction, dapat {other:?}"),
        }
    }

    #[test]
    fn fmt_size_sama_dengan_js() {
        assert_eq!(fmt_size(0), "0 B");
        assert_eq!(fmt_size(512), "512 B");
        assert_eq!(fmt_size(1024), "1.0 KB");
        assert_eq!(fmt_size(1536), "1.5 KB");
        assert_eq!(fmt_size(1024 * 1024), "1.00 MB");
        assert_eq!(fmt_size(1024 * 1024 * 3), "3.00 MB");
    }
}
