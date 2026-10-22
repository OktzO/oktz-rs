# BRIEF: Paritas 1:1 tampilan JS → Rust

## Tujuan

Bot Rust (`/workspaces/oktz-rs`) harus menampilkan **balasan yang identik
byte-per-byte** dengan bot JS (`/workspaces/oktz-rs/oktz-md`) untuk command
yang sama. Tidak ada boleh ada beda satu karakter pun: teks, emoji, pemisah,
urutan baris, format markdown, dan nama tombol.

Referensi JS: `/workspaces/oktz-rs/oktz-md/plugins/<kategori>/<file>.js`
Referensi Rust: `/workspaces/oktz-rs/plugin/src/<kategori>/<file>.rs`

## Dua jenis pekerjaan

### 1. Plugin BEDA (sudah ada di Rust, outputnya beda)

Baca file JS DAN file Rust-nya. Ubah **hanya sisi Rust** supaya output yang
dihasilkan identik dengan JS:

- Kata dan kalimat: sama persis (termasuk kapitalisasi).
- Emoji: karakter yang sama, bukan yang "mirip".
- Pemisah / garis / kotak (`╭─│┃┆╰⬣╮╯`): karakter yang sama.
- Urutan baris: sama.
- Format markdown (`*bold*`, `_italic_`, `` `code` ``, ``` ``` ```): sama.
- Pemotongan baris: JS memotong di lebar tertentu (`slice(0, N)` / `substring`).
  Samakan batas potongnya kalau JS memotong.
- Jumlah desimal (`toFixed(1)`) dan format angka (`toLocaleString` → pisah
  titik) harus sama.

Kalau Rust memakai helper/const yang berbeda, **ubah nilainya** atau
perbaiki helper-nya supaya menghasilkan string yang sama.

### 2. Plugin HILANG (belum ada di Rust)

Port plugin JS ke Rust dari nol. Ikuti konvensi repo:

```rust
//! `plugins/<kategori>/<file>.js` — <deskripsi>.
//!
//! <catatan penyimpangan yang memang tidak bisa都一样, kalau ada>

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "<nama persis seperti JS>",
    alias = ["<alias JS>"],
    category = "<kategori>",
    description = "<description JS persis>",
    usage = "<usage JS>",
    example = "<example JS>",
    cooldown = <angka JS>,
    energi = <angka JS>,
    is_group = <bool JS>,
    is_private = <bool JS>,
    is_owner = <bool JS>,
    is_premium = <bool JS>,
    is_admin = <bool JS>,
    is_bot_admin = <bool JS>
)]
pub fn <fn>(ctx: Ctx) -> Reply {
    Reply::text(<isi sama seperti JS>)
}
```

- `alias` JS berisi SEMUA nama lain (dari `name` array + `alias` array).
  Kalau `name` di JS berupa array, elemen pertama jadi `name`, sisanya masuk
  `alias`.
- `description`, `usage`, `example`, `cooldown`, `energi`, flag
  `is_group`/`is_private`/`is_owner`/`is_premium`/`is_admin`/`is_bot_admin`
  — ambil persis dari JS. Ini yang tampil di `.menu` dan di tabel fitur.
- Isi balasan: samakan dengan JS. Kalau JS pakai `m.reply(...)` → `Reply::text`.
  Kalau JS pakai `m.react(...)` → `.with_reaction("...")`.
- Register modul baru di `plugin/src/<kategori>/mod.rs` (tambah `pub mod x;`).
- Kalau plugin butuh scraping/API yang belum ada di `oktz-scraper`, pakai
  `ctx.http` (`ctx.http.get(url)` / `ctx.http.call(&HttpCall::get(url)...)`)
  dan tulis scraping-nya di file plugin itu sendiri.

## API yang tersedia (sudah lengkap, jangan diubah)

### Kartu interaktif (WA native-flow)

```rust
use oktz_plugin_api::registry::{
    CardHeader, CardList, CardLocation, CardQuote, CardRow, CardSection,
    InteractiveButton, MediaSource,
};

Reply::interactive(body)                  // kartu native-flow
  .with_footer("...")
  .with_button(InteractiveButton)
  .with_buttons(vec![...])
  .with_header(CardHeader)
  .with_message_params(serde_json::Value)  // nativeFlowMessage.messageParamsJson
  .with_context(CardContext)              // interactiveMessage.contextInfo

CardHeader::new().title(s).subtitle(s)
CardHeader::new().image(MediaSource::path(p))
CardHeader::new().video(MediaSource::path(p))
CardHeader::new().location(CardLocation {
    latitude, longitude, name, address: Option<String>, thumbnail: Option<Vec<u8>>,
})

CardContext::new().mention(jid).forwarded(9).newsletter(jid, name)
CardContext::new().quote(CardQuote::Contact { display_name, vcard })
CardContext::new().quote(CardQuote::Location { latitude, longitude, name, thumbnail })
CardContext::new().quote(CardQuote::Video { caption, seconds, mimetype, thumbnail, file_length: u64 })
CardContext::new().quote(CardQuote::Sticker { source: MediaSource })
CardContext::new().quote(CardQuote::Text(String))
CardContext::new().quote_trigger()        // kutip pesan pemicu (`quoted: m`)

