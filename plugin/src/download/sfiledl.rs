//! `plugins/download/sfiledl.js` — unduh file dari Sfile.mobi.
//!
//! Urutan backend mengikuti `src/lib/resolve.js`: scraper lokal
//! `src/scraper/sfiledl.js` dulu, lalu aggregator `neoxr` sebagai cadangan
//! (`src/capabilities/sfile.js:26-60`). Key `neoxr` dibaca dari runtime `Ctx`.
//! Key kosong membuat `aggregator.hit` dilewati tanpa request, seperti JS.
//!
//! Dua pesan khusus di `catch` sekarang terjangkau: key aggregator kosong
//! (`missingEnvNameOf` + `explainFailure`) dan respons yang gagal dinormalisasi
//! (`adaNormalisasiGagal`).
//!
//! Dokumen dikirim seperti `sfiledl.js:73-83`: `sendMedia(..., { type:
//! "document", fileName: data.filename, mimetype: data.mime })` →
//! [`Reply::Document`]. Mime dari scraper lokal selalu kosong — pipeline memakai
//! `application/octet-stream` sebagai bawaan (`socket.js:813`).

use oktz_plugin_api::{Ctx, HttpCall, HttpGet, MediaSource, Reply, plugin};
use serde_json::Value;

use crate::common::host_dari;

/// `sfiledl.js:49-52` — kartu pakai, delimiter markdown-nya backtick.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}sfiledl <url_sfile>`\n\n\
         > Contoh: `{prefix}sfiledl https://sfile.mobi/xxxxx`"
    )
}

/// `sfiledl.js:56` — host bukan Sfile.
pub(crate) const BUKAN_HOST: &str = "❌ URL harus dari sfile.mobi atau sfile.co!";

/// `sfiledl.js:105` — `adaNormalisasiGagal(error)` benar.
pub(crate) const NORMALISASI_GAGAL: &str =
    "❌ Gagal mendapatkan link download. File mungkin tidak tersedia.";

/// `explainFailure(new MissingApiKeyError("aggregator", envName), …)`
/// (`src/lib/stalker-fallback.js:29-33`). `command` sudah dirangkai jadi
/// `` `${prefix}sfiledl` `` oleh pemanggil.
pub(crate) fn api_key_belum_diset(command: &str, env_name: &str) -> String {
    format!(
        "❌ *API Key belum diset!*\n\n\
         > Command `{command}` butuh `{env_name}` di `.env`\n\
         > Saat ini belum ada, jadi tidak bisa dipanggil."
    )
}

/// `hostSfile` (`sfiledl.js:32-42`) tanpa crate URL: protokol harus http(s),
/// userinfo dan port dibuang, host berhenti di `/?#`, lalu dibandingkan per label
/// (`sfile.mobi`, `*.sfile.mobi`, `sfile.co`, `*.sfile.co`).
///
/// Guard per label, bukan `includes`: `sfile.mobi.evil.example` dan
/// `notsfile.mobi` tetap memuat teks "sfile" tapi bukan Sfile.
pub(crate) fn host_sfile(url: &str) -> bool {
    let Some(host) = host_dari(url) else {
        return false;
    };
    host == "sfile.mobi"
        || host.ends_with(".sfile.mobi")
        || host == "sfile.co"
        || host.ends_with(".sfile.co")
}

/// Hasil `normalize` (`src/capabilities/sfile.js:84-96`): `filename`, `url`,
/// `mime` — `size` tidak pernah dibaca plugin, jadi tidak dibawa.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SfileData {
    pub filename: String,
    pub url: String,
    pub mime: String,
}

