//! `plugins/search/spotify.js` — mencari daftar lagu di Spotify.
//!
//! `ponytail:` JS memakai `resolver.resolve("spotify", { q })`, yang hanya
//! dilayani backend agregator `nexray` (`src/capabilities/spotify.js:149-160`):
//! `GET https://api.nexray.eu.cc/search/spotify?q=…`. Path-nya TANPA prefix
//! `/api` — menambahkannya membuat host menjawab 404 (sama seperti
//! `search/applemusic.rs`).
//!
//! Bentuk respons dinormalkan di `normalize` (`src/capabilities/spotify.js:148`):
//! amplop `{ status, result }` dibuka, `result` berupa array dibaca sebagai
//! daftar track, dan tiap track disaring `trackCari` (`:110-129`) — entri
//! tanpa judul atau tanpa tautan dibuang, `duration` yang bukan string
//! menjadi `"0:00"`.
//!
//! `ponytail:` tombol "Unduh Lagu Pertama" hanya dikirim kalaucover pertama
//! bisa diambil (dikecilkan ke 300×170 lewat `sharp`) DAN tautannya cocok
//! `open.spotify.com/track/`. Kalau salah satu tidak terpenuhi, plugin
//! mengirim teks polos — persis `spotify.js:62-104`.

use oktz_plugin_api::{
    Ctx, HttpError, HttpGet, Reply, plugin,
};
use oktz_plugin_api::registry::{
    CardContext, CardHeader, CardLocation, InteractiveButton,
};
use serde_json::Value;

/// `aggregator.hit("nexray", "/search/spotify", { params: { q } })`
/// (`src/capabilities/spotify.js:153-156`). Path tanpa prefix `/api`.
pub(crate) const ENDPOINT: &str = "https://api.nexray.eu.cc/search/spotify";

/// Satu lagu hasil pencarian — bentuk `trackCari`
/// (`src/capabilities/spotify.js:110-129`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Track {
    pub title: String,
    pub artist: String,
    pub url: String,
    pub cover: String,
    pub duration: String,
}

