//! `plugins/download/spotifydl.js` — unduh lagu dari Spotify.
//!
//! Urutan backend mengikuti `src/lib/resolve.js`: scraper lokal `spotyloader`
//! dulu, lalu agregator `nexray` sebagai cadangan
//! (`src/capabilities/spotify.js:13-72`). Bentuk lokal dan amplop aggregator
//! dinormalisasi terpisah.
//!
//! Audio dikirim seperti `spotifydl.js:33-38`: `{ audio: { url }, mimetype:
//! "audio/mpeg" }` → [`Reply::Audio`]. `fileName` tidak punya padanan di
//! [`MediaSource`], dan Baileys juga mengabaikannya untuk `audioMessage`.

use oktz_plugin_api::{Ctx, MediaSource as Ms, Reply, plugin};
use serde_json::Value;

use super::douyindl::query_encode;

/// `nexray` di `src/lib/aggregator.js:22`.
pub(crate) const ENDPOINT: &str = "https://api.nexray.eu.cc/downloader/spotify";

/// `spotifydl.js:18` — `/open\.spotify\.com\/track/i`.
pub(crate) fn is_track_url(text: &str) -> bool {
    text.to_lowercase().contains("open.spotify.com/track")
}

/// `spotifydl.js:19` — link hilang atau bukan track.
pub(crate) const BAD_URL: &str = "❌ *Waduh, link Spotify-nya mana nih atau kurang tepat!*\n\n\
     Kamu harus memasukkan tautan (link) lagu dari Spotify yang valid. \
     Pastikan itu adalah link ke track/lagu ya! \n\n\
     Contoh: `.spdl https://open.spotify.com/track/3RY0NyQQXxuAiyk5eAS4fC`";

/// `spotifydl.js:50` — backend menjawab tapi tidak ada URL unduhan.
pub(crate) const NO_DOWNLOAD: &str =
    "⚠️ *Gagal mengambil lagu!* \n\nServer tidak merespon dengan tautan unduhan yang valid.";

/// `spotifydl.js:53` — kegagalan proses.
pub(crate) const SYS_ERROR: &str = "😔 *Terjadi kesalahan sistem saat memproses tautan Spotify tersebut.* \
     Mohon coba lagi nanti ya!";

/// Hasil `trackUnduh` (`src/capabilities/spotify.js:131`). `Err` = respons
/// tanpa URL unduhan yang bisa dipakai — kelas yang di plugin dipisahkan dari
/// kegagalan proses lewat `adaNormalisasiGagal`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Track {
    pub title: String,
    pub artist: String,
    pub url: String,
}

/// Kegagalan normalisasi aggregator dipisahkan dari kegagalan backend.
/// `status: false` dilempar oleh `lewatNexray` sebelum `normalize`, jadi bukan
/// bukti link tidak ada (`src/capabilities/spotify.js:57-72`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AggregatorFailure {
    Backend,
    Normalization,
}

