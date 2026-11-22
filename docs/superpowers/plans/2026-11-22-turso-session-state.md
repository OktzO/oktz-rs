# Sesi WhatsApp utuh di Turso (`session_state`) — Implementation Plan

**Goal:** Session bot tersimpan di Turso sebagai SATU snapshot utuh (creds + db SQLite
+ sidik identitas) di tabel `session_state` scope `main`. Purge logout ikut
menghapus Turso. Mirror per-baris (`SESSION_MIRROR_TABLES`) dibuang.

**Spec:** percakapan 2026-11-22 (analisis root cause di chat) — tidak ada file spec.

## Context

Kondisi sekarang (root cause bug "hapus session lokal → restore Turso → rusak"):

1. Rust tidak pernah menulis `session_creds` — session Turso = snapshot onigis basi.
2. Mirror per-baris tidak pernah di-restore; skema `(k, v TEXT JSON)` + BLOB hex
   tidak punya jalur balik.
3. Creds & ratchet tidak atomik: `pulihkanTurso` (launcher.js:347) tulis creds
   lalu keys gagal = jalan tanpa fixture → device baru + ratchet lawas → Bad Mac.
4. Purge lokal (`purge_session`) tidak menyentuh Turso → reboot restore creds basi.

## Global Constraints

- Lokal SELALU sumber utama saat sehat; Turso = restore saat lokal hilang.
- Turso mati = warning, TIDAK pernah gagalkan boot.
- Snapshot Turso hanya ditulis kalau sidik creds lokal == sidik yang dipakai
  runtime (marker `.ident`); kalau tidak (=creds sudah di-kuarantin), jangan tulis.
- libsql tetap `remote`-only (oktz-store/Cargo.toml); tanpa embedded replica.
- `cargo fmt` + `cargo clippy --release --all-targets` + `cargo check` sebelum commit.
- Build/test penuh di GitHub Actions; lokal `cargo check` + test package terfokus.

## Review Focus

- Race: purge lokal jalan bersamaan dengan flush snapshot — snapshot tidak boleh
  menimpa Turso dengan sesi yang baru di-purge (guard marker `.ident`).
- Urutan restore: db → creds → `.ident`; gagal di tengah = hapus semua, bukan
  setengah-setengah.
- Blob db di Turso: ukuran (oktz-run.db bisa MB-an) — gzip + base64 di TEXT kolom.

## Tasks

### Task 6: Plugin `.clearsession` — bersihkan karantina + snapshot basi

- New: `plugin/src/owner/clearsession.rs` (ikuti pola `stopdandeletejadibot.rs`).
- Owner-only. TIDAK PERNAH menyentuh:
  - db session aktif (`database/wa/oktz-run.db` + sidecar + `.ident`),
  - `creds.json` aktif,
  - Turso `session_state` scope `main`.
- Yang dibersihkan (keduanya, laporan per item):
  - Lokal: folder karantina `*.broken-*` di samping db session + creds
    (padanan `BROKEN_SESSION_KEEP` — semua karantina dihapus).
  - Turso: semua scope `session_state` KECUALI `main` (backup `purged-<ts>`,
    scope uji), plus tabel legacy `session_creds` + `session_keys`
    (`DELETE FROM` keduanya).
- Ctx tidak membawa TursoStore (`ponytail:` plugin API belum punya jalur Turso;
  upgrade path: `Ctx::turso()`). Plugin mengembalikan instruksi + jalur lokal;
  penghapusan Turso lewat helper binary `oktz clear-session --turso` ATAU
  lewat env `TURSO_*` dibaca plugin langsung via `libsql` remote dari
  `oktz-store` — pilih: panggil helper `oktz_wa::clear_turso_session_backup`
  publik yang dibuat di modul `pairing_run` (dipanggil lewat `tokio::spawn`).
- Test: per plugin — karantina palsu di temp dir + scope `purged-<ts>` di
  `:memory:` → dibersihkan; scope `main` + db aktif UTUH.

### Task 1: Helper `session_state` di `oktz-store/turso.rs`

- Modify: `lib/oktz-store/turso.rs`
- Tambah tabel ke `ensure_tables`:
  `CREATE TABLE IF NOT EXISTS session_state (scope TEXT PRIMARY KEY, creds TEXT, db TEXT, ident TEXT, updated_at INTEGER)`
- Interface baru:
  - `get_session_state(conn, scope) -> Result<Option<SessionState>>`
    (`SessionState { creds: String, db: String, ident: String }`)
  - `put_session_state(conn, scope, creds, db_b64, ident) -> Result<()>` (UPSERT)
  - `delete_session_state(conn, scope) -> Result<bool>` (`Ok(true)` = ada yang dihapus)
  - `list_session_scopes(conn) -> Result<Vec<String>>`