CardList::new(title, sections).icon("DEFAULT")   // icon: "DEFAULT" | "REVIEW"
CardSection::new(title).highlight_label(s).row(CardRow::new(title, id).description(d).header(h))

InteractiveButton::quick_reply(display_text, id)   // id = PERINTAH, mis. ".menucat tools"
InteractiveButton::url(display_text, url)
InteractiveButton::url_merchant(display_text, url, merchant_url)
InteractiveButton::select(&CardList)
InteractiveButton::copy(display_text, id, copy_code)
InteractiveButton::empty(name)                      // name "" + params ""
```

### Tabel WhatsApp

```rust
Reply::table(title, headers, rows)   // rows: Vec<Vec<String>>
  .with_header_text("...")
  .with_table_footer("...")
```

### Link preview (`sock.sendPreview`)

```rust
Reply::preview(caption, url)        // -> extendedTextMessage + matchedText
  .with_preview_title("...")
  .with_preview_description("...")
  .with_preview_thumbnail(MediaSource::url(u))   // gambar, di-upload otomatis
  .with_preview_type(0)              // 0 = NONE, 1 = VIDEO
  .with_preview_context(CardContext::new().mention(jid))
```

### Balasan lain

```rust
Reply::text(s)                       // mengutip pemicu + mention
Reply::text_no_mention(s)            // mengutip, tanpa mention
Reply::text_quoted(s)                // TIDAK mengutip
Reply::image(MediaSource, caption)
Reply::video(MediaSource, caption)
Reply::audio(MediaSource)
Reply::document(MediaSource, file_name)
Reply::sticker(MediaSource)
Reply::sequence(vec![...])           // beberapa pesan berurutan
Reply::interactive(..).with_reaction("✅")
```

`MediaSource::path(p)` · `::url(u)` · `::bytes(b)` · `.with_mimetype(m)`.

## Sumber daya lain

- `ctx.text` — teks setelah nama command.
- `ctx.args` — token split whitespace.
- `ctx.prefix` — prefix command.
- `ctx.push_name`, `ctx.sender`, `ctx.chat`.
- `ctx.is_owner`, `ctx.is_premium`, `ctx.is_group`.
- `ctx.uptime_secs` — uptime bot.
- `ctx.config()` → `&Runtime` (`.bot.name`, `.bot.version`, `.bot.developer`,
  `.bot.website`, `.info.website`, `.owner.number`, `.saluran.id`,
  `.saluran.name`, `.assets.get("foto")`).
- `ctx.user_field("level")` → `Option<&serde_json::Value>`.
- `ctx.setting_i64("nama")` → `Option<i64>`.
- `ctx.http.get(url)` → `Result<HttpResponse, HttpError>`; `.json()` → `Option<Value>`.
- `ctx.berkas().baca(path)` → `Result<Vec<u8>>`.
- `oktz_plugin_api::gambar::sharp(&bytes)?.ubah_ukuran(w, h).ke_jpeg(80)` → resize.
- `oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&args, &bytes)` → ffmpeg.

## Aturan keras

1. **Hanya edit `plugin/src/<kategori>/...`** (termasuk `mod.rs` kategori itu).
   DILARANG menyentuh `lib/**`, `src/**`, `plugin/src/lib.rs`, dan kategori
   lain.
2. `cargo build --release` harus lulus. Kalau gagal karena file kategori LAIN
   (dibeingi agent lain), abaikan error itu — pastikan file milikmu sendiri
   tidak punya error.
3. Setiap plugin yang berubah harus punya test unit yang mengunci teks
   outputnya (assert_contains / assert_eq). Namanya风格: `#[test] fn
   output_sama_dengan_js()`.
4. Komentar dalam bahasa Indonesia, sertakan rujukan baris JS, contoh:
   `// ping.js:42-58`.
5. Jangan mengubah `#[plugin(...)]` milik plugin yang sudah ada kecuali
   memang perlu koreksi kecil (mis. alias JS yang belum ada).
6. Kalau suatu bagian JS **tidak bisa** ditiru (mis. Butuh API yang tidak
   ada), buat output paling dekat yang mungkin dan JELASKAN di doc-comment
   plugin + di laporan akhir.
7. Jangan sentuh file `scripts/paritas-audit.py` dan `docs/paritas/*`.

## Verifikasi sebelum lapor

```
export PATH="$HOME/.cargo/bin:$PATH"
cd /workspaces/oktz-rs
cargo build --release
cargo test --release -p oktz-plugins --lib <kategori>:
```

## Laporan akhir

Sebutkan:
- Plugin yang diperbaiki (nama → file Rust).
- Plugin baru yang dibuat (nama → file Rust + baris `pub mod` yang perlu
  ditambah di `mod.rs` kategori).
- Penyimpangan yang tidak bisa ditiru + alasannya.
- Status build & test.
