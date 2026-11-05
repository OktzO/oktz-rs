//! `plugins/tools/setpp.js` — mengubah foto profil bot.
//!
//! Port penuh: unduh gambar (quoted ATAU trigger) lewat `ctx.media_fetch`,
//! lalu `sock.updateProfilePicture(botJid, buffer)` → `ctx.sock().set_foto_profil`.
//!
//! `ponytail:` `sock.user?.id` (JID bot) tidak ada di `Ctx` — tapi
//! `SockKlien::set_foto_profil` hanya mendukung foto profil bot sendiri untuk
//! jid non-grup, jadi argumen jid dilewati apa adanya (paritas tetap: foto bot
//! yang berubah). Upgrade path: ekspos `bot_id` di `Ctx` untuk validasi jid.

use oktz_plugin_api::{Ctx, IncomingKind, MediaOrigin, Reply, plugin};

fn usage(prefix: &str) -> String {
    let prefix = if prefix.is_empty() { "." } else { prefix };
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > Reply gambar + `{prefix}setpp`\n\
         > Kirim gambar + caption `{prefix}setpp`"
    )
}

fn sukses() -> &'static str {
    "✅ *ᴘᴘ ʙᴏᴛ ᴅɪᴜʙᴀʜ*\n\n> Foto profil bot berhasil diperbarui!"
}

fn gagal(e: &str) -> String {
    format!("❌ *ɢᴀɢᴀʟ*\n\n> Tidak dapat mengubah foto bot.\n> _{e}_")
}

fn gagal_ambil() -> &'static str {
    "❌ Gagal mengambil gambar."
}

