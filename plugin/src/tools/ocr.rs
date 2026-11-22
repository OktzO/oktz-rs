//! `plugins/tools/ocr.js` — extract teks dari gambar (Offline/Local).
//!
//! ## Penyimpangan yang tidak bisa ditiru
//!
//! JS memakai OCR lokal: `ourin-native` (binary `.node`) dengan cadangan
//! `tesseract.js` (`src/lib/native-loader.js:43-73`). Keduanya jalan di proses
//! Node dan tidak punya padanan di Rust — crate `oktz-scraper` tidak punya
//! modul OCR, dan menambah dependensi (tesseract/ocr crate) berarti menyentuh
//! `Cargo.toml` di luar wilayah yang boleh disentuh.
//!
//! Port ini memakai layanan OCR gratis OCR.Space
//! (`https://api.ocr.space/parse/image`, kunci `helloworld`) lewat
//! `ctx.http` — hasil keluaran bentuknya sama: teks hasil OCR per baris.
//! Bahasa default `eng`, sama seperti `createWorker("eng")` di JS.
//!
//! `sendToolsPreview` (`src/lib/context.js:135-149`) mengirim pesan preview
//! dengan `caption: ${config.info.website} ${text}`, `title`, dan
//! `description`. API `Reply` tidak punya varian preview, jadi port ini
//! mengirim caption yang sama persis sebagai pesan teks; `title` dan
//! `description` tidak punya tempat di `Reply`.

use oktz_plugin_api::registry::MultipartPart;
use oktz_plugin_api::{Ctx, HttpCall, Reply, plugin};
use serde_json::Value;

const OCR_SPACE: &str = "https://api.ocr.space/parse/image";
const API_KEY: &str = "helloworld";

/// `title` preview — `ocr.js:66`.
const JUDUL: &str = "📖 *ᴏᴄʀ*";

fn cara_pakai(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > Reply gambar dengan `{prefix}ocr`\n\n\
         > Media yang didukung:\n\
         > JPG, PNG, GIF, WEBP"
    )
}

fn memproses() -> &'static str {
    "🕕 *ᴍᴇᴍᴘʀᴏsᴇs...*\n\n> Mengekstrak teks dari gambar..."
}

fn gagal_unduh() -> &'static str {
    "❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat download gambar"
}

fn tidak_ada_teks() -> &'static str {
    "❌ *ᴛɪᴅᴀᴋ ᴀᴅᴀ ᴛᴇᴋs*\n\n> Tidak ada teks yang terdeteksi di gambar"
}

