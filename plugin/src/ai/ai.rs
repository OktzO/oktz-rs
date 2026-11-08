//! `plugins/ai/ai.js` — chat cerdas dengan AI (Gemini) + renderer `AIRich`.
//!
//! Alur JS (`ai.js:39-145`): panggil scraper `gemini` (dengan `instruction`
//! dan `sessionId` per-user) → pecah jawaban markdown jadi blok teks / tabel
//! / kode → kirim lewat `AIRich` (`src/lib/builder.js`).
//!
//! `ponytail:` `AIRich` mengirim SATU pesan `botForwardedMessage >
//! richResponseMessage` yang berisi beberapa submessage. API plugin Rust tidak
//! punya varian "rich response", jadi tiap blok dikirim sebagai pesan sendiri
//! lewat `Reply::sequence` — teks tetap sama, hanya bentuknya berbeda.
//! Tabel memakai `Reply::table` (`sock.sendTable` di Rust), blok kode memakai
//! teks ber-fence ``` (AIRich merender kartu kode; teks fence adalah
//! padanan terdekat yang tersedia).
//!
//! `ponytail:` `extractIE` (`builder.js`) memecah inline entity (link, sitasi,
//! LaTeX) dari teks sebelum dikirim. Rust tidak punya parser itu, jadi teks
//! mentah dipakai apa adanya.
//!
//! `ponytail:` `sessions` di JS adalah `Map` modul-local yang di-evict kalau
//! lewat 500 entri. `Ctx::sessions` adalah `HashMap` global tanpa urutan,
//! jadi eviction tidak bisa disalin; kunci dipakai berawalan `ai:gemini:`
//! supaya tidak bentrok dengan plugin lain.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::{Value, json};

/// `systemPrompt` (`ai.js:32-37`).
const SYSTEM_PROMPT: &str = "Kamu adalah asisten AI yang cerdas dan canggih.\n\
Gunakan format markdown secara ketat:\n\
1. Jika membuat daftar perbandingan atau sekumpulan data, SELALU gunakan format tabel markdown (diawali dan diakhiri dengan '|').\n\
2. Jika memberikan kode pemrograman, SELALU bungkus dengan markdown code block (```bahasa ... ```).\n\
3. Gunakan formatting teks tebal (*teks*) untuk menekankan sesuatu, atau hashtag (#) untuk judul / penjelas besar.\n\
Pastikan semua respon terstruktur dengan baik agar sistem AIRich dapat merendernya dengan cantik.";

/// Prefix kunci sesi — map `sessions` di JS per-modul, di Rust `Ctx::sessions`
/// global sehingga butuh prefix supaya tidak menabrak plugin lain.
const KUNCI_SESI: &str = "ai:gemini:";

/// Kartu pakai — `ai.js:42-51`.
fn help(prefix: &str) -> String {
    format!(
        "🤖 *AI*\n\n\
         > Halo! Aku asisten cerdas\n\n\
         *Cara penggunaan:*\n\
         > `{prefix}ai <pertanyaan>`\n\n\
         *Contoh:*\n\
         > `{prefix}ai buatkan tabel jadwal piket`"
    )
}

/// Satu blok hasil pemecahan markdown.
#[derive(Debug)]
enum Blok {
    /// `aiRich.addText(...)`.
    Teks(String),
    /// `aiRich.addCode(lang, code)`.
    Kode { bahasa: String, kode: String },
    /// `aiRich.addTable(rows)` — baris pertama = header.
    Tabel(Vec<Vec<String>>),
}

/// `line.split('|').map(trim).filter((_, i, arr) => i !== 0 && i !== arr.length - 1)`
/// (`ai.js:90`).
fn sel_baris_tabel(line: &str) -> Vec<String> {
    let semua: Vec<&str> = line.split('|').collect();
    let n = semua.len();
    semua
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 0 && *i + 1 != n)
        .map(|(_, c)| c.trim().to_string())
        .collect()
}

/// Baris separator tabel (`|---|---|`) dibuang — `ai.js:92`.
fn bukan_baris_pemisah(row: &[String]) -> bool {
    !row.iter()
        .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
}

