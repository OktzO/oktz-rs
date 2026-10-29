//! `plugins/sticker/qc.js` — sticker quote chat dengan warna custom.
//!
//! Yang diport apa adanya:
//! - Tabel warna `COLORS` (`qc.js:21-68`) — dipakai untuk daftar warna di
//!   kartu pakai dan untuk validasi nama warna. Urutannya penting: JS
//!   menampilkan `Object.keys(COLORS).join(', ')` (`qc.js:74`), jadi urutan
//!   penyisipan objek itu yang tampil.
//! - Semua cabang teks: kartu pakai, warna tidak dikenal, teks kosong,
//!   teks kepanjangan (`qc.js:73-107`).
//! - Reaksi penutup `✅` / `☢`.
//!
//! `ponytail:` render quote-nya (`qc.js:114-155`) memakai `@napi-rs/canvas`:
//! kanvas 512x512 diisi warna latar, teks putih dibungkus baris (lebar maksimal
//! 460px, font 36px), lalu `~ <nama>` di pojok kanan bawah. Tidak ada
//! renderer teks di Rust dan dependensi repo tidak boleh diubah, jadi jalur
//! setelah guard berakhir di cabang `catch` JS: `error_template` + `☢`
//! (`qc.js:165-167`).

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `qc.js:21-68` — nama warna → hex. Urutan = urutan penyisipan di JS.
pub(crate) const WARNA: [(&str, &str); 46] = [
    ("pink", "#f68ac9"),
    ("blue", "#6cace4"),
    ("red", "#f44336"),
    ("green", "#4caf50"),
    ("yellow", "#ffeb3b"),
    ("purple", "#9c27b0"),
    ("darkblue", "#0d47a1"),
    ("lightblue", "#03a9f4"),
    ("ash", "#9e9e9e"),
    ("orange", "#ff9800"),
    ("black", "#000000"),
    ("white", "#ffffff"),
    ("teal", "#008080"),
    ("lightpink", "#FFC0CB"),
    ("chocolate", "#A52A2A"),
    ("salmon", "#FFA07A"),
    ("magenta", "#FF00FF"),
    ("tan", "#D2B48C"),
    ("wheat", "#F5DEB3"),
    ("deeppink", "#FF1493"),
    ("fire", "#B22222"),
    ("skyblue", "#00BFFF"),
    ("brightskyblue", "#1E90FF"),
    ("hotpink", "#FF69B4"),
    ("lightskyblue", "#87CEEB"),
    ("seagreen", "#20B2AA"),
    ("darkred", "#8B0000"),
    ("orangered", "#FF4500"),
    ("cyan", "#48D1CC"),
    ("violet", "#BA55D3"),
    ("mossgreen", "#00FF7F"),
    ("darkgreen", "#008000"),
    ("navyblue", "#191970"),
    ("darkorange", "#FF8C00"),
    ("darkpurple", "#9400D3"),
    ("fuchsia", "#FF00FF"),
    ("darkmagenta", "#8B008B"),
    ("darkgray", "#2F4F4F"),
    ("peachpuff", "#FFDAB9"),
    ("darkishgreen", "#BDB76B"),
    ("darkishred", "#DC143C"),
    ("goldenrod", "#DAA520"),
    ("darkishgray", "#696969"),
    ("darkishpurple", "#483D8B"),
    ("gold", "#FFD700"),
    ("silver", "#C0C0C0"),
];

/// `qc.js:74` — `Object.keys(COLORS).join(', ')`.
pub(crate) fn daftar_warna() -> String {
    WARNA
        .iter()
        .map(|(nama, _)| *nama)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `qc.js:74-85` — kartu pakai.
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "💬 *ǫᴜᴏᴛᴇ sᴛɪᴄᴋᴇʀ*\n\n\
         ╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ* 」\n\
         ┃ ◦ `{prefix}qc <warna> <text>`\n\
         ┃ ◦ Reply pesan + `{prefix}qc <warna>`\n\
         ╰┈┈⬡\n\n\
         > Contoh: `{prefix}qc pink Hai semuanya!`\n\n\
         ╭┈┈⬡「 🎨 *ᴡᴀʀɴᴀ* 」\n\
         ┃ {}\n\
         ╰┈┈⬡",
        daftar_warna()
    )
}

/// `qc.js:92` — warna tidak ada di tabel.
pub(crate) fn warna_tidak_dikenal(warna: &str) -> String {
    format!(
        "❌ *ᴇʀʀᴏʀ*\n\n> Warna `{warna}` tidak ditemukan!\n> Gunakan salah satu warna yang tersedia."
    )
}

/// `qc.js:102` — teks quote kosong.
pub(crate) const TEKS_KOSONG: &str = "❌ *ᴇʀʀᴏʀ*\n\n> Masukkan text untuk quote!";

/// `qc.js:106` — teks lebih dari 80 karakter.
pub(crate) fn terlalu_panjang(panjang: usize) -> String {
    format!("❌ *ᴇʀʀᴏʀ*\n\n> Maksimal 80 karakter! (Saat ini: {panjang})")
}

/// `qc.js:88-89` — cari hex warna; `None` kalau nama warnanya tidak dikenal.
pub(crate) fn hex_warna(nama: &str) -> Option<&'static str> {
    WARNA
        .iter()
        .find(|(n, _)| *n == nama)
        .map(|(_, hex)| *hex)
}

