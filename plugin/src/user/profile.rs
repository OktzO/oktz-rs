//! `plugins/user/profile.js` — profil lengkap user dengan RPG stats.
//!
//! `ponytail:` Sembilan panggilan `sock`/lib JS dibungkus `safe()` di JS —
//! gagal berarti `null` dan barisnya tetap dicetak dengan nilai kosong.
//! Di `Ctx::sock` hanya dua yang tersedia, sisanya dianggap gagal seperti
//! `safe()` yang error (baris tampil `-` / nilai default):
//! - `sock.onWhatsApp` → `sock().ada_whatsapp` (Eksistensi WA + JID kanonik).
//! - `sock.profilePictureUrl` → `sock().foto_profil` (avatar + `ppMedia`).
//! - `fetchStatus`, `getBusinessProfile`, `getCatalog`, `getCollections`,
//!   `getLidFromJid`, `getContact`, `getDevice` → tidak ada di `Sock`
//!   (Bio, Business Info, LID, Device ID, "Bot WhatsApp" selalu `null`).
//! - `m.key?.id` (Device ID) dan `sock.user?.id` / `platform` / Node
//!   `process.version` (Bot Viewpoint) tidak diekspos `Ctx`.
//! - `user.rpg` yang JS tulis-ulang (level/health/mana/stamina) dikirim
//!   lewat `Mutation::SetUser` dengan objek `rpg` utuh dari snapshot store.
//! - `config.isOwner`/`config.isPremium` untuk TARGET ditiru dari
//!   `config().owner.number` + `db.data.owner` + flag `isPremium` user +
//!   `settings.premiumUsers` (salinan `config.js:364-446` yang
//!   terjangkau `Ctx`).

use oktz_plugin_api::{Ctx, MediaSource, Mutation, Reply, plugin};
use serde_json::Value;

use crate::store::done::date_ke_ms;
use crate::user::exp::{EXP_PER_LEVEL, role_of};

/// `config.assets["pp-kosong"]` tidak terisi → fallback JS (`profile.js:135`).
pub(crate) const PP_KOSONG: &str = "https://i.imgur.com/TuItj4L.png";

/// `formatNumber` (`profile.js:24-26`) — `toLocaleString("id-ID")`.
pub(crate) fn format_number(num: i64) -> String {
    crate::tools::kalkulatormbg::thousands_id(num as f64)
}

/// `getLevelBar` (`profile.js:29-38`) — 10 kotak ▰/▱.
pub(crate) fn level_bar(current: i64, target: i64) -> String {
    let total_bars = 10;
    let filled = if target > 0 {
        ((current as f64 / target as f64) * total_bars as f64).floor() as usize
    } else {
        0
    };
    let filled = filled.min(total_bars);
    let empty = total_bars - filled;
    "▰".repeat(filled) + &"▱".repeat(empty)
}

/// Nomor bersih dari jid: split `@` lalu split `:` (`profile.js:60`).
fn phone_of(jid: &str) -> &str {
    jid.split('@').next().unwrap_or("").split(':').next().unwrap_or("")
}

/// `config.isOwner(target)` (`config.js:364`) — bagian yang terjangkau Ctx.
fn target_owner(ctx: &Ctx, target: &str) -> bool {
    let nomor = phone_of(target);
    ctx.config()
        .owner
        .number
        .iter()
        .any(|o| o.split('@').next().unwrap_or(o) == nomor)
        || ctx.db_owners().iter().any(|o| {
            o == target || o.split('@').next().unwrap_or(o) == nomor
        })
}

/// `config.isPremium(target)` (`config.js:406`) — owner, flag user, atau
/// daftar `settings.premiumUsers`.
fn target_premium(ctx: &Ctx, target: &str, user: &Value) -> bool {
    if target_owner(ctx, target) {
        return true;
    }
    if crate::common::js_truthy_value(user.get("isPremium").unwrap_or(&Value::Null)) {
        return true;
    }
    let nomor = phone_of(target);
    ctx.setting_list("premiumUsers").iter().any(|p| {
        p.as_str().is_some_and(|s| s.split('@').next().unwrap_or(s) == nomor)
    })
}

