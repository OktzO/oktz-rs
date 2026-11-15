# Ronde 2 Implementation Plan: churn store + guard decode media

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (metode native, dipilih owner di ronde 1) untuk implementasi task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Kurangi churn alokasi per-pesan (serialisasi store 2×→1× per batch) dan tolak decode gambar raksasa sebelum alokasi RGBA, tanpa menyentuh render.

**Architecture:** C: `JsonStore::write_bytes` (atomic tmp+rename yang sama) dipakai `apply()` untuk menulis byte backup yang sudah diserialisasi sekali. D: helper `dekode_dgn_batas` membaca dimensi header dulu, menolak > 268402689 px (default `limitInputPixels` sharp), baru decode.

**Tech Stack:** Rust, serde_json (preserve_order), image 0.24.9.

**Spec:** `docs/superpowers/specs/2026-11-15-ronde2-churn-guard-design.md`

## Global Constraints

- Paritas byte-per-byte balasan tidak berubah; jangan sentuh teks render.
- `cargo update` TERLARANG; tidak ada dep baru.
- Komentar Bahasa Indonesia, hanya alasan non-obvious.
- Semua perintah cargo wajib: `export PATH="/nix/store/0554jm1l1qw1pcfqsliw91hnifn11w8m-gnumake-4.4.1/bin:/nix/store/061ybr50r1m2jnvih4vvpgmgwb37df3r-gcc-wrapper-12.2.0/bin:$PATH" CARGO_TARGET_DIR=/tmp/opencode/oktz-target`.
- JANGAN jalankan `scripts/paritas-audit.py` di mesin ini (merusak `docs/paritas/`, lihat ledger ronde 1).
- Jangan sentuh: `vendor/`, `throttle_send` 300ms, ffmpeg params, `brat.rs` decode self-render, path `.topixel`.

## Review Focus

- Dimensi PERSIS == batas (16383×16383) harus LOLOS (sharp: `>` bukan `>=`) → test `dimensi_persis_batas_lolos` di Task 2.
- Byte garbage ke `sharp()`/`to_rgba` → pesan error tidak berubah dari hari ini → semua test sticker/brat/kanvas existing hijau di Task 2.
- `apply()` gagal di tengah batch → store dikembalikan, disk tetap kondisi terakhir yang baik → test rollback existing (`pipeline.rs:5589-5648`) hijau di Task 1.
- Byte disk hasil `apply()` sukses == serialisasi `Value` hasil mutasi (bukan byte stale) → test `apply_sukses_disk_isi_mutasi` di Task 1.
- GIF animasi/dimensi canvas: guard membaca header canvas, bukan frame → decode sticker GIF existing tetap jalan (suite sticker) di Task 2.

---

### Task 1: Serialisasi store sekali per batch (`write_bytes`)

**Files:**
- Modify: `lib/oktz-store/json_store.rs` (tambah `write_bytes`, refactor `write()`)
- Modify: `lib/oktz-wa/pipeline.rs:384-431` (`apply()`)

**Interfaces:**
- Consumes: `JsonStore::all() -> &Value` (`json_store.rs:79`), TMP_SEQ + tmp/rename pattern `write()` (`json_store.rs:100-114`).
- Produces: `JsonStore::write_bytes(&self, body: &[u8]) -> Result<()>` — atomic write byte mentah, error type sama dengan `write()`.

- [ ] **Step 1: Tulis test gagal di `json_store.rs` (mod tests)**

```rust
#[test]
fn write_bytes_tulis_persis_dan_tanpa_tmp_sisa() {
    let p = tmpfile("array-write-bytes");
    let mut s = JsonStore::open(&p).unwrap();
    s.replace(serde_json::json!({"a": 1})).unwrap();
    s.write().unwrap();
    // Tulis byte lain lewat write_bytes: isi file = body persis.
    s.write_bytes(br#"{"b":2}"#).unwrap();
    assert_eq!(std::fs::read(&p).unwrap(), br#"{"b":2}"#);
    // Tidak ada file .tmp sisa di dir yang sama.
    let dir = p.parent().unwrap();
    assert!(!dir.read_dir().unwrap().any(|e| {
        e.unwrap().file_name().to_string_lossy().contains(".tmp")
    }));
}
```

- [ ] **Step 2: Jalankan, pastikan gagal**

Run: `cargo test --release -p oktz-store --lib write_bytes` → compile error `no method named write_bytes` (RED).

- [ ] **Step 3: Implementasi `write_bytes` + refactor `write()` di `json_store.rs`**

```rust
/// Tulis body JSON mentah atomik (tmp+rename) — pemanggil yang sudah
/// memegang byte serialisasi tidak perlu serialisasi kedua.
pub fn write_bytes(&self, body: &[u8]) -> Result<()> {
    if let Some(parent) = self.path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = self
        .path
        .with_extension(format!("{}-{seq}.tmp", std::process::id()));
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, &self.path)?;
    Ok(())
}
```
`write()` jadi: serialize `to_string(&self.data)` → `self.write_bytes(body.as_bytes())`.

- [ ] **Step 4: Jalankan test oktz-store semua**

Run: `cargo test --release -p oktz-store --lib` → PASS semua.

