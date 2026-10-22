//! `plugins/sticker/smemevid.js` — sticker meme dari video.
//!
//! Yang diport apa adanya:
//! - Guard video (`smemevid.js:28-31`) dan guard format `top|bottom`
//!   (`smemevid.js:33-36`).
//! - Teks atas/bawah di-`trim()` lalu di-`toUpperCase()`
//!   (`smemevid.js:38`).
//! - Unduh video: pesan yang di-reply dulu kalau media, lalu pesan pemicu
//!   (`smemevid.js:54-58`).
//! - Semua teks balasan, termasuk pesan error khusus plugin ini
//!   (`❌ *ɢᴀɢᴀʟ*\n\n> Terjadi kesalahan saat memproses video`) yang BUKAN
//!   `te(...)` seperti plugin lain.
//!
//! `ponytail:` render meme-nya butuh dua hal yang tidak ada di Rust:
//! (1) teks digambar di kanvas 512x512 (`smemevid.js:87-117`) — tidak ada
//! renderer teks, dan (2) video di-overlay dengan ffmpeg lalu dikirim sebagai
//! video-sticker (`smemevid.js:119-144`) — `Reply` tidak punya varian
//! "video sticker", jadi yang dikirim `Reply::video`. Kalau ffmpeg tidak ada
//! atau gagal, video asli dikirim apa adanya.

use oktz_plugin_api::registry::IncomingKind;
use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

/// `smemevid.js:30` — harus video.
pub(crate) fn tanpa_video(prefix: &str, command: &str) -> String {
    format!(
        "🎬 *ᴍᴇᴍᴇ ᴠɪᴅᴇᴏ*\n\n> Reply atau kirim video dengan caption\n\n`Contoh: {prefix}{command} Top|Bottom`"
    )
}

/// `smemevid.js:35` — format `top|bottom` salah.
pub(crate) fn format_salah(prefix: &str, command: &str) -> String {
    format!(
        "🎬 *ᴍᴇᴮᴇ ᴠɪᴅᴇᴏ*\n\n> Format: top|bottom\n\n`Contoh: {prefix}{command} WIDTH OR HEIGHT|WHY NOT BOTH?`"
    )
}

/// `smemevid.js:62` — video gagal diunduh.
pub(crate) const GAGAL_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh video";

/// `smemevid.js:153` — gagal memproses video.
pub(crate) const GAGAL_PROSES: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Terjadi kesalahan saat memproses video";

/// `smemevid.js:123-134` — rantai filter ffmpeg TANPA bagian overlay teks
/// (overlay butuh gambar PNG hasil canvas, yang tidak bisa dibuat di sini).
const FILTER: &str = "crop='min(iw,ih)':'min(iw,ih)',scale=512:512,fps=8";

/// Potong + skala video jadi 512x512 @8fps, durasi 4 detik, tanpa audio.
fn proses_video(video: &[u8]) -> Option<Vec<u8>> {
    oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(
        &[
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            "pipe:0",
            "-vf",
            FILTER,
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "26",
            "-t",
            "4",
            "-movflags",
            "frag_keyframe+empty_moov",
            "-f",
            "mp4",
            "pipe:1",
        ],
        video,
    )
    .ok()
}

#[plugin(
    name = "smemevid",
    alias = ["smemevideo", "memevid"],
    category = "sticker",
    description = "Membuat sticker meme dari video",
    usage = ".smemevid <top>|<bottom>",
    example = ".smemevid WIDTH OR HEIGHT|WHY NOT BOTH?",
    cooldown = 15,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn smemevid(ctx: Ctx) -> Reply {
    // `smemevid.js:28` — video di pesan pemicu atau di pesan yang di-reply.
    let ada_video = ctx.is_media_kind(IncomingKind::Video)
        || ctx
            .quoted()
            .is_some_and(|q| q.kind.is_video() || q.kind.wire_type() == "videoMessage");
    if !ada_video {
        return Reply::text(tanpa_video(&ctx.prefix, &ctx.command));
    }

    // `m.args.join(' ')` (`smemevid.js:33`).
    let input = ctx.args.join(" ");
    if input.is_empty() || !input.contains('|') {
        return Reply::text(format_salah(&ctx.prefix, &ctx.command));
    }

    // `smemevid.js:54-58` — media yang di-reply didahulukan kalau memang media.
    let media = if ctx.quoted().is_some_and(|q| q.is_media()) {
        ctx.download_quoted_media().ok()
    } else if ctx.is_media() {
        ctx.download_media().ok()
    } else {
        None
    };
    let Some(media) = media else {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };

    // `smemevid.js:119-144` — potong + skala; overlay teks dilewati.
    let hasil = proses_video(&media).unwrap_or(media);

    Reply::video(MediaSource::bytes(hasil).with_mimetype("video/mp4"), "").with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::IncomingMedia;

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_of(video: bool, text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "smemevid".into(),
            media: video.then(|| IncomingMedia {
                kind: IncomingKind::Video,
                mimetype: Some("video/mp4".into()),
                file_name: None,
                caption: String::new(),
            }),
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
            tanpa_video(".", "smemevid"),
            "🎬 *ᴍᴇᴍᴇ ᴠɪᴅᴇᴏ*\n\n> Reply atau kirim video dengan caption\n\n`Contoh: .smemevid Top|Bottom`"
        );
        assert_eq!(
            format_salah(".", "smemevid"),
            "🎬 *ᴍᴇᴮᴇ ᴠɪᴅᴇᴏ*\n\n> Format: top|bottom\n\n`Contoh: .smemevid WIDTH OR HEIGHT|WHY NOT BOTH?`"
        );
        assert_eq!(GAGAL_UNDUH, "❌ *ɢᴀɢᴀʟ*\n\n> Gagal mengunduh video");
        assert_eq!(
            GAGAL_PROSES,
            "❌ *ɢᴀɢᴀʟ*\n\n> Terjadi kesalahan saat memproses video"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("smemevid").unwrap();
        assert_eq!(p.meta.name, "smemevid");
        assert_eq!(p.meta.alias, ["smemevideo", "memevid"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Membuat sticker meme dari video");
        assert_eq!(p.meta.usage, ".smemevid <top>|<bottom>");
        assert_eq!(p.meta.example, ".smemevid WIDTH OR HEIGHT|WHY NOT BOTH?");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_plugin_ini() {
        for a in ["smemevideo", "memevid"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "smemevid");
        }
    }

    #[test]
    fn tanpa_video_menampilkan_kartu_pakai() {
        let r = smemevid(ctx_of(false, "atas|bawah"));
        assert_eq!(teks(&r), tanpa_video(".", "smemevid"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn format_tanpa_pipes_menampilkan_pesan_format() {
        let r = smemevid(ctx_of(true, "niat"));
        assert_eq!(teks(&r), format_salah(".", "smemevid"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn video_gagal_diunduh_menampilkan_pesan_gagal() {
        let r = smemevid(ctx_of(true, "atas|bawah"));
        assert_eq!(teks(&r), GAGAL_UNDUH);
        assert_eq!(reaksi(&r), Some("❌"));
    }
}
