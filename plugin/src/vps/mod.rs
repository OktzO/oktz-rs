//! Kategori `vps` — plugin DigitalOcean (`plugins/vps/*.js`).
//!
//! ## Penyimpangan: sumber token & daftar penjual
//!
//! JS membaca `config.digitalocean` (`config.js:301-306`):
//!
//! ```js
//! digitalocean: {
//!   token: "",
//!   region: "sgp1",
//!   sellers: [],
//!   ownerPanels: [],
//! },
//! ```
//!
//! Nilai itu ada di `oktz_core::config::Runtime::digitalocean`, diisi
//! `Runtime::from_config` dari `DIGITALOCEAN_TOKEN`, `DIGITALOCEAN_SELLERS`,
//! dan `DIGITALOCEAN_OWNER_PANELS` (daftar dipisah koma). Kosong = perilaku
//! bawaan `config.js` (`token: ""`), plugin membalas
//! `⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*`.
//!
//! Catatan: `Runtime` punya `api_keys.linode` (dari `LINODE_API_TOKEN`) —
//! itu dipakai plugin `panel/linode.js`, bukan kategori ini.

pub mod cekvps;
pub mod delvps;
pub mod listvps;
pub mod sisavps;
pub mod vpskontrol;

/// `config.digitalocean.token`.
pub(crate) fn token(ctx: &oktz_plugin_api::Ctx) -> String {
    ctx.config().digitalocean.token.clone()
}

/// `hasAccess(sender, isOwner)` — sama persis di kelima plugin `vps`.
pub(crate) fn has_access(ctx: &oktz_plugin_api::Ctx) -> bool {
    if ctx.is_owner {
        return true;
    }
    let clean = ctx.sender.split('@').next().unwrap_or(&ctx.sender);
    if clean.is_empty() {
        return false;
    }
    let dc = &ctx.config().digitalocean;
    dc.sellers.iter().any(|s| s == clean) || dc.owner_panels.iter().any(|s| s == clean)
}

/// `${nilai}` di dalam template literal JS — `String(x)`.
///
/// Field DigitalOcean diinterpolasi apa adanya, jadi nilai yang hilang ikut
/// tampil sebagai `undefined`, bukan sebagai string kosong.
pub(crate) fn js_str(v: Option<&serde_json::Value>) -> String {
    use serde_json::Value;
    match v {
        None | Some(Value::Null) => String::from("undefined"),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| match x {
                Value::Null => String::new(),
                other => js_str(Some(other)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => String::from("[object Object]"),
    }
}

/// `String(number)` JS — angka bulat tanpa desimal, pecahan dengan desimal
/// yang tidak berbunga nol.
pub(crate) fn fmt_num(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if n == n.trunc() && n.abs() < 1e21 {
        return format!("{}", n as i64);
    }
    let mut s = format!("{n}");
    if s.contains('e') {
        s = format!("{n:.17}");
    }
    s
}

/// Nama bulan panjang bahasa Indonesia — `Intl.DateTimeFormat('id-ID', {
/// month: 'long' })` (`src/lib/time.js:23-26`).
pub(crate) const BULAN: [&str; 12] = [
    "Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September",
    "Oktober", "November", "Desember",
];

/// Ubah timestamp ISO-8601 menjadi bagian waktu Asia/Jakarta (UTC+7).
///
/// `new Date(iso)` di JS memakai ISO-8601 yang bawaan offset; kalau stringnya
/// berakhiran `Z` berarti UTC, kalau tidak ada offset berarti waktu lokal
/// server — yang di produksi selalu UTC. Offset eksplisit (`+07:00`) juga
/// dihormati.
///
/// Kembalian: `(tahun, bulan 1-12, hari, jam, menit, detik)`.
pub(crate) fn jakarta(iso: &str) -> Option<(i64, u32, u32, u32, u32, u32)> {
    let s = iso.trim();
    let bytes = s.as_bytes();
    if bytes.len() < 10 {
        return None;
    }
    let tahun: i64 = s.get(0..4)?.parse().ok()?;
    let bulan: u32 = s.get(5..7)?.parse().ok()?;
    let hari: u32 = s.get(8..10)?.parse().ok()?;
    let (mut jam, mut menit, mut detik) = (0u32, 0u32, 0u32);
    if bytes.len() >= 19 {
        jam = s.get(11..13)?.parse().ok()?;
        menit = s.get(14..16)?.parse().ok()?;
        detik = s.get(17..19)?.parse().ok()?;
    }
    // Zona waktu: cari offset eksplisit di belakang string.
    let mut offset_menit: i64 = 0;
    let zona = &s[std::cmp::min(19, s.len())..];
    if let Some(pos) = zona.rfind(['+', '-']) {
        let off = &zona[pos + 1..];
        let (h, m) = if let Some((a, b)) = off.split_once(':') {
            (a.parse::<i64>().ok()?, b.parse::<i64>().ok()?)
        } else if off.len() >= 4 {
            (
                off.get(0..2)?.parse::<i64>().ok()?,
                off.get(2..4)?.parse::<i64>().ok()?,
            )
        } else {
            (off.parse::<i64>().ok()?, 0)
        };
        offset_menit = h * 60 + m;
        if zona.as_bytes()[pos] == b'+' {
            offset_menit = -offset_menit;
        }
    }
    // Ke waktu UTC, lalu tambah 7 jam (Asia/Jakarta tidak punya DST).
    let epoch = epoch_dari(tahun, bulan, hari, jam, menit, detik) - offset_menit * 60;
    let total = epoch + 7 * 3600;
    Some(dari_epoch(total))
}

/// Epoch detik untuk kalender proleptic Gregorian.
fn epoch_dari(y: i64, m: u32, d: u32, h: u32, mi: u32, s: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = ((m + 9) % 12) as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146097 + doe - 719468) * 86_400 + h as i64 * 3600 + mi as i64 * 60 + s as i64
}

/// `(tahun, bulan, hari, jam, menit, detik)` dari epoch detik (UTC).
fn dari_epoch(epoch: i64) -> (i64, u32, u32, u32, u32, u32) {
    let hari = epoch.div_euclid(86_400);
    let sisa = epoch.rem_euclid(86_400);
    let (jam, menit, detik) = (
        (sisa / 3600) as u32,
        ((sisa % 3600) / 60) as u32,
        (sisa % 60) as u32,
    );
    // hari sejak 1970-01-01 -> ymd, algoritma Howard Hinnant.
    let z = hari + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, jam, menit, detik)
}

/// `fromTimestamp(iso, "DD MMMM YYYY HH:mm:ss")` (`src/lib/time.js:112-114`).
pub(crate) fn format_tanggal(iso: &str) -> String {
    let Some((y, m, d, h, mi, s)) = jakarta(iso) else {
        // `new Date("bukan tanggal")` = Invalid Date; `Intl` lalu melempar
        // `RangeError`, dan `catch` di plugin DigitalOcean menutupnya jadi
        // template error. Nilai fallback dipakai hanya supaya tidak panic.
        return "Invalid Date".to_string();
    };
    format!(
        "{:02} {} {:04} {:02}:{:02}:{:02}",
        d,
        BULAN[(m as usize).saturating_sub(1).min(11)],
        y,
        h,
        mi,
        s
    )
}