/// Pencarian lagu di Spotify.
///
/// `isiAmplop` + `normalize` (`src/capabilities/spotify.js:110-160`).
pub(crate) fn cari_lagu(http: &dyn HttpGet, q: &str) -> Result<Vec<Track>, HttpError> {
    let url = format!("{ENDPOINT}?q={}", encode_q(q));
    let res = http.get(&url)?;
    let raw = res.json().unwrap_or(Value::Null);

    // `isiAmplop` (`src/capabilities/spotify.js:96-108`).
    let isi = if raw.is_array() || !(raw.get("status").is_some() || raw.get("result").is_some()) {
        raw
    } else {
        if !crate::common::js_truthy_value(raw.get("status").unwrap_or(&Value::Null)) {
            let alasan = raw
                .get("msg")
                .or_else(|| raw.get("error"))
                .map(teks_js)
                .unwrap_or_else(|| "tanpa alasan".to_string());
            return Err(HttpError {
                message: format!("spotify: aggregator menandai gagal — {alasan}"),
            });
        }
        match raw.get("result") {
            Some(v) if v.is_object() || v.is_array() => v.clone(),
            _ => {
                return Err(HttpError {
                    message: "spotify: aggregator menjawab tanpa result yang bisa dipakai".to_string(),
                });
            }
        }
    };

    // `normalize`: `Array.isArray(isi) ? isi : isi.tracks`.
    let daftar = if isi.is_array() {
        isi.as_array().cloned().unwrap_or_default()
    } else {
        isi.get("tracks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    Ok(daftar.iter().filter_map(track_cari).collect())
}

/// `trackCari(entri)` (`src/capabilities/spotify.js:110-129`).
fn track_cari(entri: &Value) -> Option<Track> {
    if !entri.is_object() {
        return None;
    }
    let judul = entri
        .get("title")
        .map(teks_js)
        .unwrap_or_default()
        .trim()
        .to_string();
    let url = entri
        .get("url")
        .map(teks_js)
        .unwrap_or_default()
        .trim()
        .to_string();
    if judul.is_empty() || url.is_empty() {
        return None;
    }
    let artist = entri
        .get("artist")
        .map(teks_js)
        .unwrap_or_default()
        .trim()
        .to_string();
    let cover = entri
        .get("thumbnail")
        .map(teks_js)
        .unwrap_or_default()
        .trim()
        .to_string();
    // `typeof entri.duration === "string" && entri.duration ? … : "0:00"`.
    let duration = match entri.get("duration") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => "0:00".to_string(),
    };
    Some(Track {
        title: judul,
        artist,
        url,
        cover,
        duration,
    })
}

/// `String(value)` JS untuk nilai JSON.
fn teks_js(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(a) => a.iter().map(teks_js).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// `URLSearchParams.set` meng-encode query sebagai
/// `application/x-www-form-urlencoded` — spasi jadi `+`.
fn encode_q(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                out.push(*b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `/^https?:\/\/open\.spotify\.com\/track\//i` (`spotify.js:62`).
pub(crate) fn tautan_track_spotify(url: &str) -> bool {
    let u = url.to_ascii_lowercase();
    u.starts_with("http://open.spotify.com/track/")
        || u.starts_with("https://open.spotify.com/track/")
}

/// `spotify.js:24` — kata kunci kosong.
pub(crate) fn tanpa_kunci() -> String {
    "❌ *Waduh, kata kuncinya mana nih?*\n\nKamu harus memasukkan judul lagu atau nama artis yang ingin dicari di Spotify. \n\nContoh penggunaan: `.spotify bruno mars`".to_string()
}

/// `spotify.js:35` — tidak ada hasil.
pub(crate) fn tidak_ditemukan(text: &str) -> String {
    format!("⚠️ *Maaf, lagu tidak ditemukan!* \n\nAku sudah mencari dengan kata kunci *{text}* tapi tidak ada hasil di Spotify. Coba gunakan judul yang lebih spesifik ya.")
}

/// `spotify.js:111` — API bermasalah.
pub(crate) fn api_bermasalah() -> String {
    "😔 *Aduh, sepertinya API sedang bermasalah.* \n\nTerjadi kesalahan fatal saat mencoba memproses pencarian Spotify. Silakan coba lagi nanti ya!".to_string()
}

/// Teks daftar hasil + catatan (`spotify.js:40-77`).
pub(crate) fn teks_hasil(text: &str, hasil: &[Track], pakai_tombol: bool) -> String {
    let mut content = format!("✨ *HASIL PENCARIAN SPOTIFY* ✨\n\nHalo! Aku berhasil menemukan beberapa lagu berdasarkan kata kunci *{text}*. Berikut adalah daftar teratasnya:\n\n");
    for (i, t) in hasil.iter().enumerate() {
        content.push_str(&format!("*{}. {}*\n", i + 1, t.title));
        content.push_str(&format!("   🎤 Artis: {}\n", t.artist));
        content.push_str(&format!("   ⏱️ Durasi: {}\n", t.duration));
        content.push_str(&format!("   🔗 Link: {}\n\n", t.url));
    }
    let first = &hasil[0];
    let bisa_unduh = tautan_track_spotify(&first.url);
    let mut catatan: Vec<&str> = vec!["*Catatan*:"];
    if bisa_unduh {
        catatan.push(
            "Kamu bisa menyalin link lagu di atas dan menggunakan perintah `.spdl <link>` untuk mengunduhnya secara langsung!",
        );
    }
    if pakai_tombol {
        catatan.push("Atau tekan tombol di bawah ini untuk lagu pertama.");
    }
    catatan.push("🚀");
    content.push_str(&catatan.join(" "));
    content
}

#[plugin(
    name = "spotify",
    alias = ["spotifysearch", "spsearch"],
    category = "search",
    description = "Mencari daftar lagu di Spotify berdasarkan judul atau artis",
    usage = ".spotify <query>",
    example = ".spotify neffex grateful",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn spotify(ctx: Ctx) -> Reply {
    let text = ctx.text.trim().to_string();
    if text.is_empty() {
        return Reply::text(tanpa_kunci());
    }

    let hasil = match cari_lagu(ctx.http.as_ref(), &text) {
        Ok(t) => t.into_iter().take(5).collect::<Vec<_>>(),
        Err(_) => {
            // `spotify.js:108-112` — reaksi ☢ lalu pesan API bermasalah.
            return Reply::text(api_bermasalah()).with_reaction("☢");
        }
    };

    if hasil.is_empty() {
        // `spotify.js:33-36` — reaksi ❌ lalu pesan tidak ditemukan.
        return Reply::text(tidak_ditemukan(&text)).with_reaction("❌");
    }

    let first = &hasil[0];
    // `spotify.js:49-56` — cover diambil lalu dikecilkan ke 300×170. Kalau
    // gagal, `thumbnailBuffer` tetap `null` dan tombol tidak dikirim.
    let thumbnail = if first.cover.is_empty() {
        None
    } else {
        ctx.http
            .get(&first.cover)
            .ok()
            .and_then(|res| oktz_plugin_api::gambar::sharp(res.body.bytes()).ok())
            .map(|g| g.ubah_ukuran(300, 170))
            .and_then(|g| g.ke_jpeg(80).ok())
    };

    let bisa_unduh = tautan_track_spotify(&first.url);
    let pakai_tombol = bisa_unduh && thumbnail.is_some();
    let content = teks_hasil(&text, &hasil, pakai_tombol);

    if pakai_tombol {
        // `spotify.js:79-102` — kartu `buttonsMessage` + tombol unduh.
        let header = CardHeader::new().location(CardLocation {
            latitude: 0.0,
            longitude: 0.0,
            name: first.title.clone(),
            address: Some(format!("🎤 {} | ⏱️ {}", first.artist, first.duration)),
            thumbnail: thumbnail.clone(),
        });
        return Reply::interactive(content)
            .with_footer("🚀 Spotify Search")
            .with_header(header)
            .with_button(InteractiveButton::quick_reply(
                "🎵 Unduh Lagu Pertama",
                format!(".spdl {}", first.url),
            ))
            .with_context(CardContext::new().quote_trigger())
            .with_reaction("✅");
    }

    Reply::text(content).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpResponse};
    use std::sync::{Arc, Mutex};

    /// Stub HTTP: membalas body yang sudah disiapkan, atau gagal total.
    #[derive(Debug)]
    struct Stub {
        body: Option<String>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn body(v: Value) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(v.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn raw(text: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(text.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                body: None,
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let text = self
                .body
                .clone()
                .ok_or_else(|| HttpError {
                    message: "offline".into(),
                })?;
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
            unimplemented!("cuma GET")
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
            command: "spotify".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("spotify").unwrap().handler)(ctx)
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

    fn respons(track: Value) -> Value {
        serde_json::json!({ "status": true, "result": [track] })
    }

    const LAGU: &str = r#"{"title":"Grateful","artist":"Neffex","url":"https://open.spotify.com/track/abc","thumbnail":"https://i.scdn.co/cover.jpg","duration":"3:12"}"#;

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("spotify").unwrap();
        assert_eq!(p.meta.name, "spotify");
        assert_eq!(p.meta.alias, ["spotifysearch", "spsearch"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(
            p.meta.description,
            "Mencari daftar lagu di Spotify berdasarkan judul atau artis"
        );
        assert_eq!(p.meta.usage, ".spotify <query>");
        assert_eq!(p.meta.example, ".spotify neffex grateful");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `spotify.js:24` — kata kunci kosong.
        assert_eq!(
            text_of(&run(Stub::offline(), "")),
            "❌ *Waduh, kata kuncinya mana nih?*\n\nKamu harus memasukkan judul lagu atau nama artis yang ingin dicari di Spotify. \n\nContoh penggunaan: `.spotify bruno mars`"
        );
        // `spotify.js:35` — tidak ada hasil.
        let r = run(Stub::body(serde_json::json!({ "status": true, "result": [] })), "x");
        assert_eq!(
            text_of(&r),
            "⚠️ *Maaf, lagu tidak ditemukan!* \n\nAku sudah mencari dengan kata kunci *x* tapi tidak ada hasil di Spotify. Coba gunakan judul yang lebih spesifik ya."
        );
        assert_eq!(reaction_of(&r), Some("❌"));
        // `spotify.js:111` — API bermasalah.
        let r = run(Stub::offline(), "x");
        assert_eq!(
            text_of(&r),
            "😔 *Aduh, sepertinya API sedang bermasalah.* \n\nTerjadi kesalahan fatal saat mencoba memproses pencarian Spotify. Silakan coba lagi nanti ya!"
        );
        assert_eq!(reaction_of(&r), Some("☢"));
    }

    #[test]
    fn daftar_lagu_tertampilkan_seperti_js() {
        let http = Stub::body(respons(serde_json::json!({
            "title": "Grateful",
            "artist": "Neffex",
            "url": "https://open.spotify.com/track/abc",
            "thumbnail": "",
            "duration": "3:12"
        })));
        let r = run(http.clone(), "neffex grateful");
        let t = text_of(&r);
        assert!(
            t.starts_with("✨ *HASIL PENCARIAN SPOTIFY* ✨\n\nHalo! Aku berhasil menemukan beberapa lagu berdasarkan kata kunci *neffex grateful*. Berikut adalah daftar teratasnya:\n\n"),
            "{t}"
        );
        assert!(t.contains("*1. Grateful*\n   🎤 Artis: Neffex\n   ⏱️ Durasi: 3:12\n   🔗 Link: https://open.spotify.com/track/abc\n\n"), "{t}");
        assert!(
            t.ends_with("*Catatan*: Kamu bisa menyalin link lagu di atas dan menggunakan perintah `.spdl <link>` untuk mengunduhnya secara langsung! 🚀"),
            "{t}"
        );
        assert_eq!(reaction_of(&r), Some("✅"));
        assert_eq!(
            http.seen.lock().unwrap()[0],
            "https://api.nexray.eu.cc/search/spotify?q=neffex+grateful"
        );
    }

    #[test]
    fn track_tanpa_judul_atau_tautan_dibuang() {
        let http = Stub::body(serde_json::json!({
            "status": true,
            "result": [
                {"title": "", "url": "https://open.spotify.com/track/a"},
                {"title": "Ada", "url": ""},
                {"title": "Ada", "url": "https://open.spotify.com/track/c"}
            ]
        }));
        let tracks = cari_lagu(http.as_ref(), "q").unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Ada");
    }

    #[test]
    fn duration_bukan_string_jadi_nol() {
        let http = Stub::body(respons(serde_json::json!({
            "title": "Lagu", "artist": "A", "url": "https://open.spotify.com/track/x", "duration": 200
        })));
        let tracks = cari_lagu(http.as_ref(), "q").unwrap();
        assert_eq!(tracks[0].duration, "0:00");
    }

    #[test]
    fn amplop_gagal_melempar_pesan_agregator() {
        let http = Stub::body(serde_json::json!({ "status": false, "msg": "kuota habis" }));
        let err = cari_lagu(http.as_ref(), "q").unwrap_err();
        assert_eq!(
            err.message,
            "spotify: aggregator menandai gagal — kuota habis"
        );
    }

    #[test]
    fn tautan_track_spotify_hanya_yang_track() {
        assert!(tautan_track_spotify(
            "https://open.spotify.com/track/abc"
        ));
        assert!(tautan_track_spotify(
            "HTTPS://OPEN.SPOTIFY.COM/TRACK/abc"
        ));
        assert!(!tautan_track_spotify(
            "https://open.spotify.com/album/abc"
        ));
        assert!(!tautan_track_spotify("https://spotify.com/track/abc"));
    }
}
