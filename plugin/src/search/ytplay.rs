//! `plugins/search/ytplay.js` — putar video YouTube via WebSocket player.
//!
//! PENYIMPANGAN (dijelaskan di laporan akhir):
//!
//! JS membangun HTML player sendiri (`buildPlayerHtml`, `ytplay.js:44-844`)
//! dan mengirimkannya sebagai kartu AI-Rich (`AIRich` + `unifiedResponse`),
//! lalu player itu menarik video lewat WebSocket
//! `wss://stream.kanara.my.id/?url=…`.
//!
//! Kartu HTML dikirim lewat `RichAi` (`Reply::TabelAi`) dengan section
//! `FOAHtmlPrimitiveDemoDONOTUSE` — sama seperti `aiRich.addSection(...)
//! ; aiRich.send` (`ytplay.js:891-905`): HANYA section HTML, tanpa
//! submessage teks (kartu JS tidak menampilkan ringkasan di luar player).
//!
//! `ponytail:` streaming WebSocket `wss://stream.kanara.my.id/?url=…`
//! (`ytplay.js:879`, `startVideoStream`) tidak bisa dicapai dari API plugin
//! (tidak ada klien WebSocket) — player HTML memakai URL direct 360p
//! (`getYoutubeDirectUrl`, sama seperti `search/playvid.rs`) sebagai
//! `<video src>`, dan URL WS ditampilkan sebagai teks di dalam player
//! untuk pemutar eksternal. Upgrade path: tambah klien WebSocket di
//! `oktz-plugin-api`, atau varian HTML RichAi di sana bila klien butuh
//! `wsUrl` terstruktur.
//!
//! HTML player `buildPlayerHtml` 800 baris (`ytplay.js:44-844`) tidak
//! disalin utuh: versi ringkas di sini memuat title, channel, durasi,
//! poster base64 (download seperti `ytplay.js:864-873`), `<video controls>`
//! dengan `src` direct 360p, dan tombol ganti kualitas 360p/720p.

use oktz_plugin_api::{Ctx, Reply, RichAi, plugin};
use serde_json::json;

use super::play2::{cari_video, poster_base64};

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

/// `info.duration` → `"m:ss"` (`ytplay.js:875-877`).
pub(crate) fn format_durasi(detik: u64) -> String {
    if detik == 0 {
        return String::new();
    }
    format!("{}:{:02}", detik / 60, detik % 60)
}

/// `escapeHtmlJs` (`ytplay.js:35-42`) — escape untuk string di dalam
/// template HTML: `\`, `"`, backtick, `${`, dan `<`.
pub(crate) fn escape_js(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace("${", "\\${")
        .replace('<', "\\x3C")
}

