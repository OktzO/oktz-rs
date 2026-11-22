# Plugin `.duck` — duck.ai (GPT-6 Luna / GPT-5.4 / Claude Haiku 5.5)

Tanggal: 2026-11-22
Status: menunggu persetujuan

## Kenapa

Bot punya banyak plugin AI, semuanya backend pihak ketiga tanpa pilihan model.
duck.ai menawarkan chat gratis tanpa akun untuk beberapa model frontier, plus
WebSearch bawaan dan `reasoningEffort`. Satu plugin `.duck` dengan pemilihan
model membuat fitur AI bot bisa naik kualitas tanpa biaya API.

`.ai` (Gemini) **tidak** disentuh. Plugin lain (`deepseek`, `qwen3`, dst) tetap.

## Tujuan

- `.duck <model> <pertanyaan>`: pilih model per chat, lalu tanya.
- Model yang didukung: `gpt-6` (`gpt-6-luna`), `gpt-5.4` (`gpt-5.4-mini`),
  `haiku-5.5` (`claude-haiku-5-5`). Hanya model `entityHasAccess: true`.
- WebSearch selalu aktif (`canUseTools: true` + `metadata.toolChoice.WebSearch`).
- Reasoning selalu aktif, level tertinggi yang didukung model itu
  (`gpt-5.4` → `medium`, sisanya → `low`).
- Model terakhir per chat diingat di `Ctx::sessions`.

## Bukan tujuan

- Ganti nama / hapus plugin AI lain.
- Image generation, upload file, riwayat percakapan multi-turn.
- Bypass berulang saat duck.ai menolak (satu percobaan solve per request).

## Kendala (hasil uji live 2026-11-22)

- Chat wajib header `x-vqd-hash-1` = base64(JSON) hasil *challenge* JS dari
  `GET https://duckduckgo.com/duckchat/v1/status` (header bernama sama, isi blob
  JS ter-obfuscate, nama variabel + konstanta rotasi berubah tiap request).
- `client_hashes` = `base64(sha256(raw_i))`, `raw = [UA, n1, n2]`; n1/n2 hasil
  feature-detect DOM. `server_hashes` (3 literal b64), `challenge_id`,
  `timestamp`, salt, semuanya ada di blob.
- `meta` wajib ditambah `origin`, `stack`, `duration`.
- Server memvalidasi ketiga `client_hashes`; nilai karangan → `418 ERR_CHALLENGE`.
- Replay header hasil browser asli dari curl → `200`. Jadi Rust bisa, asal
  header-nya benar.
- Headless browser diblok (`ERR_BN_LIMIT`), jadi plugin tidak boleh bergantung
  pada browser; tidak ada browser di panel.
- Katalog model publik: `GET /duckchat/v1/models`.
- Gambar: `content[]` menerima `{type:"image", mimeType, image:"data:image/...;base64,..."}`,
  tetapi tanpa `moderationToken` stream membalas `ERR_MODERATION_UNAVAILABLE`.
  Uji di app asli (headed, lampirkan PNG lewat file input) juga gagal:
  `ERR_IMAGE_UPLOAD_DISPLAY` + `ERR_IMAGE_PROCESSING` di tier anonim/free.
  Jadi gambar tidak masuk scope sampai duck.ai membuka jalur moderasinya.

## Pendekatan

Challenge duck.ai dijalankan sebagai JS di **QuickJS yang di-embed** (`rquickjs`,
C statis di dalam binary). Tanpa browser, tanpa proses tambahan.

1. `GET status` (`x-vqd-accept: 1`, UA Chrome) → header base64 → JS.
2. JS dijalankan di QuickJS dengan shim DOM kecil (document/createElement/
   querySelector `#jsa`/iframe/NodeList/window globals). Hasilnya `client_hashes`
   mentah.
3. `client_hashes = base64(sha256(raw))` di Rust, `meta` ditambah `origin`,
   `stack`, `duration`, lalu di-base64 → header `x-vqd-hash-1`.
4. `POST /duckchat/v1/chat` dengan header itu.
5. Parse SSE: `role: assistant` → append `message`; `role: reasoning` → buffer
   thinking; `role: tool-invocation` state `result` → ambil max 3 URL sumber;
   `[DONE]` → selesai.

Dependency: `rquickjs = "0.14"` (QuickJS C, build +~35s, binary +~1.4MB) dan
`sha2 = "0.11"` (sudah ada di `Cargo.lock`). base64 encode/decode ditulis lokal
(pola `play2.rs`).

Parser murni Rust sempat dicoba dan ditolak: obfuscator memakai decoder
per-scope, 2 dari 3 sampel menghasilkan angka salah. Shim QuickJS terbukti
menghasilkan angka identik dengan browser pada sampel yang sama.

`innerHTML` di shim memakai tabel 4 template yang pernah dikirim DDG (aturan
parse HTML5 Chrome tidak bisa disederhanakan). Template baru → angka salah →
server balas 418; tambahkan satu baris di tabel.

## UX

- `.duck` → kartu bantuan + daftar model + model aktif chat ini.
- `.duck gpt-6 <tanya>` → set model `gpt-6`, jawab.
- `.duck <tanya>` → pakai model tersimpan (default `gpt-6`).
- Alias model: `gpt-6`/`gpt6`/`luna`, `gpt-5.4`/`gpt54`/`mini`,
  `haiku-5.5`/`haiku55`/`haiku`.
- Jawaban: teks (potong 4096) + reaksi ✅. Kalau ada thinking:
  `💭 *Proses Berpikir:*` (preview 800 char) di depan. Kalau WebSearch dipakai:
  daftar maks 3 sumber di akhir.

## Penanganan gagal

- Status `418` / `429` → kartu "duck.ai menolak (challenge/limit)" + reaksi ☢.
- Challenge gagal dijalankan (shim kurang / template `innerHTML` baru) →
  kartu error biasa, tanpa panic.
- Transport error → `crate::common::error_template`.
- Tidak ada retry otomatis berantai.

## File

- `plugin/src/ai/duckai.rs` — plugin + shim + solver + parser SSE + test.
- `plugin/src/ai/fixtures/duckai-vqd.js` — satu blob challenge nyata (fixture tes).
- `plugin/src/ai/mod.rs` — tambah `pub mod duckai;`.
- `plugin/Cargo.toml` — `rquickjs = "0.14"`, `sha2 = "0.11"`.

## Tes

- sha256 known-answer ("" dan "abc").
- base64 encode/decode round-trip.
- Challenge fixture menghasilkan `[UA, "9687", "5246"]` (angka browser nyata).
- Header hasil `header_vqd` berisi sha256 + meta tambahan.
- SSE parser: assistant delta, reasoning, tool-invocation result, `[DONE]`.
- Resolusi alias model + model tersimpan per chat.
- 418 → kartu error, bukan panic.
- Offline → `error_template_bawaan(".", "duck")`.

## Catatan risiko

Bentuk blob bisa berubah kapan saja; solver sengaja punya jalur gagal eksplisit.
Kalau DDG menambah bentuk baru, cukup tambah satu cabang di pengenal bentuk.