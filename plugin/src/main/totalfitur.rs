//! `plugins/main/totalfitur.js` — distribusi jumlah plugin per kategori.
//!
//! JS mengirim tabel WhatsApp asli lewat `sock.sendTable`
//! (`src/lib/socket.js:1300`), yaitu `botForwardedMessage > richResponseMessage`
//! (`src/lib/rich-messages.js:609-637`). Sistem tabelnya sudah ada di
//! `oktz-wa::tables` (`build_table`) dan `Reply::table`
//! (`oktz-plugin-api::registry`), jadi plugin ini memakai `Reply::Table` —
//! bukan teks yang meniru tabel.
//!
//! Reaksi `📊` dikirim sebelum tabel (`totalfitur.js:49`), jadi balasan
//! dibungkus `Reply::with_reaction`. Cabang `catch` JS (`totalfitur.js:74-77`)
//! membalas template error + reaksi `☢`; di Rust penghitungan memakai
//! `checked_add`, jadi luapan jadi galat biasa dan masuk cabang yang sama.

use oktz_plugin_api::{Ctx, Reply, all_plugins, plugin};

/// `ICONS` (`totalfitur.js:20-29`) — urutan tidak memengaruhi hasil.
const IKON: [(&str, &str); 37] = [
    ("main", "🏠"),
    ("tools", "🔧"),
    ("downloader", "📥"),
    ("download", "📥"),
    ("sticker", "🎨"),
    ("ai", "🤖"),
    ("media", "📷"),
    ("game", "🎮"),
    ("rpg", "⚔️"),
    ("maker", "🖼️"),
    ("fun", "🎭"),
    ("group", "👥"),
    ("owner", "👑"),
    ("premium", "💎"),
    ("info", "📊"),
    ("search", "🔍"),
    ("canvas", "🎨"),
    ("anime", "🌸"),
    ("nsfw", "🔞"),
    ("utility", "🛠️"),
    ("economy", "💰"),
    ("stalker", "🔎"),
    ("random", "🎲"),
    ("religi", "🕌"),
    ("islamic", "☪️"),
    ("cek", "✅"),
    ("store", "🛒"),
    ("panel", "🖥️"),
    ("convert", "🔄"),
    ("primbon", "🔮"),
    ("tts", "🗣️"),
    ("otp", "🔑"),
    ("vps", "☁️"),
    ("pushkontak", "📱"),
    ("jpm", "🎰"),
    ("ephoto", "📸"),
    ("other", "📦"),
];

/// Rekap satu kategori — `cats[cat]` (`totalfitur.js:40`).
struct Rekap {
    kategori: String,
    total: usize,
    aktif: usize,
}

/// Galat saat menghitung distribusi.
///
/// Di JS blok perhitungan dibungkus `try` (`totalfitur.js:32`) dan `catch`
/// (`:74-77`) membalas template error + reaksi `☢`. Penghitungan Rust tidak
/// melakukan I/O dan praktis tidak pernah gagal; penhitungnya memakai
/// `checked_add` supaya luapan jadi galat biasa — bukan panic — dan ikut
/// masuk cabang `catch` yang sama.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GagalHitung;

/// Hitung total dan jumlah aktif per kategori, lalu urutkan total menurun.
///
/// `sort_by_key` stabil, jadi kategori dengan jumlah sama tetap mengikuti urutan
/// registry seperti `Object.entries(...).sort(...)` di JS (`totalfitur.js:51`).
fn rekap() -> Result<(Vec<Rekap>, usize, usize), GagalHitung> {
    let mut daftar: Vec<Rekap> = Vec::new();
    let (mut total, mut aktif) = (0usize, 0usize);

    for p in all_plugins() {
        let meta = &p.meta;
        // Plugin tanpa config dilewati di JS (`totalfitur.js:38`); setiap entri
        // registry punya meta, jadi tidak ada yang dilewati di sini.
        let kategori = if meta.category.is_empty() {
            "other"
        } else {
            meta.category.as_str()
        };

        let i = if let Some(i) = daftar.iter().position(|r| r.kategori.as_str() == kategori) {
            i
        } else {
            daftar.push(Rekap {
                kategori: kategori.to_string(),
                total: 0,
                aktif: 0,
            });
            daftar.len() - 1
        };

        daftar[i].total = daftar[i].total.checked_add(1).ok_or(GagalHitung)?;
        total = total.checked_add(1).ok_or(GagalHitung)?;
        // `p.config.isEnabled !== false` (`totalfitur.js:43`) — aktif kecuali
        // dinonaktifkan eksplisit; `meta.is_enabled` sudah embody itu.
        if meta.is_enabled {
            daftar[i].aktif = daftar[i].aktif.checked_add(1).ok_or(GagalHitung)?;
            aktif = aktif.checked_add(1).ok_or(GagalHitung)?;
        }
    }

    // `Object.entries(cats).sort((a, b) => b[1].total - a[1].total)`
    // (`totalfitur.js:51`) — total menurun.
    daftar.sort_by_key(|r| std::cmp::Reverse(r.total));
    Ok((daftar, total, aktif))
}

