//! `plugins/random/anime.js` — random gambar anime/reaksi (Nexray Source).
//!
//! `name` di JS berupa array `["loli", ...nexrayTypes]` — di Rust jadi
//! `name = "loli"` + `commands = [...nexrayTypes]` (map `commands` JS),
//! dan lookup `commands` lebih dulu daripada alias, persis
//! `plugins.js:539-544`. Efeknya `waifu` milik `anime/mywaifu.js` (alias)
//! kalah dari plugin ini, sama seperti di JS.
//!
//! `ponytail:` pengiriman view-once interaktif (`generateWAMessageFromContent`
//! + `relayMessage`) dan tombol `quick_reply` "Lanjut Cari Lagi?" tidak ada
//! di `Reply` — gambar dikirim biasa. Reaksi `🖼️` (`anime.js:53`) dibuang:
//! membungkus media dengan `with_reaction` membuat gambar tidak pernah
//! di-upload (lihat `random/cecanindo.rs`). Deteksi GIF tetap jalan: byte
//! diambil lewat `ctx.http` lalu dicek magic `GIF` seperti `anime.js:56-57`,
//! dan GIF dikirim sebagai video `gifPlayback`.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use crate::common::error_template;

/// Daftar tipe Nexray (`anime.js:7-12`) — juga nama command tambahan.
pub(crate) const NEXRAY_TYPES: [&str; 29] = [
    "waifu", "neko", "shinobu", "megumin", "bully", "cuddle", "cry", "hug", "awoo", "kiss",
    "lick", "pat", "smug", "bonk", "yeet", "blush", "smile", "wave", "highfive", "handhold",
    "nom", "bite", "glomp", "slap", "kill", "happy", "poke", "dance", "cringe",
];

/// URL gambar loli (`anime.js:38`).
pub(crate) const LOLI_URL: &str = "https://api.nexray.web.id/random/loli";

/// `https://api.nexray.eu.cc/random/anime?type=${cmd}` (`anime.js:55`).
pub(crate) fn url_tipe(tipe: &str) -> String {
    format!("https://api.nexray.eu.cc/random/anime?type={tipe}")
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
    description = "Random gambar anime/reaction (Nexray Source)",
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
        return Reply::image(MediaSource::url(LOLI_URL), "👧 *ʀᴀɴᴅᴏᴍ ʟᴏʟɪ*");
    }

    // Tipe Nexray: unduh byte, deteksi GIF dari magic-nya (`anime.js:52-58`).
    if NEXRAY_TYPES.contains(&cmd.as_str()) {
        let ok = ctx
            .http
            .get(&url_tipe(&cmd))
            .ok()
            .filter(|res| (200..300).contains(&res.status))
            .map(|res| res.body.bytes().to_vec());
        let Some(data) = ok else {
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

    /// HTTP stub yang selalu sukses dengan body tetap — untuk jalur tipe.
    struct Tetap(Vec<u8>);
    impl std::fmt::Debug for Tetap {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Tetap")
        }
    }
    impl oktz_plugin_api::HttpGet for Tetap {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            assert!(url.starts_with("https://api.nexray.eu.cc/random/anime?type="));
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: std::sync::Arc::from(self.0.clone().into_boxed_slice()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError { message: "tidak dipakai".into() })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError { message: "tidak dipakai".into() })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError { message: "tidak dipakai".into() })
        }
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

    fn ctx_premium(command: &str) -> Ctx {
        Ctx {
            is_premium: true,
            ..ctx_of(command)
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
            "Random gambar anime/reaction (Nexray Source)"
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
    fn loli_premium_mengirim_url_loli() {
        let r = anime(ctx_premium("loli"));
        assert!(r.has_media(), "harus image");
        match r.media_source().unwrap() {
            MediaSource::Url { url, .. } => assert_eq!(url, LOLI_URL),
            other => panic!("sumber harus Url, dapat {other:?}"),
        }
        assert_eq!(r.text_body().unwrap(), "👧 *ʀᴀɴᴅᴏᴍ ʟᴏʟɪ*");
    }

    #[test]
    fn tipe_nexray_mengirim_gambar_dengan_caption_kapital() {
        let ctx = Ctx {
            http: std::sync::Arc::new(Tetap(vec![0x89, b'P', b'N', b'G'])),
            ..ctx_of("waifu")
        };
        let r = anime(ctx);
        assert!(r.has_media());
        match r.media_source().unwrap() {
            MediaSource::Bytes { data, .. } => assert_eq!(data.as_ref(), &[0x89, b'P', b'N', b'G']),
            other => panic!("sumber harus Bytes, dapat {other:?}"),
        }
        assert_eq!(r.text_body().unwrap(), caption_tipe("waifu"));
        assert_eq!(caption_tipe("hug"), "✨ *ʀᴀɴᴅᴏᴍ HUG*");
    }

    #[test]
    fn byte_gif_dikirim_sebagai_video_gif_playback() {
        let ctx = Ctx {
            http: std::sync::Arc::new(Tetap(vec![b'G', b'I', b'F', b'8'])),
            ..ctx_of("hug")
        };
        let r = anime(ctx);
        assert!(r.has_media());
        assert!(
            matches!(r.without_mutations(), Reply::Video { .. }),
            "GIF harus jadi Video, dapat {:?}",
            r.without_mutations()
        );
        assert_eq!(r.text_body().unwrap(), caption_tipe("hug"));
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
        let teks = r.text_body().unwrap().to_string();
        assert!(!teks.is_empty(), "harus teks error template");
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "☢"),
            other => panic!("harus reaksi ☢, dapat {other:?}"),
        }
    }
}
