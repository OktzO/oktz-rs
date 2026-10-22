//! `plugins/vps/listvps.js` — list semua VPS DigitalOcean.
//!
//! ## Penyimpangan
//!
//! `config.digitalocean.token` / `.sellers` / `.ownerPanels` dibaca dari
//! environment — lihat `crate::vps`.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

use crate::common::error_template;
use crate::vps::{has_access, js_str, token};

/// `droplet.networks?.v4?.find(n => n.type === 'public')?.ip_address || '-'`
/// (`listvps.js:45`).
fn ipv4_publik(droplet: &Value) -> String {
    droplet
        .pointer("/networks/v4")
        .and_then(Value::as_array)
        .and_then(|arr| {
            arr.iter()
                .find(|n| n.get("type").and_then(Value::as_str) == Some("public"))
        })
        .and_then(|n| n.get("ip_address"))
        .map(|v| js_str(Some(v)))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

/// Render daftar droplet — `listvps.js:54-63`.
fn teks_daftar(droplets: &[Value]) -> String {
    let mut t = String::from("📋 *ʟɪsᴛ ᴠᴘs ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ*\n");
    t.push_str(&format!("> Total: {} droplet\n\n", droplets.len()));
    for d in droplets {
        let status = if d.get("status").and_then(Value::as_str) == Some("active") {
            "🟢"
        } else {
            "🔴"
        };
        let region = d
            .pointer("/region/slug")
            .map(|v| js_str(Some(v)))
            .filter(|s| !s.is_empty() && s != "undefined")
            .unwrap_or_else(|| "-".to_string());
        t.push_str("╭─────────────\n");
        t.push_str(&format!("┃ {status} *{}*\n", js_str(d.get("name"))));
        t.push_str(&format!("┃ 🆔 ID: `{}`\n", js_str(d.get("id"))));
        t.push_str(&format!("┃ 🌐 IP: `{}`\n", ipv4_publik(d)));
        t.push_str(&format!("┃ 💾 RAM: {} MB\n", js_str(d.get("memory"))));
        t.push_str(&format!("┃ ⚡ CPU: {} vCPU\n", js_str(d.get("vcpus"))));
        t.push_str(&format!("┃ 💿 Disk: {} GB\n", js_str(d.get("disk"))));
        t.push_str(&format!("┃ 📍 Region: {region}\n"));
        t.push_str("╰─────────────\n\n");
    }
    t
}

#[plugin(
    name = "listvps",
    alias = ["listdroplet", "vpslist"],
    category = "vps",
    description = "List semua VPS DigitalOcean",
    usage = ".listvps",
    example = ".listvps",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn listvps(ctx: Ctx) -> Reply {
    let token = token(&ctx);
    if token.is_empty() {
        return Reply::text("⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*");
    }
    if !has_access(&ctx) {
        return Reply::text("❌ *ᴀᴋsᴇs ᴅɪᴛᴏʟᴀᴋ*");
    }

    // `await m.reply('🕕 *ᴍᴇɴɢᴀᴍʙɪʟ ᴅᴀᴛᴀ ᴠᴘs...*')` (`listvps.js:36`).
    let progres = Reply::text("🕕 *ᴍᴇɴɢᴀᴍʙɪʟ ᴅᴀᴛᴀ ᴠᴘs...*");

    let call = oktz_plugin_api::HttpCall::get("https://api.digitalocean.com/v2/droplets")
        .header("Authorization", format!("Bearer {token}"));
    let response = match ctx.http.call(&call) {
        Ok(r) => r,
        Err(_) => {
            return Reply::sequence(vec![
                progres,
                Reply::text(error_template(&ctx)),
            ]);
        }
    };
    let droplets: Vec<Value> = response
        .json()
        .as_ref()
        .and_then(|v| v.pointer("/droplets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    if droplets.is_empty() {
        return Reply::sequence(vec![
            progres,
            Reply::text("📋 *ʟɪsᴛ ᴠᴘs*\n\n> Tidak ada VPS yang tersedia."),
        ]);
    }

    Reply::sequence(vec![progres, Reply::text(teks_daftar(&droplets))])
}

#[cfg(test)]
mod tests {
    use super::*;
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
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("listvps hanya GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("listvps hanya GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("listvps hanya GET")
        }
    }

    const CONTOH: &str = r#"{"droplets":[
        {"id":1,"name":"web-01","status":"active","memory":2048,"vcpus":2,"disk":50,
         "networks":{"v4":[{"type":"public","ip_address":"203.0.113.9"}]},"region":{"slug":"sgp1"}},
        {"id":2,"name":"db-01","status":"off","memory":1024,"vcpus":1,"disk":25,
         "networks":{"v4":[]},"region":{"slug":"nyc1"}}]}"#;

    fn run(http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "listvps".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            http,
            is_owner: true,
            config: std::sync::Arc::new(oktz_plugin_api::config::Runtime {
                digitalocean: oktz_plugin_api::config::DigitalOcean {
                    token: "test".into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        (oktz_plugin_api::find("listvps").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("listvps").unwrap();
        assert_eq!(p.meta.name, "listvps");
        assert_eq!(p.meta.alias, ["listdroplet", "vpslist"]);
        assert_eq!(p.meta.category, "vps");
        assert_eq!(p.meta.description, "List semua VPS DigitalOcean");
        assert_eq!(p.meta.usage, ".listvps");
        assert_eq!(p.meta.example, ".listvps");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
    }

    #[test]
    fn output_sama_dengan_js() {
        let r = run(Stub::with(CONTOH));
        assert_eq!(
            semua_teks(&r),
            vec![
                "🕕 *ᴍᴇɴɢᴀᴍʙɪʟ ᴅᴀᴛᴀ ᴠᴘs...*".to_string(),
                concat!(
                    "📋 *ʟɪsᴛ ᴠᴘs ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ*\n",
                    "> Total: 2 droplet\n\n",
                    "╭─────────────\n",
                    "┃ 🟢 *web-01*\n",
                    "┃ 🆔 ID: `1`\n",
                    "┃ 🌐 IP: `203.0.113.9`\n",
                    "┃ 💾 RAM: 2048 MB\n",
                    "┃ ⚡ CPU: 2 vCPU\n",
                    "┃ 💿 Disk: 50 GB\n",
                    "┃ 📍 Region: sgp1\n",
                    "╰─────────────\n\n",
                    "╭─────────────\n",
                    "┃ 🔴 *db-01*\n",
                    "┃ 🆔 ID: `2`\n",
                    "┃ 🌐 IP: `-`\n",
                    "┃ 💾 RAM: 1024 MB\n",
                    "┃ ⚡ CPU: 1 vCPU\n",
                    "┃ 💿 Disk: 25 GB\n",
                    "┃ 📍 Region: nyc1\n",
                    "╰─────────────\n\n"
                )
                .to_string()
            ]
        );
    }

    #[test]
    fn daftar_kosong_menghasilkan_pesan_khusus() {
        let r = run(Stub::with(r#"{"droplets":[]}"#));
        assert_eq!(
            semua_teks(&r),
            vec![
                "🕕 *ᴍᴇɴɢᴀᴍʙɪʟ ᴅᴀᴛᴀ ᴠᴘs...*".to_string(),
                "📋 *ʟɪsᴛ ᴠᴘs*\n\n> Tidak ada VPS yang tersedia.".to_string()
            ]
        );
    }
}
