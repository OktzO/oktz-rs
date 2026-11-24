//! Router: pipeline pesan dari parse command sampai dispatch plugin.
//!
//! Port dari `src/handler.js` (2404 baris) dan `src/lib/middleware.js`.
//! Scope saat ini HANYA logika murni yang bisa diuji tanpa koneksi WA:
//! parsing command, permission, mode. Gate yang butuh I/O (dedup 100ms,
//! rate limiter, energi, group protection) TIDAK di sini — lihat README crate
//! dan `TODO` di tiap modul. Menambahkannya tanpa bukti hanya menambah
//! kode mati.
//!
//! Yang TIDAK bisa dilemarkan: urutan. `check_mode` jalan sebelum
//! `check_permission`, dan keduanya sebelum lookup plugin — sama seperti
//! handler.js:879 → 1709 → 1810 → 1896.

pub mod command;
pub mod gates;
pub mod group;
pub mod mode;
pub mod permission;
pub mod serialize;

pub use command::{ParseOptions, ParsedCommand, parse_command};
pub use gates::{
    AutoWorkGuard, CooldownStore, EnergiSettings, RateLimiter, charge_energi, check_cooldown,
    consume_auto_work, is_spamming,
};
pub use mode::{BotMode, DenyMode, GroupBotMode, GroupOverrides, ModeSettings};
pub use permission::{AccessGrant, Caller, DenyReason, GroupState, PermissionInput, UserState};

use oktz_plugin_api::{Ctx, PluginEntry, Reply, all_answer_handlers};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Ditandai router kalau handler plugin panik/gagal jalan (2 jalur catch_unwind).
/// Dibaca + di-reset pipeline tepat setelah `route()` balik, khusus untuk
/// tanda ✕ di UI log (oktz-wa/ui.rs). Ponytail: flag global, bukan field
/// Decision — supaya bentuk enum & 11 titik match `Decision::Handled` tak
/// tersentuh; jendela race antar pesan yang bersamaan hanya salah label
/// tampilan, bukan salah perilaku.
pub static PLUGIN_GAGAL: AtomicBool = AtomicBool::new(false);

/// Keputusan router untuk satu pesan — router yang memutuskan; ini yang
/// differentiate "diam" (drop) dari "reply".
///
/// `Eq` tidak diturunkan: `Reply` boleh membawa koordinat `f64` pada kartu
/// interaktif, dan `f64` tidak punya `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Bukan command — biarkan pipeline lain (smart trigger, game) yang
    /// mencoba. Router tidak balas apa pun.
    NotACommand,
    /// Command tapi tidak ada plugin yang cocok — senyap, seperti JS
    /// (`getPlugin` miss → `return` tanpa reply).
    NoPlugin,
    /// Ditolak permission — router kirim `reason`.
    DeniedPermission(String),
    /// Ditolak mode — `DenyMode::Afk`/`OnlyThisGroup` reply, `Silent` drop.
    DeniedMode(DenyMode),
    /// Ditolak botMode per-grup — router kirim pesan saran mode.
    DeniedGroupMode(String),
    /// Plugin jalan, mengembalikan balasan.
    Handled(Reply),
}

/// Semua yang router butuhkan untuk memutuskan. Trait ini exist supaya test
/// bisa pakai stub tanpa DB.
pub trait RouterContext {
    fn user_state(&self, sender: &str) -> UserState;
    fn group_state(&self, chat: &str) -> Option<GroupState>;
    fn group_bot_mode(&self, chat: &str) -> String;
    fn mode_settings(&self) -> ModeSettings;
    fn group_overrides(&self, chat: &str) -> Option<GroupOverrides>;
    fn capprem(&self, name: &str) -> Option<bool>;
    fn uptime_secs(&self) -> u64;
}

/// Config parse + dispatch.
pub struct Router<'a, C: RouterContext> {
    ctx: &'a C,
    parse_opts: ParseOptions,
    now_ms: i64,
}

