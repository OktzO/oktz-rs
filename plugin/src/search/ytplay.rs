//! `plugins/search/ytplay.js` — putar video YouTube via WebSocket player.
//!
//! PENYIMPANGAN (dijelaskan di laporan akhir):
//!
//! JS membangun HTML player sendiri (`buildPlayerHtml`, `ytplay.js:44-844`)
//! dan mengirimkannya sebagai kartu AI-Rich (`AIRich` + `unifiedResponse`),
//! lalu player itu menarik video lewat WebSocket
//! `wss://stream.kanara.my.id/?url=…`.
//!
//! `oktz-plugin-api` TIDAK punya (a) klien WebSocket, dan (b) primitif
//! `unifiedResponse`/AI-Rich — `Reply` hanya punya `Interactive`, `Table`,
//! `Preview`, dan media biasa. Jadi HTML player 800 baris tidak bisa
//! dikirim.
//!
//! Yang dilakukan di sini: pencarian + link video langsung (SaveTube 360p,
//! sama seperti `search/playvid.rs`) dikirim sebagai pratinjau tautan
//! (`Reply::Preview`) — pratinjau itu adalah pesan native WhatsApp dengan
//! thumbnail video, jadi hasilnya paling dekat dengan "player" yang bisa
//! dilakukan tanpa WebSocket. Teks judul/channel/durasi mengikuti header yang
//! sama dengan kartu JS (`YOUTUBE PLAY - <judul>`).

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use oktz_plugin_api::registry::{CardContext, CardHeader};

use super::ytsearch::{self, format_views};

/// `ytplay.js:849-851` — judul/link kosong.
pub(crate) fn tanpa_judul(prefix: &str) -> String {
    format!("Mana judul atau link video YouTube yang mau diputar?\n\nContoh: {prefix}ytplay melukis senja")
}

/// `ytplay.js:889` — judul header kartu.
pub(crate) fn judul_header(judul: &str) -> String {
    format!("YOUTUBE PLAY - {judul}")
}

/// `ytplay.js:910` — gagal.
pub(crate) fn gagal(pesan: &str) -> String {
    format!("Gagal memutar video YouTube.\n{pesan}")
}

/// Ringkasan video — memakai header yang sama dengan kartu JS.
pub(crate) fn ringkasan(v: &ytsearch::Video, durasi: &str, direct: &str) -> String {
    format!(
        "🎬 *YOUTUBE PLAY - {}*\n\n\
         📺 Channel: {}\n\
         ⏱️ Durasi: {}\n\
         👀 Views: {}\n\
         📅 Upload: {}\n\n\
         🔗 {}\n\
         ▶️ Tonton: {}",
        v.title,
        v.author_name,
        durasi,
        format_views(v.views),
        v.ago,
        v.url,
        direct
    )
}

/// `info.duration` → `"m:ss"` (`ytplay.js:875-877`).
pub(crate) fn format_durasi(detik: u64) -> String {
    if detik == 0 {
        return String::new();
    }
    format!("{}:{:02}", detik / 60, detik % 60)
}

#[plugin(
    name = "ytplay",
    alias = ["ytp", "playvideo"],
    category = "search",
    description = "Putar video YouTube via WebSocket player",
    usage = ".ytplay <judul>",
    example = ".ytplay melukis senja",
    cooldown = 20,
    energi = 3,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ytplay(ctx: Ctx) -> Reply {
    let query = ctx.text.trim().to_string();
    if query.is_empty() {
        return Reply::text(tanpa_judul(&ctx.prefix));
    }

    let videos = match ytsearch::search(ctx.http.as_ref(), &query) {
        Ok(v) => v,
        Err(e) => {
            // `ytplay.js:908-911` — ☢ lalu pesan gagal.
            return Reply::text(gagal(&e.message)).with_reaction("☢");
        }
    };

    let Some(video) = videos.first().cloned() else {
        // `ytplay.js:25` — `throw new Error("Video tidak ditemukan.")`.
        return Reply::text(gagal("Video tidak ditemukan.")).with_reaction("☢");
    };

    // `ytplay.js:858` — `getYoutubeDirectUrl(yt.url, "360")`.
    let direct =
        match oktz_scraper::youtube::get_youtube_direct_url(ctx.http.as_ref(), &video.url, Some("360"))
        {
            Ok(v) => v,
            Err(e) => {
                return Reply::text(gagal(&e.message)).with_reaction("☢");
            }
        };
    let Some(direct_url) = direct.get("url").and_then(|v| v.as_str()) else {
        // `ytplay.js:860-862` — `throw new Error("Gagal mendapatkan direct link video.")`.
        return Reply::text(gagal("Gagal mendapatkan direct link video.")).with_reaction("☢");
    };

    let durasi = direct
        .get("duration")
        .and_then(|d| d.as_u64())
        .map(format_durasi)
        .unwrap_or_default();

    let body = ringkasan(&video, &durasi, direct_url);
    let judul = direct
        .get("title")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or(video.title.as_str());

    Reply::interactive(body)
        .with_footer(judul_header(judul))
        .with_header(
            CardHeader::new().image(MediaSource::url(video.thumbnail.clone())),
        )
        .with_context(CardContext::new().quote_trigger())
        .with_reaction("✅")
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
            command: "ytplay".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("ytplay").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("ytplay").unwrap();
        assert_eq!(p.meta.name, "ytplay");
        assert_eq!(p.meta.alias, ["ytp", "playvideo"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Putar video YouTube via WebSocket player");
        assert_eq!(p.meta.usage, ".ytplay <judul>");
        assert_eq!(p.meta.example, ".ytplay melukis senja");
        assert_eq!(p.meta.cooldown, 20);
        assert_eq!(p.meta.energi, 3);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `ytplay.js:849-851` — judul kosong.
        assert_eq!(
            tanpa_judul("."),
            "Mana judul atau link video YouTube yang mau diputar?\n\nContoh: .ytplay melukis senja"
        );
        // `ytplay.js:889` — judul header.
        assert_eq!(judul_header("Lagu"), "YOUTUBE PLAY - Lagu");
        // `ytplay.js:910` — pesan gagal.
        assert_eq!(
            gagal("Video tidak ditemukan."),
            "Gagal memutar video YouTube.\nVideo tidak ditemukan."
        );
        // `ytplay.js:875-877` — durasi `m:ss`.
        assert_eq!(format_durasi(213), "3:33");
        assert_eq!(format_durasi(3723), "62:03");
        assert_eq!(format_durasi(0), "");
    }

    #[test]
    fn tanpa_judul_menampilkan_petunjuk() {
        let r = run(Stub::body("{}"), "");
        assert_eq!(text_of(&r), tanpa_judul("."));
        assert_eq!(reaction_of(&r), None);
    }

    #[test]
    fn video_tidak_ditemukan_menampilkan_pesan_gagal() {
        let r = run(Stub::body(r#"{"contents":{"tabs":[]}}"#), "x");
        assert_eq!(text_of(&r), gagal("Video tidak ditemukan."));
        assert_eq!(reaction_of(&r), Some("☢"));
    }
}
