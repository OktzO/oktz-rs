//! `plugins/group/close.js` — tutup grup (hanya admin yang bisa chat).
//!
//! Yang dipindah: cek status `announce` (`close.js:22`), teks validasi
//! (`:24-27`), dan teks sukses (`:35`).
//!
//! CATATAN: `sock.groupSettingUpdate(m.chat, 'announcement')` (`close.js:30`)
//! tidak ada padanannya di trait `Sock` — kredensial grup tidak bisa diubah
//! lewat API plugin. Jadi status `announce` hanya BACA (lewat
//! `sock.metadata_grup`), dan BALASAN SUKSES tetap sama persis dengan JS
//! walau sisi efektifnya belum dijalankan. Butuh `groupSettingUpdate` di
//! `Sock` (atau `Reply::GroupSetting`) agar perintah benar-benar menutup
//! grup.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

/// `close.js:24-27` — grup sudah tertutup.
const SUDAH_TERTUTUP: &str = "⚠️ *ᴠᴀʟɪᴅᴀsɪ ɢᴀɢᴀʟ*\n\
     \n\
     > Grup sudah dalam keadaan `tertutup`.\n\
     > Hanya admin yang bisa mengirim pesan.";

/// `close.js:48-51` — cabang `catch`.
fn gagal(pesan: &str) -> String {
    format!("❌ *ᴇʀʀᴏʀ*\n\n> Gagal menutup grup.\n> _{pesan}_")
}

/// `m.groupMetadata.announce` (`close.js:22`).
fn sudah_tertutup(ctx: &Ctx) -> Option<bool> {
    let meta = ctx.sock().metadata_grup(&ctx.chat).ok()?;
    Some(
        meta.get("announce")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    )
}

#[plugin(
    name = "close",
    alias = ["tutup", "closegroup", "tutupgroup"],
    category = "group",
    description = "Menutup grup agar hanya admin yang bisa chat",
    usage = ".close",
    example = ".close",
    cooldown = 10,
    energi = 0,
    is_group = true,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn close(ctx: Ctx) -> Reply {
    // `close.js:22` — kalau metadata tidak terbaca, JS tetap memakai
    // `groupMetadata` yang sudah menempel di pesan; di Rust sumbernya
    // `sock`, jadi kegagalan di sini sama dengan kegagalan `catch`.
    match sudah_tertutup(&ctx) {
        Some(true) => return Reply::text(SUDAH_TERTUTUP),
        None => {
            return Reply::text(gagal("metadata grup tidak terbaca"));
        }
        Some(false) => {}
    }

    // `close.js:30` — `sock.groupSettingUpdate(m.chat, 'announcement')`.
    // Lihat catatan modul: operasi ini belum ada di `Sock`.

    // `close.js:33-37` — `const senderNum = m.sender.split('@')[0]`.
    Reply::text_mention(
        format!("✅ @{} telah menutup grup ini", ctx.sender_number()),
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
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, SockError> {
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
    }

    fn ctx_dengan(sock: Arc<StubSock>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "close".into(),
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
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn mentions(r: &Reply) -> Vec<String> {
        match r.without_mutations() {
            Reply::Text { mentions, .. } => mentions.clone(),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("close").unwrap();
        assert_eq!(p.meta.name, "close");
        assert_eq!(p.meta.alias, ["tutup", "closegroup", "tutupgroup"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Menutup grup agar hanya admin yang bisa chat");
        assert_eq!(p.meta.usage, ".close");
        assert_eq!(p.meta.example, ".close");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && p.meta.is_admin && p.meta.is_bot_admin);
        assert!(!p.meta.is_private && !p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["tutup", "closegroup", "tutupgroup"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|p| p.meta.name.as_str()),
                Some("close"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js_saat_grup_sudah_tertutup() {
        let sock = Arc::new(StubSock {
            announce: Some(true),
            ..Default::default()
        });
        let r = close(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "⚠️ *ᴠᴀʟɪᴅᴀsɪ ɢᴀɢᴀʟ*\n\n\
             > Grup sudah dalam keadaan `tertutup`.\n\
             > Hanya admin yang bisa mengirim pesan."
        );
    }

    #[test]
    fn output_sama_dengan_js_saat_grup_masih_terbuka() {
        let sock = Arc::new(StubSock {
            announce: Some(false),
            ..Default::default()
        });
        let r = close(ctx_dengan(sock));
        assert_eq!(teks(&r), "✅ @628111 telah menutup grup ini");
        assert_eq!(mentions(&r), vec![ME.to_string()]);
    }

    #[test]
    fn output_sama_dengan_js_saat_metadata_gagal() {
        let sock = Arc::new(StubSock {
            gagal: true,
            ..Default::default()
        });
        let r = close(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "❌ *ᴇʀʀᴏʀ*\n\n> Gagal menutup grup.\n> _metadata grup tidak terbaca_"
        );
    }
}
