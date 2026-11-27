//! Video ukhty.

use oktz_plugin_api::{Ctx, HttpGet, MediaSource, Reply, plugin};
use serde_json::Value;

const PATH: &str = "src/tiktok/ukhty.json";

/// Batas percobaan URL per command — sebagian URL pool sudah mati.
const MAX_COBA: usize = 5;

const ERR_KOSONG: &str = "❌ Data tidak tersedia";
const ERR_VIDEO: &str = "❌ *ᴇʀʀᴏʀ*\n\n> Video tidak ditemukan";

/// `loadJsonData('ukhty.json')` —.
fn muat_data(berkas: &dyn oktz_plugin_api::berkas::Berkas) -> Vec<String> {
    let Ok(teks) = berkas.baca_teks(PATH) else {
        return Vec::new();
    };
    let Ok(data): Result<Value, _> = serde_json::from_str(&teks) else {
        return Vec::new();
    };
    let Some(arr) = data.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            item.get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect()
}

#[plugin(
    name = "ukhty",
    alias = ["ukht"],
    category = "asupan",
    description = "Video ukhty",
    usage = ".ukhty",
    example = ".ukhty",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ukhty(ctx: Ctx) -> Reply {
    let urls = match muat_data(ctx.berkas()) {
        u if !u.is_empty() => u,
        // `m.react('❌')` lalu `m.reply('❌ Data tidak tersedia')`
        //
        _ => return Reply::text(ERR_KOSONG).with_reaction("❌"),
    };

    match ambil_video(ctx.http.as_ref(), &urls) {
        // `m.react('✅')` setelah `sock.sendMedia`.
        Some(data) => Reply::video(MediaSource::bytes(data), "").with_reaction("✅"),
        // `sendMedia` melempar di versi awal (URL mati) → `catch`.
        None => Reply::text(ERR_VIDEO).with_reaction("❌"),
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
        HttpBody, HttpError, HttpResponse,
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
                None => Err(HttpError {
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

    fn run(berkas: BerkasPalsu, http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "ukhty".into(),
            berkas: Arc::new(berkas),
            http,
            ..Ctx::default()
        };
        ukhty(ctx)
    }

    fn body_of(r: &Reply) -> String {
        r.text_body().unwrap_or_default().to_string()
    }

    fn video_bytes(r: &Reply) -> Option<Vec<u8>> {
        // `with_reaction` membungkus balasan di `Mutate` + `Reaction`, jadi
        // keduanya harus dilepas dulu sebelum varian media dilihat.
        match r.without_mutations().tanpa_reaksi() {
            Reply::Video { media, .. } => media.as_ref().and_then(|m| match m {
                MediaSource::Bytes { data, .. } => Some(data.to_vec()),
                _ => None,
            }),
            _ => None,
        }
    }

    #[test]
    fn meta_sesuai_js() {
        let p = oktz_plugin_api::find("ukhty").unwrap();
        assert_eq!(p.meta.name, "ukhty");
        assert_eq!(p.meta.alias, ["ukht"]);
        assert_eq!(p.meta.category, "asupan");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn berkas_kosong_atau_tidak_ada_mengembalikan_pesan_kosong() {
        let r = run(BerkasPalsu::baru(), Stub::bytes("V"));
        assert_eq!(body_of(&r), ERR_KOSONG);
    }

    #[test]
    fn json_salah_dilaporkan_kosong() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(PATH, "bukan json");
        let r = run(b, Stub::bytes("V"));
        assert_eq!(body_of(&r), ERR_KOSONG);
    }

    #[test]
    fn data_kosong_dilaporkan_kosong() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(PATH, "[]");
        let r = run(b, Stub::bytes("V"));
        assert_eq!(body_of(&r), ERR_KOSONG);
    }

    #[test]
    fn url_dikirim_sebagai_video() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(
            PATH,
            r#"[{"url":"https://cdn/ukhty1.mp4"},{"url":"https://cdn/ukhty2.mp4"}]"#,
        );
        let r = run(b, Stub::bytes("VIDEO"));
        assert_eq!(video_bytes(&r), Some(b"VIDEO".to_vec()));
    }

    #[test]
    fn url_pertama_gagal_lalu_kedua_sukses() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(
            PATH,
            r#"[{"url":"https://cdn/mati.mp4"},{"url":"https://cdn/hidup.mp4"}]"#,
        );
        let http = Stub::gagal_awal("VIDEO", 1);
        let r = run(b, http.clone());
        assert_eq!(video_bytes(&r), Some(b"VIDEO".to_vec()));
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2, "harus coba dua URL: {seen:?}");
        assert_ne!(seen[0], seen[1], "URL tidak boleh diulang");
    }

    #[test]
    fn semua_url_gagal_jadi_pesan_error() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(
            PATH,
            r#"[{"url":"https://cdn/a.mp4"},{"url":"https://cdn/b.mp4"}]"#,
        );
        let r = run(b, Stub::offline());
        assert_eq!(body_of(&r), ERR_VIDEO);
        assert_eq!(video_bytes(&r), None);
    }
}
