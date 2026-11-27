//! Unduh video/file dari TeraBox.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

use crate::common::scrub_nama_berkas;

/// Kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "📦 *TeraBox Downloader*\n\n\
         Download video atau file dari TeraBox.\n\n\
         *PENGGUNAAN:*\n\
         > *{prefix}terabox <link>*\n\n\
         *CONTOH:*\n\
         > *{prefix}terabox https://terabox.com/s/xxx*\n\n\
         _File dikirim sebagai dokumen, mungkin agak lama_"
    )
}

/// `result.status` falsy; `result.error` ikut tercetak
/// `undefined` kalau field-nya tidak ada.
pub(crate) fn gagal_pakai(error: &str) -> String {
    format!("❌ *TeraBox Gagal*\n\n> {error}")
}

/// Cabang `catch`.
pub(crate) const GAGAL: &str = "❌ Gagal mengambil data TeraBox, coba lagi nanti";

/// Bentuk objek yang `TeraBoxDL` kembalikan.
///
/// Scraper menyalin kunci apa adanya, jadi field yang tidak ada di item
/// Respons berarti `undefined` di versi awal — karena itu semuanya `Option`,
/// pemformatnya memakai `"undefined"`, bukan string kosong.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct HasilTerabox {
    pub status: bool,
    pub error: Option<String>,
    pub file_name: Option<String>,
    pub thumbnail: Option<String>,
    pub download_url: Option<String>,
    pub stream_url: Option<String>,
    pub file_size: Option<String>,
    pub duration: Option<String>,
    pub extension: Option<String>,
}

/// Baca hasil scraper. Kunci yang tidak ada di JSON berarti `undefined` di versi awal.
fn hasil_dari(data: &Value) -> HasilTerabox {
    let teks = |k: &str| data.get(k).and_then(Value::as_str).map(str::to_string);
    HasilTerabox {
        status: data
            .get("status")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        error: teks("error"),
        file_name: teks("file_name"),
        thumbnail: teks("thumbnail"),
        download_url: teks("download_url"),
        stream_url: teks("stream_url"),
        file_size: teks("file_size"),
        duration: teks("duration"),
        extension: teks("extension"),
    }
}

/// Caption.
pub(crate) fn caption(hasil: &HasilTerabox) -> String {
    format!(
        "📦 *TeraBox*\n\n\
         > 📌 {}\n\
         > 📏 Size: {}\n\
         > ⏱️ Durasi: {}",
        hasil.file_name.as_deref().unwrap_or("undefined"),
        hasil.file_size.as_deref().unwrap_or("undefined"),
        hasil.duration.as_deref().unwrap_or("undefined")
    )
}

/// `result.stream_url && result.stream_url.endsWith(".m3u8")`.
///
/// `endsWith` tidak membedakan huruf besar/kecil: `MOV.M3U8` **tidak** ikut,
/// karena `"MOV.M3U8".endsWith(".m3u8")` bernilai `false` di versi awal.
pub(crate) fn stream_hls(hasil: &HasilTerabox) -> bool {
    hasil
        .stream_url
        .as_deref()
        .is_some_and(|u| !u.is_empty() && u.ends_with(".m3u8"))
}

/// Nama berkas video hasil ffmpeg. `result.file_name`
/// `undefined` diteruskan sebagai string kosong supaya `|| "video"` tetap
/// berlaku, persis seperti di versi awal.
pub(crate) fn nama_berkas_video(file_name: &str) -> String {
    let dasar = if file_name.is_empty() { "video" } else { file_name };
    format!("{}.mp4", scrub_nama_berkas(dasar))
}

/// Nama berkas unduhan biasa; `|| ".mp4"` hanya dipakai
/// kalau `result.extension` kosong atau `null`.
pub(crate) fn nama_berkas_unduhan(file_name: &str, extension: Option<&str>) -> String {
    let dasar = if file_name.is_empty() { "file" } else { file_name };
    let ext = match extension {
        Some(e) if !e.is_empty() => e,
        _ => ".mp4",
    };
    format!("{}{}", scrub_nama_berkas(dasar), ext)
}