impl<'a, C: RouterContext> Router<'a, C> {
    pub fn new(ctx: &'a C, parse_opts: ParseOptions, now_ms: i64) -> Self {
        Self {
            ctx,
            parse_opts,
            now_ms,
        }
    }

    /// Pipeline lengkap: parse → mode → lookup → permission → group mode → dispatch.
    pub async fn route(
        &self,
        caller: &mut Caller,
        body: &str,
        plugin_lookup: impl Fn(&str) -> Option<&'static PluginEntry>,
    ) -> Decision {
        let parsed = parse_command(body, &self.parse_opts);
        if !parsed.is_command {
            return Decision::NotACommand;
        }
        caller.command = parsed.command.clone();

        // 1. Mode global (handler.js:879) — SEBELUM permission.
        let overrides = if caller.is_group {
            self.ctx.group_overrides(&caller.chat)
        } else {
            None
        };
        if let Err(reason) =
            mode::check_mode(caller, &self.ctx.mode_settings(), overrides, self.now_ms)
        {
            return Decision::DeniedMode(reason);
        }

        // 2. Lookup plugin.
        let Some(entry) = plugin_lookup(&parsed.command) else {
            return Decision::NoPlugin;
        };

        let user = self.ctx.user_state(&caller.sender);

        // 3. Permission (handler.js:1810).
        let input = permission::PermissionInput {
            meta: &entry.meta,
            now_ms: self.now_ms,
            capprem: self.ctx.capprem(&entry.meta.name),
            group: if caller.is_group {
                self.ctx.group_state(&caller.chat)
            } else {
                None
            },
        };
        if let Err(reason) = permission::check(caller, &user, &input) {
            return Decision::DeniedPermission(reason.message().to_string());
        }

        // 4. botMode per-grup (handler.js:1709). Non-group: skip.
        if caller.is_group && !caller.is_owner && !caller.is_admin {
            let bot_mode = self.ctx.group_bot_mode(&caller.chat);
            if mode::group_feature_mode(caller, &entry.meta, None, &bot_mode, false).is_err() {
                let gm = mode::GroupBotMode::parse(&bot_mode);
                let msg = mode::group_mode_denied_message(
                    &gm,
                    &entry.meta.category.to_lowercase(),
                    &parsed.prefix,
                    &parsed.command,
                );
                return Decision::DeniedGroupMode(msg);
            }
        }

        let c = self.build_ctx(caller, &parsed);
        // Handler jalan di thread pool blocking — HTTP/ffmpeg/image tidak
        // menahan worker tokio (ureq timeout 20s, ffmpeg puluhan detik).
        // catch_unwind tetap DI DALAM closure: panik plugin = balasan error,
        // bukan JoinError yang telanjang.
        let dipanik = tokio::task::spawn_blocking(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (entry.handler)(c)))
        })
        .await;
        match dipanik {
            Ok(Ok(r)) => Decision::Handled(r),
            Ok(Err(_)) => {
                tracing::error!("plugin panik, ditahan di router");
                // ponytail: tanda UI ✓/✕ (oktz-wa/ui.rs) butuh tahu plugin error;
                // dibaca+direset pipeline tepat setelah route() balik.
                PLUGIN_GAGAL.store(true, std::sync::atomic::Ordering::Relaxed);
                Decision::Handled(Reply::text("❌ Plugin error, coba lagi nanti."))
            }
            Err(join) => {
                tracing::error!(error = %join, "spawn_blocking handler gagal jalan");
                PLUGIN_GAGAL.store(true, std::sync::atomic::Ordering::Relaxed);
                Decision::Handled(Reply::text("❌ Plugin error, coba lagi nanti."))
            }
        }
    }

    /// Entry point kedua: `answerHandler(m, sock)` (`src/handler.js:440-473`).
    ///
    /// Rantai answer handler dijalankan berurutan dan berhenti di yang
    /// pertama mengembalikan `Some`. Tidak ada gate permission atau mode
    /// di sini, sama seperti JS: `handleGameAnswer` dipanggil dari
    /// `try` yang menutupi galat, dan memang tidak mengecek izin.
    pub async fn answer(&self, caller: &Caller, body: &str) -> Option<Reply> {
        self.answer_with(caller, body, |_| {}).await
    }

    /// [`Router::answer`] dengan kesempatan mengisi `Ctx` sebelum rantai
    /// handler jalan. Pipeline memakai ini untuk store/http/sessions — tanpa
    /// isi itu answer handler melihat `user_data` Null dan sesi game kosong,
    /// sehingga balasan user tidak pernah terbaca.
    pub async fn answer_with<F>(&self, caller: &Caller, body: &str, fill: F) -> Option<Reply>
    where
        F: FnOnce(&mut Ctx) + Send + 'static,
    {
        if all_answer_handlers().is_empty() {
            return None;
        }
        let parsed = parse_command(body, &self.parse_opts);
        // `answerHandler` menerima `m` apa adanya, bukan hasil parse command,
        // jadi `Ctx.text` isi mentah dan `command` kosong.
        let parsed = ParsedCommand {
            command: String::new(),
            text: body.to_string(),
            ..parsed
        };
        let mut ctx = self.build_ctx(caller, &parsed);
        fill(&mut ctx);
        // JoinError = handler mati sebelum sempat panik-tertangkap → None,
        // paritas try/catch JS (`handler.js:440-473`) yang menelan galat.
        tokio::task::spawn_blocking(move || oktz_plugin_api::run_answer_handlers(&ctx))
            .await
            .unwrap_or(None)
    }

    /// Isi `Ctx` dari `Caller` — sama untuk dispatch command dan answer
    /// handler, supaya keduanya lihat pesan yang identik.
    fn build_ctx(&self, caller: &Caller, parsed: &ParsedCommand) -> Ctx {
        Ctx {
            text: parsed.text.clone(),
            args: parsed.args.clone(),
            sender: caller.sender.clone(),
            chat: caller.chat.clone(),
            mentioned: caller.mentioned.clone(),
            command: parsed.command.clone(),
            prefix: parsed.prefix.clone(),
            quoted: caller.quoted.clone(),
            // Media pesan pemicu + jalur unduhnya ikut ke `Ctx`: handler yang
            // memanggil `m.download()`/`ctx.download_media()` butuh keduanya,
            // dan router-lah satu-satunya tempat `caller` bertemu `Ctx`.
            media: caller.media.clone(),
            media_fetch: caller
                .media_fetch
                .clone()
                .unwrap_or_else(|| Arc::new(oktz_plugin_api::NoMediaFetch)),
            is_group: caller.is_group,
            is_owner: caller.is_owner,
            is_premium: caller.is_premium,
            push_name: caller.push_name.clone(),
            uptime_secs: self.ctx.uptime_secs(),
            // Field `Ctx` yang lain (roster, state store, sessions, http) diisi
            // `oktz-wa::pipeline` lewat pembungkus handler; router tidak punya
            // akses ke sana.
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::PluginMeta;
    use std::sync::Arc;

    struct Stub {
        user: UserState,
        group: Option<GroupState>,
        mode: ModeSettings,
    }

    impl RouterContext for Stub {
        fn user_state(&self, _s: &str) -> UserState {
            self.user.clone()
        }
        fn group_state(&self, _c: &str) -> Option<GroupState> {
            self.group
        }
        fn group_bot_mode(&self, _c: &str) -> String {
            "all".into()
        }
        fn mode_settings(&self) -> ModeSettings {
            self.mode.clone()
        }
        fn group_overrides(&self, _c: &str) -> Option<GroupOverrides> {
            None
        }
        fn capprem(&self, _n: &str) -> Option<bool> {
            None
        }
        fn uptime_secs(&self) -> u64 {
            42
        }
    }

    fn stub() -> Stub {
        Stub {
            user: UserState::default(),
            group: None,
            mode: ModeSettings {
                mode: "public".into(),
                ..Default::default()
            },
        }
    }

    fn entry(name: &str, category: &str, reply: Reply) -> PluginEntry {
        PluginEntry {
            meta: PluginMeta {
                name: name.into(),
                category: category.into(),
                ..Default::default()
            },
            handler: Arc::new(move |_| reply.clone()),
        }
    }

    fn caller() -> Caller {
        Caller {
            sender: "628123@s.whatsapp.net".into(),
            chat: "628123@s.whatsapp.net".into(),
            ..Default::default()
        }
    }

    /// Answer handler uji: hanya menjawab teks `jawab` dan mencatat urutan
    /// pemanggilan lewat statik global.
    static DITANGANI: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

    /// `DITANGANI` global dan dipakai bersama, jadi tes yang membacanya harus
    /// berurutan: tanpa ini `is_empty()` balapan dengan tes lain yang sedang
    /// `push` (CI pernah gagal karena itu, lolos saat dijalankan lokal).
    static KUNCI_UJI: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn kunci_uji() -> std::sync::MutexGuard<'static, ()> {
        KUNCI_UJI.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[oktz_plugin_api::answer_handler]
    fn uji_jawab(ctx: &oktz_plugin_api::Ctx) -> Option<oktz_plugin_api::Reply> {
        if ctx.text != "jawab" {
            return None;
        }
        DITANGANI.lock().unwrap().push("pertama");
        Some(oktz_plugin_api::Reply::text("pertama menjawab"))
    }

    #[oktz_plugin_api::answer_handler]
    fn uji_jawab_kedua(ctx: &oktz_plugin_api::Ctx) -> Option<oktz_plugin_api::Reply> {
        if ctx.text != "jawab" {
            return None;
        }
        DITANGANI.lock().unwrap().push("kedua");
        Some(oktz_plugin_api::Reply::text("kedua menjawab"))
    }

    #[tokio::test]
    async fn handler_dijalankan_di_thread_bukan_tokio_worker() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(None));
        let sink = seen.clone();
        let e = Box::leak(Box::new(PluginEntry {
            meta: PluginMeta {
                name: "tnt".into(),
                category: "tools".into(),
                ..Default::default()
            },
            handler: Arc::new(move |_| {
                *sink.lock().unwrap() = Some(std::thread::current().id());
                Reply::None
            }),
        }));
        let _ = r.route(&mut c, ".tnt", |_| Some(e)).await;
        let got = seen.lock().unwrap().expect("handler harus jalan");
        assert_ne!(
            got,
            std::thread::current().id(),
            "handler harus di thread blocking, bukan thread tokio worker"
        );
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)] // `KUNCI_UJI` hanya dipegang tes ini
    async fn rantai_answer_berhenti_di_yang_pertama_membalas() {
        let _kunci = kunci_uji();
        DITANGANI.lock().unwrap().clear();
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let balasan = r
            .answer(&caller(), "jawab")
            .await
            .expect("harus ada yang menjawab");
        assert_eq!(balasan, oktz_plugin_api::Reply::text("pertama menjawab"));
        assert_eq!(*DITANGANI.lock().unwrap(), vec!["pertama"]);
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)] // `KUNCI_UJI` hanya dipegang tes ini
    async fn jawaban_di_luar_pola_melewati_seluruh_rantai() {
        let _kunci = kunci_uji();
        DITANGANI.lock().unwrap().clear();
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        assert!(r.answer(&caller(), "bukan jawaban").await.is_none());
        assert!(DITANGANI.lock().unwrap().is_empty());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)] // `KUNCI_UJI` hanya dipegang tes ini
    async fn answer_menerima_pesan_mentah_bukan_hasil_parse() {
        let _kunci = kunci_uji();
        DITANGANI.lock().unwrap().clear();
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        // Command yang tidak dikenal diteruskan apa adanya ke handler.
        assert!(r.answer(&caller(), "/tak-dikenal").await.is_none());
        assert!(DITANGANI.lock().unwrap().is_empty());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)] // `KUNCI_UJI` hanya dipegang tes ini
    async fn answer_with_runs_the_fill_before_the_handler_chain() {
        let _kunci = kunci_uji();
        DITANGANI.lock().unwrap().clear();
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        // Pipeline mengisi `Ctx` (user_data/sessions/http) lewat closure ini.
        let balasan = r
            .answer_with(&caller(), "isian-pipeline", |c| c.text = "jawab".into())
            .await
            .expect("fill harus membuat teks cocok");
        assert_eq!(balasan, oktz_plugin_api::Reply::text("pertama menjawab"));
    }

    #[tokio::test]
    async fn non_command_returns_not_a_command() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let d = r.route(&mut c, "halo", |_| None).await;
        assert_eq!(d, Decision::NotACommand);
    }

    #[tokio::test]
    async fn command_without_plugin_is_silent() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let d = r.route(&mut c, ".tidakada", |_| None).await;
        assert_eq!(d, Decision::NoPlugin);
    }

    #[tokio::test]
    async fn plugin_runs_and_returns_reply() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let e = Box::leak(Box::new(entry("ping", "main", Reply::text("pong"))));
        let d = r.route(&mut c, ".ping", |_| Some(e)).await;
        assert_eq!(d, Decision::Handled(Reply::text("pong")));
    }

    #[tokio::test]
    async fn mode_denied_before_plugin_lookup() {
        // Mode self, caller bukan owner: embora plugin ada, mode menolak dulu.
        let s = Stub {
            mode: ModeSettings {
                mode: "self".into(),
                ..Default::default()
            },
            ..stub()
        };
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let d = r.route(&mut c, ".ping", |_| None).await;
        assert_eq!(d, Decision::DeniedMode(DenyMode::Silent));
    }

    #[tokio::test]
    async fn permission_denied_returns_message() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let owner_only = {
            let m = PluginMeta {
                name: "secret".into(),
                is_owner: true,
                ..Default::default()
            };
            Box::leak(Box::new(PluginEntry {
                meta: m,
                handler: Arc::new(|_| Reply::text("x")),
            }))
        };
        let d = r.route(&mut c, ".secret", |_| Some(owner_only)).await;
        // `config.js:198`, bukan fallback `"🚫 Owner only!"` dari
        // `middleware.js:57` — fallback cuma aktif kalau config kosong.
        assert!(
            matches!(&d, Decision::DeniedPermission(msg) if msg == "🔒 *Owner Only!* Hmph! Fitur ini cuma buat Owner bot. Kamu mana cukup? (｡•ˇ‸ˇ•｡)")
        );
    }

    #[tokio::test]
    async fn panicking_handler_is_caught_not_propagated() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let e = Box::leak(Box::new(PluginEntry {
            meta: PluginMeta {
                name: "boom".into(),
                category: "tools".into(),
                ..Default::default()
            },
            handler: Arc::new(|_| panic!("boom")),
        }));
        let d = r.route(&mut c, ".boom", |_| Some(e)).await;
        assert!(
            matches!(d, Decision::Handled(ref rep) if rep.text_body().is_some_and(|t| t.contains("Plugin error")))
        );
    }

    #[tokio::test]
    async fn ctx_carries_parsed_fields_to_handler() {
        let s = stub();
        let r = Router::new(&s, ParseOptions::default(), 1000);
        let mut c = caller();
        let seen = std::sync::Arc::new(std::sync::Mutex::new(None));
        let sink = seen.clone();
        let e = Box::leak(Box::new(PluginEntry {
            meta: PluginMeta {
                name: "echo".into(),
                category: "tools".into(),
                ..Default::default()
            },
            handler: Arc::new(move |c: Ctx| {
                *sink.lock().unwrap() = Some(c);
                Reply::None
            }),
        }));
        r.route(&mut c, ".echo satu dua", |_| Some(e)).await;
        let got = seen.lock().unwrap().clone().expect("handler harus jalan");
        assert_eq!(got.command, "echo");
        assert_eq!(got.args, vec!["satu", "dua"]);
        assert_eq!(got.text, "satu dua");
        assert_eq!(got.uptime_secs, 42);
    }
}
