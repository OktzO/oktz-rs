//! `plugins/download/videy.js` — unduh video dari videy.co.
//!
//! Urutan backend mengikuti `src/lib/resolve.js`: backend lokal `videyShare`
//! (probe CDN, `src/scraper/videy.js:101-127`) dulu, lalu aggregator `neoxr`
//! sebagai cadangan (`src/capabilities/videy.js:25-63`). Key `neoxr` dibaca dari
//! runtime `Ctx`; key kosong membuat `aggregator.hit` melewati aggregator tanpa
//! request, seperti JS.
//!
//! `normalize` (`src/capabilities/videy.js:80-109`) membuka amplop
//! `{status, data:{url}}` dan menolak `status: false` serta URL kosong; bentuk
//! lokal `{id, ext, url}` lewat tanpa amplop.
//!
//! Video dikirim seperti `videy.js:72-80`: `sock.sendMedia(..., { type:
//! "video" })` → [`Reply::Video`], lalu reaksi ✅.
//!
//! Dua pesan khusus di `catch` sekarang terjangkau: key aggregator kosong
//! (`missingEnvNameOf` + `explainFailure`) dan respons yang gagal dinormalisasi
//! (`adaNormalisasiGagal`).

use oktz_plugin_api::{Ctx, HttpCall, MediaSource as Ms, Reply, plugin};
use serde_json::Value;

use crate::common::host_dari;

/// `videy.js:48-51` — kartu pakai. Baris contoh dibungkus satu pasang backtick,
/// sama seperti di JS.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🎬 *ᴠɪᴅᴇʏ ᴅᴏᴡɴʟᴏᴀᴅ*\n\n\
         > Masukkan URL videy.co\n\n\
         `Contoh: {prefix}videy https://videy.co/v?id=7ZH1ZRIF`"
    )
}

/// `videy.js:55` — host bukan Videy.
pub(crate) const BUKAN_HOST: &str = "❌ URL tidak valid. Gunakan link dari videy.co";

/// `videy.js:101` — `adaNormalisasiGagal(error)` benar.
pub(crate) const NORMALISASI_GAGAL: &str =
    "❌ Gagal mengambil video. Link tidak valid atau sudah expired.";

/// `explainFailure(new MissingApiKeyError("aggregator", envName), …)`
/// (`src/lib/stalker-fallback.js:29-33`). `command` sudah dirangkai jadi
/// `` `${prefix}videy` `` oleh pemanggil.
pub(crate) fn api_key_belum_diset(command: &str, env_name: &str) -> String {
    format!(
        "❌ *API Key belum diset!*\n\n\
         > Command `{command}` butuh `{env_name}` di `.env`\n\
         > Saat ini belum ada, jadi tidak bisa dipanggil."
    )
}

/// `hostVidey` (`videy.js:31-41`) tanpa crate URL: protokol harus http(s),
/// userinfo dan port dibuang, host berhenti di `/?#`, lalu dibandingkan per
/// label. Sub-domain ikut diterima karena halaman videy dilayani dari
/// `www.videy.co`.
pub(crate) fn host_videy(url: &str) -> bool {
    let Some(host) = host_dari(url) else {
        return false;
    };
    host == "videy.co" || host.ends_with(".videy.co")
}

/// Kelas kegagalan resolver yang dibedakan plugin `videy.js:82-104`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolveFailure {
    MissingApiKey,
    Normalization,
    Process,
}

/// `resolver.resolve("videy", { url })`: backend lokal dulu, lalu aggregator
/// `neoxr` (`src/capabilities/videy.js:117-129`). `aggregator.hit` melewati
/// aggregator tanpa request bila key kosong.
fn resolve_videy(
    http: &dyn oktz_plugin_api::HttpGet,
    url: &str,
    apikey: &str,
) -> Result<String, ResolveFailure> {
    // 1. `lewatCdnVidey` — `videyShare` mengembalikan `{id, ext, url}` rata.
    match oktz_scraper::videy::videy_share(http, url) {
        Ok(value) => match url_video(&value) {
            Some(video) => return Ok(video),
            None => {}
        },
        Err(_) => {}
    }

    if apikey.is_empty() {
        return Err(ResolveFailure::MissingApiKey);
    }

    // 2. `lewatNeoxr` — `aggregator.hit("neoxr", "/api/videy", ...)`.
    // Key masuk query dan header, seperti `src/lib/aggregator.js:117-139`.
    let target = format!(
        "https://api.neoxr.eu/api/videy?url={}&apikey={}",
        super::douyindl::query_encode(url),
        super::douyindl::query_encode(apikey)
    );
    let body = http
        .call(&HttpCall::get(target).header("apikey", apikey))
        .ok()
        .and_then(|response| response.json())
        .ok_or(ResolveFailure::Process)?;
    normalize_videy(&body).map_err(|_| ResolveFailure::Normalization)
}

