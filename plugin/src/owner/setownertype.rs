//! Atur variant tampilan owner message.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::json;

/// `VARIANTS`.
pub(crate) const VARIANTS: [(&str, &str, &str); 2] = [
    ("1", "Current Design", "Tampilan default saat ini"),
    ("2", "Multiple Contact", "Kirim contact card semua owner"),
];

/// `current = db.setting('ownerType') || 1` — baca dari settings.
pub(crate) fn owner_type_saat_ini(ctx: &Ctx) -> i64 {
    ctx.setting("ownerType")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(1)
}

/// `/^v?[1-3]$/` → id.
pub(crate) fn parse_varian(variant: &str) -> Option<i64> {
    let bersih = variant.trim_start_matches('v');
    if bersih.len() == 1 && bersih.chars().next().is_some_and(|c| matches!(c, '1' | '2' | '3')) {
        bersih.parse::<i64>().ok()
    } else {
        None
    }
}

/// Teks saat pilihan variant.
pub(crate) fn teks_pilihan(current: i64, _nama_bot: &str) -> String {
    let varian = VARIANTS.iter().find(|(id, _, _)| id == &current.to_string());
    let (nama, _) = varian.map(|(_, n, d)| (*n, *d)).unwrap_or(("Current Design", "Tampilan default saat ini"));
    format!(
        "🎨 *sᴇᴛ ᴏᴡɴᴇʀ ᴛʏᴘᴇ*\n\n> Type saat ini: *V{current}*\n> _{}_\n\n> Pilih variant owner:",
        nama
    )
}

/// Daftar tombol variant, tanpa markup tombol WA.
pub(crate) fn daftar_varian(current: i64) -> Vec<String> {
    let mut out = Vec::new();
    for (id, nama, _) in VARIANTS {
        let tandai = if id.parse::<i64>().unwrap_or(0) == current { " ✓" } else { "" };
        out.push(format!("V{id}{tandai} - {nama}"));
    }
    out
}

/// Sukses ganti variant.
pub(crate) fn berhasil_ganti(id: i64) -> String {
    let varian = VARIANTS.iter().find(|(i, _, _)| *i == id.to_string());
    let (nama, desc) = varian.map(|(_, n, d)| (*n, *d)).unwrap_or(("", ""));
    format!(
        "✅ Owner type diubah ke *V{id}*\n\n> *{nama}*\n> _{desc}_"
    )
}

#[plugin(
    name = "setownertype",
    alias = ["ownertype", "ownervariant", "ownerstyle"],
    category = "owner",
    description = "Mengatur variant tampilan owner message",
    usage = ".setownertype",
    example = ".setownertype",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn setownertype(ctx: Ctx) -> Reply {
    let current = owner_type_saat_ini(&ctx);

    // `if (variant && /^v?[1-3]$/.test(variant))`.
    if let Some(id) = ctx.args.first().and_then(|a| parse_varian(a)) {
        return Reply::text(berhasil_ganti(id)).with_mutation(Mutation::SetSetting {
            key: "ownerType".into(),
            value: json!(id),
        });
    }

    // Varian tanpa tombol: daftar + perintah balik.
    let mut text = teks_pilihan(current, &ctx.config().bot.name);
    for baris in daftar_varian(current) {
        text.push_str(&format!("\n> {baris}"));
    }
    text.push_str(&format!(
        "\n\n> Balas `{}setownertype v1` atau `{}setownertype v2`",
        ctx.prefix, ctx.prefix
    ));
    Reply::text(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_dan_alias() {
        let p = oktz_plugin_api::find("setownertype").unwrap();
        assert_eq!(p.meta.name, "setownertype");
        assert_eq!(p.meta.alias, ["ownertype", "ownervariant", "ownerstyle"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tabel_varian_sama_dengan_js() {
        assert_eq!(VARIANTS.len(), 2);
        assert_eq!(VARIANTS[0], ("1", "Current Design", "Tampilan default saat ini"));
        assert_eq!(VARIANTS[1], ("2", "Multiple Contact", "Kirim contact card semua owner"));
    }

    #[test]
    fn owner_type_saat_ini_default_1() {
        let ctx = Ctx {
            settings: serde_json::json!({}),
            ..Ctx::default()
        };
        assert_eq!(owner_type_saat_ini(&ctx), 1);
    }

    #[test]
    fn owner_type_saat_ini_baca_dari_settings() {
        let ctx = Ctx {
            settings: serde_json::json!({ "ownerType": 2 }),
            ..Ctx::default()
        };
        assert_eq!(owner_type_saat_ini(&ctx), 2);
    }

    #[test]
    fn parse_varian_mirip_regex_js() {
        assert_eq!(parse_varian("1"), Some(1));
        assert_eq!(parse_varian("v1"), Some(1));
        assert_eq!(parse_varian("2"), Some(2));
        assert_eq!(parse_varian("v3"), Some(3));
        assert_eq!(parse_varian("4"), None);
        assert_eq!(parse_varian("v4"), None);
        assert_eq!(parse_varian("abc"), None);
        assert_eq!(parse_varian(""), None);
    }

    #[test]
    fn daftar_varian_tandai_terpilih() {
        let out = daftar_varian(1);
        assert_eq!(out, vec!["V1 ✓ - Current Design", "V2 - Multiple Contact"]);
        let out = daftar_varian(2);
        assert_eq!(out, vec!["V1 - Current Design", "V2 ✓ - Multiple Contact"]);
    }

    #[test]
    fn berhasil_ganti_bentuk_js() {
        assert_eq!(
            berhasil_ganti(2),
            "✅ Owner type diubah ke *V2*\n\n> *Multiple Contact*\n> _Kirim contact card semua owner_"
        );
    }

    #[test]
    fn teks_pilihan_bawah_dari_varian() {
        assert!(
            teks_pilihan(1, "Oktz").contains("Type saat ini: *V1*")
        );
        assert!(teks_pilihan(1, "Oktz").contains("Current Design"));
    }

    fn ctx(args: &[&str]) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "setownertype".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            settings: serde_json::json!({}),
            ..Ctx::default()
        }
    }

    #[test]
    fn tanpa_arg_menampilkan_daftar_varian_dan_tanpa_mutasi() {
        let r = setownertype(ctx(&[]));
        let t = r.without_mutations().text_body().unwrap();
        assert!(t.contains("Type saat ini: *V1*"), "{t}");
        assert!(t.contains("V1 ✓ - Current Design"), "{t}");
        assert!(t.contains("V2 - Multiple Contact"), "{t}");
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn v2_menulis_setting_owner_type_dan_berpesan_sukses() {
        let r = setownertype(ctx(&["v2"]));
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            "✅ Owner type diubah ke *V2*\n\n> *Multiple Contact*\n> _Kirim contact card semua owner_"
        );
        assert_eq!(
            r.mutations(),
            [Mutation::SetSetting {
                key: "ownerType".into(),
                value: serde_json::json!(2),
            }]
        );
    }

    #[test]
    fn arg_non_varian_jatuh_ke_daftar() {
        let r = setownertype(ctx(&["abc"]));
        assert!(r.without_mutations().text_body().unwrap().contains("Type saat ini"));
        assert!(r.mutations().is_empty());
    }
}
