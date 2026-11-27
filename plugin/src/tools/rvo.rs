//! Baca pesan sekali lihat (view once).

use oktz_plugin_api::{Ctx, MediaOrigin, QuotedKind, Reply, plugin};

fn no_quote(prefix: &str) -> String {
    format!(
        "Reply pesan sekali lihat (view once) untuk membukanya.\n\n\
         `Contoh: {prefix}rvo` (reply pesan view once)"
    )
}

fn not_view_once() -> &'static str {
    "❌ Reply pesan view once (sekali lihat) untuk membukanya."
}

fn gagal_buka(msg: &str) -> String {
    format!("❌ *Gagal Membuka View Once*\n\n> {msg}")
}

/// Pesan untuk galat unduh yang mengindikasikan media tak bisa diakses.
fn galat_kadaluarsa() -> &'static str {
    "Media tidak bisa dibuka dari perangkat ini.\n\n\
     Sejak update WhatsApp Nov 2026, isi view once tidak dikirim ke companion web-class. \
     Buka pesan di HP yang dipasangkan atau pasangkan bot sebagai companion Android.\n\n\
     Media yang terlalu lama atau sering dibuka juga bisa kadaluarsa dari server WhatsApp."
}

/// Deteksi galat unduh yang mengindikasikan media hangus dari server
/// . Dua keyword terakhir tidak ada di versi awal tapi perlu di Rust:
/// wacore melempar galat internal seperti "Missing file_sha256 for unencrypted
/// media" / "Missing media_key" untuk media yang kuncinya sudah dibakar server.
fn galat_mengarah_ke_kadaluarsa(msg: &str) -> bool {
    [
        "Gagal download",
        "decrypt",
        "download",
        "Timeout",
        "404",
        "Gone",
        "file_sha256",
        "fileSha256",
        "file_enc_sha256",
        "media_key",
        "kunci media",
    ]
    .iter()
    .any(|k| msg.contains(k))
}

fn mimetype_default(default: &str, quoted: &oktz_plugin_api::QuotedMessage) -> String {
    quoted
        .mimetype
        .clone()
        .unwrap_or_else(|| default.to_string())
}

fn caption_of(quoted_text: &str) -> String {
    if quoted_text.is_empty() {
        String::new()
    } else {
        format!("`Pesan :`\n> {quoted_text}")
    }
}

fn ekstensi_dari_kind(kind: &oktz_plugin_api::QuotedKind) -> String {
    // `wire_type()` = "imageMessage" dst; `Other(s)` menyimpan tipe wire mentah.
    let wire = kind.wire_type();
    let ext = wire.trim_end_matches("Message").to_lowercase();
    if ext.is_empty() {
        "bin".to_string()
    } else {
        ext
    }
}

