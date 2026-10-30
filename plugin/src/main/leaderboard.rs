//! `plugins/main/leaderboard.js` — lihat leaderboard global (koin, exp, energi).

use oktz_plugin_api::{Ctx, Reply, Tombol, plugin};
use serde_json::Value;

pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🏆 *Lᴇᴀᴅᴇʀʙᴏᴀʀᴅ*\n\n> Lihat leaderboard global (koin, exp, energi)\n\n`{prefix}leaderboard`",
        prefix = prefix
    )
}

#[derive(Clone)]
struct UserRow {
    jid: String,
    koin: i64,
    exp: i64,
    energi: i64,
    level: i64,
    name: String,
}

fn format_number(num: i64) -> String {
    let n = num as f64;
    if num >= 1_000_000_000_000 {
        return format!("{:.2}T", n / 1_000_000_000_000.0);
    }
    if num >= 1_000_000_000 {
        return format!("{:.2}B", n / 1_000_000_000.0);
    }
    if num >= 1_000_000 {
        return format!("{:.2}M", n / 1_000_000.0);
    }
    if num >= 1_000 {
        return format!("{:.2}K", n / 1_000.0);
    }
    // ribuan dengan titik (id-ID)
    let s = num.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push('.');
        }
        out.push(*b as char);
    }
    out
}

fn load_users(ctx: &Ctx) -> Vec<UserRow> {
    let mut out = Vec::new();
    let Ok(teks) = ctx.berkas().baca_teks("database/main/settings.json") else {
        return out;
    };
    let Ok(root): Result<Value, _> = serde_json::from_str(&teks) else {
        return out;
    };
    let Some(users) = root.get("users").and_then(Value::as_object) else {
        return out;
    };
    for (jid, data) in users {
        if jid.is_empty() || jid == "undefined" {
            continue;
        }
        if jid.len() > 15 || jid.starts_with("120") {
            continue;
        }
        let koin = data.get("koin").and_then(Value::as_i64).unwrap_or(0);
        let exp = data
            .get("rpg")
            .and_then(|r| r.get("exp"))
            .and_then(Value::as_i64)
            .or_else(|| data.get("exp").and_then(Value::as_i64))
            .unwrap_or(0);
        let energi = data.get("energi").and_then(Value::as_i64).unwrap_or(0);
        let level = data
            .get("rpg")
            .and_then(|r| r.get("level"))
            .and_then(Value::as_i64)
            .or_else(|| data.get("level").and_then(Value::as_i64))
            .unwrap_or(1);
        let name = data
            .get("name")
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| jid.split('@').next().unwrap_or(jid).to_string());
        out.push(UserRow {
            jid: jid.clone(),
            koin,
            exp,
            energi,
            level,
            name,
        });
    }
    out
}

