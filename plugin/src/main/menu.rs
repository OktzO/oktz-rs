//! `plugins/main/menu.js` — menu utama.
//!
//! Port penuh: sembilan varian tampilan yang dipilih lewat setting
//! `menuVariant`, lengkap dengan kartu native-flow, tombol `single_select` /
//! `quick_reply` / `cta_url`, header media (gambar, video, lokasi), dan
//! pesan yang dirujuk (kontak palsu, kartu cuaca). Varian 16 adalah port
//! menu utama `Rubby-MD/plugins/main/menu.js` (smallcaps + tombol kategori
//! `.menu <kategori>` + tombol Owner), dipilih lewat `.setmenu v16`.
//!
//! Yang diport apa adanya:
//! - `CATEGORY_EMOJIS` (`menu.js:36-73`) dan urutan kategori
//!   (`menu.js:112-138`).
//! - `toMonoUpperBold` (`menu.js:74-108`) — huruf mono-upper-bold unicode.
//! - `buildMenuText` (`menu.js:174-303`) — blok INFO USER / INFO BOT /
//!   DAFTAR MENU.
//! - `getVerifiedQuoted` (`menu.js:305-334`) — kutipan kontak palsu.
//! - Delapan varian tampilan (`menu.js:372-1378`).
//!
//! `ponytail:` yang berbeda dari JS, beserta alasannya:
//! - "Riwayat command teratas" (`menu.js:990-1007`) dibaca dari
//!   `database/main/stats.json`, yang tidak ada di Rust — blok "History" pada
//!   varian 6 tidak ditulis.
//! - Mode global `config.mode` tidak ada di `Ctx`; baris "Mode" memakai
//!   `PUBLIC` seperti config produksi.
//! - Varian 7 mengirim video kedua yang menempel pada kartu lewat
//!   `messageAssociation` (`menu.js:1215-1229`). Di Rust video itu dikirim
//!   sebagai pesan biasa setelah kartu, jadi tidak menempel.
//! - Varian 8 memakai `prepareWAMessageMedia` + `mediaTypeOverride:
//!   "thumbnail-link"` untuk thumbnail preview; di Rust thumbnail dikirim
//!   sebagai gambar biasa tanpa field CDN preview.
//!
//! Fallback ke balasan teks polos mengikuti `try/catch` JS
//! (`menu.js:1379-1406`): kalau aset header tidak ada, kartu tidak bisa
//! dibangun lalu yang dikirim teksnya.

use std::collections::BTreeMap;

use oktz_plugin_api::registry::{
    CardContext, CardHeader, CardList, CardLocation, CardQuote, CardRow, CardSection,
    InteractiveButton, MediaSource,
};
use oktz_plugin_api::{Ctx, Reply, all_plugins, plugin};
use serde_json::json;

use crate::common::{jakarta_parts, now_ms, pad2};

/// Setting yang memilih varian tampilan (`menu.js:337`).
const SETTING_VARIANT: &str = "menuVariant";

/// `config.ui.menuVariant` (`config.js:189-191`) = 3.
const DEFAULT_VARIANT: i64 = 3;

/// Urutan kategori `menu.js:112-138` (sama dengan `buildMenuText`).
const URUTAN: &[&str] = &[
    "owner",
    "main",
    "utility",
    "tools",
    "fun",
    "game",
    "download",
    "search",
    "sticker",
    "media",
    "ai",
    "group",
    "religi",
    "info",
    "cek",
    "economy",
    "user",
    "canvas",
    "random",
    "premium",
    "ephoto",
    "jpm",
    "pushkontak",
    "panel",
    "store",
];

/// Kategori yang disembunyikan mode `md` (`menu.js:146`).
const DISEMBUNYI_MD: &[&str] = &["panel", "pushkontak", "store"];

/// `CATEGORY_EMOJIS` (`menu.js:36-73`).
fn emoji(kategori: &str) -> &'static str {
    match kategori {
        "owner" => "👑",
        "main" => "🏠",
        "utility" => "🔧",
        "tools" => "🛠️",
        "fun" => "🎮",
        "game" => "🎯",
        "download" => "📥",
        "downloader" => "📥",
        "search" => "🔍",
        "sticker" => "🖼️",
        "media" => "🎬",
        "ai" => "🤖",
        "group" => "👥",
        "religi" => "☪️",
        "islamic" => "🕌",
        "info" => "ℹ️",
        "cek" => "📁",
        "user" => "📊",
        "canvas" => "🎨",
        "random" => "🎲",
        "ephoto" => "🖌️",
        "jpm" => "📨",
        "anime" => "🍥",
        "asupan" => "🎞️",
        "clan" => "⚔️",
        "convert" => "🔄",
        "berita" => "📰",
        "rpg" => "🗡️",
        "nsfw" => "🔞",
        "linode" => "☁️",
        "primbon" => "🔮",
        "cecan" => "💃",
        "stalker" => "🕵️",
        "tts" => "🗣️",
        "vps" => "🌊",
        "panel" => "🖥️",
        _ => "📁",
    }
}

/// `toMonoUpperBold` (`menu.js:74-108`) — A-Z → 𝗔-𝗭.
fn mono_upper_bold(text: &str) -> String {
    const PETA: [char; 26] = [
        '𝗔', '𝗕', '𝗖', '𝗗', '𝗘', '𝗙', '𝗚', '𝗛', '𝗜', '𝗝', '𝗞', '𝗟', '𝗠', '𝗡', '𝗢', '𝗣',
        '𝗤', '𝗥', '𝗦', '𝗧', '𝗨', '𝗩', '𝗪', '𝗫', '𝗬', '𝗭',
    ];
    text.to_uppercase()
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                PETA[(c as u8 - b'A') as usize]
            } else {
                c
            }
        })
        .collect()
}

/// `getAssetBuffer("foto2")` + `sharp(...).resize(512, 512).webp()`
/// (`menu.js:1144-1146`) — stiker palsu pada kutipan kartu varian 7.
fn stiker_fake(ctx: &Ctx) -> Option<MediaSource> {
    let aset = ctx.config().assets.get("foto2")?;
    let byte = ctx.berkas().baca(aset).ok()?;
    let webp = oktz_plugin_api::gambar::sharp(&byte)
        .ok()?
        .ubah_ukuran(512, 512)
        .ke_webp(80)
        .ok()?;
    Some(MediaSource::bytes(webp).with_mimetype("image/webp"))
}

/// `getTimeGreeting()` (`formatter.js:508-515`), jam Asia/Jakarta.
fn sapa() -> &'static str {
    match jakarta_parts(now_ms()).3 {
        4..=9 => "Selamat Pagi 🌅",
        10..=14 => "Selamat Siang ☀️",
        15..=17 => "Selamat Sore 🌇",
        _ => "Selamat Malam 🌙",
    }
}

/// `formatUptime` (`stats.js:28-42`).
fn uptime(detik: u64) -> String {
    let hari = detik / 86_400;
    let jam = (detik % 86_400) / 3600;
    let menit = (detik % 3600) / 60;
    let sisa = detik % 60;

    let mut bagian = Vec::new();
    if hari > 0 {
        bagian.push(format!("{hari}d"));
    }
    if jam > 0 {
        bagian.push(format!("{jam}h"));
    }
    if menit > 0 {
        bagian.push(format!("{menit}m"));
    }
    if sisa > 0 || bagian.is_empty() {
        bagian.push(format!("{sisa}s"));
    }
    bagian.join(" ")
}

