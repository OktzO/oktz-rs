//! `plugins/sticker/bratextra.js` — bikin brat versi custom (13 template).
//!
//! Yang diport apa adanya:
//! - Kartu pakai (`bratextra.js:166-168`) — memakai `m.command`, jadi nama
//!   alias yang diketik user ikut tampil.
//! - Percabangan per-alias: `brathd` memakai endpoint brat-HD eksternal
//!   (`bratextra.js:175-191`), 12 alias lain memakai template gambar lokal
//!   (`bratextra.js:193-317`).
//! - Reaksi penutup `✅` / `☢` dan pesan error `te(...)`.
//!
//! `ponytail:` yang tidak bisa ditiru:
//! - Render teks di atas template (`bratextra.js:198-312`) butuh
//!   `@napi-rs/canvas` + font Poppins + tokenisasi emoji + line wrap +
//!   rotasi. Tidak ada renderer teks di Rust dan dependensi repo tidak boleh
//!   diubah, jadi 12 template itu tidak bisa dibuat. Akibatnya plugin ini
//!   memakai endpoint brat yang sama dengan `fetchBratFromAPI`
//!   (`src/lib/brat.js:447-463`, dipakai plugin `brat`) supaya user tetap
//!   menerima sticker brat berisi teksnya. Bedanya: sticker dari API tidak
//!   memakai template khusus dan tidak mendukung `-blur`.
//! - `brathd` memakai `axios` dengan `validateStatus: () => true`; di sini
//!   `HttpCall::terima_semua_status()` meniru perilaku itu.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};

use super::brat::{api_url, is_blank};

/// `bratextra.js:167` — kartu pakai. `m.command` ikut tampil, jadi alias yang
/// diketik user (mis. `bratbahlil`) muncul apa adanya.
pub(crate) fn pakai(prefix: &str, command: &str) -> String {
    format!("⚠️ Harap masukkan teksnya!\nContoh: `{prefix}{command} Halo semuanya`")
}

/// `bratextra.js:176` — endpoint brat HD.
const ENDPOINT_BRATHD: &str = "https://aqul-brat.hf.space/";

/// `bratextra.js:26-75` — tabel template. Disimpan supaya nama alias dan URL
/// template tetap terdokumentasi di Rust walau render-nya belum ada.
pub(crate) const TEMPLATE: [(&str, &str); 13] = [
    (
        "bratanime",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1772989415819.jpeg",
    ),
    (
        "bratbahlil",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1772229450331.jpeg",
    ),
    (
        "bratpatrick",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1772794292812.jpeg",
    ),
    (
        "bratsquidward",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1772794955890.jpeg",
    ),
    (
        "bratwhite",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1771727655279.jpeg",
    ),
    (
        "bratchika",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1771527351581.jpeg",
    ),
    (
        "bratkobato",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1771728510134.jpeg",
    ),
    (
        "bratmenhera",
        "https://c.termai.cc/i145/qJsN.jpg",
    ),
    (
        "bratnezuko",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1772793912974.jpeg",
    ),
    (
        "bratqiqi",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1532869746.jpeg",
    ),
    (
        "bratruromiya",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1771827988894.jpeg",
    ),
    (
        "bratumaru",
        "https://raw.githubusercontent.com/kayzzaoshi-code/Uploader/main/file_1771524973616.jpeg",
    ),
    (
        "brathd",
        "https://aqul-brat.hf.space/",
    ),
];

/// URL template untuk satu alias — `None` kalau aliasnya tidak punya template.
pub(crate) fn template_untuk(command: &str) -> Option<&'static str> {
    TEMPLATE
        .iter()
        .find(|(nama, _)| *nama == command)
        .map(|(_, url)| *url)
}

/// `bratextra.js:176` — URL endpoint `brathd`.
pub(crate) fn url_brathd(text: &str) -> String {
    format!(
        "{ENDPOINT_BRATHD}?text={}",
        crate::tools::phisingataubukan::encode_uri_component(text)
    )
}

/// `bratextra.js:182` — `content-type` harus `image/*` dan buffer minimal 100
/// byte, kalau tidak lempar `"brathd gagal"`.
pub(crate) fn brathd_valid(content_type: &str, buf: &[u8]) -> bool {
    content_type.starts_with("image/") && buf.len() >= 100
}

