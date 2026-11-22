//! `plugins/tools/toimg.js` — mengubah sticker menjadi gambar.
//!
//! Port penuh (plan Task 6): unduh sticker (quoted ATAU trigger) lewat
//! `ctx.media_fetch`, decode webp, encode PNG, kirim `Reply::image`.
//! Sticker animasi tidak bisa dibedakan dari statis di `Ctx` (`isAnimated`
//! tidak terekspos) — `ponytail:` sticker animasi tetap diproses sebagai
//! gambar; upgrade path: ekspos `isAnimated` di `IncomingMedia` untuk
//! menyarankan `.tovideo` seperti JS.

use oktz_plugin_api::{plugin, Ctx, IncomingKind, MediaOrigin, Reply};

fn no_sticker(prefix: &str) -> String {
    // Ctx::default() punya prefix kosong (test); produksi selalu terisi.
    let prefix = if prefix.is_empty() { "." } else { prefix };
    format!(
        "❌ *ɢᴀɢᴀʟ*\n\n\
         > Tidak ada sticker yang terdeteksi!\n\n\
         *Cara penggunaan:*\n\
         > 1. Kirim sticker + caption `{prefix}toimg`\n\
         > 2. Reply sticker dengan `{prefix}toimg`"
    )
}

fn gagal_unduh() -> String {
    "❌ *ɢᴀɢᴀʟ*\n\n\
     > Tidak dapat mengunduh sticker.\n\
     > Sticker mungkin sudah tidak tersedia."
        .to_string()
}

fn file_korup() -> String {
    "❌ *ꜰɪʟᴇ ᴋᴏʀᴜᴘ*\n\n\
     > File sticker tidak valid atau rusak.\n\
     > Coba kirim ulang stickernya."
        .to_string()
}

fn gagal_proses(e: &str) -> String {
    format!(
        "❌ *ᴇʀʀᴏʀ*\n\n\
         > Terjadi kesalahan saat memproses.\n\
         > _{e}_"
    )
}

