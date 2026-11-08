//! `plugins/tools/qrcustom.js` — generate QR code custom dengan logo.
//!
//! Encoder QR di bawah adalah port 1:1 dari lib `qrcode` yang dipakai JS
//! (`QRCode.toBuffer(data, { width: 512, margin: 2, errorCorrectionLevel: 'H' })`):
//! tabel RS-block/codewords, pola alignment, format/version info, 8 mask
//! dengan penalty N1-N4, dan zigzag data — semuanya mengikuti
//! `node-qrcode/lib/core/*` supaya hasilnya scannable seperti JS.
//!
//! `ponytail:` DUA hal JS tidak di-port:
//! - Komposit logo (`sharp(logo).resize(120,120)` + `composite centre`,
//!   `qrcustom.js:38-41`): `oktz_plugin_api::gambar` tidak mengekspos piksel
//!   mentah (hanya resize/encode), jadi logo dari `m.download()` /
//!   `m.quoted.download()` tidak bisa ditempel ke matriks QR. QR dikirim
//!   tanpa logo.
//! - Reaksi `📱` (`qrcustom.js:51`) dibuang: membungkus gambar dengan
//!   `with_reaction` membuat gambar tidak pernah di-upload (lihat
//!   `random/cecanindo.rs`). Pesan `🕕` tetap dikirim lebih dulu seperti JS.
//!
//! Input di-encode sebagai SATU segmen BYTE. JS mengoptimasi segmen
//! (numeric/alphanumeric) lewat `Segments.fromString`; hasil scan tetap
//! sama, hanya susunan bit internalnya yang berbeda.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use crate::common::error_template;

/// Lebar gambar keluaran JS (`qrcustom.js:32`) dan margin modul (`margin: 2`).
const LEBAR: u32 = 512;
const MARGIN: u32 = 2;

// ---------------------------------------------------------------------------
// Galois field GF(256) — `galois-field.js`
// ---------------------------------------------------------------------------

/// Log/anti-log GF(256), polinomial 0x11D (`galois-field.js:14-30`).
fn tabel_gf() -> &'static (Vec<u8>, Vec<u8>) {
    use std::sync::OnceLock;
    static TABEL: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
    TABEL.get_or_init(|| {
        let mut exp = vec![0u8; 512];
        let mut log = vec![0u8; 256];
        let mut x: u16 = 1;
        for (i, e) in exp.iter_mut().take(255).enumerate() {
            *e = x as u8;
            log[x as usize] = i as u8;
            x <<= 1;
            if x & 0x100 != 0 {
                x ^= 0x11D;
            }
        }
        for i in 255..512 {
            exp[i] = exp[i - 255];
        }
        (exp, log)
    })
}

/// `GF.mul` (`galois-field.js:53-62`).
fn gf_kali(x: u8, y: u8) -> u8 {
    if x == 0 || y == 0 {
        return 0;
    }
    let (exp, log) = tabel_gf();
    exp[log[x as usize] as usize + log[y as usize] as usize]
}

/// Polinomial generator derajat `n`: hasil kali (x + α^i), i = 0..n
/// (`polynomial.js` `generateECPolynomial`), monik dengan koefisien
/// tertinggi lebih dulu.
fn poly_generator(n: usize) -> Vec<u8> {
    let mut gen_poly = vec![1u8];
    for i in 0..n {
        // (x + α^i) dikali gen: result[j] += gen[j] (dari x), result[j+1] += gen[j]·α^i.
        let a = tabel_gf().0[i];
        let mut baru = vec![0u8; gen_poly.len() + 1];
        for (j, &c) in gen_poly.iter().enumerate() {
            baru[j] ^= c;
            baru[j + 1] ^= gf_kali(c, a);
        }
        gen_poly = baru;
    }
    gen_poly
}

/// Sisa pembagian data oleh polinomial generator = codeword EC
/// (`reed-solomon-encoder.js` + `polynomial.js` `mod`).
fn rs_ec(data: &[u8], ec_len: usize) -> Vec<u8> {
    let gen_poly = poly_generator(ec_len);
    let mut rem = vec![0u8; data.len() + ec_len];
    rem[..data.len()].copy_from_slice(data);
    for i in 0..data.len() {
        let coef = rem[i];
        if coef != 0 {
            for j in 1..=ec_len {
                rem[i + j] ^= gf_kali(gen_poly[j], coef);
            }
        }
    }
    rem[data.len()..].to_vec()
}

// ---------------------------------------------------------------------------
// Tabel spesifikasi — `error-correction-code.js` (kolom H) + `utils.js`
// ---------------------------------------------------------------------------

