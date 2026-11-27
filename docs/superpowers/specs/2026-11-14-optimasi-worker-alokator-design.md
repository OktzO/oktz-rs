# Desain: Isolasi worker blocking + alokator jemalloc

> **Catatan (2026-11-27):** tujuan paritas byte-per-byte terhadap bot JS sudah
> tidak berlaku. Rust adalah acuan perilaku. Kalimat paritas di bawah hanya
> konteks sejarah.

Tanggal: 2026-11-14 · Status: disetujui (ronde 1 dari A+B) · Ronde 2 (C+D) menunggu hasil ukur

## Intent

Dua keluhan produksi:

1. **Bottleneck multi-balasan** — beberapa command berat paralel (play/tovn/sticker/brat +
   scraper HTTP) membekukan seluruh bot: heartbeat WS, update grup, dan balasan chat lain
   ikut nunggu.
2. **RAM merangkak naik dan tidak pernah turun** — RSS naik per hari pemakaian, tidak
   pernah kembali ke OS.

## Akar masalah (hasil analisis)

- `Router::route` (`lib/oktz-router/lib.rs:146`) memanggil handler plugin **sinkron di
  thread tokio worker**, di dalam `Pipeline::process` yang async. Handler berisi kerja
  berat: HTTP `ureq` timeout 20s (`oktz-plugin-api/ureq_http.rs:24`, tanpa
  `block_in_place`), ffmpeg subprocess (`oktz-plugin-api/ffmpeg.rs:154-212`), decode/encode
  gambar `image`. Worker tokio = jumlah CPU, semaphore pipeline 8
  (`lib/oktz-wa/pipeline.rs:4751`) — 3-4 command berat = semua worker macet.
- RSS tak turun = **glibc malloc tidak mengembalikan memori bebas ke OS**. Alokasi burst
  besar-besar (body HTTP ≤25MB `ureq_http.rs:25`, buffer RGBA penuh, snapshot store JSON
  diserialisasi 2× per mutasi `pipeline.rs:408-429`) memfragmentasi arena glibc. Semua
  cache utama SUDAH bounded (RosterCache TTL 300s, recent_media 512/30/512, regex cap 200,
  dedup prune) — bukan itu penyebabnya.

## Goals

- Handler plugin (dan answer-handler) jalan di thread pool blocking tokio, bukan thread
  worker → runtime tetap responsif berapa pun command berat yang paralel.
- Alokator memori yang mengembalikan memori bebas ke OS (jemalloc + decay) → RSS turun
  kembali setelah burst, bukan ratchet naik.
- **Paritas byte-per-byte tetap terjaga** (teks, emoji, pemisah, urutan) — perubahan ini
  tidak menyentuh render/konten.
- Seluruh test yang ada tetap hijau.

## Non-goals / anti-goals

- TIDAK mengubah throttle kirim 300ms global (`pipeline.rs:4039`) — pilihan antiflood yang
  ada, menyentuhnya mengubah perilaku antiflood.
- TIDAK mengubah parameter encode ffmpeg/preset/resolusi (paritas perilaku JS).
- TIDAK mengubah handler plugin jadi async (membongkar seluruh registry — jauh di luar
  scope ini; `ponytail:` catatan `ureq_http.rs:5-7`).
- TIDAK menyentuh `vendor/`.
- Tidak ada "unload plugin jarang dipakai dari RAM": di binary statis Rust tidak ada
  konsep unload kode; pemakai RAM nyata adalah alokator + buffer per-panggilan + churn
  store, yang justru jadi target di sini.

## Pendekatan yang dipertimbangkan

1. **Isolasi worker (dipilih)** — bungkus pemanggilan handler di `spawn_blocking`.
   Diff kecil-tengah, risiko rendah, menyerang bottleneck langsung.
2. Handler async penuh (reqwest + `HandlerFn` jadi future) — benar secara arsitektur tapi
   membongkar registry dan ratusan plugin; terlalu besar untuk masalah ini.
