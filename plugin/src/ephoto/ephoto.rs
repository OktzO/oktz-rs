//! `plugins/ephoto/ephoto.js` — efek teks dari en.ephoto360.com.
//!
//! `pluginConfig.name` di JS adalah array 32 efek; `#[plugin]` hanya menerima
//! satu string, jadi efek pertama jadi `name` dan sisanya jadi command
//! tambahan (`PluginMeta::commands`, `plugins.js:207-209`), lalu alias JS
//! (`ephoto`) menyusul — aturan yang sama seperti `siapa.rs` dan
//! `husbualbum.rs`.
//!
//! Alur scraper `ephoto(url, textInput)` (`ephoto.js:64-101`) kini di-port
//! utuh lewat `HttpGet::call` (header kustom, multipart, dan cookie sesi
//! didukung penuh):
//!
//! 1. GET halaman efek dengan UA Chrome → kumpulkan `set-cookie` dan baca
//!    `<input name=token|build_server|build_server_id>` dari HTML.
//! 2. POST multipart ke URL yang sama (`text[]`, token, build_server,
//!    build_server_id + cookie sesi) → baca `<input name=form_value_input>`
//!    (nilai JSON), salin `text` ke kunci `text[]`.
//! 3. POST urlencoded `form_value_input` ke `/effect/create-image` dengan
//!    cookie sesi → ambil `image` dari JSON-nya.
//!
//! Hasilnya `buildServer + image` (`ephoto.js:100`).
//!
//! `ponytail:` JS melempar di langkah 3 kalau `set-cookie` kosong
//! (`initialResponse.headers['set-cookie'].join`), dan `sendMedia` di dalam
//! `try` gagal kalau URL hasilnya tidak bisa diunduh — keduanya jatuh ke
//! template error yang sama, jadi di sini keduanya `Err`.

use oktz_plugin_api::{
    Ctx, HttpCall, HttpGet, HttpResponse, MediaSource, MultipartPart, Reply, plugin,
};
use serde_json::Value;

/// `EFFECT_URLS` (`ephoto.js:5-38`) — urutan objek JS dipertahankan karena
/// `Object.keys(...).map(...)` di `ephoto.js:117` ikut bergantung padanya.
/// Kunci bukan identifier Rust yang sah (`1917style`), jadi tabel bukan map.
pub(crate) const EFEK: [(&str, &str); 32] = [
    (
        "glitchtext",
        "https://en.ephoto360.com/create-digital-glitch-text-effects-online-767.html",
    ),
    (
        "writetext",
        "https://en.ephoto360.com/write-text-on-wet-glass-online-589.html",
    ),
    (
        "advancedglow",
        "https://en.ephoto360.com/advanced-glow-effects-74.html",
    ),
    (
        "typographytext",
        "https://en.ephoto360.com/create-typography-text-effect-on-pavement-online-774.html",
    ),
    (
        "pixelglitch",
        "https://en.ephoto360.com/create-pixel-glitch-text-effect-online-769.html",
    ),
    (
        "neonglitch",
        "https://en.ephoto360.com/create-impressive-neon-glitch-text-effects-online-768.html",
    ),
    (
        "flagtext",
        "https://en.ephoto360.com/nigeria-3d-flag-text-effect-online-free-753.html",
    ),
    (
        "flag3dtext",
        "https://en.ephoto360.com/free-online-american-flag-3d-text-effect-generator-725.html",
    ),
    (
        "deletingtext",
        "https://en.ephoto360.com/create-eraser-deleting-text-effect-online-717.html",
    ),
    (
        "blackpinkstyle",
        "https://en.ephoto360.com/online-blackpink-style-logo-maker-effect-711.html",
    ),
    (
        "glowingtext",
        "https://en.ephoto360.com/create-glowing-text-effects-online-706.html",
    ),
    (
        "underwatertext",
        "https://en.ephoto360.com/3d-underwater-text-effect-online-682.html",
    ),
    (
        "logomaker",
        "https://en.ephoto360.com/free-bear-logo-maker-online-673.html",
    ),
    (
        "cartoonstyle",
        "https://en.ephoto360.com/create-a-cartoon-style-graffiti-text-effect-online-668.html",
    ),
    (
        "papercutstyle",
        "https://en.ephoto360.com/multicolor-3d-paper-cut-style-text-effect-658.html",
    ),
    (
        "watercolortext",
        "https://en.ephoto360.com/create-a-watercolor-text-effect-online-655.html",
    ),
    (
        "effectclouds",
        "https://en.ephoto360.com/write-text-effect-clouds-in-the-sky-online-619.html",
    ),
    (
        "blackpinklogo",
        "https://en.ephoto360.com/create-blackpink-logo-online-free-607.html",
    ),
    (
        "gradienttext",
        "https://en.ephoto360.com/create-3d-gradient-text-effect-online-600.html",
    ),
    (
        "summerbeach",
        "https://en.ephoto360.com/write-in-sand-summer-beach-online-free-595.html",
    ),
    (
        "luxurygold",
        "https://en.ephoto360.com/create-a-luxury-gold-text-effect-online-594.html",
    ),
    (
        "multicoloredneon",
        "https://en.ephoto360.com/create-multicolored-neon-light-signatures-591.html",
    ),
    (
        "sandsummer",
        "https://en.ephoto360.com/write-in-sand-summer-beach-online-576.html",
    ),
    (
        "galaxywallpaper",
        "https://en.ephoto360.com/create-galaxy-wallpaper-mobile-online-528.html",
    ),
    (
        "1917style",
        "https://en.ephoto360.com/1917-style-text-effect-523.html",
    ),
    (
        "makingneon",
        "https://en.ephoto360.com/making-neon-light-text-effect-with-galaxy-style-521.html",
    ),
    (
        "royaltext",
        "https://en.ephoto360.com/royal-text-effect-online-free-471.html",
    ),
    (
        "freecreate",
        "https://en.ephoto360.com/free-create-a-3d-hologram-text-effect-441.html",
    ),
    (
        "galaxystyle",
        "https://en.ephoto360.com/create-galaxy-style-free-name-logo-438.html",
    ),
    (
        "amongustext",
        "https://en.ephoto360.com/create-a-cover-image-for-the-game-among-us-online-762.html",
    ),
    (
        "rainytext",
        "https://en.ephoto360.com/foggy-rainy-text-effect-75.html",
    ),
    ("lighteffects", "https://en.ephoto360.com/light-effects-74.html"),
];

