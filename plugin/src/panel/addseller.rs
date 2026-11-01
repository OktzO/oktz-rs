//! `plugins/panel/addseller.js` — kelola seller/reseller panel.
//!
//! Satu plugin JS dengan enam pintu masuk: `name: 'addseller'` dan
//! `alias: ['addreseller','delseller','delreseller','listseller','listreseller']`.
//! Semua kunci masuk `alias` di Rust — `addseller` sebagai `name`, sisanya
//! persis daftar `alias` JS.
//!
//! State disimpan di `database/cpanel/legacy_sellers.json`
//! (`getLegacyData`/`saveLegacyData`, `addseller.js:32-51`) — dibaca/tulis
//! lewat `Ctx::berkas()`, jadi plugin ini full-port, bukan stub.
//!
//! `ponytail:` `isLid`/`lidToJid` (`addseller.js:4`) tidak ada di `Ctx`; JID
//! dipakai apa adanya seperti `cleanJid` tanpa cabang LID. Sinkronisasi
//! `config.pterodactyl` dari legacy data (`addseller.js:65-68`) tidak ada
//! padanannya karena `Ctx::config()` read-only dan tanpa field pterodactyl —
//! tidak berpengaruh pada balasan.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::{Map, Value};

/// `getLegacyData()` — jalur berkas + bentuk default.
const LEGACY_PATH: &str = "database/cpanel/legacy_sellers.json";

fn legacy_default() -> Value {
    let mut m = Map::new();
    m.insert("sellers".to_string(), Value::Array(Vec::new()));
    m.insert("ownerPanels".to_string(), Value::Array(Vec::new()));
    Value::Object(m)
}

/// `getLegacyData()` (`addseller.js:32-38`) — gagal parse = default, sama
/// seperti `try { JSON.parse } catch { default }`.
fn muat_legacy(ctx: &Ctx) -> Value {
    match ctx.berkas().baca_teks(LEGACY_PATH) {
        Ok(teks) => serde_json::from_str(&teks).unwrap_or_else(|_| legacy_default()),
        Err(_) => legacy_default(),
    }
}

/// `saveLegacyData()` (`addseller.js:40-51`) — buat folder induk dulu.
fn simpan_legacy(ctx: &Ctx, data: &Value) -> bool {
    let _ = ctx.berkas().buat_folder("database/cpanel");
    let Ok(json) = serde_json::to_string_pretty(data) else {
        return false;
    };
    ctx.berkas().tulis_teks(LEGACY_PATH, &json).is_ok()
}

/// `getNumber(jid)` (`addseller.js:27-30`) — `cleanJid(jid).split('@')[0]`;
/// `cleanJid` tanpa LID: JID tanpa `@` diberi `@s.whatsapp.net`.
fn nomor_dari_jid(jid: &str) -> Option<String> {
    let clean = if jid.contains('@') {
        jid.to_string()
    } else if jid.is_empty() {
        return None;
    } else {
        format!("{jid}@s.whatsapp.net")
    };
    let nomor = clean.split('@').next().unwrap_or("").to_string();
    if nomor.is_empty() { None } else { Some(nomor) }
}

/// `hasAccess(senderJid, isOwner, legacyData)` (`addseller.js:53-59`) —
/// owner, atau nomor sender ada di `ownerPanels`.
fn punya_akses(ctx: &Ctx, data: &Value) -> bool {
    if ctx.is_owner {
        return true;
    }
    let Some(clean) = nomor_dari_jid(&ctx.sender) else {
        return false;
    };
    data.get("ownerPanels")
        .and_then(Value::as_array)
        .map(|a| a.iter().any(|v| v.as_str() == Some(clean.as_str())))
        .unwrap_or(false)
}

