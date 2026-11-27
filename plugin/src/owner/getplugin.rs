//! Kirim source code plugin sebagai dokumen.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use super::addplugin::push_name;

/// Akar pohon plugin Rust.
const AKAR: [(&str, &str); 1] = [("plugin/src", "rs")];

/// Teks bantuan.
pub(crate) fn bantuan(nama: &str) -> String {
    format!(
        "Halo *{nama}*, sepertinya kamu lupa memasukkan nama plugin yang ingin dicari.\n\n\
         Silakan gunakan format berikut:\n\
         - .getplugin <nama plugin>\n\n\
         Contoh penggunaan:\n\
         - .getplugin menu\n\
         - .getplugin sticker\n\
         - .getplugin game/tebakgambar"
    )
}

/// Nama plugin tidak valid.
pub(crate) const NAMA_INVALID: &str = "❌ Nama plugin tidak valid.";

/// Plugin tidak ditemukan + daftar mirip.
pub(crate) fn tidak_ditemukan(nama: &str, push: &str, mirip: &[String]) -> String {
    let mut out =
        format!("Maaf ya *{push}*, plugin dengan nama *{nama}* tidak dapat ditemukan.\n\n");
    if !mirip.is_empty() {
        out.push_str("Mungkin maksud kamu salah satu dari plugin ini:\n");
        for s in mirip {
            out.push_str(&format!("- {s}\n"));
        }
    }
    out
}

/// `^[\w-]+$` untuk kategori dan `^[\w.-]+$` untuk nama berkas
///
fn nama_valid(kategori: &str, berkas: &str) -> bool {
    let kategori_ok = kategori
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let berkas_ok = berkas
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
    kategori_ok && berkas_ok
}

/// Satu hasil pencarian: `(kategori, nama_berkas, jalur_lengkap)`.
struct Hasil {
    kategori: String,
    berkas: String,
    jalur: String,
}

/// `searchPlugin` — cari nama berkas persis, lalu
/// alias yang tertulis di dalam berkas.
fn cari_plugin(ctx: &Ctx, nama: &str) -> Option<Hasil> {
    let target = nama.to_lowercase();

    // Tahap 1: nama berkas persis.
    for (akar, ekstensi) in AKAR {
        let Ok(kategori_daftar) = ctx.berkas().daftar(akar) else {
            continue;
        };
        for kategori in kategori_daftar {
            let Ok(entri) = ctx.berkas().daftar(&format!("{akar}/{kategori}")) else {
                continue;
            };
            for berkas in entri {
                if !berkas.ends_with(&format!(".{ekstensi}")) {
                    continue;
                }
                let base = berkas
                    .trim_end_matches(&format!(".{ekstensi}"))
                    .to_lowercase();
                if base == target {
                    return Some(Hasil {
                        kategori: kategori.clone(),
                        berkas: berkas.clone(),
                        jalur: format!("{akar}/{kategori}/{berkas}"),
                    });
                }
            }
        }
    }

    // Tahap 2: alias di dalam berkas (`/alias:\s*\[([^\]]+)\]/`).
    for (akar, _) in AKAR {
        let Ok(kategori_daftar) = ctx.berkas().daftar(akar) else {
            continue;
        };
        for kategori in kategori_daftar {
            let Ok(entri) = ctx.berkas().daftar(&format!("{akar}/{kategori}")) else {
                continue;
            };
            for berkas in entri {
                let Ok(isi) = ctx
                    .berkas()
                    .baca_teks(&format!("{akar}/{kategori}/{berkas}"))
                else {
                    continue;
                };
                if isi_has_alias(&isi, &target) {
                    return Some(Hasil {
                        kategori: kategori.clone(),
                        berkas: berkas.clone(),
                        jalur: format!("{akar}/{kategori}/{berkas}"),
                    });
                }
            }
        }
    }
    None
}

