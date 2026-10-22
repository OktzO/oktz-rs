//! `plugins/owner/ganti-mp4.mp4.js` — ganti aset foto.mp4.
//!
//! Alur sama dengan JS: cek jenis media, unduh buffer, balas
//! "⏳ Sedang mengupload gambar...", lalu tulis ulang berkas aset dan
//! laporkan jalurnya. Helper-nya ada di `ganti_gambar.rs`.
//!
//! Catatan: `updateAssetUrl` di JS juga menulis ulang `config.js`; di Rust
//! jalur aset sudah ada di `config.assets`, jadi cukup berkas asetnya yang
//! ditulis.

use oktz_plugin_api::{Ctx, Reply, plugin};

use super::ganti_gambar::{Aset, Jenis, jalankan};

/// Parameter aset untuk plugin ini.
const ASET: Aset = Aset {
    emoji_judul: "🎬",
    judul: "ɢᴀɴᴛɪ ᴍᴘ4.ᴍᴘ4",
    file_tampil: "assets/video/foto.mp4",
    key: "mp4",
    nama_file: "foto.mp4",
    label_sukses: "File",
    label_error: "file",
    jenis: Jenis::Video,
};

#[plugin(
    name = "ganti-mp4.mp4",
    alias = ["gantimp4", "setmp4"],
    category = "owner",
    description = "Ganti video foto.mp4",
    usage = ".ganti-mp4.mp4 (reply/kirim video)",
    example = ".ganti-mp4.mp4",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganti_mp4(ctx: Ctx) -> Reply {
    jalankan(&ctx, &ASET)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{IncomingKind, IncomingMedia, QuotedKind, QuotedMessage};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("ganti-mp4.mp4").unwrap();
        assert_eq!(p.meta.name, "ganti-mp4.mp4");
        assert_eq!(p.meta.alias, ["gantimp4", "setmp4"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Ganti video foto.mp4");
        assert_eq!(p.meta.usage, ".ganti-mp4.mp4 (reply/kirim video)");
        assert_eq!(p.meta.example, ".ganti-mp4.mp4");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private && !p.meta.is_premium);
    }

    #[test]
    fn bukan_media_menampilkan_kartu_pakai() {
        let ctx = Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "ganti-mp4.mp4".into(),
            ..Ctx::default()
        };
        let r = ganti_mp4(ctx);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            super::super::ganti_gambar::pakai(&ASET)
        );
    }

    #[test]
    fn unduhan_gagal_menampilkan_pesan_gagal() {
        let media = if ASET.jenis == Jenis::Gambar {
            IncomingKind::Image
        } else {
            IncomingKind::Video
        };
        let ctx = Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "ganti-mp4.mp4".into(),
            media: Some(IncomingMedia {
                kind: media,
                mimetype: None,
                file_name: None,
                caption: String::new(),
            }),
            media_fetch: Arc::new(oktz_plugin_api::NoMediaFetch),
            ..Ctx::default()
        };
        let r = ganti_mp4(ctx);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            super::super::ganti_gambar::gagal_unduh(&ASET)
        );
    }

    #[test]
    fn quoted_media_dilayani_sama() {
        let kind = if ASET.jenis == Jenis::Gambar {
            QuotedKind::Image
        } else {
            QuotedKind::Video
        };
        let ctx = Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "ganti-mp4.mp4".into(),
            quoted: Some(QuotedMessage {
                id: "1".into(),
                sender: ME.into(),
                chat: ME.into(),
                text: String::new(),
                kind,
                mimetype: None,
                file_name: None,
                from_me: false,
            }),
            media_fetch: Arc::new(oktz_plugin_api::NoMediaFetch),
            ..Ctx::default()
        };
        let r = ganti_mp4(ctx);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            super::super::ganti_gambar::gagal_unduh(&ASET)
        );
    }
}
