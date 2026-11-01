//! `plugins/canvas/fakedev7.js` — fake developer (Python Theme).
//!
//! `ponytail:` inti plugin JS adalah render pixel `@napi-rs/canvas` 1080x1080
//! (background kode Python acak, gradien radial, avatar lingkaran dari gambar
//! yang diunduh, teks melingkar, badge crown/leaf/blue tick). `Kanvas`
//! (`oktz_plugin_api::kanvas`) tidak punya compositing gambar/gradien radial
//! terhadap gambar/font ganda, dan `Gambar` tidak membuka akses piksel mentah
//! — render 1:1 mustahil tanpa dependensi baru (`Cargo.toml` di luar jangkauan
//! port ini). Yang diport: seluruh cabang sebelum render persis JS (help,
//! sumber avatar, pp-kosong); langkah render jatuh ke cabang `catch` JS
//! (`fakedev7.js:171-174`).
//!
//! `ponytail:` reaksi `🕕` (`fakedev7.js:43`) hilang — `Reply` hanya membawa
//! satu reaksi; yang dipegang reaksi catch (`❌`).

use oktz_plugin_api::{Ctx, IncomingKind, Reply, plugin};

/// `fakedev7.js:26` — kartu saat nama kosong.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🎮 *ꜰᴀᴋᴇ ᴅᴇᴠᴇʟᴏᴘᴇʀ 7*\n\n> Masukkan nama untuk profile\n\n\
         *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ:*\n\
         > 1. Kirim foto + caption `{prefix}fakedev7 <nama>`\n\
         > 2. Reply foto dengan `{prefix}fakedev7 <nama>`"
    )
}

/// `fakedev7.js:41` — dipakai kalau buffer tetap kosong setelah fallback.
pub(crate) const AVATAR_KOSONG: &str = "❌ Kirim/reply gambar untuk dijadikan avatar!";

/// `fakedev7.js:173` — pesan cabang `catch`.
pub(crate) const GAGAL: &str = "❌ Terjadi kesalahan saat memproses gambar.";

/// `getAssetBuffer("pp-kosong")` (`asset-manager.js:46-70`) — baca
/// `config.assets["pp-kosong"]` kalau bukan URL, lewat `ctx.berkas()`.
/// Cache JS sengaja tidak ditiru (di Rust setiap panggilan membaca ulang).
pub(crate) fn aset_pp_kosong(ctx: &Ctx) -> Option<Vec<u8>> {
    let jalur = ctx.config().assets.get("pp-kosong")?;
    if jalur.starts_with("http") {
        return None;
    }
    let rel = jalur.strip_prefix("./").unwrap_or(jalur);
    ctx.berkas().baca(rel).ok()
}

/// Sumber avatar (`fakedev7.js:28-40`): quoted image → trigger image →
/// foto profil → aset pp-kosong.
///
/// `Err(())` untuk dua bentuk kegagalan yang di JS berbeda jalur:
/// unduhan quoted/trigger yang gagal langsung `return m.reply(te(...))`
/// (`fakedev7.js:30,32`), sedangkan kegagalan jalur foto profil jatuh ke
/// pp-kosong dan `!buffer` bila asetnya tidak ada (`fakedev7.js:41`).
pub(crate) enum Avatar {
    Bytes(Vec<u8>),
    UnduhanGagal,
    TidakAda,
}

pub(crate) fn avatar(ctx: &Ctx) -> Avatar {
    if let Some(q) = &ctx.quoted
        && q.kind.is_image()
    {
        return match ctx.download_quoted_media() {
            Ok(b) => Avatar::Bytes(b),
            Err(_) => Avatar::UnduhanGagal,
        };
    }
    if ctx.is_media_kind(IncomingKind::Image) {
        return match ctx.download_media() {
            Ok(b) => Avatar::Bytes(b),
            Err(_) => Avatar::UnduhanGagal,
        };
    }
    // Jalur foto profil: kegagalan `profilePictureUrl` maupun unduhan
    // ditelan JS (`fakedev7.js:34-39`) → pp-kosong.
    if let Ok(Some(url)) = ctx.sock().foto_profil(&ctx.sender, "image")
        && let Ok(res) = ctx.http.get(&url)
    {
        return Avatar::Bytes(res.body.bytes().to_vec());
    }
    match aset_pp_kosong(ctx) {
        Some(b) => Avatar::Bytes(b),
        None => Avatar::TidakAda,
    }
}

