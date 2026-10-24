//! `plugins/stalker/stalkdc.js` — profil Discord via dclookup.id.
//!
//! Alur JS dipertahankan: ambil versi collector dari `api.js`, minta token
//! reCAPTCHA v3 (`anchor` lalu `reload`), baru `POST /api/user/lookup`.
//! Status HTTP 4xx/5xx dianggap jawaban biasa (`validateStatus: () => true`),
//! jadi semua request memakai `terima_semua_status()`.
//!
//! `ponytail:` `randomCb()` memakai `rand` (JS `Math.random`) dan versi
//! collector TIDAK di-cache antar panggilan seperti `_cachedV` — antrean
//! plugin Rust tidak punya state modul. Biaya satu request kecil per panggilan.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

const BASE: &str = "https://www.dclookup.id";
const SITEKEY: &str = "6Lc0ZystAAAAAKESQ1HMsCQujyLgDvSu6FW1CHfP";
const ACTION: &str = "lookup";
const RECAPTCHA_BASE: &str = "https://www.google.com/recaptcha/";
const UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

/// `String(x)` versi JS untuk field profil.
fn teks_js(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(lain) => lain.to_string(),
    }
}

fn ambil_antara<'a>(s: &'a str, awal: &str, akhir: char) -> Option<&'a str> {
    let mulai = s.find(awal)? + awal.len();
    let sisa = &s[mulai..];
    let akhir_idx = sisa.find(akhir)?;
    Some(&sisa[..akhir_idx])
}

/// Base64 standar tanpa padding (`toString('base64').replace(/=/g, '')`).
fn base64_tanpa_padding(data: &[u8]) -> String {
    const ABJAD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for potong in data.chunks(3) {
        let b = [
            potong[0],
            potong.get(1).copied().unwrap_or(0),
            potong.get(2).copied().unwrap_or(0),
        ];
        out.push(ABJAD[(b[0] >> 2) as usize] as char);
        out.push(ABJAD[(((b[0] & 0b11) << 4) | (b[1] >> 4)) as usize] as char);
        if potong.len() > 1 {
            out.push(ABJAD[(((b[1] & 0b1111) << 2) | (b[2] >> 6)) as usize] as char);
        }
        if potong.len() > 2 {
            out.push(ABJAD[(b[2] & 0b111111) as usize] as char);
        }
    }
    out
}

/// `buildCo(url)` — base64 dari `origin:port`.
fn build_co(url: &str) -> String {
    let (skema, sisa) = url.split_once("://").unwrap_or(("https", url));
    let host = sisa.split('/').next().unwrap_or(sisa);
    let port = if skema == "https" { 443 } else { 80 };
    base64_tanpa_padding(format!("{skema}://{host}:{port}").as_bytes())
}

fn origin(url: &str) -> String {
    let (skema, sisa) = url.split_once("://").unwrap_or(("https", url));
    let host = sisa.split('/').next().unwrap_or(sisa);
    format!("{skema}://{host}")
}

/// `randomCb()` — 12 karakter `[a-z0-9]`.
fn random_cb() -> String {
    use rand::Rng;
    const ABJAD: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..12)
        .map(|_| ABJAD[rng.random_range(0..ABJAD.len())] as char)
        .collect()
}

