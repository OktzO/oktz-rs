//! `plugins/ai/text2img.js` — buat gambar dari teks lewat Firefly.
//!
//! Alur JS (`text2img.js:24-51`): cek teks → `m.react('🕕')` → cek
//! `config.APIkey.firefly` → GET
//! `https://firefly.maiku.my.id/api/deepai?apikey=…&prompt=…` →
//! `data.data.data.output_url` → kirim gambar dengan caption = prompt.
//!
//! `explainFailure(new MissingApiKeyError('firefly', 'APIKEY_FIREFLY'), cmd)`
//! (`src/lib/stalker-fallback.js:29-34`) menghasilkan kartu "API Key belum
//! diset" — teksnya dipakai apa adanya di sini.
//!
//! Catatan klasifikasi: `firefly.maiku.my.id` sudah mati (Cloudflare HTTP 530,
//! Argo Tunnel origin down) — lihat `docs/rust-rewrite/PLUGIN-CLASSIFICATION.md`
//! kelas E. Jalur HTTP tetap ditulis supaya perilakunya sama persis dengan JS
//! ketika key tersedia.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::Value;

use crate::tools::phisingataubukan::encode_uri_component;

/// `explainFailure(new MissingApiKeyError('firefly', 'APIKEY_FIREFLY'), cmd)`
/// (`src/lib/stalker-fallback.js:29-34`).
fn missing_key(command: &str) -> String {
    format!(
        "❌ *API Key belum diset!*\n\n\
         > Command `{command}` butuh `APIKEY_FIREFLY` di `.env`\n\
         > Saat ini belum ada, jadi tidak bisa dipanggil."
    )
}

/// `https://firefly.maiku.my.id/api/deepai?apikey=…&prompt=…`
/// (`text2img.js:37`).
fn api_url(api_key: &str, prompt: &str) -> String {
    format!(
        "https://firefly.maiku.my.id/api/deepai?apikey={api_key}&prompt={}",
        encode_uri_component(prompt)
    )
}

