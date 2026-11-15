//! Unggah buffer gambar ke host publik, lalu kembalikan URL-nya.
//!
//! Dipakai API yang hanya menerima link (`?image=`), bukan multipart —
//! `apiii-xrina.my.id/ai-image/*` salah satunya.
//!
//! Host yang sudah dicek 2026-11-15:
//!   uguu.se  — 200, JSON `{success, files[0].url}`.
//!   kappa    — cadangan kalau uguu menolak.
//! `litterbox.catbox.moe` DIHAPUS dari daftar: memulangkan teks
//! `Invalid uploader`, jadi setiap unggahan gagal di provider pertama.
//!
//! `ponytail:` cuma dua host, tanpa retry bertingkat. Kalau keduanya mati,
//! angkat `upload_image` di `plugin/src/ai/tomekah.rs` (6 host lengkap) ke
//! crate ini — salinan itu sudah ada, jadi tidak perlu ditulis ulang.

use oktz_plugin_api::{HttpCall, HttpGet, MultipartPart};

const UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36";

/// Nama berkas part multipart. Host yang memeriksa ekstensi menolak tanpa ini.
const NAMA_BERKAS: &str = "image.jpg";

/// `mime.lookup("image.jpg")` (`upload-providers.js:19`).
const MIME_BERKAS: &str = "image/jpeg";

fn ok(res: &oktz_plugin_api::HttpResponse) -> bool {
    (200..300).contains(&res.status)
}

/// MIME dari magic number. `/ai-image/nanobanana` memulangkan byte PNG padahal
/// header-nya `image/jpeg`; kirim salah MIME = media ditolak klien.
pub fn mime_gambar(buf: &[u8]) -> &'static str {
    if buf.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if buf.len() > 11 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/jpeg"
    }
}

/// `uguu` (`upload-providers.js:61-82`).
fn uguu(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let res = http
        .call(
            &HttpCall::post("https://uguu.se/upload.php")
                .header("accept", "*/*")
                .header("origin", "https://uguu.se")
                .header("referer", "https://uguu.se/")
                .header("user-agent", UA)
                .multipart(vec![
                    MultipartPart::file("files[]", NAMA_BERKAS, buffer.to_vec())
                        .with_mimetype(MIME_BERKAS),
                ]),
        )
        .map_err(|e| e.message)?;
    if !ok(&res) {
        return Err("Uguu gagal".into());
    }
    let data = res.json().unwrap_or(serde_json::Value::Null);
    data.get("files")
        .and_then(serde_json::Value::as_array)
        .and_then(|files| files.first())
        .and_then(|f| f.get("url"))
        .and_then(serde_json::Value::as_str)
        .filter(|u| !u.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "Invalid response".into())
}

/// `kappa` (`upload-providers.js:39-59`).
fn kappa(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let res = http
        .call(
            &HttpCall::post("https://kappa.lol/api/upload")
                .header("User-Agent", UA)
                .header("Accept", "*/*")
                .header("Origin", "https://kappa.lol")
                .header("Referer", "https://kappa.lol/")
                .multipart(vec![
                    MultipartPart::file("file", NAMA_BERKAS, buffer.to_vec())
                        .with_mimetype(MIME_BERKAS),
                ]),
        )
        .map_err(|e| e.message)?;
    if !ok(&res) {
        return Err("Kappa gagal".into());
    }
    res.json()
        .unwrap_or(serde_json::Value::Null)
        .get("link")
        .and_then(serde_json::Value::as_str)
        .filter(|u| !u.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "Invalid response".into())
}

/// `uploadImage(buffer, "image.jpg")` (`uploader.js`) — provider pertama yang
/// berhasil. Kegagalan dikembalikan sebagai pesan provider terakhir.
pub fn gambar(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    match uguu(http, buffer) {
        Ok(url) => Ok(url),
        Err(_) => kappa(http, buffer),
    }
}