/// `runtime(seconds)` (`menu.js:793-801`) — "2 Jam 3 Menit 4 Detik".
fn runtime(detik: u64) -> String {
    let hari = detik / 86_400;
    let menit = (detik % 3600) / 60;
    let sisa = detik % 60;
    format!("{hari} Jam {menit} Menit {sisa} Detik")
}

/// `runtimeStr(seconds)` (`menu.js:1082-1092`) — "2 hari 3 jam 4 menit".
fn runtime_pendek(detik: u64) -> String {
    let hari = detik / 86_400;
    if hari > 0 {
        return format!("{hari} hari");
    }
    let jam = (detik % 86_400) / 3600;
    if jam > 0 {
        return format!("{jam} jam");
    }
    let menit = (detik % 3600) / 60;
    if menit > 0 {
        return format!("{menit} menit");
    }
    format!("{} detik", detik % 60)
}

/// Kategori yang punya plugin, urut seperti JS. `owner` hanya untuk owner.
pub(crate) fn kategori(ctx: &Ctx) -> Vec<(&'static str, usize)> {
    let mut hitung: BTreeMap<&'static str, usize> = BTreeMap::new();
    for p in all_plugins() {
        let cat = p.meta.category.as_str();
        if cat == "owner" && !ctx.is_owner {
            continue;
        }
        if DISEMBUNYI_MD.contains(&cat) {
            continue;
        }
        *hitung.entry(cat).or_default() += 1;
    }
    let mut hasil: Vec<(&'static str, usize)> = hitung.into_iter().collect();
    hasil.sort_by_key(|(cat, _)| {
        URUTAN
            .iter()
            .position(|u| u == cat)
            .map_or(999, |i| i as i32)
    });
    hasil
}

/// Kategori + command aktifnya — `getSortedCategories` (`menu.js:109-173`).
///
/// `getCommandsByCategory()` (`plugins.js:395-410`) hanya menghitung plugin
/// yang `isEnabled`, jadi plugin nonaktif tidak pernah masuk daftar.
fn kategori_perintah(ctx: &Ctx) -> Vec<(&'static str, Vec<&'static str>)> {
    let mut hitung: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    for p in all_plugins() {
        if !p.meta.is_enabled {
            continue;
        }
        let cat = p.meta.category.as_str();
        if cat == "owner" && !ctx.is_owner {
            continue;
        }
        if DISEMBUNYI_MD.contains(&cat) {
            continue;
        }
        hitung
            .entry(cat)
            .or_default()
            .push(p.meta.name.as_str());
    }
    let mut hasil: Vec<(&'static str, Vec<&'static str>)> = hitung.into_iter().collect();
    hasil.sort_by_key(|(cat, _)| {
        URUTAN
            .iter()
            .position(|u| u == cat)
            .map_or(999, |i| i as i32)
    });
    hasil
}

/// Prefix perintah — `m.prefix` / `config.command.prefix`.
fn prefix(ctx: &Ctx) -> &str {
    if ctx.prefix.is_empty() {
        "."
    } else {
        ctx.prefix.as_str()
    }
}

/// `buildMenuText` (`menu.js:174-303`) — teks menu polos.
fn build_menu_text(ctx: &Ctx) -> String {
    let pref = prefix(ctx);
    let (_, _, _, jam, menit, _) = jakarta_parts(now_ms());
    let push = if ctx.push_name.is_empty() {
        "User"
    } else {
        ctx.push_name.as_str()
    };
    let nomor = ctx.sender.split('@').next().unwrap_or(&ctx.sender);
    let bot = ctx.config();
    let nama_bot = if bot.bot.name.is_empty() {
        "Bot"
    } else {
        bot.bot.name.as_str()
    };
    let pengembang = if bot.bot.developer.is_empty() {
        "Owner"
    } else {
        bot.bot.developer.as_str()
    };
    let versi = if bot.bot.version.is_empty() {
        "1.2.0"
    } else {
        bot.bot.version.as_str()
    };
    let peran = if ctx.is_owner {
        "Owner"
    } else if ctx.is_premium {
        "Premium"
    } else {
        "User"
    };

    let mut t = format!("{}\n\n", sapa());
    t.push_str(&format!("      {push}\n\n"));
    t.push_str("Halo Kawan 🌱\n");
    t.push_str(&format!(
        "Perkenalkan aku {nama_bot}, buatan {pengembang} Saya siap membantu\n"
    ));
    t.push_str("kebutuhanmu, mulai dari download\n");
    t.push_str("video, main game, tanya jawab, nyari\n");
    t.push_str("info/sesuatu, bikin sticker, dan lain-lain.\n\n");

    t.push_str("    ᯓ INFO USER\n");
    t.push_str(&format!("╭   • Nama : {push}\n"));
    t.push_str(&format!("┆   • Status : {peran}\n"));
    t.push_str("┆   • Mode : PUBLIC\n");
    t.push_str(&format!("┆   • Nomor : {nomor}\n"));
    t.push_str(&format!("┆   • Waktu : {}:{} WIB\n", pad2(jam), pad2(menit)));
    t.push_str("╰➤------------------------------\n");

    t.push_str("    ᯓ INFO BOT\n");
    t.push_str(&format!("╭  • Name : {nama_bot}\n"));
    t.push_str(&format!("┆  • Author : {pengembang}\n"));
    t.push_str(&format!("┆  • Versi : {versi}\n"));
    t.push_str("┆  • Type script : Multi-Device\n");
    t.push_str(&format!("┆  • Uptime : {}\n", uptime(ctx.uptime_secs)));
    t.push_str("╰➤------------------------------\n");

    let daftar = kategori(ctx);
    if !daftar.is_empty() {
        t.push_str(" ✦ DAFTAR MENU ✦\n");
        for (i, (cat, _)) in daftar.iter().enumerate() {
            let awalan = if i == 0 { "╭   •" } else { "┆   •" };
            t.push_str(&format!(
                "{awalan} {pref}menucat {cat} {}\n",
                emoji(cat)
            ));
        }
        t.push_str("╰➤------------------------------\n");
    }
    t
}

/// `getVerifiedQuoted` (`menu.js:305-334`) — kutipan kontak palsu.
fn kutipan_kontak(ctx: &Ctx) -> CardQuote {
    let nomor = ctx.sender.split('@').next().unwrap_or(&ctx.sender);
    CardQuote::Contact {
        display_name: format!("🍂 Yth. {}", ctx.push_name),
        vcard: format!(
            "BEGIN:VCARD\nVERSION:3.0\nN:XL;ttname,;;;\nFN:ttname\nitem1.TEL;waid={nomor}:{nomor}\nitem1.X-ABLabel:Ponsel\nEND:VCARD"
        ),
    }
}

/// `safeThumbnail(getAssetBuffer(nama), 300)` (`asset-manager.js:79-99`).
fn thumbnail(ctx: &Ctx, nama: &str) -> Option<Vec<u8>> {
    let aset = ctx.config().assets.get(nama)?;
    let byte = ctx.berkas().baca(aset).ok()?;
    oktz_plugin_api::gambar::sharp(&byte)
        .ok()?
        .ubah_ukuran(300, 300)
        .ke_jpeg(80)
        .ok()
}

/// `weatherMenu(city)` (`menu.js:818-838`) — cuaca open-meteo.
///
/// Gagal jaringan tidak boleh menggagalkan command: hasilnya teks
/// `"Cuaca tidak tersedia"` persis seperti JS.
pub(crate) fn cuaca(ctx: &Ctx, kota: &str) -> String {
    let kode = |c: i64| -> &'static str {
        match c {
            0 => "☀️ Cerah",
            1 => "🌤️ Cerah Berawan",
            2 => "⛅ Berawan",
            3 => "☁️ Mendung",
            45 => "🌫️ Berkabut",
            48 => "🌫️ Kabut Tebal",
            51 => "🌦️ Gerimis",
            61 => "🌧️ Hujan Ringan",
            63 => "🌧️ Hujan",
            65 => "⛈️ Hujan Lebat",
            80 => "🌦️ Hujan Lokal",
            95 => "⛈️ Badai Petir",
            _ => "🌍 Tidak diketahui",
        }
    };
    let geo = ctx
        .http
        .get(&format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
            kota.replace(' ', "%20")
        ))
        .ok();
    let Some(geo) = geo else {
        return "Cuaca tidak tersedia".to_string();
    };
    let Some(geo) = geo.json() else {
        return "Cuaca tidak tersedia".to_string();
    };
    let Some(loc) = geo
        .get("results")
        .and_then(|r| r.get(0))
        .filter(|l| !l.is_null())
    else {
        return "Cuaca tidak tersedia".to_string();
    };
    let (lat, lon) = match (loc.get("latitude").and_then(|v| v.as_f64()), loc.get("longitude").and_then(|v| v.as_f64())) {
        (Some(a), Some(b)) => (a, b),
        _ => return "Cuaca tidak tersedia".to_string(),
    };
    let forecast = ctx
        .http
        .get(&format!(
            "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code"
        ))
        .ok();
    let Some(forecast) = forecast else {
        return "Cuaca tidak tersedia".to_string();
    };
    let Some(forecast) = forecast.json() else {
        return "Cuaca tidak tersedia".to_string();
    };
    let Some(current) = forecast.get("current") else {
        return "Cuaca tidak tersedia".to_string();
    };
    let suhu = current
        .get("temperature_2m")
        .and_then(|v| v.as_f64())
        .unwrap_or_default()
        .round() as i64;
    let code = current
        .get("weather_code")
        .and_then(|v| v.as_i64())
        .unwrap_or_default();
    let nama_lokasi = loc
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(kota);
    format!("{} | 🌡️ {suhu}°C\n📍 {nama_lokasi}", kode(code))
}

