//! `plugins/ai/tooilpainting.js` — ubah foto menjadi gaya lukisan minyak.
//!
//! Alur JS (`tooilpainting.js:27-57`): cek gambar → unduh (quoted dulu kalau
//! `m.quoted.isMedia`, lalu `m.isMedia`) → `live3d(buffer, PROMPT)` →
//! kirim gambar. Kegagalan apa pun masuk `catch` → template error `te(...)`.
//!
//! `live3d` dari `oktz_scraper::seaart` menerima `&dyn HttpGet`, buffer
//! gambar, dan prompt; mengembalikan `Result<Vec<u8>, HttpError>`.
//!
//! `ponytail:` reaksi `🕕` (`tooilpainting.js:35`) tidak ikut — `Reply` hanya
//! bisa membawa satu emoji, jadi yang dikirim reaksi penutup.

use oktz_plugin_api::{Ctx, IncomingKind, MediaError, MediaSource, QuotedKind, Reply, plugin};

const PROMPT: &str = "Transform this image into a classical oil painting style. \nApply thick brushstrokes, rich colors, and the texture of traditional oil paint on canvas. \nKeep the original composition but make it look like a masterpiece painting \nwith visible brushwork, artistic color blending, and that timeless gallery-quality aesthetic.";

/// Kartu pakai — `tooilpainting.js:29-35` memakai `${m.prefix}`.
fn help(prefix: &str) -> String {
    format!(
        "🖼️ *ᴛᴏ ᴏɪʟ ᴘᴀɪɴᴛɪɴɢ*\n\n\
         > Kirim/reply gambar untuk diubah ke gaya lukisan minyak\n\n\
         `{prefix}tooilpainting`"
    )
}

/// `m.isImage || (m.quoted && (m.quoted.isImage || m.quoted.type === 'imageMessage'))`
/// (`tooilpainting.js:27`).
fn is_image(ctx: &Ctx) -> bool {
    ctx.is_media_kind(IncomingKind::Image)
        || ctx
            .quoted()
            .map(|q| q.kind == QuotedKind::Image)
            .unwrap_or(false)
}

/// `m.quoted && m.quoted.isMedia` → quoted; `m.isMedia` → pemicu
/// (`tooilpainting.js:38-42`).
fn unduh_media(ctx: &Ctx) -> Option<Result<Vec<u8>, MediaError>> {
    if ctx.quoted().is_some_and(|q| q.is_media()) {
        return Some(ctx.download_quoted_media());
    }
    if ctx.is_media() {
        return Some(ctx.download_media());
    }
    None
}

