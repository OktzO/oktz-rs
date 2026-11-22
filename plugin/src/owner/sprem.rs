//! `plugins/owner/sprem.js` — kirim ulang sticker sebagai premium (AI).
//!
//! ## Penyimpangan
//!
//! JS membangun `proto.Message.StickerMessage` dengan flag premium
//! (`isAiSticker: true`, `premium: 1`, `isLottie: false`, `isAnimated: true`)
//! lalu mengirimnya lewat `sock.relayMessage`. `Reply::Sticker` di API Rust
//! hanya bisa mengirim sticker biasa — flag premium/AI tidak ada di sana.
//! Yang dip-port: pengenalan stiker, penerusan byte stiker yang di-reply,
//! dan reaksi `✅`.
//!
//! `m.args[0]` di JS bisa mengarahkan sticker ke nomor lain; di Rust
//! `Reply` selalu dikirim ke chat pemicu, jadi itu tidak ikut.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

/// Teks bantuan `sprem.js:18-22`.
pub(crate) fn tanpa_sticker(prefix: &str) -> String {
    format!(
        "⭐ *sᴛɪᴄᴋᴇʀ ᴘʀᴇᴍɪᴜᴍ*\n\n\
         > Reply sticker yang mau dijadikan premium!\n\n\
         > Penggunaan: `{prefix}sprem`"
    )
}

/// `sprem.js:29` — sticker tidak bisa dibaca.
pub(crate) const GAGAL_BACA: &str = "❌ Gagal membaca data sticker";

#[plugin(
    name = "sprem",
    commands = [
        "stickerpremium", "premiumsticker",
    ],
    alias = [],
    category = "owner",
    description = "Kirim ulang sticker sebagai premium (Lottie/AI)",
    usage = ".sprem (reply sticker)",
    example = ".sprem",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn sprem(ctx: Ctx) -> Reply {
    let Some(q) = ctx.quoted() else {
        return Reply::text(tanpa_sticker(&ctx.prefix));
    };

    if !q.kind.is_sticker() {
        return Reply::text(GAGAL_BACA);
    }

    let Ok(byte) = ctx.download_quoted_media() else {
        return Reply::text(format!("❌ Gagal: {}", "sticker tidak dapat diunduh"));
    };

    Reply::sticker(MediaSource::bytes(byte)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{MediaFetch, MediaOrigin, QuotedKind, QuotedMessage};
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct Unduh {
        ok: bool,
    }

    impl MediaFetch for Unduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, oktz_plugin_api::MediaError> {
            if self.ok {
                Ok(b"STICKER".to_vec())
            } else {
                Err(oktz_plugin_api::MediaError {
                    message: "gagal".into(),
                })
            }
        }
    }

    fn run(quoted: Option<QuotedKind>, unduh_ok: bool) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "sprem".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: String::new(),
            args: Vec::new(),
            quoted: quoted.map(|kind| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: String::new(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
            timestamp: 0,
            }),
            media_fetch: Arc::new(Unduh { ok: unduh_ok }),
            ..Default::default()
        };
        sprem(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("sprem").unwrap();
        assert_eq!(p.meta.name, "sprem");
        assert_eq!(p.meta.commands, ["stickerpremium", "premiumsticker"]);
        assert!(p.meta.alias.is_empty(), "alias JS: []");
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Kirim ulang sticker sebagai premium (Lottie/AI)"
        );
        assert_eq!(p.meta.usage, ".sprem (reply sticker)");
        assert_eq!(p.meta.example, ".sprem");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_quote_menampilkan_panduan() {
        let r = run(None, true);
        assert_eq!(
            teks(&r),
            "⭐ *sᴛɪᴄᴋᴇʀ ᴘʀᴇᴍɪᴜᴍ*\n\n\
             > Reply sticker yang mau dijadikan premium!\n\n\
             > Penggunaan: `.sprem`"
        );
    }

    #[test]
    fn quoted_bukan_sticker_ditolak() {
        let r = run(Some(QuotedKind::Text), true);
        assert_eq!(teks(&r), GAGAL_BACA);
    }

    #[test]
    fn sticker_dikirim_dengan_reaksi_centang() {
        let r = run(Some(QuotedKind::Sticker), true);
        match r.without_mutations() {
            Reply::Reaction { emoji, reply } => {
                assert_eq!(emoji, "✅");
                assert!(matches!(reply.as_ref(), Reply::Sticker { .. }));
            }
            other => panic!("harus Reaction, dapat {other:?}"),
        }
    }

    #[test]
    fn gagal_unduh_menampilkan_pesan_gagal() {
        let r = run(Some(QuotedKind::Sticker), false);
        assert!(teks(&r).starts_with("❌ Gagal:"), "{}", teks(&r));
    }
}
