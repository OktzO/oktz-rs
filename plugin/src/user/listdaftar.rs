//! `plugins/user/listdaftar.js` — daftar user terdaftar dengan filter dan
//! pagination (owner only).
//!
//! `db.getAllUsers()` tidak ada di `Ctx` (hanya `user_data` milik sender);
//! cara yang dipakai `main/leaderboard.rs` dipakai juga di sini: snapshot
//! store dibaca lewat `ctx.berkas()` di `database/main/settings.json`.
//! Kunci map `users` adalah nomor bersih, dan objek user JS tidak selalu
//! punya field `lid`/`jid` — `@${user.lid || user.jid}` di JS memakai field
//! itu apa adanya; port ini jatuh ke kunci map kalau keduanya tidak ada,
//! supaya mention tidak berubah jadi `@undefined`.
//!
//! `ponytail:` `contextInfo.forwardedNewsletterMessageInfo` (saluran) pada
//! pengiriman JS tidak ada di `Reply::Text` — mentions tetap dikirim.

use oktz_plugin_api::{Ctx, Reply, plugin};

use crate::store::done::date_ke_ms;

/// `PAGE_SIZE = 20` (`listdaftar.js:6`).
const PAGE_SIZE: usize = 20;

/// Kata kunci yang menghentikan penampungan token search
/// (`listdaftar.js:64-72`).
const STOPWORDS: [&str; 8] = ["page", "hal", "halaman", "search", "cari", "nama", "sort", "urut"];

/// Opsi hasil parse `parseListOptions` (`listdaftar.js:41-135`).
#[derive(Debug, PartialEq)]
pub(crate) struct Opsi {
    page: usize,
    search: String,
    sort_terbaru: bool,
}

impl Default for Opsi {
    fn default() -> Self {
        Self {
            page: 1,
            search: String::new(),
            sort_terbaru: false,
        }
    }
}

/// `parseListOptions(m.text)` — port 1:1 termasuk perilaku `i += 1` di dalam
/// loop search JS yang melompati token terakhir yang diserap.
pub(crate) fn parse_opsi(input: &str) -> Opsi {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut opsi = Opsi::default();
    let mut i = 0usize;
    while i < tokens.len() {
        let token = tokens[i].to_lowercase();

        if token == "page" || token == "hal" || token == "halaman" {
            if let Some(next) = tokens.get(i + 1).and_then(|t| t.parse::<usize>().ok()).filter(|&n| n > 0) {
                opsi.page = next;
                i += 1;
            }
            i += 1;
            continue;
        }

        if token == "search" || token == "cari" || token == "nama" {
            let mut search_tokens: Vec<&str> = Vec::new();
            let mut j = i + 1;
            while j < tokens.len() {
                let next_token = tokens[j].to_lowercase();
                if STOPWORDS.contains(&next_token.as_str()) {
                    break;
                }
                search_tokens.push(tokens[j]);
                i = j;
                j += 1;
            }
            if !search_tokens.is_empty() {
                opsi.search = search_tokens.join(" ").trim().to_string();
            }
            i += 1;
            continue;
        }

        if token == "sort" || token == "urut" {
            if let Some(next) = tokens.get(i + 1).map(|t| t.to_lowercase())
                && (next == "terbaru" || next == "newest") {
                    opsi.sort_terbaru = true;
                    i += 1;
                }
            i += 1;
            continue;
        }

        if token == "terbaru" || token == "newest" {
            opsi.sort_terbaru = true;
            i += 1;
            continue;
        }

        // `/^\d+$/` dan `page === 1` — hanya halaman pertama yang dioverride.
        if !token.is_empty() && token.chars().all(|c| c.is_ascii_digit()) && opsi.page == 1 {
            opsi.page = token.parse().unwrap_or(1);
        }
        i += 1;
    }
    opsi
}

/// `getRegistrationTime` — `lastRegisteredAt || registeredAt`, gagal parse = 0.
pub(crate) fn waktu_daftar(user: &serde_json::Value) -> i64 {
    // `user?.lastRegisteredAt || user?.registeredAt || null` — JS `||` jatuh
    // ke registeredAt juga saat lastRegisteredAt null/kosong, bukan hanya
    // saat field-nya absen.
    let last = user.get("lastRegisteredAt").unwrap_or(&serde_json::Value::Null);
    let nilai = if crate::common::js_truthy_value(last) {
        last
    } else {
        user.get("registeredAt").unwrap_or(&serde_json::Value::Null)
    };
    date_ke_ms(nilai)
}

