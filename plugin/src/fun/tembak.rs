//! `plugins/fun/tembak.js` — menembak seseorang untuk pacaran.
//!
//! ## Penyimpangan
//!
//! - **Baca data user lain.** `db.getUser(targetJid)` hanya bisa dijawab kalau
//!   target ada di roster grup (`Participant::data`) — pola `putus.rs`. Untuk
//!   target yang tidak ada di grup, data dianggap kosong (`{}`), sama seperti
//!   user yang belum pernah ada di store.
//! - **JID bot** (`m.botNumber`) tidak ada di `Ctx`, jadi guard
//!   `targetJid === m.botNumber` ("Bot tidak bisa pacaran!") dilewati.
//! - **`sentMsg?.key?.id`** yang disimpan ke sesi tidak bisa ditiru — `Reply`
//!   tidak pernah mengembalikan id pesan terkirim. Tidak ada yang membacanya:
//!   `answerHandler` JS mencari sesi lewat `target` + `chat`, bukan id.
//! - **Kedaluwarsa 1 jam** di JS lewat `setTimeout` per sesi; di sini dicek
//!   malas saat answer handler membaca sesi.
//! - **`saluranCtx()` + `{ contextInfo }`** — mention target + pengirim sudah
//!   tercakup oleh `Reply::Text.mentions`.
//! - **`delete x.fun.pasangan`** ditulis `null` — `Mutation::SetUser` adalah
//!   merge partial (pola `putus.rs`), pembaca JS `fun?.pasangan` memperlakukan
//!   `null` sama dengan tidak ada.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::{Value, json};

use crate::common::{jid_number, now_ms};

/// Peta sesi JS punya sendiri (`global.tembakSessions`) — awalan membedakannya
/// di map global `Ctx::sessions`.
const SESSION_PREFIX: &str = "tembakSessions";

/// Umur sesi tembakan: 1 jam (`tembak.js:120-123`).
const EXPIRED_MS: i64 = 3_600_000;

fn session_key(chat: &str, target: &str) -> String {
    format!("{SESSION_PREFIX}:{chat}_{target}")
}

/// `args[0].replace(/[^0-9]/g, '')` dengan syarat panjang `> 5 && < 20`
/// (`tembak.js:32-36`).
fn target_dari_args(arg: Option<&String>) -> Option<String> {
    let arg = arg?;
    let num: String = arg.chars().filter(|c| c.is_ascii_digit()).collect();
    if num.len() > 5 && num.len() < 20 {
        Some(format!("{num}@s.whatsapp.net"))
    } else {
        None
    }
}

/// `db.getUser(targetJid) || {}` — hanya untuk JID yang ada di roster.
fn data_user(ctx: &Ctx, jid: &str) -> Value {
    ctx.roster
        .participants
        .iter()
        .find(|p| p.jid == jid)
        .map(|p| p.data.clone())
        .unwrap_or(Value::Null)
}

/// `data.fun?.pasangan`.
fn pasangan(data: &Value) -> Option<&str> {
    data.get("fun").and_then(|f| f.get("pasangan")).and_then(Value::as_str)
}

/// `data.fun?.tembakTarget`.
fn tembak_target(data: &Value) -> Option<&str> {
    data.get("fun").and_then(|f| f.get("tembakTarget")).and_then(Value::as_str)
}

/// Kartu cara pakai (`tembak.js:39-45`).
fn cara_pakai(prefix: &str) -> String {
    format!(
        "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
         > `{prefix}tembak @tag`\n\n\
         > Contoh:\n\
         > `{prefix}tembak @628xxx`\n\
         > Reply pesan + `{prefix}tembak`"
    )
}

