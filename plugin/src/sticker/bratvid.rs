//! `plugins/sticker/bratvid.js` — animated brat (simplified native port).

use std::process::{Command, Stdio};
use std::time::Duration;

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

fn render_raw(teks: &str, width: u32, height: u32) -> Vec<u8> {
    oktz_plugin_api::render_frame_raw(teks, width, height)
}

fn encode_mp4(raw: &[u8], width: u32, height: u32, fps: u32) -> Result<Vec<u8>, String> {
    use std::io::Write;
    encode_mp4_dengan(width, height, fps, |stdin| stdin.write_all(raw))
}

/// Alirkan frame ke ffmpeg lalu baca hasilnya dari file temp.
///
/// Output `pipe:1` tidak bisa dipakai: muxer MP4 butuh seekable
/// (`muxer does not support non seekable output`) — JS juga menulis ke
/// berkas temp (`bratvid2.js`). Frame dialirkan, bukan ditampung, supaya
/// animasi 1000×1000 tidak menelan ratusan MB RAM.
fn encode_mp4_dengan<F>(
    width: u32,
    height: u32,
    fps: u32,
    mut isi: F,
) -> Result<Vec<u8>, String>
where
    F: FnMut(&mut std::process::ChildStdin) -> std::io::Result<()>,
{
    let ffmpeg = oktz_plugin_api::ffmpeg::jalur_ffmpeg().unwrap_or_else(|| std::path::PathBuf::from("ffmpeg"));
    let keluar = oktz_plugin_api::nama_unik("oktz-bratvid", "mp4");
    let keluar_arg = keluar.to_string_lossy().to_string();
    let mut child = Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &format!("{width}x{height}"),
            "-r",
            &fps.to_string(),
            "-i",
            "pipe:0",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
            &keluar_arg,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("gagal menjalankan ffmpeg: {e}"))?;

    let tulis = match child.stdin.take() {
        Some(mut stdin) => isi(&mut stdin).map_err(|e| format!("gagal menulis frame: {e}")),
        None => Err("stdin ffmpeg tidak tersedia".to_string()),
    };

    let mulai = std::time::Instant::now();
    loop {
        match child.try_wait().map_err(|e| format!("gagal menunggu ffmpeg: {e}"))? {
            Some(_) => break,
            None if mulai.elapsed() > Duration::from_secs(120) => {
                let _ = child.kill();
                let _ = std::fs::remove_file(&keluar);
                return Err("ffmpeg timeout".to_string());
            }
            None => std::thread::sleep(Duration::from_millis(25)),
        }
    }
    let keluaran = child
        .wait_with_output()
        .map_err(|e| format!("gagal menunggu ffmpeg: {e}"))?;
    if let Err(e) = tulis {
        let _ = std::fs::remove_file(&keluar);
        return Err(e);
    }
    if !keluaran.status.success() {
        let _ = std::fs::remove_file(&keluar);
        return Err(String::from_utf8_lossy(&keluaran.stderr).to_string());
    }
    let hasil = std::fs::read(&keluar).map_err(|e| format!("gagal baca mp4: {e}"));
    let _ = std::fs::remove_file(&keluar);
    hasil
}

pub fn render_frame_bratvid2(teks: &str, _frame: u32, _total: u32) -> Vec<u8> {
    render_raw(teks, 1000, 1000)
}

pub fn render_frame_bratlocal(teks: &str, _frame: u32, _total: u32) -> Vec<u8> {
    render_raw(teks, 512, 512)
}

fn kartu_pakai(ctx: &Ctx, cmd: &str) -> Reply {
    Reply::text(format!(
        "🎬 *ʙʀᴀᴛ ᴀɴɪᴍᴀᴛᴇᴅ*\n\n> Masukkan teks\n\n`Contoh: {}{cmd} Hai semua`",
        ctx.prefix
    ))
}

fn gagal(ctx: &Ctx) -> Reply {
    Reply::sequence(vec![
        Reply::text(crate::common::error_template(ctx)),
        Reply::Reaction {
            emoji: "☢".into(),
            reply: Box::new(Reply::None),
        },
    ])
}

