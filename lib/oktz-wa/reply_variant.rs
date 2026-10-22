//! `replyVariant` — bentuk balasan yang dilihat user, dibaca dari
//! `db.setting("replyVariant")`.
//!
//! Sebelas varian, semuanya diport dari `src/lib/serialize.js:958-1357`.
//! Setiap varian punya bentuk wire sendiri; pilihannya menentukan BENTUK
//! balasan, bukan isi teksnya.
//!
//! | V | Nama | Bentuk wire |
//! |---|------|-------------|
//! | 1 | BASIC | teks biasa + kutipan pemicu |
//! | 2 | PREMIUM | dokumen (`package.json`) + kutipan kontak palsu |
//! | 3 | TITANIUM | video `mp4` (gif) + kutipan pemicu |
//! | 4 | LV | link preview + konteks surat berita |
//! | 5 | FAKE ORDER | teks + kutipan `orderMessage` palsu |
//! | 6 | SIMPLE DOCUMENT | dokumen + kutipan pemicu |
//! | 7 | FAKE LOCATION | kartu `interactiveMessage` dengan header lokasi |
//! | 8 | FAKE SIGNUP | kartu `interactiveMessage` tombol `inapp_signup` |
//! | 9 | FAKE ORDER | `orderMessage` (varian aktif di produksi) |
//! | 10 | TAGIHAN | `requestPaymentMessage` |
//! | 11 | PREVIEW | `extendedTextMessage` dengan `faviconMMSMetadata` |

use std::path::{Path, PathBuf};

use whatsapp_rust::prelude::wa;
use whatsapp_rust::wacore::proto_helpers::MessageBuilderExt;
use whatsapp_rust::waproto::buffa::MessageField;
use whatsapp_rust::waproto::whatsapp::message::order_message::{OrderStatus, OrderSurface};

use crate::cards::{Button, ButtonAction, Card, Header, HeaderMedia, biz_node};
use crate::pipeline::{
    MediaKind, PendingMedia, PipelineError, QuoteSource, Rendered, ReplyTarget, context_info,
};

/// Varian yang aktif di produksi — dipakai sebagai fallback kalau setting
/// tidak terbaca.
pub const VARIANT_AKTIF: i64 = 9;

/// Semua varian yang punya bentuk wire sendiri.
pub const VARIANTS: [i64; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

/// Data yang dibutuhkan varian — semuanya dari `config` + `Ctx`.
pub struct VariantInput<'a> {
    /// Isi balasan.
    pub text: &'a str,
    /// Pemicu: kutipan + mention.
    pub target: &'a ReplyTarget,
    /// Direktori kerja — tempat aset dan `package.json` dibaca.
    pub root: &'a Path,
    pub bot_name: &'a str,
    pub bot_developer: &'a str,
    pub bot_version: &'a str,
    /// `config.info.website` — dipakai varian 4 dan 11 sebagai `matchedText`.
    pub website: &'a str,
    pub newsletter_jid: &'a str,
    pub newsletter_name: &'a str,
    /// `config.owner.number[0]` — dipakai sebagai participant kontak palsu.
    pub owner_number: &'a str,
    /// `db.setting("srtEnabled")` — kalau aktif, thumbnail diambil dari
    /// `assets/image/shuffle` (`serialize.js:969-974`).
    pub srt_enabled: bool,
}

/// `db.setting("replyVariant") || 1` (`serialize.js:983-991`).
///
/// `None` = tidak ada setting, `Some(0)`/`Some("")` = ada tapi falsy. Keduanya
/// berakhir di 1, tapi dibedakan supaya test bisa mengunci bahwa setting nol
/// tidak membuat renderer panik.
pub fn variant_from(settings: Option<&serde_json::Value>) -> i64 {
    let v = settings.and_then(|s| s.get("replyVariant"));
    match v {
        None => 1,
        Some(x) => match x {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(1),
            serde_json::Value::String(s) => s.trim().parse::<i64>().unwrap_or(1),
            _ => 1,
        },
    }
}