#[plugin(
    name = "qc",
    alias = ["qcstc", "stcqc", "qcstic", "qcstick", "quotesticker"],
    category = "sticker",
    description = "Membuat sticker quote chat dengan warna custom",
    usage = ".qc <warna> <text>",
    example = ".qc pink Hai semuanya!",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn qc(ctx: Ctx) -> Reply {
    // `m.args.length < 2` (`qc.js:73`).
    if ctx.args.len() < 2 {
        return Reply::text(pakai(&ctx.prefix));
    }

    // `args[0].toLowerCase()` (`qc.js:88`).
    let warna = ctx.args[0].to_lowercase();
    if hex_warna(&warna).is_none() {
        return Reply::text(warna_tidak_dikenal(&warna));
    }

    // `args.slice(1).join(' ')` (`qc.js:95`).
    let mut pesan = ctx.args[1..].join(" ");

    // `if (m.quoted && !message) message = m.quoted.text || m.quoted.body || ''`
    // (`qc.js:97-99`).
    if pesan.is_empty() {
        if let Some(q) = ctx.quoted() {
            pesan = if q.text.is_empty() {
                String::new()
            } else {
                q.text.clone()
            };
        }
    }

    if pesan.is_empty() {
        return Reply::text(TEKS_KOSONG);
    }

    if pesan.chars().count() > 80 {
        return Reply::text(terlalu_panjang(pesan.chars().count()));
    }

    // Render quote butuh canvas — lihat `ponytail:` modul.
    Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::QuotedKind;

    fn ctx_of(args: &[&str], quoted_text: Option<&str>) -> Ctx {
        Ctx {
            text: args.join(" "),
            args: args.iter().map(|a| (*a).to_string()).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "qc".into(),
            quoted: quoted_text.map(|t| oktz_plugin_api::QuotedMessage {
                id: "MSG1".into(),
                sender: "628111@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: t.into(),
                kind: QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            timestamp: 0,
            }),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or("").to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        // `qc.js:92` — warna tidak dikenal.
        assert_eq!(
            warna_tidak_dikenal("pinkk"),
            "❌ *ᴇʀʀᴏʀ*\n\n> Warna `pinkk` tidak ditemukan!\n> Gunakan salah satu warna yang tersedia."
        );
        // `qc.js:102` — teks kosong.
        assert_eq!(TEKS_KOSONG, "❌ *ᴇʀʀᴏʀ*\n\n> Masukkan text untuk quote!");
        // `qc.js:106` — terlalu panjang.
        assert_eq!(
            terlalu_panjang(81),
            "❌ *ᴇʀʀᴏʀ*\n\n> Maksimal 80 karakter! (Saat ini: 81)"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("qc").unwrap();
        assert_eq!(p.meta.name, "qc");
        assert_eq!(
            p.meta.alias,
            ["qcstc", "stcqc", "qcstic", "qcstick", "quotesticker"]
        );
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(
            p.meta.description,
            "Membuat sticker quote chat dengan warna custom"
        );
        assert_eq!(p.meta.usage, ".qc <warna> <text>");
        assert_eq!(p.meta.example, ".qc pink Hai semuanya!");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_plugin_ini() {
        for a in ["qcstc", "stcqc", "qcstic", "qcstick", "quotesticker"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "qc");
        }
    }

    #[test]
    fn daftar_warna_ikut_urutan_js() {
        let daftar = daftar_warna();
        assert!(daftar.starts_with("pink, blue, red, green, yellow, purple, darkblue"));
        assert!(daftar.ends_with("darkishpurple, gold, silver"));
        assert_eq!(WARNA.len(), 46);
    }

    #[test]
    fn argumen_kurang_dari_dua_menampilkan_kartu_pakai() {
        let r = qc(ctx_of(&[], None));
        assert_eq!(teks(&r), pakai("."));
        assert_eq!(reaksi(&r), None);
        assert!(teks(&r).contains("┃ pink, blue, red,"), "{}", teks(&r));
    }

    #[test]
    fn warna_tak_dikenal_menampilkan_pesan_error() {
        let r = qc(ctx_of(&["merah", "halo"], None));
        assert_eq!(teks(&r), warna_tidak_dikenal("merah"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn warna_kecil_huruf_dikenal() {
        assert_eq!(hex_warna("pink"), Some("#f68ac9"));
        assert_eq!(hex_warna("PINK"), None);
        assert_eq!(hex_warna("tidak-ada"), None);
    }

    #[test]
    fn tanpa_teks_dan_tanpa_reply_menampilkan_teks_kosong() {
        let r = qc(ctx_of(&["pink", ""], None));
        assert_eq!(teks(&r), TEKS_KOSONG);
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn teks_dari_pesan_yang_direply_dipakai() {
        let r = qc(ctx_of(&["pink", ""], Some("halo dunia")));
        // Lanjut ke render (canvas) → cabang catch.
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "qc"));
        assert_eq!(reaksi(&r), Some("☢"));
    }

    #[test]
    fn teks_lebih_dari_80_karakter_ditolak() {
        let panjang = "a".repeat(81);
        let r = qc(ctx_of(&["pink", &panjang], None));
        assert_eq!(teks(&r), terlalu_panjang(81));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn teks_tepat_80_karakter_lolos() {
        let panjang = "a".repeat(80);
        let r = qc(ctx_of(&["pink", &panjang], None));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "qc"));
    }
}
