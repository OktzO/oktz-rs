//! `plugins/main/allmenu.js` — semua command lengkap per kategori.
//!
//! Port penuh: blok "Informasi User" / "Informasi Bot" / "Kelebihan Bot Ini",
//! bracket box per kategori, simbol permission, urutan + penyaringan kategori
//! sesuai mode bot grup, empat varian tampilan (setting `allmenuVariant`),
//! dan audio menu (setting `audioMenu`).
//!
//! Yang diport apa adanya:
//! - `pluginConfig` (`allmenu.js:24-38`).
//! - `CATEGORY_EMOJIS` (`allmenu.js:39-68`) — 28 kunci, disalin verbatim.
//! - `createBracketBox` (`allmenu.js:70-77`).
//! - `getCommandSymbols` (`allmenu.js:78-89`) — `🅞 🅟 🅛 🅐 🅖 🅟🅡`.
//!   `🅛` (limit) tidak bisa muncul: `PluginMeta` Rust tidak punya field
//!   `limit` (`lib/oktz-plugin-api/meta.rs`), jadi hanya lima simbol lain
//!   yang bisa ditulis — sisanya tetap urut dan sama.
//! - `getContextInfo` (`allmenu.js:90-104`) — mention + forwarded `9999` +
//!   info surat berita.
//! - `categoryOrder` (`allmenu.js:158-179`) dan `MODES`
//!   (`plugins/group/botmode.js:19-56`) yang dipakai `allmenu.js:197-207`.
//! - Varian 1 (teks polos), 2 (kartu + header gambar + tombol "Kembali Ke
//!   Menu Utama"), 5 (kartu native-flow + header video + quote lokasi cuaca)
//!   dan 6 (kartu + header lokasi) (`allmenu.js:235-415`).
//! - Audio menu yang dikirim setelah varian (`allmenu.js:419-591`).
//!
//! `ponytail:` yang berbeda dari JS, beserta alasannya:
//! - "case" dari `case/foto.js` tidak ada di registry Rust, jadi hanya
//!   plugin yang bisa dihitung. Angka "fitur" = jumlah plugin aktif saja —
//!   persis seperti JS kalau `casesByCategory` kosong.
//! - Urutan command di dalam satu kategori: JS memakai urutan registrasi
//!   plugin, Rust memakai urutan alfabetis. `all_plugins()` tidak menjamin
//!   urutan registrasi, dan hasil yang bisa diulang lebih berguna.
//! - `createBracketBox` di JS menerima `emoji` tapi TIDAK memakainya
//!   (`allmenu.js:71-75`) — judul kotak cuma `${title} MENU `. Parameternya
//!   tetap dibawa di sini supaya tanda tangannya sama, tapi ditandai
//!   `_emoji` supaya tidak ada peringatan compiler.
//! - Varian 2/5/6 butuh aset header (`foto` / `mp4`). Kalau asetnya hilang,
//!   kartu tidak bisa dibangun → jatuh ke balasan cadangan, sama seperti
//!   `catch` JS (`allmenu.js:592-606`).
//! - Audio menu: JS mengubah MP3 jadi Opus (voice note) lewat ffmpeg untuk
//!   gaya 1 & 2 (`allmenu.js:428-465`). Di sini MP3 dikirim apa adanya
//!   (`Reply::audio`), yaitu cabang cadangan JS (`allmenu.js:458-464`).
//! - `Reply::Image` di API Rust tidak punya `contextInfo`, jadi forwarded +
//!   surat berita dari `getContextInfo` tidak ikut pada balasan cadangan
//!   gambar; mention pemicunya tetap dibawa.

use oktz_plugin_api::registry::{
    CardContext, CardHeader, CardLocation, CardQuote, InteractiveButton, MediaSource, PluginEntry,
};
use oktz_plugin_api::{Ctx, PluginMeta, Reply, all_plugins, plugin};

// ---------------------------------------------------------------------------
// Konstanta dari JS
// ---------------------------------------------------------------------------

/// `CATEGORY_EMOJIS` (`allmenu.js:39-68`) — 28 kunci, disalin apa adanya.
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
    ("linode", "☁️"),
    ("random", "🎲"),
    ("canvas", "🎨"),
    ("vps", "🌊"),
    ("store", "🏪"),
    ("premium", "💎"),
    ("convert", "🔄"),
    ("economy", "💰"),
    ("cek", "📋"),
    ("ephoto", "🎨"),
    ("jpm", "📢"),
    ("pushkontak", "📱"),
];

/// `categoryOrder` (`allmenu.js:158-179`) — urutan kategori di daftar menu.
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
];