/// Pecah jawaban markdown seperti loop di `ai.js:103-136`.
fn pecah(teks: &str) -> Vec<Blok> {
    let lines: Vec<&str> = teks.split('\n').collect();
    // JS `currentTable` berisi baris mentah (string), bukan sel.
    let mut current_table: Vec<String> = Vec::new();
    let mut current_code: Vec<String> = Vec::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut text_buffer: Vec<String> = Vec::new();
    let mut blocks: Vec<Blok> = Vec::new();

    // `flushText` (`ai.js:80-85`).
    fn flush_text(buffer: &mut Vec<String>, blocks: &mut Vec<Blok>) {
        if !buffer.is_empty() {
            blocks.push(Blok::Teks(buffer.join("\n").trim().to_string()));
            buffer.clear();
        }
    }

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if !in_code {
                flush_text(&mut text_buffer, &mut blocks);
                if let Some(t) = flush_tabel_teks(&mut current_table) {
                    blocks.push(t);
                }
                in_code = true;
                // `line.trim().substring(3).trim() || 'text'` (`ai.js:111`).
                let lang = trimmed
                    .strip_prefix("```")
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string();
                code_lang = if lang.is_empty() {
                    "text".to_string()
                } else {
                    lang
                };
            } else {
                in_code = false;
                blocks.push(Blok::Kode {
                    bahasa: code_lang.clone(),
                    kode: current_code.join("\n"),
                });
                current_code.clear();
            }
            continue;
        }

        if in_code {
            current_code.push(line.to_string());
            continue;
        }

        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            flush_text(&mut text_buffer, &mut blocks);
            current_table.push(trimmed.to_string());
            continue;
        }

        // `flushTable` sebelum menambah baris teks biasa.
        if let Some(t) = flush_tabel_teks(&mut current_table) {
            blocks.push(t);
        }
        text_buffer.push(line.to_string());
    }

    flush_text(&mut text_buffer, &mut blocks);
    if let Some(t) = flush_tabel_teks(&mut current_table) {
        blocks.push(t);
    }

    blocks
}

/// `flushTable` (`ai.js:87-101`) — baris mentah disimpan untuk cabang
/// fallback `addText(currentTable.join('\n'))`.
fn flush_tabel_teks(current_table: &mut Vec<String>) -> Option<Blok> {
    if current_table.is_empty() {
        return None;
    }
    let mentah: Vec<String> = current_table.clone();
    let data: Vec<Vec<String>> = mentah.iter().map(|l| sel_baris_tabel(l)).collect();
    // JS `filter(row => !row.every(c => /^[-:]+$/.test(c)))` — buang baris
    // separator (`|---|---|`), sisakan baris isi.
    let disaring: Vec<Vec<String>> = data
        .iter()
        .filter(|row| bukan_baris_pemisah(row))
        .cloned()
        .collect();
    current_table.clear();

    if !disaring.is_empty() && disaring.iter().all(|row| !row.is_empty()) {
        Some(Blok::Tabel(disaring))
    } else {
        Some(Blok::Teks(mentah.join("\n")))
    }
}

/// Blocks → `Reply`.
fn blocks_ke_reply(blocks: Vec<Blok>) -> Reply {
    let mut out: Vec<Reply> = Vec::new();
    for b in blocks {
        match b {
            Blok::Teks(t) => {
                // Submessage teks kosong di AIRich tidak terlihat; pesan kosong
                // di WA justru ditolak, jadi dilewati.
                if t.is_empty() {
                    continue;
                }
                out.push(Reply::text(t));
            }
            Blok::Kode { bahasa, kode } => {
                out.push(Reply::text(format!("```{bahasa}\n{kode}\n```")));
            }
            Blok::Tabel(rows) => {
                let mut iter = rows.into_iter();
                let headers = iter.next().unwrap_or_default();
                // `toTableMetadata` menyamakan panjang baris dengan header
                // (`builder.js:2212-2214`).
                let lebar = headers.len();
                let body: Vec<Vec<String>> = iter
                    .map(|mut r| {
                        while r.len() < lebar {
                            r.push(String::new());
                        }
                        r
                    })
                    .collect();
                let mut tabel = oktz_plugin_api::Tabel::baru("");
                tabel = tabel.header(headers);
                for row in body {
                    tabel = tabel.baris(row);
                }
                out.push(Reply::Tabel { tabel });
            }
        }
    }
    Reply::sequence(out)
}

