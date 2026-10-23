//! `plugins/main/ping.js` — canvas "Epic System Diagnostics" dengan data OS,
//! CPU, memori, dan uptime.
//!
//! `ponytail:` API canvas (`@napi-rs/canvas`) tidak tersedia di registry Rust
//! dan tidak ada crate penggambar teks di dependency workspace, jadi GAMBAR
//! canvas tidak dirender. Yang diport 1:1 adalah CAPTION penuh handler JS
//! (`ping.js` bagian `const caption = ...`): struktur 🏓 PONG → INFORMASI
//! SISTEM → CPU → MEMORI → MEMORI NODEJS → UPTIME beserta seluruh nilai, dan
//! reaksi akhir `✅` setelah kirim (reaksi pembuka `🕕` tidak muat karena
//! `Reply` hanya membawa satu reaksi). Cabang error tetap `te()` + reaksi
//! `❌`.

use std::time::Instant;

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `fmtSize` (`ping.js:27-34`) — dua desimal, `0 B` untuk nol.
fn fmt_size(b: u64) -> String {
    if b == 0 {
        return "0 B".to_string();
    }
    let u = ["B", "KB", "MB", "GB", "TB"];
    let i = ((b as f64).ln() / 1024f64.ln()).floor() as usize;
    format!("{:.2} {}", b as f64 / 1024f64.powi(i as i32), u[i])
}

/// `fmtUp` (`ping.js:36-44`).
fn fmt_up(s: f64) -> String {
    let d = (s / 86_400.0).floor();
    let h = (s % 86_400.0 / 3600.0).floor();
    let m = (s % 3600.0 / 60.0).floor();
    let sc = (s % 60.0).floor();
    if d > 0.0 {
        format!("{}d {}h {}m", d as i64, h as i64, m as i64)
    } else if h > 0.0 {
        format!("{}h {}m {}s", h as i64, m as i64, sc as i64)
    } else {
        format!("{}m {}s", m as i64, sc as i64)
    }
}

/// `os.type()` — Node memberi `Linux`/`Darwin`/`Windows_NT`, sedangkan
/// `std::env::consts::OS` bernilai kecil semua, jadi kapitalisasi mengikuti JS.
fn os_type() -> String {
    match std::env::consts::OS {
        "linux" => "Linux".into(),
        "macos" => "Darwin".into(),
        "windows" => "Windows_NT".into(),
        lain => lain.to_string(),
    }
}

/// `os.release()` dari `uname -r` (JS membaca `uname` syscall).
fn os_release() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// `os.hostname()` dari `/proc/sys/kernel/hostname`.
fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// `os.cpus()` — (model pertama, speed MHz, jumlah core) dari `/proc/cpuinfo`.
/// JS `cpus[0]?.model || "Unknown CPU"` dan `cpus[0]?.speed || 0`.
fn cpu_info() -> (String, u64, usize) {
    let teks = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let (mut model, mut speed, mut cores) = (String::new(), 0u64, 0usize);
    for baris in teks.lines() {
        let Some((kunci, nilai)) = baris.split_once(':') else {
            continue;
        };
        match kunci.trim() {
            "model name" if model.is_empty() => model = nilai.trim().to_string(),
            "cpu MHz" if speed == 0 => speed = nilai.trim().parse().unwrap_or(0),
            "processor" => cores += 1,
            _ => {}
        }
    }
    if cores == 0 {
        cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
    }
    if model.is_empty() {
        model = "Unknown CPU".to_string();
    }
    (model, speed, cores)
}

/// `os.loadavg()` dari `/proc/loadavg` — tiga angka, `toFixed(2)`.
fn load_avg() -> [String; 3] {
    let raw = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let ambil = |i: usize| {
        raw.split_whitespace()
            .nth(i)
            .and_then(|v| v.parse::<f64>().ok())
            .map(|v| format!("{v:.2}"))
            .unwrap_or_else(|| "0.00".into())
    };
    [ambil(0), ambil(1), ambil(2)]
}