#[plugin(
    name = "tooilpainting",
    alias = ["oilpainting", "tooil", "oil"],
    category = "ai",
    description = "Ubah foto menjadi gaya lukisan minyak (oil painting)",
    usage = ".tooilpainting (reply/kirim gambar)",
    example = ".tooilpainting",
    cooldown = 60,
    energi = 3,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = true,
    is_admin = false,
    is_bot_admin = false
)]
pub fn tooilpainting(ctx: Ctx) -> Reply {
    if !is_image(&ctx) {
        return Reply::text(help(&ctx.prefix));
    }

    let Some(hasil) = unduh_media(&ctx) else {
        return Reply::text("❌ Gagal mendownload gambar").with_reaction("❌");
    };
    let Ok(buffer) = hasil else {
        return Reply::text("❌ Gagal mendownload gambar").with_reaction("❌");
    };

    // `tooilpainting.js:45-52` — hasil kosong = exception → cabang `catch`.
    match oktz_scraper::seaart::live3d(ctx.http.as_ref(), &buffer, PROMPT) {
        Ok(gambar) => Reply::image(MediaSource::bytes(gambar), String::new()).with_reaction("✅"),
        Err(_) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpError, HttpGet, HttpResponse, IncomingMedia, MediaFetch, MediaOrigin,
        QuotedMessage,
    };
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct HttpStub {
        balasan: Mutex<Vec<(u16, String)>>,
        hitung: Mutex<usize>,
    }

    impl HttpStub {
        fn urutan(daftar: Vec<(u16, &str)>) -> Arc<HttpStub> {
            Arc::new(HttpStub {
                balasan: Mutex::new(daftar.into_iter().map(|(s, t)| (s, t.to_string())).collect()),
                hitung: Mutex::new(0),
            })
        }
        fn offline() -> Arc<HttpStub> {
            Arc::new(HttpStub::default())
        }
        fn jawaban(&self) -> Result<HttpResponse, HttpError> {
            let mut h = self.hitung.lock().unwrap();
            let b = self.balasan.lock().unwrap();
            if b.is_empty() {
                return Err(HttpError {
                    message: "offline".into(),
                });
            }
            let i = (*h).min(b.len() - 1);
            *h += 1;
            let (status, teks) = b[i].clone();
            Ok(HttpResponse {
                status,
                body: HttpBody::Bytes {
                    data: Arc::from(teks.into_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
    }

    impl HttpGet for HttpStub {
        fn call(&self, _: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.jawaban()
        }
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            self.jawaban()
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
            self.jawaban()
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            self.jawaban()
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            self.jawaban()
        }
    }

    #[derive(Debug)]
    struct MediaStub(Result<Vec<u8>, MediaError>);

    impl MediaFetch for MediaStub {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.0.clone()
        }
    }

    fn ctx_dasar() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "tooilpainting".into(),
            ..Ctx::default()
        }
    }

    fn ctx_gambar(
        http: Arc<HttpStub>,
        unduh: Result<Vec<u8>, MediaError>,
        quoted: Option<QuotedKind>,
    ) -> Ctx {
        Ctx {
            media: Some(IncomingMedia {
                kind: IncomingKind::Image,
                mimetype: Some("image/jpeg".into()),
                file_name: None,
                caption: String::new(),
            }),
            quoted: quoted.map(|kind| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: ME.into(),
                text: String::new(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            http: http.clone(),
            media_fetch: Arc::new(MediaStub(unduh)),
            ..ctx_dasar()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = oktz_plugin_api::find("tooilpainting").unwrap();
        assert_eq!(p.meta.name, "tooilpainting");
        assert_eq!(p.meta.alias, ["oilpainting", "tooil", "oil"]);
        assert_eq!(p.meta.category, "ai");
        assert_eq!(
            p.meta.description,
            "Ubah foto menjadi gaya lukisan minyak (oil painting)"
        );
        assert_eq!(p.meta.usage, ".tooilpainting (reply/kirim gambar)");
        assert_eq!(p.meta.example, ".tooilpainting");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 3);
        assert!(p.meta.is_premium);
        assert!(!p.meta.is_owner && !p.meta.is_group && !p.meta.is_private);
        for a in ["oilpainting", "tooil", "oil"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "tooilpainting");
        }
    }

    #[test]
    fn prompt_sama_dengan_js() {
        assert_eq!(
            PROMPT,
            "Transform this image into a classical oil painting style. \nApply thick brushstrokes, rich colors, and the texture of traditional oil paint on canvas. \nKeep the original composition but make it look like a masterpiece painting \nwith visible brushwork, artistic color blending, and that timeless gallery-quality aesthetic."
        );
    }

    #[test]
    fn kartu_pakai_memakai_prefix_ctx() {
        let ctx = Ctx {
            prefix: "!".into(),
            ..ctx_dasar()
        };
        let r = tooilpainting(ctx);
        assert_eq!(
            teks(&r),
            "🖼️ *ᴛᴏ ᴏɪʟ ᴘᴀɪɴᴛɪɴɢ*\n\n> Kirim/reply gambar untuk diubah ke gaya lukisan minyak\n\n`!tooilpainting`"
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn gagal_unduh_membalas_pesan_dan_reaksi_silang() {
        let r = tooilpainting(ctx_gambar(
            HttpStub::offline(),
            Err(MediaError {
                message: "cdn mati".into(),
            }),
            None,
        ));
        assert_eq!(teks(&r), "❌ Gagal mendownload gambar");
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn gagal_live3d_mengembalikan_template_error_js() {
        let r = tooilpainting(ctx_gambar(HttpStub::offline(), Ok(vec![1, 2, 3]), None));
        assert_eq!(
            teks(&r),
            crate::common::error_template_bawaan(".", "tooilpainting")
        );
        assert_eq!(reaksi(&r), Some("☢".into()));
    }
}
