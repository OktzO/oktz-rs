//! Sound sticker (sticker + audio), port `plugins/sticker/saudio.js`.
//!
//! Audio diunduh dari URL di teks, dikonversi ke Opus/Ogg lewat ffmpeg, lalu
//! ikut dikirim sebagai `StickerMessage.audioMessage` oleh pipeline.

use oktz_plugin_api::{
    Ctx, EXIF_EMOJIS, HttpCall, IncomingKind, MediaSource, QuotedKind, Reply, StickerOpsi,
    gambar_ke_stiker_webp, plugin, tempel_metadata_stiker, video_ke_stiker_webp,
};

use crate::sticker::sticker::metadata;

const MAKS_ANIMASI: usize = 900 * 1024;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

fn kartu_bantuan(prefix: &str) -> String {
    format!(
        "🔊 *sᴏᴜɴᴅ sᴛɪᴄᴋᴇʀ ᴍᴀᴋᴇʀ* 🎵\n\n\
         ✦ *Cara Penggunaan:*\n\
         • Kirim *foto / stiker / video* dengan caption `{prefix}sd <URL audio>`.\n\
         • Reply foto / stiker / video lalu ketik `{prefix}sd <URL audio>`.\n\
         • Kirim/reply *video* dengan caption `{prefix}sd` → audio dari videonya.\n\n\
         📌 *Contoh:*\n\
         `{prefix}sd https://cdn.example.com/sound.mp3`"
    )
}

/// Foto/video/stiker yang dikirim BERSAMA caption sama-sama jadi target,
/// seperti `const target = m.quoted || m` di JS.
fn kind_dari_media(kind: IncomingKind) -> QuotedKind {
    match kind {
        IncomingKind::Image => QuotedKind::Image,
        IncomingKind::Video => QuotedKind::Video,
        IncomingKind::Sticker => QuotedKind::Sticker,
        IncomingKind::Audio => QuotedKind::Audio,
        IncomingKind::Document => QuotedKind::Document,
    }
}

fn url_audio(teks: &str) -> Option<&str> {
    teks.split_whitespace().find_map(|k| {
        let mulai = k.find("https://").or_else(|| k.find("http://"))?;
        let url = k[mulai..].trim_matches(|c: char| {
            matches!(c, '`' | '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']')
        });
        (!url.is_empty()).then_some(url)
    })
}

/// `checkIsAnimatedWebp` (`saudio.js`): flag animasi VP8X atau chunk `ANIM`.
fn webp_beranimasi(data: &[u8]) -> bool {
    if data.len() < 30 {
        return false;
    }
    if &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        if &data[12..16] == b"VP8X" && data.len() > 20 && data[20] & 0x02 != 0 {
            return true;
        }
        if data.windows(4).any(|w| w == b"ANIM") {
            return true;
        }
    }
    false
}

fn kompres_webp(data: &[u8]) -> Result<Vec<u8>, String> {
    let keluar = oktz_plugin_api::nama_unik("oktz-saudio", "webp");
    let path = keluar.to_str().ok_or("path sementara tidak valid")?;
    let filter = "scale=384:384:force_original_aspect_ratio=decrease,fps=10,pad=384:384:-1:-1:color=white@0.0";
    let args = [
        "-hide_banner",
        "-y",
        "-loglevel",
        "error",
        "-i",
        "pipe:0",
        "-vf",
        filter,
        "-vcodec",
        "libwebp",
        "-loop",
        "0",
        "-t",
        "3",
        "-preset",
        "default",
        "-quality",
        "40",
        "-an",
        "-vsync",
        "0",
        path,
    ];
    let hasil = oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&args, data)
        .and_then(|_| std::fs::read(path).map_err(|e| e.to_string()));
    let _ = std::fs::remove_file(&keluar);
    let hasil = hasil?;
    if hasil.is_empty() {
        return Err("hasil kompres kosong".to_string());
    }
    Ok(hasil)
}

fn ke_opus(data: &[u8]) -> Result<Vec<u8>, String> {
    let args = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-i",
        "pipe:0",
        "-vn",
        "-c:a",
        "libopus",
        "-b:a",
        "64k",
        "-vbr",
        "on",
        "-compression_level",
        "10",
        "-ar",
        "48000",
        "-ac",
        "1",
        "-f",
        "ogg",
        "pipe:1",
    ];
    oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&args, data)
        .map_err(|e| format!("gagal konversi audio ke Opus: {e}"))
}

fn unduh_audio(ctx: &Ctx, url: &str) -> Option<Vec<u8>> {
    let res = ctx
        .http
        .call(&HttpCall::get(url).header("User-Agent", USER_AGENT))
        .ok()?;
    if !(200..300).contains(&res.status) {
        return None;
    }
    let data = res.body.bytes();
    (!data.is_empty()).then(|| data.to_vec())
}

fn siapkan_stiker(ctx: &Ctx, data: &[u8], kind: &QuotedKind) -> Result<(Vec<u8>, bool), String> {
    let (webp, animasi) = match kind {
        QuotedKind::Video => (
            video_ke_stiker_webp(data, &[]).or_else(|video| {
                gambar_ke_stiker_webp(data, &[])
                    .map_err(|gambar| format!("video: {video}; gambar: {gambar}"))
            })?,
            true,
        ),
        QuotedKind::Image => (
            gambar_ke_stiker_webp(data, &[]).or_else(|gambar| {
                video_ke_stiker_webp(data, &[])
                    .map_err(|video| format!("gambar: {gambar}; video: {video}"))
            })?,
            false,
        ),
        QuotedKind::Sticker => (data.to_vec(), false),
        _ => return Err("bukan stiker/foto/video".to_string()),
    };
    let (packname, author) = metadata(ctx, None, None);
    let webp = tempel_metadata_stiker(&webp, &packname, &author, &EXIF_EMOJIS).unwrap_or(webp);
    let animasi = animasi || webp_beranimasi(&webp);
    if animasi && webp.len() > MAKS_ANIMASI {
        let kecil = kompres_webp(&webp).unwrap_or(webp);
        let kecil =
            tempel_metadata_stiker(&kecil, &packname, &author, &EXIF_EMOJIS).unwrap_or(kecil);
        return Ok((kecil, animasi));
    }
    Ok((webp, animasi))
}

