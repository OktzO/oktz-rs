//! `plugins/owner/ganti-v8.jpg.js` — ganti aset v8.jpeg.
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
    emoji_judul: "🖼️",
    judul: "ɢᴀɴᴛɪ ᴠ8.ᴊᴘɢ",
    file_tampil: "assets/image/v9.jpeg",
    key: "v8",
    nama_file: "v8.jpeg",
    label_sukses: "Gambar",
    label_error: "gambar",
    jenis: Jenis::Gambar,
};

#[plugin(
    name = "ganti-v8.jpg",
    alias = ["gantiv8", "setv8"],
    category = "owner",
    description = "Ganti gambar v8.jpg (thumbnail welcome)",
    usage = ".ganti-v8.jpg (reply/kirim gambar)",
    example = ".ganti-v8.jpg",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganti_v8(ctx: Ctx) -> Reply {
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
        let p = oktz_plugin_api::find("ganti-v8.jpg").unwrap();
        assert_eq!(p.meta.name, "ganti-v8.jpg");
        assert_eq!(p.meta.alias, ["gantiv8", "setv8"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Ganti gambar v8.jpg (thumbnail welcome)");
        assert_eq!(p.meta.usage, ".ganti-v8.jpg (reply/kirim gambar)");
        assert_eq!(p.meta.example, ".ganti-v8.jpg");
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
            command: "ganti-v8.jpg".into(),
            ..Ctx::default()
        };
        let r = ganti_v8(ctx);
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
            command: "ganti-v8.jpg".into(),
            media: Some(IncomingMedia {
                kind: media,
                mimetype: None,
                file_name: None,
                caption: String::new(),
            }),
            media_fetch: Arc::new(oktz_plugin_api::NoMediaFetch),
            ..Ctx::default()
        };
        let r = ganti_v8(ctx);
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
            command: "ganti-v8.jpg".into(),
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
        let r = ganti_v8(ctx);
        assert_eq!(
            r.without_mutations().text_body().unwrap(),
            super::super::ganti_gambar::gagal_unduh(&ASET)
        );
    }
}
