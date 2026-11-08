//! `plugins/search/play2.js` — pemutar lagu dengan LIRIK SINKRON.
//!
//! PENYIMPANGAN (satu baris per butir):
//! - `ytmusic-api` tidak ada: pencarian judul pakai `ytsearch::search` yang
//!   sama dengan jalur URL di JS; pesan "Lagu tidak ditemukan di YT Music"
//!   dipertahankan untuk jalur judul.
//! - Savetube + `createDecipheriv` tidak diport: unduhan audio memakai rantai
//!   `crate::download::ytmp3::unduh_audio` (`ytdl-native → izuka → nexray`).
//! - Kartu player HTML dikirim lewat `RichAi` (`Reply::TabelAi`) dengan
//!   section `GenAIaeacdsnwHtmlPrimitive` — sama seperti `sendMusicPlayer`
//!   (`play2.js:893-943`), submessage teksnya `"${title} - ${artist}"`.
//!   Lirik sinkron hidup di dalam player browser (JSON base64 + highlight
//!   `timeupdate`), bukan WS — jadi sync lyrics live JS tidak butuh wire
//!   baru. Voice note opus (ffmpeg `libopus` 16k/24kHz/mono/ogg) tetap
//!   dikirim sebagai fallback untuk klien yang tidak merender primitif HTML.
//! - Fallback lirik `ytm.getLyrics` (ytmusic-api) dan resize `sharp`
//!   dilewati (`ponytail:` thumbnail dikirim apa adanya, tanpa resize
//!   250x250 q50; upgrade path: tambah encoder image bila perlu).
//!
//! Yang dipor penuh karena murni dan bisa diuji: parser LRC, konversi lirik
//! polos menjadi lirik bertime, normalisasi, pemformat durasi,
//! `secondsFromTimestamp`, `escapeHtml`, dan kartu pakai.
#![allow(clippy::collapsible_if)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use oktz_plugin_api::{AiSubMessage, Ctx, HttpCall, MediaSource, Reply, RichAi, plugin};
use serde_json::json;

use crate::download::douyindl::query_encode;
use crate::download::ytmp3::{mimetype_for, unduh_audio};

use super::ytsearch;

/// `play2.js:954` — kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{prefix}play2 <judul lagu / URL YouTube>`\n\nKartu player interaktif dengan lagu full + lirik sinkron.\n\nContoh: `{prefix}play2 another love tom odell`"
    )
}

/// `play2.js:1104` — pesan `catch`.
pub(crate) fn gagal(message: &str) -> String {
    format!("❌ Gagal memproses lagu.\n\n> {message}")
}

/// `play2.js:980` — pesan yang dilempar saat pencarian tidak menghasilkan
/// entri untuk input judul.
pub(crate) const LAGU_TIDAK_DITEMUKAN: &str = "Lagu tidak ditemukan di YT Music";

/// `play2.js:1002` — `yts` tidak menemukan video untuk URL yang diberikan.
pub(crate) const VIDEO_TIDAK_DITEMUKAN: &str = "Video tidak ditemukan";

/// `play2.js:281-285` — regex ID video; pesan untuk ID yang tidak ketemu.
#[cfg(test)]
pub(crate) const URL_TIDAK_VALID: &str = "URL YouTube tidak valid";

/// `play2.js:295` / `:308` / `:327` / `:345` — pesan savetube; rantai resolver
/// pengganti hanya memakai `Download gagal`, sisanya disimpan sebagai teks JS.
#[allow(dead_code)]
pub(crate) const CDN_TIDAK_TERSEDIA: &str = "CDN tidak tersedia";
#[allow(dead_code)]
pub(crate) const METADATA_KOSONG: &str = "Metadata kosong";
#[allow(dead_code)]
pub(crate) const DECRYPT_GAGAL: &str = "Decrypt metadata gagal";
pub(crate) const DOWNLOAD_GAGAL: &str = "Download gagal";

/* CONFIG (`play2.js:12-46`). */

const LRCLIB_API: &str = "https://lrclib.net/api";
const LRCLIB_USER_AGENT: &str = "WA-Bot/1.0 (https://github.com/)";

/// `User-Agent` savetube (`play2.js:21`) — ikut dipakai `downloadAudioBuffer`.
const USER_AGENT: &str =
    "Mozilla/5.0 (Linux; Android 15) AppleWebKit/537.36 Chrome/130 Mobile Safari/537.36";

const MAX_ORIGINAL_AUDIO_MB: usize = 25;
const MAX_ORIGINAL_AUDIO_SIZE: usize = MAX_ORIGINAL_AUDIO_MB * 1024 * 1024;

const MAX_COMPRESSED_AUDIO_MB: usize = 6;
const MAX_COMPRESSED_AUDIO_SIZE: usize = MAX_COMPRESSED_AUDIO_MB * 1024 * 1024;

const MAX_BASE64_AUDIO_BYTES: usize = 8 * 1024 * 1024;

/* LIRIK (`play2.js:104-215`). */

/// `play2.js:104` — `parseLrcTimestamp`. Menolak `seconds >= 60`.
pub(crate) fn parse_lrc_timestamp(menit: f64, detik: f64, pecahan: Option<&str>) -> Option<f64> {
    if !menit.is_finite() || !detik.is_finite() || !(0.0..60.0).contains(&detik) {
        return None;
    }
    let milidetik = match pecahan {
        None => 0.0,
        Some("") => 0.0,
        Some(t) => {
            let angka: f64 = t.parse().unwrap_or(0.0);
            match t.chars().count() {
                1 => angka * 100.0,
                2 => angka * 10.0,
                _ => t.chars().take(3).collect::<String>().parse().unwrap_or(0.0),
            }
        }
    };
    Some(menit * 60.0 + detik + milidetik / 1000.0)
}

/// Satu baris lirik bertime.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Baris {
    pub time: f64,
    pub text: String,
}

/// `parseSyncedLyrics` (`play2.js:134-173`).
///
/// Baris tanpa timestamp atau tanpa teks dilewati; hasil diurutkan naik dan
/// duplikat (waktu SELISIH < 1 ms dan teks sama) dibuang.
pub(crate) fn parse_synced_lyrics(lrc: &str) -> Vec<Baris> {
    let mut result: Vec<Baris> = Vec::new();
    for raw in lrc.split('\n') {
        let raw = raw.trim_end_matches('\r');
        if raw.trim().is_empty() {
            continue;
        }
        let mut stamps: Vec<(f64, f64, String)> = Vec::new();
        // `rawLine.replace(timestampRegex, "")` membuang SEMUA timestamp,
        // termasuk yang muncul setelah teks — jadi sisanya dikumpulkan, bukan
        // hanya potongan setelah bracket pertama.
        let mut sisa = String::new();
        let mut cursor = 0;
        while let Some(open) = raw[cursor..].find('[') {
            let open = cursor + open;
            let Some(close_rel) = raw[open..].find(']') else {
                sisa.push_str(&raw[cursor..]);
                break;
            };
            let close = open + close_rel;
            sisa.push_str(&raw[cursor..open]);
            match pecah_timestamp(&raw[open + 1..close]) {
                Some((m, d, f)) => {
                    stamps.push((m, d, f));
                    cursor = close + 1;
                }
                // Kurung siku bukan timestamp: ikut dibuang oleh regex di JS.
                None => cursor = close + 1,
            }
        }
        if cursor < raw.len() {
            sisa.push_str(&raw[cursor..]);
        }
        if stamps.is_empty() {
            continue;
        }
        let text = sisa.trim().to_string();
        if text.is_empty() {
            continue;
        }
        for (m, d, f) in stamps {
            if let Some(time) = parse_lrc_timestamp(m, d, Some(f.as_str())) {
                if time.is_finite() && time >= 0.0 {
                    result.push(Baris {
                        time,
                        text: text.clone(),
                    });
                }
            }
        }
    }
    result.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));

    let mut cleaned: Vec<Baris> = Vec::new();
    for item in result {
        if let Some(last) = cleaned.last()
            && (last.time - item.time).abs() < 0.001
            && last.text == item.text
        {
            continue;
        }
        cleaned.push(item);
    }
    cleaned
}