/// Percent-encode `URLSearchParams` (unreserved dibiarkan).
fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn pasangan(daftar: &[(&str, &str)]) -> String {
    daftar
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// `fetchLatestVersion()` — `releases/<v>/` di `api.js`.
fn versi_collector(ctx: &Ctx) -> Option<String> {
    let r = ctx
        .http
        .call(
            &HttpCall::get(format!("{RECAPTCHA_BASE}api.js"))
                .header("referer", "https://www.google.com/")
                .terima_semua_status(),
        )
        .ok()?;
    let isi = r.text();
    let mulai = isi.find("releases/")? + "releases/".len();
    let sisa = &isi[mulai..];
    let akhir = sisa.find('/')?;
    Some(sisa[..akhir].to_string())
}

/// `extractRresp(body)`.
fn extract_rresp(body: &str) -> Option<String> {
    let bersih = body.trim_start_matches(")]}'").trim_start();
    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(bersih) {
        if arr.first().and_then(Value::as_str) == Some("rresp") {
            if let Some(t) = arr.get(1).and_then(Value::as_str) {
                return Some(t.to_string());
            }
        }
    }
    ambil_antara(bersih, "\"rresp\",\"", '"').map(str::to_string)
}

/// `solveRecaptchaV3(...)` — `(token, versi)`.
fn solve_recaptcha(ctx: &Ctx) -> Option<(String, String)> {
    let v = versi_collector(ctx)?;
    let base = format!("{RECAPTCHA_BASE}api2/");
    let co = build_co(BASE);
    let mut cb = random_cb();

    // `retries = 2` → tiga percobaan.
    for percobaan in 0..3 {
        let anchor_qs = pasangan(&[
            ("ar", "1"),
            ("k", SITEKEY),
            ("co", co.as_str()),
            ("hl", "id"),
            ("v", v.as_str()),
            ("size", "invisible"),
            ("anchor-ms", "20000"),
            ("execute-ms", "30000"),
            ("cb", cb.as_str()),
        ]);
        let anchor_url = format!("{base}anchor?{anchor_qs}");
        let anchor = match ctx.http.call(
            &HttpCall::get(&anchor_url)
                .header("referer", format!("{}/", origin(BASE)))
                .header("sec-fetch-dest", "iframe")
                .terima_semua_status(),
        ) {
            Ok(r) => r,
            Err(_) => {
                if percobaan < 2 {
                    cb = random_cb();
                    continue;
                }
                return None;
            }
        };
        if anchor.status != 200 {
            if percobaan < 2 {
                cb = random_cb();
                continue;
            }
            return None;
        }
        let c = match ambil_antara(&anchor.text(), "id=\"recaptcha-token\" value=\"", '"') {
            Some(c) => c.to_string(),
            None => {
                if percobaan < 2 {
                    cb = random_cb();
                    continue;
                }
                return None;
            }
        };

        let body = pasangan(&[("v", v.as_str()), ("reason", "q"), ("c", c.as_str()), ("k", SITEKEY), ("co", co.as_str()), ("sa", ACTION)]);
        let reload = match ctx.http.call(
            &HttpCall::post(format!("{base}reload?k={}", url_encode(SITEKEY)))
                .header(
                    "content-type",
                    "application/x-www-form-urlencoded",
                )
                .header("origin", "https://www.google.com")
                .header("referer", &anchor_url)
                .header("accept", "*/*")
                .header("sec-fetch-site", "same-origin")
                .header("sec-fetch-mode", "cors")
                .header("sec-fetch-dest", "empty")
                .form(body)
                .terima_semua_status(),
        ) {
            Ok(r) => r,
            Err(_) => {
                if percobaan < 2 {
                    cb = random_cb();
                    continue;
                }
                return None;
            }
        };
        if reload.status != 200 {
            if percobaan < 2 {
                cb = random_cb();
                continue;
            }
            return None;
        }
        match extract_rresp(&reload.text()) {
            Some(token) => return Some((token, v)),
            None => {
                if percobaan < 2 {
                    cb = random_cb();
                    continue;
                }
                return None;
            }
        }
    }
    None
}

/// `lookupUser(discordId, token)`.
fn lookup_user(ctx: &Ctx, id: &str, token: &str) -> Option<Value> {
    let r = ctx
        .http
        .call(
            &HttpCall::post(format!("{BASE}/api/user/lookup"))
                .header(
                    "content-type",
                    "application/x-www-form-urlencoded; charset=UTF-8",
                )
                .header("accept", "application/json, text/javascript, */*; q=0.01")
                .header("x-requested-with", "XMLHttpRequest")
                .header("x-recaptcha-token", token)
                .header("user-agent", UA)
                .header("origin", BASE)
                .header("referer", format!("{BASE}/"))
                .form(pasangan(&[("id", id)]))
                .terima_semua_status(),
        )
        .ok()?;
    r.json()
}

fn atau_minus(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "-".to_string(),
        Some(Value::String(s)) if s.is_empty() => "-".to_string(),
        lain => teks_js(lain),
    }
}

fn caption_sukses(d: &Value) -> String {
    let is_bot = if crate::common::js_truthy(d.get("is_bot")) {
        "Ya (Bot)"
    } else {
        "Tidak (User)"
    };
    let badges = d
        .get("badges")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .map(|b| teks_js(b.get("name")))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string());

    let mut caption = String::from(
        "👾 *DISCORD STALKER* 👾\n\n\
         Pencarian berhasil! Berikut adalah informasi profil dari Discord tersebut:\n\n",
    );
    caption.push_str(&format!(
        "👤 *Username:* {}\n\
         🏷️ *Global Name:* {}\n\
         🆔 *User ID:* {}\n\
         🤖 *Apakah Bot?:* {is_bot}\n\
         🗓️ *Dibuat Sejak:* {}\n\
         🏅 *Badges:* {badges}\n",
        teks_js(d.get("username")),
        atau_minus(d.get("global_name")),
        teks_js(d.get("id")),
        atau_minus(d.get("creation_date")),
    ));
    if let Some(clan) = d.get("clan").filter(|c| crate::common::js_truthy(Some(c))) {
        caption.push_str(&format!(
            "🛡️ *Clan:* {} ({})\n",
            teks_js(clan.get("tag")),
            teks_js(clan.get("name"))
        ));
    }
    caption.push_str("\n_Powered by dclookup.id_");
    caption
}

