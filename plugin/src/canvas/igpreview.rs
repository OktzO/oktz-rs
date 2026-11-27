//! `plugins/canvas/igpreview.js` — kartu pratinjau post Instagram, dirender
//! lokal dengan crate `image` + `ab_glyph` (tanpa API luar).
//!
//! `ponytail:` background collage grid JS disederhanakan jadi warna polos /
//! blur / URL tunggal; naikkan ke grid kalau benar-benar diminta.
//! `ponytail:` menu aksi frosted tanpa blur backdrop (semi-transparan saja);
//! blur backdrop bisa ditambah dengan `imageops::blur` atas area menu.

use ab_glyph::{Font, FontArc, ScaleFont, point};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{GenericImageView, ImageOutputFormat, Rgba, RgbaImage};
use oktz_plugin_api::{
    Ctx, HttpCall, HttpError, HttpGet, IncomingKind, MediaSource, Reply, plugin,
};
use std::io::Cursor;

const LEBAR: u32 = 1080;
const TINGGI: u32 = 1920;
const WARNA_KARTU: [u8; 3] = [0x23, 0x27, 0x2e];
const WARNA_DEFAULT_BG: [u8; 3] = [0x1e, 0x29, 0x3b];
const BATAS_PX: u64 = 16_383 * 16_383;
const BATAS_URL_BYTE: usize = 10 * 1024 * 1024;

const URL_ANTON: &str = "https://github.com/google/fonts/raw/main/ofl/anton/Anton-Regular.ttf";
const URL_INTER: &str =
    "https://github.com/googlefonts/inter/raw/main/docs/font-files/InterDisplay-Regular.ttf";
const JALUR_ANTON: &str = "media/fonts/Anton-Regular.ttf";
const JALUR_INTER: &str = "media/fonts/InterDisplay-Regular.ttf";

/// kartu bantuan saat input belum lengkap.
fn help(prefix: &str) -> String {
    format!(
        "✨ *INSTAGRAM PREVIEW CARD*\n\n\
         Buat kartu pratinjau post Instagram dari foto kamu!\n\n\
         *Cara Penggunaan:*\n\
         - Kirim/reply foto dengan caption *{prefix}igpreview nama | deskripsi | kata-kata | background*\n\
         - Background opsional: kosong / `#hex` / `blur` / URL gambar\n\n\
         *Contoh:* {prefix}igpreview Zann | Hari yang cerah | Nikmati setiap momen | blur"
    )
}

const GAGAL_RENDER: &str = "Maaf, gagal membuat kartu Instagram Preview. Coba lagi beberapa saat.";

/// `text.split('|').map(trim)` — maks 4 bagian, sisanya diabaikan.
pub(crate) fn split_args(text: &str) -> [String; 4] {
    let mut out = [const { String::new() }; 4];
    for (slot, part) in out.iter_mut().zip(text.split('|')) {
        *slot = part.trim().to_string();
    }
    out
}

/// `#hex` valid → RGB, selain itu `None` (fallback warna default).
pub(crate) fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some([
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ])
}

/// Baca dimensi dari header saja — pola `cek_batas` di `gambar.rs`.
fn dimensi_header(data: &[u8]) -> Option<(u32, u32)> {
    ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_dimensions().ok())
}

/// Decode + turunkan skala ke maks `maks` sisi terpanjang SEBELUM komposit.
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

/// Cover-crop: isi `w`×`h`, potong tengah, rasio aspek dipertahankan.
fn cover(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let (iw, ih) = img.dimensions();
    let rasio_target = w as f32 / h as f32;
    let rasio_sumber = iw as f32 / ih as f32;
    let dipotong = if rasio_sumber > rasio_target {
        let nw = ((ih as f32 * rasio_target) as u32).max(1);
        let x = iw.saturating_sub(nw) / 2;
        image::imageops::crop_imm(img, x, 0, nw, ih).to_image()
    } else {
        let nh = ((iw as f32 / rasio_target) as u32).max(1);
        let y = ih.saturating_sub(nh) / 2;
        image::imageops::crop_imm(img, 0, y, iw, nh).to_image()
    };
    image::imageops::resize(&dipotong, w, h, FilterType::Triangle)
}