/// `name` (efek pertama) + sisa array `name` + alias JS `ephoto` — urutan ini
/// dipakai tes untuk membandingkan attribute `#[plugin]` dengan array JS.
#[cfg(test)]
pub(crate) const COMMANDS: [&str; 33] = [
    "glitchtext",
    "writetext",
    "advancedglow",
    "typographytext",
    "pixelglitch",
    "neonglitch",
    "flagtext",
    "flag3dtext",
    "deletingtext",
    "blackpinkstyle",
    "glowingtext",
    "underwatertext",
    "logomaker",
    "cartoonstyle",
    "papercutstyle",
    "watercolortext",
    "effectclouds",
    "blackpinklogo",
    "gradienttext",
    "summerbeach",
    "luxurygold",
    "multicoloredneon",
    "sandsummer",
    "galaxywallpaper",
    "1917style",
    "makingneon",
    "royaltext",
    "freecreate",
    "galaxystyle",
    "amongustext",
    "rainytext",
    "lighteffects",
    "ephoto",
];

/// `'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like
/// Gecko) Chrome/105.0.0.0 Safari/537.36'` — dipakai ketiga langkah
/// (`ephoto.js:68,85,94`).
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36";

/// `https://en.ephoto360.com/effect/create-image` (`ephoto.js:92`).
const ENDPOINT_CREATE: &str = "https://en.ephoto360.com/effect/create-image";

/// `EFFECT_URLS[command]` (`ephoto.js:130`).
pub(crate) fn url_efek(command: &str) -> Option<&'static str> {
    EFEK
        .iter()
        .find(|(nama, _)| *nama == command)
        .map(|(_, url)| *url)
}

/// `ephoto.js:117-123` — daftar efek untuk command `ephoto`.
pub(crate) fn daftar_efek(prefix: &str) -> String {
    let mut out = String::new();
    for (nama, _) in EFEK.iter() {
        out.push_str(&format!("• `{prefix}{nama}`\n"));
    }
    out.pop();
    out
}

/// `ephoto.js:127` — teks kosong.
pub(crate) fn tanpa_teks(prefix: &str, command: &str) -> String {
    format!("❌ *ᴇʀʀᴏʀ*\n\n> Masukkan text!\n> *Contoh:* {prefix}{command} Foto-AI")
}

/// `ephoto.js:132` — efek tidak ada di peta.
pub(crate) const EFEK_TAK_ADA: &str = "❌ Efek tidak ditemukan";

/// `te(m.prefix, m.command, m.pushName)` dengan `config.errorTemplate`
/// (`config.js:153`). `m.pushName` tidak ada di `Ctx`, jadi fallback `'User'`
/// yang dipakai — sama seperti `tod.rs`/`wwkill.rs`.
#[allow(dead_code)]
pub(crate) fn error_template(prefix: &str, command: &str) -> String {
    format!(
        "☢ Ehhh~ command `{prefix}{command}` lagi ada kendala, User\n\
         B-bukan karena kamu salah kok... coba lagi nanti ya (˃̣̣̥ᯅ˂̣̣̥)\n\n\
         _Jika masalah berlanjut, hubungi owner bot_"
    )
}

/// `initialResponse.headers['set-cookie']?.join('; ')` (`ephoto.js:87`) —
/// axios menyimpan `set-cookie` sebagai array, jadi semua entri digabung.
fn cookie_header(headers: &[(String, String)]) -> Option<String> {
    let cookies: Vec<&str> = headers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, v)| v.as_str())
        .collect();
    (!cookies.is_empty()).then(|| cookies.join("; "))
}

