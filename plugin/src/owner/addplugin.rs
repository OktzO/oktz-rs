//! `plugins/owner/addplugin.js` — tambah plugin baru dari code yang di-reply.
//!
//! ## Yang dip-port
//!
//! Semua validasi dan semua teks balasan: Absence quote, kode terlalu
//! pendek, kode bukan format plugin, nama tidak terdeteksi, nama file
//! tidak valid, file sudah ada, dan pesan sukses.
//!
//! ## Penyimpangan
//!
//! 1. `hotReloadPlugin(filePath)` (`src/lib/plugins.js`) tidak ada di Rust —
//!    plugin Rust dikompilasi ke dalam binari, jadi berkas yang ditulis
//!    saat runtime baru aktif setelah bot dibangun ulang. Karena itu
//!    "Status Reload" selalu `Pending`, bukan `Berhasil`.
//! 2. Lokasi berkas mengikuti JS persis (`<cwd>/plugins/<folder>/<file>.js`)
//!    supaya path yang dilaporkan sama, tetapi router Rust memuat plugin
//!    dari `plugin/src/`, bukan dari folder itu.
//!
//! `m.react("🕕")` sebelum menulis dan `m.react("✅")` setelahnya ikut
//! sebagai reaksi pada pesan pemicu.

use oktz_plugin_api::{Ctx, QuotedKind, Reply, plugin};

/// Batas panjang kode JS (`addplugin.js:58`).
const MIN_KODE: usize = 50;

/// `m.pushName` selalu string di JS (`serialize.js:760`): `"Unknown"` kalau
/// kosong. `Ctx::push_name` boleh kosong, jadi fallback-nya dipasang di sini.
pub(crate) fn push_name(ctx: &Ctx) -> &str {
    if ctx.push_name.is_empty() {
        "Unknown"
    } else {
        &ctx.push_name
    }
}

/// `extractPluginInfo` (`addplugin.js:22-29`) — cari label `name:` atau
/// `category:` lalu ambil string di dalam kutip pertama.
pub(crate) fn ambil_label(code: &str, label: &str) -> Option<String> {
    let lower = code.to_lowercase();
    let mut dari = 0usize;
    while let Some(pos) = lower[dari..].find(label) {
        let start = dari + pos + label.len();
        let bytes = code.as_bytes();
        // `\s*` — lompat spasi putih apa adanya.
        let mut i = start;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i < bytes.len() && matches!(bytes[i], b'\'' | b'"' | b'`') {
            let isi_start = i + 1;
            let mut j = isi_start;
            while j < bytes.len() && !matches!(bytes[j], b'\'' | b'"' | b'`') {
                j += 1;
            }
            if j > isi_start {
                return Some(code[isi_start..j].to_string());
            }
        }
        dari = start;
    }
    None
}

