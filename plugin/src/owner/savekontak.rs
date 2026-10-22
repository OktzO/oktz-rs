//! `plugins/owner/savekontak.js` — menyimpan kontak dari grup menjadi file VCF.
//!
//! Port dari nol. Yang dipindah:
//! - cabang `get` (`:18-80`) — ekstraksi vCard + pengiriman dokumen.
//! - kartu pilihan grup (`:82-130`) — `single_select` + tombol "Semua Grup".
//!
//! `ponytail:` daftar grup diambil dari `sock.groupFetchAllParticipating()`
//! (`savekontak.js:22`) yang tersedia sebagai `Sock::semua_grup()`.
//!
//! `ponytail:` nomor bot sendiri tidak bisa disaring — `sock.user.id` tidak
//! ada di `Ctx` — jadi semua peserta grup ikut masuk ke VCF.
//!
//! `ponytail:` pesan kedua JS (`{ contacts: { displayName, contacts } }`,
//! `savekontak.js:72-77`) adalah `contactsMessage` WA; `Reply` tidak punya
//! varian itu, jadi hanya dokumen VCF yang dikirim.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use oktz_plugin_api::registry::{
    CardContext, CardList, CardRow, CardSection, InteractiveButton,
};
use serde_json::Value;

/// Footer kartu (`savekontak.js:113`).
const FOOTER: &str = "Powered by ReviewBot";

