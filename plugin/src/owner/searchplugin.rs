//! `plugins/owner/searchplugin.js` — cari dan tampilkan info plugin.
//!
//! Di JS, `getAllPlugins()` mengembalikan peta plugin yang sudah dimuat
//! (nama + alias + config yang sudah digabung dengan `defaultConfig`).
//! Di Rust padanannya `oktz_plugin_api::find()` yang mencari nama dulu lalu
//! alias — urutan yang sama dengan `findPluginInfo` (`searchplugin.js:21-49`).
//!
//! `info.limit` tidak ada di `PluginMeta` Rust, jadi nilainya diambil dari
//! `defaultConfig` JS (`limit: 1`, `plugins.js:71`) agar angkanya sama.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `defaultConfig.limit` (`src/lib/plugins.js:71`).
const LIMIT_DEFAULT: i32 = 1;

/// Teks bantuan `searchplugin.js:87-93`.
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "🔍 *sᴇᴀʀᴄʜ ᴘʟᴜɢɪɴ*\n\n\
         > Cari dan tampilkan info plugin\n\n\
         *ᴄᴏɴᴛᴏʜ:*\n\
         > `{prefix}splugin sticker`\n\
         > `{prefix}splugin menu`"
    )
}

/// Plugin tidak ditemukan `searchplugin.js:107-110`.
pub(crate) fn tidak_ditemukan(nama: &str) -> String {
    format!("❌ *ᴛɪᴅᴀᴋ ᴅɪᴛᴇᴍᴜᴋᴀɴ*\n\n> Plugin `{nama}` tidak ditemukan")
}

/// `aliases` — `info.alias.join(", ")` atau `"-"` (`searchplugin.js:123-125`).
pub(crate) fn daftar_alias(alias: &[String]) -> String {
    if alias.is_empty() {
        "-".to_string()
    } else {
        alias.join(", ")
    }
}

/// `ya_tidak` — `info.isX ? "✅ Ya" : "❌ Tidak"`.
fn ya_tidak(v: bool) -> &'static str {
    if v { "✅ Ya" } else { "❌ Tidak" }
}

/// Kartu info plugin `searchplugin.js:133-151`.
pub(crate) fn kartu_info(
    nama: &str,
    alias: &[String],
    kategori: &str,
    deskripsi: &str,
    usage: &str,
    example: &str,
    is_enabled: bool,
    is_owner: bool,
    is_premium: bool,
    is_group: bool,
    is_admin: bool,
    cooldown: u32,
) -> String {
    format!(
        "📋 *ɪɴꜰᴏ ᴘʟᴜɢɪɴ*\n\n\
         ╭┈┈⬡「 📝 *ᴅᴇᴛᴀɪʟ* 」\n\
         ┃ 📛 ɴᴀᴍᴀ: `{nama}`\n\
         ┃ 🏷️ ᴀʟɪᴀs: `{alias}`\n\
         ┃ 📁 ᴄᴀᴛᴇɢᴏʀʏ: `{kategori}`\n\
         ┃ 📄 ᴅᴇsᴄ: {deskripsi}\n\
         ┃ 📝 ᴜsᴀɢᴇ: `{usage}`\n\
         ┃ 📌 ᴇxᴀᴍᴘʟᴇ: `{example}`\n\
         ╰┈┈⬡\n\n\
         ╭┈┈⬡「 ⚙️ *sᴇᴛᴛɪɴɢs* 」\n\
         ┃ 🔓 ᴇɴᴀʙʟᴇᴅ: {enabled}\n\
         ┃ 👑 ᴏᴡɴᴇʀ ᴏɴʟʏ: {owner}\n\
         ┃ 💎 ᴘʀᴇᴍɪᴜᴍ: {premium}\n\
         ┃ 👥 ɢʀᴏᴜᴘ ᴏɴʟʏ: {group}\n\
         ┃ 🛡️ ᴀᴅᴍɪɴ ᴏɴʟʏ: {admin}\n\
         ┃ ⏱️ ᴄᴏᴏʟᴅᴏᴡɴ: `{cooldown}s`\n\
         ┃ 🎫 ʟɪᴍɪᴛ: `{limit}`\n\
         ╰┈┈⬡",
        alias = daftar_alias(alias),
        enabled = ya_tidak(is_enabled),
        owner = ya_tidak(is_owner),
        premium = ya_tidak(is_premium),
        group = ya_tidak(is_group),
        admin = ya_tidak(is_admin),
        limit = LIMIT_DEFAULT,
    )
}

