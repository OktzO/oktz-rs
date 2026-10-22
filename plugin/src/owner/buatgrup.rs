//! `plugins/owner/buatgrup.js` — buat grup baru.
//!
//! ## Penyimpangan
//!
//! - `sock.groupCreate(subject, participants)` ada padanannya di
//!   `Sock::buat_grup`, jadi pembuatan grup ikut dip-port.
//! - Penghapusan otomatis setelah durasi habis (`setTimeout` +
//!   `groupParticipantsUpdate` + `groupLeave`, `buatgrup.js:79-99`) tidak
//!   bisa ditiru: `Reply` hanya kirim ke chat pemicu dan tidak ada
//!   penjadwal di API plugin. Balasan sukses tetap menyebut masa aktif,
//!   seperti di JS.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Teks bantuan `buatgrup.js:19-32`.
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "👥 *BUAT GRUP BARU* 👥\n\n\
         Halo kak Owner! Mau bikin grup baru secara instan?\n\n\
         *Cara Pakai:*\n\
         👉 `{prefix}buatgrup Nama Grup | 628xxx,628yyy | Durasi(menit)`\n\n\
         *Detail:*\n\
         • Gunakan `|` untuk memisahkan nama, peserta, dan durasi\n\
         • Pisahkan nomor peserta dengan koma\n\
         • Jika durasi diisi, bot akan menendang semua member dan menghapus grup saat waktu habis!\n\
         • Bot otomatis menjadi admin\n\n\
         *Contoh Tanpa Durasi:*\n\
         `{prefix}buatgrup Tim Alpha | 628123,628456`\n\n\
         *Contoh Dengan Durasi (Masa Aktif 60 Menit):*\n\
         `{prefix}buatgrup Tim Beta | 628123,628456 | 60`"
    )
}

/// `buatgrup.js:40`.
pub(crate) const NAMA_KEPENDEKAN: &str =
    "❌ Waduh kak, nama grupnya kependekan! Minimal 2 karakter ya.";

/// `buatgrup.js:50`.
pub(crate) const TANPA_PESERTA: &str =
    "❌ Lho kak, nomor pesertanya mana? Masukkan minimal 1 nomor ya.";

/// `buatgrup.js:104` — gagal membuat grup.
pub(crate) fn gagal_gagal(pesan: &str) -> String {
    format!("❌ Maaf kak, gagal membuat grup! 😭\nError: {pesan}")
}

/// Pesan sukses `buatgrup.js:66-76`.
pub(crate) fn sukses(nama: &str, id: &str, jumlah_peserta: usize, durasi_menit: Option<i64>) -> String {
    let mut out = format!(
        "👥 *GRUP BERHASIL DIBUAT* 👥\n\n\
         ✨ *Nama:* {nama}\n\
         🆔 *ID:* {id}\n\
         👤 *Peserta:* {jumlah_peserta} orang\n"
    );
    if let Some(menit) = durasi_menit {
        out.push_str(&format!(
            "⏳ *Masa Aktif:* {menit} Menit\n\
             \n⚠️ _Grup ini akan otomatis dihapus dan semua member akan dikeluarkan saat masa aktif habis!_\n"
        ));
    }
    out.push_str("\n_Bot otomatis menjadi admin grup ini ya kak!");
    out
}

/// `participantsStr.split(/[,;\s]+/)` lalu hanya angka ≥ 5 digit
/// (`buatgrup.js:43-47`).
pub(crate) fn parse_peserta(teks: &str) -> Vec<String> {
    let mut hasil = Vec::new();
    for bagian in teks.split(|c: char| c == ',' || c == ';' || c.is_whitespace()) {
        let angka: String = bagian.chars().filter(char::is_ascii_digit).collect();
        if angka.chars().count() >= 5 {
            hasil.push(format!("{angka}@s.whatsapp.net"));
        }
    }
    hasil
}