#[plugin(
    name = "ai",
    alias = ["ai4chat", "gemini"],
    category = "ai",
    description = "Chat cerdas dengan AI (mendukung tabel, kode, dll via AIRich)",
    usage = ".ai <pertanyaan>",
    example = ".ai buatkan tabel perbandingan vue dan react",
    cooldown = 5,
    energi = 1,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ai(ctx: Ctx) -> Reply {
    let text = ctx.text.trim().to_string();
    if text.is_empty() {
        return Reply::text(help(&ctx.prefix));
    }

    let kunci = format!("{KUNCI_SESI}{}", ctx.sender);
    let session_id = ctx
        .session_get(&kunci)
        .and_then(|v| v.as_str().map(str::to_string));

    let input = json!({
        "message": text,
        "instruction": SYSTEM_PROMPT,
        "sessionId": session_id,
    });

    let hasil = match oktz_scraper::gemini::chat(ctx.http.as_ref(), &input) {
        Ok(v) => v,
        // `ai.js:142-144` — `catch` memakai template error `te(...)`.
        Err(_) => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    // `ai.js:65-67` — simpan session id baru kalau ada.
    if let Some(sid) = hasil.get("sessionId").and_then(Value::as_str)
        && !sid.is_empty()
    {
        ctx.session_set(&kunci, Value::String(sid.to_string()));
    }

    let reply_text = hasil.get("text").and_then(Value::as_str).unwrap_or("");
    blocks_ke_reply(pecah(reply_text)).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpCall, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    /// Stub HTTP: scraper `gemini` memakai `HttpCall::call` dengan body form,
    /// jadi stub menimpa `call` (bukan `get`/`post_json`) dan mencatat
    /// permintaannya supaya bisa diperiksa test.
    #[derive(Debug)]
    struct Stub {
        balasan: Mutex<Vec<Result<HttpResponse, HttpError>>>,
        permintaan: Mutex<Vec<HttpCall>>,
        hitung: Mutex<usize>,
    }

    impl Stub {
        /// Satu balasan (200, `body`) dipakai berulang untuk semua permintaan —
        /// scraper hanya membaca status + header `batchexecute`, dan body yang
        /// sama tetap valid untuk pemanggilan berikutnya.
        fn ok(body: &str) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(vec![Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(body.as_bytes().to_vec().into_boxed_slice()),
                        mimetype: None,
                    },
                    headers: vec![],
                })]),
                permintaan: Mutex::new(Vec::new()),
                hitung: Mutex::new(0),
            })
        }

        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(vec![Err(HttpError {
                    message: "offline".into(),
                })]),
                permintaan: Mutex::new(Vec::new()),
                hitung: Mutex::new(0),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("plugin ini hanya POST")
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("plugin ini hanya POST form")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("plugin ini hanya POST form")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("plugin ini tidak multipart")
        }
        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            self.permintaan.lock().unwrap().push(req.clone());
            let mut hitung = self.hitung.lock().unwrap();
            let balasan = self.balasan.lock().unwrap();
            let i = (*hitung).min(balasan.len().saturating_sub(1));
            *hitung += 1;
            balasan[i].clone()
        }
    }

    /// Satu blok respons StreamGenerate yang bisa dibaca scraper:
    /// `parsed[4][0][1][0]` = jawaban, `parse1[1]` = `["id_lama"]`.
    fn blok_stream(jawaban: &str) -> String {
        let parsed = json!([
            "wrb.fr",
            ["id_lama"],
            null,
            null,
            [["id_baru", [jawaban]]]
        ]);
        format!("5\n{}\n", json!([["wrb.fr", null, parsed.to_string()]]))
    }

    fn run(http: Arc<Stub>, sessions: oktz_plugin_api::SessionStore, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "ai".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            sessions,
            ..Default::default()
        };
        (oktz_plugin_api::find("ai").unwrap().handler)(ctx)
    }

    fn blok_teks(r: &Reply) -> Vec<String> {
        match r.without_mutations().tanpa_reaksi() {
            Reply::Sequence(list) => list
                .iter()
                .filter_map(|x| x.without_mutations().text_body().map(str::to_string))
                .collect(),
            lain => lain
                .without_mutations()
                .text_body()
                .map(|t| vec![t.to_string()])
                .unwrap_or_default(),
        }
    }

    /// Body JSON `request_body` di dalam `f.req=` permintaan StreamGenerate
    /// (permintaan kedua; yang pertama `batchexecute` hanya mengambil cookie).
    fn isi_stream(stub: &Stub, indeks: usize) -> Value {
        let p = stub.permintaan.lock().unwrap();
        let req = &p[indeks];
        assert!(req.url.contains("StreamGenerate"), "bukan StreamGenerate");
        let Some(HttpBody::Form(form)) = &req.body else {
            panic!("body bukan form");
        };
        let isi = form.strip_prefix("f.req=").expect("awalan f.req=");
        // Kebalikan URLSearchParams: `+` = spasi, `%XX` = byte.
        let bytes = isi.as_bytes();
        let mut decode = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'+' => {
                    decode.push(b' ');
                    i += 1;
                }
                b'%' => {
                    let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
                    decode.push(u8::from_str_radix(hex, 16).unwrap());
                    i += 3;
                }
                _ => {
                    decode.push(bytes[i]);
                    i += 1;
                }
            }
        }
        let payload: Value = serde_json::from_slice(&decode).unwrap();
        serde_json::from_str(payload[1].as_str().unwrap()).unwrap()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = oktz_plugin_api::find("ai").unwrap();
        assert_eq!(p.meta.name, "ai");
        assert_eq!(p.meta.alias, ["ai4chat", "gemini"]);
        assert_eq!(p.meta.category, "ai");
        assert_eq!(
            p.meta.description,
            "Chat cerdas dengan AI (mendukung tabel, kode, dll via AIRich)"
        );
        assert_eq!(p.meta.usage, ".ai <pertanyaan>");
        assert_eq!(p.meta.example, ".ai buatkan tabel perbandingan vue dan react");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_admin);
        for a in ["ai", "ai4chat", "gemini"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "ai");
        }
    }

    #[test]
    fn pertanyaan_kosong_menampilkan_kartu_pakai() {
        let http = Stub::ok("");
        let r = run(http.clone(), oktz_plugin_api::new_session_store(), "   ");
        assert_eq!(
            blok_teks(&r),
            vec![
                "🤖 *AI*\n\n> Halo! Aku asisten cerdas\n\n*Cara penggunaan:*\n> `.ai <pertanyaan>`\n\n*Contoh:*\n> `.ai buatkan tabel jadwal piket`"
            ]
        );
        assert!(http.permintaan.lock().unwrap().is_empty());
    }

    #[test]
    fn teks_polos_dikirim_apa_adanya() {
        let http = Stub::ok(&blok_stream("Halo dunia"));
        let r = run(http.clone(), oktz_plugin_api::new_session_store(), "halo");
        assert_eq!(blok_teks(&r), vec!["Halo dunia"]);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "✅"
        ));
        // Permintaan pertama `batchexecute` (ambil cookie), kedua StreamGenerate.
        assert_eq!(http.permintaan.lock().unwrap().len(), 2);
        let body = isi_stream(&http, 1);
        assert_eq!(body[0][0], "halo", "message");
        assert_eq!(body[24][2], SYSTEM_PROMPT, "instruction");
        assert_eq!(body[2][0], "", "tanpa sesi, resumeArray bawaan");
    }

    #[test]
    fn session_id_disimpan_dan_dikirim_ulang() {
        let store = oktz_plugin_api::new_session_store();
        let http = Stub::ok(&blok_stream("ok"));
        let _ = run(http.clone(), store.clone(), "halo");
        let tersimpan = store
            .lock()
            .unwrap()
            .get("ai:gemini:628111@s.whatsapp.net")
            .cloned();
        let Some(Value::String(sid)) = tersimpan else {
            panic!("sesi tidak tersimpan");
        };
        assert!(!sid.is_empty(), "sessionId scraper berupa base64 non-kosong");

        // Sesi tersimpan dipakai ulang: resumeArray dari sessionId muncul di
        // permintaan berikutnya.
        let http2 = Stub::ok(&blok_stream("ok2"));
        let _ = run(http2.clone(), store.clone(), "lagi");
        let body = isi_stream(&http2, 1);
        assert_eq!(
            body[2][0], "id_lama",
            "resumeArray dari sesi tersimpan dipakai ulang"
        );
    }

    #[test]
    fn blok_kode_dibungkus_fence() {
        let blok = pecah("```js\nconsole.log(1)\n```");
        assert_eq!(blok.len(), 1);
        match &blok[0] {
            Blok::Kode { bahasa, kode } => {
                assert_eq!(bahasa, "js");
                assert_eq!(kode, "console.log(1)");
            }
            lain => panic!("harus Kode, dapat {lain:?}"),
        }
    }

    #[test]
    fn bahasa_kosong_default_text() {
        let blok = pecah("```\nhalo\n```");
        match &blok[0] {
            Blok::Kode { bahasa, .. } => assert_eq!(bahasa, "text"),
            lain => panic!("harus Kode, dapat {lain:?}"),
        }
    }

    #[test]
    fn tabel_menghasilkan_header_dan_baris() {
        let blok = pecah("| A | B |\n|---|---|\n| 1 | 2 |");
        match &blok[0] {
            Blok::Tabel(rows) => {
                // `.clone()` supaya tipe kiri-kanan sama (Vec<Vec<String>>);
                // `&Vec` tidak punya impl `PartialEq` ke `Vec`.
                assert_eq!(
                    rows.clone(),
                    vec![
                        vec!["A".to_string(), "B".to_string()],
                        vec!["1".to_string(), "2".to_string()]
                    ]
                );
            }
            lain => panic!("harus Tabel, dapat {lain:?}"),
        }
    }

    #[test]
    fn reply_tabel_memakai_baris_pertama_sebagai_header() {
        let blok = pecah("| A | B |\n| 1 | 2 |");
        let r = blocks_ke_reply(blok);
        let Reply::Sequence(list) = r.without_mutations().tanpa_reaksi() else {
            panic!("harus Sequence");
        };
        assert_eq!(list.len(), 1);
        match &list[0] {
            Reply::Tabel { tabel } => {
                assert_eq!(tabel.judul, "");
                assert_eq!(tabel.header, vec!["A".to_string(), "B".to_string()]);
                assert_eq!(tabel.baris, vec![vec!["1".to_string(), "2".to_string()]]);
            }
            lain => panic!("harus Tabel, dapat {lain:?}"),
        }
    }

    #[test]
    fn teks_dan_tabel_berurutan_jadi_dua_pesan() {
        let blok = pecah("Intro\n\n| A |\n|---|\n| 1 |\n\nOutro");
        assert_eq!(blok.len(), 3);
        let r = blocks_ke_reply(blok);
        let Reply::Sequence(list) = r.without_mutations().tanpa_reaksi() else {
            panic!("harus Sequence");
        };
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].text_body(), Some("Intro"));
        assert!(matches!(list[1], Reply::Tabel { .. }));
        assert_eq!(list[2].text_body(), Some("Outro"));
    }

    #[test]
    fn tabel_berisi_sel_kosong_tetap_jadi_tabel_sesuai_js() {
        // `| |` → satu sel kosong. JS `every(row => row.length > 0)` tetap
        // true (array berisi satu string kosong panjangnya 1), jadi tabel.
        let blok = pecah("| |");
        match &blok[0] {
            Blok::Tabel(rows) => assert_eq!(rows.clone(), vec![vec!["".to_string()]]),
            lain => panic!("harus Tabel, dapat {lain:?}"),
        }
    }

    #[test]
    fn baris_pemisah_saja_jadi_teks_mentah() {
        // Semua baris adalah separator → hasil filter kosong → fallback
        // `aiRich.addText(currentTable.join('\n'))` di JS.
        let blok = pecah("|---|---|");
        match &blok[0] {
            Blok::Teks(t) => assert_eq!(t, "|---|---|"),
            lain => panic!("harus Teks, dapat {lain:?}"),
        }
    }

    #[test]
    fn gagal_scraper_menampilkan_template_error() {
        let r = run(
            Stub::offline(),
            oktz_plugin_api::new_session_store(),
            "halo",
        );
        assert_eq!(
            r.without_mutations().text_body(),
            Some(crate::common::error_template_bawaan(".", "ai").as_str())
        );
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "☢"
        ));
    }
}
