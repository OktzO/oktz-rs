//! `plugins/owner/block.js` — blokir nomor WhatsApp.
//!
//! Port penuh: `sock.updateBlockStatus(targetJid, 'block')` →
//! `ctx.sock().blokir(target, true)`, lalu reaksi 🚫 + kartu sukses yang
//! men-mention target. Kegagalan `sock` → template error (`te`).
//!
//! `ponytail:` guard JS "jangan blokir bot sendiri" butuh `sock.user?.id`
//! (JID bot) yang tidak ada di `Ctx` — guard tidak di-port, blokir nomor
//! bot sendiri tetap mungkin. Upgrade path: ekspos `bot_id` di `Ctx`.
//!
//! `unblock.rs` memakai `resolve_target` dari file ini.

use oktz_plugin_api::{Ctx, Reply, plugin};

pub(crate) const INVALID_NUMBER: &str = "❌ Nomor tidak valid.";

pub(crate) const USAGE: &str = "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
     > `.block 628xxx` — Blokir via nomor\n\
     > `.block` (reply pesan) — Blokir pengirim\n\
     > `.block @mention` — Blokir yang di-mention\n\
     > `.block` (di private chat) — Blokir user ini";

/// Hasil resolusi target, memetakan tiga cabang `else if` di JS.
pub(crate) enum Target {
    /// Punya JID — akan diblokir.
    Jid(String),
    /// `m.args[0]` ada tapi setelah filter digit jadi string kosong.
    InvalidNumber,
    /// Tidak ada cabang yang kena → tampilkan [`USAGE`].
    None,
}

/// `m.mentionedJid[0]` → `m.quoted.sender` → angka dari `m.args[0]` →
/// `m.chat` kalau private. Urutannya tidak boleh diacak: mention menang atas
/// quote, quote menang atas angka.
pub(crate) fn resolve_target(ctx: &Ctx) -> Target {
    if let Some(first) = ctx.mentioned.first() {
        return Target::Jid(first.clone());
    }
    if let Some(quoted) = ctx.quoted() {
        // JS: `m.quoted.sender || m.quoted.participant`. `QuotedMessage` hanya
        // punya `sender`; string kosong = falsy di JS, jadi hasil ""
        // sama dengan tidak kena dan jatuh ke guard `!targetJid`.
        if quoted.sender.is_empty() {
            return Target::None;
        }
        return Target::Jid(quoted.sender.clone());
    }
    if let Some(raw) = ctx.args.first() {
        let num: String = raw.chars().filter(char::is_ascii_digit).collect();
        if num.is_empty() {
            return Target::InvalidNumber;
        }
        return Target::Jid(format!("{num}@s.whatsapp.net"));
    }
    if !ctx.is_group {
        return Target::Jid(ctx.chat.clone());
    }
    Target::None
}

/// Kartu sukses JS (`block.js:44-49`): reaksi 🚫 + teks yang men-mention target.
pub(crate) fn sukses_blokir(number: &str) -> String {
    format!(
        "🚫 *ɴᴏᴍᴏʀ ᴅɪʙʟᴏᴋɪʀ*\n\n\
         > Target: @{number}\n\
         > Gunakan `.unblock` untuk membuka blokir"
    )
}

