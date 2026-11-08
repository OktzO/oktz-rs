//! `plugins/search/pin.js` — cari gambar Pinterest dan kirimnya sebagai album.
//!
//! Resolver `pinterest` JS (`src/capabilities/pinterest.js`) untuk query
//! search hanya punya satu backend yang berlaku: `nexrayWeb`
//! `GET /api/search/pinterest?q=…` (`ilovepin` hanya berlaku untuk argumen
//! `url`). Dipindah sebagai HTTP langsung di [`resolve_pinterest`],
//! mengikuti pola agregator di `facebookdl.rs` / `applemusic.rs`.
//! Normalisasi mengikuti `normalize` JS: amplop `{status, result}` — `status`
//! bukan `true` melempar (→ cabang `catch`); `result` berupa array atau
//! `{pins: [...]}` dipetakan lewat `pinCari` (`image || images_url` yang
//! diawali `https?://`).
//!
//! `ponytail:` album `albumMessage` + `messageContextInfo.messageSecret` +
//! `sock.relayMessage` + `sock.waUploadToServer` tidak punya padanan di
//! `Reply`, jadi yang dikirim adalah fallback JS-nya sendiri (kirim
//! satu-satu, `pin.js:114-125`) lewat `Reply::sequence` berisi `Reply::image`
//! ber-URL. Tiap gambar tetap diunduh dulu dengan header `Referer` dan
//! ambang 1000 byte (`pin.js:54-69`), persis JS: yang lolos ambanglah yang
//! masuk daftar kirim.

use oktz_plugin_api::{Ctx, HttpCall, HttpGet, MediaSource, Reply, plugin};

/// `src/capabilities/pinterest.js:47-78` — backend `nexray`
/// (`aggregator.hit("nexrayWeb", "/api/search/pinterest", …)`; base di
/// `src/lib/aggregator.js:30`).
pub(crate) const ENDPOINT: &str = "https://api.nexray.web.id/api/search/pinterest";


/// `pin.js:30-35` — kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🔍 *ᴘɪɴᴛᴇʀᴇsᴛ sᴇᴀʀᴄʜ*\n\n\
         > Contoh:\n\
         `{prefix}pin Zhao Lusi`"
    )
}

/// `pin.js:49` — resolver menjawab tapi tidak ada pin.
pub(crate) fn tidak_ditemukan(query: &str) -> String {
    format!("❌ Tidak ditemukan hasil untuk: {query}")
}

/// `pin.js:73` — ada pin tapi tidak satu pun buffer yang lolos unduhan.
pub(crate) const GAGAL_MEMUAT: &str = "❌ Gagal memuat gambar";

/// `pin.js:63` — ambang ukuran; di bawah itu buffer diabaikan diam-diam.
pub(crate) const MIN_BYTES: usize = 1000;

/// `axios.get(imageUrl, { timeout: 15000, headers: { Referer } })` (`pin.js:56-59`).
pub(crate) fn download_request(url: &str) -> HttpCall {
    HttpCall::get(url)
        .header("Referer", "https://www.pinterest.com/")
        .with_timeout(15_000)
}

/// `data?.pins ?? []` → `pin?.image` → `filter(Boolean)` → `slice(0, 10)`
/// (`pin.js:43-46`).
pub(crate) fn pick_images(pins: &[serde_json::Value]) -> Vec<String> {
    pins.iter()
        .filter_map(|p| p.get("image"))
        .filter_map(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .take(10)
        .map(str::to_string)
        .collect()
}

/// `pinCari` (`src/capabilities/pinterest.js:140-153`) — `image || images_url`
/// yang diawali `https?://`; yang lain dibuang menjadi `None`.
fn pin_cari(entri: &serde_json::Value) -> Option<serde_json::Value> {
    let teks = |v: Option<&serde_json::Value>| {
        v.and_then(serde_json::Value::as_str).unwrap_or("").trim()
    };
    let image = teks(entri.get("image"));
    let image = if image.is_empty() {
        teks(entri.get("images_url"))
    } else {
        image
    };
    if !(image.starts_with("http://") || image.starts_with("https://")) {
        return None;
    }
    Some(serde_json::json!({ "image": image }))
}

/// `normalize` cabang pencarian (`pinterest.js:173-184`) + `isiAmplop`
/// (`:160-171`): `status: false` melempar; `result` array atau
/// `result.pins` array dipetakan lewat [`pin_cari`]. Bentuk lain (unduhan)
/// tidak punya `pins`, jadi `Err` — persis `normalize` yang melempar untuk
/// bentuk tak dikenal.
pub(crate) fn normalize(body: &serde_json::Value) -> Result<Vec<serde_json::Value>, ()> {
    if body.get("status").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(());
    }
    let result = body.get("result").filter(|r| !r.is_null()).ok_or(())?;
    let daftar = if let Some(a) = result.as_array() {
        a.clone()
    } else if let Some(a) = result.get("pins").and_then(serde_json::Value::as_array) {
        a.clone()
    } else {
        return Err(());
    };
    Ok(daftar.iter().filter_map(pin_cari).collect())
}

