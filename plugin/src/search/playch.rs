//! `plugins/search/playch.js` — putar musik ke saluran (convert opus).
//!
//! `ponytail:` JS memakai paket npm `yt-search`; padanannya ada di
//! `search/ytsearch.rs`. Unduhan audio memakai kapabilitas `youtube` format
//! `mp3` → `oktz_scraper::youtube::get_youtube_direct_url`.
//!
//! PENYIMPANGAN (dijelaskan di laporan akhir):
//! - `sock.sendMessage(chId, { audio, ptt, waveform })` mengirim voice note ke
//!   SALURAN. `Reply` di `oktz-plugin-api` tidak punya "kirim ke chat lain",
//!   jadi voice note-nya dikirim ke chat tempat perintah dipanggil.
//! - `waveform` kustom tidak ada di `Reply::Audio`.
//! - `extensionFor(format)` tidak dipakai: input ffmpeg dibaca lewat
//!   `pipe:0` (ffmpeg mendeteksi format dari isi), bukan berkas sementara
//!   berekstensi.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

use super::ytsearch::{self, Video};

/// Batas ukuran audio sebelum konversi (`playch.js:144`).
const MIN_AUDIO: usize = 50_000;

/// Batas ukuran opus hasil konversi (`playch.js:146`).
const MIN_OPUS: usize = 10_000;

/// `playch.js:100-102` — kartu pakai.
pub(crate) fn help(prefix: &str) -> String {
    format!(
        "🎵 *PLAY SALURAN*\n\n`{prefix}playch <judul lagu>`\n`{prefix}playch --idch <id_saluran> <judul lagu>`"
    )
}

/// `playch.js:104-106` — saluran belum diatur.
pub(crate) fn tanpa_saluran() -> String {
    "❌ Saluran belum diatur. Gunakan `--idch <id>` atau atur di config.js".to_string()
}

/// `playch.js:112` — video tidak ditemukan.
pub(crate) fn video_tidak_ditemukan() -> String {
    "❌ Video tidak ditemukan".to_string()
}

/// `playch.js:121-135` — teks informasi.
pub(crate) fn info(
    video: &Video,
    kanal: &str,
    ch_id: &str,
    deskripsi_potong: Option<(String, bool)>,
) -> String {
    let mut info = String::from("🎵 *NOW PLAYING (SALURAN)*\n\n");
    info.push_str(&format!("📌 *Judul:* {}\n\n", video.title));
    info.push_str("*DETAIL*\n");
    info.push_str(&format!("👤 Channel: *{kanal}*\n"));
    info.push_str(&format!("⏱️ Durasi: *{}*\n", video.timestamp));
    info.push_str(&format!(
        "👀 Views: *{}*\n",
        ytsearch::format_views(video.views)
    ));
    info.push_str(&format!("📅 Upload: *{}*\n", video.ago));
    info.push_str(&format!("🆔 ID: `{}`\n\n", video.video_id));
    if let Some((desc, dipotong)) = deskripsi_potong {
        info.push_str(&format!(
            "*Deskripsi:*\n_{desc}{}_\n\n",
            if dipotong { "..." } else { "" }
        ));
    }
    info.push_str(&format!("📡 Saluran: `{ch_id}`\n"));
    info.push_str(&format!("🔗 {}\n\n", video.url));
    info.push_str("_⏳ mengirim audio ke saluran, harap tunggu..._");
    info
}

/// `playch.js:130-131` — deskripsi dipotong 150 karakter dan newline diganti
/// spasi. Mengembalikan `(teks, apakah_dipotong)`.
pub(crate) fn potong_deskripsi(deskripsi: &str) -> Option<(String, bool)> {
    if deskripsi.is_empty() {
        return None;
    }
    let dipotong = deskripsi.chars().count() > 150;
    // `substring(0, 150)` di JS memotong 160 unit UTF-16; untuk teks biasa
    // batas karakter sama, jadi `char` dipakai.
    let potong: String = deskripsi.chars().take(150).collect();
    Some((potong.replace('\n', " "), dipotong))
}

/// `pickVideo(search)` (`playch.js:26-29`).
pub(crate) fn pilih_video(videos: &[Video]) -> Option<&Video> {
    videos
        .iter()
        .find(|v| v.seconds > 0 && v.seconds < 900)
        .or_else(|| videos.first())
}

