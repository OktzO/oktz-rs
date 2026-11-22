//! Pencarian YT Music (katalog musik) untuk `play`/`play2`.
//!
//! `ponytail:` pengganti `ytmusic-api` (npm) — InnerTube `WEB_REMIX`
//! langsung, tanpa dependensi baru. Padanan `ytmusicapi` python:
//! `POST music.youtube.com/youtubei/v1/search` dengan `params` filter
//! songs, parse `musicResponsiveListItemRenderer`. Ini fokus musik:
//! tidak ada vlog/podcast/episode. Bila YT Music gagal/kosong, pemanggil
//! jatuh ke `ytsearch` dengan filter durasi lagu.
//!
//! `ponytail:` `X-Goog-Visitor-Id` dicache dengan TTL 24 jam + batas
//! pakai 100x per ID; fetch kosong/tak-sahih tidak mengisi cache (cache
//! basi yang sahih tetap dipakai). Upgrade path: refresh proaktif bila
//! Google mulai menolak ID basi.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime};

use oktz_plugin_api::{HttpCall, HttpError, HttpGet};
use serde_json::{Value, json};

use super::ytsearch::Video;

const YTM_DOMAIN: &str = "https://music.youtube.com";
const YTM_KEY: &str = "AIzaSyC9XL3ZjWddXya6X74dJoCTL-WEYFDNX30";
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:88.0) Gecko/20100101 Firefox/88.0";
/// `get_search_params("songs")` — `EgWKAQ` + `II` + `AWoKEAkQBRAKEAM`.
/// `ponytail:` params baru ytmusicapi `AWoMEA4QChADEAQQCRAF` deprecated
/// 2026-11 (Google balas "No results" HTTP 200); params lama masih hidup.
const PARAMS_SONGS: &str = "EgWKAQIIAWoKEAkQBRAKEAM";
/// `initialize_context()` — versi tetap, tanpa tanggal dinamis.
const CLIENT_VERSION: &str = "1.20250101.01.00";

/// TTL cache `X-Goog-Visitor-Id`: 24 jam, lalu fetch ulang.
const VISITOR_TTL: Duration = Duration::from_secs(24 * 60 * 60);
/// Batas pakai per ID: 100 — ID lama dipaksa refresh meski TTL belum lewat.
const VISITOR_MAX_PAKAI: u32 = 100;
/// ID sahih = tidak kosong dan panjang >= 10.
const VISITOR_MIN_PANJANG: usize = 10;

struct EntriVisitor {
    nilai: String,
    kedaluwarsa: SystemTime,
    pakai: u32,
}

fn cache_visitor() -> &'static Mutex<Option<EntriVisitor>> {
    static C: OnceLock<Mutex<Option<EntriVisitor>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

/// Bentuk ID yang layak dicache/dipakai.
fn visitor_sahih(nilai: &str) -> bool {
    !nilai.trim().is_empty() && nilai.len() >= VISITOR_MIN_PANJANG
}

/// Cache masih bisa dipakai: sahih, belum lewat TTL, pakai < batas.
fn visitor_bisa_dipakai(e: &EntriVisitor) -> bool {
    visitor_sahih(&e.nilai) && SystemTime::now() < e.kedaluwarsa && e.pakai < VISITOR_MAX_PAKAI
}

/// `get_visitor_id()` — `ytcfg.set({...});` → `VISITOR_DATA`.
///
/// Hasil dicache (TTL 24 jam, maks 100x pakai) supaya tiap pencarian tidak
/// mengulang GET homepage `music.youtube.com`. Fetch gagal/kosong/tak-sahih
/// tidak mengisi cache: cache basi yang sahih tetap dipakai, bila tidak
/// request jalan tanpa header itu.
fn visitor_id(http: &dyn HttpGet) -> String {
    if let Ok(mut c) = cache_visitor().lock()
        && let Some(e) = c.as_mut()
        && visitor_bisa_dipakai(e)
    {
        e.pakai += 1;
        return e.nilai.clone();
    }
    let baru = ambil_visitor_baru(http);
    if visitor_sahih(&baru) {
        if let Ok(mut c) = cache_visitor().lock() {
            *c = Some(EntriVisitor {
                nilai: baru.clone(),
                kedaluwarsa: SystemTime::now() + VISITOR_TTL,
                pakai: 1,
            });
        }
        return baru;
    }
    cache_visitor()
        .lock()
        .ok()
        .and_then(|c| {
            c.as_ref()
                .filter(|e| visitor_sahih(&e.nilai))
                .map(|e| e.nilai.clone())
        })
        .unwrap_or_default()
}