/// `formatDateTime` — `dd/mm/yyyy, hh.mm`, atau `-` kalau kosong/tidak valid.
pub(crate) fn format_waktu(ms: i64) -> String {
    if ms == 0 {
        return "-".to_string();
    }
    let (y, mo, d, h, mi, _) = crate::common::jakarta_parts(ms);
    format!("{d:02}/{mo:02}/{y}, {h:02}.{mi:02}")
}

/// Emoji gender (`listdaftar.js:160-164`).
fn emoji_gender(user: &serde_json::Value) -> &'static str {
    match user.get("regGender").and_then(|v| v.as_str()) {
        Some("Laki-laki") => "👨",
        Some("Perempuan") => "👩",
        _ => "👤",
    }
}

/// Semua user dari snapshot store, apa adanya (`db.getAllUsers()`).
fn semua_user(ctx: &Ctx) -> Vec<(String, serde_json::Value)> {
    let Ok(teks) = ctx.berkas().baca_teks("database/main/settings.json") else {
        return Vec::new();
    };
    let Ok(root) = serde_json::from_str::<serde_json::Value>(&teks) else {
        return Vec::new();
    };
    root.get("users")
        .and_then(|u| u.as_object())
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}

/// Jid tampilan: `user.lid || user.jid`, jatuh ke kunci map.
fn tampil_jid(user: &serde_json::Value, kunci: &str) -> String {
    user.get("lid")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| user.get("jid").and_then(|v| v.as_str()).filter(|s| !s.is_empty()))
        .unwrap_or(kunci)
        .to_string()
}

/// Satu baris user (`listdaftar.js:166-171`).
fn baris_user(no: usize, user: &serde_json::Value, jid: &str) -> String {
    let nama = user
        .get("regName")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown");
    let umur = user
        .get("regAge")
        .map(|v| match v {
            serde_json::Value::String(s) if !s.is_empty() => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => "?".to_string(),
        })
        .unwrap_or_else(|| "?".to_string());
    let waktu = format_waktu(waktu_daftar(user));
    format!("{no}. {} *{nama}*\n   > @{jid} | {umur} tahun | {waktu}\n", emoji_gender(user))
}