/// `MODES` (`plugins/group/botmode.js:19-56`) yang dipakai `allmenu.js:197-207`:
/// (mode, `allowedCategories`, `excludeCategories`).
///
/// `None` berarti "tidak ada batas" — persis `null` di JS, yang berarti
/// seluruh kategori lolos.
type ModeKatori = (&'static str, Option<&'static [&'static str]>, Option<&'static [&'static str]>);

const MODE: &[ModeKatori] = &[
    ("md", None, Some(&["cpanel", "pushkontak", "store"])),
    ("all", None, None),
    (
        "cpanel",
        Some(&["main", "group", "sticker", "owner", "tools", "panel"]),
        None,
    ),
    (
        "pushkontak",
        Some(&["main", "group", "sticker", "owner", "pushkontak"]),
        None,
    ),
    ("store", Some(&["main", "group", "sticker", "owner", "store"]), None),
    ("otp", Some(&["main", "group", "sticker", "owner", "otp"]), None),
];

/// `db.setting("allmenuVariant") || botConfig.ui?.allmenuVariant || 2`
/// (`allmenu.js:231-232`). `config.ui` (`config.js:189-191`) cuma punya
/// `menuVariant`, jadi tidak ada `allmenuVariant` — default-nya literal `2`.
const DEFAULT_VARIANT: i64 = 2;

/// Nama setting varian allmenu (`allmenu.js:231`).
const SETTING_VARIANT: &str = "allmenuVariant";

/// `db.setting("audioMenu")` (`allmenu.js:419`).
const SETTING_AUDIO: &str = "audioMenu";

/// `newsletterJid` cadangan (`allmenu.js:91` dan `:331`).
const JID_SALURAN_CADANGAN: &str = "120363400911374213@newsletter";

/// Kota default untuk lookup cuaca (`allmenu.js:300`).
const KOTA_CUACA: &str = "Jakarta";

/// Nama bot cadangan (`allmenu.js:136`).
const NAMA_BOT_CADANGAN: &str = "Remi AI";

/// Nama pengembang cadangan (`allmenu.js:137`).
const NAMA_DEV_CADANGAN: &str = "Ell";

/// Versi bot cadangan (`allmenu.js:138`).
const VERSI_BOT_CADANGAN: &str = "11.0.0";

/// Footer kartu varian 5 (`allmenu.js:324`).
const FOOTER_VARIAN_5: &str = "Please select the button in below";

/// Label tombol "balik ke daftar kategori" (`allmenu.js:346`).
const TOMBOL_DAFTAR_KATEGORI: &str = "📂 Kembali Ke Daftar Kategori";

/// Label tombol "balik ke menu utama" (`allmenu.js:283` dan `:350`).
const TOMBOL_MENU_UTAMA: &str = "🍅 Kembali Ke Menu Utama";

// ---------------------------------------------------------------------------
// Teks
// ---------------------------------------------------------------------------

/// `createBracketBox(emoji, title, lines)` (`allmenu.js:70-77`).
///
/// `emoji` sengaja tidak dipakai: di JS pun parameternya tidak masuk ke
/// badan fungsi (`allmenu.js:71-75`) — kotak cuma berisi judul dan baris
/// command.
fn kotak(_emoji: &str, judul: &str, baris: &[String]) -> String {
    let mut teks = format!("╭➤--「{judul} MENU 」\n");
    for baris in baris {
        teks.push_str(&format!("┆ ⇝ {baris}\n"));
    }
    teks.push_str("╰➤-------------------------------\n\n");
    teks
}

/// `getCommandSymbols(cmdName)` (`allmenu.js:78-89`).
///
/// `🅛` butuh `plugin.config.limit` (`allmenu.js:84`) — `PluginMeta` Rust
/// tidak punya field itu, jadi simbolnya tidak pernah ditulis. `🅟🅡`
/// (private) ditulis sebagai SATU entri, persis seperti di JS.
fn simbol(meta: &PluginMeta) -> String {
    let mut simbol_urut: Vec<&str> = Vec::new();
    if meta.is_owner {
        simbol_urut.push("🅞");
    }
    if meta.is_premium {
        simbol_urut.push("🅟");
    }
    if meta.is_admin {
        simbol_urut.push("🅐");
    }
    if meta.is_group {
        simbol_urut.push("🅖");
    }
    if meta.is_private {
        simbol_urut.push("🅟🅡");
    }
    if simbol_urut.is_empty() {
        return String::new();
    }
    format!(" {}", simbol_urut.join(" "))
}

/// `CATEGORY_EMOJIS[cat] || "📋"` (`allmenu.js:223`).
fn emoji_kategori(kategori: &str) -> &'static str {
    CATEGORY_EMOJIS
        .iter()
        .find(|(kunci, _)| kunci.eq_ignore_ascii_case(kategori))
        .map(|(_, emoji)| *emoji)
        .unwrap_or("📋")
}

// ---------------------------------------------------------------------------
// Koleksi kategori & command
// ---------------------------------------------------------------------------

/// `getCommandsByCategory()` (`plugins.js:395-410`) — plugin aktif per
/// kategori; plugin yang dimatikan dilewati.
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

/// `getCategories()` (`plugins.js:363-365`) — kategori unik yang ada di
/// registry.
fn semua_kategori() -> Vec<&'static str> {
    let mut kategori: Vec<&'static str> = Vec::new();
    for p in all_plugins() {
        if !kategori
            .iter()
            .any(|k| k.eq_ignore_ascii_case(p.meta.category.as_str()))
        {
            kategori.push(p.meta.category.as_str());
        }
    }
    kategori
}

/// `groupData.botMode || "md"` (`allmenu.js:108-109`).
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

/// `modeAllowedMap[botMode]` / `modeExcludeMap[botMode] || []`
/// (`allmenu.js:208-209`).
///
/// Mode yang tidak ada di `MODES` tidak menyaring apa pun — persis
/// `undefined` di JS yang lolos ke `includes`/`|| []`.
fn aturan_mode(mode: &str) -> (Option<&'static [&'static str]>, &'static [&'static str]) {
    MODE.iter()
        .find(|(kunci, _, _)| *kunci == mode)
        .map(|(_, allowed, exclude)| (*allowed, exclude.unwrap_or(&[])))
        .unwrap_or((None, &[]))
}

