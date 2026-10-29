//! `plugins/owner/join.js` — bot join ke grup lewat link invite.
//!
//! ## Penyimpangan
//!
//! - Cek "bot sudah jadi member" sebelum_accept invite memakai
//!   `sock.user.id` (JID bot sendiri) yang tidak ada di `Ctx`. Karena itu
//!   pemeriksaan itu dilewati; bila grup sudah diikuti, error dari
//!   `groupAcceptInvite` dipetakan dengan aturan yang sama seperti
//!   `catch` di JS (`join.js:69-73`).
//! - `contextInfo` surat berita pada balasan sukses tidak ada di `Reply`
//!   teks biasa.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

/// Prefix link invite yang dikenali (`join.js:24-27`).
const PREFIX: [&str; 2] = ["chat.whatsapp.com/", "invite.whatsapp.com/"];

/// Panjang minimum kode invite (`{20,}`).
const MIN_KODE: usize = 20;

/// Teks bantuan `join.js:87-97`.
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "🔗 *Join Grup*\n\n\
         Bot akan join ke grup berdasarkan link invite yang kamu berikan.\n\n\
         *PENGGUNAAN:*\n\
         > *{prefix}join <link>* — Join via link langsung\n\
         > *{prefix}join* (reply pesan) — Join dari link di pesan yang di-reply\n\n\
         *CONTOH:*\n\
         > *{prefix}join https://chat.whatsapp.com/xxx*\n\
         > Reply pesan berisi link lalu ketik *{prefix}join*\n\n\
         _Bot akan mendeteksi semua link grup di pesan dan join satu per satu_"
    )
}

/// `join.js:102-109`.
pub(crate) fn tanpa_link() -> String {
    String::from(
        "❌ *Tidak Ada Link Grup*\n\n\
         > Bot tidak menemukan link invite grup di pesan tersebut.\n\n\
         *Format link yang didukung:*\n\
         > *https://chat.whatsapp.com/xxx*\n\
         > *https://invite.whatsapp.com/xxx*",
    )
}

/// `join.js:120-121`.
pub(crate) fn sudah_member(subjek: &str) -> String {
    format!("❌ *Sudah Menjadi Member*\n\n> Bot sudah join di grup *{subjek}*")
}

/// `join.js:126`.
pub(crate) fn gagal_join(pesan: &str) -> String {
    format!("❌ *Gagal Join*\n\n> {pesan}")
}

/// `join.js:131-136`.
pub(crate) fn berhasil(subjek: &str, member: i64, owner: &str) -> String {
    format!(
        "✅ *Berhasil Join!*\n\n\
         > 🏠 Nama: *{subjek}*\n\
         > 👥 Member: *{member}*\n\
         > 👤 Owner: *{owner}*"
    )
}

/// `extractAllInviteCodes` (`join.js:19-41`) — semua kode invite yang
/// ditemukan, unik dan berurutan kemunculan.
pub(crate) fn kode_invite(teks: &str) -> Vec<String> {
    let lower = teks.to_lowercase();
    let mut hasil: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < lower.len() {
        let Some(prefix) = PREFIX.iter().find(|p| lower[i..].starts_with(**p)) else {
            i += 1;
            continue;
        };
        let start = i + prefix.len();
        let mut j = start;
        let byte = lower.as_bytes();
        while j < byte.len() && (byte[j].is_ascii_alphanumeric()) {
            j += 1;
        }
        let kode = &teks[start..j];
        if kode.chars().count() >= MIN_KODE && !hasil.iter().any(|k| k == kode) {
            hasil.push(kode.to_string());
        }
        i = j.max(i + 1);
    }
    hasil
}

/// Petakan pesan error BAileys ke teks yang sama dengan JS (`join.js:69-73`).
pub(crate) fn petakan_error(pesan: &str) -> String {
    if pesan.contains("not-authorized") {
        "Link sudah tidak valid atau expired".to_string()
    } else if pesan.contains("gone")
        || pesan.contains("item-not-found")
        || pesan.contains("404")
    {
        "Grup tidak ditemukan (link ngasal/sudah direvoke)".to_string()
    } else if pesan.contains("conflict") {
        "Bot sudah menjadi member".to_string()
    } else {
        "Link tidak valid atau bot dilarang join".to_string()
    }
}

/// Hasil percobaan join satu grup.
struct HasilJoin {
    sukses: bool,
    subjek: String,
    member: i64,
    owner: String,
    error: String,
}

