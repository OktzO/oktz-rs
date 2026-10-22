//! Sistem tabel WhatsApp — `sock.sendTable` / `sock.sendList` di JS.
//!
//! Bentuk wire-nya bukan pesan biasa, tapi `botForwardedMessage` yang
//! membungkus `richResponseMessage` (`AIRichResponseMessage`):
//!
//! ```text
//! Message {
//!   botForwardedMessage: FutureProofMessage {
//!     message: Message {
//!       richResponseMessage: AIRichResponseMessage {
//!         messageType: STANDARD,
//!         submessages: [ TEXT, TABLE, TEXT ],
//!         contextInfo: { ... }
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! Referensi: `src/lib/rich-messages.js:609-659` (`generateTableContent` /
//! `generateListContent`) dan `src/lib/socket.js:1290-1310` (`sendTable`,
//! `sendList`).
//!
//! `ponytail:` JS pernah mengirim `richResponseMessage` sebagai field
//! langsung `Message.richResponseMessage`; yang dipakai di sini adalah
//! `botForwardedMessage` — bentuk yang dipakai `buildBotForwardedMessage`
//! (`rich-messages.js:432-452`) dan yang terbukti dirender WA.

use whatsapp_rust::prelude::wa;
#[cfg(test)]
use whatsapp_rust::wacore::proto_helpers::MessageBuilderExt;
use whatsapp_rust::waproto::buffa::MessageField;

/// Isi tabel: judul, baris judul, dan baris data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    /// Judul tabel — `tableMetadata.title`.
    pub title: String,
    /// Baris judul — jadi baris pertama dengan `isHeading: true`.
    pub headers: Vec<String>,
    /// Baris data.
    pub rows: Vec<Vec<String>>,
    /// Teks sebelum tabel — submessage `messageType: 2`.
    pub header_text: Option<String>,
    /// Teks setelah tabel — submessage `messageType: 2`.
    pub footer: Option<String>,
}

impl Table {
    pub fn new(
        title: impl Into<String>,
        headers: impl IntoIterator<Item = impl Into<String>>,
        rows: impl IntoIterator<Item = Vec<String>>,
    ) -> Self {
        Self {
            title: title.into(),
            headers: headers.into_iter().map(Into::into).collect(),
            rows: rows.into_iter().collect(),
            header_text: None,
            footer: None,
        }
    }

    /// `options.headerText` (`rich-messages.js:616`).
    pub fn header_text(mut self, text: impl Into<String>) -> Self {
        self.header_text = Some(text.into());
        self
    }

    /// `options.footer` (`rich-messages.js:616`).
    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(text.into());
        self
    }
}

/// `generateListContent` (`rich-messages.js:638-659`) — tabel tanpa baris
/// judul; tiap item jadi baris satu kolom.
pub fn list_table(title: impl Into<String>, items: impl IntoIterator<Item = impl Into<String>>) -> Table {
    Table {
        title: title.into(),
        headers: Vec::new(),
        rows: items.into_iter().map(|i| vec![i.into()]).collect(),
        header_text: None,
        footer: None,
    }
}

/// Rakit pesan tabel siap kirim.
///
/// `context_info` = `buildRichContextInfo(quoted)` (`rich-messages.js:417-431`):
/// `forwardingScore: 1`, `isForwarded: true`, `forwardOrigin: 4`, plus
/// kutipan kalau ada.
pub fn build_table(table: &Table, context_info: Option<wa::ContextInfo>) -> wa::Message {
    let mut submessages = Vec::new();
    if let Some(text) = &table.header_text {
        submessages.push(text_submessage(text));
    }

    let mut rows = Vec::new();
    if !table.headers.is_empty() {
        rows.push(wa::ai_rich_response_table_metadata::AIRichResponseTableRow {
            items: table.headers.clone(),
            is_heading: Some(true),
        });
    }
    for row in &table.rows {
        rows.push(wa::ai_rich_response_table_metadata::AIRichResponseTableRow {
            items: row.clone(),
            is_heading: None,
        });
    }
    submessages.push(wa::AIRichResponseSubMessage {
        message_type: Some(wa::AIRichResponseSubMessageType::AiRichResponseTable),
        table_metadata: MessageField::some(wa::AIRichResponseTableMetadata {
            rows,
            title: Some(table.title.clone()),
        }),
        ..Default::default()
    });

    if let Some(text) = &table.footer {
        submessages.push(text_submessage(text));
    }

    let rich = wa::AIRichResponseMessage {
        message_type: Some(wa::AIRichResponseMessageType::AiRichResponseTypeStandard),
        submessages,
        context_info: match context_info {
            Some(ci) => MessageField::some(ci),
            None => MessageField::none(),
        },
        ..Default::default()
    };

    wa::Message {
        bot_forwarded_message: MessageField::some(wa::message::FutureProofMessage {
            message: MessageField::some(wa::Message {
                rich_response_message: MessageField::some(rich),
                ..Default::default()
            }),
        }),
        ..Default::default()
    }
}

