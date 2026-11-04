# Turso Mirror + replyVariant Error + Port Stub Media — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Data (session, group settings, RPG) tersimpan di Turso, pesan error ikut `replyVariant`, dan stub media (`.toimg` dll) jadi plugin penuh.

**Architecture:** (1) `oktz-store::turso` jadi klien libsql nyata dengan model blob-per-store persis JS (`stores`, `session_creds`, `session_keys` + mirror tabel session SQLite), write-behind 15s, boot-load/restore, lokal tetap sumber utama. (2) Jalur error pipeline dirender lewat `render_with_reply_variant` yang sama dengan balasan sukses. (3) Stub yang terblokir media/`sock` di-port pakai `media_fetch` + perluasan trait `Sock`.

**Tech Stack:** crate `libsql` (fitur `remote` — pure HTTP, tanpa C), tokio, diesel sqlite yang sudah ada.

**Spec:** `docs/superpowers/specs/2026-11-04-turso-replyvariant-port-design.md`

## Global Constraints

- Lokal SELALU sumber utama; Turso = mirror best-effort. Turso mati = warning log, JANGAN pernah error fatal / jangan balas gagal.
- Skema Turso mengikuti JS: `stores(key TEXT PRIMARY KEY, data TEXT, updated_at INTEGER)`, `session_creds(scope TEXT PRIMARY KEY, creds TEXT, updated_at INTEGER)`, `session_keys(scope, category, id, data, updated_at, PRIMARY KEY(scope,category,id))` — kompatibel dengan DB Turso lama oktz-md.
- Store names = JS: `users`, `groups`, `settings`, `premium`, `owner`, `partner` (blob JSON utuh, `lowdb::stringify` byte-identik).
- Semua penamaan ikut gaya repo (Indonesia: `muat`, `simpan`, `tandai_kotor`).
- `cargo test` di root harus hijau; `panic = "unwind"` jangan diubah.
- Ponytail marker untuk penyederhanaan sadar + upgrade path-nya.

## Review Focus

1. **Turso down saat flush** — bot harus tetap jalan, tulis file lokal tetap terjadi. Test: Task 1 (mock error) & Task 2 (flush tak pernah me-return error).
2. **Boot dengan Turso kosong tapi lokal terisi** — lokal JANGAN ditimpa; sebaliknya seed Turso. Test: Task 2.
3. **Boot dengan lokal kosong & Turso terisi** — restore ke lokal. Test: Task 2/3.
4. **`replyVariant` non-teks** — error yang dirender varian hanya untuk teks; kegagalan render varian error tidak boleh mematikan pipeline. Test: Task 5.
5. **`.toimg` dengan sticker animasi** — JS mengirim video/gif untuk sticker animasi; minimal harus tetap jalan untuk sticker statis + pesan jelas. Test: Task 6.

---

### Task 1: Klien Turso nyata di `oktz-store`

**Files:**
- Modify: `lib/oktz-store/Cargo.toml` (tambah `libsql = { version = "0.9", default-features = false, features = ["core", "remote"] }`)
- Modify: `lib/oktz-store/turso.rs`

**Interfaces:**
- Produces (dipakai Task 2–3):
  - `TursoStore::connect(&self) -> Result<libsql::Database, Error>`
  - `TursoStore::ensure_tables(&self, conn: &libsql::Connection) -> Result<(), Error>` (buat 3 tabel)
  - `TursoStore::load_store(&self, conn: &libsql::Connection, key: &str) -> Result<Option<serde_json::Value>, Error>` (baca 1 row `stores`)
  - `TursoStore::put_store(&self, conn: &libsql::Connection, key: &str, v: &serde_json::Value) -> Result<(), Error>` (UPSERT `INSERT INTO stores ... ON CONFLICT(key) DO UPDATE`)
  - `TursoStore::has_any_store(&self, conn: &libsql::Connection) -> Result<bool, Error>`

- [ ] **Step 1: Test gagal dulu** — di `turso.rs` `mod tests`: pakai `libsql::Builder::new_local(":memory:")` (test tak butuh jaringan):
  `ensure_tables` lalu `put_store(users)` → `load_store("users")` mengembalikan nilai sama; `load_store("ngawur")` → `Ok(None)`; `has_any_store` false→true.
