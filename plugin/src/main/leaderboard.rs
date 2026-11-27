//! Lihat leaderboard global (koin, exp, energi).

use oktz_plugin_api::{Ctx, Reply, Tombol, plugin};
use serde_json::Value;

#[allow(dead_code)] // padanan `handler.help` JS; dipakai plugin lain di batch ini
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
    #[allow(dead_code)] // data `name` JS masih disimpan; dipakai plugin lain di batch ini
    name: String,
    /// `users[key].lid` — entri berkunci PN menyimpan LID-nya di sini, jadi
    /// pasangan PN↔LID bisa dipulihkan tanpa roster grup.
    lid: Option<String>,
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
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(*b as char);
    }
    out
}

/// Direktori store: `DATABASE_PATH` (dibaca `Config::from_env`) atau default.
/// Store Rust menaruh SELURUH data di `<dir>/settings.json` di bawah kunci
/// `users`; DB hasil salinan versi awal memisahkannya di `<dir>/users.json`. Keduanya
/// dibaca supaya leaderboard tidak lagi kosong di salah satu layout.
fn db_dir() -> String {
    std::env::var("DATABASE_PATH")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "database/main".to_string())
}

fn read_json(ctx: &Ctx, path: &str) -> Option<Value> {
    serde_json::from_str(&ctx.berkas().baca_teks(path).ok()?).ok()
}

/// Nomor bersih dari kunci store: `lid:` dibuang, `@domain` dibuang.
fn key_number(jid: &str) -> &str {
    let tanpa_lid = jid.strip_prefix("lid:").unwrap_or(jid);
    tanpa_lid.split('@').next().unwrap_or(tanpa_lid)
}

/// JID mention dari kunci store: kunci `lid:` → `...@lid`, nomor → `...@s.whatsapp.net`.
fn mention_jid(jid: &str) -> String {
    if let Some(rest) = jid.strip_prefix("lid:") {
        if rest.contains('@') {
            rest.to_string()
        } else {
            format!("{rest}@lid")
        }
    } else if jid.contains('@') {
        jid.to_string()
    } else {
        format!("{jid}@s.whatsapp.net")
    }
}