/// `CODEWORDS_COUNT` (`utils.js:2-8`) — total codeword per versi.
const TOTAL_CODEWORDS: [u16; 41] = [
    0, 26, 44, 70, 100, 134, 172, 196, 242, 292, 346, 404, 466, 532, 581, 655, 733, 815, 901,
    991, 1085, 1156, 1258, 1364, 1474, 1588, 1706, 1828, 1921, 2051, 2185, 2323, 2465, 2611,
    2761, 2876, 3034, 3196, 3362, 3532, 3706,
];

/// `EC_BLOCKS_TABLE` kolom H (`error-correction-code.js:8-49`) — jumlah blok.
const BLOK_H: [u16; 40] = [
    1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35,
    37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
];

/// `EC_CODEWORDS_TABLE` kolom H (`error-correction-code.js:53-94`) — EC per blok.
const EC_H: [u16; 40] = [
    17, 28, 44, 64, 88, 112, 130, 156, 192, 224, 264, 308, 352, 384, 432, 480, 532, 588, 650,
    700, 750, 816, 900, 960, 1050, 1110, 1200, 1260, 1350, 1440, 1530, 1620, 1710, 1800, 1890,
    1980, 2100, 2220, 2310, 2430,
];

/// `getSymbolTotalCodewords`.
fn total_codewords(versi: usize) -> u16 {
    TOTAL_CODEWORDS[versi]
}

/// Kapasitas byte mode untuk satu versi pada level H (`version.js:73-105`).
fn kapasitas_byte(versi: usize) -> usize {
    let sisa = total_codewords(versi) - EC_H[versi - 1];
    let bit_data = (sisa as usize) * 8;
    // reserved: 4 bit mode + char count (BYTE: 8 bit v1-9, 16 bit v10-40).
    let cc = if versi < 10 { 8 } else { 16 };
    (bit_data - 4 - cc) / 8
}

/// Versi terkecil yang memuat `len` byte (`version.js` `getBestVersionForData`).
fn versi_terbaik(len: usize) -> Option<usize> {
    (1..=40).find(|&v| len <= kapasitas_byte(v))
}

// ---------------------------------------------------------------------------
// BCH format/version info — `format-info.js` + `version.js`
// ---------------------------------------------------------------------------

/// `getBCHDigit`.
fn bch_digit(d: u32) -> u32 {
    let mut digit = 0;
    let mut d = d;
    while d != 0 {
        digit += 1;
        d >>= 1;
    }
    digit
}

/// Bit format info untuk level H (bit 2) + mask (`format-info.js:22-40`).
fn bit_format(mask: u32) -> u32 {
    const G15: u32 = (1 << 10) | (1 << 8) | (1 << 5) | (1 << 4) | (1 << 2) | (1 << 1) | 1;
    const G15_MASK: u32 = (1 << 14) | (1 << 12) | (1 << 10) | (1 << 4) | (1 << 1);
    let level_h = 2u32;
    let data = (level_h << 3) | mask;
    let mut d = data << 10;
    let g15_bch = bch_digit(G15);
    while bch_digit(d) >= g15_bch {
        d ^= G15 << (bch_digit(d) - g15_bch);
    }
    ((data << 10) | d) ^ G15_MASK
}

/// Bit version info untuk versi >= 7 (`version.js:78-100`).
fn bit_versi(versi: usize) -> u32 {
    const G18: u32 =
        (1 << 12) | (1 << 11) | (1 << 10) | (1 << 9) | (1 << 8) | (1 << 5) | (1 << 2) | 1;
    let g18_bch = bch_digit(G18);
    let mut d = (versi as u32) << 12;
    while bch_digit(d) >= g18_bch {
        d ^= G18 << (bch_digit(d) - g18_bch);
    }
    ((versi as u32) << 12) | d
}

// ---------------------------------------------------------------------------
// Matriks modul
// ---------------------------------------------------------------------------

/// BitMatrix dengan penanda reserved (`bit-matrix.js`).
struct Matriks {
    size: usize,
    data: Vec<bool>,
    reserved: Vec<bool>,
}

impl Matriks {
    fn baru(size: usize) -> Self {
        Self {
            size,
            data: vec![false; size * size],
            reserved: vec![false; size * size],
        }
    }

    fn get(&self, r: usize, c: usize) -> bool {
        self.data[r * self.size + c]
    }

    fn set(&mut self, r: i32, c: i32, v: bool) {
        if r < 0 || c < 0 || r as usize >= self.size || c as usize >= self.size {
            return;
        }
        let (r, c) = (r as usize, c as usize);
        self.data[r * self.size + c] = v;
    }

