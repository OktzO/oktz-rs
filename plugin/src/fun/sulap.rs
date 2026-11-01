//! `plugins/fun/sulap.js` — pertunjukan sulap: admin "menghilangkan" member.
//!
//! ## Penyimpangan
//!
//! - **Kunci sesi.** JS mengunci sesi dengan `sent.key.id` (id pesan prompt yang
//!   baru dikirim). `Reply` tidak pernah mengembalikan id pesan terkirim, jadi
//!   sesi dikunci `sulapSessions:<chat>:<admin>` — padanan semantiknya, karena
//!   guard `session.admin !== m.sender` tetap dijalankan di alur reply.
//! - **Kedaluwarsa 120 detik** di JS lewat `setTimeout`; di sini dicek malas
//!   saat alur reply membaca sesi (timestamp disimpan apa adanya).
//! - **JID bot** (`sock.user?.id`) tidak ada di `Ctx`, jadi dua guard bot
//!   (`m.quoted.sender !== sock.user?.id` dan `targetNumber === botNumber`)
//!   dilewati. Cabang tanpa mention: kalau pesan yang di-quote punya sender
//!   (orang lain), alur berhenti seperti JS; kalau tidak ada, keluar teks
//!   "Mention orangnya dong!".
//! - **`replyHandler` di-port sebagai `answer_handler`** (pola `confess.rs`):
//!   di JS ia berjalan untuk pesan biasa yang me-quote prompt, bukan command.
//! - **`await sleep(2000)`** sebelum kick tidak ditiru — handler Rust sinkron,
//!   dan jeda dramatis tidak mengubah output.

use oktz_plugin_api::{AksiPeserta, Ctx, Reply, plugin};
use serde_json::json;

use crate::common::{jid_number, now_ms, random_index};

/// Peta sesi JS punya sendiri (`global.sulapSessions`), sedangkan
/// `Ctx::sessions` satu map global — makanya diberi awalan.
const SESSION_PREFIX: &str = "sulapSessions";

/// Umur sesi 120 detik (`sulap.js:45-47`).
const EXPIRED_MS: i64 = 120_000;

/// `successLines` (`sulap.js:21-26`).
const SUCCESS_LINES: [&str; 4] = [
    "💨 *POOF!* Dan... dia menghilang!",
    "🌟 Sulap berhasil! Sampai jumpa lagi~",
    "✨ Absen dulu ya, ditunggu berikutnya!",
    "🎪 Pertunjukan selesai! 👏",
];

fn session_key(chat: &str, admin: &str) -> String {
    format!("{SESSION_PREFIX}:{chat}:{admin}")
}

/// Prompt pertunjukan (`sulap.js:35-37`).
fn prompt() -> String {
    "🎩✨ *ᴘᴇʀᴛᴜɴᴊᴜᴋᴀɴ sᴜʟᴀᴘ*\n\n\
     Siapa yang ingin dihilangkan?\n\n\
     > Reply pesan ini + mention orangnya"
    .to_string()
}

/// Baris sukses acak — `successLines[Math.floor(Math.random() * len)]`.
fn success_line() -> &'static str {
    SUCCESS_LINES[random_index(SUCCESS_LINES.len())]
}

/// Pesan penutup setelah kick berhasil (`sulap.js:115-121`).
fn success_text(target_number: &str, sender_number: &str) -> String {
    format!(
        "{}\n\n🎯 @{target_number} telah menghilang!\n🎩 Pesulap: @{sender_number}\n\n> _Pertunjukan selesai~_ ✨",
        success_line()
    )
}

#[plugin(
    name = "sulap",
    alias = ["magic", "magictrick"],
    category = "fun",
    description = "Pertunjukan sulap - kick member secara dramatis",
    usage = ".sulap",
    example = ".sulap",
    cooldown = 60,
    energi = 0,
    is_group = true,
    is_private = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn sulap(ctx: Ctx) -> Reply {
    ctx.session_set(
        &session_key(&ctx.chat, &ctx.sender),
        json!({
            "admin": ctx.sender,
            "chat": ctx.chat,
            "timestamp": now_ms(),
        }),
    );
    // `await m.react('🎩')` lalu `await m.reply(prompt)`.
    Reply::text(prompt()).with_reaction("🎩")
}

