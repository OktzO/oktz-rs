//! `plugins/stalker/wastalk.js` — stalk profil WhatsApp.
//!
//! ## Penyimpangan
//!
//! - **`awesome-phonenumber`.** Tidak ada padanannya di dependency workspace;
//!   port ini memakai pemetaan kode negara → nama Inggris (pengganti
//!   `Intl.DisplayNames`) yang terbatas pada kode panggil umum, dan
//!   `getNumber('international')` diganti pengelompokan sederhana mengikuti
//!   pola `tools/qwa.rs` (3-4-4 untuk 62, 3-4-3 lainnya). Kode tak dikenal
//!   jatuh ke `"Unknown"` — persis cabang `catch` JS yang men-set
//!   `country = 'Unknown'`.
//! - **`format.isValid()`** hanya dipakai untuk `console.log` di JS — tidak
//!   berpengaruh ke output, jadi tidak di-port.
//! - Sisanya full-port: `onWhatsApp`, `profilePictureUrl` (+URL cadangan),
//!   `fetchStatus`, `getName`, `getBusinessProfile`, dan `LLLL` dari
//!   `timeHelper.fromTimestamp` (Asia/Jakarta, id-ID).

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};
use serde_json::{Value, Value as Json, json};

use crate::common::{days_from_civil, jakarta_parts};

/// URL cadangan saat profil tidak punya foto (`wastalk.js:38`).
const FOTO_CADANGAN: &str = "https://telegra.ph/file/70e8de9b1879568954f09.jpg";

/// Pemetaan kode panggil → (kode wilayah, nama Inggris) — pengganti
/// `awesome-phonenumber` + `Intl.DisplayNames(['en'])`. Dicocokkan dengan
/// prefix terpanjang; kode di luar daftar jatuh ke "Unknown" (cabang catch).
const KODE_NEGARA: [(&str, &str, &str); 64] = [
    ("62", "ID", "Indonesia"),
    ("60", "MY", "Malaysia"),
    ("65", "SG", "Singapore"),
    ("63", "PH", "Philippines"),
    ("66", "TH", "Thailand"),
    ("84", "VN", "Vietnam"),
    ("855", "KH", "Cambodia"),
    ("856", "LA", "Laos"),
    ("95", "MM", "Myanmar"),
    ("673", "BN", "Brunei"),
    ("880", "BD", "Bangladesh"),
    ("91", "IN", "India"),
    ("92", "PK", "Pakistan"),
    ("94", "LK", "Sri Lanka"),
    ("977", "NP", "Nepal"),
    ("975", "BT", "Bhutan"),
    ("86", "CN", "China"),
    ("81", "JP", "Japan"),
    ("82", "KR", "South Korea"),
    ("886", "TW", "Taiwan"),
    ("852", "HK", "Hong Kong"),
    ("853", "MO", "Macao"),
    ("960", "MV", "Maldives"),
    ("98", "IR", "Iran"),
    ("964", "IQ", "Iraq"),
    ("963", "SY", "Syria"),
    ("962", "JO", "Jordan"),
    ("961", "LB", "Lebanon"),
    ("972", "IL", "Israel"),
    ("966", "SA", "Saudi Arabia"),
    ("971", "AE", "United Arab Emirates"),
    ("974", "QA", "Qatar"),
    ("973", "BH", "Bahrain"),
    ("968", "OM", "Oman"),
    ("998", "UZ", "Uzbekistan"),
    ("995", "GE", "Georgia"),
    ("994", "AZ", "Azerbaijan"),
    ("90", "TR", "Turkey"),
    ("7", "RU", "Russia"),
    ("380", "UA", "Ukraine"),
    ("48", "PL", "Poland"),
    ("49", "DE", "Germany"),
    ("44", "GB", "United Kingdom"),
    ("33", "FR", "France"),
    ("34", "ES", "Spain"),
    ("39", "IT", "Italy"),
    ("31", "NL", "Netherlands"),
    ("32", "BE", "Belgium"),
    ("41", "CH", "Switzerland"),
    ("43", "AT", "Austria"),
    ("46", "SE", "Sweden"),
    ("47", "NO", "Norway"),
    ("45", "DK", "Denmark"),
    ("358", "FI", "Finland"),
    ("30", "GR", "Greece"),
    ("351", "PT", "Portugal"),
    ("36", "HU", "Hungary"),
    ("40", "RO", "Romania"),
    ("421", "SK", "Slovakia"),
    ("420", "CZ", "Czechia"),
    ("385", "HR", "Croatia"),
    ("1", "US", "United States"),
    ("61", "AU", "Australia"),
    ("64", "NZ", "New Zealand"),
];