/// Daftar seller sebagai `Vec<String>` — `legacyData.sellers || []`.
fn daftar_seller(data: &Value) -> Vec<String> {
    data.get("sellers")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn sudah_seller(data: &Value, nomor: &str) -> bool {
    daftar_seller(data).iter().any(|s| s == nomor)
}

/// `m.react('✅')` sebelum `m.reply(...)` (`addseller.js:131-140`,
/// `:153-162`).
fn balas_dengan_reaksi(text: String) -> Reply {
    Reply::text(text).with_reaction("✅")
}

#[plugin(
    name = "addseller",
    alias = [
        "addreseller", "delseller", "delreseller", "listseller", "listreseller",
    ],
    category = "panel",
    description = "Kelola seller/reseller panel",
    usage = ".addseller @user atau .delseller @user",
    example = ".addseller @user",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn addseller(ctx: Ctx) -> Reply {
    let data = muat_legacy(&ctx);
    let cmd = ctx.command.as_str();

    // Guard akses sebelum cabang mana pun (`addseller.js:70-72`).
    if !punya_akses(&ctx, &data) {
        return Reply::text("❌ *ᴀᴋsᴇs ᴅɪᴛᴏʟᴀᴋ*\n\n> Fitur ini hanya untuk Owner atau Owner Panel.");
    }

    let is_add = cmd == "addseller" || cmd == "addreseller";
    let is_del = cmd == "delseller" || cmd == "delreseller";
    let is_list = cmd == "listseller" || cmd == "listreseller";

    if is_list {
        return cmd_list(&data);
    }

    // Target: quoted → mention → teks → sender (`addseller.js:96-105`).
    let target_user = if let Some(q) = ctx.quoted() {
        nomor_dari_jid(&q.sender)
    } else if let Some(jid) = ctx.mentioned.first() {
        nomor_dari_jid(jid)
    } else if !ctx.text.trim().is_empty() {
        let digits: String = ctx.text.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() { None } else { Some(digits) }
    } else {
        nomor_dari_jid(&ctx.sender)
    };

    let Some(target) = target_user else {
        let cmd = ctx.command.as_str();
        return Reply::text(format!(
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `.{cmd} @user`\n> `.{cmd} 628xxx`\n> Reply pesan user"
        ));
    };

    if is_add {
        return cmd_add(&ctx, &data, &target);
    }
    if is_del {
        return cmd_del(&ctx, &data, &target);
    }
    // Tidak terjangkau lewat registry (enam kunci tercakup semua); di JS
    // handler hanya berhenti tanpa balasan.
    Reply::None
}

/// `isList` (`addseller.js:82-94`).
fn cmd_list(data: &Value) -> Reply {
    let sellers = daftar_seller(data);
    if sellers.is_empty() {
        return Reply::text("📋 *ᴅᴀꜰᴛᴀʀ sᴇʟʟᴇʀ/ʀᴇsᴇʟʟᴇʀ*\n\n> Belum ada seller terdaftar.");
    }
    let mut txt = format!(
        "📋 *ᴅᴀꜰᴛᴀʀ sᴇʟʟᴇʀ/ʀᴇsᴇʟʟᴇʀ*\n\n> Total: *{}* seller\n\n",
        sellers.len()
    );
    for (i, s) in sellers.iter().enumerate() {
        txt.push_str(&format!("{}. `{}`\n", i + 1, s));
    }
    txt.push_str("\n> _Seller bisa create server (1gb-10gb v1/v2/v3)_");
    Reply::text(txt)
}

/// `isAdd` (`addseller.js:116-144`).
fn cmd_add(ctx: &Ctx, data: &Value, target: &str) -> Reply {
    if sudah_seller(data, target) {
        return Reply::text(format!("❌ `{target}` sudah menjadi seller."));
    }

    let mut data = data.clone();
    let mut role_changed = String::new();
    let owner_panels = data
        .get("ownerPanels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(pos) = owner_panels.iter().position(|v| v.as_str() == Some(target)) {
        let mut baru = owner_panels;
        baru.remove(pos);
        if let Some(obj) = data.as_object_mut() {
            obj.insert("ownerPanels".to_string(), Value::Array(baru));
        }
        role_changed = "\n> ⚡ Auto-downgrade dari Owner Panel ke Seller".to_string();
    }

    let mut sellers = daftar_seller(&data);
    sellers.push(target.to_string());
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "sellers".to_string(),
            Value::Array(sellers.iter().map(|s| Value::String(s.clone())).collect()),
        );
    }

    if !simpan_legacy(ctx, &data) {
        return Reply::text("❌ Gagal menyimpan ke database");
    }

    balas_dengan_reaksi(format!(
        "✅ *sᴇʟʟᴇʀ ᴅɪᴛᴀᴍʙᴀʜᴋᴀɴ*\n\n\
         ╭┈┈⬡「 📋 *ᴅᴇᴛᴀɪʟ* 」\n\
         ┃ 📱 ɴᴏᴍᴏʀ: `{target}`\n\
         ┃ 🏷️ sᴛᴀᴛᴜs: `Seller/Reseller`\n\
         ┃ 🔓 ᴀᴋsᴇs: `Create Server (1gb-10gb v1-v3)`\n\
         ┃ 📊 ᴛᴏᴛᴀʟ: `{}` seller\n\
         ╰┈┈⬡{role_changed}",
        sellers.len()
    ))
}

