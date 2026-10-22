//! `plugins/download/ttmp3.js` — download audio TikTok.
//!
//! Command-nya dipanggil dengan beberapa nama: JS `name: ["ttmp3"]` dan
//! `alias: ["ttmusic", "tiktokmusic"]` (`ttmp3.js:11-13`). Tombol "🎵 Ambil
//! Music" di plugin `tiktok` menunjuk ke perintah ini
//! (`tiktokdl.js:399-405`).
//!
//! Yang diport apa adanya:
//! - Pemeriksaan URL (`ttmp3.js:95`) — `tiktok.com` atau `vt.tiktok`.
//! - Alur: pakai MP3 dari provider; kalau tidak ada, ambil video lalu
//!   ekstrak audio-nya dengan ffmpeg (`ttmp3.js:104-135`).
//!
//! `ponytail:` yang berbeda dari JS, beserta alasannya:
//! - JS menulis berkas sementara di `tmp/` lalu menghapusnya; di sini
//!   video dan audio mengalir lewat pipe ffmpeg, jadi tidak ada berkas
//!   yang perlu dibersihkan.
//! - `fileName: TikTok_Audio_${Date.now()}.mp3` (`ttmp3.js:147`) tidak bisa
//!   ditulis: kontrak `Reply::Audio` tidak punya nama berkas.
//! - `Referer` saat mengunduh MP3 memakai origin URL (`ttmp3.js:127`); di sini
//!   memakai origin yang sama persis.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

/// `ttmp3.js:31-33` — UA unduhan audio.
const UA_AUDIO: &str = "Mozilla/5.0 (Linux; Android 15; SM-F958 Build/AP3A.240905.015) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.6723.86 Mobile Safari/537.36";

/// `ttmp3.js:34` — `Referer` unduhan video.
const REFERER_VIDEO: &str = "https://www.tiktok.com/";

/// `ttmp3.js:86-91` — pesan tanpa URL.
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "╭┈┈⬡「 🎵 *ᴛɪᴋᴛᴏᴋ ᴅᴏᴡɴʟᴏᴀᴅ* 」\n┃ ㊗ ᴜsᴀɢᴇ: `{prefix}ttmp3 <url>`\n╰┈┈⬡\n\n> Contoh: {prefix}ttmp3 https://vt.tiktok.com/xxx"
    )
}

/// `ttmp3.js:95` — `/tiktok\.com|vt\.tiktok/i`.
pub(crate) fn url_tiktok_valid(url: &str) -> bool {
    let kecil = url.to_lowercase();
    kecil.contains("tiktok.com") || kecil.contains("vt.tiktok")
}

/// `result.downloads` → unduhan pertama yang cocok.
fn unduhan<'a>(hasil: &'a Value, tipe: &str) -> Option<&'a Value> {
    hasil
        .get("downloads")?
        .as_array()?
        .iter()
        .find(|d| d.get("type").and_then(Value::as_str) == Some(tipe))
}

/// `extractAudioFromVideo` (`ttmp3.js:36-70`) — video → MP3 lewat ffmpeg.
///
/// `ffmpeg -i pipe:0 -vn -acodec libmp3lame -f mp3 pipe:1` — skalanya sama
/// dengan `ffmpeg(input).noVideo().audioCodec("libmp3lame").format("mp3")`.
fn ekstrak_audio(video: &[u8]) -> Result<Vec<u8>, String> {
    oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(
        &[
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            "pipe:0",
            "-vn",
            "-acodec",
            "libmp3lame",
            "-f",
            "mp3",
            "pipe:1",
        ],
        video,
    )
}

