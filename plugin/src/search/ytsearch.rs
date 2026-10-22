//! Padanan paket npm `yt-search` untuk plugin `search/*`.
//!
//! JS memakai `import yts from "yt-search"` di `search/yts.js`,
//! `search/playvid.js`, `search/playch.js`, dan `search/ytplay.js`. Paket itu
//! mengambil halaman hasil pencarian YouTube dan mengembalikan bentuk berikut
//! (yang dipakai keempat plugin):
//!
//! ```js
//! {
//!   url: "https://www.youtube.com/watch?v=<id>",
//!   title, thumbnail, description, videoId,
//!   views: <number>, ago: "2 years ago", timestamp: "3:33",
//!   author: { name, username, url },
//!   duration: { timestamp: "3:33", seconds: 213 },
//! }
//! ```
//!
//! Data sumbernya adalah `ytInitialData` — payload yang sama dengan jawaban
//! InnerTube `youtubei/v1/search`. Di sini endpoint InnerTube yang dipakai
//! (kunci API dan header identik dengan `oktz-scraper::youtube::search_youtube`)
//! supaya tidak ada request HTML yang perlu di-parse.
//!
//! Yang TIDAK bisa ditiru: `yt-search` juga menyaring jenis hasil (video /
//! playlist / channel). Yang dikembalikan di sini hanya video, karena keempat
//! plugin JS hanya pernah memakai `videos`.

use oktz_plugin_api::{HttpCall, HttpError, HttpGet};
use serde_json::{Value, json};

/// `innertubeKey` (`src/scraper/youtube.js:264`).
const INNER_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

/// `innertubeUa` (`src/scraper/youtube.js:265-266`).
const INNER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Satu hasil video — bentuk `yt-search`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Video {
    pub url: String,
    pub title: String,
    pub thumbnail: String,
    pub description: String,
    pub video_id: String,
    /// Jumlah penonton — DIANGKA seperti `yt-search` (angka, bukan teks).
    pub views: u64,
    /// Teks waktu unggah, mis. "2 years ago".
    pub ago: String,
    /// Durasi dalam bentuk teks, mis. "3:33".
    pub timestamp: String,
    /// Durasi dalam detik.
    pub seconds: u64,
    /// Nama channel.
    pub author_name: String,
    /// `author.username` — handle channel kalau ada.
    pub author_username: String,
    /// `author.url` — URL channel.
    pub author_url: String,
}

impl Video {
    /// `formatViews(n)` (`playvid.js:16-21`, `playch.js:31-36`).
    ///
    /// `if (!n) return "0"` jadi nol (dan `undefined`) jadi "0".
    pub(crate) fn format_views(&self) -> String {
        format_views(self.views)
    }
}

/// `formatViews(n)` — dipakai `playvid.js` dan `playch.js`.
pub(crate) fn format_views(n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    if n >= 1_000_000 {
        return format!("{}M", trim_float(n as f64 / 1_000_000.0));
    }
    if n >= 1_000 {
        return format!("{}K", trim_float(n as f64 / 1_000.0));
    }
    n.to_string()
}

/// `Number.prototype.toFixed(1)` — satu desimal, dengan pembulatan banker's
/// yang sama seperti V8 (ECMAScript `Number::toFixed` memakai pembulatan ke
/// representasi desimal terdekat, bukan pembulatan ke-even).
fn trim_float(v: f64) -> String {
    let s = format!("{v:.1}");
    s
}