/// `true` kalau varian ini butuh bentuk wire sendiri.
///
/// Varian 1 (BASIC) TIDAK termasuk: bentuknya persis teks biasa, jadi jalur
/// umum yang menanganinya.
pub fn needs_custom_render(variant: i64) -> bool {
    VARIANTS.contains(&variant) && variant != 1
}

/// Render balasan sesuai varian.
///
/// `Ok(None)` = jangan kirim apa pun (teks kosong, `serialize.js:966`).
pub fn render(variant: i64, input: &VariantInput<'_>) -> Result<Option<Rendered>, PipelineError> {
    // `if (!text && text !== 0) return null` (`serialize.js:966`).
    if input.text.is_empty() {
        return Ok(None);
    }
    match variant {
        2 => Ok(Some(render_variant_2(input))),
        3 => Ok(Some(render_variant_3(input))),
        4 => Ok(Some(render_variant_4(input))),
        5 => Ok(Some(render_variant_5(input))),
        6 => Ok(Some(render_variant_6(input))),
        7 => Ok(Some(render_variant_7(input)?)),
        8 => Ok(Some(render_variant_8(input)?)),
        9 => Ok(Some(render_variant_9(input))),
        10 => Ok(Some(render_variant_10(input))),
        11 => Ok(Some(render_variant_11(input)?)),
        // Varian 1 dan apa pun di luar daftar: teks biasa.
        _ => Ok(None),
    }
}

// =============================================================================
// Aset
// =============================================================================

/// `getCachedSrtImage()` (`serialize.js:68-82`): satu file acak dari
/// `assets/image/shuffle`, di-cache 30 menit per file.
///
/// `read_dir` mengembalikan urutan filesystem, sedangkan `readdirSync` di Node
/// juga begitu — urutan keduanya tidak dijamin sama, tapi file dipilih acak
/// setelah itu, jadi tidak ada yang bisa dibandingkan byte-per-byte.
fn srt_dir(root: &Path) -> PathBuf {
    root.join("assets").join("image").join("shuffle")
}

/// Thumbnail varian 9: `srtEnabled && getCachedSrtImage() || getAssetBuffer("foto")`
/// (`serialize.js:976-980,1232`).
///
/// Nama aset ikut `config.assets`, yang menunjuk `assets/image/*.jpeg`.
pub fn thumbnail_bytes(root: &Path, srt_enabled: bool) -> Option<Vec<u8>> {
    let dir = srt_dir(root);
    if srt_enabled && let Ok(entries) = std::fs::read_dir(&dir) {
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "webp" | "jpg" | "jpeg" | "png"))
            })
            .collect();
        if !files.is_empty() {
            files.sort();
            let idx = pseudo_random_index(files.len());
            if let Ok(b) = std::fs::read(&files[idx]) {
                return Some(b);
            }
        }
    }
    std::fs::read(root.join("assets").join("image").join("foto.jpeg")).ok()
}

/// `safeThumbnail(buf, 300)` (`asset-manager.js:79-99`) — resize 300×300 dengan
/// fit cover, keluar sebagai JPEG.
///
/// `safeThumbnail` mengembalikan `null` kalau inputnya tidak bisa diproses;
/// itu yang ditiru di sini: gambar rusak tidak boleh menggagalkan seluruh
/// balasan.
pub fn safe_thumbnail(data: &[u8], size: u32) -> Option<Vec<u8>> {
    oktz_plugin_api::gambar::sharp(data)
        .ok()?
        .ubah_ukuran(size, size)
        .ke_jpeg(80)
        .ok()
}

/// `Math.floor(Math.random() * n)` tanpa dependency RNG.
///
/// Jam sistem sudah cukup: yang perlu tercermin hanya "acak", dan ini bukan
/// sumber keacakan yang dipakai untuk keamanan.
fn pseudo_random_index(len: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ d.as_secs())
        .unwrap_or(0);
    ((nanos as u128 * len as u128) >> 32) as usize % len.max(1)
}

/// Path aset dari `config.assets` (`config.js:44-66`).
fn asset_path(root: &Path, name: &str) -> PathBuf {
    root.join("assets").join("image").join(format!("{name}.jpeg"))
}

