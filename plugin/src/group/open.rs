//! Buka grup (semua member bisa chat).

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

/// Grup sudah terbuka.
const SUDAH_TERBUKA: &str = "⚠️ *ᴠᴀʟɪᴅᴀsɪ ɢᴀɢᴀʟ*\n\n\
     > Grup sudah dalam keadaan `terbuka`.\n\
     > Semua member sudah bisa mengirim pesan.";

/// Cabang `catch`.
fn gagal(pesan: &str) -> String {
    format!("❌ *ᴇʀʀᴏʀ*\n\n> Gagal membuka grup.\n> _{pesan}_")
}

/// `m.groupMetadata.announce`.
fn sudah_terbuka(ctx: &Ctx) -> Option<bool> {
    let meta = ctx.sock().metadata_grup(&ctx.chat).ok()?;
    Some(
        meta.get("announce")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    )
}

#[plugin(
    name = "open",
    alias = ["buka", "opengroup", "bukagrup"],
    category = "group",
    description = "Membuka grup agar semua member bisa chat",
    usage = ".open",
    example = ".open",
    cooldown = 10,
    energi = 0,
    is_group = true,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn open(ctx: Ctx) -> Reply {
    match sudah_terbuka(&ctx) {
        Some(false) => return Reply::text(SUDAH_TERBUKA),
        None => {
            return Reply::text(gagal("metadata grup tidak terbaca"));
        }
        Some(true) => {}
    }

    // `sock.groupSettingUpdate(m.chat, 'not_announcement')`.
    // Lihat catatan modul: operasi ini belum ada di `Sock`.

    // `const senderNum = m.sender.split('@')[0]`.
    Reply::text_mention(
        format!(
            "✅ @{} telah membuka grup ini\n_Sekarang kalian bisa mengirim pesan_",
            ctx.sender_number()
        ),
        ctx.sender.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::GroupRoster;
    use oktz_plugin_api::{Participant, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        announce: Option<bool>,
        gagal: bool,
    }

    impl Sock for StubSock {
        fn metadata_grup(&self, _jid: &str) -> Result<Value, SockError> {
            if self.gagal {
                return Err(SockError::baru("tidak terhubung"));
            }
            Ok(serde_json::json!({
                "announce": self.announce.unwrap_or(false),
                "subject": "Grup Uji",
            }))
        }
        fn ada_whatsapp(
            &self,
            _: &[String],
        ) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, SockError> {
            Err(SockError::baru("x"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("x"))
        }
        fn semua_grup(&self) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn ubah_peserta(
            &self,
            _: &str,
            _: &[String],
            _: oktz_plugin_api::AksiPeserta,
        ) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn set_foto_profil(&self, _: &str, _: &[u8]) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn keluar_grup(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn kode_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn info_undangan(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn buat_grup(&self, _: &str, _: &[String]) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn tandai_dibaca(&self, _: &[Value]) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn status_teks(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<Value>, SockError> {
            Err(SockError::baru("x"))
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
        fn blokir(&self, _: &str, _: bool) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    fn ctx_dengan(sock: Arc<StubSock>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "open".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            sock,
            roster: GroupRoster {
                participants: vec![Participant {
                    jid: ME.into(),
                    is_admin: true,
                    is_owner: true,
                    data: Default::default(),
                }],
                is_bot_admin: true,
            },
            ..Default::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn mentions(r: &Reply) -> Vec<String> {
        match r.without_mutations() {
            Reply::Text { mentions, .. } => mentions.clone(),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("open").unwrap();
        assert_eq!(p.meta.name, "open");
        assert_eq!(p.meta.alias, ["buka", "opengroup", "bukagrup"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(
            p.meta.description,
            "Membuka grup agar semua member bisa chat"
        );
        assert_eq!(p.meta.usage, ".open");
        assert_eq!(p.meta.example, ".open");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && p.meta.is_admin && p.meta.is_bot_admin);
        assert!(!p.meta.is_private && !p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["buka", "opengroup", "bukagrup"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|p| p.meta.name.as_str()),
                Some("open"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js_saat_grup_sudah_terbuka() {
        let sock = Arc::new(StubSock {
            announce: Some(false),
            ..Default::default()
        });
        let r = open(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "⚠️ *ᴠᴀʟɪᴅᴀsɪ ɢᴀɢᴀʟ*\n\n\
             > Grup sudah dalam keadaan `terbuka`.\n\
             > Semua member sudah bisa mengirim pesan."
        );
    }

    #[test]
    fn output_sama_dengan_js_saat_grup_masih_tertutup() {
        let sock = Arc::new(StubSock {
            announce: Some(true),
            ..Default::default()
        });
        let r = open(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "✅ @628111 telah membuka grup ini\n_Sekarang kalian bisa mengirim pesan_"
        );
        assert_eq!(mentions(&r), vec![ME.to_string()]);
    }

    #[test]
    fn output_sama_dengan_js_saat_metadata_gagal() {
        let sock = Arc::new(StubSock {
            gagal: true,
            ..Default::default()
        });
        let r = open(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "❌ *ᴇʀʀᴏʀ*\n\n> Gagal membuka grup.\n> _metadata grup tidak terbaca_"
        );
    }
}
