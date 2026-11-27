//! Kirim foto dari Pinterest sesuai tipe.
//!
//! `Keterbatasan:` tombol `nativeFlowMessage`, `viewOnceMessage`, `prepareWAMessageMedia`,
//! dan `sock.relayMessage` dari versi awal tidak punya padanan di `Reply` — jadi yang
//! dikirim cuma gambarnya, tanpa kartu interaktif.

#![allow(dead_code)] // `GAGAL_MENYIAPKAN` cuma dipakai test (tak ada prepare media)

use oktz_plugin_api::{Ctx, HttpCall, HttpGet, MediaSource, Reply, plugin};
use oktz_scraper::rinn;

/// `QUERIES` — urutan query per tipe; yang pertama
/// menjawab dengan pin dipakai.
pub(crate) const QUERIES_CEWE: [&str; 3] = [
    "cewe cantik indonesia",
    "gadis cantik",
    "wanita indonesia cantik",
];
pub(crate) const QUERIES_COWO: [&str; 3] = [
    "cowo ganteng indonesia",
    "pria tampan",
    "cowok ganteng",
];
pub(crate) const QUERIES_FEMBOY: [&str; 3] = ["femboy", "femboy anime", "femboy aesthetic"];

/// `VALID_TYPES = Object.keys(QUERIES)` — urutan itu penting
/// karena juga menjadi daftar yang dicetak di pesan error.
pub(crate) const VALID_TYPES: [&str; 3] = ["cewe", "cowo", "femboy"];

/// `QUERIES[type] || []`.
pub(crate) fn queries_for(tipe: &str) -> &'static [&'static str] {
    match tipe {
        "cewe" => &QUERIES_CEWE,
        "cowo" => &QUERIES_COWO,
        "femboy" => &QUERIES_FEMBOY,
        _ => &[],
    }
}

/// Guard argumen. `m.args?.[0]` di-`trim().toLowerCase()`,
/// jadi spasi excess dan huruf besar tidak menggagalkan.
pub(crate) fn pesan_tipe_invalid() -> String {
    "❌ Pilih salah satu tipe pap yang tersedia: `cewe`, `cowo`, atau `femboy`.\n\nContoh: `.pap cewe`".to_string()
}

/// Hasil kosong yang sah.
pub(crate) fn kosong(arg: &str) -> String {
    format!("❌ Waduh, pap {arg} lagi kosong nih. Coba lagi nanti.")
}

/// Tidak ada satu pun URL yang bisa diunduh.
pub(crate) const GAMBAR_TIDAK_TERSEDIA: &str = "⚠️ Gambar tidak tersedia. Coba lagi nanti.";

/// `prepareWAMessageMedia` tidak mengembalikan `imageMessage`.
pub(crate) const GAGAL_MENYIAPKAN: &str = "⚠️ Gagal menyiapkan gambar. Coba lagi nanti.";

/// Ambang ukuran minimum, terpisah dari magic number supaya
/// both bisa diuji tanpa buffer sungguhan.
pub(crate) const MIN_BYTES: usize = 1000;

/// `isImage(buf)` — magic number JPEG/PNG/WebP.
pub(crate) fn is_image(buf: &[u8]) -> bool {
    if buf.len() < 4 {
        return false;
    }
    // JPEG: FF D8 FF
    if buf[0] == 0xff && buf[1] == 0xd8 && buf[2] == 0xff {
        return true;
    }
    // PNG: 89 50 4E 47
    if buf[0] == 0x89 && buf[1] == 0x50 && buf[2] == 0x4e && buf[3] == 0x47 {
        return true;
    }
    // WebP: "RIFF".... "WEBP"
    if buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return true;
    }
    false
}

/// Gabungan kedua syarat.
pub(crate) fn buffer_terima(buf: &[u8]) -> bool {
    buf.len() > MIN_BYTES && is_image(buf)
}

/// `fetchImageBuffer` — Pinterest menolak tanpa `Referer`.
pub(crate) fn download_request(url: &str) -> HttpCall {
    HttpCall::get(url).header("Referer", "https://www.pinterest.com/")
}

/// `fetchImageUrls` — loop query per tipe lewat
/// `rinn::pinterest`; query pertama yang mengembalikan pin dipakai.
///
/// `Err` hanya kalau tidak ada satu pun query yang menjawab. Query
/// menjawab tapi kosong tetap dianggap hasil kosong yang sah (`:115-116`),
/// supaya user tidak disuruh retry ke host yang baru saja gagal.
fn fetch_urls(tipe: &str, http: &dyn HttpGet) -> Result<Vec<String>, ()> {
    let mut ada_menjawab = false;
    let mut ada_gagal = false;
    for &q in queries_for(tipe) {
        match rinn::pinterest(http, q, 5) {
            Ok(urls) if !urls.is_empty() => return Ok(urls),
            Ok(_) => ada_menjawab = true,
            Err(_) => ada_gagal = true,
        }
    }
    if ada_gagal && !ada_menjawab {
        Err(())
    } else {
        Ok(Vec::new())
    }
}