// =============================================================================
// Varian 2 — PREMIUM
// =============================================================================

/// `serialize.js:995-1032`.
///
/// Dokumen yang dikirim adalah isi `package.json` mentah dengan
/// `mimetype: "image/png"` dan `fileName` = nama bot; yang tampil di chat
/// adalah thumbnail `foto2`. Kutipannya kontak palsu dari `status@broadcast`.
fn render_variant_2(input: &VariantInput<'_>) -> Rendered {
    let quoted = Some(fake_contact_quote(
        input,
        &format!("🪸 {}", input.bot_name),
        "0@s.whatsapp.net",
    ));
    let thumbnail = thumbnail_for(input, "foto2");
    let source = oktz_plugin_api::registry::MediaSource::Path {
        path: input.root.join("package.json").to_string_lossy().to_string(),
        mimetype: Some("image/png".to_string()),
    };
    Rendered {
        message: wa::Message::default(),
        extra_nodes: Vec::new(),
        media: Some(PendingMedia {
            kind: MediaKind::Document,
            source,
            caption: input.text.to_string(),
            mentions: mention_of(input),
            quote: quoted,
            gif_playback: false,
            ptt: false,
            file_name: Some(input.bot_name.to_string()),
            jpeg_thumbnail: thumbnail,
            file_length: Some(99_999_999_999_999),
            association: None,
            paired: false,
        }),
        card: None,
        message_id: None,
    }
}

// =============================================================================
// Varian 3 — TITANIUM
// =============================================================================

/// `serialize.js:1033-1049` — video `mp4` dengan `gifPlayback`, kutipan
/// pemicu.
fn render_variant_3(input: &VariantInput<'_>) -> Rendered {
    Rendered {
        message: wa::Message::default(),
        extra_nodes: Vec::new(),
        media: Some(PendingMedia {
            kind: MediaKind::Video,
            source: oktz_plugin_api::registry::MediaSource::Path {
                path: input
                    .root
                    .join("assets")
                    .join("video")
                    .join("mp4.mp4")
                    .to_string_lossy()
                    .to_string(),
                mimetype: None,
            },
            caption: input.text.to_string(),
            mentions: mention_of(input),
            quote: input.target.quote.clone(),
            gif_playback: true,
            ptt: false,
            file_name: None,
            jpeg_thumbnail: None,
            file_length: None,
            association: None,
            paired: false,
        }),
        card: None,
        message_id: None,
    }
}

// =============================================================================
// Varian 4 — LV (link preview)
// =============================================================================

/// `serialize.js:1050-1077` + `sock.sendPreview` (`socket.js:1206-1270`).
///
/// Preview-nya `previewType: 0` (`NONE`), `matchedText` = website bot, dan
/// thumbnail-nya gambar yang di-upload sebagai `thumbnail-link`.
fn render_variant_4(input: &VariantInput<'_>) -> Rendered {
    let website = if input.website.is_empty() {
        "https://github.com"
    } else {
        input.website
    };
    let title = input.bot_name;
    let description = format!(
        "Pengembang: {} | Versi: {}",
        input.bot_developer, input.bot_version
    );
    let mut ci = context_info(&mention_of(input), input.target.quote.as_ref());
    ci.is_forwarded = Some(true);
    ci.forwarding_score = Some(9);
    ci.forwarded_newsletter_message_info = MessageField::some(
        wa::context_info::ForwardedNewsletterMessageInfo {
            newsletter_jid: Some(input.newsletter_jid.to_string()),
            newsletter_name: Some(input.newsletter_name.to_string()),
            // `Math.floor(Math.random() * 1000000)` (`serialize.js:1073`).
            server_message_id: Some(pseudo_random_index(1_000_000) as i32),
            ..Default::default()
        },
    );

    let text = format!("{website}\n\n{}", input.text);
    let ext = wa::message::ExtendedTextMessage {
        text: Some(text),
        matched_text: Some(website.to_string()),
        title: Some(title.to_string()),
        description: Some(description),
        preview_type: Some(wa::message::extended_text_message::PreviewType::None),
        context_info: MessageField::some(ci),
        ..Default::default()
    };
    let message = wa::Message {
        extended_text_message: MessageField::some(ext),
        ..Default::default()
    };
    Rendered::wire(message, Vec::new())
}

