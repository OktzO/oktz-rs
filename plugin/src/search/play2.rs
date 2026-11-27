//! Pemutar lagu dengan LIRIK SINKRON.
//!
//! PENYIMPANGAN (satu baris per butir):
//! - `ytmusic-api` diganti [`super::ytmusic::search_lagu`] (InnerTube
//! WEB_REMIX filter songs) untuk jalur judul; jalur URL tetap
//! `ytsearch::search`. Pesan "Lagu tidak ditemukan di YT Music"
//! dipertahankan untuk jalur judul.
//! - Savetube/ytmp3.ing tidak ditambahkan di sini: unduhan audio memakai
//! [`oktz_scraper::playytmp3::savetube_download`] →
//! [`oktz_scraper::playytmp3::ytmp3ing_download`] (jalur sama
//! `.play`), lalu fallback `crate::download::ytmp3::unduh_audio`
//! (`ytdl-native → savetube → izuka → nexray`).
//! - Kartu player HTML dikirim lewat `RichAi` (`Reply::TabelAi`)
//! section `GenAIaeacdsnwHtmlPrimitive` — sama seperti `sendMusicPlayer`
//! , submessage teksnya `"${title} - ${artist}"`.
//! Amplop TIDAK lagi ikut versi awal: botJid `0@bot` + score 1 + edit susulan
//! identik (adendum 8 — terbukti render di Android 2.26.45.99 lewat
//! `.cekkartu` K/L/M; amplop versi awal `867051314767696@bot`/999 hanya bukti
//! versi WA lama).
//! Lirik sinkron hidup di dalam player browser (JSON base64 + highlight
//! `rAF`), bukan WS. Audio PENUH tertanam di kartu (opus 16k,
//! langit-langit 768 KB terukur di klien owner — adendum 11) dan diputar
//! lewat **Web Audio** `decodeAudioData`, bukan `<audio src>`: kartu
//! dirender dengan origin opaque sehingga Blink menolak semua URL media
//! (adendum 13). Lagu yang lebih panjang dipangkas durasinya khusus
//! kartu. Voice note opus (ffmpeg `libopus` 16k/24kHz/mono/ogg) tetap
//! dikirim sebagai fallback untuk klien non-Android yang tidak merender
//! primitif HTML.
//! - Fallback lirik `ytm.getLyrics` (ytmusic-api) dilewati (`Keterbatasan:`;
//! LRCLIB cukup; upgrade path: tambah InnerTube timedtext bila perlu).

#![allow(clippy::collapsible_if)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use oktz_plugin_api::{AiSubMessage, Ctx, HttpCall, MediaSource, Reply, RichAi, plugin};
use serde_json::json;

use crate::download::douyindl::query_encode;
use crate::download::ytmp3::{mimetype_for, unduh_audio};

use super::{ytmusic, ytsearch};

/// Kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{prefix}play2 <judul lagu / URL YouTube>`\n\nKartu player interaktif dengan lagu full + lirik sinkron.\n\nContoh: `{prefix}play2 another love tom odell`"
    )
}

/// Pesan `catch`.
pub(crate) fn gagal(message: &str) -> String {
    format!("❌ Gagal memproses lagu.\n\n> {message}")
}

/// Pesan yang dilempar saat pencarian tidak menghasilkan
/// entri untuk input judul.
pub(crate) const LAGU_TIDAK_DITEMUKAN: &str = "Lagu tidak ditemukan di YT Music";

/// `yts` tidak menemukan video untuk URL yang diberikan.
pub(crate) const VIDEO_TIDAK_DITEMUKAN: &str = "Video tidak ditemukan";

/// Regex ID video; pesan untuk ID yang tidak ketemu.
#[cfg(test)]
pub(crate) const URL_TIDAK_VALID: &str = "URL YouTube tidak valid";

/// / `:308` / `:327` / `:345` — pesan savetube; rantai resolver
/// pengganti hanya memakai `Download gagal`, sisanya disimpan sebagai teks versi awal.
#[allow(dead_code)]
pub(crate) const CDN_TIDAK_TERSEDIA: &str = "CDN tidak tersedia";
#[allow(dead_code)]
pub(crate) const METADATA_KOSONG: &str = "Metadata kosong";
#[allow(dead_code)]
pub(crate) const DECRYPT_GAGAL: &str = "Decrypt metadata gagal";
pub(crate) const DOWNLOAD_GAGAL: &str = "Download gagal";

/* Konfigurasi. */

const LRCLIB_API: &str = "https://lrclib.net/api";
const LRCLIB_USER_AGENT: &str = "WA-Bot/1.0 (https://github.com/)";

/// `User-Agent` savetube — ikut dipakai `downloadAudioBuffer`.
const USER_AGENT: &str =
    "Mozilla/5.0 (Linux; Android 15) AppleWebKit/537.36 Chrome/130 Mobile Safari/537.36";

const MAX_ORIGINAL_AUDIO_MB: usize = 25;
const MAX_ORIGINAL_AUDIO_SIZE: usize = MAX_ORIGINAL_AUDIO_MB * 1024 * 1024;

const MAX_COMPRESSED_AUDIO_MB: usize = 6;
const MAX_COMPRESSED_AUDIO_SIZE: usize = MAX_COMPRESSED_AUDIO_MB * 1024 * 1024;

/* Lirik. */

/// `parseLrcTimestamp`. Menolak `seconds >= 60`.
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

/// `parseSyncedLyrics`.
///
/// Baris tanpa timestamp atau tanpa teks dilewati; hasil diurutkan naik
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
                // Kurung siku bukan timestamp: ikut dibuang oleh regex di versi awal.
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

/// `plainLyricsToSynced` — jarak antar baris 2..8 detik.
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

/// `normalizeLyrics` — buang waktu bukan angka dan teks
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

/// `escapeHtml` — urutan penggantian penting: `&` dulu.
/// Hanya dipakai kartu HTML versi awal yang tidak diport; disimpan untuk perilaku teks.
#[allow(dead_code)]
pub(crate) fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}

/// `escapeAttr` = `escapeHtml`.
pub(crate) fn escape_attr(text: &str) -> String {
    escape_html(text)
}

