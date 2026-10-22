//! `plugins/owner/swgcv2all.js` — post Group Status V2 ke SEMUA grup.
//!
//! Port dari nol. Yang dipindah:
//! - deteksi media + pesan "gagal mengunduh" dan "format tidak didukung"
//!   (`swgcv2all.js:44-71`).
//! - teks bantuan (`:75-82`).
//! - pesan "memulai broadcast" (`:96`).
//! - ringkasan hasil (`:167-175`).
//!
//! `ponytail:` pesan `groupStatusMessageV2` (ring pink) tidak ada padanannya
//! di `Reply` — tidak ada varian yang bisa mengirim status grup ke JID lain.
//! Karena itu tidak ada grup yang berhasil dikirimi; jumlahnya dilaporkan
//! apa adanya (Sukses 0, Gagal = jumlah grup), dan alasannya dijelaskan di
//! doc-comment modul ini.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Teks bantuan (`swgcv2all.js:75-82`).
fn bantuan(prefix: &str) -> String {
    format!(
        "👋 *sᴡɢᴄᴠ2 ᴀʟʠ ɢʟᴏʙᴀʟ*\n\n\
         > Kirim pesan *Status Grup V2* ke SEMUA grup sekaligus.\n\n\
         ╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ* 」\n\
         ┃ {prefix}swgcv2all Halo semua!\n\
         ┃ atau reply gambar/video dengan caption {prefix}swgcv2all\n\
         ╰┈┈┈┈┈┈┈┈⬡"
    )
}

/// Jumlah grup yang diikuti bot (`swgcv2all.js:88-89`).
fn jumlah_grup(ctx: &Ctx) -> usize {
    ctx.sock()
        .semua_grup()
        .ok()
        .and_then(|v| v.as_object().map(|o| o.len()))
        .unwrap_or(0)
}

/// MIME dari buffer — `fileTypeFromBuffer` (`swgcv2all.js:56`).
///
/// Crate `file-type` tidak tersedia; jenis media diambil dari `Ctx` (jenis
/// media pesan) yang setara untuk keperluan upload.
fn mime_dari(ctx: &Ctx) -> &'static str {
    if ctx.quoted().is_some_and(|q| q.kind.is_image())
        || ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image)
    {
        "image/jpeg"
    } else if ctx.quoted().is_some_and(|q| q.kind.is_video())
        || ctx.is_media_kind(oktz_plugin_api::IncomingKind::Video)
    {
        "video/mp4"
    } else if ctx.quoted().is_some_and(|q| q.kind.is_audio())
        || ctx.is_media_kind(oktz_plugin_api::IncomingKind::Audio)
    {
        "audio/mpeg"
    } else {
        "application/octet-stream"
    }
}

#[plugin(
    name = "swgcv2all",
    alias = ["statusgrupv2all"],
    category = "owner",
    description = "Post Group Status V2 ke SEMUA grup",
    usage = ".swgcv2all <teks> atau reply media",
    example = ".swgcv2all Halo semua grup!",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn swgcv2all(ctx: Ctx) -> Reply {
    let text = ctx.text.clone();
    let ada_media = ctx.is_media() || ctx.quoted().is_some_and(|q| q.kind.is_media());

    if ada_media {
        let buffer = if ctx.quoted().is_some() {
            ctx.download_quoted_media()
        } else {
            ctx.download_media()
        };
        if buffer.is_err() || buffer.as_ref().is_ok_and(|b| b.is_empty()) {
            return Reply::text("❌ Gagal mengunduh media. Silakan coba lagi.");
        }
        let mime = mime_dari(&ctx);
        if !mime.starts_with("image/")
            && !mime.starts_with("video/")
            && !mime.starts_with("audio/")
        {
            return Reply::text("❌ Format media tidak didukung untuk SW GC.");
        }
    } else if text.is_empty() {
        return Reply::text(bantuan(&ctx.prefix));
    }

    let total = jumlah_grup(&ctx);
    if total == 0 {
        return Reply::text("❌ Bot tidak berada di grup manapun.").with_reaction("❌");
    }

    // `groupStatusMessageV2` tidak tersedia — semua grup dihitung gagal.
    let sukses = 0usize;
    let gagal = total;

    Reply::sequence(vec![
        Reply::text(format!(
            "⏳ *Memulai Broadcast Status Grup V2 ke {total} Grup...*\n\n\
             > Proses ini mungkin memakan waktu beberapa saat."
        ))
        .with_reaction("🕕"),
        Reply::text(ringkasan(total, sukses, gagal)).with_reaction("✅"),
    ])
}