/// Padanan `replyHandler` (`sulap.js:50-128`) — dijalankan untuk pesan
/// non-command yang me-quote prompt sulap.
#[oktz_plugin_api::answer_handler]
pub fn sulap_reply(ctx: Ctx) -> Option<Reply> {
    let quoted = ctx.quoted()?;

    let key = session_key(&ctx.chat, &ctx.sender);
    let session = ctx.session_get(&key)?;

    // Kedaluwarsa 120 detik — padanan `setTimeout(..., 120000)` yang menghapus
    // sesi; di sini dicek malas lalu sesi dibuang.
    let timestamp = session.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0);
    if now_ms() - timestamp >= EXPIRED_MS {
        ctx.session_take(&key);
        return None;
    }

    if session.get("chat").and_then(|v| v.as_str()) != Some(ctx.chat.as_str()) {
        return None;
    }
    if session.get("admin").and_then(|v| v.as_str()) != Some(ctx.sender.as_str()) {
        return None;
    }

    let target_jid = match ctx.mentioned.first() {
        Some(jid) => jid.clone(),
        // JS: `m.quoted?.sender && m.quoted.sender !== sock.user?.id` → return
        // false. JID bot tidak tersedia, jadi cabang ini dipakai untuk semua
        // quoted yang punya sender (kasus umum: bukan bot).
        None if !quoted.sender.is_empty() => return None,
        None => {
            return Some(Reply::text("❌ Mention orangnya dong!"));
        }
    };

    ctx.session_take(&key);

    let target_number = jid_number(&target_jid).to_string();
    let sender_number = jid_number(&ctx.sender).to_string();

    if target_jid == ctx.sender {
        return Some(Reply::text("🎭 Tidak bisa menghilangkan diri sendiri!"));
    }

    // `groupMetadata.participants.find(...)` — roster grup pesan ini.
    let target = ctx
        .roster
        .participants
        .iter()
        .find(|p| p.jid == target_jid || p.jid.contains(&target_number));
    let Some(target) = target else {
        return Some(Reply::text("👻 Orang itu tidak ada di grup!"));
    };

    if target.is_admin {
        return Some(Reply::text("🛡️ Admin kebal terhadap sihir!"));
    }

    // `sock.groupParticipantsUpdate(m.chat, [targetJid], 'remove')`; kegagalan
    // jatuh ke `catch` JS: "😅 Sulapnya gagal...".
    if let Err(e) = ctx.sock().ubah_peserta(&ctx.chat, &[target_jid.clone()], AksiPeserta::Keluar) {
        return Some(Reply::text(format!(
            "😅 Sulapnya gagal...\n\n> {}",
            e.pesan
        )));
    }

    Some(Reply::sequence(vec![
        Reply::text_mention(
            format!("🪄 *Bersiaplah @{target_number}...* ✨"),
            target_jid.clone(),
        ),
        Reply::Text {
            text: success_text(&target_number, &sender_number),
            mentions: vec![target_jid, ctx.sender.clone()],
        },
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::{GroupRoster, Participant};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";
    const TARGET: &str = "628222@s.whatsapp.net";

    fn roster() -> GroupRoster {
        GroupRoster {
            participants: vec![
                Participant {
                    jid: ME.into(),
                    is_admin: true,
                    is_owner: false,
                    data: serde_json::Value::Null,
                },
                Participant {
                    jid: TARGET.into(),
                    is_admin: false,
                    is_owner: false,
                    data: serde_json::Value::Null,
                },
            ],
            is_bot_admin: true,
        }
    }

    /// Sock stub: kick selalu berhasil, atau gagal dengan pesan tetap.
    #[derive(Debug)]
    struct SockSulap {
        gagal: bool,
    }
    impl oktz_plugin_api::Sock for SockSulap {
        fn ada_whatsapp(
            &self,
            _: &[String],
        ) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn metadata_grup(&self, _: &str) -> Result<serde_json::Value, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn semua_grup(&self) -> Result<serde_json::Value, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn ubah_peserta(
            &self,
            _: &str,
            _: &[String],
            _: AksiPeserta,
        ) -> Result<(), oktz_plugin_api::SockError> {
            if self.gagal {
                Err(oktz_plugin_api::SockError::baru("not admin"))
            } else {
                Ok(())
            }
        }
        fn set_foto_profil(&self, _: &str, _: &[u8]) -> Result<(), oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn keluar_grup(&self, _: &str) -> Result<(), oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn kode_undangan(&self, _: &str) -> Result<String, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn info_undangan(&self, _: &str) -> Result<serde_json::Value, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn buat_grup(
            &self,
            _: &str,
            _: &[String],
        ) -> Result<serde_json::Value, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn tandai_dibaca(&self, _: &[serde_json::Value]) -> Result<(), oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn status_teks(&self, _: &str) -> Result<serde_json::Value, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<serde_json::Value>, oktz_plugin_api::SockError> {
            Err(oktz_plugin_api::SockError::baru("x"))
        }
    }

    fn ctx_prompt(sessions: oktz_plugin_api::SessionStore) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "sulap".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            roster: roster(),
            sessions,
            sock: Arc::new(SockSulap { gagal: false }),
            ..Default::default()
        }
    }

    fn reply_ctx(sessions: oktz_plugin_api::SessionStore, quoted_sender: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: String::new(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            mentioned: vec![TARGET.into()],
            quoted: Some(oktz_plugin_api::QuotedMessage {
                id: "PROMPTID".into(),
                sender: quoted_sender.into(),
                chat: "120363@g.us".into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: true,
                timestamp: 0,
            }),
            roster: roster(),
            sessions,
            sock: Arc::new(SockSulap { gagal: false }),
            ..Default::default()
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("sulap").unwrap();
        assert_eq!(p.meta.name, "sulap");
        assert_eq!(p.meta.alias, ["magic", "magictrick"]);
        assert_eq!(p.meta.category, "fun");
        assert_eq!(p.meta.description, "Pertunjukan sulap - kick member secara dramatis");
        assert_eq!(p.meta.usage, ".sulap");
        assert_eq!(p.meta.example, ".sulap");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && !p.meta.is_private);
        assert!(p.meta.is_admin && p.meta.is_bot_admin);
        assert!(p.meta.is_enabled);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        for a in ["magic", "magictrick"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "sulap");
        }
    }

    #[test]
    fn prompt_menyimpan_sesi_dan_mereaksi_topi() {
        let sessions = oktz_plugin_api::new_session_store();
        let r = sulap(ctx_prompt(sessions.clone()));
        match r.without_mutations() {
            Reply::Reaction { emoji, reply } => {
                assert_eq!(emoji, "🎩");
                assert_eq!(reply.text_body().unwrap(), prompt());
            }
            other => panic!("harus Reaction, dapat {other:?}"),
        }
        let sesi = sessions
            .lock()
            .unwrap()
            .get(&session_key("120363@g.us", ME))
            .cloned()
            .unwrap();
        assert_eq!(sesi["admin"], ME);
        assert_eq!(sesi["chat"], "120363@g.us");
        assert!(sesi["timestamp"].as_i64().unwrap() > 0);
    }

    #[test]
    fn tanpa_sesi_atau_bukan_quoted_alur_reply_menolak() {
        // Tidak ada sesi → None.
        let r = sulap_reply(reply_ctx(oktz_plugin_api::new_session_store(), "bot@x"));
        assert!(r.is_none());
        // Tidak me-reply apa pun → None.
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "bot@x");
        c.quoted = None;
        assert!(sulap_reply(c).is_none());
    }

    #[test]
    fn alur_lengkap_menghapus_sesi_lalu_kick_dengan_dua_pesan() {
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let r = sulap_reply(reply_ctx(sessions.clone(), "bot@x")).expect("harus menangani");
        // Sesi habis setelah dipakai.
        assert!(sessions.lock().unwrap().get(&session_key("120363@g.us", ME)).is_none());
        let Reply::Sequence(list) = r else {
            panic!("harus Sequence, dapat {r:?}");
        };
        match &list[0] {
            Reply::Text { text, mentions } => {
                assert_eq!(text, "🪄 *Bersiaplah @628222...* ✨");
                assert_eq!(mentions, &[TARGET.to_string()]);
            }
            other => panic!("pesan 1 harus Text, dapat {other:?}"),
        }
        match &list[1] {
            Reply::Text { text, mentions } => {
                let num = jid_number(ME);
                assert!(text.contains(format!("🎯 @628222 telah menghilang!").as_str()), "{text}");
                assert!(text.contains(format!("🎩 Pesulap: @{num}").as_str()), "{text}");
                assert!(text.ends_with("> _Pertunjukan selesai~_ ✨"), "{text}");
                assert_eq!(mentions, &[TARGET.to_string(), ME.to_string()]);
            }
            other => panic!("pesan 2 harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn baris_sukses_selalu_dari_daftar_js() {
        for _ in 0..200 {
            let text = success_text("628222", "628111");
            assert!(
                SUCCESS_LINES.iter().any(|l| text.starts_with(l)),
                "baris pembuka bukan dari successLines: {text}"
            );
        }
    }

    #[test]
    fn admin_dan_target_diri_sendiri_ditolak() {
        // Target = pengirim.
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "bot@x");
        c.mentioned = vec![ME.into()];
        assert_eq!(
            sulap_reply(c).unwrap().text_body().unwrap(),
            "🎭 Tidak bisa menghilangkan diri sendiri!"
        );

        // Target admin di roster.
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "bot@x");
        let mut roster = roster();
        roster.participants[1].is_admin = true;
        c.roster = roster;
        assert_eq!(
            sulap_reply(c).unwrap().text_body().unwrap(),
            "🛡️ Admin kebal terhadap sihir!"
        );
    }

    #[test]
    fn target_tidak_di_grup_ditolak() {
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "bot@x");
        c.mentioned = vec!["628999@s.whatsapp.net".into()];
        assert_eq!(
            sulap_reply(c).unwrap().text_body().unwrap(),
            "👻 Orang itu tidak ada di grup!"
        );
    }

    #[test]
    fn kick_gagal_jatuh_ke_pesan_catch_js() {
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "bot@x");
        c.sock = Arc::new(SockSulap { gagal: true });
        let text = sulap_reply(c).unwrap().text_body().unwrap().to_string();
        assert_eq!(text, "😅 Sulapnya gagal...\n\n> not admin");
    }

    #[test]
    fn tanpa_mention_tanpa_sender_quoted_meminta_mention() {
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "");
        c.mentioned = vec![];
        assert_eq!(
            sulap_reply(c).unwrap().text_body().unwrap(),
            "❌ Mention orangnya dong!"
        );

        // Quoted punya sender (orang lain) → JS `return false`.
        let sessions = oktz_plugin_api::new_session_store();
        sulap(ctx_prompt(sessions.clone()));
        let mut c = reply_ctx(sessions, "628333@s.whatsapp.net");
        c.mentioned = vec![];
        assert!(sulap_reply(c).is_none());
    }

    #[test]
    fn sesi_kedaluwarsa_dibuang_dan_alur_menolak() {
        let sessions = oktz_plugin_api::new_session_store();
        ctx_prompt(sessions.clone());
        sessions.lock().unwrap().insert(
            session_key("120363@g.us", ME),
            json!({ "admin": ME, "chat": "120363@g.us", "timestamp": now_ms() - EXPIRED_MS }),
        );
        let r = sulap_reply(reply_ctx(sessions.clone(), "bot@x"));
        assert!(r.is_none(), "sesi kadaluarsa harus ditolak");
        assert!(
            sessions.lock().unwrap().get(&session_key("120363@g.us", ME)).is_none(),
            "sesi kadaluarsa harus dihapus"
        );
    }

    #[test]
    fn sesi_admin_lain_tidak_mengganggu() {
        // Admin lain men-quote prompt milik ME → ditolak.
        let sessions = oktz_plugin_api::new_session_store();
        ctx_prompt(sessions.clone());
        let mut c = reply_ctx(sessions, "bot@x");
        c.sender = "628444@s.whatsapp.net".into();
        assert!(sulap_reply(c).is_none());
    }
}