#[plugin(
    name = "stalkdc",
    alias = ["discordstalk", "cekkdc", "stalkdiscord"],
    category = "stalker",
    description = "Mengetahui info profil akun Discord berdasarkan ID",
    usage = ".stalkdc <id_discord>",
    example = ".stalkdc 1280009524941426700",
    cooldown = 10,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn stalkdc(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    if text.is_empty() {
        return Reply::text(format!(
            "👾 *DISCORD STALKER* 👾\n\n\
             Fitur ini membantumu untuk melacak profil pengguna Discord hanya dari ID-nya saja!\n\n\
             *CARA PENGGUNAAN:*\n\
             - Ketik `{}stalkdc <ID Discord>`\n\
             - Contoh: `{}stalkdc 1280009524941426700`\n\n\
             _Pastikan kamu memasukkan ID berbentuk angka (bukan username ya!)._",
            ctx.prefix, ctx.prefix
        ));
    }

    let discord_id: String = text.chars().filter(char::is_ascii_digit).collect();
    if discord_id.is_empty() {
        return Reply::text(
            "❌ *ID TIDAK VALID*\n\nPastikan ID Discord yang kamu masukkan hanya berupa angka.",
        );
    }

    let Some((token, _versi)) = solve_recaptcha(&ctx) else {
        return Reply::text(
            "❌ *GAGAL MELACAK AKUN*\n\nMaaf, sistem sedang mengalami gangguan saat melakukan *bypass reCAPTCHA* atau memanggil API. Silakan coba beberapa saat lagi.\n\n_Error: recaptcha gagal_",
        )
        .with_reaction("❌");
    };

    let Some(resp) = lookup_user(&ctx, &discord_id, &token) else {
        return Reply::text(
            "❌ *GAGAL MELACAK AKUN*\n\nMaaf, sistem sedang mengalami gangguan saat melakukan *bypass reCAPTCHA* atau memanggil API. Silakan coba beberapa saat lagi.\n\n_Error: respons bukan JSON_",
        )
        .with_reaction("❌");
    };

    if !crate::common::js_truthy(resp.get("success")) {
        let pesan = resp.get("message").and_then(Value::as_str).unwrap_or("-");
        return Reply::text(format!(
            "❌ *AKUN TIDAK DITEMUKAN*\n\nSistem gagal menemukan akun dengan ID *{discord_id}*. Mungkin ID tersebut salah atau akun telah dihapus.\n\n_Pesan error: {pesan}_"
        ))
        .with_reaction("❌");
    }

    let d = resp.get("data").cloned().unwrap_or(Value::Null);
    let caption = caption_sukses(&d);
    let avatar = ["avatar_url", "default_avatar_url", "avatar"]
        .iter()
        .find_map(|k| d.get(*k).and_then(Value::as_str))
        .filter(|s| !s.is_empty());

    match avatar {
        Some(url) => Reply::image(MediaSource::url(url.to_string()), caption).with_reaction("✅"),
        None => Reply::text(caption).with_reaction("✅"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::uji::{HttpRangkaian, reaksi, teks};
    use std::sync::Arc;

    fn ctx_of(http: Arc<HttpRangkaian>, text: &str) -> Ctx {
        Ctx {
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "stalkdc".into(),
            text: text.into(),
            http,
            ..Ctx::default()
        }
    }

    fn jalur_sukses(hasil: &str) -> Arc<HttpRangkaian> {
        HttpRangkaian::urutan(vec![
            "var x = 'https://www.google.com/recaptcha/releases/abcDEF-1x/api.js';".to_string(),
            "<input id=\"recaptcha-token\" value=\"TOKEN-C\" />".to_string(),
            ")]}'\n[\"rresp\",\"TOKEN-R\"]".to_string(),
            hasil.to_string(),
        ])
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("stalkdc").unwrap();
        assert_eq!(p.meta.alias, ["discordstalk", "cekkdc", "stalkdiscord"]);
        assert_eq!(
            p.meta.description,
            "Mengetahui info profil akun Discord berdasarkan ID"
        );
        assert_eq!(p.meta.usage, ".stalkdc <id_discord>");
        assert_eq!(p.meta.example, ".stalkdc 1280009524941426700");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 2);
    }

    #[test]
    fn base64_dan_build_co_sama_dengan_js() {
        assert_eq!(base64_tanpa_padding(b"https://www.dclookup.id:443"), "aHR0cHM6Ly93d3cuZGNsb29rdXAuaWQ6NDQz");
        assert_eq!(base64_tanpa_padding(b"Ma"), "TWE");
        assert_eq!(build_co("https://www.dclookup.id"), "aHR0cHM6Ly93d3cuZGNsb29rdXAuaWQ6NDQz");
    }

    #[test]
    fn tanpa_id_dan_id_bukan_angka() {
        let t = teks(&stalkdc(ctx_of(HttpRangkaian::offline(), "")));
        assert!(t.starts_with("👾 *DISCORD STALKER* 👾"), "{t}");
        assert!(t.contains("`.stalkdc <ID Discord>`"));

        assert_eq!(
            teks(&stalkdc(ctx_of(HttpRangkaian::offline(), "abcdef"))),
            "❌ *ID TIDAK VALID*\n\nPastikan ID Discord yang kamu masukkan hanya berupa angka."
        );
    }

    #[test]
    fn sukses_mengirim_avatar_dengan_caption_lengkap() {
        let http = jalur_sukses(
            r#"{"success":true,"data":{"username":"budi","global_name":null,"id":"123",
                "is_bot":false,"creation_date":"1 Januari 2020","badges":[{"name":"Staff"}],
                "clan":{"tag":"OKT","name":"Oktz"},
                "avatar_url":"https://cdn.example/a.png"}}"#,
        );
        let r = stalkdc(ctx_of(http.clone(), "1280009524941426700"));
        assert_eq!(reaksi(&r), ["✅"]);
        // `with_reaction` membungkus balasan: `Mutate { Reaction { Image } }`.
        // `without_mutations` turunkan `Mutate`, jadi `Reaction` masih ada —
        // perlu `tanpa_reaksi` lagi.
        let Reply::Image { media, caption, .. } = r.without_mutations().tanpa_reaksi() else {
            panic!("harus Image: {}", teks(&r));
        };
        assert_eq!(
            media,
            &Some(MediaSource::url("https://cdn.example/a.png"))
        );
        assert!(caption.contains("👤 *Username:* budi"), "{caption}");
        assert!(caption.contains("🏷️ *Global Name:* -"));
        assert!(caption.contains("🆔 *User ID:* 123"));
        assert!(caption.contains("🤖 *Apakah Bot?:* Tidak (User)"));
        assert!(caption.contains("🗓️ *Dibuat Sejak:* 1 Januari 2020"));
        assert!(caption.contains("🏅 *Badges:* Staff"));
        assert!(caption.contains("🛡️ *Clan:* OKT (Oktz)"));
        assert!(caption.ends_with("_Powered by dclookup.id_"));
        assert_eq!(http.terlihat().len(), 4);
    }

    #[test]
    fn tanpa_avatar_dan_akun_tidak_ditemukan() {
        let http = jalur_sukses(r#"{"success":true,"data":{"username":"ani","id":"9"}}"#);
        let r = stalkdc(ctx_of(http, "9"));
        assert_eq!(reaksi(&r), ["✅"]);
        let t = teks(&r);
        assert!(t.contains("🏷️ *Global Name:* -"), "{t}");
        assert!(t.contains("🏅 *Badges:* -"));
        assert!(!t.contains("🛡️ *Clan:*"));

        let http = jalur_sukses(r#"{"success":false,"message":"tidak ada"}"#);
        let r = stalkdc(ctx_of(http, "9"));
        assert_eq!(reaksi(&r), ["❌"]);
        assert_eq!(
            teks(&r),
            "❌ *AKUN TIDAK DITEMUKAN*\n\nSistem gagal menemukan akun dengan ID *9*. Mungkin ID tersebut salah atau akun telah dihapus.\n\n_Pesan error: tidak ada_"
        );
    }

    #[test]
    fn recaptcha_gagal_bereaksi_silang() {
        let r = stalkdc(ctx_of(HttpRangkaian::offline(), "9"));
        assert_eq!(reaksi(&r), ["❌"]);
        assert!(teks(&r).contains("_Error: recaptcha gagal_"));
    }
}