/// `resolver.resolve("pinterest", { q })` lewat backend `nexray`
/// (`pinterest.js:47-78`): `GET {ENDPOINT}?q=…`. `Err` = melempar, sama
/// seperti `normalize` yang melempar sebelum plugin menyentuh hasilnya.
/// Param di-encode seperti `axios` (`URLSearchParams`): spasi jadi `+`,
/// byte non-ASCII jadi `%XX` uppercase.
pub(crate) fn resolve_pinterest(
    query: &str,
    http: &dyn HttpGet,
) -> Result<Vec<serde_json::Value>, ()> {
    let target = format!("{ENDPOINT}?q={}", encode(query));
    let body = http
        .get(&target)
        .ok()
        .and_then(|r| r.json())
        .ok_or(())?;
    normalize(&body)
}

/// `URLSearchParams.set` — sama polanya dengan `search::applemusic::encode`.
fn encode(raw: &str) -> String {
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

#[plugin(
    name = "pin",
    alias = ["pinsearch", "pinterestsearch", "pins"],
    category = "search",
    description = "Cari gambar di Pinterest (album)",
    usage = ".pin <query>",
    example = ".pin Zhao Lusi",
    cooldown = 10,
    energi = 1,
    is_enabled = true,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn pin(ctx: Ctx) -> Reply {
    let query = ctx.text.trim();

    if query.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    // `await m.react("🕕")` di `:36` sudah jalan sebelum resolver.
    match resolve_pinterest(query, ctx.http.as_ref()) {
        Ok(pins) => {
            let results = pick_images(&pins);
            if results.is_empty() {
                return Reply::text(tidak_ditemukan(query)).with_reaction("❌");
            }
            // `pin.js:54-69` — tiap URL diunduh lewat `download_request`
            // (header `Referer`, timeout 15 s); yang gagal atau `<= 1000`
            // byte dilewati satu per satu, persis `try/catch { continue }`.
            let mut media: Vec<String> = Vec::new();
            for url in &results {
                let Ok(resp) = ctx.http.call(&download_request(url)) else {
                    continue;
                };
                if resp.body.bytes().len() > MIN_BYTES {
                    media.push(url.clone());
                }
            }
            if media.is_empty() {
                // `pin.js:71-74` — ada pin tapi tidak satu pun lolos unduhan.
                return Reply::text(GAGAL_MEMUAT).with_reaction("❌");
            }
            // `pin.js:114-125` fallback album: kirim satu-satu. `Reply::image`
            // ber-URL (`MediaSource::Url`) — pipeline yang mengunduh byte saat
            // kirim, sama seperti `sendMessage({ image: buffer })` di JS.
            let kirim: Vec<Reply> = media
                .iter()
                .map(|u| Reply::image(MediaSource::url(u.clone()), ""))
                .collect();
            Reply::sequence(kirim).with_reaction("✅")
        }
        // `pin.js:127-131` — `m.pushName` tidak ada di `Ctx`, jadi "User".
        Err(()) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpMethod, HttpResponse, HttpGet};
    use std::sync::{Arc, Mutex};

    /// Stub `HttpGet`: pencarian mengembalikan `cari`, gambar `gambar_ok`
    /// (byte besar) atau gagal. `call` di-override supaya header hasil
    /// `download_request` (Referer) ikut tercatat — default `call` membuang
    /// header (`registry.rs`).
    #[derive(Debug)]
    struct Stub {
        cari: Option<String>,
        gambar_ok: bool,
        seen: Mutex<Vec<(String, Vec<(String, String)>)>>,
    }

    impl Stub {
        fn new(v: serde_json::Value) -> Arc<Stub> {
            Arc::new(Stub {
                cari: Some(v.to_string()),
                gambar_ok: true,
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                cari: None,
                gambar_ok: false,
                seen: Mutex::new(Vec::new()),
            })
        }
        fn seen_urls(&self) -> Vec<String> {
            self.seen
                .lock()
                .unwrap()
                .iter()
                .map(|(u, _)| u.clone())
                .collect()
        }
    }

    fn resp(body: &str) -> HttpResponse {
        HttpResponse {
            status: 200,
            body: HttpBody::Bytes {
                data: Arc::from(body.to_string().into_bytes()),
                mimetype: Some("application/octet-stream".into()),
            },
            headers: vec![],
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen
                .lock()
                .unwrap()
                .push((url.to_string(), vec![]));
            if url.starts_with(ENDPOINT) {
                return self
                    .cari
                    .clone()
                    .map(|b| resp(&b))
                    .ok_or_else(|| HttpError {
                        message: "offline".into(),
                    });
            }
            if self.gambar_ok {
                Ok(resp(&"x".repeat(2000)))
            } else {
                Err(HttpError {
                    message: "offline".into(),
                })
            }
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen
                .lock()
                .unwrap()
                .push((req.url.clone(), req.headers.clone()));
            if req.url.starts_with(ENDPOINT) {
                return self.get(&req.url.clone());
            }
            if self.gambar_ok {
                Ok(resp(&"x".repeat(2000)))
            } else {
                Err(HttpError {
                    message: "offline".into(),
                })
            }
        }
        fn post_json(&self, _: &str, _: &serde_json::Value) -> Result<HttpResponse, HttpError> {
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

    fn run(text: &str) -> Reply {
        run_with(Stub::offline(), text)
    }

    fn run_with(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "pin".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("pin").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        let mut cur = r;
        loop {
            match cur {
                Reply::Reaction { emoji, .. } => return Some(emoji),
                Reply::Mutate { reply, .. } => cur = reply,
                _ => return None,
            }
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("pin").unwrap();
        assert_eq!(p.meta.name, "pin");
        assert_eq!(p.meta.alias, ["pinsearch", "pinterestsearch", "pins"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Cari gambar di Pinterest (album)");
        assert_eq!(p.meta.usage, ".pin <query>");
        assert_eq!(p.meta.example, ".pin Zhao Lusi");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["pinsearch", "pinterestsearch", "pins"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "pin");
        }
    }

    #[test]
    fn an_empty_query_shows_the_help_card() {
        for text in ["", "  ", "\t\n"] {
            assert_eq!(
                text_of(&run(text)),
                "🔍 *ᴘɪɴᴛᴇʀᴇsᴛ sᴇᴀʀᴄʜ*\n\n> Contoh:\n`.pin Zhao Lusi`",
                "input {text:?}"
            );
        }
        assert!(help("!").contains("`!pin Zhao Lusi`"));
        assert_eq!(reaction_of(&run("")), None);
    }

    #[test]
    fn a_query_with_a_dead_backend_hits_the_error_branch() {
        let r = run("Zhao Lusi");
        assert_eq!(text_of(&r), crate::common::error_template_bawaan(".", "pin"));
        assert_eq!(reaction_of(&r), Some("☢"));
    }

    #[test]
    fn a_search_hits_the_nexray_endpoint_and_sends_the_downloaded_images() {
        let http = Stub::new(serde_json::json!({
            "status": true,
            "result": [
                { "image": "https://i.pinimg.com/a.jpg" },
                { "image": "", "images_url": "https://i.pinimg.com/b.jpg" },
                { "image": "ftp://tidak/http" },
            ]
        }));
        let r = run_with(http.clone(), "Zhao Lusi");
        let seen = http.seen_urls();
        assert_eq!(
            seen[0],
            "https://api.nexray.web.id/api/search/pinterest?q=Zhao+Lusi"
        );
        assert_eq!(seen.len(), 3, "1 cari + 2 gambar: {seen:?}");
        // Header Referer ikut terkirim pada unduhan gambar.
        let headers = http.seen.lock().unwrap()[1].1.clone();
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == "Referer" && v == "https://www.pinterest.com/")
        );
        // Fallback album JS: gambar dikirim satu-satu (`pin.js:118-124`).
        let isi = r.without_mutations();
        let inner = match isi {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        };
        match inner {
            Reply::Sequence(list) => {
                assert_eq!(list.len(), 2, "anggota non-https terbuang: {list:?}");
                for (img, reply) in [
                    ("https://i.pinimg.com/a.jpg", &list[0]),
                    ("https://i.pinimg.com/b.jpg", &list[1]),
                ] {
                    match reply {
                        Reply::Image { media, .. } => {
                            assert!(matches!(
                                media.as_ref().unwrap(),
                                MediaSource::Url { url, .. } if url == img
                            ));
                        }
                        lain => panic!("harus image, dapat {lain:?}"),
                    }
                }
            }
            lain => panic!("harus sequence, dapat {lain:?}"),
        }
        assert_eq!(reaction_of(&r), Some("✅"));
    }

    #[test]
    fn an_empty_result_reports_not_found_and_a_dead_download_reports_failure() {
        let body = serde_json::json!({ "status": true, "result": [] });
        let r = run_with(Stub::new(body), "x");
        assert_eq!(text_of(&r), "❌ Tidak ditemukan hasil untuk: x");
        assert_eq!(reaction_of(&r), Some("❌"));

        // Ada pin, unduhan mati semua → `GAGAL_MEMUAT` (`pin.js:71-74`).
        let http = Arc::new(Stub {
            cari: Some(
                serde_json::json!({
                    "status": true,
                    "result": [{ "image": "https://i.pinimg.com/a.jpg" }]
                })
                .to_string(),
            ),
            gambar_ok: false,
            seen: Mutex::new(Vec::new()),
        });
        let r = run_with(http, "x");
        assert_eq!(text_of(&r), GAGAL_MEMUAT);
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn a_failed_envelope_throws_like_normalize_does() {
        for body in [
            serde_json::json!({ "status": false, "msg": "gagal" }),
            serde_json::json!({ "status": true }),
            serde_json::json!({ "status": true, "result": null }),
            serde_json::json!({ "status": true, "result": { "type": "image" } }),
        ] {
            let r = run_with(Stub::new(body.clone()), "x");
            assert_eq!(
                text_of(&r),
                crate::common::error_template_bawaan(".", "pin"),
                "{body}"
            );
            assert_eq!(reaction_of(&r), Some("☢"), "{body}");
        }
    }

    #[test]
    fn the_pinterest_request_carries_the_referer_header() {
        let call = download_request("https://i.pinimg.com/originals/x.jpg");
        assert_eq!(call.method, HttpMethod::Get);
        assert_eq!(call.timeout_ms, Some(15_000));
        assert!(call
            .headers
            .iter()
            .any(|(k, v)| k == "Referer" && v == "https://www.pinterest.com/"));
    }

    #[test]
    fn odd_and_very_long_queries_do_not_panic() {
        for input in ["<b>", "a|b", "🎨", "'\"", "a\u{202e}b", &"x".repeat(5000)] {
            let r = run(input);
            assert!(!text_of(&r).is_empty(), "input {input:?}");
        }
    }

    #[test]
    fn pin_urls_are_collected_dropping_blanks_and_capped_at_ten() {
        let pins: Vec<serde_json::Value> = (0..13)
            .map(|i| serde_json::json!({ "image": format!("https://i.pinimg.com/{i}.jpg") }))
            .chain([serde_json::json!({ "image": "" }), serde_json::json!({}), serde_json::json!({ "image": null })])
            .collect();
        let urls = pick_images(&pins);
        assert_eq!(urls.len(), 10, "slice(0, 10) sebelum filter kedua");
        assert_eq!(urls[0], "https://i.pinimg.com/0.jpg");
        assert_eq!(urls[9], "https://i.pinimg.com/9.jpg");
    }

    #[test]
    fn an_empty_or_scalar_pin_list_yields_no_urls() {
        assert!(pick_images(&[]).is_empty());
        assert!(pick_images(&[serde_json::json!({})]).is_empty());
        assert!(pick_images(&[serde_json::json!({ "image": 42 })]).is_empty());
    }

    #[test]
    fn the_not_found_and_load_failure_messages_are_kept() {
        assert_eq!(
            tidak_ditemukan("Zhao Lusi"),
            "❌ Tidak ditemukan hasil untuk: Zhao Lusi"
        );
        assert_eq!(GAGAL_MEMUAT, "❌ Gagal memuat gambar");
    }

}