/// Indeks kategori di `categoryOrder`; 999 kalau tidak ada
/// (`allmenu.js:180-184`).
fn indeks_urutan(kategori: &str) -> i32 {
    URUTAN_KATEGORI
        .iter()
        .position(|urut| *urut == kategori)
        .map_or(999, |i| i as i32)
}

/// Kategori yang boleh ditampilkan (`allmenu.js:210-218`).
///
/// - `owner` hanya untuk owner (`allmenu.js:211`).
/// - Kategori di luar `allowedCategories` dilewati (`allmenu.js:212-216`).
/// - Kategori di `excludeCategories` dilewati (`allmenu.js:217-218`).
/// - Kategori tanpa command aktif dilewati (`allmenu.js:222`).
fn kategori_terlihat(ctx: &Ctx) -> Vec<&'static str> {
    let (allowed, exclude) = aturan_mode(&mode_grup(ctx));
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
            if let Some(allowed) = allowed
                && !allowed
                    .iter()
                    .any(|kunci| kunci.eq_ignore_ascii_case(kategori))
            {
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

/// `totalCommands` (`allmenu.js:113-116`) — jumlah plugin aktif di semua
/// kategori (kategori `owner` ikut dihitung, sama seperti JS).
fn total_perintah() -> usize {
    all_plugins()
        .iter()
        .filter(|p| p.meta.is_enabled)
        .count()
}

// ---------------------------------------------------------------------------
// Nilai runtime
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

/// `botConfig.bot?.name || "Remi AI"` (`allmenu.js:136`).
fn nama_bot(ctx: &Ctx) -> String {
    let nama = &ctx.config().bot.name;
    if nama.is_empty() {
        NAMA_BOT_CADANGAN.to_string()
    } else {
        nama.clone()
    }
}

/// `botConfig.bot?.developer || "Ell"` (`allmenu.js:137`).
fn nama_dev(ctx: &Ctx) -> String {
    let nama = &ctx.config().bot.developer;
    if nama.is_empty() {
        NAMA_DEV_CADANGAN.to_string()
    } else {
        nama.clone()
    }
}

/// `botConfig.bot?.version || "11.0.0"` (`allmenu.js:138`).
fn versi_bot(ctx: &Ctx) -> String {
    let versi = &ctx.config().bot.version;
    if versi.is_empty() {
        VERSI_BOT_CADANGAN.to_string()
    } else {
        versi.clone()
    }
}

/// `m.pushName || "User"` (`allmenu.js:139`).
fn nama_pengguna(ctx: &Ctx) -> String {
    if ctx.push_name.is_empty() {
        "User".to_string()
    } else {
        ctx.push_name.clone()
    }
}

/// `new Date().toLocaleTimeString('id-ID', { timeZone: 'Asia/Jakarta' })`
/// lalu `.replace(/\./g, ':')` (`allmenu.js:140`).
///
/// Format `id-ID` memakai titik sebagai pemisah (`23.13.14`), jadi setelah
/// digantititik sama dengan `HH:MM:SS`.
fn waktu_sekarang() -> String {
    let (_, _, _, jam, menit, detik) = crate::common::jakarta_parts(crate::common::now_ms());
    format!("{jam:02}:{menit:02}:{detik:02}")
}

/// `userRole` (`allmenu.js:119-124`).
fn peran_pengguna(ctx: &Ctx) -> &'static str {
    if ctx.is_owner {
        "Owner"
    } else if ctx.is_premium {
        "Premium"
    } else {
        "User"
    }
}

/// `userLimit` (`allmenu.js:135`).
fn limit_pengguna(ctx: &Ctx) -> String {
    if ctx.is_premium || ctx.is_owner {
        return "∞ Unlimited".to_string();
    }
    ctx.user_field("limit")
        .and_then(|nilai| nilai.as_i64())
        .unwrap_or(0)
        .to_string()
}

/// `botConfig.saluran?.id || "120363400911374213@newsletter"`
/// (`allmenu.js:91` dan `:331`).
fn jid_saluran(ctx: &Ctx) -> String {
    let id = &ctx.config().saluran.id;
    if id.is_empty() {
        JID_SALURAN_CADANGAN.to_string()
    } else {
        id.clone()
    }
}