#[plugin(
    name = "block",
    alias = ["blokir"],
    category = "owner",
    description = "Blokir nomor WhatsApp",
    usage = ".block <nomor/reply/mention>",
    example = ".block 628xxx",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn block(ctx: Ctx) -> Reply {
    let target = match resolve_target(&ctx) {
        Target::Jid(jid) => jid,
        Target::InvalidNumber => return Reply::text(INVALID_NUMBER),
        Target::None => return Reply::text(USAGE),
    };
    let number = target.split('@').next().unwrap_or(&target);
    match ctx.sock().blokir(&target, true) {
        Ok(()) => Reply::text_mention(sukses_blokir(number), target).with_reaction("🚫"),
        Err(_) => Reply::text(crate::common::error_template(&ctx)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        AdaWhatsapp, AksiPeserta, PluginEntry, QuotedKind, QuotedMessage, Sock, SockError, find,
    };
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub sock: mencatat (jid, blokir) yang dikirim.
    #[derive(Debug, Default)]
    struct SockUji {
        blok: Mutex<Vec<(String, bool)>>,
        gagal: bool,
    }

    impl Sock for SockUji {
        fn blokir(&self, jid: &str, blokir: bool) -> Result<(), SockError> {
            if self.gagal {
                return Err(SockError::baru("IQ gagal: timeout"));
            }
            self.blok.lock().unwrap().push((jid.to_string(), blokir));
            Ok(())
        }

        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn metadata_grup(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn semua_grup(&self) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn ubah_peserta(&self, _: &str, _: &[String], _: AksiPeserta) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn set_foto_profil(&self, _: &str, _: &[u8]) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn keluar_grup(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn kode_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn info_undangan(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn buat_grup(&self, _: &str, _: &[String]) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn tandai_dibaca(&self, _: &[Value]) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn status_teks(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<Value>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn set_bio(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn set_nama(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn hapus_foto_bot(&self) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    fn quote_of(sender: &str) -> QuotedMessage {
        QuotedMessage {
            id: "Q1".into(),
            sender: sender.into(),
            chat: "628111@s.whatsapp.net".into(),
            text: String::new(),
            kind: QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        }
    }

    struct Spec {
        prefix: &'static str,
        is_group: bool,
    }

    fn ctx(spec: Spec, args: &[&str], mentioned: &[&str], quoted: Option<QuotedMessage>) -> Ctx {
        ctx_with(spec, args, mentioned, quoted, Arc::new(SockUji::default()))
    }

    fn ctx_with(
        spec: Spec,
        args: &[&str],
        mentioned: &[&str],
        quoted: Option<QuotedMessage>,
        sock: Arc<dyn Sock>,
    ) -> Ctx {
        Ctx {
            prefix: spec.prefix.into(),
            command: "block".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            is_group: spec.is_group,
            args: args.iter().map(|s| (*s).to_string()).collect(),
            mentioned: mentioned.iter().map(|s| (*s).to_string()).collect(),
            quoted,
            sock,
            ..Default::default()
        }
    }

    /// `find("block")` tidak bisa dipakai di test: `ban.js` juga mendaftarkan
    /// `block` sebagai alias (`plugins/owner/ban.js:6`), jadi lookup itu
    /// mengembalikan siapa saja yang menang di distributed slice. `blokir`
    /// milik plugin ini saja.
    fn entry() -> &'static PluginEntry {
        find("blokir").expect("block harus terdaftar lewat alias blokir")
    }

    fn run(c: Ctx) -> String {
        (entry().handler)(c)
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    const PC: Spec = Spec { prefix: ".", is_group: false };
    const GC: Spec = Spec { prefix: ".", is_group: true };

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = entry();
        assert_eq!(p.meta.name, "block");
        assert_eq!(p.meta.alias, ["blokir"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn a_number_is_stripped_to_digits_and_gets_the_wa_server() {
        let t = resolve_target(&ctx(GC, &["+62 (8) 811-1"], &[], None));
        assert!(matches!(t, Target::Jid(j) if j == "6288111@s.whatsapp.net"));
    }

    #[test]
    fn a_number_with_no_digits_at_all_is_rejected() {
        assert!(matches!(resolve_target(&ctx(GC, &["abc"], &[], None)), Target::InvalidNumber));
        assert_eq!(run(ctx(GC, &["abc"], &[], None)), INVALID_NUMBER);
    }

    #[test]
    fn a_bare_command_in_private_falls_back_to_the_chat_itself() {
        assert!(matches!(resolve_target(&ctx(PC, &[], &[], None)), Target::Jid(_)));
    }

    #[test]
    fn a_bare_command_in_a_group_has_no_target_and_shows_the_usage() {
        assert_eq!(run(ctx(GC, &[], &[], None)), USAGE);
    }

    #[test]
    fn blocking_reaches_the_sock_with_true() {
        let sock = Arc::new(SockUji::default());
        run(ctx_with(GC, &["628777"], &[], None, sock.clone()));
        assert_eq!(
            *sock.blok.lock().unwrap(),
            vec![("628777@s.whatsapp.net".to_string(), true)]
        );
    }

    #[test]
    fn success_text_mentions_the_target_and_reacts() {
        let r = (entry().handler)(ctx(GC, &["628777"], &[], None));
        assert_eq!(
            r.without_mutations().text_body(),
            Some(sukses_blokir("628777").as_str())
        );
        // JS `{ mentions: [targetJid] }` — target di-tag supaya muncul di notif.
        match r.without_mutations() {
            oktz_plugin_api::Reply::Reaction { reply, .. } => match reply.as_ref() {
                oktz_plugin_api::Reply::Text { mentions, .. } => {
                    assert_eq!(mentions, &["628777@s.whatsapp.net"]);
                }
                other => panic!("harus Text di dalam Reaction, dapat {other:?}"),
            },
            other => panic!("harus dibungkus reaksi, dapat {other:?}"),
        }
        match &r {
            oktz_plugin_api::Reply::Mutate { reply, .. } => {
                assert!(matches!(
                    reply.as_ref(),
                    oktz_plugin_api::Reply::Reaction { emoji, .. } if emoji == "🚫"
                ));
            }
            other => panic!("harus dibungkus reaksi 🚫, dapat {other:?}"),
        }
    }

    #[test]
    fn sock_failure_falls_back_to_the_error_template() {
        let sock = Arc::new(SockUji { blok: Mutex::new(Vec::new()), gagal: true });
        let t = run(ctx_with(GC, &["628777"], &[], None, sock));
        assert!(t.contains("kendala"), "harus template error: {t}");
    }
}