fn gagal(pesan: &str) -> Reply {
    Reply::text(format!("❌ *ɢᴀɢᴀʟ*\n\n> {pesan}")).with_reaction("❌")
}

#[plugin(
    name = "saudio",
    alias = ["soundsticker", "stikersuara", "sd"],
    category = "sticker",
    description = "Membuat sticker dengan audio (sound sticker)",
    usage = ".sd <url audio> atau .saudio <url audio> (kirim/reply foto, video, stiker)",
    example = ".sd https://cdn.example.com/sound.mp3",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn saudio(ctx: Ctx) -> Reply {
    let (kind, teks_target, dari_kutipan) = if let Some(q) = ctx.quoted() {
        (q.kind.clone(), q.text.clone(), true)
    } else if let Some(media) = ctx.media.as_ref() {
        (kind_dari_media(media.kind), media.caption.clone(), false)
    } else {
        return Reply::text(kartu_bantuan(&ctx.prefix));
    };

    if !matches!(
        kind,
        QuotedKind::Sticker | QuotedKind::Image | QuotedKind::Video
    ) {
        return Reply::text(
            "❌ Harap *kirim/reply* stiker (statis/bergerak), video, atau foto \
             yang ingin dipasangkan suara — dengan caption `.sd <URL audio>`!",
        );
    }

    let dari_video = matches!(kind, QuotedKind::Video);
    let url = match url_audio(&ctx.text).or_else(|| url_audio(&teks_target)) {
        Some(url) => Some(url.to_string()),
        None if dari_video => None,
        None => {
            return Reply::text(
                "❌ *URL audio belum ada*\n\n\
                 > Video: kirim/reply dengan caption `.sd` untuk memakai audio videonya.\n\
                 > Foto/stiker: kirim/reply dengan caption `.sd <URL audio>`.",
            );
        }
    };

    let unduh = if dari_kutipan {
        ctx.download_quoted_media()
    } else {
        ctx.download_media()
    };
    let Ok(raw) = unduh else {
        return gagal("Gagal mengunduh stiker/media target!");
    };
    if raw.is_empty() {
        return gagal("Gagal mengunduh stiker/media target!");
    }

    let (webp, animasi) = match siapkan_stiker(&ctx, &raw, &kind) {
        Ok(hasil) => hasil,
        Err(e) => {
            tracing::warn!(error = %e, kind = ?kind, "saudio: gagal menyiapkan WebP stiker");
            return gagal(&format!("Gagal memproses format WebP stiker: {e}"));
        }
    };

    let (audio, dari_url) = match url {
        Some(url) => match unduh_audio(&ctx, &url) {
            Some(audio) => (audio, true),
            None => return gagal("Gagal mengunduh file audio dari URL!"),
        },
        None => (raw, false),
    };
    let opus = match ke_opus(&audio) {
        Ok(opus) => opus,
        Err(e) if !dari_url => return gagal(&e),
        Err(_) => audio,
    };

    let opus = MediaSource::bytes(opus).with_mimetype("audio/ogg; codecs=opus");
    let opsi = StickerOpsi::gambar()
        .with_audio(opus)
        .with_animated(animasi);
    Reply::sticker_opsi(MediaSource::bytes(webp).with_mimetype("image/webp"), opsi)
        .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_diambil_dari_teks_apa_adanya() {
        assert_eq!(
            url_audio("https://cdn.example.com/sound.mp3"),
            Some("https://cdn.example.com/sound.mp3")
        );
        assert_eq!(
            url_audio("sd https://cdn.example.com/s.mp3 extra"),
            Some("https://cdn.example.com/s.mp3")
        );
        assert_eq!(
            url_audio("pakai `https://cdn.example.com/s.mp3` ya"),
            Some("https://cdn.example.com/s.mp3")
        );
        assert_eq!(
            url_audio("(https://cdn.example.com/s.mp3)"),
            Some("https://cdn.example.com/s.mp3")
        );
        assert_eq!(url_audio("bukan-url"), None);
    }

    #[test]
    fn media_pemicu_dipetakan_ke_kind_yang_benar() {
        assert_eq!(kind_dari_media(IncomingKind::Image), QuotedKind::Image);
        assert_eq!(kind_dari_media(IncomingKind::Video), QuotedKind::Video);
        assert_eq!(kind_dari_media(IncomingKind::Sticker), QuotedKind::Sticker);
    }

    #[test]
    fn animasi_terdeteksi_dari_flag_vp8x_dan_chunk_anim() {
        let mut vp8x = vec![0u8; 40];
        vp8x[0..4].copy_from_slice(b"RIFF");
        vp8x[8..12].copy_from_slice(b"WEBP");
        vp8x[12..16].copy_from_slice(b"VP8X");
        vp8x[20] = 0x02;
        assert!(webp_beranimasi(&vp8x));

        let mut anim = vec![0u8; 40];
        anim[0..4].copy_from_slice(b"RIFF");
        anim[8..12].copy_from_slice(b"WEBP");
        anim[20..24].copy_from_slice(b"ANIM");
        assert!(webp_beranimasi(&anim));

        assert!(!webp_beranimasi(&vp8x[..20]));
        vp8x[20] = 0;
        assert!(!webp_beranimasi(&vp8x));
    }
}
