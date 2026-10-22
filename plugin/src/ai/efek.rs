//! Helper bersama untuk plugin `to*` yang memakai API efek gambar:
//! `uploadImage` (`src/lib/uploader.js` + `src/lib/upload-providers.js`)
//! lalu `applyImageEffect` (`src/lib/effect-api.js`).
//!
//! Dipakai `tofigure`, `tohijab`, dan `tojapanese`. Berkas `tomekah.rs`,
//! `toghibli.rs`, `tofigurev2.rs`, dan `toemotebatu.rs` masih punya salinan
//! sendiri (port sebelumnya) — keduanya menghasilkan output yang sama.
//!
//! `ponytail:` `withTimeout(..., 20000)` per provider tidak bisa disalin ke
//! mesin sinkron ini (tidak ada timer). `UreqHttp` sudah memasang timeout
//! global 20 detik (`ureq_http.rs:24`), kebetulan sama dengan
//! `PROVIDER_TIMEOUT_MS` di JS.

use oktz_plugin_api::{HttpCall, HttpGet, HttpResponse, MultipartPart};
use serde_json::Value;

use crate::tools::phisingataubukan::encode_uri_component;

/// `upload-providers.js:16-17` — UA browser untuk host yang menolaknya.
const UA_UPLOAD: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36";

/// Nama berkas + content-type: JS selalu memanggil
/// `uploadImage(buffer, 'image.jpg')`, dan `mime.lookup('image.jpg')` =
/// `image/jpeg` (`upload-providers.js:19`).
const NAMA_BERKAS: &str = "image.jpg";
const MIME_BERKAS: &str = "image/jpeg";

/// `res.ok` — padanan `if (!res.ok) throw`.
pub(crate) fn respons_ok(res: &HttpResponse) -> bool {
    (200..300).contains(&res.status)
}