#[plugin(
    name = "setpp",
    alias = ["setprofilebot", "setppbot", "setfotobot"],
    category = "tools",
    description = "Mengubah foto profil bot",
    usage = ".setpp (reply gambar)",
    example = ".setpp",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true
)]
pub fn setpp(ctx: Ctx) -> Reply {
    // Sumber: quoted gambar dulu (JS `m.quoted.isImage`), lalu gambar dengan
    // caption (JS `m.isImage`).
    let origin = match (&ctx.quoted, &ctx.media) {
        (Some(q), _) if q.kind == oktz_plugin_api::QuotedKind::Image => MediaOrigin::Quoted,
        (None, Some(m)) if matches!(m.kind, IncomingKind::Image) => MediaOrigin::Trigger,
        _ => return Reply::text(usage(&ctx.prefix)),
    };

    let buffer = match ctx.media_fetch.download(origin) {
        Ok(b) => b,
        Err(_) => return Reply::text(gagal_ambil()),
    };
    if buffer.is_empty() {
        return Reply::text(usage(&ctx.prefix));
    }

    match ctx.sock().set_foto_profil(&ctx.sender, &buffer) {
        Ok(()) => Reply::text(sukses()),
        Err(e) => Reply::text(gagal(&e.pesan)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        AdaWhatsapp, AksiPeserta, Ctx, IncomingMedia, MediaError, MediaFetch, QuotedKind,
        QuotedMessage, Sock, SockError,
    };
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    /// PNG 32x32 valid (>100 byte).
    const PNG_FIXTURE: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 32, 0, 0, 0, 32, 8,
        6, 0, 0, 0, 62, 177, 168, 86, 0, 0, 1, 164, 73, 68, 65, 84, 120, 156, 237, 207, 49, 14, 0,
        48, 12, 2, 192, 190, 183, 199, 102, 33, 23, 110, 214, 25, 116, 24, 24, 3, 133, 110, 176,
        131, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61,
        216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65,
        30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108,
        163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61,
        216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65,
        30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108,
        163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61,
        216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65,
        30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108, 163, 61, 216, 65, 30, 108,
        163, 61, 216, 193, 215, 227, 15, 22, 232, 50, 37, 254, 230, 0, 0, 0, 0, 73, 69, 78, 68,
        174, 66, 96, 130,
    ];

    /// Stub sock: mencatat foto yang dikirim, `gagal = true` → selalu gagal.
    #[derive(Debug, Default)]
    struct SockUji {
        foto: Mutex<Vec<Vec<u8>>>,
        gagal: bool,
    }

    impl Sock for SockUji {
        fn set_foto_profil(&self, _jid: &str, isi: &[u8]) -> Result<(), SockError> {
            if self.gagal {
                return Err(SockError::baru("IQ gagal: timeout"));
            }
            self.foto.lock().unwrap().push(isi.to_vec());
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
        fn blokir(&self, _: &str, _: bool) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    /// Stub unduhan gambar.
    #[derive(Debug, Clone)]
    struct Unduh {
        hasil: Result<Vec<u8>, MediaError>,
    }

    impl MediaFetch for Unduh {
        fn download(&self, _origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.hasil.clone()
        }
    }

    fn quoted(kind: QuotedKind) -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: String::new(),
            kind,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            from_me: false,
            timestamp: 0,
        }
    }

    fn quoted_image() -> QuotedMessage {
        quoted(QuotedKind::Image)
    }

    fn media_image() -> IncomingMedia {
        IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        }
    }

    fn run(quoted: Option<QuotedMessage>, media: Option<IncomingMedia>, unduh: Unduh, sock: Arc<SockUji>) -> String {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "setpp".into(),
            sender: "628111@s.whatsapp.net".into(),
            quoted,
            media,
            media_fetch: Arc::new(unduh),
            sock,
            ..Default::default()
        };
        (oktz_plugin_api::find("setpp").unwrap().handler)(ctx)
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn unduh_ok() -> Unduh {
        Unduh { hasil: Ok(PNG_FIXTURE.to_vec()) }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("setpp").unwrap();
        assert_eq!(p.meta.name, "setpp");
        assert_eq!(p.meta.alias, ["setprofilebot", "setppbot", "setfotobot"]);
        assert_eq!(p.meta.category, "tools");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn without_an_image_it_shows_the_usage_card() {
        // JS: `buffer` tetap null untuk quoted non-gambar dan pesan tanpa media.
        assert_eq!(run(None, None, unduh_ok(), Default::default()), usage("."));
        for kind in [QuotedKind::Text, QuotedKind::Video, QuotedKind::Sticker, QuotedKind::Document] {
            assert_eq!(
                run(Some(quoted(kind.clone())), None, unduh_ok(), Default::default()),
                usage("."),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn quoted_image_is_uploaded_and_confirmed() {
        let sock: Arc<SockUji> = Default::default();
        let t = run(Some(quoted_image()), None, unduh_ok(), sock.clone());
        assert_eq!(t, sukses());
        let foto = sock.foto.lock().unwrap();
        assert_eq!(foto.len(), 1);
        assert_eq!(foto[0], PNG_FIXTURE, "byte gambar harus utuh ke sock");
    }

    #[test]
    fn trigger_image_uses_trigger_origin() {
        let sock: Arc<SockUji> = Default::default();
        let t = run(None, Some(media_image()), unduh_ok(), sock.clone());
        assert_eq!(t, sukses());
        assert_eq!(sock.foto.lock().unwrap().len(), 1);
    }

    #[test]
    fn download_failure_is_gagal_ambil() {
        let unduh = Unduh { hasil: Err(MediaError { message: "410".into() }) };
        assert_eq!(
            run(Some(quoted_image()), None, unduh, Default::default()),
            gagal_ambil()
        );
    }

    #[test]
    fn empty_download_shows_the_usage_card_like_js() {
        // JS `if (!buffer)` — unduhan kosong = tidak ada gambar.
        let unduh = Unduh { hasil: Ok(vec![]) };
        assert_eq!(run(Some(quoted_image()), None, unduh, Default::default()), usage("."));
    }

    #[test]
    fn sock_failure_carries_the_error_message() {
        let sock = Arc::new(SockUji { foto: Mutex::new(Vec::new()), gagal: true });
        assert_eq!(run(Some(quoted_image()), None, unduh_ok(), sock), gagal("IQ gagal: timeout"));
    }
}