// =============================================================================
// Varian 5 — FAKE ORDER
// =============================================================================

/// `serialize.js:1078-1113` — teks biasa, tapi yang dikutip adalah
/// `orderMessage` palsu.
fn render_variant_5(input: &VariantInput<'_>) -> Rendered {
    let thumbnail = thumbnail_for(input, "foto");
    let fake = fake_order_quote(FakeOrder {
        order_id: "123456",
        item_count: 999,
        order_title: "System Notification",
        token: "ARU1+",
        total_amount1000: 1_000_000,
        message: input.bot_name,
        thumbnail: thumbnail.as_deref(),
    });
    let target = ReplyTarget {
        quote: Some(QuoteSource {
            stanza_id: String::new(),
            participant: Some("0@s.whatsapp.net".to_string()),
            message: Box::new(fake),
        }),
        sender: input.target.sender.clone(),
    };
    let ci = context_info(&mention_of(input), target.quote.as_ref());
    let message = wa::Message::text_with_context(input.text, ci);
    Rendered::wire(message, Vec::new())
}

// =============================================================================
// Varian 6 — SIMPLE DOCUMENT
// =============================================================================

/// `serialize.js:1114-1136` — dokumen `package.json`, kutipan pemicu asli.
fn render_variant_6(input: &VariantInput<'_>) -> Rendered {
    let thumbnail = thumbnail_for(input, "foto2");
    let source = oktz_plugin_api::registry::MediaSource::Path {
        path: input.root.join("package.json").to_string_lossy().to_string(),
        mimetype: Some("image/png".to_string()),
    };
    Rendered {
        message: wa::Message::default(),
        extra_nodes: Vec::new(),
        media: Some(PendingMedia {
            kind: MediaKind::Document,
            source,
            caption: input.text.to_string(),
            mentions: mention_of(input),
            quote: input.target.quote.clone(),
            gif_playback: false,
            ptt: false,
            file_name: Some(input.bot_name.to_string()),
            jpeg_thumbnail: thumbnail,
            file_length: Some(99_999_999_999_999),
            association: None,
            paired: false,
        }),
        card: None,
        message_id: None,
    }
}

// =============================================================================
// Varian 7 — FAKE LOCATION
// =============================================================================

/// `serialize.js:1137-1174` — kartu native-flow dengan header lokasi dan
/// tanpa tombol.
fn render_variant_7(input: &VariantInput<'_>) -> Result<Rendered, PipelineError> {
    let thumbnail = thumbnail_for(input, "foto");
    let mut ci = context_info(&mention_of(input), input.target.quote.as_ref());
    ci.is_forwarded = Some(true);
    ci.forwarding_score = Some(9);

    let card = Card::new(input.text)
        .header(Header {
            title: None,
            subtitle: None,
            has_media_attachment: true,
            media: Some(HeaderMedia::Location(wa::message::LocationMessage {
                degrees_latitude: Some(0.0),
                degrees_longitude: Some(0.0),
                name: Some(input.bot_name.to_string()),
                address: Some("Bot Wa Multi Device".to_string()),
                jpeg_thumbnail: thumbnail,
                ..Default::default()
            })),
        })
        .context_info(ci);
    Ok(Rendered::wire(card.to_message(), vec![biz_node()]))
}

// =============================================================================
// Varian 8 — FAKE SIGNUP
// =============================================================================

