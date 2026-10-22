//! `plugins/owner/backupsc.js` — backup script bot dalam bentuk zip.
//!
//! ## Penyimpangan (tidak bisa ditiru)
//!
//! - `archiver` (zip) tidak ada di API plugin: tidak ada crate kompresi
//!   yang boleh ditambahkan (brief melarang menyentuh `Cargo.toml`), dan
//!   `oktz_plugin_api` tidak mengekspos pembuatan zip.
//! - `fs.readdirSync`/`statSync` rekursif atas seluruh `process.cwd()`:
//!   `ctx.berkas()` berakar ke direktori kerja, jadi jalurnya bisa,
//!   tapi tidak ada penomoran berkas yang sah untuk zip.
//! - `contextInfo` newsletter pada `sock.sendMessage` tidak ada di `Reply`.
//!
//! Yang tetap sama: urutan balasan. JS mengirim reaction `🕕`, lalu pesan
//! "Memproses backup...", lalu (berhasil) dokumen zip. Di Rust langkah
//! terakhir tidak bisa dijalankan, jadi urutan itu diakhiri `error_template`
//! + reaction `☢` — persis seperti cabang `catch` di JS.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Pesan proses `backupsc.js:101-103`.
pub(crate) const MEMPROSES: &str =
    "📦 *ʙᴀᴄᴋᴜᴘ sᴄʀɪᴘᴛ*\n\n> Memproses backup...\n> Mohon tunggu sebentar...";

/// Caption dokumen berhasil `backupsc.js:196-204`.
pub(crate) fn caption(nama_zip: &str, ukuran_mb: String, jumlah_berkas: usize, tanggal: &str) -> String {
    format!(
        "✅ *ʙᴀᴄᴋᴜᴘ sᴇʟᴇsᴀɪ*\n\n\
         ╭┈┈⬡「 📋 *ᴅᴇᴛᴀɪʟ* 」\n\
         ┃ 📝 ɴᴀᴍᴀ: `{nama_zip}`\n\
         ┃ 📊 sɪᴢᴇ: `{ukuran_mb} MB`\n\
         ┃ 📁 ꜰɪʟᴇ: `{jumlah_berkas}`\n\
         ┃ 📅 ᴛᴀɴɢɢᴀʟ: `{tanggal}`\n\
         ╰┈┈⬡"
    )
}

#[plugin(
    name = "backupsc",
    alias = ["backup", "backupscript", "backupsource"],
    category = "owner",
    description = "Backup script bot dalam bentuk zip",
    usage = ".backupsc",
    example = ".backupsc",
    cooldown = 60,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn backupsc(ctx: Ctx) -> Reply {
    // `await m.react("🕕")` lalu `await m.reply(...)` (`backupsc.js:100-103`).
    Reply::sequence(vec![
        Reply::None.with_reaction("🕕"),
        Reply::text(MEMPROSES),
        // `catch` JS: `await m.react("☢")` + `await m.reply(te(...))`.
        Reply::None.with_reaction("☢"),
        Reply::text(crate::common::error_template(&ctx)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> Reply {
        backupsc(Ctx {
            prefix: ".".into(),
            command: "backupsc".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
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
        let p = oktz_plugin_api::find("backupsc").unwrap();
        assert_eq!(p.meta.name, "backupsc");
        assert_eq!(p.meta.alias, ["backup", "backupscript", "backupsource"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Backup script bot dalam bentuk zip");
        assert_eq!(p.meta.usage, ".backupsc");
        assert_eq!(p.meta.example, ".backupsc");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn urutan_balasan_sama_dengan_js() {
        let r = run();
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus Sequence");
        };
        assert_eq!(list.len(), 4);
        assert!(matches!(list[0].without_mutations(), Reply::Reaction { .. }));
        assert_eq!(list[1].text_body(), Some(MEMPROSES));
        assert!(matches!(list[2].without_mutations(), Reply::Reaction { .. }));
        assert_eq!(
            list[3].text_body(),
            Some(crate::common::error_template_bawaan(".", "backupsc").as_str())
        );
    }

    #[test]
    fn caption_dokumen_pola_js() {
        let t = caption("Bot_backup_2026-10-22.zip", "1.25".into(), 42, "22/10/2026");
        assert!(t.starts_with("✅ *ʙᴀᴄᴋᴜᴘ sᴇʟᴇsᴀɪ*"));
        assert!(t.contains("┃ 📝 ɴᴀᴍᴀ: `Bot_backup_2026-10-22.zip`"));
        assert!(t.contains("┃ 📊 sɪᴢᴇ: `1.25 MB`"));
        assert!(t.contains("┃ 📁 ꜰɪʟᴇ: `42`"));
        assert!(t.contains("┃ 📅 ᴛᴀɴɢɢᴀʟ: `22/10/2026`"));
    }
}
