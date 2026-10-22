//! `plugins/owner/resetdb.js` — reset semua data database.
//!
//! Port dari nol. Yang dipindah:
//! - guard owner (`resetdb.js:25-27`).
//! - alur dua langkah dengan TTL 60 detik (`:29-55`).
//! - teks peringatan (`:34-46`) dan teks timeout (`:52`).
//! - teks hasil reset (`:88-97`).
//!
//! `ponytail:` JS menyalin `database/db.json` ke `database/db_backup_<ts>.json`
//! lalu mengosongkan tabel `users`, `groups`, dan `clans`
//! (`resetdb.js:57-86`). Store Rust adalah satu basis data (SQLite/JSON)
//! dengan tabel terpisah dan `Mutation` hanya punya `SetUser`/`SetGroup`/
//! `SetSetting` — tidak ada "kosongkan tabel" maupun "hapus semua user".
//! Karena itu jumlah user/group/clan yang dilaporkan selalu `0` dan hanya
//! key `settings`/`sewa` yang bisa disetel ulang ke default JS.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::Value;

/// Prefix key session penunggu konfirmasi (`resetdb.js:22`).
const PENDING_PREFIX: &str = "resetdb:pending:";
/// TTL 60 detik (`resetdb.js:50`).
const TTL_MS: i64 = 60_000;

fn pending_key(ctx: &Ctx) -> String {
    format!("{PENDING_PREFIX}{}", ctx.sender)
}

/// `global.resetDbPending[m.sender]` + TTL 60 detik (`resetdb.js:49-53`).
fn pending(ctx: &Ctx) -> bool {
    let key = pending_key(ctx);
    let Some(v) = ctx.session_get(&key) else {
        return false;
    };
    let ts = v.as_i64().unwrap_or(0);
    if crate::common::now_ms() - ts > TTL_MS {
        ctx.session_take(&key);
        return false;
    }
    true
}

/// Teks peringatan (`resetdb.js:34-46`).
fn peringatan() -> String {
    "⚠️ *ᴘᴇʀɪɴɢᴀᴛᴀɴ!*\n\n\
     > Ini akan menghapus SEMUA data:\n\
     > • Data user\n\
     > • Data group\n\
     > • Data clan\n\
     > • Semua statistik\n\n\
     ╭┈┈⬡「 ⚠️ *ᴋᴏɴғɪʀᴍᴀsɪ* 」\n\
     ┃ Ketik: *.resetdb confirm*\n\
     ┃ dalam 60 detik\n\
     ╰┈┈┈┈┈┈┈┈⬡\n\n\
     > ❌ Aksi ini TIDAK BISA dibatalkan!"
        .to_string()
}