/// `getSimilarPlugins` — base name yang saling
/// memuat, maksimal 5.
fn plugin_mirip(ctx: &Ctx, nama: &str) -> Vec<String> {
    let target = nama.to_lowercase();
    let mut hasil = Vec::new();
    for (akar, ekstensi) in AKAR {
        let Ok(kategori_daftar) = ctx.berkas().daftar(akar) else {
            continue;
        };
        for kategori in kategori_daftar {
            let Ok(entri) = ctx.berkas().daftar(&format!("{akar}/{kategori}")) else {
                continue;
            };
            for berkas in entri {
                if !berkas.ends_with(&format!(".{ekstensi}")) {
                    continue;
                }
                let base = berkas
                    .trim_end_matches(&format!(".{ekstensi}"))
                    .to_lowercase();
                if base.contains(&target) || target.contains(&base) {
                    hasil.push(format!("{kategori}/{berkas}"));
                }
            }
        }
    }
    hasil.truncate(5);
    hasil
}

/// Apakah isi berkas memuat `alias: [...]` yang memuat alias dicari.
fn isi_has_alias(isi: &str, target: &str) -> bool {
    let Some(pos) = isi.find("alias:") else {
        return false;
    };
    let Some(buka) = isi[pos..].find('[') else {
        return false;
    };
    let Some(tutup) = isi[pos + buka..].find(']') else {
        return false;
    };
    let daftar = &isi[pos + buka + 1..pos + buka + tutup];
    for kutip in ['\'', '"', '`'] {
        let needle = format!("{kutip}{target}{kutip}");
        if daftar.contains(&needle) {
            return true;
        }
    }
    false
}

/// Caption dokumen.
pub(crate) fn caption(kategori: &str, berkas: &str, ukuran: usize) -> String {
    format!("📁 *{kategori}/{berkas}*\n{ukuran} bytes")
}

#[plugin(
    name = "getplugin",
    alias = ["gp", "getcode", "plugincode", "sourcecode"],
    category = "owner",
    description = "Dapatkan source code plugin",
    usage = ".getplugin <nama plugin>",
    example = ".getplugin menu",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn getplugin(ctx: Ctx) -> Reply {
    if !ctx.is_owner {
        return Reply::text("❌ *Owner Only!*");
    }

    // `m.args?.[0]?.trim()`.
    let plugin_name = ctx
        .args
        .first()
        .map(|a| a.trim().to_string())
        .unwrap_or_default();

    if plugin_name.is_empty() {
        return Reply::text(bantuan(push_name(&ctx)));
    }

    if plugin_name.contains('/') {
        let mut bagian = plugin_name.splitn(2, '/');
        let kategori = bagian.next().unwrap_or_default().to_string();
        let berkas = bagian.next().unwrap_or_default().to_string();
        if !nama_valid(&kategori, &berkas) {
            return Reply::text(NAMA_INVALID);
        }
        // ditambahkan kalau belum ada ekstensi.
        let jalur = format!("plugin/src/{kategori}/{berkas}");
        if ctx.berkas().ada(&jalur) {
            return kirim_dokumen(&ctx, &kategori, berkas.as_str(), &jalur);
        }
        // Tanpa ekstensi: coba tambahkan `.rs` /.
        for (akar, ekstensi) in AKAR {
            let jalur = format!("{akar}/{kategori}/{berkas}.{ekstensi}");
            if ctx.berkas().ada(&jalur) {
                return kirim_dokumen(&ctx, &kategori, &format!("{berkas}.{ekstensi}"), &jalur);
            }
        }
        let mirip = plugin_mirip(&ctx, &plugin_name);
        return Reply::text(tidak_ditemukan(&plugin_name, push_name(&ctx), &mirip));
    }

    match cari_plugin(&ctx, &plugin_name) {
        Some(hasil) => kirim_dokumen(&ctx, &hasil.kategori, &hasil.berkas, &hasil.jalur),
        None => {
            let mirip = plugin_mirip(&ctx, &plugin_name);
            Reply::text(tidak_ditemukan(&plugin_name, push_name(&ctx), &mirip))
        }
    }
}

