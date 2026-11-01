//! `plugins/canvas/igstoryimg.js` — layout Instagram Story estetik dari foto.
//!
//! ## BLOKIR sebagian — jalur render.
//!
//! Render memakai `@napi-rs/canvas` (`createCanvas`, `loadImage`,
//! `GlobalFonts.register` dua font Inter, blur 28px dengan clip sudut-bawah
//! membulat, edge blur 10px) yang tidak ada padanannya di `Ctx`/`Reply` —
//! helper `gambar` cuma resize, dan `Kanvas` tidak punya clip/quadratic curve/
//! ukur teks. Jalur tanpa media (kartu pakai) di-port penuh; jalur dengan media
//! jatuh ke blok `catch` JS (`m.react('❌')` + "GAGAL MEMPROSES GAMBAR").
//! Konstanta layout di-port untuk paritas.
//!
//! `ponytail:` kalau API canvas penuh tersedia di `Ctx` nanti, port ulang
//! `generateIgimg` dari `plugins/canvas/igstoryimg.js` (bg igimg.png 898x1600,
//! foto zona (38,136)-(860,912) radius 20, PP bundar 80px, nama auto-fit
//! 25→16px, username 17px).

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

/// `IGIMG_BG_URL` + dua font Inter (`igstoryimg.js:7-9`).
#[cfg(test)]
pub(crate) const IGIMG_BG_URL: &str =
    "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/Image/igimg.png";
#[cfg(test)]
pub(crate) const FONT_SEMIBOLD_URL: &str = "https://fonts.gstatic.com/s/inter/v18/UcCO3FwrK3iLTeHuS_nVMrMxCp50SjIw2boKoduKmMEVuGKYAZ9hiJ-Ek-_EeA.woff2";
#[cfg(test)]
pub(crate) const FONT_REGULAR_URL: &str = "https://fonts.gstatic.com/s/inter/v18/UcCO3FwrK3iLTeHuS_nVMrMxCp50SjIw2boKoduKmMEVuLyfAZ9hiJ-Ek-_EeA.woff2";

/// `configLayout` (`igstoryimg.js:18-24`) — `(a, b, c, d, radius)` zona foto.
#[cfg(test)]
pub(crate) const FOTO_ZONE: (i32, i32, i32, i32, i32) = (136, 912, 38, 860, 20);
#[cfg(test)]
pub(crate) const EDGE_BLUR: (u32, u32) = (3, 10);
#[cfg(test)]
pub(crate) const PP: (i32, i32, u32) = (110, 82, 80);
#[cfg(test)]
pub(crate) const NAMA_TEKS: (i32, i32, u32, u32, u32) = (170, 58, 25, 500, 16);
#[cfg(test)]
pub(crate) const USERNAME_TEKS: (i32, i32, u32) = (170, 90, 17);
#[cfg(test)]
pub(crate) const BG_W: u32 = 898;
#[cfg(test)]
pub(crate) const BG_H: u32 = 1600;

/// Kartu pakai saat tidak ada gambar (`igstoryimg.js:237-244`).
fn pakai(prefix: &str) -> String {
    format!(
        "📸 *FITUR IG STORY IMAGE*\n\n\
         Fitur ini memungkinkan kamu untuk menyulap foto biasamu menjadi tampilan ala Instagram Story yang estetik dan kekinian.\n\n\
         *CARA PENGGUNAAN:*\n\
         - Kirimkan gambar langsung dengan caption `{prefix}igstoryimg`\n\
         - Atau balas (*reply*) pesan gambar yang sudah ada dengan perintah `{prefix}igstoryimg`\n\n\
         _Bot akan otomatis memasang foto profil dan namamu di dalam gambarnya!_"
    )
}

/// Pesan `catch` (`igstoryimg.js:283`).
const GAGAL: &str = "❌ *GAGAL MEMPROSES GAMBAR*\n\nMaaf, terjadi kesalahan saat mencoba membuat gambar IG Story. Pastikan gambar yang dikirim valid dan coba lagi beberapa saat.";

#[plugin(
    name = "igstoryimg",
    alias = ["igstoryimage"],
    category = "canvas",
    description = "Membuat gambar layout Instagram Story dari gambarmu.",
    usage = ".igstoryimg [kirim/reply gambar]",
    example = "",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn igstoryimg(ctx: Ctx) -> Reply {
    // JS: quoted.isMedia dulu, lalu media pesan pemicu (`igstoryimg.js:229-233`).
    let media_ada = ctx.quoted().is_some_and(|q| q.is_media())
        || ctx.is_media_kind(IncomingKind::Image)
        || ctx.is_media_kind(IncomingKind::Video);

    if !media_ada {
        return Reply::text(pakai(&ctx.prefix));
    }

    // Jalur render `@napi-rs/canvas` tidak tersedia — jatuh ke blok `catch`
    // JS: reaksi ❌ + pesan gagal. Lihat catatan kepala.
    Reply::text(GAGAL).with_reaction("❌")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dengan_media: bool) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "igstoryimg".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            media: jika(dengan_media),
            ..Default::default()
        };
        igstoryimg(ctx)
    }

    fn jika(ada: bool) -> Option<oktz_plugin_api::IncomingMedia> {
        ada.then(|| oktz_plugin_api::IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: None,
            file_name: None,
            caption: String::new(),
        })
    }

    #[test]
    fn meta_dan_alias() {
        let p = oktz_plugin_api::find("igstoryimg").unwrap();
        assert_eq!(p.meta.name, "igstoryimg");
        assert_eq!(p.meta.alias, ["igstoryimage"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(
            p.meta.description,
            "Membuat gambar layout Instagram Story dari gambarmu."
        );
        assert_eq!(p.meta.usage, ".igstoryimg [kirim/reply gambar]");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        for a in ["igstoryimage"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "igstoryimg");
        }
    }

    #[test]
    fn konstanta_layout_sama_dengan_js() {
        assert_eq!(BG_W, 898);
        assert_eq!(BG_H, 1600);
        // foto: { a: 136, b: 912, c: 38, d: 860, radius: 20 }
        assert_eq!(FOTO_ZONE, (136, 912, 38, 860, 20));
        // edgeBlur: { width: 3, blur: 10 }
        assert_eq!(EDGE_BLUR, (3, 10));
        // pp: { x: 110, y: 82, size: 80 }
        assert_eq!(PP, (110, 82, 80));
        // namaTeks: { x: 170, y: 58, fontSize: 25, maxWidth: 500, minFontSize: 16 }
        assert_eq!(NAMA_TEKS, (170, 58, 25, 500, 16));
        // usernameTeks: { x: 170, y: 90, fontSize: 17 }
        assert_eq!(USERNAME_TEKS, (170, 90, 17));
        assert!(IGIMG_BG_URL.ends_with("igimg.png"));
        assert!(FONT_SEMIBOLD_URL.contains("inter"));
        assert!(FONT_REGULAR_URL.contains("inter"));
    }

    #[test]
    fn tanpa_gambar_menampilkan_kartu_pakai() {
        let r = run(false);
        assert_eq!(r.without_mutations().text_body().unwrap(), pakai("."));
        assert!(pakai(".").contains("caption `.igstoryimg`"));
        assert!(pakai(".").contains("ala Instagram Story yang estetik"));
    }

    #[test]
    fn dengan_gambar_jatuh_ke_catch_js() {
        let r = run(true);
        assert_eq!(r.without_mutations().text_body().unwrap(), GAGAL);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "❌"
        ));
    }
}
