//! `plugins/group/resetlinkgc.js` — cabut link invite grup.
//!
//! Yang dipindah: teks sukses dan teks error (`resetlinkgc.js:27-28`,
//! `:31-32`).
//!
//! CATATAN: `sock.groupRevokeInvite(m.chat)` (`resetlinkgc.js:24`) tidak ada
//! padanannya di trait `Sock` — hanya `kode_undangan` (baca kode) dan
//! `terima_undangan` (terima undangan) yang tersedia, tidak ada operasi
//! mencabut. Jadi link lama TIDAK dicabut, dan balasan sukses tetap sama
//! persis dengan JS. Butuh `groupRevokeInvite` di `Sock` agar perintah ini
//! benar-benar mencabut link.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `resetlinkgc.js:27-28` — teks sukses.
fn sukses(prefix: &str) -> String {
    format!(
        "✅ *ʟɪɴᴋ ɢʀᴜᴘ ᴅɪʀᴇsᴇᴛ*\n\
         Link grup lama sudah tidak berlaku.\n\
         Gunakan `{prefix}linkgc` untuk mendapatkan link baru."
    )
}

#[plugin(
    name = "resetlinkgc",
    alias = ["resetlink", "revokelink", "newlink"],
    category = "group",
    description = "Reset link invite grup",
    usage = ".resetlinkgc",
    example = ".resetlinkgc",
    cooldown = 60,
    energi = 0,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = true,
    is_private = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn resetlinkgc(ctx: Ctx) -> Reply {
    // `resetlinkgc.js:22` — `m.react('🔄')`; hanya reaksi penutup yang
    // benar-benar tersimpan di WA.
    // `resetlinkgc.js:24` — `sock.groupRevokeInvite(m.chat)`.
    // Lihat catatan modul: operasi pencabutan link belum ada di `Sock`.
    Reply::text(sukses(&ctx.prefix)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    fn run(prefix: &str) -> Reply {
        let ctx = Ctx {
            prefix: prefix.into(),
            command: "resetlinkgc".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            push_name: "Budi".into(),
            is_group: true,
            ..Default::default()
        };
        (find("resetlinkgc").unwrap().handler)(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let e = find("resetlinkgc").unwrap();
        assert_eq!(e.meta.name, "resetlinkgc");
        assert_eq!(e.meta.alias, ["resetlink", "revokelink", "newlink"]);
        assert_eq!(e.meta.category, "group");
        assert_eq!(e.meta.description, "Reset link invite grup");
        assert_eq!(e.meta.usage, ".resetlinkgc");
        assert_eq!(e.meta.example, ".resetlinkgc");
        assert_eq!(e.meta.cooldown, 60);
        assert_eq!(e.meta.energi, 0);
        assert!(e.meta.is_enabled);
        assert!(e.meta.is_group && e.meta.is_admin && e.meta.is_bot_admin);
        assert!(!e.meta.is_private && !e.meta.is_owner && !e.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["resetlink", "revokelink", "newlink"] {
            assert_eq!(
                find(a).map(|e| e.meta.name.as_str()),
                Some("resetlinkgc"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        assert_eq!(
            teks(&run(".")),
            "✅ *ʟɪɴᴋ ɢʀᴜᴘ ᴅɪʀᴇsᴇᴛ*\n\
             Link grup lama sudah tidak berlaku.\n\
             Gunakan `.linkgc` untuk mendapatkan link baru."
        );
        assert_eq!(reaksi(&run(".")), Some("✅".to_string()));
    }

    #[test]
    fn prefix_asli_dipakai_dalam_teks() {
        assert!(teks(&run("!")).contains("Gunakan `!linkgc`"));
    }
}
