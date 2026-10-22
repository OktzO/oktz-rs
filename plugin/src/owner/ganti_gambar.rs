//! Helper bersama untuk plugin `ganti-*.jpg` / `ganti-mp4.mp4`.
//!
//! Seluruh plugin itu punya bentuk yang sama: cek jenis media, unduh
//! buffer, balas "⏳ Sedang mengupload gambar...", lalu tulis ulang berkas
//! aset dan laporkan jalurnya.
//!
//! `updateAssetUrl` (`src/lib/uploader.js`) menulis buffer ke
//! `config.assets[key]` setelah di-resize ke JPEG 1280x720 (`fit: inside`,
//! kualitas 80) dan mengembalikan jalurnya. Di Rust penulisan berkas
//! dilakukan lewat `ctx.berkas()` (berakar ke direktori kerja bot, sama
//! seperti `process.cwd()` di JS), dan resize memakai
//! `oktz_plugin_api::gambar::sharp`.
//!
//! `ponytail:` `fit: inside` murni belum ada di `gambar` — `ubah_ukuran`
//! memotong (cover) lebih dulu, lalu `pas_di_dalam()` mengecilkan ke dalam
//! 1280x720. Hasil gambarnya sedikit berbeda dari JS, teks balasannya tidak.

use oktz_plugin_api::{Ctx, IncomingKind, Reply};

/// Jenis media yang boleh diunggah plugin ini.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Jenis {
    Gambar,
    Video,
}

/// Parameter yang berbeda tiap plugin.
pub(crate) struct Aset {
    /// Emoji pada judul kartu "pakai".
    pub emoji_judul: &'static str,
    /// Judul kartu "pakai" (huruf kecil kecil).
    pub judul: &'static str,
    /// Path aset yang ditampilkan di kartu "pakai".
    pub file_tampil: &'static str,
    /// Kunci aset di `config.assets`.
    pub key: &'static str,
    /// Nama berkas tujuan.
    pub nama_file: &'static str,
    /// Kata pada pesan sukses ("Gambar" / "File").
    pub label_sukses: &'static str,
    /// Kata pada pesan gagal unggah ("gambar" / "file").
    pub label_error: &'static str,
    /// Jenis media yang diterima.
    pub jenis: Jenis,
}

/// Kartu "pakai" — `m.reply(...)` saat bukan media yang benar.
pub(crate) fn pakai(a: &Aset) -> String {
    format!(
        "{} *{}*\n\n> Kirim/reply {} untuk mengganti\n> File: {}",
        a.emoji_judul,
        a.judul,
        if a.jenis == Jenis::Gambar {
            "gambar"
        } else {
            "video"
        },
        a.file_tampil
    )
}

/// Pesan gagal unduh — `❌ Gagal mendownload gambar` / `... video`.
pub(crate) fn gagal_unduh(a: &Aset) -> String {
    format!(
        "❌ Gagal mendownload {}",
        if a.jenis == Jenis::Gambar {
            "gambar"
        } else {
            "video"
        }
    )
}

/// Pesan "sedang mengupload" — di JS teksnya selalu "gambar", termasuk
/// untuk plugin video (`ganti-mp4.js:38`).
pub(crate) const SEDANG_UPLOAD: &str = "⏳ Sedang mengupload gambar...";

/// Pesan sukses `ganti-*.js`.
pub(crate) fn sukses(a: &Aset, jalur: &str) -> String {
    format!(
        "✅ *ʙᴇʀʜᴀsɪʟ*\n\n> {} {} telah diganti ke URL baru:\n> {}\n> Config telah diupdate secara realtime!",
        a.label_sukses, a.nama_file, jalur
    )
}

/// Pesan gagal unggah.
pub(crate) fn gagal_upload(a: &Aset, pesan: &str) -> String {
    format!("❌ Gagal mengupload {}: {pesan}", a.label_error)
}

/// Apakah pemicu atau quoted berupa media yang diterima.
fn cocok(ctx: &Ctx, jenis: Jenis) -> bool {
    match jenis {
        Jenis::Gambar => {
            ctx.is_media_kind(IncomingKind::Image)
                || ctx.quoted().is_some_and(|q| q.kind.is_image())
        }
        Jenis::Video => {
            ctx.is_media_kind(IncomingKind::Video)
                || ctx.quoted().is_some_and(|q| q.kind.is_video())
        }
    }
}

