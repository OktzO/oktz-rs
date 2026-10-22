//! `plugins/owner/restart.js` — restart proses bot.
//!
//! ## Penyimpangan (tidak bisa ditiru)
//!
//! JS melakukan `spawn("node", [...execArgv, "index.js"], { detached: true })`
//! lalu `process.exit(0)` dua detik kemudian — proses Node barunya
//! menggantikan proses lama. Di Rust tidak ada kontrol proses dari dalam
//! plugin: `Ctx` tidak punya jalur untuk menjalankan proses baru, dan
//! `process.exit` akan mematikan bot beserta sesi WhatsApp-nya tanpa
//! opportunity reconnect.
//!
//! Karena itu port ini:
//! 1. tetap membalas blok "info" yang sama persis dengan JS (waktu lokal
//!    format `id-ID` dan PID proses aktif), dan
//! 2. menutup dengan `error_template` + reaction `☢` — padanan closest
//!    dengan cabang `catch` di JS, karena restart tidak benar-benar terjadi.
//!
//! `m.react('🔄')` tetap dipasang pada pesan pemicu.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `new Date().toLocaleTimeString('id-ID')` — `HH.mm.ss` (locale Indonesia
/// memakai titik sebagai pemisah waktu).
pub(crate) fn waktu_id(ms: i64) -> String {
    let (_, _, _, h, mi, s) = crate::common::jakarta_parts(ms);
    format!("{h:02}.{mi:02}.{s:02}")
}

/// Blok info `restart.js:30-37`.
pub(crate) fn blok_info(waktu: &str, pid: u32) -> String {
    format!(
        "🔄 *ʀᴇsᴛᴀʀᴛɪɴɢ ʙᴏᴛ...*\n\n\
         ╭┈┈⬡「 📊 *ɪɴꜰᴏ* 」\n\
         ┃ ⏰ Time: {waktu}\n\
         ┃ 🔧 Method: Process Spawn\n\
         ┃ 📦 PID: {pid}\n\
         ╰┈┈⬡\n\n\
         > Bot akan restart dalam 2 detik...\n\
         > Proses mungkin memakan waktu 10-30 detik"
    )
}

#[plugin(
    name = "restart",
    alias = ["reset", "reboot", "restartbot"],
    category = "owner",
    description = "Restart bot process (real restart)",
    usage = ".restart",
    example = ".restart",
    cooldown = 30,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn restart(ctx: Ctx) -> Reply {
    if !ctx.is_owner {
        return Reply::text("❌ *Owner Only!*");
    }

    let waktu = waktu_id(crate::common::now_ms());
    let pid = std::process::id();

    Reply::sequence(vec![
        Reply::None.with_reaction("🔄"),
        Reply::text(blok_info(&waktu, pid)),
        // `catch` JS: `await m.react('☢')` + `await m.reply(te(...))`.
        Reply::None.with_reaction("☢"),
        Reply::text(crate::common::error_template(&ctx)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(is_owner: bool) -> Reply {
        restart(Ctx {
            prefix: ".".into(),
            command: "restart".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            is_owner,
            ..Default::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("restart").unwrap();
        assert_eq!(p.meta.name, "restart");
        assert_eq!(p.meta.alias, ["reset", "reboot", "restartbot"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Restart bot process (real restart)");
        assert_eq!(p.meta.usage, ".restart");
        assert_eq!(p.meta.example, ".restart");
        assert_eq!(p.meta.cooldown, 30);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn bukan_owner_menolak() {
        let r = run(false);
        assert_eq!(teks(&r), "❌ *Owner Only!*");
    }

    #[test]
    fn blok_info_pola_js() {
        let t = blok_info("05.13.20", 4242);
        assert!(t.starts_with("🔄 *ʀᴇsᴛᴀʀᴛɪɴɢ ʙᴏᴛ...*"));
        assert!(t.contains("┃ ⏰ Time: 05.13.20"));
        assert!(t.contains("┃ 🔧 Method: Process Spawn"));
        assert!(t.contains("┃ 📦 PID: 4242"));
        assert!(t.ends_with("> Proses mungkin memakan waktu 10-30 detik"));
    }

    #[test]
    fn waktu_id_format_titik() {
        // 2023-11-15 06:00:00 WIB
        assert_eq!(waktu_id(1_700_002_800_000), "06.00.00");
    }

    #[test]
    fn urutan_balasan_sama_dengan_js() {
        let r = run(true);
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus Sequence");
        };
        assert_eq!(list.len(), 4);
        assert!(matches!(list[0].without_mutations(), Reply::Reaction { .. }));
        assert!(list[1].text_body().unwrap().starts_with("🔄 *ʀᴇsᴛᴀʀᴛɪɴɢ ʙᴏᴛ...*"));
        assert!(matches!(list[2].without_mutations(), Reply::Reaction { .. }));
        assert_eq!(
            list[3].text_body(),
            Some(crate::common::error_template_bawaan(".", "restart").as_str())
        );
    }
}
