//! `plugins/sticker/smeme-animated.js` — stiker meme animasi (Neoxr memegenvid).
//!
//! Alur yang diport apa adanya (`smeme-animated.js:18-78`):
//! - Guard teks (`smeme-animated.js:21-23`) dan guard gambar
//!   (`smeme-animated.js:33-35`).
//! - Pecah `atas|bawah` lalu `trim()` (`smeme-animated.js:25-27`).
//! - Unduh media: pesan yang di-reply kalau itu gambar, kalau tidak pesan
//!   pemicu (`smeme-animated.js:40-46`).
//! - Unggah gambar, lalu panggil `api.neoxr.eu/api/memegenvid` dengan
//!   `image`, `top`, `bottom`, `apikey` (`smeme-animated.js:48-50`).
//! - Kalau API gagal, gambar ASLI dikirim sebagai sticker
//!   (`smeme-animated.js:66-71`).
//! - Reaksi penutup `✅` / `☢`.
//!
//! `ponytail:`
//! - `uploadImage` (`src/lib/uploader.js:14-40`) mencoba 9 host keyless
//!   berurutan. Di sini hanya host pertama yang dipakai — Litterbox
//!   (`src/lib/upload-providers.js:21-37`) — karena bentuk multipart-nya
//!   persis sama dan host lain butuh parsing HTML/cookie khusus.
//! - `sock.sendImageAsSticker` tidak ada; yang dipakai `Reply::Sticker`.
//!   Metadata `packname`/`author` (`config.sticker`) tidak punya kanal di
//!   `Reply::Sticker`, sama seperti plugin sticker lain di repo ini.

use oktz_plugin_api::registry::{IncomingKind, MultipartPart};
use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

use crate::tools::phisingataubukan::encode_uri_component;

/// `smeme-animated.js:22` — teks atas/bawah belum diisi.
pub(crate) fn teks_kosong(prefix: &str, command: &str) -> String {
    format!("⚠️ Harap masukkan teks atas dan bawah!\nContoh: `{prefix}{command} atas|bawah`")
}

/// `smeme-animated.js:34` — harus kirim atau balas gambar.
pub(crate) fn tanpa_gambar(prefix: &str, command: &str) -> String {
    format!(
        "⚠️ Harap kirim atau balas gambar dengan caption `{prefix}{command} teks_atas|teks_bawah`"
    )
}

/// `smeme-animated.js:50` — endpoint Neoxr memegenvid.
pub(crate) fn url_memegenvid(image: &str, top: &str, bottom: &str, apikey: &str) -> String {
    format!(
        "https://api.neoxr.eu/api/memegenvid?image={}&top={}&bottom={}&apikey={apikey}",
        encode_uri_component(image),
        encode_uri_component(top),
        encode_uri_component(bottom)
    )
}

/// `src/lib/upload-providers.js:21-37` — unggah ke Litterbox (host pertama
/// di daftar `PROVIDERS`).
pub(crate) fn unggah_litterbox(
    http: &dyn oktz_plugin_api::HttpGet,
    buffer: &[u8],
) -> Result<String, String> {
    let req = HttpCall::post("https://litterbox.catbox.moe/resources/internals/api.php")
        .multipart(vec![
            MultipartPart::field("reqtype", "fileupload"),
            MultipartPart::field("time", "72h"),
            MultipartPart::file("fileToUpload", "image.jpg", buffer.to_vec())
                .with_mimetype("image/jpeg"),
        ]);
    let res = http.call(&req).map_err(|e| e.message)?;
    let teks = res.text().trim().to_string();
    if teks.starts_with("http") {
        Ok(teks)
    } else {
        Err("respons tanpa URL".to_string())
    }
}

