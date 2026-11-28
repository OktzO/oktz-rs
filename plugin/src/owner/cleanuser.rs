//! Hapus entri `users`/`groups` yang tidak mungkin dipakai bot.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};

fn kunci_user_valid(key: &str) -> bool {
    if let Some(lid) = key.strip_prefix("lid:") {
        return (10..=20).contains(&lid.len()) && lid.bytes().all(|b| b.is_ascii_digit());
    }
    !key.is_empty()
        && key.len() <= 15
        && key.bytes().all(|b| b.is_ascii_digit())
        && !key.starts_with("120")
}

fn kunci_grup_valid(key: &str) -> bool {
    let Some(id) = key.strip_suffix("@g.us") else {
        return false;
    };
    id.len() >= 10 && id.bytes().all(|b| b.is_ascii_digit())
}

#[plugin(
    name = "cleanuser",
    alias = ["bersihuser", "hapususerinvalid", "cleangroup"],
    category = "owner",
    description = "Hapus user/grup invalid dari database",
    usage = ".cleanuser",
    example = ".cleanuser",
    cooldown = 60,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cleanuser(ctx: Ctx) -> Reply {
    let Some(store) = ctx.store() else {
        return Reply::text("❌ Snapshot store tidak tersedia");
    };

    let mut mutations = Vec::new();
    let mut contoh_user = Vec::new();
    let mut contoh_grup = Vec::new();
    let mut user = 0usize;
    let mut grup = 0usize;

    store.for_each_user(&mut |key, value| {
        if kunci_user_valid(key) && value.is_object() {
            return;
        }
        if contoh_user.len() < 10 {
            contoh_user.push(key.to_string());
        }
        user += 1;
        mutations.push(Mutation::DeleteUser {
            key: key.to_string(),
        });
    });
    store.for_each_group(&mut |key, value| {
        if kunci_grup_valid(key) && value.is_object() {
            return;
        }
        if contoh_grup.len() < 10 {
            contoh_grup.push(key.to_string());
        }
        grup += 1;
        mutations.push(Mutation::DeleteGroup {
            key: key.to_string(),
        });
    });

    let mut teks = format!(
        "╭┈┈⬡「 🧹 *ᴄʟᴇᴀɴ ᴅᴀᴛᴀ* 」\n┃\n┃ 👤 User invalid: *{user}*\n┃ 👥 Grup invalid: *{grup}*"
    );
    if !contoh_user.is_empty() {
        teks.push_str(&format!("\n┃\n┃ User: `{}`", contoh_user.join("`, `")));
    }
    if !contoh_grup.is_empty() {
        teks.push_str(&format!("\n┃\n┃ Grup: `{}`", contoh_grup.join("`, `")));
    }
    teks.push_str("\n┃\n┃ ✅ Data valid tidak disentuh\n╰┈┈⬡");

    Reply::text(teks).with_reaction("🧹").mutating(mutations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn kunci_user_valid_hanya_nomor_dan_lid() {
        assert!(kunci_user_valid("628123456789"));
        assert!(kunci_user_valid("lid:123456789012345"));
        assert!(!kunci_user_valid("628123456789@s.whatsapp.net"));
        assert!(!kunci_user_valid("628123456789:5"));
        assert!(!kunci_user_valid("unknown"));
        assert!(!kunci_user_valid("120363123456789"));
        assert!(!kunci_user_valid("62812abc"));
        assert!(!kunci_user_valid(""));
        assert!(kunci_user_valid("62812"), "nomor pendek bukan bukti junk");
    }

    #[test]
    fn kunci_grup_valid_hanya_jid_grup() {
        assert!(kunci_grup_valid("120363123456789012@g.us"));
        assert!(!kunci_grup_valid("120363123456789012"));
        assert!(!kunci_grup_valid("bukan-grup"));
        assert!(!kunci_grup_valid("120363123456789012@s.whatsapp.net"));
    }

    #[test]
    fn terdaftar_dengan_meta_owner() {
        let p = find("cleanuser").unwrap();
        assert_eq!(p.meta.category, "owner");
        assert!(p.meta.is_owner);
        assert_eq!(
            p.meta.alias,
            ["bersihuser", "hapususerinvalid", "cleangroup"]
        );
    }
}