/// Isi `[..]` — `(\d{1,3}):(\d{2})(?:[.:](\d{1,3}))?`.
fn pecah_timestamp(isi: &str) -> Option<(f64, f64, String)> {
    let (menit, sisanya) = isi.split_once(':')?;
    if menit.is_empty() || menit.len() > 3 || !menit.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (detik, pecahan) = match sisanya.split_once([':', '.']) {
        Some((d, f)) => (d, f),
        None => (sisanya, ""),
    };
    if detik.len() != 2 || !detik.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !pecahan.is_empty()
        && (pecahan.len() > 3 || !pecahan.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    Some((menit.parse().ok()?, detik.parse().ok()?, pecahan.to_string()))
}

/// `plainLyricsToSynced` (`play2.js:179-199`) — jarak antar baris 2..8 detik.
pub(crate) fn plain_lyrics_to_synced(lyrics: &str, durasi: f64) -> Vec<Baris> {
    if lyrics.is_empty() {
        return Vec::new();
    }
    let lines: Vec<&str> = lyrics
        .split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return Vec::new();
    }
    let mut interval = 5.0;
    if durasi.is_finite() && durasi > 0.0 && lines.len() > 1 {
        interval = (durasi / lines.len() as f64).clamp(2.0, 8.0);
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(i, text)| Baris {
            time: i as f64 * interval,
            text: text.to_string(),
        })
        .collect()
}

/// `normalizeLyrics` (`play2.js:201-215`) — buang waktu bukan angka dan teks
/// kosong setelah `trim()`, lalu urutkan.
pub(crate) fn normalize_lyrics(mut lyrics: Vec<Baris>) -> Vec<Baris> {
    lyrics.retain(|b| b.time.is_finite() && !b.text.trim().is_empty());
    for b in &mut lyrics {
        b.text = b.text.trim().to_string();
    }
    lyrics.retain(|b| !b.text.is_empty());
    lyrics.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    lyrics
}

/// `escapeHtml` (`play2.js:235-242`) — urutan penggantian penting: `&` dulu.
/// Hanya dipakai kartu HTML JS yang tidak diport; disimpan untuk paritas teks.
#[allow(dead_code)]
pub(crate) fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}

/// `escapeAttr` = `escapeHtml` (`play2.js:244-246`).
pub(crate) fn escape_attr(text: &str) -> String {
    escape_html(text)
}

