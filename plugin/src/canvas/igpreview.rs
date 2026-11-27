//! `plugins/canvas/igpreview.js` — kartu pratinjau post Instagram, dirender
//! lokal dengan crate `image` + `ab_glyph` (tanpa API luar).
//!
//! Geometri kartu, gradient, bayangan, kaca buram, path ikon, dan tata letak
//! teks mengikuti `igpreview.js` apa adanya. Anton diunduh sekali ke
//! `media/fonts`; Inter di-embed sebagai instance statis (lihat `FONT_*`), emoji
//! Apple dibaca dari `media/fonts/emoji-apple-image.json`.

use ab_glyph::{Font, FontArc, GlyphId, ScaleFont, point};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{GenericImageView, ImageOutputFormat, Rgba, RgbaImage};
use oktz_plugin_api::{Ctx, IncomingKind, MediaSource, Reply, plugin};
use std::collections::HashMap;
use std::io::Cursor;

const LEBAR: u32 = 1080;
const TINGGI: u32 = 1920;
const BATAS_PX: u64 = 16_383 * 16_383;
const BATAS_URL_BYTE: usize = 10 * 1024 * 1024;

const WARNA_KARTU: [u8; 3] = [0x23, 0x27, 0x2e];
const WARNA_KOLASE: [u8; 3] = [0xc9, 0xcc, 0xd6];
const PUTIH: Rgba<u8> = Rgba([255, 255, 255, 255]);

const URL_ANTON: &str = "https://github.com/google/fonts/raw/main/ofl/anton/Anton-Regular.ttf";
const JALUR_ANTON: &str = "media/fonts/Anton-Regular.ttf";

// Canvas memakai Inter variabel dengan optical sizing otomatis: sumbu `opsz`
// mengikuti ukuran px (mentok 32). `ab_glyph` tidak bisa memilih sumbu, jadi
// tiap teks memakai instance statis (opsz, wght) yang dipakai persis oleh JS.
const FONT_NAMA: &[u8] = include_bytes!("../../../assets/fonts/inter/Inter-Nama.ttf");
const FONT_DESKRIPSI: &[u8] = include_bytes!("../../../assets/fonts/inter/Inter-Deskripsi.ttf");
const FONT_KOLASE: &[u8] = include_bytes!("../../../assets/fonts/inter/Inter-Kolase.ttf");
const FONT_MENU: &[u8] = include_bytes!("../../../assets/fonts/inter/Inter-Menu.ttf");

const PETA_EMOJI: &str = "media/fonts/emoji-apple-image.json";

const GAGAL_UNDUH: &str = "❌ Gagal mengunduh gambar!";
const GAGAL_RENDER: &str = "❌ Gagal membuat preview, coba lagi beberapa saat.";

/// Teks bantuan — `igpreview.js:49-64`.
fn help(panggilan: &str) -> String {
    format!(
        "*📸 INSTAGRAM PREVIEW CARD MAKER*\n\n\
         Reply/kirim gambar dengan format:\n\
         `{panggilan} nama | deskripsi | kata-kata | background(opsional)`\n\n\
         ✦ *Opsi Background:*\n\
         • Kosong ➔ Collage grid default\n\
         • URL gambar ➔ `https://...`\n\
         • Warna hex ➔ `#1e293b`\n\
         • `blur` ➔ Blur dari foto utama\n\n\
         *Contoh:*\n\
         `{panggilan} kyu manusia biasa | Semangat | tips waras = punya cewe cakep | blur`"
    )
}

/// `text.split('|').map(trim)` — maks 4 bagian, sisanya diabaikan.
pub(crate) fn split_args(text: &str) -> [String; 4] {
    let mut out = [const { String::new() }; 4];
    for (slot, part) in out.iter_mut().zip(text.split('|')) {
        *slot = part.trim().to_string();
    }
    out
}

/// `#rgb` / `#rrggbb` → RGB.
pub(crate) fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let hex = s.strip_prefix('#')?;
    let digit = |c: char| c.to_digit(16).map(|v| v as u8);
    let mut it = hex.chars();
    match hex.chars().count() {
        3 => {
            let r = digit(it.next()?)?;
            let g = digit(it.next()?)?;
            let b = digit(it.next()?)?;
            Some([r * 17, g * 17, b * 17])
        }
        6 => {
            let mut byte = || {
                let a = digit(it.next()?)?;
                let b = digit(it.next()?)?;
                Some(a * 16 + b)
            };
            Some([byte()?, byte()?, byte()?])
        }
        _ => None,
    }
}

/// Background hasil parsing argumen ke-4.
#[derive(Debug, PartialEq)]
pub(crate) enum Bg {
    Default,
    Warna([u8; 3]),
    Blur,
    Url(String),
    Gambar(Vec<u8>),
}

pub(crate) fn parse_bg(arg: &str) -> Bg {
    let a = arg.trim();
    if a.is_empty() {
        return Bg::Default;
    }
    if a.eq_ignore_ascii_case("blur") {
        return Bg::Blur;
    }
    if a.starts_with("http://") || a.starts_with("https://") {
        return Bg::Url(a.to_string());
    }
    if let Some(rgb) = parse_hex(a) {
        return Bg::Warna(rgb);
    }
    Bg::Default
}

// ---------------------------------------------------------------------------
// Pixel
// ---------------------------------------------------------------------------

fn taruh_alfa(img: &mut RgbaImage, x: i64, y: i64, warna: Rgba<u8>, alfa: f32) {
    if alfa <= 0.0 || (x < 0) || (y < 0) || x >= img.width() as i64 || y >= img.height() as i64 {
        return;
    }
    let a = (alfa.clamp(0.0, 1.0)) * (warna[3] as f32 / 255.0);
    if a >= 0.999 {
        img.put_pixel(
            x as u32,
            y as u32,
            Rgba([warna[0], warna[1], warna[2], 255]),
        );
        return;
    }
    let latar = img.get_pixel(x as u32, y as u32);
    let campur = |l: u8, u: u8| (l as f32 * (1.0 - a) + u as f32 * a).round() as u8;
    img.put_pixel(
        x as u32,
        y as u32,
        Rgba([
            campur(latar[0], warna[0]),
            campur(latar[1], warna[1]),
            campur(latar[2], warna[2]),
            255,
        ]),
    );
}

#[derive(Clone, Copy)]
struct Bundar {
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
    r: f32,
}

impl Bundar {
    fn dari(x: f32, y: f32, w: f32, h: f32, r: f32) -> Self {
        let hw = w / 2.0;
        let hh = h / 2.0;
        Self {
            cx: x + hw,
            cy: y + hh,
            hw,
            hh,
            r: r.min(hw).min(hh),
        }
    }

    fn x(&self) -> f32 {
        self.cx - self.hw
    }

    fn y(&self) -> f32 {
        self.cy - self.hh
    }

    fn w(&self) -> f32 {
        self.hw * 2.0
    }

    fn h(&self) -> f32 {
        self.hh * 2.0
    }

