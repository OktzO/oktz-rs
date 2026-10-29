//! `plugins/download/tiktokdl.js` — download video/slide TikTok tanpa watermark.
//!
//! Command-nya dipanggil dengan beberapa nama: JS `name: ["tiktok", "tt",
//! "ttmp4"]` dan `alias: ["tiktokdl", "ttdown"]` (`tiktokdl.js:381-384`), jadi
//! di sini `tiktok` jadi nama dan sisanya alias.
//!
//! Yang diport apa adanya:
//! - Pemeriksaan URL (`tiktokdl.js:407`) — `tiktok.com`, `vt.tiktok`, atau
//!   `vm.tiktok`.
//! - Caption video (`tiktokdl.js:411-431`) dan caption foto (`tiktokdl.js:452`).
//! - Tombol "🎵 Ambil Music" (`tiktokdl.js:399-405`) — di JS dikirim lewat
//!   `sock.sendButton(..., { type: "video", buttons })`, jadi video-nya jadi
//!   HEADER kartu native-flow dan tombolnya menempel di kartu yang sama.
//! - Pesan gagal (`tiktokdl.js:509-514`) dan pesan "slide berhasil"
//!   (`tiktokdl.js:504`).
//!
//! `ponytail:` yang berbeda dari JS, beserta alasannya:
//! - `resolveTikTokUrl` (`tiktokdl.js:3-24`) mengikuti redirect短 link
//!   `vt.tiktok.com` dan mengambil URL akhirnya. Klien HTTP Rust mengikuti
//!   redirect secara internal tapi tidak参加会议 URL akhir, jadi short link
//!   diteruskan apa adanya — Savett dan TikWM sama-sama menerimanya.
//! - Urutan provider: JS mencoba TikWM lalu Savett (`tiktokdl.js:352-375`).
//!   `oktz_scraper::tiktok::ttdown` sudah merantai TikWM → Savett → YuuLabs →
//!   MusicalDown, jadi rantai yang lebih panjang itu yang dipakai di sini.
//! - `formatDate` (`tiktokdl.js:35-46`) memakai `toLocaleDateString` dengan
//!   locale Inggris; tanpa ICU di Rust, scraper memakai format ISO-like.
//! - Album slide butuh `albumMessage` + `relayMessage` per slide
//!   (`tiktokdl.js:470+`); kontrak `Reply` cuma satu media per pesan, jadi
//!   cabang foto mengirim foto pertama saja.

use oktz_plugin_api::registry::{CardHeader, InteractiveButton};
use oktz_plugin_api::{Ctx, HttpCall, MediaSource as Ms, Reply, plugin};
use serde_json::Value;

/// `tiktokdl.js:401-405` — regex `/tiktok\.com|vt\.tiktok|vm\.tiktok/i`.
pub(crate) fn url_tiktok_valid(url: &str) -> bool {
    let kecil = url.to_lowercase();
    kecil.contains("tiktok.com") || kecil.contains("vt.tiktok") || kecil.contains("vm.tiktok")
}

/// `tiktokdl.js:410-411` — pesan tanpa URL.
pub(crate) fn bantuan(prefix: &str, command: &str) -> String {
    format!("📌 Contoh: *{prefix}{command} https://vt.tiktok.com/...*")
}

/// `tiktokdl.js:407-409` — URL bukan TikTok.
pub(crate) const URL_TIDAK_VALID: &str =
    "❌ URL tidak valid. Gunakan link TikTok yang valid.\n\nContoh: `.tt https://vt.tiktok.com/ZSxxxx/`";

/// `tiktokdl.js:434-441` — UA unduhan video.
const UA_VIDEO: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";

/// `tiktokdl.js:435` — `Referer` unduhan video.
const REFERER_VIDEO: &str = "https://www.tiktok.com/";

/// `result.data` → unduhan pertama yang cocok.
fn unduhan<'a>(hasil: &'a Value, tipe: &str) -> Option<&'a Value> {
    let downloads = hasil.get("downloads")?.as_array()?;
    downloads
        .iter()
        .find(|d| d.get("type").and_then(Value::as_str) == Some(tipe))
}

/// `result.data[].url` untuk video tanpa watermark.
fn url_video(hasil: &Value) -> Option<String> {
    for tipe in ["nowatermark_hd", "nowatermark", "video", "watermark"] {
        if let Some(u) = unduhan(hasil, tipe)
            .and_then(|d| d.get("url"))
            .and_then(Value::as_str)
            .filter(|u| !u.is_empty())
        {
            return Some(u.to_string());
        }
    }
    None
}