/// Ambil objek user untuk `target` dari snapshot store (`db.getUser(target)`).
fn user_of(ctx: &Ctx, target: &str) -> Value {
    let Ok(teks) = ctx.berkas().baca_teks("database/main/settings.json") else {
        return Value::Null;
    };
    let Ok(root) = serde_json::from_str::<Value>(&teks) else {
        return Value::Null;
    };
    let users = root.get("users").and_then(Value::as_object);
    let Some(users) = users else { return Value::Null };
    if let Some(u) = users.get(target) {
        return u.clone();
    }
    let nomor = phone_of(target);
    users
        .iter()
        .find(|(k, _)| k.split('@').next().unwrap_or(k) == nomor)
        .map(|(_, v)| v.clone())
        .unwrap_or(Value::Null)
}

#[plugin(
    name = "profile",
    alias = ["me", "profil", "myprofile", "my", "stats", "status"],
    category = "user",
    description = "Melihat profil user dengan RPG stats",
    usage = ".profile [@user]",
    example = ".profile",
    cooldown = 5,
    energi = 0,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false
)]
pub fn profile(ctx: Ctx) -> Reply {
    // `m.mentionedJid?.[0] || m.quoted?.sender || m.sender` (`profile.js:55`).
    let target = ctx
        .mentioned
        .first()
        .cloned()
        .or_else(|| ctx.quoted().map(|q| q.sender.clone()))
        .unwrap_or_else(|| ctx.sender.clone());

    let user = user_of(&ctx, &target);

    let is_lid = target.ends_with("@lid");
    let _is_group = target.ends_with("@g.us");

    // `sock.getJid` tidak ada — `resolvedJid = target`.
    let resolved_jid = target.clone();
    let phone = phone_of(&resolved_jid).to_string();

    let on_wa = ctx
        .sock()
        .ada_whatsapp(std::slice::from_ref(&phone))
        .ok()
        .and_then(|v| v.into_iter().next());
    let exists = on_wa.as_ref().is_some_and(|w| w.ada);
    let canonical_jid = on_wa
        .as_ref()
        .map(|w| w.jid.clone())
        .unwrap_or_else(|| resolved_jid.clone());

    // `safe(() => sock.profilePictureUrl(target, "image"))`.
    let pp_url = ctx.sock().foto_profil(&target, "image").ok().flatten();

    let is_owner_user = target_owner(&ctx, &target);
    let is_premium_user = target_premium(&ctx, &target, &user);

    // ---- RPG stats yang JS tulis-ulang ke db (`profile.js:120-129`) ----
    let user_exp = user.get("exp").and_then(Value::as_i64).unwrap_or(0);
    let user_level = user_exp.div_euclid(EXP_PER_LEVEL) + 1;
    let mut rpg = user.get("rpg").cloned().unwrap_or(Value::Null);
    if !rpg.is_object() {
        rpg = serde_json::json!({});
    }
    let rpg_lama = rpg.clone();
    let health = rpg_lama.get("health").and_then(Value::as_i64).unwrap_or(100);
    let mana = rpg_lama.get("mana").and_then(Value::as_i64).unwrap_or(100);
    let stamina = rpg_lama.get("stamina").and_then(Value::as_i64).unwrap_or(100);
    rpg["level"] = serde_json::json!(user_level);
    rpg["health"] = serde_json::json!(health);
    rpg["maxHealth"] = serde_json::json!(100 + (user_level - 1) * 10);
    rpg["mana"] = serde_json::json!(mana);
    rpg["maxMana"] = serde_json::json!(100 + (user_level - 1) * 5);
    rpg["stamina"] = serde_json::json!(stamina);
    rpg["maxStamina"] = serde_json::json!(100 + (user_level - 1) * 5);

    let current_level_exp = (user_level - 1) * EXP_PER_LEVEL;
    let level_up_exp = user_level * EXP_PER_LEVEL;
    let exp_in_level = user_exp - current_level_exp;
    let exp_needed = level_up_exp - current_level_exp;
    let role = role_of(user_level);

    // ---- caption (`profile.js:137-244`) ----
    let mut caption = String::new();
    caption.push_str(&format!("Halo kak @{phone}! 👋\n"));
    caption.push_str("Ini adalah rincian lengkap dari profil, status, dan seluruh aset yang kakak miliki saat ini di dalam sistem bot:\n\n");

    caption.push_str("*〔 👤 INFORMASI PRIBADI 〕*\n");
    let nama = user
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map_or_else(
            || {
                if ctx.push_name.is_empty() {
                    "User".to_string()
                } else {
                    ctx.push_name.clone()
                }
            },
            str::to_string,
        );
    caption.push_str(&format!("- *Nama Asli:* {nama}\n"));
    if user.get("isRegistered").is_some_and(crate::common::js_truthy_value) {
        let reg_name = user.get("regName").and_then(Value::as_str).unwrap_or("");
        let reg_age = user.get("regAge").map(|v| v.to_string()).unwrap_or_default();
        let reg_gender = user.get("regGender").and_then(Value::as_str).unwrap_or("");
        caption.push_str(&format!(
            "- *Nama Daftar:* {reg_name} ({reg_age} tahun, {reg_gender})\n"
        ));
    }
    caption.push_str(&format!("- *Tag / Mention:* @{}\n", target.split('@').next().unwrap_or("")));

    // `${isOwnerUser ? "👑 Owner" : isPremiumUser ? "💎 Premium" : "🆓 Free User"}`
    let status_akun = if is_owner_user {
        "👑 Owner"
    } else if is_premium_user {
        "💎 Premium"
    } else {
        "🆓 Free User"
    };
    caption.push_str(&format!("- *Status Akun:* {status_akun}\n"));
    if user.get("isBanned").is_some_and(crate::common::js_truthy_value) {
        caption.push_str("- *Banned:* 🚫 Ya (Tidak bisa akses fitur bot)\n");
    }
    if let Some(registered_at) = user.get("registeredAt")
        && crate::common::js_truthy_value(registered_at) {
            let ms = date_ke_ms(registered_at);
            if ms != 0 {
                let (y, mo, d, _, _, _) = crate::common::jakarta_parts(ms);
                caption.push_str(&format!("- *Tanggal Terdaftar:* {d:02}/{mo:02}/{y}\n"));
            }
        }
    if let Some(clan) = user.get("clanId").and_then(Value::as_str)
        && !clan.is_empty() {
            caption.push_str(&format!("- *Klan / Guild:* {clan}\n"));
        }
    let spouse = user
        .get("rpg")
        .and_then(|r| r.get("spouse"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Some(s) = &spouse {
        caption.push_str(&format!(
            "- *Pasangan (Spouse):* @{}\n",
            s.split('@').next().unwrap_or("")
        ));
    }

    caption.push_str("\n*〔 ⚔️ RPG STATS & LEVEL 〕*\n");
    caption.push_str(&format!("- *Role / Pangkat:* {role}\n"));
    caption.push_str(&format!("- *Level Saat Ini:* {}\n", rpg["level"]));
    caption.push_str(&format!("- *Total Exp:* {} XP\n", format_number(user_exp)));
    caption.push_str(&format!(
        "- *Kesehatan (Health):* ❤️ {} / {}\n",
        rpg["health"], rpg["maxHealth"]
    ));
    caption.push_str(&format!(
        "- *Mana (Magic):* 💧 {} / {}\n",
        rpg["mana"], rpg["maxMana"]
    ));
    caption.push_str(&format!(
        "- *Stamina:* ⚡ {} / {}\n",
        rpg["stamina"], rpg["maxStamina"]
    ));
    caption.push_str(&format!(
        "- *Progress Ke Level {}:* \n  {} \n  _{} / {} XP_\n",
        user_level + 1,
        level_bar(exp_in_level, exp_needed),
        format_number(exp_in_level),
        format_number(exp_needed)
    ));

    caption.push_str("\n*〔 💰 ASET & KEUANGAN 〕*\n");
    let koin = user.get("koin").and_then(Value::as_i64).unwrap_or(0);
    let bank = rpg_lama.get("bank").and_then(Value::as_i64).unwrap_or(0);
    let energi = user.get("energi").map(|v| v.to_string()).unwrap_or_else(|| "undefined".to_string());
    caption.push_str(&format!(
        "- *Koin Tunai:* 🪙 Rp {} _(Digunakan untuk fitur RPG)_\n",
        format_number(koin)
    ));
    caption.push_str(&format!(
        "- *Uang di Bank:* 🏦 Rp {} _(Aman dari resiko perampokan)_\n",
        format_number(bank)
    ));
    caption.push_str(&format!(
        "- *Sisa Energi:* ⚡ {} _(Dibutuhkan setiap kali memakai command)_\n",
        if is_owner_user || is_premium_user {
            "∞ Tak Terbatas".to_string()
        } else {
            energi
        }
    ));

    caption.push_str("\n*〔 📱 WHATSAPP INFO 〕*\n");
    caption.push_str(&format!("- *Nomor:* +{phone}\n"));
    caption.push_str(&format!("- *Eksistensi WA:* {}\n", if exists { "Ya" } else { "Tidak" }));
    caption.push_str(&format!("- *JID:* {canonical_jid}\n"));
    // lid: m.key?.participant / getLidFromJid / onWhatsApp().lid — tidak ada.
    caption.push_str("- *LID:* -\n");
    caption.push_str(&format!(
        "- *Tipe:* {}\n",
        if _is_group { "Group" } else if is_lid { "LID" } else { "S.WhatsApp.Net" }
    ));
    // `contactQuery?.isBot` / deviceInfo — safe() gagal → false.
    caption.push_str("- *Bot WhatsApp:* Bukan\n");
    // `m.quoted?.key?.id` / `m.key?.id` tidak diekspos Ctx.
    caption.push_str("- *Device ID:* -\n");

    caption.push_str("\n*〔 ℹ️ BIO & PROFILE 〕*\n");
    caption.push_str(&format!(
        "- *Avatar:* {}\n",
        if pp_url.is_some() { "Ada" } else { "Tidak Ada" }
    ));
    caption.push_str("- *Bio:* -\n");
    caption.push_str("- *Bio Set:* -\n");

    caption.push_str("\n*〔 🏢 BUSINESS INFO 〕*\n");
    caption.push_str("- *Tipe Akun:* WhatsApp Biasa\n");

    caption.push_str("\n*〔 🤖 BOT VIEWPOINT 〕*\n");
    caption.push_str("- *Bot JID:* -\n");
    caption.push_str("- *Bot Platform:* -\n");
    caption.push_str("- *Runtime:* Node -\n");

    let inventory = user.get("inventory").and_then(Value::as_object);
    if let Some(inv) = inventory {
        let items: Vec<(&String, &Value)> =
            inv.iter().filter(|(_, qty)| qty.as_i64().is_some_and(|q| q > 0)).collect();
        if !items.is_empty() {
            caption.push_str("\n*〔 🎒 ISI INVENTORY 〕*\n");
            caption.push_str("Barang-barang yang berhasil kakak kumpulkan:\n");
            for (item, qty) in items {
                let nama_item = format!(
                    "{}{}",
                    item.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default(),
                    item.chars().skip(1).collect::<String>()
                );
                caption.push_str(&format!(
                    "- *{nama_item}:* sejumlah {} item\n",
                    qty
                ));
            }
        }
    }

    let unlocked = user.get("unlockedFeatures").and_then(Value::as_array);
    if let Some(list) = unlocked
        && !list.is_empty() {
            caption.push_str("\n*〔 🔓 FITUR PREMIUM TERBUKA 〕*\n");
            caption.push_str("Fitur eksklusif yang sudah kakak beli secara permanen:\n");
            for f in list {
                caption.push_str(&format!("- *{}*\n", f.as_str().unwrap_or("")));
            }
        }

    let mut mentions = vec![target.clone()];
    if let Some(s) = &spouse {
        mentions.push(s.clone());
    }

    // `ppMedia` — `sock.profilePictureUrl` sukses → url; gagal → fallback.
    let media = pp_url.map(MediaSource::url).unwrap_or_else(|| {
        ctx.config()
            .assets
            .get("pp-kosong")
            .map(|p| MediaSource::path(p.to_string()))
            .unwrap_or_else(|| MediaSource::url(PP_KOSONG))
    });

    Reply::Image { media: Some(media), caption, mentions }.mutating(vec![
        Mutation::SetUser {
            jid: target,
            data: serde_json::json!({ "rpg": rpg }),
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};

    const STORE: &str = r#"{"users":{
        "628111":{"name":"Andi","isRegistered":true,"regName":"Andi Wijaya","regAge":21,"regGender":"Laki-laki","registeredAt":"2026-01-01T00:00:00.000Z","exp":25000,"koin":1234567,"energi":50,"inventory":{"ayam":3,"emas":0},"unlockedFeatures":["stickerwm"],"clanId":"GG-01","rpg":{"bank":500000,"spouse":"628555@s.whatsapp.net","health":80}}
    }}"#;

    fn ctx_of(text: &str, command: &str) -> Ctx {
        let berkas = BerkasPalsu::baru();
        berkas.tulis_teks("database/main/settings.json", STORE).unwrap();
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: command.into(),
            push_name: "Kakak".into(),
            berkas: std::sync::Arc::new(berkas),
            ..Ctx::default()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("profile").unwrap();
        assert_eq!(p.meta.name, "profile");
        assert_eq!(p.meta.alias, ["me", "profil", "myprofile", "my", "stats", "status"]);
        assert_eq!(p.meta.category, "user");
        assert_eq!(p.meta.description, "Melihat profil user dengan RPG stats");
        assert_eq!(p.meta.usage, ".profile [@user]");
        assert_eq!(p.meta.example, ".profile");
        assert_eq!(p.meta.cooldown, 5);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn alias_stats_kalah_dari_main_stats_seperti_js() {
        // `stats` milik `main/stats.js` (command) — lookup commands dulu.
        assert_eq!(
            oktz_plugin_api::find("stats").map(|p| p.meta.name.as_str()),
            Some("stats")
        );
        assert_eq!(
            oktz_plugin_api::find("profil").map(|p| p.meta.name.as_str()),
            Some("profile")
        );
    }

    #[test]
    fn profil_sendiri_dengan_data_lengkap() {
        let r = profile(ctx_of("", "profile"));
        let t = r.text_body().unwrap().to_string();
        assert!(t.starts_with("Halo kak @628111! 👋"), "{t}");
        assert!(t.contains("- *Nama Asli:* Andi"));
        assert!(t.contains("- *Nama Daftar:* Andi Wijaya (21 tahun, Laki-laki)"));
        assert!(t.contains("- *Status Akun:* 🆓 Free User"));
        assert!(t.contains("- *Klan / Guild:* GG-01"));
        assert!(t.contains("- *Pasangan (Spouse):* @628555"));
        assert!(t.contains("- *Role / Pangkat:* 🛡️ Warrior"));
        assert!(t.contains("- *Level Saat Ini:* 3"), "exp 25000 → level 3");
        assert!(t.contains("- *Total Exp:* 25.000 XP"));
        assert!(t.contains("❤️ 80 / 120"));
        assert!(t.contains("💧 100 / 110"));
        assert!(t.contains("⚡ 100 / 110"));
        assert!(t.contains("- *Koin Tunai:* 🪙 Rp 1.234.567"));
        assert!(t.contains("- *Uang di Bank:* 🏦 Rp 500.000"));
        assert!(t.contains("- *Sisa Energi:* ⚡ 50"));
        assert!(t.contains("- *Eksistensi WA:* Tidak"), "sock mati = safe() gagal");
        assert!(t.contains("- *Bio:* -"));
        assert!(t.contains("- *Tipe Akun:* WhatsApp Biasa"));
        assert!(t.contains("- *Runtime:* Node -"));
        assert!(t.contains("- *Ayam:* sejumlah 3 item"), "inventory qty > 0");
        assert!(!t.contains("Emas"), "qty 0 tidak dicetak");
        assert!(t.contains("- *stickerwm*"));
    }

    #[test]
    fn rpg_default_ditulis_ke_store() {
        let c = ctx_of("", "profile");
        let r = profile(c);
        for m in r.mutations() {
            if let Mutation::SetUser { jid, data } = m {
                assert_eq!(jid, "628111@s.whatsapp.net");
                assert_eq!(data["rpg"]["level"], 3);
                assert_eq!(data["rpg"]["health"], 80, "health lama dipertahankan");
                assert_eq!(data["rpg"]["maxHealth"], 120);
                assert_eq!(data["rpg"]["mana"], 100);
                assert_eq!(data["rpg"]["maxMana"], 110);
                assert_eq!(data["rpg"]["stamina"], 100);
                assert_eq!(data["rpg"]["maxStamina"], 110);
                assert_eq!(data["rpg"]["bank"], 500000, "field rpg lain utuh");
                return;
            }
        }
        panic!("harus ada SetUser rpg");
    }

    #[test]
    fn avatar_jatuh_ke_asset_pp_kosong() {
        let mut c = ctx_of("", "profile");
        let mut assets = oktz_plugin_api::config::Assets::default();
        assets.0.insert("pp-kosong".into(), "./assets/image/pp-kosong.jpeg".into());
        c.config = std::sync::Arc::new(oktz_plugin_api::config::Runtime {
            assets,
            ..Default::default()
        });
        let r = profile(c);
        match r.without_mutations() {
            Reply::Image { media, .. } => match media.as_ref().unwrap() {
                MediaSource::Path { path, .. } => {
                    assert_eq!(path, "./assets/image/pp-kosong.jpeg")
                }
                other => panic!("harus Path, dapat {other:?}"),
            },
            other => panic!("harus Image, dapat {other:?}"),
        }
    }

    #[test]
    fn mention_menjadi_target_profil() {
        let mut c = ctx_of("", "profile");
        c.mentioned = vec!["628111@s.whatsapp.net".into()];
        let r = profile(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("Halo kak @628111!"));
        match r.without_mutations() {
            Reply::Image { mentions, .. } => {
                assert_eq!(mentions[0], "628111@s.whatsapp.net")
            }
            other => panic!("harus Image, dapat {other:?}"),
        }
    }

    #[test]
    fn user_baru_tanpa_store_tetap_membalas() {
        let c = Ctx {
            sender: "628777@s.whatsapp.net".into(),
            chat: "628777@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "profile".into(),
            ..Ctx::default()
        };
        let r = profile(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("Halo kak @628777!"));
        assert!(t.contains("- *Nama Asli:* User"), "push_name kosong → User");
        assert!(t.contains("- *Status Akun:* 🆓 Free User"));
        assert!(t.contains("❤️ 100 / 100"), "level 1 default");
    }

    #[test]
    fn level_bar_satu_puluh_kotak() {
        assert_eq!(level_bar(0, 10_000), "▱▱▱▱▱▱▱▱▱▱");
        assert_eq!(level_bar(5_000, 10_000), "▰▰▰▰▰▱▱▱▱▱");
        assert_eq!(level_bar(10_000, 10_000), "▰▰▰▰▰▰▰▰▰▰");
        assert_eq!(level_bar(99_999, 10_000), "▰▰▰▰▰▰▰▰▰▰");
    }
}
