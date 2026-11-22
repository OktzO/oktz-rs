//! `plugins/tools/delpp.js` — menghapus foto profil bot.
//!
//! Port penuh: `sock.removeProfilePicture(botJid)` → `ctx.sock().hapus_foto_bot()`.
//! Handler JS tidak punya guard; `sock.user?.id` tidak ada di `Ctx`, tapi
//! `hapus_foto_bot` di `SockKlien` sudah murni aksi profil bot sendiri.

use oktz_plugin_api::{Ctx, Reply, plugin};

fn sukses() -> &'static str {
    "✅ *ᴘᴘ ʙᴏᴛ ᴅɪʜᴀᴘᴜs*\n\n> Foto profil bot berhasil dihapus!"
}

fn gagal(e: &str) -> String {
    format!("❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat menghapus foto bot.\n> _{e}_")
}

#[plugin(
    name = "delpp",
    alias = ["delprofilebot", "delppbot", "hapusppbot"],
    category = "tools",
    description = "Menghapus foto profil bot",
    usage = ".delpp",
    example = ".delpp",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true
)]
pub fn delpp(ctx: Ctx) -> Reply {
    match ctx.sock().hapus_foto_bot() {
        Ok(()) => Reply::text(sukses()),
        Err(e) => Reply::text(gagal(&e.pesan)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{AdaWhatsapp, AksiPeserta, Ctx, Sock, SockError};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub sock: `gagal = true` → hapus selalu gagal; selain itu mencatat.
    #[derive(Debug, Default)]
    struct SockUji {
        dihapus: Mutex<Vec<()>>,
        gagal: bool,
    }

    impl Sock for SockUji {
        fn hapus_foto_bot(&self) -> Result<(), SockError> {
            if self.gagal {
                return Err(SockError::baru("IQ gagal: timeout"));
            }
            self.dihapus.lock().unwrap().push(());
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
        fn blokir(&self, _: &str, _: bool) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    fn run_with(sock: Arc<dyn Sock>) -> String {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "delpp".into(),
            sender: "628111@s.whatsapp.net".into(),
            sock,
            ..Default::default()
        };
        (oktz_plugin_api::find("delpp").unwrap().handler)(ctx)
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("delpp").unwrap();
        assert_eq!(p.meta.name, "delpp");
        assert_eq!(p.meta.alias, ["delprofilebot", "delppbot", "hapusppbot"]);
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn success_reports_the_deletion() {
        assert_eq!(run_with(Arc::new(SockUji::default())), sukses());
    }

    #[test]
    fn sock_failure_carries_the_error_message() {
        let sock = Arc::new(SockUji { dihapus: Mutex::new(Vec::new()), gagal: true });
        assert_eq!(run_with(sock), gagal("IQ gagal: timeout"));
    }

    #[test]
    fn the_deletion_reaches_the_sock() {
        let sock = Arc::new(SockUji::default());
        run_with(sock.clone());
        assert_eq!(sock.dihapus.lock().unwrap().len(), 1);
    }
}
