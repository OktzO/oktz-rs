//! `plugins/sticker/animebrat.js` — sticker brat versi anime (latar pink).
//!
//! Perilaku yang diport apa adanya:
//! - Kartu pakai saat teks kosong (`animebrat.js:22-25`).
//! - `parseBratArgs` (`src/lib/brat.js:466-472`) — flag `-blur N` dibuang dari
//!   teks sebelum dikirim.
//! - Reaksi penutup `✅` / `☢`.
//!
//! `ponytail:` yang tidak bisa ditiru: `generateBrat` (`src/lib/brat.js:384-445`)
//! merender teks di atas kanvas 1000x1000 dengan font ArialNarrow dan latar
//! `#FF69B4` (hot pink) — tidak ada renderer teks/canvas di Rust, dan tidak ada
//! crate canvas yang boleh ditambah (dependensi repo tidak bisa diubah).
//! Supaya user tetap mendapat sticker brat berisi teksnya, plugin memakai
//! endpoint brat yang sama dengan `fetchBratFromAPI` (`src/lib/brat.js:447-463`)
//! yang dipakai plugin `brat` (`brat.js:142`). Bedanya: sticker dari API tidak
//! punya latar pink dan mengabaikan `-blur`.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use super::brat::{api_url, is_blank, parse_brat_args};

/// `animebrat.js:24` — kartu pakai.
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "🖼️ *ʙʀᴀᴛ ᴀɴɪᴍᴇ sᴛɪᴄᴋᴇʀ*\n\n> Masukkan teks\n\n`Contoh: {prefix}animebrat Hai semua`"
    )
}

#[plugin(
    name = "bratanime",
    alias = ["animebrat"],
    category = "sticker",
    description = "Membuat sticker brat",
    usage = ".animebrat <text>",
    example = ".animebrat Hai semua",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bratanime(ctx: Ctx) -> Reply {
    // `m.args.join(' ')` (`animebrat.js:22`).
    let text = ctx.args.join(" ");
    if text.is_empty() {
        return Reply::text(pakai(&ctx.prefix));
    }

    // `parseBratArgs` membuang flag `-blur N` (`src/lib/brat.js:466-472`).
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
        fn status(status: u16) -> Arc<Stub> {
            Arc::new(Stub {
                status,
                balasan: Some(b"halaman error".to_vec()),
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

    /// PNG palsu yang lolos ambang 100 byte (`brat.rs` memakai pola sama).
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
            command: "bratanime".into(),
            http,
            ..Ctx::default()
        };
        bratanime(ctx)
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
            "🖼️ *ʙʀᴀᴛ ᴀɴɪᴍᴇ sᴛɪᴄᴋᴇʀ*\n\n> Masukkan teks\n\n`Contoh: .animebrat Hai semua`"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("bratanime").unwrap();
        assert_eq!(p.meta.name, "bratanime");
        assert_eq!(p.meta.alias, ["animebrat"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Membuat sticker brat");
        assert_eq!(p.meta.usage, ".animebrat <text>");
        assert_eq!(p.meta.example, ".animebrat Hai semua");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn alias_animebrat_menunjuk_plugin_ini() {
        assert_eq!(
            oktz_plugin_api::find("animebrat").unwrap().meta.name,
            "bratanime"
        );
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
        let png = png_palsu();
        let http = Stub::ok(&png);
        let r = run(http.clone(), "Halo");
        assert_eq!(reaksi(&r), Some("✅"));
        let Reply::Reaction { reply, .. } = r.without_mutations() else {
            panic!("harus dibungkus reaksi");
        };
        match reply.as_ref() {
            Reply::Sticker { media } => assert_eq!(
                media,
                &Some(MediaSource::Bytes {
                    data: Arc::from(png),
                    mimetype: None,
                })
            ),
            lain => panic!("harus sticker, dapat {lain:?}"),
        }
        assert_eq!(
            http.seen.lock().unwrap().as_slice(),
            &["https://api.theresav.eu/api/maker/brat?text=Halo".to_string()]
        );
    }

    #[test]
    fn flag_blur_dibuang_sebelum_request() {
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
        for http in [
            Stub::offline(),
            Stub::status(500),
            Stub::ok(b"<html>blank</html>"),
        ] {
            let r = run(http, "Halo");
            assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "bratanime"));
            assert_eq!(reaksi(&r), Some("☢"));
        }
    }
}