/// Satu GET homepage → `VISITOR_DATA`, `""` bila tak dapat.
fn ambil_visitor_baru(http: &dyn HttpGet) -> String {
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

/// Hanya untuk test: kosongkan cache antar skenario.
#[cfg(test)]
pub(crate) fn visitor_cache_bersihkan() {
    if let Ok(mut c) = cache_visitor().lock() {
        *c = None;
    }
}

/// Hanya untuk test: suntik entri cache (umur/kedaluwarsa + pakai palsu).
#[cfg(test)]
pub(crate) fn visitor_cache_suntik(nilai: &str, kedaluwarsa: SystemTime, pakai: u32) {
    if let Ok(mut c) = cache_visitor().lock() {
        *c = Some(EntriVisitor {
            nilai: nilai.to_string(),
            kedaluwarsa,
            pakai,
        });
    }
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
    if item.is_empty() {
        // Top result (musicCardShelfRenderer) — bentuk search tanpa hasil
        // shelf; satu item, sama kolomnya.
        if let Some(card) = data
            .get("contents")
            .and_then(|c| c.pointer("/tabbedSearchResultsRenderer"))
            .and_then(cari_card)
        {
            item.push(card);
        }
    }
    Ok(item.iter().filter_map(|v| lagu_dari_item(v)).collect())
}

/// Cari `musicCardShelfRenderer` (top result) di mana pun.
fn cari_card(node: &Value) -> Option<&Value> {
    match node {
        Value::Object(m) => {
            if let Some(c) = m.get("musicCardShelfRenderer") {
                return Some(c);
            }
            m.values().find_map(cari_card)
        }
        Value::Array(a) => a.iter().find_map(cari_card),
        _ => None,
    }
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

/// `lagu_dari_item` — kolom kedua bentuk card/shelf bisa `Video • artist •
/// 234M views • 3:59`; run views (`N views`) bukan artist.
fn is_views(teks: &str) -> bool {
    let t = teks.trim();
    let Some(i) = t.rfind(' ') else { return false };
    let angka = &t[..i];
    // Suffix magnitudo K/M/B boleh (234M, 3.1K).
    let inti = angka
        .trim_end_matches(|c: char| c.eq_ignore_ascii_case(&'k') || c.eq_ignore_ascii_case(&'m') || c.eq_ignore_ascii_case(&'b'));
    !inti.is_empty()
        && inti
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
        && (t[i + 1..].eq_ignore_ascii_case("views")
            || t[i + 1..].eq_ignore_ascii_case("plays"))
}

fn lagu_dari_item(item: &Value) -> Option<Video> {
    let video_id = video_id_dari_item(item)?;
    if !pola_id(&video_id) {
        return None;
    }
    let flex = item.get("flexColumns").and_then(Value::as_array);
    fn runs_of(v: &Value) -> Vec<Option<&Value>> {
        v.get("runs")
            .and_then(Value::as_array)
            .map(|a| a.iter().map(Some).collect())
            .unwrap_or_default()
    }
    let title = flex
        .and_then(|f| f.first())
        .map(|k| runs_teks(kolom_runs(k)))
        .or_else(|| item.get("title").map(|t| runs_teks(runs_of(t))))
        .unwrap_or_default();
    if title.is_empty() {
        return None;
    }
    // `parse_song_runs` — flexColumns[1]: [Song • artist • album • durasi].
    let mut artist: Vec<String> = Vec::new();
    let mut duration = String::new();
    let runs_kedua = flex
        .and_then(|f| f.get(1))
        .map(kolom_runs)
        .or_else(|| item.get("subtitle").map(runs_of));
    if let Some(runs) = runs_kedua {
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
            } else if is_tahun(teks) || is_views(teks) {
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
            Self::dengan_halaman(
                body,
                r#"<script>ytcfg.set({"VISITOR_DATA":"visitor-123"});</script>"#.to_string(),
            )
        }
        fn dengan_halaman(body: String, halaman: String) -> Arc<Stub> {
            Arc::new(Stub {
                body,
                permintaan: Mutex::new(Vec::new()),
                halaman,
            })
        }
    }

    /// Serialisasi test cache di file ini (cache global, anti balapan).
    fn kunci_cache() -> &'static Mutex<()> {
        static K: OnceLock<Mutex<()>> = OnceLock::new();
        K.get_or_init(|| Mutex::new(()))
    }

    fn hitung_get(p: &[HttpCall]) -> usize {
        p.iter().filter(|r| r.method == HttpMethod::Get).count()
    }

    fn header_post(p: &[HttpCall], nama: &str) -> Option<String> {
        p.iter()
            .find(|r| r.method == HttpMethod::Post)
            .and_then(|r| {
                r.headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(nama))
                    .map(|(_, v)| v.clone())
            })
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
        let _g = kunci_cache().lock().unwrap();
        visitor_cache_bersihkan();
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
        // Cache dikosongkan → GET homepage visitor dulu, lalu POST search.
        assert_eq!(hitung_get(&p), 1, "visitor id diambil sekali: {p:?}");
        for r in p.iter().filter(|r| r.method == HttpMethod::Get) {
            assert!(r.url.starts_with(YTM_DOMAIN), "GET: {}", r.url);
        }
        let post = p
            .iter()
            .find(|r| r.method == HttpMethod::Post)
            .expect("POST search");
        assert!(post.url.contains("music.youtube.com/youtubei/v1/search"));
        assert!(post.url.contains("alt=json"));
        assert_eq!(
            header_post(&p, "x-goog-visitor-id").as_deref(),
            Some("visitor-123"),
            "header visitor dikirim"
        );
        let badan = String::from_utf8(post.body.as_ref().unwrap().bytes().to_vec()).unwrap();
        assert!(badan.contains("WEB_REMIX"), "{badan}");
        assert!(badan.contains("EgWKAQIIAWoKEAkQBRAKEAM"), "{badan}");
        assert!(badan.contains("lagu uji"), "{badan}");
    }

    #[test]
    fn cache_hit_tidak_refetch() {
        let _g = kunci_cache().lock().unwrap();
        visitor_cache_bersihkan();
        visitor_cache_suntik(
            "visitor-123",
            SystemTime::now() + Duration::from_secs(3600),
            1,
        );
        let http = Stub::baru(HASIL.to_string());
        search_lagu(http.as_ref(), "lagu uji").unwrap();
        search_lagu(http.as_ref(), "lagu uji").unwrap();
        let p = http.permintaan.lock().unwrap();
        assert_eq!(hitung_get(&p), 0, "cache hit: tanpa GET homepage: {p:?}");
        assert_eq!(
            header_post(&p, "x-goog-visitor-id").as_deref(),
            Some("visitor-123")
        );
    }

    #[test]
    fn cache_expire_refetch() {
        let _g = kunci_cache().lock().unwrap();
        visitor_cache_bersihkan();
        visitor_cache_suntik(
            "visitor-123",
            SystemTime::now() - Duration::from_secs(3600),
            1,
        );
        let http = Stub::baru(HASIL.to_string());
        search_lagu(http.as_ref(), "lagu uji").unwrap();
        let p = http.permintaan.lock().unwrap();
        assert_eq!(hitung_get(&p), 1, "expire → refetch sekali: {p:?}");
    }

    #[test]
    fn cache_habis_pakai_refetch() {
        let _g = kunci_cache().lock().unwrap();
        visitor_cache_bersihkan();
        visitor_cache_suntik(
            "visitor-123",
            SystemTime::now() + Duration::from_secs(3600),
            VISITOR_MAX_PAKAI,
        );
        let http = Stub::baru(HASIL.to_string());
        search_lagu(http.as_ref(), "lagu uji").unwrap();
        let p = http.permintaan.lock().unwrap();
        assert_eq!(hitung_get(&p), 1, "pakai habis → refetch: {p:?}");
    }

    #[test]
    fn visitor_tidak_sahih_tidak_dicache() {
        let _g = kunci_cache().lock().unwrap();
        visitor_cache_bersihkan();
        // ID di halaman < 10 char → tak-sahih, tidak dicache.
        let http = Stub::dengan_halaman(
            HASIL.to_string(),
            r#"<script>ytcfg.set({"VISITOR_DATA":"pendek"});</script>"#.to_string(),
        );
        search_lagu(http.as_ref(), "lagu uji").unwrap();
        search_lagu(http.as_ref(), "lagu uji").unwrap();
        let p = http.permintaan.lock().unwrap();
        assert_eq!(hitung_get(&p), 2, "tak-sahih → GET tiap cari: {p:?}");
        assert_eq!(header_post(&p, "x-goog-visitor-id"), None);
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
    fn top_result_card_dipakai_bila_shelf_kosong() {
        let http = Stub::baru(
            r#"{"contents":{"tabbedSearchResultsRenderer":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[{"itemSectionRenderer":{"contents":[{"musicCardShelfRenderer":{"title":{"runs":[{"text":"Lagu Kartu","navigationEndpoint":{"watchEndpoint":{"videoId":"bbbbbbbbbbb"}}}]},"subtitle":{"runs":[{"text":"Video"},{"text":" • "},{"text":"Raim Laode"},{"text":" • "},{"text":"234M views"},{"text":" • "},{"text":"3:59"}]},"thumbnail":{"musicThumbnailRenderer":{"thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/b/hq.jpg"}]}}}}}]}}]}}}}]}}}"#
                .to_string(),
        );
        let v = search_lagu(http.as_ref(), "lagu kartu").unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].video_id, "bbbbbbbbbbb");
        assert_eq!(v[0].title, "Lagu Kartu");
        assert_eq!(v[0].author_name, "Raim Laode", "views bukan artist");
        assert_eq!(v[0].timestamp, "3:59");
        assert_eq!(v[0].seconds, 239);
    }

    #[test]
    fn hasil_kosong_mengembalikan_vec_kosong() {
        let http = Stub::baru(
            r#"{"contents":{"tabbedSearchResultsRenderer":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[{"itemSectionRenderer":{"contents":[{"messageRenderer":{"text":{"runs":[{"text":"No results for "},{"text":"zzz"}]}}}]}}]}}}}]}}}"#
                .to_string(),
        );
        assert!(search_lagu(http.as_ref(), "zzz").unwrap().is_empty());
    }

    #[test]
    fn klasifikasi_durasi_dan_tahun() {
        assert!(is_durasi("3:33") && is_durasi("1:02:03") && !is_durasi("Song"));
        assert!(is_tahun("2024") && !is_tahun("2024:01"));
        assert!(is_views("234M views") && is_views("3.1M views") && !is_views("Raim Laode"));
        assert_eq!(parse_durasi("3:33"), 213);
    }
}