    fn set_reserved(&mut self, r: i32, c: i32, v: bool) {
        if r < 0 || c < 0 || r as usize >= self.size || c as usize >= self.size {
            return;
        }
        let (r, c) = (r as usize, c as usize);
        self.data[r * self.size + c] = v;
        self.reserved[r * self.size + c] = true;
    }

    fn is_reserved(&self, r: usize, c: usize) -> bool {
        self.reserved[r * self.size + c]
    }

    fn xor(&mut self, r: usize, c: usize, v: bool) {
        let i = r * self.size + c;
        self.data[i] ^= v;
    }
}

/// `setupFinderPattern` (`qrcode.js:38-62`).
fn pola_finder(m: &mut Matriks) {
    let size = m.size;
    for &(row, col) in &[(0usize, 0usize), (0, size - 7), (size - 7, 0)] {
        for r in -1i32..=7 {
            if row as i32 + r < 0 || size as i32 <= row as i32 + r {
                continue;
            }
            for c in -1i32..=7 {
                if col as i32 + c < 0 || size as i32 <= col as i32 + c {
                    continue;
                }
                let gelap = ((0..=6).contains(&r) && (c == 0 || c == 6))
                    || ((0..=6).contains(&c) && (r == 0 || r == 6))
                    || ((2..=4).contains(&r) && (2..=4).contains(&c));
                m.set_reserved(row as i32 + r, col as i32 + c, gelap);
            }
        }
    }
}

/// `setupTimingPattern` (`qrcode.js:70-78`).
fn pola_timing(m: &mut Matriks) {
    let size = m.size;
    for r in 8..size - 8 {
        let v = r % 2 == 0;
        m.set_reserved(r as i32, 6, v);
        m.set_reserved(6, r as i32, v);
    }
}

/// Koordinat baris/kolom alignment (`alignment-pattern.js:30-48`).
fn koordinat_alignment(versi: usize) -> Vec<usize> {
    if versi == 1 {
        return Vec::new();
    }
    let pos_count = versi / 7 + 2;
    let size = versi * 4 + 17;
    let intervals = if size == 145 {
        26
    } else {
        (size - 13).div_ceil(2 * pos_count - 2) * 2
    };
    let mut pos = vec![size - 7];
    for i in 1..pos_count - 1 {
        pos.push(pos[i - 1] - intervals);
    }
    pos.push(6);
    pos.reverse();
    pos
}

/// `setupAlignmentPattern` (`qrcode.js:86-106`).
fn pola_alignment(m: &mut Matriks, versi: usize) {
    let coords = koordinat_alignment(versi);
    let n = coords.len();
    for i in 0..n {
        for j in 0..n {
            if (i == 0 && j == 0) || (i == 0 && j == n - 1) || (i == n - 1 && j == 0) {
                continue;
            }
            let (row, col) = (coords[i], coords[j]);
            for r in -2i32..=2 {
                for c in -2i32..=2 {
                    let gelap = r == -2 || r == 2 || c == -2 || c == 2 || (r == 0 && c == 0);
                    m.set_reserved(row as i32 + r, col as i32 + c, gelap);
                }
            }
        }
    }
}

/// `setupFormatInfo` (`qrcode.js:177-205`) — ditempatkan apa adanya, juga
/// untuk mask dummy 0 saat reservasi.
fn pasang_format_info(m: &mut Matriks, mask: u32) {
    let size = m.size;
    let bits = bit_format(mask);
    for i in 0..15u32 {
        let mod_ = (bits >> i) & 1 == 1;
        // vertikal
        if i < 6 {
            m.set_reserved(i as i32, 8, mod_);
        } else if i < 8 {
            m.set_reserved(i as i32 + 1, 8, mod_);
        } else {
            m.set_reserved(size as i32 - 15 + i as i32, 8, mod_);
        }
        // horizontal
        if i < 8 {
            m.set_reserved(8, size as i32 - i as i32 - 1, mod_);
        } else if i < 9 {
            m.set_reserved(8, 15 - i as i32 - 1 + 1, mod_);
        } else {
            m.set_reserved(8, 15 - i as i32 - 1, mod_);
        }
    }
    m.set_reserved(size as i32 - 8, 8, true);
}

/// `setupVersionInfo` (`qrcode.js:135-152`).
fn pasang_versi_info(m: &mut Matriks, versi: usize) {
    let size = m.size;
    let bits = bit_versi(versi);
    for i in 0..18u32 {
        let row = (i / 3) as i32;
        let col = (i % 3) as i32 + size as i32 - 8 - 3;
        let mod_ = (bits >> i) & 1 == 1;
        m.set_reserved(row, col, mod_);
        m.set_reserved(col, row, mod_);
    }
}