- [ ] **Step 2:** `cargo test -p oktz-store` → FAIL (fungsi belum ada).
- [ ] **Step 3:** Implement 5 fungsi di atas dengan `libsql::Builder::new_remote(url, token)` di `connect`; semua `execute`/`query` async tapi `oktz-store` bebas tokio-runtime — kembalikan `Database` dan biarkan pemanggil (oktz-wa, sudah async) menjalankannya.
- [ ] **Step 4:** `cargo test -p oktz-store` → PASS.
- [ ] **Step 5:** Commit `feat(store): klien libsql nyata + tabel stores`.

### Task 2: Mirror lowdb (users/groups/settings/premium) → Turso

**Files:**
- Modify: `lib/oktz-wa/bot.rs` (wiring + flush task + boot load/seed)
- Modify: `lib/oktz-wa/pipeline.rs` (tandai kotor saat `apply_mutation` sukses)
- Modify: `src/main.rs` (bangun `TursoStore` dari `cfg.turso`, pass ke wiring)

**Interfaces:**
- Consumes: Task 1 semua fungsi; `PipelineWiring::from_settings(...)` (bot.rs:233); `apply_mutation` (pipeline.rs:436).
- Produces:
  - `PipelineWiring::with_turso(self, store: TursoStore) -> Self`
  - `PipelineWiring::flush_dirty_turso(&self)` — tulis store kotor (blocking di `block_in_place`), kotor = set oleh `StoreContext::apply` via flag `AtomicU64` bitmask per store.
  - `PipelineWiring::boot_sync_turso(&self)` — 3 cabang: Turso kosong + lokal terisi → seed Turso; Turso terisi + lokal kosong (settings.json tak ada) → load ke `JsonStore`; keduanya terisi → lokal menang (log info).
  - `PipelineWiring::spawn_flush_task(&self)` — `tokio::spawn` interval 15s + flush saat `tunggu_sinyal` selesai (dipanggil di `src/main.rs::run` sebelum select).

- [ ] **Step 1: Test gagal** — test integrasi di `bot.rs`: store lokal sementara dengan `users` terisi, `TursoStore` menunjuk `Builder::new_local(":memory:")` (buat konstruktor test `with_turso_db(libsql::Database)`): `boot_sync_turso` → Turso terisi; tulis mutasi `SetUser` via `apply_mutation` → flag kotor naik; `flush_dirty_turso` → `load_store("users")` memuat user baru.
- [ ] **Step 2:** `cargo test -p oktz-wa` → FAIL.
- [ ] **Step 3:** Implement: flag kotor di `StoreContext` (bit per store: users/groups/settings/lainnya), set di `apply` (pipeline.rs:~4248) HANYA jika `apply_mutation` `Ok`; `flush_dirty_turso` baca snapshot `JsonStore` via `read()`, stringify, `put_store`; kegagalan = `tracing::warn!`, flag kotor dipertahankan (coba lagi siklus berikutnya); `boot_sync_turso` sesuai 3 cabang di atas (bandingkan via `has_any_store` + `settings.json` ada/tidak).
- [ ] **Step 4:** `cargo test -p oktz-wa` → PASS; jalankan `cargo test` root.
- [ ] **Step 5:** Commit `feat(wa): mirror lowdb ke Turso (write-behind + boot sync)`.

### Task 3: Mirror session WhatsApp → Turso

**Files:**
- Modify: `lib/oktz-wa/bot.rs` (restore + mirror task)
- Modify: `lib/oktz-store/turso.rs` (helper row-copy generik)
- Modify: `src/main.rs` (urutan panggil)

**Interfaces:**
- Consumes: Task 1 `connect`; diesel `SqliteStore` (local file `database/wa/oktz-run.db`) tetap dibuat `prepare_store` seperti sekarang (main.rs:436).
- Produces:
  - `TursoStore::mirror_sqlite_tables(&self, conn: &libsql::Connection, local_path: &Path, tables: &[&str], scope: &str) -> Result<(), Error>` — buka `Builder::new_local(local_path)`, per tabel: `SELECT *`, baca nama kolom via `rows.column_types()`, buat tabel Turso sekali (skema generik `CREATE TABLE IF NOT EXISTS <t> (k INTEGER PRIMARY KEY, v TEXT)` — row diserialisasi JSON), UPSERT semua row.
  - `TursoStore::restore_sqlite_tables(&self, conn, local_path, tables, scope) -> Result<bool, Error>` — kebalikannya; `Ok(true)` = ada yang direstore.
  - `oktz_wa::SESSION_MIRROR_TABLES: &[&str]` — `["device", "identities", "sessions", "prekeys", "signed_prekeys", "sender_keys", "app_state_keys"]`.
