//! `plugins/tools/cekidch.js` — cek ID dan info lengkap channel dari link.
//!
//! ## Penyimpangan yang tidak bisa ditiru
//!
//! JS memakai `sock.cekIDSaluran(text)` (`src/lib/socket.js:1312-1329`), yang
//! memanggil `sock.newsletterMetadata("INVITE", channelId)` — protokol
//! WhatsApp internal. Trait `Sock` di `oktz-plugin-api` tidak punya
//! `newsletter_metadata`, dan `lib/oktz-plugin-api` berada di luar wilayah
//! yang boleh disentuh, jadi metadata itu tidak bisa diambil apa adanya.
//!
//! Port ini mengambil halaman web channel resmi
//! (`https://whatsapp.com/channel/<id>`) dan mem-parsing meta tag-nya:
//! - `og:title` → nama channel.
//! - `meta[name=description]` → deskripsi (prefix `Follow <nama>'s WhatsApp
//!   Channel. ` dibuang).
//! - `og:description` → jumlah pengikut (`N followers`), kalau ada.
//! - `og:image` → foto channel (`preview`).
//!
//! Field yang benar-benar tidak ada di halaman web — `verification` dan
//! `creation_time` — memakai nilai default yang sama dengan JS:
//! `Unverified` dan `—`. `subscribers` memakai jumlah pengikut dari
//! `og:description`; kalau tidak ada, `0` (nilai default JS).
//!
//! Saat pengambilan gagal, port ini kembali ke `{ id: channelId }` — persis
//! apa yang JS lakukan di `catch` (`socket.js:1327`).

use oktz_plugin_api::registry::{CardHeader, InteractiveButton, MediaSource};
use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::json;

const BASE: &str = "https://whatsapp.com/channel/";

/// `formatDate` (`cekidch.js:20-25`) — `—` kalau timestamp kosong/falsy.
fn format_date(timestamp: Option<i64>) -> String {
    let Some(ts) = timestamp else {
        return "—".to_string();
    };
    if ts == 0 {
        return "—".to_string();
    }
    // `typeof timestamp === "number" && timestamp < 1e12 ? ts * 1000 : ts`
    let ms = if ts < 1_000_000_000_000 { ts * 1000 } else { ts };
    let parts = crate::common::jakarta_parts(ms);
    format!(
        "{:02}/{:02}/{} {:02}:{:02}",
        parts.2, parts.1, parts.0, parts.3, parts.4
    )
}

/// `formatSubs` (`cekidch.js:27-32`).
fn format_subs(count: i64) -> String {
    if count == 0 {
        return "0".to_string();
    }
    if count >= 1_000_000 {
        let v = count as f64 / 1_000_000.0;
        let s = format!("{v:.1}");
        return format!("{}M", s.strip_suffix(".0").unwrap_or(&s));
    }
    if count >= 1_000 {
        let v = count as f64 / 1_000.0;
        let s = format!("{v:.1}");
        return format!("{}K", s.strip_suffix(".0").unwrap_or(&s));
    }
    count.to_string()
}

