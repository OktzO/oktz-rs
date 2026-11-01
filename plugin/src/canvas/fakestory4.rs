//! `plugins/canvas/fakestory4.js` — fake Instagram story, 2 gambar berbeda.
//!
//! Sama seperti `canvas/fakestory.rs`: render `@napi-rs/canvas` (background
//! blur dua panel, kartu, avatar, ikon Path2D) tidak dapat diport; alur
//! sebelum render diport persis JS dan langkah render jatuh ke cabang `catch`
//! (`fakestory4.js:240-243`). Helper bersama ada di `canvas/fakestory.rs`.
//! Berbeda dari fakestory: tidak ada cek `isImage` — gambar pemicu dan quoted
//! boleh berbeda (`fakestory4.js:204-214`).

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

use super::fakestory::{gagal, get_avatar, username_args};

/// `fakestory4.js:217-223` — kartu saat tidak ada gambar sama sekali.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "📷 *ꜰᴀᴋᴇ sᴛᴏʀʏ 4*\n\n\
         > Kirim/reply 1-2 gambar!\n\n\
         > Format: `{prefix}fakestory4 <nama>`\n\
         > Contoh: `{prefix}fakestory4 Misaki`\n\n\
         > Tips: Kirim gambar + reply gambar lain untuk 2 gambar berbeda"
    )
}

#[plugin(
    name = "fakestory4",
    alias = ["fstory4", "igstory4", "albumstory"],
    category = "canvas",
    description = "Fake Instagram story dengan 2 gambar berbeda",
    usage = ".fakestory4 <nama>",
    example = ".fakestory4 Misaki (reply album/2 gambar)",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn fakestory4(ctx: Ctx) -> Reply {
    let _username = username_args(&ctx);

    let Some(_avatar) = get_avatar(&ctx) else {
        return gagal(&ctx);
    };

    // `fakestory4.js:204-214` — trigger image lebih dulu (bottom = quoted
    // image bila ada, kalau tidak bottom = top); baru quoted image (bottom
    // = top). Tidak ada cek isImage di plugin ini.
    let (top, bottom) = if ctx.is_media_kind(IncomingKind::Image) {
        let top = ctx.download_media().ok();
        let bottom = if ctx.quoted().is_some_and(|q| q.kind.is_image()) {
            ctx.download_quoted_media().ok()
        } else {
            top.clone()
        };
        (top, bottom)
    } else if ctx.quoted().is_some_and(|q| q.kind.is_image()) {
        let top = ctx.download_quoted_media().ok();
        (top.clone(), top)
    } else {
        (None, None)
    };

    let Some(_top) = top else {
        return Reply::text(help(&ctx.prefix)).with_reaction("❌");
    };
    let _ = bottom;

    gagal(&ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{IncomingMedia, MediaError, MediaFetch, MediaOrigin};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    /// MediaFetch yang mencatat asal unduhan dan bisa gagal per asal.
    #[derive(Debug, Default)]
    struct Unduh {
        ok: Mutex<Vec<(MediaOrigin, bool)>>,
    }

    impl Unduh {
        fn baru() -> Arc<Self> {
            Arc::new(Self::default())
        }
        fn set(&self, origin: MediaOrigin, ok: bool) {
            self.ok.lock().unwrap().push((origin, ok));
        }
        fn hasil(&self, origin: &MediaOrigin) -> bool {
            self.ok
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|(o, _)| o == origin)
                .map(|(_, ok)| *ok)
                .unwrap_or(true)
        }
    }

    impl MediaFetch for Unduh {
        fn download(&self, origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            if self.hasil(&origin) {
                Ok(b"IMG".to_vec())
            } else {
                Err(MediaError { message: "gagal".into() })
            }
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
            command: "fakestory4".into(),
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
        (oktz_plugin_api::find("fakestory4").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("fakestory4").unwrap();
        assert_eq!(p.meta.name, "fakestory4");
        assert_eq!(p.meta.alias, ["fstory4", "igstory4", "albumstory"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Fake Instagram story dengan 2 gambar berbeda");
        assert_eq!(p.meta.usage, ".fakestory4 <nama>");
        assert_eq!(p.meta.example, ".fakestory4 Misaki (reply album/2 gambar)");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        for k in ["fakestory4", "fstory4", "igstory4", "albumstory"] {
            assert_eq!(oktz_plugin_api::find(k).unwrap().meta.name, "fakestory4");
        }
    }

    #[test]
    fn avatar_gagal_membalas_template_error_dan_radiasi() {
        let r = run(ctx_of("Misaki"));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory4"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn tanpa_gambar_membalas_help_dengan_silang() {
        // Tidak ada cek isImage: langsung cabang `!imageTopBuffer`.
        let r = run(ctx_sukses_avatar("Misaki"));
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn gambar_pemicu_jalan_ke_cabang_catch() {
        let mut ctx = ctx_sukses_avatar("Misaki");
        ctx.media = media_gambar();
        let unduh = Unduh::baru();
        ctx.media_fetch = unduh.clone();
        let r = run(ctx);
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory4"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn gambar_pemicu_tanpa_quoted_tidak_mengunduh_quoted() {
        // `fakestory4.js:204-210` — bottom = top tanpa unduhan quoted kedua.
        let mut ctx = ctx_sukses_avatar("Misaki");
        ctx.media = media_gambar();
        let unduh = Unduh::baru();
        ctx.media_fetch = unduh.clone();
        let _ = run(ctx);
        let unduhan = unduh.ok.lock().unwrap();
        // Hanya Trigger yang diminta ( quoted tidak ada → tidak dipanggil).
        assert!(unduhan.iter().all(|(o, _)| *o == MediaOrigin::Trigger));
    }

    #[test]
    fn unduhan_pemicu_gagal_membalas_help() {
        let mut ctx = ctx_sukses_avatar("Misaki");
        ctx.media = media_gambar();
        let unduh = Unduh::baru();
        unduh.set(MediaOrigin::Trigger, false);
        ctx.media_fetch = unduh;
        let r = run(ctx);
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#fakestory4 <nama>`"));
    }
}