/// `buildPlayerHtml` (`ytplay.js:44-844`) versi ringkas: title, channel,
/// durasi, poster base64, `<video controls src=direct>` + ganti kualitas
/// 360p/720p (`hd_url` kosong = tanpa tombol), dan `wsUrl` sebagai teks
/// fallback (streaming WS tidak tercapai — lihat `ponytail:` di header
/// modul). Pause/seek/mute pakai kontrol native `<video controls>` (JS
/// custom bar tidak disalin ulang).
pub(crate) fn build_player_html(
    title: &str,
    channel: &str,
    durasi: &str,
    direct_url: &str,
    ws_url: &str,
    poster: &str,
    hd_url: &str,
) -> String {
    let t = escape_js(title);
    let c = escape_js(channel);
    let d = escape_js(durasi);
    let v = escape_js(direct_url);
    let w = escape_js(ws_url);
    let hd = escape_js(hd_url);
    let tombol = if hd_url.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="kualitas"><button onclick="gantiQ('{v}',this)">360p</button><button onclick="gantiQ('{hd}',this)">720p</button></div>"#
        )
    };
    format!(
        r#"<style>
*{{box-sizing:border-box;-webkit-tap-highlight-color:transparent}}
body{{margin:0;background:#090a0f;color:#fff;font-family:Arial,Helvetica,sans-serif}}
.wrap{{display:flex;justify-content:center;padding:12px}}
.player{{width:100%;max-width:440px;background:#111318;border:1px solid rgba(255,255,255,.12);border-radius:20px;overflow:hidden}}
.poster{{width:100%;aspect-ratio:16/9;background:#000}}
.poster img{{width:100%;height:100%;object-fit:cover;display:block}}
.pad{{padding:14px 16px}}
.title{{font-size:15px;font-weight:700;line-height:1.35}}
.meta{{font-size:12px;color:rgba(255,255,255,.65);margin-top:4px}}
video{{width:100%;border-radius:12px;margin-top:12px;background:#000}}
.kualitas{{display:flex;gap:8px;margin-top:10px}}
.kualitas button{{flex:1;padding:8px;background:#1c1f27;color:#fff;border:1px solid rgba(255,255,255,.15);border-radius:10px;font-size:13px}}
.ws{{font-size:11px;color:rgba(255,255,255,.55);margin-top:10px;word-break:break-all}}
</style>
<script>
const vid=document.getElementById('vid');
function gantiQ(u,btn){{const t=vid.currentTime;vid.src=u;vid.onloadedmetadata=()=>{{vid.currentTime=t;}};vid.play();}}
</script>
<div class="wrap"><div class="player">
<div class="poster"><img src="{poster}" alt="{t}"></div>
<div class="pad">
<div class="title">{t}</div>
<div class="meta">{c} • {d}</div>
<video id="vid" controls preload="metadata" src="{v}"></video>
{tombol}
<div class="ws">Stream WS: {w}</div>
</div>
</div></div>"#
    )
}

/// Section `FOAHtmlPrimitiveDemoDONOTUSE` — `ytplay.js:892-904`.
/// `view_model.title` = `titleHeader` (`YOUTUBE PLAY - <judul>`).
pub(crate) fn section_html(title_header: &str, payload: &str) -> serde_json::Value {
    json!({
        "__typename": "GenAIUnifiedResponseSection",
        "title": title_header,
        "view_model": {
            "__typename": "GenAISingleLayoutViewModel",
            "title": title_header,
            "primitive": {
                "__typename": "FOAHtmlPrimitiveDemoDONOTUSE",
                "trusted_sources": [],
                "payload": payload.trim(),
            }
        }
    })
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

    let videos = match cari_video(ctx.http.as_ref(), &query) {
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

    let judul = direct
        .get("title")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or(video.title.as_str());
    let channel = direct
        .get("channel")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
        .unwrap_or(video.author_name.as_str());

    // `posterBase64` (`ytplay.js:864-873`): thumbnail diunduh lalu base64.
    // Rust memakai `thumb` direct bila ada, jatuh ke thumbnail pencarian.
    let thumb = direct
        .get("thumb")
        .and_then(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .unwrap_or(video.thumbnail.as_str());
    let poster = poster_base64(ctx.http.as_ref(), thumb);

    // `wsUrl` (`ytplay.js:879`) — ditampilkan sebagai teks di player;
    // streaming WS-nya sendiri tidak tercapai (lihat `ponytail:` modul).
    let ws_url = format!(
        "wss://stream.kanara.my.id/?url={}",
        url_encode(direct_url)
    );
    // Kualitas HD opsional — gagal diam-diam (tombol disembunyikan).
    let hd_url = oktz_scraper::youtube::get_youtube_direct_url(ctx.http.as_ref(), &video.url, Some("720"))
        .ok()
        .and_then(|v| v.get("url").and_then(|u| u.as_str()).map(str::to_string))
        .unwrap_or_default();
    let html = build_player_html(judul, channel, &durasi, direct_url, &ws_url, &poster, &hd_url);
    let rich = RichAi {
        // `ytplay.js:882-898` hanya `addSection` — tidak ada `addText`, jadi
        // `submessages` kosong dan yang tampil murni kartu HTML.
        submessages: Vec::new(),
        sections: vec![section_html(&judul_header(judul), &html)],
        disclaimer: None,
        quoted: false,
    };
    Reply::TabelAi(Box::new(rich)).with_reaction("✅")
}

/// `encodeURIComponent` standar untuk `wsUrl` — spasi `%20`, bukan `+`.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
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

    #[test]
    fn player_html_memuat_title_poster_video_dan_ws() {
        let html = build_player_html(
            "A<B",
            "Chan",
            "3:33",
            "https://cdn.example/v.mp4",
            "wss://stream.kanara.my.id/?url=x",
            "data:image/jpeg;base64,QQ==",
            "https://cdn.example/v720.mp4",
        );
        assert!(html.contains("A\\x3CB"), "judul di-escape: {html}");
        assert!(html.contains("data:image/jpeg;base64,QQ=="));
        assert!(html.contains(r#"<video id="vid" controls"#));
        assert!(html.contains("https://cdn.example/v.mp4"));
        assert!(html.contains("720p"));
        assert!(html.contains("gantiQ"));
        assert!(html.contains("wss://stream.kanara.my.id/?url=x"));
    }

    #[test]
    fn section_html_membawa_typename_foa_dan_judul_header() {
        let s = section_html("YOUTUBE PLAY - Lagu", "<div>x</div>");
        assert_eq!(
            s["__typename"],
            serde_json::json!("GenAIUnifiedResponseSection")
        );
        assert_eq!(
            s["view_model"]["primitive"]["__typename"],
            serde_json::json!("FOAHtmlPrimitiveDemoDONOTUSE")
        );
        assert_eq!(
            s["view_model"]["primitive"]["payload"],
            serde_json::json!("<div>x</div>")
        );
    }

    #[test]
    fn url_encode_memakai_persen_bukan_plus() {
        assert_eq!(url_encode("https://x/y z?a=b&c=d"), "https%3A%2F%2Fx%2Fy%20z%3Fa%3Db%26c%3Dd");
        assert_eq!(url_encode("abc-_.~"), "abc-_.~");
    }
}
