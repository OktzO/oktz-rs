//! `plugins/anime/autoanimewinbu.js` — manajer auto-upload anime winbu.net.
//!
//! ## Penyimpangan
//!
//! - **Daemon auto-check** (`startAutoCheck`/`stopAutoCheck`/`runCheck`,
//!   `src/lib/auto-anime.js`) menjalankan `setInterval` di proses Node yang
//!   scrape winbu.net lalu kirim pesan PROAKTIF ke grup target. API plugin
//!   tidak punya penjadwal dan `Reply` tidak bisa mengirim ke chat lain, jadi
//!   penanda "sedang berjalan" dicatat di `ctx.sessions` (pola
//!   `owner/startschedule.rs`) dan sub-command `cek`/`check`/`list` — yang
//!   intinya `runCheck`/`getOngoingAnimeList` (scrape + broadcast) — jatuh ke
//!   blok `catch` JS (`te(...)` = error template).
//! - **State & riwayat** disimpan lewat `ctx.berkas()` di jalur JS apa adanya:
//!   `src/data/autoanime_winbu_state.json` dan
//!   `src/data/autoanime_winbu_sent.json` (batas `SENT_MAX` 5000, buang
//!   tertua).
//! - **Interactive buttons** (`quick_reply`) memakai `Reply::interactive` +
//!   `InteractiveButton::quick_reply` (padanan wire `interactiveButtons`).
//! - **`sent.size`** — JS memakai `Set`; di sini array JSON (urutan insertion
//!   sama seperti iterasi `Set.keys()` untuk pemotongan 5000).

use oktz_plugin_api::{Ctx, Reply, plugin};
use oktz_plugin_api::registry::{CardContext, InteractiveButton};
use serde_json::{Value, json};

use crate::common::{error_template, js_truthy};

/// Jalur state & riwayat — persis `auto-anime.js:8-10`, relatif ke direktori
/// kerja (`ctx.berkas()` berakar ke sana).
const STATE_PATH: &str = "src/data/autoanime_winbu_state.json";
const SENT_PATH: &str = "src/data/autoanime_winbu_sent.json";
const DATA_DIR: &str = "src/data";

/// `SENT_MAX` (`auto-anime.js:11`).
const SENT_MAX: usize = 5000;

/// Kunci penanda daemon berjalan di `ctx.sessions` — padanan `isRunning()`.
const RUNNING_KEY: &str = "autoanimewinbu:running";

/// State default (`auto-anime.js:47`).
fn state_default() -> Value {
    json!({ "enabled": false, "groups": [], "interval": 5 })
}

/// `loadState` (`auto-anime.js:45-52`).
fn load_state(ctx: &Ctx) -> Value {
    if !ctx.berkas().ada(STATE_PATH) {
        return state_default();
    }
    ctx.berkas()
        .baca_teks(STATE_PATH)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(state_default)
}

/// `saveState` (`auto-anime.js:54-59`) — `JSON.stringify(state, null, 2)`;
/// kegagalan diabaikan seperti `catch {}`.
fn save_state(ctx: &Ctx, state: &Value) {
    let _ = ctx.berkas().buat_folder(DATA_DIR);
    let pretty = serde_json::to_string_pretty(state).unwrap_or_default();
    let _ = ctx.berkas().tulis_teks(STATE_PATH, &pretty);
}

/// `loadSent` (`auto-anime.js:20-27`) — daftar urut insertion.
fn load_sent(ctx: &Ctx) -> Vec<String> {
    if !ctx.berkas().ada(SENT_PATH) {
        return Vec::new();
    }
    ctx.berkas()
        .baca_teks(SENT_PATH)
        .ok()
        .and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok())
        .unwrap_or_default()
}

/// `saveSent` (`auto-anime.js:29-43`) — buang tertua sampai <= SENT_MAX.
fn save_sent(ctx: &Ctx, mut set: Vec<String>) {
    if set.len() > SENT_MAX {
        let excess = set.len() - SENT_MAX;
        set.drain(..excess);
    }
    let _ = ctx.berkas().buat_folder(DATA_DIR);
    let _ = ctx.berkas().tulis_teks(SENT_PATH, &serde_json::to_string(&set).unwrap_or_default());
}

/// `isRunning()` — penanda memori proses.
fn is_running(ctx: &Ctx) -> bool {
    ctx.session_get(RUNNING_KEY).is_some()
}

