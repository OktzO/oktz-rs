//! Port rantai group-protection dari `src/handler.js:770-871`.
//!
//! Rantai itu 16 sub-gate yang jalan HANYA kalau `m.isGroup && !m.isNewsletter`,
//! dan tiap gate yang `return true` langsung menghentikan pipeline. Gate PERTAMA
//! yang match yang menang — urutan itu load-bearing, bukan detail.
//!
//! Modul ini fournir dua hal terpisah:
//!
//! 1. **Framework** — [`GroupGuard`] + [`Decision`]. Gate =
//!    `fn(&GateInput) -> GateOutcome`; guard menjalankan daftar berurutan dan
//!    berhenti di gate pertama yang _match_.
//! 2. **Gate yang benar-benar ported** — lihat [`default_gate_chain`] untuk
//!    urutan dan daftar gate yang TIDAK ikut dipindah beserta alasannya.
//!
//! Data yang dipakai gate (daftar admin grup, `selfMode`, pesan) sudah
//! diteruskan caller sebagai nilai biasa, jadi tidak ada I/O dan tidak ada
//! ketergantungan wacore di sini.

/// Caller yang dipakai gate. Cerminan flag `m.*` yang dibaca rantai JS.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupCaller {
    pub sender: String,
    /// `m.key.participant` — bisa beda dari `sender` di grup addressing LID.
    pub participant: Option<String>,
    pub chat: String,
    pub is_group: bool,
    pub is_newsletter: bool,
    pub is_owner: bool,
    pub is_admin: bool,
    pub is_partner: bool,
    pub is_bot_admin: bool,
    pub from_me: bool,
    pub is_command: bool,
    pub is_document: bool,
    /// `m.type` — dipakai `checkAntidocument` (`documentMessage` /
    /// `documentWithCaptionMessage`).
    pub message_type: String,
    /// `detectSwGcType(rawMsg)` (group-protection.js:509-514) — path tipe
    /// SW-group pada pohon pesan, mis. `"message.groupMentionedMessage"`,
    /// `"message.groupStatusMessageV2"`, atau
    /// `"message.extendedTextMessage.contextInfo.groupMentions"`.
    ///
    /// `None` = pesan ini bukan SW-group (gate antiswgc lolos). Caller mengisi
    /// lewat `pipeline.rs::sw_group_type_of`: memetakan wrapper SW-group dan
    /// `contextInfo.groupMentions` tanpa masuk ke subtree `quoted_message`
    /// (`findSwGcType`, group-protection.js:467-520).
    ///
    /// `ponytail:` `groupStatusMessageV2Extension` tidak tersedia di waproto
    /// 0.7, sehingga path JS itu belum terdeteksi.
    pub sw_group_type: Option<String>,
    /// `m.body || ""`
    pub body: String,
    /// `m.mentionedJid`
    pub mentioned_jid: Vec<String>,
    /// Jumlah member grup (`groupMetadata.participants.length`).
    pub participant_count: usize,
}

/// Participant grup, seperlunya untuk cek admin.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Participant {
    /// `p.jid || p.id`
    pub id: String,
    pub lid: String,
    pub phone_number: String,
    pub admin: bool,
}

/// Setelan grup — semua field yang dibaca gate. String `Option` dipakai untuk
/// toggle yang di JS dibandingkan dengan `!== "on"` (bukan `!group.x`), jadi
/// nilai boolean `true` tidak menyalakan gate. Mode `None` = `"remove"`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroupSettings {
    pub antilink: Option<String>,
    pub antilink_mode: Option<String>,
    pub antilinkgc: Option<String>,
    pub antilinkgc_mode: Option<String>,
    pub antilinkall: Option<String>,
    pub antilinkall_mode: Option<String>,
    pub antijudol: Option<String>,
    /// `group.antijudolMode` — dilewatkan `normalizeProtectionMode` (:296).
    pub antijudol_mode: Option<String>,
    pub antiphising: Option<String>,
    pub antiphising_mode: Option<String>,
    pub anticustom: Option<String>,
    pub anticustom_mode: Option<String>,
    /// `group.anticustomRules` — aturan buatan admin grup (JSON di store).
    pub anticustom_rules: Vec<CustomRule>,
    pub antiswgc: Option<String>,
    pub antihidetag: Option<String>,
    /// Gate ini pakai `!groupData.antidocument` (truthiness), bukan `"on"`.
    pub antidocument: bool,
    pub mute: bool,
    pub muted_members: Vec<String>,
    pub mutegc: bool,
    pub slowmode: Option<Slowmode>,
}

/// `groupData.slowmode`
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Slowmode {
    pub enabled: bool,
    /// Detik; `None`/0 → default 30 (`sm.delay || 30`).
    pub delay_secs: Option<u64>,
    /// `None`/kosong → `"all"`. `"onlycommand"` punya cabang sendiri.
    pub mode: Option<String>,
}

/// Satu rule AntiCustom dari `group.anticustomRules`
/// (`plugins/group/anticustom.js:432-434` menyimpan
/// `{ name, groupName, pattern, type, action, flags }`).
///
/// Di JS apa pun selain `type: "regex"` diperlakukan sebagai contains
/// (`group-protection.js:1197-1203`), jadi `rule_type` disimpan mentah.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomRule {
    pub name: String,
    pub pattern: String,
    /// `"regex"` atau nilai lain (= contains).
    pub rule_type: String,
    /// `"kick"`/`"remove"`/`"delete"` — menimpa `anticustom_mode` bila ada
    /// (`rule.action || group.anticustomMode`, group-protection.js:2053).
    pub action: Option<String>,
    /// Flag regex mentah; dibersihkan `sanitizeCustomFlags` (:1141).
    pub flags: Option<String>,
}

/// Semua yang dibutuhkan gate besides caller + setelan grup.
#[derive(Debug, Clone, Default)]
pub struct GateEnv {
    /// `sock.user.id.split(":")[0] + "@s.whatsapp.net"` — BENTUK BUGGY yang
    /// dipakai `handleAntilink*` (group-protection.js:1327/1898/1953).
    /// `handleAntiSwGc` memakai `botJidOf()` yang benar (:1376-1379);
    /// `is_same_participant` membandingkan digit, jadi hasil PN tetap sama.
    pub bot_number: String,
    /// `sock.user.lid` — kandidat kedua untuk cek "bot admin".
    pub bot_lid: Option<String>,
    /// `db.setting("selfMode") === true`.
    pub self_mode: bool,
    /// Metadata grup berhasil dibaca?
    ///
    /// JS: tiap gate yang butuh roster memanggil `await sock.groupMetadata()`
    /// DI DALAM `try`; kalau gagal, `catch` → `return false` → gate DILEWATI
    /// SENYAP (`group-protection.js:1330/1278/2104/2153`). Tanpa flag ini,
    /// roster kosong terbaca sebagai "bot bukan admin" dan bot membalas
    /// `notAdmin` — balasan yang di JS tidak pernah muncul saat metadata gagal.
    pub metadata_available: bool,
    pub participants: Vec<Participant>,
    /// `Date.now()` untuk gate slowmode.
    pub now_ms: i64,
}

/// Hasil satu gate: `None` = tidak trigger,_chain lanjut_.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateOutcome {
    /// Gate tidak trigger — lanjut ke gate berikutnya.
    Pass,
    /// Gate memicu aksi, chain BERHENTI.
    Decide(Decision),
    /// Gate melakukan aksi tapi JS TIDAK `return` — chain LANJUT.
    /// Cuma `checkSpam` yang begini (handler.js:833-842).
    Continue,
}

/// Keputusan router yang harus dieksekusi caller.
///
/// `DeleteAndReply` dan `KickAndReply` dipisah dari `Reply`/`DeleteAndStop`
/// karena majority gate menghapus pesan DAN membalas; kalau dipipihkan jadi
/// satu varian, salah satu efeknya hilang diam-diam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Lolos semua gate — pesan lanjut ke pipeline router.
    Allow,
    /// Berhenti; balas teks. Pesan TIDAK dihapus.
    Reply(String),
    /// Berhenti; hapus pesan; tanpa balasan.
    DeleteAndStop,
    /// Berhenti tanpa aksi sama sekali.
    Stop,
    /// Hapus pesan lalu balas. Urutan eksekusi mengikuti JS: delete → reply.
    DeleteAndReply(String),
    /// Hapus pesan, keluarkan pengirim, lalu balas.
    ///
    /// Kalau kick GAGAL: `handleAntilink` (:1349) TIDAK menangkap error-nya,
    /// jadi error naik ke `catch` luar (:1363) → `return false` → tanpa
    /// balasan sama sekali. Caller tidak mengirim apa pun saat kick gagal.
    KickAndReply(String),
    /// Sama seperti [`Decision::KickAndReply`], tapi kalau kick GAGAL tetap
    /// ada balasan: teks mode-remove.
    ///
    /// `handleAntilinkGc/All` (group-protection.js:1921/1976) dan
    /// `executeProtectionAction` (:1243) menangkap error kick lalu mengirim
    /// pesan mode-remove. `handleAntilink` (:1349) tidak — makanya dua
    /// varian, bukan satu varian dengan fallback kosong.
    KickAndReplyFallback {
        /// Teks mode-kick — dikirim kalau kick BERHASIL.
        kick: String,
        /// Teks mode-remove — dikirim kalau kick GAGAL.
        fallback: String,
    },
}

/// Varian `Reply` yang perlu konten `KickAndReply`. Helper supaya caller
/// tidak perlu menyimpan string kedua di luar.
impl Decision {
    /// Teks balasan, kalau decision ini punya balasan.
    pub fn reply_text(&self) -> Option<&str> {
        match self {
            Self::Reply(t) | Self::DeleteAndReply(t) | Self::KickAndReply(t) => Some(t),
            Self::KickAndReplyFallback { kick, .. } => Some(kick),
            Self::Allow | Self::DeleteAndStop | Self::Stop => None,
        }
    }

    /// Teks yang dikirim kalau kick GAGAL — `None` = jangan kirim apa pun.
    ///
    /// Hanya bermakna untuk varian yang melakukan kick; caller lain tidak
    /// pernah memanggilnya.
    pub fn kick_fallback(&self) -> Option<&str> {
        match self {
            Self::KickAndReplyFallback { fallback, .. } => Some(fallback),
            _ => None,
        }
    }
}

/// Input yang diteruskan ke setiap gate.
pub struct GateInput<'a> {
    pub caller: &'a GroupCaller,
    pub settings: &'a GroupSettings,
    pub env: &'a GateEnv,
    /// `lastMessageTime.get("chat_sender") || 0` (slowmode.js:135).
    pub last_message_ms: i64,
}

type GateFn = fn(&GateInput<'_>) -> GateOutcome;

/// Satu gate + nama untuk debug/log. Nama ada supaya urutan rantai bisa
/// diuji tanpa membandingkan alamat fungsi (yang bisa di-dedup optimizer).
#[derive(Clone)]
pub struct Gate {
    pub name: &'static str,
    pub run: GateFn,
}

impl Gate {
    pub const fn new(name: &'static str, run: GateFn) -> Self {
        Self { name, run }
    }
}

/// Rantai gate. Urutan daftar = urutan eksekusi.
pub struct GroupGuard {
    gates: Vec<Gate>,
}

impl GroupGuard {
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }

    /// Nama gate berurutan — berguna untuk log saat gate pertama match.
    pub fn gate_names(&self) -> Vec<&'static str> {
        self.gates.iter().map(|g| g.name).collect()
    }

    /// Jalankan rantai. Gate pertama yang `Decide` menang.
    pub fn run(&self, input: &GateInput<'_>) -> Decision {
        // handler.js:770 — seluruh blok dilewati kalau bukan grup / newsletter.
        if !input.caller.is_group || input.caller.is_newsletter {
            return Decision::Allow;
        }
        for gate in &self.gates {
            if let GateOutcome::Decide(d) = (gate.run)(input) {
                return d;
            }
        }
        Decision::Allow
    }
}

/// Rantai default — urutan persis `handler.js:773-871`, gate yang tidak
/// diport di-lewati (komentarnya menyebut alasannya; urutan relatif gate yang
/// ada tetap sama).
pub fn default_gate_chain() -> GroupGuard {
    GroupGuard::new(vec![
        Gate::new("antijudol", gate_anti_judol),     // handler.js:773
        Gate::new("antiphising", gate_anti_phising), // handler.js:776
        Gate::new("anticustom", gate_anti_custom),   // handler.js:779
        Gate::new("antilink", gate_antilink),        // handler.js:782
        Gate::new("antilinkgc", gate_antilink_gc),   // handler.js:785
        Gate::new("antilinkall", gate_antilink_all), // handler.js:788
        Gate::new("antihidetag", gate_hidetag),      // handler.js:791
        Gate::new("antiswgc", gate_anti_swgc),       // handler.js:794
        Gate::new("antidocument", gate_antidocument), // handler.js:797
        Gate::new("mute", gate_mute),                // handler.js:809
        Gate::new("mutedmember", gate_muted_member), // handler.js:818
        Gate::new("mutegc", gate_mutegc),            // handler.js:827
        Gate::new("slowmode", gate_slowmode),        // handler.js:844
                                                     // Dilewati: detectBot (butuh `analyzeBotMessage` + `groupMetadata`),
                                                     // checkSpam (butuh `spamTracker` lintas pesan + `handleSpamAction`),
                                                     // isToxic (butuh `toxicWords` + counter `toxicWarns` per member di DB).
                                                     // AntiJudol/AntiPhising/AntiCustom/AntiSwGc kini ikut (sebelumnya
                                                     // dilewati — lihat git history).
    ])
}

// ---------------------------------------------------------------------------
// Gate
// ---------------------------------------------------------------------------

/// `group-protection.js:1311` `handleAntilink`.
///
/// PENTING: gate ini TIDAK mengecek `selfMode` sebelum skip pesan bot sendiri
/// (bandingkan `handleAntilinkGc` :1899-1900). Dan `botNumber` di sini bentuk
/// buggy `id.split(":")[0] + "@s.whatsapp.net"` — sama seperti JS.
fn gate_antilink(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antilink.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    if !contains_link(&c.body) {
        return GateOutcome::Pass;
    }

    let sender_tag = sender_tag(&c.sender);
    if is_same_participant(
        c.participant.as_deref().unwrap_or(&c.sender),
        &input.env.bot_number,
    ) {
        return GateOutcome::Pass;
    }
    if !input.env.metadata_available {
        // JS: `await sock.groupMetadata(m.chat)` gagal → `catch` (:1363) →
        // `return false` → gate dilewati SENYAP. Bukan balas "bukan admin".
        return GateOutcome::Pass;
    }

    if is_admin_check(&input.env.participants, c.participant.as_deref(), &c.sender) {
        return GateOutcome::Pass;
    }
    if !is_bot_admin_check(input.env) {
        // Gate ini replying "bot bukan admin" — berbeda dari Gc/All yang diam.
        return GateOutcome::Decide(Decision::Reply(NOT_ADMIN.to_string()));
    }

    let kicked = input.settings.antilink_mode.as_deref() == Some("kick");
    let text = if kicked {
        msg(ANTILINK_KICK, &sender_tag)
    } else {
        msg(ANTILINK, &sender_tag)
    };
    GateOutcome::Decide(if kicked {
        Decision::KickAndReply(text)
    } else {
        Decision::DeleteAndReply(text)
    })
}

/// `group-protection.js:1888` `handleAntilinkGc` — hanya link WA spesifik.
///
/// Berbeda dari `handleAntilink` di dua hal yang mengubah perilaku:
/// - self-sender hanya di-skip kalau `selfMode` MATI (:1900);
/// - bot bukan admin → `return false`, SENYAP (:1909), bukan reply.
fn gate_antilink_gc(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antilinkgc.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    if !contains_wa_specific_link(&c.body) {
        return GateOutcome::Pass;
    }

    let sender_tag = sender_tag(&c.sender);
    if !input.env.self_mode
        && is_same_participant(
            c.participant.as_deref().unwrap_or(&c.sender),
            &input.env.bot_number,
        )
    {
        return GateOutcome::Pass;
    }

    if !input.env.metadata_available {
        // JS: `await sock.groupMetadata(m.chat)` gagal → `catch` (:1936) →
        // `return false` → gate dilewati SENYAP.
        return GateOutcome::Pass;
    }

    if is_admin_check(&input.env.participants, c.participant.as_deref(), &c.sender) {
        return GateOutcome::Pass;
    }
    if !is_bot_admin_check(input.env) {
        return GateOutcome::Pass;
    }

    let kicked = input.settings.antilinkgc_mode.as_deref() == Some("kick");
    let remove_text = msg(ANTILINK_GC, &sender_tag);
    let text = if kicked {
        msg(ANTILINK_GC_KICK, &sender_tag)
    } else {
        remove_text.clone()
    };
    GateOutcome::Decide(if kicked {
        // Kick gagal → JS tetap balas pesan mode-remove (:1917-1924).
        Decision::KickAndReplyFallback {
            kick: text,
            fallback: remove_text,
        }
    } else {
        Decision::DeleteAndReply(text)
    })
}

/// `group-protection.js:1943` `handleAntilinkAll` — link apa pun (TLD daftar).
///
/// Strukturnya identik dengan `handleAntilinkGc` (termasuk senyap saat bot
/// bukan admin, :1964) — hanya deteksi link dan teks pesannya yang beda.
fn gate_antilink_all(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antilinkall.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    if !contains_any_link(&c.body) {
        return GateOutcome::Pass;
    }

    let sender_tag = sender_tag(&c.sender);
    if !input.env.self_mode
        && is_same_participant(
            c.participant.as_deref().unwrap_or(&c.sender),
            &input.env.bot_number,
        )
    {
        return GateOutcome::Pass;
    }

    if !input.env.metadata_available {
        // JS: `await sock.groupMetadata(m.chat)` gagal → `catch` (:1989) →
        // `return false` → gate dilewati SENYAP.
        return GateOutcome::Pass;
    }

    if is_admin_check(&input.env.participants, c.participant.as_deref(), &c.sender) {
        return GateOutcome::Pass;
    }
    if !is_bot_admin_check(input.env) {
        return GateOutcome::Pass;
    }

    let kicked = input.settings.antilinkall_mode.as_deref() == Some("kick");
    let remove_text = msg(ANTILINK_ALL, &sender_tag);
    let text = if kicked {
        msg(ANTILINK_ALL_KICK, &sender_tag)
    } else {
        remove_text.clone()
    };
    GateOutcome::Decide(if kicked {
        // Kick gagal → JS tetap balas pesan mode-remove (:1972-1979).
        Decision::KickAndReplyFallback {
            kick: text,
            fallback: remove_text,
        }
    } else {
        Decision::DeleteAndReply(text)
    })
}

/// `group-protection.js:1253-1309` `handleTextProtection` — kerangka bersama
/// `handleAntiJudol` (:1998) dan `handleAntiPhising` (:2011) SETELAH detektor
/// cocok: skip bot sendiri (hanya saat selfMode mati, :1276) → admin → bot
/// bukan admin membalas `notAdmin` (:1286-1292, beda dari antilinkGc/All yang
/// senyap) → mode `normalizeProtectionMode` (:1294) → delete/kick + reply.
fn text_protection_action(
    input: &GateInput<'_>,
    mode_raw: Option<&str>,
    remove_text: String,
    kick_text: String,
) -> GateOutcome {
    let c = input.caller;
    if !input.env.self_mode
        && is_same_participant(
            c.participant.as_deref().unwrap_or(&c.sender),
            &input.env.bot_number,
        )
    {
        return GateOutcome::Pass;
    }
    if !input.env.metadata_available {
        // JS: `await sock.groupMetadata(m.chat)` gagal → `catch` (:1306) →
        // `return false` → gate dilewati SENYAP.
        return GateOutcome::Pass;
    }
    if is_admin_check(&input.env.participants, c.participant.as_deref(), &c.sender) {
        return GateOutcome::Pass;
    }
    if !is_bot_admin_check(input.env) {
        return GateOutcome::Decide(Decision::Reply(NOT_ADMIN.to_string()));
    }
    if normalize_protection_mode(mode_raw) == "kick" {
        // Kick gagal → JS membalas pesan remove (:1233-1243, lewat catch).
        GateOutcome::Decide(Decision::KickAndReplyFallback {
            kick: kick_text,
            fallback: remove_text,
        })
    } else {
        GateOutcome::Decide(Decision::DeleteAndReply(remove_text))
    }
}

/// `getProtectionText` (group-protection.js:304-306) — `m.body || m.text`
/// lalu trim. Di Rust caller hanya meneruskan `m.body`.
fn protection_text(c: &GroupCaller) -> &str {
    c.body.trim()
}

/// `handleAntiJudol` (group-protection.js:1998-2009): toggle `antijudol`,
/// detektor [`detect_judol`], pesan `antijudol`/`antijudolKick`.
///
/// Catatan porting: `config.groupProtection?.[key]` di `gpMsg` belum diport —
/// `config.rs` sengaja tidak menyalin `groupProtection` (lihat komentar
/// `Runtime` di oktz-core), jadi teks default `gpMsg` dipakai selalu.
fn gate_anti_judol(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antijudol.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    let text = protection_text(c);
    if !detect_judol(text).matched {
        return GateOutcome::Pass;
    }
    let sender_tag = sender_tag(&c.sender);
    text_protection_action(
        input,
        input.settings.antijudol_mode.as_deref(),
        msg(ANTIJUDOL, &sender_tag),
        msg(ANTIJUDOL_KICK, &sender_tag),
    )
}

/// `handleAntiPhising` (group-protection.js:2011-2022): toggle `antiphising`,
/// detektor [`detect_phishing`], pesan `antiphising`/`antiphisingKick`.
fn gate_anti_phising(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antiphising.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    let text = protection_text(c);
    if !detect_phishing(text).matched {
        return GateOutcome::Pass;
    }
    let sender_tag = sender_tag(&c.sender);
    text_protection_action(
        input,
        input.settings.antiphising_mode.as_deref(),
        msg(ANTIPHISING, &sender_tag),
        msg(ANTIPHISING_KICK, &sender_tag),
    )
}

/// `handleAntiCustom` (group-protection.js:2024-2073): toggle `anticustom`,
/// rule cocok via [`match_custom_rule`] (pola buatan admin, sudah dianalisis
/// keamanannya), mode dari `rule.action || group.anticustomMode` (:2053-2056).
fn gate_anti_custom(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.anticustom.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    let text = protection_text(c);
    let Some(rule) = match_custom_rule(text, &input.settings.anticustom_rules) else {
        return GateOutcome::Pass;
    };
    // `rule.action || group.anticustomMode` — string kosong falsy di JS,
    // jadi jatuh ke mode grup (:2053-2055).
    let mode_raw = match rule.action.as_deref() {
        Some(a) if !a.is_empty() => Some(a),
        _ => input.settings.anticustom_mode.as_deref(),
    };
    // Replacements `{ rule: rule.name || rule.pattern || "custom-rule" }`
    // (group-protection.js:2066-2068).
    let rule_label = if !rule.name.is_empty() {
        rule.name.clone()
    } else if !rule.pattern.is_empty() {
        rule.pattern.clone()
    } else {
        "custom-rule".to_string()
    };
    let sender_tag = sender_tag(&c.sender);
    text_protection_action(
        input,
        mode_raw,
        msg(ANTICUSTOM, &sender_tag).replace("%rule%", &rule_label),
        msg(ANTICUSTOM_KICK, &sender_tag).replace("%rule%", &rule_label),
    )
}

/// `handleAntiSwGc` (group-protection.js:2075-2143) — hapus pesan yang
/// membawa tipe SW-group ("share status ke grup").
///
/// Perbedaan penting dari gate teks:
/// - TIDAK ada selfMode dan TIDAK ada mode kick; yang dicek `key.fromMe`,
///   `botJidOf` (bentuk benar), lalu LID bot (:2096-2102).
/// - Deteksi tipe diisi caller lewat `pipeline.rs::sw_group_type_of` (lihat
///   komentar field).
/// - Cek admin hanya memakai `key.participant` versi ternormalisasi
///   (`senderCand = [sender]`, :2106) — bukan `[participant, m.sender]`.
fn gate_anti_swgc(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antiswgc.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    // `detectSwGcType(rawMsg)` (:2085) — `None` berarti bukan SW-group.
    let Some(sw_type) = c.sw_group_type.as_deref() else {
        return GateOutcome::Pass;
    };
    // `resolveMessageSenderJid(key, sock)` (:2088, :308-311) — participant
    // asli; fallback fromMe → user id bot, selain itu remoteJid.
    let fallback = if c.from_me {
        input.env.bot_number.clone()
    } else {
        c.chat.clone()
    };
    let sender = normalize_comparable_jid(c.participant.as_deref().unwrap_or(&fallback));
    // :2096-2102 — pesan bot sendiri tidak boleh diproses; bandingkan PN
    // (bentuk `botJidOf`) DAN LID. `env.bot_number` di sini bentuk buggy
    // `split(":")[0] + "@s.whatsapp.net"` — `is_same_participant` membandingkan
    // digit, jadi hasilnya sama dengan `botJidOf`.
    if c.from_me || is_same_participant(&sender, &input.env.bot_number) {
        return GateOutcome::Pass;
    }
    if let Some(lid) = &input.env.bot_lid {
        if is_same_participant(c.participant.as_deref().unwrap_or(""), lid) {
            return GateOutcome::Pass;
        }
    }
    if !input.env.metadata_available {
        // JS: `await sock.groupMetadata(chatId)` gagal → `catch` (:2140) →
        // `return false` → gate dilewati SENYAP.
        return GateOutcome::Pass;
    }
    // :2110 — senderCand hanya [sender].
    if is_admin_check(&input.env.participants, Some(&sender), "") {
        return GateOutcome::Pass;
    }
    if !is_bot_admin_check(input.env) {
        // :2111-2117 — balas notAdmin, tanpa revoke.
        return GateOutcome::Decide(Decision::Reply(NOT_ADMIN.to_string()));
    }
    // senderTag = (sender || botNumber || "Unknown").split("@")[0] (:2108).
    let tag_source = if sender.is_empty() {
        if input.env.bot_number.is_empty() {
            "Unknown".to_string()
        } else {
            input.env.bot_number.clone()
        }
    } else {
        sender
    };
    let text = msg(ANTISWGC, &sender_tag(&tag_source)).replace("%type%", sw_type);
    GateOutcome::Decide(Decision::DeleteAndReply(text))
}

