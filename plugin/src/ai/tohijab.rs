//! `plugins/ai/tohijab.js` — tambahkan hijab ke gambar.
//!
//! Alur JS (`tohijab.js:19-45`): cek gambar → unduh (quoted dulu kalau
//! `m.quoted.isMedia`, lalu `m.isMedia`) → `uploadImage(buffer, 'image.jpg')`
//! → `applyImageEffect('tohijab', imageUrl)` → kirim gambar.
//!
//! Helper `uploadImage` + `applyImageEffect` ada di
//! [`super::efek`](crate::ai::efek) — salinan `src/lib/uploader.js`,
//! `src/lib/upload-providers.js`, dan `src/lib/effect-api.js`.
//!
//! `ponytail:` reaksi `🕕` (`tohijab.js:26`) tidak ikut — `Reply` hanya
//! membawa satu reaksi, jadi yang dikirim reaksi penutup.

use oktz_plugin_api::{Ctx, IncomingKind, MediaError, MediaSource, QuotedKind, Reply, plugin};

use super::efek::{apply_image_effect, upload_image};

/// Efek di `applyImageEffect` (`tohijab.js:38`).
const EFEK: &str = "tohijab";

/// Kartu pakai — `tohijab.js:21-23` memakai `${m.prefix}`.
fn help(prefix: &str) -> String {
    format!(
        "🧕 *ʜɪᴊᴀʙ sᴛʏʟᴇ*\n\n\
         > Kirim/reply gambar\n\n\
         `{prefix}tohijab`"
    )
}

/// `m.isImage || (m.quoted && m.quoted.type === 'imageMessage')`
/// (`tohijab.js:19`).
fn is_image(ctx: &Ctx) -> bool {
    ctx.is_media_kind(IncomingKind::Image)
        || ctx
            .quoted()
            .map(|q| q.kind == QuotedKind::Image)
            .unwrap_or(false)
}