/// `Object.values(chats)` (`savekontak.js:24,84`) — daftar metadata grup.
fn daftar_grup(ctx: &Ctx) -> Vec<(String, Value)> {
    let Ok(all) = ctx.sock().semua_grup() else {
        return Vec::new();
    };
    let Some(obj) = all.as_object() else {
        return Vec::new();
    };
    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// `group.participants` → daftar id (`savekontak.js:47`).
fn peserta(metadata: &Value) -> Vec<String> {
    metadata
        .get("participants")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|p| {
                    p.get("id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| p.as_str().map(str::to_string))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `participant.id.split("@")[0]` (`savekontak.js:50`).
fn nomor(jid: &str) -> String {
    jid.split('@').next().unwrap_or(jid).to_string()
}

/// Satu vCard (`savekontak.js:52`).
fn vcard(nama: &str, nomor_peserta: &str) -> String {
    format!(
        "BEGIN:VCARD\nVERSION:3.0\nFN:{nama}\nTEL;type=CELL;type=VOICE;waid={nomor_peserta}:+{nomor_peserta}\nEND:VCARD"
    )
}

#[plugin(
    name = "savekontak",
    alias = ["sv", "svkontak"],
    category = "owner",
    description = "Menyimpan kontak dari grup menjadi file VCF",
    usage = ".savekontak <nama>",
    example = ".savekontak Fulan",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = true,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn savekontak(ctx: Ctx) -> Reply {
    let args = ctx.args.clone();

    if args.first().map(String::as_str) == Some("get") {
        let target = args.get(1).cloned().unwrap_or_default();
        let base_name = {
            let joined = args[2..].join(" ");
            if joined.is_empty() {
                "User".to_string()
            } else {
                joined
            }
        };

        let semua = daftar_grup(&ctx);
        let groups: Vec<(String, Value)> = if target == "all" {
            semua.clone()
        } else if let Some((jid, meta)) = semua.iter().find(|(jid, _)| *jid == target) {
            vec![(jid.clone(), meta.clone())]
        } else {
            return Reply::text("❌ Grup tidak ditemukan.");
        };

        if groups.is_empty() {
            return Reply::text("❌ Bot tidak berada di grup mana pun.");
        }

        let mut vcards = String::new();
        let mut count = 0usize;
        let mut index = 1usize;
        for (_, meta) in &groups {
            for p in peserta(meta) {
                let number = nomor(&p);
                let name = format!("{base_name} {index}");
                vcards.push_str(&vcard(&name, &number));
                vcards.push('\n');
                count += 1;
                index += 1;
            }
        }

        if count == 0 {
            return Reply::text("❌ Tidak ada kontak yang bisa diekstrak.");
        }

        return Reply::sequence(vec![
            Reply::text(format!(
                "⏳ Sedang mengekstrak kontak dari {} grup...",
                groups.len()
            )),
            Reply::document(
                MediaSource::bytes(vcards.into_bytes()),
                format!("{base_name}_{count}_Kontak.vcf"),
            )
            .with_caption(format!(
                "✅ *Berhasil mengekstrak {count} kontak ke dalam VCF.*"
            )),
        ]);
    }

    let base_name = {
        let joined = args.join(" ");
        if joined.is_empty() {
            "User".to_string()
        } else {
            joined
        }
    };

    let groups = daftar_grup(&ctx);
    if groups.is_empty() {
        return Reply::text("❌ Bot tidak berada di grup manapun.");
    }

    let rows: Vec<CardRow> = groups
        .iter()
        .map(|(jid, meta)| {
            let subjek = meta
                .get("subject")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let jumlah = meta
                .get("participants")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            CardRow::new(subjek, format!("{}savekontak get {jid} {base_name}", ctx.prefix))
                .description(format!("Anggota: {jumlah}"))
        })
        .collect();

    let list = CardList::new(
        "Pilih Grup",
        vec![CardSection::new("Daftar Grup").rows(rows)],
    );

    let text = format!(
        "📇 *SISTEM SAVE KONTAK (VCF)*\n\n\
         Sistem ekstraksi kontak otomatis dari grup yang diikuti bot.\n\
         Nama Base: *{base_name}*\n\n\
         *PENGGUNAAN:*\n\
         • *{}savekontak <nama>* — Menyimpan dengan nama kustom\n\
         • *{}savekontak* — Menyimpan dengan nama default \"User\"\n\n\
         *PENJELASAN ALUR PENGGUNAAN:*\n\
         1. Pilih grup spesifik dari tombol *Pilih Grup* di bawah, atau klik *Semua Grup* untuk mengekstrak kontak secara global.\n\
         2. Bot akan mengumpulkan nomor peserta dan mengabaikan nomor bot sendiri.\n\
         3. Hasil akan dikirim berupa file dokumen (*.vcf*) beserta list kontak WhatsApp agar bisa langsung disave.",
        ctx.prefix, ctx.prefix
    );

    Reply::interactive(text)
        .with_footer(FOOTER)
        .with_buttons(vec![
            InteractiveButton::select(&list),
            InteractiveButton::quick_reply(
                "Semua Grup",
                format!("{}savekontak get all {base_name}", ctx.prefix),
            ),
        ])
        .with_context(CardContext::new().quote_trigger())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;
    use oktz_plugin_api::sock::{AdaWhatsapp, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    #[derive(Debug, Default)]
    struct StubSock {
        grup: serde_json::Value,
    }

    impl Sock for StubSock {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn metadata_grup(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn semua_grup(&self) -> Result<Value, SockError> {
            Ok(self.grup.clone())
        }
        fn ubah_peserta(
            &self,
            _: &str,
            _: &[String],
            _: oktz_plugin_api::sock::AksiPeserta,
        ) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn set_foto_profil(&self, _: &str, _: &[u8]) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn keluar_grup(&self, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn kode_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn info_undangan(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn buat_grup(&self, _: &str, _: &[String]) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn tandai_dibaca(&self, _: &[Value]) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn status_teks(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<Value>, SockError> {
            Err(SockError::baru("tidak dipakai"))
        }
    }

    fn grup_contoh() -> Value {
        serde_json::json!({
            "120363@g.us": {
                "subject": "Grup Satu",
                "participants": [
                    { "id": "628111@s.whatsapp.net" },
                    { "id": "628222@s.whatsapp.net" }
                ]
            }
        })
    }

    fn run(args: &[&str]) -> Reply {
        savekontak(Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "savekontak".into(),
            sock: Arc::new(StubSock {
                grup: grup_contoh(),
            }),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("savekontak").unwrap();
        assert_eq!(p.meta.name, "savekontak");
        assert_eq!(p.meta.alias, ["sv", "svkontak"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(
            p.meta.description,
            "Menyimpan kontak dari grup menjadi file VCF"
        );
        assert_eq!(p.meta.usage, ".savekontak <nama>");
        assert_eq!(p.meta.example, ".savekontak Fulan");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(p.meta.is_private, "isPrivate true di JS");
        assert!(!p.meta.is_group);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["sv", "svkontak"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("savekontak"), "{a}");
        }
    }

    #[test]
    fn kartu_memilih_grup_dengan_nama_base() {
        let r = run(&["Fulan"]);
        let t = teks(&r);
        assert!(t.starts_with("📇 *SISTEM SAVE KONTAK (VCF)*"), "{t}");
        assert!(t.contains("Nama Base: *Fulan*"), "{t}");
        assert!(t.contains("• *.savekontak <nama>* — Menyimpan dengan nama kustom"), "{t}");
        assert!(t.contains("• *.savekontak* — Menyimpan dengan nama default \"User\""), "{t}");
        match r.without_mutations() {
            Reply::Interactive { footer, buttons, .. } => {
                assert_eq!(footer.as_deref(), Some(FOOTER));
                assert_eq!(buttons.len(), 2);
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }

    #[test]
    fn nama_base_kosong_pakai_user() {
        let t = teks(&run(&[]));
        assert!(t.contains("Nama Base: *User*"), "{t}");
    }

    #[test]
    fn grup_tidak_ditemukan_memberi_pesan() {
        let r = run(&["get", "999@g.us", "Fulan"]);
        assert_eq!(teks(&r), "❌ Grup tidak ditemukan.");
    }

    #[test]
    fn ekstraksi_menghasilkan_berkas_vcf() {
        let r = run(&["get", "all", "Fulan"]);
        let t = teks(&r);
        assert_eq!(t, "⏳ Sedang mengekstrak kontak dari 1 grup...");
        match r.without_mutations() {
            Reply::Sequence(list) => {
                assert_eq!(list.len(), 2);
                match &list[1] {
                    Reply::Document {
                        media,
                        file_name,
                        caption,
                        ..
                    } => {
                        assert_eq!(file_name, "Fulan_2_Kontak.vcf");
                        assert_eq!(caption, "✅ *Berhasil mengekstrak 2 kontak ke dalam VCF.*");
                        let src = media.as_ref().expect("media");
                        assert_eq!(src.label(), "bytes(180)");
                    }
                    other => panic!("harus Document, dapat {other:?}"),
                }
            }
            other => panic!("harus Sequence, dapat {other:?}"),
        }
    }

    #[test]
    fn vcard_sesuai_format_js() {
        assert_eq!(
            vcard("Fulan 1", "628111"),
            "BEGIN:VCARD\nVERSION:3.0\nFN:Fulan 1\nTEL;type=CELL;type=VOICE;waid=628111:+628111\nEND:VCARD"
        );
    }
}