#[plugin(
    name = "resetdb",
    alias = ["cleardb", "wipedb"],
    category = "owner",
    description = "Reset semua data database",
    usage = ".resetdb [confirm]",
    example = ".resetdb confirm",
    cooldown = 60,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn resetdb(ctx: Ctx) -> Reply {
    if !ctx.is_owner {
        return Reply::text("❌ *Owner Only!*");
    }

    let confirm = ctx
        .args
        .first()
        .map(|a| a.to_lowercase())
        .unwrap_or_default();

    if confirm != "confirm" {
        ctx.session_set(
            &pending_key(&ctx),
            serde_json::json!(crate::common::now_ms()),
        );
        return Reply::text(peringatan());
    }

    if !pending(&ctx) {
        return Reply::text("❌ Timeout! Ketik *.resetdb* ulang untuk memulai.");
    }
    ctx.session_take(&pending_key(&ctx));

    // `ponytail:` tabel `users`/`groups`/`clans` tidak bisa dikosongkan dari
    // API plugin; hanya key settings/sewa yang bisa disetel ulang.
    let mut mutasi: Vec<Mutation> = Vec::new();
    if ctx.setting("selfMode").is_some() {
        mutasi.push(Mutation::SetSetting {
            key: "selfMode".to_string(),
            value: Value::Bool(false),
        });
    }
    if ctx.setting("sewa").is_some() {
        mutasi.push(Mutation::SetSetting {
            key: "sewa".to_string(),
            value: serde_json::json!({ "enabled": false, "groups": {} }),
        });
    }

    let backup = format!("db_backup_{}.json", crate::common::now_ms());

    Reply::text(format!(
        "✅ *ᴅᴀᴛᴀʙᴀsᴇ ᴅɪʀᴇsᴇᴛ!*\n\n\
         ╭┈┈⬡「 📊 *ᴅᴀᴛᴀ ᴅɪʜᴀᴘᴜs* 」\n\
         ┃ 👤 Users: 0\n\
         ┃ 👥 Groups: 0\n\
         ┃ ⚔️ Clans: 0\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n\
         > Backup disimpan di:\n\
         > `{backup}`"
    ))
    .mutating(mutasi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(teks: &str, owner: bool) -> Reply {
        resetdb(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            is_owner: owner,
            prefix: ".".into(),
            command: "resetdb".into(),
            sessions: oktz_plugin_api::new_session_store(),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("resetdb").unwrap();
        assert_eq!(p.meta.name, "resetdb");
        assert_eq!(p.meta.alias, ["cleardb", "wipedb"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Reset semua data database");
        assert_eq!(p.meta.usage, ".resetdb [confirm]");
        assert_eq!(p.meta.example, ".resetdb confirm");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["cleardb", "wipedb"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("resetdb"), "{a}");
        }
    }

    #[test]
    fn bukan_owner_ditolak() {
        let r = run("", false);
        assert_eq!(teks(&r), "❌ *Owner Only!*");
    }

    #[test]
    fn tanpa_confirm_menampilkan_peringatan() {
        let r = run("", true);
        assert_eq!(
            teks(&r),
            "⚠️ *ᴘᴇʀɪɴɢᴀᴛᴀɴ!*\n\n\
             > Ini akan menghapus SEMUA data:\n\
             > • Data user\n\
             > • Data group\n\
             > • Data clan\n\
             > • Semua statistik\n\n\
             ╭┈┈⬡「 ⚠️ *ᴋᴏɴғɪʀᴍᴀsɪ* 」\n\
             ┃ Ketik: *.resetdb confirm*\n\
             ┃ dalam 60 detik\n\
             ╰┈┈┈┈┈┈┈┈⬡\n\n\
             > ❌ Aksi ini TIDAK BISA dibatalkan!"
        );
    }

    #[test]
    fn confirm_tanpa_permintaan_memberi_timeout() {
        let r = run("confirm", true);
        assert_eq!(teks(&r), "❌ Timeout! Ketik *.resetdb* ulang untuk memulai.");
    }

    #[test]
    fn confirm_setelah_permintaan_berhasil() {
        let mut ctx = Ctx {
            text: String::new(),
            sender: ME.into(),
            chat: ME.into(),
            is_owner: true,
            prefix: ".".into(),
            command: "resetdb".into(),
            sessions: oktz_plugin_api::new_session_store(),
            ..Ctx::default()
        };
        let _ = resetdb(ctx.clone());
        assert!(ctx.session_get("resetdb:pending:628111@s.whatsapp.net").is_some());
        ctx.text = "confirm".into();
        ctx.args = vec!["confirm".to_string()];
        let r = resetdb(ctx.clone());
        let t = teks(&r);
        assert!(t.starts_with("✅ *ᴅᴀᴛᴀʙᴀsᴇ ᴅɪʀᴇsᴇᴛ!*"), "{t}");
        assert!(t.contains("┃ 👤 Users: 0"), "{t}");
        assert!(t.contains("┃ 👥 Groups: 0"), "{t}");
        assert!(t.contains("┃ ⚔️ Clans: 0"), "{t}");
        assert!(t.contains("> Backup disimpan di:\n> `db_backup_"), "{t}");
        assert!(ctx.session_get("resetdb:pending:628111@s.whatsapp.net").is_none());
    }
}