/// `setupData` (`qrcode.js:213-250`) — zigzag dua kolom dari kanan bawah.
fn pasang_data(m: &mut Matriks, data: &[u8]) {
    let size = m.size;
    let mut inc: i32 = -1;
    let mut row: i32 = size as i32 - 1;
    let mut bit_index: i32 = 7;
    let mut byte_index: usize = 0;
    let mut col: i32 = size as i32 - 1;
    while col > 0 {
        if col == 6 {
            col -= 1;
        }
        loop {
            for c in 0..2 {
                let cc = (col - c) as usize;
                if !m.is_reserved(row as usize, cc) {
                    let gelap = if byte_index < data.len() {
                        (data[byte_index] >> bit_index) & 1 == 1
                    } else {
                        false
                    };
                    m.set(row, col - c, gelap);
                    bit_index -= 1;
                    if bit_index == -1 {
                        byte_index += 1;
                        bit_index = 7;
                    }
                }
            }
            row += inc;
            if row < 0 || row as usize >= size {
                row -= inc;
                inc = -inc;
                break;
            }
        }
        col -= 2;
    }
}

/// Nilai mask pada posisi (i, j) (`mask-pattern.js:156-171`).
fn nilai_mask(p: usize, i: usize, j: usize) -> bool {
    match p {
        0 => (i + j).is_multiple_of(2),
        1 => i.is_multiple_of(2),
        2 => j.is_multiple_of(3),
        3 => (i + j).is_multiple_of(3),
        4 => (i / 2 + j / 3).is_multiple_of(2),
        5 => (i * j) % 2 + (i * j) % 3 == 0,
        6 => ((i * j) % 2 + (i * j) % 3).is_multiple_of(2),
        7 => ((i * j) % 3 + (i + j) % 2).is_multiple_of(2),
        _ => false,
    }
}

fn terapkan_mask(p: usize, m: &mut Matriks) {
    let size = m.size;
    for col in 0..size {
        for row in 0..size {
            if m.is_reserved(row, col) {
                continue;
            }
            let v = nilai_mask(p, row, col);
            m.xor(row, col, v);
        }
    }
}

/// Penalty N1 (`mask-pattern.js:51-94`).
fn penalty_n1(m: &Matriks) -> u32 {
    let size = m.size;
    let mut poin = 0u32;
    for row in 0..size {
        let mut sama_col = 0u32;
        let mut sama_row = 0u32;
        let mut last_col: Option<bool> = None;
        let mut last_row: Option<bool> = None;
        for col in 0..size {
            let module = m.get(row, col);
            if Some(module) == last_col {
                sama_col += 1;
            } else {
                if sama_col >= 5 {
                    poin += 3 + (sama_col - 5);
                }
                last_col = Some(module);
                sama_col = 1;
            }
            let module = m.get(col, row);
            if Some(module) == last_row {
                sama_row += 1;
            } else {
                if sama_row >= 5 {
                    poin += 3 + (sama_row - 5);
                }
                last_row = Some(module);
                sama_row = 1;
            }
        }
        if sama_col >= 5 {
            poin += 3 + (sama_col - 5);
        }
        if sama_row >= 5 {
            poin += 3 + (sama_row - 5);
        }
    }
    poin
}

/// Penalty N2 (`mask-pattern.js:101-118`).
fn penalty_n2(m: &Matriks) -> u32 {
    let size = m.size;
    let mut poin = 0u32;
    for row in 0..size - 1 {
        for col in 0..size - 1 {
            let total = m.get(row, col) as u32
                + m.get(row, col + 1) as u32
                + m.get(row + 1, col) as u32
                + m.get(row + 1, col + 1) as u32;
            if total == 4 || total == 0 {
                poin += 1;
            }
        }
    }
    poin * 3
}

/// Penalty N3 (`mask-pattern.js:125-146`).
fn penalty_n3(m: &Matriks) -> u32 {
    let size = m.size;
    let mut poin = 0u32;
    for row in 0..size {
        let mut bits_col = 0u32;
        let mut bits_row = 0u32;
        for col in 0..size {
            bits_col = ((bits_col << 1) & 0x7FF) | m.get(row, col) as u32;
            if col >= 10 && (bits_col == 0x5D0 || bits_col == 0x05D) {
                poin += 1;
            }
            bits_row = ((bits_row << 1) & 0x7FF) | m.get(col, row) as u32;
            if col >= 10 && (bits_row == 0x5D0 || bits_row == 0x05D) {
                poin += 1;
            }
        }
    }
    poin * 40
}

