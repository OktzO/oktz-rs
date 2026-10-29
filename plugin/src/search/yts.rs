//! `plugins/search/yts.js` — mencari video di YouTube.
//!
//! `ponytail:` JS memakai paket npm `yt-search`; padanannya ada di
//! `search/ytsearch.rs` (data sumbernya sama: `ytInitialData`).
//!
//! `ponytail:` kartu yang dikirim adalah `buttonsMessage` (headerType 6) dengan
//! `locationMessage` berisi thumbnail 300×170, judul, dan alamat
//! `📺 Channel: … | ⏱️ Durasi: …`. Rust memakai `CardHeader::location`.
//!
//! `ponytail:` `config.bot.name` jadi footer kartu.

use oktz_plugin_api::{Ctx, Reply, plugin};
use oktz_plugin_api::registry::{
    CardContext, CardHeader, CardLocation, InteractiveButton,
};

use super::ytsearch::{self, Video};

/// `yts.js:25` — kata kunci kosong.
pub(crate) fn tanpa_kunci() -> String {
    "❌ *Waduh, kata kuncinya kosong!*\n\nKamu harus memasukkan kata kunci judul video yang ingin dicari ya. \n\nContoh penggunaan: `.yts lagu galau indonesia`".to_string()
}

/// `yts.js:36` — tidak ada hasil.
pub(crate) fn tidak_ada_hasil() -> String {
    "⚠️ *Maaf banget, pencarian tidak menemukan hasil apa pun.* \n\nMungkin kata kuncinya terlalu spesifik. Coba gunakan kata kunci lain yang lebih umum ya!".to_string()
}

/// `yts.js:95` — error sistem.
pub(crate) fn ada_masalah() -> String {
    "😔 *Aduh, sepertinya ada masalah di sistemku.* \n\nTerjadi kesalahan saat mencoba mencari video tersebut di YouTube. Mohon tunggu beberapa saat dan coba lagi nanti ya!".to_string()
}

/// `yts.js:44-58` — isi kartu.
pub(crate) fn isi_kartu(text: &str, v: &Video) -> String {
    format!(
        "✨ *HASIL PENCARIAN YOUTUBE* ✨\n\n\
         Halo! Ini dia hasil pencarian teratas yang aku temukan berdasarkan kata kunci yang kamu berikan. \n\n\
         🔎 *Kata Kunci Pencarian*: {text}\n\
         🎬 *Judul Video*: {}\n\
         📺 *Nama Channel*: {}\n\
         ⏱️ *Durasi Video*: {}\n\
         👁️ *Jumlah Penonton*: {} views\n\
         📅 *Waktu Diunggah*: {}\n\
         🔗 *Tautan Video*: {}\n\n\
         *Catatan Tambahan*: Thumbnail dari video ini sudah aku sematkan di bagian atas pesan (peta lokasi) sesuai permintaanmu. Keren kan? 😎\n\n\
         Pilih salah satu tombol di bawah ini untuk langsung mengunduh hasil video atau audio-nya!",
        v.title, v.author_name, v.timestamp, v.views, v.ago, v.url
    )
}

/// Ambil thumbnail 300×170 seperti `sharp(...).resize(300, 170).jpeg()`
/// (`yts.js:41-42`).
fn thumbnail(http: &dyn oktz_plugin_api::HttpGet, url: &str) -> Option<Vec<u8>> {
    if url.is_empty() {
        return None;
    }
    let res = http.get(url).ok()?;
    oktz_plugin_api::gambar::sharp(res.body.bytes())
        .ok()?
        .ubah_ukuran(300, 170)
        .ke_jpeg(80)
        .ok()
}