/// Encoder base64 standar (dengan padding) — crate `base64` tidak ada
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

/// `getThumb`: `sharp.resize(250,250,{fit:'cover'})
/// .jpeg({quality:50})` → `data:image/jpeg;base64...` via
/// `oktz_plugin_api::gambar::sharp` (tanpa dependensi baru). Gagal/unduhan
/// di atas 100 KB/gambar tak dikenali → `""` (HTML memakai placeholder SVG,
///
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
    match oktz_plugin_api::gambar::sharp(data)
        .and_then(|g| g.ubah_ukuran(250, 250).ke_jpeg(50))
    {
        Ok(jpeg) => format!("data:image/jpeg;base64,{}", base64_encode(&jpeg)),
        Err(_) => String::new(),
    }
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
/// `Keterbatasan:` tanpa TTL; upgrade path: simpan waktu + evict per entri.
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

/// `secondsFromTimestamp` — 2 atau 3 bagian; selain itu 0.
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

/// `formatDuration` — jam muncul hanya di atas nol.
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

/// `/youtube\.com|youtu\.be/i.test(text)`.
pub(crate) fn looks_like_youtube_url(text: &str) -> bool {
    let low = text.to_lowercase();
    low.contains("youtube.com") || low.contains("youtu.be")
}

/// `(?:v=|shorts\/|youtu\.be\/|embed\/)([a-zA-Z0-9_-]{11})`
/// Kejadian pertama di URL, lalu tepat 11 karakter.
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

/* LRCLIB. */

/// Bentuk jawaban `getLRCLyrics` yang dipakai plugin.
pub(crate) struct LirikLagu {
    pub plain: String,
    pub synced: String,
}

/// `getLRCLyrics` — `None` = semua kegagalan (transport,
/// status tidak ok, body rusak) di versi awal ditelan `catch`/pemeriksaan status.
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

/* Audio, unduhan lewat rantai `ytmp3::unduh_audio`. */

/// Ringkas pesan galat tahap unduhan supaya muat di satu baris balasan.
fn ringkas(pesan: &str) -> String {
    let bersih = pesan.replace('\n', " ");
    if bersih.chars().count() > 120 {
        format!("{}…", bersih.chars().take(120).collect::<String>())
    } else {
        bersih
    }
}

/// `downloadAudioBuffer`.
fn unduh_buffer(ctx: &Ctx, url: &str) -> Result<Vec<u8>, String> {
    if url.is_empty() {
        return Err("URL audio kosong".to_string());
    }
    // Berkas audio bisa >3 MB; default `UreqHttp` 20 detik terlalu ketat
    // panel lambat (gagal → fallback rantai unduh yang juga lambat).
    let res = ctx
        .http
        .call(
            &HttpCall::get(url)
                .header("user-agent", USER_AGENT)
                .with_timeout(60_000),
        )
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

/// Argumen `compressAudio` — opus 16k/24kHz/mono/ogg.
/// `pipe:1` WAJIB: ffmpeg menolak jalan tanpa berkas keluaran
/// (`At least one output file must be specified`), dan versi awal selalu
/// mengirimkannya.
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

/// `compressAudio` — stdin `pipe:0`, stdout `pipe:1`.
/// Batas 6 MB keluaran diperiksa di [`proses`] supaya bisa diuji tanpa
/// ffmpeg sungguhan.
fn kompres_audio(input: &[u8]) -> Result<Vec<u8>, String> {
    kompres_dengan(input, &ARGUMEN_FFMPEG)
}

/// Seperti [`kompres_audio`] tapi dibatasi `detik` pertama (`-t`) — dipakai
/// saat lagu penuh membuat kartu melewati [`MAX_HTML_PAYLOAD_BYTES`].
fn kompres_audio_durasi(input: &[u8], detik: u32) -> Result<Vec<u8>, String> {
    let detik = detik.to_string();
    let mut argumen: Vec<&str> = ARGUMEN_FFMPEG[..ARGUMEN_FFMPEG.len() - 1].to_vec();
    argumen.extend(["-t", &detik, "pipe:1"]);
    kompres_dengan(input, &argumen)
}

fn kompres_dengan(input: &[u8], argumen: &[&str]) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Err("Input buffer kosong".to_string());
    }
    if oktz_plugin_api::ffmpeg::jalur_ffmpeg().is_none() {
        return Err("FFmpeg tidak ditemukan.".to_string());
    }
    let output = oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(argumen, input)
        .map_err(|_| "FFmpeg gagal".to_string())?;
    if output.is_empty() {
        return Err("FFmpeg menghasilkan audio kosong".to_string());
    }
    Ok(output)
}

/// Tipe fungsi kompresi — parameternya dibuka supaya test bisa memalsukan
/// ffmpeg tanpa menyentuh env global.
type Kompresor = fn(&[u8]) -> Result<Vec<u8>, String>;

/// Poster bawaan kalau thumbnail kosong/gagal — sama dengan `createMusicPlayer`
///
pub(crate) const POSTER_PLACEHOLDER: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0MDAiIGhlaWdodD0iNDAwIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjQwMCIgZmlsbD0iIzFhMGQxMiIvPjx0ZXh0IHg9IjIwMCIgeT0iMjEwIiBmb250LXNpemU9IjM0IiBmb250LWZhbWlseT0ic2Fucy1zZXJpZiIgZmlsbD0iI2ZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSI+TUFJTiBQQ0xAYVlFUlI8L3RleHQ+PC9zdmc+";

/// Batas muatan HTML primitif. TERUKUR di perangkat owner (adendum 11):
/// kartu Y 768 KB render di Android 2.26.45.99 — angka komunitas yang lebih
/// kecil (Vanta "native limit is 60000", kotonehara 64 KiB) tidak berlaku
/// build ini; zaileys malah menetapkan 256 KiB. 768_000 = tepi zona terbukti.
pub(crate) const MAX_HTML_PAYLOAD_BYTES: usize = 768_000;

