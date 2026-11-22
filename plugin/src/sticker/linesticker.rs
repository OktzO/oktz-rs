//! `plugins/sticker/linesticker.js` — unduh sticker pack LINE lewat Neoxr.
//!
//! Yang diport apa adanya: validasi URL, pembuatan endpoint, kartu info pack,
//! daftar URL sticker (versi animasi kalau ada), dan semua pesan balasan
//! (`linesticker.js:23-119`).
//!
//! `ponytail:` `sock.sendImageAsSticker` / `sendVideoAsSticker` dengan
//! `packname`/`author` tidak tersedia di `Reply::Sticker`; metadata itu tidak
//! punya kanal sama sekali di registry Rust. Sticker dikirim apa adanya lewat
//! `Reply::Sticker`, dan video animasi juga dikirim sebagai sticker (tidak
//! ada varian "video sticker").
//!
//! `ponytail:` `Reply::Sequence` tidak boleh dibungkus `Reaction` — pipeline
//! merender `Sequence` lewat loop, bukan `render_reply`, jadi reaksi ✅/☢ pada
//! jalur multi-pesan tidak ikut (pola yang sama dipakai
//! `download/tiktokdl2.rs`).

use oktz_plugin_api::{Ctx, Reply, plugin};

const API: &str = "https://api.neoxr.eu/api/linesticker";

/// Kartu pakai (`linesticker.js:24-36`).
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "🎨 *ʟɪɴᴇ sᴛɪᴄᴋᴇʀ ᴘᴀᴄᴋ*\n\n\
         > Download LINE sticker pack\n\n\
         ╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ* 」\n\
         ┃ {prefix}linesticker <url>\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n\
         *ᴄᴀʀᴀ ᴅᴀᴘᴀᴛ ᴜʀʟ:*\n\
         > 1. Buka https://store.line.me\n\
         > 2. Pilih sticker pack\n\
         > 3. Copy URL dari browser\n\n\
         *ᴄᴏɴᴛᴏʜ:*\n\
         > {prefix}linesticker https://store.line.me/stickershop/product/9801/en"
    )
}

/// `/^(https?:\/\/store\.line\.me\/)/` (`linesticker.js:23`).
pub(crate) fn is_url_line(url: &str) -> bool {
    url.starts_with("https://store.line.me/") || url.starts_with("http://store.line.me/")
}

/// `apiUrl` (`linesticker.js:48`).
pub(crate) fn api_url(url: &str, apikey: &str) -> String {
    format!(
        "{API}?url={}&apikey={}",
        crate::tools::phisingataubukan::encode_uri_component(url),
        apikey
    )
}

/// `!res.data?.status || !res.data?.data` (`linesticker.js:51`).
pub(crate) fn respons_valid(data: &serde_json::Value) -> bool {
    crate::common::js_truthy(data.get("status")) && data.get("data").is_some()
}

/// `❌ Gagal mengambil sticker dari URL tersebut!` (`linesticker.js:53`).
pub(crate) const GAGAL_AMBIL: &str = "❌ Gagal mengambil sticker dari URL tersebut!";

/// `❌ API Key Neoxr tidak ditemukan di config!` (`linesticker.js:45`).
pub(crate) const KUNCI_TIDAK_ADA: &str = "❌ API Key Neoxr tidak ditemukan di config!";

/// `❌ Tidak ada sticker ditemukan!` (`linesticker.js:67`).
pub(crate) const TIDAK_ADA_STICKER: &str = "❌ Tidak ada sticker ditemukan!";

/// `❌ Gagal mengirim sticker` (`linesticker.js:112`).
pub(crate) const GAGAL_KIRIM: &str = "❌ Gagal mengirim sticker";

