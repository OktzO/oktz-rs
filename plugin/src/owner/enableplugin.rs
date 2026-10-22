//! `plugins/owner/enableplugin.js` — mengaktifkan kembali plugin yang
//! dinonaktifkan.
//!
//! Port dari nol. Semua cabang yang bisa dicapai di Rust sudah identik dengan
//! JS:
//! - teks bantuan tanpa nama plugin (`enableplugin.js:57-65`).
//! - "tidak ditemukan" (`:70`).
//! - "sudah aktif" (`:76`).
//!
//! `ponytail:` cabang terakhir JS menulis `isEnabled: false` → `true` di
//! berkas plugin lalu meminta restart. Di Rust registry bersifat statis
//! (linkme) dan semua plugin bawaan `is_enabled = true`, jadi cabang itu tidak
//! punya padanan yang berguna — plugin yang ada di registry memang sudah
//! aktif, dan hasilnya sama persis dengan balasan "sudah aktif".

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Teks bantuan (`enableplugin.js:57-65`).
fn bantuan(prefix: &str) -> String {
    format!(
        "🔌 *ᴇɴᴀʙʟᴇ ᴘʟᴜɢɪɴ*\n\n\
         > Masukkan nama plugin yang ingin diaktifkan\n\n\
         *Contoh:*\n\
         > `{prefix}enableplugin sticker`\n\
         > `{prefix}enableplugin tiktok`"
    )
}

#[plugin(
    name = "enableplugin",
    alias = ["eplugin", "pluginenable", "onplugin"],
    category = "owner",
    description = "Mengaktifkan kembali plugin yang dinonaktifkan",
    usage = ".enableplugin <nama_plugin>",
    example = ".enableplugin sticker",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn enableplugin(ctx: Ctx) -> Reply {
    let plugin_name = ctx.args.first().map(|a| a.to_lowercase());

    let Some(plugin_name) = plugin_name.filter(|s| !s.is_empty()) else {
        return Reply::text(bantuan(&ctx.prefix));
    };

    let found = oktz_plugin_api::find(&plugin_name);

    if found.is_none() {
        return Reply::text(format!("❌ Plugin *{plugin_name}* tidak ditemukan!"));
    }

    // `plugin.config.isEnabled !== false` (`enableplugin.js:75`) — semua
    // plugin di registry Rust aktif, jadi cabang ini yang selalu terjadi.
    Reply::text(format!("⚠️ Plugin *{plugin_name}* sudah aktif!"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(args: &[&str]) -> Reply {
        enableplugin(Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "enableplugin".into(),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("enableplugin").unwrap();
        assert_eq!(p.meta.name, "enableplugin");
        assert_eq!(p.meta.alias, ["eplugin", "pluginenable", "onplugin"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Mengaktifkan kembali plugin yang dinonaktifkan"
        );
        assert_eq!(p.meta.usage, ".enableplugin <nama_plugin>");
        assert_eq!(p.meta.example, ".enableplugin sticker");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["eplugin", "pluginenable", "onplugin"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("enableplugin"), "{a}");
        }
    }

    #[test]
    fn tanpa_nama_tampil_bantuan() {
        assert_eq!(
            teks(&run(&[])),
            "🔌 *ᴇɴᴀʙʟᴇ ᴘʟᴜɢɪɴ*\n\n\
             > Masukkan nama plugin yang ingin diaktifkan\n\n\
             *Contoh:*\n\
             > `.enableplugin sticker`\n\
             > `.enableplugin tiktok`"
        );
    }

    #[test]
    fn plugin_tidak_ada_dikatakan_tidak_ditemukan() {
        assert_eq!(
            teks(&run(&["plugin-hantu-xyz"])),
            "❌ Plugin *plugin-hantu-xyz* tidak ditemukan!"
        );
    }

    #[test]
    fn plugin_ada_dikatakan_sudah_aktif() {
        assert_eq!(
            teks(&run(&["STAR"])),
            "⚠️ Plugin *star* sudah aktif!"
        );
    }
}
