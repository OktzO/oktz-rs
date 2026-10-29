//! `plugins/ai/to3d.js` — ubah foto menjadi gaya 3D render.
//!
//! Alur JS (`to3d.js:24-58`): cek gambar → unduh (quoted dulu kalau
//! `m.quoted.isMedia`, lalu `m.isMedia`) → `live3d(buffer, PROMPT)` →
//! kirim gambar. Kegagalan apa pun masuk `catch` → template error `te(...)`.
//!
//! `live3d` dari `oktz_scraper::seaart` menerima `&dyn HttpGet`, buffer
//! gambar, dan prompt; mengembalikan `Result<Vec<u8>, HttpError>`.
//!
//! `ponytail:` reaksi `🕕` (`to3d.js:40`) tidak ikut — `Reply::with_reaction`
//! hanya bisa membawa satu emoji, jadi yang dikirim reaksi penutup.
//!
//! `ponytail:` `m.quoted.isMedia` (bukan `isImage`) yang dipakai untuk
//! menentukan sumber unduhan; `Ctx::quoted().is_some_and(|q| q.is_media())`
//! adalah padanannya.

use oktz_plugin_api::{Ctx, IncomingKind, MediaSource, QuotedKind, Reply, plugin};

const PROMPT: &str = "Transform this image into a high-quality 3D rendered style like Pixar or DreamWorks CGI. \nApply realistic lighting, smooth textures, and that polished 3D animated movie look. \nKeep the original composition but make it look like a frame from a modern 3D animated film \nwith subsurface scattering on skin, detailed hair, and cinematic lighting.";

fn is_image_input(ctx: &Ctx) -> bool {
    ctx.is_media_kind(IncomingKind::Image)
        || ctx
            .quoted()
            .map(|q| q.kind == QuotedKind::Image)
            .unwrap_or(false)
}

/// `m.quoted && m.quoted.isMedia` → quoted; `m.isMedia` → pemicu.
fn unduh_media(ctx: &Ctx) -> Option<Result<Vec<u8>, oktz_plugin_api::MediaError>> {
    if ctx.quoted().is_some_and(|q| q.is_media()) {
        return Some(ctx.download_quoted_media());
    }
    if ctx.is_media() {
        return Some(ctx.download_media());
    }
    None
}

#[plugin(
    name = "to3d",
    alias = ["3d", "3dfy", "to3dmodel"],
    category = "ai",
    description = "Ubah foto menjadi gaya 3D render",
    usage = ".to3d (reply/kirim gambar)",
    example = ".to3d",
    cooldown = 60,
    energi = 3,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = true,
    is_admin = false,
    is_bot_admin = false
)]
pub fn to3d(ctx: Ctx) -> Reply {
    if !is_image_input(&ctx) {
        // `to3d.js:26-32` — contoh memakai `${m.prefix}`.
        return Reply::text(format!(
            "🎮 *ᴛᴏ 3ᴅ*\n\n\
             > Kirim/reply gambar untuk diubah ke gaya 3D\n\n\
             `{prefix}to3d`",
            prefix = ctx.prefix
        ));
    }

    let Some(hasil) = unduh_media(&ctx) else {
        return Reply::text("❌ Gagal mendownload gambar").with_reaction("❌");
    };
    let Ok(buffer) = hasil else {
        return Reply::text("❌ Gagal mendownload gambar").with_reaction("❌");
    };

    // `to3d.js:42-49` — hasil kosong = exception → cabang `catch`.
    match oktz_scraper::seaart::live3d(ctx.http.as_ref(), &buffer, PROMPT) {
        Ok(image_bytes) => Reply::image(MediaSource::bytes(image_bytes), "").with_reaction("✅"),
        Err(_) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse, MediaError, MediaFetch, MediaOrigin};
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
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
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
            command: "to3d".into(),
            ..Ctx::default()
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

    fn ctx_gambar(
        http: Arc<HttpStub>,
        unduh: Result<Vec<u8>, MediaError>,
        quoted: Option<QuotedKind>,
    ) -> Ctx {
        Ctx {
            media: Some(oktz_plugin_api::IncomingMedia {
                kind: IncomingKind::Image,
                mimetype: Some("image/jpeg".into()),
                file_name: None,
                caption: String::new(),
            }),
            quoted: quoted.map(|kind| oktz_plugin_api::QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: ME.into(),
                text: String::new(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
            timestamp: 0,
            }),
            http: http.clone(),
            media_fetch: Arc::new(MediaStub(unduh)),
            ..ctx_dasar()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("to3d").unwrap();
        assert_eq!(p.meta.name, "to3d");
        assert_eq!(p.meta.alias, ["3d", "3dfy", "to3dmodel"]);
        assert_eq!(p.meta.category, "ai");
        assert_eq!(p.meta.description, "Ubah foto menjadi gaya 3D render");
        assert_eq!(p.meta.usage, ".to3d (reply/kirim gambar)");
        assert_eq!(p.meta.example, ".to3d");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 3);
        assert!(p.meta.is_premium);
        assert!(!p.meta.is_owner && !p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn setiap_alias_menunjuk_ke_plugin_ini() {
        for a in ["3d", "3dfy", "to3dmodel"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "to3d");
        }
    }

    #[test]
    fn bukan_gambar_mengembalikan_bantuan_dengan_prefix_ctx() {
        let ctx = Ctx {
            prefix: "!".into(),
            ..ctx_dasar()
        };
        let r = to3d(ctx);
        assert_eq!(
            teks(&r),
            "🎮 *ᴛᴏ 3ᴅ*\n\n> Kirim/reply gambar untuk diubah ke gaya 3D\n\n`!to3d`"
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn gagal_unduh_mengembalikan_pesan_dan_reaksi_silang() {
        let r = to3d(ctx_gambar(
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
        let r = to3d(ctx_gambar(HttpStub::offline(), Ok(vec![1, 2, 3]), None));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "to3d"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }
}