/// Baca berkas lalu kirim sebagai dokumen.
fn kirim_dokumen(ctx: &Ctx, kategori: &str, berkas: &str, jalur: &str) -> Reply {
    let byte = match ctx.berkas().baca(jalur) {
        Ok(b) => b,
        Err(e) => {
            return Reply::text(format!("❌ Gagal membaca plugin: {e}"));
        }
    };
    Reply::document(
        MediaSource::bytes(byte.clone()).with_mimetype("text/plain"),
        berkas.to_string(),
    )
    .with_caption(caption(kategori, berkas, byte.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use std::sync::Arc;

    fn ctx_dengan_berkas(berkas: &[(&str, &str)]) -> Ctx {
        let palsu = BerkasPalsu::baru();
        for (path, isi) in berkas {
            palsu.tulis_teks(path, isi).unwrap();
        }
        Ctx {
            prefix: ".".into(),
            command: "getplugin".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            push_name: "Fauzan".into(),
            text: String::new(),
            args: Vec::new(),
            is_owner: true,
            berkas: Arc::new(palsu),
            ..Default::default()
        }
    }

    fn run(ctx: Ctx) -> Reply {
        getplugin(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("getplugin").unwrap();
        assert_eq!(p.meta.name, "getplugin");
        assert_eq!(p.meta.alias, ["gp", "getcode", "plugincode", "sourcecode"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Dapatkan source code plugin");
        assert_eq!(p.meta.usage, ".getplugin <nama plugin>");
        assert_eq!(p.meta.example, ".getplugin menu");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn bukan_owner_menolak() {
        let mut ctx = ctx_dengan_berkas(&[]);
        ctx.is_owner = false;
        let r = run(ctx);
        assert_eq!(teks(&r), "❌ *Owner Only!*");
    }

    #[test]
    fn tanpa_nama_menampilkan_bantuan() {
        let r = run(ctx_dengan_berkas(&[]));
        assert_eq!(
            teks(&r),
            "Halo *Fauzan*, sepertinya kamu lupa memasukkan nama plugin yang ingin dicari.\n\n\
             Silakan gunakan format berikut:\n\
             - .getplugin <nama plugin>\n\n\
             Contoh penggunaan:\n\
             - .getplugin menu\n\
             - .getplugin sticker\n\
             - .getplugin game/tebakgambar"
        );
    }

    #[test]
    fn plugin_yang_tidak_ada_ditemukan_ditolak() {
        let mut ctx = ctx_dengan_berkas(&[]);
        ctx.args = vec!["menu".into()];
        let r = run(ctx);
        let t = teks(&r);
        assert!(
            t.starts_with("Maaf ya *Fauzan*, plugin dengan nama *menu* tidak dapat ditemukan."),
            "{t}"
        );
    }

    #[test]
    fn plugin_yang_ada_dikirim_sebagai_dokumen_dengan_caption() {
        let ctx =
            ctx_dengan_berkas(&[("plugin/src/owner/menu.rs", "fn main() {}\n// isi berkas\n")]);
        let mut ctx = ctx;
        ctx.args = vec!["menu".into()];
        let r = run(ctx);
        match r.without_mutations() {
            Reply::Document {
                caption, file_name, ..
            } => {
                assert_eq!(file_name, "menu.rs");
                assert_eq!(caption, "📁 *owner/menu.rs*\n27 bytes");
            }
            other => panic!("harus Document, dapat {other:?}"),
        }
    }

    #[test]
    fn caption_pola_js() {
        assert_eq!(
            caption("owner", "menu.js", 1234),
            "📁 *owner/menu.js*\n1234 bytes"
        );
    }

    #[test]
    fn nama_dengan_slash_divalidasi() {
        let mut ctx = ctx_dengan_berkas(&[]);
        ctx.args = vec!["../rahasia/file".into()];
        let r = run(ctx);
        assert_eq!(teks(&r), NAMA_INVALID);
    }
}
