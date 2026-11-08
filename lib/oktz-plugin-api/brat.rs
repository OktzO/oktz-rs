//! Brat static image renderer.

use ab_glyph::{Font, FontArc, ScaleFont};
use image::{DynamicImage, RgbaImage};

const FONT: &[u8] = include_bytes!("../../assets/fonts/arialnarrow.ttf");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BratTema {
    Putih,
    Hijau,
    Anime,
    Cewek,
    Hitam,
}

impl BratTema {
    pub fn warna_latar(self) -> [u8; 4] {
        match self {
            Self::Putih => [255, 255, 255, 255],
            Self::Hijau => [138, 206, 0, 255],
            Self::Anime => [255, 105, 180, 255],
            Self::Cewek => [255, 192, 203, 255],
            Self::Hitam => [0, 0, 0, 255],
        }
    }

    pub fn warna_teks(self) -> [u8; 4] {
        match self {
            Self::Hitam => [255, 255, 255, 255],
            _ => [0, 0, 0, 255],
        }
    }
}

pub fn render_brat(teks: &str, tema: BratTema, _blur: u8) -> Result<Vec<u8>, String> {
    let canvas = render_rgba(teks, tema)?;
    let mut buf = Vec::new();
    DynamicImage::ImageRgba8(canvas)
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageOutputFormat::Png)
        .map_err(|e| format!("gagal encode brat: {e}"))?;
    Ok(buf)
}

fn render_rgba(teks: &str, tema: BratTema) -> Result<RgbaImage, String> {
    let tata = TataLetak::baru(teks)?;
    let alpha = vec![1.0; tata.kata.len()];
    let geser = vec![0.0; tata.kata.len()];
    Ok(gambar(&tata, tema, &alpha, &geser))
}

const KANVAS: u32 = 1000;
const PAD: f32 = 80.0;
const JARAK_BARIS: f32 = 20.0;
const FONT_MAKS: f32 = 220.0;
const FONT_MIN: f32 = 12.0;
pub const BRAT_ANIM_FPS: u32 = 30;

struct Kata {
    teks: String,
    x: f32,
    baseline: f32,
}

struct TataLetak {
    font: FontArc,
    ukuran: f32,
    kata: Vec<Kata>,
}

fn ukur(font: &FontArc, ukuran: f32, teks: &str) -> f32 {
    let scaled = font.as_scaled(ukuran);
    teks.chars().map(|c| scaled.h_advance(font.glyph_id(c))).sum()
}

fn tinggi_baris(font: &FontArc, ukuran: f32) -> f32 {
    let scaled = font.as_scaled(ukuran);
    scaled.ascent() - scaled.descent() + JARAK_BARIS
}

fn bungkus(font: &FontArc, ukuran: f32, teks: &str, max_w: f32) -> Vec<String> {
    let mut baris: Vec<String> = Vec::new();
    let mut saat_ini = String::new();
    for kata in teks.split_whitespace() {
        let coba = if saat_ini.is_empty() {
            kata.to_string()
        } else {
            format!("{saat_ini} {kata}")
        };
        if saat_ini.is_empty() || ukur(font, ukuran, &coba) <= max_w {
            saat_ini = coba;
        } else {
            baris.push(saat_ini);
            saat_ini = kata.to_string();
        }
    }
    if !saat_ini.is_empty() {
        baris.push(saat_ini);
    }
    baris
}

impl TataLetak {
    fn baru(teks: &str) -> Result<Self, String> {
        let font = FontArc::try_from_slice(FONT).map_err(|_| "font brat tidak bisa dibaca".to_string())?;
        let max_w = KANVAS as f32 - PAD * 2.0;

        // Auto-fit: ukuran terbesar yang seluruh baris + tingginya muat.
        // Teks pendek jadi besar memenuhi lebar, teks panjang mengecil.
        let mut ukuran = FONT_MAKS;
        let baris = loop {
            let b = bungkus(&font, ukuran, teks, max_w);
            let muat = b.is_empty()
                || (b.iter().all(|l| ukur(&font, ukuran, l) <= max_w)
                    && b.len() as f32 * tinggi_baris(&font, ukuran) <= max_w);
            if muat || ukuran <= FONT_MIN {
                break b;
            }
            ukuran = (ukuran * 0.94).max(FONT_MIN);
        };

        let scaled = font.as_scaled(ukuran);
        let line_h = tinggi_baris(&font, ukuran);
        let spasi = ukur(&font, ukuran, " ");
        let mut kata = Vec::new();
        let mut atas = PAD;
        for baris in &baris {
            let mut x = PAD;
            for (i, teks_kata) in baris.split(' ').filter(|k| !k.is_empty()).enumerate() {
                if i > 0 {
                    x += spasi;
                }
                kata.push(Kata {
                    teks: teks_kata.to_string(),
                    x,
                    baseline: atas + scaled.ascent(),
                });
                x += ukur(&font, ukuran, teks_kata);
            }
            atas += line_h;
        }
        Ok(Self { font, ukuran, kata })
    }
}

