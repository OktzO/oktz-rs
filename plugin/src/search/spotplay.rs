//! `plugins/search/spotplay.js` — putar musik dari Spotify.
//!
//! `ponytail:` JS memakai dua resolutions berurutan:
//! 1. `resolver.resolve("spotify", { q })` — cari track, ambil yang tautannya
//!    `open.spotify.com/track/` (`spotplay.js:30`).
//! 2. `resolver.resolve("spotify", { url })` — ambil link unduhan.
//!
//! Backend lokal (spotyloader) sudah ada di crate scraper: `oktz_scraper::spotify::download_spotify`
//! (`src/scraper/spotify.js`). Backend pencarian tidak punya padanan lokal —
//! sama seperti `search/spotify.rs`, pencarian memakai agregator `nexray`.
//!
//! `ponytail:` `sock.sendMedia(..., { type: "audio", mimetype: "audio/mpeg",
//! ptt: false, fileName })` — `Reply::Audio` tidak punya field nama berkas,
//! jadi nama berkas `Artis - Judul.mp3` tidak ikut (dicatat di laporan).

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use super::spotify::{cari_lagu, tautan_track_spotify};

/// `spotplay.js:19` — kartu pakai.
fn help(prefix: &str) -> String {
    format!("⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{prefix}spotplay <query>`")
}

/// `spotplay.js:34` — tidak ada track yang cocok.
const TIDAK_DITEMUKAN: &str = "❌ Lagu Spotify tidak ditemukan.";

/// `spotplay.js:41` — unduhan gagal.
const GAGAL_UNDUH: &str = "❌ Gagal mengambil link download lagu Spotify.";

#[plugin(
    name = "spotplay",
    alias = ["splay"],
    category = "search",
    description = "Putar musik dari Spotify",
    usage = ".spotplay <query>",
    example = ".spotplay neffex grateful",
    cooldown = 15,
    energi = 1,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn spotplay(ctx: Ctx) -> Reply {
    let query = ctx.text.trim().to_string();
    if query.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    let tracks = match cari_lagu(ctx.http.as_ref(), &query) {
        Ok(t) => t,
        Err(_) => {
            // `spotplay.js:52-56` — ☢ lalu template error.
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    // `spotplay.js:30` — hanya track Spotify yang bisa diputar.
    let first = tracks
        .iter()
        .find(|t| tautan_track_spotify(&t.url))
        .cloned();
    let Some(first) = first else {
        // `spotplay.js:32-35` — ❌ lalu pesan tidak ditemukan.
        return Reply::text(TIDAK_DITEMUKAN).with_reaction("❌");
    };

    let unduhan =
        match oktz_scraper::spotify::download_spotify(ctx.http.as_ref(), &first.url) {
            Ok(v) => v,
            Err(_) => {
                // `spotplay.js:52-56` — catch → ☢ + template error.
                return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
            }
        };

    let Some(url) = unduhan
        .get("url")
        .and_then(|v| v.as_str())
        .filter(|u| !u.is_empty())
    else {
        // `spotplay.js:39-42` — ❌ lalu pesan gagal unduh.
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };

    // `spotplay.js:44-49` — kirim audio.
    Reply::audio(MediaSource::url(url).with_mimetype("audio/mpeg")).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub HTTP: balasan berurutan sesuai daftar; habis itu balasan terakhir
    /// diulang (supaya unduhan spotyloader dapat balasan yang sama).
    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<String>>,
        hit: Mutex<usize>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn baru(balasan: Vec<&str>) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(balasan.iter().map(|s| s.to_string()).collect()),
                hit: Mutex::new(0),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let mut hit = self.hit.lock().unwrap();
            let b = self.balasan.lock().unwrap();
            let body = if b.is_empty() {
                String::new()
            } else {
                b[(*hit).min(b.len() - 1)].clone()
            };
            *hit += 1;
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(body.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.get(&req.url)
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            self.get("")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "spotplay".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("spotplay").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("spotplay").unwrap();
        assert_eq!(p.meta.name, "spotplay");
        assert_eq!(p.meta.alias, ["splay"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Putar musik dari Spotify");
        assert_eq!(p.meta.usage, ".spotplay <query>");
        assert_eq!(p.meta.example, ".spotplay neffex grateful");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `spotplay.js:19` — kartu pakai: `ᴄᴀʀᴀ ᴘᴀᴋᴀɪ` (bukan `ᴘᴀᴋᴜɪ`).
        assert_eq!(
            help("."),
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `.spotplay <query>`"
        );
        // `spotplay.js:34` — track tidak ditemukan.
        let http = Stub::baru(vec![r#"{"status":true,"result":[]}"#]);
        let r = run(http, "x");
        assert_eq!(text_of(&r), TIDAK_DITEMUKAN);
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn track_bukan_spotify_dilewati() {
        let http = Stub::baru(vec![
            r#"{"status":true,"result":[{"title":"Lagu","artist":"A","url":"https://example.com/x","duration":"1:00"}]}"#,
        ]);
        let r = run(http, "x");
        assert_eq!(text_of(&r), TIDAK_DITEMUKAN);
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn unduhan_tanpa_download_link_masuk_catch() {
        // Pencarian berhasil, spotyloader menjawab tanpa `downloadLink`.
        let http = Stub::baru(vec![
            r#"{"status":true,"result":[{"title":"Lagu","artist":"A","url":"https://open.spotify.com/track/x","duration":"1:00"}]}"#,
            r#"{"post":{"name":"Lagu","artist":"A"}}"#,
        ]);
        let r = run(http, "x");
        assert_eq!(text_of(&r), crate::common::error_template_bawaan(".", "spotplay"));
        assert_eq!(reaction_of(&r), Some("☢"));
    }

    #[test]
    fn audio_dikirim_dengan_url_unduhan() {
        let http = Stub::baru(vec![
            r#"{"status":true,"result":[{"title":"Lagu","artist":"A","url":"https://open.spotify.com/track/x","duration":"1:00"}]}"#,
            r#"{"downloadLink":"https://dl/x.mp3","post":{"name":"Lagu","artist":"A","mime":"audio/mpeg"}}"#,
        ]);
        let r = run(http, "x");
        assert_eq!(reaction_of(&r), Some("✅"));
        match r.without_mutations().tanpa_reaksi() {
            Reply::Audio { media, ptt } => {
                assert!(!ptt, "bukan voice note");
                match media {
                    Some(MediaSource::Url { url, mimetype }) => {
                        assert_eq!(url, "https://dl/x.mp3");
                        assert_eq!(mimetype.as_deref(), Some("audio/mpeg"));
                    }
                    other => panic!("media harus URL, dapat {other:?}"),
                }
            }
            other => panic!("harus Audio, dapat {other:?}"),
        }
    }
}