#[plugin(
    name = "terabox",
    alias = ["tb", "tera", "teraboxdl", "tbdl"],
    category = "download",
    description = "Download video/file dari TeraBox",
    usage = ".terabox <url>",
    example = ".terabox https://terabox.com/s/xxx",
    cooldown = 15,
    energi = 2,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn terabox(ctx: Ctx) -> Reply {
    let text = ctx.text.trim();
    if text.is_empty() {
        return Reply::text(help(&ctx.prefix)).with_reaction("❌");
    }

    let hasil = match oktz_scraper::terabox::tera_box_dl(ctx.http.as_ref(), text) {
        Ok(data) => hasil_dari(&data),
        // Scraper tidak pernah melempar; cabang ini hanya jaga-jaga kalau suatu saat
        // badannya keluar dari `try` seperti di versi awal.
        Err(_) => return Reply::text(GAGAL).with_reaction("☢"),
    };

    if !hasil.status {
        // `result.error` boleh `undefined`.
        let teks = hasil.error.as_deref().unwrap_or("undefined");
        return Reply::text(gagal_pakai(teks)).with_reaction("☢");
    }

    //
    let caption = caption(&hasil);

    if stream_hls(&hasil) {
        // Remux ffmpeg dilewati, URL stream dikirim
        // apa adanya dengan nama berkas dan mimetype yang sama.
        let stream = hasil.stream_url.clone().unwrap_or_default();
        return Reply::document(
            MediaSource::url(stream).with_mimetype("video/mp4"),
            nama_berkas_video(hasil.file_name.as_deref().unwrap_or("")),
        )
        .with_caption(caption)
        .with_reaction("✅");
    }

    if let Some(url) = hasil.download_url.as_deref() {
        // Dokumen biasa; versi awal menarik isinya ke memori
        // dulu, di sini URL-nya yang ditulis.
        return Reply::document(
            MediaSource::url(url).with_mimetype("application/zip"),
            nama_berkas_unduhan(
                hasil.file_name.as_deref().unwrap_or(""),
                hasil.extension.as_deref(),
            ),
        )
        .with_caption(caption)
        .with_reaction("✅");
    }

    // Tanpa `.m3u8` dan tanpa `download_url` tidak ada
    // pesan yang dikirim, hanya reaksi centang.
    Reply::None.with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpGet, HttpMethod, HttpResponse, find};
    use std::sync::{Arc, Mutex};

    const SENDER: &str = "628111@s.whatsapp.net";

    /// Stub `HttpGet`: halaman bot dulu (GET), lalu hasil pencarian (POST).
    #[derive(Debug, Default)]
    struct StubHttp {
        dipanggil: Mutex<Vec<HttpCall>>,
        halaman: String,
        hasil: String,
    }

    impl HttpGet for StubHttp {
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini lewat HttpCall::call")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("scraper ini tidak multipart")
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.dipanggil.lock().unwrap().push(req.clone());
            let body = if req.method == HttpMethod::Get {
                &self.halaman
            } else {
                &self.hasil
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(body.as_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
    }

    /// Halaman bot yang memuat token CSRF, seperti yang dicari scraper.
    const HALAMAN: &str =
        r#"<html><head><meta name="csrf-token" content="tok123"></head><body></body></html>"#;

    fn ctx_dengan(text: &str, halaman: &str, hasil: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "terabox".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http: Arc::new(StubHttp {
                dipanggil: Mutex::new(Vec::new()),
                halaman: halaman.to_string(),
                hasil: hasil.to_string(),
            }),
            ..Default::default()
        }
    }

    fn run(text: &str, halaman: &str, hasil: &str) -> Reply {
        (find("terabox").unwrap().handler)(ctx_dengan(text, halaman, hasil))
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    /// Isi balasan sebenarnya: `without_mutations` hanya membuang `Mutate`,
    /// jadi `Reaction { reply }` masih harus dibuka satu lapis.
    fn isi(r: &Reply) -> &Reply {
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        }
    }

    fn hasil_lengkap() -> HasilTerabox {
        HasilTerabox {
            status: true,
            error: None,
            file_name: Some("Video HD.mp4".into()),
            thumbnail: Some("https://img/t.jpg".into()),
            download_url: Some("https://cdn/d.mp4".into()),
            stream_url: Some("https://cdn/s.m3u8".into()),
            file_size: Some("120 MB".into()),
            duration: Some("00:45".into()),
            extension: Some(".mp4".into()),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = find("terabox").unwrap();
        assert_eq!(p.meta.name, "terabox");
        assert_eq!(p.meta.alias, ["tb", "tera", "teraboxdl", "tbdl"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download video/file dari TeraBox");
        assert_eq!(p.meta.usage, ".terabox <url>");
        assert_eq!(p.meta.example, ".terabox https://terabox.com/s/xxx");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["tb", "tera", "teraboxdl", "tbdl"] {
            assert_eq!(find(a).unwrap().meta.name, "terabox", "alias {a}");
        }
    }

    #[test]
    fn an_empty_argument_reacts_then_shows_the_card() {
        for text in ["", "   ", "\t"] {
            let r = run(text, HALAMAN, "{}");
            assert_eq!(text_of(&r), help("."), "input {text:?}");
            assert_eq!(reaksi(&r), Some("❌"), "input {text:?}");
        }
    }

    #[test]
    fn the_card_follows_the_prefix() {
        let mut ctx = ctx_dengan("", HALAMAN, "{}");
        ctx.prefix = "!!".into();
        let r = (find("terabox").unwrap().handler)(ctx);
        assert_eq!(text_of(&r), help("!!"));
    }

    #[test]
    fn the_scraper_never_throws_so_a_failed_lookup_reports_the_backend_message() {
        // `TeraBoxDL` menelan error di dalam `try`, jadi yang sampai ke plugin
        // selalu `{status:false, error}` — bukan lemparan.
        let r = run(
            "https://terabox.com/s/xxx",
            "<html><body>t tanpa token</body></html>",
            r#"{"error":false,"data":[]}"#,
        );
        assert_eq!(
            text_of(&r),
            "❌ *TeraBox Gagal*\n\n> CSRF Token not found"
        );
        assert_eq!(reaksi(&r), Some("☢"));
    }

    #[test]
    fn the_status_false_branch_prints_undefined_verbatim() {
        assert_eq!(
            gagal_pakai("Tidak ada data ditemukan"),
            "❌ *TeraBox Gagal*\n\n> Tidak ada data ditemukan"
        );
        assert_eq!(gagal_pakai("undefined"), "❌ *TeraBox Gagal*\n\n> undefined");
        // `TeraBoxDL` bisa mengembalikan `status:false` tanpa `error` kalau
        // pemformatan JSON-nya gagal; cetaknya tetap `undefined`.
        let kosong = hasil_dari(&serde_json::json!({}));
        assert!(!kosong.status);
        assert_eq!(kosong.error.as_deref().unwrap_or("undefined"), "undefined");
    }

    #[test]
    fn the_caption_uses_the_three_js_labels() {
        assert_eq!(
            caption(&hasil_lengkap()),
            "📦 *TeraBox*\n\n> 📌 Video HD.mp4\n> 📏 Size: 120 MB\n> ⏱️ Durasi: 00:45"
        );
        // Field hilang tetap menyisakan barisnya, dan nilainya tercetak
        // `undefined` seperti di versi awal — bukan string kosong.
        let kosong = HasilTerabox::default();
        assert_eq!(
            caption(&kosong),
            "📦 *TeraBox*\n\n> 📌 undefined\n> 📏 Size: undefined\n> ⏱️ Durasi: undefined"
        );
    }

    #[test]
    fn only_a_lowercase_m3u8_suffix_counts_as_hls() {
        assert!(stream_hls(&hasil_lengkap()));
        let mut lain = hasil_lengkap();
        lain.stream_url = Some("https://cdn/s.M3U8".into());
        assert!(!stream_hls(&lain), "endsWith di JS tidak case-insensitive");
        lain.stream_url = Some("https://cdn/s.mp4".into());
        assert!(!stream_hls(&lain));
        lain.stream_url = None;
        assert!(!stream_hls(&lain));
        lain.stream_url = Some(String::new());
        assert!(!stream_hls(&lain), "string kosong tidak lewat &&");
    }

    #[test]
    fn the_two_file_names_scrub_reserved_characters() {
        assert_eq!(nama_berkas_video(""), "video.mp4");
        // `@` bukan anggota kelas karakter versi awal `[<>:"/\\|?*]`, jadi ikut tinggal.
        assert_eq!(nama_berkas_video("x<>:@\"|?*y"), "x@y.mp4");
        assert_eq!(nama_berkas_video("a/b"), "ab.mp4");
    }

    #[test]
    fn the_download_file_name_uses_the_reported_extension() {
        assert_eq!(nama_berkas_unduhan("Arsip", Some(".zip")), "Arsip.zip");
        assert_eq!(nama_berkas_unduhan("", Some(".rar")), "file.rar");
        // `result.extension || ".mp4"` — `null` dan string kosong sama-
        // jatuh ke `.mp4`.
        assert_eq!(nama_berkas_unduhan("A", None), "A.mp4");
        assert_eq!(nama_berkas_unduhan("A", Some("")), "A.mp4");
    }

    #[test]
    fn an_m3u8_stream_is_sent_as_a_video_mp4_document() {
        let balasan = r#"{"error":false,"data":[{"file_name":"Film HD.mp4",
            "thumbnail":"https://img/t.jpg","download_url":"https://cdn/d.mp4",
            "stream_url":"https://cdn/s.m3u8","file_size":"120 MB","duration":"00:45",
            "extension":".mp4"}]}"#;
        let r = run("https://terabox.com/s/xxx", HALAMAN, balasan);
        assert_eq!(reaksi(&r), Some("✅"));
        match isi(&r) {
            Reply::Document {
                file_name, caption, media, ..
            } => {
                assert_eq!(file_name, "Film HD.mp4.mp4");
                assert_eq!(
                    caption,
                    "📦 *TeraBox*\n\n> 📌 Film HD.mp4\n> 📏 Size: 120 MB\n> ⏱️ Durasi: 00:45"
                );
                assert_eq!(
                    media.as_ref().map(|m| m.label()),
                    Some("url".to_string())
                );
                assert_eq!(media.as_ref().unwrap().mimetype(), Some("video/mp4"));
            }
            lain => panic!("harus dokumen, dapat {lain:?}"),
        }
    }

    #[test]
    fn a_plain_download_url_is_sent_as_a_zip_document() {
        let balasan = r#"{"error":false,"data":[{"file_name":"Arsip",
            "download_url":"https://cdn/d.zip","file_size":"12 MB","duration":"00:10",
            "extension":".zip"}]}"#;
        let r = run("https://terabox.com/s/xxx", HALAMAN, balasan);
        assert_eq!(reaksi(&r), Some("✅"));
        match isi(&r) {
            Reply::Document {
                file_name, caption, media, ..
            } => {
                assert_eq!(file_name, "Arsip.zip");
                assert_eq!(
                    caption,
                    "📦 *TeraBox*\n\n> 📌 Arsip\n> 📏 Size: 12 MB\n> ⏱️ Durasi: 00:10"
                );
                assert_eq!(
                    media.as_ref().unwrap().mimetype(),
                    Some("application/zip")
                );
            }
            lain => panic!("harus dokumen, dapat {lain:?}"),
        }
    }

    #[test]
    fn a_result_without_any_media_only_reacts() {
        // Tanpa `.m3u8` dan tanpa `download_url` tidak ada
        // pesan yang dikirim.
        let balasan = r#"{"error":false,"data":[{"file_name":"x","file_size":"1 MB","duration":"00:01"}]}"#;
        let r = run("https://terabox.com/s/xxx", HALAMAN, balasan);
        assert_eq!(reaksi(&r), Some("✅"));
        assert!(r.without_mutations().text_body().is_none());
    }

    #[test]
    fn the_plugin_only_reads_and_never_mutates() {
        let balasan = r#"{"error":false,"data":[{"file_name":"x","download_url":"https://cdn/d.zip"}]}"#;
        let r = run("https://terabox.com/s/x", HALAMAN, balasan);
        assert!(r.mutations().is_empty());
    }
}