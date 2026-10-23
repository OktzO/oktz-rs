//! `plugins/vps/delvps.js` — hapus VPS DigitalOcean.
//!
//! ## Penyimpangan
//!
//! `config.digitalocean.token` / `.sellers` / `.ownerPanels` dibaca dari
//! environment — lihat `crate::vps`.
//!
//! `axios.delete` dikirim lewat `HttpCall` dengan metode `DELETE`; klien
//! `UreqHttp` (`lib/oktz-plugin-api/ureq_http.rs`) meneruskan metode itu
//! apa adanya.

use oktz_plugin_api::{Ctx, Reply, plugin};

use crate::common::error_template;
use crate::vps::{has_access, token};

#[plugin(
    name = "delvps",
    commands = [
        "deldroplet", "deletevps",
    ],
    alias = [],
    category = "vps",
    description = "Hapus VPS DigitalOcean",
    usage = ".delvps <id>",
    example = ".delvps 123456789",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn delvps(ctx: Ctx) -> Reply {
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
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{}delvps <droplet_id>`\n\n> Gunakan `{}listvps` untuk melihat ID",
            ctx.prefix, ctx.prefix
        ));
    }

    // `await m.reply('🗑️ *ᴍᴇɴɢʜᴀᴘᴜs ᴠᴘs...*\n\n> ID: `...`')` (`delvps.js:35`).
    let progres = Reply::text(format!(
        "🗑️ *ᴍᴇɴɢʜᴀᴘᴜs ᴠᴘs...*\n\n> ID: `{droplet_id}`"
    ));

    let url = format!("https://api.digitalocean.com/v2/droplets/{droplet_id}");
    let call = oktz_plugin_api::HttpCall::new(oktz_plugin_api::HttpMethod::Delete, url)
        .header("Authorization", format!("Bearer {token}"));
    if ctx.http.call(&call).is_err() {
        return Reply::sequence(vec![progres, Reply::text(error_template(&ctx))]);
    }

    Reply::sequence(vec![
        progres,
        Reply::text(format!(
            "✅ *ᴠᴘs ʙᴇʀʜᴀsɪʟ ᴅɪʜᴀᴘᴜs*\n\n> ID: `{droplet_id}`"
        ))
        .with_reaction("✅"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::Arc;

    #[derive(Debug)]
    struct Stub {
        seen: std::sync::Mutex<Vec<(String, String)>>,
    }

    impl Stub {
        fn baru() -> Arc<Stub> {
            Arc::new(Stub {
                seen: std::sync::Mutex::new(Vec::new()),
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
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen
                .lock()
                .unwrap()
                .push((req.method.as_str().to_string(), req.url.clone()));
            Ok(HttpResponse {
                status: 204,
                body: HttpBody::Bytes {
                    data: Arc::from(Vec::new()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "delvps".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            http,
            is_owner: true,
            ..Default::default()
        };
        (oktz_plugin_api::find("delvps").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("delvps").unwrap();
        assert_eq!(p.meta.name, "delvps");
        assert_eq!(p.meta.commands, ["deldroplet", "deletevps"]);
        assert!(p.meta.alias.is_empty(), "alias JS: []");
        assert_eq!(p.meta.category, "vps");
        assert_eq!(p.meta.description, "Hapus VPS DigitalOcean");
        assert_eq!(p.meta.usage, ".delvps <id>");
        assert_eq!(p.meta.example, ".delvps 123456789");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
    }

    #[test]
    fn output_sama_dengan_js() {
        let r = run(Stub::baru(), "123");
        assert_eq!(
            semua_teks(&r),
            vec!["⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*".to_string()]
        );
    }
}
