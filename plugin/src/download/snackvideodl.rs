//! `plugins/download/snackvideodl.js` — unduh video SnackVideo.
//!
//! Scraper `snackvideo` memakai kontrak resmi `btch-downloader@6.3.6` melalui
//! `oktz_scraper::btch`, termasuk envelope `result` dan penanganan error
//! transport seperti SDK Node.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use serde_json::Value;

/// `snackvideodl.js:31` — `/snackvideo\.com/i`, tidak dikunci ke host.
pub(crate) fn is_snackvideo_url(url: &str) -> bool {
    url.to_lowercase().contains("snackvideo.com")
}

/// `snackvideodl.js:23-28` — kartu pakai, delimiter markdown-nya backtick.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}svdl <url>`\n\n\
         > Contoh:\n\
         > `{prefix}svdl https://www.snackvideo.com/@xxx/video/xxx`"
    )
}

/// `snackvideodl.js:32` — URL bukan SnackVideo.
pub(crate) const BAD_URL: &str = "❌ URL tidak valid. Gunakan link SnackVideo.";

/// `snackvideodl.js:41` — backend menjawab tanpa `status`/`result.videoUrl`.
pub(crate) const NO_VIDEO: &str = "❌ Gagal mengambil video. Coba link lain.";

/// `snackvideodl.js:40` — `data?.status && data?.result?.videoUrl`.
pub(crate) fn accepted(data: &Value) -> bool {
    data.get("status").and_then(Value::as_bool) == Some(true)
        && data
            .get("result")
            .and_then(|r| r.get("videoUrl"))
            .and_then(Value::as_str)
            .is_some_and(|s| !s.is_empty())
}

/// `snackvideodl.js:46` — `result.videoUrl`, tapi hanya dibaca setelah guard
/// `status`/`videoUrl` lulus; di luar itu plugin tidak pernah menyentuh URL.
pub(crate) fn video_url(data: &Value) -> String {
    if !accepted(data) {
        return String::new();
    }
    data.get("result")
        .and_then(|r| r.get("videoUrl"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[plugin(
    name = "snackvideodl",
    alias = ["svdl", "snackvideo"],
    category = "download",
    description = "Download video SnackVideo",
    usage = ".svdl <url>",
    example = ".svdl https://www.snackvideo.com/@xxx/video/xxx",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn snackvideodl(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();
    if url.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    if !is_snackvideo_url(url) {
        return Reply::text(BAD_URL);
    }

    // `snackvideodl.js:38` memakai SDK Node yang menangkap error transport dan
    // mengembalikan envelope `status: false`.
    let data = oktz_scraper::btch::snackvideo(ctx.http.as_ref(), url);

    if !accepted(&data) {
        return Reply::text(NO_VIDEO).with_reaction("🕕");
    }

    // `snackvideodl.js:46-52` — `sendMedia` video tanpa caption; reaksi ⏱️
    // adalah satu-satunya reaksi yang dikirim JS.
    Reply::video(MediaSource::url(video_url(&data)), "").with_reaction("🕕")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        body: Result<String, HttpError>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn json(value: Value) -> Arc<Stub> {
            Arc::new(Stub {
                body: Ok(value.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                body: Err(HttpError {
                    message: "offline".into(),
                }),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let body = self.body.clone()?;
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(body.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }

        fn post_json(&self, _url: &str, _body: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("hanya GET")
        }

        fn post_form(&self, _url: &str, _form: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("hanya GET")
        }

        fn post_multipart(
            &self,
            _url: &str,
            _parts: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("hanya GET")
        }
    }

    fn run(text: &str) -> Reply {
        run_with_http(text, Stub::offline())
    }

    fn run_with_http(text: &str, http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "svdl".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: vec![text.into()],
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("snackvideodl").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn isi(r: &Reply) -> &Reply {
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("snackvideodl").unwrap();
        assert_eq!(p.meta.name, "snackvideodl");
        assert_eq!(p.meta.alias, ["svdl", "snackvideo"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video SnackVideo");
        assert_eq!(p.meta.usage, ".svdl <url>");
        assert_eq!(
            p.meta.example,
            ".svdl https://www.snackvideo.com/@xxx/video/xxx"
        );
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["svdl", "snackvideo"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "snackvideodl");
        }
    }

    #[test]
    fn an_empty_argument_shows_the_usage_card() {
        for text in ["", "   "] {
            assert_eq!(text_of(&run(text)), help("."), "input {text:?}");
        }
    }

    #[test]
    fn a_url_without_the_domain_is_rejected() {
        assert_eq!(text_of(&run("https://youtube.com/watch?v=x")), BAD_URL);
    }

    #[test]
    fn the_unanchored_regex_quirk_of_the_js_is_kept() {
        assert!(is_snackvideo_url("https://snackvideo.com.evil.example/x"));
        assert!(is_snackvideo_url("https://notsnackvideo.com/x"));
        assert!(is_snackvideo_url("https://www.SnackVideo.com/@a/video/1"));
    }

    #[test]
    fn a_response_without_video_url_is_refused() {
        for data in [
            serde_json::json!({ "status": false, "result": { "videoUrl": "https://cdn/v.mp4" } }),
            serde_json::json!({ "status": true }),
            serde_json::json!({ "status": true, "result": {} }),
            serde_json::json!({ "status": true, "result": { "videoUrl": "" } }),
            serde_json::json!({ "status": true, "result": { "videoUrl": null } }),
            serde_json::json!({}),
        ] {
            assert!(!accepted(&data), "{data} seharusnya ditolak");
            assert_eq!(video_url(&data), "", "{data}");
        }
    }

    #[test]
    fn scraper_btch_membungkus_data_snackvideo_dalam_result() {
        let http = Stub::json(serde_json::json!({
            "videoUrl": "https://cdn.example/video.mp4"
        }));
        let r = run_with_http("https://www.snackvideo.com/@a/video/1", http.clone());

        match isi(&r) {
            Reply::Video { media, .. } => {
                assert_eq!(
                    media.as_ref().and_then(|m| match m {
                        MediaSource::Url { url, .. } => Some(url.as_str()),
                        _ => None,
                    }),
                    Some("https://cdn.example/video.mp4")
                );
            }
            lain => panic!("harus video, dapat {lain:?}"),
        }
        assert_eq!(
            http.seen.lock().unwrap().as_slice(),
            [
                "https://backend1.tioo.eu.org/snackvideo?url=https%3A%2F%2Fwww.snackvideo.com%2F%40a%2Fvideo%2F1"
            ]
        );
    }

    #[test]
    fn kegagalan_transport_btch_menghasilkan_branch_gagal_mengambil_video() {
        let r = run("https://www.snackvideo.com/@a/video/1");
        assert_eq!(text_of(&r), NO_VIDEO);
        assert!(
            matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "🕕"),
            "reaksi jam harus tetap ada"
        );
    }

    #[test]
    #[ignore = "uji jaringan SnackVideo; set OKTZ_TEST_SNACKVIDEO_URL"]
    fn live_snackvideodl_mengambil_video() {
        // Uji live padanan `plugins/download/snackvideodl.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_SNACKVIDEO_URL");
        let (http, ctx) = live_ctx("snackvideodl", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("snackvideodl").unwrap().handler)(ctx);
        assert_media(&reply, &["video"], &http);
    }
}