/// `joinGroup` (`join.js:43-76`).
fn join_grup(ctx: &Ctx, kode: &str) -> HasilJoin {
    let Ok(info) = ctx.sock().info_undangan(kode) else {
        return HasilJoin {
            sukses: false,
            subjek: String::new(),
            member: 0,
            owner: String::new(),
            error: "Tidak dapat mengambil info grup".to_string(),
        };
    };
    let subjek = info
        .get("subject")
        .and_then(Value::as_str)
        .unwrap_or("Unknown")
        .to_string();
    let member = info
        .get("size")
        .and_then(Value::as_i64)
        .or_else(|| {
            info.get("participants")
                .and_then(Value::as_array)
                .map(|a| a.len() as i64)
        })
        .unwrap_or(0);
    let owner = info
        .get("owner")
        .and_then(Value::as_str)
        .map(|s| s.split('@').next().unwrap_or(s).to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    match ctx.sock().terima_undangan(kode) {
        Ok(_) => HasilJoin {
            sukses: true,
            subjek,
            member,
            owner,
            error: String::new(),
        },
        Err(e) => HasilJoin {
            sukses: false,
            subjek,
            member,
            owner,
            error: petakan_error(&e.pesan),
        },
    }
}

#[plugin(
    name = "join",
    alias = ["joingrup", "joingroup", "gabung"],
    category = "owner",
    description = "Bot join ke grup via link invite, support reply pesan yang mengandung link",
    usage = ".join <link> / .join (reply pesan berisi link)",
    example = ".join https://chat.whatsapp.com/xxx",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn join(ctx: Ctx) -> Reply {
    // `m.args.join(" ").trim()` (`join.js:79`).
    let input = ctx.args.join(" ").trim().to_string();
    let mut sumber = input;
    if sumber.is_empty() {
        if let Some(q) = ctx.quoted() {
            sumber = q.text.clone();
        }
    }

    if sumber.is_empty() {
        return Reply::text(bantuan(&ctx.prefix));
    }

    let kode_kode = kode_invite(&sumber);
    if kode_kode.is_empty() {
        return Reply::text(tanpa_link());
    }

    if kode_kode.len() == 1 {
        let hasil = join_grup(&ctx, &kode_kode[0]);
        if !hasil.sukses {
            return Reply::text(gagal_join(&hasil.error)).with_reaction("❌");
        }
        return Reply::text(berhasil(&hasil.subjek, hasil.member, &hasil.owner))
            .with_reaction("✅");
    }

    let mut teks = format!(
        "🔗 *Multi Join — {} Link Terdeteksi*\n\n\
         Bot akan join ke semua grup satu per satu.\n\n",
        kode_kode.len()
    );
    let mut sukses = 0usize;
    let mut sudah = 0usize;
    let mut gagal = 0usize;
    for (i, kode) in kode_kode.iter().enumerate() {
        let hasil = join_grup(&ctx, kode);
        if hasil.sukses {
            sukses += 1;
            teks.push_str(&format!(
                "*.{}* {} — ✅ Berhasil join\n",
                i + 1,
                hasil.subjek
            ));
        } else if hasil.error == "Bot sudah menjadi member" {
            sudah += 1;
            teks.push_str(&format!("*.{}* {} — ⚠️ Sudah member\n", i + 1, hasil.subjek));
        } else {
            gagal += 1;
            let potong: String = kode.chars().take(12).collect();
            teks.push_str(&format!("*.{}* {potong}... — ❌ {}\n", i + 1, hasil.error));
        }
    }
    teks.push_str(&format!(
        "\n*Hasil:*\n\
         > ✅ Berhasil: *{sukses}*\n\
         > ⚠️ Sudah member: *{sudah}*\n\
         > ❌ Gagal: *{gagal}*\n\
         > 📊 Total: *{}*",
        kode_kode.len()
    ));
    Reply::text(teks).with_reaction(if sukses > 0 { "✅" } else { "❌" })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, quoted: Option<&str>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "join".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            quoted: quoted.map(|t| oktz_plugin_api::QuotedMessage {
                id: "q1".into(),
                sender: "628999@s.whatsapp.net".into(),
                chat: "628111@s.whatsapp.net".into(),
                text: t.into(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            timestamp: 0,
            }),
            ..Default::default()
        };
        join(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("join").unwrap();
        assert_eq!(p.meta.name, "join");
        assert_eq!(p.meta.alias, ["joingrup", "joingroup", "gabung"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Bot join ke grup via link invite, support reply pesan yang mengandung link"
        );
        assert_eq!(p.meta.usage, ".join <link> / .join (reply pesan berisi link)");
        assert_eq!(p.meta.example, ".join https://chat.whatsapp.com/xxx");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_link_menampilkan_bantuan() {
        let r = run("", None);
        assert_eq!(
            teks(&r),
            "🔗 *Join Grup*\n\n\
             Bot akan join ke grup berdasarkan link invite yang kamu berikan.\n\n\
             *PENGGUNAAN:*\n\
             > *.join <link>* — Join via link langsung\n\
             > *.join* (reply pesan) — Join dari link di pesan yang di-reply\n\n\
             *CONTOH:*\n\
             > *.join https://chat.whatsapp.com/xxx*\n\
             > Reply pesan berisi link lalu ketik *.join*\n\n\
             _Bot akan mendeteksi semua link grup di pesan dan join satu per satu_"
        );
    }

    #[test]
    fn link_pendek_tidak_dikenali_sebagai_invite() {
        let r = run("https://chat.whatsapp.com/abc", None);
        assert_eq!(teks(&r), tanpa_link());
    }

    #[test]
    fn kode_invite_diekstraksi_dan_dideduplikasi() {
        let kode = "AbCdEfGhIjKlMnOpQrStUvWxYz012345";
        let teks_nya = format!("cek {kode} dan https://invite.whatsapp.com/{kode} lagi");
        let hasil = kode_invite(&teks_nya);
        assert_eq!(hasil.len(), 1);
        assert_eq!(hasil[0], kode);
    }

    #[test]
    fn kode_lebih_dari_20_huruf_diambil_sepenuhnya() {
        let kode = "abcdefghijklmnopqrstuvwxyz0123456789";
        let hasil = kode_invite(&format!("https://chat.whatsapp.com/{kode}"));
        assert_eq!(hasil, vec![kode.to_string()]);
    }

    #[test]
    fn pemetaan_error_sama_dengan_js() {
        assert_eq!(
            petakan_error("not-authorized"),
            "Link sudah tidak valid atau expired"
        );
        assert_eq!(
            petakan_error("item-not-found"),
            "Grup tidak ditemukan (link ngasal/sudah direvoke)"
        );
        assert_eq!(petakan_error("conflict"), "Bot sudah menjadi member");
        assert_eq!(
            petakan_error("lain-lain"),
            "Link tidak valid atau bot dilarang join"
        );
    }
}
