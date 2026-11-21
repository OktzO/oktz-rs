//! `plugins/random/anime.js` — random gambar anime/reaksi (nekos.best).
//!
//! `name` di JS berupa array `["loli", ...nexrayTypes]` — di Rust jadi
//! `name = "loli"` + `commands = [...nexrayTypes]` (map `commands` JS),
//! dan lookup `commands` lebih dulu daripada alias, persis
//! `plugins.js:539-544`. Efeknya `waifu` milik `anime/mywaifu.js` (alias)
//! kalah dari plugin ini, sama seperti di JS.
//!
//! `ponytail:` sumber lama `api.nexray.eu.cc/random/anime` (500) dan
//! `api.nexray.web.id/random/loli` (301 ke host mati) sudah mati sejak
//! 2026-11-15; tipe yang didukung pindah ke `nekos.best/api/v2/<tipe>` dan
//! `lick` ke `otakugifs.xyz`. Alur: unduh JSON (wajib UA browser — tanpa UA
//! nekos.best balas 403), ambil `results[0].url`/`url`, lalu unduh byte-nya.
//! 7 tipe tanpa sumber hidup dipetakan ke tipe terdekat — lihat [`tipe_efektif`].
//!
//! `ponytail:` pengiriman view-once interaktif
//! (`generateWAMessageFromContent` + `relayMessage`) dan tombol `quick_reply`
//! "Lanjut Cari Lagi?" tidak ada di `Reply` — gambar dikirim biasa.
//! Reaksi `🖼️` (`anime.js:53`) dibuang:
//! membungkus media dengan `with_reaction` membuat gambar tidak pernah
//! di-upload (lihat `random/cecanindo.rs`). Deteksi GIF tetap jalan: byte
//! diambil lewat `ctx.http` lalu dicek magic `GIF` seperti `anime.js:56-57`,
//! dan GIF dikirim sebagai video `gifPlayback`.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

use crate::common::error_template;

/// Daftar tipe Nexray (`anime.js:7-12`) — juga nama command tambahan.
pub(crate) const NEXRAY_TYPES: [&str; 29] = [
    "waifu", "neko", "shinobu", "megumin", "bully", "cuddle", "cry", "hug", "awoo", "kiss", "lick",
    "pat", "smug", "bonk", "yeet", "blush", "smile", "wave", "highfive", "handhold", "nom", "bite",
    "glomp", "slap", "kill", "happy", "poke", "dance", "cringe",
];

/// UA browser — Cloudflare di depan nekos.best menolak klien tanpa UA.
const UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36";

/// Sumber `.loli` (`anime.js:38`) — pengganti `api.nexray.web.id/random/loli`
/// yang mati (301). Endpoint ini JSON, jadi byte gambarnya diunduh lewat
/// [`ambil_gambar`].
pub(crate) const LOLI_URL: &str = "https://nekos.best/api/v2/waifu";

/// Peta 7 tipe yang sumbernya mati ke tipe hidup terdekat.
///
/// `ponytail:` `api.nexray.eu.cc/random/anime` mati (500) sejak 2026-11-15 dan
/// tidak ada sumber pengganti untuk tipe ini. Peta bisa dibuang kalau sumber
/// aslinya hidup lagi.
fn tipe_efektif(tipe: &str) -> &str {
    match tipe {
        "shinobu" | "megumin" => "waifu",
        "bully" => "bonk",
        "awoo" => "happy",
        "glomp" => "hug",
        "kill" => "slap",
        "cringe" => "smile",
        lain => lain,
    }
}

/// URL JSON sumber gambar: `nekos.best` per tipe, `otakugifs` untuk `lick`.
fn url_json(tipe: &str) -> String {
    if tipe == "lick" {
        "https://api.otakugifs.xyz/gif?reaction=lick&format=gif".to_string()
    } else {
        format!("https://nekos.best/api/v2/{}", tipe_efektif(tipe))
    }
}

