//! `plugins/owner/addsewa.js` — tambah grup ke whitelist sewa + auto join.
//!
//! Port dari nol. Yang dipindah:
//! - parsing durasi (`parseDuration` / `formatDuration`, `addsewa.js:22-54`).
//! - resolusi grup dari link undangan atau `@g.us` (`resolveGroupId`, `:56-84`).
//! - penyimpanan entri whitelist (`db.db.data.sewa.groups[groupId]`, `:159-166`).
//! - teks hasil (`📝 *TAMBAH SEWA*` sampai `✅ *SEWA BERHASIL DITAMBAHKAN*`).
//!
//! `ponytail:` tabel `db.db.data.sewa` adalah tabel top-level, bukan milik
//! user/grup — dipetakan ke key global `sewa` di `settings`, sama seperti
//! `owner/sewabot.rs` dan `owner/listsewa.rs`.
//!
//! `ponytail:` `sock.sendText(groupId, ...)` (pesan perkenalan ke grup,
//! `addsewa.js:184-191`) tidak punya padanan: `Reply` hanya sampai ke
//! `caller.chat`. Teks balasan tetap utuh karena `sendText` dibungkus
//! `try {} catch {}` yang tidak mengubah teks.
//!
//! `ponytail:` cek "bot sudah ada di grup" (`tryJoinGroup`, `:96-101`) butuh
//! JID bot sendiri yang tidak ada di `Ctx`; jadi `terima_undangan` selalu
//! dicoba dan hasilnya dilaporkan apa adanya.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::{Map, Value};

/// Key global tempat `db.db.data.sewa` dipetakan.
const SEWA_KEY: &str = "sewa";

/// `["lifetime","permanent","forever","unlimited"]` (`addsewa.js:24-28`).
const LIFETIME: [&str; 4] = ["lifetime", "permanent", "forever", "unlimited"];

/// `const multiplier = { i: 60000, h: 3600000, d: 86400000, m: 2592000000,
/// y: 31536000000 }` (`addsewa.js:33-38`) dalam milidetik.
fn multiplier(unit: char) -> Option<i64> {
    match unit {
        'i' => Some(60_000),
        'h' => Some(3_600_000),
        'd' => Some(86_400_000),
        'm' => Some(2_592_000_000),
        'y' => Some(31_536_000_000),
        _ => None,
    }
}

/// `str.match(/^(\d+)([iIdDmMyYhH])$/)` (`addsewa.js:29`).
///
/// Balikin `(angka, satuan kecil)`.
fn cocok_durasi(s: &str) -> Option<(i64, char)> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 {
        return None;
    }
    let (digit_part, unit) = chars.split_at(chars.len() - 1);
    if digit_part.is_empty() || !digit_part.iter().all(char::is_ascii_digit) {
        return None;
    }
    let unit = unit[0].to_ascii_lowercase();
    if !matches!(unit, 'i' | 'h' | 'd' | 'm' | 'y') {
        return None;
    }
    let value: i64 = digit_part.iter().collect::<String>().parse().ok()?;
    Some((value, unit))
}

/// `parseDuration(str)` (`addsewa.js:22-41`).
///
/// `Ok(Some(ms))` = durasi normal, `Ok(None)` = `Infinity` (lifetime),
/// `Err(())` = format tidak valid (`null` di JS).
fn parse_durasi(s: &str) -> Result<Option<i64>, ()> {
    let lower = s.to_lowercase();
    if LIFETIME.contains(&lower.as_str()) {
        return Ok(None);
    }
    let Some((value, unit)) = cocok_durasi(s) else {
        return Err(());
    };
    let Some(mult) = multiplier(unit) else {
        return Err(());
    };
    Ok(Some(crate::common::now_ms() + value * mult))
}

/// `formatDuration(str)` (`addsewa.js:43-54`).
fn format_durasi(s: &str) -> String {
    let lower = s.to_lowercase();
    if LIFETIME.contains(&lower.as_str()) {
        return "Permanent".to_string();
    }
    let Some((value, unit)) = cocok_durasi(s) else {
        return s.to_string();
    };
    let satuan = match unit {
        'i' => "menit",
        'h' => "jam",
        'd' => "hari",
        'm' => "bulan",
        'y' => "tahun",
        _ => "",
    };
    format!("{value} {satuan}")
}

