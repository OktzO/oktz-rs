//! `plugins/stalker/stalkdc.js` — profil Discord via dclookup.id.
//!
//! Alur JS dipertahankan: ambil versi collector dari `api.js`
//! (`stalkdc.js:51-60`), minta token reCAPTCHA v3 (`anchor` lalu `reload`,
//! `stalkdc.js:72-133`), baru `POST /api/user/lookup`
//! (`stalkdc.js:135-158`). Status HTTP 4xx/5xx dianggap jawaban biasa
//! (`validateStatus: () => true`), jadi semua request memakai
//! `terima_semua_status()`.
//!
//! `ponytail:` `randomCb()` memakai `rand` (JS `Math.random`) dan versi
//! collector TIDAK di-cache antar panggilan seperti `_cachedV` — antrean
//! plugin Rust tidak punya state modul. Biaya satu request kecil per panggilan.
//!
//! `ponytail:` JS bereaksi `🕕` sebelum permintaan lalu `✅`/`❌` sesudahnya
//! (`stalkdc.js:179,185,215,218`), sedangkan `Reply::with_reaction` hanya
//! bisa membawa satu reaksi — jadi `🕕` hilang dan yang ikut ke balasan
//! hanya reaksi penutup. Reaksi `✅` juga dikirim sebelum media avatar
//! diunggah, jadi kegagalan upload tidak menambah balasan error.
//!
//! `ponytail:` JS mengirim avatar dengan `{ quoted: m }`
//! (`stalkdc.js:210`); `Reply::image` tidak bisa mengutip pesan pemicu,
//! jadi avatar dikirim tanpa quote.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

const BASE: &str = "https://www.dclookup.id";
const SITEKEY: &str = "6Lc0ZystAAAAAKESQ1HMsCQujyLgDvSu6FW1CHfP";
const ACTION: &str = "lookup";
const RECAPTCHA_BASE: &str = "https://www.google.com/recaptcha/";
const UA: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";
/// `sec-ch-ua` instance `gc` (`stalkdc.js:30`).
const SEC_CH_UA: &str = "\"Chromium\";v=\"127\", \"Not)A;Brand\";v=\"99\", \"Microsoft Edge Simulate\";v=\"127\", \"Lemur\";v=\"127\"";

/// Header bawaan instance `gc` axios (`stalkdc.js:25-35`) — dipakai untuk
/// ketiga permintaan reCAPTCHA (versi collector, anchor, reload).
fn dengan_header_gc(call: HttpCall) -> HttpCall {
    call.header("user-agent", UA)
        .header("accept-language", "id-ID")
        .header("sec-ch-ua", SEC_CH_UA)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Android\"")
}

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

