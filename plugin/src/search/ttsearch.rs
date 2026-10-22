//! `plugins/search/ttsearch.js` — cari video TikTok.
//!
//! `ponytail:` JS memakai scraper `tiktokSearchVideo`
//! (`src/scraper/tiktoksearch.js`); padanannya sudah ada di crate scraper:
//! `oktz_scraper::tiktoksearch::tiktok_search_video`.
//!
//! `ponytail:` JS mengirim album berisi maksimal 5 video (`maxShow =
//! Math.min(videos.length, 5)`) dengan `contextInfo` forwarded. `Reply::Video`
//! tidak punya `contextInfo`, jadi penandaan forwarded tidak ikut — sisanya
//! (video + caption) sama.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

/// `ttsearch.js:29-37` — kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "╭┈┈⬡「 🎵 *ᴛɪᴋᴛᴏᴋ sᴇᴀʀᴄʜ* 」\n┃\n㊗ ᴅsᴀɢᴇ: `{prefix}ttsearch <query>`\n┃\n╰┈┈⬡\n\n> `Contoh: {prefix}ttsearch anime`"
    )
}

/// `ttsearch.js:47` — tidak ada hasil.
pub(crate) fn tidak_ditemukan(query: &str) -> String {
    format!("❌ Tidak ditemukan video untuk: {query}")
}

/// `ttsearch.js:54-59` — caption tiap video.
pub(crate) fn caption(video: &Value) -> String {
    let judul = match video.get("title") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => "-".to_string(),
    };
    let nickname = match video.get("author").and_then(|a| a.get("nickname")) {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => "-".to_string(),
    };
    let plays = video
        .get("stats")
        .and_then(|s| s.get("plays"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let likes = video
        .get("stats")
        .and_then(|s| s.get("likes"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    format!(
        "🎵 *TIKTOK SEARCH*\n\n📌 {judul}\n👤 {nickname}\n👀 {plays} views\n❤️ {likes} likes"
    )
}

#[plugin(
    name = "ttsearch",
    alias = ["tiktoksearch", "tts", "searchtiktok"],
    category = "search",
    description = "Cari video TikTok",
    usage = ".ttsearch <query>",
    example = ".ttsearch jj epep",
    cooldown = 15,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ttsearch(ctx: Ctx) -> Reply {
    let query = ctx.args.join(" ").trim().to_string();
    if query.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    let videos = match oktz_scraper::tiktoksearch::tiktok_search_video(ctx.http.as_ref(), &query) {
        Ok(v) => v,
        Err(_) => {
            // `ttsearch.js:117-120` — ☢ lalu template error.
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    let list = videos.as_array().cloned().unwrap_or_default();
    if list.is_empty() {
        // `ttsearch.js:45-48` — ❌ lalu pesan tidak ditemukan.
        return Reply::text(tidak_ditemukan(&query)).with_reaction("❌");
    }

    // `ttsearch.js:50-64` — maksimal 5 video.
    let msgs: Vec<Reply> = list
        .iter()
        .take(5)
        .filter_map(|v| {
            let link = v.get("link").and_then(Value::as_str)?;
            if link.is_empty() {
                return None;
            }
            Some(Reply::video(
                MediaSource::url(link.to_string()),
                caption(v),
            ))
        })
        .collect();

    Reply::sequence(msgs).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    /// Stub HTTP: membalas body yang sama untuk semua request.
    #[derive(Debug)]
    struct Stub {
        body: Option<String>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn body(v: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(v.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                body: None,
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let body = self.body.clone().ok_or_else(|| HttpError {
                message: "offline".into(),
            })?;
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(body.into_bytes()),
                    mimetype: Some("application/json".into()),
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
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            let body = self.body.clone().ok_or_else(|| HttpError {
                message: "offline".into(),
            })?;
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

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ttsearch".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("ttsearch").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("ttsearch").unwrap();
        assert_eq!(p.meta.name, "ttsearch");
        assert_eq!(p.meta.alias, ["tiktoksearch", "tts", "searchtiktok"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Cari video TikTok");
        assert_eq!(p.meta.usage, ".ttsearch <query>");
        assert_eq!(p.meta.example, ".ttsearch jj epep");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `ttsearch.js:29-37` — kartu pakai.
        assert_eq!(
            help("."),
            "╭┈┈⬡「 🎵 *ᴛɪᴋᴛᴏᴋ sᴇᴀʀᴄʜ* 」\n┃\n㊗ ᴅsᴀɢᴇ: `.ttsearch <query>`\n┃\n╰┈┈⬡\n\n> `Contoh: .ttsearch anime`"
        );
        // `ttsearch.js:47` — tidak ditemukan.
        assert_eq!(tidak_ditemukan("anime"), "❌ Tidak ditemukan video untuk: anime");
        // `ttsearch.js:54-59` — caption.
        let v = serde_json::json!({
            "title": "Video A",
            "link": "https://www.tiktok.com/@a/video/1",
            "author": {"nickname": "Pembuat"},
            "stats": {"plays": 1200, "likes": 34}
        });
        assert_eq!(
            caption(&v),
            "🎵 *TIKTOK SEARCH*\n\n📌 Video A\n👤 Pembuat\n👀 1200 views\n❤️ 34 likes"
        );
        // Judul/kosong jadi "-".
        let kosong = serde_json::json!({"title": "", "author": {"nickname": ""}, "stats": {}});
        assert_eq!(
            caption(&kosong),
            "🎵 *TIKTOK SEARCH*\n\n📌 -\n👤 -\n👀 0 views\n❤️ 0 likes"
        );
    }

    #[test]
    fn tanpa_query_tampilkan_kartu_pakai() {
        let r = run(Stub::offline(), "");
        assert_eq!(text_of(&r), help("."));
        assert_eq!(reaction_of(&r), None);
    }

    #[test]
    fn hasil_kosong_menampilkan_pesan_tidak_ditemukan() {
        let r = run(
            Stub::body(r#"{"status":true,"result":[]}"#),
            "anime",
        );
        assert_eq!(text_of(&r), "❌ Tidak ditemukan video untuk: anime");
        assert_eq!(reaction_of(&r), Some("❌"));
    }
}
