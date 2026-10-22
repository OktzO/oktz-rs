//! `plugins/tools/hd3.js` — memperjelas gambar blur menjadi tajam dengan AI
//! (Unblur).
//!
//! ## Penyimpangan
//!
//! - JS memakai `resolver.resolve("hd", { kind: "unblur", media })`
//!   (`src/capabilities/hd.js`): backend lokal `fgsi.dpdns.org` (buat task +
//!   polling) lalu agregator `my.izuka-api.xyz`. Port ini memakai scraper
//!   `oktz_scraper::hdvid` (backend lokal) dan, kalau gagal, memanggil
//!   agregator secara langsung — urutan dan bentuk respons sama.
//! - Batas polling di JS untuk jalur ini 60 detik (`POLL_YATIM_MS`,
//!   `src/capabilities/hd.js:86`); interval polling 3 detik dan maksimal 5
//!   kegagalan sementara ikut dipakai.
//! - `jpegThumbnail` pada pesan dokumen tidak punya field di `Reply::Document`,
//!   jadi thumbnail 50×50 yang dibuat JS tidak ikut. Nama berkas tetap sama.

use oktz_plugin_api::registry::MultipartPart;
use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

const IZUKA: &str = "https://my.izuka-api.xyz/api/tools/unblur";
const POLL_INTERVAL_MS: u64 = 3000;
const POLL_TIMEOUT_MS: u64 = 60_000;
const MAX_TRANSIEN: u32 = 5;

fn cara_pakai(prefix: &str) -> String {
    format!(
        "✨ *FITUR HD ENHANCE V3 (UNBLUR)*\n\nFitur canggih untuk memperbaiki gambar yang buram (blur) menjadi jelas dan tajam kembali menggunakan kecerdasan buatan!\n\n*Cara Penggunaan:*\n- Kirim gambar dan tambahkan pesan *.hd3*\n- Atau balas (reply) gambar yang sudah terkirim dengan perintah *.hd3*\n\n_Proses rendering mungkin memerlukan waktu beberapa saat._"
    )
}

fn gagal_unduh() -> &'static str {
    "Maaf, sistem gagal mengunduh gambar yang kamu berikan. Silakan coba kirim ulang gambarnya!"
}

fn gagal_proses() -> &'static str {
    "Maaf, AI gagal memproses gambarmu kali ini. Silakan coba lagi dalam beberapa saat!"
}

/// Backend lokal — `fgsi.dpdns.org` (`src/scraper/hdvid.js`).
fn lewat_fgsi(ctx: &Ctx, path: &str, bytes: &[u8], api_key: &str) -> Option<String> {
    let task =
        oktz_scraper::hdvid::create_enhance_task(ctx.http.as_ref(), path, bytes, api_key).ok()?;
    let poll_url = task.get("pollUrl").and_then(Value::as_str)?;
    let hasil = oktz_scraper::hdvid::poll_enhance_task(
        ctx.http.as_ref(),
        poll_url,
        api_key,
        POLL_INTERVAL_MS,
        POLL_TIMEOUT_MS,
        MAX_TRANSIEN,
    )
    .ok()?;
    let url = crate::tools::hd2::url_hasil(&hasil);
    if url.is_empty() { None } else { Some(url) }
}

/// Backend agregator — `my.izuka-api.xyz` (`src/capabilities/hd.js`).
fn lewat_izuka(ctx: &Ctx, gambar: &[u8]) -> Option<String> {
    let req = HttpCall::post(IZUKA).multipart(vec![
        MultipartPart::file("image", "image.jpg", gambar.to_vec()).with_mimetype("image/jpeg"),
    ]);
    let res = ctx.http.call(&req).ok()?;
    if res.status >= 400 {
        return None;
    }
    let body = res.json()?;
    if body.get("status").and_then(Value::as_bool) == Some(false) {
        return None;
    }
    let url = crate::tools::hd2::url_hasil(&body);
    if url.is_empty() { None } else { Some(url) }
}

#[plugin(
    name = "hd3",
    alias = ["enhance3", "upscale3", "unblur"],
    category = "tools",
    description = "Memperjelas gambar blur menjadi tajam dengan AI (Unblur)",
    usage = ".hd3 (reply gambar)",
    example = ".hd3",
    cooldown = 20,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn hd3(ctx: Ctx) -> Reply {
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

    let temp_dir = "tmp";
    let _ = ctx.berkas().buat_folder(temp_dir);
    let path = format!("{temp_dir}/hd-{}.jpg", crate::common::now_ms());
    if ctx.berkas().tulis(&path, &bytes).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    let api_key = ctx.config().api_keys.fgsi.clone();
    let url = lewat_fgsi(&ctx, &path, &bytes, &api_key).or_else(|| lewat_izuka(&ctx, &bytes));
    let _ = ctx.berkas().hapus(&path);

    let Some(url) = url else {
        return Reply::text(gagal_proses()).with_reaction("❌");
    };

    // Thumbnail 50×50 quality 30 — tidak bisa dikirim lewat `Reply::Document`.
    let _ = oktz_plugin_api::gambar::sharp(&bytes)
        .map(|g| g.ubah_ukuran(50, 50))
        .and_then(|g| g.ke_jpeg(30));

    let nama_berkas = format!("UNBLUR_BY_{}.jpg", ctx.config().bot.name);
    let media = MediaSource::url(url).with_mimetype("image/jpeg");

    Reply::document(media, nama_berkas).with_reaction("✅")
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
            command: "hd3".into(),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_bukan_gambar() {
        let r = hd3(ctx_kosong());
        assert_eq!(body(&r), "✨ *FITUR HD ENHANCE V3 (UNBLUR)*\n\nFitur canggih untuk memperbaiki gambar yang buram (blur) menjadi jelas dan tajam kembali menggunakan kecerdasan buatan!\n\n*Cara Penggunaan:*\n- Kirim gambar dan tambahkan pesan *.hd3*\n- Atau balas (reply) gambar yang sudah terkirim dengan perintah *.hd3*\n\n_Proses rendering mungkin memerlukan waktu beberapa saat._");
    }

    #[test]
    fn output_sama_dengan_js_gagal() {
        assert_eq!(
            gagal_unduh(),
            "Maaf, sistem gagal mengunduh gambar yang kamu berikan. Silakan coba kirim ulang gambarnya!"
        );
        assert_eq!(
            gagal_proses(),
            "Maaf, AI gagal memproses gambarmu kali ini. Silakan coba lagi dalam beberapa saat!"
        );
    }
}