/// `createMusicPlayer` versi Rust — port PENUH: bg
/// poster blur + veil, header, heart, mini-lirik sinkron, progress bar,
/// controls ±10s + play/pause, audio tertanam. Template verbatim versi awal
/// token `__…__` (bukan `format!` — kurung kurawal CSS/versi awal tidak perlu
/// di-escape, dan template bisa di-diff langsung terhadap oracle).
///
/// `audio_b64` = audio opus (hasil [`kompres_audio`]) dalam base64; kartu
/// memuatnya lewat Web Audio `decodeAudioData`, BUKAN `<audio src>`. Kartu
/// dirender WhatsApp dengan origin OPAQUE,
/// `HTMLMediaElement::IsSafeToLoadURL` menolak SEMUA URL media pada origin
/// opaque — `data:`, blob in-document, semuanya, "Media load rejected by URL
/// safety check" (terukur di perangkat owner, adendum 13). Web Audio tidak
/// melewati pemeriksaan URL itu, jadi dipakai sebagai mesin pemutar; UI tetap
/// `createMusicPlayer`.
pub(crate) fn player_html(
    title: &str,
    artist: &str,
    durasi: &str,
    poster: &str,
    lyrics: &[Baris],
    audio_b64: &str,
) -> String {
    let safe_title = escape_html(title);
    let safe_artist = escape_html(artist);
    let safe_dur = escape_html(durasi);
    let poster = if poster.is_empty() {
        POSTER_PLACEHOLDER
    } else {
        poster
    };
    let lirik_json: Vec<serde_json::Value> = lyrics
        .iter()
        .map(|b| json!({"time": b.time, "text": b.text}))
        .collect();
    let lirik_b64 = base64_encode(serde_json::to_string(&lirik_json).unwrap_or_default().as_bytes());
    PLAYER_TEMPLATE
        .replace("__POSTER__", &escape_attr(poster))
        .replace("__TITLE_ATTR__", &escape_attr(&safe_title))
        .replace("__TITLE__", &safe_title)
        .replace("__ARTIST__", &safe_artist)
        .replace("__DUR__", &safe_dur)
        .replace("__AUDIO_B64__", audio_b64)
        .replace("__LYRICS_B64__", &lirik_b64)
}

/// `player_html` dengan poster dilepas kalau muatannya melewati
/// [`MAX_HTML_PAYLOAD_BYTES`]. Jaring pengaman pertama; kalau masih
/// kelebihan, [`proses`] memangkas durasi audio sebelum membangun ulang.
fn player_html_aman(
    title: &str,
    artist: &str,
    durasi: &str,
    poster: &str,
    lyrics: &[Baris],
    audio_b64: &str,
) -> String {
    let html = player_html(title, artist, durasi, poster, lyrics, audio_b64);
    if html.len() <= MAX_HTML_PAYLOAD_BYTES {
        html
    } else {
        player_html(title, artist, durasi, "", lyrics, audio_b64)
    }
}

