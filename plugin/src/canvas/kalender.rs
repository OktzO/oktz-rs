//! `plugins/canvas/kalender.js` — kalender Indonesia 2026 per bulan/setahun.
//!
//! ## BLOKIR sebagian — jalur render.
//!
//! Render memakai `@napi-rs/canvas` (header gradient + motif batik, kartu
//! bulan, legenda, 7 font yang diunduh + didaftarkan via `GlobalFonts`) yang
//! tidak ada padanannya di `Ctx`/`Reply`. Yang di-port penuh: parsing mode
//! (tahun/bulan), tabel hari libur, hitungan geometri kartu, caption, dan
//! kartu format salah. Jalur valid jatuh ke blok `catch` JS (`m.react('❌')` +
//! "GAGAL MEMBUAT KALENDER"), karena di sanalah render gagal berakhir.
//!
//! `ponytail:` kalau API canvas penuh tersedia di `Ctx` nanti, port ulang
//! `renderTahun`/`renderBulan` dari `plugins/canvas/kalender.js` (3 kolom,
//! skala kartu 1.6/2.1, statistik 17/8/25).

use oktz_plugin_api::{Ctx, Reply, plugin};

use crate::common::days_from_civil;

/// `TAHUN` (`kalender.js:22`).
pub(crate) const TAHUN: i64 = 2026;

/// `NAMA_BULAN` (`kalender.js:24`).
pub(crate) const NAMA_BULAN: [&str; 12] = [
    "Januari",
    "Februari",
    "Maret",
    "April",
    "Mei",
    "Juni",
    "Juli",
    "Agustus",
    "September",
    "Oktober",
    "November",
    "Desember",
];

/// `NAMA_HARI` (`kalender.js:25`).
#[cfg(test)]
pub(crate) const NAMA_HARI: [&str; 7] = ["SEN", "SEL", "RAB", "KAM", "JUM", "SAB", "MIN"];

/// Hari libur 2026 — `LIBUR` (`kalender.js:27-53`), kunci `"bulan-tanggal"`.
pub(crate) const LIBUR: [(&str, &str, &str); 25] = [
    ("1-1", "Tahun Baru Masehi", "nasional"),
    ("1-16", "Isra Mi'raj Nabi Muhammad SAW", "keagamaan"),
    ("2-16", "Cuti Bersama Imlek", "cuti"),
    ("2-17", "Tahun Baru Imlek 2577 Kongzili", "budaya"),
    ("3-18", "Cuti Bersama Nyepi", "cuti"),
    ("3-19", "Hari Raya Nyepi (Tahun Baru Saka 1948)", "keagamaan"),
    ("3-20", "Cuti Bersama Idul Fitri", "cuti"),
    ("3-21", "Hari Raya Idul Fitri 1447 H", "keagamaan"),
    ("3-22", "Hari Raya Idul Fitri 1447 H", "keagamaan"),
    ("3-23", "Cuti Bersama Idul Fitri", "cuti"),
    ("3-24", "Cuti Bersama Idul Fitri", "cuti"),
    ("4-3", "Wafat Yesus Kristus (Jumat Agung)", "keagamaan"),
    ("4-5", "Kebangkitan Yesus Kristus (Paskah)", "keagamaan"),
    ("5-1", "Hari Buruh Internasional", "nasional"),
    ("5-14", "Kenaikan Yesus Kristus", "keagamaan"),
    ("5-15", "Cuti Bersama Kenaikan Yesus Kristus", "cuti"),
    ("5-27", "Hari Raya Idul Adha 1447 H", "keagamaan"),
    ("5-28", "Cuti Bersama Idul Adha", "cuti"),
    ("5-31", "Hari Raya Waisak 2570 BE", "keagamaan"),
    ("6-1", "Hari Lahir Pancasila", "nasional"),
    ("6-16", "Tahun Baru Islam 1448 H", "keagamaan"),
    ("8-17", "Hari Kemerdekaan Republik Indonesia", "nasional"),
    ("8-25", "Maulid Nabi Muhammad SAW", "keagamaan"),
    ("12-24", "Cuti Bersama Natal", "cuti"),
    ("12-25", "Hari Raya Natal", "keagamaan"),
];

/// `NAMA_KE_NOMOR` (`kalender.js:545-548`).
pub(crate) fn nama_ke_nomor(nama: &str) -> Option<u32> {
    match nama {
        "januari" => Some(1),
        "februari" => Some(2),
        "maret" => Some(3),
        "april" => Some(4),
        "mei" => Some(5),
        "juni" => Some(6),
        "juli" => Some(7),
        "agustus" => Some(8),
        "september" => Some(9),
        "oktober" => Some(10),
        "november" => Some(11),
        "desember" => Some(12),
        _ => None,
    }
}

/// `jumlahHariDiBulan` (`kalender.js:81-83`) — `new Date(2026, bulan, 0)`.
pub(crate) fn jumlah_hari_di_bulan(bulan: u32) -> u32 {
    match bulan {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28, // 2026 bukan kabisat.
        _ => 0,
    }
}

