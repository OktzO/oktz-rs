//! `plugins/owner/setallmenu.js` — atur variant tampilan allmenu.
//!
//! ## Penyimpangan
//!
//! - `sock.sendButton(chat, bufferFoto, body, m, { buttons })` dipetakan ke
//!   `Reply::interactive(..)` dengan header gambar dari `config.assets.foto`
//!   dan satu tombol `single_select`.
//! - `db.setting("allmenuVariant", id)` + `db.save()` dipetakan ke
//!   `Mutation::SetSetting`.

use oktz_plugin_api::registry::{CardHeader, CardList, CardRow, CardSection, InteractiveButton};
use oktz_plugin_api::{Ctx, MediaSource, Mutation, Reply, plugin};

/// Satu variant allmenu (`setallmenu.js:19-44`).
struct Variant {
    id: i64,
    key: &'static str,
    name: &'static str,
    desc: &'static str,
    emoji: &'static str,
}

const VARIANTS: &[Variant] = &[
    Variant {
        id: 1,
        key: "v1",
        name: "ALLMENU BASIC",
        desc: "ini mengikuti dari setreply",
        emoji: "📝",
    },
    Variant {
        id: 2,
        key: "v2",
        name: "ALLMENU PREMIUM",
        desc: "",
        emoji: "🖼️",
    },
    Variant {
        id: 5,
        key: "v5",
        name: "ALLMENU NATIVEFLOW",
        desc: "Tampilan native flow premium dengan video & cuaca",
        emoji: "✨",
    },
    Variant {
        id: 6,
        key: "v6",
        name: "ALLMENU LOCATION",
        desc: "Tampilan location message tanpa tombol interaktif",
        emoji: "📍",
    },
];

/// Kunci setting untuk variant allmenu.
const SETTING_KEY: &str = "allmenuVariant";

/// `config.ui?.allmenuVariant || 2` — `config.ui` tidak ada di `Runtime`.
const DEFAULT_VARIANT: i64 = 2;

/// `❌ *VARIANT TIDAK VALID*` (`setallmenu.js:53`).
pub(crate) const TIDAK_VALID: &str =
    "❌ *VARIANT TIDAK VALID*\n\nGunakan: *v1*, *v2*, *v5*, atau *v6*";

/// Pesan sukses `setallmenu.js:61-64`.
pub(crate) fn pesan_diubah(v: &Variant) -> String {
    format!(
        "✅ *ALLMENU VARIANT DIUBAH*\n\n\
         {} *V{} — {}*\n\
         _{}_",
        v.emoji, v.id, v.name, v.desc
    )
}

/// Nama variant aktif — `VARIANTS[`v${current}`]?.name || "Unknown"`.
pub(crate) fn nama_variant(id: i64) -> &'static str {
    VARIANTS
        .iter()
        .find(|v| v.id == id)
        .map(|v| v.name)
        .unwrap_or("Unknown")
}

/// Teks body kartu `setallmenu.js:90-100`.
pub(crate) fn body_teks(current: i64) -> String {
    format!(
        "📋📑 *ALLMENU VARIANT*\n\n\
         Atur tampilan allmenu yang menampilkan seluruh daftar perintah bot dalam satu halaman 📖✨\n\
         Variant aktif saat ini: *V{current} — {nama}* 🎯\n\n\
         *PENJELASAN VARIANT:*\n\n\
         - *V1 Simple Text* 📝 — Daftar perintah ditampilkan sebagai text biasa tanpa gambar atau contextInfo, paling ringan dan cepat dimuat\n\n\
         - *V2 Image + Context* 🖼️ — Gambar header allmenu + full contextInfo dengan label forwarded newsletter, tampilan standar yang informatif\n\n\
         - *V3 Document* 📄 — Allmenu dikirim sebagai file document dengan thumbnail kecil dan verified quoted reply, terlihat seperti file resmi\n\n\
         - *V4 Interactive Button* 🔘 — Pesan interaktif dengan tombol single_select untuk memilih kategori dan quick_reply untuk navigasi, tampilan modern\n\n\
         - *V5 NativeFlow* ✨ — NativeFlow message dengan limited_time_offer badge dan interactive buttons, tampilan paling premium dan eye-catching\n\n\
         > Pilih variant allmenu dari tombol di bawah 👇",
        nama = nama_variant(current)
    )
}

