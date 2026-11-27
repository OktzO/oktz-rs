# Desain ronde 2: churn store per-pesan + guard decode media

> **Catatan (2026-11-27):** tujuan paritas byte-per-byte terhadap bot JS sudah
> tidak berlaku. Rust adalah acuan perilaku. Kalimat paritas di bawah hanya
> konteks sejarah.

Tanggal: 2026-11-15 · Status: lanjutan disetujui (ronde 1 A+B terbukti: RSS turun lagi
setelah burst, v0.2.47 rilis). Ronde 2 menyasar dua sisa akar dari analisis ronde 1.

## Intent

Ronde 1 memperbaiki pengembalian RAM ke OS (jemalloc) dan bottleneck worker. Dua sisa
biaya per-pesan yang masih ada:

1. **Churn store** — `StoreContext::apply` (`pipeline.rs:384-431`) menserialisasi SELURUH
   store JSON 2× per batch mutasi: sekali untuk backup rollback (`:408`, `to_string`)
   dan sekali lagi di `JsonStore::write` (`:111`) untuk disk. Store bot produksi bisa
   jutaan byte → 2 alokasi besar per pesan bermutasi.
2. **Decode media tanpa batas** — titik decode gambar dari byte EKSTERNAL
   (`gambar.rs::sharp`, `sticker.rs::to_rgba`) memakai `image::load_from_memory` /
   `Reader::decode()` tanpa batas dimensi: gambar 50000×50000 (yang di JS sharp ditolak
   `limitInputPixels`) tetap di-decode → alokasi RGBA hingga belasan GB.

## Goals

- 1 serialisasi penuh per batch mutasi, bukan 2 — byte di disk identik.
- Decode byte eksternal menolak gambar di atas batas piksel SEBELUM alokasi besar,
  sama seperti perilaku default `limitInputPixels` sharp (268402689 px = 16383²).
- Paritas byte-per-byte balasan tidak berubah.

## Non-goals / anti-goals

- TIDAK mengubah tipe `Ctx` (roster `Value` clone per peserta tetap) — mengubahnya
  memecah API semua plugin; biayanya lebih besar daripada manfaatnya.
- TIDAK mengubah `throttle_send` 300ms, parameter ffmpeg, atau store mutator lain.
- TIDAK menyentuh decode PNG self-render (`brat.rs:276,294` — artefak canvas sendiri,
  dimensi bounded) dan path `.topixel` (JS memakai `limitInputPixels: false`
  di `topixel.js:27`; Rust-nya punya divergensi tercatat `ponytail:`, tidak lewat
  helper).
- TIDAK menambah batas untuk `ureq` body (sudah ada `MAX_BODY` 25MB).

## Pendekatan yang dipertimbangkan

1. **Serialize sekali, pakai ulang byte (dipilih untuk C)** — backup `to_vec` dipakai
   untuk rollback DAN ditulis ke disk via `JsonStore::write_bytes` (atomic tmp+rename
   yang sama). Serialisasi serde_json deterministik utk Value yang sama (preserve_order)
   → byte disk identik dengan perilaku lama. Diff kecil, tanpa perubahan semantik
   rollback.
2. Reload-from-disk saat gagal (buang backup total) — menyingkirkan backup, TAPI disk
   bisa lebih tua dari memori saat boot-restore Turso belum ter-tulis; batch pertama
   yang gagal akan membuang state restore. Ditolak: risiko hilangnya ratchet.
3. Guard decode di level `image::io::Limits` per decoder — format yang tidak
   menghormati limits lolos; cek dimensi header manual (`into_dimensions`) bekerja
   untuk semua format. Dipilih cek manual.

## Desain

### C. Store churn (`lib/oktz-store/json_store.rs` + `lib/oktz-wa/pipeline.rs`)

- `JsonStore::write_bytes(&self, body: &[u8]) -> Result<()>` — path atomic yang sama
  dgn `write()` (tmp per-pid+seq, fs::write, rename). `write()` di-refactor jadi
  serialize + `write_bytes` (satu sumber kebenaran).
- `apply()` clone-on-write: mutasi dikerjakan pada salinan (`inner.clone()`), disk
  ditulis dari salinan (`kerja.write()` — 1 serialisasi, kondisi PASCA-mutasi),
  memori di-commit setelah disk sukses (`*inner = kerja`). Rollback = buang salinan:
  semantik identik dengan backup serialisasi lama (inner pramutasi), tanpa serialisasi
  kedua. REVISI: draft awal "tulis byte backup ke disk" salah — byte backup = kondisi
  pramutasi, membuat disk stale; ditangkap test pipeline saat implementasi.

### D. Guard decode media (`lib/oktz-plugin-api`)

- `gambar.rs`: `pub(crate) const BATAS_PX: u64 = 268_402_689;` + `pub(crate) fn
  lewat_batas_px(w: u32, h: u32) -> bool` (`w*h > BATAS_PX`, saturating) + `pub(crate)
  fn dekode_dgn_batas(data: &[u8]) -> Result<DynamicImage, String>`:
  `ImageReader::new(Cursor::new(data)).with_guessed_format()` → `into_dimensions()`
  (baca header saja) → tolak bila > batas → baru `image::load_from_memory`.
  Pesan error tetap persis string lama ("tidak bisa mendeteksi format gambar",
  "gagal decode gambar") supaya test & teks balasan tidak berubah.
- `gambar.rs::sharp()` decode lewat `dekode_dgn_batas`; `sticker.rs::to_rgba` idem.
- `brat.rs`, `kanvas.rs`, decode self-render TIDAK disentuh.

## Ukuran kerja

Diff kecil: 2 file lib (`json_store.rs`, `pipeline.rs`) + 2 file plugin-api
(`gambar.rs`, `sticker.rs`) + test. Tanpa perubahan render.

## Ukuran keberhasilan

- `cargo test --release` + `cargo clippy --release --all-targets` hijau penuh.
- `oktz plugins` 788, exit 0.
- Tidak ada perubahan pada teks balasan (diff hanya infrastruktur).