#[plugin(
    name = "bratextra",
    alias = [
        "bratanime",
        "bratbahlil",
        "brathd",
        "bratpatrick",
        "bratsquidward",
        "bratwhite",
        "bratchika",
        "bratkobato",
        "bratmenhera",
        "bratnezuko",
        "bratqiqi",
        "bratruromiya",
        "bratumaru",
    ],
    category = "sticker",
    description = "Bikin brat versi custom",
    usage = ".bratanime <teks>",
    example = ".bratanime Halo",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bratextra(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    if text.is_empty() {
        return Reply::text(pakai(&ctx.prefix, &ctx.command));
    }

    let command = ctx.command.to_lowercase();

    if command == "brathd" {
        // `bratextra.js:175-191` — endpoint brat-HD; `validateStatus: () =>
        // true` jadi status 4xx/5xx tetap boleh dibaca bodinya.
        let Ok(res) = ctx.http.call(
            &HttpCall::get(url_brathd(text)).terima_semua_status(),
        ) else {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        };
        let content_type = res
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let buf = res.body.bytes().to_vec();
        if !brathd_valid(&content_type, &buf) {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
        return Reply::sticker(MediaSource::bytes(buf)).with_reaction("✅");
    }

    // `bratextra.js:193-196` — alias tanpa template dilempar ("Template tidak
    // ditemukan") dan masuk cabang `catch`.
    if template_untuk(&command).is_none() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    // Render template butuh canvas — lihat `ponytail:` modul. Cadangan: API
    // brat yang sama dengan plugin `brat`.
    let Ok(res) = ctx.http.get(&api_url(text)) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };
    if !(200..300).contains(&res.status) {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }
    let buf = res.body.bytes().to_vec();
    if is_blank(&buf) {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    Reply::sticker(MediaSource::bytes(buf)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Stub {
        status: u16,
        headers: Vec<(String, String)>,
        balasan: Option<Vec<u8>>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn ok(bytes: &[u8]) -> Arc<Stub> {
            Arc::new(Stub {
                status: 200,
                headers: vec![("content-type".into(), "image/png".into())],
                balasan: Some(bytes.to_vec()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn dengan_header(content_type: &str, bytes: Vec<u8>) -> Arc<Stub> {
            Arc::new(Stub {
                status: 200,
                headers: vec![("content-type".into(), content_type.into())],
                balasan: Some(bytes),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                status: 0,
                headers: vec![],
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
                    headers: self.headers.clone(),
                }),
                None => Err(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                }),
            }
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
            self.get(&req.url)
        }
    }

    fn png_palsu() -> Vec<u8> {
        let mut v = b"\x89PNG\r\n\x1a\n".to_vec();
        v.resize(200, 0);
        v
    }

    fn run(http: Arc<Stub>, command: &str, text: &str) -> Reply {
        let ctx = Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: command.to_string(),
            http,
            ..Ctx::default()
        };
        bratextra(ctx)
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
            pakai(".", "bratanime"),
            "⚠️ Harap masukkan teksnya!\nContoh: `.bratanime Halo semuanya`"
        );
        assert_eq!(
            pakai("!", "bratbahlil"),
            "⚠️ Harap masukkan teksnya!\nContoh: `!bratbahlil Halo semuanya`"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("bratextra").unwrap();
        assert_eq!(p.meta.name, "bratextra");
        assert_eq!(
            p.meta.alias,
            [
                "bratanime",
                "bratbahlil",
                "brathd",
                "bratpatrick",
                "bratsquidward",
                "bratwhite",
                "bratchika",
                "bratkobato",
                "bratmenhera",
                "bratnezuko",
                "bratqiqi",
                "bratruromiya",
                "bratumaru",
            ]
        );
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Bikin brat versi custom");
        assert_eq!(p.meta.usage, ".bratanime <teks>");
        assert_eq!(p.meta.example, ".bratanime Halo");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn semua_alias_terdaftar() {
        for a in [
            "bratanime",
            "bratbahlil",
            "brathd",
            "bratpatrick",
            "bratsquidward",
            "bratwhite",
            "bratchika",
            "bratkobato",
            "bratmenhera",
            "bratnezuko",
            "bratqiqi",
            "bratruromiya",
            "bratumaru",
        ] {
            // `bratanime` juga jadi NAMA plugin `animebrat` — di JS router
            // nama menang lebih dulu, dan `find` di Rust juga begitu.
            let ditemukan = oktz_plugin_api::find(a).unwrap();
            assert!(
                ditemukan.meta.name == "bratextra" || ditemukan.meta.name == "bratanime",
                "alias {a} menunjuk ke {}",
                ditemukan.meta.name
            );
        }
        assert_eq!(oktz_plugin_api::find("bratextra").unwrap().meta.name, "bratextra");
    }

    #[test]
    fn template_kosong_tidak_ada_template() {
        assert!(template_untuk("bratbahlil").is_some());
        assert!(template_untuk("brathd").is_some());
        assert!(template_untuk("bratxyz").is_none());
    }

    #[test]
    fn tanpa_teks_menampilkan_kartu_pakai_dengan_command_yang_dipakai() {
        let http = Stub::offline();
        let r = run(http.clone(), "bratbahlil", "");
        assert_eq!(teks(&r), pakai(".", "bratbahlil"));
        assert_eq!(reaksi(&r), None);
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn brathd_menggunakan_endpoint_js_dan_mengirim_sticker() {
        let png = png_palsu();
        let http = Stub::dengan_header("image/png", png.clone());
        let r = run(http.clone(), "brathd", "Halo");
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
            &["https://aqul-brat.hf.space/?text=Halo".to_string()]
        );
    }

    #[test]
    fn brathd_menolak_bukan_gambar_atau_buffer_kecil() {
        assert!(!brathd_valid("text/html", &png_palsu()));
        assert!(!brathd_valid("image/png", &[0u8; 10]));
        assert!(brathd_valid("image/png", &png_palsu()));
    }

    #[test]
    fn brathd_gagal_jadi_error_template() {
        let html = Stub::dengan_header("text/html", png_palsu());
        let r = run(html, "brathd", "Halo");
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "brathd"));
        assert_eq!(reaksi(&r), Some("☢"));
    }

    #[test]
    fn alias_tanpa_template_jatuh_ke_error_template() {
        let http = Stub::offline();
        let r = run(http, "bratxyz", "Halo");
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "bratxyz"));
        assert_eq!(reaksi(&r), Some("☢"));
    }

    #[test]
    fn template_alias_mengirim_sticker_dari_api_brat() {
        let png = png_palsu();
        let http = Stub::ok(&png);
        let r = run(http.clone(), "bratbahlil", "Halo");
        assert_eq!(reaksi(&r), Some("✅"));
        assert_eq!(
            http.seen.lock().unwrap().as_slice(),
            &["https://api.theresav.eu/api/maker/brat?text=Halo".to_string()]
        );
    }
}
