//! `plugins/owner/unblock.js` — buka blokir nomor WhatsApp.
//!
//! Port penuh: `sock.updateBlockStatus(targetJid, 'unblock')` →
//! `ctx.sock().blokir(target, false)`, lalu reaksi ✅ + kartu sukses.
//! `unblock.js` TIDAK punya guard "jangan buka blokir bot sendiri" seperti
//! `block.js`, jadi file ini tidak menambahkannya.
//!
//! Resolusi target diambil dari `block.rs` (urutan mention → quote → angka →
//! chat private harus identik di kedua plugin).

use oktz_plugin_api::{Ctx, Reply, plugin};

use crate::owner::block::{INVALID_NUMBER, Target, resolve_target};

pub(crate) const USAGE: &str = "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
     > `.unblock 628xxx` — Unblock via nomor\n\
     > `.unblock` (reply pesan) — Unblock pengirim\n\
     > `.unblock @mention` — Unblock yang di-mention\n\
     > `.unblock` (di private chat) — Unblock user ini";

fn sukses_unblock(number: &str) -> String {
    format!("✅ *ɴᴏᴍᴏʀ ᴅɪ-ᴜɴʙʟᴏᴄᴋ*\n\n> Target: @{number}")
}

#[plugin(
    name = "unblock",
    alias = ["unblocknomor"],
    category = "owner",
    description = "Buka blokir nomor WhatsApp",
    usage = ".unblock <nomor/reply/mention>",
    example = ".unblock 628xxx",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn unblock(ctx: Ctx) -> Reply {
    let target = match resolve_target(&ctx) {
        Target::Jid(jid) => jid,
        Target::InvalidNumber => return Reply::text(INVALID_NUMBER),
        Target::None => return Reply::text(USAGE),
    };
    let number = target.split('@').next().unwrap_or(&target);
    match ctx.sock().blokir(&target, false) {
        Ok(()) => Reply::text_mention(sukses_unblock(number), target).with_reaction("✅"),
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

    fn ctx_with(
        spec: Spec,
        args: &[&str],
        mentioned: &[&str],
        quoted: Option<QuotedMessage>,
        sock: Arc<dyn Sock>,
    ) -> Ctx {
        Ctx {
            prefix: spec.prefix.into(),
            command: "unblock".into(),
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

    /// `find("unblock")` tidak bisa dipakai di test: `unban.js` juga
    /// mendaftarkan `unblock` sebagai alias (`plugins/owner/unban.js:6`).
    /// `unblocknomor` milik plugin ini saja.
    fn entry() -> &'static PluginEntry {
        find("unblocknomor").expect("unblock harus terdaftar lewat alias unblocknomor")
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
        assert_eq!(p.meta.name, "unblock");
        assert_eq!(p.meta.alias, ["unblocknomor"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn the_second_name_in_the_js_array_resolves_as_an_alias() {
        assert_eq!(find("unblocknomor").unwrap().meta.name, "unblock");
    }

    /// Tabrakan warisan JS: `unban.js` mendaftarkan `unblock` sebagai alias,
    /// `unblock.js` mendaftarkannya sebagai nama. `find("unblock")` mengembalikan
    /// salah satu dari keduanya. Tidak dirapikan diam-diam; lihat laporan batch.
    #[test]
    fn the_unblock_command_is_shared_with_unban_just_like_in_javascript() {
        let owner = find("unblock").unwrap().meta.name.clone();
        assert!(
            owner == "unblock" || owner == "unban",
            "pemilik `unblock` harus salah satu dari dua plugin JS: {owner}"
        );
    }

    #[test]
    fn the_usage_text_is_the_unblock_wording_not_the_block_one() {
        assert!(USAGE.contains(".unblock 628xxx"));
        assert!(!USAGE.contains(".block"), "menyalin usage blokir akan boda user: {USAGE}");
        assert_ne!(USAGE, crate::owner::block::USAGE);
    }

    #[test]
    fn a_number_with_no_digits_is_rejected() {
        assert_eq!(run(ctx_with(GC, &["abc"], &[], None, Arc::new(SockUji::default()))), INVALID_NUMBER);
    }

    #[test]
    fn a_bare_command_in_a_group_shows_the_usage() {
        assert_eq!(run(ctx_with(GC, &[], &[], None, Arc::new(SockUji::default()))), USAGE);
    }

    #[test]
    fn unblocking_reaches_the_sock_with_false() {
        let sock = Arc::new(SockUji::default());
        run(ctx_with(GC, &["628777"], &[], None, sock.clone()));
        assert_eq!(
            *sock.blok.lock().unwrap(),
            vec![("628777@s.whatsapp.net".to_string(), false)]
        );
    }

    #[test]
    fn success_text_mentions_the_target_and_reacts() {
        let r = (entry().handler)(ctx_with(GC, &["628777"], &[], None, Arc::new(SockUji::default())));
        assert_eq!(
            r.without_mutations().text_body(),
            Some(sukses_unblock("628777").as_str())
        );
        match r.without_mutations() {
            oktz_plugin_api::Reply::Reaction { reply, .. } => match reply.as_ref() {
                oktz_plugin_api::Reply::Text { mentions, .. } => {
                    assert_eq!(mentions, &["628777@s.whatsapp.net"]);
                }
                other => panic!("harus Text di dalam Reaction, dapat {other:?}"),
            },
            other => panic!("harus dibungkus reaksi, dapat {other:?}"),
        }
    }

    #[test]
    fn sock_failure_falls_back_to_the_error_template() {
        let sock = Arc::new(SockUji { blok: Mutex::new(Vec::new()), gagal: true });
        let t = run(ctx_with(GC, &["628777"], &[], None, sock));
        assert!(t.contains("kendala"), "harus template error: {t}");
    }

    #[test]
    fn a_quote_target_is_resolved_before_the_number() {
        let t = resolve_target(&ctx_with(
            GC,
            &["628777"],
            &[],
            Some(quote_of("628888@s.whatsapp.net")),
            Arc::new(SockUji::default()),
        ));
        assert!(matches!(t, Target::Jid(j) if j == "628888@s.whatsapp.net"));
    }

    #[test]
    fn private_chat_without_args_targets_the_chat_itself() {
        let t = resolve_target(&ctx_with(PC, &[], &[], None, Arc::new(SockUji::default())));
        assert!(matches!(t, Target::Jid(_)));
    }
}
