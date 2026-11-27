//! Video bocil, khusus premium/owner.

use oktz_plugin_api::{Ctx, HttpGet, MediaSource, Reply, plugin};
use serde_json::Value;

const PATH: &str = "src/tiktok/bocil.json";

/// Batas percobaan URL per command — sebagian URL pool sudah mati.
const MAX_COBA: usize = 5;

const ERR_KOSONG: &str = "❌ Data tidak tersedia";
const ERR_VIDEO: &str = "❌ *ᴇʀʀᴏʀ*\n\n> Video tidak ditemukan";

/// `m.reply` cabang premium.
pub(crate) fn pesan_premium(prefix: &str) -> String {
    format!(
        "🔞 *Konten Premium*\n\n\
         Video ini hanya bisa diakses pengguna *Premium*.\n\
         > Ketik *{prefix}benefitpremium* untuk info upgrade"
    )
}

/// `loadJsonData('bocil.json')` — item dibiarkan utuh supaya `item.url`
/// hilang tetap jatuh ke cabang `catch` seperti di versi awal.
fn muat_data(berkas: &dyn oktz_plugin_api::berkas::Berkas) -> Vec<Value> {
    let Ok(teks) = berkas.baca_teks(PATH) else {
        return Vec::new();
    };
    match serde_json::from_str::<Value>(&teks) {
        Ok(Value::Array(arr)) => arr,
        _ => Vec::new(),
    }
}