    fn sdf(&self, px: f32, py: f32) -> f32 {
        let qx = (px - self.cx).abs() - (self.hw - self.r);
        let qy = (py - self.cy).abs() - (self.hh - self.r);
        let ax = qx.max(0.0);
        let ay = qy.max(0.0);
        (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - self.r
    }

    fn isi(&self, img: &mut RgbaImage, warna: Rgba<u8>) {
        let (w, h) = (img.width(), img.height());
        self.jelajah(w, h, |px, py, cov| {
            taruh_alfa(img, px, py, warna, cov);
        });
    }

    /// Topeng alfa bentuk: piksel ditulis `(hitam, cov*alfa)`, bukan dicampur.
    fn topeng(&self, img: &mut RgbaImage, alfa: f32) {
        let a = alfa.clamp(0.0, 1.0);
        let (w, h) = (img.width(), img.height());
        self.jelajah(w, h, |px, py, cov| {
            let alfa = (cov * a * 255.0).round() as u8;
            img.put_pixel(px as u32, py as u32, Rgba([0, 0, 0, alfa]));
        });
    }

    fn jelajah(&self, w: u32, h: u32, mut aksi: impl FnMut(i64, i64, f32)) {
        let x0 = self.x().floor().max(0.0) as i64;
        let y0 = self.y().floor().max(0.0) as i64;
        let x1 = ((self.x() + self.w()).ceil() as i64).clamp(0, w as i64);
        let y1 = ((self.y() + self.h()).ceil() as i64).clamp(0, h as i64);
        for py in y0..y1 {
            for px in x0..x1 {
                let cov = (0.5 - self.sdf(px as f32 + 0.5, py as f32 + 0.5)).clamp(0.0, 1.0);
                if cov > 0.0 {
                    aksi(px, py, cov);
                }
            }
        }
    }
}

fn jarak_segmen(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let panjang = dx * dx + dy * dy;
    let t = if panjang <= f32::EPSILON {
        0.0
    } else {
        (((px - a.0) * dx + (py - a.1) * dy) / panjang).clamp(0.0, 1.0)
    };
    let (qx, qy) = (a.0 + t * dx, a.1 + t * dy);
    ((px - qx).powi(2) + (py - qy).powi(2)).sqrt()
}

/// `ctx.stroke()` pada polyline, ketebalan `lebar`, ujung bulat.
fn gores(img: &mut RgbaImage, titik: &[(f32, f32)], lebar: f32, warna: Rgba<u8>, tutup: bool) {
    let Some(pertama) = titik.first().copied() else {
        return;
    };
    let (mut minx, mut miny) = (pertama.0, pertama.1);
    let (mut maxx, mut maxy) = (pertama.0, pertama.1);
    for (x, y) in titik {
        minx = minx.min(*x);
        miny = miny.min(*y);
        maxx = maxx.max(*x);
        maxy = maxy.max(*y);
    }
    let tepi = lebar / 2.0 + 1.0;
    let x0 = (minx - tepi).floor().max(0.0) as i64;
    let y0 = (miny - tepi).floor().max(0.0) as i64;
    let x1 = ((maxx + tepi).ceil() as i64).min(img.width() as i64);
    let y1 = ((maxy + tepi).ceil() as i64).min(img.height() as i64);
    for py in y0..y1 {
        for px in x0..x1 {
            let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
            let mut jarak = f32::MAX;
            if titik.len() == 1 {
                jarak = ((fx - pertama.0).powi(2) + (fy - pertama.1).powi(2)).sqrt();
            } else {
                for w in titik.windows(2) {
                    jarak = jarak.min(jarak_segmen(fx, fy, w[0], w[1]));
                }
                if tutup && let Some(akhir) = titik.last() {
                    jarak = jarak.min(jarak_segmen(fx, fy, *akhir, pertama));
                }
            }
            let cov = (lebar / 2.0 + 0.5 - jarak).clamp(0.0, 1.0);
            taruh_alfa(img, px, py, warna, cov);
        }
    }
}

fn titik_bezier(p: [(f32, f32); 4], keluar: &mut Vec<(f32, f32)>) {
    const LANGKAH: usize = 32;
    for i in 0..=LANGKAH {
        let t = i as f32 / LANGKAH as f32;
        let u = 1.0 - t;
        let x = u * u * u * p[0].0
            + 3.0 * u * u * t * p[1].0
            + 3.0 * u * t * t * p[2].0
            + t * t * t * p[3].0;
        let y = u * u * u * p[0].1
            + 3.0 * u * u * t * p[1].1
            + 3.0 * u * t * t * p[2].1
            + t * t * t * p[3].1;
        keluar.push((x, y));
    }
}

fn titik_busur(cx: f32, cy: f32, r: f32, a0: f32, a1: f32, keluar: &mut Vec<(f32, f32)>) {
    const LANGKAH: usize = 64;
    for i in 0..=LANGKAH {
        let a = a0 + (a1 - a0) * i as f32 / LANGKAH as f32;
        keluar.push((cx + r * a.cos(), cy + r * a.sin()));
    }
}

fn isi_gradien_tegak(img: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, stop: &[(f32, [u8; 3])]) {
    let x0 = x.floor().max(0.0) as i64;
    let y0 = y.floor().max(0.0) as i64;
    let x1 = ((x + w).ceil() as i64).min(img.width() as i64);
    let y1 = ((y + h).ceil() as i64).min(img.height() as i64);
    for py in y0..y1 {
        let t = ((py as f32 + 0.5 - y) / h).clamp(0.0, 1.0);
        let warna = warna_stop3(stop, t);
        for px in x0..x1 {
            img.put_pixel(
                px as u32,
                py as u32,
                Rgba([warna[0], warna[1], warna[2], 255]),
            );
        }
    }
}

fn warna_stop3(stop: &[(f32, [u8; 3])], t: f32) -> [u8; 3] {
    if stop.is_empty() {
        return [0, 0, 0];
    }
    if t <= stop[0].0 {
        return stop[0].1;
    }
    for w in stop.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t <= t1 {
            let f = if (t1 - t0).abs() <= f32::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            return [
                (c0[0] as f32 + (c1[0] as f32 - c0[0] as f32) * f).round() as u8,
                (c0[1] as f32 + (c1[1] as f32 - c0[1] as f32) * f).round() as u8,
                (c0[2] as f32 + (c1[2] as f32 - c0[2] as f32) * f).round() as u8,
            ];
        }
    }
    stop[stop.len() - 1].1
}

fn warna_stop4(stop: &[(f32, [u8; 4])], t: f32) -> [u8; 4] {
    if stop.is_empty() {
        return [0, 0, 0, 0];
    }
    if t <= stop[0].0 {
        return stop[0].1;
    }
    for w in stop.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t <= t1 {
            let f = if (t1 - t0).abs() <= f32::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            let mut out = [0u8; 4];
            for i in 0..4 {
                out[i] = (c0[i] as f32 + (c1[i] as f32 - c0[i] as f32) * f).round() as u8;
            }
            return out;
        }
    }
    stop[stop.len() - 1].1
}

fn isi_gradien_radial(
    img: &mut RgbaImage,
    cx: f32,
    cy: f32,
    r0: f32,
    r1: f32,
    klip: (f32, f32, f32, f32),
    stop: &[(f32, [u8; 4])],
) {
    let (kx, ky, kw, kh) = klip;
    let x0 = kx.floor().max(0.0) as i64;
    let y0 = ky.floor().max(0.0) as i64;
    let x1 = ((kx + kw).ceil() as i64).min(img.width() as i64);
    let y1 = ((ky + kh).ceil() as i64).min(img.height() as i64);
    let bentang = (r1 - r0).max(f32::EPSILON);
    for py in y0..y1 {
        for px in x0..x1 {
            let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
            let jarak = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            let t = ((jarak - r0) / bentang).clamp(0.0, 1.0);
            let warna = warna_stop4(stop, t);
            taruh_alfa(img, px, py, Rgba(warna), 1.0);
        }
    }
}

fn cover(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let (iw, ih) = img.dimensions();
    let rasio_target = w as f32 / h as f32;
    let rasio_sumber = iw as f32 / ih as f32;
    let potong = if rasio_sumber > rasio_target {
        let nw = ((ih as f32 * rasio_target) as u32).max(1);
        let x = iw.saturating_sub(nw) / 2;
        image::imageops::crop_imm(img, x, 0, nw, ih).to_image()
    } else {
        let nh = ((iw as f32 / rasio_target) as u32).max(1);
        let y = ih.saturating_sub(nh) / 2;
        image::imageops::crop_imm(img, 0, y, iw, nh).to_image()
    };
    image::imageops::resize(&potong, w, h, FilterType::Triangle)
}

