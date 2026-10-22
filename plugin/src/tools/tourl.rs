//! `plugins/tools/tourl.js` — upload media ke beberapa host, atau paste teks
//! dan dapet link.
//!
//! ## Penyimpangan
//!
//! - JS menjalankan fan-out keenam host lewat `runUploadFanout`
//!   (`src/lib/upload-fanout.js`) dengan batas waktu per host dan batas
//!   total. Trait `HttpGet` di `oktz-plugin-api` tidak punya batas waktu
//!   per-request yang bisa diatur plugin, jadi port ini mencoba host satu per
//!   satu secara berurutan (tanpa fan-out paralel) dan berhenti setelah
//!   dadanya. Semua pesan dan urutan tombol sama dengan JS.
//! - `sock.sendButton` dengan header media (`type: "image" | "video"`) tidak
//!   ada di `Reply`; port ini mengirim kartunya tanpa header media.
//! - `config.bot.name` dipakai sebagai footer — sama seperti JS.

use oktz_plugin_api::registry::{CardHeader, InteractiveButton};
use oktz_plugin_api::{Ctx, HttpCall, Reply, plugin};
use serde_json::Value;

/// Batas keras: file di atas ini ditolak sebelum contacting host mana pun.
const MAX_BYTES: usize = 25 * 1024 * 1024;

const UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36";

/// Satu hasil unggahan.
#[derive(Clone)]
struct Hasil {
    host: String,
    url: String,
    expires: String,
}

/// `formatBytes` (`tourl.js:34-39`).
fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        return format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    }
    format!("{} MB", (bytes as f64 / (1024.0 * 1024.0)).round())
}

/// `getFileExtension` (`tourl.js:150-167`).
fn ekstensi(mimetype: &str) -> &'static str {
    match mimetype {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "video/mp4" => "mp4",
        "video/3gpp" => "3gp",
        "video/quicktime" => "mov",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/mp4" => "m4a",
        "application/pdf" => "pdf",
        "application/zip" => "zip",
        _ => "bin",
    }
}

// ---------------------------------------------------------------------------
// Host upload
// ---------------------------------------------------------------------------