/// `isDel` (`addseller.js:146-163`).
fn cmd_del(ctx: &Ctx, data: &Value, target: &str) -> Reply {
    if !sudah_seller(data, target) {
        return Reply::text(format!("❌ `{target}` bukan seller."));
    }

    let mut data = data.clone();
    let sisa: Vec<String> = daftar_seller(data.get("sellers").unwrap_or(&Value::Null))
        .into_iter()
        .filter(|s| s != target)
        .collect();
    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "sellers".to_string(),
            Value::Array(sisa.iter().map(|s| Value::String(s.clone())).collect()),
        );
    }

    if !simpan_legacy(ctx, &data) {
        return Reply::text("❌ Gagal menyimpan ke database");
    }

    balas_dengan_reaksi(format!(
        "✅ *sᴇʟʟᴇʀ ᴅɪʜᴀᴘᴜs*\n\n> Nomor: `{target}`\n> Total: *{}* seller",
        sisa.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use oktz_plugin_api::find;
    use std::sync::Arc;

    fn ctx(command: &str, sender: &str, is_owner: bool) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: command.into(),
            sender: sender.into(),
            chat: sender.into(),
            is_owner,
            berkas: Arc::new(BerkasPalsu::baru()),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    const OWNER: &str = "628111@s.whatsapp.net";
    const PANEL: &str = "628222@s.whatsapp.net";

    #[test]
    fn semua_enam_kunci_js_resolve() {
        for kunci in [
            "addseller", "addreseller", "delseller", "delreseller", "listseller", "listreseller",
        ] {
            assert_eq!(
                find(kunci).unwrap_or_else(|| panic!("{kunci} tidak terdaftar")).meta.name,
                "addseller",
                "kunci {kunci}"
            );
        }
        let m = &find("addseller").unwrap().meta;
        assert_eq!(m.alias, ["addreseller", "delseller", "delreseller", "listseller", "listreseller"]);
        assert_eq!(m.category, "panel");
        assert_eq!(m.description, "Kelola seller/reseller panel");
        assert_eq!(m.cooldown, 5);
        assert!(!m.is_owner, "isOwner: false di JS");
        assert!(!m.is_group && !m.is_private);
    }

    #[test]
    fn akses_ditolak_untuk_bukan_owner_dan_bukan_owner_panel() {
        let c = ctx("addseller", "628999@s.whatsapp.net", false);
        let r = addseller(c);
        assert_eq!(
            teks(&r),
            "❌ *ᴀᴋsᴇs ᴅɪᴛᴏʟᴀᴋ*\n\n> Fitur ini hanya untuk Owner atau Owner Panel."
        );
    }

    #[test]
    fn owner_panel_dari_legacy_berhak_akses() {
        let berkas = Arc::new(BerkasPalsu::baru());
        let _ = berkas.tulis_teks(
            LEGACY_PATH,
            &serde_json::json!({
                "sellers": [],
                "ownerPanels": ["628222"]
            })
            .to_string(),
        );
        let c = Ctx {
            command: "listseller".into(),
            sender: PANEL.into(),
            chat: PANEL.into(),
            is_owner: false,
            berkas,
            ..Ctx::default()
        };
        let r = addseller(c);
        assert!(teks(&r).contains("Belum ada seller terdaftar"), "{}", teks(&r));
    }

    #[test]
    fn list_seller_kosong_dan_terisi() {
        let berkas = Arc::new(BerkasPalsu::baru());
        let buat = |command: &str| Ctx {
            prefix: ".".into(),
            command: command.into(),
            sender: OWNER.into(),
            chat: OWNER.into(),
            is_owner: true,
            berkas: berkas.clone(),
            ..Ctx::default()
        };

        let r = addseller(buat("listseller"));
        assert_eq!(
            teks(&r),
            "📋 *ᴅᴀꜰᴛᴀʀ sᴇʟʟᴇʀ/ʀᴇsᴇʟʟᴇʀ*\n\n> Belum ada seller terdaftar."
        );

        let mut c2 = buat("addseller");
        c2.text = "628777".into();
        let r = addseller(c2);
        assert!(teks(&r).contains("sᴇʟʟᴇʀ ᴅɪᴛᴀᴍʙᴀʜᴋᴀɴ"), "{}", teks(&r));

        let t = teks(&addseller(buat("listseller")));
        assert!(t.contains("1. `628777`"), "{t}");
        assert!(t.contains("> Total: *1* seller"), "{t}");
        assert!(t.contains("_Seller bisa create server (1gb-10gb v1/v2/v3)_"), "{t}");
    }

    #[test]
    fn tambah_seller_lalu_duplikat_ditolak() {
        let mut c = ctx("addseller", OWNER, true);
        c.text = "628777".into();
        let r = addseller(c.clone());
        assert!(teks(&r).contains("✅ *sᴇʟʟᴇʀ ᴅɪᴛᴀᴍʙᴀʜᴋᴀɴ*"));
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "✅"
        ));

        let r = addseller(c);
        assert_eq!(teks(&r), "❌ `628777` sudah menjadi seller.");
    }

    #[test]
    fn tambah_seller_dari_nomor_owner_panel_memicu_downgrade() {
        let berkas = Arc::new(BerkasPalsu::baru());
        let _ = berkas.tulis_teks(
            LEGACY_PATH,
            &serde_json::json!({
                "sellers": [],
                "ownerPanels": ["628777"]
            })
            .to_string(),
        );
        let mut c = ctx("addreseller", OWNER, true);
        c.berkas = berkas;
        c.text = "628777".into();
        let t = teks(&addseller(c));
        assert!(
            t.contains("⚡ Auto-downgrade dari Owner Panel ke Seller"),
            "{t}"
        );
    }

    #[test]
    fn hapus_seller_yang_tidak_ada_dan_yang_ada() {
        let mut c = ctx("delseller", OWNER, true);
        c.text = "628888".into();
        let r = addseller(c.clone());
        assert_eq!(teks(&r), "❌ `628888` bukan seller.");

        // Tambah dulu, lalu hapus.
        c.command = "addseller".into();
        let _ = addseller(c.clone());
        c.command = "delreseller".into();
        let r = addseller(c);
        assert!(teks(&r).contains("✅ *sᴇʟʟᴇʀ ᴅɪʜᴀᴘᴜs*"), "{}", teks(&r));
        assert!(teks(&r).contains("> Total: *0* seller"), "{}", teks(&r));
    }

    #[test]
    fn target_dari_mention_dan_quoted() {
        let mut c = ctx("addseller", OWNER, true);
        c.mentioned = vec!["628555:3@s.whatsapp.net".into()];
        let t = teks(&addseller(c));
        assert!(t.contains("ɴᴏᴍᴏʀ: `628555:3`"), "{t}");

        let mut c = ctx("addseller", OWNER, true);
        c.quoted = Some(oktz_plugin_api::QuotedMessage {
            id: "abc".into(),
            sender: "628444@s.whatsapp.net".into(),
            chat: OWNER.into(),
            text: String::new(),
            kind: oktz_plugin_api::QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        });
        let t = teks(&addseller(c));
        assert!(t.contains("ɴᴏᴍᴏʀ: `628444`"), "{t}");
    }

    #[test]
    fn tanpa_target_sama_sekali_pakai_nomor_sender() {
        // `else { targetUser = getNumber(m.sender) }` — sender selalu valid,
        // jadi kartu bantuan hanya muncul bila sender tak bisa dibaca.
        let c = ctx("addseller", "", true);
        let t = teks(&addseller(c));
        assert!(t.contains("⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*"), "{t}");
        assert!(t.contains("`.addseller @user`"), "{t}");
    }
}