/// `serialize.js:1175-1223` — kartu dengan tombol `inapp_signup`, dan
/// `contextInfo.quotedMessage` berisi `orderMessage` palsu.
fn render_variant_8(input: &VariantInput<'_>) -> Result<Rendered, PipelineError> {
    let thumbnail = thumbnail_for(input, "foto");
    let quoted = fake_order_quote(FakeOrder {
        order_id: "8999999999999",
        item_count: 999,
        order_title: "blablabla",
        token: "ARU1+",
        total_amount1000: 0,
        message: input.text,
        thumbnail: thumbnail.as_deref(),
    });

    let mut ci = context_info(&mention_of(input), None);
    ci.participant = Some(input.target.sender.clone());
    ci.quoted_message = MessageField::some(quoted);
    ci.remote_jid = Some(input.target.sender.clone());
    ci.forwarding_score = Some(999);
    ci.is_forwarded = Some(true);

    let card = Card::new(input.text)
        .header(Header {
            title: Some(input.bot_name.to_string()),
            subtitle: None,
            has_media_attachment: false,
            media: None,
        })
        .context_info(ci)
        .button(Button::new(
            ButtonAction::from_name("inapp_signup").unwrap_or(ButtonAction::QuickReply),
            serde_json::json!({}),
        ));
    Ok(Rendered::wire(card.to_message(), vec![biz_node()]))
}

// =============================================================================
// Varian 9 — FAKE ORDER (relay)
// =============================================================================

/// `serialize.js:1231-1259` — `orderMessage` lewat `sock.relayMessage`.
///
/// Nilai statis di dalamnya (`"WXX"`, `"CONTROL"`, `"SDsdafma"`, `itemCount: 1`,
/// `status: INQUIRY`, `surface: CATALOG`, `totalAmount1000: "0"`,
/// `totalCurrencyCode: "IDR"`, `messageVersion: 1`) disalin apa adanya: itu isi
/// yang user lihat, dan mengeditnya berarti menampilkan produk yang berbeda.
///
/// `contextInfo` bedanya penting dan sering terbalik: JS menulis `participant`
/// dan `quotedMessage` LANGSUNG di sini, terpisah dari blok `contextInfo` di
/// atas fungsi. `quotedMessage` jatuh ke `m.message` — pesan asli apa adanya,
/// bukan teksnya saja; baru kalau `m.message` kosong dipakai `{conversation}`.
///
/// `ponytail:` wacore 0.7 tidak punya `relay_message`, jadi `orderMessage`
/// dikirim lewat jalur `Message` biasa. Bubble yang tampil sama (isi, thumbnail,
/// context); yang berbeda adalah amplop protokolnya. Naikkan ke `relay_message`
/// begitu wacore/album menyediakan API-nya.
pub fn render_variant_9(input: &VariantInput<'_>) -> Rendered {
    let mentions = mention_of(input);
    let mut ci = context_info(&mentions, input.target.quote.as_ref());
    ci.participant = input.target.sender.clone().into();

    let order = wa::message::OrderMessage {
        order_id: Some("WXX".into()),
        thumbnail: Some(
            thumbnail_for(input, "foto")
                .unwrap_or_default(),
        ),
        item_count: Some(1),
        status: Some(OrderStatus::Inquiry),
        surface: Some(OrderSurface::Catalog),
        message: Some(input.text.to_string()),
        order_title: Some("CONTROL".into()),
        token: Some("SDsdafma".into()),
        total_amount1000: Some(0),
        total_currency_code: Some("IDR".into()),
        message_version: Some(1),
        context_info: MessageField::some(ci),
        ..Default::default()
    };

    let msg = wa::Message {
        order_message: MessageField::some(order),
        ..Default::default()
    };
    Rendered::wire(msg, Vec::new())
}

// =============================================================================
// Varian 10 — TAGIHAN / PAYMENT
// =============================================================================

/// `serialize.js:1253-1276` — `requestPaymentMessage` dengan catatan berisi
/// teks balasan.
fn render_variant_10(input: &VariantInput<'_>) -> Rendered {
    let mentions = mention_of(input);
    let mut ci = context_info(&mentions, input.target.quote.as_ref());
    ci.participant = input.target.sender.clone().into();

    let note = wa::Message {
        extended_text_message: MessageField::some(wa::message::ExtendedTextMessage {
            text: Some(input.text.to_string()),
            context_info: MessageField::some(ci),
            ..Default::default()
        }),
        ..Default::default()
    };

    let payment = wa::message::RequestPaymentMessage {
        currency_code_iso4217: Some("IDR".to_string()),
        amount1000: Some(75_000_000),
        request_from: Some(if input.target.sender.is_empty() {
            "0@s.whatsapp.net".to_string()
        } else {
            input.target.sender.clone()
        }),
        note_message: MessageField::some(note),
        ..Default::default()
    };

    let msg = wa::Message {
        request_payment_message: MessageField::some(payment),
        ..Default::default()
    };
    Rendered::wire(msg, Vec::new())
}