/// `ICONS[cat] || '📦'` (`totalfitur.js:56`) — kategori tanpa ikon pakai 📦.
fn ikon(kategori: &str) -> &'static str {
    IKON
        .iter()
        .find(|(nama, _)| *nama == kategori)
        .map_or("📦", |(_, emoji)| *emoji)
}

/// `((bagian / total) * 100).toFixed(1)` (`totalfitur.js:54`) — dibulatkan
/// setengah ke atas seperti `toFixed`, bukan setengah ke genap seperti format
/// float Rust.
fn persen(bagian: usize, total: usize) -> String {
    if total == 0 {
        return "NaN".to_string();
    }
    let sepuluh = (bagian * 1000 + total / 2) / total;
    format!("{}.{}", sepuluh / 10, sepuluh % 10)
}

/// `sock.sendTable(...)` (`totalfitur.js:62-72`) — tabel WhatsApp asli.
///
/// Judul `Distribusi Fitur`, baris judul `Kategori | Jumlah | Persen`, lalu
/// satu baris per kategori: ikon + nama besar, jumlah, dan persen. `headerText`
/// dan `footer` mengikuti `options` (`src/lib/rich-messages.js:616`).
fn tabel(daftar: &[Rekap], total: usize, aktif: usize) -> Reply {
    // `tableData` (`totalfitur.js:53-60`).
    let baris: Vec<Vec<String>> = daftar
        .iter()
        .map(|r| {
            vec![
                format!("{} {}", ikon(&r.kategori), r.kategori.to_uppercase()),
                r.total.to_string(),
                format!("{}%", persen(r.total, total)),
            ]
        })
        .collect();

    Reply::table(
        "Distribusi Fitur",
        ["Kategori", "Jumlah", "Persen"],
        baris,
    )
    .with_header_text(format!(
        "Total: {total} | Aktif: {aktif} | Kategori: {}",
        daftar.len()
    ))
    .with_table_footer(format!("Total {total} fitur tersedia"))
}

/// `catch` (`totalfitur.js:74-77`) — `m.react('☢')` lalu `m.reply(te(...))`.
fn balas_gagal(ctx: &Ctx) -> Reply {
    Reply::text(crate::common::error_template(ctx)).with_reaction("☢")
}