/// Submessage teks — `messageType: 2`.
fn text_submessage(text: &str) -> wa::AIRichResponseSubMessage {
    wa::AIRichResponseSubMessage {
        message_type: Some(wa::AIRichResponseSubMessageType::AiRichResponseText),
        message_text: Some(text.to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use whatsapp_rust::waproto::buffa::Message as _;

    fn rich(msg: &wa::Message) -> &wa::AIRichResponseMessage {
        msg.bot_forwarded_message
            .as_option()
            .expect("botForwardedMessage")
            .message
            .as_option()
            .expect("pesan dalam")
            .rich_response_message
            .as_option()
            .expect("richResponseMessage")
    }

    #[test]
    fn tabel_membungkus_bot_forwarded_message() {
        let msg = build_table(
            &Table::new(
                "Distribusi Fitur",
                ["Kategori", "Jumlah", "Persen"],
                [vec!["🏠 MAIN".into(), "10".into(), "5.0%".into()]],
            ),
            None,
        );
        assert!(msg.bot_forwarded_message.is_set());
        let r = rich(&msg);
        assert_eq!(
            r.message_type,
            Some(wa::AIRichResponseMessageType::AiRichResponseTypeStandard)
        );
        assert_eq!(r.submessages.len(), 1, "tanpa header/footer = 1 submessage");
        let table = r.submessages[0].table_metadata.as_option().expect("table");
        assert_eq!(table.title.as_deref(), Some("Distribusi Fitur"));
        assert_eq!(table.rows.len(), 2, "baris judul + 1 baris data");
        assert_eq!(table.rows[0].is_heading, Some(true));
        assert_eq!(table.rows[0].items, ["Kategori", "Jumlah", "Persen"]);
        assert_eq!(table.rows[1].is_heading, None);
        assert_eq!(table.rows[1].items[0], "🏠 MAIN");
    }

    #[test]
    fn header_dan_footer_jadi_submessage_teks_di_awal_dan_akhir() {
        let table = Table::new("T", ["A"], [vec!["1".into()]])
            .header_text("Total: 10")
            .footer("Total 10 fitur");
        let msg = build_table(&table, None);
        let r = rich(&msg);
        assert_eq!(r.submessages.len(), 3);
        assert_eq!(
            r.submessages[0].message_type,
            Some(wa::AIRichResponseSubMessageType::AiRichResponseText)
        );
        assert_eq!(r.submessages[0].message_text.as_deref(), Some("Total: 10"));
        assert_eq!(
            r.submessages[2].message_type,
            Some(wa::AIRichResponseSubMessageType::AiRichResponseText)
        );
        assert_eq!(r.submessages[2].message_text.as_deref(), Some("Total 10 fitur"));
    }

    #[test]
    fn list_tanpa_baris_judul() {
        let table = list_table("Judul", ["satu", "dua"]);
        assert!(table.headers.is_empty());
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["satu".to_string()]);
    }

    #[test]
    fn context_info_bawa_kutipan_pemicu() {
        let ci = wa::ContextInfo {
            mentioned_jid: vec!["6281@s.whatsapp.net".into()],
            quoted_message: MessageField::some(wa::Message::text("asal")),
            ..Default::default()
        };
        let msg = build_table(&Table::new("T", ["A"], []), Some(ci));
        let r = rich(&msg);
        let got = r.context_info.as_option().expect("contextInfo");
        assert_eq!(got.mentioned_jid, vec!["6281@s.whatsapp.net".to_string()]);
        assert!(got.quoted_message.as_option().is_some());
    }

    #[test]
    fn pesan_tabel_bisa_diencode() {
        let msg = build_table(&Table::new("T", ["A"], [vec!["1".into()]]), None);
        let mut buf = Vec::new();
        msg.encode(&mut buf);
        assert!(!buf.is_empty());
    }
}
