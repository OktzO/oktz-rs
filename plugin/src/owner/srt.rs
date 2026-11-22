//! `plugins/owner/srt.js` — Sistem Shuffle Reply Thumb (SRT).
//!
//! Port dari nol. Yang dipindah:
//! - semua sub-perintah `on` / `off` / `c` / `d` / `list` / `del`
//!   (`srt.js:47-171`).
//! - session tangkapan gambar per-chat dengan TTL 10 menit (`:24-38`).
//! - hitung gambar di `assets/image/shuffle` (`:41-45`).
//! - penyimpanan gambar tangkapan dengan nama `srt_<md5[0..10]>.jpeg` dan
//!   resize `1280x720 fit:inside` + JPEG kualitas 85 (`:190-208`).
//! - penghapusan gambar (`:131-162`).
//!
//! `ponytail:` MD5 (`crypto.createHash("md5")`) tidak punya crate yang
//! tersedia di `oktz-plugins`, jadi implementasinya diletakkan di berkas ini
//! (lihat [`md5_hex`]). Hasilnya identik dengan MD5 standar, sehingga nama
//! berkas `srt_<hash>.jpeg` sama dengan yang dipakai JS.
//!
//! `ponytail:` album gambar (`albumMessage`, `srt.js:86-125`) tidak ada di
//! `Reply`; jadi gambarnya dikirim berurutan sebagai pesan terpisah lewat
//! `Reply::sequence`.
//!
//! `ponytail:` `global.srtSession` dipetakan ke `ctx.sessions` dengan key
//! `srt:session:<chat>`; `srtEnabled` dipetakan ke setting global
//! `srtEnabled`.

use oktz_plugin_api::{Ctx, MediaSource, Mutation, Reply, answer_handler, plugin};
use serde_json::Value;

/// Folder koleksi gambar (`srt.js:28`).
const SHUFFLE_DIR: &str = "assets/image/shuffle";
/// TTL idle 10 menit (`srt.js:29`).
const TTL_MS: i64 = 10 * 60 * 1000;
/// Prefix key session (`global.srtSession`).
const SESSION_PREFIX: &str = "srt:session:";
/// Key setting penentu `srtEnabled` (`srt.js:57`).
const KEY_ENABLED: &str = "srtEnabled";

fn session_key(chat: &str) -> String {
    format!("{SESSION_PREFIX}{chat}")
}

/// `getSrtSession(chat)` (`srt.js:31-38`) — entri kedaluwarsa dibuang.
fn sesi(ctx: &Ctx) -> Option<Value> {
    let key = session_key(&ctx.chat);
    let v = ctx.session_get(&key)?;
    let ts = v.get("ts").and_then(Value::as_i64).unwrap_or(0);
    if crate::common::now_ms() - ts > TTL_MS {
        ctx.session_take(&key);
        return None;
    }
    Some(v)
}

/// Simpan sesi tangkapan (`srt.js:65`).
fn simpan_sesi(ctx: &Ctx, count: i64) {
    ctx.session_set(
        &session_key(&ctx.chat),
        serde_json::json!({
            "sender": ctx.sender,
            "count": count,
            "ts": crate::common::now_ms(),
        }),
    );
}

/// Ekstensi gambar yang dihitung (`srt.js:44`).
fn adalah_gambar(nama: &str) -> bool {
    nama.ends_with(".webp")
        || nama.ends_with(".jpg")
        || nama.ends_with(".png")
        || nama.ends_with(".jpeg")
}

/// `countShuffleImages()` (`srt.js:41-45`).
fn hitung_gambar(ctx: &Ctx) -> usize {
    ctx.berkas()
        .daftar(SHUFFLE_DIR)
        .unwrap_or_default()
        .into_iter()
        .filter(|f| adalah_gambar(f))
        .count()
}

/// Daftar file gambar yang tersimpan (`srt.js:80`).
fn daftar_gambar(ctx: &Ctx) -> Vec<String> {
    ctx.berkas()
        .daftar(SHUFFLE_DIR)
        .unwrap_or_default()
        .into_iter()
        .filter(|f| adalah_gambar(f))
        .collect()
}