/// Pisahkan `--idch <id>` dari teks (`playch.js:93-97`).
pub(crate) fn pisah_idch(raw: &str) -> (Option<String>, String) {
    let Some(pos) = raw.find("--idch") else {
        return (None, raw.trim().to_string());
    };
    let setelah = &raw[pos + "--idch".len()..];
    let Some(awal) = setelah.find(char::is_whitespace) else {
        return (None, raw.trim().to_string());
    };
    let id = &setelah[awal..].trim_start();
    let Some(akhir) = id.find(char::is_whitespace) else {
        return (Some(id.to_string()), raw.trim().to_string());
    };
    let nilai = id[..akhir].to_string();
    let sisa = format!("{}{}", &raw[..pos], &id[akhir..]);
    (Some(nilai), sisa.trim().to_string())
}

/// Konversi ke Ogg/Opus (`playch.js:47-65`).
fn ke_opus(masukan: &[u8]) -> Result<Vec<u8>, String> {
    oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(
        &[
            "-y",
            "-i",
            "pipe:0",
            "-vn",
            "-map_metadata",
            "-1",
            "-ac",
            "1",
            "-ar",
            "48000",
            "-c:a",
            "libopus",
            "-b:a",
            "96k",
            "-vbr",
            "on",
            "-application",
            "audio",
            "-f",
            "ogg",
            "pipe:1",
        ],
        masukan,
    )
}

