//! `plugins/tools/crm.js` — convert pesan yang di-reply jadi kode relayMessage.
//!
//! ## Penyimpangan
//!
//! - JS membaca `m.quoted.message` — objek pesan mentah lengkap (semua field
//!   Baileys, termasuk `jpegThumbnail`, `contextInfo`, dan lain-lain). API
//!   `QuotedMessage` hanya membawa `kind`, `text`, `mimetype`, dan
//!   `file_name`, jadi port ini membangun ulang isi pesan dari empat field
//!   itu. Bentuk `extendedTextMessage` (hasil `normalizeForRelay`) sama
//!   persis dengan JS untuk pesan teks; untuk media, isinya lebih ringkas
//!   karena byte dan field protokolnya tidak tersedia.
//! - `sock.relayMessage` (mengirim ulang pesan yang di-reply) tidak ada di
//!   `Reply`; port ini hanya menghasilkan kode dan berkasnya.
//! - Snippet mode memakai `Reply::table` — pesan `richResponseMessage` yang
//!   sama-sama berformat tabel, dengan teks judul dan kode di baris berikutnya.

use oktz_plugin_api::registry::MediaSource;
use oktz_plugin_api::{Ctx, QuotedKind, QuotedMessage, Reply, plugin};
use serde_json::{Map, Value, json};

fn tanpa_quote(ctx: &Ctx) -> String {
    format!(
        "❌ Reply pesan yang mau di-relay dulu, lalu ketik *{}{}*",
        ctx.prefix, ctx.command
    )
}

const JENIS_TIDAK_DIDUKUNG: &str = "❌ Jenis pesan ini belum didukung untuk di-relay.";

/// `toJsLiteral` (`crm.js:45-88`).
fn to_js_literal(value: &Value, indent: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(a) => {
            if a.is_empty() {
                return "[]".to_string();
            }
            let pad = " ".repeat(indent);
            let pad_close = " ".repeat(indent.saturating_sub(2));
            let items: Vec<String> = a
                .iter()
                .map(|v| format!("{pad}{}", to_js_literal(v, indent + 2)))
                .collect();
            format!("[\n{}\n{pad_close}]", items.join(",\n"))
        }
        Value::Object(o) => {
            if o.is_empty() {
                return "{}".to_string();
            }
            let pad = " ".repeat(indent);
            let pad_close = " ".repeat(indent.saturating_sub(2));
            let lines: Vec<String> = o
                .iter()
                .map(|(k, v)| {
                    let key = if is_valid_ident(k) {
                        k.clone()
                    } else {
                        serde_json::to_string(k).unwrap_or_else(|_| "\"\"".to_string())
                    };
                    format!("{pad}{key}: {}", to_js_literal(v, indent + 2))
                })
                .collect();
            format!("{{\n{}\n{pad_close}}}", lines.join(",\n"))
        }
    }
}

/// `/^[A-Za-z_$][A-Za-z0-9_$]*$/` (`crm.js:79`).
fn is_valid_ident(k: &str) -> bool {
    let mut chars = k.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Bangun isi pesan dari `QuotedMessage` — padanan `normalizeForRelay`.
fn konten_dari_quoted(q: &QuotedMessage) -> Value {
    let mut map: Map<String, Value> = Map::new();
    match q.kind {
        QuotedKind::Text => {
            map.insert("conversation".into(), Value::String(q.text.clone()));
        }
        QuotedKind::Image => {
            map.insert(
                "imageMessage".into(),
                json!({ "mimetype": q.mimetype, "caption": q.text }),
            );
        }
        QuotedKind::Video => {
            map.insert(
                "videoMessage".into(),
                json!({ "mimetype": q.mimetype, "caption": q.text }),
            );
        }
        QuotedKind::Audio => {
            map.insert(
                "audioMessage".into(),
                json!({ "mimetype": q.mimetype }),
            );
        }
        QuotedKind::Sticker => {
            map.insert("stickerMessage".into(), json!({ "mimetype": q.mimetype }));
        }
        QuotedKind::Document => {
            map.insert(
                "documentMessage".into(),
                json!({ "mimetype": q.mimetype, "fileName": q.file_name, "caption": q.text }),
            );
        }
        _ => {}
    }
    // `normalizeForRelay` — `conversation` jadi `extendedTextMessage`.
    if let Some(Value::String(teks)) = map.get("conversation").cloned() {
        map.remove("conversation");
        map.insert("extendedTextMessage".into(), json!({ "text": teks }));
    }
    Value::Object(map)
}

/// `typeNameFromContent` (`crm.js:116-120`).
fn nama_tipe(konten: &Value) -> String {
    let key = konten
        .as_object()
        .and_then(|o| o.keys().next())
        .cloned()
        .unwrap_or_else(|| "UnknownMessage".to_string());
    let mut chars = key.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "UnknownMessage".to_string(),
    }
}

