//! `plugins/group/demote.js` — turunkan admin menjadi member biasa.
//!
//! Gerbang (target, peserta, tolak yang bukan admin, tolak owner) + penurunan
//! `sock.groupParticipantsUpdate(m.chat, [id], 'demote')` (`demote.js:63,360`)
//! lewat `ctx.sock().ubah_peserta(.., Turunkan)`. Balasan memakai teks JS tanpa
//! emoji awalan, mention `target`.

use oktz_plugin_api::{AksiPeserta, Ctx, Reply, plugin};

use crate::group::kick::find_participant;

/// `m.quoted ? m.quoted.sender : m.mentionedJid[0]` (`demote.js:24-28`).
fn demote_target(ctx: &Ctx) -> Option<String> {
    if let Some(q) = ctx.quoted() {
        return Some(q.sender.clone());
    }
    ctx.mentioned.first().cloned()
}

/// `demote.js:31-35`.
fn no_target(prefix: &str) -> String {
    format!(
        "❌ *ᴛᴀʀɢᴇᴛ ᴛɪᴅᴀᴋ ᴅɪᴛᴇᴍᴜᴋᴀɴ*\n\n\
         > Reply pesan user atau mention!\n\
         > Contoh: `{prefix}demote @user`"
    )
}

const NOT_FOUND: &str = "❌ *ɢᴀɢᴀʟ*\n\n> User tidak ditemukan di grup!";
const NOT_ADMIN: &str = "❌ *ɢᴀɢᴀʟ*\n\n> User bukan admin!";
const IS_OWNER: &str = "❌ *ɢᴀɢᴀʟ*\n\n> Tidak bisa demote owner grup!";

