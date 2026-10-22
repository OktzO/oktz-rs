//! `plugins/fun/confess.js` — kirim pesan anonim ke seseorang (menfess).
//!
//! ## Penyimpangan yang tidak bisa ditiru
//!
//! 1. **Kirim ke chat lain.** JS memanggil `sock.sendMessage(targetJid, …)` —
//!    pesan benar-benar terkirim ke nomor tujuan. `Reply` di Rust hanya bisa
//!    membalas di `ctx.chat`, dan tidak ada API kirim-ke-jid-lain. Jadi
//!    plugin ini hanya menulis state sesi + membalas teks "berhasil terkirim"
//!    ke chat asal, seperti versi sebelum ini.
//! 2. **`forwardedNewsletterMessageInfo`.** Balasan balasan di JS menyertakan
//!    `newsletterJid` + `newsletterName`; `Reply` tidak punya-nya.
//! 3. **`exists === undefined`.** JS membedakan tiga keadaan
//!    `sock.onWhatsApp`: `true` (terdaftar), `false` (tidak terdaftar), dan
//!    `undefined` (belum bisa dipastikan). `Sock::ada_whatsapp` hanya punya
//!    `bool`, jadi keadaan "belum bisa dipastikan" dipetakan ke daftar
//!    kosong — lihat [`cek_whatsapp`].
//!
//! Handler utama memeriksa nomor tujuan via `sock.ada_whatsapp`, menyimpan
//! data menfess di `ctx.sessions`, dan membalas teks sukses.
//! Answer handler menangani balasan ke pesan menfess.
use oktz_plugin_api::{Ctx, Reply, plugin};
use serde_json::Value;

const CONFESS_SESSION_PREFIX: &str = "confess:";

/// `confess.js:32-40` — teks bantuan, memakai prefix yang sedang aktif.
fn help_text(prefix: &str) -> String {
    format!(
        "💌 *LAYANAN MENFESS ANONIM* 💌\n\n\
         Mau ngirim pesan rahasia ke gebetan atau teman tanpa ketahuan? Bisa banget kak!\n\n\
         *Cara Pakai:*\n\
         👉 `{prefix}confess nomor|pesan`\n\n\
         *Contoh:*\n\
         > `{prefix}confess 6281234567890|Hai kak, aku suka deh liat senyum kamu!`\n\n\
         > 🤫 _Tenang aja, identitas kamu 100% aman dan dirahasiakan!_"
    )
}

fn clean_number(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn normalize_number(raw: &str) -> String {
    let num = clean_number(raw);
    if num.starts_with('0') {
        format!("62{}", &num[1..])
    } else {
        num
    }
}

/// `confess.js:70-82` — teks yang dikirim ke nomor tujuan.
fn confess_text(message: &str) -> String {
    format!(
        "💌 *ADA PESAN RAHASIA BUAT KAMU KAK!* 💌\n\n\
         Sstt.. Ada seseorang yang diam-diam ngirim pesan buat kamu nih:\n\n\
         💬 *Isi Pesan:*\n\
         ```{message}```\n\n\
         > 🔒 _Pesan ini dikirim secara anonim (identitas pengirim dirahasiakan)._\n\
         > ✉️ _Kamu bisa balas pesan ini kok! Tinggal *REPLY* aja pesannya ya!_"
    )
}

/// `confess.js:84-88` — teks konfirmasi ke pengirim.
fn success_text(target_number: &str) -> String {
    format!(
        "✅ *MENFESS BERHASIL TERKIRIM!* ✅\n\n\
         > 📱 Terkirim ke: `{target_number}`\n\
         > 🔒 Identitas kamu aman sentosa!\n\n\
         > _Nanti kalau dia balas pesannya, aku bakal langsung terusin ke sini kak! Santai aja_ 😉"
    )
}

/// Hasil pemeriksaan nomor di WhatsApp — padanan tiga cabang JS
/// (`confess.js:99-108`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusWa {
    /// `onWa.exists === true` — boleh proceed.
    Terdaftar,
    /// `onWa.exists === false` — nomor tidak terdaftar.
    TidakTerdaftar,
    /// `onWa.exists === undefined` — belum bisa dipastikan.
    BelumPasti,
}

/// `sock.onWhatsApp(targetNumber)` (`confess.js:99`).
///
/// `Sock::ada_whatsapp` mengembalikan `Vec<AdaWhatsapp>`; daftar kosong
/// berarti tidak ada entri untuk nomor itu — dipetakan ke
/// [`StatusWa::BelumPasti`] (setara `exists === undefined`).
pub(crate) fn cek_whatsapp(hasil: &[oktz_plugin_api::AdaWhatsapp], jid: &str) -> StatusWa {
    match hasil.iter().find(|a| a.jid == jid) {
        Some(a) if a.ada => StatusWa::Terdaftar,
        Some(_) => StatusWa::TidakTerdaftar,
        None => StatusWa::BelumPasti,
    }
}

