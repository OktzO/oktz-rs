//! `plugins/canvas/fakestory3.js` — fake Instagram story dengan text overlay.
//!
//! Sama seperti `canvas/fakestory.rs`: render `@napi-rs/canvas` (background
//! blur, dua panel gambar dengan teks ber-shadow yang di-wrap `measureText`)
//! tidak dapat diport; alur sebelum render diport persis JS dan langkah render
//! jatuh ke cabang `catch` (`fakestory3.js:294-297`). Helper bersama ada di
//! `canvas/fakestory.rs`.

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

use super::fakestory::{gagal, get_avatar, is_image};

/// `fakestory3.js:247-253` — kartu saat input kosong / tanpa pipa.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "📷 *ꜰᴀᴋᴇ sᴛᴏʀʏ 3*\n\n\
         > Reply gambar dengan format:\n\
         > `{prefix}fakestory3 nama|text1|text2`\n\n\
         > Contoh:\n\
         > `{prefix}fakestory3 Misaki|Tersenyumlah|untuk menutupi kesedihan`"
    )
}

/// `fakestory3.js:265` — pesan bukan gambar.
pub(crate) const BUKAN_GAMBAR: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Reply gambar untuk membuat fake story!";

/// `fakestory3.js:275` — unduhan gambar kosong/gagal.
pub(crate) const GAGAL_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa download gambar";

/// `fakestory3.js:245-258` — `input.split("|").map(trim)`; username =
/// `parts[0] || m.pushName || "User"` (part kosong jatuh ke pushName);
/// `text1`/`text2` kosong bila tidak ada.
pub(crate) fn pisah_input(ctx: &Ctx) -> (String, String, String) {
    let input = ctx.args.join(" ");
    let mut bagian = input.split('|').map(str::trim);
    let username = bagian.next().unwrap_or("");
    let username = if username.is_empty() {
        if ctx.push_name.is_empty() {
            "User".to_string()
        } else {
            ctx.push_name.clone()
        }
    } else {
        username.to_string()
    };
    let text1 = bagian.next().unwrap_or("").to_string();
    let text2 = bagian.next().unwrap_or("").to_string();
    (username, text1, text2)
}

#[plugin(
    name = "fakestory3",
    alias = ["fstory3", "igstory3", "quotestory"],
    category = "canvas",
    description = "Fake Instagram story dengan text overlay",
    usage = ".fakestory3 <nama>|<text1>|<text2>",
    example = ".fakestory3 Misaki|Tersenyumlah|untuk menutupi kesedihan",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn fakestory3(ctx: Ctx) -> Reply {
    // `if (!input || !input.includes("|"))` (`fakestory3.js:246`).
    let input = ctx.args.join(" ");
    if input.is_empty() || !input.contains('|') {
        return Reply::text(help(&ctx.prefix));
    }
    let (_username, _text1, _text2) = pisah_input(&ctx);

    let Some(_avatar) = get_avatar(&ctx) else {
        return gagal(&ctx);
    };

    if !is_image(&ctx) {
        return Reply::text(BUKAN_GAMBAR).with_reaction("❌");
    }

    let gambar = if ctx.is_media_kind(IncomingKind::Image) {
        ctx.download_media().ok()
    } else if ctx.quoted().is_some_and(|q| q.kind.is_image()) {
        ctx.download_quoted_media().ok()
    } else {
        None
    };
    let Some(_gambar) = gambar else {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };

    gagal(&ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{IncomingMedia, MediaError, MediaFetch, MediaOrigin};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Unduh(Result<Vec<u8>, MediaError>);

    impl MediaFetch for Unduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.0.clone()
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

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "fakestory3".into(),
            ..Ctx::default()
        }
    }

    fn ctx_sukses_avatar(text: &str) -> Ctx {
        let mut ctx = ctx_of(text);
        ctx.http = crate::canvas::fakestory::tests::http_pp_example();
        ctx.sock = crate::canvas::fakestory::tests::sock_pp_example();
        ctx
    }

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("fakestory3").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("fakestory3").unwrap();
        assert_eq!(p.meta.name, "fakestory3");
        assert_eq!(p.meta.alias, ["fstory3", "igstory3", "quotestory"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Fake Instagram story dengan text overlay");
        assert_eq!(p.meta.usage, ".fakestory3 <nama>|<text1>|<text2>");
        assert_eq!(p.meta.example, ".fakestory3 Misaki|Tersenyumlah|untuk menutupi kesedihan");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        for k in ["fakestory3", "fstory3", "igstory3", "quotestory"] {
            assert_eq!(oktz_plugin_api::find(k).unwrap().meta.name, "fakestory3");
        }
    }

    #[test]
    fn tanpa_pipa_atau_kosong_membalas_help_tanpa_reaksi() {
        for text in ["", "Misaki"] {
            let r = run(ctx_of(text));
            assert_eq!(teks(&r), help("."), "text {text:?}");
            assert_eq!(reaksi(&r), None);
        }
    }

    #[test]
    fn pisah_input_mengikuti_js() {
        let mut ctx = ctx_of("Misaki|Tersenyumlah|untuk menutupi kesedihan");
        assert_eq!(
            pisah_input(&ctx),
            (
                "Misaki".into(),
                "Tersenyumlah".into(),
                "untuk menutupi kesedihan".into()
            )
        );
        // Part kosong jatuh ke pushName → "User".
        ctx = ctx_of("|a|b");
        assert_eq!(pisah_input(&ctx).0, "User");
        // pushName terisi dipakai.
        ctx.push_name = "Push".into();
        assert_eq!(pisah_input(&ctx).0, "Push");
        // Teks ketiga dan seterusnya dibuang.
        ctx = ctx_of("a|b|c|d");
        let (_, t1, t2) = pisah_input(&ctx);
        assert_eq!((t1.as_str(), t2.as_str()), ("b", "c"));
    }

    #[test]
    fn bukan_gambar_membalas_pesan_bukan_gambar() {
        let r = run(ctx_sukses_avatar("Misaki|a|b"));
        assert_eq!(teks(&r), BUKAN_GAMBAR);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn gambar_tersedia_jalan_ke_cabang_catch() {
        let mut ctx = ctx_sukses_avatar("Misaki|a|b");
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Ok(b"IMG".to_vec())));
        let r = run(ctx);
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory3"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn unduhan_gagal_membalas_pesan_gagal_unduh() {
        let mut ctx = ctx_sukses_avatar("Misaki|a|b");
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Err(MediaError { message: "gagal".into() })));
        let r = run(ctx);
        assert_eq!(teks(&r), GAGAL_UNDUH);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#fakestory3 nama|text1|text2`"));
    }
}
