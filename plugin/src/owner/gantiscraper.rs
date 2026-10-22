//! `plugins/owner/gantiscraper.js` — ganti code scraper yang sudah ada di
//! `src/scraper`.
//!
//! Port dari nol. Yang dipindah:
//! - daftar scraper (`gantiscraper.js:109-127`) — dibaca lewat
//!   `ctx.berkas().daftar("src/scraper")` + `ukuran()`.
//! - teks bantuan (`:130-138`).
//! - guard kode terlalu pendek / bukan scraper / nama tak terdeteksi / nama
//!   tidak valid (`:154-196`).
//! - penulisan + backup berkas (`:203-230`).
//!
//! `ponytail:` hot reload (`import()` dinamis + `hotReloadPlugin`,
//! `gantiscraper.js:82-101`) MUSTAHIL di Rust: registry plugin/scraper
//! didaftarkan saat compile (linkme), tidak ada evaluasi JS saat runtime.
//! Jadi berkas scraper tetap ditulis dan di-backup, tapi bagian "Hot Reload"
//! diganti pemberitahuan bahwa bot perlu restart.
//!
//! `ponytail:` `ctx.berkas()` berakar ke direktori kerja (cwd) — sama dengan
//! `process.cwd()` di JS, jadi `./src/scraper` dan `./backup/scraper` punya
//! padanan persis.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Folder scraper (`gantiscraper.js:23`).
const SCRAPER_DIR: &str = "src/scraper";
/// Folder backup (`gantiscraper.js:217`).
const BACKUP_DIR: &str = "backup/scraper";

/// `fs.readdirSync(SCRAPER_DIR).filter(f => f.endsWith(".js"))`
/// (`gantiscraper.js:27-31`) — urutan `readdirSync` dipertahankan.
fn daftar_scraper(ctx: &Ctx) -> Vec<String> {
    ctx.berkas()
        .daftar(SCRAPER_DIR)
        .unwrap_or_default()
        .into_iter()
        .filter(|f| f.ends_with(".js"))
        .collect()
}

/// Teks bantuan (`gantiscraper.js:131-138`).
fn bantuan(prefix: &str) -> String {
    format!(
        "🔄 *GANTI SCRAPER*\n\n\
         Reply code scraper baru dengan caption:\n\
         `{prefix}gantiscraper` - Auto detect dari export\n\
         `{prefix}gantiscraper namafile` - Custom nama file\n\n\
         📋 *Lihat daftar scraper:*\n\
         `{prefix}gantiscraper list`\n\n\
         ⚠️ *PERINGATAN:*\n\
         Code lama akan di-backup sebelum diganti"
    )
}

