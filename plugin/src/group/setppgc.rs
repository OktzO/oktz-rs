//! `plugins/group/setppgc.js` — ubah foto profil grup.
//!
//! Yang dipindah: seluruh alur — deteksi gambar (`setppgc.js:17-27`), pesan
//! cara pakai (`:41-45`), unggah avatar (`:49`), dan teks sukses/error
//! (`:51-52`, `:55-58`).
//!
//! Avatar diunggah lewat `sock.updateProfilePicture(m.chat, buffer)` yang di
//! Rust sudah ada sebagai [`oktz_plugin_api::Sock::set_foto_profil`], dan
//! gambarnya diambil lewat `ctx.download_quoted_media()` /
//! `ctx.download_media()`. Jadi plugin ini bekerja penuh — tidak ada
//! penyimpangan.

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

/// `setppgc.js:41-45` — cara pakai.
fn cara_pakai(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > Reply gambar + `{prefix}setppgc`\n\
         > Kirim gambar + caption `{prefix}setppgc`"
    )
}

/// `setppgc.js:29-32` — gagal mengambil gambar.
const GAGAL_AMBIL: &str = "❌ Gagal mengambil gambar.";

/// `setppgc.js:55-58` — gagal mengubah foto grup.
fn gagal_ubah(pesan: &str) -> String {
    format!("❌ Gagal mengubah foto grup.\n> _{pesan}_")
}

#[plugin(
    name = "setppgc",
    alias = ["setprofilegc", "setppgroup", "setppgrup"],
    category = "group",
    description = "Mengubah foto profil grup",
    usage = ".setppgc (reply gambar)",
    example = ".setppgc",
    cooldown = 10,
    energi = 0,
    is_group = true,
    is_private = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn setppgc(ctx: Ctx) -> Reply {
    // `setppgc.js:17-27` — sumber gambar: reply dulu, lalu pesan pemicu.
    let buffer = if ctx.quoted().is_some_and(|q| q.kind.is_image()) {
        match ctx.download_quoted_media() {
            Ok(b) => b,
            Err(_) => return Reply::text(GAGAL_AMBIL),
        }
    } else if ctx.is_media_kind(IncomingKind::Image) {
        match ctx.download_media() {
            Ok(b) => b,
            Err(_) => return Reply::text(GAGAL_AMBIL),
        }
    } else {
        // `setppgc.js:34-46` — tidak ada gambar.
        return Reply::text(cara_pakai(&ctx.prefix));
    };

    // `setppgc.js:49` — `sock.updateProfilePicture(m.chat, buffer)`.
    match ctx.sock().set_foto_profil(&ctx.chat, &buffer) {
        Ok(()) => Reply::text("✅ Foto profil grup berhasil diperbarui!"),
        Err(e) => Reply::text(gagal_ubah(&e.pesan)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::GroupRoster;
    use oktz_plugin_api::{
        MediaError, MediaFetch, MediaOrigin, QuotedKind, QuotedMessage, Sock, SockError,
    };
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        gagal: bool,
    }

    impl Sock for StubSock {
        fn set_foto_profil(&self, _jid: &str, isi: &[u8]) -> Result<(), SockError> {
            if self.gagal {
                Err(SockError::baru("ditolak"))
            } else if isi.is_empty() {
                Err(SockError::baru("kosong"))
            } else {
                Ok(())
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

    #[derive(Debug)]
    struct StubMedia {
        isi: Vec<u8>,
    }

    impl MediaFetch for StubMedia {
        fn download(&self, _origin: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            Ok(self.isi.clone())
        }
    }

    fn quoted(kind: QuotedKind) -> QuotedMessage {
        QuotedMessage {
            id: "MSG1".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            text: String::new(),
            kind,
            mimetype: None,
            file_name: None,
            from_me: false,
        }
    }

    fn run(sock: Arc<StubSock>, q: Option<QuotedMessage>, isi: Vec<u8>) -> Reply {
        let ctx = Ctx {
            prefix: "!".into(),
            command: "setppgc".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            quoted: q,
            sock,
            media_fetch: Arc::new(StubMedia { isi }),
            roster: GroupRoster {
                participants: vec![],
                is_bot_admin: true,
            },
            ..Default::default()
        };
        (oktz_plugin_api::find("setppgc").unwrap().handler)(ctx)
    }

    fn body(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("setppgc").unwrap();
        assert_eq!(p.meta.name, "setppgc");
        assert_eq!(p.meta.alias, ["setprofilegc", "setppgroup", "setppgrup"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Mengubah foto profil grup");
        assert_eq!(p.meta.usage, ".setppgc (reply gambar)");
        assert_eq!(p.meta.example, ".setppgc");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && p.meta.is_admin && p.meta.is_bot_admin);
        assert!(!p.meta.is_private && !p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["setprofilegc", "setppgroup", "setppgrup"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|p| p.meta.name.as_str()),
                Some("setppgc"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js_tanpa_gambar() {
        let r = run(Arc::new(StubSock::default()), None, vec![1, 2, 3]);
        assert_eq!(
            body(&r),
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
             > Reply gambar + `!setppgc`\n\
             > Kirim gambar + caption `!setppgc`"
        );
    }

    #[test]
    fn output_sama_dengan_js_setelah_avatar_terunggah() {
        let r = run(
            Arc::new(StubSock::default()),
            Some(quoted(QuotedKind::Image)),
            vec![9, 9, 9],
        );
        assert_eq!(body(&r), "✅ Foto profil grup berhasil diperbarui!");
    }

    #[test]
    fn output_sama_dengan_js_saat_unggah_ditolak() {
        let r = run(
            Arc::new(StubSock { gagal: true }),
            Some(quoted(QuotedKind::Image)),
            vec![9, 9, 9],
        );
        assert_eq!(body(&r), "❌ Gagal mengubah foto grup.\n> _ditolak_");
    }

    #[test]
    fn reply_bukan_gambar_tidak_dianggap_gambar() {
        let r = run(
            Arc::new(StubSock::default()),
            Some(quoted(QuotedKind::Text)),
            vec![9, 9, 9],
        );
        assert!(body(&r).contains("*ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*"), "{}", body(&r));
    }
}
