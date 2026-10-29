//! `plugins/main/menucat.js` — daftar command per kategori.
//!
//! Port penuh: teksnya (bracket box + small caps + simbol permission),
//! legendanya, urutan kategorinya, dan empat varian tampilan yang dipilih
//! lewat setting `menucatVariant`.
//!
//! Yang diport apa adanya:
//! - `toSmallCaps` (`menucat.js:62-96`) — tabel huruf small-caps unicode.
//! - `createBracketBox` (`menucat.js:98-105`).
//! - `getCommandSymbols` (`menucat.js:107-118`) — urutan `Ⓞ ⓟ Ⓛ Ⓐ Ⓖ Ⓟ`.
//!   `Ⓛ` (limit) tidak bisa muncul: `PluginMeta` Rust tidak punya field
//!   `limit` (`lib/oktz-plugin-api/meta.rs`), jadi hanya lima simbol lain
//!   yang bisa ditulis — sisanya tetap urut dan sama.
//! - `CATEGORY_EMOJIS` (`menucat.js:31-60`) dan `categoryOrder`
//!   (`menucat.js:155-181`).
//! - Legend `KETERANGAN` (`menucat.js:203-210`).
//! - Varian 1 (teks polos), 2 (kartu + header gambar),
//!   5 (kartu native-flow + header video) dan 6 (kartu + header lokasi).
//!   (`menucat.js:227-361` dan `:408-613`).

//!
//! `ponytail:` yang berbeda dari JS, beserta alasannya:
//! - "case" dari `case/foto.js` tidak ada di registry Rust, jadi hanya
//!   plugin yang bisa dihitung. Angka `(X plugin + Y case)`
//!   (`menucat.js:403-405`) karena itu tidak pernah ditulis — persis seperti
//!   JS kalau `casesByCategory` kosong.
//! - Urutan command di dalam satu kategori: JS memakai urutan registrasi
//!   plugin, Rust memakai urutan alfabetis. `all_plugins()` tidak menjamin
//!   urutan registrasi, dan hasil yang bisa diulang lebih berguna.
//! - Cuaca gagal diambil (open-meteo tidak bisa dihubungi) → teks
//!   `"Cuaca tidak tersedia"` (`menucat.js:307-309`) dan kartu tetap
//!   terkirim, sama seperti JS. Command tidak pernah gagal karena jaringan.
//! - Varian 6 di JS hanya terpasang di cabang "pakai kategori"; di sini
//!   kedua cabangnya memakai kartu yang sama, supaya satu setting
//!   `menucatVariant` berperilaku sama di mana pun.
//!
//! Fallback ke balasan teks polos mengikuti `try/catch` JS
//! (`menucat.js:362-364`): kalau aset header tidak ada, kartu tidak bisa
//! dibangun lalu yang dikirim teksnya.

use oktz_plugin_api::registry::{
    CardContext, CardHeader, CardLocation, CardQuote, InteractiveButton, MediaSource, PluginEntry,
};
use oktz_plugin_api::{Ctx, PluginMeta, Reply, all_plugins, plugin};

// ---------------------------------------------------------------------------
// Konstanta dari JS
// ---------------------------------------------------------------------------

/// `CATEGORY_EMOJIS` (`menucat.js:31-60`).
const CATEGORY_EMOJIS: &[(&str, &str)] = &[
    ("owner", "👑"),
    ("main", "🏠"),
    ("utility", "🔧"),
    ("fun", "🎮"),
    ("group", "👥"),
    ("download", "📥"),
    ("search", "🔍"),
    ("tools", "🛠️"),
    ("sticker", "🖼️"),
    ("ai", "🤖"),
    ("game", "🎯"),
    ("media", "🎬"),
    ("info", "ℹ️"),
    ("religi", "☪️"),
    ("panel", "🖥️"),
    ("user", "📊"),
    ("jpm", "📢"),
    ("pushkontak", "📱"),
    ("ephoto", "🎨"),
    ("store", "🛒"),
    ("linode", "☁️"),
    ("random", "🎲"),
    ("canvas", "🎨"),
    ("vps", "🌊"),
    ("premium", "💎"),
    ("convert", "🔄"),
    ("economy", "💰"),
    ("cek", "📋"),
];

/// `categoryOrder` (`menucat.js:155-181`) — urutan kategori di daftar menu.
/// Kategori di luar daftar ini dapat indeks 999, sama seperti
/// `indexOf(...) === -1 ? 999` di JS.
const URUTAN_KATEGORI: &[&str] = &[
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
    "jpm",
    "pushkontak",
    "panel",
    "ephoto",
    "store",
];

/// `modeExcludeMap` — dibangun dari `MODES` `plugins/group/botmode.js:19-56`
/// oleh `menucat.js:143-153`. Hanya mode `md` punya `excludeCategories`; mode
/// lain bernilai `null`, jadi `modeExcludeMap[botMode] || modeExcludeMap.md`
/// (`menucat.js:153`) selalu jatuh ke daftar `md`.
const MODE_EXCLUDE: &[(&str, Option<&[&str]>)] = &[
    ("md", Some(&["cpanel", "pushkontak", "store"])),
    ("all", None),
    ("cpanel", None),
    ("pushkontak", None),
    ("store", None),
    ("otp", None),
];

