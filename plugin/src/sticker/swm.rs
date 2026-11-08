//! `plugins/sticker/swm.js` — ganti packname/author sticker.
//!
//! Guard (reply, sticker, packname) dan pemecahan `packname|author` di-port
//! apa adanya. `quoted.download()` → `Ctx::download_quoted_media()`;
//! `addExifToWebp` → `oktz_plugin_api::tempel_metadata_stiker` dengan
//! `EXIF_EMOJIS`, dan gambar/video non-WebP dikonversi lewat
//! `gambar_ke_stiker_webp` / `video_ke_stiker_webp` sebelum ditempel —
//! pola yang sama dengan `sticker::siapkan`. Kirim lewat
//! `Reply::sticker_opsi` + `StickerOpsi::gambar()` (forwarded + skor 1,
//! sama seperti `contextInfo` JS).
//!
//! `ponytail:` `contextInfo: { isForwarded: true, forwardingScore: 1 }` hanya
//! tersimpan sebagai flag di `StickerOpsi`; kutipan `quoted: m` mengikuti
//! default pipeline. Tag `☢` hanya di cabang `catch` JS (konversi gagal).

use oktz_plugin_api::{
    Ctx, EXIF_EMOJIS, MediaSource, Reply, StickerOpsi, plugin,
};

use crate::search::applemusic::error_reply;

/// `swm.js:22-30` — kartu cara pakai saat tidak ada pesan yang di-reply.
pub(crate) fn usage_card(prefix: &str) -> String {
    format!(
        "🖼️ *sᴛɪᴄᴋᴇʀ ᴡᴀᴛᴇʀᴍᴀʀᴋ*\n\n\
         > Reply sticker dengan caption:\n\
         > `{prefix}swm packname`\n\n\
         *ᴄᴏɴᴛᴏʜ:*\n\
         > `{prefix}swm Foto-AI`\n\
         > `{prefix}swm Foto-AI|LuckyArchz` _(packname + author)_"
    )
}

/// `swm.js:39-47` — `input.includes('|') ? [parts[0], parts[1]] : [input, '']`,
/// tiap bagian `.trim()` dan yang kosong jadi `""`.
pub(crate) fn split_input(input: &str) -> (String, String) {
    let mut parts = input.split('|');
    let packname = parts.next().unwrap_or("").trim().to_string();
    match parts.next() {
        // `parts[1]` saja — pipa ketiga diabaikan, bukan masuk author.
        Some(second) => {
            let author = second.trim().to_string();
            (packname, author)
        }
        None => (packname, String::new()),
    }
}

/// `swm.js:66-69` — buffer kosong atau unduhan gagal.
pub(crate) const GAGAL_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mendownload sticker";

/// `swm.js:72-74` — cek signature WebP (`RIFF....WEBP`).
pub(crate) fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

/// `swm.js:83-84` — cek signature video: byte 0..3 `000000` (hex) atau
/// `ftyp` di byte 4..8.
pub(crate) fn is_video(bytes: &[u8]) -> bool {
    bytes.len() >= 8
        && (&bytes[0..3] == [0u8; 3] || &bytes[4..8] == b"ftyp")
}

