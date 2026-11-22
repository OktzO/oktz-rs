//! `plugins/canvas/fakecall.js` — gambar panggilan palsu WhatsApp (Android/iOS).
//!
//! API: `my.izuka-api.xyz/api/canvas/fakecall-android|ios` (multipart POST,
//! `fakecall.js:73-95`). Provider pindah ke `apiii-xrina.my.id` dengan bentuk
//! beda: `GET /canvas/call-andro|call-ios?pp=&nama=&durasi=` (openapi.json
//! 2026-11-21, dites live balas PNG). Avatar jadi URL `pp`, bukan berkas
//! multipart — buffer byte tak bisa dipakai, jadi foto profil sender
//! (`ctx.sock().foto_profil`) dipakai dan catbox jadi cadangan.
//!
//! `ponytail:` `m.react('🕕')` lalu reaksi `📞`/`☢` — `Reply` hanya menyimpan
//! satu reaksi, jadi yang dipegang adalah yang terakhir. JS gagal API tetap
//! kirim `m.quoted.download()` sebagai gambar; ini balas error template.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use oktz_scraper::common::encode_uri_component;

/// Avatar cadangan (`fakecall.js:70`).
pub(crate) const AVATAR_CADANGAN: &str = "https://files.catbox.moe/nwvkbt.png";

/// Balasan saat format tidak ada `|` atau teks kosong (`fakecall.js:25-39`).
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "Halo! Sepertinya format yang kamu masukkan kurang tepat.\n\n\
         Fitur ini digunakan untuk membuat gambar panggilan palsu (Fake Call) seolah-olah seseorang sedang meneleponmu secara langsung.\n\n\
         *Daftar Perintah yang Tersedia:*\n\
         - *{prefix}fakecall* (Untuk tampilan panggilan Android)\n\
         - *{prefix}fakecall-ios* (Untuk tampilan panggilan iPhone atau iOS)\n\n\
         *Cara Penggunaan:*\n\
         Ketik perintah diikuti dengan *Nama* dan *Durasi* yang dipisahkan oleh tanda pipa (|).\n\n\
         *Contoh Penggunaan:*\n\
         - *{prefix}fakecall Zann | 03:33:33*\n\
         - *{prefix}fakecall-ios Sayangku | 12:00:00*\n\n\
         *Tips Tambahan:*\n\
         Kamu bisa me-reply (membalas) sebuah gambar jika ingin menggunakan foto tersebut sebagai avatar profil sang penelepon!"
    )
}

/// `https://apiii-xrina.my.id/canvas/{endpoint}` — `pp=&nama=&durasi=`.
pub(crate) fn api_url(endpoint: &str, pp: &str, nama: &str, durasi: &str) -> String {
    format!(
        "https://apiii-xrina.my.id/canvas/{endpoint}?pp={}&nama={}&durasi={}",
        encode_uri_component(pp),
        encode_uri_component(nama),
        encode_uri_component(durasi)
    )
}

/// Caption tidak ada (`fakecall.js` hanya mengirim image tanpa caption).
const CAPTION: &str = "";

