//! `plugins/owner/swgc.js` — post Group Status/Story ke grup pilihan.
//!
//! Port dari nol. Yang dipindah:
//! - branch `--confirm <grup>` dan pesan "tidak ada data pending"
//!   (`swgc.js:106-116`).
//! - volunteer media (quoted / pemicu) dan storinya ke folder `temp/`
//!   (`:193-257`, `:273-277`).
//! - kartu pemilih grup dengan `single_select` + tombol batal (`:291-358`).
//! - pesan bantuan (`:260-271`).
//!
//! `ponytail:` `sock.sendMessage(jid, { groupStatusMessage })` — pesan status
//! grup dengan border hijau — tidak punya padanan di `Reply` (tidak ada varian
//! `groupStatusMessage`). Karena itu branch `--confirm` berakhir di bentuk
//! `catch` JS: `❌ *ᴇʀʀᴏʀ*` dengan pesan gagalnya.
//!
//! `ponytail:` `pendingSwgc` (Map di memori) dipetakan ke `ctx.sessions` dengan
//! key `swgc:pending:<sender>`; media disimpan ke `temp/swgc_<ts>.<ext>` —
//! padanan persis dari `tempFile` JS (`swgc.js:205`).

use oktz_plugin_api::{Ctx, Reply, plugin};
use oktz_plugin_api::registry::{
    CardContext, CardList, CardRow, CardSection, InteractiveButton,
};
use serde_json::Value;

/// Prefix key session pending (`pendingSwgc`).
const PENDING_PREFIX: &str = "swgc:pending:";
/// Folder berkas temp (`swgc.js:182`).
const TEMP_DIR: &str = "temp";

fn pending_key(ctx: &Ctx) -> String {
    format!("{PENDING_PREFIX}{}", ctx.sender)
}

/// `Object.entries(groups)` (`swgc.js:295`) — daftar (jid, metadata).
fn daftar_grup(ctx: &Ctx) -> Vec<(String, Value)> {
    let Ok(all) = ctx.sock().semua_grup() else {
        return Vec::new();
    };
    let Some(obj) = all.as_object() else {
        return Vec::new();
    };
    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// `mediaType` (`swgc.js:309-317`).
fn label_media(jenis: &str) -> &'static str {
    match jenis {
        "text" => "Teks",
        "image" => "Gambar",
        "video" => "Video",
        "audio" => "Audio",
        _ => "Media",
    }
}