/// `crypto.createHash("md5").update(buffer).digest("hex").substring(0, 10)`
/// (`srt.js:150`).
fn hash10(buffer: &[u8]) -> String {
    md5_hex(buffer)[..10].to_string()
}

// ---------------------------------------------------------------------------
// MD5 — implementasi RFC 1321 ringkas (crate `md5` tidak tersedia di crate
// `oktz-plugins`).
// ---------------------------------------------------------------------------

/// Tabel konstanta `K[i] = floor(abs(sin(i + 1)) * 2^32)`.
const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
    0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
    0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
    0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
    0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
    0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
    0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
    0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
    0xeb86d391,
];

/// Tabel pergeseran `S[i]`.
const MD5_S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
    9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
    15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// `crypto.createHash("md5").update(data).digest("hex")`.
fn md5_hex(data: &[u8]) -> String {
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_le_bytes());

    let (mut h0, mut h1, mut h2, mut h3) =
        (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);

    for chunk in msg.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            *word = u32::from_le_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        let (mut a, mut b, mut c, mut d) = (h0, h1, h2, h3);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let temp = d;
            d = c;
            c = b;
            let sum = a
                .wrapping_add(f)
                .wrapping_add(MD5_K[i])
                .wrapping_add(m[g]);
            b = b.wrapping_add(sum.rotate_left(MD5_S[i]));
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
    }

    let mut out = String::with_capacity(32);
    for v in [h0, h1, h2, h3] {
        for byte in v.to_le_bytes() {
            out.push_str(&format!("{byte:02x}"));
        }
    }
    out
}

