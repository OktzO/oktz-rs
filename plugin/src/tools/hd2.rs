//! `plugins/tools/hd2.js` — enhance gambar menjadi HD dengan AI (V3).
//!
//! ## Penyimpangan
//!
//! - JS memakai `resolver.resolve("hd", { kind: "imglarger", media })`
//!   (`src/capabilities/hd.js`), yang mencoba dua backend: `photoai.imglarger.com`
//!   (lokal) lalu agregator `my.izuka-api.xyz` (jaringan). Port ini memakai
//!   scraper `oktz_scraper::hd` (backend lokal) dan, kalau gagal, memanggil
//!   agregator secara langsung — urutan dan bentuk respons sama.
//! - `jpegThumbnail` pada pesan dokumen tidak punya field di `Reply::Document`,
//!   jadi thumbnail 50×50 yang dibuat JS tidak ikut. Nama berkas dan caption
//!   tetap sama.

use oktz_plugin_api::registry::MultipartPart;
use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

const IZUKA: &str = "https://my.izuka-api.xyz/api/tools/imglarger";

fn cara_pakai(prefix: &str) -> String {
    format!(
        "✨ *FITUR HD ENHANCE V2*\n\n\
         Tingkatkan resolusi gambar kamu menjadi jauh lebih HD dan tajam menggunakan AI!\n\n\
         *Cara Penggunaan:*\n\
         - Kirim gambar dan tambahkan pesan *{prefix}hd2*\n\
         - Atau balas (reply) gambar yang sudah terkirim dengan perintah *{prefix}hd2*\n\n\
         _Proses rendering mungkin memerlukan waktu beberapa detik hingga satu menit._"
    )
}

fn gagal_unduh() -> &'static str {
    "Maaf, sistem gagal mengunduh gambar yang kamu berikan. Silakan coba kirim ulang gambarnya!"
}

fn gagal_proses() -> &'static str {
    "Maaf, AI gagal memproses gambarmu kali ini. Silakan coba lagi dalam beberapa saat!"
}

/// Kandidat URL hasil — `kandidat()` (`src/capabilities/hd.js`).
pub(crate) fn kandidat(isi: &Value, keluar: &mut Vec<String>) {
    if let Some(s) = isi.as_str() {
        keluar.push(s.to_string());
        return;
    }
    let Some(obj) = isi.as_object() else {
        return;
    };
    for kunci in ["url", "img_url", "download_url", "res_url"] {
        if let Some(Value::String(s)) = obj.get(kunci) {
            keluar.push(s.clone());
        }
    }
    for kunci in ["downloadUrls", "output_url"] {
        if let Some(Value::Array(a)) = obj.get(kunci)
            && let Some(Value::String(s)) = a.first()
        {
            keluar.push(s.clone());
        }
    }
}

/// `urlHasil()` (`src/capabilities/hd.js`) — satu level amplop lebih dulu.
pub(crate) fn url_hasil(bentuk: &Value) -> String {
    let mut keluar = Vec::new();
    kandidat(bentuk, &mut keluar);
    if let Some(r) = bentuk.get("result") {
        kandidat(r, &mut keluar);
    }
    if let Some(d) = bentuk.get("data") {
        kandidat(d, &mut keluar);
    }
    for url in keluar {
        let url = url.trim();
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }
    }
    String::new()
}

/// Backend lokal — `photoai.imglarger.com` (`src/scraper/hd.js`).
fn lewat_photoai(ctx: &Ctx, path: &str) -> Option<String> {
    let unggahan = oktz_scraper::hd::upload(ctx.http.as_ref(), path).ok()?;
    let code = unggahan.get("code").and_then(Value::as_str)?.trim().to_string();
    if code.is_empty() {
        return None;
    }
    let hasil = oktz_scraper::hd::get(ctx.http.as_ref(), &code).ok()?;
    let url = url_hasil(&hasil);
    if url.is_empty() { None } else { Some(url) }
}

/// Backend agregator — `my.izuka-api.xyz` (`src/capabilities/hd.js`).
fn lewat_izuka(ctx: &Ctx, gambar: &[u8]) -> Option<String> {
    let req = HttpCall::post(IZUKA).multipart(vec![
        MultipartPart::file("image", "image.jpg", gambar.to_vec()).with_mimetype("image/jpeg"),
        MultipartPart::field("type", "upscale"),
        MultipartPart::field("scale", "2"),
    ]);
    let res = ctx.http.call(&req).ok()?;
    if res.status >= 400 {
        return None;
    }
    let body = res.json()?;
    if body.get("status").and_then(Value::as_bool) == Some(false) {
        return None;
    }
    let url = url_hasil(&body);
    if url.is_empty() { None } else { Some(url) }
}