/// `messageParamsJson` varian 4/5/6 (`menu.js:756-770`).
fn params_bottom_sheet() -> serde_json::Value {
    json!({
        "bottom_sheet": {
            "in_thread_buttons_limit": 2,
            "divider_indices": [1, 2, 3, 4, 5, 999],
            "list_title": "Please select the menu",
            "button_title": "🍙 See Category",
        },
        "tap_target_configuration": {
            "title": " X ",
            "description": "bomboclard",
            "canonical_url": "https://example.com",
            "domain": "shop.example.com",
            "button_index": 0,
        },
    })
}

/// `limited_time_offer` + `bottom_sheet` + `tap_target_configuration`
/// (`menu.js:902-922`).
fn params_offer(sapaan: &str) -> serde_json::Value {
    json!({
        "limited_time_offer": {
            "text": sapaan,
            "url": "Hai",
            "expiration_time": now_ms() + 10_000,
        },
        "bottom_sheet": {
            "in_thread_buttons_limit": 2,
            "divider_indices": [1, 2, 3, 4, 5, 999],
            "list_title": "Please select the menu",
            "button_title": "🍙 See Category",
        },
        "tap_target_configuration": {
            "title": " X ",
            "description": "bomboclard",
            "canonical_url": "https://example.com",
            "domain": "shop.example.com",
            "button_index": 0,
        },
    })
}

/// `String.fromCharCode(8206).repeat(4001)` (`menu.js:465`).
fn readmore() -> String {
    "\u{200e}".repeat(4001)
}

// ---------------------------------------------------------------------------
// Varian tampilan
// ---------------------------------------------------------------------------

/// `menu.js:373-452` — kartu dengan dua `single_select`.
fn varian_1(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let push = if ctx.push_name.is_empty() {
        "User"
    } else {
        ctx.push_name.as_str()
    };
    let energy = ctx.user_field("energi").and_then(|v| v.as_i64()).unwrap_or(0);
    let koin = ctx.user_field("koin").and_then(|v| v.as_i64()).unwrap_or(0);
    let level = ctx.user_field("level").and_then(|v| v.as_i64()).unwrap_or(0);
    let exp = ctx.user_field("exp").and_then(|v| v.as_i64()).unwrap_or(0);
    let terdaftar = ctx
        .user_field("isRegistered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let member = if ctx.is_owner {
        "Bukan, tapi Owner"
    } else if ctx.is_premium {
        "Bukan, tapi Premium"
    } else {
        "Iyapp"
    };

    let footer = format!(
        "Hai @{push} 👋\n            \n🌿 Selamat datang di assistant {}\n\n\
╭┈┈⫹⫺ *INFORMASI BOT* ⫹⫺┈┈╮\n\
│ ◈ *Nama Bot* : *{}*\n\
│ ◈ *Versi* : *{}*  \n\
│ ◈ *Pengembang* : *{}*  \n\
│ ◈ *Pustaka*: `foto-baileys`\n\
╰┈┈┈┈┈┈┈┈\n\n\
╭┈┈⫹⫺ *INFORMASI PENGGUNA* ⫹⫺┈┈╮\n\
│ ◈ *Nama* : *{push}*\n\
│ ◈ *Member?* : *{member}*\n\
│ ◈ *Level* : *{level}*\n\
│ ◈ *Exp* : *{exp}* \n\
│ ◈ *Energi* : *{energy}*\n\
│ ◈ *Koin* : *{koin}*\n\
│ ◈ *Register* : *{}*\n\
│ ◈ *Energi* : *{energy}*\n\
╰┈┈┈┈┈┈┈┈\n\n\
Tekan tombol dibawah untuk info lebih lanjut dan untuk memilih kategori\n",
        bot.bot.name,
        bot.bot.name,
        bot.bot.version,
        bot.bot.developer,
        if terdaftar { "Sudah" } else { "Belum" },
    );

    let pref = prefix(ctx);
    let daftar = kategori_perintah(ctx);
    let baris = CardSection::new("Berikut adalah pilihan nya").rows(
        daftar
            .iter()
            .map(|(cat, cmds)| {
                CardRow::new(
                    format!("{} {}", emoji(cat), mono_upper_bold(cat)),
                    format!("{pref}menucat {cat}"),
                )
                .description(format!(
                    "Command ini memiliki  ({}) Perintah",
                    cmds.len()
                ))
            })
            .collect::<Vec<_>>(),
    );
    let lainnya = CardSection::new("Berikut adalah pilihan nya")
        .row(
            CardRow::new(
                "🍔 Lihat semua menu yang ada di bot ini",
                format!("{pref}sc"),
            )
            .description("Silahkan tap lalu kirim"),
        )
        .row(
            CardRow::new(
                "🥰 Kamu mau samaan kayak bot ini?",
                format!("{pref}sc"),
            )
            .description("Silahkan tap lalu kirim, nanti bot akan mengirimkan sebuah link download"),
        )
        .row(
            CardRow::new(
                "🌾 Siapakah owner dari bot ini?",
                format!("{pref}owner"),
            )
            .description("Tap dan klik kirim untuk melanjutkan"),
        );

    Reply::interactive("")
        .with_footer(footer)
        .with_header(CardHeader::new().image(aset_foto(ctx)))
        .with_button_card(InteractiveButton::select(
            &CardList::new("🍃 Menu Utama", [baris]).icon("DEFAULT"),
        ))
        .with_button_card(InteractiveButton::select(
            &CardList::new("Selengkapnya", [lainnya]).icon("REVIEW"),
        ))
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .quote(kutipan_kontak(ctx)),
        )
}