/// `weatherCode` (`menucat.js:294-296`) — kode cuaca open-meteo ke label.
const CUACA_KODE: &[(i64, &str)] = &[
    (0, "☀️ Cerah"),
    (1, "🌤️ Cerah Berawan"),
    (2, "⛅ Berawan"),
    (3, "☁️ Mendung"),
    (45, "🌫️ Berkabut"),
    (48, "🌫️ Kabut Tebal"),
    (51, "🌦️ Gerimis"),
    (61, "🌧️ Hujan Ringan"),
    (63, "🌧️ Hujan"),
    (65, "⛈️ Hujan Lebat"),
    (80, "🌦️ Hujan Lokal"),
    (95, "⛈️ Badai Petir"),
];

/// `weatherCode[...] || "🌍 Tidak diketahui"` (`menucat.js:305`).
const CUACA_TIDAK_DIKETAHUI: &str = "🌍 Tidak diketahui";

/// Fallback `weatherMenu` kalau API tidak bisa dihubungi
/// (`menucat.js:302` dan `:308`).
const CUACA_TIDAK_TERSEDIA: &str = "Cuaca tidak tersedia";

/// `db.setting("menucatVariant") || config.ui?.menucatVariant || 2`
/// (`menucat.js:127-128`). `config.ui` tidak ada di `Runtime` Rust, jadi
/// default-nya `2`.
const DEFAULT_VARIANT: i64 = 2;

/// Nama setting varian menucat.
const SETTING_VARIANT: &str = "menucatVariant";

/// Footer kartu varian 2 (`menucat.js:255`).
const FOOTER_VARIAN_2: &str = "Pilih tombol dibawah untuk kembali ke menu utama";

/// Footer kartu varian 5 (`menucat.js:324`).
const FOOTER_VARIAN_5: &str = "Please select the button in below";

/// Label tombol "balik ke daftar kategori" (`menucat.js:460`).
const TOMBOL_DAFTAR_KATEGORI: &str = "📂 Kembali Ke Daftar Kategori";

/// Label tombol "balik ke menu utama" (`menucat.js:279`).
const TOMBOL_MENU_UTAMA: &str = "🍅 Kembali Ke Menu Utama";

/// `newsletterJid` cadangan (`menucat.js:331`).
const JID_SALURAN_CADANGAN: &str = "120363400911374213@newsletter";

/// Kota default untuk lookup cuaca (`menucat.js:298`).
const KOTA_CUACA: &str = "Jakarta";

// ---------------------------------------------------------------------------
// Teks
// ---------------------------------------------------------------------------

/// `toSmallCaps(text)` (`menucat.js:62-96`).
///
/// Tabelnya disalin apa adanya — termasuk `s` dan `x` yang memang memetakan
/// ke dirinya sendiri di JS.
fn ke_small_caps(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'a' => 'ᴀ',
            'b' => 'ʙ',
            'c' => 'ᴄ',
            'd' => 'ᴅ',
            'e' => 'ᴇ',
            'f' => 'ꜰ',
            'g' => 'ɢ',
            'h' => 'ʜ',
            'i' => 'ɪ',
            'j' => 'ᴊ',
            'k' => 'ᴋ',
            'l' => 'ʟ',
            'm' => 'ᴍ',
            'n' => 'ɴ',
            'o' => 'ᴏ',
            'p' => 'ᴘ',
            'q' => 'ǫ',
            'r' => 'ʀ',
            's' => 's',
            't' => 'ᴛ',
            'u' => 'ᴜ',
            'v' => 'ᴠ',
            'w' => 'ᴡ',
            'x' => 'x',
            'y' => 'ʏ',
            'z' => 'ᴢ',
            other => other,
        })
        .collect()
}

/// `createBracketBox(emoji, title, lines)` (`menucat.js:98-105`).
fn kotak(emoji: &str, judul: &str, baris: &[String]) -> String {
    let mut teks = format!("╭─〔 {emoji} `{judul}`\n");
    for baris in baris {
        teks.push_str(&format!("┃ *{}*\n", ke_small_caps(baris)));
    }
    teks.push_str("╰─⬣\n\n");
    teks
}

/// `getCommandSymbols(cmdName)` (`menucat.js:107-118`).
///
/// `Ⓛ` butuh `plugin.config.limit` (`menucat.js:113`) — `PluginMeta` Rust
/// tidak punya field itu, jadi simbolnya tidak pernah ditulis.
fn simbol(meta: &PluginMeta) -> String {
    let mut simbol_urut: Vec<&str> = Vec::new();
    if meta.is_owner {
        simbol_urut.push("Ⓞ");
    }
    if meta.is_premium {
        simbol_urut.push("ⓟ");
    }
    if meta.is_admin {
        simbol_urut.push("Ⓐ");
    }
    if meta.is_group {
        simbol_urut.push("Ⓖ");
    }
    if meta.is_private {
        simbol_urut.push("Ⓟ");
    }
    if simbol_urut.is_empty() {
        return String::new();
    }
    format!(" {}", simbol_urut.join(" "))
}

/// `CATEGORY_EMOJIS[cat] || cadangan` (`menucat.js:217` dan `:393`).
fn emoji_kategori(kategori: &str, cadangan: &'static str) -> &'static str {
    CATEGORY_EMOJIS
        .iter()
        .find(|(kunci, _)| *kunci == kategori)
        .map(|(_, emoji)| *emoji)
        .unwrap_or(cadangan)
}

// ---------------------------------------------------------------------------
// Koleksi kategori & command
// ---------------------------------------------------------------------------

/// `getCommandsByCategory()` (`plugins.js:395-410`) — nama plugin per
/// kategori, plugin yang dimatikan dilewati.
fn plugin_kategori(kategori: &str) -> Vec<&'static PluginEntry> {
    let mut daftar: Vec<&'static PluginEntry> = all_plugins()
        .iter()
        .copied()
        .filter(|p| p.meta.is_enabled)
        .filter(|p| p.meta.category.eq_ignore_ascii_case(kategori))
        .collect();
    // JS memakai urutan registrasi plugin; Rust tidak menjamin itu, jadi
    // diurutkan alfabetis supaya hasilnya bisa diulang.
    daftar.sort_by(|a, b| a.meta.name.cmp(&b.meta.name));
    daftar
}

