//! `plugins/search/play.js` — putar musik dari YouTube.
//!
//! Jalur utama: sistem baru `play.js` (akar repo) lewat
//! [`oktz_scraper::playytmp3::ytmp3`] — pencarian halaman `results`
//! (`ytInitialData`) + backend savetube → ytmp3.ing.
//!
//! ponytail: fallback = jalur ourin (`play.js` produksi): `yt-search` +
//! nexray ([`crate::download::ytmp3::unduh_audio_nexray`]), dipakai hanya
//! bila sistem baru gagal semua.

#![allow(clippy::too_many_arguments)]

use oktz_plugin_api::{Ctx, MediaSource as Ms, Reply, plugin};

use crate::download::ytmp3::{AudioDownload, mimetype_for, unduh_audio_nexray};

use super::ytsearch;

/// `play.js:26` — kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!("🎵 *ᴘʟᴀʏ*\n\n> Contoh:\n`{prefix}play komang`")
}

/// `play.js:91` — pesan kegagalan, tanpa template `te()`.
pub(crate) const GAGAL: &str =
    "Wahhh, fitur putar musiknya lagi ada kendala kak, coba lagi nanti yak, jangan spam";

/// `play.js:16-21` — `formatViews`.
///
/// `n` datang dari `yt-search` sebagai angka; `!n` menangkap `0`, `NaN`, dan
/// `undefined`, semuanya jadi `"0"`.
pub(crate) fn format_views(n: f64) -> String {
    if !n.is_finite() || n == 0.0 {
        return "0".to_string();
    }
    if n >= 1e6 {
        return format!("{}M", satu_desimal(n / 1e6));
    }
    if n >= 1e3 {
        return format!("{}K", satu_desimal(n / 1e3));
    }
    format!("{n}")
}

/// `Number.prototype.toFixed(1)` — pembulatan ke atas untuk tepat `.5`.
fn satu_desimal(n: f64) -> String {
    let kali_seri = n * 10.0;
    let bawah = kali_seri.floor();
    let naik = if kali_seri - bawah >= 0.5 { 1.0 } else { 0.0 };
    format!("{:.1}", (bawah + naik) / 10.0)
}

/// `play.js:42-55` — kartu `NOW PLAYING`. `desc` = deskripsi video.
pub(crate) fn now_playing(
    title: &str,
    author: &str,
    timestamp: &str,
    views: f64,
    ago: &str,
    video_id: &str,
    description: &str,
    url: &str,
) -> String {
    let mut info = String::from("🎵 *NOW PLAYING*\n\n");
    info.push_str(&format!("📌 *Judul:* {title}\n\n"));
    info.push_str("*DETAIL*\n");
    info.push_str(&format!("👤 Channel: *{author}*\n"));
    info.push_str(&format!("⏱️ Durasi: *{timestamp}*\n"));
    info.push_str(&format!("👀 Views: *{}*\n", format_views(views)));
    info.push_str(&format!("📅 Upload: *{ago}*\n"));
    info.push_str(&format!("🆔 ID: `{video_id}`\n\n"));
    // `if (video.description)` — string kosong falsy, bloknya dilewati.
    if !description.is_empty() {
        let chars: Vec<char> = description.chars().collect();
        let desc: String = chars.iter().take(150).collect();
        let desc = desc.replace('\n', " ");
        info.push_str(&format!(
            "*Deskripsi:*\n_{desc}{}_\n\n",
            if chars.len() > 150 { "..." } else { "" }
        ));
    }
    info.push_str(&format!("🔗 {url}\n\n"));
    info.push_str("_⏳ mengirim audio, harap tunggu..._");
    info
}

/// Jalur utama: sistem baru `play.js` (`oktz_scraper::playytmp3`) —
/// pencarian `results` + savetube → ytmp3ing. Gagal semua → fallback
/// ourin: `yt-search` + nexray.
///
/// Kartu balik `(info, url, title, thumbnail)` — `thumbnail` untuk
/// `Reply::preview` (`play.js:57-66`, `image: video.thumbnail`).
fn cari_dan_unduh(
    ctx: &Ctx,
    query: &str,
) -> Result<((String, String, String, String), AudioDownload), ()> {
    if let Ok(hasil) = oktz_scraper::playytmp3::ytmp3(ctx.http.as_ref(), query, "128") {
        let video = &hasil.video;
        let info = now_playing(
            &video.title,
            &video.author,
            &video.duration,
            0.0,
            &video.published,
            &video.id,
            "",
            &video.url,
        );
        let kartu = (info, video.url.clone(), video.title.clone(), video.thumbnail.clone());
        let audio = AudioDownload {
            title: video.title.clone(),
            format: "mp3".to_string(),
            url: hasil
                .download
                .get("downloadUrl")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string(),
        };
        if !audio.url.is_empty() {
            return Ok((kartu, audio));
        }
    }
    // --- Fallback ourin (`play.js` produksi): yt-search + nexray -------
    let video = ytsearch::search(ctx.http.as_ref(), query)
        .map_err(|_| ())?
        .into_iter()
        .next()
        .ok_or(())?;
    let audio = unduh_audio_nexray(ctx, &video.url)?;
    let info = now_playing(
        &video.title,
        &video.author_name,
        &video.timestamp,
        video.views as f64,
        &video.ago,
        &video.video_id,
        &video.description,
        &video.url,
    );
    Ok(((info, video.url, video.title, video.thumbnail), audio))
}

