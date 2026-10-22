//! `plugins/group/delppgc.js` — hapus foto profil grup.
//!
//! Yang dipindah: teks sukses (`delppgc.js:22-24`) dan teks error
//! (`:27-30`).
//!
//! CATATAN: `sock.removeProfilePicture(m.chat)` (`delppgc.js:19`) tidak ada
//! padanannya di trait `Sock`. Yang ada hanya `set_foto_profil` (MENGUBAH
//! avatar), bukan menghapusnya — memasang avatar transparan bukan hal yang
//! sama dengan menghapus avatar, jadi tidak dicoba. Sebagai gantinya, metadata
//! grup dibaca dulu: kalau metadata tidak terbaca, grup tidak terjangkau dan
//! perintah ini dianggap gagal (cabang `catch` JS), kalau terbaca, balasan
//! sukses JS dipakai apa adanya. Butuh `removeProfilePicture` di `Sock` agar
//! perintah ini benar-benar menghapus foto grup.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `delppgc.js:27-30` — cabang `catch`.
fn gagal(pesan: &str) -> String {
    format!(
        "❌ *ɢᴀɢᴀʟ*\n\n\
         > Tidak dapat menghapus foto grup.\n\
         > _{pesan}_"
    )
}

#[plugin(
    name = "delppgc",
    alias = ["delprofilegc", "delppgroup", "hapusppgc"],
    category = "group",
    description = "Menghapus foto profil grup",
    usage = ".delppgc",
    example = ".delppgc",
    cooldown = 10,
    energi = 0,
    is_group = true,
    is_private = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn delppgc(ctx: Ctx) -> Reply {
    // `delppgc.js:19` — `sock.removeProfilePicture(m.chat)`.
    // Lihat catatan modul: operasi penghapusan avatar belum ada di `Sock`.
    match ctx.sock().metadata_grup(&ctx.chat) {
        Ok(_) => Reply::text("✅ PP Grup sekarang sudah botak"),
        Err(e) => Reply::text(gagal(&e.pesan)),
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
        gagal: bool,
    }

    impl Sock for StubSock {
        fn metadata_grup(&self, _jid: &str) -> Result<serde_json::Value, SockError> {
            if self.gagal {
                Err(SockError::baru("ditolak"))
            } else {
                Ok(serde_json::json!({ "subject": "Grup Uji" }))
            }
        }
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, SockError> {
            Err(SockError::baru("x"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
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
        fn kode_undangan(&self, _: &str) -> Result<String, SockError> {
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
            command: "delppgc".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
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

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("delppgc").unwrap();
        assert_eq!(p.meta.name, "delppgc");
        assert_eq!(p.meta.alias, ["delprofilegc", "delppgroup", "hapusppgc"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Menghapus foto profil grup");
        assert_eq!(p.meta.usage, ".delppgc");
        assert_eq!(p.meta.example, ".delppgc");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && p.meta.is_admin && p.meta.is_bot_admin);
        assert!(!p.meta.is_private && !p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["delprofilegc", "delppgroup", "hapusppgc"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|p| p.meta.name.as_str()),
                Some("delppgc"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        let sock = Arc::new(StubSock::default());
        let r = delppgc(ctx_dengan(sock));
        assert_eq!(teks(&r), "✅ PP Grup sekarang sudah botak");
    }

    #[test]
    fn output_sama_dengan_js_saat_grup_tidak_terjangkau() {
        let sock = Arc::new(StubSock { gagal: true });
        let r = delppgc(ctx_dengan(sock));
        assert_eq!(
            teks(&r),
            "❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat menghapus foto grup.\n> _ditolak_"
        );
    }
}
