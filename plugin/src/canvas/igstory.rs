//! `plugins/canvas/igstory.js` — simulasi post Instagram Story dari foto.
//!
//! ## BLOKIR sebagian — jalur render.
//!
//! Render memakai `@napi-rs/canvas` (`createCanvas`, `loadImage`,
//! `GlobalFonts.register`, `ctx.filter = blur`, komposit ikon SVG recolor) yang
//! tidak ada padanannya di `Ctx`/`Reply` — helper `gambar` cuma resize, dan
//! `Kanvas` tidak punya komposit/garis-bebas/ukur teks. Jalur tanpa media
//! (kartu pakai) di-port penuh; jalur dengan media jatuh ke blok `catch` JS
//! (`m.react('❌')` + "GAGAL MEMPROSES GAMBAR"), karena di sanalah render
//! gagal berakhir. Konstanta layout di-port untuk paritas.
//!
//! `ponytail:` kalau API canvas penuh tersedia di `Ctx` nanti, port ulang
//! `generateIgStory` dari `plugins/canvas/igstory.js` (latar storyig.png,
//! foto utama blur 28px cover+blur, avatar bundar, ikon recolor
//! `source-in`, crop 1400px bawah).

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

/// `BG_URL` / `FONT_URL` (`igstory.js:22-23`).
#[cfg(test)]
pub(crate) const BG_URL: &str =
    "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/storyig.png";
#[cfg(test)]
pub(crate) const FONT_URL: &str =
    "https://github.com/rsms/inter/raw/refs/heads/master/docs/font-files/Inter-Medium.woff2";

/// `ICON_URLS` (`igstory.js:25-32`).
#[cfg(test)]
pub(crate) const ICON_URLS: [(&str, &str); 6] = [
    ("like", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/like.svg"),
    ("comment", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/comment.svg"),
    ("repost", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/repost.svg"),
    ("share", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/share.svg"),
    ("save", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/save.svg"),
    ("verify", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/igstory/verifyig.svg"),
];

#[cfg(test)]
pub(crate) const ICON_COLOR: &str = "#f9fdfe";
#[cfg(test)]
pub(crate) const VERIFY_COLOR: &str = "#0095F6";

/// Ukuran kanvas dan zona layout (`igstory.js:41-68`).
#[cfg(test)]
pub(crate) const CANVAS_W: u32 = 1080;
#[cfg(test)]
pub(crate) const CANVAS_H: u32 = 1564;
#[cfg(test)]
pub(crate) const OUTPUT_HEIGHT: u32 = 1400;
#[cfg(test)]
pub(crate) const ICON_SIZE: u32 = 80;
#[cfg(test)]
pub(crate) const VERIFY_SIZE: u32 = 40;
#[cfg(test)]
pub(crate) const DEFAULT_X: (u32, u32, u32, u32, u32) = (22, 256, 459, 688, 978);

/// Kartu pakai saat tidak ada gambar (`igstory.js:272-279`).
fn pakai(prefix: &str) -> String {
    format!(
        "📸 *FITUR IG STORY POST*\n\n\
         Sistem ini akan membuat simulasi gambar postingan Instagram Story menggunakan foto yang kamu kirimkan.\n\n\
         *CARA PENGGUNAAN:*\n\
         - Kirimkan gambar langsung dengan caption `{prefix}igstory`\n\
         - Atau balas (*reply*) pesan gambar yang sudah ada dengan perintah `{prefix}igstory`\n\n\
         _Bot akan otomatis memasukkan nama pengguna, foto profilmu, beserta detail interaksi seperti jumlah suka dan komentar secara acak!_"
    )
}

/// Pesan `catch` (`igstory.js:322`).
const GAGAL: &str = "❌ *GAGAL MEMPROSES GAMBAR*\n\nMaaf, terjadi kesalahan saat mencoba membuat gambar IG Story. Pastikan gambar yang dikirim valid dan coba lagi beberapa saat.";

#[plugin(
    name = "igstory",
    alias = ["igstorypost"],
    category = "canvas",
    description = "Membuat gambar simulasi post Instagram Story dari foto profil dan gambarmu.",
    usage = ".igstory [kirim/reply gambar]",
    example = "",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn igstory(ctx: Ctx) -> Reply {
    // JS: quoted.isMedia dulu, lalu media pesan pemicu (`igstory.js:264-268`).
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
            command: "igstory".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            media: jika(dengan_media),
            ..Default::default()
        };
        igstory(ctx)
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
        let p = oktz_plugin_api::find("igstory").unwrap();
        assert_eq!(p.meta.name, "igstory");
        assert_eq!(p.meta.alias, ["igstorypost"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(
            p.meta.description,
            "Membuat gambar simulasi post Instagram Story dari foto profil dan gambarmu."
        );
        assert_eq!(p.meta.usage, ".igstory [kirim/reply gambar]");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        for a in ["igstorypost"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "igstory");
        }
    }

    #[test]
    fn konstanta_layout_sama_dengan_js() {
        assert_eq!(CANVAS_W, 1080);
        assert_eq!(CANVAS_H, 1564);
        assert_eq!(OUTPUT_HEIGHT, 1400);
        assert_eq!(ICON_SIZE, 80);
        assert_eq!(VERIFY_SIZE, 40);
        // DEFAULT_X: like, comment, repost, share, save.
        assert_eq!(DEFAULT_X, (22, 256, 459, 688, 978));
        assert_eq!(ICON_COLOR, "#f9fdfe");
        assert_eq!(VERIFY_COLOR, "#0095F6");
        assert_eq!(ICON_URLS.len(), 6);
        assert!(BG_URL.ends_with("storyig.png"));
        assert!(FONT_URL.ends_with("Inter-Medium.woff2"));
    }

    #[test]
    fn tanpa_gambar_menampilkan_kartu_pakai() {
        let r = run(false);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            pakai(".")
        );
        assert!(pakai(".").contains("caption `.igstory`"));
        assert!(pakai(".").contains("perintah `.igstory`"));
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