/// `urlVideo` (`videy.js:97-99`) — `String(nilai ?? "").trim()`, lalu uji
/// `^https?:\/\//i`.
fn url_video(isi: &Value) -> Option<String> {
    let url = match isi.get("url") {
        Some(Value::String(s)) => s.clone(),
        Some(lain) => lain.to_string(),
        None => String::new(),
    };
    let url = url.trim().to_string();
    let kecil = url.to_lowercase();
    (kecil.starts_with("http://") || kecil.starts_with("https://")).then_some(url)
}

/// `normalize` (`videy.js:80-110`) — ampel aggregator dibuka di sini.
fn normalize_videy(raw: &Value) -> Result<String, ()> {
    // `raw.status === false` — bentuk lokal `{status: "error", msg}` dari
    // scraper tidak kena pemeriksaan ini, tapi URL-nya pasti kosong, jadi
    // jalurnya tetap berakhir `Err` lewat pemeriksaan URL.
    if raw.get("status") == Some(&Value::Bool(false)) {
        return Err(());
    }
    let isi = match raw.get("data") {
        Some(d) if d.is_object() => d,
        _ => raw,
    };
    url_video(isi).ok_or(())
}

/// Unduh byte media final dengan header probe CDN.
///
/// `src/scraper/videy.js:101-127` memakai UA + Referer untuk memprobe CDN.
/// `MediaSource::Url` tidak membawa header per-media, jadi URL final harus
/// ditarik byte di scope plugin; pipeline hanya akan memakai UA generik.
fn unduh_media_videy(
    http: &dyn oktz_plugin_api::HttpGet,
    url: &str,
) -> Result<Vec<u8>, ResolveFailure> {
    let response = http
        .call(
            &HttpCall::get(url)
                .header("user-agent", oktz_scraper::videy::UA)
                .header("referer", oktz_scraper::videy::REFERER)
                .header("accept", "video/mp4,video/quicktime,*/*"),
        )
        .map_err(|_| ResolveFailure::Process)?;
    if !(200..300).contains(&response.status) {
        return Err(ResolveFailure::Process);
    }
    let data = response.body.bytes();
    if data.is_empty() {
        return Err(ResolveFailure::Process);
    }
    Ok(data.to_vec())
}