/// Info pack (`linesticker.js:70-79`).
pub(crate) fn render_info(title: &str, author: &str, animated: bool, total: usize) -> String {
    format!(
        "🎨 *ʟɪɴᴇ sᴛɪᴄᴋᴇʀ ᴘᴀᴄᴋ*\n\n\
         ╭┈┈⬡「 📦 *ɪɴꜰᴏ* 」\n\
         ┃ 📝 *Title:* {title}\n\
         ┃ 👤 *Author:* {author}\n\
         ┃ 🎬 *Animated:* {}\n\
         ┃ 📊 *Total:* {total}\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n\
         > 🕕 Mengirim sticker...",
        if animated { "Ya" } else { "Tidak" }
    )
}

/// Sukses kirim (`linesticker.js:108-109`).
pub(crate) fn berhasil_kirim(terkirim: usize, total: usize) -> String {
    format!("✅ Berhasil kirim {terkirim}/{total} sticker")
}

/// `linesticker.js:61-63` — daftar URL sticker: yang animasi kalau ada,
/// kalau tidak `sticker_url`.
pub(crate) fn url_sticker(data: &serde_json::Value, animated: bool) -> Vec<String> {
    let animated_urls = data
        .get("sticker_animation_url")
        .and_then(|v| v.as_array());
    if animated
        && let Some(list) = animated_urls
        && !list.is_empty()
    {
        return list
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    data.get("sticker_url")
        .and_then(|v| v.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[plugin(
    name = "linesticker",
    alias = ["linepack", "line"],
    category = "sticker",
    description = "Download sticker pack LINE",
    usage = ".linesticker <url>",
    example = ".linesticker https://store.line.me/stickershop/product/9801/en",
    cooldown = 25,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn linesticker(ctx: Ctx) -> Reply {
    let url = match ctx.args.first() {
        Some(u) => u.trim(),
        None => return Reply::text(pakai(&ctx.prefix)),
    };

    if url.is_empty() || !is_url_line(url) {
        return Reply::text(pakai(&ctx.prefix));
    }

    let apikey = ctx.config().api_keys.neoxr.clone();
    if apikey.is_empty() {
        return Reply::text(KUNCI_TIDAK_ADA).with_reaction("❌");
    }

    let Ok(res) = ctx.http.get(&api_url(url, &apikey)) else {
        return Reply::text(crate::common::error_template(&ctx))
            .with_reaction("☢");
    };
    let data = res.json().unwrap_or(serde_json::Value::Null);
    if !respons_valid(&data) {
        return Reply::text(GAGAL_AMBIL).with_reaction("❌");
    }

    let pack = data.get("data").unwrap_or(&serde_json::Value::Null);
    let title = pack.get("title").and_then(|v| v.as_str()).unwrap_or("LINE Sticker");
    let author = pack.get("author").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let animated = pack.get("animated").is_some_and(|v| crate::common::js_truthy(Some(v)));
    let sticker_urls = url_sticker(pack, animated);

    // `linesticker.js:65-68` — pack tanpa sticker.
    if sticker_urls.is_empty() {
        return Reply::text(TIDAK_ADA_STICKER).with_reaction("❌");
    }

    // `linesticker.js:70-79` — kartu info pack.
    let info = Reply::text(render_info(title, author, animated, sticker_urls.len()));

    // `linesticker.js:81-104` — unduh dan kirim maksimal 10 sticker.
    let mut terkirim = 0usize;
    let mut balasan: Vec<Reply> = vec![info];
    for url in sticker_urls.iter().take(10) {
        let Ok(res) = ctx.http.get(url) else {
            continue;
        };
        let buf = res.body.bytes().to_vec();
        if buf.is_empty() {
            continue;
        }
        terkirim += 1;
        balasan.push(Reply::sticker(oktz_plugin_api::MediaSource::bytes(buf)));
    }

    // `linesticker.js:107-113` — hasil pengiriman.
    //
    // `ponytail:` `Reply::Sequence` tidak boleh dibungkus `Reaction` (pipeline
    // merender `Sequence` lewat loop, bukan `render_reply`), jadi reaksi ✅/☢
    // tidak ikut di jalur ini — sama seperti `download/tiktokdl2.rs`.
    if terkirim > 0 {
        balasan.push(Reply::text(berhasil_kirim(terkirim, sticker_urls.len())));
    } else {
        balasan.push(Reply::text(GAGAL_KIRIM));
    }
    Reply::sequence(balasan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_dan_alias() {
        let p = oktz_plugin_api::find("linesticker").unwrap();
        assert_eq!(p.meta.name, "linesticker");
        assert_eq!(p.meta.alias, ["linepack", "line"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.cooldown, 25);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn pakai_dan_validasi_url() {
        assert!(
            pakai(".").contains(".linesticker https://store.line.me/stickershop/product/9801/en")
        );
        assert!(is_url_line("https://store.line.me/stickershop/product/9801/en"));
        assert!(is_url_line("http://store.line.me/x"));
        assert!(!is_url_line("https://google.com"));
        assert!(!is_url_line("store.line.me/x"));
    }

    #[test]
    fn api_url_mengemudikan_dan_masukkan_kunci() {
        assert_eq!(
            api_url("https://store.line.me/x", "KUNCI"),
            "https://api.neoxr.eu/api/linesticker?url=https%3A%2F%2Fstore.line.me%2Fx&apikey=KUNCI"
        );
    }

    #[test]
    fn respons_valid_miripkan_js() {
        assert!(!respons_valid(&serde_json::json!({})));
        assert!(!respons_valid(&serde_json::json!({ "status": true })));
        assert!(!respons_valid(&serde_json::json!({ "status": false, "data": {} })));
        assert!(respons_valid(&serde_json::json!({ "status": true, "data": {} })));
    }

    #[test]
    fn render_info_bentuk_js() {
        let out = render_info("Pack A", "Author B", true, 25);
        assert!(out.contains("📝 *Title:* Pack A"));
        assert!(out.contains("👤 *Author:* Author B"));
        assert!(out.contains("🎬 *Animated:* Ya"));
        assert!(out.contains("📊 *Total:* 25"));
        assert!(out.contains("> 🕕 Mengirim sticker..."));

        let out = render_info("X", "Y", false, 0);
        assert!(out.contains("🎬 *Animated:* Tidak"));
    }

    #[test]
    fn berhasil_kirim_bentuk_js() {
        assert_eq!(berhasil_kirim(3, 10), "✅ Berhasil kirim 3/10 sticker");
    }

    #[test]
    fn tanpa_url_tampilkan_pakai() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "linesticker".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Ctx::default()
        };
        let r = linesticker(ctx);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            pakai(".")
        );
    }

    #[test]
    fn kunci_kosong_reaksi_batsu() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "linesticker".into(),
            args: vec!["https://store.line.me/x".into()],
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Ctx::default()
        };
        let r = linesticker(ctx);
        assert_eq!(r.without_mutations().text_body().unwrap(), KUNCI_TIDAK_ADA);
        assert!(
            matches!(
                r.without_mutations(),
                Reply::Reaction { emoji, .. } if emoji == "❌"
            )
        );
    }

    #[test]
    fn url_bukan_line_tampilkan_pakai() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "linesticker".into(),
            args: vec!["https://google.com".into()],
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Ctx::default()
        };
        let r = linesticker(ctx);
        assert_eq!(r.without_mutations().text_body().unwrap(), pakai("."));
    }

    /// Stub HTTP: menjawab JSON untuk endpoint Neoxr dan byte PNG untuk URL
    /// sticker.
    #[derive(Debug)]
    struct Stub {
        body: String,
        seen: std::sync::Mutex<Vec<String>>,
    }

    impl oktz_plugin_api::HttpGet for Stub {
        fn get(&self, url: &str) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let body = if url.contains("neoxr") {
                self.body.clone().into_bytes()
            } else {
                b"PNG".to_vec()
            };
            Ok(oktz_plugin_api::HttpResponse {
                status: 200,
                body: oktz_plugin_api::HttpBody::Bytes {
                    data: std::sync::Arc::from(body),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
        fn post_json(
            &self,
            _: &str,
            _: &serde_json::Value,
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_form(
            &self,
            _: &str,
            _: &str,
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, oktz_plugin_api::HttpBody)],
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            unimplemented!("cuma GET")
        }
    }

    fn ctx_dengan(body: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "linesticker".into(),
            args: vec!["https://store.line.me/x".into()],
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            config: std::sync::Arc::new(oktz_plugin_api::config::Runtime {
                api_keys: oktz_plugin_api::config::ApiKeys {
                    neoxr: "KUNCI".into(),
                    ..oktz_plugin_api::config::ApiKeys::default()
                },
                ..oktz_plugin_api::config::Runtime::default()
            }),
            http: std::sync::Arc::new(Stub {
                body: body.to_string(),
                seen: std::sync::Mutex::new(Vec::new()),
            }),
            ..Ctx::default()
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        assert_eq!(TIDAK_ADA_STICKER, "❌ Tidak ada sticker ditemukan!");
        assert_eq!(GAGAL_KIRIM, "❌ Gagal mengirim sticker");
        assert_eq!(
            render_info("Pack A", "Author B", true, 25),
            "🎨 *ʟɪɴᴇ sᴛɪᴄᴋᴇʀ ᴘᴀᴄᴋ*\n\n\
             ╭┈┈⬡「 📦 *ɪɴꜰᴏ* 」\n\
             ┃ 📝 *Title:* Pack A\n\
             ┃ 👤 *Author:* Author B\n\
             ┃ 🎬 *Animated:* Ya\n\
             ┃ 📊 *Total:* 25\n\
             ╰┈┈┈┈┈┈┈┈⬡\n\n\
             > 🕕 Mengirim sticker..."
        );
        assert_eq!(berhasil_kirim(3, 10), "✅ Berhasil kirim 3/10 sticker");
    }

    #[test]
    fn pack_tanpa_sticker_menampilkan_pesan_tidak_ditemukan() {
        let body = r#"{"status":true,"data":{"title":"Pack","author":"A","animated":false,"sticker_url":[]}}"#;
        let r = linesticker(ctx_dengan(body));
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            TIDAK_ADA_STICKER
        );
        assert!(
            matches!(
                r.without_mutations(),
                Reply::Reaction { emoji, .. } if emoji == "❌"
            )
        );
    }

    #[test]
    fn pack_dengan_sticker_mengirim_info_sticker_dan_laporan() {
        let body = r#"{"status":true,"data":{"title":"Pack","author":"A","animated":false,"sticker_url":["https://cdn/1.png","https://cdn/2.png"]}}"#;
        let r = linesticker(ctx_dengan(body));
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus sequence, dapat {:?}", r.without_mutations());
        };
        // info + 2 sticker + laporan
        assert_eq!(list.len(), 4);
        assert!(list[0].text_body().unwrap().contains("📊 *Total:* 2"));
        assert!(matches!(list[1], Reply::Sticker { .. }));
        assert!(matches!(list[2], Reply::Sticker { .. }));
        assert_eq!(
            list[3].text_body().unwrap(),
            "✅ Berhasil kirim 2/2 sticker"
        );
    }

    #[test]
    fn url_sticker_memakai_versi_animasi_bila_ada() {
        let data = serde_json::json!({
            "sticker_url": ["https://cdn/a.png"],
            "sticker_animation_url": ["https://cdn/a.mp4"]
        });
        assert_eq!(
            url_sticker(&data, true),
            vec!["https://cdn/a.mp4".to_string()]
        );
        assert_eq!(url_sticker(&data, false), vec!["https://cdn/a.png".to_string()]);
        // Tanpa daftar animasi → jatuh ke `sticker_url`.
        let kosong = serde_json::json!({"sticker_url": ["https://cdn/a.png"]});
        assert_eq!(
            url_sticker(&kosong, true),
            vec!["https://cdn/a.png".to_string()]
        );
        assert!(url_sticker(&serde_json::json!({}), false).is_empty());
    }
}