/// Baca satu atribut dari string tag `<input ...>` — nilai bisa berkutip
/// ganda, berkutip tunggal, atau tanpa kutip.
fn atribut(tag: &str, nama: &str) -> Option<String> {
    let mut i = 0;
    let bytes = tag.as_bytes();
    while i < bytes.len() {
        // lewati spasi antar atribut
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        // nama atribut: sampai '=', whitespace, atau '/'
        let mulai = i;
        while i < bytes.len()
            && bytes[i] != b'='
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let key = &tag[mulai..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        // atribut tanpa nilai (`<input disabled>`)
        if i >= bytes.len() || bytes[i] != b'=' {
            if key.eq_ignore_ascii_case(nama) {
                return Some(String::new());
            }
            continue;
        }
        i += 1; // '='
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let nilai = if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
            let kutip = bytes[i];
            i += 1;
            let mulai = i;
            while i < bytes.len() && bytes[i] != kutip {
                i += 1;
            }
            let v = &tag[mulai..i.min(tag.len())];
            i += 1; // kutip penutup
            v
        } else {
            let mulai = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                i += 1;
            }
            &tag[mulai..i]
        };
        if key.eq_ignore_ascii_case(nama) {
            return Some(nilai.to_string());
        }
    }
    None
}

/// HTML-escape: `&quot;` `&amp;` `&#39;` `&apos;` `&lt;` `&gt;` `&nbsp;` dan
/// entri numerik (`&#68;` / `&#x44;`). `cheerio` men-decode entitas ini saat
/// membaca nilai atribut, jadi `.val()` hasilnya sudah polos.
fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'&'
            && let Some(semi) = s[i + 1..].find(';').map(|p| p + i + 1).filter(|&p| p - i <= 10) {
                let ent = &s[i + 1..semi];
                let ganti = match ent {
                    "quot" => Some('"'),
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "apos" => Some('\''),
                    "nbsp" => Some('\u{a0}'),
                    _ => ent.strip_prefix('#').and_then(|num| {
                        let code = if let Some(hex) =
                            num.strip_prefix('x').or_else(|| num.strip_prefix('X'))
                        {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            num.parse::<u32>().ok()
                        };
                        code.and_then(char::from_u32)
                    }),
                };
                if let Some(ch) = ganti {
                    out.push(ch);
                    i = semi + 1;
                    continue;
                }
            }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// `$('input[name=X]').val()` (`ephoto.js:76-79,88`) — cari tag `<input>`
/// pertama dengan atribut `name` yang cocok, lalu baca `value`-nya.
///
/// `ponytail:` elemen tanpa atribut `value` menghasilkan string kosong
/// (`cheerio` `.val()`), sedangkan `input` tidak ditemukan sama sekali
/// menghasilkan `undefined` di JS — di sini jadi `Err` agar jatuh ke template
/// error, bukan mengirim `"undefined"` ke server.
fn input_value(html: &str, nama: &str) -> Option<String> {
    let mut rest = html;
    while let Some(mulai) = rest.find("<input") {
        let sisa = &rest[mulai..];
        let selesai = sisa.find('>')?;
        let tag = &sisa[..=selesai];
        if let Some(name) = atribut(tag, "name")
            && name == nama
        {
            return Some(decode_entities(&atribut(tag, "value").unwrap_or_default()));
        }
        rest = &sisa[selesai + 1..];
    }
    None
}

/// `String(nilai)` di JS — koersi yang dipakai `new URLSearchParams(obj)`
/// (`ephoto.js:93`) untuk tiap nilai properti.
fn js_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // `String(1917)` = `"1917"`, bukan `"1917.0"` — angka bulat JS
            // tidak punya bagian pecahan.
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64()
                && f.fract() == 0.0
                && f.abs() < 1e15
            {
                format!("{}", f as i64)
            } else {
                n.to_string()
            }
        }
        Value::Null => "null".into(),
        // `String([1,2])` = `"1,2"` — elemen array dikoersi satu per satu.
        Value::Array(a) => a
            .iter()
            .map(js_string)
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}