/// `allCategories` (`menucat.js:183-185` dan `:368-370`) — kategori unik
/// yang ada di registry.
fn semua_kategori() -> Vec<&'static str> {
    let mut kategori: Vec<&'static str> = Vec::new();
    for p in all_plugins() {
        if !kategori.contains(&p.meta.category.as_str()) {
            kategori.push(p.meta.category.as_str());
        }
    }
    kategori
}

/// `allCategories.find(c => c.toLowerCase() === categoryArg)`
/// (`menucat.js:371`).
fn cocok_kategori(argumen: &str) -> Option<&'static str> {
    semua_kategori()
        .into_iter()
        .find(|kategori| kategori.to_lowercase() == argumen)
}

/// `modeExcludeMap[botMode] || modeExcludeMap.md` (`menucat.js:153`).
fn exclude_kategori(mode: &str) -> &'static [&'static str] {
    MODE_EXCLUDE
        .iter()
        .find(|(kunci, _)| *kunci == mode)
        .and_then(|(_, daftar)| *daftar)
        .or_else(|| {
            MODE_EXCLUDE
                .iter()
                .find(|(kunci, _)| *kunci == "md")
                .and_then(|(_, daftar)| *daftar)
        })
        .unwrap_or(&[])
}

/// `groupData.botMode || "md"` (`menucat.js:132-133`).
fn mode_grup(ctx: &Ctx) -> String {
    if !ctx.is_group {
        return "md".to_string();
    }
    ctx.group_field("botMode")
        .and_then(|nilai| nilai.as_str())
        .filter(|mode| !mode.is_empty())
        .unwrap_or("md")
        .to_string()
}

/// Indeks kategori di `categoryOrder`; 999 kalau tidak ada
/// (`menucat.js:187-191`).
fn indeks_urutan(kategori: &str) -> i32 {
    URUTAN_KATEGORI
        .iter()
        .position(|urut| *urut == kategori)
        .map_or(999, |i| i as i32)
}

/// `visibleCats` (`menucat.js:193-200`) — kategori yang boleh dilihat.
fn kategori_terlihat(ctx: &Ctx) -> Vec<&'static str> {
    let exclude = exclude_kategori(&mode_grup(ctx));
    let mut kategori = semua_kategori();
    // `Array.prototype.sort` di JS stabil, jadi kategori dengan indeks
    // urutan yang sama tetap ikut urutan aslinya. Rust tidak punya urutan
    // registrasi yang dijamin, jadi nama kategori jadi pemutus seri.
    kategori.sort_by(|a, b| {
        indeks_urutan(a)
            .cmp(&indeks_urutan(b))
            .then_with(|| a.cmp(b))
    });
    kategori
        .into_iter()
        .filter(|kategori| {
            if *kategori == "owner" && !ctx.is_owner {
                return false;
            }
            if exclude
                .iter()
                .any(|kunci| kunci.eq_ignore_ascii_case(kategori))
            {
                return false;
            }
            !plugin_kategori(kategori).is_empty()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cuaca & thumbnail
// ---------------------------------------------------------------------------

/// `getTimeGreeting()` (`src/lib/formatter.js:508-515`), jam Asia/Jakarta.
fn sapa() -> &'static str {
    match crate::common::jakarta_parts(crate::common::now_ms()).3 {
        4..=9 => "Selamat Pagi 🌅",
        10..=14 => "Selamat Siang ☀️",
        15..=17 => "Selamat Sore 🌇",
        _ => "Selamat Malam 🌙",
    }
}

/// `weatherMenu(city)` (`menucat.js:298-310`).
///
/// Kegagalan apa pun (jaringan, JSON, hasil kosong) menghasilkan
/// `"Cuaca tidak tersedia"` — command tidak pernah gagal karena jaringan.
fn cuaca(ctx: &Ctx) -> String {
    let Some(lokasi) = geocode(ctx) else {
        return CUACA_TIDAK_TERSEDIA.to_string();
    };
    let (lat, lon, nama) = lokasi;
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code"
    );
    let Ok(res) = ctx.http.get(&url) else {
        return CUACA_TIDAK_TERSEDIA.to_string();
    };
    let Some(sekarang) = res.json().and_then(|json| json.get("current").cloned()) else {
        return CUACA_TIDAK_TERSEDIA.to_string();
    };
    let kode_cuaca = sekarang
        .get("weather_code")
        .and_then(|nilai| nilai.as_i64())
        .unwrap_or(-1);
    let kondisi = CUACA_KODE
        .iter()
        .find(|(kode, _)| *kode == kode_cuaca)
        .map(|(_, label)| *label)
        .unwrap_or(CUACA_TIDAK_DIKETAHUI);
    let suhu = sekarang
        .get("temperature_2m")
        .and_then(|nilai| nilai.as_f64())
        .map(|nilai| nilai.round() as i64);
    let suhu_teks = suhu.map_or_else(|| "NaN".to_string(), |nilai| nilai.to_string());
    format!("{kondisi} | 🌡️ {suhu_teks}°C\n📍 {nama}")
}