/// `group-protection.js:2145` `handleAntiHidetag`.
///
/// Threshold-nya `mentionedJid.length < participants.length` → lolos, jadi
/// yang dihapus hanya hidetag yang menyasar SEMUA member. Bot bukan admin
/// → senyap (`return false`), bukan reply.
fn gate_hidetag(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if input.settings.antihidetag.as_deref() != Some("on") {
        return GateOutcome::Pass;
    }
    if c.mentioned_jid.is_empty() {
        return GateOutcome::Pass;
    }
    if !input.env.metadata_available {
        // JS: `await sock.groupMetadata(m.chat)` gagal → `catch` (:2170) →
        // `return false` → gate dilewati SENYAP.
        return GateOutcome::Pass;
    }
    if c.mentioned_jid.len() < c.participant_count {
        return GateOutcome::Pass;
    }
    if c.is_admin || c.is_owner || c.from_me {
        return GateOutcome::Pass;
    }
    if !c.is_bot_admin {
        return GateOutcome::Pass;
    }

    let text = msg(ANTIHIDETAG, &sender_tag(&c.sender));
    GateOutcome::Decide(Decision::DeleteAndReply(text))
}

/// `plugins/group/antidocument.js:32` `checkAntidocument`.
///
/// Punya exemption `isAdmin || isOwner || fromMe` di baris PERTAMA — sudah
/// ada sebelum toggle dicek. Toggle-nya truthiness (`!groupData.antidocument`),
/// bukan `"on"` seperti gate antilink.
fn gate_antidocument(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if c.is_admin || c.is_owner || c.from_me {
        return GateOutcome::Pass;
    }
    if !input.settings.antidocument {
        return GateOutcome::Pass;
    }
    let is_document = c.is_document
        || c.message_type == "documentMessage"
        || c.message_type == "documentWithCaptionMessage";
    if !is_document {
        return GateOutcome::Pass;
    }

    let text = msg(ANTIDOCUMENT, &sender_tag(&c.sender));
    GateOutcome::Decide(Decision::DeleteAndReply(text))
}

/// `handler.js:809-816` + `plugins/group/mute.js:31` `isMuted`.
///
/// Exemption `!isAdmin && !isOwner` ada di handler, jadi bot bukan admin TIDAK
/// exempt: dia tetap delete kalau punya hak, lalu stop. Reply? Tidak ada.
fn gate_mute(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if c.is_admin || c.is_owner {
        return GateOutcome::Pass;
    }
    if !input.settings.mute {
        return GateOutcome::Pass;
    }
    GateOutcome::Decide(if c.is_bot_admin {
        Decision::DeleteAndStop
    } else {
        Decision::Stop
    })
}

/// `handler.js:818-825` + `plugins/group/mutemember.js:102` `isMutedMember`.
fn gate_muted_member(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if c.is_admin || c.is_owner {
        return GateOutcome::Pass;
    }
    if input.settings.muted_members.is_empty() {
        return GateOutcome::Pass;
    }
    if !is_muted_member(&input.settings.muted_members, &c.sender) {
        return GateOutcome::Pass;
    }
    GateOutcome::Decide(if c.is_bot_admin {
        Decision::DeleteAndStop
    } else {
        Decision::Stop
    })
}

/// `handler.js:827-831` + `plugins/group/mutegc.js:49` `isMutegc`.
///
/// Satu-satunya gate dengan exemption tigaflag `!isAdmin && !isOwner &&
/// !isPartner` (handler.js:827). Tidak ada delete — `return` polos.
fn gate_mutegc(input: &GateInput<'_>) -> GateOutcome {
    let c = input.caller;
    if c.is_admin || c.is_owner || c.is_partner {
        return GateOutcome::Pass;
    }
    if !input.settings.mutegc {
        return GateOutcome::Pass;
    }
    GateOutcome::Decide(Decision::Stop)
}

/// `handler.js:844-856` + `plugins/group/slowmode.js:120` `checkSlowmode`.
///
/// Dua cabang yang saling berbeda dan keduanya penting:
/// - `onlycommand` + pesan command + terlalu cepat → `return` SENYAP, pesan
///   TIDAK dihapus (:849).
/// - mode lain + terlalu cepat → delete lalu `return` (:851-852).
///
/// Sisi caller: kalau mode `all`/`onlycommand` dan waktu sudah cukup, JS
/// `lastMessageTime.set(key, now)` (slowmode.js:142). Itu efek samping dan
/// gate ini pure, jadi [`GroupGuard::run_with_record`] melaporkannya lewat
/// return value kedua — bukan dit-ulangkan di dalam gate.
fn gate_slowmode(input: &GateInput<'_>) -> GateOutcome {
    match eval_slowmode(input) {
        SlowVerdict::Pass => GateOutcome::Pass,
        SlowVerdict::Stop => GateOutcome::Decide(Decision::Stop),
        SlowVerdict::Delete => GateOutcome::Decide(Decision::DeleteAndStop),
    }
}

/// Hasil evaluasi slowmode — dipakai gate DAN penentu efek samping
/// `lastMessageTime`, supaya logikanya tidak ditulis dua kali.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlowVerdict {
    Pass,
    Stop,
    Delete,
}

fn eval_slowmode(input: &GateInput<'_>) -> SlowVerdict {
    let c = input.caller;
    if c.is_admin || c.is_owner {
        return SlowVerdict::Pass;
    }
    let Some(sm) = input.settings.slowmode.as_ref() else {
        return SlowVerdict::Pass;
    };
    if !sm.enabled {
        return SlowVerdict::Pass;
    }
    let raw = sm.mode.as_deref().unwrap_or("all");
    let mode = if raw.is_empty() { "all" } else { raw };
    if mode == "onlycommand" && !c.is_command {
        return SlowVerdict::Pass;
    }

    let delay = sm.delay_secs.filter(|d| *d > 0).unwrap_or(30) as f64;
    let elapsed_secs = (input.env.now_ms - input.last_message_ms) as f64 / 1000.0;
    if elapsed_secs >= delay {
        return SlowVerdict::Pass;
    }
    if mode == "onlycommand" {
        // handler.js:849 — `return` tanpa delete.
        SlowVerdict::Stop
    } else {
        SlowVerdict::Delete
    }
}

/// `true` kalau slowmode aktif untuk pesan ini DAN waktunya sudah cukup —
/// artinya caller harus menyimpan `now_ms` untuk `chat_sender`.
fn slowmode_records_timestamp(input: &GateInput<'_>) -> bool {
    let c = input.caller;
    let Some(sm) = input.settings.slowmode.as_ref() else {
        return false;
    };
    if !sm.enabled {
        return false;
    }
    let raw = sm.mode.as_deref().unwrap_or("all");
    let mode = if raw.is_empty() { "all" } else { raw };
    if mode == "onlycommand" && !c.is_command {
        return false;
    }
    // `eval_slowmode` sudah menolak admin/owner; di sini cukup kasus tidak
    // exempt agar hasilnya sama persis dengan "lolos + bukan exempt".
    if c.is_admin || c.is_owner {
        return false;
    }
    let delay = sm.delay_secs.filter(|d| *d > 0).unwrap_or(30) as f64;
    (input.env.now_ms - input.last_message_ms) as f64 / 1000.0 >= delay
}

impl GroupGuard {
    /// Sama seperti [`Self::run`], tapi melaporkan apakah caller harus
    /// menyimpan `now_ms` untuk key `chat_sender` (efek samping
    /// `lastMessageTime.set()` di slowmode.js:142).
    pub fn run_with_record(&self, input: &GateInput<'_>) -> (Decision, bool) {
        let d = self.run(input);
        let record = d == Decision::Allow && slowmode_records_timestamp(input);
        (d, record)
    }
}

// ---------------------------------------------------------------------------
// Teks default `gpMsg` (group-protection.js:25-53)
// ---------------------------------------------------------------------------

pub const NOT_ADMIN: &str = "⚠ Bot bukan admin, tidak bisa menghapus pesan.";
pub const ANTILINK: &str = "⚠ *Antilink* — @%user% mengirim link.\nPesan dihapus.";
pub const ANTILINK_KICK: &str = "⚠ *Antilink* — @%user% di-kick karena mengirim link.";
pub const ANTILINK_GC: &str = "⚠ *Antilink WA* — @%user% mengirim link WA.\nPesan dihapus.";
pub const ANTILINK_GC_KICK: &str = "⚠ *Antilink WA* — @%user% di-kick karena mengirim link WA.";
pub const ANTILINK_ALL: &str = "⚠ *Antilink* — @%user% mengirim link.\nPesan dihapus.";
pub const ANTILINK_ALL_KICK: &str = "⚠ *Antilink* — @%user% di-kick karena mengirim link.";
pub const ANTIHIDETAG: &str = "⚠ *AntiHidetag* — Hidetag dari @%user% dihapus.";
pub const ANTIDOCUMENT: &str = "⚠ *AntiDocument* — Dokumen dari @%user% dihapus.";
/// `gpMsg("antijudol")` (group-protection.js:38-44).
pub const ANTIJUDOL: &str =
    "⚠ *AntiJudol* — @%user% terdeteksi kirim konten judol.\nPesan dihapus.";
pub const ANTIJUDOL_KICK: &str = "⚠ *AntiJudol* — @%user% di-kick karena kirim konten judol.";
/// `gpMsg("antiphising")`.
pub const ANTIPHISING: &str =
    "⚠ *AntiPhising* — @%user% terdeteksi kirim konten phising.\nPesan dihapus.";
pub const ANTIPHISING_KICK: &str = "⚠ *AntiPhising* — @%user% di-kick karena kirim konten phising.";
/// `gpMsg("anticustom")` — `%rule%` diganti nama rule (fallback:
/// pattern, lalu `"custom-rule"`; group-protection.js:2066-2068).
pub const ANTICUSTOM: &str =
    "⚠ *AntiCustom* — @%user% melanggar rule custom *%rule%*.\nPesan dihapus.";
pub const ANTICUSTOM_KICK: &str =
    "⚠ *AntiCustom* — @%user% di-kick karena melanggar rule custom *%rule%*.";
/// `gpMsg("antiswgc")` — `%type%` diganti path tipe dari `detectSwGcType`.
pub const ANTISWGC: &str = "⚠ *AntiSWGC* — SW group type *%type%* dari @%user% dihapus.";

/// `gpMsg(key, { user })` — ganti `%user%` dengan nomor tanpa domain.
fn msg(template: &str, sender_tag: &str) -> String {
    template.replace("%user%", sender_tag)
}

/// `m.sender.split("@")[0]`
fn sender_tag(sender: &str) -> String {
    sender.split('@').next().unwrap_or(sender).to_string()
}

// ---------------------------------------------------------------------------
// Cek admin (port `adminFlagsFor` serialize.js:140, `isSameParticipant` lid.js:454)
// ---------------------------------------------------------------------------

/// Digit saja dari sebuah JID — `replace(/[^0-9]/g, "")`.
fn digits(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

/// `_numHit` (serialize.js:118): exact dulu, lalu fuzzy suffix hanya kalau
/// kedua sisi panjangnya >= 8. Tanpa guard itu, nomor pendek ("1") akan cocok
/// dengan semua participant dan semua orang jadi admin.
fn num_hit(candidate_nums: &[String], num: &str) -> bool {
    if num.is_empty() {
        return false;
    }
    if candidate_nums.iter().any(|n| n == num) {
        return true;
    }
    if num.len() < 8 {
        return false;
    }
    candidate_nums
        .iter()
        .filter(|n| n.len() >= 8)
        .any(|n| n.ends_with(num) || num.ends_with(n.as_str()))
}

/// `_participantNums` (serialize.js:101) tanpa cabang LID→PN yang butuh cache.
fn participant_nums(p: &Participant) -> Vec<String> {
    let mut nums: Vec<String> = Vec::new();
    let mut add = |v: &str| {
        let d = digits(v);
        if !d.is_empty() && !nums.contains(&d) {
            nums.push(d);
        }
    };
    add(&p.id);
    add(&p.lid);
    add(&p.phone_number);
    nums
}

/// `adminFlagsFor(...).isAdmin` — sender cukup salah satu kandidat yang
/// match participant admin.
///
/// Publik: pipeline memakai helper yang sama untuk `m.isAdmin`
/// (`serialize.js:846`) supaya hasil cek admin roster di `Ctx` dan di gate
/// tidak berbeda.
pub fn sender_is_admin(participants: &[Participant], candidates: &[&str]) -> bool {
    let sender_nums: Vec<String> = candidates
        .iter()
        .map(|c| digits(c))
        .filter(|d| !d.is_empty())
        .collect();
    if sender_nums.is_empty() {
        return false;
    }
    participants.iter().any(|p| {
        p.admin && {
            let nums = participant_nums(p);
            sender_nums.iter().any(|sn| num_hit(&nums, sn))
        }
    })
}

/// `adminFlagsFor(...).isBotAdmin` — bot harus cocok ke `bot_number` atau
/// `bot_lid`, supaya grup addressing LID tetap terdeteksi (lihat komentar
/// serialize.js:259-268).
///
/// Publik: pipeline memakai helper yang sama untuk `m.isBotAdmin`
/// (`serialize.js:847`) saat menyusun roster grup.
pub fn bot_is_admin(participants: &[Participant], bot_jids: &[String]) -> bool {
    let bot_nums: Vec<String> = bot_jids
        .iter()
        .map(|c| digits(c))
        .filter(|d| !d.is_empty())
        .collect();
    if bot_nums.is_empty() {
        return false;
    }
    participants.iter().any(|p| {
        p.admin && {
            let nums = participant_nums(p);
            bot_nums.iter().any(|bn| num_hit(&nums, bn))
        }
    })
}

/// `adminFlagsFor(...).isAdmin` dengan kandidat `[participant, sender]`.
fn is_admin_check(participants: &[Participant], participant: Option<&str>, sender: &str) -> bool {
    let candidates: Vec<&str> = participant
        .into_iter()
        .chain(std::iter::once(sender))
        .collect();
    sender_is_admin(participants, &candidates)
}

/// `is_bot_admin_check` versi env — kandidat bot = PN + LID.
fn is_bot_admin_check(env: &GateEnv) -> bool {
    let mut bot_jids: Vec<String> = vec![env.bot_number.clone()];
    if let Some(lid) = &env.bot_lid {
        bot_jids.push(lid.clone());
    }
    bot_is_admin(&env.participants, &bot_jids)
}

/// `normalizeComparableJid` (lid.js:438) TANPA konversi LID→PN (butuh cache).
/// Yang performed: buang suffix device `:12` pada JID `@s.whatsapp.net`.
fn normalize_comparable_jid(jid: &str) -> String {
    let value = jid.trim();
    if value.is_empty() {
        return String::new();
    }
    if value.contains(':') && value.ends_with("@s.whatsapp.net") {
        return format!(
            "{}@s.whatsapp.net",
            value.split(':').next().unwrap_or(value)
        );
    }
    value.to_string()
}

/// `isSameParticipant` (lid.js:454) tanpa cabang LID→PN.
fn is_same_participant(left: &str, right: &str) -> bool {
    let l = normalize_comparable_jid(left);
    let r = normalize_comparable_jid(right);
    if l.is_empty() || r.is_empty() {
        return false;
    }
    if l == r {
        return true;
    }
    let (ln, rn) = (digits(&l), digits(&r));
    if ln.is_empty() || rn.is_empty() {
        return false;
    }
    ln == rn || ln.ends_with(&rn) || rn.ends_with(&ln)
}

/// `mutemember.js:102` `isMutedMember` — cocok longgar via `endsWith`, sama
/// seperti JS (memakai JID tersimpan apa adanya, termasuk yang sudah ke-LID).
fn is_muted_member(muted: &[String], sender: &str) -> bool {
    let strip = |v: &str| v.split('@').next().unwrap_or(v).to_string();
    let sender_number = strip(sender);
    muted.iter().any(|jid| {
        let c = strip(jid);
        c == sender_number || c.ends_with(&sender_number) || sender_number.ends_with(&c)
    })
}

// ---------------------------------------------------------------------------
// Deteksi link — padanan 3 regex di group-protection.js:91-94, 1857-1866
// ---------------------------------------------------------------------------

// `ponytail:` crate `regex` tidak ada di dependency oktz-router dan
// Cargo.toml di luar file yang boleh diubah, jadi ketiga regex ditulis ulang
// sebagai scanner. Sudah dicocokkan dengan regex aslinya di node lewat test
// `link_detection_matches_js_regex`; kalau regex crate masuk workspace,
// ganti ketiganya dengan `Regex::new` persis seperti JS dan hapus `TLDS`.

/// TLD untuk `ALL_LINK_PATTERN` (group-protection.js:1866).
const TLDS: &[&str] = &[
    "com",
    "net",
    "org",
    "io",
    "id",
    "co",
    "cc",
    "me",
    "ly",
    "gg",
    "gl",
    "it",
    "tv",
    "ru",
    "de",
    "fr",
    "uk",
    "us",
    "info",
    "biz",
    "xyz",
    "top",
    "site",
    "online",
    "store",
    "app",
    "dev",
    "ai",
    "link",
    "click",
    "fun",
    "space",
    "live",
    "world",
    "tech",
    "digital",
    "cloud",
    "pro",
    "vip",
    "pw",
    "tk",
    "ml",
    "ga",
    "cf",
    "gq",
    "club",
    "mobi",
    "name",
    "asia",
    "tel",
    "wang",
    "win",
    "bid",
    "loan",
    "ren",
    "ink",
    "art",
    "shop",
    "life",
    "game",
    "news",
    "blog",
    "design",
    "studio",
    "photo",
    "press",
    "media",
    "social",
    "group",
    "team",
    "community",
    "domain",
];

/// Karakter sebelum `.` pada `LINK_REGEX`: `[-a-zA-Z0-9@:%._+~#=]`.
fn is_link_lead(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, '-' | '@' | ':' | '%' | '.' | '_' | '+' | '~' | '#' | '=')
}

/// Karakter TLD pada `LINK_REGEX`: `[a-zA-Z0-9()]`.
fn is_link_tld(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '(' || c == ')'
}

/// Char `\w` untuk uji word-boundary `\b`.
fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `LINK_REGEX` (group-protection.js:91) — bentuk aslinya
/// `(?:https?:\/\/)?(?:www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b[-a-zA-Z0-9()@:%_+.~#?&//=]*`
///
/// Yang menentukan: ada `.` yang didahului char lead, diikuti 1–6 char TLD,
/// lalu `\b`. Run TLD di-backtrack dari terpanjang ke terpendek karena `\b`
/// bisa gagal untuk run terpanjang tapi lolos untuk yang lebih pendek.
fn contains_link(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    for i in 1..chars.len() {
        if chars[i] != '.' || !is_link_lead(chars[i - 1]) {
            continue;
        }
        let mut run = 0;
        while run < 6 && i + 1 + run < chars.len() && is_link_tld(chars[i + 1 + run]) {
            run += 1;
        }
        // Backtrack dari run terpanjang ke terpendek; `\b` harus berlaku.
        // `\b` = tepat satu sisi yang `\w`. Ujung string dianggap NON-word,
        // jadi "com" di akhir teks tetap boundary.
        for take in (1..=run).rev() {
            let j = i + 1 + take;
            let before_is_word = is_word(chars[j - 1]);
            let after_is_word = chars.get(j).is_some_and(|c| is_word(*c));
            if before_is_word != after_is_word {
                return true;
            }
        }
    }
    false
}

/// `containsWaSpecificLink` (group-protection.js:1868) — 5 pola, case-insensitive.
fn contains_wa_specific_link(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    // (prefix, suffix harus alphanumeric kecuali api.whatsapp.com/send)
    const PREFIXES: &[(&str, bool)] = &[
        ("chat.whatsapp.com/", true),
        ("wa.me/", true),
        ("whatsapp.com/channel/", true),
        ("whatsapp.com/c/", true),
        ("api.whatsapp.com/send", false),
    ];
    for (prefix, need_tail) in PREFIXES {
        let mut from = 0;
        while let Some(rel) = lower[from..].find(prefix) {
            let tail = &lower[from + rel + prefix.len()..];
            if !need_tail
                || tail
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphanumeric())
            {
                return true;
            }
            from += rel + 1;
        }
    }
    false
}

/// `containsAnyLink` (group-protection.js:1878) — `ALL_LINK_PATTERN`.
///
/// Char sebelum `.` harus ada dan bukan salah satu karakter yang dikecualikan
/// (`\s<>"{}|\^`[]`); setelah `.` harus salah satu TLD di [`TLDS`].
fn contains_any_link(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    for i in 1..chars.len() {
        if chars[i] != '.' {
            continue;
        }
        let prev = chars[i - 1];
        if is_js_space(prev)
            || matches!(
                prev,
                '<' | '>' | '"' | '{' | '}' | '|' | '\\' | '^' | '`' | '[' | ']'
            )
        {
            continue;
        }
        let rest: String = chars[i + 1..]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase();
        if TLDS.iter().any(|tld| rest.starts_with(tld)) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Deteksi teks judol/phising — port `normalizeProtectionText` /
// `buildProtectionTextVariants` / `scorePatterns` / `scoreKeywords` /
// `detectJudol` / `detectPhishing` (group-protection.js:296-440).
// ---------------------------------------------------------------------------

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// `normalizeProtectionMode` (group-protection.js:296-302) dengan fallback
/// `"remove"` — satu-satunya fallback yang dipakai gate judol/phising/custom.
/// `"delete"` disamakan `"remove"`; nilai lain → fallback.
fn normalize_protection_mode(mode: Option<&str>) -> &'static str {
    const FALLBACK: &str = "remove";
    let Some(value) = mode else {
        // `String(undefined || fallback)` — mode tidak ada.
        return FALLBACK;
    };
    if value.is_empty() {
        // String kosong falsy di JS → fallback.
        return FALLBACK;
    }
    match value.to_ascii_lowercase().as_str() {
        "kick" => "kick",
        "remove" | "delete" => "remove",
        _ => FALLBACK,
    }
}

/// `LEET_CHAR_MAP` (group-protection.js:95-110) — angka & simbol → huruf.
fn leet_char(c: char) -> Option<char> {
    match c {
        '0' => Some('o'),
        '1' => Some('i'),
        '2' => Some('z'),
        '3' => Some('e'),
        '4' => Some('a'),
        '5' => Some('s'),
        '6' => Some('g'),
        '7' => Some('t'),
        '8' => Some('b'),
        '9' => Some('g'),
        '@' => Some('a'),
        '$' => Some('s'),
        '!' => Some('i'),
        '+' => Some('t'),
        _ => None,
    }
}

/// `normalizeProtectionText` (group-protection.js:313-329).
///
/// PENTING urutannya: lowercase → pemetaan LEET → baru buang non-`[a-z0-9]`
/// jadi spasi. Karena itu `"SLOT GACOR!!!"` menjadi `"slot gacoriii"` (tanda
/// seru diubah jadi `i` SEBELUM dibuang) — perilaku JS dibiarkan apa adanya.
///
/// Deviasi yang diketahui: NFKD penuh tidak diport (crate
/// `unicode-normalization` tidak ada di dependency oktz-router). Yang tetap
/// dilakukan: buang combining marks `\u0300-\u036f` seperti `replace` JS.
/// TODO: ganti ke `unicode-normalization` bila crate itu masuk workspace.
fn normalize_protection_text(text: &str) -> String {
    let stripped: String = text
        .chars()
        .filter(|c| !('\u{300}'..='\u{36f}').contains(c))
        .collect();
    let lower = stripped.to_lowercase();
    let mapped: String = lower
        .chars()
        .map(|c| leet_char(c).unwrap_or(c))
        // `replace(/[^a-z0-9]+/g, " ")` — setiap non-alnum jadi SATU spasi
        // (run ditangani `.replace(/\s+/g, " ")` di bawah).
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() {
                c
            } else {
                ' '
            }
        })
        .collect();
    // `.replace(/\s+/g, " ").trim()` — collapse + trim sekaligus.
    let collapsed = mapped.split_whitespace().collect::<Vec<_>>().join(" ");
    compact_letter_fragments(&collapsed)
}

/// `mapped.replace(/\b(?:[a-z0-9]\s+){2,}[a-z0-9]\b/g, frag => frag tanpa
/// spasi)` (group-protection.js:326-328) — `"j u d i"` → `"judi"`,
/// `"x y z a b c"` → `"xyzabc"` (kuantifier `{2,}` serakah makan seluruh
/// run; sudah diverifikasi dengan node).
fn compact_letter_fragments(text: &str) -> String {
    static RE: OnceLock<MiniRegex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        MiniRegex::new(r"\b(?:[a-z0-9]\s+){2,}[a-z0-9]\b", "")
            .expect("pola kompaksi bawaan harus valid")
    });
    re.replace_all(text, |frag: &str| {
        frag.chars().filter(|c| !c.is_whitespace()).collect()
    })
}

