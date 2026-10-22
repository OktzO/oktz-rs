//! `plugins/owner/ganticode.js` — ganti code plugin yang sudah ada.
//!
//! ## Yang dip-port
//!
//! Semua validasi dan teks balasan: quote kosong, kode terlalu pendek,
//! kode bukan format plugin, nama tidak terdeteksi, nama file tidak valid,
//! dan pesan selesai (ditambahkan / diperbarui, ukuran lama, lokasi backup).
//!
//! ## Penyimpangan
//!
//! 1. `hotReloadPlugin(filePath)` tidak ada di Rust — plugin dikompilasi ke
//!    dalam binari, jadi "Status Reload" selalu `Pending`.
//! 2. Pencarian file plugin memakai `ctx.berkas()` yang berakar ke direktori
//!    kerja, sama seperti `process.cwd()` di JS.
//! 3. `fs.copyFileSync` untuk backup memakai `ctx.berkas().salin()`.

use oktz_plugin_api::{Ctx, Reply, plugin};

use super::addplugin::{ambil_label, bersihkan_nama, push_name};

/// Batas panjang kode JS (`ganticode.js:79`).
const MIN_KODE: usize = 50;

/// Root folder plugin, sama seperti `path.join(process.cwd(), "plugins")`.
const PLUGINS_DIR: &str = "plugins";

/// `ganticode.js:56-63`.
pub(crate) fn tanpa_quote(nama: &str) -> String {
    format!(
        "Halo *{nama}*, sepertinya kamu belum mereply kode plugin barunya.\n\n\
         Silakan reply kode plugin yang baru dengan perintah:\n\
         - .ganticode (untuk deteksi otomatis)\n\
         - .ganticode <nama file> (untuk nama kustom)\n\
         - .ganticode <nama file> <folder> (untuk kustom nama dan folder)\n\n\
         Tenang saja, kode yang lama akan otomatis dibackup sebelum diganti."
    )
}

/// `ganticode.js:80`.
pub(crate) fn terlalu_pendek(nama: &str) -> String {
    format!(
        "Maaf *{nama}*, proses gagal karena kode terlalu pendek atau tidak valid."
    )
}

/// `ganticode.js:87`.
pub(crate) fn bukan_format_valid(nama: &str) -> String {
    format!(
        "Maaf *{nama}*, proses gagal karena kode bukan format plugin yang valid. Pastikan ada export dan config di dalamnya."
    )
}

/// `ganticode.js:99`.
pub(crate) fn nama_tidak_terdeteksi(nama: &str) -> String {
    format!(
        "Maaf *{nama}*, aku tidak bisa mendeteksi nama pluginnya. Silakan gunakan perintah dengan format .ganticode <nama file>."
    )
}

/// `ganticode.js:107`.
pub(crate) fn nama_file_tidak_valid(nama: &str) -> String {
    format!("Maaf *{nama}*, proses gagal karena nama file tidak valid.")
}

/// `ganticode.js:161-179` — pesan selesai.
pub(crate) fn sukses(
    file: &str,
    folder: &str,
    ukuran: usize,
    ukuran_lama: Option<u64>,
    backup_relatif: Option<&str>,
) -> String {
    let status = if ukuran_lama.is_some() {
        "diperbarui"
    } else {
        "ditambahkan"
    };
    let mut out = format!(
        "Proses selesai! Kode plugin berhasil {status}.\n\n\
         - File: {file}.js\n\
         - Folder: {folder}\n\
         - Ukuran: {ukuran} bytes\n"
    );
    if let Some(lama) = ukuran_lama {
        out.push_str(&format!("- Ukuran Lama: {lama} bytes\n"));
    }
    out.push_str("- Status Reload: Pending\n\n");
    if let Some(backup) = backup_relatif {
        out.push_str(&format!(
            "File lama sudah dibackup dengan aman di lokasi berikut:\n{backup}\n\n"
        ));
    }
    out.push_str("Plugin sudah aktif dan siap digunakan, silakan dicoba ya!");
    out
}