/// `swm.js:71-91` — tempel EXIF ke WebP, konversi gambar/video non-WebP
/// dulu (pola `sticker::siapkan`). `Err` = cabang `catch` JS.
fn siapkan(bytes: &[u8], packname: &str, author: &str) -> Result<Vec<u8>, String> {
    let webp = if is_webp(bytes) {
        bytes.to_vec()
    } else if is_video(bytes) {
        oktz_plugin_api::video_ke_stiker_webp(bytes, &[])?
    } else {
        oktz_plugin_api::gambar_ke_stiker_webp(bytes, &[])?
    };
    Ok(oktz_plugin_api::tempel_metadata_stiker(&webp, packname, author, &EXIF_EMOJIS)
        .unwrap_or(webp))
}
#[plugin(
    name = "swm",
    alias = ["wm", "stickerwm", "stickermark", "colong"],
    category = "sticker",
    description = "Mengganti packname dan author pada sticker",
    usage = ".swm <packname> atau .swm <packname>|<author>",
    example = ".swm BotName",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn swm(ctx: Ctx) -> Reply {
    let Some(quoted) = ctx.quoted() else {
        return Reply::text(usage_card(&ctx.prefix));
    };

    // `swm.js:33` — `quoted.type === 'stickerMessage' || quoted.isSticker`.
    if !quoted.kind.is_sticker() {
        // `.replace('Message', '')` lalu `|| 'media lain'` — jadi hanya string
        // kosong (atau `Other("")`) yang jatuh ke fallback.
        let label = quoted.kind.wire_type().replace("Message", "");
        let label = if label.is_empty() {
            "media lain".to_string()
        } else {
            label
        };
        return Reply::text(format!("❌ *ɢᴀɢᴀʟ*\n\n> Reply pesan sticker, bukan {label}"));
    }

    let input = ctx.text.trim();
    if input.is_empty() {
        let prefix = &ctx.prefix;
        return Reply::text(format!(
            "❌ *ɢᴀɢᴀʟ*\n\n\
             > Masukkan packname\n\n\
             *ᴄᴏɴᴛᴏʜ:*\n\
             > `{prefix}swm Foto-AI`\n\
             > `{prefix}swm Foto-AI|LuckyArchz` _(+ author)_"
        ));
    }

    let (packname, author) = split_input(input);

    // `swm.js:61-93` — `m.react('🕕')`, unduh, tempel EXIF, kirim sticker.
    let Ok(bytes) = ctx.download_quoted_media() else {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };
    if bytes.is_empty() {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    }

    match siapkan(&bytes, &packname, &author) {
        Ok(webp) => {
            let opsi = StickerOpsi::gambar()
                .with_packname(packname)
                .with_author(author);
            Reply::sticker_opsi(MediaSource::bytes(webp).with_mimetype("image/webp"), opsi)
                .with_reaction("✅")
        }
        // `swm.js:95-99` — `catch` → template error + ☢.
        Err(_) => Reply::text(error_reply(&ctx.prefix, &ctx.command)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        MediaError, MediaFetch, MediaOrigin, QuotedKind, QuotedMessage,
    };
    use std::sync::Arc;

    /// WebP minimal `RIFF....WEBP` + chunk VP8L 5 byte (pola `sticker.rs`).
    fn webp_bytes() -> Vec<u8> {
        let payload = [0x2f, 0x01, 0x02, 0x03, 0x04];
        let mut body = Vec::new();
        body.extend_from_slice(b"VP8L");
        body.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        body.extend_from_slice(&payload);
        if payload.len() % 2 == 1 {
            body.push(0);
        }
        let mut out = Vec::new();
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(4 + body.len() as u32).to_le_bytes());
        out.extend_from_slice(b"WEBP");
        out.extend_from_slice(&body);
        out
    }

    #[derive(Debug)]
    struct Unduh(Result<Vec<u8>, ()>);

    impl MediaFetch for Unduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.0.clone().map_err(|()| MediaError {
                message: "gagal unduh".into(),
            })
        }
    }

    fn quoted(kind: QuotedKind) -> Option<QuotedMessage> {
        Some(QuotedMessage {
            id: "MSG1".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            text: String::new(),
            kind,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        })
    }

    fn run(text: &str, q: Option<QuotedMessage>, fetch: Arc<Unduh>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "swm".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            text: text.into(),
            args: vec![text.into()],
            quoted: q,
            media_fetch: fetch,
            ..Default::default()
        };
        (oktz_plugin_api::find("swm").unwrap().handler)(ctx)
    }

    fn run_ok(text: &str, q: Option<QuotedMessage>) -> Reply {
        run(text, q, Arc::new(Unduh(Ok(webp_bytes()))))
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("swm").unwrap();
        assert_eq!(p.meta.name, "swm");
        assert_eq!(p.meta.alias, ["wm", "stickerwm", "stickermark", "colong"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(
            p.meta.description,
            "Mengganti packname dan author pada sticker"
        );
        assert_eq!(p.meta.usage, ".swm <packname> atau .swm <packname>|<author>");
        assert_eq!(p.meta.example, ".swm BotName");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["wm", "stickerwm", "stickermark", "colong"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "swm");
        }
    }

    #[test]
    fn without_a_quote_it_prints_the_usage_card() {
        let r = run_ok("Foto-AI", None);
        assert_eq!(text_of(&r), usage_card("."));
        assert!(text_of(&r).contains("`.swm Foto-AI|LuckyArchz`"));
        assert!(matches!(r, Reply::Text { .. }), "tanpa mutasi apa pun");
    }

    #[test]
    fn the_usage_card_follows_the_active_prefix() {
        let ctx = Ctx {
            prefix: "!".into(),
            command: "swm".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            text: "Foto-AI".into(),
            ..Default::default()
        };
        let r = (oktz_plugin_api::find("swm").unwrap().handler)(ctx);
        let body = text_of(&r);
        assert!(body.contains("`!swm packname`"), "{body}");
        assert!(!body.contains("`.swm"), "prefix lama tidak boleh bocor: {body}");
    }

    #[test]
    fn a_quoted_non_sticker_is_refused_by_its_wire_type() {
        for (kind, label) in [
            (QuotedKind::Image, "image"),
            (QuotedKind::Video, "video"),
            (QuotedKind::Audio, "audio"),
            (QuotedKind::Document, "document"),
            (QuotedKind::Text, "conversation"),
            (QuotedKind::Other(String::new()), "media lain"),
        ] {
            let r = run_ok("Foto-AI", quoted(kind));
            assert_eq!(
                text_of(&r),
                format!("❌ *ɢᴀɢᴀʟ*\n\n> Reply pesan sticker, bukan {label}"),
                "{label}"
            );
        }
    }

    #[test]
    fn a_quote_without_a_packname_asks_for_one() {
        let r = run_ok("   ", quoted(QuotedKind::Sticker));
        let body = text_of(&r);
        assert!(body.starts_with("❌ *ɢᴀɢᴀʟ*\n\n> Masukkan packname"), "{body}");
        assert!(body.contains("> `.swm Foto-AI`"), "{body}");
        assert!(body.contains("> `.swm Foto-AI|LuckyArchz` _(+ author)_"), "{body}");
        assert!(matches!(r, Reply::Text { .. }), "tanpa mutasi apa pun");
    }

    #[test]
    fn packname_and_author_are_split_on_the_first_pipe() {
        assert_eq!(
            split_input("Foto-AI|LuckyArchz"),
            ("Foto-AI".to_string(), "LuckyArchz".to_string())
        );
        assert_eq!(
            split_input("  Foto-AI  |  Lucky  Archz  "),
            ("Foto-AI".to_string(), "Lucky  Archz".to_string())
        );
        assert_eq!(
            split_input("Foto-AI|Lucky|Army"),
            ("Foto-AI".to_string(), "Lucky".to_string())
        );
        assert_eq!(
            split_input("Foto-AI|"),
            ("Foto-AI".to_string(), String::new())
        );
        assert_eq!(
            split_input("Foto-AI|   "),
            ("Foto-AI".to_string(), String::new())
        );
        assert_eq!(split_input("Foto-AI"), ("Foto-AI".to_string(), String::new()));
    }

    #[test]
    fn valid_inputs_send_a_sticker_with_the_packname_and_author() {
        for (text, pack, author) in [
            ("Foto-AI", "Foto-AI", None),
            ("Foto-AI|LuckyArchz", "Foto-AI", Some("LuckyArchz")),
            ("  Foto-AI  |  Lucky  ", "Foto-AI", Some("Lucky")),
        ] {
            let r = run_ok(text, quoted(QuotedKind::Sticker));
            let inner = match r.without_mutations() {
                Reply::Reaction { reply, .. } => reply.as_ref(),
                lain => lain,
            };
            match inner {
                Reply::Sticker { media, opsi } => {
                    let bytes = match media.as_ref().expect("sticker butuh sumber") {
                        MediaSource::Bytes { data, .. } => data.clone(),
                        m => panic!("harus bytes: {m:?}"),
                    };
                    assert!(is_webp(&bytes), "input {text:?}");
                    assert_eq!(opsi.packname.as_deref(), Some(pack), "input {text:?}");
                    assert_eq!(opsi.author.as_deref(), author, "input {text:?}");
                }
                lain => panic!("harus sticker, dapat {lain:?}"),
            }
            assert!(
                matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "✅"),
                "harus bereaksi ✅"
            );
        }
    }

    #[test]
    fn a_failed_or_empty_download_reports_the_js_message() {
        let r = run("Foto-AI", quoted(QuotedKind::Sticker), Arc::new(Unduh(Err(()))));
        assert_eq!(text_of(&r), GAGAL_UNDUH);
        assert!(
            matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "❌"),
            "harus bereaksi ❌"
        );
        let r = run("Foto-AI", quoted(QuotedKind::Sticker), Arc::new(Unduh(Ok(Vec::new()))));
        assert_eq!(text_of(&r), GAGAL_UNDUH);
    }

    #[test]
    fn signature_checks_follow_the_js_byte_rules() {
        let webp = webp_bytes();
        assert!(is_webp(&webp));
        assert!(!is_webp(b"RIFFxxxxWEB"));
        assert!(!is_webp(&[]));
        assert!(is_video(&[0, 0, 0, 0x04, b'f', b't', b'y', b'p']));
        assert!(is_video(&[0, 0, 0, b'x', b'f', b't', b'y', b'p']));
        assert!(!is_video(b"RIFF"));
        assert!(!is_video(&[]));
    }
}