/// `typeof url !== 'string' || !url` (`uploader.js:30`).
pub(crate) fn url_dari(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// `litterbox` (`upload-providers.js:21-37`).
pub(crate) fn litterbox(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let res = http
        .call(
            &HttpCall::post("https://litterbox.catbox.moe/resources/internals/api.php").multipart(
                vec![
                    MultipartPart::field("reqtype", "fileupload"),
                    MultipartPart::field("time", "72h"),
                    MultipartPart::file("fileToUpload", NAMA_BERKAS, buffer.to_vec())
                        .with_mimetype(MIME_BERKAS),
                ],
            ),
        )
        .map_err(|e| e.message)?;
    if !respons_ok(&res) {
        return Err("Litterbox gagal".into());
    }
    let url = res.text().trim().to_string();
    if !url.starts_with("http") {
        return Err("Invalid response".into());
    }
    Ok(url)
}

/// `kappa` (`upload-providers.js:39-59`).
pub(crate) fn kappa(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let res = http
        .call(
            &HttpCall::post("https://kappa.lol/api/upload")
                .header("User-Agent", UA_UPLOAD)
                .header("Accept", "*/*")
                .header("Origin", "https://kappa.lol")
                .header("Referer", "https://kappa.lol/")
                .multipart(vec![
                    MultipartPart::file("file", NAMA_BERKAS, buffer.to_vec())
                        .with_mimetype(MIME_BERKAS),
                ]),
        )
        .map_err(|e| e.message)?;
    if !respons_ok(&res) {
        return Err("Kappa gagal".into());
    }
    let data = res.json().unwrap_or(Value::Null);
    url_dari(data.get("link")).ok_or_else(|| "Invalid response".into())
}

/// `uguu` (`upload-providers.js:61-82`).
pub(crate) fn uguu(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let res = http
        .call(
            &HttpCall::post("https://uguu.se/upload.php")
                .header("accept", "*/*")
                .header("origin", "https://uguu.se")
                .header("referer", "https://uguu.se/")
                .header("user-agent", UA_UPLOAD)
                .multipart(vec![
                    MultipartPart::file("files[]", NAMA_BERKAS, buffer.to_vec())
                        .with_mimetype(MIME_BERKAS),
                ]),
        )
        .map_err(|e| e.message)?;
    if !respons_ok(&res) {
        return Err("Uguu gagal".into());
    }
    let data = res.json().unwrap_or(Value::Null);
    let url = data
        .get("files")
        .and_then(Value::as_array)
        .and_then(|files| files.first())
        .and_then(|f| f.get("url"));
    if !crate::common::js_truthy(data.get("success")) {
        return Err("Invalid response".into());
    }
    url_dari(url).ok_or_else(|| "Invalid response".into())
}

/// `nekohime` (`upload-providers.js:84-101`).
pub(crate) fn nekohime(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let res = http
        .call(
            &HttpCall::post("https://cdn.nekohime.site/upload").multipart(vec![
                MultipartPart::file("file", NAMA_BERKAS, buffer.to_vec())
                    .with_mimetype(MIME_BERKAS),
            ]),
        )
        .map_err(|e| e.message)?;
    if !respons_ok(&res) {
        return Err("Nekohime gagal".into());
    }
    let data = res.json().unwrap_or(Value::Null);
    // `Array.isArray(f) ? f[0]?.url : f?.url` (`upload-providers.js:97-98`).
    let url = match data.get("files") {
        Some(Value::Array(list)) => list.first().and_then(|f| f.get("url")),
        Some(f) => f.get("url"),
        None => None,
    };
    url_dari(url).ok_or_else(|| "Nekohime gagal".into())
}

/// `\s*` — jumlah byte spasi di awal teks (dipakai scanner regex di bawah).
pub(crate) fn lewati_spasi(t: &str) -> usize {
    t.len() - t.trim_start().len()
}

/// Regex `Download link:\s*<a href=([^>\s]+)>` (case-insensitive,
/// `upload-providers.js:124`).
pub(crate) fn tautan_leopard(html: &str) -> Option<String> {
    let turun = html.to_ascii_lowercase();
    let mut dari = 0;
    while let Some(rel) = turun[dari..].find("download link:") {
        let mut i = dari + rel + "download link:".len();
        dari = i;
        i += lewati_spasi(&turun[i..]);
        if !turun[i..].starts_with("<a href=") {
            continue;
        }
        let sisa = &html[i + "<a href=".len()..];
        let akhir = sisa.find(|c: char| c == '>' || c.is_whitespace());
        if let Some(akhir) = akhir.filter(|n| *n > 0) {
            return Some(sisa[..akhir].to_string());
        }
    }
    None
}

/// `leopard` (`upload-providers.js:103-127`).
pub(crate) fn leopard(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    const HALAMAN: &str = "https://leopard.hosting.pecon.us/upload.php";
    // Pemanasan: status tidak diperiksa, tapi kegagalan jaringan tetap
    // menggagalkan provider ini (`await fetch(...)` tanpa catch di JS).
    // `terima_semua_status` = `fetch` JS yang tidak melempar untuk 4xx/5xx.
    http.call(
        &HttpCall::get(HALAMAN)
            .header("User-Agent", UA_UPLOAD)
            .terima_semua_status(),
    )
    .map_err(|e| e.message)?;
    let html = http
        .call(
            &HttpCall::post(HALAMAN)
                .header("User-Agent", UA_UPLOAD)
                .header("Origin", "https://leopard.hosting.pecon.us")
                .header("Referer", HALAMAN)
                .multipart(vec![
                    MultipartPart::file("uploadContent", NAMA_BERKAS, buffer.to_vec())
                        .with_mimetype(MIME_BERKAS),
                    MultipartPart::field("password", ""),
                    MultipartPart::field("showname", "yes"),
                ])
                .terima_semua_status(),
        )
        .map_err(|e| e.message)?
        .text();
    tautan_leopard(&html).ok_or_else(|| "Leopard gagal".into())
}

/// `/startUpload\("([^"]+)"/` (`upload-providers.js:142`).
pub(crate) fn start_upload(body: &str) -> Option<String> {
    let mut dari = 0;
    while let Some(rel) = body[dari..].find("startUpload(\"") {
        let mulai = dari + rel + "startUpload(\"".len();
        dari = mulai;
        let sisa = &body[mulai..];
        if let Some(akhir) = sisa.find('"').filter(|n| *n > 0) {
            return Some(sisa[..akhir].to_string());
        }
    }
    None
}

/// Regex `id=["']file_src["'][^>]*value=["']([^"']+)["']` (case-insensitive,
/// `upload-providers.js:172`) versi scanner.
pub(crate) fn file_src_upload_ee(html: &str) -> Option<String> {
    let turun = html.to_ascii_lowercase();
    let mut dari = 0;
    while let Some(rel) = turun[dari..].find("id=") {
        let mulai = dari + rel + 3;
        dari = mulai;
        let sisa = &turun[mulai..];
        let Some(sisa) = sisa.strip_prefix('"').or_else(|| sisa.strip_prefix('\'')) else {
            continue;
        };
        let Some(sisa) = sisa.strip_prefix("file_src") else {
            continue;
        };
        if !sisa.starts_with('"') && !sisa.starts_with('\'') {
            continue;
        }
        // `[^>]*` — `value=` yang dicari terbatas sebelum `>` pertama.
        let setelah = mulai + 1 + "file_src".len() + 1;
        let batas = turun[setelah..]
            .find('>')
            .map_or(turun.len(), |p| setelah + p);
        let area = &turun[setelah..batas];
        let mut cari = 0;
        while let Some(rel2) = area[cari..].find("value=") {
            let vk = cari + rel2 + "value=".len();
            let sisa_kutip = &html[setelah + vk..];
            let isi = if sisa_kutip.starts_with('"') || sisa_kutip.starts_with('\'') {
                &sisa_kutip[1..]
            } else {
                cari = vk;
                continue;
            };
            if let Some(akhir) = isi.find(['"', '\'']).filter(|n| *n > 0) {
                return Some(isi[..akhir].to_string());
            }
            cari = vk;
        }
    }
    None
}

/// Regex `View file:\s*<br\s*\/?>\s*<a href=["']?([^"'>\s]+)["']?`
/// (case-insensitive, `upload-providers.js:173`) versi scanner.
pub(crate) fn view_file_upload_ee(html: &str) -> Option<String> {
    let turun = html.to_ascii_lowercase();
    let mut dari = 0;
    while let Some(rel) = turun[dari..].find("view file:") {
        let mut i = dari + rel + "view file:".len();
        dari = i;
        i += lewati_spasi(&turun[i..]);
        if !turun[i..].starts_with("<br") {
            continue;
        }
        i += 3;
        i += lewati_spasi(&turun[i..]);
        if turun[i..].starts_with('/') {
            i += 1;
        }
        if !turun[i..].starts_with('>') {
            continue;
        }
        i += 1;
        i += lewati_spasi(&turun[i..]);
        if !turun[i..].starts_with("<a href=") {
            continue;
        }
        i += "<a href=".len();
        let sisa = &html[i..];
        let sisa = if sisa.starts_with('"') || sisa.starts_with('\'') {
            &sisa[1..]
        } else {
            sisa
        };
        let akhir = sisa.find(|c: char| c == '"' || c == '\'' || c == '>' || c.is_whitespace());
        if let Some(akhir) = akhir.filter(|n| *n > 0) {
            return Some(sisa[..akhir].to_string());
        }
    }
    None
}

/// `uploadEe` (`upload-providers.js:129-179`).
pub(crate) fn upload_ee(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    // `ext` (`upload-providers.js:130-133`) ditulis apa adanya, walau dengan
    // nama berkas tetap "image.jpg" cabang `cat_file` tidak pernah tercapai.
    let ext = NAMA_BERKAS
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let is_image = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg", "avif"].contains(&ext.as_str());

    // Pemanasan GET — JS tidak memeriksa status, hanya gagal jaringan.
    // `terima_semua_status` = `fetch` JS yang tidak melempar untuk 4xx/5xx.
    http.call(
        &HttpCall::get("https://www.upload.ee/?")
            .header("User-Agent", UA_UPLOAD)
            .terima_semua_status(),
    )
    .map_err(|e| e.message)?;
    let id_body = http
        .call(
            &HttpCall::get(format!(
                "https://www.upload.ee/ubr_link_upload.php?rnd_id={}",
                crate::common::now_ms()
            ))
            .header("User-Agent", UA_UPLOAD)
            .header("Referer", "https://www.upload.ee/?")
            .terima_semua_status(),
        )
        .map_err(|e| e.message)?
        .text();
    let upload_id = start_upload(&id_body).ok_or("Upload ID tidak ditemukan")?;

    let upload_url = format!(
        "https://www.upload.ee/cgi-bin/ubr_upload.pl?X-Progress-ID={upload_id}&upload_id={upload_id}"
    );
    http.call(
        &HttpCall::post(&upload_url)
            .header("User-Agent", UA_UPLOAD)
            .header("Origin", "https://www.upload.ee")
            .header("Referer", "https://www.upload.ee/?")
            .multipart(vec![
                MultipartPart::file("upfile_0", NAMA_BERKAS, buffer.to_vec())
                    .with_mimetype(MIME_BERKAS),
                MultipartPart::field("link", ""),
                MultipartPart::field("email", ""),
                MultipartPart::field(
                    "category",
                    if is_image { "cat_picture" } else { "cat_file" },
                ),
                MultipartPart::field("big_resize", "none"),
                MultipartPart::field("small_resize", "120x90"),
            ])
            .terima_semua_status(),
    )
    .map_err(|e| e.message)?;

    let html = http
        .call(
            &HttpCall::get(format!(
                "https://www.upload.ee/?page=finished&upload_id={upload_id}"
            ))
            .header("User-Agent", UA_UPLOAD)
            .header("Referer", upload_url.as_str())
            .terima_semua_status(),
        )
        .map_err(|e| e.message)?
        .text();
    let raw = file_src_upload_ee(&html)
        .or_else(|| view_file_upload_ee(&html))
        .ok_or("Upload.ee gagal")?;

    // `replaceAll("&amp;", "&")` lalu `replaceAll("&quot;", "\"")`.
    let mut url = raw.replace("&amp;", "&").replace("&quot;", "\"");
    if is_image {
        // `String.replace("/files/", …)` JS mengganti kemunculan PERTAMA.
        url = url.replacen("/files/", "/image/", 1);
        if let Some(tanpa_html) = url.strip_suffix(".html") {
            url = tanpa_html.to_string();
        }
    }
    Ok(url)
}

/// `uploadImage(buffer, 'image.jpg')` (`uploader.js:14-40`) — provider
/// dicoba berurutan sampai ada yang mengembalikan URL.
pub(crate) fn upload_image(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    // Urutan `PROVIDERS` (`upload-providers.js:183-190`).
    // Alias lokal: pecah tipe kompleks pasangan nama + fn provider.
    type ProviderUnggah = (&'static str, fn(&dyn HttpGet, &[u8]) -> Result<String, String>);
    let providers: [ProviderUnggah; 6] = [
        ("Litterbox", litterbox),
        ("Kappa", kappa),
        ("Uguu", uguu),
        ("Nekohime", nekohime),
        ("Leopard", leopard),
        ("Upload.ee", upload_ee),
    ];
    let mut kegagalan: Vec<String> = Vec::new();
    for (nama, jalankan) in providers {
        match jalankan(http, buffer) {
            Ok(url) => return Ok(url),
            Err(pesan) => kegagalan.push(format!("{nama}: {pesan}")),
        }
    }
    Err(format!(
        "Upload gagal di semua host — {}",
        kegagalan.join(" | ")
    ))
}

/// UA request efek (`effect-api.js:20`).
const UA_EFEK: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36";

/// Signature PNG/JPEG/GIF/WEBP/RIFF/BMP (`effect-api.js:57-67`).
pub(crate) fn looks_like_image(buf: &[u8]) -> bool {
    if buf.len() < 12 {
        return false;
    }
    buf.starts_with(&[0x89, 0x50, 0x4e, 0x47])
        || buf.starts_with(&[0xff, 0xd8, 0xff])
        || buf.starts_with(b"GIF8")
        || buf.starts_with(b"RIFF")
        || buf.starts_with(b"BM")
}

/// `applyImageEffect(effect, imageUrl)` — daftar provider dari
/// `config.effectApi.providers` tidak ada di `oktz_core::config::Runtime`, jadi
/// hanya host fallback `faa` (`effect-api.js:26-28`) yang dipakai.
///
/// `ponytail:` `withTimeout(..., 45000)` dan loop provider
/// (`effect-api.js:106-120`) tidak bisa disalin tanpa timer + config; ekspos
/// `effectApi` di `Runtime` kalau host cadangan mau dipakai.
pub(crate) fn apply_image_effect(http: &dyn HttpGet, effect: &str, image_url: &str) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://api-faa.my.id/faa/{}?url={}",
        encode_uri_component(effect),
        encode_uri_component(image_url)
    );
    let res = http
        .call(
            &HttpCall::get(&url)
                .header("user-agent", UA_EFEK)
                .header("accept", "image/*,*/*"),
        )
        .map_err(|e| e.message)?;
    if !respons_ok(&res) {
        return Err(format!("faa: HTTP {} (server error)", res.status));
    }
    let buf = res.body.bytes().to_vec();
    if !looks_like_image(&buf) {
        return Err("faa: host membalas HTML, bukan gambar".into());
    }
    Ok(buf)
}


