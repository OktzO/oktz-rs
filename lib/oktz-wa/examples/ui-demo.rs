//! Demo tampilan UI log (ui.rs) — data contoh, sama seperti mockup yang
//! disetujui owner. Jalankan: `cargo run -p oktz-wa --example ui-demo`
use chrono::{DateTime, TimeZone, Utc};
use oktz_wa::ui;

fn ts(h: u32, m: u32, s: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 16, h, m, s).unwrap()
}

fn main() {
    // TZ Jakarta supaya jam() = 09:41:05 dst, dan paksa tampil (stdout demo
    // bukan tty). Unsafe: single-threaded di awal main, sebelum chrono Local
    // membaca TZ. Format POSIX <+07>-7 = UTC+7 tanpa perlu tzdata (sandbox
    // produksi punya tzdata, boleh pakai nama zona).
    unsafe { std::env::set_var("TZ", "<+07>-7") };
    ui::tampil_selalu(true);

    // boot: log sistem gaya [tag]
    ui::sistem(
        "launcher",
        ui::KUNING,
        &format!(
            "node v22.23.3 · {}",
            ui::f(ui::ABU, "TZ Asia/Jakarta · rss 41.2 MB")
        ),
    );
    ui::sistem(
        "plugin",
        ui::VIOLET,
        &format!(
            "{} {} · {} {}",
            "87",
            ui::f(ui::ABU, "plugin dimuat"),
            "12",
            ui::f(ui::ABU, "kategori")
        ),
    );
    ui::sistem(
        "pairing",
        ui::CYAN_TERANG,
        &format!("device tersambung · {}", ui::f("", "+62 895-4111-2223")),
    );
    ui::sistem(
        "turso",
        ui::CYAN,
        &format!(
            "restore {} {}",
            ui::f("", "4"),
            ui::f(ui::ABU, "tabel · 1.2s")
        ),
    );

    // chat biasa (bukan command) → tanpa tanda
    ui::pesan_masuk(
        ts(2, 41, 5),
        "budi",
        "Keluarga Semur",
        "Woy pada! *jangan lupa* jam 3 siang ya",
    );
    ui::pesan_masuk(ts(2, 41, 38), "siti", "pribadi", "halo bot, .menu dong");
    ui::status_kirim(ts(2, 41, 39), true, "siti", 1.234, None);

    ui::sistem(
        "turso",
        ui::CYAN,
        &format!(
            "flush {} {}",
            ui::f("", "12"),
            ui::f(ui::ABU, "row · berikutnya 60s")
        ),
    );

    // plugin error spesifik → ✕
    ui::pesan_masuk(ts(2, 42, 11), "budi", "Keluarga Semur", ".ytmp3 lagu enak");
    ui::status_kirim(
        ts(2, 42, 13),
        false,
        "budi",
        0.0,
        Some("plugin: video tidak ditemukan"),
    );

    // plugin error generik "Command ada kendala" → ✕
    ui::pesan_masuk(ts(2, 42, 30), "rizky", "Keluarga Semur", ".sticker png");
    ui::status_kirim(
        ts(2, 42, 31),
        false,
        "rizky",
        0.0,
        Some("Command ada kendala"),
    );

    // media masuk, bukan command → tanpa tanda
    ui::pesan_masuk(
        ts(2, 43, 2),
        "budi",
        "Keluarga Semur",
        &format!(
            "{} {}",
            ui::f(ui::CYAN, "[ptt 00:47]"),
            ui::f(ui::ABU, "82.2 KB")
        ),
    );

    // warning & error sistem — gaya teks, tanpa ✓/✕
    ui::sistem(
        "launcher",
        ui::KUNING,
        &format!(
            "{} {} {}",
            ui::f(ui::KUNING, "!"),
            ui::f("", "ffmpeg tidak ditemukan"),
            ui::f(ui::ABU, "· stiker/audio nonaktif")
        ),
    );
    ui::sistem(
        "turso",
        ui::CYAN,
        &format!(
            "{} {}",
            ui::f(ui::MERAH, "flush gagal"),
            ui::f(ui::ABU, "· koneksi ulang 5s")
        ),
    );
}
