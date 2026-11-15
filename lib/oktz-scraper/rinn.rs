//! API `apiii-xrina.my.id` — pengganti provider yang sudah mati.
//!
//! Diperiksa 2026-11-15 (HTTP live):
//!   TikWM (`tikwm.com/api`) 403, `api.nexray.eu.cc/random/anime` 500,
//!   `api.nexray.web.id/api/search/pinterest` 404,
//!   `ihancer.com/api/enhance` 500, `kyzznekoo.zone.id/api/img/v2/nanobanana` 404.
//! Semua route di bawah ini dibaca dari `openapi.json` (365 route) dan sudah
//! dites satu per satu; `/ai-image/*` memulangkan byte gambar langsung.

use oktz_plugin_api::{HttpCall, HttpError, HttpGet};
use serde_json::{Value, json};

use crate::common::{encode_uri_component, truthy};

pub const BASE: &str = "https://apiii-xrina.my.id";

/// UA browser — Cloudflare di depan API menolak klien tanpa UA.
const UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36";

fn ambil(http: &dyn HttpGet, jalur: &str) -> Result<oktz_plugin_api::HttpResponse, HttpError> {
    let res = http.call(&HttpCall::get(format!("{BASE}{jalur}")).header("User-Agent", UA))?;
    if !(200..300).contains(&res.status) {
        return Err(HttpError {
            message: format!("rinn {} untuk {}", res.status, jalur),
        });
    }
    Ok(res)
}

fn json(http: &dyn HttpGet, jalur: &str) -> Result<Value, HttpError> {
    let res = ambil(http, jalur)?;
    let data = res.json().unwrap_or(Value::Null);
    // Semua route memakai amplop `{ status, creator, data }`; `status: false`
    // membawa penjelasan di `message`.
    if !data.get("status").map(truthy).unwrap_or(false) {
        let pesan = data
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("rinn: status false");
        return Err(HttpError {
            message: pesan.to_string(),
        });
    }
    Ok(data)
}

fn bytes(http: &dyn HttpGet, jalur: &str) -> Result<Vec<u8>, HttpError> {
    let res = ambil(http, jalur)?;
    let data = res.body.bytes().to_vec();
    if data.is_empty() {
        return Err(HttpError {
            message: format!("rinn: hasil kosong dari {jalur}"),
        });
    }
    Ok(data)
}

fn teks(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

/// `ttdown` versi `apiii-xrina`: `/download/tiktok2`.
///
/// Bentuk keluaran disamakan dengan [`crate::tiktok::ttdown`] supaya
/// `download/tiktok.rs` dan `download/ttmp3.rs` tidak perlu tahu providernya:
/// `downloads[]` dengan `type` = `nowatermark` / `watermark` / `mp3` / `photo`.
///
/// Tipe dibaca dari `medias[].type` yang dilaporkan host — bukan ditebak dari
/// `duration`, cara lama yang membuat foto slide bisa dikirim sebagai video.
pub fn tiktok(http: &dyn HttpGet, url: &str) -> Result<Value, HttpError> {
    let data = json(http, &format!("/download/tiktok2?url={}", encode_uri_component(url)))?;
    let d = data.get("data").cloned().unwrap_or(Value::Null);

    let mut downloads: Vec<Value> = Vec::new();
    let mut sudah_video = false;
    for m in d
        .get("medias")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let tipe = m.get("type").and_then(Value::as_str).unwrap_or_default();
        let alamat = m.get("url").and_then(Value::as_str).unwrap_or_default();
        if alamat.is_empty() {
            continue;
        }
        match tipe {
            "video" => {
                // Video pertama = tanpa watermark (`quality: without_watermark`).
                if sudah_video {
                    continue;
                }
                sudah_video = true;
                downloads.push(json!({ "type": "nowatermark", "label": "Video", "url": alamat }));
            }
            "audio" => {
                downloads.push(json!({ "type": "mp3", "label": "Audio MP3", "url": alamat }));
            }
            "image" | "photo" | "images" => {
                downloads.push(json!({ "type": "photo", "label": "Photo", "url": alamat }));
            }
            _ => {}
        }
    }

    if downloads.is_empty() {
        return Err(HttpError {
            message: "rinn tiktok: tidak ada media".to_string(),
        });
    }

    let stats = d.get("statistics").cloned().unwrap_or(Value::Null);
    let stat = |kunci: &str| {
        stats
            .get(kunci)
            .map(|v| teks(Some(v)))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "-".to_string())
    };
    let durasi = teks(d.get("duration"));
    let avatar = teks(d.get("author").and_then(|a| a.get("avatar")));

    Ok(json!({
        "_provider": "rinn",
        "title": teks(d.get("title")),
        "author": {
            "username": teks(d.get("author").and_then(|a| a.get("nickname"))),
            "fullname": teks(d.get("author").and_then(|a| a.get("username"))),
            "nickname": teks(d.get("author").and_then(|a| a.get("nickname"))),
            "avatar": if avatar.is_empty() { Value::Null } else { Value::String(avatar) },
        },
        "music_info": { "title": "-", "author": "-" },
        "duration": durasi.clone(),
        "durations": if sudah_video { 1 } else { 0 },
        "taken_at": "-",
        "region": "-",
        "stats": {
            "views": "-",
            "likes": stat("likes"),
            "comment": stat("comments"),
            "share": stat("shares"),
            "download": "-",
        },
        "cover": Value::Null,
        "downloads": downloads,
    }))
}

