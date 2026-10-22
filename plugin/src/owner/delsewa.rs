//! `plugins/owner/delsewa.js` — hapus grup dari whitelist sewa.
//!
//! `db.db.data.sewa` dipetakan ke key global `sewa` di `settings`, sama
//! seperti `listsewa.rs` (lihat catatan di modul itu).
//!
//! ## Penyimpangan
//!
//! - `sock.groupGetInviteInfo(code)` untuk link `chat.whatsapp.com/...`
//!   tersedia lewat `Sock::info_undangan`, jadi jalur ini ikut dip-port.
//! - `sock.sendText(groupId, ...)` + `sock.groupLeave(groupId)` setelah
//!   penghapusan tidak bisa dilakukan: `Reply` hanya kirim ke chat pemicu.
//!   Karena itu bagian "bot keluar dari grup" tidak ada di port ini.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::Value;

/// Key global tempat `db.db.data.sewa` dipetakan.
const SEWA_KEY: &str = "sewa";

/// Teks bantuan `delsewa.js:46-53`.
pub(crate) fn bantuan(prefix: &str) -> String {
    format!(
        "📝 *HAPUS SEWA*\n\n\
         Dari private: *{prefix}delsewa <link/id>*\n\
         Dari grup: ketik *{prefix}delsewa* langsung di grup\n\n\
         Contoh:\n\
         • {prefix}delsewa https://chat.whatsapp.com/xxx\n\
         • {prefix}delsewa 120363xxx\n\n\
         ⚠️ Jika sewabot aktif, bot akan otomatis keluar dari grup yang dihapus"
    )
}

/// `delsewa.js:59`.
pub(crate) const LINK_INVALID: &str = "❌ Link tidak valid atau grup tidak ditemukan";

/// `delsewa.js:64`.
pub(crate) const TIDAK_BISA_TENTUKAN: &str = "❌ Tidak dapat menentukan grup";

/// `delsewa.js:68-70`.
pub(crate) fn tidak_terdaftar(prefix: &str) -> String {
    format!(
        "❌ Grup tidak terdaftar dalam sistem sewa\n\nLihat daftar: *{prefix}listsewa*"
    )
}

/// `delsewa.js:79`.
pub(crate) fn dihapus(nama_grup: &str, id_grup: &str) -> String {
    format!("✅ *SEWA DIHAPUS*\n\nGrup: *{nama_grup}*\nID: {id_grup}")
}

/// `resolveGroupId` (`delsewa.js:19-29`) — link invite → id grup, kalau tidak
/// ada maka id apa adanya.
fn id_dari_masukan(ctx: &Ctx, input: &str) -> Option<(String, Option<String>)> {
    if input.contains("chat.whatsapp.com/") {
        let kode = input
            .split("chat.whatsapp.com/")
            .nth(1)?
            .split(|c: char| c.is_whitespace() || c == '?')
            .next()?
            .to_string();
        let Ok(meta) = ctx.sock().info_undangan(&kode) else {
            return None;
        };
        let id = meta.get("id").and_then(Value::as_str)?.to_string();
        let nama = meta
            .get("subject")
            .and_then(Value::as_str)
            .map(str::to_string);
        return Some((id, nama));
    }
    let id = if input.contains("@g.us") {
        input.to_string()
    } else {
        format!("{input}@g.us")
    };
    Some((id, None))
}

