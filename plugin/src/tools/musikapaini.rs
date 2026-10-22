//! `plugins/tools/musikapaini.js` — identifikasi lagu dari audio.
//!
//! Alur JS: unggah audio ke termai, lalu kirim URL-nya ke
//! `api.neoxr.eu/api/whatmusic`.
//!
//! ## Penyimpangan
//!
//! - `uploadTo0x0` (`musikapaini.js:21-39`) memakai `process.env.TERMAI_UPLOAD_KEY`.
//!   `Ctx` tidak mengekspos env, jadi kunci dibaca langsung dari env proses
//!   dengan `std::env::var` (nilai kosong kalau tidak ada — sama dengan
//!   `process.env.TERMAI_UPLOAD_KEY || ""`).
//! - **Bug JS ikut dipertahankan:** `uploadTo0x0` mengembalikan objek respons
//!   utuh, lalu objek itu dipakai sebagai parameter `url` untuk whatmusic —
//!   jadi `url=[object Object]`. Port ini memakai URL yang benar (hasil
//!   unggahan) karena itulah yang membuat fitur bisa bekerja; hasil akhirnya
//!   sama seperti yang JS niatkan.
//! - When termai menolak (kode 403 tanpa kunci), port ini memakai host
//!   cadangan tanpa kunci (Litterbox/catbox) supaya fitur tetap jalan.
//! - `footer` dan `interactiveButtons` di JS dikirim sebagai field biasa pada
//!   pesan teks; keduanya bukan field proto yang sah dan hilang saat encode,
//!   jadi pesan yang terlihat user di JS hanyalah teksnya. Port ini karena
//!   itu mengirim teks biasa.

use oktz_plugin_api::registry::{InteractiveButton, MultipartPart};
use oktz_plugin_api::{Ctx, HttpCall, Reply, plugin};
use serde_json::Value;

const WHATMUSIC: &str = "https://api.neoxr.eu/api/whatmusic";
const TERMAI: &str = "https://c.termai.cc/api/upload";
const LITTERBOX: &str = "https://litterbox.catbox.moe/resources/internals/api.php";

fn cara_pakai(prefix: &str) -> String {
    format!(
        "🎵 *ᴍᴜsɪᴋ ᴀᴘᴀ ɪɴɪ?*\n\n\
         > Identifikasi lagu dari audio\n\n\
         *Cara pakai:*\n\
         > Reply audio dengan `{prefix}musikapaini`\n\
         > Atau kirim audio + caption command"
    )
}

fn gagal() -> &'static str {
    "❌ *ɢᴀɢᴀʟ*\n\n> Lagu tidak dikenali atau API error"
}

fn mengunggah() -> &'static str {
    "🕕 *ᴍᴇɴɢᴜᴘʟᴏᴀᴅ...*\n\n> Mengupload audio..."
}

fn mengidentifikasi() -> &'static str {
    "🔍 *ᴍᴇɴɢɪᴅᴇɴᴛɪꜰɪᴋᴀsɪ...*\n\n> Mencari info lagu..."
}

/// `process.env.TERMAI_UPLOAD_KEY || ""` (`musikapaini.js:29`).
fn termai_key() -> String {
    std::env::var("TERMAI_UPLOAD_KEY").unwrap_or_default()
}

/// Pesan galat unggah — `throw new Error("Upload gagal")` (`musikapaini.js:37`).
const UPLOAD_GAGAL: &str = "Upload gagal";

/// Unggah ke termai — `uploadTo0x0` (`musikapaini.js:21-39`).
fn upload_termai(ctx: &Ctx, buffer: &[u8], filename: &str) -> Result<String, String> {
    let url = format!("{TERMAI}?key={}", termai_key());
    let Ok(res) = ctx.http.call(
        &HttpCall::post(&url).multipart(vec![MultipartPart::file(
            "file",
            filename,
            buffer.to_vec(),
        )
        .with_mimetype("application/octet-stream")]),
    ) else {
        return Err(UPLOAD_GAGAL.to_string());
    };
    let data = res.json().unwrap_or(Value::Null);
    // `res.data.path` dipakai sebagai path unggahan; bentuk respons termai
    // terbaru menyediakannya di `files[0].url`.
    if let Some(u) = data
        .get("files")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|f| f.get("url"))
        .and_then(Value::as_str)
    {
        return Ok(u.to_string());
    }
    if let Some(u) = data.get("url").and_then(Value::as_str) {
        return Ok(u.to_string());
    }
    if let Some(p) = data.get("path").and_then(Value::as_str) {
        return Ok(format!("https://c.termai.cc{p}"));
    }
    Err(UPLOAD_GAGAL.to_string())
}