#[plugin(
    name = "ttmp3",
    alias = ["ttmusic", "tiktokmusic"],
    category = "download",
    description = "Download audio TikTok",
    usage = ".ttmp3 <url>",
    example = ".ttmp3 https://vt.tiktok.com/xxx",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ttmp3(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();
    if url.is_empty() {
        return Reply::text(bantuan(&ctx.prefix));
    }
    if !url_tiktok_valid(url) {
        return Reply::text("❌ URL tidak valid. Gunakan link TikTok.");
    }

    let gagal = |pesan: &str| {
        Reply::text(format!("❌ *ɢᴀɢᴀʟ ᴍᴇɢɴᴅᴜᴅᴜʜ*\n\n> {pesan}"))
            .with_reaction("❌")
    };

    let Ok(hasil) = oktz_scraper::tiktok::ttdown(&*ctx.http, url) else {
        return gagal("Gagal mengambil data TikTok.");
    };

    // `ttmp3.js:106` — MP3 langsung dari provider.
    if let Some(audio) = unduhan(&hasil, "mp3")
        .and_then(|d| d.get("url"))
        .and_then(Value::as_str)
        .filter(|u| !u.is_empty())
    {
        let Ok(res) = ctx.http.call(
            &HttpCall::get(audio)
                .header("User-Agent", UA_AUDIO)
                .header("Referer", origin_dari(audio)),
        ) else {
            return gagal("Gagal mengunduh audio TikTok.");
        };
        if (200..300).contains(&res.status) {
            return Reply::audio(
                MediaSource::bytes(res.body.bytes().to_vec()).with_mimetype("audio/mpeg"),
            )
            .with_reaction("✅");
        }
    }

    // `ttmp3.js:111-118` — tidak ada MP3, ambil videonya lalu ekstrak audio.
    let Some(video) = ["nowatermark_hd", "nowatermark"]
        .iter()
        .find_map(|tipe| {
            unduhan(&hasil, tipe)
                .and_then(|d| d.get("url"))
                .and_then(Value::as_str)
                .filter(|u| !u.is_empty())
        })
    else {
        return gagal("Audio TikTok tidak ditemukan.");
    };
    let Ok(res) = ctx.http.call(
        &HttpCall::get(video)
            .header("User-Agent", UA_AUDIO)
            .header("Referer", REFERER_VIDEO),
    ) else {
        return gagal("Gagal mengunduh video TikTok.");
    };
    if !(200..300).contains(&res.status) {
        return gagal("Gagal mengunduh video TikTok.");
    }
    match ekstrak_audio(res.body.bytes()) {
        Ok(mp3) if !mp3.is_empty() => {
            Reply::audio(MediaSource::bytes(mp3).with_mimetype("audio/mpeg"))
                .with_reaction("✅")
        }
        Ok(_) => gagal("Gagal mengekstrak audio TikTok"),
        Err(e) => gagal(&format!("Gagal mengekstrak audio TikTok: {e}")),
    }
}

/// `new URL(audioSource).origin` (`ttmp3.js:127`).
fn origin_dari(url: &str) -> String {
    let tanpa_skema = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host = tanpa_skema.split('/').next().unwrap_or(tanpa_skema);
    if url.starts_with("http://") {
        format!("http://{host}/")
    } else {
        format!("https://{host}/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("ttmp3").unwrap();
        assert_eq!(p.meta.name, "ttmp3");
        // JS `name: ["ttmp3"]` + `alias: ["ttmusic", "tiktokmusic"]`.
        assert_eq!(p.meta.alias, ["ttmusic", "tiktokmusic"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download audio TikTok");
        assert_eq!(p.meta.usage, ".ttmp3 <url>");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn semua_alias_menunjuk_plugin_ini() {
        for alias in ["ttmusic", "tiktokmusic"] {
            let p = oktz_plugin_api::find(alias).expect(alias);
            assert_eq!(p.meta.name, "ttmp3", "alias {alias} salah plugin");
        }
    }

    #[test]
    fn url_tiktok_valid_menurut_regex_js() {
        // `/tiktok\.com|vt\.tiktok/i` — `vm.tiktok.com` tetap cocok karena
        // mengandung `tiktok.com`.
        assert!(url_tiktok_valid("https://vt.tiktok.com/ZSxxxx/"));
        assert!(url_tiktok_valid("https://www.tiktok.com/@user/video/1"));
        assert!(url_tiktok_valid("https://vm.tiktok.com/abc"));
        assert!(!url_tiktok_valid("https://youtube.com/watch?v=1"));
    }

    #[test]
    fn bantuan_menyebut_prefix() {
        assert!(bantuan(".").contains("`.ttmp3 <url>`"));
        assert!(bantuan(".").contains("https://vt.tiktok.com/xxx"));
    }

    #[test]
    fn origin_dikirim_dari_host_url() {
        assert_eq!(
            origin_dari("https://www.tikwm.com/audio.mp3"),
            "https://www.tikwm.com/"
        );
        assert_eq!(origin_dari("http://cdn.tikwm.com/a.mp3"), "http://cdn.tikwm.com/");
    }
}
