//! `plugins/canvas/ttqc.js` — fake quote chat ala TikTok.
//!
//! ## BLOKIR sebagian — jalur render.
//!
//! Render memakai `@napi-rs/canvas` (template ttqc/qyzwa.png, lima font yang
//! diunduh + didaftarkan, `measureText` untuk wrap + lebar bubble) yang tidak
//! ada padanannya di `Ctx`/`Reply`. Yang di-port penuh: seleksi teks
//! (argumen vs quoted), konstanta template/menu, dan kartu pakai. Jalur dengan
//! teks jatuh ke blok `catch` JS (`m.react('☢')` + `te(...)` = error template),
//! karena di sanalah render gagal berakhir.
//!
//! `ponytail:` kalau API canvas penuh tersedia di `Ctx` nanti, port ulang
//! `render` dari `plugins/canvas/ttqc.js` (canvas 1080x2280 @2x, PP bundar,
//! bubble putih wrap 520px, menu 6 item Font Awesome).

use oktz_plugin_api::{Ctx, Reply, plugin};

use crate::common::error_template;

/// `TEMPLATE_URL` (`ttqc.js:7`).
#[cfg(test)]
pub(crate) const TEMPLATE_URL: &str =
    "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/ttqc/qyzwa.png";

/// `FONT_ASSETS` (`ttqc.js:9-15`).
#[cfg(test)]
pub(crate) const FONT_ASSETS: [(&str, &str); 5] = [
    ("Plus Jakarta Sans", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/ttqc/PlusJakartaSans-Regular.ttf"),
    ("Plus Jakarta Sans", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/ttqc/PlusJakartaSans-Medium.ttf"),
    ("Plus Jakarta Sans", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/ttqc/PlusJakartaSans-Bold.ttf"),
    ("Font Awesome 6 Free", "https://raw.githubusercontent.com/Ditzzx-vibecoder/Assets/main/ttqc/fa-solid-900.ttf"),
    ("Noto Color Emoji", "https://github.com/googlefonts/noto-emoji/raw/main/fonts/NotoColorEmoji.ttf"),
];

/// `MENU_ICONS` (`ttqc.js:17-24`) — `(unicode, teks, warna)`.
#[cfg(test)]
pub(crate) const MENU_ICONS: [(&str, &str, &str); 6] = [
    ("\u{f3e5}", "Balas", "#000000"),
    ("\u{f064}", "Teruskan", "#000000"),
    ("\u{f0c5}", "Salin", "#000000"),
    ("\u{f1ab}", "Terjemahkan", "#000000"),
    ("\u{f2ed}", "Hapus untuk saya", "#000000"),
    ("\u{f024}", "Laporkan", "#ea4335"),
];

/// `canvasConfig` (`ttqc.js:26-33`).
#[cfg(test)]
pub(crate) const TOP_PP: (i32, i32, u32) = (183, 83, 42);
#[cfg(test)]
pub(crate) const TOP_NAME: (i32, i32, u32) = (250, 82, 34);
#[cfg(test)]
pub(crate) const CHAT_PP: (i32, u32) = (75, 38);
#[cfg(test)]
pub(crate) const TEXT_XY: (i32, i32) = (175, 962);
#[cfg(test)]
pub(crate) const BUBBLE: (u32, u32) = (520, 30);

/// Kartu pakai saat teks kosong (`ttqc.js:254-256`).
fn pakai(prefix: &str) -> String {
    format!(
        "💬 *ᴛɪᴋᴛᴏᴋ ǫᴜᴏᴛᴇ*\n\n\
         > Masukkan teks atau reply pesan yang ingin dijadikan quote.\n\n\
         `Contoh: {prefix}ttqc Hallo Dunia`"
    )
}

#[plugin(
    name = "ttqc",
    alias = ["tiktokquote", "ttq"],
    category = "canvas",
    description = "Buat fake quote chat ala TikTok",
    usage = ".ttqc <teks> (atau reply teks)",
    example = ".ttqc Hallo Dunia",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn ttqc(ctx: Ctx) -> Reply {
    // `m.args.join(" ")` — Ctx::args sudah token split whitespace.
    let mut text = ctx.args.join(" ");
    let _target_jid = ctx.sender.clone();

    // Reply pesan teks: pakai teks quote kalau argumen kosong (`ttqc.js:247-251`).
    if let Some(q) = ctx.quoted() {
        if !q.text.is_empty() {
            if text.is_empty() {
                text = q.text.clone();
            }
            // `m.quoted.pushName` tidak ada di `QuotedMessage`; nama diambil
            // dari `sock.getName`, gagal → "User".
        }
    }

    if text.is_empty() {
        return Reply::text(pakai(&ctx.prefix));
    }

    // Jalur render `@napi-rs/canvas` tidak tersedia — jatuh ke blok `catch`
    // JS: reaksi ☢ + `te(m.prefix, m.command, m.pushName)`. Lihat catatan kepala.
    Reply::text(error_template(&ctx)).with_reaction("☢")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, quoted_text: Option<&str>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ttqc".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            push_name: "Budi".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            quoted: quoted_text.map(|t| oktz_plugin_api::QuotedMessage {
                id: "Q".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: t.into(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
                timestamp: 0,
            }),
            ..Default::default()
        };
        ttqc(ctx)
    }

    #[test]
    fn meta_dan_alias() {
        let p = oktz_plugin_api::find("ttqc").unwrap();
        assert_eq!(p.meta.name, "ttqc");
        assert_eq!(p.meta.alias, ["tiktokquote", "ttq"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Buat fake quote chat ala TikTok");
        assert_eq!(p.meta.usage, ".ttqc <teks> (atau reply teks)");
        assert_eq!(p.meta.example, ".ttqc Hallo Dunia");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        for a in ["tiktokquote", "ttq"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "ttqc");
        }
    }

    #[test]
    fn konstanta_render_sama_dengan_js() {
        assert!(TEMPLATE_URL.ends_with("ttqc/qyzwa.png"));
        assert_eq!(FONT_ASSETS.len(), 5);
        assert_eq!(FONT_ASSETS[3].0, "Font Awesome 6 Free");
        assert_eq!(MENU_ICONS.len(), 6);
        // Enam item menu: lima hitam + Laporkan merah.
        assert_eq!(MENU_ICONS[5], ("\u{f024}", "Laporkan", "#ea4335"));
        assert_eq!(MENU_ICONS[0].1, "Balas");
        // canvasConfig.
        assert_eq!(TOP_PP, (183, 83, 42));
        assert_eq!(TOP_NAME, (250, 82, 34));
        assert_eq!(CHAT_PP, (75, 38));
        assert_eq!(TEXT_XY, (175, 962));
        assert_eq!(BUBBLE, (520, 30));
    }

    #[test]
    fn tanpa_teks_menampilkan_kartu_pakai() {
        let r = run("", None);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            pakai(".")
        );
        assert!(pakai(".").contains("Contoh: .ttqc Hallo Dunia"));
    }

    #[test]
    fn dengan_teks_jatuh_ke_catch_js() {
        let r = run("Hallo Dunia", None);
        // te(...) memakai template bawaan dengan pushName Budi.
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            "☢ Kayaknya command `.ttqc` lagi ada kendala\n\
             Silahkan coba lagi nanti, Budi\n\n\
             _Jika masalah berlanjut, silahkan hubungi owner bot_"
        );
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "☢"
        ));
    }

    #[test]
    fn teks_argumen_menang_dari_quote() {
        // JS: argumen ada → teks argumen yang dipakai (cabang render, tetap
        // error); argumen kosong + quote ada → teks quote.
        let r = run("argumen", Some("dari quote"));
        assert!(r.without_mutations().text_body().is_some());
        let r = run("", Some("dari quote"));
        assert!(r.without_mutations().text_body().is_some());
    }

    #[test]
    fn quote_tanpa_teks_tetap_kartu_pakai() {
        let r = run("", Some(""));
        assert_eq!(r.without_mutations().text_body().unwrap(), pakai("."));
    }
}