#[plugin(
    name = "fakedev7",
    alias = [],
    category = "canvas",
    description = "Membuat fake developer (Python Theme)",
    usage = ".fakedev7 <nama> (reply/kirim foto)",
    example = ".fakedev7 Misaki",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn fakedev7(ctx: Ctx) -> Reply {
    // `const name = m.text?.trim()` (`fakedev7.js:24`).
    let nama = ctx.text.trim();
    if nama.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    let buffer = match avatar(&ctx) {
        Avatar::Bytes(b) => b,
        Avatar::UnduhanGagal => {
            // `catch (e) { return m.reply(te(m.prefix, m.command, m.pushName)) }`
            // (`fakedev7.js:30,32`) — sebelum `m.react("🕕")`, jadi tanpa reaksi.
            return Reply::text(crate::common::error_template(&ctx));
        }
        Avatar::TidakAda => return Reply::text(AVATAR_KOSONG),
    };
    let _ = buffer;

    // Render canvas tidak tersedia — cabang catch JS (`fakedev7.js:171-174`).
    Reply::text(GAGAL).with_reaction("❌")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{
        HttpBody, HttpCall, HttpError, HttpResponse, MediaError, MediaFetch, MediaOrigin,
        QuotedKind, QuotedMessage, Sock, SockError,
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
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            let isi = if req.url.contains("pp.example") {
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
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!()
        }
    }

    #[derive(Debug, Default)]
    struct StubSock {
        foto: Mutex<Option<Result<Option<String>, SockError>>>,
    }

    impl Sock for StubSock {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            self.foto.lock().unwrap().clone().unwrap_or(Err(SockError::baru("mati")))
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

    #[derive(Debug, Default)]
    struct Unduh {
        ok: bool,
    }

    impl MediaFetch for Unduh {
        fn download(&self, _: MediaOrigin) -> Result<Vec<u8>, MediaError> {
            if self.ok {
                Ok(b"AVATAR".to_vec())
            } else {
                Err(MediaError {
                    message: "gagal".into(),
                })
            }
        }
    }

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "fakedev7".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Ctx::default()
        }
    }

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("fakedev7").unwrap().handler)(ctx)
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
    fn meta_sesuai_config_js() {
        let p = oktz_plugin_api::find("fakedev7").unwrap();
        assert_eq!(p.meta.name, "fakedev7");
        assert!(p.meta.alias.is_empty());
        assert_eq!(p.meta.category, "canvas");
        assert_eq!(p.meta.description, "Membuat fake developer (Python Theme)");
        assert_eq!(p.meta.usage, ".fakedev7 <nama> (reply/kirim foto)");
        assert_eq!(p.meta.example, ".fakedev7 Misaki");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn nama_kosong_mengirim_help_tanpa_reaksi() {
        for text in ["", "   "] {
            let r = run(ctx_of(text));
            assert_eq!(teks(&r), help("."), "text {text:?}");
            assert_eq!(reaksi(&r), None);
        }
    }

    #[test]
    fn quoted_gambar_sukses_jalan_ke_catch_render() {
        let mut ctx = ctx_of("Misaki");
        ctx.quoted = Some(QuotedMessage {
            id: "q1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: ME.into(),
            text: String::new(),
            kind: QuotedKind::Image,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        });
        ctx.media_fetch = Arc::new(Unduh { ok: true });
        let r = run(ctx);
        assert_eq!(teks(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn unduhan_gagal_membalas_error_template_tanpa_reaksi() {
        let mut ctx = ctx_of("Misaki");
        ctx.media = Some(oktz_plugin_api::IncomingMedia {
            kind: IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        });
        ctx.media_fetch = Arc::new(Unduh { ok: false });
        let r = run(ctx);
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "fakedev7"));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn tanpa_gambar_dan_tanpa_pp_aset_membalas_pesan_avatar() {
        // SockMati default + BerkasPalsu default → pp gagal → pp-kosong tidak ada.
        let r = run(ctx_of("Misaki"));
        assert_eq!(teks(&r), AVATAR_KOSONG);
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn foto_profil_sukses_dipakai_sebagai_avatar() {
        let http = Arc::new(StubHttp {
            pp: Some(b"PP".to_vec()),
        });
        let sock = Arc::new(StubSock::default());
        *sock.foto.lock().unwrap() = Some(Ok(Some("https://pp.example/a.jpg".into())));
        let mut ctx = ctx_of("Misaki");
        ctx.http = http;
        ctx.sock = sock;
        let r = run(ctx);
        assert_eq!(teks(&r), GAGAL);
        assert_eq!(reaksi(&r), Some("❌".into()));
    }

    #[test]
    fn aset_pp_kosong_dibaca_dari_config_assets() {
        let mut ctx = ctx_of("Misaki");
        // Runtime::default() di test tidak punya kunci aset → getAssetBuffer
        // JS dengan config tanpa kunci juga mengembalikan null.
        assert_eq!(aset_pp_kosong(&ctx), None);

        // Kunci yang menunjuk URL diabaikan getAssetBuffer JS (`!startsWith("http")`).
        let mut rt = ctx.config().clone();
        rt.assets
            .0
            .insert("pp-kosong".into(), "https://cdn.example/pp.png".into());
        ctx.config = Arc::new(rt);
        assert_eq!(aset_pp_kosong(&ctx), None);

        // Kunci berkas → dibaca lewat `ctx.berkas()`; BerkasPalsu kosong → None,
        // jadi jalur baca sukses hanya tercakup di produksi (BerkasNyata).
        let mut rt = ctx.config().clone();
        rt.assets
            .0
            .insert("pp-kosong".into(), "./assets/image/pp-kosong.jpeg".into());
        ctx.config = Arc::new(rt);
        assert_eq!(aset_pp_kosong(&ctx), None);
        assert!(ctx.config().assets.get("pp-kosong").is_some());
    }

    #[test]
    fn help_menggunakan_prefix_dari_ctx() {
        assert!(help("#").contains("`#fakedev7 <nama>`"));
    }
}