3. Runtime multi-runtime / thread pool manual sendiri — duplikasi fitur tokio, lebih rawan.
4. mimalloc / `malloc_trim` glibc periodik — alternatif alokator; jemalloc terbukti
   paling terkontrol via `dirty_decay_ms` + `background_thread`.

## Desain

### A. Isolasi worker (`oktz-router` + `oktz-wa`)

- `Router::route` jadi `async fn`. Pada titik dispatch, pemanggilan handler dibungkus:
  ```rust
  let reply = tokio::task::spawn_blocking(move || {
      catch_unwind(AssertUnwindSafe(|| (entry.handler)(c)))
  }).await
  ```
  - `catch_unwind` tetap di dalam closure → panik plugin tetap jadi balasan
    "❌ Plugin error…" (`lib.rs:150`), bukan JoinError.
  - Semua trait di `Ctx` (`HttpGet`/`MediaFetch`/`Sock`/`Berkas`) sudah `Send + Sync`
    dan `HandlerFn: Fn(Ctx) -> Reply + Send + Sync` — `Ctx` sah dikirim ke thread lain.
- `Router::answer` idem: rantai answer-handler dijalankan di `spawn_blocking`.
- Pemanggil produksi hanya satu (`pipeline.rs:4589` + `:4512`) — tambah `.await`.
- Batas paralel tetap dari semaphore existing pipeline (8) — handler paralel ≤ 8, thread
  pool blocking default tokio (max 512) tidak bisa meledak.
- Test router yang memanggil `route`/`answer` langsung (7 di `lib.rs`) → `#[tokio::test]`.

### B. Alokator jemalloc (bin `oktz`)

- Dep: `tikv-jemallocator` (feature `unprefixed_malloc_on_supported_platforms`) +
  `tikv-jemalloc-ctl`.
- `#[global_allocator]` di `src/main.rs` — satu file, crate bin saja.
- Runtime setting sekali di awal `main`:
  `background_thread: true`, `dirty_decay_ms: 5000`, `muzzy_decay_ms: 5000` → memori bebas
  dikembalikan ke OS ~5 detik setelah free, tanpa sinyal per-alokasi. Gagal set = log warn,
  jalan lanjut (bukan panic).
- Build release (`cargo build --release`) DAN aset statis CI
  (`x86_64-unknown-linux-gnu` + `+crt-static`, `.github/workflows/release.yml`) wajib
  sukses — jemalloc di glibc statis sudah jalur umum, tapi diverifikasi build lokal
  sebelum commit. Kalau statis gagal link: stop, laporkan, jangan diam-diam matikan fitur.

### Ukuran kerja

Diff kecil: 2 lib file (`oktz-router/lib.rs`, `oktz-wa/pipeline.rs`) + `src/main.rs` +
`Cargo.toml` bin + test router. Tanpa perubahan perilaku teks/balasan.

## Yang diserahkan ke builder

- Nama persis versi crate (cari versi `tikv-jemallocator` stabil terbaru yang cocok
  rustc 1.94; `cargo add` tanpa meng-upgrade dependency lain — `cargo update` TERLARANG).
- Jika `jemalloc_ctl` butuh runtime handle, pola inisialisasinya (get-or-log-fail).
- Penyesuaian kecil test yang jadi async.

## Ukuran keberhasilan

- `cargo build --release` + `cargo test --release` + `cargo clippy --release --all-targets`
  hijau.
- Build statis CI-style lokal (`RUSTFLAGS=-C target-feature=+crt-static cargo build
  --release --target x86_64-unknown-linux-gnu`) sukses.
- Test `oktz plugins`, satu test integrasi `live_route` jalan.
- Verifikasi manual RSS: burst command berat → RSS naik → turun kembali dalam ~10 dtk
  (bisa lewat `rss_kb` yang sudah ada di log CONNECTED/done).