/// Grup target: `state.groups || []`.
fn groups(state: &Value) -> Vec<String> {
    state
        .get("groups")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// `state.interval || 5` — `||` JS membuat interval `0` ikut default.
fn interval(state: &Value) -> i64 {
    state.get("interval").and_then(Value::as_i64).filter(|n| *n != 0).unwrap_or(5)
}

/// `x ? a : b` untuk teks.
fn jika(kondisi: bool, ya: &str, tidak: &str) -> String {
    if kondisi { ya.to_string() } else { tidak.to_string() }
}

/// `(typeof args === 'string' ? args : '').replace(/^(addgrup|addgroup)\s*/i,
/// '').trim()` — buang awalan sub-command (case-insensitive) lalu trim.
fn strip_sub(text: &str, sub: &str) -> String {
    if text.len() >= sub.len() && text[..sub.len()].eq_ignore_ascii_case(sub) {
        text[sub.len()..].trim().to_string()
    } else {
        text.trim().to_string()
    }
}

#[plugin(
    name = "autoanimewinbu",
    alias = ["aaw", "autoanime"],
    category = "anime",
    description = "Auto upload ongoing anime & donghua dari winbu.net (720p Pixeldrain)",
    usage = ".autoanimewinbu <start|stop|status|cek|list|reset|addgrup|delgrup>",
    example = ".autoanimewinbu start",
    cooldown = 10,
    energi = 0,
    is_owner = true,
    is_group = false,
    is_private = false
)]
pub fn autoanimewinbu(ctx: Ctx) -> Reply {
    // `const sub = m.text` — di framework JS ini token pertama setelah command
    // (switch(sub) mencocokkan persis 'start' dst), dan `args` adalah sisanya.
    let sub = ctx.args.first().map(String::as_str).unwrap_or("");
    let sisa_args = ctx.args.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
    let state = load_state(&ctx);

    match sub {
        "start" => {
            if is_running(&ctx) {
                return Reply::text("⚠️ AutoAnime sudah berjalan!");
            }

            let grup = groups(&state);
            if grup.is_empty() {
                return Reply::text(format!(
                    "❌ Belum ada grup target!\n\n\
                     > Tambahkan grup dulu:\n\
                     > `{prefix}autoanimewinbu addgrup` (di grup target)\n\
                     > `{prefix}autoanimewinbu addgrup 120363xxx@g.us`",
                    prefix = ctx.prefix
                ));
            }

            let interval = interval(&state);
            ctx.session_set(RUNNING_KEY, json!(true));
            let mut baru = state.clone();
            baru["enabled"] = json!(true);
            save_state(&ctx, &baru);

            Reply::interactive(format!(
                "✅ *ᴀᴜᴛᴏ ᴀɴɪᴍᴇ sᴛᴀʀᴛᴇᴅ*\n\n\
                 > 📲 Grup target: *{}*\n\
                 > ⏱️ Interval: *{} menit*\n\
                 > 🎞️ Filter: *Pixeldrain 720p+*\n\
                 > ⏰ Max age: *24 jam*\n\n\
                 Pengecekan pertama dimulai...",
                grup.len(),
                interval
            ))
            .with_buttons(vec![
                InteractiveButton::quick_reply(
                    "📊 Status",
                    format!("{prefix}autoanimewinbu status", prefix = ctx.prefix),
                ),
                InteractiveButton::quick_reply(
                    "🛑 Stop",
                    format!("{prefix}autoanimewinbu stop", prefix = ctx.prefix),
                ),
            ])
            .with_context(CardContext::new().quote_trigger())
        }

        "stop" => {
            ctx.session_take(RUNNING_KEY);
            let mut baru = state.clone();
            baru["enabled"] = json!(false);
            save_state(&ctx, &baru);
            Reply::text("🛑 *AutoAnime dihentikan*")
        }

        "status" => {
            let sent = load_sent(&ctx);
            let grup = groups(&state);

            let mut txt = format!(
                "📊 *ᴀᴜᴛᴏ ᴀɴɪᴍᴇ sᴛᴀᴛᴜs*\n\n\
                 > 🔄 Status: *{}*\n\
                 > 💾 Auto-start: *{}*\n\
                 > 📋 Sudah terkirim: *{}* episode\n\
                 > ⏱️ Interval: *{} menit*\n\
                 > 📲 Grup target: *{}*\n",
                jika(is_running(&ctx), "🟢 ON", "🔴 OFF"),
                jika(js_truthy(state.get("enabled")), "Ya", "Tidak"),
                sent.len(),
                interval(&state),
                grup.len(),
            );

            if !grup.is_empty() {
                txt.push_str("\n*Grup:*\n");
                for (i, g) in grup.iter().enumerate() {
                    txt.push_str(&format!("> {}. `{}`\n", i + 1, g));
                }
            }

            Reply::text(txt)
        }

        "cek" | "check" => {
            if !is_running(&ctx) {
                ctx.session_set(RUNNING_KEY, json!(true));
            }
            // `runCheck()` (scrape winbu.net + kirim proaktif ke grup) tidak
            // tersedia di API plugin — jatuh ke blok `catch` JS.
            Reply::sequence_text([
                "🔍 Mengecek anime terbaru...".to_string(),
                error_template(&ctx),
            ])
        }

        "list" => {
            // `getOngoingAnimeList()` (scrape winbu.net) tidak tersedia —
            // jatuh ke blok `catch` JS.
            Reply::sequence_text([
                "📺 Mengambil daftar anime...".to_string(),
                error_template(&ctx),
            ])
        }

        "reset" => {
            let count = load_sent(&ctx).len();
            save_sent(&ctx, Vec::new());
            Reply::text(format!(
                "✅ Reset! *{}* episode dihapus dari riwayat.\n> Semua episode bisa terkirim ulang.",
                count
            ))
        }

        "addgrup" | "addgroup" => {
            let mut grup_id = strip_sub(&sisa_args, if sub == "addgrup" { "addgrup" } else { "addgroup" });

            if grup_id.is_empty() && ctx.is_group {
                grup_id = ctx.chat.clone();
            }

            if grup_id.is_empty() || !grup_id.contains("@g.us") {
                return Reply::text(format!(
                    "❌ ID grup tidak valid\n\n\
                     > Gunakan di dalam grup, atau:\n\
                     > `{prefix}autoanimewinbu addgrup 120363xxx@g.us`",
                    prefix = ctx.prefix
                ));
            }

            let mut grup = groups(&state);
            if grup.iter().any(|g| g == &grup_id) {
                return Reply::text("⚠️ Grup sudah ada di daftar target");
            }

            grup.push(grup_id.clone());
            let mut baru = state.clone();
            baru["groups"] = json!(grup);
            let total = baru["groups"].as_array().map(Vec::len).unwrap_or(0);
            save_state(&ctx, &baru);
            Reply::text(format!(
                "✅ Grup `{grup_id}` ditambahkan ke target\n> Total: *{total}* grup"
            ))
        }

        "delgrup" | "delgroup" => {
            let mut grup_id = strip_sub(&sisa_args, if sub == "delgrup" { "delgrup" } else { "delgroup" });

            if grup_id.is_empty() && ctx.is_group {
                grup_id = ctx.chat.clone();
            }

            let mut grup = groups(&state);
            let Some(idx) = grup.iter().position(|g| g == &grup_id) else {
                return Reply::text("❌ Grup tidak ditemukan di daftar target");
            };

            grup.remove(idx);
            let mut baru = state.clone();
            baru["groups"] = json!(grup);
            let total = baru["groups"].as_array().map(Vec::len).unwrap_or(0);
            save_state(&ctx, &baru);
            Reply::text(format!(
                "✅ Grup `{grup_id}` dihapus dari target\n> Sisa: *{total}* grup"
            ))
        }

        "interval" => {
            let rest = strip_sub(&sisa_args, "interval");
            let mins = crate::common::parse_int(&rest);

            let Some(mins) = mins.filter(|m| (1..=60).contains(m)) else {
                return Reply::text(format!(
                    "❌ Interval harus 1-60 menit\n\n> Contoh: `{prefix}autoanimewinbu interval 10`",
                    prefix = ctx.prefix
                ));
            };

            let mut baru = state.clone();
            baru["interval"] = json!(mins);
            save_state(&ctx, &baru);

            // JS me-restart interval saat berjalan; penanda memori tidak
            // berubah.
            Reply::text(format!("✅ Interval diubah ke *{mins} menit*"))
        }

        _ => {
            let running = is_running(&ctx);
            Reply::interactive(format!(
                "🎬 *ᴀᴜᴛᴏ ᴀɴɪᴍᴇ ᴡɪɴʙᴜ*\n\n\
                 > Status: *{}*\n\n\
                 *ᴄᴏᴍᴍᴀɴᴅs:*\n\
                 > `{prefix}aaw start` — Mulai auto-check\n\
                 > `{prefix}aaw stop` — Hentikan\n\
                 > `{prefix}aaw status` — Lihat status\n\
                 > `{prefix}aaw cek` — Manual check sekarang\n\
                 > `{prefix}aaw list` — Daftar anime terbaru\n\
                 > `{prefix}aaw addgrup` — Tambah grup target\n\
                 > `{prefix}aaw delgrup` — Hapus grup target\n\
                 > `{prefix}aaw interval 10` — Ubah interval\n\
                 > `{prefix}aaw reset` — Reset riwayat terkirim",
                jika(running, "🟢 ON", "🔴 OFF"),
                prefix = ctx.prefix
            ))
            .with_buttons(vec![
                InteractiveButton::quick_reply(
                    &jika(running, "🛑 Stop", "▶️ Start"),
                    format!(
                        "{prefix}autoanimewinbu {}",
                        jika(running, "stop", "start"),
                        prefix = ctx.prefix
                    ),
                ),
                InteractiveButton::quick_reply(
                    "📊 Status",
                    format!("{prefix}autoanimewinbu status", prefix = ctx.prefix),
                ),
            ])
            .with_context(CardContext::new().quote_trigger())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::Berkas;
    use oktz_plugin_api::berkas::BerkasPalsu;
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    /// Clone Ctx secukupnya — inherent impl pada tipe asing tidak boleh.
    fn clone_ctx(ctx: &Ctx) -> Ctx {
        Ctx {
            prefix: ctx.prefix.clone(),
            command: ctx.command.clone(),
            sender: ctx.sender.clone(),
            chat: ctx.chat.clone(),
            is_group: ctx.is_group,
            is_owner: ctx.is_owner,
            text: ctx.text.clone(),
            args: ctx.args.clone(),
            berkas: ctx.berkas.clone(),
            sessions: ctx.sessions.clone(),
            ..Default::default()
        }
    }

    fn ctx_dengan(text: &str, state: Option<Value>) -> Ctx {
        let berkas = BerkasPalsu::baru();
        if let Some(s) = state {
            berkas
                .tulis_teks(STATE_PATH, &serde_json::to_string(&s).unwrap())
                .unwrap();
        }
        Ctx {
            prefix: ".".into(),
            command: "autoanimewinbu".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            is_owner: true,
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            berkas: Arc::new(berkas),
            ..Default::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn is_running_marker(sessions: &oktz_plugin_api::SessionStore) -> bool {
        sessions.lock().unwrap().get(RUNNING_KEY).is_some()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("autoanimewinbu").unwrap();
        assert_eq!(p.meta.name, "autoanimewinbu");
        assert_eq!(p.meta.alias, ["aaw", "autoanime"]);
        assert_eq!(p.meta.category, "anime");
        assert_eq!(
            p.meta.description,
            "Auto upload ongoing anime & donghua dari winbu.net (720p Pixeldrain)"
        );
        assert_eq!(
            p.meta.usage,
            ".autoanimewinbu <start|stop|status|cek|list|reset|addgrup|delgrup>"
        );
        assert_eq!(p.meta.example, ".autoanimewinbu start");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
        for a in ["aaw", "autoanime"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "autoanimewinbu");
        }
    }

    #[test]
    fn default_tampil_menu_dengan_tombol() {
        let r = autoanimewinbu(ctx_dengan("", None));
        let body = teks(&r);
        assert!(body.starts_with("🎬 *ᴀᴜᴛᴏ ᴀɴɪᴍᴇ ᴡɪɴʙᴜ*"), "{body}");
        assert!(body.contains("> Status: *🔴 OFF*"), "{body}");
        assert!(body.contains("`.aaw start` — Mulai auto-check"), "{body}");
        assert!(body.contains("`.aaw reset` — Reset riwayat terkirim"), "{body}");
        match r.without_mutations() {
            Reply::Interactive { kartu } => {
                assert_eq!(kartu.buttons.len(), 2);
                // Belum berjalan → tombol pertama Start.
                assert_eq!(kartu.buttons[0].name, "quick_reply");
                assert!(kartu.buttons[0].params.nilai().to_string().contains("▶️ Start"));
                assert!(kartu.buttons[0].params.nilai().to_string().contains("start"));
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }

    #[test]
    fn start_ditolak_tanpa_grup_target() {
        let r = autoanimewinbu(ctx_dengan("start", None));
        assert!(teks(&r).starts_with("❌ Belum ada grup target!"), "{}", teks(&r));
        assert!(teks(&r).contains("`.autoanimewinbu addgrup 120363xxx@g.us`"));
    }

    #[test]
    fn start_dengan_grup_menyalakan_state_dan_penanda() {
        let ctx = ctx_dengan(
            "start",
            Some(json!({ "enabled": false, "groups": ["120363xxx@g.us"], "interval": 5 })),
        );
        let r = autoanimewinbu(clone_ctx(&ctx));
        let body = teks(&r);
        assert!(body.starts_with("✅ *ᴀᴜᴛᴏ ᴀɴɪᴍᴇ sᴛᴀʀᴛᴇᴅ*"), "{body}");
        assert!(body.contains("Grup target: *1*"), "{body}");
        assert!(body.contains("Interval: *5 menit*"), "{body}");
        assert!(body.contains("Filter: *Pixeldrain 720p+*"), "{body}");
        assert!(body.contains("Max age: *24 jam*"), "{body}");
        assert!(is_running_marker(&ctx.sessions));
        // State enabled=true tersimpan.
        let v: Value =
            serde_json::from_str(&ctx.berkas().baca_teks(STATE_PATH).unwrap()).unwrap();
        assert_eq!(v["enabled"], true);
        // Tombol: Status + Stop.
        match r.without_mutations() {
            Reply::Interactive { kartu } => {
                assert!(kartu.buttons[0].params.nilai().to_string().contains("📊 Status"));
                assert!(kartu.buttons[1].params.nilai().to_string().contains("🛑 Stop"));
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }

    #[test]
    fn start_dua_kali_ditolak() {
        let ctx = ctx_dengan(
            "start",
            Some(json!({ "enabled": false, "groups": ["120363xxx@g.us"], "interval": 5 })),
        );
        autoanimewinbu(clone_ctx(&ctx));
        let r = autoanimewinbu(ctx);
        assert_eq!(teks(&r), "⚠️ AutoAnime sudah berjalan!");
    }

    #[test]
    fn stop_mematikan_penanda_dan_state() {
        let ctx = ctx_dengan("stop", Some(json!({ "enabled": true, "groups": ["x@g.us"], "interval": 5 })));
        ctx.sessions.lock().unwrap().insert(RUNNING_KEY.into(), json!(true));
        let r = autoanimewinbu(ctx);
        assert_eq!(teks(&r), "🛑 *AutoAnime dihentikan*");
    }

    #[test]
    fn status_menampilkan_semua_baris() {
        let ctx = ctx_dengan(
            "status",
            Some(json!({ "enabled": true, "groups": ["a@g.us", "b@g.us"], "interval": 7 })),
        );
        ctx.sessions.lock().unwrap().insert(RUNNING_KEY.into(), json!(true));
        ctx.berkas()
            .tulis_teks(SENT_PATH, &json!(["satu", "dua"]).to_string())
            .unwrap();
        let body = teks(&autoanimewinbu(ctx));
        assert!(body.starts_with("📊 *ᴀᴜᴛᴏ ᴀɴɪᴍᴇ sᴛᴀᴛᴜs*"), "{body}");
        assert!(body.contains("Status: *🟢 ON*"), "{body}");
        assert!(body.contains("Auto-start: *Ya*"), "{body}");
        assert!(body.contains("Sudah terkirim: *2* episode"), "{body}");
        assert!(body.contains("Interval: *7 menit*"), "{body}");
        assert!(body.contains("Grup target: *2*"), "{body}");
        assert!(body.contains("> 1. `a@g.us`"), "{body}");
        assert!(body.contains("> 2. `b@g.us`"), "{body}");
    }

    #[test]
    fn cek_dan_list_jatuh_ke_catch_js() {
        let r = autoanimewinbu(ctx_dengan("cek", None));
        match r.without_mutations() {
            Reply::Sequence(list) => {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0].text_body().unwrap(), "🔍 Mengecek anime terbaru...");
                assert!(list[1].text_body().unwrap().contains("☢"));
            }
            other => panic!("harus Sequence, dapat {other:?}"),
        }

        let r = autoanimewinbu(ctx_dengan("check", None));
        match r.without_mutations() {
            Reply::Sequence(list) => {
                assert_eq!(list[0].text_body().unwrap(), "🔍 Mengecek anime terbaru...");
            }
            other => panic!("harus Sequence, dapat {other:?}"),
        }

        let r = autoanimewinbu(ctx_dengan("list", None));
        match r.without_mutations() {
            Reply::Sequence(list) => {
                assert_eq!(list[0].text_body().unwrap(), "📺 Mengambil daftar anime...");
            }
            other => panic!("harus Sequence, dapat {other:?}"),
        }
    }

    #[test]
    fn reset_menghapus_riwayat() {
        let ctx = ctx_dengan("reset", None);
        ctx.berkas()
            .tulis_teks(SENT_PATH, &json!(["a", "b", "c"]).to_string())
            .unwrap();
        let r = autoanimewinbu(ctx);
        assert_eq!(
            teks(&r),
            "✅ Reset! *3* episode dihapus dari riwayat.\n> Semua episode bisa terkirim ulang."
        );
    }

    #[test]
    fn addgrup_dari_argumen_dan_dari_chat() {
        // Dari argumen.
        let r = autoanimewinbu(ctx_dengan("addgrup 120363xxx@g.us", None));
        assert_eq!(
            teks(&r),
            "✅ Grup `120363xxx@g.us` ditambahkan ke target\n> Total: *1* grup"
        );

        // Dari chat (is_group) — teks kosong setelah sub-command.
        let r = autoanimewinbu(ctx_dengan("addgroup ", None));
        assert_eq!(
            teks(&r),
            "✅ Grup `120363@g.us` ditambahkan ke target\n> Total: *1* grup"
        );

        // Duplikat.
        let r = autoanimewinbu(ctx_dengan(
            "addgrup 120363xxx@g.us",
            Some(json!({ "enabled": false, "groups": ["120363xxx@g.us"], "interval": 5 })),
        ));
        assert_eq!(teks(&r), "⚠️ Grup sudah ada di daftar target");

        // ID tidak valid.
        let r = autoanimewinbu(ctx_dengan("addgrup 120363xxx", None));
        assert!(teks(&r).starts_with("❌ ID grup tidak valid"), "{}", teks(&r));
    }

    #[test]
    fn delgrup_menghapus_dan_melaporkan_sisa() {
        let state = json!({ "enabled": false, "groups": ["a@g.us", "b@g.us"], "interval": 5 });
        let r = autoanimewinbu(ctx_dengan("delgrup a@g.us", Some(state.clone())));
        assert_eq!(
            teks(&r),
            "✅ Grup `a@g.us` dihapus dari target\n> Sisa: *1* grup"
        );
        // Tidak ada → ditolak.
        let r = autoanimewinbu(ctx_dengan("delgrup zzz@g.us", Some(state)));
        assert_eq!(teks(&r), "❌ Grup tidak ditemukan di daftar target");
    }

    #[test]
    fn interval_divalidasi_1_sampai_60() {
        let r = autoanimewinbu(ctx_dengan("interval 10", None));
        assert_eq!(teks(&r), "✅ Interval diubah ke *10 menit*");

        for teks_masuk in ["interval 0", "interval 61", "interval abc", "interval"] {
            let r = autoanimewinbu(ctx_dengan(teks_masuk, None));
            assert!(
                teks(&r).starts_with("❌ Interval harus 1-60 menit"),
                "{teks_masuk}: {}",
                teks(&r)
            );
        }
    }

    #[test]
    fn strip_sub_case_insensitive_dan_trim() {
        assert_eq!(strip_sub("addgrup  X@g.us", "addgrup"), "X@g.us");
        assert_eq!(strip_sub("ADDGRUP X@g.us", "addgrup"), "X@g.us");
        assert_eq!(strip_sub("addgroup X@g.us", "addgroup"), "X@g.us");
        assert_eq!(strip_sub("X@g.us", "addgrup"), "X@g.us");
        assert_eq!(strip_sub("interval 10", "interval"), "10");
    }

    #[test]
    fn state_default_saat_berkas_tidak_ada() {
        let ctx = ctx_dengan("", None);
        assert_eq!(
            load_state(&ctx),
            json!({ "enabled": false, "groups": [], "interval": 5 })
        );
        // interval || 5: interval 0 jatuh ke default.
        assert_eq!(interval(&json!({ "interval": 0 })), 5);
        assert_eq!(interval(&json!({ "interval": 9 })), 9);
        assert_eq!(interval(&json!({})), 5);
    }

    #[test]
    fn sent_cap_5000_membuang_tertua() {
        let ctx = ctx_dengan("reset", None);
        let banyak: Vec<String> = (0..SENT_MAX + 10).map(|i| format!("k{i}")).collect();
        save_sent(&ctx, banyak);
        let sisa = load_sent(&ctx);
        assert_eq!(sisa.len(), SENT_MAX);
        assert_eq!(sisa[0], "k10", "10 tertua harus terbuang");
        assert_eq!(sisa[sisa.len() - 1], format!("k{}", SENT_MAX + 9));
    }
}