/// `yts(query)` — pencarian video YouTube.
///
/// Mengembalikan daftar kosong kalau tidak ada video; error jaringan dilempar
/// apa adanya (JS `yts` melempar, dan plugin memanggilnya di dalam `try`).
pub(crate) fn search(http: &dyn HttpGet, q: &str) -> Result<Vec<Video>, HttpError> {
    let res = http.call(
        &HttpCall::post(format!(
            "https://www.youtube.com/youtubei/v1/search?key={INNER_KEY}"
        ))
        .header("content-type", "application/json")
        .header("user-agent", INNER_UA)
        .header("x-youtube-client-name", "1")
        .header("x-youtube-client-version", "2.20240101.00.00")
        .json(&json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20240101.00.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": q
        })),
    )?;

    if !(200..300).contains(&res.status) {
        return Err(HttpError {
            message: format!("Pencarian gagal HTTP {}", res.status),
        });
    }
    let data = res.json().unwrap_or(Value::Null);

    let mut renderers: Vec<&Value> = Vec::new();
    kumpulkan_renderer(data.get("contents"), &mut renderers);
    Ok(renderers
        .iter()
        .filter_map(|r| video_dari_renderer(r))
        .collect())
}

/// Menelusuri `videoRenderer` — sama seperti `findVideoRenderer` di
/// `src/scraper/youtube.js:268-275`, tapi WITHOUT berhenti di node pertama:
/// `yt-search` mengembalikan SEMUA video, bukan cuma yang pertama.
fn kumpulkan_renderer<'a>(node: Option<&'a Value>, out: &mut Vec<&'a Value>) {
    let Some(node) = node.filter(|v| v.is_object() || v.is_array()) else {
        return;
    };
    if let Some(r) = node.get("videoRenderer") {
        out.push(r);
        return;
    }
    match node {
        Value::Array(a) => {
            for v in a {
                kumpulkan_renderer(Some(v), out);
            }
        }
        Value::Object(m) => {
            for v in m.values() {
                kumpulkan_renderer(Some(v), out);
            }
        }
        _ => {}
    }
}

