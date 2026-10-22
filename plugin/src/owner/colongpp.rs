//! `plugins/owner/colongpp.js` — ambil & pakai foto profil target sebagai PP bot.
//!
//! ## Penyimpangan
//!
//! - JS memakai `@napi-rs/canvas` (`createCanvas(640, 640)` + `drawImage`).
//!   Di Rust yang dipakai `oktz_plugin_api::gambar::sharp` dengan
//!   `ubah_ukuran(640, 640)` (fit cover, sama seperti `drawImage` yang
//!   meregangkan ke kotak penuh) lalu `ke_jpeg`.
//! - `sock.updateProfilePicture(botJid, ...)` butuh JID bot sendiri, yang
//!   tidak ada di `Ctx`. Yang dikirim adalah JID non-grup (dipakai bot
//!   sendiri), karena implementasi `Sock::set_foto_profil` untuk JID
//!   non-grup selalu mengubah foto profil bot sendiri.
//! - `axios.get(..., { responseType: "arraybuffer" })` dipetakan ke
//!   `ctx.http.get(url)` + `.body.bytes()`.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// PP cadangan kalau target tidak punya foto profil (`colongpp.js:21`).
const FALLBACK_PP: &str = "https://telegra.ph/file/1ecdb5a0aee62ef17d7fc.jpg";

/// Ukuran PP bot (`colongpp.js:22`).
const PP_SIZE: u32 = 640;

/// Teks bantuan `colongpp.js:36-40`.
pub(crate) fn tanpa_target(prefix: &str) -> String {
    format!(
        "🕵️ *ᴄᴏʟᴏɴɢ ᴘᴘ*\n\n\
         > Reply pesan seseorang untuk mencuri PP-nya\n\n\
         *ᴄᴀʀᴀ:*\n\
         > Reply pesan target → `{prefix}colongpp`"
    )
}

/// Pesan sukses `colongpp.js:66-70`.
pub(crate) fn sukses(target_number: &str, sumber: &str) -> String {
    format!(
        "✅ *ᴘᴘ ʙᴇʀʜᴀsɪʟ ᴅɪᴄᴏʟᴏɴɢ!*\n\n\
         > 🎯 Target: @{target_number}\n\
         > 📸 Sumber: {sumber}"
    )
}

/// Label sumber PP (`colongpp.js:35` / `:59`).
pub(crate) const SUMBER_PROFIL: &str = "profil";
pub(crate) const SUMBER_DEFAULT: &str = "default (target tidak punya PP)";

/// Resize ke 640x640 JPEG — `resizeForPP` (`colongpp.js:23-30`).
fn resize_untuk_pp(buffer: &[u8]) -> Result<Vec<u8>, String> {
    oktz_plugin_api::gambar::sharp(buffer)?
        .ubah_ukuran(PP_SIZE, PP_SIZE)
        .ke_jpeg(100)
}

#[plugin(
    name = "colongpp",
    alias = ["stealpp", "malingpp", "ambilpp"],
    category = "owner",
    description = "Ambil & pakai foto profil target sebagai PP bot",
    usage = ".colongpp (reply pesan target)",
    example = ".colongpp",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn colongpp(ctx: Ctx) -> Reply {
    // `m.quoted?.sender || m.mentions?.[0]` (`colongpp.js:32`).
    let target = ctx
        .quoted()
        .map(|q| q.sender.clone())
        .or_else(|| ctx.mentioned.first().cloned());

    let Some(target) = target else {
        return Reply::text(tanpa_target(&ctx.prefix));
    };

    // `await m.react("🕵️")` (`colongpp.js:42`).
    let mut pp_buffer: Option<Vec<u8>> = None;
    let mut source = SUMBER_PROFIL;

    if let Ok(Some(url)) = ctx.sock().foto_profil(&target, "image") {
        if let Ok(res) = ctx.http.get(&url) {
            let byte = res.body.bytes().to_vec();
            if !byte.is_empty() {
                pp_buffer = Some(byte);
            }
        }
    }

    if pp_buffer.is_none() {
        // Fallback PP default (`colongpp.js:53-60`).
        if let Ok(res) = ctx.http.get(FALLBACK_PP) {
            let byte = res.body.bytes().to_vec();
            if !byte.is_empty() {
                pp_buffer = Some(byte);
                source = SUMBER_DEFAULT;
            }
        }
    }

    let Some(pp_buffer) = pp_buffer else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    let processed = match resize_untuk_pp(&pp_buffer) {
        Ok(p) => p,
        Err(_) => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    // JID non-grup → implementasi `Sock` mengubah foto profil bot sendiri.
    if ctx.sock().set_foto_profil(&ctx.sender, &processed).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    Reply::Text {
        text: sukses(crate::common::jid_number(&target), source),
        mentions: vec![target],
    }
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::QuotedMessage;

    fn run(quoted: Option<&str>, mentioned: Option<&str>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "colongpp".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: String::new(),
            args: Vec::new(),
            mentioned: mentioned.into_iter().map(str::to_string).collect(),
            quoted: quoted.map(|s| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            ..Default::default()
        };
        colongpp(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("colongpp").unwrap();
        assert_eq!(p.meta.name, "colongpp");
        assert_eq!(p.meta.alias, ["stealpp", "malingpp", "ambilpp"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Ambil & pakai foto profil target sebagai PP bot"
        );
        assert_eq!(p.meta.usage, ".colongpp (reply pesan target)");
        assert_eq!(p.meta.example, ".colongpp");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_target_menampilkan_panduan() {
        let r = run(None, None);
        assert_eq!(
            teks(&r),
            "🕵️ *ᴄᴏʟᴏɴɢ ᴘᴘ*\n\n\
             > Reply pesan seseorang untuk mencuri PP-nya\n\n\
             *ᴄᴀʀᴀ:*\n\
             > Reply pesan target → `.colongpp`"
        );
    }

    #[test]
    fn tanpa_sumber_gambar_mengakhiri_dengan_error_template() {
        // `SockMati` + `NoHttp` → tidak ada PP yang bisa diambil.
        let r = run(Some("628999@s.whatsapp.net"), None);
        assert_eq!(
            teks(&r),
            crate::common::error_template_bawaan(".", "colongpp")
        );
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "☢"),
            other => panic!("harus Reaction, dapat {other:?}"),
        }
    }

    #[test]
    fn pesan_sesuai_dengan_pola_js() {
        assert_eq!(
            sukses("628999", SUMBER_PROFIL),
            "✅ *ᴘᴘ ʙᴇʀʜᴀsɪʟ ᴅɪᴄᴏʟᴏɴɢ!*\n\n\
             > 🎯 Target: @628999\n\
             > 📸 Sumber: profil"
        );
        assert!(sukses("628999", SUMBER_DEFAULT).contains("default (target tidak punya PP)"));
    }
}