#[plugin(
    name = "yts",
    alias = ["ytsearch", "youtubesearch"],
    category = "search",
    description = "Mencari video di YouTube berdasarkan kata kunci dan menampilkan detail lengkap beserta thumbnail.",
    usage = ".yts <query>",
    example = ".yts lagu pop terbaru",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn yts(ctx: Ctx) -> Reply {
    let text = ctx.text.trim().to_string();
    if text.is_empty() {
        return Reply::text(tanpa_kunci());
    }

    let videos = match ytsearch::search(ctx.http.as_ref(), &text) {
        Ok(v) => v,
        Err(_) => {
            // `yts.js:92-96` — ❌ lalu pesan sistem.
            return Reply::text(ada_masalah()).with_reaction("❌");
        }
    };

    let Some(first) = videos.first() else {
        // `yts.js:34-37` — ❌ lalu pesan tidak ada hasil.
        return Reply::text(tidak_ada_hasil()).with_reaction("❌");
    };

    let body = isi_kartu(&text, first);
    let thumb = thumbnail(ctx.http.as_ref(), &first.thumbnail);

    let header = CardHeader::new().location(CardLocation {
        latitude: 0.0,
        longitude: 0.0,
        name: first.title.clone(),
        address: Some(format!(
            "📺 Channel: {} | ⏱️ Durasi: {}",
            first.author_name, first.timestamp
        )),
        thumbnail: thumb,
    });

    Reply::interactive(body)
        .with_footer(ctx.config().bot.name.clone())
        .with_header(header)
        .with_button_card(InteractiveButton::quick_reply(
            "🎥 Unduh Video",
            format!(".ytmp4 {}", first.url),
        ))
        .with_button_card(InteractiveButton::quick_reply(
            "🎵 Unduh Audio",
            format!(".ytmp3 {}", first.url),
        ))
        .with_context(CardContext::new().quote_trigger())
        .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub HTTP: membalas body yang sama untuk semua request.
    #[derive(Debug)]
    struct Stub {
        body: String,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn body(v: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: v.to_string(),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: None,
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
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "yts".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("yts").unwrap().handler)(ctx)
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

    const HASIL: &str = r#"{"contents":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[
      {"itemSectionRenderer":{"contents":[
        {"videoRenderer":{
          "videoId":"aaaaaaaaaaa",
          "title":{"runs":[{"text":"Lagu Uji"}]},
          "ownerText":{"runs":[{"text":"Channel Uji"}]},
          "viewCountText":{"simpleText":"1,234,567 views"},
          "publishedTimeText":{"simpleText":"2 years ago"},
          "lengthText":{"simpleText":"3:33"}
        }}
      ]}}
    ]}}}}]}}"#;

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("yts").unwrap();
        assert_eq!(p.meta.name, "yts");
        assert_eq!(p.meta.alias, ["ytsearch", "youtubesearch"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.usage, ".yts <query>");
        assert_eq!(p.meta.example, ".yts lagu pop terbaru");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `yts.js:25` — kata kunci kosong.
        assert_eq!(
            text_of(&run(Stub::body("{}"), "")),
            "❌ *Waduh, kata kuncinya kosong!*\n\nKamu harus memasukkan kata kunci judul video yang ingin dicari ya. \n\nContoh penggunaan: `.yts lagu galau indonesia`"
        );
        // `yts.js:36` — tidak ada hasil.
        let r = run(Stub::body(r#"{"contents":{"tabs":[]}}"#), "x");
        assert_eq!(
            text_of(&r),
            "⚠️ *Maaf banget, pencarian tidak menemukan hasil apa pun.* \n\nMungkin kata kuncinya terlalu spesifik. Coba gunakan kata kunci lain yang lebih umum ya!"
        );
        assert_eq!(reaction_of(&r), Some("❌"));
        // `yts.js:44-58` — isi kartu.
        let r = run(Stub::body(HASIL), "lagu uji");
        let t = text_of(&r);
        assert!(
            t.starts_with("✨ *HASIL PENCARIAN YOUTUBE* ✨\n\nHalo! Ini dia hasil pencarian teratas yang aku temukan berdasarkan kata kunci yang kamu berikan. \n\n"),
            "{t}"
        );
        assert!(t.contains("🔎 *Kata Kunci Pencarian*: lagu uji\n"), "{t}");
        assert!(t.contains("🎬 *Judul Video*: Lagu Uji\n"), "{t}");
        assert!(t.contains("📺 *Nama Channel*: Channel Uji\n"), "{t}");
        assert!(t.contains("⏱️ *Durasi Video*: 3:33\n"), "{t}");
        assert!(t.contains("👁️ *Jumlah Penonton*: 1234567 views\n"), "{t}");
        assert!(t.contains("📅 *Waktu Diunggah*: 2 years ago\n"), "{t}");
        assert!(
            t.contains("🔗 *Tautan Video*: https://www.youtube.com/watch?v=aaaaaaaaaaa\n\n"),
            "{t}"
        );
        assert!(t.ends_with("Pilih salah satu tombol di bawah ini untuk langsung mengunduh hasil video atau audio-nya!"), "{t}");
        assert_eq!(reaction_of(&r), Some("✅"));
    }

    #[test]
    fn tombol_unduh_memakai_url_video() {
        let r = run(Stub::body(HASIL), "lagu uji");
        match r.without_mutations().tanpa_reaksi() {
            Reply::Interactive { kartu } => {
                assert_eq!(kartu.buttons.len(), 2);
                assert_eq!(kartu.buttons[0].name, "quick_reply");
                assert_eq!(
                    kartu.buttons[0].params["display_text"],
                    "🎥 Unduh Video"
                );
                assert_eq!(
                    kartu.buttons[0].params["id"],
                    ".ytmp4 https://www.youtube.com/watch?v=aaaaaaaaaaa"
                );
                assert_eq!(kartu.buttons[1].params["display_text"], "🎵 Unduh Audio");
                assert_eq!(
                    kartu.buttons[1].params["id"],
                    ".ytmp3 https://www.youtube.com/watch?v=aaaaaaaaaaa"
                );
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }
}
