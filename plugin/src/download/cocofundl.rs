//! `plugins/download/cocofundl.js` — unduh video CocoFun.
//!
//! Scraper `cocofun` memakai kontrak resmi `btch-downloader@6.3.6` melalui
//! `oktz_scraper::btch`, termasuk envelope `result` dan penanganan error
//! transport seperti SDK Node.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

/// `cocofundl.js:31` — `/cocofun\.com/i`, tidak dikunci ke host.
pub(crate) fn is_cocofun_url(url: &str) -> bool {
    url.to_lowercase().contains("cocofun.com")
}

/// `cocofundl.js:23-28` — kartu pakai, delimiter markdown-nya backtick.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}cfdl <url>`\n\n\
         > Contoh:\n\
         > `{prefix}cfdl https://www.cocofun.com/share/post/xxx`"
    )
}

/// `cocofundl.js:32` — URL bukan CocoFun.
pub(crate) const BAD_URL: &str = "❌ URL tidak valid. Gunakan link CocoFun.";

/// `cocofundl.js:41` — backend menjawab tanpa `status`/`result`.
pub(crate) const NO_RESULT: &str = "❌ Gagal mengambil video. Coba link lain.";

/// `cocofundl.js:48` — `result` ada tapi kedua field URL kosong.
pub(crate) const NO_VIDEO: &str = "❌ Video tidak ditemukan.";

/// `cocofundl.js:40` — `data?.status` harus `true` dan `data?.result` harus ada.
pub(crate) fn accepted(data: &Value) -> bool {
    data.get("status").and_then(Value::as_bool) == Some(true)
        && data.get("result").is_some_and(|r| !r.is_null())
}

/// `cocofundl.js:45` — `result.no_watermark || result.watermark`.
pub(crate) fn video_url(data: &Value) -> String {
    let result = data.get("result").unwrap_or(&Value::Null);
    result
        .get("no_watermark")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .or_else(|| result.get("watermark").and_then(Value::as_str))
        .unwrap_or("")
        .to_string()
}

#[plugin(
    name = "cocofundl",
    alias = ["cfdl", "cocofun"],
    category = "download",
    description = "Download video CocoFun",
    usage = ".cfdl <url>",
    example = ".cfdl https://www.cocofun.com/share/post/xxx",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cocofundl(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();
    if url.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    if !is_cocofun_url(url) {
        return Reply::text(BAD_URL);
    }

    // `cocofundl.js:38` memakai SDK Node yang menangkap error transport dan
    // mengembalikan envelope `status: false`.
    let data = oktz_scraper::btch::cocofun(ctx.http.as_ref(), url);

    if !accepted(&data) {
        return Reply::text(NO_RESULT).with_reaction("🕕");
    }

    let video = video_url(&data);
    if video.is_empty() {
        return Reply::text(NO_VIDEO).with_reaction("🕕");
    }

    // `cocofundl.js:51-57` — `sendMedia` video tanpa caption; reaksi ⏱️ adalah
    // satu-satunya reaksi yang dikirim JS.
    Reply::video(MediaSource::url(video), "").with_reaction("🕕")
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
            command: "cfdl".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: vec![text.into()],
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("cocofundl").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("cocofundl").unwrap();
        assert_eq!(p.meta.name, "cocofundl");
        assert_eq!(p.meta.alias, ["cfdl", "cocofun"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video CocoFun");
        assert_eq!(p.meta.usage, ".cfdl <url>");
        assert_eq!(
            p.meta.example,
            ".cfdl https://www.cocofun.com/share/post/xxx"
        );
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["cfdl", "cocofun"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "cocofundl");
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
        assert!(is_cocofun_url("https://cocofun.com.evil.example/x"));
        assert!(is_cocofun_url("https://notcocofun.com/x"));
        assert!(is_cocofun_url("https://www.CocoFun.com/share/post/1"));
    }

    #[test]
    fn the_two_failure_messages_are_told_apart() {
        // Tanpa `result` → "Gagal mengambil video"; `result` ada tapi URL-nya
        // kosong → "Video tidak ditemukan".
        for data in [
            serde_json::json!({ "status": false, "result": { "no_watermark": "https://cdn/v.mp4" } }),
            serde_json::json!({ "status": true }),
            serde_json::json!({ "status": true, "result": null }),
            serde_json::json!({}),
        ] {
            assert!(!accepted(&data), "{data} seharusnya ditolak");
        }
        assert_eq!(video_url(&serde_json::json!({})), "");

        for result in [
            serde_json::json!({ "no_watermark": "", "watermark": "" }),
            serde_json::json!({}),
            serde_json::json!({ "no_watermark": null, "watermark": null }),
        ] {
            let data = serde_json::json!({ "status": true, "result": result });
            assert!(accepted(&data));
            assert_eq!(video_url(&data), "", "{data}");
        }
    }

    #[test]
    fn the_watermarked_url_is_only_the_second_choice() {
        assert_eq!(
            video_url(&serde_json::json!({
                "status": true,
                "result": { "no_watermark": "https://cdn/a.mp4", "watermark": "https://cdn/b.mp4" }
            })),
            "https://cdn/a.mp4"
        );
        assert_eq!(
            video_url(&serde_json::json!({
                "status": true,
                "result": { "no_watermark": "", "watermark": "https://cdn/b.mp4" }
            })),
            "https://cdn/b.mp4"
        );
    }

    #[test]
    fn scraper_btch_membungkus_data_cocofun_dalam_result() {
        let http = Stub::json(serde_json::json!({
            "no_watermark": "https://cdn.example/video.mp4",
            "watermark": "https://cdn.example/watermark.mp4"
        }));
        let r = run_with_http("https://www.cocofun.com/share/post/xxx", http.clone());

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
                "https://backend1.tioo.eu.org/cocofun?url=https%3A%2F%2Fwww.cocofun.com%2Fshare%2Fpost%2Fxxx"
            ]
        );
    }

    #[test]
    fn kegagalan_transport_btch_menghasilkan_branch_gagal_mengambil_video() {
        let r = run("https://www.cocofun.com/share/post/xxx");
        assert_eq!(text_of(&r), NO_RESULT);
        assert!(
            matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "🕕"),
            "reaksi jam harus tetap ada"
        );
    }

    #[test]
    #[ignore = "uji jaringan CocoFun; set OKTZ_TEST_COCOFUN_URL"]
    fn live_cocofundl_mengambil_video() {
        // Uji live padanan `plugins/download/cocofundl.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_COCOFUN_URL");
        let (http, ctx) = live_ctx("cocofundl", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("cocofundl").unwrap().handler)(ctx);
        assert_media(&reply, &["video"], &http);
    }
}