/// `geocoding-api.open-meteo.com` (`menucat.js:300`) — `(lat, lon, nama)`.
fn geocode(ctx: &Ctx) -> Option<(f64, f64, String)> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={KOTA_CUACA}&count=1"
    );
    let res = ctx.http.get(&url).ok()?;
    let json = res.json()?;
    let lokasi = json
        .get("results")
        .and_then(|hasil| hasil.as_array())
        .and_then(|hasil| hasil.first())?;
    let lat = lokasi.get("latitude").and_then(|nilai| nilai.as_f64())?;
    let lon = lokasi.get("longitude").and_then(|nilai| nilai.as_f64())?;
    let nama = lokasi
        .get("name")
        .and_then(|nilai| nilai.as_str())
        .unwrap_or(KOTA_CUACA)
        .to_string();
    Some((lat, lon, nama))
}

/// `safeThumbnail(getAssetBuffer("foto"), 300)` (`menucat.js:312`).
///
/// Thumbnail 300x300 dari aset `foto`; kalau asetnya tidak ada atau bukan
/// gambar, hasilnya `None` (JS: `safeThumbnail` mengembalikan `null`).
fn thumbnail(ctx: &Ctx) -> Option<Vec<u8>> {
    let aset = ctx.config().assets.get("foto")?;
    let byte = ctx.berkas().baca(aset).ok()?;
    oktz_plugin_api::gambar::sharp(&byte)
        .ok()?
        .ubah_ukuran(300, 300)
        .ke_jpeg(80)
        .ok()
}

// ---------------------------------------------------------------------------
// Kartu per varian
// ---------------------------------------------------------------------------

/// Varian aktif: `db.setting("menucatVariant") || config.ui?.menucatVariant || 2`
/// (`menucat.js:127-128`).
fn variant_aktif(ctx: &Ctx) -> i64 {
    ctx.setting_i64(SETTING_VARIANT)
        .filter(|varian| *varian > 0)
        .unwrap_or(DEFAULT_VARIANT)
}

/// Nama bot; JS pakai `config.bot?.name` (`menucat.js:262`).
fn nama_bot(ctx: &Ctx) -> String {
    let nama = &ctx.config().bot.name;
    if nama.is_empty() {
        "Bot".to_string()
    } else {
        nama.clone()
    }
}

/// `newsletterJid` (`menucat.js:331`).
fn jid_saluran(ctx: &Ctx) -> String {
    let id = &ctx.config().saluran.id;
    if id.is_empty() {
        JID_SALURAN_CADANGAN.to_string()
    } else {
        id.clone()
    }
}

/// `newsletterName` (`menucat.js:332`).
fn nama_saluran(ctx: &Ctx) -> String {
    let saluran = &ctx.config().saluran.name;
    if !saluran.is_empty() {
        return saluran.clone();
    }
    let bot = &ctx.config().bot.name;
    if bot.is_empty() {
        "Bot".to_string()
    } else {
        bot.clone()
    }
}

/// `contextInfo` varian 2 (`menucat.js:257-265`).
fn konteks_varian_2(ctx: &Ctx) -> CardContext {
    let mut context = CardContext::new()
        .forwarded(9)
        .quote(CardQuote::Text(nama_bot(ctx)));
    context.participant = Some("0@s.whatsapp.net".to_string());
    if !ctx.sender.is_empty() {
        context.mentioned.push(ctx.sender.clone());
    }
    context
}

/// `contextInfo` varian 5 & 6 (`menucat.js:326-334`).
fn konteks_saluran(ctx: &Ctx) -> CardContext {
    let mut context = CardContext::new()
        .forwarded(9)
        .newsletter(jid_saluran(ctx), nama_saluran(ctx));
    if !ctx.sender.is_empty() {
        context.mentioned.push(ctx.sender.clone());
    }
    context
}

/// Varian 2 — kartu interaktif dengan header gambar `foto2`
/// (`menucat.js:231-291`).
fn kartu_varian_2(ctx: &Ctx, teks: &str, dengan_kategori: bool) -> Option<Reply> {
    let aset = ctx.config().assets.get("foto2")?;
    if !ctx.berkas().ada(aset) {
        return None;
    }
    let pengembang = &ctx.config().bot.developer;
    let params = serde_json::json!({
        "limited_time_offer": {
            "text": sapa(),
            "url": "Hai",
            "copy_code": format!("Dibuat oleh {}", if pengembang.is_empty() { "Owner" } else { pengembang.as_str() }),
            "expiration_time": crate::common::now_ms() + 1_000_000,
        }
    });

    let mut reply = Reply::interactive(teks.to_string())
        .with_footer(FOOTER_VARIAN_2)
        .with_header(
            CardHeader::new()
                .title("")
                .subtitle("")
                .image(MediaSource::path(aset)),
        )
        .with_context(konteks_varian_2(ctx))
        .with_message_params(params);

    if dengan_kategori {
        reply = reply.with_button_card(InteractiveButton::quick_reply(
            TOMBOL_DAFTAR_KATEGORI,
            format!("{}menucat", ctx.prefix),
        ));
    }
    reply = reply.with_button_card(InteractiveButton::quick_reply(
        TOMBOL_MENU_UTAMA,
        format!("{}menu", ctx.prefix),
    ));
    Some(reply)
}

