//! Bukti bahwa `.menu` benar-benar terkirim sebagai KARTU native-flow.
//!
//! `tests/live_route.rs` membuktikan command mencapai plugin sungguhan.
//! Test ini melanjutkan dari situ: balasan `.menu` harus sampai ke WA
//! sebagai kartu native-flow plus extra stanza node `biz` — bukan teks
//! biasa. Tanpa node `biz`, WA diam-diam membuang kartu.
//!
//! Batas: `MessageSender` di sini tiruan, jadi soket WA belum tersentuh.
//! Yang terbukti: plugin `menu` mengembalikan kartu, pipeline meneruskannya
//! ke chat asal, dan node `biz` ikut. Bentuk wire (viewOnceMessage +
//! nativeFlowMessage) sudah dibuktikan unit test `oktz-wa` (`pipeline.rs`,
//! `cards.rs`) — di sini yang diuji jalur nyata: plugin → pipeline →
//! pengirim.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use oktz_router::command::ParseOptions;
use oktz_store::json_store::JsonStore;
use oktz_wa::pipeline::PendingCard;
use oktz_wa::{
    InboundMessage, Jid, Message, MessageInfo, MessageSender, MessageSource, Node, Outcome,
    Pipeline, SendBox, SendOptions, StoreContext,
};

// Tanpa rujukan ini `oktz_plugins` tidak ter-link ke binari test dan
// `find("menu")` mengembalikan `None` — bukan karena pluginnya hilang, tapi
// karena tidak ada yang mendaftarkannya.
const _: fn() = || {
    let _ = oktz_plugins::main::menu::menu;
};

const CHAT: &str = "628111@s.whatsapp.net";
const BOT: &str = "628999@s.whatsapp.net";

static SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct Recorder {
    kartu: Mutex<Vec<(String, PendingCard, Vec<Node>)>>,
    teks: Mutex<Vec<(String, String)>>,
}

impl MessageSender for Recorder {
    fn send(&self, to: &str, message: Message, _opts: SendOptions) -> SendBox<'_> {
        let isi = message.conversation.clone().unwrap_or_default();
        self.teks.lock().unwrap().push((to.to_string(), isi));
        Box::pin(async { Ok(()) })
    }

    /// Kartu `.menu` punya header gambar, jadi belum wire-ready: pipeline
    /// menahannya di [`PendingCard`] sampai media header ter-upload.
    fn send_card(&self, to: &str, card: PendingCard, opts: SendOptions) -> SendBox<'_> {
        self.kartu.lock().unwrap().push((
            to.to_string(),
            card,
            opts.extra_stanza_nodes.clone(),
        ));
        Box::pin(async { Ok(()) })
    }
}

fn store_kosong() -> Arc<JsonStore> {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("oktz-menu-card-{n}.json"));
    std::fs::write(&p, r#"{"settings":{},"users":{}}"#).unwrap();
    Arc::new(JsonStore::open(&p).unwrap())
}

/// Config seperti produksi: nama bot terisi dan nomor owner ada — tanpa
/// nomor owner, tombol "Owner Kami" jadi `quick_reply` (bukan `cta_url`).
fn config_produksi() -> oktz_plugin_api::config::Runtime {
    oktz_plugin_api::config::Runtime {
        bot: oktz_plugin_api::config::BotInfo {
            name: "Oktz Bot".into(),
            version: "3.3.1".into(),
            developer: "Zann".into(),
            website: String::new(),
        },
        owner: oktz_plugin_api::config::Owner {
            name: "Oktz".into(),
            number: vec!["6285608953677".into()],
        },
        ..Default::default()
    }
}

fn inbound(body: &str) -> InboundMessage {
    InboundMessage::builder()
        .message(Arc::new(Message {
            conversation: Some(body.to_string()),
            ..Default::default()
        }))
        .info(Arc::new(MessageInfo {
            source: MessageSource {
                chat: CHAT.parse::<Jid>().unwrap(),
                sender: CHAT.parse::<Jid>().unwrap(),
                is_from_me: false,
                is_group: false,
                ..Default::default()
            },
            id: "M1".to_string(),
            ..Default::default()
        }))
        .build()
}

#[tokio::test]
async fn menu_dikirim_sebagai_kartu_native_flow_dengan_node_biz() {
    let rec = Arc::new(Recorder::default());
    let p = Pipeline::new(
        StoreContext::new(store_kosong()),
        rec.clone(),
        BOT,
        ParseOptions::default(),
    )
    .with_config(config_produksi());

    let out = p.process(&inbound(".menu"), 1_000).await.unwrap();
    assert_eq!(out, Outcome::Sent, ".menu harus dibalas");

    let terkirim = rec.kartu.lock().unwrap().clone();
    assert_eq!(terkirim.len(), 1, "tepat satu kartu");
    let (chat, card, node) = &terkirim[0];
    assert_eq!(chat, CHAT, "kartu harus dikirim ke chat asal");

    // Node `biz` wajib; tanpa itu WA membuang kartu.
    let tag_biz = node.iter().any(|n| n.tag == "biz");
    assert!(tag_biz, "node biz wajib ada: {node:?}");

    // Varian bawaan (`config.ui.menuVariant` = 3) — kartu daftar menu.
    assert!(
        card.body.contains("Hello Brother"),
        "body kartu: {}",
        card.body
    );
    assert!(
        card.body.contains("BOT INFORMATION"),
        "body kartu: {}",
        card.body
    );

    let aksi: Vec<&str> = card.buttons.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(
        aksi,
        vec!["single_select", "quick_reply", "quick_reply"],
        "tombol kartu: {aksi:?}"
    );
}
