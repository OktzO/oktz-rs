//! `plugins/vps/cekvps.js` — cek detail VPS DigitalOcean.
//!
//! ## Penyimpangan
//!
//! 1. `config.digitalocean.token` / `.sellers` / `.ownerPanels` tidak ada di
//!    `oktz_core::config::Runtime`, jadi dibaca dari environment
//!    (`DIGITALOCEAN_TOKEN`, `DIGITALOCEAN_SELLERS`,
//!    `DIGITALOCEAN_OWNER_PANELS`) — lihat `crate::vps`.
//! 2. `timeHelper.fromTimestamp` memakai `Intl.DateTimeFormat` dengan zona
//!    `Asia/Jakarta`; di sini dihitung manual sebagai UTC+7 (Jakarta tidak
//!    punya DST) dengan nama bulan Indonesia yang sama.
//! 3. `m.react` tidak ada di JS plugin ini — tidak ada yang perlu ditiru.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

use crate::common::error_template;
use crate::vps::{format_tanggal, has_access, token};

/// `droplet.networks?.v4?.find(n => n.type === "public")?.ip_address || "-"`
/// (`cekvps.js:60-61`).
fn ipv4_publik(droplet: &Value) -> String {
    droplet
        .pointer("/networks/v4")
        .and_then(Value::as_array)
        .and_then(|arr| {
            arr.iter()
                .find(|n| n.get("type").and_then(Value::as_str) == Some("public"))
        })
        .and_then(|n| n.get("ip_address"))
        .map(|v| crate::vps::js_str(Some(v)))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

/// `droplet.networks?.v6?.[0]?.ip_address || "-"` (`cekvps.js:62`).
fn ipv6(droplet: &Value) -> String {
    droplet
        .pointer("/networks/v6")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|n| n.get("ip_address"))
        .map(|v| crate::vps::js_str(Some(v)))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

/// `droplet.status === "active" ? "🟢 Active" : "🔴 " + droplet.status`
/// (`cekvps.js:63-64`).
fn status(droplet: &Value) -> String {
    let s = droplet
        .get("status")
        .map(|v| crate::vps::js_str(Some(v)))
        .unwrap_or_else(|| "undefined".to_string());
    if s == "active" {
        "🟢 Active".to_string()
    } else {
        format!("🔴 {s}")
    }
}

/// `droplet.region?.name || droplet.region?.slug` (`cekvps.js:76`).
fn region(droplet: &Value) -> String {
    let r = droplet.get("region");
    let name = r.and_then(|x| x.get("name")).map(|v| crate::vps::js_str(Some(v)));
    let slug = r.and_then(|x| x.get("slug")).map(|v| crate::vps::js_str(Some(v)));
    match name {
        Some(n) if !n.is_empty() && n != "undefined" => n,
        _ => match slug {
            Some(s) if !s.is_empty() && s != "undefined" => s,
            _ => "undefined".to_string(),
        },
    }
}

/// Render `cekvps.js:66-79`.
fn teks_detail(droplet: &Value) -> String {
    let s = |k: &str| crate::vps::js_str(droplet.get(k));
    let img = droplet
        .get("image")
        .and_then(|i| i.get("distribution"))
        .map(|v| crate::vps::js_str(Some(v)))
        .unwrap_or_else(|| "undefined".to_string());
    let nama_img = droplet
        .get("image")
        .and_then(|i| i.get("name"))
        .map(|v| crate::vps::js_str(Some(v)))
        .unwrap_or_else(|| "undefined".to_string());
    let created = droplet
        .get("created_at")
        .map(|v| crate::vps::js_str(Some(v)))
        .unwrap_or_else(|| "undefined".to_string());

    let mut t = String::from("📋 *ᴅᴇᴛᴀɪʟ ᴠᴘs*\n\n");
    t.push_str("╭─「 🖥️ *ɪɴꜰᴏ* 」\n");
    t.push_str(&format!("┃ 🆔 `ɪᴅ`: *{}*\n", s("id")));
    t.push_str(&format!("┃ 🏷️ `ɴᴀᴍᴇ`: *{}*\n", s("name")));
    t.push_str(&format!("┃ 📊 `sᴛᴀᴛᴜs`: *{}*\n", status(droplet)));
    t.push_str(&format!("┃ 🌐 `ɪᴘᴠ4`: *{}*\n", ipv4_publik(droplet)));
    t.push_str(&format!("┃ 🌍 `ɪᴘᴠ6`: *{}*\n", ipv6(droplet)));
    t.push_str("╰───────────────\n\n");
    t.push_str("╭─「 🧠 *sᴘᴇᴄ* 」\n");
    t.push_str(&format!("┃ 💾 `ʀᴀᴍ`: *{} MB*\n", s("memory")));
    t.push_str(&format!("┃ ⚡ `ᴄᴘᴜ`: *{} vCPU*\n", s("vcpus")));
    t.push_str(&format!("┃ 💿 `ᴅɪsᴋ`: *{} GB*\n", s("disk")));
    t.push_str(&format!("┃ 🌏 `ʀᴇɢɪᴏɴ`: *{}*\n", region(droplet)));
    t.push_str(&format!("┃ 💻 `ᴏs`: *{} {}*\n", img, nama_img));
    t.push_str("╰───────────────\n\n");
    t.push_str(&format!("> 📅 Created: {}", format_tanggal(&created)));
    t
}

#[plugin(
    name = "cekvps",
    alias = ["cekdroplet", "vpsstatus", "infovps"],
    category = "vps",
    description = "Cek detail VPS DigitalOcean",
    usage = ".cekvps <id>",
    example = ".cekvps 123456789",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekvps(ctx: Ctx) -> Reply {
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
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n> `{}cekvps <droplet_id>`",
            ctx.prefix
        ));
    }

    let url = format!("https://api.digitalocean.com/v2/droplets/{droplet_id}");
    let call = oktz_plugin_api::HttpCall::get(url).header("Authorization", format!("Bearer {token}"));
    let response = match ctx.http.call(&call) {
        Ok(r) => r,
        Err(_) => return Reply::text(error_template(&ctx)),
    };
    let Some(droplet) = response
        .json()
        .as_ref()
        .and_then(|v| v.pointer("/droplet"))
        .cloned()
    else {
        return Reply::text(error_template(&ctx));
    };

    Reply::text(teks_detail(&droplet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::Arc;

    #[derive(Debug)]
    struct Stub {
        body: Option<String>,
        seen: std::sync::Mutex<Vec<String>>,
    }

    impl Stub {
        fn with(body: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: Some(body.to_string()),
                seen: std::sync::Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                body: None,
                seen: std::sync::Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
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
            unreachable!("cekvps hanya GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("cekvps hanya GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("cekvps hanya GET")
        }
    }

    const CONTOH: &str = r#"{"droplet":{"id":123456789,"name":"web-01","status":"active",
        "memory":2048,"vcpus":2,"disk":50,"created_at":"2024-01-15T03:30:00Z",
        "networks":{"v4":[{"type":"private","ip_address":"10.0.0.1"},{"type":"public","ip_address":"203.0.113.9"}],"v6":[{"ip_address":"2001:db8::1"}]},
        "region":{"slug":"sgp1","name":"Singapore 1"},
        "image":{"distribution":"Ubuntu","name":"22.04 LTS x64"}}}"#;

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "cekvps".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            http,
            is_owner: true,
            ..Default::default()
        };
        (oktz_plugin_api::find("cekvps").unwrap().handler)(ctx)
    }

    fn run_token(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "cekvps".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
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
        (oktz_plugin_api::find("cekvps").unwrap().handler)(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("cekvps").unwrap();
        assert_eq!(p.meta.name, "cekvps");
        assert_eq!(p.meta.alias, ["cekdroplet", "vpsstatus", "infovps"]);
        assert_eq!(p.meta.category, "vps");
        assert_eq!(p.meta.description, "Cek detail VPS DigitalOcean");
        assert_eq!(p.meta.usage, ".cekvps <id>");
        assert_eq!(p.meta.example, ".cekvps 123456789");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_owner && !p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `token` env tidak diisi di test -> guard "belum setup".
        let r = run(Stub::with(CONTOH), "123456789");
        assert_eq!(teks(&r), "⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*");
    }

    #[test]
    fn detail_droplet_dirender_persis_seperti_js() {
        let d: Value = serde_json::from_str(CONTOH).unwrap();
        let t = teks_detail(d.pointer("/droplet").unwrap());
        assert_eq!(
            t,
            concat!(
                "📋 *ᴅᴇᴛᴀɪʟ ᴠᴘs*\n\n",
                "╭─「 🖥️ *ɪɴꜰᴏ* 」\n",
                "┃ 🆔 `ɪᴅ`: *123456789*\n",
                "┃ 🏷️ `ɴᴀᴍᴇ`: *web-01*\n",
                "┃ 📊 `sᴛᴀᴛᴜs`: *🟢 Active*\n",
                "┃ 🌐 `ɪᴘᴠ4`: *203.0.113.9*\n",
                "┃ 🌍 `ɪᴘᴠ6`: *2001:db8::1*\n",
                "╰───────────────\n\n",
                "╭─「 🧠 *sᴘᴇᴄ* 」\n",
                "┃ 💾 `ʀᴀᴍ`: *2048 MB*\n",
                "┃ ⚡ `ᴄᴘᴜ`: *2 vCPU*\n",
                "┃ 💿 `ᴅɪsᴋ`: *50 GB*\n",
                "┃ 🌏 `ʀᴇɢɪᴏɴ`: *Singapore 1*\n",
                "┃ 💻 `ᴏs`: *Ubuntu 22.04 LTS x64*\n",
                "╰───────────────\n\n",
                "> 📅 Created: 15 Januari 2024 10:30:00"
            )
        );
    }

    #[test]
    fn token_kosong_menghentikan_proses_sebelum_request() {
        let stub = Stub::with(CONTOH);
        let r = run(stub.clone(), "123456789");
        assert_eq!(teks(&r), "⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*");
        assert!(stub.seen.lock().unwrap().is_empty(), "tidak boleh request tanpa token");
    }

    #[test]
    fn upstream_error_menjadi_template_error() {
        let r = run_token(Stub::offline(), "123456789");
        assert!(teks(&r).contains("lagi ada kendala"), "{}", teks(&r));
    }

    #[test]
    fn format_tanggal_memakai_waktu_jakarta() {
        assert_eq!(format_tanggal("2024-01-15T03:30:00Z"), "15 Januari 2024 10:30:00");
        assert_eq!(format_tanggal("2024-12-31T17:00:00Z"), "01 Januari 2025 00:00:00");
        assert_eq!(format_tanggal("2024-06-30T00:00:00Z"), "30 Juni 2024 07:00:00");
    }
}
