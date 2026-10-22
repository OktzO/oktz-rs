//! `plugins/group/getpp.js` — kirim foto profil target (mention/reply).
//!
//! Yang dipindah: seluruh penentuan target (`getpp.js:18-28`), karena itu
//! murni string, dan pengiriman gambarnya (`getpp.js:39`).
//!
//! Avatar diambil lewat `sock.profilePictureUrl(target, 'image')` yang di
//! Rust sudah ada sebagai [`oktz_plugin_api::Sock::foto_profil`]. Kalau
//! target tidak punya avatar, JS memakai fallback
//! `https://files.catbox.moe/ejy4ky.jpg` (`getpp.js:35`) — di Rust fallback
//! yang sama dipakai juga saat pemanggilan gagal.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

/// `getpp.js:35` — avatar cadangan kalau tidak ada / gagal diambil.
const FALLBACK: &str = "https://files.catbox.moe/ejy4ky.jpg";

/// `getpp.js:18-28`. `num.startsWith('0')` → `62` + sisa digit, persis
/// konversi awalan `0` ala Indonesia.
pub(crate) fn resolve_target(ctx: &Ctx) -> String {
    if let Some(q) = ctx.quoted() {
        return q.sender.clone();
    }
    if let Some(jid) = ctx.mentioned.first() {
        return jid.clone();
    }
    if let Some(arg) = ctx.args.first() {
        let mut num: String = arg.chars().filter(char::is_ascii_digit).collect();
        if num.starts_with('0') {
            num = format!("62{}", &num[1..]);
        }
        return format!("{num}@s.whatsapp.net");
    }
    ctx.sender.clone()
}

#[plugin(
    name = "getpp",
    alias = ["pp", "profilepic", "avatar"],
    category = "group",
    description = "Ambil foto profil target (mention/reply)",
    usage = ".getpp @user",
    example = ".getpp @628xxx",
    cooldown = 5,
    energi = 0,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn getpp(ctx: Ctx) -> Reply {
    let target = resolve_target(&ctx);
    // `getpp.js:30` — `const targetNum = target.split('@')[0]`.
    let target_num = target.split('@').next().unwrap_or(&target).to_string();

    // `getpp.js:32-37` — ambil URL avatar, fallback kalau tidak ada.
    let url = match ctx.sock().foto_profil(&target, "image") {
        Ok(Some(url)) => url,
        Ok(None) | Err(_) => FALLBACK.to_string(),
    };

    // `getpp.js:39` — `sock.sendMedia(m.chat, ppUrl, caption, m, {type:'image', mentions:[target]})`.
    Reply::Image {
        media: Some(MediaSource::url(url)),
        caption: format!("Foto profil milik @{target_num}"),
        mentions: vec![target],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::GroupRoster;
    use oktz_plugin_api::{QuotedKind, QuotedMessage, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        foto: Option<String>,
    }

    impl Sock for StubSock {
        fn foto_profil(&self, _jid: &str, _tipe: &str) -> Result<Option<String>, SockError> {
            Ok(self.foto.clone())
        }
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, SockError> {
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

    fn base() -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "getpp".into(),
            sender: ME.into(),
            chat: ME.into(),
            sock: Arc::new(StubSock::default()),
            roster: GroupRoster {
                participants: vec![],
                is_bot_admin: false,
            },
            ..Default::default()
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let e = oktz_plugin_api::find("getpp").unwrap();
        assert_eq!(e.meta.name, "getpp");
        assert_eq!(e.meta.alias, ["pp", "profilepic", "avatar"]);
        assert_eq!(e.meta.category, "group");
        assert_eq!(
            e.meta.description,
            "Ambil foto profil target (mention/reply)"
        );
        assert_eq!(e.meta.usage, ".getpp @user");
        assert_eq!(e.meta.example, ".getpp @628xxx");
        assert_eq!(e.meta.cooldown, 5);
        assert_eq!(e.meta.energi, 0);
        assert!(e.meta.is_enabled);
        assert!(!e.meta.is_group && !e.meta.is_private);
        assert!(!e.meta.is_admin && !e.meta.is_bot_admin);
        assert!(!e.meta.is_owner && !e.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["pp", "profilepic", "avatar"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|e| e.meta.name.as_str()),
                Some("getpp"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn without_quote_mention_or_number_the_sender_is_the_target() {
        let ctx = base();
        assert_eq!(resolve_target(&ctx), ME);
    }

    #[test]
    fn a_mention_wins_over_a_number_argument() {
        let ctx = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            args: vec!["628777".into()],
            ..base()
        };
        assert_eq!(resolve_target(&ctx), "628999@s.whatsapp.net");
    }

    #[test]
    fn a_reply_wins_over_everything() {
        let ctx = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            args: vec!["628777".into()],
            quoted: Some(QuotedMessage {
                id: "M1".into(),
                sender: "628555:2@s.whatsapp.net".into(),
                chat: "120363@g.us".into(),
                text: "hai".into(),
                kind: QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            ..base()
        };
        assert_eq!(resolve_target(&ctx), "628555:2@s.whatsapp.net");
    }

    #[test]
    fn digits_are_extracted_from_a_noisy_argument() {
        for (arg, expect) in [
            ("@628123", "628123@s.whatsapp.net"),
            ("+62 8123", "628123@s.whatsapp.net"),
            ("0812-3456", "628123456@s.whatsapp.net"),
            // Tanpa digit sama sekali → `'' + '@s.whatsapp.net'`, bukan error.
            ("halo", "@s.whatsapp.net"),
        ] {
            let ctx = Ctx {
                args: vec![arg.into()],
                ..base()
            };
            assert_eq!(resolve_target(&ctx), expect, "arg {arg:?}");
        }
    }

    #[test]
    fn a_leading_zero_becomes_62_exactly_once() {
        let ctx = Ctx {
            args: vec!["00".into()],
            ..base()
        };
        assert_eq!(
            resolve_target(&ctx),
            "620@s.whatsapp.net",
            "'62' + num.slice(1)"
        );
    }

    #[test]
    fn output_sama_dengan_js_kirim_foto_profil() {
        let ctx = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            sock: Arc::new(StubSock {
                foto: Some("https://example.com/pp.jpg".into()),
            }),
            ..base()
        };
        let r = getpp(ctx);
        let Reply::Image {
            media,
            caption,
            mentions,
        } = r.without_mutations()
        else {
            panic!("harus Image");
        };
        assert_eq!(caption, "Foto profil milik @628999");
        assert_eq!(mentions, &vec!["628999@s.whatsapp.net".to_string()]);
        assert!(matches!(
            media.as_ref(),
            Some(MediaSource::Url { url, .. }) if url == "https://example.com/pp.jpg"
        ));
    }

    #[test]
    fn output_sama_dengan_js_pakai_avatar_cadangan() {
        let ctx = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            sock: Arc::new(StubSock { foto: None }),
            ..base()
        };
        let r = getpp(ctx);
        let Reply::Image { media, caption, .. } = r.without_mutations() else {
            panic!("harus Image");
        };
        assert_eq!(caption, "Foto profil milik @628999");
        assert!(
            matches!(media.as_ref(), Some(MediaSource::Url { url, .. }) if url == FALLBACK),
            "avatar cadangan dipakai kalau target tidak punya foto"
        );
    }
}