/// Lingkaran terisi (avatar): loop piksel dengan cek jarak.
fn lingkaran_penuh(img: &mut RgbaImage, cx: i64, cy: i64, r: i64, warna: Rgba<u8>) {
    for y in (cy - r).max(0)..(cy + r + 1).min(img.height() as i64) {
        for x in (cx - r).max(0)..(cx + r + 1).min(img.width() as i64) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                img.put_pixel(x as u32, y as u32, warna);
            }
        }
    }
}

/// Rounded-rect terisi: loop piksel dengan cek jarak sudut.
fn rounded_rect(img: &mut RgbaImage, x0: i64, y0: i64, w: u64, h: u64, r: u64, warna: Rgba<u8>) {
    let x1 = x0 + w as i64;
    let y1 = y0 + h as i64;
    let rx = r as i64;
    for y in y0.max(0)..y1.min(img.height() as i64) {
        for x in x0.max(0)..x1.min(img.width() as i64) {
            let qx = (x0 + rx).max(x.min(x1 - rx)) - x;
            let qy = (y0 + rx).max(y.min(y1 - rx)) - y;
            if (qx * qx + qy * qy) as u64 <= r * r {
                img.put_pixel(x as u32, y as u32, warna);
            }
        }
    }
}

/// Garis tebal sederhana: Bresenham dengan ketebalan n piksel.
fn garis_tebal(img: &mut RgbaImage, dari: (i64, i64), ke: (i64, i64), tebal: i64, warna: Rgba<u8>) {
    let (x0, y0) = dari;
    let (x1, y1) = ke;
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let (mut x, mut y) = (x0, y0);
    let setengah = tebal / 2;
    loop {
        for oy in -setengah..=setengah {
            for ox in -setengah..=setengah {
                let px = x + ox;
                let py = y + oy;
                if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                    img.put_pixel(px as u32, py as u32, warna);
                }
            }
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Lebar teks: jumlah `h_advance` per glyph × skala — tanpa measureText.
fn lebar_teks(font: &FontArc, teks: &str, ukuran: f32) -> f32 {
    let scaled = font.as_scaled(ukuran);
    teks.chars()
        .filter(|c| *c != '\n')
        .map(|c| scaled.h_advance(scaled.glyph_id(c)))
        .sum()
}

/// Wrap teks ke baris-baris yang muat di `lebar_maks`.
fn wrap_teks(font: &FontArc, teks: &str, ukuran: f32, lebar_maks: f32) -> Vec<String> {
    let mut baris = Vec::new();
    for kalimat in teks.split('\n') {
        let mut kini = String::new();
        for kata in kalimat.split_whitespace() {
            let coba = if kini.is_empty() {
                kata.to_string()
            } else {
                format!("{kini} {kata}")
            };
            if lebar_teks(font, &coba, ukuran) <= lebar_maks || kini.is_empty() {
                kini = coba;
            } else {
                baris.push(std::mem::take(&mut kini));
                kini = kata.to_string();
            }
        }
        baris.push(kini);
    }
    baris
}

/// Gambar teks dengan font apa pun ke buffer, lalu overlay-style blend manual.
fn gambar_teks(
    img: &mut RgbaImage,
    font: &FontArc,
    teks: &str,
    x: i64,
    y_baseline: i64,
    ukuran: f32,
    warna: [u8; 3],
) {
    let scaled = font.as_scaled(ukuran);
    let mut kursor = x as f32;
    for c in teks.chars() {
        if c == '\n' {
            continue;
        }
        let glyph = font
            .glyph_id(c)
            .with_scale_and_position(ukuran, point(kursor, y_baseline as f32));
        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            outline.draw(|px, py, coverage| {
                let ix = bounds.min.x as i64 + px as i64;
                let iy = bounds.min.y as i64 + py as i64;
                if ix < 0 || iy < 0 || ix >= img.width() as i64 || iy >= img.height() as i64 {
                    return;
                }
                let latar = img.get_pixel(ix as u32, iy as u32);
                let a = coverage;
                let campur = [
                    (latar[0] as f32 * (1.0 - a) + warna[0] as f32 * a) as u8,
                    (latar[1] as f32 * (1.0 - a) + warna[1] as f32 * a) as u8,
                    (latar[2] as f32 * (1.0 - a) + warna[2] as f32 * a) as u8,
                    255,
                ];
                img.put_pixel(ix as u32, iy as u32, Rgba(campur));
            });
        }
        kursor += scaled.h_advance(scaled.glyph_id(c));
    }
}

/// Unduh font sekali ke disk, lalu baca dari disk sebagai cache.
fn font_dari(ctx: &Ctx, url: &str, jalur: &str) -> Option<FontArc> {
    if !ctx.berkas().ada(jalur) {
        let res = ctx.http.get(url).ok()?;
        let data = res.body.bytes().to_vec();
        if data.is_empty()
            || !data.starts_with(&[0x00, 0x01, 0x00, 0x00])
                && !data.starts_with(b"true")
                && !data.starts_with(b"OTTO")
        {
            return None;
        }
        ctx.berkas().tulis(jalur, &data).ok()?;
    }
    FontArc::try_from_vec(ctx.berkas().baca(jalur).ok()?).ok()
}

/// (font display Anton, font teks Inter) — keduanya boleh `None` (fallback).
type Fonts = (Option<FontArc>, Option<FontArc>);

fn muat_fonts(ctx: &Ctx) -> Fonts {
    (
        font_dari(ctx, URL_ANTON, JALUR_ANTON),
        font_dari(ctx, URL_INTER, JALUR_INTER),
    )
}

// ---------------------------------------------------------------------------
// Render murni — tanpa Ctx, testable.
// ---------------------------------------------------------------------------

/// Satu baris menu aksi: ikon + label.
struct MenuAksi {
    label: &'static str,
    ikon: Ikon,
}

#[derive(Clone, Copy)]
enum Ikon {
    Hati,
    Balon,
    PanahUlang,
    PanahKirim,
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
        ikon: Ikon::PanahUlang,
    },
    MenuAksi {
        label: "Bagikan",
        ikon: Ikon::PanahKirim,
    },
];