/// `fetchLatestVersion()` — `releases/<v>/` di `api.js` (`stalkdc.js:51-60`).
fn versi_collector(ctx: &Ctx) -> Result<String, String> {
    let call = dengan_header_gc(HttpCall::get(format!("{RECAPTCHA_BASE}api.js")))
        .header("referer", "https://www.google.com/")
        .terima_semua_status();
    let r = ctx.http.call(&call).map_err(|e| e.message)?;
    let isi = r.text();
    let Some(mulai) = isi.find("releases/") else {
        return Err("gagal grab versi collector (v) dari api.js".to_string());
    };
    let sisa = &isi[mulai + "releases/".len()..];
    let Some(akhir) = sisa.find('/') else {
        return Err("gagal grab versi collector (v) dari api.js".to_string());
    };
    Ok(sisa[..akhir].to_string())
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

/// Satu percobaan reCAPTCHA: anchor → reload → token
/// (`stalkdc.js:88-124`).
fn percobaan_recaptcha(ctx: &Ctx, v: &str, anchor_qs: &str) -> Result<String, String> {
    let base = format!("{RECAPTCHA_BASE}api2/");
    let co = build_co(BASE);
    let anchor_url = format!("{base}anchor?{anchor_qs}");
    let call_anchor = dengan_header_gc(HttpCall::get(&anchor_url))
        .header("referer", format!("{}/", origin(BASE)))
        .header("sec-fetch-dest", "iframe")
        .terima_semua_status();
    let anchor = ctx.http.call(&call_anchor).map_err(|e| e.message)?;
    if anchor.status != 200 {
        return Err(format!("anchor HTTP {}", anchor.status));
    }
    let isi_anchor = anchor.text();
    let c = ambil_antara(&isi_anchor, "id=\"recaptcha-token\" value=\"", '"')
        .ok_or_else(|| "recaptcha-token tidak ditemukan di anchor".to_string())?;

    let body = pasangan(&[
        ("v", v),
        ("reason", "q"),
        ("c", c),
        ("k", SITEKEY),
        ("co", co.as_str()),
        ("sa", ACTION),
    ]);
    let call_reload = dengan_header_gc(
        HttpCall::post(format!("{base}reload?k={}", url_encode(SITEKEY)))
            .header("content-type", "application/x-www-form-urlencoded")
            .header("origin", "https://www.google.com")
            .header("referer", &anchor_url)
            .header("accept", "*/*")
            .header("sec-fetch-site", "same-origin")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-dest", "empty")
            .form(body)
            .terima_semua_status(),
    );
    let reload = ctx.http.call(&call_reload).map_err(|e| e.message)?;
    if reload.status != 200 {
        return Err(format!("reload HTTP {}", reload.status));
    }
    extract_rresp(&reload.text()).ok_or_else(|| "rresp/token tidak ditemukan di reload".to_string())
}

/// `solveRecaptchaV3(...)` — `(token, versi)`. `retries = 2` → tiga percobaan,
/// error terakhir diteruskan (`stalkdc.js:87-132`).
fn solve_recaptcha(ctx: &Ctx) -> Result<(String, String), String> {
    // `v` diambil SEKALI di luar loop retry (`stalkdc.js:75-85`), jadi
    // kegagalan grab versi collector tidak diulang.
    let v = versi_collector(ctx)?;
    let co = build_co(BASE);
    let mut cb = random_cb();
    let mut galat = String::new();

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
        match percobaan_recaptcha(ctx, &v, &anchor_qs) {
            Ok(token) => return Ok((token, v)),
            Err(pesan) => {
                galat = pesan;
                if percobaan < 2 {
                    cb = random_cb();
                }
            }
        }
    }
    Err(galat)
}

/// `lookupUser(discordId, token)` (`stalkdc.js:135-158`).
fn lookup_user(ctx: &Ctx, id: &str, token: &str) -> Result<Value, String> {
    let call = HttpCall::post(format!("{BASE}/api/user/lookup"))
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
        .terima_semua_status();
    let r = ctx.http.call(&call).map_err(|e| e.message)?;
    // `typeof res.data !== 'object'` → lempar (`stalkdc.js:154-156`).
    match r.json() {
        Some(v) if v.is_object() => Ok(v),
        _ => Err(format!("Response bukan JSON (HTTP {})", r.status)),
    }
}