/// `videoRenderer` → bentuk `yt-search`.
fn video_dari_renderer(r: &Value) -> Option<Video> {
    let video_id = r
        .get("videoId")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())?
        .to_string();

    let title = teks_sederhana(r.get("title"));
    let author_name = teks_sederhana(r.get("ownerText"));
    let author_username = r
        .get("ownerText")
        .and_then(|o| o.get("runs"))
        .and_then(Value::as_array)
        .and_then(|runs| runs.first())
        .and_then(|run| run.get("navigationEndpoint"))
        .and_then(|n| n.get("browseEndpoint"))
        .and_then(|b| b.get("browseId"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let author_url = if author_username.is_empty() {
        String::new()
    } else {
        format!("https://www.youtube.com/{}", author_username)
    };

    // `viewCountText` = "1,234,567 views" — `yt-search` mengubahnya jadi angka.
    let views = parse_views(&teks_sederhana(r.get("viewCountText")));
    let ago = teks_sederhana(r.get("publishedTimeText"));
    let timestamp = teks_sederhana(r.get("lengthText"));
    let seconds = parse_durasi(&timestamp);

    let thumbnail = r
        .get("thumbnail")
        .and_then(|t| t.get("thumbnails"))
        .and_then(Value::as_array)
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let description = r
        .get("detailedMetadataSnippets")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|s| s.get("snippetText"))
        .and_then(|s| s.get("runs"))
        .and_then(Value::as_array)
        .map(|runs| {
            runs.iter()
                .filter_map(|run| run.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    Some(Video {
        url: format!("https://www.youtube.com/watch?v={video_id}"),
        title,
        thumbnail,
        description,
        video_id,
        views,
        ago,
        timestamp,
        seconds,
        author_name,
        author_username,
        author_url,
    })
}

/// Teks sederhana dari `title`/`ownerText`: `simpleText` atau gabungan
/// `runs[].text`.
fn teks_sederhana(node: Option<&Value>) -> String {
    let Some(node) = node else {
        return String::new();
    };
    if let Some(s) = node.get("simpleText").and_then(Value::as_str) {
        return s.to_string();
    }
    node.get("runs")
        .and_then(Value::as_array)
        .map(|runs| {
            runs.iter()
                .filter_map(|run| run.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

/// `"1,234,567 views"` → `1234567`. Teks tanpa digit → 0.
fn parse_views(teks: &str) -> u64 {
    let digit: String = teks.chars().filter(|c| c.is_ascii_digit()).collect();
    digit.parse().unwrap_or(0)
}

/// `"3:33"` → 213. `"1:02:03"` → 3723. Teks lain → 0.
fn parse_durasi(timestamp: &str) -> u64 {
    let bagian: Vec<u64> = timestamp
        .split(':')
        .filter_map(|p| p.trim().parse::<u64>().ok())
        .collect();
    match bagian.len() {
        2 => bagian[0] * 60 + bagian[1],
        3 => bagian[0] * 3600 + bagian[1] * 60 + bagian[2],
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpMethod, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        body: String,
        permintaan: Mutex<Vec<HttpCall>>,
    }

    impl HttpGet for Stub {
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.permintaan.lock().unwrap().push(req.clone());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.as_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
    }

    const HASIL: &str = r#"{"contents":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[
      {"itemSectionRenderer":{"contents":[
        {"videoRenderer":{
          "videoId":"aaaaaaaaaaa",
          "title":{"runs":[{"text":"Lagu Uji"}]},
          "ownerText":{"runs":[{"text":"Channel Uji","navigationEndpoint":{"browseEndpoint":{"browseId":"UC123"}}}]},
          "viewCountText":{"simpleText":"1,234,567 views"},
          "publishedTimeText":{"simpleText":"2 years ago"},
          "lengthText":{"simpleText":"3:33"},
          "thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/a/low.jpg"},{"url":"https://i.ytimg.com/vi/a/hq.jpg"}]},
          "detailedMetadataSnippets":[{"snippetText":{"runs":[{"text":"Deskripsi video."}]}}]
        }},
        {"videoRenderer":{"videoId":"bbbbbbbbbbb","title":{"simpleText":"Video Dua"},"lengthText":{"simpleText":"1:02:03"}}}
      ]}}
    ]}}}}]}}"#;

    #[test]
    fn parsing_renderer_menghasilkan_bentuk_yt_search() {
        let http = Stub {
            body: HASIL.to_string(),
            permintaan: Mutex::new(Vec::new()),
        };
        let v = search(&http, "lagu uji").unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].url, "https://www.youtube.com/watch?v=aaaaaaaaaaa");
        assert_eq!(v[0].title, "Lagu Uji");
        assert_eq!(v[0].author_name, "Channel Uji");
        assert_eq!(v[0].author_username, "UC123");
        assert_eq!(v[0].views, 1_234_567);
        assert_eq!(v[0].ago, "2 years ago");
        assert_eq!(v[0].timestamp, "3:33");
        assert_eq!(v[0].seconds, 213);
        assert_eq!(v[0].thumbnail, "https://i.ytimg.com/vi/a/hq.jpg");
        assert_eq!(v[0].description, "Deskripsi video.");
        assert_eq!(v[1].seconds, 3723);

        let p = http.permintaan.lock().unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].method, HttpMethod::Post);
        assert!(p[0].url.starts_with("https://www.youtube.com/youtubei/v1/search?key="));
    }

    #[test]
    fn format_views_seperti_js() {
        // `playvid.js:16-21`.
        assert_eq!(format_views(0), "0");
        assert_eq!(format_views(999), "999");
        assert_eq!(format_views(1_000), "1.0K");
        assert_eq!(format_views(1_500), "1.5K");
        assert_eq!(format_views(1_000_000), "1.0M");
        assert_eq!(format_views(1_234_567), "1.2M");
    }

    #[test]
    fn parse_durasi_menangani_berbagai_bentuk() {
        assert_eq!(parse_durasi("3:33"), 213);
        assert_eq!(parse_durasi("1:02:03"), 3723);
        assert_eq!(parse_durasi("LIVE"), 0);
        assert_eq!(parse_durasi(""), 0);
    }
}