/// `buildReadableSendCode` (`crm.js:107-114`).
fn kode_relay(konten: &Value) -> String {
    format!(
        "=> conn.relayMessage(\n  m.chat,\n  {},\n  {{}}\n)",
        to_js_literal(konten, 2)
    )
}

#[plugin(
    name = "crm",
    alias = ["crmsnipp", "relaycode", "codetorelay"],
    category = "tools",
    description = "Convert pesan yang di-reply jadi kode relayMessage",
    usage = ".crm (reply pesan) | .crm -snipp (reply pesan)",
    example = ".crm",
    cooldown = 5,
    energi = 0,
    is_owner = true,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn crm(ctx: Ctx) -> Reply {
    let Some(quoted) = ctx.quoted().cloned() else {
        return Reply::text(tanpa_quote(&ctx));
    };

    let konten = konten_dari_quoted(&quoted);
    if konten.as_object().is_none_or(|o| o.is_empty()) {
        return Reply::text(JENIS_TIDAK_DIDUKUNG);
    }

    let kode = kode_relay(&konten);
    let is_snippet = ctx
        .args
        .first()
        .is_some_and(|a| a.to_lowercase() == "-snipp")
        || ctx.command.to_lowercase().contains("snipp");

    if is_snippet {
        return Reply::table(
            "Convert relay message 🔥🔥.",
            Vec::<String>::new(),
            vec![vec![kode]],
        );
    }

    let tipe = nama_tipe(&konten);
    let filename = format!("{tipe}.js");
    let dir = "tmp/crm";
    let _ = ctx.berkas().buat_folder(dir);
    let path = format!("{dir}/{filename}");
    if ctx.berkas().tulis_teks(&path, &kode).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    Reply::document(MediaSource::path(path), filename.clone())
        .with_caption(format!("📄 {filename}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, Reply as R};

    const ME: &str = "628111@s.whatsapp.net";

    fn quoted_teks(teks: &str) -> QuotedMessage {
        QuotedMessage {
            id: "q1".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: teks.into(),
            kind: QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
        }
    }

    fn ctx_dengan(quoted: Option<QuotedMessage>, command: &str, args: &[&str]) -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            quoted,
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_quote() {
        let r = crm(ctx_dengan(None, "crm", &[]));
        assert_eq!(
            body(&r),
            "❌ Reply pesan yang mau di-relay dulu, lalu ketik *.crm*"
        );
    }

    #[test]
    fn output_sama_dengan_js_jenis_tidak_didukung() {
        let q = QuotedMessage {
            id: "q1".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: String::new(),
            kind: QuotedKind::Other("reactionMessage".into()),
            mimetype: None,
            file_name: None,
            from_me: false,
        };
        let r = crm(ctx_dengan(Some(q), "crm", &[]));
        assert_eq!(body(&r), JENIS_TIDAK_DIDUKUNG);
    }

    #[test]
    fn kode_teks_sama_dengan_js() {
        let konten = konten_dari_quoted(&quoted_teks("Halo"));
        assert_eq!(
            kode_relay(&konten),
            "=> conn.relayMessage(\n  m.chat,\n  {\n  extendedTextMessage: {\n    text: \"Halo\"\n  }\n},\n  {}\n)"
        );
    }

    #[test]
    fn nama_tipe_sama_dengan_js() {
        let konten = konten_dari_quoted(&quoted_teks("Halo"));
        assert_eq!(nama_tipe(&konten), "ExtendedTextMessage");
    }
}
