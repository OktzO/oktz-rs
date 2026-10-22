//! `plugins/sticker/bratvid2.js` — brat video v2 (animasi per-kata + emoji Apple).
//!
//! Yang diport apa adanya:
//! - Kartu pakai (`bratvid2.js:568-572`) — memakai `m.prefix` + `m.command`.
//! - `m.args.join(" ")` sebagai teks (`bratvid2.js:566`).
//! - Reaksi penutup `✅` / `☢` dan pesan `te(...)`.
//!
//! `ponytail:` yang tidak bisa ditiru: `generateBratVideo`
//! (`bratvid2.js:328-547`) merender 60 bingkai 1000x1000 di kanvas, menyalakan
//! animasi per-kata (skala + alpha + sapuan cahaya), lalu mengalirkan RGBA
//! mentah ke ffmpeg lewat `pipe:0`. Butuh `@napi-rs/canvas`, font Impact, dan
//! peta emoji Apple — tidak ada renderer teks di Rust dan dependensi repo
//! tidak boleh diubah. Jadi jalur setelah guard teks berakhir di cabang
//! `catch` JS: `error_template` + `☢` (`bratvid2.js:597-601`), sama seperti
//! `sticker/bratvid.rs`.
//!
//! `packname`/`author` (`config.sticker`) juga tidak punya kanal di
//! `Reply::Video`.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `bratvid2.js:570` — kartu pakai.
pub(crate) fn pakai(prefix: &str, command: &str) -> String {
    format!(
        "🎬 *ʙʀᴀᴛ ᴠɪᴅᴇᴏ ᴠ2*\n\n> Masukkan teks\n\n`Contoh: {prefix}{command} hello world`"
    )
}

#[plugin(
    name = "bratvid2",
    alias = ["bratv2"],
    category = "sticker",
    description = "Generate brat video v2 (animasi per-kata + emoji Apple)",
    usage = ".bratvid2 <text>",
    example = ".bratvid2 hello world",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bratvid2(ctx: Ctx) -> Reply {
    // `m.args.join(" ")` (`bratvid2.js:566`).
    let text = ctx.args.join(" ");
    if text.is_empty() {
        return Reply::text(pakai(&ctx.prefix, &ctx.command));
    }

    // Render animasi butuh canvas — lihat `ponytail:` modul.
    Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_of(command: &str, text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: command.to_string(),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or("").to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        assert_eq!(
            pakai(".", "bratvid2"),
            "🎬 *ʙʀᴀᴛ ᴠɪᴅᴇᴏ ᴠ2*\n\n> Masukkan teks\n\n`Contoh: .bratvid2 hello world`"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("bratvid2").unwrap();
        assert_eq!(p.meta.name, "bratvid2");
        assert_eq!(p.meta.alias, ["bratv2"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(
            p.meta.description,
            "Generate brat video v2 (animasi per-kata + emoji Apple)"
        );
        assert_eq!(p.meta.usage, ".bratvid2 <text>");
        assert_eq!(p.meta.example, ".bratvid2 hello world");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn alias_bratv2_menunjuk_plugin_ini() {
        assert_eq!(oktz_plugin_api::find("bratv2").unwrap().meta.name, "bratvid2");
    }

    #[test]
    fn tanpa_teks_menampilkan_kartu_pakai_tanpa_reaksi() {
        let r = bratvid2(ctx_of("bratvid2", ""));
        assert_eq!(teks(&r), pakai(".", "bratvid2"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn dengan_teks_jatuh_ke_error_template_dan_reaksi_bahu() {
        let r = bratvid2(ctx_of("bratvid2", "hello world"));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "bratvid2"));
        assert_eq!(reaksi(&r), Some("☢"));
    }
}
