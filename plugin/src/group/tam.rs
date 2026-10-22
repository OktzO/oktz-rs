//! `plugins/group/tam.js` — top member paling aktif di grup.
//!
//! Yang dipindah: seluruh perhitungan peringkat dari `group.chatStats`
//! (`tam.js:31-38`) dan isi snapshot poll (`tam.js:41-49`).
//!
//! Penyimpangan: JS mengirim `pollResultSnapshotMessage` lewat
//! `sock.relayMessage` (`tam.js:59-60`) — pesan poll native. Varian `Reply` di
//! `oktz-plugin-api` tidak punya poll, jadi isi poll yang sama dikirim sebagai
//! teks biasa (baris nama + jumlah suara). Nama baris option poll
//! (`1. Nama 🏆`) dan judul snapshot (`top N member active all time!`) tetap
//! persis seperti JS. Pesan juga tidak mengutip pemicu, jadi dipakai
//! `Reply::text_quoted` (tanpa quote), sama seperti `sock.relayMessage`.
//!
//! Penyimpangan kedua: `db.getUser(u.jid)` (`tam.js:43`) membaca store user
//! milik PESERTA lain. `Ctx` hanya membawa store milik pengirim
//! (`ctx.user_data`), jadi nama peserta yang tidak punya `chatStats.name`
//! jatuh ke nomor JID — sama dengan cabang fallback JS
//! (`u.jid.split("@")[0]`).

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

/// `tam.js:29` — batas atas jumlahKANAN yang boleh diminta.
const MAKS: i64 = 20;

/// `tam.js:41-49` — judul snapshot poll.
fn judul(limit: usize, nama_grup: &str) -> String {
    format!("top {limit} member active all time!\nat {nama_grup}")
}

/// `tam.js:42-47` — option poll: `${rank}. ${name}` + `🏆` untuk peringkat 1.
fn option(rank: usize, name: &str, juara: bool) -> String {
    if juara {
        format!("{rank}. {name} 🏆")
    } else {
        format!("{rank}. {name}")
    }
}

/// `Math.min(Math.max(parseInt(m.text) || 10, 1), 20)` (`tam.js:28`).
///
/// `parseInt` yang gagal (atau bernilai 0 — `0` falsy di JS) kembali ke 10.
pub(crate) fn limit_dari(text: &str) -> usize {
    let parsed = crate::common::parse_int(text).unwrap_or(0);
    let nilai = if parsed == 0 { 10 } else { parsed };
    nilai.max(1).min(MAKS) as usize
}