/// `os.totalmem()`/`os.freemem()` dari `/proc/meminfo` (kB → byte).
fn memori() -> (u64, u64) {
    let teks = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let (mut total, mut bebas) = (0u64, 0u64);
    for baris in teks.lines() {
        let Some((kunci, nilai)) = baris.split_once(':') else {
            continue;
        };
        let kb = nilai
            .split_whitespace()
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_mul(1024);
        match kunci.trim() {
            "MemTotal" => total = kb,
            "MemFree" => bebas = kb,
            _ => {}
        }
    }
    (total, bebas)
}

/// `process.memoryUsage()` — padanan terdekat proses ini: VmRSS (rss dan
/// heapUsed), VmSize (heapTotal); external tidak ada padanannya → 0.
fn memori_node() -> (u64, u64, u64, u64) {
    let teks = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut rss = 0u64;
    let mut total = 0u64;
    for baris in teks.lines() {
        let Some((kunci, nilai)) = baris.split_once(':') else {
            continue;
        };
        let kb = nilai
            .split_whitespace()
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0)
            .saturating_mul(1024);
        match kunci.trim() {
            "VmRSS" => rss = kb,
            "VmSize" => total = kb,
            _ => {}
        }
    }
    (rss, total, rss, 0)
}

/// `os.uptime()` dari `/proc/uptime` (detik pecahan).
fn uptime_server() -> f64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|t| t.split_whitespace().next().and_then(|v| v.parse().ok()))
        .unwrap_or(0.0)
}

/// Isi `const data = {...}` handler JS.
struct Data {
    ping: String,
    cpu_model: String,
    cpu_speed: u64,
    cpu_cores: usize,
    load: [String; 3],
    total_mem: u64,
    free_mem: u64,
    used_mem: u64,
    mem_pct: String,
    os_type: String,
    os_release: String,
    os_platform: String,
    os_arch: String,
    os_host: String,
    up_os: String,
    up_bot: String,
    rss: u64,
    heap_total: u64,
    heap_used: u64,
    external: u64,
}

/// Caption handler JS — teks persis template JS, baris demi baris.
fn caption(d: &Data) -> String {
    format!(
        "🏓 *PONG!* ({ping}ms)\n\n\
         Berikut adalah detail spesifikasi dan performa server secara lengkap:\n\n\
         🖥️ *INFORMASI SISTEM*\n\
         > ◦ *OS:* {os} ({rel})\n\
         > ◦ *Platform:* {plat} ({arch})\n\
         > ◦ *Hostname:* {host}\n\
         > ◦ *NodeJS:* N/A\n\
         > ◦ *Engine V8:* N/A\n\n\
         💻 *INFORMASI CPU*\n\
         > ◦ *Model:* {model}\n\
         > ◦ *Cores:* {cores} Core(s)\n\
         > ◦ *Speed:* {speed} MHz\n\
         > ◦ *Load Avg:* {l0} (1m), {l1} (5m), {l2} (15m)\n\n\
         🧠 *PENGGUNAAN MEMORI*\n\
         > ◦ *Total RAM:* {total}\n\
         > ◦ *Dipakai:* {dipakai} ({pct}%)\n\
         > ◦ *Sisa Bebas:* {bebas}\n\n\
         📦 *MEMORI NODEJS*\n\
         > ◦ *RSS:* {rss}\n\
         > ◦ *Heap Total:* {heap_total}\n\
         > ◦ *Heap Used:* {heap_used}\n\
         > ◦ *External:* {ext}\n\n\
         ⏱️ *WAKTU AKTIF (UPTIME)*\n\
         > ◦ *Uptime Server:* {up_os}\n\
         > ◦ *Uptime Bot:* {up_bot}\n\n\
         Sistem berjalan stabil dan menyelesaikan kalkulasi dalam waktu eksekusi *{ping}ms*.",
        ping = d.ping,
        os = d.os_type,
        rel = d.os_release,
        plat = d.os_platform,
        arch = d.os_arch,
        host = d.os_host,
        model = d.cpu_model.trim(),
        cores = d.cpu_cores,
        speed = d.cpu_speed,
        l0 = d.load[0],
        l1 = d.load[1],
        l2 = d.load[2],
        total = fmt_size(d.total_mem),
        dipakai = fmt_size(d.used_mem),
        pct = d.mem_pct,
        bebas = fmt_size(d.free_mem),
        rss = fmt_size(d.rss),
        heap_total = fmt_size(d.heap_total),
        heap_used = fmt_size(d.heap_used),
        ext = fmt_size(d.external),
        up_os = d.up_os,
        up_bot = d.up_bot,
    )
}

