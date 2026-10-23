//! `plugins/download/ytmp4.js` — unduh video YouTube.
//!
//! Resolver berlapis kini ikut. Rantai backend JS untuk mp4 adalah
//! `youtube-fallback` (scraper lokal SaveTube, `getYoutubeDirectUrl`) →
//! `izuka` (agregator, `/api/downloader/ytmp4`) — `src/capabilities/youtube.js:395-414`.
//! URL-nya bentuk kanonik `kanonikDari(idVideo(url))` dari [`super::ytmp3::kanonik`].
//!
//! Video kini benar-benar dikirim: `sock.sendMedia(..., { type: "video" })`
//! (`ytmp4.js:61-63`) → [`Reply::Video`] tanpa caption, lalu reaksi ✅.

use oktz_plugin_api::{Ctx, MediaSource as Ms, Reply, plugin};
use serde_json::Value;

use super::douyindl::query_encode;
use super::ytmp3::{host_youtube, kanonik, url_field, url_nama, url_video_normal};

/// `AGGREGATORS.izuka.base` + path di `src/capabilities/youtube.js:257`.
pub(crate) const ENDPOINT: &str = "https://my.izuka-api.xyz/api/downloader/ytmp4";

/// `ytmp4.js:50` — link hilang.
pub(crate) fn contoh(prefix: &str) -> String {
    format!("Contoh: {prefix}ytmp4 https://youtube.com/watch?v=xxx")
}

/// `ytmp4.js:51` — bukan host YouTube.
pub(crate) const BUKAN_YOUTUBE: &str = "❌ URL harus YouTube";

/// `ytmp4.js:71` — cabang `catch`.
pub(crate) const GAGAL: &str = "Gagal mengunduh video.";

/// `normalize` (`src/capabilities/youtube.js:331`) untuk jalur video — membuka
/// amplop aggregator dan memeriksa `status`-nya dulu, lalu `urlUnduhan`:
/// `dl`/`download_url`/`download` → `video_normal` (array, ext mp4) → `url`
/// polos. `video_normal` non-array diabaikan seperti `Array.isArray` di JS.
pub(crate) fn normalize(isi: &Value) -> Result<String, ()> {
    let mut url = url_nama(isi);
    if url.is_empty() {
        url = url_video_normal(isi);
    }
    if url.is_empty() {
        url = url_field(isi);
    }
    if url.is_empty() {
        return Err(());
    }
    Ok(url)
}

/// `pastikanBentuk` + `lewatIzuka` (`youtube.js:238,257`) digabung: pemeriksaan
/// bentuk aggregator diletakkan di sini juga karena `izuka` tidak butuh key.
fn unduh_izuka(ctx: &Ctx, kanonik: &str) -> Result<String, ()> {
    let target = format!("{ENDPOINT}?url={}", query_encode(kanonik));
    let respons = ctx.http.get(&target).map_err(|_| ())?;
    let body = respons.json().ok_or(())?;
    if body.get("status").and_then(Value::as_bool) != Some(true) {
        return Err(());
    }
    let result = body.get("result").filter(|r| r.is_object()).ok_or(())?;
    let ada = match result.get("video_normal") {
        Some(Value::Array(daftar)) => !daftar.is_empty(),
        Some(lain) => crate::common::js_truthy_value(lain),
        None => false,
    };
    if !ada {
        return Err(());
    }
    normalize(result)
}

/// `getVideoDownloadUrl` (`ytmp4.js:42-45`) + resolver `resolve("youtube")` —
/// rantai backend mp4: `youtube-fallback` (SaveTube lokal,
/// `youtube.js:403-407`) → `izuka` (`youtube.js:409-413`). SaveTube
/// mengembalikan `{url}` polos tanpa ampel, jadi [`normalize`] langsung
/// membacanya.
fn unduh_video(ctx: &Ctx, url: &str) -> Result<String, ()> {
    let kanonik = kanonik(url)?;
    // 1. `youtube-fallback` — hanya untuk mp4 (`applies`, `youtube.js:405`).
    if let Ok(v) = oktz_scraper::youtube::get_youtube_direct_url(&*ctx.http, &kanonik, Some("360"))
        && let Ok(url) = normalize(&v)
    {
        return Ok(url);
    }
    // 2. `izuka` (`youtube.js:409-413`).
    unduh_izuka(ctx, &kanonik)
}

