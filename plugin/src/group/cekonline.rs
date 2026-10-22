//! `plugins/group/cekonline.js` — cek member yang online di grup.
//!
//! Yang dipindah: seluruh perakitan teks laporan (`cekonline.js:26-99`),
//! termasuk kotak `╭┈┈⬡「 … 」` dan penanda admin `👑`.
//!
//! Penyimpangan (satu-satunya): JS berlangganan presence lewat
//! `sock.presenceSubscribe(p.id)` lalu menunggu event `presence.update`
//! (`cekonline.js:46-60`). `Sock` di `oktz-plugin-api` hanya punya
//! `presence()` yang memetakan `sock.sendPresenceUpdate` (kirim status
//! sendiri), dan tidak ada jalur event (`sock.ev.on`) sama sekali. Karena itu
//! daftar "member online" tidak bisa diisi di Rust — hasil yang paling dekat
//! adalah cabang `onlineMembers.length === 0` dari JS, dan itu yang dipakai
//! di sini. Kotak "member online" tetap ikut port supaya saat API presence
//! tersedia tinggal diisi.
//!
//! Reaksi: JS memasang `🔍` di awal lalu menimpanya dengan `✅`/`❌`/`☢`.
//! WhatsApp hanya menyimpan satu reaksi per pesan, jadi Rust hanya mengirim
//! reaksi terakhir.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `cekonline.js:38` — roster kosong = metadata/anggota grup tidak terbaca.
const TANPA_MEMBER: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa mendapatkan data member grup";

/// `cekonline.js:41` — pesan "menunggu response" sebelum laporan dikirim.
fn menunggu(jumlah: usize) -> String {
    format!(
        "🔍 *ᴍᴇɴᴄᴀʀɪ ᴍᴇᴍʙᴇʀ ᴏɴʟɪɴᴇ...*\n\n\
         > Menunggu response dari {jumlah} member\n\
         > Estimasi: 5-10 detik"
    )
}

/// `cekonline.js:76-82` — kotak info grup.
fn info_grup(subject: &str, total: usize, online: usize) -> String {
    format!(
        "📊 *ᴄᴇᴋ ᴏɴʟɪɴᴇ*\n\n\
         ╭┈┈⬡「 📋 *ɪɴꜰᴏ ɢʀᴜᴘ* 」\n\
         ┃ 👥 ɴᴀᴍᴀ: *{subject}*\n\
         ┃ 👤 ᴛᴏᴛᴀʟ: `{total}` member\n\
         ┃ 🟢 ᴏɴʟɪɴᴇ: `{online}` member\n\
         ╰┈┈⬡\n\n"
    )
}

/// `cekonline.js:89-99` — kotak member yang online.
///
/// `members` berisi `(nomor, admin, ikon_presence)`. `presences` di JS berisi
/// `lastKnownPresence` per JID: `available` → `🟢`, `composing` → `⌨️`,
/// `recording` → `🎤`.
fn member_online(members: &[(String, bool, &str)]) -> String {
    let mut text = String::from("╭┈┈⬡「 🟢 *ᴍᴇᴍʙᴇʀ ᴏɴʟɪɴᴇ* 」\n");
    let mut count = 0usize;
    for (number, is_admin, ikon) in members {
        if count >= 50 {
            text.push_str(&format!(
                "┃ ... dan {} member lainnya\n",
                members.len() - 50
            ));
            break;
        }
        let admin_badge = if *is_admin { " 👑" } else { "" };
        text.push_str(&format!("┃ {ikon} @{number}{admin_badge}\n"));
        count += 1;
    }
    text.push_str("╰┈┈⬡\n\n");
    text.push_str("> 🟢 Online | ⌨️ Mengetik | 🎤 Rekam Audio");
    text
}

/// `cekonline.js:83-87` — cabang "tidak ada member online".
fn tanpa_member_online() -> String {
    String::from(
        "> _Tidak ada member yang terdeteksi online_\n\
         > _Pastikan member telah membuka WA_",
    )
}