fn tempel_bundar(
    kanvas: &mut RgbaImage,
    sumber: &RgbaImage,
    x: i64,
    y: i64,
    w: u32,
    h: u32,
    bundar: Bundar,
) {
    let isi = cover(sumber, w, h);
    for dy in 0..h {
        for dx in 0..w {
            let gx = x + dx as i64;
            let gy = y + dy as i64;
            let cov = (0.5 - bundar.sdf(gx as f32 + 0.5, gy as f32 + 0.5)).clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }
            taruh_alfa(kanvas, gx, gy, *isi.get_pixel(dx, dy), cov);
        }
    }
}

/// `ctx.shadowBlur` + `shadowOffsetY` untuk bentuk rounded-rect.
fn bayangan(kanvas: &mut RgbaImage, b: Bundar, blur: f32, alfa: f32, offset_y: f32) {
    let sigma = (blur / 2.0).max(0.1);
    let margin = (sigma * 3.0).ceil() as u32;
    let mw = b.w().ceil() as u32 + margin * 2;
    let mh = b.h().ceil() as u32 + margin * 2;
    let mut topeng = RgbaImage::new(mw, mh);
    let lokal = Bundar {
        cx: margin as f32 + b.hw,
        cy: margin as f32 + b.hh,
        ..b
    };
    lokal.topeng(&mut topeng, alfa);
    let pudar = image::imageops::blur(&topeng, sigma);
    image::imageops::overlay(
        kanvas,
        &pudar,
        b.x() as i64 - margin as i64,
        (b.y() + offset_y) as i64 - margin as i64,
    );
}

/// Background foto: cover penuh, gelap 0.75, lalu blur 12px.
fn pudar_foto(img: &RgbaImage) -> RgbaImage {
    let mut redup = img.clone();
    for p in redup.pixels_mut() {
        for kanal in p.0.iter_mut().take(3) {
            *kanal = (*kanal as f32 * 0.75).round() as u8;
        }
    }
    image::imageops::blur(&redup, 12.0)
}

// ---------------------------------------------------------------------------
// Teks & emoji
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Dasar {
    Atas,
    Tengah,
}

#[derive(Clone, Copy)]
struct GayaTeks<'a> {
    font: &'a FontArc,
    ukuran: f32,
    warna: Rgba<u8>,
    dasar: Dasar,
    emoji: Option<(f32, f32)>,
}

/// `ab_glyph` mengukur tinggi font sebagai ascent+descent, sedangkan canvas
/// memakai ukuran em. Konversi supaya metrik teks sama dengan `ctx.font`.
fn skala_em(font: &FontArc, ukuran: f32) -> f32 {
    let tinggi = font.height_unscaled();
    match font.units_per_em() {
        Some(em) if em > 0.0 && tinggi.abs() > f32::EPSILON => ukuran * tinggi / em,
        _ => ukuran,
    }
}

fn lebar_teks(font: &FontArc, teks: &str, ukuran: f32) -> f32 {
    let skala = font.as_scaled(skala_em(font, ukuran));
    let mut lebar = 0.0;
    let mut sebelum: Option<GlyphId> = None;
    for c in teks.chars() {
        if c == '\n' {
            continue;
        }
        let id = skala.glyph_id(c);
        if let Some(p) = sebelum {
            lebar += skala.kern(p, id);
        }
        lebar += skala.h_advance(id);
        sebelum = Some(id);
    }
    lebar
}

fn wrap_teks(font: &FontArc, teks: &str, ukuran: f32, lebar_maks: f32) -> Vec<String> {
    let mut baris = Vec::new();
    for paragraf in teks.split('\n').map(|p| p.trim_end_matches('\r')) {
        let kata: Vec<&str> = paragraf.split_whitespace().collect();
        if kata.is_empty() {
            baris.push(String::new());
            continue;
        }
        let mut kini = String::new();
        for satu in kata {
            let coba = if kini.is_empty() {
                satu.to_string()
            } else {
                format!("{kini} {satu}")
            };
            if kini.is_empty() || lebar_teks(font, &coba, ukuran) <= lebar_maks {
                kini = coba;
            } else {
                baris.push(std::mem::take(&mut kini));
                kini = satu.to_string();
            }
        }
        if !kini.is_empty() {
            baris.push(kini);
        }
    }
    baris
}

/// Awal grapheme berikut — cukup untuk emoji (ZWJ, variasi, skin tone, tag).
fn grafem(teks: &str) -> Vec<&str> {
    fn penanda(c: char) -> bool {
        matches!(c as u32,
            0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05BD | 0x0610..=0x061A
            | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06DC | 0x0730..=0x074A
            | 0x07A6..=0x07B0 | 0x0900..=0x0903 | 0x093A..=0x094F | 0x0951..=0x0957
            | 0x0962..=0x0963 | 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E
            | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF
            | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F | 0x1F3FB..=0x1F3FF
            | 0xE0020..=0xE007F)
    }
    fn ri(c: char) -> bool {
        matches!(c as u32, 0x1F1E6..=0x1F1FF)
    }
    let chars: Vec<(usize, char)> = teks.char_indices().collect();
    let mut hasil = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let mulai = i;
        i += 1;
        let mut ri_terpakai = ri(chars[mulai].1);
        while i < chars.len() {
            let c = chars[i].1;
            if c == '\u{200D}' {
                i += 1;
                if i < chars.len() {
                    i += 1;
                }
                while i < chars.len() && penanda(chars[i].1) {
                    i += 1;
                }
                ri_terpakai = false;
            } else if penanda(c) {
                i += 1;
            } else if ri(c) && ri_terpakai {
                i += 1;
                ri_terpakai = false;
            } else {
                break;
            }
        }
        let akhir = chars.get(i).map(|(idx, _)| *idx).unwrap_or(teks.len());
        hasil.push(&teks[chars[mulai].0..akhir]);
    }
    hasil
}

fn decode_base64(teks: &str) -> Vec<u8> {
    let mut keluar = Vec::with_capacity(teks.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bit = 0u32;
    for b in teks.bytes() {
        let nilai = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => continue,
        };
        buf = ((buf << 6) | nilai as u32) & 0x00FF_FFFF;
        bit += 6;
        if bit >= 8 {
            bit -= 8;
            keluar.push((buf >> bit) as u8);
        }
    }
    keluar
}

fn data_emoji<'a>(grafem: &str, peta: &'a HashMap<String, String>) -> Option<&'a str> {
    if let Some(v) = peta.get(grafem) {
        return Some(v);
    }
    let kode: Vec<String> = grafem.chars().map(|c| format!("{:x}", c as u32)).collect();
    if let Some(v) = peta.get(&kode.join("-")) {
        return Some(v);
    }
    let tanpa: Vec<&str> = kode
        .iter()
        .map(String::as_str)
        .filter(|k| *k != "fe0f")
        .collect();
    peta.get(&tanpa.join("-")).map(String::as_str)
}

fn emoji_untuk<'a>(
    grafem: &str,
    peta: Option<&HashMap<String, String>>,
    cache: &'a mut HashMap<String, Option<RgbaImage>>,
) -> Option<&'a RgbaImage> {
    let peta = peta?;
    if grafem.chars().all(|c| (c as u32) < 0x00A9) {
        return None;
    }
    let kunci = grafem.replace('\u{FE0F}', "");
    if !cache.contains_key(&kunci) {
        let gambar = data_emoji(grafem, peta)
            .and_then(|b64| b64.split(',').next_back())
            .map(decode_base64)
            .and_then(|mentah| image::load_from_memory(&mentah).ok())
            .map(|img| img.to_rgba8());
        cache.insert(kunci.clone(), gambar);
    }
    cache.get(&kunci).and_then(|v| v.as_ref())
}

