//! Set status AFK dengan alasan.
//!
//! `Keterbatasan:` dua beda yang dicatat, bukan disembunyikan:
//! - versi awal memanggil `checkAfk` untuk SEMUA pesan, Rust hanya untuk non-command
//! (jalur answer, `oktz-wa/src/pipeline.rs:4773`), jadi command tidak
//! menghapus AFK.
//! - `getAfkUser` versi awal jatuh ke `afkStorage` (memori) untuk JID `lid` yang tidak
//! bisa dipersist; Rust tidak punya store memori itu, jadi AFK JID `lid`
//! tidak terbaca. `notifyThrottle` dipetakan ke `Ctx::sessions`.
//! - Sebutan AFK dibaca dari `ctx.roster` (`p.data` = `db.getUser(p.jid)`),
//! jadi hanya berlaku di grup yang rosternya terisi.

use oktz_plugin_api::{Ctx, Mutation, Reply, answer_handler, plugin};
use serde_json::Value;

pub(crate) const NO_REASON: &str = "Tidak ada alasan";

/// `NOTIFY_TTL_MS` — throttle notifikasi sebutan AFK.
const NOTIFY_TTL_MS: i64 = 60 * 60 * 1000;

/// `canPersist(jid)` — JID yang aman ditulis ke store. Key `lid` (>15 digit)
/// dan yang berawalan `120` tidak bisa dipakai sebagai key store, jadi AFK-nya
/// hanya hidup selama ada di memori.
pub(crate) fn can_persist(jid: &str) -> bool {
    let clean = jid.split('@').next().unwrap_or(jid);
    clean.len() <= 15 && !clean.starts_with("120")
}

#[plugin(
    name = "afk",
    alias = ["away", "brb"],
    category = "group",
    description = "Set status AFK dengan alasan",
    usage = ".afk <alasan>",
    example = ".afk lagi makan",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false
)]
pub fn afk(ctx: Ctx) -> Reply {
    let reason = if ctx.text.is_empty() {
        NO_REASON.to_string()
    } else {
        ctx.text.clone()
    };
    let record = serde_json::json!({ "reason": reason, "since": crate::common::now_ms() });

    let reply = Reply::Text {
        text: format!(
            "💤 *ᴀꜰᴋ ᴀᴋᴛɪꜰ*\n\n\
             ```@{} sekarang AFK```\n\
             🍀 `Alasan:` *{}*\n\n\
             _Ketik apapun untuk menonaktifkan AFK._",
            crate::common::jid_number(&ctx.sender),
            reason
        ),
        mentions: vec![ctx.sender.clone()],
    };

    if can_persist(&ctx.sender) {
        reply.with_mutation(Mutation::SetUser {
            jid: ctx.sender.clone(),
            data: serde_json::json!({ "afk": record }),
        })
    } else {
        reply
    }
}

/// `formatAfkDuration` — sama
/// `oktz_router::mode::format_afk_duration`, disalin karena plugin tidak
/// bergantung pada `oktz-router`.
pub(crate) fn format_durasi(ms: i64) -> String {
    let detik = ms / 1000;
    let menit = detik / 60;
    let jam = menit / 60;
    let hari = jam / 24;
    if hari > 0 {
        return format!("{} hari {} jam", hari, jam % 24);
    }
    if jam > 0 {
        return format!("{} jam {} menit", jam, menit % 60);
    }
    if menit > 0 {
        return format!("{menit} menit");
    }
    format!("{detik} detik")
}

/// `afk` user yang aktif (objek = truthy, `null`/hilang = tidak aktif).
fn afk_aktif(data: &Value) -> Option<&Value> {
    data.get("afk").filter(|v| !v.is_null())
}

/// `db.getUser(jid)` untuk sebutan: data pengirim sudah ada di `Ctx`, peserta
/// lain lewat roster (`p.data` diisi `PluginCtxSource::fill`).
fn data_peserta<'a>(ctx: &'a Ctx, jid: &str) -> Option<&'a Value> {
    if jid == ctx.sender {
        return Some(&ctx.user_data);
    }
    ctx.roster
        .participants
        .iter()
        .find(|p| p.jid == jid)
        .map(|p| &p.data)
}