#[plugin(
    name = "videy",
    alias = ["vdl", "videydownload", "videydl"],
    category = "download",
    description = "Download video dari videy.co",
    usage = ".videy <url>",
    example = ".videy https://videy.co/v?id=7ZH1ZRIF",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn videy(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();

    if url.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    if !host_videy(url) {
        return Reply::text(BUKAN_HOST);
    }

    match resolve_videy(ctx.http.as_ref(), url, &ctx.config().api_keys.neoxr) {
        // `videy.js:72-80` — `sendMedia` video, lalu reaksi ✅. Media final
        // diunduh byte lebih dulu agar header CDN Videy tidak hilang.
        Ok(url_video) => match unduh_media_videy(ctx.http.as_ref(), &url_video) {
            Ok(data) => Reply::video(Ms::bytes(data), "").with_reaction("✅"),
            Err(ResolveFailure::Process) => {
                Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
            }
            Err(ResolveFailure::MissingApiKey | ResolveFailure::Normalization) => {
                unreachable!("unduhan media tidak mengubah klasifikasi resolver")
            }
        },
        // `missingEnvNameOf` diperiksa lebih dulu, seperti `videy.js:89-93`.
        Err(ResolveFailure::MissingApiKey) => Reply::text(api_key_belum_diset(
            &format!("{}videy", ctx.prefix),
            "APIKEY_NEOXR",
        ))
        .with_reaction("❌"),
        // `adaNormalisasiGagal` memakai pesan link, bukan template operator.
        Err(ResolveFailure::Normalization) => Reply::text(NORMALISASI_GAGAL).with_reaction("❌"),
        Err(ResolveFailure::Process) => {
            Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpResponse, MediaSource};
    use std::sync::{Arc, Mutex};

    const SENDER: &str = "628111@s.whatsapp.net";

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "videy".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        (find("videy").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    /// Isi balasan sebenarnya: `Reaction { reply }` dibuka satu lapis.
    fn isi(r: &Reply) -> &Reply {
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = find("videy").unwrap();
        assert_eq!(p.meta.name, "videy");
        assert_eq!(p.meta.alias, ["vdl", "videydownload", "videydl"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video dari videy.co");
        assert_eq!(p.meta.usage, ".videy <url>");
        assert_eq!(p.meta.example, ".videy https://videy.co/v?id=7ZH1ZRIF");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["vdl", "videydownload", "videydl"] {
            assert_eq!(find(a).unwrap().meta.name, "videy", "alias {a}");
        }
    }

    #[test]
    fn a_missing_link_shows_the_card_without_reacting() {
        for text in ["", "   ", "\t"] {
            let r = run(text);
            assert_eq!(text_of(&r), help("."), "input {text:?}");
            assert_eq!(reaksi(&r), None, "input {text:?}");
        }
    }

    #[test]
    fn the_card_follows_the_prefix() {
        let ctx = Ctx {
            prefix: "!".into(),
            command: "videy".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            ..Default::default()
        };
        let r = (find("videy").unwrap().handler)(ctx);
        assert_eq!(text_of(&r), help("!"));
    }

    #[test]
    fn only_videy_hosts_pass_the_guard() {
        for good in [
            "https://videy.co/v?id=7ZH1ZRIF",
            "http://videy.co/v?id=x",
            "https://www.videy.co/v?id=x",
            "https://sub.videy.co/v?id=x",
            "https://VIDEY.CO/v?id=x",
            "  https://videy.co/v?id=x  ",
            "https://user:pw@videy.co:8443/v?x=1#z",
        ] {
            assert!(host_videy(good), "harus lolos: {good}");
        }
        for bad in [
            // Per label, bukan `includes`.
            "https://videy.co.evil.example/x",
            "https://notvidey.co/x",
            "https://xvidey.co/x",
            "https://example.com/?u=videy.co",
            "videy.co/v?id=x",
            "ftp://videy.co/x",
            "javascript:alert(1)",
            "",
        ] {
            assert!(!host_videy(bad), "harus ditolak: {bad}");
        }
    }

    #[test]
    fn a_foreign_host_is_rejected_without_reacting() {
        for text in [
            "https://example.com/x",
            "https://videy.co.evil.example/x",
            "halo dunia",
        ] {
            let r = run(text);
            assert_eq!(text_of(&r), BUKAN_HOST, "input {text}");
            assert_eq!(reaksi(&r), None, "input {text}");
        }
    }

    #[test]
    fn aggregator_key_kosong_memberi_pesan_key() {
        // `aggregator.hit` melewati `neoxr` tanpa request saat key kosong;
        // `missingEnvNameOf` menemukan `APIKEY_NEOXR` dari alasan kegagalan.
        let r = run("https://videy.co/v?id=7ZH1ZRIF");
        assert_eq!(text_of(&r), api_key_belum_diset(".videy", "APIKEY_NEOXR"));
        assert_eq!(reaksi(&r), Some("❌"));
    }

    /// Stub `HttpGet` yang mencatat permintaan dan membalas JSON per-backend.
    #[derive(Debug)]
    struct Stub {
        per_backend: Vec<(&'static str, Option<Value>)>,
        seen: Mutex<Vec<HttpCall>>,
        gagal_probe: bool,
    }

    impl Stub {
        fn per_backend(per_backend: Vec<(&'static str, Option<Value>)>) -> Arc<Stub> {
            Arc::new(Stub {
                per_backend,
                seen: Mutex::new(Vec::new()),
                gagal_probe: false,
            })
        }

        /// Stub yang membuat probe Range gagal tetapi mengizinkan GET penuh.
        fn per_backend_gagal_probe(per_backend: Vec<(&'static str, Option<Value>)>) -> Arc<Stub> {
            Arc::new(Stub {
                per_backend,
                seen: Mutex::new(Vec::new()),
                gagal_probe: true,
            })
        }
    }

    impl oktz_plugin_api::HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "stub tidak melayani POST".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "stub tidak melayani POST".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, oktz_plugin_api::HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "stub tidak melayani multipart".into(),
            })
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
            let (_, nilai) = self
                .per_backend
                .iter()
                .find(|(penanda, _)| req.url.contains(penanda))
                .ok_or(HttpError {
                    message: format!("URL tak dikenal stub: {}", req.url),
                })?;
            if self.gagal_probe
                && req
                    .headers
                    .iter()
                    .any(|(name, value)| name.eq_ignore_ascii_case("range") && value == "bytes=0-0")
            {
                return Err(HttpError {
                    message: "probe offline".into(),
                });
            }
            let teks = nilai
                .clone()
                .ok_or(HttpError {
                    message: "offline".into(),
                })?
                .to_string();
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: std::sync::Arc::from(teks.into_bytes()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    /// Stub khusus unduhan media Videy: probe Range dan GET penuh memakai
    /// URL sama, tetapi body dibedakan lewat header `range`.
    #[derive(Debug, Default)]
    struct MediaStub {
        seen: Mutex<Vec<HttpCall>>,
    }

    impl oktz_plugin_api::HttpGet for MediaStub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "stub tidak melayani POST".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "stub tidak melayani POST".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "stub tidak melayani multipart".into(),
            })
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
            let probe = req
                .headers
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("range") && value == "bytes=0-0");
            let data = if probe {
                b"PROBE".as_slice()
            } else {
                b"VIDEO".as_slice()
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: std::sync::Arc::from(data),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    fn run_dengan(http: Arc<dyn oktz_plugin_api::HttpGet>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "videy".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (find("videy").unwrap().handler)(ctx)
    }

    fn run_dengan_key(http: Arc<Stub>, text: &str, key: &str) -> Reply {
        let mut ctx = Ctx {
            prefix: ".".into(),
            command: "videy".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        let mut config = ctx.config.as_ref().clone();
        config.api_keys.neoxr = key.to_string();
        ctx.config = Arc::new(config);
        (find("videy").unwrap().handler)(ctx)
    }

    #[test]
    fn the_local_cdn_backend_sends_the_video() {
        // `videyShare` mengembalikan `{id, ext, url}` rata; probe GET Range
        // ke CDN dibalas 200 oleh stub (isi body tidak dibaca).
        let http = Stub::per_backend(vec![("cdn.videy.co", Some(serde_json::json!({})))]);
        let r = run_dengan(http, "https://videy.co/v?id=7ZH1ZRIF");
        match isi(&r) {
            Reply::Video { media, caption, .. } => {
                let m = media.as_ref().expect("video harus punya sumber");
                assert_eq!(m, &MediaSource::bytes(b"{}".to_vec()));
                assert_eq!(caption, "");
            }
            lain => panic!("harus video, dapat {lain:?}"),
        }
        assert_eq!(reaksi(&r), Some("✅"));
    }

    #[test]
    fn final_cdn_media_diunduh_dengan_header_dan_dikirim_sebagai_byte() {
        // `src/scraper/videy.js:101-127` memakai UA + Referer untuk probe CDN.
        // `MediaSource::Url` tidak membawa header, jadi media final harus
        // diunduh byte di scope plugin sebelum dikirim.
        let http = Arc::new(MediaStub::default());
        let r = run_dengan(http.clone(), "https://videy.co/v?id=7ZH1ZRIF");
        match isi(&r) {
            Reply::Video { media, .. } => {
                assert_eq!(media, &Some(MediaSource::bytes(b"VIDEO".to_vec())))
            }
            lain => panic!("harus video, dapat {lain:?}"),
        }
        assert_eq!(reaksi(&r), Some("✅"));

        let seen = http.seen.lock().unwrap();
        assert_eq!(seen.len(), 2, "probe lalu unduhan media penuh");
        for request in seen.iter() {
            let cari = |nama: &str| {
                request
                    .headers
                    .iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case(nama))
                    .map(|(_, value)| value.as_str())
            };
            assert_eq!(cari("user-agent"), Some("Mozilla/5.0 (Linux; Android 10)"));
            assert_eq!(cari("referer"), Some("https://videy.co/"));
        }
    }

    #[test]
    fn the_neoxr_fallback_opens_the_envelope() {
        // Backend lokal gagal (transport), neoxr menjawab ampel
        // `{status: 200, data: {url}}` — `normalize` membuka `data`.
        let http = Stub::per_backend_gagal_probe(vec![
            (
                "api.neoxr.eu",
                Some(serde_json::json!({
                    "status": 200,
                    "data": { "url": "https://cdn.videy.co/abc123.mp4" }
                })),
            ),
            ("cdn.videy.co", Some(serde_json::json!({}))),
        ]);
        let r = run_dengan_key(http, "https://videy.co/v?id=7ZH1ZRIF", "KUNCI-RUNTIME");
        assert!(matches!(isi(&r), Reply::Video { .. }), "{r:?}");
    }

    #[test]
    fn aggregator_neoxr_menggunakan_key_runtime_dan_query_seperti_js() {
        let http = Stub::per_backend_gagal_probe(vec![
            (
                "api.neoxr.eu",
                Some(serde_json::json!({
                    "status": true,
                    "data": { "url": "https://cdn.videy.co/abc123.mp4" }
                })),
            ),
            ("cdn.videy.co", Some(serde_json::json!({}))),
        ]);
        let r = run_dengan_key(
            http.clone(),
            "https://videy.co/v?id=7ZH1ZRIF",
            "KUNCI-RUNTIME",
        );
        assert!(matches!(isi(&r), Reply::Video { .. }), "{r:?}");
        let seen = http.seen.lock().unwrap();
        let aggregator = seen
            .iter()
            .find(|request| request.url.contains("api.neoxr.eu"))
            .expect("request aggregator");
        assert_eq!(
            aggregator.url,
            "https://api.neoxr.eu/api/videy\
             ?url=https%3A%2F%2Fvidey.co%2Fv%3Fid%3D7ZH1ZRIF&apikey=KUNCI-RUNTIME"
        );
        assert!(
            aggregator.headers.iter().any(
                |(name, value)| name.eq_ignore_ascii_case("apikey") && value == "KUNCI-RUNTIME"
            )
        );
    }

    #[test]
    fn key_kosong_melewati_aggregator_dan_memberi_pesan_key() {
        let http = Stub::per_backend(vec![(
            "api.neoxr.eu",
            Some(serde_json::json!({
                "status": true,
                "data": { "url": "https://cdn.videy.co/abc123.mp4" }
            })),
        )]);
        let r = run_dengan_key(http.clone(), "https://videy.co/v?id=7ZH1ZRIF", "");
        assert_eq!(text_of(&r), api_key_belum_diset(".videy", "APIKEY_NEOXR"));
        assert_eq!(reaksi(&r), Some("❌"));
        assert!(
            http.seen
                .lock()
                .unwrap()
                .iter()
                .all(|request| !request.url.contains("api.neoxr.eu")),
            "aggregator tidak boleh dihubungi tanpa key"
        );
    }

    #[test]
    fn the_neoxr_status_false_is_a_failure() {
        let http = Stub::per_backend(vec![(
            "api.neoxr.eu",
            Some(serde_json::json!({
                "status": false,
                "data": { "url": "https://cdn.videy.co/abc123.mp4" }
            })),
        )]);
        let r = run_dengan_key(http, "https://videy.co/v?id=7ZH1ZRIF", "KUNCI-RUNTIME");
        // `status: false` ditolak `normalize` (`src/capabilities/videy.js:88-90`),
        // sehingga plugin memakai pesan link invalid, bukan template proses.
        assert_eq!(text_of(&r), NORMALISASI_GAGAL);
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn the_two_unreachable_catch_messages_are_still_byte_exact() {
        assert_eq!(
            NORMALISASI_GAGAL,
            "❌ Gagal mengambil video. Link tidak valid atau sudah expired."
        );
        assert_eq!(
            api_key_belum_diset(".videy", "APIKEY_NEOXR"),
            "❌ *API Key belum diset!*\n\n\
             > Command `.videy` butuh `APIKEY_NEOXR` di `.env`\n\
             > Saat ini belum ada, jadi tidak bisa dipanggil."
        );
    }

    #[test]
    fn the_plugin_only_reads_and_never_mutates() {
        assert!(run("https://videy.co/v?id=x").mutations().is_empty());
    }

    #[test]
    #[ignore = "uji jaringan Videy; set OKTZ_TEST_VIDEY_URL dan APIKEY_NEOXR bila perlu"]
    fn live_videy_mengambil_video() {
        // Uji live padanan `plugins/download/videy.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_VIDEY_URL");
        let (http, ctx) = live_ctx("videy", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("videy").unwrap().handler)(ctx);
        assert_media(&reply, &["video"], &http);
    }
}
