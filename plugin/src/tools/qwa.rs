//! `plugins/tools/qwa.js` — membuat gambar quote WhatsApp lewat
//! `qwa.eeq.my.id/api/generate`.
//!
//! ## Penyimpangan
//!
//! - `mainMsg.messageTimestamp` (`qwa.js:59`) tidak ada di `Ctx`, jadi waktu
//!   yang dikirim diambil dari jam sekarang (cabang `new Date()` di JS).
//! - `pushName` pesan yang di-reply tidak ada di `QuotedMessage`, jadi untuk
//!   quote yang di-reply dipakai nilai cadangan `~ User` — sama dengan
//!   cabang JS ketika `pushName` kosong.
//! - `formatNumber` memakai `awesome-phonenumber`. Port ini memakai
//!   pemformatan internasional sederhana: `+62` diikuti nomor nasional yang
//!   dikelompokkan 3-4-4 (pola `libphonenumber` untuk nomor sel Indonesia),
//!   lalu `-` diganti spasi seperti di JS. Nomor non-Indonesia memakai
//!   pengelompokan 3-4-3.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::json;

const ENDPOINT: &str = "https://qwa.eeq.my.id/api/generate";
const PP_CADANGAN: &str = "https://files.catbox.moe/ios0gb.jfif";

fn format_salah(prefix: &str) -> String {
    format!(
        "❌ *FORMAT SALAH*\n\nKirim perintah `{prefix}qwa <teks>` atau reply pesan orang lain dengan `{prefix}qwa`."
    )
}

fn gagal() -> &'static str {
    "❌ *GAGAL MEMBUAT QUOTE*\n\n> Terjadi kesalahan atau API sedang bermasalah."
}

fn caption_sukses() -> &'static str {
    "✅ Berhasil membuat quote WhatsApp!"
}

/// `formatNumber` (`qwa.js:85-94`) — nomor internasional dengan spasi.
fn format_number(num_str: &str) -> String {
    let bersih = num_str.split('@').next().unwrap_or(num_str);
    if bersih.starts_with('+') {
        return bersih.to_string();
    }
    let digits: String = bersih.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return format!("+{bersih}");
    }
    if let Some(nasional) = digits.strip_prefix("62") {
        return format!("+62 {}",kelompokkan(nasional, &[3, 4, 4]));
    }
    if let Some(nasional) = digits.strip_prefix('0') {
        return format!("+{}", kelompokkan(nasional, &[3, 4, 3]));
    }
    format!("+{}", kelompokkan(&digits, &[3, 4, 3]))
}

/// Kelompokkan digit sesuai pola; sisa terakhir mengikuti pola terakhir.
fn kelompokkan(digits: &str, pola: &[usize]) -> String {
    let total: usize = pola.iter().sum();
    if digits.len() < total {
        return digits.to_string();
    }
    let mut out = String::new();
    let mut idx = 0usize;
    for (i, &ukuran) in pola.iter().enumerate() {
        if idx >= digits.len() {
            break;
        }
        let ambil = ukuran.min(digits.len() - idx);
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&digits[idx..idx + ambil]);
        idx += ambil;
    }
    if idx < digits.len() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&digits[idx..]);
    }
    out
}

/// `~ ${pushName}` atau `~ User` (`qwa.js:97`).
fn nama_sender(push_name: &str) -> String {
    if push_name.is_empty() {
        "~ User".to_string()
    } else {
        format!("~ {push_name}")
    }
}

