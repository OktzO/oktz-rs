//! `plugins/canvas/gura.js` — efek canvas Gura dari foto lewat API Nexray.
//!
//! Port penuh: gambar (reply/kirim) → unggah lewat rantai provider
//! `src/lib/uploader.js` → `GET api.nexray.eu.cc/canvas/gura?url=...` → kirim
//! gambar hasil (`gura.js:23-61`).
//!
//! Penyimpangan yang tercatat apa adanya:
//!
//! - **Reaksi `🕕` dibuang** (`gura.js:42`). `Reply` hanya bisa membawa satu
//!   reaksi, jadi yang ikut adalah reaksi penutup (`✅`/`☢`).
//! - **`m.pushName` dipakai apa adanya** — `Ctx` punya `push_name`.
//!
//! `ponytail:` blok SALINAN `uploadImage` sama persis dengan
//! `canvas/profileig.rs` dan empat berkas `ai/*`; angkat ke `crate::common`
//! kalau plugin lain turut memakainya.

use oktz_plugin_api::{
    Ctx, HttpCall, HttpGet, HttpResponse, IncomingKind, MediaSource, MultipartPart, Reply, plugin,
};
use serde_json::Value;

use crate::tools::phisingataubukan::encode_uri_component;

// ===========================================================================
// SALINAN `src/lib/uploader.js` + `src/lib/upload-providers.js`
// (identik dengan blok di `canvas/profileig.rs` — lihat catatan di sana).
// ===========================================================================

/// `upload-providers.js:16-17` — UA browser untuk host yang menolaknya.
const UA_UPLOAD: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36";

const NAMA_BERKAS: &str = "image.jpg";
const MIME_BERKAS: &str = "image/jpeg";

/// `res.ok` — padanan `if (!res.ok) throw`.
fn respons_ok(res: &HttpResponse) -> bool {
    (200..300).contains(&res.status)
}