/// `trackUnduh` (`src/capabilities/spotify.js:131-146`).
fn track_dari(isi: &Value) -> Result<Track, ()> {
    let raw = isi
        .get("url")
        .filter(|value| !value.is_null())
        .or_else(|| isi.get("download_url"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let Some((scheme, _)) = raw.split_once("://") else {
        return Err(());
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Err(());
    }
    let text = |key: &str| {
        isi.get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string()
    };
    Ok(Track {
        title: text("title"),
        artist: text("artist"),
        url: raw,
    })
}

/// Bentuk lokal `downloadSpotify` (`src/scraper/spotify.js:8-53`).
fn normalize_local(body: &Value) -> Result<Track, ()> {
    track_dari(body)
}

/// Bentuk amplop aggregator (`isiAmplop`, `src/capabilities/spotify.js:96-108`).
fn normalize_aggregator(body: &Value) -> Result<Track, AggregatorFailure> {
    let envelope =
        body.is_array() || (body.get("status").is_some() || body.get("result").is_some());
    if !envelope {
        return track_dari(body).map_err(|_| AggregatorFailure::Normalization);
    }
    if !body
        .get("status")
        .map(oktz_scraper::common::truthy)
        .unwrap_or(false)
    {
        return Err(AggregatorFailure::Backend);
    }
    let result = body
        .get("result")
        .filter(|value| value.is_object())
        .ok_or(AggregatorFailure::Normalization)?;
    track_dari(result).map_err(|_| AggregatorFailure::Normalization)
}

/// `spotifydl.js:31` — `${artist || "Spotify"} - ${title || "Audio"}.mp3`.
pub(crate) fn filename(track: &Track) -> String {
    let artist = if track.artist.is_empty() {
        "Spotify"
    } else {
        &track.artist
    };
    let title = if track.title.is_empty() {
        "Audio"
    } else {
        &track.title
    };
    format!("{artist} - {title}.mp3")
}

#[plugin(
    name = "spotifydl",
    alias = ["spdl", "spotify-dl", "spotdl"],
    category = "download",
    description = "Unduh lagu favoritmu langsung dari Spotify tanpa ribet!",
    usage = ".spdl <link>",
    example = ".spdl https://open.spotify.com/track/...",
    cooldown = 15,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn spotifydl(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    if text.is_empty() || !is_track_url(text) {
        return Reply::text(BAD_URL);
    }

    // `resolve.js` mencoba backend lokal sebelum backend API. Error lokal
    // tidak menghentikan aggregator; hanya normalisasi yang gagal yang membuat
    // pesan tautan, sisanya adalah kegagalan proses.
    let mut normalisasi_gagal = false;
    let local = match oktz_scraper::spotify::download_spotify(ctx.http.as_ref(), text) {
        Ok(body) => match normalize_local(&body) {
            Ok(track) => Some(track),
            Err(()) => {
                normalisasi_gagal = true;
                None
            }
        },
        Err(_) => None,
    };
    let data = local.or_else(|| {
        let url = format!("{ENDPOINT}?url={}", query_encode(text));
        let body = ctx.http.get(&url).ok().and_then(|response| response.json());
        match body.as_ref().map(|body| normalize_aggregator(body)) {
            Some(Ok(track)) => Some(track),
            Some(Err(AggregatorFailure::Normalization)) => {
                normalisasi_gagal = true;
                None
            }
            Some(Err(AggregatorFailure::Backend)) | None => None,
        }
    });

    match data {
        // `spotifydl.js:33-40` — `{ audio: { url }, mimetype: "audio/mpeg" }`
        // lalu reaksi ✅.
        Some(track) => {
            Reply::audio(Ms::url(track.url).with_mimetype("audio/mpeg")).with_reaction("✅")
        }
        None if normalisasi_gagal => Reply::text(NO_DOWNLOAD).with_reaction("❌"),
        None => Reply::text(SYS_ERROR).with_reaction("❌"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        local: Result<String, HttpError>,
        aggregator: Result<String, HttpError>,
        seen: Mutex<Vec<oktz_plugin_api::HttpCall>>,
    }

    impl Stub {
        fn body(v: Value) -> Arc<Stub> {
            Arc::new(Stub {
                local: Err(HttpError {
                    message: "local offline".into(),
                }),
                aggregator: Ok(v.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn local(v: Value) -> Arc<Stub> {
            Arc::new(Stub {
                local: Ok(v.to_string()),
                aggregator: Err(HttpError {
                    message: "aggregator offline".into(),
                }),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn pair(local: Value, aggregator: Value) -> Arc<Stub> {
            Arc::new(Stub {
                local: Ok(local.to_string()),
                aggregator: Ok(aggregator.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                local: Err(HttpError {
                    message: "local offline".into(),
                }),
                aggregator: Err(HttpError {
                    message: "aggregator offline".into(),
                }),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen
                .lock()
                .unwrap()
                .push(oktz_plugin_api::HttpCall::get(url));
            let text = self.aggregator.clone()?;
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper lokal memakai HttpCall::call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper lokal memakai HttpCall::call")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper lokal memakai HttpCall::call")
        }

        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
            let text = match req.method {
                oktz_plugin_api::HttpMethod::Post => self.local.clone()?,
                oktz_plugin_api::HttpMethod::Get => self.aggregator.clone()?,
                lain => panic!("metode {lain:?} tidak dipakai"),
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "spdl".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: vec![text.into()],
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("spotifydl").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn ok_body() -> Value {
        serde_json::json!({
            "status": true,
            "result": {
                "title": "Indonesia Raya",
                "artist": "Nasional",
                "url": "https://cdn.example/a.mp3",
                "mime": "audio/mpeg"
            }
        })
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("spotifydl").unwrap();
        assert_eq!(p.meta.name, "spotifydl");
        assert_eq!(p.meta.alias, ["spdl", "spotify-dl", "spotdl"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(
            p.meta.description,
            "Unduh lagu favoritmu langsung dari Spotify tanpa ribet!"
        );
        assert_eq!(p.meta.usage, ".spdl <link>");
        assert_eq!(p.meta.example, ".spdl https://open.spotify.com/track/...");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["spdl", "spotify-dl", "spotdl"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "spotifydl");
        }
    }

    #[test]
    fn only_track_urls_are_accepted() {
        assert!(is_track_url("https://open.spotify.com/track/abc"));
        assert!(is_track_url("HTTPS://OPEN.SPOTIFY.COM/TRACK/abc"));
        for bad in [
            "https://open.spotify.com/album/abc",
            "https://open.spotify.com/playlist/abc",
            "https://spotify.com/track/abc",
            "",
        ] {
            assert!(!is_track_url(bad), "harus ditolak: {bad}");
        }
    }

    #[test]
    fn a_missing_or_non_track_link_is_refused_without_a_request() {
        let http = Stub::body(ok_body());
        for text in ["", "   ", "https://open.spotify.com/album/x"] {
            assert_eq!(text_of(&run(http.clone(), text)), BAD_URL, "input {text:?}");
        }
        assert!(http.seen.lock().unwrap().is_empty());
    }

    /// `Reply::Audio` dari balasan, kalau ada.
    fn audio_dari(r: &Reply) -> Option<(String, bool)> {
        match r {
            Reply::Audio { media, .. } => {
                let m = media.as_ref()?;
                Some((m.mimetype().unwrap_or("").to_string(), true))
            }
            _ => None,
        }
    }

    /// Isi balasan sebenarnya: `without_mutations` hanya membuang `Mutate`,
    /// jadi `Reaction { reply }` masih harus dibuka satu lapis.
    fn isi(r: &Reply) -> &Reply {
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        }
    }

    #[test]
    fn a_resolved_track_sends_the_audio_media() {
        let http = Stub::body(ok_body());
        let r = run(
            http.clone(),
            "https://open.spotify.com/track/3RY0NyQQXxuAiyk5eAS4fC",
        );
        // `spotifydl.js:33-40` — media audio + reaksi ✅.
        let (mime, _) = audio_dari(isi(&r)).expect("audio harus terkirim");
        assert_eq!(mime, "audio/mpeg");
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].method, oktz_plugin_api::HttpMethod::Post);
        assert_eq!(seen[0].url, "https://spotyloader.com/api/spotify/track");
        assert_eq!(seen[1].method, oktz_plugin_api::HttpMethod::Get);
        assert!(
            seen[1]
                .url
                .starts_with("https://api.nexray.eu.cc/downloader/spotify?url=https%3A%2F%2F"),
            "{}",
            seen[1].url
        );
    }

    #[test]
    fn scraper_lokal_sukses_dipakai_sebelum_aggregator() {
        let http = Stub::local(serde_json::json!({
            "downloadLink": "https://cdn.example/local.mp3",
            "post": {
                "name": "Lagu Lokal",
                "artist": "Artis Lokal",
                "mime": "audio/mpeg"
            }
        }));
        let r = run(http.clone(), "https://open.spotify.com/track/x");

        assert!(audio_dari(isi(&r)).is_some());
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(
            seen.len(),
            1,
            "aggregator tidak boleh dihubungi setelah local sukses"
        );
        assert_eq!(seen[0].method, oktz_plugin_api::HttpMethod::Post);
        assert_eq!(seen[0].url, "https://spotyloader.com/api/spotify/track");
    }

    #[test]
    fn normalisasi_lokal_gagal_lanjut_ke_aggregator() {
        let http = Stub::pair(
            serde_json::json!({
                "downloadLink": "ftp://invalid/audio.mp3",
                "post": { "name": "Lagu", "artist": "Artis" }
            }),
            ok_body(),
        );
        let r = run(http.clone(), "https://open.spotify.com/track/x");

        assert!(audio_dari(isi(&r)).is_some());
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].method, oktz_plugin_api::HttpMethod::Post);
        assert_eq!(seen[1].method, oktz_plugin_api::HttpMethod::Get);
    }

    #[test]
    fn missing_artist_and_title_still_send_the_audio() {
        let body =
            serde_json::json!({ "status": true, "result": { "url": "https://cdn.example/a.mp3" } });
        let r = run(Stub::body(body), "https://open.spotify.com/track/x");
        assert!(
            audio_dari(isi(&r)).is_some(),
            "audio tetap terkirim tanpa judul"
        );
    }

    #[test]
    fn a_response_without_a_download_url_gets_the_amber_warning() {
        // `adaNormalisasiGagal` (`spotifydl.js:48`): backend menjawab normalisasi
        // gagal karena tidak ada yang bisa dikirim — track tidak ada atau
        // terkunci region.
        for body in [
            serde_json::json!({ "status": true, "result": { "title": "x", "url": "ftp://x" } }),
            serde_json::json!({ "status": true, "result": {} }),
            serde_json::json!({ "status": true }),
            serde_json::json!({}),
        ] {
            let r = run(Stub::body(body.clone()), "https://open.spotify.com/track/x");
            assert_eq!(text_of(&r), NO_DOWNLOAD, "body {body}");
            assert!(
                matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "❌"),
                "harus bereaksi ❌"
            );
        }
    }

    #[test]
    fn aggregator_status_false_adalah_kegagalan_backend_bukan_normalisasi_gagal() {
        // `lewatNexray` melempar sebelum `normalize` di `src/capabilities/spotify.js`.
        // Jadi `status: false` tidak menghasilkan awalan `normalisasi gagal:` dan
        // pesan yang benar adalah kegagalan proses, bukan tautan tidak ada.
        let r = run(
            Stub::body(serde_json::json!({ "status": false, "msg": "not found" })),
            "https://open.spotify.com/track/x",
        );
        assert_eq!(text_of(&r), SYS_ERROR);
    }

    #[test]
    fn a_transport_failure_reports_the_system_error_instead() {
        // Kegagalan proses harus dilaporkan sebagai kegagalan proses, bukan
        // disamarkan jadi "tidak ada tautan".
        let r = run(Stub::offline(), "https://open.spotify.com/track/x");
        assert_eq!(text_of(&r), SYS_ERROR);
    }

    #[test]
    fn download_url_from_the_other_field_name_is_also_accepted() {
        let body = serde_json::json!({
            "status": true,
            "result": { "download_url": "https://cdn.example/b.mp3", "title": "T" }
        });
        let r = run(Stub::body(body), "https://open.spotify.com/track/x");
        assert!(audio_dari(isi(&r)).is_some(), "audio tetap terkirim");
    }

    #[test]
    #[ignore = "uji jaringan Spotify; set OKTZ_TEST_SPOTIFY_URL"]
    fn live_spotifydl_mengambil_audio() {
        // Uji live padanan `plugins/download/spotifydl.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_SPOTIFY_URL");
        let (http, ctx) = live_ctx("spotifydl", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("spotifydl").unwrap().handler)(ctx);
        assert_media(&reply, &["audio"], &http);
    }
}
