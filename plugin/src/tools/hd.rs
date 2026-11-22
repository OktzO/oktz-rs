//! `plugins/tools/hd.js` — enhance gambar jadi HD.
//!
//! `ponytail:` reaksi `🕕` pembuka di JS tidak dibawa; `Reply` hanya menyimpan
//! satu reaksi, jadi yang dikirim `✅`/`❌`. API `ihancer.com` mati, alurnya
//! pindah ke `rinn::hd` yang menerima URL — jadi gambar diunggah dulu lewat
//! `unggah::gambar`.

use oktz_plugin_api::{Ctx, IncomingKind, MediaSource, Reply, plugin};
use oktz_scraper::{rinn, unggah};

const ERR_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh gambar";
const ERR_API: &str =
    "Maaf, terjadi kesalahan saat memproses gambar dari scraper. Coba lagi nanti ya.";
const CAPTION: &str = "✅ *BERHASIL*\n\n> Gambar telah berhasil di-upscale dan dijernihkan.";

pub(crate) fn help(prefix: &str) -> String {
    format!(
        "*🪁 HD IMAGE*\n> Reply gambar yang mau dijernihkan\n\n`{prefix}remini`",
        prefix = prefix
    )
}

#[plugin(
    name = "remini",
    alias = ["hd", "enhance", "upscale"],
    category = "tools",
    description = "Enhance gambar jadi HD",
    usage = ".remini (reply gambar)",
    example = ".remini",
    cooldown = 15,
    energi = 1,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn hd(ctx: Ctx) -> Reply {
    let trigger = ctx.is_media_kind(IncomingKind::Image);
    let quoted = ctx.quoted().is_some_and(|q| q.kind.is_image());
    if !trigger && !quoted {
        return Reply::text(help(&ctx.prefix));
    }

    // `m.quoted?.isMedia ? quoted.download() : m.download()` (`hd.js:52`) —
    // quoted diperiksa lebih dulu, bukan gambar trigger.
    let unduhan = if ctx.quoted().is_some_and(|q| q.kind.is_image()) {
        ctx.download_quoted_media()
    } else {
        ctx.download_media()
    };
    let Ok(buffer) = unduhan else {
        return Reply::text(ERR_UNDUH).with_reaction("❌");
    };

    let hasil = unggah::gambar(ctx.http.as_ref(), &buffer)
        .and_then(|url| rinn::hd(ctx.http.as_ref(), &url).map_err(|e| e.message));
    match hasil {
        Ok(data) => {
            let mime = unggah::mime_gambar(&data);
            Reply::image(MediaSource::bytes(data).with_mimetype(mime), CAPTION).with_reaction("✅")
        }
        Err(_) => Reply::text(ERR_API).with_reaction("❌"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpCall, HttpError, HttpGet, HttpResponse, MediaError, MediaFetch, MediaOrigin,
        QuotedKind, QuotedMessage,
    };
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    /// PNG 8 byte — cukup untuk `mime_gambar`.
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    const UPLOAD: &str = r#"{"success":true,"files":[{"url":"https://cdn.example/x.jpg"}]}"#;

    /// HTTP + MediaFetch sekaligus; balasan `call` berurutan, habis = offline.
    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<(u16, Vec<u8>)>>,
        hitung: Mutex<usize>,
        panggilan: Mutex<Vec<HttpCall>>,
        unduhan: Mutex<Vec<MediaOrigin>>,
        hasil_unduh: Result<Vec<u8>, MediaError>,
    }

    impl Stub {
        fn baru(
            balasan: Vec<(u16, Vec<u8>)>,
            hasil_unduh: Result<Vec<u8>, MediaError>,
        ) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(balasan),
                hitung: Mutex::new(0),
                panggilan: Mutex::new(Vec::new()),
                unduhan: Mutex::new(Vec::new()),
                hasil_unduh,
            })
        }
    }

    impl HttpGet for Stub {
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.panggilan.lock().unwrap().push(req.clone());
            let i = {
                let mut hitung = self.hitung.lock().unwrap();
                let i = *hitung;
                *hitung += 1;
                i
            };
            match self.balasan.lock().unwrap().get(i).cloned() {
                Some((status, isi)) => Ok(HttpResponse {
                    status,
                    body: HttpBody::Bytes {
                        data: Arc::from(isi),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
    }

    impl MediaFetch for Stub {
        fn download(&self, origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.unduhan.lock().unwrap().push(origin);
            self.hasil_unduh.clone()
        }
    }

    fn media_gambar() -> Option<oktz_plugin_api::IncomingMedia> {
        Some(oktz_plugin_api::IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        })
    }

    fn ctx_stub(
        http: Arc<Stub>,
        media: Option<oktz_plugin_api::IncomingMedia>,
        quoted: Option<QuotedKind>,
        prefix: &str,
    ) -> Ctx {
        Ctx {
            text: String::new(),
            args: Vec::new(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: prefix.into(),
            command: "hd".into(),
            media,
            quoted: quoted.map(|kind| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: ME.into(),
                text: String::new(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
                timestamp: 0,
            }),
            http: http.clone(),
            media_fetch: http,
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    fn gambar(r: &Reply) -> (Vec<u8>, Option<String>) {
        match r.without_mutations().tanpa_reaksi() {
            Reply::Image { media, .. } => match media {
                Some(MediaSource::Bytes { data, mimetype }) => (data.to_vec(), mimetype.clone()),
                lain => panic!("harus bytes, dapat {lain:?}"),
            },
            lain => panic!("harus image, dapat {lain:?}"),
        }
    }

    #[test]
    fn selalu_membalas_kartu_pakai() {
        let http = Stub::baru(vec![], Ok(Vec::new()));
        let r = hd(ctx_stub(http.clone(), None, None, "."));
        assert!(teks(&r).contains("HD IMAGE"));
        assert_eq!(reaksi(&r), None);
        assert!(http.panggilan.lock().unwrap().is_empty());
    }

    #[test]
    fn sukses_unggah_lalu_hd_kirim_byte() {
        let http = Stub::baru(
            vec![(200, UPLOAD.as_bytes().to_vec()), (200, PNG.to_vec())],
            Ok(b"gambar-asli".to_vec()),
        );
        let r = hd(ctx_stub(http.clone(), media_gambar(), None, "."));

        let (data, mimetype) = gambar(&r);
        assert_eq!(data, PNG.to_vec());
        assert_eq!(mimetype.as_deref(), Some("image/png"));
        assert_eq!(teks(&r), CAPTION);
        assert_eq!(reaksi(&r), Some("✅".into()));
        assert_eq!(*http.unduhan.lock().unwrap(), [MediaOrigin::Trigger]);

        let panggilan = http.panggilan.lock().unwrap();
        assert_eq!(panggilan.len(), 2);
        assert_eq!(panggilan[0].url, "https://uguu.se/upload.php");
        assert!(
            panggilan[1]
                .url
                .starts_with("https://apiii-xrina.my.id/ai-image/hd?url="),
            "url={}",
            panggilan[1].url
        );
    }

    #[test]
    fn gambar_quoted_diunduh_dari_pesan_reply() {
        let http = Stub::baru(
            vec![(200, UPLOAD.as_bytes().to_vec()), (200, PNG.to_vec())],
            Ok(b"gambar-quoted".to_vec()),
        );
        let r = hd(ctx_stub(http.clone(), None, Some(QuotedKind::Image), "."));
        assert_eq!(gambar(&r).0, PNG.to_vec());
        assert_eq!(reaksi(&r), Some("✅".into()));
        assert_eq!(*http.unduhan.lock().unwrap(), [MediaOrigin::Quoted]);
    }

    #[test]
    fn unduhan_gagal_membalas_pesan_dan_reaksi_silang() {
        let http = Stub::baru(
            vec![],
            Err(MediaError {
                message: "cdn mati".into(),
            }),
        );
        let r = hd(ctx_stub(http.clone(), media_gambar(), None, "."));
        assert_eq!(teks(&r), ERR_UNDUH);
        assert_eq!(reaksi(&r), Some("❌".into()));
        assert!(http.panggilan.lock().unwrap().is_empty());
    }

    #[test]
    fn api_gagal_membalas_pesan_kesalahan_dan_reaksi_silang() {
        // Unggah sukses, tiga route HD (`hd`, `hd2`, `unblur`) semuanya 500.
        let http = Stub::baru(
            vec![
                (200, UPLOAD.as_bytes().to_vec()),
                (500, Vec::new()),
                (500, Vec::new()),
                (500, Vec::new()),
            ],
            Ok(b"gambar-asli".to_vec()),
        );
        let r = hd(ctx_stub(http, media_gambar(), None, "."));
        assert_eq!(teks(&r), ERR_API);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }
}
