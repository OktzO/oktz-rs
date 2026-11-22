//! `play.js` (akar repo) — sistem ytmp3 baru: pencarian YouTube via halaman
//! `results` (`ytInitialData`), unduhan SaveTube (tanpa retry) dan ytmp3.ing.
//!
//! Yang disalin dari JS:
//! - `YouTubeSearch.search`: GET `www.youtube.com/results?search_query=`,
//!   ekstrak `var ytInitialData = {...};` (dua pola), parse
//!   `twoColumnSearchResultsRenderer…videoRenderer`.
//! - `SavetubeDownloader`: `random-cdn` → `POST {cdn}/v2/info` → dekrip
//!   AES-128-CBC (`ky` = kunci yang sama) → `POST {cdn}/download` — TANPA
//!   retry loop (beda dari `src/scraper/youtube.js` yang 4x).
//! - `Ytmp3IngDownloader`: GET homepage (cookie + CSRF `value="…"`) →
//!   POST `/audio` multipart `url=…` → `response.data.url` base64 →
//!   `downloadUrl`, `filename`.
//! - `ytmp3(query)`: video PERTAMA, backend berurutan savetube → ytmp3ing;
//!   semua gagal = error "Semua backend gagal".
//!
//! Beda: `CookieJar` axios tidak ada — cookie diambil dari header
//! `set-cookie` respons (cukup: ytmp3.ing hanya butuh cookie sesi itu).

use oktz_plugin_api::{HttpCall, HttpError, HttpGet, MultipartPart};
use serde_json::{Map, Value, json};

use crate::common::{field, truthy};

const UA_CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
/// `this.cryptoKey` (`play.js:50`) — kunci AES-128 SaveTube.
const KY: &str = "C5D58EF67A7584E4A29F6C35BBC4EB12";

/// Satu hasil `YouTubeSearch._parseResults` (`play.js:34-44`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoHasil {
    pub id: String,
    pub url: String,
    pub title: String,
    pub author: String,
    pub views: String,
    pub duration: String,
    pub thumbnail: String,
    pub published: String,
}

/// Hasil `ytmp3(query)` — `{ status, backend, video, download }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasilYtmp3 {
    pub backend: &'static str,
    pub video: VideoHasil,
    /// `{ title, thumbnail, duration, downloadUrl, format }`.
    pub download: Value,
}

/// `YouTubeSearch.search` (`play.js:11-25`).
///
/// `query` di-encode seperti `params: { search_query }` axios — spasi jadi
/// `+` (bukan `%20`), sisanya percent-encoding.
pub fn youtube_search(http: &dyn HttpGet, query: &str) -> Result<Vec<VideoHasil>, HttpError> {
    let q = query.replace(' ', "+");
    let html = http
        .call(
            &HttpCall::get(format!(
                "https://www.youtube.com/results?search_query={q}"
            ))
            .header("accept", "*/*")
            .header("accept-language", "en-US,en;q=0.9")
            .header("user-agent", UA_CHROME)
            .with_timeout(60_000),
        )?
        .text();
    let data = ambil_ytinitial(&html)?;
    Ok(parse_hasil(&data))
}

/// `html.match(/var ytInitialData = (\{[\s\S]*?\});/)` dengan pola kedua
/// `var ytInitialData\s*=\s*(\{[\s\S]*?\});\s*var` (`play.js:21-22`).
///
/// Regex JS non-greedy `.*?` + terminator: dicari di sini dengan pencarian
/// kurung seimbang sejak `{` pertama setelah penanda — setara dan lebih
/// murah daripada `[\s\S]*?` yang bisa berhenti di `}` pertama di dalam string.
fn ambil_ytinitial(html: &str) -> Result<Value, HttpError> {
    // Dua regex JS sama-sama berakhir di `{` pertama setelah penanda.
    let mulai = html
        .find("var ytInitialData")
        .and_then(|i| html[i..].find('{').map(|j| i + j))
        .ok_or_else(|| HttpError {
            message: "Gagal parse YouTube data".to_string(),
        })?;
    let sisa = &html[mulai..];
    let akhir = kurung_tutup(sisa).ok_or(HttpError {
        message: "Gagal parse YouTube data".to_string(),
    })?;
    serde_json::from_str(&sisa[..=akhir]).map_err(|e| HttpError {
        message: format!("Gagal parse YouTube data: {e}"),
    })
}