/// `normalize` (`src/capabilities/sfile.js:63-100`) untuk backend lokal —
/// bentuk scraper: `{file_name, size_from_text, author_name, upload_date,
/// download_count, download_url}` dengan `null` untuk yang tidak ketemu.
/// Ampel aggregator (`{status, data:{url,...}}`) dibuka juga, mengikuti JS.
///
/// URL kosong/bukan http = kegagalan: `sfile()` mengembalikan
/// `download_url: null` pada tiga jalur keluar, dan meneruskannya berarti
/// `sendMedia(undefined)` sambil memberi centang hijau.
fn normalize_sfile(raw: &Value) -> Result<SfileData, ()> {
    // `raw.status === false` — bentuk lokal tidak punya `status`, jadi
    // pemeriksaannya eksplisit (`sfile.js:69-74`).
    if raw.get("status") == Some(&Value::Bool(false)) {
        return Err(());
    }
    // Ampel neoxr dibuka: `data` satu level di bawah `status`, array tidak.
    let isi = match raw.get("data") {
        Some(d) if d.is_object() => d,
        _ => raw,
    };
    let ambil = |kunci: &str| -> String {
        match isi.get(kunci) {
            Some(Value::String(s)) => s.trim().to_string(),
            // `null` dan bentuk lain dianggap kosong, bukan "undefined".
            _ => String::new(),
        }
    };
    let url = if ambil("download_url").is_empty() {
        ambil("url")
    } else {
        ambil("download_url")
    };
    let kecil = url.to_lowercase();
    if !(kecil.starts_with("http://") || kecil.starts_with("https://")) {
        return Err(());
    }
    Ok(SfileData {
        filename: if ambil("file_name").is_empty() {
            ambil("filename")
        } else {
            ambil("file_name")
        },
        url,
        mime: ambil("mime"),
    })
}

/// Kelas kegagalan resolver yang dibedakan plugin `sfiledl.js:85-109`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolveFailure {
    MissingApiKey,
    Normalization,
    Process,
}

/// `resolver.resolve("sfile", { url })`: backend lokal dulu, lalu aggregator
/// `neoxr` (`src/capabilities/sfile.js:125-141`). `aggregator.hit` melewati
/// aggregator tanpa request bila key kosong.
fn resolve_sfile(http: &dyn HttpGet, url: &str, apikey: &str) -> Result<SfileData, ResolveFailure> {
    match oktz_scraper::sfiledl::sfile(http, url) {
        Ok(body) => match normalize_sfile(&body) {
            Ok(data) => return Ok(data),
            Err(()) => {}
        },
        Err(_) => {}
    }

    if apikey.is_empty() {
        return Err(ResolveFailure::MissingApiKey);
    }

    // `aggregator.hit("neoxr", "/api/sfile", ...)`: key masuk query dan header.
    let target = format!(
        "https://api.neoxr.eu/api/sfile?url={}&apikey={}",
        super::douyindl::query_encode(url),
        super::douyindl::query_encode(apikey)
    );
    let body = http
        .call(&HttpCall::get(target).header("apikey", apikey))
        .ok()
        .and_then(|response| response.json())
        .ok_or(ResolveFailure::Process)?;
    normalize_sfile(&body).map_err(|_| ResolveFailure::Normalization)
}