/// Kode panggil Amerika Utara/Karibia yang bukan AS (ikutan prefix `1`).
const KODE_NEGARA_1: [(&str, &str); 4] = [
    ("1242", "Bahamas"),
    ("1876", "Jamaica"),
    ("1868", "Trinidad and Tobago"),
    ("1671", "Guam"),
];

/// `PhoneNum('+' + num).getRegionCode('mobile')` + `regionNames.of(...)`.
/// `None` = kode tak dikenal (JS: catch → `country = 'Unknown'`).
pub(crate) fn negara_dari_digits(digits: &str) -> Option<(&'static str, &'static str)> {
    // Prefix 4 digit dulu (kasus `1xxx` Karibia), lalu 3/2/1.
    for len in [4, 3, 2, 1] {
        if digits.len() < len {
            continue;
        }
        let prefix = &digits[..len];
        if len == 4 {
            if let Some((_, nama)) = KODE_NEGARA_1.iter().find(|(k, _)| *k == prefix) {
                return Some(("US", nama));
            }
            continue;
        }
        if let Some((_, region, nama)) = KODE_NEGARA.iter().find(|(k, ..)| *k == prefix) {
            return Some((region, nama));
        }
    }
    None
}

/// `format.getNumber('international')` versi sederhana (pola `tools/qwa.rs`):
/// `+{kode_negara}` diikuti nomor nasional yang dikelompokkan 3 lalu sisanya.
/// `prefix_len` = panjang kode panggil dari [`negara_dari_digits`].
pub(crate) fn format_international(digits: &str, prefix_len: usize) -> String {
    let kelompokkan = |d: &str| -> String {
        if d.len() <= 3 {
            return d.to_string();
        }
        format!("{} {}", &d[..3], &d[3..])
    };
    if prefix_len == 0 {
        return format!("+{}", kelompokkan(digits));
    }
    if let Some(nasional) = digits.get(prefix_len..) {
        return format!("+{} {}", &digits[..prefix_len], kelompokkan(nasional));
    }
    format!("+{digits}")
}

/// `timeHelper.fromTimestamp(setAt, 'LLLL')` — `Senin, 5 Januari 2026 pukul
/// HH.mm` di Asia/Jakarta (`time.js:45`).
pub(crate) fn format_LLLL(ms: i64) -> String {
    const HARI: [&str; 7] = [
        "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu", "Minggu",
    ];
    let (tahun, bulan, tanggal, jam, menit, _) = jakarta_parts(ms);
    let days = days_from_civil(tahun, bulan, tanggal);
    // 1970-01-01 = Kamis (indeks 3, SEN=0).
    let idx = (days + 3).rem_euclid(7) as usize;
    format!(
        "{}, {} {} {} pukul {jam:02}.{menit:02}",
        HARI[idx],
        tanggal,
        crate::common::MONTHS_ID[(bulan - 1) as usize],
        tahun
    )
}

/// Baris atas caption (`wastalk.js:72-79`).
fn caption_utama(
    country_upper: &str,
    name: &str,
    formatted: &str,
    number: &str,
    status: &str,
    date_status: &str,
) -> String {
    format!(
        "\t\t\t\t*▾ WHATSAPP ▾*\n\n\
         *° Country :* {country_upper}\n\
         *° Name :* {name}\n\
         *° Format Number :* {formatted}\n\
         *° Url Api :* wa.me/{number}\n\
         *° Mentions :* @{number}\n\
         *° Status :* {status}\n\
         *° Date Status :* {date_status}\n\n"
    )
}

