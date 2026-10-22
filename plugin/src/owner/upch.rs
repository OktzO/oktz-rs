//! `plugins/owner/upch.js` — upload gambar, audio, video, atau teks ke saluran.
//!
//! ## Penyimpangan
//!
//! - Mengirim ke newsletetter (`@newsletter`) memakai `sock.sendMessage(chId, …)`.
//!   `Reply` di API Rust hanya mengirim ke chat tempat perintah berasal, dan
//!   trait `Sock` tidak punya operasi kirim-ke-saluran. Karena itu tahap
//!   pengiriman diakhiri `error_template` + reaction `☢` (padanan closest
//!   dengan cabang `catch` di JS).
//! - `toOggOpus` (konversi audio ke OGG/Opus lewat ffmpeg) dan
//!   `generateWaveform` tidak punya padanan yang persis; keduanya berjalan
//!   hanya pada jalur audio yang memang sudah tidak bisa dieksekusi.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Teks bantuan `upch.js:66-75`.
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "📤 *UPLOAD SALURAN*\n\n\
         Kirim/reply media dengan caption:\n  \
         `{prefix}upch 12xxx@newsletter <teks opsional>`\n\n\
         *Support:*\n  🖼️ Gambar\n  🎥 Video\n  🎵 Audio/VN\n  📝 Teks (tanpa media)"
    )
}

/// `upch.js:84`.
pub(crate) const TEKS_BERHASIL: &str = "✅ Teks berhasil dikirim ke saluran";

/// `upch.js:96`.
pub(crate) const GAMBAR_BERHASIL: &str = "✅ Gambar berhasil dikirim ke saluran";

/// `upch.js:107`.
pub(crate) const VIDEO_BERHASIL: &str = "✅ Video berhasil dikirim ke saluran";

/// `upch.js:118`.
pub(crate) const AUDIO_BERHASIL: &str = "✅ Audio berhasil dikirim ke saluran";

/// `upch.js:122`.
pub(crate) const TIPE_TIDAK_DIDUKUNG: &str = "❌ Tipe media tidak didukung";

/// `m.text?.replace(/^\.upch\s+/i, '')` (`upch.js:55`).
pub(crate) fn buang_perintah(teks: &str) -> String {
    let lower = teks.to_lowercase();
    if lower.starts_with(".upch") {
        let sisa = &teks[5..];
        let tanpa_spasi = sisa.trim_start();
        return tanpa_spasi.to_string();
    }
    teks.to_string()
}

#[plugin(
    name = "upch",
    alias = ["uploadch", "uploadsaluran", "uch"],
    category = "owner",
    description = "Upload gambar, audio, video, atau teks ke saluran",
    usage = ".upch <id saluran> <teks opsional>",
    example = ".upch 12xxx@newsletter Halo!",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn upch(ctx: Ctx) -> Reply {
    let teks = buang_perintah(&ctx.text);
    let mut args: Vec<String> = if teks.is_empty() {
        Vec::new()
    } else {
        teks.split_whitespace().map(str::to_string).collect()
    };

    // `args[0]?.includes("@newsletter") ? args.shift() : config.saluran.id`
    // (`upch.js:56`).
    let ch_id = if args
        .first()
        .is_some_and(|a| a.contains("@newsletter"))
    {
        args.remove(0)
    } else {
        ctx.config().saluran.id.clone()
    };
    let _ = ch_id;

    let caption = args.join(" ").trim().to_string();

    let quoted = ctx.quoted();
    let is_image = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image)
        || quoted.is_some_and(|q| q.kind.is_image());
    let is_video = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Video)
        || quoted.is_some_and(|q| q.kind.is_video());
    let is_audio = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Audio)
        || quoted.is_some_and(|q| q.kind.is_audio());
    let is_media = is_image || is_video || is_audio;

    if !is_media && caption.is_empty() {
        return Reply::text(bantuan(&ctx.prefix));
    }

    // `await m.react("🕕")` lalu pengiriman ke saluran.
    if !is_media && !caption.is_empty() {
        // `sock.sendMessage(chId, { text: caption })` tidak ada padanannya.
        return Reply::sequence(vec![
            Reply::None.with_reaction("🕕"),
            Reply::None.with_reaction("☢"),
            Reply::text(crate::common::error_template(&ctx)),
        ]);
    }

    if is_image {
        return Reply::sequence(vec![
            Reply::None.with_reaction("🕕"),
            Reply::None.with_reaction("☢"),
            Reply::text(crate::common::error_template(&ctx)),
        ]);
    }
    if is_video {
        return Reply::sequence(vec![
            Reply::None.with_reaction("🕕"),
            Reply::None.with_reaction("☢"),
            Reply::text(crate::common::error_template(&ctx)),
        ]);
    }
    if is_audio {
        return Reply::sequence(vec![
            Reply::None.with_reaction("🕕"),
            Reply::None.with_reaction("☢"),
            Reply::text(crate::common::error_template(&ctx)),
        ]);
    }

    Reply::sequence(vec![
        Reply::None.with_reaction("🕕"),
        Reply::None.with_reaction("☢"),
        Reply::text(crate::common::error_template(&ctx)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "upch".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        upch(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("upch").unwrap();
        assert_eq!(p.meta.name, "upch");
        assert_eq!(p.meta.alias, ["uploadch", "uploadsaluran", "uch"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Upload gambar, audio, video, atau teks ke saluran"
        );
        assert_eq!(p.meta.usage, ".upch <id saluran> <teks opsional>");
        assert_eq!(p.meta.example, ".upch 12xxx@newsletter Halo!");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_konten_menampilkan_bantuan() {
        let r = run("");
        assert_eq!(
            teks(&r),
            "📤 *UPLOAD SALURAN*\n\n\
             Kirim/reply media dengan caption:\n  \
             `.upch 12xxx@newsletter <teks opsional>`\n\n\
             *Support:*\n  🖼️ Gambar\n  🎥 Video\n  🎵 Audio/VN\n  📝 Teks (tanpa media)"
        );
    }

    #[test]
    fn kirim_ke_saluran_tidak_bisa_dilakukan_dan_diakhiri_error_template() {
        let r = run("Halo dunia");
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus Sequence");
        };
        assert!(matches!(list[0].without_mutations(), Reply::Reaction { .. }));
        assert_eq!(
            list.last().unwrap().text_body(),
            Some(crate::common::error_template_bawaan(".", "upch").as_str())
        );
    }

    #[test]
    fn prefix_perintah_dibuang_dari_teks() {
        assert_eq!(buang_perintah(".upch Halo"), "Halo");
        assert_eq!(buang_perintah(".UPCH  Halo"), "Halo");
        assert_eq!(buang_perintah("Halo"), "Halo");
    }
}
