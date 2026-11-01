//! `plugins/fun/tolak.js` — menolak tembakan dari seseorang.
//!
//! ## Penyimpangan
//!
//! - **Baca data user lain.** `db.getUser(shooterJid)` hanya bisa dijawab
//!   kalau penembak ada di roster grup (`Participant::data`) — pola
//!   `putus.rs`/`tembak.rs`. Untuk penembak yang tidak ada di grup, data
//!   dianggap kosong.
//! - **JID bot** (`m.botNumber`) tidak ada di `Ctx`, jadi guard
//!   `shooterJid === m.botNumber` ("Bot tidak punya hati untuk ditolak!")
//!   dilewati.
//! - **`delete x.fun.pasangan` / `delete x.fun.tembakTarget`** ditulis `null` —
//!   `Mutation::SetUser` adalah merge partial (pola `putus.rs`); pembaca JS
//!   `fun?.pasangan` memperlakukan `null` sama dengan tidak ada.
//! - Sesi `global.tembakSessions` dibaca/dihapus lewat awalan
//!   `tembakSessions:` di map global `Ctx::sessions` (lihat `tembak.rs`).

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::{Value, json};

use crate::common::jid_number;

/// Awalan sesi milik `tembak.js` di map global (lihat `fun/tembak.rs`).
const SESSION_PREFIX: &str = "tembakSessions";

fn session_key(chat: &str, target: &str) -> String {
    format!("{SESSION_PREFIX}:{chat}_{target}")
}

/// `db.getUser(jid) || {}` — hanya untuk JID yang ada di roster.
fn data_user(ctx: &Ctx, jid: &str) -> Value {
    ctx.roster
        .participants
        .iter()
        .find(|p| p.jid == jid)
        .map(|p| p.data.clone())
        .unwrap_or(Value::Null)
}

/// Kartu cara pakai (`tolak.js:42-46`).
fn cara_pakai(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > Reply pesan tembakan + `{prefix}tolak`\n\
         > Atau `{prefix}tolak @tag`"
    )
}

/// Pesan penolakan (`tolak.js:93-98`) — teks yang sama dipakai answer handler
/// di `tembak.js`.
fn tolak_text(shooter_num: &str, sender_num: &str) -> String {
    format!(
        "💔 *WADUHH, YANG SABAR YAK* @{shooter_num}\n\n\
         @{sender_num} menolak @{shooter_num} sebagai pacarnya\n\n\
         Sabar ya, masih banyak yang lain! 😢"
    )
}