#[plugin(
    name = "delsewa",
    alias = ["sewadel", "hapussewa", "removesewa"],
    category = "owner",
    description = "Hapus grup dari whitelist sewa",
    usage = ".delsewa <link/id grup>",
    example = ".delsewa https://chat.whatsapp.com/xxx",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn delsewa(ctx: Ctx) -> Reply {
    // `if (!db.db.data.sewa) { db.db.data.sewa = {...}; db.db.write() }`
    // (`delsewa.js:35-38`).
    let init = crate::common::js_falsy(ctx.setting(SEWA_KEY)).then(|| Mutation::SetSetting {
        key: SEWA_KEY.to_string(),
        value: serde_json::json!({ "enabled": false, "groups": {} }),
    });

    let input = ctx.text.trim().to_string();

    let (group_id, group_name) = if input.is_empty() {
        if !ctx.is_group {
            return dengan_init(Reply::text(bantuan(&ctx.prefix)), init);
        }
        (ctx.chat.clone(), None)
    } else {
        match id_dari_masukan(&ctx, &input) {
            Some((id, nama)) => (id, nama),
            None => return dengan_init(Reply::text(LINK_INVALID), init),
        }
    };

    if group_id.is_empty() {
        return dengan_init(Reply::text(TIDAK_BISA_TENTUKAN), init);
    }

    let tabel = ctx
        .setting(SEWA_KEY)
        .cloned()
        .unwrap_or(Value::Null);
    let groups = tabel
        .get("groups")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let Some(data) = groups.get(&group_id) else {
        return dengan_init(Reply::text(tidak_terdaftar(&ctx.prefix)), init);
    };

    let nama_grup = group_name
        .or_else(|| {
            data.get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| {
            group_id
                .split('@')
                .next()
                .unwrap_or(&group_id)
                .to_string()
        });

    let mut groups_baru = groups.clone();
    groups_baru.remove(&group_id);

    let mut tabel_baru = tabel.clone();
    if let Some(obj) = tabel_baru.as_object_mut() {
        obj.insert("groups".to_string(), Value::Object(groups_baru));
    } else {
        tabel_baru = serde_json::json!({ "enabled": false, "groups": groups_baru });
    }

    Reply::text(dihapus(&nama_grup, group_id.split('@').next().unwrap_or(&group_id)))
        .with_reaction("✅")
        .with_mutation(Mutation::SetSetting {
            key: SEWA_KEY.to_string(),
            value: tabel_baru,
        })
}

/// Gabungkan balasan dengan mutasi inisialisasi tabel sewa.
fn dengan_init(reply: Reply, init: Option<Mutation>) -> Reply {
    match init {
        Some(m) => reply.with_mutation(m),
        None => reply,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, is_group: bool, settings: Value) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "delsewa".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: if is_group {
                "120363@g.us".into()
            } else {
                "628111@s.whatsapp.net".into()
            },
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            is_group,
            settings,
            ..Default::default()
        };
        delsewa(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("delsewa").unwrap();
        assert_eq!(p.meta.name, "delsewa");
        assert_eq!(p.meta.alias, ["sewadel", "hapussewa", "removesewa"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Hapus grup dari whitelist sewa");
        assert_eq!(p.meta.usage, ".delsewa <link/id grup>");
        assert_eq!(p.meta.example, ".delsewa https://chat.whatsapp.com/xxx");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn private_tanpa_argumen_menampilkan_bantuan() {
        let r = run("", false, Value::Null);
        assert_eq!(
            teks(&r),
            "📝 *HAPUS SEWA*\n\n\
             Dari private: *.delsewa <link/id>*\n\
             Dari grup: ketik *.delsewa* langsung di grup\n\n\
             Contoh:\n\
             • .delsewa https://chat.whatsapp.com/xxx\n\
             • .delsewa 120363xxx\n\n\
             ⚠️ Jika sewabot aktif, bot akan otomatis keluar dari grup yang dihapus"
        );
    }

    #[test]
    fn grup_yang_tidak_terdaftar_ditolak() {
        let settings = serde_json::json!({ "sewa": { "enabled": true, "groups": {} } });
        let r = run("120363", false, settings);
        assert_eq!(
            teks(&r),
            "❌ Grup tidak terdaftar dalam sistem sewa\n\nLihat daftar: *.listsewa*"
        );
    }

    #[test]
    fn grup_terdaftar_dihapus_dari_tabel() {
        let settings = serde_json::json!({
            "sewa": { "enabled": true, "groups": { "120363@g.us": { "name": "Grupku" } } }
        });
        let r = run("120363", false, settings);
        assert_eq!(teks(&r), "✅ *SEWA DIHAPUS*\n\nGrup: *Grupku*\nID: 120363");
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetSetting { key, value } => {
                assert_eq!(key, SEWA_KEY);
                assert!(value.get("groups").unwrap().get("120363@g.us").is_none());
            }
            other => panic!("harus SetSetting, dapat {other:?}"),
        }
    }

    #[test]
    fn nama_grup_jatuh_ke_id_bila_tidak_ada_nama() {
        let settings = serde_json::json!({
            "sewa": { "enabled": true, "groups": { "120363@g.us": {} } }
        });
        let r = run("120363", false, settings);
        assert!(teks(&r).contains("Grup: *120363*"), "{}", teks(&r));
    }

    #[test]
    fn tabel_hilang_dibuat_saat_perintah_dijalankan() {
        let r = run("", false, Value::Null);
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetSetting { key, value } => {
                assert_eq!(key, SEWA_KEY);
                assert_eq!(value, &serde_json::json!({ "enabled": false, "groups": {} }));
            }
            other => panic!("harus SetSetting, dapat {other:?}"),
        }
    }
}
