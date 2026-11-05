//! `plugins/tools/setbio.js` — mengubah bio/status bot.
//!
//! Port penuh: `sock.updateProfileStatus(bioToSet)` → `ctx.sock().set_bio(bio)`.
//! `clear` mengosongkan bio. Balasan sukses mengikuti JS apa adanya; kegagalan
//! `sock` jadi kartu ❌ yang membawa pesan error-nya.
//!
//! Panjang dihitung dengan `encode_utf16().count()` karena `String.length` di JS
//! menghitung unit UTF-16, bukan karakter.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `bioToSet.length > 139` (setbio.js:31).
const MAX: usize = 139;

fn usage(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}setbio Bio bot baru`\n\
         > `{prefix}setbio clear` - Hapus bio"
    )
}

fn too_long() -> &'static str {
    "⚠️ *ᴠᴀʟɪᴅᴀsɪ*\n\n> Bio maksimal 139 karakter."
}

fn sukses_ubah(bio: &str) -> String {
    format!("✅ *ʙɪᴏ ʙᴏᴛ ᴅɪᴜʙᴀʜ*\n\n> Bio bot sekarang:\n> _{bio}_")
}

fn sukses_hapus() -> &'static str {
    "✅ *ʙɪᴏ ʙᴏᴛ ᴅɪʜᴀᴘᴜs*\n\n> Bio bot berhasil dihapus!"
}

fn gagal(e: &str) -> String {
    format!("❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat mengubah bio bot.\n> _{e}_")
}

/// `newBio?.toLowerCase() === 'clear' ? '' : (newBio || '')` (setbio.js:29).
fn bio_to_set(new_bio: &str) -> &str {
    if new_bio.eq_ignore_ascii_case("clear") {
        ""
    } else {
        new_bio
    }
}

#[plugin(
    name = "setbio",
    alias = ["setbiobot", "setstatus", "setabout"],
    category = "tools",
    description = "Mengubah bio/status bot",
    usage = ".setbio <bio baru>",
    example = ".setbio Bot WhatsApp by Lucky Archz",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true
)]
pub fn setbio(ctx: Ctx) -> Reply {
    let new_bio = ctx.text.trim();
    if new_bio.is_empty() && ctx.args.is_empty() {
        return Reply::text(usage(&ctx.prefix));
    }

    let bio = bio_to_set(new_bio);
    if bio.encode_utf16().count() > MAX {
        return Reply::text(too_long());
    }

    match ctx.sock().set_bio(bio) {
        Ok(()) => {
            if bio.is_empty() {
                Reply::text(sukses_hapus())
            } else {
                Reply::text(sukses_ubah(bio))
            }
        }
        Err(e) => Reply::text(gagal(&e.pesan)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{AdaWhatsapp, AksiPeserta, Ctx, Sock, SockError};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// Stub sock: `gagal = true` → set_bio selalu gagal; selain itu mencatat.
    #[derive(Debug, Default)]
    struct SockUji {
        bio: Mutex<Vec<String>>,
        gagal: bool,
    }

    impl Sock for SockUji {
        fn set_bio(&self, status: &str) -> Result<(), SockError> {
            if self.gagal {
                return Err(SockError::baru("IQ gagal: timeout"));
            }
            self.bio.lock().unwrap().push(status.to_string());
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

    fn run_with(sock: Arc<dyn Sock>, text: &str) -> String {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "setbio".into(),
            sender: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sock,
            ..Default::default()
        };
        (oktz_plugin_api::find("setbio").unwrap().handler)(ctx)
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn run(text: &str) -> String {
        run_with(Arc::new(SockUji::default()), text)
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("setbio").unwrap();
        assert_eq!(p.meta.name, "setbio");
        assert_eq!(p.meta.alias, ["setbiobot", "setstatus", "setabout"]);
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn a_bare_command_shows_the_usage_card() {
        for input in ["", "   "] {
            let t = run(input);
            assert!(t.starts_with("⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*"), "input {input:?}: {t}");
        }
    }

    #[test]
    fn valid_bio_is_set_and_confirmed_with_the_js_text() {
        assert_eq!(run("halo dunia"), sukses_ubah("halo dunia"));
    }

    #[test]
    fn a_bio_of_exactly_the_limit_is_accepted() {
        let bio = "a".repeat(MAX);
        assert_eq!(run(&bio), sukses_ubah(bio.as_str()));
    }

    #[test]
    fn a_bio_one_character_too_long_is_refused() {
        let bio = "a".repeat(MAX + 1);
        assert_eq!(run(&bio), too_long());
    }

    #[test]
    fn the_limit_counts_utf16_units_like_the_js_string_length() {
        // U+1F642 = 2 unit UTF-16: 69 emoji = 138 (sah), 70 = 140 (tolak).
        assert_eq!(run(&"\u{1F642}".repeat(69)), sukses_ubah(&"\u{1F642}".repeat(69)));
        assert_eq!(run(&"\u{1F642}".repeat(70)), too_long());
    }

    #[test]
    fn clear_empties_the_bio_case_insensitively() {
        for input in ["clear", "CLEAR", "Clear"] {
            assert_eq!(run(input), sukses_hapus(), "input {input:?}");
        }
        assert_eq!(bio_to_set("halo"), "halo");
    }

    #[test]
    fn sock_failure_carries_the_error_message() {
        let t = run_with(
            Arc::new(SockUji { bio: Mutex::new(Vec::new()), gagal: true }),
            "halo",
        );
        assert_eq!(t, gagal("IQ gagal: timeout"));
    }

    #[test]
    fn the_bio_reaches_the_sock() {
        let sock = Arc::new(SockUji::default());
        run_with(sock.clone(), "bio baru");
        assert_eq!(*sock.bio.lock().unwrap(), vec!["bio baru".to_string()]);
    }
}