#[plugin(
    name = "srt",
    alias = ["shufflereplythumb", "shufflereply"],
    category = "owner",
    description = "Sistem Shuffle Reply Thumb (SRT) untuk reply random image interaktif",
    usage = ".srt on",
    example = ".srt on",
    cooldown = 0,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn srt(ctx: Ctx) -> Reply {
    let action = ctx.args.first().map(|a| a.to_lowercase());

    let Some(action) = action.filter(|a| !a.is_empty()) else {
        return Reply::text(
            "🛠️ *SISTEM SHUFFLE REPLY THUMB (SRT)*\n\n\
             Selamat datang di menu pengelolaan gambar balasan otomatis. Sistem ini memungkinkan bot untuk membalas dengan gambar *thumbnail* yang diacak secara otomatis dari koleksi yang kamu simpan.\n\n\
             Berikut adalah daftar perintah yang tersedia:\n\
             - *.srt on* : Mengaktifkan fitur shuffle secara global.\n\
             - *.srt off* : Menonaktifkan fitur shuffle dan kembali ke pengaturan awal.\n\
             - *.srt c* : Membuka sesi tangkapan gambar untuk menambahkan koleksi baru ke dalam database.\n\
             - *.srt d* : Menutup sesi tangkapan gambar.\n\
             - *.srt list* : Menampilkan seluruh koleksi gambar yang telah tersimpan di dalam database bot.\n\
             - *.srt del* : Menghapus gambar dari database (Gunakan *.srt list* terlebih dahulu untuk memunculkan gambarnya, lalu *reply* salah satu gambar tersebut dengan perintah ini).",
        );
    };

    if action == "on" {
        return Reply::text(
            "✅ *FITUR SRT BERHASIL DIAKTIFKAN*\n\n\
             Fitur ini telah menyala secara global. Setiap balasan bot yang mendukung *thumbnail* sekarang akan menampilkan gambar secara acak dari dalam folder shuffle yang telah dikumpulkan.",
        )
        .with_mutation(Mutation::SetSetting {
            key: KEY_ENABLED.to_string(),
            value: Value::Bool(true),
        });
    }

    if action == "off" {
        return Reply::text(
            "❌ *FITUR SRT BERHASIL DINONAKTIFKAN*\n\n\
             Penggunaan *thumbnail* acak telah dimatikan. Semua balasan bot akan kembali menggunakan gambar *default* bawaan sistem.",
        )
        .with_mutation(Mutation::SetSetting {
            key: KEY_ENABLED.to_string(),
            value: Value::Bool(false),
        });
    }

    if action == "c" || action == "capture" {
        simpan_sesi(&ctx, 0);
        return Reply::text(format!(
            "📸 *SESI TANGKAPAN GAMBAR DIMULAI*\n\n\
             Silakan kirimkan gambar satu per satu secara terus menerus ke dalam obrolan ini. Bot akan membaca setiap gambar tersebut dan langsung menyimpannya secara otomatis ke dalam sistem *database shuffle*.\n\n\
             - Total gambar tersimpan saat ini: *{}*\n\
             - Jika semua gambar sudah selesai dikirimkan, hentikan sesi dengan perintah `{}srt d`.",
            hitung_gambar(&ctx),
            ctx.prefix
        ));
    }

    if action == "d" || action == "done" {
        let Some(s) = sesi(&ctx) else {
            return Reply::text(
                "❌ Kamu sedang tidak berada di dalam sesi penangkapan gambar aktif untuk saat ini.",
            );
        };
        let sender = s.get("sender").and_then(Value::as_str).unwrap_or("");
        if sender != ctx.sender {
            return Reply::text(
                "❌ Kamu sedang tidak berada di dalam sesi penangkapan gambar aktif untuk saat ini.",
            );
        }
        let count = s.get("count").and_then(Value::as_i64).unwrap_or(0);
        ctx.session_take(&session_key(&ctx.chat));
        return Reply::text(format!(
            "✅ *SESI TANGKAPAN GAMBAR SELESAI*\n\n\
             Sesi telah dihentikan dan seluruh gambar telah diproses.\n\
             - Total gambar baru yang ditambahkan: *{count}*\n\
             - Total keseluruhan gambar di sistem: *{}*",
            hitung_gambar(&ctx)
        ));
    }

    if action == "list" {
        if !ctx.berkas().ada(SHUFFLE_DIR) {
            return Reply::text(
                "❌ Belum ada satu pun gambar yang tersimpan di dalam direktori *shuffle*. Silakan lakukan penangkapan gambar terlebih dahulu.",
            );
        }
        let files = daftar_gambar(&ctx);
        if files.is_empty() {
            return Reply::text(
                "❌ Direktori *shuffle* masih kosong. Silakan gunakan perintah tangkapan gambar untuk mulai menambahkan.",
            );
        }
        let mut list = vec![Reply::text(format!(
            "📂 *DAFTAR GAMBAR SHUFFLE*\n\n\
             Sistem menemukan *{}* gambar yang tersimpan. Sedang memproses dan merangkai album untuk ditampilkan, harap tunggu sebentar.",
            files.len()
        ))];
        for f in files {
            list.push(Reply::image(
                MediaSource::path(format!("{SHUFFLE_DIR}/{f}")),
                "",
            ));
        }
        return Reply::sequence(list);
    }

    if action == "del" || action == "delete" {
        let is_image = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image)
            || ctx.quoted().is_some_and(|q| q.kind.is_image());
        if !is_image {
            return Reply::text(
                "❌ *CARA MENGHAPUS GAMBAR:*\n\n\
                 1. Ketik `.srt list` untuk menampilkan seluruh koleksi gambar.\n\
                 2. Balas (*reply*) salah satu gambar yang muncul dari album tersebut dengan perintah `.srt del`.",
            );
        }
        let buffer = if ctx.quoted().is_some() {
            ctx.download_quoted_media()
        } else {
            ctx.download_media()
        };
        let Ok(buffer) = buffer else {
            return Reply::text("❌ Gagal mendownload gambar untuk dihapus.");
        };
        let hash = hash10(&buffer);
        let kandidat = [
            format!("srt_{hash}.jpeg"),
            format!("srt_{hash}.webp"),
            format!("srt_{hash}.jpg"),
        ];
        let chosen = kandidat
            .iter()
            .find(|n| ctx.berkas().ada(&format!("{SHUFFLE_DIR}/{n}")))
            .cloned()
            .unwrap_or_else(|| kandidat[0].clone());
        let path = format!("{SHUFFLE_DIR}/{chosen}");
        return if ctx.berkas().ada(&path) {
            let _ = ctx.berkas().hapus(&path);
            Reply::text(
                "✅ *GAMBAR BERHASIL DIHAPUS*\n\n\
                 Gambar berhasil dihapus dari database shuffle.",
            )
            .with_reaction("✅")
        } else {
            Reply::text(
                "❌ *GAMBAR TIDAK DITEMUKAN*\n\n\
                 Gambar ini tidak cocok dengan yang ada di dalam database shuffle (atau sudah dihapus sebelumnya).",
            )
            .with_reaction("✅")
        }
    }

    Reply::text(format!(
        "❌ Perintah lanjutan \"{action}\" tidak dapat dikenali oleh sistem."
    ))
}