#[plugin(
    name = "demote",
    alias = ["unadmin", "turunkan"],
    category = "group",
    description = "Turunkan admin menjadi member biasa",
    usage = ".demote @user",
    example = ".demote @user",
    cooldown = 5,
    energi = 0,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = true,
    is_private = false,
    is_admin = true,
    is_bot_admin = true
)]
pub fn demote(ctx: Ctx) -> Reply {
    let Some(target) = demote_target(&ctx) else {
        return Reply::text(no_target(&ctx.prefix));
    };

    // `findParticipantByNumber` di grup `addressing_mode=lid` library mengirim
    // { id: LID, phoneNumber: PN }; roster Rust hanya menyimpan satu JID per
    // peserta, jadi cocoknya hanya pada field itu.
    let Some(p) = find_participant(&ctx.roster.participants, &target) else {
        return Reply::text(NOT_FOUND);
    };

    if !p.is_admin {
        return Reply::text(NOT_ADMIN);
    }

    if p.is_owner {
        return Reply::text(IS_OWNER);
    }

    let number = target.split('@').next().unwrap_or(&target);
    match ctx.sock().ubah_peserta(
        &ctx.chat,
        std::slice::from_ref(&p.jid),
        AksiPeserta::Turunkan,
    ) {
        Ok(()) => Reply::text(format!("@{number} sekarang bukan admin lagi."))
            .with_mentions(vec![target]),
        Err(_) => Reply::text(crate::common::error_template(&ctx)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::Participant;
    use oktz_plugin_api::find;
    use oktz_plugin_api::registry::GroupRoster;
    use serde_json::Value;
    use std::sync::Arc;

    use crate::group::test_sock::CatatSock;

    fn p(jid: &str, admin: bool, owner: bool) -> Participant {
        Participant {
            jid: jid.into(),
            is_admin: admin,
            is_owner: owner,
            data: Value::Null,
        }
    }

    fn ctx_with(target: Option<&str>) -> Ctx {
        ctx_with_sock(target, Arc::new(CatatSock::default()))
    }

    fn ctx_with_sock(target: Option<&str>, sock: Arc<CatatSock>) -> Ctx {
        let roster = GroupRoster {
            participants: vec![
                p("628111@s.whatsapp.net", false, false),
                p("628999@s.whatsapp.net", true, false),
                p("628555@s.whatsapp.net", true, true),
            ],
            is_bot_admin: true,
        };
        Ctx {
            prefix: ".".into(),
            command: "demote".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            is_group: true,
            mentioned: target.map(|t| vec![t.to_string()]).unwrap_or_default(),
            roster,
            sock,
            ..Default::default()
        }
    }

    fn body(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn run(ctx: Ctx) -> Reply {
        (find("demote").unwrap().handler)(ctx)
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let e = find("demote").unwrap();
        assert_eq!(e.meta.name, "demote");
        assert_eq!(e.meta.alias, ["unadmin", "turunkan"]);
        assert_eq!(e.meta.category, "group");
        assert_eq!(e.meta.description, "Turunkan admin menjadi member biasa");
        assert_eq!(e.meta.usage, ".demote @user");
        assert_eq!(e.meta.example, ".demote @user");
        assert_eq!(e.meta.cooldown, 5);
        assert_eq!(e.meta.energi, 0);
        assert!(e.meta.is_enabled);
        assert!(e.meta.is_group && e.meta.is_admin && e.meta.is_bot_admin);
        assert!(!e.meta.is_private && !e.meta.is_owner && !e.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["unadmin", "turunkan"] {
            assert_eq!(
                find(a).map(|e| e.meta.name.as_str()),
                Some("demote"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn without_quote_or_mention_it_asks_for_a_target() {
        let r = run(ctx_with(None));
        assert_eq!(
            body(&r),
            "❌ *ᴛᴀʀɢᴇᴛ ᴛɪᴅᴀᴋ ᴅɪᴛᴇᴍᴜᴋᴀɴ*\n\n\
             > Reply pesan user atau mention!\n\
             > Contoh: `.demote @user`"
        );
    }

    #[test]
    fn a_missing_participant_is_reported_before_the_admin_checks() {
        let r = run(ctx_with(Some("628777@s.whatsapp.net")));
        assert_eq!(body(&r), NOT_FOUND);
    }

    #[test]
    fn a_plain_member_cannot_be_demoted() {
        let r = run(ctx_with(Some("628111@s.whatsapp.net")));
        assert_eq!(body(&r), NOT_ADMIN);
    }

    #[test]
    fn the_group_owner_is_never_demoted() {
        let r = run(ctx_with(Some("628555@s.whatsapp.net")));
        assert_eq!(body(&r), IS_OWNER);
    }

    #[test]
    fn a_real_admin_is_demoted_with_js_text() {
        let sock = Arc::new(CatatSock::default());
        let r = run(ctx_with_sock(Some("628999@s.whatsapp.net"), sock.clone()));
        assert_eq!(body(&r), "@628999 sekarang bukan admin lagi.");
        assert_eq!(
            *sock.seen.lock().unwrap(),
            vec![(
                "120363@g.us".to_string(),
                vec!["628999@s.whatsapp.net".to_string()],
                AksiPeserta::Turunkan
            )]
        );
    }

    #[test]
    fn a_failed_demote_reports_the_error_template() {
        let r = run(ctx_with_sock(
            Some("628999@s.whatsapp.net"),
            Arc::new(CatatSock {
                gagal: true,
                ..Default::default()
            }),
        ));
        assert_eq!(body(&r), crate::common::error_template_bawaan(".", "demote"));
    }

    #[test]
    fn the_target_follows_quote_first_then_mention() {
        let mut ctx = ctx_with(Some("628111@s.whatsapp.net"));
        ctx.quoted = Some(oktz_plugin_api::QuotedMessage {
            id: "M1".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: "120363@g.us".into(),
            text: "hai".into(),
            kind: oktz_plugin_api::QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        });
        let r = run(ctx);
        assert_eq!(body(&r), "@628999 sekarang bukan admin lagi.");
    }
}
