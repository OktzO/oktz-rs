//! Model balasan rich (V2, list, AI rich, prepared rich) untuk plugin.
//!
//! Padanan JS: `src/lib/rich-messages.js`
//! (`generateTableContentV2`, `generateListContent`, `prepareRichResponseMessage`)
//! dan `src/lib/builder.js` (`AIRich`).
//!
//! Bentuk plugin sudah berupa model SEMANTIK: apa yang ditampilkan, bukan
//! byte protobuf-nya. Pemetaan ke `wa::Message` ada di `oktz-wa::rich`.
//! Batas itu disengaja — `oktz-plugin-api` tidak boleh bergantung pada crate
//! yang memegang protokol WA (lihat komentar `kartu` di `lib.rs`).
//!
//! # Kenapa empat model terpisah, bukan satu
//!
//! JS punya empat jalur yang berbeda BESAR wire-nya: V1 tabel/list
//! (tanpa `unifiedResponse`), V2 (`unifiedResponse` + outer
//! `messageContextInfo`), `prepareRichResponseMessage` (proof +
//! `botResponseId`), dan AIRich builder (bot metadata tanpa proof).
//! Menyatukannya jadi satu struct berarti satu field yang salah isi bisa
//! mengubah pesan yang terlihat user di jalur yang tak dimaksud. Karena itu
//! tiap model berdiri sendiri, dan `botJid`+`forwardingScore` masing-masing
//! dipegang di sisi `oktz-wa` (`oktz-wa::rich`).
//!
//! `sections` sengaja `Vec<serde_json::Value>`: isinya milik produksi JS
//! (bentuk `view_model`/`primitive`/`__typename`) dan tidak ada satu pun
//! plugin yang membangunnya selain menyalin. Memberinya tipe Rust berarti
//! menebak bentuk yang belum dipakai.

/// Tabel V2 — `generateTableContentV2` (`rich-messages.js:978`).
///
/// `header` + `baris` dipakai dua kali: sebagai baris `tableMetadata`
/// (`AI_RICH_RESPONSE_TABLE`) dan sebagai `unified_rows` di
/// `unifiedResponse` (`GenATableUXPrimitive`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RichV2 {
    /// `tableMetadata.title` / judul section heading.
    pub judul: String,
    /// Baris heading (`AI_RICH_RESPONSE_TABLE`, `isHeading`).
    pub header: Vec<String>,
    /// Baris isi. Sel yang kosong ikut dikirim — JS tidak padding
    /// sel, tapi `unified_rows` padding-kan max: itu bagian `oktz-wa`.
    pub baris: Vec<Vec<String>>,
    /// Section heading (`headerText || title`).
    pub heading: Option<String>,
    /// Section teks terpisah.
    pub teks: Option<String>,
    /// Section footer.
    pub footer: Option<String>,
}

impl RichV2 {
    pub fn baru(judul: impl Into<String>) -> Self {
        Self {
            judul: judul.into(),
            ..Default::default()
        }
    }

    pub fn header(mut self, header: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.header = header.into_iter().map(Into::into).collect();
        self
    }

    pub fn baris(mut self, baris: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.baris.push(baris.into_iter().map(Into::into).collect());
        self
    }

    pub fn heading(mut self, teks: impl Into<String>) -> Self {
        self.heading = Some(teks.into());
        self
    }

    pub fn teks(mut self, teks: impl Into<String>) -> Self {
        self.teks = Some(teks.into());
        self
    }

    pub fn footer(mut self, teks: impl Into<String>) -> Self {
        self.footer = Some(teks.into());
        self
    }
}

/// Daftar/list V1 — `generateListContent` (`rich-messages.js:638`).
///
/// Bedanya dengan tabel: TIDAK ADA baris heading. `items` yang bukan array
/// dibungkus jadi satu sel (`[String(item)]`) di sisi JS.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RichList {
    /// `tableMetadata.title`.
    pub judul: String,
    /// Baris isi; tiap baris = sel per kolom.
    pub items: Vec<Vec<String>>,
    /// Teks sebelum tabel (`messageType: 2`).
    pub heading: Option<String>,
    /// Teks setelah tabel (`messageType: 2`).
    pub footer: Option<String>,
}