/// `/search/pinterest?q=&limit=` → URL gambar. `limit` hanya menerima
/// 5/10/15/20/30, jadi nilainya dipaksa ke salah satu itu.
pub fn pinterest(http: &dyn HttpGet, q: &str, limit: u32) -> Result<Vec<String>, HttpError> {
    let limit = match limit {
        0..=5 => 5,
        6..=10 => 10,
        11..=15 => 15,
        16..=20 => 20,
        _ => 30,
    };
    let data = json(
        http,
        &format!(
            "/search/pinterest?q={}&limit={limit}",
            encode_uri_component(q)
        ),
    )?;
    Ok(data
        .get("data")
        .and_then(Value::as_array)
        .map(|pins| {
            pins.iter()
                .filter_map(|p| p.get("image").and_then(Value::as_str))
                .filter(|u| u.starts_with("http"))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default())
}

/// `/ai-image/nanobanana?image=&prompt=` → byte gambar hasil edit.
pub fn nanobanana(http: &dyn HttpGet, image_url: &str, prompt: &str) -> Result<Vec<u8>, HttpError> {
    bytes(
        http,
        &format!(
            "/ai-image/nanobanana?image={}&prompt={}",
            encode_uri_component(image_url),
            encode_uri_component(prompt)
        ),
    )
}

/// `/ai-image/hd` → `/ai-image/hd2` → `/ai-image/unblur`.
///
/// Tiga route dilayani model berbeda; yang pertama berhasil dipakai supaya satu
/// model yang sedang rusak tidak mematikan fitur.
pub fn hd(http: &dyn HttpGet, image_url: &str) -> Result<Vec<u8>, HttpError> {
    let enc = encode_uri_component(image_url);
    let jalur = [
        format!("/ai-image/hd?url={enc}&scale=2"),
        format!("/ai-image/hd2?url={enc}"),
        format!("/ai-image/unblur?url={enc}"),
    ];
    let mut terakhir = HttpError {
        message: "rinn: semua route HD gagal".to_string(),
    };
    for j in jalur {
        match bytes(http, &j) {
            Ok(data) => return Ok(data),
            Err(e) => terakhir = e,
        }
    }
    Err(terakhir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Stub {
        seen: Mutex<Vec<String>>,
        balasan: Mutex<Vec<(u16, Vec<u8>)>>,
    }

    impl Stub {
        fn baru(balasan: Vec<(u16, &str)>) -> Arc<Stub> {
            Arc::new(Stub {
                seen: Mutex::new(Vec::new()),
                balasan: Mutex::new(
                    balasan
                        .into_iter()
                        .map(|(s, b)| (s, b.as_bytes().to_vec()))
                        .collect(),
                ),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let mut b = self.balasan.lock().unwrap();
            let (status, data) = if b.len() > 1 { b.remove(0) } else { b[0].clone() };
            Ok(HttpResponse {
                status,
                body: HttpBody::Bytes {
                    data: Arc::from(data),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
    }

    #[test]
    fn tiktok_membaca_tipe_dari_medias() {
        let http = Stub::baru(vec![(
            200,
            r#"{"status":true,"data":{"title":"t","duration":"02:38","author":{"username":"u","nickname":"N","avatar":"a"},"statistics":{"likes":"1K","comments":"2","shares":"3"},"medias":[{"type":"video","url":"https://v/1.mp4","quality":"without_watermark"},{"type":"audio","url":"https://a/1.mp3"}]}}"#,
        )]);
        let hasil = tiktok(http.as_ref(), "https://vt.tiktok.com/x").unwrap();
        let tipe: Vec<&str> = hasil["downloads"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["type"].as_str().unwrap())
            .collect();
        assert_eq!(tipe, ["nowatermark", "mp3"]);
        assert_eq!(hasil["durations"], 1);
        assert_eq!(hasil["stats"]["likes"], "1K");
    }

    #[test]
    fn tiktok_foto_tidak_punya_video() {
        let http = Stub::baru(vec![(
            200,
            r#"{"status":true,"data":{"title":"t","duration":"0","medias":[{"type":"image","url":"https://p/1.jpg"},{"type":"image","url":"https://p/2.jpg"}]}}"#,
        )]);
        let hasil = tiktok(http.as_ref(), "https://vt.tiktok.com/x").unwrap();
        assert_eq!(hasil["downloads"].as_array().unwrap().len(), 2);
        assert_eq!(hasil["downloads"][0]["type"], "photo");
        assert_eq!(hasil["durations"], 0);
    }

    #[test]
    fn status_false_jadi_error_dengan_pesan_host() {
        let http = Stub::baru(vec![(200, r#"{"status":false,"message":"URL tidak valid"}"#)]);
        let e = pinterest(http.as_ref(), "kucing", 5).unwrap_err();
        assert_eq!(e.message, "URL tidak valid");
    }

    #[test]
    fn pinterest_memaksa_limit_ke_pilihan_host() {
        let http = Stub::baru(vec![(
            200,
            r#"{"status":true,"data":[{"image":"https://i.pinimg.com/a.jpg"},{"image":""}]}"#,
        )]);
        let urls = pinterest(http.as_ref(), "cewe cantik", 7).unwrap();
        assert_eq!(urls, ["https://i.pinimg.com/a.jpg"]);
        assert!(http.seen.lock().unwrap()[0].contains("limit=10"));
        assert!(http.seen.lock().unwrap()[0].contains("q=cewe%20cantik"));
    }

    #[test]
    fn hd_mencoba_route_kedua_kalau_pertama_gagal() {
        let http = Stub::baru(vec![(500, ""), (200, "PNGDATA")]);
        let data = hd(http.as_ref(), "https://x/y.jpg").unwrap();
        assert_eq!(data, b"PNGDATA");
        assert_eq!(http.seen.lock().unwrap().len(), 2);
    }

    #[test]
    fn nanobanana_mengenkode_prompt() {
        let http = Stub::baru(vec![(200, "P")]);
        nanobanana(http.as_ref(), "https://x/y.jpg", "ubah jadi anime").unwrap();
        let url = http.seen.lock().unwrap()[0].clone();
        assert!(url.contains("/ai-image/nanobanana?image=https%3A%2F%2Fx%2Fy.jpg"));
        assert!(url.contains("prompt=ubah%20jadi%20anime"));
    }
}