/// `textBaseline: top/middle` canvas: ascent dinormalkan ke tinggi font
/// (ascent+descent), bukan ke em seperti skala glyph.
fn garis_dasar(font: &FontArc, ukuran: f32, y: f32, dasar: Dasar) -> f32 {
    let tinggi = font.height_unscaled();
    let atas = if tinggi.abs() > f32::EPSILON {
        ukuran * font.ascent_unscaled() / tinggi
    } else {
        ukuran
    };
    match dasar {
        Dasar::Atas => y + atas,
        Dasar::Tengah => y + atas - ukuran / 2.0,
    }
}

fn gambar_glyph(
    img: &mut RgbaImage,
    font: &FontArc,
    c: char,
    x: f32,
    baseline: f32,
    ukuran: f32,
    warna: Rgba<u8>,
) {
    let glyph = font
        .glyph_id(c)
        .with_scale_and_position(skala_em(font, ukuran), point(x, baseline));
    let Some(outline) = font.outline_glyph(glyph) else {
        return;
    };
    let bounds = outline.px_bounds();
    outline.draw(|gx, gy, cov| {
        if cov <= 0.0 {
            return;
        }
        taruh_alfa(
            img,
            bounds.min.x as i64 + gx as i64,
            bounds.min.y as i64 + gy as i64,
            warna,
            cov,
        );
    });
}

fn gambar_teks(img: &mut RgbaImage, teks: &str, x: f32, y: f32, gaya: &GayaTeks) {
    let skala = gaya.font.as_scaled(skala_em(gaya.font, gaya.ukuran));
    let baseline = garis_dasar(gaya.font, gaya.ukuran, y, gaya.dasar);
    let mut kursor = x;
    let mut sebelum: Option<GlyphId> = None;
    for c in teks.chars() {
        if c == '\n' {
            continue;
        }
        let id = skala.glyph_id(c);
        if let Some(p) = sebelum {
            kursor += skala.kern(p, id);
        }
        gambar_glyph(img, gaya.font, c, kursor, baseline, gaya.ukuran, gaya.warna);
        kursor += skala.h_advance(id);
        sebelum = Some(id);
    }
}

fn gambar_rich(
    img: &mut RgbaImage,
    teks: &str,
    x: f32,
    y: f32,
    gaya: &GayaTeks,
    peta: Option<&HashMap<String, String>>,
    cache: &mut HashMap<String, Option<RgbaImage>>,
) {
    let skala = gaya.font.as_scaled(skala_em(gaya.font, gaya.ukuran));
    let baseline = garis_dasar(gaya.font, gaya.ukuran, y, gaya.dasar);
    let (emoji_ukuran, emoji_offset) = gaya.emoji.unwrap_or((gaya.ukuran, 0.0));
    let mut kursor = x;
    for satu in grafem(teks) {
        if let Some(gambar) = emoji_untuk(satu, peta, cache) {
            let w = emoji_ukuran.max(1.0) as u32;
            let h = emoji_ukuran.max(1.0) as u32;
            let kecil = image::imageops::resize(gambar, w, h, FilterType::Triangle);
            image::imageops::overlay(
                img,
                &kecil,
                (kursor + 1.0) as i64,
                (y + emoji_offset) as i64,
            );
            kursor += emoji_ukuran + 4.0;
            continue;
        }
        let mut sebelum: Option<GlyphId> = None;
        for c in satu.chars() {
            if c == '\n' {
                continue;
            }
            let id = skala.glyph_id(c);
            if let Some(p) = sebelum {
                kursor += skala.kern(p, id);
            }
            gambar_glyph(img, gaya.font, c, kursor, baseline, gaya.ukuran, gaya.warna);
            kursor += skala.h_advance(id);
            sebelum = Some(id);
        }
    }
}

// ---------------------------------------------------------------------------
// Font
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub(crate) struct Fonts {
    anton: Option<FontArc>,
    regular: Option<FontArc>,
    medium: Option<FontArc>,
    semibold: Option<FontArc>,
    bold: Option<FontArc>,
}

