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

/// Prefix kunci sesi — `sessions` di JS鍵专属, di Rust `Ctx::sessions` global.
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

/// `ai.js:87-101` — `flushTable`.
fn flush_tabel(current_table: &mut Vec<Vec<String>>) -> Option<Blok> {
    if current_table.is_empty() {
        return None;
    }
    let data: Vec<Vec<String>> = current_table.iter().map(|l| sel_baris_tabel(l)).collect();
    let disaring: Vec<Vec<String>> = data
        .iter()
        .filter(|row| !bukan_baris_pemisah(row))
        .cloned()
        .collect();
    current_table.clear();

    if !disaring.is_empty() && disaring.iter().all(|row| !row.is_empty()) {
        Some(Blok::Tabel(disaring))
    } else {
        // `aiRich.addText(currentTable.join('\n'))` — sudah dikosongkan di atas,
        // jadi baris aslinya harus disimpan sebelum dibuang.
        Some(Blok::Teks(String::new()))
    }
}

/// Pecah jawaban markdown seperti loop di `ai.js:103-136`.
fn pecah(teks: &str) -> Vec<Blok> {
    let lines: Vec<&str> = teks.split('\n').collect();
    let mut current_table: Vec<Vec<String>> = Vec::new();
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
                let lang = trimmed[3..].trim().to_string();
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

/// `flushTable` (`ai.js:87-101`) — versi yang menyimpan teks baris mentah
/// untuk cabang fallback `addText(currentTable.join('\n'))`.
fn flush_tabel_teks(current_table: &mut Vec<Vec<String>>) -> Option<Blok> {
    if current_table.is_empty() {
        return None;
    }
    let mentah: Vec<String> = current_table.clone();
    let data: Vec<Vec<String>> = mentah.iter().map(|l| sel_baris_tabel(l)).collect();
    let disaring: Vec<Vec<String>> = data
        .iter()
        .filter(|row| !bukan_baris_pemisah(row))
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
                out.push(Reply::table("", headers, body));
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
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        resp: Option<String>,
        seen: Mutex<Vec<(String, String)>>,
    }

    impl Stub {
        fn ok(body: &str) -> Arc<Stub> {
            Arc::new(Stub {
                resp: Some(body.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                resp: None,
                seen: Mutex::new(Vec::new()),
            })
        }
        fn respond(&self) -> Result<HttpResponse, HttpError> {
            let Some(text) = &self.resp else {
                return Err(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                });
            };
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.clone().into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push((url.into(), String::new()));
            self.respond()
        }
        fn post_json(&self, url: &str, body: &Value) -> Result<HttpResponse, HttpError> {
            self.seen
                .lock()
                .unwrap()
                .push((url.into(), body.to_string()));
            self.respond()
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("plugin ini hanya POST JSON")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("plugin ini hanya POST JSON")
        }
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
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn teks_polos_dikirim_apa_adanya() {
        let http = Stub::ok(r#"{"text":"Halo dunia","sessionId":"abc"}"#);
        let r = run(http.clone(), oktz_plugin_api::new_session_store(), "halo");
        assert_eq!(blok_teks(&r), vec!["Halo dunia"]);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "✅"
        ));
        let seen = http.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1);
        let body: Value = serde_json::from_str(&seen[0].1).unwrap();
        assert_eq!(body["message"], "halo");
        assert_eq!(body["instruction"], SYSTEM_PROMPT);
        assert!(body["sessionId"].is_null());
    }

    #[test]
    fn session_id_disimpan_dan_dikirim_ulang() {
        let store = oktz_plugin_api::new_session_store();
        let http = Stub::ok(r#"{"text":"ok","sessionId":"SID-1"}"#);
        let _ = run(http.clone(), store.clone(), "halo");
        assert_eq!(
            store.lock().unwrap().get("ai:gemini:628111@s.whatsapp.net"),
            Some(&Value::String("SID-1".into()))
        );

        let http2 = Stub::ok(r#"{"text":"ok2","sessionId":"SID-2"}"#);
        let _ = run(http2.clone(), store.clone(), "lagi");
        let seen = http2.seen.lock().unwrap().clone();
        let body: Value = serde_json::from_str(&seen[0].1).unwrap();
        assert_eq!(body["sessionId"], "SID-1");
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
                assert_eq!(rows, vec![vec!["A", "B"], vec!["1", "2"]]);
            }
            lain => panic!("harus Tabel, dapat {lain:?}"),
        }
    }

    #[test]
    fn reply_tabel_memakai_baris_pertama_sebagai_header() {
        let blok = pecah("| A | B |\n| 1 | 2 |");
        let r = blocks_ke_reply(blok);
        match r.without_mutations().tanpa_reaksi() {
            Reply::Sequence(list) => match &list[0] {
                Reply::Table {
                    title,
                    headers,
                    rows,
                    ..
                } => {
                    assert_eq!(title, "");
                    assert_eq!(headers, &["A".to_string(), "B".to_string()]);
                    assert_eq!(rows, &vec![vec!["1".to_string(), "2".to_string()]]);
                }
                lain => panic!("harus Table, dapat {lain:?}"),
            },
            lain => panic!("harus Sequence, dapat {lain:?}"),
        }
    }

    #[test]
    fn teks_dan_tabel_berurutan_jadi_dua_pesan() {
        let blok = pecah("Intro\n\n| A |\n|---|\n| 1 |\n\nOutro");
        assert_eq!(blok.len(), 3);
        let r = blocks_ke_reply(blok);
        match r.without_mutations().tanpa_reaksi() {
            Reply::Sequence(list) => {
                assert_eq!(list.len(), 3);
                assert_eq!(list[0].text_body(), Some("Intro"));
                assert!(matches!(list[1], Reply::Table { .. }));
                assert_eq!(list[2].text_body(), Some("Outro"));
            }
            lain => panic!("harus Sequence, dapat {lain:?}"),
        }
    }

    #[test]
    fn tabel_tanpa_sel_kosong_jadi_teks_biasa() {
        // `| |` menghasilkan satu sel kosong → bukan tabel, jadi teks biasa.
        let blok = pecah("| |");
        match &blok[0] {
            Blok::Teks(t) => assert_eq!(t, "| |"),
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
