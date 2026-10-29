//! `plugins/tools/transkrip.js` — konversi voice note / audio ke teks
//! (Speech-to-Text) lewat Groq Whisper.
//!
//! ## Penyimpangan
//!
//! - Konversi ke WAV memakai `oktz_plugin_api::ffmpeg` dengan argumen yang
//!   sama seperti JS: `-ar 16000 -ac 1 -f wav`.
//! - `error.response?.status` di JS tidak ada padanannya di `HttpError`, jadi
//!   port ini memakai `terima_semua_status()` dan membaca `res.status`
//!   langsung. Balasan 401 dan 429 tetap dipisah seperti di JS.

use oktz_plugin_api::registry::MultipartPart;
use oktz_plugin_api::{Ctx, HttpCall, Reply, plugin};

const ENDPOINT: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

fn cara_pakai(prefix: &str) -> String {
    format!(
        "🎤 *ᴛʀᴀɴsᴋʀɪᴘ*\n\n\
         > Reply voice note atau audio untuk mengonversi ke teks\n\
         > Contoh: reply VN → ketik `{prefix}transkrip`"
    )
}

fn tanpa_kunci() -> &'static str {
    "❌ *ɢᴀɢᴀʟ*\n\n\
     > API Key Groq belum diatur\n\
     > Set di config.js → APIkey.groq\n\
     > Gratis di https://console.groq.com"
}

fn terlalu_kecil() -> &'static str {
    "❌ Audio terlalu kecil atau gagal diunduh"
}

fn tidak_terdeteksi() -> &'static str {
    "❌ Tidak dapat mendeteksi suara. Pastikan audio jelas dan tidak terlalu pendek."
}

fn kunci_invalid() -> &'static str {
    "❌ API Key Groq invalid. Cek config.js → APIkey.groq"
}

fn rate_limit() -> &'static str {
    "❌ Rate limit Groq tercapai. Coba lagi nanti."
}

/// `transcribeWithGroq` (`transkrip.js:32-47`).
fn transcribe(ctx: &Ctx, wav: &[u8], api_key: &str) -> Result<String, u16> {
    let parts = vec![
        MultipartPart::file("file", "audio.wav", wav.to_vec()).with_mimetype("audio/wav"),
        MultipartPart::field("model", "whisper-large-v3"),
        MultipartPart::field("language", "id"),
        MultipartPart::field("response_format", "json"),
    ];
    let req = HttpCall::post(ENDPOINT)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(parts)
        .terima_semua_status();
    let res = ctx.http.call(&req).map_err(|_| 0u16)?;
    if res.status >= 400 {
        return Err(res.status);
    }
    Ok(res
        .json()
        .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(str::to_string))
        .unwrap_or_default())
}