#[plugin(
    name = "playch",
    alias = ["pch", "playsaluran"],
    category = "search",
    description = "Putar musik ke saluran (convert opus)",
    usage = ".playch <query> atau .playch --idch <id> <query>",
    example = ".playch komang",
    cooldown = 15,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn playch(ctx: Ctx) -> Reply {
    let raw = ctx.text.trim().to_string();
    let (id_dari_argumen, q) = pisah_idch(&raw);

    let ch_id = id_dari_argumen
        .or_else(|| {
            let v = ctx.config().saluran.id.clone();
            if v.is_empty() { None } else { Some(v) }
        })
        .unwrap_or_default();

    if q.is_empty() {
        // `playch.js:99-102`.
        return Reply::text(help(&ctx.prefix));
    }
    if ch_id.is_empty() {
        // `playch.js:103-106`.
        return Reply::text(tanpa_saluran());
    }

    let videos = match ytsearch::search(ctx.http.as_ref(), &q) {
        Ok(v) => v,
        Err(_) => {
            // `playch.js:158-166` — ☢ lalu template error.
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    let Some(video) = pilih_video(&videos) else {
        // `playch.js:112`.
        return Reply::text(video_tidak_ditemukan());
    };

    // `playch.js:119` — unduhan audio mp3.
    let audio = match oktz_scraper::youtube::get_youtube_direct_url(
        ctx.http.as_ref(),
        &video.url,
        Some("mp3"),
    ) {
        Ok(v) => v,
        Err(_) => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };
    let Some(audio_url) = audio.get("url").and_then(|v| v.as_str()) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    // `playch.js:114` — nama channel.
    let kanal = if video.author_name.is_empty() {
        if video.author_username.is_empty() {
            "Unknown".to_string()
        } else {
            video.author_username.clone()
        }
    } else {
        video.author_name.clone()
    };

    let info = info(video, &kanal, &ch_id, potong_deskripsi(&video.description));

    // `playch.js:141-144` — unduh byte audio.
    let audio_buf = match ctx.http.get(audio_url) {
        Ok(res) => res.body.bytes().to_vec(),
        Err(_) => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };
    if audio_buf.len() < MIN_AUDIO {
        // `playch.js:144` — `throw new Error("Audio terlalu kecil")`.
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    }

    // `playch.js:145-146` — konversi opus.
    let opus = match ke_opus(&audio_buf) {
        Ok(b) if b.len() >= MIN_OPUS => b,
        _ => {
            return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
        }
    };

    let judul = video.title.clone();
    let gambar = Reply::image(MediaSource::url(video.thumbnail.clone()), info);
    let suara = Reply::voice_note(
        MediaSource::bytes(opus).with_mimetype("audio/ogg; codecs=opus"),
    );
    let konfirmasi = Reply::text(format!("✅ *{judul}* berhasil dikirim ke saluran"));

    Reply::sequence(vec![gambar, suara, konfirmasi]).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Stub {
        body: String,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn body(v: &str) -> Arc<Stub> {
            Arc::new(Stub {
                body: v.to_string(),
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
        fn post_json(&self, _: &str, _: &Value) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_form(&self, _: &str, _: &str) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("cuma GET")
        }
        fn call(&self, req: &oktz_plugin_api::HttpCall) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(req.url.clone());
            Ok(HttpResponse {
                status: 200,
                body: HttpBody::Bytes {
                    data: Arc::from(self.body.clone().into_bytes()),
                    mimetype: None,
                },
                headers: vec![],
            })
        }
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "playch".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        };
        (oktz_plugin_api::find("playch").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    fn video(sec: u64) -> Video {
        Video {
            url: "https://www.youtube.com/watch?v=aaaaaaaaaaa".into(),
            title: "Lagu Uji".into(),
            thumbnail: "https://i.ytimg.com/vi/a/hq.jpg".into(),
            description: "Deskripsi".into(),
            video_id: "aaaaaaaaaaa".into(),
            views: 1_234_567,
            ago: "2 years ago".into(),
            timestamp: "3:33".into(),
            seconds: sec,
            author_name: "Channel Uji".into(),
            author_username: "UC1".into(),
            author_url: "https://www.youtube.com/UC1".into(),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("playch").unwrap();
        assert_eq!(p.meta.name, "playch");
        assert_eq!(p.meta.alias, ["pch", "playsaluran"]);
        assert_eq!(p.meta.category, "search");
        assert_eq!(p.meta.description, "Putar musik ke saluran (convert opus)");
        assert_eq!(p.meta.usage, ".playch <query> atau .playch --idch <id> <query>");
        assert_eq!(p.meta.example, ".playch komang");
        assert_eq!(p.meta.cooldown, 15);
        assert_eq!(p.meta.energi, 1);
    }

    #[test]
    fn output_sama_dengan_js() {
        // `playch.js:100-102` — kartu pakai.
        assert_eq!(
            help("."),
            "🎵 *PLAY SALURAN*\n\n`.playch <judul lagu>`\n`.playch --idch <id_saluran> <judul lagu>`"
        );
        // `playch.js:104-106` — saluran belum diatur.
        assert_eq!(
            tanpa_saluran(),
            "❌ Saluran belum diatur. Gunakan `--idch <id>` atau atur di config.js"
        );
        // `playch.js:112` — video tidak ditemukan.
        assert_eq!(video_tidak_ditemukan(), "❌ Video tidak ditemukan");
        // `playch.js:121-135` — teks informasi.
        let v = video(213);
        let t = info(&v, "Channel Uji", "123@newsletter", Some(("Deskripsi".into(), false)));
        assert_eq!(
            t,
            "🎵 *NOW PLAYING (SALURAN)*\n\n\
             📌 *Judul:* Lagu Uji\n\n\
             *DETAIL*\n\
             👤 Channel: *Channel Uji*\n\
             ⏱️ Durasi: *3:33*\n\
             👀 Views: *1.2M*\n\
             📅 Upload: *2 years ago*\n\
             🆔 ID: `aaaaaaaaaaa`\n\n\
             *Deskripsi:*\n_Deskripsi_\n\n\
             📡 Saluran: `123@newsletter`\n\
             🔗 https://www.youtube.com/watch?v=aaaaaaaaaaa\n\n\
             _⏳ mengirim audio ke saluran, harap tunggu..._"
        );
    }

    #[test]
    fn deskripsi_dipotong_150_karakter() {
        let panjang = "a".repeat(200);
        let (teks, dipotong) = potong_deskripsi(&panjang).unwrap();
        assert_eq!(teks.len(), 150);
        assert!(dipotong);
        let (teks, dipotong) = potong_deskripsi("pendek").unwrap();
        assert_eq!(teks, "pendek");
        assert!(!dipotong);
        assert!(potong_deskripsi("").is_none());
    }

    #[test]
    fn pisah_idch_mengambil_id_dan_sisanya() {
        let (id, sisa) = pisah_idch("--idch 123@newsletter komang");
        assert_eq!(id.as_deref(), Some("123@newsletter"));
        assert_eq!(sisa, "komang");
        let (id, sisa) = pisah_idch("komang");
        assert_eq!(id, None);
        assert_eq!(sisa, "komang");
    }

    #[test]
    fn pilih_video_membuat_videos_pendek_dipilih_dulu() {
        let panjang = video(1200);
        let pendek = video(200);
        let daftar = vec![panjang.clone(), pendek.clone()];
        assert_eq!(pilih_video(&daftar), Some(&pendek));
        let daftar = vec![panjang.clone()];
        assert_eq!(pilih_video(&daftar), Some(&panjang));
        assert!(pilih_video(&[]).is_none());
    }

    #[test]
    fn tanpa_query_tampilkan_kartu_pakai() {
        let r = run(Stub::body("{}"), "");
        assert_eq!(text_of(&r), help("."));
    }
}