/// `offsetHariPertama` (`kalender.js:85-88`) — indeks SEN=0.
pub(crate) fn offset_hari_pertama(bulan: u32) -> u32 {
    let days = days_from_civil(TAHUN, bulan, 1);
    ((days + 3).rem_euclid(7)) as u32
}

/// Satu libur bulan tertentu — `liburDiBulan` (`kalender.js:90-95`), urut tgl.
pub(crate) fn libur_di_bulan(bulan: u32) -> Vec<(u32, &'static str, &'static str)> {
    let mut out: Vec<(u32, &'static str, &'static str)> = LIBUR
        .iter()
        .filter_map(|(k, nama, tipe)| {
            let (b, t) = k.split_once('-')?;
            (b.parse::<u32>().ok()? == bulan)
                .then_some((t.parse::<u32>().ok()?, *nama, *tipe))
        })
        .collect();
    out.sort_by_key(|(tgl, _, _)| *tgl);
    out
}

/// `hitungTinggiKartuBulan` (`kalender.js:299-318`).
#[cfg(test)]
pub(crate) fn hitung_tinggi_kartu_bulan(bulan: u32, skala: f64) -> f64 {
    let tinggi_head = 58.0 * skala;
    let header_hari_y = tinggi_head + 20.0 * skala;
    let offset = offset_hari_pertama(bulan) as f64;
    let total_hari = jumlah_hari_di_bulan(bulan) as f64;
    let baris = ((offset + total_hari) / 7.0).ceil();
    let tinggi_sel = 34.0 * skala;
    let grafik_mulai_y = header_hari_y + 16.0 * skala;
    let y_tabel_selesai = grafik_mulai_y + tinggi_sel * baris + 8.0 * skala;

    let libur = libur_di_bulan(bulan).len() as f64;
    let tinggi_daftar = if libur == 0.0 {
        12.0 * skala
    } else {
        libur * 19.0 * skala
    };

    let y_daftar_mulai = y_tabel_selesai + 22.0 * skala;
    y_daftar_mulai + tinggi_daftar + 16.0 * skala
}

/// Caption hasil render tahun (`kalender.js:584-587`).
#[cfg(test)]
pub(crate) fn caption_tahun() -> String {
    "📅 *KALENDER INDONESIA 2026*\n\nBerikut adalah kalender lengkap tahun 2026 yang dilengkapi dengan seluruh hari libur nasional dan cuti bersama.".to_string()
}

/// Caption hasil render bulan (`kalender.js:591-594`).
#[cfg(test)]
pub(crate) fn caption_bulan(nama: &str) -> String {
    // `charAt(0).toUpperCase() + slice(1)` — input selalu huruf kecil.
    let mut chars = nama.chars();
    let proper = match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    format!(
        "📅 *KALENDER {} 2026*\n\nBerikut adalah kalender untuk bulan {} tahun 2026 yang dilengkapi dengan daftar hari libur.",
        proper.to_uppercase(),
        proper
    )
}

/// Pesan format salah (`kalender.js:565-568`).
const FORMAT_SALAH: &str = "❌ *FORMAT BULAN TIDAK VALID*\n\nSilakan masukkan nama bulan yang benar (contoh: *januari*, *februari*, dll) atau biarkan kosong untuk melihat kalender penuh selama setahun.";

/// Pesan `catch` (`kalender.js:603`).
const GAGAL: &str = "❌ *GAGAL MEMBUAT KALENDER*\n\nMaaf, sistem mengalami gangguan saat mencoba membuat gambar kalender. Silakan coba lagi nanti.";

