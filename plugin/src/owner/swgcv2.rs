//! `plugins/owner/swgcv2.js` — post Group Status V2 ke grup pilihan.
//!
//! ## Penyimpangan
//!
//! - Posting story grup butuh `sock.relayMessage(jid, { groupStatusMessageV2 })`.
//!   Trait `Sock` tidak punya API group status, jadi tahap konfirmasi
//!   (`--confirm <id grup>`) berakhir di cabang error JS.
//! - Daftar grup tetap bisa ditampilkan: `sock.groupFetchAllParticipating()`
//!   ada padanannya di `Sock::semua_grup`.

use oktz_plugin_api::registry::{CardList, CardRow, CardSection, InteractiveButton};
use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

/// Teks bantuan `swgcv2.js:237-242`.
pub(crate) fn cara_pakai(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}swgcv2 teks` - Story teks\n\
         > Reply gambar/video/audio + `{prefix}swgcv2`\n\
         > Kirim gambar/video + caption `{prefix}swgcv2`"
    )
}

/// `swgcv2.js:271`.
pub(crate) const TANPA_GRUP: &str = "⚠️ *Bot tidak berada di grup manapun.*";

/// `swgcv2.js:58-60`.
pub(crate) const TANPA_PENDING: &str =
    "⚠️ *Tidak ada data pending. Silakan kirim ulang media + .swgcv2*";

/// `swgcv2.js:146-148` — gagal posting.
pub(crate) fn gagal_posting(pesan: &str) -> String {
    format!("❌ *ᴇʀʀᴏʀ*\n\n> Gagal posting story V2.\n> _{pesan}_")
}

/// `swgcv2.js:133`.
pub(crate) fn berhasil(nama_grup: &str) -> String {
    format!("✅ Berhasil up sw (V2) ke grup {nama_grup}")
}

/// Body kartu `swgcv2.js:297-301`.
pub(crate) fn body_kartu(jenis_media: &str, jumlah_grup: usize) -> String {
    format!(
        "📋 *ᴘɪʟɪʄ ɢʀᴜᴘ ᴜɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ ᴠ2*\n\n\
         > Media: *{jenis_media}*\n\
         > Total Grup: *{jumlah_grup}*\n\n\
         _Pilih grup dari daftar di bawah:_"
    )
}

/// Jenis media `swgcv2.js:282-290`.
pub(crate) fn jenis_media(teks: bool, gambar: bool, video: bool, audio: bool) -> &'static str {
    if teks {
        "Teks"
    } else if gambar {
        "Gambar"
    } else if video {
        "Video"
    } else if audio {
        "Audio"
    } else {
        "Media"
    }
}

/// Daftar grup `(id, subject)` dari `sock.groupFetchAllParticipating()`.
fn daftar_grup(ctx: &Ctx) -> Vec<(String, String)> {
    let Ok(grup) = ctx.sock().semua_grup() else {
        return Vec::new();
    };
    let Some(obj) = grup.as_object() else {
        return Vec::new();
    };
    obj.iter()
        .map(|(id, meta)| {
            let subject = meta
                .get("subject")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .unwrap_or("Unknown Group")
                .to_string();
            (id.clone(), subject)
        })
        .collect()
}

#[plugin(
    name = "swgcv2",
    alias = ["statusgrupv2"],
    category = "owner",
    description = "Post Group Status V2 ke grup pilihan",
    usage = ".swgcv2 <teks> atau reply media",
    example = ".swgcv2 Halo semua!",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn swgcv2(ctx: Ctx) -> Reply {
    let args = &ctx.args;
    let text = ctx.text.trim();

    // `--confirm <id grup>` (`swgcv2.js:53-151`).
    if args.first().map(|a| a.as_str()) == Some("--confirm")
        && args.len() > 1
    {
        let group_id = args[1].clone();
        let pending = ctx.session_get("swgcv2:pending");
        if pending.is_none() {
            return Reply::text(TANPA_PENDING);
        }
        // Posting group status butuh `relayMessage` — tidak ada di `Sock`.
        let _ = group_id;
        return Reply::text(gagal_posting(
            "group status V2 tidak didukung pada port Rust",
        ));
    }

    let quoted_gambar = ctx.quoted().is_some_and(|q| q.kind.is_image());
    let quoted_video = ctx.quoted().is_some_and(|q| q.kind.is_video());
    let quoted_audio = ctx.quoted().is_some_and(|q| q.kind.is_audio());
    let trigger_gambar = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Image);
    let trigger_video = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Video);
    let trigger_audio = ctx.is_media_kind(oktz_plugin_api::IncomingKind::Audio);

    let gambar = quoted_gambar || trigger_gambar;
    let video = quoted_video || trigger_video;
    let audio = quoted_audio || trigger_audio;
    let ada_media = gambar || video || audio;

    if !ada_media && text.is_empty() {
        return Reply::text(cara_pakai(&ctx.prefix));
    }

    ctx.session_set(
        "swgcv2:pending",
        serde_json::json!({ "text": text, "gambar": gambar, "video": video, "audio": audio }),
    );

    let grup = daftar_grup(&ctx);
    if grup.is_empty() {
        return Reply::text(TANPA_GRUP);
    }

    let rows: Vec<CardRow> = grup
        .iter()
        .map(|(id, subject)| {
            CardRow::new(subject.clone(), format!("{}swgcv2 --confirm {id}", ctx.prefix))
                .description(id.clone())
        })
        .collect();

    let list = CardList::new("🏠 Pilih Grup", vec![CardSection::new("Daftar Grup").rows(rows)])
        .icon("DEFAULT");

    let footer = {
        let nama = ctx.config().bot.name.clone();
        if nama.is_empty() {
            "Bot".to_string()
        } else {
            nama
        }
    };

    Reply::interactive(body_kartu(
        jenis_media(!text.is_empty(), gambar, video, audio),
        grup.len(),
    ))
    .with_footer(footer)
    .with_button(InteractiveButton::select(&list))
    .with_button(InteractiveButton::quick_reply(
        "❌ Batal",
        format!("{}cancelswgcv2", ctx.prefix),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str], text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "swgcv2".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            ..Default::default()
        };
        swgcv2(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("swgcv2").unwrap();
        assert_eq!(p.meta.name, "swgcv2");
        assert_eq!(p.meta.alias, ["statusgrupv2"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Post Group Status V2 ke grup pilihan");
        assert_eq!(p.meta.usage, ".swgcv2 <teks> atau reply media");
        assert_eq!(p.meta.example, ".swgcv2 Halo semua!");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_teks_tanpa_media_menampilkan_cara_pakai() {
        let r = run(&[], "");
        assert_eq!(
            teks(&r),
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
             > `.swgcv2 teks` - Story teks\n\
             > Reply gambar/video/audio + `.swgcv2`\n\
             > Kirim gambar/video + caption `.swgcv2`"
        );
    }

    #[test]
    fn tanpa_grup_menampilkan_pesan_kosong() {
        // `SockMati` → `semua_grup()` gagal → daftar grup kosong.
        let r = run(&["Halo"], "Halo");
        assert_eq!(teks(&r), TANPA_GRUP);
    }

    #[test]
    fn confirm_tanpa_pending_menampilkan_peringatan() {
        let r = run(&["--confirm", "120363@g.us"], "");
        assert_eq!(teks(&r), TANPA_PENDING);
    }

    #[test]
    fn jenis_media_sesuai_prioritas_js() {
        assert_eq!(jenis_media(true, false, false, false), "Teks");
        assert_eq!(jenis_media(false, true, false, false), "Gambar");
        assert_eq!(jenis_media(false, false, true, false), "Video");
        assert_eq!(jenis_media(false, false, false, true), "Audio");
        assert_eq!(jenis_media(false, false, false, false), "Media");
    }

    #[test]
    fn body_kartu_pola_js() {
        let t = body_kartu("Teks", 3);
        assert!(t.starts_with("📋 *ᴘɪʟɪʄ ɢʀᴜᴘ ᴜɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ ᴠ2*"));
        assert!(t.contains("> Media: *Teks*"));
        assert!(t.contains("> Total Grup: *3*"));
        assert!(t.ends_with("_Pilih grup dari daftar di bawah:_"));
    }
}
