# Sistem AIRich Rust — desain

Tanggal: 2026-11-17
Status: disetujui 2026-11-17
Cakupan: `plugin/src/ai/ai.rs`, `plugin/src/search/ytplay.rs`,
`plugin/src/search/play2.rs`, `lib/oktz-plugin-api/`, `lib/oktz-wa/`

## Niat

Plugin `.ai`, `.ytplay`, dan `.play2` di ourin-md mengirim kartu lewat
`AIRich` (`src/lib/builder.js`). Port Rust sudah punya amplop wire-nya,
tetapi lapisan isi belum lengkap: link/sitasi/LaTeX tidak jadi inline
entity, blok kode tidak jadi submessage kode ber-highlight, dan `.play2`
ikut mengirim stanza edit yang di JS tidak ada.

Tujuan: **perilaku yang dilihat user sama dengan ourin-md untuk ketiga
plugin itu**, dengan amplop wire yang sudah terbukti render.

## Keputusan yang sudah dikunci bukti

Amplop tidak diubah. Nilainya sudah diverifikasi ke sumber luar:

| Field | Nilai | Alasan |
|---|---|---|
| `botJid` | `867051314767696@bot` | JID asli Meta AI (Baileys PR #2857, `ai-groups.js`). `0@bot` placeholder. |
| `forwardingScore` | 1 | v4.6 & v4.7 |
| `forwardOrigin` | 4 (`META_AI`) | v4.6 & v4.7 |
| `quotedType` | `EXPLICIT` saat quoted | `builder.js:1489`, v4.7 |
| `botResponseId` | UUID v4 per build | v4.7 |
| `verificationMetadata` | **tidak dikirim** | README ruby-baileys: proof palsu = `FAILED` = kartu termasker; tanpa proof = `UNKNOWN` = aman |
| `richResponseSourcesMetadata` | tidak dikirim | v4.7 menghapusnya |
| `unifiedResponse.data` | byte JSON (bukan ASCII base64) | protobufjs `writer.bytes` men-decode base64 |
| stanza ke-2 | `protocolMessage type=14` di dalam `botForwardedMessage` | "unified response baru tergambar setelah stanza kedua" (README ruby-baileys; NIXCODE v4.7) |

`messageType: 1` dan `botResponseId` tetap. `PreparedRich` tidak disentuh
(tidak dipakai plugin mana pun), tetapi tidak dijadikan contoh.

## Non-goal

- **Tidak menambah alias bahasa** (`js`→`javascript`, `ts`→`typescript`,
  `py`→`python`, `sh`→`bash`). JS tidak punya alias; parity ketat diikut
  apa adanya walau blok ` ```js ` tidak dapat highlight keyword.
  Perbaikan hanya bila muncul bug nyata saat pengujian owner, sebagai
  commit terpisah dengan `ponytail:` + alasan + test deviasi.
- Tidak menambah API AIRich yang tidak dipakai ketiga plugin:
  `addImage`, `addVideo`, `addReels`, `addProduct`, `addPost`,
  `addSource`, `addTip`, `addSuggest`, `notification`/
  `sessionTransparency`, `setTitle`/`setSubtitle`/`setBody`/`setFooter`,
  `loadFrom`, streaming.
- Tidak mengubah scraper (`gemini`, `youtube`), isi HTML player, atau
  teks plugin.
- Tidak menyentuh `ourin-md/` dan tidak mengubah `scripts/paritas-audit.py`.

## Anti-goal

- Kartu berubah bentuk di klien yang sebelumnya render.
- `.play2` kehilangan audio/voice-note fallback.
- Section dibangun di dua tempat dan bisa menyimpang lagi.

## Pendekatan

Dua opsi disodorkan:

- **A. Permukaan yang dipakai saja + `addSection` mentah** — teks, kode,
  tabel, kutipan, flag include. Semua kebutuhan ketiga plugin, tanpa kode
  spekulatif. **Dipilih.**
- B. Seluruh API AIRich 4.6/4.7. Ditolak: 2–3x kerja, tidak ada pemakai.

Cara membangun: satu modul builder di `oktz-plugin-api` yang meniru
metode `AIRich.addText/addCode/addTable` (submessage + section sekaligus),
supaya plugin tidak lagi menyusun JSON section sendiri.

Konten mengikuti **ourin-md v4.6** (sumber perilaku ketiga plugin);
amplop mengikuti v4.7 yang terbukti render. Hybrid ini disengaja: yang
dilihat user (section + teks) sama dengan JS lama, sedangkan envelope
memakai kontrak yang paint di klien.

## Bagian yang dibangun

### 1. `oktz_plugin_api::airich` (baru)

`extract_ie` — port `builder.js:40-212`:

- `[label](url)` → hyperlink; `[label](!url)` → `is_trusted: false`;
  `[](url)` → citation; `[txt|w|h|fh|pad]<url>` → LaTeX.
- Key: `NIXEL_HYPERLINK_{n}`, `NIXEL_CITATION_{n}`, `NIXEL_LATEX_{n}`.
- Teks hasil: `{{key}}...{{/key}}`.
- Entity: `GenAIInlineLinkItem`, `GenAISearchCitationItem`,
  `GenAILatexItem` persis `createIE`.
- Bug-for-bug: stack `[`, escape `\[`, `continue` saat jenis dimatikan
  (teks asli dibiarkan), `depth` untuk url berkurung, trim label/url,
  `Number(x) || default` untuk LaTeX.
- Iterasi per `char`; hasil sama dengan index UTF-16 JS karena pemisah
  yang relevan semuanya ASCII.

`tokenize_kode` — port `builder.js:1540-2203`:

- Peta kata kunci 12 bahasa (javascript, typescript, python, java, golang,
  c, cpp, php, rust, html, bash, markdown).
- `text`/`txt`/`plaintext`/kosong → satu token `DEFAULT`.
- Aturan token sama: whitespace, komentar `//` (dan `#` untuk python/bash),
  string `"'\``, angka `[0-9._]`, identifier `[a-zA-Z0-9_$]`
  (+ `-` css, `:` html), method bila diikuti `(`, css `prop:`, html
  `<tag>`.
- `lang.toLowerCase()` untuk lookup kata kunci dan kelas identifier;
  `lang` mentah untuk cabang css/html. Token bersebelahan dengan jenis
  sama digabung.
- Hasil: `Vec<KodeToken>` (`isi`, `jenis`) + konversi ke
  `{content, type}` dengan `DEFAULT|KEYWORD|METHOD|STR|NUMBER|COMMENT`
  (`STR`, bukan `STRING`).

Builder di `impl RichAi`:

```rust
pub fn teks(&mut self, teks: &str) -> &mut Self;       // addText
pub fn kode(&mut self, bahasa: &str, kode: &str);      // addCode
pub fn tabel(&mut self, baris: Vec<Vec<String>>);      // addTable
pub fn section(&mut self, section: Value);             // addSection mentah
```

- `teks`: submessage `Teks { teks: extracted }` — `builder.js:1048-1051`
  mengirim teks SETELAH `extractIE` (bukan markdown asli); section
  `{text: extracted, inline_entities?, __typename:
  GenAIMarkdownTextUXPrimitive}`.
- `kode`: submessage `Kode { bahasa, blok }` + section
  `{language, code_blocks, __typename: GenAICodeUXPrimitive}`.
- `tabel`: padding sel ke lebar maksimum dulu (seperti `toTableMetadata`),
  lalu:
  - submessage `Tabel { judul: "", baris: padded, heading: true }`
  - section `{rows: [{is_header, cells, markdown_cells?}], __typename:
    GenATableUXPrimitive}` dengan `markdown_cells` hanya bila ada sel
    ber-entity.

### 2. Model (`oktz-plugin-api::rich`)

`AiSubMessage` menjadi enum:

```rust
pub enum AiSubMessage {
    Teks { teks: String },
    Tabel { judul: String, baris: Vec<Vec<String>>, heading: bool },
    Kode { bahasa: String, blok: Vec<KodeToken> },
}

pub struct KodeToken { pub isi: String, pub jenis: Highlight }
pub enum Highlight { Default, Keyword, Method, Str, Number, Comment }
```

`RichAi`:

```rust
pub struct RichAi {
    pub submessages: Vec<AiSubMessage>,
    pub sections: Vec<serde_json::Value>,
    pub disclaimer: String,   // default "" — selalu ditulis
    pub quoted: bool,
    pub skor: Option<u32>,
    pub kirim_edit: bool,     // default false
}
```

- `disclaimer` berubah dari `Option<String>` ke `String` karena kedua
  lineage JS selalu menulis `messageDisclaimerText` (walau `""`).
- `kirim_edit` memisahkan AIRich (`true`) dari kartu play2 hand-rolled
  (`false`). Bukan opsi bebas: nilainya ditentukan jalur JS masing-masing.

### 3. Wire (`oktz-wa::rich`)

- `ai_submessages` mencocokkan enum:
  - `Teks` → `messageType 2`, `messageText`.
  - `Tabel` → `messageType 4`, `tableMetadata {title, rows}`, `isHeading`
    hanya baris pertama.
  - `Kode` → `messageType 5`, `codeMetadata {codeLanguage, codeBlocks}`.
- `ai_outer` selalu menulis `message_disclaimer_text: Some(...)`.
- `escape_ascii_7f` menjadi `stringify_escaped`: iterasi **UTF-16 code
  unit** agar emoji menjadi `\ud83d\ude80` seperti JS
  (`JSON.stringify` lalu replace `[\u007f-\uffff]`). Nilai JSON tetap
  valid; yang diperbaiki hanya byte parity.

### 4. Pipeline

`deliver_reply` hanya mengirim stanza edit bila
`Reply::TabelAi(ai)` dan `ai.kirim_edit`. `.play2` tidak lagi mengirim
edit. Reaksi/urutan lain tidak berubah.

### 5. Plugin

- `ai.rs`: `blocks_ke_reply` memakai `RichAi::teks/kode/tabel`;
  `quoted = true`, `kirim_edit = true`. Blok teks kosong tetap dilewati
  (deviasi sadar: pesan kosong ditolak WA).
- `ytplay.rs`: `kirim_edit = true`; hanya `section()` seperti sekarang.
- `play2.rs`: `kirim_edit = false` (default); `disclaimer` tetap `""`,
  `skor: Some(999)`, `quoted: false`.

## Test

1. `extract_ie`: teks polos, hyperlink, untrusted, citation, LaTeX,
   escape `\[`, dua link, label bersarang, bracket tak tertutup, emoji
   sebelum link, flag dimatikan.
2. `tokenize_kode`: js (keyword/method/string/number/komentar), python
   `#`, css `prop:`, html `<div>`, `text` → satu `DEFAULT`, bahasa tak
   dikenal, penggabungan token.
3. Section builder: JSON persis (key order) untuk teks+link, tabel
   ber-link (`markdown_cells` hanya saat perlu), kode.
4. Wire: submessage `messageType 5` + `codeMetadata`; `isHeading` baris
   isi `None`; disclaimer `""`; emoji ter-escape sebagai surrogate pair.
5. Pipeline: AIRich mengirim edit; play2 tidak.
6. Oracle: fungsi `extractIE`/`tokenizer` disalin apa adanya dari
   `ourin-md/src/lib/builder.js` ke skrip Node sementara; keluaran JSON
   disalin ke test dengan komentar sumber (pola yang sudah dipakai
   `v2_sections_byte_for_byte_match_the_js_oracle`). Oracle membandingkan
   **submessages dan sections**, termasuk submessage teks yang berisi
   sentinel `{{NIXEL_…}}` setelah `extractIE`.
7. Regresi: `cargo test --release` (semua test lama), lalu
   `cargo clippy --release --all-targets`.

## Risiko

- **Kartu yang sudah render bisa berubah**: perubahan isi hanya menambah
  `inline_entities`/`markdown_cells` dan mengganti submessage kode — hal
  yang memang dilakukan JS. Amplop tidak berubah.
- **Emoji escape** mengubah byte `unifiedResponse.data`; nilainya sama
  setelah di-parse. Diuji round-trip.
- **`AiSubMessage` enum** menyentuh ~15 literal di test/`main.rs`;
  kompilasi akan menangkap semua yang terlewat.
- **`.play2` tanpa edit**: sesuai JS. Kalau ternyata kartu play2 butuh
  edit di klien tertentu, cukup set `kirim_edit = true` di satu tempat.

## Yang diserahkan ke builder

- Nama/penempatan persis modul `airich` dan apakah helper diekspor publik.
- Bentuk test oracle (skrip sementara vs vektor hardcoded).
- Urutan pengerjaan dan pemecahan commit.

## Adendum 2026-11-19 — akar kartu tidak pernah muncul

Laporan owner pasca-v0.2.58: `.ai` hanya reaksi, kartu tidak tampil sama
sekali. Ternyata bukan isi `AIRich`, melainkan klasifikasi stanza di
`vendor/wacore/src/send/classify.rs`: `botForwardedMessage` tidak di-unwrap
dan `richResponseMessage` tidak punya cabang, jadi stanza keluar
`type="media"` TANPA `mediatype`. Server tetap ACK; semua penerima
me-render kosong. Ini persis kegagalan yang dilaporkan upstream
`oxidezap/whatsapp-rust` PR #1366 dan `vinikjkkj/zapo` #272 → #273.

Perbaikan (mengikuti upstream #1366):

- `bot_forwarded_message` masuk daftar `unwrap_message`, sehingga pesan
  dalam yang menentukan tipe; `rich_response_message` dan wrapper kosong
  dijawab `text`.
- Lottie di dalam wrapper tetap `sticker` (kalau tidak: `media` tanpa
  `mediatype`).
- Test di `vendor/wacore/src/send/tests.rs` + regression test
  `ai_rich_classified_as_text_bukan_media` di `lib/oktz-wa/rich.rs`.

Klaim lama di dokumen ini bahwa kartu baru tergambar setelah stanza
`protocolMessage type=14` **salah**. Tipe 14 = `MESSAGE_EDIT`; ruby-baileys
memakainya untuk update streaming, bukan syarat render. ourin-md tidak
pernah mengirimnya. Karena itu `RichAi.kirim_edit`,
`rich_ai_edit_to_message`, dan pengiriman stanza edit dihapus — `.ai` dan
`.ytplay` kini satu stanza, sama seperti JS. Perbaikan yang sama juga
menghidupkan kartu V2/prepared rich/play2 karena semuanya memakai
`botForwardedMessage`.

## Adendum 2026-11-19 (2) — `.ai` tampil, kartu HTML tidak

Setelah klasifikasi stanza dibetulkan, `.ai` benar-benar tampil sebagai
kartu AIRich. Dua plugin lain tetap gagal dengan gelembung *"versi
WhatsApp Anda tidak mendukungnya"*, dan **tidak ada kartu sama sekali**:
`.play2` dan `.ytplay` — keduanya pemakai primitif HTML
(`GenAIaeacdsnwHtmlPrimitive` / `FOAHtmlPrimitiveDemoDONOTUSE`).

Bukti yang mengunci arah:

- Reaksi ✅ tetap muncul dan log bersih → seluruh urutan terkirim sukses.
  Jadi bukan kegagalan upload/kirim; klien yang menolak merender.
- Amplop `.play2` di-encode ulang di Rust dan dibandingkan byte-per-byte
  dengan `proto.Message.encode` Baileys 7.0.0-rc14: **identik** kecuali
  UUID `response_id` (termasuk urutan kunci JSON, `forwardingScore: 999`,
  `forwardOrigin: 4`, `deviceListMetadata: {}`, `messageType: 1/2`).
  Jadi isi `richResponseMessage`/`unifiedResponse` bukan penyebabnya.
- Satu-satunya delta yang tersisa vs JS `sendMusicPlayer`
  (`play2.js:941`) adalah **ID stanza**: JS memakai
  `relayMessage(jid, msg, { messageId: responseId })`, Rust memakai ID
  acak baru.

Perbaikan: `MessageSender::send_with_id` (default delegasi ke
`send_with_result`, jadi stub lama tetap aman). `deliver` memakai
`mci.botMetadata.botResponseId` sebagai ID stanza untuk semua balasan
AIRich, dan `ClientSender` menurunkan itu ke
`SendOptions::with_message_id`. Dengan begitu `response_id` di JSON
unified = `botResponseId` amplop = ID stanza, persis yang diterima klien
dari JS. `.ai` tidak terpengaruh (primitif teks/markdown/tabel tidak
butuh pengikatan respons).

Test: `kartu_ai_dikirim_dengan_id_sama_dengan_bot_response_id`
(`lib/oktz-wa/pipeline.rs`).

Sisa risiko: `.ytplay` di JS memakai `messageId` acak
(`AIRich.send`), jadi belum pernah terbukti render di klien mana pun;
`.ytplay` Rust sekarang ikut memakai `botResponseId` dan perlu retest
perangkat.

## Adendum 2026-11-19 (3) — batas muatan primitif HTML + typename demo

Setelah ID stanza disamakan, `.play2`/`.ytplay` **masih** "tidak
mendukung", sementara `.ai` dan `.ping2` normal (`.ping2` bahkan sudah
membuktikan klien benar-benar merender `unifiedResponse.sections`:
`submessages`-nya kosong). Jadi bug-nya spesifik primitif HTML.

Bukti dari implementasi publik + capture pesan nyata:

- Batas muatan HTML asli klien ~60 KiB. Capture pesan nyata 2026-11-18
  menulis "native limit is 60000" (`Vanta-effect/lib/richhtml.js:25`);
  port whatsmeow memakai `maxRichHTMLPayloadBytes = 64 << 10`
  (`kotonehara/rich_html.go:17`). Di atas batas, klien menampilkan
  "versi WhatsApp Anda tidak mendukungnya" untuk **seluruh** kartu.
- Diukur di sini: `player_html` `.play2` **563 941 B** (audio lagu penuh
  `data:audio/ogg;base64,...` = 560 KB) — jauh di atas batas. `ytplay`
  **25 453 B** (poster 20 KB) — di bawah batas, jadi batas bukan
  penyebabnya.
- Penyebab `.ytplay` yang lain: typenamenya `FOAHtmlPrimitiveDemoDONOTUSE`
  (nama "DEMO" — di repo publik hanya muncul di contoh mainan), sedangkan
  capture nyata + 4 implementasi lain memakai `GenAIaeacdsnwHtmlPrimitive`.
- `messageSecret` BUKAN syarat: Baileys hanya menyuntiknya di
  `generateWAMessageContent` (`Utils/messages.js:543-546`); jalur
  `relayMessage` mentah memakai `encodeWAMessage` (`Utils/generics.js:59`)
  tanpa suntikan, dan capture nyata di atas juga tanpa `messageSecret`.

Perbaikan:

- `.play2`: kartu TIDAK menanam audio lagi (lirik dirender statis, tanpa
  `<script>`); audio penuh tetap dikirim sebagai voice note yang sudah ada.
  Tambah `MAX_HTML_PAYLOAD_BYTES = 60 KiB` + `player_html_aman` (buang
  poster kalau kelebihan) dan guard di handler.
- `.ytplay`: primitive diganti `GenAIaeacdsnwHtmlPrimitive`; muatan
  dibatasi `MAX_HTML_PAYLOAD_BYTES` (poster dibuang kalau kelebihan,
  jatuh ke placeholder).

Catatan desain yang belum diambil: WebView primitif HTML hanya ada di
klien Android dan **tidak punya akses jaringan** (catatan `zaileys`), jadi
`<video src=googlevideo>`/`<audio src=...>` di dalam kartu tidak bisa
streaming. Untuk `ytplay`, memutar video sebaiknya lewat pesan video biasa
(`Reply::Video`, seperti `playvid`), bukan kartu HTML.

## Adendum 2026-11-19 (4) — `messageSecret` wajib untuk `unifiedResponse`

Setelah batas muatan dan typename dibetulkan, kartu HTML **tetap**
ditolak. Pemeriksaan wire menemukan satu beda yang tersisa dan bersih:
balasan AIRich tidak membawa `messageContextInfo.messageSecret`.

Bukti:

- `rich_ai_to_message` (`.ai`, `.play2`, `.ytplay`) → `ai_outer` tanpa
  `messageSecret`. Jalur kirim wacore **tidak** menambahkannya:
  `reporting_token::generate_reporting_token` mengembalikan `None` untuk
  `botForwardedMessage` (diuji langsung; `should_include_reporting_token`
  true, tapi konten token kosong), jadi `extra_context` absen.
- `Reply::tabel_v2` (`.ping2`) MENULIS `messageSecret` sendiri
  (`rich.rs` V2 / `rich-messages.js:1059`) — dan kartu `.ping2` adalah
  satu-satunya balasan lintas-plugin yang isinya HANYA
  `unifiedResponse.sections` (submessages kosong) dan **terbukti render**.
- Generator publik yang jalan selalu mengirimnya: GamesWhats
  `html-transport.ts:40` (`messageSecret: randomBytes(32)`), dan catatan
  pemilik sendiri di commit `0634f3f`: "messageContextInfo wajib
  messageSecret+threadId+deviceListMetadata berisi (pola sendTableV2
  terbukti)" — persis pola V2 yang render.
- Sumber komunitas memperingatkan hal yang sama: "menambahkan metadata
  yang tidak perlu bisa membuat klien menampilkan 'Update WhatsApp'"
  (GamesWhats `WHATSAPP_HTML_TRANSPORT.md`), jadi amplop harus persis
  pola yang terbukti.

Perbaikan: `ai_outer` menulis `message_secret: secret_32()` (32 byte
acak, baru per pesan) untuk SEMUA balasan AIRich. Test
`ai_rich_membawa_message_secret_32_byte` juga men-decode plaintext
penerima hasil `MessageUtils::encode_dm_plaintexts` untuk memastikan
`messageSecret` dan `botMetadata` dua-duanya sampai (merge reporting
tidak menghapus salah satunya).

## Adendum 2026-11-19 (5) — kode klien + diagnostik `.cekkartu`

Riset lanjutan membaca kode klien WhatsApp (dump `wa-diff`) dan pengukuran
di perangkat (repo `elaina-baileys`, `Tederby/Whatsapp-Multi-Bots`):

- `WAWebParseRichResponseMessageProto.js` — jalur yang benar-benar
  menghasilkan "versi WhatsApp tidak mendukung" (`MSG_TYPE.UNKNOWN`) untuk
  pengirim nomor biasa: `u(contextInfo)` harus true
  (`isForwarded === true` DAN `forwardedAiBotMessageInfo.botJid != null`),
  lalu `if (u(ctx) && !isRichResponseForwardReceivingEnabled()) return
  UNKNOWN`. Amplop kita sudah memenuhi syarat pertama; `.ai`/`.ping2`
  render, jadi flag penerimaan di klien owner menyala.
- `WAWebRichResponseParseUtils.parseUnifiedResponse` = `TextDecoder` +
  `JSON.parse` **atas byte mentah** `unifiedResponse.data` — jadi JSON
  mentah kita benar (bukan base64).
- Batas ukuran: tidak ada validasi di klien; ambang empiris ±1,35 MB
  (diukur per perangkat). Batas 60 KiB yang dipakai banyak builder cuma
  tebakan. **Ukuran bukan penyebab.**
- `data:` gambar justru satu-satunya yang bisa dimuat di WebView primitif;
  `https://` tidak pernah (walau ada di `trusted_sources`). `<script>` dan
  `<style>` diterima; `eval`/WASM diblok.
- `E()` di parser submessages melempar "No fragments parsed" kalau
  `submessages` kosong — kartu `.ytplay` (tanpa submessage) masuk jalur itu.

Karena kandidat "amplop" sudah tersingkir dan kandidatnya tinggal isi
payload/renderer, satu perintah diagnostik dibuat: `.cekkartu`
(`plugin/src/tools/cekkartu.rs`) mengirim sembilan kartu yang hanya
berbeda satu variabel (A baseline, B `img` data-URI, C `<script>`,
D `trusted_sources` terisi, E dokumen utuh, F non-ASCII, G payload ~30 KB,
H section dibungkus + `title`, I `submessages` kosong). Jawaban owner
(huruf mana yang tampil) menentukan perbaikan tanpa menebak lagi.