/// `menu.js:453-534` — kartu premium: daftar command per kategori.
fn varian_2(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let energy = ctx.user_field("energi").and_then(|v| v.as_i64()).unwrap_or(0);
    let koin = ctx.user_field("koin").and_then(|v| v.as_i64()).unwrap_or(0);
    let level = ctx.user_field("level").and_then(|v| v.as_i64()).unwrap_or(0);
    let exp = ctx.user_field("exp").and_then(|v| v.as_i64()).unwrap_or(0);
    let terdaftar = ctx
        .user_field("isRegistered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let peran = if ctx.is_owner {
        "🔥 Owner"
    } else if ctx.is_premium {
        "👑 Premium"
    } else {
        "😊 User"
    };

    let mut daftar = String::new();
    for (cat, cmds) in kategori_perintah(ctx) {
        daftar.push_str(&format!("╭─☰ {}\n", mono_upper_bold(cat)));
        for cmd in cmds {
            daftar.push_str(&format!("> {pref}{cmd}\n"));
        }
        daftar.push_str("╰─⬣\n\n");
    }

    let body = format!(
        "🥞 *Hello Brother*\n\nWelcome to {}, Our bot will help you\n\n\
🍅 *BOT INFORMATION*\n> 🤖 *Name*: {}\n> ⚙️ *Version*: {}\n> 👨‍💻 *Developer*: {}\n> 🧩 *Library*: `OnigiS`\n\n\
🍅 *USER INFORMATION*\n> 🧑 *Name*: {}\n> 🥐 *Role*: {peran}\n> 🧀 *Level*: {level}\n> 🍗 *Exp*: {exp}\n> 🍩 *Energi*: {energy}\n> 🎏 *Koin*: {koin}\n> 🍬 *Register*: {}\n\n{}{daftar}",
        bot.bot.name,
        bot.bot.name,
        bot.bot.version,
        bot.bot.developer,
        ctx.push_name,
        if terdaftar { "Sudah" } else { "Belum" },
        readmore(),
    );

    let owner = bot.owner.number.first().cloned().unwrap_or_default();
    Reply::interactive(body)
        .with_footer("Pilih tombol dibawah untuk info lebih lanjut")
        .with_header(CardHeader::new().image(aset_foto(ctx)))
        .with_button_card(InteractiveButton::url_merchant(
            "🍫 Owner Kami",
            format!("https://wa.me/{owner}"),
            format!("https://wa.me/{owner}"),
        ))
        .with_button_card(InteractiveButton::quick_reply(
            "🍛 Dapatkan Script ( Gratis )",
            format!("{pref}sc"),
        ))
        .with_context(CardContext::new().mention(ctx.sender.clone()))
}

/// `menu.js:536-674` — kartu list menu + cuaca (varian bawaan produksi).
fn varian_3(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let energy = ctx.user_field("energi").and_then(|v| v.as_i64()).unwrap_or(0);
    let koin = ctx.user_field("koin").and_then(|v| v.as_i64()).unwrap_or(0);
    let level = ctx.user_field("level").and_then(|v| v.as_i64()).unwrap_or(0);
    let exp = ctx.user_field("exp").and_then(|v| v.as_i64()).unwrap_or(0);
    let terdaftar = ctx
        .user_field("isRegistered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let peran = if ctx.is_owner {
        "🔥 Owner"
    } else if ctx.is_premium {
        "👑 Premium"
    } else {
        "😊 User"
    };

    let body = format!(
        "🥞 *Hello Brother* {}\n\nWelcome to {}, Our bot will help you\n\n\
🍅 *BOT INFORMATION*\n> 🤖 *Name*: {}\n> ⚙️ *Version*: {}\n> 👨‍💻 *Developer*: {}\n> 🧩 *Library*: `OnigiS`\n\n\
🍅 *USER INFORMATION*\n> 🧑 *Name*: {}\n> 🥐 *Role*: {peran}\n> 🧀 *Level*: {level}\n> 🍗 *Exp*: {exp}\n> 🍩 *Energi*: {energy}\n> 🎏 *Koin*: {koin}\n> 🍬 *Register*: {}",
        sapa(),
        bot.bot.name,
        bot.bot.name,
        bot.bot.version,
        bot.bot.developer,
        ctx.push_name,
        if terdaftar { "Sudah" } else { "Belum" },
    );

    let daftar = kategori_perintah(ctx);
    let baris = CardSection::new(format!(
        "Berikut adalah pilihan nya ({} kategori)",
        daftar.len()
    ))
    .rows(
        daftar
            .iter()
            .map(|(cat, cmds)| {
                CardRow::new(
                    format!("{} {cat}", emoji(cat)),
                    format!("{pref}menucat {cat}"),
                )
                .description(format!("{} command", cmds.len()))
            })
            .collect::<Vec<_>>(),
    );

    let cuaca = cuaca(ctx, "Merangin");
    Reply::interactive(body)
        .with_footer("🍔 Silahkan pilih dari salah satu tombol di bawah")
        .with_header(CardHeader::new().subtitle(bot.bot.name.clone()).image(aset_foto(ctx)))
        .with_button_card(InteractiveButton::select(
            &CardList::new("🍃 List Menu", [baris]).icon("DEFAULT"),
        ))
        .with_button_card(InteractiveButton::quick_reply("🧀 Owner", format!("{pref}owner")))
        .with_button_card(InteractiveButton::quick_reply(
            "💐 Allmenu",
            format!("{pref}allmenu"),
        ))
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .quote(CardQuote::Location {
                    latitude: 0.0,
                    longitude: 0.0,
                    name: cuaca,
                    thumbnail: thumbnail(ctx, "foto"),
                }),
        )
}

