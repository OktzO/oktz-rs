//! `plugins/tools/toaudio.js` — mengubah video/voice note menjadi audio MP3.
//!
//! ## Penyimpangan
//!
//! - `m.message?.audioMessage?.ptt === true` (`toaudio.js:27,40`) tidak punya
//!   padanan di `oktz-plugin-api`: `IncomingMedia` dan `QuotedMessage` tidak
//!   menyimpan flag `ptt`. Port ini memakai启发istik: audio dengan mimetype
//!   `audio/ogg` (voice note WA) dianggap PTT, audio lain dianggap file audio
//!   biasa. Efeknya: reply voice note tetap diekstrak (cabang JS untuk PTT),
//!   sedangkan reply file audio biasa memunculkan peringatan `sudah dalam
//!   format audio` (cabang JS untuk non-PTT).
//! - Ekstraksi memakai ffmpeg lewat `oktz_plugin_api::ffmpeg` dengan argumen
//!   yang sama seperti JS: `-vn -ar 44100 -ac 2 -b:a 192k`.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

/// `m.quoted` / `m` sebagai sumber media.
struct Sumber {
    /// `true` kalau video, `false` kalau audio.
    video: bool,
    /// `true` kalau voice note (PTT).
    ptt: bool,
    /// `true` kalau media ada di pesan pemicu.
    dari_pemicu: bool,
}

fn tanpa_media(prefix: &str) -> String {
    format!(
        "❌ *ɢᴀɢᴀʟ*\n\n\
         > Tidak ada video/voice note yang terdeteksi!\n\n\
         *Cara penggunaan:*\n\
         > 1. Kirim video + caption `{prefix}toaudio`\n\
         > 2. Reply video/VN dengan `{prefix}toaudio`"
    )
}

fn sudah_audio(prefix: &str) -> String {
    format!(
        "⚠️ *sᴜᴅᴀʜ ᴀᴜᴅɪᴏ*\n\n\
         > Media ini sudah dalam format audio.\n\
         > Gunakan `{prefix}tovn` jika ingin mengubah ke voice note."
    )
}

fn memproses() -> &'static str {
    "🕕 *ᴍᴇᴍᴘʀᴏsᴇs...*\n\n> Mengekstrak audio dari media..."
}

fn konversi_gagal() -> &'static str {
    "❌ *ᴋᴏɴᴠᴇʀsɪ ɢᴀɢᴀʟ*\n\n\
     > Gagal mengekstrak audio dari media.\n\
     > Pastikan ffmpeg terinstall dengan benar."
}

fn unduh_gagal() -> &'static str {
    "❌ *ɢᴀɢᴀʟ*\n\n\
     > Tidak dapat mengunduh media.\n\
     > Media mungkin sudah tidak tersedia."
}

fn error(message: &str) -> String {
    format!("❌ *ᴇʀʀᴏʀ*\n\n> Terjadi kesalahan saat memproses.\n> _{message}_")
}

/// Deteksi PTT dari mimetype — voice note WA selalu `audio/ogg`.
fn ptt_dari_mimetype(mimetype: Option<&str>) -> bool {
    mimetype.is_some_and(|m| m.to_ascii_lowercase().contains("ogg"))
}