/// `buildProtectionTextVariants` (group-protection.js:331-338) —
/// `[raw, normalized, compact]` tanpa duplikat dan tanpa string kosong.
fn build_protection_text_variants(text: &str) -> Vec<String> {
    let raw = text.to_lowercase().trim().to_string();
    let normalized = normalize_protection_text(text);
    // compact = normalized.replace(/[^a-z0-9]+/g, "")
    let compact: String = normalized
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect();
    let mut out: Vec<String> = Vec::new();
    for v in [raw, normalized, compact] {
        if !v.is_empty() && !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

/// Satu entri tabel pola regex (`JUDOL_PATTERNS` / `PHISHING_PATTERNS`).
struct TextPattern {
    /// Sumber regex persis JS, tanpa `/.../ ` dan flag `i` implisit.
    source: &'static str,
    score: u32,
    label: &'static str,
}

/// Satu entri tabel kata compact (`*_COMPACT_KEYWORDS`) — dicocokkan sebagai
/// substring (`source.includes(keyword)`).
struct TextKeyword {
    keyword: &'static str,
    score: u32,
    label: &'static str,
}

/// `JUDOL_PATTERNS` (group-protection.js:111-146).
const JUDOL_PATTERNS: &[TextPattern] = &[
    TextPattern {
        source: r"\bjud[iol]{1,4}\b",
        score: 3,
        label: "judi",
    },
    TextPattern {
        source: r"\bslot\b",
        score: 3,
        label: "slot",
    },
    TextPattern {
        source: r"\bgacor\b",
        score: 3,
        label: "gacor",
    },
    TextPattern {
        source: r"\bslot\s+gacor\b",
        score: 4,
        label: "slot gacor",
    },
    TextPattern {
        source: r"\bmax\s?win\b",
        score: 3,
        label: "maxwin",
    },
    TextPattern {
        source: r"\bscatter\b",
        score: 2,
        label: "scatter",
    },
    TextPattern {
        source: r"\btogel\b",
        score: 3,
        label: "togel",
    },
    TextPattern {
        source: r"\bcasino\b",
        score: 3,
        label: "casino",
    },
    TextPattern {
        source: r"\brtp\b",
        score: 2,
        label: "rtp",
    },
    TextPattern {
        source: r"\bpragmatic\b",
        score: 2,
        label: "pragmatic",
    },
    TextPattern {
        source: r"\bpg\s?soft\b",
        score: 2,
        label: "pgsoft",
    },
    TextPattern {
        source: r"\b(habanero|joker|spadegaming|microgaming)\b",
        score: 2,
        label: "provider",
    },
    TextPattern {
        source: r"\bjackpot\b",
        score: 2,
        label: "jackpot",
    },
    TextPattern {
        source: r"\bfree\s?spin\b",
        score: 2,
        label: "free spin",
    },
    TextPattern {
        source: r"\bspin\s?gratis\b",
        score: 2,
        label: "spin gratis",
    },
    TextPattern {
        source: r"\bpola\s?slot\b",
        score: 2,
        label: "pola slot",
    },
    TextPattern {
        source: r"\bdeposit\b",
        score: 1,
        label: "deposit",
    },
    TextPattern {
        source: r"\bdepo\b",
        score: 1,
        label: "depo",
    },
    TextPattern {
        source: r"\bwithdraw\b",
        score: 1,
        label: "withdraw",
    },
    TextPattern {
        source: r"\bwd\b",
        score: 1,
        label: "wd",
    },
    TextPattern {
        source: r"bonus\s+new\s+member",
        score: 2,
        label: "bonus new member",
    },
    TextPattern {
        source: r"bonus\s+(member|new\s*member)",
        score: 2,
        label: "bonus member",
    },
    TextPattern {
        source: r"link\s+alternatif",
        score: 2,
        label: "link alternatif",
    },
    TextPattern {
        source: r"bandar\s+(slot|togel|judi)",
        score: 2,
        label: "bandar",
    },
    TextPattern {
        source: r"\bbet\s?(kecil|besar)?\b",
        score: 1,
        label: "bet",
    },
    TextPattern {
        source: r"\btaruhan\b",
        score: 2,
        label: "taruhan",
    },
];

/// `JUDOL_COMPACT_KEYWORDS` (group-protection.js:147-170).
const JUDOL_COMPACT_KEYWORDS: &[TextKeyword] = &[
    TextKeyword {
        keyword: "judi",
        score: 3,
        label: "judi",
    },
    TextKeyword {
        keyword: "judol",
        score: 3,
        label: "judol",
    },
    TextKeyword {
        keyword: "slot",
        score: 3,
        label: "slot",
    },
    TextKeyword {
        keyword: "gacor",
        score: 3,
        label: "gacor",
    },
    TextKeyword {
        keyword: "slotgacor",
        score: 4,
        label: "slot gacor",
    },
    TextKeyword {
        keyword: "maxwin",
        score: 3,
        label: "maxwin",
    },
    TextKeyword {
        keyword: "scatter",
        score: 2,
        label: "scatter",
    },
    TextKeyword {
        keyword: "togel",
        score: 3,
        label: "togel",
    },
    TextKeyword {
        keyword: "casino",
        score: 3,
        label: "casino",
    },
    TextKeyword {
        keyword: "rtp",
        score: 2,
        label: "rtp",
    },
    TextKeyword {
        keyword: "pragmatic",
        score: 2,
        label: "pragmatic",
    },
    TextKeyword {
        keyword: "pgsoft",
        score: 2,
        label: "pgsoft",
    },
    TextKeyword {
        keyword: "jackpot",
        score: 2,
        label: "jackpot",
    },
    TextKeyword {
        keyword: "freespin",
        score: 2,
        label: "free spin",
    },
    TextKeyword {
        keyword: "spingratis",
        score: 2,
        label: "spin gratis",
    },
    TextKeyword {
        keyword: "polaslot",
        score: 2,
        label: "pola slot",
    },
    TextKeyword {
        keyword: "bonusmember",
        score: 2,
        label: "bonus member",
    },
    TextKeyword {
        keyword: "bonusnewmember",
        score: 2,
        label: "bonus new member",
    },
    TextKeyword {
        keyword: "linkalternatif",
        score: 2,
        label: "link alternatif",
    },
    TextKeyword {
        keyword: "bandarslot",
        score: 2,
        label: "bandar slot",
    },
    TextKeyword {
        keyword: "bandartogel",
        score: 2,
        label: "bandar togel",
    },
    TextKeyword {
        keyword: "bandarjudi",
        score: 2,
        label: "bandar judi",
    },
];

/// `PHISHING_PATTERNS` (group-protection.js:171-223).
const PHISHING_PATTERNS: &[TextPattern] = &[
    TextPattern {
        source: r"klik\s+(link|tautan)",
        score: 2,
        label: "klik link",
    },
    TextPattern {
        source: r"(tap|tekan)\s+(link|tautan)",
        score: 2,
        label: "tap link",
    },
    TextPattern {
        source: r"verifikasi\s+(akun|nomor|rekening|wallet|email|data)",
        score: 3,
        label: "verifikasi",
    },
    TextPattern {
        source: r"(login|log\s?in|signin|sign\s?in)\s+(disini|di\s+sini|sekarang|ulang)",
        score: 3,
        label: "login",
    },
    TextPattern {
        source: r"masuk\s+(disini|di\s+sini)",
        score: 2,
        label: "masuk",
    },
    TextPattern {
        source: r"\b(otp|pin|password|passcode|kode\s?otp|kode\s?verifikasi)\b",
        score: 2,
        label: "credential",
    },
    TextPattern {
        source: r"akun\s+(dibatasi|terblokir|bermasalah|expired|kedaluwarsa)",
        score: 2,
        label: "akun",
    },
    TextPattern {
        source: r"klaim\s+(hadiah|saldo|voucher|bonus)",
        score: 2,
        label: "klaim",
    },
    TextPattern {
        source: r"(saldo|dana|ovo|gopay|shopeepay)\s+(gratis|kaget|masuk)",
        score: 2,
        label: "saldo",
    },
    TextPattern {
        source: r"hadiah\s+(gratis|menarik|langsung)",
        score: 1,
        label: "hadiah",
    },
    TextPattern {
        source: r"cek\s+(resi|paket|rekening)",
        score: 1,
        label: "cek",
    },
    TextPattern {
        source: r"reset\s+(password|akun)",
        score: 3,
        label: "reset",
    },
    TextPattern {
        source: r"pulihkan\s+akun",
        score: 3,
        label: "pulihkan akun",
    },
    TextPattern {
        source: r"update\s+(akun|rekening|wallet|data)",
        score: 2,
        label: "update akun",
    },
    TextPattern {
        source: r"konfirmasi\s+(akun|identitas|rekening|data)",
        score: 2,
        label: "konfirmasi",
    },
    TextPattern {
        source: r"isi\s+form(ulir)?",
        score: 2,
        label: "form",
    },
    TextPattern {
        source: r"akses\s+(sekarang|link)",
        score: 1,
        label: "akses",
    },
    TextPattern {
        source: r"hadiah\s+menanti",
        score: 1,
        label: "hadiah menanti",
    },
];

/// `PHISHING_COMPACT_KEYWORDS` (group-protection.js:224-251).
const PHISHING_COMPACT_KEYWORDS: &[TextKeyword] = &[
    TextKeyword {
        keyword: "kliklink",
        score: 2,
        label: "klik link",
    },
    TextKeyword {
        keyword: "kliktautan",
        score: 2,
        label: "klik tautan",
    },
    TextKeyword {
        keyword: "taplink",
        score: 2,
        label: "tap link",
    },
    TextKeyword {
        keyword: "tekanlink",
        score: 2,
        label: "tekan link",
    },
    TextKeyword {
        keyword: "verifikasiakun",
        score: 3,
        label: "verifikasi akun",
    },
    TextKeyword {
        keyword: "verifikasinomor",
        score: 3,
        label: "verifikasi nomor",
    },
    TextKeyword {
        keyword: "verifikasirekening",
        score: 3,
        label: "verifikasi rekening",
    },
    TextKeyword {
        keyword: "verifikasiwallet",
        score: 3,
        label: "verifikasi wallet",
    },
    TextKeyword {
        keyword: "loginsekarang",
        score: 3,
        label: "login sekarang",
    },
    TextKeyword {
        keyword: "logindisini",
        score: 3,
        label: "login disini",
    },
    TextKeyword {
        keyword: "masukdisini",
        score: 2,
        label: "masuk disini",
    },
    TextKeyword {
        keyword: "kodeotp",
        score: 2,
        label: "kode otp",
    },
    TextKeyword {
        keyword: "kodeverifikasi",
        score: 2,
        label: "kode verifikasi",
    },
    TextKeyword {
        keyword: "resetpassword",
        score: 3,
        label: "reset password",
    },
    TextKeyword {
        keyword: "resetakun",
        score: 3,
        label: "reset akun",
    },
    TextKeyword {
        keyword: "pulihkanakun",
        score: 3,
        label: "pulihkan akun",
    },
    TextKeyword {
        keyword: "klaimhadiah",
        score: 2,
        label: "klaim hadiah",
    },
    TextKeyword {
        keyword: "klaimsaldo",
        score: 2,
        label: "klaim saldo",
    },
    TextKeyword {
        keyword: "klaimbonus",
        score: 2,
        label: "klaim bonus",
    },
    TextKeyword {
        keyword: "saldogratis",
        score: 2,
        label: "saldo gratis",
    },
    TextKeyword {
        keyword: "saldokaget",
        score: 2,
        label: "saldo kaget",
    },
    TextKeyword {
        keyword: "updateakun",
        score: 2,
        label: "update akun",
    },
    TextKeyword {
        keyword: "updaterekening",
        score: 2,
        label: "update rekening",
    },
    TextKeyword {
        keyword: "konfirmasiakun",
        score: 2,
        label: "konfirmasi akun",
    },
    TextKeyword {
        keyword: "konfirmasiidentitas",
        score: 2,
        label: "konfirmasi identitas",
    },
    TextKeyword {
        keyword: "isiform",
        score: 2,
        label: "isi form",
    },
];

/// Host shortener untuk `SHORTENER_PATTERN` (group-protection.js:252-253).
const SHORTENER_HOSTS: &[&str] = &[
    "bit.ly",
    "tinyurl.com",
    "cutt.ly",
    "s.id",
    "t.co",
    "goo.gl",
    "is.gd",
    "rebrand.ly",
    "linktr.ee",
];

/// Hasil `detectJudol` / `detectPhishing`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectResult {
    pub matched: bool,
    pub has_link: bool,
    pub score: u32,
    pub matches: Vec<String>,
}

/// `summarizeScoreHits` (group-protection.js:340-356): label sama di-deup
/// dengan skor terbesar yang menang, urutan sesuai kemunculan pertama.
fn summarize_score_hits(hits: Vec<(String, u32)>) -> (u32, Vec<String>) {
    let mut kept: Vec<(String, u32)> = Vec::new();
    for (label, score) in hits {
        match kept.iter_mut().find(|(l, _)| *l == label) {
            Some((_, s)) => {
                if score > *s {
                    *s = score;
                }
            }
            None => kept.push((label, score)),
        }
    }
    let score = kept.iter().map(|(_, s)| s).sum();
    (score, kept.into_iter().map(|(l, _)| l).collect())
}

/// Pola bawaan sudah selesai dikompilasi — tabel JS di-precompile saat modul
/// dimuat; di sini versi malas dengan `OnceLock` (sekali per proses).
struct CompiledPattern {
    re: MiniRegex,
    score: u32,
    label: &'static str,
}

fn compile_patterns(specs: &[TextPattern]) -> Vec<CompiledPattern> {
    specs
        .iter()
        .map(|p| CompiledPattern {
            re: MiniRegex::new(p.source, "i").expect("pola bawaan harus bisa dikompilasi"),
            score: p.score,
            label: p.label,
        })
        .collect()
}

fn judol_compiled() -> &'static [CompiledPattern] {
    static CACHE: OnceLock<Vec<CompiledPattern>> = OnceLock::new();
    CACHE.get_or_init(|| compile_patterns(JUDOL_PATTERNS))
}

fn phishing_compiled() -> &'static [CompiledPattern] {
    static CACHE: OnceLock<Vec<CompiledPattern>> = OnceLock::new();
    CACHE.get_or_init(|| compile_patterns(PHISHING_PATTERNS))
}

/// `scorePatterns` (group-protection.js:358-378) — per pola, varian PERTAMA
/// yang cocok menang (`break`), lalu label di-summarize.
fn score_patterns(variants: &[String], patterns: &[CompiledPattern]) -> (u32, Vec<String>) {
    let mut hits = Vec::new();
    for p in patterns {
        for variant in variants {
            if p.re.test(variant) {
                hits.push((p.label.to_string(), p.score));
                break;
            }
        }
    }
    summarize_score_hits(hits)
}

/// `scoreKeywords` (group-protection.js:380-391) — substring mentah.
fn score_keywords(source: &str, keywords: &[TextKeyword]) -> (u32, Vec<String>) {
    let hits = keywords
        .iter()
        .filter(|k| source.contains(k.keyword))
        .map(|k| (k.label.to_string(), k.score))
        .collect();
    summarize_score_hits(hits)
}

/// `detectJudol` (group-protection.js:393-407).
///
/// Ambang: `totalScore >= 3 || (hasLink && totalScore >= 2)` — dan skor
/// memang DIHITUNG GANDA antara pola regex dan kata compact (jumlah
/// `patternResult.score + compactResult.score` tanpa dedup antar-dua; sudah
/// diverifikasi dengan node: "slot gacor" → 20).
pub fn detect_judol(text: &str) -> DetectResult {
    if text.is_empty() {
        return DetectResult {
            matched: false,
            has_link: false,
            score: 0,
            matches: vec![],
        };
    }
    let variants = build_protection_text_variants(text);
    // `variants[variants.length - 1] || ""` — varian terakhir = bentuk compact.
    let compact = variants.last().cloned().unwrap_or_default();
    let has_link = contains_any_link(text);
    let (pattern_score, pattern_labels) = score_patterns(&variants, judol_compiled());
    let (compact_score, compact_labels) = score_keywords(&compact, JUDOL_COMPACT_KEYWORDS);
    // `result` di JS hanya dipakai untuk daftar `matches` (skor 0 per label).
    let merged = pattern_labels
        .iter()
        .chain(compact_labels.iter())
        .map(|l| (l.clone(), 0))
        .collect();
    let (_, matches) = summarize_score_hits(merged);
    let total = pattern_score + compact_score;
    let matched = total >= 3 || (has_link && total >= 2);
    DetectResult {
        matched,
        has_link,
        score: total,
        matches,
    }
}

/// `detectPhishing` (group-protection.js:409-440).
///
/// Ambang: `(hasLink && score >= 2) || score >= 3`. Empat pola tambahan
/// (shortener, IP, userinfo, punycode) diuji pada teks MENTAH dan masing-masing
/// menambah 2 (:421-436).
pub fn detect_phishing(text: &str) -> DetectResult {
    if text.is_empty() {
        return DetectResult {
            matched: false,
            has_link: false,
            score: 0,
            matches: vec![],
        };
    }
    let variants = build_protection_text_variants(text);
    let compact = variants.last().cloned().unwrap_or_default();
    let has_link = contains_any_link(text);
    let (mut score, mut labels) = score_patterns(&variants, phishing_compiled());
    // `[...new Set([...patternResult.matches, ...compactResult.matches])]`
    // — dedup dengan urutan sisip dipertahankan.
    let mut seen: Vec<String> = Vec::new();
    for l in labels.drain(..) {
        if !seen.contains(&l) {
            seen.push(l);
        }
    }
    let (compact_score, compact_labels) = score_keywords(&compact, PHISHING_COMPACT_KEYWORDS);
    score += compact_score;
    for l in compact_labels {
        if !seen.contains(&l) {
            seen.push(l);
        }
    }
    if has_shortener_link(text) {
        score += 2;
        seen.push("shortener".to_string());
    }
    if is_ip_url(text) {
        score += 2;
        seen.push("ip-url".to_string());
    }
    if is_at_in_url(text) {
        score += 2;
        seen.push("url-userinfo".to_string());
    }
    if is_punycode(text) {
        score += 2;
        seen.push("punycode".to_string());
    }
    let matched = (has_link && score >= 2) || score >= 3;
    DetectResult {
        matched,
        has_link,
        score,
        matches: seen,
    }
}

/// `SHORTENER_PATTERN` (group-protection.js:252-253) — host + `/` di mana pun
/// (case-insensitive). `contains` pada teks lowercase ekuivalen dengan regex
/// tanpa anchor.
fn has_shortener_link(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    SHORTENER_HOSTS
        .iter()
        .any(|h| lower.contains(&format!("{h}/")))
}

/// `IP_URL_PATTERN` (group-protection.js:254-255):
/// `(?:https?:\/\/)?(?:\d{1,3}\.){3}\d{1,3}(?::\d+)?(?:\/|\b)`.
/// Port opsional diuji dua kali (dengan/tanpa) — setara dengan backtrack JS
/// karena batas `\b` setelah digit hanya bisa tercapai lewat char non-word.
fn is_ip_url(text: &str) -> bool {
    // `IP_URL_PATTERN` memakai flag `i`; skema HTTP boleh kapital.
    let chars: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let n = chars.len();
    // Panjang run digit di `i`, maksimal 3.
    let digits_at = |i: usize| -> usize {
        let mut j = i;
        while j < n && chars[j].is_ascii_digit() && j - i < 3 {
            j += 1;
        }
        j - i
    };
    for start in 0..n {
        let mut i = start;
        if chars[start..].starts_with(&['h', 't', 't', 'p', 's', ':', '/', '/']) {
            i = start + 8;
        } else if chars[start..].starts_with(&['h', 't', 't', 'p', ':', '/', '/']) {
            i = start + 7;
        }
        let mut ok = true;
        for _ in 0..3 {
            let d = digits_at(i);
            if d == 0 || chars.get(i + d) != Some(&'.') {
                ok = false;
                break;
            }
            i += d + 1;
        }
        if !ok {
            continue;
        }
        let d = digits_at(i);
        if d == 0 {
            continue;
        }
        let after_number = i + d;
        for with_port in [true, false] {
            let mut j = after_number;
            if with_port {
                if chars.get(j) != Some(&':') {
                    continue;
                }
                j += 1;
                let port_start = j;
                while j < n && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if j == port_start {
                    continue;
                }
            }
            if chars.get(j) == Some(&'/') {
                return true;
            }
            // `\b`: char sebelumnya digit (word) → boundary iff char di `j`
            // bukan word (termasuk akhir teks).
            if !chars.get(j).copied().map_or(false, is_word) {
                return true;
            }
        }
    }
    false
}

/// `AT_IN_URL_PATTERN` (group-protection.js:256): `https?:\/\/[^\s/]+@[^\s]+`.
fn is_at_in_url(text: &str) -> bool {
    // `AT_IN_URL_PATTERN` memakai flag `i`; skema HTTP boleh kapital.
    let chars: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let n = chars.len();
    for i in 0..n {
        let scheme_len = if chars[i..].starts_with(&['h', 't', 't', 'p', 's', ':', '/', '/']) {
            8
        } else if chars[i..].starts_with(&['h', 't', 't', 'p', ':', '/', '/']) {
            7
        } else {
            continue;
        };
        // `[^\s/]+@` — kelas tidak boleh melewati spasi atau `/`.
        let mut k = i + scheme_len;
        while k < n {
            let c = chars[k];
            if is_js_space(c) || c == '/' {
                break;
            }
            if c == '@' && k > i + scheme_len {
                // `[^\s]+` butuh minimal satu char non-spasi setelah '@'.
                if k + 1 < n && !is_js_space(chars[k + 1]) {
                    return true;
                }
            }
            k += 1;
        }
    }
    false
}