/// Template verbatim `createMusicPlayer`; token
/// diganti [`player_html`]. Note bawah disesuaikan: lagu penuh juga
/// dikirim sebagai voice note (deviasi Rust yang dipertahankan).
const PLAYER_TEMPLATE: &str = r#"<style>
  :root {
    --ink: #ffffff;
    --muted: #b9b1b6;
    --sys: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; -webkit-tap-highlight-color: transparent; }
  html, body { background: transparent; color: var(--ink); font-family: var(--sys); -webkit-font-smoothing: antialiased; }
  .wrap { display: flex; align-items: center; justify-content: center; padding: 10px; }
  .player { position: relative; width: 100%; max-width: 330px; border-radius: 18px; overflow: hidden; background: #1a0d12; box-shadow: 0 18px 40px rgba(0,0,0,.5); }
  .bg { position: absolute; inset: -30%; width: 160%; height: 160%; object-fit: cover; filter: blur(38px) saturate(1.5); opacity: .85; z-index: 0; }
  .veil { position: absolute; inset: 0; z-index: 1; background: linear-gradient(180deg, rgba(20,8,12,.65) 0%, rgba(20,8,12,.8) 45%, rgba(12,5,8,.96) 100%); }
  .content { position: relative; z-index: 2; padding: 16px 18px 18px; }
  .head { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 14px; }
  .head__icon { width: 18px; height: 18px; color: var(--ink); opacity: .85; flex: none; }
  .head__mid { text-align: center; flex: 1; min-width: 0; }
  .head__from { font-size: 9px; letter-spacing: .14em; text-transform: uppercase; color: var(--muted); }
  .head__album { font-size: 12px; font-weight: 600; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .poster { width: 100%; aspect-ratio: 1; border-radius: 10px; overflow: hidden; background: rgba(255,255,255,.06); box-shadow: 0 12px 26px rgba(0,0,0,.45); margin-bottom: 14px; }
  .poster img { width: 100%; height: 100%; object-fit: cover; display: block; }
  .info { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; margin-bottom: 10px; }
  .info__names { min-width: 0; }
  .info__title { font-size: 17px; font-weight: 600; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .info__artist { font-size: 12px; color: var(--muted); margin-top: 3px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .info__heart { width: 34px; height: 34px; flex: none; display: flex; align-items: center; justify-content: center; background: none; border: none; color: var(--muted); cursor: pointer; padding: 0; }
  .info__heart.is-on { color: #ff5c8a; }
  .mini-lyrics { position: relative; height: 82px; overflow-x: hidden; overflow-y: auto; scrollbar-width: none; -ms-overflow-style: none; margin-bottom: 10px; overscroll-behavior: contain; scroll-behavior: auto; mask-image: linear-gradient(180deg, transparent 0%, black 18%, black 82%, transparent 100%); -webkit-mask-image: linear-gradient(180deg, transparent 0%, black 18%, black 82%, transparent 100%); }
  .mini-lyrics::-webkit-scrollbar { display: none; }
  .lyrics-text { width: 100%; padding: 32px 0; display: flex; flex-direction: column; gap: 5px; }
  .lyric-line { font-size: 10.5px; color: rgba(255,255,255,.4); line-height: 1.4; transition: color .25s ease, font-size .25s ease, opacity .25s ease, transform .25s ease; text-align: left; white-space: normal; word-wrap: break-word; overflow-wrap: anywhere; font-weight: 500; opacity: .75; transform: translateX(0) scale(1); transform-origin: left center; }
  .lyric-line.is-active { font-size: 12px; color: #fff; font-weight: 700; opacity: 1; transform: translateX(2px) scale(1.01); }
  .lyrics-empty { font-size: 10.5px; color: var(--muted); text-align: left; padding: 16px 0; }
  .bar { position: relative; height: 4px; border-radius: 4px; background: rgba(255,255,255,.22); cursor: pointer; margin-bottom: 6px; touch-action: none; }
  .bar__fill { position: absolute; left: 0; top: 0; bottom: 0; width: 0; border-radius: 4px; background: #fff; pointer-events: none; }
  .bar__dot { position: absolute; top: 50%; left: 0; width: 11px; height: 11px; border-radius: 50%; background: #fff; transform: translate(-50%,-50%); pointer-events: none; }
  .time { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); margin-bottom: 12px; font-variant-numeric: tabular-nums; }
  .controls { display: flex; align-items: center; justify-content: space-between; }
  .ctrl { width: 34px; height: 34px; display: flex; align-items: center; justify-content: center; color: var(--ink); background: none; border: none; cursor: pointer; padding: 0; transition: opacity .2s ease, transform .15s ease; }
  .ctrl:active { opacity: .6; transform: scale(.92); }
  .play { width: 52px; height: 52px; border-radius: 50%; background: #fff; color: #12070b; border: none; cursor: pointer; display: flex; align-items: center; justify-content: center; flex: none; padding: 0; box-shadow: 0 6px 16px rgba(0,0,0,.4); transition: transform .15s ease; }
  .play:active { transform: scale(.93); }
  .note { margin-top: 12px; text-align: center; font-size: 10px; color: var(--muted); line-height: 1.6; }
</style>

<div class="wrap">
  <div class="player">
    <img class="bg" src="__POSTER__" alt="">
    <div class="veil"></div>
    <div class="content">
      <div class="head">
        <svg class="head__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
        <div class="head__mid">
          <div class="head__from">YT Music Audio</div>
          <div class="head__album">__ARTIST__</div>
        </div>
        <svg class="head__icon" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="1.8"/><circle cx="12" cy="12" r="1.8"/><circle cx="12" cy="19" r="1.8"/></svg>
      </div>

      <div class="poster">
        <img src="__POSTER__" alt="__TITLE_ATTR__">
      </div>

      <div class="info">
        <div class="info__names">
          <div class="info__title">__TITLE__</div>
          <div class="info__artist">__ARTIST__</div>
        </div>
        <button class="info__heart" id="heart" aria-label="Favorite">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="19" height="19"><path d="M20.8 5.6 a5.1 5.1 0 0 0-7.2 0 L12 7.2 l-1.6-1.6 a5.1 5.1 0 0 0-7.2 7.2 l1.6 1.6 L12 21.6 l7.2-7.2 1.6-1.6 a5.1 5.1 0 0 0 0-7.2z"/></svg>
        </button>
      </div>

      <div class="mini-lyrics" id="mini-lyrics">
        <div class="lyrics-text" id="lyrics-text"></div>
      </div>

      <div class="bar" id="bar">
        <div class="bar__fill" id="fill"></div>
        <div class="bar__dot" id="dot"></div>
      </div>

      <div class="time">
        <span id="cur">0:00</span>
        <span id="dur">__DUR__</span>
      </div>

      <div class="controls">
        <button class="ctrl" style="opacity:.5; cursor:default;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="20" height="20"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        </button>

        <button class="ctrl" id="btn-rw" aria-label="Mundur 10 detik">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="22" height="22"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><text x="12" y="16.5" font-size="7.5" font-family="sans-serif" font-weight="bold" stroke="none" fill="currentColor" text-anchor="middle">10</text></svg>
        </button>

        <button class="play" id="play" aria-label="Play">
          <svg id="icon-play" viewBox="0 0 24 24" fill="currentColor" width="26" height="26"><path d="M8 5.6 v12.8 a.6.6 0 0 0 .92.5 l10-6.4 a.6.6 0 0 0 0-1 l-10-6.4 a.6.6 0 0 0-.92.5z"/></svg>
          <svg id="icon-pause" viewBox="0 0 24 24" fill="currentColor" width="26" height="26" style="display:none"><rect x="6.5" y="5" width="3.8" height="14" rx="1"/><rect x="13.7" y="5" width="3.8" height="14" rx="1"/></svg>
        </button>

        <button class="ctrl" id="btn-fw" aria-label="Maju 10 detik">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="22" height="22"><path d="M21 12a9 9 0 1 1-9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><text x="12" y="16.5" font-size="7.5" font-family="sans-serif" font-weight="bold" stroke="none" fill="currentColor" text-anchor="middle">10</text></svg>
        </button>

        <button class="ctrl" style="opacity:.5; cursor:default;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="20" height="20"><path d="m17 2 4 4-4 4"/><path d="M3 11v-1 a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1 a4 4 0 0 1-4 4H3"/></svg>
        </button>
      </div>

      <div class="note">🎧 Lagu penuh juga dikirim sebagai pesan suara di atas kartu ini.</div>
    </div>
  </div>
</div>

<script>
(function() {
  'use strict';

  var AUDIO_B64 = '__AUDIO_B64__';
  const play = document.getElementById('play');
  const bar = document.getElementById('bar');
  const fill = document.getElementById('fill');
  const dot = document.getElementById('dot');
  const cur = document.getElementById('cur');
  const dur = document.getElementById('dur');
  const heart = document.getElementById('heart');
  const iconPlay = document.getElementById('icon-play');
  const iconPause = document.getElementById('icon-pause');
  const lyricsContainer = document.getElementById('mini-lyrics');
  const lyricsText = document.getElementById('lyrics-text');
  const btnRw = document.getElementById('btn-rw');
  const btnFw = document.getElementById('btn-fw');

  let lyrics = [];
  try {
    const encoded = '__LYRICS_B64__';
    const binary = atob(encoded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const json = new TextDecoder('utf-8').decode(bytes);
    const parsed = JSON.parse(json);
    if (Array.isArray(parsed)) {
      lyrics = parsed
        .filter(item => item && Number.isFinite(Number(item.time)))
        .map(item => ({ time: Number(item.time), text: String(item.text || '♪') }))
        .sort((a, b) => a.time - b.time);
    }
  } catch {
    lyrics = [];
  }

  const lyricElements = [];
  function renderLyrics() {
    lyricsText.innerHTML = '';
    lyricElements.length = 0;

    if (!Array.isArray(lyrics) || !lyrics.length) {
      lyricsText.innerHTML = '<div class="lyrics-empty">Lirik belum tersedia untuk lagu ini.</div>';
      return;
    }

    const fragment = document.createDocumentFragment();
    lyrics.forEach((line, index) => {
      const el = document.createElement('div');
      el.className = 'lyric-line';
      el.dataset.time = String(line.time);
      el.dataset.index = String(index);
      el.textContent = line.text || '♪';
      fragment.appendChild(el);
      lyricElements.push(el);
    });
    lyricsText.appendChild(fragment);
  }
  renderLyrics();

  let activeLyricIndex = -1;
  function findLyricIndex(time) {
    if (!lyrics.length) return -1;
    let low = 0, high = lyrics.length - 1, result = -1;
    while (low <= high) {
      const mid = Math.floor((low + high) / 2);
      if (Number(lyrics[mid].time) <= time) {
        result = mid;
        low = mid + 1;
      } else {
        high = mid - 1;
      }
    }
    return result;
  }

  let targetScrollTop = 0, currentScrollTop = 0, scrollAnimationFrame = null;
  const SCROLL_SMOOTHNESS = 0.18;

  function calculateLyricScroll(element) {
    if (!element || !lyricsContainer) return 0;
    const elementTop = element.offsetTop;
    const elementHeight = element.offsetHeight;
    const containerHeight = lyricsContainer.clientHeight;
    let target = elementTop - (containerHeight / 2) + (elementHeight / 2);
    const maxScroll = Math.max(0, lyricsContainer.scrollHeight - containerHeight);
    return Math.max(0, Math.min(target, maxScroll));
  }

  function setLyricScrollTarget(element, immediate = false) {
    if (!element || !lyricsContainer) return;
    targetScrollTop = calculateLyricScroll(element);
    if (immediate) {
      currentScrollTop = targetScrollTop;
      lyricsContainer.scrollTop = targetScrollTop;
      return;
    }
    startScrollAnimation();
  }

  function startScrollAnimation() {
    if (scrollAnimationFrame !== null) return;
    function animate() {
      const difference = targetScrollTop - currentScrollTop;
      if (Math.abs(difference) < 0.5) {
        currentScrollTop = targetScrollTop;
        lyricsContainer.scrollTop = currentScrollTop;
        scrollAnimationFrame = null;
        return;
      }
      currentScrollTop += difference * SCROLL_SMOOTHNESS;
      lyricsContainer.scrollTop = currentScrollTop;
      scrollAnimationFrame = requestAnimationFrame(animate);
    }
    scrollAnimationFrame = requestAnimationFrame(animate);
  }

  function updateLyrics(currentTime, { immediate = false } = {}) {
    if (!lyrics.length || !lyricElements.length) return;
    const time = Number(currentTime) || 0;
    const index = findLyricIndex(time);

    if (index < 0) {
      if (activeLyricIndex !== -1) {
        activeLyricIndex = -1;
        lyricElements.forEach(el => el.classList.remove('is-active'));
      }
      return;
    }

    if (index === activeLyricIndex && !immediate) return;

    activeLyricIndex = index;
    lyricElements.forEach((el, i) => el.classList.toggle('is-active', i === index));
    const active = lyricElements[index];
    if (active) setLyricScrollTarget(active, immediate);
  }

  function resetLyrics() {
    activeLyricIndex = -1;
    lyricElements.forEach(el => el.classList.remove('is-active'));
    targetScrollTop = 0;
    currentScrollTop = 0;
    if (lyricsContainer) lyricsContainer.scrollTop = 0;
  }

  function formatTime(sec) {
    sec = Number(sec);
    if (!Number.isFinite(sec) || sec < 0) return '0:00';
    const h = Math.floor(sec / 3600);
    const m = Math.floor(sec % 3600 / 60);
    const s = Math.floor(sec % 60);
    if (h > 0) return h + ':' + String(m).padStart(2, '0') + ':' + String(s).padStart(2, '0');
    return m + ':' + String(s).padStart(2, '0');
  }

  var ctx = null, buf = null, node = null;
  var playing = false, off = 0, at = 0, loading = false, raf = null;

  function AudioCtx() { return window.AudioContext || window.webkitAudioContext; }

  function siapkanCtx() {
    if (ctx) return ctx;
    var AC = AudioCtx();
    if (!AC) return null;
    try { ctx = new AC({ sampleRate: 24000, latencyHint: 'playback' }); }
    catch (e) { try { ctx = new AC(); } catch (e2) { return null; } }
    return ctx;
  }

  function durasiAudio() { return buf ? buf.duration : 0; }

  function waktuSekarang() {
    if (!buf) return 0;
    if (!playing) return off;
    return Math.min(off + (ctx.currentTime - at), buf.duration);
  }

  function lepasNode() {
    if (!node) return;
    try { node.stop(); } catch (e) {}
    try { node.disconnect(); } catch (e) {}
    node = null;
  }

  function mulaiNode() {
    lepasNode();
    if (!buf || !ctx) return;
    node = ctx.createBufferSource();
    node.buffer = buf;
    node.connect(ctx.destination);
    at = ctx.currentTime;
    node.start(0, Math.min(off, Math.max(0, buf.duration - 0.001)));
    playing = true;
  }

  function render() {
    var total = durasiAudio();
    if (total <= 0) return;
    var t = waktuSekarang();
    var percent = Math.min(100, (t / total) * 100);
    fill.style.width = percent + '%';
    dot.style.left = percent + '%';
    cur.textContent = formatTime(t);
    dur.textContent = formatTime(total);
    updateLyrics(t);
  }

  function loop() {
    if (!playing) { raf = null; return; }
    if (waktuSekarang() >= durasiAudio() - 0.02) {
      playing = false;
      off = 0;
      lepasNode();
      raf = null;
      fill.style.width = '0%';
      dot.style.left = '0%';
      cur.textContent = '0:00';
      resetLyrics();
      setPaused();
      return;
    }
    render();
    raf = requestAnimationFrame(loop);
  }

  function muat(saatSiap) {
    if (buf || loading) return;
    var c = siapkanCtx();
    if (!c) { dur.textContent = 'audio tidak didukung'; return; }
    loading = true;
    var bin, u;
    try {
      bin = atob(AUDIO_B64);
      u = new Uint8Array(bin.length);
      for (var i = 0; i < bin.length; i++) u[i] = bin.charCodeAt(i);
    } catch (e) { loading = false; dur.textContent = 'audio rusak'; return; }
    var selesai = function (b) {
      if (buf) return;
      buf = b;
      loading = false;
      dur.textContent = formatTime(b.duration);
      render();
      if (saatSiap) saatSiap();
    };
    var gagal = function () { loading = false; dur.textContent = 'audio gagal'; };
    try {
      var p = c.decodeAudioData(u.buffer, selesai, gagal);
      if (p && p.then) p.then(function () {}, function () { if (!buf) gagal(); });
    } catch (e) { gagal(); }
  }

  function putar() {
    if (!buf) return;
    if (off >= durasiAudio() - 0.02) off = 0;
    if (ctx.state === 'suspended' && ctx.resume) {
      var pr = ctx.resume();
      if (pr && pr.then) { pr.then(function () { mulaiNode(); loop(); }); return; }
    }
    mulaiNode();
    loop();
  }

  function jeda() {
    off = waktuSekarang();
    lepasNode();
    playing = false;
    render();
  }

  function lompat(t) {
    if (!buf) return;
    off = Math.max(0, Math.min(durasiAudio() - 0.01, Number(t) || 0));
    if (ctx.state === 'suspended' && ctx.resume) ctx.resume();
    if (playing) mulaiNode(); else render();
    updateLyrics(off, { immediate: true });
    render();
  }

  function setPlaying() {
    iconPlay.style.display = 'none';
    iconPause.style.display = 'block';
  }

  function setPaused() {
    iconPlay.style.display = 'block';
    iconPause.style.display = 'none';
  }

  play.addEventListener('click', function () {
    siapkanCtx();
    if (!ctx) { dur.textContent = 'audio tidak didukung'; return; }
    if (ctx.state === 'suspended' && ctx.resume) ctx.resume();
    if (!buf) {
      if (loading) return;
      muat(function () { putar(); setPlaying(); });
      return;
    }
    if (playing) { jeda(); setPaused(); }
    else { putar(); setPlaying(); }
  });

  heart.addEventListener('click', function () { heart.classList.toggle('is-on'); });

  btnRw.addEventListener('click', function () { lompat(waktuSekarang() - 10); });

  btnFw.addEventListener('click', function () { lompat(waktuSekarang() + 10); });

  bar.addEventListener('pointerdown', function (event) {
    if (!buf) return;
    var rect = bar.getBoundingClientRect();
    if (!rect.width) return;
    var x = Math.max(0, Math.min(event.clientX - rect.left, rect.width));
    lompat((x / rect.width) * durasiAudio());
  });

  setPaused();
})();
</script>
"#;

/// Section HTML `GenAIaeacdsnwHtmlPrimitive` — `sendMusicPlayer`
///
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

/* Handler. */

/// Alur `try`. `Err` memuat `error.message`
/// template `catch`.
fn proses(ctx: &Ctx, text: &str, kompres: Kompresor) -> Result<Reply, String> {
    // versi awal: judul → `ytmusic.search`, URL → `yts(ytUrl)`; di sini judul →
    // `ytmusic::search_lagu`, URL → `ytsearch::search` (lihat deviasi header).
    let adalah_url = looks_like_youtube_url(text);
    let pesan_kosong = if adalah_url {
        VIDEO_TIDAK_DITEMUKAN
    } else {
        LAGU_TIDAK_DITEMUKAN
    };
    let videos = if adalah_url {
        cari_video(ctx.http.as_ref(), text).map_err(|_| pesan_kosong.to_string())?
    } else {
        ytmusic::search_lagu(ctx.http.as_ref(), text).map_err(|_| pesan_kosong.to_string())?
    };
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

    // Unduhan audio: jalur `.play` dulu (`playytmp3`: savetube → ytmp3.ing),
    // lalu fallback rantai `unduh_audio`. Alasan tiap tahap dikumpulkan —
    // "Download gagal" polos tidak memberi tahu tahap mana yang mati.
    let mut sebab: Vec<String> = Vec::new();
    let ambil_url = |v: &serde_json::Value| -> Option<String> {
        v.get("downloadUrl")
            .and_then(serde_json::Value::as_str)
            .filter(|u| !u.is_empty())
            .map(str::to_string)
    };
    let mut audio_utama = match oktz_scraper::playytmp3::savetube_download(
        ctx.http.as_ref(),
        &video.url,
        "128",
    ) {
        Ok(v) => {
            let u = ambil_url(&v);
            if u.is_none() {
                sebab.push("savetube: downloadUrl kosong".to_string());
            }
            u
        }
        Err(e) => {
            sebab.push(format!("savetube: {}", ringkas(&e.message)));
            None
        }
    };
    if audio_utama.is_none() {
        match oktz_scraper::playytmp3::ytmp3ing_download(ctx.http.as_ref(), &video.url) {
            Ok(v) => {
                audio_utama = ambil_url(&v);
                if audio_utama.is_none() {
                    sebab.push("ytmp3.ing: downloadUrl kosong".to_string());
                }
            }
            Err(e) => sebab.push(format!("ytmp3.ing: {}", ringkas(&e.message))),
        }
    }
    let asli = match audio_utama.and_then(|u| match unduh_buffer(ctx, &u) {
        Ok(b) => Some(b),
        Err(e) => {
            sebab.push(format!("unduh buffer: {e}"));
            None
        }
    }) {
        Some(b) => b,
        None => {
            let audio = match unduh_audio(ctx, &video.url) {
                Ok(a) => a,
                Err(()) => {
                    return Err(format!("{DOWNLOAD_GAGAL} [{}]", sebab.join("; ")));
                }
            };
            unduh_buffer(ctx, &audio.url)?
        }
    };
    let kompresi = kompres(&asli)?;
    if kompresi.len() > MAX_COMPRESSED_AUDIO_SIZE {
        return Err("Audio compress terlalu besar".to_string());
    }

    // `sendMusicPlayer(sock, m.chat, html, "${title} - ${artist}")`
    // : kartu memuat poster, lirik, dan AUDIO PENUH
    // tertanam (adendum 11 — langit-langit 768 KB terukur di klien owner).
    // Voice note tetap dikirim sebagai fallback klien non-Android.
    let poster = poster_base64(ctx.http.as_ref(), &video.thumbnail);
    let durasi = format_duration(duration_sec);
    let audio_b64 = base64_encode(&kompresi);
    let mut html = player_html_aman(title, artist, &durasi, &poster, &synced, &audio_b64);
    if html.len() > MAX_HTML_PAYLOAD_BYTES {
        // Lagu terlalu panjang untuk zona terbukti: pangkas durasi audio
        // kartu (16 kbps ≈ 2 KB/detik + overhead ogg), bukan mematikan
        // player. Voice note tetap lagu penuh.
        let overhead = player_html(title, artist, &durasi, "", &synced, "").len();
        let budget_mentah = MAX_HTML_PAYLOAD_BYTES.saturating_sub(overhead + 1024) / 4 * 3;
        let detik = (budget_mentah / 2048) as u32;
        if detik >= 1 {
            if let Ok(pangkas) = kompres_audio_durasi(&asli, detik) {
                let b64 = base64_encode(&pangkas);
                html = player_html_aman(title, artist, &durasi, &poster, &synced, &b64);
            }
        }
    }
    // Buffer asli (≤25 MB) hanya dipakai fallback trim di atas; lepas
    // sebelum rakit `RichAi` supaya puncak RSS tidak menumpuk dua-duanya.
    drop(asli);
    if html.len() > MAX_HTML_PAYLOAD_BYTES {
        return Err("Kartu HTML terlalu besar untuk klien WhatsApp.".to_string());
    }
    let rich = RichAi {
        // Amplop kartu HTML terbukti render di Android 2.26.45.99 (adendum 8,
        // `.cekkartu` K/L/M ×3): botJid `0@bot` + edit susulan identik.
        // Score 999 `sendMusicPlayer` versi awal ditinggalkan — itu bukti versi WA
        // lama; envelope proven memakai score 1. Disclaimer "" tetap (versi awal).
        submessages: vec![AiSubMessage::Teks {
            teks: format!("{title} - {artist}"),
        }],
        sections: vec![section_html(&html)],
        disclaimer: Some(String::new()),
        quoted: false,
        bot_jid: Some("0@bot".into()),
        edit_susulan: true,
        ..Default::default()
    };

    // `with_reaction` di TOP-LEVEL sequence, bukan di elemen: `Mutate`
    // bersarang di dalam `Sequence` tidak pernah lewat `into_parts()`
    // `Pipeline::process`, jadi mencapai `render_reply` dan ditolak
    // (`UnappliedMutation`). Di top-level, `proses` memisahnya dulu.
    Ok(Reply::sequence(vec![
        Reply::reaction("🎧"),
        // Fallback voice note untuk klien yang tidak merender primitif HTML.
        // `compressAudio` menghasilkan ogg/opus — mimetype sama
        // `mimetypeFor("opus")`.
        Reply::voice_note(
            MediaSource::bytes(kompresi).with_mimetype(mimetype_for("opus")),
        ),
        Reply::TabelAi(Box::new(rich)),
    ])
    .with_reaction("✅"))
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
    /// rantai backend ytmp3, dan host audio `cdn.example` — dicocokkan
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

    /// Hasil YT Music dengan satu lagu (jalur judul).
    const HASIL: &str = r#"{"contents":{"tabbedSearchResultsRenderer":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[{"musicShelfRenderer":{"contents":[{"musicResponsiveListItemRenderer":{"flexColumns":[{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":"Lagu Uji"}]}}},{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":"Song"},{"text":" • "},{"text":"Channel Uji","navigationEndpoint":{"browseEndpoint":{"browseId":"UC123"}}},{"text":" • "},{"text":"Album Uji","navigationEndpoint":{"browseEndpoint":{"browseId":"MPREabc"}}},{"text":" • "},{"text":"3:33"}]}}}],"thumbnail":{"musicThumbnailRenderer":{"thumbnail":{"thumbnails":[{"url":"https://i.ytimg.com/vi/a/hq.jpg"}]}}},"overlay":{"musicItemThumbnailOverlayRenderer":{"content":{"musicPlayButtonRenderer":{"playNavigationEndpoint":{"watchEndpoint":{"videoId":"aaaaaaaaaaa"}}}}}}}}]}}]}}}}]}}}"#;

    /// Hasil `yt-search` (jalur URL YouTube).
    #[allow(dead_code)]
    const HASIL_VIDEO: &str = r#"{"contents":{"tabs":[{"tabRenderer":{"content":{"sectionListRenderer":{"contents":[
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
        // `with_reaction("✅")` kini di top-level sequence (bukan di elemen),
        // jadi bentuknya Mutate(Reaction(Sequence)). Di dalam sequence tetap:
        // 🎧 → voice note → kartu RichAi HTML.
        let Reply::Sequence(list) = r.without_mutations().tanpa_reaksi() else {
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
        let Reply::TabelAi(rich) = list[2].without_mutations().tanpa_reaksi() else {
            panic!("harus TabelAi");
        };
        let AiSubMessage::Teks { teks } = &rich.submessages[0] else {
            panic!("harus submessage teks");
        };
        assert_eq!(teks, "Lagu Uji - Channel Uji");
        // Amplop proven adendum 8 (`.cekkartu` K/L/M render ×3 di 2.26.45.99).
        assert_eq!(rich.bot_jid.as_deref(), Some("0@bot"));
        assert!(rich.edit_susulan, "edit susulan identik wajib");
        assert_eq!(rich.skor, None, "score 1 default; 999 JS = bukti WA lama");
        let prim = &rich.sections[0]["view_model"]["primitive"];
        assert_eq!(
            prim["__typename"],
            serde_json::json!("GenAIaeacdsnwHtmlPrimitive")
        );
        let payload = prim["payload"].as_str().expect("payload html");
        assert!(matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "✅"));
        assert!(payload.contains("Lagu Uji"), "{payload}");
        // Audio penuh tertanam (base64 di const versi awal) dan diputar Web Audio —
        // `<audio src>` ditolak origin opaque (adendum 13). Voice note tetap
        // fallback non-Android.
        assert!(payload.contains("AUDIO_B64 = '"), "audio tertanam");
        assert!(payload.contains("decodeAudioData"), "mesin Web Audio");
        assert!(!payload.contains("<audio"), "tanpa <audio src>");
        assert!(payload.contains("id=\"play\""), "kontrol play ada");
        assert!(
            payload.len() <= MAX_HTML_PAYLOAD_BYTES,
            "muatan {} > batas {MAX_HTML_PAYLOAD_BYTES}",
            payload.len()
        );

        let seen = http.seen.lock().unwrap().clone();
        assert!(
            seen.iter()
                .any(|u| u.contains("media.savetube.vip/api/random-cdn")),
            "unduhan harus lewat jalur .play dulu: {seen:?}"
        );
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
        let Reply::Sequence(list) = r.without_mutations().tanpa_reaksi() else {
            panic!("harus sequence");
        };
        // Tanpa lirik: kartu HTML tetap dikirim dengan baris "Lirik belum
        // tersedia".
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
        let Reply::Sequence(list) = r.without_mutations().tanpa_reaksi() else {
            panic!("harus sequence");
        };
        // Lirik polos jadi sinkron interval 8 dtk (213/3=71 → clamp 8); kartu
        // menanam lirik sebagai JSON base64 untuk script player, urut.
        let Reply::TabelAi(rich) = list[2].without_mutations().tanpa_reaksi() else {
            panic!("harus TabelAi");
        };
        let payload = rich.sections[0]["view_model"]["primitive"]["payload"]
            .as_str()
            .expect("payload html");
        let awal = payload.find("const encoded = '").expect("token lirik") + "const encoded = '".len();
        let akhir = payload[awal..].find("';").expect("penutup lirik") + awal;
        let json = decode_base64_test(&payload[awal..akhir]);
        let v: serde_json::Value = serde_json::from_slice(&json).expect("json sah");
        let baris = v.as_array().expect("array lirik");
        let teks: Vec<&str> = baris.iter().map(|b| b["text"].as_str().unwrap()).collect();
        assert_eq!(teks, ["satu", "dua", "tiga"], "urutan lirik");
        let waktu: Vec<f64> = baris.iter().map(|b| b["time"].as_f64().unwrap()).collect();
        assert!(waktu.windows(2).all(|w| w[0] < w[1]), "waktu naik: {waktu:?}");
    }

    #[test]
    fn audio_asli_terlalu_besar_ditolak_sebelum_ffmpeg() {
        // > 25 MB → "Buffer audio bermasalah" (`downloadAudioBuffer`, versi awal:384).
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
        let html = player_html("<b>A</b>", "B & C", "3:33", "", &[], "");
        assert!(html.contains("&lt;b&gt;A&lt;/b&gt;"), "{html}");
        assert!(html.contains("B &amp; C"), "{html}");
        assert!(html.contains("data:image/svg+xml;base64,"), "placeholder SVG");
        assert!(html.contains("Lirik belum tersedia"), "{html}");
        assert!(!html.contains("<audio"), "tanpa <audio>: origin opaque menolak URL");
        assert!(html.contains("decodeAudioData"), "mesin Web Audio");
        assert!(html.contains("<script"), "player interaktif");
    }

    #[test]
    fn player_html_port_penuh_create_music_player() {
        let html = player_html("T", "A", "3:33", "", &[], "QUJD");
        assert!(html.contains("AUDIO_B64 = 'QUJD'"), "audio tertanam");
        for penanda in [
            "class=\"bg\"",
            "class=\"veil\"",
            "YT Music Audio",
            "id=\"heart\"",
            "id=\"mini-lyrics\"",
            "id=\"bar\"",
            "id=\"fill\"",
            "id=\"dot\"",
            "id=\"btn-rw\"",
            "id=\"btn-fw\"",
            "id=\"play\"",
            "id=\"icon-pause\"",
            "decodeAudioData",
            "createBufferSource",
            "requestAnimationFrame",
            "AUDIO_B64",
        ] {
            assert!(html.contains(penanda), "penanda JS hilang: {penanda}");
        }
        // Poster dipakai dua kali (bg blur + poster) seperti versi awal.
        assert_eq!(html.matches("data:image/svg+xml;base64,").count(), 2);
    }

    #[test]
    fn player_html_lirik_json_base64_bisa_dibaca_script() {
        let lirik = vec![
            Baris { time: 1.5, text: "halo <b>".into() },
            Baris { time: 12.0, text: "baris dua".into() },
        ];
        let html = player_html("T", "A", "3:33", "", &lirik, "");
        let awal = html.find("const encoded = '").expect("token lirik") + "const encoded = '".len();
        let akhir = html[awal..].find("';").expect("penutup lirik") + awal;
        let json = decode_base64_test(&html[awal..akhir]);
        let v: serde_json::Value = serde_json::from_slice(&json).expect("json sah");
        assert_eq!(v[0]["time"], json!(1.5));
        assert_eq!(v[0]["text"], json!("halo <b>"), "teks lirik mentah, di-escape DOM nanti");
        assert_eq!(v[1]["text"], json!("baris dua"));
    }

    #[test]
    fn player_html_aman_membuang_poster_yang_membuat_muatan_kelebihan() {
        let besar = format!("data:image/jpeg;base64,{}", "A".repeat(MAX_HTML_PAYLOAD_BYTES));
        let html = player_html_aman("T", "A", "3:33", &besar, &[], "");
        assert!(html.len() <= MAX_HTML_PAYLOAD_BYTES, "{}", html.len());
        assert!(html.contains("data:image/svg+xml;base64,"), "poster diganti placeholder");
        assert!(html.contains(">T</div>"), "judul tetap ada: {html}");
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
        // ffmpeg menolak jalan tanpa berkas keluaran; versi awal mengakhiri
        // `pipe:1`.
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