/// Varian 5 — kartu native-flow dengan header video `mp4` dan pesan yang
/// dirujuk berupa lokasi cuaca (`menucat.js:293-357`).
fn kartu_varian_5(ctx: &Ctx, teks: &str, dengan_kategori: bool) -> Option<Reply> {
    let aset = ctx.config().assets.get("mp4")?;
    if !ctx.berkas().ada(aset) {
        return None;
    }
    let params = serde_json::json!({
        "limited_time_offer": {
            "text": sapa(),
            "url": "Hai",
            "expiration_time": crate::common::now_ms() + 10_000,
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
        }
    });

    let mut context = konteks_saluran(ctx);
    context = context.quote(CardQuote::Location {
        latitude: 0.0,
        longitude: 0.0,
        name: cuaca(ctx),
        thumbnail: thumbnail(ctx),
    });

    let mut reply = Reply::interactive(teks.to_string())
        .with_footer(FOOTER_VARIAN_5)
        .with_header(
            CardHeader::new()
                .title("")
                .subtitle("")
                .video(MediaSource::path(aset)),
        )
        .with_context(context)
        .with_message_params(params)
        // Slot kosong pertama (`menucat.js:343`) — WA butuh minimal satu
        // tombol yang tidak berisi aksi.
        .with_button_card(InteractiveButton::empty(""));

    if dengan_kategori {
        reply = reply.with_button_card(InteractiveButton::quick_reply(
            TOMBOL_DAFTAR_KATEGORI,
            format!("{}menucat", ctx.prefix),
        ));
    }
    reply = reply.with_button_card(InteractiveButton::quick_reply(
        TOMBOL_MENU_UTAMA,
        format!("{}menu", ctx.prefix),
    ));
    Some(reply)
}

/// Varian 6 — kartu dengan header lokasi (nama bot + alamat cuaca), tanpa
/// tombol (`menucat.js:550-609`).
fn kartu_varian_6(ctx: &Ctx, teks: &str) -> Option<Reply> {
    let header = CardHeader::new().location(CardLocation {
        latitude: 0.0,
        longitude: 0.0,
        name: nama_bot(ctx),
        address: Some(cuaca(ctx)),
        thumbnail: thumbnail(ctx),
    });

    let mut context = konteks_saluran(ctx);
    // JS: `wrapInteractive(..., { quoted: m })` (`menucat.js:605`) — kartu
    // mengutip pesan pemicu.
    if !ctx.text.is_empty() {
        context = context.quote(CardQuote::Text(ctx.text.clone()));
    }

    Some(
        Reply::interactive(teks.to_string())
            .with_header(header)
            .with_context(context),
    )
}

/// Kirim sesuai varian aktif; jatuh ke teks polos kalau kartu tidak bisa
/// dibangun (aset hilang) — sama seperti `catch` JS (`menucat.js:362-364`).
fn kirim(ctx: &Ctx, teks: &str, dengan_kategori: bool) -> Reply {
    let teks_polos = || Reply::text(teks.to_string());
    match variant_aktif(ctx) {
        2 => kartu_varian_2(ctx, teks, dengan_kategori).unwrap_or_else(teks_polos),
        5 => kartu_varian_5(ctx, teks, dengan_kategori).unwrap_or_else(teks_polos),
        6 => kartu_varian_6(ctx, teks).unwrap_or_else(teks_polos),
        _ => teks_polos(),
    }
}

// ---------------------------------------------------------------------------
// Teks daftar kategori
// ---------------------------------------------------------------------------