/// `m.quoted && m.quoted.isMedia` → quoted; `m.isMedia` → pemicu
/// (`tohijab.js:28-32`).
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
    name = "tohijab",
    alias = ["hijab", "hijabstyle", "addhijab"],
    category = "ai",
    description = "Tambahkan hijab ke gambar",
    usage = ".tohijab (reply gambar)",
    example = ".tohijab",
    cooldown = 30,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn tohijab(ctx: Ctx) -> Reply {
    if !is_image(&ctx) {
        return Reply::text(help(&ctx.prefix));
    }

    let Some(hasil) = unduh_media(&ctx) else {
        return Reply::text("❌ Gagal mendownload gambar").with_reaction("❌");
    };
    let Ok(buffer) = hasil else {
        return Reply::text("❌ Gagal mendownload gambar").with_reaction("❌");
    };

    // `tohijab.js:37-39` — upload + efek; kegagalan keduanya masuk `catch`.
    let Ok(image_url) = upload_image(&*ctx.http, &buffer) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };
    let Ok(gambar) = apply_image_effect(&*ctx.http, EFEK, &image_url) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    Reply::image(MediaSource::bytes(gambar), String::new()).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use oktz_plugin_api::{
        HttpBody, HttpError, HttpResponse, IncomingMedia, MediaFetch, MediaOrigin, QuotedMessage,
    };
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    /// JPEG 12 byte — lolos `looks_like_image`.
    const GAMBAR: &[u8] = b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01";

    /// Stub HTTP + MediaFetch: balasan per `call` berurutan, habis berarti offline.
    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<(u16, Vec<u8>)>>,
        hitung: Mutex<usize>,
        panggilan: Mutex<Vec<oktz_plugin_api::HttpCall>>,
        unduhan: Mutex<Vec<MediaOrigin>>,
        hasil_unduh: Result<Vec<u8>, MediaError>,
    }

    impl Stub {
        fn baru(
            balasan: Vec<(u16, Vec<u8>)>,
            hasil_unduh: Result<Vec<u8>, MediaError>,
        ) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(balasan),
                hitung: Mutex::new(0),
                panggilan: Mutex::new(Vec::new()),
                unduhan: Mutex::new(Vec::new()),
                hasil_unduh,
            })
        }
    }

    impl oktz_plugin_api::HttpGet for Stub {
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.panggilan.lock().unwrap().push(req.clone());
            let i = {
                let mut hitung = self.hitung.lock().unwrap();
                let i = *hitung;
                *hitung += 1;
                i
            };
            match self.balasan.lock().unwrap().get(i).cloned() {
                Some((status, isi)) => Ok(HttpResponse {
                    status,
                    body: HttpBody::Bytes {
                        data: Arc::from(isi),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&oktz_plugin_api::HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
    }

    impl MediaFetch for Stub {
        fn download(&self, origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.unduhan.lock().unwrap().push(origin);
            self.hasil_unduh.clone()
        }
    }

    fn ctx_stub(
        http: Arc<Stub>,
        media: Option<IncomingMedia>,
        quoted: Option<QuotedKind>,
        prefix: &str,
    ) -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: prefix.into(),
            command: "tohijab".into(),
            media,
            quoted: quoted.map(|kind| QuotedMessage {
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
            media_fetch: http,
            ..Ctx::default()
        }
    }

    fn media_gambar() -> Option<IncomingMedia> {
        Some(IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        })
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

    fn bytes_gambar(r: &Reply) -> Vec<u8> {
        match r.without_mutations().tanpa_reaksi() {
            Reply::Image { media, .. } => match media {
                Some(MediaSource::Bytes { data, .. }) => data.to_vec(),
                lain => panic!("harus bytes, dapat {lain:?}"),
            },
            lain => panic!("harus image, dapat {lain:?}"),
        }
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = oktz_plugin_api::find("tohijab").unwrap();
        assert_eq!(p.meta.name, "tohijab");
        assert_eq!(p.meta.alias, ["hijab", "hijabstyle", "addhijab"]);
        assert_eq!(p.meta.category, "ai");
        assert_eq!(p.meta.description, "Tambahkan hijab ke gambar");
        assert_eq!(p.meta.usage, ".tohijab (reply gambar)");
        assert_eq!(p.meta.example, ".tohijab");
        assert_eq!(p.meta.cooldown, 30);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_owner && !p.meta.is_group && !p.meta.is_private);
        for a in ["hijab", "hijabstyle", "addhijab"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "tohijab");
        }
    }

    #[test]
    fn kartu_pakai_pakai_prefix_ctx() {
        // `tohijab.js:23` memakai `${m.prefix}`.
        let http = Stub::baru(vec![], Ok(Vec::new()));
        let r = tohijab(ctx_stub(http.clone(), None, None, "!"));
        assert_eq!(
            teks(&r),
            "🧕 *ʜɪᴊᴀʙ sᴛʏʟᴇ*\n\n> Kirim/reply gambar\n\n`!tohijab`"
        );
        assert_eq!(reaksi(&r), None);
        assert!(http.panggilan.lock().unwrap().is_empty());
    }

    #[test]
    fn unduhan_quoted_dipakai_lebih_dulu() {
        let http = Stub::baru(
            vec![
                (200, b"https://litterbox.example/img.jpg".to_vec()),
                (200, GAMBAR.to_vec()),
            ],
            Ok(b"gambar-asli".to_vec()),
        );
        let r = tohijab(ctx_stub(
            http.clone(),
            media_gambar(),
            Some(QuotedKind::Image),
            ".",
        ));
        assert_eq!(bytes_gambar(&r), GAMBAR);
        assert_eq!(reaksi(&r), Some("✅".into()));
        assert_eq!(*http.unduhan.lock().unwrap(), [MediaOrigin::Quoted]);
    }

    #[test]
    fn unduhan_gagal_membalas_pesan_dan_reaksi_silang() {
        let http = Stub::baru(
            vec![],
            Err(MediaError {
                message: "cdn mati".into(),
            }),
        );
        let r = tohijab(ctx_stub(http.clone(), media_gambar(), None, "."));
        assert_eq!(teks(&r), "❌ Gagal mendownload gambar");
        assert_eq!(reaksi(&r), Some("❌".into()));
        assert!(http.panggilan.lock().unwrap().is_empty());
    }

    #[test]
    fn upload_gagal_membalas_template_error() {
        let balasan: Vec<(u16, Vec<u8>)> = (0..8).map(|_| (500, Vec::new())).collect();
        let http = Stub::baru(balasan, Ok(b"gambar-asli".to_vec()));
        let r = tohijab(ctx_stub(http.clone(), media_gambar(), None, "."));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "tohijab"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn efek_non_2xx_membalas_template_error() {
        let http = Stub::baru(
            vec![
                (200, b"https://litterbox.example/img.jpg".to_vec()),
                (500, Vec::new()),
            ],
            Ok(b"gambar-asli".to_vec()),
        );
        let r = tohijab(ctx_stub(http.clone(), media_gambar(), None, "."));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "tohijab"));
        assert_eq!(reaksi(&r), Some("☢".into()));
        let panggilan = http.panggilan.lock().unwrap();
        assert_eq!(
            panggilan[1].url,
            "https://api-faa.my.id/faa/tohijab?url=https%3A%2F%2Flitterbox.example%2Fimg.jpg"
        );
    }
}