#[plugin(
    name = "searchplugin",
    alias = ["splugin", "findplugin", "infoplugin"],
    category = "owner",
    description = "Cari dan tampilkan info plugin",
    usage = ".splugin <nama>",
    example = ".splugin sticker",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn searchplugin(ctx: Ctx) -> Reply {
    // `m.text?.trim()` (`searchplugin.js:84`).
    let name = ctx.text.trim();
    if name.is_empty() {
        return Reply::text(bantuan(&ctx.prefix));
    }

    let Some(entry) = oktz_plugin_api::find(name) else {
        return Reply::text(tidak_ditemukan(name)).with_reaction("❌");
    };
    let meta = &entry.meta;

    Reply::text(kartu_info(
        &meta.name,
        &meta.alias,
        &meta.category,
        &meta.description,
        &meta.usage,
        &meta.example,
        meta.is_enabled,
        meta.is_owner,
        meta.is_premium,
        meta.is_group,
        meta.is_admin,
        meta.cooldown,
    ))
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "searchplugin".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            is_owner: true,
            ..Default::default()
        };
        searchplugin(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("searchplugin").unwrap();
        assert_eq!(p.meta.name, "searchplugin");
        assert_eq!(p.meta.alias, ["splugin", "findplugin", "infoplugin"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Cari dan tampilkan info plugin");
        assert_eq!(p.meta.usage, ".splugin <nama>");
        assert_eq!(p.meta.example, ".splugin sticker");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_nama_menampilkan_bantuan() {
        for text in ["", "   "] {
            let r = run(text);
            assert_eq!(teks(&r), bantuan("."));
            assert!(r.mutations().is_empty());
        }
        assert!(bantuan(".").starts_with("🔍 *sᴇᴀʀᴄʜ ᴘʟᴜɢɪɴ*"));
        assert!(bantuan(".").contains("> `.splugin sticker`"));
    }

    #[test]
    fn plugin_tidak_ada_menampilkan_pesan_tidak_ditemukan() {
        let r = run("plugin-hantu-xyz");
        assert_eq!(teks(&r), "❌ *ᴛɪᴅᴀᴋ ᴅɪᴛᴇᴍᴜᴋᴀɴ*\n\n> Plugin `plugin-hantu-xyz` tidak ditemukan");
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn plugin_yang_ada_menampilkan_kartu_lengkap() {
        let r = run("searchplugin");
        let t = teks(&r);
        assert!(t.starts_with("📋 *ɪɴꜰᴏ ᴘʟᴜɢɪɴ*"), "{t}");
        assert!(t.contains("┃ 📛 ɴᴀᴍᴀ: `searchplugin`"), "{t}");
        assert!(t.contains("┃ 🏷️ ᴀʟɪᴀs: `splugin, findplugin, infoplugin`"), "{t}");
        assert!(t.contains("┃ 📁 ᴄᴀᴛᴇɢᴏʀʏ: `owner`"), "{t}");
        assert!(t.contains("┃ 📄 ᴅᴇsᴄ: Cari dan tampilkan info plugin"), "{t}");
        assert!(t.contains("┃ 📝 ᴜsᴀɢᴇ: `.splugin <nama>`"), "{t}");
        assert!(t.contains("┃ 📌 ᴇxᴀᴍᴘʟᴇ: `.splugin sticker`"), "{t}");
        assert!(t.contains("┃ 🔓 ᴇɴᴀʙʟᴇᴅ: ✅ Ya"), "{t}");
        assert!(t.contains("┃ 👑 ᴏᴡɴᴇʀ ᴏɴʟʏ: ✅ Ya"), "{t}");
        assert!(t.contains("┃ 💎 ᴘʀᴇᴍɪᴜᴍ: ❌ Tidak"), "{t}");
        assert!(t.contains("┃ 👥 ɢʀᴏᴜᴘ ᴏɴʟʏ: ❌ Tidak"), "{t}");
        assert!(t.contains("┃ 🛡️ ᴀᴅᴍɪɴ ᴏɴʟʏ: ❌ Tidak"), "{t}");
        assert!(t.contains("┃ ⏱️ ᴄᴏᴏʟᴅᴏᴡɴ: `5s`"), "{t}");
        assert!(t.contains("┃ 🎫 ʟɪᴍɪᴛ: `1`"), "{t}");
        assert_eq!(reaksi(&r), Some("✅"));
    }

    #[test]
    fn alias_juga_mencari_plugin_yang_sama() {
        let r = run("splugin");
        assert!(teks(&r).contains("ɴᴀᴍᴀ: `searchplugin`"), "{}", teks(&r));
    }

    #[test]
    fn alias_kosong_menampilkan_tanda_hyphen() {
        assert_eq!(daftar_alias(&[]), "-");
        assert_eq!(daftar_alias(&["a".into(), "b".into()]), "a, b");
    }
}
