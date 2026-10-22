//! `plugins/vps/sisavps.js` — cek sisa kuota VPS DigitalOcean.
//!
//! ## Penyimpangan
//!
//! `config.digitalocean.token` / `.sellers` / `.ownerPanels` dibaca dari
//! environment — lihat `crate::vps`.
//!
//! `Promise.all` dua endpoint dijemput berurutan; hasilnya identik karena
//! keduanya read-only.

use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

use crate::common::error_template;
use crate::vps::{has_access, js_str, token};

/// Render `sisavps.js:50-60`.
fn teks_kuota(account: &Value, droplets_used: usize) -> String {
    let limit = js_str(account.get("droplet_limit"));
    // `dropletLimit - dropletsUsed` — aritmetika number JS, jadi `10 - 2`
    // = `8` (bukan "10 - 2"). Kalau limit bukan angka, hasilnya `NaN`.
    let remaining = match account.get("droplet_limit").and_then(Value::as_f64) {
        Some(l) => crate::vps::fmt_num(l - droplets_used as f64),
        None => "NaN".to_string(),
    };
    let mut t = String::from("📊 *ᴋᴜᴏᴛᴀ ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ*\n\n");
    t.push_str("╭─────────────\n");
    t.push_str(&format!("┃ 📦 Limit: *{limit}* droplet\n"));
    t.push_str(&format!("┃ ✅ Terpakai: *{droplets_used}* droplet\n"));
    t.push_str(&format!("┃ 📋 Sisa: *{remaining}* droplet\n"));
    t.push_str("╰─────────────\n\n");
    t.push_str(&format!("> 👤 Email: {}\n", js_str(account.get("email"))));
    t.push_str(&format!("> ✅ Status: {}", js_str(account.get("status"))));
    t
}

#[plugin(
    name = "sisavps",
    alias = ["sisadroplet", "vpsquota"],
    category = "vps",
    description = "Cek sisa kuota VPS",
    usage = ".sisavps",
    example = ".sisavps",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn sisavps(ctx: Ctx) -> Reply {
    let token = token(&ctx);
    if token.is_empty() {
        return Reply::text("⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*");
    }
    if !has_access(&ctx) {
        return Reply::text("❌ *ᴀᴋsᴇs ᴅɪᴛᴏʟᴀᴋ*");
    }

    let auth = format!("Bearer {token}");
    let account = ctx.http.call(
        &oktz_plugin_api::HttpCall::get("https://api.digitalocean.com/v2/account")
            .header("Authorization", auth.clone()),
    );
    let droplets = ctx.http.call(
        &oktz_plugin_api::HttpCall::get("https://api.digitalocean.com/v2/droplets")
            .header("Authorization", auth),
    );
    let (Ok(account), Ok(droplets)) = (account, droplets) else {
        return Reply::text(error_template(&ctx));
    };

    let Some(account) = account.json().as_ref().and_then(|v| v.pointer("/account")).cloned()
    else {
        return Reply::text(error_template(&ctx));
    };
    let droplets_used = droplets
        .json()
        .as_ref()
        .and_then(|v| v.pointer("/droplets"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);

    Reply::text(teks_kuota(&account, droplets_used))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::Arc;

    #[derive(Debug)]
    struct Stub {
        bodies: Vec<String>,
    }

    impl Stub {
        fn with(bodies: &[&str]) -> Arc<Stub> {
            Arc::new(Stub {
                bodies: bodies.iter().map(|s| s.to_string()).collect(),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, _url: &str) -> Result<HttpResponse, HttpError> {
            let text = self
                .bodies
                .first()
                .cloned()
                .unwrap_or_else(|| "{}".to_string());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(text.into_bytes()),
                    mimetype: Some("application/json".into()),
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unreachable!("sisavps hanya GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unreachable!("sisavps hanya GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unreachable!("sisavps hanya GET")
        }
    }

    fn run(http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "sisavps".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            http,
            is_owner: true,
            ..Default::default()
        };
        (oktz_plugin_api::find("sisavps").unwrap().handler)(ctx)
    }

    fn run_token(http: Arc<Stub>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "sisavps".into(),
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
        (oktz_plugin_api::find("sisavps").unwrap().handler)(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("sisavps").unwrap();
        assert_eq!(p.meta.name, "sisavps");
        assert_eq!(p.meta.alias, ["sisadroplet", "vpsquota"]);
        assert_eq!(p.meta.category, "vps");
        assert_eq!(p.meta.description, "Cek sisa kuota VPS");
        assert_eq!(p.meta.usage, ".sisavps");
        assert_eq!(p.meta.example, ".sisavps");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
    }

    #[test]
    fn output_sama_dengan_js() {
        let r = run_token(Stub::with(&[r#"{"account":{"droplet_limit":10,"email":"a@b.c","status":"active"}}"#]));
        assert_eq!(
            teks(&r),
            concat!(
                "📊 *ᴋᴜᴏᴛᴀ ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ*\n\n",
                "╭─────────────\n",
                "┃ 📦 Limit: *10* droplet\n",
                "┃ ✅ Terpakai: *0* droplet\n",
                "┃ 📋 Sisa: *10* droplet\n",
                "╰─────────────\n\n",
                "> 👤 Email: a@b.c\n",
                "> ✅ Status: active"
            )
        );
    }

    #[test]
    fn token_kosong_menghasilkan_pesan_belum_setup() {
        let r = run(Stub::with(&["{}"]));
        assert_eq!(teks(&r), "⚠️ *ᴅɪɢɪᴛᴀʟᴏᴄᴇᴀɴ ʙᴇʟᴜᴍ ᴅɪsᴇᴛᴜᴘ*");
    }
}