/// Blok business (`wastalk.js:82-89`).
fn caption_bisnis(business: &Json) -> String {
    let teks = |k: &str| -> String {
        match business.get(k) {
            Some(Value::String(s)) if !s.is_empty() => s.clone(),
            Some(other) => other.to_string(),
            None => "undefined".to_string(),
        }
    };
    // `${business.x ? business.x : '-'}` — hanya website/email/address/
    // timezone/description yang punya fallback; category tidak.
    let pilih = |k: &str| -> String {
        match business.get(k).and_then(Value::as_str) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => "-".to_string(),
        }
    };
    let timezone = business
        .get("business_hours")
        .and_then(|h| h.get("timezone"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("-");
    format!(
        "\t\t\t\t*▾ INFO BUSINESS ▾*\n\n\
         *° BusinessId :* {}\n\
         *° Website :* {}\n\
         *° Email :* {}\n\
         *° Category :* {}\n\
         *° Address :* {}\n\
         *° Timezone :* {}\n\
         *° Description :* {}",
        teks("wid"),
        pilih("website"),
        pilih("email"),
        teks("category"),
        pilih("address"),
        timezone,
        pilih("description"),
    )
}

#[plugin(
    name = "wastalk",
    alias = ["whatsappstalk", "stalkwa"],
    category = "stalker",
    description = "Stalk profile WhatsApp",
    usage = ".wastalk <nomor/tag>",
    example = ".wastalk 6281234567890",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_bot_admin = false,
    is_admin = false
)]
pub fn wastalk(ctx: Ctx) -> Reply {
    // `m.quoted?.sender || m.mentionedJid?.[0] || text` (`wastalk.js:24`).
    let sumber = if let Some(q) = ctx.quoted() {
        (!q.sender.is_empty()).then(|| q.sender.clone())
    } else if let Some(m) = ctx.mentioned.first() {
        Some(m.clone())
    } else {
        (!ctx.text.is_empty()).then(|| ctx.text.clone())
    };

    let Some(sumber) = sumber else {
        return Reply::text(format!(
            "Example: {prefix}{command} @tag / 628xxx",
            prefix = ctx.prefix,
            command = ctx.command
        ));
    };

    // `num.replace(/\D/g, '') + '@s.whatsapp.net'`.
    let digits: String = sumber.chars().filter(|c| c.is_ascii_digit()).collect();
    let jid = format!("{digits}@s.whatsapp.net");
    let number = digits.clone();

    // `sock.onWhatsApp(num)` — entri pertama saja (`wastalk.js:33-36`).
    match ctx.sock().ada_whatsapp(&[digits.clone()]) {
        Ok(list) => {
            let ada = list.first().is_some_and(|a| a.ada);
            if !ada {
                return Reply::text("❌ User not exists on WhatsApp");
            }
        }
        Err(_) => return Reply::text("❌ Failed to stalk user."),
    }

    // Foto profil — gagal/None → URL cadangan (`wastalk.js:38-41`).
    let img = match ctx.sock().foto_profil(&jid, "image") {
        Ok(Some(url)) if !url.is_empty() => url,
        _ => FOTO_CADANGAN.to_string(),
    };

    // Bio — gagal → `{}` (`wastalk.js:44-46`).
    let bio = ctx.sock().status_teks(&jid).unwrap_or(Value::Null);

    // Nama — gagal → 'Unknown' (`wastalk.js:48-51`).
    let name = match ctx.sock().nama_kontak(&jid) {
        Ok(n) if !n.is_empty() => n,
        Ok(_) => number.clone(),
        Err(_) => "Unknown".to_string(),
    };

    // Business — gagal → null (`wastalk.js:53-56`).
    let business = ctx.sock().profil_bisnis(&jid).unwrap_or(None);

    // Negara + format nomor.
    let (country_upper, formatted) = match negara_dari_digits(&digits) {
        Some((region, nama)) => {
            let prefix_len = KODE_NEGARA_1
                .iter()
                .find(|(k, _)| digits.starts_with(k))
                .map(|(k, _)| k.len())
                .or_else(|| {
                    KODE_NEGARA
                        .iter()
                        .find(|(k, ..)| *k == region || digits.starts_with(*k))
                        .map(|(k, ..)| k.len())
                })
                .unwrap_or(0);
            (nama.to_uppercase(), format_international(&digits, prefix_len))
        }
        None => ("UNKNOWN".to_string(), number.clone()),
    };

    let status = bio
        .get("status")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("-");
    let date_status = bio
        .get("setAt")
        .and_then(Value::as_i64)
        .filter(|t| *t != 0)
        .map(format_LLLL)
        .unwrap_or_else(|| "-".to_string());

    let mut res = caption_utama(
        &country_upper,
        &name,
        &formatted,
        &number,
        status,
        &date_status,
    );

    match &business {
        Some(b) => res.push_str(&caption_bisnis(b)),
        None => res.push_str("*Standard WhatsApp Account*"),
    }

    Reply::Image {
        media: Some(MediaSource::url(img)),
        caption: res,
        mentions: vec![jid],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{AdaWhatsapp, Sock, SockError};
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";
    const TARGET: &str = "628222@s.whatsapp.net";

    /// Sock stub dengan jawaban tetap.
    #[derive(Debug, Default)]
    struct SockStub {
        ada: bool,
        foto: Option<String>,
        status: Value,
        nama: String,
        bisnis: Option<Value>,
    }

    impl Sock for SockStub {
        fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
            Ok(vec![AdaWhatsapp {
                jid: TARGET.into(),
                ada: self.ada,
            }])
        }
        fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
            Ok(self.foto.clone())
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
            if self.status.is_null() {
                Err(SockError::baru("no bio"))
            } else {
                Ok(self.status.clone())
            }
        }
        fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
            if self.nama.is_empty() {
                Err(SockError::baru("x"))
            } else {
                Ok(self.nama.clone())
            }
        }
        fn profil_bisnis(&self, _: &str) -> Result<Option<Value>, SockError> {
            Ok(self.bisnis.clone())
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

    fn run(sock: SockStub, text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "wastalk".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: text.into(),
            sock: Arc::new(sock),
            ..Default::default()
        };
        wastalk(ctx)
    }

    fn caption_of(r: Reply) -> (String, MediaSource, Vec<String>) {
        match r {
            Reply::Image { media, caption, mentions } => (caption, media.unwrap(), mentions),
            other => panic!("harus Image, dapat {other:?}"),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("wastalk").unwrap();
        assert_eq!(p.meta.name, "wastalk");
        assert_eq!(p.meta.alias, ["whatsappstalk", "stalkwa"]);
        assert_eq!(p.meta.category, "stalker");
        assert_eq!(p.meta.description, "Stalk profile WhatsApp");
        assert_eq!(p.meta.usage, ".wastalk <nomor/tag>");
        assert_eq!(p.meta.example, ".wastalk 6281234567890");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 2);
        assert!(!p.meta.is_group && !p.meta.is_admin && !p.meta.is_bot_admin);
        for a in ["whatsappstalk", "stalkwa"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "wastalk");
        }
    }

    #[test]
    fn tanpa_nomor_menampilkan_contoh() {
        let r = run(SockStub::default(), "");
        assert_eq!(r.text_body().unwrap(), "Example: .wastalk @tag / 628xxx");
    }

    #[test]
    fn nomor_tidak_terdaftar_ditolak() {
        let r = run(SockStub { ada: false, ..Default::default() }, "628222");
        assert_eq!(r.text_body().unwrap(), "❌ User not exists on WhatsApp");
    }

    #[test]
    fn caption_standard_account_lengkap() {
        let sock = SockStub {
            ada: true,
            foto: Some("https://pp.example/foto.jpg".into()),
            status: json!({ "status": "Hidup", "setAt": 1_767_139_200_000i64 }),
            nama: "Budi".into(),
            bisnis: None,
        };
        let (caption, media, mentions) = caption_of(run(sock, "628222"));
        assert!(caption.contains("*° Country :* INDONESIA"), "{caption}");
        assert!(caption.contains("*° Name :* Budi"), "{caption}");
        assert!(caption.contains("*° Format Number :* +62 822 2"), "{caption}");
        assert!(caption.contains("*° Url Api :* wa.me/628222"), "{caption}");
        assert!(caption.contains("*° Mentions :* @628222"), "{caption}");
        assert!(caption.contains("*° Status :* Hidup"), "{caption}");
        assert!(caption.contains("*° Date Status :* "), "{caption}");
        assert!(caption.ends_with("*Standard WhatsApp Account*"), "{caption}");
        assert!(caption.starts_with("\t\t\t\t*▾ WHATSAPP ▾*"), "{caption}");
        match media {
            MediaSource::Url { url, .. } => assert_eq!(url, "https://pp.example/foto.jpg"),
            other => panic!("harus Url, dapat {other:?}"),
        }
        assert_eq!(mentions, &[TARGET.to_string()]);
    }

    #[test]
    fn bio_dan_nama_kosong_jadi_fallback_js() {
        let sock = SockStub {
            ada: true,
            foto: None,
            status: Value::Null,
            nama: "".to_string(),
            bisnis: None,
        };
        let (caption, media, _) = caption_of(run(sock, "628222"));
        assert!(caption.contains("*° Name :* Unknown"), "{caption}");
        assert!(caption.contains("*° Status :* -"), "{caption}");
        assert!(caption.contains("*° Date Status :* -"), "{caption}");
        match media {
            MediaSource::Url { url, .. } => assert_eq!(url, FOTO_CADANGAN),
            other => panic!("harus Url, dapat {other:?}"),
        }
    }

    #[test]
    fn blok_bisnis_dengan_semua_field() {
        let sock = SockStub {
            ada: true,
            foto: None,
            status: Value::Null,
            nama: "Toko".into(),
            bisnis: Some(json!({
                "wid": "628222@biz",
                "website": "https://toko.id",
                "email": "cs@toko.id",
                "category": "Retail",
                "address": "Jl. Mawar 1",
                "business_hours": { "timezone": "Asia/Jakarta" },
                "description": "Jual aneka"
            })),
        };
        let (caption, _, _) = caption_of(run(sock, "628222"));
        assert!(caption.contains("*▾ INFO BUSINESS ▾*"), "{caption}");
        assert!(caption.contains("*° BusinessId :* 628222@biz"), "{caption}");
        assert!(caption.contains("*° Website :* https://toko.id"), "{caption}");
        assert!(caption.contains("*° Category :* Retail"), "{caption}");
        assert!(caption.contains("*° Timezone :* Asia/Jakarta"), "{caption}");
        assert!(caption.contains("*° Description :* Jual aneka"), "{caption}");
    }

    #[test]
    fn blok_bisnis_tanpa_field_jadi_tanda_hubung() {
        let sock = SockStub {
            ada: true,
            foto: None,
            status: Value::Null,
            nama: "Toko".into(),
            bisnis: Some(json!({ "wid": "628222@biz", "category": "Retail" })),
        };
        let (caption, _, _) = caption_of(run(sock, "628222"));
        assert!(caption.contains("*° Website :* -"), "{caption}");
        assert!(caption.contains("*° Email :* -"), "{caption}");
        assert!(caption.contains("*° Timezone :* -"), "{caption}");
    }

    #[test]
    fn negara_dari_digits_longest_prefix() {
        assert_eq!(negara_dari_digits("628123"), Some(("ID", "Indonesia")));
        assert_eq!(negara_dari_digits("12125550123"), Some(("US", "United States")));
        // 4 digit Karibia menang atas prefix 1.
        assert_eq!(
            negara_dari_digits("12425551234"),
            Some(("US", "Bahamas"))
        );
        assert_eq!(negara_dari_digits("447911123456"), Some(("GB", "United Kingdom")));
        assert_eq!(negara_dari_digits("999123"), None);
    }

    #[test]
    fn format_international_mengikuti_pola_qwa() {
        assert_eq!(format_international("628123456789", 2), "+62 812 3456789");
        assert_eq!(format_international("6281234567890", 2), "+62 812 34567890");
        assert_eq!(format_international("12125550123", 1), "+1 212 5550123");
        assert_eq!(format_international("812", 0), "+812");
    }

    #[test]
    fn format_LLLL_meniru_time_js() {
        // 2026-01-01T00:00:00Z → 07:00 WIB, hari Kamis.
        let ms = 1_767_225_600_000i64;
        assert_eq!(format_LLLL(ms), "Kamis, 1 Januari 2026 pukul 07.00");
        // 2026-08-17T00:00 WIB = Senin.
        let ms = 1_786_899_600_000i64;
        assert_eq!(format_LLLL(ms), "Senin, 17 Agustus 2026 pukul 00.00");
    }

    #[test]
    fn quoted_dan_mention_dipakai_sebelum_teks() {
        // Quoted sender menang.
        let ctx = Ctx {
            prefix: ".".into(),
            command: "wastalk".into(),
            sender: ME.into(),
            chat: ME.into(),
            text: "abc".into(),
            mentioned: vec!["628444@s.whatsapp.net".into()],
            quoted: Some(oktz_plugin_api::QuotedMessage {
                id: "Q".into(),
                sender: TARGET.into(),
                chat: ME.into(),
                text: String::new(),
                kind: oktz_plugin_api::QuotedKind::Text,
                mimetype: None,
                file_name: None,
                from_me: false,
                timestamp: 0,
            }),
            sock: Arc::new(SockStub {
                ada: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        let (_, _, mentions) = caption_of(wastalk(ctx));
        assert_eq!(mentions, &[TARGET.to_string()]);
    }
}