/// Serializer `application/x-www-form-urlencoded` milik
/// `new URLSearchParams(...)` — spasi jadi `+`, sisanya (kecuali
/// `A-Za-z0-9*-._`) di-escape sebagai byte UTF-8.
fn encode_komponen(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                out.push(*b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn form_encode(pasangan: &[(String, String)]) -> String {
    pasangan
        .iter()
        .map(|(k, v)| format!("{}={}", encode_komponen(k), encode_komponen(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// `axios` menolak status non-2xx (`validateStatus` bawaan) — semua `catch`
/// di handler JS berasal dari sini atau dari field yang hilang.
fn periksa_status(res: &HttpResponse) -> Result<(), String> {
    if (200..300).contains(&res.status) {
        Ok(())
    } else {
        Err(format!("HTTP {}", res.status))
    }
}

/// `ephoto(url, textInput)` (`ephoto.js:64-101`) — tiga langkah scraper:
/// GET halaman efek, POST multipart ber-cookie, POST final ke endpoint
/// `create-image`. Hasil: `buildServer + image`.
fn ephoto(http: &dyn HttpGet, url: &str, text: &str) -> Result<String, String> {
    // Langkah 1 (`ephoto.js:67-74`): GET halaman efek dengan UA Chrome.
    let awal = http
        .call(
            &HttpCall::get(url)
                .terima_semua_status()
                .header("user-agent", USER_AGENT),
        )
        .map_err(|e| e.message)?;
    periksa_status(&awal)?;
    let cookie = cookie_header(&awal.headers);
    let html = awal.text();

    let token = input_value(&html, "token").ok_or("input token tidak ditemukan")?;
    let build_server =
        input_value(&html, "build_server").ok_or("input build_server tidak ditemukan")?;
    let build_server_id = input_value(&html, "build_server_id")
        .ok_or("input build_server_id tidak ditemukan")?;

    // Langkah 2 (`ephoto.js:66,81-90`): POST multipart ke URL yang sama.
    // `formData.append('text[]', textInput)` dulu, baru token dst — urutan
    // part dipertahankan.
    let mut panggilan = HttpCall::post(url)
        .terima_semua_status()
        .multipart(vec![
            MultipartPart::field("text[]", text),
            MultipartPart::field("token", token),
            MultipartPart::field("build_server", build_server.clone()),
            MultipartPart::field("build_server_id", build_server_id),
        ])
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("user-agent", USER_AGENT);
    // `'cookie': initialResponse.headers['set-cookie']?.join('; ')` — opsional:
    // kalau kosong, axios tidak mengirim header cookie sama sekali.
    if let Some(c) = &cookie {
        panggilan = panggilan.header("cookie", c);
    }
    let post = http.call(&panggilan).map_err(|e| e.message)?;
    periksa_status(&post)?;
    let post_html = post.text();

    // `JSON.parse($$('input[name=form_value_input]').val())` (`ephoto.js:88`).
    let nilai_form = input_value(&post_html, "form_value_input")
        .ok_or("input form_value_input tidak ditemukan")?;
    let parsed: Value = serde_json::from_str(&nilai_form)
        .map_err(|e| format!("form_value_input bukan JSON: {e}"))?;
    let obj = parsed.as_object().ok_or("form_value_input bukan objek")?;

    // `formValueInput['text[]'] = formValueInput.text; delete
    // formValueInput.text` (`ephoto.js:89-90`) — kunci lain tetap urut asli,
    // `text[]` ditambahkan di akhir.
    let mut pasangan: Vec<(String, String)> = Vec::with_capacity(obj.len());
    for (k, v) in obj {
        if k == "text" {
            continue;
        }
        pasangan.push((k.clone(), js_string(v)));
    }
    // `obj.text` yang hilang jadi `undefined` → `"undefined"` setelah
    // koersi `URLSearchParams`.
    let teks_akhir = obj
        .get("text")
        .map(js_string)
        .unwrap_or_else(|| "undefined".into());
    pasangan.push(("text[]".into(), teks_akhir));

    // Langkah 3 (`ephoto.js:92-98`): POST urlencoded ke endpoint create-image.
    // JS tanpa `.?` di sini — `set-cookie` kosong melempar dan jatuh ke
    // template error.
    let cookie = cookie.ok_or("set-cookie kosong")?;
    let akhir = http
        .call(
            &HttpCall::post(ENDPOINT_CREATE)
                .terima_semua_status()
                .form(form_encode(&pasangan))
                .header("user-agent", USER_AGENT)
                .header("cookie", cookie),
        )
        .map_err(|e| e.message)?;
    periksa_status(&akhir)?;

    // `finalResponseData.image` (`ephoto.js:99`).
    let image = akhir
        .json()
        .and_then(|v| v.get("image")?.as_str().map(str::to_string))
        .ok_or("field image tidak ditemukan")?;
    Ok(format!("{build_server}{image}"))
}

#[plugin(
    name = "glitchtext",
    commands = [
        "writetext", "advancedglow", "typographytext", "pixelglitch", "neonglitch", "flagtext",
        "flag3dtext", "deletingtext", "blackpinkstyle", "glowingtext", "underwatertext",
        "logomaker", "cartoonstyle", "papercutstyle", "watercolortext", "effectclouds",
        "blackpinklogo", "gradienttext", "summerbeach", "luxurygold", "multicoloredneon",
        "sandsummer", "galaxywallpaper", "1917style", "makingneon", "royaltext", "freecreate",
        "galaxystyle", "amongustext", "rainytext", "lighteffects",
    ],
    alias = ["ephoto"],
    category = "ephoto",
    description = "Buat efek text keren dengan berbagai style",
    usage = ".<effect> <text>",
    example = ".glitchtext Foto-AI",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn glitchtext(ctx: Ctx) -> Reply {
    let command = ctx.command.to_lowercase();
    let text = ctx.text.trim();

    if command == "ephoto" {
        return Reply::text(format!(
            "🎨 *ᴇᴘʜᴏᴛᴏ ᴇꜰꜰᴇᴄᴛs*\n\n\
             > Buat efek text keren!\n\n\
             ╭┈┈⬡「 📋 *ᴅᴀꜰᴛᴀʀ ᴇꜰᴇᴋ* 」\n{}\n╰┈┈┈┈┈┈┈┈⬡\n\n\
             > *Contoh:* {}glitchtext Foto-AI",
            daftar_efek(&ctx.prefix),
            ctx.prefix
        ));
    }

    if text.is_empty() {
        return Reply::text(tanpa_teks(&ctx.prefix, &command));
    }

    let Some(effect_url) = url_efek(&command) else {
        return Reply::text(EFEK_TAK_ADA);
    };

    match ephoto(&*ctx.http, effect_url, text) {
        // `sock.sendMedia(..., null, m, { type: 'image' })` — caption null,
        // jadi caption kosong dan yang dibawa hanya URL gambarnya.
        Ok(image_url) => Reply::image(MediaSource::url(image_url), "").with_reaction("✅"),
        Err(_) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpMethod, NoHttp};
    use std::sync::{Arc, Mutex};

    fn run_with(http: Arc<dyn HttpGet>, command: &str, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: command.into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: if text.is_empty() {
                Vec::new()
            } else {
                vec![text.into()]
            },
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("glitchtext").unwrap().handler)(ctx)
    }

    fn run(command: &str, text: &str) -> Reply {
        run_with(Arc::new(NoHttp), command, text)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    fn gambar_dari(r: &Reply) -> Option<String> {
        let dalam = match r.without_mutations() {
            Reply::Reaction { reply, .. } => &**reply,
            lain => lain,
        };
        match dalam {
            Reply::Image {
                media: Some(MediaSource::Url { url, .. }),
                ..
            } => Some(url.clone()),
            _ => None,
        }
    }

    /// `ephoto.js:118-123` dirakit utuh supaya teks tampilan bisa dibandingkan
    /// tanpa menyalin 32 baris efek dua kali.
    fn harapan_daftar(prefix: &str) -> String {
        format!(
            "🎨 *ᴇᴘʜᴏᴛᴏ ᴇꜰꜰᴇᴄᴛs*\n\n\
             > Buat efek text keren!\n\n\
             ╭┈┈⬡「 📋 *ᴅᴀꜰᴛᴀʀ ᴇꜰᴇᴋ* 」\n{}\n╰┈┈┈┈┈┈┈┈⬡\n\n\
             > *Contoh:* {prefix}glitchtext Foto-AI",
            daftar_efek(prefix)
        )
    }

    /// Scraper tiruan: GET halaman efek, POST multipart, POST create-image —
    /// ketiganya seperti en.ephoto360.com sungguhan.
    #[derive(Debug, Default)]
    struct ScraperStub {
        /// Header respons GET; tanpa `set-cookie` untuk menguji jalur error.
        cookie: Vec<(&'static str, &'static str)>,
        /// JSON `form_value_input` yang dibungkus tag input di HTML POST.
        nilai_form: &'static str,
        /// Field `image` di respons create-image; `None` = tidak ada.
        image: Option<&'static str>,
        /// Semua call yang masuk, untuk inspeksi header/body.
        panggilan: Mutex<Vec<HttpCall>>,
    }

    impl ScraperStub {
        fn html_halaman() -> String {
            "<html><form>\
             <input type=\"hidden\" name=\"token\" value=\"tok-123\">\
             <input type=\"hidden\" name=\"build_server\" value=\"https://e.ephoto360.com/\">\
             <input type=\"hidden\" name=\"build_server_id\" value=\"e.ephoto360.com\">\
             </form></html>"
                .into()
        }

        fn html_post(nilai_form: &str) -> String {
            // Nilai JSON di-escape sebagai entitas, seperti HTML nyata.
            format!("<html><input name=\"form_value_input\" value='{nilai_form}'></html>")
        }

        fn respons(status: u16, headers: Vec<(&str, &str)>, body: String) -> HttpResponse {
            HttpResponse {
                status,
                body: HttpBody::Bytes {
                    data: Arc::from(body.into_bytes()),
                    mimetype: None,
                },
                headers: headers
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        }

        fn catat(&self, req: &HttpCall) {
            self.panggilan.lock().unwrap().push(req.clone());
        }
    }

    /// Cari satu header di daftar `HttpCall::headers` (case-insensitive).
    fn header_dari<'a>(req: &'a HttpCall, nama: &str) -> Option<&'a str> {
        req.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(nama))
            .map(|(_, v)| v.as_str())
    }

    impl HttpGet for ScraperStub {
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "pakai call()".into(),
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "pakai call()".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "pakai call()".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "pakai call()".into(),
            })
        }

        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.catat(req);
            match (req.method, &req.body) {
                (HttpMethod::Get, _) => Ok(Self::respons(
                    200,
                    self.cookie.clone(),
                    Self::html_halaman(),
                )),
                (HttpMethod::Post, Some(HttpBody::Multipart(_))) => Ok(Self::respons(
                    200,
                    vec![("content-type", "text/html")],
                    Self::html_post(self.nilai_form),
                )),
                (HttpMethod::Post, Some(HttpBody::Form(_))) => Ok(Self::respons(
                    200,
                    vec![("content-type", "application/json")],
                    match self.image {
                        Some(img) => format!("{{\"image\":\"{img}\",\"success\":true}}"),
                        None => "{\"success\":false}".into(),
                    },
                )),
                _ => Err(HttpError {
                    message: "permintaan tak diharapkan".into(),
                }),
            }
        }
    }

    fn stub() -> ScraperStub {
        ScraperStub {
            cookie: vec![("set-cookie", "PHPSESSID=abc123; path=/"), ("set-cookie", "lang=en")],
            // `{"text":"Foto-AI","token":"tok-123"}` dalam bentuk ter-escape.
            nilai_form: "{&quot;text&quot;:&quot;Foto-AI&quot;,&quot;token&quot;:&quot;tok-123&quot;}",
            image: Some("image/2024/10/abcd.jpg"),
            panggilan: Mutex::new(Vec::new()),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("glitchtext").unwrap();
        assert_eq!(p.meta.name, "glitchtext");
        assert_eq!(p.meta.commands.len(), 31);
        assert_eq!(
            p.meta.commands,
            COMMANDS[1..32]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(p.meta.alias, ["ephoto"]);
        assert_eq!(p.meta.category, "ephoto");
        assert_eq!(p.meta.description, "Buat efek text keren dengan berbagai style");
        assert_eq!(p.meta.usage, ".<effect> <text>");
        assert_eq!(p.meta.example, ".glitchtext Foto-AI");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
        assert!(p.meta.is_enabled);
    }

    #[test]
    fn every_effect_name_resolves_to_this_plugin() {
        for key in COMMANDS {
            assert_eq!(
                oktz_plugin_api::find(key).map(|p| p.meta.name.clone()),
                Some("glitchtext".to_string()),
                "key {key} tidak ditemukan"
            );
        }
    }

    #[test]
    fn the_effect_map_keeps_the_js_key_order_and_urls() {
        assert_eq!(EFEK.len(), 32);
        assert_eq!(EFEK[0].0, "glitchtext");
        assert_eq!(
            EFEK[0].1,
            "https://en.ephoto360.com/create-digital-glitch-text-effects-online-767.html"
        );
        assert_eq!(EFEK[31].0, "lighteffects");
        assert_eq!(EFEK[31].1, "https://en.ephoto360.com/light-effects-74.html");
        // `1917style` bukan identifier Rust — masih harus terbaca sebagai efek.
        assert_eq!(EFEK[24].0, "1917style");
        assert_eq!(EFEK[24].1, "https://en.ephoto360.com/1917-style-text-effect-523.html");
        assert_eq!(url_efek("1917style"), Some(EFEK[24].1));
        assert_eq!(url_efek("glitchtext"), Some(EFEK[0].1));
        assert_eq!(url_efek("writetext"), Some(EFEK[1].1));
        assert_eq!(
            url_efek("writetext").unwrap(),
            "https://en.ephoto360.com/write-text-on-wet-glass-online-589.html"
        );
        assert_eq!(
            url_efek("blackpinkstyle").unwrap(),
            "https://en.ephoto360.com/online-blackpink-style-logo-maker-effect-711.html"
        );
        assert_eq!(
            url_efek("rainytext").unwrap(),
            "https://en.ephoto360.com/foggy-rainy-text-effect-75.html"
        );
        assert_eq!(url_efek("ephoto"), None, "alias bukan kunci peta");
        assert_eq!(url_efek("tidak-ada"), None);
    }

    /// Semua 32 URL template sama persis dengan `EFFECT_URLS` JS
    /// (`ephoto.js:5-38`) — diulang utuh di sini supaya pergeseran satu baris
    /// saja ketahuan.
    #[test]
    fn every_effect_url_matches_the_js_table_verbatim() {
        let js: &[(&str, &str)] = &[
            ("glitchtext", "https://en.ephoto360.com/create-digital-glitch-text-effects-online-767.html"),
            ("writetext", "https://en.ephoto360.com/write-text-on-wet-glass-online-589.html"),
            ("advancedglow", "https://en.ephoto360.com/advanced-glow-effects-74.html"),
            ("typographytext", "https://en.ephoto360.com/create-typography-text-effect-on-pavement-online-774.html"),
            ("pixelglitch", "https://en.ephoto360.com/create-pixel-glitch-text-effect-online-769.html"),
            ("neonglitch", "https://en.ephoto360.com/create-impressive-neon-glitch-text-effects-online-768.html"),
            ("flagtext", "https://en.ephoto360.com/nigeria-3d-flag-text-effect-online-free-753.html"),
            ("flag3dtext", "https://en.ephoto360.com/free-online-american-flag-3d-text-effect-generator-725.html"),
            ("deletingtext", "https://en.ephoto360.com/create-eraser-deleting-text-effect-online-717.html"),
            ("blackpinkstyle", "https://en.ephoto360.com/online-blackpink-style-logo-maker-effect-711.html"),
            ("glowingtext", "https://en.ephoto360.com/create-glowing-text-effects-online-706.html"),
            ("underwatertext", "https://en.ephoto360.com/3d-underwater-text-effect-online-682.html"),
            ("logomaker", "https://en.ephoto360.com/free-bear-logo-maker-online-673.html"),
            ("cartoonstyle", "https://en.ephoto360.com/create-a-cartoon-style-graffiti-text-effect-online-668.html"),
            ("papercutstyle", "https://en.ephoto360.com/multicolor-3d-paper-cut-style-text-effect-658.html"),
            ("watercolortext", "https://en.ephoto360.com/create-a-watercolor-text-effect-online-655.html"),
            ("effectclouds", "https://en.ephoto360.com/write-text-effect-clouds-in-the-sky-online-619.html"),
            ("blackpinklogo", "https://en.ephoto360.com/create-blackpink-logo-online-free-607.html"),
            ("gradienttext", "https://en.ephoto360.com/create-3d-gradient-text-effect-online-600.html"),
            ("summerbeach", "https://en.ephoto360.com/write-in-sand-summer-beach-online-free-595.html"),
            ("luxurygold", "https://en.ephoto360.com/create-a-luxury-gold-text-effect-online-594.html"),
            ("multicoloredneon", "https://en.ephoto360.com/create-multicolored-neon-light-signatures-591.html"),
            ("sandsummer", "https://en.ephoto360.com/write-in-sand-summer-beach-online-576.html"),
            ("galaxywallpaper", "https://en.ephoto360.com/create-galaxy-wallpaper-mobile-online-528.html"),
            ("1917style", "https://en.ephoto360.com/1917-style-text-effect-523.html"),
            ("makingneon", "https://en.ephoto360.com/making-neon-light-text-effect-with-galaxy-style-521.html"),
            ("royaltext", "https://en.ephoto360.com/royal-text-effect-online-free-471.html"),
            ("freecreate", "https://en.ephoto360.com/free-create-a-3d-hologram-text-effect-441.html"),
            ("galaxystyle", "https://en.ephoto360.com/create-galaxy-style-free-name-logo-438.html"),
            ("amongustext", "https://en.ephoto360.com/create-a-cover-image-for-the-game-among-us-online-762.html"),
            ("rainytext", "https://en.ephoto360.com/foggy-rainy-text-effect-75.html"),
            ("lighteffects", "https://en.ephoto360.com/light-effects-74.html"),
        ];
        assert_eq!(EFEK.as_slice(), js);
    }

    #[test]
    fn the_ephoto_alias_lists_every_effect_in_order() {
        let r = run("ephoto", "");
        assert_eq!(text_of(&r), harapan_daftar("."));
        assert_eq!(reaksi(&r), None, "daftar efek bukan kegagalan");
    }

    #[test]
    fn the_prefix_is_substituted_in_the_effect_list() {
        assert!(daftar_efek(".").starts_with("• `.glitchtext`\n"));
        assert!(daftar_efek("!").starts_with("• `!glitchtext`\n"));
        assert!(harapan_daftar("!").contains("> *Contoh:* !glitchtext Foto-AI"));
    }

    #[test]
    fn an_empty_text_asks_for_one_before_touching_the_network() {
        for command in ["glitchtext", "royaltext", "1917style"] {
            for text in ["", "   "] {
                let r = run(command, text);
                assert_eq!(
                    text_of(&r),
                    format!("❌ *ᴇʀʀᴏʀ*\n\n> Masukkan text!\n> *Contoh:* .{command} Foto-AI"),
                    "command {command}, teks {text:?}"
                );
            }
        }
    }

    #[test]
    fn an_unknown_effect_is_rejected() {
        let r = run("glitchtex", "Foto-AI");
        assert_eq!(text_of(&r), EFEK_TAK_ADA);
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn a_failed_scrape_reacts_and_reports_the_error_template() {
        let r = run("glitchtext", "Foto-AI");
        assert_eq!(reaksi(&r), Some("☢"));
        assert_eq!(
            text_of(&r),
            "☢ Ehhh~ command `.glitchtext` lagi ada kendala, User\n\
             B-bukan karena kamu salah kok... coba lagi nanti ya (˃̣̣̥ᯅ˂̣̣̥)\n\n\
             _Jika masalah berlanjut, hubungi owner bot_"
        );
    }

    #[test]
    fn the_effect_list_has_one_line_per_effect_with_the_prefix_applied() {
        let daftar = daftar_efek("!");
        let baris: Vec<&str> = daftar.lines().collect();
        assert_eq!(baris.len(), 32);
        assert_eq!(baris[0], "• `!glitchtext`");
        assert_eq!(baris[31], "• `!lighteffects`");
        assert!(!daftar.ends_with('\n'), "join tidak menambah baris kosong");
    }

    // ---- scraper `ephoto(url, textInput)` (`ephoto.js:64-101`) ----

    #[test]
    fn the_scraper_returns_build_server_plus_image() {
        let s = Arc::new(stub());
        let r = run_with(s.clone(), "royaltext", "Foto-AI");
        assert_eq!(
            gambar_dari(&r),
            Some("https://e.ephoto360.com/image/2024/10/abcd.jpg".into())
        );
        assert_eq!(reaksi(&r), Some("✅"));
    }

    #[test]
    fn the_get_step_uses_the_js_user_agent_and_the_effect_url() {
        let s = Arc::new(stub());
        run_with(s.clone(), "glitchtext", "Foto-AI");
        let panggilan = s.panggilan.lock().unwrap();
        let get = &panggilan[0];
        assert_eq!(get.method, HttpMethod::Get);
        assert_eq!(
            get.url,
            "https://en.ephoto360.com/create-digital-glitch-text-effects-online-767.html"
        );
        assert_eq!(header_dari(get, "user-agent"), Some(USER_AGENT));
    }

    #[test]
    fn the_multipart_step_carries_text_token_and_the_session_cookie() {
        let s = Arc::new(stub());
        run_with(s.clone(), "glitchtext", "Foto-AI");
        let panggilan = s.panggilan.lock().unwrap();
        let post = &panggilan[1];
        assert_eq!(
            post.url,
            "https://en.ephoto360.com/create-digital-glitch-text-effects-online-767.html"
        );
        assert_eq!(
            header_dari(post, "cookie"),
            Some("PHPSESSID=abc123; path=/; lang=en")
        );
        assert_eq!(header_dari(post, "Accept"), Some("*/*"));
        assert_eq!(
            header_dari(post, "Accept-Language"),
            Some("en-US,en;q=0.9")
        );
        let Some(HttpBody::Multipart(parts)) = &post.body else {
            panic!("langkah 2 harus multipart");
        };
        let isi: Vec<(String, String)> = parts
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    String::from_utf8(p.body.bytes().to_vec()).unwrap(),
                )
            })
            .collect();
        assert_eq!(
            isi,
            vec![
                ("text[]".into(), "Foto-AI".into()),
                ("token".into(), "tok-123".into()),
                ("build_server".into(), "https://e.ephoto360.com/".into()),
                ("build_server_id".into(), "e.ephoto360.com".into()),
            ]
        );
    }

    #[test]
    fn the_final_step_posts_the_rekeyed_form_value_to_create_image() {
        let s = Arc::new(stub());
        run_with(s.clone(), "glitchtext", "Foto-AI");
        let panggilan = s.panggilan.lock().unwrap();
        let post = &panggilan[2];
        assert_eq!(post.url, "https://en.ephoto360.com/effect/create-image");
        // `text` dihapus, `text[]` ditambahkan di akhir (`ephoto.js:89-90`).
        assert_eq!(
            header_dari(post, "cookie"),
            Some("PHPSESSID=abc123; path=/; lang=en")
        );
        let Some(HttpBody::Form(form)) = &post.body else {
            panic!("langkah 3 harus form-urlencoded");
        };
        assert_eq!(form, "token=tok-123&text%5B%5D=Foto-AI");
    }

    #[test]
    fn a_missing_set_cookie_fails_like_the_js_join() {
        // `initialResponse.headers['set-cookie'].join('; ')` di langkah 3
        // melempar kalau tidak ada cookie (`ephoto.js:94`).
        let s = Arc::new(ScraperStub {
            cookie: vec![],
            ..stub()
        });
        let r = run_with(s, "glitchtext", "Foto-AI");
        assert_eq!(reaksi(&r), Some("☢"));
        assert!(text_of(&r).contains("lagi ada kendala"));
    }

    #[test]
    fn a_missing_image_field_reports_the_error_template() {
        // `sendMedia` dengan URL rusak melempar di dalam `try` — hasil akhirnya
        // tetap template error.
        let s = Arc::new(ScraperStub {
            image: None,
            ..stub()
        });
        let r = run_with(s, "glitchtext", "Foto-AI");
        assert_eq!(reaksi(&r), Some("☢"));
        assert!(text_of(&r).contains("lagi ada kendala"));
    }

    #[test]
    fn form_value_input_entities_are_decoded_before_json_parse() {
        // Nilai atribut berkutip tunggal penuh `&quot;`, seperti HTML nyata.
        let html = "<input name=\"form_value_input\" value='{&quot;text&quot;:&quot;A B&quot;}'>";
        assert_eq!(
            input_value(html, "form_value_input"),
            Some("{\"text\":\"A B\"}".into())
        );
        assert_eq!(input_value(html, "token"), None);
    }

    #[test]
    fn input_value_matches_exact_names_and_skips_others() {
        let html = ScraperStub::html_halaman();
        assert_eq!(input_value(&html, "token"), Some("tok-123".into()));
        // `build_server` tidak boleh kena `build_server_id`.
        assert_eq!(
            input_value(&html, "build_server"),
            Some("https://e.ephoto360.com/".into())
        );
        assert_eq!(
            input_value(&html, "build_server_id"),
            Some("e.ephoto360.com".into())
        );
        assert_eq!(input_value(&html, "tidak-ada"), None);
    }

    #[test]
    fn js_string_coerces_like_javascript() {
        assert_eq!(js_string(&Value::String("x".into())), "x");
        assert_eq!(js_string(&serde_json::json!(1917)), "1917");
        assert_eq!(js_string(&serde_json::json!(true)), "true");
        assert_eq!(js_string(&Value::Null), "null");
        assert_eq!(js_string(&serde_json::json!([1, 2])), "1,2");
        assert_eq!(js_string(&serde_json::json!({})), "[object Object]");
    }

    #[test]
    fn form_encoding_follows_urlsearchparams() {
        assert_eq!(encode_komponen("text[]"), "text%5B%5D");
        assert_eq!(encode_komponen("Foto AI"), "Foto+AI");
        assert_eq!(encode_komponen("a&b=c"), "a%26b%3Dc");
        // `!*'()` ikut di-escape (beda dari `encodeURIComponent`).
        assert_eq!(encode_komponen("a!b'c(d)e"), "a%21b%27c%28d%29e");
        assert_eq!(
            form_encode(&[("a".into(), "1".into()), ("b c".into(), "d&e".into())]),
            "a=1&b+c=d%26e"
        );
    }
}
