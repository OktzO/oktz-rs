//! `plugins/owner/buatsaluran.js` — buat saluran/newsletter baru.
//!
//! Port dari nol. Yang dipindah:
//! - pemecahan `nama|deskripsi` (`buatsaluran.js:16-25`).
//! - guard nama terlalu pendek + teks contoh (`:27-36`).
//! - teks sukses (`:43-50`) dan teks gagal (`:52`).
//!
//! `ponytail:` `sock.newsletterCreate(name, description)` tidak ada di
//! `oktz_plugin_api::sock::Sock` — API itu hanya punya operasi grup dan
//! kontak. Jadi pembuatan saluran selalu jatuh ke cabang `catch` JS, dan
//! pesannya menjelaskan alasannya.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Nama minimal (`:27`).
const MIN_NAMA: usize = 2;

/// Teks contoh saat nama kurang dari 2 karakter (`buatsaluran.js:28-35`).
fn contoh(prefix: &str) -> String {
    format!(
        "📢 *ʙᴜᴀᴛ sᴀʟᴜʀᴀɴ*\n\n\
         > `{prefix}buatsaluran Nama Saluran`\n\
         > `{prefix}buatsaluran Nama|Deskripsi`\n\n\
         📝 Contoh:\n\
         > `{prefix}buatsaluran Info Bot`\n\
         > `{prefix}buatsaluran Info Bot|Update terbaru bot kami`"
    )
}

#[plugin(
    name = "buatsaluran",
    commands = [
        "createsaluran", "createnewsletter",
    ],
    alias = [],
    category = "owner",
    description = "Buat saluran/newsletter baru",
    usage = ".buatsaluran <nama>|<deskripsi>",
    example = ".buatsaluran Info Bot|Update terbaru bot kami",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn buatsaluran(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    let (name, description) = match text.find('|') {
        None => (text.to_string(), String::new()),
        Some(idx) => (
            text[..idx].trim().to_string(),
            text[idx + 1..].trim().to_string(),
        ),
    };

    if name.chars().count() < MIN_NAMA {
        return Reply::text(contoh(&ctx.prefix));
    }

    // `sock.newsletterCreate(...)` tidak tersedia — cabang `catch` JS
    // (`buatsaluran.js:51-53`).
    Reply::text(format!(
        "❌ Gagal membuat saluran: \
         fitur pembuatan saluran belum tersedia di versi Rust \
         (butuh `sock.newsletterCreate`). \
         Nama yang diminta: {name}{}",
        if description.is_empty() {
            String::new()
        } else {
            format!(" | {description}")
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(teks: &str) -> Reply {
        buatsaluran(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "buatsaluran".into(),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("buatsaluran").unwrap();
        assert_eq!(p.meta.name, "buatsaluran");
        assert_eq!(p.meta.commands, ["createsaluran", "createnewsletter"]);
        assert!(p.meta.alias.is_empty(), "alias JS: []");
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Buat saluran/newsletter baru");
        assert_eq!(p.meta.usage, ".buatsaluran <nama>|<deskripsi>");
        assert_eq!(
            p.meta.example,
            ".buatsaluran Info Bot|Update terbaru bot kami"
        );
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn nama_kurang_dari_dua_huruf_tampil_contoh() {
        let t = teks(&run("a"));
        assert_eq!(
            t,
            "📢 *ʙᴜᴀᴛ sᴀʟᴜʀᴀɴ*\n\n\
             > `.buatsaluran Nama Saluran`\n\
             > `.buatsaluran Nama|Deskripsi`\n\n\
             📝 Contoh:\n\
             > `.buatsaluran Info Bot`\n\
             > `.buatsaluran Info Bot|Update terbaru bot kami`"
        );
    }

    #[test]
    fn nama_kosong_tampil_contoh() {
        assert!(teks(&run("")).starts_with("📢 *ʙᴜᴀᴛ sᴀʟᴜʀᴀɴ*"));
    }

    #[test]
    fn nama_valid_gagal_dengan_pesan_javascript() {
        let t = teks(&run("Info Bot|Update terbaru bot kami"));
        assert!(t.starts_with("❌ Gagal membuat saluran:"), "{t}");
        assert!(t.contains("Info Bot | Update terbaru bot kami"), "{t}");
    }

    #[test]
    fn prefix_dari_ctx_dipakai_di_contoh() {
        let r = buatsaluran(Ctx {
            text: "x".into(),
            prefix: "!".into(),
            command: "buatsaluran".into(),
            ..Ctx::default()
        });
        assert!(teks(&r).contains("> `!buatsaluran Nama Saluran`"), "{}", teks(&r));
    }
}
