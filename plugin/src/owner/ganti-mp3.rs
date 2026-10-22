//! `plugins/owner/ganti-mp3.js` — ganti audio foto.mp3.
//!
//! Port dari nol. Yang dipindah:
//! - guard "bukan audio" + teks kartu (`ganti-mp3.js:20-24`).
//! - unduh media (`:27-36`).
//! - penulisan aset + teks sukses (`:38-44`).
//!
//! `updateAssetUrl` menulis berkas aset ke
//! `path.resolve(process.cwd(), localPath)`; `ctx.berkas()` berakar ke
//! direktori kerja sehingga langkah itu ditiru persis lewat
//! [`crate::owner::aset::tulis_aset`]. Yang hilang hanya penulisan ulang
//! `config.js` dan unggah ke CDN (lihat modul `aset`).

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Judul kartu (`ganti-mp3.js:23`).
const JUDUL: &str = "🎵 *ɢᴀɴᴛɪ ᴍᴘ3.ᴍᴘ3*";
/// Path aset (`ganti-mp3.js:23`).
const FILE: &str = "assets/audio/foto.mp3";

#[plugin(
    name = "ganti-mp3.mp3",
    alias = ["gantimp3", "setmp3"],
    category = "owner",
    description = "Ganti audio foto.mp3",
    usage = ".ganti-mp3.mp3 (reply/kirim audio)",
    example = ".ganti-mp3.mp3",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganti_mp3(ctx: Ctx) -> Reply {
    if !crate::owner::aset::adalah_audio(&ctx) {
        return Reply::text(crate::owner::aset::pakai_media(JUDUL, FILE, "audio"));
    }

    let Some(buffer) = crate::owner::aset::unduh_media(&ctx) else {
        return Reply::text("❌ Gagal mendownload audio");
    };

    match crate::owner::aset::tulis_aset(&ctx, "mp3", "foto.mp3", &buffer) {
        Ok(new_url) => Reply::text(format!(
            "✅ *ʙᴇʀʜᴀsɪʟ*\n\n\
             > File foto.mp3 telah diganti ke URL baru:\n\
             > {new_url}\n\
             > Config telah diupdate secara realtime!"
        )),
        Err(e) => Reply::text(format!("❌ Gagal mengupload file: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::IncomingKind;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use oktz_plugin_api::find;
    use oktz_plugin_api::IncomingMedia;
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubFetch {
        ok: bool,
        bytes: Vec<u8>,
    }

    impl oktz_plugin_api::MediaFetch for StubFetch {
        fn download(
            &self,
            _: oktz_plugin_api::MediaOrigin,
        ) -> Result<Vec<u8>, oktz_plugin_api::MediaError> {
            if self.ok {
                Ok(self.bytes.clone())
            } else {
                Err(oktz_plugin_api::MediaError {
                    message: "gagal".into(),
                })
            }
        }
    }

    fn run(is_audio: bool, download_ok: bool) -> (Reply, Arc<BerkasPalsu>) {
        let media = if is_audio {
            Some(IncomingMedia {
                kind: oktz_plugin_api::IncomingKind::Audio,
                mimetype: Some("audio/mpeg".into()),
                file_name: None,
                caption: String::new(),
            })
        } else {
            None
        };
        let berkas = Arc::new(BerkasPalsu::baru());
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ganti-mp3.mp3".into(),
            sender: ME.into(),
            chat: ME.into(),
            media,
            media_fetch: Arc::new(StubFetch {
                ok: download_ok,
                bytes: b"MP3".to_vec(),
            }),
            berkas: berkas.clone(),
            ..Ctx::default()
        };
        (ganti_mp3(ctx), berkas)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("ganti-mp3.mp3").unwrap();
        assert_eq!(p.meta.name, "ganti-mp3.mp3");
        assert_eq!(p.meta.alias, ["gantimp3", "setmp3"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Ganti audio foto.mp3");
        assert_eq!(p.meta.usage, ".ganti-mp3.mp3 (reply/kirim audio)");
        assert_eq!(p.meta.example, ".ganti-mp3.mp3");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["gantimp3", "setmp3"] {
            assert_eq!(
                find(a).map(|e| e.meta.name.as_str()),
                Some("ganti-mp3.mp3"),
                "{a}"
            );
        }
    }

    #[test]
    fn bukan_audio_tampil_kartu_pakai() {
        let (r, _b) = run(false, true);
        assert_eq!(
            teks(&r),
            "🎵 *ɢᴀɴᴛɪ ᴍᴘ3.ᴍᴘ3*\n\n\
             > Kirim/reply audio untuk mengganti\n\
             > File: assets/audio/foto.mp3"
        );
    }

    #[test]
    fn audio_ditulis_ke_aset_dan_dilaporkan() {
        let (r, b) = run(true, true);
        assert_eq!(
            teks(&r),
            "✅ *ʙᴇʀʜᴀsɪʟ*\n\n\
             > File foto.mp3 telah diganti ke URL baru:\n\
             > ./assets/audio/foto.mp3\n\
             > Config telah diupdate secara realtime!"
        );
        assert!(b.ada("assets/audio/foto.mp3"));
    }
}