/// `timeHelper.fromTimestamp(expiredAt, "D MMMM YYYY HH:mm")` (`addsewa.js:170`).
fn expired_str(ms: i64) -> String {
    format!(
        "{} {}",
        crate::common::date_long(ms),
        crate::common::time_hm(ms)
    )
}

/// Tabel `sewa` setelah init (`addsewa.js:111-114`).
fn tabel_sewa(ctx: &Ctx) -> Value {
    match ctx.setting(SEWA_KEY) {
        Some(v) if crate::common::js_truthy_value(v) => v.clone(),
        _ => serde_json::json!({ "enabled": false, "groups": {} }),
    }
}

/// Mutasi: tulis tabel `sewa` (dengan `groups[groupId]` baru).
fn simpan_grup(ctx: &Ctx, group_id: &str, nama: &str, expired_at: i64, lifetime: bool) -> Mutation {
    let tabel = tabel_sewa(ctx);
    let mut objek = tabel.as_object().cloned().unwrap_or_default();
    let mut groups = objek
        .get("groups")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    groups.insert(
        group_id.to_string(),
        serde_json::json!({
            "name": nama,
            "addedAt": crate::common::now_ms(),
            "expiredAt": expired_at,
            "isLifetime": lifetime,
            "addedBy": ctx.sender,
        }),
    );
    objek.insert("groups".to_string(), Value::Object(groups));
    Mutation::SetSetting {
        key: SEWA_KEY.to_string(),
        value: Value::Object(objek),
    }
}

/// Mutasi init tabel `sewa` kalau belum ada (`addsewa.js:111-114`).
fn init_kalau_kosong(ctx: &Ctx) -> Option<Mutation> {
    crate::common::js_falsy(ctx.setting(SEWA_KEY)).then(|| Mutation::SetSetting {
        key: SEWA_KEY.to_string(),
        value: serde_json::json!({ "enabled": false, "groups": {} }),
    })
}

/// Gabung balasan dengan mutasi init (biar tidak/doUBLE).
fn dengan_init(reply: Reply, init: Option<Mutation>) -> Reply {
    match init {
        Some(m) => reply.with_mutation(m),
        None => reply,
    }
}

/// `resolveGroupId(sock, input)` (`addsewa.js:56-84`).
///
/// Balikin `Some((id, nama, invite_code))` atau `None` (hanya untuk link
/// undangan yang gagal dibaca).
fn resolve_grup(ctx: &Ctx, input: &str) -> Option<(String, String, Option<String>)> {
    if input.contains("chat.whatsapp.com/") {
        let kode = input
            .split("chat.whatsapp.com/")
            .nth(1)?
            .split([' ', '?'])
            .next()
            .unwrap_or("")
            .to_string();
        if kode.is_empty() {
            return None;
        }
        let meta = ctx.sock().info_undangan(&kode).ok()?;
        let id = meta.get("id").and_then(Value::as_str)?.to_string();
        let nama = meta
            .get("subject")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("Unknown")
            .to_string();
        return Some((id, nama, Some(kode)));
    }
    let group_id = if input.contains("@g.us") {
        input.to_string()
    } else {
        format!("{input}@g.us")
    };
    match ctx.sock().metadata_grup(&group_id) {
        Ok(meta) => {
            let nama = meta
                .get("subject")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("Unknown")
                .to_string();
            Some((group_id, nama, None))
        }
        Err(_) => Some((group_id, "Unknown".to_string(), None)),
    }
}

/// `tryJoinGroup(sock, inviteCode, groupId)` (`addsewa.js:86-107`).
///
/// Balikin `(joined, alasan)`.
fn try_join(ctx: &Ctx, invite_code: Option<&String>) -> (bool, String) {
    let Some(kode) = invite_code else {
        return (
            false,
            "Tidak ada invite code, tambahkan bot secara manual".to_string(),
        );
    };
    match ctx.sock().terima_undangan(kode) {
        Ok(_) => (true, "Bot berhasil join grup".to_string()),
        Err(e) => (false, e.pesan),
    }
}