impl RichList {
    pub fn baru(judul: impl Into<String>) -> Self {
        Self {
            judul: judul.into(),
            ..Default::default()
        }
    }

    pub fn item(mut self, item: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.items.push(item.into_iter().map(Into::into).collect());
        self
    }

    pub fn heading(mut self, teks: impl Into<String>) -> Self {
        self.heading = Some(teks.into());
        self
    }

    pub fn footer(mut self, teks: impl Into<String>) -> Self {
        self.footer = Some(teks.into());
        self
    }
}

use crate::airich::KodeToken;

/// Submessage untuk AIRich dan prepared rich — `builder.js:1037-1115`.
///
/// Tiga varian inilah yang dipakai `builder.js`: teks (`messageType: 2`),
/// tabel (`messageType: 4`), kode (`messageType: 5`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiSubMessage {
    Teks { teks: String },
    Tabel { judul: String, baris: Vec<Vec<String>>, heading: bool },
    Kode { bahasa: String, blok: Vec<KodeToken> },
}

/// AI rich — kontrak ruby-baileys `AIRich` (`MessageBuilder/index.js:2174-2380`),
/// bukan `AIRich` ourin-md. Alasan: kontrak ruby-baileys/rich-webui/play2
/// yang terbukti render (botResponseId SELALU ada, botJid
/// `867051314767696@bot`, TANPA `richResponseSourcesMetadata`). Satu stanza
/// saja — `sendEdit` ruby-baileys adalah edit streaming (`MESSAGE_EDIT`),
/// bukan syarat render (ourin-md tidak mengirimnya).
///
/// `ponytail:` `RichAi.bot_response_id`/`bot_jid` dihapus — bukan lagi opsi,
/// melainkan bagian kontrak tetap (`ai_outer`). Kalau butuh variasi, tambah
/// field lagi.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RichAi {
    pub submessages: Vec<AiSubMessage>,
    pub sections: Vec<serde_json::Value>,
    /// `botMetadata.messageDisclaimerText` — `AIRich.send` menulis `_title`
    /// (`builder.js:1509`) yang untuk `.ai`/`.ytplay` `undefined` → field
    /// TIDAK dikirim; play2 (`sendMusicPlayer`) menulis `""` eksplisit.
    pub disclaimer: Option<String>,
    /// `AIRich.send(jid, { quoted })` (`ruby-baileys:2196-2204`): pasang
    /// `stanzaId`/`participant`/`quotedType: 0`/`quotedMessage` pemicu ke
    /// `contextInfo` dalam.
    pub quoted: bool,
    /// Override `forwardingScore` context dalam — play2 `sendMusicPlayer`
    /// (`play2.js:932`) pakai 999, bukan 1.
    pub skor: Option<u32>,
    /// Override `forwardedAiBotMessageInfo.botJid` context dalam. `None` =
    /// `867051314767696@bot` (kontrak teks `.ai`, terbukti render). Kartu
    /// HTML wajib `0@bot`: satu-satunya nilai yang render mandiri di
    /// Android 2.26.45.99 (`.cekkartu` L/M ×3, adendum 8; capture Vanta
    /// 2026-11-18). Dengan JID Meta AI, kartu HTML hanya render lewat
    /// edit susulan (`.cekkartu` K).
    pub bot_jid: Option<String>,
    /// Kirim `botMetadata.richResponseSourcesMetadata = {sources: []}`.
    /// Default tidak dikirim (ruby-baileys v4.7 menghapusnya); capture Vanta
    /// memuatnya kosong. Diagnostik `.cekkartu`.
    pub sources_metadata: bool,
    /// Susulkan edit identik `protocolMessage type=14` setelah kartu
    /// (zaileys `bypassDownload`, commit `35f9c7b4`): di Android 2.26.45.99
    /// kartu HTML ber-botJid Meta AI hanya render setelah edit-relai
    /// (`.cekkartu` K ×3, adendum 8). Edit dibungkus `botForwardedMessage`
    /// seperti aslinya. Dipakai `.play2`/`.ytplay` bersama `bot_jid` `0@bot`.
    pub edit_susulan: bool,
}

