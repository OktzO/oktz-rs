//! `plugins/sticker/smeme.js` — sticker meme dari gambar (memegen.link).
//!
//! Alur yang diport apa adanya (`smeme.js:25-127`):
//! - Guard: pesan pemicu atau pesan yang di-reply harus gambar/sticker.
//! - Format `top|bottom` — teks dipecah pada `|` lalu di-`trim()`.
//! - Unduh media: pesan yang di-reply dulu kalau memang media, baru pesan
//!   pemicu (`smeme.js:48-52`).
//! - Unggah gambar ke Telegraph (`https://telegra.ph/upload`, multipart dengan
//!   field `file`), lalu `src` digabung dengan `https://telegra.ph`
//!   (`smeme.js:79-89`).
//! - URL meme `api.memegen.link/images/custom/<top>/<bottom>.png?background=<url>`
//!   (`smeme.js:107`), hasilnya dikirim sebagai sticker.
//! - Semua teks balasan (kartu pakai, format, gagal unduh, gagal upload)
//!   persis seperti JS.
//!
//! `ponytail:`
//! - JS mengubah gambar ke 512x512 `fit: "contain"` dengan latar transparan
//!   sebelum diunggah (`smeme.js:58-67`). `oktz_plugin_api::gambar` tidak punya
//!   mode "contain + pad transparan" (hanya `cover` dan `inside`), jadi gambar
//!   diunggah apa adanya (dikonversi ke PNG kalau bisa). Memegen.link sendiri
//!   menyesuaikan ukuran latar, jadi hasil akhirnya hampir sama.
//! - `addExifToWebp` tidak ada di Rust, jadi metadata packname/author pada
//!   sticker tidak ikut — sama seperti plugin sticker lain di repo ini.

use oktz_plugin_api::registry::{IncomingKind, MultipartPart};
use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

use crate::tools::phisingataubukan::encode_uri_component;

/// `smeme.js:32` — kartu pakai saat tidak ada gambar.
pub(crate) fn pakai(prefix: &str, command: &str) -> String {
    format!(
        "😂 *ᴍᴇᴍᴇ sᴛɪᴄᴋᴇʀ*\n\n> Reply atau kirim gambar/sticker dengan caption\n\n`Contoh: {prefix}{command} Top|Bottom`"
    )
}

/// `smeme.js:38` — format `top|bottom` salah.
pub(crate) fn format_salah(prefix: &str, command: &str) -> String {
    format!(
        "😂 *ᴍᴇᴍᴇ sᴛɪᴄᴋᴇʀ*\n\n> Format: top|bottom\n\n`Contoh: {prefix}{command} Ketika|Kamu Lupa`"
    )
}

/// `smeme.js:55` — media gagal diunduh.
pub(crate) const GAGAL_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh media";

/// `smeme.js:95` — unggah gambar gagal.
pub(crate) const GAGAL_UPLOAD: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Gagal upload gambar, coba lagi nanti";

/// `smeme.js:98-104` — `encodeText` untuk memegen: teks kosong jadi `_`, lalu
/// `encodeURIComponent` dengan `-` → `--`, `_` → `__`, `%20` → `_`.
pub(crate) fn encode_text(teks: &str) -> String {
    if teks.is_empty() {
        return "_".to_string();
    }
    encode_uri_component(teks)
        .replace('-', "--")
        .replace('_', "__")
        .replace("%20", "_")
}

/// `smeme.js:107` — URL meme.
pub(crate) fn url_meme(top: &str, bottom: &str, background: &str) -> String {
    format!(
        "https://api.memegen.link/images/custom/{}/{}.png?background={}",
        encode_text(top),
        encode_text(bottom),
        encode_uri_component(background)
    )
}

