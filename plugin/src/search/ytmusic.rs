//! Pencarian YT Music (katalog musik) untuk `play`/`play2`.
//!
//! `ponytail:` pengganti `ytmusic-api` (npm) — InnerTube `WEB_REMIX`
//! langsung, tanpa dependensi baru. Padanan `ytmusicapi` python:
//! `POST music.youtube.com/youtubei/v1/search` dengan `params` filter
//! songs, parse `musicResponsiveListItemRenderer`. Ini fokus musik:
//! tidak ada vlog/podcast/episode. Bila YT Music gagal/kosong, pemanggil
//! jatuh ke `ytsearch` dengan filter durasi lagu.
//!
//! `ponytail:` `X-Goog-Visitor-Id` dicache tanpa TTL; upgrade path:
//! simpan waktu + refresh periodik bila Google mulai menolak ID basi.

use oktz_plugin_api::{HttpCall, HttpError, HttpGet};
use serde_json::{Value, json};

use super::ytsearch::Video;

const YTM_DOMAIN: &str = "https://music.youtube.com";
const YTM_KEY: &str = "AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30";
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";
/// `get_search_params("songs")` — `EgWKAQ` + `II` + `AWoMEA4QChADEAQQCRAF`.
const PARAMS_SONGS: &str = "EgWKAQIIAWoMEA4QChADEAQQCRAF";
/// `initialize_context()` — versi tetap, tanpa tanggal dinamis.
const CLIENT_VERSION: &str = "1.20250101.01.00";

/// `get_visitor_id()` — `ytcfg.set({...});` → `VISITOR_DATA`.
/// Gagal ambil = string kosong, request tetap jalan tanpa header itu.
/// ponytail: tanpa cache; upgrade path: cache + TTL bila traffic tinggi.
fn visitor_id(http: &dyn HttpGet) -> String {
    let v = http
        .call(
            &HttpCall::get(YTM_DOMAIN)
                .header("user-agent", USER_AGENT)
                .header("accept", "*/*"),
        )
        .map(|r| r.text())
        .unwrap_or_default();
    ambil_visitor_data(&v)
}

/// Ambil `"VISITOR_DATA":"..."` dari `ytcfg.set(...)` tanpa regex.
fn ambil_visitor_data(html: &str) -> String {
    let Some(awal) = html.find("VISITOR_DATA") else {
        return String::new();
    };
    let sisa = &html[awal..];
    let Some(kolon) = sisa.find(':') else {
        return String::new();
    };
    let sisa = sisa[kolon + 1..].trim_start();
    // Bisa `"value"` atau `'value'`.
    for kutip in ['"', '\''] {
        if let Some(rest) = sisa.strip_prefix(kutip) {
            return rest.split(kutip).next().unwrap_or("").to_string();
        }
    }
    String::new()
}

/// `search_lagu(q)` — padanan `ytmusic.search(q, filter="songs")`.
/// Mengembalikan daftar kosong bila tidak ada lagu; error jaringan dilempar.
pub(crate) fn search_lagu(http: &dyn HttpGet, q: &str) -> Result<Vec<Video>, HttpError> {
    let visitor = visitor_id(http);
    let mut req = HttpCall::post(format!("{YTM_DOMAIN}/youtubei/v1/search?alt=json&key={YTM_KEY}"))
        .header("user-agent", USER_AGENT)
        .header("accept", "*/*")
        .header("accept-encoding", "gzip, deflate")
        .header("content-type", "application/json")
        .header("content-encoding", "gzip")
        .header("origin", YTM_DOMAIN)
        .json(&json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": CLIENT_VERSION,
                },
                "user": {}
            },
            "query": q,
            "params": PARAMS_SONGS,
        }));
    if !visitor.is_empty() {
        req = req.header("x-goog-visitor-id", visitor);
    }
    let res = http.call(&req)?;
    if !(200..300).contains(&res.status) {
        return Err(HttpError {
            message: format!("Pencarian YT Music gagal HTTP {}", res.status),
        });
    }
    let data = res.json().unwrap_or(Value::Null);
    let mut item: Vec<&Value> = Vec::new();
    kumpulkan_item(data.get("contents"), &mut item);
    Ok(item.iter().filter_map(|v| lagu_dari_item(v)).collect())
}

/// Kumpulkan `musicResponsiveListItemRenderer` di mana pun.
fn kumpulkan_item<'a>(node: Option<&'a Value>, out: &mut Vec<&'a Value>) {
    let Some(node) = node.filter(|v| v.is_object() || v.is_array()) else {
        return;
    };
    if let Some(r) = node.get("musicResponsiveListItemRenderer") {
        out.push(r);
        return;
    }
    match node {
        Value::Array(a) => {
            for v in a {
                kumpulkan_item(Some(v), out);
            }
        }
        Value::Object(m) => {
            for v in m.values() {
                kumpulkan_item(Some(v), out);
            }
        }
        _ => {}
    }
}