/// `menu.js:676-790` — kartu video dengan satu `single_select` per kategori.
fn varian_4(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let body = format!(
        "*{} {}, 𝘔𝘺 𝘯𝘢𝘮𝘦 𝘪𝘴 {}.\n\n\
  🏔 𝘐 𝘊𝘢𝘯 𝘩𝘦𝘳𝘱 𝘺𝘰𝘶 𝘸𝘪𝘵𝘩 𝘴𝘦𝘷𝘦𝘳𝘢𝘯 𝘵𝘩𝘪𝘯𝘨𝘴 𝘸𝘪𝘵𝘪𝘯 𝘞𝘩𝘢𝘵𝘈𝘱𝘱. 𝘈𝘯𝘥 𝘐 𝘢𝘮 𝘈𝘳𝘮𝘦𝘥 𝘢 *𝘑𝘢𝘷𝘢𝘴𝘤𝘳𝘪𝘱𝘵* 𝘗𝘳𝘰𝘨𝘳𝘢𝘮 𝘈𝘴𝘴𝘦𝘮𝘣𝘭𝘦𝘥 𝘣𝘺 𝘮𝘺 𝘤𝘳𝘦𝘢𝘵𝘰𝘳.\n\n\
`INFORMATION BOT`\n> 🍛 *Creator*: {}\n> 🥞 *Name*: {}\n> 🥩 *Version*: {}\n> 🍂 *Type*: `Plugin x Cases`\n> 🦴 *Mode*: *🍕 Unlock for everyone*\n\n\
Enjoy your use brother.",
        sapa(),
        ctx.push_name,
        bot.bot.name,
        bot.bot.developer,
        bot.bot.name,
        bot.bot.version,
    );

    let daftar = kategori_perintah(ctx);
    let mut tombol = vec![InteractiveButton::url_merchant(
        "🧀 Visit My Creator",
        format!("https://wa.me/{}", bot.owner.number.first().cloned().unwrap_or_default()),
        format!("https://wa.me/{}", bot.owner.number.first().cloned().unwrap_or_default()),
    )];
    for (cat, cmds) in &daftar {
        let baris = CardSection::new("Please select the command")
            .highlight_label(bot.bot.name.clone())
            .rows(
                cmds.iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        CardRow::new(
                            format!("{} {cmd}", i + 1),
                            format!("{pref}{cmd}"),
                        )
                        .description("Select this command?")
                    })
                    .collect::<Vec<_>>(),
            );
        tombol.push(InteractiveButton::select(
            &CardList::new(format!("{} {cat} Menu", emoji(cat)), [baris]).icon("REVIEW"),
        ));
    }

    Reply::interactive(body)
        .with_footer("Please select the button in below")
        .with_header(CardHeader::new().video(aset_mp4(ctx)))
        .with_message_params(params_bottom_sheet())
        .with_buttons(tombol)
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .forwarded(9)
                .newsletter(bot.saluran.id.clone(), bot.saluran.name.clone())
                .quote(CardQuote::Video {
                    caption: bot.bot.name.clone(),
                    seconds: 999_999_999,
                    mimetype: "video/mp4".to_string(),
                    thumbnail: thumbnail(ctx, "foto"),
                    file_length: 9_999_999,
                }),
        )
}

/// `menu.js:792-951` — kartu video dengan badge `limited_time_offer`.
fn varian_5(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let status = if ctx.is_premium {
        "💎 Premium"
    } else if ctx.is_owner {
        "👑 Owner"
    } else {
        "🏷️ Free"
    };
    let body = format!(
        "🍟 Hai *{}* \n                   \n\
_i am an automated system (WhatsApp bot) that can help to do something search and get data / information only through WhatsApp._\n\n\
*`乂 I N F O - B O T`*\n┌ ◦ Name : {}\n│ ◦ Author : @{}\n│ ◦ Type Script : Case x Plugins\n│ ◦ Uptime : {}\n└ ◦ Versi : {}\n\n\
*`乂 I N F O - U S E R`*\n┌ ◦ Nama : {}\n│ ◦ Status : {status}\n│ ◦ Mode : Bisa digunakan semua\n│ ◦ Nomor : @{}\n└ ◦ {}\n",
        ctx.push_name,
        bot.bot.name,
        bot.bot.developer,
        runtime(ctx.uptime_secs),
        bot.bot.version,
        ctx.push_name,
        ctx.sender.split('@').next().unwrap_or(&ctx.sender),
        sapa(),
    );

    let cuaca = cuaca(ctx, "Jakarta");
    Reply::interactive(body)
        .with_footer("Please select the button in below")
        .with_header(CardHeader::new().video(aset_mp4(ctx)))
        .with_message_params(params_offer(sapa()))
        .with_button_card(InteractiveButton::empty(""))
        .with_button_card(InteractiveButton::quick_reply(
            "𐔌 Allmenu  𐦯",
            format!("{pref}allmenu"),
        ))
        .with_button_card(InteractiveButton::quick_reply(
            "𐔌 Peraturan  𐦯",
            format!("{pref}rules"),
        ))
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .forwarded(9)
                .newsletter(bot.saluran.id.clone(), bot.saluran.name.clone())
                .quote(CardQuote::Location {
                    latitude: 0.0,
                    longitude: 0.0,
                    name: cuaca,
                    thumbnail: thumbnail(ctx, "foto"),
                }),
        )
}

/// `menu.js:952-1080` — kartu dengan header lokasi.
fn varian_6(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let cuaca = cuaca(ctx, "Jakarta");
    Reply::interactive(build_menu_text(ctx))
        .with_header(CardHeader::new().location(CardLocation {
            latitude: 0.0,
            longitude: 0.0,
            name: bot.bot.name.clone(),
            address: Some(cuaca),
            thumbnail: thumbnail(ctx, "foto"),
        }))
        .with_message_params(params_offer(sapa()))
        .with_button_card(InteractiveButton::empty(""))
        .with_button_card(InteractiveButton::quick_reply(
            "𐔌 Allmenu  𐦯",
            format!("{pref}allmenu"),
        ))
        .with_button_card(InteractiveButton::quick_reply(
            "𐔌 Peraturan  𐦯",
            format!("{pref}rules"),
        ))
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .forwarded(9)
                .quote_trigger(),
        )
}

/// `menu.js:1081-1232` — kartu inggris + video kedua.
fn varian_7(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let limit = ctx
        .user_field("limit")
        .and_then(|v| v.as_i64())
        .map(|l| l.to_string())
        .unwrap_or_else(|| "unlimited".to_string());
    let status = if ctx.is_premium {
        "premium"
    } else {
        "free"
    };
    let role = if ctx.is_owner { "owner" } else { "user" };

    let mut body = format!(
        "Hey, *{}*! ✨\n\nI'm {}, your intelligent assistant powered by {}. Whether you need information, entertainment, or utilities, I'm always ready to help.\n\n",
        ctx.push_name, bot.bot.name, bot.bot.developer
    );
    body.push_str(&format!(
        "╭╮ `✯ {}`\n││  name : {}\n││  mode : public\n││  prefix : `{pref}`\n││  runtime : {}\n╰╯\n",
        mono_upper_bold("BOT DETAIL"),
        bot.bot.name,
        runtime_pendek(ctx.uptime_secs)
    ));
    body.push_str(&format!(
        "╭╮ `❀ {}`\n││  name : _{}_\n││  status : {status}\n││  role : {role}\n││  energi : {limit}\n╰╯\n",
        mono_upper_bold("USER DETAIL"),
        ctx.push_name
    ));
    body.push_str(&readmore());
    body.push_str(&format!(
        "╭╮ `✧ {}`\n",
        mono_upper_bold("MENU CATEGORY")
    ));
    for (cat, cmds) in kategori_perintah(ctx) {
        body.push_str(&format!("││  ▸ {} : {} fitur\n", cat.to_lowercase(), cmds.len()));
    }
    body.push_str("╰╯");

    let owner = bot.owner.number.first().cloned().unwrap_or_default();
    let kartu = Reply::interactive(body)
        .with_footer(format!(
            "Use: {pref}menucat <category>\n\nIf you have any questions, please contact the owner."
        ))
        .with_header(CardHeader::new().image(aset_foto(ctx)))
        .with_button_card(InteractiveButton::url_merchant(
            "🥐 Contact Owner",
            format!("http://wa.me/{owner}"),
            format!("http://wa.me/{owner}"),
        ))
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .forwarded(99)
                .quote(CardQuote::Sticker {
                    source: stiker_fake(ctx).unwrap_or_else(|| aset_foto2(ctx)),
                }),
        );
    // `menu.js:1215-1229` — video kedua yang menempel pada kartu lewat
    // `messageAssociation` (`associationType: 12`).
    Reply::sequence(vec![
        kartu,
        Reply::video(aset_mp4(ctx), ""),
    ])
}