/// Baris tombol `single_select` (`setallmenu.js:72-79`).
pub(crate) fn baris(prefix: &str, current: i64) -> Vec<CardRow> {
    VARIANTS
        .iter()
        .map(|v| {
            let mark = if v.id == current { " ✓" } else { "" };
            CardRow::new(
                format!("{} {}{} — {}", v.emoji, v.key.to_uppercase(), mark, v.name),
                format!("{prefix}setallmenu {}", v.key),
            )
            .description(v.desc)
        })
        .collect()
}

/// Variant aktif: `db.setting("allmenuVariant") || config.ui?.allmenuVariant || 2`.
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
    name = "setallmenu",
    alias = ["allmenuvariant", "allmenustyle"],
    category = "owner",
    description = "Mengatur variant tampilan allmenu",
    usage = ".setallmenu <v1-v5>",
    example = ".setallmenu v2",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn setallmenu(ctx: Ctx) -> Reply {
    let variant = ctx.args.first().map(|s| s.to_lowercase());

    if let Some(key) = variant.as_deref() {
        let Some(selected) = VARIANTS.iter().find(|v| v.key == key) else {
            return Reply::text(TIDAK_VALID);
        };
        return Reply::text(pesan_diubah(selected)).with_mutation(Mutation::SetSetting {
            key: SETTING_KEY.to_string(),
            value: serde_json::json!(selected.id),
        });
    }

    let current = variant_aktif(&ctx);
    let list = CardList::new(
        "📋 Pilih Variant Allmenu",
        vec![CardSection::new("Daftar Variant Allmenu").rows(baris(
            &ctx.prefix, current,
        ))],
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

    fn run(args: &[&str], settings: serde_json::Value) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "setallmenu".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            settings,
            ..Default::default()
        };
        setallmenu(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("setallmenu").unwrap();
        assert_eq!(p.meta.name, "setallmenu");
        assert_eq!(p.meta.alias, ["allmenuvariant", "allmenustyle"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Mengatur variant tampilan allmenu");
        assert_eq!(p.meta.usage, ".setallmenu <v1-v5>");
        assert_eq!(p.meta.example, ".setallmenu v2");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn variant_tidak_valid_ditolak() {
        let r = run(&["v9"], serde_json::json!({}));
        assert_eq!(teks(&r), TIDAK_VALID);
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn variant_valid_disimpan_dan_dilaporkan() {
        let r = run(&["V5"], serde_json::json!({}));
        assert_eq!(
            teks(&r),
            "✅ *ALLMENU VARIANT DIUBAH*\n\n\
             ✨ *V5 — ALLMENU NATIVEFLOW*\n\
             _Tampilan native flow premium dengan video & cuaca_"
        );
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetSetting { key, value } => {
                assert_eq!(key, SETTING_KEY);
                assert_eq!(value, &serde_json::json!(5));
            }
            other => panic!("harus SetSetting, dapat {other:?}"),
        }
    }

    #[test]
    fn tanpa_argumen_menampilkan_kartu_dengan_tombol() {
        let r = run(&[], serde_json::json!({ "allmenuVariant": 2 }));
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("harus Interactive");
        };
        let body = &kartu.body;
        let buttons = &kartu.buttons;
        assert!(body.contains("📋📑 *ALLMENU VARIANT*"));
        assert!(body.contains("Variant aktif saat ini: *V2 — ALLMENU PREMIUM* 🎯"));
        assert!(body.ends_with("> Pilih variant allmenu dari tombol di bawah 👇"));
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].name, "single_select");
    }

    #[test]
    fn baris_menandai_variant_aktif_dengan_centang() {
        let baris = baris(".", 2);
        assert_eq!(baris.len(), 4);
        assert!(baris[0].title.contains("📝 V1 — ALLMENU BASIC"));
        assert!(baris[1].title.contains("🖼️ V2 ✓ — ALLMENU PREMIUM"));
        assert_eq!(baris[1].id, ".setallmenu v2");
    }

    #[test]
    fn nama_variant_tidak_dikenal_menjadi_unknown() {
        assert_eq!(nama_variant(2), "ALLMENU PREMIUM");
        assert_eq!(nama_variant(99), "Unknown");
    }
}