#[plugin(
    name = "transkrip",
    alias = ["stt", "speechtotext", "transcribe"],
    category = "tools",
    description = "Konversi voice note / audio ke teks (Speech-to-Text)",
    usage = ".transkrip (reply voice note)",
    example = ".transkrip",
    cooldown = 10,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn transkrip(ctx: Ctx) -> Reply {
    // `const quoted = m.quoted || m` (`transkrip.js:49`).
    let (is_audio, dari_pemicu) = match ctx.quoted() {
        Some(q) => {
            let audio = matches!(q.kind, oktz_plugin_api::QuotedKind::Audio)
                || q.mimetype.as_deref().is_some_and(|m| m.contains("audio"));
            (audio, false)
        }
        None => {
            let audio = matches!(ctx.media_type(), Some("audioMessage"))
                || ctx
                    .media
                    .as_ref()
                    .and_then(|m| m.mimetype.as_deref())
                    .is_some_and(|m| m.contains("audio"));
            (audio, true)
        }
    };

    if !is_audio {
        return Reply::text(cara_pakai(&ctx.prefix));
    }

    let groq_key = ctx.config().api_keys.groq.clone();
    if groq_key.is_empty() {
        return Reply::text(tanpa_kunci());
    }

    let buffer = if dari_pemicu {
        ctx.download_media()
    } else {
        ctx.download_quoted_media()
    };
    let buffer = match buffer {
        Ok(b) => b,
        Err(_) => return Reply::text(crate::common::error_template(&ctx)).with_reaction("❌"),
    };
    if buffer.len() < 1000 {
        return Reply::text(terlalu_kecil()).with_reaction("❌");
    }

    // Konversi ke WAV 16 kHz mono — `convertToWav` (`transkrip.js:23-31`).
    let temp_dir = "tmp";
    let _ = ctx.berkas().buat_folder(temp_dir);
    let ts = crate::common::now_ms();
    let input_file = format!("{temp_dir}/stt_{ts}.ogg");
    let wav_file = format!("{temp_dir}/stt_{ts}.wav");
    if ctx.berkas().tulis(&input_file, &buffer).is_err() {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("❌");
    }
    let ffmpeg_args = [
        "ffmpeg",
        "-y",
        "-i",
        &input_file,
        "-ar",
        "16000",
        "-ac",
        "1",
        "-f",
        "wav",
        &wav_file,
    ];
    let hasil_ffmpeg = oktz_plugin_api::ffmpeg::jalankan_ke_berkas(&ffmpeg_args);
    let _ = ctx.berkas().hapus(&input_file);
    let wav = match hasil_ffmpeg {
        Ok(()) => ctx.berkas().baca(&wav_file).ok(),
        Err(_) => None,
    };
    let _ = ctx.berkas().hapus(&wav_file);
    let Some(wav) = wav else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("❌");
    };

    let teks = match transcribe(&ctx, &wav, &groq_key) {
        Ok(t) => t,
        Err(401) => return Reply::text(kunci_invalid()).with_reaction("❌"),
        Err(429) => return Reply::text(rate_limit()).with_reaction("❌"),
        Err(_) => return Reply::text(crate::common::error_template(&ctx)).with_reaction("❌"),
    };

    if teks.trim().is_empty() {
        return Reply::text(tidak_terdeteksi()).with_reaction("❌");
    }

    // `(buffer.length / 1024).toFixed(1)` (`transkrip.js:95`).
    let ukuran = format!("{:.1}", buffer.len() as f64 / 1024.0);
    let balasan = format!(
        "🎤 *ᴛʀᴀɴsᴋʀɪᴘ*\n\n\
         ╭┈┈⬡「 📝 *ʜᴀsɪʟ* 」\n\
         ┃\n\
         ┃ {teks}\n\
         ┃\n\
         ╰┈┈⬡\n\n\
         > 🤖 Model: Whisper Large V3\n\
         > 🌐 Bahasa: Indonesia\n\
         > 📊 Ukuran: ~{ukuran} KB"
    );

    Reply::text(balasan).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, QuotedKind, QuotedMessage, Reply as R};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct Stub {
        balasan: Mutex<Option<(u16, String)>>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn json(body: &str) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(Some((200, body.to_string()))),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn kode(kode: u16) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(Some((kode, "{}".to_string()))),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Mutex::new(None),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl oktz_plugin_api::HttpGet for Stub {
        fn get(&self, url: &str) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn post_json(
            &self,
            url: &str,
            _: &serde_json::Value,
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn post_form(
            &self,
            url: &str,
            _: &str,
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn post_multipart(
            &self,
            url: &str,
            _: &[(&str, oktz_plugin_api::HttpBody)],
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Err(oktz_plugin_api::HttpError {
                message: "offline".into(),
            })
        }
        fn call(
            &self,
            req: &oktz_plugin_api::HttpCall,
        ) -> Result<oktz_plugin_api::HttpResponse, oktz_plugin_api::HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            let guard = self.balasan.lock().unwrap();
            match guard.as_ref() {
                Some((kode, body)) => Ok(oktz_plugin_api::HttpResponse {
                    status: *kode,
                    body: oktz_plugin_api::HttpBody::Bytes {
                        data: Arc::from(body.clone().into_bytes()),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(oktz_plugin_api::HttpError {
                    message: "offline".into(),
                }),
            }
        }
    }

    fn ctx_dengan(http: Arc<Stub>, quoted: Option<QuotedMessage>, kunci: &str) -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "transkrip".into(),
            quoted,
            http,
            config: Arc::new(oktz_plugin_api::config::Runtime {
                api_keys: oktz_plugin_api::config::ApiKeys {
                    groq: kunci.to_string(),
                    ..oktz_plugin_api::config::ApiKeys::default()
                },
                ..oktz_plugin_api::config::Runtime::default()
            }),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_bukan_audio() {
        let r = transkrip(ctx_dengan(Stub::offline(), None, ""));
        assert_eq!(
            body(&r),
            "🎤 *ᴛʀᴀɴsᴋʀɪᴘ*\n\n\
             > Reply voice note atau audio untuk mengonversi ke teks\n\
             > Contoh: reply VN → ketik `.transkrip`"
        );
    }

    #[test]
    fn output_sama_dengan_js_tanpa_kunci_groq() {
        let q = QuotedMessage {
            id: "q1".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: String::new(),
            kind: QuotedKind::Audio,
            mimetype: Some("audio/ogg; codecs=opus".into()),
            file_name: None,
            from_me: false,
            timestamp: 0,
        };
        let r = transkrip(ctx_dengan(Stub::offline(), Some(q), ""));
        assert_eq!(
            body(&r),
            "❌ *ɢᴀɢᴀʟ*\n\n\
             > API Key Groq belum diatur\n\
             > Set di config.js → APIkey.groq\n\
             > Gratis di https://console.groq.com"
        );
    }

    #[test]
    fn audio_terlalu_kecil_memberi_pesan_yang_sama() {
        let q = QuotedMessage {
            id: "q1".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: String::new(),
            kind: QuotedKind::Audio,
            mimetype: Some("audio/ogg; codecs=opus".into()),
            file_name: None,
            from_me: false,
            timestamp: 0,
        };
        assert_eq!(terlalu_kecil(), "❌ Audio terlalu kecil atau gagal diunduh");
        let r = transkrip(ctx_dengan(Stub::offline(), Some(q), "KUNCI"));
        assert!(body(&r).contains("lagi ada kendala"), "{}", body(&r));
    }

    #[test]
    fn endpoint_groq_benar() {
        assert_eq!(
            ENDPOINT,
            "https://api.groq.com/openai/v1/audio/transcriptions"
        );
    }
}