/// Buang entity HTML yang dipakai `content="..."` di meta tag.
fn html_unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(idx) = rest.find("&") {
        out.push_str(&rest[..idx]);
        rest = &rest[idx..];
        let Some(end) = rest.find(';').filter(|e| *e <= 12) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let ent = &rest[1..end];
        let decoded = match ent {
            "amp" => Some("&".to_string()),
            "lt" => Some("<".to_string()),
            "gt" => Some(">".to_string()),
            "quot" => Some("\"".to_string()),
            "apos" | "#39" => Some("'".to_string()),
            "nbsp" => Some(" ".to_string()),
            "bull" => Some("\u{2022}".to_string()),
            "middot" => Some("\u{2022}".to_string()),
            _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                u32::from_str_radix(&ent[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
                    .map(|c| c.to_string())
            }
            _ if ent.starts_with('#') => ent[1..]
                .parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string()),
            _ => None,
        };
        match decoded {
            Some(s) => {
                out.push_str(&s);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Ambil nilai `content="..."` dari meta tag yang punya `property`/`name`
/// tertentu.
fn meta_content(html: &str, key: &str) -> Option<String> {
    let mut nilai = String::new();
    for pola in [format!("property=\"{key}\""), format!("name=\"{key}\"")] {
        let Some(pos) = html.find(&pola) else { continue };
        let tag = &html[pos..pos + html[pos..].find('>')?];
        let Some(cpos) = tag.find("content=\"") else { continue };
        let vstart = cpos + "content=\"".len();
        let vend = tag[vstart..].find('"')?;
        nilai = html_unescape(&tag[vstart..vstart + vend]);
        break;
    }
    if nilai.is_empty() {
        let needle = format!("{key}=\"");
        let start = html.find(&needle)? + needle.len();
        let end = html[start..].find('"')?;
        nilai = html_unescape(&html[start..start + end]);
    }
    Some(nilai)
}

/// Metadata channel yang berhasil dikumpulkan.
struct Metadata {
    id: String,
    name: String,
    subscribers: i64,
    description: String,
    preview: String,
    /// `metadata.verification` — halaman web tidak menyediakannya, jadi selalu
    /// `None` di port ini.
    verification: Option<String>,
}

/// `metadata.verification === 'VERIFIED' ? '✓ Verified' : 'Unverified'`
/// (`cekidch.js:64`).
fn status_verifikasi(verification: Option<&str>) -> &'static str {
    if verification == Some("VERIFIED") {
        "✓ Verified"
    } else {
        "Unverified"
    }
}

/// Ambil ID channel dari link — `socket.js:1313-1320`.
fn channel_id_dari(url: &str) -> String {
    if let Some(rest) = url.split("whatsapp.com/channel/").nth(1) {
        return rest.split(['/', '?', '#'].as_slice()).next().unwrap_or("").trim().to_string();
    }
    if let Some(rest) = url.split("wa.me/channel/").nth(1) {
        return rest.split(['/', '?', '#'].as_slice()).next().unwrap_or("").trim().to_string();
    }
    url.to_string()
}

/// Ambil jumlah pengikut dari `og:description` ("Channel • 25 followers • ...").
fn subscribers_dari_og(html: &str) -> Option<i64> {
    let og = meta_content(html, "og:description")?;
    let idx = og.find("followers")?;
    let bagian = &og[..idx];
    let digit = bagian
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit() || *c == ' ')
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let digit = digit.trim();
    if digit.is_empty() {
        None
    } else {
        digit.parse::<i64>().ok()
    }
}

fn scrape_metadata(ctx: &Ctx, id: &str) -> Metadata {
    let fallback = Metadata {
        id: id.to_string(),
        name: String::new(),
        subscribers: 0,
        description: String::new(),
        preview: String::new(),
        verification: None,
    };
    let Ok(res) = ctx.http.get(&format!("{BASE}{id}")) else {
        return fallback;
    };
    let html = res.text();
    if html.is_empty() {
        return fallback;
    }
    let name = meta_content(&html, "og:title").unwrap_or_default();
    let preview = meta_content(&html, "og:image").unwrap_or_default();
    let mut description = meta_content(&html, "description").unwrap_or_default();
    // Buang prefix "Follow <Nama>'s WhatsApp Channel. ".
    if let Some(idx) = description.find(". ") {
        let head = &description[..idx];
        if head.starts_with("Follow ") && head.ends_with("'s WhatsApp Channel") {
            description = description[idx + 2..].to_string();
        }
    }
    Metadata {
        id: id.to_string(),
        name,
        subscribers: subscribers_dari_og(&html).unwrap_or(0),
        description,
        preview,
        verification: None,
    }
}

fn usage(prefix: &str) -> String {
    format!(
        "ℹ️ *INFORMASI PENGGUNAAN*\n\n\
         Silakan masukkan link channel WhatsApp yang ingin Anda cek informasinya secara detail.\n\n\
         *CONTOH PENGGUNAAN:*\n\
         • `{prefix}cekidch https://whatsapp.com/channel/xxxxx`"
    )
}

fn link_tidak_valid() -> &'static str {
    "❌ *LINK TIDAK VALID*\n\nPastikan link yang Anda masukkan adalah link channel WhatsApp yang sah dan benar."
}

fn channel_tidak_ditemukan() -> &'static str {
    "❌ *CHANNEL TIDAK DITEMUKAN*\n\nMaaf, sistem tidak dapat menemukan informasi dari channel tersebut. Mungkin link sudah kedaluwarsa atau channel telah dihapus."
}