/// Teks bantuan (`swgc.js:261-268`).
fn bantuan(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}swgc teks` - Story teks\n\
         > Reply gambar/video/audio + `{prefix}swgc`\n\
         > Kirim gambar/video + caption `{prefix}swgc`"
    )
}

#[plugin(
    name = "swgc",
    alias = ["statusgrup", "swgroup", "groupstory", "toswgc"],
    category = "owner",
    description = "Post Group Status/Story ke grup pilihan (border hijau)",
    usage = ".swgc <teks> atau reply media",
    example = ".swgc Halo semua!",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn swgc(ctx: Ctx) -> Reply {
    let args = ctx.args.clone();

    if args.first().map(String::as_str) == Some("--confirm") && args.len() >= 2 {
        let target_group_id = args[1].clone();
        let pending = ctx.session_get(&pending_key(&ctx));
        let Some(pending) = pending else {
            return Reply::text(
                "⚠️ *Tidak ada data pending. Silakan kirim ulang media + .swgc*",
            );
        };

        let _group_name = ctx
            .sock()
            .metadata_grup(&target_group_id)
            .ok()
            .and_then(|m| m.get("subject").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "Grup".to_string());

        let _ = ctx.berkas().hapus(
            pending
                .get("tempFile")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        );
        ctx.session_take(&pending_key(&ctx));

        // `sock.sendMessage(jid, { groupStatusMessage })` tidak ada padanannya,
        // jadi hasilnya sama dengan `catch` JS.
        return Reply::text(
            "❌ *ᴇʀʀᴏʀ*\n\n\
             > Gagal posting story.\n\
             > _grup status (groupStatusMessage) tidak tersedia di versi Rust_",
        );
    }

    // Deteksi media (quoted dulu, lalu pemicu) — `swgc.js:193-258`.
    let (jenis, buffer) = if ctx
        .quoted()
        .is_some_and(|q| q.kind.is_media())
    {
        match ctx.download_quoted_media() {
            Ok(b) => (jenis_dari_quoted(&ctx), Some(b)),
            Err(_) => {
                return Reply::text(crate::common::error_template(&ctx));
            }
        }
    } else if ctx.is_media() {
        match ctx.download_media() {
            Ok(b) => (jenis_dari_trigger(&ctx), Some(b)),
            Err(_) => {
                return Reply::text(crate::common::error_template(&ctx));
            }
        }
    } else {
        ("text".to_string(), None)
    };

    let text = ctx.text.trim();

    if jenis == "text" && text.is_empty() {
        return Reply::text(bantuan(&ctx.prefix));
    }

    // Simpan media ke `temp/` supaya bisa dipakai ulang saat `--confirm`
    // (`swgc.js:205`, `:218`).
    let mut temp_file = String::new();
    if let Some(buffer) = buffer {
        let ext = match jenis.as_str() {
            "image" => "jpg",
            "video" => "mp4",
            "audio" => "mp3",
            _ => "bin",
        };
        let _ = ctx.berkas().buat_folder(TEMP_DIR);
        temp_file = format!("{TEMP_DIR}/swgc_{}.{ext}", crate::common::now_ms());
        let _ = ctx.berkas().tulis(&temp_file, &buffer);
    }

    ctx.session_set(
        &pending_key(&ctx),
        serde_json::json!({
            "kind": jenis,
            "caption": text,
            "tempFile": temp_file,
            "timestamp": crate::common::now_ms(),
        }),
    );

    let groups = daftar_grup(&ctx);
    if groups.is_empty() {
        return Reply::text("⚠️ *Bot tidak berada di grup manapun.*");
    }

    let rows: Vec<CardRow> = groups
        .iter()
        .map(|(jid, meta)| {
            let subjek = meta
                .get("subject")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("Unknown Group");
            CardRow::new(subjek, format!("{}swgc --confirm {jid}", ctx.prefix))
                .description(jid.clone())
        })
        .collect();

    let list = CardList::new(
        "🏠 Pilih Grup",
        vec![CardSection::new("Daftar Grup").rows(rows)],
    );

    let body = format!(
        "📋 *ᴘɪʟɪʜ ɢʀᴜᴘ ᴅɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ*\n\n\
         > Media: *{}*\n\
         > Total Grup: *{}*\n\n\
         _Pilih grup dari daftar di bawah:_",
        label_media(&jenis),
        groups.len()
    );

    let footer = {
        let n = ctx.config().bot.name.clone();
        if n.is_empty() { "Bot".to_string() } else { n }
    };

    Reply::interactive(body)
        .with_footer(footer)
        .with_buttons(vec![
            InteractiveButton::select(&list),
            InteractiveButton::quick_reply(
                "❌ Batal",
                format!("{}cancelswgc", ctx.prefix),
            ),
        ])
        .with_context(CardContext::new().quote_trigger())
}

/// `m.quoted.isImage / isVideo / isAudio` (`swgc.js:208-222`).
fn jenis_dari_quoted(ctx: &Ctx) -> String {
    let Some(q) = ctx.quoted() else {
        return "text".to_string();
    };
    if q.kind.is_image() {
        "image".into()
    } else if q.kind.is_video() {
        "video".into()
    } else if q.kind.is_audio() {
        "audio".into()
    } else {
        "text".into()
    }
}

/// `m.isImage / m.isVideo / m.isAudio` (`swgc.js:228-258`).
fn jenis_dari_trigger(ctx: &Ctx) -> String {
    if ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image) {
        "image".into()
    } else if ctx.is_media_kind(oktz_plugin_api::IncomingKind::Video) {
        "video".into()
    } else if ctx.is_media_kind(oktz_plugin_api::IncomingKind::Audio) {
        "audio".into()
    } else {
        "text".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;
    use oktz_plugin_api::sock::{AdaWhatsapp, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        grup: Value,
    }

    impl Sock for StubSock {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn metadata_grup(&self, jid: &str) -> Result<Value, SockError> {
            if jid == "120363@g.us" {
                Ok(serde_json::json!({ "subject": "Grup Satu" }))
            } else {
                Err(SockError::baru("tidak ada"))
            }
        }
        fn semua_grup(&self) -> Result<Value, SockError> {
            Ok(self.grup.clone())
        }
        fn ubah_peserta(
            &self,
            _: &str,
            _: &[String],
            _: oktz_plugin_api::sock::AksiPeserta,
        ) -> Result<(), SockError> {
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
    }

    fn run(args: &[&str], teks: &str, grup: Value) -> Reply {
        let mut config = oktz_plugin_api::config::Runtime::default();
        config.bot.name = "Oktz Bot".into();
        swgc(Ctx {
            text: teks.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "swgc".into(),
            sock: Arc::new(StubSock { grup }),
            sessions: oktz_plugin_api::new_session_store(),
            config: Arc::new(config),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("swgc").unwrap();
        assert_eq!(p.meta.name, "swgc");
        assert_eq!(
            p.meta.alias,
            ["statusgrup", "swgroup", "groupstory", "toswgc"]
        );
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Post Group Status/Story ke grup pilihan (border hijau)"
        );
        assert_eq!(p.meta.usage, ".swgc <teks> atau reply media");
        assert_eq!(p.meta.example, ".swgc Halo semua!");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["statusgrup", "swgroup", "groupstory", "toswgc"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("swgc"), "{a}");
        }
    }

    #[test]
    fn tanpa_teks_tampil_bantuan() {
        let r = run(&[], "", Value::Null);
        assert_eq!(
            teks(&r),
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
             > `.swgc teks` - Story teks\n\
             > Reply gambar/video/audio + `.swgc`\n\
             > Kirim gambar/video + caption `.swgc`"
        );
    }

    #[test]
    fn confirm_tanpa_pending_dikatakan_tidak_ada_data() {
        let r = run(&["--confirm", "120363@g.us"], "halo", Value::Null);
        assert_eq!(
            teks(&r),
            "⚠️ *Tidak ada data pending. Silakan kirim ulang media + .swgc*"
        );
    }

    #[test]
    fn kartu_memilih_grup_dengan_media_teks() {
        let grup = serde_json::json!({
            "120363@g.us": { "subject": "Grup Satu" }
        });
        let r = run(&[], "Halo semua", grup);
        let t = teks(&r);
        assert!(t.starts_with("📋 *ᴘɪʟɪʜ ɢʀᴜᴘ ᴅɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ*"), "{t}");
        assert!(t.contains("> Media: *Teks*"), "{t}");
        assert!(t.contains("> Total Grup: *1*"), "{t}");
        assert!(t.contains("_Pilih grup dari daftar di bawah:_"), "{t}");
        match r.without_mutations() {
            Reply::Interactive { buttons, footer, .. } => {
                assert_eq!(buttons.len(), 2);
                assert_eq!(footer.as_deref(), Some("Oktz Bot"));
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }

    #[test]
    fn tanpa_grup_memberi_pesan() {
        let r = run(&[], "Halo semua", Value::Null);
        assert_eq!(teks(&r), "⚠️ *Bot tidak berada di grup manapun.*");
    }

    #[test]
    fn confirm_dengan_pending_berakhir_di_error_grup_status() {
        let grup = serde_json::json!({ "120363@g.us": { "subject": "Grup Satu" } });
        let s = oktz_plugin_api::new_session_store();
        let ctx = Ctx {
            text: "Halo semua".into(),
            args: Vec::new(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "swgc".into(),
            sock: Arc::new(StubSock { grup }),
            sessions: s.clone(),
            ..Ctx::default()
        };
        let _ = swgc(ctx.clone());
        assert!(s.lock().unwrap().contains_key("swgc:pending:628111@s.whatsapp.net"));

        let r = swgc(Ctx {
            text: String::new(),
            args: vec!["--confirm".into(), "120363@g.us".into()],
            sessions: s.clone(),
            ..ctx
        });
        assert_eq!(
            teks(&r),
            "❌ *ᴇʀʀᴏʀ*\n\n\
             > Gagal posting story.\n\
             > _grup status (groupStatusMessage) tidak tersedia di versi Rust_"
        );
        assert!(!s.lock().unwrap().contains_key("swgc:pending:628111@s.whatsapp.net"));
    }
}
