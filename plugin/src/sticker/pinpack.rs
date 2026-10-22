//! `plugins/sticker/pinpack.js` — cari gambar Pinterest lalu jadikan sticker pack.
//!
//! Yang diport apa adanya:
//! - Kartu pakai (`pinpack.js:54-63`).
//! - Pencarian gambar: GET `https://api.siputzx.my.id/api/s/pinterest?query=<query>`
//!   (`pinpack.js:69`) — query-nya TIDAK di-encode di JS, jadi ditiru apa
//!   adanya.
//! - Semua pesan: hasil kosong, laporan mengunduh, gagal unduh, pack gagal →
//!   kirim satu per satu, berhasil kirim, gagal kirim (`pinpack.js:72-155`).
//!
//! `ponytail:`
//! - `sock.sendStickerPack` tidak ada di `Reply`, jadi sticker dikirim satu per
//!   satu lewat `Reply::sequence`; percabangan "Pack gagal, mengirim satu per
//!   satu..." (`:115-119`) tidak pernah tersentuh.
//! - `sharp(...).resize(512, 512, { fit: "contain", background: transparent })`
//!   tidak punya padanan persis di `oktz_plugin_api::gambar`; gambar
//!   dikonversi ke WebP apa adanya.
//! - `addExifToWebp` tidak ada, jadi metadata packname/author tidak ikut.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

/// `pinpack.js:13` — batas gambar per pack.
pub(crate) const MAX_STICKERS: usize = 20;

/// `pinpack.js:54-63` — kartu pakai.
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "── .✦ 𝗣𝗜𝗇 𝗣𝗔𝗖𝗞 ✦. ── 𝜗ৎ\n\n\
         Cari gambar Pinterest → jadikan sticker pack!\n\n\
         ╭─〔 Cara Pakai 〕───⬣\n\
         │  ✦ {prefix}pinpack <query>\n\
         ╰──────────────⬣\n\n\
         *{prefix}pinpack anime cat*\n\
         *{prefix}pinpack aesthetic*\n\n\
         .☘︎ ݁˖"
    )
}

/// `pinpack.js:74` — tidak ada hasil.
pub(crate) fn tidak_ditemukan(query: &str) -> String {
    format!("── .✦ ──\n\n> Tidak ditemukan hasil untuk: *{query}* .☘︎ ݁˖")
}

/// `pinpack.js:78` — laporan mengunduh.
pub(crate) fn mengunduh(jumlah: usize) -> String {
    format!(
        "── .✦ ──\n\n> Mengunduh *{jumlah}* gambar dari Pinterest\n> Lalu dikonversi ke sticker pack... .☘︎ ݁˖"
    )
}

/// `pinpack.js:100` — gagal mengunduh gambar.
pub(crate) const UNDUH_GAGAL: &str = "── .✦ ──\n\n> Gagal mendownload gambar .☘︎ ݁˖";

/// `pinpack.js:118` — pack gagal, kirim satu per satu.
pub(crate) const PACK_GAGAL: &str = "── .✦ ──\n\n> Pack gagal, mengirim satu per satu... .☘︎ ݁˖";

/// `pinpack.js:150` — berhasil mengirim sebagian sticker.
pub(crate) fn berhasil_kirim(jumlah: usize, packname: &str) -> String {
    format!(
        "── .✦ ──\n\n> Berhasil kirim *{jumlah}* sticker dari *{packname}* .☘︎ ݁˖"
    )
}

/// `pinpack.js:154` — gagal mengirim sticker.
pub(crate) const KIRIM_GAGAL: &str = "── .✦ ──\n\n> Gagal mengirim sticker .☘︎ ݁˖";

/// `pinpack.js:69` — endpoint pencarian.
pub(crate) fn url_pinterest(query: &str) -> String {
    format!("https://api.siputzx.my.id/api/s/pinterest?query={query}")
}

/// `pinpack.js:70` — daftar URL gambar dari hasil pencarian.
pub(crate) fn url_gambar(body: &Value) -> Vec<String> {
    let Some(data) = body.get("data").and_then(Value::as_array) else {
        return Vec::new();
    };
    data.iter()
        .filter_map(|item| {
            let url = item.get("image_url").and_then(Value::as_str)?;
            (!url.is_empty()).then(|| url.to_string())
        })
        .collect()
}

/// Ubah gambar jadi WebP — padanan `.webp({ quality: 80 })`.
fn ke_webp(media: &[u8]) -> Option<Vec<u8>> {
    oktz_plugin_api::gambar::sharp(media)
        .ok()
        .and_then(|g| g.ke_webp(80).ok())
}