/// Prepared rich — `prepareRichResponseMessage` (`rich-messages.js:1502`).
///
/// `wrapToBotForwardedMessage` (`:1480`) menambahkan proof acak
/// (`verificationMetadata`) dan `botResponseId = unified.response_id`.
/// Context dalam: score 1, `867051314767696@bot` — SAMA dengan V1 list,
/// tapi beda dari AIRich (`0@bot`) dan V2 (`259786046210223@bot`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreparedRich {
    pub submessages: Vec<AiSubMessage>,
    pub sections: Vec<serde_json::Value>,
    pub disclaimer: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v2() -> RichV2 {
        RichV2::baru("Judul")
            .header(["A", "B"])
            .baris(["1", "2"])
            .heading("H")
            .teks("T")
            .footer("F")
    }

    #[test]
    fn rich_v2_preserves_parsed_rows_and_sections() {
        let v = v2();
        assert_eq!(v.judul, "Judul");
        assert_eq!(v.header, vec!["A", "B"]);
        assert_eq!(v.baris, vec![vec!["1", "2"]]);
        assert_eq!(v.heading.as_deref(), Some("H"));
        assert_eq!(v.teks.as_deref(), Some("T"));
        assert_eq!(v.footer.as_deref(), Some("F"));
        // Default cuma dipakai builder: tidak boleh memuat nilai karangan.
        assert_eq!(RichV2::default(), RichV2::baru(""));
    }

    #[test]
    fn rich_list_keeps_rows_without_a_heading_row() {
        let l = RichList::baru("Judul")
            .item(["satu"])
            .item(["x", "y"])
            .heading("H")
            .footer("F");
        assert_eq!(l.items, vec![vec!["satu"], vec!["x", "y"]]);
        assert_eq!(l.heading.as_deref(), Some("H"));
        assert_eq!(l.footer.as_deref(), Some("F"));
    }

    #[test]
    fn ai_and_prepared_carry_the_same_shape_but_stay_distinct_types() {
        // Bentuk identik DISENGAJA — plugin melihat satu bentuk. Tipe
        // terpisah supaya builder tidak bisa tertukar di kompilasi.
        let ai = RichAi {
            submessages: vec![AiSubMessage::Teks { teks: "t".into() }],
            sections: vec![serde_json::json!({"s": 1})],
            disclaimer: Some("d".into()),
            quoted: false,
            skor: None,
            ..Default::default()
        };
        let p = PreparedRich {
            submessages: ai.submessages.clone(),
            sections: ai.sections.clone(),
            disclaimer: ai.disclaimer.clone(),
        };
        assert_eq!(ai.submessages, p.submessages);
        assert_eq!(ai.sections, p.sections);
        assert_eq!(ai.disclaimer, p.disclaimer);
        // Section mentah diteruskan apa adanya — plugin bisa saja
        // membawa bentuk `view_model` yang belum dimodelkan di sini.
        assert_eq!(ai.sections[0], serde_json::json!({"s": 1}));
    }

    #[test]
    fn ai_submessage_table_carries_title_rows_and_heading_flag() {
        let s = AiSubMessage::Tabel {
            judul: "J".into(),
            baris: vec![vec!["a".into()]],
            heading: true,
        };
        let AiSubMessage::Tabel { judul, baris: rows, heading } = &s else {
            panic!("tabel harus terisi");
        };
        assert_eq!(judul, "J");
        assert_eq!(rows, &vec![vec!["a".to_string()]]);
        assert!(heading);
    }
}