fn litterbox(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<Hasil> {
    let res = ctx
        .http
        .call(
            &HttpCall::post("https://litterbox.catbox.moe/resources/internals/api.php")
                .multipart(vec![
                    oktz_plugin_api::registry::MultipartPart::field("reqtype", "fileupload"),
                    oktz_plugin_api::registry::MultipartPart::field("time", "72h"),
                    oktz_plugin_api::registry::MultipartPart::file(
                        "fileToUpload",
                        filename,
                        buffer.to_vec(),
                    ),
                ]),
        )
        .ok()?;
    let url = res.text().trim().to_string();
    if url.starts_with("http") {
        Some(Hasil {
            host: "Litterbox".into(),
            url,
            expires: "72 jam".into(),
        })
    } else {
        None
    }
}

fn kappa(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<Hasil> {
    let res = ctx
        .http
        .call(
            &HttpCall::post("https://kappa.lol/api/upload")
                .header("User-Agent", UA)
                .header("Accept", "*/*")
                .header("Origin", "https://kappa.lol")
                .header("Referer", "https://kappa.lol/")
                .multipart(vec![oktz_plugin_api::registry::MultipartPart::file(
                    "file",
                    filename,
                    buffer.to_vec(),
                )]),
        )
        .ok()?;
    let body = res.json()?;
    let url = body.get("link").and_then(Value::as_str)?;
    Some(Hasil {
        host: "Kappa".into(),
        url: url.to_string(),
        expires: "Permanent".into(),
    })
}

fn uguu(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<Hasil> {
    let res = ctx
        .http
        .call(
            &HttpCall::post("https://uguu.se/upload.php")
                .header("accept", "*/*")
                .header("origin", "https://uguu.se")
                .header("referer", "https://uguu.se/")
                .header("user-agent", UA)
                .multipart(vec![oktz_plugin_api::registry::MultipartPart::file(
                    "files[]",
                    filename,
                    buffer.to_vec(),
                )]),
        )
        .ok()?;
    let body = res.json()?;
    if !body.get("success").is_some_and(crate::common::js_truthy_value) {
        return None;
    }
    let url = body
        .get("files")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|f| f.get("url"))
        .and_then(Value::as_str)?;
    Some(Hasil {
        host: "Uguu".into(),
        url: url.to_string(),
        expires: "48 jam".into(),
    })
}

fn nekohime(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<Hasil> {
    let res = ctx
        .http
        .call(
            &HttpCall::post("https://cdn.nekohime.site/upload").multipart(vec![
                oktz_plugin_api::registry::MultipartPart::file("file", filename, buffer.to_vec()),
            ]),
        )
        .ok()?;
    let body = res.json()?;
    let files = body.get("files")?;
    let url = match files {
        Value::Array(a) => a
            .first()
            .and_then(|f| f.get("url"))
            .and_then(Value::as_str),
        Value::Object(_) => files.get("url").and_then(Value::as_str),
        _ => None,
    }?;
    Some(Hasil {
        host: "Nekohime".into(),
        url: url.to_string(),
        expires: "Permanent".into(),
    })
}

fn leopard(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<Hasil> {
    let page = "https://leopard.hosting.pecon.us/upload.php";
    let _ = ctx.http.call(&HttpCall::get(page).header("User-Agent", UA));
    let res = ctx
        .http
        .call(
            &HttpCall::post(page)
                .header("User-Agent", UA)
                .header("Origin", "https://leopard.hosting.pecon.us")
                .header("Referer", page)
                .multipart(vec![
                    oktz_plugin_api::registry::MultipartPart::file(
                        "uploadContent",
                        filename,
                        buffer.to_vec(),
                    ),
                    oktz_plugin_api::registry::MultipartPart::field("password", ""),
                    oktz_plugin_api::registry::MultipartPart::field("showname", "yes"),
                ]),
        )
        .ok()?;
    let html = res.text();
    let url = ambil_link_leopard(&html)?;
    Some(Hasil {
        host: "Leopard".into(),
        url,
        expires: "Permanent".into(),
    })
}

/// `html.match(/Download link:\s*<a href=([^>\s]+)>/i)?.[1]`.
fn ambil_link_leopard(html: &str) -> Option<String> {
    let idx = html.to_ascii_lowercase().find("download link:")?;
    let rest = &html[idx + "download link:".len()..];
    let rest_lower = rest.to_ascii_lowercase();
    let a = rest_lower.find("<a href=")?;
    let setelah = &rest[a + "<a href=".len()..];
    let akhir = setelah
        .find(|c: char| c == '>' || c.is_whitespace())
        .unwrap_or(setelah.len());
    let url = setelah[..akhir].trim_matches(|c| c == '"' || c == '\'');
    if url.is_empty() {
        None
    } else {
        Some(url.to_string())
    }
}

fn upload_ee(ctx: &Ctx, buffer: &[u8], filename: &str) -> Option<Hasil> {
    let _ = ctx
        .http
        .call(&HttpCall::get("https://www.upload.ee/?").header("User-Agent", UA));
    let rnd = crate::common::now_ms();
    let id_body = ctx
        .http
        .call(
            &HttpCall::get(format!(
                "https://www.upload.ee/ubr_link_upload.php?rnd_id={rnd}"
            ))
            .header("User-Agent", UA)
            .header("Referer", "https://www.upload.ee/?"),
        )
        .ok()?
        .text();
    let upload_id = ambil_start_upload(&id_body)?;

    let ext = filename
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let is_image = matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "svg" | "avif"
    );
    let category = if is_image { "cat_picture" } else { "cat_file" };

    let upload_url = format!(
        "https://www.upload.ee/cgi-bin/ubr_upload.pl?X-Progress-ID={upload_id}&upload_id={upload_id}"
    );
    let _ = ctx.http.call(
        &HttpCall::post(&upload_url)
            .header("User-Agent", UA)
            .header("Origin", "https://www.upload.ee")
            .header("Referer", "https://www.upload.ee/?")
            .multipart(vec![
                oktz_plugin_api::registry::MultipartPart::file(
                    "upfile_0",
                    filename,
                    buffer.to_vec(),
                ),
                oktz_plugin_api::registry::MultipartPart::field("link", ""),
                oktz_plugin_api::registry::MultipartPart::field("email", ""),
                oktz_plugin_api::registry::MultipartPart::field("category", category),
                oktz_plugin_api::registry::MultipartPart::field("big_resize", "none"),
                oktz_plugin_api::registry::MultipartPart::field("small_resize", "120x90"),
            ]),
    );

    let html = ctx
        .http
        .call(
            &HttpCall::get(format!(
                "https://www.upload.ee/?page=finished&upload_id={upload_id}"
            ))
            .header("User-Agent", UA)
            .header("Referer", &upload_url),
        )
        .ok()?
        .text();
    let mut url = ambil_file_src(&html)?;
    url = url.replace("&amp;", "&").replace("&quot;", "\"");
    if is_image {
        url = url.replace("/files/", "/image/");
        if url.ends_with(".html") {
            url.truncate(url.len() - ".html".len());
        }
    }
    Some(Hasil {
        host: "Upload.ee".into(),
        url,
        expires: "Permanent".into(),
    })
}

/// `idBody.match(/startUpload\("([^"]+)"/)?.[1]`.
fn ambil_start_upload(html: &str) -> Option<String> {
    let idx = html.find("startUpload(")?;
    let rest = &html[idx + "startUpload(".len()..];
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// `html.match(/id=["']file_src["'][^>]*value=["']([^"']+)["']/i)?.[1]`.
fn ambil_file_src(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let idx = lower.find("file_src")?;
    let rest = &html[idx..];
    let v = rest.find("value=")? + "value=".len();
    let rest = &rest[v..];
    let q = rest.chars().next()?;
    if q == '"' || q == '\'' {
        let body = &rest[1..];
        let end = body.find(q)?;
        Some(body[..end].to_string())
    } else {
        let end = rest.find(|c: char| c.is_whitespace() || c == '>')?;
        Some(rest[..end].to_string())
    }
}

/// Daftar host — urutan percobaan sama seperti `UPLOAD_PROVIDERS`
/// (`src/lib/upload-providers.js`).
fn unggah_semua(ctx: &Ctx, buffer: &[u8], filename: &str) -> (Vec<Hasil>, Vec<String>) {
    let mut hasil = Vec::new();
    let mut gagal = Vec::new();
    let percobaan: Vec<(&str, Option<Hasil>)> = vec![
        ("Litterbox", litterbox(ctx, buffer, filename)),
        ("Kappa", kappa(ctx, buffer, filename)),
        ("Uguu", uguu(ctx, buffer, filename)),
        ("Nekohime", nekohime(ctx, buffer, filename)),
        ("Leopard", leopard(ctx, buffer, filename)),
        ("Upload.ee", upload_ee(ctx, buffer, filename)),
    ];
    for (nama, r) in percobaan {
        match r {
            Some(h) => hasil.push(h),
            None => gagal.push(nama.to_string()),
        }
    }
    (hasil, gagal)
}

/// `uploadToPaster` (`tourl.js:56-72`).
fn upload_paster(ctx: &Ctx, nama: &str, url_paster: &str, text: &str) -> Option<Hasil> {
    let res = ctx
        .http
        .call(
            &HttpCall::post(url_paster)
                .header("content-type", "text/plain; charset=utf-8")
                .raw(text.as_bytes().to_vec(), "text/plain; charset=utf-8"),
        )
        .ok()?;
    if res.status >= 400 {
        return None;
    }
    let raw = res.text().trim().to_string();
    let url = raw.split('\n').next().unwrap_or("").trim().to_string();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }
    Some(Hasil {
        host: nama.to_string(),
        url,
        expires: "Permanent".into(),
    })
}

/// `resolveText` (`tourl.js:78-86`).
fn resolve_text(ctx: &Ctx) -> String {
    let args = ctx.text.trim();
    if !args.is_empty() {
        return args.to_string();
    }
    ctx.quoted()
        .map(|q| q.text.trim().to_string())
        .unwrap_or_default()
}

fn media_uploader(prefix: &str) -> String {
    format!(
        "📤 *MEDIA UPLOADER* 📤\n\n\
         Halo kak! Butuh link untuk media kamu? Aku bisa bantu uploadin ke berbagai server gratisan loh!\n\n\
         *Cara Pakai:*\n\
         👉 Kirim media dengan caption `{prefix}tourl`\n\
         👉 Atau reply media yang udah ada dengan `{prefix}tourl`\n\
         👉 Kirim teks setelah command (`{prefix}tourl <teks>`) buat dapet link paste"
    )
}

/// Jalur paste teks (`handleText`, `tourl.js:88-148`).
fn handle_text(ctx: &Ctx, text: &str) -> Reply {
    let mut hasil = Vec::new();
    let mut gagal = Vec::new();
    for (nama, url) in [
        ("Paste.rs", "https://paste.rs"),
        ("C-Net", "https://paste.c-net.org"),
    ] {
        match upload_paster(ctx, nama, url, text) {
            Some(h) => hasil.push(h),
            None => gagal.push(nama.to_string()),
        }
    }

    if hasil.is_empty() {
        return Reply::text(format!(
            "❌ Aduh kak, paste-nya gagal semua!\n\n> Gagal di server: {}",
            gagal.join(", ")
        ))
        .with_reaction("❌");
    }

    let mut body = String::from("📝 *TEXT UPLOAD BERHASIL!* 📝\n\n");
    body.push_str("Teks kamu udah di-paste, yuk ambil linknya! ✨\n\n");
    let mut content = String::new();
    for (i, r) in hasil.iter().enumerate() {
        content.push_str(&format!("📋 *Server :* {}\n", r.host));
        content.push_str("⏳ *Expired :* ∞ Permanen\n");
        content.push_str("🔗 *Link :*\n");
        content.push_str(&r.url);
        if i < hasil.len() - 1 {
            content.push_str("\n\n");
        }
    }
    body.push_str(&content);
    if !gagal.is_empty() {
        body.push_str(&format!(
            "\n\n⚠️ _Fyi kak, ada yang gagal di server: {}_",
            gagal.join(", ")
        ));
    }

    let buttons = hasil
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, r)| {
            InteractiveButton::copy(
                format!("📋 Salin Link {}", r.host),
                format!("copy_{i}"),
                r.url.clone(),
            )
        })
        .collect();

    Reply::interactive(body)
        .with_header(CardHeader::new().title("T O U R L"))
        .with_footer(ctx.config().bot.name.clone())
        .with_buttons(buttons)
        .with_reaction("✅")
}

#[plugin(
    name = "tourl",
    alias = ["upload", "url"],
    category = "tools",
    description = "Upload media ke multiple host, atau paste teks dan dapet link",
    usage = ".tourl (reply/kirim media) | .tourl <teks>",
    example = ".tourl",
    cooldown = 10,
    energi = 1,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn tourl(ctx: Ctx) -> Reply {
    let quoted_is_media = ctx.quoted().is_some_and(|q| q.is_media());
    let trigger_is_media = ctx.is_media();

    if !quoted_is_media && !trigger_is_media {
        // Pesan teks: perlakukan sebagai isi paste.
        let paste_text = resolve_text(&ctx);
        if !paste_text.is_empty() {
            return handle_text(&ctx, &paste_text);
        }
        return Reply::text(media_uploader(&ctx.prefix));
    }

    if quoted_is_media && !trigger_is_media {
        let q = ctx.quoted().expect("sudah dicek");
        let mimetype = q
            .mimetype
            .clone()
            .unwrap_or_else(|| "application/octet-stream".to_string());
        let filename = q
            .file_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("file.{}", ekstensi(&mimetype)));
        let Ok(buffer) = ctx.download_quoted_media() else {
            return Reply::text(crate::common::error_template(&ctx));
        };
        return kirim_media(&ctx, buffer, mimetype, filename);
    }

    if !trigger_is_media {
        // Ada quote tapi bukan media, dan pesan pemicu juga bukan media.
        let paste_text = resolve_text(&ctx);
        if !paste_text.is_empty() {
            return handle_text(&ctx, &paste_text);
        }
        return Reply::text("⚠️ Kak, tolong reply ke file (gambar/video/audio/berkas) ya!");
    }

    let mimetype = ctx
        .media
        .as_ref()
        .and_then(|m| m.mimetype.clone())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let filename = ctx
        .media
        .as_ref()
        .and_then(|m| m.file_name.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("file.{}", ekstensi(&mimetype)));
    let Ok(buffer) = ctx.download_media() else {
        return Reply::text(crate::common::error_template(&ctx));
    };
    kirim_media(&ctx, buffer, mimetype, filename)
}