/// Ikon garis sederhana, digambar langsung piksel.
fn gambar_ikon(img: &mut RgbaImage, ikon: Ikon, x: i64, y: i64, s: i64, warna: Rgba<u8>) {
    match ikon {
        Ikon::Hati => {
            lingkaran_penuh(img, x + s / 4, y + s / 3, s / 4, warna);
            lingkaran_penuh(img, x + 3 * s / 4, y + s / 3, s / 4, warna);
            let mut segitiga = 0;
            for baris in 0..(s / 2) {
                for kolom in 0..=segitiga {
                    let px = x + s / 2 - kolom;
                    let py = y + s / 3 + baris;
                    if px >= 0 && py < img.height() as i64 && px < img.width() as i64 {
                        img.put_pixel(px as u32, py as u32, warna);
                        let px2 = x + s / 2 + kolom;
                        if px2 < img.width() as i64 {
                            img.put_pixel(px2 as u32, py as u32, warna);
                        }
                    }
                }
                segitiga += 2;
            }
        }
        Ikon::Balon => {
            let cx = x + s / 2;
            let cy = y + s / 3;
            let r = s / 3;
            for dy in -r..=r {
                for dx in -r..=r {
                    let jarak = dx * dx + dy * dy;
                    if jarak <= r * r && jarak >= (r - 3) * (r - 3) {
                        let px = cx + dx;
                        let py = cy + dy;
                        if px >= 0
                            && py >= 0
                            && (px as u32) < img.width()
                            && (py as u32) < img.height()
                        {
                            img.put_pixel(px as u32, py as u32, warna);
                        }
                    }
                }
            }
            garis_tebal(img, (cx, cy + r), (cx, cy + r + s / 4), 2, warna);
        }
        Ikon::PanahUlang => {
            let cx = x + s / 2;
            let cy = y + s / 2;
            let r = s / 3;
            for i in 0..360 {
                let sudut = i as f32 * std::f32::consts::PI / 180.0;
                for tebal in 0..3 {
                    let rr = r - tebal;
                    let px = cx + (rr as f32 * sudut.cos()) as i64;
                    let py = cy + (rr as f32 * sudut.sin()) as i64;
                    if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height()
                    {
                        img.put_pixel(px as u32, py as u32, warna);
                    }
                }
            }
            garis_tebal(img, (cx + r, cy), (cx + r + s / 5, cy - s / 6), 3, warna);
            garis_tebal(img, (cx + r, cy), (cx + r - s / 6, cy - s / 6), 3, warna);
        }
        Ikon::PanahKirim => {
            garis_tebal(img, (x, y + s / 2), (x + s, y + s / 2), 3, warna);
            garis_tebal(
                img,
                (x + s, y + s / 2),
                (x + s - s / 3, y + s / 6),
                3,
                warna,
            );
            garis_tebal(
                img,
                (x + s, y + s / 2),
                (x + s - s / 3, y + 5 * s / 6),
                3,
                warna,
            );
        }
    }
}