#[cfg(test)]
mod tests {
    use super::*;

    /// JPEG 12 byte — lolos `looks_like_image`.
    const GAMBAR: &[u8] = b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01";

    #[test]
    fn signature_gambar_dikenali() {
        assert!(looks_like_image(GAMBAR));
        assert!(looks_like_image(b"\x89PNG\r\n\x1a\n\x00\x00\x00\x00"));
        assert!(looks_like_image(b"GIF89a\x00\x00\x00\x00\x00\x00"));
        assert!(looks_like_image(b"RIFF\x00\x00\x00\x00WEBP"));
        assert!(!looks_like_image(b"<html>bukan gambar</html>"));
        assert!(!looks_like_image(b"pendek"));
    }

    #[test]
    fn scanner_tautan_leopard_dan_upload_ee() {
        assert_eq!(
            tautan_leopard("Download link: <a href=https://leopard.example/x.png>").as_deref(),
            Some("https://leopard.example/x.png")
        );
        assert_eq!(
            tautan_leopard("download LINK:\n<a href=https://x/y>").as_deref(),
            Some("https://x/y")
        );
        assert_eq!(
            file_src_upload_ee(r#"<input id="file_src" name="x" value="https://upload.ee/a.html">"#)
                .as_deref(),
            Some("https://upload.ee/a.html")
        );
        assert_eq!(
            view_file_upload_ee("View file: <br /><a href=\"https://upload.ee/b.html\">").as_deref(),
            Some("https://upload.ee/b.html")
        );
        assert_eq!(
            start_upload(r#"x.startUpload("abc123", 1);"#).as_deref(),
            Some("abc123")
        );
    }
}
