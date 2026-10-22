//! `plugins/owner/sistemdaftar.js` — kelola sistem wajib daftar dan statistik
//! pendaftaran.
//!
//! Port dari nol. Yang dipindah:
//! - pembacaan status (`db.setting("registrationRequired") ??
//!   config.registration?.enabled ?? false`, `sistemdaftar.js:68-69`).
//! - teks status + statistik (`:72-86`).
//! - teks statistik (`:90-107`).
//! - cabang `on` / `off` beserta teksnya (`:111-156`).
//! - balasan opsi tidak valid (`:158-160`).
//!
//! `ponytail:` statistik pendaftaran (`getRegistrationStats`,
//! `sistemdaftar.js:29-44`) menghitung SELURUH user yang sudah daftar —
//! `db.getAllUsers()` tidak tersedia di `Ctx` (hanya data pengirim yang
//! diekspos). Jadi jumlahnya selalu `0`.
//!
//! `ponytail:` `sock.sendMessage(..., { contextInfo: getRegistrationContextInfo() })`
//! menandai pesan sebagai forwarded dari surat berita. `Reply::Text` tidak
//! punya slot `contextInfo`, jadi penandaan itu tidak ikut.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};

/// Key setting untuk status wajib daftar (`sistemdaftar.js:116,141`).
const KEY: &str = "registrationRequired";