#[plugin(
    name = "qwa",
    alias = ["quotewa", "fakeqwa"],
    category = "tools",
    description = "Membuat gambar quote WhatsApp",
    usage = ".qwa [teks]",
    example = ".qwa Halo Dunia",
    cooldown = 5,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn qwa(ctx: Ctx) -> Reply {
    let text_to_quote = ctx.args.join(" ");
    let quoted = ctx.quoted().cloned();

    // `mainMsg` = pesan yang di-reply kalau ada teks; selain itu pesan itu
    // sendiri yang jadi isi quote (`qwa.js:33-53`).
    let (main_push_name, main_sender, teks, quote_pakai) = match (&quoted, text_to_quote.is_empty())
    {
        (Some(q), true) => (String::new(), q.sender.clone(), q.text.clone(), false),
        (Some(_q), false) => (
            ctx.push_name.clone(),
            ctx.sender.clone(),
            text_to_quote.clone(),
            true,
        ),
        (None, _) => (
            ctx.push_name.clone(),
            ctx.sender.clone(),
            text_to_quote.clone(),
            false,
        ),
    };

    let main_is_media = if quote_pakai {
        ctx.is_media()
    } else {
        quoted.as_ref().is_some_and(|q| q.is_media())
    };

    if teks.is_empty() && !main_is_media {
        return Reply::text(format_salah(&ctx.prefix));
    }

    // `getPp(sock, jid)` (`qwa.js:22-29`).
    let avatar = ctx
        .sock()
        .foto_profil(&main_sender, "image")
        .ok()
        .flatten()
        .unwrap_or_else(|| PP_CADANGAN.to_string());

    // `new Date()` — `mainMsg.messageTimestamp` tidak ada di `Ctx`.
    let parts = crate::common::jakarta_parts(crate::common::now_ms());
    let time_str = format!("{:02}.{:02}", parts.3, parts.4);

    let mut payload = json!({
        "sender_name": nama_sender(&main_push_name),
        "sender_number": format_number(&main_sender),
        "sender_avatar": avatar,
        "message": teks,
        "time": time_str,
        "background": false,
    });

    if quote_pakai
        && let Some(q) = &quoted
    {
        // `pushName` pesan yang di-reply tidak tersedia; JS memakai `~ User`.
        payload["quoted"] = json!({
            "name": nama_sender(""),
            "number": format_number(&q.sender),
            "message": q.text,
        });
    }

    let res = ctx
        .http
        .call(
            &HttpCall::post(ENDPOINT)
                .header("Content-Type", "application/json")
                .json(&payload),
        );
    let Ok(res) = res else {
        return Reply::text(gagal()).with_reaction("❌");
    };
    if res.status >= 400 {
        return Reply::text(gagal()).with_reaction("❌");
    }
    let bytes = res.body.bytes().to_vec();
    if bytes.is_empty() {
        return Reply::text(gagal()).with_reaction("❌");
    }

    Reply::image(MediaSource::bytes(bytes), caption_sukses()).with_reaction("✅")
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
            command: "qwa".into(),
            ..Ctx::default()
        }
    }

    fn quoted(text: &str) -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: ME.into(),
            text: text.into(),
            kind: QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_teks() {
        let r = qwa(ctx_kosong());
        assert_eq!(
            body(&r),
            "❌ *FORMAT SALAH*\n\nKirim perintah `.qwa <teks>` atau reply pesan orang lain dengan `.qwa`."
        );
    }

    #[test]
    fn output_sama_dengan_js_caption_sukses() {
        assert_eq!(caption_sukses(), "✅ Berhasil membuat quote WhatsApp!");
        assert_eq!(
            gagal(),
            "❌ *GAGAL MEMBUAT QUOTE*\n\n> Terjadi kesalahan atau API sedang bermasalah."
        );
    }

    #[test]
    fn nomor_sama_dengan_js() {
        assert_eq!(format_number("6281234567890"), "+62 812 3456 7890");
        assert_eq!(format_number("6281234567890@s.whatsapp.net"), "+62 812 3456 7890");
        assert_eq!(format_number("+1 212 555 0123"), "+1 212 555 0123");
    }

    #[test]
    fn nama_sender_sama_dengan_js() {
        assert_eq!(nama_sender("Budi"), "~ Budi");
        assert_eq!(nama_sender(""), "~ User");
    }
}