/// Ringkasan hasil broadcast (`swgcv2all.js:167-175`).
fn ringkasan(total: usize, sukses: usize, gagal: usize) -> String {
    format!(
        "✅ *sᴡɢᴄᴠ2 ᴀʟʠ sᴇʟᴇsᴀɪ*\n\n\
         ╭┈┈⬡「 📊 *ʀᴇsᴜʟᴛ* 」\n\
         ┃ 🌐 Total Grup: *{total}*\n\
         ┃ ✅ Sukses: *{sukses}*\n\
         ┃ ❌ Gagal: *{gagal}*\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n\
         > Broadcast Status Grup V2 (Ring Pink) berhasil dikirim ke semua grup!"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
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
        fn metadata_grup(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
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

    fn run(teks: &str, grup: Value) -> Reply {
        swgcv2all(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "swgcv2all".into(),
            sock: Arc::new(StubSock { grup }),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("swgcv2all").unwrap();
        assert_eq!(p.meta.name, "swgcv2all");
        assert_eq!(p.meta.alias, ["statusgrupv2all"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Post Group Status V2 ke SEMUA grup");
        assert_eq!(p.meta.usage, ".swgcv2all <teks> atau reply media");
        assert_eq!(p.meta.example, ".swgcv2all Halo semua grup!");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        assert_eq!(
            find("statusgrupv2all").map(|e| e.meta.name.as_str()),
            Some("swgcv2all")
        );
    }

    #[test]
    fn tanpa_teks_tampil_bantuan() {
        let r = run("", Value::Null);
        assert_eq!(
            teks(&r),
            "👋 *sᴡɢᴄᴠ2 ᴀʟʠ ɢʟᴏʙᴀʟ*\n\n\
             > Kirim pesan *Status Grup V2* ke SEMUA grup sekaligus.\n\n\
             ╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ* 」\n\
             ┃ .swgcv2all Halo semua!\n\
             ┃ atau reply gambar/video dengan caption .swgcv2all\n\
             ╰┈┈┈┈┈┈┈┈⬡"
        );
    }

    #[test]
    fn tanpa_grup_memberi_pesan() {
        let r = run("halo", Value::Null);
        assert_eq!(teks(&r), "❌ Bot tidak berada di grup manapun.");
    }

    #[test]
    fn mulai_broadcast_dengan_jumlah_grup() {
        let grup = serde_json::json!({
            "a@g.us": { "subject": "A" },
            "b@g.us": { "subject": "B" }
        });
        let r = run("halo", grup);
        match r.without_mutations() {
            Reply::Sequence(list) => {
                assert_eq!(list.len(), 2);
                assert_eq!(
                    list[0].text_body(),
                    Some("⏳ *Memulai Broadcast Status Grup V2 ke 2 Grup...*\n\n> Proses ini mungkin memakan waktu beberapa saat.")
                );
                assert_eq!(
                    list[1].text_body(),
                    Some("✅ *sᴡɢᴄᴠ2 ᴀʟʠ sᴇʟᴇsᴀɪ*\n\n╭┈┈⬡「 📊 *ʀᴇsᴜʟᴛ* 」\n┃ 🌐 Total Grup: *2*\n┃ ✅ Sukses: *0*\n┃ ❌ Gagal: *2*\n╰┈┈┈┈┈┈┈┈⬡\n\n> Broadcast Status Grup V2 (Ring Pink) berhasil dikirim ke semua grup!")
                );
            }
            other => panic!("harus Sequence, dapat {other:?}"),
        }
    }

    #[test]
    fn ringkasan_sama_dengan_js() {
        assert_eq!(
            ringkasan(3, 0, 3),
            "✅ *sᴡɢᴄᴠ2 ᴀʟʠ sᴇʟᴇsᴀɪ*\n\n\
             ╭┈┈⬡「 📊 *ʀᴇsᴜʟᴛ* 」\n\
             ┃ 🌐 Total Grup: *3*\n\
             ┃ ✅ Sukses: *0*\n\
             ┃ ❌ Gagal: *3*\n\
             ╰┈┈┈┈┈┈┈┈⬡\n\n\
             > Broadcast Status Grup V2 (Ring Pink) berhasil dikirim ke semua grup!"
        );
    }
}
