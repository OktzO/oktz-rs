//! `plugins/sticker/stickerpack.js` — cari dan kirim sticker pack dari
//! getstickerpack.com.
//!
//! Yang diport apa adanya:
//! - Kartu pakai (`stickerpack.js:95-105`).
//! - Pencarian pack: POST `https://getstickerpack.com/api/v1/stickerdb/search`
//!   dengan `{ query, page }` (`stickerpack.js:29-49`), lalu detail pack dari
//!   `https://getstickerpack.com/api/v1/stickerdb/stickers/<id>`
//!   (`stickerpack.js:51-67`).
//! - Semua pesan: pack tidak ditemukan, detail gagal, jumlah sticker, gagal
//!   unduh, pack gagal → kirim satu per satu, berhasil kirim, gagal kirim
//!   (`stickerpack.js:114-206`).
//!
//! `ponytail:`
//! - `sock.sendStickerPack` tidak ada di `Reply`, jadi sticker dikirim satu per
//!   satu lewat `Reply::sequence`. Konsekuensinya pesan "Pack gagal, mengirim
//!   satu per satu..." (percabangan `catch` JS) tidak pernah muncul — paket
//!   selalu dikirim per satu.
//! - `sharp(...).resize(512, 512, { fit: "contain", background: transparent })`
//!   tidak punya padanan persis di `oktz_plugin_api::gambar` (hanya `cover`
//!   dan `inside`); gambar dikonversi ke WebP apa adanya.
//! - `addExifToWebp` tidak ada, jadi metadata packname/author tidak ikut.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, plugin};
use serde_json::Value;

/// `stickerpack.js:70` — batas sticker per pack.
pub(crate) const MAX_STICKERS: usize = 20;

/// `stickerpack.js:95-105` — kartu pakai.
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "── .✦ 𝗦𝗧𝗜𝗖𝗞𝗘𝗥 𝗣𝗔𝗖𝗞 ✦. ── 𝜗ৎ\n\n\
         Cari dan kirim sticker pack!\n\n\
         ╭─〔 Cara Pakai 〕───⬣\n\
         │  ✦ {prefix}stickerpack <query>\n\
         ╰──────────────⬣\n\n\
         *{prefix}stickerpack anime*\n\
         *{prefix}stickerpack cat*\n\n\
         .☘︎ ݁˖"
    )
}

/// `stickerpack.js:117` — pack tidak ditemukan.
pub(crate) fn tidak_ditemukan(query: &str) -> String {
    format!("── .✦ ──\n\n> Tidak ada sticker pack untuk: *{query}* .☘︎ ݁˖")
}

/// `stickerpack.js:127` — detail pack gagal diambil.
pub(crate) const DETAIL_GAGAL: &str = "── .✦ ──\n\n> Gagal mengambil detail sticker pack .☘︎ ݁˖";

/// `stickerpack.js:131` — sedang mengunduh pack.
pub(crate) fn mengunduh(nama: &str, jumlah: usize) -> String {
    format!(
        "── .✦ ──\n\n> Mengunduh *{nama}*\n> {jumlah} sticker .☘︎ ݁˖"
    )
}

/// `stickerpack.js:150` — gagal mengunduh sticker.
pub(crate) const UNDUH_GAGAL: &str = "── .✦ ──\n\n> Gagal mendownload sticker .☘︎ ݁˖";

/// `stickerpack.js:169` — pack gagal, kirim satu per satu.
pub(crate) const PACK_GAGAL: &str = "── .✦ ──\n\n> Pack gagal, mengirim satu per satu... .☘︎ ݁˖";

/// `stickerpack.js:201` — berhasil mengirim sebagian sticker.
pub(crate) fn berhasil_kirim(jumlah: usize, packname: &str) -> String {
    format!(
        "── .✦ ──\n\n> Berhasil kirim *{jumlah}* sticker dari *{packname}* .☘︎ ݁˖"
    )
}

/// `stickerpack.js:205` — gagal mengirim sticker.
pub(crate) const KIRIM_GAGAL: &str = "── .✦ ──\n\n> Gagal mengirim sticker .☘︎ ݁˖";

/// `stickerpack.js:29-49` — daftar hasil pencarian.
pub(crate) fn cari_pack(body: &Value) -> Vec<(String, String)> {
    let Some(data) = body.get("data").and_then(Value::as_array) else {
        return Vec::new();
    };
    data.iter()
        .map(|item| {
            let title = item
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let slug = item
                .get("slug")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            (title, slug)
        })
        .collect()
}

/// `stickerpack.js:56` — ID pack diambil dari URL `.../stickers/<id>`.
pub(crate) fn id_dari_url(url: &str) -> String {
    match url.rfind("/stickers/") {
        Some(pos) => url[pos + "/stickers/".len()..].to_string(),
        None => url.to_string(),
    }
}

