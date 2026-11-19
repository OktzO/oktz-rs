# Sistem AIRich Rust (`.ai` / `.play2` / `.ytplay`) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Isi kartu AIRich dari `.ai`, `.ytplay`, `.play2` sama dengan ourin-md (`extractIE`, tokenizer kode ber-highlight, `markdown_cells`) tanpa mengubah amplop wire yang sudah terbukti render; `.play2` berhenti mengirim stanza edit.

**Architecture:** Modul baru `oktz_plugin_api::airich` memegang port murni `extractIE` + `tokenizer` + builder `RichAi` (`teks`/`kode`/`tabel`/`section`). Model `AiSubMessage` jadi enum 3 varian; `oktz-wa::rich` memetakan varian ke proto. Pipeline memakai flag `RichAi::kirim_edit` untuk memutuskan stanza `protocolMessage type=14`.

**Tech Stack:** Rust workspace (`oktz-plugin-api`, `oktz-wa`, `oktz-plugins`), `serde_json` (`preserve_order` sudah aktif), `whatsapp-rust 0.7.0` / `waproto 0.7.0` (buffa codegen).

**Spec:** `docs/superpowers/specs/2026-11-17-airich-system-design.md`

## Global Constraints

- **Paritas ketat ourin-md v4.6** untuk isi: tanpa alias bahasa (`js`/`ts`/`py` TIDAK dipetakan — blok ` ```js ` memang tanpa keyword), key JSON & urutannya persis JS, `STR` bukan `STRING`.
- Amplop tetap kontrak v4.7 yang sudah render (`botJid 867051314767696@bot`, `forwardingScore`, `botResponseId`, tanpa `verificationMetadata`, stanza edit). **Jangan sentuh** nilai-nilai itu selain yang disebut di plan.
- Jangan edit `ourin-md/`, `scripts/paritas-audit.py`, `docs/paritas/*`, `vendor/*`.
- Komentar kode Bahasa Indonesia, hanya untuk hal non-obvious + rujukan baris JS (`builder.js:1048`). Penyimpangan sadar → `ponytail:` + alasan.
- Diff sesingkat mungkin; tanpa abstraksi yang belum dipakai plugin.
- Urutan perintah verifikasi: `cargo build --release` → `cargo test --release` → `cargo clippy --release --all-targets`.

## Review Focus

1. **Byte sections == JS** — `extract_ie` + tokenizer + tabel (`markdown_cells`) harus menghasilkan JSON identik, termasuk urutan key dan angka (`100` bukan `100.0`). Test: Task 1/2/3 + ornamen end-to-end Task 5.
2. **Wire kode** — submessage `messageType: 5` + `codeMetadata` dengan `highlightType` enum (0–5), bukan teks ber-fence. Test: Task 4.
3. **`.play2` tidak kirim edit** — satu-satunya perubahan perilaku kirim; jangan sampai `.ai`/`.ytplay` ikut berhenti. Test: Task 6.
4. **Emoji** — `unifiedResponse.data` harus `\ud83d\ude80` (surrogate pair), bukan `\u1f680` (escape tidak valid). Test: Task 4.
5. **Regresi** — semua test lama hijau berarti amplop tidak berubah.

## Deviasi yang diterima (jangan "diperbaiki" di plan ini)

- Blok teks hasil `trim()` kosong dilewati (`blocks_ke_reply` sekarang); JS tetap `addText("")`. Pesan teks kosong ditolak WA.
- `Number(x)` JS menerima `0x10`/`Infinity`; port hanya `f64::parse` + `|| default`. Nilai LaTeX dari AI/manusia tidak memakai bentuk itu.
- `.play2` JS mengirim `relayMessage(..., { messageId: responseId })`; pipeline Rust memakai message id dari socket. Di luar cakupan isi AIRich, tidak diubah (play2 tidak mengirim edit sehingga tidak ada yang bergantung pada id itu).

---

### Task 1: `extract_ie` — port `builder.js:40-212`

**Files:**
- Create: `lib/oktz-plugin-api/airich.rs`
- Modify: `lib/oktz-plugin-api/lib.rs` (tambah `pub mod airich;` + re-export setelah blok `rich`)

**Interfaces (dipakai Task 2–5):**

```rust
#[derive(Debug, Clone, Copy)]
pub struct Flag { pub extract: bool, pub hyperlink: bool, pub citation: bool, pub latex: bool }
impl Default for Flag { fn default() -> Self { Self { extract: true, hyperlink: true, citation: true, latex: true } } }

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HasilEkstrak { pub teks: String, pub entities: Vec<serde_json::Value> }

pub fn extract_ie(teks: &str, flag: &Flag) -> HasilEkstrak
```

- [ ] **Step 1: Test gagal** — `mod tests` di `airich.rs`, satu helper:

  ```rust
  fn cek(input: &str, flag: Flag, teks: &str, entities: &str) {
      let h = extract_ie(input, &flag);
      assert_eq!(h.teks, teks);
      assert_eq!(serde_json::to_string(&h.entities).unwrap(), entities);
  }
  ```

  Vektor persis keluaran oracle JS (fungsi `extractIE` disalin apa adanya dari `builder.js` ke skrip Node sementara — skrip boleh dibuat ulang, **keluaran di tabel ini yang mengikat**):

  | input | teks hasil | entities |
  |---|---|---|
  | `[Vue](https://vuejs.org)` | `{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}` | `[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}]` |
  | `[x](!https://a.b)` | `{{NIXEL_HYPERLINK_0}}https://a.b{{/NIXEL_HYPERLINK_0}}` | `[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"x","is_trusted":false,"url":"https://a.b","__typename":"GenAIInlineLinkItem"}}]` |
  | `[](https://a.b)` | `{{NIXEL_CITATION_0}}https://a.b{{/NIXEL_CITATION_0}}` | `[{"key":"NIXEL_CITATION_0","metadata":{"reference_id":1,"reference_url":"https://a.b","reference_title":"https://a.b","reference_display_name":"https://a.b","sources":[],"__typename":"GenAISearchCitationItem"}}]` |
  | `[x|10|20|30|40]<https://l>` | `{{NIXEL_LATEX_0}}x{{/NIXEL_LATEX_0}}` | `[{"key":"NIXEL_LATEX_0","metadata":{"latex_expression":"x","latex_image":{"url":"https://l","width":10,"height":20},"font_height":30,"padding":40,"__typename":"GenAILatexItem"}}]` |
  | `[x]<https://l>` (default angka) | idem | `...{"url":"https://l","width":100,"height":100},"font_height":83.333333333333,"padding":15...` |
  | `[x|0|0|0|0]<https://l>` | idem | idem (`0` → default, karena `Number(x) || 100`) |
  | `[x\|1.5\|2.25]<https://l>` | idem | `width:1.5, height:2.25, font_height:83.333333333333, padding:15` |
  | `\[bukan](https://x) [ok](https://y)` | `\[bukan](https://x) {{NIXEL_HYPERLINK_0}}https://y{{/NIXEL_HYPERLINK_0}}` | 1 entity, `display_name":"ok"` |
  | `[a [b] c](https://x)` | `[a {{NIXEL_HYPERLINK_0}}https://x{{/NIXEL_HYPERLINK_0}}` | `display_name":"b] c"` |
  | `[a](https://x [b](https://y)` | `[a](https://x {{NIXEL_HYPERLINK_0}}https://y{{/NIXEL_HYPERLINK_0}}` | `display_name":"b"` |
  | `🚀 [Vue](https://vuejs.org)` | `🚀 {{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}` | 1 hyperlink |
  | `[Vue](https://vuejs.org) sisa` + `hyperlink:false` | input apa adanya | `[]` |
  | `[x|10]<https://l>` + `latex:false` | input apa adanya | `[]` |
  | `[](https://a.b)` + `citation:false` | input apa adanya | `[]` |
  | `[a](https://x)` + `extract:false` | input apa adanya | `[]` |

  Tambahan yang wajib ada: dua link berurutan → `HYPERLINK_0` + `HYPERLINK_1` (indeks urut), dan `[]()`, `[` tanpa penutup → teks utuh tanpa entity.

- [ ] **Step 2:** `cargo test -p oktz-plugin-api --lib airich` → FAIL (fungsi belum ada).

- [ ] **Step 3: Implementasi** — port literal, indeks per `char` (`Vec<char>`):

  - `flag.extract == false` → teks apa adanya, `entities` kosong.
  - Loop `i`: `[` dan `chars[i-1] != '\\'` → push `i` ke `stack`; `]` yang diikuti `(`/`<` → `pop()` (kosong = `continue`).
  - `open = chars[i+1]`, `close = ')'` atau `'>'`, `end = i+2`, `depth = 1`; selama `end < n && depth > 0`: `open`/`close` yang bukan escape (`chars[end-1] != '\\'`) menaik/menurunkan depth, `end += 1`; `depth != 0` → `continue` (stack sudah ter-pop, teks asli dipertahankan).
  - `raw = chars[start+1..i].trim()`, `url = chars[i+2..end-1].trim()`.
  - LaTeX: `raw.split('|')` → `txt` (default `""`), `width/height/font_height/padding`; tag `{{KEY}}{txt || "image"}{{/KEY}}`, `KEY = NIXEL_LATEX_{n}` (`n` mulai 0).
  - Hyperlink (`raw` non-kosong): `!url.starts_with('!')` = `is_trusted`; `!` dibuang dari url; tag `{{KEY}}{url}{{/KEY}}`; `NIXEL_HYPERLINK_{n}`.
  - Citation (`raw` kosong): `reference_id = citation_index` (mulai 1, naik setelah dipakai), tag `{{KEY}}{url}{{/KEY}}`; `NIXEL_CITATION_{reference_id-1}`.
  - Jenis dimatikan (`flag`) → `continue` **tanpa** menulis `result`/`last` (teks asli tertinggal), persis `continue` JS.
  - `result += chars[last..start] + tag; last = end; i = end - 1;`
  - Entity memakai `json!` dengan urutan key JS (lihat tabel). Angka LaTeX lewat helper:

    ```rust
    /// `JSON.stringify` JS: bilangan bulat tanpa `.0`.
    fn angka_js(v: f64) -> Value {
        if v.fract() == 0.0 && v.abs() <= 9_007_199_254_740_992.0 {
            Value::from(v as i64)
        } else {
            serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
        }
    }
    ```

    Nilai = `parse::<f64>()` sukses & `!= 0.0` & finite, kalau tidak pakai default (`100`, `100`, `83.333333333333`, `15`).

- [ ] **Step 4:** `cargo test -p oktz-plugin-api --lib airich` → PASS.
- [ ] **Step 5:** Commit `feat(plugin-api): port extractIE (inline entity link/sitasi/LaTeX)`.

### Task 2: `tokenize_kode` — port `builder.js:1540-2203`

**Files:**
- Modify: `lib/oktz-plugin-api/airich.rs`
- Modify: `lib/oktz-plugin-api/lib.rs` (re-export `Highlight`, `KodeToken`, `tokenize_kode`)

**Interfaces (dipakai Task 3–5):**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight { Default, Keyword, Method, Str, Number, Comment }
impl Highlight { pub fn nama(&self) -> &'static str } // "DEFAULT" | "KEYWORD" | "METHOD" | "STR" | "NUMBER" | "COMMENT"

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KodeToken { pub isi: String, pub jenis: Highlight }

pub fn tokenize_kode(kode: &str, bahasa: &str) -> Vec<KodeToken>
```

- [ ] **Step 1: Test gagal** — vektor dari oracle (fungsi `tokenizer` disalin apa adanya dari `builder.js`), keluaran `unified_codeBlock` dipetakan ke `(isi, jenis)`:

  - `tokenize_kode("const app = createApp({});\napp.mount(\"#app\"); // hi", "javascript")` →
    `[("const",Keyword),(" app = ",Default),("createApp",Method),("({});\napp.",Default),("mount",Method),("(",Default),("\"#app\"",Str),("); ",Default),("// hi",Comment)]`
  - `("def f(x):\n  # kom\n  return 1", "python")` →
    `[("def",Keyword),(" ",Default),("f",Method),("(x):\n  ",Default),("# kom",Comment),("\n  ",Default),("return",Keyword),(" ",Default),("1",Number)]`
  - `(".a { color: red; }", "css")` → `[(".a { ",Default),("color",Keyword),(": red; }",Default)]`
  - `("<div class=\"x\">hi</div>", "html")` → `[("<",Default),("div",Keyword),(" class=",Default),("\"x\"",Str),(">hi</",Default),("div",Keyword),(">",Default)]`
  - `("halo", "text")` → `[("halo",Default)]`; `("", "text")` → `[("",Default)]` (satu token kosong, bukan vektor kosong).
  - `("let x = \"1\"", "zzz")` → `[("let x = ",Default),("\"1\"",Str)]` (tanpa keyword, string tetap).
  - `("a() color: x", "js")` → `[("a",Method),("() color: x",Default)]` — **tanpa alias**, `js` bukan `javascript`.
  - `("a() color: x", "Text")` → `[("a",Method),("() color: x",Default)]`; versi `"text"` → `[("a() color: x",Default)]` (cabang early-return peka huruf besar).
  - `("a() color: x", "CSS")` → `[("a",Method),("() color: x",Default)]`; versi `"css"` → `[("a",Method),("() ",Default),("color",Keyword),(": x",Default)]` (cabang css pakai `lang` mentah, bukan lowercase).
  - Gabung token: `("a  b", "javascript")` → satu token `("a  b", Default)`.

- [ ] **Step 2:** `cargo test -p oktz-plugin-api --lib airich` → FAIL.

- [ ] **Step 3: Implementasi**:

  - `keywords` = fungsi `kata_kunci(lang_lower: &str) -> &'static [&'static str]` berisi 12 set dari `builder.js:1541-2076` (javascript, typescript, python, java, golang, c, cpp, php, rust, html, bash, markdown). Isi harus **disalin**, bukan ditebak — beda satu kata = beda output.
  - Early return: `bahasa.is_empty() || bahasa == "txt" || bahasa == "text" || bahasa == "plaintext"` → `vec![KodeToken { isi: kode.into(), jenis: Highlight::Default }]` (bahkan saat `kode` kosong).
  - `spasi_js(c)` = set `\s` JS persis: `\t \n \u{b} \u{c} \r ' ' \u{a0} \u{1680} \u{2000}..=\u{200a} \u{2028} \u{2029} \u{202f} \u{205f} \u{3000} \u{feff}`.
  - `is_identifier(c)` per `bahasa.to_lowercase()`: css → `[A-Za-z0-9_$-]`, html → `[A-Za-z0-9_$:-]`, default → `[A-Za-z0-9_$]`.
  - Urutan cabang literal: spasi → komentar (`//`; `#` hanya bila `bahasa == "python" || bahasa == "bash"` — raw) → string `" '\`` (escape `\` lewati 2 char) → angka `[0-9]` lalu `[0-9._]` → identifier start `[A-Za-z_$]` lalu `is_identifier` → fallback satu char `Default`.
  - Jenis kata: `keywords.contains(word)` → Keyword; lalu **`bahasa == "css"`** (raw) lookahead spasi lalu `:` → Keyword; **`bahasa == "html"`** (raw) lookbehind spasi lalu `<` atau `</` → Keyword; kalau masih Default, lookahead spasi lalu `(` → Method.
  - `push` menggabungkan token bila `jenis` sama dengan token terakhir.
  - Indeks per `char` (bukan byte); aman karena semua predikat ASCII.

- [ ] **Step 4:** `cargo test -p oktz-plugin-api --lib airich` → PASS.
- [ ] **Step 5:** Commit `feat(plugin-api): port tokenizer kode AIRich (highlight per token)`.

### Task 3: Model + builder `RichAi`

**Files:**
- Modify: `lib/oktz-plugin-api/rich.rs`
- Modify: `lib/oktz-plugin-api/airich.rs` (`impl RichAi`)
- Modify: `lib/oktz-plugin-api/lib.rs`

**Interfaces (dipakai Task 4–6):**

```rust
pub enum AiSubMessage {
    Teks { teks: String },
    Tabel { judul: String, baris: Vec<Vec<String>>, heading: bool },
    Kode { bahasa: String, blok: Vec<KodeToken> },
}

pub struct RichAi {
    pub submessages: Vec<AiSubMessage>,
    pub sections: Vec<serde_json::Value>,
    pub disclaimer: String,   // selalu ditulis; default ""
    pub quoted: bool,
    pub skor: Option<u32>,
    pub kirim_edit: bool,     // default false
}

impl RichAi {
    pub fn teks(&mut self, teks: &str) -> &mut Self;
    pub fn kode(&mut self, bahasa: &str, kode: &str) -> &mut Self;
    pub fn tabel(&mut self, baris: Vec<Vec<String>>) -> &mut Self;
    pub fn section(&mut self, section: serde_json::Value) -> &mut Self;
}
```

- [ ] **Step 1: Test gagal** — di `airich.rs`:

  - `RichAi::default().teks("Halo dunia")` → `submessages == [Teks { teks: "Halo dunia" }]`; section JSON persis `{"view_model":{"primitive":{"text":"Halo dunia","__typename":"GenAIMarkdownTextUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}`.
  - `.teks("[Vue](https://vuejs.org) bagus")` → section persis oracle:
    `{"view_model":{"primitive":{"text":"{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}} bagus","inline_entities":[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}],"__typename":"GenAIMarkdownTextUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}`
  - `.kode("js", "const app = createApp({});")` → submessage `Kode` dengan blok hasil tokenizer `js` (bukan keyword); section `{"view_model":{"primitive":{"language":"js","code_blocks":[{"content":"const app = ","type":"DEFAULT"},{"content":"createApp","type":"METHOD"},{"content":"({});","type":"DEFAULT"}],"__typename":"GenAICodeUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}`.
  - `.tabel(vec![vec!["A","B"], vec!["1"]])` → submessage `Tabel { judul: "", baris: [["A","B"],["1",""]], heading: true }`; section cells dipadding `""`; `markdown_cells` **tidak ada**.
  - `.tabel` dengan sel `[Vue](https://vuejs.org)` → section persis oracle `TABLE link` di bawah; `markdown_cells` berisi satu entri per sel: `{"text":"Vue"}`, dan sel link `{"text":"{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}","inline_entities":[...]}`.
  - `.tabel(Vec::new())` → no-op (tidak panik).
  - `.section(json!({"a":1}))` → `sections` berisi nilai itu apa adanya.

  Vektor oracle tabel (dari `/tmp/opencode/airich-sections.mjs`):

  ```
  input: [["Nama","Link"],["Vue","[Vue](https://vuejs.org)"],["React","https://react.dev"]]
  section: {"view_model":{"primitive":{"rows":[{"is_header":true,"cells":["Nama","Link"]},{"is_header":false,"cells":["Vue","[Vue](https://vuejs.org)"],"markdown_cells":[{"text":"Vue"},{"text":"{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}","inline_entities":[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}]}]},{"is_header":false,"cells":["React","https://react.dev"]}],"__typename":"GenATableUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}
  submessage tabel: {"items":["Nama","Link"],"isHeading":true} lalu {"items":["Vue","[Vue](https://vuejs.org)"]} + {"items":["React","https://react.dev"]}
  ```

- [ ] **Step 2:** `cargo test -p oktz-plugin-api --lib` → FAIL (tipe lama).

- [ ] **Step 3: Implementasi**:

  - `AiSubMessage` jadi enum (hapus `#[derive(Default)]`; tidak ada pemakai `AiSubMessage::default()`), dengan `use crate::airich::KodeToken;`. Doc komentar `AiSubMessage` (yang bilang "code butuh tokenizer ... bukan milik task ini") diganti: tiga varian inilah yang dipakai `builder.js`.
  - `RichAi`: `disclaimer: String`, `kirim_edit: bool`; doc comment `disclaimer` diperbarui (kedua lineage selalu menulis `messageDisclaimerText`).
  - Test lama di `rich.rs` yang menyentuh bentuk lama wajib ikut: `ai_submessage_table_carries_title_rows_and_heading_flag` (destructure varian `Tabel`), dan `ai_and_prepared_carry_the_same_shape_but_stay_distinct_types` (`AiSubMessage::Teks`, `disclaimer: "d".into()`, bandingkan `Some(ai.disclaimer.clone())` vs `p.disclaimer`).
  - Builder `teks`: `extract_ie(teks, &Flag::default())` → submessage `Teks` (teks **setelah** extract) + section `{text, inline_entities?, __typename: GenAIMarkdownTextUXPrimitive}` dengan urutan key `text`, `inline_entities`, `__typename`.
  - Builder `kode`: `tokenize_kode(kode, bahasa)`; section `{language, code_blocks: [{content, type}], __typename: GenAICodeUXPrimitive}` (urutan key itu); submessage menyimpan token mentah.
  - Builder `tabel`: **padding dulu** ke `max` lebar semua baris (termasuk header) dengan `""`; `unified_rows` = per baris `{is_header, cells}` + `markdown_cells` hanya bila minimal satu sel menghasilkan entity; `markdown_cells` = `{text}` / `{text, inline_entities}` per sel. Submessage `Tabel { judul: "", baris: padded, heading: true }`.
  - Builder `section`: push apa adanya.
  - Kunci JSON yang butuh urutan tertentu dibangun dengan `serde_json::Map` (bukan `json!` bertingkat) supaya `is_header` → `cells` → `markdown_cells` pasti urut.

- [ ] **Step 4:** `cargo test -p oktz-plugin-api --lib` → PASS.
- [ ] **Step 5:** Commit `feat(plugin-api): AiSubMessage enum + builder teks/kode/tabel AIRich`.

### Task 4: Wire kode + disclaimer + escape UTF-16

**Files:**
- Modify: `lib/oktz-wa/rich.rs`

**Interfaces:**
- Consumes: `AiSubMessage::{Teks,Tabel,Kode}`, `Highlight`, `RichAi::disclaimer: String`.
- Produces: submessage `messageType: 5` + `codeMetadata`; `messageDisclaimerText` selalu `Some`; `stringify_escaped` (ganti `escape_ascii_7f`).

- [ ] **Step 1: Test gagal** — di `mod tests`:

  - `rich_ai_to_message` dengan `AiSubMessage::Kode { bahasa: "javascript", blok: vec![KodeToken{isi:"const",jenis:Keyword}, KodeToken{isi:" x",jenis:Default}] }` → submessage pertama `message_type == AI_RICH_RESPONSE_CODE`, `code_metadata.code_language == Some("javascript")`, `code_blocks[0].highlight_type == Some(AI_RICH_RESPONSE_CODE_HIGHLIGHT_KEYWORD)`, `code_blocks[1].highlight_type == Some(..._DEFAULT)`.
  - `RichAi::default()` → `bot_metadata(msg).message_disclaimer_text == Some("")`; `RichAi { disclaimer: "Syarat".into(), .. }` → `Some("Syarat")`.
  - Emoji: `RichAi { sections: vec![json!({"text":"🚀"})], .. }` → `unified_data` berisi ASCII `\ud83d\ude80` (cari substring), dan `serde_json::from_slice::<Value>(&unified_data)` berhasil dengan nilai `"🚀"`.
  - Nama fungsi di test lama: `escape_ascii_7f` diganti `stringify_escaped` (langsung unit-test: `stringify_escaped("a\u{7f}b🚀") == "a\\u007fb\\ud83d\\ude80"`).

- [ ] **Step 2:** `cargo test -p oktz-wa --lib rich` → FAIL.

- [ ] **Step 3: Implementasi**:

  - Import (nama sudah diverifikasi dari `OUT_DIR/whatsapp.rs` buffa): `whatsapp_rust::waproto::whatsapp::ai_rich_response_code_metadata::{AIRichResponseCodeBlock, AIRichResponseCodeHighlightType}`. Field: `code_metadata: MessageField<AIRichResponseCodeMetadata>`, `code_language: Option<String>`, `code_blocks: Vec<AIRichResponseCodeBlock>`, `highlight_type: Option<AIRichResponseCodeHighlightType>`, `code_content: Option<String>`.
  - `ai_submessages` → `match` enum: `Teks` = perilaku `sub_text` sekarang; `Tabel` = perilaku sekarang (`is_heading` hanya indeks 0); `Kode` = bangun `AIRichResponseCodeMetadata { code_language, code_blocks }` dengan pemetaan `Highlight` → enum proto. Doc komentar "code butuh tokenizer ... bukan milik task ini" dihapus.
  - `ai_outer`: `message_disclaimer_text: Some(input.disclaimer.clone())`.
  - `escape_ascii_7f` → `stringify_escaped`: iterasi `char`; `c as u32 > 0xffff` → `encode_utf16` lalu escape **tiap** unit (`\u{:04x}`); `0x7f..=0xffff` → satu escape; selain itu push char. Doc: `builder.js`/ruby-baileys `stringifyEscaped` (`MessageBuilder/index.js:561`) — `JSON.stringify` lalu replace per **UTF-16 code unit**; `char`-based membuat `\u1f680` yang bukan JSON valid.
  - Perbarui test lama `ai_rich_disclaimer_lands_in_bot_metadata` dan `play2_send_music_player_contract_is_not_airich` (`disclaimer: String::new()` / `"Syarat pakai".into()`), plus semua literal `AiSubMessage` di test ini.

- [ ] **Step 4:** `cargo test -p oktz-wa --lib rich` → PASS.
- [ ] **Step 5:** Commit `feat(wa): submessage kode AIRich + escape UTF-16 + disclaimer selalu ditulis`.

### Task 5: `.ai` pakai builder (end-to-end oracle)

**Files:**
- Modify: `plugin/src/ai/ai.rs`
- Modify: `src/main.rs` (varian debug `richai`)

**Interfaces:**
- Consumes: `RichAi::{teks,kode,tabel}`, `AiSubMessage`.
- Produces: `blocks_ke_reply` tanpa section buatan sendiri.

- [ ] **Step 1: Test gagal** — di `mod tests`:

  - Ganti `kode_jadi_section_code_satu_token_default` → `kode_jadi_submessage_kode`: `pecah("```js\nconsole.log(1)\n```")` → `RichAi` dengan satu `AiSubMessage::Kode { bahasa: "js", blok }`, `blok[1] == KodeToken { isi: "log", jenis: Method }`, section `language == "js"`, `__typename == GenAICodeUXPrimitive`; **tidak ada** submessage berisi fence ` ``` `.
  - `teks_dan_tabel_berurutan_jadi_satu_richai` & `reply_tabel_memakai_baris_pertama_sebagai_header` pindah ke pattern-match enum; `blok_teks` helper filter `AiSubMessage::Teks`.
  - **Oracle end-to-end** (paling penting): satu test `sections_sama_dengan_oracle_js` dengan input:

    ```
    Halo! Ini **contoh** dari [Vue](https://vuejs.org).
    | Nama | Link |
    |---|---|
    | Vue | [Vue](https://vuejs.org) |
    | React | https://react.dev |
    ```js
    const app = createApp({});
    app.mount("#app"); // hi
    ```
    Penutup.
    ```

    `serde_json::to_string(&rich.sections)` harus sama persis dengan `sections` di **Lampiran A** (satu baris). Tambahan assert submessage: teks pertama memuat `{{NIXEL_HYPERLINK_0}}`, tabel 3 baris dengan `heading`, kode `bahasa == "js"` + 8 blok, teks terakhir `"Penutup."`.

- [ ] **Step 2:** `cargo test -p oktz-plugins --lib ai:` → FAIL.

- [ ] **Step 3: Implementasi**:

  - Hapus `section_teks`, `section_kode`, `section_tabel`, `unified_rows` (pindah ke plugin-api).
  - `blocks_ke_reply`: `Blok::Teks(t)` kosong → `continue`; selain itu `rich.teks(&t)`, `rich.kode(&bahasa, &kode)`, `rich.tabel(rows)`. Hapus padding di sini (builder yang padding) dan hapus bungkus fence pada submessage kode.
  - Setelah loop: `if rich.submessages.is_empty() { return Reply::None; }`, lalu `rich.quoted = true; rich.kirim_edit = true;`.
  - Header modul: buang dua paragraf `ponytail:` (tokenizer & extractIE sudah diport).
  - `src/main.rs`: varian `richai` pakai `AiSubMessage::Teks` / `AiSubMessage::Tabel`.

- [ ] **Step 4:** `cargo test -p oktz-plugins --lib ai:` → PASS.
- [ ] **Step 5:** Commit `feat(ai): pakai builder AIRich (entity, highlight kode, markdown_cells)`.

### Task 6: `.play2` tanpa edit, `.ytplay` dengan edit

**Files:**
- Modify: `plugin/src/search/play2.rs`
- Modify: `plugin/src/search/ytplay.rs`
- Modify: `lib/oktz-wa/pipeline.rs`

**Interfaces:**
- Consumes: `RichAi::kirim_edit`.
- Produces: `deliver_reply` hanya mengirim stanza edit bila `TabelAi(_) && kirim_edit`.

- [ ] **Step 1: Test gagal** — `pipeline.rs` `mod tests`, pakai pola `entry_named` + `pipeline_for` + `inbound_n` (lihat test `mutation_from_one_message_is_visible_to_the_next_through_the_real_store`):

  ```rust
  #[tokio::test]
  async fn airich_mengirim_stanza_edit() {
      let lookup = entry_named("kartu", |_c| Reply::TabelAi(Box::new(RichAi {
          kirim_edit: true,
          submessages: vec![AiSubMessage::Teks { teks: "t".into() }],
          ..Default::default()
      })));
      let (p, sender) = pipeline_for(EMPTY_STORE, lookup);
      let m = inbound_n(".kartu", MEMBER, MEMBER, false);
      assert_eq!(p.process(&m, 10_000).await.unwrap(), Outcome::Sent);
      assert_eq!(sender.sent.lock().unwrap().len(), 2, "kartu + edit");
      let edit = sender.sent.lock().unwrap()[1].1.clone();
      let proto = edit.bot_forwarded_message.as_option().unwrap()
          .message.as_option().unwrap().protocol_message.as_option().unwrap();
      assert_eq!(proto.r#type, Some(wa::message::protocol_message::Type::MESSAGE_EDIT));
  }

  #[tokio::test]
  async fn play2_tanpa_kirim_edit_hanya_satu_kiriman() { /* kirim_edit: false → 1 */ }
  ```

  Nama test berbeda; `entry_named`/`pipeline_for`/`inbound_n` sudah ada di modul yang sama.

- [ ] **Step 2:** `cargo test -p oktz-wa --lib pipeline` → FAIL (test play2 dapat 2 kiriman).

- [ ] **Step 3: Implementasi**:

  - Pipeline: `if let Reply::TabelAi(ai) = other && ai.kirim_edit { ... }` — komentar diperbarui: edit hanya milik `AIRich.send` (`.ai`, `.ytplay`), bukan `sendMusicPlayer` play2. Sekalian ubah literal `AiSubMessage { text: ..., tabel: None }` di test `setiap_balasan_rich_memakai_jalur_wirenya_sendiri` (sekitar `pipeline.rs:6592`) ke varian `Teks`.
  - `ytplay.rs`: `kirim_edit: true` di literal `RichAi`.
  - `play2.rs`: `disclaimer: String::new()`, `kirim_edit: false` (atau biarkan default; tulis eksplisit karena kontrak), komentar diperbarui.

- [ ] **Step 4:** `cargo test -p oktz-wa --lib pipeline` + `cargo test -p oktz-plugins --lib search::` → PASS.
- [ ] **Step 5:** Commit `fix(wa): stanza edit AIRich hanya untuk RichAi.kirim_edit (play2 tidak)`.

### Task 7: Verifikasi penuh

- [ ] **Step 1:** `cargo build --release` → sukses.
- [ ] **Step 2:** `cargo test --release` → semua hijau (`--test live_route` butuh jaringan; kalau offline, catat test yang di-skip, jangan dihapus).
- [ ] **Step 3:** `cargo clippy --release --all-targets` → tanpa warning baru.
- [ ] **Step 4:** `python scripts/paritas-audit.py` → tidak menambah temuan baru. Catatan: `docs/rust-rewrite/STUBS.md` masih memuat dua entri `ponytail:` `ai.rs` yang sudah dihapus dan tidak ada skrip generator di repo — biarkan (di luar scope), jangan mengarang angka baru.
- [ ] **Step 5:** Cek cepat `./target/release/oktz plugins | head` (registry tidak kosong — `all_plugins()` > 0).
- [ ] **Step 6:** Commit `chore(airich): verifikasi build/test/clippy` bila ada sisa perubahan (mis. doc), lalu laporkan hasil nyata (angka test, bukan klaim).

---

## Lampiran A — keluaran oracle end-to-end

Dihasilkan skrip Node sementara yang menyalin `extractIE`, `tokenizer`, `toTableMetadata`, `newLayout` dari `builder.js` + loop `plugins/ai/ai.js:77-137`. Skripnya tidak disimpan; bagian `sections` (satu baris) di bawah adalah kontrak test Task 5:

```json
[{"view_model":{"primitive":{"text":"Halo! Ini **contoh** dari {{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}.","inline_entities":[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}],"__typename":"GenAIMarkdownTextUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}},{"view_model":{"primitive":{"rows":[{"is_header":true,"cells":["Nama","Link"]},{"is_header":false,"cells":["Vue","[Vue](https://vuejs.org)"],"markdown_cells":[{"text":"Vue"},{"text":"{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}","inline_entities":[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}]}]},{"is_header":false,"cells":["React","https://react.dev"]}],"__typename":"GenATableUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}},{"view_model":{"primitive":{"language":"js","code_blocks":[{"content":"const app = ","type":"DEFAULT"},{"content":"createApp","type":"METHOD"},{"content":"({});\napp.","type":"DEFAULT"},{"content":"mount","type":"METHOD"},{"content":"(","type":"DEFAULT"},{"content":"\"#app\"","type":"STR"},{"content":"); ","type":"DEFAULT"},{"content":"// hi","type":"COMMENT"}],"__typename":"GenAICodeUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}},{"view_model":{"primitive":{"text":"Penutup.","__typename":"GenAIMarkdownTextUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}]
```

Bagian `submessages`:

```json
[{"messageType":2,"messageText":"Halo! Ini **contoh** dari {{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}."},{"messageType":4,"tableMetadata":{"title":"","rows":[{"items":["Nama","Link"],"isHeading":true},{"items":["Vue","[Vue](https://vuejs.org)"]},{"items":["React","https://react.dev"]}]}},{"messageType":5,"codeMetadata":{"codeLanguage":"js","codeBlocks":[{"codeContent":"const app = ","highlightType":0},{"codeContent":"createApp","highlightType":2},{"codeContent":"({});\napp.","highlightType":0},{"codeContent":"mount","highlightType":2},{"codeContent":"(","highlightType":0},{"codeContent":"\"#app\"","highlightType":3},{"codeContent":"); ","highlightType":0},{"codeContent":"// hi","highlightType":5}]}},{"messageType":2,"messageText":"Penutup."}]
```

## Lampiran B — rujukan sumber

| Apa | Lokasi JS |
|---|---|
| `extractIE` | `src/lib/builder.js:40-212` |
| `addText` | `builder.js:1037-1064` |
| `addCode` | `builder.js:1066-1090` |
| `addTable` | `builder.js:1092-1115` |
| `tokenizer` | `builder.js:1540-2203` |
| `toTableMetadata` | `builder.js:2205-2251` |
| `newLayout` | `builder.js:2253-2262` |
| loop `.ai` | `plugins/ai/ai.js:77-137` |
| `ytplay.addSection` | `plugins/search/ytplay.js:884-901` |
| `sendMusicPlayer` | `plugins/search/play2.js:893-943` |
| `stringifyEscaped` | ruby-baileys `MessageBuilder/index.js:561` |