/// Unduh buffer: quoted dulu, baru pesan pemicu.
fn ambil_buffer(ctx: &Ctx) -> Option<Vec<u8>> {
    if ctx.quoted().is_some_and(|q| q.is_media()) {
        return ctx.download_quoted_media().ok();
    }
    if ctx.is_media() {
        return ctx.download_media().ok();
    }
    None
}

/// Jalur aset tujuan: `config.assets[key]`, atau fallback JS
/// (`./assets/<folder>/<file>`).
pub(crate) fn jalur_aset(ctx: &Ctx, a: &Aset) -> String {
    if let Some(p) = ctx.config().assets.get(a.key) {
        if !p.is_empty() {
            return p.to_string();
        }
    }
    let folder = if a.nama_file.ends_with(".mp4") {
        "video"
    } else if a.nama_file.ends_with(".mp3") {
        "audio"
    } else {
        "image"
    };
    format!("./assets/{folder}/{}", a.nama_file)
}

/// Resize ke JPEG 1280x720 (`sharp(buffer).resize(1280, 720, { fit: 'inside' })`).
///
/// Kalau gagal, buffer asli dipakai — sama seperti `catch` di JS.
fn perkecil_jpeg(buffer: &[u8]) -> Vec<u8> {
    match oktz_plugin_api::gambar::sharp(buffer) {
        Ok(gambar) => gambar
            .ubah_ukuran(1280, 720)
            .pas_di_dalam()
            .ke_jpeg(80)
            .unwrap_or_else(|_| buffer.to_vec()),
        Err(_) => buffer.to_vec(),
    }
}

/// Alur lengkap satu plugin `ganti-*`.
pub(crate) fn jalankan(ctx: &Ctx, a: &Aset) -> Reply {
    if !cocok(ctx, a.jenis) {
        return Reply::text(pakai(a));
    }

    let Some(buffer) = ambil_buffer(ctx) else {
        return Reply::text(gagal_unduh(a));
    };

    let jalur = jalur_aset(ctx, a);
    let isi = if a.nama_file.ends_with(".jpeg") || a.nama_file.ends_with(".jpg") {
        perkecil_jpeg(&buffer)
    } else {
        buffer
    };

    match ctx.berkas().tulis(&jalur, &isi) {
        Ok(()) => Reply::sequence(vec![
            Reply::text(SEDANG_UPLOAD),
            Reply::text(sukses(a, &jalur)),
        ]),
        Err(e) => Reply::sequence(vec![
            Reply::text(SEDANG_UPLOAD),
            Reply::text(gagal_upload(a, &e)),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASET_GAMBAR: Aset = Aset {
        emoji_judul: "🖼️",
        judul: "ɢᴀɴᴛɪ ᴅᴇᴍᴏᴛᴇ.ᴊᴘɢ",
        file_tampil: "assets/image/demote.jpeg",
        key: "demote",
        nama_file: "demote.jpeg",
        label_sukses: "Gambar",
        label_error: "gambar",
        jenis: Jenis::Gambar,
    };

    #[test]
    fn kartu_pakai_pola_js() {
        assert_eq!(
            pakai(&ASET_GAMBAR),
            "🖼️ *ɢᴀɴᴛɪ ᴅᴇᴍᴏᴛᴇ.ᴊᴘɢ*\n\n\
             > Kirim/reply gambar untuk mengganti\n\
             > File: assets/image/demote.jpeg"
        );
    }

    #[test]
    fn pesan_sukses_pola_js() {
        assert_eq!(
            sukses(&ASET_GAMBAR, "./assets/image/demote.jpeg"),
            "✅ *ʙᴇʀʜᴀsɪʟ*\n\n\
             > Gambar demote.jpeg telah diganti ke URL baru:\n\
             > ./assets/image/demote.jpeg\n\
             > Config telah diupdate secara realtime!"
        );
    }

    #[test]
    fn pesan_gagal_pola_js() {
        assert_eq!(
            gagal_upload(&ASET_GAMBAR, "boom"),
            "❌ Gagal mengupload gambar: boom"
        );
        assert_eq!(gagal_unduh(&ASET_GAMBAR), "❌ Gagal mendownload gambar");
    }
}