#[plugin(
    name = "sfiledl",
    alias = ["sfile", "sfiledownload"],
    category = "download",
    description = "Download file dari Sfile.mobi",
    usage = ".sfiledl <url>",
    example = ".sfiledl https://sfile.mobi/xxx",
    cooldown = 15,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn sfiledl(ctx: Ctx) -> Reply {
    let url = ctx.text.trim();

    if url.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    if !host_sfile(url) {
        return Reply::text(BUKAN_HOST);
    }

    match resolve_sfile(ctx.http.as_ref(), url, &ctx.config().api_keys.neoxr) {
        // `sfiledl.js:73-83` — `sendMedia` document dari URL, nama dari
        // `data.filename`. Mime lokal selalu kosong (scraper tidak membaca
        // mime), jadi `MediaSource` tidak membawa mimetype — pipeline memakai
        // bawaan `application/octet-stream` seperti `socket.js:813`.
        Ok(data) => Reply::document(MediaSource::url(data.url), data.filename).with_reaction("✅"),
        // `missingEnvNameOf` diperiksa lebih dulu, seperti `sfiledl.js:93-97`.
        Err(ResolveFailure::MissingApiKey) => Reply::text(api_key_belum_diset(
            &format!("{}sfiledl", ctx.prefix),
            "APIKEY_NEOXR",
        ))
        .with_reaction("❌"),
        // `adaNormalisasiGagal` memakai pesan link, bukan template operator.
        Err(ResolveFailure::Normalization) => Reply::text(NORMALISASI_GAGAL).with_reaction("❌"),
        Err(ResolveFailure::Process) => {
            Reply::text(crate::common::error_template(&ctx)).with_reaction("☢")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpResponse};
    use std::sync::{Arc, Mutex};

    const SENDER: &str = "628111@s.whatsapp.net";

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "sfiledl".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        (find("sfiledl").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn isi(r: &Reply) -> &Reply {
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        }
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = find("sfiledl").unwrap();
        assert_eq!(p.meta.name, "sfiledl");
        assert_eq!(p.meta.alias, ["sfile", "sfiledownload"]);
        assert_eq!(p.meta.category, "download");
        assert_eq!(p.meta.description, "Download file dari Sfile.mobi");
        assert_eq!(p.meta.usage, ".sfiledl <url>");
        assert_eq!(p.meta.example, ".sfiledl https://sfile.mobi/xxx");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["sfile", "sfiledownload"] {
            assert_eq!(find(a).unwrap().meta.name, "sfiledl", "alias {a}");
        }
    }

    #[test]
    fn a_missing_link_shows_the_card_without_reacting() {
        for text in ["", "   ", "\t"] {
            let r = run(text);
            assert_eq!(text_of(&r), help("."), "input {text:?}");
            assert_eq!(reaksi(&r), None, "input {text:?}");
        }
    }

    #[test]
    fn the_card_follows_the_prefix() {
        let ctx = Ctx {
            prefix: "!".into(),
            command: "sfiledl".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            ..Default::default()
        };
        let r = (find("sfiledl").unwrap().handler)(ctx);
        assert_eq!(text_of(&r), help("!"));
    }

    #[test]
    fn only_sfile_hosts_pass_the_guard() {
        for good in [
            "https://sfile.mobi/xxxxx",
            "http://sfile.mobi/x",
            "https://www.sfile.mobi/x",
            "https://download1.sfile.co/x",
            "https://sfile.co/x",
            "https://SFILE.MOBI/x",
            "  https://sfile.mobi/x  ",
            "https://user:pw@sfile.mobi:8443/x?y=1#z",
        ] {
            assert!(host_sfile(good), "harus lolos: {good}");
        }
        for bad in [
            // Per label, bukan `includes`.
            "https://sfile.mobi.evil.example/x",
            "https://notsfile.mobi/x",
            "https://xsfile.co/x",
            "https://example.com/?u=sfile.mobi",
            "sfile.mobi/x",
            "ftp://sfile.mobi/x",
            "javascript:alert(1)",
            "",
        ] {
            assert!(!host_sfile(bad), "harus ditolak: {bad}");
        }
    }

    #[test]
    fn a_foreign_host_is_rejected_without_reacting() {
        for text in [
            "https://example.com/x",
            "https://sfile.mobi.evil.example/x",
            "halo dunia",
        ] {
            let r = run(text);
            assert_eq!(text_of(&r), BUKAN_HOST, "input {text}");
            assert_eq!(reaksi(&r), None, "input {text}");
        }
    }

    #[test]
    fn aggregator_key_kosong_memberi_pesan_key() {
        // `aggregator.hit` melewati `neoxr` tanpa request saat key kosong;
        // `missingEnvNameOf` menemukan `APIKEY_NEOXR` dari alasan kegagalan.
        let r = run("https://sfile.mobi/xxxxx");
        assert_eq!(text_of(&r), api_key_belum_diset(".sfiledl", "APIKEY_NEOXR"));
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn the_two_unreachable_catch_messages_are_still_byte_exact() {
        assert_eq!(
            NORMALISASI_GAGAL,
            "❌ Gagal mendapatkan link download. File mungkin tidak tersedia."
        );
        assert_eq!(
            api_key_belum_diset(".sfiledl", "APIKEY_NEOXR"),
            "❌ *API Key belum diset!*\n\n\
             > Command `.sfiledl` butuh `APIKEY_NEOXR` di `.env`\n\
             > Saat ini belum ada, jadi tidak bisa dipanggil."
        );
    }

    #[test]
    fn the_plugin_only_reads_and_never_mutates() {
        assert!(run("https://sfile.mobi/x").mutations().is_empty());
    }

    /// Stub `HttpGet` yang membalas tiga halaman sfile berurutan.
    #[derive(Debug)]
    struct StubSfile {
        halaman: Mutex<Vec<String>>,
        seen: Mutex<Vec<String>>,
    }

    impl StubSfile {
        fn baru(halaman: Vec<&str>) -> Arc<StubSfile> {
            Arc::new(StubSfile {
                halaman: Mutex::new(halaman.into_iter().map(str::to_string).collect()),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl oktz_plugin_api::HttpGet for StubSfile {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            let mut antrean = self.halaman.lock().unwrap();
            let teks = if antrean.is_empty() {
                String::new()
            } else {
                antrean.remove(0)
            };
            Ok(HttpResponse {
                status: 200,
                body: oktz_plugin_api::HttpBody::Bytes {
                    data: std::sync::Arc::from(teks.into_bytes()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "cuma GET".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "cuma GET".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, oktz_plugin_api::HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "cuma GET".into(),
            })
        }
    }

    /// Stub untuk jalur lokal lalu fallback aggregator. Halaman sfile dibalas
    /// dari antrean; respons aggregator dibedakan lewat host URL.
    #[derive(Debug)]
    struct StubAggregate {
        halaman: Mutex<Vec<String>>,
        aggregator: Option<Value>,
        seen: Mutex<Vec<HttpCall>>,
    }

    impl StubAggregate {
        fn baru(halaman: Vec<&str>, aggregator: Option<Value>) -> Arc<StubAggregate> {
            Arc::new(StubAggregate {
                halaman: Mutex::new(halaman.into_iter().map(str::to_string).collect()),
                aggregator,
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl oktz_plugin_api::HttpGet for StubAggregate {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "cuma GET".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "cuma GET".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(HttpError {
                message: "cuma GET".into(),
            })
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.clone());
            let teks = if req.url.contains("api.neoxr.eu") {
                serde_json::to_string(&self.aggregator.clone().ok_or(HttpError {
                    message: "aggregator offline".into(),
                })?)
                .map_err(|error| HttpError {
                    message: error.to_string(),
                })?
            } else {
                let mut antrean = self.halaman.lock().unwrap();
                if antrean.is_empty() {
                    String::new()
                } else {
                    antrean.remove(0)
                }
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: std::sync::Arc::from(teks.into_bytes()),
                    mimetype: None,
                },
                headers: Vec::new(),
            })
        }
    }

    fn run_dengan(http: Arc<StubSfile>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "sfiledl".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (find("sfiledl").unwrap().handler)(ctx)
    }

    fn run_dengan_key(http: Arc<StubAggregate>, text: &str, key: &str) -> Reply {
        let mut ctx = Ctx {
            prefix: ".".into(),
            command: "sfiledl".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        let mut config = ctx.config.as_ref().clone();
        config.api_keys.neoxr = key.to_string();
        ctx.config = Arc::new(config);
        (find("sfiledl").unwrap().handler)(ctx)
    }

    #[test]
    fn the_local_scraper_sends_the_document() {
        // Tiga halaman sfile: masukan (og:url) → gerbang (#download) →
        // unduhan (URL ter-escape di <script>).
        let http = StubSfile::baru(vec![
            r#"<html><head><meta property="og:url" content="https://sfile.co/abc"></head><body><h1>Berkas Uji</h1></body></html>"#,
            r#"<html><body><a id="download" href="https://sfile.co/gate/1">Unduh</a></body></html>"#,
            r#"<html><body><script>var x = {"url":"https:\/\/download01.sfile.co\/downloadfile\/1\/2\/abc\/uji.zip?k=kunci"};</script></body></html>"#,
        ]);
        let r = run_dengan(http, "https://sfile.co/abc");
        // `with_reaction` membungkus dokumen dalam `Reaction`.
        let isi = match r.without_mutations() {
            Reply::Reaction { reply, .. } => reply.as_ref(),
            lain => lain,
        };
        match isi {
            Reply::Document {
                media, file_name, ..
            } => {
                let m = media.as_ref().expect("dokumen harus punya sumber");
                assert!(matches!(m, MediaSource::Url { .. }));
                assert_eq!(
                    m.mimetype(),
                    None,
                    "mime scraper lokal kosong — pipeline yang memakai bawaan"
                );
                assert_eq!(file_name, "Berkas Uji");
            }
            lain => panic!("harus dokumen, dapat {lain:?}"),
        }
        assert_eq!(reaksi(&r), Some("✅"));
    }

    #[test]
    fn a_null_download_url_is_a_failure_not_an_empty_document() {
        // `download_url: null` (tanpa og:url) = kegagalan, bukan file sah.
        let http = StubSfile::baru(vec![r#"<html><body><h1>Matii</h1></body></html>"#]);
        let r = run_dengan(http, "https://sfile.co/abc");
        assert_eq!(text_of(&r), api_key_belum_diset(".sfiledl", "APIKEY_NEOXR"));
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn aggregator_neoxr_menggunakan_key_runtime_dan_query_seperti_js() {
        let http = StubAggregate::baru(
            Vec::new(),
            Some(serde_json::json!({
                "status": true,
                "data": {
                    "url": "https://download01.sfile.co/x.zip",
                    "filename": "x.zip",
                    "mime": "application/zip"
                }
            })),
        );
        let r = run_dengan_key(http.clone(), "https://sfile.co/abc", "KUNCI-RUNTIME");
        match isi(&r) {
            Reply::Document {
                media, file_name, ..
            } => {
                assert!(matches!(media.as_ref(), Some(MediaSource::Url { .. })));
                assert_eq!(file_name, "x.zip");
            }
            lain => panic!("harus dokumen, dapat {lain:?}"),
        }
        let seen = http.seen.lock().unwrap();
        let aggregator = seen.last().expect("request aggregator");
        assert_eq!(
            aggregator.url,
            "https://api.neoxr.eu/api/sfile\
             ?url=https%3A%2F%2Fsfile.co%2Fabc&apikey=KUNCI-RUNTIME"
        );
        assert!(
            aggregator.headers.iter().any(
                |(name, value)| name.eq_ignore_ascii_case("apikey") && value == "KUNCI-RUNTIME"
            )
        );
    }

    #[test]
    fn key_kosong_melewati_aggregator_dan_memberi_pesan_key() {
        let http = StubAggregate::baru(
            Vec::new(),
            Some(serde_json::json!({
                "status": true,
                "data": { "url": "https://download01.sfile.co/x.zip", "filename": "x.zip" }
            })),
        );
        let r = run_dengan_key(http.clone(), "https://sfile.co/abc", "");
        assert_eq!(text_of(&r), api_key_belum_diset(".sfiledl", "APIKEY_NEOXR"));
        assert_eq!(reaksi(&r), Some("❌"));
        assert!(
            http.seen
                .lock()
                .unwrap()
                .iter()
                .all(|request| !request.url.contains("api.neoxr.eu")),
            "aggregator tidak boleh dihubungi tanpa key"
        );
    }

    #[test]
    fn normalize_opens_the_envelope_and_rejects_status_false() {
        // Ampel aggregator dibuka: `data.url` dibaca.
        let ampel = serde_json::json!({
            "status": true,
            "data": { "url": "https://download01.sfile.co/x.zip", "filename": "x.zip" }
        });
        let data = normalize_sfile(&ampel).unwrap();
        assert_eq!(data.url, "https://download01.sfile.co/x.zip");
        assert_eq!(data.filename, "x.zip");
        // `status: false` — URL di `data` tidak boleh lolos.
        assert!(
            normalize_sfile(&serde_json::json!({
                "status": false,
                "data": { "url": "https://download01.sfile.co/x.zip" }
            }))
            .is_err()
        );
        // URL kosong/bukan http = gagal.
        assert!(
            normalize_sfile(&serde_json::json!({"file_name": "x", "download_url": null})).is_err()
        );
        assert!(normalize_sfile(&serde_json::json!({"download_url": "ftp://x/a"})).is_err());
    }

    #[test]
    #[ignore = "uji jaringan Sfile; set OKTZ_TEST_SFILE_URL dan APIKEY_NEOXR bila perlu"]
    fn live_sfiledl_mengambil_dokumen() {
        // Uji live padanan `plugins/download/sfiledl.js`.
        use crate::download::live_test::{assert_media, env_required, live_ctx};

        let url = env_required("OKTZ_TEST_SFILE_URL");
        let (http, ctx) = live_ctx("sfiledl", &url, vec![url.clone()]);
        let reply = (oktz_plugin_api::find("sfiledl").unwrap().handler)(ctx);
        assert_media(&reply, &["document"], &http);
    }
}
