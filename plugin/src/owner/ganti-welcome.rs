//! `plugins/owner/ganti-welcome.js` — ganti gambar welcome.jpg.
//!
//! Port dari nol. Yang dipindah:
//! - guard "bukan gambar" + teks kartu (`ganti-welcome.js:20-24`).
//! - unduh media (`:27-36`).
//! - penulisan aset + teks sukses (`:38-44`).
//!
//! `updateAssetUrl` menulis berkas aset ke
//! `path.resolve(process.cwd(), localPath)`; `ctx.berkas()` berakar ke
//! direktori kerja sehingga langkah itu ditiru persis lewat
//! [`crate::owner::aset::tulis_aset`]. Yang hilang hanya penulisan ulang
//! `config.js` dan unggah ke CDN (lihat modul `aset`).

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Judul kartu (`ganti-welcome.js:23`).
const JUDUL: &str = "🖼️ *ɢᴀɴᴛɪ ᴡᴇʟᴄᴏᴍᴇ.ᴊᴘɢ*";
/// Path aset (`ganti-welcome.js:23`).
const FILE: &str = "assets/image/welcome.jpeg";

#[plugin(
    name = "ganti-welcome.jpg",
    alias = ["gantiwelcome", "setfotowelcome"],
    category = "owner",
    description = "Ganti gambar welcome.jpg (thumbnail welcome)",
    usage = ".ganti-welcome.jpg (reply/kirim gambar)",
    example = ".ganti-welcome.jpg",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganti_welcome(ctx: Ctx) -> Reply {
    if !crate::owner::aset::adalah_gambar(&ctx) {
        return Reply::text(crate::owner::aset::pakai(JUDUL, FILE));
    }

    let Some(buffer) = crate::owner::aset::unduh_media(&ctx) else {
        return Reply::text("❌ Gagal mendownload gambar");
    };

    match crate::owner::aset::tulis_aset(&ctx, "welcome", "welcome.jpeg", &buffer) {
        Ok(new_url) => Reply::text(format!(
            "✅ *ʙᴇʀʜᴀsɪʟ*\n\n\
             > Gambar welcome.jpeg telah diganti ke URL baru:\n\
             > {new_url}\n\
             > Config telah diupdate secara realtime!"
        )),
        Err(e) => Reply::text(format!("❌ Gagal mengupload gambar: {e}")),
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

    fn run(is_image: bool, download_ok: bool) -> (Reply, Arc<BerkasPalsu>) {
        let media = if is_image {
            Some(IncomingMedia {
                kind: oktz_plugin_api::IncomingKind::Image,
                mimetype: Some("image/jpeg".into()),
                file_name: None,
                caption: String::new(),
            })
        } else {
            None
        };
        let berkas = Arc::new(BerkasPalsu::baru());
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ganti-welcome.jpg".into(),
            sender: ME.into(),
            chat: ME.into(),
            media,
            media_fetch: Arc::new(StubFetch {
                ok: download_ok,
                bytes: b"IMG".to_vec(),
            }),
            berkas: berkas.clone(),
            ..Ctx::default()
        };
        (ganti_welcome(ctx), berkas)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("ganti-welcome.jpg").unwrap();
        assert_eq!(p.meta.name, "ganti-welcome.jpg");
        assert_eq!(p.meta.alias, ["gantiwelcome", "setfotowelcome"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Ganti gambar welcome.jpg (thumbnail welcome)");
        assert_eq!(p.meta.usage, ".ganti-welcome.jpg (reply/kirim gambar)");
        assert_eq!(p.meta.example, ".ganti-welcome.jpg");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["gantiwelcome", "setfotowelcome"] {
            assert_eq!(
                find(a).map(|e| e.meta.name.as_str()),
                Some("ganti-welcome.jpg"),
                "{a}"
            );
        }
    }

    #[test]
    fn bukan_gambar_tampil_kartu_pakai() {
        let (r, _b) = run(false, true);
        assert_eq!(
            teks(&r),
            "🖼️ *ɢᴀɴᴛɪ ᴡᴇʟᴄᴏᴍᴇ.ᴊᴘɢ*\n\n\
             > Kirim/reply gambar untuk mengganti\n\
             > File: assets/image/welcome.jpeg"
        );
    }

    #[test]
    fn gambar_ditulis_ke_aset_dan_dilaporkan() {
        let (r, b) = run(true, true);
        assert_eq!(
            teks(&r),
            "✅ *ʙᴇʀʜᴀsɪʟ*\n\n\
             > Gambar welcome.jpeg telah diganti ke URL baru:\n\
             > ./assets/image/welcome.jpeg\n\
             > Config telah diupdate secara realtime!"
        );
        assert!(b.ada("assets/image/welcome.jpeg"));
    }
}
