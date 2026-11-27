//! Random video asupan.
//!
//! `Keterbatasan:` versi awal membaca 8 berkas JSON lalu mengambil byte video lewat HTTP.
//! Di Rust berkas dibaca lewat `ctx.berkas()` dan byte video lewat `ctx.http.get()`. Reaksi
//! `🕕` / `❌` dari versi awal diruntuhkan jadi satu reaksi terakhir karena
//! `Reply::with_reaction` hanya boleh satu emoji.

use oktz_plugin_api::{Ctx, HttpGet, MediaSource, Reply, plugin};
use serde_json::Value;

const DIR: &str = "src/tiktok";

/// Batas percobaan URL per command — sebagian URL pool sudah mati.
const MAX_COBA: usize = 5;

const FILES: [&str; 8] = [
    "bocil.json",
    "gheayubi.json",
    "kayes.json",
    "notnot.json",
    "panrika.json",
    "santuy.json",
    "tiktokgirl.json",
    "ukhty.json",
];

const ERR_DATA_EMPTY: &str = "❌ Data asupan tidak tersedia";
const ERR_AMBIL: &str = "❌ *ᴇʀʀᴏʀ*\n\n> Video asupan tidak ditemukan";

/// `loadJsonData` — gabungkan `url` dari semua berkas JSON —.
fn muat_url(berkas: &dyn oktz_plugin_api::berkas::Berkas) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    for file in FILES {
        let path = format!("{DIR}/{file}");
        let Ok(teks) = berkas.baca_teks(&path) else {
            continue;
        };
        let Ok(data): Result<Value, _> = serde_json::from_str(&teks) else {
            continue;
        };
        let Some(arr) = data.as_array() else {
            continue;
        };
        for item in arr {
            if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                urls.push(url.to_string());
            }
        }
    }
    urls
}

#[plugin(
    name = "asupan",
    alias = ["asupanrandom"],
    category = "asupan",
    description = "Random video asupan",
    usage = ".asupan",
    example = ".asupan",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn asupan(ctx: Ctx) -> Reply {
    let urls = muat_url(ctx.berkas());
    if urls.is_empty() {
        return Reply::text(ERR_DATA_EMPTY);
    }

    match ambil_video(ctx.http.as_ref(), &urls) {
        Some(data) => Reply::video(MediaSource::bytes(data), ""),
        None => Reply::text(ERR_AMBIL),
    }
}

