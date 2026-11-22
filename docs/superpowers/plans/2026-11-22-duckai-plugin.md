# Plugin `.duck` (duck.ai) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Plugin `.duck` untuk chat duck.ai dengan pilihan model (`gpt-6`, `gpt-5.4`, `haiku-5.5`), WebSearch + reasoning selalu aktif, memakai solver challenge murni Rust.

**Architecture:** Satu modul `plugin/src/ai/duckai.rs`: solver challenge (blob base64 → dijalankan di QuickJS embed dengan shim DOM → `client_hashes` di-sha256 → header `x-vqd-hash-1`), klien dua langkah (status → chat) lewat `HttpCall`, parser SSE, lalu handler plugin dengan model per chat di `Ctx::sessions`.

**Tech Stack:** Rust 2024, `oktz-plugin-api` (`HttpGet::call`, `HttpCall`, `Reply`, `Ctx`), `serde_json`, `sha2` (sudah di `Cargo.lock`).

**Spec:** `docs/superpowers/specs/2026-11-22-duckai-plugin-design.md`

## Global Constraints

- Tanpa proses tambahan, tanpa browser. Challenge dijalankan di QuickJS yang di-embed.
- Dependency baru: `rquickjs = "0.14"` (QuickJS C di-embed) + `sha2 = "0.11"` (`sha2` sudah ada di lock).
- base64 encode/decode ditulis lokal; `base64_encode` pola `plugin/src/search/play2.rs`.
- Tanpa `unwrap()`/`expect()` di jalur produksi; tanpa komentar yang menjelaskan kode.
- Model gratis saja: `gpt-6-luna`, `gpt-5.4-mini`, `claude-haiku-5-5`.
- WebSearch selalu: `canUseTools: true` + `metadata.toolChoice.WebSearch`.
- Reasoning selalu: `gpt-5.4` → `medium`, lainnya → `low`.
- Gambar di luar scope.
- Format + lint sebelum commit: `cargo fmt`, `cargo clippy -p oktz-plugins --all-targets`.

## Review Focus

- Blob challenge berubah bentuk → plugin wajib gagal rapi, bukan panic.
- `418`/`429` → kartu error + ☢, tanpa retry berantai.
- Arg pertama mirip model tapi tak dikenal (`.duck gpt-7 halo`) → jangan telan sebagai pertanyaan; tampilkan model tak dikenal.
- Balasan kosong (semua event bukan `assistant`) → kartu gagal.
- Jawaban > 4096 char → dipotong, bukan terpotong di tengah UTF-8.

---

### Task 1: Helper base64 + sha256

**Files:**
- Modify: `plugin/Cargo.toml`
- Create: `plugin/src/ai/duckai.rs` (helper dulu)

**Interfaces:**
- Produces: `fn b64_encode(&[u8]) -> String`, `fn b64_decode(&str) -> Option<Vec<u8>>`, `fn sha256_b64(&str) -> String`.

- [ ] Tulis tes: encode `b"hi"` = `"aGk="`, decode balik, `sha256_b64("abc")` = `"ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0="`, `b64_decode("!!!")` = `None`.
- [ ] Jalankan `cargo test -p oktz-plugins --lib b64` → FAIL (belum ada).
- [ ] Implement helper; sha256 lewat `sha2::{Digest, Sha256}`.
- [ ] Tes lulus. Commit.

### Task 2: Jalankan challenge di QuickJS + hitung header

**Files:**
- Modify: `plugin/src/ai/duckai.rs`
- Fixture: `plugin/src/ai/fixtures/duckai-vqd-1.js` (blob nyata hasil `GET /duckchat/v1/status`), `-2.js`, `-3.js`

**Interfaces:**
- Produces:
  - `struct Tantangan { server_hashes: Vec<String>, raw: Vec<String>, challenge_id: String, timestamp: String, debug: String }`
  - `fn jalankan_challenge(js: &str) -> Option<Value>`
  - `fn header_vqd(blob_b64: &str) -> Option<String>`

- [ ] Simpan 3 blob fixture (curl, `x-vqd-accept: 1`).
- [ ] Tes: `jalankan_challenge` pada fixture → `client_hashes` = `[UA, "9687", "5246"]`, 3 `server_hashes`, `meta.challenge_id` ada.
- [ ] Jalankan → FAIL.
- [ ] Implement: shim DOM JS (document/createElement/querySelector `#jsa`, NodeList, window globals, tabel `innerHTML` 4 template) + `Runtime`/`Context` + pump `execute_pending_job`.
- [ ] `header_vqd` = base64 dari JSON `server_hashes`, `client_hashes` (sha256 tiap raw), `signals:{}`, `meta{...,origin,stack,duration}`.
- [ ] Tes lulus. Commit.

### Task 3: Status → header → chat → SSE

**Files:**
- Modify: `plugin/src/ai/duckai.rs`

**Interfaces:**
- Produces: `fn jawab(http: &dyn HttpGet, model: &str, effort: &str, tanya: &str) -> Hasil`
- `enum Hasil { Ok { teks: String, pikir: String, sumber: Vec<String> }, Ditolak(String), Gagal }`
- `fn parse_sse(raw: &str) -> (String, String, Vec<String>)` → (jawaban, thinking, sumber).

- [ ] Tes `parse_sse`: `assistant` delta digabung; `reasoning` masuk pikir; `tool-invocation` state `result` ambil `data.urls`; `[DONE]` berhenti; `[CHAT_TITLE:..]` diabaikan.
- [ ] Tes `jawab` dengan stub `HttpGet`: stub memeriksa URL status & chat, body memuat `canUseTools: true`, `metadata.toolChoice.WebSearch`, `reasoningEffort`; balasan `418` → `Ditolak`.
- [ ] Jalankan → FAIL, implement, lulus. Commit.

### Task 4: Handler plugin `.duck`

**Files:**
- Modify: `plugin/src/ai/duckai.rs`
- Modify: `plugin/src/ai/mod.rs` (`pub mod duckai;`)

**Interfaces:**
- Consumes: Task 1–3.
- Produces: `#[plugin(name = "duck", ...)] pub fn duck(ctx: Ctx) -> Reply`.

- [ ] Tes: registrasi meta (nama/alias/kategori); `.duck` kosong → kartu bantuan + daftar model, tanpa request; `.duck gpt-5.4 hai` → model `gpt-5.4-mini` + `reasoningEffort=medium` tersimpan di sesi; `.duck gpt-7 hai` → kartu model tak dikenal tanpa request; `.duck hai` setelah memilih → pakai model tersimpan; transport error → `error_template_bawaan`.
- [ ] Implement model enum + alias (`gpt-6`/`gpt6`/`luna`, `gpt-5.4`/`gpt54`/`mini`, `haiku-5.5`/`haiku55`/`haiku`), sesi `duckai:model:<chat>`, kartu help, balasan (clip 4096, blok 💭, daftar 🌐 maks 3).
- [ ] Daftarkan modul. Tes lulus. Commit.

### Task 5: Verifikasi + rilis

- [ ] `cargo fmt`; `cargo clippy -p oktz-plugins --all-targets`; `cargo check -p oktz-plugins`.
- [ ] Tes live sekali (`#[ignore]`, pakai `UreqHttp`): status → header → chat `gpt-6-luna` → ada jawaban.
- [ ] Commit, push, pastikan workflow Actions + asset Release terbaru.