/// `menu.js:1233-1375` — pratinjau tautan (bukan kartu).
fn varian_8(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let level = ctx.user_field("level").and_then(|v| v.as_i64()).unwrap_or(1);
    let rank = match level {
        10..=19 => "⭐ Elite",
        20..=39 => "🎖️ Master",
        40..=59 => "💪 Grandmaster",
        60..=79 => "💜 Epic",
        80..=99 => "⚔️ Legend",
        _ => "🛡️ Warrior",
    };
    let jabatan = if ctx.is_owner {
        "[ Owner ]"
    } else if ctx.is_premium {
        "[ Premium ]"
    } else {
        "[ User ]"
    };
    let koin = ctx.user_field("koin").and_then(|v| v.as_i64()).unwrap_or(0);
    let exp = ctx.user_field("exp").and_then(|v| v.as_i64()).unwrap_or(0);
    let hari = ctx.user_field("activeDays").and_then(|v| v.as_i64()).unwrap_or(0);

    let mut body = format!(
        "*YOUR STATUS*\n❑ Role: {jabatan}\n❑ Rank: {rank}\n❑ Level: {level}\n❑ Coin: {koin}\n❑ Exp: {exp}\n❑ Energy: unlimited\n\n\
Hello, my friend *\"{}\"*!\nHow are you today? You're feeling well, right?\n\nYou've been online for *{hari} days*\n\n",
        ctx.push_name
    );
    body.push_str(&readmore());
    let peta = [
        ("info", "INFO BOT"),
        ("jadibot", "JADI BOT"),
        ("economy", "EKONOMI"),
        ("main", "MAIN MENU"),
        ("owner", "OWNER MENU"),
    ];
    for (cat, cmds) in kategori_perintah(ctx) {
        let nama = peta
            .iter()
            .find(|(k, _)| *k == cat.to_lowercase())
            .map_or_else(|| cat.to_uppercase(), |(_, v)| (*v).to_string());
        body.push_str(&format!("*{nama}*\n"));
        for cmd in cmds {
            body.push_str(&format!("⇨ {pref}{cmd}\n"));
        }
        body.push('\n');
    }
    let body = body.trim_end().to_string();

    Reply::text(format!("{} {body}", bot.info.website))
}

/// Aset `foto` — `getAssetBuffer("foto")`.
fn aset_foto(ctx: &Ctx) -> MediaSource {
    MediaSource::path(
        ctx.config()
            .assets
            .get("foto")
            .unwrap_or("./assets/image/foto.jpeg"),
    )
}

/// Aset `foto2` — `getAssetBuffer("foto2")`.
fn aset_foto2(ctx: &Ctx) -> MediaSource {
    MediaSource::path(
        ctx.config()
            .assets
            .get("foto2")
            .unwrap_or("./assets/image/foto2.jpeg"),
    )
}

/// Aset `mp4` — `config.assets["mp4"]`.
fn aset_mp4(ctx: &Ctx) -> MediaSource {
    MediaSource::path(
        ctx.config()
            .assets
            .get("mp4")
            .unwrap_or("./assets/video/mp4.mp4"),
    )
}

/// `db.setting("menuVariant") || config.ui?.menuVariant || 2` (`menu.js:337-338`).
fn varian_aktif(ctx: &Ctx) -> i64 {
    ctx.setting_i64(SETTING_VARIANT)
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_VARIANT)
}