/// `fileName.toLowerCase().replace(/[^a-z0-9\-_]/g, "")` (`addplugin.js:84`).
pub(crate) fn bersihkan_nama(nama: &str) -> String {
    nama.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Teks bantuan `addplugin.js:35-41`.
pub(crate) fn tanpa_quote(nama: &str) -> String {
    format!(
        "Halo *{nama}*, sepertinya kamu belum mereply kode pluginnya.\n\n\
         Silakan reply kode plugin yang ingin ditambahkan dengan perintah:\n\
         - .addplugin (untuk deteksi otomatis)\n\
         - .addplugin <nama file> (untuk nama kustom)\n\
         - .addplugin <nama file> <folder> (untuk nama dan folder kustom)"
    )
}

/// `addplugin.js:59`.
pub(crate) fn terlalu_pendek(nama: &str) -> String {
    format!(
        "Maaf *{nama}*, proses gagal karena kode terlalu pendek atau tidak valid."
    )
}

/// `addplugin.js:66`.
pub(crate) fn bukan_format_valid(nama: &str) -> String {
    format!(
        "Maaf *{nama}*, proses gagal karena kode bukan format plugin yang valid. Pastikan ada export dan config di dalamnya."
    )
}

/// `addplugin.js:78`.
pub(crate) fn nama_tidak_terdeteksi(nama: &str) -> String {
    format!(
        "Maaf *{nama}*, aku tidak bisa mendeteksi nama pluginnya. Silakan gunakan perintah dengan format .addplugin <nama file>."
    )
}

/// `addplugin.js:88`.
pub(crate) fn nama_file_tidak_valid(nama: &str) -> String {
    format!("Maaf *{nama}*, proses gagal karena nama file tidak valid.")
}

/// `addplugin.js:105-106`.
pub(crate) fn sudah_ada(nama: &str, file: &str, folder: &str) -> String {
    format!(
        "Maaf *{nama}*, file {file}.js sudah ada di folder {folder}.\n\n\
         💡 Tips: Gunakan perintah .ganticode {file} {folder} jika kamu ingin mengganti kode plugin yang sudah ada."
    )
}

/// Pesan sukses `addplugin.js:118-124`.
pub(crate) fn sukses(file: &str, folder: &str, ukuran: usize) -> String {
    format!(
        "Proses selesai! Plugin berhasil ditambahkan ke dalam sistem.\n\n\
         - File: {file}.js\n\
         - Folder: {folder}\n\
         - Ukuran: {ukuran} bytes\n\
         - Status Reload: Pending\n\n\
         Plugin sudah aktif dan siap digunakan, silakan dicoba ya!"
    )
}

/// Isi balasan: `quoted.text || quoted.body`, atau isi berkas media yang
/// di-reply (`addplugin.js:44-56`).
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

/// `quoted.isDocument || quoted.type === "documentMessage" || quoted.isMedia`.
fn quoted_adalah_media(ctx: &Ctx) -> bool {
    ctx.quoted()
        .is_some_and(|q| matches!(q.kind, QuotedKind::Document) || q.kind.is_media())
}

#[plugin(
    name = "addplugin",
    alias = ["addpl", "tambahplugin"],
    category = "owner",
    description = "Tambah plugin baru dari code yang di-reply",
    usage = ".addplugin [namafile] [folder]",
    example = ".addplugin bliblidl downloader",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn addplugin(ctx: Ctx) -> Reply {
    let nama = push_name(&ctx);

    if ctx.quoted().is_none() {
        return Reply::text(tanpa_quote(nama));
    }

    let kode = kode_dari_ctx(&ctx);
    if quoted_adalah_media(&ctx) && kode.trim().is_empty() {
        // Berkas media gagal diunduh — JS menangkap error dan membalas
        // pesan unduhan gagal (`addplugin.js:54`).
        if ctx.download_quoted_media().is_err() {
            return Reply::text(format!(
                "Maaf *{nama}*, proses gagal karena file tidak dapat diunduh."
            ));
        }
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
    let folder_name = ctx
        .args
        .get(1)
        .cloned()
        .or(detected_category)
        .unwrap_or_else(|| "other".to_string());

    if file_name.is_empty() {
        return Reply::text(nama_tidak_terdeteksi(nama));
    }

    let file_name = bersihkan_nama(&file_name);
    let folder_name = bersihkan_nama(&folder_name);

    if file_name.is_empty() {
        return Reply::text(nama_file_tidak_valid(nama));
    }

    // `plugins/<folder>/<file>.js` — path sama dengan JS.
    let folder_path = format!("plugins/{folder_name}");
    let file_path = format!("{folder_path}/{file_name}.js");

    if ctx.berkas().ada(&file_path) {
        return Reply::text(sudah_ada(nama, &file_name, &folder_name))
            .with_reaction("❌");
    }

    let _ = ctx.berkas().buat_folder(&folder_path);
    if ctx.berkas().tulis_teks(&file_path, &kode).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    Reply::text(success_text(&file_name, &folder_name, kode.len()))
        .with_reaction("✅")
}

/// Pesan sukses — dipisah agar bisa diuji tanpa efek samping berkas.
pub(crate) fn success_text(file: &str, folder: &str, ukuran: usize) -> String {
    sukses(file, folder, ukuran)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::QuotedMessage;

    const KODE: &str = "import x from 'y';\nconst pluginConfig = { name: 'demo', category: 'owner' };\nexport { pluginConfig as config, handler };";

    fn ctx_dengan_quoted(text: &str, quoted_text: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "addplugin".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            push_name: "Fauzan".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
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
        let p = oktz_plugin_api::find("addplugin").unwrap();
        assert_eq!(p.meta.name, "addplugin");
        assert_eq!(p.meta.alias, ["addpl", "tambahplugin"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Tambah plugin baru dari code yang di-reply"
        );
        assert_eq!(p.meta.usage, ".addplugin [namafile] [folder]");
        assert_eq!(p.meta.example, ".addplugin bliblidl downloader");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private && !p.meta.is_premium);
    }

    #[test]
    fn tanpa_quote_menampilkan_panduan() {
        let r = addplugin(ctx_dengan_quoted("", None));
        assert_eq!(
            teks(&r),
            "Halo *Fauzan*, sepertinya kamu belum mereply kode pluginnya.\n\n\
             Silakan reply kode plugin yang ingin ditambahkan dengan perintah:\n\
             - .addplugin (untuk deteksi otomatis)\n\
             - .addplugin <nama file> (untuk nama kustom)\n\
             - .addplugin <nama file> <folder> (untuk nama dan folder kustom)"
        );
    }

    #[test]
    fn kode_terlalu_pendek_ditolak() {
        let r = addplugin(ctx_dengan_quoted("", Some("pendek")));
        assert_eq!(
            teks(&r),
            "Maaf *Fauzan*, proses gagal karena kode terlalu pendek atau tidak valid."
        );
    }

    #[test]
    fn kode_tanpa_export_ditolak() {
        let kode = "const pluginConfig = { name: 'demo' };\n// ".to_string() + &"x".repeat(60);
        let r = addplugin(ctx_dengan_quoted("", Some(&kode)));
        assert_eq!(
            teks(&r),
            "Maaf *Fauzan*, proses gagal karena kode bukan format plugin yang valid. Pastikan ada export dan config di dalamnya."
        );
    }

    #[test]
    fn nama_tidak_terdeteksi_kosong_menolak() {
        // Ada export + config, tapi tidak ada `name:`.
        let kode = "export { config };\n// config\n".to_string() + &"x".repeat(60);
        let r = addplugin(ctx_dengan_quoted("", Some(&kode)));
        assert_eq!(
            teks(&r),
            "Maaf *Fauzan*, aku tidak bisa mendeteksi nama pluginnya. Silakan gunakan perintah dengan format .addplugin <nama file>."
        );
    }

    #[test]
    fn label_ekstraksi_sama_dengan_regex_js() {
        assert_eq!(
            ambil_label("name: 'demo', category: 'owner'", "name:").as_deref(),
            Some("demo")
        );
        assert_eq!(
            ambil_label("name: 'demo', category: 'owner'", "category:").as_deref(),
            Some("owner")
        );
        assert_eq!(ambil_label("name: \"demo\"", "name:").as_deref(), Some("demo"));
        assert_eq!(ambil_label("name: `demo`", "name:").as_deref(), Some("demo"));
        assert_eq!(ambil_label("NAME  :  'demo'", "name:").as_deref(), None);
        assert_eq!(ambil_label("tidak ada", "name:"), None);
    }

    #[test]
    fn nama_file_dibersihkan_seperti_js() {
        assert_eq!(bersihkan_nama("Demo File.js"), "demofilejs");
        assert_eq!(bersihkan_nama("keep-me_ok"), "keep-me_ok");
        assert_eq!(bersihkan_nama("!!!"), "");
    }

    #[test]
    fn push_name_kosong_jadi_unknown() {
        let mut ctx = ctx_dengan_quoted("", None);
        ctx.push_name = String::new();
        assert_eq!(push_name(&ctx), "Unknown");
    }

    #[test]
    fn pesan_sesuai_dengan_pola_js() {
        let t = sukses("demo", "owner", 1234);
        assert!(t.starts_with("Proses selesai! Plugin berhasil ditambahkan ke dalam sistem.\n\n"));
        assert!(t.contains("- File: demo.js\n"));
        assert!(t.contains("- Folder: owner\n"));
        assert!(t.contains("- Ukuran: 1234 bytes\n"));
        assert!(t.contains("- Status Reload: Pending\n"));
        assert!(t.ends_with("Plugin sudah aktif dan siap digunakan, silakan dicoba ya!"));
    }

    #[test]
    fn file_yang_sudah_ada_menolak_dengan_tips_ganticode() {
        let t = sudah_ada("Fauzan", "demo", "owner");
        assert!(t.contains("file demo.js sudah ada di folder owner"));
        assert!(t.contains(".ganticode demo owner"));
    }
}