/// `srtAnswerHandler` (`srt.js:173-217`) — menangkap gambar selama sesi aktif.
#[answer_handler]
pub fn srt_answer(ctx: Ctx) -> Option<Reply> {
    if !ctx.command.is_empty() {
        return None;
    }
    let s = sesi(&ctx)?;
    if s.get("sender").and_then(Value::as_str).unwrap_or("") != ctx.sender {
        return None;
    }
    let is_image = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image)
        || ctx.quoted().is_some_and(|q| q.kind.is_image());
    if !is_image {
        return None;
    }

    let buffer = if ctx.quoted().is_some() {
        ctx.download_quoted_media()
    } else {
        ctx.download_media()
    };
    let Ok(buffer) = buffer else {
        return Some(
            Reply::text("❌ Terjadi kesalahan fatal saat mencoba mengunduh dan menyimpan gambar tersebut.")
                .with_reaction("❌"),
        );
    };

    let _ = ctx.berkas().buat_folder(SHUFFLE_DIR);
    let hash = hash10(&buffer);
    let path = format!("{SHUFFLE_DIR}/srt_{hash}.jpeg");
    if ctx.berkas().ada(&path) {
        return Some(Reply::text(
            "⚠️ Gambar yang sama terdeteksi telah tersimpan di dalam database.",
        ));
    }

    // `sharp(buffer).resize(1280, 720, { fit: 'inside', withoutEnlargement: false })
    // .jpeg({ quality: 85 })` (`srt.js:202`).
    let bytes = match oktz_plugin_api::gambar::sharp(&buffer)
        .ok()
        .map(|g| g.ubah_ukuran(1280, 720).pas_di_dalam().ke_jpeg(85))
    {
        Some(Ok(b)) => b,
        _ => buffer,
    };
    if ctx.berkas().tulis(&path, &bytes).is_err() {
        return Some(
            Reply::text("❌ Terjadi kesalahan fatal saat mencoba mengunduh dan menyimpan gambar tersebut.")
                .with_reaction("❌"),
        );
    }

    let count = s.get("count").and_then(Value::as_i64).unwrap_or(0) + 1;
    simpan_sesi(&ctx, count);

    Some(
        Reply::text(format!(
            "✅ *GAMBAR BERHASIL DISIMPAN*\n\n\
             Gambar telah diamankan ke dalam penyimpanan lokal bot.\n\
             - Total gambar ditambahkan pada sesi ini: *{count}*"
        ))
        .with_reaction("✅"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::BerkasPalsu;
    use oktz_plugin_api::find;
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(args: &[&str], sessions: oktz_plugin_api::SessionStore) -> Reply {
        srt(Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "srt".into(),
            sessions,
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("srt").unwrap();
        assert_eq!(p.meta.name, "srt");
        assert_eq!(p.meta.alias, ["shufflereplythumb", "shufflereply"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Sistem Shuffle Reply Thumb (SRT) untuk reply random image interaktif"
        );
        assert_eq!(p.meta.usage, ".srt on");
        assert_eq!(p.meta.example, ".srt on");
        assert_eq!(p.meta.cooldown, 0);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn md5_sesuai_standar() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            md5_hex(b"The quick brown fox jumps over the lazy dog"),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn hash10_ekor_md5() {
        assert_eq!(hash10(b"abc"), "900150983c");
    }

    #[test]
    fn tanpa_argumen_tampil_menu() {
        let r = run(&[], oktz_plugin_api::new_session_store());
        let t = teks(&r);
        assert!(t.starts_with("🛠️ *SISTEM SHUFFLE REPLY THUMB (SRT)*"), "{t}");
        assert!(t.contains("- *.srt on* : Mengaktifkan fitur shuffle secara global."), "{t}");
        assert!(t.contains("- *.srt list* : Menampilkan seluruh koleksi gambar"), "{t}");
    }

    #[test]
    fn on_menyimpan_setting() {
        let r = run(&["on"], oktz_plugin_api::new_session_store());
        assert_eq!(
            teks(&r),
            "✅ *FITUR SRT BERHASIL DIAKTIFKAN*\n\n\
             Fitur ini telah menyala secara global. Setiap balasan bot yang mendukung *thumbnail* sekarang akan menampilkan gambar secara acak dari dalam folder shuffle yang telah dikumpulkan."
        );
        assert!(r.mutations().iter().any(|m| matches!(
            m,
            Mutation::SetSetting { key, value } if key == KEY_ENABLED && *value == Value::Bool(true)
        )));
    }

    #[test]
    fn off_menyimpan_setting() {
        let r = run(&["off"], oktz_plugin_api::new_session_store());
        assert!(teks(&r).starts_with("❌ *FITUR SRT BERHASIL DINONAKTIFKAN*"));
        assert!(r.mutations().iter().any(|m| matches!(
            m,
            Mutation::SetSetting { key, value } if key == KEY_ENABLED && *value == Value::Bool(false)
        )));
    }

    #[test]
    fn capture_membuka_sesi_dan_done_menutupnya() {
        let s = oktz_plugin_api::new_session_store();
        let r = run(&["c"], s.clone());
        let t = teks(&r);
        assert!(t.starts_with("📸 *SESI TANGKAPAN GAMBAR DIMULAI*"), "{t}");
        assert!(t.contains("- Total gambar tersimpan saat ini: *0*"), "{t}");
        assert!(t.contains("hentikan sesi dengan perintah `.srt d`."), "{t}");
        assert!(s.lock().unwrap().contains_key("srt:session:628111@s.whatsapp.net"));

        let r = run(&["d"], s.clone());
        assert_eq!(
            teks(&r),
            "✅ *SESI TANGKAPAN GAMBAR SELESAI*\n\n\
             Sesi telah dihentikan dan seluruh gambar telah diproses.\n\
             - Total gambar baru yang ditambahkan: *0*\n\
             - Total keseluruhan gambar di sistem: *0*"
        );
        assert!(!s.lock().unwrap().contains_key("srt:session:628111@s.whatsapp.net"));
    }

    #[test]
    fn done_tanpa_sesi_ditolak() {
        let r = run(&["d"], oktz_plugin_api::new_session_store());
        assert_eq!(
            teks(&r),
            "❌ Kamu sedang tidak berada di dalam sesi penangkapan gambar aktif untuk saat ini."
        );
    }

    #[test]
    fn list_tanpa_folder_memberi_pesan() {
        let r = srt(Ctx {
            text: "list".into(),
            args: vec!["list".into()],
            command: "srt".into(),
            berkas: Arc::new(BerkasPalsu::baru()),
            ..Ctx::default()
        });
        assert_eq!(
            teks(&r),
            "❌ Belum ada satu pun gambar yang tersimpan di dalam direktori *shuffle*. Silakan lakukan penangkapan gambar terlebih dahulu."
        );
    }

    #[test]
    fn del_tanpa_gambar_memberi_panduan() {
        let r = run(&["del"], oktz_plugin_api::new_session_store());
        assert_eq!(
            teks(&r),
            "❌ *CARA MENGHAPUS GAMBAR:*\n\n\
             1. Ketik `.srt list` untuk menampilkan seluruh koleksi gambar.\n\
             2. Balas (*reply*) salah satu gambar yang muncul dari album tersebut dengan perintah `.srt del`."
        );
    }

    #[test]
    fn perintah_tidak_dikenal_ditolak() {
        let r = run(&["ngawur"], oktz_plugin_api::new_session_store());
        assert_eq!(
            teks(&r),
            "❌ Perintah lanjutan \"ngawur\" tidak dapat dikenali oleh sistem."
        );
    }
}