/// `m.groupMetadata.subject` (`cekonline.js:78`) — nama grup dari metadata WA.
///
/// `None` kalau metadata tidak terbaca; di JS ini melempar dan jatuh ke
/// cabang `catch` (`te(...)`).
fn subject_grup(ctx: &Ctx) -> Option<String> {
    ctx.sock()
        .metadata_grup(&ctx.chat)
        .ok()
        .and_then(|meta| {
            meta.get("subject")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
}

/// Daftar member yang terdeteksi online.
///
/// Selalu kosong di Rust: `sock.presenceSubscribe` + `sock.ev.on('presence.
/// update')` tidak ada padanannya di `Sock` (lihat doc-comment modul).
fn presence_online(_ctx: &Ctx) -> Vec<(String, bool, &'static str)> {
    Vec::new()
}

#[plugin(
    name = "cekonline",
    alias = ["checkonline", "online", "siapayangonline", "whosonline"],
    category = "group",
    description = "Cek member yang online di grup",
    usage = ".cekonline",
    example = ".cekonline",
    cooldown = 60,
    energi = 1,
    is_group = true,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekonline(ctx: Ctx) -> Reply {
    // `cekonline.js:28-30` — `m.groupMembers` = roster pesan ini.
    let total = ctx.roster.participants.len();

    if total == 0 {
        return Reply::text(TANPA_MEMBER).with_reaction("❌");
    }

    // `cekonline.js:26` — `m.groupMetadata.subject`.
    let Some(subject) = subject_grup(&ctx) else {
        return Reply::text(crate::common::error_template(&ctx)).with_reaction("☢");
    };

    let online = presence_online(&ctx);
    let mut text = info_grup(&subject, total, online.len());
    if online.is_empty() {
        text.push_str(&tanpa_member_online());
    } else {
        text.push_str(&member_online(&online));
    }

    // `cekonline.js:41` lalu `:98` — dua pesan berurutan; yang terakhir
    // meng-mention member online (`mentions: []` kalau tidak ada → tanpa
    // mention, jadi `Reply::text_no_mention`).
    Reply::sequence(vec![
        Reply::text(menunggu(total)),
        Reply::text_no_mention(text),
    ])
    .with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Participant, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        subject: Option<String>,
    }

    impl Sock for StubSock {
        fn metadata_grup(&self, _jid: &str) -> Result<serde_json::Value, SockError> {
            match &self.subject {
                Some(s) => Ok(serde_json::json!({ "subject": s })),
                None => Err(SockError::baru("tidak ada metadata")),
            }
        }
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<oktz_plugin_api::AdaWhatsapp>, SockError> {
            Err(SockError::baru("x"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("x"))
        }
        fn semua_grup(&self) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn ubah_peserta(
            &self,
            _: &str,
            _: &[String],
            _: oktz_plugin_api::AksiPeserta,
        ) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn set_foto_profil(&self, _: &str, _: &[u8]) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn keluar_grup(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn kode_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn info_undangan(&self, _: &str) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn buat_grup(&self, _: &str, _: &[String]) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn tandai_dibaca(&self, _: &[serde_json::Value]) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn status_teks(&self, _: &str) -> Result<serde_json::Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<serde_json::Value>, SockError> {
            Err(SockError::baru("x"))
        }
    }

    fn ctx_dengan(
        sock: Arc<StubSock>,
        jumlah_member: usize,
        push_name: &str,
    ) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cekonline".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            push_name: push_name.into(),
            is_group: true,
            sock,
            roster: oktz_plugin_api::registry::GroupRoster {
                participants: (0..jumlah_member)
                    .map(|i| Participant {
                        jid: format!("62800{i}@s.whatsapp.net"),
                        is_admin: false,
                        is_owner: false,
                        data: Default::default(),
                    })
                    .collect(),
                is_bot_admin: true,
            },
            ..Default::default()
        }
    }

    fn sock_dengan_subject(subject: &str) -> Arc<StubSock> {
        Arc::new(StubSock {
            subject: Some(subject.to_string()),
        })
    }

    /// Teks dari balasan — descends `Reaction`/`Mutate` lalu ambil elemen
    /// `Sequence` pertama yang punya teks.
    fn teks(r: &Reply) -> String {
        let mut out = String::new();
        let mut ambil = |reply: &Reply| {
            if let Reply::Sequence(list) = reply {
                for item in list {
                    if let Some(t) = item.text_body() {
                        out.push_str(t);
                        out.push('\n');
                    }
                }
            } else if let Some(t) = reply.text_body() {
                out.push_str(t);
            }
        };
        ambil(r.without_mutations().tanpa_reaksi());
        out
    }

    fn reaksi(r: &Reply) -> Option<String> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.clone()),
            _ => None,
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("cekonline").unwrap();
        assert_eq!(p.meta.name, "cekonline");
        assert_eq!(
            p.meta.alias,
            ["checkonline", "online", "siapayangonline", "whosonline"]
        );
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Cek member yang online di grup");
        assert_eq!(p.meta.usage, ".cekonline");
        assert_eq!(p.meta.example, ".cekonline");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 1);
        assert!(p.meta.is_group);
        assert!(!p.meta.is_private && !p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["checkonline", "online", "siapayangonline", "whosonline"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|p| p.meta.name.as_str()),
                Some("cekonline"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn output_sama_dengan_js_kosong_member() {
        let r = cekonline(ctx_dengan(sock_dengan_subject("Grup"), 0, "Budi"));
        assert_eq!(teks(&r), TANPA_MEMBER);
        assert_eq!(reaksi(&r), Some("❌".to_string()));
    }

    #[test]
    fn output_sama_dengan_js_laporan_kosong_online() {
        let r = cekonline(ctx_dengan(sock_dengan_subject("Grup Official"), 3, "Budi"));
        let t = teks(&r);
        assert!(t.contains("🔍 *ᴍᴇɴᴄᴀʀɪ ᴍᴇᴍʙᴇʀ ᴏɴʟɪɴᴇ...*"), "{t}");
        assert!(t.contains("> Menunggu response dari 3 member"), "{t}");
        assert!(t.contains("📊 *ᴄᴇᴋ ᴏɴʟɪɴᴇ*"), "{t}");
        assert!(t.contains("╭┈┈⬡「 📋 *ɪɴꜰᴏ ɢʀᴜᴘ* 」"), "{t}");
        assert!(t.contains("┃ 👥 ɴᴀᴍᴀ: *Grup Official*"), "{t}");
        assert!(t.contains("┃ 👤 ᴛᴏᴛᴀʟ: `3` member"), "{t}");
        assert!(t.contains("┃ 🟢 ᴏɴʟɪɴᴇ: `0` member"), "{t}");
        assert!(t.contains("╰┈┈⬡"), "{t}");
        assert!(t.contains("> _Tidak ada member yang terdeteksi online_"), "{t}");
        assert!(t.contains("> _Pastikan member telah membuka WA_"), "{t}");
        assert_eq!(reaksi(&r), Some("✅".to_string()));
    }

    #[test]
    fn metadata_grup_gagal_menghasilkan_template_error() {
        let sock = Arc::new(StubSock { subject: None });
        let r = cekonline(ctx_dengan(sock, 2, "Budi"));
        assert!(teks(&r).contains("Silahkan coba lagi nanti, Budi"), "{:?}", teks(&r));
        assert_eq!(reaksi(&r), Some("☢".to_string()));
    }

    #[test]
    fn kotak_member_online_menandai_admin_dan_membatasi_50() {
        let members: Vec<(String, bool, &str)> = (0..60)
            .map(|i| (format!("628{i:03}"), i % 2 == 0, "🟢"))
            .collect();
        let t = member_online(&members);
        assert!(t.starts_with("╭┈┈⬡「 🟢 *ᴍᴇᴍʙᴇʀ ᴏɴʟɪɴᴇ* 」"), "{t}");
        assert!(t.contains("┃ 🟢 @628000 👑"), "{t}");
        assert!(t.contains("┃ 🟢 @628001"), "{t}");
        assert!(t.contains("┃ ... dan 10 member lainnya"), "{t}");
        assert!(t.ends_with("> 🟢 Online | ⌨️ Mengetik | 🎤 Rekam Audio"), "{t}");
    }

    #[test]
    fn ikon_presence_ikut_mewarisi_status_js() {
        let members = vec![
            ("628111".to_string(), false, "⌨️"),
            ("628222".to_string(), false, "🎤"),
        ];
        let t = member_online(&members);
        assert!(t.contains("┃ ⌨️ @628111"), "{t}");
        assert!(t.contains("┃ 🎤 @628222"), "{t}");
    }
}