#[plugin(
    name = "toaudio",
    alias = ["tomp3", "videotoaudio", "extractaudio"],
    category = "tools",
    description = "Mengubah video/voice note menjadi audio MP3",
    usage = ".toaudio (reply/caption video/vn)",
    example = ".toaudio",
    cooldown = 5,
    energi = 1,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn toaudio(ctx: Ctx) -> Reply {
    // `selfIsVideo` / `selfIsAudio` dicek lebih dulu (`toaudio.js:42-58`).
    let sumber = if matches!(ctx.media_type(), Some("videoMessage")) {
        Some(Sumber {
            video: true,
            ptt: false,
            dari_pemicu: true,
        })
    } else if matches!(ctx.media_type(), Some("audioMessage")) {
        let ptt = ptt_dari_mimetype(ctx.media.as_ref().and_then(|m| m.mimetype.as_deref()));
        Some(Sumber {
            video: false,
            ptt,
            dari_pemicu: true,
        })
    } else {
        ctx.quoted().map(|q| {
            let video = matches!(q.kind, oktz_plugin_api::QuotedKind::Video);
            let audio = matches!(q.kind, oktz_plugin_api::QuotedKind::Audio);
            let ptt = !video && audio && ptt_dari_mimetype(q.mimetype.as_deref());
            Sumber {
                video,
                ptt,
                dari_pemicu: false,
            }
        })
    };

    let Some(sumber) = sumber.filter(|s| s.video || s.ptt || s.dari_pemicu || true) else {
        return Reply::text(tanpa_media(&ctx.prefix));
    };

    // Kalau sumbernya audio biasa (bukan PTT), JS keluar lebih dulu.
    if !sumber.video && !sumber.ptt {
        // Audio biasa: JS tidak punya flag ptt, jadi yang bisa dipastikan
        // hanyalah "ini audio".
        let _ = sumber;
        return Reply::text(sudah_audio(&ctx.prefix));
    }

    let unduhan = if sumber.dari_pemicu {
        ctx.download_media()
    } else {
        ctx.download_quoted_media()
    };
    let buffer = match unduhan {
        Ok(b) if !b.is_empty() => b,
        _ => {
            return Reply::sequence(vec![
                Reply::text(memproses()),
                Reply::text(unduh_gagal()),
            ]);
        }
    };

    let temp_dir = "temp";
    let _ = ctx.berkas().buat_folder(temp_dir);
    let ts = crate::common::now_ms();
    let ext = if sumber.video { "mp4" } else { "ogg" };
    let input_path = format!("{temp_dir}/input_{ts}.{ext}");
    let output_path = format!("{temp_dir}/audio_{ts}.mp3");

    if ctx.berkas().tulis(&input_path, &buffer).is_err() {
        return Reply::sequence(vec![
            Reply::text(memproses()),
            Reply::text(error("Gagal menulis berkas input")),
        ]);
    }

    let ffmpeg_args = [
        "ffmpeg",
        "-y",
        "-i",
        &input_path,
        "-vn",
        "-ar",
        "44100",
        "-ac",
        "2",
        "-b:a",
        "192k",
        &output_path,
    ];

    if oktz_plugin_api::ffmpeg::jalankan_ke_berkas(&ffmpeg_args).is_err() {
        let _ = ctx.berkas().hapus(&input_path);
        return Reply::sequence(vec![
            Reply::text(memproses()),
            Reply::text(konversi_gagal()),
        ]);
    }

    if !ctx.berkas().ada(&output_path) {
        let _ = ctx.berkas().hapus(&input_path);
        return Reply::sequence(vec![
            Reply::text(memproses()),
            Reply::text(konversi_gagal()),
        ]);
    }

    let audio = match ctx.berkas().baca(&output_path) {
        Ok(b) => b,
        Err(e) => {
            let _ = ctx.berkas().hapus(&input_path);
            let _ = ctx.berkas().hapus(&output_path);
            return Reply::sequence(vec![
                Reply::text(memproses()),
                Reply::text(error(&e)),
            ]);
        }
    };

    let _ = ctx.berkas().hapus(&input_path);
    let _ = ctx.berkas().hapus(&output_path);

    Reply::sequence(vec![
        Reply::text(memproses()),
        Reply::audio(MediaSource::bytes(audio).with_mimetype("audio/mpeg")),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, QuotedKind, QuotedMessage, Reply as R};

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_kosong() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "toaudio".into(),
            ..Ctx::default()
        }
    }

    fn quoted(kind: QuotedKind, mimetype: Option<&str>) -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: String::new(),
            kind,
            mimetype: mimetype.map(str::to_string),
            file_name: None,
            from_me: false,
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_media() {
        let r = toaudio(ctx_kosong());
        assert_eq!(
            body(&r),
            "❌ *ɢᴀɢᴀʟ*\n\n\
             > Tidak ada video/voice note yang terdeteksi!\n\n\
             *Cara penggunaan:*\n\
             > 1. Kirim video + caption `.toaudio`\n\
             > 2. Reply video/VN dengan `.toaudio`"
        );
    }

    #[test]
    fn output_sama_dengan_js_audio_biasa() {
        let ctx = Ctx {
            quoted: Some(quoted(QuotedKind::Audio, Some("audio/mpeg"))),
            ..ctx_kosong()
        };
        let r = toaudio(ctx);
        assert_eq!(
            body(&r),
            "⚠️ *sᴜᴅᴀʜ ᴀᴜᴅɪᴏ*\n\n\
             > Media ini sudah dalam format audio.\n\
             > Gunakan `.tovn` jika ingin mengubah ke voice note."
        );
    }

    #[test]
    fn ptt_ditebak_dari_mimetype_ogg() {
        assert!(ptt_dari_mimetype(Some("audio/ogg; codecs=opus")));
        assert!(!ptt_dari_mimetype(Some("audio/mpeg")));
        assert!(!ptt_dari_mimetype(None));
    }

    #[test]
    fn pesan_status_dan_gagal_sama_dengan_js() {
        assert_eq!(
            memproses(),
            "🕕 *ᴍᴇᴍᴘʀᴏsᴇs...*\n\n> Mengekstrak audio dari media..."
        );
        assert_eq!(
            konversi_gagal(),
            "❌ *ᴋᴏɴᴠᴇʀsɪ ɢᴀɢᴀʟ*\n\n\
             > Gagal mengekstrak audio dari media.\n\
             > Pastikan ffmpeg terinstall dengan benar."
        );
        assert_eq!(
            unduh_gagal(),
            "❌ *ɢᴀɢᴀʟ*\n\n\
             > Tidak dapat mengunduh media.\n\
             > Media mungkin sudah tidak tersedia."
        );
    }
}