- `ponytail:` mirror generik JSON-per-row (bukan skema kolom): sederhana & seluruh row selaras; upgrade ke kolom asli kalau ukuran `msg_secrets` bermasalah.

- [ ] **Step 1: Test gagal** — buat SQLite lokal sementara (`Builder::new_local(path)` + `CREATE TABLE identities (a TEXT, b BLOB)` + 2 row), mirror → cek Turso (`:memory:`) berisi 2 row; kosongkan lokal, restore → row kembali.
- [ ] **Step 2:** `cargo test -p oktz-store` → FAIL.
- [ ] **Step 3:** Implement (lihat Interfaces; BLOB → base64 di JSON).
- [ ] **Step 4:** PASS.
- [ ] **Step 5:** Commit `feat(store): mirror tabel session SQLite ⇄ Turso`.

### Task 4: Pasang mirror session di runtime

**Files:**
- Modify: `src/main.rs::run` (urutan: `prepare_store` → restore kalau file db baru/kosong → mirror task tiap 15s + flush sebelum exit)
- Modify: `lib/oktz-wa/bot.rs` (`spawn_session_mirror_task(&self, db_path)`)

**Interfaces:** Consumes Task 3. Dipanggil hanya `cfg.turso.enabled` (env `TURSO_URL`/`TURSO_AUTH_TOKEN` sudah dibaca `config.rs:440`).

- [ ] **Step 1: Test** — test kecil di `bot.rs`: `spawn_session_mirror_task` dengan db sementara & turso `:memory:`; tulis row ke sqlite; panggil flush manual; cek row muncul di turso.
- [ ] **Step 2:** FAIL → implement → PASS (`cargo test -p oktz-wa`).
- [ ] **Step 3:** Commit `feat(wa): sesi WhatsApp termirror ke Turso saat runtime`.

### Task 5: Pesan error ikut replyVariant

**Files:**
- Modify: `lib/oktz-wa/pipeline.rs` (arm `Err` di `Pipeline::run`, baris ~4350)
- Modify: `lib/oktz-core/config.rs` — verifikasi `error_template` Runtime (sudah ada, baris 258)

**Interfaces:**
- Consumes: `render_with_reply_variant` (pipeline.rs:3705), `config.error_template` (format JS `{prefix}/{command}/{pushName}`), `Outcome`, `caller` (chat/sender/push_name).
- Produces: `Pipeline::kirim_pesan_error(&self, chat, prefix, command, push_name, err: &PipelineError)` — buat teks dari `error_template` (substitusi placeholder; default template kalau kosong), `Reply::text`, render varian, deliver; kegagalan di dalam = `tracing::error!`, TIDAK pernah panic/loop.

- [ ] **Step 1: Test gagal** — test di `pipeline.rs` tests: stub sender yang gagal deliver reply pertama memicu `Err` → teks yang TERKIRIM berisi teks `error_template` + command; dengan `settings.replyVariant = 5` pesan error dirender sebagai wire `FakeOrderQuote` (cek `message.extended_text_message`/order ada, bukan teks polos).
- [ ] **Step 2:** FAIL.
- [ ] **Step 3:** Implement: di arm `Err` `run()` (run hanya tahu `m`): ekstrak command = kata pertama body, prefix `.` (dari `ParseOptions`/body), push_name dari caller → panggil `kirim_pesan_error`. Error saat KIRIM error dibiarkan jadi log (tanpa rekursi).
- [ ] **Step 4:** `cargo test -p oktz-wa` → PASS.
- [ ] **Step 5:** Commit `feat(wa): pesan error pipeline ikut replyVariant + error_template`.

### Task 6: `.toimg` penuh

**Files:**
- Modify: `plugin/src/tools/toimg.rs`

**Interfaces:**
- Consumes: `ctx.media_fetch.download(MediaOrigin::Quoted | Trigger)` (`media_fetch.rs:205`), `oktz_plugin_api::gambar::sharp` (webp→png), `Reply::image(MediaSource::bytes(..))`.
- Produces: `.toimg` berfungsi: quoted sticker ATAU sticker dengan caption (trigger) → unduh → decode webp → `ke_png()` → `Reply::image`.