/// `result.data[].url` untuk MP3.
fn url_mp3(hasil: &Value) -> Option<String> {
    unduhan(hasil, "mp3")
        .and_then(|d| d.get("url"))
        .and_then(Value::as_str)
        .filter(|u| !u.is_empty())
        .map(str::to_string)
}

/// Ambil field bertingkat: `"a.b"`.
fn field(hasil: &Value, path: &str) -> String {
    let mut cur = hasil;
    for bagian in path.split('.') {
        match cur.get(bagian) {
            Some(v) => cur = v,
            None => return String::new(),
        }
    }
    match cur {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// `tiktokdl.js:411-431` — caption video.
pub(crate) fn caption_video(hasil: &Value) -> String {
    let provider = {
        let p = field(hasil, "_provider");
        if p.is_empty() {
            "tikwm".to_string()
        } else {
            p
        }
    };
    format!(
        "🎵 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞  -  𝗗 𝗢 𝗪 𝗡 𝗟 𝗢 𝗔 𝗗 𝗘 𝗥* [{provider}]\n\n\
- Author: *{}* ({})\n\
- Caption: {}\n\
- Music: {} - {}\n\
- Duration: {}\n\
- Uploaded: {}\n\
- Region: {}\n\n\
*Statistik Video:*\n\
- Views: *{}*\n\
- Likes: *{}*\n\
- Comments: *{}*\n\
- Shares: *{}*\n\
- Downloads: *{}*",
        field(hasil, "author.username"),
        field(hasil, "author.fullname"),
        {
            let t = field(hasil, "title");
            if t.is_empty() { "-".to_string() } else { t }
        },
        field(hasil, "music_info.title"),
        field(hasil, "music_info.author"),
        field(hasil, "duration"),
        field(hasil, "taken_at"),
        field(hasil, "region"),
        field(hasil, "stats.views"),
        field(hasil, "stats.likes"),
        field(hasil, "stats.comment"),
        field(hasil, "stats.share"),
        field(hasil, "stats.download"),
    )
}

/// `tiktokdl.js:467-479` — caption foto.
///
/// Perbedaannya dengan `caption_video`: baris `- Music:` ikut, baris
/// `- Uploaded:` (bukan `- Taken At:`), dan judul statistiknya
/// `*Statistik Konten:*`.
pub(crate) fn caption_foto(hasil: &Value) -> String {
    let provider = {
        let p = field(hasil, "_provider");
        if p.is_empty() {
            "tikwm".to_string()
        } else {
            p
        }
    };
    format!(
        "📸 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞  -  𝗗 𝗢 𝗪 𝗡 𝗟 𝗢 𝗔 𝗗 𝗘 𝗥* [{provider}]\n\n\
- Author: *{}* ({})\n\
- Caption: {}\n\
- Music: {} - {}\n\
- Uploaded: {}\n\
- Region: {}\n\n\
*Statistik Konten:*\n\
- Views: *{}*\n\
- Likes: *{}*\n\
- Comments: *{}*\n\
- Shares: *{}*\n\
- Downloads: *{}*",
        field(hasil, "author.username"),
        field(hasil, "author.fullname"),
        {
            let t = field(hasil, "title");
            if t.is_empty() { "-".to_string() } else { t }
        },
        field(hasil, "music_info.title"),
        field(hasil, "music_info.author"),
        field(hasil, "taken_at"),
        field(hasil, "region"),
        field(hasil, "stats.views"),
        field(hasil, "stats.likes"),
        field(hasil, "stats.comment"),
        field(hasil, "stats.share"),
        field(hasil, "stats.download"),
    )
}

/// `tiktokdl.js:504` — pesan setelah slide berhasil dikirim.
pub(crate) const SLIDE_BERHASIL: &str =
    "📸 Slide berhasil dikirim!\nTekan tombol di bawah untuk ambil music.";

/// `tiktokdl.js:509-514` — cabang `catch`.
///
/// `detail` = `e.message?.slice(0, 300) || "Unknown error"` (`tiktokdl.js:512`).
pub(crate) fn gagal(prefix: &str, detail: &str) -> String {
    let potong: String = detail.chars().take(300).collect();
    let detail = if potong.is_empty() {
        "Unknown error"
    } else {
        potong.as_str()
    };
    format!(
        "❌ *Gagal mengunduh TikTok*\n\n> {detail}\n\nCoba lagi nanti, atau bisa coba *{prefix}tt2* sebagai alternatif.\nJika tetap gagal, pastikan link valid dan video tidak private."
    )
}

#[plugin(
    name = "tiktok",
    alias = ["tt", "ttmp4", "tiktokdl", "ttdown"],
    category = "download",
    description = "Download video/slide TikTok tanpa watermark",
    usage = ".tiktok <url>",
    example = ".tiktok https://vt.tiktok.com/xxx",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn tiktok(ctx: Ctx) -> Reply {
    let teks = ctx.text.trim();
    if teks.is_empty() {
        return Reply::text(bantuan(&ctx.prefix, &ctx.command)).with_reaction("❌");
    }
    if !url_tiktok_valid(teks) {
        return Reply::text(URL_TIDAK_VALID).with_reaction("❌");
    }

    let gagal_reply = |detail: &str| {
        Reply::text(gagal(&ctx.prefix, detail)).with_reaction("❌")
    };

    let Ok(hasil) = oktz_scraper::tiktok::ttdown(&*ctx.http, teks) else {
        return gagal_reply("Semua provider TikTok gagal");
    };

    // `tiktokdl.js:399-405` — tombol "Ambil Music" menunjuk ke `.ttmp3`.
    let tombol_musik = InteractiveButton::quick_reply(
        "🎵 Ambil Music",
        format!("{}ttmp3 {teks}", ctx.prefix),
    );

    if let Some(alamat) = url_video(&hasil) {
        // `axios.get(videoItem.url, { responseType: "arraybuffer" })`
        // (`tiktokdl.js:433-441`).
        let Ok(res) = ctx.http.call(
            &HttpCall::get(&alamat)
                .header("User-Agent", UA_VIDEO)
                .header("Referer", REFERER_VIDEO),
        ) else {
            return gagal_reply("Gagal mendapatkan file video");
        };
        if !(200..300).contains(&res.status) {
            return gagal_reply("Gagal mendapatkan file video");
        }
        let content_type = res
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        // `if (ct.includes("text/html")) throw` (`tiktokdl.js:443-444`).
        if content_type.contains("text/html") {
            return gagal_reply("Gagal mendapatkan file video (response HTML)");
        }
        let mimetype = if content_type.contains("video") {
            content_type
        } else {
            "video/mp4".to_string()
        };
        // `sock.sendButton(jid, buffer, caption, m, { type: "video", buttons })`
        // (`tiktokdl.js:447-453`) — video jadi header kartu native-flow.
        return Reply::interactive(caption_video(&hasil))
            .with_header(CardHeader::new().video(
                Ms::bytes(res.body.bytes().to_vec()).with_mimetype(mimetype),
            ))
            .with_button_card(tombol_musik)
            .with_reaction("✅");
    }

    // Cabang foto/slide (`tiktokdl.js:455+`).
    if let Some(foto) = unduhan(&hasil, "photo")
        .and_then(|d| d.get("url"))
        .and_then(Value::as_str)
        .filter(|u| !u.is_empty())
    {
        // `tiktokdl.js:496-506` — album slide lalu tombol "ambil music".
        //
        // `ponytail:` `Reply::Sequence` tidak boleh dibungkus `Reaction`
        // (pipeline merender `Sequence` lewat loop, bukan `render_reply`), jadi
        // reaksi ✅ pada jalur ini tidak ikut — sama seperti
        // `download/tiktokdl2.rs`.
        return Reply::sequence(vec![
            Reply::interactive(caption_foto(&hasil))
                .with_header(CardHeader::new().image(
                    Ms::url(foto).with_mimetype("image/jpeg"),
                ))
                .with_button_card(tombol_musik),
            Reply::text(SLIDE_BERHASIL),
        ]);
    }

    // Cabang MP3 (`tiktokdl.js:487+`).
    if let Some(mp3) = url_mp3(&hasil) {
        return Reply::sequence(vec![
            Reply::text(caption_foto(&hasil)),
            Reply::audio(Ms::url(mp3).with_mimetype("audio/mpeg")),
        ]);
    }

    gagal_reply("Tidak ada slide foto ditemukan")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hasil_contoh() -> Value {
        json!({
            "title": "Contoh video",
            "author": { "username": "Zann", "fullname": "zann" },
            "cover": "https://www.tikwm.com/cover.jpg",
            "duration": "12 Seconds",
            "durations": 12,
            "taken_at": "2025-01-02 03:04:05 UTC",
            "region": "ID",
            "stats": {
                "views": "1.234", "likes": "56", "comment": "7",
                "share": "8", "download": "9"
            },
            "music_info": { "title": "Lagu", "author": "Artis" },
            "_provider": "tikwm",
            "downloads": [
                { "type": "nowatermark", "url": "https://www.tikwm.com/video.mp4" },
                { "type": "mp3", "url": "https://www.tikwm.com/audio.mp3" }
            ]
        })
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("tiktok").unwrap();
        assert_eq!(p.meta.name, "tiktok");
        // JS `name: ["tiktok", "tt", "ttmp4"]` + `alias: ["tiktokdl", "ttdown"]`.
        assert_eq!(p.meta.alias, ["tt", "ttmp4", "tiktokdl", "ttdown"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video/slide TikTok tanpa watermark");
        assert_eq!(p.meta.usage, ".tiktok <url>");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn semua_alias_menunjuk_plugin_ini() {
        for alias in ["tt", "ttmp4", "tiktokdl", "ttdown"] {
            let p = oktz_plugin_api::find(alias).expect(alias);
            assert_eq!(p.meta.name, "tiktok", "alias {alias} salah plugin");
        }
    }

    #[test]
    fn url_tiktok_valid_menurut_regex_js() {
        assert!(url_tiktok_valid("https://vt.tiktok.com/ZSxxxx/"));
        assert!(url_tiktok_valid("https://www.tiktok.com/@user/video/1"));
        assert!(url_tiktok_valid("https://vm.tiktok.com/abc"));
        assert!(!url_tiktok_valid("https://youtube.com/watch?v=1"));
    }

    #[test]
    fn caption_video_mengikuti_baris_javascript() {
        let c = caption_video(&hasil_contoh());
        assert!(c.contains("🎵 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞"), "{c}");
        assert!(c.contains("[tikwm]"), "{c}");
        assert!(c.contains("- Author: *Zann* (zann)"), "{c}");
        assert!(c.contains("- Caption: Contoh video"), "{c}");
        assert!(c.contains("- Music: Lagu - Artis"), "{c}");
        assert!(c.contains("- Duration: 12 Seconds"), "{c}");
        assert!(c.contains("- Region: ID"), "{c}");
        assert!(c.contains("- Views: *1.234*"), "{c}");
        assert!(c.contains("*Statistik Video:*"), "{c}");
    }

    #[test]
    fn caption_foto_mengikuti_baris_javascript() {
        // `tiktokdl.js:467-479` — ikon 📸, baris `- Music:`, `- Uploaded:` dan
        // judul `*Statistik Konten:*`.
        let c = caption_foto(&hasil_contoh());
        assert!(c.starts_with("📸 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞"), "{c}");
        assert!(c.contains("[tikwm]"), "{c}");
        assert!(c.contains("- Author: *Zann* (zann)"), "{c}");
        assert!(c.contains("- Caption: Contoh video"), "{c}");
        assert!(c.contains("- Music: Lagu - Artis"), "{c}");
        assert!(c.contains("- Uploaded: 2025-01-02 03:04:05 UTC"), "{c}");
        assert!(c.contains("- Region: ID"), "{c}");
        assert!(c.contains("*Statistik Konten:*"), "{c}");
        assert!(!c.contains("*Statistik Video:*"), "{c}");
        assert!(!c.contains("Taken At"), "{c}");
    }

    #[test]
    fn output_sama_dengan_js() {
        // `tiktokdl.js:513` — pesan gagal.
        assert_eq!(
            gagal(".", "Gagal mengunduh"),
            "❌ *Gagal mengunduh TikTok*\n\n> Gagal mengunduh\n\nCoba lagi nanti, atau bisa coba *.tt2* sebagai alternatif.\nJika tetap gagal, pastikan link valid dan video tidak private."
        );
        // `tiktokdl.js:504` — pesan slide.
        assert_eq!(
            SLIDE_BERHASIL,
            "📸 Slide berhasil dikirim!\nTekan tombol di bawah untuk ambil music."
        );
        // Detail kosong → "Unknown error" (`tiktokdl.js:512`).
        assert!(gagal(".", "").contains("> Unknown error"));
    }

    #[test]
    fn tombol_music_menunjuk_ttmp3() {
        let b = InteractiveButton::quick_reply("🎵 Ambil Music", ".ttmp3 https://vt.tiktok.com/x");
        assert_eq!(b.name, "quick_reply");
        assert_eq!(b.params["id"], ".ttmp3 https://vt.tiktok.com/x");
    }

    #[test]
    fn url_video_memilih_tanpa_watermark_dulu() {
        assert_eq!(
            url_video(&hasil_contoh()).as_deref(),
            Some("https://www.tikwm.com/video.mp4")
        );
    }
}