fn lagu_dari_item(item: &Value) -> Option<Video> {
    let video_id = video_id_dari_item(item)?;
    if !pola_id(&video_id) {
        return None;
    }
    let flex = item.get("flexColumns")?.as_array()?;
    let title = runs_teks(kolom_runs(flex.first()?));
    if title.is_empty() {
        return None;
    }
    // `parse_song_runs` — flexColumns[1]: [Song • artist • album • durasi].
    let mut artist: Vec<String> = Vec::new();
    let mut duration = String::new();
    if let Some(kedua) = flex.get(1) {
        let runs = kolom_runs(kedua);
        // Lewati penanda tipe ("Song") bila diikuti • dan run berikutnya
        // terkelasifikasi sebagai artist/duration/year (lihat songs.py).
        let mulai = if runs.len() > 2
            && runs[1].and_then(|r| r.get("text")).and_then(Value::as_str) == Some(" • ")
            && runs[0].and_then(|r| r.get("navigationEndpoint")).is_none()
        {
            2
        } else {
            0
        };
        for (i, run) in runs.iter().enumerate() {
            if i < mulai || i % 2 == 1 {
                continue;
            }
            let Some(run) = run else { continue };
            let teks = run.get("text").and_then(Value::as_str).unwrap_or("");
            if teks.is_empty() {
                continue;
            }
            if is_durasi(teks) {
                duration = teks.to_string();
            } else if is_tahun(teks) {
                continue;
            } else {
                // Album = browseId MPRE; jangan jadikan artist.
                if run
                    .get("navigationEndpoint")
                    .and_then(|n| n.get("browseEndpoint"))
                    .and_then(|b| b.get("browseId"))
                    .and_then(Value::as_str)
                    .is_some_and(|id| id.starts_with("MPRE"))
                {
                    continue;
                }
                artist.push(teks.to_string());
            }
        }
    }
    let seconds = parse_durasi(&duration);
    let thumbnail = item
        .get("thumbnail")
        .and_then(|t| t.get("musicThumbnailRenderer"))
        .and_then(|t| t.get("thumbnail"))
        .and_then(|t| t.get("thumbnails"))
        .and_then(Value::as_array)
        .and_then(|a| a.last())
        .and_then(|t| t.get("url"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some(Video {
        url: format!("https://www.youtube.com/watch?v={video_id}"),
        title,
        thumbnail,
        description: String::new(),
        video_id,
        views: 0,
        ago: String::new(),
        timestamp: duration,
        seconds,
        author_name: artist.join(", "),
        author_username: String::new(),
        author_url: String::new(),
    })
}

fn kolom_runs(kolom: &Value) -> Vec<Option<&Value>> {
    kolom
        .get("musicResponsiveListItemFlexColumnRenderer")
        .and_then(|k| k.get("text"))
        .and_then(|t| t.get("runs"))
        .and_then(Value::as_array)
        .map(|a| a.iter().map(Some).collect())
        .unwrap_or_default()
}

fn runs_teks(runs: Vec<Option<&Value>>) -> String {
    runs.iter()
        .filter_map(|r| r.and_then(|v| v.get("text")).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

/// Cari `videoId` di `watchEndpoint` dalam item (overlay play button atau
/// navigationEndpoint run).
fn video_id_dari_item(item: &Value) -> Option<String> {
    // overlay.musicPlayButtonRenderer.playNavigationEndpoint.watchEndpoint.videoId
    let id = item
        .get("overlay")
        .and_then(|o| o.get("musicItemThumbnailOverlayRenderer"))
        .and_then(|o| o.get("content"))
        .and_then(|o| o.get("musicPlayButtonRenderer"))
        .and_then(|o| o.get("playNavigationEndpoint"))
        .and_then(|o| o.get("watchEndpoint"))
        .and_then(|o| o.get("videoId"))
        .and_then(Value::as_str);
    if let Some(id) = id {
        return Some(id.to_string());
    }
    // Fallback: navigasi di runs / item itu sendiri.
    cari_watch_id(item)
}

fn cari_watch_id(node: &Value) -> Option<String> {
    match node {
        Value::Object(m) => {
            if let Some(id) = m
                .get("watchEndpoint")
                .and_then(|w| w.get("videoId"))
                .and_then(Value::as_str)
            {
                return Some(id.to_string());
            }
            for v in m.values() {
                if let Some(id) = cari_watch_id(v) {
                    return Some(id);
                }
            }
            None
        }
        Value::Array(a) => a.iter().find_map(cari_watch_id),
        _ => None,
    }
}

fn pola_id(s: &str) -> bool {
    s.len() == 11
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn is_durasi(teks: &str) -> bool {
    // `^(\d+:)*\d+:\d+$`
    let t = teks.trim();
    if !t.contains(':') {
        return false;
    }
    t.split(':').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn is_tahun(teks: &str) -> bool {
    teks.len() == 4 && teks.chars().all(|c| c.is_ascii_digit())
}

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
        halaman: String,
    }

    impl Stub {
        fn baru(body: String) -> Arc<Stub> {
            Arc::new(Stub {
                body,
                permintaan: Mutex::new(Vec::new()),
                halaman: format!(
                    r#"<script>ytcfg.set({{"VISITOR_DATA":"visitor-123"}});</script>"#
                ),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.permintaan
                .lock()
                .unwrap()
                .push(HttpCall::get(url.to_string()));
            let teks = if url.contains("music.youtube.com") && !url.contains("youtubei") {
                self.halaman.clone()
            } else {
                self.body.clone()
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(teks.into_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn post_json(&self, url: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            self.permintaan
                .lock()
                .unwrap()
                .push(HttpCall::post(url.to_string()));
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: Vec::new(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma POST JSON")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma POST JSON")
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.permintaan.lock().unwrap().push(req.clone());
            let teks = if req.method == HttpMethod::Get {
                self.halaman.clone()
            } else {
                self.body.clone()
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(teks.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: Vec::new(),
            })
        }
    }

    const HASIL: &str = r#"{"contents":{"tabbedSearchResultsRenderer":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[{"musicShelfRenderer":{"contents":[{"musicResponsiveListItemRenderer":{"flexColumns":[{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":"Lagu Uji"}]}}},{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":"Song"},{"text":" • "},{"text":"Artist Uji","navigationEndpoint":{"browseEndpoint":{"browseId":"UC123"}}},{"text":" • "},{"text":"Album Uji","navigationEndpoint":{"browseEndpoint":{"browseId":"MPREabc"}}},{"text":" • "},{"text":"3:33"}]}}}],"thumbnail":{"musicThumbnailRenderer":{"thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/a/low.jpg"},{"url":"https://i.ytimg.com/vi/a/hq.jpg"}]}}},"overlay":{"musicItemThumbnailOverlayRenderer":{"content":{"musicPlayButtonRenderer":{"playNavigationEndpoint":{"watchEndpoint":{"videoId":"aaaaaaaaaaa"}}}}}}}}]}}]}}}}]}}}"#;

    #[test]
    fn visitor_data_dibaca_dari_ytcfg_tanpa_regex() {
        assert_eq!(
            ambil_visitor_data(r#"ytcfg.set({"VISITOR_DATA":"abc-123"});"#),
            "abc-123"
        );
        assert_eq!(ambil_visitor_data("tidak ada"), "");
    }

    #[test]
    fn hasil_search_memetakan_lagu_dari_item_yt_music() {
        let http = Stub::baru(HASIL.to_string());
        let v = search_lagu(http.as_ref(), "lagu uji").unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].video_id, "aaaaaaaaaaa");
        assert_eq!(v[0].url, "https://www.youtube.com/watch?v=aaaaaaaaaaa");
        assert_eq!(v[0].title, "Lagu Uji");
        assert_eq!(v[0].author_name, "Artist Uji", "album MPRE bukan artist");
        assert_eq!(v[0].timestamp, "3:33");
        assert_eq!(v[0].seconds, 213);
        assert!(v[0].thumbnail.ends_with("hq.jpg"));

        let p = http.permintaan.lock().unwrap();
        assert_eq!(p[0].method, HttpMethod::Get, "visitor id dulu");
        assert!(p[0].url.starts_with(YTM_DOMAIN));
        let post = &p[1];
        assert_eq!(post.method, HttpMethod::Post);
        assert!(post.url.contains("music.youtube.com/youtubei/v1/search"));
        assert!(post.url.contains("alt=json"));
        let badan = String::from_utf8(post.body.as_ref().unwrap().bytes().to_vec()).unwrap();
        assert!(badan.contains("WEB_REMIX"), "{badan}");
        assert!(badan.contains("EgWKAQIIAWoMEA4QChADEAQQCRAF"), "{badan}");
        assert!(badan.contains("lagu uji"), "{badan}");
    }

    #[test]
    fn item_tanpa_title_atau_id_invalid_dibuang() {
        let http = Stub::baru(
            r#"{"contents":{"sectionListRenderer":{"contents":[{"musicResponsiveListItemRenderer":{"flexColumns":[{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":""}]}}}]}}]}}}"#
                .to_string(),
        );
        assert!(search_lagu(http.as_ref(), "x").unwrap().is_empty());
    }

    #[test]
    fn klasifikasi_durasi_dan_tahun() {
        assert!(is_durasi("3:33") && is_durasi("1:02:03") && !is_durasi("Song"));
        assert!(is_tahun("2024") && !is_tahun("2024:01"));
        assert_eq!(parse_durasi("3:33"), 213);
    }
}