#[plugin(
    name = "kalender",
    alias = ["calendar", "tanggalan"],
    category = "canvas",
    description = "Melihat kalender lengkap tahun 2026 atau per bulan.",
    usage = ".kalender [bulan/tahun]",
    example = ".kalender januari",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn kalender(ctx: Ctx) -> Reply {
    let input = ctx.text.trim().to_lowercase();

    // `if (text)` — teks kosong jatuh ke mode tahun (`kalender.js:556-570`).
    if !input.is_empty() && input != "tahun" && input != "2026" {
        if nama_ke_nomor(&input).is_none() {
            return Reply::text(FORMAT_SALAH);
        }
    }

    // Jalur render `@napi-rs/canvas` tidak tersedia — jatuh ke blok `catch`
    // JS: reaksi ❌ + pesan gagal. Lihat catatan kepala.
    Reply::text(GAGAL).with_reaction("❌")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "kalender".into(),
            text: text.into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        kalender(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn meta_dan_alias() {
        let p = oktz_plugin_api::find("kalender").unwrap();
        assert_eq!(p.meta.name, "kalender");
        assert_eq!(p.meta.alias, ["calendar", "tanggalan"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Melihat kalender lengkap tahun 2026 atau per bulan.");
        assert_eq!(p.meta.usage, ".kalender [bulan/tahun]");
        assert_eq!(p.meta.example, ".kalender januari");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        for a in ["calendar", "tanggalan"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "kalender");
        }
    }

    #[test]
    fn tabel_libur_sama_dengan_js() {
        assert_eq!(LIBUR.len(), 25);
        // Statistik header JS: 17 nasional? (label "Libur Nasional" = 17,
        // "Cuti Bersama" = 8, total 25). Hitung dari tabel:
        let nasional = LIBUR.iter().filter(|(.., t)| *t == "nasional").count();
        let cuti = LIBUR.iter().filter(|(.., t)| *t == "cuti").count();
        let keagamaan = LIBUR.iter().filter(|(.., t)| *t == "keagamaan").count();
        let budaya = LIBUR.iter().filter(|(.., t)| *t == "budaya").count();
        assert_eq!(nasional + cuti + keagamaan + budaya, 25);
        // Statistik JS: "17" Libur Nasional (nasional + keagamaan + budaya?),
        // "8" Cuti Bersama. Yang pasti: cuti = 8 dan total = 25.
        assert_eq!(cuti, 8);
        assert_eq!(nasional, 4);
        assert_eq!(keagamaan, 12);
        assert_eq!(budaya, 1);
    }

    #[test]
    fn jumlah_hari_dan_offset_sama_dengan_js() {
        // 2026: Jan 31, Feb 28, Apr 30...
        assert_eq!(jumlah_hari_di_bulan(1), 31);
        assert_eq!(jumlah_hari_di_bulan(2), 28);
        assert_eq!(jumlah_hari_di_bulan(4), 30);
        assert_eq!(jumlah_hari_di_bulan(12), 31);
        // 1 Januari 2026 = Kamis → offset (SEN=0) = 3.
        assert_eq!(offset_hari_pertama(1), 3);
        // 1 Maret 2026 = Minggu → offset 6.
        assert_eq!(offset_hari_pertama(3), 6);
        // NAMA_HARI urut SEN..MIN dipakai header kartu.
        assert_eq!(NAMA_HARI[0], "SEN");
        assert_eq!(NAMA_HARI[6], "MIN");
    }

    #[test]
    fn libur_di_bulan_urut_tanggal() {
        let maret = libur_di_bulan(3);
        let tgls: Vec<u32> = maret.iter().map(|(t, ..)| *t).collect();
        assert_eq!(tgls, vec![18, 19, 20, 21, 22, 23, 24]);
        assert_eq!(maret[0].1, "Cuti Bersama Nyepi");
        assert!(libur_di_bulan(7).is_empty());
        // Semua bulan punya entri yang benar: Agustus 2 libur.
        assert_eq!(libur_di_bulan(8).len(), 2);
    }

    #[test]
    fn tinggi_kartu_mengikuti_jumlah_libur_dan_baris() {
        // Kartu tanpa libur (Juli, 31 hari, offset?) lebih pendek dari Maret
        // yang punya 7 libur.
        let juli = hitung_tinggi_kartu_bulan(7, 1.6);
        let maret = hitung_tinggi_kartu_bulan(3, 1.6);
        assert!(juli < maret, "juli {juli} harus < maret {maret}");
        // Skala memperbesar.
        assert!(hitung_tinggi_kartu_bulan(7, 2.1) > juli);
    }

    #[test]
    fn nama_ke_nomor_lengkap() {
        for (i, nama) in NAMA_BULAN.iter().enumerate() {
            assert_eq!(
                nama_ke_nomor(&nama.to_lowercase()),
                Some((i + 1) as u32),
                "{nama}"
            );
        }
        assert_eq!(nama_ke_nomor("january"), None);
        assert_eq!(nama_ke_nomor(""), None);
    }

    #[test]
    fn caption_mengikuti_js() {
        assert!(caption_tahun().starts_with("📅 *KALENDER INDONESIA 2026*"));
        assert_eq!(
            caption_bulan("januari"),
            "📅 *KALENDER JANUARI 2026*\n\nBerikut adalah kalender untuk bulan Januari tahun 2026 yang dilengkapi dengan daftar hari libur."
        );
    }

    #[test]
    fn format_salah_dan_mode_valid() {
        // Bulan tidak dikenal → FORMAT_SALAH.
        assert_eq!(teks(&run("januarijuga")), FORMAT_SALAH);
        assert_eq!(teks(&run("2027")), FORMAT_SALAH);
        // Mode valid (tahun/kosong/2026/nama bulan) → jalur render gagal →
        // blok catch JS.
        for teks in ["", "tahun", "2026", "januari", "  Desember  "] {
            let r = run(teks);
            assert_eq!(
                r.without_mutations().text_body().unwrap(),
                GAGAL,
                "teks {teks:?}"
            );
            assert!(matches!(
                r.without_mutations(),
                Reply::Reaction { emoji, .. } if emoji == "❌"
            ));
        }
    }
}
