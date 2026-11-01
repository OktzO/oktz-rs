//! `plugins/canvas/iqcpink.js` — Fake Quote iOS style versi pink.
//!
//! `ponytail:` inti plugin JS adalah render pixel `@napi-rs/canvas` (background
//! catbox 906x1736, font Inter yang diunduh, emoji Apple dari JSON GitHub,
//! bubble rounded-rect + shadow, wrapping dengan `measureText`). `Kanvas`
//! (`oktz_plugin_api::kanvas`) tidak punya compositing gambar, font ganda,
//! shadow, maupun `measureText`, dan `Gambar` tidak membuka akses piksel
//! mentah — render 1:1 mustahil tanpa dependensi baru (`Cargo.toml` di luar
//! jangkauan port ini). Yang diport: cabang validasi persis JS; langkah
//! render jatuh ke cabang `catch` JS (`iqcpink.js:490-498`).
//!
//! `ponytail:` reaksi `🕕` (`iqcpink.js:468`) hilang — `Reply` hanya membawa
//! satu reaksi, jadi yang dipegang reaksi penutup (`❌` pada cabang catch;
//! `✅` tidak terjangkau selama render belum ada).

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `iqcpink.js:450-457` — kartu saat teks kosong DAN tidak ada gambar.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🩷 *FITUR FAKE QUOTE iOS PINK*\n\n\
         Sistem ini akan mengubah teks yang kamu berikan menjadi gambar *quote* bergaya iOS dengan tema Pink yang cantik dan elegan.\n\n\
         *CARA PENGGUNAAN:*\n\
         - Ketik `{prefix}iqcpink <teks kamu>`\n\
         - Atau balas (*reply*) pesan orang lain dengan perintah `{prefix}iqcpink`\n\n\
         _Pesan yang dibalas akan otomatis diubah menjadi quote menggunakan nama dan foto profil pembuat pesan tersebut!_"
    )
}

/// `iqcpink.js:497` — pesan cabang `catch`.
pub(crate) const GAGAL: &str =
    "❌ *GAGAL MEMPROSES QUOTE*\n\nMaaf, terjadi kesalahan saat mencoba membuat gambar quote pink. Silakan coba lagi beberapa saat.";

/// `text || (m.quoted && m.quoted.text ? m.quoted.text : "")`
/// (`iqcpink.js:434`).
pub(crate) fn target_text(ctx: &Ctx) -> String {
    if !ctx.text.is_empty() {
        return ctx.text.clone();
    }
    ctx.quoted().map(|q| q.text.clone()).unwrap_or_default()
}

/// `iqcpink.js:436-447` — quoted MEDIA dulu, baru media pesan pemicu.
/// Media bukan gambar diabaikan (JS `m.quoted.download` hanya untuk isMedia,
/// dan video/foto pemicu yang bukan gambar tetap dipakai apa adanya oleh JS —
/// `loadImage` akan gagal dan masuk catch, jadi di sini unduhan apa pun
/// diterima dan kegagalannya jatuh ke cabang yang sama).
pub(crate) fn target_image(ctx: &Ctx) -> Option<Vec<u8>> {
    if let Some(q) = &ctx.quoted
        && q.kind.is_media()
    {
        return ctx.download_quoted_media().ok();
    }
    if ctx.media.is_some() {
        return ctx.download_media().ok();
    }
    None
}

#[plugin(
    name = "iqcpink",
    alias = [],
    category = "canvas",
    description = "Membuat Fake Quote iOS style versi pink.",
    usage = ".iqcpink [text/reply]",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn iqcpink(ctx: Ctx) -> Reply {
    let text = target_text(&ctx);
    let gambar = target_image(&ctx);

    // `if (!targetText && !targetImgBuffer)` (`iqcpink.js:449`).
    if text.is_empty() && gambar.is_none() {
        return Reply::text(help(&ctx.prefix));
    }

    // Render canvas tidak tersedia — cabang catch JS (`iqcpink.js:490-498`).
    let _ = (&text, &gambar);
    Reply::text(GAGAL).with_reaction("❌")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{IncomingKind, MediaError, QuotedKind, QuotedMessage};

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_of(text: &str, quoted: Option<QuotedKind>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "iqcpink".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            quoted: quoted.map(|kind| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: ME.into(),
                text: matches!(kind, QuotedKind::Text)
                    .then(|| "kutipan".to_string())
                    .unwrap_or_default(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
                timestamp: 0,
            }),
            ..Ctx::default()
        }
    }

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("iqcpink").unwrap().handler)(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn meta_sesuai_config_js() {
        let p = oktz_plugin_api::find("iqcpink").unwrap();
        assert_eq!(p.meta.name, "iqcpink");
        assert!(p.meta.alias.is_empty());
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Membuat Fake Quote iOS style versi pink.");
        assert_eq!(p.meta.usage, ".iqcpink [text/reply]");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
    }

    #[test]
    fn teks_dan_gambar_kosong_mengirim_help_tanpa_reaksi() {
        let r = run(ctx_of("", None));
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn teks_terisi_jalan_ke_cabang_catch_dengan_silang() {
        let r = run(ctx_of("Kesendirian adalah teman terbaik ku😂😂", None));
        assert_eq!(teks(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn teks_spasi_bukan_kosong() {
        // JS `text ||` — spasi adalah truthy, jadi masuk jalur render.
        let r = run(ctx_of(" ", None));
        assert_eq!(teks(&r), GAGAL);
    }

    #[test]
    fn quoted_teks_dipakai_sebagai_target() {
        let r = run(ctx_of("", Some(QuotedKind::Text)));
        assert_eq!(teks(&r), GAGAL);
    }

    #[test]
    fn quoted_media_yang_unduhannya_gagal_jatuh_ke_help() {
        // JS menelan error `m.quoted.download()` (`iqcpink.js:447`) —
        // targetImgBuffer tetap null → help.
        let r = run(ctx_of("", Some(QuotedKind::Image)));
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn gambar_pemicu_tanpa_teks_jalan() {
        // Unduhan gagal (default NoMediaFetch) → targetImgBuffer tetap None,
        // tapi teks kosong → help, sesuai `!targetText && !targetImgBuffer`.
        let r = run(ctx_of("", None));
        assert_eq!(teks(&r), help("."));
    }

    #[test]
    fn target_text_mengikuti_prioritas_js() {
        let mut ctx = ctx_of("teks sendiri", Some(QuotedKind::Text));
        if let Some(q) = ctx.quoted.as_mut() {
            q.text = "teks kutipan".into();
        }
        assert_eq!(target_text(&ctx), "teks sendiri");
        ctx.text = String::new();
        assert_eq!(target_text(&ctx), "teks kutipan");
        ctx.quoted = None;
        assert_eq!(target_text(&ctx), "");
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#iqcpink <teks kamu>`"));
    }

    // MediaFetch yang selalu gagal — menegaskan unduhan yang error tidak
    // mengubah jalur (JS menelan error unduhan, `iqcpink.js:447`).
    #[test]
    fn unduhan_gagal_tetap_sama_dengan_tanpa_gambar() {
        #[derive(Debug)]
        struct Gagal;
        impl oktz_plugin_api::MediaFetch for Gagal {
            fn download(
                &self,
                _: oktz_plugin_api::MediaOrigin,
            ) -> Result<Vec<u8>, MediaError> {
                Err(MediaError {
                    message: "gagal".into(),
                })
            }
        }
        let mut ctx = ctx_of("", None);
        ctx.media = Some(oktz_plugin_api::IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        });
        ctx.media_fetch = std::sync::Arc::new(Gagal);
        let r = run(ctx);
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), None);
    }
}