#[plugin(
    name = "play",
    alias = ["playaudio"],
    category = "search",
    description = "Putar musik dari YouTube",
    usage = ".play <query>",
    example = ".play komang",
    cooldown = 15,
    energi = 1,
    is_enabled = true,
    is_group = false,
    is_private = false,
    // Tidak ada di pluginConfig JS.
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn play(ctx: Ctx) -> Reply {
    let query = ctx.text.trim();

    if query.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    match cari_dan_unduh(&ctx, query) {
        // `play.js:42-86` — kartu NOW PLAYING (preview-link ber-thumbnail,
        // `play.js:57-70`), lalu audio, lalu ✅. Reaksi tidak boleh membungkus
        // `Reply::Sequence`, jadi 🕐/✅ jadi elemen urutan (pola `sticker.rs`).
        Ok((kartu, audio)) => Reply::sequence(vec![
            Reply::reaction("🕐"),
            // `socket.js:1216` — `if (preview.image)`: thumbnail kosong/falsy
            // = kartu tanpa gambar.
            Reply::preview(
                kartu.0,
                kartu.1,
                kartu.2,
                "YouTube Video",
                (!kartu.3.is_empty()).then(|| Ms::url(kartu.3)),
            ),
            Reply::audio(Ms::url(audio.url).with_mimetype(mimetype_for(&audio.format))),
            Reply::reaction("✅"),
        ]),
        // `play.js:87-93` — 😭 lalu pesan kegagalan.
        Err(()) => Reply::text(GAGAL).with_reaction("😭"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub HTTP multi-URL: POST pencarian YouTube (lewat `call`), rantai
    /// backend ytmp3 (`ymcdn` → `izuka` → `nexray`), dicocokkan dari potongan
    /// URL. `None` = transport error.
    #[derive(Debug)]
    struct Stub {
        per_url: Vec<(&'static str, Option<Value>)>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn per_url(per_url: Vec<(&'static str, Option<Value>)>) -> Arc<Stub> {
            Arc::new(Stub {
                per_url,
                seen: Mutex::new(Vec::new()),
            })
        }
        fn jawab(&self, url: &str) -> Result<HttpResponse, HttpError> {
            let (_, nilai) = self
                .per_url
                .iter()
                .find(|(penanda, _)| url.contains(penanda))
                .ok_or(HttpError {
                    message: format!("URL tak dikenal stub: {url}"),
                })?;
            let text = nilai
                .clone()
                .ok_or(HttpError { message: "offline".into() })?
                .to_string();
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

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            self.jawab(url)
        }
        fn post_json(&self, url: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            self.jawab(url)
        }
        fn post_form(&self, url: &str, _: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            self.jawab(url)
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET/POST JSON")
        }
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            self.jawab(&req.url)
        }
    }

    fn run_http(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "play".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("play").unwrap().handler)(ctx)
    }

    fn run(text: &str) -> Reply {
        run_http(Stub::per_url(vec![("", None)]), text)
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

    /// Hasil `yt-search` dengan satu video.
    const HASIL_VIDEO: &str = r#"{"contents":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[
      {"itemSectionRenderer":{"contents":[
        {"videoRenderer":{
          "videoId":"aaaaaaaaaaa",
          "title":{"runs":[{"text":"Lagu Uji"}]},
          "ownerText":{"runs":[{"text":"Channel Uji"}]},
          "viewCountText":{"simpleText":"1,234,567 views"},
          "publishedTimeText":{"simpleText":"2 years ago"},
          "lengthText":{"simpleText":"3:33"},
          "thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/aaaaaaaaaaa/hq.jpg"}]},
          "detailedMetadataSnippets":[{"snippetText":{"runs":[{"text":"Deskripsi video."}]}}]
        }}
      ]}}
    ]}}}}]}}"#;

    /// `play.js:31-40` — `yt-search` lalu nexray.
    fn stub_sukses() -> Arc<Stub> {
        Stub::per_url(vec![
            (
                "youtubei/v1/search",
                Some(serde_json::from_str(HASIL_VIDEO).unwrap()),
            ),
            (
                "nexray",
                Some(serde_json::json!({
                    "status": true,
                    "result": {
                        "title": "Lagu Uji",
                        "format": "mp3",
                        "url": "https://cdn.example/lagu.mp3"
                    }
                })),
            ),
        ])
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("play").unwrap();
        assert_eq!(p.meta.name, "play");
        assert_eq!(p.meta.alias, ["playaudio"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Putar musik dari YouTube");
        assert_eq!(p.meta.usage, ".play <query>");
        assert_eq!(p.meta.example, ".play komang");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
        assert!(p.meta.is_enabled);
        // Tidak ada di pluginConfig JS.
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        assert_eq!(
            oktz_plugin_api::find("playaudio").unwrap().meta.name,
            "play"
        );
    }

    #[test]
    fn an_empty_query_shows_the_help_card() {
        for text in ["", "  ", "\t\n"] {
            assert_eq!(
                text_of(&run(text)),
                "🎵 *ᴘʟᴀʏ*\n\n> Contoh:\n`.play komang`",
                "input {text:?}"
            );
        }
        assert!(help("!").contains("`!play komang`"));
        assert_eq!(reaction_of(&run("")), None);
    }

    #[test]
    fn a_successful_search_sends_the_now_playing_card_then_the_audio() {
        let http = stub_sukses();
        let r = run_http(http.clone(), "komang");
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus sequence, dapat {:?}", r.without_mutations());
        };
        assert_eq!(list.len(), 4);
        assert!(matches!(&list[0], Reply::Reaction { emoji, .. } if emoji == "🕐"));
        // `play.js:57-70` — kartu preview-link: caption/info, URL video,
        // judul, deskripsi "YouTube Video", thumbnail `video.thumbnail`.
        let (info, url, judul) = match &list[1] {
            Reply::Preview { text, url, title, description, preview_type, thumbnail } => {
                assert_eq!(description, "YouTube Video");
                assert_eq!(*preview_type, 1, "previewType: 1 (VIDEO)");
                assert!(thumbnail.is_some(), "image: video.thumbnail");
                (text.as_str(), url.clone(), title.clone())
            }
            lain => panic!("harus kartu preview, dapat {lain:?}"),
        };
        assert!(info.starts_with("🎵 *NOW PLAYING*\n\n📌 *Judul:* Lagu Uji\n"), "{info}");
        assert!(info.contains("👤 Channel: *Channel Uji*\n"), "{info}");
        assert!(info.contains("⏱️ Durasi: *3:33*\n"), "{info}");
        assert!(info.ends_with("_⏳ mengirim audio, harap tunggu..._"), "{info}");
        assert_eq!(url, "https://www.youtube.com/watch?v=aaaaaaaaaaa");
        assert_eq!(judul, "Lagu Uji");
        match &list[2] {
            Reply::Audio { media, ptt } => {
                assert!(!ptt, "JS kirim ptt: false");
                let m = media.as_ref().expect("audio harus punya sumber");
                assert_eq!(m.mimetype(), Some("audio/mpeg"));
            }
            lain => panic!("harus audio, dapat {lain:?}"),
        }
        assert!(matches!(&list[3], Reply::Reaction { emoji, .. } if emoji == "✅"));

        // Pencarian `yt-search` (video pertama) lalu nexray — tidak ada
        // jalur YT Music.
        let seen = http.seen.lock().unwrap().clone();
        assert!(
            seen.iter()
                .any(|u| u.contains("www.youtube.com/youtubei/v1/search")),
            "pencarian yt-search: {seen:?}"
        );
        assert!(
            !seen.iter().any(|u| u.contains("music.youtube.com")),
            "tidak boleh menyentuh YT Music: {seen:?}"
        );
        assert!(
            seen.iter().any(|u| u.contains(
                "api.nexray.web.id/api/downloader/ytmp3?url=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3Daaaaaaaaaaa"
            )),
            "nexray menerima URL kanonik: {seen:?}"
        );
    }

    #[test]
    fn a_search_without_results_falls_into_the_catch_branch() {
        // `throw new Error("Video tidak ditemukan")` → catch yang sama.
        let http = Stub::per_url(vec![("youtubei/v1/search", Some(serde_json::json!({"contents": {}})))]);
        let r = run_http(http, "zzzz tidak ada");
        assert_eq!(text_of(&r), GAGAL);
        assert_eq!(reaction_of(&r), Some("😭"));
    }

    #[test]
    fn a_backend_failure_falls_into_the_catch_branch() {
        // nexray satu-satunya backend `.play`; status false → catch JS.
        let http = Stub::per_url(vec![
            (
                "youtubei/v1/search",
                Some(serde_json::from_str(HASIL_VIDEO).unwrap()),
            ),
            ("nexray", Some(serde_json::json!({"status": false}))),
        ]);
        let r = run_http(http, "komang");
        assert_eq!(text_of(&r), GAGAL);
        assert_eq!(reaction_of(&r), Some("😭"));
    }

    #[test]
    fn query_url_youtube_tetap_lewat_search_dan_nexray() {
        // `yts(url)` juga mengembalikan video (`play.js:31`); jalur sama.
        let http = stub_sukses();
        let r = run_http(http.clone(), "https://youtu.be/dQw4w9WgXcQ");
        let Reply::Sequence(_) = r.without_mutations() else {
            panic!("harus sequence, dapat {r:?}");
        };
        assert!(
            http.seen
                .lock()
                .unwrap()
                .iter()
                .any(|u| u.contains("api.nexray.web.id")),
            "URL tetap diunduh lewat nexray"
        );
    }

    #[test]
    fn odd_and_very_long_queries_do_not_panic() {
        for input in ["<b>", "a|b", "🎵", "'\"", "a\u{202e}b", &"x".repeat(5000)] {
            assert!(!text_of(&run(input)).is_empty(), "input {input:?}");
        }
    }

    #[test]
    fn view_counts_switch_to_k_and_m_at_the_right_threshold() {
        assert_eq!(format_views(0.0), "0");
        assert_eq!(format_views(999.0), "999");
        assert_eq!(format_views(1000.0), "1.0K");
        assert_eq!(format_views(1500.0), "1.5K");
        assert_eq!(format_views(999_999.0), "1000.0K", "di bawah 1e6, belum M");
        assert_eq!(format_views(1_000_000.0), "1.0M");
        assert_eq!(format_views(2_450_000.0), "2.5M", "2.45 → 2.5, bulat ke atas");
        assert_eq!(format_views(12_345_678.0), "12.3M");
        assert_eq!(format_views(f64::NAN), "0");
        assert_eq!(format_views(f64::INFINITY), "0");
    }

    #[test]
    fn the_now_playing_card_carries_every_line_the_javascript_writes() {
        let t = now_playing(
            "Komang",
            "Vidiomalay",
            "3:45",
            12_345_678.0,
            "5 tahun lalu",
            "abc123XYZ_-",
            "Deskripsi singkat",
            "https://youtu.be/abc123XYZ_-",
        );
        assert!(t.starts_with("🎵 *NOW PLAYING*\n\n📌 *Judul:* Komang\n\n*DETAIL*\n"), "{t}");
        assert!(t.contains("👤 Channel: *Vidiomalay*\n"), "{t}");
        assert!(t.contains("👀 Views: *12.3M*\n"), "{t}");
        assert!(t.contains("🆔 ID: `abc123XYZ_-`\n\n"), "{t}");
        assert!(t.contains("*Deskripsi:*\n_Deskripsi singkat_\n\n"), "{t}");
        assert!(t.ends_with("_⏳ mengirim audio, harap tunggu..._"), "{t}");
    }

    #[test]
    fn an_empty_description_is_skipped_entirely() {
        let t = now_playing("T", "A", "1:00", 10.0, "baru", "id", "", "u");
        assert!(!t.contains("Deskripsi"), "{t}");
    }

    #[test]
    fn a_long_description_is_cut_at_150_with_its_ellipsis() {
        let desc = "a\nb".repeat(100);
        let t = now_playing("T", "A", "1:00", 10.0, "baru", "id", &desc, "u");
        let baris = t
            .lines()
            .find(|l| l.starts_with("_"))
            .expect("ada baris deskripsi");
        assert!(baris.ends_with("..._"), "{baris}");
        assert!(!baris.contains('\n'));
        // `substring(0, 150)` mengambil 150 karakter; `\n`-nya jadi spasi
        // sesudahnya, jadi panjangnya tetap 150.
        let isi = baris.trim_matches('_').trim_end_matches("...");
        assert_eq!(isi.chars().count(), 150, "{baris}");
        assert!(!baris.contains('\n'), "deskripsi satu baris: {baris}");
    }

    #[test]
    fn a_description_of_exactly_150_characters_gets_no_ellipsis() {
        let desc = "z".repeat(150);
        let t = now_playing("T", "A", "1:00", 10.0, "baru", "id", &desc, "u");
        assert!(t.contains(&format!("_{desc}_\n")), "{t}");
    }
}