#[plugin(
    name = "tembak",
    alias = ["nembak"],
    category = "fun",
    description = "Menembak seseorang untuk pacaran",
    usage = ".tembak @tag",
    example = ".tembak @628xxx",
    cooldown = 30,
    energi = 1,
    is_group = true,
    is_private = false
)]
pub fn tembak(ctx: Ctx) -> Reply {
    // Urutan target: quoted → mention → argumen angka (`tembak.js:27-36`).
    let target_jid = if let Some(q) = ctx.quoted() {
        Some(q.sender.clone())
    } else if let Some(m) = ctx.mentioned.first() {
        Some(m.clone())
    } else {
        target_dari_args(ctx.args.first())
    };

    let Some(target_jid) = target_jid.filter(|j| !j.is_empty()) else {
        return Reply::text(cara_pakai(&ctx.prefix));
    };

    if target_jid == ctx.sender {
        return Reply::text("Tidak bisa menembak diri sendiri!");
    }
    // Guard `targetJid === m.botNumber` dilewati — lihat catatan kepala.

    let sender_data = ctx.user_data.clone();
    let target_data = data_user(&ctx, &target_jid);

    // Pengirim sudah punya pasangan yang saling (`tembak.js:62-72`).
    if let Some(pasangan_sender) = pasangan(&sender_data) {
        let partner_data = data_user(&ctx, pasangan_sender);
        if pasangan(&partner_data) == Some(ctx.sender.as_str()) {
            let nama_partner = partner_data
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| "undefined".to_string());
            return Reply::Text {
                text: format!(
                    "❌ *sᴜᴅᴀʜ ᴘᴜɴʏᴀ ᴘᴀsᴀɴɢᴀɴ*\n\n\
                     Pasanganmu: @{}\n\
                     Putus dulu sama {nama_partner} dengan cara: `{}putus`",
                    jid_number(pasangan_sender),
                    ctx.prefix
                ),
                mentions: vec![pasangan_sender.to_string()],
            };
        }
    }

    // Target sudah pacaran dengan orang lain (`tembak.js:74-83`).
    if let Some(pasangan_target) = pasangan(&target_data) {
        if pasangan_target != ctx.sender {
            let partner_data = data_user(&ctx, pasangan_target);
            if pasangan(&partner_data) == Some(target_jid.as_str()) {
                return Reply::Text {
                    text: format!(
                        "💔 *ᴅɪᴀ sᴜᴅᴀʜ ᴘᴀᴄᴀʀᴀɴ*\n\nPasangannya: @{}",
                        jid_number(pasangan_target)
                    ),
                    mentions: vec![pasangan_target.to_string()],
                };
            }
        }
    }

    // Target pernah menembak pengirim (atau sudah pasangannya) → balasan
    // diterima langsung (`tembak.js:85-104`).
    if tembak_target(&target_data) == Some(ctx.sender.as_str())
        || pasangan(&target_data) == Some(ctx.sender.as_str())
    {
        // `db.setUser` dua sisi (`tembak.js:89-93`).
        let mutations = vec![
            Mutation::SetUser {
                jid: ctx.sender.clone(),
                data: json!({ "fun": { "pasangan": target_jid } }),
            },
            Mutation::SetUser {
                jid: target_jid.clone(),
                data: json!({ "fun": { "pasangan": ctx.sender } }),
            },
        ];
        // `delete global.tembakSessions[...]` — menghapus kunci sesi balasan.
        ctx.session_take(&session_key(&ctx.chat, &target_jid));
        return Reply::Text {
            text: format!(
                "💕 *CIE CIEE :3*\n\n\
                 @{} dan @{} resmi pacaran !\n\n\
                 Semoga langgeng yak! 💍",
                jid_number(&ctx.sender),
                jid_number(&target_jid)
            ),
            mentions: vec![ctx.sender.clone(), target_jid.clone()],
        }
        .mutating(mutations)
        .with_reaction("💕");
    }

    // Catat tembakan baru (`tembak.js:106-123`).
    let count = sender_data
        .get("fun")
        .and_then(|f| f.get("tembakCount"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        + 1;

    let mutations = vec![Mutation::SetUser {
        jid: ctx.sender.clone(),
        data: json!({ "fun": { "tembakTarget": target_jid, "tembakCount": count } }),
    }];

    ctx.session_set(
        &session_key(&ctx.chat, &target_jid),
        json!({
            "shooter": ctx.sender,
            "target": target_jid,
            "chat": ctx.chat,
            "timestamp": now_ms(),
        }),
    );

    Reply::Text {
        text: format!(
            "💘 *ADA YANG NEMBAK NIHH*\n\n\
             Hei @{} , kamu ditembak oleh @{} nichh\n\n\
             ⏱️ Berlaku *1 jam* dari sekarang\n\
             gunakan: `{}` / `{}`",
            jid_number(&target_jid),
            jid_number(&ctx.sender),
            format!("{}terima", ctx.prefix),
            format!("{}tolak", ctx.prefix)
        ),
        mentions: vec![target_jid, ctx.sender.clone()],
    }
    .mutating(mutations)
    .with_reaction("💘")
}

/// Padanan `answerHandler` (`tembak.js:142-218`): teks persis `terima`/`tolak`
/// (tanpa prefix) dari target tembakan yang me-reply pesan apa pun.
#[oktz_plugin_api::answer_handler]
pub fn tembak_answer(ctx: Ctx) -> Option<Reply> {
    let text = ctx.text.trim().to_lowercase();
    if text != "terima" && text != "tolak" {
        return None;
    }
    if ctx.quoted.is_none() {
        return None;
    }

    // Semua sesi dengan target = pengirim di chat ini (`tembak.js:151-153`),
    // yang belum kedaluwarsa satu jam (`:157-161`).
    let kunci = {
        let map = ctx
            .sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        map.iter()
            .filter(|(k, v)| {
                k.starts_with(&format!("{SESSION_PREFIX}:"))
                    && v.get("target").and_then(Value::as_str) == Some(ctx.sender.as_str())
                    && v.get("chat").and_then(Value::as_str) == Some(ctx.chat.as_str())
            })
            .find(|(_, v)| now_ms() - v.get("timestamp").and_then(Value::as_i64).unwrap_or(0) < EXPIRED_MS)
            .map(|(k, _)| k.clone())
    }?;
    let sesi = ctx.session_get(&kunci)?;
    let shooter = sesi.get("shooter").and_then(Value::as_str)?.to_string();

    let shooter_num = jid_number(&shooter).to_string();
    let sender_num = jid_number(&ctx.sender).to_string();

    if text == "terima" {
        ctx.session_take(&kunci);
        let mutations = vec![
            Mutation::SetUser {
                jid: shooter.clone(),
                data: json!({ "fun": { "pasangan": ctx.sender } }),
            },
            Mutation::SetUser {
                jid: ctx.sender.clone(),
                data: json!({ "fun": { "pasangan": shooter } }),
            },
        ];
        return Some(
            Reply::Text {
                text: format!(
                    "💕 *WIDIHHHH, CIE CIE DITERIMA* @{shooter_num}\n\n\
                     @{sender_num} dan @{shooter_num} resmi pacaran\n\n\
                     Semoga langgeng dan bahagia 💍"
                ),
                mentions: vec![ctx.sender.clone(), shooter],
            }
            .mutating(mutations)
            .with_reaction("💕"),
        );
    }

    // text == "tolak"
    ctx.session_take(&kunci);
    let mutations = vec![
        Mutation::SetUser {
            jid: shooter.clone(),
            data: json!({ "fun": { "pasangan": null, "tembakTarget": null } }),
        },
        Mutation::SetUser {
            jid: ctx.sender.clone(),
            data: json!({ "fun": { "pasangan": null } }),
        },
    ];
    Some(
        Reply::Text {
            text: format!(
                "💔 *WADUHH, YANG SABAR YAK* @{shooter_num}\n\n\
                 @{sender_num} menolak @{shooter_num} sebagai pacarnya\n\n\
                 Sabar ya, masih banyak yang lain! 😢"
            ),
            mentions: vec![ctx.sender.clone(), shooter],
        }
        .mutating(mutations)
        .with_reaction("💔"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::registry::{GroupRoster, Participant};

    const ME: &str = "628111@s.whatsapp.net";
    const THEM: &str = "628222@s.whatsapp.net";
    const OTHER: &str = "628333@s.whatsapp.net";

    fn anggota(jid: &str, data: Value) -> Participant {
        Participant {
            jid: jid.into(),
            is_admin: false,
            is_owner: false,
            data,
        }
    }

    fn roster(data_target: Value) -> GroupRoster {
        GroupRoster {
            participants: vec![
                anggota(ME, Value::Null),
                anggota(THEM, data_target),
                anggota(OTHER, Value::Null),
            ],
            is_bot_admin: false,
        }
    }

    fn run(user_data: Value, data_target: Value, quoted: Option<&str>, mention: Option<&str>, arg: Option<&str>) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tembak".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            text: String::new(),
            args: arg.map(str::to_string).into_iter().collect(),
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
            user_data,
            roster: roster(data_target),
            sessions: oktz_plugin_api::new_session_store(),
            ..Default::default()
        };
        tembak(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("tembak").unwrap();
        assert_eq!(p.meta.name, "tembak");
        assert_eq!(p.meta.alias, ["nembak"]);
        assert_eq!(p.meta.category, "fun");
        assert_eq!(p.meta.description, "Menembak seseorang untuk pacaran");
        assert_eq!(p.meta.usage, ".tembak @tag");
        assert_eq!(p.meta.example, ".tembak @628xxx");
        assert_eq!(p.meta.cooldown, 30);
        assert_eq!(p.meta.energi, 1);
        assert!(p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_admin);
        for a in ["nembak"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "tembak");
        }
    }

    #[test]
    fn tanpa_target_menampilkan_cara_pakai() {
        for (quoted, mention, arg) in [(None, None, None), (None, None, Some("12"))] {
            let r = run(Value::Null, Value::Null, quoted, mention, arg);
            assert_eq!(teks(&r), cara_pakai("."), "quoted={quoted:?} mention={mention:?} arg={arg:?}");
        }
    }

    #[test]
    fn argumen_angka_valid_menjadi_target() {
        let r = run(Value::Null, Value::Null, None, None, Some("+62 812-3456-789"));
        assert!(teks(&r).contains("Hei @628123456789 ,"), "{}", teks(&r));
    }

    #[test]
    fn menembak_diri_sendiri_ditolak() {
        let r = run(Value::Null, Value::Null, Some(ME), None, None);
        assert_eq!(teks(&r), "Tidak bisa menembak diri sendiri!");
    }

    #[test]
    fn pengirim_yang_saling_dipasangan_ditolak() {
        // Pasangan pengirim saling mencatat: partnerData.fun.pasangan === sender.
        let user = json!({ "fun": { "pasangan": OTHER } });
        let data_other = json!({ "fun": { "pasangan": ME }, "name": "Budi" });
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tembak".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            mentioned: vec![THEM.into()],
            user_data: user,
            roster: GroupRoster {
                participants: vec![
                    anggota(ME, Value::Null),
                    anggota(THEM, Value::Null),
                    anggota(OTHER, data_other),
                ],
                is_bot_admin: false,
            },
            sessions: oktz_plugin_api::new_session_store(),
            ..Default::default()
        };
        let r = tembak(ctx);
        assert!(teks(&r).starts_with("❌ *sᴜᴅᴀʜ ᴘᴜɴʏᴀ ᴘᴀsᴀɴɢᴀɴ*"), "{}", teks(&r));
        assert!(teks(&r).contains("Pasanganmu: @628333"), "{}", teks(&r));
        assert!(teks(&r).contains("Putus dulu sama Budi dengan cara: `.putus`"), "{}", teks(&r));
        match r.without_mutations() {
            Reply::Text { mentions, .. } => assert_eq!(mentions, &[OTHER.to_string()]),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn target_sudah_pacaran_dengan_orang_lain_ditolak() {
        let data_target = json!({ "fun": { "pasangan": OTHER } });
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tembak".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            mentioned: vec![THEM.into()],
            roster: GroupRoster {
                participants: vec![
                    anggota(ME, Value::Null),
                    anggota(THEM, data_target),
                    anggota(OTHER, json!({ "fun": { "pasangan": THEM } })),
                ],
                is_bot_admin: false,
            },
            sessions: oktz_plugin_api::new_session_store(),
            ..Default::default()
        };
        let r = tembak(ctx);
        assert!(teks(&r).starts_with("💔 *ᴅɪᴀ sᴜᴅᴀʜ ᴘᴀᴄᴀʀᴀɴ*"), "{}", teks(&r));
        assert!(teks(&r).contains("Pasangannya: @628333"), "{}", teks(&r));
    }

    #[test]
    fn target_yang_menembak_balik_langsung_jadi_pasangan() {
        let data_target = json!({ "fun": { "tembakTarget": ME } });
        let r = run(Value::Null, data_target, None, Some(THEM), None);
        let (emoji, isi) = match r.without_mutations() {
            Reply::Reaction { emoji, reply } => (emoji.clone(), reply.as_ref().clone()),
            other => panic!("harus Reaction, dapat {other:?}"),
        };
        assert_eq!(emoji, "💕");
        match isi {
            Reply::Text { text, mentions } => {
                assert!(text.contains("@628111 dan @628222 resmi pacaran !"), "{text}");
                assert_eq!(mentions, &[ME.to_string(), THEM.to_string()]);
            }
            other => panic!("harus Text, dapat {other:?}"),
        }
        let muts = r.mutations();
        assert_eq!(muts.len(), 2, "dua SetUser pasangan");
    }

    #[test]
    fn tembakan_baru_menyimpan_sesi_dan_mutation_target() {
        let sessions = oktz_plugin_api::new_session_store();
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tembak".into(),
            sender: ME.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            mentioned: vec![THEM.into()],
            user_data: json!({ "fun": { "tembakCount": 2 } }),
            roster: roster(Value::Null),
            sessions: sessions.clone(),
            ..Default::default()
        };
        let r = tembak(ctx);
        let (emoji, isi) = match r.without_mutations() {
            Reply::Reaction { emoji, reply } => (emoji.clone(), reply.as_ref().clone()),
            other => panic!("harus Reaction, dapat {other:?}"),
        };
        assert_eq!(emoji, "💘");
        match isi {
            Reply::Text { text, mentions } => {
                assert!(text.starts_with("💘 *ADA YANG NEMBAK NIHH*"), "{text}");
                assert!(text.contains("Hei @628222 , kamu ditembak oleh @628111 nichh"), "{text}");
                assert!(text.contains("gunakan: `.terima` / `.tolak`"), "{text}");
                assert_eq!(mentions, &[THEM.to_string(), ME.to_string()]);
            }
            other => panic!("harus Text, dapat {other:?}"),
        }
        // Mutasi: tembakTarget + tembakCount naik.
        match &r.mutations()[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, ME);
                assert_eq!(data["fun"]["tembakTarget"], THEM);
                assert_eq!(data["fun"]["tembakCount"], 3);
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
        // Sesi tersimpan.
        let sesi = sessions.lock().unwrap();
        let v = sesi.get(&session_key("120363@g.us", THEM)).unwrap();
        assert_eq!(v["shooter"], ME);
        assert_eq!(v["target"], THEM);
        assert_eq!(v["chat"], "120363@g.us");
    }

    #[test]
    fn answer_menolak_teks_lain_dan_tanpa_quote() {
        let sessions = oktz_plugin_api::new_session_store();
        // JS: body.trim().toLowerCase() harus persis "terima"/"tolak", plus
        // wajib me-reply sesuatu.
        for teks_masuk in ["", "terima ya", "tolak.", "Terimakasih"] {
            let c = Ctx {
                text: teks_masuk.into(),
                sessions: sessions.clone(),
                ..Default::default()
            };
            assert!(tembak_answer(c).is_none(), "teks {teks_masuk:?} harus ditolak");
        }
        // Teks valid tapi tanpa quoted → None.
        let c = Ctx {
            text: "terima".into(),
            sessions: sessions.clone(),
            ..Default::default()
        };
        assert!(tembak_answer(c).is_none());
    }

    #[test]
    fn answer_terima_menyatukan_pasangan() {
        let sessions = oktz_plugin_api::new_session_store();
        sessions.lock().unwrap().insert(
            session_key("120363@g.us", ME),
            json!({ "shooter": ME, "target": THEM, "chat": "120363@g.us", "timestamp": now_ms() }),
        );
        let c = Ctx {
            prefix: ".".into(),
            command: String::new(),
            sender: THEM.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            text: "terima".into(),
            quoted: Some(oktz_plugin_api::QuotedMessage {
                id: "X".into(),
                sender: ME.into(),
                chat: "120363@g.us".into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: true,
                timestamp: 0,
            }),
            sessions: sessions.clone(),
            ..Default::default()
        };
        let r = tembak_answer(c).expect("harus menangani");
        assert!(sessions.lock().unwrap().get(&session_key("120363@g.us", ME)).is_none());
        let (emoji, isi) = match r.without_mutations() {
            Reply::Reaction { emoji, reply } => (emoji.clone(), reply.as_ref().clone()),
            other => panic!("harus Reaction, dapat {other:?}"),
        };
        assert_eq!(emoji, "💕");
        match isi {
            Reply::Text { text, mentions } => {
                assert!(
                    text.starts_with("💕 *WIDIHHHH, CIE CIE DITERIMA* @628111"),
                    "{text}"
                );
                assert_eq!(mentions, &[THEM.to_string(), ME.to_string()]);
            }
            other => panic!("harus Text, dapat {other:?}"),
        }
        let muts = r.mutations();
        assert_eq!(muts.len(), 2);
        match &muts[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, ME);
                assert_eq!(data["fun"]["pasangan"], THEM);
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn answer_tolak_menghapus_tembakan() {
        let sessions = oktz_plugin_api::new_session_store();
        sessions.lock().unwrap().insert(
            session_key("120363@g.us", ME),
            json!({ "shooter": ME, "target": THEM, "chat": "120363@g.us", "timestamp": now_ms() }),
        );
        let c = Ctx {
            prefix: ".".into(),
            sender: THEM.into(),
            chat: "120363@g.us".into(),
            is_group: true,
            text: "tolak".into(),
            quoted: Some(oktz_plugin_api::QuotedMessage {
                id: "X".into(),
                sender: ME.into(),
                chat: "120363@g.us".into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: true,
                timestamp: 0,
            }),
            sessions: sessions.clone(),
            ..Default::default()
        };
        let r = tembak_answer(c).expect("harus menangani");
        let (emoji, isi) = match r.without_mutations() {
            Reply::Reaction { emoji, reply } => (emoji.clone(), reply.as_ref().clone()),
            other => panic!("harus Reaction, dapat {other:?}"),
        };
        assert_eq!(emoji, "💔");
        match isi {
            Reply::Text { text, mentions } => {
                assert!(
                    text.starts_with("💔 *WADUHH, YANG SABAR YAK* @628111"),
                    "{text}"
                );
                assert_eq!(mentions, &[THEM.to_string(), ME.to_string()]);
            }
            other => panic!("harus Text, dapat {other:?}"),
        }
        let muts = r.mutations();
        assert_eq!(muts.len(), 2);
        match &muts[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, ME);
                assert_eq!(data["fun"]["pasangan"], Value::Null);
                assert_eq!(data["fun"]["tembakTarget"], Value::Null);
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn answer_sesi_kedaluwarsa_dan_chat_lain_diabaikan() {
        let sessions = oktz_plugin_api::new_session_store();
        sessions.lock().unwrap().insert(
            session_key("120363@g.us", ME),
            json!({ "shooter": ME, "target": THEM, "chat": "120363@g.us", "timestamp": now_ms() - EXPIRED_MS }),
        );
        let c = Ctx {
            sender: THEM.into(),
            chat: "120363@g.us".into(),
            text: "terima".into(),
            quoted: Some(oktz_plugin_api::QuotedMessage {
                id: "X".into(),
                sender: ME.into(),
                chat: "120363@g.us".into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: true,
                timestamp: 0,
            }),
            sessions: sessions.clone(),
            ..Default::default()
        };
        assert!(tembak_answer(c).is_none(), "sesi kedaluwarsa harus diabaikan");
    }
}
