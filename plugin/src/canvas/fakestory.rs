//! `plugins/canvas/fakestory.js` — fake Instagram story (kartu 2 gambar).
//!
//! `ponytail:` inti plugin JS adalah render pixel `@napi-rs/canvas` 720x1150
//! (background blur `ctx.filter`, kartu rounded-rect + clip, avatar lingkaran,
//! ikon `Path2D` SVG, teks dengan shadow). `Kanvas` tidak punya compositing
//! gambar, blur, clip, maupun `Path2D` — render 1:1 mustahil tanpa
//! dependensi baru (`Cargo.toml` di luar jangkauan port ini). Yang diport:
//! seluruh alur sebelum render persis JS (username, avatar, cek gambar,
//! unduhan); langkah render jatuh ke cabang `catch` JS (`fakestory.js:241-244`).
//!
//! `ponytail:` reaksi `🕕` (`fakestory.js:204`) hilang — `Reply` hanya membawa
//! satu reaksi; yang dipegang reaksi cabang terakhir (`❌`/`☢`).
//!
//! Helper `aset_pp_kosong`, `get_avatar`, dan `gagal` dipakai bersama
//! `fakestory2/3/4` (satu nama = satu berkas tetap terjaga).

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

/// `fakestory.js:212-218` — kartu saat pesan bukan gambar.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "📷 *ꜰᴀᴋᴇ sᴛᴏʀʏ*\n\n\
         > Reply 1 atau 2 gambar!\n\n\
         > Format: `{prefix}fakestory <nama>`\n\
         > Contoh: `{prefix}fakestory Misaki`"
    )
}

/// `fakestory.js:229` — unduhan gambar kosong/gagal.
pub(crate) const GAGAL_UNDUH: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa download gambar";

/// `getAssetBuffer("pp-kosong")` (`asset-manager.js:46-70`) — baca
/// `config.assets["pp-kosong"]` kalau bukan URL, lewat `ctx.berkas()`.
pub(crate) fn aset_pp_kosong(ctx: &Ctx) -> Option<Vec<u8>> {
    let jalur = ctx.config().assets.get("pp-kosong")?;
    if jalur.starts_with("http") {
        return None;
    }
    let rel = jalur.strip_prefix("./").unwrap_or(jalur);
    ctx.berkas().baca(rel).ok()
}

/// `getAvatarBuffer` (`fakestory.js:190-201`): foto profil → unduh →
/// pp-kosong; kalau semuanya gagal, `throw` → cabang catch.
pub(crate) fn get_avatar(ctx: &Ctx) -> Option<Vec<u8>> {
    if let Ok(Some(url)) = ctx.sock().foto_profil(&ctx.sender, "image")
        && let Ok(res) = ctx.http.get(&url)
    {
        return Some(res.body.bytes().to_vec());
    }
    aset_pp_kosong(ctx)
}

/// `m.args.join(" ").trim() || m.pushName || "User"` (`fakestory.js:203`).
pub(crate) fn username_args(ctx: &Ctx) -> String {
    let gabung = ctx.args.join(" ");
    let nama = gabung.trim();
    if !nama.is_empty() {
        return nama.to_string();
    }
    if !ctx.push_name.is_empty() {
        return ctx.push_name.clone();
    }
    "User".to_string()
}

/// Cabang catch JS (`fakestory.js:241-244`): template error + reaksi `☢`.
pub(crate) fn gagal(ctx: &Ctx) -> Reply {
    Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
}

/// `m.isImage || (m.quoted && m.quoted.isImage)` (`fakestory.js:210`).
pub(crate) fn is_image(ctx: &Ctx) -> bool {
    ctx.is_media_kind(IncomingKind::Image)
        || ctx.quoted().is_some_and(|q| q.kind.is_image())
}