/// Penalty N4 (`mask-pattern.js:153-165`).
fn penalty_n4(m: &Matriks) -> u32 {
    let mut gelap = 0u32;
    let total = (m.size * m.size) as u32;
    for i in 0..m.size * m.size {
        gelap += m.data[i] as u32;
    }
    // `Math.ceil((dark * 100 / total) / 5) - 10`
    let k = ((gelap * 100).div_ceil(total * 5)) as i32 - 10;
    (k.unsigned_abs()) * 10
}

/// `getBestMask` (`mask-pattern.js:174-199`).
fn mask_terbaik(m: &mut Matriks) -> usize {
    let mut best = 0usize;
    let mut terendah = u32::MAX;
    for p in 0..8usize {
        pasang_format_info(m, p as u32);
        terapkan_mask(p, m);
        let penalty = penalty_n1(m) + penalty_n2(m) + penalty_n3(m) + penalty_n4(m);
        terapkan_mask(p, m); // batalkan mask
        if penalty < terendah {
            terendah = penalty;
            best = p;
        }
    }
    best
}

/// `createData` (`qrcode.js:259-316`) — satu segmen BYTE (mode bit 4).
fn buat_data(versi: usize, data: &[u8]) -> Vec<u8> {
    let mut bits: Vec<bool> = Vec::new();
    let taruh = |v: u32, n: u32, bits: &mut Vec<bool>| {
        for i in (0..n).rev() {
            bits.push((v >> i) & 1 == 1);
        }
    };
    taruh(4, 4, &mut bits); // mode BYTE
    let cc = if versi < 10 { 8 } else { 16 };
    taruh(data.len() as u32, cc, &mut bits);
    for &b in data {
        taruh(b as u32, 8, &mut bits);
    }

    let sisa = total_codewords(versi) - EC_H[versi - 1];
    let bit_data = (sisa as usize) * 8;
    if bits.len() + 4 <= bit_data {
        taruh(0, 4, &mut bits);
    }
    while !bits.len().is_multiple_of(8) {
        bits.push(false);
    }
    let sisa_byte = (bit_data - bits.len()) / 8;
    for i in 0..sisa_byte {
        taruh(if i % 2 == 1 { 0x11 } else { 0xEC }, 8, &mut bits);
    }
    assert_eq!(bits.len(), bit_data, "buffer tidak pas");

    // Byte-kan.
    let mut buf = Vec::with_capacity(bits.len() / 8);
    for chunk in bits.chunks(8) {
        let mut b = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                b |= 1 << (7 - i);
            }
        }
        buf.push(b);
    }
    interleave(&buf, versi)
}

/// `createCodewords` (`qrcode.js:325-395`) — bagi blok, hitung EC, interleave.
fn interleave(buf: &[u8], versi: usize) -> Vec<u8> {
    let total = total_codewords(versi) as usize;
    let ec_total = EC_H[versi - 1] as usize;
    let data_total = total - ec_total;
    let blok = BLOK_H[versi - 1] as usize;
    let blok_g2 = total % blok;
    let blok_g1 = blok - blok_g2;
    let total_g1 = total / blok;
    let data_g1 = data_total / blok;
    let data_g2 = data_g1 + 1;
    let ec_count = total_g1 - data_g1;

    let mut dc: Vec<Vec<u8>> = Vec::with_capacity(blok);
    let mut ec: Vec<Vec<u8>> = Vec::with_capacity(blok);
    let mut offset = 0usize;
    for b in 0..blok {
        let ukuran = if b < blok_g1 { data_g1 } else { data_g2 };
        dc.push(buf[offset..offset + ukuran].to_vec());
        ec.push(rs_ec(&dc[b], ec_count));
        offset += ukuran;
    }

    let mut out = Vec::with_capacity(total);
    for i in 0..data_g2 {
        for d in &dc {
            if i < d.len() {
                out.push(d[i]);
            }
        }
    }
    for i in 0..ec_count {
        for e in &ec {
            out.push(e[i]);
        }
    }
    out
}

/// `createSymbol` (`qrcode.js:400-460`) — matriks akhir + mask yang dipakai.
fn matriks_qr(data: &[u8]) -> Option<(Matriks, usize)> {
    let versi = versi_terbaik(data.len())?;
    let codewords = buat_data(versi, data);

    let size = versi * 4 + 17;
    let mut m = Matriks::baru(size);
    pola_finder(&mut m);
    pola_timing(&mut m);
    pola_alignment(&mut m, versi);
    pasang_format_info(&mut m, 0); // reservasi dummy
    if versi >= 7 {
        pasang_versi_info(&mut m, versi);
    }
    pasang_data(&mut m, &codewords);

    let mask = mask_terbaik(&mut m);
    terapkan_mask(mask, &mut m);
    pasang_format_info(&mut m, mask as u32);
    Some((m, mask))
}

