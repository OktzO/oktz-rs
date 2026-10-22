//! `plugins/tools/converter.js` — convert file ke format lain lewat
//! `mconverter.eu`.
//!
//! Scraper `mconverter` sudah ada di `oktz-scraper`
//! (`oktz_scraper::mconverter::convert`), jadi plugin ini tidak menyalin
//! logikanya.
//!
//! ## Penyimpangan
//!
//! - Berkas sementara ditulis lewat `ctx.berkas()` dengan jalur relatif
//!   `temp/...` (`process.cwd()/temp` di JS). Scrapercopenia membaca berkasnya
//!   dengan `std::fs`, jadi jalurnya harus relatif terhadap direktori kerja
//!   berjalan. Di produksi `BerkasNyata` berakar ke `current_dir()`
//!   (`lib/oktz-wa/bot.rs:153`), jadi keduanya menunjuk berkas yang sama.
//!   Di test, `BerkasPalsu` menyimpan di memori dan scraper membaca dari
//!   disk — hasilnya `File tidak ada`, sama seperti jalur gagal di JS.
//! - `Date.now()` dipakai untuk nama berkas, jadi nama berkas yang terkirim
//!   (`converted_<timestamp>.<fmt>`) tidak pernah sama persis antar-jalur.
//!   Bagian yang dilihat user (nama format, pesan status) tetap sama.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

fn usage(prefix: &str) -> String {
    format!(
        "🔄 *ᴄᴏɴᴠᴇʀᴛᴇʀ*\n\n\
         > Reply file dengan format tujuan\n\n\
         *Format:*\n\
         > `{prefix}converter <format>`\n\n\
         *Contoh:*\n\
         > `{prefix}converter mp3`\n\
         > `{prefix}converter mp4`\n\
         > `{prefix}converter png`\n\n\
         *Cara pakai:*\n\
         > 1. Reply file yang mau diconvert\n\
         > 2. Ketik `{prefix}converter <format>`",
    )
}

fn tanpa_format(prefix: &str) -> String {
    format!("❌ Masukkan format tujuan!\n\n> Contoh: `{prefix}converter mp3`")
}

/// `filename.split(".").pop() || "bin"` (`converter.js:84`).
fn ekstensi(nama_berkas: &str) -> String {
    match nama_berkas.rsplit_once('.') {
        Some((_, ext)) if !ext.is_empty() => ext.to_string(),
        _ => "bin".to_string(),
    }
}

#[plugin(
    name = "converter",
    alias = ["convert", "konversi"],
    category = "tools",
    description = "Convert file ke format lain",
    usage = ".converter <format> (reply file)",
    example = ".converter mp3",
    cooldown = 30,
    energi = 3,
    is_owner = false,
    is_premium = true,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn converter(ctx: Ctx) -> Reply {
    let target_format = ctx.text.trim().to_lowercase();

    let quoted_media = ctx.quoted().is_some_and(|q| q.is_media());
    let has_media = ctx.is_media();

    // `if (!m.quoted && !m.isMedia)` (`converter.js:26`) — pesan yang
    // di-reply ADA tapi bukan media belum disqualifikasi di sini; disqualifikasi
    //-nya terjadi di bawah, setelah cek format.
    if ctx.quoted().is_none() && !has_media {
        return Reply::text(usage(&ctx.prefix));
    }

    if target_format.is_empty() {
        return Reply::text(tanpa_format(&ctx.prefix));
    }

    if !quoted_media && !has_media {
        return Reply::text("❌ Reply file yang mau diconvert!");
    }

    // `quoted?.isMedia` menang atas `m.isMedia` (`converter.js:52-58`).
    let (nama_berkas, unduhan) = if quoted_media {
        let q = ctx.quoted().expect("sudah dicek");
        let nama = q
            .file_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("file_{}", crate::common::now_ms()));
        (nama, ctx.download_quoted_media())
    } else {
        let nama = ctx
            .media
            .as_ref()
            .and_then(|m| m.file_name.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("file_{}", crate::common::now_ms()));
        (nama, ctx.download_media())
    };

    let buffer = match unduhan {
        Ok(b) => b,
        Err(_) => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    let ext = ekstensi(&nama_berkas);
    let temp_dir = "temp";
    let _ = ctx.berkas().buat_folder(temp_dir);
    let temp_path = format!(
        "{temp_dir}/convert_{}.{ext}",
        crate::common::now_ms()
    );
    if ctx.berkas().tulis(&temp_path, &buffer).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    let hasil = oktz_scraper::mconverter::convert(&*ctx.http, &temp_path, &target_format);
    let _ = ctx.berkas().hapus(&temp_path);

    let hasil = match hasil {
        Ok(v) => v,
        Err(_) => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    if let Some(err) = hasil.get("error").and_then(Value::as_str) {
        return Reply::text(format!("❌ *ɢᴀɢᴀʟ ᴄᴏɴᴠᴇʀᴛ*\n\n> {err}")).with_reaction("❌");
    }

    let Some(url) = hasil.get("url").and_then(Value::as_str) else {
        return Reply::text(format!("❌ *ɢᴀɢᴀʟ ᴄᴏɴᴠᴇʀᴛ*\n\n> {hasil}")).with_reaction("❌");
    };

    let nama_keluaran = format!("converted_{}.{target_format}", crate::common::now_ms());
    let media = MediaSource::url(url).with_mimetype(format!("application/{target_format}"));

    Reply::sequence(vec![
        Reply::text("🕕 *ᴍᴇɴɢᴜɴᴅᴜʜ ғɪʟᴇ...*"),
        Reply::text(format!("🔄 *ᴄᴏɴᴠᴇʀᴛɪɴɢ...*\n\n> {ext} → {target_format}")),
        Reply::document(media, nama_keluaran),
    ])
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, QuotedKind, QuotedMessage, Reply as R};

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_dengan_quoted(teks: &str) -> Ctx {
        Ctx {
            text: teks.to_string(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "converter".into(),
            quoted: Some(QuotedMessage {
                id: "q1".into(),
                sender: ME.into(),
                chat: ME.into(),
                text: String::new(),
                kind: QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_media() {
        let r = converter(Ctx {
            quoted: None,
            ..ctx_dengan_quoted("")
        });
        assert_eq!(
            body(&r),
            "🔄 *ᴄᴏɴᴠᴇʀᴛᴇʀ*\n\n\
             > Reply file dengan format tujuan\n\n\
             *Format:*\n\
             > `.converter <format>`\n\n\
             *Contoh:*\n\
             > `.converter mp3`\n\
             > `.converter mp4`\n\
             > `.converter png`\n\n\
             *Cara pakai:*\n\
             > 1. Reply file yang mau diconvert\n\
             > 2. Ketik `.converter <format>`"
        );
    }

    #[test]
    fn output_sama_dengan_js_tanpa_format() {
        let r = converter(ctx_dengan_quoted(""));
        assert_eq!(body(&r), tanpa_format("."));
    }

    #[test]
    fn ekstensi_sama_dengan_js() {
        assert_eq!(ekstensi("a.mp3"), "mp3");
        assert_eq!(ekstensi("video.tar.gz"), "gz");
        assert_eq!(ekstensi("tanpa_ext"), "bin");
    }

    #[test]
    fn pesan_format_kosong_sama_dengan_js() {
        assert_eq!(
            tanpa_format("."),
            "❌ Masukkan format tujuan!\n\n> Contoh: `.converter mp3`"
        );
    }
}