fn url_dari(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// `litterbox` (`upload-providers.js:21-37`).
fn litterbox(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
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
fn kappa(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
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
fn uguu(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
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
fn nekohime(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
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
    let url = match data.get("files") {
        Some(Value::Array(list)) => list.first().and_then(|f| f.get("url")),
        Some(f) => f.get("url"),
        None => None,
    };
    url_dari(url).ok_or_else(|| "Nekohime gagal".into())
}

fn lewati_spasi(t: &str) -> usize {
    t.len() - t.trim_start().len()
}

/// Regex `Download link:\s*<a href=([^>\s]+)>` (case-insensitive) versi scanner.
fn tautan_leopard(html: &str) -> Option<String> {
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
fn leopard(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    const HALAMAN: &str = "https://leopard.hosting.pecon.us/upload.php";
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

/// `/startUpload\("([^"]+)"/` versi scanner.
fn start_upload(body: &str) -> Option<String> {
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

/// Regex `id=["']file_src["'][^>]*value=["']([^"']+)["']` (case-insensitive)
/// versi scanner.
fn file_src_upload_ee(html: &str) -> Option<String> {
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

/// Regex `View file:\s*<br\s*\/?>\s*<a href=["']?([^"'>\s]+)["']?` versi scanner.
fn view_file_upload_ee(html: &str) -> Option<String> {
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
fn upload_ee(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let ext = NAMA_BERKAS
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    let is_image = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg", "avif"].contains(&ext.as_str());

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

    let mut url = raw.replace("&amp;", "&").replace("&quot;", "\"");
    if is_image {
        url = url.replacen("/files/", "/image/", 1);
        if let Some(tanpa_html) = url.strip_suffix(".html") {
            url = tanpa_html.to_string();
        }
    }
    Ok(url)
}

/// `uploadImage(buffer, 'image.jpg')` (`uploader.js:14-40`).
type Unggah = fn(&dyn HttpGet, &[u8]) -> Result<String, String>;

pub(crate) fn upload_image(http: &dyn HttpGet, buffer: &[u8]) -> Result<String, String> {
    let providers: [(&str, Unggah); 6] = [
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

// ===========================================================================
// Plugin
// ===========================================================================

/// `gura.js:29` — kartu untuk pesan pemicu bukan gambar.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🦈 *GURA CANVAS*\n\nKirim atau reply foto dengan perintah `{prefix}gura` untuk memberikan efek Gura!"
    )
}

/// `gura.js:47` — URL API Nexray.
pub(crate) fn api_url(img_url: &str) -> String {
    format!("https://api.nexray.eu.cc/canvas/gura?url={}", encode_uri_component(img_url))
}

/// `gura.js:54` — caption sukses.
pub(crate) const CAPTION: &str = "🦈 *RAWWRR! Gura is here!*";

/// `gura.js:24-38` — media: quoted HARUS gambar (`⚠️`), pemicu HARUS gambar
/// (`help`), lalu unduh; unduhan/keberadaan kosong → `gura.js:40`.
pub(crate) enum Media {
    Bytes(Vec<u8>),
    BukanQuotedGambar,
    BukanGambar,
    TidakAda,
}

pub(crate) fn ambil_media(ctx: &Ctx) -> Media {
    if let Some(q) = &ctx.quoted {
        if !q.kind.is_image() {
            return Media::BukanQuotedGambar;
        }
        return match ctx.download_quoted_media() {
            Ok(b) => Media::Bytes(b),
            Err(_) => Media::TidakAda,
        };
    }
    if ctx.media.is_some() && !ctx.is_media_kind(IncomingKind::Image) {
        return Media::BukanGambar;
    }
    match ctx.download_media() {
        Ok(b) => Media::Bytes(b),
        Err(_) => Media::TidakAda,
    }
}

/// `gura.js:89` — kartu catch: template error + reaksi `☢`.
pub(crate) fn gagal(ctx: &Ctx) -> Reply {
    Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
}

#[plugin(
    name = "gura",
    alias = ["guracanvas"],
    category = "canvas",
    description = "Bikin efek canvas gura dari fotomu",
    usage = ".gura (reply/kirim foto)",
    example = ".gura",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn gura(ctx: Ctx) -> Reply {
    let buffer = match ambil_media(&ctx) {
        Media::Bytes(b) => b,
        Media::BukanQuotedGambar => {
            return Reply::text("⚠️ Kak, tolong reply ke pesan gambar ya!");
        }
        Media::BukanGambar => return Reply::text(help(&ctx.prefix)),
        Media::TidakAda => return Reply::text("❌ Gagal membaca media, coba lagi!"),
    };

    // `try { uploadImage; fetch(apiUrl); send ✅ } catch { ☢ + te }`
    // (`gura.js:44-60`). Reaksi `🕕` (`gura.js:42`) hilang — satu reaksi saja.
    let Some(img_url) = upload_image(&*ctx.http, &buffer).ok() else {
        return gagal(&ctx);
    };

    let Ok(res) = ctx.http.call(&HttpCall::get(&api_url(&img_url))) else {
        return gagal(&ctx);
    };
    // `if (!res.ok) throw` (`gura.js:50`).
    if !respons_ok(&res) {
        return gagal(&ctx);
    }

    Reply::image(
        MediaSource::bytes(res.body.bytes().to_vec()),
        CAPTION,
    )
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpError, IncomingMedia, MediaError, MediaFetch, MediaOrigin, QuotedKind,
        QuotedMessage,
    };
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";
    const GAMBAR: &[u8] = b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01";
    const HASIL_API: &[u8] = b"GURA-CANVAS";

    fn resp(status: u16, isi: impl AsRef<[u8]>) -> (u16, Vec<u8>) {
        (status, isi.as_ref().to_vec())
    }

    /// HTTP berurutan: unggahan (8 panggilan maksimum) lalu API.
    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<(u16, Vec<u8>)>>,
        hitung: Mutex<usize>,
        panggilan: Mutex<Vec<HttpCall>>,
        unduhan: Mutex<Vec<MediaOrigin>>,
        hasil_unduh: Result<Vec<u8>, MediaError>,
    }

    impl Stub {
        fn baru(balasan: Vec<(u16, Vec<u8>)>, hasil_unduh: Result<Vec<u8>, MediaError>) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(balasan),
                hitung: Mutex::new(0),
                panggilan: Mutex::new(Vec::new()),
                unduhan: Mutex::new(Vec::new()),
                hasil_unduh,
            })
        }
    }

    impl HttpGet for Stub {
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.panggilan.lock().unwrap().push(req.clone());
            let i = {
                let mut hitung = self.hitung.lock().unwrap();
                let i = *hitung;
                *hitung += 1;
                i
            };
            match self.balasan.lock().unwrap().get(i).cloned() {
                Some((status, isi)) => Ok(HttpResponse {
                    status,
                    body: HttpBody::Bytes {
                        data: Arc::from(isi),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_multipart(&self, _: &str, _: &[(&str, HttpBody)]) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
    }

    impl MediaFetch for Stub {
        fn download(&self, origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.unduhan.lock().unwrap().push(origin);
            self.hasil_unduh.clone()
        }
    }

    fn media_gambar() -> Option<IncomingMedia> {
        Some(IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        })
    }

    fn ctx_stub(http: Arc<Stub>, text: &str, media: Option<IncomingMedia>, quoted: Option<QuotedKind>) -> Ctx {
        Ctx {
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "gura".into(),
            media,
            quoted: quoted.map(|kind| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: ME.into(),
                text: String::new(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
                timestamp: 0,
            }),
            http: http.clone(),
            media_fetch: http,
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    fn isi(r: &Reply) -> &Reply {
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        }
    }

    fn bytes_gambar(r: &Reply) -> Vec<u8> {
        match isi(r) {
            Reply::Image { media: Some(MediaSource::Bytes { data, .. }), .. } => data.to_vec(),
            lain => panic!("harus image bytes, dapat {lain:?}"),
        }
    }

    #[test]
    fn meta_dan_alias_sesuai_js() {
        let p = oktz_plugin_api::find("gura").unwrap();
        assert_eq!(p.meta.name, "gura");
        assert_eq!(p.meta.alias, ["guracanvas"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Bikin efek canvas gura dari fotomu");
        assert_eq!(p.meta.usage, ".gura (reply/kirim foto)");
        assert_eq!(p.meta.example, ".gura");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        for k in ["gura", "guracanvas"] {
            assert_eq!(oktz_plugin_api::find(k).unwrap().meta.name, "gura");
        }
    }

    #[test]
    fn quoted_bukan_gambar_membalas_peringatan() {
        let http = Stub::baru(vec![], Ok(Vec::new()));
        let r = gura(ctx_stub(http, "", None, Some(QuotedKind::Text)));
        assert_eq!(teks(&r), "⚠️ Kak, tolong reply ke pesan gambar ya!");
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn pemicu_bukan_gambar_membalas_help() {
        let http = Stub::baru(vec![], Ok(Vec::new()));
        let r = gura(ctx_stub(
            http,
            "",
            Some(IncomingMedia {
                kind: IncomingKind::Video,
                mimetype: Some("video/mp4".into()),
                file_name: None,
                caption: String::new(),
            }),
            None,
        ));
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn unduhan_gagal_membalas_gagal_membaca_media() {
        let http = Stub::baru(vec![], Err(MediaError { message: "gagal".into() }));
        let r = gura(ctx_stub(http, "", media_gambar(), None));
        assert_eq!(teks(&r), "❌ Gagal membaca media, coba lagi!");
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn alur_sukses_unggah_lalu_api_lalu_gambar() {
        let http = Stub::baru(
            vec![
                resp(200, b"https://litterbox.example/img.jpg"),
                resp(200, HASIL_API),
            ],
            Ok(GAMBAR.to_vec()),
        );
        let r = gura(ctx_stub(http.clone(), "", media_gambar(), None));
        assert_eq!(bytes_gambar(&r), HASIL_API);
        assert_eq!(reaksi(&r), Some("✅".into()));
        let panggilan = http.panggilan.lock().unwrap();
        assert_eq!(panggilan.len(), 2);
        assert_eq!(panggilan[0].url, "https://litterbox.catbox.moe/resources/internals/api.php");
        assert_eq!(panggilan[1].url, api_url("https://litterbox.example/img.jpg"));
        assert_eq!(*http.unduhan.lock().unwrap(), [MediaOrigin::Trigger]);
    }

    #[test]
    fn quoted_gambar_diunduh_dari_quoted() {
        let http = Stub::baru(
            vec![
                resp(200, b"https://litterbox.example/img.jpg"),
                resp(200, HASIL_API),
            ],
            Ok(GAMBAR.to_vec()),
        );
        let r = gura(ctx_stub(http, "", None, Some(QuotedKind::Image)));
        assert_eq!(reaksi(&r), Some("✅".into()));
    }

    #[test]
    fn unggahan_gagal_total_membalas_template_error_dan_radiasi() {
        // 8 panggilan unggahan semuanya gagal → rantai provider habis.
        let balasan: Vec<(u16, Vec<u8>)> = (0..8).map(|_| resp(500, b"")).collect();
        let http = Stub::baru(balasan, Ok(GAMBAR.to_vec()));
        let r = gura(ctx_stub(http, "", media_gambar(), None));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "gura"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn api_non_2xx_membalas_template_error_dan_radiasi() {
        let http = Stub::baru(
            vec![resp(200, b"https://litterbox.example/img.jpg"), resp(500, b"")],
            Ok(GAMBAR.to_vec()),
        );
        let r = gura(ctx_stub(http, "", media_gambar(), None));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "gura"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn api_url_mengenkode_parameter_url() {
        assert_eq!(
            api_url("https://a.example/b c.jpg"),
            "https://api.nexray.eu.cc/canvas/gura?url=https%3A%2F%2Fa.example%2Fb%20c.jpg"
        );
    }
}