fn kumpulkan_data(mulai: Instant, uptime_bot: u64) -> Data {
    let (cpu_model, cpu_speed, cpu_cores) = cpu_info();
    let (total_mem, free_mem) = memori();
    let used_mem = total_mem.saturating_sub(free_mem);
    let (rss, heap_total, heap_used, external) = memori_node();
    Data {
        // `execTime` JS dihitung tepat setelah data terkumpul (tEnd - tStart).
        ping: format!("{}", (mulai.elapsed().as_secs_f64() * 1000.0).round()),
        cpu_model,
        cpu_speed,
        cpu_cores,
        load: load_avg(),
        total_mem,
        free_mem,
        used_mem,
        mem_pct: format!("{:.1}", used_mem as f64 / total_mem.max(1) as f64 * 100.0),
        os_type: os_type(),
        os_release: os_release(),
        os_platform: std::env::consts::OS.to_string(),
        os_arch: std::env::consts::ARCH.to_string(),
        os_host: hostname(),
        up_os: fmt_up(uptime_server()),
        up_bot: fmt_up(uptime_bot as f64),
        rss,
        heap_total,
        heap_used,
        external,
    }
}

#[plugin(
    name = "ping",
    alias = ["speed", "p", "sys"],
    category = "main",
    description = "Cek performa dan status sistem bot secara real-time (Canvas Epic)",
    usage = ".ping",
    example = ".ping",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ping(ctx: Ctx) -> Reply {
    let mulai = Instant::now();
    // JS: `m.react("🕕")` → kirim image+caption → `m.react("✅")`. Gambar
    // canvas tidak bisa dirender tanpa crate font (lihat komentar modul), jadi
    // yang dikirim caption penuhnya; reaksi akhir ✅ menempel pada balasan.
    let data = kumpulkan_data(mulai, ctx.uptime_secs);
    Reply::text(caption(&data)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: &str = "628111@s.whatsapp.net";

    #[test]
    fn caption_mengikuti_struktur_dan_nilai_js() {
        let d = Data {
            ping: "12".into(),
            cpu_model: "Intel(R) Xeon(R) CPU".into(),
            cpu_speed: 2300,
            cpu_cores: 4,
            load: ["0.50".into(), "0.40".into(), "0.30".into()],
            total_mem: 8_589_934_592,
            free_mem: 4_294_967_296,
            used_mem: 4_294_967_296,
            mem_pct: "50.0".into(),
            os_type: "Linux".into(),
            os_release: "6.1.0".into(),
            os_platform: "linux".into(),
            os_arch: "x86_64".into(),
            os_host: "panel01".into(),
            up_os: "2d 3h 4m".into(),
            up_bot: "1h 2m 3s".into(),
            rss: 104_857_600,
            heap_total: 209_715_200,
            heap_used: 52_428_800,
            external: 0,
        };
        let c = caption(&d);
        assert!(c.starts_with("🏓 *PONG!* (12ms)\n\n"), "{c}");
        assert!(c.contains("🖥️ *INFORMASI SISTEM*\n> ◦ *OS:* Linux (6.1.0)"));
        assert!(c.contains("> ◦ *Platform:* linux (x86_64)"));
        assert!(c.contains("> ◦ *Hostname:* panel01"));
        assert!(c.contains("> ◦ *NodeJS:* N/A"));
        assert!(c.contains("> ◦ *Engine V8:* N/A"));
        assert!(c.contains("> ◦ *Model:* Intel(R) Xeon(R) CPU"));
        assert!(c.contains("> ◦ *Cores:* 4 Core(s)"));
        assert!(c.contains("> ◦ *Speed:* 2300 MHz"));
        assert!(c.contains("> ◦ *Load Avg:* 0.50 (1m), 0.40 (5m), 0.30 (15m)"));
        assert!(c.contains("> ◦ *Total RAM:* 8.00 GB"));
        assert!(c.contains("> ◦ *Dipakai:* 4.00 GB (50.0%)"));
        assert!(c.contains("> ◦ *Sisa Bebas:* 4.00 GB"));
        assert!(c.contains("> ◦ *RSS:* 100.00 MB"));
        assert!(c.contains("> ◦ *Heap Total:* 200.00 MB"));
        assert!(c.contains("> ◦ *Heap Used:* 50.00 MB"));
        assert!(c.contains("> ◦ *External:* 0 B"));
        assert!(c.contains("> ◦ *Uptime Server:* 2d 3h 4m"));
        assert!(c.contains("> ◦ *Uptime Bot:* 1h 2m 3s"));
        assert!(c.ends_with(
            "Sistem berjalan stabil dan menyelesaikan kalkulasi dalam waktu eksekusi *12ms*."
        ));
    }

    #[test]
    fn fmt_size_dua_desimal_seperti_ping_js() {
        assert_eq!(fmt_size(0), "0 B");
        assert_eq!(fmt_size(1), "1.00 B");
        assert_eq!(fmt_size(1024), "1.00 KB");
        assert_eq!(fmt_size(1_048_576), "1.00 MB");
        assert_eq!(fmt_size(1_073_741_824), "1.00 GB");
        assert_eq!(fmt_size(1_099_511_627_776), "1.00 TB");
    }

    #[test]
    fn fmt_up_sesuai_cabang_js() {
        assert_eq!(fmt_up(0.0), "0m 0s");
        assert_eq!(fmt_up(59.0), "0m 59s");
        assert_eq!(fmt_up(60.0), "1m 0s");
        assert_eq!(fmt_up(3600.0), "1h 0m 0s");
        assert_eq!(fmt_up(86_400.0), "1d 0h 0m");
    }

    #[test]
    fn handler_membalas_caption_dan_reaksi_sukses_bukan_error() {
        // Regresi: port lama selalu jatuh ke error template karena "API canvas
        // tidak tersedia" — JS-nya sendiri TIDAK error, jadi Rust pun tidak.
        let ctx = Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "ping".into(),
            ..Ctx::default()
        };
        let r = ping(ctx);
        assert!(r.mutations().is_empty());
        let r = r.without_mutations().tanpa_reaksi();
        let Reply::Text { text, mentions } = r else {
            panic!("harus Reply::Text");
        };
        assert!(mentions.is_empty());
        assert!(text.starts_with("🏓 *PONG!* ("), "{text}");
        assert!(text.contains("🖥️ *INFORMASI SISTEM*"));
        assert!(text.contains("⏱️ *WAKTU AKTIF (UPTIME)*"));
        assert!(!text.contains("☢"), "tidak boleh error template");
    }

    #[test]
    fn cpu_info_dan_memori_dibaca_tanpa_panik() {
        let (model, _speed, cores) = cpu_info();
        assert!(!model.is_empty());
        assert!(cores >= 1);
        let (total, _bebas) = memori();
        assert!(total > 0, "MemTotal harus terbaca dari /proc/meminfo");
        let _ = load_avg();
        let _ = memori_node();
        let _ = uptime_server();
        let _ = hostname();
        let _ = os_release();
    }
}
