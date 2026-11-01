//! `plugins/canvas/fakestory2.js` — fake Instagram story, 1 gambar penuh.
//!
//! Sama seperti `canvas/fakestory.rs`: render `@napi-rs/canvas` (background
//! blur, kartu, avatar, ikon Path2D) tidak dapat diport; alur sebelum render
//! diport persis JS dan langkah render jatuh ke cabang `catch`
//! (`fakestory2.js:208-211`). Helper bersama ada di `canvas/fakestory.rs`.

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

use super::fakestory::{gagal, get_avatar, is_image, username_args};

/// `fakestory2.js:182-187` — kartu saat pesan bukan gambar.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "📷 *ꜰᴀᴋᴇ sᴛᴏʀʏ 2*\n\n\
         > Reply gambar!\n\n\
         > Format: `{prefix}fakestory2 <nama>`\n\
         > Contoh: `{prefix}fakestory2 Misaki`"
    )
}

/// `fakestory2.js:197` — unduhan gambar kosong/gagal.
pub(crate) const GAGAL_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa download gambar";

#[plugin(
    name = "fakestory2",
    alias = ["fstory2", "igstory2"],
    category = "canvas",
    description = "Fake Instagram story dengan 1 gambar full",
    usage = ".fakestory2 <nama>",
    example = ".fakestory2 Misaki (reply gambar)",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn fakestory2(ctx: Ctx) -> Reply {
    let _username = username_args(&ctx);

    let Some(_avatar) = get_avatar(&ctx) else {
        return gagal(&ctx);
    };

    if !is_image(&ctx) {
        return Reply::text(help(&ctx.prefix)).with_reaction("❌");
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
    use oktz_plugin_api::{
        HttpBody, HttpCall, HttpError, HttpResponse, IncomingMedia, MediaError, MediaFetch,
        MediaOrigin,
    };
    use serde_json::Value;
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubHttp {
        pp: Option<Vec<u8>>,
    }

    impl oktz_plugin_api::HttpGet for StubHttp {
        fn call(&self, _: &HttpCall) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma get")
        }
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            let isi = if url.contains("pp.example") {
                self.pp.clone().unwrap_or_default()
            } else {
                Vec::new()
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(isi),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_multipart(&self, _: &str, _: &[(&str, HttpBody)]) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
    }

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

    fn ctx_sukses_avatar(text: &str) -> Ctx {
        let mut ctx = Ctx {
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "fakestory2".into(),
            ..Ctx::default()
        };
        ctx.http = crate::canvas::fakestory::tests::http_pp_example();
        ctx.sock = crate::canvas::fakestory::tests::sock_pp_example();
        ctx
    }

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("fakestory2").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("fakestory2").unwrap();
        assert_eq!(p.meta.name, "fakestory2");
        assert_eq!(p.meta.alias, ["fstory2", "igstory2"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Fake Instagram story dengan 1 gambar full");
        assert_eq!(p.meta.usage, ".fakestory2 <nama>");
        assert_eq!(p.meta.example, ".fakestory2 Misaki (reply gambar)");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        for k in ["fakestory2", "fstory2", "igstory2"] {
            assert_eq!(oktz_plugin_api::find(k).unwrap().meta.name, "fakestory2");
        }
    }

    #[test]
    fn avatar_gagal_membalas_template_error_dan_radiasi() {
        let r = run(Ctx {
            text: "Misaki".into(),
            args: vec!["Misaki".into()],
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "fakestory2".into(),
            ..Ctx::default()
        });
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory2"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn bukan_gambar_membalas_help_dengan_silang() {
        let r = run(ctx_sukses_avatar("Misaki"));
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn gambar_tersedia_jalan_ke_cabang_catch() {
        let mut ctx = ctx_sukses_avatar("Misaki");
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Ok(b"IMG".to_vec())));
        let r = run(ctx);
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory2"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn unduhan_gagal_membalas_pesan_gagal_unduh() {
        let mut ctx = ctx_sukses_avatar("Misaki");
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Err(MediaError { message: "gagal".into() })));
        let r = run(ctx);
        assert_eq!(teks(&r), GAGAL_UNDUH);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#fakestory2 <nama>`"));
    }
}