#[plugin(
    name = "toimg",
    alias = ["toimage", "stickertoimage", "stimg"],
    category = "tools",
    description = "Mengubah sticker menjadi gambar",
    usage = ".toimg (reply/caption sticker)",
    example = ".toimg",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn toimg(ctx: Ctx) -> Reply {
    // Sumber: quoted sticker dulu (JS `quotedIsSticker`), lalu sticker dengan
    // caption (`selfIsSticker`).
    let asal = match (&ctx.quoted, &ctx.media) {
        (Some(q), _) if q.kind == oktz_plugin_api::QuotedKind::Sticker => MediaOrigin::Quoted,
        (None, Some(m)) if matches!(m.kind, IncomingKind::Sticker) => MediaOrigin::Trigger,
        _ => return Reply::text(no_sticker(&ctx.prefix)),
    };

    let buffer = match ctx.media_fetch.download(asal) {
        Ok(b) => b,
        Err(e) => return Reply::text(gagal_proses(&e.message)),
    };
    if buffer.is_empty() {
        return Reply::text(gagal_unduh());
    }
    if buffer.len() < 100 {
        return Reply::text(file_korup());
    }

    // Sticker WA = webp; `sharp` decode + `ke_png` encode — persis
    // `sock.sendMedia(..., { type: 'image' })` di JS.
    match oktz_plugin_api::gambar::sharp(&buffer) {
        Ok(gambar) => match gambar.ke_png() {
            Ok(png) => Reply::image(
                oktz_plugin_api::MediaSource::bytes(png).with_mimetype("image/png"),
                "",
            ),
            Err(e) => Reply::text(gagal_proses(&e)),
        },
        Err(e) => Reply::text(gagal_proses(&e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        Ctx, IncomingMedia, MediaError, MediaFetch, QuotedKind, QuotedMessage, Reply as R,
    };
    use std::sync::{Arc, Mutex};

    // PNG 32x32 valid (>100 byte) — fixture tanpa crate `image` di tests.
    const PNG_FIXTURE: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13,
        73, 72, 68, 82, 0, 0, 0, 32, 0, 0, 0, 32,
        8, 6, 0, 0, 0, 115, 122, 122, 244, 0, 0, 0,
        45, 73, 68, 65, 84, 120, 156, 237, 206, 33, 1, 0,
        0, 12, 2, 48, 210, 190, 127, 3, 30, 3, 51, 49,
        191, 228, 218, 41, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 129, 117, 224, 1, 104, 106,
        144, 136, 110, 58, 132, 145, 0, 0, 0, 0, 73, 69,
        78, 68, 174, 66, 96, 130,
    ];
    /// Stub unduhan: mencatat origin, mengembalikan hasil tetap.
    #[derive(Debug, Clone)]
    struct Stub {
        unduhan: Arc<Mutex<Vec<MediaOrigin>>>,
        hasil: Result<Vec<u8>, MediaError>,
    }

    impl MediaFetch for Stub {
        fn download(&self, origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.unduhan.lock().unwrap().push(origin);
            self.hasil.clone()
        }
    }

    fn stub_ok(bytes: &[u8]) -> Stub {
        Stub {
            unduhan: Arc::new(Mutex::new(Vec::new())),
            hasil: Ok(bytes.to_vec()),
        }
    }

    fn stub_err(msg: &str) -> Stub {
        Stub {
            unduhan: Arc::new(Mutex::new(Vec::new())),
            hasil: Err(MediaError { message: msg.to_string() }),
        }
    }

    fn quoted_sticker() -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: String::new(),
            kind: QuotedKind::Sticker,
            mimetype: Some("image/webp".into()),
            file_name: None,
            from_me: false,
            timestamp: 0,
        }
    }

    fn media_sticker() -> IncomingMedia {
        IncomingMedia {
            kind: IncomingKind::Sticker,
            mimetype: Some("image/webp".into()),
            file_name: None,
            caption: String::new(),
        }
    }

    fn run(ctx: Ctx) -> R {
        (oktz_plugin_api::find("toimg").unwrap().handler)(ctx)
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("toimg").unwrap();
        assert_eq!(p.meta.name, "toimg");
        assert_eq!(p.meta.alias, ["toimage", "stickertoimage", "stimg"]);
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn without_a_sticker_it_explains_both_ways() {
        let r = run(Ctx::default());
        assert_eq!(r.text_body(), Some(no_sticker(".").as_str()));
    }

    #[test]
    fn quoted_sticker_is_downloaded_and_sent_as_png() {
        let stub = stub_ok(PNG_FIXTURE);
        let ctx = Ctx {
            quoted: Some(quoted_sticker()),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        let r = run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Quoted]);
        match r.without_mutations() {
            R::Image { media: Some(oktz_plugin_api::MediaSource::Bytes { data, mimetype }), .. } => {
                assert!(data.len() > 100, "hasil re-encode harus PNG valid");
                assert_eq!(&data[..4], &[0x89, b'P', b'N', b'G'], "harus PNG");
                assert_eq!(mimetype.as_deref(), Some("image/png"));
            }
            other => panic!("harus Image bytes png, dapat {other:?}"),
        }
    }

    #[test]
    fn sticker_with_caption_uses_trigger_origin() {
        let stub = stub_ok(PNG_FIXTURE);
        let ctx = Ctx {
            media: Some(media_sticker()),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        let r = run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Trigger]);
        assert!(r.has_media(), "harus kirim gambar");
    }

    #[test]
    fn quoted_wins_over_caption_sticker_like_js() {
        let stub = stub_ok(PNG_FIXTURE);
        let ctx = Ctx {
            quoted: Some(quoted_sticker()),
            media: Some(media_sticker()),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Quoted]);
    }

    #[test]
    fn download_failure_becomes_error_text() {
        let stub = stub_err("unduh media quoted gagal: 410");
        let ctx = Ctx {
            quoted: Some(quoted_sticker()),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        let r = run(ctx);
        let t = r.text_body().unwrap();
        assert!(t.contains("ᴇʀʀᴏʀ"), "{t}");
        assert!(t.contains("410"), "pesan error harus membawa sebab: {t}");
    }

    #[test]
    fn empty_buffer_is_a_failure_message() {
        let stub = stub_ok(&[]);
        let ctx = Ctx {
            quoted: Some(quoted_sticker()),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        assert_eq!(run(ctx).text_body(), Some(gagal_unduh().as_str()));
    }

    #[test]
    fn tiny_buffer_is_reported_corrupt() {
        let stub = stub_ok(&[1, 2, 3]);
        let ctx = Ctx {
            quoted: Some(quoted_sticker()),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        assert_eq!(run(ctx).text_body(), Some(file_korup().as_str()));
    }

    #[test]
    fn undecodable_bytes_are_an_error_not_a_panic() {
        let stub = stub_ok(&[0x52; 200]); // 200 byte 'R' — bukan gambar
        let ctx = Ctx {
            quoted: Some(quoted_sticker()),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        let r = run(ctx);
        let t = r.text_body().unwrap();
        assert!(t.contains("ᴇʀʀᴏʀ"), "{t}");
    }
}