/// Encoder base64 standar (dengan padding) — crate `base64` tidak ada di
/// workspace ini (`google.rs:180`).
pub(crate) fn base64_encode(data: &[u8]) -> String {
    const TABEL: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = chunk.len();
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        out.push(TABEL[(b[0] >> 2) as usize] as char);
        out.push(TABEL[(((b[0] & 3) << 4) | (b[1] >> 4)) as usize] as char);
        out.push(if n > 1 {
            TABEL[(((b[1] & 15) << 2) | (b[2] >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if n > 2 {
            TABEL[(b[2] & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Batas ukuran unduhan thumbnail poster.
const MAX_POSTER_BYTES: usize = 100 * 1024;

/// `getThumb` (`play2.js:485-500`) tanpa `sharp`: thumbnail apa adanya
/// sebagai `data:image/jpeg;base64,...`. Gagal/unduhan > 100 KB → `""`
/// (HTML memakai placeholder SVG, `play2.js:510-512`).
pub(crate) fn poster_base64(http: &dyn oktz_plugin_api::HttpGet, url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    let res = match http.call(&HttpCall::get(url).header("user-agent", USER_AGENT)) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    if !(200..300).contains(&res.status) {
        return String::new();
    }
    let data = res.body.bytes();
    if data.is_empty() || data.len() > MAX_POSTER_BYTES {
        return String::new();
    }
    format!("data:image/jpeg;base64,{}", base64_encode(data))
}

fn cache_cari() -> &'static Mutex<HashMap<String, Vec<ytsearch::Video>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Vec<ytsearch::Video>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// `ytsearch::search` + cache ringan per-query (lowercase, trim).
///
/// Hanya hasil non-kosong yang disimpan; error/kosong tidak di-cache.
/// `Max 64 entri, penuh → clear` — tanpa `shared` yang perlu diedit
/// (`ytsearch.rs` di luar cakupan), tanpa dependency baru.
/// `ponytail:` tanpa TTL; upgrade path: simpan waktu + evict per entri.
pub(crate) fn cari_video(
    http: &dyn oktz_plugin_api::HttpGet,
    q: &str,
) -> Result<Vec<ytsearch::Video>, oktz_plugin_api::HttpError> {
    let kunci = q.trim().to_lowercase();
    if let Some(v) = cache_cari()
        .lock()
        .ok()
        .and_then(|m| m.get(&kunci).cloned())
    {
        return Ok(v);
    }
    let v = ytsearch::search(http, q)?;
    if !v.is_empty()
        && let Ok(mut m) = cache_cari().lock()
    {
        if m.len() >= 64 {
            m.clear();
        }
        m.insert(kunci, v.clone());
    }
    Ok(v)
}

/// `secondsFromTimestamp` (`play2.js:248-257`) — 2 atau 3 bagian; selain itu 0.
pub(crate) fn seconds_from_timestamp(timestamp: &str) -> f64 {
    if timestamp.is_empty() {
        return 0.0;
    }
    let parts: Vec<f64> = timestamp
        .split(':')
        .map(|p| p.parse::<f64>().unwrap_or(f64::NAN))
        .collect();
    if parts.iter().any(|p| p.is_nan()) {
        return 0.0;
    }
    match parts.len() {
        2 => parts[0] * 60.0 + parts[1],
        3 => parts[0] * 3600.0 + parts[1] * 60.0 + parts[2],
        _ => 0.0,
    }
}

/// `formatDuration` (`play2.js:260-275`) — jam muncul hanya di atas nol.
pub(crate) fn format_duration(detik: f64) -> String {
    if !detik.is_finite() || detik <= 0.0 {
        return "0:00".to_string();
    }
    let total = detik.floor() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{:02}:{:02}", m, s)
    } else {
        format!("{m}:{s:02}")
    }
}

/// `/youtube\.com|youtu\.be/i.test(text)` (`play2.js:970`).
pub(crate) fn looks_like_youtube_url(text: &str) -> bool {
    let low = text.to_lowercase();
    low.contains("youtube.com") || low.contains("youtu.be")
}

/// `(?:v=|shorts\/|youtu\.be\/|embed\/)([a-zA-Z0-9_-]{11})` (`play2.js:1010`)
/// — kejadian pertama di URL, lalu tepat 11 karakter.
#[cfg(test)]
pub(crate) fn video_id_dari(url: &str) -> Option<String> {
    let awalan = ["v=", "shorts/", "youtu.be/", "embed/"];
    let mut kandidat: Option<(usize, &str)> = None;
    for p in awalan {
        if let Some(i) = url.find(p)
            && kandidat.is_none_or(|(j, _)| i < j)
        {
            kandidat = Some((i, p));
        }
    }
    let (i, p) = kandidat?;
    let sisa = &url[i + p.len()..];
    let id: String = sisa
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(11)
        .collect();
    (id.len() == 11).then_some(id)
}

/* LRCLIB (`play2.js:49-98`). */

/// Bentuk jawaban `getLRCLyrics` yang dipakai plugin.
pub(crate) struct LirikLagu {
    pub plain: String,
    pub synced: String,
}

/// `getLRCLyrics` (`play2.js:49-98`) — `None` = semua kegagalan (transport,
/// status tidak ok, body rusak) di JS ditelan `catch`/pemeriksaan status.
pub(crate) fn cari_lirik(
    http: &dyn oktz_plugin_api::HttpGet,
    judul: &str,
    artis: &str,
    durasi: f64,
) -> Option<LirikLagu> {
    if judul.is_empty() || artis.is_empty() {
        return None;
    }
    // `new URLSearchParams({ track_name, artist_name })` — serialisasi
    // x-www-form-urlencoded (spasi jadi `+`, persis `query_encode`).
    let mut url = format!(
        "{LRCLIB_API}/get?track_name={}&artist_name={}",
        query_encode(judul.trim()),
        query_encode(artis.trim())
    );
    // `Number.isFinite && 1 <= n <= 3600` → `String(Math.round(n))`.
    if durasi.is_finite() && (1.0..=3600.0).contains(&durasi) {
        url.push_str(&format!("&duration={}", durasi.round() as i64));
    }

    let res = http
        .call(
            &HttpCall::get(&url)
                .header("accept", "application/json")
                .header("user-agent", LRCLIB_USER_AGENT),
        )
        .ok()?;
    // `if (res.status === 404 || res.status === 429 || !res.ok) return null`.
    if res.status == 404 || res.status == 429 || !(200..300).contains(&res.status) {
        return None;
    }
    let data = res.json()?;

    let teks = |kunci: &str| -> String {
        data.get(kunci)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    Some(LirikLagu {
        plain: teks("plainLyrics"),
        synced: teks("syncedLyrics"),
    })
}

/* AUDIO (`play2.js:281-479`, unduhan lewat rantai `ytmp3::unduh_audio`). */

/// `downloadAudioBuffer` (`play2.js:375-389`).
fn unduh_buffer(ctx: &Ctx, url: &str) -> Result<Vec<u8>, String> {
    if url.is_empty() {
        return Err("URL audio kosong".to_string());
    }
    let res = ctx
        .http
        .call(&HttpCall::get(url).header("user-agent", USER_AGENT))
        .map_err(|_| DOWNLOAD_GAGAL.to_string())?;
    if !(200..300).contains(&res.status) {
        return Err(format!("Download audio gagal ({})", res.status));
    }
    let buffer = res.body.bytes().to_vec();
    if buffer.is_empty() || buffer.len() > MAX_ORIGINAL_AUDIO_SIZE {
        return Err("Buffer audio bermasalah".to_string());
    }
    Ok(buffer)
}

/// Argumen `compressAudio` (`play2.js:395-479`) — opus 16k/24kHz/mono/ogg.
/// `pipe:1` WAJIB: ffmpeg menolak jalan tanpa berkas keluaran
/// (`At least one output file must be specified`), dan JS selalu
/// mengirimkannya (`play2.js:419`).
const ARGUMEN_FFMPEG: [&str; 19] = [
    "-hide_banner",
    "-loglevel",
    "error",
    "-i",
    "pipe:0",
    "-vn",
    "-c:a",
    "libopus",
    "-b:a",
    "16k",
    "-ar",
    "24000",
    "-ac",
    "1",
    "-application",
    "audio",
    "-f",
    "ogg",
    "pipe:1",
];

/// `compressAudio` (`play2.js:395-479`) — stdin `pipe:0`, stdout `pipe:1`.
/// Batas 6 MB keluaran diperiksa di [`proses`] supaya bisa diuji tanpa
/// ffmpeg sungguhan.
fn kompres_audio(input: &[u8]) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Err("Input buffer kosong".to_string());
    }
    if oktz_plugin_api::ffmpeg::jalur_ffmpeg().is_none() {
        return Err("FFmpeg tidak ditemukan.".to_string());
    }
    let output = oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&ARGUMEN_FFMPEG, input)
        .map_err(|_| "FFmpeg gagal".to_string())?;
    if output.is_empty() {
        return Err("FFmpeg menghasilkan audio kosong".to_string());
    }
    Ok(output)
}

/// Tipe fungsi kompresi — parameternya dibuka supaya test bisa memalsukan
/// ffmpeg tanpa menyentuh env global.
type Kompresor = fn(&[u8]) -> Result<Vec<u8>, String>;

/// `createMusicPlayer` (`play2.js:506-887`) versi ringkas tapi fungsional:
/// poster, title/artist, `<audio>` dari `data:audio/ogg;base64,...`, dan
/// lirik sinkron dari JSON base64 (`timeupdate` menyorot baris aktif).
/// Placeholder SVG sama dengan JS ketika poster kosong (`:510-512`).
pub(crate) fn player_html(
    title: &str,
    artist: &str,
    durasi: &str,
    audio_src: &str,
    poster: &str,
    lyrics: &[Baris],
) -> String {
    let safe_title = escape_html(title);
    let safe_artist = escape_html(artist);
    let safe_dur = escape_html(durasi);
    let poster = if poster.is_empty() {
        "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0MDAiIGhlaWdodD0iNDAwIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjQwMCIgZmlsbD0iIzFhMGQxMiIvPjx0ZXh0IHg9IjIwMCIgeT0iMjEwIiBmb250LXNpemU9IjM0IiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZmlsbD0iI2ZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+TUFJTiBQQ0xAYVlFUlI8L3RleHQ+PC9zdmc+"
    } else {
        poster
    };
    let poster = escape_attr(poster);
    let lirik_json: Vec<serde_json::Value> = lyrics
        .iter()
        .map(|b| json!({ "time": b.time, "text": b.text }))
        .collect();
    let lirik_b64 = base64_encode(serde_json::to_string(&lirik_json).unwrap_or_default().as_bytes());
    format!(
        r#"<style>
*{{box-sizing:border-box;-webkit-tap-highlight-color:transparent}}
body{{margin:0;background:#1a0d12;color:#fff;font-family:Arial,Helvetica,sans-serif}}
.wrap{{display:flex;justify-content:center;padding:10px}}
.player{{width:100%;max-width:330px;border-radius:18px;overflow:hidden;background:#1a0d12;box-shadow:0 18px 40px rgba(0,0,0,.5)}}
.content{{padding:16px 18px}}
.poster{{width:100%;aspect-ratio:1;border-radius:10px;overflow:hidden;background:rgba(255,255,255,.06);margin-bottom:14px}}
.poster img{{width:100%;height:100%;object-fit:cover;display:block}}
.title{{font-size:17px;font-weight:600;line-height:1.3}}
.artist{{font-size:12px;color:#b9b1b6;margin-top:3px}}
.lyrics{{height:82px;overflow-y:auto;margin:10px 0;font-size:11px;line-height:1.5}}
.ln{{color:rgba(255,255,255,.45);padding:2px 0}}
.ln.on{{color:#fff;font-weight:700;font-size:12px}}
.bar{{height:4px;border-radius:4px;background:rgba(255,255,255,.22);cursor:pointer;margin-bottom:6px}}
.fill{{height:100%;width:0;background:#fff;border-radius:4px}}
.time{{display:flex;justify-content:space-between;font-size:11px;color:#b9b1b6;margin-bottom:12px}}
.ctrl{{display:flex;justify-content:center;gap:18px;align-items:center}}
button{{width:52px;height:52px;border-radius:50%;background:#fff;color:#12070b;border:none;cursor:pointer;font-size:20px}}
</style>
<div class="wrap"><div class="player"><div class="content">
<div class="poster"><img src="{poster}" alt="{safe_title}"></div>
<div class="title">{safe_title}</div>
<div class="artist">{safe_artist}</div>
<div class="lyrics" id="ly"></div>
<div class="bar" id="bar"><div class="fill" id="fill"></div></div>
<div class="time"><span id="cur">0:00</span><span>{safe_dur}</span></div>
<div class="ctrl"><button id="btn" aria-label="Play">▶</button></div>
</div></div></div>
<audio id="au" preload="metadata" src="{audio_src}"></audio>
<script>
(function(){{
'use strict';
var au=document.getElementById('au'),btn=document.getElementById('btn'),
fill=document.getElementById('fill'),cur=document.getElementById('cur'),
ly=document.getElementById('ly'),bar=document.getElementById('bar'),lines=[],els=[];
try{{
var bin=atob('{lirik_b64}'),by=new Uint8Array(bin.length);
for(var i=0;i<bin.length;i++)by[i]=bin.charCodeAt(i);
lines=JSON.parse(new TextDecoder('utf-8').decode(by));
}}catch(e){{lines=[];}}
function fmt(s){{if(!isFinite(s)||s<0)s=0;var m=Math.floor(s/60);return m+':'+String(Math.floor(s%60)).padStart(2,'0');}}
function render(){{
ly.innerHTML='';els=[];
if(!lines.length){{ly.innerHTML='<div class="ln">Lirik belum tersedia untuk lagu ini.</div>';return;}}
var f=document.createDocumentFragment();
lines.forEach(function(l){{var d=document.createElement('div');d.className='ln';d.textContent=l.text||'♪';f.appendChild(d);els.push(d);}});
ly.appendChild(f);
}}
render();
var act=-1;
function upd(){{
var t=au.currentTime||0,d=au.duration||0;
cur.textContent=fmt(t);
if(d>0)fill.style.width=(t/d*100)+'%';
var ix=-1;for(var i=0;i<lines.length;i++){{if(lines[i].time<=t)ix=i;else break;}}
if(ix!==act){{act=ix;els.forEach(function(e,i){{e.classList.toggle('on',i===ix);}});if(ix>=0&&els[ix])els[ix].scrollIntoView({{block:'center'}});}}
}}
btn.addEventListener('click',function(){{if(au.paused){{au.play();}}else{{au.pause();}}}});
au.addEventListener('play',function(){{btn.textContent='❚❚';}});
au.addEventListener('pause',function(){{btn.textContent='▶';}});
au.addEventListener('timeupdate',upd);
au.addEventListener('ended',function(){{act=-1;upd();}});
bar.addEventListener('click',function(e){{var d=au.duration||0;if(d<=0)return;var r=bar.getBoundingClientRect();au.currentTime=(e.clientX-r.left)/r.width*d;upd();}});
}})();
</script>"#
    )
}

/// Section HTML `GenAIaeacdsnwHtmlPrimitive` — `sendMusicPlayer`
/// (`play2.js:916-927`).
pub(crate) fn section_html(html: &str) -> serde_json::Value {
    json!({
        "view_model": {
            "primitive": {
                "__typename": "GenAIaeacdsnwHtmlPrimitive",
                "payload": html,
                "trusted_sources": [],
            },
            "__typename": "GenAISingleLayoutViewModel",
        }
    })
}

/* HANDLER (`play2.js:949-1107`). */

/// Alur `try` (`play2.js:960-1099`). `Err` memuat `error.message` untuk
/// template `catch`.
fn proses(ctx: &Ctx, text: &str, kompres: Kompresor) -> Result<Reply, String> {
    // JS: judul → `ytmusic.search`, URL → `yts(ytUrl)`; di sini keduanya
    // `ytsearch::search` (lihat catatan deviasi di header).
    let adalah_url = looks_like_youtube_url(text);
    let pesan_kosong = if adalah_url {
        VIDEO_TIDAK_DITEMUKAN
    } else {
        LAGU_TIDAK_DITEMUKAN
    };
    let videos = cari_video(ctx.http.as_ref(), text).map_err(|_| pesan_kosong.to_string())?;
    let video = videos.first().ok_or_else(|| pesan_kosong.to_string())?;

    let title = if video.title.is_empty() {
        "Unknown"
    } else {
        &video.title
    };
    let artist = if video.author_name.is_empty() {
        "YouTube"
    } else {
        &video.author_name
    };
    let duration = if video.timestamp.is_empty() {
        "0:00"
    } else {
        &video.timestamp
    };
    let duration_sec = seconds_from_timestamp(duration);

    // Lirik: LRCLIB sinkron → LRCLIB polos. Fallback `ytm.getLyrics` tidak ada
    // padanannya (deviasi di header).
    let mut synced: Vec<Baris> = Vec::new();
    if let Some(lrc) = cari_lirik(ctx.http.as_ref(), title, artist, duration_sec) {
        if !lrc.synced.is_empty() {
            synced = parse_synced_lyrics(&lrc.synced);
        }
        if synced.is_empty() && !lrc.plain.is_empty() {
            synced = plain_lyrics_to_synced(&lrc.plain, duration_sec);
        }
    }
    let synced = normalize_lyrics(synced);

    // Unduhan audio: `savetubeRetry` di JS → rantai `unduh_audio` di sini.
    let audio = unduh_audio(ctx, &video.url).map_err(|_| DOWNLOAD_GAGAL.to_string())?;
    let asli = unduh_buffer(ctx, &audio.url)?;
    let kompresi = kompres(&asli)?;
    if kompresi.len() > MAX_COMPRESSED_AUDIO_SIZE {
        return Err("Audio compress terlalu besar".to_string());
    }

    // `data:audio/ogg;base64,...` melebihi 8 MB → batas pesan WA.
    let base64_len = kompresi.len().div_ceil(3) * 4;
    if base64_len > MAX_BASE64_AUDIO_BYTES {
        return Err(
            "Audio Base64 masih terlalu besar, kurangi bitrate FFMPEG.".to_string(),
        );
    }

    // `sendMusicPlayer(sock, m.chat, html, "${title} - ${artist}")`
    // (`play2.js:1098`): poster `getThumb` → `imageSrc`, audio
    // `data:audio/ogg;base64` dari buffer kompresi, lirik sinkron di
    // player HTML browser.
    let poster = poster_base64(ctx.http.as_ref(), &video.thumbnail);
    let audio_src = format!(
        "data:audio/ogg;base64,{}",
        base64_encode(&kompresi)
    );
    let html = player_html(
        title,
        artist,
        &format_duration(duration_sec),
        &audio_src,
        &poster,
        &synced,
    );
    let rich = RichAi {
        submessages: vec![AiSubMessage {
            text: Some(format!("{title} - {artist}")),
            tabel: None,
        }],
        sections: vec![section_html(&html)],
        disclaimer: None,
        // `sendMusicPlayer` (`play2.js:896-943`) tidak mengutip pemicu.
        quoted: false,
    };

    Ok(Reply::sequence(vec![
        Reply::reaction("🎧"),
        // Fallback voice note untuk klien yang tidak merender primitif HTML.
        // `compressAudio` menghasilkan ogg/opus — mimetype sama dengan
        // `mimetypeFor("opus")`.
        Reply::voice_note(
            MediaSource::bytes(kompresi).with_mimetype(mimetype_for("opus")),
        ),
        Reply::TabelAi(Box::new(rich)).with_reaction("✅"),
    ]))
}

#[plugin(
    name = "play2",
    alias = ["spotifywebui", "playweb"],
    category = "search",
    description = "Kirim kartu music player interaktif (AIRICH) dengan lagu full + lirik sinkron",
    usage = ".play2 <judul lagu / URL YouTube>",
    example = ".play2 another love tom odell",
    cooldown = 15,
    energi = 1,
    is_enabled = true,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn play2(ctx: Ctx) -> Reply {
    jalankan(ctx, kompres_audio)
}

fn jalankan(ctx: Ctx, kompres: Kompresor) -> Reply {
    let text = ctx.text.trim();

    if text.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    // `await m.react('🎧')` di `:958`; `catch` menggantinya dengan ❌.
    match proses(&ctx, text, kompres) {
        Ok(reply) => reply,
        Err(message) => Reply::text(gagal(&message)).with_reaction("❌"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub HTTP multi-URL: pencarian YouTube (POST via `call`), LRCLIB,
    /// rantai backend ytmp3, dan host audio `cdn.example` — dicocokkan dari
    /// potongan URL. `None` = transport error.
    #[derive(Debug)]
    struct Stub {
        per_url: Vec<(&'static str, Option<Value>)>,
        /// Body biner untuk URL audio (bukan JSON).
        audio: Option<Vec<u8>>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn baru(
            per_url: Vec<(&'static str, Option<Value>)>,
            audio: Option<Vec<u8>>,
        ) -> Arc<Stub> {
            Arc::new(Stub {
                per_url,
                audio,
                seen: Mutex::new(Vec::new()),
            })
        }
        fn jawab(&self, url: &str) -> Result<HttpResponse, HttpError> {
            if url.contains("cdn.example") {
                let data = self
                    .audio
                    .as_ref()
                    .ok_or(HttpError {
                        message: "audio offline".into(),
                    })?
                    .as_slice();
                return Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(data),
                        mimetype: Some("audio/mpeg".into()),
                    },
                    headers: vec![],
                });
            }
            let (_, nilai) = self
                .per_url
                .iter()
                .find(|(penanda, _)| url.contains(penanda))
                .ok_or(HttpError {
                    message: format!("URL tak dikenal stub: {url}"),
                })?;
            let text = nilai
                .clone()
                .ok_or(HttpError { message: "offline".into() })?
                .to_string();
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            self.jawab(url)
        }
        fn post_json(&self, url: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            self.jawab(url)
        }
        fn post_form(&self, url: &str, _: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            self.jawab(url)
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET/POST JSON")
        }
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            self.jawab(&req.url)
        }
    }

    fn run_http(http: Arc<Stub>, text: &str) -> Reply {
        jalankan(
            Ctx {
                prefix: ".".into(),
                command: "play2".into(),
                sender: "628111@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: text.into(),
                args: text.split_whitespace().map(str::to_string).collect(),
                http,
                ..Default::default()
            },
            |b| Ok(b.to_vec()),
        )
    }

    fn run(text: &str) -> Reply {
        run_http(Stub::baru(vec![("", None)], None), text)
    }

    /// Decode base64 standar untuk memeriksa JSON lirik di payload HTML.
    fn decode_base64_test(input: &str) -> Vec<u8> {
        fn nilai(c: u8) -> u8 {
            match c {
                b'A'..=b'Z' => c - b'A',
                b'a'..=b'z' => c - b'a' + 26,
                b'0'..=b'9' => c - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => 0,
            }
        }
        let bersih: Vec<u8> = input.bytes().filter(|&c| c != b'=').collect();
        bersih
            .chunks(4)
            .flat_map(|c| {
                let n = (nilai(c[0]) as u32) << 18
                    | (nilai(c[1]) as u32) << 12
                    | (nilai(*c.get(2).unwrap_or(&0)) as u32) << 6
                    | nilai(*c.get(3).unwrap_or(&0)) as u32;
                let b = [(n >> 16) as u8, (n >> 8) as u8, n as u8];
                b[..c.len().saturating_sub(1)].to_vec()
            })
            .collect()
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        let mut cur = r;
        loop {
            match cur {
                Reply::Reaction { emoji, .. } => return Some(emoji),
                Reply::Mutate { reply, .. } => cur = reply,
                _ => return None,
            }
        }
    }

    /// Hasil pencarian YouTube dengan satu video.
    const HASIL: &str = r#"{"contents":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[
      {"itemSectionRenderer":{"contents":[
        {"videoRenderer":{
          "videoId":"aaaaaaaaaaa",
          "title":{"runs":[{"text":"Lagu Uji"}]},
          "ownerText":{"runs":[{"text":"Channel Uji"}]},
          "viewCountText":{"simpleText":"1,234,567 views"},
          "publishedTimeText":{"simpleText":"2 years ago"},
          "lengthText":{"simpleText":"3:33"}
        }}
      ]}}
    ]}}}}]}}"#;

    const LIRIK: &str = r#"{"id":1,"trackName":"Lagu Uji","artistName":"Channel Uji","duration":213,"instrumental":false,"plainLyrics":"","syncedLyrics":"[00:12.00]Halo\n[00:15.50]Dunia"}"#;

    fn stub_sukses() -> Arc<Stub> {
        stub_dengan_audio(Some(b"ID3data-audio-palsu".to_vec()))
    }

    fn stub_dengan_audio(audio: Option<Vec<u8>>) -> Arc<Stub> {
        Stub::baru(
            vec![
                ("youtubei/v1/search", Some(serde_json::from_str(HASIL).unwrap())),
                ("lrclib.net/api/get", Some(serde_json::from_str(LIRIK).unwrap())),
                // ytdl-native gagal → jatuh ke izuka.
                ("ymcdn", Some(serde_json::json!({"status": false}))),
                (
                    "izuka",
                    Some(serde_json::json!({
                        "status": true,
                        "result": {
                            "title": "Lagu Uji",
                            "format": "mp3",
                            "download_url": "https://cdn.example/lagu.mp3"
                        }
                    })),
                ),
            ],
            audio,
        )
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("play2").unwrap();
        assert_eq!(p.meta.name, "play2");
        assert_eq!(p.meta.alias, ["spotifywebui", "playweb"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(
            p.meta.description,
            "Kirim kartu music player interaktif (AIRICH) dengan lagu full + lirik sinkron"
        );
        assert_eq!(p.meta.usage, ".play2 <judul lagu / URL YouTube>");
        assert_eq!(p.meta.example, ".play2 another love tom odell");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["spotifywebui", "playweb"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "play2");
        }
    }

    #[test]
    fn an_empty_query_shows_the_help_card() {
        for text in ["", "  ", "\t\n"] {
            let t = text_of(&run(text));
            assert!(t.starts_with("⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*"), "input {text:?}");
            assert!(t.contains("`.play2 <judul lagu / URL YouTube>`"), "{t}");
            assert!(t.ends_with("Contoh: `.play2 another love tom odell`"), "{t}");
        }
        assert!(help("!").contains("`!play2 another love tom odell`"));
    }

    #[test]
    fn alur_sukses_mengirim_lirik_sinkron_lalu_voice_note() {
        let http = stub_sukses();
        let r = run_http(http.clone(), "another love tom odell");
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus sequence, dapat {:?}", r.without_mutations());
        };
        // 🎧 → voice note fallback → kartu RichAi HTML.
        assert_eq!(list.len(), 3);
        assert!(matches!(&list[0], Reply::Reaction { emoji, .. } if emoji == "🎧"));
        match &list[1] {
            Reply::Audio { media, ptt } => {
                assert!(ptt, "voice note = ptt true");
                let m = media.as_ref().expect("audio harus punya sumber");
                assert_eq!(m.mimetype(), Some("audio/ogg"));
            }
            lain => panic!("harus voice note, dapat {lain:?}"),
        }
        // `with_reaction("✅")` membungkus di Mutate+Reaction; unwrap ke TabelAi.
        let Reply::TabelAi(rich) = list[2].without_mutations().tanpa_reaksi() else {
            panic!("harus TabelAi");
        };
        assert_eq!(rich.submessages[0].text.as_deref(), Some("Lagu Uji - Channel Uji"));
        let prim = &rich.sections[0]["view_model"]["primitive"];
        assert_eq!(
            prim["__typename"],
            serde_json::json!("GenAIaeacdsnwHtmlPrimitive")
        );
        let payload = prim["payload"].as_str().expect("payload html");
        assert!(matches!(list[2].without_mutations(), Reply::Reaction { emoji, .. } if emoji == "✅"));
        assert!(payload.contains("Lagu Uji"), "{payload}");
        assert!(payload.contains("data:audio/ogg;base64,"), "{payload}");
        assert!(payload.contains("atob("), "lirik sinkron dari JSON base64");

        let seen = http.seen.lock().unwrap().clone();
        assert!(seen.iter().any(|u| u.contains("lrclib.net/api/get?track_name=Lagu+Uji&artist_name=Channel+Uji&duration=213")), "{seen:?}");
        assert!(seen.iter().any(|u| u.contains("cdn.example/lagu.mp3")));
    }

    #[test]
    fn tanpa_lirik_mengirim_pesan_kosong() {
        let http = Stub::baru(
            vec![
                ("youtubei/v1/search", Some(serde_json::from_str(HASIL).unwrap())),
                // LRCLIB offline → tanpa lirik.
                ("lrclib.net/api/get", None),
                ("ymcdn", Some(serde_json::json!({"status": false}))),
                (
                    "izuka",
                    Some(serde_json::json!({
                        "status": true,
                        "result": {
                            "title": "Lagu Uji",
                            "format": "mp3",
                            "download_url": "https://cdn.example/lagu.mp3"
                        }
                    })),
                ),
            ],
            Some(b"audio".to_vec()),
        );
        let r = run_http(http, "lagu tanpa lirik");
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus sequence");
        };
        // Tanpa lirik: player HTML tetap dikirim, script-nya menampilkan
        // "Lirik belum tersedia" dari JSON kosong.
        let Reply::TabelAi(rich) = list[2].without_mutations().tanpa_reaksi() else {
            panic!("harus TabelAi");
        };
        let payload = rich.sections[0]["view_model"]["primitive"]["payload"]
            .as_str()
            .expect("payload html");
        assert!(payload.contains("Lirik belum tersedia untuk lagu ini."), "{payload}");
    }

    #[test]
    fn lirik_polos_jadi_sinkron_dengan_interval_durasi() {
        let http = Stub::baru(
            vec![
                ("youtubei/v1/search", Some(serde_json::from_str(HASIL).unwrap())),
                (
                    "lrclib.net/api/get",
                    Some(serde_json::json!({
                        "id": 2,
                        "plainLyrics": "satu\ndua\ntiga",
                        "syncedLyrics": ""
                    })),
                ),
                ("ymcdn", Some(serde_json::json!({"status": false}))),
                (
                    "izuka",
                    Some(serde_json::json!({
                        "status": true,
                        "result": {
                            "download_url": "https://cdn.example/lagu.mp3"
                        }
                    })),
                ),
            ],
            Some(b"audio".to_vec()),
        );
        let r = run_http(http, "lagu polos");
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus sequence");
        };
        // Lirik polos jadi sinkron interval 8 dtk (213/3=71 → clamp 8):
        // JSON base64 di payload memuat time 0/8/16.
        let Reply::TabelAi(rich) = list[2].without_mutations().tanpa_reaksi() else {
            panic!("harus TabelAi");
        };
        let payload = rich.sections[0]["view_model"]["primitive"]["payload"]
            .as_str()
            .expect("payload html");
        let b64 = payload
            .split("atob('")
            .nth(1)
            .and_then(|s| s.split('\'').next())
            .expect("lirik base64 di payload");
        let json: Vec<serde_json::Value> =
            serde_json::from_slice(&decode_base64_test(b64)).unwrap();
        assert_eq!(json[0]["time"], serde_json::json!(0.0));
        assert_eq!(json[0]["text"], serde_json::json!("satu"));
        assert_eq!(json[1]["time"], serde_json::json!(8.0));
        assert_eq!(json[2]["time"], serde_json::json!(16.0));
        assert_eq!(json[2]["text"], serde_json::json!("tiga"));
    }

    #[test]
    fn audio_asli_terlalu_besar_ditolak_sebelum_ffmpeg() {
        // > 25 MB → "Buffer audio bermasalah" (`downloadAudioBuffer`, JS:384).
        let http = stub_dengan_audio(Some(vec![0u8; MAX_ORIGINAL_AUDIO_SIZE + 1]));
        let r = run_http(http, "lagu besar");
        assert_eq!(text_of(&r), gagal("Buffer audio bermasalah"));
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn hasil_kompres_terlalu_besar_ditolak() {
        let http = stub_sukses();
        // Pemalsuan kompresor mengembalikan 7 MB — di atas batas 6 MB.
        let r = jalankan(
            Ctx {
                prefix: ".".into(),
                command: "play2".into(),
                sender: "628111@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: "lagu besar".into(),
                args: vec!["lagu".into(), "besar".into()],
                http,
                ..Default::default()
            },
            |_| Ok(vec![0u8; MAX_COMPRESSED_AUDIO_SIZE + 1]),
        );
        assert_eq!(text_of(&r), gagal("Audio compress terlalu besar"));
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn pencarian_kosong_memberi_pesan_sesuai_jenis_input() {
        let http = Stub::baru(
            vec![("youtubei/v1/search", Some(serde_json::json!({"contents": {}})))],
            None,
        );
        let r = run_http(http, "tidak ada lagunya");
        assert_eq!(
            text_of(&r),
            "❌ Gagal memproses lagu.\n\n> Lagu tidak ditemukan di YT Music"
        );
        assert_eq!(reaction_of(&r), Some("❌"));

        let http = Stub::baru(
            vec![("youtubei/v1/search", Some(serde_json::json!({"contents": {}})))],
            None,
        );
        let r = run_http(http, "https://www.youtube.com/watch?v=abc123XYZ_-");
        assert_eq!(
            text_of(&r),
            "❌ Gagal memproses lagu.\n\n> Video tidak ditemukan"
        );
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn odd_and_very_long_inputs_do_not_panic() {
        for input in ["<b>", "a|b", "🎧", "'\"", "a\u{202e}b", "%00"] {
            assert!(text_of(&run(input)).starts_with("❌ Gagal memproses lagu."), "input {input:?}");
        }
        let panjang = "lagu ".repeat(3000);
        assert!(text_of(&run(&panjang)).contains("Lagu tidak ditemukan"));
    }

    #[test]
    fn a_lrc_line_yields_one_entry_per_timestamp() {
        let lrc = "[00:12.00]Halo\n[00:15.50]Dunia";
        let bars = parse_synced_lyrics(lrc);
        assert_eq!(bars.len(), 2);
        assert!((bars[0].time - 12.0).abs() < 1e-9);
        assert!((bars[1].time - 15.5).abs() < 1e-9);
        assert_eq!(bars[0].text, "Halo");
        assert_eq!(bars[1].text, "Dunia");
    }

    #[test]
    fn two_timestamps_on_one_line_become_two_entries() {
        let bars = parse_synced_lyrics("[00:01.00][00:03.00]Sama");
        assert_eq!(bars.len(), 2);
        assert!((bars[0].time - 1.0).abs() < 1e-9);
        assert!((bars[1].time - 3.0).abs() < 1e-9);
        assert!(bars.iter().all(|b| b.text == "Sama"));
    }

    #[test]
    fn lrc_lines_without_timestamps_or_text_are_skipped() {
        let lrc = "bukan lirik\n\n[00:01.00]\n   \n[00:02.00]Ada\n[zz:yy]Salah";
        let bars = parse_synced_lyrics(lrc);
        assert_eq!(bars.len(), 1);
        assert!((bars[0].time - 2.0).abs() < 1e-9);
        assert_eq!(bars[0].text, "Ada");
    }

    #[test]
    fn entries_come_out_sorted_by_time() {
        let bars = parse_synced_lyrics("[00:30.00]Akhir\n[00:10.00]Awal");
        assert!((bars[0].time - 10.0).abs() < 1e-9);
        assert!((bars[1].time - 30.0).abs() < 1e-9);
    }

    #[test]
    fn identical_lines_less_than_a_millisecond_apart_are_deduplicated() {
        let bars = parse_synced_lyrics("[00:10.000]Sama\n[00:10.0005]Sama");
        assert_eq!(bars.len(), 1);
        let bars = parse_synced_lyrics("[00:10.000]Sama\n[00:10.002]Sama");
        assert_eq!(bars.len(), 2, "selisih >= 1 ms tetap dua baris");
        let bars = parse_synced_lyrics("[00:10.00]Satu\n[00:10.00]Dua");
        assert_eq!(bars.len(), 2, "teks berbeda tidak digabung");
    }

    #[test]
    fn a_seconds_value_of_sixty_or_more_is_rejected() {
        assert!(parse_lrc_timestamp(0.0, 60.0, None).is_none());
        assert!(parse_lrc_timestamp(0.0, 59.0, None).is_some());
        assert!(parse_lrc_timestamp(0.0, -1.0, None).is_none());
        assert!(parse_lrc_timestamp(f64::NAN, 1.0, None).is_none());
        assert!((parse_lrc_timestamp(1.0, 30.0, None).unwrap() - 90.0).abs() < 1e-9);
    }

    #[test]
    fn fraction_length_decides_the_millisecond_scale() {
        let t = |f: Option<&str>| parse_lrc_timestamp(0.0, 1.0, f).unwrap();
        assert!((t(None) - 1.0).abs() < 1e-9);
        assert!((t(Some("")) - 1.0).abs() < 1e-9, "string kosong -> 0");
        assert!((t(Some("5")) - 1.5).abs() < 1e-9, "1 digit -> *100");
        assert!((t(Some("05")) - 1.05).abs() < 1e-9, "2 digit -> *10");
        assert!((t(Some("005")) - 1.005).abs() < 1e-9, "3 digit -> apa adanya");
        assert!((t(Some("1234")) - 1.123).abs() < 1e-9, "digit ke-4 dipotong");
    }

    #[test]
    fn plain_lyrics_get_an_interval_between_two_and_eight_seconds() {
        // 4 baris, durasi 16 detik -> interval 4.
        let bars = plain_lyrics_to_synced("a\nb\nc\nd", 16.0);
        assert_eq!(bars.len(), 4);
        assert_eq!(bars[0].time, 0.0);
        assert_eq!(bars[3].time, 12.0);
        // Durasi 10 detik / 4 baris = 2.5, tetap di dalam rentang.
        let bars = plain_lyrics_to_synced("a\nb\nc\nd", 10.0);
        assert_eq!(bars[3].time, 7.5);
        // Durasi sangat besar -> clamp ke 8.
        let bars = plain_lyrics_to_synced("a\nb\nc\nd", 1000.0);
        assert_eq!(bars[3].time, 24.0);
        // Durasi 0 -> default 5.
        let bars = plain_lyrics_to_synced("a\nb", 0.0);
        assert_eq!(bars[1].time, 5.0);
        // Satu baris saja -> interval default, tapi tidak terlihat.
        assert_eq!(plain_lyrics_to_synced("tunggal", 0.0).len(), 1);
        assert!(plain_lyrics_to_synced("", 10.0).is_empty());
        assert!(plain_lyrics_to_synced("   \n  ", 10.0).is_empty());
    }

    #[test]
    fn normalization_drops_bad_times_and_blank_text() {
        let bars = normalize_lyrics(vec![
            Baris {
                time: f64::NAN,
                text: "bukan waktu".into(),
            },
            Baris {
                time: 5.0,
                text: "   ".into(),
            },
            Baris {
                time: 9.0,
                text: "  spac  ".into(),
            },
            Baris {
                time: 1.0,
                text: "pertama".into(),
            },
        ]);
        assert_eq!(bars.len(), 2);
        assert!((bars[0].time - 1.0).abs() < 1e-9);
        assert_eq!(bars[1].text, "spac");
    }

    #[test]
    fn escaping_replaces_the_ampersand_first() {
        assert_eq!(
            escape_html(r#"<b>&"'</b>"#),
            "&lt;b&gt;&amp;&quot;&#039;&lt;/b&gt;"
        );
        assert_eq!(escape_html("&lt;"), "&amp;lt;", "tidak dua kali escape");
        assert_eq!(escape_html(""), "");
        assert_eq!(escape_attr("<a>"), "&lt;a&gt;");
    }

    #[test]
    fn timestamps_convert_to_seconds_for_two_or_three_parts_only() {
        assert!((seconds_from_timestamp("3:45") - 225.0).abs() < 1e-9);
        assert!((seconds_from_timestamp("1:02:03") - 3723.0).abs() < 1e-9);
        assert_eq!(seconds_from_timestamp(""), 0.0);
        assert_eq!(seconds_from_timestamp("45"), 0.0, "satu bagian -> 0");
        assert_eq!(seconds_from_timestamp("1:2:3:4"), 0.0, "empat bagian -> 0");
        assert_eq!(seconds_from_timestamp("a:b"), 0.0, "bukan angka -> 0");
    }

    #[test]
    fn duration_formatting_pads_seconds_and_drops_a_zero_hour() {
        assert_eq!(format_duration(0.0), "0:00");
        assert_eq!(format_duration(-5.0), "0:00");
        assert_eq!(format_duration(f64::NAN), "0:00");
        assert_eq!(format_duration(9.0), "0:09");
        assert_eq!(format_duration(65.0), "1:05");
        assert_eq!(format_duration(3600.0), "1:00:00");
        assert_eq!(format_duration(3725.0), "1:02:05");
        assert_eq!(format_duration(3661.9), "1:01:01", "detik dip-floor, bukan dibulatkan");
    }

    #[test]
    fn the_youtube_url_check_is_case_insensitive() {
        assert!(looks_like_youtube_url("https://YOUTUBE.COM/watch?v=x"));
        assert!(looks_like_youtube_url("https://youtu.be/x"));
        assert!(!looks_like_youtube_url("another love tom odell"));
        assert!(!looks_like_youtube_url(""));
    }

    #[test]
    fn the_video_id_takes_exactly_eleven_characters() {
        assert_eq!(
            video_id_dari("https://www.youtube.com/watch?v=abc123XYZ_-"),
            Some("abc123XYZ_-".to_string())
        );
        assert_eq!(video_id_dari("https://youtu.be/abcdefghijk"), Some("abcdefghijk".to_string()));
        assert_eq!(
            video_id_dari("https://www.youtube.com/shorts/abc123XYZ_-"),
            Some("abc123XYZ_-".to_string())
        );
        assert_eq!(video_id_dari("pendek"), None);
        assert_eq!(video_id_dari(""), None);
    }

    #[test]
    fn base64_encode_memakai_padding_standar() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"a"), "YQ==");
        assert_eq!(base64_encode(b"ab"), "YWI=");
        assert_eq!(base64_encode(b"abc"), "YWJj");
        assert_eq!(base64_encode(b"ID3data-audio-palsu"), "SUQzZGF0YS1hdWRpby1wYWxzdQ==");
        assert_eq!(decode_base64_test("YWJj"), b"abc".to_vec());
    }

    #[test]
    fn player_html_memakai_judul_yang_diescape_dan_placeholder_poster() {
        let html = player_html("<b>A</b>", "B & C", "3:33", "data:audio/ogg;base64,YQ==", "", &[]);
        assert!(html.contains("&lt;b&gt;A&lt;/b&gt;"), "{html}");
        assert!(html.contains("B &amp; C"), "{html}");
        assert!(html.contains("data:image/svg+xml;base64,"), "placeholder SVG");
        assert!(html.contains("data:audio/ogg;base64,YQ=="), "{html}");
    }

    #[test]
    fn section_html_membawa_typename_aihtml() {
        let s = section_html("<div>x</div>");
        assert_eq!(
            s["view_model"]["primitive"]["__typename"],
            serde_json::json!("GenAIaeacdsnwHtmlPrimitive")
        );
        assert_eq!(s["view_model"]["primitive"]["payload"], serde_json::json!("<div>x</div>"));
        assert_eq!(
            s["view_model"]["primitive"]["trusted_sources"],
            serde_json::json!([])
        );
    }

    #[test]
    fn argumen_ffmpeg_menulis_ke_pipe_stdout() {
        // ffmpeg menolak jalan tanpa berkas keluaran; JS mengakhiri dengan
        // `pipe:1` (`play2.js:419`).
        assert_eq!(ARGUMEN_FFMPEG.last(), Some(&"pipe:1"));
        assert!(ARGUMEN_FFMPEG.contains(&"pipe:0"));
    }

    #[test]
    fn the_javascript_error_messages_are_kept() {
        assert_eq!(gagal("X"), "❌ Gagal memproses lagu.\n\n> X");
        assert_eq!(URL_TIDAK_VALID, "URL YouTube tidak valid");
        assert_eq!(CDN_TIDAK_TERSEDIA, "CDN tidak tersedia");
        assert_eq!(METADATA_KOSONG, "Metadata kosong");
        assert_eq!(DECRYPT_GAGAL, "Decrypt metadata gagal");
        assert_eq!(DOWNLOAD_GAGAL, "Download gagal");
    }
}