// ---------------------------------------------------------------------------
// Penulis PNG grayscale — padanan `toBuffer({ type: 'png', width: 512 })`
// ---------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Zlib dengan blok deflate "stored" (tanpa kompresi) — PNG tetap valid.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut chunks = data.chunks(65535).peekable();
    if data.is_empty() {
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
    }
    while let Some(chunk) = chunks.next() {
        let terakhir = chunks.peek().is_none();
        out.push(if terakhir { 1 } else { 0 });
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// PNG grayscale 8-bit dari matriks modul.
fn matriks_ke_png(m: &Matriks) -> Vec<u8> {
    let size = m.size as u32;
    let q = size + MARGIN * 2;
    let scale = (LEBAR / q).max(1);
    let dim = q * scale;

    // Baris piksel: filter 0 + `dim` byte (0 = hitam, 255 = putih).
    let mut raw = Vec::with_capacity((dim as usize + 1) * dim as usize);
    for y in 0..dim {
        raw.push(0);
        let my = y / scale;
        for x in 0..dim {
            let mx = x / scale;
            let dalam = mx >= MARGIN && my >= MARGIN && mx < MARGIN + size && my < MARGIN + size;
            let gelap = dalam && m.get((my - MARGIN) as usize, (mx - MARGIN) as usize);
            raw.push(if gelap { 0 } else { 255 });
        }
    }

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&dim.to_be_bytes());
    ihdr.extend_from_slice(&dim.to_be_bytes());
    ihdr.extend_from_slice(&[8, 0, 0, 0, 0]);
    for (tipe, isi) in [(b"IHDR", &ihdr[..]), (b"IDAT", &zlib_stored(&raw)[..])] {
        png.extend_from_slice(&(isi.len() as u32).to_be_bytes());
        png.extend_from_slice(tipe);
        png.extend_from_slice(isi);
        png.extend_from_slice(&crc32(&[tipe as &[u8], isi].concat()).to_be_bytes());
    }
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&crc32(b"IEND").to_be_bytes());
    png
}

/// `QRCode.toBuffer(data, { type: 'png', width: 512, margin: 2, errorCorrectionLevel: 'H' })`.
fn qr_png(data: &[u8]) -> Option<Vec<u8>> {
    matriks_qr(data).map(|(m, _)| matriks_ke_png(&m))
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Kartu pakai saat teks kosong (`qrcustom.js:20-29`).
pub(crate) fn kartu_pakai(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{prefix}qrcustom <url/text>`\n\n*Contoh:*\n> `{prefix}qrcustom https://wa.me/628xxx`\n\n💡 Reply gambar untuk custom logo di tengah QR"
    )
}

/// `📱 *QR Code*\n> ${data.substring(0, 50)}${data.length > 50 ? '...' : ''}`
/// (`qrcustom.js:47`).
pub(crate) fn caption_qr(data: &str) -> String {
    let potong: String = data.chars().take(50).collect();
    format!("📱 *QR Code*\n> {potong}{}", if data.chars().count() > 50 { "..." } else { "" })
}

