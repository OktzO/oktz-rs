//! `plugins/canvas/topixel.js` — pixel art maker (sharp block-average).
//!
//! `ponytail:` inti plugin JS adalah loop piksel mentah sharp
//! (`image.raw().toBuffer()` + rata-rata blok per `block` piksel,
//! `topixel.js:26-81`). `Gambar` (`oktz_plugin_api::gambar`) tidak membuka
//! akses piksel mentah dan crate `image` bukan dependensi langsung
//! `oktz-plugins` (`Cargo.toml` di luar jangkauan port ini), jadi render 1:1
//! mustahil. Yang diport: seluruh cabang validasi + `getBlock` persis JS;
//! langkah pixelate jatuh ke cabang `catch` JS (`topixel.js:118-121`).
//!
//! `ponytail:` reaksi `🕕` (`topixel.js:103`) hilang — `Reply` hanya membawa
//! satu reaksi; yang dipegang reaksi catch (`☢`).

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

/// `topixel.js:21-24` — `Math.min(Math.max(Number(level) || 12, 1), 40)`,
/// lalu `41 - value`.
///
/// `Number(level)` ditiru: `f64` yang terparse; `NaN`/negatif-nol seperti JS.
/// `|| 12` JS memicu untuk `NaN`, `0`, `-0` (falsy).
pub(crate) fn get_block(level: &str) -> u32 {
    let nilai: f64 = parse_number(level).unwrap_or(f64::NAN);
    let nilai = if nilai == 0.0 || nilai.is_nan() { 12.0 } else { nilai };
    let nilai = nilai.clamp(1.0, 40.0);
    (41.0 - nilai) as u32
}

/// `Number(str)` untuk bentuk yang relevan dari argumen CLI: desimal,
/// tanda, spasi, `Infinity`. Selain itu `NaN`.
pub(crate) fn parse_number(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        // Number("") === 0, tapi `0 || 12` → 12; kembalikan Some(0) agar
        // get_block meniru rantai `||`.
        return Some(0.0);
    }
    let t = t.strip_prefix('+').unwrap_or(t);
    let (negatif, t) = match t.strip_prefix('-') {
        Some(sisa) => (true, sisa),
        None => (false, t),
    };
    let infinity = matches!(
        t.to_ascii_lowercase().as_str(),
        "infinity" | "inf"
    );
    if infinity {
        return Some(if negatif { f64::NEG_INFINITY } else { f64::INFINITY });
    }
    if t.is_empty() {
        return None;
    }
    // Bagian numerik: digit + paling banyak satu titik + eksponen.
    let (mantissa, eksponen) = match t.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (t, None),
    };
    let (bulat, pecahan) = match mantissa.split_once('.') {
        Some((b, p)) => (b, Some(p)),
        None => (mantissa, None),
    };
    if bulat.is_empty() && pecahan.map_or(true, str::is_empty) {
        return None;
    }
    let digit_ok = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digit_ok(bulat) || !pecahan.map_or(true, digit_ok) {
        return None;
    }
    let exp: Option<i32> = match eksponen {
        Some(e) => {
            let (neg_e, e) = match e.strip_prefix(['+', '-']) {
                Some(sisa) => (e.starts_with('-'), sisa),
                None => (false, e),
            };
            if !digit_ok(e) {
                return None;
            }
            Some(if neg_e {
                -(e.parse::<i32>().ok()?)
            } else {
                e.parse::<i32>().ok()?
            })
        }
        None => None,
    };
    let mut nilai: f64 = format!("{}.{}", bulat, pecahan.unwrap_or("")).parse().ok()?;
    if let Some(exp) = exp {
        nilai *= 10f64.powi(exp);
    }
    Some(if negatif { -nilai } else { nilai })
}

/// `topixel.js:96` — kartu untuk pesan pemicu bukan gambar.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "👾 *PIXEL ART MAKER*\n\nKirim atau reply foto dengan perintah `{prefix}topixel [level]` untuk mengubah fotomu menjadi gaya retro pixel art!\n\n_Catatan: Level opsional antara 1-40 (semakin besar semakin kotak-kotak)._"
    )
}

/// `topixel.js:87-101` — media: quoted HARUS gambar (`⚠️`), pemicu HARUS
/// gambar (`help`), unduhan/keberadaan kosong → `topixel.js:101`.
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

