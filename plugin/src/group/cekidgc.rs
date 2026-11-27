//! Cek ID dan info lengkap grup.

use oktz_plugin_api::registry::{CardContext, InteractiveButton};
use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::{json, Value};

/// `config.saluran?.id || "120363400911374213@newsletter"`.
const SALURAN_ID_DEFAULT: &str = "120363400911374213@newsletter";

/// `config.bot?.name || "Bot"`.
const BOT_NAMA_DEFAULT: &str = "Bot";

/// Link invite kosong.
const LINK_TIDAK_VALID: &str = "── .✦ ──\n\n> Link grup tidak valid .☘︎ ݁˖";

/// `groupGetInviteInfo` gagal.
const LINK_EXPIRED: &str = "── .✦ ──\n\n> Link grup tidak valid atau sudah expired .☘︎ ݁˖";

/// `groupMetadata` gagal untuk JID grup.
const TIDAK_BISA_AKSES: &str = "── .✦ ──\n\n> Tidak bisa mengakses grup tersebut .☘︎ ݁˖";

/// Metadata/JID kosong setelah semua percobaan.
const TIDAK_DITEMUKAN: &str = "── .✦ ──\n\n> Tidak dapat menemukan info grup .☘︎ ݁˖";

/// Cara pakai, dipakai kalau bukan di grup dan tanpa
/// input.
fn cara_pakai(prefix: &str) -> String {
    format!(
        "── .✦ 𝗖𝗘𝗞 𝗜𝗗 𝗚𝗥𝗨𝗣 ✦. ── 𝜗ৎ\n\n\
         > Gunakan di grup atau masukkan link grup\n\n\
         > `{prefix}cekidgc` — di dalam grup\n\
         > `{prefix}cekidgc https://chat.whatsapp.com/xxx`"
    )
}

/// `formatDate` — asumsi zona Jakarta (UTC+7).
fn format_date(ms: i64) -> String {
    if ms == 0 {
        return "—".to_string();
    }
    // Timestamp < 1e12 dibaca sebagai detik.
    let ms = if ms < 1_000_000_000_000 { ms * 1000 } else { ms };
    let (y, m, d, h, mi, _s) = crate::common::jakarta_parts(ms);
    format!("{d:02}/{m:02}/{y} {h:02}:{mi:02}")
}

/// `groupDesc.slice(0, 120) + "..."`.
///
/// Tiga titik ASCII, bukan karakter `…`.
fn potong_deskripsi(desc: &str) -> String {
    if desc.chars().count() > 120 {
        let awal: String = desc.chars().take(120).collect();
        format!("{awal}...")
    } else {
        desc.to_string()
    }
}

/// `participants.length || groupMeta.size || 0`.
///
/// `||` versi awal: panjang 0 jatuh ke `size`, bukan mengambil yang lebih besar.
fn jumlah_member(participants: &[Value], meta: &Value) -> usize {
    if !participants.is_empty() {
        return participants.len();
    }
    meta.get("size")
        .and_then(Value::as_u64)
        .map(|s| s as usize)
        .unwrap_or(0)
}

/// `admins.length`.
fn jumlah_admin(participants: &[Value]) -> usize {
    participants
        .iter()
        .filter(|p| {
            p.get("admin")
                .and_then(Value::as_str)
                .is_some_and(|a| a == "admin" || a == "superadmin")
        })
        .count()
}

/// `groupOwner.replace(/@.+/g, "")`.
fn owner_tanpa_server(owner: &str) -> String {
    owner.split('@').next().unwrap_or(owner).to_string()
}