/// `responseText` (`ocr.js:53-61`).
fn render_teks(teks: &str) -> String {
    let baris = teks
        .split('\n')
        .map(|l| format!("┃ {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "📖 *ᴏᴄʀ ʀᴇsᴜʟᴛ*\n\n\
         ╭┈┈⬡「 📝 *ᴛᴇᴋs* 」\n\
         {baris}\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n\
         > Total: {panjang} karakter",
        panjang = teks.chars().count()
    )
}

/// Panggil OCR.Space dan ambil teks hasil.
fn ocr_jaringan(ctx: &Ctx, gambar: &[u8]) -> Option<String> {
    let req = HttpCall::post(OCR_SPACE)
        .header("apikey", API_KEY)
        .multipart(vec![
            MultipartPart::file("file", "image.png", gambar.to_vec()),
            MultipartPart::field("language", "eng"),
            MultipartPart::field("isOverlayRequired", "false"),
            MultipartPart::field("scale", "true"),
        ]);
    let res = ctx.http.call(&req).ok()?;
    if res.status >= 400 {
        return None;
    }
    let body: Value = res.json()?;
    let parsed = body.get("ParsedResults").and_then(Value::as_array)?;
    let first = parsed.first()?;
    Some(
        first
            .get("ParsedText")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string(),
    )
}

#[plugin(
    name = "ocr",
    alias = ["totext", "imagetotext", "readtext"],
    category = "tools",
    description = "Extract teks dari gambar (Offline/Local)",
    usage = ".ocr (reply gambar)",
    example = ".ocr",
    cooldown = 10,
    energi = 1,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ocr(ctx: Ctx) -> Reply {
    let is_image = ctx
        .media
        .as_ref()
        .is_some_and(|m| m.kind == oktz_plugin_api::IncomingKind::Image)
        || ctx.quoted().is_some_and(|q| q.kind.is_image());

    if !is_image {
        return Reply::text(cara_pakai(&ctx.prefix));
    }

    let bytes = if ctx
        .media
        .as_ref()
        .is_some_and(|m| m.kind == oktz_plugin_api::IncomingKind::Image)
    {
        ctx.download_media()
    } else {
        ctx.download_quoted_media()
    };
    let bytes = match bytes {
        Ok(b) if !b.is_empty() => b,
        _ => {
            return Reply::sequence(vec![
                Reply::text(memproses()),
                Reply::text(gagal_unduh()),
            ])
            .with_reaction("❌");
        }
    };

    let Some(teks) = ocr_jaringan(&ctx, &bytes) else {
        return Reply::sequence(vec![
            Reply::text(memproses()),
            Reply::text(crate::common::error_template(&ctx)),
        ])
        .with_reaction("☢");
    };

    if teks.is_empty() {
        return Reply::sequence(vec![
            Reply::text(memproses()),
            Reply::text(tidak_ada_teks()),
        ])
        .with_reaction("❌");
    }

    // `sendToolsPreview` — caption-nya `${config.info.website} ${text}`,
    // title `📖 *ᴏᴄʀ*`, description `${len} chars`. `Reply` tidak punya varian
    // preview, jadi tiga bagian itu dipetakan ke kartu interaktif: title ->
    // header, caption -> body, description -> footer.
    let website = ctx.config().info.website.clone();
    let panjang = teks.chars().count();
    let caption = format!("{website} {}", render_teks(&teks));

    Reply::sequence(vec![
        Reply::text(memproses()),
        Reply::interactive(caption)
            .with_header(oktz_plugin_api::registry::CardHeader::new().title(JUDUL))
            .with_footer(format!("{panjang} chars")),
    ])
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, QuotedKind, QuotedMessage, Reply as R};

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_kosong() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "ocr".into(),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_bukan_gambar() {
        let r = ocr(ctx_kosong());
        assert_eq!(
            body(&r),
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
             > Reply gambar dengan `.ocr`\n\n\
             > Media yang didukung:\n\
             > JPG, PNG, GIF, WEBP"
        );
    }

    #[test]
    fn output_sama_dengan_js_hasil_ocr() {
        assert_eq!(
            render_teks("Halo\nDunia"),
            "📖 *ᴏᴄʀ ʀᴇsᴜʟᴛ*\n\n\
             ╭┈┈⬡「 📝 *ᴛᴇᴋs* 」\n\
             ┃ Halo\n\
             ┃ Dunia\n\
             ╰┈┈┈┈┈┈┈┈⬡\n\n\
             > Total: 10 karakter"
        );
    }

    #[test]
    fn pesan_gagal_sama_dengan_js() {
        assert_eq!(gagal_unduh(), "❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat download gambar");
        assert_eq!(
            tidak_ada_teks(),
            "❌ *ᴛɪᴅᴀᴋ ᴀᴅᴀ ᴛᴇᴋs*\n\n> Tidak ada teks yang terdeteksi di gambar"
        );
    }

    #[test]
    fn quote_bukan_gambar_tidak_meloloskan_ke_balasan_berikutnya() {
        // `m.quoted.type === "imageMessage"` — teks biasa tidak boleh dianggap
        // gambar.
        let ctx = Ctx {
            quoted: Some(QuotedMessage {
                id: "q1".into(),
                sender: ME.into(),
                chat: ME.into(),
                text: "halo".into(),
                kind: QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            timestamp: 0,
            }),
            ..ctx_kosong()
        };
        let r = ocr(ctx);
        assert!(body(&r).contains("*ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*"));
    }
}