/// Varian 16 — menu utama persis `Rubby-MD/plugins/main/menu.js` (pilih
/// lewat `.setmenu v16`): header smallcaps + status, body "{bot} in here",
/// menu di footer, dan tombol `single_select` kategori + quick Owner.
///
/// Rubby tidak menyaring kategori berdasarkan permission — semua kategori di
/// registry tampil (termasuk `owner`). Command per kategori diurut
/// alfabetis seperti `formatSection` JS.
fn varian_rubby(ctx: &Ctx) -> Reply {
    let bot = ctx.config();
    let pref = prefix(ctx);
    let sub = ctx.args.first().map(|s| s.to_lowercase()).unwrap_or_default();

    // smallcaps (`toSmallFont` JS) + emoji kategori (`categoryEmojis`).
    fn sc(teks: &str) -> String {
        const PETA: &str = "ᴬᴮᶜᴰᴱᶠᴳᴴᴵᴶᴷᴸᴹᴺᴼᴾꞯᴿꜱᴛᴜᴠᴡxʏᴢ";
        teks.chars().map(|c| {
            if c.is_ascii_alphabetic() {
                let u = c.to_ascii_uppercase();
                PETA.chars()
                    .nth((u as u8 - b'A') as usize)
                    .unwrap_or(c)
            } else {
                c
            }
        }).collect()
    }

    fn emoji(kategori: &str) -> &'static str {
        match kategori.to_lowercase().as_str() {
            "ai" => "🤖",
            "ai-image" => "🪄",
            "asupan" => "💃",
            "audio" => "🎧",
            "cewerandom" => "👧",
            "convert" => "🔄",
            "downloader" => "📥",
            "download" => "📥",
            "ephoto" => "📸",
            "example" => "💡",
            "fun" => "🎲",
            "github" => "🐙",
            "group" | "grup" => "👥",
            "info" => "ℹ️",
            "islamic" => "🌙",
            "main" => "🎀",
            "maker" => "🎨",
            "manga" => "📚",
            "news" => "📰",
            "novel" => "📖",
            "nsfw" => "🔞",
            "other" => "✨",
            "owner" => "👑",
            "panel" => "⚡",
            "premium" => "💎",
            "primbon" => "🔮",
            "rpg" => "⚔️",
            "search" => "🔍",
            "stalk" | "stalker" => "🕵️",
            "streaming" => "🎬",
            "tools" => "🔧",
            "webapi" => "🌐",
            _ => "🌸",
        }
    }

    // (kategori, daftar perintah aktif serupanya) — alfabetis, tanpa filter
    // permission (paritas Rubby). Uniq + sort per `formatSection` JS.
    let registry = |c: &Ctx| -> Vec<(String, Vec<String>)> {
        let mut hitung: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for p in oktz_plugin_api::all_plugins() {
            if !p.meta.is_enabled {
                continue;
            }
            let cat = p.meta.category.clone();
            if cat.is_empty() {
                continue;
            }
            hitung.entry(cat).or_default().push(p.meta.name.clone());
        }
        let mut hasil: Vec<(String, Vec<String>)> = hitung
            .into_iter()
            .map(|(cat, mut cmds)| {
                cmds.sort();
                cmds.dedup();
                (cat, cmds)
            })
            .collect();
        let _ = c; // konteks argumen dipakai pemanggil; registry murni.
        hasil.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        hasil
    };

    let cats = registry(ctx);
    let total: usize = cats.iter().map(|(_, c)| c.len()).sum();

    // Sapaan per jam (`getGreeting`), huruf smallcaps.
    let greeting: String = match jakarta_parts(now_ms()).3 {
        4..=10 => sc("Selamat Pagi") + " 🌄",
        11..=14 => sc("Selamat Siang") + " ☀️",
        15..=17 => sc("Selamat Sore") + " 🌇",
        _ => sc("Selamat Malam") + " 🌙",
    };

    // Pengirim: @nomor kecuali JID LID (pakai pushName).
    let display: String = if ctx.sender.ends_with("@lid") {
        if ctx.push_name.is_empty() {
            "User".to_string()
        } else {
            ctx.push_name.clone()
        }
    } else {
        let num: String = ctx.sender.chars().filter(char::is_ascii_digit).collect();
        if num.is_empty() {
            if ctx.push_name.is_empty() {
                "User".to_string()
            } else {
                ctx.push_name.clone()
            }
        } else {
            format!("@{num}")
        }
    };

    let status = if ctx.is_owner {
        sc("Owner")
    } else if ctx.is_premium {
        sc("Premium")
    } else {
        sc("Free User")
    };
    let nama_bot = bot.bot.name.clone();
    let galat_uptime = format!(
        "• {} : *{}*\n• {} : {}\n• {} : {}\n• {} : {}\n• {} : {}",
        sc("Status"),
        status,
        sc("Uptime"),
        uptime(ctx.uptime_secs),
        sc("Server Uptime"),
        server_uptime(ctx.uptime_secs),
        sc("Mode"),
        sc("Public"),
        sc("Prefix"),
        pref
    );

    let section = |cat: &str, cmds: &[String]| {
        let mut s = format!("{} *{}*\n", emoji(cat), sc(&cat.to_uppercase()));
        for cmd in cmds {
            s.push_str(&format!("• {pref}{}\n", sc(cmd)));
        }
        s.trim_end().to_string()
    };

    let menu_kategori: Vec<(String, Vec<String>)> = if sub == "all" {
        cats.clone()
    } else if let Some((c, cmds)) = cats.iter().find(|(c, _)| c.eq_ignore_ascii_case(&sub)) {
        vec![(c.clone(), cmds.clone())]
    } else {
        Vec::new()
    };

    let footer = if sub == "all" {
        let mut teks = format!(
            "{greeting}, {display} 👋\n{}\n\n{galat_uptime}\n\n",
            sc("Ini semua fitur")
        );
        for (cat, cmds) in &menu_kategori {
            teks.push_str(&section(cat, cmds));
            teks.push_str("\n\n");
        }
        teks.push_str(&format!("{}: {total} {}", sc("Total"), sc("Commands")));
        teks
    } else if let Some((cat, cmds)) = menu_kategori.first() {
        format!(
            "{greeting}, {display} 👋\n\n{galat_uptime}\n\n{}",
            section(cat, cmds)
        )
    } else {
        format!(
            "{greeting}, {display} 👋\n{} *{}*!\n\n{galat_uptime}\n\n{}:\n{pref}menu <{}>\n{pref}menu all",
            sc("Selamat Datang di"),
            sc(&nama_bot),
            sc("Pilih kategori di tombol bawah, atau ketik"),
            sc("kategori")
        )
    };

    // Baris `single_select` — sama dengan JS (`categoryRows`).
    let mut rows: Vec<CardRow> = vec![CardRow::new(
        format!("📑 {}", sc("semua menu")),
        format!("{pref}menu all"),
    )
    .description(sc("tampilkan semua perintah bot"))];
    for (cat, cmds) in &cats {
        let _ = cmds;
        rows.push(
            CardRow::new(
                format!("{} {}", emoji(cat), sc(&format!("menu {}", cat.to_lowercase()))),
                format!("{pref}menu {}", cat.to_lowercase()),
            )
            .description(sc(&format!("daftar perintah {}", cat.to_lowercase()))),
        );
    }

    let list = CardList::new(
        sc("daftar menu"),
        vec![CardSection::new(sc("kategori menu")).rows(rows)],
    )
    .icon("DEFAULT");

    let header = CardHeader::new()
        .title(nama_bot.clone())
        .subtitle(bot.info.website.clone());
    let mut reply = Reply::interactive(format!("{nama_bot} in here"))
        .with_footer(footer)
        .with_button_card(InteractiveButton::select(&list))
        .with_button_card(InteractiveButton::quick_reply(
            format!("👑 {}", sc("Owner")),
            format!("{pref}owner"),
        ))
        .with_context(
            CardContext::new()
                .mention(ctx.sender.clone())
                .quote_trigger(),
        );
    if let Some(aset) = ctx.config().assets.get("foto")
        && ctx.berkas().ada(aset)
    {
        reply = reply.with_header_card(header.image(MediaSource::path(aset)));
    } else {
        reply = reply.with_header(header);
    }
    reply
}

/// Server uptime dari `/proc/uptime` (Linux); jatuh ke uptime bot.
fn server_uptime(bot_uptime: u64) -> String {
    if let Ok(isi) = std::fs::read_to_string("/proc/uptime")
        && let Some(pertama) = isi.split_whitespace().next()
        && let Ok(v) = pertama.parse::<f64>()
    {
        return uptime(v as u64);
    }
    uptime(bot_uptime)
}

