//! `plugins/sticker/bratgreen.js` — sticker brat hijau.
//!
//! Yang diport apa adanya:
//! - Kartu pakai (`bratgreen.js:24`) — `m.prefix` + nama command `bratgreen`
//!   (bukan alias `brat2`).
//! - `m.args.join(' ').trim()` sebagai teks (`bratgreen.js:22`).
//! - `parseBratArgs` (`src/lib/brat.js:466-472`) dan reaksi penutup.
//!
//! `ponytail:` `generateBrat({ theme: "green" })` (`src/lib/brat.js:384-445`)
//! merender teks di kanvas 1000x1000 dengan latar `#8ace00` — tidak ada
//! renderer teks di Rust dan dependensi repo tidak boleh diubah. Supaya user
//! tetap menerima sticker brat berisi teksnya, plugin memakai endpoint brat
//! yang sama dengan `fetchBratFromAPI` (`src/lib/brat.js:447-463`, dipakai
//! plugin `brat`). Bedanya: sticker dari API tidak hijau dan mengabaikan
//! `-blur`.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use super::brat::{api_url, is_blank, parse_brat_args};

/// `bratgreen.js:24` — kartu pakai.
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "🖼️ *BRAT GREEN*\n\n> Masukkan teks\n\n`Contoh: {prefix}bratgreen Hai semua`"
    )
}

#[plugin(
    name = "bratgreen",
    alias = ["brat2"],
    category = "sticker",
    description = "Membuat sticker brat ijo",
    usage = ".brat2 <text>",
    example = ".brat2 Hai semua",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bratgreen(ctx: Ctx) -> Reply {
    // `m.args.join(' ').trim()` (`bratgreen.js:22`).
    let text = ctx.args.join(" ").trim().to_string();
    if text.is_empty() {
        return Reply::text(pakai(&ctx.prefix));
    }

    let (parsed, _blur) = parse_brat_args(&text);

    let Ok(res) = ctx.http.get(&api_url(&parsed)) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };
    if !(200..300).contains(&res.status) {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }
    let buf = res.body.bytes().to_vec();
    if is_blank(&buf) {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    let cfg = ctx.config();
    let (pn, au) = (
        if cfg.sticker.packname.is_empty() { "Sticker".to_string() } else { cfg.sticker.packname.clone() },
        if cfg.sticker.author.is_empty() { "Bot".to_string() } else { cfg.sticker.author.clone() },
    );
    let webp = oktz_plugin_api::gambar_ke_stiker_webp(&buf, &[]).unwrap_or(buf.clone());
    let bytes = oktz_plugin_api::tempel_metadata_stiker(&webp, &pn, &au, &oktz_plugin_api::EXIF_EMOJIS).unwrap_or(webp);
    Reply::sticker_opsi(MediaSource::bytes(bytes).with_mimetype("image/webp"), oktz_plugin_api::StickerOpsi::gambar().with_packname(pn).with_author(au)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Stub {
        status: u16,
        balasan: Option<Vec<u8>>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn ok(bytes: &[u8]) -> Arc<Stub> {
            Arc::new(Stub {
                status: 200,
                balasan: Some(bytes.to_vec()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                status: 0,
                balasan: None,
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            match &self.balasan {
                Some(b) => Ok(HttpResponse {
                    status: self.status,
                    body: HttpBody::Bytes {
                        data: Arc::from(b.clone()),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
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
    }

    fn png_palsu() -> Vec<u8> {
        let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
        v.resize(200, 0);
        v
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "bratgreen".into(),
            http,
            ..Ctx::default()
        };
        bratgreen(ctx)
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
            pakai("."),
            "🖼️ *BRAT GREEN*\n\n> Masukkan teks\n\n`Contoh: .bratgreen Hai semua`"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("bratgreen").unwrap();
        assert_eq!(p.meta.name, "bratgreen");
        assert_eq!(p.meta.alias, ["brat2"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Membuat sticker brat ijo");
        assert_eq!(p.meta.usage, ".brat2 <text>");
        assert_eq!(p.meta.example, ".brat2 Hai semua");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn alias_brat2_menunjuk_plugin_ini() {
        assert_eq!(oktz_plugin_api::find("brat2").unwrap().meta.name, "bratgreen");
    }

    #[test]
    fn tanpa_teks_menampilkan_kartu_pakai_tanpa_reaksi() {
        let http = Stub::offline();
        let r = run(http.clone(), "");
        assert_eq!(teks(&r), pakai("."));
        assert_eq!(reaksi(&r), None);
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn sukses_mengirim_sticker_dan_reaksi_centang() {
        let http = Stub::ok(&png_palsu());
        let r = run(http.clone(), "Halo -blur 2");
        assert_eq!(reaksi(&r), Some("✅"));
        assert_eq!(
            http.seen.lock().unwrap().as_slice(),
            &["https://api.theresav.eu/api/maker/brat?text=Halo".to_string()]
        );
    }

    #[test]
    fn jalur_gagal_menghasilkan_error_template_dan_reaksi_bahu() {
        let r = run(Stub::offline(), "Halo");
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "bratgreen"));
        assert_eq!(reaksi(&r), Some("☢"));
    }
}