#[plugin(
    name = "cekidch",
    alias = ["idch", "channelid", "infoch"],
    category = "tools",
    description = "Cek ID dan info lengkap channel dari link",
    usage = ".cekidch <link channel>",
    example = ".cekidch https://whatsapp.com/channel/xxxxx",
    cooldown = 5,
    energi = 0,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekidch(ctx: Ctx) -> Reply {
    let text = ctx.text.trim().to_string();

    if text.is_empty() {
        return Reply::text(usage(&ctx.prefix));
    }

    if !text.contains("https://whatsapp.com/channel/") {
        return Reply::text(link_tidak_valid());
    }

    let id = channel_id_dari(&text);
    let meta = scrape_metadata(&ctx, &id);

    if meta.id.is_empty() {
        return Reply::text(channel_tidak_ditemukan()).with_reaction("❌");
    }

    let ch_name = if meta.name.is_empty() {
        "Unknown".to_string()
    } else {
        meta.name.clone()
    };
    let ch_desc = if meta.description.is_empty() {
        "—".to_string()
    } else {
        meta.description.clone()
    };
    // `metadata.verification === 'VERIFIED'` — halaman web tidak memberi
    // status verifikasi, jadi nilainya selalu "Unverified".
    let ch_verified = status_verifikasi(meta.verification.as_deref());
    let ch_created = format_date(None);
    let ch_pic = if meta.preview == "https://mmg.whatsapp.net" {
        "https://files.catbox.moe/lp9tpd.jpg".to_string()
    } else if meta.preview.is_empty() {
        "https://files.catbox.moe/lp9tpd.jpg".to_string()
    } else {
        meta.preview.clone()
    };

    let desc_preview = if ch_desc.chars().count() > 120 {
        let cut: String = ch_desc.chars().take(120).collect();
        format!("{cut}...")
    } else {
        ch_desc.clone()
    };

    let info_text = format!(
        "Berikut adalah detail informasi lengkap mengenai channel yang Anda cari:\n\n\
         *RINCIAN CHANNEL:*\n\
         • Nama: *{ch_name}*\n\
         • ID Channel: `{id}`\n\
         • Subscriber: *{subs}*\n\
         • Status: *{ch_verified}*\n\
         • Dibuat Pada: *{ch_created}*\n\n\
         *DESKRIPSI:*\n\
         {desc_preview}",
        id = meta.id,
        subs = format_subs(meta.subscribers),
    );

    let buttons = vec![
        InteractiveButton::new(
            "cta_copy",
            json!({
                "display_text": "📋 Ambil ID Saluran nya",
                "copy_code": meta.id,
            }),
        ),
        InteractiveButton::new(
            "cta_url",
            json!({
                "display_text": "🔗 Buka Channel nya",
                "url": text,
            }),
        ),
    ];

    let footer = format!(
        "© {}",
        if ctx.config().bot.name.is_empty() {
            "Foto-AI"
        } else {
            &ctx.config().bot.name
        }
    );

    Reply::interactive(info_text)
        .with_footer(footer)
        .with_header(CardHeader::new().image(MediaSource::url(ch_pic)))
        .with_buttons(buttons)
        .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpGet, HttpResponse};
    use std::sync::{Arc, Mutex};

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct Stub {
        balasan: Option<String>,
        seen: Mutex<Vec<String>>,
    }

    impl Stub {
        fn body(v: &str) -> Arc<Stub> {
            Arc::new(Stub {
                balasan: Some(v.to_string()),
                seen: Mutex::new(Vec::new()),
            })
        }
        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                balasan: None,
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.seen.lock().unwrap().push(url.to_string());
            match &self.balasan {
                Some(b) => Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(b.clone().into_bytes()),
                        mimetype: None,
                    },
                    headers: Vec::new(),
                }),
                None => Err(HttpError {
                    message: "offline".into(),
                }),
            }
        }
        fn post_json(
            &self,
            _: &str,
            _: &serde_json::Value,
        ) -> Result<HttpResponse, HttpError> {
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
    }

    fn run(http: Arc<Stub>, text: &str) -> Reply {
        let ctx = Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "cekidch".into(),
            http,
            ..Ctx::default()
        };
        cekidch(ctx)
    }

    fn body(r: &Reply) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_kosong() {
        assert_eq!(
            body(&run(Stub::offline(), "")),
            "ℹ️ *INFORMASI PENGGUNAAN*\n\n\
             Silakan masukkan link channel WhatsApp yang ingin Anda cek informasinya secara detail.\n\n\
             *CONTOH PENGGUNAAN:*\n\
             • `.cekidch https://whatsapp.com/channel/xxxxx`"
        );
    }

    #[test]
    fn output_sama_dengan_js_link_tidak_valid() {
        let r = run(Stub::offline(), "https://example.com/abc");
        assert_eq!(
            body(&r),
            "❌ *LINK TIDAK VALID*\n\nPastikan link yang Anda masukkan adalah link channel WhatsApp yang sah dan benar."
        );
    }

    #[test]
    fn kartu_menampilkan_rincian_channel() {
        let html = "<html><head><meta property=\"og:title\" content=\"OKTZ\" />\
                    <meta property=\"og:image\" content=\"https://mmg.whatsapp.net/x.jpg\" />\
                    <meta property=\"og:description\" content=\"Channel • 25 followers • halo\" />\
                    <meta name=\"description\" content=\"Follow OKTZ's WhatsApp Channel. *OKTZ OFFICIAL*\" />\
                    </head><body></body></html>";
        let r = run(Stub::body(html), "https://whatsapp.com/channel/0029ABC");
        let t = body(&r);
        assert!(t.contains("• Nama: *OKTZ*"), "{t}");
        assert!(t.contains("• ID Channel: `0029ABC`"), "{t}");
        assert!(t.contains("• Subscriber: *25*"), "{t}");
        assert!(t.contains("• Status: *Unverified*"), "{t}");
        assert!(t.contains("• Dibuat Pada: *—*"), "{t}");
        assert!(t.contains("*DESKRIPSI:*\n*OKTZ OFFICIAL*"), "{t}");
    }

    #[test]
    fn format_subs_sama_dengan_js() {
        assert_eq!(format_subs(0), "0");
        assert_eq!(format_subs(999), "999");
        assert_eq!(format_subs(1000), "1K");
        assert_eq!(format_subs(1500), "1.5K");
        assert_eq!(format_subs(1_000_000), "1M");
        assert_eq!(format_subs(2_500_000), "2.5M");
    }

    #[test]
    fn id_channel_diambil_dari_link() {
        assert_eq!(
            channel_id_dari("https://whatsapp.com/channel/0029ABC?x=1"),
            "0029ABC"
        );
        assert_eq!(channel_id_dari("https://wa.me/channel/0029XYZ/1"), "0029XYZ");
    }
}
