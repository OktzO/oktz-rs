//! UI log terminal gaya badge (desain baru owner): `[jam] [TIPE] nama(id) [REPLY] » isi`
//! + baris status `↳ ✓ Terkirim → X · ms`. Warna lewat `owo-colors`.
//!
//! ponytail: desain khusus Rust — JS referensi (gaya blok ▣) sengaja TIDAK
//! diikuti karena ini permintaan desain baru owner. Paritas pesan WhatsApp tidak
//! tersentuh; ini murni console UI.
//!
//! Semua cetakan hanya keluar kalau stdout tty (panel/terminal). Ke pipe/CI
//! otomatis diam + tanpa warna, supaya unit test pipeline yang memanggil
//! `process()` tidak berisik.

use chrono::{DateTime, Local, Utc};
use owo_colors::{OwoColorize, Style};
use std::io::IsTerminal;
use std::sync::OnceLock;

// SGR codes untuk jalur `sistem()` gaya lama (dipakai bot.rs) — owo-colors
// hanya untuk badge pesan & tag tracing.
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

/// Penanda "bot online" untuk deteksi panel Pterodactyl.
///
/// Panel menandai server Running saat string ini muncul di console
/// (`{ "done": ... }` di Start Configuration). String harus POLOS — tanpa
/// escape warna dan tanpa karakter regex (`[`, `(`, dst.) — karena Wings
/// mencocokkannya sebagai regex. Ubah di sini DAN di README kalau perlu.
pub const PENANDA_ONLINE: &str = "oktz bot online";

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

/// Tipe chat pesan masuk → badge berwarna.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipeChat {
    Private,
    Group,
    Channel,
    Status,
}

impl TipeChat {
    fn badge(self) -> &'static str {
        match self {
            TipeChat::Private => "[PRIVATE]",
            TipeChat::Group => "[GROUP]",
            TipeChat::Channel => "[CHANNEL]",
            TipeChat::Status => "[STATUS]",
        }
    }

    fn gaya(self) -> Style {
        match self {
            TipeChat::Private => Style::new().on_truecolor(68, 114, 196).white().bold(),
            TipeChat::Group => Style::new().on_truecolor(16, 124, 65).white().bold(),
            TipeChat::Channel => Style::new().on_truecolor(155, 48, 255).white().bold(),
            TipeChat::Status => Style::new().on_truecolor(96, 96, 96).white().bold(),
        }
    }
}

/// Bungkus teks dengan warna hanya kalau warna aktif; tanpa itu polos.
/// (owo-colors `.cyan()` selalu menulis escape, jadi gate-nya di sini.)
fn warnai(aktif: bool, s: &str, f: impl FnOnce(&str) -> String) -> String {
    if aktif { f(s) } else { s.to_string() }
}

/// Baris pesan masuk: `[jam] [TIPE] nama(id)[ [REPLY]] » isi`.
/// `isi` sudah ditentukan pemanggil (non-command = "NoCmd"); baris baru di
/// dalamnya ikut apa adanya. `aktif` = apakah warna dipakai (murni, gampang
/// dites tanpa bergantung tty global).
fn format_masuk(
    ts: DateTime<Utc>,
    tipe: TipeChat,
    nama: &str,
    id: &str,
    is_reply: bool,
    isi: &str,
    aktif: bool,
) -> String {
    let jam = warnai(aktif, &format!("[{}]", jam(ts)), |s| {
        s.bright_black().to_string()
    });
    let badge = warnai(aktif, tipe.badge(), |s| s.style(tipe.gaya()).to_string());
    let id = warnai(aktif, id, |s| s.bright_black().to_string());
    let reply = if is_reply {
        format!(
            " {}",
            warnai(aktif, "[REPLY]", |s| s.yellow().bold().to_string())
        )
    } else {
        String::new()
    };
    format!("{jam} {badge} {nama}({id}){reply} » {isi}")
}

pub fn baris_masuk(
    ts: DateTime<Utc>,
    tipe: TipeChat,
    nama: &str,
    id: &str,
    is_reply: bool,
    isi: &str,
) -> String {
    format_masuk(ts, tipe, nama, id, is_reply, isi, warna_aktif())
}

/// Rail pesan masuk. `chat` = tipe chat; `isi` = body (atau "NoCmd").
pub fn pesan_masuk(
    ts: DateTime<Utc>,
    tipe: TipeChat,
    nama: &str,
    id: &str,
    is_reply: bool,
    isi: &str,
) {
    if !boleh_tampil() {
        return;
    }
    println!("{}", baris_masuk(ts, tipe, nama, id, is_reply, isi));
}

/// Baris status kirim: `[jam] ↳ ✓ Terkirim → X · ms` / `↳ ✕ Gagal → X · alasan`.
/// Hanya dicetak kalau bot BENAR-BENAR mengirim (bukan chat biasa).
fn format_status(
    ts: DateTime<Utc>,
    ok: bool,
    tujuan: &str,
    dur_ms: f64,
    alasan: Option<&str>,
    aktif: bool,
) -> String {
    let jam = warnai(aktif, &format!("[{}]", jam(ts)), |s| {
        s.bright_black().to_string()
    });
    let branch = warnai(aktif, "↳", |s| s.bright_black().to_string());
    let (mark, sisa) = if ok {
        (
            warnai(aktif, "✓", |s| s.green().bold().to_string()),
            format!("Terkirim → {tujuan} · {dur_ms:.1}ms"),
        )
    } else {
        (
            warnai(aktif, "✕", |s| s.red().bold().to_string()),
            format!("Gagal → {tujuan}"),
        )
    };
    let sisa = warnai(aktif, &sisa, |s| s.bright_black().to_string());
    let alasan = alasan
        .map(|a| format!(" · {}", warnai(aktif, a, |s| s.bright_black().to_string())))
        .unwrap_or_default();
    format!("{jam} {branch} {mark} {sisa}{alasan}")
}