/// Panduan lengkap (`addsewa.js:118-135`).
fn panduan(prefix: &str) -> String {
    format!(
        "📝 *TAMBAH SEWA*\n\n\
         Format: *{prefix}addsewa <link/id> <durasi>*\n\n\
         *FORMAT DURASI:*\n\
         • 30i = 30 menit\n\
         • 12h = 12 jam\n\
         • 7d = 7 hari\n\
         • 1m = 1 bulan (30 hari)\n\
         • 1y = 1 tahun\n\
         • lifetime = Permanent\n\n\
         *INPUT GRUP:*\n\
         • Link: https://chat.whatsapp.com/xxx\n\
         • ID: 120363xxx@g.us\n\n\
         *CONTOH:*\n\
         • {prefix}addsewa https://chat.whatsapp.com/xxx 30d\n\
         • {prefix}addsewa 120363xxx 1m\n\n\
         💡 Jika pakai link, bot akan otomatis join ke grup tersebut!"
    )
}

#[plugin(
    name = "addsewa",
    alias = ["sewaadd", "tambahsewa"],
    category = "owner",
    description = "Tambah grup ke whitelist sewa + auto join",
    usage = ".addsewa <link/id grup> <durasi>",
    example = ".addsewa https://chat.whatsapp.com/xxx 30d",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn addsewa(ctx: Ctx) -> Reply {
    let init = init_kalau_kosong(&ctx);

    if ctx.args.len() < 2 {
        return dengan_init(Reply::text(panduan(&ctx.prefix)), init);
    }

    let input = ctx.args[0].clone();
    let duration_str = ctx.args[1].clone();

    let expired_at = match parse_durasi(&duration_str) {
        Err(()) => {
            return dengan_init(
                Reply::text("❌ Format durasi tidak valid\n\nContoh: 7d, 1m, 1y, lifetime"),
                init,
            );
        }
        Ok(Some(ms)) => ms,
        Ok(None) => 0,
    };
    let is_lifetime = expired_at == 0;

    let Some((group_id, group_name, invite_code)) = resolve_grup(&ctx, &input) else {
        return dengan_init(
            Reply::text("❌ Grup tidak ditemukan atau link tidak valid").with_reaction("❌"),
            init,
        );
    };

    let expired_str = if is_lifetime {
        "Permanent".to_string()
    } else {
        expired_str(expired_at)
    };

    let mut text = String::from("✅ *SEWA BERHASIL DITAMBAHKAN*\n\n");
    text.push_str(&format!("Grup: *{group_name}*\n"));
    text.push_str(&format!(
        "ID: {}\n",
        group_id.split('@').next().unwrap_or(&group_id)
    ));
    text.push_str(&format!("Durasi: *{}*\n", format_durasi(&duration_str)));
    text.push_str(&format!("Expired: *{expired_str}*\n\n"));

    let (joined, alasan) = try_join(&ctx, invite_code.as_ref());
    if joined {
        text.push_str(&format!("✅ {alasan}"));
    } else {
        text.push_str(&format!(
            "⚠️ Auto-join gagal: {alasan}\nTambahkan bot ke grup secara manual."
        ));
    }

    dengan_init(
        Reply::text(text).with_reaction("✅"),
        init,
    )
    .with_mutation(simpan_grup(
        &ctx,
        &group_id,
        &group_name,
        expired_at,
        is_lifetime,
    ))
}