#[plugin(
    name = "totalfitur",
    alias = ["totalfeature", "totalcmd", "countplugin", "distribusi"],
    category = "main",
    description = "Lihat total fitur/command bot",
    usage = ".totalfitur",
    example = ".totalfitur",
    cooldown = 10,
    energi = 0,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn totalfitur(ctx: Ctx) -> Reply {
    match rekap() {
        // `m.react('📊')` (`totalfitur.js:49`) lalu `sock.sendTable(...)`
        // (`totalfitur.js:62-72`).
        Ok((daftar, total, aktif)) => tabel(&daftar, total, aktif).with_reaction("📊"),
        // `catch` (`totalfitur.js:74-77`) — reaksi `☢` + template error.
        Err(_) => balas_gagal(&ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("totalfitur").unwrap();
        assert_eq!(p.meta.name, "totalfitur");
        assert_eq!(
            p.meta.alias,
            ["totalfeature", "totalcmd", "countplugin", "distribusi"]
        );
        assert_eq!(p.meta.category, "main");
        assert_eq!(p.meta.description, "Lihat total fitur/command bot");
        assert_eq!(p.meta.usage, ".totalfitur");
        assert_eq!(p.meta.example, ".totalfitur");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
    }

    #[test]
    fn hitungan_mengikuti_registry() {
        let (daftar, total, aktif) = rekap().expect("registry tidak kosong");
        assert!(total > 0);
        assert_eq!(total, all_plugins().len());
        assert!(aktif <= total);
        assert_eq!(daftar.iter().map(|r| r.total).sum::<usize>(), total);
        assert_eq!(daftar.iter().map(|r| r.aktif).sum::<usize>(), aktif);
        assert!(daftar.iter().any(|r| r.kategori == "main"));
        assert!(daftar.windows(2).all(|w| w[0].total >= w[1].total));
    }

    #[test]
    fn balasan_adalah_tabel_whatsapp_dengan_judul_dan_footer() {
        let (daftar, total, aktif) = rekap().expect("registry tidak kosong");
        let r = totalfitur(Ctx::default());
        let balasan = r.without_mutations().tanpa_reaksi();

        let Reply::Table {
            title,
            headers,
            rows,
            header_text,
            footer,
        } = balasan
        else {
            panic!("harus Reply::Table, dapat {balasan:?}");
        };

        assert_eq!(title, "Distribusi Fitur");
        assert_eq!(
            headers.iter().map(String::as_str).collect::<Vec<_>>(),
            ["Kategori", "Jumlah", "Persen"]
        );
        assert_eq!(
            header_text.as_deref(),
            Some(
                format!(
                    "Total: {total} | Aktif: {aktif} | Kategori: {}",
                    daftar.len()
                )
                .as_str()
            )
        );
        assert_eq!(
            footer.as_deref(),
            Some(format!("Total {total} fitur tersedia").as_str())
        );

        // Satu baris per kategori, urutan sama dengan hasil sort.
        assert_eq!(rows.len(), daftar.len());
        for (baris, r) in rows.iter().zip(daftar.iter()) {
            assert_eq!(baris.len(), 3, "baris harus tiga kolom");
            assert_eq!(
                baris[0],
                format!("{} {}", ikon(&r.kategori), r.kategori.to_uppercase())
            );
            assert_eq!(baris[1], r.total.to_string());
            assert_eq!(baris[2], format!("{}%", persen(r.total, total)));
        }
    }

    #[test]
    fn reaksi_statistik_membungkus_tabelnya() {
        let r = totalfitur(Ctx::default());
        let Reply::Reaction { emoji, reply } = r.without_mutations() else {
            panic!("harus dibungkus reaksi, dapat {:?}", r.without_mutations());
        };
        assert_eq!(emoji, "📊");
        assert!(
            matches!(**reply, Reply::Table { .. }),
            "reaksi tidak boleh menutupi tabel"
        );
        assert!(r.mutations().is_empty(), "plugin ini hanya membaca");
    }

    #[test]
    fn keluaran_memuat_total_dan_kategori_main() {
        let (daftar, total, aktif) = rekap().expect("registry tidak kosong");
        let r = totalfitur(Ctx::default());
        let balasan = r.without_mutations().tanpa_reaksi();
        let Reply::Table {
            rows, header_text, ..
        } = balasan
        else {
            panic!("harus Reply::Table");
        };

        assert!(header_text
            .as_deref()
            .unwrap_or_default()
            .contains(&format!("Total: {total} | Aktif: {aktif} | Kategori: ")));
        assert!(rows.iter().any(|b| b[0] == "🏠 MAIN"));
        assert!(daftar.iter().any(|r| r.kategori == "main"));
    }

    #[test]
    fn cabang_gagal_membalas_template_error_dengan_reaksi_radiasi() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "totalfitur".into(),
            ..Default::default()
        };
        let r = balas_gagal(&ctx);
        let Reply::Reaction { emoji, reply } = r.without_mutations() else {
            panic!("harus dibungkus reaksi");
        };
        assert_eq!(emoji, "☢");
        let teks = reply.text_body().unwrap_or_default();
        assert!(teks.contains(".totalfitur"), "{teks}");
    }

    #[test]
    fn persen_satu_desimal_seperti_tofixed() {
        assert_eq!(persen(1, 16), "6.3");
        assert_eq!(persen(1, 3), "33.3");
        assert_eq!(persen(16, 16), "100.0");
        assert_eq!(persen(0, 0), "NaN");
    }
}