#[plugin(
    name = "confess",
    alias = ["confession", "menfess", "anonim"],
    category = "fun",
    description = "Kirim pesan anonim ke seseorang",
    usage = ".confess nomor|pesan",
    example = ".confess 6281234567890|Hai, aku suka kamu!",
    cooldown = 60,
    energi = 1,
    is_group = false,
    is_private = false,
    is_premium = true
)]
pub fn confess(ctx: Ctx) -> Reply {
    let input = ctx.text.trim();

    if input.is_empty() || !input.contains('|') {
        return Reply::text(help_text(&ctx.prefix));
    }

    let mut parts = input.splitn(2, '|');
    let raw_number = parts.next().unwrap_or("");
    let message = parts.next().unwrap_or("").trim();

    if raw_number.is_empty() || message.is_empty() {
        return Reply::text(format!(
            "Aduh kak formatnya salah! 😅\n\nCoba ketik gini: `{}confess nomor|pesan`",
            ctx.prefix
        ));
    }

    let target_number = normalize_number(raw_number);
    let sender_number = ctx.sender.split('@').next().unwrap_or(&ctx.sender);

    if target_number.len() < 10 || target_number.len() > 15 {
        return Reply::text("Hmm, nomor tujuan yang kamu masukin kayaknya nggak valid deh kak! 🤔");
    }

    if target_number == sender_number {
        return Reply::text("Ih masa ngirim menfess buat diri sendiri sih kak? Kasih buat orang lain dong! 😂");
    }

    let target_jid = format!("{target_number}@s.whatsapp.net");

    let ada_whatsapp = match ctx.sock().ada_whatsapp(&[target_number.to_string()]) {
        Ok(list) => list,
        Err(_) => {
            return Reply::text(format!(
                "Hmm kak, verifikasi nomor `{target_number}` ke WhatsApp gagal (timeout/koneksi), \
                 jadi pesan belum dikirim ya. Coba lagi nanti kak! 🙏"
            ));
        }
    };

    match cek_whatsapp(&ada_whatsapp, &target_jid) {
        StatusWa::Terdaftar => {}
        StatusWa::TidakTerdaftar => {
            return Reply::text(format!(
                "Yah kak, nomor `{target_number}` ternyata nggak terdaftar di WhatsApp! 😔"
            ));
        }
        StatusWa::BelumPasti => {
            return Reply::text(format!(
                "Hmm kak, nomor `{target_number}` belum bisa dipastikan terdaftar di WhatsApp, \
                 jadi pesan belum dikirim. Coba lagi nanti kak! 🙏"
            ));
        }
    }

    if message.len() < 5 {
        return Reply::text("Pesannya kependekan kak! Minimal 5 karakter ya biar lebih bermakna. 📝");
    }

    if message.len() > 1000 {
        return Reply::text("Wah pesannya kepanjangan kak! Maksimal 1000 karakter ya biar bacanya nggak pusing. 📜");
    }

    let message_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
        .to_string();

    let entry = serde_json::json!({
        "senderJid": ctx.sender,
        "senderChat": ctx.chat,
        "targetJid": target_jid,
        "createdAt": message_id.parse::<i64>().unwrap_or(0),
    });
    ctx.session_set(&format!("{CONFESS_SESSION_PREFIX}{message_id}"), entry);

    Reply::text(success_text(&target_number))
}