/// `code.match(/export\s+default\s+(?:async\s+)?function\s+(\w+)|export\s+default\s+(\w+)/)`
/// (`gantiscraper.js:173-176`) — diganti parser sederhana tanpa regex crate.
fn nama_default_export(code: &str) -> Option<String> {
    let idx = code.find("export")?;
    let rest = &code[idx + "export".len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("default")?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix("async").map_or(rest, |r| r.trim_start());
    let rest = rest.strip_prefix("function").map_or(rest, |r| r.trim_start());
    Some(ident_awal(rest)?.to_string())
}

/// `code.match(/export\s+(?:async\s+)?function\s+(\w+)|export\s+const\s+(\w+)\s*=/)`
/// (`gantiscraper.js:179-182`).
fn nama_named_export(code: &str) -> Option<String> {
    let mut dari = 0usize;
    while let Some(rel) = code[dari..].find("export") {
        let idx = dari + rel;
        let rest = code[idx + "export".len()..].trim_start();
        if let Some(r) = rest.strip_prefix("async").map(|r| r.trim_start()) {
            if let Some(r) = r.strip_prefix("function").map(|r| r.trim_start()) {
                if let Some(id) = ident_awal(r) {
                    return Some(id.to_string());
                }
            }
        }
        if let Some(r) = rest.strip_prefix("function").map(|r| r.trim_start()) {
            if let Some(id) = ident_awal(r) {
                return Some(id.to_string());
            }
        }
        if let Some(r) = rest.strip_prefix("const").map(|r| r.trim_start()) {
            if let Some(id) = ident_awal(r) {
                return Some(id.to_string());
            }
        }
        dari = idx + "export".len();
    }
    None
}

/// `\w+` — huruf/angka/underscore di awal.
fn ident_awal(s: &str) -> Option<&str> {
    let end = s
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    if end == 0 { None } else { Some(&s[..end]) }
}

/// `code.includes("module.exports") || code.includes("export ") ||
/// code.includes("export default") || code.includes("export {")`
/// (`gantiscraper.js:158-162`).
fn punya_export(code: &str) -> bool {
    code.contains("module.exports")
        || code.contains("export ")
        || code.contains("export default")
        || code.contains("export {")
}

/// `fileName.toLowerCase().replace(/[^a-z0-9\-_]/g, "")`
/// (`gantiscraper.js:194`).
fn bersihkan_nama(nama: &str) -> String {
    nama.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-' || *c == '_')
        .collect()
}

/// `new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19)`
/// (`gantiscraper.js:222-224`) — timestamp UTC `YYYY-MM-DDTHH-MM-SS`.
fn stempel() -> String {
    let ms = crate::common::now_ms();
    // ISO = UTC; helper lokal memakai Asia/Jakarta (+7), jadi dikurangi 7 jam.
    let (y, m, d, h, mi, s) = crate::common::jakarta_parts(ms - 7 * 3_600_000);
    format!(
        "{y:04}-{m:02}-{d:02}T{h:02}-{mi:02}-{s:02}",
        y = y,
        m = m,
        d = d,
        h = h,
        mi = mi,
        s = s
    )
}

#[plugin(
    name = "gantiscraper",
    alias = ["replacescraper", "updatescraper", "gantiscrape"],
    category = "owner",
    description = "Ganti code scraper yang sudah ada di src/scraper",
    usage = ".gantiscraper [namafile]",
    example = ".gantiscraper ig",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn gantiscraper(ctx: Ctx) -> Reply {
    let args = ctx.args.clone();

    if args.first().is_some_and(|a| a.to_lowercase() == "list") {
        let scrapers = daftar_scraper(&ctx);
        if scrapers.is_empty() {
            return Reply::text("📂 Folder src/scraper kosong");
        }
        let mut text = String::from("📂 *DAFTAR SCRAPER*\n\n╭─〔 *src/scraper* 〕───⬣\n");
        for (i, s) in scrapers.iter().enumerate() {
            let size = ctx
                .berkas()
                .ukuran(&format!("{SCRAPER_DIR}/{s}"))
                .unwrap_or(0);
            text.push_str(&format!(
                "│ {}. `{s}` ({} KB)\n",
                i + 1,
                format!("{:.1}", size as f64 / 1024.0)
            ));
        }
        text.push_str(&format!(
            "╰───────⬡\n\nTotal: {} scraper\n\n> Gunakan `{}gantiscraper <nama>` dengan reply code",
            scrapers.len(),
            ctx.prefix
        ));
        return Reply::text(text);
    }

    let Some(quoted) = ctx.quoted() else {
        return Reply::text(bantuan(&ctx.prefix));
    };

    let mut code = quoted.text.clone();

    let unduh_berkas = quoted.mimetype.as_deref() == Some("application/javascript")
        || quoted
            .file_name
            .as_deref()
            .is_some_and(|f| f.ends_with(".js"));
    if unduh_berkas {
        match ctx.download_quoted_media() {
            Ok(bytes) => code = String::from_utf8_lossy(&bytes).to_string(),
            Err(_) => return Reply::text("❌ *GAGAL*\n\nGagal download file"),
        }
    }

    if code.chars().count() < 30 {
        return Reply::text("❌ *GAGAL*\n\nCode terlalu pendek atau tidak valid");
    }

    if !punya_export(&code) {
        return Reply::text("❌ *GAGAL*\n\nCode bukan format scraper yang valid\nHarus ada export");
    }

    let mut file_name = args.first().cloned();
    if file_name.is_none() {
        file_name = nama_default_export(&code).or_else(|| nama_named_export(&code));
    }

    let Some(file_name) = file_name else {
        return Reply::text(format!(
            "❌ *GAGAL*\n\nTidak bisa mendeteksi nama scraper\nGunakan `{}gantiscraper <namafile>`",
            ctx.prefix
        ));
    };

    let file_name = bersihkan_nama(&file_name);
    if file_name.is_empty() {
        return Reply::text("❌ *GAGAL*\n\nNama file tidak valid");
    }

    // `fs.mkdirSync(SCRAPER_DIR, { recursive: true })` (`:203-205`).
    let _ = ctx.berkas().buat_folder(SCRAPER_DIR);

    let target = format!("{SCRAPER_DIR}/{file_name}.js");
    let existing = ctx.berkas().ada(&target);
    let old_size = ctx.berkas().ukuran(&target).unwrap_or(0);
    let mut backup_rel: Option<String> = None;

    if existing {
        let _ = ctx.berkas().buat_folder(BACKUP_DIR);
        let backup = format!("{BACKUP_DIR}/{file_name}_{}.js", stempel());
        let _ = ctx.berkas().salin(&target, &backup);
        backup_rel = Some(backup);
    }

    if ctx.berkas().tulis_teks(&target, &code).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    let mut reply_text = format!(
        "✅ *SCRAPER {}*\n\n╭─〔 *DETAIL* 〕───⬡\n│ File: `{file_name}.js`\n│ Folder: `src/scraper`\n│ Size: `{} bytes`\n",
        if existing { "DIGANTI" } else { "DITAMBAH" },
        code.chars().count()
    );
    if existing {
        reply_text.push_str(&format!("│ Old Size: `{old_size} bytes`\n"));
    }
    reply_text.push_str("╰───────⬡\n\n");
    if let Some(backup) = backup_rel {
        reply_text.push_str(&format!("💾 *Backup:*\n`{backup}`\n\n"));
    }
    reply_text.push_str(
        "⚠️ *Hot Reload tidak tersedia di versi Rust.*\n\
         Scraper didaftarkan secara statis saat compile — \
         restart bot agar scraper baru dipakai.\n",
    );

    Reply::text(reply_text).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_dengan(berkas: Arc<BerkasPalsu>, quoted_text: Option<&str>, args: &[&str]) -> Ctx {
        Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "gantiscraper".into(),
            berkas,
            quoted: quoted_text.map(|t| oktz_plugin_api::QuotedMessage {
                id: "Q1".into(),
                sender: ME.into(),
                chat: ME.into(),
                text: t.into(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("gantiscraper").unwrap();
        assert_eq!(p.meta.name, "gantiscraper");
        assert_eq!(
            p.meta.alias,
            ["replacescraper", "updatescraper", "gantiscrape"]
        );
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Ganti code scraper yang sudah ada di src/scraper"
        );
        assert_eq!(p.meta.usage, ".gantiscraper [namafile]");
        assert_eq!(p.meta.example, ".gantiscraper ig");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["replacescraper", "updatescraper", "gantiscrape"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("gantiscraper"), "{a}");
        }
    }

    #[test]
    fn tanpa_reply_tampil_bantuan() {
        let b = Arc::new(BerkasPalsu::baru());
        let r = gantiscraper(ctx_dengan(b, None, &[]));
        assert_eq!(
            teks(&r),
            "🔄 *GANTI SCRAPER*\n\n\
             Reply code scraper baru dengan caption:\n\
             `.gantiscraper` - Auto detect dari export\n\
             `.gantiscraper namafile` - Custom nama file\n\n\
             📋 *Lihat daftar scraper:*\n\
             `.gantiscraper list`\n\n\
             ⚠️ *PERINGATAN:*\n\
             Code lama akan di-backup sebelum diganti"
        );
    }

    #[test]
    fn list_kosong_tampil_pesan_folder_kosong() {
        let b = Arc::new(BerkasPalsu::baru());
        let r = gantiscraper(ctx_dengan(b, None, &["list"]));
        assert_eq!(teks(&r), "📂 Folder src/scraper kosong");
    }

    #[test]
    fn list_menampilkan_scraper_dan_ukuran() {
        let b = Arc::new(BerkasPalsu::baru());
        b.tulis_teks("src/scraper/ig.js", &"x".repeat(2048)).unwrap();
        b.tulis_teks("src/scraper/yt.js", &"y".repeat(512)).unwrap();
        let r = gantiscraper(ctx_dengan(b, None, &["list"]));
        let t = teks(&r);
        assert!(t.starts_with("📂 *DAFTAR SCRAPER*\n\n╭─〔 *src/scraper* 〕───⬣\n"), "{t}");
        assert!(t.contains("│ 1. `ig.js` (2.0 KB)"), "{t}");
        assert!(t.contains("│ 2. `yt.js` (0.5 KB)"), "{t}");
        assert!(t.contains("╰───────⬡\n\nTotal: 2 scraper"), "{t}");
        assert!(t.contains("> Gunakan `.gantiscraper <nama>` dengan reply code"), "{t}");
    }

    #[test]
    fn kode_pendek_ditolak() {
        let b = Arc::new(BerkasPalsu::baru());
        let r = gantiscraper(ctx_dengan(b, Some("pendek"), &[]));
        assert_eq!(teks(&r), "❌ *GAGAL*\n\nCode terlalu pendek atau tidak valid");
    }

    #[test]
    fn kode_tanpa_export_ditolak() {
        let b = Arc::new(BerkasPalsu::baru());
        let kode = "const a = 1; // ".to_string() + &"x".repeat(40);
        let r = gantiscraper(ctx_dengan(b, Some(&kode), &[]));
        assert_eq!(
            teks(&r),
            "❌ *GAGAL*\n\nCode bukan format scraper yang valid\nHarus ada export"
        );
    }

    #[test]
    fn nama_tidak_terdeteksi_ditolak() {
        let b = Arc::new(BerkasPalsu::baru());
        let kode = "export {somethingElse}; ".to_string() + &"x".repeat(40);
        let r = gantiscraper(ctx_dengan(b, Some(&kode), &[]));
        assert_eq!(
            teks(&r),
            "❌ *GAGAL*\n\nTidak bisa mendeteksi nama scraper\nGunakan `.gantiscraper <namafile>`"
        );
    }

    #[test]
    fn nama_otomatis_dari_default_export_dan_berkas_ditulis() {
        let b = Arc::new(BerkasPalsu::baru());
        let kode = "export default async function instagramScraper() {}\n// padding padding";
        let r = gantiscraper(ctx_dengan(b.clone(), Some(kode), &[]));
        let t = teks(&r);
        assert!(t.starts_with("✅ *SCRAPER DITAMBAH*"), "{t}");
        assert!(t.contains("│ File: `instagramscraper.js`"), "{t}");
        assert!(t.contains("│ Folder: `src/scraper`"), "{t}");
        assert!(
            b.baca_teks("src/scraper/instagramscraper.js").unwrap() == kode,
            "berkas harus ditulis"
        );
    }

    #[test]
    fn scraper_lama_dibackup_dan_ditandai_diganti() {
        let b = Arc::new(BerkasPalsu::baru());
        b.tulis_teks("src/scraper/ig.js", "export default function ig(){}").unwrap();
        let kode = "export default function ig() { /* baru */ }\n// padding";
        let r = gantiscraper(ctx_dengan(b.clone(), Some(kode), &[]));
        let t = teks(&r);
        assert!(t.starts_with("✅ *SCRAPER DIGANTI*"), "{t}");
        assert!(t.contains("│ Old Size: `30 bytes`"), "{t}");
        assert!(t.contains("💾 *Backup:*\n`backup/scraper/ig_"), "{t}");
        assert!(t.contains("Hot Reload tidak tersedia di versi Rust"), "{t}");
        assert_eq!(b.baca_teks("src/scraper/ig.js").unwrap(), kode);
    }

    #[test]
    fn nama_eksplisit_dipakai_dan_dibersihkan() {
        let b = Arc::new(BerkasPalsu::baru());
        let kode = "export default function ig() {}\n// padding padding padding";
        let r = gantiscraper(ctx_dengan(b.clone(), Some(kode), &["My Scraper!"]));
        assert!(teks(&r).contains("│ File: `myscraper.js`"), "{}", teks(&r));
        assert!(b.ada("src/scraper/myscraper.js"));
    }

    #[test]
    fn parser_nama_ekspor_sama_dengan_js() {
        assert_eq!(
            nama_default_export("export default async function foo() {}").as_deref(),
            Some("foo")
        );
        assert_eq!(
            nama_default_export("export default bar").as_deref(),
            Some("bar")
        );
        assert_eq!(nama_default_export("export const x = 1"), None);
        assert_eq!(
            nama_named_export("export function foo() {}").as_deref(),
            Some("foo")
        );
        assert_eq!(nama_named_export("export const foo = 1").as_deref(), Some("foo"));
        assert_eq!(nama_named_export("export async function foo() {}").as_deref(), Some("foo"));
    }
}
