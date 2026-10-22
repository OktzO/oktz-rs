# Port "tampilan sama persis" dari bot JS (oktz-md) ke bot Rust (oktz-rs)

Referensi: `/workspaces/oktz-rs/oktz-md` (JS, onigis/Baileys fork)
Target: `/workspaces/oktz-rs` (Rust, whatsapp-rust 0.7 / wacore)

Status: **selesai** — build ✅ · clippy ✅ · 6787 test ✅

## A. Pustaka

- [A1] `oktz-plugin-api/registry.rs` — model kartu: `CardHeader`, `CardMedia`,
      `CardLocation`, `CardContext`, `CardQuote`, `CardList`/`CardSection`/`CardRow`,
      `NewsletterInfo`; `Reply::Interactive` dikasih `header`, `message_params`,
      `context`; `Reply::Table`; `Reply::PairedVideo`; builder
      `with_header/with_message_params/with_context/with_buttons/with_header_text`;
      `InteractiveButton::{quick_reply,url,url_merchant,select,empty}`;
      `Reply::is_text_like()`.
- [A2] `oktz-wa/cards.rs` — `HeaderMedia` (image/video/location), `Header.media`,
      `List.icon`, encoder `to_message()` mengisi oneof `media` header,
      `ButtonAction::InAppSignup`.
- [A3] `oktz-wa/pipeline.rs` — `PendingCard`, `build_card_proto`,
      `build_card_context`, `card_quote_message`, `build_quote_message`,
      `build_location_message`, `MessageSender::send_card` +
      `ClientSender::send_card` (upload header media & stiker fake-quote),
      `Rendered.card` + `Rendered.message_id`, `SendOptions` pada trait
      `send`/`send_media`/`send_card`, `messageAssociation` (`PairedVideo`).
- [A4] `oktz-wa/reply_variant.rs` — 11 varian `replyVariant` (JS
      `serialize.js:958-1357`).
- [A5] `oktz-wa/tables.rs` — sistem tabel: `Message.botForwardedMessage` →
      `richResponseMessage` (`AIRichResponseMessage`).

## B. Plugin

- [B1] `plugin/src/main/menu.rs` — 8 varian menu (JS `plugins/main/menu.js`).
- [B2] `plugin/src/main/menucat.rs` — varian menucat 1/2/5/6.
- [B3] `plugin/src/main/allmenu.rs` — varian allmenu 1/2/5/6 + audio menu.
- [B4] `plugin/src/main/totalfitur.rs` — memakai sistem tabel.
- [B5] Alias plugin JS → Rust (audit 817 plugin JS): 11 alias yang hilang
      ditambahkan; `.tiktok`/`.tt`/`.ttmp4` dan `.ttmp3`/`.ttmusic`/
      `.tiktokmusic` kini ada.
- [B6] `plugin/src/main/sc.rs` — kartu ikut mengutip pesan pemicu (`quoted: m`).
- [B7] `plugin/src/download/tiktok.rs` + `ttmp3.rs` — command TikTok dengan
      tombol "🎵 Ambil Music" (video jadi header kartu native-flow).

## C. Verifikasi

- [C1] `cargo build --release` — ✅
- [C2] `cargo test --release` — ✅ 6787 passed, 0 failed
- [C3] `cargo clippy --release --all-targets` — ✅ 0 warning

## Deviasi yang diketahui (semua tercatat di doc-comment plugin terkait)

- Riwayat command teratas (`database/main/stats.json`) tidak ada di Rust, jadi
  blok "History" varian 6 tidak ditulis.
- `config.mode` global tidak ada di `Ctx`; baris "Mode" memakai `PUBLIC`.
- Registry "case" (`case/foto.js`) tidak ada di Rust, jadi penghitung
  `(X plugin + Y case)` tidak pernah ditulis.
- `resolveTikTokUrl` (short link `vt.tiktok.com`) tidak bisa mengambil URL
  akhir karena klien HTTP Rust mengikuti redirect secara internal tanpa
  communicasikannya ke pemanggil; short link diteruskan apa adanya.
- `formatDate` versi JS memakai `toLocaleDateString` locale Inggris; scraper
  memakai format ISO-like karena tidak ada ICU di Rust.