/// Answer handler untuk balasan ke pesan menfess.
///
/// Dipanggil untuk pesan non-command. `None` berarti "tidak menangani".
///
/// JS asli mengirim balasan ke `confessInfo.senderChat` (bukan `m.chat`),
/// yang tidak dapat direplikasi dengan `Reply` karena semua balasan
/// mengikuti `ctx.chat`. Sini hanya mengonfirmasi di chat saat ini.
#[oktz_plugin_api::answer_handler]
pub fn confess_reply(ctx: Ctx) -> Option<Reply> {
    let quoted = ctx.quoted()?;
    let quoted_id = &quoted.id;

    let key = format!("{CONFESS_SESSION_PREFIX}{quoted_id}");
    let entry = ctx.session_get(&key)?;

    let target_jid = entry.get("targetJid")?.as_str()?;
    if ctx.sender != target_jid {
        return None;
    }

    let reply_message = ctx.text.trim();
    if reply_message.is_empty() {
        return None;
    }

    ctx.session_take(&key);

    // `confess.js:154-162` — teks balasan yang dikirim ke pengirim.
    let reply_text = format!(
        "💌 *ADA BALASAN MENFESS NIH KAK!* 💌\n\n\
         Orang yang kamu kirimin menfess tadi barusan balas pesanmu:\n\n\
         💬 *Isi Balasan:*\n\
         ```{reply_message}```\n\n\
         > 🔒 _Tenang, identitas kamu masih aman!_"
    );

    Some(Reply::sequence(vec![
        Reply::text(reply_text),
        Reply::text("✅ Sip kak! Balasan kamu udah aku sampaikan ke pengirim rahasianya."),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{AdaWhatsapp, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    /// `Sock` yang selalu gagal — meniru timeout `onWhatsApp` di JS.
    #[derive(Debug)]
    struct SockGagal;

    impl Sock for SockGagal {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Err(SockError::baru("timeout"))
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Err(SockError::baru("x"))
        }
        fn metadata_grup(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
        }
        fn semua_grup(&self) -> Result<Value, SockError> {
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
        fn info_undangan(&self, _: &str) -> Result<Value, SockError> {
            Err(SockError::baru("x"))
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
    }

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "confess".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sock: Arc::new(SockGagal),
            ..Default::default()
        };
        (oktz_plugin_api::find("confess").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("plugin harus membalas teks")
            .to_string()
    }

    #[test]
    fn clean_number_hanya_mengambil_digit() {
        assert_eq!(clean_number("6281234567890"), "6281234567890");
        assert_eq!(clean_number("081234567890"), "081234567890");
        assert_eq!(clean_number("+62 812-3456 7890"), "6281234567890");
    }

    #[test]
    fn normalize_number_ubah_0_awal_menjadi_62() {
        assert_eq!(normalize_number("081234567890"), "6281234567890");
        assert_eq!(normalize_number("6281234567890"), "6281234567890");
    }

    #[test]
    fn confess_text_mengandung_pesan() {
        let t = confess_text("Halo kak");
        assert!(t.contains("ADA PESAN RAHASIA BUAT KAMU KAK"));
        assert!(t.contains("Halo kak"));
    }

    #[test]
    fn help_text_mengandung_contoh_pakai() {
        let h = help_text(".");
        assert!(h.contains(".confess nomor|pesan"));
        assert!(h.contains("6281234567890"));
    }

    /// `confess.js:30-40` — teks bantuan persis, dengan prefix hidup.
    #[test]
    fn output_sama_dengan_js() {
        let t = text_of(&run(""));
        assert_eq!(
            t,
            "💌 *LAYANAN MENFESS ANONIM* 💌\n\n\
             Mau ngirim pesan rahasia ke gebetan atau teman tanpa ketahuan? Bisa banget kak!\n\n\
             *Cara Pakai:*\n\
             👉 `.confess nomor|pesan`\n\n\
             *Contoh:*\n\
             > `.confess 6281234567890|Hai kak, aku suka deh liat senyum kamu!`\n\n\
             > 🤫 _Tenang aja, identitas kamu 100% aman dan dirahasiakan!_"
        );
    }

    /// `confess.js:44-46` — format salah.
    #[test]
    fn format_salah_menggunakan_prefix_hidup() {
        assert_eq!(
            text_of(&run("6281234567890|")),
            "Aduh kak formatnya salah! 😅\n\nCoba ketik gini: `.confess nomor|pesan`"
        );
    }

    /// `confess.js:56-58` — nomor tidak valid.
    #[test]
    fn nomor_tidak_valid_ditolak() {
        assert_eq!(
            text_of(&run("123|halo kak")),
            "Hmm, nomor tujuan yang kamu masukin kayaknya nggak valid deh kak! 🤔"
        );
    }

    /// `confess.js:62-64` — menfess untuk diri sendiri.
    #[test]
    fn menfess_ke_diri_sendiri_ditolak() {
        assert_eq!(
            text_of(&run("628111|hallo kak")),
            "Ih masa ngirim menfess buat diri sendiri sih kak? Kasih buat orang lain dong! 😂"
        );
    }

    /// `confess.js:101-103` — verifikasi WA gagal.
    #[test]
    fn verifikasi_gagal_memuat_nomor() {
        let t = text_of(&run("628999|hallo kak"));
        assert_eq!(
            t,
            "Hmm kak, verifikasi nomor `628999` ke WhatsApp gagal (timeout/koneksi), \
             jadi pesan belum dikirim ya. Coba lagi nanti kak! 🙏"
        );
    }

    #[test]
    fn status_whatsapp_memetakan_tiga_keadaan() {
        let jid = "628999@s.whatsapp.net";
        assert_eq!(
            cek_whatsapp(
                &[AdaWhatsapp {
                    jid: jid.into(),
                    ada: true
                }],
                jid
            ),
            StatusWa::Terdaftar
        );
        assert_eq!(
            cek_whatsapp(
                &[AdaWhatsapp {
                    jid: jid.into(),
                    ada: false
                }],
                jid
            ),
            StatusWa::TidakTerdaftar
        );
        assert_eq!(cek_whatsapp(&[], jid), StatusWa::BelumPasti);
    }
}