/// `stickerpack.js:58-62` — daftar sticker (URL gambar) dari detail pack.
pub(crate) fn sticker_dari_detail(body: &Value) -> Vec<String> {
    let Some(images) = body
        .get("data")
        .and_then(|d| d.get("images"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    images
        .iter()
        .filter_map(|item| {
            let url = item.get("url").and_then(Value::as_str)?;
            Some(format!("https://s3.getstickerpack.com/{url}"))
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
    name = "stickerpack",
    alias = ["sp", "stickersearch", "searchsticker"],
    category = "sticker",
    description = "Cari dan kirim sticker pack",
    usage = ".stickerpack <query>",
    example = ".stickerpack anime",
    cooldown = 20,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn stickerpack(ctx: Ctx) -> Reply {
    let query = ctx.args.join(" ").trim().to_string();
    if query.is_empty() {
        return Reply::text(pakai(&ctx.prefix));
    }

    // `stickerpack.js:32-37` — pencarian.
    let Ok(res) = ctx.http.call(
        &HttpCall::post("https://getstickerpack.com/api/v1/stickerdb/search")
            .json(&serde_json::json!({ "query": query, "page": 1 })),
    ) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("✘");
    };
    let body = res.json().unwrap_or(Value::Null);
    let packs = cari_pack(&body);
    if packs.is_empty() {
        return Reply::text(tidak_ditemukan(&query)).with_reaction("✘");
    }

    // `stickerpack.js:121-123` — pilih satu pack secara acak.
    let indeks = crate::common::random_index(packs.len());
    let (nama, slug) = packs[indeks].clone();

    // `stickerpack.js:51-67` — detail pack.
    let Ok(res) = ctx.http.get(&format!(
        "https://getstickerpack.com/api/v1/stickerdb/stickers/{}",
        id_dari_url(&format!("https://getstickerpack.com/stickers/{slug}"))
    )) else {
        return Reply::text(DETAIL_GAGAL).with_reaction("✘");
    };
    let detail = res.json().unwrap_or(Value::Null);
    let urls = sticker_dari_detail(&detail);
    if urls.is_empty() {
        return Reply::text(DETAIL_GAGAL).with_reaction("✘");
    }

    // `stickerpack.js:130-132` — laporan sedang mengunduh.
    let jumlah = urls.len().min(MAX_STICKERS);
    let laporan = Reply::text(mengunduh(&nama, jumlah));

    // `stickerpack.js:137-146` — unduh sticker (maksimal 20).
    let mut sticker_bytes: Vec<Vec<u8>> = Vec::new();
    for url in urls.iter().take(MAX_STICKERS) {
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

    // `stickerpack.js:156-160` — JS mengirim paket; di sini satu per satu.
    let packname = if nama.is_empty() {
        "Sticker.ly Pack".to_string()
    } else {
        nama.clone()
    };
    let mut balasan: Vec<Reply> = vec![laporan];
    for bytes in sticker_bytes {
        balasan.push(Reply::sticker(MediaSource::bytes(bytes)));
    }
    balasan.push(Reply::text(berhasil_kirim(MAX_STICKERS.min(jumlah), &packname)));
    // `ponytail:` reaksi ✓ tidak bisa menempel pada `Reply::Sequence` — lihat
    // catatan di atas.
    Reply::sequence(balasan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<(String, Vec<u8>)>>,
    }

    impl Stub {
        fn baru() -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.balasan.lock().unwrap().push((url.to_string(), vec![]));
            let body = if url.contains("stickerdb/stickers/") {
                br#"{"data":{"title":"Pack","images":[{"url":"a.webp"},{"url":"b.webp"}]}}"#.to_vec()
            } else {
                b"{}".to_vec()
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
            self.balasan.lock().unwrap().push((req.url.clone(), vec![]));
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(
                        br#"{"data":[{"title":"Pack Keren","slug":"pack-keren"}]}"#.to_vec(),
                    ),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    fn ctx_of(http: Arc<Stub>, args: &[&str]) -> Ctx {
        Ctx {
            text: args.join(" "),
            args: args.iter().map(|a| (*a).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "stickerpack".into(),
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
            "── .✦ ──\n\n> Tidak ada sticker pack untuk: *kucing* .☘︎ ݁˖"
        );
        assert_eq!(
            DETAIL_GAGAL,
            "── .✦ ──\n\n> Gagal mengambil detail sticker pack .☘︎ ݁˖"
        );
        assert_eq!(
            mengunduh("Pack Keren", 5),
            "── .✦ ──\n\n> Mengunduh *Pack Keren*\n> 5 sticker .☘︎ ݁˖"
        );
        assert_eq!(
            UNDUH_GAGAL,
            "── .✦ ──\n\n> Gagal mendownload sticker .☘︎ ݁˖"
        );
        assert_eq!(
            PACK_GAGAL,
            "── .✦ ──\n\n> Pack gagal, mengirim satu per satu... .☘︎ ݁˖"
        );
        assert_eq!(
            berhasil_kirim(3, "Pack Keren"),
            "── .✦ ──\n\n> Berhasil kirim *3* sticker dari *Pack Keren* .☘︎ ݁˖"
        );
        assert_eq!(
            KIRIM_GAGAL,
            "── .✦ ──\n\n> Gagal mengirim sticker .☘︎ ݁˖"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("stickerpack").unwrap();
        assert_eq!(p.meta.name, "stickerpack");
        assert_eq!(p.meta.alias, ["sp", "stickersearch", "searchsticker"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(p.meta.description, "Cari dan kirim sticker pack");
        assert_eq!(p.meta.usage, ".stickerpack <query>");
        assert_eq!(p.meta.example, ".stickerpack anime");
        assert_eq!(p.meta.cooldown, 20);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_plugin_ini() {
        for a in ["sp", "stickersearch", "searchsticker"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "stickerpack");
        }
    }

    #[test]
    fn id_pack_dari_url() {
        assert_eq!(
            id_dari_url("https://getstickerpack.com/stickers/pack-keren"),
            "pack-keren"
        );
        assert_eq!(id_dari_url("pack-keren"), "pack-keren");
    }

    #[test]
    fn tanpa_query_menampilkan_kartu_pakai() {
        let r = stickerpack(ctx_of(Stub::baru(), &[]));
        assert_eq!(teks(&r), pakai("."));
        assert!(teks(&r).contains("╭─〔 Cara Pakai 〕───⬣"), "{}", teks(&r));
    }
}