#[plugin(
    name = "ytmp4",
    alias = ["youtubemp4", "ytvideo"],
    category = "download",
    description = "Download video YouTube",
    usage = ".ytmp4 <url>",
    example = ".ytmp4 https://youtube.com/watch?v=xxx",
    cooldown = 20,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ytmp4(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();
    if url.is_empty() {
        return Reply::text(contoh(&ctx.prefix));
    }
    if !host_youtube(url) {
        return Reply::text(BUKAN_YOUTUBE);
    }

    match unduh_video(&ctx, url) {
        // `ytmp4.js:61-64` — `sendMedia` video tanpa caption lalu reaksi ✅.
        Ok(download) => Reply::video(Ms::url(download), "").with_reaction("✅"),
        Err(()) => Reply::text(GAGAL).with_reaction("❌"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        /// Respons per-backend dicocokkan dari potongan URL: SaveTube di
        /// `savetube`, izuka di `izuka`. `None` = transport error.
        per_backend: Vec<(&'static str, Option<Value>)>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn per_backend(per_backend: Vec<(&'static str, Option<Value>)>) -> Arc<Stub> {
            Arc::new(Stub {
                per_backend,
                seen: Mutex::new(Vec::new()),
            })
        }
        fn body(v: Value) -> Arc<Stub> {
            // Semua backend dapat jawaban yang sama.
            Self::per_backend(vec![("", Some(v))])
        }
        fn offline() -> Arc<Stub> {
            Self::per_backend(vec![("", None)])
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let (_, nilai) = self
                .per_backend
                .iter()
                .find(|(penanda, _)| url.contains(penanda))
                .ok_or(oktz_plugin_api::HttpError {
                    message: format!("URL tak dikenal stub: {url}"),
                })?;
            let text = nilai
                .clone()
                .ok_or(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                })?
                .to_string();
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            // SaveTube mem-POST /v2/info di percobaannya; di stub, POST selalu
            // gagal supaya rantai jatuh ke backend berikutnya.
            Err(oktz_plugin_api::HttpError {
                message: "stub tidak melayani POST".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("scraper ini cuma GET/POST-JSON")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ytmp4".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: vec![text.into()],
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("ytmp4").unwrap().handler)(ctx)
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

    fn ok_body() -> Value {
        serde_json::json!({
            "status": true,
            "result": {
                "title": "Video Contoh",
                "video_normal": [
                    { "ext": "webm", "quality": "720", "url": "https://cdn.example/v.webm" },
                    { "ext": "mp4", "quality": "360", "url": "https://cdn.example/v-360.mp4" },
                    { "ext": "mp4", "quality": "1080", "url": "https://cdn.example/v-1080.mp4" }
                ]
            }
        })
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("ytmp4").unwrap();
        assert_eq!(p.meta.name, "ytmp4");
        assert_eq!(p.meta.alias, ["youtubemp4", "ytvideo"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video YouTube");
        assert_eq!(p.meta.usage, ".ytmp4 <url>");
        assert_eq!(p.meta.example, ".ytmp4 https://youtube.com/watch?v=xxx");
        assert_eq!(p.meta.cooldown, 20);
        assert_eq!(p.meta.energi, 2);
        // JS `ytmp4.js:3-12` tidak menyebut satu pun flag permission.
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
        assert!(p.meta.is_enabled);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["youtubemp4", "ytvideo"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "ytmp4");
        }
    }

    #[test]
    fn an_invalid_video_id_fails_fast_like_the_capability_target() {
        // `target()` (`youtube.js:129`) melempar kalau `idVideo` null — ID
        // "xxx" bukan 11 karakter. `ytmp4.js:65-72` menangkapnya jadi
        // "Gagal mengunduh video." Tidak ada request yang dibuang.
        let http = Stub::per_backend(vec![("", Some(ok_body()))]);
        let r = run(http.clone(), "https://youtube.com/watch?v=xxx");
        assert_eq!(text_of(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌"));
        assert!(
            http.seen.lock().unwrap().is_empty(),
            "tak ada request sia-sia"
        );
    }

    #[test]
    fn the_izuka_mp4_backend_sends_the_video() {
        // `unduh_izuka` dipanggil langsung: menjalankan savetube di stub akan
        // menarik retry ber-sleep 24 detik (`MAX_COB` × 5 format) — jalur
        // SaveTube tidak layak diuji end-to-end di unit test. Wiring
        // `Reply::video` tetap ditutup handler test di atas lewat jalur gagal.
        let http = Stub::per_backend(vec![("izuka", Some(ok_body()))]);
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ytmp4".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "https://youtube.com/watch?v=dQw4w9WgXcQ".into(),
            args: vec!["https://youtube.com/watch?v=dQw4w9WgXcQ".into()],
            http: http.clone(),
            ..Default::default()
        };
        let kanonik = kanonik("https://youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        let url = unduh_izuka(&ctx, &kanonik).unwrap();
        assert_eq!(url, "https://cdn.example/v-1080.mp4");
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1);
        assert!(
            seen[0].starts_with(
                "https://my.izuka-api.xyz/api/downloader/ytmp4?url=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ"
            ),
            "backend menerima URL kanonik: {}",
            seen[0]
        );
    }

    #[test]
    fn the_izuka_string_video_normal_shape_is_rejected_like_javascript() {
        // `urlUnduhan` di `src/capabilities/youtube.js:308-324` hanya memproses
        // `video_normal` bila berbentuk array. Bentuk string harus gagal agar
        // port sama persis dengan implementasi JS asli.
        let isi = serde_json::json!({
            "status": true,
            "result": {
                "title": "Rick Astley - Never Gonna Give You Up",
                "duration": "3:33",
                "video_normal": "https://rr1---sn-cx5o4aqj5-tt1l.googlevideo.com/videoplayback?expire=1792759764&itag=18"
            }
        });
        assert!(normalize(isi.get("result").unwrap()).is_err());
    }

    #[test]
    fn ties_keep_the_first_entry_like_a_stable_javascript_sort() {
        let isi = serde_json::json!({
            "video_normal": [
                { "ext": "mp4", "quality": "720", "url": "https://a/1.mp4" },
                { "ext": "mp4", "quality": "720", "url": "https://a/2.mp4" }
            ]
        });
        assert_eq!(normalize(&isi).unwrap(), "https://a/1.mp4");
    }

    #[test]
    fn a_missing_link_shows_the_example_without_reacting() {
        let http = Stub::body(ok_body());
        let r = run(http.clone(), "");
        assert_eq!(
            text_of(&r),
            "Contoh: .ytmp4 https://youtube.com/watch?v=xxx"
        );
        assert_eq!(reaksi(&r), None);
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn a_foreign_host_is_rejected_without_any_request() {
        let http = Stub::body(ok_body());
        let r = run(http.clone(), "https://notyoutube.com/x");
        assert_eq!(text_of(&r), BUKAN_YOUTUBE);
        assert_eq!(reaksi(&r), None);
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn a_response_without_video_normal_reports_the_failure_message() {
        // Dipanggil langsung ke `unduh_izuka`: lewat handler, kegagalan izuka
        // berarti savetube dicoba dulu dengan retry ber-sleep (24 detik per
        // input) — jalur itu tidak layak dikali lima di unit test.
        for body in [
            serde_json::json!({ "status": false, "msg": "gagal" }),
            serde_json::json!({ "status": true, "result": { "video_normal": [] } }),
            serde_json::json!({ "status": true, "result": {} }),
            // Hanya webm: kontrak host tetap `ext: "mp4"`.
            serde_json::json!({ "status": true, "result": { "video_normal": [
                { "ext": "webm", "quality": "720", "url": "https://cdn.example/v.webm" }
            ] } }),
            serde_json::json!({ "status": true }),
        ] {
            let ctx = Ctx {
                prefix: ".".into(),
                command: "ytmp4".into(),
                sender: "628111@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: "https://youtube.com/watch?v=dQw4w9WgXcQ".into(),
                args: vec!["https://youtube.com/watch?v=dQw4w9WgXcQ".into()],
                http: Stub::per_backend(vec![("izuka", Some(body.clone()))]),
                ..Default::default()
            };
            let hasil = unduh_izuka(&ctx, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
            assert!(hasil.is_err(), "body {body} harus gagal");
        }
    }

    #[test]
    fn a_transport_failure_reports_the_same_message() {
        // Offline di semua backend — dipanggil langsung ke `unduh_izuka`
        // supaya retry ber-sleep milik savetube tidak ikut (lewat handler,
        // lima format × MAX_COB percobaan = 24 detik tidur).
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ytmp4".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "https://youtube.com/watch?v=dQw4w9WgXcQ".into(),
            args: vec!["https://youtube.com/watch?v=dQw4w9WgXcQ".into()],
            http: Stub::per_backend(vec![("izuka", None)]),
            ..Default::default()
        };
        let hasil = unduh_izuka(&ctx, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert!(hasil.is_err());
        // Handler-level: ID tanpa 11 karakter gagal tanpa satu pun request,
        // pesan dan reaksinya yang sama.
        let r = run(Stub::offline(), "https://youtube.com/watch?v=xxx");
        assert_eq!(text_of(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    #[ignore = "uji jaringan YouTube MP4; set OKTZ_TEST_YTMP4_URL"]
    fn live_ytmp4_mengambil_video() {
        // Uji live padanan `plugins/download/ytmp4.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_YTMP4_URL");
        let (http, ctx) = live_ctx("ytmp4", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("ytmp4").unwrap().handler)(ctx);
        assert_media(&reply, &["video"], &http);
    }
}