impl Fonts {
    fn pilih<'a>(&'a self, urutan: &[&'a Option<FontArc>]) -> Option<&'a FontArc> {
        urutan.iter().find_map(|f| f.as_ref())
    }

    fn display(&self) -> Option<&FontArc> {
        self.pilih(&[&self.anton, &self.bold, &self.semibold, &self.regular])
    }

    fn teks(&self) -> Option<&FontArc> {
        self.pilih(&[&self.regular, &self.medium, &self.semibold, &self.bold])
    }

    fn sedang(&self) -> Option<&FontArc> {
        self.pilih(&[&self.medium, &self.regular, &self.semibold, &self.bold])
    }

    fn setengah_tebal(&self) -> Option<&FontArc> {
        self.pilih(&[&self.semibold, &self.medium, &self.regular, &self.bold])
    }

    fn tebal(&self) -> Option<&FontArc> {
        self.pilih(&[&self.bold, &self.semibold, &self.medium, &self.regular])
    }
}

fn tampak_font(data: &[u8]) -> bool {
    data.len() > 1024
        && (data.starts_with(&[0x00, 0x01, 0x00, 0x00])
            || data.starts_with(b"true")
            || data.starts_with(b"OTTO"))
}

fn font_dari(ctx: &Ctx, url: &str, jalur: &str) -> Option<FontArc> {
    if !ctx.berkas().ada(jalur) {
        let res = ctx.http.get(url).ok()?;
        let data = res.body.bytes().to_vec();
        if !tampak_font(&data) {
            return None;
        }
        ctx.berkas().tulis(jalur, &data).ok()?;
    }
    FontArc::try_from_vec(ctx.berkas().baca(jalur).ok()?).ok()
}

fn font_bawaan(data: &'static [u8]) -> Option<FontArc> {
    FontArc::try_from_slice(data).ok()
}

fn muat_fonts(ctx: &Ctx) -> Fonts {
    Fonts {
        anton: font_dari(ctx, URL_ANTON, JALUR_ANTON),
        regular: font_bawaan(FONT_DESKRIPSI),
        medium: font_bawaan(FONT_MENU),
        semibold: font_bawaan(FONT_KOLASE),
        bold: font_bawaan(FONT_NAMA),
    }
}

fn muat_peta_emoji(ctx: &Ctx) -> Option<HashMap<String, String>> {
    if !ctx.berkas().ada(PETA_EMOJI) {
        return None;
    }
    let mentah = ctx.berkas().baca(PETA_EMOJI).ok()?;
    serde_json::from_slice(&mentah).ok()
}

// ---------------------------------------------------------------------------
// Render murni — tanpa Ctx, testable.
// ---------------------------------------------------------------------------

const LABEL_KOLASE: [&str; 15] = [
    "hidup",
    "bagemara pun hsaingki kamu",
    "tl bea",
    "tips",
    "",
    "hidup",
    "cda ean",
    "ppg dia",
    "",
    "tl bea",
    "hsaingki kamu",
    "",
    "cda ean",
    "",
    "ppg dia",
];

fn kolase(fonts: &Fonts) -> RgbaImage {
    let mut img = RgbaImage::from_pixel(
        LEBAR,
        TINGGI,
        Rgba([WARNA_KOLASE[0], WARNA_KOLASE[1], WARNA_KOLASE[2], 255]),
    );
    let kolom = 3u32;
    let baris = 5u32;
    let cw = LEBAR / kolom;
    let ch = TINGGI / baris;
    for r in 0..baris {
        for c in 0..kolom {
            let (x, y) = (c * cw, r * ch);
            let (fx, fy) = (x as f32, y as f32);
            let (fw, fh) = (cw as f32, ch as f32);
            isi_gradien_tegak(
                &mut img,
                fx,
                fy,
                fw,
                fh,
                &[
                    (0.0, [90, 169, 236]),
                    (0.6, [207, 231, 251]),
                    (1.0, [232, 241, 250]),
                ],
            );
            isi_gradien_radial(
                &mut img,
                fx + fw / 2.0,
                fy + fh * 0.92,
                10.0,
                fw * 0.72,
                (fx - fw * 0.2, fy + fh * 0.55, fw * 1.4, fh * 0.5),
                &[
                    (0.0, [63, 154, 44, 255]),
                    (0.7, [122, 193, 67, 255]),
                    (1.0, [122, 193, 67, 0]),
                ],
            );
            let label = LABEL_KOLASE[(r * kolom + c) as usize];
            if label.is_empty() {
                continue;
            }
            let Some(font) = fonts.setengah_tebal() else {
                continue;
            };
            let gaya = GayaTeks {
                font,
                ukuran: 40.0,
                warna: Rgba([0x22, 0x22, 0x22, 255]),
                dasar: Dasar::Atas,
                emoji: None,
            };
            for (i, satu) in wrap_teks(font, label, 40.0, fw - 40.0).iter().enumerate() {
                gambar_teks(
                    &mut img,
                    satu,
                    fx + 20.0,
                    fy + 24.0 + 46.0 * i as f32,
                    &gaya,
                );
            }
        }
    }
    let mut pudar = image::imageops::blur(&img, 3.0);
    for p in pudar.pixels_mut() {
        let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
        let luma = 0.213 * r + 0.715 * g + 0.072 * b;
        let campur = |c: f32, dasar: u8| {
            let disaring = luma + (c - luma) * 0.8;
            (disaring * 0.85 + dasar as f32 * 0.15)
                .clamp(0.0, 255.0)
                .round() as u8
        };
        *p = Rgba([
            campur(r, WARNA_KOLASE[0]),
            campur(g, WARNA_KOLASE[1]),
            campur(b, WARNA_KOLASE[2]),
            255,
        ]);
    }
    pudar
}

fn tempel_penuh(isi: &RgbaImage) -> RgbaImage {
    let mut kanvas = RgbaImage::new(LEBAR, TINGGI);
    image::imageops::overlay(&mut kanvas, isi, -30, -30);
    kanvas
}

/// Lapisan dasar sebelum blur/redup — juga dipakai kaca menu, yang di canvas
/// mem-blur ulang foto asli (bukan latar yang sudah diredupkan).
fn sumber_latar(k: &Konten, foto: &RgbaImage) -> Result<RgbaImage, String> {
    match k.bg {
        Bg::Warna(rgb) => Ok(RgbaImage::from_pixel(
            LEBAR,
            TINGGI,
            Rgba([rgb[0], rgb[1], rgb[2], 255]),
        )),
        Bg::Blur => Ok(tempel_penuh(&cover(foto, LEBAR + 60, TINGGI + 60))),
        Bg::Gambar(data) => {
            let img = decode_kecil(data, 1600)?;
            Ok(tempel_penuh(&cover(&img, LEBAR + 60, TINGGI + 60)))
        }
        _ => Ok(kolase(k.fonts)),
    }
}

/// Kaca buram menu: sumber di-blur 30px lewat klip rounded-rect, lalu tinta 55%.
fn kaca_menu(kanvas: &mut RgbaImage, sumber: &RgbaImage, b: Bundar) {
    let sigma = 30.0f32;
    let margin = (sigma * 3.0) as i64;
    let x0 = (b.x().floor() as i64 - margin).max(0);
    let y0 = (b.y().floor() as i64 - margin).max(0);
    let x1 = ((b.x() + b.w()).ceil() as i64 + margin).min(sumber.width() as i64);
    let y1 = ((b.y() + b.h()).ceil() as i64 + margin).min(sumber.height() as i64);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let potong = image::imageops::crop_imm(
        sumber,
        x0 as u32,
        y0 as u32,
        (x1 - x0) as u32,
        (y1 - y0) as u32,
    )
    .to_image();
    let pudar = image::imageops::blur(&potong, sigma);
    let bx = b.x().floor() as i64;
    let by = b.y().floor() as i64;
    let bxi = (b.x() + b.w()).ceil() as i64;
    let byi = (b.y() + b.h()).ceil() as i64;
    for py in by.max(y0)..byi.min(y1) {
        for px in bx.max(x0)..bxi.min(x1) {
            let cov = (0.5 - b.sdf(px as f32 + 0.5, py as f32 + 0.5)).clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }
            let p = pudar.get_pixel((px - x0) as u32, (py - y0) as u32);
            taruh_alfa(kanvas, px, py, Rgba([p[0], p[1], p[2], 255]), cov);
        }
    }
    b.isi(kanvas, Rgba([120, 120, 125, 140]));
}

struct MenuAksi {
    label: &'static str,
    ikon: Ikon,
}

#[derive(Clone, Copy)]
enum Ikon {
    Hati,
    Balon,
    Ulang,
    Kirim,
}

const MENU: [MenuAksi; 4] = [
    MenuAksi {
        label: "Suka",
        ikon: Ikon::Hati,
    },
    MenuAksi {
        label: "Komentar",
        ikon: Ikon::Balon,
    },
    MenuAksi {
        label: "Posting ulang",
        ikon: Ikon::Ulang,
    },
    MenuAksi {
        label: "Bagikan",
        ikon: Ikon::Kirim,
    },
];

/// `drawMenuIcon` — semua path digores putih 4.2px, ujung bulat.
fn gambar_ikon(img: &mut RgbaImage, ikon: Ikon, cx: f32, cy: f32, ukuran: f32) {
    let r = ukuran / 2.0;
    let tebal = 4.2;
    match ikon {
        Ikon::Hati => {
            let mut p = vec![(cx, cy + r * 0.78)];
            titik_bezier(
                [
                    (cx, cy + r * 0.78),
                    (cx - r * 1.25, cy - r * 0.05),
                    (cx - r * 0.65, cy - r * 0.98),
                    (cx, cy - r * 0.35),
                ],
                &mut p,
            );
            titik_bezier(
                [
                    (cx, cy - r * 0.35),
                    (cx + r * 0.65, cy - r * 0.98),
                    (cx + r * 1.25, cy - r * 0.05),
                    (cx, cy + r * 0.78),
                ],
                &mut p,
            );
            gores(img, &p, tebal, PUTIH, false);
        }
        Ikon::Balon => {
            let mut p = Vec::new();
            titik_busur(
                cx,
                cy,
                r * 0.78,
                std::f32::consts::PI * 0.78,
                std::f32::consts::PI * 2.55,
                &mut p,
            );
            p.push((cx - r * 0.78, cy + r * 0.78));
            gores(img, &p, tebal, PUTIH, true);
        }
        Ikon::Ulang => {
            let w = r * 0.72;
            let h = r * 0.55;
            gores(
                img,
                &[
                    (cx - w, cy - h * 0.2),
                    (cx - w, cy - h),
                    (cx + w * 0.75, cy - h),
                ],
                tebal,
                PUTIH,
                false,
            );
            gores(
                img,
                &[
                    (cx + w * 0.4, cy - h - 6.0),
                    (cx + w * 0.8, cy - h),
                    (cx + w * 0.4, cy - h + 6.0),
                ],
                tebal,
                PUTIH,
                false,
            );
            gores(
                img,
                &[
                    (cx + w, cy + h * 0.2),
                    (cx + w, cy + h),
                    (cx - w * 0.75, cy + h),
                ],
                tebal,
                PUTIH,
                false,
            );
            gores(
                img,
                &[
                    (cx - w * 0.4, cy + h - 6.0),
                    (cx - w * 0.8, cy + h),
                    (cx - w * 0.4, cy + h + 6.0),
                ],
                tebal,
                PUTIH,
                false,
            );
        }
        Ikon::Kirim => {
            gores(
                img,
                &[
                    (cx - r * 0.8, cy - r * 0.2),
                    (cx + r * 0.8, cy - r * 0.75),
                    (cx + r * 0.18, cy + r * 0.82),
                    (cx - r * 0.08, cy + r * 0.1),
                ],
                tebal,
                PUTIH,
                true,
            );
            gores(
                img,
                &[(cx - r * 0.08, cy + r * 0.1), (cx + r * 0.8, cy - r * 0.75)],
                tebal,
                PUTIH,
                false,
            );
        }
    }
}

