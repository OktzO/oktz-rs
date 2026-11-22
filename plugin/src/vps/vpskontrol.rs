//! `plugins/vps/vpskontrol.js` — kontrol VPS (on/off/restart).
//!
//! ## Penyimpangan
//!
//! 1. `config.digitalocean.token` / `.sellers` / `.ownerPanels` dibaca dari
//!    environment — lihat `crate::vps`.
//! 2. JS punya SATU file untuk empat command (`turnon`, `turnoff`,
//!    `restartvps`, `rebootvps`) dan memilih aksi lewat `m.command`. Di Rust
//!    satu `#[plugin]` = satu nama, jadi tiap aksi punya plugin sendiri;
//!    teks yang dihasilkan tetap sama karena `action.text` sama.

use oktz_plugin_api::{Ctx, Reply, plugin};

use crate::common::error_template;
use crate::vps::{has_access, js_str, token};

/// Satu baris `actions` (`vpskontrol.js:40-45`).
pub(crate) struct Aksi {
    /// `type` — nilai body API.
    pub(crate) tipe: &'static str,
    /// `emoji` — emoji status.
    pub(crate) emoji: &'static str,
    /// `text` — kata kerja yang dipakai di teks.
    pub(crate) teks: &'static str,
}

/// `actions[m.command]` (`vpskontrol.js:46-50`).
pub(crate) fn aksi(command: &str) -> Option<Aksi> {
    Some(match command {
        "turnon" => Aksi {
            tipe: "power_on",
            emoji: "🟢",
            teks: "menghidupkan",
        },
        "turnoff" => Aksi {
            tipe: "power_off",
            emoji: "🔴",
            teks: "mematikan",
        },
        "restartvps" | "rebootvps" => Aksi {
            tipe: "reboot",
            emoji: "🔄",
            teks: "merestart",
        },
        _ => return None,
    })
}

/// Plugin `turnon` — nyalakan VPS.
#[plugin(
    name = "turnon",
    alias = [],
    category = "vps",
    description = "Kontrol VPS (on/off/restart)",
    usage = ".turnon <id>",
    example = ".turnon 123456789",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn turnon(ctx: Ctx) -> Reply {
    kontrol(ctx, "turnon")
}

/// Plugin `turnoff` — matikan VPS.
#[plugin(
    name = "turnoff",
    alias = [],
    category = "vps",
    description = "Kontrol VPS (on/off/restart)",
    usage = ".turnoff <id>",
    example = ".turnoff 123456789",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn turnoff(ctx: Ctx) -> Reply {
    kontrol(ctx, "turnoff")
}

/// Plugin `restartvps` — restart VPS.
#[plugin(
    name = "restartvps",
    alias = [],
    category = "vps",
    description = "Kontrol VPS (on/off/restart)",
    usage = ".restartvps <id>",
    example = ".restartvps 123456789",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn restartvps(ctx: Ctx) -> Reply {
    kontrol(ctx, "restartvps")
}

/// Plugin `rebootvps` — sama dengan `restartvps` (`vpskontrol.js:44`).
#[plugin(
    name = "rebootvps",
    alias = [],
    category = "vps",
    description = "Kontrol VPS (on/off/restart)",
    usage = ".rebootvps <id>",
    example = ".rebootvps 123456789",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn rebootvps(ctx: Ctx) -> Reply {
    kontrol(ctx, "rebootvps")
}

/// Badan handler yang dipakai keempat command — `vpskontrol.js:31-78`.
fn kontrol(ctx: Ctx, command: &str) -> Reply {
    let token = token(&ctx);
    if token.is_empty() {
        return Reply::text("⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*");
    }
    if !has_access(&ctx) {
        return Reply::text("❌ *ᴀᴋsᴇs ᴅɪᴛᴏʟᴀᴋ*");
    }

    let droplet_id = ctx.text.trim();
    if droplet_id.is_empty() {
        return Reply::text(format!(
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{}{command} <droplet_id>`",
            ctx.prefix
        ));
    }

    let Some(aksi) = aksi(command) else {
        return Reply::text("❌ Aksi tidak dikenali.");
    };

    let progres = Reply::text(format!(
        "{} *sᴇᴅᴀɴɢ {} ᴠᴘs...*\n\n> ID: `{}`",
        aksi.emoji,
        aksi.teks.to_uppercase(),
        droplet_id
    ));

    let url = format!("https://api.digitalocean.com/v2/droplets/{droplet_id}/actions");
    let call = oktz_plugin_api::HttpCall::post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({ "type": aksi.tipe }));
    let response = match ctx.http.call(&call) {
        Ok(r) => r,
        Err(_) => {
            return Reply::sequence(vec![progres, Reply::text(error_template(&ctx))]);
        }
    };
    let status = response
        .json()
        .as_ref()
        .and_then(|v| v.pointer("/action/status"))
        .map(|v| js_str(Some(v)))
        .unwrap_or_else(|| "undefined".to_string());

    Reply::sequence(vec![
        progres,
        Reply::text(format!(
            "✅ *ᴀᴋsɪ ʙᴇʀʜᴀsɪʟ*\n\n> {} VPS sedang di-{}\n> Status: {}",
            aksi.emoji, aksi.teks, status
        ))
        .with_reaction("✅"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::Arc;

    #[derive(Debug)]
    struct Stub {
        body: Option<String>,
    }

    impl Stub {
        fn with(body: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(body.to_string()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, _url: &str) -> Result<HttpResponse, HttpError> {
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn post_json(&self, _url: &str, _body: &Value) -> Result<HttpResponse, HttpError> {
            let Some(text) = &self.body else {
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
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("vpskontrol hanya POST JSON")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("vpskontrol hanya POST JSON")
        }
    }

    fn run(command: &str, http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: command.into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            http,
            is_owner: true,
            ..Default::default()
        };
        (oktz_plugin_api::find(command).unwrap().handler)(ctx)
    }

    fn semua_teks(r: &Reply) -> Vec<String> {
        match r.without_mutations() {
            Reply::Sequence(list) => list
                .iter()
                .map(|x| x.text_body().unwrap_or_default().to_string())
                .collect(),
            other => vec![other.text_body().unwrap_or_default().to_string()],
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        for (nama, usage) in [
            ("turnon", ".turnon <id>"),
            ("turnoff", ".turnoff <id>"),
            ("restartvps", ".restartvps <id>"),
            ("rebootvps", ".rebootvps <id>"),
        ] {
            let p = oktz_plugin_api::find(nama).unwrap();
            assert_eq!(p.meta.name, nama);
            assert_eq!(p.meta.category, "vps");
            assert_eq!(p.meta.description, "Kontrol VPS (on/off/restart)");
            assert_eq!(p.meta.usage, usage);
            assert_eq!(p.meta.cooldown, 10);
            assert_eq!(p.meta.energi, 0);
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        // Tanpa token env -> guard pertama, sama seperti JS.
        let r = run(
            "turnon",
            Stub::with(r#"{"action":{"status":"in-progress"}}"#),
            "123",
        );
        assert_eq!(
            semua_teks(&r),
            vec!["⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*".to_string()]
        );
    }

    #[test]
    fn aksi_sesuai_command() {
        assert_eq!(aksi("turnon").unwrap().tipe, "power_on");
        assert_eq!(aksi("turnoff").unwrap().tipe, "power_off");
        assert_eq!(aksi("restartvps").unwrap().tipe, "reboot");
        assert_eq!(aksi("rebootvps").unwrap().tipe, "reboot");
        assert!(aksi("ngawur").is_none());
    }
}