/// Coba sampai [`MAX_COBA`] URL berbeda (acak) sampai ada respons 2xx berisi.
fn ambil_video(http: &dyn HttpGet, urls: &[String]) -> Option<Vec<u8>> {
    use rand::Rng as _;
    let mut idx: Vec<usize> = (0..urls.len()).collect();
    let mut rng = rand::rng();
    let ambil = MAX_COBA.min(idx.len());
    for i in 0..ambil {
        let j = rng.random_range(i..idx.len());
        idx.swap(i, j);
    }
    for i in idx.into_iter().take(ambil) {
        let Ok(res) = http.get(&urls[i]) else {
            continue;
        };
        if !(200..300).contains(&res.status) {
            continue;
        }
        let data = res.body.bytes().to_vec();
        if !data.is_empty() {
            return Some(data);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpError, HttpGet, HttpResponse,
        berkas::{Berkas, BerkasPalsu},
    };
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct Stub {
        body: Option<String>,
        /// Berapa panggilan pertama yang dipaksa gagal (simulasi URL mati).
        sisa_gagal: Mutex<usize>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn bytes(data: impl Into<Vec<u8>>) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(String::from_utf8(data.into()).unwrap()),
                sisa_gagal: Mutex::new(0),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                body: None,
                sisa_gagal: Mutex::new(0),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn gagal_awal(data: impl Into<Vec<u8>>, n: usize) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(String::from_utf8(data.into()).unwrap()),
                sisa_gagal: Mutex::new(n),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let gagal = {
                let mut sisa = self.sisa_gagal.lock().unwrap();
                if *sisa > 0 {
                    *sisa -= 1;
                    true
                } else {
                    false
                }
            };
            if gagal {
                return Err(HttpError {
                    message: "mati".into(),
                });
            }
            match &self.body {
                Some(b) => Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(b.as_bytes().to_vec()),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
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

    fn berkas_dengan(files: &[(&str, &str)]) -> BerkasPalsu {
        let b = BerkasPalsu::baru();
        for (path, isi) in files {
            let _ = b.tulis_teks(path, isi);
        }
        b
    }

    fn run(berkas: BerkasPalsu, http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "asupan".into(),
            http,
            berkas: Arc::new(berkas),
            ..Ctx::default()
        };
        asupan(ctx)
    }

    fn body_of(r: &Reply) -> String {
        r.text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn tanpa_url_mengembalikan_pesan_data_kosong_tanpa_request() {
        let berkas = berkas_dengan(&[]);
        let http = Stub::offline();
        let r = run(berkas, http.clone());
        assert_eq!(body_of(&r), ERR_DATA_EMPTY);
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn satu_url_dikirim_sebagai_video() {
        let berkas =
            berkas_dengan(&[("src/tiktok/ukhty.json", r#"[{"url":"https://cdn/a.mp4"}]"#)]);
        let http = Stub::bytes("VIDEO_BYTES");
        let r = run(berkas, http.clone());
        match r {
            Reply::Video { media, .. } => {
                assert_eq!(
                    media,
                    Some(MediaSource::Bytes {
                        data: Arc::from("VIDEO_BYTES".as_bytes()),
                        mimetype: None
                    })
                );
            }
            other => panic!("harus Video, dapat {other:?}"),
        }
        assert_eq!(http.seen.lock().unwrap().as_slice(), &["https://cdn/a.mp4"]);
    }

    #[test]
    fn beberapa_berkas_digabung_dan_ambil_satu() {
        let berkas = berkas_dengan(&[
            ("src/tiktok/bocil.json", r#"[{"url":"https://cdn/b.mp4"}]"#),
            ("src/tiktok/santuy.json", "[]"),
            (
                "src/tiktok/tiktokgirl.json",
                r#"[{"url":"https://cdn/c.mp4"},{"url":"https://cdn/d.mp4"}]"#,
            ),
        ]);
        let http = Stub::bytes("VID");
        let _ = run(berkas, http.clone());
        let seen = http.seen.lock().unwrap().clone();
        assert!(
            seen == vec!["https://cdn/b.mp4"]
                || seen == vec!["https://cdn/c.mp4"]
                || seen == vec!["https://cdn/d.mp4"]
        );
    }

    #[test]
    fn url_pertama_gagal_lalu_kedua_sukses() {
        let berkas = berkas_dengan(&[(
            "src/tiktok/ukhty.json",
            r#"[{"url":"https://cdn/mati.mp4"},{"url":"https://cdn/hidup.mp4"}]"#,
        )]);
        let http = Stub::gagal_awal("VIDEO", 1);
        let r = run(berkas, http.clone());
        assert!(matches!(&r, Reply::Video { .. }), "harus Video: {r:?}");
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2, "harus coba dua URL: {seen:?}");
        assert_ne!(seen[0], seen[1], "URL tidak boleh diulang");
    }

    #[test]
    fn semua_url_gagal_jadi_pesan_error() {
        let berkas = berkas_dengan(&[(
            "src/tiktok/ukhty.json",
            r#"[{"url":"https://cdn/a.mp4"},{"url":"https://cdn/b.mp4"}]"#,
        )]);
        let http = Stub::offline();
        let r = run(berkas, http.clone());
        assert_eq!(body_of(&r), ERR_AMBIL);
        assert_eq!(http.seen.lock().unwrap().len(), 2);
    }

    #[test]
    fn maksimal_lima_url_dicoba_tanpa_ulang() {
        let isi: Vec<String> = (0..7)
            .map(|i| format!(r#"{{"url":"https://cdn/{i}.mp4"}}"#))
            .collect();
        let json = format!("[{}]", isi.join(","));
        let berkas = berkas_dengan(&[("src/tiktok/ukhty.json", json.as_str())]);
        let http = Stub::offline();
        let r = run(berkas, http.clone());
        assert_eq!(body_of(&r), ERR_AMBIL);
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 5, "maksimal 5 percobaan");
        let unik: std::collections::HashSet<_> = seen.iter().collect();
        assert_eq!(unik.len(), 5, "tidak boleh mengulang URL");
    }

    #[test]
    fn json_berkas_salah_diabaikan() {
        let berkas = berkas_dengan(&[("src/tiktok/bocil.json", "bukan json")]);
        let http = Stub::offline();
        let r = run(berkas, http);
        assert_eq!(body_of(&r), ERR_DATA_EMPTY);
    }
}