#[plugin(
    name = "menu",
    alias = ["help", "bantuan", "commands", "m"],
    category = "main",
    description = "Menampilkan menu utama bot",
    usage = ".menu",
    example = ".menu",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false
)]
pub fn menu(ctx: Ctx) -> Reply {
    match varian_aktif(&ctx) {
        1 => varian_1(&ctx),
        2 => varian_2(&ctx),
        3 => varian_3(&ctx),
        4 => varian_4(&ctx),
        5 => varian_5(&ctx),
        6 => varian_6(&ctx),
        7 => varian_7(&ctx),
        8 => varian_8(&ctx),
        16 => varian_rubby(&ctx),
        _ => Reply::text(build_menu_text(&ctx)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::config::Runtime;

    fn ctx() -> Ctx {
        Ctx {
            prefix: ".".into(),
            push_name: "Oktz".into(),
            sender: "628111@s.whatsapp.net".into(),
            uptime_secs: 90_061,
            config: std::sync::Arc::new(Runtime {
                bot: oktz_plugin_api::config::BotInfo {
                    name: "Oktz Bot".into(),
                    version: "3.3.1".into(),
                    developer: "Zann".into(),
                    website: String::new(),
                },
                info: oktz_plugin_api::config::Info {
                    website: "https://oktzz.xyz".into(),
                    grupwa: String::new(),
                },
                ..Runtime::default()
            }),
            ..Ctx::default()
        }
    }

    fn teks(r: Reply) -> String {
        r.text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn memuat_info_user_bot_dan_daftar_kategori() {
        let t = build_menu_text(&ctx());
        assert!(t.contains("Oktz Bot"), "{t}");
        assert!(t.contains("Zann"), "{t}");
        assert!(t.contains("Nama : Oktz"), "{t}");
        assert!(t.contains("Nomor : 628111"), "{t}");
        assert!(t.contains("Uptime : 1d 1h 1m 1s"), "{t}");
        assert!(t.contains(".menucat main 🏠"), "{t}");
        assert!(!t.contains(".menucat owner"), "owner bukan owner: {t}");
    }

    #[test]
    fn owner_melihat_kategori_owner() {
        let mut c = ctx();
        c.is_owner = true;
        let t = build_menu_text(&c);
        assert!(t.contains(".menucat owner 👑"), "{t}");
        assert!(t.contains("Status : Owner"), "{t}");
    }

    #[test]
    fn kategori_md_menyembunyikan_panel_pushkontak_store() {
        let semua: Vec<_> = kategori(&ctx()).into_iter().map(|(c, _)| c).collect();
        for cat in ["panel", "pushkontak", "store"] {
            assert!(!semua.contains(&cat), "{cat} harus disembunyikan: {semua:?}");
        }
    }

    #[test]
    fn semua_kategori_terdaftar_punya_emoji() {
        let semua = kategori(&ctx());
        assert!(semua.iter().all(|(_, n)| *n > 0));
    }

    #[test]
    fn mono_upper_bold_memetakan_huruf_kapital() {
        assert_eq!(mono_upper_bold("main"), "𝗠𝗔𝗜𝗡");
        assert_eq!(mono_upper_bold("tools"), "𝗧𝗢𝗢𝗟𝗦");
    }

    #[test]
    fn varian_bawaan_adalah_tiga() {
        // `config.ui.menuVariant` = 3 (`config.js:189-191`).
        assert_eq!(varian_aktif(&ctx()), 3);
        let mut c = ctx();
        c.settings = json!({ "menuVariant": 5 });
        assert_eq!(varian_aktif(&c), 5);
    }

    #[test]
    fn varian_tiga_menghasilkan_kartu_dengan_tombol() {
        let r = menu(ctx());
        let Reply::Interactive { kartu } = r.without_mutations()
        else {
            panic!("harus kartu interaktif");
        };
        let body = &kartu.body;
        let footer = &kartu.footer;
        let buttons = &kartu.buttons;
        assert!(body.contains("Hello Brother"), "{body}");
        assert_eq!(
            footer.as_deref(),
            Some("🍔 Silahkan pilih dari salah satu tombol di bawah")
        );
        assert_eq!(buttons.len(), 3, "list + owner + allmenu");
        assert_eq!(buttons[0].name, "single_select");
        assert_eq!(buttons[1].name, "quick_reply");
        assert_eq!(buttons[2].name, "quick_reply");
        assert_eq!(buttons[1].params["id"], ".owner");
        assert_eq!(buttons[2].params["id"], ".allmenu");
    }

    #[test]
    fn varian_satu_menghasilkan_kartu_dengan_dua_daftar() {
        let mut c = ctx();
        c.settings = json!({ "menuVariant": 1 });
        let r = menu(c);
        let Reply::Interactive { kartu } = r.without_mutations()
        else {
            panic!("harus kartu interaktif");
        };
        let body = &kartu.body;
        let footer = &kartu.footer;
        let buttons = &kartu.buttons;
        assert!(body.is_empty());
        assert!(footer.as_deref().unwrap_or_default().contains("INFORMASI BOT"));
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0].name, "single_select");
        assert_eq!(buttons[1].name, "single_select");
    }

    #[test]
    fn varian_delapan_menghasilkan_teks_dengan_alamat_web() {
        let mut c = ctx();
        c.settings = json!({ "menuVariant": 8 });
        let t = teks(menu(c));
        assert!(t.contains("YOUR STATUS"), "{t}");
        assert!(t.contains("https://oktzz.xyz"), "{t}");
    }

    #[test]
    fn varian_rubby_memakai_footer_dan_tombol_kategori() {
        let mut c = ctx();
        c.settings = json!({ "menuVariant": 16 });
        c.args = vec![];
        let r = menu(c);
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("varian 16 harus kartu interaktif");
        };
        // Body = "{bot} in here" (Rubby `setBody`), isi menu di footer.
        assert_eq!(kartu.body, "Oktz Bot in here");
        let footer = kartu.footer.as_deref().unwrap_or_default();
        assert!(footer.contains("ꜱᴇʟᴀᴍᴀᴛ"), "{footer}"); // smallcaps greeting
        assert!(footer.contains("ᴏᴋᴛᴢ"), "{footer}"); // smallcaps "Oktz"
        assert!(footer.contains(".menu"), "{footer}");
        // Tombol: single_select kategori + quick Owner.
        assert_eq!(kartu.buttons.len(), 2);
        assert_eq!(kartu.buttons[0].name, "single_select");
        assert_eq!(kartu.buttons[1].name, "quick_reply");
        // Baris pertama = semua menu (`.menu all`), berikutnya per kategori.
        let rows = kartu.buttons[0]
            .params
            .nilai()["sections"][0]["rows"]
            .as_array()
            .unwrap();
        assert_eq!(rows[0]["id"], ".menu all");
        assert!(rows.iter().any(|r| r["id"] == ".menu main"));
        // Header memuat nama bot.
        let header = kartu.header.as_ref().expect("varian rubby harus punya header");
        assert_eq!(header.title.as_deref(), Some("Oktz Bot"));
        assert_eq!(header.subtitle.as_deref(), Some("https://oktzz.xyz"));
    }

    #[test]
    fn varian_rubby_sub_kategori_memuat_seksi_itu() {
        let mut c = ctx();
        c.settings = json!({ "menuVariant": 16 });
        c.args = vec!["tools".to_string()];
        let r = menu(c);
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("varian 16 harus kartu interaktif");
        };
        let footer = kartu.footer.as_deref().unwrap_or_default();
        assert!(footer.contains("ꜱᴇʟᴀᴍᴀᴛ"), "{footer}");
        // Tanggal Rubby memakai emoji sendiri, bukan map menu.js oktz.
        assert!(footer.contains("🔧"), "{footer}");
    }

    #[test]
    fn varian_rubby_sub_all_memuat_total() {
        let mut c = ctx();
        c.settings = json!({ "menuVariant": 16 });
        c.args = vec!["all".to_string()];
        let r = menu(c);
        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("varian 16 harus kartu interaktif");
        };
        let footer = kartu.footer.as_deref().unwrap_or_default();
        assert!(footer.contains("ᴛᴏᴛᴀʟ"), "{footer}");
        assert!(footer.contains("ᴄᴏᴍᴍᴀɴᴅs"), "{footer}");
    }
}
