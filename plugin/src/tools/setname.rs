//! `plugins/tools/setname.js` — mengubah nama profil bot.
//!
//! Port penuh: `sock.updateProfileName(newName)` → `ctx.sock().set_nama(nama)`.
//! Validasi 1-25 karakter (`String.length` JS = unit UTF-16).

use oktz_plugin_api::{Ctx, Reply, plugin};

const MIN: usize = 1;
const MAX: usize = 25;

fn usage(prefix: &str) -> String {
    format!("⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{prefix}setname Nama Bot Baru`")
}

fn out_of_range() -> &'static str {
    "⚠️ *ᴠᴀʟɪᴅᴀsɪ*\n\n> Nama bot harus 1-25 karakter."
}

fn sukses(nama: &str) -> String {
    format!("✅ *ɴᴀᴍᴀ ʙᴏᴛ ᴅɪᴜʙᴀʜ*\n\n> Nama bot sekarang: *{nama}*")
}

fn gagal(e: &str) -> String {
    format!("❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat mengubah nama bot.\n> _{e}_")
}

#[plugin(
    name = "setname",
    alias = ["setnamebot", "setbotnama"],
    category = "tools",
    description = "Mengubah nama profil bot",
    usage = ".setname <nama baru>",
    example = ".setname Foto-AI",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true
)]
pub fn setname(ctx: Ctx) -> Reply {
    let new_name = ctx.text.trim();
    if new_name.is_empty() {
        return Reply::text(usage(&ctx.prefix));
    }
    let len = new_name.encode_utf16().count();
    if !(MIN..=MAX).contains(&len) {
        return Reply::text(out_of_range());
    }
    match ctx.sock().set_nama(new_name) {
        Ok(()) => Reply::text(sukses(new_name)),
        Err(e) => Reply::text(gagal(&e.pesan)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{AdaWhatsapp, AksiPeserta, Ctx, Sock, SockError};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub sock: `gagal = true` → set_nama selalu gagal; selain itu mencatat.
    #[derive(Debug, Default)]
    struct SockUji {
        nama: Mutex<Vec<String>>,
        gagal: bool,
    }

    impl Sock for SockUji {
        fn set_nama(&self, nama: &str) -> Result<(), SockError> {
            if self.gagal {
                return Err(SockError::baru("IQ gagal: timeout"));
            }
            self.nama.lock().unwrap().push(nama.to_string());
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
        fn hapus_foto_bot(&self) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn blokir(&self, _: &str, _: bool) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    fn run_with(sock: Arc<dyn Sock>, text: &str) -> String {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "setname".into(),
            sender: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sock,
            ..Default::default()
        };
        (oktz_plugin_api::find("setname").unwrap().handler)(ctx)
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn run(text: &str) -> String {
        run_with(Arc::new(SockUji::default()), text)
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("setname").unwrap();
        assert_eq!(p.meta.name, "setname");
        assert_eq!(p.meta.alias, ["setnamebot", "setbotnama"]);
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn a_bare_command_shows_the_usage_card() {
        for input in ["", "   "] {
            assert_eq!(run(input), usage("."));
        }
    }

    #[test]
    fn the_length_window_is_one_to_twenty_five() {
        assert_eq!(run(&"a".repeat(MAX)), sukses(&"a".repeat(MAX)), "tepat 25 masih sah");
        assert_eq!(run(&"a".repeat(MAX + 1)), out_of_range());
        // `""` sudah ditangkap kartu usage lebih dulu — `length < 1` di JS
        // tidak bisa terjadi karena `!newName` sudah keluar duluan.
        assert_eq!(run(&"a".repeat(0)), usage("."));
    }

    #[test]
    fn the_length_window_counts_utf16_units() {
        // U+1F642 = 2 unit: 12 emoji = 24 (sah), 13 = 26 (tolak).
        assert_eq!(run(&"\u{1F642}".repeat(12)), sukses(&"\u{1F642}".repeat(12)));
        assert_eq!(run(&"\u{1F642}".repeat(13)), out_of_range());
    }

    #[test]
    fn a_valid_name_reaches_the_sock() {
        let sock = Arc::new(SockUji::default());
        run_with(sock.clone(), "Foto-AI");
        assert_eq!(*sock.nama.lock().unwrap(), vec!["Foto-AI".to_string()]);
    }

    #[test]
    fn sock_failure_carries_the_error_message() {
        let t = run_with(
            Arc::new(SockUji { nama: Mutex::new(Vec::new()), gagal: true }),
            "Foto-AI",
        );
        assert_eq!(t, gagal("IQ gagal: timeout"));
    }
}
