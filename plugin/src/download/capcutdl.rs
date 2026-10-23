//! `plugins/download/capcutdl.js` — unduh video CapCut.
//!
//! Scraper `capcut` memakai kontrak resmi `btch-downloader@6.3.6` melalui
//! `oktz_scraper::btch`, termasuk endpoint, header, envelope, dan penanganan
//! error transport seperti SDK Node.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

/// `capcutdl.js:31` — `/capcut\.com/i`, tidak dikunci ke host.
pub(crate) fn is_capcut_url(url: &str) -> bool {
    url.to_lowercase().contains("capcut.com")
}

/// `capcutdl.js:23-28` — kartu pakai, delimiter markdown-nya backtick.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}ccdl <url>`\n\n\
         > Contoh:\n\
         > `{prefix}ccdl https://www.capcut.com/t/xxx`"
    )
}

/// `capcutdl.js:32` — URL bukan CapCut.
pub(crate) const BAD_URL: &str = "❌ URL tidak valid. Gunakan link CapCut.";

/// `capcutdl.js:41` — backend menjawab tanpa `originalVideoUrl`.
pub(crate) const NO_VIDEO: &str = "❌ Gagal mengambil video. Coba link lain.";

/// `capcutdl.js:40` — `data?.status` harus `true`.
pub(crate) fn status_ok(data: &Value) -> bool {
    data.get("status").and_then(Value::as_bool) == Some(true)
}

/// `capcutdl.js:40` — `data?.originalVideoUrl`.
pub(crate) fn original_video_url(data: &Value) -> String {
    data.get("originalVideoUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[plugin(
    name = "capcutdl",
    alias = ["ccdl", "capcut", "cc"],
    category = "download",
    description = "Download video CapCut",
    usage = ".ccdl <url>",
    example = ".ccdl https://www.capcut.com/t/xxx",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn capcutdl(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();
    if url.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    if !is_capcut_url(url) {
        return Reply::text(BAD_URL);
    }

    // `capcutdl.js:38` memakai SDK Node yang menangkap error transport dan
    // mengembalikan envelope `status: false`.
    let data = oktz_scraper::btch::capcut(ctx.http.as_ref(), url);

    let video = original_video_url(&data);
    if !status_ok(&data) || video.is_empty() {
        return Reply::text(NO_VIDEO).with_reaction("🕕");
    }

    // `capcutdl.js:44-50` — `sendMedia` video tanpa caption; reaksi ⏱️ adalah
    // satu-satunya reaksi yang dikirim JS.
    Reply::video(MediaSource::url(video), "").with_reaction("🕕")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        body: Result<String, HttpError>,
        seen: Mutex<Vec<HttpCall>>,
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
        fn get(&self, _url: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("klien btch memakai HttpCall::call")
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

        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
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
    }

    fn run(text: &str) -> Reply {
        run_with_http(text, Stub::offline())
    }

    fn run_with_http(text: &str, http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ccdl".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: vec![text.into()],
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("capcutdl").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("capcutdl").unwrap();
        assert_eq!(p.meta.name, "capcutdl");
        assert_eq!(p.meta.alias, ["ccdl", "capcut", "cc"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video CapCut");
        assert_eq!(p.meta.usage, ".ccdl <url>");
        assert_eq!(p.meta.example, ".ccdl https://www.capcut.com/t/xxx");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        // `cc` diklaim dua kali di JS: `download/capcutdl.js:5` (alias) dan
        // `group/clearchat.js:3` (array `name`). Tabrakan bawaan repo, bukan
        // hasil port — hanya satu yang bisa menang di registry.
        for a in ["ccdl", "capcut"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "capcutdl");
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
        assert_eq!(text_of(&run("capcut co m")), BAD_URL);
    }

    #[test]
    fn the_unanchored_regex_quirk_of_the_js_is_kept() {
        // `/capcut\.com/i` tidak dikunci ke host, jadi `notcapcut.com` dan
        // `capcut.com.evil.example` tetap lolos — Guardian ini harus persis
        // sama dengan JS.
        assert!(is_capcut_url("https://capcut.com.evil.example/x"));
        assert!(is_capcut_url("https://notcapcut.com/x"));
        assert!(is_capcut_url("https://www.CAPCUT.com/t/1"));
    }

    #[test]
    fn a_response_without_the_video_field_is_the_no_video_message() {
        for data in [
            serde_json::json!({ "status": false, "originalVideoUrl": "https://cdn.example/v.mp4" }),
            serde_json::json!({ "status": true }),
            serde_json::json!({ "status": true, "originalVideoUrl": "" }),
            serde_json::json!({ "status": true, "originalVideoUrl": null }),
            serde_json::json!({}),
        ] {
            let video = original_video_url(&data);
            assert!(!status_ok(&data) || video.is_empty(), "{data} lolos");
        }
    }

    #[test]
    fn scraper_btch_memakai_endpoint_header_dan_envelope_yang_sama() {
        let http = Stub::json(serde_json::json!({
            "originalVideoUrl": "https://cdn.example/video.mp4"
        }));
        let r = run_with_http("https://www.capcut.com/t/xxx", http.clone());

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

        let seen = http.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(
            seen[0].url,
            "https://backend1.tioo.eu.org/capcut?url=https%3A%2F%2Fwww.capcut.com%2Ft%2Fxxx"
        );
        let header = |nama: &str| {
            seen[0]
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(nama))
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(header("content-type"), Some("application/json"));
        assert_eq!(header("user-agent"), Some("btch/6.3.6"));
        assert_eq!(header("x-client-version"), Some("6.3.6"));
    }

    #[test]
    fn kegagalan_transport_btch_menghasilkan_branch_gagal_mengambil_video() {
        // `btch-downloader` menangkap error HTTP dan mengembalikan envelope
        // `status: false`, jadi plugin masuk branch `Gagal mengambil video`.
        let r = run("https://www.capcut.com/t/xxx");
        assert_eq!(text_of(&r), NO_VIDEO);
        assert!(
            matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "🕕"),
            "reaksi jam harus tetap ada"
        );
    }

    #[test]
    #[ignore = "uji jaringan CapCut; set OKTZ_TEST_CAPCUT_URL"]
    fn live_capcutdl_mengambil_video() {
        // Uji live padanan `plugins/download/capcutdl.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_CAPCUT_URL");
        let (http, ctx) = live_ctx("capcutdl", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("capcutdl").unwrap().handler)(ctx);
        assert_media(&reply, &["video"], &http);
    }
}