pub fn baris_status(
    ts: DateTime<Utc>,
    ok: bool,
    tujuan: &str,
    dur_ms: f64,
    alasan: Option<&str>,
) -> String {
    format_status(ts, ok, tujuan, dur_ms, alasan, warna_aktif())
}

/// Status kirim. `tujuan` = nama tampil tujuan; `alasan` diisi saat gagal.
pub fn status_kirim(ts: DateTime<Utc>, ok: bool, tujuan: &str, dur_ms: f64, alasan: Option<&str>) {
    if !boleh_tampil() {
        return;
    }
    println!("{}", baris_status(ts, ok, tujuan, dur_ms, alasan));
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
    fn format_masuk_badge_tipe_polos() {
        assert_eq!(
            format_masuk(
                contoh_ts(),
                TipeChat::Private,
                "oktz",
                "6285...",
                false,
                ".menu",
                false
            ),
            "[02:41:05] [PRIVATE] oktz(6285...) » .menu"
        );
        assert_eq!(
            format_masuk(
                contoh_ts(),
                TipeChat::Group,
                "zilll",
                "6285...",
                false,
                ".ping",
                false
            ),
            "[02:41:05] [GROUP] zilll(6285...) » .ping"
        );
        assert_eq!(
            format_masuk(
                contoh_ts(),
                TipeChat::Channel,
                "admin",
                "x",
                false,
                "post",
                false
            ),
            "[02:41:05] [CHANNEL] admin(x) » post"
        );
        assert_eq!(
            format_masuk(contoh_ts(), TipeChat::Status, "a", "b", false, "s", false),
            "[02:41:05] [STATUS] a(b) » s"
        );
    }

    #[test]
    fn format_masuk_tandai_reply() {
        assert_eq!(
            format_masuk(
                contoh_ts(),
                TipeChat::Private,
                "budi",
                "6281...",
                true,
                "91 x 29 berapa",
                false
            ),
            "[02:41:05] [PRIVATE] budi(6281...) [REPLY] » 91 x 29 berapa"
        );
        assert_eq!(
            format_masuk(
                contoh_ts(),
                TipeChat::Private,
                "budi",
                "6281...",
                false,
                "NoCmd",
                false
            ),
            "[02:41:05] [PRIVATE] budi(6281...) » NoCmd"
        );
    }

    #[test]
    fn format_masuk_pertahankan_baris_baru() {
        let out = format_masuk(
            contoh_ts(),
            TipeChat::Group,
            "a",
            "b",
            false,
            "baris1\nbaris2",
            false,
        );
        assert!(
            out.ends_with("» baris1\nbaris2"),
            "isi multi-baris terpotong: {out:?}"
        );
    }

    #[test]
    fn format_polos_tanpa_escape_warna_menyala_pakai_escape() {
        // aktif=false → tanpa tty sekalipun output polos
        assert!(
            !format_masuk(contoh_ts(), TipeChat::Group, "a", "b", true, ".x", false)
                .contains('\x1b')
        );
        assert!(!format_status(contoh_ts(), true, "a", 1.0, None, false).contains('\x1b'));
        // aktif=true → ada escape
        assert!(
            format_masuk(contoh_ts(), TipeChat::Group, "a", "b", true, ".x", true).contains('\x1b')
        );
        assert!(format_status(contoh_ts(), true, "a", 1.0, None, true).contains('\x1b'));
    }

    #[test]
    fn format_status_ok_dan_gagal() {
        assert_eq!(
            format_status(contoh_ts(), true, "oktz", 1072.7, None, false),
            "[02:41:05] ↳ ✓ Terkirim → oktz · 1072.7ms"
        );
        assert_eq!(
            format_status(
                contoh_ts(),
                false,
                "budi",
                0.0,
                Some("plugin: error"),
                false
            ),
            "[02:41:05] ↳ ✕ Gagal → budi · plugin: error"
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
    fn penanda_online_polos_untuk_regex_panel() {
        assert!(
            PENANDA_ONLINE
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == ' '),
            "penanda panel tidak boleh punya karakter regex/escape: {PENANDA_ONLINE:?}"
        );
    }

    #[test]
    fn cetak_paksa_ke_stdout_via_selalu() {
        tampil_selalu(true);
        pesan_masuk(
            contoh_ts(),
            TipeChat::Private,
            "budi",
            "6281...",
            true,
            ".menu",
        );
        status_kirim(contoh_ts(), true, "budi", 1.234, None);
        status_kirim(contoh_ts(), false, "rizky", 0.0, Some("plugin: panik"));
        sistem("turso", CYAN, &f(HIJAU, "restore 4 tabel"));
        tampil_selalu(false);
    }
}