#[plugin(
    name = "fakestory",
    alias = ["fstory", "fakeinsta"],
    category = "canvas",
    description = "Membuat fake Instagram story dengan 2 gambar",
    usage = ".fakestory <nama>",
    example = ".fakestory Misaki (reply 2 gambar)",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn fakestory(ctx: Ctx) -> Reply {
    let _username = username_args(&ctx);

    // `avatarBuffer = await getAvatarBuffer(...)` — kegagalan → catch.
    let Some(_avatar) = get_avatar(&ctx) else {
        return gagal(&ctx);
    };

    if !is_image(&ctx) {
        return Reply::text(help(&ctx.prefix)).with_reaction("❌");
    }

    // `m.isImage && m.download` lebih dulu, baru quoted (`fakestory.js:220-226`);
    // top dan bottom memakai gambar yang sama.
    let top = if ctx.is_media_kind(IncomingKind::Image) {
        ctx.download_media().ok()
    } else if ctx.quoted().is_some_and(|q| q.kind.is_image()) {
        ctx.download_quoted_media().ok()
    } else {
        None
    };
    let Some(top) = top else {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };
    let _bottom = top;

    // Render canvas tidak tersedia — cabang catch JS.
    gagal(&ctx)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpCall, HttpError, HttpResponse, IncomingMedia, MediaError, MediaFetch,
        MediaOrigin, QuotedKind, QuotedMessage, Sock, SockError,
    };
    use oktz_plugin_api::sock::{AdaWhatsapp, AksiPeserta};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubHttp {
        pp: Option<Vec<u8>>,
    }

    impl oktz_plugin_api::HttpGet for StubHttp {
        fn call(&self, _: &HttpCall) -> Result<HttpResponse, HttpError> {
            unreachable!("cuma get")
        }
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            let isi = if url.contains("pp.example") {
                self.pp.clone().unwrap_or_default()
            } else {
                Vec::new()
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(isi),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_multipart(&self, _: &str, _: &[(&str, HttpBody)]) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
    }

    /// Sock dengan foto profil yang bisa disetel.
    #[derive(Debug, Default)]
    struct StubSock {
        foto: Mutex<Option<Result<Option<String>, SockError>>>,
    }

    impl Sock for StubSock {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            self.foto
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| Err(SockError::baru("mati")))
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
    }

    #[derive(Debug)]
    struct Unduh(Result<Vec<u8>, MediaError>);

    impl MediaFetch for Unduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            self.0.clone()
        }
    }

    fn media_gambar() -> Option<IncomingMedia> {
        Some(IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        })
    }

    /// Sock test yang mengembalikan URL pp.example — dipakai juga test
    /// `fakestory2/3/4` supaya cabang setelah avatar terjangkau.
    pub(crate) fn sock_pp_example() -> Arc<dyn Sock> {
        let sock = Arc::new(StubSock::default());
        *sock.foto.lock().unwrap() = Some(Ok(Some("https://pp.example/a.jpg".into())));
        sock
    }

    pub(crate) fn http_pp_example() -> Arc<dyn oktz_plugin_api::HttpGet> {
        Arc::new(StubHttp { pp: Some(b"PP".to_vec()) })
    }

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "fakestory".into(),
            ..Ctx::default()
        }
    }

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("fakestory").unwrap().handler)(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn meta_dan_alias_sesuai_js() {
        let p = oktz_plugin_api::find("fakestory").unwrap();
        assert_eq!(p.meta.name, "fakestory");
        assert_eq!(p.meta.alias, ["fstory", "fakeinsta"]);
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Membuat fake Instagram story dengan 2 gambar");
        assert_eq!(p.meta.usage, ".fakestory <nama>");
        assert_eq!(p.meta.example, ".fakestory Misaki (reply 2 gambar)");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
        for k in ["fakestory", "fstory", "fakeinsta"] {
            assert_eq!(oktz_plugin_api::find(k).unwrap().meta.name, "fakestory");
        }
    }

    #[test]
    fn avatar_gagal_membalas_template_error_dan_radiasi() {
        // SockMati default + aset kosong → getAvatarBuffer throw → catch.
        let r = run(ctx_of("Misaki"));
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn avatar_dari_foto_profil_lalu_bukan_gambar_membalas_help() {
        let mut ctx = ctx_of("Misaki");
        ctx.http = http_pp_example();
        ctx.sock = sock_pp_example();
        let r = run(ctx);
        assert_eq!(teks(&r), help("."));
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn gambar_tersedia_jalan_ke_cabang_catch() {
        let mut ctx = ctx_of("Misaki");
        ctx.http = http_pp_example();
        ctx.sock = sock_pp_example();
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Ok(b"IMG".to_vec())));
        let r = run(ctx);
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakestory"));
        assert_eq!(reaksi(&r), Some("☢".into()));
    }

    #[test]
    fn unduhan_gagal_membalas_pesan_gagal_unduh() {
        let mut ctx = ctx_of("Misaki");
        ctx.http = http_pp_example();
        ctx.sock = sock_pp_example();
        ctx.media = media_gambar();
        ctx.media_fetch = Arc::new(Unduh(Err(MediaError { message: "gagal".into() })));
        let r = run(ctx);
        assert_eq!(teks(&r), GAGAL_UNDUH);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn username_args_mengikuti_rantai_js() {
        let mut ctx = ctx_of("Misaki Extra");
        assert_eq!(username_args(&ctx), "Misaki Extra");
        ctx = ctx_of("");
        ctx.push_name = "Push".into();
        assert_eq!(username_args(&ctx), "Push");
        ctx.push_name = String::new();
        assert_eq!(username_args(&ctx), "User");
        // trim: spasi saja dianggap kosong.
        ctx = ctx_of("   ");
        assert_eq!(username_args(&ctx), "User");
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#fakestory <nama>`"));
    }

    #[test]
    fn aset_pp_kosong_mengikuti_get_asset_buffer() {
        let ctx = ctx_of("");
        // Runtime::default() tanpa kunci → null (seperti JS tanpa kunci).
        assert_eq!(aset_pp_kosong(&ctx), None);
    }
}