#[plugin(
    name = "pinpack",
    alias = ["ppack", "pinsticker", "pinsearchpack"],
    category = "sticker",
    description = "Cari gambar Pinterest lalu jadikan sticker pack",
    usage = ".pinpack <query>",
    example = ".pinpack cat",
    cooldown = 20,
    energi = 3,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn pinpack(ctx: Ctx) -> Reply {
    let query = ctx.args.join(" ").trim().to_string();
    if query.is_empty() {
        return Reply::text(pakai(&ctx.prefix));
    }

    // `pinpack.js:69` — query tidak di-encode di JS.
    let Ok(res) = ctx.http.get(&url_pinterest(&query)) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("✘");
    };
    let body = res.json().unwrap_or(Value::Null);
    let hasil = url_gambar(&body);
    let hasil: Vec<String> = hasil.into_iter().take(MAX_STICKERS).collect();
    if hasil.is_empty() {
        return Reply::text(tidak_ditemukan(&query)).with_reaction("✘");
    }

    // `pinpack.js:77-79` — laporan mengunduh.
    let laporan = Reply::text(mengunduh(hasil.len()));

    // `pinpack.js:83-95` — unduh gambar lalu konversi ke WebP.
    let mut sticker_bytes: Vec<Vec<u8>> = Vec::new();
    for url in &hasil {
        let Ok(res) = ctx.http.get(url) else {
            continue;
        };
        let raw = res.body.bytes().to_vec();
        let webp = ke_webp(&raw).unwrap_or(raw);
        sticker_bytes.push(webp);
    }

    if sticker_bytes.is_empty() {
        // `ponytail:` `Reply::Sequence` tidak boleh dibungkus `Reaction`
        // (pipeline merender `Sequence` lewat loop, bukan `render_reply`), jadi
        // reaksi ✘ tidak ikut — sama seperti `download/tiktokdl2.rs`.
        return Reply::sequence(vec![laporan, Reply::text(UNDUH_GAGAL)]);
    }

    // `pinpack.js:102-114` — JS mengirim paket; di sini satu per satu.
    let packname = format!("Pinterest: {query}");
    let mut balasan: Vec<Reply> = vec![laporan];
    for bytes in sticker_bytes {
        balasan.push(Reply::sticker(MediaSource::bytes(bytes)));
    }
    balasan.push(Reply::text(berhasil_kirim(hasil.len(), &packname)));
    // `ponytail:` reaksi ✓ tidak bisa menempel pada `Reply::Sequence` — lihat
    // catatan di atas.
    Reply::sequence(balasan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Stub {
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn baru() -> Arc<Stub> {
            Arc::new(Stub {
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let body = if url.contains("siputzx") {
                br#"{"data":[{"image_url":"https://img.example/a.jpg"}]}"#.to_vec()
            } else {
                b"IMG".to_vec()
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(body),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.get(&req.url)
        }
    }

    fn ctx_of(http: Arc<Stub>, args: &[&str]) -> Ctx {
        Ctx {
            text: args.join(" "),
            args: args.iter().map(|a| (*a).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "pinpack".into(),
            http,
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or("").to_string()
    }

    #[test]
    fn output_sama_dengan_js() {
        assert_eq!(
            tidak_ditemukan("kucing"),
            "── .✦ ──\n\n> Tidak ditemukan hasil untuk: *kucing* .☘︎ ݁˖"
        );
        assert_eq!(
            mengunduh(3),
            "── .✦ ──\n\n> Mengunduh *3* gambar dari Pinterest\n> Lalu dikonversi ke sticker pack... .☘︎ ݁˖"
        );
        assert_eq!(
            UNDUH_GAGAL,
            "── .✦ ──\n\n> Gagal mendownload gambar .☘︎ ݁˖"
        );
        assert_eq!(
            PACK_GAGAL,
            "── .✦ ──\n\n> Pack gagal, mengirim satu per satu... .☘︎ ݁˖"
        );
        assert_eq!(
            berhasil_kirim(2, "Pinterest: kucing"),
            "── .✦ ──\n\n> Berhasil kirim *2* sticker dari *Pinterest: kucing* .☘︎ ݁˖"
        );
        assert_eq!(
            KIRIM_GAGAL,
            "── .✦ ──\n\n> Gagal mengirim sticker .☘︎ ݁˖"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("pinpack").unwrap();
        assert_eq!(p.meta.name, "pinpack");
        assert_eq!(p.meta.alias, ["ppack", "pinsticker", "pinsearchpack"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(
            p.meta.description,
            "Cari gambar Pinterest lalu jadikan sticker pack"
        );
        assert_eq!(p.meta.usage, ".pinpack <query>");
        assert_eq!(p.meta.example, ".pinpack cat");
        assert_eq!(p.meta.cooldown, 20);
        assert_eq!(p.meta.energi, 3);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_plugin_ini() {
        for a in ["ppack", "pinsticker", "pinsearchpack"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "pinpack");
        }
    }

    #[test]
    fn url_pinterest_tidak_mengencode_query() {
        assert_eq!(
            url_pinterest("anime cat"),
            "https://api.siputzx.my.id/api/s/pinterest?query=anime cat"
        );
    }

    #[test]
    fn tanpa_query_menampilkan_kartu_pakai() {
        let r = pinpack(ctx_of(Stub::baru(), &[]));
        assert_eq!(teks(&r), pakai("."));
        assert!(teks(&r).contains("Cari gambar Pinterest"), "{}", teks(&r));
    }
}