- [ ] **Step 5: Rework `apply()` di `pipeline.rs:408-429` (clone-on-write)**

Mutasi dikerjakan pada salinan: `let mut kerja = inner.clone();` → `apply_mutation(&mut kerja, ...)?` (gagal → `Err` langsung, `inner` utuh = semantik backup lama) → `kerja.write().map_err(|e| StoreWriteError { detail: format!("tulis gagal: {e}"), ... })?` → sukses: `*inner = kerja;` lalu dirty flag seperti semula. `inner` yang ditulis ke disk adalah kondisi PASCA-mutasi — backup pramutasi TIDAK boleh ikut ke disk.

- [ ] **Step 6: Test pipeline**

Run: `cargo test --release -p oktz-wa --lib` (termasuk test rollback `5589-5648`) → PASS 357+.

Kalau tidak ada test yang membaca FILE hasil `apply` (bukan hanya baca store), tambahkan satu test: apply satu mutasi sukses → `std::fs::read(path)` di-parse `serde_json::from_str::<Value>` → berisi mutasi (bukan kondisi lama) — pin bahwa byte yang ditulis adalah hasil mutasi terkini, bukan backup stale.

- [ ] **Step 7: Commit**

```bash
git add lib/oktz-store/json_store.rs lib/oktz-wa/pipeline.rs
git commit -m "perf(store): serialisasi store sekali per batch mutasi (write_bytes)"
```

### Task 2: Guard decode media (batas piksel sebelum decode)

**Files:**
- Modify: `lib/oktz-plugin-api/gambar.rs` (BATAS_PX, lewat_batas_px, dekode_dgn_batas, `sharp()` pakai helper)
- Modify: `lib/oktz-plugin-api/sticker.rs:111-114` (`to_rgba` pakai helper)

**Interfaces:**
- Consumes: `image::io::Reader::into_dimensions()` (baca header saja, image 0.24.9), `image::load_from_memory`.
- Produces: `gambar::dekode_dgn_batas(data: &[u8]) -> Result<image::DynamicImage, String>`, `gambar::lewat_batas_px(w: u32, h: u32) -> bool`, `pub(crate) const BATAS_PX: u64 = 268_402_689`. Pesan error TIDAK berubah dari string hari ini ("tidak bisa mendeteksi format gambar", "gagal decode gambar"); pesan baru hanya untuk penolakan batas.

- [ ] **Step 1: Tulis test gagal di `gambar.rs` (mod tests)**

```rust
#[test]
fn dimensi_di_atas_batas_ditolak() {
    assert!(lewat_batas_px(16384, 16384));
    assert!(lewat_batas_px(u32::MAX, 2));
}

#[test]
fn dimensi_persis_batas_lolos() {
    // sharp: exceeds = > batas, bukan >= — 16383² = 268_402_689 lolos.
    assert!(!lewat_batas_px(16383, 16383));
    assert!(!lewat_batas_px(0, 0));
}
```

- [ ] **Step 2: Jalankan, pastikan gagal**

Run: `cargo test --release -p oktz-plugin-api --lib dimensi_` → compile error (RED).

- [ ] **Step 3: Implementasi const + fn + wiring `sharp()` & `to_rgba`**

`lewat_batas_px`: `(w as u64).saturating_mul(h as u64) > BATAS_PX`. `dekode_dgn_batas`: reader pertama untuk format error yang sama dgn hari ini, reader kedua (fresh Cursor) `into_dimensions()` untuk cek, lalu `image::load_from_memory(data)` dengan pesan error "gagal decode gambar" persis. `sharp()` tetap deteksi format dulu (pesan sama), decode via helper. `sticker.rs::to_rgba` → `dekode_dgn_batas(data)?.to_rgba8()`.

- [ ] **Step 4: Test plugin-api + suite sticker/brat/kanvas**

Run: `cargo test --release -p oktz-plugin-api --lib && cargo test --release -p oktz-plugins --lib sticker: && cargo test --release -p oktz-plugins --lib brat: && cargo test --release -p oktz-plugins --lib canvas:` → PASS semua (test decode nyata membuktikan guard tidak menolak gambar normal).

- [ ] **Step 5: Commit**

```bash
git add lib/oktz-plugin-api/gambar.rs lib/oktz-plugin-api/sticker.rs
git commit -m "perf(media): tolak decode gambar di atas batas piksel sharp (header check)"
```

### Task 3: Verifikasi penuh

**Files:** tidak ada perubahan — hanya verifikasi.

- [ ] **Step 1:** `cargo test --release` → PASS semua, 0 fail.
- [ ] **Step 2:** `cargo clippy --release --all-targets` → tidak ada warning baru di file yang disentuh.
- [ ] **Step 3:** `node --test launcher.test.js` → PASS 17.
- [ ] **Step 4:** build release + `oktz plugins` → 788, exit 0.
- [ ] **Step 5:** `git diff` audit: hanya `json_store.rs`, `pipeline.rs` (apply), `gambar.rs`, `sticker.rs` — nol perubahan teks render.
- [ ] **Step 6:** commit docs (spec+plan) + ledger; lapor hasil.
