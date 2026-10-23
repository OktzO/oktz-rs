//! Pipeline live: pesan inbound → `oktz_router::Router` → balasan ke WhatsApp.
//!
//! Tiga bagian, dipisah supaya bagian yang bisa diuji offline tetap murni:
//!
//! 1. **Normalisasi JID** ([`normalize_user_key`]) — port `getUser`
//!    (`src/lib/database.js:529-536`) dan pembantu `isLidLikeJid`
//!    (`database.js:7-13`, `lib/lid.js:124-167`).
//! 2. **Adapter store** ([`StoreContext`]) — `RouterContext` di atas
//!    `oktz_store::JsonStore`. `oktz-store` tidak punya `getUser`/`getGroup`/
//!    `setting` (itu hanya di JS), jadi peta JSON dibaca langsung di sini.
//! 3. **Pipeline** ([`Pipeline`]) — dedup → gate → route → render → kirim.
//!
//! Batas yang diketahui dan sengaja dibiarkan terlihat, bukan disembunyikan:
//! - `DenyMode` tidak membawa payload, jadi pesan AFK/`onlyThisGroup` tidak
//!   bisa menginterpolasi `botAfk.reason`/durasi/`onlyThisGroup.link` seperti
//!   JS. Yang dikirim adalah kalimat yang benar tanpa data yang dikarang.
//! - Kuota auto-work TIDAK dibakar per-pesan. Di JS `allowAutoWork()` dipanggil
//!   fitur tertentu saja (`handler.js:984, 1129, 1136, 1143, 1149, 1422`), dan
//!   `Reply` tidak punya field yang menandai konsumsi itu — membakarnya di sini
//!   akan menguras kuota untuk command biasa. Gate-nya tetap tersedia lewat
//!   [`Pipeline::allow_auto_work`].
//!
//! Media: `Reply::Image` / `Sticker` / `Video` / `Audio` / `Document` membawa
//! [`MediaSource`]. Tidak ada "kirim dari URL" di whatsapp-rust — `Client::upload`
//! (`whatsapp-rust-0.7.0/src/upload.rs:409`) hanya menerima `Vec<u8>`, jadi
//! `Url`/`Path` di-resolve ke byte dulu ([`load_media`]) sebelum upload.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::{Instant, SystemTime};

use oktz_plugin_api::berkas::Berkas;
use oktz_plugin_api::registry::{
    CardContext, CardHeader, CardLocation, CardMedia, CardQuote, GroupRoster, HttpGet, MediaSource,
    Mutation, Participant,
};
use oktz_plugin_api::{NoHttp, QuotedKind, QuotedMessage, SessionStore, new_session_store};
use oktz_plugin_api::{Ctx, PluginEntry, Reply};
use oktz_plugin_api::{Sock, SockMati};
use oktz_router::gates::{
    AutoWorkDecision, AutoWorkGuard, AutoWorkSettings, EnergiSettings, resolve_energi_enabled,
};
use oktz_router::group::{self, GateEnv, GroupCaller};
use oktz_router::serialize::{MessageKind, MessageParams, get_message_body, resolve_message_type};
use oktz_router::{
    AccessGrant, Caller, Decision, DenyMode, GroupOverrides, GroupState, ModeSettings,
    ParseOptions, RateLimiter, Router, RouterContext, UserState, consume_auto_work, is_spamming,
    parse_command,
};
use oktz_store::json_store::JsonStore;
use serde_json::Value;
use crate::grup_event::{self, AksiGrup, UpdateGrup};
use crate::pesan_grup::{self, KonteksPesan};
use crate::reply_variant;
use whatsapp_rust::RevokeType;
use whatsapp_rust::http::{HttpClient, HttpRequest, UreqHttpClient};
use whatsapp_rust::media::{
    AudioOptions, DocumentOptions, ImageOptions, VideoOptions, audio_message, document_message,
    image_message, video_message,
};
use whatsapp_rust::prelude::*;
use whatsapp_rust::GroupMetadata;
use whatsapp_rust::upload::{UploadOptions, UploadResponse};
use whatsapp_rust::waproto::buffa::MessageField;
use whatsapp_rust::waproto::whatsapp::__buffa::oneof::message::interactive_response_message::InteractiveResponseMessage as IrOneof;
use whatsapp_rust::wacore::download::MediaType;
use whatsapp_rust::wacore_binary::Node;

use crate::cards::{Button, ButtonAction, Card, HeaderMedia, Header, biz_node};
use crate::pipeline_gate::{self, GateState};

// =============================================================================
// 1. Normalisasi JID — port `getUser` (database.js:529-536)
// =============================================================================

/// `VALID_COUNTRY_CODES` (`src/lib/lid.js:136-148`).
const VALID_COUNTRY_CODES: &[&str] = &[
    "1", "7", "20", "27", "30", "31", "32", "33", "34", "36", "39", "40", "41", "43", "44", "45",
    "46", "47", "48", "49", "51", "52", "53", "54", "55", "56", "57", "58", "60", "61", "62", "63",
    "64", "65", "66", "81", "82", "84", "86", "90", "91", "92", "93", "94", "95", "98", "212", "213",
    "216", "218", "220", "221", "234", "249", "254", "255", "256", "260", "263", "351", "352",
    "353", "354", "355", "356", "357", "358", "359", "370", "371", "372", "373", "374", "375",
    "376", "377", "378", "380", "381", "382", "383", "385", "386", "387", "389", "420", "421",
    "423", "852", "853", "855", "856", "880", "886", "960", "961", "962", "963", "964", "965",
    "966", "967", "968", "970", "971", "972", "973", "974", "975", "976", "977", "992", "993",
    "994", "995", "996", "998",
];

/// Port `isLidConverted` (`src/lib/lid.js:150-167`).
pub fn is_lid_converted(jid: &str) -> bool {
    let Some(number) = jid.strip_suffix("@s.whatsapp.net") else {
        return false;
    };
    if number.len() > 14 {
        return true;
    }
    for len in 1..=3usize {
        let Some(prefix) = number.get(..len) else {
            continue;
        };
        if VALID_COUNTRY_CODES.contains(&prefix)
            && number.len() >= len + 6
            && number.len() <= len + 12
        {
            return false;
        }
    }
    true
}

/// Port `isLidLikeJid` (`src/lib/database.js:7-13`).
pub fn is_lid_like_jid(jid: &str) -> bool {
    if jid.is_empty() {
        return false;
    }
    if jid.ends_with("@lid") {
        return true;
    }
    if !is_lid_converted(jid) {
        return false;
    }
    let Some(number) = jid.strip_suffix("@s.whatsapp.net") else {
        return false;
    };
    // JS `/^\d+$/.test(number) && number.length >= 10`
    !number.is_empty() && number.len() >= 10 && number.bytes().all(|b| b.is_ascii_digit())
}

/// Kunci `users[]` untuk sebuah JID, atau `None` kalau `getUser` mengembalikan
/// `null` (`database.js:529-536`).
///
/// `jid.replace(/@.+/g, "")` di JS = semua sebelum `@` pertama, jadi
/// `split('@').next()`. JID `@lid` — dan nomor LID yang sudah dikonversi —
/// memakai prefiks `lid:` pada kunci; sisanya ditolak kalau terlalu panjang
/// atau diawali `120` (nomor grup).
pub fn normalize_user_key(jid: &str) -> Option<String> {
    if jid.is_empty() {
        return None;
    }
    let is_lid = is_lid_like_jid(jid);
    let clean = jid.split('@').next().unwrap_or(jid);
    if !is_lid && (clean.len() > 15 || clean.starts_with("120")) {
        return None;
    }
    Some(if is_lid {
        format!("lid:{clean}")
    } else {
        clean.to_string()
    })
}

// =============================================================================
// 2. Adapter store
// =============================================================================

fn top<'a>(store: &'a JsonStore, key: &str) -> Option<&'a Value> {
    store.get(key)
}

/// `normalizeNumber` (`config.js:346-352`) — buang suffix `:device` dan
/// `@domain`, lalu sisakan digit saja.
fn normalize_number(v: &str) -> String {
    let v = v.split(':').next().unwrap_or(v);
    let v = v.split('@').next().unwrap_or(v);
    v.chars().filter(char::is_ascii_digit).collect()
}

/// `matchesNumber` (`config.js:357-362`) — strict equality setelah
/// normalisasi. BUKAN `contains`/`ends_with`: nomor yang kebetulan memuat
/// digit owner lain tidak boleh lolos (privilege escalation).
fn matches_number(a: &str, b: &str) -> bool {
    let na = normalize_number(a);
    let nb = normalize_number(b);
    !na.is_empty() && na == nb
}

/// `new Date(str).getTime()` — parser ISO-8601 minimal untuk field
/// `expiredAt` (`config.js:442`). Format yang dipakai: `YYYY-MM-DDTHH:MM:SS`
/// opsional pecahan detik dan `Z`/offset.
fn iso_to_ms(s: &str) -> Option<i64> {
    let s = s.trim();
    let (date, rest) = s.split_once('T')?;
    let mut date_parts = date.split('-');
    let y: i64 = date_parts.next()?.parse().ok()?;
    let mo: i64 = date_parts.next()?.parse().ok()?;
    let d: i64 = date_parts.next()?.parse().ok()?;
    let time = rest.split(['Z', '+']).next()?;
    let mut time_parts = time.split(':');
    let h: i64 = time_parts.next()?.parse().ok()?;
    let mi: i64 = time_parts.next()?.parse().ok()?;
    let sec: f64 = time_parts
        .next()
        .unwrap_or("0")
        .split_once('.')
        .map(|(a, _)| a.parse().ok())
        .unwrap_or(Some(0.0))?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    // Hari sejak epoch (algoritma Howard Hinnant, civil-from-days terbalik).
    let y_adj = if mo <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = y_adj - era * 400;
    let mp = (mo + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(((days * 24 + h) * 60 + mi) * 60_000 + (sec * 1000.0) as i64)
}

/// Entri koleksi `premium`/`partner` — `config.js:440-450`:
/// string (tanpa masa berlaku) atau objek `{ id, expired | expiredAt }`.
///
/// `expired` dibaca sebagai angka milidetik; `expiredAt` sebagai tanggal ISO.
/// String non-numerik di `expired` diabaikan, sama seperti hasil perbandingan
/// longgar di JS (`"2025-01-01" < now` → false → dianggap belum kedaluwarsa).
fn entry_is_active(entry: &Value, jid: &str, now_ms: i64) -> bool {
    let matched = match entry {
        Value::String(s) => matches_number(s, jid),
        Value::Object(_) => entry
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| matches_number(id, jid)),
        _ => false,
    };
    if !matched {
        return false;
    }
    if matches!(entry, Value::String(_)) {
        return true;
    }
    let expire = entry
        .get("expired")
        .and_then(Value::as_i64)
        .or_else(|| {
            entry
                .get("expiredAt")
                .and_then(Value::as_str)
                .and_then(iso_to_ms)
        });
    // `if (expireTime && expireTime < now)` — 0 dianggap "tidak ada masa
    // berlaku".
    !matches!(expire, Some(e) if e != 0 && e < now_ms)
}

/// Objek settings — dua layout file yang harus sama-sama jalan.
///
/// Layout Rust (merged store, `src/main.rs`): `{"settings": {...}, "users":
/// {...}, "groups": {...}}` — semua koleksi dalam satu file.
///
/// Layout JS (`database/main/settings.json` hasil salinan bot JS): file itu
/// ISi koleksi settings, TANPA pembungkus — `{"replyVariant": 9, ...}`.
///
/// Kalau key `"settings"` ada dan berupa objek → itu yang dipakai. Kalau
/// tidak → seluruh file dianggap settings. Tanpa fallback ini, database hasil
/// salinan JS dibaca sebagai "settings tidak ada", sehingga `replyVariant`,
/// `botMode`, `energi`, dan `notiflimit` diam-diam kembali ke default —
/// persis gejala "replyVariant tidak jalan" di bot Rust.
fn settings_obj(store: &JsonStore) -> Value {
    match store.get("settings") {
        Some(s) if s.is_object() => s.clone(),
        _ => {
            let all = store.all();
            if all.is_object() { all.clone() } else { Value::Null }
        }
    }
}

/// Grant akses dari `user.access[]` (`middleware.js:38-52`).
///
/// `expired` punya tiga kasus dan bedanya berarti:
/// `null` → tidak pernah kedaluwarsa, **hilang** → `undefined` di JS, yang gagal
/// di `=== null` DAN gagal di `> Date.now()` sehingga dianggap SUDAH kedaluwarsa.
fn access_grants(user: &Value) -> Vec<AccessGrant> {
    let Some(arr) = user.get("access").and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|g| {
            let cmd = g.get("cmd").and_then(Value::as_str)?;
            let expired = match g.get("expired") {
                None => Some(0),
                Some(Value::Null) => None,
                Some(v) => Some(v.as_i64().unwrap_or(0)),
            };
            Some(AccessGrant {
                cmd: cmd.to_string(),
                expired,
            })
        })
        .collect()
}

/// `users[key]` → [`UserState`]. `key` harus sudah lewat [`normalize_user_key`].
pub fn user_state_from(users: Option<&Value>, key: &str) -> UserState {
    let Some(user) = users.and_then(|u| u.get(key)) else {
        return UserState::default();
    };
    UserState {
        access: access_grants(user),
        is_registered: user
            .get("isRegistered")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

/// `groups[chat]` → `game` / `rpg`.
///
/// `getGroup` (`database.js:711-714`) TIDAP menormalisasi JID — kunci-nya JID
/// mentah. `None` berarti grup tidak ada, dan `permission.rs:44-50` memperlakukannya
/// sebagai "aturan 8/9 dilewati", bukan ditolak.
pub fn group_state_from(groups: Option<&Value>, chat: &str) -> Option<GroupState> {
    let g = groups?.get(chat)?;
    Some(GroupState {
        game: g.get("game").and_then(Value::as_bool),
        rpg: g.get("rpg").and_then(Value::as_bool),
    })
}

/// `groups[chat].botMode`, default `"all"` (`handler.js:1517`, `handler.js:1711`).
pub fn group_bot_mode_from(groups: Option<&Value>, chat: &str) -> String {
    groups
        .and_then(|g| g.get(chat))
        .and_then(|g| g.get("botMode"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("all")
        .to_string()
}

/// `settings.onlyThisGroup` — string atau `{ jid, ... }` (`middleware.js:180-196`).
fn only_this_group_jid(settings: Option<&Value>) -> Option<String> {
    match settings?.get("onlyThisGroup")? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Object(_) => settings?
            .get("onlyThisGroup")?
            .get("jid")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

/// `onlyThisGroup.name` / `.link` (`middleware.js:183,185`). String biasa tidak
/// punya keduanya, jadi kosong — sama seperti JS yang memakai `*undefined*`.
fn only_this_group_field(settings: Option<&Value>, key: &str) -> String {
    settings
        .and_then(|s| s.get("onlyThisGroup"))
        .and_then(|v| v.get(key))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn str_list(settings: Option<&Value>, key: &str) -> Vec<String> {
    settings
        .and_then(|s| s.get(key))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn flag(settings: Option<&Value>, key: &str) -> bool {
    settings
        .and_then(|s| s.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// `settings` → [`ModeSettings`]. Kunci persis `middleware.js:143-241`.
pub fn mode_settings_from(settings: Option<&Value>) -> ModeSettings {
    ModeSettings {
        // `db.setting("botMode") || config.mode || "public"` — string kosong
        // dibiarkan kosong; `mode::check_mode` (mode.rs:73-77) sudah fallback
        // ke "public".
        mode: settings
            .and_then(|s| s.get("botMode"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        afk_active: settings
            .and_then(|s| s.get("botAfk"))
            .and_then(|v| v.get("active"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        afk_reason: settings
            .and_then(|s| s.get("botAfk"))
            .and_then(|v| v.get("reason"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        afk_since_ms: settings
            .and_then(|s| s.get("botAfk"))
            .and_then(|v| v.get("since"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        only_this_group_name: only_this_group_field(settings, "name"),
        only_this_group_link: only_this_group_field(settings, "link"),
        only_gc: flag(settings, "onlyGc"),
        only_pc: flag(settings, "onlyPc"),
        only_this_group: only_this_group_jid(settings),
        self_groups: str_list(settings, "selfGroups"),
        public_groups: str_list(settings, "publicGroups"),
        self_admin: flag(settings, "selfAdmin"),
        public_admin: flag(settings, "publicAdmin"),
        only_admin: flag(settings, "onlyAdmin"),
    }
}

/// `groups[chat].{selfAdmin,publicAdmin,onlyAdmin}` — `None` per field berarti
/// "tidak ada override", jadi `settings` global yang dipakai
/// (`middleware.js:151-156, 240-242`).
pub fn group_overrides_from(groups: Option<&Value>, chat: &str) -> Option<GroupOverrides> {
    let g = groups?.get(chat)?;
    let f = |k: &str| g.get(k).and_then(Value::as_bool);
    Some(GroupOverrides {
        self_admin: f("selfAdmin"),
        public_admin: f("publicAdmin"),
        only_admin: f("onlyAdmin"),
    })
}

/// `settings.capprem[name]` (`middleware.js:69`). `Some(false)` berarti override
/// implisit dan HARUS menang atas `meta.is_premium`.
pub fn capprem_from(settings: Option<&Value>, name: &str) -> Option<bool> {
    settings?
        .get("capprem")?
        .get(name)?
        .as_bool()
}

/// File-file store yang dipakai bot JS: satu file per koleksi
/// (`database.js:127-136` — `fileMap`: `users.json`, `groups.json`,
/// `settings.json`, `premium.json`, `owner.json`, `partner.json`).
///
/// Rust sendiri menaruh semua koleksi di satu `settings.json`. Kalau folder
/// database punya file JS, file-file itu yang jadi sumber kebenaran:
/// `groups.json` memuat flag group-protection (`antilink`, `antiswgc`, ...)
/// yang TIDAK ada di `settings.json`. Tanpa pembacaan ini semua proteksi
/// grup mati di deployment yang database-nya disalin dari bot JS.
#[derive(Debug, Clone)]
struct StoreFiles {
    /// `settings.json` — isi settings, atau seluruh store (layout Rust).
    primary: PathBuf,
    /// Koleksi JS di samping file utama. `None` → layout Rust (satu file).
    users: Option<PathBuf>,
    groups: Option<PathBuf>,
    premium: Option<PathBuf>,
    owner: Option<PathBuf>,
    partner: Option<PathBuf>,
}

/// Koleksi yang punya file sendiri di layout JS.
const KOLEKSI_JS: [&str; 5] = ["users", "groups", "premium", "owner", "partner"];

impl StoreFiles {
    /// Deteksi layout dari file utama.
    ///
    /// Syaratnya file utama bernama `settings.json` — nama file store bot JS
    /// (`database.js:129`). File lain yang kebetulan ada di folder yang sama
    /// (mis. file test) tidak boleh mengubah layout store.
    fn detect(primary: PathBuf) -> Self {
        let dir = primary.parent().map(Path::to_path_buf).unwrap_or_default();
        let bernama_settings = primary
            .file_name()
            .map(|n| n == "settings.json")
            .unwrap_or(false);
        let side = |name: &str| -> Option<PathBuf> {
            if !bernama_settings {
                return None;
            }
            let p = dir.join(name);
            p.exists().then_some(p)
        };
        let users = side("users.json");
        let groups = side("groups.json");
        // Layout JS kalau ada file KOLEKSI di samping `settings.json`.
        let js = users.is_some() || groups.is_some();
        Self {
            primary,
            // File yang belum ada tetap remembering path: bot JS juga membuat
            // file koleksi saat store dibuka (`database.js:138-152`).
            users: if js {
                Some(users.unwrap_or_else(|| dir.join("users.json")))
            } else {
                None
            },
            groups: if js {
                Some(groups.unwrap_or_else(|| dir.join("groups.json")))
            } else {
                None
            },
            premium: side("premium.json"),
            owner: side("owner.json"),
            partner: side("partner.json"),
        }
    }

    /// Layout JS? (file koleksi terpisah dari `settings.json`)
    fn is_js(&self) -> bool {
        self.users.is_some() || self.groups.is_some()
    }

    /// Semua file yang dibaca — untuk sidik jari (waktu modifikasi + panjang).
    fn all_paths(&self) -> Vec<PathBuf> {
        let mut v = vec![self.primary.clone()];
        for p in [
            self.users.clone(),
            self.groups.clone(),
            self.premium.clone(),
            self.owner.clone(),
            self.partner.clone(),
        ]
        .into_iter()
        .flatten()
        {
            v.push(p);
        }
        v
    }
}

/// Sidik jari file: waktu modifikasi + panjang. `None` = file tidak ada.
type Stamp = Option<(SystemTime, u64)>;

fn stamps(files: &StoreFiles) -> Vec<(PathBuf, Stamp)> {
    files
        .all_paths()
        .into_iter()
        .map(|p| {
            let st = std::fs::metadata(&p).ok().map(|m| {
                (
                    m.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    m.len(),
                )
            });
            (p, st)
        })
        .collect()
}

/// Baca file JSON; file rusak / tidak ada → `None` (bukan error fatal).
fn load_json(path: &Path) -> Option<Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Pandangan satu file untuk seluruh koleksi — bentuk yang dipakai
/// [`StoreContext`] sebagai snapshot.
///
/// Layout JS: `settings.json` = settings, tiap file lain = satu koleksi.
/// Layout Rust: semua koleksi ada di root `settings.json`.
fn merged_value(files: &StoreFiles) -> Value {
    let primary =
        load_json(&files.primary).unwrap_or_else(|| Value::Object(Default::default()));
    let mut map = serde_json::Map::new();
    let ada_settings = primary.get("settings").is_some_and(Value::is_object);
    // Settings: layout merged → key "settings"; layout JS → seluruh file
    // (kecuali key koleksi, itu bukan setting).
    let settings = if ada_settings {
        primary
            .get("settings")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()))
    } else {
        let mut s = primary.clone();
        if let Some(obj) = s.as_object_mut() {
            for key in KOLEKSI_JS {
                obj.remove(key);
            }
        }
        s
    };
    map.insert("settings".into(), settings);
    // File JS menang atas key root — itu sumber kebenaran layout JS.
    for (key, path) in [
        ("users", &files.users),
        ("groups", &files.groups),
        ("premium", &files.premium),
        ("owner", &files.owner),
        ("partner", &files.partner),
    ] {
        if let Some(p) = path
            && let Some(v) = load_json(p)
        {
            map.insert(key.into(), v);
        }
    }
    // Layout Rust: koleksi ada di root file utama.
    if let Some(obj) = primary.as_object() {
        for key in KOLEKSI_JS {
            if let Some(v) = obj.get(key) {
                map.entry(key.to_string()).or_insert_with(|| v.clone());
            }
        }
    }
    Value::Object(map)
}

/// [`RouterContext`] di atas `JsonStore`.
///
/// Snapshot dibaca lewat `RwLock` bukan `Arc` polos karena `JsonStore::set`
/// butuh `&mut self` sementara store ini dipakai bersama: setelah mutasi
/// ditulis, snapshot di-GANTI dengan `JsonStore` hasil baca ulang, bukan
/// diubah in-place. Itu satu-satunya cara supaya pesan berikutnya membaca
/// hasil yang sama dengan yang ada di disk.
///
/// `Clone` dipakai runtime: satu pipeline hidup sudah menempel ke bus event,
/// jadi saat `Event::Connected` datang lagi (reconnect) pipeline itu yang
/// dipakai ulang, bukan yang dibangun ulang dari store yang sudah terbaca.
#[derive(Clone)]
pub struct StoreContext {
    /// Apa yang dibaca pipeline sekarang. Ditukar penuh oleh
    /// [`StoreContext::apply`] setiap kali mutasi ditulis.
    snapshot: Arc<RwLock<Arc<JsonStore>>>,
    /// Lokasi dokumen. Tulis selalu lewat buka-ulang file ini, jadi proses
    /// lain yang menulis file yang sama tidak diam-diam ditimpa.
    path: PathBuf,
    /// File store yang dibaca (layout Rust satu file, atau layout JS).
    files: StoreFiles,
    /// Sidik jari file terakhir yang dimuat — dasar [`StoreContext::refresh`].
    stamps: Arc<Mutex<Vec<(PathBuf, Stamp)>>>,
    started: Instant,
}

impl StoreContext {
    pub fn new(store: Arc<JsonStore>) -> Self {
        let path = store.path().to_path_buf();
        let files = StoreFiles::detect(path.clone());
        let merged = merged_value(&files);
        let sidik = stamps(&files);
        let snapshot = match JsonStore::open(&files.primary) {
            Ok(mut s) => {
                let _ = s.replace(merged);
                Arc::new(s)
            }
            // File utama tidak bisa dibuka — pakai store yang sudah diberikan.
            Err(_) => store,
        };
        Self {
            snapshot: Arc::new(RwLock::new(snapshot)),
            path,
            files,
            stamps: Arc::new(Mutex::new(sidik)),
            started: Instant::now(),
        }
    }

    /// Dengan titik mulai tetap — supaya `uptime_secs` bisa diuji.
    pub fn with_started_at(store: Arc<JsonStore>, started: Instant) -> Self {
        let mut this = Self::new(store);
        this.started = started;
        this
    }

    /// Muat ulang file store kalau ada yang berubah DI LUAR proses.
    ///
    /// Panel bot JS (atau proses lain) menulis `groups.json`/`settings.json`
    /// langsung ke disk. Tanpa refresh, perubahan seperti toggle `antilink`
    /// baru terlihat setelah mutasi Rust berikutnya — proteksi grup yang
    /// baru dinyalakan owner tetap mati di sisi Rust.
    ///
    /// Biayanya cuma `stat()` per file; file yang berubah saja yang dibaca
    /// ulang.
    pub fn refresh(&self) {
        let baru = stamps(&self.files);
        {
            let cur = self.stamps.lock().unwrap_or_else(|e| e.into_inner());
            if *cur == baru {
                return;
            }
        }
        let merged = merged_value(&self.files);
        if let Ok(mut s) = JsonStore::open(&self.files.primary) {
            let _ = s.replace(merged);
            *self
                .snapshot
                .write()
                .unwrap_or_else(|e| e.into_inner()) = Arc::new(s);
        }
        *self.stamps.lock().unwrap_or_else(|e| e.into_inner()) = baru;
    }

    fn read(&self) -> Arc<JsonStore> {
        Arc::clone(&self.snapshot.read().unwrap_or_else(|e| e.into_inner()))
    }

    /// Terapkan mutasi lalu PERSIST ke disk, sebelum pemanggil membalas.
    ///
    /// Urutan penting dan bukan pilihan: plugin JS menulis `db.save()` lalu
    /// baru `m.reply(...)`, jadi saat user melihat "✅ tersimpan", datanya
    /// sudah ada. Kalau pipeline membalas dulu dan menulis kemudian, user
    /// bisa melihat konfirmasi untuk data yang hilang kalau proses mati di
    /// antaranya.
    ///
    /// Kegagalan dikembalikan, TIDAK ditelan: pemanggil wajib melaporkannya
    /// (lihat pemakaian di `Pipeline::process`).
    ///
    /// Mengembalikan jumlah mutasi yang benar-benar diterapkan.
    pub fn apply(&self, mutations: &[Mutation], now_ms: i64) -> Result<usize, StoreWriteError> {
        if mutations.is_empty() {
            return Ok(0);
        }
        // Lock penuh: mutate + tulis + tukar snapshot satu transaksi. Dua
        // pesan yang sama-sama menulis user yang sama tidak boleh saling
        // menimpa snapshot di antara baca dan tulis.
        let mut guard = self.snapshot.write().unwrap_or_else(|e| e.into_inner());
        if self.files.is_js() {
            // Layout bot JS: satu file per koleksi — `users.json` ditulis
            // untuk user, `groups.json` untuk grup, `settings.json` untuk
            // setting (`database.js:127-136`).
            self.apply_js(mutations, now_ms)?;
            let merged = merged_value(&self.files);
            if let Ok(mut s) = JsonStore::open(&self.files.primary) {
                let _ = s.replace(merged);
                *guard = Arc::new(s);
            }
        } else {
            let mut fresh = JsonStore::open(&self.path).map_err(|e| StoreWriteError {
                path: self.path.clone(),
                detail: format!("buka gagal: {e}"),
            })?;

            for m in mutations {
                apply_mutation(&mut fresh, m, now_ms, self.path.clone())?;
            }

            fresh.write().map_err(|e| StoreWriteError {
                path: self.path.clone(),
                detail: format!("tulis gagal: {e}"),
            })?;

            // Snapshot ditukar HANYA setelah `write()` sukses. Kalau tulis
            // gagal, pipeline masih membaca state lama — sama dengan isi disk.
            *guard = Arc::new(fresh);
        }
        *self.stamps.lock().unwrap_or_else(|e| e.into_inner()) = stamps(&self.files);
        Ok(mutations.len())
    }

    /// Tulis mutasi ke file KOLEKSI bot JS.
    ///
    /// `users.json` berisi objek user (key = nomor), `groups.json` objek
    /// grup, `settings.json` isi setting — persis `fileMap` JS
    /// (`database.js:127-136`). Menulis ke `settings.json` saja (seperti
    /// layout Rust) akan mengaburkan file JS dan corrup `settings.json` yang
    /// dipakai bot JS.
    fn apply_js(&self, mutations: &[Mutation], now_ms: i64) -> Result<(), StoreWriteError> {
        let mut user_mut = Vec::new();
        let mut group_mut = Vec::new();
        for m in mutations {
            match m {
                Mutation::SetUser { .. } => user_mut.push(m),
                Mutation::SetGroup { .. } => group_mut.push(m),
                Mutation::SetSetting { .. } => {}
            }
        }

        if !user_mut.is_empty() {
            let path = self
                .files
                .users
                .clone()
                .unwrap_or_else(|| self.path.clone());
            let mut store = JsonStore::open(&path).map_err(|e| StoreWriteError {
                path: path.clone(),
                detail: format!("buka gagal: {e}"),
            })?;
            let mut users = store.all().clone();
            if !users.is_object() {
                users = Value::Object(Default::default());
            }
            for m in user_mut {
                let Mutation::SetUser { jid, data } = m else {
                    continue;
                };
                // `getUser` mengembalikan `null` untuk jid yang tidak bisa
                // dinormalisasi (`database.js:530-531`) → `setUser` diam.
                let Some(key) = normalize_user_key(jid) else {
                    continue;
                };
                apply_set_user(&mut users, &key, data, now_ms);
            }
            store.replace(users).map_err(|e| StoreWriteError {
                path: path.clone(),
                detail: format!("setUser gagal: {e}"),
            })?;
            store.write().map_err(|e| StoreWriteError {
                path,
                detail: format!("tulis gagal: {e}"),
            })?;
        }

        if !group_mut.is_empty() {
            let path = self
                .files
                .groups
                .clone()
                .unwrap_or_else(|| self.path.clone());
            let mut store = JsonStore::open(&path).map_err(|e| StoreWriteError {
                path: path.clone(),
                detail: format!("buka gagal: {e}"),
            })?;
            let mut groups = store.all().clone();
            if !groups.is_object() {
                groups = Value::Object(Default::default());
            }
            for m in group_mut {
                let Mutation::SetGroup { jid, data } = m else {
                    continue;
                };
                if jid.is_empty() {
                    continue;
                }
                apply_set_group(&mut groups, jid, data);
            }
            store.replace(groups).map_err(|e| StoreWriteError {
                path: path.clone(),
                detail: format!("setGroup gagal: {e}"),
            })?;
            store.write().map_err(|e| StoreWriteError {
                path,
                detail: format!("tulis gagal: {e}"),
            })?;
        }

        // Setting tetap di file utama: `settings.json` (flat, tanpa key
        // "settings") → ditulis di root, sama seperti `db.setting()` JS.
        let setting_mut: Vec<&Mutation> = mutations
            .iter()
            .filter(|m| matches!(m, Mutation::SetSetting { .. }))
            .collect();
        if !setting_mut.is_empty() {
            let mut store = JsonStore::open(&self.path).map_err(|e| StoreWriteError {
                path: self.path.clone(),
                detail: format!("buka gagal: {e}"),
            })?;
            for m in setting_mut {
                apply_mutation(&mut store, m, now_ms, self.path.clone())?;
            }
            store.write().map_err(|e| StoreWriteError {
                path: self.path.clone(),
                detail: format!("tulis gagal: {e}"),
            })?;
        }
        Ok(())
    }
}

/// Mutasi gagal ditulis. Path ikut dibawa supaya log menyebut file yang
/// bermasalah, bukan cuma "write failed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreWriteError {
    pub path: PathBuf,
    pub detail: String,
}

impl fmt::Display for StoreWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.detail, self.path.display())
    }
}

impl std::error::Error for StoreWriteError {}

/// Terapkan satu mutasi ke store yang SUDAH dibuka untuk tulis.
///
/// Tidak menyentuh disk, jadi semantik `setUser`/`setGroup` bisa diuji
/// tanpa file sama sekali.
fn apply_mutation(
    store: &mut JsonStore,
    m: &Mutation,
    now_ms: i64,
    path: PathBuf,
) -> Result<(), StoreWriteError> {
    match m {
        Mutation::SetUser { jid, data } => {
            // `getUser` mengembalikan `null` untuk jid yang tidak bisa
            // dinormalisasi (`database.js:530-531`), jadi `setUser` ikut
            // `return null` — diam, tanpa menulis apa pun.
            let Some(key) = normalize_user_key(jid) else {
                return Ok(());
            };
            let mut users = store
                .get("users")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));
            apply_set_user(&mut users, &key, data, now_ms);
            store.set("users", users).map_err(|e| StoreWriteError {
                path,
                detail: format!("setUser({key}) gagal: {e}"),
            })?;
        }
        Mutation::SetGroup { jid, data } => {
            if jid.is_empty() {
                return Ok(());
            }
            let mut groups = store
                .get("groups")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));
            apply_set_group(&mut groups, jid, data);
            store.set("groups", groups).map_err(|e| StoreWriteError {
                path,
                detail: format!("setGroup({jid}) gagal: {e}"),
            })?;
        }
        Mutation::SetSetting { key, value } => {
            // Dua layout file (lihat `settings_obj`). Kalau file ini layout JS
            // — file = isi koleksi settings, tanpa key "settings" — setting
            // ditulis di ROOT. Kalau tidak, layout berubah setengah jalan:
            // file jadi punya key "settings", dan semua setting lama yang
            // ada di root ikut hilang dari pembacaan.
            let flat = !store.get("settings").is_some_and(Value::is_object);
            if flat {
                store.set(key, value.clone()).map_err(|e| StoreWriteError {
                    path,
                    detail: format!("setting({key}) gagal: {e}"),
                })?;
                return Ok(());
            }
            let mut settings = store
                .get("settings")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));
            if !settings.is_object() {
                settings = Value::Object(Default::default());
            }
            settings[key] = value.clone();
            store.set("settings", settings).map_err(|e| StoreWriteError {
                path,
                detail: format!("setting({key}) gagal: {e}"),
            })?;
        }
    }
    Ok(())
}

/// `setUser` (`database.js:536-603`) atas `users[key]`.
///
/// Bentuk implementannya: map user yang ada di-TIRU ke `u` (owned),
/// semua nilai dihitung dari `u`, lalu ditulis balik sekali di akhir.
/// Menyalin-pintas menghindari borrow conflict dan sekaligus membuat
/// urutan "hapus field legacy → salin field bebas → tulis field default"
/// sama persis dengan urutan baris di JS.
///
/// Dua bagian yang sering terlewat dan di sini dijaga:
///
/// 1. `USER_SET_OVERRIDDEN` (`database.js:23-28`) — key yang ADA di
///    `data` TIDAK disalin mentah; nilainya ditulis ulang dari
///    `data.k ?? existing.k ?? default`. Bedanya chiefly di `name`:
///    `data.name || ...` memakai `||`, jadi `name: ""` jatuh ke
///    `"Unknown"`, bukan tersimpan sebagai string kosong.
/// 2. `existing.rpg` / `existing.inventory` di-MERGE satu level
///    (`{...existing, ...data}`), bukan diganti. Plugin seperti `bank.js`
///    menulis `rpg.bank` tanpa mengirim ulang seluruh blok `rpg`; kalau
///    ini jadi replace, `rpg.exp` milik user hilang.
pub fn apply_set_user(users: &mut Value, key: &str, data: &Value, now_ms: i64) {
    let mut u = users
        .get(key)
        .cloned()
        .filter(Value::is_object)
        .unwrap_or_else(|| Value::Object(Default::default()));
    let u = u.as_object_mut().expect("dijamin object di atas");

    // `existing.balance`/`existing.limit` dibaca lalu DIHAPUS supaya field
    // legacy tidak ikut di-serialize; `balance` jadi `koin`, `limit`
    // jadi `energi` (database.js:539-548).
    let existing_balance = u.get("balance").and_then(Value::as_i64).unwrap_or(0);
    let existing_limit = u
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(DEFAULT_USER_LIMIT);
    u.remove("balance");
    u.remove("limit");

    if let Some(d) = data.as_object() {
        for (k, v) in d {
            if !USER_SET_OVERRIDDEN.contains(&k.as_str()) {
                u.insert(k.clone(), v.clone());
            }
        }
    }

    // `data.x ?? existing.x ?? default` — `??` hanya jatuh ke kanan saat
    // kiri `null`/`undefined`, jadi `0` dan `""` TIDAK jadi default.
    // Plugin rpg bergantung pada ini: `koin: 0` harus tersimpan 0.
    let num_or = |field: &str, default: Value| -> Value {
        let exist_v = u.get(field).cloned();
        match data.get(field) {
            Some(v) if !v.is_null() => v.clone(),
            _ => match exist_v {
                Some(v) if !v.is_null() => v,
                _ => default,
            },
        }
    };

    // `data.rpg !== undefined && data.rpg !== existing.rpg
    //  ? {...existing.rpg, ...data.rpg} : existing.rpg || {}`
    // (database.js:592-594). Perbandingan `!==` JS di sini sama dengan
    // "kedua sisi ada dan berbeda nilainya".
    let merged = |field: &str| -> Value {
        let exist_v = u.get(field).cloned();
        match (data.get(field), &exist_v) {
            (Some(dv), Some(ev)) if dv != ev => {
                let mut m = ev.clone();
                match (m.as_object_mut(), dv.as_object()) {
                    (Some(m), Some(dv)) => {
                        for (k, v) in dv {
                            m.insert(k.clone(), v.clone());
                        }
                    }
                    // Salah satu bukan object: spread di JS hanya berlaku
                    // kalau keduanya object; selain itu `data` yang menang.
                    _ => m = dv.clone(),
                }
                m
            }
            _ => exist_v.unwrap_or_else(|| Value::Object(Default::default())),
        }
    };

    let is_lid = key.starts_with(USER_KEY_LID_PREFIX);
    let clean = key.strip_prefix(USER_KEY_LID_PREFIX).unwrap_or(key);
    let num = || {
        if is_lid {
            Value::Null
        } else {
            Value::from(clean)
        }
    };

    // `data.name || existing.name || "Unknown"` — `||`, bukan `??`.
    let name = data
        .get("name")
        .filter(|v| truthy(v))
        .or_else(|| u.get("name").filter(|v| truthy(v)))
        .cloned()
        .unwrap_or_else(|| Value::from("Unknown"));
    // `data.access || existing.access || []` — `||` juga: array kosong di
    // `data` TIDAK menang, existing dipakai.
    let access = data
        .get("access")
        .filter(|v| truthy(v))
        .or_else(|| u.get("access").filter(|v| truthy(v)))
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let lid = if is_lid {
        Value::from(clean)
    } else {
        u.get("lid").cloned().unwrap_or(Value::Null)
    };
    let rpg = merged("rpg");
    let inventory = merged("inventory");

    // Semua nilai dihitung DULU ke dalam daftar, baru ditulis. Kalau
    // `u.insert` diselang dengan `num_or(...)` yang membaca `u`, borrow
    // checker menolak — dan urutan hitungannya jadi tidak bisa
    // dibandingkan baris-per-baris dengan JS.
    let fields: Vec<(&str, Value)> = vec![
        ("jid", num()),
        ("lid", lid),
        ("name", name),
        ("number", num()),
        ("energi", num_or("energi", Value::from(existing_limit))),
        ("isPremium", num_or("isPremium", Value::from(false))),
        ("isBanned", num_or("isBanned", Value::from(false))),
        ("exp", num_or("exp", Value::from(0))),
        ("level", num_or("level", Value::from(1))),
        ("koin", num_or("koin", Value::from(existing_balance))),
        ("saldo", num_or("saldo", Value::from(0))),
        (
            "unlockedFeatures",
            num_or("unlockedFeatures", Value::Array(Vec::new())),
        ),
        ("registeredAt", num_or("registeredAt", Value::Null)),
        (
            "lastRegisteredAt",
            num_or("lastRegisteredAt", Value::Null),
        ),
        (
            "registrationCount",
            num_or("registrationCount", Value::from(0)),
        ),
        (
            "hasClaimedRegisterReward",
            num_or("hasClaimedRegisterReward", Value::from(false)),
        ),
        ("unregisteredAt", num_or("unregisteredAt", Value::Null)),
        ("lastSeen", Value::from(iso_utc(now_ms))),
        (
            "cooldowns",
            num_or("cooldowns", Value::Object(Default::default())),
        ),
        ("clanId", num_or("clanId", Value::Null)),
        (
            "isRegistered",
            num_or("isRegistered", Value::from(false)),
        ),
        ("regName", num_or("regName", Value::Null)),
        ("regAge", num_or("regAge", Value::Null)),
        ("regGender", num_or("regGender", Value::Null)),
        ("rpg", rpg),
        ("inventory", inventory),
        ("access", access),
    ];
    // Urutan list di atas = urutan baris `u.x = ...` di `database.js:559-601`.
    for (k, v) in fields {
        u.insert(k.to_string(), v);
    }

    if let Some(root) = users.as_object_mut() {
        root.insert(key.to_string(), Value::Object(u.clone()));
    }
}

/// `setGroup` (`database.js:717-746`) atas `groups[jid]`.
pub fn apply_set_group(groups: &mut Value, jid: &str, data: &Value) {
    let mut g = groups
        .get(jid)
        .cloned()
        .filter(Value::is_object)
        .unwrap_or_else(|| Value::Object(Default::default()));
    let g = g.as_object_mut().expect("dijamin object di atas");

    // `{...existing, ...data}` dulu — field lain ikut mewarisi.
    if let Some(d) = data.as_object() {
        for (k, v) in d {
            g.insert(k.clone(), v.clone());
        }
    }

    // `data.x ?? existing.x ?? default` untuk field yang punya default
    // eksplisit di JS. Dihitung semua dulu supaya tidak ada `&g` hidup
    // sementara `g` di-`insert`.
    let pick = |field: &str, default: Value| -> Value {
        let exist_v = g.get(field).cloned();
        match data.get(field) {
            Some(v) if !v.is_null() => v.clone(),
            _ => match exist_v {
                Some(v) if !v.is_null() => v,
                _ => default,
            },
        }
    };
    // Urutan = urutan baris di `database.js:735-744`.
    let toggles: Vec<(&str, Value)> = vec![
        // `config.welcome?.defaultEnabled ?? false` (`database.js:723-725`);
        // config repo ini `false` (`config.js:186`).
        ("welcome", pick("welcome", Value::from(GROUP_TOGGLE_DEFAULT))),
        ("leave", pick("leave", Value::from(GROUP_TOGGLE_DEFAULT))),
        ("goodbye", pick("goodbye", Value::from(GROUP_TOGGLE_DEFAULT))),
        ("antilink", pick("antilink", Value::from(false))),
        ("antitoxic", pick("antitoxic", Value::from(false))),
        ("mute", pick("mute", Value::from(false))),
        ("game", pick("game", Value::from(true))),
        ("rpg", pick("rpg", Value::from(true))),
        ("warnings", pick("warnings", Value::Array(Vec::new()))),
    ];
    for (field, v) in toggles {
        g.insert(field.to_string(), v);
    }
    // `welcomeMsg`/`goodbyeMsg`/`intro` TIDAK punya default:
    // `data.x ?? existing.x ?? undefined` berarti key yang tidak ada di
    // `data` DAN existing tetap `null` di JSON, bukan `undefined` yang
    // akan hilang saat serialize.
    for field in ["welcomeMsg", "goodbyeMsg", "intro"] {
        g.entry(field.to_string()).or_insert(Value::Null);
        if let Some(v) = data.get(field) {
            g.insert(field.to_string(), v.clone());
        }
    }
    g.entry("chat".to_string())
        .or_insert_with(|| Value::Object(Default::default()));
    // `name` dihitung dari objek yang SUDAH ter-merge di atas, jadi
    // `name` dari `existing` tidak hilang oleh default.
    let name = data
        .get("name")
        .filter(|v| truthy(v))
        .or_else(|| g.get("name").filter(|v| truthy(v)))
        .cloned()
        .unwrap_or_else(|| Value::from("Unknown Group"));
    g.insert("jid".into(), Value::from(jid));
    g.insert("name".into(), name);

    if let Some(root) = groups.as_object_mut() {
        root.insert(jid.to_string(), Value::Object(g.clone()));
    }
}

/// Key yang `setUser` TULIS ULANG dari `data`, jadi tidak boleh disalin
/// mentah lebih dulu — `USER_SET_OVERRIDDEN` (`database.js:23-28`).
const USER_SET_OVERRIDDEN: &[&str] = &[
    "jid", "name", "number", "energi", "isPremium", "isBanned", "exp", "level", "koin", "saldo",
    "unlockedFeatures", "registeredAt", "lastRegisteredAt", "registrationCount",
    "hasClaimedRegisterReward", "unregisteredAt", "lastSeen", "cooldowns", "clanId",
    "isRegistered", "regName", "regAge", "regGender", "rpg", "inventory", "access",
];

/// Prefiks kunci user untuk JID LID (`normalize_user_key`).
const USER_KEY_LID_PREFIX: &str = "lid:";

/// `existing.limit !== undefined ? existing.limit : config.energi?.default || 25`
/// (`database.js:541-543`). Default bot sebenarnya `99999`
/// (`config.js:114` = `EnergiSettings::default().default`), jadi kedua
/// angka itu sama dan `25` tidak pernah terlihat di produksi.
const DEFAULT_USER_LIMIT: i64 = 99_999;

/// `config.welcome.defaultEnabled` (`config.js:186`).
const GROUP_TOGGLE_DEFAULT: bool = false;

/// Truthiness JavaScript: `false`, `0`, `""`, `null`, `NaN` salah.
/// `serde_json` tidak punya `NaN`, jadi yang tersisa empat kasus itu.
/// Array dan object — termasuk yang KOSONG — selalu truthy di JS.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// `new Date(ms).toISOString()` — format sama persis dengan yang ditulis
/// `database.js:585`, karena `lastSeen` dibaca `Date.parse` oleh
/// `handler.js:960` dan dibandingkan sebagai string ISO di
/// `data-pruner.js:36`.
pub fn iso_utc(ms: i64) -> String {
    let (secs, milli) = (ms.div_euclid(1000), ms.rem_euclid(1000));
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let (y, mo, da) = civil_from_days(days);
    format!(
        "{y:04}-{mo:02}-{da:02}T{:02}:{:02}:{:02}.{milli:03}Z",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

/// Hari sejak epoch → `(tahun, bulan, hari)`, algoritma Howard Hinnant.
/// `chrono` tidak dipakai: satu fungsi 20 baris lebih murah daripada
/// dependency untuk satu format string.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

impl RouterContext for StoreContext {
    fn user_state(&self, sender: &str) -> UserState {
        let store = self.read();
        match normalize_user_key(sender) {
            Some(key) => user_state_from(top(&store, "users"), &key),
            None => UserState::default(),
        }
    }

    fn group_state(&self, chat: &str) -> Option<GroupState> {
        let store = self.read();
        group_state_from(top(&store, "groups"), chat)
    }

    fn group_bot_mode(&self, chat: &str) -> String {
        let store = self.read();
        group_bot_mode_from(top(&store, "groups"), chat)
    }

    fn mode_settings(&self) -> ModeSettings {
        let store = self.read();
        let s = settings_obj(&store);
        mode_settings_from(if s.is_object() { Some(&s) } else { None })
    }

    fn group_overrides(&self, chat: &str) -> Option<GroupOverrides> {
        let store = self.read();
        group_overrides_from(top(&store, "groups"), chat)
    }

    fn capprem(&self, name: &str) -> Option<bool> {
        let store = self.read();
        let s = settings_obj(&store);
        capprem_from(if s.is_object() { Some(&s) } else { None }, name)
    }

    /// JS memakai `process.uptime()`. Tidak ada backing di store, jadi ini
    /// elapsed sejak `StoreContext` dibuat.
    fn uptime_secs(&self) -> u64 {
        self.started.elapsed().as_secs()
    }
}

impl StoreContext {
    /// `users[key]` mentah untuk `sender` — objek yang di-Ctx jadi
    /// `user_data`. `Value::Null` kalau tidak ada, persis `getUser` yang
    /// mengembalikan `null` (`database.js:529-536`).
    pub fn user_entry(&self, sender: &str) -> Value {
        let Some(key) = normalize_user_key(sender) else {
            return Value::Null;
        };
        self.read()
            .get("users")
            .and_then(|u| u.get(&key))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// `groups[chat]` mentah untuk `chat`. Tanpa normalisasi JID, sama seperti
    /// `getGroup` (`database.js:711-714`).
    pub fn group_entry(&self, chat: &str) -> Value {
        self.read()
            .get("groups")
            .and_then(|g| g.get(chat))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// `settings` mentah — objek yang jadi `Ctx::settings`, sumber
    /// `db.setting(key)` (`database.js:753`).
    pub fn settings_entry(&self) -> Value {
        settings_obj(&self.read())
    }

    /// `config.isOwner(number)` (`config.js:364-404`) — sumber store besides
    /// config: koleksi `owner` dan `settings.ownerNumbers`.
    pub fn is_owner_user(&self, jid: &str) -> bool {
        let store = self.read();
        if store
            .get("owner")
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|e| matches_number(e.as_str().unwrap_or(""), jid)))
        {
            return true;
        }
        settings_obj(&store)
            .get("ownerNumbers")
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|e| matches_number(e.as_str().unwrap_or(""), jid)))
    }

    /// `config.isPremium(number)` (`config.js:406-460`) — koleksi `premium`
    /// dan `settings.premiumUsers`.
    ///
    /// `ponytail:` JS menghapus entri kedaluwarsa dari store saat ditemukan;
    /// di sini entri itu hanya DILEWATI (dicek readonly), jadi file store bisa
    /// masih menyimpan premium lama yang sudah expired.
    pub fn is_premium_user(&self, jid: &str, now_ms: i64) -> bool {
        let store = self.read();
        if store
            .get("premium")
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|e| entry_is_active(e, jid, now_ms)))
        {
            return true;
        }
        settings_obj(&store)
            .get("premiumUsers")
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|e| matches_number(e.as_str().unwrap_or(""), jid)))
    }

    /// `config.isPartner(number)` (`config.js:465-505`) — koleksi `partner`.
    pub fn is_partner_user(&self, jid: &str, now_ms: i64) -> bool {
        let store = self.read();
        store
            .get("partner")
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|e| entry_is_active(e, jid, now_ms)))
    }

    /// `config.isBanned(number)` (`config.js:507-519`) — `settings.bannedUsers`.
    pub fn is_banned_user(&self, jid: &str) -> bool {
        let store = self.read();
        settings_obj(&store)
            .get("bannedUsers")
            .and_then(Value::as_array)
            .is_some_and(|arr| arr.iter().any(|e| matches_number(e.as_str().unwrap_or(""), jid)))
    }

    /// `settings.capenergi` (`handler.js:1856`) — override biaya energi per
    /// plugin. Nilai `0` BERARTI "command ini gratis" dan harus tetap menang
    /// atas `plugin.config.energi`, jadi setiap key ikut dibaca bukan hanya
    /// yang truthy. Key non-numerik diabaikan, bukan diperlakukan `0`.
    pub fn energy_overrides(&self) -> HashMap<String, i64> {
        let store = self.read();
        let s = settings_obj(&store);
        s.get("capenergi")
            .and_then(Value::as_object)
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_i64()?)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `settings.energi` (`handler.js:1851-1854`). `None` berarti setting-nya
    /// tidak ada, jadi ikut [`resolve_energi_enabled`] dengan nilai config.
    pub fn energi_enabled_setting(&self) -> Option<bool> {
        let store = self.read();
        settings_obj(&store).get("energi").and_then(Value::as_bool)
    }

    /// `db.setting("notiflimit")` (`handler.js:1884`) — kalau aktif, potong
    /// energi dilaporkan ke user.
    pub fn notiflimit_setting(&self) -> bool {
        let store = self.read();
        settings_obj(&store)
            .get("notiflimit")
            .map(truthy_js)
            .unwrap_or(false)
    }
}

// =============================================================================
// 3. Dedup 100 ms — port `debounceMessage` (src/lib/performance.js:10-29)
// =============================================================================

pub const DEBOUNCE_MS: i64 = 100;

/// `true` berarti duplikat → buang. Kunci dipakai apa adanya; pemanggil yang
/// membentuknya (`handler.js:926`: `${botId}_${m.chat}_${m.sender}_${m.id}`).
#[derive(Debug, Default)]
pub struct Dedup {
    entries: HashMap<String, i64>,
    last_clean: i64,
}

impl Dedup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn debounce(&mut self, key: &str, now_ms: i64) -> bool {
        if let Some(&last) = self.entries.get(key)
            && now_ms - last < DEBOUNCE_MS
        {
            return true;
        }
        self.entries.insert(key.to_string(), now_ms);

        // Pembersihan malas: hanya saat >1000 entri DAN sudah 5 s sejaknowrap
        // pembersihan sebelumnya (performance.js:20-26).
        if self.entries.len() > 1000 && now_ms - self.last_clean > 5000 {
            self.last_clean = now_ms;
            let cutoff = now_ms - DEBOUNCE_MS * 2;
            self.entries.retain(|_, v| *v >= cutoff);
        }
        false
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// =============================================================================
// 4. Lapisan kirim — trait supaya pipeline bisa diuji tanpa koneksi WA
// =============================================================================

pub type SendBox<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

/// Tituhubung yang mengabstraksi `Client::send_message_with_options`
/// (`whatsapp-rust-0.7.0/src/send/mod.rs:901`).
pub trait MessageSender: Send + Sync {
    fn send(&self, to: &str, message: wa::Message, opts: SendOptions) -> SendBox<'_>;

    /// Klien WA sungguhan, kalau pengirim ini memegangnya.
    ///
    /// Dipakai satu hal: memasang [`oktz_plugin_api::MediaFetch`] untuk
    /// `m.download()`. Pengirim stub (test) mengembalikan `None`, dan itu
    /// membuat `Ctx::media_fetch` tetap `NoMediaFetch` — menolak dengan alasan
    /// jelas, bukan mengembalikan byte palsu.
    fn client(&self) -> Option<Arc<Client>> {
        None
    }

    /// Kirim media: baca sumbernya → upload ke CDN → kirim.
    ///
    /// Terpisah dari `send` karena butuh I/O (disk/HTTP/upload) yang `send`
    /// tidak punya, dan karena pemanggil harus memilihjalur yang benar.
    /// Default-nya error, BUKAN diam: implementasi yang tidak punya
    /// `Client` (test) akan menolak dengan jelas alih-alih menelan reply.
    fn send_media(&self, _to: &str, _media: PendingMedia, _opts: SendOptions) -> SendBox<'_> {
        Box::pin(async move {
            Err("pengirim ini tidak mendukung kirim media ( butuh Client )".to_string())
        })
    }

    /// Kirim kartu interaktif yang media header/stikernya belum ter-upload.
    ///
    /// Default-nya error dengan alasan yang sama seperti
    /// [`MessageSender::send_media`]: tanpa `Client` tidak ada yang bisa
    /// meng-upload, dan diam berarti kartu hilang tanpa kabar.
    fn send_card(&self, _to: &str, _card: PendingCard, _opts: SendOptions) -> SendBox<'_> {
        Box::pin(async move {
            Err("pengirim ini tidak mendukung kirim kartu ( butuh Client )".to_string())
        })
    }

    /// Kirim preview link yang thumbnail-nya belum ter-upload.
    ///
    /// Default-nya error dengan alasan yang sama seperti
    /// [`MessageSender::send_media`]: tanpa `Client` tidak ada yang bisa
    /// meng-upload.
    fn send_preview(&self, _to: &str, _preview: PendingPreview, _opts: SendOptions) -> SendBox<'_> {
        Box::pin(async move {
            Err("pengirim ini tidak mendukung kirim preview ( butuh Client )".to_string())
        })
    }

    /// Hapus pesan untuk semua orang di chat (`sock.sendMessage(chat, {delete:
    /// m.key})`, dipakai rantai group-protection di
    /// `handler.js:811, 823, 848`).
    /// `original_sender` adalah pengirim pesan yang dihapus — WA membedakan
    /// revoke-sendiri dari revoke-admin, dan ini yang kedua: bot menghapus
    /// pesan orang lain, bukan pesan sendiri.
    ///
    /// Default-nya error dengan alasan yang sama seperti
    /// [`MessageSender::send_media`]: diam berarti pesan terlarangan tetap
    /// terlihat di grup, yaitu kegagalan yang paling mahal untuk tidak disadari.
    fn delete_message(
        &self,
        _chat: &str,
        _message_id: &str,
        _original_sender: &str,
    ) -> SendBox<'_> {
        Box::pin(async move {
            Err("pengirim ini tidak mendukung hapus pesan ( butuh Client )".to_string())
        })
    }

    /// Reaksi ke pesan (`m.react("⏱️")`, `handler.js:1846`).
    ///
    /// Default-nya error dan hanya dilog, bukan dikembalikan sebagai kegagalan
    /// ke pemanggil — di JS reaksi juga di-`.catch(() => {})`, jadi gagal
    /// bereaksi tidak boleh menggagalkan pipeline.
    fn react(&self, _chat: &str, _target: &wa::MessageKey, _emoji: &str) -> SendBox<'_> {
        Box::pin(async move {
            Err("pengirim ini tidak mendukung reaksi ( butuh Client )".to_string())
        })
    }
}

/// Sender sungguhan, membungkus `Arc<Client>`.
pub struct ClientSender {
    client: Arc<Client>,
}

impl ClientSender {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl MessageSender for ClientSender {
    fn client(&self) -> Option<Arc<Client>> {
        Some(self.client.clone())
    }

    fn send(&self, to: &str, message: wa::Message, opts: SendOptions) -> SendBox<'_> {
        let client = self.client.clone();
        let to = to.to_string();
        Box::pin(async move {
            let jid: Jid = to
                .parse()
                .map_err(|e| format!("JID tidak valid {to}: {e}"))?;
            client
                .send_message_with_options(jid, message, opts)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }

    /// Alur media nyata: `Client::upload` (`src/upload.rs:409`) → bangun
    /// `wa::Message` dari `UploadResponse` → `send_message_with_options`.
    fn send_media(&self, to: &str, media: PendingMedia, opts: SendOptions) -> SendBox<'_> {
        let client = self.client.clone();
        let to = to.to_string();
        Box::pin(async move {
            let jid: Jid = to
                .parse()
                .map_err(|e| format!("JID tidak valid {to}: {e}"))?;
            let data = load_media(&media.source).await.map_err(|e| e.to_string())?;
            // `MediaType::Sticker` dan `MediaType::Image` sama-sama mendarat
            // di `/mms/image` (`wacore-0.7.0/src/download.rs:91`) — persis
            // seperti WA Web, tapi tetap pakai tipe yang jujur per jenis.
            let upload = client
                .upload(data, media.kind.upload_type(), UploadOptions::default())
                .await
                .map_err(|e| format!("upload media gagal: {e}"))?;
            let message = build_media_message(media.kind, upload, &media);
            client
                .send_message_with_options(jid, message, opts)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }

    /// Kartu native-flow: upload media header (gambar/video) dan stiker
    /// fake-quote dulu, lalu rakit proto kartunya.
    ///
    /// `biz_node()` wajib ikut sebagai extra stanza node — tanpa itu relay
    /// sukses tapi kartu tidak muncul di chat (`src/lib/socket.js:398-410`).
    fn send_card(&self, to: &str, card: PendingCard, opts: SendOptions) -> SendBox<'_> {
        let client = self.client.clone();
        let to = to.to_string();
        Box::pin(async move {
            let jid: Jid = to
                .parse()
                .map_err(|e| format!("JID tidak valid {to}: {e}"))?;

            // 1. Media header — gambar atau video.
            let header_media = match card.header.as_ref().and_then(|h| h.media.as_ref()) {
                Some(CardMedia::Image { source }) => {
                    let up = upload_source(&client, source, MediaType::Image)
                        .await
                        .map_err(|e| e.to_string())?;
                    Some(HeaderMedia::Image(image_proto(up, source)))
                }
                Some(CardMedia::Video { source }) => {
                    let up = upload_source(&client, source, MediaType::Video)
                        .await
                        .map_err(|e| e.to_string())?;
                    Some(HeaderMedia::Video(video_proto(up, source)))
                }
                Some(CardMedia::Location(loc)) => {
                    Some(HeaderMedia::Location(build_location_message(loc)))
                }
                None => None,
            };

            // 2. Stiker fake-quote — perlukan upload seperti media biasa.
            let quote_message = match card
                .context
                .as_ref()
                .and_then(|c| c.quote.as_deref())
            {
                Some(CardQuote::Sticker { source }) => {
                    let up = upload_source(&client, source, MediaType::Image)
                        .await
                        .map_err(|e| e.to_string())?;
                    Some(sticker_proto(up, source))
                }
                Some(other) => build_quote_message(other),
                None => None,
            };

            let built = build_card_proto(&card, header_media, quote_message, None)
                .map_err(|e| format!("gagal merakit kartu: {e}"))?;
            let message = built.to_message();
            client
                .send_message_with_options(jid, message, opts.with_extra_stanza_nodes(vec![biz_node()]))
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }

    /// Preview link: upload thumbnail dulu, lalu rakit `extendedTextMessage`.
    fn send_preview(&self, to: &str, preview: PendingPreview, opts: SendOptions) -> SendBox<'_> {
        let client = self.client.clone();
        let to = to.to_string();
        Box::pin(async move {
            let jid: Jid = to
                .parse()
                .map_err(|e| format!("JID tidak valid {to}: {e}"))?;

            let mut thumb = None;
            if let Some(src) = &preview.thumbnail {
                let bytes = load_media(src).await.map_err(|e| e.to_string())?;
                let up = client
                    .upload(bytes.clone(), MediaType::Image, UploadOptions::default())
                    .await
                    .map_err(|e| format!("upload thumbnail gagal: {e}"))?;
                thumb = Some((bytes, up));
            }

            let mut ext = wa::message::ExtendedTextMessage {
                text: Some(preview.text.clone()),
                matched_text: Some(preview.matched_text.clone()),
                title: preview.title.clone(),
                description: preview.description.clone(),
                preview_type: Some(match preview.preview_type {
                    1 => wa::message::extended_text_message::PreviewType::Video,
                    _ => wa::message::extended_text_message::PreviewType::None,
                }),
                context_info: match preview.context {
                    Some(c) => MessageField::some(c),
                    None => MessageField::none(),
                },
                ..Default::default()
            };
            if let Some((bytes, up)) = thumb {
                ext.jpeg_thumbnail = Some(bytes);
                ext.thumbnail_direct_path = Some(up.direct_path);
                ext.thumbnail_sha256 = Some(up.file_sha256.to_vec());
                ext.thumbnail_enc_sha256 = Some(up.file_enc_sha256.to_vec());
                ext.media_key = Some(up.media_key.to_vec());
                ext.media_key_timestamp = Some(up.media_key_timestamp);
            }

            let message = wa::Message {
                extended_text_message: MessageField::some(ext),
                ..Default::default()
            };
            client
                .send_message_with_options(jid, message, opts)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }

    /// `Client::revoke_message` (`whatsapp-rust-0.7.0/src/send/actions.rs:14`).
    ///
    /// Bot menghapus pesan orang lain di grup, jadi `RevokeType::Admin` —
    /// `RevokeType::Sender`-meaning `from_me=true` akan membuat WA mengarang
    /// pesan kosong alih-alih menghapus (`actions.rs:33-38`), dan `Admin`
    /// memang menolak di luar grup (`actions.rs:41-45`).
    fn delete_message(
        &self,
        chat: &str,
        message_id: &str,
        original_sender: &str,
    ) -> SendBox<'_> {
        let client = self.client.clone();
        let chat = chat.to_string();
        let message_id = message_id.to_string();
        let original_sender = original_sender.to_string();
        Box::pin(async move {
            let chat_jid: Jid = chat
                .parse()
                .map_err(|e| format!("JID grup tidak valid {chat}: {e}"))?;
            let sender_jid: Jid = original_sender
                .parse()
                .map_err(|e| format!("JID pengirim tidak valid {original_sender}: {e}"))?;
            client
                .revoke_message(
                    chat_jid,
                    message_id,
                    RevokeType::Admin {
                        original_sender: sender_jid,
                    },
                )
                .await
                .map_err(|e| e.to_string())
        })
    }

    /// `Client::send_reaction` (`whatsapp-rust-0.7.0/src/features/reaction.rs:35`)
    /// dengan `MessageContext::message_key()` (`bot.rs:172`) sebagai target.
    fn react(&self, chat: &str, target: &wa::MessageKey, emoji: &str) -> SendBox<'_> {
        let client = self.client.clone();
        let chat = chat.to_string();
        let target = target.clone();
        let emoji = emoji.to_string();
        Box::pin(async move {
            let jid: Jid = chat
                .parse()
                .map_err(|e| format!("JID tidak valid {chat}: {e}"))?;
            client
                .send_reaction(jid, target, &emoji)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }
}

/// Client HTTP untuk [`MediaSource::Url`]. Satu instance per proses —
/// `UreqHttpClient::new()` membangun connection pool
/// (`whatsapp-rust-ureq-http-client-0.7.0/src/lib.rs:47`), dan fetch media
/// cukup sering.
fn http_client() -> &'static UreqHttpClient {
    static HTTP: OnceLock<UreqHttpClient> = OnceLock::new();
    HTTP.get_or_init(UreqHttpClient::new)
}

/// Resolve [`MediaSource`] jadi byte. Satu-satunya tempat di pipeline yang
/// menyentuh disk atau jaringan — `oktz-plugin-api` tetap bebas I/O.
///
/// Kegagalan selalu error bertipe, tidak pernah `Ok(vec![])` diam-diam: byte
/// kosong akan terkirim ke CDN dan gagal jauh dari penyebabnya.
async fn load_media(source: &MediaSource) -> Result<Vec<u8>, PipelineError> {
    match source {
        MediaSource::Bytes { data, .. } => {
            if data.is_empty() {
                return Err(PipelineError::Media(
                    "sumber media bytes kosong".to_string(),
                ));
            }
            Ok(data.to_vec())
        }
        // `tokio::fs`, bukan `std::fs` — `read` blocking di thread tokio
        // worker akan menahan seluruh dispatch event bot.
        MediaSource::Path { path, .. } => {
            let p = std::path::Path::new(path);
            let data = tokio::fs::read(p).await.map_err(|e| {
                PipelineError::Media(format!("gagal baca file media `{path}`: {e}"))
            })?;
            if data.is_empty() {
                return Err(PipelineError::Media(format!(
                    "file media `{path}` kosong"
                )));
            }
            Ok(data)
        }
        MediaSource::Url { url, .. } => {
            // Header `User-Agent` bawaan: beberapa CDN (TikTok, Facebook,
            // Twitter) menolak request tanpa UA dengan 403 — dan di JS plugin
            // yang mengunduh sendiri selalu mengirim UA. Tanpa ini, URL media
            // yang valid gagal tepat di langkah terakhir.
            //
            // `ponytail:` header khusus per downloader (mis. `Referer` CDNs
            // TikTok) belum dibawa lewat `MediaSource`; plugin yang butuh
            // header itu harus mengunduh sendiri lewat `ctx.http` lalu
            // mengirim `MediaSource::bytes`.
            let resp = http_client()
                .execute(
                    HttpRequest::get(url).with_header(
                        "User-Agent",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                         (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
                    ),
                )
                .await
                .map_err(|e| PipelineError::Media(format!("gagal fetch media: {e}")))?;
            // URL sengaja tidak ikut pesan: media URL sering membawa token
            // bertanda tangan yang tidak belong di log.
            if !(200..300).contains(&resp.status_code) {
                return Err(PipelineError::Media(format!(
                    "server menolak fetch media: HTTP {}",
                    resp.status_code
                )));
            }
            if resp.body.is_empty() {
                return Err(PipelineError::Media(
                    "server membalas body kosong".to_string(),
                ));
            }
            Ok(resp.body)
        }
    }
}

// =============================================================================
// 5. Reply → wire
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    /// `Reply` tidak membawa sumber media sama sekali — `media: None` di
    /// `Reply::Image` / `Reply::Sticker` (`oktz-plugin-api/src/registry.rs`).
    /// Reported, bukan di-drop diam-diam.
    MediaSourceMissing { variant: &'static str },
    /// Sumber media ada tapi gagal jadi byte / gagal upload.
    Media(String),
    /// `Reply::Mutate` mencapai [`render_reply`] tanpa lewat
    /// `Reply::into_parts` dulu — mutasi store tidak ditulis, jadi
    /// pesan yang dikirim akan mengklaim sesuatu yang tidak terjadi.
    UnappliedMutation { count: usize },
    /// Varian `Reply` yang tidak punya jalur render pesan.
    Unsupported(String),
    Other(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MediaSourceMissing { variant } => write!(
                f,
                "Reply::{variant} tidak punya sumber media — `media: None`, jadi tidak \
                 ada byte yang bisa di-upload. Pesan TIDAK dikirim."
            ),
            Self::Media(m) => write!(f, "{m}"),
            Self::Unsupported(m) => write!(f, "{m}"),
            Self::UnappliedMutation { count } => write!(
                f,
                "Reply::Mutate dengan {count} mutasi mencapai render_reply tanpa ditulis ke \
                 store dulu. Pesan TIDAK dikirim — mengirimpesan 'tersimpan' tanpa \
                 write = berbohong ke user."
            ),
            Self::Other(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Sumber untuk `Reply::TextQuoted` — pesan yang perlu di-quote.
#[derive(Debug, Clone)]
pub struct QuoteSource {
    pub stanza_id: String,
    pub participant: Option<String>,
    pub message: Box<wa::Message>,
}

impl QuoteSource {
    /// Dari pesan inbound yang memicu (`context_info.stanzaId` + `participant` +
    /// `quotedMessage` — `waproto-0.7.0/src/whatsapp.proto:1624-1631`).
    pub fn from_inbound(m: &InboundMessage) -> Option<Self> {
        Some(Self {
            stanza_id: m.info.id.clone(),
            participant: Some(m.info.source.sender.to_string()),
            message: m.message.as_ref().prepare_for_quote(),
        })
    }
}

/// Satu pesan siap kirim.
#[derive(Debug)]
pub struct Rendered {
    /// Berisi kalau reply ini sudah jadi wire-ready. KOSONG persis ketika
    /// [`Self::media`] `Some` — pesan aslinya baru ada setelah byte media
    /// di-upload, jadi tidak boleh dikirim apa adanya.
    pub message: wa::Message,
    /// Extra stanza node — hanya `biz` untuk kartu native-flow.
    pub extra_nodes: Vec<Node>,
    /// Media yang belum di-upload. `Some` = kirim lewat
    /// [`MessageSender::send_media`], bukan [`MessageSender::send`].
    pub media: Option<PendingMedia>,
    /// Kartu yang header/stiker-nya masih perlu di-upload. `Some` = kirim
    /// lewat [`MessageSender::send_card`].
    pub card: Option<PendingCard>,
    /// Preview link yang thumbnail-nya masih perlu di-upload. `Some` = kirim
    /// lewat [`MessageSender::send_preview`].
    pub preview: Option<PendingPreview>,
    /// Id pesan yang harus dipakai — pipeline mengaturnya supaya pesan
    /// berikutnya bisa mengaitkan diri lewat `messageAssociation`.
    pub message_id: Option<String>,
}

impl Rendered {
    pub(crate) fn wire(message: wa::Message, extra_nodes: Vec<Node>) -> Self {
        Self {
            message,
            extra_nodes,
            media: None,
            card: None,
            preview: None,
            message_id: None,
        }
    }

    /// Kartu yang belum bisa dikirim: media header atau stiker fake-quote
    /// masih di CDN.
    pub(crate) fn card(card: PendingCard, extra_nodes: Vec<Node>) -> Self {
        Self {
            message: wa::Message::default(),
            extra_nodes,
            media: None,
            card: Some(card),
            preview: None,
            message_id: None,
        }
    }

    /// Preview link yang thumbnail-nya belum ter-upload.
    pub(crate) fn preview(pending: PendingPreview) -> Self {
        Self {
            message: wa::Message::default(),
            extra_nodes: Vec::new(),
            media: None,
            card: None,
            preview: Some(pending),
            message_id: None,
        }
    }

    /// `false` kalau yang perlu dikirim media. Kalau `false`, `message`
    /// guaranteed terisi.
    pub fn needs_media(&self) -> bool {
        self.media.is_some() || self.card.is_some()
    }
}

/// Preview link yang thumbnail-nya belum naik ke CDN.
///
/// `sock.sendPreview` (JS) mengirim `extendedTextMessage` dengan `matchedText`
/// + `previewType` + thumbnail. Field thumbnail (`thumbnailDirectPath`,
/// `mediaKey`, `thumbnailSha256`, ...) berasal dari hasil upload, jadi
/// thumbnail harus di-upload dulu sebelum pesannya bisa disusun.
#[derive(Debug, Clone)]
pub struct PendingPreview {
    pub text: String,
    pub matched_text: String,
    pub title: Option<String>,
    pub description: Option<String>,
    /// WA `PreviewType`: 0 = NONE, 1 = VIDEO.
    pub preview_type: i32,
    pub thumbnail: Option<MediaSource>,
    pub context: Option<wa::ContextInfo>,
}

/// Kartu interaktif yang belum siap dikirim.
///
/// Bedanya dengan [`crate::cards::Card`] (yang sudah proto): media header
/// gambar/video dan stiker fake-quote harus naik ke CDN dulu
/// (`Client::upload`), dan baru setelah itu proto-nya bisa dirakit. Jadi
/// selama ini, kartu ditahan di sini dan dikirim lewat
/// [`MessageSender::send_card`].
#[derive(Debug, Clone)]
pub struct PendingCard {
    pub body: String,
    pub footer: Option<String>,
    pub header: Option<Box<CardHeader>>,
    pub buttons: Vec<oktz_plugin_api::registry::InteractiveButton>,
    pub message_params: Option<Value>,
    pub context: Option<Box<CardContext>>,
}

impl PendingCard {
    /// `true` kalau ada media yang harus di-upload sebelum kartu bisa dikirim.
    pub fn needs_upload(&self) -> bool {
        let header_needs_upload = matches!(
            self.header.as_ref().and_then(|h| h.media.as_ref()),
            Some(CardMedia::Image { .. }) | Some(CardMedia::Video { .. })
        );
        let quote_needs_upload = matches!(
            self.context.as_ref().and_then(|c| c.quote.as_deref()),
            Some(CardQuote::Sticker { .. })
        );
        header_needs_upload || quote_needs_upload
    }
}

/// Kartu + media yang sudah ter-upload → proto siap kirim.
fn build_card_proto(
    pending: &PendingCard,
    header_media: Option<HeaderMedia>,
    quote_message: Option<wa::Message>,
    trigger: Option<&QuoteSource>,
) -> Result<Card, PipelineError> {
    let mut card = Card::new(pending.body.clone());
    if let Some(f) = &pending.footer {
        card = card.footer(f.clone());
    }
    if let Some(h) = &pending.header {
        let mut header = Header {
            title: h.title.clone(),
            subtitle: h.subtitle.clone(),
            has_media_attachment: h.media.is_some(),
            media: header_media,
        };
        if header.media.is_none() {
            header.has_media_attachment = false;
        }
        card = card.header(header);
    }
    for b in &pending.buttons {
        let action = ButtonAction::from_name(&b.name).ok_or_else(|| {
            PipelineError::Unsupported(format!(
                "tombol native-flow `{}` tidak dikenal WA",
                b.name
            ))
        })?;
        card = card.button(Button::new(action, b.params.clone()));
    }
    if let Some(p) = &pending.message_params {
        card = card.message_params(p.clone());
    }
    if let Some(ctx) = build_card_context(
        pending.context.as_deref(),
        quote_message,
        trigger,
    ) {
        card = card.context_info(ctx);
    }
    Ok(card)
}

/// `SendOptions` untuk satu pesan: node `biz` untuk kartu native-flow, plus
/// id pesan kalau pipeline yang menentukan.
///
/// Id pesan WA = 16 karakter heks huruf besar (`wacore-0.7.0/src/request.rs:190`
/// — `RequestUtils::message_id_at`). Yang dipakai di sini id yang sama
/// bentuknya, supaya WA tidak menolak.
fn send_options(rendered: &Rendered) -> SendOptions {
    let mut opts = SendOptions::default();
    if !rendered.extra_nodes.is_empty() {
        opts = opts.with_extra_stanza_nodes(rendered.extra_nodes.clone());
    }
    if let Some(id) = &rendered.message_id {
        opts = opts.with_message_id(id.clone());
    }
    opts
}

/// Id pesan acak — 16 karakter heks huruf besar.
fn message_id_baru() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut acc = nanos ^ (seq.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut id = String::with_capacity(16);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for _ in 0..16 {
        id.push(HEX[(acc & 0x0F) as usize] as char);
        acc >>= 4;
        acc ^= acc.rotate_left(7);
    }
    id
}

/// `interactiveMessage.contextInfo` — `mention`, `forwarded`, surat berita,
/// dan pesan yang dirujuk.
fn build_card_context(
    context: Option<&CardContext>,
    quote_message: Option<wa::Message>,
    trigger: Option<&QuoteSource>,
) -> Option<wa::ContextInfo> {
    let ctx = context?;
    let mut ci = wa::ContextInfo {
        mentioned_jid: ctx.mentioned.clone(),
        // JS hanya menulis `isForwarded: true` — `false` tidak pernah muncul
        // di payload, jadi di sini juga dibiarkan kosong (bukan `Some(false)`).
        is_forwarded: ctx.is_forwarded.then_some(true),
        forwarding_score: (ctx.forwarding_score > 0).then_some(ctx.forwarding_score as u32),
        participant: ctx.participant.clone(),
        stanza_id: ctx.stanza_id.clone(),
        remote_jid: ctx.remote_jid.clone(),
        ..Default::default()
    };
    // `generateWAMessageFromContent` mengisi `stanzaId` dan `participant`
    // dari pesan yang dikutip (`onigis/lib/Utils/messages.js:577-580`).
    if ctx.quote_trigger
        && let Some(t) = trigger
    {
        if ci.stanza_id.is_none() {
            ci.stanza_id = Some(t.stanza_id.clone());
        }
        if ci.participant.is_none() {
            ci.participant = t.participant.clone();
        }
    }
    if let Some(n) = &ctx.forwarded_newsletter {
        ci.forwarded_newsletter_message_info = MessageField::some(
            wa::context_info::ForwardedNewsletterMessageInfo {
                newsletter_jid: Some(n.jid.clone()),
                newsletter_name: Some(n.name.clone()),
                server_message_id: Some(n.server_message_id as i32),
                ..Default::default()
            },
        );
    }
    if let Some(q) = quote_message {
        ci.quoted_message = MessageField::some(q);
    }
    Some(ci)
}

/// Pesan yang dirujuk kartu.
///
/// Dua sumber: pesan palsu yang ditulis plugin (`CardQuote`), atau pesan
/// pemicu (`quoted: m` di JS). Keduanya berakhir di
/// `interactiveMessage.contextInfo.quotedMessage` — karena
/// `normalizeMessageContent` membuka wrapper `viewOnceMessage`, kutipan dari
/// `generateWAMessageFromContent` mendarat di situ, bukan di `contextInfo`
/// pesan luar (`onigis/lib/Utils/messages.js:558-610`).
fn card_quote_message(
    context: Option<&CardContext>,
    target_quote: Option<&QuoteSource>,
) -> Option<wa::Message> {
    let ctx = context?;
    if ctx.quote_trigger {
        return target_quote.map(|q| *q.message.clone());
    }
    ctx.quote.as_deref().and_then(build_quote_message)
}

/// Pesan yang dirujuk kartu, kalau bukan stiker (stiker butuh upload).
fn build_quote_message(quote: &CardQuote) -> Option<wa::Message> {
    match quote {
        CardQuote::Contact {
            display_name,
            vcard,
        } => Some(wa::Message {
            contact_message: MessageField::some(wa::message::ContactMessage {
                display_name: Some(display_name.clone()),
                vcard: Some(vcard.clone()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        CardQuote::Location {
            latitude,
            longitude,
            name,
            thumbnail,
        } => Some(wa::Message {
            location_message: MessageField::some(wa::message::LocationMessage {
                degrees_latitude: Some(*latitude),
                degrees_longitude: Some(*longitude),
                name: Some(name.clone()),
                jpeg_thumbnail: thumbnail.clone(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        CardQuote::Video {
            caption,
            seconds,
            mimetype,
            thumbnail,
            file_length,
        } => Some(wa::Message {
            video_message: MessageField::some(wa::message::VideoMessage {
                caption: Some(caption.clone()),
                seconds: Some(*seconds),
                mimetype: Some(mimetype.clone()),
                jpeg_thumbnail: thumbnail.clone(),
                file_length: Some(*file_length),
                ..Default::default()
            }),
            ..Default::default()
        }),
        CardQuote::Text(text) => Some(wa::Message::text(text)),
        // Stiker butuh upload — bukan urusan builder proto.
        CardQuote::Sticker { .. } => None,
    }
}

/// `locationMessage` untuk header kartu.
fn build_location_message(loc: &CardLocation) -> wa::message::LocationMessage {
    wa::message::LocationMessage {
        degrees_latitude: Some(loc.latitude),
        degrees_longitude: Some(loc.longitude),
        name: Some(loc.name.clone()),
        address: loc.address.clone(),
        jpeg_thumbnail: loc.thumbnail.clone(),
        ..Default::default()
    }
}

/// Jenis media yang di-upload dan dikirim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Sticker,
    Video,
    Audio,
    Document,
}

impl MediaKind {
    /// Tipe untuk `Client::upload`. Dipisah supaya pemetaan ini bisa diuji
    /// tanpa jaringan.
    pub fn upload_type(self) -> MediaType {
        match self {
            Self::Image => MediaType::Image,
            Self::Sticker => MediaType::Sticker,
            Self::Video => MediaType::Video,
            Self::Audio => MediaType::Audio,
            Self::Document => MediaType::Document,
        }
    }
}

/// Satu balasan media yang menunggu di-upload dan dikirim.
#[derive(Debug, Clone)]
pub struct PendingMedia {
    pub kind: MediaKind,
    pub source: MediaSource,
    pub caption: String,
    pub mentions: Vec<String>,
    pub quote: Option<QuoteSource>,
    /// `{ type: 'video', gifPlayback: true }` (`socket.js:793`).
    pub gif_playback: bool,
    /// `{ type: 'audio', ptt: true }` (`socket.js:802`).
    pub ptt: bool,
    /// `fileName` dokumen. `None` → `"file"` seperti `socket.js:775`.
    pub file_name: Option<String>,
    /// `jpegThumbnail` — thumbnail yang ditampilkan di chat.
    ///
    /// `serialize.js:1022-1024` mengirim `safeThumbnail(foto2, 300)` untuk
    /// varian reply 2 dan 6; tanpa ini chat menampilkan dokumen tanpa
    /// gambar.
    pub jpeg_thumbnail: Option<Vec<u8>>,
    /// `fileLength` — dipaksa jadi `99999999999999` pada varian reply 2/6
    /// (`serialize.js:1019`).
    pub file_length: Option<u64>,
    /// `messageAssociation` — `(id induk, jid induk)`.
    ///
    /// `menu.js:1223-1228` mengirim video kedua yang menempel pada kartu
    /// menu: `messageAssociation { associationType: 12, parentMessageKey }`.
    pub association: Option<(String, String)>,
    /// `contextInfo.pairedMediaType` — media yang berpasangan dengan pesan
    /// lain (`menu.js:1183-1185` untuk gambar, `:1217-1220` untuk video).
    pub paired: bool,
}

/// `UploadResponse` → `wa::Message` siap kirim.
///
/// Image memakai builder resmi library (`media::image_message`,
/// `whatsapp-rust-0.7.0/src/media.rs:67`) supaya tidak merakit sendiri field
/// CDN-nya. Untuk sticker library TIDAK menyediakan builder
/// (`media.rs:66-176` hanya `image_message`/`video_message`/
/// `document_message`/`audio_message`), jadi `stickerMessage` dirakit dari
/// `UploadResponse` dengan field yang sama seperti builder image
/// (`waproto-0.7.0/src/whatsapp.proto:1584-1609`).
fn build_media_message(kind: MediaKind, up: UploadResponse, media: &PendingMedia) -> wa::Message {
    let ci = if media.mentions.is_empty() && media.quote.is_none() {
        None
    } else {
        Some(Box::new(context_info(&media.mentions, media.quote.as_ref())))
    };
    // WA menolak caption kosong; `Some("")` dikirim apa adanya.
    let caption = (!media.caption.is_empty()).then(|| media.caption.clone());
    let mut msg = match kind {
        MediaKind::Image => image_message(up, image_options(media, caption, ci)),
        MediaKind::Video => video_message(up, video_options(media, caption, ci)),
        MediaKind::Audio => audio_message(up, audio_options(media, ci)),
        MediaKind::Document => document_message(up, document_options(media, caption, ci)),
        MediaKind::Sticker => wa::Message {
            sticker_message: MessageField::some(wa::message::StickerMessage {
                url: Some(up.url),
                direct_path: Some(up.direct_path),
                media_key: Some(up.media_key.to_vec()),
                file_sha256: Some(up.file_sha256.to_vec()),
                file_enc_sha256: Some(up.file_enc_sha256.to_vec()),
                file_length: Some(up.file_length),
                media_key_timestamp: Some(up.media_key_timestamp),
                // Sticker WA Web memang webp; `image/webp` hanya label, byte
                // yang di-upload adalah apa adanya dari `MediaSource`.
                mimetype: Some(
                    media
                        .source
                        .mimetype()
                        .unwrap_or("image/webp")
                        .to_string(),
                ),
                context_info: ci.map_or_else(MessageField::none, |c| {
                    MessageField::some(*c)
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
    };
    // `messageAssociation` (`menu.js:1223-1228`) — video yang menempel pada
    // pesan sebelumnya. Nilai `associationType` di wire = 12, sama seperti
    // yang ditulis JS; di proto itu `MOTION_PHOTO`.
    if let Some((parent_id, parent_jid)) = &media.association {
        msg.message_context_info = MessageField::some(wa::MessageContextInfo {
            message_association: MessageField::some(wa::MessageAssociation {
                association_type: Some(wa::message_association::AssociationType::MOTION_PHOTO),
                parent_message_key: MessageField::some(wa::MessageKey {
                    remote_jid: Some(parent_jid.clone()),
                    from_me: Some(true),
                    id: Some(parent_id.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    msg
}

/// Upload satu sumber media ke CDN.
async fn upload_source(
    client: &Client,
    source: &MediaSource,
    media_type: MediaType,
) -> Result<UploadResponse, PipelineError> {
    let data = load_media(source).await?;
    client
        .upload(data, media_type, UploadOptions::default())
        .await
        .map_err(|e| PipelineError::Media(format!("upload media gagal: {e}")))
}

/// `imageMessage` dari hasil upload — dipakai untuk header kartu.
fn image_proto(up: UploadResponse, source: &MediaSource) -> wa::message::ImageMessage {
    let msg = image_message(
        up,
        ImageOptions {
            mimetype: source.mimetype().map(str::to_string),
            ..Default::default()
        },
    );
    msg.image_message
        .as_option()
        .cloned()
        .unwrap_or_default()
}

/// `videoMessage` dari hasil upload — dipakai untuk header kartu.
fn video_proto(up: UploadResponse, source: &MediaSource) -> wa::message::VideoMessage {
    let msg = video_message(
        up,
        VideoOptions {
            mimetype: source.mimetype().map(str::to_string),
            ..Default::default()
        },
    );
    msg.video_message
        .as_option()
        .cloned()
        .unwrap_or_default()
}

/// `stickerMessage` dari hasil upload — dipakai untuk fake-quote stiker
/// (`plugins/main/menu.js:1151-1163`).
fn sticker_proto(up: UploadResponse, source: &MediaSource) -> wa::Message {
    wa::Message {
        sticker_message: MessageField::some(wa::message::StickerMessage {
            url: Some(up.url),
            direct_path: Some(up.direct_path),
            media_key: Some(up.media_key.to_vec()),
            file_sha256: Some(up.file_sha256.to_vec()),
            file_enc_sha256: Some(up.file_enc_sha256.to_vec()),
            file_length: Some(up.file_length),
            media_key_timestamp: Some(up.media_key_timestamp),
            mimetype: Some(
                source
                    .mimetype()
                    .unwrap_or("image/webp")
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Opsi per jenis media, dipisah dari `build_media_message` supaya bisa diuji
/// tanpa jaringan: `UploadResponse` itu `#[non_exhaustive]`, jadi tidak bisa
/// dibuat dari luar crate `whatsapp-rust`.
fn image_options(
    media: &PendingMedia,
    caption: Option<String>,
    ci: Option<Box<wa::ContextInfo>>,
) -> ImageOptions {
    ImageOptions {
        caption,
        mimetype: media.source.mimetype().map(str::to_string),
        jpeg_thumbnail: media.jpeg_thumbnail.clone(),
        context_info: ci,
    }
}

/// `video_message` (`whatsapp-rust-0.7.0/src/media.rs:92`). Mimetype bawaannya
/// `video/mp4`, sama seperti yang WA kirim untuk `{ video }`.
fn video_options(
    media: &PendingMedia,
    caption: Option<String>,
    ci: Option<Box<wa::ContextInfo>>,
) -> VideoOptions {
    let mut ci = ci.map(|c| *c);
    if media.paired
        && let Some(c) = ci.as_mut()
    {
        // `pairedMediaType: 6` + `statusSourceType: 0` (`menu.js:1217-1220`).
        c.paired_media_type = Some(wa::context_info::PairedMediaType::MOTION_PHOTO_CHILD);
        c.status_source_type = Some(wa::context_info::StatusSourceType::IMAGE);
    }
    VideoOptions {
        caption,
        mimetype: media.source.mimetype().map(str::to_string),
        jpeg_thumbnail: media.jpeg_thumbnail.clone(),
        duration_seconds: None,
        gif_playback: Some(media.gif_playback),
        context_info: ci.map(Box::new),
    }
}

/// `audio_message` (`media.rs:150`). JS menulis `audio/mpeg` kalau pemanggil
/// tidak memberi mimetype (`socket.js:798`), jadi itu yang dipakai di sini —
/// bukan default `audio/ogg; codecs=opus` builder.
fn audio_options(media: &PendingMedia, ci: Option<Box<wa::ContextInfo>>) -> AudioOptions {
    AudioOptions {
        mimetype: Some(
            media
                .source
                .mimetype()
                .unwrap_or("audio/mpeg")
                .to_string(),
        ),
        duration_seconds: None,
        ptt: Some(media.ptt),
        waveform: None,
        context_info: ci,
    }
}

/// `document_message` (`media.rs:119`). `fileName` bawaannya `"file"` dan
/// mimetype bawaannya `application/octet-stream` — persis `socket.js:775-776`.
fn document_options(
    media: &PendingMedia,
    caption: Option<String>,
    ci: Option<Box<wa::ContextInfo>>,
) -> DocumentOptions {
    DocumentOptions {
        mimetype: Some(
            media
                .source
                .mimetype()
                .unwrap_or("application/octet-stream")
                .to_string(),
        ),
        file_name: Some(
            media
                .file_name
                .clone()
                .unwrap_or_else(|| "file".to_string()),
        ),
        title: None,
        caption,
        page_count: None,
        jpeg_thumbnail: media.jpeg_thumbnail.clone(),
        context_info: ci,
    }
}

pub(crate) fn context_info(mentions: &[String], quote: Option<&QuoteSource>) -> wa::ContextInfo {
    wa::ContextInfo {
        mentioned_jid: mentions.to_vec(),
        quoted_message: quote
            .map(|q| MessageField::some(*q.message.clone()))
            .unwrap_or_default(),
        stanza_id: quote.map(|q| q.stanza_id.clone()),
        participant: quote.and_then(|q| q.participant.clone()),
        ..Default::default()
    }
}

/// Ubah `Reply` menjadi pesan yang bisa dikirim.
///
/// `Ok(None)` = jangan kirim apa pun (`Reply::None`).
///
/// `Reply::Mutate` yang tiba di sini itu ERROR, bukan "ambil bagian
/// dalamnya": mutasi adalah efek samping yang harus ditulis store, dan
/// fungsi ini tidak punya akses ke store. `Pipeline::process` memanggil
/// [`Reply::into_parts`] lebih dulu supaya tulisnya terjadi sebelum
/// balasan dikirim. Kalau `Mutate` tetap sampai ke sini, ada pemanggil
/// yang melewatkan langkah itu — dan diam-diamnya membuang write
///ersistent adalah kegagalan yang paling mahal, jadi ia jadi error
/// bertipe.
///
/// Per-varian:
/// - `Text` → mengutip pemicu dan menyebut pengirim secara default
///   (`serialize.js:983-1010`). Tanpa pemicu dan tanpa mention → `conversation`
///   polos; kalau ada → `extendedTextMessage` + `context_info.mentioned_jid`
///   (`waproto-0.7.0/src/whatsapp.proto:1629`).
/// - `NoQuote` → `quoted: false`, `TextNoMention` → `mentions: []`.
/// - `TextQuoted` → sama, ditambah `context_info.quoted_message`. Tanpa
///   `quote`, teks tetap terkirim tapi quote hilang.
/// - `Interactive` → `Card::new(body).to_message()` + node `biz`
///   (bentuk yang sudah terbukti, `crate::cards`).
/// - `Image` / `Sticker` / `Video` / `Audio` / `Document` → [`Rendered::media`]
///   terisi, `message` sengaja kosong: byte-nya harus di-upload dulu sebelum ada
///   pesan yang bisa dikirim. `media: None` → `Err(MediaSourceMissing)`, bukan
///   `Ok(None)` — drop senyap akan membuat plugin terlihat sukses padahal tidak
///   mengirim apa pun.
#[derive(Debug, Clone, Default)]
pub struct ReplyTarget {
    pub quote: Option<QuoteSource>,
    pub sender: String,
}

/// `m` versi pipeline: pemicu (untuk kutipan) + pengirim (untuk mention
/// default). Satu-satunya cara membuat target supaya tidak ada tempat yang
/// lupa mengirim `sender`.
pub fn reply_target(m: &InboundMessage, sender: &str) -> ReplyTarget {
    ReplyTarget {
        quote: QuoteSource::from_inbound(m),
        sender: sender.to_string(),
    }
}

pub fn render_reply(
    reply: &Reply,
    target: &ReplyTarget,
) -> Result<Option<Rendered>, PipelineError> {
    let quote = target.quote.as_ref();
    // `serialize.js:994` — `[]` di JS berarti TIDAK ada mention, jadi daftar
    // kosong tidak bisa berarti "pakai default".
    // Sender kosong = tidak ada pemicu nyata (test/kode non-WA), jadi tidak ada
    // mention. Kalau diisi, JS menulis `[m.sender]`.
    let default_mention = || {
        if target.sender.is_empty() {
            Vec::new()
        } else {
            vec![target.sender.clone()]
        }
    };
    let mentions_of = |m: &[String]| {
        if m.is_empty() {
            default_mention()
        } else {
            m.to_vec()
        }
    };
    let pending = |kind: MediaKind,
                   variant: &'static str,
                   src: &Option<MediaSource>,
                   caption: &str,
                   mentions: &[String]|
     -> Result<PendingMedia, PipelineError> {
        Ok(PendingMedia {
            kind,
            source: src
                .clone()
                .ok_or(PipelineError::MediaSourceMissing { variant })?,
            caption: caption.to_string(),
            mentions: mentions_of(mentions),
            quote: quote.cloned(),
            gif_playback: false,
            ptt: false,
            file_name: None,
            jpeg_thumbnail: None,
            file_length: None,
            association: None,
            paired: false,
        })
    };
    match reply {
        Reply::None => Ok(None),
        Reply::Sticker { media: src } => Ok(Some(Rendered {
            message: wa::Message::default(),
            extra_nodes: Vec::new(),
            media: Some(pending(MediaKind::Sticker, "Sticker", src, "", &[])?),
            card: None,
            preview: None,
            message_id: None,
        })),
        Reply::Image {
            media: src,
            caption,
            mentions,
        } => Ok(Some(Rendered {
            message: wa::Message::default(),
            extra_nodes: Vec::new(),
            media: Some(pending(MediaKind::Image, "Image", src, caption, mentions)?),
            card: None,
            preview: None,
            message_id: None,
        })),
        Reply::Video {
            media: src,
            caption,
            mentions,
            gif_playback,
        } => {
            let mut media = pending(MediaKind::Video, "Video", src, caption, mentions)?;
            media.gif_playback = *gif_playback;
            Ok(Some(Rendered {
                message: wa::Message::default(),
                extra_nodes: Vec::new(),
                media: Some(media),
                card: None,
                preview: None,
                message_id: None,
            }))
        }
        Reply::PairedVideo {
            media: src,
            caption,
            mentions,
        } => {
            let mut media = pending(MediaKind::Video, "PairedVideo", src, caption, mentions)?;
            media.paired = true;
            Ok(Some(Rendered {
                message: wa::Message::default(),
                extra_nodes: Vec::new(),
                media: Some(media),
                card: None,
                preview: None,
                message_id: None,
            }))
        }
        Reply::Audio { media: src, ptt } => {
            let mut media = pending(MediaKind::Audio, "Audio", src, "", &[])?;
            media.ptt = *ptt;
            Ok(Some(Rendered {
                message: wa::Message::default(),
                extra_nodes: Vec::new(),
                media: Some(media),
                card: None,
                preview: None,
                message_id: None,
            }))
        }
        Reply::Document {
            media: src,
            file_name,
            caption,
            mentions,
        } => {
            let mut media = pending(MediaKind::Document, "Document", src, caption, mentions)?;
            media.file_name = Some(file_name.clone());
            Ok(Some(Rendered {
                message: wa::Message::default(),
                extra_nodes: Vec::new(),
                media: Some(media),
                card: None,
                preview: None,
                message_id: None,
            }))
        }
        // `quotedMsg = options.quoted !== false ? msg : undefined` — jadi
        // yang TANPA kutipan harus opt-out eksplisit.
        Reply::Text { text, mentions } => Ok(Some(Rendered::wire(
            text_message(text, &mentions_of(mentions), quote),
            Vec::new(),
        ))),
        Reply::TextQuoted { text, mentions } => Ok(Some(Rendered::wire(
            text_message(text, &mentions_of(mentions), quote),
            Vec::new(),
        ))),
        Reply::NoQuote { text, mentions } => Ok(Some(Rendered::wire(
            text_message(text, &mentions_of(mentions), None),
            Vec::new(),
        ))),
        Reply::TextNoMention { text } => Ok(Some(Rendered::wire(
            text_message(text, &[], quote),
            Vec::new(),
        ))),
        Reply::Interactive {
            body,
            footer,
            buttons,
            header,
            message_params,
            context,
        } => {
            let pending = PendingCard {
                body: body.clone(),
                footer: footer.clone(),
                header: header.clone(),
                buttons: buttons.clone(),
                message_params: message_params.clone(),
                context: context.clone(),
            };
            if pending.needs_upload() {
                // Header gambar/video atau stiker fake-quote harus naik ke CDN
                // dulu; kartu ditahan sampai sisanya siap.
                return Ok(Some(Rendered::card(pending, vec![biz_node()])));
            }
            let quote = card_quote_message(context.as_deref(), quote);
            let card = build_card_proto(&pending, None, quote, target.quote.as_ref())?;
            Ok(Some(Rendered::wire(card.to_message(), vec![biz_node()])))
        }
        Reply::Preview {
            text,
            matched_text,
            title,
            description,
            thumbnail,
            preview_type,
            context,
        } => {
            let ctx_info = match context.as_deref() {
                Some(c) => build_card_context(Some(c), None, quote),
                None => None,
            };
            let ctx_for_ext = ctx_info.clone();
            let pending = PendingPreview {
                text: text.clone(),
                matched_text: matched_text.clone(),
                title: title.clone(),
                description: description.clone(),
                preview_type: *preview_type,
                thumbnail: thumbnail.clone(),
                context: ctx_info,
            };
            if pending.thumbnail.is_some() {
                return Ok(Some(Rendered::preview(pending)));
            }
            let ext = wa::message::ExtendedTextMessage {
                text: Some(text.clone()),
                matched_text: Some(matched_text.clone()),
                title: title.clone(),
                description: description.clone(),
                preview_type: Some(match *preview_type {
                    1 => wa::message::extended_text_message::PreviewType::Video,
                    _ => wa::message::extended_text_message::PreviewType::None,
                }),
                context_info: match ctx_for_ext {
                    Some(c) => MessageField::some(c),
                    None => MessageField::none(),
                },
                ..Default::default()
            };
            let message = wa::Message {
                extended_text_message: MessageField::some(ext),
                ..Default::default()
            };
            Ok(Some(Rendered::wire(message, Vec::new())))
        }
        Reply::Table {
            title,
            headers,
            rows,
            header_text,
            footer,
        } => {
            let table = crate::tables::Table {
                title: title.clone(),
                headers: headers.clone(),
                rows: rows.clone(),
                header_text: header_text.clone(),
                footer: footer.clone(),
            };
            let ci = context_info(&default_mention(), quote);
            Ok(Some(Rendered::wire(
                crate::tables::build_table(&table, Some(ci)),
                Vec::new(),
            )))
        }
        // Reaksi bukan message; dikirim lewat `send_reaction`, bukan render.
        Reply::Reaction { .. } => Err(PipelineError::Unsupported(
            "Reaction harus lewat send_reaction, bukan render".into(),
        )),
        // `Sequence` handled pipeline sebelum `render_reply` dipanggil (satu
        // pemanggilan = satu pesan, sequence perlu loop). Kalau sampai sini,
        // pemanggilnya salah.
        Reply::Sequence(_) => Err(PipelineError::Unsupported(
            "Sequence harus dikirim lewat loop pipeline, bukan render_reply".into(),
        )),
        // Pagar: mutasi yang belum ditulis store TIDAK boleh dirender
        // menjadi pesan "✅ tersimpan" yang bohong. Lihat catatan fungsi.
        Reply::Mutate { mutations, .. } => Err(PipelineError::UnappliedMutation {
            count: mutations.len(),
        }),
    }
}

fn text_message(text: &str, mentions: &[String], quote: Option<&QuoteSource>) -> wa::Message {
    if mentions.is_empty() && quote.is_none() {
        // `MessageBuilderExt::text` (wacore-0.7.0/src/proto_helpers.rs:105) —
        // `conversation` polos, paling kecil di wire.
        wa::Message::text(text)
    } else {
        wa::Message::text_with_context(text, context_info(mentions, quote))
    }
}

/// `middleware.js:167-171`. Alasan dan durasi datang dari `DenyMode::Afk`
/// — teks tanpa keduanya bukan kalimat yang sama dengan yang produksi.
pub fn afk_text(reason: &str, elapsed_ms: i64) -> String {
    format!(
        "💤 *ʙᴏᴛ sᴇᴅᴀɴɢ ᴀꜰᴋ*\n\n\
╭┈┈⬡「 📋 *ɪɴꜰᴏ* 」\n\
┃ 📝 ᴀʟᴀsᴀɴ: `{reason}`\n\
┃ ⏱️ sᴇᴊᴀᴋ: `{}` yang lalu\n\
╰┈┈⬡\n\n\
> Bot tidak bisa menerima perintah saat ini\n\
> Mohon tunggu sampai owner mengaktifkan kembali",
        oktz_router::mode::format_afk_duration(elapsed_ms)
    )
}

/// `middleware.js:182-186`.
pub fn only_this_group_text(name: &str, link: &str) -> String {
    format!(
        "🔒 *Akses Ditolak*\n\n\
Maaf, saat ini bot kami hanya bisa diakses dan digunakan secara eksklusif di Grup Utama (*{name}*).\n\n\
Silakan bergabung ke grup utama kami melalui tautan berikut:\n\
🔗 {link}\n\n\
Setelah bergabung, Anda bebas menggunakan semua fitur bot. Terima kasih!"
    )
}

/// `Decision` → balasan yang dikirim.
pub fn decision_to_reply(d: &Decision) -> Reply {
    match d {
        Decision::Handled(r) => r.clone(),
        Decision::DeniedPermission(msg) | Decision::DeniedGroupMode(msg) => Reply::text(msg),
        Decision::DeniedMode(DenyMode::Afk { reason, elapsed_ms }) => {
            Reply::text(afk_text(reason, *elapsed_ms))
        }
        Decision::DeniedMode(DenyMode::OnlyThisGroup { name, link }) => {
            Reply::text(only_this_group_text(name, link))
        }
        // `middleware.js:200-202` self-group dan `middleware.js:177-178` onlyGc/
        // onlyPc menolak tanpa pesan; `DenyMode::Silent` juga tidak membawa teks.
        Decision::DeniedMode(DenyMode::Silent | DenyMode::SelfGroup) => Reply::None,
        Decision::NotACommand | Decision::NoPlugin => Reply::None,
    }
}

// =============================================================================
// 6. Ekstraksi body — `oktz_router::serialize`
// =============================================================================

/// Nama field protobuf untuk pesan ini, `""` kalau di luar daftar serialize.
/// `context_info` pertama yang ada di pesan. WA menaruhnya di variant yang
/// berbeda-beda; satu penjelajahuan dipakai untuk mention maupun quote supaya
/// keduanya tidak bisa melihat sumber berbeda untuk pesan yang sama.
fn context_info_of(m: &wa::Message) -> Option<&wa::ContextInfo> {
    let b = m.get_base_message();
    [
        b.extended_text_message.as_option().and_then(|v| v.context_info.as_option()),
        b.image_message.as_option().and_then(|v| v.context_info.as_option()),
        b.video_message.as_option().and_then(|v| v.context_info.as_option()),
        b.document_message.as_option().and_then(|v| v.context_info.as_option()),
        b.audio_message.as_option().and_then(|v| v.context_info.as_option()),
        b.sticker_message.as_option().and_then(|v| v.context_info.as_option()),
        b.contact_message.as_option().and_then(|v| v.context_info.as_option()),
        b.location_message.as_option().and_then(|v| v.context_info.as_option()),
        b.buttons_response_message
            .as_option()
            .and_then(|v| v.context_info.as_option()),
        b.list_response_message
            .as_option()
            .and_then(|v| v.context_info.as_option()),
        b.template_button_reply_message
            .as_option()
            .and_then(|v| v.context_info.as_option()),
        b.poll_creation_message
            .as_option()
            .and_then(|v| v.context_info.as_option()),
        b.poll_creation_message_v2
            .as_option()
            .and_then(|v| v.context_info.as_option()),
        b.poll_creation_message_v3
            .as_option()
            .and_then(|v| v.context_info.as_option()),
        b.view_once_message
            .as_option()
            .and_then(|v| v.message.as_option())
            .and_then(context_info_of),
        b.ephemeral_message
            .as_option()
            .and_then(|v| v.message.as_option())
            .and_then(context_info_of),
    ]
    .into_iter()
    .flatten()
    .next()
}

fn mentioned_jids(m: &InboundMessage) -> Vec<String> {
    context_info_of(&m.message)
        .map(|c| c.mentioned_jid.clone())
        .unwrap_or_default()
}

/// `detectSwGcType(rawMsg)` (`group-protection.js:467-520`) — path tipe
/// SW-group ("share status ke grup") di pohon pesan.
///
/// JS menelusuri SELURUH pohon pesan secara generik dan mengembalikan path
/// seperti `message.groupMentionedMessage` atau
/// `message.extendedTextMessage.contextInfo.groupMentions`. Path itu dipakai
/// sebagai `%type%` di `gpMsg("antiswgc")`.
///
/// Port Rust memakai padanan protobuf (waproto 0.7):
/// - `group_mentioned_message`, `status_mention_message`,
///   `group_status_mention_message`, `group_status_message`,
///   `group_status_message_v2` — wrapper SW-group;
/// - `context_info.group_mentions` — mention grup di context info.
///
/// `ponytail:` `groupStatusMessageV2Extension` tidak ada di waproto 0.7, jadi
/// tidak bisa dideteksi. `ponytail:` wrapper `device_sent`/`ephemeral`/
/// `view_once` tidak menambah prefiks path seperti di JS — isi `%type%` tetap
/// menunjuk tipe yang sama.
fn sw_group_type_of(m: &wa::Message) -> Option<String> {
    let b = m.get_base_message();
    if b.group_mentioned_message.as_option().is_some() {
        return Some("message.groupMentionedMessage".to_string());
    }
    if b.status_mention_message.as_option().is_some() {
        return Some("message.statusMentionMessage".to_string());
    }
    if b.group_status_mention_message.as_option().is_some() {
        return Some("message.groupStatusMentionMessage".to_string());
    }
    if b.group_status_message.as_option().is_some() {
        return Some("message.groupStatusMessage".to_string());
    }
    if b.group_status_message_v2.as_option().is_some() {
        return Some("message.groupStatusMessageV2".to_string());
    }
    // `contextInfoOf(node).groupMentions.length > 0` — satu mention grup
    // sudah cukup; JS tidak mensyaratkan lebih dari satu.
    let variants: [(&str, Option<&wa::ContextInfo>); 13] = [
        (
            "extendedTextMessage",
            b.extended_text_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "imageMessage",
            b.image_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "videoMessage",
            b.video_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "documentMessage",
            b.document_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "audioMessage",
            b.audio_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "stickerMessage",
            b.sticker_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "contactMessage",
            b.contact_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "locationMessage",
            b.location_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "buttonsResponseMessage",
            b.buttons_response_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "listResponseMessage",
            b.list_response_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "templateButtonReplyMessage",
            b.template_button_reply_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "pollCreationMessage",
            b.poll_creation_message
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
        (
            "pollCreationMessageV2",
            b.poll_creation_message_v2
                .as_option()
                .and_then(|v| v.context_info.as_option()),
        ),
    ];
    for (nama, ci) in variants {
        if let Some(ci) = ci
            && !ci.group_mentions.is_empty()
        {
            return Some(format!("message.{nama}.contextInfo.groupMentions"));
        }
    }
    None
}

/// `m.quoted` — pesan yang di-reply pengirim.
fn quoted_of(m: &InboundMessage) -> Option<QuotedMessage> {
    let ci = context_info_of(&m.message)?;
    let quoted = ci.quoted_message.as_option()?;
    Some(QuotedMessage {
        id: ci.stanza_id.clone().unwrap_or_default(),
        // `participant` = pengirim asli di grup. Kalau kosong, pesan itu
        // dikirim di private chat oleh `sender` — bukan berarti pesannya dari
        // bot, jadi `from_me` harus dibaca terpisah.
        sender: ci
            .participant
            .clone()
            .unwrap_or_else(|| m.info.source.sender.to_string()),
        chat: m.info.source.chat.to_string(),
        text: quoted_body(quoted),
        kind: quoted_kind(quoted),
        mimetype: quoted_mimetype(quoted),
        file_name: quoted_file_name(quoted),
        from_me: m.info.source.is_from_me,
    })
}

fn quoted_kind(m: &wa::Message) -> QuotedKind {
    let b = m.get_base_message();
    if b.conversation.is_some() || b.extended_text_message.as_option().is_some() {
        QuotedKind::Text
    } else if b.image_message.as_option().is_some() {
        QuotedKind::Image
    } else if b.video_message.as_option().is_some() {
        QuotedKind::Video
    } else if b.audio_message.as_option().is_some() {
        QuotedKind::Audio
    } else if b.sticker_message.as_option().is_some() {
        QuotedKind::Sticker
    } else if b.document_message.as_option().is_some() {
        QuotedKind::Document
    } else if b.contact_message.as_option().is_some() {
        QuotedKind::Contact
    } else if b.location_message.as_option().is_some() {
        QuotedKind::Location
    } else if b.reaction_message.as_option().is_some() {
        QuotedKind::Reaction
    } else if b.poll_creation_message.as_option().is_some()
        || b.poll_creation_message_v2.as_option().is_some()
        || b.poll_creation_message_v3.as_option().is_some()
    {
        QuotedKind::Poll
    } else {
        QuotedKind::Other(wire_type_of(m).to_string())
    }
}

/// `getMessageBody` (`src/lib/serialize.js:327-372`) — sama persis dengan yang
/// dipakai untuk pesan biasa, termasuk `""` untuk tipe yang tidak punya body.
fn quoted_body(m: &wa::Message) -> String {
    extract_body(m).unwrap_or_default()
}

fn quoted_mimetype(m: &wa::Message) -> Option<String> {
    let b = m.get_base_message();
    b.image_message
        .as_option()
        .and_then(|v| v.mimetype.clone())
        .or_else(|| {
            b.video_message
                .as_option()
                .and_then(|v| v.mimetype.clone())
        })
        .or_else(|| {
            b.audio_message
                .as_option()
                .and_then(|v| v.mimetype.clone())
        })
        .or_else(|| {
            b.sticker_message
                .as_option()
                .and_then(|v| v.mimetype.clone())
        })
        .or_else(|| {
            b.document_message
                .as_option()
                .and_then(|v| v.mimetype.clone())
        })
}

fn quoted_file_name(m: &wa::Message) -> Option<String> {
    let b = m.get_base_message();
    b.document_message
        .as_option()
        .and_then(|v| v.file_name.clone())
}

fn wire_type_of(m: &wa::Message) -> &'static str {
    use MessageKind as K;
    let b = m.get_base_message();
    let pick: Option<K> = if b.conversation.is_some() {
        Some(K::Conversation)
    } else if b.extended_text_message.as_option().is_some() {
        Some(K::ExtendedText)
    } else if b.image_message.as_option().is_some() {
        Some(K::Image)
    } else if b.video_message.as_option().is_some() {
        Some(K::Video)
    } else if b.document_message.as_option().is_some() {
        Some(K::Document)
    } else if b.buttons_response_message.as_option().is_some() {
        Some(K::ButtonsResponse)
    } else if b.list_response_message.as_option().is_some() {
        Some(K::ListResponse)
    } else if b.template_button_reply_message.as_option().is_some() {
        Some(K::TemplateButtonReply)
    } else if b.interactive_response_message.as_option().is_some() {
        Some(K::InteractiveResponse)
    } else if b.poll_creation_message.as_option().is_some() {
        Some(K::PollCreation)
    } else if b.poll_creation_message_v2.as_option().is_some() {
        Some(K::PollCreationV2)
    } else if b.poll_creation_message_v3.as_option().is_some() {
        Some(K::PollCreationV3)
    } else {
        None
    };

    if let Some(k) = pick {
        return k.wire_type();
    }
    // `messageContextInfo` + native-flow response → dipaksa jadi interactive
    // (`serialize.rs:111-120`).
    if b.message_context_info.as_option().is_some() && interactive_params_json(b).is_some() {
        return K::InteractiveResponse.wire_type();
    }
    ""
}

fn interactive_params_json(m: &wa::Message) -> Option<String> {
    let ir = m.get_base_message().interactive_response_message.as_option()?;
    match ir.interactive_response_message.as_ref()? {
        IrOneof::NativeFlowResponseMessage(nf) => nf.params_json.clone(),
    }
}

/// Body pesan masuk, sesuai `getMessageBody` (`src/lib/serialize.js:327-372`).
pub fn extract_body(m: &wa::Message) -> Option<String> {
    let wire = wire_type_of(m);
    if wire.is_empty() {
        return None;
    }
    let kind = MessageKind::from_wire_type(wire);
    let b = m.get_base_message();
    let params_json = interactive_params_json(b);
    let params = MessageParams {
        conversation: b.conversation.as_deref(),
        text: b
            .extended_text_message
            .as_option()
            .and_then(|e| e.text.as_deref()),
        caption: b
            .image_message
            .as_option()
            .and_then(|i| i.caption.as_deref())
            .or_else(|| {
                b.video_message
                    .as_option()
                    .and_then(|i| i.caption.as_deref())
            })
            .or_else(|| {
                b.document_message
                    .as_option()
                    .and_then(|i| i.caption.as_deref())
            }),
        selected_button_id: b
            .buttons_response_message
            .as_option()
            .and_then(|r| r.selected_button_id.as_deref()),
        selected_row_id: b
            .list_response_message
            .as_option()
            .and_then(|r| r.single_select_reply.as_option())
            .and_then(|s| s.selected_row_id.as_deref()),
        selected_id: b
            .template_button_reply_message
            .as_option()
            .and_then(|r| r.selected_id.as_deref()),
        params_json: params_json.as_deref(),
        name: b
            .poll_creation_message
            .as_option()
            .and_then(|p| p.name.as_deref())
            .or_else(|| {
                b.poll_creation_message_v2
                    .as_option()
                    .and_then(|p| p.name.as_deref())
            })
            .or_else(|| {
                b.poll_creation_message_v3
                    .as_option()
                    .and_then(|p| p.name.as_deref())
            }),
    };
    // `resolve_message_type` dipanggil supaya jalur `messageContextInfo` tetap
    // satu sumber kebenaran (serialize.rs:111-120).
    let resolved = resolve_message_type(Some(wire), b.interactive_response_message.as_option().is_some());
    let kind = resolved
        .as_deref()
        .map(MessageKind::from_wire_type)
        .unwrap_or(kind);
    let body = get_message_body(kind, &params);
    if body.is_empty() { None } else { Some(body) }
}

// =============================================================================
// 7. Pipeline
// =============================================================================

/// `config.messages?.energiExceeded || "⚡ Energi habis!"` (`handler.js:1879`).
/// Tidak datang dari store, jadi constant — `config.messages` tidak ada di
/// `oktz_core::Config`.
/// `truthy` JS — dipakai untuk baca setting boolean dari store.
fn truthy_js(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// `MessageKey` untuk pesan inbound — port `MessageContext::message_key`
/// (`whatsapp-rust-0.7.0/src/bot.rs:172-182`).
///
/// `MessageContext` adalah struct yang butuh `Arc<Client>`, sedangkan pipeline
/// hanya pegang `Arc<dyn MessageSender>` yang mungkin tidak punya Client sama
/// sekali (test, mode offline). Rumus key-nya pendek dan tidak bergantung
/// Client, jadi disalin di sini; `participant` hanya diisi untuk grup/status
/// karena di sana saja WA butuh untuk identifies pengirim asli.
fn message_key_of(m: &InboundMessage) -> wa::MessageKey {
    use whatsapp_rust::wacore_binary::JidExt;
    let needs_participant =
        m.info.source.is_group || m.info.source.chat.is_status_broadcast();
    wa::MessageKey {
        remote_jid: Some(m.info.source.chat.to_string()),
        from_me: Some(m.info.source.is_from_me),
        id: Some(m.info.id.clone()),
        participant: needs_participant.then(|| m.info.source.sender.to_string()),
    }
}

/// Sumber plugin untuk satu command. Default-nya registry global
/// [`oktz_plugin_api::find`].
///
/// Ada sebagai seam, bukan sebagai kebutuhan produksi: `oktz-wa` tidak
/// depend ke `oktz-plugins`, jadi registry plugin KOSONG di test binary crate
/// ini — `#[plugin]` menulis ke `linkme::distributed_slice`, dan `linkme` bukan
/// dependency langsung `oktz-wa`. Tanpa seam, gate cooldown dan energi tidak
/// bisa diuji sama sekali karena keduanya butuh `meta.cooldown` / `meta.energi`
/// dari plugin yang benar-benar terdaftar.
pub type PluginLookup = Arc<dyn Fn(&str) -> Option<&'static PluginEntry> + Send + Sync>;

/// Sumber roster grup. Pipeline yang tidak punya `Client` (test, mode offline)
/// memakai [`NoRoster`] dan dapat roster kosong — sama dengan "tidak diketahui",
/// bukan "grup ini kosong".
///
/// Implementasi sungguhan adalah [`ClientSender`], yang memanggil
/// `Client::groups().get_metadata()` (`whatsapp-rust-0.7.0/src/features/groups.rs:689`).
pub trait RosterSource: Send + Sync {
    fn roster(&self, chat: &str) -> SendBoxRoster<'_>;
}

pub type SendBoxRoster<'a> =
    Pin<Box<dyn Future<Output = Result<RosterSnapshot, String>> + Send + 'a>>;

/// Roster grup dari metadata: dua bentuk dari data yang sama.
///
/// - `roster` → bentuk plugin (`Ctx::roster`): satu JID per peserta.
/// - `participants` → bentuk gate: satu peserta bisa punya tiga identitas
///   (`id`/`lid`/`phone_number`), persis `p.jid`/`p.lid`/`p.phoneNumber` yang
///   dibaca `_participantNums` (`serialize.js:101-116`). Tanpa bentuk ini,
///   admin di grup addressing LID tidak pernah dikenali: roster hanya punya
///   LID, sementara identitas bot/peserta bisa PN.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RosterSnapshot {
    /// Bentuk plugin (`Ctx::roster`).
    pub roster: GroupRoster,
    /// Bentuk gate — identitas tiga-bentuk per peserta.
    pub participants: Vec<oktz_router::group::Participant>,
    /// Metadata grup berhasil dibaca?
    ///
    /// `false` = fetch gagal → di JS `catch` → gate group-protection dilewati
    /// SENYAP (`group-protection.js:1330/1278/2104/2153`), bukan membalas
    /// "bot bukan admin".
    pub available: bool,
}

/// Default: tidak ada yang bisa tahu roster. Selalu kosong, bukan error —
/// pesan dari grup tetap diproses, plugin `listadmin` yang perlu jailed.
pub struct NoRoster;

impl RosterSource for NoRoster {
    fn roster(&self, _chat: &str) -> SendBoxRoster<'_> {
        Box::pin(async { Ok(RosterSnapshot::default()) })
    }
}

impl RosterSource for ClientSender {
    /// `Groups::get_metadata` mengembalikan `GroupMetadata.participants:
    /// Vec<GroupParticipant>` (`features/groups.rs:151,236`) dengan
    /// `participant_type: ParticipantType` = `Member | Admin | SuperAdmin`
    /// (`wacore-0.7.0/src/iq/groups.rs:184`) — persis tri-state yang dibaca
    /// JS lewat `p.admin`.
    fn roster(&self, chat: &str) -> SendBoxRoster<'_> {
        let client = self.client.clone();
        let chat = chat.to_string();
        Box::pin(async move {
            let jid: Jid = chat
                .parse()
                .map_err(|e| format!("JID grup tidak valid {chat}: {e}"))?;
            let meta = client
                .groups()
                .get_metadata(&jid)
                .await
                .map_err(|e| e.to_string())?;

            // Bot sendiri dihitung dari daftar peserta: JID bot tidak pernah
            // datang di `MessageInfo`, jadi satu-satunya sumber yang benar
            // adalah roster itu sendiri.
            let mut bot_jids: Vec<String> = Vec::new();
            if let Some(pn) = client.pn() {
                bot_jids.push(pn.to_string());
            }
            if let Some(lid) = client.lid() {
                bot_jids.push(lid.to_string());
            }
            Ok(roster_snapshot_from(&meta, &bot_jids))
        })
    }
}

/// `GroupMetadata` → roster + identitas multi-bentuk untuk gate.
///
/// `GroupParticipant` bawa tiga bentuk: `jid` (LID di grup addressing LID,
/// PN di grup addressing PN), `phone_number` (PN), dan `lid`
/// (`whatsapp-rust-0.7.0/src/features/groups.rs:236-261`). `get_metadata`
/// juga mengisi `phone_number` untuk peserta LID dari cache LID↔PN
/// (`features/groups.rs:689-701`), jadi bentuk PN tersedia tanpa lookup
/// tambahan.
///
/// `ParticipantType` = `Member | Admin | SuperAdmin`
/// (`wacore-0.7.0/src/iq/groups.rs:184`) dibaca lewat
/// `is_admin()`/`is_super_admin()` (`features/groups.rs:246,250`), persis dua
/// nilai yang diporn plugin JS lewat `p.admin`.
pub fn roster_snapshot_from(meta: &GroupMetadata, bot_jids: &[String]) -> RosterSnapshot {
    let participants: Vec<oktz_router::group::Participant> = meta
        .participants
        .iter()
        .map(|p| oktz_router::group::Participant {
            id: p.jid.to_string(),
            lid: p.lid.as_ref().map(|j| j.to_string()).unwrap_or_default(),
            phone_number: p
                .phone_number
                .as_ref()
                .map(|j| j.to_string())
                .unwrap_or_default(),
            admin: p.is_admin(),
        })
        .collect();
    RosterSnapshot {
        roster: GroupRoster {
            // `adminFlagsFor(...).isBotAdmin` (serialize.js:153) — pencocokan
            // ANGKA, bukan JID persis: di grup addressing LID `p.jid` = LID
            // sementara bot dikenal sebagai PN (dan sebaliknya).
            is_bot_admin: oktz_router::group::bot_is_admin(&participants, bot_jids),
            participants: meta
                .participants
                .iter()
                .map(|p| Participant {
                    jid: p.jid.to_string(),
                    is_admin: p.is_admin(),
                    is_owner: p.is_super_admin(),
                    data: Value::Null,
                })
                .collect(),
        },
        participants,
        available: true,
    }
}

/// [`roster_snapshot_from`] — hanya bentuk plugin (`Ctx::roster`).
pub fn roster_from(meta: &GroupMetadata, bot_jids: &[String]) -> GroupRoster {
    roster_snapshot_from(meta, bot_jids).roster
}

/// Roster grup hasil baca, per chat, supaya satu grup tidak di-query ulang
/// untuk tiap pesan.
#[derive(Default)]
struct RosterCache {
    map: HashMap<String, (Instant, RosterSnapshot)>,
    ttl: std::time::Duration,
}

impl RosterCache {
    /// TTL default: roster berubahRare dan `get_metadata` selalu unduh penuh
    /// tanpa phash (`features/groups.rs:690`), jadi cache menyeimbangkan antara
    /// biaya jaringan dan staleness.
    const DEFAULT_TTL: std::time::Duration = std::time::Duration::from_secs(300);

    fn get(&self, chat: &str) -> Option<RosterSnapshot> {
        let (at, r) = self.map.get(chat)?;
        (at.elapsed() < self.ttl).then(|| r.clone())
    }

    fn put(&mut self, chat: &str, roster: RosterSnapshot) {
        self.map.insert(chat.to_string(), (Instant::now(), roster));
    }
}

/// Apa yang boleh dibaca handler plugin untuk mengisi `Ctx`.
///
/// Dipisah dari pipeline supaya handler yang sudah dibungkus bisa memegangnya
/// sendiri: handler bertipe `&'static`, jadi data per-pesan TIDAK boleh
/// ditangkap di dalam handler — hanya sumber yang Deterministik dari `Ctx`
/// (chat/sender) boleh.
#[derive(Clone)]
struct PluginCtxSource {
    store: Arc<StoreContext>,
    rosters: Arc<Mutex<RosterCache>>,
    sessions: SessionStore,
    http: Arc<dyn HttpGet>,
    /// `config.js` untuk `ctx.config()`. Default-nya `Runtime::default()`
    /// yang kosong; produksi memasang lewat `Pipeline::with_config`.
    config: Arc<oktz_core::config::Runtime>,
    /// `sock` JS. Default [`SockMati`].
    sock: Arc<dyn Sock>,
    /// `fs` untuk plugin. Default [`BerkasPalsu`]; produksi memasang
    /// [`BerkasNyata`] berakar ke direktori kerja.
    berkas: Arc<dyn Berkas>,
}

impl PluginCtxSource {
    /// Isi field `Ctx` yang tidak bisa dihitung router: roster grup + state
    /// store. Idempoten dan bebas I/O — semua sudah ada di memori.
    ///
    /// `settings` ikut di sini karena `db.setting(key)` (`database.js:753`)
    /// dibaca plugin dari snapshot yang SAMA dengan yang dibaca gate
    /// permission. Kalau plugin membaca settings lewat jalur lain, ia bisa
    /// melihat override yang tidak dipakai gate untuk pesan yang sama.
    fn fill(&self, c: &mut Ctx) {
        let mut roster = self
            .rosters
            .lock()
            .expect("roster cache")
            .get(&c.chat)
            .map(|snapshot| snapshot.roster)
            .unwrap_or_default();
        // `db.getUser(p.jid)` per peserta — di JS tiap plugin yang mengiterasi
        // roster memanggilnya sendiri, jadi ini satu baca per peserta per pesan.
        for p in &mut roster.participants {
            p.data = self.store.user_entry(&p.jid);
        }
        c.roster = roster;
        c.user_data = self.store.user_entry(&c.sender);
        c.group_data = self.store.group_entry(&c.chat);
        c.settings = self.store.settings_entry();
        c.sessions = self.sessions.clone();
        c.http = self.http.clone();
        c.config = self.config.clone();
        c.sock = self.sock.clone();
        c.berkas = self.berkas.clone();
    }
}

/// Handler asli + pembungkus yang mengisi `Ctx`. Satu per plugin.
///
/// Dibuat dengan `Box::leak` SEKALI per plugin, bukan per pesan: kalau
/// dibungkus per pesan, tiap pesan jadi satu alokasi yang tidak pernah
/// dibebaskan.
#[derive(Default)]
struct WrappedHandlers {
    map: HashMap<usize, &'static PluginEntry>,
}

impl WrappedHandlers {
    fn get_or_insert(
        &mut self,
        real: &'static PluginEntry,
        src: &PluginCtxSource,
    ) -> &'static PluginEntry {
        let key = std::ptr::from_ref(real) as usize;
        if let Some(e) = self.map.get(&key) {
            return e;
        }
        let handler = real.handler.clone();
        let src = src.clone();
        let entry: &'static PluginEntry = Box::leak(Box::new(PluginEntry {
            meta: real.meta.clone(),
            handler: Arc::new(move |mut c: Ctx| {
                src.fill(&mut c);
                handler(c)
            }),
        }));
        self.map.insert(key, entry);
        entry
    }
}

/// Identitas pemanggil. Sumber roster TIDAK ada di crate ini — `oktz_core::Config`
/// hanya punya `owner_numbers`, dan premium/admin tidak punya accessor sama sekali.
/// Semua default kosong (bukan siapa pun owner/premium/admin) sampai caller
/// mengisinya; mencocokkan menerima JID penuh ATAU nomor saja.
#[derive(Debug, Clone, Default)]
pub struct Identity {
    pub owners: HashSet<String>,
    pub premium: HashSet<String>,
    pub partners: HashSet<String>,
    pub admins: HashSet<String>,
}

/// Cocok dengan JID penuh, bagian sebelum `@`, atau nomor tanpa sufiks
/// perangkat (`6281:9@s.whatsapp.net` → `6281`).
fn listed(set: &HashSet<String>, jid: &str) -> bool {
    if set.contains(jid) {
        return true;
    }
    let user = jid.split('@').next().unwrap_or(jid);
    set.contains(user) || set.contains(user.split(':').next().unwrap_or(user))
}

impl Identity {
    pub fn is_owner(&self, jid: &str) -> bool {
        listed(&self.owners, jid)
    }
    pub fn is_premium(&self, jid: &str) -> bool {
        listed(&self.premium, jid)
    }
    pub fn is_partner(&self, jid: &str) -> bool {
        listed(&self.partners, jid)
    }
    pub fn is_admin(&self, jid: &str) -> bool {
        listed(&self.admins, jid)
    }
}

/// Hasil satu pesan masuk — dipakai test untuk memastikan tahap mana yang menahan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Tidak ada body teks (stiker, reaction, kontak, …).
    NoBody,
    /// Bukan command.
    NotACommand,
    /// Duplikat dalam 100 ms.
    Deduped,
    /// Kena rate limiter spam.
    SpamBlocked,
    /// Rantai group-protection menghentikan pesan (`handler.js:770-871`).
    /// Pesan sudah dihapus kalau gate-nya `DeleteAndStop`, jadi tidak ada
    /// balasan yang dikirim — bedanya dari [`Outcome::Denied`] yang membalas.
    GroupStopped,
    /// Cooldown masih aktif: pipeline bereaksi `⏱️` dan membatalkan dispatch
    /// (`handler.js:1846`). Tidak ada pesan yang dikirim.
    CooldownBlocked,
    /// Saldo energi kurang: user diberi tahu dan dispatch dibatalkan
    /// (`handler.js:1879-1880`). Energi TIDAK dipotong.
    EnergiDenied,
    /// Mode/permission ditolak, dan memang ada pesan yang dikirim.
    Denied,
    /// User banned: command dibalas pesan banned, pesan non-command diabaikan
    /// (`handler.js:901-913`).
    Banned,
    /// Grup sedang banned: command dari non-owner diabaikan tanpa balasan
    /// (`handler.js:916-921`).
    GroupBanned,
    /// Plugin jalan dan balasan terkirim.
    Sent,
/// Plugin membalas media yang `Reply` tidak punya sumbernya.
    ///
    /// TIDAK lagi dipakai: kasus itu sekarang jadi
    /// [`PipelineError::MediaSourceMissing`], bukan `Outcome` —
    /// `process` mengembalikan `Err` supaya tidak ada yang mengira pesan terkirim.
    /// Varian dipertahankan supaya `Outcome` tidak berubah bentuk bagi
    /// pemanggil yang sudah mematch-nya.
    MediaMissing,
    /// Mutasi plugin gagal ditulis ke store. Balasan plugin TIDAK dikirim,
    /// karena hampir semua teksnya mengklaim data sudah tersimpan.
    ///
    /// `Outcome` dan bukan `Err`: kegagalan ini TERJADI pada jalur normal
    /// (disk penuh, file terkunci proses lain, file jadi rusak) dan tidak
    /// boleh menghentikan loop pesan — pesan berikutnya tetap diproses dan
    /// akan membaca store apa adanya. Yang akibatnya dicatat di sini dan
    /// di-`tracing::error!`, bukan hilang.
    StoreWriteFailed,
}

/// `AutoWorkGuard` yang diikat ke satu pesan.
///
/// JS membuat guard baru per pesan (`handler.js:416-422`) lalu dibuang setelah
/// pesan selesai. Di sini satu pesan diproses oleh satu task
/// ([`Pipeline::run`]), jadi slot tunggal cukup: guard direset begitu
/// `message_id` berganti. Scoped per-pesan dengan begini persis sama dengan
/// scope JS, tapi tanpa alokasi per pesan.
#[derive(Default)]
struct AutoWorkSlot {
    message_id: String,
    guard: AutoWorkGuard,
}

pub struct Pipeline {
    ctx: Arc<StoreContext>,
    sender: Arc<dyn MessageSender>,
    dedup: Mutex<Dedup>,
    spam: Mutex<RateLimiter>,
    auto_work_limiter: Mutex<RateLimiter>,
    /// Rantai group-protection. `OnceLock` karena rantai itu konstan —
    /// daftar gate-nya sudah ditentukan saat compile.
    group_guard: OnceLock<group::GroupGuard>,
    /// Cooldown per `(user, command)`, saldo energi per user, dan timestamp
    /// slowmode. Satu proses = satu instance; lihat catatan disk di
    /// [`pipeline_gate`].
    gates: Arc<GateState>,
    /// `createAutoWorkGuard` (`handler.js:416-422`) — satu keputusan per pesan.
    /// Auto-AI, transkripsi, dan smart trigger semua memanggil `allowAutoWork`
    /// pada pesan yang sama; tanpa ini satu pesan bisa membakar kuota 6×.
    auto_work: Mutex<AutoWorkSlot>,
    /// Setelan energi global (`handler.js:1851-1854`). Override biaya
    /// per-plugin dibaca per-message dari store, jadi tidak disimpan di sini.
    energi_settings: EnergiSettings,
    parse_opts: ParseOptions,
    bot_id: String,
    identity: Identity,
    anti_spam_enabled: bool,
    auto_work_settings: AutoWorkSettings,
    /// Sumber roster grup. Default [`NoRoster`] supaya konstruktor tetap
    /// kompatibel dengan pemanggil lama (`bot.rs`).
    roster: Arc<dyn RosterSource>,
    /// Registry plugin global, kecuali[`Pipeline::with_plugin_lookup`] menggantinya.
    plugin_lookup: PluginLookup,
    /// Yang boleh dibaca handler plugin: store + cache roster.
    plugin_src: PluginCtxSource,
    /// Handler yang sudah dibungkus — satu per plugin.
    wrapped: Mutex<WrappedHandlers>,
    /// Wajib disimpan: `Subscription` unsubscribe saat di-drop
    /// (wacore-0.7.0/src/types/events.rs:538-544).
    subscription: Mutex<Option<Subscription>>,
}

impl Pipeline {
    pub fn new(
        ctx: StoreContext,
        sender: Arc<dyn MessageSender>,
        bot_id: impl Into<String>,
        parse_opts: ParseOptions,
    ) -> Self {
        let ctx = Arc::new(ctx);
        Self {
            plugin_src: PluginCtxSource {
                store: Arc::clone(&ctx),
                rosters: Arc::new(Mutex::new(RosterCache {
                    map: HashMap::new(),
                    ttl: RosterCache::DEFAULT_TTL,
                })),
                sessions: new_session_store(),
                http: Arc::new(NoHttp),
                config: Arc::new(oktz_core::config::Runtime::default()),
                sock: Arc::new(SockMati),
                berkas: Arc::new(oktz_plugin_api::berkas::BerkasPalsu::baru()),
            },
            ctx,
            sender,
            dedup: Mutex::new(Dedup::new()),
            spam: Mutex::new(RateLimiter::command_global()),
            auto_work_limiter: Mutex::new(RateLimiter::auto_work()),
            group_guard: OnceLock::new(),
            gates: Arc::new(GateState::new()),
            auto_work: Mutex::new(AutoWorkSlot::default()),
            energi_settings: EnergiSettings::default(),
            parse_opts,
            bot_id: bot_id.into(),
            identity: Identity::default(),
            anti_spam_enabled: true,
            auto_work_settings: AutoWorkSettings::default(),
            roster: Arc::new(NoRoster),
            plugin_lookup: Arc::new(oktz_plugin_api::find),
            wrapped: Mutex::new(WrappedHandlers::default()),
            subscription: Mutex::new(None),
        }
    }

    pub fn with_identity(mut self, identity: Identity) -> Self {
        self.identity = identity;
        self
    }

    /// Pasang sumber roster. Tanpa ini, `Ctx::roster` selalu kosong.
    pub fn with_roster_source(mut self, roster: Arc<dyn RosterSource>) -> Self {
        self.roster = roster;
        self
    }

    /// Klien HTTP untuk plugin class B. Default-nya [`NoHttp`] supaya unit
    /// test tidak pernah menembak jaringan.
    pub fn with_http(mut self, http: Arc<dyn HttpGet>) -> Self {
        self.plugin_src.http = http;
        self
    }

    /// Pasang `config.js` runtime untuk `ctx.config()`.
    pub fn with_config(mut self, config: oktz_core::config::Runtime) -> Self {
        self.plugin_src.config = Arc::new(config);
        self
    }

    /// Pasang `sock` JS untuk `ctx.sock()`. Default-nya [`SockMati`].
    pub fn with_sock(mut self, sock: Arc<dyn Sock>) -> Self {
        self.plugin_src.sock = sock;
        self
    }

    /// Pasang `fs` untuk `ctx.berkas`. Default-nya [`BerkasPalsu`].
    pub fn with_berkas(mut self, berkas: Arc<dyn Berkas>) -> Self {
        self.plugin_src.berkas = berkas;
        self
    }

    pub fn with_anti_spam(mut self, enabled: bool) -> Self {
        self.anti_spam_enabled = enabled;
        self
    }

    /// Setelan gate energi (`config.energi`, `handler.js:1859-1862`).
    pub fn with_energi_settings(mut self, settings: EnergiSettings) -> Self {
        self.energi_settings = settings;
        self
    }

    /// Setelan kuota auto-work (`config.autoWork`, `handler.js:382-408`).
    pub fn with_auto_work_settings(mut self, settings: AutoWorkSettings) -> Self {
        self.auto_work_settings = settings;
        self
    }

    /// Ganti sumber plugin (test). Produksi tidak pernah memanggil ini.
    pub fn with_plugin_lookup(mut self, lookup: PluginLookup) -> Self {
        self.plugin_lookup = lookup;
        self
    }

    /// State gate — tempat cooldown dan energi hidup. Dibaca test untuk
    /// membuktikan charge terjadi tepat sekali.
    pub fn gates(&self) -> &Arc<GateState> {
        &self.gates
    }

    /// Bagikan state gate dengan pipeline lain di proses yang sama (test).
    pub fn with_gate_state(self, gates: Arc<GateState>) -> Self {
        Self {
            gates,
            ..self
        }
    }

    pub fn ctx(&self) -> &StoreContext {
        &self.ctx
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Kuota auto-work untuk fitur yang memang memakainya. Dipanggil per-fitur,
    /// bukan per-pesan — lihat catatan modul.
    ///
    /// `message_id` mengikat keputusan ke satu pesan: panggilan kedua pada
    /// pesan yang sama membaca hasil yang tersimpan, jadi kuota tidak dibakar
    /// dua kali (sesuai `createAutoWorkGuard`, `handler.js:416-422`).
    pub fn allow_auto_work(
        &self,
        message_id: &str,
        caller: &Caller,
        key: &str,
        user_energi: Option<i64>,
        now_ms: i64,
    ) -> AutoWorkDecision {
        let mut slot = self.auto_work.lock().unwrap_or_else(|e| e.into_inner());
        if slot.message_id != message_id {
            slot.message_id.clear();
            slot.message_id.push_str(message_id);
            slot.guard = AutoWorkGuard::new();
        }
        let settings = self.auto_work_settings;
        slot.guard.allow(|| {
            let mut limiter = self.auto_work_limiter.lock().expect("auto_work limiter");
            consume_auto_work(
                &mut limiter,
                caller,
                key,
                &settings,
                user_energi,
                now_ms,
            )
        })
    }

    /// Caller dari `MessageInfo.source` (`MessageSource`, wacore-0.7.0
    /// `src/types/message.rs:104`).
    pub fn caller_of(&self, m: &InboundMessage) -> Caller {
        let s = &m.info.source;
        let sender = s.sender.to_string();
        // `config.isOwner/isPremium/isPartner` (`config.js:364-505`) membaca
        // DUA sumber: config DAN store. Store Rust hanya berisi config
        // (owner dari `config.json`), jadi tanpa baca store, user premium
        // yang terdaftar di `premium.json`/`settings.premiumUsers` tetap
        // dianggap non-premium dan command premium terus dibalas
        // "Premium Only!".
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or_default();
        // Media pesan + jalur unduhnya. Pengirim stub (test) tidak punya
        // `Client`, jadi `media_fetch` tetap `None` dan `Ctx` memakai
        // `NoMediaFetch` — `Ctx::media` tetap terisi supaya plugin bisa
        // memeriksa `m.isImage` tanpa sesi WA.
        let (media, media_fetch) = crate::media_fetch::ctx_media(self.sender.client(), m);
        Caller {
            sender: sender.clone(),
            chat: s.chat.to_string(),
            command: String::new(),
            is_group: s.is_group,
            is_owner: self.identity.is_owner(&sender) || self.ctx.is_owner_user(&sender),
            is_partner: self.identity.is_partner(&sender)
                || self.ctx.is_partner_user(&sender, now_ms),
            is_premium: self.identity.is_premium(&sender)
                || self.ctx.is_premium_user(&sender, now_ms),
            is_admin: s.is_group && self.identity.is_admin(&sender),
            is_bot_admin: false,
            push_name: m.info.push_name.clone(),
            mentioned: mentioned_jids(m),
            quoted: quoted_of(m),
            media,
            media_fetch,
        }
    }

    /// Bentuk buggy `sock.user.id.split(":")[0] + "@s.whatsapp.net"` yang dipakai
    /// `handleAntilink*` (`group-protection.js:1327/1898/1953`). Gate yang pakai
    /// bentuk yang benar (`botJidOf`) memang tidak di-port ke `group.rs`.
    fn bot_id_at(&self) -> String {
        let id = self.bot_id.split(':').next().unwrap_or(&self.bot_id);
        format!("{id}@s.whatsapp.net")
    }

    /// Kandidat identitas bot: PN + LID — `botCand` di JS
    /// (`group-protection.js:1333`, `serialize.js:847`).
    ///
    /// Tanpa LID, bot tidak pernah dikenali sebagai admin di grup addressing
    /// LID: roster peserta LID cuma punya angka LID, sedangkan `bot_number`
    /// adalah PN.
    fn bot_jids(&self) -> Vec<String> {
        let mut jids: Vec<String> = Vec::new();
        if let Some(client) = self.sender.client() {
            if let Some(pn) = client.pn() {
                jids.push(pn.to_string());
            }
            if let Some(lid) = client.lid() {
                jids.push(lid.to_string());
            }
        }
        if jids.is_empty() {
            jids.push(self.bot_id_at());
        }
        jids
    }

    /// `m.sender` versi JS untuk pesan grup.
    ///
    /// JS: `serialize.js:694` mulai dari `msg.key.participant`; kalau
    /// LID-ish, pasangan PN dicari (`participantAlt`/`participantPn`/sock/
    /// cache, :700-747) → `m.sender` jadi PN. Rust: `MessageSource.sender_alt`
    /// = `participant_pn` (addressing LID) atau `participant_lid` (addressing
    /// PN) — `wacore-0.7.0/src/messages.rs:1110-1114`.
    ///
    /// Kalau wire tidak menyertakan pasangan itu, pengirim tetap LID — sama
    /// seperti JS yang cache/sock-nya tidak bisa resolve.
    fn resolved_sender(source: &whatsapp_rust::types::message::MessageSource) -> String {
        let sender = source.sender.to_string();
        let Some(alt) = source.sender_alt.as_ref() else {
            return sender;
        };
        // PN preferred: LID adalah identitas grup, PN untuk teks/mention.
        if source.sender.is_lid() && !alt.is_lid() {
            return alt.to_string();
        }
        sender
    }

    /// Caller untuk rantai group-protection, dari flag yang sudah dibaca
    /// [`Pipeline::caller_of`] plus sisa field yang cuma group.rs yang baca.
    ///
    /// `is_bot_admin` diisi dari roster, bukan `false` seperti di
    /// [`Pipeline::caller_of`]: tanpa itu, gate yang butuh menghapus pesan akan
    /// selalu melihat "bot bukan admin" dan membalas `NOT_ADMIN`.
    fn group_caller_of(
        &self,
        m: &InboundMessage,
        caller: &Caller,
        body: &str,
        snapshot: &RosterSnapshot,
    ) -> GroupCaller {
        // `m.sender` versi JS — participant wire di-resolve ke PN kalau
        // pasangannya ada di wire.
        let sender = Self::resolved_sender(&m.info.source);
        // `m.key.participant` — JS menulis ulang key ini jadi `m.sender`
        // (`serialize.js:756-758`), jadi di gate keduanya sama: pengirim yang
        // sudah di-resolve. BUG lama: Rust mengambil participant dari
        // `context_info` (pesan yang DIKUTIP), bukan pesan sekarang.
        let participant = Some(sender.clone());
        let parsed = parse_command(body, &self.parse_opts);
        GroupCaller {
            sender: sender.clone(),
            participant,
            chat: caller.chat.clone(),
            is_group: caller.is_group,
            // `is_newsletter` — grup tidak punya DialogType newsletter, dan
            // `group_guard` sendiri sudah melewati apa pun yang bukan grup.
            is_newsletter: false,
            is_owner: caller.is_owner,
            // `m.isAdmin` (serialize.js:846) = admin GRUP dari metadata roster,
            // bukan daftar admin global. Daftar admin global tidak ada di JS
            // (`config.js` cuma punya owner/premium/partner), jadi angka
            // `identity.admins` yang dipakai Rust selalu kosong → admin grup
            // tidak pernah dikenali: pesan mereka dihapus dan toggle
            // ditolak.
            is_admin: oktz_router::group::sender_is_admin(
                &snapshot.participants,
                &[sender.as_str()],
            ),
            is_partner: caller.is_partner,
            is_bot_admin: oktz_router::group::bot_is_admin(
                &snapshot.participants,
                &self.bot_jids(),
            ),
            from_me: m.info.source.is_from_me,
            is_command: parsed.is_command,
            is_document: wire_type_of(&m.message) == MessageKind::Document.wire_type(),
            message_type: wire_type_of(&m.message).to_string(),
            // `detectSwGcType(rawMsg)` — path tipe SW-group untuk gate
            // antiswgc (`group-protection.js:467-520`).
            sw_group_type: sw_group_type_of(&m.message),
            body: body.to_string(),
            mentioned_jid: m
                .message
                .get_base_message()
                .extended_text_message
                .as_option()
                .and_then(|e| e.context_info.as_option())
                .map(|c| c.mentioned_jid.clone())
                .unwrap_or_default(),
            participant_count: snapshot.roster.participants.len(),
        }
    }

    /// Plugin yang AKAN dipanggil router untuk pesan ini, atau `None` kalau
    /// router akan menolak/menolak diam sebelum dispatch.
    ///
    /// Mengulang langkah 1-4 [`Router::route`] (`oktz-router/src/lib.rs:94-139`)
    /// dengan fungsi yang PERSAMA, jadi hasilnya tidak mungkin beda dari
    /// keputusan router. Semua fungsi itu murni — tanpa I/O, tanpa efek
    /// samping — jadi mengulangnya aman; yang tidak boleh diulang adalah
    /// `handler(c)` di langkah 5, dan itulah yang tidak diulang.
    ///
    /// Mengembalikan `Some` berarti gate 4/5 boleh membakar kuota: kalau
    /// router ternyata menolak nanti (karena state berubah di antara dua
    /// pemanggilan), konsekuensinya sama seperti di JS, karena JS juga punya
    /// jendela di antara permission dan dispatch.
    fn dispatch_target(
        &self,
        caller: &mut Caller,
        body: &str,
        lookup: impl Fn(&str) -> Option<&'static PluginEntry>,
        now_ms: i64,
    ) -> Option<&'static PluginEntry> {
        use oktz_router::{mode, permission};
        let parsed = parse_command(body, &self.parse_opts);
        if !parsed.is_command {
            return None;
        }
        // `caller.command` diisi router di `route` (lib.rs:92); diisi di sini
        // juga supaya `start_cooldown` setelah dispatch memakai nama yang
        // sama dengan yang dilihat gate.
        caller.command = parsed.command.clone();

        // 1. Mode global (handler.js:879).
        let overrides = if caller.is_group {
            self.ctx.group_overrides(&caller.chat)
        } else {
            None
        };
        mode::check_mode(caller, &self.ctx.mode_settings(), overrides, now_ms).ok()?;

        let entry = lookup(&parsed.command)?;

        // 2. Permission (handler.js:1810).
        permission::check(
            caller,
            &self.ctx.user_state(&caller.sender),
            &permission::PermissionInput {
                meta: &entry.meta,
                now_ms,
                capprem: self.ctx.capprem(&entry.meta.name),
                group: if caller.is_group {
                    self.ctx.group_state(&caller.chat)
                } else {
                    None
                },
            },
        )
        .ok()?;

        // 3. botMode per-grup (handler.js:1709).
        if caller.is_group && !caller.is_owner && !caller.is_admin {
            let bot_mode = self.ctx.group_bot_mode(&caller.chat);
            mode::group_feature_mode(caller, &entry.meta, None, &bot_mode, false).ok()?;
        }
        Some(entry)
    }

    /// Render balasan sesuai `db.setting("replyVariant")`.
    ///
    /// Varian yang sudah ada selalu menang kalau reply-nya TEKS; kalau tidak
    /// ada varian yang cocok, jatuh ke [`render_reply`] persis seperti
    /// sebelumnya.
    ///
    /// Varian reply HANYA berlaku untuk teks: di JS `m.reply` yang punya
    ///cabang varian, sementara kartu/kirim media lewat `sock.sendMessage`/
    /// `relayMessage` langsung (`serialize.js:958` vs `socket.js:353`). Kalau
    /// kartu ikut dirender sebagai varian, tombolnya hilang.
    ///
    /// Akar aset diambil dari direktori kerja, sama seperti di JS.
    fn render_with_reply_variant(
        &self,
        reply: &Reply,
        target: &ReplyTarget,
    ) -> Result<Option<Rendered>, PipelineError> {
        let settings = self.ctx.settings_entry();
        let variant = reply_variant::variant_from(Some(&settings));
        if !reply_variant::needs_custom_render(variant) || !reply.is_text_like() {
            return render_reply(reply, target);
        }
        let Some(text) = reply.text_body() else {
            return render_reply(reply, target);
        };
        let root = std::env::current_dir().unwrap_or_default();
        let cfg = &self.plugin_src.config;
        let bot_name = cfg.bot.name.as_str();
        let bot_developer = cfg.bot.developer.as_str();
        let bot_version = cfg.bot.version.as_str();
        let website = cfg.info.website.as_str();
        let newsletter_jid = cfg.saluran.id.as_str();
        let newsletter_name = cfg.saluran.name.as_str();
        let owner_number = cfg.owner.number.first().map(String::as_str).unwrap_or("");
        let input = reply_variant::VariantInput {
            text,
            target,
            root: &root,
            bot_name,
            bot_developer,
            bot_version,
            website,
            newsletter_jid,
            newsletter_name,
            owner_number,
            srt_enabled: flag(Some(&settings), "srtEnabled"),
        };
        match reply_variant::render(variant, &input)? {
            Some(rendered) => Ok(Some(rendered)),
            // Varian 1 (atau yang tidak dikenal) = teks biasa.
            None => render_reply(reply, target),
        }
    }

    /// Kirim hasil render: media lewat jalur upload, kartu lewat jalur upload
    /// header-nya, teks lewat `send`.
    async fn deliver(&self, chat: &str, rendered: Rendered) -> Result<(), PipelineError> {
        // `Rendered` untuk media sengaja punya `message` kosong, jadi TIDAK
        // mungkin terkirim apa adanya — harus lewat `send_media`.
        // Kartu yang butuh upload header juga begitu: `message` kosong sampai
        // `send_card` merakitnya.
        let opts = send_options(&rendered);
        let sent = match (rendered.media, rendered.card, rendered.preview) {
            (Some(pending), _, _) => self.sender.send_media(chat, pending, opts).await,
            (None, Some(card), _) => self.sender.send_card(chat, card, opts).await,
            (None, None, Some(preview)) => self.sender.send_preview(chat, preview, opts).await,
            (None, None, None) => self.sender.send(chat, rendered.message, opts).await,
        };
        sent.map_err(|e| PipelineError::Other(format!("gagal kirim: {e}")))
    }

    /// `db.setting("capenergi") || {}` (`handler.js:1856`) — override biaya
    /// per-plugin. Dibaca per-pesan supaya perubahan dari panel langsung berlaku.
    fn cap_energi_overrides(&self) -> HashMap<String, i64> {
        self.ctx
            .energy_overrides()
    }

    /// Eksekusi keputusan rantai group-protection di live path.
    ///
    /// Urutan delete → kick → reply mengikuti JS: `await sock.sendMessage(m.chat,
    /// {delete: m.key}); await m.reply(...)` (`handler.js:811/823/848`).
    ///
    /// Kegagalan delete MEMBATALKAN aksi: di JS error delete naik ke `catch`
    /// gate → `return false` → tanpa kick, tanpa balasan
    /// (`group-protection.js:1346/1911/1966` + `catch` :1363/1936/1989).
    /// Kegagalan kick berbeda per gate: `handleAntilinkGc/All` dan
    /// `executeProtectionAction` menangkapnya lalu mengirim pesan mode-remove,
    /// sedangkan `handleAntilink` tidak → tanpa balasan.
    ///
    /// `mention` = pengirim yang dipakai untuk mention balasan (`[m.sender]`
    /// di JS, `serialize.js:958`).
    async fn execute_group_decision(
        &self,
        decision: group::Decision,
        m: &InboundMessage,
        caller: &Caller,
        mention: &str,
    ) -> Result<Outcome, PipelineError> {
        let deletes = matches!(
            decision,
            group::Decision::DeleteAndStop
                | group::Decision::DeleteAndReply(_)
                | group::Decision::KickAndReply(_)
                | group::Decision::KickAndReplyFallback { .. }
        );
        if deletes {
            // Participant revoke = participant ASLI dari wire (LID kalau grup
            // addressing LID). `whatsapp-rust` minta participant sama persis
            // dengan key pesan asli — JANGAN konversi LID→PN
            // (`send/actions.rs:39-55`), seperti WhatsApp Web.
            //
            // `ponytail:` JS menulis `m.key.participant` yang sudah di-resolve
            // ke PN (`serialize.js:757`), jadi revoke-nya bisa tidak cocok di
            // server untuk grup LID. Rust memakai participant wire asli supaya
            // penghapusan benar-benar bekerja.
            let participant = m.info.source.sender.to_string();
            if let Err(e) = self
                .sender
                .delete_message(&caller.chat, &m.info.id, &participant)
                .await
            {
                tracing::warn!(
                    error = %e,
                    chat = %caller.chat,
                    id = %m.info.id,
                    "gagal menghapus pesan terlarangan"
                );
                // JS: aksi dibatalkan — tanpa kick, tanpa balasan.
                return Ok(Outcome::GroupStopped);
            }
        }
        // JS: delete → kick → reply.
        let mut text = decision.reply_text().map(str::to_string);
        if matches!(
            decision,
            group::Decision::KickAndReply(_) | group::Decision::KickAndReplyFallback { .. }
        ) {
            // Target kick = participant wire asli (LID kalau addressing LID) —
            // roster grup memakai JID yang sama, jadi kick benar-benar
            // mengeluarkan peserta.
            let target = m.info.source.sender.to_string();
            if let Err(e) = self.plugin_src.sock.ubah_peserta(
                &caller.chat,
                std::slice::from_ref(&target),
                oktz_plugin_api::AksiPeserta::Keluar,
            ) {
                tracing::warn!(
                    error = %e,
                    chat = %caller.chat,
                    peserta = %target,
                    "gagal kick peserta"
                );
                // JS: `handleAntilinkGc/All` + `executeProtectionAction`
                // menangkap error kick lalu mengirim pesan mode-remove;
                // `handleAntilink` tidak → tanpa balasan.
                match decision.kick_fallback() {
                    Some(fallback) => text = Some(fallback.to_string()),
                    None => return Ok(Outcome::GroupStopped),
                }
            }
        }
        match text {
            Some(text) => {
                // Di JS pesan gate ini dikirim `m.reply(...)` (group-protection
                // memakai `m.reply`/`sock.sendMessage(m.chat, {text})` via
                // helper) — bentuknya ikut `replyVariant` (serialize.js:958).
                let reply = Reply::text(text);
                let target = reply_target(m, mention);
                if let Some(rendered) = self.render_with_reply_variant(&reply, &target)? {
                    self.deliver(&caller.chat, rendered).await?;
                }
                Ok(Outcome::Sent)
            }
            None => Ok(Outcome::GroupStopped),
        }
    }

    /// Roster untuk chat ini, dari cache atau dari jaringan.
    ///
    /// Gagal membaca metadata TIDAK menggagalkan pesan — plugin tanpa roster
    /// tetap jalan, hanya `Ctx::roster` kosong. Align dengan JS:
    /// `m.groupMetadata` bisa `undefined` dan plugin tetap jalan
    /// (`plugins/group/tagall.js:28-32`).
    ///
    /// Tapi gate group-protection harus TAHU bedanya: di JS fetch metadata
    /// yang gagal membuat gate dilewati senyap (`catch` → `return false`),
    /// bukan membalas "bot bukan admin". Itu Flags `available`.
    async fn roster_of(&self, chat: &str, is_group: bool) -> RosterSnapshot {
        if !is_group {
            return RosterSnapshot::default();
        }
        if let Some(r) = self
            .plugin_src
            .rosters
            .lock()
            .expect("roster cache")
            .get(chat)
        {
            return r;
        }
        let snapshot = match self.roster.roster(chat).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, chat = %chat, "gagal ambil roster grup");
                RosterSnapshot::default()
            }
        };
        // Kosong tidak dicache: hasil gagal baca harus dicoba ulang, bukan
        // dikunci jadi "grup ini tidak punya anggota" selama TTL.
        if snapshot.available && !snapshot.roster.is_empty() {
            self.plugin_src
                .rosters
                .lock()
                .expect("roster cache")
                .put(chat, snapshot.clone());
        }
        snapshot
    }

    /// Satu pesan inbound → satu keputusan → satu pengiriman.
    pub async fn process(&self, m: &InboundMessage, now_ms: i64) -> Result<Outcome, PipelineError> {
        // File store bisa diubah dari luar (panel bot JS, proses lain) —
        // mis. owner menyalakan `antilink` di `groups.json`. Snapshot di-load
        // ulang dulu supaya setting itu berlaku untuk pesan ini.
        self.ctx.refresh();
        let Some(body) = extract_body(&m.message) else {
            return Ok(Outcome::NoBody);
        };

        let mut caller = self.caller_of(m);

        // Dedup: kunci persis handler.js:926.
        let dedup_key = format!(
            "{}_{}_{}_{}",
            self.bot_id,
            caller.chat,
            caller.sender,
            m.info.id
        );
        if self
            .dedup
            .lock()
            .expect("dedup")
            .debounce(&dedup_key, now_ms)
        {
            return Ok(Outcome::Deduped);
        }

        // Roster dimuat SEBELUM gate grup supaya `isBotAdmin` (yang menentukan
        // boleh/tidaknya bot menghapus pesan) benar, dan supaya plugin pertama
        // yang jalan melihat `Ctx` lengkap. Router tidak punya Client maupun
        // store.
        let snapshot = self.roster_of(&caller.chat, caller.is_group).await;

        // `m.isAdmin` (serialize.js:846) = admin GRUP dari metadata roster.
        // Rust sebelumnya memakai daftar admin global (`identity.admins`) yang
        // di produksi selalu KOSONG — `config.js` JS tidak punya daftar itu —
        // jadi admin grup tidak pernah dikenali: pesan mereka dihapus gate,
        // dan toggle proteksi ditolak. Sekarang dihitung dari roster.
        if caller.is_group {
            caller.is_admin = oktz_router::group::sender_is_admin(
                &snapshot.participants,
                &[caller.sender.as_str()],
            );
        }

        // --- Gate 1: rantai group-protection (`handler.js:770-871`) ---------
        // Dijalankan untuk SETIAP pesan grup, sebelum command apa pun. Ini
        // yang membuat `DeleteAndStop` masuk akal: pesan sudah hilang dari
        // grup sebelum plugin pernah melihatnya.
        if caller.is_group {
            let settings = pipeline_gate::group_settings_from(&self.ctx.group_entry(&caller.chat));
            let env = GateEnv {
                // `sock.user.id.split(":")[0] + "@s.whatsapp.net"` — bentuk
                // buggy yang dipakai `handleAntilink*`
                // (group-protection.js:1327/1898/1953); group.rs sudah tahu.
                bot_number: self.bot_id_at().to_string(),
                // `sock.user.lid` — kandidat kedua "bot admin"
                // (group-protection.js:1333). Tanpa ini bot tidak dikenali
                // sebagai admin di grup addressing LID.
                bot_lid: self
                    .sender
                    .client()
                    .and_then(|client| client.lid())
                    .map(|lid| lid.to_string()),
                self_mode: self.ctx.mode_settings().mode.eq_ignore_ascii_case("self"),
                // Metadata gagal dibaca → JS `catch` → gate dilewati senyap.
                metadata_available: snapshot.available,
                participants: snapshot.participants.clone(),
                now_ms,
            };
            let gcaller = self.group_caller_of(m, &caller, &body, &snapshot);
            let slow_key = format!("{}_{}", caller.chat, caller.sender);
            let input = group::GateInput {
                caller: &gcaller,
                settings: &settings,
                env: &env,
                last_message_ms: self.gates.last_message_ms(&slow_key),
            };
            let (decision, record) = self
                .group_guard
                .get_or_init(group::default_gate_chain)
                .run_with_record(&input);
            // `lastMessageTime.set()` hanya saat rantai lolos (`slowmode.js:142`).
            if record {
                self.gates.record_message(&slow_key, now_ms);
            }
            if !matches!(decision, group::Decision::Allow) {
                // Mention balasan = `m.sender` versi JS (serialize.js:958).
                let mention = gcaller.sender.clone();
                return self
                    .execute_group_decision(decision, m, &caller, &mention)
                    .await;
            }
        }

        // --- Gate 1.5: user banned (`handler.js:901-913`) -------------------
        // `m.isBanned` → kalau pesan ini command, balas pesan banned; kalau
        // bukan, diam. Setelah itu pipeline berhenti — plugin tidak jalan.
        // Sumber ban: `settings.bannedUsers` (`config.js:507-519`).
        if self.ctx.is_banned_user(&caller.sender) {
            if parse_command(&body, &self.parse_opts).is_command {
                let pesan = oktz_core::config::Messages::default().banned;
                let reply = Reply::text(pesan);
                let target = reply_target(m, &caller.sender);
                // `m.reply(...)` di JS — bentuknya ikut `replyVariant`.
                if let Some(rendered) = self.render_with_reply_variant(&reply, &target)? {
                    self.deliver(&caller.chat, rendered).await?;
                }
            }
            return Ok(Outcome::Banned);
        }

        // --- Gate 1.6: grup banned (`handler.js:916-921`) --------------------
        // Command dari user non-owner di grup yang di-banned diabaikan diam;
        // owner tetap bisa pakai bot.
        if caller.is_group
            && !caller.is_owner
            && parse_command(&body, &self.parse_opts).is_command
            && self
                .ctx
                .group_entry(&caller.chat)
                .get("isBanned")
                .map(truthy_js)
                .unwrap_or(false)
        {
            return Ok(Outcome::GroupBanned);
        }

        // --- Gate 2: rate limiter spam (`handler.js:1508`) ------------------
        // Hanya untuk yang bukan owner/premium — dicek di dalam `is_spamming`.
        let spam_key = format!("{}_{}", self.bot_id, caller.sender);
        if is_spamming(
            &mut self.spam.lock().expect("spam"),
            &caller,
            &spam_key,
            now_ms,
            self.anti_spam_enabled,
        ) {
            return Ok(Outcome::SpamBlocked);
        }

        let lookup = |cmd: &str| {
            let real = (self.plugin_lookup)(cmd)?;
            // Kunci hanya selama lookup cache; TIDAK dipegang lintas
            // pemanggilan handler, jadi dua pesan paralel tidak saling blokir.
            Some(
                self.wrapped
                    .lock()
                    .expect("wrapped handlers")
                    .get_or_insert(real, &self.plugin_src),
            )
        };

        // --- Entry point kedua: answer handler -----------------------------
        // `handleGameAnswer` (`src/handler.js:440-473`) dipanggil untuk jawaban
        // main yang BUKAN command, jadi router biasa tidak akan pernah
        // melihatnya. Di JS panggilan itu dijaga predicate `hasActiveSession`
        // / `hasSuitGame` / `hasTTTGame` / `hasUTGame`; predicate itu belum
        // ada di sini, jadi setiap answer handler yang memutuskan sendiri —
        // semuanya mengembalikan `None` kalau tidak ada sesi aktif.
        //
        // `ponytail:` kalau nanti sesi game punya store sendiri, pindahkan
        // penjaga ini ke `if session_ada(&caller.chat)` supaya jalur non-game
        // tidak pernah menyentuh rantai ini.
        if !parse_command(&body, &self.parse_opts).is_command {
            let router = Router::new(&*self.ctx, self.parse_opts.clone(), now_ms);
            if let Some(reply) = router.answer(&caller, &body) {
                let target = reply_target(m, &caller.sender);
                // Jawaban game di JS dikirim `m.reply` (`handler.js:440-473`),
                // jadi bentuknya ikut `replyVariant`.
                if let Some(rendered) = self.render_with_reply_variant(&reply, &target)? {
                    self.deliver(&caller.chat, rendered).await?;
                }
                return Ok(Outcome::Sent);
            }
        }

        // --- Gate 3: kuota auto-work + `AutoWorkGuard` ----------------------
        // Gate-nya dibakar per-FEATURE yang memakainya, bukan per-pesan: di JS
        // `allowAutoWork()` dipanggil auto-AI, CAI chat, auto-download,
        // auto-join, transkripsi, dan smart trigger saja
        // (`handler.js:984,1129,1136,1143,1149,1422`), tidak pernah untuk
        // command biasa. Membakarnya per-pesan akan menguras kuota orang yang
        // cuma mengetik `/menu`. Yang dijaga di sini adalah memoization-nya:
        // satu pesan tidak boleh membakar kuota lebih dari sekali
        // (`createAutoWorkGuard`, `handler.js:416-422`) — itu yang
        // [`Pipeline::allow_auto_work`] jamin.
        //
        // --- Gate 4 + 5: cooldown lalu energi, HANYA untuk pesan yang akan
        // benar-benar dispatch -------------------------------------------
        // `Router::route` menyatukan permission DAN pemanggilan handler
        // (`oktz-router/src/lib.rs:122-159`), sedangkan JS memakai permission
        // di 1810, cooldown di 1837, energi di 1851, baru dispatch di 1896.
        // Gate 4/5 butuh `meta` plugin, jadi keduanya hanya jalan setelah
        // router menyatakan pesan ini layak — kalau tidak, user yang
        // permission-nya ditolak akan tetap kehilangan energi dan cooldown-nya
        // dibakar untuk command yang tidak pernah dieksekusi.
        // ------------------------------------------------------------------------
        let mut cooldown_key: Option<(String, u32)> = None;
        if let Some(entry) = self.dispatch_target(&mut caller, &body, lookup, now_ms) {
            cooldown_key = Some((entry.meta.name.clone(), entry.meta.cooldown));
            let meta = &entry.meta;
            // `cmdKey` (`handler.js:1816`) — nama plugin, bukan command yang
            // diketik user, jadi alias menulis cooldown yang sama.
            let cmd_key = meta.name.as_str();

            if self
                .gates
                .cooldown_remaining(&caller, cmd_key, meta.cooldown, now_ms)
                .is_some()
            {
                // `m.react("⏱️")` gagal tidak boleh menggagalkan pipeline —
                // di JS juga `.catch(() => {})`.
                if let Err(e) = self
                    .sender
                    .react(&caller.chat, &message_key_of(m), "⏱️")
                    .await
                {
                    tracing::warn!(error = %e, "gagal bereaksi cooldown");
                }
                return Ok(Outcome::CooldownBlocked);
            }

            // `energiEnabled = db.setting("energi") !== undefined ? ... : config.energi?.enabled !== false`
            // (`handler.js:1851-1854`) — setting store menang atas config.
            let mut energi = self.energi_settings;
            energi.enabled =
                resolve_energi_enabled(self.ctx.energi_enabled_setting(), self.energi_settings.enabled);
            let energy =
                self.gates
                    .charge_energi(&caller, meta, &energi, &self.cap_energi_overrides());
            if !energy.allowed {
                // `config.messages?.energiExceeded || "⚡ Energi habis!"`
                // (`handler.js:1879`) — pakai pesan config, bukan fallback.
                // Dikirim lewat `m.reply` di JS, jadi bentuknya ikut
                // `replyVariant` (`serialize.js:958`).
                let pesan = oktz_core::config::Messages::default().energi_exceeded;
                let reply = Reply::text(pesan);
                let target = reply_target(m, &caller.sender);
                if let Some(rendered) = self.render_with_reply_variant(&reply, &target)? {
                    self.deliver(&caller.chat, rendered).await?;
                }
                return Ok(Outcome::EnergiDenied);
            }
            // `db.setting("notiflimit")` → laporkan potongannya
            // (`handler.js:1884-1888`).
            if energy.deducted > 0
                && let Some(sisa) = energy.remaining
                && self.ctx.notiflimit_setting()
            {
                let tpl = oktz_core::config::Messages::default().limit_deducted;
                let pesan = oktz_router::gates::render_limit_deducted(&tpl, energy.deducted, sisa);
                let reply = Reply::text(pesan);
                let target = reply_target(m, &caller.sender);
                // `m.reply` di JS — bentuknya ikut `replyVariant`.
                if let Some(rendered) = self.render_with_reply_variant(&reply, &target)? {
                    self.deliver(&caller.chat, rendered).await?;
                }
            }
        }

        {
            // `&*self.ctx`: trait-nya diimplementasikan untuk `StoreContext`, bukan
            // `Arc<StoreContext>` — Arc cuma dipakai supaya bisa dipegang wrapper.
            let router = Router::new(&*self.ctx, self.parse_opts.clone(), now_ms);
            let decision = router.route(&mut caller, &body, lookup);
            let reply = decision_to_reply(&decision);

            // --- Tulis mutasi SEBELUM balasan dikirim --------------------
            // Plugin JS melakukan `db.save()` lalu `m.reply(...)`, jadi
            // pada saat user membaca "✅ tersimpan", data sudah di disk.
            // Kalau urutannya dibalik, user bisa melihat konfirmasi untuk
            // data yang hilang kalau proses mati di antaranya.
            //
            // Kegagalan TIDAK ditelan: teks balasan plugin hampir selalu
            // mengklaim berhasil ("PRODUK DITAMBAHKAN", "TERSIMPAN"), jadi
            // mengirim pesan itu tanpa data = berbohong ke user. Yang
            // dilakukan: log error dengan konteks penuh dan JANGAN kirim
            // balasan yang menyesatkan. Pesan berikutnya dari user yang
            // sama akan membaca store apa adanya, jadi tidak ada state
            // setengah jadi.
            let (reply, mutations) = reply.into_parts();
            if !mutations.is_empty() {
                let plugin = match &decision {
                    Decision::Handled(_) => caller.command.clone(),
                    _ => "<router>".to_string(),
                };
                if let Err(e) = self.ctx.apply(&mutations, now_ms) {
                    tracing::error!(
                        error = %e,
                        plugin = %plugin,
                        chat = %caller.chat,
                        sender = %caller.sender,
                        mutations = mutations.len(),
                        "gagal menyimpan mutasi plugin; balasan tidak dikirim"
                    );
                    return Ok(Outcome::StoreWriteFailed);
                }
            }

            let target = reply_target(m, &caller.sender);
            // `Reply::Reaction` (JS `m.react('✅')`) bukan message — dikirim
            // lewat jalur reaksi. Seperti cooldown, kegagalan bereaksi tidak
            // boleh menggagalkan sisa pipeline.
            if let Reply::Reaction { emoji, reply: inner } = &reply {
                if let Err(e) = self.sender.react(&caller.chat, &message_key_of(m), emoji).await {
                    tracing::warn!(error = %e, emoji = %emoji, "gagal bereaksi");
                }
                // Teks balasan (kalau ada) tetap dikirim setelah reaksi.
                if let Some(rendered) = render_reply(inner, &target)? {
                    self.deliver(&caller.chat, rendered).await?;
                }
                return Ok(Outcome::Sent);
            }

            // `replyVariant` = 9 (setting produksi aktif) mengirim
            // `orderMessage`, bukan teks biasa (`serialize.js:1231-1259`).
            // Varian lain belum diimplementasikan dan tetap teks.
            //
            // `ponytail:` reply BERMEDIA tidak ikut varian — di JS `serialize`
            // menerima teks terpisah dari media, jadi perlu Arcade
            // `src/lib/socket.js` untuk memastikan apa yang terjadi pada
            // `m.reply({image}, {text})`. Sampai itu dicek, jalur lama
            // dipertahankan: user tetap dapat gambar, cuma bukan bubble pesanan.
            // `Reply::Sequence` — beberapa pesan berurutan dari satu handler.
            // Mutasi SUDAH ditulis di atas, jadi setiap elemen cukup dirender
            // dan dikirim; error di tengah menghentikan sisa, meniru `await`
            // berurutan di JS yang melempar dan tidak mengirim pesan berikutnya.
            if let Reply::Sequence(list) = reply.without_mutations() {
                // Id pesan Assignment: pesan berikutnya yang menempel
                // (`messageAssociation`) harus tahu id induknya, jadi id
                // ditulis sebelum pesan dikirim.
                let mut induk: Option<String> = None;
                for item in list {
                    let menempel = matches!(
                        item.without_mutations(),
                        Reply::PairedVideo { .. }
                    );
                    let Some(mut r) = self.render_with_reply_variant(item, &target)? else {
                        continue;
                    };
                    let id = message_id_baru();
                    r.message_id = Some(id.clone());
                    if menempel
                        && let Some(parent) = &induk
                        && let Some(media) = r.media.as_mut()
                    {
                        media.association = Some((parent.clone(), caller.chat.clone()));
                        media.paired = true;
                    }
                    self.deliver(&caller.chat, r).await?;
                    induk = Some(id);
                }
                if matches!(decision, Decision::Handled(_))
                    && let Some((name, secs)) = cooldown_key
                {
                    self.gates.start_cooldown(&caller, &name, secs, now_ms);
                }
                return Ok(if matches!(decision, Decision::Handled(_)) {
                    Outcome::Sent
                } else {
                    Outcome::Denied
                });
            }

            let Some(rendered) = self.render_with_reply_variant(&reply, &target)? else {
                // `Reply::None`: `DeniedMode::Silent`, `SelfGroup`, `NoPlugin`,
                // `NotACommand`, atau `Reply::None` dari plugin — semua sengaja diam.
                return Ok(if matches!(decision, Decision::Handled(_)) {
                    Outcome::Sent
                } else {
                    Outcome::NotACommand
                });
            };

            let outcome = if matches!(decision, Decision::Handled(_)) {
                Outcome::Sent
            } else {
                Outcome::Denied
            };

            self.deliver(&caller.chat, rendered).await?;

            // Cooldown baru mulai SETELAH handler selesai
            // (`handler.js:1922-1924`), jadi command yang gagal di tengah tidak
            // membekukan cooldown.
            if matches!(decision, Decision::Handled(_))
                && let Some((name, secs)) = cooldown_key
            {
                self.gates.start_cooldown(&caller, &name, secs, now_ms);
            }
            Ok(outcome)
        }
    }

    /// Kapasitas antrean inbound. Fixed, bukan tak berbatas: `handle_event`
    /// sinkron dan dipanggil INLINE di bus event (wacore-0.7.0
    /// `src/types/events.rs:609-621`), jadi satu-satunya operasi yang aman di
    /// sana adalah `try_send` — `send().await` akan menahan seluruh dispatch
    /// event bot, dan `send_blocking` bisa deadlock kalau handler-nya jalan di
    /// thread tokio worker. Channel tak berbatas (pola `ChannelEventHandler`,
    /// `events.rs:362-370`) bebas deadlock tapi membiarkan memory tumbuh tanpa
    /// batas saat bot tidak mampu mengikuti, jadi dipakai capacity tetap.
    const INGEST_CAPACITY: usize = 256;

    /// Drain antrean: satu pesan → satu [`Pipeline::process`].
    ///
    /// Dipisah dari [`Pipeline::attach`] supaya loop ini bisa diuji dengan
    /// channel buatan sendiri — `attach` sendiri butuh `Arc<Client>` sungguhan.
    ///
    /// Berhenti hanya kalau semua `tx` di-drop, yaitu saat `Pipeline` beserta
    /// task ini dijatuhkan.
    pub async fn run(self: Arc<Self>, rx: IngestRx) {
        while let Ok(m) = rx.recv().await {
            let preview: String = extract_body(&m.message)
                .unwrap_or_default()
                .chars()
                .filter(|c| !c.is_control())
                .take(120)
                .collect();
            match self.process(&m, now_ms()).await {
                Ok(o) => tracing::info!(
                    id = %m.info.id,
                    chat = %m.info.source.chat,
                    sender = %m.info.source.sender,
                    body = %preview,
                    outcome = ?o,
                    "pesan inbound diproses"
                ),
                Err(e) => tracing::error!(
                    error = %e,
                    id = %m.info.id,
                    chat = %m.info.source.chat,
                    sender = %m.info.source.sender,
                    body = %preview,
                    "pipeline gagal memproses pesan"
                ),
            }
        }
        tracing::debug!("antrean pesan ditutup; loop pipeline berhenti");
    }

    /// Daftarkan handler ke bus event dan mulai task async pemroses.
    ///
    /// Aman dipanggil lebih dari sekali: `Event::Connected` dikirim ulang per
    /// generasi koneksi (`whatsapp-rust-0.7.0/src/client/lifecycle.rs:230`),
    /// jadi reconnect menjalankan `on_ready` lagi. `Subscription` lama di-drop
    /// (`wacore-0.7.0/src/types/events.rs:538-544`) → handler lama unsubscribe
    /// → `tx`-nya hilang → task lama keluar dari [`Pipeline::run`]. Kalau tidak,
    /// dua loop hidup dan satu pesan dibalas dua kali.
    pub fn attach(self: &Arc<Self>, client: &Arc<Client>) {
        let (tx, rx) = whatsapp_rust::async_channel::bounded(Self::INGEST_CAPACITY);
        let (grup_tx, grup_rx) = whatsapp_rust::async_channel::bounded(Self::INGEST_CAPACITY);
        let handler = Arc::new(PipelineHandler {
            tx,
            grup: grup_tx,
        });
        let sub = client.subscribe(
            EventInterest::of(&[EventKind::Messages, EventKind::GroupUpdate]),
            handler,
        );
        let mut slot = self.subscription.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(sub);
        drop(slot);
        tokio::spawn(self.clone().run(rx));
        tokio::spawn(self.clone().run_grup(grup_rx));
    }

    /// Drain antrean update peserta grup → [`Pipeline::tangani_update_grup`].
    ///
    /// Pisah dari [`Pipeline::run`] karena bentuk datanya beda; dua-duanya
    /// berbagi satu `Subscription` supaya reconnect tidak meninggalkan
    /// langganan yatim.
    pub async fn run_grup(self: Arc<Self>, rx: GrupRx) {
        while let Ok(u) = rx.recv().await {
            if let Err(e) = self.tangani_update_grup(&u).await {
                tracing::warn!(
                    error = %e,
                    grup = %u.id,
                    aksi = ?u.aksi,
                    "update peserta grup gagal ditangani"
                );
            }
        }
        tracing::debug!("antrean update grup ditutup");
    }

    /// Kirim teks ke sebuah chat; `mention` mengisi `mentions` (peserta yang
    /// baru masuk/keluar), `None` = tanpa mention.
    async fn kirim_teks(
        &self,
        chat: &str,
        teks: &str,
        mention: Option<&str>,
    ) -> Result<(), PipelineError> {
        let target = ReplyTarget {
            quote: None,
            sender: mention.unwrap_or("").to_string(),
        };
        let rendered = render_reply(&Reply::text(teks), &target)?;
        if let Some(r) = rendered {
            let opts = send_options(&r);
            self.sender
                .send(chat, r.message, opts)
                .await
                .map_err(|e| PipelineError::Other(format!("gagal kirim: {e}")))?;
        }
        Ok(())
    }

    async fn tidur(&self, ms: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }

    /// Bot sendiri ada di daftar peserta yang baru masuk?
    ///
    /// `isBotAdded` (`src/connection.js:1218-1235`) memakai `phoneNumber || id`
    /// lalu membandingkan digit maupun `includes`, jadi dua bentuk itu ikut
    /// diperiksa di sini.
    fn bot_baru_ditambahkan(&self, u: &UpdateGrup) -> bool {
        u.aksi == AksiGrup::Add
            && u.peserta.iter().any(|p| {
                let kandidat = p.nomor.as_deref().unwrap_or(&p.jid);
                let nomor = grup_event::nomor_jid(kandidat);
                nomor == self.bot_id
                    || self.bot_id.contains(&nomor)
                    || kandidat.contains(&self.bot_id)
            })
    }

    /// `handleAntiCulik` (`plugins/group/anticulik.js:71-127`).
    ///
    /// `true` = bot ditambahkan orang asing dan sudah dijadwalkan keluar.
    /// Badge newsletter (`saluranCtx()`) tidak ikut — jalur context itu belum
    /// ada, sama seperti catatan di `docs/rust-rewrite/BLOCKED.md` §15.
    async fn anticulik_menolak(&self, u: &UpdateGrup) -> Result<bool, PipelineError> {
        let grup = self.ctx.group_entry(&u.id);
        let setting = self.ctx.settings_entry();
        let status = grup
            .get("anticulik")
            .and_then(Value::as_str)
            .or_else(|| setting.get("anticulik").and_then(Value::as_str))
            .unwrap_or("off");
        if status != "on" {
            return Ok(false);
        }

        let pelaku = u.pelaku.clone().unwrap_or_default();
        // JS punya catatan khusus `event.author` yang bisa berupa LID; owner
        // yang mengundang tidak boleh dianggap orang asing
        // (`plugins/group/anticulik.js:100-109`).
        if !pelaku.is_empty()
            && (self.identity.is_owner(&pelaku)
                || grup_event::nomor_jid(&pelaku) == self.bot_id)
        {
            return Ok(false);
        }

        let sebut = if pelaku.is_empty() {
            "seseorang".to_string()
        } else {
            format!("@{}", grup_event::nomor_jid(&pelaku))
        };
        let teks = format!(
            "🛡️ *Anti Culik*\n\nMinimal izin dulu ya bang, jangan asal culik 🗿\n\n\
             > Bot ditambah oleh {sebut} tanpa izin\n\
             > Bot akan keluar dari grup ini\n\n\
             _Hubungi owner untuk menambahkan bot dengan cara yang benar_"
        );
        self.kirim_teks(&u.id, &teks, (!pelaku.is_empty()).then_some(pelaku.as_str()))
            .await?;
        // JS menunggu 2 detik supaya pesannya sempat terkirim sebelum keluar.
        self.tidur(2000).await;
        self.plugin_src.sock.keluar_grup(&u.id).map_err(|e| {
            PipelineError::Other(format!("keluar grup {} (anticulik): {e}", u.id))
        })?;
        tracing::warn!(grup = %u.id, pelaku = %pelaku, "anticulik: bot keluar dari grup");
        Ok(true)
    }

    /// `src/connection.js:1244-1272` — grup di luar daftar sewa ditinggalkan.
    ///
    /// `Some(teks)` = kirim teks itu lalu keluar; `None` = boleh tinggal.
    fn sewa_penolakan(&self, u: &UpdateGrup) -> Option<String> {
        let setting = self.ctx.settings_entry();
        let sewa = setting.get("sewa")?;
        if sewa.get("enabled").and_then(Value::as_bool) != Some(true) {
            return None;
        }
        let grup = sewa.get("groups").and_then(|g| g.get(&u.id));
        let terdaftar = grup
            .map(|g| {
                g.get("isLifetime").and_then(Value::as_bool) == Some(true)
                    || g.get("expiredAt")
                        .and_then(Value::as_i64)
                        .map(|t| t > now_ms())
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        if terdaftar {
            return None;
        }
        // `config.bot.support` belum punya jalur file di Rust; JS jatuh ke
        // `config.bot.developer` lalu "owner" kalau dua-duanya kosong.
        let kontak = {
            let dev = self.plugin_src.config.bot.developer.trim();
            if dev.is_empty() {
                "owner".to_string()
            } else {
                dev.to_string()
            }
        };
        Some(format!(
            "⛔ *sᴇᴡᴀʙᴏᴛ*\n\n\
             > Grup ini tidak terdaftar dalam sistem sewa.\n\
             > Bot akan meninggalkan grup ini.\n\n\
             _Hubungi {kontak} untuk sewa bot._"
        ))
    }

    /// Salam saat bot baru dimasukkan ke grup (`src/connection.js:1274-1317`).
    async fn salam_bot_bergabung(&self, u: &UpdateGrup) -> Result<(), PipelineError> {
        let pelaku = u.pelaku.clone().unwrap_or_default();
        let sebut = if pelaku.is_empty() {
            "seseorang".to_string()
        } else {
            format!("@{}", grup_event::nomor_jid(&pelaku))
        };
        let meta = self.plugin_src.sock.metadata_grup(&u.id);
        let nama_grup = meta
            .ok()
            .and_then(|m| m.get("subject").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "grup ini".to_string());
        let prefix = self
            .parse_opts
            .prefixes
            .first()
            .cloned()
            .unwrap_or_else(|| ".".to_string());
        let dev = {
            let d = self.plugin_src.config.bot.developer.trim();
            if d.is_empty() {
                "Lucky Archz".to_string()
            } else {
                d.to_string()
            }
        };
        let nama_bot = {
            let n = self.plugin_src.config.bot.name.trim();
            if n.is_empty() {
                "Bot".to_string()
            } else {
                n.to_string()
            }
        };
        let teks = format!(
            "👋 *ʜᴀɪ, sᴀʟᴀᴍ ᴋᴇɴᴀʟ!*\n\n\
             Aku *{nama_bot}* 🤖\n\n\
             Terima kasih sudah mengundang aku ke *{nama_grup}*!\n\
             Aku diundang oleh {sebut} ✨\n\n\
             ╭┈┈⬡「 📋 *ɪɴꜰᴏ* 」\n\
             ┃ 🔧 Developer: *{dev}*\n\
             ┃ 📢 Prefix: `{prefix}`\n\
             ┃ 📩 Support: -\n\
             ╰┈┈⬡\n\n\
             > Ketik `{prefix}menu` untuk melihat daftar fitur\n\
             > Ketik `{prefix}help` untuk bantuan"
        );
        self.kirim_teks(&u.id, &teks, (!pelaku.is_empty()).then_some(pelaku.as_str()))
            .await?;
        tracing::info!(grup = %u.id, nama = %nama_grup, "bot bergabung ke grup");
        Ok(())
    }

    /// `groupHandler(update, sock)` (`src/handler.js:1981`) — versi Rust.
    ///
    /// Yang dipindah: grup tanpa catatan diberi default dulu
    /// (`src/handler.js:1996-2003`), gerbang `welcome`/`goodbye` per grup,
    /// terjemahan LID → JID lewat metadata, lalu kirim pesan teks (tipe 1)
    /// dengan mention peserta.
    ///
    /// Yang DIKETAHUI belum: `welcomeType`/`goodbyeType` 2..8 (kartu/album,
    /// butuh media + relay) dan banner `promote`/`demote`
    /// (`src/handler.js:2090-2130`, butuh `config.assets`) — dilewati dengan
    /// `warn`, bukan diganti bentuk lain.
    async fn tangani_update_grup(&self, u: &UpdateGrup) -> Result<(), PipelineError> {
        match u.aksi {
            AksiGrup::Promote | AksiGrup::Demote => {
                tracing::warn!(grup = %u.id, aksi = ?u.aksi, "banner promote/demote belum ada jalur medianya; dilewati");
                return Ok(());
            }
            AksiGrup::Add | AksiGrup::Remove => {}
            AksiGrup::Modify | AksiGrup::Lain => return Ok(()),
        }

        // Satu event diproses DUA listener di JS: `src/connection.js:1199`
        // (bot baru ditambahkan → anticulik, sewa, salam) dan
        // `src/handler.js:1981` (welcome/goodbye). Dua-duanya jalan, jadi di
        // sini pun tidak ada early-return setelah jalur bot.
        if self.bot_baru_ditambahkan(u) {
            self.anticulik_menolak(u).await?;
            if let Some(teks) = self.sewa_penolakan(u) {
                self.kirim_teks(&u.id, &teks, None).await?;
                self.tidur(2000).await;
                self.plugin_src.sock.keluar_grup(&u.id).map_err(|e| {
                    PipelineError::Other(format!("keluar grup {} (sewa): {e}", u.id))
                })?;
                tracing::warn!(grup = %u.id, "sewa: bot keluar dari grup tak terdaftar");
            } else {
                self.salam_bot_bergabung(u).await?;
            }
        }

        if self.ctx.group_entry(&u.id).is_null() {
            self.ctx
                .apply(
                    &[Mutation::SetGroup {
                        jid: u.id.clone(),
                        data: grup_event::default_grup(),
                    }],
                    now_ms(),
                )
                .map_err(|e| PipelineError::Other(format!("tulis default grup: {e}")))?;
        }
        let grup = self.ctx.group_entry(&u.id);
        let masuk = u.aksi == AksiGrup::Add;
        let boleh = if masuk {
            grup_event::boleh_welcome(&grup)
        } else {
            grup_event::boleh_goodbye(&grup)
        };
        if !boleh {
            return Ok(());
        }
        let setting = self.ctx.settings_entry();
        let tipe = if masuk {
            grup_event::tipe_welcome(&setting)
        } else {
            grup_event::tipe_goodbye(&setting)
        };
        if tipe != 1 {
            tracing::warn!(
                grup = %u.id,
                tipe,
                "welcome/goodbye tipe ini butuh kartu/album yang belum ada; dilewati"
            );
            return Ok(());
        }

        let meta = self
            .plugin_src
            .sock
            .metadata_grup(&u.id)
            .map_err(|e| PipelineError::Other(format!("metadata grup {}: {e}", u.id)))?;
        let peserta_meta = meta.get("participants").cloned().unwrap_or(Value::Null);
        let jumlah = peserta_meta.as_array().map(Vec::len).unwrap_or(0);
        let nama_grup = meta
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or("Grup")
            .to_string();
        // JS menyerahkan `groupMeta.descOwner` sebagai "groupDesc"
        // (`plugins/group/welcome.js:137`, `goodbye.js:165`) — kuirk yang
        // ditiru apa adanya, bukan diperbaiki.
        let deskripsi = meta
            .get("descOwner")
            .and_then(Value::as_str)
            .map(str::to_string);
        let owner = meta
            .get("owner")
            .and_then(Value::as_str)
            .map(grup_event::nomor_jid)
            .unwrap_or_default();
        let waktu = grup_event::waktu_lokal();
        let prefix = self
            .parse_opts
            .prefixes
            .first()
            .cloned()
            .unwrap_or_else(|| ".".to_string());
        let bot_nama = self.plugin_src.config.bot.name.clone();
        let custom = if masuk {
            grup.get("welcomeMsg")
        } else {
            grup.get("goodbyeMsg")
        }
        .and_then(Value::as_str)
        .map(str::to_string);
        // JS `Math.random()`; di sini jam dinding supaya pilihan tips berganti
        // tiap pesan tanpa crate rng.
        let acak = now_ms() as u64;

        for p in &u.peserta {
            let jid = grup_event::jid_peserta(p, &peserta_meta);
            let nomor = grup_event::nomor_jid(&jid);
            let k = KonteksPesan {
                nomor: &nomor,
                nama_grup: &nama_grup,
                deskripsi: deskripsi.as_deref(),
                jumlah_member: jumlah,
                owner_grup: &owner,
                prefix: &prefix,
                bot_nama: &bot_nama,
                tanggal: &waktu.tanggal,
                jam: &waktu.jam,
                hari: &waktu.hari,
            };
            let teks = if masuk {
                pesan_grup::welcome(custom.as_deref(), &k, acak)
            } else {
                pesan_grup::goodbye(custom.as_deref(), &k, acak)
            };
            let target = ReplyTarget {
                quote: None,
                sender: jid.clone(),
            };
            let rendered = render_reply(&Reply::text(teks), &target)?;
            if let Some(r) = rendered {
                let opts = send_options(&r);
                self.sender
                    .send(&u.id, r.message, opts)
                    .await
                    .map_err(|e| PipelineError::Other(format!("gagal kirim sapaan: {e}")))?;
            }
        }
        Ok(())
    }
}

/// Sisi terima antrean inbound. Alias supaya tanda tangan `run`/`attach`
/// tidak menulis ulang path crate.
pub type IngestRx = whatsapp_rust::async_channel::Receiver<InboundMessage>;
type IngestTx = whatsapp_rust::async_channel::Sender<InboundMessage>;

/// Sisi terima antrean update peserta grup.
pub type GrupRx = whatsapp_rust::async_channel::Receiver<UpdateGrup>;
type GrupTx = whatsapp_rust::async_channel::Sender<UpdateGrup>;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Menerima event dan mengantre ke task pemrosesan.
///
/// Satu-satunya pekerjaan di sini adalah `try_send`. Dipanggil inline di
/// thread mana pun yang sedang memproses stanza, jadi tidak boleh-blocking,
/// tidak boleh async, dan tidak boleh panik.
struct PipelineHandler {
    tx: IngestTx,
    grup: GrupTx,
}

impl EventHandler for PipelineHandler {
    fn handle_event(&self, event: Arc<Event>) {
        // Satu langganan dua bentuk: update grup dicoba dulu (murah, cuma
        // cocokkan varian), sisanya jalur pesan seperti sebelumnya.
        if let Some(u) = grup_event::UpdateGrup::dari_event(&event) {
            if let Err(e) = self.grup.try_send(u) {
                tracing::warn!(
                    error = %e,
                    queue = Pipeline::INGEST_CAPACITY,
                    "antrean update grup penuh; event dibuang"
                );
            }
            return;
        }
        let Event::Messages(batch) = &*event else {
            return;
        };
        for m in batch.iter() {
            // Penuh = bot tidak mampu mengikuti. Menunggu di sini akan menahan
            // bus, jadi yang tersisa adalah buang + catat: menelan perintah
            // tanpa jejak jauh lebih sulit dicari daripada satu baris warn.
            if let Err(e) = self.tx.try_send(m.clone()) {
                tracing::warn!(
                    error = %e,
                    id = %m.info.id,
                    chat = %m.info.source.chat,
                    queue = Pipeline::INGEST_CAPACITY,
                    "antrean pesan penuh; pesan dibuang"
                );
            }
        }
    }

    fn interest(&self) -> EventInterest {
        EventInterest::of(&[EventKind::Messages, EventKind::GroupUpdate])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grup_event::PesertaGrup;
    use oktz_plugin_api::registry::InteractiveButton;
    use serde_json::json;
    use whatsapp_rust::{GroupParticipant, ParticipantType};
    use whatsapp_rust::types::message::MessageSource;
    use whatsapp_rust::waproto::whatsapp::__buffa::oneof::message::interactive_message::InteractiveMessage as ImOneof;

    // ---------- helper ----------

    // `cargo test` menjalankan test dalam thread paralel; nama file tetap
    // membuat mereka saling menimpa file yang sama. Nomor unik per panggilan.
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    fn store_with(name: &str, body: &str) -> Arc<JsonStore> {
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("oktz-pipeline-{name}-{n}.json"));
        std::fs::write(&p, body).unwrap();
        Arc::new(JsonStore::open(&p).unwrap())
    }

    /// Store dengan layout file bot JS: satu file per koleksi di satu folder
    /// (`database.js:127-136`). Kembalikan foldernya supaya test bisa
    /// menulis file koleksi lain.
    fn store_js_layout(
        name: &str,
        settings: &str,
        users: &str,
        groups: &str,
    ) -> (PathBuf, Arc<JsonStore>) {
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("oktz-pipeline-js-{name}-{n}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("settings.json"), settings).unwrap();
        std::fs::write(dir.join("users.json"), users).unwrap();
        std::fs::write(dir.join("groups.json"), groups).unwrap();
        let p = dir.join("settings.json");
        let store = Arc::new(JsonStore::open(&p).unwrap());
        (dir, store)
    }

    /// `PipelineHandler` untuk pesan; antrean grup dibuat tapi tidak dipakai.
    fn handler_pesan(tx: IngestTx) -> PipelineHandler {
        let (grup, _grx) = whatsapp_rust::async_channel::unbounded();
        PipelineHandler { tx, grup }
    }

    #[derive(Clone, Default)]
    struct RecordingSender {
        sent: Arc<Mutex<Vec<(String, wa::Message, usize)>>>,
        media: Arc<Mutex<Vec<(String, PendingMedia)>>>,
        /// `(chat, message_id, original_sender)` per panggilan hapus.
        deleted: Arc<Mutex<Vec<(String, String, String)>>>,
        /// `(chat, emoji)` per panggilan reaksi.
        reacted: Arc<Mutex<Vec<(String, String)>>>,
        /// `delete_message` gagal — menguji pembatalan aksi saat hapus gagal
        /// (JS: error delete naik ke `catch` gate → aksi dibatalkan).
        delete_fails: bool,
    }

    impl RecordingSender {
        fn deleted(&self) -> Vec<(String, String, String)> {
            self.deleted.lock().unwrap().clone()
        }
        fn reacted(&self) -> Vec<(String, String)> {
            self.reacted.lock().unwrap().clone()
        }
        fn texts(&self) -> Vec<String> {
            self.sent
                .lock()
                .unwrap()
                .iter()
                .map(|(to, m, _)| format!("{to}: {}", body_of(m).unwrap_or("<non-teks>")))
                .collect()
        }
    }

    impl MessageSender for RecordingSender {
        fn send(&self, to: &str, message: wa::Message, opts: SendOptions) -> SendBox<'_> {
            self.sent
                .lock()
                .unwrap()
                .push((to.to_string(), message, opts.extra_stanza_nodes.len()));
            Box::pin(async move { Ok(()) })
        }

        fn send_media(&self, to: &str, media: PendingMedia, _opts: SendOptions) -> SendBox<'_> {
            self.media
                .lock()
                .unwrap()
                .push((to.to_string(), media));
            Box::pin(async move { Ok(()) })
        }

        fn delete_message(
            &self,
            chat: &str,
            message_id: &str,
            original_sender: &str,
        ) -> SendBox<'_> {
            self.deleted.lock().unwrap().push((
                chat.to_string(),
                message_id.to_string(),
                original_sender.to_string(),
            ));
            if self.delete_fails {
                return Box::pin(async { Err("gagal hapus".into()) });
            }
            Box::pin(async move { Ok(()) })
        }

        fn react(&self, chat: &str, _target: &wa::MessageKey, emoji: &str) -> SendBox<'_> {
            self.reacted
                .lock()
                .unwrap()
                .push((chat.to_string(), emoji.to_string()));
            Box::pin(async move { Ok(()) })
        }
    }

    /// Roster tetap yang mengaku bot sebagai admin grup.
    struct AdminRoster;

    impl RosterSource for AdminRoster {
        fn roster(&self, _chat: &str) -> SendBoxRoster<'_> {
            Box::pin(async {
                Ok(RosterSnapshot {
                    roster: GroupRoster {
                        is_bot_admin: true,
                        participants: vec![
                            Participant {
                                jid: "628111@s.whatsapp.net".into(),
                                is_admin: false,
                                is_owner: false,
                                data: Default::default(),
                            },
                            Participant {
                                jid: "628222@s.whatsapp.net".into(),
                                is_admin: true,
                                is_owner: true,
                                data: Default::default(),
                            },
                            // Bot = participant admin dengan nomor "bot".
                            Participant {
                                jid: "628999@lid".into(),
                                is_admin: true,
                                is_owner: false,
                                data: Default::default(),
                            },
                        ],
                    },
                    // Bentuk gate: identitas tiga-bentuk per peserta.
                    participants: vec![
                        oktz_router::group::Participant {
                            id: "628111@s.whatsapp.net".into(),
                            lid: String::new(),
                            phone_number: "628111@s.whatsapp.net".into(),
                            admin: false,
                        },
                        oktz_router::group::Participant {
                            id: "628222@s.whatsapp.net".into(),
                            lid: String::new(),
                            phone_number: "628222@s.whatsapp.net".into(),
                            admin: true,
                        },
                        oktz_router::group::Participant {
                            id: "628999@lid".into(),
                            lid: "628999@lid".into(),
                            phone_number: "628999@s.whatsapp.net".into(),
                            admin: true,
                        },
                    ],
                    available: true,
                })
            })
        }
    }

    fn inbound(body: &str, sender: &str, chat: &str, is_group: bool, id: &str) -> InboundMessage {
        // `InboundMessage` `#[non_exhaustive]` (events.rs:1125) — dibangun lewat
        // builder hasil derive `bon::Builder`.
        InboundMessage::builder()
            .message(Arc::new(wa::Message {
                conversation: Some(body.to_string()),
                ..Default::default()
            }))
            .info(Arc::new(MessageInfo {
                source: MessageSource {
                    chat: chat.parse().unwrap(),
                    sender: sender.parse().unwrap(),
                    is_from_me: false,
                    is_group,
                    ..Default::default()
                },
                id: id.to_string(),
                ..Default::default()
            }))
            .build()
    }

    fn pipeline_with(body: &str, identity: Identity) -> (Arc<Pipeline>, Arc<RecordingSender>) {
        let ctx = StoreContext::new(store_with("e2e", body));
        let sender = Arc::new(RecordingSender::default());
        let p = Pipeline::new(ctx, sender.clone(), "bot", ParseOptions::default())
            .with_identity(identity);
        (Arc::new(p), sender)
    }

    fn body_of(m: &wa::Message) -> Option<&str> {
        if let Some(c) = m.conversation.as_deref() {
            return Some(c);
        }
        m.extended_text_message.as_option()?.text.as_deref()
    }

    fn ctx_of(m: &wa::Message) -> Option<&wa::ContextInfo> {
        m.extended_text_message.as_option()?.context_info.as_option()
    }

    const OK_STORE: &str = r#"{
      "users": {
        "628111": {"isRegistered": true,
          "access": [{"cmd":"ping","expired":null},{"cmd":"old","expired":1}]},
        "lid:9998887771": {"isRegistered": false, "access": []}
      },
      "groups": {
        "120363@g.us": {"game": false, "rpg": true, "botMode": "store",
                       "selfAdmin": true, "publicAdmin": false, "onlyAdmin": true}
      },
      "settings": {
        "botMode": "public", "onlyGc": true, "onlyPc": false,
        "botAfk": {"active": true, "since": 1, "reason": "libur"},
        "onlyThisGroup": {"jid": "120999@g.us", "name": "Utama", "link": "https://x"},
        "selfGroups": ["111@g.us"], "publicGroups": ["222@g.us"],
        "selfAdmin": false, "publicAdmin": true, "onlyAdmin": false,
        "capprem": {"ping": false, "tebak": true}
      }
    }"#;

    // ---------- 1. normalisasi JID ----------

    #[test]
    fn plain_pn_jid_key_is_the_number() {
        assert_eq!(
            normalize_user_key("628123456789@s.whatsapp.net").as_deref(),
            Some("628123456789")
        );
        assert_eq!(
            normalize_user_key("628123456789:5@s.whatsapp.net").as_deref(),
            Some("628123456789:5")
        );
    }

/// Nomor grup diawali `120` dan nomor yang terlalu panjang ditolak
/// (`database.js:533`). JID `@s.whatsapp.net` dengan >14 digit justru
/// diklasifikasi LID oleh `isLidConverted` (`lid.js:156`) sehingga lolos —
/// itu perilaku JS, bukan bug.
#[test]
fn group_number_and_long_number_are_rejected() {
    // Nomor grup diawali 120.
    assert_eq!(normalize_user_key("120363001234567890@g.us"), None);
    assert_eq!(normalize_user_key("120363@g.us"), None);
    // > 15 karakter, dan bukan LID karena domainnya bukan @s.whatsapp.net/@lid.
    assert_eq!(normalize_user_key("1234567890123456@newsletter"), None);
    assert_eq!(normalize_user_key(""), None);
    // 15 digit di @s.whatsapp.net = "converted LID" -> tetap diterima, dengan
    // prefiks `lid:`. Dikunci supaya tidak dianggap regresi.
    assert_eq!(
        normalize_user_key("1234567890123456@s.whatsapp.net").as_deref(),
        Some("lid:1234567890123456")
    );
}

    #[test]
    fn lid_jid_uses_lid_prefixed_key_and_skips_length_check() {
        assert!(is_lid_like_jid("9998887771@lid"));
        assert_eq!(
            normalize_user_key("9998887771@lid").as_deref(),
            Some("lid:9998887771")
        );
        // Nomor LID > 15 karakter tetap lolos karena isLidLikeJid benar.
        assert_eq!(
            normalize_user_key("123456789012345678@lid").as_deref(),
            Some("lid:123456789012345678")
        );
    }

    #[test]
    fn converted_lid_number_is_detected_and_prefixed() {
        // 20 digit: > 14 dan >= 10 digit -> isLidConverted true (lid.js:156).
        assert!(is_lid_converted("99988877710001112233@s.whatsapp.net"));
        assert!(is_lid_like_jid("99988877710001112233@s.whatsapp.net"));
        assert_eq!(
            normalize_user_key("99988877710001112233@s.whatsapp.net").as_deref(),
            Some("lid:99988877710001112233")
        );
    }

    #[test]
    fn normal_phone_number_is_not_lid_like() {
        // "62" + 9 digit = 11,Within [len+6, len+12] -> isLidConverted false.
        assert!(!is_lid_converted("628123456789@s.whatsapp.net"));
        assert!(!is_lid_like_jid("628123456789@s.whatsapp.net"));
        // Prefix "@lid" wajib; "@s.whatsapp.net" lain tidak.
        assert!(!is_lid_converted("628123@g.us"));
    }

    // ---------- 2. adapter store ----------

    #[test]
    fn user_state_reads_access_and_registration_from_users_map() {
        let s = store_with("userstate", OK_STORE);
        let ctx = StoreContext::new(s);
        let u = ctx.user_state("628111@s.whatsapp.net");
        assert!(u.is_registered);
        let ping = u.access.iter().find(|a| a.cmd == "ping").unwrap();
        assert_eq!(ping.expired, None, "expired:null = tidak pernah kedaluwarsa");
        let old = u.access.iter().find(|a| a.cmd == "old").unwrap();
        assert_eq!(old.expired, Some(1));
    }

    #[test]
    fn missing_expired_key_counts_as_expired_not_never() {
        // `undefined` gagal di `=== null` DAN gagal di `> Date.now()` (middleware.js:43).
        let s = store_with(
            "missingexp",
            r#"{"users": {"628": {"access": [{"cmd": "x"}]}}}"#,
        );
        let ctx = StoreContext::new(s);
        let u = ctx.user_state("628@s.whatsapp.net");
        assert_eq!(u.access[0].expired, Some(0));
        assert!(!has_access_for(&u, "x", 1));
    }

    fn has_access_for(u: &UserState, cmd: &str, now_ms: i64) -> bool {
        u.access.iter().any(|a| a.cmd == cmd && a.expired.is_none_or(|e| e > now_ms))
    }

    #[test]
    fn lid_user_is_found_by_normalized_key() {
        let s = store_with("liduser", OK_STORE);
        let ctx = StoreContext::new(s);
        let u = ctx.user_state("9998887771@lid");
        assert!(!u.is_registered, "harus membaca users['lid:9998887771']");
    }

    #[test]
    fn rejected_jid_yields_default_user_state() {
        let s = store_with("rejectuser", OK_STORE);
        let ctx = StoreContext::new(s);
        let u = ctx.user_state("120363001234567890@g.us");
        assert!(!u.is_registered);
        assert!(u.access.is_empty());
    }

    #[test]
    fn group_state_reads_game_and_rpg_and_absent_group_is_none() {
        let s = store_with("groupstate", OK_STORE);
        let ctx = StoreContext::new(s);
        let g = ctx.group_state("120363@g.us").expect("grup ada");
        assert_eq!(g.game, Some(false));
        assert_eq!(g.rpg, Some(true));
        // Tidak ada di store -> None, bukan Some(default) (permission.rs:44-50).
        assert_eq!(ctx.group_state("999@g.us"), None);
    }

    #[test]
    fn group_bot_mode_defaults_to_all() {
        let s = store_with("botmode", OK_STORE);
        let ctx = StoreContext::new(s);
        assert_eq!(ctx.group_bot_mode("120363@g.us"), "store");
        assert_eq!(ctx.group_bot_mode("999@g.us"), "all");
    }

    #[test]
    fn mode_settings_read_every_documented_key() {
        let s = store_with("modesettings", OK_STORE);
        let ctx = StoreContext::new(s);
        let m = ctx.mode_settings();
        assert_eq!(m.mode, "public");
        assert!(m.only_gc);
        assert!(!m.only_pc);
        assert!(m.afk_active);
        // Object `{jid,...}` dinormalkan ke jid (middleware.js:184).
        assert_eq!(m.only_this_group.as_deref(), Some("120999@g.us"));
        assert_eq!(m.self_groups, vec!["111@g.us".to_string()]);
        assert_eq!(m.public_groups, vec!["222@g.us".to_string()]);
        assert!(!m.self_admin);
        assert!(m.public_admin);
        assert!(!m.only_admin);
    }

    #[test]
    fn only_this_group_accepts_plain_string_form() {
        let s = store_with(
            "otgstr",
            r#"{"settings": {"onlyThisGroup": "120abc@g.us"}}"#,
        );
        let ctx = StoreContext::new(s);
        assert_eq!(
            ctx.mode_settings().only_this_group.as_deref(),
            Some("120abc@g.us")
        );
    }

    #[test]
    fn missing_settings_map_falls_back_to_defaults() {
        let ctx = StoreContext::new(store_with("nosettings", "{}"));
        let m = ctx.mode_settings();
        assert_eq!(m.mode, "");
        assert!(!m.only_gc);
        assert_eq!(m.only_this_group, None);
        assert!(m.self_groups.is_empty());
    }

    #[test]
    fn group_overrides_are_per_field_optional() {
        let s = store_with("overrides", OK_STORE);
        let ctx = StoreContext::new(s);
        let o = ctx.group_overrides("120363@g.us").expect("grup ada");
        assert_eq!(o.self_admin, Some(true));
        assert_eq!(o.public_admin, Some(false));
        assert_eq!(o.only_admin, Some(true));
        assert!(ctx.group_overrides("999@g.us").is_none());
    }

    #[test]
    fn capprem_distinguishes_absent_from_false() {
        let s = store_with("capprem", OK_STORE);
        let ctx = StoreContext::new(s);
        // `false` HARUS-bedanya dari `None` (permission.rs:120-122).
        assert_eq!(ctx.capprem("ping"), Some(false));
        assert_eq!(ctx.capprem("tebak"), Some(true));
        assert_eq!(ctx.capprem("tidakada"), None);
    }

    #[test]
    fn uptime_is_elapsed_since_context_started() {
        let started = Instant::now() - std::time::Duration::from_secs(120);
        let ctx = StoreContext::with_started_at(store_with("uptime", "{}"), started);
        assert_eq!(ctx.uptime_secs(), 120);
    }

    // ---------- 3. dedup ----------

    #[test]
    fn duplicate_within_100ms_is_dropped() {
        let mut d = Dedup::new();
        assert!(!d.debounce("k", 1_000));
        assert!(d.debounce("k", 1_000 + DEBOUNCE_MS - 1));
    }

    #[test]
    fn same_key_after_window_is_accepted_again() {
        let mut d = Dedup::new();
        assert!(!d.debounce("k", 1_000));
        assert!(!d.debounce("k", 1_000 + DEBOUNCE_MS));
    }

    #[test]
    fn different_keys_do_not_collide() {
        let mut d = Dedup::new();
        assert!(!d.debounce("a", 1_000));
        assert!(!d.debounce("b", 1_000));
    }

    #[test]
    fn cleanup_only_runs_past_1000_entries_and_5s() {
        let mut d = Dedup::new();
        for i in 0..1001 {
            d.debounce(&format!("k{i}"), 1_000);
        }
        assert_eq!(d.len(), 1001);
        //_ENTRY ke-1002: > 1000 dan now-0 > 5000 (t=7000) -> sweep.
        d.debounce("trigger", 7_000);
        assert!(
            d.len() < 1001,
            "sweep harus membuang entri basi, dapat {}",
            d.len()
        );
    }

    // ---------- 4. Reply -> wire ----------

    #[test]
    fn text_without_mentions_is_plain_conversation() {
        let r = render_reply(&Reply::text("pong"), &ReplyTarget::default()).unwrap().unwrap();
        assert_eq!(r.message.conversation.as_deref(), Some("pong"));
        assert!(r.extra_nodes.is_empty());
    }

    #[test]
    fn text_mentions_go_into_context_info_not_body() {
        let reply = Reply::text_mention("hai @1", "6281@s.whatsapp.net");
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        assert!(r.message.conversation.is_none(), "harus extendedTextMessage");
        let c = ctx_of(&r.message).expect("context_info");
        assert_eq!(c.mentioned_jid, vec!["6281@s.whatsapp.net".to_string()]);
        assert_eq!(body_of(&r.message), Some("hai @1"));
    }

    #[test]
    fn text_quoted_carries_quoted_message_and_mention() {
        let quote = QuoteSource {
            stanza_id: "STANZA1".into(),
            participant: Some("6281@s.whatsapp.net".into()),
            message: Box::new(wa::Message::text("asal")),
        };
        let reply = Reply::TextQuoted {
            text: "tanggapan".into(),
            mentions: vec!["6282@s.whatsapp.net".into()],
        };
        let r = render_reply(&reply, &ReplyTarget { quote: Some(quote.clone()), sender: String::new() }).unwrap().unwrap();
        let c = ctx_of(&r.message).expect("context_info");
        assert_eq!(c.stanza_id.as_deref(), Some("STANZA1"));
        assert_eq!(c.participant.as_deref(), Some("6281@s.whatsapp.net"));
        assert!(c.quoted_message.as_option().is_some());
        assert_eq!(c.mentioned_jid, vec!["6282@s.whatsapp.net".to_string()]);
    }

    #[test]
    fn text_quoted_without_source_still_sends_text() {
        let reply = Reply::TextQuoted {
            text: "tanpa quote".into(),
            mentions: vec![],
        };
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        assert_eq!(body_of(&r.message), Some("tanpa quote"));
    }

    // ---------------------------------------------------------------
    // Default quote + mention (`serialize.js:983-1010`). Tanpa test ini,
    // `Reply::text` bisa kembali jadi "tidak mengutip, tidak mention" tanpa
    // ada yang gagal.
    // ---------------------------------------------------------------

    fn pemicu() -> QuoteSource {
        QuoteSource {
            stanza_id: "STANZA9".into(),
            participant: Some("6281@s.whatsapp.net".into()),
            message: Box::new(wa::Message::text("pesan pemicu")),
        }
    }

    fn target() -> ReplyTarget {
        ReplyTarget {
            quote: Some(pemicu()),
            sender: "6289@s.whatsapp.net".into(),
        }
    }

    #[test]
    fn plain_text_quotes_the_trigger_and_mentions_the_sender_by_default() {
        // `quotedMsg = options.quoted !== false ? msg : undefined` dan
        // `mentionedJid = options?.mentions || [m.sender] || []`.
        let r = render_reply(&Reply::text("pong"), &target()).unwrap().unwrap();
        let c = ctx_of(&r.message).expect("context_info");
        assert!(c.quoted_message.as_option().is_some(), "harus mengutip pemicu");
        assert_eq!(c.stanza_id.as_deref(), Some("STANZA9"));
        assert_eq!(c.mentioned_jid, vec!["6289@s.whatsapp.net".to_string()]);
    }

    #[test]
    fn explicit_mentions_replace_the_default_sender() {
        let reply = Reply::text_mention("hai", "6287@s.whatsapp.net");
        let r = render_reply(&reply, &target()).unwrap().unwrap();
        let c = ctx_of(&r.message).expect("context_info");
        assert_eq!(c.mentioned_jid, vec!["6287@s.whatsapp.net".to_string()]);
        assert!(c.quoted_message.as_option().is_some());
    }

    #[test]
    fn no_quote_drops_the_quoted_message_but_keeps_the_default_mention() {
        // `quoted: false` — hanya kutipan yang hilang, mention tetap default.
        let r = render_reply(&Reply::text_quoted("tanpa kutip"), &target())
            .unwrap()
            .unwrap();
        let c = ctx_of(&r.message).expect("context_info");
        assert!(c.quoted_message.as_option().is_none());
        assert_eq!(c.mentioned_jid, vec!["6289@s.whatsapp.net".to_string()]);
    }

    #[test]
    fn no_mention_quotes_but_lists_nobody() {
        // `mentions: []` di JS = daftar kosong, bukan "pakai default".
        let r = render_reply(&Reply::text_no_mention("diam"), &target())
            .unwrap()
            .unwrap();
        let c = ctx_of(&r.message).expect("context_info");
        assert!(c.quoted_message.as_option().is_some());
        assert!(c.mentioned_jid.is_empty());
    }

    #[test]
    fn interactive_reuses_card_and_adds_biz_node() {
        let r = render_reply(&Reply::interactive("pilih"), &ReplyTarget::default())
            .unwrap()
            .unwrap();
        assert_eq!(r.extra_nodes.len(), 1, "node biz wajib untuk native-flow");
        let inner = r
            .message
            .view_once_message
            .as_option()
            .and_then(|v| v.message.as_option())
            .expect("dibungkus viewOnceMessage");
        let im = inner.interactive_message.as_option().expect("interactive");
        let Some(ImOneof::NativeFlowMessage(nf)) = im.interactive_message.as_ref() else {
            panic!("harus native_flow");
        };
        assert_eq!(nf.message_params_json, None);
        assert!(nf.buttons.is_empty(), "tanpa interactiveButtons[]");
    }

    #[test]
    fn interactive_meneruskan_tombol_dari_interactivebuttons_js() {
        let r = render_reply(
            &Reply::interactive("hasil").with_button(InteractiveButton::copy(
                "📋 Copy Hasil Teks",
                "copy",
                "hasil",
            )),
            &ReplyTarget::default(),
        )
        .unwrap()
        .unwrap();
        let inner = r
            .message
            .view_once_message
            .as_option()
            .and_then(|v| v.message.as_option())
            .expect("dibungkus viewOnceMessage");
        let im = inner.interactive_message.as_option().expect("interactive");
        let Some(ImOneof::NativeFlowMessage(nf)) = im.interactive_message.as_ref() else {
            panic!("harus native_flow");
        };
        assert_eq!(nf.buttons.len(), 1);
        let b = &nf.buttons[0];
        assert_eq!(b.name.as_deref(), Some("cta_copy"));
        let params: serde_json::Value =
            serde_json::from_str(b.button_params_json.as_deref().unwrap()).unwrap();
        assert_eq!(params["copy_code"], "hasil");
        assert_eq!(params["id"], "copy");

        // Nama aksi di luar daftar WA ditolak, bukan dikirim buta.
        let err = render_reply(
            &Reply::interactive("x").with_button(InteractiveButton::new("cta_misterius", serde_json::json!({}))),
            &ReplyTarget::default(),
        )
        .unwrap_err();
        assert!(matches!(err, PipelineError::Unsupported(_)));
    }

    #[test]
    fn image_reply_without_a_source_is_a_typed_error_not_a_silent_drop() {
        let reply = Reply::Image {
            media: None,
            caption: "c".into(),
            mentions: vec![],
        };
        let err = render_reply(&reply, &ReplyTarget::default()).unwrap_err();
        assert_eq!(
            err,
            PipelineError::MediaSourceMissing { variant: "Image" }
        );
        let msg = err.to_string();
        assert!(msg.contains("tidak punya sumber media"), "pesan harus menyebut gap: {msg}");
        assert!(msg.contains("TIDAK dikirim"), "harus tegas tidak dikirim: {msg}");
    }

    #[test]
    fn sticker_reply_without_a_source_is_a_typed_error() {
        let err = render_reply(&Reply::Sticker { media: None }, &ReplyTarget::default()).unwrap_err();
        assert_eq!(
            err,
            PipelineError::MediaSourceMissing {
                variant: "Sticker"
            }
        );
    }

    #[test]
    fn image_reply_with_a_source_is_pending_media_not_a_wire_message() {
        let reply = Reply::image(MediaSource::url("https://x/y.png"), "halo");
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        assert!(r.needs_media(), "harus lewat jalur upload");
        let p = r.media.expect("pending media");
        assert_eq!(p.kind, MediaKind::Image);
        assert_eq!(p.source, MediaSource::url("https://x/y.png"));
        assert_eq!(p.caption, "halo");
        assert!(p.quote.is_none());
        // Invarian yang dikunci test `media_never_carries_a_wire_message`:
        // pesan TIDAK boleh terkirim apa adanya.
        assert_eq!(r.message, wa::Message::default());
        assert!(r.extra_nodes.is_empty());
    }

    #[test]
    fn image_reply_keeps_caption_and_mentions_for_the_pending_media() {
        let reply = Reply::Image {
            media: Some(MediaSource::bytes(vec![1, 2, 3])),
            caption: "capt".into(),
            mentions: vec!["6281@s.whatsapp.net".into()],
        };
        let p = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap().media.unwrap();
        assert_eq!(p.caption, "capt");
        assert_eq!(p.mentions, vec!["6281@s.whatsapp.net".to_string()]);
        assert_eq!(p.source.mimetype(), None);
    }

    #[test]
    fn sticker_reply_maps_to_sticker_pending_media() {
        let src = MediaSource::path("/tmp/a.webp").with_mimetype("image/webp");
        let r = render_reply(&Reply::sticker(src.clone()), &ReplyTarget::default())
            .unwrap()
            .unwrap();
        let p = r.media.expect("pending media");
        assert_eq!(p.kind, MediaKind::Sticker);
        assert_eq!(p.source, src);
        assert_eq!(p.caption, "", "sticker tidak punya caption");
    }

    #[test]
    fn video_reply_membawa_gif_playback_dan_caption() {
        let reply = Reply::video_gif(MediaSource::url("https://x/v.mp4"), "cap");
        let r = render_reply(&reply, &ReplyTarget::default())
            .unwrap()
            .unwrap();
        assert!(r.needs_media(), "video harus lewat jalur upload");
        let p = r.media.expect("pending media");
        assert_eq!(p.kind, MediaKind::Video);
        assert_eq!(p.source, MediaSource::url("https://x/v.mp4"));
        assert_eq!(p.caption, "cap");
        assert!(p.gif_playback);
        assert!(!p.ptt, "video tidak pernah ptt");

        let biasa = Reply::video(MediaSource::url("https://x/v.mp4"), "");
        let p2 = render_reply(&biasa, &ReplyTarget::default())
            .unwrap()
            .unwrap()
            .media
            .unwrap();
        assert!(!p2.gif_playback);
    }

    #[test]
    fn audio_reply_ptt_hanya_kalau_diminta() {
        let p = render_reply(
            &Reply::voice_note(MediaSource::bytes(vec![1])),
            &ReplyTarget::default(),
        )
        .unwrap()
        .unwrap()
        .media
        .unwrap();
        assert_eq!(p.kind, MediaKind::Audio);
        assert!(p.ptt);
        assert_eq!(p.caption, "", "audio tidak punya caption");

        let p2 = render_reply(
            &Reply::audio(MediaSource::bytes(vec![1])),
            &ReplyTarget::default(),
        )
        .unwrap()
        .unwrap()
        .media
        .unwrap();
        assert!(!p2.ptt);
    }

    #[test]
    fn document_reply_membawa_nama_berkas() {
        let p = render_reply(
            &Reply::document(MediaSource::url("https://x/a.zip"), "bukti.zip"),
            &ReplyTarget::default(),
        )
        .unwrap()
        .unwrap()
        .media
        .unwrap();
        assert_eq!(p.kind, MediaKind::Document);
        assert_eq!(p.file_name.as_deref(), Some("bukti.zip"));
    }

    #[test]
    fn media_baru_tanpa_sumber_jadi_error_bertipe() {
        // Sama seperti Image/Sticker: media: None TIDAK boleh jadi drop senyap.
        for (reply, variant) in [
            (
                Reply::Video {
                    media: None,
                    caption: String::new(),
                    mentions: Vec::new(),
                    gif_playback: false,
                },
                "Video",
            ),
            (
                Reply::Audio {
                    media: None,
                    ptt: false,
                },
                "Audio",
            ),
            (
                Reply::Document {
                    media: None,
                    file_name: "a".into(),
                    caption: String::new(),
                    mentions: Vec::new(),
                },
                "Document",
            ),
        ] {
            let err = render_reply(&reply, &ReplyTarget::default()).unwrap_err();
            assert_eq!(err, PipelineError::MediaSourceMissing { variant });
        }
    }

    /// `PendingMedia` minimal untuk menguji pemetaan opsi tanpa jaringan.
    fn pending_of(kind: MediaKind, source: MediaSource) -> PendingMedia {
        PendingMedia {
            kind,
            source,
            caption: String::new(),
            mentions: Vec::new(),
            quote: None,
            gif_playback: false,
            ptt: false,
            file_name: None,
            jpeg_thumbnail: None,
            file_length: None,
            association: None,
            paired: false,
        }
    }

    #[test]
    fn opsi_video_audio_dokumen_mengikuti_socket_js() {
        let _ = pending_of(MediaKind::Image, MediaSource::url("https://x"));

        // Video: gifPlayback diteruskan; mimetype dari MediaSource kalau ada.
        let mut p = pending_of(
            MediaKind::Video,
            MediaSource::url("https://x").with_mimetype("video/mp4"),
        );
        p.gif_playback = true;
        let v = video_options(&p, Some("cap".into()), None);
        assert_eq!(v.gif_playback, Some(true));
        assert_eq!(v.mimetype.as_deref(), Some("video/mp4"));
        assert_eq!(v.caption.as_deref(), Some("cap"));

        // Audio: ptt diteruskan; mimetype bawaannya `audio/mpeg` seperti
        // `socket.js:798`, bukan default builder (`audio/ogg`).
        let mut p = pending_of(MediaKind::Audio, MediaSource::url("https://x"));
        p.ptt = true;
        let a = audio_options(&p, None);
        assert_eq!(a.ptt, Some(true));
        assert_eq!(a.mimetype.as_deref(), Some("audio/mpeg"));

        // Dokumen: fileName dari plugin; default `"file"` kalau tidak diisi,
        // dan mimetype default `application/octet-stream` (`socket.js:775`).
        let p = pending_of(MediaKind::Document, MediaSource::url("https://x"));
        let d = document_options(&p, None, None);
        assert_eq!(d.file_name.as_deref(), Some("file"));
        assert_eq!(d.mimetype.as_deref(), Some("application/octet-stream"));

        let mut p = pending_of(MediaKind::Document, MediaSource::url("https://x"));
        p.file_name = Some("laporan.pdf".into());
        let d = document_options(&p, None, None);
        assert_eq!(d.file_name.as_deref(), Some("laporan.pdf"));

        // Mimetype dokumen dari pemanggil menang atas default.
        let p = pending_of(
            MediaKind::Document,
            MediaSource::url("https://x").with_mimetype("application/zip"),
        );
        assert_eq!(
            document_options(&p, None, None).mimetype.as_deref(),
            Some("application/zip")
        );
    }

    #[test]
    fn upload_type_media_baru_tidak_tertukar() {
        assert_eq!(MediaKind::Video.upload_type(), MediaType::Video);
        assert_eq!(MediaKind::Audio.upload_type(), MediaType::Audio);
        assert_eq!(MediaKind::Document.upload_type(), MediaType::Document);
        assert_ne!(
            MediaKind::Document.upload_type().upload_path(),
            MediaKind::Image.upload_type().upload_path()
        );
    }

    #[tokio::test]
    async fn every_media_reply_routes_to_send_media_with_its_own_kind() {
        // Yang diuji: pemetaan variant `Reply` → jalur kirim. Media TIDAK
        // boleh jatuh ke `send` biasa.
        for (reply, want_kind, want_label) in [
            (
                Reply::image(MediaSource::bytes(vec![7]), "c"),
                MediaKind::Image,
                "bytes(1)",
            ),
            (
                Reply::sticker(MediaSource::url("https://x/s.webp")),
                MediaKind::Sticker,
                "url",
            ),
            (
                Reply::sticker(MediaSource::path("/tmp/s.webp")),
                MediaKind::Sticker,
                "path",
            ),
            (
                Reply::video(MediaSource::url("https://x/v.mp4"), "c"),
                MediaKind::Video,
                "url",
            ),
            (
                Reply::voice_note(MediaSource::bytes(vec![1, 2])),
                MediaKind::Audio,
                "bytes(2)",
            ),
            (
                Reply::document(MediaSource::url("https://x/a.zip"), "a.zip"),
                MediaKind::Document,
                "url",
            ),
        ] {
            let s = RecordingSender::default();
            let pending = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap().media.unwrap();
            s.send_media(
                "6281@s.whatsapp.net",
                pending.clone(),
                SendOptions::default(),
            )
                .await
                .expect("harus terkirim");
            let sent = s.media.lock().unwrap();
            assert_eq!(sent.len(), 1, "{want_kind:?} harus sekali kirim");
            assert_eq!(sent[0].0, "6281@s.whatsapp.net");
            assert_eq!(sent[0].1.kind, want_kind);
            assert_eq!(sent[0].1.source.label(), want_label);
            // Tidak boleh ikut terkirim lewat jalur teks.
            assert!(s.sent.lock().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn text_reply_still_goes_through_the_plain_send() {
        let s = RecordingSender::default();
        s.send(
            "6281@s.whatsapp.net",
            wa::Message::text("hi"),
            SendOptions::default(),
        )
            .await
            .unwrap();
        assert_eq!(s.sent.lock().unwrap().len(), 1);
        assert!(
            s.media.lock().unwrap().is_empty(),
            "jalur teks tidak boleh menyentuh jalur media"
        );
    }

    #[test]
    fn media_never_carries_a_wire_message_and_wire_never_carries_media() {
        // Invarian yang membuat drop senyap mustahil: kalau `media` terisi,
        // `message` WAJIB kosong (belum ada upload). Kalau `media` kosong,
        // `message` yang dikirim.
        let cases = [
            Reply::None,
            Reply::text("t"),
            Reply::TextQuoted {
                text: "q".into(),
                mentions: vec![],
            },
            Reply::interactive("b"),
            Reply::image(MediaSource::bytes(vec![1]), "c"),
            Reply::Sticker {
                media: Some(MediaSource::bytes(vec![1])),
            },
            Reply::video(MediaSource::bytes(vec![1]), "c"),
            Reply::audio(MediaSource::bytes(vec![1])),
            Reply::document(MediaSource::bytes(vec![1]), "a.txt"),
        ];
        for c in cases {
            let Some(r) = render_reply(&c, &ReplyTarget::default()).unwrap() else {
                continue;
            };
            if r.needs_media() {
                assert_eq!(
                    r.message,
                    wa::Message::default(),
                    "{c:?}: pesan kosong WAJIB sebelum upload"
                );
            } else {
                assert_ne!(
                    r.message,
                    wa::Message::default(),
                    "{c:?}: pesan non-media tidak boleh kosong"
                );
            }
        }
    }

    #[test]
    fn media_kind_maps_to_the_library_upload_type() {
        // `MediaType::Image` dan `MediaType::Sticker` sama-sama ke
        // `/mms/image` (wacore-0.7.0/src/download.rs:91) — persis WA Web.
        assert_eq!(MediaKind::Image.upload_type(), MediaType::Image);
        assert_eq!(MediaKind::Sticker.upload_type(), MediaType::Sticker);
        assert_eq!(
            MediaKind::Image.upload_type().upload_path(),
            MediaKind::Sticker.upload_type().upload_path()
        );
    }

    #[tokio::test]
    async fn bytes_media_resolves_to_those_bytes() {
        let data = load_media(&MediaSource::bytes(vec![9, 8, 7])).await.unwrap();
        assert_eq!(data, vec![9, 8, 7]);
    }

    #[tokio::test]
    async fn path_media_is_read_from_disk() {
        let p = std::env::temp_dir().join(format!(
            "oktz-media-{}.png",
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&p, b"PNG").unwrap();
        let got = load_media(&MediaSource::path(p.to_str().unwrap())).await.unwrap();
        assert_eq!(got, b"PNG".to_vec());
        let _ = std::fs::remove_file(&p);
    }

    #[tokio::test]
    async fn missing_or_empty_media_source_is_an_explicit_error_never_empty_bytes() {
        // Path tidak ada.
        let e = load_media(&MediaSource::path("/tidak/ada/file.png"))
            .await
            .unwrap_err();
        assert!(matches!(e, PipelineError::Media(_)));
        assert!(e.to_string().contains("gagal baca file media"));
        // File ada tapi kosong — `Ok(vec![])` akan lolos ke CDN dan gagal
        // jauh dari penyebabnya.
        let p = std::env::temp_dir().join(format!(
            "oktz-media-empty-{}.png",
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&p, b"").unwrap();
        let e = load_media(&MediaSource::path(p.to_str().unwrap()))
            .await
            .unwrap_err();
        assert!(e.to_string().contains("kosong"), "{e}");
        let _ = std::fs::remove_file(&p);
        // Bytes kosong.
        let e = load_media(&MediaSource::bytes(Vec::new())).await.unwrap_err();
        assert!(e.to_string().contains("kosong"), "{e}");
    }

    #[test]
    fn reply_none_renders_to_nothing() {
        assert!(render_reply(&Reply::None, &ReplyTarget::default()).unwrap().is_none());
    }

    // ---------- 5. Decision -> Reply ----------

    #[test]
    fn non_command_and_no_plugin_are_silent() {
        assert_eq!(decision_to_reply(&Decision::NotACommand), Reply::None);
        assert_eq!(decision_to_reply(&Decision::NoPlugin), Reply::None);
    }

    #[test]
    fn permission_and_group_mode_denials_reply_with_message() {
        assert_eq!(
            decision_to_reply(&Decision::DeniedPermission("x".into())),
            Reply::text("x")
        );
        assert_eq!(
            decision_to_reply(&Decision::DeniedGroupMode("y".into())),
            Reply::text("y")
        );
    }

    #[test]
    fn silent_modes_send_nothing_but_afk_and_only_this_group_reply() {
        assert_eq!(
            decision_to_reply(&Decision::DeniedMode(DenyMode::Silent)),
            Reply::None
        );
        assert_eq!(
            decision_to_reply(&Decision::DeniedMode(DenyMode::SelfGroup)),
            Reply::None
        );
        for m in [
            DenyMode::Afk {
                reason: "libur".into(),
                elapsed_ms: 3_600_000,
            },
            DenyMode::OnlyThisGroup {
                name: "Grup Utama".into(),
                link: "https://chat.whatsapp.com/abc".into(),
            },
        ] {
            let label = format!("{m:?}");
            let r = decision_to_reply(&Decision::DeniedMode(m.clone()));
            assert!(!r.is_none(), "{label} harus membalas");
            assert!(r.text_body().is_some_and(|t| !t.is_empty()));
        }
    }

    #[test]
    fn afk_text_carries_the_reason_and_the_duration() {
        // `middleware.js:169` menyisipkan `botAfk.reason`; tanpa itu teksnya
        // bukan kalimat yang sama dengan produksi.
        let t = afk_text("libur", 3_600_000);
        assert!(t.contains("libur"));
        assert!(t.contains("1 jam 0 menit"));
    }

    #[test]
    fn only_this_group_text_carries_name_and_link() {
        let t = only_this_group_text("Grup Utama", "https://chat.whatsapp.com/abc");
        assert!(t.contains("Grup Utama"));
        assert!(t.contains("https://chat.whatsapp.com/abc"));
    }

    #[test]
    fn handled_passes_reply_through_untouched() {
        let original = Reply::text("pong");
        assert_eq!(
            decision_to_reply(&Decision::Handled(original.clone())),
            original
        );
    }

    // ---------- 6. body extraction ----------

    #[test]
    fn conversation_body_is_extracted() {
        assert_eq!(extract_body(&wa::Message::text("halo")), Some("halo".into()));
    }

    #[test]
    fn extended_text_body_is_extracted() {
        let m = wa::Message::text_with_context("hai", wa::ContextInfo::default());
        assert_eq!(extract_body(&m), Some("hai".into()));
    }

    #[test]
    fn message_without_text_has_no_body() {
        assert_eq!(extract_body(&wa::Message::default()), None);
    }

    #[test]
    fn empty_conversation_is_treated_as_no_body() {
        let m = wa::Message {
            conversation: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(extract_body(&m), None);
    }

    // ---------- kartu interaktif ----------

    /// Kartu tanpa media header bisa langsung jadi pesan: `Rendered.message`
    /// terisi dan node `biz` ikut.
    #[test]
    fn kartu_tanpa_media_header_langsung_jadi_pesan() {
        let r = render_reply(
            &Reply::interactive("pilih").with_button(InteractiveButton::quick_reply("Ya", ".ya")),
            &ReplyTarget::default(),
        )
        .unwrap()
        .unwrap();
        assert!(r.card.is_none(), "tanpa media tidak perlu jalur upload");
        assert!(r.message.view_once_message.is_set());
        assert_eq!(r.extra_nodes.len(), 1, "node biz wajib untuk native-flow");
    }

    /// Kartu dengan gambar header harus lewat jalur upload: pesannya belum
    /// boleh dikirim apa adanya.
    #[test]
    fn kartu_dengan_media_header_ditahan_sampai_upload() {
        let reply = Reply::interactive("pilih").with_header(
            oktz_plugin_api::registry::CardHeader::new()
                .image(MediaSource::path("./assets/image/foto.jpeg")),
        );
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        assert!(r.needs_media(), "harus lewat jalur upload");
        let card = r.card.expect("kartu ditahan");
        assert!(card.needs_upload());
        assert!(r.message.view_once_message.is_unset(), "belum boleh dikirim");
    }

    /// Header lokasi tidak butuh upload — kartu langsung jadi pesan.
    #[test]
    fn kartu_dengan_header_lokasi_tidak_butuh_upload() {
        let reply = Reply::interactive("pilih").with_header(
            oktz_plugin_api::registry::CardHeader::new().location(
                oktz_plugin_api::registry::CardLocation {
                    latitude: 0.0,
                    longitude: 0.0,
                    name: "Oktz Bot".into(),
                    address: Some("Cuaca tidak tersedia".into()),
                    thumbnail: None,
                },
            ),
        );
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        assert!(r.card.is_none());
        assert!(r.message.view_once_message.is_set());
    }

    /// `messageParamsJson` harus sampai ke native flow.
    #[test]
    fn message_params_dan_context_sampai_ke_kartu() {
        let reply = Reply::interactive("pilih")
            .with_message_params(json!({"bottom_sheet": {"list_title": "Pilih"}}))
            .with_context(
                oktz_plugin_api::registry::CardContext::new()
                    .mention("6281@s.whatsapp.net")
                    .forwarded(9),
            );
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        let inner = r
            .message
            .view_once_message
            .as_option()
            .unwrap()
            .message
            .as_option()
            .unwrap();
        let im = inner.interactive_message.as_option().unwrap();
        let ImOneof::NativeFlowMessage(nf) = im.interactive_message.as_ref().unwrap() else {
            panic!("oneof bukan NativeFlowMessage");
        };
        let params: serde_json::Value =
            serde_json::from_str(nf.message_params_json.as_deref().unwrap()).unwrap();
        assert_eq!(params["bottom_sheet"]["list_title"], "Pilih");
        let ci = im.context_info.as_option().expect("contextInfo");
        assert_eq!(ci.mentioned_jid, vec!["6281@s.whatsapp.net".to_string()]);
        assert_eq!(ci.forwarding_score, Some(9));
    }

    /// `quote_trigger` = `quoted: m` di JS — kartu mengutip pesan pemicu.
    #[test]
    fn kartu_bisa_mengutip_pesan_pemicu() {
        let target = ReplyTarget {
            quote: Some(QuoteSource {
                stanza_id: "STANZA1".into(),
                participant: Some("6281@s.whatsapp.net".into()),
                message: Box::new(wa::Message::text("pesan asal")),
            }),
            sender: "6289@s.whatsapp.net".into(),
        };
        let reply = Reply::interactive("pilih")
            .with_context(oktz_plugin_api::registry::CardContext::new().quote_trigger());
        let r = render_reply(&reply, &target).unwrap().unwrap();
        let inner = r
            .message
            .view_once_message
            .as_option()
            .unwrap()
            .message
            .as_option()
            .unwrap();
        let im = inner.interactive_message.as_option().unwrap();
        let ci = im.context_info.as_option().expect("contextInfo");
        let quoted = ci.quoted_message.as_option().expect("quotedMessage");
        assert_eq!(quoted.conversation.as_deref(), Some("pesan asal"));
        assert_eq!(ci.stanza_id.as_deref(), Some("STANZA1"));
    }

    /// Tabel WhatsApp = `botForwardedMessage > richResponseMessage`.
    #[test]
    fn tabel_dirender_sebagai_rich_response_message() {
        let reply = Reply::table(
            "Distribusi Fitur",
            ["Kategori", "Jumlah"],
            [vec!["🏠 MAIN".to_string(), "10".to_string()]],
        )
        .with_header_text("Total: 10")
        .with_table_footer("Total 10 fitur");
        let r = render_reply(&reply, &ReplyTarget::default()).unwrap().unwrap();
        let rich = r
            .message
            .bot_forwarded_message
            .as_option()
            .unwrap()
            .message
            .as_option()
            .unwrap()
            .rich_response_message
            .as_option()
            .expect("richResponseMessage");
        assert_eq!(rich.submessages.len(), 3);
        let table = rich.submessages[1].table_metadata.as_option().unwrap();
        assert_eq!(table.title.as_deref(), Some("Distribusi Fitur"));
        assert_eq!(table.rows.len(), 2);
    }

    /// Video yang menempel (`PairedVideo`) memakai jalur media seperti
    /// video biasa, dan ditandai sebagai media berpasangan.
    #[test]
    fn paired_video_menggunakan_jalur_media() {
        let r = render_reply(
            &Reply::paired_video(MediaSource::path("./assets/video/mp4.mp4"), ""),
            &ReplyTarget::default(),
        )
        .unwrap()
        .unwrap();
        assert!(r.needs_media());
        let m = r.media.unwrap();
        assert_eq!(m.kind, MediaKind::Video);
        assert!(m.paired, "harus ditandai berpasangan");
    }

    #[test]
    fn interactive_response_yields_selected_id_from_params_json() {        let m = wa::Message {
            interactive_response_message: MessageField::some(wa::message::InteractiveResponseMessage {
                interactive_response_message: Some(IrOneof::NativeFlowResponseMessage(Box::new(
                    wa::message::interactive_response_message::NativeFlowResponseMessage {
                        params_json: Some(r#"{"id":"btn_1"}"#.into()),
                        ..Default::default()
                    },
                ))),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(extract_body(&m), Some("btn_1".into()));
    }

    // ---------- 7. pipeline end-to-end (tanpa koneksi WA) ----------

    #[tokio::test]
    async fn non_command_is_ignored_and_nothing_sent() {
        let (p, s) = pipeline_with("{}", Identity::default());
        let m = inbound("halo dunia", "6281@s.whatsapp.net", "6281@s.whatsapp.net", false, "M1");
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::NotACommand);
        assert!(s.sent.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn message_without_body_is_ignored() {
        let (p, s) = pipeline_with("{}", Identity::default());
        let m = InboundMessage::builder()
            .message(Arc::new(wa::Message::default()))
            .info(
                inbound("", "6281@s.whatsapp.net", "6281@s.whatsapp.net", false, "M1").info,
            )
            .build();
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::NoBody);
        assert!(s.sent.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn duplicate_message_id_is_deduped_and_sent_once() {
        let (p, s) = pipeline_with("{}", Identity::default());
        let m = inbound(".ping", "6281@s.whatsapp.net", "6281@s.whatsapp.net", false, "SAME");
        let o1 = p.process(&m, 1_000).await.unwrap();
        let o2 = p.process(&m, 1_010).await.unwrap();
        assert_eq!(o2, Outcome::Deduped);
        assert_ne!(o1, Outcome::Deduped);
        // Tanpa plugin terdaftar di oktz-wa tidak ada yang terkirim; yang
        // dibuktikan di sini adalah gate dedup, bukan dispatch plugin.
        assert!(s.sent.lock().unwrap().len() <= 1);
    }

    #[tokio::test]
    async fn afk_mode_replies_to_a_command() {
        let store = r#"{"settings": {"botAfk": {"active": true, "since": 1}}}"#;
        let (p, s) = pipeline_with(store, Identity::default());
        let m = inbound(".ping", "6281@s.whatsapp.net", "6281@s.whatsapp.net", false, "A1");
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Denied);
        let sent = s.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "6281@s.whatsapp.net");
        assert!(body_of(&sent[0].1).unwrap().contains("sᴇᴅᴀɴɢ ᴀꜰᴋ"));
    }

    #[tokio::test]
    async fn owner_is_exempt_from_afk() {
        let store = r#"{"settings": {"botAfk": {"active": true, "since": 1}}}"#;
        let mut owners = HashSet::new();
        owners.insert("6281".to_string());
        let (p, s) = pipeline_with(store, Identity { owners, ..Default::default() });
        let m = inbound(".ping", "6281@s.whatsapp.net", "6281@s.whatsapp.net", false, "A2");
        // Owner lolos mode; lalu `find` tidak punya plugin -> NoPlugin -> diam.
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::NotACommand);
        assert!(s.sent.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn owner_matched_by_full_jid_or_number() {
        let mut owners = HashSet::new();
        owners.insert("6281@s.whatsapp.net".to_string());
        let id = Identity { owners, ..Default::default() };
        assert!(id.is_owner("6281@s.whatsapp.net"));
        let mut owners = HashSet::new();
        owners.insert("6281".to_string());
        let id = Identity { owners, ..Default::default() };
        assert!(id.is_owner("6281:9@s.whatsapp.net"), "cocok via nomor");
        assert!(!Identity::default().is_owner("6281@s.whatsapp.net"));
    }

    #[tokio::test]
    async fn spam_gate_blocks_repeated_commands() {
        // Limiter global: 8 titik / 3 detik (gates.rs:64-66).
        let (p, s) = pipeline_with("{}", Identity::default());
        let mut blocked = 0;
        for i in 0..12 {
            let m = inbound(
                ".ping",
                "6281@s.whatsapp.net",
                "6281@s.whatsapp.net",
                false,
                &format!("SP{i}"),
            );
            if p.process(&m, 2_000).await.unwrap() == Outcome::SpamBlocked {
                blocked += 1;
            }
        }
        assert!(blocked > 0, "limiter harus menahan setelah 8 perintah");
        drop(s);
    }

    #[tokio::test]
    async fn auto_work_gate_is_not_charged_by_ordinary_commands() {
        // Dedup memakai id berbeda, jadi semua perintah benar-benar lewat gate.
        let (p, _s) = pipeline_with("{}", Identity::default());
        let caller = Caller {
            sender: "6281@s.whatsapp.net".into(),
            chat: "6281@s.whatsapp.net".into(),
            ..Default::default()
        };
        for i in 0..5 {
            let m = inbound(
                ".ping",
                "6281@s.whatsapp.net",
                "6281@s.whatsapp.net",
                false,
                &format!("W{i}"),
            );
            p.process(&m, 2_000).await.unwrap();
        }
        // Kuota 10/menit; 5 perintah biasa tidak boleh memakainya -> saldo penuh.
        // `W{i}` yang berbeda dari id pesan di atas, jadi guard-nya belum diputus.
        for i in 0..10 {
            let d = p.allow_auto_work(&format!("probe{i}"), &caller, "auto", Some(999_999), 3_000);
            assert!(d.allowed, "kuota tak boleh terpakai: #{i}");
        }
    }

    #[tokio::test]
    async fn auto_work_guard_charges_once_per_message_not_once_per_caller() {
        // `createAutoWorkGuard` (handler.js:416-422): auto-AI, CAI chat,
        // auto-download, auto-join, transkripsi, dan smart trigger semuanya
        // panggil `allowAutoWork()` pada pesan yang sama. Tanpa memoization
        // satu pesan bisa membakar kuota berkali-kali.
        let (p, _s) = pipeline_with("{}", Identity::default());
        let caller = Caller {
            sender: "6281@s.whatsapp.net".into(),
            chat: "6281@s.whatsapp.net".into(),
            ..Default::default()
        };
        // Satu pesan, enam fitur auto-work — seperti di JS.
        let mut hits = 0;
        for _ in 0..6 {
            let d = p.allow_auto_work("MSG-1", &caller, "auto", Some(999_999), 1_000);
            if d.allowed {
                hits += 1;
            }
            assert_eq!(d.deduct, 1, "potongan selalu sama, di-cache atau bukan");
        }
        assert_eq!(hits, 6, "semua caller melihat keputusan yang sama");

        // Bukti bahwa kuota benar-benar dibakar SATU kali, bukan enam: kuota
        // 10/menit, MSG-1 sudah memakai titik ke-1, jadi 9 pesan berikutnya
        // masih muat (titik 2..10).
        for i in 0..9 {
            let d = p.allow_auto_work(&format!("next{i}"), &caller, "auto", Some(999_999), 1_000);
            assert!(d.allowed, "pesan #{i} seharusnya masih punya kuota");
        }
        // Titik ke-11 habis. Kalau MSG-1 sempat membakar 6, titik ke-11 ini
        // masih muat — jadi test ini memang menangkap peng Calculating ganda.
        assert!(
            !p.allow_auto_work("last", &caller, "auto", Some(999_999), 1_000).allowed,
            "kuota harus habis setelah 10 titik, jadi MSG-1 membakar tepat 1"
        );
    }

    #[tokio::test]
    async fn group_reply_is_addressed_to_the_group_chat() {
        let (p, s) = pipeline_with(
            r#"{"settings": {"botAfk": {"active": true, "since": 1}}}"#,
            Identity::default(),
        );
        let m = inbound(".ping", "6281@s.whatsapp.net", "120363@g.us", true, "G1");
        p.process(&m, 1_000).await.unwrap();
        let sent = s.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "120363@g.us", "tujuan = chat, bukan sender");
    }

    #[test]
    fn store_map_with_unexpected_shape_does_not_panic() {
        //(users bukan object) — harus tetap aman, bukan panic.
        let s = store_with("weird", r#"{"users": [], "groups": 5, "settings": "x"}"#);
        let ctx = StoreContext::new(s);
        assert!(ctx.user_state("6281@s.whatsapp.net").access.is_empty());
        assert_eq!(ctx.group_state("120363@g.us"), None);
        assert_eq!(ctx.group_bot_mode("120363@g.us"), "all");
        assert!(ctx.mode_settings().self_groups.is_empty());
        assert_eq!(ctx.capprem("x"), None);
        let _ = json!({});
    }

    // ---------- 8. jembatan bus event → pipeline (tanpa Client) ----------

    /// Bungkus beberapa pesan jadi satu `Event::Messages` persis seperti yang
    /// dikirim library (`events.rs:858`). `MessageBatch` `#[non_exhaustive]`
    /// (`events.rs:1157`) jadi harus lewat builder.
    fn messages_event(ids: &[&str]) -> Arc<Event> {
        let messages: Arc<[InboundMessage]> = Arc::from(
            ids.iter()
                .map(|id| {
                    inbound(
                        ".ping",
                        "6281@s.whatsapp.net",
                        "6281@s.whatsapp.net",
                        false,
                        id,
                    )
                })
                .collect::<Vec<_>>(),
        );
        Arc::new(Event::Messages(
            whatsapp_rust::types::events::MessageBatch::builder()
                .messages(messages)
                .origin(whatsapp_rust::types::events::BatchOrigin::Live)
                .build(),
        ))
    }

    #[test]
    fn handler_enqueues_every_message_in_a_batch() {
        let (tx, rx) = whatsapp_rust::async_channel::unbounded();
        let h = handler_pesan(tx);
        h.handle_event(messages_event(&["B1", "B2", "B3"]));
        let ids: Vec<String> = std::iter::from_fn(|| rx.try_recv().ok())
            .map(|m| m.info.id.clone())
            .collect();
        assert_eq!(ids, vec!["B1", "B2", "B3"]);
    }

    #[test]
    fn handler_ignores_non_message_events() {
        let (tx, rx) = whatsapp_rust::async_channel::unbounded();
        let h = handler_pesan(tx);
        h.handle_event(Arc::new(Event::Connected(
            whatsapp_rust::types::events::Connected::builder().build(),
        )));
        assert!(rx.try_recv().is_err(), " Connected bukan pesan");
    }

    #[test]
    fn handler_interest_is_messages_and_group_updates() {
        let (tx, _rx) = whatsapp_rust::async_channel::unbounded();
        let h = handler_pesan(tx);
        assert_eq!(
            h.interest(),
            EventInterest::of(&[EventKind::Messages, EventKind::GroupUpdate])
        );
    }

    #[test]
    fn full_queue_is_reported_and_never_blocks() {
        let (tx, rx) = whatsapp_rust::async_channel::bounded(2);
        let h = handler_pesan(tx);
        // Kapasitas 2, Offering 4: `try_send` HARUS balik tanpa menunggu dan
        // 2 sisanya tercatat sebagai overflow, bukan hilang tanpa jejak.
        for e in [
            messages_event(&["Q1"]),
            messages_event(&["Q2"]),
            messages_event(&["Q3"]),
            messages_event(&["Q4"]),
        ] {
            h.handle_event(e);
        }
        let mut got = Vec::new();
        while let Ok(m) = rx.try_recv() {
            got.push(m.info.id.clone());
        }
        assert_eq!(got, vec!["Q1", "Q2"], "sisanya harus dibuang, bukan diantrekan");
    }

    #[tokio::test]
    async fn run_processes_each_queued_message_end_to_end() {
        // Jembatan penuh: event bus → antrean → run → pipeline → sender palsu.
        // AFK aktif supaya `Decision::DeniedMode(Afk)` jadi balasan yang benar-
        // benar terkirim, jadi pemetaan Decision→Reply→wire ikut teruji.
        let store = r#"{"settings": {"botAfk": {"active": true, "since": 1}}}"#;
        let ctx = StoreContext::new(store_with("bridge", store));
        let sender = Arc::new(RecordingSender::default());
        let p = Arc::new(Pipeline::new(
            ctx,
            sender.clone(),
            "bot",
            ParseOptions::default(),
        ));

        let (tx, rx) = whatsapp_rust::async_channel::unbounded();
        let handle = handler_pesan(tx);
        tokio::spawn(p.clone().run(rx));

        handle.handle_event(messages_event(&["E1", "E2"]));

        // Tunggu sampai kedua pesan selesai diproses.
        let sent = wait_for_len(&sender, 2).await;
        assert_eq!(sent.len(), 2, "satu balasan per pesan inbound");
        for (to, m, extra) in &sent {
            assert_eq!(to, "6281@s.whatsapp.net");
            assert_eq!(extra, &0, "teks balasan tidak butuh node tambahan");
            assert!(body_of(m).unwrap().contains("sᴇᴅᴀɴɢ ᴀꜰᴋ"));
        }
    }

    #[derive(Clone, Default)]
    struct FailingSender {
        sends: Arc<Mutex<usize>>,
    }

    impl MessageSender for FailingSender {
        fn send(&self, _to: &str, _m: wa::Message, _opts: SendOptions) -> SendBox<'_> {
            *self.sends.lock().unwrap() += 1;
            Box::pin(async { Err("socket putus".into()) })
        }
    }

    #[tokio::test]
    async fn send_failure_does_not_kill_the_loop() {
        // Kegagalan kirim harus jadi Err dari `process` → di-log di `run` →
        // loop lanjut ke pesan berikutnya. Kalau `run` ikut mati, bot berhenti
        // diam-diam setelah satu error jaringan.
        let store = r#"{"settings": {"botAfk": {"active": true, "since": 1}}}"#;
        let ctx = StoreContext::new(store_with("failbridge", store));
        let sender = Arc::new(FailingSender::default());
        let p = Arc::new(Pipeline::new(
            ctx,
            sender.clone(),
            "bot",
            ParseOptions::default(),
        ));

        let (tx, rx) = whatsapp_rust::async_channel::unbounded();
        let handler = handler_pesan(tx);
        handler.handle_event(messages_event(&["F1", "F2", "F3"]));
        let runner = tokio::spawn(p.run(rx));

        // Tunggu sampai ketiga pesan dicoba (semua gagal kirim).
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        while *sender.sends.lock().unwrap() < 3 {
            assert!(
                tokio::time::Instant::now() < deadline,
                "loop berhenti setelah {} percobaan kirim",
                sender.sends.lock().unwrap()
            );
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(
            !runner.is_finished(),
            "loop harus tetap hidup setelah error kirim"
        );

        // Menutup antrean menghentikan loop — itu jalan keluarnya yang bersih.
        drop(handler);
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), runner).await;
    }

    #[tokio::test]
    async fn a_media_reply_without_a_source_never_reports_success() {
        // Kegagalan harus sampai ke pemanggil sebagai error bertipe, bukan
        // panik dan bukan "sukses diam" yang membuat caller menganggap pesan
        // terkirim. Yang diuji: `Reply` tanpa sumber media tidak pernah jadi
        // `Ok` — tidak ada jalur yang bisa menjatuhkannya.
        let ctx = StoreContext::new(store_with("media", "{}"));
        let sender = RecordingSender::default();
        let _p = Pipeline::new(
            ctx,
            Arc::new(sender.clone()),
            "bot",
            ParseOptions::default(),
        );
        for reply in [
            Reply::Image {
                media: None,
                caption: "c".into(),
                mentions: vec![],
            },
            Reply::Sticker { media: None },
        ] {
            let err = render_reply(&reply, &ReplyTarget::default()).unwrap_err();
            assert!(matches!(err, PipelineError::MediaSourceMissing { .. }));
            assert!(err.to_string().contains("sumber media"));
        }
        assert!(sender.sent.lock().unwrap().is_empty());
        assert!(
            sender.media.lock().unwrap().is_empty(),
            "tanpa sumber, tidak boleh ada yang dikirim"
        );
    }

    // ---------- 9. Ctx: roster grup + state store ----------

    /// Bentuk roster uji: satu JID per peserta, tanpa pasangan LID/PN.
    fn snapshot_of(roster: GroupRoster) -> RosterSnapshot {
        let participants = roster
            .participants
            .iter()
            .map(|p| oktz_router::group::Participant {
                id: p.jid.clone(),
                lid: String::new(),
                phone_number: p.jid.clone(),
                admin: p.is_admin,
            })
            .collect();
        RosterSnapshot {
            roster,
            participants,
            available: true,
        }
    }

    fn real_group_metadata() -> GroupMetadata {
        // Bentuk persis `GroupInfoResponse` yang di-`GroupMetadata::from`
        // (`features/groups.rs:268`): `participant_type` tri-state.
        let p = |jid: &str, t: ParticipantType| GroupParticipant {
            jid: jid.parse().unwrap(),
            phone_number: None,
            lid: None,
            username: None,
            participant_type: t,
            details: None,
        };
        GroupMetadata {
            participants: vec![
                p("111@s.whatsapp.net", ParticipantType::SuperAdmin),
                p("222@s.whatsapp.net", ParticipantType::Admin),
                p("333@s.whatsapp.net", ParticipantType::Member),
                p("444@lid", ParticipantType::Member),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn roster_is_built_from_the_real_group_metadata_shape() {
        let roster = roster_from(&real_group_metadata(), &[]);
        assert_eq!(roster.participants.len(), 4);
        assert_eq!(
            roster.member_jids(),
            vec![
                "111@s.whatsapp.net",
                "222@s.whatsapp.net",
                "333@s.whatsapp.net",
                "444@lid",
            ]
        );
        // Superadmin = owner DAN admin; admin biasa hanya admin; member tidak
        // keduanya. Persis `p.admin === 'superadmin' | 'admin'` di JS.
        assert_eq!(roster.owners().count(), 1);
        assert_eq!(roster.admins().count(), 2);
        assert!(roster.is_admin("222@s.whatsapp.net"));
        assert!(!roster.is_admin("333@s.whatsapp.net"));
        assert!(!roster.is_bot_admin, "bot tidak ada di daftar -> bukan admin");
    }

    #[test]
    fn bot_admin_is_detected_against_the_roster_itself() {
        // JID bot tidak pernah datang di `MessageInfo`, jadi satu-satunya
        // sumber kebenaran adalah daftar peserta.
        let meta = real_group_metadata();
        // Admin biasa dan superadmin sama-sama bot-admin.
        assert!(roster_from(&meta, &["222@s.whatsapp.net".into()]).is_bot_admin);
        assert!(roster_from(&meta, &["111@s.whatsapp.net".into()]).is_bot_admin);
        // Member biasa bukan admin, LID pun tidak: yang menentukan adalah
        // `participant_type`, bukan bentuk JID.
        assert!(!roster_from(&meta, &["333@s.whatsapp.net".into()]).is_bot_admin);
        assert!(!roster_from(&meta, &["444@lid".into()]).is_bot_admin);
        // Bot tidak ada di daftar.
        assert!(!roster_from(&meta, &["999@s.whatsapp.net".into()]).is_bot_admin);
        assert!(!roster_from(&meta, &[]).is_bot_admin);
        // JID bot dalam format device-suffiks tidak boleh dianggap cocok
        // dengan JID wire yang bersih.
        assert!(!roster_from(&meta, &["222:3@s.whatsapp.net".into()]).is_bot_admin);
    }

    #[tokio::test]
    async fn without_a_roster_source_group_messages_still_get_processed() {
        // `NoRoster` = "tidak tahu", bukan "grup kosong" dan bukan error:
        // pesan harus tetap sampai ke plugin.
        assert!(NoRoster.roster("120363@g.us").await.unwrap().roster.is_empty());
    }

    #[test]
    fn stored_state_is_read_from_the_same_maps_the_gates_use() {
        let ctx = StoreContext::new(store_with("ctxstate", OK_STORE));
        let u = ctx.user_entry("628111@s.whatsapp.net");
        assert_eq!(u["isRegistered"], json!(true));
        assert_eq!(u["access"].as_array().unwrap().len(), 2);
        let g = ctx.group_entry("120363@g.us");
        assert_eq!(g["rpg"], json!(true));
        assert_eq!(g["botMode"], json!("store"));
    }

    #[test]
    fn missing_user_or_group_is_null_not_a_default_object() {
        let ctx = StoreContext::new(store_with("ctxmissing", OK_STORE));
        assert_eq!(ctx.user_entry("999@s.whatsapp.net"), Value::Null);
        assert_eq!(ctx.group_entry("999@g.us"), Value::Null);
        // JID yang ditolak `normalize_user_key` -> `null`, bukan panik.
        assert_eq!(ctx.user_entry("120363001234567890@g.us"), Value::Null);
        assert_eq!(ctx.user_entry(""), Value::Null);
    }

    #[test]
    fn store_map_with_odd_shape_does_not_panic_when_building_ctx_state() {
        let ctx = StoreContext::new(store_with("ctxweird", r#"{"users": 5, "groups": []}"#));
        assert_eq!(ctx.user_entry("628111@s.whatsapp.net"), Value::Null);
        assert_eq!(ctx.group_entry("120363@g.us"), Value::Null);
    }

    fn enriched_entry(seen: Arc<Mutex<Option<Ctx>>>) -> &'static PluginEntry {
        Box::leak(Box::new(PluginEntry {
            meta: oktz_plugin_api::meta::PluginMeta {
                name: "echo".into(),
                ..Default::default()
            },
            handler: Arc::new(move |c: Ctx| {
                *seen.lock().unwrap() = Some(c);
                Reply::None
            }),
        }))
    }

    fn plugin_src_with(store: &str) -> PluginCtxSource {
        PluginCtxSource {
            store: Arc::new(StoreContext::new(store_with("wrapsrc", store))),
            rosters: Arc::new(Mutex::new(RosterCache {
                map: HashMap::new(),
                ttl: RosterCache::DEFAULT_TTL,
            })),
            sessions: new_session_store(),
            http: Arc::new(NoHttp),
            config: Arc::new(oktz_core::config::Runtime::default()),
            sock: Arc::new(SockMati),
            berkas: Arc::new(oktz_plugin_api::berkas::BerkasPalsu::baru()),
        }
    }

    #[test]
    fn wrapped_handler_receives_roster_and_state_without_losing_router_fields() {
        // Yang disuntik pipeline: roster + state. Yang dihitung router:
        // text/args/sender/prefix. Keduanya harus sampai ke plugin.
        let seen = Arc::new(Mutex::new(None));
        let real = enriched_entry(seen.clone());
        let roster = GroupRoster {
            participants: vec![Participant {
                jid: "111@s.whatsapp.net".into(),
                is_admin: true,
                is_owner: true,
                data: Default::default(),
            }],
            is_bot_admin: true,
        };
        let mut cache = WrappedHandlers::default();
        let src = plugin_src_with(OK_STORE);
        src.rosters
            .lock()
            .unwrap()
            .put("120363@g.us", snapshot_of(roster.clone()));
        let wrapped = cache.get_or_insert(real, &src);

        // `Ctx` seperti yang router buat sebelum disuntik.
        let router_ctx = Ctx {
            text: "satu dua".into(),
            args: vec!["satu".into(), "dua".into()],
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            prefix: ".".into(),
            command: "echo".into(),
            is_group: true,
            uptime_secs: 7,
            ..Default::default()
        };
        (wrapped.handler)(router_ctx);

        let c = seen.lock().unwrap().clone().expect("handler harus jalan");
        // Field router utuh.
        assert_eq!(c.text, "satu dua");
        assert_eq!(c.args, vec!["satu", "dua"]);
        assert_eq!(c.sender, "628111@s.whatsapp.net");
        assert_eq!(c.prefix, ".");
        assert_eq!(c.uptime_secs, 7);
        // Field yang disuntik terisi.
        assert!(c.roster.is_bot_admin);
        assert_eq!(c.roster.owners().count(), 1);
        assert_eq!(
            c.user_field("isRegistered").and_then(|v| v.as_bool()),
            Some(true),
            "state user harus terbaca dari store"
        );
        assert_eq!(c.group_field("rpg").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            c.user_list("access").len(),
            2,
            "list dari store harus bisa dibaca plugin"
        );
    }

    #[test]
    fn nested_state_path_matches_what_rpg_plugins_read() {
        // Plugin rpg membaca `user.rpg.stamina` / `user.rpg.bank`
        // (`plugins/rpg/bank.js:27-34`). Jalur bertingkat harus bisa dibaca
        // tanpa plugin tahu bentuk JSON-nya.
        //
        // Kunci store adalah hasil `normalize_user_key`, bukan JID mentah —
        // sama seperti `getUser` (`database.js:529-536`). Plugin cukup
        // baca `ctx.user_data`, tidak perlu tahu aturan normalisasi.
        let src = plugin_src_with(
            r#"{"users": {"628111": {"koin": 5, "rpg": {"stamina": 30, "bank": 120}}}}"#,
        );
        let mut c = Ctx {
            sender: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        src.fill(&mut c);
        assert_eq!(c.user_field("rpg.stamina").and_then(|v| v.as_i64()), Some(30));
        assert_eq!(c.user_field("rpg.bank").and_then(|v| v.as_i64()), Some(120));
        assert_eq!(c.user_field("koin").and_then(|v| v.as_i64()), Some(5));
        assert_eq!(c.user_field("rpg.hp"), None);
        // Versi dengan sufiks device menunjuk ke user yang sama.
        let mut c2 = Ctx {
            sender: "628111:4@s.whatsapp.net".into(),
            ..Default::default()
        };
        src.fill(&mut c2);
        assert_eq!(
            c2.user_field("koin").and_then(|v| v.as_i64()),
            None,
            "nomor perangkat = kunci berbeda, seperti di JS"
        );
    }

    #[test]
    fn wrapper_is_reused_but_never_pins_one_messages_state() {
        // Regresi: handler dibungkus SEKALI per plugin, jadi kalau isinya
        // capturing `Enrich` pesan pertama, semua pesan berikutnya melihat
        // state pesan pertama. Isi wrapper harus dibaca dari sumber, bukan
        // dari capture.
        let seen = Arc::new(Mutex::new(Vec::<Ctx>::new()));
        let real = Box::leak(Box::new(PluginEntry {
            meta: oktz_plugin_api::meta::PluginMeta {
                name: "echo".into(),
                ..Default::default()
            },
            handler: Arc::new({
                let seen = seen.clone();
                move |c: Ctx| {
                    seen.lock().unwrap().push(c);
                    Reply::None
                }
            }),
        }));
        let src = plugin_src_with(OK_STORE);
        let mut cache = WrappedHandlers::default();

        let first = cache.get_or_insert(real, &src);
        let second = cache.get_or_insert(real, &src);
        assert!(
            std::ptr::eq(first, second),
            "entri yang sama harus dipakai ulang (tidak boleh leak per pesan)"
        );

        // Pesan dari user yang ADA di store.
        (second.handler)(Ctx {
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Default::default()
        });
        // Pesan berikutnya dari user yang TIDAK ada di store.
        (second.handler)(Ctx {
            sender: "628999@s.whatsapp.net".into(),
            chat: "628999@s.whatsapp.net".into(),
            ..Default::default()
        });

        let got = seen.lock().unwrap().clone();
        assert_eq!(got.len(), 2);
        assert_eq!(
            got[0].user_field("isRegistered").and_then(|v| v.as_bool()),
            Some(true),
            "pesan pertama: user ada"
        );
        assert_eq!(
            got[1].user_field("isRegistered").and_then(|v| v.as_bool()),
            None,
            "pesan kedua harus melihat store SAAT INI, bukan pesan pertama"
        );
        assert!(got[1].roster.is_empty());
    }

    #[test]
    fn unknown_chat_yields_an_empty_roster_not_a_stale_one() {
        let src = plugin_src_with(OK_STORE);
        src.rosters.lock().unwrap().put("120363@g.us", snapshot_of(roster_from(
            &real_group_metadata(),
            &["222@s.whatsapp.net".into()],
        )));
        let mut c = Ctx {
            chat: "999@g.us".into(),
            ..Default::default()
        };
        src.fill(&mut c);
        assert!(
            c.roster.is_empty(),
            "chat lain tidak boleh mewarisi roster chat sebelumnya"
        );
        assert_eq!(c.group_field("rpg"), None);

        let mut c = Ctx {
            chat: "120363@g.us".into(),
            ..Default::default()
        };
        src.fill(&mut c);
        assert_eq!(c.roster.participants.len(), 4);
        assert!(c.roster.is_bot_admin);
    }

    #[test]
    fn roster_cache_expires_and_a_failed_read_is_not_cached() {
        let mut c = RosterCache {
            map: HashMap::new(),
            ttl: RosterCache::DEFAULT_TTL,
        };
        let r = GroupRoster {
            participants: vec![Participant {
                jid: "a".into(),
                is_admin: false,
                is_owner: false,
                data: Default::default(),
            }],
            is_bot_admin: false,
        };
        c.put("g", snapshot_of(r.clone()));
        assert_eq!(c.get("g"), Some(snapshot_of(r)));
        assert_eq!(c.get("lain"), None);
        // TTL habis -> dianggap tidak ada, supaya di-refresh.
        c.map.get_mut("g").unwrap().0 = Instant::now() - RosterCache::DEFAULT_TTL * 2;
        assert_eq!(c.get("g"), None, "roster basi tidak boleh dipakai selamanya");
    }

    #[tokio::test]
    async fn roster_failure_never_blocks_the_message() {
        struct Broken;
        impl RosterSource for Broken {
            fn roster(&self, _chat: &str) -> SendBoxRoster<'_> {
                Box::pin(async { Err("grup tidak bisa dibaca".into()) })
            }
        }
        let ctx = StoreContext::new(store_with("rosterbroken", "{}"));
        let p = Pipeline::new(ctx, Arc::new(RecordingSender::default()), "bot", ParseOptions::default())
            .with_roster_source(Arc::new(Broken));
        // Pesan tetap diproses; roster kosong bukan reason untuk menolak.
        let m = inbound(
            ".ping",
            "6281@s.whatsapp.net",
            "120363@g.us",
            true,
            "R1",
        );
        let o = p.process(&m, 1_000).await.unwrap();
        assert!(matches!(
            o,
            Outcome::NotACommand | Outcome::Denied | Outcome::SpamBlocked
        ));
    }

    #[tokio::test]
    async fn private_chat_never_asks_for_a_roster() {
        // Non-grup: tidak ada roster, dan sumber roster tidak boleh dipanggil.
        struct Tripwire(Arc<Mutex<usize>>);
        impl RosterSource for Tripwire {
            fn roster(&self, _chat: &str) -> SendBoxRoster<'_> {
                *self.0.lock().unwrap() += 1;
                Box::pin(async { Ok(RosterSnapshot::default()) })
            }
        }
        let calls = Arc::new(Mutex::new(0));
        let ctx = StoreContext::new(store_with("rostertrip", "{}"));
        let p = Pipeline::new(ctx, Arc::new(RecordingSender::default()), "bot", ParseOptions::default())
            .with_roster_source(Arc::new(Tripwire(calls.clone())));
        let m = inbound("halo", "6281@s.whatsapp.net", "6281@s.whatsapp.net", false, "R2");
        p.process(&m, 1_000).await.unwrap();
        assert_eq!(*calls.lock().unwrap(), 0);
    }

    async fn wait_for_len(s: &RecordingSender, n: usize) -> Vec<(String, wa::Message, usize)> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let got = s.sent.lock().unwrap().clone();
            if got.len() >= n {
                return got;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "hanya {} dari {n} balasan terkirim",
                got.len()
            );
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    // =====================================================================
    // Gate di live path — semua memakai `RecordingSender`, tanpa koneksi WA
    // =====================================================================

    /// Plugin uji nyata untuk gate.
///
/// `oktz-wa` tidak depend ke `oktz-plugins`, jadi registry plugin kosong di
/// test binary ini — `#[plugin]` menulis ke `linkme::distributed_slice`, dan
/// `linkme` bukan dependency langsung `oktz-wa` (lihat
/// [`PluginLookup`]). Jadi plugin-nya dibuat langsung dan dipasang lewat
/// `with_plugin_lookup`.
///
/// Yang dibuktikan test ini bukan cuma "tidak ada balasan", tapi "handler
/// plugin benar-benar tidak dipanggil": `calls` dihitung DI DALAM handler.
struct TestPlugin {
    entry: &'static PluginEntry,
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

impl TestPlugin {
    fn new(cooldown: u32, energi: i64, is_premium: bool) -> Self {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let entry: &'static PluginEntry = Box::leak(Box::new(PluginEntry {
            meta: oktz_plugin_api::meta::PluginMeta {
                name: "gatetest".into(),
                category: "fun".into(),
                cooldown,
                energi,
                is_premium,
                is_private: false,
                ..Default::default()
            },
            handler: Arc::new(move |_c: Ctx| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Reply::text("gatetest:jalan")
            }),
        }));
        Self { entry, calls }
    }

    fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Lookup yang menjawab command apa pun ke plugin ini — cukup untuk
    /// membuktikan gate, tanpa perlu mendaftarkan 30 plugin sungguhan.
    fn lookup(&self) -> PluginLookup {
        let entry = self.entry;
        Arc::new(move |_cmd| Some(entry))
    }

    fn pipeline_with_sender(
        &self,
        ctx: StoreContext,
        sender: Arc<RecordingSender>,
        energi: EnergiSettings,
    ) -> Pipeline {
        Pipeline::new(ctx, sender, BOT, ParseOptions::default())
            .with_identity(Identity::default())
            .with_plugin_lookup(self.lookup())
            .with_energi_settings(energi)
    }
}

    const GROUP: &str = "120363@g.us";
    const MEMBER: &str = "628111@s.whatsapp.net";
    /// JID bot uji — angka di dalamnya harus sama dengan [`BOT`].
    const BOT_JID: &str = "628999@s.whatsapp.net";
    /// `bot_id` harus mengandung angka karena `is_bot_admin_check` membandingkan
    /// digit (`digits("628999@s.whatsapp.net")` = "628999"), dan roster
    /// [`AdminRoster`] menaruh bot di `628999@lid`.
    const BOT: &str = "628999";

    /// Tier default 1 — cukup untuk membuktikan gate menolak, dan cukup untuk
    /// membuktikan `capenergi: 0` benar-benar mematikan biaya. Owner/premium
    /// `-1` (tak terbatas) sudah sesuai `config.js:112-117`.
    fn poor() -> EnergiSettings {
        EnergiSettings {
            enabled: true,
            owner: -1,
            premium: -1,
            default: 1,
        }
    }

    /// Pipeline DM yang memakai plugin uji. Sender dikembalikan supaya test
    /// bisa memeriksa isi pesan yang benar-benar terkirim.
    fn dm(plugin: &TestPlugin, store: &str, energi: EnergiSettings) -> (Arc<Pipeline>, Arc<RecordingSender>) {
        let sender = Arc::new(RecordingSender::default());
        let ctx = StoreContext::new(store_with(&format!("gate-dm-{}", n()), store));
        let p = plugin.pipeline_with_sender(ctx, sender.clone(), energi);
        (Arc::new(p), sender)
    }

    /// Pipeline grup dengan roster yang mengaku bot admin — tanpa itu gate
    /// hapus-pesan membalas `NOT_ADMIN` alih-alih menghapus.
    fn grp(plugin: &TestPlugin, store: &str) -> (Arc<Pipeline>, Arc<RecordingSender>) {
        let sender = Arc::new(RecordingSender::default());
        let ctx = StoreContext::new(store_with(&format!("gate-grp-{}", n()), store));
        let p = plugin
            .pipeline_with_sender(ctx, sender.clone(), EnergiSettings::default())
            .with_roster_source(Arc::new(AdminRoster));
        (Arc::new(p), sender)
    }

    fn n() -> usize {
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    fn inbound_n(body: &str, sender: &str, chat: &str, is_group: bool) -> InboundMessage {
        inbound(body, sender, chat, is_group, &format!("M{}", n()))
    }

    // --- Gate 1: rantai group-protection ------------------------------

    #[tokio::test]
    async fn blocked_link_in_group_is_deleted_and_the_plugin_never_runs() {
        // `antilink: "on"` → `handleAntilink` (handler.js:782) →
        // `DeleteAndReply`: pesan dihapus, peringatan dikirim, chain berhenti
        // SEBELUM dispatch.
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = grp(&plug, r#"{"groups": {"120363@g.us": {"antilink": "on"}}}"#);
        // `.example` bukan TLD, jadi `contains_link` (group.rs:746) sengaja tidak
        // menghitungnya — pakai host sungguhan.
        let m = inbound_n(".gatetest https://contoh.com/x", MEMBER, GROUP, true);
        let out = p.process(&m, 1_000).await.unwrap();

        assert_eq!(out, Outcome::Sent, "gate membalas peringatan, bukan plugin");
        assert_eq!(plug.calls(), 0, "handler plugin TIDAK boleh dipanggil");
        assert_eq!(
            s.deleted(),
            vec![(GROUP.to_string(), m.info.id.clone(), MEMBER.to_string())],
            "pesan link harus dihapus untuk semua orang"
        );
        let texts = s.texts();
        assert_eq!(texts.len(), 1, "tepat satu balasan: {texts:?}");
        assert!(
            texts[0].contains("Antilink"),
            "balasan harus peringatan antilink, bukan hasil plugin: {:?}",
            texts[0]
        );
    }

    #[tokio::test]
    async fn sw_group_mention_dihapus_dan_diperingatkan() {
        // `handleAntiSwGc` (`group-protection.js:2075-2140`) — pesan SW-group
        // ("share status ke grup") di grup dengan `antiswgc: "on"` dihapus,
        // lalu bot kirim peringatan "AntiSWGC".
        //
        // Deteksi tipe SW di Rust: `sw_group_type_of` (pipeline.rs) membaca
        // wrapper protobuf `group_mentioned_message` — padanan
        // `message.groupMentionedMessage` di JS.
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = grp(
            &plug,
            r#"{"groups":{"120363@g.us":{"antiswgc":"on"}}}"#,
        );
        let mut msg = wa::Message {
            conversation: Some("hai".into()),
            ..Default::default()
        };
        msg.group_mentioned_message.get_or_insert_default();
        let m = InboundMessage::builder()
            .message(Arc::new(msg))
            .info(Arc::new(MessageInfo {
                source: MessageSource {
                    chat: GROUP.parse().unwrap(),
                    sender: MEMBER.parse().unwrap(),
                    is_from_me: false,
                    is_group: true,
                    ..Default::default()
                },
                id: format!("M{}", n()),
                ..Default::default()
            }))
            .build();
        let out = p.process(&m, 1_000).await.unwrap();

        assert_eq!(out, Outcome::Sent, "gate membalas peringatan, bukan plugin");
        assert_eq!(plug.calls(), 0, "plugin TIDAK boleh dipanggil");
        assert_eq!(
            s.deleted(),
            vec![(GROUP.to_string(), m.info.id.clone(), MEMBER.to_string())],
            "pesan SW-group harus dihapus"
        );
        let texts = s.texts();
        assert_eq!(texts.len(), 1, "tepat satu balasan: {texts:?}");
        assert!(
            texts[0].contains("AntiSWGC"),
            "balasan harus peringatan antiswgc: {:?}",
            texts[0]
        );
    }

    #[tokio::test]
    async fn group_gate_reply_pakai_reply_variant_dari_store() {
        // Peringatan antilink di JS dikirim `m.reply(...)` di dalam
        // `executeProtectionAction` — bentuknya ikut `replyVariant`.
        let plug = TestPlugin::new(3, 0, false);
        let sender = Arc::new(RecordingSender::default());
        let ctx = StoreContext::new(store_with(
            &format!("varian-grp-{}", n()),
            r#"{"settings":{"replyVariant":9},"groups":{"120363@g.us":{"antilink":"on"}}}"#,
        ));
        let p = plug
            .pipeline_with_sender(ctx, sender.clone(), EnergiSettings::default())
            .with_roster_source(Arc::new(AdminRoster));
        let m = inbound_n(".gatetest https://contoh.com/x", MEMBER, GROUP, true);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Sent);

        let sent = sender.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1, "tepat satu balasan peringatan");
        let order = sent[0]
            .1
            .order_message
            .as_option()
            .expect("peringatan gate harus orderMessage (varian 9)");
        assert!(
            order.message.as_deref().unwrap_or("").contains("Antilink"),
            "isi peringatan: {:?}",
            order.message
        );
        // Mention %user% tetap dioper sebagai mention JID (serialize varian).
        let ci = order.context_info.as_option().expect("contextInfo");
        assert!(
            ci.mentioned_jid.contains(&MEMBER.to_string()),
            "mention pengirim harus ada: {:?}",
            ci.mentioned_jid
        );
    }

    #[tokio::test]
    async fn delete_and_stop_removes_the_message_and_says_nothing() {
        // `mute: true` → `gate_mute` (handler.js:809) → `DeleteAndStop`:
        // bot admin menghapus, TANPA balasan. Inilah yang butuh API delete
        // sungguhan — tanpa `delete_message` pesannya hanya diam dan tetap
        // terlihat di grup.
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = grp(&plug, r#"{"groups": {"120363@g.us": {"mute": true}}}"#);
        let m = inbound_n(".gatetest halo", MEMBER, GROUP, true);
        let out = p.process(&m, 1_000).await.unwrap();

        assert_eq!(out, Outcome::GroupStopped);
        assert_eq!(plug.calls(), 0, "plugin harus berhenti di chain");
        assert_eq!(s.deleted().len(), 1, "pesan harus dihapus");
        assert!(
            s.sent.lock().unwrap().is_empty(),
            "DeleteAndStop tidak mengirim apa pun, tapi terkirim: {:?}",
            s.texts()
        );
    }

    #[tokio::test]
    async fn group_chain_runs_before_the_plugin_even_for_a_valid_command() {
        // Tanpa gate, `.gatetest halo` di grup yang dimute akan tetap dibalas
        // plugin-nya. Ini yang mengunci urutan: chain < dispatch.
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = grp(&plug, r#"{"groups": {"120363@g.us": {"mute": true}}}"#);
        let m = inbound_n(".gatetest halo", MEMBER, GROUP, true);
        p.process(&m, 1_000).await.unwrap();
        assert_eq!(plug.calls(), 0);
        assert!(s.sent.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn group_admin_is_exempt_from_the_chain() {
        // `!isAdmin && !isOwner` ada di handler JS untuk mute/hidetag/mutegc,
        // jadi admin grup yang mengirim pesan tidak dihapus.
        let plug = TestPlugin::new(3, 0, false);
        let sender = Arc::new(RecordingSender::default());
        let mut id = Identity::default();
        id.admins.insert("628222@s.whatsapp.net".to_string());
        let p = plug
            .pipeline_with_sender(
                StoreContext::new(store_with("admin-exempt", "{}")),
                sender.clone(),
                EnergiSettings::default(),
            )
            .with_roster_source(Arc::new(AdminRoster))
            .with_identity(id);
        let p = Arc::new(p);

        let m = inbound_n(".gatetest halo", "628222@s.whatsapp.net", GROUP, true);
        let out = p.process(&m, 1_000).await.unwrap();

        assert_ne!(out, Outcome::GroupStopped, "admin tidak boleh dihapus");
        assert!(sender.deleted().is_empty(), "tidak boleh ada delete");
        assert_eq!(plug.calls(), 1, "command admin harus sampai ke plugin");
    }

    // --- Gate 2: rate limiter spam ------------------------------------

    #[tokio::test]
    async fn spammed_user_is_dropped_and_the_plugin_stops_running() {
        // Limiter global 8 titik / 3 detik (`gates.rs:64-66`).
        let plug = TestPlugin::new(0, 0, false);
        let (p, s) = dm(&plug, "{}", EnergiSettings::default());
        let mut ran = 0;
        for i in 0..12 {
            let before = s.sent.lock().unwrap().len();
            let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
            if p.process(&m, 2_000).await.unwrap() == Outcome::SpamBlocked {
                assert_eq!(
                    s.sent.lock().unwrap().len(),
                    before,
                    "pesan #{i} kena spam tapi pipeline tetap mengirim"
                );
            } else {
                ran += 1;
            }
        }
        assert_eq!(ran, 8, "tepat 8 yang lolos");
        assert_eq!(plug.calls(), 8, "plugin hanya jalan untuk 8 pesan itu");
    }

    #[tokio::test]
    async fn owner_never_hits_the_spam_limiter() {
        // `handler.js:1508`: `!m.isOwner && !m.isPremium`.
        let plug = TestPlugin::new(0, 0, false);
        let mut id = Identity::default();
        id.owners.insert(MEMBER.to_string());
        let sender = Arc::new(RecordingSender::default());
        let p = Arc::new(
            plug
                .pipeline_with_sender(
                    StoreContext::new(store_with("spam-owner", "{}")),
                    sender,
                    EnergiSettings::default(),
                )
                .with_identity(id),
        );

        for _ in 0..30 {
            let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
            assert_ne!(
                p.process(&m, 2_000).await.unwrap(),
                Outcome::SpamBlocked,
                "owner diblokir"
            );
        }
        assert_eq!(plug.calls(), 30, "owner tidak pernah kena limiter");
    }

    // --- Gate 4: cooldown ---------------------------------------------

    #[tokio::test]
    async fn user_in_cooldown_gets_the_reaction_path_and_no_dispatch() {
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = dm(&plug, "{}", EnergiSettings::default());

        let first = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&first, 1_000).await.unwrap(), Outcome::Sent);
        assert_eq!(plug.calls(), 1, "command pertama jalan");
        assert_eq!(s.texts().len(), 1);

        // Pemanggilan kedua dalam cooldown → reaksi, bukan pesan.
        let second = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        let out = p.process(&second, 2_000).await.unwrap();
        assert_eq!(out, Outcome::CooldownBlocked);
        assert_eq!(plug.calls(), 1, "plugin TIDAK boleh jalan dua kali");
        assert_eq!(s.reacted(), vec![(MEMBER.to_string(), "⏱️".to_string())]);
        assert_eq!(s.texts().len(), 1, "tidak boleh ada balasan kedua");

        // Setelah cooldown 3 detik habis, command boleh jalan lagi.
        let third = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&third, 5_000).await.unwrap(), Outcome::Sent);
        assert_eq!(plug.calls(), 2);
        assert_eq!(s.texts().len(), 2);
    }

    #[tokio::test]
    async fn cooldown_is_keyed_per_command_not_per_user() {
        // `cmdKey` = `plugin.config.name` (`handler.js:1816`), jadi alias
        // menulis cooldown yang sama dan command lain tidak ikut.
        let plug = TestPlugin::new(3, 0, false);
        let (p, _s) = dm(&plug, "{}", EnergiSettings::default());
        let m1 = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        p.process(&m1, 1_000).await.unwrap();
        assert_eq!(plug.calls(), 1);

        // `.other` dipetakan ke plugin yang sama oleh lookup uji, jadi ini
        // membuktikanhaust-nya per COMMAND: key plugin sama → tetap kena.
        let m2 = inbound_n(".gatetestpremium", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m2, 1_100).await.unwrap(), Outcome::CooldownBlocked);
    }

    // --- Gate 5: energi -----------------------------------------------

    /// `db.setting("notiflimit")` → potong energi dilaporkan ke user
    /// (`handler.js:1884-1888`).
    #[tokio::test]
    async fn notiflimit_melaporkan_potongan_energi() {
        let plug = TestPlugin::new(0, 1, false);
        let (p, s) = dm(
            &plug,
            r#"{"settings": {"notiflimit": true, "capenergi": {"gatetest": 1}}}"#,
            poor(),
        );

        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        p.process(&m, 1_000).await.unwrap();

        let tpl = oktz_core::config::Messages::default().limit_deducted;
        let sisa = oktz_router::gates::render_limit_deducted(&tpl, 1, 0);
        assert!(
            s.texts().iter().any(|t| t.contains(&sisa)),
            "potongan harus dilaporkan: {:?}",
            s.texts()
        );
    }

    /// Tanpa `notiflimit`, potongannya tidak dilaporkan (`handler.js:1884`).
    #[tokio::test]
    async fn tanpa_notiflimit_tidak_melaporkan_potongan() {
        let plug = TestPlugin::new(0, 1, false);
        let (p, s) = dm(
            &plug,
            r#"{"settings": {"capenergi": {"gatetest": 1}}}"#,
            poor(),
        );

        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        p.process(&m, 1_000).await.unwrap();

        assert!(
            !s.texts().iter().any(|t| t.contains("Limit kau berkurang")),
            "tanpa notiflimit tidak boleh ada notifikasi: {:?}",
            s.texts()
        );
    }

    #[tokio::test]
    async fn insufficient_energi_is_denied_and_not_charged() {
        // `capenergi` memberi `.gatetest` biaya 5; tier default 1 → kurang.
        let plug = TestPlugin::new(0, 0, false);
        let (p, s) = dm(
            &plug,
            r#"{"settings": {"capenergi": {"gatetest": 5}}}"#,
            poor(),
        );

        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        let out = p.process(&m, 1_000).await.unwrap();

        assert_eq!(out, Outcome::EnergiDenied);
        assert_eq!(plug.calls(), 0, "plugin TIDAK boleh jalan tanpa energi");
        assert_eq!(
            s.texts(),
            vec![format!(
                "{MEMBER}: {}",
                oktz_core::config::Messages::default().energi_exceeded
            )],
            "user diberi tahu, dan tidak ada balasan plugin"
        );
        // Saldo tetap 1: gate menolak SEBELUM memotong (`handler.js:1879-1880`
        // `return` tanpa `updateEnergi`).
        assert_eq!(p.gates().energy(MEMBER), Some(1), "energi tak boleh berkurang");
    }

    #[tokio::test]
    async fn a_denied_energi_check_does_not_charge_on_retry() {
        // Kalau gate sempat memotong sebelum menolak, saldo akan turun tiap
        // percobaan.
        let plug = TestPlugin::new(0, 0, false);
        let (p, _s) = dm(
            &plug,
            r#"{"settings": {"capenergi": {"gatetest": 5}}}"#,
            poor(),
        );

        for _ in 0..5 {
            let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
            assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::EnergiDenied);
        }
        assert_eq!(
            p.gates().energy(MEMBER),
            Some(1),
            "lima penolakan tidak boleh memotong apa pun"
        );
    }

    #[tokio::test]
    async fn enough_energi_is_charged_exactly_once_before_dispatch() {
        let plug = TestPlugin::new(0, 0, false);
        let (p, s) = dm(
            &plug,
            r#"{"settings": {"capenergi": {"gatetest": 5}}}"#,
            EnergiSettings { default: 10, ..poor() },
        );

        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Sent);
        assert_eq!(plug.calls(), 1, "plugin jalan setelah charge");
        assert_eq!(p.gates().energy(MEMBER), Some(5), "10 - 5 = 5");
        assert_eq!(s.texts().len(), 1);
    }

    #[tokio::test]
    async fn energi_override_of_zero_wins_over_the_plugin_cost() {
        // `capenergi[name] !== undefined ? ... : energi` — operatornya
        // `!== undefined`, jadi override `0` mematikan biaya
        // (`gates.rs:461-466`).
        let plug = TestPlugin::new(0, 5, false);
        let (p, _s) = dm(
            &plug,
            r#"{"settings": {"capenergi": {"gatetest": 0}}}"#,
            poor(),
        );
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Sent);
        assert_eq!(plug.calls(), 1);
        assert_eq!(p.gates().energy(MEMBER), Some(1), "override 0 = gratis");
    }

    #[tokio::test]
    async fn energi_disabled_in_settings_skips_the_gate_entirely() {
        // `db.setting("energi") === false` mematikan gate
        // (`handler.js:1851-1854`, `gates.rs:414-416`).
        let plug = TestPlugin::new(0, 5, false);
        let (p, _s) = dm(
            &plug,
            r#"{"settings": {"energi": false, "capenergi": {"gatetest": 5}}}"#,
            poor(),
        );
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(
            p.process(&m, 1_000).await.unwrap(),
            Outcome::Sent,
            "gate energi mati = semua orang lolos"
        );
        assert_eq!(plug.calls(), 1);
    }

    #[tokio::test]
    async fn owner_energi_is_unlimited_and_never_charged() {
        // `handler.js:1866-1869`: owner/partner dengan tier `-1` keluar tanpa
        // cek dan tanpa potong.
        let plug = TestPlugin::new(0, 5, false);
        let mut id = Identity::default();
        id.owners.insert(MEMBER.to_string());
        let sender = Arc::new(RecordingSender::default());
        let p = Arc::new(
            plug
                .pipeline_with_sender(
                    StoreContext::new(store_with("energi-owner", "{}")),
                    sender,
                    poor(),
                )
                .with_identity(id),
        );
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Sent);
        assert_eq!(p.gates().energy(MEMBER), Some(-1), "saldo owner tak terbatas");
    }

    // --- Urutan antar gate --------------------------------------------

    #[tokio::test]
    async fn group_chain_beats_the_spam_limiter() {
        // `handler.js:770-871` (chain) dihitung SEBELUM `handler.js:1508`
        // (spam). Kalau urutannya dibalik, pesan yang sudah dihapus dari grup
        // tetap membakar kuota spam pengirimnya.
        let plug = TestPlugin::new(0, 0, false);
        let (p, s) = grp(&plug, r#"{"groups": {"120363@g.us": {"mute": true}}}"#);
        for _ in 0..12 {
            let m = inbound_n("halo dunia", MEMBER, GROUP, true);
            assert_eq!(
                p.process(&m, 2_000).await.unwrap(),
                Outcome::GroupStopped,
                "pesan harus berhenti di chain, bukan di limiter"
            );
        }
        assert_eq!(s.deleted().len(), 12);
        assert_eq!(plug.calls(), 0);
        assert!(s.sent.lock().unwrap().is_empty(), "tidak ada balasan sama sekali");
    }

    #[tokio::test]
    async fn cooldown_beats_energi_so_a_blocked_message_is_never_charged() {
        // Urutan `handler.js:1837` → `1851`. Kalau dibalik, percobaan kedua
        // akan memotong energi lalu baru ditolak cooldown.
        let plug = TestPlugin::new(3, 0, false);
        let (p, _s) = dm(
            &plug,
            r#"{"settings": {"capenergi": {"gatetest": 5}}}"#,
            EnergiSettings { default: 10, ..poor() },
        );

        let m1 = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m1, 1_000).await.unwrap(), Outcome::Sent);
        assert_eq!(p.gates().energy(MEMBER), Some(5));

        let m2 = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m2, 1_100).await.unwrap(), Outcome::CooldownBlocked);
        assert_eq!(
            p.gates().energy(MEMBER),
            Some(5),
            "pesan yang diblokir cooldown tidak boleh memotong energi"
        );
        assert_eq!(plug.calls(), 1);
    }

    #[tokio::test]
    async fn permission_denial_never_burns_cooldown_or_energi() {
        // `handler.js:1810` permission menolak dan `return` — gate cooldown
        // (1837) dan energi (1851) TIDAK pernah tercapai. Kalau
        // `dispatch_target` longgar, user kehilangan energi untuk command yang
        // tidak pernah jalan.
        let plug = TestPlugin::new(3, 5, true);
        let (p, _s) = dm(&plug, "{}", poor());
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        let out = p.process(&m, 1_000).await.unwrap();

        assert_eq!(out, Outcome::Denied, "plugin premium, user biasa");
        assert_eq!(plug.calls(), 0);
        assert!(
            p.gates().energy(MEMBER).is_none(),
            "energi tidak boleh disentuh untuk command yang ditolak — gate tidak pernah jalan"
        );
        // Tidak ada cooldown yang dimulai, jadi percobaan berikutnya (dengan
        // plugin yang sama) juga harus ditolak oleh permission, bukan cooldown.
        let m2 = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m2, 1_100).await.unwrap(), Outcome::Denied);
    }

    /// Pipeline DM dengan store berisi settings — dipakai test replyVariant.
    fn dm_dengan_settings(
        plug: &TestPlugin,
        settings: &str,
    ) -> (Arc<Pipeline>, Arc<RecordingSender>) {
        let sender = Arc::new(RecordingSender::default());
        let ctx = StoreContext::new(store_with(
            &format!("varian-dm-{}", n()),
            &format!(r#"{{"settings":{settings}}}"#),
        ));
        let p = plug.pipeline_with_sender(ctx, sender.clone(), EnergiSettings::default());
        (Arc::new(p), sender)
    }

    /// Pipeline DM dengan file store mentah — untuk layout file JS (tanpa
    /// pembungkus `{"settings": ...}`) dan koleksi di root.
    fn dm_dengan_file(plug: &TestPlugin, raw: &str) -> (Arc<Pipeline>, Arc<RecordingSender>) {
        let sender = Arc::new(RecordingSender::default());
        let ctx = StoreContext::new(store_with(&format!("varian-file-{}", n()), raw));
        let p = plug.pipeline_with_sender(ctx, sender.clone(), EnergiSettings::default());
        (Arc::new(p), sender)
    }

    #[tokio::test]
    async fn settings_layout_js_flat_tetap_dibaca_reply_variant() {
        // `database/main/settings.json` hasil salinan bot JS berisi settings
        // TANPA pembungkus (`database.js:130` — file itu ISi koleksi
        // settings). Kalau bentuk itu tidak dibaca, `replyVariant` hilang dan
        // balasan sistem kembali jadi teks biasa — gejala "replyVariant tidak
        // jalan" di deployment yang database-nya disalin dari bot JS.
        let plug = TestPlugin::new(3, 0, true);
        let (p, s) = dm_dengan_file(&plug, r#"{"replyVariant":9}"#);
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Denied);

        let sent = s.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1);
        let order = sent[0]
            .1
            .order_message
            .as_option()
            .expect("denial harus orderMessage (varian 9 dari layout JS)");
        assert!(
            order.message.as_deref().unwrap_or("").contains("Premium Only"),
            "isi: {:?}",
            order.message
        );
    }

    #[tokio::test]
    async fn setting_ditulis_di_root_pada_layout_js_flat() {
        // Layout JS: setting baru harus masuk ke file yang sama, TANPA
        // membuat key "settings" — kalau key itu muncul, semua setting lama
        // di root ikut hilang dari pembacaan.
        let mut store = JsonStore::open(
            std::env::temp_dir().join(format!("oktz-flat-{}.json", n())),
        )
        .unwrap();
        store.write().unwrap();
        store
            .set("replyVariant", serde_json::json!(9))
            .unwrap();
        store
            .set(
                "settings",
                serde_json::json!({ "replyVariant": 9, "botMode": "public" }),
            )
            .unwrap();
        let path = store.path().to_path_buf();
        let mut store = JsonStore::open(&path).unwrap();
        apply_mutation(
            &mut store,
            &Mutation::SetSetting {
                key: "notiflimit".into(),
                value: serde_json::json!(true),
            },
            1_000,
            path.clone(),
        )
        .unwrap();
        let after = store.all().clone();
        assert_eq!(
            after.get("notiflimit"),
            Some(&serde_json::json!(true)),
            "setting harus ditulis di root pada layout flat"
        );
        assert!(
            after.get("settings").is_none(),
            "layout file tidak boleh berubah jadi merged: {after}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn premium_dari_store_tidak_dibalas_premium_only() {
        // `config.isPremium` (config.js:406-460) membaca koleksi `premium` dan
        // `settings.premiumUsers`. Tanpa itu, user premium terdaftar tetap
        // dianggap non-premium dan command premium-nya selalu dibalas
        // "Premium Only!".
        let plug = TestPlugin::new(3, 0, true);
        let (p, _s) = dm_dengan_file(&plug, r#"{"premium":["628111"]}"#);
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Sent);
        assert_eq!(plug.calls(), 1, "user premium store tidak boleh ditolak");
    }

    #[tokio::test]
    async fn premium_kedaluwarsa_di_store_tidak_dianggap_premium() {
        // `expired` = ms epoch; kalau sudah lewat, premium gugur
        // (config.js:444-452).
        let plug = TestPlugin::new(3, 0, true);
        let (p, _s) = dm_dengan_file(
            &plug,
            r#"{"premium":[{"id":"628111","expired":1000}]}"#,
        );
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        let out = p.process(&m, 2_000_000_000_000).await.unwrap();
        assert_eq!(out, Outcome::Denied, "premium expired harus ditolak");
        assert_eq!(plug.calls(), 0);
    }

    #[tokio::test]
    async fn owner_dari_store_dibaca_seperti_config_owner() {
        // `config.isOwner` (config.js:383-398) membaca koleksi `owner` dan
        // `settings.ownerNumbers`.
        let store = store_with(
            &format!("owner-store-{}", n()),
            r#"{"owner":["628111"],"settings":{"ownerNumbers":["628999"]}}"#,
        );
        let ctx = StoreContext::new(store);
        assert!(ctx.is_owner_user(MEMBER), "koleksi owner harus dibaca");
        assert!(ctx.is_owner_user("628999@s.whatsapp.net"));
        assert!(!ctx.is_owner_user("628777@s.whatsapp.net"));
    }

    #[tokio::test]
    async fn user_banned_dibalas_pesan_banned_dan_plugin_tidak_jalan() {
        // `handler.js:901-913` — user banned: kalau command, balas pesan
        // banned; setelah itu pipeline berhenti.
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = dm_dengan_file(
            &plug,
            r#"{"settings":{"bannedUsers":["628111"]}}"#,
        );
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Banned);
        assert_eq!(plug.calls(), 0, "plugin tidak boleh jalan untuk user banned");
        let texts = s.texts();
        assert_eq!(texts.len(), 1);
        assert!(texts[0].contains("Kamu Dibanned"), "{texts:?}");
    }

    #[tokio::test]
    async fn layout_js_baca_koleksi_dari_file_terpisah() {
        // Bot JS menyimpan satu file per koleksi (`database.js:127-136`).
        // `groups.json` memuat flag proteksi grup; `settings.json` cuma
        // settings. Kalau Rust cuma baca `settings.json`, semua proteksi grup
        // mati di deployment yang database-nya disalin dari bot JS.
        let (_dir, store) = store_js_layout(
            "koleksi",
            r#"{"replyVariant":9}"#,
            r#"{"628111":{"isPremium":true}}"#,
            r#"{"120363@g.us":{"antilink":"on"}}"#,
        );
        let ctx = StoreContext::new(store);
        assert_eq!(
            ctx.group_entry("120363@g.us").get("antilink"),
            Some(&serde_json::json!("on")),
            "groups.json harus terbaca"
        );
        assert_eq!(
            ctx.settings_entry().get("replyVariant"),
            Some(&serde_json::json!(9)),
            "settings.json harus terbaca"
        );
        assert!(
            ctx.user_entry(MEMBER).get("isPremium").is_some(),
            "users.json harus terbaca"
        );
    }

    #[tokio::test]
    async fn layout_js_tulis_koleksi_ke_file_asalnya() {
        // Mutasi harus masuk ke file KOLEKSI JS, bukan digabung ke
        // `settings.json` — kalau tidak, `settings.json` yang dipakai bot JS
        // berubah bentuk dan user/grup jadi hilang dari sana.
        let (dir, store) = store_js_layout("tulis", r#"{"replyVariant":9}"#, "{}", "{}");
        let ctx = StoreContext::new(store);
        ctx.apply(
            &[Mutation::SetUser {
                jid: MEMBER.to_string(),
                data: serde_json::json!({ "name": "Budi" }),
            }],
            1_000,
        )
        .unwrap();
        let users: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("users.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            users.get("628111").and_then(|u| u.get("name")),
            Some(&serde_json::json!("Budi")),
            "user harus ditulis ke users.json"
        );
        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            settings.get("replyVariant"),
            Some(&serde_json::json!(9)),
            "settings.json tidak boleh berubah bentuk: {settings}"
        );
        assert!(
            settings.get("users").is_none(),
            "koleksi tidak boleh masuk ke settings.json: {settings}"
        );
        // Snapshot setelah tulis harus ikut berubah.
        assert_eq!(
            ctx.user_entry(MEMBER).get("name"),
            Some(&serde_json::json!("Budi"))
        );
    }

    #[tokio::test]
    async fn layout_js_setting_ditulis_di_root_settings_json() {
        let (dir, store) = store_js_layout("setting", r#"{"replyVariant":9}"#, "{}", "{}");
        let ctx = StoreContext::new(store);
        ctx.apply(
            &[Mutation::SetSetting {
                key: "notiflimit".into(),
                value: serde_json::json!(true),
            }],
            1_000,
        )
        .unwrap();
        let settings: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            settings.get("notiflimit"),
            Some(&serde_json::json!(true)),
            "setting harus ditulis di root settings.json: {settings}"
        );
    }

    #[tokio::test]
    async fn layout_js_perubahan_eksternal_terbaca_setiap_pesan() {
        // Owner menyalakan `antilink` dari panel (bot JS menulis
        // `groups.json` langsung ke disk). Pesan berikutnya di Rust harus
        // sudah melihat perubahan itu — tanpa refresh, proteksi baru
        // dinyalakan tapi tidak berlaku.
        let (dir, store) = store_js_layout(
            "refresh",
            r#"{"replyVariant":9}"#,
            "{}",
            r#"{"120363@g.us":{}}"#,
        );
        let ctx = StoreContext::new(store.clone());
        assert!(
            ctx.group_entry("120363@g.us")
                .get("antilink")
                .is_none(),
            "awalnya belum ada flag"
        );
        std::fs::write(
            dir.join("groups.json"),
            r#"{"120363@g.us":{"antilink":"on"}}"#,
        )
        .unwrap();
        ctx.refresh();
        assert_eq!(
            ctx.group_entry("120363@g.us").get("antilink"),
            Some(&serde_json::json!("on")),
            "perubahan dari luar harus terbaca setelah refresh"
        );
    }

    #[tokio::test]
    async fn layout_merged_tetap_satu_file() {
        // Tanpa file koleksi di sampingnya, store Rust tetap satu file —
        // file lain yang kebetulan ada di folder yang sama tidak boleh
        // mengubah layout.
        let store = store_with("merged", r#"{"settings":{"replyVariant":9}}"#);
        let ctx = StoreContext::new(store.clone());
        ctx.apply(
            &[Mutation::SetUser {
                jid: MEMBER.to_string(),
                data: serde_json::json!({ "name": "Budi" }),
            }],
            1_000,
        )
        .unwrap();
        let after: Value =
            serde_json::from_str(&std::fs::read_to_string(store.path()).unwrap()).unwrap();
        assert_eq!(
            after.get("users").and_then(|u| u.get("628111")).and_then(|u| u.get("name")),
            Some(&serde_json::json!("Budi")),
            "layout satu file tetap menulis di file yang sama: {after}"
        );
    }

    #[tokio::test]
    async fn grup_banned_mengabaikan_command_non_owner() {
        // `handler.js:916-921` — command dari non-owner di grup banned
        // diabaikan tanpa balasan.
        let plug = TestPlugin::new(3, 0, false);
        let (p, s) = grp(
            &plug,
            r#"{"groups":{"120363@g.us":{"isBanned":true}}}"#,
        );
        let m = inbound_n(".gatetest halo", MEMBER, GROUP, true);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::GroupBanned);
        assert_eq!(plug.calls(), 0);
        assert!(s.texts().is_empty(), "grup banned: diam, tanpa balasan");
    }

    #[tokio::test]
    async fn permission_denial_pakai_reply_variant_dari_store() {
        // JS: balasan checkPermission dikirim lewat `m.reply(...)`
        // (middleware.js:50-137), jadi bentuknya ikut `db.setting("replyVariant")`
        // (serialize.js:958-991) — varian 9 produksi mengirim `orderMessage`
        // berisi teks "Premium Only!", bukan teks biasa. Port lama melewatkan
        // varian di jalur ini.
        let plug = TestPlugin::new(3, 0, true);
        let (p, s) = dm_dengan_settings(&plug, r#"{"replyVariant":9}"#);
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::Denied);
        assert_eq!(plug.calls(), 0, "plugin premium tidak boleh jalan");

        let sent = s.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1, "denial tetap dibalas satu pesan");
        let order = sent[0]
            .1
            .order_message
            .as_option()
            .expect("denial premium harus orderMessage (varian 9)");
        assert_eq!(
            order.message.as_deref(),
            Some("💎 *Premium Only!* Fitur ini khusus member Premium. Ketik *.benefitpremium* untuk info upgrade.")
        );
        assert_eq!(order.order_title.as_deref(), Some("CONTROL"));
    }

    #[tokio::test]
    async fn energi_denial_pakai_reply_variant_dari_store() {
        // JS: `handler.js:1879` balas energiExceeded via `m.reply` — ikut
        // replyVariant juga. `poor()` = tier default 1, plugin butuh 5.
        let plug = TestPlugin::new(0, 5, false);
        let sender = Arc::new(RecordingSender::default());
        let ctx = StoreContext::new(store_with(
            &format!("varian-energi-{}", n()),
            r#"{"settings":{"replyVariant":9}}"#,
        ));
        let p = plug.pipeline_with_sender(ctx, sender.clone(), poor());
        let m = inbound_n(".gatetest halo", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m, 1_000).await.unwrap(), Outcome::EnergiDenied);

        let sent = sender.sent.lock().unwrap().clone();
        assert_eq!(sent.len(), 1);
        let order = sent[0]
            .1
            .order_message
            .as_option()
            .expect("denial energi harus orderMessage (varian 9)");
        assert!(
            order.message.as_deref().unwrap_or("").contains("Energi Habis"),
            "isi pesan energi: {:?}",
            order.message
        );
    }

    // =====================================================================
    // Task 2 — mutasi store: round-trip dua pesan lewat store sungguhan
    // =====================================================================

    /// Satu entry plugin dengan handler yang ditentukan pemanggil.
    fn entry_named(
        name: &'static str,
        handler: impl Fn(Ctx) -> Reply + Send + Sync + 'static,
    ) -> PluginLookup {
        let entry: &'static PluginEntry = Box::leak(Box::new(PluginEntry {
            meta: oktz_plugin_api::meta::PluginMeta {
                name: name.into(),
                category: "store".into(),
                ..Default::default()
            },
            handler: Arc::new(handler),
        }));
        Arc::new(move |cmd| (cmd == name).then_some(entry))
    }

    /// Plugin yang MEMBACA `ctx.user_data` dan mengembalikan apa yang dia
    /// lihat — dipakai sebagai "pesan berikutnya" untuk membuktikan write
    /// tidak hanya hidup di memori plugin.
    fn echo_koin_lookup() -> PluginLookup {
        entry_named("balance", |c: Ctx| {
            let koin = c.user_field("koin").and_then(Value::as_i64).unwrap_or(-1);
            Reply::text(format!("koin={koin}"))
        })
    }

    fn pipeline_for(store_body: &str, lookup: PluginLookup) -> (Arc<Pipeline>, Arc<RecordingSender>) {
        let ctx = StoreContext::new(store_with("mutate", store_body));
        let sender = Arc::new(RecordingSender::default());
        let p = Arc::new(
            Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
                .with_identity(Identity::default())
                .with_plugin_lookup(lookup),
        );
        (p, sender)
    }

    const EMPTY_STORE: &str = r#"{"users":{},"groups":{},"settings":{}}"#;

    /// BUKTI KUNCI Task 2: mutasi yang ditulis plugin untuk pesan pertama
    /// HARUS terlihat oleh pesan kedua, dan harus benar-benar ada di file.
    ///
    /// Yang dibuktikan tiga lapis, bukan cuma yang pertama:
    /// 1. pesan #2 (plugin berbeda, `Ctx` baru) membaca nilai baru —
    ///    jadi snapshot `StoreContext` benar-benar tertukar;
    /// 2. file JSON di disk memuat nilai baru — jadi `JsonStore::write`
    ///    benar-benar dipanggil, bukan hanya state di memori;
    /// 3. `JsonStore` yang di-BUKA ULANG dari disk memuat nilai yang sama —
    ///    jadi tidak ada cache yang menutupi masalah.
    #[tokio::test]
    async fn mutation_from_one_message_is_visible_to_the_next_through_the_real_store() {
        // Pesan #1: plugin menyimpan, pesan #2: plugin membaca.
        let saving = entry_named("nabung", |c: Ctx| {
            let before = c.user_field("koin").and_then(Value::as_i64).unwrap_or(0);
            Reply::text("✅ TERSIMPAN").with_mutation(Mutation::SetUser {
                jid: c.sender.clone(),
                data: serde_json::json!({ "koin": before + 500 }),
            })
        });
        let reader = echo_koin_lookup();
        let lookup = Arc::new(move |cmd: &str| {
            let s = saving(cmd);
            s.or_else(|| reader(cmd))
        });
        let (p, sender) = pipeline_for(EMPTY_STORE, lookup);

        let path = p.ctx().read().path().to_path_buf();

        // --- pesan #1: menulis ------------------------------------------
        let m1 = inbound_n(".nabung", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m1, 10_000).await.unwrap(), Outcome::Sent);
        assert_eq!(
            sender.texts(),
            vec![format!("{MEMBER}: ✅ TERSIMPAN")],
            "pesan konfirmasi harus terkirim"
        );

        // (2) file di disk sudah memuat nilai baru SEBELUM pesan #2.
        let disk = std::fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&disk).unwrap();
        assert_eq!(
            v["users"]["628111"]["koin"].as_i64(),
            Some(500),
            "write harus sampai ke disk, bukan hanya ke state pipeline"
        );

        // (3) store yang dibuka ULANG dari disk punya nilai yang sama —
        //     tidak ada cache yang menahan data lama.
        let reopened = JsonStore::open(&path).unwrap();
        assert_eq!(
            reopened.get("users").unwrap()["628111"]["koin"].as_i64(),
            Some(500),
            "store baru dari file yang sama harus melihat write tadi"
        );

        // (1) pesan #2, plugin berbeda, Ctx baru: nilai baru harus terlihat.
        let m2 = inbound_n(".balance", MEMBER, MEMBER, false);
        assert_eq!(p.process(&m2, 11_000).await.unwrap(), Outcome::Sent);
        assert_eq!(
            sender.texts().last().unwrap(),
            &format!("{MEMBER}: koin=500"),
            "pesan kedua harus membaca hasil write pesan pertama"
        );
    }

    /// `setSetting` juga harus lintas-pesan lewat disk — inilah yang
    /// plugin `store/*` andalkan (`db.setting('storeLists', ...)`).
    #[tokio::test]
    async fn a_setting_written_by_a_plugin_is_read_by_the_next_message() {
        let saving = entry_named("setlist", |_c: Ctx| {
            Reply::text("ok").with_mutation(Mutation::SetSetting {
                key: "storeLists".into(),
                value: serde_json::json!([{"name": "S&K"}]),
            })
        });
        let reading = entry_named("lists", |c: Ctx| {
            let n = c.setting_list("storeLists").len();
            Reply::text(format!("jumlah={n}"))
        });
        let lookup = Arc::new(move |cmd: &str| saving(cmd).or_else(|| reading(cmd)));
        let (p, sender) = pipeline_for(EMPTY_STORE, lookup);

        p.process(&inbound_n(".setlist", MEMBER, MEMBER, false), 1_000)
            .await
            .unwrap();
        p.process(&inbound_n(".lists", MEMBER, MEMBER, false), 2_000)
            .await
            .unwrap();

        assert_eq!(sender.texts().last().unwrap(), &format!("{MEMBER}: jumlah=1"));
    }

    /// Mutasi TANPA balasan (`Reply::Mutate { reply: None }`) harus tetap
    /// ditulis. Ini padanan pola JS `await m.react('✅')` lalu `return`.
    #[tokio::test]
    async fn a_plugin_can_store_without_replying() {
        let saving = entry_named("diam", |c: Ctx| {
            Reply::None.with_mutation(Mutation::SetUser {
                jid: c.sender.clone(),
                data: serde_json::json!({ "level": 7 }),
            })
        });
        let reader = echo_koin_lookup();
        let lookup = Arc::new(move |cmd: &str| saving(cmd).or_else(|| reader(cmd)));
        let (p, sender) = pipeline_for(EMPTY_STORE, lookup);

        assert_eq!(
            p.process(&inbound_n(".diam", MEMBER, MEMBER, false), 1_000)
                .await
                .unwrap(),
            Outcome::Sent,
            "reply None = sengaja diam, bukan gagal"
        );
        assert!(sender.texts().is_empty(), "tidak ada pesan yang dikirim");

        let disk: Value =
            serde_json::from_str(&std::fs::read_to_string(p.ctx().read().path()).unwrap()).unwrap();
        assert_eq!(disk["users"]["628111"]["level"].as_i64(), Some(7));
    }

    /// Kegagalan write TIDAK boleh diam: outcome-nya harus berbeda dari
    /// `Sent`, dan TIDAK ada pesan "tersimpan" yang terkirim.
    ///
    /// Cara memaksa gagalnya: file store diganti jadi DIREKTORI, jadi
    /// `JsonStore::open` gagal dan `write()` tidak pernah kesampaian.
    #[tokio::test]
    async fn a_failed_write_is_surfaced_and_the_optimistic_reply_is_withheld() {
        let saving = entry_named("nabung", |c: Ctx| {
            Reply::text("✅ TERSIMPAN").with_mutation(Mutation::SetUser {
                jid: c.sender.clone(),
                data: serde_json::json!({ "koin": 999 }),
            })
        });
        let (p, sender) = pipeline_for(EMPTY_STORE, saving);

        // Ganti file dengan direktori yang namanya sama.
        let path = p.ctx().read().path().to_path_buf();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        let guard = TempDirGuard(Some(path.clone()));

        let out = p
            .process(&inbound_n(".nabung", MEMBER, MEMBER, false), 1_000)
            .await
            .unwrap();
        assert_eq!(
            out,
            Outcome::StoreWriteFailed,
            "kegagalan write harus terlihat di outcome"
        );
        assert!(
            sender.texts().is_empty(),
            "pesan 'TERSIMPAN' TIDAK boleh dikirim kalau tidak ada yang tersimpan: {:?}",
            sender.texts()
        );

        // Pesan BERIKUTNYA tetap jalan — loop tidak boleh mati.
        let reader = echo_koin_lookup();
        drop(reader);
        guard.remove();
    }

    /// Direktori yang dibikin test dibersihkan setelah test, apa pun hasilnya.
    /// Dipakai `Drop` supaya test yang gagal di tengah tidak meninggalkan
    /// `/tmp` penuh file bernama `oktz-pipeline-mutate-N.json`.
    struct TempDirGuard(Option<PathBuf>);

    impl TempDirGuard {
        fn remove(mut self) {
            if let Some(p) = self.0.take() {
                let _ = std::fs::remove_dir_all(p);
            }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            if let Some(p) = self.0.take() {
                let _ = std::fs::remove_dir_all(p);
            }
        }
    }

    /// `render_reply` harus MENOLAK `Reply::Mutate`, bukan diam-diam
    /// mengirim teksnya tanpa menulis store. Ini pagar gegen pemanggil
    /// baru yang lupa `into_parts`.
    #[test]
    fn rendering_an_unapplied_mutation_is_an_error_not_a_lie() {
        let r = Reply::text("✅ TERSIMPAN").with_mutation(Mutation::SetUser {
            jid: "628111@s.whatsapp.net".into(),
            data: serde_json::json!({"koin": 1}),
        });
        match render_reply(&r, &ReplyTarget::default()) {
            Err(PipelineError::UnappliedMutation { count }) => assert_eq!(count, 1),
            other => panic!("harus error UnappliedMutation, dapat {other:?}"),
        }
    }

    // --- Semantik setUser (murni, tanpa file) ---------------------------

    fn users_with(body: &str) -> Value {
        serde_json::from_str(body).unwrap()
    }

    #[test]
    fn set_user_creates_a_full_record_with_all_defaults() {
        let mut users = users_with("{}");
        apply_set_user(
            &mut users,
            "628111",
            &serde_json::json!({}),
            0,
        );
        let u = &users["628111"];
        // Setiap field default dari `database.js:559-601` harus ada.
        for (field, want) in [
            ("jid", serde_json::json!("628111")),
            ("number", serde_json::json!("628111")),
            ("lid", serde_json::Value::Null),
            ("name", serde_json::json!("Unknown")),
            ("energi", serde_json::json!(99_999)),
            ("isPremium", serde_json::json!(false)),
            ("isBanned", serde_json::json!(false)),
            ("exp", serde_json::json!(0)),
            ("level", serde_json::json!(1)),
            ("koin", serde_json::json!(0)),
            ("saldo", serde_json::json!(0)),
            ("unlockedFeatures", serde_json::json!([])),
            ("registeredAt", serde_json::Value::Null),
            ("registrationCount", serde_json::json!(0)),
            ("hasClaimedRegisterReward", serde_json::json!(false)),
            ("unregisteredAt", serde_json::Value::Null),
            ("clanId", serde_json::Value::Null),
            ("isRegistered", serde_json::json!(false)),
            ("regName", serde_json::Value::Null),
            ("regAge", serde_json::Value::Null),
            ("regGender", serde_json::Value::Null),
            ("rpg", serde_json::json!({})),
            ("inventory", serde_json::json!({})),
            ("access", serde_json::json!([])),
            ("cooldowns", serde_json::json!({})),
            ("lastSeen", serde_json::json!("1970-01-01T00:00:00.000Z")),
        ] {
            assert_eq!(u[field], want, "field {field}");
        }
    }

    /// `rpg.bank = 5000` tanpa mengirim ulang blok `rpg` harus MEMERGE.
    /// Kalau jadi replace, `rpg.exp` yang sudah ada ikut hilang — dan itu
    /// persis yang terjadi pada `plugins/rpg/bank.js:50-51`.
    #[test]
    fn set_user_merges_rpg_and_inventory_one_level_deep() {
        let mut users = users_with(r#"{"628111":{"rpg":{"exp":250,"bank":0},"inventory":{"potion":2}}}"#);
        apply_set_user(
            &mut users,
            "628111",
            &serde_json::json!({ "rpg": { "bank": 5000 } }),
            0,
        );
        assert_eq!(users["628111"]["rpg"]["bank"].as_i64(), Some(5000));
        assert_eq!(
            users["628111"]["rpg"]["exp"].as_i64(),
            Some(250),
            "rpg.exp harus selamat dari merge"
        );
        assert_eq!(users["628111"]["inventory"]["potion"].as_i64(), Some(2));
    }

    /// `koin: 0` harus tersimpan 0. `??` (bukan `||`) yang dipakai
    /// `database.js:578` — kalau `||`, saldo habis akan naik lagi ke default.
    #[test]
    fn set_user_keeps_zero_because_js_uses_nullish_coalescing() {
        let mut users = users_with(r#"{"628111":{"koin":900,"level":9}}"#);
        apply_set_user(
            &mut users,
            "628111",
            &serde_json::json!({ "koin": 0, "level": 0 }),
            0,
        );
        assert_eq!(users["628111"]["koin"].as_i64(), Some(0));
        assert_eq!(users["628111"]["level"].as_i64(), Some(0));
    }

    /// `data.name || existing.name || "Unknown"` memakai `||`, jadi
    /// string KOSONG di `data` harus jatuh ke existing, bukan disimpan "".
    #[test]
    fn empty_name_falls_back_because_set_user_uses_or_not_nullish() {
        let mut users = users_with(r#"{"628111":{"name":"Budi"}}"#);
        apply_set_user(&mut users, "628111", &serde_json::json!({ "name": "" }), 0);
        assert_eq!(users["628111"]["name"].as_str(), Some("Budi"));
    }

    /// `balance`/`limit` adalah field legacy: dibaca lalu DIHAPUS, dan
    /// nilainya jadi `koin`/`energi` (`database.js:539-548`).
    #[test]
    fn legacy_balance_and_limit_migrate_into_koin_and_energi() {
        let mut users = users_with(r#"{"628111":{"balance":777,"limit":42}}"#);
        apply_set_user(&mut users, "628111", &serde_json::json!({}), 0);
        assert_eq!(users["628111"]["koin"].as_i64(), Some(777));
        assert_eq!(users["628111"]["energi"].as_i64(), Some(42));
        assert!(users["628111"].get("balance").is_none(), "balance harus hilang");
        assert!(users["628111"].get("limit").is_none(), "limit harus hilang");
    }

    /// JID LID: key `lid:xxx`, tapi `jid`/`number` jadi `null` dan `lid`
    /// diisi bersihnya (`database.js:583-586`).
    #[test]
    fn lid_user_gets_null_jid_and_a_filled_lid_field() {
        let mut users = users_with("{}");
        apply_set_user(&mut users, "lid:9998887771", &serde_json::json!({}), 0);
        let u = &users["lid:9998887771"];
        assert_eq!(u["jid"], serde_json::Value::Null);
        assert_eq!(u["number"], serde_json::Value::Null);
        assert_eq!(u["lid"].as_str(), Some("9998887771"));
    }

/// `access` pakai `||` (`database.js:600`), dan `[]` di JS itu TRUTHY —
/// jadi `access: []` dari plugin BENAR-BENAR mengosongkan grant. Yang
/// membuat `existing.access` dipakai adalah `access` tidak ada sama sekali
/// atau `null`. Dikunci karena mudah salah di sini: kalau `access`
/// ikut diperlakukan `??`, `[]` jadi tidak bisa dipakai untuk mencabut
/// akses — dan `handler.js` justru bergantung pada itu.
#[test]
fn an_explicit_empty_access_array_clears_grants_because_arrays_are_truthy_in_js() {
    let mut users = users_with(r#"{"628111":{"access":[{"cmd":"hd","expired":null}]}}"#);
    apply_set_user(&mut users, "628111", &serde_json::json!({ "access": [] }), 0);
    assert_eq!(
        users["628111"]["access"],
        serde_json::json!([]),
        "[] harus menang atas access lama — `[]` truthy di JS"
    );
}

/// `access` yang tidak disebut plugin harus MEWARISI yang lama, karena
/// `data.access` `undefined` falsy lalu jatuh ke `existing.access`.
#[test]
fn an_absent_access_field_keeps_the_existing_grant() {
    let mut users = users_with(r#"{"628111":{"access":[{"cmd":"hd","expired":null}]}}"#);
    apply_set_user(&mut users, "628111", &serde_json::json!({ "koin": 5 }), 0);
    assert_eq!(
        users["628111"]["access"][0]["cmd"].as_str(),
        Some("hd"),
        "access lama harus bertahan saat plugin tidak menyebut field itu"
    );
}

/// `access: null` juga falsy, jadi grant lama ikut dipakai — inilah
/// perbedaan nyata antara `||` dan `??` di baris ini.
#[test]
fn a_null_access_field_keeps_the_existing_grant() {
    let mut users = users_with(r#"{"628111":{"access":[{"cmd":"hd","expired":null}]}}"#);
    apply_set_user(&mut users, "628111", &serde_json::json!({ "access": null }), 0);
    assert_eq!(users["628111"]["access"][0]["cmd"].as_str(), Some("hd"));
}

    #[test]
    fn unknown_fields_survive_set_user() {
        let mut users = users_with(r#"{"628111":{"rpg":{"guildId":"g1"},"custom":{"deep":[1,2]}}}"#);
        apply_set_user(&mut users, "628111", &serde_json::json!({ "koin": 5 }), 0);
        assert_eq!(users["628111"]["rpg"]["guildId"].as_str(), Some("g1"));
        assert_eq!(users["628111"]["custom"]["deep"][1].as_i64(), Some(2));
    }

    /// --- setGroup -----------------------------------------------------

    #[test]
    fn set_group_creates_a_full_record_with_documented_defaults() {
        let mut groups = users_with("{}");
        apply_set_group(&mut groups, "120363@g.us", &serde_json::json!({}));
        let g = &groups["120363@g.us"];
        for (field, want) in [
            ("jid", serde_json::json!("120363@g.us")),
            ("name", serde_json::json!("Unknown Group")),
            ("welcome", serde_json::json!(false)),
            ("leave", serde_json::json!(false)),
            ("goodbye", serde_json::json!(false)),
            ("antilink", serde_json::json!(false)),
            ("antitoxic", serde_json::json!(false)),
            ("mute", serde_json::json!(false)),
            ("game", serde_json::json!(true)),
            ("rpg", serde_json::json!(true)),
            ("warnings", serde_json::json!([])),
            ("chat", serde_json::json!({})),
        ] {
            assert_eq!(g[field], want, "field {field}");
        }
    }

    /// `mute: true` yang sudah ada tidak boleh dikembalikan `false` oleh
    /// default saat plugin cuma menulis field lain.
    #[test]
    fn set_group_keeps_existing_toggles_when_data_omits_them() {
        let mut groups = users_with(
            r#"{"120363@g.us":{"mute":true,"game":false,"name":"Komunitas","custom":1}}"#,
        );
        apply_set_group(&mut groups, "120363@g.us", &serde_json::json!({ "antilink": true }));
        let g = &groups["120363@g.us"];
        assert_eq!(g["mute"], serde_json::json!(true), "mute lama harus bertahan");
        assert_eq!(g["game"], serde_json::json!(false), "game:false bukan 'tak ada'");
        assert_eq!(g["antilink"], serde_json::json!(true));
        assert_eq!(g["name"].as_str(), Some("Komunitas"));
        assert_eq!(g["custom"].as_i64(), Some(1), "field asing harus bertahan");
    }

    // --- iso_utc -------------------------------------------------------

    #[test]
    fn iso_timestamps_match_javascript_to_iso_string() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_utc(1_767_225_600_000), "2026-01-01T00:00:00.000Z");
        // Milidetik tidak boleh terpotong.
        assert_eq!(iso_utc(1_767_225_600_123), "2026-01-01T00:00:00.123Z");
        // 2024 adalah tahun loncat (Feb 29) — algoritma kabisat.
        assert_eq!(iso_utc(1_709_164_800_000), "2024-02-29T00:00:00.000Z");
        // Sisa milidetik sebelum rollover.
        assert_eq!(iso_utc(999), "1970-01-01T00:00:00.999Z");
        assert_eq!(iso_utc(1_000), "1970-01-01T00:00:01.000Z");
    }

    #[test]
    fn js_truthiness_matches_the_four_falsy_values() {
        assert!(!truthy(&Value::Null));
        assert!(!truthy(&Value::from(false)));
        assert!(!truthy(&Value::from(0)));
        assert!(!truthy(&serde_json::json!("")));
        assert!(truthy(&Value::from(1)));
        assert!(truthy(&serde_json::json!("x")));
        // Array dan object KOSONG tetap truthy di JS — sumber bug klasik
        // kalau `||` dipakai untuk nilai yang bisa berupa array.
        assert!(truthy(&serde_json::json!([])));
        assert!(truthy(&serde_json::json!({})));
    }

    fn inbound_reply(
        body: &str,
        sender: &str,
        chat: &str,
        is_group: bool,
        quoted_body: &str,
        participant: Option<&str>,
    ) -> InboundMessage {
        let mut m = inbound(body, sender, chat, is_group, "Q1");
        m.message = Arc::new(wa::Message {
            extended_text_message: MessageField::some(wa::message::ExtendedTextMessage {
                text: Some(body.to_string()),
                context_info: MessageField::some(wa::ContextInfo {
                    stanza_id: Some("QUOTED1".to_string()),
                    participant: participant.map(str::to_string),
                    mentioned_jid: vec!["628999@s.whatsapp.net".to_string()],
                    quoted_message: MessageField::some(wa::Message {
                        conversation: Some(quoted_body.to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        });
        m
    }

    #[test]
    fn quoted_and_mentions_reach_the_plugin_context() {
        let p = Pipeline::new(
            StoreContext::new(store_with("q", "{}")),
            Arc::new(RecordingSender::default()),
            "bot",
            ParseOptions::default(),
        );
        let m = inbound_reply(
            "halo",
            "628111@s.whatsapp.net",
            "120363@g.us",
            true,
            "pesan lama",
            Some("628222@s.whatsapp.net"),
        );

        let caller = p.caller_of(&m);
        assert_eq!(caller.mentioned, vec!["628999@s.whatsapp.net"]);
        let q = caller.quoted.expect("quoted harus terbaca");
        assert_eq!(q.id, "QUOTED1");
        assert_eq!(q.text, "pesan lama");
        assert_eq!(q.kind, QuotedKind::Text);
        assert_eq!(q.sender, "628222@s.whatsapp.net");
        assert_eq!(q.chat, "120363@g.us");
        assert!(!q.is_media());
    }

    #[test]
    fn message_without_context_info_has_no_quoted_and_no_mentions() {
        let p = Pipeline::new(
            StoreContext::new(store_with("q0", "{}")),
            Arc::new(RecordingSender::default()),
            "bot",
            ParseOptions::default(),
        );
        let m = inbound("halo", "628111@s.whatsapp.net", "628111@s.whatsapp.net", false, "M0");
        let caller = p.caller_of(&m);
        assert!(caller.quoted.is_none());
        assert!(caller.mentioned.is_empty());
    }

    #[test]
    fn quoted_image_carries_mimetype_and_media_flag() {
        let mut m = inbound("lihat", "628111@s.whatsapp.net", "628111@s.whatsapp.net", false, "M1");
        m.message = Arc::new(wa::Message {
            extended_text_message: MessageField::some(wa::message::ExtendedTextMessage {
                text: Some("lihat".into()),
                context_info: MessageField::some(wa::ContextInfo {
                    quoted_message: MessageField::some(wa::Message {
                        image_message: MessageField::some(wa::message::ImageMessage {
                            mimetype: Some("image/jpeg".into()),
                            caption: Some("foto".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        });
        let p = Pipeline::new(
            StoreContext::new(store_with("qi", "{}")),
            Arc::new(RecordingSender::default()),
            "bot",
            ParseOptions::default(),
        );
        let q = p.caller_of(&m).quoted.expect("quoted image");
        assert_eq!(q.kind, QuotedKind::Image);
        assert_eq!(q.kind.wire_type(), "imageMessage");
        assert!(q.is_media());
        assert_eq!(q.text, "foto");
        assert_eq!(q.mimetype.as_deref(), Some("image/jpeg"));
        // Tanpa participant = pesan di private chat, jadi sender tetap sumber
        // aslinya — bukan `""`.
        assert_eq!(q.sender, "628111@s.whatsapp.net");
    }

    #[test]
    fn every_participant_carries_its_own_store_entry() {
        let src = plugin_src_with(r#"{"users":{"628111":{"limit":3},"628222":{"limit":7}}}"#);
        src.rosters.lock().unwrap().put(
            "120363@g.us",
            snapshot_of(GroupRoster {
                participants: vec![
                    Participant {
                        jid: "628111@s.whatsapp.net".into(),
                        is_admin: false,
                        is_owner: false,
                        data: Value::Null,
                    },
                    Participant {
                        jid: "628222@s.whatsapp.net".into(),
                        is_admin: false,
                        is_owner: false,
                        data: Value::Null,
                    },
                ],
                is_bot_admin: false,
            }),
        );
        let mut c = Ctx {
            chat: "120363@g.us".into(),
            ..Default::default()
        };
        src.fill(&mut c);
        let limits: Vec<i64> = c
            .roster
            .participants
            .iter()
            .map(|p| p.data.get("limit").and_then(Value::as_i64).unwrap())
            .collect();
        assert_eq!(limits, vec![3, 7]);
    }

    #[test]
    fn sessions_survive_across_fills_because_they_are_shared() {
        let src = plugin_src_with("{}");
        let mut a = Ctx {
            chat: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        src.fill(&mut a);
        a.session_set("game", serde_json::json!({"skor": 10}));
        a.session_update("game", |v| {
            v["skor"] = serde_json::json!(11);
        });

        let mut b = Ctx {
            chat: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        src.fill(&mut b);
        assert_eq!(
            b.session_get("game").and_then(|v| v["skor"].as_i64()),
            Some(11),
            "pesan kedua harus melihat state pesan pertama"
        );
        assert!(b.session_take("game").is_some());
        assert!(b.session_take("game").is_none());
    }

    #[test]
    fn default_http_never_reaches_the_network() {
        let src = plugin_src_with("{}");
        let mut c = Ctx::default();
        src.fill(&mut c);
        assert!(c.http.get("https://example.invalid").is_err());
        assert!(c.http.post_json("https://example.invalid", &Value::Null).is_err());
    }

    /// `Sock` uji: hanya `ubah_peserta` yang dicatat, sisanya error.
    #[derive(Debug, Default)]
    struct SockUji {
        aksi: Mutex<Vec<(String, Vec<String>, oktz_plugin_api::AksiPeserta)>>,
        /// Metadata grup yang dikembalikan [`Sock::metadata_grup`]; `None` =
        /// panggilan itu error, bukan metadata kosong.
        meta: Option<Value>,
        /// Grup yang diminta keluar lewat [`Sock::keluar_grup`].
        keluar: Mutex<Vec<String>>,
        /// `ubah_peserta` gagal — menguji fallback saat kick ditolak server.
        kick_fails: bool,
    }

    macro_rules! sock_tolak {
        ($($nama:ident($($arg:ident: $tipe:ty),* $(,)?) -> $hasil:ty;)*) => {
            $(
                fn $nama(&self, $($arg: $tipe),*) -> Result<$hasil, oktz_plugin_api::SockError> {
                    $(let _ = $arg;)*
                    Err(oktz_plugin_api::SockError::baru("sock uji"))
                }
            )*
        };
    }

    impl Sock for SockUji {
        fn ubah_peserta(
            &self,
            jid: &str,
            peserta: &[String],
            aksi: oktz_plugin_api::AksiPeserta,
        ) -> Result<(), oktz_plugin_api::SockError> {
            self.aksi
                .lock()
                .unwrap()
                .push((jid.to_string(), peserta.to_vec(), aksi));
            if self.kick_fails {
                return Err(oktz_plugin_api::SockError::baru("kick ditolak"));
            }
            Ok(())
        }

        fn metadata_grup(&self, jid: &str) -> Result<Value, oktz_plugin_api::SockError> {
            match &self.meta {
                Some(m) => Ok(m.clone()),
                None => Err(oktz_plugin_api::SockError::baru(format!(
                    "sock uji tanpa metadata ({jid})"
                ))),
            }
        }

        fn keluar_grup(&self, jid: &str) -> Result<(), oktz_plugin_api::SockError> {
            self.keluar.lock().unwrap().push(jid.to_string());
            Ok(())
        }

        sock_tolak! {
            ada_whatsapp(nomor: &[String]) -> Vec<oktz_plugin_api::AdaWhatsapp>;
            foto_profil(jid: &str, tipe: &str) -> Option<String>;
            semua_grup() -> Value;
            set_foto_profil(jid: &str, isi: &[u8]) -> ();
            kode_undangan(jid: &str) -> String;
            info_undangan(kode: &str) -> Value;
            buat_grup(subjek: &str, peserta: &[String]) -> Value;
            terima_undangan(kode: &str) -> String;
            tandai_dibaca(pesan: &[Value]) -> ();
            presence(jenis: &str, jid: &str) -> ();
            status_teks(jid: &str) -> Value;
            nama_kontak(jid: &str) -> String;
            profil_bisnis(jid: &str) -> Option<Value>;
        }
    }

    /// Metadata grup bentuk Baileys untuk [`SockUji`].
    fn meta_grup(peserta: Value) -> Value {
        json!({
            "subject": "Grup Uji",
            "descOwner": "628999@s.whatsapp.net",
            "owner": "628999@s.whatsapp.net",
            "participants": peserta,
        })
    }

    fn update_grup(aksi: AksiGrup, jid: &str) -> UpdateGrup {
        UpdateGrup {
            id: GROUP.to_string(),
            aksi,
            peserta: vec![PesertaGrup {
                jid: jid.to_string(),
                nomor: None,
                lid: None,
            }],
            pelaku: None,
            mode_lid: false,
        }
    }

    fn pipa_grup(
        nama: &str,
        isi_store: &str,
        meta: Value,
    ) -> (Arc<Pipeline>, Arc<RecordingSender>, Arc<SockUji>) {
        let ctx = StoreContext::new(store_with(nama, isi_store));
        let sender = Arc::new(RecordingSender::default());
        let sock = Arc::new(SockUji {
            meta: Some(meta),
            ..Default::default()
        });
        let p = Arc::new(
            Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
                .with_sock(sock.clone()),
        );
        (p, sender, sock)
    }

    #[tokio::test]
    async fn update_grup_add_mengirim_welcome_dengan_mention_peserta() {
        let (p, sender, _sock) = pipa_grup(
            "grup-welcome",
            r#"{"groups": {"120363@g.us": {"welcome": true}}, "settings": {}}"#,
            meta_grup(json!([{"id": MEMBER, "admin": null}])),
        );
        p.tangani_update_grup(&update_grup(AksiGrup::Add, MEMBER))
            .await
            .unwrap();

        let sent = sender.texts();
        assert_eq!(sent.len(), 1, "satu sapaan per peserta: {sent:?}");
        assert!(sent[0].contains("WELCOME"), "{}", sent[0]);
        assert!(sent[0].contains("628111"), "{}", sent[0]);
        let mentah = sender.sent.lock().unwrap()[0].1.clone();
        assert_eq!(
            ctx_of(&mentah).map(|c| c.mentioned_jid.clone()).unwrap_or_default(),
            vec![MEMBER.to_string()],
            "peserta baru harus di-mention, bukan cuma ditulis"
        );
    }

    #[tokio::test]
    async fn update_grup_remove_mengirim_goodbye() {
        let (p, sender, _sock) = pipa_grup(
            "grup-goodbye",
            r#"{"groups": {"120363@g.us": {"goodbye": true}}, "settings": {}}"#,
            meta_grup(json!([{"id": MEMBER, "admin": null}])),
        );
        p.tangani_update_grup(&update_grup(AksiGrup::Remove, MEMBER))
            .await
            .unwrap();

        let sent = sender.texts();
        assert_eq!(sent.len(), 1, "satu ucapan per peserta: {sent:?}");
        assert!(sent[0].contains("SAYONARA"), "{}", sent[0]);
    }

    #[tokio::test]
    async fn update_grup_tanpa_catatan_menulis_default_tanpa_menyapa() {
        let (p, sender, _sock) = pipa_grup(
            "grup-default",
            r#"{"groups": {}, "settings": {}}"#,
            meta_grup(json!([{"id": MEMBER, "admin": null}])),
        );
        p.tangani_update_grup(&update_grup(AksiGrup::Add, MEMBER))
            .await
            .unwrap();

        // `config.js:186-187` memasang `defaultEnabled: false`, jadi grup baru
        // dicatat mati — bukan langsung menyapa.
        let grup = p.ctx.group_entry(GROUP);
        assert_eq!(grup.get("welcome"), Some(&Value::Bool(false)));
        assert_eq!(grup.get("goodbye"), Some(&Value::Bool(false)));
        assert_eq!(grup.get("leave"), Some(&Value::Bool(false)));
        assert!(sender.texts().is_empty(), "default false = tidak menyapa");
    }

    #[tokio::test]
    async fn update_grup_mati_atau_tipe_lain_tidak_mengirim() {
        for (nama, isi) in [
            ("grup-off", r#"{"groups": {"120363@g.us": {"welcome": false}}}"#),
            (
                "grup-tipe",
                r#"{"groups": {"120363@g.us": {"welcome": true}}, "settings": {"welcomeType": 2}}"#,
            ),
        ] {
            let (p, sender, _sock) = pipa_grup(
                nama,
                isi,
                meta_grup(json!([{"id": MEMBER, "admin": null}])),
            );
            p.tangani_update_grup(&update_grup(AksiGrup::Add, MEMBER))
                .await
                .unwrap();
            assert!(
                sender.texts().is_empty(),
                "{nama}: tidak boleh ada pesan terkirim"
            );
        }
    }

    #[tokio::test]
    async fn anticulik_mengeluarkan_bot_yang_diculik_orang_asing() {
        let (p, sender, sock) = pipa_grup(
            "grup-anticulik",
            r#"{"groups": {"120363@g.us": {"anticulik": "on"}}}"#,
            meta_grup(json!([{"id": BOT_JID, "admin": null}])),
        );
        let mut u = update_grup(AksiGrup::Add, BOT_JID);
        u.pelaku = Some("628777@s.whatsapp.net".to_string());
        p.tangani_update_grup(&u).await.unwrap();

        assert!(
            sender.texts().iter().any(|t| t.contains("Anti Culik")),
            "{:?}",
            sender.texts()
        );
        assert_eq!(
            sock.keluar.lock().unwrap().clone(),
            vec![GROUP.to_string()],
            "harus keluar setelah memberi tahu grup"
        );
    }

    #[tokio::test]
    async fn anticulik_tidak_keluar_kalau_owner_yang_mengundang() {
        let ctx = StoreContext::new(store_with(
            "grup-anticulik-owner",
            r#"{"groups": {"120363@g.us": {"anticulik": "on"}}}"#,
        ));
        let sender = Arc::new(RecordingSender::default());
        let sock = Arc::new(SockUji {
            meta: Some(meta_grup(json!([{"id": BOT_JID, "admin": null}]))),
            ..Default::default()
        });
        let p = Arc::new(
            Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
                .with_identity(Identity {
                    owners: ["628777".to_string()].into_iter().collect(),
                    ..Default::default()
                })
                .with_sock(sock.clone()),
        );
        let mut u = update_grup(AksiGrup::Add, BOT_JID);
        u.pelaku = Some("628777@s.whatsapp.net".to_string());
        p.tangani_update_grup(&u).await.unwrap();

        assert!(
            sock.keluar.lock().unwrap().is_empty(),
            "owner mengundang bot: jangan keluar"
        );
        assert!(
            sender.texts().iter().all(|t| !t.contains("Anti Culik")),
            "undangan owner bukan culikan: {:?}",
            sender.texts()
        );
    }

    #[tokio::test]
    async fn sewa_menolak_grup_tak_terdaftar() {
        let (p, sender, sock) = pipa_grup(
            "grup-sewa",
            r#"{"settings": {"sewa": {"enabled": true, "groups": {}}}}"#,
            meta_grup(json!([{"id": BOT_JID, "admin": null}])),
        );
        p.tangani_update_grup(&update_grup(AksiGrup::Add, BOT_JID))
            .await
            .unwrap();

        assert!(
            sender.texts().iter().any(|t| t.contains("sᴇᴡᴀʙᴏᴛ")),
            "{:?}",
            sender.texts()
        );
        assert_eq!(sock.keluar.lock().unwrap().clone(), vec![GROUP.to_string()]);
    }

    #[tokio::test]
    async fn sewa_terdaftar_mendapat_salam_bukan_dikeluarkan() {
        let (p, sender, sock) = pipa_grup(
            "grup-sewa-ok",
            r#"{"settings": {"sewa": {"enabled": true, "groups": {"120363@g.us": {"isLifetime": true}}}}}"#,
            meta_grup(json!([{"id": BOT_JID, "admin": null}])),
        );
        let mut u = update_grup(AksiGrup::Add, BOT_JID);
        u.pelaku = Some("628777@s.whatsapp.net".to_string());
        p.tangani_update_grup(&u).await.unwrap();

        assert!(
            sender.texts().iter().any(|t| t.contains("sᴀʟᴀᴍ ᴋᴇɴᴀʟ")),
            "{:?}",
            sender.texts()
        );
        assert!(sock.keluar.lock().unwrap().is_empty(), "grup sewa: tetap di dalam");
    }

    #[tokio::test]
    async fn update_grup_menghormati_lid_lewat_metadata() {        // Wire memberi LID; metadata grup yang tahu nomor teleponnya. Yang
        // di-mention harus nomor, sama seperti `resolveAnyLidToJid`.
        let (p, sender, _sock) = pipa_grup(
            "grup-lid",
            r#"{"groups": {"120363@g.us": {"welcome": true}}}"#,
            meta_grup(json!([{"id": "628111@s.whatsapp.net", "lid": "12345@lid", "admin": null}])),
        );
        let mut u = update_grup(AksiGrup::Add, "12345@lid");
        u.mode_lid = true;
        p.tangani_update_grup(&u).await.unwrap();

        let mentah = sender.sent.lock().unwrap()[0].1.clone();
        assert_eq!(
            ctx_of(&mentah).map(|c| c.mentioned_jid.clone()).unwrap_or_default(),
            vec![MEMBER.to_string()],
            "mention harus nomor telepon, bukan LID"
        );
    }

    #[test]
    fn ctx_berkas_default_palsu_dan_produksi_bisa_nyata() {
        let akar = std::env::temp_dir().join(format!(
            "oktz-berkas-{}",
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&akar).unwrap();
        let ctx = StoreContext::new(store_with(
            "berkas",
            r#"{"users":{},"groups":{},"settings":{}}"#,
        ));
        let nyata = Arc::new(
            Pipeline::new(
                ctx.clone(),
                Arc::new(RecordingSender::default()),
                BOT,
                ParseOptions::default(),
            )
            .with_berkas(Arc::new(oktz_plugin_api::berkas::BerkasNyata::baru(&akar))),
        );
        let mut c = Ctx {
            chat: GROUP.to_string(),
            sender: MEMBER.to_string(),
            ..Ctx::default()
        };
        nyata.plugin_src.fill(&mut c);
        c.berkas.tulis_teks("uji.txt", "halo").unwrap();
        assert_eq!(
            std::fs::read_to_string(akar.join("uji.txt")).unwrap(),
            "halo",
            "BerkasNyata harus menulis ke disk"
        );

        let palsu = Arc::new(Pipeline::new(
            ctx,
            Arc::new(RecordingSender::default()),
            BOT,
            ParseOptions::default(),
        ));
        palsu.plugin_src.fill(&mut c);
        c.berkas.tulis_teks("palsu.txt", "halo").unwrap();
        assert!(
            !akar.join("palsu.txt").exists(),
            "default pipeline tidak boleh menyentuh disk"
        );
    }

    #[test]
    fn ctx_sock_mengikuti_sumber_yang_dipasang() {        let mut src = plugin_src_with("{}");
        src.sock = Arc::new(SockUji::default());
        let mut c = Ctx::default();
        src.fill(&mut c);
        assert_eq!(
            c.sock().ada_whatsapp(&[]).unwrap_err().pesan,
            "sock uji",
            "Ctx harus memakai sock dari sumber, bukan SockMati"
        );
    }

    #[tokio::test]
    async fn keputusan_kick_menghapus_mengkick_lalu_membalas() {
        let sender = Arc::new(RecordingSender::default());
        let sock = Arc::new(SockUji::default());
        let ctx = StoreContext::new(store_with("kick-path", "{}"));
        let p = Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
            .with_sock(sock.clone());

        let m = inbound("https://contoh.invalid", MEMBER, GROUP, true, "K1");
        let caller = p.caller_of(&m);
        p.execute_group_decision(
            group::Decision::KickAndReply("dikick".into()),
            &m,
            &caller,
            MEMBER,
        )
        .await
        .expect("keputusan kick harus jalan tanpa menggagalkan pipeline");

        assert_eq!(
            sender.deleted(),
            vec![(GROUP.to_string(), "K1".to_string(), MEMBER.to_string())],
            "pesan dihapus lebih dulu"
        );
        let aksi = sock.aksi.lock().unwrap().clone();
        assert_eq!(
            aksi,
            vec![(
                GROUP.to_string(),
                vec![MEMBER.to_string()],
                oktz_plugin_api::AksiPeserta::Keluar
            )],
            "kick lewat sock.ubah_peserta setelah delete"
        );
        assert!(
            sender.texts().iter().any(|t| t.contains("dikick")),
            "balasan terkirim setelah kick: {:?}",
            sender.texts()
        );
    }

    // ---------- group-protection: identitas LID/PN + kegagalan aksi ----------

    /// Pesan grup dari pengirim LID yang wire-nya menyertakan pasangan PN
    /// (`participant_pn`, addressing mode LID).
    fn inbound_lid(body: &str, lid: &str, pn: &str, chat: &str, id: &str) -> InboundMessage {
        InboundMessage::builder()
            .message(Arc::new(wa::Message {
                conversation: Some(body.to_string()),
                ..Default::default()
            }))
            .info(Arc::new(MessageInfo {
                source: MessageSource {
                    chat: chat.parse().unwrap(),
                    sender: lid.parse().unwrap(),
                    sender_alt: Some(pn.parse().unwrap()),
                    is_from_me: false,
                    is_group: true,
                    ..Default::default()
                },
                id: id.to_string(),
                ..Default::default()
            }))
            .build()
    }

    #[test]
    fn group_caller_memakai_pn_dari_participant_pn() {
        // JS: `m.sender` = participant yang sudah di-resolve ke PN
        // (`serialize.js:694-748`), lalu `m.key.participant` ditulis ulang jadi
        // `m.sender` (:756-758) — jadi keduanya sama di gate.
        let p = Pipeline::new(
            StoreContext::new(store_with("lid-sender", "{}")),
            Arc::new(RecordingSender::default()),
            BOT,
            ParseOptions::default(),
        );
        let m = inbound_lid(
            "halo",
            "9998887771:12@lid",
            "628111@s.whatsapp.net",
            GROUP,
            "L1",
        );
        let caller = p.caller_of(&m);
        let gcaller = p.group_caller_of(
            &m,
            &caller,
            "halo",
            &snapshot_of(GroupRoster::default()),
        );
        assert_eq!(gcaller.sender, "628111@s.whatsapp.net");
        assert_eq!(gcaller.participant.as_deref(), Some("628111@s.whatsapp.net"));
    }

    #[test]
    fn admin_grup_lid_dikenali_lewat_bentuk_pn() {
        // Roster grup addressing LID: `id` = LID, `phone_number` = PN.
        // `_participantNums` (serialize.js:101-116) mengumpulkan KETIGA
        // bentuk, jadi admin tetap dikenali walau pengirimnya PN.
        let snapshot = RosterSnapshot {
            roster: GroupRoster::default(),
            participants: vec![oktz_router::group::Participant {
                id: "9998887771:12@lid".into(),
                lid: "9998887771:12@lid".into(),
                phone_number: "628222@s.whatsapp.net".into(),
                admin: true,
            }],
            available: true,
        };
        let p = Pipeline::new(
            StoreContext::new(store_with("lid-admin", "{}")),
            Arc::new(RecordingSender::default()),
            BOT,
            ParseOptions::default(),
        );
        let m = inbound_lid(
            "halo",
            "9998887771:12@lid",
            "628222@s.whatsapp.net",
            GROUP,
            "L2",
        );
        let caller = p.caller_of(&m);
        let gcaller = p.group_caller_of(&m, &caller, "halo", &snapshot);
        assert!(
            gcaller.is_admin,
            "admin grup LID harus dikenali lewat phone_number"
        );
    }

    #[tokio::test]
    async fn gagal_hapus_membatalkan_kick_dan_balasan() {
        // JS: error delete naik ke `catch` gate → `return false` → tanpa kick,
        // tanpa balasan (`group-protection.js:1346` + `catch` :1363).
        let sender = Arc::new(RecordingSender {
            delete_fails: true,
            ..Default::default()
        });
        let sock = Arc::new(SockUji::default());
        let ctx = StoreContext::new(store_with("del-fail", "{}"));
        let p = Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
            .with_sock(sock.clone());

        let m = inbound("https://contoh.invalid", MEMBER, GROUP, true, "D1");
        let caller = p.caller_of(&m);
        let out = p
            .execute_group_decision(
                group::Decision::KickAndReplyFallback {
                    kick: "MODE-KICK".into(),
                    fallback: "MODE-REMOVE".into(),
                },
                &m,
                &caller,
                MEMBER,
            )
            .await
            .expect("keputusan tidak boleh menggagalkan pipeline");
        assert!(matches!(out, Outcome::GroupStopped));
        assert!(
            sock.aksi.lock().unwrap().is_empty(),
            "kick tidak boleh jalan setelah delete gagal"
        );
        assert!(
            sender.texts().is_empty(),
            "tidak boleh ada balasan setelah delete gagal: {:?}",
            sender.texts()
        );
    }

    #[tokio::test]
    async fn gagal_kick_mengirim_pesan_mode_remove() {
        // JS: `handleAntilinkGc/All` menangkap error kick lalu mengirim pesan
        // mode-remove (`group-protection.js:1917-1924`).
        let sender = Arc::new(RecordingSender::default());
        let sock = Arc::new(SockUji {
            kick_fails: true,
            ..Default::default()
        });
        let ctx = StoreContext::new(store_with("kick-fail", "{}"));
        let p = Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
            .with_sock(sock.clone());

        let m = inbound("https://contoh.invalid", MEMBER, GROUP, true, "K2");
        let caller = p.caller_of(&m);
        let out = p
            .execute_group_decision(
                group::Decision::KickAndReplyFallback {
                    kick: "MODE-KICK".into(),
                    fallback: "MODE-REMOVE".into(),
                },
                &m,
                &caller,
                MEMBER,
            )
            .await
            .expect("keputusan tidak boleh menggagalkan pipeline");
        assert!(matches!(out, Outcome::Sent));
        let texts = sender.texts();
        assert!(
            texts.iter().any(|t| t.contains("MODE-REMOVE")),
            "kick gagal → balas pesan mode-remove: {texts:?}"
        );
        assert!(
            !texts.iter().any(|t| t.contains("MODE-KICK")),
            "pesan mode-kick tidak boleh terkirim saat kick gagal: {texts:?}"
        );
    }

    #[tokio::test]
    async fn gagal_kick_pada_antilink_polos_tidak_balasan() {
        // JS: `handleAntilink` (:1349) TIDAK menangkap error kick → error naik
        // ke `catch` luar → tanpa balasan sama sekali.
        let sender = Arc::new(RecordingSender::default());
        let sock = Arc::new(SockUji {
            kick_fails: true,
            ..Default::default()
        });
        let ctx = StoreContext::new(store_with("kick-fail-polos", "{}"));
        let p = Pipeline::new(ctx, sender.clone(), BOT, ParseOptions::default())
            .with_sock(sock.clone());

        let m = inbound("https://contoh.invalid", MEMBER, GROUP, true, "K3");
        let caller = p.caller_of(&m);
        let out = p
            .execute_group_decision(
                group::Decision::KickAndReply("MODE-KICK".into()),
                &m,
                &caller,
                MEMBER,
            )
            .await
            .expect("keputusan tidak boleh menggagalkan pipeline");
        assert!(matches!(out, Outcome::GroupStopped));
        assert!(
            sender.texts().is_empty(),
            "kick gagal tanpa fallback → tanpa balasan: {:?}",
            sender.texts()
        );
    }
}
