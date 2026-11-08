//! `plugins/owner/setmenu.js` — mengatur variant tampilan menu.
//!
//! ## Penyimpangan
//!
//! - `sock.sendButton(chat, bufferFoto, bodyText, m, { buttons })` dipetakan ke
//!   `Reply::interactive(..)` dengan header gambar dari `config.assets.foto`
//!   dan satu tombol `single_select`.
//! - `db.setting("menuVariant", id)` + `db.save()` dipetakan ke
//!   `Mutation::SetSetting`.
//!
//! Quirk JS yang disalin apa adanya: teks error menyebut `*v1* s/d *v7*`
//! walau `VARIANTS` punya `v8` (`setmenu.js:76`), dan default variant diambil
//! dari `config.ui.menuVariant` = 3 (`config.js:190`) — bukan literal `|| 2`
//! di `setmenu.js:89` yang cuma berlaku kalau `config.ui` tidak ada.

use oktz_plugin_api::registry::{CardHeader, CardList, CardRow, CardSection, InteractiveButton};
use oktz_plugin_api::{Ctx, MediaSource, Mutation, Reply, plugin};

/// `VARIANTS` (`setmenu.js:19-68`) — `(key, id, name, desc, emoji)`.
///
/// `v16` tambahan dari sesi ini: varian Rubby (menu utama ala
/// `Rubby-MD/plugins/main/menu.js`) yang tata letaknya ditulis di
/// `plugin/src/main/menu.rs` `varian_rubby`.
const VARIANTS: &[(&str, i64, &str, &str, &str)] = &[
    ("v1", 1, "BASIC", "", "🖼️"),
    ("v2", 2, "PREMIUM", "", "✅"),
    ("v3", 3, "PREMIUM", "", "✅"),
    ("v4", 4, "LV", "", "✅"),
    ("v5", 5, "LV 2", "", "✅"),
    ("v6", 6, "LOCATION", "Location Message dengan Buttons", "📍"),
    ("v7", 7, "RA NGERTI", "Location Message dengan Buttons", "📍"),
    ("v8", 8, "THUMBNAIL", "Thumbnail Image", "📍"),
    (
        "v16",
        16,
        "RUBBY",
        "Menu utama ala Rubby-MD (smallcaps + tombol kategori)",
        "🎀",
    ),
];

/// Kunci setting untuk variant menu (`setmenu.js:79`).
const SETTING_KEY: &str = "menuVariant";

/// `config.ui.menuVariant` (`config.js:190`) = 3.
const DEFAULT_VARIANT: i64 = 3;

/// `❌ *VARIANT TIDAK VALID*` (`setmenu.js:76`) — teks JS menyebut `v7`
/// padahal `v8` ada; quirk itu dibiarkan.
pub(crate) const TIDAK_VALID: &str =
    "❌ *VARIANT TIDAK VALID*\n\nGunakan: *v1* s/d *v7*";

/// Pesan sukses `setmenu.js:81-85`.
pub(crate) fn pesan_diubah(emoji: &str, id: i64, name: &str, desc: &str) -> String {
    format!("✅ *MENU VARIANT DIUBAH*\n\n{emoji} *V{id} — {name}*\n_{desc}_")
}

/// Nama variant aktif — `VARIANTS[`v${current}`]?.name || "Unknown"`.
pub(crate) fn nama_variant(id: i64) -> &'static str {
    VARIANTS
        .iter()
        .find(|(_, vid, _, _, _)| *vid == id)
        .map(|(_, _, name, _, _)| *name)
        .unwrap_or("Unknown")
}