#[plugin(
    name = "tolak",
    alias = ["reject", "no", "gaktau"],
    category = "fun",
    description = "Menolak tembakan dari seseorang",
    usage = ".tolak @tag",
    example = ".tolak @628xxx",
    cooldown = 5,
    energi = 0,
    is_group = true,
    is_private = false
)]
pub fn tolak(ctx: Ctx) -> Reply {
    // Urutan penembak: quoted → mention → sesi tembakan (`tolak.js:24-39`).
    let shooter_jid = if let Some(q) = ctx.quoted() {
        Some(q.sender.clone()).filter(|s| !s.is_empty())
    } else if let Some(m) = ctx.mentioned.first() {
        Some(m.clone())
    } else {
        // Sesi dengan target = pengirim di chat ini.
        let map = ctx.sessions.lock().unwrap_or_else(|e| e.into_inner());
        map.iter()
            .filter(|(k, v)| {
                k.starts_with(&format!("{SESSION_PREFIX}:"))
                    && v.get("target").and_then(Value::as_str) == Some(ctx.sender.as_str())
                    && v.get("chat").and_then(Value::as_str) == Some(ctx.chat.as_str())
            })
            .next()
            .and_then(|(_, v)| v.get("shooter").and_then(Value::as_str))
            .map(str::to_string)
    };

    let Some(shooter_jid) = shooter_jid else {
        return Reply::text(cara_pakai(&ctx.prefix));
    };

    if shooter_jid == ctx.sender {
        return Reply::text("❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa menolak diri sendiri!");
    }
    // Guard `shooterJid === m.botNumber` dilewati — lihat catatan kepala.

    let shooter_data = data_user(&ctx, &shooter_jid);
    let shooter_num = jid_number(&shooter_jid).to_string();

    // Penembak tidak sedang menembak pengirim (`tolak.js:63-72`).
    let pasangan = shooter_data
        .get("fun")
        .and_then(|f| f.get("pasangan"))
        .and_then(Value::as_str);
    let tembak_target = shooter_data
        .get("fun")
        .and_then(|f| f.get("tembakTarget"))
        .and_then(Value::as_str);
    if pasangan != Some(ctx.sender.as_str()) && tembak_target != Some(ctx.sender.as_str()) {
        return Reply::Text {
            text: format!(
                "❌ *ᴛɪᴅᴀᴋ ᴍᴇɴᴇᴍʙᴀᴋ*\n\n> @{shooter_num} tidak sedang menembakmu"
            ),
            mentions: vec![shooter_jid],
        };
    }

    // Hapus pasangan/tembakan, catat hitungan ditolak (`tolak.js:74-82`).
    let ditolak = shooter_data
        .get("fun")
        .and_then(|f| f.get("ditolakCount"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        + 1;
    let mutations = vec![
        Mutation::SetUser {
            jid: shooter_jid.clone(),
            data: json!({ "fun": { "pasangan": null, "tembakTarget": null, "ditolakCount": ditolak } }),
        },
        Mutation::SetUser {
            jid: ctx.sender.clone(),
            data: json!({ "fun": { "pasangan": null } }),
        },
    ];

    // `delete global.tembakSessions[`${m.chat}_${m.sender}`]`.
    ctx.session_take(&session_key(&ctx.chat, &ctx.sender));

    Reply::Text {
        text: tolak_text(&shooter_num, &jid_number(&ctx.sender)),
        mentions: vec![ctx.sender.clone(), shooter_jid],
    }
    .mutating(mutations)
    .with_reaction("💔")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::{GroupRoster, Participant};

    const ME: &str = "628111@s.whatsapp.net";
    const SHOOTER: &str = "628222@s.whatsapp.net";

    fn anggota(jid: &str, data: Value) -> Participant {
        Participant {
            jid: jid.into(),
            is_admin: false,
            is_owner: false,
            data,
        }
    }

    fn run(data_shooter: Value, quoted: Option<&str>, mention: Option<&str>, sesi_ada: bool) -> Reply {
        let sessions = oktz_plugin_api::new_session_store();
        if sesi_ada {
            sessions.lock().unwrap().insert(
                session_key("120363@g.us", ME),
                json!({ "shooter": SHOOTER, "target": ME, "chat": "120363@g.us", "timestamp": 0 }),
            );
        }
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tolak".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            quoted: quoted.map(|s| oktz_plugin_api::QuotedMessage {
                id: "Q".into(),
                sender: s.into(),
                chat: "120363@g.us".into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
                timestamp: 0,
            }),
            mentioned: mention.map(str::to_string).into_iter().collect(),
            roster: GroupRoster {
                participants: vec![anggota(ME, Value::Null), anggota(SHOOTER, data_shooter)],
                is_bot_admin: false,
            },
            sessions,
            ..Default::default()
        };
        tolak(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("tolak").unwrap();
        assert_eq!(p.meta.name, "tolak");
        assert_eq!(p.meta.alias, ["reject", "no", "gaktau"]);
        assert_eq!(p.meta.category, "fun");
        assert_eq!(p.meta.description, "Menolak tembakan dari seseorang");
        assert_eq!(p.meta.usage, ".tolak @tag");
        assert_eq!(p.meta.example, ".tolak @628xxx");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_admin);
        for a in ["reject", "no", "gaktau"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "tolak");
        }
    }

    #[test]
    fn tanpa_penembak_menampilkan_cara_pakai() {
        let r = run(Value::Null, None, None, false);
        assert_eq!(teks(&r), cara_pakai("."));
        assert!(cara_pakai(".").contains("Reply pesan tembakan + `.tolak`"));
    }

    #[test]
    fn tanpa_quoted_mention_pakai_sesi_tembakan() {
        // Sesi ada → penembak dari sesi; data roster kosong berarti bukan
        // `db.getUser` lengkap, tapi guard butuh tembakTarget/pasangan.
        let data = json!({ "fun": { "tembakTarget": ME } });
        let r = run(data, None, None, true);
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "💔"),
            other => panic!("harus Reaction, dapat {other:?}"),
        }
    }

    #[test]
    fn menolak_diri_sendiri_ditolak() {
        let r = run(Value::Null, Some(ME), None, false);
        assert_eq!(teks(&r), "❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa menolak diri sendiri!");
    }

    #[test]
    fn penembak_tidak_menembak_ditolak_dengan_mention() {
        let r = run(json!({ "name": "Anu" }), Some(SHOOTER), None, false);
        assert_eq!(
            teks(&r),
            "❌ *ᴛɪᴅᴀᴋ ᴍᴇɴᴇᴍʙᴀᴋ*\n\n> @628222 tidak sedang menembakmu"
        );
        match r.without_mutations() {
            Reply::Text { mentions, .. } => assert_eq!(mentions, &[SHOOTER.to_string()]),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn penolakan_normal_menghapus_dan_menghitung() {
        let data = json!({ "fun": { "tembakTarget": ME, "ditolakCount": 4 } });
        let r = run(data, Some(SHOOTER), None, false);
        let (emoji, isi) = match r.without_mutations() {
            Reply::Reaction { emoji, reply } => (emoji.clone(), reply.as_ref().clone()),
            other => panic!("harus Reaction, dapat {other:?}"),
        };
        assert_eq!(emoji, "💔");
        match isi {
            Reply::Text { text, mentions } => {
                assert_eq!(text, tolak_text("628222", "628111"));
                assert!(text.starts_with("💔 *WADUHH, YANG SABAR YAK* @628222"), "{text}");
                assert_eq!(mentions, &[ME.to_string(), SHOOTER.to_string()]);
            }
            other => panic!("harus Text, dapat {other:?}"),
        }
        let muts = r.mutations();
        assert_eq!(muts.len(), 2);
        match &muts[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, SHOOTER);
                assert_eq!(data["fun"]["pasangan"], Value::Null);
                assert_eq!(data["fun"]["tembakTarget"], Value::Null);
                assert_eq!(data["fun"]["ditolakCount"], 5);
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
        match &muts[1] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, ME);
                assert_eq!(data["fun"]["pasangan"], Value::Null);
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn sesi_tembakan_dihapus_setelah_menolak() {
        let sessions = oktz_plugin_api::new_session_store();
        sessions.lock().unwrap().insert(
            session_key("120363@g.us", ME),
            json!({ "shooter": SHOOTER, "target": ME, "chat": "120363@g.us", "timestamp": 0 }),
        );
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tolak".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            mentioned: vec![SHOOTER.into()],
            roster: GroupRoster {
                participants: vec![
                    anggota(ME, Value::Null),
                    anggota(SHOOTER, json!({ "fun": { "tembakTarget": ME } })),
                ],
                is_bot_admin: false,
            },
            sessions: sessions.clone(),
            ..Default::default()
        };
        let r = tolak(ctx);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "💔"
        ));
        assert!(
            sessions
                .lock()
                .unwrap()
                .get(&session_key("120363@g.us", ME))
                .is_none(),
            "sesi tembakan harus dihapus"
        );
    }

    #[test]
    fn mention_tanpa_quote_memakai_mention_sebagai_penembak() {
        let data = json!({ "fun": { "pasangan": ME } });
        let r = run(data, None, Some(SHOOTER), false);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "💔"
        ));
    }
}