#[plugin(
    name = "buatgrup",
    alias = ["creategroup", "newgroup"],
    category = "owner",
    description = "Buat grup baru",
    usage = ".buatgrup <nama>|<nomor1,nomor2,...>|<durasi_menit>",
    example = ".buatgrup Grup Baru|628xxx,628yyy|60",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn buatgrup(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    let args: Vec<&str> = text.split('|').collect();

    if args.len() < 2 {
        return Reply::text(bantuan(&ctx.prefix));
    }

    let nama = args[0].trim();
    let peserta_teks = args[1].trim();
    let durasi_teks = args.get(2).map(|s| s.trim()).unwrap_or("");

    if nama.chars().count() < 2 {
        return Reply::text(NAMA_KEPENDEKAN);
    }

    let peserta = parse_peserta(peserta_teks);
    if peserta.is_empty() {
        return Reply::text(TANPA_PESERTA);
    }

    // `parseInt(durationStr.replace(/[^0-9]/g, ''))` (`buatgrup.js:56`).
    let durasi_menit = durasi_teks
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse::<i64>()
        .ok()
        .filter(|d| *d > 0);

    let hasil = ctx.sock().buat_grup(nama, &peserta);
    let Ok(grup) = hasil else {
        return Reply::text(gagal_gagal("gagal membuat grup")).with_reaction("❌");
    };

    let id = grup
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    Reply::text(sukses(nama, &id, peserta.len(), durasi_menit)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "buatgrup".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        buatgrup(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("buatgrup").unwrap();
        assert_eq!(p.meta.name, "buatgrup");
        assert_eq!(p.meta.alias, ["creategroup", "newgroup"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Buat grup baru");
        assert_eq!(
            p.meta.usage,
            ".buatgrup <nama>|<nomor1,nomor2,...>|<durasi_menit>"
        );
        assert_eq!(p.meta.example, ".buatgrup Grup Baru|628xxx,628yyy|60");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_pemisah_menampilkan_panduan() {
        let r = run("Grup Baru");
        assert_eq!(
            teks(&r),
            "👥 *BUAT GRUP BARU* 👥\n\n\
             Halo kak Owner! Mau bikin grup baru secara instan?\n\n\
             *Cara Pakai:*\n\
             👉 `.buatgrup Nama Grup | 628xxx,628yyy | Durasi(menit)`\n\n\
             *Detail:*\n\
             • Gunakan `|` untuk memisahkan nama, peserta, dan durasi\n\
             • Pisahkan nomor peserta dengan koma\n\
             • Jika durasi diisi, bot akan menendang semua member dan menghapus grup saat waktu habis!\n\
             • Bot otomatis menjadi admin\n\n\
             *Contoh Tanpa Durasi:*\n\
             `.buatgrup Tim Alpha | 628123,628456`\n\n\
             *Contoh Dengan Durasi (Masa Aktif 60 Menit):*\n\
             `.buatgrup Tim Beta | 628123,628456 | 60`"
        );
    }

    #[test]
    fn nama_terlalu_pendek_ditolak() {
        let r = run("A|628123");
        assert_eq!(teks(&r), NAMA_KEPENDEKAN);
    }

    #[test]
    fn tanpa_nomor_peserta_ditolak() {
        let r = run("Tim Alpha|123");
        assert_eq!(teks(&r), TANPA_PESERTA);
    }

    #[test]
    fn nomor_peserta_diparse_seperti_js() {
        assert_eq!(
            parse_peserta("628123,628456"),
            vec![
                "628123@s.whatsapp.net".to_string(),
                "628456@s.whatsapp.net".to_string()
            ]
        );
        assert_eq!(
            parse_peserta("+62 812-3456;628999"),
            vec![
                "8123456@s.whatsapp.net".to_string(),
                "628999@s.whatsapp.net".to_string()
            ]
        );
        assert!(parse_peserta("12,34").is_empty());
    }

    #[test]
    fn pesan_sesuai_dengan_pola_js() {
        let t = sukses("Tim Alpha", "120363@g.us", 2, Some(60));
        assert!(t.starts_with("👥 *GRUP BERHASIL DIBUAT* 👥"));
        assert!(t.contains("✨ *Nama:* Tim Alpha"));
        assert!(t.contains("🆔 *ID:* 120363@g.us"));
        assert!(t.contains("👤 *Peserta:* 2 orang"));
        assert!(t.contains("⏳ *Masa Aktif:* 60 Menit"));
        assert!(t.ends_with("_Bot otomatis menjadi admin grup ini ya kak!"));

        let tanpa_durasi = sukses("Tim Beta", "120363@g.us", 1, None);
        assert!(!tanpa_durasi.contains("Masa Aktif"));
    }
}
