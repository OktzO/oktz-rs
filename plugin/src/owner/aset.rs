//! Helper bersama untuk plugin `ganti-*` (keluarga `owner`).

use oktz_plugin_api::Ctx;

/// Path aset lokal seperti yang dikembalikan versi awal.
///
/// Mengembalikan string dengan awalan `./` — persis bentuk `localPath` versi awal.
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
        // withoutEnlargement: false }).jpeg({ quality: 80 })`.
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
/// Bentuknya persis sama dengan tiap plugin versi awal, hanya judul dan path-nya
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

/// Unduh buffer: quoted dulu, baru pemicu.
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