/// Jalur media (`tourl.js:233-315`).
fn kirim_media(ctx: &Ctx, buffer: Vec<u8>, mimetype: String, filename: String) -> Reply {
    if buffer.is_empty() {
        return Reply::text("❌ Waduh kak, medianya nggak kebaca. Coba kirim ulang deh!");
    }
    if buffer.len() > MAX_BYTES {
        return Reply::text(format!(
            "❌ *Kak, filenya kelewat besar!*\n\n\
             > Ukuran: *{}*\n\
             > Batas: *{}*\n\n\
             > Kebanyakan host gratisan nggak kuat file segede itu,\
             jadi percuma dicoba. Compres dulu ya, kak 🙏",
            format_bytes(buffer.len()),
            format_bytes(MAX_BYTES)
        ));
    }

    let (hasil, gagal) = unggah_semua(ctx, &buffer, &filename);
    let not_succeeded = gagal.clone();

    if hasil.is_empty() {
        return Reply::text(format!(
            "❌ Aduh kak, semuanya pada error pas upload!\n\n> Gagal di server: {}",
            not_succeeded.join(", ")
        ))
        .with_reaction("❌");
    }

    let mut text = String::from("🚀 *UPLOAD BERHASIL!* 🚀\n\n");
    text.push_str("Yeay! Media kamu udah berhasil di-upload ke server awan. Silakan pilih linknya dan salin pakai tombol di bawah ya kak! ✨\n\n");
    let mut content = String::new();
    for (i, r) in hasil.iter().enumerate() {
        let status = if r.expires == "Permanent" {
            "∞ Permanen".to_string()
        } else {
            r.expires.clone()
        };
        content.push_str(&format!("☁️ *Server :* {}\n", r.host));
        content.push_str(&format!("⏳ *Expired :* {status}\n"));
        content.push_str("🔗 *Link :*\n");
        content.push_str(&r.url);
        if i < hasil.len() - 1 {
            content.push_str("\n\n");
        }
    }
    text.push_str(&content);
    if !not_succeeded.is_empty() {
        text.push_str(&format!(
            "\n\n⚠️ _Fyi kak, ada yang gagal di server: {}_",
            not_succeeded.join(", ")
        ));
    }

    let buttons = hasil
        .iter()
        .take(5)
        .enumerate()
        .map(|(i, r)| {
            InteractiveButton::copy(
                format!("📋 Salin Link {}", r.host),
                format!("copy_{i}"),
                r.url.clone(),
            )
        })
        .collect();

    let _ = &mimetype;

    Reply::interactive(text)
        .with_header(CardHeader::new().title("T O U R L"))
        .with_footer(ctx.config().bot.name.clone())
        .with_buttons(buttons)
        .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, Reply as R};

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_kosong() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "tourl".into(),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_media_dan_teks() {
        let r = tourl(ctx_kosong());
        assert_eq!(
            body(&r),
            "📤 *MEDIA UPLOADER* 📤\n\n\
             Halo kak! Butuh link untuk media kamu? Aku bisa bantu uploadin ke berbagai server gratisan loh!\n\n\
             *Cara Pakai:*\n\
             👉 Kirim media dengan caption `.tourl`\n\
             👉 Atau reply media yang udah ada dengan `.tourl`\n\
             👉 Kirim teks setelah command (`.tourl <teks>`) buat dapet link paste"
        );
    }

    #[test]
    fn output_sama_dengan_js_file_kelewat_besar() {
        let ctx = ctx_kosong();
        let r = kirim_media(&ctx, vec![0u8; MAX_BYTES + 1], "image/png".into(), "a.png".into());
        assert_eq!(
            body(&r),
            "❌ *Kak, filenya kelewat besar!*\n\n\
             > Ukuran: *25 MB*\n\
             > Batas: *25 MB*\n\n\
             > Kebanyakan host gratisan nggak kuat file segede itu,\
             jadi percuma dicoba. Compres dulu ya, kak 🙏"
        );
    }

    #[test]
    fn format_bytes_sama_dengan_js() {
        assert_eq!(format_bytes(0), "0 MB");
        assert_eq!(format_bytes(1024 * 1024), "1 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn ekstensi_sama_dengan_js() {
        assert_eq!(ekstensi("image/jpeg"), "jpg");
        assert_eq!(ekstensi("application/octet-stream"), "bin");
    }
}