#[plugin(
    name = "listdaftar",
    alias = ["listuser", "registeredusers", "daftarlist"],
    category = "user",
    description = "Lihat daftar user yang sudah terdaftar dengan filter dan pagination",
    usage = ".listdaftar [page <nomor>] [search <nama>] [sort terbaru]",
    example = ".listdaftar search zann sort terbaru page 2",
    cooldown = 10,
    energi = 0,
    is_owner = true,
    is_premium = false,
    is_group = false,
    is_private = false
)]
pub fn listdaftar(ctx: Ctx) -> Reply {
    let semua = semua_user(&ctx);
    let opsi = parse_opsi(&ctx.text);
    let mut terdaftar: Vec<(String, serde_json::Value)> = semua
        .into_iter()
        .filter(|(_, u)| crate::common::js_truthy_value(u.get("isRegistered").unwrap_or(&serde_json::Value::Null)))
        .collect();

    if terdaftar.is_empty() {
        return Reply::text("❌ Belum ada user yang terdaftar!");
    }

    if !opsi.search.is_empty() {
        let keyword = opsi.search.to_lowercase();
        terdaftar.retain(|(_, u)| {
            u.get("regName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase()
                .contains(&keyword)
        });
    }

    if opsi.sort_terbaru {
        terdaftar.sort_by(|a, b| waktu_daftar(&b.1).cmp(&waktu_daftar(&a.1)));
    }

    if terdaftar.is_empty() {
        return Reply::text(format!(
            "❌ Tidak ada user yang cocok dengan pencarian: *{}*",
            opsi.search
        ));
    }

    let total_halaman = terdaftar.len().div_ceil(PAGE_SIZE).max(1);
    let halaman = opsi.page.clamp(1, total_halaman);
    let mulai = (halaman - 1) * PAGE_SIZE;
    let tampil = &terdaftar[mulai..(mulai + PAGE_SIZE).min(terdaftar.len())];

    let mut text = String::from("📋 *ᴅᴀꜰᴛᴀʀ ᴜsᴇʀ ᴛᴇʀᴅᴀꜰᴛᴀʀ*\n\n");
    text.push_str(&format!("> Total hasil: *{}* user\n", terdaftar.len()));
    text.push_str(&format!("> Halaman: *{}/{}*\n", halaman, total_halaman));
    text.push_str(&format!(
        "> Urut: *{}*\n",
        if opsi.sort_terbaru { "Terbaru" } else { "Default" }
    ));
    if !opsi.search.is_empty() {
        text.push_str(&format!("> Search: *{}*\n", opsi.search));
    }
    text.push('\n');

    let mut mentions = Vec::new();
    for (i, (kunci, user)) in tampil.iter().enumerate() {
        let jid = tampil_jid(user, kunci);
        text.push_str(&baris_user(mulai + i + 1, user, &jid));
        mentions.push(if user.get("lid").and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty()) {
            format!("{jid}@lid")
        } else {
            format!("{jid}@s.whatsapp.net")
        });
    }

    if total_halaman > 1 {
        let berikut = if halaman + 1 > total_halaman { total_halaman } else { halaman + 1 };
        text.push_str(&format!(
            "\n> Gunakan `{p}listdaftar page {berikut}` untuk halaman lain",
            p = ctx.prefix
        ));
    }

    // Reply::Text membawa mentions langsung — `with_mentions` hanya untuk kartu.
    Reply::Text { text, mentions }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use serde_json::json;

    /// Store snapshot dengan 3 user terdaftar + 1 belum.
    const STORE: &str = r#"{"users":{
        "628111":{"isRegistered":true,"regName":"Andi","regAge":21,"regGender":"Laki-laki","registeredAt":"2026-01-01T00:00:00.000Z","lastRegisteredAt":"2026-01-05T00:00:00.000Z"},
        "628222":{"isRegistered":true,"regName":"Sari","regAge":19,"regGender":"Perempuan","registeredAt":"2026-02-01T00:00:00.000Z","lastRegisteredAt":"2026-02-01T00:00:00.000Z"},
        "628333":{"isRegistered":true,"regName":"Budi","regAge":30,"regGender":"Laki-laki","registeredAt":"2026-01-03T00:00:00.000Z"},
        "628444":{"isRegistered":false}
    }}"#;

    fn ctx_of(text: &str) -> Ctx {
        let berkas = BerkasPalsu::baru();
        berkas.tulis_teks("database/main/settings.json", STORE).unwrap();
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628999@s.whatsapp.net".into(),
            chat: "628999@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "listdaftar".into(),
            berkas: std::sync::Arc::new(berkas),
            ..Ctx::default()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("listdaftar").unwrap();
        assert_eq!(p.meta.name, "listdaftar");
        assert_eq!(p.meta.alias, ["listuser", "registeredusers", "daftarlist"]);
        assert_eq!(p.meta.category, "user");
        assert_eq!(
            p.meta.description,
            "Lihat daftar user yang sudah terdaftar dengan filter dan pagination"
        );
        assert_eq!(
            p.meta.usage,
            ".listdaftar [page <nomor>] [search <nama>] [sort terbaru]"
        );
        assert_eq!(p.meta.example, ".listdaftar search zann sort terbaru page 2");
        assert_eq!(p.meta.cooldown, 10);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn parse_opsi_mengikuti_js() {
        assert_eq!(parse_opsi(""), Opsi::default());
        assert_eq!(
            parse_opsi("page 2"),
            Opsi { page: 2, ..Default::default() }
        );
        assert_eq!(parse_opsi("hal 3"), Opsi { page: 3, ..Default::default() });
        assert_eq!(parse_opsi("2"), Opsi { page: 2, ..Default::default() });
        // `page === 1` saja yang dioverride angka telanjang.
        assert_eq!(
            parse_opsi("page 2 5"),
            Opsi { page: 2, ..Default::default() }
        );
        assert_eq!(
            parse_opsi("search zann sort terbaru page 2"),
            Opsi { page: 2, search: "zann".into(), sort_terbaru: true }
        );
        assert_eq!(
            parse_opsi("cari budi umur"),
            Opsi { search: "budi umur".into(), ..Default::default() }
        );
        assert_eq!(
            parse_opsi("sort newest"),
            Opsi { sort_terbaru: true, ..Default::default() }
        );
        assert_eq!(
            parse_opsi("urut terbaru"),
            Opsi { sort_terbaru: true, ..Default::default() }
        );
        // JS: `parseInt("0")` gagal `> 0` pada cabang `page <n>`, tetapi
        // token telanjang "0" tetap masuk lewat cabang `/^\d+$/` — keduanya
        // menghasilkan page 0 yang dijepit jadi 1 saat render (`Math.max`).
        let nol = Opsi { page: 0, ..Default::default() };
        assert_eq!(parse_opsi("page 0"), nol);
        assert_eq!(parse_opsi("0"), nol);
    }

    #[test]
    fn tanpa_riwayat_daftar_pesannya_tepat() {
        let berkas = BerkasPalsu::baru();
        let c = Ctx {
            berkas: std::sync::Arc::new(berkas),
            command: "listdaftar".into(),
            ..Ctx::default()
        };
        assert_eq!(listdaftar(c).text_body().unwrap(), "❌ Belum ada user yang terdaftar!");
    }

    #[test]
    fn daftar_menampilkan_semua_user_terdaftar() {
        let r = listdaftar(ctx_of(""));
        let t = r.text_body().unwrap().to_string();
        assert!(t.starts_with("📋 *ᴅᴀꜰᴛᴀʀ ᴜsᴇʀ ᴛᴇʀᴅᴀꜰᴛᴀʀ*"), "{t}");
        assert!(t.contains("> Total hasil: *3* user"));
        assert!(t.contains("> Halaman: *1/1*"));
        assert!(t.contains("> Urut: *Default*"));
        assert!(t.contains("1. 👨 *Andi*\n   > @628111 | 21 tahun | 05/01/2026, 07.00\n"));
        assert!(t.contains("2. 👩 *Sari*\n   > @628222 | 19 tahun | 01/02/2026, 07.00\n"));
        assert!(t.contains("3. 👨 *Budi*\n   > @628333 | 30 tahun | 03/01/2026, 07.00\n"));
        assert!(!t.contains("628444"), "belum terdaftar tidak ikut");
        match &r {
            Reply::Text { mentions, .. } => assert_eq!(
                mentions,
                &["628111@s.whatsapp.net".to_string(), "628222@s.whatsapp.net".to_string(), "628333@s.whatsapp.net".to_string()]
            ),
            other => panic!("harus Text, dapat {other:?}"),
        }
        let t2 = r.text_body().unwrap().to_string();
        assert!(!t2.contains("628444") || !t.contains("628444"));
    }

    #[test]
    fn search_dan_sort_terbaru() {
        let r = listdaftar(ctx_of("search sari"));
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("> Search: *sari*"));
        assert!(t.contains("*Sari*"));
        assert!(!t.contains("*Andi*"));
        match r {
            Reply::Text { mentions, .. } => assert_eq!(mentions.len(), 1),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn sort_terbaru_mengurutkan_last_registered() {
        let r = listdaftar(ctx_of("sort terbaru"));
        let t = r.text_body().unwrap().to_string();
        let pos_sari = t.find("*Sari*").unwrap();
        let pos_andi = t.find("*Andi*").unwrap();
        let pos_budi = t.find("*Budi*").unwrap();
        assert!(pos_sari < pos_andi && pos_andi < pos_budi, "{t}");
        assert!(t.contains("> Urut: *Terbaru*"));
    }

    #[test]
    fn pencarian_tanpa_hasil() {
        let r = listdaftar(ctx_of("search zzz"));
        assert_eq!(
            r.text_body().unwrap(),
            "❌ Tidak ada user yang cocok dengan pencarian: *zzz*"
        );
    }

    #[test]
    fn pagination_dua_puluh_per_halaman() {
        // 21 user terdaftar → 2 halaman.
        let mut users = String::from("{\"users\":{");
        for i in 0..21 {
            if i > 0 {
                users.push(',');
            }
            users.push_str(&format!(
                "\"6281{i:02}\":{{\"isRegistered\":true,\"regName\":\"User{i:02}\",\"registeredAt\":\"2026-01-01T00:00:00.000Z\"}}"
            ));
        }
        users.push_str("}}");
        let berkas = BerkasPalsu::baru();
        berkas.tulis_teks("database/main/settings.json", &users).unwrap();
        let c = Ctx {
            text: "page 2".into(),
            args: vec!["page".into(), "2".into()],
            prefix: ".".into(),
            command: "listdaftar".into(),
            berkas: std::sync::Arc::new(berkas),
            ..Ctx::default()
        };
        let r = listdaftar(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("> Halaman: *2/2*"));
        assert!(t.contains("21. 👤 *User20*"));
        assert!(t.contains("> Gunakan `.listdaftar page 2` untuk halaman lain"), "{t}");
        assert!(!t.contains("1. *User00*"), "halaman 1 tidak ikut");
    }

    #[test]
    fn waktu_daftar_mengikuti_js() {
        assert_eq!(
            waktu_daftar(&json!({"lastRegisteredAt": "2026-01-05T00:00:00.000Z", "registeredAt": "2026-01-01T00:00:00.000Z"})),
            1_767_571_200_000
        );
        assert_eq!(waktu_daftar(&json!({"registeredAt": "2026-01-01T00:00:00.000Z"})), 1_767_225_600_000);
        assert_eq!(waktu_daftar(&json!({})), 0);
        assert_eq!(format_waktu(0), "-");
        assert_eq!(format_waktu(1_767_225_600_000), "01/01/2026, 07.00");
    }
}