fn decode_kecil(data: &[u8], maks: u32) -> Result<RgbaImage, String> {
    let Some((w, h)) = dimensi_header(data) else {
        return Err("format gambar tidak dikenali".into());
    };
    if (w as u64).saturating_mul(h as u64) > BATAS_PX {
        return Err(format!("dimensi {w}x{h} melebihi batas {BATAS_PX} piksel"));
    }
    let img = image::load_from_memory(data).map_err(|e| format!("gagal decode: {e}"))?;
    let (w, h) = img.dimensions();
    let sisi = w.max(h);
    if sisi > maks {
        let skala = maks as f32 / sisi as f32;
        let nw = ((w as f32 * skala) as u32).max(1);
        let nh = ((h as f32 * skala) as u32).max(1);
        return Ok(image::imageops::resize(
            &img.to_rgba8(),
            nw,
            nh,
            FilterType::Triangle,
        ));
    }
    Ok(img.to_rgba8())
}

fn dimensi_header(data: &[u8]) -> Option<(u32, u32)> {
    ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_dimensions().ok())
}

/// Isi kartu: semua argumen render yang sudah diunduh handler.
pub(crate) struct Konten<'a> {
    pub foto_utama: &'a [u8],
    pub avatar: Option<&'a [u8]>,
    pub nama: &'a str,
    pub deskripsi: &'a str,
    pub kata_kata: &'a str,
    pub bg: &'a Bg,
    pub fonts: &'a Fonts,
    pub emoji: Option<&'a HashMap<String, String>>,
}