/// `checkAfk` — hook yang dipanggil SEBELUM plugin lain.
///
/// Di versi awal ini berjalan untuk SEMUA pesan; jalur answer Rust cuma dipanggil
/// untuk pesan non-command (`oktz-wa/src/pipeline.rs:4773`), jadi AFK tidak
/// ikut terhapus saat pengirim mengetik command. Pengirim tetap menang atas
/// rantai answer lain: begitu ada balasan, pipeline berhenti.
#[answer_handler]
pub fn afk_answer(ctx: &Ctx) -> Option<Reply> {
    let mut urutan: Vec<Reply> = Vec::new();
    let mut mutasi: Option<Mutation> = None;

    // `getAfkUser(m.sender)` lalu `removeAfkUser` + balasan "AFK BERAKHIR".
    if let Some(afk) = afk_aktif(&ctx.user_data).cloned() {
        let since = afk.get("since").and_then(Value::as_i64).unwrap_or(0);
        let durasi = format_durasi(crate::common::now_ms().saturating_sub(since));
        mutasi = Some(Mutation::SetUser {
            jid: ctx.sender.clone(),
            data: serde_json::json!({ "afk": null }),
        });
        urutan.push(Reply::Text {
            text: format!(
                "👋 *ᴀꜰᴋ ʙᴇʀᴀᴋʜɪʀ*\n\n\
                 ```@{} sudah kembali!```\n\
                 🍀 `Durasi AFK:` *{}*",
                crate::common::jid_number(&ctx.sender),
                durasi
            ),
            mentions: vec![ctx.sender.clone()],
        });
    }

    // Notifikasi user AFK yang disebut, grup saja, throttle 1 jam per pasangan.
    if ctx.is_group {
        for jid in &ctx.mentioned {
            if jid == &ctx.sender {
                continue;
            }
            let Some(afk) = data_peserta(ctx, jid).and_then(afk_aktif).cloned() else {
                continue;
            };
            let kunci = format!("afk_notify:{}|{}", ctx.sender, jid);
            let sudah = ctx
                .session_get(&kunci)
                .and_then(|v| v.as_i64())
                .is_some_and(|ts| crate::common::now_ms() - ts < NOTIFY_TTL_MS);
            if sudah {
                continue;
            }
            ctx.session_set(&kunci, serde_json::json!(crate::common::now_ms()));

            let since = afk.get("since").and_then(Value::as_i64).unwrap_or(0);
            let durasi = format_durasi(crate::common::now_ms().saturating_sub(since));
            let alasan = afk
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or(NO_REASON);
            urutan.push(Reply::Text {
                text: format!(
                    "💤 *ᴜsᴇʀ ᴀꜰᴋ*\n\n\
                     ```Hustt, jangan di ganggu!``` `@{}` lagi AFK\n\
                     🍀 `Alasan:` *{}*\n\
                     🍀 `Sejak:` *{} yang lalu*",
                    crate::common::jid_number(jid),
                    alasan,
                    durasi
                ),
                mentions: vec![jid.clone()],
            });
        }
    }

    if urutan.is_empty() {
        return None;
    }
    let balasan = Reply::sequence(urutan);
    Some(match mutasi {
        Some(m) => balasan.with_mutation(m),
        None => balasan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENDER: &str = "6281234567890@s.whatsapp.net";

    fn run(sender: &str, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "afk".into(),
            sender: sender.into(),
            chat: "628999@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        (oktz_plugin_api::find("afk").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn written_afk(r: &Reply) -> &Value {
        assert_eq!(r.mutations().len(), 1, "harus tepat satu mutasi");
        match &r.mutations()[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, SENDER);
                &data["afk"]
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("afk").unwrap();
        assert_eq!(p.meta.name, "afk");
        assert_eq!(p.meta.alias, ["away", "brb"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Set status AFK dengan alasan");
        assert_eq!(p.meta.usage, ".afk <alasan>");
        assert_eq!(p.meta.example, ".afk lagi makan");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group, "JS: isGroup: false");
        assert!(!p.meta.is_private, "JS: isPrivate: false");
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["away", "brb"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "afk");
        }
    }

    #[test]
    fn the_sender_is_mentioned_in_the_reply() {
        match run(SENDER, "lagi makan").without_mutations() {
            Reply::Text { mentions, text } => {
                assert_eq!(mentions, &vec![SENDER.to_string()]);
                assert!(text.contains("@6281234567890 sekarang AFK"), "{text}");
                assert!(text.contains("🍀 `Alasan:` *lagi makan*"), "{text}");
            }
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    #[test]
    fn an_empty_reason_falls_back_to_the_default_wording() {
        let r = run(SENDER, "");
        assert!(
            text_of(&r).contains("🍀 `Alasan:` *Tidak ada alasan*"),
            "{}",
            text_of(&r)
        );
        assert_eq!(written_afk(&r)["reason"], serde_json::json!("Tidak ada alasan"));
    }

    #[test]
    fn the_reason_keeps_its_inner_spacing_and_case() {
        let r = run(SENDER, "  Makan  Nasi  ");
        assert_eq!(written_afk(&r)["reason"], serde_json::json!("  Makan  Nasi  "));
    }

    #[test]
    fn a_persistable_jid_is_written_to_the_store_with_a_timestamp() {
        let before = crate::common::now_ms();
        let r = run(SENDER, "jalan");
        let afk = written_afk(&r);
        assert_eq!(afk["reason"], serde_json::json!("jalan"));
        let since = afk["since"].as_i64().expect("since harus angka");
        assert!(
            (crate::common::now_ms() - since) < 5_000 && since >= before,
            "since harus waktu sekarang: {since}"
        );
    }

    #[test]
    fn a_lid_jid_is_not_written_but_still_gets_a_reply() {
        // Key `lid` >15 digit tidak bisa jadi key store.
        let lid = "12345678901234567@lid";
        let r = run(lid, "sibuk");
        assert!(r.mutations().is_empty(), "tidak boleh menulis ke store");
        assert!(text_of(&r).contains("sibuk"), "{}", text_of(&r));
    }

    #[test]
    fn a_group_jid_prefix_is_not_persistable() {
        assert!(can_persist(SENDER));
        assert!(can_persist("628123@lid"));
        assert!(!can_persist("120363123456789@g.us"), "prefix 120");
        assert!(!can_persist("1234567890123456@s.whatsapp.net"), ">15 digit");
        // `"".replace(/@.+/g, '')` tetap `""` dan panjangnya 0, jadi versi awal
        // justru Menganggapnya boleh dipersist. Meng faithfully; sender
        // kosong tidak mungkin terjadi dari router.
        assert!(can_persist(""));
    }

    #[test]
    fn a_120_prefixed_lid_is_also_skipped() {
        let r = run("120363123456789012@lid", "x");
        assert!(r.mutations().is_empty());
        assert!(text_of(&r).contains("```@120363123456789012 sekarang AFK```"));
    }

    fn grup(user_data: Value, mentioned: &[&str], peserta: &[(&str, Value)]) -> Ctx {
        Ctx {
            sender: SENDER.into(),
            chat: "120363000000000000@g.us".into(),
            is_group: true,
            mentioned: mentioned.iter().map(|s| s.to_string()).collect(),
            user_data,
            roster: oktz_plugin_api::registry::GroupRoster {
                participants: peserta
                    .iter()
                    .map(|(jid, data)| oktz_plugin_api::registry::Participant {
                        jid: (*jid).to_string(),
                        data: data.clone(),
                        ..Default::default()
                    })
                    .collect(),
                is_bot_admin: false,
            },
            ..Ctx::default()
        }
    }

    fn balasan_pertama(r: &Reply) -> Reply {
        match r.without_mutations() {
            Reply::Sequence(list) => list.first().cloned().expect("ada balasan"),
            lain => lain.clone(),
        }
    }

    fn semua_teks(r: &Reply) -> String {
        match r.without_mutations() {
            Reply::Sequence(list) => list
                .iter()
                .filter_map(|x| x.text_body())
                .collect::<Vec<_>>()
                .join("\n"),
            lain => lain.text_body().unwrap_or("").to_string(),
        }
    }

    #[test]
    fn format_durasi_mengikuti_middleware_js() {
        assert_eq!(format_durasi(30_000), "30 detik");
        assert_eq!(format_durasi(5 * 60_000), "5 menit");
        assert_eq!(format_durasi(2 * 3_600_000 + 3 * 60_000), "2 jam 3 menit");
        assert_eq!(format_durasi(50 * 3_600_000), "2 hari 2 jam");
    }

    #[test]
    fn afk_aktif_dihapus_dan_dibalas_saat_kembali() {
        let since = crate::common::now_ms() - 65 * 60_000;
        let ctx = grup(
            serde_json::json!({ "afk": { "reason": "makan", "since": since } }),
            &[],
            &[],
        );
        let r = afk_answer(&ctx).expect("harus menangani");
        let all = semua_teks(&r);
        assert!(all.contains("ᴀꜰᴋ ʙᴇʀᴀᴋʜɪʀ"), "{all}");
        assert!(all.contains("1 jam 5 menit"), "{all}");
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, SENDER);
                assert_eq!(data["afk"], Value::Null);
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn tanpa_afk_tidak_menangani() {
        let ctx = grup(serde_json::json!({}), &[], &[]);
        assert!(afk_answer(&ctx).is_none());
    }

    #[test]
    fn sebutan_afk_di_grup_dibalas_dan_dithrottle() {
        let other = "628999888777@s.whatsapp.net";
        let ctx = grup(
            serde_json::json!({}),
            &[other],
            &[(
                other,
                serde_json::json!({ "afk": { "reason": "tidur", "since": crate::common::now_ms() - 120_000 } }),
            )],
        );
        let first = afk_answer(&ctx).expect("harus notify");
        let all = semua_teks(&first);
        assert!(all.contains("ᴜsᴇʀ ᴀꜰᴋ"), "{all}");
        assert!(all.contains("tidur"), "{all}");
        assert!(first.mutations().is_empty(), "notify tidak menulis store");
        // Panggilan kedua dalam window throttle: tidak ada balasan lagi.
        assert!(afk_answer(&ctx).is_none());
    }

    #[test]
    fn sebutan_afk_di_luar_grup_diabaikan() {
        let other = "628999888777@s.whatsapp.net";
        let mut ctx = grup(
            serde_json::json!({}),
            &[other],
            &[(
                other,
                serde_json::json!({ "afk": { "reason": "tidur", "since": 0 } }),
            )],
        );
        ctx.is_group = false;
        assert!(afk_answer(&ctx).is_none());
    }

    #[test]
    fn balasan_pertama_membawa_mention_pengirim() {
        let ctx = grup(
            serde_json::json!({ "afk": { "reason": "x", "since": crate::common::now_ms() } }),
            &[],
            &[],
        );
        let r = afk_answer(&ctx).expect("harus menangani");
        match balasan_pertama(&r).without_mutations() {
            Reply::Text { mentions, .. } => assert_eq!(mentions.as_slice(), [SENDER]),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }
}