- Tidak mengubah tabel/tipe lama (kompatibel JS lama: `stores` tetap).
- Test: `:memory:` — put → get roundtrip, delete, list_scopes.

### Task 2: Purge logout hapus `session_state` Turso

- Modify: `lib/oktz-wa/pairing_run.rs`
- `PurgeReport` tambah field `turso_deleted: bool` (default `false`).
- `PurgePlan` tambah `turso: Option<oktz_store::turso::TursoStore>`.
- `PurgePlan::resolve()` tetap tanpa Turso (env process-wide; pemanggil yang
  menempelkan store — `connect_device` di `bot.rs`).
- Backup 401 palsu: SEBELUM delete, salin row ke scope `purged-<now_ms>`.
- Hapus → `DELETE FROM session_state WHERE scope='main'`.
- `purge_session` murni filesystem sekarang; penanganan Turso di
  `purge_handler` (async, spawn task) supaya signature lama tidak pecah.
- Modify: `lib/oktz-wa/bot.rs` — `connect_device` menempelkan
  `TursoStore::bare` (kalau `cfg.turso.enabled`) ke plan.
- Test: plan dengan store `:memory:` → put_session_state("main") → purge →
  scope "main" hilang + scope `purged-<ts>` ada.

### Task 3: Snapshot db → Turso saat runtime (ganti mirror per-baris)

- Modify: `lib/oktz-wa/bot.rs`
- Buang: `SESSION_MIRROR_TABLES`, `mirror_sqlite_table`, `unwrap_json1_strings`
  beserta tesnya, dan pemanggilan mirror di `turso_boot_once`.
- Ganti: `snapshot_session_to_turso(store, conn, db_path) -> Result<bool>` —
  - Guard: marker `<db>.ident` harus ada DAN isinya == sidik creds.json aktif
    (baca `CREDS_ENV`); tidak cocok = `Ok(false)`, jangan tulis.
  - `VACUUM INTO <tmp>` (sqlite, diesel), gzip, base64 → kolom `db`.
  - Creds dari `CREDS_ENV` path → kolom `creds`; marker → kolom `ident`.
  - `put_session_state(conn, "main", ...)`.
- Dipanggil di flush task (`flush_once_turso`) berdampingan dengan flush blob
  `stores` — gate flag dirty terpisah: snapshot session HANYA saat ada tulisan
  disk db session (deteksi via mtime db berubah sejak snapshot terakhir;
  `ponytail:` mtime cukup, upgrade path: hook tulisan store).
- Hapus juga pemanggilan mirror tabel session di `turso_boot_once` (task 4 plan lama).
- Test: store `:memory:` + db sqlite sementara (marker ident cocok) →
  snapshot → `get_session_state` berisi creds + db + ident; marker beda →
  `Ok(false)` dan Turso tidak berubah.

### Task 4: Launcher restore dari `session_state`

- Modify: `launcher.js`
- Ganti `pulihkanTurso`: baca `session_state` scope `main`
  (`SELECT creds, db, ident FROM session_state WHERE scope = ?`).
- Tulis: `creds.json` dari kolom `creds`; `database/wa/oktz-run.db` dari kolom
  `db` (base64 → gunzip → buffer, mode 0o600); marker `.ident` dari kolom
  `ident` (path: `<db>.ident` = `with_extension("ident")` dari bot.rs:602 —
  `oktz-run.ident`).
- Hapus logika `session_keys` + fixture + `turso-session.json`.
- Guard: kolom `db` kosong → restore creds saja (pairing ulang db, perilaku
  lama) — TIDAK setengah-setengah: kalau creds ada tapi db gagal ditulis,
  hapus creds lagi → pairing flow.
- Node gunzip: `zlib.gunzipSync` (zlib bawaan, tanpa dependency).

### Task 5: Bersih-bersih jalur fixture lama

- Modify: `src/main.rs` — hapus `SESSION_FIXTURE_ENV` + `fixture_sesi` +
  argumen fixture dari `run`/`smoke`; `prepare_store` dipanggil `fixture: None`.
- Modify: `lib/oktz-wa/bot.rs` — `prepare_store` buang parameter
  `session_fixture` + jalur `load::load_into_store`; `crate::load` modul
  importer onigis dibiarkan (dipakai fixture test) ATAU dihapus kalau tidak ada
  pemakai lain — cek pemakai.
- README: update bagian Turso (`README.md:12-24,50,84`) — restore dari
  `session_state`, bukan `session_keys`.

## Ordering

Task 1 → 2 → 3 → 4 → 5. Task 2 butuh Task 1 (helper). Task 3 butuh Task 1.
Task 4 butuh Task 1 (skema). Task 5 terakhir.

## Rollback safety

- Tabel `session_creds`/`session_keys` lama TIDAK di-drop di Turso (kompatibel
  JS lama); cuma berhenti dipakai. Drop manual nanti kalau yakin.
