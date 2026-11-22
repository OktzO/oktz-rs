//! `plugins/tools/ptv.js` — kirim video sebagai PTV (circle video).
//!
//! Port penuh: unduh video (quoted dulu, lalu video dengan caption pada pesan
//! pemicu) lewat `ctx.media_fetch`, kirim `Reply::video_gif` dengan mimetype
//! `video/mp4` + reaksi ✅.
//!
//! `ponytail:` WA `ptv: true` (bentuk video-note bulat) tidak ada padanannya
//! di `Reply::Video` (hanya `gif_playback`) — dikirim sebagai video loop tanpa
//! suara (efek visual sama seperti JS yang juga memakai `gifPlayback: true`).
//! Upgrade path: tambah flag `ptv` di `Reply::Video` + wire `videoNote`.

use oktz_plugin_api::{plugin, Ctx, IncomingKind, MediaOrigin, Reply};

fn usage(prefix: &str) -> String {
    let prefix = if prefix.is_empty() { "." } else { prefix };
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > Kirim *video* atau *balas video* lalu ketik:\n\
         > `{prefix}ptv`"
    )
}

fn gagal_unduh(asal: &str) -> String {
    format!("❌ Gagal download video dari {asal}.")
}

#[plugin(
    name = "ptv",
    alias = ["pvideo", "circlevideo"],
    category = "tools",
    description = "Kirim video sebagai PTV (circle video)",
    usage = ".ptv (reply video)",
    example = ".ptv",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn ptv(ctx: Ctx) -> Reply {
    // Sumber: quoted video dulu (JS `m.quoted.isVideo`), lalu video pada pesan
    // pemicu (JS `m.isVideo`).
    let (asal, label) = match (&ctx.quoted, &ctx.media) {
        (Some(q), _) if q.kind == oktz_plugin_api::QuotedKind::Video => {
            (MediaOrigin::Quoted, "quoted")
        }
        (None, Some(m)) if matches!(m.kind, IncomingKind::Video) => {
            (MediaOrigin::Trigger, "pesan")
        }
        _ => return Reply::text(usage(&ctx.prefix)),
    };

    let buffer = match ctx.media_fetch.download(asal) {
        Ok(b) => b,
        Err(_) => return Reply::text(gagal_unduh(label)),
    };
    // JS `if (!video)` → kartu cara pakai.
    if buffer.is_empty() {
        return Reply::text(usage(&ctx.prefix));
    }

    Reply::video_gif(
        oktz_plugin_api::MediaSource::bytes(buffer).with_mimetype("video/mp4"),
        "",
    )
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{IncomingMedia, MediaError, MediaFetch, QuotedKind, QuotedMessage, Reply as R};
    use std::sync::{Arc, Mutex};

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

    fn quoted_video() -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: String::new(),
            kind: QuotedKind::Video,
            mimetype: Some("video/mp4".into()),
            file_name: None,
            from_me: false,
            timestamp: 0,
        }
    }

    fn media_video() -> IncomingMedia {
        IncomingMedia {
            kind: IncomingKind::Video,
            mimetype: Some("video/mp4".into()),
            file_name: None,
            caption: String::new(),
        }
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

    fn run(ctx: Ctx) -> R {
        (oktz_plugin_api::find("ptv").unwrap().handler)(ctx)
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("ptv").unwrap();
        assert_eq!(p.meta.name, "ptv");
        assert_eq!(p.meta.alias, ["pvideo", "circlevideo"]);
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn without_a_video_it_shows_the_usage_card() {
        assert_eq!(run(Ctx::default()).text_body(), Some(usage("").as_str()));
    }

    #[test]
    fn quoted_video_is_downloaded_and_sent_as_looping_video() {
        let stub = stub_ok(&[0x00; 500]);
        let ctx = Ctx {
            quoted: Some(quoted_video()),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        let r = run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Quoted]);
        match payload(&r) {
            R::Video { media: Some(oktz_plugin_api::MediaSource::Bytes { data, mimetype }), caption, gif_playback: true, .. } => {
                assert_eq!(data.len(), 500);
                assert_eq!(mimetype.as_deref(), Some("video/mp4"));
                assert!(caption.is_empty());
            }
            other => panic!("harus Video gif_playback, dapat {other:?}"),
        }
    }

    #[test]
    fn trigger_video_uses_trigger_origin() {
        let stub = stub_ok(&[0x00; 500]);
        let ctx = Ctx {
            media: Some(media_video()),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        let r = run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Trigger]);
        assert!(payload(&r).has_media(), "harus kirim video");
    }

    #[test]
    fn quoted_wins_over_trigger_video_like_js() {
        let stub = stub_ok(&[0x00; 500]);
        let ctx = Ctx {
            quoted: Some(quoted_video()),
            media: Some(media_video()),
            media_fetch: Arc::new(stub.clone()),
            ..Default::default()
        };
        run(ctx);
        assert_eq!(*stub.unduhan.lock().unwrap(), vec![MediaOrigin::Quoted]);
    }

    #[test]
    fn non_video_quote_shows_usage_like_js() {
        for kind in [QuotedKind::Text, QuotedKind::Image, QuotedKind::Audio, QuotedKind::Sticker] {
            let mut q = quoted_video();
            q.kind = kind.clone();
            let ctx = Ctx {
                quoted: Some(q),
                ..Default::default()
            };
            assert_eq!(run(ctx).text_body(), Some(usage("").as_str()), "{kind:?}");
        }
    }

    #[test]
    fn download_error_becomes_gagal_download_text() {
        let ctx = Ctx {
            quoted: Some(quoted_video()),
            media_fetch: Arc::new(stub_err("unduh media quoted gagal: 410")),
            ..Default::default()
        };
        assert_eq!(run(ctx).text_body(), Some(gagal_unduh("quoted").as_str()));
    }

    #[test]
    fn empty_buffer_shows_usage_card_like_js() {
        // JS `if (!video)` — unduhan kosong dianggap tidak ada video.
        let ctx = Ctx {
            quoted: Some(quoted_video()),
            media_fetch: Arc::new(stub_ok(&[])),
            ..Default::default()
        };
        assert_eq!(run(ctx).text_body(), Some(usage("").as_str()));
    }

    #[test]
    fn success_wraps_with_checkmark_reaction() {
        let ctx = Ctx {
            quoted: Some(quoted_video()),
            media_fetch: Arc::new(stub_ok(&[0x00; 500])),
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
}