/// Status saat ini (`sistemdaftar.js:68-69`).
fn status(ctx: &Ctx) -> bool {
    ctx.setting(KEY)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Statistik pendaftaran — selalu nol (lihat catatan modul).
fn statistik() -> (usize, usize, usize, usize) {
    (0, 0, 0, 0)
}

/// Teks status utama (`sistemdaftar.js:73-86`).
fn teks_status(ctx: &Ctx, on: bool) -> String {
    let (total, hari_ini, unreg, sesi) = statistik();
    format!(
        "⚙️ *sɪsᴛᴇᴍ ᴅᴀꜰᴛᴀʀ*\n\n\
         Status: {}\n\n\
         *Statistik:*\n\
         > Total registered: *{total}*\n\
         > Register hari ini: *{hari_ini}*\n\
         > Unreg hari ini: *{unreg}*\n\
         > Sesi aktif: *{sesi}*\n\n\
         *Usage:*\n\
         > `{}sistemdaftar on` - Wajibkan daftar\n\
         > `{}sistemdaftar off` - Matikan wajib daftar\n\
         > `{}sistemdaftar stats` - Lihat statistik\n\n\
         > Jika ON, user harus `{}daftar` sebelum pakai command",
        if on { "✅ ON (Wajib Daftar)" } else { "❌ OFF" },
        ctx.prefix,
        ctx.prefix,
        ctx.prefix,
        ctx.prefix
    )
}

/// Teks statistik (`sistemdaftar.js:91-101`).
fn teks_statistik(on: bool) -> String {
    let (total, hari_ini, unreg, sesi) = statistik();
    format!(
        "📊 *sᴛᴀᴛɪsᴛɪᴋ ᴅᴀꜰᴛᴀʀ*\n\n\
         Status sistem: {}\n\n\
         ╭┈┈⬡「 📈 *sᴛᴀᴛs* 」\n\
         ┃ Total registered: *{total}*\n\
         ┃ Register hari ini: *{hari_ini}*\n\
         ┃ Unreg hari ini: *{unreg}*\n\
         ┃ Sesi aktif: *{sesi}*\n\
         ╰┈┈┈┈┈┈┈┈⬡",
        if on { "✅ ON (Wajib Daftar)" } else { "❌ OFF" }
    )
}

#[plugin(
    name = "sistemdaftar",
    alias = ["regmode", "wajibdaftar", "togglereg"],
    category = "owner",
    description = "Kelola sistem wajib daftar dan statistik pendaftaran",
    usage = ".sistemdaftar <on/off/stats>",
    example = ".sistemdaftar stats",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn sistemdaftar(ctx: Ctx) -> Reply {
    let on = status(&ctx);
    let args = ctx.text.trim().to_lowercase();

    if args.is_empty() {
        return Reply::text(teks_status(&ctx, on));
    }

    if args == "stats" {
        return Reply::text(teks_statistik(on)).with_reaction("📊");
    }

    if args == "on" || args == "1" || args == "true" {
        return Reply::text(format!(
            "✅ *sɪsᴛᴇᴍ ᴅᴀꜰᴛᴀʀ ᴅɪᴀᴋᴛɪꜰᴋᴀɴ!*\n\n\
             User sekarang wajib daftar sebelum menggunakan command!\n\n\
             > Command: `{}daftar`",
            ctx.prefix
        ))
        .with_reaction("✅")
        .with_mutation(Mutation::SetSetting {
            key: KEY.to_string(),
            value: serde_json::Value::Bool(true),
        });
    }

    if args == "off" || args == "0" || args == "false" {
        return Reply::text(
            "❌ *sɪsᴛᴇᴍ ᴅᴀꜰᴛᴀʀ ᴅɪɴᴏɴᴀᴋᴛɪꜰᴋᴀɴ!*\n\n\
             User tidak perlu daftar untuk menggunakan command.",
        )
        .with_reaction("❌")
        .with_mutation(Mutation::SetSetting {
            key: KEY.to_string(),
            value: serde_json::Value::Bool(false),
        });
    }

    Reply::text("❌ Option tidak valid!\n\n> Gunakan: `on`, `off`, atau `stats`")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(teks: &str, settings: serde_json::Value) -> Reply {
        sistemdaftar(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "sistemdaftar".into(),
            settings,
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn nilai_setting(r: &Reply) -> Option<serde_json::Value> {
        r.mutations().iter().find_map(|m| match m {
            Mutation::SetSetting { key, value } if key == KEY => Some(value.clone()),
            _ => None,
        })
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("sistemdaftar").unwrap();
        assert_eq!(p.meta.name, "sistemdaftar");
        assert_eq!(p.meta.alias, ["regmode", "wajibdaftar", "togglereg"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Kelola sistem wajib daftar dan statistik pendaftaran"
        );
        assert_eq!(p.meta.usage, ".sistemdaftar <on/off/stats>");
        assert_eq!(p.meta.example, ".sistemdaftar stats");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["regmode", "wajibdaftar", "togglereg"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("sistemdaftar"), "{a}");
        }
    }

    #[test]
    fn tanpa_argumen_menampilkan_status_off() {
        let t = teks(&run("", serde_json::Value::Null));
        assert!(t.starts_with("⚙️ *sɪsᴛᴇᴍ ᴅᴀꜰᴛᴀʀ*\n\nStatus: ❌ OFF\n\n"), "{t}");
        assert!(t.contains("> Total registered: *0*"), "{t}");
        assert!(t.contains("> Sesi aktif: *0*"), "{t}");
        assert!(t.contains("> `.sistemdaftar on` - Wajibkan daftar"), "{t}");
        assert!(t.contains("> Jika ON, user harus `.daftar` sebelum pakai command"), "{t}");
    }

    #[test]
    fn status_on_terbaca_dari_setting() {
        let t = teks(&run("", serde_json::json!({ "registrationRequired": true })));
        assert!(t.contains("Status: ✅ ON (Wajib Daftar)"), "{t}");
    }

    #[test]
    fn stats_menampilkan_kotak_statistik() {
        let r = run("stats", serde_json::Value::Null);
        let t = teks(&r);
        assert!(t.starts_with("📊 *sᴛᴀᴛɪsᴛɪᴋ ᴅᴀꜰᴛᴀʀ*"), "{t}");
        assert!(t.contains("╭┈┈⬡「 📈 *sᴛᴀᴛs* 」"), "{t}");
        assert!(t.contains("┃ Total registered: *0*"), "{t}");
        assert!(t.ends_with("╰┈┈┈┈┈┈┈┈⬡"), "{t}");
        assert_eq!(reaksi(&r), Some("📊"));
    }

    #[test]
    fn on_menyimpan_setting_dan_reaksi() {
        let r = run("on", serde_json::Value::Null);
        assert_eq!(
            teks(&r),
            "✅ *sɪsᴛᴇᴍ ᴅᴀꜰᴛᴀʀ ᴅɪᴀᴋᴛɪꜰᴋᴀɴ!*\n\n\
             User sekarang wajib daftar sebelum menggunakan command!\n\n\
             > Command: `.daftar`"
        );
        assert_eq!(reaksi(&r), Some("✅"));
        assert_eq!(nilai_setting(&r), Some(serde_json::Value::Bool(true)));
    }

    #[test]
    fn off_menyimpan_setting_dan_reaksi() {
        let r = run("off", serde_json::json!({ "registrationRequired": true }));
        assert_eq!(
            teks(&r),
            "❌ *sɪsᴛᴇᴍ ᴅᴀꜰᴛᴀʀ ᴅɪɴᴏɴᴀᴋᴛɪꜰᴋᴀɴ!*\n\n\
             User tidak perlu daftar untuk menggunakan command."
        );
        assert_eq!(reaksi(&r), Some("❌"));
        assert_eq!(nilai_setting(&r), Some(serde_json::Value::Bool(false)));
    }

    #[test]
    fn opsi_tidak_valid_ditolak() {
        let r = run("ngawur", serde_json::Value::Null);
        assert_eq!(
            teks(&r),
            "❌ Option tidak valid!\n\n> Gunakan: `on`, `off`, atau `stats`"
        );
        assert!(nilai_setting(&r).is_none());
    }
}
