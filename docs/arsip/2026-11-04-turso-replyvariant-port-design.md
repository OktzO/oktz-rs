# Design: Port penuh plugin + Turso + replyVariant untuk error

Tanggal: 2026-11-04
Status: disetujui (percakapan 2026-11-04)

## Intent

Melengkapi port JS (oktz-md) → Rust (oktz):
1. Session WhatsApp & data lowdb tersimpan di Turso (sekarang lokal-only, hapus = hilang).
2. Group settings (antilinkgc per grup, welcome per grup) & data RPG (koin, exp, level) tersimpan di Turso.
3. Plugin stub (mis. `.toimg`) di-port penuh — infra media sudah ada (`media_fetch.rs`).
4. Pesan error ikut setting `replyVariant` (bukan acak).

## Keputusan (disetujui user)

- Turso client: **crate resmi `libsql`** (async, tokio).
- Mirror: **blob per store**, model persis JS — tabel `stores` (users, groups, settings, premium, owner, partner, dst sebagai JSON utuh) + `session_creds` + `session_keys`. Kompatibel dengan data Turso lama dari JS.
- Session Rust (SQLite diesel, tabelnya beda dari JS): mirror tabel utama per-baris ke Turso (creds, prekeys, identities, sessions, sender_keys, app_state_keys), restore saat boot, **lokal tetap sumber utama**, fallback file kalau Turso down (model `useDurableAuthState`).
- Error → replyVariant: semua teks error yang terkirim ke user lewat `reply_variant::variant_from(settings)` + `render` yang sama seperti balasan sukses. Tanpa random.
- Port stub bertahap: Batch 1 = `.toimg` + stub yang tinggal colok `media_fetch` + `Reply::image` (keluarga ai-image, dll). Batch 2 = sisanya sesuai penyebab masing-masing.

## Model perilaku (dari JS, oktz-md/src/lib/database.js + turso-session.js)

- Write-behind: `markDirty(key)` → flush timer (min 15s), flush-all saat shutdown.
- Boot: load dari Turso kalau lokal kosong; kalau Turso kosong → seed dari lokal.
- Turso down → warning + tulis file lokal (bukan error fatal).
- Session: tulis lokal + mirror Turso best-effort; baca lokal dulu, Turso hanya untuk id yang hilang; creds direcover dari Turso hanya kalau lokal unpaired.

## Non-goals

- Skema relasional per-baris untuk users/groups.
- `m.download(filename)` ke storage/temp.
- Retry re-encrypt `updateMediaMessage` (CDN 400/410) — tetap terblokir, lihat docs/rust-rewrite/BLOCKED.md.

## Yang tersisa ke builder

- Pilih titik sambung flush timer & boot-load di `bot.rs`/`pipeline.rs` (mutasi `SetUser/SetGroup/SetSetting` → markDirty).
- Daftar stub Batch 1 diverifikasi per file (penyebab stub bisa beda-beda).
- Tes: self-check untuk turso store (kalau memungkinkan pakai libsql lokal/file://), assert per plugin port.