fn ease_out_back(x: f32) -> f32 {
    let c1 = 1.4;
    let c3 = c1 + 1.0;
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

fn gambar(tata: &TataLetak, tema: BratTema, alpha: &[f32], geser: &[f32]) -> RgbaImage {
    let mut canvas = RgbaImage::from_pixel(KANVAS, KANVAS, image::Rgba(tema.warna_latar()));
    let warna = tema.warna_teks();
    let scaled = tata.font.as_scaled(tata.ukuran);
    for (i, kata) in tata.kata.iter().enumerate() {
        let a = alpha.get(i).copied().unwrap_or(1.0).clamp(0.0, 1.0);
        if a <= 0.0 {
            continue;
        }
        let dy = geser.get(i).copied().unwrap_or(0.0);
        let mut x = kata.x;
        for karakter in kata.teks.chars() {
            let id = tata.font.glyph_id(karakter);
            let glyph = id.with_scale_and_position(tata.ukuran, ab_glyph::point(x, kata.baseline + dy));
            if let Some(outline) = tata.font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                outline.draw(|px, py, coverage| {
                    let gx = bounds.min.x as i32 + px as i32;
                    let gy = bounds.min.y as i32 + py as i32;
                    if gx < 0 || gy < 0 || gx as u32 >= KANVAS || gy as u32 >= KANVAS {
                        return;
                    }
                    // Blend ke latar (bukan timpa alpha) supaya fade animasi
                    // tidak menghasilkan piksel transparan di MP4.
                    let t = coverage * a;
                    let p = canvas.get_pixel_mut(gx as u32, gy as u32);
                    for c in 0..3 {
                        p[c] = (warna[c] as f32 * t + p[c] as f32 * (1.0 - t)).round() as u8;
                    }
                    p[3] = 255;
                });
            }
            x += scaled.h_advance(id);
        }
    }
    canvas
}

/// Animasi `.bratvid2` (port `plugins/sticker/bratvid2.js`): kata muncul satu
/// per satu dengan easing lalu ditahan. Highlight sweep JS tidak diport.
pub struct BratAnimasi {
    tata: TataLetak,
    muncul: u32,
    stagger: u32,
    tahan: u32,
}

impl BratAnimasi {
    pub fn baru(teks: &str) -> Result<Self, String> {
        Ok(Self {
            tata: TataLetak::baru(teks)?,
            muncul: 12,
            stagger: 3,
            tahan: 45,
        })
    }

    pub fn total_frame(&self) -> u32 {
        if self.tata.kata.is_empty() {
            return self.tahan + 1;
        }
        (self.tata.kata.len() as u32 - 1) * self.stagger + self.muncul + self.tahan
    }

    /// Frame RGBA 1000×1000, siap dialirkan ke ffmpeg `-f rawvideo`.
    pub fn frame_rgba(&self, frame: u32) -> Vec<u8> {
        let n = self.tata.kata.len();
        let mut alpha = vec![0.0; n];
        let mut geser = vec![0.0; n];
        for k in 0..n {
            let mulai = k as u32 * self.stagger;
            let Some(lewat) = frame.checked_sub(mulai) else {
                continue;
            };
            let p = ((lewat + 1) as f32 / self.muncul as f32).min(1.0);
            alpha[k] = (p * 1.8).min(1.0);
            geser[k] = -35.0 * (1.0 - ease_out_back(p));
        }
        gambar(&self.tata, BratTema::Putih, &alpha, &geser).into_raw()
    }
}

pub const BRAT_PROGRESIF_FPS: u32 = 25;
pub const BRAT_PROGRESIF_DURASI: u32 = 18; // ~0.7 dtk per kata
pub const BRAT_PROGRESIF_DURASI_AKHIR: u32 = 38; // ~1.5 dtk tahan

/// Animasi `.bratvid` (port `brat-canvas` `bratVid`): tiap frame menampilkan
/// teks yang makin lengkap, satu kata per langkah. Layout dihitung ulang per
/// prefix seperti JS (`bratGen(frameText)`).
pub struct BratProgresif {
    kata: Vec<String>,
}

impl BratProgresif {
    pub fn baru(teks: &str) -> Result<Self, String> {
        let kata: Vec<String> = teks
            .split_whitespace()
            .map(str::to_string)
            .collect();
        if kata.is_empty() {
            return Err("teks kosong".to_string());
        }
        Ok(Self { kata })
    }

    pub fn jumlah_kata(&self) -> u32 {
        self.kata.len() as u32
    }

    /// `sampai` = jumlah kata pertama yang ditampilkan (1..=jumlah_kata).
    pub fn frame_rgba(&self, sampai: u32) -> Result<Vec<u8>, String> {
        if sampai == 0 {
            return Err("indeks kata harus >= 1".to_string());
        }
        let n = (sampai as usize).clamp(1, self.kata.len());
        Ok(render_rgba(&self.kata[..n].join(" "), BratTema::Putih)?.into_raw())
    }
}