/// `botConfig.saluran?.name || botConfig.bot?.name || "Bot"`
/// (`allmenu.js:92-93` dan `:332`).
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

/// `getContextInfo(botConfig, m)` (`allmenu.js:90-104`).
///
/// `mentionedJid: [m.sender]`, `forwardingScore: 9999`, dan info surat
/// berita dengan `serverMessageId: 127`.
fn konteks_info(ctx: &Ctx) -> CardContext {
    let mut context = CardContext::new()
        .forwarded(9999)
        .newsletter(jid_saluran(ctx), nama_saluran(ctx));
    if !ctx.sender.is_empty() {
        context.mentioned.push(ctx.sender.clone());
    }
    context
}

/// `contextInfo` varian 5 & 6 (`allmenu.js:326-334` dan `:397-405`).
///
/// `forwardingScore: 9` + info surat berita.
fn konteks_saluran(ctx: &Ctx) -> CardContext {
    let mut context = CardContext::new()
        .forwarded(9)
        .newsletter(jid_saluran(ctx), nama_saluran(ctx));
    if !ctx.sender.is_empty() {
        context.mentioned.push(ctx.sender.clone());
    }
    context
}

/// `contextInfo` varian 2 (`allmenu.js:259-269`).
///
/// `fprwardingScore` di JS itu TYPO (huruf `p` kelewat), jadi skor
/// forwarding tidak ikut terisi di sana; di sini dipakai `9` supaya
/// kartu tetap ditandai forwarded.
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

/// `thumbnail = sharp(getAssetBuffer("foto")).resize(300, 300)`
/// (`allmenu.js:314` dan `:379`).
///
/// Thumbnail 300x300 dari aset `foto`; kalau asetnya tidak ada atau bukan
/// gambar, hasilnya `None`.
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
// Teks daftar command
// ---------------------------------------------------------------------------

/// Blok "Informasi User" / "Informasi Bot" / "Kelebihan Bot Ini" +
/// bracket box per kategori terlihat (`allmenu.js:142-230`).
fn teks_menu(ctx: &Ctx, prefix: &str) -> String {
    let mut txt = String::new();
    txt.push_str("✦ *Informasi User*\n");
    txt.push_str(&format!("• Nama   : {}\n", nama_pengguna(ctx)));
    txt.push_str(&format!("• Status : {}\n", peran_pengguna(ctx)));
    txt.push_str(&format!("• Limit  : {}\n\n", limit_pengguna(ctx)));
    txt.push_str("✦ *Informasi Bot*\n");
    txt.push_str(&format!("• Nama   : {}\n", nama_bot(ctx)));
    txt.push_str(&format!("• Fitur  : {} fitur\n", total_perintah()));
    txt.push_str(&format!("• Versi  : {}\n", versi_bot(ctx)));
    txt.push_str(&format!("• Waktu  : {} WIB\n\n", waktu_sekarang()));
    txt.push_str("✦ *Kelebihan Bot Ini*\n");
    txt.push_str("• Fast response\n");
    txt.push_str("• Stable system\n");
    txt.push_str("• Multi feature\n");
    txt.push_str("• User friendly\n\n");
    txt.push_str("Silakan pilih menu di bawah ini.\n");
    txt.push_str("Gunakan sesuai kebutuhan dan jangan lupa gunakan dengan bijak.\n\n");

    for kategori in kategori_terlihat(ctx) {
        let perintah = plugin_kategori(kategori);
        if perintah.is_empty() {
            continue;
        }
        let baris: Vec<String> = perintah
            .iter()
            .map(|p| format!("{prefix}{}{}", p.meta.name, simbol(&p.meta)))
            .collect();
        txt.push_str(&kotak(
            emoji_kategori(kategori),
            &kategori.to_uppercase(),
            &baris,
        ));
    }
    txt
}

// ---------------------------------------------------------------------------
// Kartu per varian
// ---------------------------------------------------------------------------

/// Varian aktif: `db.setting("allmenuVariant") || config.ui?.allmenuVariant
/// || 2` (`allmenu.js:231-232`).
fn variant_aktif(ctx: &Ctx) -> i64 {
    ctx.setting_i64(SETTING_VARIANT)
        .filter(|varian| *varian > 0)
        .unwrap_or(DEFAULT_VARIANT)
}

/// Body kartu varian 2 (`allmenu.js:254`).
fn body_varian_2(ctx: &Ctx) -> String {
    format!(
        "> Halo *{}* 👋 Perkenalkan aku *{}*, buatan *{}* Saya siap membantu kebutuhanmu, mulai dari download video, main game, tanya jawab, nyari info/sesuatu, bikin sticker, dan lain-lain.\n\n",
        nama_pengguna(ctx),
        nama_bot(ctx),
        nama_dev(ctx)
    )
}

