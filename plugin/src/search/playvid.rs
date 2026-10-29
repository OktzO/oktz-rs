//! `plugins/search/playvid.js` — cari dan putar video dari YouTube.
//!
//! `ponytail:` JS memakai paket npm `yt-search`; padanannya ada di
//! `search/ytsearch.rs`.
//!
//! `ponytail:` unduhan video memakai kapabilitas `youtube` format `mp4`, yang
//! di JS memakai `getYoutubeDirectUrl(url, "360")` (SaveTube). Padanannya
//! `oktz_scraper::youtube::get_youtube_direct_url`.
//!
//! `ponytail:` `sock.sendPreview` (pratinjau tautan) menjadi `Reply::Preview`.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use oktz_plugin_api::registry::CardContext;

use super::ytsearch::{self, format_views};

/// `m.pushName` JS kosong menjadi `"Unknown"` (`src/lib/serialize.js:760`).
pub(crate) fn push_name(ctx: &Ctx) -> String {
    if ctx.push_name.is_empty() {
        "Unknown".to_string()
    } else {
        ctx.push_name.clone()
    }
}

/// `playvid.js:31` — kartu pakai.
pub(crate) fn help(push: &str, prefix: &str) -> String {
    format!(
        "Halo *{push}* 👋\n\nUntuk mencari dan memutar video dari YouTube, silakan gunakan format:\n- `{prefix}playvid <judul video>`\n\nContoh:\n- `{prefix}playvid windah basudara`"
    )
}

/// `playvid.js:41-47` — teks informasi.
pub(crate) fn info_video(push: &str, v: &ytsearch::Video) -> String {
    format!(
        "Halo *{push}*, ini video yang kamu cari:\n\n\
         📌 *Judul:* {}\n\
         👤 *Channel:* {}\n\
         ⏱️ *Durasi:* {}\n\
         👀 *Views:* {}\n\
         📅 *Upload:* {}\n\n\
         _⏳ Sedang mengunduh video, harap tunggu sebentar ya..._",
        v.title,
        v.author_name,
        v.timestamp,
        format_views(v.views),
        v.ago
    )
}

/// `playvid.js:81` — gagal.
pub(crate) fn gagal(push: &str) -> String {
    format!("Maaf *{push}*, fitur putar videonya sedang ada kendala atau video tersebut terlalu besar. Silakan coba lagi nanti ya!")
}

#[plugin(
    name = "playvid",
    alias = ["playmp4"],
    category = "search",
    description = "Cari dan putar video dari YouTube",
    usage = ".playvid <query>",
    example = ".playvid windah basudara",
    cooldown = 15,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn playvid(ctx: Ctx) -> Reply {
    let query = ctx.text.trim().to_string();
    let push = push_name(&ctx);
    if query.is_empty() {
        return Reply::text(help(&push, &ctx.prefix));
    }

    let videos = match ytsearch::search(ctx.http.as_ref(), &query) {
        Ok(v) => v,
        Err(_) => {
            // `playvid.js:71-83` — ❌ lalu pesan kegagalan.
            return Reply::text(gagal(&push)).with_reaction("❌");
        }
    };

    let Some(video) = videos.first() else {
        // `playvid.js:38` — `throw new Error("Video tidak ditemukan")` → catch.
        return Reply::text(gagal(&push)).with_reaction("❌");
    };

    let info = info_video(&push, video);

    // `playvid.js:64` — link unduhan video.
    let direct =
        match oktz_scraper::youtube::get_youtube_direct_url(ctx.http.as_ref(), &video.url, Some("360"))
        {
            Ok(v) => v,
            Err(_) => {
                return Reply::text(gagal(&push)).with_reaction("❌");
            }
        };
    let Some(download_url) = direct.get("url").and_then(|v| v.as_str()) else {
        return Reply::text(gagal(&push)).with_reaction("❌");
    };

    // `playvid.js:49-62` — pratinjau tautan, lalu `playvid.js:66-68` — video.
    let preview = Reply::text(format!("{}\n{}", video.url, info));

    let media = Reply::video(MediaSource::url(download_url.to_string()), "");

    Reply::sequence(vec![preview, media]).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        body: String,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn body(v: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: v.to_string(),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: None,
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
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "playvid".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("playvid").unwrap().handler)(ctx)
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

    const HASIL: &str = r#"{"contents":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[
      {"itemSectionRenderer":{"contents":[
        {"videoRenderer":{
          "videoId":"aaaaaaaaaaa",
          "title":{"runs":[{"text":"Lagu Uji"}]},
          "ownerText":{"runs":[{"text":"Channel Uji"}]},
          "viewCountText":{"simpleText":"1,234,567 views"},
          "publishedTimeText":{"simpleText":"2 years ago"},
          "lengthText":{"simpleText":"3:33"}
        }}
      ]}}
    ]}}}}]}}"#;

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("playvid").unwrap();
        assert_eq!(p.meta.name, "playvid");
        assert_eq!(p.meta.alias, ["playmp4"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Cari dan putar video dari YouTube");
        assert_eq!(p.meta.usage, ".playvid <query>");
        assert_eq!(p.meta.example, ".playvid windah basudara");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 2);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `playvid.js:31` — kartu pakai.
        assert_eq!(
            help("User", "."),
            "Halo *User* 👋\n\nUntuk mencari dan memutar video dari YouTube, silakan gunakan format:\n- `.playvid <judul video>`\n\nContoh:\n- `.playvid windah basudara`"
        );
        // `playvid.js:81` — pesan kegagalan.
        assert_eq!(
            gagal("User"),
            "Maaf *User*, fitur putar videonya sedang ada kendala atau video tersebut terlalu besar. Silakan coba lagi nanti ya!"
        );
        // `playvid.js:41-47` — teks informasi.
        let v = ytsearch::Video {
            url: "https://www.youtube.com/watch?v=aaaaaaaaaaa".into(),
            title: "Lagu Uji".into(),
            thumbnail: String::new(),
            description: String::new(),
            video_id: "aaaaaaaaaaa".into(),
            views: 1_234_567,
            ago: "2 years ago".into(),
            timestamp: "3:33".into(),
            seconds: 213,
            author_name: "Channel Uji".into(),
            author_username: String::new(),
            author_url: String::new(),
        };
        assert_eq!(
            info_video("User", &v),
            "Halo *User*, ini video yang kamu cari:\n\n\
             📌 *Judul:* Lagu Uji\n\
             👤 *Channel:* Channel Uji\n\
             ⏱️ *Durasi:* 3:33\n\
             👀 *Views:* 1.2M\n\
             📅 *Upload:* 2 years ago\n\n\
             _⏳ Sedang mengunduh video, harap tunggu sebentar ya..._"
        );
    }

    #[test]
    fn tanpa_judul_menampilkan_kartu_pakai() {
        let r = run(Stub::body("{}"), "");
        assert_eq!(text_of(&r), help("Unknown", "."));
        assert_eq!(reaction_of(&r), None);
    }

    #[test]
    fn video_tidak_ditemukan_menampilkan_pesan_gagal() {
        let r = run(Stub::body(r#"{"contents":{"tabs":[]}}"#), "x");
        assert_eq!(text_of(&r), gagal("Unknown"));
        assert_eq!(reaction_of(&r), Some("❌"));
    }
}