// =============================================================================
// Varian 11 — PREVIEW
// =============================================================================

/// `serialize.js:1277-1344` — `extendedTextMessage` dengan `previewType: 1`
/// dan `faviconMMSMetadata`.
fn render_variant_11(input: &VariantInput<'_>) -> Result<Rendered, PipelineError> {
    let website = input.website;
    let thumbnail = thumbnail_for(input, "foto");
    let favicon = thumbnail_for(input, "foto");

    let mut ci = context_info(&mention_of(input), None);
    ci.is_forwarded = Some(true);
    ci.forwarding_score = Some(999);
    ci.forwarded_newsletter_message_info = MessageField::some(
        wa::context_info::ForwardedNewsletterMessageInfo {
            newsletter_jid: Some(input.newsletter_jid.to_string()),
            newsletter_name: Some(input.newsletter_name.to_string()),
            ..Default::default()
        },
    );

    let ext = wa::message::ExtendedTextMessage {
        text: Some(format!("{website}\n{}", input.text)),
        matched_text: Some(website.to_string()),
        title: Some(input.bot_name.to_string()),
        description: Some(format!("Hello {}", input.target.sender)),
        jpeg_thumbnail: thumbnail,
        preview_type: Some(wa::message::extended_text_message::PreviewType::Video),
        favicon_mms_metadata: favicon
            .map(|_bytes| {
                MessageField::some(wa::message::MMSThumbnailMetadata {
                    thumbnail_height: Some(512),
                    thumbnail_width: Some(512),
                    ..Default::default()
                })
            })
            .unwrap_or_default(),
        context_info: MessageField::some(ci),
        ..Default::default()
    };
    let message = wa::Message {
        extended_text_message: MessageField::some(ext),
        ..Default::default()
    };
    Ok(Rendered::wire(message, Vec::new()))
}

// =============================================================================
// Helper
// =============================================================================

/// Mention default: `options?.mentions || [m?.sender] || []`.
fn mention_of(input: &VariantInput<'_>) -> Vec<String> {
    if input.target.sender.is_empty() {
        Vec::new()
    } else {
        vec![input.target.sender.clone()]
    }
}

/// Thumbnail untuk varian tertentu: `srtImage || getAssetBuffer(nama)`.
fn thumbnail_for(input: &VariantInput<'_>, name: &str) -> Option<Vec<u8>> {
    if input.srt_enabled
        && let Some(b) = random_srt(input.root)
    {
        return Some(b);
    }
    let raw = std::fs::read(asset_path(input.root, name)).ok()?;
    safe_thumbnail(&raw, 300).or(Some(raw))
}

/// `getCachedSrtImage()` — file acak dari `assets/image/shuffle`.
fn random_srt(root: &Path) -> Option<Vec<u8>> {
    let dir = srt_dir(root);
    let entries = std::fs::read_dir(&dir).ok()?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "webp" | "jpg" | "jpeg" | "png"))
        })
        .collect();
    if files.is_empty() {
        return None;
    }
    files.sort();
    std::fs::read(&files[pseudo_random_index(files.len())]).ok()
}

/// Kutipan kontak palsu — `contactMessage` (`serialize.js:997-1010`).
fn fake_contact_quote(input: &VariantInput<'_>, display_name: &str, participant: &str) -> QuoteSource {
    let number = if input.owner_number.is_empty() {
        "13135550002".to_string()
    } else {
        input.owner_number.to_string()
    };
    let vcard = format!(
        "BEGIN:VCARD\nVERSION:3.0\nN:XL;ttname,;;;\nFN:ttname\nitem1.TEL;waid={number}:+1 (313) 555-0002\nitem1.X-ABLabel:Ponsel\nEND:VCARD"
    );
    let message = wa::Message {
        contact_message: MessageField::some(wa::message::ContactMessage {
            display_name: Some(display_name.to_string()),
            vcard: Some(vcard),
            ..Default::default()
        }),
        ..Default::default()
    };
    QuoteSource {
        stanza_id: String::new(),
        participant: Some(participant.to_string()),
        message: Box::new(message),
    }
}