/// Varian 2 — kartu interaktif dengan header gambar `foto`, footer berisi
/// seluruh daftar command, dan tombol "Kembali Ke Menu Utama"
/// (`allmenu.js:238-293`).
fn kartu_varian_2(ctx: &Ctx, txt: &str) -> Option<Reply> {
    let aset = ctx.config().assets.get("foto")?;
    if !ctx.berkas().ada(aset) {
        return None;
    }
    let params = serde_json::json!({
        "limited_time_offer": {
            "text": sapa(),
            "url": "Hai",
            "copy_code": format!("Dibuat oleh {}", nama_dev(ctx)),
            "expiration_time": crate::common::now_ms() + 1_000_000,
        }
    });

    Some(
        Reply::interactive(body_varian_2(ctx))
            .with_footer(txt.to_string())
            .with_header_card(
                CardHeader::new()
                    .title("")
                    .subtitle("")
                    .image(MediaSource::path(aset)),
            )
            .with_context(konteks_varian_2(ctx))
            .with_message_params(params)
            .with_button_card(InteractiveButton::quick_reply(
                TOMBOL_MENU_UTAMA,
                format!("{}menu", ctx.prefix),
            )),
    )
}

/// Varian 5 — kartu native-flow dengan header video `mp4`, tombol native
/// (kosong + daftar kategori + menu utama), dan pesan yang dirujuk berupa
/// lokasi cuaca (`allmenu.js:295-358`).
fn kartu_varian_5(ctx: &Ctx, txt: &str) -> Option<Reply> {
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
    // `qOrder` (`allmenu.js:315-318`) — pesan palsu berisi lokasi cuaca.
    context = context.quote(CardQuote::Location {
        latitude: 0.0,
        longitude: 0.0,
        name: crate::main::menu::cuaca(ctx, KOTA_CUACA),
        thumbnail: thumbnail(ctx),
    });

    Some(
        Reply::interactive(txt.to_string())
            .with_footer(FOOTER_VARIAN_5)
            .with_header_card(
                CardHeader::new()
                    .title("")
                    .subtitle("")
                    .video(MediaSource::path(aset)),
            )
            .with_context(context)
            .with_message_params(params)
            // Slot kosong pertama (`allmenu.js:343`) — WA butuh minimal satu
            // tombol yang tidak berisi aksi.
            .with_button_card(InteractiveButton::empty(""))
            .with_button_card(InteractiveButton::quick_reply(
                TOMBOL_DAFTAR_KATEGORI,
                format!("{}menucat", ctx.prefix),
            ))
            .with_button_card(InteractiveButton::quick_reply(
                TOMBOL_MENU_UTAMA,
                format!("{}menu", ctx.prefix),
            )),
    )
}

/// Varian 6 — kartu dengan header lokasi (nama bot + alamat cuaca), tanpa
/// tombol (`allmenu.js:360-414`).
fn kartu_varian_6(ctx: &Ctx, txt: &str) -> Option<Reply> {
    let header = CardHeader::new().location(CardLocation {
        latitude: 0.0,
        longitude: 0.0,
        name: nama_bot(ctx),
        address: Some(crate::main::menu::cuaca(ctx, KOTA_CUACA)),
        thumbnail: thumbnail(ctx),
    });

    let mut context = konteks_saluran(ctx);
    // `wrapInteractive(..., { quoted: m })` (`allmenu.js:411`) — kartu
    // mengutip pesan pemicu.
    context = context.quote_trigger();

    Some(
        Reply::interactive(txt.to_string())
            .with_header_card(header)
            .with_context(context),
    )
}

/// Jalur cadangan `catch` (`allmenu.js:592-606`): gambar + caption kalau
/// aset `foto` ada, kalau tidak teks polos.
///
/// `contextInfo: getContextInfo(botConfig, m)` (`allmenu.js:600`) tidak bisa
/// ditempelkan utuh: `Reply::Image` di API Rust tidak punya `contextInfo`.
/// Yang bisa dibawa adalah `mentionedJid: [m.sender]`, jadi itu yang
/// diambil dari konteks itu; `forwarded` + surat berita hilang (lihat
/// `ponytail:` di modul).
fn cadangan(ctx: &Ctx, txt: &str) -> Reply {
    let context = konteks_info(ctx);
    if let Some(aset) = ctx.config().assets.get("foto")
        && ctx.berkas().ada(aset)
    {
        return Reply::Image {
            media: Some(MediaSource::path(aset)),
            caption: txt.to_string(),
            mentions: context.mentioned.clone(),
        };
    }
    Reply::text(txt.to_string())
}

// ---------------------------------------------------------------------------
// Audio menu
// ---------------------------------------------------------------------------

/// `db.setting("audioMenu") !== false` (`allmenu.js:419`).
///
/// Hanya `false` yang mematikan; bentuk lain (termasuk tidak ada) lolos.
fn audio_aktif(ctx: &Ctx) -> bool {
    !matches!(
        ctx.setting(SETTING_AUDIO),
        Some(serde_json::Value::Bool(false))
    )
}