#[plugin(
    name = "topixel",
    alias = ["pixelate", "pixelart"],
    category = "canvas",
    description = "Ubah foto kamu jadi gambar pixel art yang keren",
    usage = ".topixel [level] (reply/kirim foto)",
    example = ".topixel 30",
    cooldown = 10,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn topixel(ctx: Ctx) -> Reply {
    let level = ctx.args.first().cloned().unwrap_or_else(|| "12".into());
    let _ = get_block(&level);

    let buffer = match ambil_media(&ctx) {
        Media::Bytes(b) => b,
        Media::BukanQuotedGambar => {
            return Reply::text("⚠️ Kak, tolong reply ke pesan gambar ya!");
        }
        Media::BukanGambar => return Reply::text(help(&ctx.prefix)),
        Media::TidakAda => return Reply::text("❌ Gagal membaca media gambar, coba lagi!"),
    };
    let _ = buffer;

    // Pixelate tidak tersedia — cabang catch JS (`topixel.js:118-121`).
    Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        IncomingMedia, MediaError, MediaFetch, MediaOrigin, QuotedKind, QuotedMessage,
    };
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Unduh(Result<Vec<u8>, MediaError>);

    impl MediaFetch for Unduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.0.clone()
        }
    }

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "topixel".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Ctx::default()
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

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("topixel").unwrap().handler)(ctx)
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

    #[test]
    fn meta_dan_alias_sesuai_js() {
        let p = oktz_plugin_api::find("topixel").unwrap();
        assert_eq!(p.meta.name, "topixel");
        assert_eq!(p.meta.alias, ["pixelate", "pixelart"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Ubah foto kamu jadi gambar pixel art yang keren");
        assert_eq!(p.meta.usage, ".topixel [level] (reply/kirim foto)");
        assert_eq!(p.meta.example, ".topixel 30");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 2);
        for k in ["topixel", "pixelate", "pixelart"] {
            assert_eq!(oktz_plugin_api::find(k).unwrap().meta.name, "topixel");
        }
    }

    #[test]
    fn get_block_meniru_rantai_number_clamp_js() {
        // `Number("30")` → 30 → block 41-30 = 11.
        assert_eq!(get_block("30"), 11);
        // Default handler: `m.args[0] || "12"` → block 41-12 = 29.
        assert_eq!(get_block("12"), 29);
        // `Number("abc")` NaN → `|| 12` → 29.
        assert_eq!(get_block("abc"), 29);
        // `Number("0")` 0 falsy → `|| 12` → 29.
        assert_eq!(get_block("0"), 29);
        // Clamp bawah: level 1 → block 40.
        assert_eq!(get_block("1"), 40);
        // Clamp atas: level 100 → 40 → block 1.
        assert_eq!(get_block("100"), 1);
        // Negatif → clamp 1 → block 40.
        assert_eq!(get_block("-5"), 40);
        // Desimal JS: Number("2.9") = 2.9 → clamp → block 41-2.9 = 38.1 →
        // JS mengembalikan pecahan; Rust memotong ke 38.
        assert_eq!(get_block("2.9"), 38);
    }

    #[test]
    fn parse_number_meniru_number_js() {
        assert_eq!(parse_number("12"), Some(12.0));
        assert_eq!(parse_number(" 30 "), Some(30.0));
        assert_eq!(parse_number("+7"), Some(7.0));
        assert_eq!(parse_number("-5"), Some(-5.0));
        assert_eq!(parse_number("2.5"), Some(2.5));
        assert_eq!(parse_number("1e2"), Some(100.0));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("12a"), None);
        assert_eq!(parse_number(""), Some(0.0));
        assert_eq!(parse_number("Infinity"), Some(f64::INFINITY));
    }

    #[test]
    fn quoted_bukan_gambar_membalas_peringatan() {
        let mut ctx = ctx_of("");
        ctx.quoted = Some(QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: ME.into(),
            text: String::new(),
            kind: QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        });
        let r = run(ctx);
        assert_eq!(teks(&r), "⚠️ Kak, tolong reply ke pesan gambar ya!");
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn pemicu_bukan_gambar_membalas_help() {
        let mut ctx = ctx_of("");
        ctx.media = Some(IncomingMedia {
            kind: IncomingKind::Video,
            mimetype: Some("video/mp4".into()),
            file_name: None,
            caption: String::new(),
        });
        let r = run(ctx);
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn tanpa_media_membalas_gagal_membaca() {
        let r = run(ctx_of(""));
        assert_eq!(teks(&r), "❌ Gagal membaca media gambar, coba lagi!");
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn gambar_tersedia_jalan_ke_cabang_catch_dengan_radiasi() {
        let mut ctx = ctx_of("30");
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Ok(b"IMG".to_vec())));
        let r = run(ctx);
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "topixel"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn unduhan_gagal_membalas_gagal_membaca() {
        let mut ctx = ctx_of("");
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Err(MediaError { message: "gagal".into() })));
        let r = run(ctx);
        assert_eq!(teks(&r), "❌ Gagal membaca media gambar, coba lagi!");
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#topixel [level]`"));
    }
}