#[plugin(
    name = "bocil",
    alias = ["bocilvid"],
    category = "asupan",
    description = "Video bocil",
    usage = ".bocil",
    example = ".bocil",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bocil(ctx: Ctx) -> Reply {
    let reaksi = |emoji: &str| Reply::None.with_reaction(emoji);

    if !ctx.is_premium && !ctx.is_owner {
        return Reply::sequence(vec![reaksi("🕕"), Reply::text(pesan_premium(&ctx.prefix))]);
    }

    let data = muat_data(ctx.berkas());
    if data.is_empty() {
        return Reply::sequence(vec![reaksi("🕕"), reaksi("❌"), Reply::text(ERR_KOSONG)]);
    }

    let urls: Vec<String> = data
        .iter()
        .filter_map(|item| item.get("url").and_then(Value::as_str).map(str::to_string))
        .collect();

    match ambil_video(ctx.http.as_ref(), &urls) {
        Some(bytes) => Reply::sequence(vec![
            reaksi("🕕"),
            Reply::video(MediaSource::bytes(bytes), ""),
            reaksi("✅"),
        ]),
        // `sock.sendMedia` melempar di versi awal (URL mati/hilang) → `catch`.
        None => Reply::sequence(vec![reaksi("🕕"), reaksi("❌"), Reply::text(ERR_VIDEO)]),
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

    fn ctx(berkas: BerkasPalsu, premium: bool, owner: bool, http: Arc<Stub>) -> Ctx {
        Ctx {
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "bocil".into(),
            is_premium: premium,
            is_owner: owner,
            berkas: Arc::new(berkas),
            http,
            ..Ctx::default()
        }
    }

    fn bagian(r: &Reply) -> Vec<Reply> {
        match r.without_mutations() {
            Reply::Sequence(v) => v.iter().map(|x| x.without_mutations().clone()).collect(),
            lain => vec![lain.clone()],
        }
    }

    fn emoji_ke(r: &Reply, i: usize) -> Option<String> {
        match bagian(r).get(i) {
            Some(Reply::Reaction { emoji, .. }) => Some(emoji.clone()),
            _ => None,
        }
    }

    fn teks_ke(r: &Reply, i: usize) -> Option<String> {
        bagian(r).get(i)?.text_body().map(str::to_string)
    }

    fn https_berisi() -> BerkasPalsu {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(
            PATH,
            r#"[{"url":"https://cdn/bocil1.mp4"},{"url":"https://cdn/bocil2.mp4"}]"#,
        );
        b
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("bocil").unwrap();
        assert_eq!(p.meta.name, "bocil");
        assert_eq!(p.meta.alias, ["bocilvid"]);
        assert_eq!(p.meta.category, "asupan");
        assert_eq!(p.meta.description, "Video bocil");
        assert_eq!(p.meta.usage, ".bocil");
        assert_eq!(p.meta.example, ".bocil");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_premium && !p.meta.is_owner && !p.meta.is_group);
    }

    #[test]
    fn pengguna_biasa_ditolak_setelah_reaksi_jam() {
        let r = bocil(ctx(https_berisi(), false, false, Stub::bytes("X")));
        assert_eq!(emoji_ke(&r, 0).as_deref(), Some("🕕"));
        assert_eq!(
            teks_ke(&r, 1).as_deref(),
            Some(
                "🔞 *Konten Premium*\n\n\
                 Video ini hanya bisa diakses pengguna *Premium*.\n\
                 > Ketik *.benefitpremium* untuk info upgrade"
            )
        );
        assert_eq!(bagian(&r).len(), 2);
        // Tidak ada media yang dikirim.
        assert!(!r.has_media());
    }

    #[test]
    fn premium_dan_owner_tembus_gate() {
        for (premium, owner) in [(true, false), (false, true)] {
            let r = bocil(ctx(https_berisi(), premium, owner, Stub::bytes("VID")));
            assert!(r.has_media(), "premium={premium} owner={owner}");
        }
    }

    #[test]
    fn data_kosong_atau_berkas_hilang_jadi_pesan_kosong() {
        let kosong = BerkasPalsu::baru();
        let _ = kosong.tulis_teks(PATH, "[]");
        for b in [BerkasPalsu::baru(), kosong] {
            let r = bocil(ctx(b, true, false, Stub::bytes("VID")));
            assert_eq!(emoji_ke(&r, 0).as_deref(), Some("🕕"));
            assert_eq!(emoji_ke(&r, 1).as_deref(), Some("❌"));
            assert_eq!(teks_ke(&r, 2).as_deref(), Some(ERR_KOSONG));
        }
    }

    #[test]
    fn json_rusak_diperlakukan_seperti_data_kosong() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(PATH, "bukan json");
        let r = bocil(ctx(b, true, false, Stub::bytes("VID")));
        assert_eq!(teks_ke(&r, 2).as_deref(), Some(ERR_KOSONG));
    }

    #[test]
    fn item_tanpa_url_masuk_cabang_catch() {
        let b = BerkasPalsu::baru();
        let _ = b.tulis_teks(PATH, r#"[{"judul":"tanpa url"}]"#);
        let r = bocil(ctx(b, true, false, Stub::bytes("VID")));
        assert_eq!(emoji_ke(&r, 1).as_deref(), Some("❌"));
        assert_eq!(teks_ke(&r, 2).as_deref(), Some(ERR_VIDEO));
        assert!(!r.has_media());
    }

    #[test]
    fn url_terpilih_dikirim_sebagai_video_lalu_reaksi_centang() {
        let http = Stub::bytes("VIDEO_BYTES");
        let r = bocil(ctx(https_berisi(), false, true, http.clone()));
        assert_eq!(emoji_ke(&r, 0).as_deref(), Some("🕕"));

        let Some(Reply::Video { media, .. }) = bagian(&r).get(1).cloned() else {
            panic!("pesan kedua harus video");
        };
        match media {
            Some(MediaSource::Bytes { data, .. }) => {
                assert_eq!(&data[..], b"VIDEO_BYTES");
            }
            lain => panic!("sumber harus byte, dapat {lain:?}"),
        }

        assert_eq!(emoji_ke(&r, 2).as_deref(), Some("✅"));
        assert_eq!(http.seen.lock().unwrap().len(), 1, "cukup satu URL");
    }

    #[test]
    fn url_pertama_gagal_lalu_kedua_sukses() {
        let http = Stub::gagal_awal("VIDEO", 1);
        let r = bocil(ctx(https_berisi(), false, true, http.clone()));
        assert!(r.has_media());
        assert_eq!(emoji_ke(&r, 2).as_deref(), Some("✅"));
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2, "harus coba dua URL: {seen:?}");
        assert_ne!(seen[0], seen[1], "URL tidak boleh diulang");
    }

    #[test]
    fn semua_url_gagal_jadi_pesan_error() {
        let r = bocil(ctx(https_berisi(), false, true, Stub::offline()));
        assert_eq!(emoji_ke(&r, 0).as_deref(), Some("🕕"));
        assert_eq!(emoji_ke(&r, 1).as_deref(), Some("❌"));
        assert_eq!(teks_ke(&r, 2).as_deref(), Some(ERR_VIDEO));
        assert!(!r.has_media());
    }
}