/// Audio menu dari aset `mp3` (`allmenu.js:421` dan `:523`).
fn audio_menu(ctx: &Ctx) -> Option<Reply> {
    let aset = ctx.config().assets.get("mp3")?;
    Some(Reply::audio(MediaSource::path(aset)))
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

#[plugin(
    name = "allmenu",
    alias = ["fullmenu", "allcommand", "semua"],
    category = "main",
    description = "Menampilkan semua command lengkap per kategori",
    usage = ".allmenu",
    example = ".allmenu",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false
)]
pub fn allmenu(ctx: Ctx) -> Reply {
    // `botConfig.command?.prefix || "."` (`allmenu.js:106`).
    let prefix = if ctx.prefix.is_empty() {
        ".".to_string()
    } else {
        ctx.prefix.clone()
    };
    let txt = teks_menu(&ctx, &prefix);

    // `switch (allmenuVariant)` (`allmenu.js:234-418`).
    let isi = match variant_aktif(&ctx) {
        1 => Reply::text(txt.clone()),
        2 => kartu_varian_2(&ctx, &txt).unwrap_or_else(|| cadangan(&ctx, &txt)),
        5 => kartu_varian_5(&ctx, &txt).unwrap_or_else(|| cadangan(&ctx, &txt)),
        6 => kartu_varian_6(&ctx, &txt).unwrap_or_else(|| cadangan(&ctx, &txt)),
        // `default: break` (`allmenu.js:416-417`) — tidak ada yang dikirim.
        _ => Reply::None,
    };

    // Audio menu dikirim SETELAH varian (`allmenu.js:419-591`).
    match audio_menu(&ctx) {
        Some(audio) if audio_aktif(&ctx) => match isi {
            Reply::None => audio,
            isi => Reply::sequence(vec![isi, audio]),
        },
        _ => isi,
    }
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
            ("mp3".to_string(), "./assets/audio/mp3.mp3".to_string()),
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
    /// kartu bisa dibangun. Audio menu aktif (default JS).
    fn ctx_dengan_aset(variant: i64) -> Ctx {
        let berkas = BerkasPalsu::baru();
        berkas.tulis("./assets/image/foto.jpeg", &[1, 2, 3]).unwrap();
        berkas.tulis("./assets/image/foto2.jpeg", &[1, 2, 3]).unwrap();
        berkas.tulis("./assets/video/mp4.mp4", &[1, 2, 3]).unwrap();
        Ctx {
            prefix: ".".into(),
            command: "allmenu".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            push_name: "Oktz".into(),
            is_owner: true,
            settings: serde_json::json!({ "allmenuVariant": variant }),
            config: Arc::new(runtime()),
            berkas: Arc::new(berkas),
            ..Ctx::default()
        }
    }

    /// Konteks dengan audio menu dimatikan, jadi balasannya satu pesan
    /// dan bisa diperiksa langsung.
    fn ctx(variant: i64) -> Ctx {
        let mut c = ctx_dengan_aset(variant);
        c.settings = serde_json::json!({ "allmenuVariant": variant, "audioMenu": false });
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

    // -- meta ---------------------------------------------------------------

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("allmenu").unwrap();
        assert_eq!(p.meta.name, "allmenu");
        assert_eq!(
            p.meta
                .alias
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["fullmenu", "allcommand", "semua"]
        );
        assert_eq!(p.meta.category, "main");
        assert_eq!(
            p.meta.description,
            "Menampilkan semua command lengkap per kategori"
        );
        assert_eq!(p.meta.usage, ".allmenu");
        assert_eq!(p.meta.example, ".allmenu");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_ke_allmenu() {
        for alias in ["fullmenu", "allcommand", "semua"] {
            let found = oktz_plugin_api::find(alias).unwrap_or_else(|| panic!("alias {alias} hilang"));
            assert_eq!(found.meta.name, "allmenu", "alias {alias} salah resolve");
        }
    }

    // -- teks ---------------------------------------------------------------

    /// `createBracketBox` JS (`allmenu.js:70-77`) — emoji tidak masuk ke
    /// badan fungsi, judulnya cuma `${title} MENU `.
    #[test]
    fn bracket_box_punya_bentuk_yang_sama_dengan_js() {
        let baris = vec![".menu".to_string(), ".tools".to_string()];
        assert_eq!(
            kotak("🏠", "MAIN", &baris),
            "╭➤--「MAIN MENU 」\n┆ ⇝ .menu\n┆ ⇝ .tools\n╰➤-------------------------------\n\n"
        );
    }

    #[test]
    fn bracket_box_tanpa_baris_tetap_punya_footer() {
        assert_eq!(
            kotak("📋", "CEK", &[]),
            "╭➤--「CEK MENU 」\n╰➤-------------------------------\n\n"
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
        // `🅛` tidak ada — `PluginMeta` tidak punya field limit.
        assert_eq!(simbol(&meta), " 🅞 🅟 🅐 🅖 🅟🅡");
        assert_eq!(simbol(&PluginMeta::default()), "");
        assert_eq!(
            simbol(&PluginMeta {
                is_group: true,
                ..PluginMeta::default()
            }),
            " 🅖"
        );
    }

    #[test]
    fn emoji_kategori_memakai_peta_dari_js() {
        assert_eq!(emoji_kategori("main"), "🏠");
        assert_eq!(emoji_kategori("pushkontak"), "📱");
        assert_eq!(emoji_kategori("store"), "🏪");
        // Kategori tak dikenal → cadangan `📋` (`allmenu.js:223`).
        assert_eq!(emoji_kategori("tidakada"), "📋");
    }

    #[test]
    fn teks_menu_memuat_blok_informasi_dan_kotak_kategori() {
        let t = teks(&allmenu(ctx(1)));
        assert!(t.contains("✦ *Informasi User*"), "{t}");
        assert!(t.contains("• Nama   : Oktz"), "{t}");
        assert!(t.contains("• Status : Owner"), "{t}");
        assert!(t.contains("• Limit  : ∞ Unlimited"), "{t}");
        assert!(t.contains("✦ *Informasi Bot*"), "{t}");
        assert!(t.contains("• Nama   : Oktz Bot"), "{t}");
        assert!(t.contains(" fitur"), "{t}");
        assert!(t.contains("• Versi  : 3.3.1"), "{t}");
        assert!(t.contains(" WIB"), "{t}");
        assert!(t.contains("✦ *Kelebihan Bot Ini*"), "{t}");
        assert!(t.contains("• Fast response"), "{t}");
        assert!(t.contains("Silakan pilih menu di bawah ini."), "{t}");
        // Kotak kategori: judul uppercase + emoji.
        assert!(t.contains("╭➤--「MAIN MENU 」"), "{t}");
        assert!(t.contains("┆ ⇝ .menu"), "{t}");
        assert!(t.contains("╰➤-------------------------------"), "{t}");
    }

    #[test]
    fn kategori_owner_tidak_muncul_untuk_non_owner() {
        let mut c = ctx(1);
        c.is_owner = false;
        let t = teks(&allmenu(c));
        assert!(!t.contains("╭➤--「OWNER MENU 」"), "{t}");
    }

    #[test]
    fn mode_cpanel_membatasi_kategori_yang_tampil() {
        let mut c = ctx(1);
        c.is_group = true;
        c.group_data = serde_json::json!({ "botMode": "cpanel" });
        let t = teks(&allmenu(c));
        assert!(t.contains("╭➤--「MAIN MENU 」"), "{t}");
        assert!(t.contains("╭➤--「STICKER MENU 」"), "{t}");
        // `utility` tidak ada di `allowedCategories` mode cpanel.
        assert!(!t.contains("╭➤--「UTILITY MENU 」"), "{t}");
    }

    #[test]
    fn variant_default_dua_bila_setting_tidak_ada() {
        // Setting yang ada dipakai apa adanya.
        assert_eq!(variant_aktif(&ctx(1)), 1);
        // Setting 0 falsy di JS (`||`), jadi bukan varian 0.
        let mut c = ctx(1);
        c.settings = serde_json::json!({ "allmenuVariant": 0 });
        assert_eq!(variant_aktif(&c), DEFAULT_VARIANT);
        let mut c = ctx(1);
        c.settings = serde_json::json!({ "allmenuVariant": 5 });
        assert_eq!(variant_aktif(&c), 5);
        // Tanpa setting sama sekali → `config.ui?.allmenuVariant || 2`.
        let mut c = ctx(1);
        c.settings = serde_json::json!({});
        assert_eq!(variant_aktif(&c), DEFAULT_VARIANT);
    }

    // -- kartu --------------------------------------------------------------

    #[test]
    fn varian_satu_mengirim_teks_polos() {
        let r = allmenu(ctx(1));
        assert!(matches!(r.without_mutations(), Reply::Text { .. }));
    }

    #[test]
    fn varian_dua_membawa_header_gambar_footer_dan_tombol() {
        let r = allmenu(ctx(2));
        let (body, footer, buttons) = kartu(&r);
        assert!(body.starts_with("> Halo *Oktz* 👋"), "{body}");
        let footer = footer.expect("footer wajib ada");
        assert!(footer.contains("✦ *Informasi User*"), "{footer}");
        assert!(footer.contains("╭➤--「MAIN MENU 」"), "{footer}");
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].name, "quick_reply");
        assert_eq!(buttons[0].params["display_text"], TOMBOL_MENU_UTAMA);
        assert_eq!(buttons[0].params["id"], ".menu");

        let Reply::Interactive { kartu } = r.without_mutations()
        else {
            panic!("harus kartu");
        };
        let header = kartu.header_card.as_ref().expect("header wajib ada");
        match header.media.as_ref().expect("media wajib ada") {
            oktz_plugin_api::registry::CardMedia::Image { source } => {
                assert_eq!(source.label(), "path");
            }
            other => panic!("harus gambar, dapat {other:?}"),
        }
        let params = kartu.params.as_ref().expect("params wajib ada");
        assert_eq!(params["limited_time_offer"]["url"], "Hai");
        assert_eq!(params["limited_time_offer"]["copy_code"], "Dibuat oleh Zann");

        let context = kartu.context_card.as_ref().expect("context wajib ada");
        assert!(context.is_forwarded);
        assert_eq!(context.mentioned, vec![SENDER.to_string()]);
        assert_eq!(context.participant.as_deref(), Some("0@s.whatsapp.net"));
        match context.quote.as_deref() {
            Some(oktz_plugin_api::registry::CardQuote::Text(teks)) => {
                assert_eq!(teks, "Oktz Bot");
            }
            other => panic!("harus quote teks, dapat {other:?}"),
        }
    }

    #[test]
    fn varian_lima_membawa_video_tombol_kosong_dan_quote_lokasi() {
        let r = allmenu(ctx(5));
        let (body, footer, buttons) = kartu(&r);
        assert!(body.contains("✦ *Informasi User*"), "{body}");
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
        let params = kartu.params.as_ref().expect("params wajib ada");
        assert_eq!(params["bottom_sheet"]["list_title"], "Please select the menu");
        assert_eq!(params["tap_target_configuration"]["domain"], "shop.example.com");
        assert_eq!(params["limited_time_offer"]["url"], "Hai");

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
                // Jaringan gagal di test → teks cadangan open-meteo.
                assert_eq!(name, "Cuaca tidak tersedia");
            }
            other => panic!("harus quote lokasi, dapat {other:?}"),
        }
    }

    #[test]
    fn varian_enam_membawa_header_lokasi_tanpa_tombol() {
        let r = allmenu(ctx(6));
        let (body, footer, buttons) = kartu(&r);
        assert!(body.contains("✦ *Informasi User*"), "{body}");
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
                assert_eq!(loc.address.as_deref(), Some("Cuaca tidak tersedia"));
            }
            other => panic!("harus lokasi, dapat {other:?}"),
        }
        let context = kartu.context_card.as_ref().expect("context wajib ada");
        assert!(context.quote_trigger, "varian 6 mengutip pesan pemicu");
    }

    #[test]
    fn aset_yang_hilang_menjatuh_ke_teks_polos() {
        let mut c = ctx(2);
        c.berkas = Arc::new(BerkasPalsu::baru());
        let r = allmenu(c);
        // Aset `foto` hilang → tidak ada gambar cadangan → teks polos.
        assert!(matches!(r.without_mutations(), Reply::Text { .. }));
        assert!(teks(&r).contains("✦ *Informasi User*"));
    }

    // -- audio menu ---------------------------------------------------------

    #[test]
    fn audio_menu_menempel_setelah_kartu() {
        // Tanpa `audioMenu: false` → audio ikut (default JS `!== false`).
        let r = allmenu(ctx_dengan_aset(2));
        let Reply::Sequence(list) = r.without_mutations() else {
            panic!("harus sequence, dapat {:?}", r.without_mutations());
        };
        assert_eq!(list.len(), 2);
        assert!(matches!(list[0].without_mutations(), Reply::Interactive { .. }));
        match list[1].without_mutations() {
            Reply::Audio { media, ptt } => {
                assert!(!ptt, "MP3 dikirim sebagai audio, bukan voice note");
                match media.as_ref().expect("media wajib ada") {
                    MediaSource::Path { path, .. } => {
                        assert_eq!(path, "./assets/audio/mp3.mp3");
                    }
                    other => panic!("harus path, dapat {other:?}"),
                }
            }
            other => panic!("harus audio, dapat {other:?}"),
        }
    }

    #[test]
    fn audio_menu_bisa_dimatikan_dengan_setting_false() {
        let mut c = ctx_dengan_aset(2);
        c.settings = serde_json::json!({ "allmenuVariant": 2, "audioMenu": false });
        let r = allmenu(c);
        assert!(matches!(r.without_mutations(), Reply::Interactive { .. }));
    }

    #[test]
    fn audio_menu_aktif_secara_bawaan() {
        assert!(audio_aktif(&Ctx::default()));
        // `!== false` — bentuk lain tetap aktif.
        for nilai in [
            serde_json::json!(true),
            serde_json::json!(0),
            serde_json::json!(""),
            serde_json::json!(null),
        ] {
            assert!(
                audio_aktif(&Ctx {
                    settings: serde_json::json!({ "audioMenu": nilai }),
                    ..Ctx::default()
                }),
                "nilai non-false tidak mematikan audio"
            );
        }
    }

    // -- konteks ------------------------------------------------------------

    #[test]
    fn konteks_info_mirip_get_context_info_js() {
        let c = ctx(1);
        let context = konteks_info(&c);
        assert!(context.is_forwarded);
        assert_eq!(context.forwarding_score, 9999);
        assert_eq!(context.mentioned, vec![SENDER.to_string()]);
        let newsletter = context
            .forwarded_newsletter
            .as_ref()
            .expect("surat berita wajib ada");
        assert_eq!(newsletter.jid, "120363411617217441@newsletter");
        assert_eq!(newsletter.name, "Join saluran resmi Oktz");
        assert_eq!(newsletter.server_message_id, 127);
    }
}
