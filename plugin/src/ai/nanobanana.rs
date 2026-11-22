//! Plugin baru (tidak ada padanannya di `ourin-md/plugins/`) — menggantikan
//! `kyzznekoo.zone.id/api/img/v2/nanobanana` yang balas 404 sejak 2026-11-15.
//!
//! Alur: unduh gambar (trigger/quoted) → `unggah::gambar` (API hanya menerima
//! link) → `rinn::nanobanana` → kirim byte hasil + MIME hasil sniff.
//!
//! `ponytail:` JS bereaksi `🕕` lalu `✅`/`❌`; `Reply::with_reaction` hanya
//! satu emoji, jadi hanya reaksi penutup yang dibawa.

use oktz_plugin_api::{Ctx, IncomingKind, MediaSource, Reply, plugin};
use oktz_scraper::unggah::{gambar, mime_gambar};

const GAGAL: &str = "🍀 *Waduhh, sepertinya ini ada kendala*\n\
                     Silahkan coba lagi nanti, dimohon jangan spam";

#[plugin(
    name = "nanobanana",
    alias = [],
    category = "ai",
    description = "Edit gambar dengan AI Nano Banana",
    usage = ".nanobanana <prompt>",
    example = ".nanobanana ubah bajunya jadi merah",
    cooldown = 30,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn nanobanana(ctx: Ctx) -> Reply {
    let prompt = ctx.args.join(" ");
    if prompt.trim().is_empty() {
        return Reply::text(
            "🍌 *NANO BANANA*\n\n\
             > Edit gambar dengan AI sesuai prompt\n\n\
             `Contoh: .nanobanana ubah bajunya jadi merah`\n\n\
             > Reply atau kirim gambar dengan caption",
        );
    }

    let is_image =
        ctx.is_media_kind(IncomingKind::Image) || ctx.quoted().is_some_and(|q| q.kind.is_image());

    if !is_image {
        return Reply::text("🍌 *ɴᴀɴᴏ ʙᴀɴᴀɴᴀ*\n\n> Reply atau kirim gambar dengan caption");
    }

    let buffer = if ctx.is_media_kind(IncomingKind::Image) {
        match ctx.download_media() {
            Ok(b) => b,
            Err(_) => {
                return Reply::text("❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh gambar")
                    .with_reaction("❌");
            }
        }
    } else {
        match ctx.download_quoted_media() {
            Ok(b) => b,
            Err(_) => {
                return Reply::text("❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh gambar")
                    .with_reaction("❌");
            }
        }
    };

    // Unggah dulu: `/ai-image/nanobanana` hanya menerima URL, bukan byte.
    let Ok(url) = gambar(ctx.http.as_ref(), &buffer) else {
        return Reply::text(GAGAL).with_reaction("❌");
    };

    match oktz_scraper::rinn::nanobanana(ctx.http.as_ref(), &url, &prompt) {
        Ok(hasil) => {
            let mime = mime_gambar(&hasil);
            Reply::image(MediaSource::bytes(hasil).with_mimetype(mime), String::new())
                .with_reaction("✅")
        }
        Err(_) => Reply::text(GAGAL).with_reaction("❌"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpError, HttpGet, HttpResponse, MediaError, MediaFetch, MediaOrigin,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct HttpStub {
        seen: Mutex<Vec<String>>,
        responses: Mutex<Vec<String>>,
        hit: Mutex<usize>,
    }

    impl HttpStub {
        fn ordered(responses: Vec<&str>) -> Arc<HttpStub> {
            Arc::new(HttpStub {
                seen: Mutex::new(Vec::new()),
                responses: Mutex::new(responses.into_iter().map(|s| s.to_string()).collect()),
                hit: Mutex::new(0),
            })
        }
        fn offline() -> Arc<HttpStub> {
            Arc::new(HttpStub {
                seen: Mutex::new(Vec::new()),
                responses: Mutex::new(Vec::new()),
                hit: Mutex::new(0),
            })
        }
    }

    impl HttpGet for HttpStub {
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            let mut hit = self.hit.lock().unwrap();
            let i = *hit;
            *hit += 1;
            let responses = self.responses.lock().unwrap();
            if i >= responses.len() {
                return Err(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                });
            }
            let body = responses.get(i).cloned().unwrap_or_default();
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(body.into_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
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

    #[derive(Debug, Default)]
    struct MediaStub {
        bytes: Vec<u8>,
        fail: bool,
    }

    impl MediaFetch for MediaStub {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            if self.fail {
                Err(MediaError {
                    message: "unduh gagal".into(),
                })
            } else {
                Ok(self.bytes.clone())
            }
        }
    }

    fn run(
        http: Arc<HttpStub>,
        media: Option<Vec<u8>>,
        media_fail: bool,
        quoted: bool,
        text: &str,
    ) -> Reply {
        let args: Vec<String> = text.split_whitespace().skip(1).map(|s| s.to_string()).collect();
        let ctx = Ctx {
            text: text.to_string(),
            args,
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            command: "nanobanana".into(),
            http,
            media: media.as_ref().map(|_| oktz_plugin_api::IncomingMedia {
                kind: oktz_plugin_api::IncomingKind::Image,
                mimetype: Some("image/jpeg".into()),
                file_name: None,
                caption: String::new(),
            }),
            media_fetch: Arc::new(MediaStub {
                bytes: media.unwrap_or_default(),
                fail: media_fail,
            }),
            quoted: if quoted {
                Some(oktz_plugin_api::QuotedMessage {
                    id: "q1".into(),
                    sender: "628999@s.whatsapp.net".into(),
                    chat: "628111@s.whatsapp.net".into(),
                    text: String::new(),
                    kind: oktz_plugin_api::QuotedKind::Image,
                    mimetype: None,
                    file_name: None,
                    from_me: false,
            timestamp: 0,
        })
            } else {
                None
            },
            ..Default::default()
        };
        nanobanana(ctx)
    }

    fn body(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or("").to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    fn bytes_gambar(r: &Reply) -> (Vec<u8>, Option<String>) {
        match r.without_mutations().tanpa_reaksi() {
            Reply::Image { media, .. } => match media {
                Some(MediaSource::Bytes { data, mimetype }) => {
                    (data.to_vec(), mimetype.clone())
                }
                lain => panic!("harus bytes, dapat {lain:?}"),
            },
            lain => panic!("harus image, dapat {lain:?}"),
        }
    }

    #[test]
    fn terdaftar_dengan_meta() {
        let p = oktz_plugin_api::find("nanobanana").unwrap();
        assert_eq!(p.meta.name, "nanobanana");
        assert!(p.meta.alias.is_empty());
        assert_eq!(p.meta.category, "ai");
        assert_eq!(p.meta.description, "Edit gambar dengan AI Nano Banana");
        assert_eq!(p.meta.usage, ".nanobanana <prompt>");
        assert_eq!(p.meta.example, ".nanobanana ubah bajunya jadi merah");
        assert_eq!(p.meta.cooldown, 30);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private && !p.meta.is_owner);
        assert!(!p.meta.is_premium && !p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn tanpa_prompt_tampilkan_bantuan() {
        let r = run(HttpStub::offline(), None, false, false, ".nanobanana");
        assert_eq!(
            body(&r),
            "🍌 *NANO BANANA*\n\n> Edit gambar dengan AI sesuai prompt\n\n\
             `Contoh: .nanobanana ubah bajunya jadi merah`\n\n\
             > Reply atau kirim gambar dengan caption"
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn tanpa_gambar_tampilkan_pesan() {
        let r = run(HttpStub::offline(), None, false, false, ".nanobanana ubah merah");
        assert_eq!(
            body(&r),
            "🍌 *ɴᴀɴᴏ ʙᴀɴᴀɴᴀ*\n\n> Reply atau kirim gambar dengan caption"
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn sukses_kirim_gambar_byte() {
        let http = HttpStub::ordered(vec![
            r#"{"files":[{"url":"https://uguu.example/x.jpg"}]}"#,
            "PNGDATA",
        ]);
        let r = run(http.clone(), Some(vec![9, 8, 7]), false, false, ".nanobanana ubah merah");
        // "PNGDATA" bukan signature PNG asli, jadi sniff jatuh ke JPEG.
        assert_eq!(bytes_gambar(&r), (b"PNGDATA".to_vec(), Some("image/jpeg".into())));
        assert_eq!(reaksi(&r), Some("✅".into()));
        // Urutan request: unggah (uguu) dulu, baru nanobanana.
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2);
        assert!(seen[0].contains("uguu.se"), "{}", seen[0]);
        assert!(seen[1].contains("/ai-image/notegpt-nano"), "{}", seen[1]);
        assert!(seen[1].contains("prompt=ubah%20merah"), "{}", seen[1]);
    }

    #[test]
    fn quoted_dipakai_kalau_tidak_ada_media_trigger() {
        let http = HttpStub::ordered(vec![
            r#"{"files":[{"url":"https://uguu.example/x.jpg"}]}"#,
            "GAMBAR",
        ]);
        let r = run(http.clone(), None, false, true, ".nanobanana ubah merah");
        assert_eq!(bytes_gambar(&r).0, b"GAMBAR");
        assert_eq!(reaksi(&r), Some("✅".into()));
    }

    #[test]
    fn unduh_gagal_kembalikan_pesan_gagal() {
        let r = run(
            HttpStub::offline(),
            Some(vec![1, 2, 3]),
            true,
            false,
            ".nanobanana ubah merah",
        );
        assert_eq!(body(&r), "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh gambar");
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn unggah_gagal_kembalikan_pesan_kendala() {
        // Stub offline: uguu + kappa dua-duanya gagal -> unggah gagal.
        let r = run(HttpStub::offline(), Some(vec![1, 2, 3]), false, false, ".nanobanana ubah merah");
        assert_eq!(body(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn nanobanana_gagal_kembalikan_pesan_kendala() {
        // Unggah sukses, tapi nanobanana membalas body kosong -> error.
        let http = HttpStub::ordered(vec![
            r#"{"files":[{"url":"https://uguu.example/x.jpg"}]}"#,
            "",
        ]);
        let r = run(http, Some(vec![1, 2, 3]), false, false, ".nanobanana ubah merah");
        assert_eq!(body(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }
}