/// `Map` dipakai supaya `serde_json::Map` ikut terpakai tanpa warnings.
#[allow(dead_code)]
type _MapAlias = Map<String, Value>;

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(args: &[&str], settings: Value) -> Reply {
        addsewa(Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "addsewa".into(),
            settings,
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn tabel(r: &Reply) -> Option<Value> {
        r.mutations().iter().rev().find_map(|m| match m {
            Mutation::SetSetting { key, value } if key == SEWA_KEY => Some(value.clone()),
            _ => None,
        })
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("addsewa").unwrap();
        assert_eq!(p.meta.name, "addsewa");
        assert_eq!(p.meta.alias, ["sewaadd", "tambahsewa"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Tambah grup ke whitelist sewa + auto join");
        assert_eq!(p.meta.usage, ".addsewa <link/id grup> <durasi>");
        assert_eq!(p.meta.example, ".addsewa https://chat.whatsapp.com/xxx 30d");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_admin);
        assert!(!p.meta.is_bot_admin);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["sewaadd", "tambahsewa"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("addsewa"), "{a}");
        }
    }

    #[test]
    fn tanpa_argumen_tampil_panduan() {
        let r = run(&[], Value::Null);
        let t = teks(&r);
        assert!(t.starts_with("📝 *TAMBAH SEWA*"), "{t}");
        assert!(t.contains("Format: *.addsewa <link/id> <durasi>*"), "{t}");
        assert!(t.contains("• 30i = 30 menit"), "{t}");
        assert!(t.contains("• lifetime = Permanent"), "{t}");
        assert!(t.contains("💡 Jika pakai link, bot akan otomatis join ke grup tersebut!"), "{t}");
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn durasi_tidak_valid_ditolak() {
        let r = run(&["120363@g.us", "xyz"], Value::Null);
        assert_eq!(
            teks(&r),
            "❌ Format durasi tidak valid\n\nContoh: 7d, 1m, 1y, lifetime"
        );
        assert!(tabel(&r).is_some(), "tabel tetap di-init");
    }

    #[test]
    fn format_durasi_sama_dengan_js() {
        assert_eq!(format_durasi("30i"), "30 menit");
        assert_eq!(format_durasi("12h"), "12 jam");
        assert_eq!(format_durasi("7d"), "7 hari");
        assert_eq!(format_durasi("1m"), "1 bulan");
        assert_eq!(format_durasi("1y"), "1 tahun");
        assert_eq!(format_durasi("lifetime"), "Permanent");
        assert_eq!(format_durasi("PERMANENT"), "Permanent");
        assert_eq!(format_durasi("abc"), "abc");
    }

    #[test]
    fn parse_durasi_lifetime_memberi_none() {
        assert_eq!(parse_durasi("lifetime"), Ok(None));
        assert_eq!(parse_durasi("forever"), Ok(None));
        assert!(parse_durasi("7d").unwrap().is_some());
        assert_eq!(parse_durasi("7x"), Err(()));
    }

    #[test]
    fn grup_tanpa_metadata_dipakai_dengan_nama_unknown() {
        // `sock.groupMetadata` gagal → nama "Unknown", invite code null.
        let r = run(&["120363", "30d"], Value::Null);
        let t = teks(&r);
        assert!(t.starts_with("✅ *SEWA BERHASIL DITAMBAHKAN*"), "{t}");
        assert!(t.contains("Grup: *Unknown*"), "{t}");
        assert!(t.contains("ID: 120363"), "{t}");
        assert!(t.contains("Durasi: *30 hari*"), "{t}");
        assert!(t.contains("Expired: *"), "{t}");
        assert!(
            t.contains(
                "⚠️ Auto-join gagal: Tidak ada invite code, tambahkan bot secara manual\nTambahkan bot ke grup secara manual."
            ),
            "{t}"
        );
        let tabel = tabel(&r).expect("harus menyimpan tabel");
        assert_eq!(tabel["groups"]["120363@g.us"]["name"], "Unknown");
        assert_eq!(tabel["groups"]["120363@g.us"]["isLifetime"], false);
    }

    #[test]
    fn lifetime_menyimpan_expired_zero() {
        let r = run(&["120363", "lifetime"], Value::Null);
        let t = teks(&r);
        assert!(t.contains("Durasi: *Permanent*"), "{t}");
        assert!(t.contains("Expired: *Permanent*"), "{t}");
        let tabel = tabel(&r).unwrap();
        assert_eq!(tabel["groups"]["120363@g.us"]["expiredAt"], 0);
        assert_eq!(tabel["groups"]["120363@g.us"]["isLifetime"], true);
    }

    #[test]
    fn grup_lama_tidak_hilang_saat_tambah_baru() {
        let awal = serde_json::json!({
            "sewa": { "enabled": true, "groups": { "a@g.us": { "name": "A" } } }
        });
        let r = run(&["120363", "30d"], awal);
        let tabel = tabel(&r).unwrap();
        assert_eq!(tabel["enabled"], true, "status enabled tidak hilang");
        assert_eq!(tabel["groups"]["a@g.us"]["name"], "A");
        assert!(tabel["groups"]["120363@g.us"].is_object());
    }
}