#[plugin(
    name = "cekidgc",
    alias = ["idgc", "idgrup", "groupid"],
    category = "group",
    description = "Cek ID dan info lengkap grup",
    usage = ".cekidgc [link grup]",
    example = ".cekidgc https://chat.whatsapp.com/xxxxx",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekidgc(ctx: Ctx) -> Reply {
    let input = ctx.text.trim().to_string();

    // Tentukan JID + metadata grup.
    let (group_jid, group_meta) = match cari_grup(&ctx, &input) {
        CariGrup::Found(jid, meta) => (jid, meta),
        CariGrup::Gagal(teks) => return Reply::text(teks).with_reaction("✘"),
        CariGrup::BukanGrup => {
            // Bukan grup dan tanpa input.
            return Reply::text(cara_pakai(&ctx.prefix));
        }
    };

    //
    let group_name = group_meta
        .get("subject")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown")
        .to_string();
    let participants: Vec<Value> = group_meta
        .get("participants")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let member_count = jumlah_member(&participants, &group_meta);
    let admin_count = jumlah_admin(&participants);
    let group_owner = group_meta
        .get("owner")
        .or_else(|| group_meta.get("subjectOwner"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("—")
        .to_string();
    let created_at = format_date(
        group_meta
            .get("creation")
            .and_then(Value::as_i64)
            .unwrap_or(0),
    );
    let group_desc = group_meta
        .get("desc")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("—")
        .to_string();
    let desc_preview = potong_deskripsi(&group_desc);
    let is_restrict = if group_meta
        .get("restrict")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "Admin Only"
    } else {
        "Semua Member"
    };
    let is_announce = if group_meta
        .get("announce")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "Aktif"
    } else {
        "Nonaktif"
    };
    let is_community = if group_meta
        .get("isCommunity")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "✓ Ya"
    } else {
        "✘ Tidak"
    };
    let join_mode = if group_meta
        .get("joinApprovalMode")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "Perlu Approval"
    } else {
        "Bebas"
    };

    //
    let saluran_id = {
        let id = ctx.config().saluran.id.clone();
        if id.is_empty() {
            SALURAN_ID_DEFAULT.to_string()
        } else {
            id
        }
    };
    let bot_nama_efektif = {
        let n = ctx.config().bot.name.clone();
        if n.is_empty() {
            BOT_NAMA_DEFAULT.to_string()
        } else {
            n
        }
    };
    let saluran_nama = {
        let n = ctx.config().saluran.name.clone();
        if n.is_empty() {
            bot_nama_efektif.clone()
        } else {
            n
        }
    };

    //
    let info_text = format!(
        "── .✦ 𝗚𝗥𝗢𝗨𝗣 𝗜𝗡𝗙𝗢 ✦. ── 𝜗ৎ\n\n\
         ╭─〔 {group_name} 〕───⬣\n\
         │  ✦ ɴᴀᴍᴀ        : *{group_name}*\n\
         │  ✦ ɪᴅ             : `{group_jid}`\n\
         │  ✦ ᴍᴇᴍʙᴇʀ     : *{member_count}*\n\
         │  ✦ ᴀᴅᴍɪɴ        : *{admin_count}*\n\
         │  ✦ ᴏᴡɴᴇʀ       : @{owner}\n\
         │  ✦ ᴅɪʙᴜᴀᴛ       : *{created_at}*\n\
         │  ✦ ᴋᴏᴍᴜɴɪᴛᴀs : *{is_community}*\n\
         │  ✦ ᴇᴅɪᴛ ɪɴꜰᴏ   : *{is_restrict}*\n\
         │  ✦ ᴀɴɴᴏᴜɴᴄᴇ : *{is_announce}*\n\
         │  ✦ ᴊᴏɪɴ ᴍᴏᴅᴇ  : *{join_mode}*\n\
         │  ✦ ᴅᴇsᴋʀɪᴘsɪ  : {desc_preview}\n\
         ╰──────────────⬣\n\n\
         .☘︎ ݁˖ © {bot_nama_efektif}",
        owner = owner_tanpa_server(&group_owner),
    );

    // Tombol salin ID grup.
    let tombol = InteractiveButton::new(
        "cta_copy",
        json!({
            "display_text": "✦ Copy ID Grup",
            "copy_code": group_jid,
        }),
    );

    // `contextInfo` kartu.
    let context = CardContext::new()
        .mention(ctx.sender.clone())
        .mention(group_owner.clone())
        .forwarded(9999)
        .newsletter(saluran_id, saluran_nama);

    Reply::interactive(info_text)
        .with_footer(format!("© {bot_nama_efektif}"))
        .with_button_card(tombol)
        .with_context(context)
        .with_reaction("✓")
}