/// Legend + satu bracket box per kategori terlihat (`menucat.js:202-224`).
fn teks_semua_kategori(ctx: &Ctx, prefix: &str) -> String {
    let mut teks = kotak(
        "🤖",
        "KETERANGAN",
        &[
            "Ⓞ = Hanya untuk owner".to_string(),
            "ⓟ = Hanya untuk premium".to_string(),
            "Ⓛ = Membutuhkan limit".to_string(),
            "Ⓐ = Hanya untuk admin".to_string(),
            "Ⓖ = Hanya di dalam grup".to_string(),
            "Ⓟ = Hanya di private chat".to_string(),
        ],
    );

    for kategori in kategori_terlihat(ctx) {
        let perintah = plugin_kategori(kategori);
        if perintah.is_empty() {
            continue;
        }
        let baris: Vec<String> = perintah
            .iter()
            .map(|p| format!("{prefix}{}{}", p.meta.name, simbol(&p.meta)))
            .collect();
        teks.push_str(&kotak(
            emoji_kategori(kategori, "📋"),
            &ke_small_caps(kategori),
            &baris,
        ));
    }
    teks
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

#[plugin(
    name = "menucat",
    alias = ["menucategory", "categorymenu", "mc", "category", "cat"],
    category = "main",
    description = "Menampilkan daftar command per kategori",
    usage = ".menucat <kategori>",
    example = ".menucat main",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false
)]
pub fn menucat(ctx: Ctx) -> Reply {
    // `config.command?.prefix || "."` (`menucat.js:121`).
    let prefix = if ctx.prefix.is_empty() {
        ".".to_string()
    } else {
        ctx.prefix.clone()
    };
    let argumen = ctx
        .args
        .first()
        .map(|argumen| argumen.to_lowercase())
        .unwrap_or_default();

    if argumen.is_empty() {
        // `menucat.js:131-366` — legend + daftar kategori.
        let teks = teks_semua_kategori(&ctx, &prefix);
        return kirim(&ctx, &teks, false);
    }

    // `menucat.js:368-405` — satu kategori.
    let Some(kategori) = cocok_kategori(&argumen) else {
        return Reply::text(format!(
            "❌ *KATEGORI TIDAK DITEMUKAN*\n\n> Kategori `{argumen}` tidak ada.\n> Ketik `{prefix}menucat` untuk list kategori."
        ));
    };

    if kategori == "owner" && !ctx.is_owner {
        return Reply::text("❌ *AKSES DITOLAK*\n\n> Kategori ini hanya untuk owner.");
    }

    let perintah = plugin_kategori(kategori);
    if perintah.is_empty() {
        return Reply::text(format!(
            "❌ *KOSONG*\n\n> Kategori `{kategori}` tidak memiliki command."
        ));
    }

    let baris: Vec<String> = perintah
        .iter()
        .map(|p| format!("{prefix}{}{}", p.meta.name, simbol(&p.meta)))
        .collect();
    let mut teks = kotak(
        emoji_kategori(kategori, "📁"),
        &ke_small_caps(kategori),
        &baris,
    );
    teks.push_str(&format!("Total: `{}` commands", perintah.len()));
    // `caseCommands` tidak ada di registry Rust, jadi bagian
    // `(X plugin + Y case)` (`menucat.js:403-405`) tidak pernah ditulis.

    kirim(&ctx, &teks, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use oktz_plugin_api::config::{Assets, BotInfo, Runtime, Saluran};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    const SENDER: &str = "628111@s.whatsapp.net";

    fn assets() -> Assets {
        Assets(BTreeMap::from([
            ("foto".to_string(), "./assets/image/foto.jpeg".to_string()),
            ("foto2".to_string(), "./assets/image/foto2.jpeg".to_string()),
            ("mp4".to_string(), "./assets/video/mp4.mp4".to_string()),
        ]))
    }

    fn runtime() -> Runtime {
        Runtime {
            bot: BotInfo {
                name: "Oktz Bot".into(),
                version: "3.3.1".into(),
                developer: "Zann".into(),
                website: String::new(),
            },
            assets: assets(),
            saluran: Saluran {
                id: "120363411617217441@newsletter".into(),
                name: "Join saluran resmi Oktz".into(),
                link: String::new(),
            },
            ..Runtime::default()
        }
    }

    /// Konteks dengan aset sungguhan ada di `BerkasPalsu` supaya varian
    /// kartu bisa dibangun.
    fn ctx_dengan_aset(variant: i64, argumen: &[&str]) -> Ctx {
        let berkas = BerkasPalsu::baru();
        berkas.tulis("./assets/image/foto.jpeg", &[1, 2, 3]).unwrap();
        berkas.tulis("./assets/image/foto2.jpeg", &[1, 2, 3]).unwrap();
        berkas.tulis("./assets/video/mp4.mp4", &[1, 2, 3]).unwrap();
        Ctx {
            prefix: ".".into(),
            args: argumen.iter().map(|a| a.to_string()).collect(),
            command: "menucat".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: argumen.join(" "),
            is_owner: true,
            settings: serde_json::json!({ "menucatVariant": variant }),
            config: Arc::new(runtime()),
            berkas: Arc::new(berkas),
            ..Ctx::default()
        }
    }

    fn ctx(argumen: &[&str]) -> Ctx {
        let mut c = ctx_dengan_aset(DEFAULT_VARIANT, argumen);
        c.settings = serde_json::Value::Null;
        c
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn kartu(r: &Reply) -> (&str, Option<&str>, &[oktz_plugin_api::kartu::Tombol]) {
        match r.without_mutations() {
            Reply::Interactive { kartu } => (kartu.body.as_str(), kartu.footer.as_deref(), kartu.buttons.as_slice()),
            other => panic!("harus kartu interaktif, dapat {other:?}"),
        }
    }

    // -- teks ---------------------------------------------------------------

    #[test]
    fn small_caps_memakai_tabel_dari_js() {
        assert_eq!(ke_small_caps("abc"), "ᴀʙᴄ");
        assert_eq!(ke_small_caps("ABC"), "ᴀʙᴄ");
        // `s` dan `x` memetakan ke dirinya sendiri di tabel JS.
        assert_eq!(ke_small_caps("sx"), "sx");
        assert_eq!(ke_small_caps("tools"), "ᴛᴏᴏʟs");
        // Angka dan tanda baca tidak berubah.
        assert_eq!(ke_small_caps(".MENUCAT 1"), ".ᴍᴇɴᴜᴄᴀᴛ 1");
    }

    #[test]
    fn bracket_box_punya_bentuk_yang_sama_dengan_js() {
        let baris = vec![".menu".to_string(), ".tools".to_string()];
        assert_eq!(
            kotak("🤖", "KETERANGAN", &baris),
            "╭─〔 🤖 `KETERANGAN`\n┃ *.ᴍᴇɴᴜ*\n┃ *.ᴛᴏᴏʟs*\n╰─⬣\n\n"
        );
    }

    /// Keluaran `createBracketBox` JS untuk legend `KETERANGAN` — diambil
    /// dari `menucat.js:203-210` yang dijalankan di Node. `toLowerCase` ikut
    /// menurunkan huruf bulat (Ⓞ -> ⓞ), jadi string ini juga menguji
    /// keakuratan tabel small caps.
    #[test]
    fn legend_sama_dengan_keluaran_js() {
        let baris: Vec<String> = [
            "Ⓞ = Hanya untuk owner",
            "ⓟ = Hanya untuk premium",
            "Ⓛ = Membutuhkan limit",
            "Ⓐ = Hanya untuk admin",
            "Ⓖ = Hanya di dalam grup",
            "Ⓟ = Hanya di private chat",
        ]
        .iter()
        .map(|baris| baris.to_string())
        .collect();
        assert_eq!(
            kotak("🤖", "KETERANGAN", &baris),
             "╭─〔 🤖 `KETERANGAN`\n\
             ┃ *ⓞ = ʜᴀɴʏᴀ ᴜɴᴛᴜᴋ ᴏᴡɴᴇʀ*\n\
             ┃ *ⓟ = ʜᴀɴʏᴀ ᴜɴᴛᴜᴋ ᴘʀᴇᴍɪᴜᴍ*\n\
             ┃ *ⓛ = ᴍᴇᴍʙᴜᴛᴜʜᴋᴀɴ ʟɪᴍɪᴛ*\n\
             ┃ *ⓐ = ʜᴀɴʏᴀ ᴜɴᴛᴜᴋ ᴀᴅᴍɪɴ*\n\
             ┃ *ⓖ = ʜᴀɴʏᴀ ᴅɪ ᴅᴀʟᴀᴍ ɢʀᴜᴘ*\n\
             ┃ *ⓟ = ʜᴀɴʏᴀ ᴅɪ ᴘʀɪᴠᴀᴛᴇ ᴄʜᴀᴛ*\n\
             ╰─⬣\n\n"
        );
    }

    #[test]
    fn simbol_mengikuti_urutan_dan_kondisi_js() {
        let meta = PluginMeta {
            is_owner: true,
            is_premium: true,
            is_admin: true,
            is_group: true,
            is_private: true,
            ..PluginMeta::default()
        };
        // `Ⓛ` tidak ada — `PluginMeta` tidak punya field limit.
        assert_eq!(simbol(&meta), " Ⓞ ⓟ Ⓐ Ⓖ Ⓟ");
        assert_eq!(simbol(&PluginMeta::default()), "");
        assert_eq!(
            simbol(&PluginMeta {
                is_group: true,
                ..PluginMeta::default()
            }),
            " Ⓖ"
        );
    }

    #[test]
    fn daftar_kategori_berisi_legend_dan_kotak_kategori() {
        let t = teks(&menucat(ctx(&[])));
        assert!(t.contains("╭─〔 🤖 `KETERANGAN`"), "{t}");
        // `toLowerCase` JS ikut menurunkan huruf bulat (Ⓞ -> ⓞ), persis
        // seperti yang dilakukan `toSmallCaps` di JS.
        assert!(t.contains("ⓞ = ʜᴀɴʏᴀ ᴜɴᴛᴜᴋ ᴏᴡɴᴇʀ"), "{t}");
        assert!(t.contains("ⓟ = ʜᴀɴʏᴀ ᴅɪ ᴘʀɪᴠᴀᴛᴇ ᴄʜᴀᴛ"), "{t}");
        assert!(t.contains("ⓛ = ᴍᴇᴍʙᴜᴛᴜʜᴋᴀɴ ʟɪᴍɪᴛ"), "{t}");
        // Kategori "main" ada dan judulnya small-caps.
        assert!(t.contains("╭─〔 🏠 `ᴍᴀɪɴ`"), "{t}");
        assert!(t.contains("╰─⬣"), "{t}");
    }

    #[test]
    fn kategori_tertentu_menampilkan_total_commands() {
        let t = teks(&menucat(ctx(&["main"])));
        assert!(t.contains("╭─〔 🏠 `ᴍᴀɪɴ`"), "{t}");
        assert!(t.contains("Total: `"), "{t}");
        assert!(t.contains("commands"), "{t}");
        // Tidak ada "case" di registry Rust, jadi bagian `(X plugin + Y case)`
        // tidak ditulis.
        assert!(!t.contains("plugin +"), "{t}");
    }

    #[test]
    fn kategori_tidak_dikenal_memberi_saran() {
        let t = teks(&menucat(ctx(&["tidakada"])));
        assert!(t.contains("KATEGORI TIDAK DITEMUKAN"), "{t}");
        assert!(t.contains("tidak ada"), "{t}");
    }

    #[test]
    fn non_owner_tidak_melihat_kategori_owner() {
        let mut c = ctx(&["owner"]);
        c.is_owner = false;
        let t = teks(&menucat(c));
        assert!(t.contains("AKSES DITOLAK"), "{t}");
    }

    #[test]
    fn owner_tetap_bisa_melihat_kategori_owner() {
        let t = teks(&menucat(ctx(&["owner"])));
        assert!(t.contains("╭─〔 👑 `ᴏᴡɴᴇʀ`"), "{t}");
    }

    #[test]
    fn kategori_owner_tidak_muncul_di_daftar_untuk_non_owner() {
        let mut c = ctx(&[]);
        c.is_owner = false;
        let t = teks(&menucat(c));
        assert!(!t.contains("👑"), "{t}");
    }

    #[test]
    fn kategori_pushkontak_dan_store_disembunyikan_di_mode_md() {
        let t = teks(&menucat(ctx(&[])));
        assert!(!t.contains("`ᴘᴜsʜᴋᴏɴᴛᴀᴋ`"), "{t}");
        assert!(!t.contains("`sᴛᴏʀᴇ`"), "{t}");
    }

    // -- kartu --------------------------------------------------------------

    #[test]
    fn varian_satu_mengirim_teks_polos() {
        let r = menucat(ctx_dengan_aset(1, &["main"]));
        assert!(matches!(r.without_mutations(), Reply::Text { .. }));
    }

    #[test]
    fn varian_dua_membawa_header_gambar_footer_dan_tombol() {
        let r = menucat(ctx_dengan_aset(2, &["main"]));
        let (body, footer, buttons) = kartu(&r);
        assert!(body.contains("╭─〔 🏠 `ᴍᴀɪɴ`"), "{body}");
        assert_eq!(footer, Some(FOOTER_VARIAN_2));
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0].name, "quick_reply");
        assert_eq!(buttons[0].params["display_text"], TOMBOL_DAFTAR_KATEGORI);
        assert_eq!(buttons[0].params["id"], ".menucat");
        assert_eq!(buttons[1].params["display_text"], TOMBOL_MENU_UTAMA);
        assert_eq!(buttons[1].params["id"], ".menu");

        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("harus kartu");
        };
        let header = kartu.header_card.as_ref().expect("header wajib ada");
        match header.media.as_ref().expect("media wajib ada") {
            oktz_plugin_api::registry::CardMedia::Image { source } => {
                assert_eq!(source.label(), "path");
            }
            other => panic!("harus gambar, dapat {other:?}"),
        }
    }

    #[test]
    fn varian_dua_tanpa_kategori_hanya_punya_satu_tombol() {
        let r = menucat(ctx_dengan_aset(2, &[]));
        let (_, _, buttons) = kartu(&r);
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].params["id"], ".menu");
    }

    #[test]
    fn varian_lima_membawa_video_tombol_kosong_dan_quote_lokasi() {
        let r = menucat(ctx_dengan_aset(5, &["main"]));
        let (_, footer, buttons) = kartu(&r);
        assert_eq!(footer, Some(FOOTER_VARIAN_5));
        // Slot kosong pertama, lalu daftar kategori, lalu menu utama.
        assert_eq!(buttons.len(), 3);
        assert_eq!(buttons[0].name, "");
        assert_eq!(buttons[0].params.nilai(), &serde_json::Value::String(String::new()));
        assert_eq!(buttons[1].params["id"], ".menucat");
        assert_eq!(buttons[2].params["id"], ".menu");

        let Reply::Interactive { kartu } = r.without_mutations()
        else {
            panic!("harus kartu");
        };
        let header = kartu.header_card.as_ref().expect("header wajib ada");
        assert!(matches!(
            header.media.as_ref(),
            Some(oktz_plugin_api::registry::CardMedia::Video { .. })
        ));
        let params = kartu.params
            .as_ref()
            .expect("message params wajib ada");
        assert!(params["bottom_sheet"]["list_title"] == "Please select the menu");
        assert!(params["tap_target_configuration"]["domain"] == "shop.example.com");
        assert!(params["limited_time_offer"]["url"] == "Hai");

        let context = kartu.context_card.as_ref().expect("context wajib ada");
        assert!(context.is_forwarded);
        assert_eq!(context.forwarding_score, 9);
        assert_eq!(context.mentioned, vec![SENDER.to_string()]);
        let newsletter = context
            .forwarded_newsletter
            .as_ref()
            .expect("surat berita wajib ada");
        assert_eq!(newsletter.jid, "120363411617217441@newsletter");
        assert_eq!(newsletter.server_message_id, 127);
        match context.quote.as_deref() {
            Some(oktz_plugin_api::registry::CardQuote::Location { name, .. }) => {
                assert!(!name.is_empty());
            }
            other => panic!("harus quote lokasi, dapat {other:?}"),
        }
    }

    #[test]
    fn varian_enam_membawa_header_lokasi_tanpa_tombol() {
        let r = menucat(ctx_dengan_aset(6, &["main"]));
        let (body, footer, buttons) = kartu(&r);
        assert!(body.contains("╭─〔 🏠 `ᴍᴀɪɴ`"), "{body}");
        assert_eq!(footer, None);
        assert!(buttons.is_empty());

        let Reply::Interactive { kartu } = r.without_mutations() else {
            panic!("harus kartu");
        };
        let header = kartu.header_card.as_ref().expect("header wajib ada");
        match header.media.as_ref().expect("media wajib ada") {
            oktz_plugin_api::registry::CardMedia::Location(loc) => {
                assert_eq!(loc.name, "Oktz Bot");
                assert_eq!(loc.latitude, 0.0);
                assert_eq!(loc.longitude, 0.0);
                assert!(loc.address.is_some());
            }
            other => panic!("harus lokasi, dapat {other:?}"),
        }
    }

    #[test]
    fn aset_yang_hilang_menjatuh_ke_teks_polos() {
        // Tanpa aset, `foto2` tidak ada → kartu tidak bisa dibangun.
        let mut c = ctx_dengan_aset(2, &["main"]);
        c.berkas = Arc::new(BerkasPalsu::baru());
        let r = menucat(c);
        assert!(matches!(r.without_mutations(), Reply::Text { .. }));
        assert!(teks(&r).contains("Total: `"));
    }

    #[test]
    fn variant_default_dua_bila_setting_tidak_ada() {
        let mut c = ctx_dengan_aset(1, &["main"]);
        c.settings = serde_json::Value::Null;
        assert_eq!(variant_aktif(&c), DEFAULT_VARIANT);
        // Setting 0 falsy di JS (`||`), jadi bukan varian 0.
        c.settings = serde_json::json!({ "menucatVariant": 0 });
        assert_eq!(variant_aktif(&c), DEFAULT_VARIANT);
        // Setting yang ada dipakai apa adanya.
        c.settings = serde_json::json!({ "menucatVariant": 5 });
        assert_eq!(variant_aktif(&c), 5);
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("menucat").unwrap();
        assert_eq!(p.meta.name, "menucat");
        assert_eq!(
            p.meta
                .alias
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["menucategory", "categorymenu", "mc", "category", "cat"]
        );
        assert_eq!(p.meta.category, "main");
        assert_eq!(p.meta.description, "Menampilkan daftar command per kategori");
        assert_eq!(p.meta.usage, ".menucat <kategori>");
        assert_eq!(p.meta.example, ".menucat main");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn setiap_alias_menunjuk_ke_menucat() {
        for alias in ["menucategory", "categorymenu", "mc", "category", "cat"] {
            let found = oktz_plugin_api::find(alias).unwrap_or_else(|| panic!("alias {alias} hilang"));
            assert_eq!(found.meta.name, "menucat", "alias {alias} salah resolve");
        }
    }
}