/// Ambil URL gambar dari JSON kedua sumber (`results[0].url` atau `url`).
fn url_gambar(body: &[u8]) -> Option<String> {
    let v: Value = serde_json::from_slice(body).ok()?;
    v.pointer("/results/0/url")
        .or_else(|| v.get("url"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Unduh byte gambar: panggil JSON (dengan UA) lalu unduh URL gambarnya.
fn ambil_gambar(ctx: &Ctx, json_url: &str) -> Option<Vec<u8>> {
    let res = ctx
        .http
        .call(&HttpCall::get(json_url).header("User-Agent", UA))
        .ok()?;
    if !(200..300).contains(&res.status) {
        return None;
    }
    let url = url_gambar(res.body.bytes())?;
    // CDN nekos.best juga di belakang Cloudflare: tanpa UA browser balas 403.
    let img = ctx
        .http
        .call(&HttpCall::get(&url).header("User-Agent", UA))
        .ok()?;
    if !(200..300).contains(&img.status) {
        return None;
    }
    Some(img.body.bytes().to_vec())
}

/// Pesan premium (`anime.js:35-37`).
pub(crate) fn teks_premium(prefix: &str) -> String {
    format!(
        "🔞 *Konten Premium*\n\nKonten ini khusus pengguna *Premium*.\n> Ketik *{prefix}benefitpremium* untuk info upgrade"
    )
}

/// `✨ *ʀᴀɴᴅᴏᴍ ${cmd.toUpperCase()}*` (`anime.js:82`).
pub(crate) fn caption_tipe(tipe: &str) -> String {
    format!("✨ *ʀᴀɴᴅᴏᴍ {}*", tipe.to_uppercase())
}

#[plugin(
    name = "loli",
    commands = [
        "waifu", "neko", "shinobu", "megumin", "bully", "cuddle", "cry", "hug",
        "awoo", "kiss", "lick", "pat", "smug", "bonk", "yeet", "blush", "smile",
        "wave", "highfive", "handhold", "nom", "bite", "glomp", "slap", "kill",
        "happy", "poke", "dance", "cringe"
    ],
    alias = [],
    category = "random",
    description = "Random gambar anime/reaction (nekos.best Source)",
    usage = ".<nama> (lihat daftar di bawah)",
    example = ".waifu",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn anime(ctx: Ctx) -> Reply {
    let cmd = ctx.command.to_lowercase();

    // `.loli` — khusus premium/owner (`anime.js:33-42`).
    if cmd == "loli" {
        if !ctx.is_premium && !ctx.is_owner {
            return Reply::text(teks_premium(&ctx.prefix));
        }
        let Some(data) = ambil_gambar(&ctx, LOLI_URL) else {
            return Reply::text(error_template(&ctx)).with_reaction("☢");
        };
        return Reply::image(MediaSource::bytes(data), "👧 *ʀᴀɴᴅᴏᴍ ʟᴏʟɪ*");
    }

    // Tipe Nexray: unduh byte, deteksi GIF dari magic-nya (`anime.js:52-58`).
    if NEXRAY_TYPES.contains(&cmd.as_str()) {
        let Some(data) = ambil_gambar(&ctx, &url_json(&cmd)) else {
            // `catch` → `m.react('☢')` + `te(...)` (`anime.js:100-103`).
            return Reply::text(error_template(&ctx)).with_reaction("☢");
        };
        let is_gif = data.len() > 3 && data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46;
        let sumber = MediaSource::bytes(data);
        return if is_gif {
            Reply::video_gif(sumber, caption_tipe(&cmd))
        } else {
            Reply::image(sumber, caption_tipe(&cmd))
        };
    }

    Reply::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpResponse};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    /// Stub berurutan: balasan dipakai berurutan — JSON dulu, lalu byte gambar.
    /// `call` mencatat header UA supaya regresi "CDN 403 tanpa UA" terjaga.
    #[derive(Debug)]
    struct Stub {
        urls: Mutex<Vec<String>>,
        ua: Mutex<Vec<String>>,
        balasan: Mutex<VecDeque<(u16, Vec<u8>)>>,
    }

    impl Stub {
        fn baru(balasan: Vec<(u16, Vec<u8>)>) -> Arc<Self> {
            Arc::new(Self {
                urls: Mutex::new(Vec::new()),
                ua: Mutex::new(Vec::new()),
                balasan: Mutex::new(balasan.into()),
            })
        }
    }

    impl oktz_plugin_api::HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.urls.lock().unwrap().push(url.to_string());
            let (status, data) = self
                .balasan
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or((200, Vec::new()));
            Ok(HttpResponse {
                status,
                body: HttpBody::Bytes {
                    data: Arc::from(data),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.ua.lock().unwrap().push(
                req.headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("user-agent"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default(),
            );
            self.get(&req.url)
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "tidak dipakai".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "tidak dipakai".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "tidak dipakai".into(),
            })
        }
    }

    /// Balasan JSON `nekos.best` dengan satu URL gambar.
    fn json_nekos(url: &str) -> (u16, Vec<u8>) {
        (
            200,
            format!(r#"{{"results":[{{"url":"{url}"}}]}}"#).into_bytes(),
        )
    }

    fn ctx_of(command: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: command.into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Ctx::default()
        }
    }

    fn ctx_stub(command: &str, http: Arc<Stub>) -> Ctx {
        Ctx {
            http,
            ..ctx_of(command)
        }
    }

    /// Reaksi ☢ dari cabang error.
    fn reaksi_radiasi(r: &Reply) {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "☢"),
            other => panic!("harus reaksi ☢, dapat {other:?}"),
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("loli").unwrap();
        assert_eq!(p.meta.name, "loli");
        assert_eq!(p.meta.commands, NEXRAY_TYPES.to_vec());
        assert!(p.meta.alias.is_empty());
        assert_eq!(p.meta.category, "random");
        assert_eq!(
            p.meta.description,
            "Random gambar anime/reaction (nekos.best Source)"
        );
        assert_eq!(p.meta.usage, ".<nama> (lihat daftar di bawah)");
        assert_eq!(p.meta.example, ".waifu");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn semua_tipe_nexray_menunjuk_plugin_ini() {
        for tipe in NEXRAY_TYPES {
            assert_eq!(
                oktz_plugin_api::find(tipe).map(|p| p.meta.name.as_str()),
                Some("loli"),
                "command {tipe} salah resolve"
            );
        }
    }

    #[test]
    fn loli_ditolak_untuk_bukan_premium() {
        let r = anime(ctx_of("loli"));
        assert_eq!(
            r.text_body().unwrap(),
            teks_premium("."),
            "harus pesan premium"
        );
        assert!(!r.has_media());
    }

    #[test]
    fn loli_premium_mengunduh_gambar_nekos() {
        let s = Stub::baru(vec![
            json_nekos("https://nekos.best/img/a.png"),
            (200, vec![0x89, b'P', b'N', b'G']),
        ]);
        let ctx = Ctx {
            is_premium: true,
            ..ctx_stub("loli", s.clone())
        };
        let r = anime(ctx);
        assert!(r.has_media(), "harus image");
        match r.media_source().unwrap() {
            MediaSource::Bytes { data, .. } => {
                assert_eq!(data.as_ref(), &[0x89, b'P', b'N', b'G'])
            }
            other => panic!("sumber harus Bytes, dapat {other:?}"),
        }
        assert_eq!(r.text_body().unwrap(), "👧 *ʀᴀɴᴅᴏᴍ ʟᴏʟɪ*");
        assert_eq!(
            *s.urls.lock().unwrap(),
            vec![
                LOLI_URL.to_string(),
                "https://nekos.best/img/a.png".to_string()
            ]
        );
    }

    #[test]
    fn tipe_nexray_mengirim_gambar_dengan_caption_kapital() {
        let s = Stub::baru(vec![
            json_nekos("https://nekos.best/img/a.png"),
            (200, vec![0x89, b'P', b'N', b'G']),
        ]);
        let r = anime(ctx_stub("waifu", s.clone()));
        assert!(r.has_media());
        match r.media_source().unwrap() {
            MediaSource::Bytes { data, .. } => assert_eq!(data.as_ref(), &[0x89, b'P', b'N', b'G']),
            other => panic!("sumber harus Bytes, dapat {other:?}"),
        }
        assert_eq!(r.text_body().unwrap(), caption_tipe("waifu"));
        assert_eq!(caption_tipe("hug"), "✨ *ʀᴀɴᴅᴏᴍ HUG*");
        assert_eq!(
            *s.urls.lock().unwrap(),
            vec![
                "https://nekos.best/api/v2/waifu".to_string(),
                "https://nekos.best/img/a.png".to_string(),
            ]
        );
    }

    #[test]
    fn byte_gif_dikirim_sebagai_video_gif_playback() {
        let s = Stub::baru(vec![
            json_nekos("https://nekos.best/img/a.gif"),
            (200, vec![b'G', b'I', b'F', b'8']),
        ]);
        let r = anime(ctx_stub("hug", s.clone()));
        assert!(r.has_media());
        assert!(
            matches!(r.without_mutations(), Reply::Video { .. }),
            "GIF harus jadi Video, dapat {:?}",
            r.without_mutations()
        );
        assert_eq!(r.text_body().unwrap(), caption_tipe("hug"));
        assert_eq!(s.urls.lock().unwrap()[0], "https://nekos.best/api/v2/hug");
    }

    #[test]
    fn tipe_mati_dipetakan_ke_tipe_hidup() {
        let peta = [
            ("shinobu", "waifu"),
            ("megumin", "waifu"),
            ("bully", "bonk"),
            ("awoo", "happy"),
            ("glomp", "hug"),
            ("kill", "slap"),
            ("cringe", "smile"),
        ];
        for (dari, ke) in peta {
            let s = Stub::baru(vec![
                json_nekos("https://nekos.best/img/a.png"),
                (200, vec![b'P']),
            ]);
            let r = anime(ctx_stub(dari, s.clone()));
            assert!(r.has_media(), "{dari} harus punya media");
            assert_eq!(
                s.urls.lock().unwrap()[0],
                format!("https://nekos.best/api/v2/{ke}"),
                "peta {dari} salah"
            );
            // Caption pakai nama tipe asli, bukan tipe target.
            assert_eq!(r.text_body().unwrap(), caption_tipe(dari));
        }
    }

    #[test]
    fn lick_memakai_otakugifs() {
        let s = Stub::baru(vec![
            (
                200,
                br#"{"url":"https://api.otakugifs.xyz/gif/lick.gif"}"#.to_vec(),
            ),
            (200, vec![b'G', b'I', b'F', b'8']),
        ]);
        let r = anime(ctx_stub("lick", s.clone()));
        assert!(
            matches!(r.without_mutations(), Reply::Video { .. }),
            "GIF harus jadi Video, dapat {:?}",
            r.without_mutations()
        );
        let urls = s.urls.lock().unwrap();
        assert_eq!(
            urls[0],
            "https://api.otakugifs.xyz/gif?reaction=lick&format=gif"
        );
        assert_eq!(urls[1], "https://api.otakugifs.xyz/gif/lick.gif");
    }

    #[test]
    fn tipe_tidak_kenal_tidak_membalas() {
        let r = anime(ctx_of("bukanwaifu"));
        assert!(r.is_none());
    }

    #[test]
    fn http_gagal_jatuh_ke_teks_error_dan_reaksi_radiasi() {
        // Ctx::default() membawa NoHttp — get() selalu gagal.
        let ctx = ctx_of("neko");
        let r = anime(ctx);
        assert!(
            !r.text_body().unwrap().is_empty(),
            "harus teks error template"
        );
        reaksi_radiasi(&r);
    }

    #[test]
    fn json_tanpa_url_jatuh_ke_error() {
        let s = Stub::baru(vec![(200, b"{}".to_vec())]);
        let r = anime(ctx_stub("neko", s.clone()));
        assert_eq!(s.urls.lock().unwrap()[0], "https://nekos.best/api/v2/neko");
        assert!(!r.text_body().unwrap().is_empty());
        reaksi_radiasi(&r);
    }

    #[test]
    fn gagal_unduh_gambar_jatuh_ke_error() {
        let s = Stub::baru(vec![
            json_nekos("https://nekos.best/img/a.png"),
            (404, Vec::new()),
        ]);
        let r = anime(ctx_stub("neko", s));
        assert!(!r.has_media());
        assert!(!r.text_body().unwrap().is_empty());
        reaksi_radiasi(&r);
    }

    #[test]
    fn json_dan_gambar_sama_sama_membawa_ua_browser() {
        let s = Stub::baru(vec![
            json_nekos("https://nekos.best/img/a.png"),
            (200, vec![0x89, b'P', b'N', b'G']),
        ]);
        let _ = anime(ctx_stub("waifu", s.clone()));
        let ua = s.ua.lock().unwrap().clone();
        assert_eq!(ua.len(), 2, "JSON + gambar");
        assert!(
            ua.iter().all(|u| u == UA),
            "UA browser wajib di keduanya: {ua:?}"
        );
    }
}