- [ ] **Step 1: Test gagal** — ubah test `toimg.rs`: `MediaStub` mengembalikan byte PNG valid (fixture kecil di test); hasil `Reply::Image` dengan `MediaSource::Bytes` mimetype `image/png`; sticker quoted animasi → tetap kirim PNG (ponytail) dengan catatan modul diubah.
- [ ] **Step 2:** FAIL.
- [ ] **Step 3:** Implement: ganti `unavailable()` dengan jalur nyata; gagal unduh/decode → teks gagal dari JS (`Gagal membuat gambar...`), tanpa sticker → `no_sticker` tetap.
- [ ] **Step 4:** `cargo test -p oktz-plugins toimg` → PASS.
- [ ] **Step 5:** Commit `feat(toimg): port penuh sticker→gambar`.

### Task 7: `.rvo` + `.ptv` (view-once → terlihat)

**Files:**
- Modify: `plugin/src/tools/rvo.rs`, `plugin/src/tools/ptv.rs` (referensi JS: `oktz-md/plugins/tools/`)

**Interfaces:** Consumes sama dengan Task 6; `Reply::video(MediaSource::bytes(..))` untuk ptv.

- [ ] **Step 1: Test gagal** — quoted image (rvo) → `Reply::Image` bytes; quoted video (ptv) → `Reply::Video` bytes; tanpa media → teks guard JS asli.
- [ ] **Step 2:** FAIL → **Step 3:** implement → **Step 4:** PASS → **Step 5:** Commit `feat(tools): rvo & ptv penuh`.

### Task 8: Perluasan trait `Sock` (bio, nama, foto, blokir)

**Files:**
- Modify: `lib/oktz-plugin-api/sock.rs` (4 method baru + impl `SockMati`)
- Modify: `lib/oktz-wa/sock_client.rs` (impl nyata: `Client::set_status_text`, `set_push_name`, `set_profile_picture`, `remove_profile_picture`, `block`, `unblock` — semua ada di whatsapp-rust `features/profile.rs:55-137`, `features/blocking.rs:60-72`)

**Interfaces:**
- Produces: `Sock::set_bio(&self, teks: &str)`, `Sock::set_nama(&self, nama: &str)`, `Sock::set_foto_bot(&self, isi: &[u8])` (sudah ada `set_foto_profil`), `Sock::hapus_foto_bot(&self)`, `Sock::blokir(&self, jid: &str, blokir: bool)`.

- [ ] **Step 1: Test gagal** — `sock_mati_selalu_gagal_tanpa_panic` ditambah 5 panggilan baru.
- [ ] **Step 2:** FAIL → **Step 3:** implement keduanya (pola `jalan()` di sock_client.rs:52) → **Step 4:** PASS → **Step 5:** Commit `feat(sock): bio/nama/foto/blokir bot`.

### Task 9: Port stub yang tinggal colok `sock`/media

**Files:**
- Modify: `plugin/src/tools/setbio.rs`, `setname.rs`, `setpp.rs`, `delpp.rs`, `plugin/src/owner/block.rs`

**Interfaces:** Consumes Task 8 + Task 6 pattern (media_fetch untuk setpp/delpp foto dari quoted).

- [ ] **Step 1: Test gagal** per plugin — stub `Sock` test yang mencatat panggilan: `setbio "halo"` → `set_bio("halo")` terpanggil + teks sukses JS; `block @user` → `blokir(jid, true)`; `setpp` dengan quoted image → unduh + `set_foto_bot`.
- [ ] **Step 2:** FAIL → **Step 3:** port (hapus pesan "tidak tersedia"; ponytail comments dihapus/diperbarui) → **Step 4:** `cargo test -p oktz-plugins` PASS → **Step 5:** Commit `feat: port setbio/setname/setpp/delpp/block`.

### Task 10: Audit & dokumentasi sisa stub

**Files:**
- Create: `docs/rust-rewrite/STUBS.md` (daftar ~90 stub tersisa dikelompokkan penyebab: SSH panel, canvas fallback, microlink screenshot, resolver, jadibot-manager, album/kirim-2-pesan, dll — Batch 2)

- [ ] **Step 1:** Kumpulkan via `grep -rln "ponytail" plugin/src` + pesan "tidak tersedia"; tulis tabel file → penyebab → capability yang dibutuhkan.
- [ ] **Step 2:** Commit `docs: audit stub tersisa (Batch 2)`.

## Urutan & dependensi

Task 1 → 2 → 3 → 4 (Turso, berurutan). Task 5, 6, 7, 8 mandiri; Task 9 butuh 8. Task 10 terakhir. Group settings & RPG TIDAK butuh task sendiri — keduanya hidup di `settings.json` (`users`/`groups`) yang termirror Task 2; verifikasi di Task 2 test (`SetGroup` antilinkgc + `SetUser` koin termirror).