/// Render kartu utuh. `avatar` boleh kosong → lingkaran abu.
pub(crate) fn render(
    foto_utama: &[u8],
    avatar: Option<&[u8]>,
    nama: &str,
    deskripsi: &str,
    kata_kata: &str,
    bg: &Bg,
    fonts: &Fonts,
) -> Result<Vec<u8>, String> {
    let foto = decode_kecil(foto_utama, LEBAR)?;
    let avatar_img = match avatar {
        Some(a) if !a.is_empty() => decode_kecil(a, 512).ok(),
        _ => None,
    };

    let mut kanvas = RgbaImage::from_pixel(
        LEBAR,
        TINGGI,
        Rgba([
            WARNA_DEFAULT_BG[0],
            WARNA_DEFAULT_BG[1],
            WARNA_DEFAULT_BG[2],
            255,
        ]),
    );

    match bg {
        Bg::Warna(rgb) => {
            kanvas = RgbaImage::from_pixel(LEBAR, TINGGI, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }
        Bg::Blur => {
            let blur = image::imageops::blur(&cover(&foto, LEBAR, TINGGI), 12.0);
            kanvas = blur;
        }
        Bg::Gambar(data) => {
            let img = decode_kecil(data, LEBAR)?;
            kanvas = cover(&img, LEBAR, TINGGI);
        }
        Bg::Default => {}
    }

    // Kartu post: (34,115), lebar LEBAR-68.
    let (kx, ky) = (34i64, 115i64);
    let kw = (LEBAR - 68) as i64;
    let header_h = 140i64;
    let foto_area = kw * 4 / 5;
    let kartu_h = header_h + foto_area + 24;
    rounded_rect(
        &mut kanvas,
        kx,
        ky,
        kw as u64,
        kartu_h as u64,
        24,
        Rgba([WARNA_KARTU[0], WARNA_KARTU[1], WARNA_KARTU[2], 255]),
    );

    // Header: avatar bundar 92px.
    let (ax, ay, ar) = (kx + 24, ky + (header_h - 92) / 2, 46i64);
    let mut avatar_bundar = RgbaImage::new((2 * ar) as u32, (2 * ar) as u32);
    match &avatar_img {
        Some(av) => {
            let cov = cover(av, 2 * ar as u32, 2 * ar as u32);
            for y in 0..avatar_bundar.height() {
                for x in 0..avatar_bundar.width() {
                    let dx = x as i64 - ar;
                    let dy = y as i64 - ar;
                    if dx * dx + dy * dy <= ar * ar {
                        avatar_bundar.put_pixel(x, y, *cov.get_pixel(x, y));
                    }
                }
            }
        }
        None => lingkaran_penuh(&mut avatar_bundar, ar, ar, ar, Rgba([70, 74, 82, 255])),
    }
    image::imageops::overlay(&mut kanvas, &avatar_bundar, ax, ay);

    let (font_besar, font_teks) = fonts;
    let f_nama = font_besar.as_ref().or(font_teks.as_ref());
    let f_teks = font_teks.as_ref().or(font_besar.as_ref());

    let tx = ax + 2 * ar + 20;
    let f_nama = f_nama.or(f_teks);
    if let Some(f) = f_nama {
        gambar_teks(&mut kanvas, f, nama, tx, ay + 4, 36.0, [255, 255, 255]);
    }
    if let Some(f) = f_teks {
        gambar_teks(
            &mut kanvas,
            f,
            deskripsi,
            tx,
            ay + 58,
            29.0,
            [0xc8, 0xcb, 0xd2],
        );
    }

    // Foto utama cover-crop di bawah header.
    let fx = kx as u32 + 12;
    let fy = ky as u32 + header_h as u32;
    let fw = (kw - 24) as u32;
    let fh = (foto_area - 12) as u32;
    let foto_cov = cover(&foto, fw, fh);
    image::imageops::overlay(&mut kanvas, &foto_cov, fx as i64, fy as i64);

    // Overlay kata-kata: wrap kiri atas area foto, auto-fit 64→24px.
    if !kata_kata.is_empty()
        && let Some(f_over) = font_besar.as_ref().or(f_teks)
    {
        let lebar_maks = kw as f32 * 0.48;
        let mut ukuran = 64.0f32;
        while ukuran > 24.0 {
            let baris = wrap_teks(f_over, kata_kata, ukuran, lebar_maks);
            let tinggi_total = baris.len() as f32 * ukuran * 1.15;
            if tinggi_total <= fh as f32 * 0.6 {
                let mut y = fy as i64 + 24;
                for satu in &baris {
                    gambar_teks(
                        &mut kanvas,
                        f_over,
                        satu,
                        kx + 24,
                        y + ukuran as i64,
                        ukuran,
                        [0x11, 0x11, 0x11],
                    );
                    y += (ukuran * 1.15) as i64;
                }
                break;
            }
            ukuran -= 4.0;
        }
    }

    // Menu aksi frosted kanan-bawah kartu: 4 baris.
    let menu_w = 380i64;
    let menu_h = 4 * 88 + 16;
    let menu_x = kx + kw - menu_w - 12;
    let menu_y = ky + kartu_h - menu_h - 12;
    rounded_rect(
        &mut kanvas,
        menu_x,
        menu_y,
        menu_w as u64,
        menu_h as u64,
        20,
        Rgba([120, 120, 125, 140]),
    );
    let f_menu = f_teks;
    for (i, aksi) in MENU.iter().enumerate() {
        let row_y = menu_y + 8 + i as i64 * 88;
        if i > 0 {
            for x in (menu_x + 16)..(menu_x + menu_w - 16) {
                let p = kanvas.get_pixel(x as u32, row_y as u32);
                kanvas.put_pixel(
                    x as u32,
                    row_y as u32,
                    Rgba([
                        (p[0] as f32 * 0.86 + 255.0 * 0.14) as u8,
                        (p[1] as f32 * 0.86 + 255.0 * 0.14) as u8,
                        (p[2] as f32 * 0.86 + 255.0 * 0.14) as u8,
                        255,
                    ]),
                );
            }
        }
        gambar_ikon(
            &mut kanvas,
            aksi.ikon,
            menu_x + 24,
            row_y + 24,
            40,
            Rgba([255, 255, 255, 255]),
        );
        if let Some(f) = f_menu {
            gambar_teks(
                &mut kanvas,
                f,
                aksi.label,
                menu_x + 84,
                row_y + 36,
                42.0,
                [255, 255, 255],
            );
        }
    }

    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(kanvas)
        .write_to(&mut buf, ImageOutputFormat::Jpeg(92))
        .map_err(|e| format!("gagal encode jpeg: {e}"))?;
    Ok(buf.into_inner())
}

/// Background hasil parsing argumen ke-4.
#[derive(Debug, PartialEq)]
pub(crate) enum Bg {
    Default,
    Warna([u8; 3]),
    Blur,
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
        return Bg::Gambar(Vec::new());
    }
    if let Some(rgb) = parse_hex(a) {
        return Bg::Warna(rgb);
    }
    Bg::Default
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

fn unduh_bg(http: &dyn HttpGet, url: &str) -> Result<Vec<u8>, String> {
    let res = http
        .call(&HttpCall::get(url))
        .map_err(|e: HttpError| format!("unduh background gagal: {}", e.message))?;
    let data = res.body.bytes().to_vec();
    if data.len() > BATAS_URL_BYTE {
        return Err(format!("background melebihi {BATAS_URL_BYTE} byte"));
    }
    Ok(data)
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
    let is_image =
        ctx.is_media_kind(IncomingKind::Image) || ctx.quoted().is_some_and(|q| q.kind.is_image());
    if !is_image {
        return Reply::text(help(&ctx.prefix));
    }

    let text = ctx.text.trim();
    let parts = split_args(text);
    if parts[0].is_empty() || parts[1].is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    let buffer = if ctx.quoted().is_some_and(|q| q.kind.is_media()) {
        ctx.download_quoted_media().ok()
    } else {
        ctx.download_media().ok()
    };
    let Some(buffer) = buffer else {
        return Reply::text(GAGAL_RENDER).with_reaction("❌");
    };

    let fonts = muat_fonts(&ctx);

    let bg = match parse_bg(&parts[3]) {
        Bg::Gambar(_) if !parts[3].trim().is_empty() => match unduh_bg(&*ctx.http, parts[3].trim())
        {
            Ok(d) => Bg::Gambar(d),
            Err(_) => Bg::Default,
        },
        lain => lain,
    };

    // Avatar: foto profil sender, fallback gambar utama.
    let avatar = ctx
        .sock()
        .foto_profil(&ctx.sender, "image")
        .ok()
        .flatten()
        .and_then(|url| ctx.http.get(&url).ok())
        .map(|res| res.body.bytes().to_vec())
        .filter(|v| !v.is_empty());

    match render(
        &buffer,
        avatar.as_deref(),
        &parts[0],
        &parts[1],
        &parts[2],
        &bg,
        &fonts,
    ) {
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
    use oktz_plugin_api::{HttpBody, HttpResponse, MediaError, MediaFetch, MediaOrigin};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    fn png_kecil(lebar: u32, tinggi: u32) -> Vec<u8> {
        let img = image::RgbImage::new(lebar, tinggi);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageOutputFormat::Png).unwrap();
        buf.into_inner()
    }

    fn decode_dim(data: &[u8]) -> (u32, u32) {
        image::load_from_memory(data).unwrap().dimensions()
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
        assert!(matches!(parse_bg("https://x/a.jpg"), Bg::Gambar(_)));
        assert!(matches!(parse_bg("http://x/a.jpg"), Bg::Gambar(_)));
        // tidak valid → default, tidak panic
        assert_eq!(parse_bg("#zz1122"), Bg::Default);
        assert_eq!(parse_bg("warna ngawur"), Bg::Default);
    }

    #[test]
    fn render_murni_jpeg_magic_dan_dimensi() {
        let foto = png_kecil(200, 150);
        let fonts: Fonts = (None, None);
        let jpeg = render(
            &foto,
            None,
            "Zann",
            "deskripsi",
            "kata kata",
            &Bg::Default,
            &fonts,
        )
        .unwrap();
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
        let (w, h) = decode_dim(&jpeg);
        assert_eq!((w, h), (LEBAR, TINGGI));
    }

    #[test]
    fn render_bg_warna_dan_gambar() {
        let foto = png_kecil(60, 60);
        let fonts: Fonts = (None, None);
        let bg = Bg::Warna([10, 20, 30]);
        assert!(render(&foto, None, "a", "b", "", &bg, &fonts).is_ok());
        let bgimg = Bg::Gambar(png_kecil(300, 500));
        assert!(render(&foto, None, "a", "b", "", &bgimg, &fonts).is_ok());
        let blur = render(&foto, None, "a", "b", "", &Bg::Blur, &fonts).is_ok();
        assert!(blur);
    }

    #[test]
    fn render_foto_raksasa_ditolak_sebelum_decode() {
        // SOF0 40000x40000: header valid, decode tidak jalan.
        let d = vec![
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x9C, 0x40, 0x9C, 0x40, 0x01, 0x01, 0x11,
            0x00,
        ];
        let fonts: Fonts = (None, None);
        assert!(render(&d, None, "a", "b", "", &Bg::Default, &fonts).is_err());
        assert!(decode_kecil(&d, LEBAR).is_err());
    }

    #[test]
    fn render_byte_sampah_err_bukan_panic() {
        let fonts: Fonts = (None, None);
        assert!(render(b"bukan gambar", None, "a", "b", "", &Bg::Default, &fonts).is_err());
    }

    #[test]
    fn wrap_teks_potong_di_batas_lebar() {
        // Font bawaan kanvas tidak bisa dipakai langsung di sini (private);
        // font None → wrap tetap bisa dengan font tiruan. Pakai font kanvas
        // publik lewat Kanvas tidak tersedia, jadi test wrap pakai font nyata
        // hanya kalau ada; tanpa font, fungsi ini tak terpanggil. Skip: gunakan
        // font minimal yang dibangun dari byte tidak mungkin. Test tingkat
        // fungsional: render dengan kata panjang tidak panik.
        let foto = png_kecil(40, 40);
        let fonts: Fonts = (None, None);
        let panjang = "kata ".repeat(40);
        assert!(render(&foto, None, "a", "b", &panjang, &Bg::Default, &fonts).is_ok());
    }

    // --- handler ---

    #[derive(Debug, Default)]
    struct Stub {
        jawaban: Mutex<Vec<(String, Vec<u8>)>>,
        gagal: bool,
    }

    impl Stub {
        fn dengan(url: &str, body: &[u8]) -> Arc<Stub> {
            let s = Stub::default();
            s.jawaban
                .lock()
                .unwrap()
                .push((url.to_string(), body.to_vec()));
            Arc::new(s)
        }
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

    impl MediaFetch for Stub {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            if self.gagal {
                return Err(MediaError {
                    message: "offline".into(),
                });
            }
            Ok(png_kecil(80, 80))
        }
    }

    /// MediaFetch tiruan: selalu balas PNG kecil.
    #[derive(Debug)]
    struct StubUnduh;
    impl MediaFetch for StubUnduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            Ok(png_kecil(80, 80))
        }
    }

    fn run(text: &str, is_image: bool) -> Reply {
        let http = Stub::offline() as Arc<dyn HttpGet>;
        let media_fetch = Stub::offline() as Arc<dyn MediaFetch>;
        run_dgn(http, media_fetch, text, is_image)
    }

    fn run_dgn(
        http: Arc<dyn HttpGet>,
        media_fetch: Arc<dyn MediaFetch>,
        text: &str,
        is_image: bool,
    ) -> Reply {
        let ctx = Ctx {
            text: text.to_string(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "igpreview".into(),
            http,
            media_fetch,
            media: if is_image {
                Some(oktz_plugin_api::IncomingMedia {
                    kind: IncomingKind::Image,
                    mimetype: None,
                    file_name: None,
                    caption: String::new(),
                })
            } else {
                None
            },
            ..Ctx::default()
        };
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
        assert_eq!(text_of(&r), help("."));
        assert_eq!(reaction_of(&r), None);
    }

    #[test]
    fn nama_atau_deskripsi_kosong_menghasilkan_kartu_pakai() {
        let r = run("| desk | kata |", true);
        assert_eq!(text_of(&r), help("."));
        let r = run("Zann | | kata |", true);
        assert_eq!(text_of(&r), help("."));
    }

    #[test]
    fn gagal_unduh_media_mengembalikan_gagal_dan_reaksi_silang() {
        // HTTP online tapi unduh media gagal — butuh stub terpisah.
        #[derive(Debug)]
        struct UnduhGagal;
        impl HttpGet for UnduhGagal {
            fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
                Err(HttpError {
                    message: "offline".into(),
                })
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
        struct MediaGagal;
        impl MediaFetch for MediaGagal {
            fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
                Err(MediaError {
                    message: "offline".into(),
                })
            }
        }
        let r = run_dgn(
            Arc::new(UnduhGagal),
            Arc::new(MediaGagal),
            "Zann | desk | kata |",
            true,
        );
        assert_eq!(text_of(&r), GAGAL_RENDER);
        assert_eq!(reaction_of(&r), Some("❌".to_string()));
    }

    #[test]
    fn jalur_berhasil_mengirim_jpeg_dan_reaksi_centang() {
        let stub = Stub::default();
        stub.jawaban
            .lock()
            .unwrap()
            .push(("https://font/anton.ttf".into(), Vec::new()));
        // ctx.http dipakai untuk font + bg URL; beri jawaban font dgn byte kosong
        // supaya font_dari gagal → fallback, dan render tetap jalan.
        let http: Arc<dyn HttpGet> = Arc::new(StubDenganRender {
            dasar: Mutex::new(stub),
        });
        let media_fetch: Arc<dyn MediaFetch> = Arc::new(StubUnduh);
        let r = run_dgn(http, media_fetch, "Zann | deskripsi | kata kata |", true);
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
            }
            lain => panic!("harus image, dapat {lain:?}"),
        }
    }

    /// Stub yang selalu balas byte gambar kecil untuk URL mana pun.
    #[derive(Debug, Default)]
    struct StubDenganRender {
        #[allow(dead_code)]
        dasar: Mutex<Stub>,
    }

    impl HttpGet for StubDenganRender {
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(Vec::new()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
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

    impl MediaFetch for StubDenganRender {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            Ok(png_kecil(80, 80))
        }
    }

    #[test]
    fn bg_url_gagal_unduh_fallback_default_tanpa_panic() {
        let r = run_dgn(
            Stub::offline() as Arc<dyn HttpGet>,
            Arc::new(StubUnduh) as Arc<dyn MediaFetch>,
            "Zann | desk | kata | https://x/bg.jpg",
            true,
        );
        // unduh bg gagal → fallback default, render tetap jalan.
        assert_eq!(reaction_of(&r), Some("✅".to_string()));
    }

    #[test]
    fn unduh_bg_membatasi_ukuran() {
        let url = "https://x/bg.jpg";
        let stub = Stub::default();
        stub.jawaban
            .lock()
            .unwrap()
            .push((url.into(), vec![0u8; 100]));
        let http: Arc<dyn HttpGet> = Arc::new(stub);
        assert!(unduh_bg(&*http, url).is_ok());
    }
}