/// `v || '-'` — nilai falsy JS (`null`, `false`, `0`, `""`, hilang) jadi `-`.
fn atau_minus(v: Option<&Value>) -> String {
    match v {
        Some(x) if crate::common::js_truthy(Some(x)) => teks_js(Some(x)),
        _ => "-".to_string(),
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

    let (token, _versi) = match solve_recaptcha(&ctx) {
        Ok(v) => v,
        Err(pesan) => return gagal(&pesan),
    };

    let resp = match lookup_user(&ctx, &discord_id, &token) {
        Ok(v) => v,
        Err(pesan) => return gagal(&pesan),
    };

    if !crate::common::js_truthy(resp.get("success")) {
        let pesan = resp
            .get("message")
            .filter(|m| crate::common::js_truthy(Some(m)))
            .map(|m| teks_js(Some(m)))
            .unwrap_or_else(|| "-".to_string());
        return Reply::text(format!(
            "❌ *AKUN TIDAK DITEMUKAN*\n\nSistem gagal menemukan akun dengan ID *{discord_id}*. Mungkin ID tersebut salah atau akun telah dihapus.\n\n_Pesan error: {pesan}_"
        ))
        .with_reaction("❌");
    }

    let d = resp.get("data").cloned().unwrap_or(Value::Null);
    let caption = caption_sukses(&d);
    // `d.avatar_url || d.default_avatar_url || d.avatar` (`stalkdc.js:208-209`)
    // — string kosong di field pertama LALU ke field berikutnya.
    let avatar = ["avatar_url", "default_avatar_url", "avatar"]
        .iter()
        .find_map(|k| d.get(*k).and_then(Value::as_str).filter(|s| !s.is_empty()))
        .map(str::to_string);

    match avatar {
        Some(url) => Reply::image(MediaSource::url(url), caption).with_reaction("✅"),
        None => Reply::text(caption).with_reaction("✅"),
    }
}

/// `catch` JS (`stalkdc.js:216-220`) — reaksi `❌` + pesan error.
fn gagal(pesan: &str) -> Reply {
    Reply::text(format!(
        "❌ *GAGAL MELACAK AKUN*\n\nMaaf, sistem sedang mengalami gangguan saat melakukan *bypass reCAPTCHA* atau memanggil API. Silakan coba beberapa saat lagi.\n\n_Error: {pesan}_"
    ))
    .with_reaction("❌")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::uji::{reaksi, teks};
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpMethod, HttpResponse};
    use std::sync::{Arc, Mutex};

    /// Stub HTTP yang MEREKAM setiap request — supaya kontrak request
    /// reCAPTCHA (metode, URL, header, body form) ikut diuji, bukan cuma
    /// jumlah request. Stub bawaan `HttpRangkaian` membuang header custom.
    #[derive(Debug, Default)]
    struct Stub {
        balasan: Mutex<Vec<(u16, String)>>,
        seen: Mutex<Vec<HttpCall>>,
    }

    impl Stub {
        fn urutan(balasan: Vec<(u16, String)>) -> Arc<Self> {
            Arc::new(Stub {
                balasan: Mutex::new(balasan),
                seen: Mutex::new(Vec::new()),
            })
        }

        fn offline() -> Arc<Self> {
            Self::urutan(Vec::new())
        }

        fn request(&self, n: usize) -> HttpCall {
            self.seen.lock().unwrap()[n].clone()
        }

        fn jumlah(&self) -> usize {
            self.seen.lock().unwrap().len()
        }
    }

    impl HttpGet for Stub {
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
            let mut balasan = self.balasan.lock().unwrap();
            if balasan.is_empty() {
                return Err(HttpError {
                    message: "offline".into(),
                });
            }
            let (status, body) = balasan.remove(0);
            Ok(HttpResponse {
                status,
                body: HttpBody::Bytes {
                    data: Arc::from(body.into_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }

        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }

        fn post_json(&self, url: &str, body: &Value) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::post(url).json(body))
        }

        fn post_form(&self, url: &str, form: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::post(url).form(form.to_string()))
        }

        fn post_multipart(
            &self,
            _url: &str,
            _parts: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("stalkdc tidak pakai multipart")
        }
    }

    fn header<'a>(call: &'a HttpCall, name: &str) -> Option<&'a str> {
        call.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    fn form_body(call: &HttpCall) -> String {
        match &call.body {
            Some(HttpBody::Form(f)) => f.clone(),
            _ => String::new(),
        }
    }

    fn ctx_of(http: Arc<Stub>, text: &str) -> Ctx {
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

    /// Balasan scripted: versi collector → anchor → reload → lookup.
    fn jalur_sukses(hasil: &str) -> Arc<Stub> {
        Stub::urutan(vec![
            (
                200,
                "var x = 'https://www.google.com/recaptcha/releases/abcDEF-1x/api.js';".to_string(),
            ),
            (
                200,
                "<input id=\"recaptcha-token\" value=\"TOKEN-C\" />".to_string(),
            ),
            (200, ")]}'\n[\"rresp\",\"TOKEN-R\"]".to_string()),
            (200, hasil.to_string()),
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
        let t = teks(&stalkdc(ctx_of(Stub::offline(), "")));
        assert!(t.starts_with("👾 *DISCORD STALKER* 👾"), "{t}");
        assert!(t.contains("`.stalkdc <ID Discord>`"));

        assert_eq!(
            teks(&stalkdc(ctx_of(Stub::offline(), "abcdef"))),
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
        assert_eq!(http.jumlah(), 4);
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
    fn request_recaptcha_dan_lookup_sama_dengan_js() {
        let http = jalur_sukses(r#"{"success":true,"data":{"username":"budi","id":"123"}}"#);
        let _ = stalkdc(ctx_of(http.clone(), "1280009524941426700"));
        assert_eq!(http.jumlah(), 4);

        // 1. versi collector (`stalkdc.js:53-55`).
        let api = http.request(0);
        assert_eq!(api.method, HttpMethod::Get);
        assert_eq!(api.url, "https://www.google.com/recaptcha/api.js");
        assert_eq!(header(&api, "user-agent"), Some(UA));
        assert_eq!(header(&api, "accept-language"), Some("id-ID"));
        assert_eq!(header(&api, "sec-ch-ua"), Some(SEC_CH_UA));
        assert_eq!(header(&api, "sec-ch-ua-mobile"), Some("?0"));
        assert_eq!(header(&api, "sec-ch-ua-platform"), Some("\"Android\""));
        assert_eq!(header(&api, "referer"), Some("https://www.google.com/"));

        // 2. anchor (`stalkdc.js:91-93`).
        let anchor = http.request(1);
        assert_eq!(anchor.method, HttpMethod::Get);
        assert!(
            anchor
                .url
                .starts_with("https://www.google.com/recaptcha/api2/anchor?"),
            "{}",
            anchor.url
        );
        assert_eq!(header(&anchor, "sec-fetch-dest"), Some("iframe"));
        assert_eq!(header(&anchor, "referer"), Some("https://www.dclookup.id/"));
        assert!(anchor.url.contains("k=6Lc0ZystAAAAAKESQ1HMsCQujyLgDvSu6FW1CHfP"));
        assert!(anchor.url.contains("co=aHR0cHM6Ly93d3cuZGNsb29rdXAuaWQ6NDQz"));
        assert!(anchor.url.contains("hl=id"));
        assert!(anchor.url.contains("size=invisible"));

        // 3. reload (`stalkdc.js:108-118`).
        let reload = http.request(2);
        assert_eq!(reload.method, HttpMethod::Post);
        assert!(reload.url.starts_with("https://www.google.com/recaptcha/api2/reload?k="));
        assert_eq!(
            header(&reload, "content-type"),
            Some("application/x-www-form-urlencoded")
        );
        assert_eq!(header(&reload, "origin"), Some("https://www.google.com"));
        assert_eq!(header(&reload, "accept"), Some("*/*"));
        assert_eq!(header(&reload, "sec-fetch-site"), Some("same-origin"));
        assert_eq!(header(&reload, "sec-fetch-mode"), Some("cors"));
        assert_eq!(header(&reload, "sec-fetch-dest"), Some("empty"));
        let body = form_body(&reload);
        assert!(body.contains("v=abcDEF-1x"), "{body}");
        assert!(body.contains("reason=q"), "{body}");
        assert!(body.contains("c=TOKEN-C"), "{body}");
        assert!(body.contains("k=6Lc0ZystAAAAAKESQ1HMsCQujyLgDvSu6FW1CHfP"), "{body}");
        assert!(body.contains("co=aHR0cHM6Ly93d3cuZGNsb29rdXAuaWQ6NDQz"), "{body}");
        assert!(body.contains("sa=lookup"), "{body}");

        // 4. lookup (`stalkdc.js:136-149`).
        let lookup = http.request(3);
        assert_eq!(lookup.method, HttpMethod::Post);
        assert_eq!(lookup.url, "https://www.dclookup.id/api/user/lookup");
        assert_eq!(header(&lookup, "x-recaptcha-token"), Some("TOKEN-R"));
        assert_eq!(header(&lookup, "user-agent"), Some(UA));
        assert_eq!(header(&lookup, "origin"), Some("https://www.dclookup.id"));
        assert_eq!(form_body(&lookup), "id=1280009524941426700");
    }

    #[test]
    fn anchor_http_500_diulang_dua_kali_lalu_pesan_js() {
        let http = Stub::urutan(vec![
            (
                200,
                "var x = 'https://www.google.com/recaptcha/releases/abcDEF-1x/api.js';".to_string(),
            ),
            (500, String::new()),
            (500, String::new()),
            (500, String::new()),
        ]);
        let r = stalkdc(ctx_of(http.clone(), "9"));
        assert_eq!(reaksi(&r), ["❌"]);
        assert!(
            teks(&r).contains("_Error: anchor HTTP 500_"),
            "{}",
            teks(&r)
        );
        assert_eq!(http.jumlah(), 4, "1 versi collector + 3 anchor");
    }

    #[test]
    fn respons_bukan_json_menghasilkan_pesan_js() {
        let http = Stub::urutan(vec![
            (
                200,
                "var x = 'https://www.google.com/recaptcha/releases/abcDEF-1x/api.js';".to_string(),
            ),
            (
                200,
                "<input id=\"recaptcha-token\" value=\"TOKEN-C\" />".to_string(),
            ),
            (200, ")]}'\n[\"rresp\",\"TOKEN-R\"]".to_string()),
            (500, "bukan json".to_string()),
        ]);
        let r = stalkdc(ctx_of(http, "9"));
        assert_eq!(reaksi(&r), ["❌"]);
        assert!(
            teks(&r).contains("_Error: Response bukan JSON (HTTP 500)_"),
            "{}",
            teks(&r)
        );
    }

    #[test]
    fn recaptcha_gagal_bereaksi_silang() {
        let r = stalkdc(ctx_of(Stub::offline(), "9"));
        assert_eq!(reaksi(&r), ["❌"]);
        let t = teks(&r);
        assert!(t.contains("*GAGAL MELACAK AKUN*"), "{t}");
        assert!(t.contains("_Error: offline_"), "{t}");
    }

    #[test]
    fn avatar_kosong_lanjut_ke_default_avatar() {
        let http = jalur_sukses(
            r#"{"success":true,"data":{"username":"ani","id":"9","avatar_url":"",
                "default_avatar_url":"https://cdn.example/d.png"}}"#,
        );
        let r = stalkdc(ctx_of(http, "9"));
        let Reply::Image { media, .. } = r.without_mutations().tanpa_reaksi() else {
            panic!("harus Image: {}", teks(&r));
        };
        assert_eq!(media, &Some(MediaSource::url("https://cdn.example/d.png")));
    }

    #[test]
    fn nilai_falsy_jadi_dash() {
        let http = jalur_sukes_falsy();
        let r = stalkdc(ctx_of(http, "9"));
        let t = teks(&r);
        assert!(t.contains("🏷️ *Global Name:* -"), "{t}");
        assert!(t.contains("🗓️ *Dibuat Sejak:* -"), "{t}");
    }

    fn jalur_sukes_falsy() -> Arc<Stub> {
        jalur_sukses(
            r#"{"success":true,"data":{"username":"ani","id":"9","global_name":false,
                "creation_date":0}}"#,
        )
    }
}