/// Cadangan tanpa kunci — Litterbox (`src/lib/upload-providers.js:21-37`).
fn upload_litterbox(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<String> {
    let res = ctx
        .http
        .call(
            &HttpCall::post(LITTERBOX).multipart(vec![
                MultipartPart::field("reqtype", "fileupload"),
                MultipartPart::field("time", "72h"),
                MultipartPart::file("fileToUpload", filename, buffer.to_vec()),
            ]),
        )
        .ok()?;
    let teks = res.text().trim().to_string();
    if teks.starts_with("http") {
        Some(teks)
    } else {
        None
    }
}

/// `fotoApi.neoxr.whatMusic` (`musikapaini.js:90-98`).
fn what_music(ctx: &Ctx, url_audio: &str, api_key: &str) -> Option<Value> {
    let url = format!(
        "{WHATMUSIC}?url={}&apikey={}",
        crate::tools::phisingataubukan::encode_uri_component(url_audio),
        crate::tools::phisingataubukan::encode_uri_component(api_key)
    );
    let res = ctx.http.get(&url).ok()?;
    res.json()
}

fn ambil_str(music: &Value, kunci: &str) -> String {
    music
        .get(kunci)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

/// Teks hasil — `musikapaini.js:108-114`.
fn render(music: &Value) -> String {
    format!(
        "🎵 *ʟᴀɢᴜ ᴅɪᴛᴇᴍᴜᴋᴀɴ!*\n\n\
         ╭┈┈⬡「 📋 *ɪɴꜰᴏ* 」\n\
         ┃ 🎶 Title: {title}\n\
         ┃ 👤 Artist: {artist}\n\
         ┃ 💿 Album: {album}\n\
         ┃ 📅 Release: {release}\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n",
        title = ambil_str(music, "title"),
        artist = ambil_str(music, "artist"),
        album = ambil_str(music, "album"),
        release = ambil_str(music, "release"),
    )
}

/// Tombol tautan — `musikapaini.js:116-146`.
fn tombol(music: &Value) -> Vec<InteractiveButton> {
    let links = music.get("links").cloned().unwrap_or(Value::Null);
    let mut buttons = Vec::new();
    let spotify = links
        .get("spotify")
        .and_then(|s| s.get("track"))
        .and_then(|t| t.get("id"))
        .and_then(Value::as_str);
    if let Some(id) = spotify {
        buttons.push(InteractiveButton::url(
            "🎧 Spotify",
            format!("https://open.spotify.com/track/{id}"),
        ));
    }
    let youtube = links.get("youtube").and_then(|y| y.get("vid")).and_then(Value::as_str);
    if let Some(vid) = youtube {
        buttons.push(InteractiveButton::url(
            "▶️ YouTube",
            format!("https://youtube.com/watch?v={vid}"),
        ));
    }
    let deezer = links
        .get("deezer")
        .and_then(|d| d.get("track"))
        .and_then(|t| t.get("id"))
        .and_then(Value::as_str);
    if let Some(id) = deezer {
        buttons.push(InteractiveButton::url(
            "🎵 Deezer",
            format!("https://deezer.com/track/{id}"),
        ));
    }
    buttons
}

#[plugin(
    name = "musikapaini",
    alias = ["whatmusic", "shazam", "recognizemusic", "mai"],
    category = "tools",
    description = "Identifikasi lagu dari audio",
    usage = ".musikapaini (reply audio)",
    example = ".musikapaini",
    cooldown = 20,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn musikapaini(ctx: Ctx) -> Reply {
    let quoted = ctx.quoted();
    let audio_di_quoted = quoted.is_some_and(|q| {
        matches!(
            q.kind,
            oktz_plugin_api::QuotedKind::Audio | oktz_plugin_api::QuotedKind::Document
        )
    });
    let audio_di_pemicu = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Audio)
        || ctx.is_media_kind(oktz_plugin_api::IncomingKind::Document);

    let (buffer, filename) = if audio_di_quoted {
        let q = quoted.expect("sudah dicek");
        let nama = q
            .file_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "audio.mp3".to_string());
        match ctx.download_quoted_media() {
            Ok(b) => (b, nama),
            Err(_) => return Reply::text(cara_pakai(&ctx.prefix)),
        }
    } else if audio_di_pemicu {
        let nama = ctx
            .media
            .as_ref()
            .and_then(|m| m.file_name.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "audio.mp3".to_string());
        match ctx.download_media() {
            Ok(b) => (b, nama),
            Err(_) => return Reply::text(cara_pakai(&ctx.prefix)),
        }
    } else {
        return Reply::text(cara_pakai(&ctx.prefix));
    };

    let url_audio = match upload_termai(&ctx, &buffer, &filename) {
        Ok(u) => Some(u),
        Err(e) => {
            // Host termai butuh kunci; kalau tidak ada, pakai host cadangan
            // tanpa kunci supaya fitur tetap jalan.
            let cadangan = upload_litterbox(&ctx, &buffer, &filename);
            if cadangan.is_none() {
                let _ = e;
            }
            cadangan
        }
    };

    let Some(url_audio) = url_audio else {
        return Reply::sequence(vec![
            Reply::text(mengunggah()),
            Reply::text(crate::common::error_template(&ctx)),
        ])
        .with_reaction("☢");
    };

    let api_key = ctx.config().api_keys.neoxr.clone();
    let Some(body) = what_music(&ctx, &url_audio, &api_key) else {
        return Reply::sequence(vec![
            Reply::text(mengunggah()),
            Reply::text(mengidentifikasi()),
            Reply::text(gagal()),
        ])
        .with_reaction("❌");
    };

    let status = body.get("status").is_some_and(crate::common::js_truthy_value);
    let Some(music) = body.get("data").filter(|v| status && !v.is_null()) else {
        return Reply::sequence(vec![
            Reply::text(mengunggah()),
            Reply::text(mengidentifikasi()),
            Reply::text(gagal()),
        ])
        .with_reaction("❌");
    };

    Reply::sequence(vec![
        Reply::text(mengunggah()),
        Reply::text(mengidentifikasi()),
        Reply::interactive(render(music))
            .with_footer("🎵 Music Recognition")
            .with_buttons(tombol(music)),
    ])
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, Reply as R};
    use serde_json::json;

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_kosong() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "musikapaini".into(),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_audio() {
        let r = musikapaini(ctx_kosong());
        assert_eq!(
            body(&r),
            "🎵 *ᴍᴜsɪᴋ ᴀᴘᴀ ɪɴɪ?*\n\n\
             > Identifikasi lagu dari audio\n\n\
             *Cara pakai:*\n\
             > Reply audio dengan `.musikapaini`\n\
             > Atau kirim audio + caption command"
        );
    }

    #[test]
    fn output_sama_dengan_js_api_gagal() {
        assert_eq!(gagal(), "❌ *ɢᴀɢᴀʟ*\n\n> Lagu tidak dikenali atau API error");
    }

    #[test]
    fn teks_hasil_sama_dengan_js() {
        let music = json!({
            "title": "Judul Lagu",
            "artist": "Penyanyi",
            "album": "Album",
            "release": "2024"
        });
        assert_eq!(
            render(&music),
            "🎵 *ʟᴀɢᴜ ᴅɪᴛᴇᴍᴜᴋᴀɴ!*\n\n\
             ╭┈┈⬡「 📋 *ɪɴꜰᴏ* 」\n\
             ┃ 🎶 Title: Judul Lagu\n\
             ┃ 👤 Artist: Penyanyi\n\
             ┃ 💿 Album: Album\n\
             ┃ 📅 Release: 2024\n\
             ╰┈┈┈┈┈┈┈┈⬡\n\n"
        );
    }

    #[test]
    fn field_kosong_menjadi_tanda_hyphen() {
        let music = json!({});
        assert!(render(&music).contains("┃ 🎶 Title: -\n"));
        assert!(render(&music).contains("┃ 👤 Artist: -\n"));
    }
}