pub fn render_frame_raw(teks: &str, width: u32, height: u32) -> Vec<u8> {
    let Ok(png) = render_brat(teks, BratTema::Putih, 0) else {
        return Vec::new();
    };
    let Ok(img) = image::load_from_memory(&png) else {
        return Vec::new();
    };
    let rgba = img.to_rgba8();
    if rgba.dimensions() == (width, height) {
        return rgba.into_raw();
    }
    DynamicImage::ImageRgba8(rgba)
        .resize_exact(width, height, image::imageops::FilterType::Triangle)
        .to_rgba8()
        .into_raw()
}


pub fn is_blank(buf: &[u8]) -> bool {
    if buf.len() < 100 {
        return true;
    }
    let Ok(img) = image::load_from_memory(buf) else {
        return true;
    };
    let img = img.to_rgba8();
    let small = DynamicImage::ImageRgba8(img).resize_exact(50, 50, image::imageops::FilterType::Triangle);
    let small = small.to_rgba8();
    let pertama = small.pixels().next().copied();
    let Some(pertama) = pertama else {
        return true;
    };
    small.pixels().all(|p| p == &pertama)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_brat_is_1000x1000_nonblank() {
        let png = render_brat("halo brat", BratTema::Putih, 0).unwrap();
        let decoded = image::load_from_memory(&png).unwrap();
        assert_eq!(decoded.dimensions(), (1000, 1000));
        use image::GenericImageView;
        assert!(!is_blank(&png));
    }

    #[test]
    fn is_blank_rejects_uniform_image() {
        let img = image::RgbaImage::from_fn(100, 100, |_x, _y| image::Rgba([255, 255, 255, 255]));
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageOutputFormat::Png)
            .unwrap();
        assert!(is_blank(&buf));
        assert!(!is_blank(&render_brat("halo", BratTema::Putih, 0).unwrap()));
    }

    #[test]
    fn invalid_bytes_are_blank() {
        assert!(is_blank(b"bukan gambar"));
        assert!(is_blank(&[0u8; 10]));
    }

    #[test]
    fn teks_mulai_dari_atas_dan_kiri() {
        let png = render_brat("halo", BratTema::Putih, 0).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        for (x, y, p) in img.enumerate_pixels() {
            if p[0] < 128 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
            }
        }
        // Tidak di tengah: tinta mulai dekat padding kiri-atas (80).
        assert!(min_y < 350, "y pertama {min_y} harus dekat atas");
        assert!(min_x < 200, "x pertama {min_x} harus rata kiri");
    }

    #[test]
    fn animasi_brat_frame_beda_dan_total_masuk_akal() {
        let a = BratAnimasi::baru("satu dua tiga").unwrap();
        assert!(a.total_frame() > 10);
        assert_ne!(a.frame_rgba(0), a.frame_rgba(a.total_frame() - 1));
    }

    #[test]
    fn progresif_mengungkap_kata_bertahap() {
        let p = BratProgresif::baru("satu dua tiga").unwrap();
        assert_eq!(p.jumlah_kata(), 3);
        let f1 = p.frame_rgba(1).unwrap();
        let f3 = p.frame_rgba(3).unwrap();
        assert_ne!(f1, f3);
        assert!(p.frame_rgba(0).is_err());
    }

    // -- .bratvid2 (port `plugins/sticker/bratvid2.js`) -----------------------
    //
    // `ponytail:` JS timing 60fps + scale bounce + sweep highlight di-frame
    // per elemen tidak diport; port Rust ini fade+slide 30fps, test mengikuti
    // implementasi Rust. Upgrade: port scale/sweep kalau parity visual
    // dengan bratvid2.js dibutuhkan.

    fn ke_gambar(px: &[u8]) -> RgbaImage {
        RgbaImage::from_raw(KANVAS, KANVAS, px.to_vec()).unwrap()
    }

    #[test]
    fn animasi_v2_total_frame_sesuai_port() {
        let a = BratAnimasi::baru("satu dua tiga").unwrap();
        // (kata-1)*stagger + fade + tahan
        assert_eq!(a.total_frame(), (3 - 1) * 3 + 12 + 45);
    }

    #[test]
    fn animasi_v2_fade_abu_ke_hitam_lalu_berteks_penuh() {
        let a = BratAnimasi::baru("halo brat").unwrap();
        // frame 0: kata pertama pudar (abu), belum hitam penuh.
        let awal = ke_gambar(&a.frame_rgba(0));
        assert!(
            awal.pixels().any(|p| p[0] > 64 && p[0] < 250),
            "frame awal harus ada teks pudar"
        );
        let akhir = ke_gambar(&a.frame_rgba(a.total_frame() - 1));
        assert!(
            akhir.pixels().any(|p| p[0] < 64),
            "frame akhir harus teks hitam"
        );
    }

    #[test]
    fn animasi_v2_berbeda_dan_bratvid_progresif() {
        let a = BratAnimasi::baru("satu dua tiga").unwrap();
        let p = BratProgresif::baru("satu dua tiga").unwrap();
        assert_ne!(
            a.frame_rgba(0),
            p.frame_rgba(1).unwrap(),
            "bratvid2 fade bertahap harus beda dari bratvid reveal"
        );
    }
}