/// `db.getGroup(m.chat) || {}` → `group.name || "Grup"` (`tam.js:30`).
fn nama_grup(ctx: &Ctx) -> String {
    ctx.group_field("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("Grup")
        .to_string()
}

/// `Object.entries(group.chatStats)` → daftar `(jid, count, name)` (`tam.js:33-35`).
///
/// `d.count || 0` dan `d.name || null` mengikuti aturan falsy JS. Urutan
/// `Object.entries` = urutan sisip, dan `serde_json` di repo ini memakai fitur
/// `preserve_order`, jadi urutannya sama.
fn statistik(ctx: &Ctx) -> Vec<(String, i64, Option<String>)> {
    let Some(map) = ctx.group_field("chatStats").and_then(Value::as_object) else {
        return Vec::new();
    };
    map.iter()
        .map(|(jid, d)| {
            let count = d.get("count").and_then(Value::as_i64).unwrap_or(0);
            let name = d
                .get("name")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            (jid.clone(), count, name)
        })
        .collect()
}

/// `.sort((a, b) => b.count - a.count).slice(0, limit)` (`tam.js:36-37`).
///
/// `Array.prototype.sort` di JS stabil, jadi urutan asli dipertahankan untuk
/// jumlah suara yang sama — `sort_by` Rust juga stabil.
fn urutkan(mut data: Vec<(String, i64, Option<String>)>, limit: usize) -> Vec<(String, i64, Option<String>)> {
    data.sort_by(|a, b| b.1.cmp(&a.1));
    data.truncate(limit);
    data
}

#[plugin(
    name = "tam",
    alias = ["topactive", "topmember"],
    category = "group",
    description = "Lihat top member paling aktif di grup",
    usage = ".tam <jumlah>",
    example = ".tam 10",
    cooldown = 10,
    energi = 0,
    is_group = true,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn tam(ctx: Ctx) -> Reply {
    let limit = limit_dari(&ctx.text);
    let chat_name = nama_grup(&ctx);
    let sorted = urutkan(statistik(&ctx), limit);

    if sorted.is_empty() {
        // `tam.js:39-43` — `m.reply` (mengutip pemicu).
        return Reply::text(
            "📊 *ᴛᴏᴘ ᴀᴄᴛɪᴠᴇ ᴍᴇᴍʙᴇʀ*\n\n- Belum ada data aktivitas di grup ini",
        );
    }

    let mut text = judul(limit, &chat_name);
    for (i, (jid, count, name)) in sorted.iter().enumerate() {
        // `tam.js:43` — `u.name || user?.name || u.jid.split("@")[0]`.
        let display = match name {
            Some(n) => n.clone(),
            None => jid.split('@').next().unwrap_or(jid).to_string(),
        };
        text.push('\n');
        text.push_str(&option(i + 1, &display, i == 0));
        text.push_str(&format!(" — {count}"));
    }

    // `sock.relayMessage` = pesan baru, bukan reply → tanpa quote.
    Reply::text_quoted(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    fn ctx_dengan(stats: Option<Value>, nama: Option<&str>, text: &str) -> Ctx {
        let mut group = serde_json::Map::new();
        if let Some(s) = stats {
            group.insert("chatStats".into(), s);
        }
        if let Some(n) = nama {
            group.insert("name".into(), Value::from(n));
        }
        Ctx {
            prefix: ".".into(),
            command: "tam".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            is_group: true,
            text: text.into(),
            args: text.split_whitespace().map(String::from).collect(),
            group_data: Value::Object(group),
            ..Default::default()
        }
    }

    fn stats_dari(pasangan: &[(&str, i64, Option<&str>)]) -> Value {
        let mut map = serde_json::Map::new();
        for (jid, count, name) in pasangan {
            let mut d = serde_json::Map::new();
            d.insert("count".into(), Value::from(*count));
            if let Some(n) = name {
                d.insert("name".into(), Value::from(*n));
            }
            map.insert((*jid).to_string(), Value::Object(d));
        }
        Value::Object(map)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = find("tam").unwrap();
        assert_eq!(p.meta.name, "tam");
        assert_eq!(p.meta.alias, ["topactive", "topmember"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Lihat top member paling aktif di grup");
        assert_eq!(p.meta.usage, ".tam <jumlah>");
        assert_eq!(p.meta.example, ".tam 10");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group);
        assert!(!p.meta.is_private && !p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["topactive", "topmember"] {
            assert_eq!(find(a).map(|p| p.meta.name.as_str()), Some("tam"), "alias {a}");
        }
    }

    #[test]
    fn output_sama_dengan_js_saat_tidak_ada_statistik() {
        let r = tam(ctx_dengan(None, None, ""));
        assert_eq!(
            teks(&r),
            "📊 *ᴛᴏᴘ ᴀᴄᴛɪᴠᴇ ᴍᴇᴍʙᴇʀ*\n\n- Belum ada data aktivitas di grup ini"
        );
    }

    #[test]
    fn output_sama_dengan_js_dengan_daftar_member() {
        let stats = stats_dari(&[
            ("628111@s.whatsapp.net", 5, Some("Budi")),
            ("628222@s.whatsapp.net", 9, None),
        ]);
        let r = tam(ctx_dengan(Some(stats), Some("Grup Aktif"), "10"));
        let t = teks(&r);
        assert!(t.starts_with("top 10 member active all time!\nat Grup Aktif"), "{t}");
        // Ranking ditentukan jumlah suara, bukan urutan sisip.
        assert!(t.contains("\n1. 628222 🏆 — 9"), "{t}");
        assert!(t.contains("\n2. Budi — 5"), "{t}");
    }

    #[test]
    fn peringkat_satu_mendapat_juara() {
        let stats = stats_dari(&[("628111@s.whatsapp.net", 7, Some("Budi"))]);
        let r = tam(ctx_dengan(Some(stats), None, ""));
        let t = teks(&r);
        assert!(t.contains("1. Budi 🏆"), "{t}");
        assert!(t.contains("at Grup"), "{t}");
    }

    #[test]
    fn limit_dipotong_ke_1_sampai_20() {
        assert_eq!(limit_dari(""), 10, "`parseInt('')` = NaN → 10");
        assert_eq!(limit_dari("abc"), 10);
        assert_eq!(limit_dari("0"), 10, "`0` falsy → 10");
        assert_eq!(limit_dari("5"), 5);
        assert_eq!(limit_dari("5 member"), 5, "parseInt berhenti di non-digit");
        assert_eq!(limit_dari("-3"), 1, "Math.max(-3, 1)");
        assert_eq!(limit_dari("99"), 20, "Math.min(99, 20)");
    }

    #[test]
    fn hanya_sampai_20_member_yang_muncul() {
        let pasang: Vec<(String, i64, Option<&str>)> = (0..30)
            .map(|i| (format!("628{i:03}@s.whatsapp.net"), i64::from(30 - i), None))
            .collect();
        let borrowed: Vec<(&str, i64, Option<&str>)> = pasang
            .iter()
            .map(|(j, c, n)| (j.as_str(), *c, *n))
            .collect();
        let stats = stats_dari(&borrowed);
        let r = tam(ctx_dengan(Some(stats), None, "50"));
        let t = teks(&r);
        assert!(t.contains("top 20 member active all time!"), "{t}");
        assert!(!t.contains("628029"), "peserta ke-21 tidak boleh muncul: {t}");
    }
}