#[plugin(
    name = "fakecall",
    commands = [
        "fakecall-android", "fakecall-ios",
    ],
    alias = ["fakecallwa"],
    category = "canvas",
    description = "Membuat gambar fake call WhatsApp (Tersedia versi Android dan iOS)",
    usage = ".fakecall nama | durasi",
    example = ".fakecall Zann | 19:00",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn fakecall(ctx: Ctx) -> Reply {
    let cmd = ctx.command.to_lowercase();
    let text = ctx.text.trim();

    if !text.contains('|') {
        return Reply::text(bantuan(&ctx.prefix));
    }

    let mut parts = text.split('|').map(str::trim);
    let nama = parts.next().unwrap_or("");
    let durasi = parts.next().unwrap_or("");

    if nama.is_empty() {
        return Reply::text(
            "Maaf, nama penelepon tidak boleh dikosongkan. Silakan isi namanya terlebih dahulu!",
        );
    }
    if durasi.is_empty() {
        return Reply::text(
            "Maaf, durasi panggilan tidak boleh dikosongkan. Silakan isi durasinya terlebih dahulu!",
        );
    }

    // `pp` wajib di API baru; JS memakai buffer yang diunggah, jadi foto
    // profil pengirim jadi sumbernya, catbox cadangan.
    let pp = ctx
        .sock()
        .foto_profil(&ctx.sender, "image")
        .ok()
        .flatten()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| AVATAR_CADANGAN.to_string());

    let endpoint = if cmd == "fakecall-ios" {
        "call-ios"
    } else {
        "call-andro"
    };

    let Ok(res) = ctx.http.get(&api_url(endpoint, &pp, nama, durasi)) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("❌");
    };
    let data = res.body.bytes().to_vec();
    if data.starts_with(b"{") {
        // API balas JSON error (bukan gambar): `catch` JS.
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("❌");
    }

    Reply::image(MediaSource::bytes(data), CAPTION).with_reaction("📞")
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct Stub {
        panggilan: Mutex<Vec<String>>,
        balasan: Vec<u8>,
        gagal: bool,
    }
    impl Stub {
        fn body(bytes: &[u8]) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: bytes.to_vec(),
                ..Default::default()
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                gagal: true,
                ..Default::default()
            })
        }
        fn json_error() -> Arc<Stub> {
            Self::body(br#"{"status":false,"message":"rusak"}"#.as_slice())
        }
    }

    impl oktz_plugin_api::HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            if self.gagal {
                return Err(HttpError {
                    message: "offline".into(),
                });
            }
            self.panggilan.lock().unwrap().push(url.to_string());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.balasan.clone()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma GET")
        }
    }

    fn run(http: Arc<Stub>, cmd: &str, text: &str) -> Reply {
        let ctx = Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: cmd.into(),
            http,
            ..Ctx::default()
        };
        fakecall(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn tanpa_pipa_menghasilkan_bantuan_tanpa_reaksi() {
        let http = Stub::offline();
        let r = run(http.clone(), "fakecall", "");
        assert_eq!(text_of(&r), bantuan("."));
        assert_eq!(reaksi(&r), None);
        assert!(http.panggilan.lock().unwrap().is_empty());
    }

    #[test]
    fn nama_kosong_ditolak() {
        let http = Stub::offline();
        let r = run(http.clone(), "fakecall", " | 03:33");
        assert_eq!(
            text_of(&r),
            "Maaf, nama penelepon tidak boleh dikosongkan. Silakan isi namanya terlebih dahulu!"
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn durasi_kosong_ditolak() {
        let http = Stub::offline();
        let r = run(http.clone(), "fakecall", "Zann |");
        assert_eq!(
            text_of(&r),
            "Maaf, durasi panggilan tidak boleh dikosongkan. Silakan isi durasinya terlebih dahulu!"
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn api_url_mengikuti_param_get_dan_terenkode() {
        assert_eq!(
            api_url("call-andro", "https://x/pp.jpg", "Zann", "03:33:33"),
            "https://apiii-xrina.my.id/canvas/call-andro?pp=https%3A%2F%2Fx%2Fpp.jpg&nama=Zann&durasi=03%3A33%3A33"
        );
        // `pp` selalu terisi (foto profil atau catbox), tapi fungsi murni ini
        // tetap memetakan string kosong apa adanya.
        assert_eq!(
            api_url("call-ios", "", "A", "1"),
            "https://apiii-xrina.my.id/canvas/call-ios?pp=&nama=A&durasi=1"
        );
    }

    #[test]
    fn android_pakai_catbox_kalau_tanpa_foto_profil() {
        let http = Stub::body(b"PNG".as_slice());
        let r = run(http.clone(), "fakecall", "Zann | 03:33");
        assert_eq!(reaksi(&r), Some("📞"));
        assert_eq!(
            http.panggilan.lock().unwrap()[0],
            "https://apiii-xrina.my.id/canvas/call-andro?pp=https%3A%2F%2Ffiles.catbox.moe%2Fnwvkbt.png&nama=Zann&durasi=03%3A33"
        );
    }

    #[test]
    fn ios_menggunakan_endpoint_ios() {
        let http = Stub::body(b"x".as_slice());
        let r = run(http.clone(), "fakecall-ios", "Zann | 12:00");
        assert_eq!(reaksi(&r), Some("📞"));
        assert!(
            http.panggilan.lock().unwrap()[0].contains("/canvas/call-ios?"),
            "{}",
            http.panggilan.lock().unwrap()[0]
        );
    }

    #[test]
    fn gagal_jaringan_menggunakan_error_template_dan_reaksi_sil() {
        let http = Stub::offline();
        let r = run(http.clone(), "fakecall", "Zann | 03:33");
        assert_eq!(
            text_of(&r),
            crate::common::error_template_bawaan(".", "fakecall")
        );
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn balasan_json_error_bukan_gambar() {
        let http = Stub::json_error();
        let r = run(http.clone(), "fakecall", "Zann | 03:33");
        assert_eq!(
            text_of(&r),
            crate::common::error_template_bawaan(".", "fakecall")
        );
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn berhasil_mengembalikan_gambar_dengan_bytes() {
        let http = Stub::body(b"PNG".as_slice());
        let r = run(http.clone(), "fakecall", "Zann | 03:33");
        assert_eq!(reaksi(&r), Some("📞"));
        assert!(r.has_media());
        match r.media_source() {
            Some(MediaSource::Bytes { data, .. }) => assert_eq!(data.as_ref(), b"PNG"),
            other => panic!("harus bytes, dapat {other:?}"),
        }
    }
}