fn timestamp_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[plugin(
    name = "rvo",
    alias = ["readvo", "readviewonce", "readview"],
    category = "tools",
    description = "Baca pesan sekali lihat (view once)",
    usage = ".rvo (reply pesan view once)",
    example = ".rvo",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn rvo(ctx: Ctx) -> Reply {
    let quoted = match ctx.quoted() {
        Some(q) if q.is_media() || matches!(q.kind, QuotedKind::ViewOnceText) => q,
        Some(_) => return Reply::text(not_view_once()),
        None => return Reply::text(no_quote(&ctx.prefix)),
    };

    if matches!(quoted.kind, QuotedKind::ViewOnceText) {
        return Reply::text(quoted.text.clone()).with_reaction("✅");
    }

    let caption = caption_of(&quoted.text);

    match ctx.media_fetch.download(MediaOrigin::Quoted) {
        Ok(buffer) if !buffer.is_empty() => {
            let reply = match &quoted.kind {
                QuotedKind::Image => Reply::image(
                    oktz_plugin_api::MediaSource::bytes(buffer)
                        .with_mimetype(mimetype_default("image/jpeg", quoted)),
                    caption,
                ),
                QuotedKind::Video => Reply::video(
                    oktz_plugin_api::MediaSource::bytes(buffer)
                        .with_mimetype(mimetype_default("video/mp4", quoted)),
                    caption,
                ),
                QuotedKind::Audio => Reply::audio(
                    oktz_plugin_api::MediaSource::bytes(buffer)
                        .with_mimetype(mimetype_default("audio/mpeg", quoted)),
                ),
                _ => {
                    let ext = ekstensi_dari_kind(&quoted.kind);
                    Reply::document(
                        oktz_plugin_api::MediaSource::bytes(buffer)
                            .with_mimetype(mimetype_default("application/octet-stream", quoted)),
                        format!("rvo_{}.{ext}", timestamp_now()),
                    )
                    .with_caption(if caption.is_empty() {
                        "📎 View once media".to_string()
                    } else {
                        caption
                    })
                }
            };
            reply.with_reaction("✅")
        }
        Ok(_) => Reply::text(gagal_buka(galat_kadaluarsa())),
        Err(e) => {
            let msg = if galat_mengarah_ke_kadaluarsa(&e.message) {
                galat_kadaluarsa().to_string()
            } else {
                e.message
            };
            Reply::text(gagal_buka(&msg)).with_reaction("☢")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        IncomingKind, IncomingMedia, MediaError, MediaFetch, QuotedKind, QuotedMessage, Reply as R,
    };
    use std::sync::{Arc, Mutex};

    /// PNG 32x32 valid (>100 byte) — fixture tanpa crate `image` di tests.
    const PNG_FIXTURE: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 32, 0, 0, 0, 32, 8,
        6, 0, 0, 0, 62, 177, 168, 86, 0, 0, 1, 164, 73, 68, 65, 84, 120, 156, 237, 207, 49, 14, 0,
        48, 12, 2, 192, 190, 183, 199, 102, 33, 23, 110, 214, 25, 116, 24, 24, 3, 133, 110, 176,
        131, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61,
        216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65,
        30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108,
        163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61,
        216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65,
        30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108,
        163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61,
        216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65,
        30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 193, 215, 227,
        15, 22, 232, 50, 37, 254, 230, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
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

    fn quoted(kind: QuotedKind) -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "isi pesan".into(),
            kind,
            mimetype: Some("image/webp".into()),
            file_name: None,
            from_me: false,
            timestamp: 0,
        }
    }

    fn run(ctx: Ctx) -> R {
        (oktz_plugin_api::find("rvo").unwrap().handler)(ctx)
    }

    /// Buka pembungkus `Mutate`/`Reaction` — balasan inti di dalamnya.
    fn payload(r: &R) -> &R {
        let mut cur = r;
        loop {
            cur = match cur {
                R::Mutate { reply, .. } => reply,
                R::Reaction { reply, .. } => reply,
                other => return other,
            };
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("rvo").unwrap();
        assert_eq!(p.meta.name, "rvo");
        assert_eq!(p.meta.alias, ["readvo", "readviewonce", "readview"]);
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn without_a_quote_it_explains_how_to_use() {
        let r = run(Ctx::default());
        assert!(r.text_body().unwrap().contains("Reply pesan sekali lihat"));
    }

    #[test]
    fn non_media_quote_is_refused() {
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Text)),
            ..Default::default()
        };
        assert_eq!(run(ctx).text_body().unwrap(), not_view_once());
    }

    #[test]
    fn view_once_text_quote_is_replied_as_text() {
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::ViewOnceText)),
            ..Default::default()
        };
        match payload(&run(ctx)) {
            R::Text { text, .. } => assert_eq!(text, "isi pesan"),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn image_quote_is_resent_as_image_with_caption() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Ok(PNG_FIXTURE.to_vec()),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Image)),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        let r = run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Quoted]);
        match payload(&r) {
            R::Image {
                media: Some(oktz_plugin_api::MediaSource::Bytes { data, mimetype }),
                caption,
                ..
            } => {
                assert_eq!(&data[..4], &[0x89, b'P', b'N', b'G']);
                assert_eq!(
                    mimetype.as_deref(),
                    Some("image/webp"),
                    "mimetype quoted harus diwarisi"
                );
                assert!(
                    caption.contains("isi pesan"),
                    "caption harus ikut: {caption}"
                );
            }
            other => panic!("harus Image, dapat {other:?}"),
        }
    }

    #[test]
    fn video_quote_is_resent_as_video() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Ok(vec![0x00; 200]),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Video)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        match payload(&run(ctx)) {
            R::Video {
                media: Some(_),
                caption,
                gif_playback: false,
                ..
            } => {
                assert!(caption.contains("isi pesan"));
            }
            other => panic!("harus Video, dapat {other:?}"),
        }
    }

    #[test]
    fn audio_quote_is_resent_as_audio() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Ok(vec![0x00; 200]),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Audio)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        match payload(&run(ctx)) {
            R::Audio {
                media: Some(_),
                ptt: false,
            } => {}
            other => panic!("harus Audio, dapat {other:?}"),
        }
    }

    #[test]
    fn unknown_media_kind_is_sent_as_document() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Ok(vec![0x00; 200]),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Document)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        match payload(&run(ctx)) {
            R::Document { file_name, .. } => {
                assert!(file_name.starts_with("rvo_"), "file_name = {file_name}");
                assert!(
                    file_name.ends_with(".document"),
                    "ekstensi dari wire kind: {file_name}"
                );
            }
            other => panic!("harus Document, dapat {other:?}"),
        }
    }

    #[test]
    fn empty_download_is_reported_as_expired() {
        // versi awal: `if (!buffer) throw new Error("Gagal download media")` → catch
        // match "Gagal download" → teks kadaluarsa. Unduhan kosong = sama.
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Ok(vec![]),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Image)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        let r = run(ctx);
        let t = r.text_body().unwrap();
        assert!(t.contains("Gagal Membuka View Once"), "{t}");
        assert!(
            t.contains("kadaluarsa"),
            "unduhan kosong = media hangus: {t}"
        );
    }

    #[test]
    fn expired_media_error_gets_friendly_message_and_radio_reaction() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Err(MediaError {
                message: "Gagal download media: 404".into(),
            }),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Image)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        let r = run(ctx);
        let t = r.text_body().unwrap();
        assert!(t.contains("Gagal Membuka View Once"), "{t}");
        assert!(t.contains("kadaluarsa"), "pesan hangus harus muncul: {t}");
        // Reaksi ☢ membungkus balasan.
        match r {
            R::Mutate { reply, mutations } => {
                assert!(matches!(*reply, R::Reaction { ref emoji, .. } if emoji == "☢"));
                assert!(mutations.is_empty());
            }
            other => panic!("harus dibungkus reaksi ☢, dapat {other:?}"),
        }
    }

    #[test]
    fn wacore_missing_key_error_maps_to_expired_text() {
        // Galat internal wacore untuk media tanpa kunci (view once hangus) harus
        // dipetakan ke teks kadaluarsa, bukan dibocorkan mentah ke pengguna.
        for pesan in [
            "unduh media quoted gagal: Missing file_sha256 for unencrypted media",
            "unduh media quoted gagal: Missing media_key for encrypted media",
            "Gagal download media quoted: kunci media hilang (media view once sudah hangus)",
        ] {
            let stub = Stub {
                unduhan: Arc::new(Mutex::new(vec![])),
                hasil: Err(MediaError {
                    message: pesan.into(),
                }),
            };
            let ctx = Ctx {
                quoted: Some(quoted(QuotedKind::Image)),
                media_fetch: Arc::new(stub),
                ..Default::default()
            };
            let r = run(ctx);
            let t = r.text_body().unwrap();
            assert!(
                t.contains("kadaluarsa"),
                "pesan {pesan:?} harus jadi teks kadaluarsa, dapat: {t}"
            );
            assert!(!t.contains("file_sha256"), "galat internal bocor: {t}");
        }
    }

    #[test]
    fn missing_key_error_points_to_web_companion_limitation() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Err(MediaError {
                message: "Gagal download media quoted: kunci media hilang".into(),
            }),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Image)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        let t = run(ctx).text_body().unwrap().to_string();
        assert!(t.contains("companion web"), "{t}");
    }

    #[test]
    fn generic_download_error_keeps_original_message() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Err(MediaError {
                message: "kripto rusak total".into(),
            }),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Image)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        let r = run(ctx);
        let t = r.text_body().unwrap();
        assert!(
            t.contains("kripto rusak total"),
            "pesan asli harus dipertahankan: {t}"
        );
        assert!(!t.contains("kadaluarsa"));
    }

    #[test]
    fn success_wraps_with_checkmark_reaction() {
        let stub = Stub {
            unduhan: Arc::new(Mutex::new(vec![])),
            hasil: Ok(PNG_FIXTURE.to_vec()),
        };
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Image)),
            media_fetch: Arc::new(stub),
            ..Default::default()
        };
        match run(ctx) {
            R::Mutate { reply, mutations } => {
                assert!(matches!(*reply, R::Reaction { ref emoji, .. } if emoji == "✅"));
                assert!(mutations.is_empty());
            }
            other => panic!("harus dibungkus reaksi ✅, dapat {other:?}"),
        }
    }

    #[test]
    fn other_incoming_media_is_ignored_like_js() {
        // Media trigger (bukan quoted) diabaikan — rvo hanya baca quoted.
        let ctx = Ctx {
            prefix: ".".into(),
            media: Some(IncomingMedia {
                kind: IncomingKind::Sticker,
                mimetype: None,
                file_name: None,
                caption: String::new(),
            }),
            ..Default::default()
        };
        assert_eq!(run(ctx).text_body().unwrap(), no_quote("."));
    }
}