/// `PUNYCODE_PATTERN` (group-protection.js:257): `xn--[a-z0-9-]+`.
fn is_punycode(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let n = chars.len();
    for i in 0..n.saturating_sub(4) {
        if chars[i] == 'x' && chars[i + 1] == 'n' && chars[i + 2] == '-' && chars[i + 3] == '-' {
            if chars[i + 4..]
                .iter()
                .any(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
            {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Mesin regex mini — pengganti `new RegExp` (crate `regex` tidak ada di
// dependency oktz-router, lihat komentar deteksi link di atas).
//
// Cakupan persis yang dibutuhkan tabel pola + rule AntiCustom: literal,
// kelas `[...]`, singkatan `\d \w \s \D \W \S`, titik, `\b`/`\B`, `^`/`$`,
// kelompok dengan alternasi `(a|b)` / `(?:...)` / `(?<nama>...)`, kuantifier
// `* + ? {n} {n,} {n,m}` (+ lazy `?`), dan escape `\xHH`/`\uHHHH`/`\u{...}`.
// Tidak didukung (fail-closed → rule dianggap tidak valid): backreference,
// lookahead/lookbehind, flag `y`, dan mode unicode-bytes. Pola yang
// mengandung bentuk itu di JS dianalisis aman tetapi di sini dinyatakan
// tidak valid oleh `analyze_custom_pattern` → rule inert (tidak pernah
// match), bukan salah match.
//
// Eksekusi = mesin backtracking VM dengan memoisasi kegagalan
// (`visited`: pasangan (pc, pos) tidak pernah dieksplorasi dua kali). Tanpa
// backreference, hasil dari satu state hanya bergantung pada state itu,
// jadi memoisasi aman DAN sekaligus menjamin waktu linear-ish — pengganti
// dua pertahanan JS (analisis statis + batas waktu 20 ms) untuk mencegah
// ReDoS pada pola admin yang lolos analisis.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClassItem {
    Char(char),
    Range(char, char),
    /// `\d` (false) / `\D` (true) di dalam kelas.
    Digit(bool),
    Word(bool),
    Space(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassSpec {
    negated: bool,
    items: Vec<ClassItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AtomAst {
    Char(char),
    Any,
    Class(ClassSpec),
    /// `\b` (true) / `\B` (false).
    WordBoundary(bool),
    AnchorStart,
    AnchorEnd,
    Group(Vec<Vec<Node>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Atom(AtomAst),
    Repeat {
        atom: Box<Node>,
        min: u32,
        max: Option<u32>,
        lazy: bool,
    },
}

/// Instruksi VM hasil kompaksi AST.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Inst {
    Char {
        c: char,
        fold: bool,
    },
    Any {
        dot_all: bool,
    },
    Class {
        spec: ClassSpec,
        fold: bool,
    },
    WordBoundary(bool),
    AnchorStart {
        multiline: bool,
    },
    AnchorEnd {
        multiline: bool,
    },
    /// Coba `pc` pertama lebih dulu (urutan prioritas = urutan cabang JS).
    Split(usize, usize),
    Jump(usize),
    Match,
}

/// Batas instruksi hasil unroll kuantifier `{n,m}` — pola yang butuh lebih
/// dinyatakan tidak valid (fail-closed) alih-alih memakan memori.
const MAX_INSTS: usize = 50_000;

struct Parser<'a> {
    src: &'a [char],
    pos: usize,
    ignore_case: bool,
    dot_all: bool,
    multiline: bool,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn peek_at(&self, off: usize) -> Option<char> {
        self.src.get(self.pos + off).copied()
    }

    /// Baca 2/4 digit hex di `pos` (tanpa menggeser pos).
    fn hex_here(&self, len: usize) -> Option<char> {
        let slice = self.src.get(self.pos..self.pos + len)?;
        if slice.len() < len || !slice.iter().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let text: String = slice.iter().collect();
        char::from_u32(u32::from_str_radix(&text, 16).ok()?)
    }

    /// `\cX` → karakter kontrol; `X` bukan huruf → literal `c` (Annex B).
    fn control_escape(&mut self) -> Result<char, ()> {
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() => {
                self.pos += 1;
                Ok((c as u8 & 0x1F) as char)
            }
            _ => Ok('c'),
        }
    }

    fn parse_alt(&mut self) -> Result<Vec<Vec<Node>>, ()> {
        let mut alts = vec![self.parse_seq()?];
        while self.peek() == Some('|') {
            self.pos += 1;
            alts.push(self.parse_seq()?);
        }
        Ok(alts)
    }

    fn parse_seq(&mut self) -> Result<Vec<Node>, ()> {
        let mut nodes = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.parse_atom()?;
            nodes.push(self.parse_quantifier(atom)?);
        }
        Ok(nodes)
    }

    /// Kuantifier setelah atom. `{` yang tidak membentuk `{n[,[m]]}` tetap
    /// literal (Annex B) — persis `readQuantifier` yang mengembalikan null.
    fn parse_quantifier(&mut self, atom: Node) -> Result<Node, ()> {
        let (min, max) = match self.peek() {
            Some('*') => {
                self.pos += 1;
                (0, None)
            }
            Some('+') => {
                self.pos += 1;
                (1, None)
            }
            Some('?') => {
                self.pos += 1;
                (0, Some(1))
            }
            Some('{') => match self.try_brace()? {
                Some((min, max, next)) => {
                    self.pos = next;
                    (min, max)
                }
                None => return Ok(atom),
            },
            _ => return Ok(atom),
        };
        let lazy = if self.peek() == Some('?') {
            self.pos += 1;
            true
        } else {
            false
        };
        Ok(Node::Repeat {
            atom: Box::new(atom),
            min,
            max,
            lazy,
        })
    }

    fn try_brace(&mut self) -> Result<Option<(u32, Option<u32>, usize)>, ()> {
        let mut i = self.pos + 1;
        let mut min_text = String::new();
        while let Some(c) = self.src.get(i) {
            if c.is_ascii_digit() {
                min_text.push(*c);
                i += 1;
            } else {
                break;
            }
        }
        if min_text.is_empty() {
            return Ok(None);
        }
        let min: u32 = min_text.parse().map_err(|_| ())?;
        if self.src.get(i) == Some(&',') {
            i += 1;
            let mut max_text = String::new();
            while let Some(c) = self.src.get(i) {
                if c.is_ascii_digit() {
                    max_text.push(*c);
                    i += 1;
                } else {
                    break;
                }
            }
            if self.src.get(i) != Some(&'}') {
                return Ok(None);
            }
            i += 1;
            // `{n,}` → max tak terbatas; `{n,}` dengan angka → Some(n).
            let max = if max_text.is_empty() {
                None
            } else {
                Some(max_text.parse().map_err(|_| ())?)
            };
            if max.is_some_and(|value| value < min) {
                return Err(());
            }
            Ok(Some((min, max, i)))
        } else if self.src.get(i) == Some(&'}') {
            Ok(Some((min, Some(min), i + 1)))
        } else {
            Ok(None)
        }
    }

    fn parse_atom(&mut self) -> Result<Node, ()> {
        let c = self.peek().ok_or(())?;
        let atom = match c {
            '(' => {
                self.pos += 1;
                if self.peek() == Some('?') {
                    match self.peek_at(1) {
                        Some(':') => self.pos += 2,
                        // Lookahead/lookbehind tidak didukung → tidak valid.
                        Some('=') | Some('!') => return Err(()),
                        Some('<') if matches!(self.peek_at(2), Some('=') | Some('!')) => {
                            return Err(());
                        }
                        // `(?<nama>...)` — lompati nama, semantik sama `(`.
                        Some('<') => {
                            self.pos += 2;
                            while let Some(ch) = self.peek() {
                                self.pos += 1;
                                if ch == '>' {
                                    break;
                                }
                            }
                        }
                        _ => return Err(()),
                    }
                }
                let alts = self.parse_alt()?;
                if self.peek() != Some(')') {
                    return Err(());
                }
                self.pos += 1;
                AtomAst::Group(alts)
            }
            '[' => self.parse_class()?,
            '.' => {
                self.pos += 1;
                AtomAst::Any
            }
            '^' => {
                self.pos += 1;
                AtomAst::AnchorStart
            }
            '$' => {
                self.pos += 1;
                AtomAst::AnchorEnd
            }
            '\\' => self.parse_escape_atom()?,
            // Kuantifier tanpa atom → SyntaxError di JS.
            '*' | '+' | '?' => return Err(()),
            c => {
                self.pos += 1;
                AtomAst::Char(c)
            }
        };
        Ok(Node::Atom(atom))
    }

    fn parse_escape_atom(&mut self) -> Result<AtomAst, ()> {
        self.pos += 1; // backslash
        let c = self.peek().ok_or(())?;
        self.pos += 1;
        Ok(match c {
            'd' => AtomAst::Class(ClassSpec {
                negated: false,
                items: vec![ClassItem::Digit(false)],
            }),
            'D' => AtomAst::Class(ClassSpec {
                negated: false,
                items: vec![ClassItem::Digit(true)],
            }),
            'w' => AtomAst::Class(ClassSpec {
                negated: false,
                items: vec![ClassItem::Word(false)],
            }),
            'W' => AtomAst::Class(ClassSpec {
                negated: false,
                items: vec![ClassItem::Word(true)],
            }),
            's' => AtomAst::Class(ClassSpec {
                negated: false,
                items: vec![ClassItem::Space(false)],
            }),
            'S' => AtomAst::Class(ClassSpec {
                negated: false,
                items: vec![ClassItem::Space(true)],
            }),
            'b' => AtomAst::WordBoundary(true),
            'B' => AtomAst::WordBoundary(false),
            'n' => AtomAst::Char('\n'),
            'r' => AtomAst::Char('\r'),
            't' => AtomAst::Char('\t'),
            'f' => AtomAst::Char('\u{c}'),
            'v' => AtomAst::Char('\u{b}'),
            '0' => AtomAst::Char('\0'),
            'x' => {
                let ch = self.hex_here(2).ok_or(())?;
                self.pos += 2;
                AtomAst::Char(ch)
            }
            'u' => {
                if self.peek_at(0) == Some('{') {
                    // \u{...} — cari '}' lalu parse hex di dalamnya.
                    let close = self.src[self.pos + 1..]
                        .iter()
                        .position(|&x| x == '}')
                        .map(|p| self.pos + 1 + p)
                        .ok_or(())?;
                    let body: String = self.src[self.pos + 1..close].iter().collect();
                    if body.is_empty() || !body.chars().all(|h| h.is_ascii_hexdigit()) {
                        return Err(());
                    }
                    let ch =
                        char::from_u32(u32::from_str_radix(&body, 16).map_err(|_| ())?).ok_or(())?;
                    self.pos = close + 1;
                    AtomAst::Char(ch)
                } else {
                    let ch = self.hex_here(4).ok_or(())?;
                    self.pos += 4;
                    AtomAst::Char(ch)
                }
            }
            // Backreference tidak didukung (analyzer menolaknya juga).
            '1'..='9' => return Err(()),
            'c' => AtomAst::Char(self.control_escape()?),
            // \. \/ \- dll — escape identitas.
            other => AtomAst::Char(other),
        })
    }

    /// Satu elemen kelas: item kelas (`\d` dll) atau satu char literal.
    fn read_class_element(&mut self) -> Result<ClassItem, ()> {
        let c = self.peek().ok_or(())?;
        if c != '\\' {
            self.pos += 1;
            return Ok(ClassItem::Char(c));
        }
        self.pos += 1;
        let e = self.peek().ok_or(())?;
        self.pos += 1;
        Ok(match e {
            'd' => ClassItem::Digit(false),
            'D' => ClassItem::Digit(true),
            'w' => ClassItem::Word(false),
            'W' => ClassItem::Word(true),
            's' => ClassItem::Space(false),
            'S' => ClassItem::Space(true),
            // Di dalam kelas `\b` adalah backspace, bukan assertion (JS).
            'b' => ClassItem::Char('\u{8}'),
            'n' => ClassItem::Char('\n'),
            'r' => ClassItem::Char('\r'),
            't' => ClassItem::Char('\t'),
            'f' => ClassItem::Char('\u{c}'),
            'v' => ClassItem::Char('\u{b}'),
            '0' => ClassItem::Char('\0'),
            'x' => {
                let ch = self.hex_here(2).ok_or(())?;
                self.pos += 2;
                ClassItem::Char(ch)
            }
            'u' => {
                if self.peek_at(0) == Some('{') {
                    let close = self.src[self.pos + 1..]
                        .iter()
                        .position(|&x| x == '}')
                        .map(|p| self.pos + 1 + p)
                        .ok_or(())?;
                    let body: String = self.src[self.pos + 1..close].iter().collect();
                    if body.is_empty() || !body.chars().all(|h| h.is_ascii_hexdigit()) {
                        return Err(());
                    }
                    let ch =
                        char::from_u32(u32::from_str_radix(&body, 16).map_err(|_| ())?).ok_or(())?;
                    self.pos = close + 1;
                    ClassItem::Char(ch)
                } else {
                    let ch = self.hex_here(4).ok_or(())?;
                    self.pos += 4;
                    ClassItem::Char(ch)
                }
            }
            'c' => ClassItem::Char(self.control_escape()?),
            '1'..='9' => return Err(()),
            other => ClassItem::Char(other),
        })
    }

    fn parse_class(&mut self) -> Result<AtomAst, ()> {
        self.pos += 1; // '['
        let mut negated = false;
        if self.peek() == Some('^') {
            negated = true;
            self.pos += 1;
        }
        let mut items = Vec::new();
        let mut first = true;
        loop {
            let Some(c) = self.peek() else {
                // Kelas tanpa penutup → SyntaxError di JS.
                return Err(());
            };
            if c == ']' && !first {
                self.pos += 1;
                break;
            }
            first = false;
            let lo = self.read_class_element()?;
            // Rentang `a-z` hanya bila kedua sisi char literal dan '-' bukan
            // penutup kelas. `a-\d` di JS non-u = literal 'a', '-', lalu
            // escape dibaca iterasi berikutnya.
            if matches!(lo, ClassItem::Char(_))
                && self.peek() == Some('-')
                && self.peek_at(1).is_some()
                && self.peek_at(1) != Some(']')
            {
                let next = self.peek_at(1).unwrap();
                if next == '\\' {
                    let ClassItem::Char(lo_c) = lo else {
                        unreachable!()
                    };
                    items.push(ClassItem::Char(lo_c));
                    items.push(ClassItem::Char('-'));
                    self.pos += 1; // konsumsi '-' saja; escape dibaca berikutnya
                    continue;
                }
                self.pos += 2; // '-' dan hi
                let ClassItem::Char(lo_c) = lo else {
                    unreachable!()
                };
                // Rentang terbalik di JS non-u: '-' jadi literal.
                if next < lo_c {
                    items.push(ClassItem::Char(lo_c));
                    items.push(ClassItem::Char('-'));
                    items.push(ClassItem::Char(next));
                } else {
                    items.push(ClassItem::Range(lo_c, next));
                }
            } else {
                items.push(lo);
            }
        }
        Ok(AtomAst::Class(ClassSpec { negated, items }))
    }
}

/// Atom tidak mengonsumsi karakter (assertion) — repetition atas atom semacam
/// ini tidak mengubah apa pun, cukup dicek sekali.
fn atom_is_zero_width(node: &Node) -> bool {
    matches!(
        node,
        Node::Atom(AtomAst::WordBoundary(_) | AtomAst::AnchorStart | AtomAst::AnchorEnd)
    )
}

struct Compiler {
    insts: Vec<Inst>,
    ignore_case: bool,
    dot_all: bool,
    multiline: bool,
}

impl Compiler {
    fn compile(
        alts: &[Vec<Node>],
        ignore_case: bool,
        dot_all: bool,
        multiline: bool,
    ) -> Result<(Vec<Inst>, usize), ()> {
        let mut c = Self {
            insts: Vec::new(),
            ignore_case,
            dot_all,
            multiline,
        };
        // `Match` adalah continuation bersama untuk semua cabang. Kompilasi
        // dilakukan dengan continuation eksplisit supaya cabang alternasi
        // dan kuantifier selalu kembali ke titik yang benar setelah sukses.
        let end = c.emit(Inst::Match)?;
        let entry = c.compile_alts(alts, end)?;
        Ok((c.insts, entry))
    }

    fn emit(&mut self, inst: Inst) -> Result<usize, ()> {
        if self.insts.len() >= MAX_INSTS {
            return Err(());
        }
        self.insts.push(inst);
        Ok(self.insts.len() - 1)
    }

    /// Kompilasi alternasi dengan continuation `cont`.
    ///
    /// `Split` menyimpan cabang kedua di stack. Jika cabang pertama gagal,
    /// VM popping stack mencoba cabang berikutnya. Jika cabang pertama sukses,
    /// stack tetap menyimpan cabang alternatif sehingga continuation yang
    /// gagal masih dapat mem-backtrack ke cabang berikutnya (semantik JS).
    fn compile_alts(&mut self, alts: &[Vec<Node>], cont: usize) -> Result<usize, ()> {
        if alts.is_empty() {
            return Ok(cont);
        }
        if alts.len() == 1 {
            return self.compile_seq(&alts[0], cont);
        }

        let split = self.emit(Inst::Split(0, 0))?;
        let first = self.compile_seq(&alts[0], cont)?;
        let rest = self.compile_alts(&alts[1..], cont)?;
        self.insts[split] = Inst::Split(first, rest);
        Ok(split)
    }

    /// Kompilasi sequence dari kanan ke kiri. Continuation setiap atom adalah
    /// entry atom berikutnya; ini mencegah `a?` melompati assertion yang
    /// mengikuti kuantifier.
    fn compile_seq(&mut self, nodes: &[Node], cont: usize) -> Result<usize, ()> {
        let mut entry = cont;
        for node in nodes.iter().rev() {
            entry = self.compile_node(node, entry)?;
        }
        Ok(entry)
    }

    fn compile_node(&mut self, node: &Node, cont: usize) -> Result<usize, ()> {
        match node {
            Node::Atom(a) => self.compile_atom(a, cont),
            Node::Repeat {
                atom,
                min,
                max,
                lazy,
            } => {
                // Assertion yang diulang: JS memperlakukannya sebagai cek
                // tunggal (kuantifier tidak menambah apa pun).
                if atom_is_zero_width(atom) {
                    if *min >= 1 {
                        return self.compile_node(atom, cont);
                    }
                    return Ok(cont);
                }

                let mut next = match max {
                    Some(mx) => {
                        let extra = mx.saturating_sub(*min);
                        self.compile_optional(atom, extra, *lazy, cont)?
                    }
                    None => self.compile_unbounded(atom, *lazy, cont)?,
                };
                // Kuantifier minimum wajib: setiap salinan memakai continuation
                // ke bagian repeat berikutnya.
                for _ in 0..*min {
                    next = self.compile_node(atom, next)?;
                }
                Ok(next)
            }
        }
    }

    /// Kuantifier tak terbatas. Body kembali ke `split`, sedangkan cabang
    /// skip langsung ke continuation setelah repeat.
    fn compile_unbounded(&mut self, atom: &Node, lazy: bool, cont: usize) -> Result<usize, ()> {
        let split = self.emit(Inst::Split(0, 0))?;
        let body = self.compile_node(atom, split)?;
        self.insts[split] = if lazy {
            Inst::Split(cont, body)
        } else {
            Inst::Split(body, cont)
        };
        Ok(split)
    }

    /// Kuantifier finite `{min,max}`. Setiap opsional adalah Split dengan
    /// branch body dan branch skip; branch skip melompat ke akhir seluruh
    /// repeat, bukan ke body berikutnya.
    fn compile_optional(
        &mut self,
        atom: &Node,
        count: u32,
        lazy: bool,
        cont: usize,
    ) -> Result<usize, ()> {
        if count == 0 {
            return Ok(cont);
        }
        let split = self.emit(Inst::Split(0, 0))?;
        let body_cont = self.compile_optional(atom, count - 1, lazy, cont)?;
        let body = self.compile_node(atom, body_cont)?;
        self.insts[split] = if lazy {
            Inst::Split(cont, body)
        } else {
            Inst::Split(body, cont)
        };
        Ok(split)
    }

    fn compile_atom(&mut self, atom: &AtomAst, cont: usize) -> Result<usize, ()> {
        match atom {
            AtomAst::Group(alts) => self.compile_alts(alts, cont),
            _ => {
                let inst = match atom {
                    AtomAst::Char(c) => Inst::Char {
                        c: *c,
                        fold: self.ignore_case,
                    },
                    AtomAst::Any => Inst::Any {
                        dot_all: self.dot_all,
                    },
                    AtomAst::Class(spec) => Inst::Class {
                        spec: spec.clone(),
                        fold: self.ignore_case,
                    },
                    AtomAst::WordBoundary(b) => Inst::WordBoundary(*b),
                    AtomAst::AnchorStart => Inst::AnchorStart {
                        multiline: self.multiline,
                    },
                    AtomAst::AnchorEnd => Inst::AnchorEnd {
                        multiline: self.multiline,
                    },
                    AtomAst::Group(_) => unreachable!(),
                };
                let at = self.emit(inst)?;
                self.emit(Inst::Jump(cont))?;
                Ok(at)
            }
        }
    }
}

/// Terminator baris ECMAScript: LF, CR, U+2028, dan U+2029.
fn is_js_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// `\s` versi JS: daftar whitespace Unicode yang diakui RegExp.
fn is_js_space(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

fn class_item_hit(item: &ClassItem, c: char) -> bool {
    match item {
        ClassItem::Char(x) => *x == c,
        ClassItem::Range(a, b) => *a <= c && c <= *b,
        ClassItem::Digit(neg) => c.is_ascii_digit() != *neg,
        ClassItem::Word(neg) => (c.is_ascii_alphanumeric() || c == '_') != *neg,
        ClassItem::Space(neg) => is_js_space(c) != *neg,
    }
}

/// Uji char terhadap kelas, dengan fold sederhana bila flag `i` aktif.
fn class_matches(spec: &ClassSpec, c: char, fold: bool) -> bool {
    let raw_hit = |ch: char| spec.items.iter().any(|item| class_item_hit(item, ch));
    let mut hit = raw_hit(c);
    if !hit && fold {
        if let Some(lo) = c.to_lowercase().next() {
            hit = raw_hit(lo);
        }
        if !hit {
            if let Some(up) = c.to_uppercase().next() {
                hit = raw_hit(up);
            }
        }
    }
    hit != spec.negated
}

fn char_eq_fold(a: char, b: char, fold: bool) -> bool {
    if a == b {
        return true;
    }
    fold && a.to_lowercase().eq(b.to_lowercase())
}

/// Regex terkompilasi — imutabel dan `Clone` (untuk cache rule custom).
#[derive(Debug, Clone)]
pub struct MiniRegex {
    insts: Vec<Inst>,
    /// Entry program; `Match` berada di indeks 0, sedangkan entry bisa
    /// berada setelah instruksi continuation.
    entry: usize,
}

impl MiniRegex {
    fn new(source: &str, flags: &str) -> Result<Self, ()> {
        let chars: Vec<char> = source.chars().collect();
        let mut p = Parser {
            src: &chars,
            pos: 0,
            ignore_case: flags.contains('i'),
            dot_all: flags.contains('s'),
            multiline: flags.contains('m'),
        };
        let alts = p.parse_alt()?;
        // Sisa karakter (mis. `)` liar) = pola tidak valid, seperti JS.
        if p.pos != chars.len() {
            return Err(());
        }
        let (insts, entry) = Compiler::compile(&alts, p.ignore_case, p.dot_all, p.multiline)?;
        Ok(Self { insts, entry })
    }

    /// `.test()` JS — cocok di posisi mana pun.
    fn test(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        self.find_from(&chars, 0).is_some()
    }

    /// Kembalikan (mulai, selesai) match pertama mulai dari `from`
    /// (indeks char), persis semantik `RegExp.exec` dengan `lastIndex`.
    fn find_from(&self, chars: &[char], from: usize) -> Option<(usize, usize)> {
        let mut vm = Vm {
            prog: &self.insts,
            chars,
            entry: self.entry,
            visited: HashSet::new(),
        };
        for start in from..=chars.len() {
            if let Some(end) = vm.run(start) {
                return Some((start, end));
            }
        }
        None
    }

    /// `String.replace(/.../g, f)` — match tidak tumpang-tindih, teks hasil
    /// penggantian tidak dipindai ulang, sesuai perilaku JS.
    fn replace_all(&self, text: &str, f: impl Fn(&str) -> String) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::new();
        let mut copied = 0usize;
        let mut from = 0usize;
        while from <= chars.len() {
            let Some((s, e)) = self.find_from(&chars, from) else {
                break;
            };
            out.extend(&chars[copied..s]);
            out.push_str(&f(&chars[s..e].iter().collect::<String>()));
            copied = e;
            // Match zero-width maju satu supaya loop tidak macet (JS juga).
            from = if e > s { e } else { e + 1 };
        }
        out.extend(&chars[copied.min(chars.len())..]);
        out
    }
}

/// Mesin backtracking dengan memoisasi kegagalan.
struct Vm<'p> {
    prog: &'p [Inst],
    chars: &'p [char],
    entry: usize,
    visited: HashSet<(usize, usize)>,
}

impl<'p> Vm<'p> {
    /// Jalankan dari `start`; kembalikan posisi SELESAI match bila berhasil.
    fn run(&mut self, start: usize) -> Option<usize> {
        let mut stack: Vec<(usize, usize)> = vec![(self.entry, start)];
        while let Some((mut pc, mut pos)) = stack.pop() {
            loop {
                // State (pc, pos) yang pernah dieksplorasi tidak pernah
                // berujung lain — lewati (juga memutus loop zero-width).
                if !self.visited.insert((pc, pos)) {
                    break;
                }
                match &self.prog[pc] {
                    Inst::Match => return Some(pos),
                    Inst::Char { c, fold } => {
                        let Some(&got) = self.chars.get(pos) else {
                            break;
                        };
                        if char_eq_fold(got, *c, *fold) {
                            pc += 1;
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    Inst::Any { dot_all } => {
                        let Some(&got) = self.chars.get(pos) else {
                            break;
                        };
                        if !*dot_all && is_js_line_terminator(got) {
                            break;
                        }
                        pc += 1;
                        pos += 1;
                    }
                    Inst::Class { spec, fold } => {
                        let Some(&got) = self.chars.get(pos) else {
                            break;
                        };
                        if class_matches(spec, got, *fold) {
                            pc += 1;
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    Inst::WordBoundary(expected) => {
                        let before = pos > 0 && is_word(self.chars[pos - 1]);
                        let after = pos < self.chars.len() && is_word(self.chars[pos]);
                        if (before != after) == *expected {
                            pc += 1;
                        } else {
                            break;
                        }
                    }
                    Inst::AnchorStart { multiline } => {
                        let after_terminator = pos > 0
                            && self
                                .chars
                                .get(pos - 1)
                                .is_some_and(|c| is_js_line_terminator(*c));
                        if pos == 0 || (*multiline && after_terminator) {
                            pc += 1;
                        } else {
                            break;
                        }
                    }
                    Inst::AnchorEnd { multiline } => {
                        let before_terminator = self
                            .chars
                            .get(pos)
                            .is_some_and(|c| is_js_line_terminator(*c));
                        if pos == self.chars.len() || (*multiline && before_terminator) {
                            pc += 1;
                        } else {
                            break;
                        }
                    }
                    Inst::Split(a, b) => {
                        // Alternatif b ditumpuk; thread sekarang lanjut ke a.
                        stack.push((*b, pos));
                        pc = *a;
                    }
                    Inst::Jump(t) => pc = *t,
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Analisis keamanan pola AntiCustom — port `analyzeCustomPattern` dan
// kawan-kawan (group-protection.js:525-1138). Menolak (fail-closed) pola
// buatan admin yang bisa membuat `.test()` backtracking eksponensial:
// kuantifier bersarang, isi kelompok yang diulang masih bisa kosong,
// alternasi overlap di bawah kuantifier, kuantifier bertumpuk yang overlap,
// backreference, dan pola terlalu panjang.
// ---------------------------------------------------------------------------

pub const CUSTOM_MAX_PATTERN_LEN: usize = 120;
pub const CUSTOM_MAX_SUBJECT_LEN: usize = 512;
pub const CUSTOM_MAX_RULES: usize = 50;
pub const CUSTOM_MAX_UNBOUNDED_QUANTIFIERS: usize = 6;
/// Milidetik — `CUSTOM_MATCH_BUDGET_MS`.
pub const CUSTOM_MATCH_BUDGET_MS: u64 = 20;
const CUSTOM_REGEX_CACHE_CAP: usize = 200;

/// Apakah satu set karakter bisa kena whitespace —
/// `"only"` = hanya whitespace (`\s`), `"never"` = tidak pernah (`\S`),
/// `"maybe"` = tidak diketahui / semua karakter (`.`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WsMode {
    Only,
    Never,
    Maybe,
}

/// Over-approximasi set karakter — padanan objek `{ any, chars, ws }` JS.
#[derive(Debug, Clone)]
struct CharSet {
    any: bool,
    chars: BTreeSet<char>,
    ws: WsMode,
}

fn char_set_any() -> CharSet {
    CharSet {
        any: true,
        chars: BTreeSet::new(),
        ws: WsMode::Maybe,
    }
}

/// `CHAR_SET_NONE`.
fn char_set_none() -> CharSet {
    CharSet {
        any: false,
        chars: BTreeSet::new(),
        ws: WsMode::Never,
    }
}

/// `CHAR_SET_ANY_WITHOUT_SPACE` — satu-satunya kelas yang tidak akan pernah
/// overlap dengan `\s`.
fn char_set_any_without_space() -> CharSet {
    CharSet {
        any: true,
        chars: BTreeSet::new(),
        ws: WsMode::Never,
    }
}

/// `WHITESPACE_CHARS` (group-protection.js:561-565).
const WHITESPACE_CHARS: &[char] = &[
    ' ', '\t', '\n', '\r', '\u{c}', '\u{b}', '\u{a0}', '\u{1680}', '\u{2000}', '\u{2001}',
    '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}',
    '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
];

fn char_set_of(chars: impl Iterator<Item = char>) -> CharSet {
    let set: BTreeSet<char> = chars.collect();
    let ws = ws_mode_of(&set);
    CharSet {
        any: false,
        chars: set,
        ws,
    }
}

fn ws_mode_of(chars: &BTreeSet<char>) -> WsMode {
    let mut has_space = false;
    let mut has_other = false;
    for c in chars {
        if WHITESPACE_CHARS.contains(c) {
            has_space = true;
        } else {
            has_other = true;
        }
    }
    if !has_space {
        return WsMode::Never;
    }
    if !has_other {
        return WsMode::Only;
    }
    WsMode::Maybe
}

/// `literalSet` — flag `i` aktif: satu literal bisa cocok huruf besar/kecilnya.
fn literal_set(c: char) -> CharSet {
    let lower = c.to_lowercase().next().unwrap_or(c);
    let upper = c.to_uppercase().next().unwrap_or(c);
    if lower == upper {
        char_set_of(std::iter::once(c))
    } else {
        char_set_of([lower, upper].into_iter())
    }
}

/// `SHORTHAND_SETS` (group-protection.js:629-634).
fn shorthand_set(c: char) -> CharSet {
    match c {
        'd' => char_set_of("0123456789".chars()),
        'w' => {
            char_set_of("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_".chars())
        }
        's' => char_set_of(WHITESPACE_CHARS.iter().copied()),
        'S' => char_set_any_without_space(),
        _ => unreachable!("bukan shorthand"),
    }
}

/// `setUnion` (group-protection.js:595-604).
fn set_union(target: &mut CharSet, other: &CharSet) {
    if other.any {
        target.any = true;
        target.ws = WsMode::Maybe;
        return;
    }
    target.chars.extend(other.chars.iter().copied());
    target.ws = ws_mode_of(&target.chars);
}

/// `setsOverlap` (group-protection.js:607-623).
fn sets_overlap(a: &CharSet, b: &CharSet) -> bool {
    if a.any && b.any {
        return true;
    }
    if a.any {
        return if a.ws == WsMode::Maybe {
            true
        } else if a.ws == WsMode::Only {
            b.ws != WsMode::Never
        } else {
            b.ws != WsMode::Only
        };
    }
    if b.any {
        return if b.ws == WsMode::Maybe {
            true
        } else if b.ws == WsMode::Only {
            a.ws != WsMode::Never
        } else {
            a.ws != WsMode::Only
        };
    }
    a.chars.iter().any(|c| b.chars.contains(c))
}

fn set_is_empty(set: &CharSet) -> bool {
    !set.any && set.chars.is_empty()
}

/// `readEscape` (group-protection.js:645-690).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeKind {
    Char,
    Backref,
    Assert,
}

struct EscapeRead {
    kind: EscapeKind,
    set: CharSet,
    next: usize,
    broken: bool,
}

fn read_escape(src: &[char], start: usize) -> EscapeRead {
    let Some(&c) = src.get(start + 1) else {
        // Backslash di akhir pola — set kosong + broken.
        return EscapeRead {
            kind: EscapeKind::Char,
            set: char_set_of(std::iter::empty()),
            next: start + 1,
            broken: true,
        };
    };
    if ('1'..='9').contains(&c) {
        return EscapeRead {
            kind: EscapeKind::Backref,
            set: char_set_any(),
            next: start + 2,
            broken: false,
        };
    }
    if matches!(c, 'd' | 'w' | 's' | 'S') {
        return EscapeRead {
            kind: EscapeKind::Char,
            set: shorthand_set(c),
            next: start + 2,
            broken: false,
        };
    }
    if matches!(c, 'D' | 'W' | 'p' | 'P') {
        // `\D`/`\W` masih bisa cocok dengan whitespace → tetap "maybe".
        return EscapeRead {
            kind: EscapeKind::Char,
            set: char_set_any(),
            next: start + 2,
            broken: false,
        };
    }
    if c == 'b' || c == 'B' {
        // Assertion, bukan karakter: `\w+\b` idiom aman yang harus lolos.
        return EscapeRead {
            kind: EscapeKind::Assert,
            set: char_set_none(),
            next: start + 2,
            broken: false,
        };
    }
    if c == 'x'
        && let Some(ch) = hex_at(src, start + 2, 2)
    {
        return EscapeRead {
            kind: EscapeKind::Char,
            set: literal_set(ch),
            next: start + 4,
            broken: false,
        };
    }
    if c == 'u' {
        if src.get(start + 2) == Some(&'{') {
            if let Some(close) = src[start + 3..]
                .iter()
                .position(|&x| x == '}')
                .map(|p| start + 3 + p)
            {
                let body: String = src[start + 3..close].iter().collect();
                if let Some(ch) = (!body.is_empty() && body.chars().all(|h| h.is_ascii_hexdigit()))
                    .then(|| u32::from_str_radix(&body, 16).ok())
                    .flatten()
                    .and_then(char::from_u32)
                {
                    return EscapeRead {
                        kind: EscapeKind::Char,
                        set: literal_set(ch),
                        next: close + 1,
                        broken: false,
                    };
                }
                // Kode di luar rentang → JS melempar (pola tak bisa diurai).
                return EscapeRead {
                    kind: EscapeKind::Char,
                    set: char_set_any(),
                    next: start + 2,
                    broken: true,
                };
            }
        } else if let Some(ch) = hex_at(src, start + 2, 4) {
            return EscapeRead {
                kind: EscapeKind::Char,
                set: literal_set(ch),
                next: start + 6,
                broken: false,
            };
        }
    }
    // `CONTROL_ESCAPES[char] ?? char`.
    let mapped = match c {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        'f' => '\u{c}',
        'v' => '\u{b}',
        '0' => '\0',
        other => other,
    };
    EscapeRead {
        kind: EscapeKind::Char,
        set: literal_set(mapped),
        next: start + 2,
        broken: false,
    }
}

/// Baca `len` digit hex di `src[start..start+len]`.
fn hex_at(src: &[char], start: usize, len: usize) -> Option<char> {
    let slice = src.get(start..start + len)?;
    if slice.len() < len || !slice.iter().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let text: String = slice.iter().collect();
    char::from_u32(u32::from_str_radix(&text, 16).ok()?)
}

struct ClassRead {
    set: CharSet,
    next: usize,
    broken: bool,
}

/// `readCharClass` (group-protection.js:692-767) — HANYA untuk analisis
/// keamanan (over-approximasi), bukan untuk matching.
fn read_char_class(src: &[char], start: usize) -> ClassRead {
    let mut index = start + 1;
    let mut negated = false;
    if src.get(index) == Some(&'^') {
        negated = true;
        index += 1;
    }
    let mut set = char_set_of(std::iter::empty());
    let mut first = true;
    let mut closed = false;
    while index < src.len() {
        let c = src[index];
        if c == ']' && !first {
            closed = true;
            index += 1;
            break;
        }
        first = false;
        if c == '\\' {
            let escape = read_escape(src, index);
            if escape.kind == EscapeKind::Backref {
                return ClassRead {
                    set: char_set_any(),
                    next: index + 2,
                    broken: true,
                };
            }
            set_union(&mut set, &escape.set);
            index = escape.next;
            continue;
        }
        // POSIX class gaya `[[:alpha:]]` tidak didukung RegExp JS → any.
        if c == '[' && src.get(index + 1) == Some(&':') {
            let close = src[index..]
                .iter()
                .position(|&x| x == ']')
                .map(|p| index + p);
            index = close.map_or(src.len(), |p| p + 1);
            set.any = true;
            continue;
        }
        if src.get(index + 1) == Some(&'-')
            && src.get(index + 2).is_some()
            && src.get(index + 2) != Some(&']')
        {
            let tail = src[index + 2];
            if tail == '\\' {
                set_union(&mut set, &literal_set(c));
                set_union(&mut set, &literal_set('-'));
                index += 2;
                continue;
            }
            let from = c as u32;
            let to = tail as u32;
            if to < from {
                set.any = true;
                index += 3;
                continue;
            }
            for code in from..=to {
                if let Some(ch) = char::from_u32(code) {
                    set.chars.insert(ch);
                }
            }
            index += 3;
            continue;
        }
        set.chars.insert(c);
        let lower = c.to_lowercase().next().unwrap_or(c);
        let upper = c.to_uppercase().next().unwrap_or(c);
        if lower != upper {
            set.chars.insert(lower);
            set.chars.insert(upper);
        }
        index += 1;
    }
    // `[abc` tanpa penutup: `new RegExp` akan throw → inert. Dianalisis
    // sebagai "apa saja" supaya tidak ada jalur aman yang lolos.
    if !closed {
        return ClassRead {
            set: char_set_any(),
            next: index,
            broken: true,
        };
    }
    ClassRead {
        set: if negated { char_set_any() } else { set },
        next: index,
        broken: false,
    }
}

struct QuantRead {
    min: u32,
    /// `None` = tak terbatas (`*`, `+`, `{n,}`).
    max: Option<u32>,
    next: usize,
}

/// `readQuantifier` (group-protection.js:769-786).
fn read_quantifier(src: &[char], start: usize) -> Option<QuantRead> {
    match src.get(start) {
        Some('*') => Some(QuantRead {
            min: 0,
            max: None,
            next: start + 1,
        }),
        Some('+') => Some(QuantRead {
            min: 1,
            max: None,
            next: start + 1,
        }),
        Some('?') => Some(QuantRead {
            min: 0,
            max: Some(1),
            next: start + 1,
        }),
        Some('{') => {
            let mut i = start + 1;
            let mut min_text = String::new();
            while let Some(c) = src.get(i) {
                if c.is_ascii_digit() {
                    min_text.push(*c);
                    i += 1;
                } else {
                    break;
                }
            }
            if min_text.is_empty() {
                return None;
            }
            let min: u32 = min_text.parse().ok()?;
            if src.get(i) == Some(&',') {
                i += 1;
                let mut max_text = String::new();
                while let Some(c) = src.get(i) {
                    if c.is_ascii_digit() {
                        max_text.push(*c);
                        i += 1;
                    } else {
                        break;
                    }
                }
                if src.get(i) != Some(&'}') {
                    return None;
                }
                i += 1;
                let max = if max_text.is_empty() {
                    None
                } else {
                    Some(max_text.parse().ok()?)
                };
                Some(QuantRead { min, max, next: i })
            } else if src.get(i) == Some(&'}') {
                Some(QuantRead {
                    min,
                    max: Some(min),
                    next: i + 1,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `readAtom` (group-protection.js:788-807).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtomKind {
    Char,
    Assert,
    Group,
    Broken,
}

struct AtomRead {
    kind: AtomKind,
    set: CharSet,
    next: usize,
    broken: bool,
    analysis: Option<Analysis>,
}

fn read_atom(src: &[char], start: usize) -> AtomRead {
    let Some(&c) = src.get(start) else {
        return AtomRead {
            kind: AtomKind::Broken,
            set: char_set_none(),
            next: start + 1,
            broken: true,
            analysis: None,
        };
    };
    match c {
        // JS: `{ ...readEscape(src, start), kind: "char" }` — kind Assert
        // (`\b`) DITIMPA jadi "char" dengan set kosong.
        '\\' => {
            let e = read_escape(src, start);
            AtomRead {
                kind: AtomKind::Char,
                set: e.set,
                next: e.next,
                broken: e.broken,
                analysis: None,
            }
        }
        '[' => {
            let parsed = read_char_class(src, start);
            AtomRead {
                kind: AtomKind::Char,
                set: parsed.set,
                next: parsed.next,
                broken: parsed.broken,
                analysis: None,
            }
        }
        '.' => AtomRead {
            kind: AtomKind::Char,
            set: char_set_any(),
            next: start + 1,
            broken: false,
            analysis: None,
        },
        '^' | '$' => AtomRead {
            kind: AtomKind::Assert,
            set: char_set_none(),
            next: start + 1,
            broken: false,
            analysis: None,
        },
        '(' => read_group_atom(src, start),
        ')' | '|' | ']' => AtomRead {
            kind: AtomKind::Broken,
            set: char_set_none(),
            next: start + 1,
            broken: true,
            analysis: None,
        },
        _ => AtomRead {
            kind: AtomKind::Char,
            set: literal_set(c),
            next: start + 1,
            broken: false,
            analysis: None,
        },
    }
}

/// `findGroupEnd` (group-protection.js:809-839) — indeks `)` pasangan.
fn find_group_end(src: &[char], start: usize) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut index = start;
    let mut in_class = false;
    while index < src.len() {
        let c = src[index];
        if in_class {
            if c == '\\' {
                index += 1;
            } else if c == ']' {
                in_class = false;
            }
            index += 1;
            continue;
        }
        match c {
            '\\' => {
                index += 2;
                continue;
            }
            '[' => in_class = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// `readGroupAtom` (group-protection.js:841-872).
fn read_group_atom(src: &[char], start: usize) -> AtomRead {
    let mut inner = start + 1;
    if src.get(inner) == Some(&'?') {
        if src.get(inner + 1) == Some(&'<')
            && src.get(inner + 2) != Some(&'=')
            && src.get(inner + 2) != Some(&'!')
        {
            // `(?<nama>...)` — lompati nama.
            let close = src[inner + 2..]
                .iter()
                .position(|&x| x == '>')
                .map(|p| inner + 2 + p);
            inner = close.map_or(inner + 1, |p| p + 1);
        } else {
            inner += 2;
        }
    }
    let Some(end) = find_group_end(src, start) else {
        return AtomRead {
            kind: AtomKind::Broken,
            set: char_set_any(),
            next: src.len(),
            broken: true,
            analysis: Some(empty_group_analysis()),
        };
    };
    let analysis = analyze_alternation(&src[inner..end]);
    AtomRead {
        kind: AtomKind::Group,
        set: analysis.first.clone(),
        next: end + 1,
        broken: analysis.broken,
        analysis: Some(analysis),
    }
}

/// `EMPTY_GROUP_ANALYSIS` (group-protection.js:874-884).
fn empty_group_analysis() -> Analysis {
    Analysis {
        first: char_set_any(),
        broken: true,
        reason: None,
        has_quantifier: false,
        can_match_empty: true,
        overlapping_branches: false,
        has_unbounded_quantifier: false,
        single_char: false,
        unbounded: 0,
    }
}

/// Panjang run literal berurutan dimulai di `start` — supaya `.*anjing`
/// (literal panjang setelah kuantifier) tidak dianggap ambigu (:886-912).
fn literal_run_length(src: &[char], start: usize) -> usize {
    if src[start] == '\\' {
        return 1;
    }
    let mut index = start;
    while index < src.len() {
        let c = src[index];
        if matches!(c, '[' | '(' | ')' | '|' | '.' | '^' | '$' | '*' | '+' | '?') {
            break;
        }
        if read_quantifier(src, index + 1).is_some() {
            break;
        }
        index += 1;
    }
    index - start
}

/// Item satu cabang — padanan objek `{ set, quantified, unbounded, expansive,
/// singleChar, canMatchEmpty }` (:968-987).
struct BranchItem {
    set: CharSet,
    quantified: bool,
    unbounded: bool,
    expansive: bool,
    single_char: bool,
    can_match_empty: bool,
}

/// State lintas cabang — `{ unbounded, broken }` JS (:1041).
struct AnalysisState {
    unbounded: usize,
    broken: bool,
}

struct BranchRead {
    items: Vec<BranchItem>,
    reason: Option<String>,
}

/// Alasan penolakan bersama — teksnya disamakan dengan JS supaya test
/// analyze bawaan bisa port apa adanya.
fn reject_overlap_stacked() -> String {
    "dua kuantifier bertumpuk menebak karakter yang sama (mis. `a*a*`)".into()
}

fn reject_overlap_swallow() -> String {
    "karakter setelah kuantifier bisa ikut ditelan kuantifier itu (mis. `.*a`)".into()
}

fn reject_nested_quantifier() -> String {
    "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)".into()
}

fn reject_empty_group() -> String {
    "isi kelompok yang diulang bisa kosong (mis. `(a*|b)+`)".into()
}

fn reject_overlapping_branches() -> String {
    "dua cabang alternasi di bawah kuantifier bisa cocok dengan teks yang sama (mis. `(a|ab)+`)"
        .into()
}

/// `analyzeBranch` (group-protection.js:918-1005).
fn analyze_branch(src: &[char], state: &mut AnalysisState) -> BranchRead {
    let mut items: Vec<BranchItem> = Vec::new();
    let mut reason: Option<String> = None;
    let mut index = 0usize;
    while index < src.len() {
        let c = src[index];
        if c == '|' || c == ')' {
            return BranchRead { items, reason };
        }
        let atom = read_atom(src, index);
        if atom.broken {
            state.broken = true;
        }
        let mut next = atom.next;
        let quantifier = read_quantifier(src, next);
        if let Some(q) = &quantifier {
            next = q.next;
        }
        if atom.kind == AtomKind::Assert {
            index = next;
            continue;
        }
        let quantified = quantifier.is_some();
        // Catat: JS menghitung `max === 1` sebagai "unbounded" juga —
        // quirk yang dibiarkan apa adanya.
        let unbounded = matches!(&quantifier, Some(q) if q.max.is_none() || q.max == Some(1));
        let run_length = if atom.kind == AtomKind::Char
            && !quantified
            && src[index] != '\\'
            && src[index] != '['
        {
            literal_run_length(src, index)
        } else {
            0
        };
        let is_group = atom.kind == AtomKind::Group;
        let analysis = atom.analysis.clone().unwrap_or_else(empty_group_analysis);
        if is_group {
            state.unbounded += analysis.unbounded;
            if quantified {
                // Kelompok yang diulang + isi yang bisa diulang/dikosongkan =
                // backtracking eksponensial.
                if analysis.has_quantifier && reason.is_none() {
                    reason = Some(reject_nested_quantifier());
                }
                if analysis.can_match_empty && reason.is_none() {
                    reason = Some(reject_empty_group());
                }
                if analysis.overlapping_branches && reason.is_none() {
                    reason = Some(reject_overlapping_branches());
                }
            }
        }
        let expansive = unbounded || (is_group && analysis.has_unbounded_quantifier);
        let single_char = if is_group {
            analysis.single_char
        } else if run_length > 0 {
            run_length <= 1
        } else {
            true
        };
        let can_match_empty = if quantified {
            quantifier.as_ref().map(|q| q.min == 0).unwrap_or(false)
                || (is_group && analysis.can_match_empty)
        } else if is_group {
            analysis.can_match_empty
        } else {
            set_is_empty(&atom.set)
        };
        let item = BranchItem {
            set: atom.set.clone(),
            quantified,
            unbounded,
            expansive,
            single_char,
            can_match_empty,
        };
        index = if run_length > 0 {
            index + run_length
        } else {
            next
        };
        if let Some(previous) = items.last() {
            if previous.expansive && sets_overlap(&previous.set, &item.set) {
                if item.quantified {
                    if reason.is_none() {
                        reason = Some(reject_overlap_stacked());
                    }
                } else if item.expansive || item.single_char {
                    if reason.is_none() {
                        reason = Some(reject_overlap_swallow());
                    }
                }
            }
        }
        if unbounded {
            state.unbounded += 1;
        }
        items.push(item);
    }
    BranchRead { items, reason }
}

/// `splitAlternation` (group-protection.js:1007-1038) — potong `|` level
/// teratas, menghormati escape/kelas/kurung.
fn split_alternation(src: &[char]) -> Vec<&[char]> {
    let mut branches = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let mut in_class = false;
    let mut index = 0usize;
    while index < src.len() {
        let c = src[index];
        if in_class {
            if c == '\\' {
                index += 1;
            } else if c == ']' {
                in_class = false;
            }
            index += 1;
            continue;
        }
        match c {
            '\\' => {
                index += 2;
                continue;
            }
            '[' => in_class = true,
            '(' => depth += 1,
            ')' => depth -= 1,
            '|' if depth == 0 => {
                branches.push(&src[start..index]);
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    branches.push(&src[start..]);
    branches
}

/// Hasil `analyzeAlternation` — padanan objek JS (:1079-1089).
#[derive(Clone)]
struct Analysis {
    first: CharSet,
    broken: bool,
    reason: Option<String>,
    has_quantifier: bool,
    can_match_empty: bool,
    overlapping_branches: bool,
    has_unbounded_quantifier: bool,
    single_char: bool,
    unbounded: usize,
}

/// `analyzeAlternation` (group-protection.js:1040-1090).
fn analyze_alternation(src: &[char]) -> Analysis {
    let mut state = AnalysisState {
        unbounded: 0,
        broken: false,
    };
    let branches = split_alternation(src);
    let mut first = char_set_of(std::iter::empty());
    let mut all_single_char = true;
    let mut can_match_empty = false;
    let mut has_quantifier = false;
    let mut has_unbounded_quantifier = false;
    let mut overlapping_branches = false;
    let mut reason: Option<String> = None;
    let mut previous_first: Option<CharSet> = None;
    for branch in &branches {
        let result = analyze_branch(branch, &mut state);
        if result.reason.is_some() && reason.is_none() {
            reason = result.reason;
        }
        if state.broken && reason.is_none() {
            reason = Some("pola tidak bisa diurai (kurung kurawal tidak seimbang)".into());
        }
        if result.items.is_empty() {
            can_match_empty = true;
            all_single_char = false;
            continue;
        }
        let head = &result.items[0];
        if head.quantified {
            all_single_char = false;
            has_quantifier = true;
        }
        if !head.single_char {
            all_single_char = false;
        }
        if head.can_match_empty {
            can_match_empty = true;
        }
        if result.items.iter().any(|i| i.quantified) {
            has_quantifier = true;
        }
        if result.items.iter().any(|i| i.unbounded) {
            has_unbounded_quantifier = true;
        }
        if result.items.iter().all(|i| i.can_match_empty) {
            can_match_empty = true;
        }
        if let Some(pf) = &previous_first {
            if sets_overlap(pf, &head.set) {
                overlapping_branches = true;
            }
        }
        set_union(&mut first, &head.set);
        previous_first = Some(head.set.clone());
    }
    Analysis {
        first,
        broken: state.broken,
        reason,
        has_quantifier,
        can_match_empty,
        overlapping_branches,
        has_unbounded_quantifier,
        single_char: all_single_char,
        unbounded: state.unbounded,
    }
}

/// `analyzeCustomPattern` (group-protection.js:1102-1138) — gerbang
/// install-time pola buatan admin. `Ok(())` = aman; `Err(alasan)` = pola
/// ditolak dengan alasan yang bisa dibaca admin.
pub fn analyze_custom_pattern(pattern: &str) -> Result<(), String> {
    if pattern.trim().is_empty() {
        return Err("pola kosong".to_string());
    }
    let source: Vec<char> = pattern.chars().collect();
    // `source.length` JS memakai code unit UTF-16 (:1106), bukan scalar Rust.
    if pattern.encode_utf16().count() > CUSTOM_MAX_PATTERN_LEN {
        return Err(format!(
            "pola terlalu panjang (maks {CUSTOM_MAX_PATTERN_LEN} karakter)"
        ));
    }
    let analysis = analyze_alternation(&source);
    if analysis.broken {
        return Err("pola tidak bisa diurai (kurung kurawal tidak seimbang)".to_string());
    }
    if let Some(r) = analysis.reason {
        return Err(r);
    }
    if analysis.unbounded > CUSTOM_MAX_UNBOUNDED_QUANTIFIERS {
        return Err(format!(
            "terlalu banyak kuantifier tak terbatas (maks {CUSTOM_MAX_UNBOUNDED_QUANTIFIERS})"
        ));
    }
    // `new RegExp(source, "i")` — validasi sintaks; mesin mini di sini lebih
    // ketat dari JS (menolak lookahead/backreference) → fail-closed.
    if MiniRegex::new(pattern, "i").is_err() {
        return Err("regex tidak valid".to_string());
    }
    Ok(())
}

/// `sanitizeCustomFlags` (group-protection.js:1141-1149) — buang `g`/`y`
/// (membuat `.test()` stateful); hasil kosong → `"i"`.
fn sanitize_custom_flags(flags: Option<&str>) -> String {
    let mut keep: Vec<char> = Vec::new();
    for f in flags.unwrap_or("i").chars() {
        if matches!(f, 'i' | 'm' | 's' | 'u' | 'v') && !keep.contains(&f) {
            keep.push(f);
        }
    }
    // `new RegExp(pattern, "uv")` throw di JS (:1167) — dua mode unicode
    // tidak boleh dipakai bersamaan.
    if keep.contains(&'u') && keep.contains(&'v') {
        return "invalid".to_string();
    }
    if keep.is_empty() {
        "i".to_string()
    } else {
        keep.into_iter().collect()
    }
}

/// Regex rule custom yang boleh jalan — `getCustomRegex`
/// (group-protection.js:1157-1175): pola tidak aman TETAP inert (cache `null`
/// di JS), dan kompilasi di-cache supaya analisis tidak diulang tiap pesan.
/// Penyederhanaan eviksi: JS `evictOldestOverCap` (tertua keluar); di sini
/// seluruh cache dibersihkan saat penuh (cap sama, 200).
fn custom_regex(rule: &CustomRule) -> Option<MiniRegex> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<MiniRegex>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let flags = sanitize_custom_flags(rule.flags.as_deref());
    if flags == "invalid" {
        return None;
    }
    let key = format!("{flags}|{}", rule.pattern);
    let mut map = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(cached) = map.get(&key) {
        return cached.clone();
    }
    let compiled = if analyze_custom_pattern(&rule.pattern).is_ok() {
        MiniRegex::new(&rule.pattern, &flags).ok()
    } else {
        None
    };
    if map.len() >= CUSTOM_REGEX_CACHE_CAP {
        map.clear();
    }
    map.insert(key, compiled.clone());
    compiled
}

/// `matchCustomRule` (group-protection.js:1183-1207) — cocokkan pesan dengan
/// rule AntiCustom milik grup. Batas kerja per pesan: subjek dipotong ke
/// 512 char, maksimal 50 rule, budget 20 ms total.
pub fn match_custom_rule<'r>(text: &str, rules: &'r [CustomRule]) -> Option<&'r CustomRule> {
    if text.is_empty() || rules.is_empty() {
        return None;
    }
    // `String(text).slice(0, CUSTOM_MAX_SUBJECT_LEN)` — JS memotong code unit
    // UTF-16 (:1186), bukan scalar Rust.
    let mut utf16_len = 0usize;
    let subject: String = text
        .chars()
        .take_while(|ch| {
            let next = ch.len_utf16();
            if utf16_len + next > CUSTOM_MAX_SUBJECT_LEN {
                false
            } else {
                utf16_len += next;
                true
            }
        })
        .collect();
    let lower = subject.to_lowercase();
    let deadline = Instant::now() + Duration::from_millis(CUSTOM_MATCH_BUDGET_MS);
    let limit = rules.len().min(CUSTOM_MAX_RULES);
    for (index, rule) in rules.iter().enumerate().take(limit) {
        if index > 0 && Instant::now() > deadline {
            break;
        }
        if rule.pattern.is_empty() {
            continue;
        }
        if rule.rule_type == "regex" {
            if let Some(re) = custom_regex(rule)
                && re.test(&subject)
            {
                return Some(rule);
            }
            continue;
        }
        if lower.contains(&rule.pattern.to_lowercase()) {
            return Some(rule);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT: &str = "628999@s.whatsapp.net";

    fn admin_participant(num: &str) -> Participant {
        Participant {
            id: format!("{num}@s.whatsapp.net"),
            admin: true,
            ..Default::default()
        }
    }

    fn member_participant(num: &str) -> Participant {
        Participant {
            id: format!("{num}@s.whatsapp.net"),
            ..Default::default()
        }
    }

    fn caller() -> GroupCaller {
        GroupCaller {
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            is_group: true,
            // `m.isBotAdmin` — flag terpisah dari `isBotAdminCheck(participants)`
            // yang dipakai gate antilink. `env()` di bawah Record bot sebagai
            // admin, jadi default-nya `true` supaya konsisten.
            is_bot_admin: true,
            ..Default::default()
        }
    }

    /// Env standar: bot adalah admin, sender bukan admin.
    fn env() -> GateEnv {
        GateEnv {
            bot_number: BOT.into(),
            // Metadata tersedia — roster-nya sudah ada, jadi gate boleh cek
            // admin. Metadata yang GAGAL diuji terpisah.
            metadata_available: true,
            participants: vec![
                admin_participant("628999"),
                admin_participant("628222"),
                member_participant("628111"),
                member_participant("628333"),
            ],
            now_ms: 100_000,
            ..Default::default()
        }
    }

    fn input<'a>(c: &'a GroupCaller, s: &'a GroupSettings, e: &'a GateEnv) -> GateInput<'a> {
        GateInput {
            caller: c,
            settings: s,
            env: e,
            last_message_ms: 0,
        }
    }

    /// Jalankan satu gate langsung, tanpa framework.
    fn run_one(gate: GateFn, c: &GroupCaller, s: &GroupSettings, e: &GateEnv) -> GateOutcome {
        gate(&input(c, s, e))
    }

    // ==================================================================
    // Framework
    // ==================================================================

    #[test]
    fn non_group_message_skips_whole_chain() {
        let mut c = caller();
        c.is_group = false;
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        let e = env();
        assert_eq!(
            default_gate_chain().run(&input(&c, &s, &e)),
            Decision::Allow
        );
    }

    #[test]
    fn newsletter_skips_whole_chain() {
        let mut c = caller();
        c.is_newsletter = true;
        c.body = "cek https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        let e = env();
        assert_eq!(
            default_gate_chain().run(&input(&c, &s, &e)),
            Decision::Allow
        );
    }

    #[test]
    fn nothing_configured_allows_everything() {
        let mut c = caller();
        c.body = "halo cek https://x.com sama wa.me/12345".into();
        let s = GroupSettings::default();
        let e = env();
        assert_eq!(
            default_gate_chain().run(&input(&c, &s, &e)),
            Decision::Allow
        );
    }

    #[test]
    fn default_chain_order_matches_handler_js() {
        assert_eq!(
            default_gate_chain().gate_names(),
            vec![
                "antijudol",    // handler.js:773
                "antiphising",  // :776
                "anticustom",   // :779
                "antilink",     // :782
                "antilinkgc",   // :785
                "antilinkall",  // :788
                "antihidetag",  // :791
                "antiswgc",     // :794
                "antidocument", // :797
                "mute",         // :809
                "mutedmember",  // :818
                "mutegc",       // :827
                "slowmode",     // :844
            ]
        );
    }

    // ---- ORDER: dua gate yang sama-sama match → yang DULU menang ----

    #[test]
    fn order_first_gate_wins_antilink_beats_hidetag() {
        // Satu pesan berisi link DAN hidetag semua member: antilink (:782)
        // dijalankan sebelum hidetag (:791), jadi yang tampil pesan antilink.
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.mentioned_jid = vec!["1@s.whatsapp.net".into(); 4];
        c.participant_count = 4;
        let s = GroupSettings {
            antilink: Some("on".into()),
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        let e = env();
        let d = default_gate_chain().run(&input(&c, &s, &e));
        assert_eq!(
            d,
            Decision::DeleteAndReply(ANTILINK.replace("%user%", "628111"))
        );
        // Bukan pesan hidetag.
        assert!(!d.reply_text().unwrap().contains("AntiHidetag"));
    }

    #[test]
    fn order_hidetag_wins_when_antilink_does_not_match() {
        // Hidetag saja: antilink lolos (toggle on tapi tidak ada link), jadi
        // gate berikutnya yang	match.
        let mut c = caller();
        c.body = "halo semua".into();
        c.mentioned_jid = vec!["1@s.whatsapp.net".into(); 4];
        c.participant_count = 4;
        let s = GroupSettings {
            antilink: Some("on".into()),
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        let e = env();
        let d = default_gate_chain().run(&input(&c, &s, &e));
        assert_eq!(
            d,
            Decision::DeleteAndReply(ANTIHIDETAG.replace("%user%", "628111"))
        );
    }

    #[test]
    fn order_mute_wins_over_slowmode() {
        // mute (:809) di sebelum slowmode (:844); keduanya match.
        let c = caller();
        let s = GroupSettings {
            mute: true,
            slowmode: Some(Slowmode {
                enabled: true,
                delay_secs: Some(30),
                mode: Some("all".into()),
            }),
            ..Default::default()
        };
        let e = env();
        // slowmode butuh last_message_ms baru; default 0 = sudah lama → lolos.
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(
            default_gate_chain().run(&i),
            Decision::DeleteAndStop,
            "gate mute lebih dulu, slowmode tidak boleh ikut dievaluasi"
        );
    }

    #[test]
    fn order_slowmode_wins_when_mute_off() {
        let c = caller();
        let s = GroupSettings {
            slowmode: Some(Slowmode {
                enabled: true,
                delay_secs: Some(30),
                mode: Some("all".into()),
            }),
            ..Default::default()
        };
        let e = env();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(default_gate_chain().run(&i), Decision::DeleteAndStop);
    }

    #[test]
    fn order_antidocument_wins_over_mute() {
        // antidocument (:797) reply, mute (:809) stop senyap. Yang lebih dulu
        // menentukan apa yang dilihat user: balasan.
        let mut c = caller();
        c.is_document = true;
        let s = GroupSettings {
            antidocument: true,
            mute: true,
            ..Default::default()
        };
        let e = env();
        let d = default_gate_chain().run(&input(&c, &s, &e));
        assert!(d.reply_text().unwrap().contains("AntiDocument"));
    }

    #[test]
    fn order_mutegc_wins_over_slowmode() {
        let c = caller();
        let s = GroupSettings {
            mutegc: true,
            slowmode: Some(Slowmode {
                enabled: true,
                delay_secs: Some(30),
                mode: Some("all".into()),
            }),
            ..Default::default()
        };
        let e = env();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(
            default_gate_chain().run(&i),
            Decision::Stop,
            "mutegc (:827) tidak delete, slowmode (:844) akan delete"
        );
    }

    #[test]
    fn order_muted_member_wins_over_mutegc() {
        let c = caller();
        let s = GroupSettings {
            muted_members: vec!["628111@s.whatsapp.net".into()],
            mutegc: true,
            ..Default::default()
        };
        let e = env();
        assert_eq!(
            default_gate_chain().run(&input(&c, &s, &e)),
            Decision::DeleteAndStop
        );
    }

    #[test]
    fn order_antilink_gc_wins_over_antilink_all() {
        // ": wa.me/12345xyz" kena KEDUA gate. Gc (:785) lebih dulu.
        let mut c = caller();
        c.body = "wa.me/12345".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            antilinkall: Some("on".into()),
            ..Default::default()
        };
        let e = env();
        let d = default_gate_chain().run(&input(&c, &s, &e));
        assert!(d.reply_text().unwrap().contains("Antilink WA"));
    }

    #[test]
    fn custom_chain_order_is_honoured() {
        // Framework, bukan urutan default, yang menentukan.
        let mut c = caller();
        c.is_document = true;
        let s = GroupSettings {
            antidocument: true,
            mute: true,
            ..Default::default()
        };
        let e = env();
        let reversed = GroupGuard::new(vec![
            Gate::new("mute", gate_mute),
            Gate::new("antidocument", gate_antidocument),
        ]);
        assert_eq!(
            reversed.run(&input(&c, &s, &e)),
            Decision::DeleteAndStop,
            "mute diletakkan duluan → mute yang menang"
        );
    }

    // ==================================================================
    // gate_antilink  (group-protection.js:1311)
    // ==================================================================

    #[test]
    fn antilink_off_lets_message_through() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("off".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Pass,
            "`!== \"on\"` — nilai lain termasuk None tidak menyalakan"
        );
    }

    #[test]
    fn antilink_on_without_link_passes() {
        let mut c = caller();
        c.body = "halo dunia".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_deletes_and_replies() {
        let mut c = caller();
        c.body = "spesIAL cek https://spam.com sekarang".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTILINK.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn antilink_kick_mode_returns_kick_decision() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            antilink_mode: Some("kick".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Decide(Decision::KickAndReply(
                ANTILINK_KICK.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn antilink_non_kick_mode_value_is_remove() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            antilink_mode: Some("nonsense".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTILINK.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn antilink_admin_is_exempt() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.sender = "628222@s.whatsapp.net".into(); // admin di participants
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_exemption_uses_participant_not_sender() {
        // Sender bukan admin, tapi `m.key.participant` adalah admin.
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.participant = Some("628222@s.whatsapp.net".into());
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_owner_without_admin_record_is_not_exempt() {
        // Gate ini cek admin dari participants, BUKAN m.isOwner. Owner yang
        // tidak tercatat admin tetap kena.
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.is_owner = true;
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn antilink_partner_is_not_exempt() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.is_partner = true;
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn antilink_replies_notadmin_when_bot_is_not_admin() {
        // Asimetri penting: gate ini REPLY, Gc/All senyap.
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(
            run_one(gate_antilink, &c, &s, &e),
            GateOutcome::Decide(Decision::Reply(NOT_ADMIN.into()))
        );
    }

    #[test]
    fn antilink_skips_bots_own_message_even_in_self_mode() {
        // handleAntilink TIDAK mengecek selfMode (:1327-1328) — berbeda dari
        // Gc/All. Kalau di-port dengan cek selfMode, bot di selfMode akan
        // mulai merevoke pesan outputnya sendiri.
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.sender = BOT.into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.self_mode = true;
        assert_eq!(run_one(gate_antilink, &c, &s, &e), GateOutcome::Pass);
    }

    #[test]
    fn antilink_matches_bot_via_device_suffix() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.sender = "628999@s.whatsapp.net".into();
        c.participant = Some("628999:12@s.whatsapp.net".into());
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_bot_lid_also_counts_for_bot_admin_check() {
        // Grup addressing LID: bot tercatat admin cuma lewat field `lid`.
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        e.participants.push(Participant {
            lid: "111222333@s.lid".into(),
            admin: true,
            ..Default::default()
        });
        e.bot_lid = Some("111222333@s.lid".into());
        assert!(matches!(
            run_one(gate_antilink, &c, &s, &e),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    // ==================================================================
    // gate_antilink_gc  (group-protection.js:1888)
    // ==================================================================

    #[test]
    fn antilink_gc_triggers_on_whatsapp_link() {
        let mut c = caller();
        c.body = "cek https://chat.whatsapp.com/AbCd1234".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_gc, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTILINK_GC.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn antilink_gc_ignores_non_whatsapp_link() {
        let mut c = caller();
        c.body = "cek https://contoh.com".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink_gc, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_gc_ignores_bare_prefix_without_payload() {
        let mut c = caller();
        c.body = "chat.whatsapp.com/".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink_gc, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_gc_matches_api_send_without_payload() {
        let mut c = caller();
        c.body = "api.whatsapp.com/send".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_antilink_gc, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn antilink_gc_kick_mode() {
        let mut c = caller();
        c.body = "wa.me/62811".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            antilinkgc_mode: Some("kick".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_gc, &c, &s, &env()),
            GateOutcome::Decide(Decision::KickAndReplyFallback {
                kick: ANTILINK_GC_KICK.replace("%user%", "628111"),
                fallback: ANTILINK_GC.replace("%user%", "628111"),
            })
        );
    }

    #[test]
    fn antilink_gc_admin_exempt() {
        let mut c = caller();
        c.body = "wa.me/12345".into();
        c.sender = "628222@s.whatsapp.net".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink_gc, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn antilink_gc_silent_when_bot_not_admin() {
        // :1909 `if (!isBotAdminCheck(...)) return false;` — SENYAP. Kalau
        // diubah jadi reply, user tetap lihat pesan bot.
        let mut c = caller();
        c.body = "wa.me/12345".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(
            run_one(gate_antilink_gc, &c, &s, &e),
            GateOutcome::Pass,
            "harus senyap, bukan reply notAdmin"
        );
    }

    #[test]
    fn antilink_gc_never_acts_on_bots_own_message() {
        // Cek ulang dengan rekonstruksi JS di node: cabang `selfMode`
        // (:1900) TIDAK mengubah hasil. Kalau bot admin → cek admin (:1908)
        //-exempt; kalau bot bukan admin → cek bot-admin gagal dan gate
        // senyap (:1909). Dua-duanya `return false`.
        //
        // Diuji sebagai invarian, bukan sebagai perbedaan selfMode, karena
        // hasilnya tidak terlihat dari `Decision` — tapi CABUTNYA cek
        // selfMode akan salah kalau suatu saat urutan cek berubah.
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        let mut c = caller();
        c.body = "wa.me/12345".into();
        c.sender = BOT.into();

        for self_mode in [false, true] {
            let mut e = env();
            e.self_mode = self_mode;
            assert_eq!(
                run_one(gate_antilink_gc, &c, &s, &e),
                GateOutcome::Pass,
                "self_mode={self_mode}: pesan bot sendiri tidak pernah kena"
            );
        }
    }

    #[test]
    fn antilink_gc_self_sender_skip_is_load_bearing_in_antilink() {
        // Kebalikan dari test di atas: `handleAntilink` TIDAK punya cek
        // selfMode sama sekali. Kalau cek self-sender-nya dihapus, pesan bot
        // sendiri dengan bot non-admin akan jatuh ke cabang "reply notAdmin"
        // — bottegur dirinya sendiri. Test ini mengunci perbedaan itu.
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        let mut c = caller();
        c.body = "https://spam.com".into();
        c.sender = BOT.into();
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));

        assert_eq!(
            run_one(gate_antilink, &c, &s, &e),
            GateOutcome::Pass,
            "tanpa cek self-sender, hasilnya akan Reply(notAdmin)"
        );
        // Bandingkan: gate GC SENYAP di kondisi yang sama.
        let s_gc = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_antilink_gc, &c, &s_gc, &e), GateOutcome::Pass);
    }

    // ==================================================================
    // gate_antilink_all  (group-protection.js:1943)
    // ==================================================================

    #[test]
    fn antilink_all_triggers_on_any_tld_link() {
        let mut c = caller();
        c.body = "cek www.contoh.co.id/page".into();
        let s = GroupSettings {
            antilinkall: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_all, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTILINK_ALL.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn antilink_all_ignores_unknown_tld() {
        let mut c = caller();
        c.body = "cek file.inventi".into();
        let s = GroupSettings {
            antilinkall: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_all, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn antilink_all_kick_mode() {
        let mut c = caller();
        c.body = "x.dev".into();
        let s = GroupSettings {
            antilinkall: Some("on".into()),
            antilinkall_mode: Some("kick".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_all, &c, &s, &env()),
            GateOutcome::Decide(Decision::KickAndReplyFallback {
                kick: ANTILINK_ALL_KICK.replace("%user%", "628111"),
                fallback: ANTILINK_ALL.replace("%user%", "628111"),
            })
        );
    }

    #[test]
    fn antilink_all_admin_exempt() {
        let mut c = caller();
        c.body = "x.dev".into();
        c.sender = "628222@s.whatsapp.net".into();
        let s = GroupSettings {
            antilinkall: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_all, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn antilink_all_silent_when_bot_not_admin() {
        let mut c = caller();
        c.body = "x.dev".into();
        let s = GroupSettings {
            antilinkall: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(run_one(gate_antilink_all, &c, &s, &e), GateOutcome::Pass);
    }

    // ==================================================================
    // gate_hidetag  (group-protection.js:2145)
    // ==================================================================

    #[test]
    fn hidetag_full_mention_is_removed() {
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 4];
        c.participant_count = 4;
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_hidetag, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTIHIDETAG.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn hidetag_partial_mention_passes() {
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 3];
        c.participant_count = 4;
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_hidetag, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn hidetag_no_mention_passes() {
        let c = caller();
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_hidetag, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn hidetag_off_passes() {
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 4];
        c.participant_count = 4;
        let s = GroupSettings {
            antihidetag: Some("off".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_hidetag, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn hidetag_admin_owner_fromme_all_exempt() {
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        for flag in ["is_admin", "is_owner", "from_me"] {
            let mut c = caller();
            c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 4];
            c.participant_count = 4;
            match flag {
                "is_admin" => c.is_admin = true,
                "is_owner" => c.is_owner = true,
                _ => c.from_me = true,
            }
            assert_eq!(
                run_one(gate_hidetag, &c, &s, &env()),
                GateOutcome::Pass,
                "{flag} harus exempt"
            );
        }
    }

    #[test]
    fn hidetag_partner_is_not_exempt() {
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 4];
        c.participant_count = 4;
        c.is_partner = true;
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_hidetag, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn hidetag_silent_when_bot_not_admin() {
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 4];
        c.participant_count = 4;
        c.is_bot_admin = false;
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        let e = GateEnv {
            bot_number: BOT.into(),
            metadata_available: true,
            participants: vec![member_participant("628111")],
            ..Default::default()
        };
        assert_eq!(run_one(gate_hidetag, &c, &s, &e), GateOutcome::Pass);
    }

    #[test]
    fn hidetag_more_mentions_than_participants_still_triggers() {
        // `<` bukan `<=` — mention melebihi jumlah participant tetap kena.
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into(); 10];
        c.participant_count = 2;
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_hidetag, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    // ==================================================================
    // gate_antidocument  (plugins/group/antidocument.js:32)
    // ==================================================================

    #[test]
    fn antidocument_deletes_document_message() {
        let mut c = caller();
        c.is_document = true;
        let s = GroupSettings {
            antidocument: true,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antidocument, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTIDOCUMENT.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn antidocument_detects_document_message_type() {
        for t in ["documentMessage", "documentWithCaptionMessage"] {
            let mut c = caller();
            c.message_type = t.into();
            let s = GroupSettings {
                antidocument: true,
                ..Default::default()
            };
            assert!(
                matches!(
                    run_one(gate_antidocument, &c, &s, &env()),
                    GateOutcome::Decide(Decision::DeleteAndReply(_))
                ),
                "{t} harus terdeteksi"
            );
        }
    }

    #[test]
    fn antidocument_ignores_other_types() {
        let mut c = caller();
        c.message_type = "imageMessage".into();
        let s = GroupSettings {
            antidocument: true,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antidocument, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn antidocument_off_passes() {
        let mut c = caller();
        c.is_document = true;
        let s = GroupSettings {
            antidocument: false,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antidocument, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn antidocument_admin_owner_fromme_exempt() {
        // antidocument.js:34 — exemption ADA, dan dicek sebelum toggle.
        let s = GroupSettings {
            antidocument: true,
            ..Default::default()
        };
        for flag in ["is_admin", "is_owner", "from_me"] {
            let mut c = caller();
            c.is_document = true;
            match flag {
                "is_admin" => c.is_admin = true,
                "is_owner" => c.is_owner = true,
                _ => c.from_me = true,
            }
            assert_eq!(
                run_one(gate_antidocument, &c, &s, &env()),
                GateOutcome::Pass,
                "{flag} harus exempt"
            );
        }
    }

    #[test]
    fn antidocument_partner_is_not_exempt() {
        let mut c = caller();
        c.is_document = true;
        c.is_partner = true;
        let s = GroupSettings {
            antidocument: true,
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_antidocument, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    // ==================================================================
    // gate_mute  (handler.js:809 + mute.js:31)
    // ==================================================================

    #[test]
    fn mute_stops_and_deletes_when_bot_admin() {
        let s = GroupSettings {
            mute: true,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_mute, &caller(), &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    #[test]
    fn mute_stops_silently_when_bot_not_admin() {
        let s = GroupSettings {
            mute: true,
            ..Default::default()
        };
        let mut c = caller();
        c.is_bot_admin = false;
        let e = GateEnv {
            bot_number: BOT.into(),
            participants: vec![member_participant("628111")],
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_mute, &c, &s, &e),
            GateOutcome::Decide(Decision::Stop),
            "tanpa hak delete, JS tetap return tapi tidak hapus"
        );
    }

    #[test]
    fn mute_off_passes() {
        let s = GroupSettings {
            mute: false,
            ..Default::default()
        };
        assert_eq!(run_one(gate_mute, &caller(), &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn mute_admin_and_owner_exempt() {
        let s = GroupSettings {
            mute: true,
            ..Default::default()
        };
        for flag in ["is_admin", "is_owner"] {
            let mut c = caller();
            if flag == "is_admin" {
                c.is_admin = true;
            } else {
                c.is_owner = true;
            }
            assert_eq!(
                run_one(gate_mute, &c, &s, &env()),
                GateOutcome::Pass,
                "{flag} harus exempt"
            );
        }
    }

    #[test]
    fn mute_partner_is_not_exempt() {
        let mut c = caller();
        c.is_partner = true;
        let s = GroupSettings {
            mute: true,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_mute, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    // ==================================================================
    // gate_muted_member  (handler.js:818 + mutemember.js:102)
    // ==================================================================

    #[test]
    fn muted_member_blocked_and_deleted() {
        let s = GroupSettings {
            muted_members: vec!["628111@s.whatsapp.net".into()],
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_muted_member, &caller(), &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    #[test]
    fn muted_member_matches_by_suffix() {
        let s = GroupSettings {
            muted_members: vec!["628111@s.whatsapp.net".into()],
            ..Default::default()
        };
        let mut c = caller();
        c.sender = "120628111@s.whatsapp.net".into();
        assert!(matches!(
            run_one(gate_muted_member, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndStop)
        ));
    }

    #[test]
    fn muted_member_other_user_passes() {
        let s = GroupSettings {
            muted_members: vec!["628999@s.whatsapp.net".into()],
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_muted_member, &caller(), &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn muted_member_empty_list_passes() {
        let s = GroupSettings::default();
        assert_eq!(
            run_one(gate_muted_member, &caller(), &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn muted_member_admin_owner_exempt() {
        let s = GroupSettings {
            muted_members: vec!["628111@s.whatsapp.net".into()],
            ..Default::default()
        };
        let mut c = caller();
        c.is_owner = true;
        assert_eq!(
            run_one(gate_muted_member, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn muted_member_partner_is_not_exempt() {
        let s = GroupSettings {
            muted_members: vec!["628111@s.whatsapp.net".into()],
            ..Default::default()
        };
        let mut c = caller();
        c.is_partner = true;
        assert_eq!(
            run_one(gate_muted_member, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    #[test]
    fn muted_member_silent_when_bot_not_admin() {
        let s = GroupSettings {
            muted_members: vec!["628111@s.whatsapp.net".into()],
            ..Default::default()
        };
        let mut c = caller();
        c.is_bot_admin = false;
        let e = GateEnv {
            bot_number: BOT.into(),
            participants: vec![member_participant("628111")],
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_muted_member, &c, &s, &e),
            GateOutcome::Decide(Decision::Stop)
        );
    }

    // ==================================================================
    // gate_mutegc  (handler.js:827 + mutegc.js:49)
    // ==================================================================

    #[test]
    fn mutegc_stops_without_deleting() {
        let s = GroupSettings {
            mutegc: true,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_mutegc, &caller(), &s, &env()),
            GateOutcome::Decide(Decision::Stop),
            "mutegc tidak pernah menghapus pesan"
        );
    }

    #[test]
    fn mutegc_off_passes() {
        let s = GroupSettings {
            mutegc: false,
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_mutegc, &caller(), &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn mutegc_exempts_admin_owner_and_partner() {
        // Gate SATU-SATUNYA dengan tiga flag exemption (:827).
        let s = GroupSettings {
            mutegc: true,
            ..Default::default()
        };
        for flag in ["is_admin", "is_owner", "is_partner"] {
            let mut c = caller();
            match flag {
                "is_admin" => c.is_admin = true,
                "is_owner" => c.is_owner = true,
                _ => c.is_partner = true,
            }
            assert_eq!(
                run_one(gate_mutegc, &c, &s, &env()),
                GateOutcome::Pass,
                "{flag} harus exempt"
            );
        }
    }

    #[test]
    fn mutegc_ignores_bot_admin_and_from_me() {
        // Berbeda dari antidocument/hidetag: `fromMe` TIDAK exempt di sini.
        let s = GroupSettings {
            mutegc: true,
            ..Default::default()
        };
        let mut c = caller();
        c.is_bot_admin = true;
        c.from_me = true;
        assert_eq!(
            run_one(gate_mutegc, &c, &s, &env()),
            GateOutcome::Decide(Decision::Stop)
        );
    }

    // ==================================================================
    // gate_slowmode  (handler.js:844 + slowmode.js:120)
    // ==================================================================

    fn slowmode_settings(mode: &str) -> GroupSettings {
        GroupSettings {
            slowmode: Some(Slowmode {
                enabled: true,
                delay_secs: Some(30),
                mode: Some(mode.into()),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn slowmode_deletes_when_too_fast() {
        let s = slowmode_settings("all");
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 1_000; // 1s < 30s
        assert_eq!(
            gate_slowmode(&i),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    #[test]
    fn slowmode_onlycommand_stops_silently_without_deleting() {
        // handler.js:849 — `return` tanpa delete. Pesan TIDAK hilang.
        let s = slowmode_settings("onlycommand");
        let e = env();
        let mut c = caller();
        c.is_command = true;
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(gate_slowmode(&i), GateOutcome::Decide(Decision::Stop));
    }

    #[test]
    fn slowmode_onlycommand_passes_non_command_messages() {
        // slowmode.js:129 — `mode === "onlycommand" && !m.isCommand` → false.
        let s = slowmode_settings("onlycommand");
        let e = env();
        let c = caller(); // is_command = false
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(gate_slowmode(&i), GateOutcome::Pass);
    }

    #[test]
    fn slowmode_default_mode_is_all() {
        let s = GroupSettings {
            slowmode: Some(Slowmode {
                enabled: true,
                delay_secs: Some(30),
                mode: None,
            }),
            ..Default::default()
        };
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(
            gate_slowmode(&i),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    #[test]
    fn slowmode_default_delay_is_30_seconds() {
        let s = GroupSettings {
            slowmode: Some(Slowmode {
                enabled: true,
                delay_secs: None,
                mode: Some("all".into()),
            }),
            ..Default::default()
        };
        let e = env();
        // 29s → masih diblokir.
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 29_000;
        assert_eq!(
            gate_slowmode(&i),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
        // 31s → lolos.
        i.last_message_ms = e.now_ms - 31_000;
        assert_eq!(gate_slowmode(&i), GateOutcome::Pass);
    }

    #[test]
    fn slowmode_boundary_is_strictly_less_than() {
        let s = slowmode_settings("all");
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 30_000; // tepat 30s
        assert_eq!(gate_slowmode(&i), GateOutcome::Pass, "tepat = lolos");
        i.last_message_ms = e.now_ms - 29_999;
        assert_eq!(
            gate_slowmode(&i),
            GateOutcome::Decide(Decision::DeleteAndStop)
        );
    }

    #[test]
    fn slowmode_disabled_passes() {
        let s = GroupSettings {
            slowmode: Some(Slowmode {
                enabled: false,
                delay_secs: Some(0),
                mode: Some("all".into()),
            }),
            ..Default::default()
        };
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(gate_slowmode(&i), GateOutcome::Pass);
    }

    #[test]
    fn slowmode_missing_config_passes() {
        let s = GroupSettings::default();
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        assert_eq!(gate_slowmode(&i), GateOutcome::Pass);
    }

    #[test]
    fn slowmode_admin_and_owner_exempt() {
        let s = slowmode_settings("all");
        let e = env();
        for flag in ["is_admin", "is_owner"] {
            let mut c = caller();
            if flag == "is_admin" {
                c.is_admin = true;
            } else {
                c.is_owner = true;
            }
            let mut i = input(&c, &s, &e);
            i.last_message_ms = e.now_ms;
            assert_eq!(gate_slowmode(&i), GateOutcome::Pass, "{flag} harus exempt");
        }
    }

    #[test]
    fn slowmode_partner_and_fromme_are_not_exempt() {
        let s = slowmode_settings("all");
        let e = env();
        for flag in ["is_partner", "from_me"] {
            let mut c = caller();
            if flag == "is_partner" {
                c.is_partner = true;
            } else {
                c.from_me = true;
            }
            let mut i = input(&c, &s, &e);
            i.last_message_ms = e.now_ms;
            assert_eq!(
                gate_slowmode(&i),
                GateOutcome::Decide(Decision::DeleteAndStop),
                "{flag} TIDAK exempt di slowmode"
            );
        }
    }

    // ---- efek samping lastMessageTime.set() ----------------------------

    #[test]
    fn slowmode_records_timestamp_when_time_has_passed() {
        let s = slowmode_settings("all");
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 60_000;
        let (_, record) = default_gate_chain().run_with_record(&i);
        assert!(record, "waktu cukup → caller harus set lastMessageTime");
    }

    #[test]
    fn slowmode_does_not_record_when_blocked() {
        let s = slowmode_settings("all");
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms;
        let (d, record) = default_gate_chain().run_with_record(&i);
        assert_eq!(d, Decision::DeleteAndStop);
        assert!(!record, "diblokir → tidak ada pencatatan waktu");
    }

    #[test]
    fn slowmode_does_not_record_for_exempt_users() {
        let s = slowmode_settings("all");
        let e = env();
        let mut c = caller();
        c.is_owner = true;
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 60_000;
        let (_, record) = default_gate_chain().run_with_record(&i);
        assert!(!record, "exempt tidak masuk slowmode");
    }

    #[test]
    fn slowmode_does_not_record_onlycommand_non_command() {
        let s = slowmode_settings("onlycommand");
        let e = env();
        let c = caller();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 60_000;
        let (_, record) = default_gate_chain().run_with_record(&i);
        assert!(!record, "early-return sebelum cek waktu");
    }

    #[test]
    fn no_record_when_a_prior_gate_blocked() {
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..slowmode_settings("all")
        };
        let e = env();
        let mut i = input(&c, &s, &e);
        i.last_message_ms = e.now_ms - 60_000;
        let (d, record) = default_gate_chain().run_with_record(&i);
        assert!(matches!(d, Decision::DeleteAndReply(_)));
        assert!(!record, "chain berhenti di antilink → slowmode tidak jalan");
    }

    // ==================================================================
    // link detection
    // ==================================================================

    #[test]
    fn contains_link_positives() {
        for t in [
            "example.com",
            "https://example.com",
            "http://www.example.co.id/path?a=1",
            "coba vk.com/x",
            "halo 3.14 itu angka",
            "user@host.tld",
        ] {
            assert!(contains_link(t), "harusnya link: {t}");
        }
    }

    #[test]
    fn contains_link_negatives() {
        for t in [
            "",
            "halo dunia",
            "tapi.",
            "gimana kabar?",
            "selamat pagi",
            "3.",
        ] {
            assert!(!contains_link(t), "harusnya bukan link: {t}");
        }
    }

    #[test]
    fn contains_wa_specific_link_matches_each_pattern() {
        for t in [
            "chat.whatsapp.com/AbCd",
            "wa.me/628123",
            "whatsapp.com/channel/ABC",
            "whatsapp.com/c/ABC",
            "api.whatsapp.com/send",
        ] {
            assert!(contains_wa_specific_link(t), "harusnya cocok: {t}");
        }
    }

    #[test]
    fn contains_wa_specific_link_is_case_insensitive() {
        assert!(contains_wa_specific_link("CHAT.WHATSAPP.COM/ABCD"));
        assert!(contains_wa_specific_link("WA.ME/628"));
    }

    #[test]
    fn contains_wa_specific_link_negatives() {
        for t in [
            "",
            "https://facebook.com",
            "chat.whatsapp.com",
            "chat.whatsapp.com/",
            "wa.me/",
            "whatsapp.com/channel/",
        ] {
            assert!(!contains_wa_specific_link(t), "harusnya tidak cocok: {t}");
        }
    }

    #[test]
    fn contains_any_link_uses_tld_allowlist() {
        for t in ["x.com", "a.b.co.id", "situs.xyz", "toko.shop", "main.ink"] {
            assert!(contains_any_link(t), "harusnya link: {t}");
        }
        for t in ["halo", "file.inventi", "3.5", "a.b_unknown"] {
            assert!(!contains_any_link(t), "harusnya bukan link: {t}");
        }
    }

    #[test]
    fn contains_any_link_excluded_chars_only_stop_at_the_boundary() {
        // Dicek dengan regex aslinya di node: char terlarang sebelum `.`
        // menghentikan scan di posisi itu, tapi regex tetap bisa mulai
        // SETELAH char itu — jadi "<x.com" tetap match.
        assert!(contains_any_link("<x.com"));
        assert!(contains_any_link("\"x.com"));
        assert!(contains_any_link("normal.com"));
        // Yang benar-benar tidak match: char terlarang tepat sebelum `.`.
        assert!(!contains_any_link("a<.com"));
    }

    #[test]
    fn link_scanners_use_js_whitespace_semantics() {
        // JS `\s` tidak memuat NEL (U+0085), tetapi memuat BOM (U+FEFF).
        // `char::is_whitespace()` Rust punya hasil kebalikan untuk dua char ini.
        assert!(contains_any_link("x\u{0085}.com"));
        assert!(!contains_any_link("x\u{feff}.com"));
        assert!(is_at_in_url("http://user@\u{0085}host"));
        assert!(!is_at_in_url("http://user@\u{feff}host"));
    }

    /// `(teks, contains_link, contains_wa_specific_link, contains_any_link)`
    /// hasil `node` atas regex JS asli (group-protection.js:91-94, 1857-1866).
    ///
    /// Tabel ini mengikat scanner tulisan-tangan ke perilaku regex asli.
    /// Scanner bisa disederhanakan selama tabel ini tetap hijau.
    const JS_LINK_TABLE: &[(&str, bool, bool, bool)] = &[
        ("example.com", true, false, true),
        ("https://example.com", true, false, true),
        ("http://www.example.co.id/path?a=1", true, false, true),
        ("coba vk.com/x", true, false, true),
        ("halo 3.14 itu angka", true, false, false),
        ("user@host.tld", true, false, false),
        ("", false, false, false),
        ("halo dunia", false, false, false),
        ("tapi.", false, false, false),
        ("gimana kabar?", false, false, false),
        ("selamat pagi", false, false, false),
        ("3.", false, false, false),
        ("cek https://spam.com", true, false, true),
        ("wa.me/12345", true, true, true),
        ("wa.me/62811", true, true, true),
        ("cek https://contoh.com", true, false, true),
        ("chat.whatsapp.com/", true, false, true),
        ("api.whatsapp.com/send", true, true, true),
        ("api.whatsapp.com/sendXYZ", true, true, true),
        ("cek https://chat.whatsapp.com/AbCd1234", true, true, true),
        ("www.contoh.co.id/page", true, false, true),
        ("file.inventi", false, false, false),
        ("x.dev", true, false, true),
        ("situs.xyz", true, false, true),
        ("toko.shop", true, false, true),
        ("main.ink", true, false, true),
        ("a.b.co.id", true, false, true),
        ("3.5", true, false, false),
        ("a.b_unknown", false, false, false),
        ("<x.com", true, false, true),
        ("\"x.com", true, false, true),
        ("normal.com", true, false, true),
        ("hi chat.whatsapp.com/", true, false, true),
        ("wa.me/", true, false, true),
        ("whatsapp.com/channel/", true, false, true),
        ("CHAT.WHATSAPP.COM/ABCD", true, true, true),
        ("WA.ME/628", true, true, true),
        ("halo semua", false, false, false),
        ("cek https://facebook.com", true, false, true),
        ("chat.whatsapp.com", true, false, true),
        ("halo cek https://x.com sama wa.me/12345", true, true, true),
        ("1.5", true, false, false),
        ("a.b", true, false, false),
        ("halo.dunia", true, false, false),
        ("www.a.co", true, false, true),
        ("x.co.uk", true, false, true),
        ("   ", false, false, false),
        (".com", false, false, false),
        (".co", false, false, false),
        ("a.", false, false, false),
        ("a..b", true, false, false),
        ("cek ini: http://bit.ly/abc", true, false, true),
        ("t.co/abcdef", true, false, true),
        ("visit foo.bar/path#frag", true, false, false),
    ];

    #[test]
    fn link_detection_matches_js_regex() {
        for (text, want_link, want_wa, want_any) in JS_LINK_TABLE {
            assert_eq!(contains_link(text), *want_link, "contains_link({text:?})");
            assert_eq!(
                contains_wa_specific_link(text),
                *want_wa,
                "contains_wa_specific_link({text:?})"
            );
            assert_eq!(
                contains_any_link(text),
                *want_any,
                "contains_any_link({text:?})"
            );
        }
    }

    // ==================================================================
    // helper
    // ==================================================================

    #[test]
    fn is_same_participant_normalizes_device_suffix() {
        assert!(is_same_participant(
            "628999:12@s.whatsapp.net",
            "628999@s.whatsapp.net"
        ));
        assert!(is_same_participant(
            "628999@s.whatsapp.net",
            "628999@s.whatsapp.net"
        ));
    }

    #[test]
    fn is_same_participant_fuzzy_by_digits() {
        assert!(is_same_participant(
            "120628999@s.whatsapp.net",
            "628999@s.whatsapp.net"
        ));
    }

    #[test]
    fn is_same_participant_rejects_empty_and_unrelated() {
        assert!(!is_same_participant("", "628999@s.whatsapp.net"));
        assert!(!is_same_participant("628999@s.whatsapp.net", ""));
        assert!(!is_same_participant("group", ""));
        assert!(!is_same_participant(
            "111@s.whatsapp.net",
            "222@s.whatsapp.net"
        ));
    }

    #[test]
    fn num_hit_requires_8_digit_fuzzy_match() {
        // Tanpa guard panjang, "1" akan cocok dengan semua participant.
        let nums = vec!["628111111".to_string()];
        assert!(!num_hit(&nums, "1"));
        assert!(num_hit(&nums, "628111111"));
    }

    #[test]
    fn admin_check_ignores_non_admin_participants() {
        let parts = vec![
            Participant {
                id: "628111@s.whatsapp.net".into(),
                admin: false,
                ..Default::default()
            },
            admin_participant("628222"),
        ];
        assert!(!is_admin_check(&parts, None, "628111@s.whatsapp.net"));
        assert!(is_admin_check(&parts, None, "628222@s.whatsapp.net"));
    }

    #[test]
    fn admin_check_with_no_candidates_is_false() {
        let parts = vec![admin_participant("628222")];
        assert!(!is_admin_check(&parts, None, ""));
    }

    #[test]
    fn bot_admin_check_false_without_bot_jid() {
        let e = GateEnv {
            participants: vec![admin_participant("628999")],
            ..Default::default()
        };
        assert!(!is_bot_admin_check(&e));
    }

    #[test]
    fn sender_tag_strips_domain() {
        assert_eq!(sender_tag("628111@s.whatsapp.net"), "628111");
        assert_eq!(sender_tag("plain"), "plain");
        assert_eq!(sender_tag(""), "");
    }

    #[test]
    fn decision_reply_text_returns_text_only_for_replying_variants() {
        assert_eq!(Decision::Reply("a".into()).reply_text(), Some("a"));
        assert_eq!(Decision::DeleteAndReply("b".into()).reply_text(), Some("b"));
        assert_eq!(Decision::KickAndReply("c".into()).reply_text(), Some("c"));
        assert_eq!(Decision::Allow.reply_text(), None);
        assert_eq!(Decision::Stop.reply_text(), None);
        assert_eq!(Decision::DeleteAndStop.reply_text(), None);
    }

    // ==================================================================
    // normalizeProtectionText / buildProtectionTextVariants
    // (group-protection.js:313-338) — kasus diverifikasi dengan node
    // terhadap JS asli.
    // ==================================================================

    #[test]
    fn normalize_protection_text_ports_js_cases() {
        // Pemetaan LEET jalan SEBELUM buang non-alnum: "!!!" jadi "iii".
        assert_eq!(normalize_protection_text("SLOT  GACOR!!!"), "slot gacoriii");
        assert_eq!(normalize_protection_text("j u d i"), "judi");
        assert_eq!(normalize_protection_text("jud0l"), "judol");
        assert_eq!(normalize_protection_text("s-l-o-t"), "slot");
        assert_eq!(normalize_protection_text("a b c d e f"), "abcdef");
        // Kuantifier {2,} serakah: seluruh run huruf tunggal ikut.
        assert_eq!(normalize_protection_text("x y z a b c"), "xyzabc");
        // "zz" bukan huruf tunggal → run tidak ikut.
        assert_eq!(normalize_protection_text("x y zz"), "x y zz");
    }

    #[test]
    fn build_protection_text_variants_dedup_and_order() {
        assert_eq!(
            build_protection_text_variants("Slot Gacor"),
            vec!["slot gacor".to_string(), "slotgacor".to_string()]
        );
        // `!` dipetakan LEET ke `i` sebelum non-alnum dibuang; JS menyimpan
        // raw "!!!" dan normalized "iii" sebagai dua varian.
        assert_eq!(
            build_protection_text_variants("!!!"),
            vec!["!!!".to_string(), "iii".to_string()]
        );
    }

    // ==================================================================
    // detectJudol / detectPhishing (group-protection.js:393-440) — skor
    // dikunci persis dari node (lihat komentar di atas).
    // ==================================================================

    #[test]
    fn detect_judol_scores_match_node_probe() {
        // "slot gacor": pola slot(3)+gacor(3)+slot gacor(4) + compact
        // slot(3)+gacor(3)+slotgacor(4) = 20 — skor DIDIHITUNG GANDA antara
        // pola dan compact, persis JS.
        let r = detect_judol("slot gacor");
        assert!(r.matched);
        assert_eq!(r.score, 20);
        assert_eq!(r.matches, vec!["slot", "gacor", "slot gacor"]);

        // compact "slotgacor": slot(3)+gacor(3)+slotgacor(4) = 10.
        let r = detect_judol("slotgacor");
        assert!(r.matched);
        assert_eq!(r.score, 10);

        // "jud0l" → normalized "judol": pola judi(3) + compact judol(3).
        let r = detect_judol("jud0l");
        assert!(r.matched);
        assert_eq!(r.score, 6);
        assert_eq!(r.matches, vec!["judi", "judol"]);

        // "j u d i" → kompaksi "judi" → 6.
        assert!(detect_judol("j u d i").matched);

        // Ambang: 2 tanpa link gagal…
        let r = detect_judol("depo wd");
        assert!(!r.matched);
        assert_eq!(r.score, 2);
        // …dan 2 DENGAN link lolos (hasLink && >= 2).
        let r = detect_judol("depo wd http://x.com");
        assert!(r.matched);
        assert!(r.has_link);

        // Word boundary: "deposito" bukan "deposit"/"depo".
        let r = detect_judol("saya deposito di bank");
        assert!(!r.matched);
        assert_eq!(r.score, 0);

        // compact menangkap "casinoku" yang lolos dari \bcasino\b.
        let r = detect_judol("casinoku");
        assert!(r.matched);
        assert_eq!(r.score, 3);

        // Kompaksi huruf tunggal: "r t p" → "rtp".
        let r = detect_judol("r t p");
        assert!(r.matched);
        assert_eq!(r.score, 4);

        // \bwd\b gagal di "wdSlot" tapi compact "slot" kena.
        let r = detect_judol("wdSlot");
        assert!(r.matched);
        assert_eq!(r.matches, vec!["slot"]);

        // Sub-ambang: bet saja (1).
        assert!(!detect_judol("bet kecil").matched);
        // Pola `taruhan` (skor 2) ada di JS; tanpa link tetap belum mencapai
        // ambang 3, tetapi label dan skornya harus sama.
        let r = detect_judol("taruhan online");
        assert!(!r.matched);
        assert_eq!(r.score, 2);
        assert_eq!(r.matches, vec!["taruhan"]);
        assert!(!detect_judol("halo dunia").matched);
    }

    #[test]
    fn detect_phishing_scores_match_node_probe() {
        let r = detect_phishing("klik link");
        assert!(r.matched);
        assert_eq!(r.score, 4);
        assert_eq!(r.matches, vec!["klik link"]);

        let r = detect_phishing("klik link http://bit.ly/xx");
        assert!(r.matched);
        assert_eq!(r.score, 6);
        assert_eq!(r.matches, vec!["klik link", "shortener"]);
        assert!(r.has_link);

        let r = detect_phishing("reset password");
        assert!(r.matched);
        assert_eq!(r.score, 8);
        assert_eq!(r.matches, vec!["credential", "reset", "reset password"]);

        // IP URL +2 tapi hasLink false (IP bukan TLD) dan skor 2 < 3 → gagal.
        let r = detect_phishing("cek http://192.168.1.1/login");
        assert!(!r.matched);
        assert_eq!(r.score, 2);
        assert_eq!(r.matches, vec!["ip-url"]);
        assert!(!r.has_link);

        // t.co +2 saja, tapi hasLink → (hasLink && >= 2) lolos.
        let r = detect_phishing("klik https://t.co/abc");
        assert!(r.matched);
        assert_eq!(r.score, 2);
        assert!(r.has_link);

        // Regex URL tambahan JS memakai flag `i`; kapitalisasi scheme tidak
        // boleh menghilangkan skor IP-URL atau userinfo.
        let r = detect_phishing("HTTP://192.168.1.1/login");
        assert_eq!(r.score, 2);
        assert_eq!(r.matches, vec!["ip-url"]);
        let r = detect_phishing("HTTPS://user@example.com/x");
        assert_eq!(r.score, 2);
        assert_eq!(r.matches, vec!["url-userinfo"]);

        assert!(detect_phishing("verifikasi akun sekarang").matched);
        let r = detect_phishing("isi formulir");
        assert!(r.matched);
        assert_eq!(r.matches, vec!["form", "isi form"]);

        assert!(!detect_phishing("log in sini").matched);
        assert!(!detect_phishing("hadiah menanti").matched);
        assert!(!detect_phishing("halo dunia").matched);
    }

    // ==================================================================
    // Mesin regex mini — semantik inti yang dipakai tabel pola + rule.
    // ==================================================================

    #[test]
    fn mini_regex_matches_js_semantics() {
        let re = |src: &str| MiniRegex::new(src, "i").expect("pola uji valid");
        // Word boundary.
        assert!(re(r"\bslot\b").test("slot"));
        assert!(!re(r"\bslot\b").test("slotgacor"));
        assert!(!re(r"\bslot\b").test("meslot"));
        // Kelas + kuantifier terbatas: jud[iol]{1,4}.
        assert!(re(r"\bjud[iol]{1,4}\b").test("judol"));
        assert!(re(r"\bjud[iol]{1,4}\b").test("judi"));
        // Lima karakter setelah "jud" melewati batas `{1,4}`.
        assert!(!re(r"\bjud[iol]{1,4}\b").test("judolllll"));
        // Alternasi + \s? opsional.
        assert!(re(r"\bmax\s?win\b").test("maxwin"));
        assert!(re(r"\bmax\s?win\b").test("max win"));
        assert!(!re(r"\bmax\s?win\b").test("maxxwin"));
        // Kelompok alternasi dengan \s* di cabang.
        assert!(re(r"bonus\s+(member|new\s*member)").test("bonus new   member"));
        assert!(!re(r"bonus\s+(member|new\s*member)").test("bonusmember"));
        // Rentang kelas + nested quantifier bounded.
        assert!(re(r"[0-9]{4,12}").test("123456"));
        assert!(!re(r"[0-9]{4,12}").test("123"));
        // Flag i.
        assert!(re("anjing").test("ANJING"));
        // Alternasi: cabang pertama gagal harus mencoba cabang kedua, bukan
        // langsung menganggap regex cocok.
        assert!(re("anjing|goblok").test("anjing"));
        assert!(re("anjing|goblok").test("goblok"));
        assert!(!re("anjing|goblok").test("kamu baik"));
        // Kuantifier opsional tidak boleh melewati assertion di بعدnya.
        assert!(!re(r"\bfoo\d?\b").test("foox"));
        // Titik + lazy.
        assert!(MiniRegex::new("a.c", "").unwrap().test("abc"));
        assert!(!MiniRegex::new("a.c", "").unwrap().test("a\nc"));
        assert!(MiniRegex::new("a.c", "s").unwrap().test("a\nc"));
        // Anchor.
        assert!(MiniRegex::new("^anjing$", "").unwrap().test("anjing"));
        assert!(!MiniRegex::new("^anjing$", "").unwrap().test("xanjing"));
        // `{` literal bila bukan kuantifier valid.
        assert!(re(r"a{").test("a{"));
        // Backreference/lookahead ditolak (fail-closed).
        assert!(MiniRegex::new(r"(a*)\1", "").is_err());
        assert!(MiniRegex::new(r"(?=anjing)a", "").is_err());
    }

    #[test]
    fn mini_regex_replace_all_matches_js_global_semantics() {
        let re = MiniRegex::new(r"\b(?:[a-z0-9]\s+){2,}[a-z0-9]\b", "").unwrap();
        let out = re.replace_all("x y z a b c", |f: &str| {
            f.chars().filter(|c| !c.is_whitespace()).collect()
        });
        assert_eq!(out, "xyzabc");
    }

    #[test]
    fn mini_regex_js_rejects_bounded_quantifier_with_max_less_than_min() {
        // `new RegExp("a{2,1}")` melempar SyntaxError di JS.
        assert!(MiniRegex::new("a{2,1}", "").is_err());
    }

    #[test]
    fn mini_regex_js_dot_excludes_all_line_terminators() {
        let re = MiniRegex::new("a.c", "").unwrap();
        for terminator in ['\n', '\r', '\u{2028}', '\u{2029}'] {
            assert!(
                !re.test(&format!("a{terminator}c")),
                "dot harus berhenti sebelum {terminator:?}"
            );
        }
        assert!(MiniRegex::new("a.c", "s").unwrap().test("a\u{2028}c"));
    }

    #[test]
    fn mini_regex_js_multiline_anchors_use_all_line_terminators() {
        let start = MiniRegex::new("^b", "m").unwrap();
        let end = MiniRegex::new("a$", "m").unwrap();
        for terminator in ['\n', '\r', '\u{2028}', '\u{2029}'] {
            assert!(
                start.test(&format!("a{terminator}b")),
                "^ harus cocok setelah {terminator:?}"
            );
            assert!(
                end.test(&format!("a{terminator}b")),
                "$ harus cocok sebelum {terminator:?}"
            );
        }
    }

    // ==================================================================
    // analyzeCustomPattern / matchCustomRule — daftar kasus diport dari
    // tests/anticustom-pattern-safety.test.mjs.
    // ==================================================================

    #[test]
    fn analyze_custom_pattern_rejects_bad_list() {
        let bad: &[(&str, &str)] = &[
            (
                "(a+)+$",
                "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)",
            ),
            (
                "(?:a+)+",
                "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)",
            ),
            (
                "((a+)+)+$",
                "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)",
            ),
            (
                "([a-z]+)*$",
                "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)",
            ),
            (
                "(a{1,2})+$",
                "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)",
            ),
            (
                "(a|a)+$",
                "dua cabang alternasi di bawah kuantifier bisa cocok dengan teks yang sama (mis. `(a|ab)+`)",
            ),
            (
                "(a|ab)+c",
                "dua cabang alternasi di bawah kuantifier bisa cocok dengan teks yang sama (mis. `(a|ab)+`)",
            ),
            (
                "(anj|anjing)+",
                "dua cabang alternasi di bawah kuantifier bisa cocok dengan teks yang sama (mis. `(a|ab)+`)",
            ),
            (
                "(a|)+b",
                "isi kelompok yang diulang bisa kosong (mis. `(a*|b)+`)",
            ),
            (
                "a*a*b",
                "dua kuantifier bertumpuk menebak karakter yang sama (mis. `a*a*`)",
            ),
            (
                "[a-z]+[a-z]+",
                "dua kuantifier bertumpuk menebak karakter yang sama (mis. `a*a*`)",
            ),
            (
                "(a+)(a+)$",
                "karakter setelah kuantifier bisa ikut ditelan kuantifier itu (mis. `.*a`)",
            ),
            (
                "(a*|b)+",
                "ada kuantifier di dalam kelompok yang juga diulang (mis. `(a+)+`)",
            ),
            (
                "a*a*a*a*a*a*a*b",
                "dua kuantifier bertumpuk menebak karakter yang sama (mis. `a*a*`)",
            ),
            (
                "(a*)\\1",
                "karakter setelah kuantifier bisa ikut ditelan kuantifier itu (mis. `.*a`)",
            ),
        ];
        for (pattern, reason) in bad {
            assert_eq!(
                analyze_custom_pattern(pattern),
                Err(reason.to_string()),
                "{pattern}"
            );
        }
    }

    #[test]
    fn analyze_custom_pattern_accepts_good_list() {
        for pattern in [
            "anjing",
            "anjing|goblok|tolol",
            "(anjing|goblok)+",
            "(anj|anjing|a+n+j+)",
            r"\banjing\b",
            "[0-9]{4,12}",
            r"\w+\s*\w+\s*\w+",
            r"\S+\s*\S+",
            ".*anjing",
            r"free\.fire",
            r"(anj|anjing|a+n+j+)\b",
        ] {
            assert!(analyze_custom_pattern(pattern).is_ok(), "{pattern}");
        }
    }

    #[test]
    fn analyze_custom_pattern_limits() {
        assert_eq!(
            analyze_custom_pattern(&"a".repeat(CUSTOM_MAX_PATTERN_LEN + 1)),
            Err("pola terlalu panjang (maks 120 karakter)".to_string())
        );
        assert!(analyze_custom_pattern(&"a".repeat(CUSTOM_MAX_PATTERN_LEN)).is_ok());
        // Panjang JS dihitung dalam code unit UTF-16. 119 emoji = 238 unit,
        // lalu satu huruf membuat 239 unit — melewati batas meski hanya 120
        // scalar Rust.
        let long_utf16 = format!("{}a", "😀".repeat(119));
        assert_eq!(
            analyze_custom_pattern(&long_utf16),
            Err("pola terlalu panjang (maks 120 karakter)".to_string())
        );
        assert_eq!(analyze_custom_pattern(""), Err("pola kosong".to_string()));
        // Kelas tanpa penutup → JS `new RegExp` throw → alasan parse.
        assert_eq!(
            analyze_custom_pattern("[abc"),
            Err("pola tidak bisa diurai (kurung kurawal tidak seimbang)".to_string())
        );
    }

    fn cr(pattern: &str, r#type: &str, extra: CustomRule) -> CustomRule {
        CustomRule {
            pattern: pattern.to_string(),
            rule_type: r#type.to_string(),
            ..extra
        }
    }

    #[test]
    fn match_custom_rule_contains_and_regex_ports_js_cases() {
        // "rule contains biasa tetap jalan".
        let rule = cr(
            "Goblok",
            "contains",
            CustomRule {
                name: "kotor".into(),
                ..Default::default()
            },
        );
        assert!(match_custom_rule("halo bro ini GOBLOK banget", &[rule.clone()]).is_some());
        assert!(match_custom_rule("halo bro ini bersih", &[rule.clone()]).is_none());

        // "regex aman tetap jalan".
        let re = cr(
            "anjing|goblok",
            "regex",
            CustomRule {
                flags: Some("i".into()),
                ..Default::default()
            },
        );
        assert!(match_custom_rule("kamu anjing", &[re.clone()]).is_some());
        assert!(match_custom_rule("kamu baik", &[re]).is_none());

        // "flag g tidak lagi membuat hasil bolong-balik" (deterministik).
        let sticky = cr(
            "anjing",
            "regex",
            CustomRule {
                flags: Some("gi".into()),
                ..Default::default()
            },
        );
        assert!(match_custom_rule("anjing", &[sticky.clone()]).is_some());
        assert!(match_custom_rule("anjing", &[sticky]).is_some());

        // `new RegExp(pattern, "uv")` throw di JS — rule harus inert.
        let invalid_unicode_flags = cr(
            "anjing",
            "regex",
            CustomRule {
                flags: Some("uv".into()),
                ..Default::default()
            },
        );
        assert!(match_custom_rule("anjing", &[invalid_unicode_flags]).is_none());

        // "rule regex yang tersimpan sebelum patch tidak membekukan bot" —
        // rule berbahaya inert, dan cepat.
        let stale = vec![
            cr("(a+)+$", "regex", CustomRule::default()),
            cr("(a|a)+$", "regex", CustomRule::default()),
            cr("a*a*a*a*a*b", "regex", CustomRule::default()),
        ];
        let text = format!("{}X", "a".repeat(400));
        let t0 = Instant::now();
        assert!(
            match_custom_rule(&text, &stale).is_none(),
            "rule berbahaya tidak boleh match"
        );
        assert!(
            t0.elapsed() < Duration::from_secs(2),
            "eval harus cepat, bukan stall"
        );
    }

    #[test]
    fn match_custom_rule_caps_subject_and_rule_count() {
        // Subjek dipotong ke 512 char: token di posisi 600 tidak terlihat.
        let far = cr(
            "zztoken",
            "contains",
            CustomRule {
                name: "far".into(),
                ..Default::default()
            },
        );
        assert!(
            match_custom_rule(&format!("{}zztoken", "x".repeat(600)), &[far.clone()]).is_none()
        );
        assert!(match_custom_rule(&format!("{}zztoken", "x".repeat(500)), &[far]).is_some());

        // `String(text).slice(0, 512)` memotong code unit UTF-16. 255 emoji
        // memakai 510 unit; dua unit tersisa hanya memotong awal token.
        let utf16_far = cr(
            "zztoken",
            "contains",
            CustomRule {
                name: "utf16".into(),
                ..Default::default()
            },
        );
        let utf16_text = format!("{}zztoken", "😀".repeat(255));
        assert!(match_custom_rule(&utf16_text, &[utf16_far]).is_none());

        // Maks 50 rule per pesan: rule ke-51 tidak dievaluasi.
        let rules: Vec<CustomRule> = (0..=CUSTOM_MAX_RULES)
            .map(|i| {
                cr(
                    &format!("zz{i:03}z"),
                    "contains",
                    CustomRule {
                        name: format!("r{i}"),
                        ..Default::default()
                    },
                )
            })
            .collect();
        assert!(match_custom_rule("halo zz001z", &rules).is_some());
        let beyond = &rules[CUSTOM_MAX_RULES];
        assert!(match_custom_rule(&format!("halo {}", beyond.pattern), &rules).is_none());
    }

    // ==================================================================
    // gate_anti_judol  (group-protection.js:1998, 1253-1309)
    // ==================================================================

    #[test]
    fn judol_off_lets_message_through() {
        let mut c = caller();
        c.body = "slot gacor".into();
        let s = GroupSettings {
            antijudol: Some("off".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_judol, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn judol_below_threshold_passes() {
        // "depo wd" = 2 < 3 tanpa link → tidak trigger.
        let mut c = caller();
        c.body = "depo wd".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_judol, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn judol_deletes_and_replies() {
        let mut c = caller();
        c.body = "buruan slot gacor".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_judol, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTIJUDOL.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn judol_link_lowers_threshold() {
        // hasLink && skor 2 → lolos ambang.
        let mut c = caller();
        c.body = "depo wd http://x.com".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_anti_judol, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn judol_kick_mode_uses_normalized_mode() {
        // normalizeProtectionMode: "kick" → kick; "delete" → remove;
        // nilai aneh → fallback remove.
        let mut c = caller();
        c.body = "slot gacor".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            antijudol_mode: Some("kick".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_judol, &c, &s, &env()),
            GateOutcome::Decide(Decision::KickAndReplyFallback {
                kick: ANTIJUDOL_KICK.replace("%user%", "628111"),
                fallback: ANTIJUDOL.replace("%user%", "628111"),
            })
        );
        let s = GroupSettings {
            antijudol: Some("on".into()),
            antijudol_mode: Some("delete".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_anti_judol, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
        let s = GroupSettings {
            antijudol: Some("on".into()),
            antijudol_mode: Some("nonsense".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_anti_judol, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn judol_admin_is_exempt() {
        let mut c = caller();
        c.body = "slot gacor".into();
        c.sender = "628222@s.whatsapp.net".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_judol, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn judol_replies_notadmin_when_bot_not_admin() {
        let mut c = caller();
        c.body = "slot gacor".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(
            run_one(gate_anti_judol, &c, &s, &e),
            GateOutcome::Decide(Decision::Reply(NOT_ADMIN.into()))
        );
    }

    #[test]
    fn judol_skips_bot_only_when_self_mode_off() {
        // handleTextProtection (:1276): skip bot sendiri HANYA saat selfMode
        // mati. Saat selfMode nyala, pesan bot diproses → di env tanpa
        // tercatat admin, bot membalas notAdmin ke dirinya (persis JS).
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        let mut c = caller();
        c.body = "slot gacor".into();
        c.sender = BOT.into();

        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(run_one(gate_anti_judol, &c, &s, &e), GateOutcome::Pass);

        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        e.self_mode = true;
        assert_eq!(
            run_one(gate_anti_judol, &c, &s, &e),
            GateOutcome::Decide(Decision::Reply(NOT_ADMIN.into()))
        );
    }

    // ==================================================================
    // gate_anti_phising  (group-protection.js:2011)
    // ==================================================================

    #[test]
    fn phising_off_lets_message_through() {
        let mut c = caller();
        c.body = "verifikasi akun sekarang".into();
        let s = GroupSettings {
            antiphising: Some("off".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_phising, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn phising_triggers_and_replies() {
        let mut c = caller();
        c.body = "verifikasi akun sekarang".into();
        let s = GroupSettings {
            antiphising: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_phising, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTIPHISING.replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn phising_kick_mode() {
        let mut c = caller();
        c.body = "reset password kamu sekarang".into();
        let s = GroupSettings {
            antiphising: Some("on".into()),
            antiphising_mode: Some("kick".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_phising, &c, &s, &env()),
            GateOutcome::Decide(Decision::KickAndReplyFallback {
                kick: ANTIPHISING_KICK.replace("%user%", "628111"),
                fallback: ANTIPHISING.replace("%user%", "628111"),
            })
        );
    }

    #[test]
    fn phising_below_threshold_without_link_passes() {
        // "klik link" saja = 4 ≥ 3 → trigger; "hadiah menanti" = 1 → lolos.
        let mut c = caller();
        c.body = "hadiah menanti buat kamu".into();
        let s = GroupSettings {
            antiphising: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_phising, &c, &s, &env()),
            GateOutcome::Pass
        );
    }

    // ==================================================================
    // gate_anti_custom  (group-protection.js:2024-2073)
    // ==================================================================

    fn custom_settings(rules: Vec<CustomRule>, mode: Option<&str>) -> GroupSettings {
        GroupSettings {
            anticustom: Some("on".into()),
            anticustom_mode: mode.map(str::to_string),
            anticustom_rules: rules,
            ..Default::default()
        }
    }

    #[test]
    fn custom_off_lets_message_through() {
        let mut c = caller();
        c.body = "halo bro ini GOBLOK banget".into();
        let s = GroupSettings {
            anticustom: Some("off".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_custom, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn custom_contains_rule_replies_with_rule_name() {
        let mut c = caller();
        c.body = "halo bro ini GOBLOK banget".into();
        let rule = CustomRule {
            name: "kotor".into(),
            pattern: "Goblok".into(),
            rule_type: "contains".into(),
            ..Default::default()
        };
        assert_eq!(
            run_one(
                gate_anti_custom,
                &c,
                &custom_settings(vec![rule], None),
                &env()
            ),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTICUSTOM
                    .replace("%user%", "628111")
                    .replace("%rule%", "kotor")
            ))
        );
    }

    #[test]
    fn custom_regex_rule_matches() {
        let mut c = caller();
        c.body = "kamu anjing".into();
        let rule = CustomRule {
            pattern: "anjing|goblok".into(),
            rule_type: "regex".into(),
            flags: Some("i".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(
                gate_anti_custom,
                &c,
                &custom_settings(vec![rule], None),
                &env()
            ),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn custom_no_matching_rule_passes() {
        let mut c = caller();
        c.body = "kamu baik".into();
        let rule = CustomRule {
            pattern: "anjing|goblok".into(),
            rule_type: "regex".into(),
            ..Default::default()
        };
        assert_eq!(
            run_one(
                gate_anti_custom,
                &c,
                &custom_settings(vec![rule], None),
                &env()
            ),
            GateOutcome::Pass
        );
    }

    #[test]
    fn custom_rule_action_overrides_group_mode() {
        // rule.action = "kick" menimpa anticustomMode = "remove" (:2053).
        let mut c = caller();
        c.body = "anjing".into();
        let rule = CustomRule {
            pattern: "anjing".into(),
            rule_type: "contains".into(),
            action: Some("kick".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(
                gate_anti_custom,
                &c,
                &custom_settings(vec![rule], Some("remove")),
                &env()
            ),
            GateOutcome::Decide(Decision::KickAndReplyFallback { .. })
        ));
    }

    #[test]
    fn custom_group_mode_used_when_rule_action_missing() {
        let mut c = caller();
        c.body = "anjing".into();
        let rule = CustomRule {
            pattern: "anjing".into(),
            rule_type: "contains".into(),
            ..Default::default()
        };
        assert!(matches!(
            run_one(
                gate_anti_custom,
                &c,
                &custom_settings(vec![rule], Some("kick")),
                &env()
            ),
            GateOutcome::Decide(Decision::KickAndReplyFallback { .. })
        ));
    }

    #[test]
    fn custom_rule_label_falls_back_to_pattern() {
        // rule.name kosong → %rule% = pattern (:2066-2068).
        let mut c = caller();
        c.body = "anjing".into();
        let rule = CustomRule {
            pattern: "anjing".into(),
            rule_type: "contains".into(),
            ..Default::default()
        };
        let outcome = run_one(
            gate_anti_custom,
            &c,
            &custom_settings(vec![rule], None),
            &env(),
        );
        match outcome {
            GateOutcome::Decide(Decision::DeleteAndReply(text)) => {
                assert!(text.contains("*anjing*"), "{text}");
            }
            other => panic!("harus DeleteAndReply, dapat {other:?}"),
        }
    }

    #[test]
    fn custom_dangerous_rule_is_inert() {
        // Pola berbahaya dianalisis tidak aman → rule tidak pernah match.
        let mut c = caller();
        c.body = format!("{}X", "a".repeat(400));
        let rule = CustomRule {
            pattern: "(a+)+$".into(),
            rule_type: "regex".into(),
            ..Default::default()
        };
        assert_eq!(
            run_one(
                gate_anti_custom,
                &c,
                &custom_settings(vec![rule], None),
                &env()
            ),
            GateOutcome::Pass
        );
    }

    #[test]
    fn custom_replies_notadmin_when_bot_not_admin() {
        let mut c = caller();
        c.body = "anjing".into();
        let rule = CustomRule {
            pattern: "anjing".into(),
            rule_type: "contains".into(),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(
            run_one(gate_anti_custom, &c, &custom_settings(vec![rule], None), &e),
            GateOutcome::Decide(Decision::Reply(NOT_ADMIN.into()))
        );
    }

    // ==================================================================
    // gate_anti_swgc  (group-protection.js:2075-2143) — diport dari
    // tests/antiswgc.test.mjs di level gate.
    // ==================================================================

    #[test]
    fn swgc_off_lets_message_through() {
        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        let s = GroupSettings {
            antiswgc: Some("off".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_swgc, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn swgc_passes_without_sw_type() {
        // Pesan biasa (sw_group_type None) tidak boleh disentuh.
        let c = caller();
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_swgc, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn swgc_deletes_and_replies_with_type() {
        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        // `resolveMessageSenderJid(key, sock)` memakai `key.participant`,
        // bukan `m.sender` yang sudah di-resolve ke PN.
        c.participant = Some(c.sender.clone());
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_swgc, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(
                ANTISWGC
                    .replace("%type%", "message.groupMentionedMessage")
                    .replace("%user%", "628111")
            ))
        );
    }

    #[test]
    fn swgc_never_acts_on_bots_own_message() {
        // :2096-2102 — fromMe, PN bot, dan LID bot ketiganya skip (tanpa
        // memandang selfMode).
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            ..Default::default()
        };

        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        c.from_me = true;
        assert_eq!(run_one(gate_anti_swgc, &c, &s, &env()), GateOutcome::Pass);

        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        c.sender = BOT.into();
        c.participant = Some(BOT.into());
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(run_one(gate_anti_swgc, &c, &s, &e), GateOutcome::Pass);

        // LID bot di `env.bot_lid`.
        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        c.participant = Some("999888777666555@lid".into());
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        e.bot_lid = Some("999888777666555@lid".into());
        assert_eq!(run_one(gate_anti_swgc, &c, &s, &e), GateOutcome::Pass);
    }

    #[test]
    fn swgc_admin_exempt_via_participant_only() {
        // :2106 — senderCand hanya key.participant yang ternormalisasi.
        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        c.participant = Some("628222@s.whatsapp.net".into());
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(run_one(gate_anti_swgc, &c, &s, &env()), GateOutcome::Pass);
    }

    #[test]
    fn swgc_replies_notadmin_when_bot_not_admin() {
        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            ..Default::default()
        };
        let mut e = env();
        e.participants.retain(|p| !p.id.starts_with("628999"));
        assert_eq!(
            run_one(gate_anti_swgc, &c, &s, &e),
            GateOutcome::Decide(Decision::Reply(NOT_ADMIN.into()))
        );
    }

    // ==================================================================
    // ORDER — posisi baru dalam rantai default.
    // ==================================================================

    #[test]
    fn order_judol_wins_over_antilink() {
        // handler.js:773 — judol dijalankan sebelum antilink (:782).
        let mut c = caller();
        c.body = "slot gacor https://spam.com".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            antilink: Some("on".into()),
            ..Default::default()
        };
        let e = env();
        let d = default_gate_chain().run(&input(&c, &s, &e));
        assert!(
            d.reply_text().unwrap().contains("AntiJudol"),
            "judol (:773) harus menang atas antilink (:782): {d:?}"
        );
    }

    #[test]
    fn order_antiswgc_beats_antidocument() {
        // handler.js:794 — antiswgc sebelum antidocument (:797).
        let mut c = caller();
        c.is_document = true;
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            antidocument: true,
            ..Default::default()
        };
        let e = env();
        let d = default_gate_chain().run(&input(&c, &s, &e));
        assert!(
            d.reply_text().unwrap().contains("AntiSWGC"),
            "antiswgc (:794) harus menang atas antidocument (:797): {d:?}"
        );
    }

    // ==================================================================
    // METADATA GRUP GAGAL — gate dilewati SENYAP
    // ==================================================================

    /// Env tanpa metadata: roster kosong + `metadata_available = false`.
    ///
    /// Ini keadaan JS setelah `await sock.groupMetadata(m.chat)` melempar:
    /// `catch` → `return false` → gate dilewati tanpa balasan
    /// (`group-protection.js:1330/1278/2104/2153`).
    fn env_tanpa_metadata() -> GateEnv {
        GateEnv {
            bot_number: BOT.into(),
            metadata_available: false,
            participants: Vec::new(),
            now_ms: 100_000,
            ..Default::default()
        }
    }

    #[test]
    fn metadata_gagal_makes_gate_antilink_dilewati_senyap() {
        // Tanpa flag ini roster kosong terbaca "bot bukan admin" dan gate
        // membalas `notAdmin` — balasan yang di JS tidak pernah muncul.
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink, &c, &s, &env_tanpa_metadata()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn metadata_gagal_makes_gate_gc_dan_judol_dilewati_senyap() {
        let mut c = caller();
        c.body = "wa.me/12345".into();
        let s = GroupSettings {
            antilinkgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_antilink_gc, &c, &s, &env_tanpa_metadata()),
            GateOutcome::Pass
        );

        let mut c = caller();
        c.body = "buruan slot gacor".into();
        let s = GroupSettings {
            antijudol: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_judol, &c, &s, &env_tanpa_metadata()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn metadata_gagal_makes_gate_hidetag_dan_antiswgc_dilewati_senyap() {
        let mut c = caller();
        c.mentioned_jid = vec!["a@s.whatsapp.net".into()];
        c.participant_count = 0;
        let s = GroupSettings {
            antihidetag: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_hidetag, &c, &s, &env_tanpa_metadata()),
            GateOutcome::Pass
        );

        let mut c = caller();
        c.sw_group_type = Some("message.groupMentionedMessage".into());
        c.participant = Some(c.sender.clone());
        let s = GroupSettings {
            antiswgc: Some("on".into()),
            ..Default::default()
        };
        assert_eq!(
            run_one(gate_anti_swgc, &c, &s, &env_tanpa_metadata()),
            GateOutcome::Pass
        );
    }

    #[test]
    fn metadata_tersedia_tetap_menghasilkan_keputusan_normal() {
        // Kebalikan dari test di atas: metadata ada → gate tetap sadar roster.
        let mut c = caller();
        c.body = "https://spam.com".into();
        let s = GroupSettings {
            antilink: Some("on".into()),
            ..Default::default()
        };
        assert!(matches!(
            run_one(gate_antilink, &c, &s, &env()),
            GateOutcome::Decide(Decision::DeleteAndReply(_))
        ));
    }

    #[test]
    fn kick_fallback_hanya_ada_pada_varian_yang_memilikinya() {
        // `handleAntilink` (:1349) tidak menangkap error kick → tanpa balasan.
        // `handleAntilinkGc/All` (:1921/1976) menangkapnya → balas mode-remove.
        assert_eq!(
            Decision::KickAndReplyFallback {
                kick: "k".into(),
                fallback: "f".into()
            }
            .kick_fallback(),
            Some("f")
        );
        assert_eq!(Decision::KickAndReply("k".into()).kick_fallback(), None);
        assert_eq!(Decision::Reply("r".into()).kick_fallback(), None);
        assert_eq!(Decision::DeleteAndReply("d".into()).kick_fallback(), None);
        assert_eq!(
            Decision::KickAndReplyFallback {
                kick: "k".into(),
                fallback: "f".into()
            }
            .reply_text(),
            Some("k")
        );
    }
}