pub(crate) fn render(k: &Konten) -> Result<Vec<u8>, String> {
    let foto = decode_kecil(k.foto_utama, 1600)?;
    let avatar = match k.avatar {
        Some(a) if !a.is_empty() => decode_kecil(a, 512).ok(),
        _ => None,
    };
    let sumber = sumber_latar(k, &foto)?;
    let mut kanvas = match k.bg {
        Bg::Warna(_) | Bg::Default => sumber.clone(),
        _ => pudar_foto(&sumber),
    };
    let mut cache: HashMap<String, Option<RgbaImage>> = HashMap::new();

    let kx = 34.0f32;
    let ky = 115.0f32;
    let kw = (LEBAR - 68) as f32;
    let header = 140.0f32;
    let natural = (kw * foto.height() as f32 / foto.width() as f32).round();
    let img_h = natural.clamp(600.0, 1040.0);
    let kartu_h = header + img_h;

    let kartu = Bundar::dari(kx, ky, kw, kartu_h, 58.0);
    bayangan(&mut kanvas, kartu, 60.0, 0.35, 20.0);
    kartu.isi(
        &mut kanvas,
        Rgba([WARNA_KARTU[0], WARNA_KARTU[1], WARNA_KARTU[2], 255]),
    );

    let av_r = 46.0f32;
    let av_cx = kx + 36.0 + av_r;
    let av_cy = ky + header / 2.0;
    let av_b = Bundar::dari(av_cx - av_r, av_cy - av_r, av_r * 2.0, av_r * 2.0, av_r);
    tempel_bundar(
        &mut kanvas,
        avatar.as_ref().unwrap_or(&foto),
        (av_cx - av_r) as i64,
        (av_cy - av_r) as i64,
        (av_r * 2.0) as u32,
        (av_r * 2.0) as u32,
        av_b,
    );
    let mut cincin = Vec::new();
    titik_busur(av_cx, av_cy, av_r, 0.0, std::f32::consts::TAU, &mut cincin);
    gores(
        &mut kanvas,
        &cincin,
        4.0,
        Rgba([0x55, 0x55, 0x55, 255]),
        true,
    );

    let teks_x = av_cx + av_r + 24.0;
    let ada_desk = !k.deskripsi.is_empty();
    if let Some(font) = k.fonts.tebal() {
        let gaya = GayaTeks {
            font,
            ukuran: 36.0,
            warna: PUTIH,
            dasar: Dasar::Tengah,
            emoji: Some((34.0, -17.0)),
        };
        let y = if ada_desk { av_cy - 18.0 } else { av_cy };
        gambar_rich(&mut kanvas, k.nama, teks_x, y, &gaya, k.emoji, &mut cache);
    }
    if ada_desk && let Some(font) = k.fonts.teks() {
        let gaya = GayaTeks {
            font,
            ukuran: 29.0,
            warna: Rgba([0xc8, 0xcb, 0xd2, 255]),
            dasar: Dasar::Tengah,
            emoji: Some((28.0, -14.0)),
        };
        gambar_rich(
            &mut kanvas,
            k.deskripsi,
            teks_x,
            av_cy + 22.0,
            &gaya,
            k.emoji,
            &mut cache,
        );
    }

    let img_y = ky + header;
    tempel_bundar(
        &mut kanvas,
        &foto,
        kx as i64,
        img_y as i64,
        kw as u32,
        img_h as u32,
        kartu,
    );

    if !k.kata_kata.is_empty()
        && let Some(font) = k.fonts.display()
    {
        let lebar_maks = kw * 0.48;
        let tinggi_maks = img_h - 80.0;
        let mut ukuran = 64.0f32;
        let mut tinggi_baris = 74.0f32;
        let mut baris: Vec<String> = Vec::new();
        while ukuran >= 24.0 {
            tinggi_baris = (ukuran * 1.18).round();
            baris = wrap_teks(font, k.kata_kata, ukuran, lebar_maks);
            if baris.len() as f32 * tinggi_baris <= tinggi_maks {
                break;
            }
            ukuran -= 4.0;
        }
        let total = baris.len() as f32 * tinggi_baris;
        let mut y = img_y + 48.0f32.max((img_h - total) * 0.45);
        let x = kx + 48.0;
        let gaya = GayaTeks {
            font,
            ukuran,
            warna: Rgba([0x11, 0x11, 0x11, 255]),
            dasar: Dasar::Atas,
            emoji: Some(((ukuran * 0.88).round(), (ukuran * 0.08).round())),
        };
        for satu in &baris {
            gambar_rich(&mut kanvas, satu, x, y, &gaya, k.emoji, &mut cache);
            y += tinggi_baris;
        }
    }

    let menu = Bundar::dari(
        kx + kw - 620.0 - 24.0,
        ky + kartu_h + 40.0,
        620.0,
        420.0,
        50.0,
    );
    bayangan(&mut kanvas, menu, 48.0, 0.25, 12.0);
    kaca_menu(&mut kanvas, &sumber, menu);

    let pad_y = 24.0f32;
    let row_h = (menu.h() - pad_y * 2.0) / MENU.len() as f32;
    for (i, aksi) in MENU.iter().enumerate() {
        let cy = menu.y() + pad_y + row_h * i as f32 + row_h / 2.0;
        gambar_ikon(&mut kanvas, aksi.ikon, menu.x() + 82.0, cy, 48.0);
        if let Some(font) = k.fonts.sedang() {
            let gaya = GayaTeks {
                font,
                ukuran: 42.0,
                warna: PUTIH,
                dasar: Dasar::Tengah,
                emoji: None,
            };
            gambar_teks(&mut kanvas, aksi.label, menu.x() + 158.0, cy + 2.0, &gaya);
        }
        if i + 1 < MENU.len() {
            let garis_y = menu.y() + pad_y + row_h * (i as f32 + 1.0);
            gores(
                &mut kanvas,
                &[
                    (menu.x() + 28.0, garis_y),
                    (menu.x() + menu.w() - 28.0, garis_y),
                ],
                1.5,
                Rgba([255, 255, 255, 36]),
                false,
            );
        }
    }

    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(kanvas)
        .write_to(&mut buf, ImageOutputFormat::Jpeg(92))
        .map_err(|e| format!("gagal encode jpeg: {e}"))?;
    Ok(buf.into_inner())
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

fn ada_gambar(ctx: &Ctx) -> bool {
    let mime_gambar = |mime: Option<&str>| mime.is_some_and(|m| m.contains("image"));
    match ctx.quoted() {
        Some(q) => q.kind.is_image() || mime_gambar(q.mimetype.as_deref()),
        None => {
            ctx.is_media_kind(IncomingKind::Image)
                || ctx
                    .media
                    .as_ref()
                    .is_some_and(|m| mime_gambar(m.mimetype.as_deref()))
        }
    }
}

#[plugin(
    name = "igpreview",
    alias = ["igcard", "postpreview"],
    category = "canvas",
    description = "Membuat kartu pratinjau post Instagram dari foto kamu",
    usage = ".igpreview nama | deskripsi | kata-kata | background (reply/kirim foto)",
    example = ".igpreview Zann | Hari yang cerah | Nikmati setiap momen | blur",
    cooldown = 5,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn igpreview(ctx: Ctx) -> Reply {
    if !ada_gambar(&ctx) {
        return Reply::text(help(&format!("{}{}", ctx.prefix, ctx.command)));
    }

    let parts = split_args(ctx.text.trim());
    let nama = if parts[0].is_empty() {
        let push = ctx.push_name.trim();
        if push.is_empty() {
            "user".to_string()
        } else {
            push.to_string()
        }
    } else {
        parts[0].clone()
    };

    let unduh = if ctx.quoted().is_some() {
        ctx.download_quoted_media()
    } else {
        ctx.download_media()
    };
    let buffer = match unduh {
        Ok(b) if !b.is_empty() => b,
        _ => return Reply::text(GAGAL_UNDUH),
    };

    let fonts = muat_fonts(&ctx);

    let bg = match parse_bg(&parts[3]) {
        Bg::Url(url) => match ctx.http.get(&url) {
            Ok(res) => {
                let data = res.body.bytes().to_vec();
                if data.len() > BATAS_URL_BYTE {
                    Bg::Default
                } else {
                    Bg::Gambar(data)
                }
            }
            Err(_) => Bg::Default,
        },
        lain => lain,
    };

    let avatar = ctx
        .sock()
        .foto_profil(&ctx.sender, "image")
        .ok()
        .flatten()
        .and_then(|url| ctx.http.get(&url).ok())
        .map(|res| res.body.bytes().to_vec())
        .filter(|v| !v.is_empty());

    let emoji = muat_peta_emoji(&ctx);
    let konten = Konten {
        foto_utama: &buffer,
        avatar: avatar.as_deref(),
        nama: &nama,
        deskripsi: &parts[1],
        kata_kata: &parts[2],
        bg: &bg,
        fonts: &fonts,
        emoji: emoji.as_ref(),
    };

    match render(&konten) {
        Ok(jpeg) => Reply::image(
            MediaSource::bytes(jpeg).with_mimetype("image/jpeg"),
            "✨ *Instagram Preview Card*",
        )
        .with_reaction("✅"),
        Err(_) => Reply::text(GAGAL_RENDER).with_reaction("❌"),
    }
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpError, HttpGet, HttpResponse, MediaError, MediaFetch, MediaOrigin,
    };
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";
    const ARIAL: &[u8] = include_bytes!("../../../assets/fonts/arialnarrow.ttf");

    fn png_kecil(lebar: u32, tinggi: u32) -> Vec<u8> {
        let img = image::RgbImage::new(lebar, tinggi);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageOutputFormat::Png).unwrap();
        buf.into_inner()
    }

    fn decode_dim(data: &[u8]) -> (u32, u32) {
        image::load_from_memory(data).unwrap().dimensions()
    }

    fn fonts_uji() -> Fonts {
        let f = FontArc::try_from_vec(ARIAL.to_vec()).ok();
        Fonts {
            anton: f.clone(),
            regular: f.clone(),
            medium: f.clone(),
            semibold: f.clone(),
            bold: f,
        }
    }

    fn konten<'a>(
        foto: &'a [u8],
        bg: &'a Bg,
        fonts: &'a Fonts,
        nama: &'a str,
        desk: &'a str,
        kata: &'a str,
    ) -> Konten<'a> {
        Konten {
            foto_utama: foto,
            avatar: None,
            nama,
            deskripsi: desk,
            kata_kata: kata,
            bg,
            fonts,
            emoji: None,
        }
    }

    #[test]
    fn meta_terdaftar_dengan_alias() {
        let p = oktz_plugin_api::find("igpreview").unwrap();
        assert_eq!(p.meta.name, "igpreview");
        assert_eq!(p.meta.alias, ["igcard", "postpreview"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group);
        assert!(!p.meta.is_private);
        for a in ["igcard", "postpreview"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "igpreview");
        }
    }

    #[test]
    fn parsing_empat_bagian() {
        let args = split_args("Zann | Hari cerah | Nikmati momen | blur");
        assert_eq!(args[0], "Zann");
        assert_eq!(args[1], "Hari cerah");
        assert_eq!(args[2], "Nikmati momen");
        assert_eq!(args[3], "blur");
    }

    #[test]
    fn parse_bg_semua_jalur() {
        assert_eq!(parse_bg(""), Bg::Default);
        assert_eq!(parse_bg("  "), Bg::Default);
        assert_eq!(parse_bg("blur"), Bg::Blur);
        assert_eq!(parse_bg("BLUR"), Bg::Blur);
        assert_eq!(parse_bg("#ff0099"), Bg::Warna([0xff, 0x00, 0x99]));
        assert_eq!(parse_bg("#abc"), Bg::Warna([0xaa, 0xbb, 0xcc]));
        assert_eq!(
            parse_bg("https://x/a.jpg"),
            Bg::Url("https://x/a.jpg".into())
        );
        assert!(matches!(parse_bg("http://x/a.jpg"), Bg::Url(_)));
        assert_eq!(parse_bg("#zz1122"), Bg::Default);
        assert_eq!(parse_bg("warna ngawur"), Bg::Default);
    }

    #[test]
    fn grafem_menggabung_emoji_zwj_dan_flag() {
        assert_eq!(grafem("ab"), vec!["a", "b"]);
        assert_eq!(grafem("👨‍👩‍👧"), vec!["👨‍👩‍👧"]);
        assert_eq!(grafem("👍🏽x"), vec!["👍🏽", "x"]);
        assert_eq!(grafem("🇮🇩🇺🇸"), vec!["🇮🇩", "🇺🇸"]);
        assert_eq!(grafem("❤️"), vec!["❤️"]);
        assert_eq!(grafem("1️⃣"), vec!["1️⃣"]);
    }

    #[test]
    fn base64_decode_benar() {
        assert_eq!(decode_base64("aGVsbG8="), b"hello");
        assert_eq!(decode_base64("aGVsbG8"), b"hello");
        assert_eq!(decode_base64(""), Vec::<u8>::new());
    }

    #[test]
    fn render_murni_jpeg_magic_dan_dimensi() {
        let foto = png_kecil(200, 150);
        let fonts = Fonts::default();
        let bg = Bg::Warna([10, 20, 30]);
        let jpeg = render(&konten(
            &foto,
            &bg,
            &fonts,
            "Zann",
            "deskripsi",
            "kata kata",
        ))
        .unwrap();
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
        assert_eq!(decode_dim(&jpeg), (LEBAR, TINGGI));
    }

    #[test]
    fn render_dengan_font_dan_kata_panjang_tidak_panik() {
        let foto = png_kecil(300, 400);
        let fonts = fonts_uji();
        let bg = Bg::Warna([0x1e, 0x29, 0x3b]);
        let panjang = "kata panjang ".repeat(30);
        assert!(render(&konten(&foto, &bg, &fonts, "Nama", "Deskripsi", &panjang)).is_ok());
    }

    #[test]
    fn render_bg_gambar_dan_blur() {
        let foto = png_kecil(60, 60);
        let fonts = Fonts::default();
        let bgimg = Bg::Gambar(png_kecil(300, 500));
        assert!(render(&konten(&foto, &bgimg, &fonts, "a", "b", "")).is_ok());
        assert!(render(&konten(&foto, &Bg::Blur, &fonts, "a", "b", "")).is_ok());
    }

    #[test]
    fn render_kolase_default() {
        let foto = png_kecil(120, 90);
        let fonts = fonts_uji();
        assert!(render(&konten(&foto, &Bg::Default, &fonts, "a", "b", "kata")).is_ok());
    }

    #[test]
    fn render_foto_raksasa_ditolak_sebelum_decode() {
        let d = vec![
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x9C, 0x40, 0x9C, 0x40, 0x01, 0x01, 0x11,
            0x00,
        ];
        let fonts = Fonts::default();
        assert!(render(&konten(&d, &Bg::Default, &fonts, "a", "b", "")).is_err());
        assert!(decode_kecil(&d, LEBAR).is_err());
    }

    #[test]
    fn render_byte_sampah_err_bukan_panic() {
        let fonts = Fonts::default();
        let bg = Bg::Warna([1, 2, 3]);
        assert!(render(&konten(b"bukan gambar", &bg, &fonts, "a", "b", "")).is_err());
    }

    #[test]
    fn wrap_teks_potong_di_batas_lebar() {
        let font = FontArc::try_from_vec(ARIAL.to_vec()).unwrap();
        let baris = wrap_teks(&font, "kata panjang sekali untuk diuji", 40.0, 150.0);
        assert!(baris.len() > 1);
        for b in &baris {
            assert!(lebar_teks(&font, b, 40.0) <= 150.0 || !b.contains(' '));
        }
        let kosong = wrap_teks(&font, "a\n\nb", 40.0, 150.0);
        assert_eq!(kosong, vec!["a", "", "b"]);
    }

    // --- handler ---

    #[derive(Debug, Default)]
    struct Stub {
        jawaban: Mutex<Vec<(String, Vec<u8>)>>,
        gagal: bool,
    }

    impl Stub {
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                gagal: true,
                ..Default::default()
            })
        }
        fn jawab(&self, url: &str) -> Option<Vec<u8>> {
            self.jawaban
                .lock()
                .unwrap()
                .iter()
                .find(|(u, _)| u == url)
                .map(|(_, b)| b.clone())
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            if self.gagal {
                return Err(HttpError {
                    message: "offline".into(),
                });
            }
            match self.jawab(url) {
                Some(b) => Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(b),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(HttpError {
                    message: format!("tidak ada stub: {url}"),
                }),
            }
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma GET")
        }
    }

    #[derive(Debug)]
    struct StubUnduh;

    impl MediaFetch for StubUnduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            Ok(png_kecil(80, 80))
        }
    }

    #[derive(Debug)]
    struct UnduhGagal;

    impl MediaFetch for UnduhGagal {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            Err(MediaError {
                message: "offline".into(),
            })
        }
    }

    fn ctx_of(text: &str, is_image: bool) -> Ctx {
        Ctx {
            text: text.to_string(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "igpreview".into(),
            media: is_image.then(|| oktz_plugin_api::IncomingMedia {
                kind: IncomingKind::Image,
                mimetype: Some("image/jpeg".into()),
                file_name: None,
                caption: String::new(),
            }),
            ..Ctx::default()
        }
    }

    fn run(text: &str, is_image: bool) -> Reply {
        let mut ctx = ctx_of(text, is_image);
        ctx.http = Stub::offline() as Arc<dyn HttpGet>;
        ctx.media_fetch = Arc::new(StubUnduh);
        igpreview(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn tanpa_gambar_menghasilkan_kartu_pakai() {
        let r = run("Zann | desk | kata |", false);
        assert_eq!(text_of(&r), help(".igpreview"));
        assert_eq!(reaction_of(&r), None);
    }

    #[test]
    fn nama_kosong_pakai_push_name_lalu_user() {
        let mut ctx = ctx_of("| desk | kata | #123456", true);
        ctx.http = Stub::offline() as Arc<dyn HttpGet>;
        ctx.media_fetch = Arc::new(StubUnduh);
        ctx.push_name = "Budi".into();
        let r = igpreview(ctx);
        assert_eq!(reaction_of(&r), Some("✅".into()));

        let mut ctx = ctx_of("| desk | kata | #123456", true);
        ctx.http = Stub::offline() as Arc<dyn HttpGet>;
        ctx.media_fetch = Arc::new(StubUnduh);
        let r = igpreview(ctx);
        assert_eq!(reaction_of(&r), Some("✅".into()));
    }

    #[test]
    fn gagal_unduh_media_mengirim_pesan_tanpa_reaksi() {
        let mut ctx = ctx_of("Zann | desk | kata |", true);
        ctx.http = Stub::offline() as Arc<dyn HttpGet>;
        ctx.media_fetch = Arc::new(UnduhGagal);
        let r = igpreview(ctx);
        assert_eq!(text_of(&r), GAGAL_UNDUH);
        assert_eq!(reaction_of(&r), None);
    }

    #[test]
    fn jalur_berhasil_mengirim_jpeg_dan_reaksi_centang() {
        let r = run("Zann | deskripsi | kata kata | #101010", true);
        assert_eq!(reaction_of(&r), Some("✅".to_string()));
        let Reply::Reaction { reply: isi, .. } = r.without_mutations() else {
            panic!("harus dibungkus reaksi");
        };
        match isi.as_ref() {
            Reply::Image { media, caption, .. } => {
                assert_eq!(caption, "✨ *Instagram Preview Card*");
                let Some(MediaSource::Bytes { data, mimetype }) = media else {
                    panic!("harus bytes");
                };
                assert_eq!(&data[..2], &[0xFF, 0xD8], "harus jpeg");
                assert_eq!(mimetype.as_deref(), Some("image/jpeg"));
                assert_eq!(decode_dim(data), (LEBAR, TINGGI));
            }
            lain => panic!("harus image, dapat {lain:?}"),
        }
    }

    #[test]
    fn bg_url_gagal_unduh_fallback_default_tanpa_panic() {
        let r = run("Zann | desk | kata | https://x/bg.jpg", true);
        assert_eq!(reaction_of(&r), Some("✅".to_string()));
    }
}