/// `smeme.js:79-89` — unggah ke Telegraph; balikannya `src` absolut.
pub(crate) fn url_telegraph(body: &Value) -> Option<String> {
    let src = body
        .get(0)
        .and_then(|v| v.get("src"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())?;
    Some(format!("https://telegra.ph{src}"))
}

/// Konversi gambar ke PNG — padanan `.png()` sharp. Kalau gagal, byte asli
/// dipakai apa adanya (cabang `catch` di `smeme.js:68-71`).
fn ke_png(media: &[u8]) -> Vec<u8> {
    match oktz_plugin_api::gambar::sharp(media) {
        Ok(gambar) => gambar.ke_png().unwrap_or_else(|_| media.to_vec()),
        Err(_) => media.to_vec(),
    }
}

#[plugin(
    name = "smeme",
    alias = ["memesticker", "memes"],
    category = "sticker",
    description = "Membuat sticker meme dari gambar",
    usage = ".smeme <top>|<bottom>",
    example = ".smeme Ketika|Kamu Lupa",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn smeme(ctx: Ctx) -> Reply {
    // `smeme.js:26-30` — guard gambar/sticker.
    let ada_gambar = ctx.is_media_kind(IncomingKind::Image)
        || ctx
            .quoted()
            .is_some_and(|q| q.kind.is_image() || q.kind.is_sticker());
    if !ada_gambar {
        return Reply::text(pakai(&ctx.prefix, &ctx.command));
    }

    // `m.args.join(" ")` (`smeme.js:35`).
    let input = ctx.args.join(" ");
    if input.is_empty() || !input.contains('|') {
        return Reply::text(format_salah(&ctx.prefix, &ctx.command));
    }

    // `input.split("|").map(s => s.trim())` — hanya dua bagian pertama yang
    // dipakai (`smeme.js:41`).
    let mut bagian = input.split('|');
    let top = bagian.next().unwrap_or("").trim().to_string();
    let bottom = bagian.next().unwrap_or("").trim().to_string();

    // `smeme.js:48-52` — media yang di-reply dulu kalau memang media, lalu
    // pesan pemicu.
    let media = if ctx.quoted().is_some_and(|q| q.is_media()) {
        ctx.download_quoted_media().ok()
    } else if ctx.is_media() {
        ctx.download_media().ok()
    } else {
        None
    };
    let Some(media) = media else {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };

    // `smeme.js:72-77` — unggah ke Telegraph.
    let png = ke_png(&media);
    let Ok(res) = ctx.http.call(
        &HttpCall::post("https://telegra.ph/upload").multipart(vec![MultipartPart::file(
            "file",
            "meme.png",
            png,
        )
        .with_mimetype("image/png")]),
    ) else {
        return Reply::text(GAGAL_UPLOAD).with_reaction("❌");
    };
    let body = res.json().unwrap_or(Value::Null);
    let Some(background) = url_telegraph(&body) else {
        return Reply::text(GAGAL_UPLOAD).with_reaction("❌");
    };

    // `smeme.js:108-116` — unduh meme jadi sticker.
    let Ok(res) = ctx.http.call(
        &HttpCall::get(url_meme(&top, &bottom, &background)).header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        ),
    ) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };
    let buf = res.body.bytes().to_vec();
    if buf.is_empty() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    Reply::sticker(MediaSource::bytes(buf)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse, IncomingMedia, MediaError, MediaFetch, MediaOrigin};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<(String, Vec<u8>)>>,
        gagal: bool,
    }

    impl Stub {
        fn baru() -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(Vec::new()),
                gagal: false,
            })
        }
        fn gagal() -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(Vec::new()),
                gagal: true,
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.balasan
                .lock()
                .unwrap()
                .push((url.to_string(), b"PNG".to_vec()));
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(b"PNG".to_vec()),
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
            if self.gagal {
                return Err(HttpError {
                    message: "offline".into(),
                });
            }
            if req.url.contains("telegra.ph") {
                return Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(br#"[{"src":"/file/abc.jpg"}]"#.to_vec()),
                        mimetype: None,
                    },
                    headers: vec![],
                });
            }
            self.get(&req.url)
        }
    }

    #[derive(Debug)]
    struct MediaStub;

    impl MediaFetch for MediaStub {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            Ok(b"PNG".to_vec())
        }
    }

    fn ctx_of(http: Arc<Stub>, gambar: bool, args: &[&str]) -> Ctx {
        Ctx {
            text: args.join(" "),
            args: args.iter().map(|a| (*a).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "smeme".into(),
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
            pakai(".", "smeme"),
            "😂 *ᴍᴇᴍᴇ sᴛɪᴄᴋᴇʀ*\n\n> Reply atau kirim gambar/sticker dengan caption\n\n`Contoh: .smeme Top|Bottom`"
        );
        assert_eq!(
            format_salah(".", "smeme"),
            "😂 *ᴍᴇᴍᴇ sᴛɪᴄᴋᴇʀ*\n\n> Format: top|bottom\n\n`Contoh: .smeme Ketika|Kamu Lupa`"
        );
        assert_eq!(GAGAL_UNDUH, "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh media");
        assert_eq!(
            GAGAL_UPLOAD,
            "❌ *ɢᴀɢᴀʟ*\n\n> Gagal upload gambar, coba lagi nanti"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("smeme").unwrap();
        assert_eq!(p.meta.name, "smeme");
        assert_eq!(p.meta.alias, ["memesticker", "memes"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Membuat sticker meme dari gambar");
        assert_eq!(p.meta.usage, ".smeme <top>|<bottom>");
        assert_eq!(p.meta.example, ".smeme Ketika|Kamu Lupa");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_plugin_ini() {
        for a in ["memesticker", "memes"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "smeme");
        }
    }

    #[test]
    fn encode_text_mengikuti_aturan_js() {
        assert_eq!(encode_text(""), "_");
        assert_eq!(encode_text("a-b"), "a--b");
        assert_eq!(encode_text("a_b"), "a__b");
        assert_eq!(encode_text("a b"), "a_b");
        assert_eq!(encode_text("Halo dunia"), "Halo_dunia");
    }

    #[test]
    fn url_meme_mengikuti_pola_js() {
        assert_eq!(
            url_meme("atas", "bawah", "https://telegra.ph/file/a.jpg"),
            "https://api.memegen.link/images/custom/atas/bawah.png?background=https%3A%2F%2Ftelegra.ph%2Ffile%2Fa.jpg"
        );
    }

    #[test]
    fn tanpa_gambar_menampilkan_kartu_pakai() {
        let r = smeme(ctx_of(Stub::baru(), false, &["a|b"]));
        assert_eq!(teks(&r), pakai(".", "smeme"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn format_tanpa_pipes_menampilkan_pesan_format() {
        let r = smeme(ctx_of(Stub::baru(), true, &["niat"]));
        assert_eq!(teks(&r), format_salah(".", "smeme"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn media_gagal_diunduh_menampilkan_pesan_gagal() {
        let r = smeme(ctx_of(Stub::baru(), true, &["a|b"]));
        // `Ctx::default()` memakai `NoMediaFetch` yang selalu gagal.
        assert_eq!(teks(&r), GAGAL_UNDUH);
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn gagal_upload_menampilkan_pesan_gagal_upload() {
        let mut ctx = ctx_of(Stub::gagal(), true, &["a|b"]);
        ctx.media_fetch = Arc::new(MediaStub);
        let r = smeme(ctx);
        assert_eq!(teks(&r), GAGAL_UPLOAD);
        assert_eq!(reaksi(&r), Some("❌"));
    }
}