/// Posisi `}` yang menutup objek mulai dari indeks 0 `{`, dengan penghitungan
/// di luar string (`"…}"` tidak dihitung).
fn kurung_tutup(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut dalam_string = false;
    let mut escape = false;
    let mut dalam_str_lit = false; // '…'
    let mut kedalaman = 0usize;
    for (i, &c) in b.iter().enumerate() {
        if dalam_string {
            if escape {
                escape = false;
            } else if c == b'\\' {
                escape = true;
            } else if c == b'"' {
                dalam_string = false;
            }
            continue;
        }
        if dalam_str_lit {
            if escape {
                escape = false;
            } else if c == b'\\' {
                escape = true;
            } else if c == b'\'' {
                dalam_str_lit = false;
            }
            continue;
        }
        match c {
            b'"' => dalam_string = true,
            b'\'' => dalam_str_lit = true,
            b'{' => kedalaman += 1,
            b'}' => {
                kedalaman -= 1;
                if kedalaman == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// `_parseResults` (`play.js:27-45`).
pub fn parse_hasil(data: &Value) -> Vec<VideoHasil> {
    let Some(contents) = data.pointer("/contents/twoColumnSearchResultsRenderer/primaryContents/sectionListRenderer/contents/0/itemSectionRenderer/contents") else {
        return Vec::new();
    };
    let Some(daftar) = contents.as_array() else {
        return Vec::new();
    };
    daftar
        .iter()
        .filter_map(|c| c.get("videoRenderer"))
        .map(|d| {
            let id = d.get("videoId").and_then(Value::as_str).unwrap_or("").to_string();
            VideoHasil {
                url: format!("https://www.youtube.com/watch?v={id}"),
                id,
                title: d.pointer("/title/runs/0/text").and_then(Value::as_str).unwrap_or("").to_string(),
                author: d.pointer("/ownerText/runs/0/text").and_then(Value::as_str).unwrap_or("").to_string(),
                views: d.pointer("/viewCountText/simpleText").and_then(Value::as_str).unwrap_or("").to_string(),
                duration: d
                    .pointer("/lengthText/accessibility/accessibilityData/label")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                thumbnail: d
                    .pointer("/thumbnail/thumbnails")
                    .and_then(Value::as_array)
                    .and_then(|a| a.last())
                    .and_then(|t| t.get("url"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                published: d
                    .pointer("/publishedTimeText/simpleText")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            }
        })
        .collect()
}

/// `SavetubeDownloader.download` (`play.js:71-90`) — TANPA retry loop.
///
/// Bentuk keluaran: `{ title, thumbnail, duration, downloadUrl, format:"mp3" }`
/// (`duration` = `info.durationLabel`, hilang bila tidak ada seperti JS).
pub fn savetube_download(http: &dyn HttpGet, video_url: &str, quality: &str) -> Result<Value, HttpError> {
    let cdn = ambil_cdn(http)?;
    let info_res = http.call(
        &HttpCall::post(format!("https://{cdn}/v2/info"))
            .header("content-type", "application/json")
            .header("user-agent", UA_CHROME)
            .json(&json!({ "url": video_url }))
            .with_timeout(60_000),
    )?;
    let info_json = info_res.json().unwrap_or(Value::Null);
    if !truthy(info_json.get("status").unwrap_or(&Value::Null)) {
        return Err(HttpError {
            message: info_json
                .get("message")
                .map(Value::to_string)
                .unwrap_or_else(|| "\"Gagal ambil info video\"".to_string()),
        });
    }
    let info = dekrip_savetube(info_json.get("data").unwrap_or(&Value::Null))?;

    let dl_res = http.call(
        &HttpCall::post(format!("https://{cdn}/download"))
            .header("content-type", "application/json")
            .header("user-agent", UA_CHROME)
            .json(&json!({
                "downloadType": "audio",
                "quality": quality,
                "key": info.get("key").cloned().unwrap_or(Value::Null),
            }))
            .with_timeout(60_000),
    )?;
    let dl = dl_res.json().unwrap_or(Value::Null);
    let download_url = field(&dl, "data.downloadUrl").and_then(Value::as_str).unwrap_or("").to_string();

    let mut out = Map::new();
    out.insert("title".to_string(), info.get("title").cloned().unwrap_or(Value::Null));
    out.insert("thumbnail".to_string(), info.get("thumbnail").cloned().unwrap_or(Value::Null));
    out.insert("duration".to_string(), info.get("durationLabel").cloned().unwrap_or(Value::Null));
    out.insert("downloadUrl".to_string(), Value::String(download_url));
    out.insert("format".to_string(), json!("mp3"));
    Ok(Value::Object(out))
}

fn ambil_cdn(http: &dyn HttpGet) -> Result<String, HttpError> {
    let res = http.get("https://media.savetube.vip/api/random-cdn")?;
    let data = res.json().unwrap_or(Value::Null);
    Ok(data
        .get("cdn")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string())
}

/// `Ytmp3IngDownloader.download` (`play.js:94-119`).
pub fn ytmp3ing_download(http: &dyn HttpGet, video_url: &str) -> Result<Value, HttpError> {
    let res = http.call(
        &HttpCall::get("https://ytmp3.ing/")
            .header("user-agent", UA_CHROME)
            .with_timeout(60_000),
    )?;
    // `res.headers["set-cookie"]?.join("; ")` — header bisa diulang.
    let cookie = res
        .headers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, v)| v.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    // `csrf = res.data.match(/value="([^"]+)"/)?.[1]` — input/hidden pertama.
    let html = res.text();
    let csrf = cari_value_attr(&html).ok_or(HttpError {
        message: "Gagal dapat CSRF token ytmp3.ing".to_string(),
    })?;

    let res = http.call(
        &HttpCall::post("https://ytmp3.ing/audio")
            .header("x-csrftoken", &csrf)
            .header("cookie", &cookie)
            .multipart(vec![MultipartPart::field("url", video_url)])
            .with_timeout(60_000),
    )?;
    let data = res.json().unwrap_or(Value::Null);
    let enc = data.get("url").and_then(Value::as_str).unwrap_or_default();
    // `Buffer.from(encryptedUrl, "base64").toString("utf-8")` — decoder Node.
    let download_url = String::from_utf8_lossy(&base64_node(enc)).to_string();

    let mut out = Map::new();
    out.insert(
        "title".to_string(),
        data.get("filename").cloned().unwrap_or(json!("Unknown")),
    );
    out.insert("downloadUrl".to_string(), Value::String(download_url));
    out.insert("format".to_string(), json!("mp3"));
    Ok(Value::Object(out))
}

/// `match(/value="([^"]+)"/)` — teks di antara `value="` dan `"` pertama.
fn cari_value_attr(html: &str) -> Option<String> {
    let i = html.find("value=\"")? + "value=\"".len();
    let sisa = &html[i..];
    let j = sisa.find('"')?;
    Some(sisa[..j].to_string())
}

/// `ytmp3(query, { quality = "128" })` (`play.js:122-152`).
///
/// Video pertama hasil pencarian, backend **berurutan** savetube → ytmp3ing;
/// semua gagal = `Err("Semua backend gagal. Error terakhir: …")`.
pub fn ytmp3(http: &dyn HttpGet, query: &str, quality: &str) -> Result<HasilYtmp3, HttpError> {
    let hasil = youtube_search(http, query)?;
    let video = hasil.first().ok_or(HttpError {
        message: "Tidak ada hasil pencarian".to_string(),
    })?;

    let mut terakhir: Option<HttpError> = None;
    // Backend berurutan savetube → ytmp3ing, seperti `backends` JS.
    for (nama, _) in [
        ("savetube", 1usize),
        ("ytmp3ing", 2),
    ] {
        let hasil_unduh = if nama == "savetube" {
            savetube_download(http, &video.url, quality)
        } else {
            ytmp3ing_download(http, &video.url)
        };
        match hasil_unduh {
            Ok(download) => {
                return Ok(HasilYtmp3 {
                    backend: nama,
                    video: video.clone(),
                    download,
                });
            }
            Err(e) => {
                terakhir = Some(e);
            }
        }
    }
    Err(HttpError {
        message: format!(
            "Semua backend gagal. Error terakhir: {}",
            terakhir.map(|e| e.message).unwrap_or_default()
        ),
    })
}

/// `Buffer.from(s, "base64")` Node — decoder yang memaafkan (alfabet URL-safe,
/// karakter asing dilewati, berhenti di `=`). Salinan `youtube.rs`.
fn base64_node(s: &str) -> Vec<u8> {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for c in s.bytes() {
        if c == b'=' {
            break;
        }
        let c = match c {
            b'-' => b'+',
            b'_' => b'/',
            lain => lain,
        };
        let Some(idx) = A.iter().position(|x| *x == c) else {
            continue;
        };
        acc = (acc << 6) | idx as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    out
}

/// Dekrip payload `/v2/info` — AES-128-CBC dengan kunci [`KY`], IV = 16 byte
/// pertama payload base64. Padanan `this._decrypt` (`play.js:58-69`); mesin
/// AES dipinjam dari modul SaveTube lama.
fn dekrip_savetube(enc: &Value) -> Result<Value, HttpError> {
    let Value::String(s) = enc else {
        return Err(HttpError {
            message: format!(
                "The first argument must be of type string or an instance of Buffer, \
                 ArrayBuffer, or Array or an Array-like Object. Received {enc:?}"
            ),
        });
    };
    let data = base64_node(s);
    let batas = data.len().min(16);
    let polos = crate::youtube::aes128_cbc_dekrip(&dari_hex(KY), &data[..batas], &data[batas..])?;
    serde_json::from_slice::<Value>(&polos).map_err(|e| HttpError {
        message: format!("JSON tidak valid dari SaveTube: {e}"),
    })
}

fn dari_hex(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len() / 2);
    let mut i = 0;
    while i + 1 < b.len() {
        if let (Some(hi), Some(lo)) = (
            (b[i] as char).to_digit(16),
            (b[i + 1] as char).to_digit(16),
        ) {
            out.push((hi * 16 + lo) as u8);
        }
        i += 2;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpResponse};
    use std::sync::{Arc, Mutex};

    /// Stub multi-URL: cocokkan dari potongan URL; `None` = transport error.
    #[derive(Debug)]
    struct Stub {
        per_url: Vec<(&'static str, Option<Value>)>,
        header_diset: Mutex<Vec<String>>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn per_url(per_url: Vec<(&'static str, Option<Value>)>) -> Arc<Stub> {
            Arc::new(Stub {
                per_url,
                header_diset: Mutex::new(Vec::new()),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, url: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::post(url))
        }
        fn post_form(&self, url: &str, _: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::post(url))
        }
        fn post_multipart(
            &self,
            url: &str,
            parts: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            // Multipart hanya menyuntik `url=` field — cukup lewat `call`.
            self.header_diset.lock().unwrap().push(parts[0].0.to_string());
            self.call(&HttpCall::post(url))
        }
        // Multipart lewat `call` — stub hanya mencatat nama part pertama.
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            if let Some(HttpBody::Multipart(parts)) = &req.body
                && let Some(p) = parts.first()
            {
                self.header_diset.lock().unwrap().push(p.name.clone());
            }
            let (_, nilai) = self
                .per_url
                .iter()
                .find(|(penanda, _)| req.url.contains(penanda))
                .ok_or(HttpError {
                    message: format!("URL tak dikenal stub: {}", req.url),
                })?;
            let headers = if req.url == "https://ytmp3.ing/" {
                vec![("set-cookie".into(), "sess=abc".into())]
            } else {
                Vec::new()
            };
            let text = match nilai.clone() {
                Some(Value::String(s)) => s,
                Some(v) => v.to_string(),
                None => return Err(HttpError { message: "offline".into() }),
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.into_bytes()),
                    mimetype: None,
                },
                headers,
            })
        }
            }

    const HTML_HASIL: &str = r#"<script>var ytInitialData = {"contents":{"twoColumnSearchResultsRenderer":{"primaryContents":{"sectionListRenderer":{"contents":[{"itemSectionRenderer":{"contents":[{"videoRenderer":{"videoId":"aaaaaaaaaaa","title":{"runs":[{"text":"Lagu Uji"}]},"ownerText":{"runs":[{"text":"Channel Uji"}]},"viewCountText":{"simpleText":"1,2 juta x ditonton"},"lengthText":{"accessibility":{"accessibilityData":{"label":"3 menit, 33 detik"}}},"thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/a/low.jpg"},{"url":"https://i.ytimg.com/vi/a/hq.jpg"}]},"publishedTimeText":{"simpleText":"2 tahun yang lalu"}}}]}}]}}}}};var meta = 1;</script>"#;

    #[test]
    fn parse_hasil_mengikuti_play_js() {
        let data: Value = serde_json::from_str(
            &HTML_HASIL["<script>var ytInitialData = ".len()..HTML_HASIL.len() - ";var meta = 1;</script>".len()],
        )
        .unwrap();
        let v = parse_hasil(&data);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, "aaaaaaaaaaa");
        assert_eq!(v[0].url, "https://www.youtube.com/watch?v=aaaaaaaaaaa");
        assert_eq!(v[0].title, "Lagu Uji");
        assert_eq!(v[0].author, "Channel Uji");
        assert_eq!(v[0].views, "1,2 juta x ditonton");
        assert_eq!(v[0].duration, "3 menit, 33 detik");
        assert!(v[0].thumbnail.ends_with("hq.jpg"), "{:?}", v[0].thumbnail);
        assert_eq!(v[0].published, "2 tahun yang lalu");
    }

    #[test]
    fn ambil_ytinitial_menemukan_objek_dengan_kurung_seimbang() {
        let data = ambil_ytinitial(HTML_HASIL).unwrap();
        assert!(data.pointer("/contents/twoColumnSearchResultsRenderer").is_some());
        assert!(ambil_ytinitial("<html>kosong</html>").is_err());
        // `}` di dalam string tidak boleh menutup objek lebih awal.
        let html = r#"var ytInitialData = {"a":"}"};var x=1;"#;
        assert_eq!(ambil_ytinitial(html).unwrap()["a"], "}");
    }

    #[test]
    fn ytmp3_backend_pertama_yang_menang_dan_video_pertama_dipakai() {
        let http = Stub::per_url(vec![
            ("youtube.com/results", Some(Value::String(HTML_HASIL.to_string()))),
            ("random-cdn", Some(serde_json::json!({"cdn": "cdn.example"}))),
            ("cdn.example/v2/info", Some(serde_json::json!({"status": false}))),
            // Penanda `/audio` lebih dulu — `ytmp3.ing/` juga cocok dengan
            // URL `/audio` (stub pakai `contains` pertama-yang cocok).
            ("ytmp3.ing/audio", Some(serde_json::json!({
                "url": "aHR0cHM6Ly9jZG4uZXhhbXBsZS9hLm1wMw==",
                "filename": "Lagu Uji.mp3"
            }))),
            ("ytmp3.ing/", Some(Value::String("<input value=\"csrf-token\">".to_string()))),
        ]);
        let hasil = ytmp3(http.as_ref(), "lagu uji", "128").unwrap();
        assert_eq!(hasil.backend, "ytmp3ing");
        assert_eq!(hasil.video.id, "aaaaaaaaaaa");
        assert_eq!(
            field(&hasil.download, "downloadUrl").unwrap(),
            "https://cdn.example/a.mp3"
        );
        assert_eq!(field(&hasil.download, "title").unwrap(), "Lagu Uji.mp3");
        // multipart field `url` terkirim.
        assert_eq!(http.header_diset.lock().unwrap().last().unwrap(), "url");
    }

    #[test]
    fn savetube_sukses_tanpa_panggil_ytmp3ing() {
        // Payload terenkripsi asli dari test youtube.rs (kunci KY yang sama).
        const INFO_B64: &str = "ABEiM0RVZneImaq7zN3u/ySRS+noY6Ay7XJ1nsZrbPP+2RcuGwwaev1qt4NlUA72hdBgn3XNiSl7QfGCo62X8B9A/hxp+1z3KaAFgGgptf1W9l6yzPe64HY5MMw0+o0UIV9n44ophBe+83JkcPivUaDXq2VdkgkvfkNkCj8KBaSrAXnP+WsnSmzRTPJUDJp/Ho+m4WSj/2N97wh2sdHfkQ==";
        let http = Stub::per_url(vec![
            ("youtube.com/results", Some(Value::String(HTML_HASIL.to_string()))),
            ("random-cdn", Some(serde_json::json!({"cdn": "cdn.example"}))),
            ("cdn.example/v2/info", Some(serde_json::json!({"status": true, "data": INFO_B64}))),
            ("cdn.example/download", Some(serde_json::json!({"data": {"downloadUrl": "https://dl.example/a.mp3"}}))),
        ]);
        let hasil = ytmp3(http.as_ref(), "lagu uji", "128").unwrap();
        assert_eq!(hasil.backend, "savetube");
        assert_eq!(field(&hasil.download, "downloadUrl").unwrap(), "https://dl.example/a.mp3");
        // info.durationLabel tidak ada → null (JSON.stringify JS).
        assert!(hasil.download.get("duration").unwrap().is_null());
        let seen = http.seen.lock().unwrap();
        assert!(!seen.iter().any(|u| u.contains("ytmp3.ing")));
        // POST /v2/info membawa URL video pertama.
        assert!(seen.iter().any(|u| u.contains("cdn.example/v2/info")));
    }

    #[test]
    fn semua_backend_gagal_mengembalikan_error_terakhir() {
        let http = Stub::per_url(vec![
            ("youtube.com/results", Some(Value::String(HTML_HASIL.to_string()))),
            ("random-cdn", Some(Value::Null)),
            ("ytmp3.ing", Some(serde_json::json!({"tanpa_value_attr": true}))),
        ]);
        let err = ytmp3(http.as_ref(), "lagu uji", "128").unwrap_err();
        assert!(err.message.starts_with("Semua backend gagal. Error terakhir:"), "{err:?}");
    }
}
