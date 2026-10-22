//! `plugins/group/linkgc.js` — kirim link invite grup.
//!
//! Yang dipindah: seluruh alur — ambil kode invite, bentuk URL, balas, dan
//! reaksi penutup (`linkgc.js:21-31`).
//!
//! Kode invite diambil lewat `sock.groupInviteCode(m.chat)` yang di Rust
//! sudah ada sebagai [`oktz_plugin_api::Sock::kode_undangan`], jadi plugin
//! ini bekerja penuh — tidak ada penyimpangan.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `linkgc.js:27` — link invite.
fn link(kode: &str) -> String {
    format!("https://chat.whatsapp.com/{kode}")
}

#[plugin(
    name = "linkgc",
    alias = ["linkgrup", "getlink", "gclink"],
    category = "group",
    description = "Dapatkan link invite grup",
    usage = ".linkgc",
    example = ".linkgc",
    cooldown = 10,
    energi = 0,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = true,
    is_private = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn linkgc(ctx: Ctx) -> Reply {
    // `linkgc.js:21` — `m.react('🕕')`; reaksi terakhir di WA menggantikan
    // reaksi sebelumnya, jadi hanya reaksi penutup yang dikirim.
    match ctx.sock().kode_undangan(&ctx.chat) {
        Ok(kode) => {
            // `linkgc.js:26` — `Link grup grup ini\n${urlGrup}`.
            Reply::text(format!("Link grup grup ini\n{}", link(&kode))).with_reaction("✅")
        }
        // `linkgc.js:29-31` — `m.react('☢')` + `te(...)`.
        Err(_) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::GroupRoster;
    use oktz_plugin_api::{Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        kode: Option<String>,
    }

    impl Sock for StubSock {
        fn kode_undangan(&self, _jid: &str) -> Result<String, SockError> {
            match &self.kode {
                Some(k) => Ok(k.clone()),
                None => Err(SockError::baru("tidak ada kode")),
            }
        }
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, SockError> {
            Err(SockError::baru("x"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("x"))
        }
        fn metadata_grup(&self, _: &str) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn semua_grup(&self) -> Result<serde_json::Value, SockError> {
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
        fn info_undangan(&self, _: &str) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn buat_grup(&self, _: &str, _: &[String]) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn tandai_dibaca(&self, _: &[serde_json::Value]) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn status_teks(&self, _: &str) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<serde_json::Value>, SockError> {
            Err(SockError::baru("x"))
        }
    }

    fn ctx_dengan(sock: Arc<StubSock>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "linkgc".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            push_name: "Budi".into(),
            is_group: true,
            sock,
            roster: GroupRoster {
                participants: vec![],
                is_bot_admin: true,
            },
            ..Default::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let e = oktz_plugin_api::find("linkgc").unwrap();
        assert_eq!(e.meta.name, "linkgc");
        assert_eq!(e.meta.alias, ["linkgrup", "getlink", "gclink"]);
        assert_eq!(e.meta.category, "group");
        assert_eq!(e.meta.description, "Dapatkan link invite grup");
        assert_eq!(e.meta.usage, ".linkgc");
        assert_eq!(e.meta.example, ".linkgc");
        assert_eq!(e.meta.cooldown, 10);
        assert_eq!(e.meta.energi, 0);
        assert!(e.meta.is_enabled);
        assert!(e.meta.is_group && e.meta.is_admin && e.meta.is_bot_admin);
        assert!(!e.meta.is_private && !e.meta.is_owner && !e.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["linkgrup", "getlink", "gclink"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|e| e.meta.name.as_str()),
                Some("linkgc"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js_dengan_kode_invite() {
        let sock = Arc::new(StubSock {
            kode: Some("ABC123".into()),
        });
        let r = linkgc(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "Link grup grup ini\nhttps://chat.whatsapp.com/ABC123"
        );
        assert_eq!(reaksi(&r), Some("✅".to_string()));
    }

    #[test]
    fn output_sama_dengan_js_saat_gagal() {
        let sock = Arc::new(StubSock { kode: None });
        let r = linkgc(ctx_dengan(sock));
        assert!(
            teks(&r).contains("Silahkan coba lagi nanti, Budi"),
            "{:?}",
            teks(&r)
        );
        assert_eq!(reaksi(&r), Some("☢".to_string()));
    }
}