/// Hasil pencarian grup — dipisah supaya tiap cabang `return` di versi awal punya
/// padanan yang jelas.
enum CariGrup {
    /// JID + metadata grup.
    Found(String, Value),
    /// Percobaan gagal — teks balasan siap pakai.
    Gagal(&'static str),
    /// Bukan di grup dan input tidak menunjuk grup mana pun.
    BukanGrup,
}

/// Tentukan JID + metadata dari input / chat saat ini.
fn cari_grup(ctx: &Ctx, input: &str) -> CariGrup {
    if input.contains("chat.whatsapp.com/") {
        // Kode invite = bagian setelah `chat.whatsapp.com/`
        // sampai spasi/`?`.
        let kode = input
            .split("chat.whatsapp.com/")
            .nth(1)
            .and_then(|s| s.split([' ', '?']).next())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(kode) = kode else {
            return CariGrup::Gagal(LINK_TIDAK_VALID);
        };
        return match ctx.sock().info_undangan(kode) {
            Ok(meta) => match meta.get("id").and_then(Value::as_str) {
                Some(jid) => CariGrup::Found(jid.to_string(), meta),
                // Metadata tanpa `id` = tidak ditemukan.
                None => CariGrup::Gagal(TIDAK_DITEMUKAN),
            },
            Err(_) => CariGrup::Gagal(LINK_EXPIRED),
        };
    }
    if input.ends_with("@g.us") {
        return match ctx.sock().metadata_grup(input) {
            Ok(meta) => CariGrup::Found(input.to_string(), meta),
            Err(_) => CariGrup::Gagal(TIDAK_BISA_AKSES),
        };
    }
    if ctx.is_group {
        return match ctx.sock().metadata_grup(&ctx.chat) {
            Ok(meta) => CariGrup::Found(ctx.chat.clone(), meta),
            Err(_) => CariGrup::Gagal(TIDAK_BISA_AKSES),
        };
    }
    CariGrup::BukanGrup
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::sock::{AdaWhatsapp, AksiPeserta, Sock, SockError};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct StubSock {
        meta: Mutex<Option<Value>>,
        meta_gagal: Mutex<bool>,
        undangan: Mutex<Option<Value>>,
        undangan_gagal: Mutex<bool>,
    }

    impl Sock for StubSock {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("x"))
        }
        fn foto_profil(&self, _jid: &str, _tipe: &str) -> Result<Option<String>, SockError> {
            Ok(None)
        }
        fn metadata_grup(&self, jid: &str) -> Result<Value, SockError> {
            if *self.meta_gagal.lock().unwrap() {
                return Err(SockError::baru("x"));
            }
            Ok(self.meta.lock().unwrap().clone().unwrap_or_else(|| {
                serde_json::json!({
                    "subject": format!("Grup {jid}"),
                    "id": jid,
                    "owner": "628111@s.whatsapp.net",
                    "creation": 1735689600000i64,
                    "participants": [
                        {"jid": "628111@s.whatsapp.net", "admin": "superadmin"},
                        {"jid": "628222@s.whatsapp.net", "admin": "admin"},
                        {"jid": "628333@s.whatsapp.net", "admin": null}
                    ],
                    "desc": "ini deskripsi grup yang cukup panjang untuk uji potongan di atas 120 karakter ............"
                })
            }))
        }
        fn semua_grup(&self) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn ubah_peserta(&self, _: &str, _: &[String], _: AksiPeserta) -> Result<(), SockError> {
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
        fn info_undangan(&self, _kode: &str) -> Result<Value, SockError> {
            if *self.undangan_gagal.lock().unwrap() {
                return Err(SockError::baru("expired"));
            }
            Ok(self.undangan.lock().unwrap().clone().unwrap_or_else(|| {
                serde_json::json!({"id": "629999@g.us", "subject": "Grup Undangan"})
            }))
        }
        fn buat_grup(&self, _: &str, _: &[String]) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn tandai_dibaca(&self, _: &[Value]) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("x"))
        }
        fn status_teks(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("x"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<Value>, SockError> {
            Err(SockError::baru("x"))
        }
    
        fn set_bio(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn set_nama(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn hapus_foto_bot(&self) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn blokir(&self, _: &str, _: bool) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    fn grup_meta() -> Value {
        serde_json::json!({
            "subject": "Coba Grup",
            "id": "621234567890@g.us",
            "owner": "628111@s.whatsapp.net",
            "creation": 1735689600000i64,
            "participants": [
                {"jid": "628111@s.whatsapp.net", "admin": "superadmin"},
                {"jid": "628222@s.whatsapp.net", "admin": "admin"},
                {"jid": "628333@s.whatsapp.net", "admin": null}
            ],
            "desc": "ini deskripsi grup yang cukup panjang untuk uji potongan di atas 120 karakter ............"
        })
    }

    fn run(chat: &str, is_group: bool, text: &str) -> Reply {
        let sock = Arc::new(StubSock::default());
        sock.meta.lock().unwrap().replace(grup_meta());
        let ctx = Ctx {
            prefix: ".".into(),
            chat: chat.to_string(),
            is_group,
            text: text.to_string(),
            command: "cekidgc".into(),
            sender: "628999@s.whatsapp.net".into(),
            sock,
            config: Arc::new(oktz_plugin_api::config::Runtime::default()),
            ..Ctx::default()
        };
        cekidgc(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("cekidgc").unwrap();
        assert_eq!(p.meta.name, "cekidgc");
        assert_eq!(p.meta.alias, ["idgc", "idgrup", "groupid"]);
        assert_eq!(p.meta.category, "group");
        assert_eq!(p.meta.description, "Cek ID dan info lengkap grup");
        assert_eq!(p.meta.usage, ".cekidgc [link grup]");
        assert_eq!(p.meta.example, ".cekidgc https://chat.whatsapp.com/xxxxx");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_back_to_this_plugin() {
        for a in ["idgc", "idgrup", "groupid"] {
            assert_eq!(
                oktz_plugin_api::find(a).map(|p| p.meta.name.as_str()),
                Some("cekidgc"),
                "alias {a}"
            );
        }
    }

    #[test]
    fn format_tanggal_jakarta() {
        assert_eq!(format_date(0), "—");
        assert_eq!(format_date(1735689600000), "01/01/2025 07:00");
    }

    #[test]
    fn output_sama_dengan_js_info_grup() {
        let r = run("621234567890@g.us", true, "");
        let t = teks(&r);
        assert!(t.contains("── .✦ 𝗚𝗥𝗢𝗨𝗣 𝗜𝗡𝗙𝗢 ✦. ── 𝜗ৎ"), "{t}");
        assert!(t.contains("╭─〔 Coba Grup 〕───⬣"), "{t}");
        assert!(t.contains("│  ✦ ɴᴀᴍᴀ        : *Coba Grup*"), "{t}");
        assert!(t.contains("│  ✦ ɪᴅ             : `621234567890@g.us`"), "{t}");
        assert!(t.contains("│  ✦ ᴍᴇᴍʙᴇʀ     : *3*"), "{t}");
        assert!(t.contains("│  ✦ ᴀᴅᴍɪɴ        : *2*"), "{t}");
        assert!(t.contains("│  ✦ ᴏᴡɴᴇʀ       : @628111"), "{t}");
        assert!(t.contains("│  ✦ ᴅɪʙᴜᴀᴛ       : *01/01/2025 07:00*"), "{t}");
        assert!(t.contains("│  ✦ ᴋᴏᴍᴜɴɪᴛᴀs : *✘ Tidak*"), "{t}");
        assert!(t.contains("│  ✦ ᴇᴅɪᴛ ɪɴꜰᴏ   : *Semua Member*"), "{t}");
        assert!(t.contains("│  ✦ ᴀɴɴᴏᴜɴᴄᴇ : *Nonaktif*"), "{t}");
        assert!(t.contains("│  ✦ ᴊᴏɪɴ ᴍᴏᴅᴇ  : *Bebas*"), "{t}");
        assert!(t.contains(".☘︎ ݁˖ © Bot"), "{t}");
        assert!(t.contains("╰──────────────⬣"), "{t}");
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "✓"),
            other => panic!("harus Reaction, dapat {other:?}"),
        }
    }

    #[test]
    fn output_sama_dengan_js_kartu_interaktif() {
        let r = run("621234567890@g.us", true, "");
        let Reply::Interactive { kartu } = r.without_mutations().tanpa_reaksi()
        else {
            panic!("harus Interactive");
        };
        let body = &kartu.body;
        let footer = &kartu.footer;
        let buttons = &kartu.buttons;
        let context = kartu.context_card.clone().expect("contextInfo wajib");
        assert!(body.contains("╭─〔 Coba Grup 〕───⬣"), "{body}");
        assert_eq!(footer.as_deref(), Some("© Bot"));
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].name, "cta_copy");
        assert_eq!(buttons[0].params["display_text"], "✦ Copy ID Grup");
        assert_eq!(buttons[0].params["copy_code"], "621234567890@g.us");
        let ctx = *context;
        assert_eq!(ctx.mentioned, vec!["628999@s.whatsapp.net", "628111@s.whatsapp.net"]);
        assert!(ctx.is_forwarded);
        assert_eq!(ctx.forwarding_score, 9999);
        let nl = ctx.forwarded_newsletter.expect("newsletter wajib");
        assert_eq!(nl.jid, SALURAN_ID_DEFAULT);
        assert_eq!(nl.server_message_id, 127);
    }

    #[test]
    fn deskripsi_dipotong_120_karakter_dengan_tiga_titik() {
        let r = run("621234567890@g.us", true, "");
        let t = teks(&r);
        let idx = t.find("│  ✦ ᴅᴇsᴋʀɪᴘsɪ  :").unwrap();
        let baris: String = t[idx..].chars().take(200).collect();
        assert!(baris.contains("..."), "harus ada tiga titik: {baris}");
        assert!(!baris.contains('…'), "bukan karakter ellipsis: {baris}");
    }

    #[test]
    fn output_sama_dengan_js_bantuan_di_bukan_grup() {
        let r = run("628111@s.whatsapp.net", false, "");
        let t = teks(&r);
        assert_eq!(
            t,
            "── .✦ 𝗖𝗘𝗞 𝗜𝗗 𝗚𝗥𝗨𝗣 ✦. ── 𝜗ৎ\n\n\
             > Gunakan di grup atau masukkan link grup\n\n\
             > `.cekidgc` — di dalam grup\n\
             > `.cekidgc https://chat.whatsapp.com/xxx`"
        );
    }

    #[test]
    fn output_sama_dengan_js_link_tidak_valid() {
        let r = run("628111@s.whatsapp.net", false, "https://chat.whatsapp.com/");
        assert_eq!(teks(&r), LINK_TIDAK_VALID);
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => assert_eq!(emoji, "✘"),
            other => panic!("harus Reaction, dapat {other:?}"),
        }
    }

    #[test]
    fn output_sama_dengan_js_link_expired() {
        let sock = Arc::new(StubSock::default());
        *sock.undangan_gagal.lock().unwrap() = true;
        let ctx = Ctx {
            prefix: ".".into(),
            chat: "628111@s.whatsapp.net".into(),
            is_group: false,
            text: "https://chat.whatsapp.com/ABCDE12345".into(),
            command: "cekidgc".into(),
            sock,
            ..Ctx::default()
        };
        let r = cekidgc(ctx);
        assert_eq!(teks(&r), LINK_EXPIRED);
    }

    #[test]
    fn output_sama_dengan_js_grup_tidak_bisa_diakses() {
        let sock = Arc::new(StubSock::default());
        *sock.meta_gagal.lock().unwrap() = true;
        let ctx = Ctx {
            prefix: ".".into(),
            chat: "628111@s.whatsapp.net".into(),
            is_group: false,
            text: "621234567890@g.us".into(),
            command: "cekidgc".into(),
            sock,
            ..Ctx::default()
        };
        let r = cekidgc(ctx);
        assert_eq!(teks(&r), TIDAK_BISA_AKSES);
    }

    #[test]
    fn link_grup_menggunakan_info_undangan() {
        let sock = Arc::new(StubSock::default());
        sock.undangan
            .lock()
            .unwrap()
            .replace(serde_json::json!({"id": "629999@g.us", "subject": "Grup Undangan"}));
        let ctx = Ctx {
            prefix: ".".into(),
            chat: "628111@s.whatsapp.net".into(),
            is_group: false,
            text: "https://chat.whatsapp.com/ABCDE12345".into(),
            command: "cekidgc".into(),
            sock,
            ..Ctx::default()
        };
        let r = cekidgc(ctx);
        let t = teks(&r);
        assert!(t.contains("Grup Undangan"), "{t}");
        assert!(t.contains("629999@g.us"), "{t}");
    }

    #[test]
    fn link_grup_tanpa_id_menghasilkan_tidak_ditemukan() {
        let sock = Arc::new(StubSock::default());
        sock.undangan
            .lock()
            .unwrap()
            .replace(serde_json::json!({"subject": "Tanpa ID"}));
        let ctx = Ctx {
            prefix: ".".into(),
            chat: "628111@s.whatsapp.net".into(),
            is_group: false,
            text: "https://chat.whatsapp.com/ABCDE".into(),
            command: "cekidgc".into(),
            sock,
            ..Ctx::default()
        };
        let r = cekidgc(ctx);
        assert_eq!(teks(&r), TIDAK_DITEMUKAN);
    }

    #[test]
    fn jumlah_member_mengikuti_operator_or_js() {
        // `participants.length || size || 0` — panjang 0 jatuh ke `size`.
        let meta = serde_json::json!({ "size": 12 });
        assert_eq!(jumlah_member(&[], &meta), 12);
        let meta = serde_json::json!({ "size": 12 });
        let parts = vec![Value::Null, Value::Null, Value::Null];
        assert_eq!(jumlah_member(&parts, &meta), 3, "panjang !== 0 menang");
        let meta = serde_json::json!({});
        assert_eq!(jumlah_member(&[], &meta), 0);
    }
}