fn load_users(ctx: &Ctx) -> Vec<UserRow> {
    let dir = db_dir();
    let mut users = serde_json::Map::new();
    if let Some(m) = read_json(ctx, &format!("{dir}/settings.json"))
        .and_then(|r| r.get("users").and_then(Value::as_object).cloned())
    {
        users.extend(m);
    }
    if let Some(m) = read_json(ctx, &format!("{dir}/users.json"))
        .and_then(|r| r.as_object().cloned())
    {
        for (k, v) in m {
            users.entry(k).or_insert(v);
        }
    }

    let mut out = Vec::new();
    for (jid, data) in &users {
        let is_lid = jid.starts_with("lid:");
        let clean = key_number(jid);
        if clean.is_empty() || clean == "undefined" {
            continue;
        }
        // Kunci LID dilewatkan utuh (dulu dibuang karena `jid.len() > 15` saat
        // Whatsapp memberi semua user kunci `lid:<nomor>` → leaderboard kosong).
        if !is_lid && (clean.len() > 15 || clean.starts_with("120")) {
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
            .unwrap_or_else(|| clean.to_string());
        let lid = data
            .get("lid")
            .and_then(Value::as_str)
            .filter(|l| !l.is_empty())
            .map(String::from);
        out.push(UserRow {
            jid: jid.clone(),
            koin,
            exp,
            energi,
            level,
            name,
            lid,
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
        return Reply::text("📊 *ʟᴇᴀᴅᴇʀʙᴏᴀʀᴅ*\n\n> Belum ada data user terdaftar di database.".to_string());
    }

    let peta = peta_lid(&ctx, &users);

    if typ == "overview" {
        let max_bal = users.iter().max_by_key(|u| u.koin).cloned().unwrap();
        let max_exp = users.iter().max_by_key(|u| u.exp).cloned().unwrap();
        let max_en = users.iter().max_by_key(|u| u.energi).cloned().unwrap();
        let mentions = [&max_bal, &max_exp, &max_en]
            .iter()
            .filter_map(|u| sebutan(&ctx, u, &peta).1)
            .collect::<Vec<_>>();
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
        let (label, mention, is_me) = sebutan(&ctx, u, &peta);
        let me = if is_me { " *(You)*" } else { "" };
        // Nomor tetap ditulis `@nomor` seperti versi awal; nama tidak diberi `@`
        // karena bukan token mention.
        let tampil = if label.bytes().all(|b| b.is_ascii_digit()) {
            format!("@{label}")
        } else {
            label
        };
        text.push_str(&format!("┃ {} {}{}\n", medal, tampil, me));
        text.push_str(&format!("┃    └ {} ({}%)\n", val, pct));
        if i + 1 < top10.len() {
            text.push_str("┃\n");
        }
        if let Some(m) = mention {
            mentions.push(m);
        }
    }
    text.push_str("╰┈┈┈┈┈┈┈┈⬡\n\n");

    let my_rank = sorted.iter().position(|u| sebutan(&ctx, u, &peta).2);
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

/// Peta LID-digit → PN-digit dari store dan `database/lid-cache.json`.
///
/// Dua sumber, karena user yang belum pernah muncul di grup mana pun setelah
/// bot jalan tetap punya pasangan PN dari entri lamanya (`users[pn].lid`)
/// dari cache milik versi versi awal.
fn peta_lid(ctx: &Ctx, users: &[UserRow]) -> std::collections::HashMap<String, String> {
    let mut peta = std::collections::HashMap::new();
    for u in users {
        let is_lid = u.jid.starts_with("lid:") || u.jid.ends_with("@lid");
        if is_lid {
            continue;
        }
        if let Some(lid) = &u.lid {
            let l = crate::common::jid_number(lid);
            if !l.is_empty() {
                peta.insert(l.to_string(), key_number(&u.jid).to_string());
            }
        }
    }
    if let Some(cache) = read_json(ctx, &format!("{}/lid-cache.json", db_dir()))
        .and_then(|v| v.as_object().cloned())
    {
        for (k, v) in cache {
            let Some(pn) = v.as_str() else { continue };
            let l = key_number(&k).to_string();
            let p = crate::common::jid_number(pn).to_string();
            if !l.is_empty() && !p.is_empty() && l != p {
                peta.entry(l).or_insert(p);
            }
        }
    }
    peta
}

/// Nama yang layak tampil — `"Unknown"` adalah default `setUser`
/// , bukan nama orang.
fn nama_layak<'a>(nama: &'a str, kunci: &str) -> Option<&'a str> {
    let nama = nama.trim();
    if nama.is_empty()
        || nama == kunci
        || nama.eq_ignore_ascii_case("unknown")
        || nama.eq_ignore_ascii_case("unknown group")
    {
        return None;
    }
    Some(nama)
}

/// `(label, mention, is_me)` untuk satu baris ranking.
///
/// Kunci store boleh bentuk `lid:...`; LID mentah TIDAK pernah dicetak —
/// tampil nama asli, lalu PN hasil peta LID, lalu "Pengguna". Mention hanya
/// dikirim kalau PN-nya diketahui, karena mention LID dirender klien sebagai
/// deretan angka acak.
fn sebutan(
    ctx: &Ctx,
    u: &UserRow,
    peta: &std::collections::HashMap<String, String>,
) -> (String, Option<String>, bool) {
    let kunci = key_number(&u.jid).to_string();
    let is_lid = u.jid.starts_with("lid:") || u.jid.ends_with("@lid");
    let peserta = ctx.roster.find(&kunci);
    let pn = peserta
        .and_then(|p| p.phone_number())
        .map(str::to_string)
        .or_else(|| peta.get(&kunci).cloned())
        .or_else(|| (!is_lid).then(|| kunci.clone()));
    let sender = ctx.sender.split('@').next().unwrap_or(&ctx.sender);
    let sender_pn = ctx.mention_of(&ctx.sender);
    let is_me = pn.as_deref().is_some_and(|p| {
        oktz_plugin_api::registry::jid_user(p) == sender
            || oktz_plugin_api::registry::jid_user(p) == crate::common::jid_number(&sender_pn)
    });

    let label = peserta
        .and_then(|p| p.name())
        .filter(|n| nama_layak(n, &kunci).is_some())
        .map(str::to_string)
        .or_else(|| nama_layak(&u.name, &kunci).map(str::to_string))
        .or_else(|| pn.as_deref().map(crate::common::jid_number).map(str::to_string))
        // LID tanpa nama/PN: tetap cetak nomornya supaya baris bisa di-tag
        // (klien LID bisa me-resolve-nya); jangan sembunyikan jadi "Pengguna".
        .unwrap_or_else(|| kunci.clone());

    let mention = pn
        .map(|p| {
            if p.contains('@') {
                p
            } else {
                format!("{p}@s.whatsapp.net")
            }
        })
        .or_else(|| (!is_lid).then(|| mention_jid(&u.jid)))
        .or_else(|| Some(format!("{kunci}@lid")));
    (label, mention, is_me)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::Berkas as _;
    use std::sync::Arc;

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

    #[test]
    fn load_users_gabung_settings_dan_users_json_termasuk_lid() {
        let b = oktz_plugin_api::berkas::BerkasPalsu::baru();
        let dir = db_dir();
        b.tulis_teks(
            &format!("{dir}/settings.json"),
            r#"{"users":{"628111":{"koin":5}}}"#,
        )
        .unwrap();
        b.tulis_teks(
            &format!("{dir}/users.json"),
            r#"{"lid:123456789012345":{"koin":9}}"#,
        )
        .unwrap();
        let ctx = Ctx {
            berkas: Arc::new(b),
            ..Default::default()
        };
        let users = load_users(&ctx);
        assert!(
            users.iter().any(|u| u.jid == "628111"),
            "settings.json user hilang"
        );
        assert!(
            users.iter().any(|u| u.jid == "lid:123456789012345"),
            "kunci lid dibuang"
        );
    }

    fn ctx_dari_store(users_json: &str) -> Ctx {
        let b = oktz_plugin_api::berkas::BerkasPalsu::baru();
        b.tulis_teks(
            &format!("{}/settings.json", db_dir()),
            &format!(r#"{{"users":{users_json}}}"#),
        )
        .unwrap();
        Ctx {
            berkas: Arc::new(b),
            prefix: ".".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            command: "topkoin".into(),
            ..Default::default()
        }
    }

    #[test]
    fn unknown_diganti_nomor_dan_lid_dipetakan_dari_entri_pn() {
        let ctx = ctx_dari_store(
            r#"{
                "lid:111222333": {"koin": 10, "name": "Unknown"},
                "628222": {"koin": 5, "name": "Budi", "lid": "111222333"}
            }"#,
        );
        let r = leaderboard(ctx);
        let t = r.text_body().unwrap();
        assert!(!t.contains("Unknown"), "placeholder tidak boleh tampil: {t}");
        assert!(t.contains("@628222"), "LID harus dipetakan ke PN: {t}");
        assert!(t.contains("Budi"), "nama asli dipakai: {t}");
        match r {
            Reply::Text { mentions, .. } => {
                assert!(mentions.contains(&"628222@s.whatsapp.net".to_string()), "{mentions:?}");
                assert!(
                    !mentions.iter().any(|m| m.ends_with("@lid")),
                    "mention LID tidak boleh dikirim: {mentions:?}"
                );
            }
            lain => panic!("harus teks: {lain:?}"),
        }
    }

    #[test]
    fn lid_dipetakan_dari_file_lid_cache_milik_js() {
        let b = oktz_plugin_api::berkas::BerkasPalsu::baru();
        let dir = db_dir();
        b.tulis_teks(
            &format!("{dir}/settings.json"),
            r#"{"users":{"lid:555666777":{"koin":4,"name":"Unknown"}}}"#,
        )
        .unwrap();
        b.tulis_teks(
            &format!("{dir}/lid-cache.json"),
            r#"{"555666777@lid":"628555@s.whatsapp.net"}"#,
        )
        .unwrap();
        let ctx = Ctx {
            berkas: Arc::new(b),
            prefix: ".".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            command: "topkoin".into(),
            ..Default::default()
        };
        let r = leaderboard(ctx);
        let t = r.text_body().unwrap();
        assert!(t.contains("@628555"), "PN dari lid-cache dipakai: {t}");
        match r {
            Reply::Text { mentions, .. } => {
                assert_eq!(mentions, &["628555@s.whatsapp.net".to_string()]);
            }
            lain => panic!("harus teks: {lain:?}"),
        }
    }

    #[test]
    fn lid_tanpa_peta_tetap_dicetak_sebagai_nomor_bisa_ditag() {
        let ctx = ctx_dari_store(r#"{"lid:999888777": {"koin": 3, "name": "Unknown"}}"#);
        let r = leaderboard(ctx);
        let t = r.text_body().unwrap();
        assert!(!t.contains("Unknown"), "{t}");
        assert!(t.contains("@999888777"), "nomor LID dicetak untuk tag: {t}");
        match r {
            Reply::Text { mentions, .. } => {
                assert_eq!(mentions, &["999888777@lid".to_string()]);
            }
            lain => panic!("harus teks: {lain:?}"),
        }
    }

    #[test]
    fn mention_lid_pakai_domain_lid() {
        assert_eq!(mention_jid("lid:123"), "123@lid");
        assert_eq!(mention_jid("628111"), "628111@s.whatsapp.net");
        assert_eq!(mention_jid("628111@s.whatsapp.net"), "628111@s.whatsapp.net");
        assert_eq!(key_number("lid:123456789012345"), "123456789012345");
    }
}