/// Teks body kartu `setmenu.js:110-114` — gaya `setallmenu.js` (penjelasan
/// per variant, supaya tampilan kartu mirip allmenu).
pub(crate) fn body_teks(current: i64) -> String {
    format!(
        "🎨🖼️ *MENU VARIANT*\n\n\
Atur tampilan menu utama bot ketika user mengetik perintah menu 📋✨\n\
Variant aktif saat ini: *V{current} — {nama}* 🎯\n\n\
*PENJELASAN VARIANT:*\n\n\
- *V1 Basic* 🖼️ — Kartu menu dasar: header gambar + daftar kategori `single_select`.\n\n\
- *V2 Premium* ✅ — Kartu menu premium dengan header gambar + footer informatif.\n\n\
- *V3 Premium* ✅ — Bawaan produksi: kartu list menu + cuaca + tombol.\n\n\
- *V4 LV* ✅ — Kartu list menu varian 4.\n\n\
- *V5 LV 2* ✅ — Kartu list menu varian 5.\n\n\
- *V6 Location* 📍 — Kartu dengan header lokasi + tombol.\n\n\
- *V7 RA NGERTI* 📍 — Sama seperti V6, pesan lokasi + tombol.\n\n\
- *V8 Thumbnail* 📍 — Kartu thumbnail dengan pratinjau tautan.\n\n\
- *V16 Rubby* 🎀 — Menu utama ala Rubby-MD: smallcaps + tombol `single_select` kategori (`.menu <kategori>`) + tombol Owner.\n\n\
> Pilih variant menu dari tombol di bawah 👇",
        nama = nama_variant(current)
    )
}

/// Baris tombol `single_select` (`setmenu.js:91-99`).
pub(crate) fn baris(prefix: &str, current: i64) -> Vec<CardRow> {
    VARIANTS
        .iter()
        .map(|(key, id, name, desc, emoji)| {
            let mark = if *id == current { " ✓" } else { "" };
            CardRow::new(
                format!("{emoji} {}{mark} — {name}", key.to_uppercase()),
                format!("{prefix}setmenu {key}"),
            )
            .description(*desc)
        })
        .collect()
}

/// Variant aktif: `db.setting("menuVariant") || config.ui?.menuVariant`.
fn variant_aktif(ctx: &Ctx) -> i64 {
    match ctx.setting(SETTING_KEY) {
        Some(v) if crate::common::js_truthy(Some(v)) => v
            .as_i64()
            .or_else(|| v.as_f64().map(|f| f as i64))
            .unwrap_or(DEFAULT_VARIANT),
        _ => DEFAULT_VARIANT,
    }
}