/// MP4 → WebP animasi ber-EXIF sebagai sticker. JS:
/// `sock.sendVideoAsSticker(...)`, bukan video biasa.
fn jadi_stiker_video(ctx: &Ctx, mp4: Vec<u8>) -> Reply {
    let cfg = ctx.config();
    let (packname, author) = (
        if cfg.sticker.packname.is_empty() { "Sticker".to_string() } else { cfg.sticker.packname.clone() },
        if cfg.sticker.author.is_empty() { "Bot".to_string() } else { cfg.sticker.author.clone() },
    );
    let webp = match oktz_plugin_api::video_ke_stiker_webp(&mp4, &[]) {
        Ok(webp) => oktz_plugin_api::tempel_metadata_stiker(&webp, &packname, &author, &oktz_plugin_api::EXIF_EMOJIS).unwrap_or(webp),
        Err(_) => return gagal(ctx),
    };
    Reply::sequence(vec![
        Reply::sticker_opsi(
            MediaSource::bytes(webp).with_mimetype("image/webp"),
            oktz_plugin_api::StickerOpsi::video().with_packname(packname).with_author(author),
        ),
        Reply::Reaction {
            emoji: "✅".into(),
            reply: Box::new(Reply::None),
        },
    ])
}

fn run(ctx: Ctx, width: u32, height: u32, fps: u32) -> Reply {
    let text = ctx.text.trim().to_string();
    if text.is_empty() {
        return kartu_pakai(&ctx, "bratvid");
    }
    let Ok(mp4) = encode_mp4(&render_raw(&text, width, height), width, height, fps) else {
        return gagal(&ctx);
    };
    jadi_stiker_video(&ctx, mp4)
}

/// `.bratvid2` — kata muncul satu per satu (port `bratvid2.js`).
fn run_animasi(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    if text.is_empty() {
        return kartu_pakai(&ctx, "bratvid2");
    }
    let Ok(animasi) = oktz_plugin_api::BratAnimasi::baru(text) else {
        return gagal(&ctx);
    };
    let Ok(mp4) = encode_mp4_dengan(1000, 1000, oktz_plugin_api::BRAT_ANIM_FPS, |stdin| {
        use std::io::Write;
        for i in 0..animasi.total_frame() {
            stdin.write_all(&animasi.frame_rgba(i))?;
        }
        Ok(())
    }) else {
        return gagal(&ctx);
    };
    jadi_stiker_video(&ctx, mp4)
}


fn run_progresif(ctx: Ctx) -> Reply {
    let text = ctx.text.trim().to_string();
    if text.is_empty() {
        return kartu_pakai(&ctx, "bratvid");
    }
    let Ok(progresif) = oktz_plugin_api::BratProgresif::baru(&text) else {
        return gagal(&ctx);
    };
    let total_kata = progresif.jumlah_kata();
    let Ok(mp4) = encode_mp4_dengan(1000, 1000, oktz_plugin_api::BRAT_PROGRESIF_FPS, |stdin| {
        use std::io::Write;
        for kata_ke in 1..=total_kata {
            let raw = progresif
                .frame_rgba(kata_ke)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let ulang = if kata_ke == total_kata {
                oktz_plugin_api::BRAT_PROGRESIF_DURASI_AKHIR
            } else {
                oktz_plugin_api::BRAT_PROGRESIF_DURASI
            };
            for _ in 0..ulang {
                stdin.write_all(&raw)?;
            }
        }
        Ok(())
    }) else {
        return gagal(&ctx);
    };
    jadi_stiker_video(&ctx, mp4)
}

#[plugin(
    name = "bratvid",
    alias = ["bratgif", "bratvideo"],
    category = "sticker",
    description = "Membuat sticker brat animated",
    usage = ".bratvid <text>",
    example = ".bratvid Hai semua",
    cooldown = 15,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn bratvid(ctx: Ctx) -> Reply {
    run_progresif(ctx)
}

#[plugin(
    name = "bratvid2",
    alias = ["bratv2"],
    category = "sticker",
    description = "Sticker brat video v2",
    usage = ".bratvid2 <text>",
    example = ".bratvid2 Hai semua",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn bratvid2(ctx: Ctx) -> Reply {
    run_animasi(ctx)
}

#[plugin(
    name = "bratlocal",
    alias = ["bratgojo", "bratgojovid", "bratvermeil", "bratvermeilvid"],
    category = "sticker",
    description = "Bikin brat versi lokal",
    usage = ".bratlocal <text>",
    example = ".bratlocal Hai",
    cooldown = 15,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn bratlocal(ctx: Ctx) -> Reply {
    run(ctx, 512, 512, 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_renderers_are_nonempty() {
        assert!(!render_frame_bratvid2("halo", 0, 1).is_empty());
        assert!(!render_frame_bratlocal("halo", 0, 1).is_empty());
    }
}
