//! UI log terminal gaya minimalis (rail 2 baris + status kirim ✓/✕ + log sistem).
//!
//! ponytail: desain tampilan khusus Rust — JS referensi (lib/ui/print.js gaya
//! blok ▣) sengaja TIDAK diikuti karena ini permintaan desain baru owner.
//! Paritas pesan WhatsApp tidak tersentuh; ini murni console UI.
//!
//! Semua cetakan hanya keluar kalau stdout tty (panel/terminal). Ke pipe/CI
//! otomatis diam + tanpa warna, supaya unit test pipeline yang memanggil
//! `process()` tidak berisik.

use chrono::{DateTime, Local, Utc};
use std::io::{IsTerminal, Write};
use std::sync::OnceLock;

// SGR codes — tanpa crate warna supaya dep tetap ramping.
pub const ABU: &str = "\x1b[90m";
pub const BIRU: &str = "\x1b[94m";
pub const HIJAU: &str = "\x1b[32m";
pub const MERAH: &str = "\x1b[91m";
pub const KUNING: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const VIOLET: &str = "\x1b[95m";
pub const CYAN_TERANG: &str = "\x1b[96m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

static ADA_TTY: OnceLock<bool> = OnceLock::new();
static PAKSA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Paksa tampil walau bukan tty (demo/unit test); warna ikut menyala.
pub fn tampil_selalu(v: bool) {
    PAKSA.store(v, std::sync::atomic::Ordering::Relaxed);
}

fn boleh_tampil() -> bool {
    *ADA_TTY
        .get_or_init(|| std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none())
        || PAKSA.load(std::sync::atomic::Ordering::Relaxed)
}

/// Bungkus teks dengan kode warna; tanpa tty balikan polos.
pub fn f(warna: &str, s: &str) -> String {
    if warna_aktif() && !warna.is_empty() {
        format!("{warna}{s}{RESET}")
    } else {
        s.to_string()
    }
}

fn warna_aktif() -> bool {
    boleh_tampil() && std::env::var_os("NO_COLOR").is_none()
}

pub fn jam(ts: DateTime<Utc>) -> String {
    ts.with_timezone(&Local).format("%H:%M:%S").to_string()
}

/// Nomor tampil dari jid: `62812...@s.whatsapp.net` → `+62812...`.
pub fn nomor(jid: &str) -> String {
    let d = jid.split('@').next().unwrap_or(jid);
    let d = d.split(':').next().unwrap_or(d);
    format!("+{d}")
}

/// Render markdown WA (*bold*, _italic_, ~strike~, ```mono```) ke ANSI.
fn render_md(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // mono ```...``` — isi tampil apa adanya
        if c == '`'
            && bytes[i..].starts_with(&['`', '`', '`'])
            && let Some(end) = find_seq(&bytes[i + 3..], &['`', '`', '`'])
        {
            let isi: String = bytes[i + 3..i + 3 + end].iter().collect();
            out += &f(CYAN, &isi);
            i += 3 + end + 3;
            continue;
        }
        let (tutup, warna): (char, &str) = match c {
            '*' => ('*', BOLD),
            '_' => ('_', "\x1b[3m"),
            '~' => ('~', "\x1b[9m"),
            _ => ('\0', ""),
        };
        if !warna.is_empty()
            && let Some(end) = find_seq(&bytes[i + 1..], &[tutup])
            && end > 0
        {
            let isi: String = bytes[i + 1..i + 1 + end].iter().collect();
            out += &f(warna, &render_md(&isi));
            i += end + 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Cari urutan karakter pertama di slice; balikin index-nya (None kalau absen).
fn find_seq(hay: &[char], needle: &[char]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Rail pesan masuk: `HH:MM:SS │ nama ~ chat` + isi ter-indent.
/// `chat` = nama grup / "pribadi"; `isi` = body mentah (markdown WA).
pub fn pesan_masuk(ts: DateTime<Utc>, nama: &str, chat: &str, isi: &str) {
    if !boleh_tampil() {
        return;
    }
    let rail = f(ABU, "│");
    let mut out = format!(
        "{} {rail} {} {} {}\n",
        f(ABU, &jam(ts)),
        f(BIRU, nama),
        f(ABU, "~"),
        f(ABU, chat),
    );
    for line in render_md(isi).lines() {
        out += &format!("{} {rail} {line}\n", " ".repeat(9));
    }
    let mut o = std::io::stdout().lock();
    let _ = o.write_all(out.as_bytes());
    let _ = o.flush();
}

/// Baris status: `✓ terkirim → tujuan · durasi` / `✕ gagal → tujuan · alasan`.
/// Hanya dicetak kalau bot BENAR-BENAR mengirim (bukan chat biasa).
pub fn status_kirim(ts: DateTime<Utc>, ok: bool, tujuan: &str, dur_ms: f64, alasan: Option<&str>) {
    if !boleh_tampil() {
        return;
    }
    let rail = f(ABU, "│");
    let (mark, warna_mark) = if ok { ("✓", HIJAU) } else { ("✕", MERAH) };
    let mut isi = if ok {
        format!(" terkirim → {tujuan} · {:.1}ms", dur_ms)
    } else {
        format!(" gagal → {tujuan}")
    };
    if let Some(a) = alasan {
        isi += &format!(" · {a}");
    }
    println!(
        "{} {rail} {}{}",
        f(ABU, &jam(ts)),
        f(warna_mark, mark),
        f(ABU, &isi),
    );
}

/// Log sistem gaya beda: `[tag] isi` — tag berwarna per subsistem, tanpa rail.
/// `isi` sudah diformat caller via `f()` (nilai terang, label redup).
pub fn sistem(tag: &str, tag_warna: &str, isi: &str) {
    if !boleh_tampil() {
        return;
    }
    println!("{} {}", f(tag_warna, &format!("[{tag}]")), isi);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contoh_ts() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-16T02:41:05Z")
            .unwrap()
            .into()
    }

    #[test]
    fn md_dirender_tanpa_tty_tetap_bersih() {
        // warna mati di test (bukan tty) → marker hilang, teks bersih
        assert_eq!(
            render_md("*halo* _dunia_ ~x~ ```kode```"),
            "halo dunia x kode"
        );
    }

    #[test]
    fn jam_pakai_zona_lokal() {
        // TZ belum di-set di test → cuma cek bentuk HH:MM:SS
        let j = jam(contoh_ts());
        assert_eq!(j.len(), 8);
        assert_eq!(j.as_bytes()[2], b':');
    }

    #[test]
    fn cetak_paksa_ke_string_buffer_via_selalu() {
        tampil_selalu(true);
        // kalau tidak panik berarti jalur cetak jalan; isi diverifikasi manual
        pesan_masuk(contoh_ts(), "budi", "Keluarga Semur", "*Woy* halo");
        status_kirim(contoh_ts(), true, "budi", 1.234, None);
        status_kirim(contoh_ts(), false, "rizky", 0.0, Some("plugin: panik"));
        sistem("turso", CYAN, &f(HIJAU, "restore 4 tabel"));
        tampil_selalu(false);
    }
}