/// `new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19)` —
/// cap waktu ISO UTC dengan tanda baca `:` dan `.` diganti `-`.
pub(crate) fn cap_waktu(ms: i64) -> String {
    let total_detik = ms.div_euclid(1000);
    let hari = total_detik.div_euclid(86_400);
    let detik_dalam_hari = total_detik.rem_euclid(86_400);
    let (y, m, d) = civil_dari_hari(hari);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}-{:02}-{:02}",
        detik_dalam_hari / 3600,
        (detik_dalam_hari % 3600) / 60,
        detik_dalam_hari % 60
    )
}

/// `civil_from_days` (Howard Hinnant) — hari sejak epoch → `(y, m, d)`.
fn civil_dari_hari(hari: i64) -> (i64, u32, u32) {
    let z = hari + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Cari file plugin yang namanya cocok — `findPluginFile` (`ganticode.js:31-50`).
///
/// Mengembalikan `(folder, file)`; `file` masih memakai ekstensi `.js`.
pub(crate) fn cari_file_plugin(ctx: &Ctx, nama: &str) -> Option<(String, String)> {
    let folder_terdaftar = ctx.berkas().daftar(PLUGINS_DIR).ok()?;
    let target = nama.to_lowercase();
    for folder in folder_terdaftar {
        let Ok(entri) = ctx.berkas().daftar(&format!("{PLUGINS_DIR}/{folder}")) else {
            continue;
        };
        for file in entri {
            if !file.ends_with(".js") {
                continue;
            }
            let base = file.trim_end_matches(".js").to_lowercase();
            if base == target {
                return Some((folder, file));
            }
        }
    }
    None
}

/// Kode yang diambil dari pesan yang di-reply (`ganticode.js:66-77`).
fn kode_dari_ctx(ctx: &Ctx) -> String {
    if let Some(q) = ctx.quoted() {
        if q.kind.is_media() {
            if let Ok(byte) = ctx.download_quoted_media() {
                return String::from_utf8_lossy(&byte).to_string();
            }
        }
        return q.text.clone();
    }
    String::new()
}

/// `quoted.mimetype === "application/javascript" || quoted.filename?.endsWith(".js")`.
fn quoted_adalah_js(ctx: &Ctx) -> bool {
    ctx.quoted().is_some_and(|q| {
        q.mimetype.as_deref() == Some("application/javascript")
            || q.file_name.as_deref().is_some_and(|f| f.ends_with(".js"))
    })
}

#[plugin(
    name = "ganticode",
    alias = ["replaceplugin", "updateplugin", "gantiplugin"],
    category = "owner",
    description = "Ganti code plugin yang sudah ada",
    usage = ".ganticode [namafile] [folder]",
    example = ".ganticode ping main",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganticode(ctx: Ctx) -> Reply {
    let nama = push_name(&ctx);

    if ctx.quoted().is_none() {
        return Reply::text(tanpa_quote(nama));
    }

    let kode = kode_dari_ctx(&ctx);
    if quoted_adalah_js(&ctx) && ctx.download_quoted_media().is_err() {
        return Reply::text(format!(
            "Maaf *{nama}*, proses gagal karena file tidak dapat diunduh."
        ));
    }

    if kode.chars().count() < MIN_KODE {
        return Reply::text(terlalu_pendek(nama));
    }

    let has_export = kode.contains("module.exports") || kode.contains("export ");
    let has_config = kode.contains("pluginConfig") || kode.contains("config");
    if !has_export || !has_config {
        return Reply::text(bukan_format_valid(nama));
    }

    let detected_name = ambil_label(&kode, "name:");
    let detected_category = ambil_label(&kode, "category:");

    let file_name = ctx
        .args
        .first()
        .cloned()
        .or(detected_name)
        .unwrap_or_default();
    let mut folder_name = ctx
        .args
        .get(1)
        .cloned()
        .or(detected_category)
        .unwrap_or_default();

    if file_name.is_empty() {
        return Reply::text(nama_tidak_terdeteksi(nama));
    }

    let file_name = bersihkan_nama(&file_name);
    if file_name.is_empty() {
        return Reply::text(nama_file_tidak_valid(nama));
    }

    // `findPluginFile` — kalau ada, folder ikut yang lama.
    let existing = cari_file_plugin(&ctx, &file_name);
    let (target_folder, file_path, ukuran_lama, backup_relatif) = match existing {
        Some((folder, file)) => {
            let path = format!("{PLUGINS_DIR}/{folder}/{file}");
            let lama = ctx.berkas().ukuran(&path).unwrap_or(0);
            // `backup/plugins/<file>_<timestamp>.js` (`ganticode.js:126-136`).
            let stamp = cap_waktu(crate::common::now_ms());
            let backup_path = format!("backup/plugins/{file_name}_{stamp}.js");
            let _ = ctx.berkas().buat_folder("backup/plugins");
            let _ = ctx.berkas().salin(&path, &backup_path);
            (folder, path, Some(lama), Some(backup_path))
        }
        None => {
            if folder_name.is_empty() {
                folder_name = "other".to_string();
            }
            let folder_name = bersihkan_nama(&folder_name);
            let path = format!("{PLUGINS_DIR}/{folder_name}/{file_name}.js");
            let _ = ctx.berkas().buat_folder(&format!("{PLUGINS_DIR}/{folder_name}"));
            (folder_name, path, None, None)
        }
    };

    if ctx.berkas().tulis_teks(&file_path, &kode).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    Reply::text(sukses(
        &file_name,
        &target_folder,
        kode.chars().count(),
        ukuran_lama,
        backup_relatif.as_deref(),
    ))
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{QuotedKind, QuotedMessage};

    fn ctx_dengan_quoted(teks: &str, quoted_text: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "ganticode".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            push_name: "Fauzan".into(),
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            quoted: quoted_text.map(|t| QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: t.into(),
                kind: QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            ..Default::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("ganticode").unwrap();
        assert_eq!(p.meta.name, "ganticode");
        assert_eq!(
            p.meta.alias,
            ["replaceplugin", "updateplugin", "gantiplugin"]
        );
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Ganti code plugin yang sudah ada");
        assert_eq!(p.meta.usage, ".ganticode [namafile] [folder]");
        assert_eq!(p.meta.example, ".ganticode ping main");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_quote_menampilkan_panduan() {
        let r = ganticode(ctx_dengan_quoted("", None));
        assert_eq!(
            teks(&r),
            "Halo *Fauzan*, sepertinya kamu belum mereply kode plugin barunya.\n\n\
             Silakan reply kode plugin yang baru dengan perintah:\n\
             - .ganticode (untuk deteksi otomatis)\n\
             - .ganticode <nama file> (untuk nama kustom)\n\
             - .ganticode <nama file> <folder> (untuk kustom nama dan folder)\n\n\
             Tenang saja, kode yang lama akan otomatis dibackup sebelum diganti."
        );
    }

    #[test]
    fn kode_terlalu_pendek_ditolak() {
        let r = ganticode(ctx_dengan_quoted("", Some("pendek")));
        assert_eq!(
            teks(&r),
            "Maaf *Fauzan*, proses gagal karena kode terlalu pendek atau tidak valid."
        );
    }

    #[test]
    fn cap_waktu_pola_iso_utc() {
        // 2023-11-14T23:00:00.000Z → "2023-11-14T23-00-00"
        assert_eq!(cap_waktu(1_700_002_800_000), "2023-11-14T23-00-00");
    }

    #[test]
    fn pesan_baru_menyebut_ditambahkan() {
        let t = sukses("demo", "other", 100, None, None);
        assert!(t.starts_with("Proses selesai! Kode plugin berhasil ditambahkan.\n\n"));
        assert!(t.contains("- File: demo.js\n"));
        assert!(t.contains("- Folder: other\n"));
        assert!(t.contains("- Ukuran: 100 bytes\n"));
        assert!(!t.contains("Ukuran Lama"));
        assert!(t.contains("- Status Reload: Pending\n\n"));
        assert!(t.ends_with("Plugin sudah aktif dan siap digunakan, silakan dicoba ya!"));
    }

    #[test]
    fn pesan_perbarui_menyebut_ukuran_lama_dan_backup() {
        let t = sukses("demo", "owner", 100, Some(42), Some("backup/plugins/demo_x.js"));
        assert!(t.starts_with("Proses selesai! Kode plugin berhasil diperbarui.\n\n"));
        assert!(t.contains("- Ukuran Lama: 42 bytes\n"));
        assert!(t.contains(
            "File lama sudah dibackup dengan aman di lokasi berikut:\nbackup/plugins/demo_x.js\n\n"
        ));
    }
}
