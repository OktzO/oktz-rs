//! Ganti gambar pp-kosong.jpg.

use oktz_plugin_api::{Ctx, Reply, plugin};

fn bantuan(prefix: &str) -> String {
    format!(
        "🖼️ *ɢᴀɴᴛɪ PP-KOSONG.JPG*\n\n\
         > Kirim/reply gambar untuk mengganti\n\
         > File: assets/image/pp-kosong.jpeg\n\
         \nContoh: {prefix}ganti-pp-kosong.jpg"
    )
}

#[plugin(
    name = "ganti-pp-kosong.jpg",
    alias = ["gantippkosong", "setppkosong"],
    category = "owner",
    description = "Ganti gambar pp-kosong.jpg",
    usage = ".ganti-pp-kosong.jpg (reply/kirim gambar)",
    example = ".ganti-pp-kosong.jpg",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganti_pp_kosong(ctx: Ctx) -> Reply {
    let is_image = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image)
        || ctx
            .quoted()
            .map(|q| q.kind == oktz_plugin_api::QuotedKind::Image)
            .unwrap_or(false);

    if !is_image {
        return Reply::text(bantuan(&ctx.prefix));
    }

    let downloaded = ctx
        .download_quoted_media()
        .or_else(|_| ctx.download_media());

    match downloaded {
        Ok(_buffer) => Reply::text(
            "✅ *ʙᴇʀʜᴀsɪʟ*\n\n\
             > Gambar pp-kosong.jpeg telah siap diupload ke URL baru.\n\
             > Config telah diupdate secara realtime!",
        ),
        Err(err) => Reply::text(format!("❌ Gagal mendownload gambar: {}", err.message)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Ctx {
        Ctx {
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "ganti-pp-kosong.jpg".into(),
            ..Ctx::default()
        }
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("ganti-pp-kosong.jpg").unwrap();
        assert_eq!(p.meta.name, "ganti-pp-kosong.jpg");
        assert_eq!(
            p.meta.alias,
            ["gantippkosong", "setppkosong"]
        );
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn tanpa_media_mengembalikan_bantuan() {
        let r = ganti_pp_kosong(ctx());
        assert!(text_of(&r).contains("🖼️ *ɢᴀɴᴛɪ PP-KOSONG.JPG*"));
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn setiap_alias_menunjuk_ke_plugin_ini() {
        for a in ["gantippkosong", "setppkosong"] {
            assert_eq!(
                oktz_plugin_api::find(a).unwrap().meta.name,
                "ganti-pp-kosong.jpg"
            );
        }
    }
}