#[plugin(
    name = "setmenu",
    alias = ["menuvariant", "menustyle"],
    category = "owner",
    description = "Mengatur variant tampilan menu",
    usage = ".setmenu <v1-v16>",
    example = ".setmenu v8",
    cooldown = 3,
    energi = 0,
    is_owner = true,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn setmenu(ctx: Ctx) -> Reply {
    // `const variant = args[0]?.toLowerCase()` (`setmenu.js:71-72`).
    let variant = ctx.args.first().map(|s| s.to_lowercase());

    if let Some(key) = variant.as_deref() {
        let Some((_, id, name, desc, emoji)) = VARIANTS.iter().find(|(k, ..)| *k == key) else {
            return Reply::text(TIDAK_VALID);
        };
        return Reply::text(pesan_diubah(emoji, *id, name, desc)).with_mutation(
            Mutation::SetSetting {
                key: SETTING_KEY.to_string(),
                value: serde_json::json!(id),
            },
        );
    }

    let current = variant_aktif(&ctx);
    let list = CardList::new(
        "🎨 Pilih Variant Menu",
        vec![CardSection::new("Daftar Variant Menu").rows(baris(&ctx.prefix, current))],
    )
    .icon("DEFAULT");

    let mut reply = Reply::interactive(body_teks(current))
        .with_button_card(InteractiveButton::select(&list));
    if let Some(aset) = ctx.config().assets.get("foto")
        && ctx.berkas().ada(aset)
    {
        reply = reply.with_header(CardHeader::new().image(MediaSource::path(aset)));
    }
    reply
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::Mutation;

    fn run(args: &[&str], settings: serde_json::Value) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "setmenu".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            settings,
            ..Default::default()
        };
        setmenu(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    /// Baris `single_select` dari kartu — JSON yang jadi `buttonParamsJson`.
    fn baris_kartu(r: &Reply) -> Vec<serde_json::Value> {
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("harus Interactive");
        };
        assert_eq!(kartu.buttons.len(), 1);
        assert_eq!(kartu.buttons[0].name, "single_select");
        kartu.buttons[0]
            .params
            .nilai()["sections"][0]["rows"]
            .as_array()
            .cloned()
            .unwrap_or_default()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("setmenu").unwrap();
        assert_eq!(p.meta.name, "setmenu");
        assert_eq!(p.meta.alias, ["menuvariant", "menustyle"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Mengatur variant tampilan menu");
        assert_eq!(p.meta.usage, ".setmenu <v1-v16>");
        assert_eq!(p.meta.example, ".setmenu v8");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn variant_v8_ada_dalam_konstanta() {
        assert!(VARIANTS.iter().any(|(k, _, _, _, _)| *k == "v8"));
    }

    #[test]
    fn variant_rubby_v16_terdaftar() {
        let r = run(&[], serde_json::json!({}));
        let baris = baris_kartu(&r);
        assert_eq!(baris.len(), 9, "v1..v8 + v16");
        assert_eq!(baris[8]["id"], ".setmenu v16");
        assert!(baris[8]["title"].as_str().unwrap().contains("V16"));
    }

    #[test]
    fn setmenu_v16_menyimpan_id_16() {
        let r = run(&["v16"], serde_json::json!({}));
        match &r.mutations()[0] {
            Mutation::SetSetting { value, .. } => assert_eq!(value, &serde_json::json!(16)),
            other => panic!("harus SetSetting, dapat {other:?}"),
        }
    }

    #[test]
    fn variant_tidak_valid_ditolak_dengan_teks_js() {
        let r = run(&["v9"], serde_json::json!({}));
        assert_eq!(
            teks(&r),
            "❌ *VARIANT TIDAK VALID*\n\nGunakan: *v1* s/d *v7*"
        );
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn variant_valid_disimpan_ke_setting_menu_variant() {
        let r = run(&["V5"], serde_json::json!({}));
        assert_eq!(
            teks(&r),
            "✅ *MENU VARIANT DIUBAH*\n\n✅ *V5 — LV 2*\n__"
        );
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetSetting { key, value } => {
                assert_eq!(key, "menuVariant");
                assert_eq!(value, &serde_json::json!(5));
            }
            other => panic!("harus SetSetting, dapat {other:?}"),
        }
    }

    #[test]
    fn tanpa_argumen_menampilkan_kartu_dengan_tombol() {
        let r = run(&[], serde_json::json!({}));
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("harus Interactive");
        };
        assert!(kartu.body.starts_with("🎨🖼️ *MENU VARIANT*"));
        assert!(kartu.body.contains("Variant aktif saat ini: *V3 — PREMIUM* 🎯"));
        assert!(kartu.body.ends_with("> Pilih variant menu dari tombol di bawah 👇"));
        assert_eq!(kartu.buttons.len(), 1);
        assert_eq!(kartu.buttons[0].name, "single_select");
    }

    #[test]
    fn baris_kartu_menandai_variant_aktif_dengan_centang() {
        let r = run(&[], serde_json::json!({ "menuVariant": 5 }));
        let baris = baris_kartu(&r);
        assert_eq!(baris.len(), 8);
        assert_eq!(baris[0]["title"], "🖼️ V1 — BASIC");
        assert_eq!(baris[4]["title"], "✅ V5 ✓ — LV 2");
        assert_eq!(baris[4]["id"], ".setmenu v5");
        assert_eq!(baris[5]["description"], "Location Message dengan Buttons");
    }

    #[test]
    fn kartu_menyebut_variant_aktif_dari_setting() {
        let r = run(&[], serde_json::json!({ "menuVariant": 8 }));
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("harus Interactive");
        };
        assert!(kartu
            .body
            .contains("Variant aktif saat ini: *V8 — THUMBNAIL* 🎯"));
    }
}