#[plugin(
    name = "qrcustom",
    commands = ["qrcode", "qr"],
    alias = [],
    category = "tools",
    description = "Generate QR code custom dengan logo",
    usage = ".qrcustom <url>",
    example = ".qrcustom https://wa.me/628xxx",
    cooldown = 10,
    energi = 1,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false
)]
pub fn qrcustom(ctx: Ctx) -> Reply {
    let data = ctx.text.trim();

    if data.is_empty() {
        return Reply::text(kartu_pakai(&ctx.prefix));
    }

    // Pesan 🕜 dikirim sebelum try (`qrcustom.js:32`).
    let awal = Reply::text("🕕 *Generating QR code...*");

    let Some(png) = qr_png(data.as_bytes()) else {
        // Data melebihi kapasitas versi 40-H — JS `throw` → catch.
        return Reply::sequence(vec![
            awal,
            Reply::text(error_template(&ctx)).with_reaction("☢"),
        ]);
    };

    Reply::sequence(vec![
        awal,
        Reply::image(MediaSource::bytes(png), caption_qr(data)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "qrcustom".into(),
            ..Ctx::default()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("qrcustom").unwrap();
        assert_eq!(p.meta.name, "qrcustom");
        assert_eq!(p.meta.commands, ["qrcode", "qr"]);
        assert!(p.meta.alias.is_empty());
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.description, "Generate QR code custom dengan logo");
        assert_eq!(p.meta.usage, ".qrcustom <url>");
        assert_eq!(p.meta.example, ".qrcustom https://wa.me/628xxx");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn kunci_qrcode_menang_dari_alias_txt2qr_seperti_js() {
        // `commands` dicek sebelum alias (`plugins.js:539-544`), jadi
        // `qrcode` milik qrcustom (command) menimpa alias `txt2qr`.
        assert_eq!(
            oktz_plugin_api::find("qrcode").map(|p| p.meta.name.as_str()),
            Some("qrcustom")
        );
        assert_eq!(
            oktz_plugin_api::find("qr").map(|p| p.meta.name.as_str()),
            Some("qrcustom")
        );
    }

    #[test]
    fn teks_kosong_menghasilkan_kartu_pakai() {
        let r = qrcustom(ctx_of(""));
        let t = r.text_body().unwrap();
        assert!(t.starts_with("⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*"));
        assert!(t.contains("`.qrcustom <url/text>`"));
        assert!(t.contains("💡 Reply gambar untuk custom logo di tengah QR"));
        assert!(!r.has_media());
    }

    #[test]
    fn alur_utama_kirim_pesan_mancing_lalu_gambar_png() {
        let r = qrcustom(ctx_of("https://wa.me/628xxx"));
        let bagian = match r.without_mutations() {
            Reply::Sequence(v) => v,
            other => panic!("harus Sequence, dapat {other:?}"),
        };
        assert_eq!(bagian.len(), 2);
        assert_eq!(bagian[0].text_body().unwrap(), "🕕 *Generating QR code...*");
        assert!(bagian[1].has_media(), "harus ada gambar QR");
        match bagian[1].media_source().unwrap() {
            MediaSource::Bytes { data, .. } => {
                assert_eq!(&data[..8], b"\x89PNG\r\n\x1a\n");
                let (w, h) =
                    oktz_plugin_api::kanvas::Kanvas::ukuran_png(data).expect("PNG valid");
                // Versi 3 = 29 modul + margin 4 = 33; scale = 512/33 = 15.
                assert_eq!((w, h), (495, 495), "ukuran PNG 512 dipetakan ke grid");
            }
            other => panic!("sumber harus Bytes, dapat {other:?}"),
        }
        assert_eq!(
            bagian[1].text_body().unwrap(),
            "📱 *QR Code*\n> https://wa.me/628xxx"
        );
    }

    #[test]
    fn caption_dipotong_50_karakter_dengan_ellipsis() {
        let data = "a".repeat(80);
        assert_eq!(caption_qr(&data), format!("📱 *QR Code*\n> {}...", "a".repeat(50)));
        assert_eq!(caption_qr("abc"), "📱 *QR Code*\n> abc");
    }

    #[test]
    fn qr_versi_dan_mask_mengikuti_lib_qrcode() {
        // Fixture dari `node-qrcode` — `create([{data, mode:'byte'}], {H})`
        // dijalankan dengan node (lib yang sama dipakai JS plugin), lalu
        // matriksnya dibuang ke sini sebagai bit-string baris-per-baris.
        let fixtures: [(&[u8], usize, usize, &str); 3] = [
            (b"oktz".as_slice(), 1, 2, FIX_OKTZ),
            (b"https://wa.me/628xxx".as_slice(), 3, 0, FIX_WA_ME),
            (&[b'x'; 130], 11, 2, FIX_X130),
        ];
        for (data, versi, mask, bits) in fixtures {
            let (m, m_mask) = matriks_qr(data).unwrap();
            assert_eq!(m.size, versi * 4 + 17, "{:?} ukuran", std::str::from_utf8(data).unwrap_or("?"));
            assert_eq!(m_mask, mask, "{:?} mask", std::str::from_utf8(data).unwrap_or("?"));
            let hex: String = m.data.iter().map(|&b| if b { '1' } else { '0' }).collect();
            assert_eq!(hex, bits, "{:?} matriks", std::str::from_utf8(data).unwrap_or("?"));
        }
    }

    #[test]
    fn qr_teks_panjang_naik_versi() {
        // Kapasitas byte-H: v9 98 byte, v10 119 byte, v11 137 byte.
        assert_eq!(versi_terbaik(98).unwrap(), 9);
        assert_eq!(versi_terbaik(119).unwrap(), 10);
        assert_eq!(versi_terbaik(120).unwrap(), 11);
    }

    #[test]
    fn data_melebihi_kapasitas_jatuh_ke_error_template() {
        let data = "a".repeat(1500);
        let ctx = ctx_of(&data);
        let r = qrcustom(ctx);
        let bagian = match r.without_mutations() {
            Reply::Sequence(v) => v,
            other => panic!("harus Sequence, dapat {other:?}"),
        };
        assert_eq!(bagian[0].text_body().unwrap(), "🕕 *Generating QR code...*");
        assert!(!bagian[1].text_body().unwrap().is_empty());
        assert!(
            matches!(bagian[1].without_mutations(), Reply::Reaction { emoji, .. } if emoji == "☢")
        );
    }

    // -- fixture dari node-qrcode (bit matriks, baris-per-baris) -----------

    const FIX_OKTZ: &str = "111111101011001111111100000101100101000001101110101010001011101101110100001001011101101110100001001011101100000101010001000001111111101010101111111000000001011000000000001110101110011100111100000011110110100001111010111010111011010111111000010000111111101001110100101011011000000001011110001101111111100001101100110100000100001011001100101110101001100101010101110101000101100000101110101111101010000100000100011110110100111111100100111010110";

    const FIX_WA_ME: &str = "1111111011111110111000111111110000010010001001010001000001101110100101110000110010111011011101011011101101000101110110111010011000100011101011101100000100110001111011010000011111111010101010101010111111100000000011111111111000000000001011101010110110100100010010101110001010100111100100010110100011010001110010000011011111100011110101001010111110100000001101011001101010110100001111001010110000000111100011010110100100110010100001001011010100101111111000011010000011000010010001100000111101011001000010010011110011010000111001111010101111110111001001101101000011010111000101111010100010110100011001011111110110000000011011011100010001001111111110000000111011101010111100000101001100000101000100001011101011111101100011111101010111010010010010010110011000101110101100000010110001100011000001001000010000101110101011111110001010100000101011011";

    const FIX_X130: &str = "1111111010000101110100110000011000011111111000001001101111111100000101001111010000111001011100100111101001010000110100000110111010110111101010100011111110110010001001111110111010111011011101001110010011110010100011010011010101101001110101011101101110100100010001010101000011111001111111100001111100101110110000010100011111000011101001000110011110100101011100010000011111111010101010101010101010101010101010101010101010101111111000000001101111010101010111110001100111011011111001110000000000111010111000100111100101011111100110001001010110001111001110100010101010100010100010110101000011011110000001100011001110011000100101111110000101010011000100110100101010011101001101101010000111111101010100011111010110101101001111100111001100011101101000000010111111110101010010001000101101011000101100100000101011111010001010111011010000001001111100000110001100110101110010010111101010010101001100010000110100101001110100110000100000011111110110010001110111011010110100111110011100110001110110100000001101110111010110001000111010110101100010110010100000101111101001010111101111000000101011110000011000110011100001001001011111001001110100110001000011010010100111010011011000000001111111010101100111011101111111010011111001110011000100011011100000101001011101011000110111101011010110001011001001000010101110101011011110111100000100101111000001100011001110000100101101111010100111010011100000001101001010011101001101100000000111111110100110011101100111001001001111100111001100010001101000000010101101110101111010010101101101011000101100100100001011111010101110111011110000100000111100000110001100111000010011010110101011111101001110010100100100101001110100110110000000001111011010011001110110010110100100111110011100110001000111111000010010100110010111111101110110110101100011111010010001000111100010110111101101000110100011110000010001000111100001101011011010101001100101101010100010010010100000101011011000010001111100101011100011010001011010000011111001110001000100011111100001001101111001001111110111000011010110011111101001000100001110101001011110110111011010000011000001101001101110000101110101100010110110001000000010001011001010011011100101100000000111110010101010000100110001101010001011100101100100010001001000001010110111111000110010111100101001011001000110100100010000110101100101111010011101011000111110000110100110111000010111000110001011011100100000111000101100001001101110010110000000011100001010101011000011100110100100110110010110010001000110100100101011011110101011000011110000110111100100011010010001100010010110010111000001110101100010110001011010011011100001011100011100101101101100000011100011110011000110111001011000000001110011101010101011001110011010010011111001011001000100101110011111101101111110000100001111000011010110010001101001001000000110001001011110100011010110001011000001101001101010111011110111010010110000110010010110001111001010011011100111100011100111011110101011001111111001101001001111010101100100011010001001011111110110111001010000111100001101010001000110100010111000100100010101011010001110011000101100000010100110111000111111011011000011010010101001011000111100101101101110010111110100011100110110101100001111110110100101111110010110010001111100100100111111011101111111111011101001110101100111111010000000000110100001010111101001000101100110010000011011000111101111111001110100001100101010101011100111111010100111101011011100000100001111011010000000010001011010010011111001110001000110111010110011111101110111101111101100000011010110001111101001011101011010000101011010101010100100001011000001100100111101101110101111010011110010101101110110110111001010011000110100110000010000111101011000000010100101100001001111100110110000011111111001001110001001011111010110110010001101011001110010111";
}