#[plugin(
    name = "text2img",
    alias = [],
    category = "ai",
    description = "Buat gambar dari teks",
    usage = ".text2img <teks>",
    example = ".text2img Buat gambar dari teks",
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
pub fn text2img(ctx: Ctx) -> Reply {
    let text = ctx.args.join(" ");
    if text.is_empty() {
        return Reply::text(format!(
            "📿 *ᴛᴇxᴛ ᴛᴏ ɪᴍᴀɢᴇ*\n\n> Masukkan teks\n\n`Contoh: {}text2img Buat gambar dari teks`",
            ctx.prefix
        ));
    }

    // `text2img.js:29-33` — reaksi `🕕` terjadi sebelum cek key, jadi ikut.
    let api_key = ctx.config().api_keys.firefly.clone();
    if api_key.is_empty() {
        return Reply::text(missing_key(&format!("{}text2img", ctx.prefix))).with_reaction("🕕");
    }

    let Some(body) = ctx
        .http
        .get(&api_url(&api_key, &text))
        .ok()
        .and_then(|r| r.json())
    else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    // `data?.data?.data?.output_url` (`text2img.js:39`).
    let Some(output) = body
        .pointer("/data/data/output_url")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    // `sock.sendMedia(m.chat, content, text, m, { type: "image" })` —
    // caption-nya prompt.
    Reply::image(MediaSource::url(output), text).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        resp: Option<String>,
        seen: Mutex<Vec<String>>,
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
            self.seen.lock().unwrap().push(url.into());
            self.respond()
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("plugin ini hanya GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("plugin ini hanya GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("plugin ini hanya GET")
        }
    }

    fn ctx_dasar() -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "text2img".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Default::default()
        }
    }

    fn run(http: Arc<Stub>, args: &str) -> Reply {
        let ctx = Ctx {
            text: args.into(),
            args: args.split_whitespace().map(str::to_string).collect(),
            http,
            ..ctx_dasar()
        };
        (oktz_plugin_api::find("text2img").unwrap().handler)(ctx)
    }

    fn body(r: &Reply) -> &str {
        r.text_body().expect("harus ada teks")
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("text2img").unwrap();
        assert_eq!(p.meta.name, "text2img");
        assert!(p.meta.alias.is_empty(), "alias di JS kosong");
        assert_eq!(p.meta.category, "ai");
        assert_eq!(p.meta.description, "Buat gambar dari teks");
        assert_eq!(p.meta.usage, ".text2img <teks>");
        assert_eq!(p.meta.example, ".text2img Buat gambar dari teks");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_admin);
        assert!(p.meta.is_enabled);
    }

    #[test]
    fn no_alias_means_the_command_itself_is_the_only_key() {
        assert_eq!(
            oktz_plugin_api::find("text2img").map(|p| p.meta.name.as_str()),
            Some("text2img")
        );
        assert_eq!(
            oktz_plugin_api::find("t2i").map(|p| p.meta.name.as_str()),
            None
        );
    }

    #[test]
    fn an_empty_prompt_shows_the_usage_card_without_a_reaction() {
        for args in ["", "   "] {
            let r = run(Stub::ok(""), args);
            assert!(body(&r).contains("📿 *ᴛᴇxᴛ ᴛᴏ ɪᴍᴀɢᴇ*"), "args={args:?}");
            assert!(
                body(&r).contains(".text2img Buat gambar dari teks"),
                "args={args:?}"
            );
            assert!(r.mutations().is_empty());
        }
    }

    #[test]
    fn a_prompt_without_an_api_key_answers_with_the_key_card() {
        let r = run(Stub::ok(""), "kucing");
        assert_eq!(
            body(&r),
            "❌ *API Key belum diset!*\n\n\
             > Command `.text2img` butuh `APIKEY_FIREFLY` di `.env`\n\
             > Saat ini belum ada, jadi tidak bisa dipanggil."
        );
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "🕕"
        ));
    }

    #[test]
    fn the_key_card_names_the_prefix_that_was_used() {
        let ctx = Ctx {
            prefix: "!".into(),
            command: "text2img".into(),
            text: "kucing".into(),
            args: vec!["kucing".into()],
            ..ctx_dasar()
        };
        let r = (oktz_plugin_api::find("text2img").unwrap().handler)(ctx);
        assert!(body(&r).contains("Command `!text2img`"), "{}", body(&r));
    }

    #[test]
    fn a_very_long_prompt_still_reaches_the_key_guard() {
        let r = run(Stub::ok(""), &"a".repeat(20_000));
        assert!(
            body(&r).contains("API Key belum diset"),
            "{}",
            &body(&r)[..80]
        );
    }

    #[test]
    fn dengan_api_key_gambar_dikirim_dengan_caption_prompt() {
        let mut cfg = oktz_plugin_api::config::Runtime::default();
        cfg.api_keys.firefly = "KEY-FIREFLY".into();
        let http = Stub::ok(
            r#"{"status":true,"data":{"data":{"output_url":"https://cdn/x.png"}}}"#,
        );
        let ctx = Ctx {
            text: "kucinglucu".into(),
            args: vec!["kucinglucu".into()],
            http: http.clone(),
            config: Arc::new(cfg),
            ..ctx_dasar()
        };
        let r = (oktz_plugin_api::find("text2img").unwrap().handler)(ctx);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "✅"
        ));
        match r.without_mutations().tanpa_reaksi() {
            Reply::Image { media, caption, .. } => {
                assert_eq!(caption, "kucinglucu");
                match media {
                    Some(MediaSource::Url { url, .. }) => assert_eq!(url, "https://cdn/x.png"),
                    lain => panic!("harus Url, dapat {lain:?}"),
                }
            }
            lain => panic!("harus Image, dapat {lain:?}"),
        }
        assert_eq!(
            http.seen.lock().unwrap()[0],
            "https://firefly.maiku.my.id/api/deepai?apikey=KEY-FIREFLY&prompt=kucinglucu"
        );
    }

    #[test]
    fn output_kosong_membalas_template_error() {
        let mut cfg = oktz_plugin_api::config::Runtime::default();
        cfg.api_keys.firefly = "K".into();
        let http = Stub::ok(r#"{"status":true,"data":{"data":{}}}"#);
        let ctx = Ctx {
            text: "kucing".into(),
            args: vec!["kucing".into()],
            http,
            config: Arc::new(cfg),
            ..ctx_dasar()
        };
        let r = (oktz_plugin_api::find("text2img").unwrap().handler)(ctx);
        assert_eq!(body(&r), crate::common::error_template_bawaan(".", "text2img"));
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "☢"
        ));
    }
}