#[plugin(
    name = "leaderboard",
    alias = ["lb", "top", "leaderboard", "ranking", "rank", "topglobal",
             "topbalance", "topbal", "topkoin", "topcoin", "topmoney",
             "toplimit", "topexp", "topxp", "toplevel",
             "topenergi", "topenergy"],
    category = "main",
    description = "Lihat leaderboard global (koin, exp, energi)",
    usage = ".leaderboard",
    example = ".topkoin",
    cooldown = 10,
    energi = 0,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn leaderboard(ctx: Ctx) -> Reply {
    let cmd = ctx.command.to_lowercase();
    let mut typ = "overview".to_string();
    if cmd.contains("koin") || cmd.contains("coin") || cmd.contains("bal") || cmd.contains("money") {
        typ = "koin".to_string();
    } else if cmd.contains("exp") || cmd.contains("xp") || cmd.contains("level") {
        typ = "exp".to_string();
    } else if cmd.contains("energi") || cmd.contains("energy") {
        typ = "energi".to_string();
    } else if let Some(arg) = ctx.args.first() {
        let a = arg.to_lowercase();
        if ["koin", "coin", "bal", "balance", "money"].contains(&a.as_str()) {
            typ = "koin".to_string();
        } else if ["exp", "xp", "level"].contains(&a.as_str()) {
            typ = "exp".to_string();
        } else if ["energi", "energy"].contains(&a.as_str()) {
            typ = "energi".to_string();
        }
    }

    let users = load_users(&ctx);
    if users.is_empty() {
        return Reply::text(format!(
            "📊 *ʟᴇᴀᴅᴇʀʙᴏᴀʀᴅ*\n\n> Belum ada data user terdaftar di database."
        ));
    }

    let sender_jid = ctx.sender.split('@').next().unwrap_or(&ctx.sender).to_string();

    if typ == "overview" {
        let max_bal = users.iter().max_by_key(|u| u.koin).cloned().unwrap();
        let max_exp = users.iter().max_by_key(|u| u.exp).cloned().unwrap();
        let max_en = users.iter().max_by_key(|u| u.energi).cloned().unwrap();
        let mentions = vec![
            if max_bal.jid.contains('@') { max_bal.jid.clone() } else { format!("{}@s.whatsapp.net", max_bal.jid) },
            if max_exp.jid.contains('@') { max_exp.jid.clone() } else { format!("{}@s.whatsapp.net", max_exp.jid) },
            if max_en.jid.contains('@') { max_en.jid.clone() } else { format!("{}@s.whatsapp.net", max_en.jid) },
        ];
        let text = "🏆 *LEADERBOARD OVERVIEW* 🏆\n\n_Pilih tombol di bawah untuk melihat ranking!_".to_string();
        // tombol quick_reply
        let buttons = vec![
            Tombol::baru(oktz_plugin_api::AksiTombol::BalasanCepat, serde_json::json!({"display_text":"💰 Top Koin","id":format!("{}topkoin", ctx.prefix)})),
            Tombol::baru(oktz_plugin_api::AksiTombol::BalasanCepat, serde_json::json!({"display_text":"✨ Top EXP","id":format!("{}topexp", ctx.prefix)})),
            Tombol::baru(oktz_plugin_api::AksiTombol::BalasanCepat, serde_json::json!({"display_text":"⚡ Top Energi","id":format!("{}topenergi", ctx.prefix)})),
        ];
        let mut kartu = oktz_plugin_api::kartu::Kartu::baru(&text);
        for b in buttons { kartu.buttons.push(b); }
        return Reply::Interactive { kartu: Box::new(kartu) }.with_mentions(mentions);
    }

    let (title, emoji, field) = match typ.as_str() {
        "koin" => ("TOP GLOBAL KOIN", "💰", "koin"),
        "exp" => ("TOP GLOBAL LEVEL", "✨", "exp"),
        _ => ("TOP GLOBAL ENERGI", "⚡", "energi"),
    };

    let mut sorted = users.clone();
    sorted.sort_by(|a, b| {
        let va = match field { "koin" => a.koin, "exp" => a.exp, _ => a.energi };
        let vb = match field { "koin" => b.koin, "exp" => b.exp, _ => b.energi };
        vb.cmp(&va)
    });
    let top10: Vec<_> = sorted.iter().take(10).cloned().collect();
    let total_field: i64 = sorted.iter().map(|u| match field { "koin" => u.koin, "exp" => u.exp, _ => u.energi }).sum();

    let mut text = format!("🏆 *{}* 🏆\n\nPeringkat para penguasa tertinggi saat ini!\n\n╭┈┈⬡「 {} *RANKING* 」\n", title, emoji);
    let mut mentions = Vec::new();
    for (i, u) in top10.iter().enumerate() {
        let medal = ["🥇", "🥈", "🥉", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟"]
            .get(i)
            .copied()
            .unwrap_or("•");
        let val = match field {
            "koin" => format!("Rp {}", format_number(u.koin)),
            "exp" => format!("Lv. {} ({}) XP", u.level, format_number(u.exp)),
            _ => format!("{} Energi", format_number(u.energi)),
        };
        let v = match field { "koin" => u.koin, "exp" => u.exp, _ => u.energi };
        let pct = if total_field > 0 {
            format!("{:.1}", (v as f64 / total_field as f64) * 100.0)
        } else {
            "0".to_string()
        };
        let is_me = if u.jid.split('@').next().unwrap_or(&u.jid) == sender_jid {
            " *(You)*"
        } else {
            ""
        };
        text.push_str(&format!("┃ {} @{}{}\n", medal, u.jid.split('@').next().unwrap_or(&u.jid), is_me));
        text.push_str(&format!("┃    └ {} ({}%)\n", val, pct));
        if i + 1 < top10.len() {
            text.push_str("┃\n");
        }
        mentions.push(if u.jid.contains('@') {
            u.jid.clone()
        } else {
            format!("{}@s.whatsapp.net", u.jid)
        });
    }
    text.push_str("╰┈┈┈┈┈┈┈┈⬡\n\n");

    let my_rank = sorted.iter().position(|u| u.jid.split('@').next().unwrap_or(&u.jid) == sender_jid);
    if let Some(idx) = my_rank {
        text.push_str(&format!(
            "> Posisi kamu: *#{}* dari *{}* user.",
            idx + 1,
            format_number(users.len() as i64)
        ));
    } else {
        text.push_str("> Kamu belum terdaftar di database.");
    }

    Reply::text(text).with_mentions(mentions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_of(cmd: &str) -> Ctx {
        Ctx {
            text: String::new(),
            args: Vec::new(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: cmd.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn selalu_membalas() {
        let r = leaderboard(ctx_of("leaderboard"));
        let t = r.text_body().unwrap();
        assert!(t.contains("LEADERBOARD") || t.contains("ʟᴇᴀᴅᴇʀʙᴏᴀʀᴅ"));
    }

    #[test]
    fn alias_topkoin_masih_pakai_handler_sama() {
        let r = leaderboard(ctx_of("topkoin"));
        assert!(r.text_body().is_some());
    }
}
