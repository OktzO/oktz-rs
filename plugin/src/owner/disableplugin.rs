//! `plugins/owner/disableplugin.js` — menonaktifkan plugin tertentu.
//!
//! Yang dipindah ke sini:
//! - teks bantuan tanpa nama plugin (`disableplugin.js:57-65`).
//! - balasan "tidak ditemukan" (`:70`).
//! - balasan "sudah dinonaktifkan" (`:76`).
//!
//! `ponytail:` cabang terakhir JS menulis `isEnabled: true` → `false` di
//! berkas plugin sumber, lalu meminta restart/hot reload. Di Rust registry
//! plugin bersifat statis (linkme) — plugin didaftarkan saat compile dan
//! `PluginMeta::is_enabled` tidak dibaca router saat dispatch. Jadi plugin
//! tidak bisa dimatikan saat runtime; balasannya menjelaskan itu.
//!
//! `ponytail:` `ctx.berkas()` berakar ke direktori kerja, bukan folder
//! `plugins/` — jadi penulisan berkas plugin sumber juga tidak tersedia di
//! sini.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Teks bantuan (`disableplugin.js:57-65`).
fn bantuan(prefix: &str) -> String {
    format!(
        "🔌 *ᴅɪsᴀʙʟᴇ ᴘʟᴜɢɪɴ*\n\n\
         > Masukkan nama plugin yang ingin dinonaktifkan\n\n\
         *Contoh:*\n\
         > `{prefix}disableplugin sticker`\n\
         > `{prefix}disableplugin tiktok`",
        prefix = prefix
    )
}

#[plugin(
    name = "disableplugin",
    alias = ["dplugin", "plugindisable", "offplugin"],
    category = "owner",
    description = "Menonaktifkan plugin tertentu",
    usage = ".disableplugin <nama_plugin>",
    example = ".disableplugin sticker",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn disableplugin(ctx: Ctx) -> Reply {
    let plugin_name = ctx.args.first().map(|a| a.to_lowercase());

    let Some(plugin_name) = plugin_name.filter(|s| !s.is_empty()) else {
        return Reply::text(bantuan(&ctx.prefix));
    };

    let Some(found) = oktz_plugin_api::find(&plugin_name) else {
        return Reply::text(format!("❌ Plugin *{plugin_name}* tidak ditemukan!"));
    };

    if !found.meta.is_enabled {
        return Reply::text(format!(
            "⚠️ Plugin *{plugin_name}* sudah dinonaktifkan!"
        ));
    }

    // `ponytail:` registry statis — plugin tidak bisa dimatikan saat runtime.
    Reply::text(
        "⚠️ *PENONAKTIFAN PLUGIN TIDAK TERSEDIA*\n\n\
         Registry plugin pada versi Rust bersifat statis (didaftarkan saat \
         compile), jadi plugin tidak bisa dimatikan saat runtime.\n\n\
         Set `is_enabled = false` pada `#[plugin(...)]` di \
         `plugin/src/owner/<file>.rs`, lalu build ulang bot.\n\n\
         Contoh JS: `.disableplugin sticker`",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(args: &[&str]) -> Reply {
        disableplugin(Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "disableplugin".into(),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("disableplugin").unwrap();
        assert_eq!(p.meta.name, "disableplugin");
        assert_eq!(p.meta.alias, ["dplugin", "plugindisable", "offplugin"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Menonaktifkan plugin tertentu");
        assert_eq!(p.meta.usage, ".disableplugin <nama_plugin>");
        assert_eq!(p.meta.example, ".disableplugin sticker");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["dplugin", "plugindisable", "offplugin"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("disableplugin"), "{a}");
        }
    }

    #[test]
    fn tanpa_nama_tampil_bantuan() {
        assert_eq!(
            teks(&run(&[])),
            "🔌 *ᴅɪsᴀʙʟᴇ ᴘʟᴜɢɪɴ*\n\n\
             > Masukkan nama plugin yang ingin dinonaktifkan\n\n\
             *Contoh:*\n\
             > `.disableplugin sticker`\n\
             > `.disableplugin tiktok`"
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
    fn plugin_ada_tidak_bisa_dimatikan_saat_runtime() {
        assert!(
            teks(&run(&["star"])).contains("PENONAKTIFAN PLUGIN TIDAK TERSEDIA"),
            "{}",
            teks(&run(&["star"]))
        );
    }
}