#[plugin(
    name = "smeme-animated",
    alias = ["smeme-animated", "smemevid"],
    category = "sticker",
    description = "Buat stiker meme animasi",
    usage = ".smeme-animated <teks_atas>|<teks_bawah>",
    example = ".smeme-animated mas|anies (sambil balas gambar)",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn smeme_animated(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    if text.is_empty() {
        return Reply::text(teks_kosong(&ctx.prefix, &ctx.command));
    }

    // `text.split('|')` — hanya dua bagian pertama yang dipakai.
    let mut bagian = text.split('|');
    let top = bagian.next().unwrap_or("").trim().to_string();
    let bottom = bagian.next().unwrap_or("").trim().to_string();

    // `smeme-animated.js:30-32` — gambar di pesan pemicu atau di pesan yang
    // di-reply.
    let quoted_image = ctx.quoted().is_some_and(|q| q.kind.is_image());
    let trigger_image = ctx.is_media_kind(IncomingKind::Image);
    if !trigger_image && !quoted_image {
        return Reply::text(tanpa_gambar(&ctx.prefix, &ctx.command));
    }

    // `smeme-animated.js:40-46` — media yang di-reply didahulukan kalau ia
    // gambar; selain itu pesan pemicu.
    let media = if quoted_image {
        ctx.download_quoted_media().ok()
    } else {
        ctx.download_media().ok()
    };
    let Some(media) = media else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    // `smeme-animated.js:48` — unggah dulu, Neoxr butuh URL publik.
    let Ok(image_url) = unggah_litterbox(ctx.http.as_ref(), &media) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    // `smeme-animated.js:50-65` — panggil API meme animasi.
    let apikey = ctx.config().api_keys.neoxr.clone();
    let url = url_memegenvid(&image_url, &top, &bottom, &apikey);
    if let Ok(res) = ctx.http.get(&url) {
        let body = res.json().unwrap_or(Value::Null);
        let status_ok = body.get("status").is_some_and(crate::common::js_truthy_value);
        let sticker_url = body
            .get("data")
            .and_then(|d| d.get("url"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from);
        if status_ok {
            if let Some(sticker_url) = sticker_url {
                if let Ok(res) = ctx.http.get(&sticker_url) {
                    let buf = res.body.bytes().to_vec();
                    if !buf.is_empty() {
                        return Reply::sticker(MediaSource::bytes(buf)).with_reaction("✅");
                    }
                }
            }
        }
    }

    // `smeme-animated.js:66-71` — API gagal: kirim gambar aslinya sebagai
    // sticker.
    Reply::sticker(MediaSource::bytes(media)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse, IncomingMedia};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Stub {
        seen: Mutex<Vec<String>>,
        gagal_upload: bool,
    }

    impl Stub {
        fn baru() -> Arc<Stub> {
            Arc::new(Stub {
                seen: Mutex::new(Vec::new()),
                gagal_upload: false,
            })
        }
        fn gagal_upload() -> Arc<Stub> {
            Arc::new(Stub {
                seen: Mutex::new(Vec::new()),
                gagal_upload: true,
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            if url.contains("memegenvid") {
                return Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(
                            br#"{"status":true,"data":{"url":"https://cdn.example/sticker.webp"}}"#
                                .to_vec(),
                        ),
                        mimetype: None,
                    },
                    headers: vec![],
                });
            }
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(b"STICKER".to_vec()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            if req.url.contains("litterbox") {
                if self.gagal_upload {
                    return Err(HttpError {
                        message: "upload gagal".into(),
                    });
                }
                return Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(b"https://litter.catbox.moe/abc.jpg".to_vec()),
                        mimetype: None,
                    },
                    headers: vec![],
                });
            }
            self.get(&req.url)
        }
    }

    fn ctx_of(http: Arc<Stub>, gambar: bool, text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "smeme-animated".into(),
            http,
            media: gambar.then(|| IncomingMedia {
                kind: IncomingKind::Image,
                mimetype: Some("image/png".into()),
                file_name: None,
                caption: String::new(),
            }),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or("").to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        assert_eq!(
            teks_kosong(".", "smeme-animated"),
            "⚠️ Harap masukkan teks atas dan bawah!\nContoh: `.smeme-animated atas|bawah`"
        );
        assert_eq!(
            tanpa_gambar(".", "smeme-animated"),
            "⚠️ Harap kirim atau balas gambar dengan caption `.smeme-animated teks_atas|teks_bawah`"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("smeme-animated").unwrap();
        assert_eq!(p.meta.name, "smeme-animated");
        assert_eq!(p.meta.alias, ["smeme-animated", "smemevid"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Buat stiker meme animasi");
        assert_eq!(p.meta.usage, ".smeme-animated <teks_atas>|<teks_bawah>");
        assert_eq!(
            p.meta.example,
            ".smeme-animated mas|anies (sambil balas gambar)"
        );
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn url_memegenvid_mengikuti_pola_js() {
        assert_eq!(
            url_memegenvid("https://litter.catbox.moe/a.jpg", "atas", "bawah", "KUNCI"),
            "https://api.neoxr.eu/api/memegenvid?image=https%3A%2F%2Flitter.catbox.moe%2Fa.jpg&top=atas&bottom=bawah&apikey=KUNCI"
        );
    }

    #[test]
    fn tanpa_teks_menampilkan_pesan_atas_bawah() {
        let r = smeme_animated(ctx_of(Stub::baru(), true, ""));
        assert_eq!(teks(&r), teks_kosong(".", "smeme-animated"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn tanpa_gambar_menampilkan_pesan_gambar() {
        let r = smeme_animated(ctx_of(Stub::baru(), false, "atas|bawah"));
        assert_eq!(teks(&r), tanpa_gambar(".", "smeme-animated"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn media_gagal_diunduh_jatuh_ke_error_template() {
        let r = smeme_animated(ctx_of(Stub::baru(), true, "atas|bawah"));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "smeme-animated"));
        assert_eq!(reaksi(&r), Some("☢"));
    }

    #[test]
    fn upload_gagal_jatuh_ke_error_template() {
        let r = smeme_animated(ctx_of(Stub::gagal_upload(), true, "atas|bawah"));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "smeme-animated"));
        assert_eq!(reaksi(&r), Some("☢"));
    }
}