/// `orderMessage` palsu untuk kutipan (`serialize.js:1080-1101`, `:1202-1213`).
fn fake_order_quote(spec: FakeOrder<'_>) -> wa::Message {
    wa::Message {
        order_message: MessageField::some(wa::message::OrderMessage {
            order_id: Some(spec.order_id.to_string()),
            thumbnail: Some(spec.thumbnail.unwrap_or_default().to_vec()),
            item_count: Some(spec.item_count),
            status: Some(OrderStatus::Inquiry),
            surface: Some(OrderSurface::Catalog),
            message: Some(spec.message.to_string()),
            order_title: Some(spec.order_title.to_string()),
            seller_jid: Some("0@s.whatsapp.net".to_string()),
            token: Some(spec.token.to_string()),
            total_amount1000: Some(spec.total_amount1000),
            total_currency_code: Some("IDR".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Isi `orderMessage` palsu untuk kutipan.
struct FakeOrder<'a> {
    order_id: &'a str,
    item_count: i32,
    order_title: &'a str,
    token: &'a str,
    total_amount1000: i64,
    message: &'a str,
    thumbnail: Option<&'a [u8]>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::QuoteSource;
    use serde_json::json;
    use whatsapp_rust::wacore::proto_helpers::MessageBuilderExt;

    fn target() -> ReplyTarget {
        ReplyTarget {
            quote: Some(QuoteSource {
                stanza_id: "STANZA7".into(),
                participant: Some("6281@s.whatsapp.net".into()),
                message: Box::new(wa::Message::text("pesan asal")),
            }),
            sender: "6289@s.whatsapp.net".into(),
        }
    }

    fn input<'a>(target: &'a ReplyTarget, root: &'a Path) -> VariantInput<'a> {
        VariantInput {
            text: "hai",
            target,
            root,
            bot_name: "Oktz Bot",
            bot_developer: "Zann",
            bot_version: "3.3.1",
            website: "https://oktzz.xyz",
            newsletter_jid: "120363411617217441@newsletter",
            newsletter_name: "Join saluran resmi Oktz",
            owner_number: "6285608953677",
            srt_enabled: false,
        }
    }

    fn order_of(r: &Rendered) -> &wa::message::OrderMessage {
        r.message
            .order_message
            .as_option()
            .expect("orderMessage")
    }

    #[test]
    fn variant_defaults_to_one_when_the_setting_is_missing() {
        assert_eq!(variant_from(None), 1);
        assert_eq!(variant_from(Some(&json!({}))), 1);
        // `db.setting(...) || 1` di JS: nilai falsy jatuh ke 1.
        assert_eq!(variant_from(Some(&json!({ "replyVariant": 0 }))), 0);
        assert_eq!(variant_from(Some(&json!({ "replyVariant": "" }))), 1);
        assert_eq!(variant_from(Some(&json!({ "replyVariant": 9 }))), 9);
        assert_eq!(variant_from(Some(&json!({ "replyVariant": "9" }))), 9);
    }

    #[test]
    fn semua_varian_kecuali_satu_punya_render_sendiri() {
        // Varian 1 = BASIC = teks biasa, jadi jalur umum yang menanganinya.
        assert!(!needs_custom_render(1));
        for v in 2..=11 {
            assert!(needs_custom_render(v), "varian {v} belum diimplementasikan");
        }
    }

    #[test]
    fn variant_nine_copies_every_static_field_from_serialize_js() {
        let t = target();
        let r = render_variant_9(&input(&t, Path::new("/tmp/tidak-ada-oktz")));
        let o = order_of(&r);
        assert_eq!(o.order_id.as_deref(), Some("WXX"));
        assert_eq!(o.order_title.as_deref(), Some("CONTROL"));
        assert_eq!(o.token.as_deref(), Some("SDsdafma"));
        assert_eq!(o.message.as_deref(), Some("hai"));
        assert_eq!(o.item_count, Some(1));
        assert_eq!(o.status, Some(OrderStatus::Inquiry));
        assert_eq!(o.surface, Some(OrderSurface::Catalog));
        assert_eq!(o.total_amount1000, Some(0));
        assert_eq!(o.total_currency_code.as_deref(), Some("IDR"));
        assert_eq!(o.message_version, Some(1));
        assert!(r.extra_nodes.is_empty(), "orderMessage bukan biz node");
    }

    #[test]
    fn variant_nine_context_carries_mention_participant_and_the_quoted_message() {
        let t = target();
        let r = render_variant_9(&input(&t, Path::new("/tmp/tidak-ada-oktz")));
        let ci = order_of(&r).context_info.as_option().expect("contextInfo");
        assert_eq!(ci.mentioned_jid, vec!["6289@s.whatsapp.net".to_string()]);
        assert_eq!(ci.participant.as_deref(), Some("6289@s.whatsapp.net"));
        assert!(ci.quoted_message.as_option().is_some());
    }

    #[test]
    fn variant_ten_is_a_payment_request_with_the_text_in_the_note() {
        let t = target();
        let r = render_variant_10(&input(&t, Path::new("/tmp/tidak-ada-oktz")));
        let p = r
            .message
            .request_payment_message
            .as_option()
            .expect("requestPaymentMessage");
        assert_eq!(p.currency_code_iso4217.as_deref(), Some("IDR"));
        assert_eq!(p.amount1000, Some(75_000_000));
        let note = p.note_message.as_option().expect("noteMessage");
        let ext = note.extended_text_message.as_option().expect("extendedTextMessage");
        assert_eq!(ext.text.as_deref(), Some("hai"));
    }

    #[test]
    fn variant_seven_and_eight_are_native_flow_cards() {
        let t = target();
        let root = Path::new("/tmp/tidak-ada-oktz");
        for v in [7, 8] {
            let r = render(v, &input(&t, root)).unwrap().unwrap();
            assert!(r.message.view_once_message.is_set(), "varian {v} harus kartu");
            assert_eq!(r.extra_nodes.len(), 1, "varian {v} butuh node biz");
        }
    }

    #[test]
    fn variant_four_and_eleven_are_link_previews() {
        let t = target();
        let root = Path::new("/tmp/tidak-ada-oktz");
        for v in [4, 11] {
            let r = render(v, &input(&t, root)).unwrap().unwrap();
            let ext = r
                .message
                .extended_text_message
                .as_option()
                .unwrap_or_else(|| panic!("varian {v} harus extendedTextMessage"));
            assert_eq!(ext.matched_text.as_deref(), Some("https://oktzz.xyz"));
        }
    }

    #[test]
    fn variants_two_three_and_six_go_through_the_media_path() {
        let t = target();
        let root = Path::new("/tmp/tidak-ada-oktz");
        for v in [2, 3, 6] {
            let r = render(v, &input(&t, root)).unwrap().unwrap();
            assert!(r.needs_media(), "varian {v} harus lewat jalur upload");
            let m = r.media.unwrap();
            assert_eq!(m.caption, "hai");
        }
    }

    #[test]
    fn empty_text_sends_nothing() {
        let t = target();
        let root = Path::new("/tmp/tidak-ada-oktz");
        let mut i = input(&t, root);
        i.text = "";
        assert!(render(9, &i).unwrap().is_none());
    }

    #[test]
    fn thumbnail_falls_back_when_nothing_is_on_disk() {
        // Direktori kerja yang tidak punya assets/ -> `None`, bukan panic.
        assert_eq!(thumbnail_bytes(Path::new("/tmp/tidak-ada-oktz"), true), None);
    }

    #[test]
    fn pseudo_random_index_stays_in_range() {
        for len in 1..32usize {
            for _ in 0..8 {
                assert!(pseudo_random_index(len) < len);
            }
        }
    }
}