/// Acak urutan, unduh satu per satu, terima buffer pertama
/// yang lolos [`buffer_terima`]. Pinterest butuh `Referer` ([`download_request`]).
fn ambil_gambar(http: &dyn HttpGet, urls: &[String]) -> Option<Vec<u8>> {
    for url in acak(urls) {
        let Ok(res) = http.call(&download_request(url)) else {
            continue;
        };
        let buf = res.body.bytes().to_vec();
        if buffer_terima(&buf) {
            return Some(buf);
        }
    }
    None
}

/// Fisher-Yates untuk `sort(() => Math.random() - 0.5)`.
fn acak(urls: &[String]) -> Vec<&str> {
    use rand::Rng as _;
    let mut out: Vec<&str> = urls.iter().map(String::as_str).collect();
    let mut rng = rand::rng();
    for i in (1..out.len()).rev() {
        let j = rng.random_range(0..=i);
        out.swap(i, j);
    }
    out
}

#[plugin(
    name = "pap",
    alias = ["papcewe", "papcowo", "papfemboy"],
    category = "search",
    description = "Minta pap cewe, cowo, atau femboy dari Pinterest",
    usage = ".pap <cewe/cowo/femboy>",
    example = ".pap cewe",
    cooldown = 5,
    energi = 1,
    is_enabled = true,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn pap(ctx: Ctx) -> Reply {
    // `typeof raw === "string" ? raw.trim().toLowerCase(): ""` (`:143-144`).
    // `m.args` di `Ctx` selalu `Vec<String>`, jadi non-string tidak mungkin.
    let arg = ctx.args.first().map(|a| a.trim().to_lowercase()).unwrap_or_default();

    if !VALID_TYPES.contains(&arg.as_str()) {
        // `safeReply` menelan error, jadi balasan ini selalu terkirim.
        return Reply::text(pesan_tipe_invalid());
    }

    // `safeReact(m, "🕕")` di `:153` — `Reply` cuma menyimpan satu reaksi,
    // jadi yang dikirim reaksi penutup (❌/✅/☢).
    match fetch_urls(&arg, ctx.http.as_ref()) {
        // Hasil kosong yang sah.
        Ok(urls) if urls.is_empty() => Reply::text(kosong(&arg)).with_reaction("❌"),
        // Unduh satu per satu sampai ada buffer valid.
        Ok(urls) => match ambil_gambar(ctx.http.as_ref(), &urls) {
            Some(buf) => Reply::image(
                MediaSource::bytes(buf).with_mimetype("image/jpeg"),
                format!("📸 *PAP {}*", arg.to_uppercase()),
            )
            .with_reaction("✅"),
            None => Reply::text(GAMBAR_TIDAK_TERSEDIA).with_reaction("❌"),
        },
        // `CapabilityError` jadi template error biasa;
        // `m.pushName` tidak ada di `Ctx` sehingga tetap "User".
        Err(()) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpResponse};
    use std::sync::{Arc, Mutex};

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "pap".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        (oktz_plugin_api::find("pap").unwrap().handler)(ctx)
    }

    /// Stub `call` berurutan; habis = offline.
    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<(u16, Vec<u8>)>>,
        hitung: Mutex<usize>,
        seen: Mutex<Vec<HttpCall>>,
    }

    impl Stub {
        fn baru(balasan: Vec<(u16, Vec<u8>)>) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(balasan),
                hitung: Mutex::new(0),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
            let i = {
                let mut hitung = self.hitung.lock().unwrap();
                let i = *hitung;
                *hitung += 1;
                i
            };
            match self.balasan.lock().unwrap().get(i).cloned() {
                Some((status, isi)) => Ok(HttpResponse {
                    status,
                    body: HttpBody::Bytes {
                        data: Arc::from(isi),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(
            &self,
            _: &str,
            _: &serde_json::Value,
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("pakai call")
        }
    }

    fn run_http(text: &str, http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "pap".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("pap").unwrap().handler)(ctx)
    }

    /// JPEG > [`MIN_BYTES`] — lolos `buffer_terima`.
    fn gambar_jpeg() -> Vec<u8> {
        let mut b = vec![b'x'; MIN_BYTES + 1];
        b[0..3].copy_from_slice(&[0xff, 0xd8, 0xff]);
        b
    }

    fn pin_json(url: &str) -> Vec<u8> {
        format!(r#"{{"status":true,"data":[{{"image":"{url}"}}]}}"#).into_bytes()
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        let mut cur = r;
        loop {
            match cur {
                Reply::Reaction { emoji, .. } => return Some(emoji),
                Reply::Mutate { reply, .. } => cur = reply,
                _ => return None,
            }
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("pap").unwrap();
        assert_eq!(p.meta.name, "pap");
        assert_eq!(p.meta.alias, ["papcewe", "papcowo", "papfemboy"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(
            p.meta.description,
            "Minta pap cewe, cowo, atau femboy dari Pinterest"
        );
        assert_eq!(p.meta.usage, ".pap <cewe/cowo/femboy>");
        assert_eq!(p.meta.example, ".pap cewe");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(p.meta.is_enabled);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["papcewe", "papcowo", "papfemboy"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "pap");
        }
    }

    #[test]
    fn an_unknown_type_prints_the_three_valid_ones() {
        assert_eq!(
            text_of(&run("ngawur")),
            "❌ Pilih salah satu tipe pap yang tersedia: `cewe`, `cowo`, atau `femboy`.\n\nContoh: `.pap cewe`"
        );
        assert_eq!(text_of(&run("")), text_of(&run("ngawur")));
        assert_eq!(text_of(&run("   ")), text_of(&run("ngawur")));
        assert_eq!(reaction_of(&run("ngawur")), None, "guard sebelum reaksi");
    }

    #[test]
    fn a_valid_type_without_network_reaches_the_error_branch() {
        for tipe in VALID_TYPES {
            let r = run(tipe);
            assert_eq!(text_of(&r), crate::common::error_template_bawaan(".", "pap"), "tipe {tipe}");
            assert_eq!(reaction_of(&r), Some("☢"), "tipe {tipe}");
        }
    }

    #[test]
    fn a_successful_search_sends_the_pin_image() {
        let http = Stub::baru(vec![
            (200, pin_json("https://i.pinimg.com/a.jpg")),
            (200, gambar_jpeg()),
        ]);
        let r = run_http("cewe", http.clone());

        assert_eq!(text_of(&r), "📸 *PAP CEWE*");
        assert_eq!(reaction_of(&r), Some("✅"));
        match r.without_mutations().tanpa_reaksi() {
            Reply::Image { media, .. } => match media {
                Some(MediaSource::Bytes { mimetype, .. }) => {
                    assert_eq!(mimetype.as_deref(), Some("image/jpeg"));
                }
                lain => panic!("harus bytes, dapat {lain:?}"),
            },
            lain => panic!("harus image, dapat {lain:?}"),
        }

        let seen = http.seen.lock().unwrap();
        assert_eq!(seen.len(), 2, "query pertama sudah menjawab, tak lanjut");
        assert!(seen[0].url.contains("/search/pinterest?q=cewe%20cantik%20indonesia"), "{}", seen[0].url);
        assert_eq!(seen[1].url, "https://i.pinimg.com/a.jpg");
        assert!(seen[1]
            .headers
            .iter()
            .any(|(k, v)| k == "Referer" && v == "https://www.pinterest.com/"));
    }

    #[test]
    fn an_empty_answer_replies_with_the_empty_message() {
        let pins_kosong = r#"{"status":true,"data":[]}"#.as_bytes().to_vec();
        let http = Stub::baru(vec![
            (200, pins_kosong.clone()),
            (200, pins_kosong.clone()),
            (200, pins_kosong),
        ]);
        let r = run_http("cowo", http.clone());
        assert_eq!(text_of(&r), kosong("cowo"));
        assert_eq!(reaction_of(&r), Some("❌"));
        assert_eq!(http.seen.lock().unwrap().len(), 3, "ketiga query dijawab kosong");
    }

    #[test]
    fn a_dead_pin_is_skipped_for_the_next_one() {
        let http = Stub::baru(vec![
            (
                200,
                br#"{"status":true,"data":[{"image":"https://i.pinimg.com/mati.jpg"},{"image":"https://i.pinimg.com/hidup.jpg"}]}"#.to_vec(),
            ),
            (200, b"<html>404</html>".to_vec()),
            (200, gambar_jpeg()),
        ]);
        let r = run_http("femboy", http.clone());
        assert_eq!(text_of(&r), "📸 *PAP FEMBOY*");
        assert_eq!(reaction_of(&r), Some("✅"));
        let seen = http.seen.lock().unwrap();
        assert!(
            (2..=3).contains(&seen.len()),
            "satu pin ditolak lalu pin kedua dipakai: {} panggilan",
            seen.len()
        );
    }

    #[test]
    fn all_queries_failing_reply_with_the_error_template() {
        let http = Stub::baru(vec![(500, Vec::new()), (500, Vec::new()), (500, Vec::new())]);
        let r = run_http("femboy", http.clone());
        assert_eq!(text_of(&r), crate::common::error_template_bawaan(".", "pap"));
        assert_eq!(reaction_of(&r), Some("☢"));
        assert_eq!(http.seen.lock().unwrap().len(), 3, "ketiga query dicoba");
    }

    #[test]
    fn a_fetch_of_urls_that_all_download_fail_replies_gambar_tidak_tersedia() {
        let http = Stub::baru(vec![
            (200, pin_json("https://i.pinimg.com/mati.jpg")),
            (200, b"<html>404</html>".to_vec()),
        ]);
        let r = run_http("cewe", http.clone());
        assert_eq!(text_of(&r), GAMBAR_TIDAK_TERSEDIA);
        assert_eq!(reaction_of(&r), Some("❌"));
    }

    #[test]
    fn odd_and_very_long_types_do_not_panic() {
        for input in ["<b>", "a|b", "🎨", "'\"", "a\u{202e}b", &"c".repeat(5000)] {
            let r = run(input);
            assert!(!text_of(&r).is_empty(), "input {input:?}");
        }
    }

    #[test]
    fn the_three_types_keep_their_query_lists() {
        assert_eq!(queries_for("cewe"), &QUERIES_CEWE);
        assert_eq!(queries_for("cowo"), &QUERIES_COWO);
        assert_eq!(queries_for("femboy"), &QUERIES_FEMBOY);
        assert!(queries_for("lain").is_empty());
        assert_eq!(VALID_TYPES, ["cewe", "cowo", "femboy"]);
    }

    #[test]
    fn magic_numbers_accept_only_jpeg_png_and_webp() {
        let jpeg = [0xffu8, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0];
        let png = [0x89u8, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0];
        let mut webp = [0u8; 12];
        webp[0..4].copy_from_slice(b"RIFF");
        webp[8..12].copy_from_slice(b"WEBP");
        let gif = [0x47u8, b'I', b'F', b'8', b'9', b'a', 0, 0, 0, 0, 0, 0];
        let html = [0x3cu8, b'!', b'D', b'O', b'C', b'T', b'Y', b'P', b'E', 0, 0, 0];

        assert!(is_image(&jpeg));
        assert!(is_image(&png));
        assert!(is_image(&webp));
        assert!(!is_image(&gif), "GIF bukan salah satu dari tiga");
        assert!(!is_image(&html), "halaman error CDN ditolak");
        assert!(!is_image(&[]));
        assert!(!is_image(&[0x89, b'P', b'N']), "kurang 4 byte");
    }

    #[test]
    fn the_buffer_gate_also_requires_more_than_a_thousand_bytes() {
        // Magic number JPEG di awal, sisanya isian apa saja.
        let kecil = {
            let mut b = vec![b'x'; MIN_BYTES];
            b[0..3].copy_from_slice(&[0xff, 0xd8, 0xff]);
            b
        };
        assert!(is_image(&kecil));
        assert!(!buffer_terima(&kecil), "tepat 1000 byte ditolak");
        let mut besar = kecil.clone();
        besar.push(b'x');
        assert!(buffer_terima(&besar), "1001 byte diterima");
        assert!(!buffer_terima(&vec![b'x'; 5000]), "bukan gambar");
    }

    #[test]
    fn the_empty_and_prepare_failure_messages_are_kept() {
        assert_eq!(
            kosong("cewe"),
            "❌ Waduh, pap cewe lagi kosong nih. Coba lagi nanti."
        );
        assert_eq!(
            GAMBAR_TIDAK_TERSEDIA,
            "⚠️ Gambar tidak tersedia. Coba lagi nanti."
        );
        assert_eq!(
            GAGAL_MENYIAPKAN,
            "⚠️ Gagal menyiapkan gambar. Coba lagi nanti."
        );
    }

    #[test]
    fn the_pinterest_request_carries_the_referer_header() {
        let call = download_request("https://i.pinimg.com/originals/x.jpg");
        assert_eq!(call.method, oktz_plugin_api::HttpMethod::Get);
        assert!(call
            .headers
            .iter()
            .any(|(k, v)| k == "Referer" && v == "https://www.pinterest.com/"));
    }
}