#[plugin(
    name = "hd2",
    alias = ["enhance2", "upscale2", "aienhancer"],
    category = "tools",
    description = "Enhance gambar menjadi HD dengan AI (V3)",
    usage = ".hd2 (reply gambar)",
    example = ".hd2",
    cooldown = 30,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn hd2(ctx: Ctx) -> Reply {
    let is_image = ctx
        .media
        .as_ref()
        .is_some_and(|m| m.kind == oktz_plugin_api::IncomingKind::Image)
        || ctx.quoted().is_some_and(|q| q.kind.is_image());

    if !is_image {
        return Reply::text(cara_pakai(&ctx.prefix));
    }

    let quoted_is_media = ctx.quoted().is_some_and(|q| q.is_media());
    let bytes = if quoted_is_media {
        ctx.download_quoted_media()
    } else {
        ctx.download_media()
    };
    let bytes = match bytes {
        Ok(b) if !b.is_empty() => b,
        _ => {
            return Reply::text(gagal_unduh()).with_reaction("❌");
        }
    };

    // Backend lokal butuh path file di disk (`src/capabilities/hd.js:41-53`).
    let temp_dir = "tmp";
    let _ = ctx.berkas().buat_folder(temp_dir);
    let path = format!("{temp_dir}/hd-{}.jpg", crate::common::now_ms());
    if ctx.berkas().tulis(&path, &bytes).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    let url = lewat_photoai(&ctx, &path).or_else(|| lewat_izuka(&ctx, &bytes));
    let _ = ctx.berkas().hapus(&path);

    let Some(url) = url else {
        return Reply::text(gagal_proses()).with_reaction("❌");
    };

    // Thumbnail 50×50Quality 30 — tidak bisa dikirim lewat `Reply::Document`.
    let _ = oktz_plugin_api::gambar::sharp(&bytes)
        .map(|g| g.ubah_ukuran(50, 50))
        .and_then(|g| g.ke_jpeg(30));

    let nama_berkas = format!("HD_BY_{}.jpg", ctx.config().bot.name);
    let media = MediaSource::url(url).with_mimetype("image/jpeg");

    Reply::document(media, nama_berkas).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, QuotedKind, QuotedMessage, Reply as R};
    use serde_json::json;

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_kosong() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "hd2".into(),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_bukan_gambar() {
        let r = hd2(ctx_kosong());
        assert_eq!(
            body(&r),
            "✨ *FITUR HD ENHANCE V2*\n\n\
             Tingkatkan resolusi gambar kamu menjadi jauh lebih HD dan tajam menggunakan AI!\n\n\
             *Cara Penggunaan:*\n\
             - Kirim gambar dan tambahkan pesan *.hd2*\n\
             - Atau balas (reply) gambar yang sudah terkirim dengan perintah *.hd2*\n\n\
             _Proses rendering mungkin memerlukan waktu beberapa detik hingga satu menit._"
        );
    }

    #[test]
    fn output_sama_dengan_js_gagal_unduh() {
        assert_eq!(
            gagal_unduh(),
            "Maaf, sistem gagal mengunduh gambar yang kamu berikan. Silakan coba kirim ulang gambarnya!"
        );
        assert_eq!(
            gagal_proses(),
            "Maaf, AI gagal memproses gambarmu kali ini. Silakan coba lagi dalam beberapa saat!"
        );
    }

    #[test]
    fn url_hasil_sama_dengan_js() {
        assert_eq!(
            url_hasil(&json!({ "status": "success", "downloadUrls": ["https://x/1.jpg"] })),
            "https://x/1.jpg"
        );
        assert_eq!(
            url_hasil(&json!({ "status": true, "result": "https://x/2.jpg" })),
            "https://x/2.jpg"
        );
        assert_eq!(
            url_hasil(&json!({ "status": true, "result": { "output_url": ["https://x/3.jpg"] } })),
            "https://x/3.jpg"
        );
        assert_eq!(url_hasil(&json!({ "status": "waiting" })), "");
    }
}
