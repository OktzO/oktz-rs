//! Helper bersama untuk plugin `ganti-*` (keluarga `owner`).
//!
//! Plugin JS `ganti-foto.js`, `ganti-foto2.js`, `ganti-goodbye.js`,
//! `ganti-mp3.js`, `ganti-pp-kosong.js`, `ganti-rpg.js`, `ganti-store.js`, dan
//! `ganti-welcome.js` semuanya memanggil `updateAssetUrl(key, buffer, filename)`
//! dari `src/lib/uploader.js`. Fungsi itu:
//!
//! 1. menentukan `localPath` — pakai `config.assets[key]` kalau ada dan bukan
//!    URL, kalau tidak `./assets/<folder>/<filename>` (`uploader.js:62-93`),
//! 2. menulis berkas di `path.resolve(process.cwd(), localPath)` dan membuat
//!    foldernya kalau perlu (`uploader.js:95-97`),
//! 3. mengubah gambar ke JPEG 1280x720 (`fit: inside`) kualitas 80 kalau
//!    ekstensinya `.jpeg`/`.jpg` (`uploader.js:99-110`),
//! 4. mengembalikan `localPath` sebagai "URL baru".
//!
//! Di Rust `ctx.berkas()` berakar ke direktori kerja — sama dengan
//! `process.cwd()` — jadi langkah 1-4 bisa ditiru persis. Yang tidak bisa:
//! penulisan ulang `config.js` (langkah 1 JS) dan unggah ke CDN
//! (`updateAssetAndSave`), karena `config` di `Ctx` bersifat `Arc` (tidak bisa
//! dimutasi) dan tidak ada API unggah. Jalur aset di `config.assets` dibaca
//! apa adanya, jadi hasil `localPath` sama dengan yang akan dipakai JS.

use oktz_plugin_api::Ctx;

/// Path aset lokal seperti yang dikembalikan JS (`uploader.js:114`).
///
/// Mengembalikan string dengan awalan `./` — persis bentuk `localPath` JS.
pub(crate) fn path_aset(ctx: &Ctx, asset_key: &str, filename: &str) -> String {
    let configured = ctx.config().assets.get(asset_key).unwrap_or_default();
    if configured.is_empty() || configured.starts_with("http") {
        let folder = if filename.ends_with(".mp4") {
            "video"
        } else if filename.ends_with(".mp3") {
            "audio"
        } else {
            "image"
        };
        format!("./assets/{folder}/{filename}")
    } else {
        configured.to_string()
    }
}

/// Path relatif untuk `ctx.berkas()` — tanpa awalan `./`.
pub(crate) fn path_berkas(local_path: &str) -> String {
    local_path.strip_prefix("./").unwrap_or(local_path).to_string()
}

/// Tulis buffer ke berkas aset dan kembalikan `localPath`.
///
/// Mengembalikan `Err` dengan pesan singkat kalau penulisan gagal.
pub(crate) fn tulis_aset(
    ctx: &Ctx,
    asset_key: &str,
    filename: &str,
    buffer: &[u8],
) -> Result<String, String> {
    let local = path_aset(ctx, asset_key, filename);
    let rel = path_berkas(&local);

    let lower = local.to_lowercase();
    let bytes = if lower.ends_with(".jpeg") || lower.ends_with(".jpg") {
        // `sharp(buffer).resize(1280, 720, { fit: "inside",
        // withoutEnlargement: false }).jpeg({ quality: 80 })` (`uploader.js:100-108`).
        match oktz_plugin_api::gambar::sharp(buffer) {
            Ok(g) => g
                .ubah_ukuran(1280, 720)
                .pas_di_dalam()
                .ke_jpeg(80)
                .unwrap_or_else(|_| buffer.to_vec()),
            Err(_) => buffer.to_vec(),
        }
    } else {
        buffer.to_vec()
    };

    ctx.berkas().tulis(&rel, &bytes)?;
    Ok(local)
}

/// Kartu "kirim/reply media" untuk plugin `ganti-*`.
///
/// Bentuknya persis sama dengan tiap plugin JS, hanya judul dan path-nya yang
/// berbeda.
pub(crate) fn pakai(judul: &str, path: &str) -> String {
    pakai_media(judul, path, "gambar")
}

pub(crate) fn pakai_media(judul: &str, path: &str, jenis: &str) -> String {
    format!("{judul}\n\n> Kirim/reply {jenis} untuk mengganti\n> File: {path}")
}

/// `m.isImage || (m.quoted && m.quoted.type === 'imageMessage')`.
pub(crate) fn adalah_gambar(ctx: &Ctx) -> bool {
    ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image)
        || ctx.quoted().is_some_and(|q| q.kind.is_image())
}

/// `m.type === 'audioMessage' || (m.quoted && m.quoted.type === 'audioMessage')`.
pub(crate) fn adalah_audio(ctx: &Ctx) -> bool {
    ctx.is_media_kind(oktz_plugin_api::IncomingKind::Audio)
        || ctx.quoted().is_some_and(|q| q.kind.is_audio())
}

/// Unduh buffer: quoted dulu, baru pemicu (`ganti-*.js:28-32`).
pub(crate) fn unduh_media(ctx: &Ctx) -> Option<Vec<u8>> {
    if ctx.quoted().is_some() {
        ctx.download_quoted_media().ok()
    } else if ctx.is_media() {
        ctx.download_media().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_berkas_membuang_awalan_dot_slash() {
        assert_eq!(path_berkas("./assets/image/foto2.jpeg"), "assets/image/foto2.jpeg");
        assert_eq!(path_berkas("assets/image/foto2.jpeg"), "assets/image/foto2.jpeg");
    }
}
