# Ringkasan paritas JS vs Rust

Total plugin JS diperiksa: 792

- HILANG (tidak ada di Rust): 44
- BEDA (baris output beda): 506
- ADA (tidak ada baris hilang): 242

| kategori | hilang | beda | ada |
|---|---|---|---|
| ai | 1 | 37 | 8 |
| anime | 1 | 2 | 0 |
| asupan | 0 | 3 | 1 |
| berita | 0 | 1 | 0 |
| canvas | 12 | 13 | 5 |
| cecan | 0 | 0 | 6 |
| cek | 0 | 0 | 48 |
| clan | 0 | 4 | 5 |
| download | 0 | 18 | 6 |
| downloader | 0 | 1 | 1 |
| ephoto | 0 | 1 | 0 |
| fun | 15 | 13 | 12 |
| game | 2 | 10 | 1 |
| group | 0 | 59 | 44 |
| info | 0 | 10 | 3 |
| islamic | 0 | 2 | 0 |
| jpm | 1 | 0 | 0 |
| linode | 1 | 0 | 0 |
| main | 3 | 13 | 3 |
| maker | 0 | 2 | 0 |
| media | 0 | 1 | 0 |
| nsfw | 0 | 1 | 0 |
| owner | 0 | 129 | 22 |
| panel | 2 | 7 | 1 |
| primbon | 0 | 1 | 7 |
| pushkontak | 0 | 1 | 0 |
| random | 1 | 0 | 5 |
| religi | 0 | 3 | 1 |
| rpg | 0 | 40 | 27 |
| search | 0 | 38 | 8 |
| stalker | 1 | 10 | 3 |
| stalker   | 0 | 1 | 0 |
| sticker | 0 | 16 | 4 |
| store | 2 | 10 | 2 |
| tools | 0 | 49 | 7 |
| tts | 0 | 1 | 1 |
| user | 2 | 8 | 6 |
| utility | 0 | 1 | 0 |
| vps | 0 | 0 | 5 |

## Plugin dengan baris output berbeda

### ai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/ai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/ai.rs`

  - JS: `[AI Error]`
    Rust: (tidak ada padanan)

### anime-gen (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/anime-gen.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/anime-gen.rs`

  - JS: `> Request terlalu lama. Coba lagi!`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *ᴛɪᴍᴇᴏᴜᴛ*`
    Rust: (tidak ada padanan)

### bananagen2 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/bananagen2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/bananagen2.rs`

  - JS: `Terjadi kesalahan`
    Rust: (tidak ada padanan)

### claudehaiku (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/claudehaiku.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/claudehaiku.rs`

  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *Claude Haiku Gagal*`
    Rust: `Claude Haiku Gagal`

### deepseek (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/deepseek.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/deepseek.rs`

  - JS: `...`
    Rust: `␀...`

### dolphin (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/dolphin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/dolphin.rs`

  - JS: `Mozilla/5.0 (Linux; Android 15; SM-F958 Build/AP3A.240905.015) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.6723.86 Mobile Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)

### dpsteai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/dpsteai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/dpsteai.rs`

  - JS: `Content-Length`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `[DPSTE AI] Gagal menyimpan sesi:`
    Rust: (tidak ada padanan)
  - JS: `dpsteaai.json`
    Rust: (tidak ada padanan)

### feelbetter (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/feelbetter.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/feelbetter.rs`

  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *FeelBetter Gagal*`
    Rust: `FeelBetter Gagal`

### gpt5 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/gpt5.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/gpt5.rs`

  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *GPT-5 Gagal*`
    Rust: `GPT-5 Gagal`

### jokowi-ai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/jokowi-ai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/jokowi-ai.rs`

  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *Jokowi AI Error*`
    Rust: `Jokowi AI Error`

### kobo-ai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/kobo-ai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/kobo-ai.rs`

  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *Kobo AI Error*`
    Rust: `Kobo AI Error`

### matematika (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/matematika.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/matematika.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)

### musicmaker (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/musicmaker.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/musicmaker.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[Music Maker AI]`
    Rust: (tidak ada padanan)

### muslimai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/muslimai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/muslimai.rs`

  - JS: `Content-Type`
    Rust: (tidak ada padanan)

### prabowo-ai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/prabowo-ai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/prabowo-ai.rs`

  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *Prabowo AI Error*`
    Rust: `Prabowo AI Error`

### quilbot (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/quillbot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/quillbot.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[Quillbot]`
    Rust: (tidak ada padanan)

### qwen3 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/qwen3.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/qwen3.rs`

  - JS: `> ␀`
    Rust: (tidak ada padanan)

### simi (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/simi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/simi.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[SimiSimi]`
    Rust: (tidak ada padanan)

### sologo (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/sologo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/sologo.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[SoLogo AI]`
    Rust: (tidak ada padanan)

### text2img (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/text2img.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/text2img.rs`

  - JS: `Output kosong`
    Rust: (tidak ada padanan)

### text2img2 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/text2img2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/text2img2.rs`

  - JS: `Contoh: ␀ car`
    Rust: `Contoh: ␀␀ car`
  - JS: `Invalid image data received`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)

### to3d (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/to3d.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/to3d.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### toanime (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/toanime.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/toanime.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### toblack (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/toblack.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/toblack.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tocartoon (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tocartoon.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tocartoon.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tocermin (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tocermin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tocermin.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tochibi (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tochibi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tochibi.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tofigure (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tofigure.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tofigure.rs`

  - JS: `image.jpg`
    Rust: (tidak ada padanan)

### tofigure3 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tofigurine.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tofigurine.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tohijab (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tohijab.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tohijab.rs`

  - JS: `image.jpg`
    Rust: (tidak ada padanan)

### toisland (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/toisland.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/toisland.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tojapanese (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tojapanese.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tojapanese.rs`

  - JS: `image.jpg`
    Rust: (tidak ada padanan)

### tomanga (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tomanga.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tomanga.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### tooilpainting (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/tooilpainting.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/tooilpainting.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)

### text2img3 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/txt2img.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/txt2img.rs`

  - JS: `Hasil kosong dari provider`
    Rust: (tidak ada padanan)
  - JS: `␀txt2img`
    Rust: (tidak ada padanan)

### text2img4 (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/txt2img2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/txt2img2.rs`

  - JS: `❌ Gagal generate gambar, coba lagi nanti`
    Rust: (tidak ada padanan)

### waguri-ai (ai)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ai/waguri-ai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ai/waguri-ai.rs`

  - JS: `> ␀`
    Rust: (tidak ada padanan)

### mywaifu (anime)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/anime/mywaifu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/anime/mywaifu.rs`

  - JS: `Gagal mengambil data dari API (Status: ␀)`
    Rust: (tidak ada padanan)
  - JS: `Maaf, terjadi gangguan saat sistem mencoba memanggil API. Pesan error: _␀_`
    Rust: `Maaf, terjadi gangguan saat sistem mencoba memanggil API.`
  - JS: `MyWaifu Error:`
    Rust: (tidak ada padanan)

### topanime (anime)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/anime/topanime.js`
- Rust: `/workspaces/oktz-rs/plugin/src/anime/topanime.rs`

  - JS: `[TOPANIME Plugin Error]`
    Rust: (tidak ada padanan)

### asupan (asupan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/asupan/asupan.js`
- Rust: `/workspaces/oktz-rs/plugin/src/asupan/asupan.rs`

  - JS: `utf-8`
    Rust: (tidak ada padanan)

### bocil (asupan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/asupan/bocil.js`
- Rust: `/workspaces/oktz-rs/plugin/src/asupan/bocil.rs`

  - JS: `bocil.json`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)

### ukhty (asupan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/asupan/ukhty.js`
- Rust: `/workspaces/oktz-rs/plugin/src/asupan/ukhty.rs`

  - JS: `ukhty.json`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)

### applemusic-canvas (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/applemusic.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/applemusic.rs`

  - JS: `[APPLEMUSIC Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `form-data`
    Rust: (tidak ada padanan)

### buatquotes (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/buatquotes.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/buatquotes.rs`

  - JS: `"␀"`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `ARIALN.ttf`
    Rust: (tidak ada padanan)
  - JS: `Background nomor ␀ tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `Gagal download ␀: ␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `Inter, sans-serif`
    Rust: (tidak ada padanan)
  - JS: `Inter-Medium.woff2`
    Rust: (tidak ada padanan)
  - JS: `Maaf, sistem mengalami gangguan saat mencoba membuat gambar quote. Silakan coba lagi nanti.`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `arialn, sans-serif`
    Rust: (tidak ada padanan)
  - JS: `node:fs`
    Rust: (tidak ada padanan)
  - JS: `node:fs/promises`
    Rust: (tidak ada padanan)
  - JS: `node:path`
    Rust: (tidak ada padanan)
  - JS: `quoteanime_␀.png`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀px ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *GAGAL MEMBUAT QUOTE*`
    Rust: (tidak ada padanan)

### facepalm (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/facepalm.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/facepalm.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `Bikin gambar facepalm dari fotomu`
    Rust: (tidak ada padanan)
  - JS: `Gagal membaca media`
    Rust: (tidak ada padanan)
  - JS: `⚠️ Harap kirim atau reply foto dengan perintah `␀␀``
    Rust: (tidak ada padanan)

### fakecall (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/fakecall.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/fakecall.rs`

  - JS: `form-data`
    Rust: (tidak ada padanan)

### fakegopay (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/fakegopay.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/fakegopay.rs`

  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)

### fakeovo (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/fakeovo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/fakeovo.rs`

  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)

### iqc (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/iqc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/iqc.rs`

  - JS: `Maaf, terjadi kesalahan saat mencoba membuat gambar quote. Silakan coba lagi beberapa saat.`
    Rust: (tidak ada padanan)
  - JS: `[IQC Plugin Error]`
    Rust: (tidak ada padanan)

### iqc2 (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/iqc2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/iqc2.rs`

  - JS: `Maaf *␀*, terjadi kesalahan saat mencoba membuat gambar quote. Silakan coba lagi beberapa saat.`
    Rust: (tidak ada padanan)
  - JS: `[IQC2 Plugin Error]`
    Rust: (tidak ada padanan)

### iqcpink (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/iqcpink.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/iqcpink.rs`

  - JS: `*CARA PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `- Atau balas (*reply*) pesan orang lain dengan perintah `␀iqcpink``
    Rust: (tidak ada padanan)
  - JS: `- Ketik `␀iqcpink <teks kamu>``
    Rust: `Contoh JS: `.iqcpink <teks kamu>``
  - JS: `22.54`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `400 ␀px Inter`
    Rust: (tidak ada padanan)
  - JS: `500 ␀px Inter`
    Rust: (tidak ada padanan)
  - JS: `600 ␀px Inter`
    Rust: (tidak ada padanan)
  - JS: `Inter-Medium.ttf`
    Rust: (tidak ada padanan)
  - JS: `Inter-Regular.ttf`
    Rust: (tidak ada padanan)
  - JS: `Inter-SemiBold.ttf`
    Rust: (tidak ada padanan)
  - JS: `Kesendirian adalah teman terbaik ku😂😂`
    Rust: (tidak ada padanan)
  - JS: `Maaf, terjadi kesalahan saat mencoba membuat gambar quote pink. Silakan coba lagi beberapa saat.`
    Rust: (tidak ada padanan)
  - JS: `Membuat Fake Quote iOS style versi pink.`
    Rust: `Membuat Fake Quote iOS style versi pink — BLOKIRAN`
  - JS: `Sistem ini akan mengubah teks yang kamu berikan menjadi gambar *quote* bergaya iOS dengan tema Pink yang cantik dan elegan.`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[iqcpink error]:`
    Rust: (tidak ada padanan)
  - JS: `_Pesan yang dibalas akan otomatis diubah menjadi quote menggunakan nama dan foto profil pembuat pesan tersebut!_`
    Rust: (tidak ada padanan)
  - JS: `iqcpink_out_␀.png`
    Rust: (tidak ada padanan)
  - JS: `iqcpink_␀.png`
    Rust: (tidak ada padanan)
  - JS: `node:fs`
    Rust: (tidak ada padanan)
  - JS: `node:fs/promises`
    Rust: (tidak ada padanan)
  - JS: `node:path`
    Rust: (tidak ada padanan)
  - JS: `rgba(0,0,0,0.05)`
    Rust: (tidak ada padanan)
  - JS: `rgba(0,0,0,0.10)`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `␀.␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *GAGAL MEMPROSES QUOTE*`
    Rust: (tidak ada padanan)
  - JS: `❤️`
    Rust: (tidak ada padanan)
  - JS: `🩷 *FITUR FAKE QUOTE iOS PINK*`
    Rust: (tidak ada padanan)

### musiccard (maker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/musiccard.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/musiccard.rs`

  - JS: `Balas gambar temanmu, lalu ketik: `.musiccard Perfect|Ed Sheeran``
    Rust: (tidak ada padanan)
  - JS: `Contoh:`
    Rust: (tidak ada padanan)
  - JS: `Kamu harus me-reply (membalas) pesan berupa *gambar* dengan format `.musiccard <judul>|<nama>`.`
    Rust: (tidak ada padanan)
  - JS: `Kamu harus mengirim sebuah gambar dengan caption (teks pelengkap) `.musiccard <judul>|<nama>` atau reply gambar yang sudah ada.`
    Rust: (tidak ada padanan)
  - JS: `Keren banget kan hasilnya? Pamerin ke teman-temanmu yuk! 🚀`
    Rust: (tidak ada padanan)
  - JS: `Kirim gambar dengan caption: `.musiccard Perfect|Ed Sheeran``
    Rust: (tidak ada padanan)
  - JS: `Sistem gagal menghubungi server pembuat kartu. Silakan coba beberapa saat lagi ya.`
    Rust: (tidak ada padanan)
  - JS: `[Music Card]`
    Rust: (tidak ada padanan)
  - JS: `content-type`
    Rust: (tidak ada padanan)
  - JS: `img.jpg`
    Rust: (tidak ada padanan)
  - JS: `pixhost.to`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *Gagal membuat Music Card.* Server merespon dengan format yang salah.`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *Gagal mengunggah gambar!* Pastikan ukuran gambarnya tidak terlalu besar dan coba lagi ya.`
    Rust: (tidak ada padanan)
  - JS: `✨ *MUSIC CARD BERHASIL DIBUAT!* ✨`
    Rust: (tidak ada padanan)
  - JS: `❌ *Waduh, gambarnya mana nih?*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Waduh, itu bukan gambar!*`
    Rust: (tidak ada padanan)
  - JS: `🎤 *Artis*: ␀`
    Rust: (tidak ada padanan)
  - JS: `🎧 *Judul*: ␀`
    Rust: (tidak ada padanan)
  - JS: `😔 *Gagal mendownload gambar.* Coba kirim ulang gambarnya ya.`
    Rust: (tidak ada padanan)
  - JS: `😔 *Terjadi masalah di sistem kami.*`
    Rust: (tidak ada padanan)

### profileig (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/profileig.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/profileig.rs`

  - JS: `"Content-Type": "application/json"`
    Rust: (tidak ada padanan)
  - JS: `"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36",`
    Rust: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36`
  - JS: `await m.react("✅");`
    Rust: (tidak ada padanan)
  - JS: `await m.react("❌");`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMessage(m.chat, { image: Buffer.from(res.data), caption: "✅ Berhasil membuat profile IG" }, { quoted: m });`
    Rust: (tidak ada padanan)
  - JS: `console.error("[ProfileIG Error]", e);`
    Rust: (tidak ada padanan)
  - JS: `const res = await axios.get(url, {`
    Rust: (tidak ada padanan)
  - JS: `const url = `https://kyzznekoo.zone.id/api/canvas/profileig?url=␀&pengikut=␀&mengikuti=␀&postingan=␀&username=␀&bio=␀&verif=␀`;`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `imgUrl = "https://i.ibb.co/3Fh9Q6M/blank-profile-picture.png";`
    Rust: (tidak ada padanan)
  - JS: `m.reply(te(m.prefix, m.command, m.pushName));`
    Rust: (tidak ada padanan)
  - JS: `responseType:`
    Rust: (tidak ada padanan)
  - JS: `t have a profile picture, provide a default URL`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: `);`

### sroast (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/sroast.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/sroast.rs`

  - JS: `28px "␀"`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `bold ␀px "␀"`
    Rust: (tidak ada padanan)
  - JS: `sans-serif`
    Rust: (tidak ada padanan)

### starboy (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/starboy.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/starboy.rs`

  - JS: `> Kirim gambar dengan caption *␀* atau balas gambar yang sudah dikirim.`
    Rust: (tidak ada padanan)
  - JS: `Buat gambar efek Starboy dari foto`
    Rust: (tidak ada padanan)
  - JS: `Gagal mengunduh media`
    Rust: (tidak ada padanan)
  - JS: `Gagal mengunggah gambar`
    Rust: (tidak ada padanan)
  - JS: `🌟 *S T A R B O Y*`
    Rust: (tidak ada padanan)

### tiktokchat (canvas)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/canvas/tiktokchat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/canvas/tiktokchat.rs`

  - JS: `[TIKTOKCHAT Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `form-data`
    Rust: (tidak ada padanan)

### claninfo (clan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/clan/claninfo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/clan/claninfo.rs`

  - JS: `␀%`
    Rust: (tidak ada padanan)
  - JS: `├ 👥 Members · ␀/50`
    Rust: `├ 👥 Members · ␀/␀`

### claninvite (clan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/clan/claninvite.js`
- Rust: `/workspaces/oktz-rs/plugin/src/clan/claninvite.rs`

  - JS: `Members: ␀/50`
    Rust: `Members: 2/50`
  - JS: `❌ Clan sudah penuh (50/50)`
    Rust: `❌ Clan sudah penuh (␀/␀)`

### clankick (clan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/clan/clankick.js`
- Rust: `/workspaces/oktz-rs/plugin/src/clan/clankick.rs`

  - JS: `Sisa members: ␀/50`
    Rust: `Sisa members: 1/50`

### clanmembers (clan)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/clan/clanmembers.js`
- Rust: `/workspaces/oktz-rs/plugin/src/clan/clanmembers.rs`

  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `␀/50 members`
    Rust: `2/50 members`

### alightmotiondl (downloader)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/alightmotion.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/alightmotion.rs`

  - JS: `[AMDL Plugin Error]`
    Rust: (tidak ada padanan)

### capcutdl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/capcutdl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/capcutdl.rs`

  - JS: `btch-downloader`
    Rust: (tidak ada padanan)

### cocofundl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/cocofundl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/cocofundl.rs`

  - JS: `btch-downloader`
    Rust: (tidak ada padanan)

### dailymotiondl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/dailymotiondl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/dailymotiondl.rs`

  - JS: `) +`
    Rust: (tidak ada padanan)
  - JS: `-bsf:a`
    Rust: (tidak ada padanan)
  - JS: `-c`
    Rust: (tidak ada padanan)
  - JS: `-i`
    Rust: (tidak ada padanan)
  - JS: `-y`
    Rust: (tidak ada padanan)
  - JS: `.dailymotiondl <url>`
    Rust: `> *␀dailymotiondl <link>*`
  - JS: `.dailymotiondl https://www.dailymotion.com/video/xxx`
    Rust: `> *␀dailymotiondl https://www.dailymotion.com/video/xxx*`
  - JS: `Download video dari Dailymotion`
    Rust: (tidak ada padanan)
  - JS: `console.error(e);`
    Rust: (tidak ada padanan)
  - JS: `dm_␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `m.react(`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`
    Rust: (tidak ada padanan)
  - JS: `{ quoted: m },`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `},`
    Rust: (tidak ada padanan)

### douyindl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/douyindl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/douyindl.rs`

  - JS: `douyin.com.evil.example`
    Rust: (tidak ada padanan)
  - JS: `notdouyin.com`
    Rust: `douyin.com`

### instagramdl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/instagramdl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/instagramdl.rs`

  - JS: `❌ *ɢᴀɢᴀʟ ᴍᴇɴɢᴜɴᴅᴜʜ*`
    Rust: `❌ *ɢᴀɢᴀʟ ᴍᴇɴɢᴜɴᴅᴜʰ*`
  - JS: `📸 *Instagram ␀*`
    Rust: `📸 *Instagram Story*`

### pindl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/pindl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/pindl.rs`

  - JS: `'Referer': 'https://www.pinterest.com/'`
    Rust: (tidak ada padanan)
  - JS: `'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',`
    Rust: (tidak ada padanan)
  - JS: `(lastError ? `> Penyebab: `␀``
    Rust: (tidak ada padanan)
  - JS: `) +`
    Rust: (tidak ada padanan)
  - JS: `) ? mUrl[1] : baseUrl + mUrl[1];`
    Rust: (tidak ada padanan)
  - JS: `) {`
    Rust: (tidak ada padanan)
  - JS: `)) {`
    Rust: (tidak ada padanan)
  - JS: `, err.message);`
    Rust: (tidak ada padanan)
  - JS: `, error);`
    Rust: (tidak ada padanan)
  - JS: `, fallbackErr.message);`
    Rust: (tidak ada padanan)
  - JS: `, gifErr.message);`
    Rust: (tidak ada padanan)
  - JS: `-c copy`
    Rust: (tidak ada padanan)
  - JS: `-c:v libx264 -pix_fmt yuv420p -movflags faststart -preset ultrafast -an`
    Rust: (tidak ada padanan)
  - JS: `-headers`
    Rust: (tidak ada padanan)
  - JS: `-i`
    Rust: (tidak ada padanan)
  - JS: `-t 30 -vf`
    Rust: (tidak ada padanan)
  - JS: `.pindl <url>`
    Rust: (tidak ada padanan)
  - JS: `.pindl https://pin.it/xxx`
    Rust: `> `␀pindl https://pin.it/xxx``
  - JS: `// 1GB yang plafon RSS-nya 550MB. `httpAxios` membawa `maxContentLength``
    Rust: (tidak ada padanan)
  - JS: `// 25MB, dan `responseType:`
    Rust: (tidak ada padanan)
  - JS: `// 550MB, dan gagal dengan pesan lebih baik daripada bot ikut mati.`
    Rust: (tidak ada padanan)
  - JS: `// Batas ukuran wajib: instance axios ini maxContentLength-nya -1,`
    Rust: (tidak ada padanan)
  - JS: `// Bukan jaringan yang rusak, jadi tidak ada yang perlu dicoba lagi:`
    Rust: (tidak ada padanan)
  - JS: `// Ditandai supaya catch di bawah tahu ini bukan masalah jaringan:`
    Rust: (tidak ada padanan)
  - JS: `// Fallback: Pinterest memblokir generic axios (403), jadi download manual pakai User-Agent.`
    Rust: (tidak ada padanan)
  - JS: `// File hasil ffmpeg dibaca utuh untuk dikirim, jadi ukurannya diperiksa`
    Rust: (tidak ada padanan)
  - JS: `// Tanpa manifest (atau tanpa varian video di dalamnya), ffmpeg`
    Rust: (tidak ada padanan)
  - JS: `// `f()` sengaja tidak dipakai di sini: ia memakai undici dan tidak punya`
    Rust: (tidak ada padanan)
  - JS: `// axios yang tidak menjelaskan apa pun.`
    Rust: (tidak ada padanan)
  - JS: `// batas ukuran, jadi satu GIF besar bisa masuk penuh ke memori di kotak`
    Rust: (tidak ada padanan)
  - JS: `// dan satu video besar bisa membuat proses melewati batas 550MB.`
    Rust: (tidak ada padanan)
  - JS: `// digabung kalau keduanya benar-benar ada di manifest.`
    Rust: (tidak ada padanan)
  - JS: `// hanya membuat user menunggu pull 64MB yang pasti gagal.`
    Rust: (tidak ada padanan)
  - JS: `// langsung ditulis ke disk tanpa disalin jadi string lebih dulu.`
    Rust: (tidak ada padanan)
  - JS: `// lebih dulu: video raksasa akan mendorong proses melewati batas RSS`
    Rust: (tidak ada padanan)
  - JS: `// memakai URL itu apa adanya; video dan audio terpisah hanya perlu`
    Rust: (tidak ada padanan)
  - JS: `// sebenarnya diteruskan apa adanya supaya user tidak membaca pesan`
    Rust: (tidak ada padanan)
  - JS: `// unduhan manual akan berhenti di batas yang sama, jadi mengulangnya`
    Rust: (tidak ada padanan)
  - JS: `// unduhan manual memakai batas yang sama dan pasti gagal. Alasan`
    Rust: (tidak ada padanan)
  - JS: `: Buffer.from(res?.data ??`
    Rust: (tidak ada padanan)
  - JS: `? res.data`
    Rust: (tidak ada padanan)
  - JS: `Download gambar/video dari Pinterest`
    Rust: `> Download gambar/video dari Pinterest`
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36`
    Rust: (tidak ada padanan)
  - JS: `Tidak ada media ditemukan`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `` +`
    Rust: (tidak ada padanan)
  - JS: `` :`
    Rust: (tidak ada padanan)
  - JS: `` mengembalikan Buffer yang bisa`
    Rust: (tidak ada padanan)
  - JS: ``);`
    Rust: `);`
  - JS: ``,`
    Rust: (tidak ada padanan)
  - JS: ``;`
    Rust: (tidak ada padanan)
  - JS: ``> Pinterest biasanya memblokir unduhan dari luar — coba lagi nanti ya.`,`
    Rust: `> Pinterest biasanya memblokir unduhan dari luar — coba lagi nanti ya.`
  - JS: ``ffmpeg -y -ignore_loop 0 -i`
    Rust: (tidak ada padanan)
  - JS: ``video ␀ MB melebihi batas ␀ MB`,`
    Rust: (tidak ada padanan)
  - JS: ``❌ *ɢᴀɢᴀʟ*`
    Rust: `❌ *ɢᴀɢᴀʟ*`
  - JS: `await queueFFmpeg(`
    Rust: (tidak ada padanan)
  - JS: `await queueFFmpeg(`ffmpeg -y -i`
    Rust: (tidak ada padanan)
  - JS: `await queueFFmpeg(`ffmpeg -y ␀ -i`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMedia(m.chat, Buffer.from(fallbackBuffer.data), null, m, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMedia(m.chat, fs.readFileSync(mp4Path), null, m, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMedia(m.chat, fs.readFileSync(outputFile), null, m, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMedia(m.chat, media.url, null, m, {`
    Rust: (tidak ada padanan)
  - JS: `console.error(`
    Rust: (tidak ada padanan)
  - JS: `const fallbackBuffer = await axios.get(media.url, {`
    Rust: (tidak ada padanan)
  - JS: `const ffmpegHdr = `-user_agent`
    Rust: (tidak ada padanan)
  - JS: `const gagal = new Error(`
    Rust: (tidak ada padanan)
  - JS: `const gifPath = path.join(tempPath, `pin-␀.gif`);`
    Rust: (tidak ada padanan)
  - JS: `const id = Date.now();`
    Rust: (tidak ada padanan)
  - JS: `const mp4Path = path.join(tempPath, `pin-␀.mp4`);`
    Rust: (tidak ada padanan)
  - JS: `const raw = Buffer.isBuffer(res?.data)`
    Rust: (tidak ada padanan)
  - JS: `const res = await httpAxios.get(media.url, {`
    Rust: (tidak ada padanan)
  - JS: `const tempPath = path.join(process.cwd(),`
    Rust: (tidak ada padanan)
  - JS: `const ukuran = fs.statSync(outputFile).size;`
    Rust: (tidak ada padanan)
  - JS: `contextInfo: {`
    Rust: (tidak ada padanan)
  - JS: `contextInfo: { forwardingScore: 99, isForwarded: true }`
    Rust: (tidak ada padanan)
  - JS: `contextInfo: { forwardingScore: 99, isForwarded: true },`
    Rust: (tidak ada padanan)
  - JS: `data.media`
    Rust: (tidak ada padanan)
  - JS: `export { rencanaVideo, pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `forwardingScore: 99,`
    Rust: (tidak ada padanan)
  - JS: `fs.mkdirSync(tempPath, { recursive: true });`
    Rust: (tidak ada padanan)
  - JS: `fs.renameSync(videoTemp, outputFile);`
    Rust: (tidak ada padanan)
  - JS: `fs.writeFileSync(gifPath, raw);`
    Rust: (tidak ada padanan)
  - JS: `gagal.terlaluBesar = true;`
    Rust: (tidak ada padanan)
  - JS: `gifPlayback: true,`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `if (!fs.existsSync(mp4Path)) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!fs.existsSync(tempPath))`
    Rust: (tidak ada padanan)
  - JS: `if (!hls || !videoStreamUrl) {`
    Rust: (tidak ada padanan)
  - JS: `if (!raw.length) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (audioStreamUrl) {`
    Rust: (tidak ada padanan)
  - JS: `if (err?.terlaluBesar) {`
    Rust: (tidak ada padanan)
  - JS: `if (fs.existsSync(audioTemp)) fs.unlinkSync(audioTemp);`
    Rust: (tidak ada padanan)
  - JS: `if (fs.existsSync(gifPath)) fs.unlinkSync(gifPath);`
    Rust: (tidak ada padanan)
  - JS: `if (fs.existsSync(mp4Path)) fs.unlinkSync(mp4Path);`
    Rust: (tidak ada padanan)
  - JS: `if (fs.existsSync(outputFile)) fs.unlinkSync(outputFile);`
    Rust: (tidak ada padanan)
  - JS: `if (fs.existsSync(videoTemp)) fs.unlinkSync(videoTemp);`
    Rust: (tidak ada padanan)
  - JS: `if (media.url.includes(`
    Rust: (tidak ada padanan)
  - JS: `if (sentCount > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (terlaluBesar(ukuran)) {`
    Rust: (tidak ada padanan)
  - JS: `isForwarded: true,`
    Rust: (tidak ada padanan)
  - JS: `lastError = err.message;`
    Rust: (tidak ada padanan)
  - JS: `lastError = fallbackErr.message;`
    Rust: (tidak ada padanan)
  - JS: `m.react(`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`
    Rust: (tidak ada padanan)
  - JS: `m.reply(te(m.prefix, m.command, m.pushName));`
    Rust: (tidak ada padanan)
  - JS: `maxContentLength: BATAS_BODY_VIDEO`
    Rust: (tidak ada padanan)
  - JS: `new URL()`
    Rust: (tidak ada padanan)
  - JS: `notpinterest.com`
    Rust: (tidak ada padanan)
  - JS: `pin.it.evil.example`
    Rust: (tidak ada padanan)
  - JS: `responseType:`
    Rust: (tidak ada padanan)
  - JS: `responseType: 'arraybuffer',`
    Rust: (tidak ada padanan)
  - JS: `sentCount += 1;`
    Rust: (tidak ada padanan)
  - JS: `split("`
    Rust: (tidak ada padanan)
  - JS: `throw gagal;`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `type:`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (error) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (fallbackErr) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (gifErr) {`
    Rust: (tidak ada padanan)
  - JS: `} else if (media.type ===`
    Rust: (tidak ada padanan)
  - JS: `} else {`
    Rust: `) ␀ else {`
  - JS: `} finally {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: `};`
  - JS: `},`
    Rust: (tidak ada padanan)
  - JS: `␀_a.mp4`
    Rust: `.mp4`
  - JS: `␀_final.mp4`
    Rust: (tidak ada padanan)
  - JS: `␀_v.mp4`
    Rust: `.mp4`

### sfiledl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/sfiledl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/sfiledl.rs`

  - JS: `!data?.url`
    Rust: (tidak ada padanan)
  - JS: `aggregator.hit`
    Rust: (tidak ada padanan)
  - JS: `data.url`
    Rust: (tidak ada padanan)
  - JS: `data.url kosong`
    Rust: (tidak ada padanan)
  - JS: `notsfile.mobi`
    Rust: `sfile.mobi`
  - JS: `sfile.mobi.evil.example`
    Rust: (tidak ada padanan)
  - JS: `␀sfiledl`
    Rust: `.sfiledl`

### shopeedl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/shopeedl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/shopeedl.rs`

  - JS: `HTTP ␀`
    Rust: (tidak ada padanan)
  - JS: `[Shopee DL]`
    Rust: (tidak ada padanan)

### snackvideodl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/snackvideodl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/snackvideodl.rs`

  - JS: `btch-downloader`
    Rust: (tidak ada padanan)

### spotifydl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/spotifydl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/spotifydl.rs`

  - JS: `!data?.url`
    Rust: (tidak ada padanan)
  - JS: `Kamu harus memasukkan tautan (link) lagu dari Spotify yang valid. Pastikan itu adalah link ke track/lagu ya!`
    Rust: `Kamu harus memasukkan tautan (link) lagu dari Spotify yang valid.`
  - JS: `[Spotify DL Error]`
    Rust: (tidak ada padanan)
  - JS: `data.url`
    Rust: (tidak ada padanan)
  - JS: `data.url kosong`
    Rust: (tidak ada padanan)
  - JS: `😔 *Terjadi kesalahan sistem saat memproses tautan Spotify tersebut.* Mohon coba lagi nanti ya!`
    Rust: `😔 *Terjadi kesalahan sistem saat memproses tautan Spotify tersebut.*`

### terabox (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/terabox.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/terabox.rs`

  - JS: `(result.file_name ||`
    Rust: (tidak ada padanan)
  - JS: `) +`
    Rust: (tidak ada padanan)
  - JS: `).replace(/[<>:`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `-bsf:a`
    Rust: (tidak ada padanan)
  - JS: `-c`
    Rust: (tidak ada padanan)
  - JS: `-i`
    Rust: (tidak ada padanan)
  - JS: `-y`
    Rust: (tidak ada padanan)
  - JS: `;`
    Rust: (tidak ada padanan)
  - JS: `caption,`
    Rust: (tidak ada padanan)
  - JS: `const ext = result.extension ||`
    Rust: (tidak ada padanan)
  - JS: `const fileName =`
    Rust: (tidak ada padanan)
  - JS: `const res = await axios.get(result.download_url, {`
    Rust: (tidak ada padanan)
  - JS: `maxContentLength: 200 * 1024 * 1024,`
    Rust: (tidak ada padanan)
  - JS: `responseType:`
    Rust: (tidak ada padanan)
  - JS: `tb_␀.mp4`
    Rust: `␀.mp4`
  - JS: `timeout: 120000,`
    Rust: (tidak ada padanan)
  - JS: `{ quoted: m },`
    Rust: (tidak ada padanan)
  - JS: `} else if (result.download_url) {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `},`
    Rust: (tidak ada padanan)

### threaddl (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/threaddl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/threaddl.rs`

  - JS: `"Android"`
    Rust: (tidak ada padanan)
  - JS: `"Google Chrome";v="147", "Not.A/Brand";v="8", "Chromium";v="147"`
    Rust: (tidak ada padanan)
  - JS: `*Semoga bermanfaat ya!* Jangan lupa mampir lagi kalau mau download yang lain. 🚀`
    Rust: (tidak ada padanan)
  - JS: `.tdl <url>`
    Rust: (tidak ada padanan)
  - JS: `.tdl https://www.threads.net/@xxx/post/xxx`
    Rust: `Contoh: `.tdl https://www.threads.net/@zuck/post/xxx``
  - JS: `?1`
    Rust: (tidak ada padanan)
  - JS: `Download foto dan video dari postingan Threads tanpa repot!`
    Rust: (tidak ada padanan)
  - JS: `Halo! Ini hasil unduhan Threads yang kamu minta:`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `[ThreadsDL]`
    Rust: (tidak ada padanan)
  - JS: `accept-language`
    Rust: (tidak ada padanan)
  - JS: `content-type`
    Rust: (tidak ada padanan)
  - JS: `cross-site`
    Rust: (tidak ada padanan)
  - JS: `id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7`
    Rust: (tidak ada padanan)
  - JS: `sec-ch-ua`
    Rust: (tidak ada padanan)
  - JS: `sec-ch-ua-mobile`
    Rust: (tidak ada padanan)
  - JS: `sec-ch-ua-platform`
    Rust: (tidak ada padanan)
  - JS: `sec-fetch-dest`
    Rust: (tidak ada padanan)
  - JS: `sec-fetch-mode`
    Rust: (tidak ada padanan)
  - JS: `sec-fetch-site`
    Rust: (tidak ada padanan)
  - JS: `u=1, i`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)
  - JS: `✨ *THREADS DOWNLOADER* ✨`
    Rust: (tidak ada padanan)
  - JS: `👤 *Pembuat*: ␀`
    Rust: (tidak ada padanan)
  - JS: `📊 *Jumlah File Media*: ␀ file`
    Rust: (tidak ada padanan)
  - JS: `📝 *Teks Postingan*: ␀`
    Rust: (tidak ada padanan)

### tiktok (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/tiktokdl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/tiktok.rs`

  - JS: `"Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36",`
    Rust: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36`
  - JS: `"Not)A;Brand" ;v="24" , "Chromium" ;v="116"`
    Rust: (tidak ada padanan)
  - JS: `"User-Agent":`
    Rust: `User-Agent`
  - JS: `$("#formatselect option").each((_, el) => {`
    Rust: (tidak ada padanan)
  - JS: `$("#video-info .my-1 span").each((_, el) => {`
    Rust: (tidak ada padanan)
  - JS: `([^`
    Rust: (tidak ada padanan)
  - JS: `(e) => e.type === "nowatermark_hd" || e.type === "nowatermark" || e.type === "video"`
    Rust: (tidak ada padanan)
  - JS: `) || result.data.find((e) => e.type === "watermark") || result.data[0];`
    Rust: (tidak ada padanan)
  - JS: `).replace(/&quot;/g, '`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `)[0]).join(`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `.map((zan, idx) => ({`
    Rust: (tidak ada padanan)
  - JS: `// Also handle savett shape where data contains nowatermark`
    Rust: (tidak ada padanan)
  - JS: `// Ambil URL foto dari thumbnail option sebagai fallback`
    Rust: (tidak ada padanan)
  - JS: `// Deteksi photo walau carousel kosong (thumbnail/profile = photo post)`
    Rust: (tidak ada padanan)
  - JS: `// Fallback if slides empty but data has photo-like URLs`
    Rust: (tidak ada padanan)
  - JS: `// Format select (MP3/watermark/nowatermark/photo) — selalu parse agar mp3 ikut ketangkep`
    Rust: (tidak ada padanan)
  - JS: `// URL array = beberapa resolusi, ambil yang terbaik (paling panjang/hd)`
    Rust: (tidak ada padanan)
  - JS: `// hd fallback: if multiple nowm URLs, treat first as hd`
    Rust: (tidak ada padanan)
  - JS: `// resolved jadi fallback kalau raw gagal.`
    Rust: (tidak ada padanan)
  - JS: `// savett lebih toleran ke raw shortlink (sama seperti tt2 yang kerja),`
    Rust: (tidak ada padanan)
  - JS: `// savett: coba raw dulu, lalu resolved`
    Rust: (tidak ada padanan)
  - JS: `// tikwm: hanya terima full URL (shortlink harus di-resolve)`
    Rust: (tidak ada padanan)
  - JS: `1970`
    Rust: (tidak ada padanan)
  - JS: `?.filter((d) => d.type === "photo" || d.url?.match(/.(jpg|jpeg|png|webp)/i) || result.durations === 0)`
    Rust: (tidak ada padanan)
  - JS: `?1`
    Rust: (tidak ada padanan)
  - JS: `Accept-Language`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: `content-type`
  - JS: `Contoh: `.tt https://vt.tiktok.com/ZSxxxx/`");`
    Rust: `Contoh: `.tt https://vt.tiktok.com/ZSxxxx/``
  - JS: `Jika tetap gagal, pastikan link valid dan video tidak private.`);`
    Rust: `Jika tetap gagal, pastikan link valid dan video tidak private.`
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Mobile Safari/537.36`
    Rust: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36`
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Mobile Safari/537.36`
    Rust: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36`
  - JS: `Referer: "https://www.tiktok.com/",`
    Rust: (tidak ada padanan)
  - JS: `Sec-Ch-Ua`
    Rust: (tidak ada padanan)
  - JS: `Sec-Ch-Ua-Mobile`
    Rust: (tidak ada padanan)
  - JS: `Sec-Ch-Ua-Platform`
    Rust: (tidak ada padanan)
  - JS: `Sec-Fetch-Dest`
    Rust: (tidak ada padanan)
  - JS: `Sec-Fetch-Mode`
    Rust: (tidak ada padanan)
  - JS: `Sec-Fetch-Site`
    Rust: (tidak ada padanan)
  - JS: `Tekan tombol di bawah untuk ambil music.`, m, {`
    Rust: `Tekan tombol di bawah untuk ambil music.`
  - JS: `TikWM error:`
    Rust: (tidak ada padanan)
  - JS: `TikWM tidak mengembalikan data valid`
    Rust: (tidak ada padanan)
  - JS: `X-Requested-With`
    Rust: (tidak ada padanan)
  - JS: `] || []).map((v) => v.split(`
    Rust: (tidak ada padanan)
  - JS: `_provider: "savett",`
    Rust: (tidak ada padanan)
  - JS: `_rawSaveTT: parsed,`
    Rust: (tidak ada padanan)
  - JS: `` +`
    Rust: (tidak ada padanan)
  - JS: ``*Statistik Konten:*`
    Rust: `*Statistik Konten:*`
  - JS: ``*Statistik Video:*`
    Rust: `*Statistik Video:*`
  - JS: ``- Author: *␀* (␀)`
    Rust: `- Author: *␀* (␀)`
  - JS: ``- Caption: ␀`
    Rust: `- Caption: ␀`
  - JS: ``- Comments: *␀*`
    Rust: `- Comments: *␀*`
  - JS: ``- Downloads: *␀*`;`
    Rust: `- Downloads: *␀*`
  - JS: ``- Duration: ␀`
    Rust: `- Duration: ␀`
  - JS: ``- Likes: *␀*`
    Rust: `- Likes: *␀*`
  - JS: ``- Music: ␀ - ␀`
    Rust: `- Music: ␀ - ␀`
  - JS: ``- Region: ␀`
    Rust: `- Region: ␀`
  - JS: ``- Shares: *␀*`
    Rust: `- Shares: *␀*`
  - JS: ``- Uploaded: ␀`
    Rust: `- Uploaded: ␀`
  - JS: ``- Views: *␀*`
    Rust: `- Views: *␀*`
  - JS: ``csrf_token=␀&url=␀`,`
    Rust: (tidak ada padanan)
  - JS: ``🎵 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞  -  𝗗 𝗢 𝗪 𝗡 𝗟 𝗢 𝗔 𝗗 𝗘 𝗥* [␀]`
    Rust: `🎵 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞  -  𝗗 𝗢 𝗪 𝗡 𝗟 𝗢 𝗔 𝗗 𝗘 𝗥* [␀]`
  - JS: ``📌 Contoh: *␀ https://vt.tiktok.com/...*``
    Rust: `📌 Contoh: *␀␀ https://vt.tiktok.com/...*`
  - JS: ``📸 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞  -  𝗗 𝗢 𝗪 𝗡 𝗟 𝗢 𝗔 𝗗 𝗘 𝗥* [␀]`
    Rust: `📸 *𝗧 𝗜 𝗞 𝗧 𝗢 𝗞  -  𝗗 𝗢 𝗪 𝗡 𝗟 𝗢 𝗔 𝗗 𝗘 𝗥* [␀]`
  - JS: `albumMessage: finalSlides,`
    Rust: (tidak ada padanan)
  - JS: `alias: ["tiktokdl", "ttdown"],`
    Rust: (tidak ada padanan)
  - JS: `async function fetchSaveTT(url, csrf, cookie) {`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `async function tiktokDl(url) {`
    Rust: (tidak ada padanan)
  - JS: `async function tiktokDlViaSaveTT(url) {`
    Rust: (tidak ada padanan)
  - JS: `author: { fullname: parsed.username || "-", nickname: parsed.username || "-", avatar: null },`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendButton(m.chat, Buffer.from(videoRes.data), caption, m, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendButton(m.chat, null, `📸 Slide berhasil dikirim!`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMessage(`
    Rust: (tidak ada padanan)
  - JS: `bookmarks: stats[2] || null,`
    Rust: (tidak ada padanan)
  - JS: `buttonParamsJson: JSON.stringify({`
    Rust: (tidak ada padanan)
  - JS: `buttons: [musicButton],`
    Rust: (tidak ada padanan)
  - JS: `caption: idx === 0 ? caption : "",`
    Rust: (tidak ada padanan)
  - JS: `category: "download",`
    Rust: (tidak ada padanan)
  - JS: `comment: parsed.comments || "-",`
    Rust: (tidak ada padanan)
  - JS: `comments: stats[3] || null,`
    Rust: (tidak ada padanan)
  - JS: `console.error("[tiktokDl] handler error:", e);`
    Rust: (tidak ada padanan)
  - JS: `console.warn("[tiktokDl] savett failed:", msg);`
    Rust: (tidak ada padanan)
  - JS: `console.warn("[tiktokDl] tikwm failed:", msg);`
    Rust: (tidak ada padanan)
  - JS: `const $ = cheerio.load(html);`
    Rust: (tidak ada padanan)
  - JS: `const best = [...json.URL].sort((a, b) => String(b).length - String(a).length)[0];`
    Rust: (tidak ada padanan)
  - JS: `const caption =`
    Rust: (tidak ada padanan)
  - JS: `const cleaned = String(raw ||`
    Rust: (tidak ada padanan)
  - JS: `const command = m?.command;`
    Rust: (tidak ada padanan)
  - JS: `const ct = videoRes.headers["content-type"] || "";`
    Rust: (tidak ada padanan)
  - JS: `const data = [];`
    Rust: (tidak ada padanan)
  - JS: `const data = {`
    Rust: (tidak ada padanan)
  - JS: `const detail = e.message?.slice(0, 300) || "Unknown error";`
    Rust: (tidak ada padanan)
  - JS: `const errors = [];`
    Rust: (tidak ada padanan)
  - JS: `const finalSlides = slides.length ? slides : result.data.map((zan, idx) => ({`
    Rust: (tidak ada padanan)
  - JS: `const html = await fetchSaveTT(url, csrf, cookie);`
    Rust: (tidak ada padanan)
  - JS: `const isVideo = result.durations > 0 && result.duration !== "0 Seconds" && result.data.some((d) => d.type.includes("nowatermark") || d.type === "video" || d.type === "watermark");`
    Rust: (tidak ada padanan)
  - JS: `const json = parseSaveTTJson($(el).attr("data-data"));`
    Rust: (tidak ada padanan)
  - JS: `const json = parseSaveTTJson($(el).attr("value"));`
    Rust: (tidak ada padanan)
  - JS: `const json = parseSaveTTJson(raw);`
    Rust: (tidak ada padanan)
  - JS: `const label = $(el).text().toLowerCase();`
    Rust: (tidak ada padanan)
  - JS: `const labels = $("#formatselect option").map((_, el) => $(el).text().toLowerCase()).get();`
    Rust: (tidak ada padanan)
  - JS: `const msg = e?.response?.status ? `TikWM ␀` : e.message;`
    Rust: (tidak ada padanan)
  - JS: `const msg = e?.response?.status ? `saveTT ␀` : e.message;`
    Rust: (tidak ada padanan)
  - JS: `const musicButton = {`
    Rust: (tidak ada padanan)
  - JS: `const parsed = parseSaveTTResponse(html);`
    Rust: (tidak ada padanan)
  - JS: `const pluginConfig = {`
    Rust: (tidak ada padanan)
  - JS: `const prefix = m.prefix;`
    Rust: (tidak ada padanan)
  - JS: `const raw = $(el).attr("value");`
    Rust: (tidak ada padanan)
  - JS: `const res = await axios.post(`
    Rust: (tidak ada padanan)
  - JS: `const resolved = await resolveTikTokUrl(url);`
    Rust: (tidak ada padanan)
  - JS: `const result = await tiktokDl(text);`
    Rust: (tidak ada padanan)
  - JS: `const result = await tiktokDlViaSaveTT(variant);`
    Rust: (tidak ada padanan)
  - JS: `const slides = $(".carousel-item[data-data]");`
    Rust: (tidak ada padanan)
  - JS: `const slides = result.data`
    Rust: (tidak ada padanan)
  - JS: `const stats = [];`
    Rust: (tidak ada padanan)
  - JS: `const text = m.text?.trim();`
    Rust: (tidak ada padanan)
  - JS: `const urlVariants = [url, resolved].filter((u, i, arr) => u && arr.indexOf(u) === i);`
    Rust: (tidak ada padanan)
  - JS: `const urls = Array.isArray(json.URL) ? json.URL : [json.URL];`
    Rust: (tidak ada padanan)
  - JS: `const videoItem = result.data.find(`
    Rust: (tidak ada padanan)
  - JS: `const videoRes = await axios.get(videoItem.url, {`
    Rust: (tidak ada padanan)
  - JS: `const { csrf, cookie } = await getSaveTTToken();`
    Rust: (tidak ada padanan)
  - JS: `cooldown: 10,`
    Rust: (tidak ada padanan)
  - JS: `cover: null,`
    Rust: (tidak ada padanan)
  - JS: `data,`
    Rust: (tidak ada padanan)
  - JS: `data.downloads.nowm.push(...urls);`
    Rust: (tidak ada padanan)
  - JS: `data.downloads.wm.push(...urls);`
    Rust: (tidak ada padanan)
  - JS: `data.mp3.push(...urls);`
    Rust: (tidak ada padanan)
  - JS: `data.slides.push({ index: data.slides.length + 1, url: String(best) });`
    Rust: (tidak ada padanan)
  - JS: `data.slides.push({ index: data.slides.length + 1, url: String(u) });`
    Rust: (tidak ada padanan)
  - JS: `data.type = "photo";`
    Rust: (tidak ada padanan)
  - JS: `data.type = "video";`
    Rust: (tidak ada padanan)
  - JS: `data: [{ type: "mp3", url: parsed.mp3[0] }],`
    Rust: (tidak ada padanan)
  - JS: `data: parsed.slides.map((s) => ({ type: "photo", url: s.url })),`
    Rust: (tidak ada padanan)
  - JS: `description: "Download video/slide TikTok tanpa watermark",`
    Rust: `Download video/slide TikTok tanpa watermark`
  - JS: `display_text: "🎵 Ambil Music",`
    Rust: (tidak ada padanan)
  - JS: `download: "-",`
    Rust: (tidak ada padanan)
  - JS: `downloads: { nowm: [], wm: [] },`
    Rust: (tidak ada padanan)
  - JS: `duration: "0 Seconds",`
    Rust: `- Duration: 12 Seconds`
  - JS: `duration: $("#video-info p.text-muted").first().text().replace(/Duration:/i, "").trim() || null,`
    Rust: (tidak ada padanan)
  - JS: `duration: parsed.duration || "1 Seconds",`
    Rust: (tidak ada padanan)
  - JS: `durations: 0,`
    Rust: (tidak ada padanan)
  - JS: `durations: 1,`
    Rust: (tidak ada padanan)
  - JS: `energi: 1,`
    Rust: (tidak ada padanan)
  - JS: `errors.push(`savett(␀): ␀`);`
    Rust: (tidak ada padanan)
  - JS: `errors.push(`tikwm: ␀`);`
    Rust: (tidak ada padanan)
  - JS: `example: ".tiktok https://vt.tiktok.com/xxx",`
    Rust: `.tiktok https://vt.tiktok.com/xxx`
  - JS: `export { pluginConfig as config, handler, tiktokDl, tiktokDlViaTikWM, tiktokDlViaSaveTT };`
    Rust: (tidak ada padanan)
  - JS: `for (const variant of urlVariants) {`
    Rust: (tidak ada padanan)
  - JS: `function parseSaveTTJson(raw) {`
    Rust: (tidak ada padanan)
  - JS: `function parseSaveTTResponse(html) {`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7`
    Rust: (tidak ada padanan)
  - JS: `id: `␀ttmp3 ␀`,`
    Rust: (tidak ada padanan)
  - JS: `if (!/tiktok.com|vt.tiktok|vm.tiktok/i.test(text)) {`
    Rust: (tidak ada padanan)
  - JS: `if (!csrf || !cookie) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!data.type && (data.downloads.nowm.length || data.downloads.wm.length)) {`
    Rust: (tidak ada padanan)
  - JS: `if (!data.type) {`
    Rust: (tidak ada padanan)
  - JS: `if (!json || !json.URL) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!raw) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!text) {`
    Rust: (tidak ada padanan)
  - JS: `if (!videoItem?.url) throw new Error("URL video tidak ditemukan");`
    Rust: (tidak ada padanan)
  - JS: `if (/.(jpe?g|png|webp)/i.test(String(u)) && !data.slides.some((s) => s.url === u)) {`
    Rust: (tidak ada padanan)
  - JS: `if (ct.includes("text/html")) throw new Error("Gagal mendapatkan file video (response HTML)");`
    Rust: (tidak ada padanan)
  - JS: `if (data.type === "photo") {`
    Rust: (tidak ada padanan)
  - JS: `if (finalSlides.length === 0) throw new Error("Tidak ada slide foto ditemukan");`
    Rust: (tidak ada padanan)
  - JS: `if (isVideo) {`
    Rust: (tidak ada padanan)
  - JS: `if (json && Array.isArray(json.URL) && json.URL.length) {`
    Rust: (tidak ada padanan)
  - JS: `if (json && Array.isArray(json.URL)) {`
    Rust: (tidak ada padanan)
  - JS: `if (label.includes("mp3")) data.mp3.push(...urls);`
    Rust: (tidak ada padanan)
  - JS: `if (label.includes("mp4") && !label.includes("watermark")) {`
    Rust: (tidak ada padanan)
  - JS: `if (labels.some((l) => l.includes("thumbnail") || l.includes("profile"))) {`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.downloads.nowm[0]) data.push({ type: "nowatermark", url: parsed.downloads.nowm[0] });`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.downloads.wm[0]) data.push({ type: "watermark", url: parsed.downloads.wm[0] });`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.mp3.length > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.mp3[0]) data.push({ type: "mp3", url: parsed.mp3[0] });`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.type === "photo" && parsed.slides.length > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.type === "video" && parsed.downloads.nowm.length > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (result) return result;`
    Rust: (tidak ada padanan)
  - JS: `if (slides.length) {`
    Rust: (tidak ada padanan)
  - JS: `image: { url: zan.url },`
    Rust: (tidak ada padanan)
  - JS: `isEnabled: true,`
    Rust: (tidak ada padanan)
  - JS: `isGroup: false,`
    Rust: (tidak ada padanan)
  - JS: `isOwner: false,`
    Rust: (tidak ada padanan)
  - JS: `isPremium: false,`
    Rust: (tidak ada padanan)
  - JS: `isPrivate: false,`
    Rust: (tidak ada padanan)
  - JS: `json.URL.forEach((u) => {`
    Rust: (tidak ada padanan)
  - JS: `likes: parsed.likes || "-",`
    Rust: (tidak ada padanan)
  - JS: `likes: stats[1] || null,`
    Rust: (tidak ada padanan)
  - JS: `m.chat,`
    Rust: (tidak ada padanan)
  - JS: `m.react("✅");`
    Rust: (tidak ada padanan)
  - JS: `m.react("❌");`
    Rust: (tidak ada padanan)
  - JS: `m.react("🕕");`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`❌ *Gagal mengunduh TikTok*`
    Rust: `❌ *Gagal mengunduh TikTok*`
  - JS: `maxRedirects: 5,`
    Rust: (tidak ada padanan)
  - JS: `mimetype: videoRes.headers["content-type"]?.includes("video") ? videoRes.headers["content-type"] : "video/mp4",`
    Rust: (tidak ada padanan)
  - JS: `mp3: [],`
    Rust: (tidak ada padanan)
  - JS: `music_info: { title: "-", author: "-", url: parsed.mp3[0] || null },`
    Rust: (tidak ada padanan)
  - JS: `music_info: { title: "-", author: "-", url: parsed.mp3[0] },`
    Rust: (tidak ada padanan)
  - JS: `name: "quick_reply",`
    Rust: (tidak ada padanan)
  - JS: `name: ["tiktok", "tt", "ttmp4"],`
    Rust: (tidak ada padanan)
  - JS: `region: "-",`
    Rust: (tidak ada padanan)
  - JS: `responseType: "arraybuffer",`
    Rust: (tidak ada padanan)
  - JS: `return JSON.parse(cleaned);`
    Rust: (tidak ada padanan)
  - JS: `return await tiktokDlViaTikWM(resolved);`
    Rust: (tidak ada padanan)
  - JS: `return data;`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return m.reply("❌ URL tidak valid. Gunakan link TikTok yang valid.`
    Rust: `❌ URL tidak valid. Gunakan link TikTok yang valid.`
  - JS: `return null;`
    Rust: (tidak ada padanan)
  - JS: `return res.data;`
    Rust: (tidak ada padanan)
  - JS: `return {`
    Rust: (tidak ada padanan)
  - JS: `return { csrf, cookie };`
    Rust: (tidak ada padanan)
  - JS: `return;`
    Rust: (tidak ada padanan)
  - JS: `same-origin`
    Rust: (tidak ada padanan)
  - JS: `share: parsed.shares || "-",`
    Rust: (tidak ada padanan)
  - JS: `shares: stats[4] || null,`
    Rust: (tidak ada padanan)
  - JS: `slides.each((_, el) => {`
    Rust: (tidak ada padanan)
  - JS: `slides: [],`
    Rust: (tidak ada padanan)
  - JS: `stats.push($(el).text().trim());`
    Rust: (tidak ada padanan)
  - JS: `stats: {`
    Rust: (tidak ada padanan)
  - JS: `status: true,`
    Rust: (tidak ada padanan)
  - JS: `taken_at: "-",`
    Rust: (tidak ada padanan)
  - JS: `throw new Error("SaveTT tidak mengembalikan data video/slide");`
    Rust: (tidak ada padanan)
  - JS: `throw new Error(errors.join(" | ") || "Semua provider TikTok gagal");`
    Rust: (tidak ada padanan)
  - JS: `timeout: 60000,`
    Rust: (tidak ada padanan)
  - JS: `title: parsed.username || "",`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `type: "video",`
    Rust: (tidak ada padanan)
  - JS: `type: null,`
    Rust: (tidak ada padanan)
  - JS: `usage: ".tiktok <url>",`
    Rust: `.tiktok <url>`
  - JS: `username: $("#video-info h3").first().text().trim(),`
    Rust: (tidak ada padanan)
  - JS: `views: parsed.views || "-",`
    Rust: (tidak ada padanan)
  - JS: `views: stats[0] || null,`
    Rust: (tidak ada padanan)
  - JS: `{`
    Rust: (tidak ada padanan)
  - JS: `{ headers: { ...SAVETT_HEADERS, Cookie: cookie }, timeout: 15000, validateStatus: (s) => s >= 200 && s < 300 }`
    Rust: (tidak ada padanan)
  - JS: `{ quoted: m }`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {`
    Rust: (tidak ada padanan)
  - JS: `} else if (label.includes("mp3")) {`
    Rust: (tidak ada padanan)
  - JS: `} else if (label.includes("watermark")) {`
    Rust: (tidak ada padanan)
  - JS: `} else {`
    Rust: (tidak ada padanan)
  - JS: `})) || [];`
    Rust: (tidak ada padanan)
  - JS: `}));`
    Rust: (tidak ada padanan)
  - JS: `}),`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `},`
    Rust: (tidak ada padanan)
  - JS: `};`
    Rust: (tidak ada padanan)

### tiktok2 (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/tiktokdl2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/tiktokdl2.rs`

  - JS: `$(`
    Rust: (tidak ada padanan)
  - JS: `'))`
    Rust: (tidak ada padanan)
  - JS: `'Referer': 'https://www.tiktok.com/'`
    Rust: (tidak ada padanan)
  - JS: `'User-Agent': 'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36',`
    Rust: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36`
  - JS: `([^`
    Rust: (tidak ada padanan)
  - JS: `)`
    Rust: (tidak ada padanan)
  - JS: `))`
    Rust: (tidak ada padanan)
  - JS: `).each((_, el) => {`
    Rust: (tidak ada padanan)
  - JS: `> Konten ini tidak memiliki video/slide, mengirim audio saja...`)`
    Rust: `> Konten ini tidak memiliki video/slide, mengirim audio saja...`
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Mobile Safari/537.36`
    Rust: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36`
  - JS: `` +`
    Rust: (tidak ada padanan)
  - JS: ``> Contoh: ␀tiktok2 https://vt.tiktok.com/xxx``
    Rust: `> Contoh: ␀tiktok2 https://vt.tiktok.com/xxx`
  - JS: ``⏱️ Duration: ␀``
    Rust: `⏱️ Duration: -`
  - JS: ``┃ ㊗ ᴜsᴀɢᴇ: `␀tiktok2 <url>``
    Rust: `┃ ㊗ ᴜsᴀɢᴇ: `␀tiktok2 <url>``
  - JS: ``╭┈┈⬡「 🎵 *ᴛɪᴋᴛᴏᴋ ᴅᴏᴡɴʟᴏᴀᴅ* 」`
    Rust: `╭┈┈⬡「 🎵 *ᴛɪᴋᴛᴏᴋ ᴅᴏᴡɴʟᴏᴀᴅ* 」`
  - JS: ``╰┈┈⬡`
    Rust: `╰┈┈⬡`
  - JS: ``✅ *Done kak*`
    Rust: `✅ *Done kak*`
  - JS: ``� Comments: ␀ | 🔗 Shares: ␀`
    Rust: `u␀ Comments: - | 🔗 Shares: -`
  - JS: ``👁️ Views: ␀ | ❤️ Likes: ␀`
    Rust: `👁️ Views: - | ❤️ Likes: -`
  - JS: ``👤 *␀*`
    Rust: `👤 *␀*`
  - JS: `albumMessage: {`
    Rust: (tidak ada padanan)
  - JS: `alias: ['tiktokdl2', 'ttdown2'],`
    Rust: (tidak ada padanan)
  - JS: `associationType: 1,`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `async function savett(url) {`
    Rust: (tidak ada padanan)
  - JS: `audio: { url: result.mp3[0] },`
    Rust: (tidak ada padanan)
  - JS: `await m.reply(`📸 *Mengirim ␀ slide...*`)`
    Rust: `📸 *Mengirim ␀ slide...*`
  - JS: `await sock.relayMessage(msg.key.remoteJid, msg.message, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.relayMessage(opener.key.remoteJid, opener.message, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMessage(`
    Rust: (tidak ada padanan)
  - JS: `caption,`
    Rust: (tidak ada padanan)
  - JS: `caption: i === 0 ? caption : ''`
    Rust: (tidak ada padanan)
  - JS: `category: 'download',`
    Rust: (tidak ada padanan)
  - JS: `console.error('[TikTokDL2] Error:', err)`
    Rust: (tidak ada padanan)
  - JS: `console.error(`[TikTok2] Failed to download slide ␀:`, e.message)`
    Rust: (tidak ada padanan)
  - JS: `const caption =`
    Rust: (tidak ada padanan)
  - JS: `const html = await fetchTikTok(url, csrf, cookie)`
    Rust: (tidak ada padanan)
  - JS: `const imgUrl = result.slides[i].url`
    Rust: (tidak ada padanan)
  - JS: `const json = JSON.parse(raw.replace(/&quot;/g,`
    Rust: (tidak ada padanan)
  - JS: `const label = $(el).text().toLowerCase()`
    Rust: (tidak ada padanan)
  - JS: `const mediaList = []`
    Rust: (tidak ada padanan)
  - JS: `const msg = await generateWAMessage(`
    Rust: (tidak ada padanan)
  - JS: `const opener = generateWAMessageFromContent(`
    Rust: (tidak ada padanan)
  - JS: `const pluginConfig = {`
    Rust: (tidak ada padanan)
  - JS: `const raw = $(el).attr(`
    Rust: (tidak ada padanan)
  - JS: `const res = await axios.get(imgUrl, {`
    Rust: (tidak ada padanan)
  - JS: `const result = await savett(url)`
    Rust: (tidak ada padanan)
  - JS: `const url = m.text?.trim()`
    Rust: (tidak ada padanan)
  - JS: `const videoRes = await axios.get(result.downloads.nowm[0], {`
    Rust: (tidak ada padanan)
  - JS: `const { csrf, cookie } = await getToken()`
    Rust: (tidak ada padanan)
  - JS: `content,`
    Rust: (tidak ada padanan)
  - JS: `cooldown: 10,`
    Rust: (tidak ada padanan)
  - JS: `data.downloads.nowm.push(...json.URL)`
    Rust: (tidak ada padanan)
  - JS: `data.downloads.wm.push(...json.URL)`
    Rust: (tidak ada padanan)
  - JS: `data.mp3.push(...json.URL)`
    Rust: (tidak ada padanan)
  - JS: `data.slides.push({ index: data.slides.length + 1, url })`
    Rust: (tidak ada padanan)
  - JS: `data.type =`
    Rust: (tidak ada padanan)
  - JS: `description: 'Download video/slide TikTok tanpa watermark',`
    Rust: `Download video/slide TikTok tanpa watermark`
  - JS: `energi: 1,`
    Rust: (tidak ada padanan)
  - JS: `example: '.tiktok2 https://vt.tiktok.com/xxx',`
    Rust: `.tiktok2 https://vt.tiktok.com/xxx`
  - JS: `expectedImageCount: mediaList.length,`
    Rust: (tidak ada padanan)
  - JS: `expectedVideoCount: 0`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler }`
    Rust: (tidak ada padanan)
  - JS: `for (const content of mediaList) {`
    Rust: (tidak ada padanan)
  - JS: `for (let i = 0; i < result.slides.length; i++) {`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `if (!imgUrl) continue`
    Rust: (tidak ada padanan)
  - JS: `if (!json.URL) return`
    Rust: (tidak ada padanan)
  - JS: `if (!raw) return`
    Rust: (tidak ada padanan)
  - JS: `if (!url) {`
    Rust: (tidak ada padanan)
  - JS: `if (!url.match(/tiktok.com|vt.tiktok/i)) {`
    Rust: (tidak ada padanan)
  - JS: `if (Array.isArray(json.URL)) {`
    Rust: (tidak ada padanan)
  - JS: `if (label.includes('mp3')) {`
    Rust: (tidak ada padanan)
  - JS: `if (label.includes('mp4') && !label.includes('watermark')) {`
    Rust: (tidak ada padanan)
  - JS: `if (label.includes('watermark')) {`
    Rust: (tidak ada padanan)
  - JS: `if (mediaList.length === 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (result.mp3.length > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (result.type === 'photo' && result.slides.length > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (result.type === 'video' && result.downloads.nowm.length > 0) {`
    Rust: (tidak ada padanan)
  - JS: `image: Buffer.from(res.data),`
    Rust: (tidak ada padanan)
  - JS: `isEnabled: true`
    Rust: (tidak ada padanan)
  - JS: `isGroup: false,`
    Rust: (tidak ada padanan)
  - JS: `isOwner: false,`
    Rust: (tidak ada padanan)
  - JS: `isPremium: false,`
    Rust: (tidak ada padanan)
  - JS: `isPrivate: false,`
    Rust: (tidak ada padanan)
  - JS: `json.URL.forEach(url => {`
    Rust: (tidak ada padanan)
  - JS: `m.chat,`
    Rust: (tidak ada padanan)
  - JS: `m.react('⏱️')`
    Rust: (tidak ada padanan)
  - JS: `m.react('☢')`
    Rust: (tidak ada padanan)
  - JS: `m.react('✅')`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`🍀 *NOTE*`
    Rust: (tidak ada padanan)
  - JS: `m.reply(te(m.prefix, m.command, m.pushName))`
    Rust: (tidak ada padanan)
  - JS: `mediaList.push({`
    Rust: (tidak ada padanan)
  - JS: `messageAssociation: {`
    Rust: (tidak ada padanan)
  - JS: `messageContextInfo: { messageSecret: crypto.randomBytes(32) },`
    Rust: (tidak ada padanan)
  - JS: `messageId: msg.key.id`
    Rust: (tidak ada padanan)
  - JS: `messageId: opener.key.id`
    Rust: (tidak ada padanan)
  - JS: `messageSecret: crypto.randomBytes(32),`
    Rust: (tidak ada padanan)
  - JS: `mimetype: 'audio/mpeg'`
    Rust: (tidak ada padanan)
  - JS: `mimetype: 'video/mp4',`
    Rust: (tidak ada padanan)
  - JS: `msg.message.messageContextInfo = {`
    Rust: (tidak ada padanan)
  - JS: `name: ['tiktok2', 'tt2'],`
    Rust: (tidak ada padanan)
  - JS: `opener.key.remoteJid,`
    Rust: (tidak ada padanan)
  - JS: `parentMessageKey: opener.key`
    Rust: (tidak ada padanan)
  - JS: `quoted: m,`
    Rust: (tidak ada padanan)
  - JS: `responseType: 'arraybuffer',`
    Rust: (tidak ada padanan)
  - JS: `return`
    Rust: (tidak ada padanan)
  - JS: `return data`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return m.reply('❌ URL tidak valid. Gunakan link TikTok.')`
    Rust: `❌ URL tidak valid. Gunakan link TikTok.`
  - JS: `return parseResponse(html)`
    Rust: (tidak ada padanan)
  - JS: `throw new Error('Gagal mengunduh gambar slide')`
    Rust: (tidak ada padanan)
  - JS: `throw new Error('Tidak ada media yang dapat diunduh')`
    Rust: (tidak ada padanan)
  - JS: `timeout: 30000`
    Rust: (tidak ada padanan)
  - JS: `timeout: 60000,`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `upload: sock.waUploadToServer`
    Rust: (tidak ada padanan)
  - JS: `usage: '.tiktok2 <url>',`
    Rust: `.tiktok2 <url>`
  - JS: `userJid: jidNormalizedUser(sock.user.id),`
    Rust: (tidak ada padanan)
  - JS: `video: Buffer.from(videoRes.data),`
    Rust: (tidak ada padanan)
  - JS: `{`
    Rust: (tidak ada padanan)
  - JS: `{ quoted: m }`
    Rust: (tidak ada padanan)
  - JS: `{ upload: sock.waUploadToServer }`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {}`
    Rust: (tidak ada padanan)
  - JS: `})`
    Rust: (tidak ada padanan)
  - JS: `},`
    Rust: (tidak ada padanan)

### ttmp3 (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/ttmp3.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/ttmp3.rs`

  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36`
    Rust: `Mozilla/5.0 (Linux; Android 15; SM-F958 Build/AP3A.240905.015) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.6723.86 Mobile Safari/537.36`
  - JS: `TikTok_Audio_␀.mp3`
    Rust: (tidak ada padanan)
  - JS: `[TTMP3] fetch mp3 langsung gagal, fallback ekstrak dari video:`
    Rust: (tidak ada padanan)
  - JS: `[TikTokDL] Error:`
    Rust: (tidak ada padanan)
  - JS: `fluent-ffmpeg`
    Rust: (tidak ada padanan)
  - JS: `ttmp3_␀.mp3`
    Rust: (tidak ada padanan)
  - JS: `ttmp3_␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `␀_␀`
    Rust: (tidak ada padanan)

### videy (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/videy.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/videy.rs`

  - JS: `!data?.url`
    Rust: (tidak ada padanan)
  - JS: `aggregator.hit`
    Rust: (tidak ada padanan)
  - JS: `data.url`
    Rust: (tidak ada padanan)
  - JS: `notvidey.co`
    Rust: `videy.co`
  - JS: `videy.co.evil.example`
    Rust: (tidak ada padanan)
  - JS: `www.videy.co`
    Rust: `.videy.co`
  - JS: `␀videy`
    Rust: `.videy`

### ytmp3 (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/ytmp3.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/ytmp3.rs`

  - JS: `&& parsed.protocol !==`
    Rust: (tidak ada padanan)
  - JS: `']+/i.exec(mentah)?.[0];`
    Rust: (tidak ada padanan)
  - JS: `)`
    Rust: (tidak ada padanan)
  - JS: `) return false;`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `*`
    Rust: (tidak ada padanan)
  - JS: `* Empat plugin YouTube lain tidak punya seam ini karena tidak ada pemanggilnya:`
    Rust: (tidak ada padanan)
  - JS: `* Satu-satunya jalan ke media: scraper lokal lebih dulu, aggregator jadi`
    Rust: (tidak ada padanan)
  - JS: `* `deps.resolve` hanya untuk test — `tests/upstream-diagnostics.test.mjs``
    Rust: (tidak ada padanan)
  - JS: `* cadangan. Jalur aggregator tidak lagi diperiksa di plugin ini karena `hit``
    Rust: (tidak ada padanan)
  - JS: `* menyuntikkan kegagalan aggregator lewat konteks plugin, tanpa jaringan.`
    Rust: (tidak ada padanan)
  - JS: `* sama sekali.`
    Rust: (tidak ada padanan)
  - JS: `* satu jalur suntik saja sudah cukup.`
    Rust: (tidak ada padanan)
  - JS: `* sudah menolak sendiri kalau key kosong, dan jalur lokal tidak butuh key`
    Rust: (tidak ada padanan)
  - JS: `*/`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `, { url: String(url).trim(), format:`
    Rust: (tidak ada padanan)
  - JS: `/**`
    Rust: (tidak ada padanan)
  - JS: `//`
    Rust: `://`
  - JS: `// WhatsApp karena container dan ekstensinya tidak cocok.`
    Rust: (tidak ada padanan)
  - JS: `// `CapabilityError.message` hanya menyatakan`
    Rust: (tidak ada padanan)
  - JS: `// `mimetype` dan ekstensi mengikuti `format` yang dilaporkan kapabilitas, bukan`
    Rust: (tidak ada padanan)
  - JS: `// `normalize` menolak respons tanpa URL unduhan, jadi `url` di sini tidak`
    Rust: (tidak ada padanan)
  - JS: `// alasannya ada di `tried`. Tanpa ikut dicetak,`
    Rust: (tidak ada padanan)
  - JS: `// dikirim sebagai `audio/mpeg` dengan nama `.mp3` ditolak oleh pemutar dan`
    Rust: (tidak ada padanan)
  - JS: `// lokal justru yang jalan pertama, jadi jalur buffer itu akan jadi jalur`
    Rust: (tidak ada padanan)
  - JS: `// mp3 ini (probe 2026-10-03, task-8-report.md §8.5), dan byte WebM yang`
    Rust: (tidak ada padanan)
  - JS: `// mp3 yang diketik di sini: aggregator ada yang menjawab webm/opus untuk slot`
    Rust: (tidak ada padanan)
  - JS: `// mungkin kosong. Versi lama melakukan `fallbackToMp3Buffer` — mengunduh`
    Rust: (tidak ada padanan)
  - JS: `// seluruh audio ke memori hanya kalau aggregator gagal; sekarang scraper`
    Rust: (tidak ada padanan)
  - JS: `// utama dan menarik video 2 jam ke kotak 1GB setiap kali `.ytmp3` dipakai.`
    Rust: (tidak ada padanan)
  - JS: `: String(err?.message ?? err);`
    Rust: (tidak ada padanan)
  - JS: `;`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock, resolve }) {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMedia(m.chat, download, null, m, {`
    Rust: (tidak ada padanan)
  - JS: `console.error(`[YTMP3] gagal — ␀`);`
    Rust: (tidak ada padanan)
  - JS: `const dasar = lengkap ?? mentah.split(/s+/).find((token) => /youtu/i.test(token)) ?? mentah;`
    Rust: (tidak ada padanan)
  - JS: `const host = parsed.hostname.toLowerCase();`
    Rust: (tidak ada padanan)
  - JS: `const run = deps.resolve ?? ((capability, args) => resolver.resolve(capability, args));`
    Rust: (tidak ada padanan)
  - JS: `const url = m.text?.trim();`
    Rust: (tidak ada padanan)
  - JS: `const { data } = await run(`
    Rust: (tidak ada padanan)
  - JS: `const { title, format, url: download } = await getAudioDownload(url, { resolve });`
    Rust: (tidak ada padanan)
  - JS: `export async function getAudioDownload(url, deps = {}) {`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `fileName: `${title ||`
    Rust: (tidak ada padanan)
  - JS: `if (!hostYoutube(url)) return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `if (!url)`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.protocol !==`
    Rust: (tidak ada padanan)
  - JS: `let parsed;`
    Rust: (tidak ada padanan)
  - JS: `m.react(`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`
    Rust: (tidak ada padanan)
  - JS: `mimetype: mimetypeFor(format),`
    Rust: (tidak ada padanan)
  - JS: `notyoutube.com`
    Rust: `youtube.com`
  - JS: `parsed = new URL(/^[a-z][a-z0-9+.-]*:/i.test(dasar) ? dasar : `https://␀`);`
    Rust: (tidak ada padanan)
  - JS: `ptt: false,`
    Rust: (tidak ada padanan)
  - JS: `putar https://youtu.be/... dong`
    Rust: `putar https://youtu.be/xxx dong`
  - JS: `return data;`
    Rust: (tidak ada padanan)
  - JS: `return false;`
    Rust: (tidak ada padanan)
  - JS: `return host ===`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`Contoh: ␀ytmp3 https://youtube.com/watch?v=xxx`);`
    Rust: `Contoh: ␀ytmp3 https://youtube.com/watch?v=xxx`
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `type:`
    Rust: (tidak ada padanan)
  - JS: `youtu.be/...`
    Rust: `youtu.be`
  - JS: `youtube.com.evil.example`
    Rust: (tidak ada padanan)
  - JS: `|| host ===`
    Rust: (tidak ada padanan)
  - JS: `|| host.endsWith(`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `}.␀`,`
    Rust: (tidak ada padanan)

### ytmp4 (download)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/download/ytmp4.js`
- Rust: `/workspaces/oktz-rs/plugin/src/download/ytmp4.rs`

  - JS: `&& parsed.protocol !==`
    Rust: (tidak ada padanan)
  - JS: `']+/i.exec(mentah)?.[0];`
    Rust: (tidak ada padanan)
  - JS: `)`
    Rust: (tidak ada padanan)
  - JS: `) return false;`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `, { url: String(url).trim(), format:`
    Rust: (tidak ada padanan)
  - JS: `// `normalize` menolak respons tanpa URL unduhan, jadi hasil di sini tidak`
    Rust: (tidak ada padanan)
  - JS: `// `sendMedia` dengan tidak ada file.`
    Rust: (tidak ada padanan)
  - JS: `// mungkin kosong — versi lama bisa mengembalikan `undefined` lalu mengirim`
    Rust: (tidak ada padanan)
  - JS: `: String(err?.message ?? err);`
    Rust: (tidak ada padanan)
  - JS: `? err.tried.map((t) => `␀: ␀`).join(`
    Rust: (tidak ada padanan)
  - JS: `async function getVideoDownloadUrl(url) {`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMedia(m.chat, downloadUrl, null, m, {`
    Rust: (tidak ada padanan)
  - JS: `console.error(`[YTMP4] gagal — ␀`);`
    Rust: (tidak ada padanan)
  - JS: `const dasar = lengkap ?? mentah.split(/s+/).find((token) => /youtu/i.test(token)) ?? mentah;`
    Rust: (tidak ada padanan)
  - JS: `const downloadUrl = await getVideoDownloadUrl(url);`
    Rust: (tidak ada padanan)
  - JS: `const host = parsed.hostname.toLowerCase();`
    Rust: (tidak ada padanan)
  - JS: `const rincian = Array.isArray(err?.tried)`
    Rust: (tidak ada padanan)
  - JS: `const url = m.text?.trim();`
    Rust: (tidak ada padanan)
  - JS: `const { data } = await resolver.resolve(`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `if (!hostYoutube(url)) return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `if (!url)`
    Rust: (tidak ada padanan)
  - JS: `if (parsed.protocol !==`
    Rust: (tidak ada padanan)
  - JS: `let parsed;`
    Rust: (tidak ada padanan)
  - JS: `m.react(`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`
    Rust: (tidak ada padanan)
  - JS: `m.youtube.com`
    Rust: (tidak ada padanan)
  - JS: `notyoutube.com`
    Rust: (tidak ada padanan)
  - JS: `parsed = new URL(/^[a-z][a-z0-9+.-]*:/i.test(dasar) ? dasar : `https://␀`);`
    Rust: (tidak ada padanan)
  - JS: `putar https://youtu.be/... dong`
    Rust: (tidak ada padanan)
  - JS: `return data.url;`
    Rust: (tidak ada padanan)
  - JS: `return false;`
    Rust: (tidak ada padanan)
  - JS: `return host ===`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`Contoh: ␀ytmp4 https://youtube.com/watch?v=xxx`);`
    Rust: `Contoh: ␀ytmp4 https://youtube.com/watch?v=xxx`
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `type:`
    Rust: (tidak ada padanan)
  - JS: `youtu.be/...`
    Rust: (tidak ada padanan)
  - JS: `youtube.com.evil.example`
    Rust: (tidak ada padanan)
  - JS: `|| host ===`
    Rust: (tidak ada padanan)
  - JS: `|| host.endsWith(`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)

### glitchtext (ephoto)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/ephoto/ephoto.js`
- Rust: `/workspaces/oktz-rs/plugin/src/ephoto/ephoto.rs`

  - JS: `*/*`
    Rust: (tidak ada padanan)
  - JS: `Accept-Language`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `en-US,en;q=0.9`
    Rust: (tidak ada padanan)
  - JS: `form-data`
    Rust: (tidak ada padanan)
  - JS: `input[name=build_server]`
    Rust: (tidak ada padanan)
  - JS: `input[name=build_server_id]`
    Rust: (tidak ada padanan)
  - JS: `input[name=form_value_input]`
    Rust: (tidak ada padanan)
  - JS: `input[name=token]`
    Rust: (tidak ada padanan)
  - JS: `set-cookie`
    Rust: (tidak ada padanan)
  - JS: `text[]`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)

### bucin (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/bucin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/bucin.rs`

  - JS: `bucin.json`
    Rust: (tidak ada padanan)

### cekkhodam (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/cekkhodam.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/cekkhodam.rs`

  - JS: `cekkhodam tts error:`
    Rust: (tidak ada padanan)
  - JS: `google-tts-api`
    Rust: (tidak ada padanan)
  - JS: `khodam-␀.mp3`
    Rust: (tidak ada padanan)

### confess (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/confess.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/confess.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `62`
    Rust: `62␀`
  - JS: `Hmm kak, nomor `␀` belum bisa dipastikan terdaftar di WhatsApp, jadi pesan belum dikirim. Coba lagi nanti kak! 🙏`
    Rust: `Hmm kak, nomor `␀` belum bisa dipastikan terdaftar di WhatsApp,`
  - JS: `Hmm kak, verifikasi nomor `␀` ke WhatsApp gagal (timeout/koneksi), jadi pesan belum dikirim ya. Coba lagi nanti kak! 🙏`
    Rust: (tidak ada padanan)
  - JS: `Tidak diketahui`
    Rust: (tidak ada padanan)

### gachahusbu (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/gachahusbu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/gachahusbu.rs`

  - JS: `(␀/100)`
    Rust: (tidak ada padanan)
  - JS: `Dia sedang berusaha meluluhkan hatimu!`
    Rust: `Dia sedang berusaha melulukkan hatimu!`
  - JS: `Jangan serakah! Jaga husbu yang kamu miliki sekarang. ␀ Ketik *␀husbuku* untuk berinteraksi dengannya.`
    Rust: (tidak ada padanan)
  - JS: `Kamu mengembalikan barang-barangnya dan memintanya untuk pergi. Dia menatapmu dengan mata kecewa yang mendalam, berbalik tanpa sepatah kata pun, lalu menghilang di tengah hujan.`
    Rust: (tidak ada padanan)
  - JS: `Karena kasih sayangnya habis (0), dia mengemasi seluruh barang-barangnya dan pergi. Kamu kehilangan husbumu! *(Ketik ␀gachahusbu untuk memulai ulang)*`
    Rust: (tidak ada padanan)
  - JS: `Simulasi kencan virtual interaktif untuk mendapatkan laki-laki anime idamanmu! Tarik perhatiannya, buat dia jatuh cinta, dan nikahi dia!`
    Rust: `Simulasi kencan virtual interaktif untuk mendapatkan laki-laki anime idamanmu!`
  - JS: `[GachaHusbu] Pinterest API error:`
    Rust: (tidak ada padanan)
  - JS: `versi WhatsApp tidak mendukung`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction hadiah`
    Rust: `␀husbuaction ␀`
  - JS: `␀husbuaction klaim`
    Rust: `␀husbuaction ␀`
  - JS: `␀husbuaction menu_belanja`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_bioskop`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_bulanmadu`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_cium`
    Rust: `␀husbuaction ␀`
  - JS: `␀husbuaction menu_jalanjalan`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_kafe`
    Rust: `␀husbuaction ␀`
  - JS: `␀husbuaction menu_lainnya`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_mandi`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_peluk`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction menu_tidur`
    Rust: (tidak ada padanan)
  - JS: `␀husbuaction mesra`
    Rust: `␀husbuaction ␀`
  - JS: `␀husbuaction nikah`
    Rust: `␀husbuaction ␀`
  - JS: `␀husbuaction rayu`
    Rust: `␀husbuaction ␀`
  - JS: `✨ *EVENT:* ␀␀␀ aff)` : ""}␀ koin)` : ""}`
    Rust: (tidak ada padanan)
  - JS: `✨ +␀ EXP␀ Koin` : ""}`
    Rust: (tidak ada padanan)
  - JS: `💢 *HENTIKAN!* Kamu bertindak terlalu agresif saat perasaannya masih samar! *␀* menepis tanganmu dengan wajah marah dan kecewa.`
    Rust: `💢 *HENTIKAN!* Kamu bertindak terlalu agresif saat perasaannya masih samar! *␀*`
  - JS: `🗓️ *Hari ke-␀*␀` : ""}`
    Rust: `🗓️ *Hari ke-␀*␀`

### gachawaifu (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/gachawaifu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/gachawaifu.rs`

  - JS: `(␀/100)`
    Rust: `␀␀ (␀/100)`
  - JS: `Karena kasih sayangnya habis (0), dia pergi meninggalkan surat basah air mata. Kamu kehilangan waifumu! *(Ketik ␀gachawaifu untuk memulai ulang)*`
    Rust: (tidak ada padanan)
  - JS: `[GachaWaifu] Pinterest API error:`
    Rust: (tidak ada padanan)
  - JS: `versi WhatsApp tidak mendukung`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction hadiah`
    Rust: `␀waifuaction ␀`
  - JS: `␀waifuaction klaim`
    Rust: `␀waifuaction ␀`
  - JS: `␀waifuaction menu_belanja`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_bioskop`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_bulanmadu`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_cium`
    Rust: `␀waifuaction ␀`
  - JS: `␀waifuaction menu_jalanjalan`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_kafe`
    Rust: `␀waifuaction ␀`
  - JS: `␀waifuaction menu_lainnya`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_mandi`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_peluk`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction menu_tidur`
    Rust: (tidak ada padanan)
  - JS: `␀waifuaction mesra`
    Rust: `␀waifuaction ␀`
  - JS: `␀waifuaction nikah`
    Rust: `␀waifuaction ␀`
  - JS: `␀waifuaction rayu`
    Rust: `␀waifuaction ␀`
  - JS: `✨ *EVENT:* ␀␀␀ aff)` : ""}␀ koin)` : ""}`
    Rust: (tidak ada padanan)
  - JS: `✨ +␀ EXP␀ Koin` : ""}`
    Rust: (tidak ada padanan)
  - JS: `🗓️ *Hari ke-␀*␀` : ""}`
    Rust: `🗓️ *Hari ke-␀*␀`

### husbualbum (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/husbualbum.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/husbualbum.rs`

  - JS: `*10 Pull Terakhir:*`
    Rust: (tidak ada padanan)
  - JS: `*Distribusi Tier:*`
    Rust: (tidak ada padanan)
  - JS: `␀. ␀ *␀* — ␀`
    Rust: (tidak ada padanan)
  - JS: `🍀 *Luck Score:* ␀ ␀`
    Rust: `🍀 *Luck Score:* 0 😅`
  - JS: `🎯 *Pity aktif:* ␀/20 roll tanpa Epic+`
    Rust: `🎯 *Pity aktif:* 7/20 roll tanpa Epic+`
  - JS: `🎲 *Total Gacha:* ␀`
    Rust: `🎲 *Total Gacha:* 4`
  - JS: `🏆 *Pull Paling Langka:* ␀ ␀ (␀)`
    Rust: `🏆 *Pull Paling Langka:* 🔴 Ravena (Mythic)`
  - JS: `💍 *Pernikahan:* ␀`
    Rust: `💍 *Pernikahan:* 0`
  - JS: `🔴 Mythic     ␀ ␀`
    Rust: `🔴 Mythic       3`
  - JS: `🔵 Rare       ␀ ␀`
    Rust: `🔵 Rare         0`
  - JS: `🟡 Legendary  ␀ ␀`
    Rust: `🟡 Legendary`
  - JS: `🟢 Common     ␀ ␀`
    Rust: `🟢 Common       9`
  - JS: `🟣 Epic       ␀ ␀`
    Rust: (tidak ada padanan)

### husbupool (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/husbupool.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/husbupool.rs`

  - JS: `Gunakan tombol untuk berpindah halaman.`
    Rust: (tidak ada padanan)
  - JS: `␀. ␀ *␀* — ␀`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: (tidak ada padanan)
  - JS: `␀husbupool next ␀`
    Rust: (tidak ada padanan)
  - JS: `␀husbupool prev ␀`
    Rust: (tidak ada padanan)
  - JS: `➡️`
    Rust: (tidak ada padanan)
  - JS: `⬅️`
    Rust: (tidak ada padanan)
  - JS: `🏷️ ␀ | 🎭 ␀`
    Rust: (tidak ada padanan)
  - JS: `📊 Total: *␀* husbu | Halaman *␀/␀*`
    Rust: (tidak ada padanan)
  - JS: `🔍 Pencarian: *"␀"*`
    Rust: (tidak ada padanan)
  - JS: `🔍 Tidak ada husbu cocok dengan *"␀"*. Coba nama/tier/franchise lain.`
    Rust: (tidak ada padanan)

### mimpi (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/mimpi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/mimpi.rs`

  - JS: `🌙 *Memasuki alam mimpi...*`
    Rust: `🌙 *Memasuki alam dream...*`

### renungan (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/renungan.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/renungan.rs`

  - JS: `renungan.json`
    Rust: `.renungan`

### terima (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/terima.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/terima.rs`

  - JS: `> @␀ tidak sedang menembakmu`
    Rust: (tidak ada padanan)
  - JS: `> Atau `␀terima @tag``
    Rust: (tidak ada padanan)
  - JS: `> Bot tidak bisa pacaran!`
    Rust: (tidak ada padanan)
  - JS: `> Reply pesan tembakan + `␀terima``
    Rust: (tidak ada padanan)
  - JS: `> Tidak bisa menerima diri sendiri!`
    Rust: (tidak ada padanan)
  - JS: `Menerima tembakan dari seseorang`
    Rust: (tidak ada padanan)
  - JS: `Semoga langgeng dan bahagia 💍`
    Rust: (tidak ada padanan)
  - JS: `␀_␀`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴛɪᴅᴀᴋ ᴍᴇɴᴇᴍʙᴀᴋ*`
    Rust: (tidak ada padanan)
  - JS: `💕 *WIDIHHHH, CIE CIE DITERIMA* @␀`
    Rust: (tidak ada padanan)

### tod (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/tod.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/tod.rs`

  - JS: `[ToD Plugin Error]`
    Rust: (tidak ada padanan)

### waifualbum (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/waifualbum.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/waifualbum.rs`

  - JS: `*10 Pull Terakhir:*`
    Rust: (tidak ada padanan)
  - JS: `*Distribusi Tier:*`
    Rust: (tidak ada padanan)
  - JS: `␀. ␀ *␀* — ␀`
    Rust: (tidak ada padanan)
  - JS: `🍀 *Luck Score:* ␀ ␀`
    Rust: `🍀 *Luck Score:* 0 😅`
  - JS: `🎯 *Pity aktif:* ␀/20 roll tanpa Epic+`
    Rust: `🎯 *Pity aktif:* 7/20 roll tanpa Epic+`
  - JS: `🎲 *Total Gacha:* ␀`
    Rust: `🎲 *Total Gacha:* 4`
  - JS: `🏆 *Pull Paling Langka:* ␀ ␀ (␀)`
    Rust: `🏆 *Pull Paling Langka:* 🔴 Ravena (Mythic)`
  - JS: `💍 *Pernikahan:* ␀`
    Rust: `💍 *Pernikahan:* 0`
  - JS: `🔴 Mythic     ␀ ␀`
    Rust: `🔴 Mythic       3`
  - JS: `🔵 Rare       ␀ ␀`
    Rust: `🔵 Rare         0`
  - JS: `🟡 Legendary  ␀ ␀`
    Rust: `🟡 Legendary`
  - JS: `🟢 Common     ␀ ␀`
    Rust: `🟢 Common       9`
  - JS: `🟣 Epic       ␀ ␀`
    Rust: (tidak ada padanan)

### waifupool (fun)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/fun/waifupool.js`
- Rust: `/workspaces/oktz-rs/plugin/src/fun/waifupool.rs`

  - JS: `Gunakan tombol untuk berpindah halaman.`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: (tidak ada padanan)
  - JS: `␀waifupool next ␀`
    Rust: (tidak ada padanan)
  - JS: `␀waifupool prev ␀`
    Rust: `.waifupool rare`
  - JS: `➡️`
    Rust: (tidak ada padanan)
  - JS: `⬅️`
    Rust: (tidak ada padanan)
  - JS: `🔍 Tidak ada waifu cocok dengan *"␀"*. Coba nama/tier/franchise lain.`
    Rust: `🔍 Tidak ada waifu cocok dengan "␀". Coba nama/tier/franchise lain.`

### dungeon (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/dungeon.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/dungeon.rs`

  - JS: `*⚔️ APA YANG INGIN KAMU LAKUKAN?*`
    Rust: (tidak ada padanan)
  - JS: `*🎁 HADIAH YANG DIDAPAT:*`
    Rust: (tidak ada padanan)
  - JS: `*💔 KERUGIAN:*`
    Rust: (tidak ada padanan)
  - JS: `*📦 BARANG JARAHAN (LOOT):*`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan `batal` jika benar-benar menyerah.`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan `lari` untuk kabur.`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan `serang` untuk melawan monster.`
    Rust: (tidak ada padanan)
  - JS: `> Balas pesan ini dengan `lari` untuk kabur (berisiko)`
    Rust: (tidak ada padanan)
  - JS: `> Balas pesan ini dengan `serang` untuk melawan`
    Rust: (tidak ada padanan)
  - JS: `> Dungeon nomor ␀ tidak ditemukan.`
    Rust: (tidak ada padanan)
  - JS: `> Kamu butuh minimal *Lv ␀*.`
    Rust: (tidak ada padanan)
  - JS: `> Level kamu (*Lv ␀*) belum cukup untuk memasuki *␀*.`
    Rust: (tidak ada padanan)
  - JS: `> Level kamu saat ini adalah *␀*. Kamu butuh minimal level *1* untuk memasuki dungeon paling mudah.`
    Rust: `> Level kamu saat ini adalah *0*. Kamu butuh minimal level *1* untuk memasuki dungeon paling mudah.`
  - JS: `> Sesi dungeon kamu sudah hangus karena tidak aktif selama 5 menit.`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `> ⚡ Stamina berkurang *␀*`
    Rust: (tidak ada padanan)
  - JS: `> ✨ EXP: *+␀*`
    Rust: (tidak ada padanan)
  - JS: `> ❤️ Darah berkurang: *-40 HP*`
    Rust: (tidak ada padanan)
  - JS: `> ❤️ Darah berkurang: *-␀ HP*`
    Rust: (tidak ada padanan)
  - JS: `> 💡 *Tips:* Gunakan perintah `.rest` atau batalkan dulu (ketik `batal`).`
    Rust: (tidak ada padanan)
  - JS: `> 💡 *Tips:* Naikan levelmu, makan potion, atau perkuat senjata!`
    Rust: (tidak ada padanan)
  - JS: `> 💰 Koin: *+␀*`
    Rust: (tidak ada padanan)
  - JS: `> 💸 Uang jatuh: *-␀ Koin*`
    Rust: (tidak ada padanan)
  - JS: `Bandit Hutan`
    Rust: (tidak ada padanan)
  - JS: `Dengan serangan mematikan, kamu berhasil menebas *␀*!`
    Rust: (tidak ada padanan)
  - JS: `Elemental Api`
    Rust: (tidak ada padanan)
  - JS: `Gargoyle Batu`
    Rust: (tidak ada padanan)
  - JS: `Goblin Liar`
    Rust: (tidak ada padanan)
  - JS: `Golem Angin`
    Rust: (tidak ada padanan)
  - JS: `Golem Es`
    Rust: (tidak ada padanan)
  - JS: `Golem Magma`
    Rust: (tidak ada padanan)
  - JS: `Griffin Liar`
    Rust: (tidak ada padanan)
  - JS: `Hantu Penasaran`
    Rust: (tidak ada padanan)
  - JS: `Harpy Petir`
    Rust: (tidak ada padanan)
  - JS: `Hiu Hantu`
    Rust: (tidak ada padanan)
  - JS: `Iblis Merah`
    Rust: (tidak ada padanan)
  - JS: `Jin Jahat`
    Rust: (tidak ada padanan)
  - JS: `Kakimu tersandung bebatuan! *␀* mengejarmu dan mendaratkan cakarnya di tubuhmu!`
    Rust: (tidak ada padanan)
  - JS: `Kalajengking Raksasa`
    Rust: (tidak ada padanan)
  - JS: `Kamu berbalik dan berlari sekuat tenaga. *␀* kehilangan jejakmu!`
    Rust: (tidak ada padanan)
  - JS: `Kamu berhasil merangkak keluar dengan tubuh penuh luka.`
    Rust: (tidak ada padanan)
  - JS: `Kamu butuh setidaknya *␀ stamina* untuk masuk.`
    Rust: (tidak ada padanan)
  - JS: `Kamu melangkah perlahan ke dalam *␀*...`
    Rust: (tidak ada padanan)
  - JS: `Kamu selamat tanpa cedera, tapi petualangan ini sia-sia.`
    Rust: (tidak ada padanan)
  - JS: `Kekuatanmu belum sebanding! *␀* memukul mundur dirimu dengan telak.`
    Rust: (tidak ada padanan)
  - JS: `Kodok Mutan`
    Rust: (tidak ada padanan)
  - JS: `Kraken Bayi`
    Rust: (tidak ada padanan)
  - JS: `Laba-laba Racun`
    Rust: (tidak ada padanan)
  - JS: `Leviathan Merah`
    Rust: (tidak ada padanan)
  - JS: `Malaikat Kematian`
    Rust: (tidak ada padanan)
  - JS: `Mummy Bangkit`
    Rust: (tidak ada padanan)
  - JS: `Naga Kecil`
    Rust: (tidak ada padanan)
  - JS: `Pohon Berjalan`
    Rust: (tidak ada padanan)
  - JS: `Prajurit Tengkorak`
    Rust: (tidak ada padanan)
  - JS: `Raja Iblis`
    Rust: (tidak ada padanan)
  - JS: `Raksasa Frost`
    Rust: (tidak ada padanan)
  - JS: `Serigala Malam`
    Rust: (tidak ada padanan)
  - JS: `Serigala Salju`
    Rust: (tidak ada padanan)
  - JS: `Shadow Fiend`
    Rust: (tidak ada padanan)
  - JS: `Siren Pemikat`
    Rust: (tidak ada padanan)
  - JS: `Sisa stamina kamu saat ini hanya *␀*.`
    Rust: (tidak ada padanan)
  - JS: `Slime Raksasa`
    Rust: (tidak ada padanan)
  - JS: `Succubus Mematikan`
    Rust: (tidak ada padanan)
  - JS: `Tiba-tiba, seekor *👹 ␀* muncul dari kegelapan dan menghadang jalanmu!`
    Rust: (tidak ada padanan)
  - JS: `Valkyrie Jatuh`
    Rust: (tidak ada padanan)
  - JS: `Viper Rawa`
    Rust: (tidak ada padanan)
  - JS: `Void Walker`
    Rust: (tidak ada padanan)
  - JS: `Worm Gurun`
    Rust: (tidak ada padanan)
  - JS: `Yeti Ganas`
    Rust: (tidak ada padanan)
  - JS: `Zombie Kelaparan`
    Rust: (tidak ada padanan)
  - JS: `⏰ *SESI DUNGEON KEDALUWARSA*`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀ (x␀)`
    Rust: (tidak ada padanan)
  - JS: `⚡ *STAMINA TIDAK CUKUP*`
    Rust: (tidak ada padanan)
  - JS: `⛏️`
    Rust: (tidak ada padanan)
  - JS: `❌ *PILIHAN TIDAK VALID*`
    Rust: (tidak ada padanan)
  - JS: `❓ *PILIHAN TIDAK DIKENAL*`
    Rust: (tidak ada padanan)
  - JS: `🎉 *KEMENANGAN GEMILANG!*`
    Rust: (tidak ada padanan)
  - JS: `🏃‍♂️ *BERHASIL KABUR!*`
    Rust: (tidak ada padanan)
  - JS: `💀 *KEKALAHAN TRAGIS!*`
    Rust: (tidak ada padanan)
  - JS: `💥 *GAGAL KABUR!*`
    Rust: (tidak ada padanan)
  - JS: `🔒 *DUNGEON TERKUNCI*`
    Rust: (tidak ada padanan)
  - JS: `🚪 *MEMASUKI DUNGEON*`
    Rust: (tidak ada padanan)
  - JS: `🚪 Kamu berhasil keluar dari Lobi Dungeon dengan selamat.`
    Rust: (tidak ada padanan)

### family100 (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/family100.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/family100.rs`

  - JS: `@␀ dapet *+␀ EXP* & *+␀ Koin* nih! 💸✨`
    Rust: (tidak ada padanan)
  - JS: `Ayo gas sisa *␀* jawaban lagi kak! 🔥⏱️`
    Rust: (tidak ada padanan)
  - JS: `Benerrr banget! ✅🎉`
    Rust: (tidak ada padanan)
  - JS: `Gapapa, next time pasti bisa full senyum! 💖✨`
    Rust: (tidak ada padanan)
  - JS: `Hayo lho, jawaban *␀* udah ada yang jawab tadi kak! Cari yang lain dong 😂✨`
    Rust: (tidak ada padanan)
  - JS: `Ini nih jawaban yang kelewatan:`
    Rust: (tidak ada padanan)
  - JS: `Kalian berhasil nebak *␀* dari *␀* jawaban! ✨`
    Rust: (tidak ada padanan)
  - JS: `Makasih udah main ya, ditunggu sesi berikutnya! 💖🎉`
    Rust: (tidak ada padanan)
  - JS: `Nih aku kasih tau jawaban sisanya:`
    Rust: (tidak ada padanan)
  - JS: `Padahal udah nebak *␀* dari *␀* lho! 👏`
    Rust: (tidak ada padanan)
  - JS: `Tettt! ❌ Salah kak! Coba dipikir-pikir lagi deh 😂🧠`
    Rust: (tidak ada padanan)
  - JS: `WOAAHH KEREN BANGET! Semua jawaban ketebak dong! 🎉🔥✨`
    Rust: (tidak ada padanan)
  - JS: `Walahh pada nyerah nih ceritanya? 🥺🏳️`
    Rust: (tidak ada padanan)
  - JS: `Yah sayang banget waktu udah habis kak! 😭😭⏱️`
    Rust: (tidak ada padanan)
  - JS: `• ␀`
    Rust: (tidak ada padanan)
  - JS: `␀. ✅ ␀ - @␀`
    Rust: (tidak ada padanan)
  - JS: `⚠️`
    Rust: (tidak ada padanan)
  - JS: `🎊 Selamat buat kalian semua yang udah ikutan mikir! Gacor banget otaknya! 🧠💯`
    Rust: (tidak ada padanan)

### kyubigame (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/kyubigame.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/kyubigame.rs`

  - JS: `*⚔️ APA YANG INGIN KAMU LAKUKAN?*`
    Rust: (tidak ada padanan)
  - JS: `*🎁 HADIAH PENYELESAIAN MISI:*`
    Rust: (tidak ada padanan)
  - JS: `*💔 KERUGIAN:*`
    Rust: (tidak ada padanan)
  - JS: `*📦 BARANG JARAHAN SHINOBI:*`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan `batal` jika ingin membatalkan misi.`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan `lari` untuk kabur.`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan `serang` untuk melawan musuh.`
    Rust: (tidak ada padanan)
  - JS: `> Balas pesan ini dengan `lari` untuk mundur (berisiko)`
    Rust: (tidak ada padanan)
  - JS: `> Balas pesan ini dengan `serang` untuk melawan`
    Rust: (tidak ada padanan)
  - JS: `> Kamu butuh minimal *Lv ␀*.`
    Rust: (tidak ada padanan)
  - JS: `> Level kamu (*Lv ␀*) belum cukup untuk memasuki *␀*.`
    Rust: (tidak ada padanan)
  - JS: `> Level kamu saat ini adalah *␀*. Kamu butuh minimal level *1* untuk memulai petualangan shinobi.`
    Rust: `Kamu butuh minimal level *1* untuk memulai petualangan shinobi.`
  - JS: `> Lokasi nomor ␀ tidak ada di peta shinobi.`
    Rust: (tidak ada padanan)
  - JS: `> Sesi misi shinobi kamu sudah hangus karena tidak aktif selama 5 menit.`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `> ⚡ Stamina berkurang *␀*`
    Rust: (tidak ada padanan)
  - JS: `> ✨ EXP: *+␀*`
    Rust: (tidak ada padanan)
  - JS: `> ❤️ Darah berkurang: *-40 HP*`
    Rust: (tidak ada padanan)
  - JS: `> ❤️ Darah berkurang: *-␀ HP*`
    Rust: (tidak ada padanan)
  - JS: `> 💡 *Tips:* Gunakan perintah `.rest` atau batalkan dulu (ketik `batal`).`
    Rust: (tidak ada padanan)
  - JS: `> 💡 *Tips:* Naikan levelmu, makan ramen, atau perkuat jutsumu!`
    Rust: (tidak ada padanan)
  - JS: `> 💰 Ryo (Koin): *+␀*`
    Rust: (tidak ada padanan)
  - JS: `> 💸 Uang jatuh: *-␀ Ryo*`
    Rust: (tidak ada padanan)
  - JS: `Anjing Hutan`
    Rust: (tidak ada padanan)
  - JS: `Bandit Liar`
    Rust: (tidak ada padanan)
  - JS: `Boneka Sasori`
    Rust: (tidak ada padanan)
  - JS: `Burung Hantu Petir`
    Rust: (tidak ada padanan)
  - JS: `Chakra Ekor Sembilan`
    Rust: (tidak ada padanan)
  - JS: `Dengan jutsu mematikan, kamu berhasil mengalahkan *␀*!`
    Rust: (tidak ada padanan)
  - JS: `Edo Tensei Kage`
    Rust: (tidak ada padanan)
  - JS: `Genin Pemula`
    Rust: (tidak ada padanan)
  - JS: `Harimau Raksasa`
    Rust: (tidak ada padanan)
  - JS: `Kakimu tersandung jebakan ninja! *␀* mengejarmu dan mendaratkan serangannya di tubuhmu!`
    Rust: (tidak ada padanan)
  - JS: `Kamu berhasil menggunakan jutsu substitusi dan merangkak keluar dengan tubuh penuh luka.`
    Rust: (tidak ada padanan)
  - JS: `Kamu butuh setidaknya *␀ stamina* untuk masuk.`
    Rust: (tidak ada padanan)
  - JS: `Kamu melemparkan bom asap dan berlari sekuat tenaga. *␀* kehilangan jejakmu!`
    Rust: (tidak ada padanan)
  - JS: `Kamu melompat perlahan menyusuri *␀*...`
    Rust: (tidak ada padanan)
  - JS: `Kamu selamat tanpa cedera, tapi petualangan ini sia-sia.`
    Rust: (tidak ada padanan)
  - JS: `Kekuatanmu belum sebanding! *␀* memukul mundur dirimu dengan telak.`
    Rust: (tidak ada padanan)
  - JS: `Kelelawar Beracun`
    Rust: (tidak ada padanan)
  - JS: `Klon Zetsu Putih`
    Rust: (tidak ada padanan)
  - JS: `Kurama Kegelapan`
    Rust: (tidak ada padanan)
  - JS: `Kyubi Liar`
    Rust: (tidak ada padanan)
  - JS: `Lipan Racun`
    Rust: (tidak ada padanan)
  - JS: `Mizukage Klon`
    Rust: (tidak ada padanan)
  - JS: `Ninja Kumo`
    Rust: (tidak ada padanan)
  - JS: `Ninja Otogakure`
    Rust: (tidak ada padanan)
  - JS: `Ninja Pelarian`
    Rust: (tidak ada padanan)
  - JS: `Ninja Pembunuh`
    Rust: (tidak ada padanan)
  - JS: `Ninja Pengintai`
    Rust: (tidak ada padanan)
  - JS: `Pasukan Klon`
    Rust: (tidak ada padanan)
  - JS: `Patung Golem`
    Rust: (tidak ada padanan)
  - JS: `Roh Bijuu`
    Rust: (tidak ada padanan)
  - JS: `Samurai Besi`
    Rust: (tidak ada padanan)
  - JS: `Serigala Listrik`
    Rust: (tidak ada padanan)
  - JS: `Shinobi Undead`
    Rust: (tidak ada padanan)
  - JS: `Sisa stamina kamu saat ini hanya *␀*.`
    Rust: (tidak ada padanan)
  - JS: `Tiba-tiba, seorang *👹 ␀* melesat dari kegelapan dan menghadang jalanmu!`
    Rust: (tidak ada padanan)
  - JS: `Uchiha Hantu`
    Rust: (tidak ada padanan)
  - JS: `Ular Orochimaru`
    Rust: (tidak ada padanan)
  - JS: `Zetsu Raksasa`
    Rust: (tidak ada padanan)
  - JS: `⏰ *MISI KEDALUWARSA*`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀ (x␀)`
    Rust: (tidak ada padanan)
  - JS: `⚔️`
    Rust: (tidak ada padanan)
  - JS: `⚡ *CHAKRA/STAMINA TIDAK CUKUP*`
    Rust: (tidak ada padanan)
  - JS: `⛩️`
    Rust: (tidak ada padanan)
  - JS: `⛩️ *MEMASUKI AREA MISI*`
    Rust: (tidak ada padanan)
  - JS: `❌ *MISI TIDAK VALID*`
    Rust: (tidak ada padanan)
  - JS: `❓ *PERINTAH TIDAK DIKENAL*`
    Rust: (tidak ada padanan)
  - JS: `🎉 *MISI BERHASIL!*`
    Rust: (tidak ada padanan)
  - JS: `🏃‍♂️ *BERHASIL KABUR!*`
    Rust: (tidak ada padanan)
  - JS: `💀 *MISI GAGAL!*`
    Rust: (tidak ada padanan)
  - JS: `💥 *GAGAL KABUR!*`
    Rust: (tidak ada padanan)
  - JS: `🔒 *MISI TERKUNCI*`
    Rust: (tidak ada padanan)
  - JS: `🗡️`
    Rust: (tidak ada padanan)
  - JS: `🚪 Kamu berhasil membatalkan misi dan kembali ke desa dengan selamat.`
    Rust: (tidak ada padanan)

### mct (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/minecraft.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/minecraft.rs`

  - JS: `*_⛏️ Menambang di ␀..._*`
    Rust: (tidak ada padanan)
  - JS: `*␀*`
    Rust: (tidak ada padanan)
  - JS: `*␀* _(Lv.␀/␀)_`
    Rust: (tidak ada padanan)
  - JS: `*␀* _(␀)_ ke *␀*`
    Rust: (tidak ada padanan)
  - JS: `*␀* _seharga 🪙 ␀ tokens_`
    Rust: (tidak ada padanan)
  - JS: `*␀* _✅ sekarang aktif_`
    Rust: (tidak ada padanan)
  - JS: `*␀* _🪙 ␀ tokens_`
    Rust: (tidak ada padanan)
  - JS: `*␀*␀`
    Rust: (tidak ada padanan)
  - JS: `*⚔️ ATK:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*⚔️ COMBAT*`
    Rust: (tidak ada padanan)
  - JS: `*⚔️ Combat*`
    Rust: (tidak ada padanan)
  - JS: `*⚔️ Combat:* ␀W/␀L`
    Rust: (tidak ada padanan)
  - JS: `*⛏️ HASIL MENAMBANG!*`
    Rust: (tidak ada padanan)
  - JS: `*⛏️ MINING*`
    Rust: (tidak ada padanan)
  - JS: `*✨ ENCHANT DIPASANG!*`
    Rust: (tidak ada padanan)
  - JS: `*❤️ HEAL!*`
    Rust: (tidak ada padanan)
  - JS: `*❤️ HP:* ␀/␀`
    Rust: (tidak ada padanan)
  - JS: `*⬆️ LEVEL UP! Level ␀*`
    Rust: (tidak ada padanan)
  - JS: `*⬆️ Level:* ␀ _(␀/␀ EXP)_`
    Rust: (tidak ada padanan)
  - JS: `*⬆️ PICKAXE UPGRADED!*`
    Rust: (tidak ada padanan)
  - JS: `*⬆️ ␀ LEVEL UP!*`
    Rust: (tidak ada padanan)
  - JS: `*⭐ EXP:* +␀`
    Rust: (tidak ada padanan)
  - JS: `*⭐ PICKAXE EXP UP!*`
    Rust: (tidak ada padanan)
  - JS: `*🆕 Ore Baru:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🌈 SSR PULL!*`
    Rust: (tidak ada padanan)
  - JS: `*🎁 DAILY REWARD!*`
    Rust: (tidak ada padanan)
  - JS: `*🎁 Drops:*`
    Rust: (tidak ada padanan)
  - JS: `*🎒 INVENTORY*`
    Rust: (tidak ada padanan)
  - JS: `*🎒 KOLEKSI PICKAXE*`
    Rust: (tidak ada padanan)
  - JS: `*🎟️ Tickets:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🎰 GACHA RESULT!*`
    Rust: (tidak ada padanan)
  - JS: `*🎰 JACKPOT SYSTEM*`
    Rust: (tidak ada padanan)
  - JS: `*🎰 JACKPOT*`
    Rust: (tidak ada padanan)
  - JS: `*🎰 ␀!*`
    Rust: (tidak ada padanan)
  - JS: `*🏆 KAMU MENANG!*`
    Rust: (tidak ada padanan)
  - JS: `*🏆 LEADERBOARD MINECRAFT*`
    Rust: (tidak ada padanan)
  - JS: `*🏦 Saldo: ␀*`
    Rust: (tidak ada padanan)
  - JS: `*🏦 Saldo:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🏷️ Title:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*👑 PRESTIGE SYSTEM*`
    Rust: (tidak ada padanan)
  - JS: `*👑 PRESTIGE UP!*`
    Rust: (tidak ada padanan)
  - JS: `*👑 PRESTIGE*`
    Rust: (tidak ada padanan)
  - JS: `*👑 Prestige:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*👤 PROFIL PENAMBANG*`
    Rust: (tidak ada padanan)
  - JS: `*👤 PROFILE*`
    Rust: (tidak ada padanan)
  - JS: `*👹 DAFTAR MOB*`
    Rust: (tidak ada padanan)
  - JS: `*💀 KAMU KALAH!*`
    Rust: (tidak ada padanan)
  - JS: `*💥 JACKPOT BESAR!*`
    Rust: (tidak ada padanan)
  - JS: `*💰 Total Value: ␀*`
    Rust: (tidak ada padanan)
  - JS: `*💰 Total: ␀*`
    Rust: (tidak ada padanan)
  - JS: `*💰 Uang:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*💸 ORE TERJUAL!*`
    Rust: (tidak ada padanan)
  - JS: `*📈 UPGRADE SHOP*`
    Rust: (tidak ada padanan)
  - JS: `*📈 Upgrades*`
    Rust: (tidak ada padanan)
  - JS: `*📈 Upgrades:*`
    Rust: (tidak ada padanan)
  - JS: `*📊 STATS DETAIL*`
    Rust: (tidak ada padanan)
  - JS: `*📚 ORE BOOK* _(␀ ore)_`
    Rust: (tidak ada padanan)
  - JS: `*📚 OreBook:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*📜 DAFTAR ENCHANTMENT*`
    Rust: (tidak ada padanan)
  - JS: `*📦 HASIL DIAMBIL!*`
    Rust: (tidak ada padanan)
  - JS: `*🔥 CRAFTING & SMELTING*`
    Rust: (tidak ada padanan)
  - JS: `*🔥 Pity Activated!*`
    Rust: (tidak ada padanan)
  - JS: `*🔥 SMELTING SELESAI!*`
    Rust: (tidak ada padanan)
  - JS: `*🔥 Streak:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🔥 Streak:* ␀ hari`
    Rust: (tidak ada padanan)
  - JS: `*🗺️ BIOME & PICKAXE*`
    Rust: (tidak ada padanan)
  - JS: `*🗺️ Biome:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🗺️ DAFTAR BIOME*`
    Rust: (tidak ada padanan)
  - JS: `*🚪 PINDAH BIOME!*`
    Rust: (tidak ada padanan)
  - JS: `*🛍️ ITEM DIBELI!*`
    Rust: (tidak ada padanan)
  - JS: `*🛍️ PICKAXE DIBELI!*`
    Rust: (tidak ada padanan)
  - JS: `*🛒 TOKO PICKAXE*`
    Rust: (tidak ada padanan)
  - JS: `*🛠️ CRAFTING BERHASIL!*`
    Rust: (tidak ada padanan)
  - JS: `*🛠️ DAFTAR CRAFTING*`
    Rust: (tidak ada padanan)
  - JS: `*🧱 Blocks:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🧱 MINECRAFT DISABLED*`
    Rust: `🧱 *MINECRAFT GAME*`
  - JS: `*🧱 MINECRAFT ENABLED*`
    Rust: `🧱 *MINECRAFT GAME*`
  - JS: `*🧱 MINECRAFT GAME*`
    Rust: `🧱 *MINECRAFT GAME*`
  - JS: `*🧱 Minecraft:* ␀`
    Rust: (tidak ada padanan)
  - JS: `*🪓 PICKAXE DIPASANG!*`
    Rust: (tidak ada padanan)
  - JS: `*🪓 Pick: ␀*`
    Rust: (tidak ada padanan)
  - JS: `*🪓 Pick:* ␀ _(Lv.␀)_`
    Rust: (tidak ada padanan)
  - JS: `*🪙 TOKEN STORE*`
    Rust: (tidak ada padanan)
  - JS: `*🪙 Tokens:* ␀`
    Rust: (tidak ada padanan)
  - JS: `2x EXP + 1200 tokens`
    Rust: (tidak ada padanan)
  - JS: `Cosmic Pickaxe + 600 tokens`
    Rust: (tidak ada padanan)
  - JS: `EXP Boost`
    Rust: (tidak ada padanan)
  - JS: `Gacha Tickets`
    Rust: (tidak ada padanan)
  - JS: `God of Mining`
    Rust: (tidak ada padanan)
  - JS: `Luck +20% + 180 tokens`
    Rust: (tidak ada padanan)
  - JS: `Master Miner`
    Rust: (tidak ada padanan)
  - JS: `OWNER REWARD`
    Rust: (tidak ada padanan)
  - JS: `Omega Pickaxe + 6000 tokens`
    Rust: (tidak ada padanan)
  - JS: `Partner 30 Hari`
    Rust: (tidak ada padanan)
  - JS: `Partner 7 Hari`
    Rust: (tidak ada padanan)
  - JS: `Penambang Baru`
    Rust: (tidak ada padanan)
  - JS: `Premium 30 Hari`
    Rust: (tidak ada padanan)
  - JS: `Premium 7 Hari`
    Rust: (tidak ada padanan)
  - JS: `Prestige Pickaxe + 60 tokens`
    Rust: (tidak ada padanan)
  - JS: `Prestige Tokens`
    Rust: (tidak ada padanan)
  - JS: `P␀`
    Rust: (tidak ada padanan)
  - JS: `UNLIMITED Energi`
    Rust: (tidak ada padanan)
  - JS: `UNLIMITED Limit`
    Rust: (tidak ada padanan)
  - JS: `_Hanya admin/owner_`
    Rust: (tidak ada padanan)
  - JS: `_Toggle hanya di grup_`
    Rust: (tidak ada padanan)
  - JS: `_⏳ Tunggu *␀* detik_`
    Rust: (tidak ada padanan)
  - JS: `_␀ x␀_`
    Rust: (tidak ada padanan)
  - JS: `_␀: ␀-␀`} (␀%)_`
    Rust: (tidak ada padanan)
  - JS: `_␀_`
    Rust: (tidak ada padanan)
  - JS: `_⚔️ ATK: ␀ | ❤️ HP: ␀/␀_`
    Rust: (tidak ada padanan)
  - JS: `_⚔️ Combat: Lv.␀ (+␀%)_`
    Rust: (tidak ada padanan)
  - JS: `_⚔️ Combat: Lv.␀_`
    Rust: (tidak ada padanan)
  - JS: `_⚡ Speed: Lv.␀ (+␀%)_`
    Rust: (tidak ada padanan)
  - JS: `_⚡ Speed: Lv.␀_`
    Rust: (tidak ada padanan)
  - JS: `_✅ Sudah max!_`
    Rust: (tidak ada padanan)
  - JS: `_✅ Sudah punya pickaxe ini!_`
    Rust: (tidak ada padanan)
  - JS: `_✅ Sudah punya ␀!_`
    Rust: (tidak ada padanan)
  - JS: `_✨ Enchant saat ini: *␀* (␀)_`
    Rust: (tidak ada padanan)
  - JS: `_✨ Enchant tidak ada!_ `.mct enchants``
    Rust: (tidak ada padanan)
  - JS: `_✨ Enchant: ␀ (␀)_`
    Rust: (tidak ada padanan)
  - JS: `_✨ Enchant: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_✨ Tentukan enchant!_ `.mct enchants``
    Rust: (tidak ada padanan)
  - JS: `_❌ Gagal! Coba lagi._`
    Rust: (tidak ada padanan)
  - JS: `_❤️ HP tersisa: ␀/␀_`
    Rust: (tidak ada padanan)
  - JS: `_❤️ HP: ␀ | ⚔️ ATK: ␀ | ⬆️ Lv.␀+_`
    Rust: (tidak ada padanan)
  - JS: `_❤️ HP: ␀/␀ | ⚔️ ATK: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_⬆️ Level ␀_`
    Rust: (tidak ada padanan)
  - JS: `_⬆️ Lv.␀/␀ | ⭐ EXP ␀/␀_`
    Rust: (tidak ada padanan)
  - JS: `_⬆️ Lv.␀/␀ | 🍀 Luck ␀% | ⚡ Speed ␀%_`
    Rust: (tidak ada padanan)
  - JS: `_⬆️ Pickaxe sudah max level!_`
    Rust: (tidak ada padanan)
  - JS: `_🆓 Free_`
    Rust: (tidak ada padanan)
  - JS: `_🍀 Luck +␀% | ⚡ Speed +␀% | 📦 Combo: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_🍀 Luck: Lv.␀ (+␀%)_`
    Rust: (tidak ada padanan)
  - JS: `_🍀 Luck: Lv.␀_`
    Rust: (tidak ada padanan)
  - JS: `_🍀 Luck: ␀% | ⚡ Speed: ␀%_`
    Rust: (tidak ada padanan)
  - JS: `_🎁 Bisa dapat _Premium_, _Partner_, _Energi_, _Limit_, bahkan _UNLIMITED_!_`
    Rust: (tidak ada padanan)
  - JS: `_🎁 Daily sudah diambil! Tunggu *␀* jam._`
    Rust: (tidak ada padanan)
  - JS: `_🎁 Jackpot bisa kasih _Premium_, _Partner_, _Energi_, _Limit_, bahkan _UNLIMITED_!_`
    Rust: (tidak ada padanan)
  - JS: `_🎁 Rewards:_`
    Rust: (tidak ada padanan)
  - JS: `_🎒 Inventory kosong!_`
    Rust: (tidak ada padanan)
  - JS: `_🎒 Inventory kosong!_ `.mct mine` _dulu._`
    Rust: (tidak ada padanan)
  - JS: `_🎟️ Tiket habis! Punya ␀_`
    Rust: (tidak ada padanan)
  - JS: `_🎯 Rate: ␀%_`
    Rust: (tidak ada padanan)
  - JS: `_🎰 Tier tidak ada!_ `.mct jackpot``
    Rust: (tidak ada padanan)
  - JS: `_🏆 Wins: ␀ | 💀 Losses: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_🏦 Saldo: ␀ | 🎟️ Tiket: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_🏦 Saldo: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_👑 P␀ | 🧱 ␀ blocks | 💰 ␀_`
    Rust: (tidak ada padanan)
  - JS: `_👑 Sudah max prestige!_`
    Rust: (tidak ada padanan)
  - JS: `_👤 Belum ada pemain!_`
    Rust: (tidak ada padanan)
  - JS: `_👹 Mob tidak ada!_ `.mct fight``
    Rust: (tidak ada padanan)
  - JS: `_👹 Tidak ada mob tersedia! Naikkan level dulu._`
    Rust: (tidak ada padanan)
  - JS: `_💎 Fortune: +␀% | 💰 Sell: +␀%_`
    Rust: (tidak ada padanan)
  - JS: `_💎 Fortune: Lv.␀ (+␀%)_`
    Rust: (tidak ada padanan)
  - JS: `_💎 Fortune: Lv.␀_`
    Rust: (tidak ada padanan)
  - JS: `_💎 Sistem jackpot dengan hadiah super besar!_`
    Rust: (tidak ada padanan)
  - JS: `_💰 Biaya: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_💰 Value: ␀ | ⬆️ Lv.␀+_`
    Rust: (tidak ada padanan)
  - JS: `_💰 Value: ␀_`
    Rust: (tidak ada padanan)
  - JS: `_💰 ␀ | 📦 x␀_`
    Rust: (tidak ada padanan)
  - JS: `_💰 ␀ | 🧱 ␀ blocks_`
    Rust: (tidak ada padanan)
  - JS: `_💰 ␀_`
    Rust: (tidak ada padanan)
  - JS: `_💸 Uang -90%, 🧱 blocks count reset_`
    Rust: (tidak ada padanan)
  - JS: `_💸 Uang kurang! Butuh ␀_`
    Rust: (tidak ada padanan)
  - JS: `_📍 Sudah di ␀!_`
    Rust: (tidak ada padanan)
  - JS: `_📚 Ore Book kosong!_ `.mct mine` _dulu_`
    Rust: (tidak ada padanan)
  - JS: `_📦 Masih ada hasil tambang!_ `.mct collect` _dulu._`
    Rust: (tidak ada padanan)
  - JS: `_📭 Tidak ada hasil._ `.mct mine` _dulu!_`
    Rust: (tidak ada padanan)
  - JS: `_📭 Tidak punya pickaxe ini!_ `.mct picks``
    Rust: (tidak ada padanan)
  - JS: `_📭 Tidak punya pickaxe!_`
    Rust: (tidak ada padanan)
  - JS: `_🔥 Combat Bonus: +␀%_`
    Rust: (tidak ada padanan)
  - JS: `_🔥 Pity: ␀/␀_`
    Rust: (tidak ada padanan)
  - JS: `_🔥 Streak: ␀x_`
    Rust: (tidak ada padanan)
  - JS: `_🔥 Tidak ada ore yang bisa di-smelt!_`
    Rust: (tidak ada padanan)
  - JS: `_🗺️ Biome tidak ada!_ `.mct travel``
    Rust: (tidak ada padanan)
  - JS: `_🛒 Item tidak ada!_ `.mct tokens``
    Rust: (tidak ada padanan)
  - JS: `_🧪 Bahan:_`
    Rust: (tidak ada padanan)
  - JS: `_🧱 Blocks kurang! Butuh ␀_`
    Rust: (tidak ada padanan)
  - JS: `_🪓 Ketik_ `.mct equip ␀` _untuk memasang_`
    Rust: (tidak ada padanan)
  - JS: `_🪓 Pick: ␀ | 🍀 Luck: ␀%_`
    Rust: (tidak ada padanan)
  - JS: `_🪓 Pickaxe aktif tidak ada!_`
    Rust: (tidak ada padanan)
  - JS: `_🪓 Pickaxe aktif tidak ada!_ `.mct picks``
    Rust: (tidak ada padanan)
  - JS: `_🪓 Pickaxe tidak ada!_ `.mct shop``
    Rust: (tidak ada padanan)
  - JS: `_🪓 Tentukan pickaxe!_ `.mct picks``
    Rust: (tidak ada padanan)
  - JS: `_🪓 Tentukan pickaxe!_ `.mct shop``
    Rust: (tidak ada padanan)
  - JS: `_🪙 Pickaxe ini dari Token/Prestige!_`
    Rust: (tidak ada padanan)
  - JS: `_🪙 Tokens kurang! Butuh ␀_`
    Rust: (tidak ada padanan)
  - JS: ``.mct buy <pick>``
    Rust: (tidak ada padanan)
  - JS: ``.mct buy <pick>` _💰 Beli pickaxe_`
    Rust: (tidak ada padanan)
  - JS: ``.mct collect` _📦 Ambil hasil_`
    Rust: (tidak ada padanan)
  - JS: ``.mct collect` untuk 📦 mengambil!`
    Rust: (tidak ada padanan)
  - JS: ``.mct craft <id>``
    Rust: (tidak ada padanan)
  - JS: ``.mct craft <id>` _🧪 Craft item_`
    Rust: (tidak ada padanan)
  - JS: ``.mct craft` _🛠️ Daftar resep_`
    Rust: (tidak ada padanan)
  - JS: ``.mct daily` _🎁 Daily reward_`
    Rust: (tidak ada padanan)
  - JS: ``.mct enchant <key>``
    Rust: (tidak ada padanan)
  - JS: ``.mct enchant <key>` _✨ Enchant pickaxe_`
    Rust: (tidak ada padanan)
  - JS: ``.mct enchant <key>` untuk ganti`
    Rust: (tidak ada padanan)
  - JS: ``.mct enchants` _📜 Daftar enchant_`
    Rust: (tidak ada padanan)
  - JS: ``.mct equip <pick>``
    Rust: (tidak ada padanan)
  - JS: ``.mct equip <pick>` _🪓 Pasang pickaxe_`
    Rust: (tidak ada padanan)
  - JS: ``.mct fight <mob>` _⚔️ Lawan mob_`
    Rust: (tidak ada padanan)
  - JS: ``.mct fight <mob>` untuk ⚔️ menyerang!`
    Rust: (tidak ada padanan)
  - JS: ``.mct fight` _👹 Daftar mob_`
    Rust: (tidak ada padanan)
  - JS: ``.mct gacha ticket` _🎟️ Gacha pakai tiket_`
    Rust: (tidak ada padanan)
  - JS: ``.mct gacha` _🎰 Gacha_`
    Rust: (tidak ada padanan)
  - JS: ``.mct heal` _❤️ Sembuhkan HP_`
    Rust: (tidak ada padanan)
  - JS: ``.mct heal` untuk ❤️ sembuh`
    Rust: (tidak ada padanan)
  - JS: ``.mct inv` _🎒 Inventory_`
    Rust: (tidak ada padanan)
  - JS: ``.mct jackpot <iron/gold/diamond/netherite/dragon>``
    Rust: (tidak ada padanan)
  - JS: ``.mct jackpot <tier>` _💎 Main jackpot_`
    Rust: (tidak ada padanan)
  - JS: ``.mct jackpot` _🎰 Daftar jackpot_`
    Rust: (tidak ada padanan)
  - JS: ``.mct me` _🪪 Profil kamu_`
    Rust: (tidak ada padanan)
  - JS: ``.mct mine` _⛏️ Mulai menambang_`
    Rust: (tidak ada padanan)
  - JS: ``.mct orebook` _📚 Koleksi ore_`
    Rust: (tidak ada padanan)
  - JS: ``.mct picks` _🎒 Koleksi pickaxe_`
    Rust: (tidak ada padanan)
  - JS: ``.mct pickup` _⬆️ Upgrade pickaxe_`
    Rust: (tidak ada padanan)
  - JS: ``.mct prestige confirm` _(hati-hati!)_`
    Rust: (tidak ada padanan)
  - JS: ``.mct prestige` _👑 Info prestige_`
    Rust: (tidak ada padanan)
  - JS: ``.mct sell` _💸 Jual ore_`
    Rust: (tidak ada padanan)
  - JS: ``.mct shop` _🛒 Toko pickaxe_`
    Rust: (tidak ada padanan)
  - JS: ``.mct smelt` _🔥 Smelt semua ore_`
    Rust: (tidak ada padanan)
  - JS: ``.mct stats` _📊 Stats detail_`
    Rust: (tidak ada padanan)
  - JS: ``.mct tokens <id>``
    Rust: (tidak ada padanan)
  - JS: ``.mct tokens` _🪙 Token store_`
    Rust: (tidak ada padanan)
  - JS: ``.mct top` _🏆 Leaderboard_`
    Rust: (tidak ada padanan)
  - JS: ``.mct travel <biome>``
    Rust: (tidak ada padanan)
  - JS: ``.mct travel <biome>` _🚪 Pindah biome_`
    Rust: (tidak ada padanan)
  - JS: ``.mct travel` _🗺️ Daftar biome_`
    Rust: (tidak ada padanan)
  - JS: ``.mct upgrade <luck/speed/fortune/combat>``
    Rust: (tidak ada padanan)
  - JS: ``.mct upgrade` _📈 Upgrade stats_`
    Rust: (tidak ada padanan)
  - JS: ``.minecraft on/off``
    Rust: (tidak ada padanan)
  - JS: ``␀`: ␀ _␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ *P␀*: _🧱 ␀ blocks | 💰 ␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀*`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀* _`␀`_`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀* _␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀*␀`
    Rust: (tidak ada padanan)
  - JS: `␀ @␀`
    Rust: (tidak ada padanan)
  - JS: `␀ Lv.␀`
    Rust: (tidak ada padanan)
  - JS: `␀ _💰 ␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ _📦 x␀ | 💰 ␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ → ␀ _💰 ␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `␀.`
    Rust: (tidak ada padanan)
  - JS: `␀_`}`
    Rust: (tidak ada padanan)
  - JS: `♻️ Duplikat: *␀* _💰 +␀_`
    Rust: (tidak ada padanan)
  - JS: `⚔️ Melawan *␀*`
    Rust: (tidak ada padanan)
  - JS: `⛏️ Hasil Menambang!`
    Rust: (tidak ada padanan)
  - JS: `⛏️ Menambang...`
    Rust: (tidak ada padanan)
  - JS: `⛏️ Mining & 🛠️ Crafting`
    Rust: (tidak ada padanan)
  - JS: `⛏️ tambang, 🛠️ craft, dan ⚔️ lawan mob di dunia minecraft`
    Rust: (tidak ada padanan)
  - JS: `✅ Aktif`
    Rust: (tidak ada padanan)
  - JS: `✨ Enchant!`
    Rust: (tidak ada padanan)
  - JS: `✨ Enchant: *␀* _(␀)_ ke pickaxe!`
    Rust: (tidak ada padanan)
  - JS: `✨ Pilih enchant`
    Rust: (tidak ada padanan)
  - JS: `❌ Nonaktif`
    Rust: (tidak ada padanan)
  - JS: `❤️ HP: ␀/␀`
    Rust: (tidak ada padanan)
  - JS: `❤️ Heal!`
    Rust: (tidak ada padanan)
  - JS: `❤️ ␀`
    Rust: (tidak ada padanan)
  - JS: `⬆️ Level ␀`
    Rust: (tidak ada padanan)
  - JS: `⬆️ Pickaxe Upgrade!`
    Rust: (tidak ada padanan)
  - JS: `⬆️ Tingkatkan stats`
    Rust: (tidak ada padanan)
  - JS: `⭐ +␀ EXP`
    Rust: (tidak ada padanan)
  - JS: `⭐ XP Boost x␀!`
    Rust: (tidak ada padanan)
  - JS: `𝗠𝗜𝗡𝗘𝗖𝗥𝗔𝗙𝗧 𝗚𝗔𝗠𝗘𝗦`
    Rust: (tidak ada padanan)
  - JS: `🌈 SSR PULL!`
    Rust: (tidak ada padanan)
  - JS: `🎁 Daily Reward!`
    Rust: (tidak ada padanan)
  - JS: `🎁 Result`
    Rust: (tidak ada padanan)
  - JS: `🎒 Inventory`
    Rust: (tidak ada padanan)
  - JS: `🎒 Koleksi Pickaxe`
    Rust: (tidak ada padanan)
  - JS: `🎟️ +␀ Tiket`
    Rust: (tidak ada padanan)
  - JS: `🎟️ +␀ Tiket Gacha`
    Rust: (tidak ada padanan)
  - JS: `🎮 Semua member wajib main Minecraft!`
    Rust: (tidak ada padanan)
  - JS: `🎰 Gacha!`
    Rust: (tidak ada padanan)
  - JS: `🎰 Jackpot!`
    Rust: (tidak ada padanan)
  - JS: `🎰 ␀`
    Rust: (tidak ada padanan)
  - JS: `🏆 Leaderboard`
    Rust: (tidak ada padanan)
  - JS: `🏆 Menang!`
    Rust: (tidak ada padanan)
  - JS: `🏦 Saldo: ␀`
    Rust: (tidak ada padanan)
  - JS: `👑 PRESTIGE!`
    Rust: (tidak ada padanan)
  - JS: `👑 Prestige`
    Rust: (tidak ada padanan)
  - JS: `👑 Top Pemain`
    Rust: (tidak ada padanan)
  - JS: `👤 Profil`
    Rust: (tidak ada padanan)
  - JS: `👹 Daftar Mob`
    Rust: (tidak ada padanan)
  - JS: `👾 ␀ mob`
    Rust: (tidak ada padanan)
  - JS: `💀 Kalah!`
    Rust: (tidak ada padanan)
  - JS: `💎 Hadiah Super Besar`
    Rust: (tidak ada padanan)
  - JS: `💥 JACKPOT BESAR!`
    Rust: (tidak ada padanan)
  - JS: `💰 +␀`
    Rust: (tidak ada padanan)
  - JS: `💰 Total: ␀`
    Rust: (tidak ada padanan)
  - JS: `💰 ␀`
    Rust: (tidak ada padanan)
  - JS: `💸 Ore Terjual!`
    Rust: (tidak ada padanan)
  - JS: `📈 Upgrade Shop`
    Rust: (tidak ada padanan)
  - JS: `📈 Upgrade!`
    Rust: (tidak ada padanan)
  - JS: `📊 Stats Detail`
    Rust: (tidak ada padanan)
  - JS: `📍 Sekarang di *␀*`
    Rust: (tidak ada padanan)
  - JS: `📚 Ore Book`
    Rust: (tidak ada padanan)
  - JS: `📜 Enchantments`
    Rust: (tidak ada padanan)
  - JS: `📦 Hasil Diambil!`
    Rust: (tidak ada padanan)
  - JS: `📦 Jumlah: ␀`
    Rust: (tidak ada padanan)
  - JS: `📦 ␀ items`
    Rust: (tidak ada padanan)
  - JS: `🔥 Smelting!`
    Rust: (tidak ada padanan)
  - JS: `🔥 Streak ␀`
    Rust: (tidak ada padanan)
  - JS: `🗺️ Daftar Biome`
    Rust: (tidak ada padanan)
  - JS: `🚪 Travel!`
    Rust: (tidak ada padanan)
  - JS: `🛒 Toko Pickaxe`
    Rust: (tidak ada padanan)
  - JS: `🛠️ Crafted!`
    Rust: (tidak ada padanan)
  - JS: `🛠️ Crafting`
    Rust: (tidak ada padanan)
  - JS: `🧪 Pilih resep`
    Rust: (tidak ada padanan)
  - JS: `🧱 +␀ ore`
    Rust: (tidak ada padanan)
  - JS: `🧱 Minecraft Game`
    Rust: (tidak ada padanan)
  - JS: `🧱 Minecraft OFF`
    Rust: (tidak ada padanan)
  - JS: `🧱 Minecraft ON`
    Rust: (tidak ada padanan)
  - JS: `🧱 ␀ jenis ore tersedia`
    Rust: (tidak ada padanan)
  - JS: `🧱 ␀ ore`
    Rust: (tidak ada padanan)
  - JS: `🪓 DAPAT PICKAXE: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🪓 Equip Pickaxe!`
    Rust: (tidak ada padanan)
  - JS: `🪓 Pickaxe Baru!`
    Rust: (tidak ada padanan)
  - JS: `🪓 Pilih pickaxe terbaik`
    Rust: (tidak ada padanan)
  - JS: `🪓 ␀ pickaxe`
    Rust: (tidak ada padanan)
  - JS: `🪙 +␀ Tokens`
    Rust: (tidak ada padanan)
  - JS: `🪙 Token Store`
    Rust: (tidak ada padanan)
  - JS: `🪙 Token Store!`
    Rust: (tidak ada padanan)
  - JS: `🪙 ␀ tokens`
    Rust: (tidak ada padanan)
  - JS: `🪧 Ketik `.mct help` untuk mulai`
    Rust: (tidak ada padanan)

### suitpvp (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/suitpvp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/suitpvp.rs`

  - JS: `*TIP: Reply pesan ini dengan pilihanmu!*`
    Rust: (tidak ada padanan)
  - JS: `> +Rp ␀`
    Rust: (tidak ada padanan)
  - JS: `> Menunggu @␀...`
    Rust: (tidak ada padanan)
  - JS: `> Menunggu lawan...`
    Rust: (tidak ada padanan)
  - JS: `> ⏱️ Timeout: 90 detik`
    Rust: (tidak ada padanan)
  - JS: `> 📩 Cek *Private Chat* untuk memilih!`
    Rust: (tidak ada padanan)
  - JS: `@␀ tidak merespon!`
    Rust: (tidak ada padanan)
  - JS: `Contoh: *batu*`
    Rust: (tidak ada padanan)
  - JS: `Ketik salah satu:`
    Rust: (tidak ada padanan)
  - JS: `Suit dibatalkan.`
    Rust: (tidak ada padanan)
  - JS: `[Suit] Failed to PM player 1:`
    Rust: (tidak ada padanan)
  - JS: `[Suit] Failed to PM player 2:`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *TIMEOUT!*`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Kedua pemain tidak memilih, suit dibatalkan!`
    Rust: (tidak ada padanan)
  - JS: `┃ ✊ *batu*`
    Rust: (tidak ada padanan)
  - JS: `┃ ✋ *kertas*`
    Rust: (tidak ada padanan)
  - JS: `┃ ✌️ *gunting*`
    Rust: (tidak ada padanan)
  - JS: `✅ Kamu memilih *␀* ␀`
    Rust: (tidak ada padanan)
  - JS: `✊✌️✋ *sᴜɪᴛ - ᴘɪʟɪʜ ᴊᴀᴡᴀʙᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `✊✌️✋ *sᴜɪᴛ ᴅɪᴍᴜʟᴀɪ!*`
    Rust: (tidak ada padanan)
  - JS: `✊✌️✋ *ʜᴀsɪʟ sᴜɪᴛ*`
    Rust: (tidak ada padanan)
  - JS: `✌️`
    Rust: (tidak ada padanan)
  - JS: `❌ @␀ menolak tantangan!`
    Rust: (tidak ada padanan)
  - JS: `🏆 @␀ menang!`
    Rust: (tidak ada padanan)
  - JS: `🕕 @␀ sudah memilih!`
    Rust: (tidak ada padanan)
  - JS: `🤝 *SERI!*`
    Rust: (tidak ada padanan)

### tictactoe (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/tictactoe.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/tictactoe.rs`

  - JS: `(r) =>`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `// ==================== Helper ====================`
    Rust: (tidak ada padanan)
  - JS: `// Check if it`
    Rust: (tidak ada padanan)
  - JS: `// Check if valid move`
    Rust: (tidak ada padanan)
  - JS: `// Continue game`
    Rust: (tidak ada padanan)
  - JS: `// Handle surrender`
    Rust: (tidak ada padanan)
  - JS: `// Make move`
    Rust: (tidak ada padanan)
  - JS: `// Reward winner`
    Rust: (tidak ada padanan)
  - JS: `> Room akan expired dalam 5 menit`
    Rust: `> Room akan expire dalam 5 menit`
  - JS: `> Selesaikan game kamu atau ketik *nyerah* untuk menyerah.`
    Rust: `Selesaikan game kamu atau ketik *nyerah* untuk menyerah.`
  - JS: `[r.game.playerX, r.game.playerO].filter(Boolean).includes(m.sender),`
    Rust: (tidak ada padanan)
  - JS: `` +`
    Rust: (tidak ada padanan)
  - JS: `` + `> Tidak ada pemenang!`,`
    Rust: (tidak ada padanan)
  - JS: `` + `␀`
    Rust: (tidak ada padanan)
  - JS: ``> Giliran: @␀`,`
    Rust: `> Giliran: @␀`
  - JS: ``@␀ menang! +Rp 500`,`
    Rust: (tidak ada padanan)
  - JS: ``@␀ menyerah!`
    Rust: (tidak ada padanan)
  - JS: ``␀`
    Rust: (tidak ada padanan)
  - JS: ``🎉 *GAME OVER!*`
    Rust: (tidak ada padanan)
  - JS: ``🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*`
    Rust: `🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*`
  - JS: ``🏆 @␀ menang! +Rp 1.000`,`
    Rust: (tidak ada padanan)
  - JS: ``🏳️ *MENYERAH!*`
    Rust: (tidak ada padanan)
  - JS: ``🤝 *SERI!*`
    Rust: (tidak ada padanan)
  - JS: `await safeReact(m, "🎉");`
    Rust: (tidak ada padanan)
  - JS: `await safeReact(m, "🏳️");`
    Rust: (tidak ada padanan)
  - JS: `await safeReact(m, "🤝");`
    Rust: (tidak ada padanan)
  - JS: `await safeReply(`
    Rust: (tidak ada padanan)
  - JS: `await safeReply(m, "❌ Bukan giliranmu!");`
    Rust: (tidak ada padanan)
  - JS: `await safeReply(m, "❌ Posisi sudah terisi!");`
    Rust: (tidak ada padanan)
  - JS: `await safeReply(m, "❌ Posisi tidak valid!");`
    Rust: (tidak ada padanan)
  - JS: `const board = renderBoard(room.game.render());`
    Rust: (tidak ada padanan)
  - JS: `const cells = arr.map((cell) => boardSymbols[String(cell)] || cell);`
    Rust: (tidak ada padanan)
  - JS: `const db = getDatabase();`
    Rust: (tidak ada padanan)
  - JS: `const isTie = room.game.board === 511 && !winner;`
    Rust: (tidak ada padanan)
  - JS: `const loser =`
    Rust: (tidak ada padanan)
  - JS: `const loser = m.sender;`
    Rust: (tidak ada padanan)
  - JS: `const move = parseInt(text);`
    Rust: (tidak ada padanan)
  - JS: `const player = room.game.playerX === m.sender ? 0 : 1;`
    Rust: (tidak ada padanan)
  - JS: `const result = room.game.turn(player, move - 1);`
    Rust: (tidak ada padanan)
  - JS: `const room = Object.values(global.tictactoeGames).find(`
    Rust: (tidak ada padanan)
  - JS: `const winner =`
    Rust: (tidak ada padanan)
  - JS: `const winner = room.game.winner;`
    Rust: (tidak ada padanan)
  - JS: `const winnerData = db.getUser(winner) || {};`
    Rust: (tidak ada padanan)
  - JS: `db.setUser(winner, winnerData);`
    Rust: (tidak ada padanan)
  - JS: `delete global.tictactoeGames[room.id];`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler, answerHandler }`
    Rust: (tidak ada padanan)
  - JS: `function renderBoard(arr) {`
    Rust: (tidak ada padanan)
  - JS: `if (!room) return false;`
    Rust: (tidak ada padanan)
  - JS: `if (isNaN(move) || move < 1 || move > 9) return false;`
    Rust: (tidak ada padanan)
  - JS: `if (isTie) {`
    Rust: (tidak ada padanan)
  - JS: `if (result === -1) {`
    Rust: (tidak ada padanan)
  - JS: `if (result === 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (room.game.currentTurn !== m.sender) {`
    Rust: (tidak ada padanan)
  - JS: `if (text === "nyerah" || text === "surrender" || text === "give up") {`
    Rust: (tidak ada padanan)
  - JS: `if (winner) {`
    Rust: (tidak ada padanan)
  - JS: `invalid position`
    Rust: (tidak ada padanan)
  - JS: `m,`
    Rust: (tidak ada padanan)
  - JS: `m.sender === room.game.playerX ? room.game.playerO : room.game.playerX;`
    Rust: (tidak ada padanan)
  - JS: `r.chat === m.chat &&`
    Rust: (tidak ada padanan)
  - JS: `r.state === "PLAYING" &&`
    Rust: (tidak ada padanan)
  - JS: `rate limit`
    Rust: (tidak ada padanan)
  - JS: `rate overlimit`
    Rust: (tidak ada padanan)
  - JS: `rate-overlimit`
    Rust: (tidak ada padanan)
  - JS: `return `┌───┬───┬───┐`
    Rust: `┌───┬───┬───┐`
  - JS: `return true;`
    Rust: (tidak ada padanan)
  - JS: `s active game`
    Rust: (tidak ada padanan)
  - JS: `s turn`
    Rust: (tidak ada padanan)
  - JS: `winner === room.game.playerX ? room.game.playerO : room.game.playerX;`
    Rust: (tidak ada padanan)
  - JS: `winnerData.koin = (winnerData.koin || 0) + 1000;`
    Rust: (tidak ada padanan)
  - JS: `winnerData.koin = (winnerData.koin || 0) + 500;`
    Rust: (tidak ada padanan)
  - JS: `{ mentions: [room.game.currentTurn] },`
    Rust: (tidak ada padanan)
  - JS: `{ mentions: [room.game.playerX, room.game.playerO] },`
    Rust: (tidak ada padanan)
  - JS: `{ mentions: [winner, loser] },`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `␀@s.whatsapp.net`
    Rust: `628999@s.whatsapp.net`
  - JS: `└───┴───┴───┘`;`
    Rust: `└───┴───┴───┘`

### ulartangga (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/ulartangga.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/ulartangga.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `.join("`
    Rust: (tidak ada padanan)
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> GG WP! Main lagi? `.ut create``
    Rust: (tidak ada padanan)
  - JS: `> Giliran: @␀`
    Rust: (tidak ada padanan)
  - JS: `> Host: @␀`
    Rust: (tidak ada padanan)
  - JS: `> Ketik *kocok*`
    Rust: (tidak ada padanan)
  - JS: `> Ketik *kocok* untuk lempar dadu!`
    Rust: (tidak ada padanan)
  - JS: `> Ketik `␀ut create` untuk membuat room.`
    Rust: (tidak ada padanan)
  - JS: `> Masih ada sesi permainan di chat ini!`
    Rust: (tidak ada padanan)
  - JS: `> Min 2 player, Max 4 player`
    Rust: (tidak ada padanan)
  - JS: `> Status: ␀`
    Rust: (tidak ada padanan)
  - JS: `> Total: ␀/4`
    Rust: (tidak ada padanan)
  - JS: `> ␀ut start`` : "🕕 Butuh 1 player lagi"}`
    Rust: (tidak ada padanan)
  - JS: `> 🎯 Giliran: @␀`
    Rust: (tidak ada padanan)
  - JS: `Ayo bergabung!`
    Rust: (tidak ada padanan)
  - JS: `Ayo main!`
    Rust: (tidak ada padanan)
  - JS: `Lempar dadu!`
    Rust: (tidak ada padanan)
  - JS: `Main ular tangga bersama player lain dengan visual board`
    Rust: `Main ular tangga bersama player lain dengan visual board — BLOKIRAN`
  - JS: `Naiki tangga, hindari ular, sampai ke 100!`
    Rust: (tidak ada padanan)
  - JS: `Permainan klasik yang penuh petualangan!`
    Rust: (tidak ada padanan)
  - JS: `Permainan klasik!`
    Rust: (tidak ada padanan)
  - JS: `Room berhasil dibuat!`
    Rust: (tidak ada padanan)
  - JS: `[ULARTANGGA ERROR]`
    Rust: (tidak ada padanan)
  - JS: `[UT] Failed to give reward:`
    Rust: (tidak ada padanan)
  - JS: `␀ @␀ sampai ke 100!`
    Rust: (tidak ada padanan)
  - JS: `␀ menang!`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀: *␀* → *␀*␀`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀: @␀`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀: @␀ - Pos: ␀`
    Rust: (tidak ada padanan)
  - JS: `␀/4 players`
    Rust: (tidak ada padanan)
  - JS: `␀`)`
    Rust: (tidak ada padanan)
  - JS: `┃ ℹ️ `␀ut info` - Info room`
    Rust: (tidak ada padanan)
  - JS: `┃ ▶️ `␀ut start` - Mulai`
    Rust: (tidak ada padanan)
  - JS: `┃ ▶️ `␀ut start` - Mulai game`
    Rust: (tidak ada padanan)
  - JS: `┃ ⚡ +␀ Energi`
    Rust: (tidak ada padanan)
  - JS: `┃ ➕ `␀ut join` - Gabung`
    Rust: (tidak ada padanan)
  - JS: `┃ ➕ `␀ut join` - Gabung room`
    Rust: (tidak ada padanan)
  - JS: `┃ ⭐ +␀ EXP`
    Rust: (tidak ada padanan)
  - JS: `┃ 🎲 `␀ut create` - Buat room`
    Rust: (tidak ada padanan)
  - JS: `┃ 👑 Host: @␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 👥 Players: 1/4`
    Rust: (tidak ada padanan)
  - JS: `┃ 👥 Players: ␀/4`
    Rust: (tidak ada padanan)
  - JS: `┃ 💰 +␀ Koin`
    Rust: (tidak ada padanan)
  - JS: `┃ 📍 Status: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🗑️ `␀ut delete` - Hapus room`
    Rust: (tidak ada padanan)
  - JS: `┃ 🗺️ Map: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🚪 `␀ut exit` - Keluar`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 🎁 *HADIAH* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 🎮 *COMMANDS* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 🏆 *HADIAH* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 👥 *PLAYERS* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *INFO ROOM* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ROOM* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *PLAYER BERGABUNG*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ROOM SUDAH ADA*`
    Rust: (tidak ada padanan)
  - JS: `❌ Bukan giliranmu!`
    Rust: (tidak ada padanan)
  - JS: `❌ Game sedang berjalan, tidak bisa join!`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya host yang dapat memulai permainan!`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya host yang dapat menghapus room!`
    Rust: (tidak ada padanan)
  - JS: `❌ Kamu sudah bergabung di room ini!`
    Rust: (tidak ada padanan)
  - JS: `❌ Kamu tidak ada di permainan ini!`
    Rust: (tidak ada padanan)
  - JS: `❌ Minimal 2 player untuk bermain!`
    Rust: (tidak ada padanan)
  - JS: `❌ Permainan sudah berjalan!`
    Rust: (tidak ada padanan)
  - JS: `❌ Room sudah penuh! (Max 4 player)`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada sesi permainan!`
    Rust: (tidak ada padanan)
  - JS: `🎉 *PEMENANG!*`
    Rust: (tidak ada padanan)
  - JS: `🎮 GAME STARTED`
    Rust: (tidak ada padanan)
  - JS: `🎲 *DADU: ␀* ␀`
    Rust: (tidak ada padanan)
  - JS: `🎲 GILIRAN`
    Rust: (tidak ada padanan)
  - JS: `🎲 ROOM CREATED`
    Rust: (tidak ada padanan)
  - JS: `🏆 WINNER!`
    Rust: (tidak ada padanan)
  - JS: `🐍 *Kena ular!*`
    Rust: (tidak ada padanan)
  - JS: `🐍🎲 *INFO ROOM*`
    Rust: (tidak ada padanan)
  - JS: `🐍🎲 *PERMAINAN DIMULAI!*`
    Rust: (tidak ada padanan)
  - JS: `🐍🎲 *ULAR TANGGA*`
    Rust: (tidak ada padanan)
  - JS: `🐍🎲 ULAR TANGGA`
    Rust: (tidak ada padanan)
  - JS: `👋 @␀ keluar dari permainan.`
    Rust: (tidak ada padanan)
  - JS: `👑 Host dipindahkan ke @␀`
    Rust: (tidak ada padanan)
  - JS: `👥 PLAYER JOINED`
    Rust: (tidak ada padanan)
  - JS: `🗑️`
    Rust: (tidak ada padanan)
  - JS: `🗑️ Room berhasil dihapus!`
    Rust: (tidak ada padanan)
  - JS: `🗑️ Room dihapus karena tidak ada player.`
    Rust: (tidak ada padanan)
  - JS: `🪜 *Naik tangga!*`
    Rust: (tidak ada padanan)

### wwkill (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/wwkill.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/wwkill.rs`

  - JS: `[WWKILL ERROR]`
    Rust: (tidak ada padanan)

### wwprotect (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/wwprotect.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/wwprotect.rs`

  - JS: `[WWPROTECT ERROR]`
    Rust: (tidak ada padanan)

### wwsee (game)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/game/wwsee.js`
- Rust: `/workspaces/oktz-rs/plugin/src/game/wwsee.rs`

  - JS: `[WWSEE ERROR]`
    Rust: (tidak ada padanan)

### absen (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/absen.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/absen.rs`

  - JS: `D MMMM YYYY`
    Rust: (tidak ada padanan)

### acc (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/acc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/acc.rs`

  - JS: `*␀.* @␀`
    Rust: (tidak ada padanan)
  - JS: `2-digit`
    Rust: (tidak ada padanan)
  - JS: `200`
    Rust: (tidak ada padanan)
  - JS: `> Contoh: `␀acc ␀ 1|2|3``
    Rust: (tidak ada padanan)
  - JS: `> Gunakan `␀acc approve all` atau `␀acc reject all``
    Rust: (tidak ada padanan)
  - JS: `> Gunakan `␀acc list` untuk melihat daftar.`
    Rust: (tidak ada padanan)
  - JS: `> Total: ␀ permintaan`
    Rust: (tidak ada padanan)
  - JS: `> ✅ Berhasil: ␀`
    Rust: (tidak ada padanan)
  - JS: `> ✅ ␀/␀ berhasil`
    Rust: (tidak ada padanan)
  - JS: `> ❌ Gagal: ␀`
    Rust: (tidak ada padanan)
  - JS: `> 📊 Total: ␀`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀ — ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *␀ SEMUA*`
    Rust: (tidak ada padanan)
  - JS: `❌ Nomor tidak valid.`
    Rust: (tidak ada padanan)
  - JS: `❌ ␀ — Error`
    Rust: (tidak ada padanan)
  - JS: `📋 *ʜᴀsɪʟ ␀*`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴅᴀꜰᴛᴀʀ ᴘᴇʀᴍɪɴᴛᴀᴀɴ ᴍᴀsᴜᴋ*`
    Rust: (tidak ada padanan)
  - JS: `📨 ␀`
    Rust: (tidak ada padanan)
  - JS: `📭 Tidak ada permintaan masuk yang tertunda.`
    Rust: (tidak ada padanan)
  - JS: `📱 ␀`
    Rust: (tidak ada padanan)
  - JS: `🕐 ␀`
    Rust: (tidak ada padanan)

### add (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/add.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/add.rs`

  - JS: `200`
    Rust: (tidak ada padanan)
  - JS: `408`
    Rust: (tidak ada padanan)
  - JS: `62`
    Rust: `62␀`
  - JS: `> Bot tidak memiliki akses ke grup ini!`
    Rust: (tidak ada padanan)
  - JS: ``␀add <nomor> <link_grup>``
    Rust: ``.add <nomor> <link_grup>``
  - JS: `not-authorized`
    Rust: (tidak ada padanan)
  - JS: `• @␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `⏭️ *sᴜᴅᴀʜ ᴅɪ ɢʀᴜᴘ (␀):*`
    Rust: `⏭️ *sᴜᴅᴀʜ ᴅɪ ɢʀᴜᴘ (␀)*:`
  - JS: `❌ *ɢᴀɢᴀʟ (␀):*`
    Rust: `❌ *ɢᴀɢᴀʟ*`
  - JS: `📨 *Dan ada juga *␀* member yang diundang:*`
    Rust: (tidak ada padanan)

### addantilink (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/addantilink.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/addantilink.rs`

  - JS: `✅ *ᴀɴᴛɪʟɪɴᴋ ᴅɪᴛᴀᴍʙᴀʜ*`
    Rust: `✅ *ᴀɴᴛɪʟɪɴᴋ ᴅɪᴛᴀᴍʘᴀʜ*`

### addcmdsticker (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/addcmdsticker.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/addcmdsticker.rs`

  - JS: `> Pastikan command yang ingin dijadikan shortcut valid.`
    Rust: `> Reply sticker + ketik command yang ingin dijadikan shortcut.`
  - JS: `> 🖼️ Sticker → `.␀``
    Rust: (tidak ada padanan)
  - JS: `_Kirim sticker tersebut untuk menjalankan command!_`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴀᴋᴛɪꜰ* 」`
    Rust: `╭┈┈⬡「 📋 *ᴀᴋᴛɪꜰ」`
  - JS: `✅ *sᴛɪᴄᴋᴇʀ ᴄᴏᴍᴍᴀɴᴅ ᴅɪᴛᴀᴍʙᴀʜᴋᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Command `␀` tidak ditemukan!`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal menyimpan sticker command!`
    Rust: (tidak ada padanan)

### addtoxic (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/addtoxic.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/addtoxic.rs`

  - JS: `> Gunakan: `.addtoxic <kata>``
    Rust: `> Gunakan: `.␀addtoxic <kata>``
  - JS: `✅ *ᴋᴀᴛᴀ ᴛᴏxɪᴄ ᴅɪᴛᴀᴍʙᴀʜ*`
    Rust: `✅ *ᴋᴀᴛᴀ ᴛᴏxɪᴄ ᴅɪᴛᴀᴍʘᴀʜ*`

### afk (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/afk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/afk.rs`

  - JS: ````@␀ sudah kembali!````
    Rust: (tidak ada padanan)
  - JS: ````Hustt, jangan di ganggu!``` `@␀` lagi AFK`
    Rust: (tidak ada padanan)
  - JS: `|␀`
    Rust: (tidak ada padanan)
  - JS: `␀|␀`
    Rust: (tidak ada padanan)
  - JS: `🍀 `Durasi AFK:` *␀*`
    Rust: (tidak ada padanan)
  - JS: `🍀 `Sejak:` *␀ yang lalu*`
    Rust: (tidak ada padanan)
  - JS: `👋 *ᴀꜰᴋ ʙᴇʀᴀᴋʜɪʀ*`
    Rust: (tidak ada padanan)
  - JS: `💤 *ᴜsᴇʀ ᴀꜰᴋ*`
    Rust: (tidak ada padanan)

### antibot (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/antibot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/antibot.rs`

  - JS: `%␀%`
    Rust: (tidak ada padanan)
  - JS: `3EB0`
    Rust: (tidak ada padanan)
  - JS: `94DD`
    Rust: (tidak ada padanan)
  - JS: `WAMID.`
    Rust: (tidak ada padanan)
  - JS: `custom-ad-reply`
    Rust: (tidak ada padanan)
  - JS: `fake-forward-score`
    Rust: (tidak ada padanan)
  - JS: `flag-isBaileys`
    Rust: (tidak ada padanan)
  - JS: `id-3EB0`
    Rust: (tidak ada padanan)
  - JS: `id-94DD`
    Rust: (tidak ada padanan)
  - JS: `id-B1E`
    Rust: (tidak ada padanan)
  - JS: `id-B24E`
    Rust: (tidak ada padanan)
  - JS: `id-BAE5`
    Rust: (tidak ada padanan)
  - JS: `id-length-16`
    Rust: (tidak ada padanan)
  - JS: `id-length-suspicious`
    Rust: (tidak ada padanan)
  - JS: `id-length-unusual`
    Rust: (tidak ada padanan)
  - JS: `id-lowercase-signature`
    Rust: (tidak ada padanan)
  - JS: `id-upper-hex`
    Rust: (tidak ada padanan)
  - JS: `invisible-characters`
    Rust: (tidak ada padanan)
  - JS: `message-deviceSent`
    Rust: (tidak ada padanan)
  - JS: `message-deviceWrapper`
    Rust: (tidak ada padanan)
  - JS: `message-type-␀`
    Rust: (tidak ada padanan)
  - JS: `participant-highDevice`
    Rust: (tidak ada padanan)
  - JS: `pushname-suspicious`
    Rust: (tidak ada padanan)
  - JS: `pushname-unknown`
    Rust: (tidak ada padanan)
  - JS: `🤖 *AntiBot* — @%user% terdeteksi sebagai bot dan di-kick.`
    Rust: (tidak ada padanan)

### anticulik (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/anticulik.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/anticulik.rs`

  - JS: `> Bot akan keluar dari grup ini`
    Rust: (tidak ada padanan)
  - JS: `> Bot ditambah oleh ␀ tanpa izin`
    Rust: (tidak ada padanan)
  - JS: `Minimal izin dulu ya bang, jangan asal culik 🗿`
    Rust: (tidak ada padanan)
  - JS: `_Hubungi owner untuk menambahkan bot dengan cara yang benar_`
    Rust: (tidak ada padanan)

### anticustom (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/anticustom.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/anticustom.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `*Reply pesan ini dengan jawabanmu*`
    Rust: (tidak ada padanan)
  - JS: `*Reply pesan ini dengan:* `hapus` atau `kick``
    Rust: (tidak ada padanan)
  - JS: `*Reply pesan ini dengan:* `ya` untuk simpan atau `batal` untuk membatalkan`
    Rust: (tidak ada padanan)
  - JS: `.test()`
    Rust: (tidak ada padanan)
  - JS: `> *Contains*: kirim kata dipisah koma atau baris baru`
    Rust: (tidak ada padanan)
  - JS: `> *Regex*: awali jawaban dengan `regex:``
    Rust: (tidak ada padanan)
  - JS: `> Action: *␀*`
    Rust: `> action: *␀*`
  - JS: `> Hapus rule lama dulu: `␀anticustom list` lalu `␀anticustom del <judul>``
    Rust: (tidak ada padanan)
  - JS: `> Judul: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Pattern: ␀``).join(", ")}`
    Rust: (tidak ada padanan)
  - JS: `> Status otomatis: *ON*`
    Rust: `> Status: *ON*`
  - JS: `> Tipe deteksi: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Total rule baru: *␀*`
    Rust: `> Total rule: *␀*`
  - JS: `> `anjing, goblok, tolol``
    Rust: (tidak ada padanan)
  - JS: `> `regex: (anj|anjing|a+n+j+)``
    Rust: (tidak ada padanan)
  - JS: `Aku belum dapat kata yang ingin dideteksi.`
    Rust: (tidak ada padanan)
  - JS: `Apakah sudah sesuai?`
    Rust: (tidak ada padanan)
  - JS: `Batasnya *␀ rule per grup* supaya pengecekan tiap pesan tetap cepat.`
    Rust: (tidak ada padanan)
  - JS: `Berarti kamu mau ini ya:`
    Rust: (tidak ada padanan)
  - JS: `Contoh contains:`
    Rust: (tidak ada padanan)
  - JS: `Contoh regex:`
    Rust: (tidak ada padanan)
  - JS: `Jawabannya masih kosong.`
    Rust: (tidak ada padanan)
  - JS: `Kalau mau lihat panduan lagi, ketik `␀anticustom``
    Rust: (tidak ada padanan)
  - JS: `Kamu bisa pilih salah satu format:`
    Rust: (tidak ada padanan)
  - JS: `Oke siap, berikut detail yang kamu mau:`
    Rust: (tidak ada padanan)
  - JS: `Oke siap, kalau pesan member mengandung kata-kata itu, kamu ingin aku *hapus pesan* atau langsung *kick*?`
    Rust: (tidak ada padanan)
  - JS: `Oke, judulnya *␀*.`
    Rust: (tidak ada padanan)
  - JS: `Pakai `contains` dengan kata kunci aja, atau pola yang lebih sederhana.`
    Rust: (tidak ada padanan)
  - JS: `Regex ditolak: ␀.`
    Rust: (tidak ada padanan)
  - JS: `Regex kosong. Isi setelah `regex:` ya.`
    Rust: (tidak ada padanan)
  - JS: `Sekarang, berikan kata-kata yang ingin dideteksi oleh aku.`
    Rust: (tidak ada padanan)
  - JS: `hapus pesan`
    Rust: (tidak ada padanan)
  - JS: `kick member`
    Rust: (tidak ada padanan)
  - JS: `␀ #␀`
    Rust: `␀ #`
  - JS: `␀. `␀``).join("`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: `␀:␀:␀`
  - JS: `✅ *AntiCustom berhasil dibuat*`
    Rust: (tidak ada padanan)
  - JS: `✅ Oke, sesi AntiCustom dibatalkan. Kalau mau mulai lagi, ketik `.anticustom add`.`
    Rust: (tidak ada padanan)
  - JS: `❌ Balas dengan `hapus` atau `kick` ya.`
    Rust: (tidak ada padanan)
  - JS: `❌ Balas dengan `ya` untuk simpan atau `batal` untuk membatalkan.`
    Rust: (tidak ada padanan)
  - JS: `❌ Group ini sudah punya ␀ rule.`
    Rust: (tidak ada padanan)
  - JS: `❌ Judul harus 2-40 karakter ya.`
    Rust: (tidak ada padanan)
  - JS: `❌ ␀`
    Rust: (tidak ada padanan)
  - JS: `🛡️ *Pertanyaan 2/4*`
    Rust: `*Pertanyaan 1/4*`
  - JS: `🛡️ *Pertanyaan 3/4*`
    Rust: `*Pertanyaan 1/4*`
  - JS: `🛡️ *Pertanyaan 4/4*`
    Rust: `*Pertanyaan 1/4*`

### antidocument (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/antidocument.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/antidocument.rs`

  - JS: `%␀%`
    Rust: (tidak ada padanan)
  - JS: `⚠ *AntiDocument* — Dokumen dari @%user% dihapus.`
    Rust: (tidak ada padanan)

### antimedia (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/antimedia.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/antimedia.rs`

  - JS: `%␀%`
    Rust: (tidak ada padanan)
  - JS: `⚠ *AntiMedia* — Media dari @%user% dihapus.`
    Rust: (tidak ada padanan)

### antispam (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/antispam.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/antispam.rs`

  - JS: `> Peringatan ke-␀ dari maksimal 3 peringatan`
    Rust: (tidak ada padanan)
  - JS: `> Teruntuk: @␀`
    Rust: (tidak ada padanan)
  - JS: `Halo @␀, tolong jangan melakukan pengiriman pesan berulang-ulang di grup ini secara cepat! Sistem kami mendeteksi aktivitasmu sebagai spam. Mohon hargai kenyamanan member lainnya`
    Rust: (tidak ada padanan)
  - JS: `Kamu telah mendapatkan 3 kali teguran peringatan karena mengirim pesan spam secara berkelanjutan. Harap segera berhenti melakukan spam atau jajaran admin grup dapat mengambil tindakan tegas terhadap pelanggaran ini!`
    Rust: (tidak ada padanan)
  - JS: `Maaf sekali @␀, kamu akan dikeluarkan secara paksa oleh sistem karena kamu terdeteksi melakukan aksi spam brutal di grup ini!`
    Rust: (tidak ada padanan)
  - JS: `Telah terdeteksi aktivitas spam brutal dari @␀, namun sistem bot sayangnya tidak dapat menendang member tersebut karena bot saat ini tidak memiliki akses sebagai admin grup. Tolong jadikan bot admin agar fitur ini bekerja maksimal`
    Rust: (tidak ada padanan)
  - JS: `␀_␀`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *PERINGATAN SPAM MAKSIMAL*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *SPAM TERDETEKSI*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *TEGURAN SPAM TERDETEKSI*`
    Rust: (tidak ada padanan)
  - JS: `🛑 *SPAMMER DIKELUARKAN*`
    Rust: (tidak ada padanan)

### antisticker (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/antisticker.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/antisticker.rs`

  - JS: `%␀%`
    Rust: (tidak ada padanan)
  - JS: `⚠ *AntiSticker* — Sticker dari @%user% dihapus.`
    Rust: (tidak ada padanan)

### antitoxic (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/antitoxic.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/antitoxic.rs`

  - JS: `%␀%`
    Rust: (tidak ada padanan)
  - JS: `(^|\s|[^a-zA-Z0-9])␀($|\s|[^a-zA-Z0-9])`
    Rust: (tidak ada padanan)
  - JS: `Peringatan ke %warn% dari %max%, pelanggaran berikutnya bisa di-%method%.`
    Rust: (tidak ada padanan)
  - JS: `\$&`
    Rust: (tidak ada padanan)
  - JS: `⚠ @%user% berkata kasar.`
    Rust: (tidak ada padanan)
  - JS: `🚫 @%user% di-%method% karena toxic. (%warn%/%max%)`
    Rust: (tidak ada padanan)

### autoai (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/autoai.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/autoai.rs`

  - JS: `.ogg`
    Rust: (tidak ada padanan)
  - JS: `Gagal generate audio dari API Firefly`
    Rust: (tidak ada padanan)
  - JS: `[AutoAI Voice] Error:`
    Rust: (tidak ada padanan)
  - JS: `[AutoAI] FFmpeg error:`
    Rust: (tidak ada padanan)
  - JS: `audio/ogg; codecs=opus`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg -y -i "␀" -c:a libopus -b:a 64k -ac 1 -ar 48000 "␀"`
    Rust: (tidak ada padanan)
  - JS: `tts_␀.mp3`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya admin yang bisa mengatur ini!`
    Rust: `❌ Hanya admin yang bisa menggunakan fitur ini!`

### autoforward (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/autoforward.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/autoforward.rs`

  - JS: `> Gunakan: `␀autoforward on/off``
    Rust: `> Gunakan `␀autoforward on/off``

### autoreply (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/autoreply.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/autoreply.rs`

  - JS: `!== null`
    Rust: (tidak ada padanan)
  - JS: `> `␀autoreply global add halo|Hai {name}!``
    Rust: `> `␀autoreply global add halo|Hai {␀}!``
  - JS: `[Autoreply] Failed to download direct image:`
    Rust: (tidak ada padanan)
  - JS: `[Autoreply] Failed to download quoted image:`
    Rust: (tidak ada padanan)
  - JS: `{name} • {tag} • {sender} • {botname} • {time} • {date}`
    Rust: (tidak ada padanan)
  - JS: `• Image: ✅ Tersimpan`
    Rust: (tidak ada padanan)
  - JS: `• {botname} - Nama bot`
    Rust: `• {␀} - Nama bot`
  - JS: `• {date} - Tanggal sekarang`
    Rust: `• {␀} - Tanggal sekarang`
  - JS: `• {name} - Nama user`
    Rust: `• {␀} - Nama user`
  - JS: `• {sender} - Nomor user`
    Rust: `• {␀} - Nomor user`
  - JS: `• {tag} - Tag @user`
    Rust: `• {␀} - Tag @user`
  - JS: `• {time} - Waktu sekarang`
    Rust: `• {␀} - Waktu sekarang`
  - JS: `• ␀ar add halo|Hai {name}! 👋`
    Rust: `• ␀ar add halo|Hai {␀}! 👋`
  - JS: `␀_␀_␀.jpg`
    Rust: (tidak ada padanan)

### botmodegrup (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/botmode.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/botmode.rs`

  - JS: `╰───────────────`
    Rust: `␀╰───────────────`
  - JS: `❌ Mode tidak valid. Pilihan: `␀``
    Rust: (tidak ada padanan)

### cekabsen (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/cekabsen.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/cekabsen.rs`

  - JS: `> *.mulaiabsen [keterangan]*`
    Rust: (tidak ada padanan)
  - JS: `> Admin dapat memulai dengan`
    Rust: (tidak ada padanan)
  - JS: `> Belum ada sesi absen di grup ini!`
    Rust: (tidak ada padanan)
  - JS: `D MMMM YYYY`
    Rust: (tidak ada padanan)
  - JS: `HH:mm`
    Rust: (tidak ada padanan)
  - JS: `┃ ␀. @␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴛɪᴅᴀᴋ ᴀᴅᴀ ᴀʙsᴇɴ*`
    Rust: (tidak ada padanan)

### cekidgc (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/cekidgc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/cekidgc.rs`

  - JS: `[CekIdGc] Error:`
    Rust: (tidak ada padanan)

### cekonline (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/cekonline.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/cekonline.rs`

  - JS: `presence.update`
    Rust: (tidak ada padanan)

### checksewa (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/checksewa.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/checksewa.rs`

  - JS: `D MMMM YYYY`
    Rust: (tidak ada padanan)
  - JS: `D MMMM YYYY HH:mm`
    Rust: (tidak ada padanan)

### clearchat (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/clearchat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/clearchat.rs`

  - JS: `> Chat grup telah dibersihkan oleh @␀`
    Rust: (tidak ada padanan)
  - JS: `Chat grup di wa bot telah dibersihkan oleh @␀`
    Rust: (tidak ada padanan)
  - JS: `Silahkan lihat sendiri di wa bot kamu`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴄʜᴀᴛ ᴅɪʙᴇʀsɪʜᴋᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `🗑️`
    Rust: (tidak ada padanan)

### delete (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/delete.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/delete.rs`

  - JS: `> Pesan mungkin sudah dihapus atau terlalu lama.`
    Rust: (tidak ada padanan)
  - JS: `[Delete Error]`
    Rust: (tidak ada padanan)
  - JS: `not found`
    Rust: (tidak ada padanan)
  - JS: `tidak terhapus`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal menghapus!*`
    Rust: (tidak ada padanan)

### delstickercmd (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/delstickercmd.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/delstickercmd.rs`

  - JS: `sticker yang di-reply`
    Rust: (tidak ada padanan)

### deltoxic (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/deltoxic.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/deltoxic.rs`

  - JS: `> Gunakan: `.deltoxic <kata>``
    Rust: `> Gunakan: `.␀deltoxic <kata>``

### demote (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/demote.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/demote.rs`

  - JS: `User tidak ditemukan`
    Rust: `> User tidak ditemukan di grup!`

### getpp (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/getpp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/getpp.rs`

  - JS: `62`
    Rust: `62␀`

### goodbye (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/goodbye.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/goodbye.rs`

  - JS: `*Sayonara* @␀ 👋`
    Rust: (tidak ada padanan)
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Goodbye diaktifkan di *␀* grup!`
    Rust: (tidak ada padanan)
  - JS: `> Goodbye dinonaktifkan di *␀* grup!`
    Rust: `> Goodbye sudah nonaktif di grup ini.`
  - JS: `> 「 ␀ 」`
    Rust: (tidak ada padanan)
  - JS: `> 🏠 *Nama* : ␀`
    Rust: (tidak ada padanan)
  - JS: `> 👥 *Sisa Member* : ␀`
    Rust: (tidak ada padanan)
  - JS: `> 📅 *Tanggal* : ␀`
    Rust: (tidak ada padanan)
  - JS: `Ada perpisahan kecil hari ini.`
    Rust: (tidak ada padanan)
  - JS: `Bukan akhir, hanya sampai jumpa.`
    Rust: (tidak ada padanan)
  - JS: `Bye bye`
    Rust: (tidak ada padanan)
  - JS: `Doakan yang terbaik untuknya ya.`
    Rust: (tidak ada padanan)
  - JS: `Goodbye @␀, from grup ␀`
    Rust: (tidak ada padanan)
  - JS: `Goodbye Canvas Error:`
    Rust: (tidak ada padanan)
  - JS: `Goodbye Error:`
    Rust: (tidak ada padanan)
  - JS: `Goodbye from ␀`
    Rust: (tidak ada padanan)
  - JS: `HH:mm`
    Rust: (tidak ada padanan)
  - JS: `Hari ini satu tomodachi harus berpamitan.`
    Rust: (tidak ada padanan)
  - JS: `Hati-hati`
    Rust: (tidak ada padanan)
  - JS: `Jaga diri baik-baik ya, tomodachi.`
    Rust: (tidak ada padanan)
  - JS: `Kenangan di sini bakal tetap ada.`
    Rust: (tidak ada padanan)
  - JS: `Oyasumi~`
    Rust: (tidak ada padanan)
  - JS: `Pintu selalu terbuka kalau suatu saat mau kembali.`
    Rust: (tidak ada padanan)
  - JS: `Sampai jumpa`
    Rust: (tidak ada padanan)
  - JS: `Satu bintang berpindah langit malam ini.`
    Rust: (tidak ada padanan)
  - JS: `See you`
    Rust: (tidak ada padanan)
  - JS: `Selamat tinggal semuanya! 👋`
    Rust: (tidak ada padanan)
  - JS: `Semoga hari-harimu selalu hangat.`
    Rust: (tidak ada padanan)
  - JS: `Semoga kita bisa bertemu lagi di lain waktu.`
    Rust: (tidak ada padanan)
  - JS: `Semoga langkahmu selalu dimudahkan ke depannya.`
    Rust: (tidak ada padanan)
  - JS: `Semoga perjalanan barunya penuh kebaikan.`
    Rust: (tidak ada padanan)
  - JS: `Sisa ␀ member`
    Rust: (tidak ada padanan)
  - JS: `Terima kasih sudah bergabung di *␀*`
    Rust: (tidak ada padanan)
  - JS: `Terima kasih sudah jadi bagian dari grup ini.`
    Rust: (tidak ada padanan)
  - JS: `Terima kasih sudah pernah berjalan bersama.`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀, *@␀* 🤍`
    Rust: (tidak ada padanan)
  - JS: `✅ *ɢᴏᴏᴅʙʏᴇ ɢʟᴏʙᴀʟ ᴏɴ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴏᴏᴅʙʏᴇ ɢʟᴏʙᴀʟ ᴏꜰꜰ*`
    Rust: (tidak ada padanan)
  - JS: `🌌 Minna-san...`
    Rust: (tidak ada padanan)
  - JS: `🌙 Oyasumi~ minna-san...`
    Rust: (tidak ada padanan)
  - JS: `🌸 _Sampai jumpa lagi, tomodachi._ 🤍`
    Rust: (tidak ada padanan)
  - JS: `👋 *Sayonara* *@␀*`
    Rust: (tidak ada padanan)
  - JS: `👋 Sayonara @␀!`
    Rust: (tidak ada padanan)
  - JS: `👋 Selamat Tinggal`
    Rust: (tidak ada padanan)
  - JS: `👋🏻 *SAYONARA MEMBER* 👋🏻`
    Rust: (tidak ada padanan)
  - JS: `💌 *Pesan*`
    Rust: (tidak ada padanan)
  - JS: `💫 Sayonara~`
    Rust: (tidak ada padanan)
  - JS: `📌 *INFO GROUP*`
    Rust: (tidak ada padanan)
  - JS: `🥀 Minna-san...`
    Rust: (tidak ada padanan)

### groupinfo (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/groupinfo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/groupinfo.rs`

  - JS: `AntiBot: ␀`
    Rust: `AntiBot: ❌`
  - JS: `AntiDocument: ␀`
    Rust: `AntiDocument: ❌`
  - JS: `AntiHidetag: ␀`
    Rust: (tidak ada padanan)
  - JS: `AntiLink: ␀`
    Rust: `AntiLink: ✅`
  - JS: `AntiMedia: ␀`
    Rust: (tidak ada padanan)
  - JS: `AntiRemove: ␀`
    Rust: (tidak ada padanan)
  - JS: `AntiSticker: ␀`
    Rust: (tidak ada padanan)
  - JS: `AntiToxic: ␀`
    Rust: (tidak ada padanan)
  - JS: `AutoAI: ␀`
    Rust: (tidak ada padanan)
  - JS: `AutoDL: ␀`
    Rust: (tidak ada padanan)
  - JS: `AutoMedia: ␀`
    Rust: (tidak ada padanan)
  - JS: `AutoSticker: ␀`
    Rust: (tidak ada padanan)
  - JS: `Autoreply: ␀`
    Rust: (tidak ada padanan)
  - JS: `D MMMM YYYY`
    Rust: (tidak ada padanan)
  - JS: `Goodbye: ␀`
    Rust: `Goodbye: ❌`
  - JS: `Welcome: ␀`
    Rust: `Welcome: ✅`

### hidetag2 (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/hidetag2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/hidetag2.rs`

  - JS: `0@s.whatsapp.net`
    Rust: `628999@s.whatsapp.net`
  - JS: `status@broadcast`
    Rust: (tidak ada padanan)

### intro (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/intro.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/intro.rs`

  - JS: `D MMMM YYYY`
    Rust: (tidak ada padanan)
  - JS: `Foto-AI`
    Rust: (tidak ada padanan)
  - JS: `HH:mm`
    Rust: (tidak ada padanan)

### jadwalgroup (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/jadwalgroup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/jadwalgroup.rs`

  - JS: `> Jadwal *buka grup* otomatis telah dihapus.`
    Rust: `> Jadwal *␀* otomatis telah dihapus.`
  - JS: `> Jadwal *tutup grup* otomatis telah dihapus.`
    Rust: `> Jadwal *␀* otomatis telah dihapus.`
  - JS: `┃ ㊗ ⏱️ ᴡᴀᴋᴛᴜ: *␀ WIB*`
    Rust: `┃ ㊗ ⏱️ ᴡᴏᴋᴛᴜ: *␀ WIB*`

### kick (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/kick.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/kick.rs`

  - JS: `> Tidak bisa kick bot sendiri!`
    Rust: `tidak bisa kick bot sendiri`
  - JS: `✅ @␀ telah dikeluarkan dari grup ini.`
    Rust: `telah dikeluarkan dari grup`

### listwarn (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/listwarn.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/listwarn.rs`

  - JS: `✅ @␀ tidak memiliki warning.`
    Rust: `tidak memiliki warning.`

### notifclosegroup (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/notifclosegroup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/notifclosegroup.rs`

  - JS: ``␀notifclosegroup off` - Nonaktifkan`
    Rust: ``.notifclosegroup off` - Nonaktifkan`
  - JS: ``␀notifclosegroup on` - Aktifkan`
    Rust: ``.notifclosegroup on` - Aktifkan`

### notifdemote (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/notifdemote.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/notifdemote.rs`

  - JS: `*Penggunaan:*`
    Rust: (tidak ada padanan)
  - JS: `> Status: ␀`
    Rust: (tidak ada padanan)
  - JS: ``␀notifdemote off` - Nonaktifkan`
    Rust: ``.notifdemote on` - Aktifkan`
  - JS: ``␀notifdemote on` - Aktifkan`
    Rust: ``.notifdemote on` - Aktifkan`
  - JS: `✅ Aktif`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya admin grup yang bisa menggunakan fitur ini`
    Rust: (tidak ada padanan)
  - JS: `❌ Nonaktif`
    Rust: (tidak ada padanan)

### notifgantitag (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/notifgantitag.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/notifgantitag.rs`

  - JS: `> Notifikasi ganti label diaktifkan di *␀* grup!`
    Rust: `> Notifikasi ganti label sudah aktif di grup ini.`
  - JS: `> Notifikasi ganti label dinonaktifkan di *␀* grup!`
    Rust: `> Notifikasi ganti label sudah nonaktif di grup ini.`
  - JS: `Hei @␀, Tag kamu mengandung kata toxic !`
    Rust: (tidak ada padanan)
  - JS: `[NotifLabelChange] Error:`
    Rust: (tidak ada padanan)
  - JS: `✅ *ɴᴏᴛɪꜰ ʟᴀʙᴇʟ ɢʟᴏʙᴀʟ ᴏɴ*`
    Rust: `⚠️ *ɴᴏᴛɪꜰ ʟᴀʙᴇʟ ɢʟᴏʙᴀʟ ᴛɪᴅᴀᴋ ᴅɪᴘᴏʀᴛ*`
  - JS: `❌ *ɴᴏᴛɪꜰ ʟᴀʙᴇʟ ɢʟᴏʙᴀʟ ᴏꜰꜰ*`
    Rust: `❌ *ɴᴏᴛɪꜰ ʟᴀʙᴇʟ ɴᴏɴᴀᴋᴛɪꜰ*`
  - JS: `🎉 @␀ telah mengubah label menjadi *␀*`
    Rust: (tidak ada padanan)
  - JS: `🥗 @␀ telah menghapus label`
    Rust: (tidak ada padanan)

### notifopengroup (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/notifopengroup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/notifopengroup.rs`

  - JS: `*Penggunaan:*`
    Rust: (tidak ada padanan)
  - JS: `> Status: ␀`
    Rust: (tidak ada padanan)
  - JS: ``␀notifopengroup off` - Nonaktifkan`
    Rust: ``.notifopengroup off` - Nonaktifkan`
  - JS: ``␀notifopengroup on` - Aktifkan`
    Rust: ``.notifopengroup off` - Nonaktifkan`
  - JS: `✅ Aktif`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya admin grup yang bisa menggunakan fitur ini`
    Rust: (tidak ada padanan)
  - JS: `❌ Nonaktif`
    Rust: (tidak ada padanan)

### notifpromote (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/notifpromote.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/notifpromote.rs`

  - JS: `*Penggunaan:*`
    Rust: (tidak ada padanan)
  - JS: `> Status: ␀`
    Rust: (tidak ada padanan)
  - JS: ``␀notifpromote off` - Nonaktifkan`
    Rust: (tidak ada padanan)
  - JS: ``␀notifpromote on` - Aktifkan`
    Rust: (tidak ada padanan)
  - JS: `✅ Aktif`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya admin grup yang bisa menggunakan fitur ini`
    Rust: (tidak ada padanan)
  - JS: `❌ Nonaktif`
    Rust: (tidak ada padanan)

### notifsholat (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/notifsholat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/notifsholat.rs`

  - JS: `❌ Hanya admin grup yang bisa menggunakan fitur ini`
    Rust: (tidak ada padanan)

### openvo (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/openvo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/openvo.rs`

  - JS: `> Balas pesan dengan ikon 1x lihat (👁️).`
    Rust: (tidak ada padanan)
  - JS: `> Gagal membuka pesan 1x lihat.`
    Rust: (tidak ada padanan)
  - JS: `> Konten pesan tidak dapat dibaca.`
    Rust: (tidak ada padanan)
  - JS: `> Media mungkin sudah kadaluarsa.`
    Rust: (tidak ada padanan)
  - JS: `> Pesan yang di-reply bukan pesan 1x lihat!`
    Rust: (tidak ada padanan)
  - JS: `> Tidak dapat membaca pesan yang di-reply.`
    Rust: (tidak ada padanan)
  - JS: `> Tidak dapat mengunduh media.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ ᴍᴇɴɢᴜɴᴅᴜʜ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ʙᴜᴋᴀɴ ᴠɪᴇᴡᴏɴᴄᴇ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴇʀʀᴏʀ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴋᴏɴᴛᴇɴ ᴛɪᴅᴀᴋ ᴅɪᴛᴇᴍᴜᴋᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴘᴇsᴀɴ ᴛɪᴅᴀᴋ ᴅɪᴛᴇᴍᴜᴋᴀɴ*`
    Rust: (tidak ada padanan)

### gcpinchat (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/pinchat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/pinchat.rs`

  - JS: `> Gagal mem-pin pesan.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ Success pin pesan ini`
    Rust: `Success pin pesan ini`
  - JS: `❌ *ᴇʀʀᴏʀ*`
    Rust: (tidak ada padanan)

### poll (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/poll.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/poll.rs`

  - JS: `> Gagal membuat poll.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴇʀʀᴏʀ*`
    Rust: (tidak ada padanan)

### promote (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/promote.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/promote.rs`

  - JS: `User tidak ditemukan`
    Rust: `> User tidak ditemukan di grup!`
  - JS: `✅ @␀ sekarang menjadi admin!`
    Rust: `sekarang menjadi admin`

### setdeskgc (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/setdeskgc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/setdeskgc.rs`

  - JS: `> Tidak dapat mengubah deskripsi grup.`
    Rust: `Mengubah deskripsi grup`
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ Deskripsi grup berhasil dihapus!`
    Rust: (tidak ada padanan)
  - JS: `✅ Deskripsi grup berhasil diperbarui!`
    Rust: `deskripsi berhasil diperbarui`
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)

### setgoodbye (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/setgoodbye.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/setgoodbye.rs`

  - JS: `.setgoodbye Bye {user}, sampai jumpa lagi!`
    Rust: `.setgoodbye Bye ␀, sampai jumpa lagi!`
  - JS: ``Sampai jumpa lagi pada {day}, {date}``
    Rust: ``Sampai jumpa lagi pada {␀}, {␀}``
  - JS: ``␀setgoodbye Bye {user}! 👋``
    Rust: ``␀setgoodbye Bye {␀}! 👋``
  - JS: `┃ ◦ `{bot}` - Nama bot`
    Rust: `┃ ◦ `{␀}` - Nama bot`
  - JS: `┃ ◦ `{count}` - Sisa member`
    Rust: `┃ ◦ `{␀}` - Sisa member`
  - JS: `┃ ◦ `{date}` - Tanggal (DD/MM/YYYY)`
    Rust: `┃ ◦ `{␀}` - Tanggal (DD/MM/YYYY)`
  - JS: `┃ ◦ `{day}` - Hari (Senin, Selasa, dll)`
    Rust: `┃ ◦ `{␀}` - Hari (Senin, Selasa, dll)`
  - JS: `┃ ◦ `{desc}` - Deskripsi grup`
    Rust: `┃ ◦ `{␀}` - Deskripsi grup`
  - JS: `┃ ◦ `{group}` - Nama grup`
    Rust: `┃ ◦ `{␀}` - Nama grup`
  - JS: `┃ ◦ `{number}` - Nomor member`
    Rust: `┃ ◦ `{␀}` - Nomor member`
  - JS: `┃ ◦ `{owner}` - Nama owner grup`
    Rust: `┃ ◦ `{␀}` - Nama owner grup`
  - JS: `┃ ◦ `{prefix}` - Prefix bot`
    Rust: `┃ ◦ `{␀}` - Prefix bot`
  - JS: `┃ ◦ `{time}` - Waktu (HH:mm WIB)`
    Rust: `┃ ◦ `{␀}` - Waktu (HH:mm WIB)`
  - JS: `┃ ◦ `{user}` - Nama member`
    Rust: `┃ ◦ `{␀}` - Nama member`

### setintro (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/setintro.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/setintro.rs`

  - JS: `> .setintro Selamat datang @user di grup @group! 👋`
    Rust: `> ␀setintro Selamat datang @user di grup @group! 👋`

### setnamegc (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/setnamegc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/setnamegc.rs`

  - JS: `> Tidak dapat mengubah nama grup.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil mengubah nama grup menjadi *␀*`
    Rust: `berhasil mengubah nama grup`
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)

### setrulesgrup (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/setrulesgrup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/setrulesgrup.rs`

  - JS: `✅ *ɢʀᴜᴘ ʀᴜʟᴇs ᴅɪᴜᴘᴅᴀᴛᴇ*`
    Rust: `✅ *ɢʀᴜᴘ ʀᴜʟᴇs ᴅɪʜᴍᴘᴀᴛᴇ*`

### setwelcome (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/setwelcome.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/setwelcome.rs`

  - JS: `.setwelcome Halo {user}, selamat datang di {group}!`
    Rust: `.setwelcome Halo ␀, selamat datang di ␀!`
  - JS: ``Selamat datang di {group} pada {day}, {date}``
    Rust: ``Selamat datang di {␀} pada {␀}, {␀}``
  - JS: ``␀setwelcome Halo {user}! 👋``
    Rust: ``␀setwelcome Halo {␀}! 👋``
  - JS: `┃ ◦ `{bot}` - Nama bot`
    Rust: `┃ ◦ `{␀}` - Nama bot`
  - JS: `┃ ◦ `{count}` - Jumlah member`
    Rust: `┃ ◦ `{␀}` - Jumlah member`
  - JS: `┃ ◦ `{date}` - Tanggal (DD/MM/YYYY)`
    Rust: `┃ ◦ `{␀}` - Tanggal (DD/MM/YYYY)`
  - JS: `┃ ◦ `{day}` - Hari (Senin, Selasa, dll)`
    Rust: `┃ ◦ `{␀}` - Hari (Senin, Selasa, dll)`
  - JS: `┃ ◦ `{desc}` - Deskripsi grup`
    Rust: `┃ ◦ `{␀}` - Deskripsi grup`
  - JS: `┃ ◦ `{group}` - Nama grup`
    Rust: `┃ ◦ `{␀}` - Nama grup`
  - JS: `┃ ◦ `{number}` - Nomor member`
    Rust: `┃ ◦ `{␀}` - Nomor member`
  - JS: `┃ ◦ `{owner}` - Nama owner grup`
    Rust: `┃ ◦ `{␀}` - Nama owner grup`
  - JS: `┃ ◦ `{prefix}` - Prefix bot`
    Rust: `┃ ◦ `{␀}` - Prefix bot`
  - JS: `┃ ◦ `{time}` - Waktu (HH:mm WIB)`
    Rust: `┃ ◦ `{␀}` - Waktu (HH:mm WIB)`
  - JS: `┃ ◦ `{user}` - Nama member`
    Rust: `┃ ◦ `{␀}` - Nama member`

### slowmode (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/slowmode.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/slowmode.rs`

  - JS: `*all* — hapus semua pesan saat delay`
    Rust: `x20x20*all* — hapus semua pesan saat delay`
  - JS: `*onlycommand* — silent command, chat bebas`
    Rust: `x20x20*onlycommand* — silent command, chat bebas`
  - JS: `Status: ␀s)` : '❌ OFF'}`
    Rust: `Status: ❌ OFF`
  - JS: `␀_␀`
    Rust: (tidak ada padanan)

### tam (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/tam.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/tam.rs`

  - JS: `␀. ␀␀`
    Rust: `␀. ␀`

### warn (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/warn.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/warn.rs`

  - JS: `> Alasan Terakhir: *␀*`
    Rust: `> Alasan: *␀*`
  - JS: `> Warning: *␀/␀*`
    Rust: `> Warning ke: *␀/␀*`
  - JS: `❌ Gak usah warn aku, aku cuma bot.`
    Rust: (tidak ada padanan)
  - JS: `🚨 *MAX WARNING TERCAPAI*`
    Rust: (tidak ada padanan)

### welcome (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/group/welcome.js`
- Rust: `/workspaces/oktz-rs/plugin/src/group/welcome.rs`

  - JS: `*Halo* @␀ 👋`
    Rust: (tidak ada padanan)
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Welcome diaktifkan di *␀* grup!`
    Rust: (tidak ada padanan)
  - JS: `> Welcome dinonaktifkan di *␀* grup!`
    Rust: `> Welcome sudah nonaktif di grup ini.`
  - JS: `> ❝ ␀␀ ❞`
    Rust: (tidak ada padanan)
  - JS: `> 「 ␀ 」`
    Rust: (tidak ada padanan)
  - JS: `> 🏠 *Nama* : ␀`
    Rust: (tidak ada padanan)
  - JS: `> 👥 *Member* : ␀`
    Rust: (tidak ada padanan)
  - JS: `> 📅 *Tanggal* : ␀`
    Rust: (tidak ada padanan)
  - JS: `Akhirnya datang juga`
    Rust: (tidak ada padanan)
  - JS: `Grup ini nambah satu keluarga lagi 🤍`
    Rust: (tidak ada padanan)
  - JS: `HH:mm`
    Rust: (tidak ada padanan)
  - JS: `Hai @␀, welcome to grup ␀`
    Rust: (tidak ada padanan)
  - JS: `Halo semuanya! 👋`
    Rust: (tidak ada padanan)
  - JS: `Hari ini kita kedatangan tomodachi baru 🌱`
    Rust: (tidak ada padanan)
  - JS: `Jangan jadi silent reader ya!`
    Rust: (tidak ada padanan)
  - JS: `Jangan malu-malu, kita semua temen!`
    Rust: (tidak ada padanan)
  - JS: `Kalau bingung mulai, nyapa aja dulu 😄`
    Rust: (tidak ada padanan)
  - JS: `Member ke-␀`
    Rust: (tidak ada padanan)
  - JS: `Ohayou~`
    Rust: (tidak ada padanan)
  - JS: `Santai aja, anggap rumah sendiri!`
    Rust: (tidak ada padanan)
  - JS: `Satu teman baru akhirnya join ✨`
    Rust: (tidak ada padanan)
  - JS: `Selamat datang`
    Rust: (tidak ada padanan)
  - JS: `Selamat datang di grup *␀* 🌸`
    Rust: (tidak ada padanan)
  - JS: `Selamat datang di grup *␀* 🎉`
    Rust: (tidak ada padanan)
  - JS: `Semoga betah dan langsung nimbrung ya~`
    Rust: (tidak ada padanan)
  - JS: `Siap-siap rame bareng!`
    Rust: (tidak ada padanan)
  - JS: `Tanoshii jikan o issho ni sugoso ne~`
    Rust: (tidak ada padanan)
  - JS: `Tomodachi baru datang bawa vibes baru 💫`
    Rust: (tidak ada padanan)
  - JS: `Welcome Error:`
    Rust: (tidak ada padanan)
  - JS: `Welcome to ␀`
    Rust: (tidak ada padanan)
  - JS: `Yokoso~`
    Rust: (tidak ada padanan)
  - JS: `Yoroshiku ne~ mari seru-seruan bareng!`
    Rust: (tidak ada padanan)
  - JS: `Yuk langsung gas ngobrol!`
    Rust: (tidak ada padanan)
  - JS: `Yuk sambut bareng-bareng~`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀, *@␀* 💫`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴡᴇʟᴄᴏᴍᴇ ɢʟᴏʙᴀʟ ᴏɴ*`
    Rust: (tidak ada padanan)
  - JS: `✨ *Tips Hari Ini*`
    Rust: (tidak ada padanan)
  - JS: `✨ Ohayou~!`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴡᴇʟᴄᴏᴍᴇ ɢʟᴏʙᴀʟ ᴏꜰꜰ*`
    Rust: (tidak ada padanan)
  - JS: `🌸 Ohayou minna-san!`
    Rust: (tidak ada padanan)
  - JS: `🌸 _Yoroshiku ne~ semoga betah ya!_ 🤍`
    Rust: (tidak ada padanan)
  - JS: `🎐 Ohayou~ minna-san!`
    Rust: (tidak ada padanan)
  - JS: `👋 Halo`
    Rust: (tidak ada padanan)
  - JS: `👋 Halo @`
    Rust: (tidak ada padanan)
  - JS: `👋 Halo ␀!`
    Rust: (tidak ada padanan)
  - JS: `👋 Welcome *@␀*`
    Rust: (tidak ada padanan)
  - JS: `👋🏻 *WELCOME MEMBER BARU* 👋🏻`
    Rust: (tidak ada padanan)
  - JS: `📌 *INFO GROUP*`
    Rust: (tidak ada padanan)
  - JS: `📝 *Deskripsi*`
    Rust: (tidak ada padanan)
  - JS: `🪸 Ohayou minna-san!`
    Rust: (tidak ada padanan)

### antara (berita)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/berita.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/berita.rs`

  - JS: `*␀. ␀*`
    Rust: (tidak ada padanan)
  - JS: `.berita <sumber>`
    Rust: (tidak ada padanan)
  - JS: `.kompas atau .berita cnn`
    Rust: (tidak ada padanan)
  - JS: `2-digit`
    Rust: (tidak ada padanan)
  - JS: `> Gunakan: `␀berita` untuk melihat daftar.`
    Rust: (tidak ada padanan)
  - JS: `Mendapatkan berita terkini dari berbagai sumber`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `_Total: ␀ artikel tersedia_`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀*`
    Rust: (tidak ada padanan)
  - JS: `␀...`
    Rust: (tidak ada padanan)
  - JS: `━━━━━━━━━━━━━━━`
    Rust: (tidak ada padanan)
  - JS: `❌ Sumber berita tidak ditemukan.`
    Rust: (tidak ada padanan)
  - JS: `❌ Sumber berita tidak valid.`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada berita ditemukan.`
    Rust: (tidak ada padanan)
  - JS: `📅 _␀_`
    Rust: (tidak ada padanan)
  - JS: `🔗 ␀`
    Rust: (tidak ada padanan)

### bluearchive-char (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/bluearchive-char.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/bluearchive-char.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Mungkin maksud: ␀`
    Rust: (tidak ada padanan)
  - JS: `Character "␀" tidak ditemukan.`
    Rust: `Character … tidak ditemukan`

### gag (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/gag.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/gag.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[GAG Info]`
    Rust: (tidak ada padanan)

### gag2 (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/gag2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/gag2.rs`

  - JS: `- ␀ *␀ × ␀*`
    Rust: `- ␀ *␀ (␀)*:`
  - JS: `- ⚙️ *GEAR (␀):*`
    Rust: (tidak ada padanan)
  - JS: `- 🌱 *SEEDS (␀):*`
    Rust: (tidak ada padanan)
  - JS: `- 📦 *CRATES (␀):*`
    Rust: (tidak ada padanan)
  - JS: `> Max 5 iterations (~2.5 menit). Re-run buat extend.`
    Rust: (tidak ada padanan)
  - JS: `[Gag2]`
    Rust: (tidak ada padanan)
  - JS: `⏱️ 5 iterations done, no match. Re-run buat lanjut.`
    Rust: (tidak ada padanan)
  - JS: `🎯 *[iter ␀] FOUND ␀ watched item(s)!*`
    Rust: (tidak ada padanan)
  - JS: `👀 *Watching for:* ␀`
    Rust: (tidak ada padanan)

### gag2watch (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/gagwatch.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/gagwatch.rs`

  - JS: `[GAG-WATCH Plugin Error]`
    Rust: (tidak ada padanan)

### harilibur (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/harilibur.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/harilibur.rs`

  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64)`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[Hari Libur]`
    Rust: (tidak ada padanan)

### howmuchenergy (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/howmuchenergy.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/howmuchenergy.rs`

  - JS: `- *␀howmuchenergy <nama_fitur1> <nama_fitur2> ...*`
    Rust: `- *.howmuchenergy <nama_fitur1> <nama_fitur2> ...*`
  - JS: `- *␀howmuchenergy hd jpm*`
    Rust: `- *.howmuchenergy hd jpm*`
  - JS: `❌ *␀* : Tidak ditemukan!`
    Rust: `❌ *tidak-ada* : Tidak ditemukan!`

### infotourney (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/infotourney.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/infotourney.rs`

  - JS: `.items-row .item`
    Rust: (tidak ada padanan)
  - JS: `<br>`
    Rust: (tidak ada padanan)
  - JS: `h2[itemprop="name"] a`
    Rust: (tidak ada padanan)
  - JS: `p img`
    Rust: (tidak ada padanan)
  - JS: `time[itemprop="datePublished"]`
    Rust: (tidak ada padanan)

### jadwalbola (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/jadwalbola.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/jadwalbola.rs`

  - JS: `.jadwalbola [liga]`
    Rust: (tidak ada padanan)
  - JS: `.jadwalbola inggris`
    Rust: (tidak ada padanan)
  - JS: `> Filter: `␀``
    Rust: (tidak ada padanan)
  - JS: `Lihat jadwal pertandingan sepak bola`
    Rust: (tidak ada padanan)
  - JS: `Tidak ada jadwal tersedia`
    Rust: (tidak ada padanan)
  - JS: `Total: *␀* pertandingan`
    Rust: (tidak ada padanan)
  - JS: `bri super league`
    Rust: (tidak ada padanan)
  - JS: `la liga spanyol`
    Rust: (tidak ada padanan)
  - JS: `liga belanda`
    Rust: (tidak ada padanan)
  - JS: `liga champions`
    Rust: (tidak ada padanan)
  - JS: `liga inggris`
    Rust: (tidak ada padanan)
  - JS: `liga italia`
    Rust: (tidak ada padanan)
  - JS: `liga jerman`
    Rust: (tidak ada padanan)
  - JS: `liga prancis`
    Rust: (tidak ada padanan)
  - JS: `liga spanyol`
    Rust: (tidak ada padanan)
  - JS: `⏰ ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀*`
    Rust: (tidak ada padanan)
  - JS: `⚽ *ᴊᴀᴅᴡᴀʟ ᴘᴇʀᴛᴀɴᴅɪɴɢᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ditemukan jadwal untuk: `␀``
    Rust: (tidak ada padanan)
  - JS: `🆚 ␀`
    Rust: (tidak ada padanan)
  - JS: `🇩🇪`
    Rust: (tidak ada padanan)
  - JS: `🇪🇸`
    Rust: (tidak ada padanan)
  - JS: `🇫🇷`
    Rust: (tidak ada padanan)
  - JS: `🇮🇩`
    Rust: (tidak ada padanan)
  - JS: `🇮🇹`
    Rust: (tidak ada padanan)
  - JS: `🇳🇱`
    Rust: (tidak ada padanan)
  - JS: `🏠 ␀`
    Rust: (tidak ada padanan)
  - JS: `🏴󠁧󠁢󠁥󠁮󠁧󠁿`
    Rust: (tidak ada padanan)
  - JS: `📅 *␀*`
    Rust: (tidak ada padanan)

### livescore (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/livescore.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/livescore.rs`

  - JS: `>(.*?)</script>/s);`
    Rust: `>(.*?)</script>/s` — penanda`
  - JS: `CANCELLED: 'Dibatalkan',`
    Rust: (tidak ada padanan)
  - JS: `FINISHED: 'Selesai',`
    Rust: (tidak ada padanan)
  - JS: `FIXTURE: 'Belum Mulai',`
    Rust: (tidak ada padanan)
  - JS: `HALF_TIME: 'Turun Minum'`
    Rust: (tidak ada padanan)
  - JS: `LIVE: 'Berlangsung',`
    Rust: (tidak ada padanan)
  - JS: `Maaf, sistem gagal mengambil data live score saat ini. Error: _␀_`);`
    Rust: `Maaf, sistem gagal mengambil data live score saat ini. Error: _␀_`
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `POSTPONED: 'Ditunda',`
    Rust: (tidak ada padanan)
  - JS: `Saat ini tidak ada pertandingan yang sedang berlangsung atau dijadwalkan.`);`
    Rust: `Saat ini tidak ada pertandingan yang sedang berlangsung atau dijadwalkan.`
  - JS: ``;`
    Rust: (tidak ada padanan)
  - JS: `accept-language`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m) {`
    Rust: (tidak ada padanan)
  - JS: `await m.react('✅');`
    Rust: (tidak ada padanan)
  - JS: `await m.react('❌');`
    Rust: (tidak ada padanan)
  - JS: `await m.react('🕕');`
    Rust: (tidak ada padanan)
  - JS: `await m.reply(caption.trim());`
    Rust: (tidak ada padanan)
  - JS: `caption += ``
    Rust: (tidak ada padanan)
  - JS: `caption += `   └ ⏳ _Status: ␀_`;`
    Rust: (tidak ada padanan)
  - JS: `caption += `_Sumber: Goal.com_`;`
    Rust: `_Sumber: Goal.com_`
  - JS: `caption += `▪️ ␀␀ *[ ␀ - ␀ ]* ␀␀`
    Rust: `▪️ ␀␀ *[ ␀ - ␀ ]* ␀␀`
  - JS: `caption += `🏆 *␀*`
    Rust: (tidak ada padanan)
  - JS: `console.error(e);`
    Rust: (tidak ada padanan)
  - JS: `const data = await ambilLivescore('id');`
    Rust: (tidak ada padanan)
  - JS: `const json = JSON.parse(cocok[1]);`
    Rust: (tidak ada padanan)
  - JS: `const liveScores = json?.props?.pageProps?.content?.liveScores;`
    Rust: (tidak ada padanan)
  - JS: `const map = {`
    Rust: (tidak ada padanan)
  - JS: `const rapi = _rapikanPertandingan(data);`
    Rust: (tidak ada padanan)
  - JS: `const skorA = p.skorTuanRumah;`
    Rust: (tidak ada padanan)
  - JS: `const skorB = p.skorTandang;`
    Rust: (tidak ada padanan)
  - JS: `count++;`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `for (const grup of rapi) {`
    Rust: (tidak ada padanan)
  - JS: `for (const p of grup.pertandingan) {`
    Rust: (tidak ada padanan)
  - JS: `function _rapikanPertandingan(dataMentah) {`
    Rust: (tidak ada padanan)
  - JS: `function _statusIndo(status) {`
    Rust: (tidak ada padanan)
  - JS: `id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7`
    Rust: (tidak ada padanan)
  - JS: `if (!cocok) throw new Error('Gagal menemukan data skor');`
    Rust: (tidak ada padanan)
  - JS: `if (!liveScores) throw new Error('Data liveScores kosong');`
    Rust: (tidak ada padanan)
  - JS: `if (!rapi || rapi.length === 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (count >= 15) break;`
    Rust: (tidak ada padanan)
  - JS: `if (grup.pertandingan.length === 0) continue;`
    Rust: (tidak ada padanan)
  - JS: `if (p.periode) caption += ` | ⏱️ _␀_`;`
    Rust: (tidak ada padanan)
  - JS: `kartuMerahTandang: m.redCards?.teamB ?? 0,`
    Rust: (tidak ada padanan)
  - JS: `kartuMerahTuanRumah: m.redCards?.teamA ?? 0,`
    Rust: (tidak ada padanan)
  - JS: `kompetisi: grup.competition?.name || '-',`
    Rust: (tidak ada padanan)
  - JS: `let caption = `⚽ *LIVE SCORE BOLA HARI INI* ⚽`
    Rust: `⚽ *LIVE SCORE BOLA HARI INI* ⚽`
  - JS: `let count = 0;`
    Rust: (tidak ada padanan)
  - JS: `let kmA = p.kartuMerahTuanRumah > 0 ? ` 🟥␀` : '';`
    Rust: (tidak ada padanan)
  - JS: `let kmB = p.kartuMerahTandang > 0 ? ` 🟥␀` : '';`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`❌ *GAGAL MENGAMBIL SKOR*`
    Rust: `❌ *GAGAL MENGAMBIL SKOR*`
  - JS: `periode: (typeof m.period === 'object' && m.period !== null) ? (m.period.name ?? m.period.short ?? m.period.long ?? '-') : (m.period || '')`
    Rust: (tidak ada padanan)
  - JS: `pertandingan: (grup.matches || []).map(m => ({`
    Rust: (tidak ada padanan)
  - JS: `return dataMentah.map(grup => ({`
    Rust: (tidak ada padanan)
  - JS: `return liveScores;`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`⚽ *LIVESCORE BOLA* ⚽`
    Rust: (tidak ada padanan)
  - JS: `return map[status] || status;`
    Rust: (tidak ada padanan)
  - JS: `skorTandang: (typeof m.score?.teamB === 'object' && m.score?.teamB !== null) ? (m.score.teamB.current ?? m.score.teamB.normalTime ?? m.score.teamB.score ?? '-') : (m.score?.teamB ?? '-'),`
    Rust: (tidak ada padanan)
  - JS: `skorTuanRumah: (typeof m.score?.teamA === 'object' && m.score?.teamA !== null) ? (m.score.teamA.current ?? m.score.teamA.normalTime ?? m.score.teamA.score ?? '-') : (m.score?.teamA ?? '-'),`
    Rust: (tidak ada padanan)
  - JS: `status: _statusIndo(m.status),`
    Rust: (tidak ada padanan)
  - JS: `tandang: m.teamB?.name,`
    Rust: (tidak ada padanan)
  - JS: `throw new Error(error.message, { cause: error });`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `tuanRumah: m.teamA?.name,`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)
  - JS: `waktu: m.startDate,`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (error) {`
    Rust: (tidak ada padanan)
  - JS: `}))`
    Rust: (tidak ada padanan)
  - JS: `}));`
    Rust: (tidak ada padanan)
  - JS: `};`
    Rust: (tidak ada padanan)

### whatrolethis (info)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/info/whatrolethis.js`
- Rust: `/workspaces/oktz-rs/plugin/src/info/whatrolethis.rs`

  - JS: `- *␀whatrolethis <nama_fitur1> <nama_fitur2> ...*`
    Rust: `- *.whatrolethis <nama_fitur1> <nama_fitur2> ...*`
  - JS: `- *␀whatrolethis hd jpm warn*`
    Rust: `- *.whatrolethis hd jpm warn*`
  - JS: `.whatrolethis hd jpm`
    Rust: `.whatrolethis hd jpm warn`
  - JS: `❌ *␀* : Tidak ditemukan!`
    Rust: `❌ *tidak-ada* : Tidak ditemukan!`

### murrotal (islamic)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/islamic/murrotal.js`
- Rust: `/workspaces/oktz-rs/plugin/src/islamic/murrotal.rs`

  - JS: `.murrotal <nama surah>`
    Rust: (tidak ada padanan)
  - JS: `.murrotal al fatihah`
    Rust: (tidak ada padanan)
  - JS: `.surah-item`
    Rust: `surah-item`
  - JS: `> Masukkan nama surah`
    Rust: (tidak ada padanan)
  - JS: `Dengarkan audio murottal Al-Quran berdasarkan surah`
    Rust: (tidak ada padanan)
  - JS: ``Contoh: ␀murrotal al fatihah``
    Rust: (tidak ada padanan)
  - JS: ``Contoh: ␀murrotal ar rahman``
    Rust: (tidak ada padanan)
  - JS: `❌ Surah *␀* tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `🎧 *MURROTTAL*`
    Rust: (tidak ada padanan)

### quran (islamic)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/islamic/quran.js`
- Rust: `/workspaces/oktz-rs/plugin/src/islamic/quran.rs`

  - JS: `.text-neutral-700`
    Rust: `text-neutral-700`
  - JS: `.text-primary-500`
    Rust: `text-primary-500`
  - JS: `[dir="rtl"]`
    Rust: (tidak ada padanan)
  - JS: `div[id]`
    Rust: (tidak ada padanan)

### allmenu (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/allmenu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/allmenu.rs`

  - JS: `-b:a`
    Rust: (tidak ada padanan)
  - JS: `-c:a`
    Rust: (tidak ada padanan)
  - JS: `-i`
    Rust: (tidak ada padanan)
  - JS: `-vbr`
    Rust: (tidak ada padanan)
  - JS: `-y`
    Rust: (tidak ada padanan)
  - JS: `120363351980387532@newsletter`
    Rust: `120363400911374213@newsletter`
  - JS: `44444444444444`
    Rust: (tidak ada padanan)
  - JS: `48k`
    Rust: (tidak ada padanan)
  - JS: `Bot Channel`
    Rust: (tidak ada padanan)
  - JS: `Dibuat oleh`
    Rust: `Dibuat oleh ␀`
  - JS: `FFmpeg error`
    Rust: (tidak ada padanan)
  - JS: `[AllMenu] Error sending dynamic audio:`
    Rust: (tidak ada padanan)
  - JS: `[AllMenu] Error:`
    Rust: (tidak ada padanan)
  - JS: `allmenu_audio.mp3`
    Rust: (tidak ada padanan)
  - JS: `allmenu_audio_opus.ogg`
    Rust: (tidak ada padanan)
  - JS: `audio/ogg; codecs=opus`
    Rust: (tidak ada padanan)
  - JS: `foto-menu-v8`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `setelin musiknya nya bang`
    Rust: (tidak ada padanan)
  - JS: `status@broadcast`
    Rust: (tidak ada padanan)
  - JS: `␀ | 🌡️ ␀°C`
    Rust: (tidak ada padanan)
  - JS: `␀@s.whatsapp.net`
    Rust: `0@s.whatsapp.net`
  - JS: `☀️ Cerah`
    Rust: (tidak ada padanan)
  - JS: `☁️ Mendung`
    Rust: (tidak ada padanan)
  - JS: `★ ␀`
    Rust: (tidak ada padanan)
  - JS: `⛅ Berawan`
    Rust: (tidak ada padanan)
  - JS: `⛈️ Badai Petir`
    Rust: (tidak ada padanan)
  - JS: `⛈️ Hujan Lebat`
    Rust: (tidak ada padanan)
  - JS: `🌍 Tidak diketahui`
    Rust: (tidak ada padanan)
  - JS: `🌤️ Cerah Berawan`
    Rust: (tidak ada padanan)
  - JS: `🌦️ Gerimis`
    Rust: (tidak ada padanan)
  - JS: `🌦️ Hujan Lokal`
    Rust: (tidak ada padanan)
  - JS: `🌧️ Hujan`
    Rust: (tidak ada padanan)
  - JS: `🌧️ Hujan Ringan`
    Rust: (tidak ada padanan)
  - JS: `🌫️ Berkabut`
    Rust: (tidak ada padanan)
  - JS: `🌫️ Kabut Tebal`
    Rust: (tidak ada padanan)
  - JS: `📋 ␀ Commands`
    Rust: (tidak ada padanan)
  - JS: `📍 ␀`
    Rust: (tidak ada padanan)

### benefitowner (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/benefitowner.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/benefitowner.rs`

  - JS: `• *␀␀*`
    Rust: (tidak ada padanan)

### benefitpremium (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/benefitpremium.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/benefitpremium.rs`

  - JS: `') }`
    Rust: (tidak ada padanan)
  - JS: `• *␀␀*`
    Rust: (tidak ada padanan)
  - JS: `␀`).join('`
    Rust: (tidak ada padanan)

### donasi (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/donasi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/donasi.rs`

  - JS: `120363400911374213@newsletter`
    Rust: `120363411617217441@newsletter`
  - JS: `␀@s.whatsapp.net`
    Rust: `628111@s.whatsapp.net`
  - JS: `📋 Copy No. ␀`
    Rust: (tidak ada padanan)

### gacha (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/gacha.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/gacha.rs`

  - JS: `*Hadiah:* ␀ ␀`
    Rust: `*Hadiah:* 💰`
  - JS: `+␀ EXP`
    Rust: (tidak ada padanan)
  - JS: `+␀ Energi`
    Rust: (tidak ada padanan)
  - JS: `+␀ Koin`
    Rust: (tidak ada padanan)

### leaderboard (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/leaderboard.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/leaderboard.rs`

  - JS: `*(You)*`
    Rust: (tidak ada padanan)
  - JS: `120`
    Rust: (tidak ada padanan)
  - JS: `4️⃣`
    Rust: (tidak ada padanan)
  - JS: `5️⃣`
    Rust: (tidak ada padanan)
  - JS: `6️⃣`
    Rust: (tidak ada padanan)
  - JS: `7️⃣`
    Rust: (tidak ada padanan)
  - JS: `8️⃣`
    Rust: (tidak ada padanan)
  - JS: `9️⃣`
    Rust: (tidak ada padanan)
  - JS: `> Belum ada data user terdaftar di database.`
    Rust: (tidak ada padanan)
  - JS: `> Kamu belum terdaftar di database.`
    Rust: (tidak ada padanan)
  - JS: `> Posisi kamu: *#␀* dari *␀* user.`
    Rust: (tidak ada padanan)
  - JS: `Lv. ␀ (␀ XP)`
    Rust: (tidak ada padanan)
  - JS: `Peringkat para penguasa tertinggi saat ini!`
    Rust: (tidak ada padanan)
  - JS: `Rp ␀`
    Rust: (tidak ada padanan)
  - JS: `TOP GLOBAL ENERGI`
    Rust: (tidak ada padanan)
  - JS: `TOP GLOBAL KOIN`
    Rust: (tidak ada padanan)
  - JS: `TOP GLOBAL LEVEL`
    Rust: (tidak ada padanan)
  - JS: `_Pilih tombol di bawah untuk melihat ranking!_`
    Rust: (tidak ada padanan)
  - JS: `␀ Energi`
    Rust: (tidak ada padanan)
  - JS: `␀.`
    Rust: (tidak ada padanan)
  - JS: `␀topenergi`
    Rust: (tidak ada padanan)
  - JS: `␀topexp`
    Rust: (tidak ada padanan)
  - JS: `␀topkoin`
    Rust: `.topkoin`
  - JS: `┃    └ ␀ (␀%)`
    Rust: (tidak ada padanan)
  - JS: `┃ ␀ @␀␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ␀ *RANKING* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `⚡ Top Energi`
    Rust: (tidak ada padanan)
  - JS: `✨ Top EXP`
    Rust: (tidak ada padanan)
  - JS: `🏆 *LEADERBOARD OVERVIEW* 🏆`
    Rust: (tidak ada padanan)
  - JS: `🏆 *␀* 🏆`
    Rust: (tidak ada padanan)
  - JS: `💰 Top Koin`
    Rust: (tidak ada padanan)
  - JS: `📊 *ʟᴇᴀᴅᴇʀʙᴏᴀʀᴅ*`
    Rust: `🏆 *Lᴇᴀᴅᴇʀʙᴏᴀʀᴅ*`

### menu (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/menu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/menu.rs`

  - JS: `*␀ ␀*, 𝘔𝘺 𝘯𝘢𝘮𝘦 𝘪𝘴 ␀.`
    Rust: `*␀ ␀, 𝘔𝘺 𝘯𝘢𝘮𝘦 𝘪𝘴 ␀.`
  - JS: `0@s.whatsapp.net`
    Rust: `628111@s.whatsapp.net`
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `30°C`
    Rust: (tidak ada padanan)
  - JS: `9999999`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: `> ␀␀`
  - JS: `> 🥩 *Energi*: ␀`
    Rust: `> 🍩 *Energi*: ␀`
  - JS: `> 🦴 *Mode*: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Dibuat oleh`
    Rust: (tidak ada padanan)
  - JS: `Don't forget to smile today 😊`
    Rust: (tidak ada padanan)
  - JS: `Don't forget to take a break 🍵`
    Rust: (tidak ada padanan)
  - JS: `FN:␀`
    Rust: (tidak ada padanan)
  - JS: `Gagal load assets:`
    Rust: (tidak ada padanan)
  - JS: `HH:mm`
    Rust: (tidak ada padanan)
  - JS: `Hey ␀, My name is ␀!`
    Rust: (tidak ada padanan)
  - JS: `Hope you have a wonderful day! 🌸`
    Rust: (tidak ada padanan)
  - JS: `Keep smiling and stay positive! ✨`
    Rust: (tidak ada padanan)
  - JS: `Keep up the great work! 🌟`
    Rust: (tidak ada padanan)
  - JS: `Start your day with a positive mindset 😇`
    Rust: (tidak ada padanan)
  - JS: `TEL;type=CELL;type=VOICE;waid=␀:+␀`
    Rust: (tidak ada padanan)
  - JS: `Thank you for using this bot 🙏`
    Rust: (tidak ada padanan)
  - JS: `What a beautiful day to create something 💻`
    Rust: (tidak ada padanan)
  - JS: `[Menu] Error on command execution:`
    Rust: (tidak ada padanan)
  - JS: `[Menu] Fallback juga gagal:`
    Rust: (tidak ada padanan)
  - JS: `item1.TEL;waid=13135550002:+1 (313) 555-0002`
    Rust: (tidak ada padanan)
  - JS: `status@broadcast`
    Rust: (tidak ada padanan)
  - JS: `velyx store`
    Rust: (tidak ada padanan)
  - JS: `␀ | ␀`
    Rust: `␀ ␀`
  - JS: `␀menucat ␀ ␀`
    Rust: `␀ ␀menucat ␀ ␀`
  - JS: `│ ◈ *Pustaka* : `foto-baileys``
    Rust: `│ ◈ *Pustaka*: `foto-baileys``
  - JS: `│ ◦ Mode : ␀`
    Rust: `││  role : ␀`
  - JS: `││  mode : ␀`
    Rust: `││  role : ␀`
  - JS: `┆   • Mode : ␀`
    Rust: `┆   • Mode : PUBLIC`
  - JS: `┆   • Waktu : ␀ WIB`
    Rust: `┆   • Waktu : ␀:␀ WIB`
  - JS: `┆   • ␀`
    Rust: `┆   •`
  - JS: `┆   • ␀␀ (␀x)`
    Rust: (tidak ada padanan)
  - JS: `╭   • Belum ada command`
    Rust: (tidak ada padanan)
  - JS: `╭   • ␀`
    Rust: `╭   •`
  - JS: `╭   • ␀␀ (␀x)`
    Rust: (tidak ada padanan)
  - JS: `⚠️ Menu gagal ditampilkan.`
    Rust: (tidak ada padanan)
  - JS: `✦ History ✦`
    Rust: (tidak ada padanan)
  - JS: `❑ Energy: ␀`
    Rust: (tidak ada padanan)
  - JS: `🏔 𝘐 𝘊𝘢𝘯 𝘩𝘦𝘭𝘱 𝘺𝘰𝘶 𝘸𝘪𝘵𝘩 𝘴𝘦𝘷𝘦𝘳𝘢𝘭 𝘵𝘩𝘪𝘯𝘨𝘴 𝘸𝘪𝘵𝘩𝘪𝘯 𝘞𝘩𝘢𝘵𝘴𝘈𝘱𝘱. 𝘈𝘯𝘥 𝘐 𝘢𝘮 𝘈𝘳𝘮𝘦𝘥 𝘢 *𝘑𝘢𝘷𝘢𝘴𝘤𝘳𝘪𝘱𝘵* 𝘗𝘳𝘰𝘨𝘳𝘢𝘮 𝘈𝘴𝘴𝘦𝘮𝘣𝘭𝘦𝘥 𝘣𝘺 𝘮𝘺 𝘤𝘳𝘦𝘢𝘵𝘰𝘳.`
    Rust: `🏔 𝘐 𝘊𝘢𝘯 𝘩𝘦𝘳𝘱 𝘺𝘰𝘶 𝘸𝘪𝘵𝘩 𝘴𝘦𝘷𝘦𝘳𝘢𝘯 𝘵𝘩𝘪𝘯𝘨𝘴 𝘸𝘪𝘵𝘪𝘯 𝘞𝘩𝘢𝘵𝘈𝘱𝘱. 𝘈𝘯𝘥 𝘐 𝘢𝘮 𝘈𝘳𝘮𝘦𝘥 𝘢 *𝘑𝘢𝘷𝘢𝘴𝘤𝘳𝘪𝘱𝘵* 𝘗𝘳𝘰𝘨𝘳𝘢𝘮 𝘈𝘴𝘴𝘦𝘮𝘣𝘭𝘦𝘥 𝘣𝘺 𝘮𝘺 𝘤𝘳𝘦𝘢𝘵𝘰𝘳.`
  - JS: `🐉 Mythic`
    Rust: (tidak ada padanan)
  - JS: `🪸 ␀`
    Rust: (tidak ada padanan)

### menucat (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/menucat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/menucat.rs`

  - JS: `(␀ plugin + ␀ case)`
    Rust: (tidak ada padanan)
  - JS: `.menucat tools`
    Rust: `.menucat`
  - JS: `Dibuat oleh`
    Rust: `Dibuat oleh ␀`
  - JS: `Menampilkan commands dalam kategori tertentu`
    Rust: (tidak ada padanan)

### owner (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/owner.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/owner.rs`

  - JS: `BEGIN:VCARD`
    Rust: (tidak ada padanan)
  - JS: `END:VCARD`
    Rust: (tidak ada padanan)
  - JS: `FN:␀`
    Rust: (tidak ada padanan)
  - JS: `FN:␀ (Owner ␀)`
    Rust: (tidak ada padanan)
  - JS: `Ini adalah owner kami`
    Rust: (tidak ada padanan)
  - JS: `TEL;type=CELL;type=VOICE;waid=␀:+␀`
    Rust: (tidak ada padanan)
  - JS: `VERSION:3.0`
    Rust: (tidak ada padanan)
  - JS: `💬 Jika kamu memiliki pertanyaan, jangan ragu untuk bertanya, owner ramah kok`
    Rust: (tidak ada padanan)

### ping (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/ping.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/ping.rs`

  - JS: `0 B`
    Rust: (tidak ada padanan)
  - JS: `15m: ␀`
    Rust: (tidak ada padanan)
  - JS: `1m:  ␀`
    Rust: (tidak ada padanan)
  - JS: `22px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `5m:  ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Cores:* ␀ Core(s)`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Dipakai:* ␀ (␀%)`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Engine V8:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *External:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Heap Total:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Heap Used:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Hostname:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Load Avg:* ␀ (1m), ␀ (5m), ␀ (15m)`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Model:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *NodeJS:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *OS:* ␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Platform:* ␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *RSS:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Sisa Bebas:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Speed:* ␀ MHz`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Total RAM:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Uptime Bot:* ␀`
    Rust: (tidak ada padanan)
  - JS: `> ◦ *Uptime Server:* ␀`
    Rust: (tidak ada padanan)
  - JS: `Arch:`
    Rust: (tidak ada padanan)
  - JS: `Berikut adalah detail spesifikasi dan performa server secara lengkap:`
    Rust: (tidak ada padanan)
  - JS: `Bot:`
    Rust: (tidak ada padanan)
  - JS: `CONNECTION: ␀`
    Rust: (tidak ada padanan)
  - JS: `CPU PROCESSOR`
    Rust: (tidak ada padanan)
  - JS: `Cores:`
    Rust: (tidak ada padanan)
  - JS: `External:`
    Rust: (tidak ada padanan)
  - JS: `Free: ␀`
    Rust: (tidak ada padanan)
  - JS: `Heap Total:`
    Rust: (tidak ada padanan)
  - JS: `Heap Used:`
    Rust: (tidak ada padanan)
  - JS: `Host:`
    Rust: (tidak ada padanan)
  - JS: `LATENCY PING`
    Rust: (tidak ada padanan)
  - JS: `MEMORY USAGE`
    Rust: (tidak ada padanan)
  - JS: `NODE.JS ENGINE`
    Rust: (tidak ada padanan)
  - JS: `Node:`
    Rust: (tidak ada padanan)
  - JS: `OS:`
    Rust: (tidak ada padanan)
  - JS: `RSS:`
    Rust: (tidak ada padanan)
  - JS: `STATUS: ONLINE`
    Rust: (tidak ada padanan)
  - JS: `SYSTEM DIAGNOSTICS`
    Rust: (tidak ada padanan)
  - JS: `SYSTEM LOAD`
    Rust: (tidak ada padanan)
  - JS: `SYSTEM SPECS`
    Rust: (tidak ada padanan)
  - JS: `Server:`
    Rust: (tidak ada padanan)
  - JS: `Sistem berjalan stabil dan menyelesaikan kalkulasi dalam waktu eksekusi *␀ms*.`
    Rust: (tidak ada padanan)
  - JS: `Speed:`
    Rust: (tidak ada padanan)
  - JS: `TOTAL: ␀`
    Rust: (tidak ada padanan)
  - JS: `USED RAM`
    Rust: (tidak ada padanan)
  - JS: `Unknown CPU`
    Rust: (tidak ada padanan)
  - JS: `Used: ␀`
    Rust: (tidak ada padanan)
  - JS: `V8 Engine:`
    Rust: (tidak ada padanan)
  - JS: `blur(80px)`
    Rust: (tidak ada padanan)
  - JS: `bold 18px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 20px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 22px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 24px Courier New`
    Rust: (tidak ada padanan)
  - JS: `bold 24px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 28px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 30px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 50px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `bold 70px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `rgba(15, 23, 42, 0.6)`
    Rust: (tidak ada padanan)
  - JS: `rgba(192, 132, 252, 0.05)`
    Rust: (tidak ada padanan)
  - JS: `rgba(255, 255, 255, 0.03)`
    Rust: (tidak ada padanan)
  - JS: `rgba(255, 255, 255, 0.1)`
    Rust: (tidak ada padanan)
  - JS: `rgba(30, 41, 59, 0.8)`
    Rust: (tidak ada padanan)
  - JS: `rgba(56, 189, 248, 0.04)`
    Rust: (tidak ada padanan)
  - JS: `rgba(56, 189, 248, 0.05)`
    Rust: (tidak ada padanan)
  - JS: `⌖ ␀`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *WAKTU AKTIF (UPTIME)*`
    Rust: (tidak ada padanan)
  - JS: `␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `␀ MHz`
    Rust: (tidak ada padanan)
  - JS: `␀ Threads`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `␀%`
    Rust: (tidak ada padanan)
  - JS: `␀d ␀h ␀m`
    Rust: (tidak ada padanan)
  - JS: `␀h ␀m ␀s`
    Rust: (tidak ada padanan)
  - JS: `␀m ␀s`
    Rust: (tidak ada padanan)
  - JS: `🏓 *PONG!* (␀ms)`
    Rust: (tidak ada padanan)
  - JS: `💻 *INFORMASI CPU*`
    Rust: (tidak ada padanan)
  - JS: `📦 *MEMORI NODEJS*`
    Rust: (tidak ada padanan)
  - JS: `🖥️ *INFORMASI SISTEM*`
    Rust: (tidak ada padanan)
  - JS: `🧠 *PENGGUNAAN MEMORI*`
    Rust: (tidak ada padanan)

### ping2 (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/ping2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/ping2.rs`

  - JS: `CPU Load`
    Rust: `CPU Load | ␀%`
  - JS: `Kecepatan Respon bot mu`
    Rust: `Kecepatan Respon bot mu | ␀ ms`
  - JS: `Uptime Bot`
    Rust: `Uptime Bot | ␀`
  - JS: `Uptime Server`
    Rust: `Uptime Server | ␀`
  - JS: `WA Roundtrip`
    Rust: `WA Roundtrip | 1 ms`
  - JS: `df -k --output=size,used /`
    Rust: `--output=size,used`
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `wmic logicaldisk where "DeviceID='C:'" get Size,FreeSpace /format:value`
    Rust: (tidak ada padanan)
  - JS: `␀ / ␀`
    Rust: `␀ ␀`
  - JS: `␀ ms`
    Rust: (tidak ada padanan)
  - JS: `␀%`
    Rust: (tidak ada padanan)

### stats (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/stats.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/stats.rs`

  - JS: `Bot | ␀;;Version | ␀;;Uptime | ␀`
    Rust: (tidak ada padanan)
  - JS: `Platform | ␀;;Node | ␀;;CPU Load | ␀`
    Rust: (tidak ada padanan)
  - JS: `RAM | ␀;;Heap | ␀;;Updated | ␀`
    Rust: (tidak ada padanan)
  - JS: `Users | ␀;;Premium | ␀;;Groups | ␀`
    Rust: (tidak ada padanan)
  - JS: `v␀`
    Rust: (tidak ada padanan)
  - JS: `␀ / ␀`
    Rust: `␀ ␀`
  - JS: `␀%`
    Rust: (tidak ada padanan)
  - JS: `📊 Bot Statistics`
    Rust: (tidak ada padanan)

### stopjadibot (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/stopjadibot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/stopjadibot.rs`

  - JS: `> Ketik `␀jadibot` untuk menjadi bot`
    Rust: (tidak ada padanan)
  - JS: `> ⏱️ Uptime: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> 💾 Session: *Tersimpan*`
    Rust: (tidak ada padanan)
  - JS: `> 📱 Nomor: *@␀*`
    Rust: (tidak ada padanan)
  - JS: `Ketik `␀jadibot` untuk mengaktifkan kembali.`
    Rust: (tidak ada padanan)
  - JS: `␀h ␀m`
    Rust: (tidak ada padanan)
  - JS: `␀m ␀s`
    Rust: (tidak ada padanan)
  - JS: `␀s`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴋᴀᴍᴜ ᴛɪᴅᴀᴋ ᴀᴅᴀʟᴀʜ ᴊᴀᴅɪʙᴏᴛ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal menghentikan jadibot: ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengidentifikasi nomor kamu`
    Rust: (tidak ada padanan)
  - JS: `🛑 *ᴊᴀᴅɪʙᴏᴛ ᴅɪʜᴇɴᴛɪᴋᴀɴ*`
    Rust: (tidak ada padanan)

### tqto (main)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/main/tqto.js`
- Rust: `/workspaces/oktz-rs/plugin/src/main/tqto.rs`

  - JS: `')}}`
    Rust: (tidak ada padanan)
  - JS: `␀*. *␀* [ ␀ ␀ ]`).join('`
    Rust: (tidak ada padanan)

### music (media)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/media/music.js`
- Rust: `/workspaces/oktz-rs/plugin/src/media/music.rs`

  - JS: `music␀`
    Rust: `.music1`

### paptt (nsfw)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/nsfw/paptt.js`
- Rust: `/workspaces/oktz-rs/plugin/src/nsfw/paptt.rs`

  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)

### addenergi (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addenergi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addenergi.rs`

  - JS: `> `.addenergi --unlimited` - unlimited`
    Rust: `> `.␀addenergi --unlimited` - unlimited`
  - JS: `> `.addenergi <jumlah> @user` - ke user`
    Rust: `> `.␀addenergi <jumlah> @user` - ke user`
  - JS: `> `.addenergi <jumlah>` - ke diri sendiri`
    Rust: `> `.␀addenergi <jumlah>` - ke diri sendiri`
  - JS: `✅ *Energi @␀ sekarang unlimited / tidak terbatas*`
    Rust: `✅ Energi @␀ sekarang unlimited / tidak terbatas`

### addexp (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addexp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addexp.rs`

  - JS: `> Jumlah exp harus lebih dari 0`
    Rust: (tidak ada padanan)
  - JS: `> `.addexp <jumlah> @user` - ke user`
    Rust: `> `.␀addexp <jumlah> @user` - ke user`
  - JS: `> `.addexp <jumlah>` - ke diri sendiri`
    Rust: `> `.␀addexp <jumlah>` - ke diri sendiri`
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)

### addkoin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addkoin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addkoin.rs`

  - JS: `> `.addkoin <jumlah> @user` - ke orang lain`
    Rust: `> `.␀addkoin <jumlah> @user` - ke orang lain`
  - JS: `> `.addkoin <jumlah>` - ke diri sendiri`
    Rust: `> `.␀addkoin <jumlah>` - ke diri sendiri`

### addowner (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addowner.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addowner.rs`

  - JS: `08`
    Rust: (tidak ada padanan)
  - JS: `62`
    Rust: (tidak ada padanan)
  - JS: `> Belum ada owner panel terdaftar.`
    Rust: (tidak ada padanan)
  - JS: `> Belum ada owner terdaftar.`
    Rust: (tidak ada padanan)
  - JS: `> Current list: ␀`
    Rust: (tidak ada padanan)
  - JS: `> Format nomor tidak valid`
    Rust: (tidak ada padanan)
  - JS: `> Gunakan `␀addowner` untuk menambah.`
    Rust: (tidak ada padanan)
  - JS: `> ⚡ Auto-upgrade dari Panel Owner`
    Rust: (tidak ada padanan)
  - JS: `> ⚡ Auto-upgrade dari Seller`
    Rust: (tidak ada padanan)
  - JS: `> ⚡ Auto-upgrade dari Seller ke Owner Panel`
    Rust: (tidak ada padanan)
  - JS: `Kelola owner bot (mode-aware)`
    Rust: `Kelola owner bot (mode-aware) — BLOKIRAN`
  - JS: `Reply/tag/ketik nomor user`
    Rust: (tidak ada padanan)
  - JS: `Total: *␀* owner`
    Rust: (tidak ada padanan)
  - JS: `Total: *␀* owner | ⭐ Main, 👑 Added`
    Rust: (tidak ada padanan)
  - JS: `Total: *␀* owner | 👑 Full, 🖥️ Panel`
    Rust: (tidak ada padanan)
  - JS: `[AddOwner] Failed to save panel config:`
    Rust: (tidak ada padanan)
  - JS: ``Contoh: ␀␀ 6281234567890``
    Rust: (tidak ada padanan)
  - JS: ``Dengan nama: ␀␀ 6281234567890 NamaOwner``
    Rust: (tidak ada padanan)
  - JS: `config.js`
    Rust: (tidak ada padanan)
  - JS: `ownerPanels: ␀`
    Rust: (tidak ada padanan)
  - JS: `sellers: ␀`
    Rust: (tidak ada padanan)
  - JS: `␀. ␀ @␀␀*` : ""}`
    Rust: (tidak ada padanan)
  - JS: `␀. 👑 @␀␀*` : ""}`
    Rust: (tidak ada padanan)
  - JS: `␀@s.whatsapp.net`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menambahkan *␀* sebagai full owner␀)` : ""}␀`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menambahkan *␀* sebagai owner jadibot`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menambahkan *␀* sebagai owner panel␀`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menghapus *␀* dari full owner`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menghapus *␀* dari owner jadibot`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menghapus *␀* dari owner panel`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal menyimpan ke config.js`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` bukan full owner.`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` bukan owner Jadibot ini.`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` bukan owner panel.`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` sudah menjadi full owner.`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` sudah menjadi owner Jadibot ini.`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` sudah menjadi owner panel.`
    Rust: (tidak ada padanan)
  - JS: `👑 *␀ OWNER*`
    Rust: (tidak ada padanan)
  - JS: `👑⭐`
    Rust: (tidak ada padanan)
  - JS: `👑🖥️`
    Rust: (tidak ada padanan)
  - JS: `📋 *DAFTAR OWNER JADIBOT* — ␀`
    Rust: (tidak ada padanan)
  - JS: `📋 *DAFTAR OWNER PANEL*`
    Rust: (tidak ada padanan)
  - JS: `📋 *DAFTAR OWNER*`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴅᴀꜰᴛᴀʀ ᴏᴡɴᴇʀ ᴊᴀᴅɪʙᴏᴛ*`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴅᴀꜰᴛᴀʀ ᴏᴡɴᴇʀ ᴘᴀɴᴇʟ*`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴅᴀꜰᴛᴀʀ ᴏᴡɴᴇʀ*`
    Rust: (tidak ada padanan)
  - JS: `🖥️`
    Rust: (tidak ada padanan)

### addpartner (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addpartner.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addpartner.rs`

  - JS: `62`
    Rust: `62␀`
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `⚠️ @␀ sudah menjadi owner!`
    Rust: (tidak ada padanan)

### addplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addplugin.rs`

  - JS: `"`]([^`
    Rust: (tidak ada padanan)
  - JS: `- Status Reload: ␀`
    Rust: `Status Reload`
  - JS: ``]+)['`
    Rust: (tidak ada padanan)
  - JS: `␀.js`
    Rust: `␀/␀.js`

### addprem (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addprem.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addprem.rs`

  - JS: `62`
    Rust: `62␀`
  - JS: `Expired: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Total: *␀* premium`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `␀. @␀`
    Rust: (tidak ada padanan)
  - JS: `␀. @␀ — ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menambahkan *␀* sebagai premium jadibot`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menghapus *␀* dari premium`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil menghapus *␀* dari premium jadibot`
    Rust: (tidak ada padanan)
  - JS: `✅ Berhasil ␀ premium *␀* selama *␀*`
    Rust: (tidak ada padanan)
  - JS: `❌ *␀* bukan premium`
    Rust: (tidak ada padanan)
  - JS: `❌ `␀` bukan premium di Jadibot ini`
    Rust: `💎 Belum ada premium di jadibot ini`
  - JS: `❌ `␀` sudah premium di Jadibot ini`
    Rust: `💎 Belum ada premium di jadibot ini`
  - JS: `💎 *DAFTAR PREMIUM JADIBOT* — ␀`
    Rust: (tidak ada padanan)
  - JS: `💎 *DAFTAR PREMIUM*`
    Rust: `💎 *DEL PREMIUM*`

### addsewa (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/addsewa.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/addsewa.rs`

  - JS: `- Aku akan keluar pada: *␀*`
    Rust: (tidak ada padanan)
  - JS: `- Masa sewa: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Bot sudah ada di grup`
    Rust: `bot sudah ada di grup`
  - JS: `Gagal join grup`
    Rust: (tidak ada padanan)
  - JS: `Ketik *␀menu* untuk melihat fitur dari bot ini.`
    Rust: (tidak ada padanan)
  - JS: `👋 *Halo Semuanya!*, perkenalkan, aku ␀`
    Rust: (tidak ada padanan)

### akses (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/akses.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/akses.rs`

  - JS: `> `␀addakses addowner 30d @user` (30 Hari)`
    Rust: `> `.addakses addowner 30d @user` (30 Hari)`
  - JS: `> `␀addakses unban permanent @user` (Selamanya)`
    Rust: `> `.addakses unban permanent @user` (Selamanya)`
  - JS: `Active Access List`
    Rust: (tidak ada padanan)
  - JS: `Format: `␀addakses <command> <durasi> <target>``
    Rust: `Format: `.addakses <command> <durasi> <target>``
  - JS: `Hapus: ␀`
    Rust: (tidak ada padanan)
  - JS: `Manage Access`
    Rust: (tidak ada padanan)
  - JS: `PILIH COMMAND`
    Rust: (tidak ada padanan)
  - JS: `Sisa durasi: ␀`
    Rust: (tidak ada padanan)
  - JS: `[DEBUG AddAccess] After save:`
    Rust: (tidak ada padanan)
  - JS: `[DEBUG AddAccess] Saving user with access:`
    Rust: (tidak ada padanan)
  - JS: `␀delakses ␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `♾️ Permanent`
    Rust: (tidak ada padanan)
  - JS: `❌ Format durasi tidak dikenali!`
    Rust: (tidak ada padanan)
  - JS: `🔴 Expired`
    Rust: (tidak ada padanan)

### aktifaudiomenu (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/aktifaudiomenu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/aktifaudiomenu.rs`

  - JS: `Gunakan: `ya` atau `gak``
    Rust: (tidak ada padanan)
  - JS: `❌ Opsi tidak valid!`
    Rust: (tidak ada padanan)

### anticall (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/anticall.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/anticall.rs`

  - JS: `> *␀anticall off* — Nonaktifkan`
    Rust: `> *␀anticall off* — Nonaktif`

### arsip (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/arsip.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/arsip.rs`

  - JS: `> Private chat tidak bisa diarsipkan sekaligus (tidak ada daftar chat)`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal: ␀`
    Rust: (tidak ada padanan)
  - JS: `📁 *␀ grup diarsipkan*`
    Rust: (tidak ada padanan)

### autobackup (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autobackup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autobackup.rs`

  - JS: `> Backup otomatis sudah dihentikan.`
    Rust: (tidak ada padanan)
  - JS: `> Backup pertama akan dikirim dalam ␀`
    Rust: (tidak ada padanan)
  - JS: `> Backup telah dikirim ke owner!`
    Rust: (tidak ada padanan)
  - JS: `> Gunakan `␀autobackup on <interval>` untuk mengaktifkan kembali.`
    Rust: (tidak ada padanan)
  - JS: `> Mohon tunggu, sedang membuat backup...`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `Owner #1`
    Rust: (tidak ada padanan)
  - JS: `Tidak diset`
    Rust: (tidak ada padanan)
  - JS: `┃ #️⃣ Total: ␀ backup`
    Rust: (tidak ada padanan)
  - JS: `┃ ⏱️ Interval: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📅 Last Backup: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📅 Last: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📤 Dikirim ke: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📤 Target: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📦 Exclude: node_modules, .git, storages, dll`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔄 Running: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔘 Enabled: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔘 Status: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ⚙️ *sᴇᴛᴛɪɴɢs* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📊 *sᴛᴀᴛᴜs* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📊 *ɪɴꜰᴏ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴀᴄᴋᴜᴘ sᴇʟᴇsᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴀᴜᴛᴏ ʙᴀᴄᴋᴜᴘ ᴅɪᴀᴋᴛɪꜰᴋᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴀᴜᴛᴏ ʙᴀᴄᴋᴜᴘ ᴅɪɴᴏɴᴀᴋᴛɪꜰᴋᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *ᴍᴇᴍʙᴜᴀᴛ ʙᴀᴄᴋᴜᴘ...*`
    Rust: (tidak ada padanan)
  - JS: `🗂️ *sᴛᴀᴛᴜs ᴀᴜᴛᴏ ʙᴀᴄᴋᴜᴘ*`
    Rust: (tidak ada padanan)

### autobio (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autobio.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autobio.rs`

  - JS: `- *␀autobio ganti_setiap <waktu>* — Mengatur interval. Contoh: `.autobio ganti_setiap 30 detik` atau `1 jam``
    Rust: `- *␀autobio ganti_setiap <waktu>* — Mengatur interval. Contoh: `.␀autobio ganti_setiap 30 detik` atau `1 jam``
  - JS: `- `{botname}` — Menampilkan nama bot dari config`
    Rust: `- `{␀}` — Menampilkan nama bot dari config`
  - JS: `- `{clock}` — Menampilkan jam saat ini`
    Rust: `- `{␀}` — Menampilkan jam saat ini`
  - JS: `- `{runtime}` — Menampilkan lama bot menyala`
    Rust: `- `{␀}` — Menampilkan lama bot menyala`
  - JS: `- `{version}` — Menampilkan versi bot`
    Rust: `- `{␀}` — Menampilkan versi bot`
  - JS: `.autobio Bot by Owner | 🕒 {clock} | ⏳ {runtime}`
    Rust: `.autobio Bot by Owner | 🕒 ␀ | ⏳ ␀`
  - JS: `Bio WhatsApp bot sekarang akan diperbarui secara otomatis setiap menit.`
    Rust: `sekarang akan diperbarui secara otomatis setiap menit.`
  - JS: `Bio WhatsApp bot tidak akan di-update lagi.`
    Rust: `tidak akan di-update lagi.`
  - JS: `Bot aktif | 🕒 {clock} | ⏳ {runtime}`
    Rust: (tidak ada padanan)
  - JS: `✅ *AUTO BIO DIAKTIFKAN*`
    Rust: (tidak ada padanan)
  - JS: `❌ *AUTO BIO DINONAKTIFKAN*`
    Rust: (tidak ada padanan)

### autojoingc (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autojoingc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autojoingc.rs`

  - JS: `⚠️ Sudah ada di grup tersebut`
    Rust: (tidak ada padanan)
  - JS: `❌ Link grup sudah expired/revoked`
    Rust: (tidak ada padanan)

### autoreactsw (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autoreactsw.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autoreactsw.rs`

  - JS: `> `␀autoreactsw on` — Aktifkan (emoji default 🔥)`
    Rust: `> `␀autoreactsw on` — Aktifkan (emoji default ␀)`
  - JS: `❌ Gunakan `on` atau `off``
    Rust: (tidak ada padanan)

### autoread (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autoread.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autoread.rs`

  - JS: `> *␀autoread off* — Nonaktifkan`
    Rust: `> *␀autoread off* — Nonaktif`

### autoreadsw (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autoreadsw.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autoreadsw.rs`

  - JS: `❌ Gunakan `on` atau `off``
    Rust: (tidak ada padanan)

### autosambut (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autosambut.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autosambut.rs`

  - JS: `2. Gunakan *{name}* untuk menyebut pushname owner, dan *{user}* untuk me-mention owner.`
    Rust: `2. Gunakan *{␀}* untuk menyebut pushname owner, dan *{␀}* untuk me-mention owner.`
  - JS: `Contoh: `␀autosambut add Halo bosku {user}!``
    Rust: `Contoh: `␀autosambut add Halo bosku {␀}!``
  - JS: `Format waktu tidak dikenali. Gunakan angka dan akhiran s, m, h, d, w, y. Contoh: `2h` (2 jam), `30m` (30 menit).`
    Rust: `Format waktu tidak dikenali. Gunakan angka dan akhiran s, m, h, d, w, y.`
  - JS: `Halo {user}! Selamat datang kembali 🙇‍♂️`
    Rust: `Halo ␀! Selamat datang kembali 🙇‍♂️`
  - JS: `Pesan terdaftar:`
    Rust: (tidak ada padanan)
  - JS: `Sekarang bot akan menyambutmu setelah kamu tidak mengetik apa-apa di grup ini selama *␀* berturut-turut.`
    Rust: `Sekarang bot akan menyambutmu setelah kamu tidak mengetik apa-apa di grup ini`
  - JS: `Semua grup (␀) sekarang menggunakan sistem sapaan yang sama dengan grup ini.`
    Rust: (tidak ada padanan)
  - JS: `Silakan gunakan `␀autosambut add <teks>` untuk menambahkan sapaan, atau `␀autosambut list` untuk melihat daftar sapaan.`
    Rust: (tidak ada padanan)
  - JS: `Tolong masukkan teks sambutannya.`
    Rust: `Tolong masukkan teks sambutan.`
  - JS: `⏱️ *Delay Auto Sambut Global Diubah ke ␀ untuk ␀ grup!*`
    Rust: (tidak ada padanan)
  - JS: `␀ *Fitur Auto Sambut Global ␀!*`
    Rust: (tidak ada padanan)
  - JS: `💬 *Pesan Baru Ditambahkan ke Daftar Global (␀ grup)!*`
    Rust: (tidak ada padanan)
  - JS: `🗑️ *Pesan Berhasil Dihapus Secara Global (␀ grup)!*`
    Rust: (tidak ada padanan)

### autosholat (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/autosholat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/autosholat.rs`

  - JS: `Gagal membuka obrolan grup ␀:`
    Rust: (tidak ada padanan)
  - JS: `Gagal mengirim pesan sholat ke grup ␀:`
    Rust: (tidak ada padanan)
  - JS: `Mari segarkan pikiran, ambil air wudhu, dan hampiri panggilan suci-Nya. Selamat menunaikan ibadah sholat! 🤲`
    Rust: (tidak ada padanan)
  - JS: `Penyiaran adzan ␀ berhasil dilakukan ke ␀ grup secara paralel.`
    Rust: (tidak ada padanan)
  - JS: `Sesi ibadah sholat **␀** telah usai. Obrolan grup sekarang sudah kubuka kembali secara otomatis. Selamat melanjutkan aktivitas kembali!`
    Rust: (tidak ada padanan)
  - JS: `Sudah saatnya mengistirahatkan sejenak urusan duniamu! Waktu untuk menunaikan ibadah sholat **␀** telah tiba untuk wilayah **␀** dan sekitarnya (tepatnya pada pukul **␀ WIB**).`
    Rust: (tidak ada padanan)
  - JS: `Terdapat kesalahan pada eksekutor:`
    Rust: (tidak ada padanan)
  - JS: `_Sebagai bentuk penghormatan, sistem akan menutup obrolan grup ini untuk sementara waktu (selama ␀ menit)._`
    Rust: (tidak ada padanan)
  - JS: `✅ **Waktu Penutupan Telah Berakhir**`
    Rust: (tidak ada padanan)
  - JS: `🕌 **Pemberitahuan Waktu Sholat ␀** 🕌`
    Rust: (tidak ada padanan)

### baca (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/baca.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/baca.rs`

  - JS: `❌ Gagal: ␀`
    Rust: (tidak ada padanan)

### backupdb (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/backupdb.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/backupdb.rs`

  - JS: `> Backup telah dikirim ke owner utama.`
    Rust: (tidak ada padanan)
  - JS: `> Type-safe backup, kompatibel dengan update mendatang.`
    Rust: (tidak ada padanan)
  - JS: `✅ *Backup Berhasil!*`
    Rust: (tidak ada padanan)
  - JS: `❌ Backup gagal: ␀`
    Rust: (tidak ada padanan)
  - JS: `📁 Files: ␀`
    Rust: (tidak ada padanan)
  - JS: `📦 Size: ␀`
    Rust: (tidak ada padanan)
  - JS: `🔖 Schema: v␀`
    Rust: (tidak ada padanan)

### backupsc (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/backupsc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/backupsc.rs`

  - JS: `.7z`
    Rust: (tidak ada padanan)
  - JS: `.avi`
    Rust: (tidak ada padanan)
  - JS: `.bak`
    Rust: (tidak ada padanan)
  - JS: `.cache`
    Rust: (tidak ada padanan)
  - JS: `.env`
    Rust: (tidak ada padanan)
  - JS: `.env.local`
    Rust: (tidak ada padanan)
  - JS: `.git`
    Rust: (tidak ada padanan)
  - JS: `.gitignore`
    Rust: (tidak ada padanan)
  - JS: `.ico`
    Rust: (tidak ada padanan)
  - JS: `.lock`
    Rust: (tidak ada padanan)
  - JS: `.log`
    Rust: (tidak ada padanan)
  - JS: `.mkv`
    Rust: (tidak ada padanan)
  - JS: `.mp3`
    Rust: (tidak ada padanan)
  - JS: `.mp4`
    Rust: (tidak ada padanan)
  - JS: `.npm`
    Rust: (tidak ada padanan)
  - JS: `.npmrc`
    Rust: (tidak ada padanan)
  - JS: `.svg`
    Rust: (tidak ada padanan)
  - JS: `.tar.gz`
    Rust: (tidak ada padanan)
  - JS: `.traineddata`
    Rust: (tidak ada padanan)
  - JS: `.wav`
    Rust: (tidak ada padanan)
  - JS: `.yarn`
    Rust: (tidak ada padanan)
  - JS: `.zip`
    Rust: (tidak ada padanan)
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `CHANGELOG.md`
    Rust: (tidak ada padanan)
  - JS: `File backup kosong atau 0KB`
    Rust: (tidak ada padanan)
  - JS: `File backup tidak ditemukan setelah proses zip`
    Rust: (tidak ada padanan)
  - JS: `Tidak ada file yang masuk ke backup script`
    Rust: (tidak ada padanan)
  - JS: `UPDATE.txt`
    Rust: (tidak ada padanan)
  - JS: `YYYY-MM-DD_HH-mm-ss`
    Rust: (tidak ada padanan)
  - JS: `boot_final.log`
    Rust: (tidak ada padanan)
  - JS: `bot_log.txt`
    Rust: (tidak ada padanan)
  - JS: `changelog.txt`
    Rust: (tidak ada padanan)
  - JS: `creds.json`
    Rust: (tidak ada padanan)
  - JS: `error.txt`
    Rust: (tidak ada padanan)
  - JS: `package-lock.json`
    Rust: (tidak ada padanan)
  - JS: `yarn.lock`
    Rust: (tidak ada padanan)
  - JS: `␀_backup_␀.zip`
    Rust: (tidak ada padanan)

### ban (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ban.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ban.rs`

  - JS: `> Tidak dapat ban owner`
    Rust: (tidak ada padanan)
  - JS: `🚫 *ʙᴀɴ ᴜsᴇʀ*`
    Rust: `🚫 *ʙᴀɴ ᴅsᴇʀ*`

### bcgc (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/bcgc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/bcgc.rs`

  - JS: `*CARA PAKAI:*`
    Rust: (tidak ada padanan)
  - JS: `*CONTOH:*`
    Rust: (tidak ada padanan)
  - JS: `*Cara yang benar:*`
    Rust: (tidak ada padanan)
  - JS: `*JEDA:*`
    Rust: (tidak ada padanan)
  - JS: `*PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `*SATUAN:*`
    Rust: (tidak ada padanan)
  - JS: `*STOP:*`
    Rust: (tidak ada padanan)
  - JS: `*Status saat ini:*`
    Rust: (tidak ada padanan)
  - JS: `1. Kirim teks/foto/video/audio/dokumen`
    Rust: (tidak ada padanan)
  - JS: `2. Reply pesan tersebut dengan *␀bcgc*`
    Rust: (tidak ada padanan)
  - JS: `3. Bot akan broadcast ke semua grup`
    Rust: (tidak ada padanan)
  - JS: `> *s* — detik • *m* — menit • *h* — jam • *d* — hari`
    Rust: (tidak ada padanan)
  - JS: `> *␀bcgc <pesan>* — Kirim broadcast teks`
    Rust: (tidak ada padanan)
  - JS: `> *␀bcgc off* — Nonaktifkan broadcast`
    Rust: (tidak ada padanan)
  - JS: `> *␀bcgc on* — Aktifkan broadcast`
    Rust: (tidak ada padanan)
  - JS: `> *␀bcgc* (reply foto/video/audio/dokumen) — Kirim dengan media`
    Rust: (tidak ada padanan)
  - JS: `> *␀bcgc* (reply pesan teks) — Kirim isi pesan yang di-reply`
    Rust: (tidak ada padanan)
  - JS: `> *␀jedabcgc 1h* → 1 jam`
    Rust: (tidak ada padanan)
  - JS: `> *␀jedabcgc 2m* — Set jeda 2 menit`
    Rust: (tidak ada padanan)
  - JS: `> *␀jedabcgc 2m* → 2 menit`
    Rust: (tidak ada padanan)
  - JS: `> *␀jedabcgc 5s* — Set jeda 5 detik`
    Rust: (tidak ada padanan)
  - JS: `> *␀jedabcgc 5s* → 5 detik`
    Rust: (tidak ada padanan)
  - JS: `> *␀jedabcgc <angka><satuan>*`
    Rust: (tidak ada padanan)
  - JS: `> *␀stopbcgc* — Hentikan broadcast yang berjalan`
    Rust: (tidak ada padanan)
  - JS: `> Bot tidak menemukan grup yang bisa dituju␀ grup di-blacklist)` : ""}`
    Rust: (tidak ada padanan)
  - JS: `> Broadcast grup telah dimatikan.`
    Rust: (tidak ada padanan)
  - JS: `> Broadcast: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Estimasi 100 grup: *␀ menit*`
    Rust: (tidak ada padanan)
  - JS: `> Jeda saat ini: *␀* (*␀ms*)`
    Rust: (tidak ada padanan)
  - JS: `> Jeda: *␀* (*␀ms*)`
    Rust: (tidak ada padanan)
  - JS: `> Ketik *␀bcgc on* dulu untuk mengaktifkan.`
    Rust: (tidak ada padanan)
  - JS: `> Ketik *␀stopbcgc* untuk menghentikan terlebih dahulu.`
    Rust: (tidak ada padanan)
  - JS: `> Sebelumnya: *␀* (*␀ms*)`
    Rust: (tidak ada padanan)
  - JS: `> Sekarang kamu bisa mengirim broadcast ke semua grup.`
    Rust: (tidak ada padanan)
  - JS: `> Sekarang: *␀* (*␀ms*)`
    Rust: (tidak ada padanan)
  - JS: `> ⏱️ Jeda: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> ⏸️ Sisa: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> ✅ Berhasil: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> ❌ Gagal: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> 👥 Target: *␀* grup`
    Rust: (tidak ada padanan)
  - JS: `> 📊 Estimasi: *␀ menit*`
    Rust: (tidak ada padanan)
  - JS: `> 📊 Total: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> 📝 Pesan: *␀␀*`
    Rust: (tidak ada padanan)
  - JS: `> 📷 Media: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Atur jeda waktu antar pengiriman pesan ke setiap grup.`
    Rust: (tidak ada padanan)
  - JS: `Broadcast pesan ke semua grup dengan dukungan semua jenis media`
    Rust: `Broadcast pesan ke semua grup dengan dukungan semua jenis media (DIBLOKIR: butuh sock, db, dan state global)`
  - JS: `Kirim pesan ke seluruh grup sekaligus dalam satu perintah.`
    Rust: (tidak ada padanan)
  - JS: `Kirim pesan, foto, audio, video, atau dokumen terlebih dahulu.`
    Rust: (tidak ada padanan)
  - JS: `Semakin lama jeda, semakin aman dari spam detection.`
    Rust: (tidak ada padanan)
  - JS: `_Sedang mengirim ke semua grup..._`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *Jeda Broadcast Grup*`
    Rust: (tidak ada padanan)
  - JS: `✅ *Broadcast Grup Diaktifkan*`
    Rust: `⏹️ *Broadcast Grup Dihentikan*`
  - JS: `✅ *Broadcast Grup Dinonaktifkan*`
    Rust: `⏹️ *Broadcast Grup Dihentikan*`
  - JS: `✅ *Broadcast Grup Selesai!*`
    Rust: (tidak ada padanan)
  - JS: `✅ *Jeda Broadcast Grup Diubah*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Broadcast Belum Aktif*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Broadcast Sedang Berjalan*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Tidak Ada Grup*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Tidak Ada Konten*`
    Rust: (tidak ada padanan)
  - JS: `❌ Format salah. Contoh: *5s*, *2m*, *1h*, *1d*`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada broadcast grup yang sedang berjalan.`
    Rust: (tidak ada padanan)
  - JS: `📢 *Broadcast Grup Dimulai*`
    Rust: `⏹️ *Broadcast Grup Dihentikan*`
  - JS: `📢 *Broadcast Grup*`
    Rust: (tidak ada padanan)

### bcpc (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/bcpc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/bcpc.rs`

  - JS: `-@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Memulai broadcast...`
    Rust: (tidak ada padanan)
  - JS: `Gagal:`
    Rust: (tidak ada padanan)
  - JS: `┃ ⏱️ ᴊᴇᴅᴀ: `␀ms``
    Rust: (tidak ada padanan)
  - JS: `┃ ✅ ʙᴇʀʜᴀsɪʟ: `␀``
    Rust: (tidak ada padanan)
  - JS: `┃ ❌ ɢᴀɢᴀʟ: `␀``
    Rust: (tidak ada padanan)
  - JS: `┃ 👥 ᴛᴀʀɢᴇᴛ: `␀` kontak`
    Rust: (tidak ada padanan)
  - JS: `┃ 📊 ᴇsᴛɪᴍᴀsɪ: `␀ menit``
    Rust: (tidak ada padanan)
  - JS: `┃ 📊 ᴛᴏᴛᴀʟ: `␀``
    Rust: (tidak ada padanan)
  - JS: `┃ 📝 ᴘᴇsᴀɴ: `␀␀``
    Rust: (tidak ada padanan)
  - JS: `┃ 📷 ᴍᴇᴅɪᴀ: `␀``
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📊 *ʜᴀsɪʟ* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴅᴇᴛᴀɪʟ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙʀᴏᴀᴅᴄᴀsᴛ ᴘʀɪᴠᴀᴛᴇ sᴇʟᴇsᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `📱 *ʙʀᴏᴀᴅᴄᴀsᴛ ᴘʀɪᴠᴀᴛᴇ*`
    Rust: (tidak ada padanan)

### block (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/block.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/block.rs`

  - JS: `> Gunakan `.unblock` untuk membuka blokir`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak bisa blokir nomor bot sendiri.`
    Rust: (tidak ada padanan)

### botafk (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/botafk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/botafk.rs`

  - JS: `␀ detik`
    Rust: (tidak ada padanan)
  - JS: `␀ hari ␀ jam`
    Rust: (tidak ada padanan)
  - JS: `␀ jam ␀ menit`
    Rust: `1 jam 30 menit`
  - JS: `␀ menit ␀ detik`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴏᴛ ᴋᴇᴍʙᴀʟɪ ᴏɴʟɪɴᴇ*`
    Rust: `✅ *ʙᴏᴛ ᴋᴇᴍʘᴀʟɪ ᴏɴʟɪɴᴇ*`

### botmode (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/botmode.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/botmode.rs`

  - JS: `[BotMode] Changed to ␀ by ␀ (␀)`
    Rust: (tidak ada padanan)

### buatgrup (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/buatgrup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/buatgrup.rs`

  - JS: `Gagal menghapus grup otomatis (␀):`
    Rust: (tidak ada padanan)
  - JS: `Sesuai perintah Owner, waktu grup ini telah habis. Sayonara semuanya! 👋`
    Rust: (tidak ada padanan)
  - JS: `_Bot otomatis menjadi admin grup ini ya kak!_`
    Rust: `_Bot otomatis menjadi admin grup ini ya kak!`
  - JS: `⏳ *MASA AKTIF GRUP HABIS* ⏳`
    Rust: (tidak ada padanan)

### buatsaluran (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/buatsaluran.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/buatsaluran.rs`

  - JS: `> Deskripsi: ␀`
    Rust: (tidak ada padanan)
  - JS: `> ID: ␀`
    Rust: (tidak ada padanan)
  - JS: `> Nama: ␀`
    Rust: (tidak ada padanan)
  - JS: `> Subscribers: ␀`
    Rust: (tidak ada padanan)
  - JS: `_Saluran ini bisa dikonfigurasi di config.saluran.id_`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal membuat saluran: ␀`
    Rust: `❌ Gagal membuat saluran:`
  - JS: `📢 *sᴀʟᴜʀᴀɴ ᴅɪʙᴜᴀᴛ*`
    Rust: (tidak ada padanan)

### capenergi (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/capenergi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/capenergi.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `␀`).join("`
    Rust: (tidak ada padanan)
  - JS: `✅ *POTONGAN ENERGI BERHASIL DIUBAH*`
    Rust: `✅ *STATUS BERHASIL DIUBAH*`

### capfree (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/capfree.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/capfree.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `*Gagal (Tidak ditemukan):*`
    Rust: `Gagal (Tidak ditemukan):`
  - JS: `␀`).join("`
    Rust: (tidak ada padanan)
  - JS: `✅ *STATUS BERHASIL DIUBAH*`
    Rust: (tidak ada padanan)

### capprem (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/capprem.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/capprem.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `*Gagal (Tidak ditemukan):*`
    Rust: `Gagal (Tidak ditemukan):`
  - JS: `␀`).join("`
    Rust: (tidak ada padanan)
  - JS: `✅ *STATUS BERHASIL DIUBAH*`
    Rust: (tidak ada padanan)

### cekschedule (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/cekschedule.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/cekschedule.rs`

  - JS: `1301`
    Rust: (tidak ada padanan)
  - JS: `KOTA JAKARTA`
    Rust: (tidak ada padanan)
  - JS: `[CekSchedule Error]`
    Rust: (tidak ada padanan)
  - JS: `en-US`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀*`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: (tidak ada padanan)
  - JS: `└ Active: ␀ | Sent: ␀`
    Rust: (tidak ada padanan)
  - JS: `└ Key: `␀``
    Rust: `└ Key: `sholat``
  - JS: `└ Last: ␀`
    Rust: (tidak ada padanan)
  - JS: `└ Lokasi: ␀`
    Rust: (tidak ada padanan)
  - JS: `└ Next: ␀ (␀ WIB)`
    Rust: (tidak ada padanan)
  - JS: `└ Total Resets: ␀`
    Rust: (tidak ada padanan)
  - JS: `└ _Gagal memuat jadwal_`
    Rust: (tidak ada padanan)
  - JS: `└ ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ Aktif: ␀`
    Rust: `✅ Aktif: 1`
  - JS: `❌ Nonaktif: ␀`
    Rust: `❌ Nonaktif: 1`

### autoclear (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/chatclear.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/chatclear.rs`

  - JS: `> Auto-clear sekarang *AKTIF*`
    Rust: (tidak ada padanan)
  - JS: `> Auto-clear sekarang *MATI*`
    Rust: (tidak ada padanan)
  - JS: `> Chat grup & private tidak akan dihapus otomatis`
    Rust: (tidak ada padanan)
  - JS: `> Perintah tidak dikenal: `␀``
    Rust: (tidak ada padanan)
  - JS: `> Semua chat grup & private akan dihapus otomatis tiap tengah malam (*00:00 WIB*)`
    Rust: (tidak ada padanan)
  - JS: `> Status: ␀`
    Rust: (tidak ada padanan)
  - JS: `> `␀autoclear off` — matikan`
    Rust: (tidak ada padanan)
  - JS: `> `␀autoclear on` — aktifkan`
    Rust: (tidak ada padanan)
  - JS: `> `␀autoclear` — lihat status`
    Rust: (tidak ada padanan)
  - JS: `Auto-hapus semua chat grup & private tiap tengah malam`
    Rust: (tidak ada padanan)
  - JS: `To aktifkan: `␀autoclear on``
    Rust: (tidak ada padanan)
  - JS: `To matikan: `␀autoclear off``
    Rust: (tidak ada padanan)
  - JS: `_Owner only_`
    Rust: (tidak ada padanan)
  - JS: `⚡ *AUTO CLEAR CHAT*`
    Rust: (tidak ada padanan)
  - JS: `✅ *HIDUP* — semua chat (grup & private) akan dihapus tiap tengah malam (00:00 WIB)`
    Rust: (tidak ada padanan)
  - JS: `❌ *MATI* — bot tidak menghapus chat otomatis`
    Rust: (tidak ada padanan)

### checkban (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/checkban.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/checkban.rs`

  - JS: `bannedList (config): ␀`
    Rust: `bannedList (config): []`
  - JS: `isOwner(): ␀`
    Rust: `isOwner(): ?`

### colongpp (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/colongpp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/colongpp.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `[ColongPP] Error:`
    Rust: (tidak ada padanan)

### cpuprofile (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/cpuprofile.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/cpuprofile.rs`

  - JS: `> Auto-stop: ✅ (max ␀ menit)`
    Rust: (tidak ada padanan)
  - JS: `> Berjalan: ␀s`
    Rust: (tidak ada padanan)
  - JS: `> Durasi: ␀s`
    Rust: `> Duration: ␀ms`
  - JS: `> File: `␀``
    Rust: `> Aksi: `␀``
  - JS: `> Heap: ␀/␀MB`
    Rust: (tidak ada padanan)
  - JS: `> RSS: ␀MB`
    Rust: (tidak ada padanan)
  - JS: `> Sisa: ␀s`
    Rust: (tidak ada padanan)
  - JS: `CPU: ❌ Tidak aktif`
    Rust: (tidak ada padanan)
  - JS: `CPU: 🟢 Aktif`
    Rust: (tidak ada padanan)
  - JS: `File .cpuprofile bisa diambil lewat file manager panel atau command *.get*.`
    Rust: (tidak ada padanan)
  - JS: `Gunakan *␀cpuprofile start [menit]* untuk memulai.`
    Rust: (tidak ada padanan)
  - JS: `Gunakan *␀cpuprofile status* untuk cek status`
    Rust: `> *␀cpuprofile status* — Cek status`
  - JS: `Gunakan *␀cpuprofile stop* untuk stop lebih awal`
    Rust: `> *␀cpuprofile stop* — Stop profiling lebih awal`
  - JS: `⏹️`
    Rust: (tidak ada padanan)
  - JS: `⏹️ *CPU Profiling Selesai*`
    Rust: (tidak ada padanan)
  - JS: `❌ ␀`
    Rust: (tidak ada padanan)
  - JS: `📊 *Status Profiler*`
    Rust: (tidak ada padanan)
  - JS: `🟢 *CPU Profiling Dimulai*`
    Rust: (tidak ada padanan)

### custompayment (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/custompayment.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/custompayment.rs`

  - JS: `> `␀custompayment Halo! Bayar ke {methods}``
    Rust: `> `␀custompayment Halo! Bayar ke {␀}``
  - JS: `• `{banks}` — Daftar bank`
    Rust: `• `{␀}` — Daftar bank`
  - JS: `• `{botname}` — Nama bot`
    Rust: `• `{␀}` — Nama bot`
  - JS: `• `{methods}` — Daftar e-wallet`
    Rust: `• `{␀}` — Daftar e-wallet`
  - JS: `• `{owner}` — Nama owner`
    Rust: `• `{␀}` — Nama owner`
  - JS: `• `{qris}` — Status QRIS`
    Rust: `• `{␀}` — Status QRIS`

### delenergi (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/delenergi.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/delenergi.rs`

  - JS: `> `.delenergi <jumlah> @user` - dari user`
    Rust: `> `.␀delenergi <jumlah> @user` - dari user`
  - JS: `> `.delenergi <jumlah>` - dari diri sendiri`
    Rust: `> `.␀delenergi <jumlah>` - dari diri sendiri`

### delexp (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/delexp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/delexp.rs`

  - JS: `> `.delexp <jumlah> @user` - dari user`
    Rust: `> `.␀delexp <jumlah> @user` - dari user`
  - JS: `> `.delexp <jumlah>` - dari diri sendiri`
    Rust: `> `.␀delexp <jumlah>` - dari diri sendiri`

### delkoin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/delkoin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/delkoin.rs`

  - JS: `> `.delkoin <jumlah> @user` - dari user`
    Rust: `> `.␀delkoin <jumlah> @user` - dari user`
  - JS: `> `.delkoin <jumlah>` - dari diri sendiri`
    Rust: `> `.␀delkoin <jumlah>` - dari diri sendiri`

### delplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/delplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/delplugin.rs`

  - JS: `- File: ␀`
    Rust: (tidak ada padanan)
  - JS: `- Folder: ␀`
    Rust: (tidak ada padanan)
  - JS: `- Status Unload: ␀`
    Rust: (tidak ada padanan)
  - JS: `.js`
    Rust: (tidak ada padanan)
  - JS: `Plugin tersebut sudah dihapus dan tidak aktif lagi.`
    Rust: (tidak ada padanan)
  - JS: `Proses selesai! Plugin berhasil dihapus dari sistem.`
    Rust: (tidak ada padanan)

### delsewa (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/delsewa.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/delsewa.rs`

  - JS: `Bot akan meninggalkan grup.`
    Rust: (tidak ada padanan)
  - JS: `Hubungi owner untuk sewa ulang.`
    Rust: (tidak ada padanan)
  - JS: `⛔ Grup ini telah dihapus dari whitelist sewa.`
    Rust: (tidak ada padanan)

### disableplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/disableplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/disableplugin.rs`

  - JS: `.js`
    Rust: (tidak ada padanan)
  - JS: `> Restart bot atau gunakan hot reload untuk apply.`
    Rust: (tidak ada padanan)
  - JS: `file://␀`
    Rust: (tidak ada padanan)
  - JS: `isEnabled: false`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `┃ 📁 Category: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 📄 File: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 📦 Plugin: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔴 Status: *Disabled*`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴅᴇᴛᴀɪʟ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴘʟᴜɢɪɴ ᴅɪsᴀʙʟᴇᴅ*`
    Rust: (tidak ada padanan)

### enableplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/enableplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/enableplugin.rs`

  - JS: `.js`
    Rust: (tidak ada padanan)
  - JS: `> Restart bot atau gunakan hot reload untuk apply.`
    Rust: (tidak ada padanan)
  - JS: `file://␀`
    Rust: (tidak ada padanan)
  - JS: `isEnabled: true`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `┃ 📁 Category: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 📄 File: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 📦 Plugin: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 🟢 Status: *Enabled*`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴅᴇᴛᴀɪʟ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴘʟᴜɢɪɴ ᴇɴᴀʙʟᴇᴅ*`
    Rust: (tidak ada padanan)

### eval (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/eval.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/eval.rs`

  - JS: `(async () => { ␀ })()`
    Rust: `(async () => ␀)()`
  - JS: `... (truncated)`
    Rust: (tidak ada padanan)
  - JS: ````␀````
    Rust: (tidak ada padanan)
  - JS: `┃ Type: ␀`
    Rust: `┃ Type: Error`
  - JS: `┃ ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ Success`
    Rust: (tidak ada padanan)

### exec (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/exec.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/exec.rs`

  - JS: `(async () => { ␀ })()`
    Rust: (tidak ada padanan)
  - JS: `...`
    Rust: ``...``
  - JS: `... (truncated)`
    Rust: (tidak ada padanan)
  - JS: `````
    Rust: (tidak ada padanan)
  - JS: ````␀````
    Rust: (tidak ada padanan)
  - JS: `┃ Type: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ `␀``
    Rust: (tidak ada padanan)
  - JS: `┃ ␀`
    Rust: (tidak ada padanan)
  - JS: `├┈┈⬡「 📊 *ʀᴇsᴜʟᴛ* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴄᴏᴅᴇ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `⚙️ *ᴇxᴇᴄ ʀᴇsᴜʟᴛ*`
    Rust: `⚙️ *ᴇxᴇᴄ*`
  - JS: `✅ Success`
    Rust: (tidak ada padanan)
  - JS: `❌ *Owner Only!*`
    Rust: (tidak ada padanan)
  - JS: `❌ Error`
    Rust: (tidak ada padanan)

### ganti-asset (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-asset.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-asset.rs`

  - JS: `*🎥 Video Assets:*`
    Rust: (tidak ada padanan)
  - JS: `*🎵 Audio Assets:*`
    Rust: (tidak ada padanan)
  - JS: `*📁 Other Assets:*`
    Rust: (tidak ada padanan)
  - JS: `*🔤 Font Assets:*`
    Rust: (tidak ada padanan)
  - JS: `*🖼️ Image Assets:*`
    Rust: (tidak ada padanan)
  - JS: `.jpeg`
    Rust: (tidak ada padanan)
  - JS: `.jpg`
    Rust: (tidak ada padanan)
  - JS: `.mp3`
    Rust: (tidak ada padanan)
  - JS: `.mp4`
    Rust: (tidak ada padanan)
  - JS: `.ogg`
    Rust: (tidak ada padanan)
  - JS: `.otf`
    Rust: (tidak ada padanan)
  - JS: `.png`
    Rust: (tidak ada padanan)
  - JS: `.ttf`
    Rust: (tidak ada padanan)
  - JS: `.wav`
    Rust: (tidak ada padanan)
  - JS: `> Asset *␀* membutuhkan file audio.`
    Rust: (tidak ada padanan)
  - JS: `> Asset *␀* membutuhkan file dokumen font (.ttf/.otf).`
    Rust: (tidak ada padanan)
  - JS: `> Asset *␀* membutuhkan file gambar (Image).`
    Rust: (tidak ada padanan)
  - JS: `> Asset *␀* membutuhkan file video.`
    Rust: (tidak ada padanan)
  - JS: `> Asset *␀* telah diganti ke:`
    Rust: (tidak ada padanan)
  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `> ␀. ␀`
    Rust: (tidak ada padanan)
  - JS: `_Silakan reply pesan ini dengan nomor (1-␀)_`
    Rust: (tidak ada padanan)
  - JS: `✅ *BERHASIL*`
    Rust: (tidak ada padanan)
  - JS: `❌ Format tidak sesuai!`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload media.`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengganti asset: ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Nomor tidak valid. Pilih antara 1-␀.`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada asset di config.js.`
    Rust: (tidak ada padanan)
  - JS: `📂 *PILIH ASSET YANG INGIN DIGANTI*`
    Rust: (tidak ada padanan)

### ganti-demote.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-demote.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_demote.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/demote.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar demote.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ᴅᴇᴍᴏᴛᴇ.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ᴅᴇᴍᴏᴛᴇ.ᴊᴘɢ`

### ganti-foto.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-foto.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_foto.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/foto.jpeg`
    Rust: `> File: assets/image/␀`
  - JS: `> Gambar foto.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `⏳ Sedang mengupload gambar...`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `❌ Gagal mendownload gambar`

### ganti-foto2.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-foto2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-foto2.rs`

  - JS: `⏳ Sedang mengupload gambar...`
    Rust: `❌ Gagal mengupload gambar: ␀`

### ganti-foto3.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-foto3.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_foto3.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/foto3.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar foto3.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ꜰᴏᴛᴏ3.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ꜰᴏᴛᴏ3.ᴊᴘɢ`

### ganti-games.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-games.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-games.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/games.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar games.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ɢᴀᴍᴇs.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ɢᴀᴍᴇs.ᴊᴘɢ`

### ganti-goodbye.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-goodbye.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_goodbye.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> Gambar goodbye.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `goodbye.jpeg`
    Rust: `ganti-goodbye.jpg`
  - JS: `⏳ Sedang mengupload gambar...`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: (tidak ada padanan)

### ganti-levelup.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-levelup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_levelup.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/levelup.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar levelup.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ʟᴇᴠᴇʟᴜᴘ.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ʟᴇᴠᴇʟᴜᴘ.ᴊᴘɢ`

### ganti-mp3.mp3 (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-mp3.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-mp3.rs`

  - JS: `⏳ Sedang mengupload gambar...`
    Rust: (tidak ada padanan)

### ganti-mp4.mp4 (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-mp4.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-mp4.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File foto.mp4 telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/video/foto.mp4`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply video untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload video`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload file: ␀`
    Rust: (tidak ada padanan)
  - JS: `🎬 *ɢᴀɴᴛɪ ᴍᴘ4.ᴍᴘ4*`
    Rust: `ɢᴀɴᴛɪ ᴍᴘ4.ᴍᴘ4`

### ganti-namabot (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-namabot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-namabot.rs`

  - JS: `"]([^`
    Rust: `]([^'`
  - JS: `"␀"`
    Rust: (tidak ada padanan)
  - JS: `'␀'`
    Rust: (tidak ada padanan)
  - JS: `*Penggunaan:*`
    Rust: (tidak ada padanan)
  - JS: `.ganti-namabot <nama baru>`
    Rust: (tidak ada padanan)
  - JS: `.ganti-namabot Bot Baru`
    Rust: `ganti-namabot`
  - JS: `> Nama bot diganti ke: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Nama saat ini: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Ganti nama bot di config.js`
    Rust: (tidak ada padanan)
  - JS: ``␀ganti-namabot <nama baru>``
    Rust: (tidak ada padanan)
  - JS: `config.js`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `🤖 *ɢᴀɴᴛɪ ɴᴀᴍᴀ ʙᴏᴛ*`
    Rust: (tidak ada padanan)

### ganti-namadev (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-namadev.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_namadev.rs`

  - JS: `"]([^`
    Rust: (tidak ada padanan)
  - JS: `.ganti-namadev <nama baru>`
    Rust: ``␀ganti-namadev <nama baru>``
  - JS: `.ganti-namadev Lucky Archz`
    Rust: (tidak ada padanan)
  - JS: `Ganti nama developer di config.js`
    Rust: (tidak ada padanan)
  - JS: `]*)['`
    Rust: (tidak ada padanan)
  - JS: `developer: '␀'`
    Rust: `developer:`

### ganti-namaowner (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-namaowner.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-namaowner.rs`

  - JS: `"]([^`
    Rust: (tidak ada padanan)
  - JS: `"␀"`
    Rust: (tidak ada padanan)
  - JS: `'␀'`
    Rust: (tidak ada padanan)
  - JS: `> Nama owner utama diganti ke: *␀*`
    Rust: `> Nama owner *␀* diganti ke: *␀*`
  - JS: `]*)['`
    Rust: (tidak ada padanan)

### ganti-pp-kosong.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-pp-kosong.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_pp_kosong.rs`

  - JS: `> Gambar pp-kosong.jpeg telah diganti ke URL baru:`
    Rust: `> Gambar pp-kosong.jpeg telah siap diupload ke URL baru.`
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `pp-kosong`
    Rust: (tidak ada padanan)
  - JS: `pp-kosong.jpeg`
    Rust: `ganti-pp-kosong.jpg`
  - JS: `⏳ Sedang mengupload gambar...`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: `❌ Gagal mendownload gambar: ␀`
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `❌ Gagal mendownload gambar: ␀`

### ganti-promote.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-promote.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-promote.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/promote.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar promote.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ᴘʀᴏᴍᴏᴛᴇ.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ᴘʀᴏᴍᴏᴛᴇ.ᴊᴘɢ`

### ganti-rpg.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-rpg.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-rpg.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> Gambar rpg.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `rpg.jpeg`
    Rust: (tidak ada padanan)
  - JS: `⏳ Sedang mengupload gambar...`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: (tidak ada padanan)

### ganti-rules.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-rules.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_rules.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/rules.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar rules.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ʀᴜʟᴇs.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ʀᴜʟᴇs.ᴊᴘɢ`

### ganti-store.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-store.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti_store.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/store.jpeg`
    Rust: `> File: assets/image/␀`
  - JS: `> Gambar store.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `⏳ Sedang mengupload gambar...`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `❌ Gagal mendownload gambar`

### ganti-v8.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-v8.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-v8.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/v9.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar v8.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🖼️ *ɢᴀɴᴛɪ ᴠ8.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ᴠ8.ᴊᴘɢ`

### ganti-welcome.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-welcome.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-welcome.rs`

  - JS: `⏳ Sedang mengupload gambar...`
    Rust: `❌ Gagal mengupload gambar: ␀`

### ganti-winner.jpg (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganti-winner.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganti-winner.rs`

  - JS: `> Config telah diupdate secara realtime!`
    Rust: (tidak ada padanan)
  - JS: `> File: assets/image/winner.jpeg`
    Rust: (tidak ada padanan)
  - JS: `> Gambar winner.jpeg telah diganti ke URL baru:`
    Rust: (tidak ada padanan)
  - JS: `> Kirim/reply gambar untuk mengganti`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload gambar`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengupload gambar: ␀`
    Rust: `⏳ Sedang mengupload gambar...`
  - JS: `🏆 *ɢᴀɴᴛɪ ᴡɪɴɴᴇʀ.ᴊᴘɢ*`
    Rust: `ɢᴀɴᴛɪ ᴡɪɴɴᴇʀ.ᴊᴘɢ`

### ganticode (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ganticode.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ganticode.rs`

  - JS: `"`]([^`
    Rust: (tidak ada padanan)
  - JS: `"`]+)[`
    Rust: (tidak ada padanan)
  - JS: `)`
    Rust: (tidak ada padanan)
  - JS: `) {`
    Rust: (tidak ada padanan)
  - JS: `) || code.includes(`
    Rust: (tidak ada padanan)
  - JS: `));`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `.filter((d) => d.isDirectory())`
    Rust: (tidak ada padanan)
  - JS: `.map((d) => d.name);`
    Rust: (tidak ada padanan)
  - JS: `.readdirSync(pluginsDir, { withFileTypes: true })`
    Rust: (tidak ada padanan)
  - JS: `.replace(/[:.]/g,`
    Rust: (tidak ada padanan)
  - JS: `.slice(0, 19);`
    Rust: (tidak ada padanan)
  - JS: `.toISOString()`
    Rust: (tidak ada padanan)
  - JS: `;`
    Rust: (tidak ada padanan)
  - JS: `]/i);`
    Rust: (tidak ada padanan)
  - JS: `` +`
    Rust: (tidak ada padanan)
  - JS: ``- .ganticode (untuk deteksi otomatis)`
    Rust: `- .ganticode (untuk deteksi otomatis)`
  - JS: ``- .ganticode <nama file> (untuk nama kustom)`
    Rust: `- .ganticode <nama file> (untuk nama kustom)`
  - JS: ``- .ganticode <nama file> <folder> (untuk kustom nama dan folder)`
    Rust: `- .ganticode <nama file> <folder> (untuk kustom nama dan folder)`
  - JS: ``- File: ␀.js`
    Rust: `- File: ␀.js`
  - JS: ``- Folder: ␀`
    Rust: `- Folder: ␀`
  - JS: ``- Status Reload: ${reloadResult.success ?`
    Rust: (tidak ada padanan)
  - JS: ``- Ukuran: ␀ bytes`
    Rust: `- Ukuran: ␀ bytes`
  - JS: ``;`
    Rust: (tidak ada padanan)
  - JS: ``Halo *␀*, sepertinya kamu belum mereply kode plugin barunya.`
    Rust: `Halo *␀*, sepertinya kamu belum mereply kode plugin barunya.`
  - JS: ``Maaf *␀*, aku tidak bisa mendeteksi nama pluginnya. Silakan gunakan perintah dengan format .ganticode <nama file>.``
    Rust: `Maaf *␀*, aku tidak bisa mendeteksi nama pluginnya. Silakan gunakan perintah dengan format .ganticode <nama file>.`
  - JS: ``Maaf *␀*, proses gagal karena kode bukan format plugin yang valid. Pastikan ada export dan config di dalamnya.``
    Rust: `Maaf *␀*, proses gagal karena kode bukan format plugin yang valid. Pastikan ada export dan config di dalamnya.`
  - JS: ``Proses selesai! Kode plugin berhasil ${isNewFile ?`
    Rust: `Proses selesai! Kode plugin berhasil ␀.`
  - JS: ``Silakan reply kode plugin yang baru dengan perintah:`
    Rust: `Silakan reply kode plugin yang baru dengan perintah:`
  - JS: ``Tenang saja, kode yang lama akan otomatis dibackup sebelum diganti.``
    Rust: `Tenang saja, kode yang lama akan otomatis dibackup sebelum diganti.`
  - JS: ``]+)['`
    Rust: (tidak ada padanan)
  - JS: ``]/i);`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `await m.react(`
    Rust: (tidak ada padanan)
  - JS: `await m.reply(te(m.prefix, m.command, m.pushName));`
    Rust: (tidak ada padanan)
  - JS: `backupPath = path.join(backupDir, `␀_␀.js`);`
    Rust: (tidak ada padanan)
  - JS: `code = (await quoted.download()).toString();`
    Rust: (tidak ada padanan)
  - JS: `const args = m.args;`
    Rust: (tidak ada padanan)
  - JS: `const backupDir = path.join(process.cwd(),`
    Rust: (tidak ada padanan)
  - JS: `const baseName = file.replace(`
    Rust: (tidak ada padanan)
  - JS: `const categoryMatch = code.match(/category:s*['"`
    Rust: (tidak ada padanan)
  - JS: `const existing = findPluginFile(pluginsDir, fileName);`
    Rust: (tidak ada padanan)
  - JS: `const extracted = extractPluginInfo(code);`
    Rust: (tidak ada padanan)
  - JS: `const files = fs.readdirSync(folderPath).filter((f) => f.endsWith(`
    Rust: (tidak ada padanan)
  - JS: `const folderPath = path.join(pluginsDir, folder);`
    Rust: (tidak ada padanan)
  - JS: `const folderPath = path.join(pluginsDir, targetFolder);`
    Rust: (tidak ada padanan)
  - JS: `const folders = fs`
    Rust: (tidak ada padanan)
  - JS: `const hasConfig = code.includes(`
    Rust: (tidak ada padanan)
  - JS: `const hasExport = code.includes(`
    Rust: (tidak ada padanan)
  - JS: `const pluginsDir = path.join(process.cwd(),`
    Rust: (tidak ada padanan)
  - JS: `const quoted = m.quoted;`
    Rust: (tidak ada padanan)
  - JS: `const relBackup = path.relative(process.cwd(), backupPath);`
    Rust: (tidak ada padanan)
  - JS: `const timestamp = new Date()`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `fileName = fileName.toLowerCase().replace(/[^a-z0-9-_]/g,`
    Rust: (tidak ada padanan)
  - JS: `filePath = existing.path;`
    Rust: (tidak ada padanan)
  - JS: `filePath = path.join(folderPath, `␀.js`);`
    Rust: (tidak ada padanan)
  - JS: `folderName = folderName.toLowerCase().replace(/[^a-z0-9-_]/g,`
    Rust: (tidak ada padanan)
  - JS: `for (const file of files) {`
    Rust: (tidak ada padanan)
  - JS: `for (const folder of folders) {`
    Rust: (tidak ada padanan)
  - JS: `fs.copyFileSync(filePath, backupPath);`
    Rust: (tidak ada padanan)
  - JS: `fs.mkdirSync(backupDir, { recursive: true });`
    Rust: (tidak ada padanan)
  - JS: `fs.mkdirSync(folderPath, { recursive: true });`
    Rust: (tidak ada padanan)
  - JS: `fs.writeFileSync(filePath, code);`
    Rust: (tidak ada padanan)
  - JS: `function findPluginFile(pluginsDir, name) {`
    Rust: (tidak ada padanan)
  - JS: `if (`
    Rust: (tidak ada padanan)
  - JS: `if (!code || code.length < 50) {`
    Rust: (tidak ada padanan)
  - JS: `if (!fileName) {`
    Rust: (tidak ada padanan)
  - JS: `if (!folderName) folderName =`
    Rust: (tidak ada padanan)
  - JS: `if (!fs.existsSync(backupDir)) {`
    Rust: (tidak ada padanan)
  - JS: `if (!fs.existsSync(folderPath)) {`
    Rust: (tidak ada padanan)
  - JS: `if (!hasExport || !hasConfig) {`
    Rust: (tidak ada padanan)
  - JS: `if (!isNewFile) {`
    Rust: (tidak ada padanan)
  - JS: `if (!quoted) {`
    Rust: (tidak ada padanan)
  - JS: `if (backupPath) {`
    Rust: (tidak ada padanan)
  - JS: `if (baseName.toLowerCase() === name.toLowerCase()) {`
    Rust: (tidak ada padanan)
  - JS: `if (categoryMatch) info.category = categoryMatch[1];`
    Rust: (tidak ada padanan)
  - JS: `if (existing) {`
    Rust: (tidak ada padanan)
  - JS: `if (nameMatch) info.name = nameMatch[1];`
    Rust: (tidak ada padanan)
  - JS: `isNewFile = true;`
    Rust: (tidak ada padanan)
  - JS: `let backupPath = null;`
    Rust: (tidak ada padanan)
  - JS: `let code = quoted.text || quoted.body ||`
    Rust: (tidak ada padanan)
  - JS: `let fileName = args[0] || extracted.name;`
    Rust: (tidak ada padanan)
  - JS: `let filePath;`
    Rust: (tidak ada padanan)
  - JS: `let folderName = args[1] || extracted.category;`
    Rust: (tidak ada padanan)
  - JS: `let isNewFile = false;`
    Rust: (tidak ada padanan)
  - JS: `let oldSize = 0;`
    Rust: (tidak ada padanan)
  - JS: `let reloadResult = { success: false };`
    Rust: (tidak ada padanan)
  - JS: `let replyText =`
    Rust: (tidak ada padanan)
  - JS: `let targetFolder;`
    Rust: (tidak ada padanan)
  - JS: `oldSize = fs.statSync(filePath).size;`
    Rust: (tidak ada padanan)
  - JS: `quoted.filename?.endsWith(`
    Rust: (tidak ada padanan)
  - JS: `quoted.mimetype ===`
    Rust: (tidak ada padanan)
  - JS: `reloadResult = (await hotReloadPlugin(filePath)) || { success: true };`
    Rust: (tidak ada padanan)
  - JS: `replyText +=`
    Rust: (tidak ada padanan)
  - JS: `replyText += `- Ukuran Lama: ␀ bytes`
    Rust: `- Ukuran Lama: ␀ bytes`
  - JS: `replyText += `File lama sudah dibackup dengan aman di lokasi berikut:`
    Rust: `File lama sudah dibackup dengan aman di lokasi berikut:`
  - JS: `replyText += `Plugin sudah aktif dan siap digunakan, silakan dicoba ya!`;`
    Rust: `Plugin sudah aktif dan siap digunakan, silakan dicoba ya!`
  - JS: `return info;`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`Maaf *␀*, proses gagal karena file tidak dapat diunduh.`);`
    Rust: `Maaf *␀*, proses gagal karena file tidak dapat diunduh.`
  - JS: `return m.reply(`Maaf *␀*, proses gagal karena kode terlalu pendek atau tidak valid.`);`
    Rust: `Maaf *␀*, proses gagal karena kode terlalu pendek atau tidak valid.`
  - JS: `return m.reply(`Maaf *␀*, proses gagal karena nama file tidak valid.`);`
    Rust: `Maaf *␀*, proses gagal karena nama file tidak valid.`
  - JS: `return m.reply(replyText);`
    Rust: (tidak ada padanan)
  - JS: `return null;`
    Rust: (tidak ada padanan)
  - JS: `return { folder, file, path: path.join(folderPath, file) };`
    Rust: (tidak ada padanan)
  - JS: `targetFolder = existing.folder;`
    Rust: (tidak ada padanan)
  - JS: `targetFolder = folderName;`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `||`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (error) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {}`
    Rust: (tidak ada padanan)
  - JS: `} else {`
    Rust: (tidak ada padanan)
  - JS: `}.`
    Rust: (tidak ada padanan)

### gantiscraper (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/gantiscraper.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/gantiscraper.rs`

  - JS: `- Plugin Berhasil Reload: ␀`
    Rust: (tidak ada padanan)
  - JS: `- Plugin Gagal Reload: ␀`
    Rust: (tidak ada padanan)
  - JS: `- Plugin Terdeteksi: ␀`
    Rust: (tidak ada padanan)
  - JS: `- Scraper: ␀`
    Rust: (tidak ada padanan)
  - JS: `- `␀``
    Rust: ``␀``
  - JS: `- `␀`␀)` : ""}`
    Rust: (tidak ada padanan)
  - JS: `Scraper sudah aktif dan siap digunakan!`
    Rust: (tidak ada padanan)
  - JS: `\$&`
    Rust: (tidak ada padanan)
  - JS: `scraper[\/].*␀\.js`
    Rust: (tidak ada padanan)
  - JS: `src[\/]scraper[\/]␀\.js`
    Rust: (tidak ada padanan)
  - JS: `ℹ️ Tidak ada plugin yang langsung mengimpor scraper ini.`
    Rust: (tidak ada padanan)
  - JS: `␀.js`
    Rust: `.js`
  - JS: `␀?t=␀`
    Rust: (tidak ada padanan)
  - JS: `␀_␀.js`
    Rust: `␀/␀_␀.js`
  - JS: `│ ␀. `␀.js` (␀ KB)`
    Rust: `│ ␀. `␀` (␀ KB)`
  - JS: `⚠️ *Plugin Gagal Reload:*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ Sebagian reload gagal, restart bot mungkin diperlukan`
    Rust: (tidak ada padanan)
  - JS: `✅ *Plugin Reloaded:*`
    Rust: (tidak ada padanan)
  - JS: `🔄 *Hot Reload:*`
    Rust: `Hot Reload`

### get (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/get.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/get.rs`

  - JS: `']([^:]+):s*([^`
    Rust: (tidak ada padanan)
  - JS: `'][^`
    Rust: (tidak ada padanan)
  - JS: `*/*`
    Rust: (tidak ada padanan)
  - JS: `... (truncated, full size:`
    Rust: (tidak ada padanan)
  - JS: `.get https://api.example.com --method POST --json {"key":"value"} --header "Authorization: Bearer token"`
    Rust: (tidak ada padanan)
  - JS: `> .get https://api.example.com`
    Rust: (tidak ada padanan)
  - JS: `> .get https://api.example.com --auth user:pass -v`
    Rust: (tidak ada padanan)
  - JS: `> .get https://api.example.com --header "Authorization: Bearer token"`
    Rust: (tidak ada padanan)
  - JS: `> .get https://api.example.com --method PUT --json {"id":1}`
    Rust: `.get https://api.example.com --method POST --json ␀`
  - JS: `> .get https://api.example.com --post --json {"key":"val"}`
    Rust: `.get https://api.example.com --method POST --json ␀`
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `Advanced HTTP request tool (Owner Only)`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `]+)["`
    Rust: (tidak ada padanan)
  - JS: `]+["`
    Rust: (tidak ada padanan)
  - JS: ``Examples:``
    Rust: (tidak ada padanan)
  - JS: ````␀````
    Rust: (tidak ada padanan)
  - JS: `content-type`
    Rust: (tidak ada padanan)
  - JS: `response_␀.json`
    Rust: (tidak ada padanan)
  - JS: `response_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `┃ ⏱️ Time: ␀ms`
    Rust: (tidak ada padanan)
  - JS: `┃ ␀ Status: ␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ ␀: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--auth user:pass` — Basic auth`
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--header "Key: Value"` — Custom header`
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--json <body>` — JSON body`
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--method <GET|POST|PUT|PATCH|DELETE>``
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--post` — Shortcut for --method POST`
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--timeout <ms>` — Request timeout`
    Rust: (tidak ada padanan)
  - JS: `┃ ◦ `--verbose` / `-v` — Show response headers`
    Rust: (tidak ada padanan)
  - JS: `┃ 📄 Type: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📦 Size: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📨 Method: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 INFO 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 OPTIONS 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📨 RESPONSE HEADERS 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `⚠️`
    Rust: (tidak ada padanan)
  - JS: `❌ *Owner Only!*`
    Rust: (tidak ada padanan)
  - JS: `❌ *REQUEST FAILED*`
    Rust: (tidak ada padanan)
  - JS: `❌ Invalid JSON body: ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Invalid URL`
    Rust: (tidak ada padanan)
  - JS: `❌ Invalid method: ␀. Valid: ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Localhost / internal / metadata address blocked`
    Rust: (tidak ada padanan)
  - JS: `🌐 *HTTP REQUEST TOOL*`
    Rust: (tidak ada padanan)
  - JS: `🌐 *HTTP RESPONSE*`
    Rust: (tidak ada padanan)
  - JS: `📄 Pretty JSON dikirim sebagai file (␀)`
    Rust: (tidak ada padanan)
  - JS: `📎 Full response dikirim sebagai file`
    Rust: (tidak ada padanan)
  - JS: `🕕 ␀ ␀ ...`
    Rust: (tidak ada padanan)

### getplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/getplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/getplugin.rs`

  - JS: `"`]([^`
    Rust: (tidak ada padanan)
  - JS: `.getplugin <nama plugin>`
    Rust: `- .getplugin <nama plugin>`
  - JS: `.getplugin menu`
    Rust: `- .getplugin menu`
  - JS: `.js`
    Rust: (tidak ada padanan)
  - JS: `Dapatkan source code plugin`
    Rust: (tidak ada padanan)
  - JS: `]/g);`
    Rust: `]);`
  - JS: ``]+)['`
    Rust: (tidak ada padanan)
  - JS: `a.replace(/['"`
    Rust: (tidak ada padanan)
  - JS: `const cleanAliases = aliases.map((a) =>`
    Rust: (tidak ada padanan)
  - JS: `if (aliases) {`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `␀ bytes`
    Rust: (tidak ada padanan)
  - JS: `␀.js`
    Rust: (tidak ada padanan)
  - JS: `❌ *Owner Only!*`
    Rust: (tidak ada padanan)
  - JS: `📁 *␀/␀*`
    Rust: (tidak ada padanan)

### goodbyeall (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/goodbyeall.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/goodbyeall.rs`

  - JS: `> Goodbye dinonaktifkan di semua grup.`
    Rust: (tidak ada padanan)
  - JS: `> Member yang keluar akan dikirim pesan perpisahan!`
    Rust: (tidak ada padanan)
  - JS: `[GoodbyeAll] Error:`
    Rust: (tidak ada padanan)
  - JS: `┃ ✅ Goodbye: *AKTIF*`
    Rust: (tidak ada padanan)
  - JS: `┃ ❌ Goodbye: *NONAKTIF*`
    Rust: (tidak ada padanan)
  - JS: `┃ 🌐 Total Grup: *␀*`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📊 *ʀᴇsᴜʟᴛ* 」`
    Rust: (tidak ada padanan)
  - JS: `✅ *ɢᴏᴏᴅʙʏᴇ ɢʟᴏʙᴀʟ ᴏɴ*`
    Rust: `👋 *ɢᴏᴏᴅʙʏᴇ ɢʟᴏʙᴀʟ*`
  - JS: `❌ *ɢᴏᴏᴅʙʏᴇ ɢʟᴏʙᴀʟ ᴏꜰꜰ*`
    Rust: `👋 *ɢᴏᴏᴅʙʏᴇ ɢʟᴏʙᴀʟ*`

### hapusdata (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/hapusdata.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/hapusdata.rs`

  - JS: `> ␀: *␀* data (␀)`
    Rust: `> ␀: *␀* data (␀ KB)`
  - JS: `␀ KB`
    Rust: (tidak ada padanan)
  - JS: `⭐ Premium`
    Rust: (tidak ada padanan)
  - JS: `👑 Owner`
    Rust: (tidak ada padanan)
  - JS: `👥 Groups`
    Rust: (tidak ada padanan)
  - JS: `👥 Users`
    Rust: (tidak ada padanan)
  - JS: `📊 Stats`
    Rust: (tidak ada padanan)
  - JS: `🤝 Partner`
    Rust: (tidak ada padanan)

### hapussaluran (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/hapussaluran.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/hapussaluran.rs`

  - JS: `❌ Gagal menghapus saluran: ␀`
    Rust: (tidak ada padanan)

### heapsnap (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/heapsnap.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/heapsnap.rs`

  - JS: `> Alasan: ␀`
    Rust: (tidak ada padanan)
  - JS: `> File: `␀``
    Rust: (tidak ada padanan)
  - JS: `> Label: ␀`
    Rust: `> Label: `␀``
  - JS: `> RSS saat ambil: ␀MB`
    Rust: (tidak ada padanan)
  - JS: `> Size: ␀MB`
    Rust: (tidak ada padanan)
  - JS: `Coba lagi saat RSS lebih rendah (misal setelah gc atau sepi chat).`
    Rust: (tidak ada padanan)
  - JS: `File .heapsnapshot bisa diambil lewat file manager panel atau command *.get*.`
    Rust: (tidak ada padanan)
  - JS: `Heap Total: ␀MB`
    Rust: (tidak ada padanan)
  - JS: `Proyeksi puncak: ␀MB`
    Rust: (tidak ada padanan)
  - JS: `Proyeksi puncak: ␀MB / ␀ (ceiling)`
    Rust: `Proyeksi puncak: tidak terbaca / ␀ (ceiling)`
  - JS: `RSS saat ini: ␀MB`
    Rust: (tidak ada padanan)
  - JS: `RSS: ␀MB / ␀ (ceiling)`
    Rust: (tidak ada padanan)
  - JS: `Snapshot: ␀`
    Rust: (tidak ada padanan)
  - JS: `⚠️ Ambil minimal 2 snapshot dengan jeda 2-6 jam untuk perbandingan (diff).`
    Rust: (tidak ada padanan)
  - JS: `❌ *Heap Snapshot DIBATALKAN (Safety Guard)*`
    Rust: (tidak ada padanan)
  - JS: `❌ ␀`
    Rust: (tidak ada padanan)
  - JS: `💾 *Heap Snapshot Berhasil*`
    Rust: (tidak ada padanan)

### join (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/join.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/join.rs`

  - JS: `*␀.* ␀ — ⚠️ Sudah member`
    Rust: `*.␀* ␀ — ⚠️ Sudah member`
  - JS: `*␀.* ␀ — ✅ Berhasil join`
    Rust: `*.␀* ␀ — ✅ Berhasil join`
  - JS: `*␀.* ␀... — ❌ ␀`
    Rust: `*.␀* ␀... — ❌ ␀`
  - JS: `Link tidak valid`
    Rust: (tidak ada padanan)

### leave (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/leave.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/leave.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Bot akan keluar dari grup ini.`
    Rust: `Bot keluar dari grup`
  - JS: `> Bot telah keluar dari: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Grup tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `> Terima kasih sudah menggunakan bot!`
    Rust: (tidak ada padanan)
  - JS: `> Tidak dapat mengambil info grup dari link`
    Rust: (tidak ada padanan)
  - JS: `Grup ini`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ* 」`
    Rust: `╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴜ* 」`
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙᴇʀʜᴀsɪʟ ᴋᴇʟᴜᴀʀ*`
    Rust: (tidak ada padanan)
  - JS: `👋 *ɢᴏᴏᴅʙʏᴇ*`
    Rust: (tidak ada padanan)

### listjadibot (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/listjadibot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/listjadibot.rs`

  - JS: `> `␀listjadibotaktif` — Detail aktif`
    Rust: (tidak ada padanan)
  - JS: `> `␀stopalljadibot` — Stop semua`
    Rust: (tidak ada padanan)
  - JS: `> `␀stopdandeletejadibot @user` — Hapus session`
    Rust: (tidak ada padanan)
  - JS: `> ⚫ Offline: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> 📊 Total: *␀* session`
    Rust: (tidak ada padanan)
  - JS: `> 🟢 Aktif: *␀*`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀.* @␀ — _␀_`
    Rust: (tidak ada padanan)
  - JS: `␀listjadibotaktif`
    Rust: `.listjadibot`
  - JS: `␀stopalljadibot`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada session jadibot tersimpan`
    Rust: `Lihat semua session jadibot yang tersimpan`
  - JS: `🛑 Stop Semua`
    Rust: (tidak ada padanan)
  - JS: `🟢 Lihat Aktif`
    Rust: (tidak ada padanan)
  - JS: `🤖 *ᴅᴀꜰᴛᴀʀ ᴊᴀᴅɪʙᴏᴛ*`
    Rust: (tidak ada padanan)

### listjadibotaktif (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/listjadibotaktif.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/listjadibotaktif.rs`

  - JS: `*␀.* 🟢 @␀`
    Rust: (tidak ada padanan)
  - JS: `> `␀stopalljadibot` — Hentikan semua`
    Rust: (tidak ada padanan)
  - JS: `> 📊 Total: *␀* bot aktif`
    Rust: (tidak ada padanan)
  - JS: `Lihat jadibot yang sedang aktif dengan detail`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *␀* — 👤 @␀`
    Rust: (tidak ada padanan)
  - JS: `␀stopalljadibot`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada jadibot yang aktif saat ini`
    Rust: (tidak ada padanan)
  - JS: `🛑 Stop Semua`
    Rust: (tidak ada padanan)
  - JS: `🟢 *ᴊᴀᴅɪʙᴏᴛ ᴀᴋᴛɪꜰ*`
    Rust: (tidak ada padanan)

### mutechat (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/mutechat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/mutechat.rs`

  - JS: `❌ Gagal: ␀`
    Rust: (tidak ada padanan)

### onlyadmin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/onlyadmin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/onlyadmin.rs`

  - JS: `┃ ❌ Member biasa di grup`
    Rust: `┃ ❌ Member biasa`
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)

### onlythisgrup (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/onlythisgrup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/onlythisgrup.rs`

  - JS: `?.admin !== null`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendapatkan metadata grup.`
    Rust: (tidak ada padanan)

### payment (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/payment.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/payment.rs`

  - JS: `• *␀*: ␀ (␀)`
    Rust: `• *␀*: ␀ (a/n: ␀)`

### pinchat (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/pinchat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/pinchat.rs`

  - JS: `> Target: ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal: ␀`
    Rust: (tidak ada padanan)
  - JS: `📌 *ᴄʜᴀᴛ ᴅɪᴘɪɴ*`
    Rust: (tidak ada padanan)
  - JS: `📍 *ᴘɪɴ ᴅɪʜᴀᴘᴜs*`
    Rust: (tidak ada padanan)

### addprefix (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/prefix.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/prefix.rs`

  - JS: `.addprefix ! # $`
    Rust: `.addprefix ! # $ 😚`
  - JS: `prefix.json`
    Rust: (tidak ada padanan)
  - JS: `┃ ␀. `␀``
    Rust: `┃ ␀.	`␀``

### ptvch (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/ptvch.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/ptvch.rs`

  - JS: `120363404849776664@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Kirim *video* atau *balas video* lalu ketik:`
    Rust: (tidak ada padanan)
  - JS: `> Video berhasil dikirim ke channel sebagai PTV.`
    Rust: (tidak ada padanan)
  - JS: `> `␀ptvch``
    Rust: (tidak ada padanan)
  - JS: `Kirim video sebagai PTV ke channel`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `✅ *sᴜᴋsᴇs*`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal download video dari quoted.`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal download video.`
    Rust: (tidak ada padanan)
  - JS: `🕕 *ᴍᴇɴɢɪʀɪᴍ ᴘᴛᴠ ᴋᴇ ᴄʜᴀɴɴᴇʟ...*`
    Rust: (tidak ada padanan)

### public (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/public.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/public.rs`

  - JS: `[Mode] Changed to PUBLIC by ␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `[Public Command Error]`
    Rust: (tidak ada padanan)

### renewsewa (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/renewsewa.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/renewsewa.rs`

  - JS: `D MMMM YYYY HH:mm`
    Rust: (tidak ada padanan)
  - JS: `Expired baru: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Grup: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Gunakan *␀addsewa* untuk menambahkan`
    Rust: (tidak ada padanan)
  - JS: `Tambahan: *␀*`
    Rust: (tidak ada padanan)
  - JS: `chat.whatsapp.com/`
    Rust: (tidak ada padanan)
  - JS: `✅ *SEWA DIPERPANJANG*`
    Rust: (tidak ada padanan)
  - JS: `❌ Grup ini sudah Permanent, tidak perlu diperpanjang`
    Rust: (tidak ada padanan)
  - JS: `❌ Grup tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `❌ Grup tidak terdaftar`
    Rust: (tidak ada padanan)
  - JS: `📢 Sewa bot telah diperpanjang!`
    Rust: (tidak ada padanan)

### resetdb (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/resetdb.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/resetdb.rs`

  - JS: `db.json`
    Rust: (tidak ada padanan)
  - JS: `┃ ⚔️ Clans: ␀`
    Rust: `┃ ⚔️ Clans: 0`
  - JS: `┃ 👤 Users: ␀`
    Rust: `┃ 👤 Users: 0`
  - JS: `┃ 👥 Groups: ␀`
    Rust: `┃ 👥 Groups: 0`

### restart (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/restart.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/restart.rs`

  - JS: `[Restart] Command triggered by:`
    Rust: (tidak ada padanan)
  - JS: `[Restart] Initiating graceful restart...`
    Rust: (tidak ada padanan)
  - JS: `[Restart] New process spawned, exiting current process...`
    Rust: (tidak ada padanan)
  - JS: `cmd.exe`
    Rust: (tidak ada padanan)

### sampah (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/sampah.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/sampah.rs`

  - JS: `🗑️`
    Rust: (tidak ada padanan)

### savedb (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/savedb.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/savedb.rs`

  - JS: `YYYY-MM-DD_HH-mm-ss`
    Rust: (tidak ada padanan)
  - JS: `db.json`
    Rust: `.json`
  - JS: `db_backup_␀.json`
    Rust: `db_backup_␀-␀-␀_␀-␀-␀.json`
  - JS: `┃ ⏰ Time: `␀``
    Rust: `┃ ⏰ Time: `␀-␀-␀``
  - JS: `┃ 📅 Date: `␀``
    Rust: `┃ 📅 Date: `␀/␀/␀``

### savekontak (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/savekontak.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/savekontak.rs`

  - JS: `␀ Kontak`
    Rust: (tidak ada padanan)

### schedule (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/schedule.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/schedule.rs`

  - JS: `*Contoh:*`
    Rust: (tidak ada padanan)
  - JS: `*Edit jadwal:*`
    Rust: (tidak ada padanan)
  - JS: `*Filter kategori:*`
    Rust: (tidak ada padanan)
  - JS: `*Format lama masih didukung:*`
    Rust: (tidak ada padanan)
  - JS: `*Format utama:*`
    Rust: (tidak ada padanan)
  - JS: `*Mode repeat opsional:*`
    Rust: (tidak ada padanan)
  - JS: `*Quick preset:*`
    Rust: (tidak ada padanan)
  - JS: `*Subcommand:*`
    Rust: (tidak ada padanan)
  - JS: `*Target opsional:*`
    Rust: (tidak ada padanan)
  - JS: `Berangkat sekolah`
    Rust: (tidak ada padanan)
  - JS: `Bikin reminder atau jadwal bebas dengan text custom`
    Rust: (tidak ada padanan)
  - JS: `Bisa dipakai untuk sekolah, pelajaran, kerja, meeting, ngedate, turnamen, atau agenda apa aja.`
    Rust: (tidak ada padanan)
  - JS: `Cek roster, room, koneksi, dan standby sebelum match dimulai.`
    Rust: (tidak ada padanan)
  - JS: `Fitur ini buat bikin jadwal atau reminder bebas.`
    Rust: (tidak ada padanan)
  - JS: `Gunakan `.schedule kategori <nama>` untuk filter list jadwal.`
    Rust: (tidak ada padanan)
  - JS: `Gunakan `.schedule list` untuk lihat semua jadwal aktif.`
    Rust: (tidak ada padanan)
  - JS: `Jadwal date`
    Rust: (tidak ada padanan)
  - JS: `Mandi, sarapan, cek buku, dan berangkat tepat waktu.`
    Rust: (tidak ada padanan)
  - JS: `Mulai kerja`
    Rust: (tidak ada padanan)
  - JS: `Pakai:`
    Rust: (tidak ada padanan)
  - JS: `Persiapan turnamen`
    Rust: (tidak ada padanan)
  - JS: `Pesan custom:`
    Rust: (tidak ada padanan)
  - JS: `Pesan yang dikirim akan mengikuti *text custom* buatan owner.`
    Rust: (tidak ada padanan)
  - JS: `Siap-siap, cek lokasi, dan datang tepat waktu.`
    Rust: (tidak ada padanan)
  - JS: `Siapkan device, cek task, dan mulai kerja tepat waktu.`
    Rust: (tidak ada padanan)
  - JS: `Text custom:`
    Rust: (tidak ada padanan)
  - JS: ``.schedule add 06:30 | sekolah | berangkat sekolah | mandi, sarapan, cek buku | me | repeat``
    Rust: (tidak ada padanan)
  - JS: ``.schedule add 08:00 628xxx repeat Selamat pagi tim``
    Rust: (tidak ada padanan)
  - JS: ``.schedule add 12:00 | kerja | standup team | masuk room meeting jam 12 tepat | here | repeat``
    Rust: (tidak ada padanan)
  - JS: ``.schedule add 19:00 | date | dinner malam | jangan lupa datang rapi dan tepat waktu | me``
    Rust: (tidak ada padanan)
  - JS: ``.schedule add 20:00 | turnamen | scrim malam | room dibuka 15 menit sebelum mulai | here``
    Rust: (tidak ada padanan)
  - JS: ``.schedule add <HH:MM> | <kategori> | <judul> | <pesan> | [target] | [repeat]``
    Rust: (tidak ada padanan)
  - JS: ``.schedule edit <id> <HH:MM> | <kategori> | <judul> | <pesan> | [target] | [repeat]``
    Rust: (tidak ada padanan)
  - JS: ``.schedule edit <id> time=08:00 | text=angkat rapat | repeat=off``
    Rust: (tidak ada padanan)
  - JS: ``.schedule kategori sekolah``
    Rust: (tidak ada padanan)
  - JS: ``.schedule kategori``
    Rust: (tidak ada padanan)
  - JS: ``.schedule preset kerja 09:00 | standup pagi | masuk room meeting | here | repeat``
    Rust: (tidak ada padanan)
  - JS: ``.schedule preset list``
    Rust: (tidak ada padanan)
  - JS: ``.schedule preset sekolah 06:30``
    Rust: `.schedule preset sekolah 06:30`
  - JS: `here (chat ini)`
    Rust: (tidak ada padanan)
  - JS: `here (private chat ini)`
    Rust: (tidak ada padanan)
  - JS: `sched_␀`
    Rust: (tidak ada padanan)
  - JS: `• *␀*`
    Rust: (tidak ada padanan)
  - JS: `• `.schedule del <id>``
    Rust: (tidak ada padanan)
  - JS: `• `.schedule detail <id>``
    Rust: (tidak ada padanan)
  - JS: `• `.schedule edit <id> ...``
    Rust: (tidak ada padanan)
  - JS: `• `.schedule kategori <nama>``
    Rust: (tidak ada padanan)
  - JS: `• `.schedule list``
    Rust: (tidak ada padanan)
  - JS: `• `.schedule preset <nama> <HH:MM>``
    Rust: (tidak ada padanan)
  - JS: `• `.schedule status``
    Rust: (tidak ada padanan)
  - JS: `• `628xxx@s.whatsapp.net` = kirim ke nomor tertentu`
    Rust: (tidak ada padanan)
  - JS: `• `daily``
    Rust: (tidak ada padanan)
  - JS: `• `harian``
    Rust: (tidak ada padanan)
  - JS: `• `here` = kirim ke chat ini`
    Rust: (tidak ada padanan)
  - JS: `• `me` = kirim ke chat owner sendiri`
    Rust: (tidak ada padanan)
  - JS: `• `repeat``
    Rust: (tidak ada padanan)
  - JS: `• ␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `⏰ Waktu: ␀ WIB`
    Rust: (tidak ada padanan)
  - JS: `⏰ ␀ WIB`
    Rust: (tidak ada padanan)
  - JS: `␀)*`}`
    Rust: (tidak ada padanan)
  - JS: `␀...`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: (tidak ada padanan)
  - JS: `␀@s.whatsapp.net`
    Rust: (tidak ada padanan)
  - JS: `⚡ *QUICK PRESET SCHEDULE*`
    Rust: (tidak ada padanan)
  - JS: `⚡ Preset: ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *JADWAL BERHASIL DIBUAT*`
    Rust: (tidak ada padanan)
  - JS: `✅ *JADWAL BERHASIL DIUPDATE*`
    Rust: (tidak ada padanan)
  - JS: `✅ *PRESET JADWAL BERHASIL DIBUAT*`
    Rust: (tidak ada padanan)
  - JS: `✅ Jadwal `␀` dihapus␀` : ""}`
    Rust: (tidak ada padanan)
  - JS: `❌ Field edit tidak dikenal. Pakai: time, kategori, judul, text, target, repeat`
    Rust: (tidak ada padanan)
  - JS: `❌ Format baru minimal: `.schedule add <HH:MM> | <kategori> | <judul> | <pesan>``
    Rust: (tidak ada padanan)
  - JS: `❌ Format edit parsial harus `field=value`, contoh: `time=08:00 | text=angkat rapat``
    Rust: (tidak ada padanan)
  - JS: `❌ Format edit: `.schedule edit <id> <HH:MM> | <kategori> | <judul> | <pesan> | [target] | [repeat]` atau `.schedule edit <id> time=08:00 | text=... | repeat=off``
    Rust: (tidak ada padanan)
  - JS: `❌ Format waktu edit salah. Gunakan HH:MM atau HH.MM`
    Rust: (tidak ada padanan)
  - JS: `❌ Format waktu salah. Gunakan HH:MM atau HH.MM, contoh: 08:00`
    Rust: (tidak ada padanan)
  - JS: `❌ Format: `.schedule add <HH:MM> | <kategori> | <judul> | <pesan> | [target] | [repeat]``
    Rust: (tidak ada padanan)
  - JS: `❌ Format: `.schedule del <id>``
    Rust: (tidak ada padanan)
  - JS: `❌ Format: `.schedule detail <id>``
    Rust: (tidak ada padanan)
  - JS: `❌ Format: `.schedule edit <id> ...``
    Rust: (tidak ada padanan)
  - JS: `❌ Format: `.schedule preset <nama> <HH:MM> [| <judul> | <pesan> | [target] | [repeat]]``
    Rust: (tidak ada padanan)
  - JS: `❌ Isi reminder tidak boleh kosong`
    Rust: (tidak ada padanan)
  - JS: `❌ Jadwal `␀` tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `❌ Jadwal dengan ID `␀` tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `❌ Nilai repeat harus salah satu: repeat, daily, harian, once, sekali, off`
    Rust: (tidak ada padanan)
  - JS: `❌ Nilai untuk field `␀` tidak boleh kosong`
    Rust: (tidak ada padanan)
  - JS: `❌ Pesan jadwal tidak boleh kosong`
    Rust: (tidak ada padanan)
  - JS: `❌ Preset tidak dikenal. Gunakan `.schedule preset list` untuk lihat preset yang tersedia.`
    Rust: (tidak ada padanan)
  - JS: `❌ Subcommand tidak dikenal. Gunakan: add, edit, list, kategori, preset, detail, del, status`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada field yang bisa diedit.`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada jadwal aktif untuk kategori `␀``
    Rust: (tidak ada padanan)
  - JS: `🆔 ID: `␀``
    Rust: (tidak ada padanan)
  - JS: `🆔 ␀`
    Rust: (tidak ada padanan)
  - JS: `🏷️ *KATEGORI JADWAL AKTIF*`
    Rust: (tidak ada padanan)
  - JS: `🏷️ *KATEGORI: ␀ (␀)*`
    Rust: (tidak ada padanan)
  - JS: `🏷️ Kategori aktif: ␀`
    Rust: (tidak ada padanan)
  - JS: `🏷️ Kategori: ␀`
    Rust: (tidak ada padanan)
  - JS: `🏷️ ␀`
    Rust: (tidak ada padanan)
  - JS: `💬 ␀`
    Rust: (tidak ada padanan)
  - JS: `📅 *SCHEDULE PLANNER*`
    Rust: (tidak ada padanan)
  - JS: `📅 Belum ada jadwal aktif. Gunakan `.schedule` untuk lihat format planner.`
    Rust: (tidak ada padanan)
  - JS: `📅 Last reset: ␀`
    Rust: (tidak ada padanan)
  - JS: `📊 *SCHEDULE PLANNER STATUS*`
    Rust: (tidak ada padanan)
  - JS: `📌 *DETAIL JADWAL*`
    Rust: (tidak ada padanan)
  - JS: `📍 Default target: ␀`
    Rust: (tidak ada padanan)
  - JS: `📍 Target: ␀`
    Rust: (tidak ada padanan)
  - JS: `📍 ␀`
    Rust: (tidak ada padanan)
  - JS: `📝 Jadwal aktif: ␀`
    Rust: (tidak ada padanan)
  - JS: `📝 Judul: ␀`
    Rust: (tidak ada padanan)
  - JS: `📝 ␀`
    Rust: (tidak ada padanan)
  - JS: `📨 Reminder terkirim: ␀`
    Rust: (tidak ada padanan)
  - JS: `🔄 Daily limit reset: ␀`
    Rust: (tidak ada padanan)
  - JS: `🔄 Mode: ␀`
    Rust: (tidak ada padanan)
  - JS: `🔄 ␀`
    Rust: (tidak ada padanan)
  - JS: `🕕 Next run: ␀ lagi`
    Rust: (tidak ada padanan)
  - JS: `🕕 ␀ lagi`
    Rust: (tidak ada padanan)
  - JS: `🗓️ Dibuat: ␀`
    Rust: (tidak ada padanan)

### searchplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/searchplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/searchplugin.rs`

  - JS: `.js`
    Rust: (tidak ada padanan)
  - JS: `> Error: `␀``
    Rust: (tidak ada padanan)
  - JS: `> File: `␀``
    Rust: (tidak ada padanan)
  - JS: `> Folder: `␀``
    Rust: (tidak ada padanan)
  - JS: `file://␀`
    Rust: (tidak ada padanan)
  - JS: `⚠️`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *ᴘʟᴜɢɪɴ ᴇʀʀᴏʀ*`
    Rust: (tidak ada padanan)

### self (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/self.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/self.rs`

  - JS: `> Hanya owner yang bisa mengubah mode bot!`
    Rust: (tidak ada padanan)
  - JS: `[Mode] Changed to SELF by ␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `[Self Command Error]`
    Rust: (tidak ada padanan)
  - JS: `🚫 *ᴀᴋsᴇs ᴅɪᴛᴏʟᴀᴋ*`
    Rust: (tidak ada padanan)

### sessionstats (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/sessionstats.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/sessionstats.rs`

  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `pre-key`
    Rust: (tidak ada padanan)
  - JS: `┃ *total* — ␀ key · ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ _sweep tiap 3 hari, hapus yang >30 hari_`
    Rust: (tidak ada padanan)
  - JS: `┃ ɴᴏ ᴋᴇʏ ᴛᴇʀsɪᴍᴘᴀɴ␀`
    Rust: `┃ ɴᴏ ᴋᴇʏ ᴛᴇʀsɪᴍᴘᴀɴ`
  - JS: `┃ ▸ `␀` — ␀ key · ␀␀`
    Rust: (tidak ada padanan)
  - JS: `┃ ⤵️ *pre-key* = ␀ key / ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal membaca statistik session: ␀`
    Rust: (tidak ada padanan)
  - JS: `⤵️`
    Rust: (tidak ada padanan)

### setallmenu (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setallmenu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setallmenu.rs`

  - JS: `Variant aktif saat ini: *V␀ — ␀`]?.name || "Unknown"}* 🎯`
    Rust: `Variant aktif saat ini: *V␀ — ␀* 🎯`

### setaudioallmenu (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setaudioallmenu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setaudioallmenu.rs`

  - JS: `Gaya audio All Menu telah sukses diubah menjadi *Varian ␀*. Silakan tes dengan mengetik *␀allmenu*.`
    Rust: `Gaya audio All Menu telah sukses diubah menjadi *Varian ␀*.`

### setgoodbyetype (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setgoodbyetype.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setgoodbyetype.rs`

  - JS: `- *V1 Canvas Image* 🎨 — Bot membuat gambar canvas otomatis berisi foto profil dan nama member yang keluar, lalu dikirim sebagai gambar`
    Rust: (tidak ada padanan)
  - JS: `- *V2 Carousel Cards* 🃏 — Menampilkan kartu carousel interaktif yang bisa di-swipe lengkap dengan tombol action, cocok untuk tampilan modern`
    Rust: (tidak ada padanan)
  - JS: `- *V3 Text Only* 📝 — Pesan teks biasa tanpa gambar sama sekali, ringan dan minimalis`
    Rust: (tidak ada padanan)
  - JS: `- *V4 Group* 👥 — Menggunakan contextInfo bergaya group forward, tampilan rapi dengan label newsletter`
    Rust: (tidak ada padanan)
  - JS: `- *V5 Simple* ✨ — Pesan teks sederhana disertai foto profile member yang keluar, tidak terlalu mencolok namun informatif`
    Rust: (tidak ada padanan)
  - JS: `- *V6 Video* 🎥 — Mengirimkan video perpisahan yang estetik dilengkapi caption otomatis untuk member`
    Rust: (tidak ada padanan)
  - JS: `- *V7 Interactive Quoted* 💬 — Mengirimkan pesan interaktif dan fake quoted dari orang yang keluar`
    Rust: (tidak ada padanan)
  - JS: `- *V8 Super Simple* 👋 — Pesan teks sangat singkat tanpa hiasan (Contoh: Goodbye @user, dari grup...)`
    Rust: (tidak ada padanan)
  - JS: `> Pilih tipe goodbye dari tombol di bawah 👇`
    Rust: (tidak ada padanan)
  - JS: `Daftar Tipe Goodbye`
    Rust: (tidak ada padanan)
  - JS: `␀setgoodbyetype v␀`
    Rust: `.setgoodbyetype`
  - JS: `👋 Pilih Tipe Goodbye`
    Rust: (tidak ada padanan)

### setmenu (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setmenu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setmenu.rs`

  - JS: `> Pilih variant menu dari tombol di bawah 👇`
    Rust: (tidak ada padanan)
  - JS: `Atur tampilan menu utama bot ketika user mengetik perintah menu 📋✨`
    Rust: (tidak ada padanan)
  - JS: `Daftar Variant Menu`
    Rust: (tidak ada padanan)
  - JS: `Gunakan: *v1* s/d *v7*`
    Rust: (tidak ada padanan)
  - JS: `Variant aktif saat ini: *V␀ — ␀`]?.name || "Unknown"}* 🎯`
    Rust: (tidak ada padanan)
  - JS: `_␀_`
    Rust: (tidak ada padanan)
  - JS: `␀ *V␀ — ␀*`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀␀ — ␀`
    Rust: (tidak ada padanan)
  - JS: `␀setmenu ␀`
    Rust: `.setmenu v8`
  - JS: `✅ *MENU VARIANT DIUBAH*`
    Rust: `🎨🖼️ *MENU VARIANT*`
  - JS: `❌ *VARIANT TIDAK VALID*`
    Rust: (tidak ada padanan)
  - JS: `🎨 Pilih Variant Menu`
    Rust: (tidak ada padanan)

### setmenucat (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setmenucat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setmenucat.rs`

  - JS: `Daftar Variant Menucat`
    Rust: (tidak ada padanan)
  - JS: `Tampilan location message tanpa tombol interaktif`
    Rust: (tidak ada padanan)
  - JS: `Tampilan native flow premium dengan video & cuaca`
    Rust: (tidak ada padanan)
  - JS: `Tampilan premium dengan gambar header, limited time offer, dan tombol navigasi interaktif`
    Rust: (tidak ada padanan)
  - JS: `Tampilan teks biasa tanpa media, cocok untuk koneksi lambat atau device yang tidak support interactive message`
    Rust: (tidak ada padanan)
  - JS: `Variant aktif saat ini: *V␀ — ␀`]?.name || "Unknown"}* 🎯`
    Rust: `Variant aktif saat ini: *V␀ — ␀* 🎯`
  - JS: `␀ *V␀ — ␀*`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀␀ — ␀`
    Rust: (tidak ada padanan)
  - JS: `␀setmenucat ␀`
    Rust: `.setmenucat v2`
  - JS: `✅ *MENUCAT VARIANT DIUBAH*`
    Rust: `📂🗂️ *MENUCAT VARIANT*`
  - JS: `📂 Pilih Variant Menucat`
    Rust: (tidak ada padanan)
  - JS: `🖼️`
    Rust: (tidak ada padanan)

### setownertype (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setownertype.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setownertype.rs`

  - JS: `Foto-AI`
    Rust: (tidak ada padanan)
  - JS: `␀setownertype v␀`
    Rust: `.setownertype`

### setreply (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setreply.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setreply.rs`

  - JS: `Daftar Variant Reply`
    Rust: (tidak ada padanan)
  - JS: `Variant aktif saat ini: *V␀ — ␀`]?.name || "Unknown"}* 🎯`
    Rust: `Variant aktif saat ini: *V9x — Unknown* 🎯`
  - JS: `␀setreply ␀`
    Rust: `.setreply v5`
  - JS: `💬 Pilih Variant Reply`
    Rust: (tidak ada padanan)

### setrules (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setrules.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setrules.rs`

  - JS: `✅ *ʙᴏᴛ ʀᴜʟᴇs ᴅɪᴜᴘᴅᴀᴛᴇ*`
    Rust: `✅ *ʙᴏᴛ ʀᴜʟᴇs ᴅɪʜᴜᴘᴅᴀᴛᴇ*`

### setwelcometype (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/setwelcometype.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/setwelcometype.rs`

  - JS: `Daftar Tipe Welcome`
    Rust: (tidak ada padanan)
  - JS: `␀setwelcometype v␀`
    Rust: `.setwelcometype`
  - JS: `👋 Pilih Tipe Welcome`
    Rust: (tidak ada padanan)

### sewabot (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/sewabot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/sewabot.rs`

  - JS: `30i (menit) u2022 12h (jam) u2022 7d (hari) u2022 1m (bulan) u2022 1y (tahun) u2022 lifetime`
    Rust: `30i (menit) • 12h (jam) • 7d (hari) • 1m (bulan) • 1y (tahun) • lifetime`
  - JS: `Akan keluar dari: ␀ grup`
    Rust: (tidak ada padanan)
  - JS: `Berhasil keluar: *␀* grup`
    Rust: (tidak ada padanan)
  - JS: `Bot akan meninggalkan grup ini.`
    Rust: `Bot tidak akan meninggalkan grup manapun.`
  - JS: `Gagal: *␀* grup`
    Rust: (tidak ada padanan)
  - JS: `Grup whitelist: *␀*`
    Rust: (tidak ada padanan)
  - JS: `Hubungi owner untuk sewa bot.`
    Rust: (tidak ada padanan)
  - JS: `Keluar dari: *␀* grup`
    Rust: (tidak ada padanan)
  - JS: `Whitelist: ␀`
    Rust: (tidak ada padanan)
  - JS: `⛔ Grup ini tidak terdaftar dalam sistem sewa.`
    Rust: (tidak ada padanan)
  - JS: `✅ *SEWA BOT AKTIF*`
    Rust: (tidak ada padanan)
  - JS: `✅ Selesai`
    Rust: (tidak ada padanan)
  - JS: `✅ Tidak ada grup yang perlu ditinggalkan`
    Rust: (tidak ada padanan)
  - JS: `👋 Grup ini tidak terdaftar dalam sistem sewa.`
    Rust: (tidak ada padanan)
  - JS: `📊 Total: ␀ grup`
    Rust: (tidak ada padanan)
  - JS: `🕕 Mengambil daftar grup...`
    Rust: (tidak ada padanan)
  - JS: `🕕 Sistem sewa diaktifkan, memproses auto-leave...`
    Rust: (tidak ada padanan)

### similarity (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/similarity.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/similarity.rs`

  - JS: `> Fitur similarity command *DIAKTIFKAN*`
    Rust: `> Fitur similarity command *␀*`
  - JS: `> Fitur similarity command *DIMATIKAN*`
    Rust: `> Fitur similarity command *␀*`

### sistemdaftar (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/sistemdaftar.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/sistemdaftar.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `␀-␀-␀`
    Rust: (tidak ada padanan)

### sprem (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/sprem.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/sprem.rs`

  - JS: `[sprem]`
    Rust: `.sprem`

### srt (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/srt.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/srt.rs`

  - JS: `Album Error:`
    Rust: (tidak ada padanan)
  - JS: `SRT List Error:`
    Rust: (tidak ada padanan)
  - JS: `❌ Terjadi kesalahan saat membuat album gambar.`
    Rust: (tidak ada padanan)

### star (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/star.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/star.rs`

  - JS: `❌ Gagal: ␀`
    Rust: (tidak ada padanan)

### startschedule (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/startschedule.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/startschedule.rs`

  - JS: `[StartSchedule Error]`
    Rust: (tidak ada padanan)

### stop (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/stop.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/stop.rs`

  - JS: `Stopping via command...`
    Rust: (tidak ada padanan)

### stopalljadibot (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/stopalljadibot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/stopalljadibot.rs`

  - JS: `> Semua session disimpan dan bisa diaktifkan ulang.`
    Rust: (tidak ada padanan)
  - JS: `> 💾 Session: *Tersimpan*`
    Rust: (tidak ada padanan)
  - JS: `> 📊 Total: *␀* jadibot`
    Rust: (tidak ada padanan)
  - JS: `Dihentikan: ␀`
    Rust: (tidak ada padanan)
  - JS: `🛑 *sᴇᴍᴜᴀ ᴊᴀᴅɪʙᴏᴛ ᴅɪʜᴇɴᴛɪᴋᴀɴ*`
    Rust: (tidak ada padanan)

### stopdandeletejadibot (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/stopdandeletejadibot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/stopdandeletejadibot.rs`

  - JS: `> Contoh: `␀stopdandeletejadibot @628xxx``
    Rust: `> Contow: `␀stopdandeletejadibot @628xxx``
  - JS: `> 📱 Nomor: *@␀*`
    Rust: (tidak ada padanan)
  - JS: `> 🗑️ Status: *Deleted*`
    Rust: (tidak ada padanan)
  - JS: `Pilih target dengan mention atau reply:`
    Rust: `> Pilih target dengan mention atau reply:`
  - JS: `Session telah dihapus secara permanen.`
    Rust: (tidak ada padanan)
  - JS: `User perlu `.jadibot` ulang untuk membuat session baru.`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀.* @␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Session jadibot untuk *@␀* tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `❌ Tidak ada session jadibot tersimpan`
    Rust: (tidak ada padanan)
  - JS: `🗑️ *ᴊᴀᴅɪʙᴏᴛ ᴅɪʜᴀᴘᴜs*`
    Rust: (tidak ada padanan)

### stopschedule (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/stopschedule.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/stopschedule.rs`

  - JS: `> Scheduler: *Sholat Scheduler*`
    Rust: `Scheduler: *sholat*`
  - JS: `Gunakan `.stopschedule` untuk melihat daftar scheduler`
    Rust: (tidak ada padanan)
  - JS: `[StopSchedule Error]`
    Rust: (tidak ada padanan)
  - JS: `_Gunakan `.startschedule sholat` untuk mengaktifkan kembali_`
    Rust: `_Gunakan `.startschedule ␀` untuk mengaktifkan kembali_`
  - JS: `• `groupschedule` - Group Schedule`
    Rust: (tidak ada padanan)
  - JS: `• `limitreset` - Daily Limit Reset`
    Rust: (tidak ada padanan)
  - JS: `• `messages` - Scheduled Messages`
    Rust: (tidak ada padanan)
  - JS: `• `sewa` - Sewa Checker`
    Rust: (tidak ada padanan)
  - JS: `ℹ️ Sholat Scheduler sudah dalam keadaan nonaktif`
    Rust: (tidak ada padanan)
  - JS: `❌ Scheduler tidak ditemukan atau sudah nonaktif`
    Rust: (tidak ada padanan)

### swgc (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/swgc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/swgc.rs`

  - JS: `> Gagal mengambil daftar grup.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `file-type`
    Rust: (tidak ada padanan)
  - JS: `swgc_␀.␀`
    Rust: `␀/swgc_␀.␀`
  - JS: `✅ Berhasil up sw ke grup ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengambil media.`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴘɪʟɪʜ ɢʀᴜᴘ ᴜɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ*`
    Rust: `📋 *ᴘɪʟɪʜ ɢʀᴜᴘ ᴅɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ*`

### swgcall (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/swgcall.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/swgcall.rs`

  - JS: `*Grup gagal:*`
    Rust: (tidak ada padanan)
  - JS: `--yes`
    Rust: (tidak ada padanan)
  - JS: `> Berhasil: *␀* ✅`
    Rust: (tidak ada padanan)
  - JS: `> Estimasi: *~␀ detik*`
    Rust: (tidak ada padanan)
  - JS: `> Gagal mengambil daftar grup.`
    Rust: (tidak ada padanan)
  - JS: `> Gagal: *␀* ❌`
    Rust: (tidak ada padanan)
  - JS: `> Kirim gambar/video + caption `␀swgcall``
    Rust: (tidak ada padanan)
  - JS: `> Media: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Reply gambar/video/audio + `␀swgcall``
    Rust: (tidak ada padanan)
  - JS: `> Total Grup: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Total: *␀* grup`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `> `␀swgcall teks` - Story teks ke semua grup`
    Rust: (tidak ada padanan)
  - JS: `> • ␀`
    Rust: (tidak ada padanan)
  - JS: `Post Group Status/Story ke SEMUA grup sekaligus (border hijau)`
    Rust: `Post Group Status/Story ke SEMUA grup (border hijau) — BLOKIRAN`
  - JS: `Voice Note`
    Rust: (tidak ada padanan)
  - JS: `_Tekan konfirmasi untuk melanjutkan._`
    Rust: (tidak ada padanan)
  - JS: `file-type`
    Rust: (tidak ada padanan)
  - JS: `swgcall_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `␀cancelswgcall`
    Rust: (tidak ada padanan)
  - JS: `␀swgcall --yes`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *Bot tidak berada di grup manapun.*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *Tidak ada data pending. Kirim ulang media + .swgcall*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ _Fitur ini akan mengirim story ke SEMUA grup!_`
    Rust: (tidak ada padanan)
  - JS: `⚠️ _Story akan dipost ke SEMUA grup!_`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙʀᴏᴀᴅᴄᴀsᴛ sᴡɢᴄ sᴇʟᴇsᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `✅ Kirim ke ␀ Grup`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴇʀʀᴏʀ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Batal`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengambil media.`
    Rust: (tidak ada padanan)
  - JS: `📢 *ᴋᴏɴꜰɪʀᴍᴀsɪ ʙʀᴏᴀᴅᴄᴀsᴛ sᴡɢᴄ*`
    Rust: (tidak ada padanan)

### swgcv2 (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/swgcv2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/swgcv2.rs`

  - JS: `> Gagal mengambil daftar grup.`
    Rust: (tidak ada padanan)
  - JS: `file-type`
    Rust: (tidak ada padanan)
  - JS: `swgcv2_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengambil media.`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴘɪʟɪʜ ɢʀᴜᴘ ᴜɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ ᴠ2*`
    Rust: `📋 *ᴘɪʟɪʄ ɢʀᴜᴘ ᴜɴᴛᴜᴋ ᴘᴏsᴛ sᴛᴏʀʏ ᴠ2*`

### swgcv2all (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/swgcv2all.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/swgcv2all.rs`

  - JS: `[SwgcV2All] Error:`
    Rust: (tidak ada padanan)
  - JS: `file-type`
    Rust: (tidak ada padanan)
  - JS: `✅ *sᴡɢᴄᴠ2 ᴀʟʟ sᴇʟᴇsᴀɪ*`
    Rust: `✅ *sᴡɢᴄᴠ2 ᴀʟʠ sᴇʟᴇsᴀɪ*`
  - JS: `👋 *sᴡɢᴄᴠ2 ᴀʟʟ ɢʟᴏʙᴀʟ*`
    Rust: `👋 *sᴡɢᴄᴠ2 ᴀʟʠ ɢʟᴏʙᴀʟ*`

### templateplugin (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/templateplugin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/templateplugin.rs`

  - JS: `alias: ['ex'],`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: `async function handler(m, ␀) {`
  - JS: `await m.reply('This is an example plugin!')`
    Rust: (tidak ada padanan)
  - JS: `await m.reply('❌ *GAGAL*\n\n> ' + error.message)`
    Rust: (tidak ada padanan)
  - JS: `category: 'general',`
    Rust: (tidak ada padanan)
  - JS: `console.error('Example Plugin Error:', error)`
    Rust: (tidak ada padanan)
  - JS: `const pluginConfig = {`
    Rust: `const pluginConfig = ␀`
  - JS: `cooldown: 3,`
    Rust: (tidak ada padanan)
  - JS: `description: 'Example plugin',`
    Rust: (tidak ada padanan)
  - JS: `energi: 1,`
    Rust: (tidak ada padanan)
  - JS: `example: '.example',`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler }`
    Rust: (tidak ada padanan)
  - JS: `isEnabled: true`
    Rust: (tidak ada padanan)
  - JS: `isGroup: false,`
    Rust: (tidak ada padanan)
  - JS: `isOwner: false,`
    Rust: (tidak ada padanan)
  - JS: `isPremium: false,`
    Rust: (tidak ada padanan)
  - JS: `isPrivate: false,`
    Rust: (tidak ada padanan)
  - JS: `name: 'example',`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `usage: '.example',`
    Rust: (tidak ada padanan)
  - JS: `} catch (error) {`
    Rust: `try ␀ catch (error) ␀`

### unban (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/unban.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/unban.rs`

  - JS: `> Nomor `␀` tidak dalam daftar banned`
    Rust: `␀␀` tidak dalam daftar banned`
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴜsᴇʀ ᴅɪᴜɴʙᴀɴ*`
    Rust: `✅ *ᴜsᴇʀ ᴅɪᴜɴʃᴀɴ*`
  - JS: `✅ *ᴜɴʙᴀɴ ᴜsᴇʀ*`
    Rust: `✅ *ᴜɴʃᴀɴ ᴅsᴇʀ*`

### unblock (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/unblock.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/unblock.rs`

  - JS: `> Target: @␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *ɴᴏᴍᴏʀ ᴅɪ-ᴜɴʙʟᴏᴄᴋ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Nomor tidak valid.`
    Rust: (tidak ada padanan)

### upch (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/upch.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/upch.rs`

  - JS: `Konversi opus gagal`
    Rust: (tidak ada padanan)
  - JS: `Media terlalu kecil atau gagal download`
    Rust: (tidak ada padanan)
  - JS: `[UpCh]`
    Rust: (tidak ada padanan)
  - JS: `audio/ogg; codecs=opus`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg -y -i "␀" -vn -map_metadata -1 -ac 1 -ar 48000 -c:a libopus -b:a 96k -vbr on -application audio -f ogg "␀"`
    Rust: (tidak ada padanan)
  - JS: `upch_in_␀`
    Rust: (tidak ada padanan)
  - JS: `upch_out_␀.ogg`
    Rust: (tidak ada padanan)

### welcomeall (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/owner/welcomeall.js`
- Rust: `/workspaces/oktz-rs/plugin/src/owner/welcomeall.rs`

  - JS: `> Semua member baru akan disambut otomatis!`
    Rust: (tidak ada padanan)
  - JS: `> Welcome dinonaktifkan di semua grup.`
    Rust: (tidak ada padanan)
  - JS: `[WelcomeAll] Error:`
    Rust: (tidak ada padanan)
  - JS: `┃ ✅ Welcome: *AKTIF*`
    Rust: (tidak ada padanan)
  - JS: `┃ ❌ Welcome: *NONAKTIF*`
    Rust: (tidak ada padanan)
  - JS: `┃ 🌐 Total Grup: *␀*`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📊 *ʀᴇsᴜʟᴛ* 」`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴡᴇʟᴄᴏᴍᴇ ɢʟᴏʙᴀʟ ᴏɴ*`
    Rust: `👋 *ᴡᴇʟᴄᴏᴍᴇ ɢʟᴏʙᴀʟ*`
  - JS: `❌ *ᴡᴇʟᴄᴏᴍᴇ ɢʟᴏʙᴀʟ ᴏꜰꜰ*`
    Rust: `👋 *ᴡᴇʟᴄᴏᴍᴇ ɢʟᴏʙᴀʟ*`

### cekjeda (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/cekjeda.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/cekjeda.rs`

  - JS: `HH:mm:ss`
    Rust: (tidak ada padanan)

### cpanel (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/cpanel.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/cpanel.rs`

  - JS: `␀:␀`
    Rust: (tidak ada padanan)
  - JS: `┃ `␀1gb␀` - `␀10gb␀` | `␀unli␀``
    Rust: `┃ `␀1gb␀` - `␀10gb␀` | `␀␀␀``
  - JS: `┃ `␀addceo␀` | `␀delceo␀` | `␀listceo␀``
    Rust: (tidak ada padanan)
  - JS: `┃ `␀addowner␀` | `␀delowner␀` | `␀listowner␀``
    Rust: (tidak ada padanan)
  - JS: `┃ `␀addreseller␀` | `␀delreseller␀` | `␀listreseller␀``
    Rust: (tidak ada padanan)
  - JS: `┃ `␀cadmin␀` | `␀deladmin␀` | `␀listadmin␀``
    Rust: (tidak ada padanan)
  - JS: `┃ `␀listserver␀` | `␀delserver␀` | `␀serverinfo␀``
    Rust: (tidak ada padanan)
  - JS: `╭─「 🎯 *ᴄᴇᴏ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ* 」`
    Rust: `🎯 *ᴄᴇᴏ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ*`
  - JS: `╭─「 👑 *ᴏᴡɴᴇʀ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ* 」`
    Rust: `👑 *ᴏᴡɴᴇʀ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ*`
  - JS: `╭─「 👥 *ʀᴇsᴇʟʟᴇʀ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ* 」`
    Rust: `👥 *ʀᴇsᴇʟʟᴇʀ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ*`
  - JS: `╭─「 📦 *ᴄʀᴇᴀᴛᴇ sᴇʀᴠᴇʀ* 」`
    Rust: `📦 *ᴄʀᴇᴀᴛᴇ sᴇʀᴠᴇʀ*`
  - JS: `╭─「 🔐 *ᴀᴅᴍɪɴ ᴘᴀɴᴇʟ* 」`
    Rust: `🔐 *ᴀᴅᴍɪɴ ᴘᴀɴᴇʟ*`
  - JS: `╭─「 🖥️ *sᴇʀᴠᴇʀ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ* 」`
    Rust: `🖥️ *sᴇʀᴠᴇʀ ᴍᴀɴᴀɢᴇᴍᴇɴᴛ*`

### installtemabilling (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/installtemabilling.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/installtemabilling.rs`

  - JS: `> Compiling panel assets...`
    Rust: (tidak ada padanan)
  - JS: `> IP atau Password tidak valid.`
    Rust: (tidak ada padanan)
  - JS: `> Mendownload & install tema Billing...`
    Rust: (tidak ada padanan)
  - JS: `> Menginstall Node.js, Yarn, Composer...`
    Rust: (tidak ada padanan)
  - JS: `> _Tema Billing + dependencies berhasil diinstall!_`
    Rust: (tidak ada padanan)
  - JS: `AKSES TOKEN`
    Rust: (tidak ada padanan)
  - JS: `Masukkan pilihan`
    Rust: (tidak ada padanan)
  - JS: `apt-get update -y && apt-get install -y curl git && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && apt-get install -y nodejs && npm i -g yarn && apt-get install -y composer`
    Rust: (tidak ada padanan)
  - JS: `bash <(curl -s https://raw.githubusercontent.com/veryLinh/Theme-Autoinstaller/main/install.sh)`
    Rust: (tidak ada padanan)
  - JS: `cd /var/www/pterodactyl && composer install --no-dev --optimize-autoloader && yarn install && export NODE_OPTIONS=--openssl-legacy-provider && yarn build:production && php artisan view:clear && php artisan config:clear`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ sᴛᴀᴛᴜs: *Terinstall*`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ ɪᴘ: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ✅ *ᴛᴇᴍᴀ ʙɪʟʟɪɴɢ* 」`
    Rust: `╭┈┈⬡「 🎨 *ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ ʙɪʟʟɪɴɢ* 」`
  - JS: `❌ Koneksi gagal!`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[1/3] ɪɴsᴛᴀʟʟ ᴅᴇᴘᴇɴᴅᴇɴᴄɪᴇs...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[2/3] ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[3/3] ʙᴜɪʟᴅ ᴀssᴇᴛs...*`
    Rust: (tidak ada padanan)

### installtemaenigma (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/installtemaenigma.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/installtemaenigma.rs`

  - JS: `> Compiling panel assets...`
    Rust: (tidak ada padanan)
  - JS: `> IP atau Password tidak valid.`
    Rust: (tidak ada padanan)
  - JS: `> Mendownload & install tema Enigma...`
    Rust: (tidak ada padanan)
  - JS: `> Menginstall Node.js, Yarn, Composer...`
    Rust: (tidak ada padanan)
  - JS: `> _Tema Enigma + dependencies berhasil diinstall!_`
    Rust: (tidak ada padanan)
  - JS: `AKSES TOKEN`
    Rust: (tidak ada padanan)
  - JS: `Masukkan pilihan`
    Rust: (tidak ada padanan)
  - JS: `apt-get update -y && apt-get install -y curl git && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && apt-get install -y nodejs && npm i -g yarn && apt-get install -y composer`
    Rust: (tidak ada padanan)
  - JS: `bash <(curl -s https://raw.githubusercontent.com/veryLinh/Theme-Autoinstaller/main/install.sh)`
    Rust: (tidak ada padanan)
  - JS: `cd /var/www/pterodactyl && composer install --no-dev --optimize-autoloader && yarn install && export NODE_OPTIONS=--openssl-legacy-provider && yarn build:production && php artisan view:clear && php artisan config:clear`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ sᴛᴀᴛᴜs: *Terinstall*`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ ɪᴘ: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ✅ *ᴛᴇᴍᴀ ᴇɴɪɢᴍᴀ* 」`
    Rust: `╭┈┈⬡「 🎨 *ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ ᴇɴɪɢᴍᴀ* 」`
  - JS: `❌ Koneksi gagal!`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[1/3] ɪɴsᴛᴀʟʟ ᴅᴇᴘᴇɴᴅᴇɴᴄɪᴇs...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[2/3] ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[3/3] ʙᴜɪʟᴅ ᴀssᴇᴛs...*`
    Rust: (tidak ada padanan)

### installtemanebula (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/installtemanebula.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/installtemanebula.rs`

  - JS: `> IP atau Password tidak valid / VPS down.`
    Rust: (tidak ada padanan)
  - JS: `> Mendownload & konfigurasi Blueprint Framework...`
    Rust: (tidak ada padanan)
  - JS: `> Menginstall Node.js 22, Yarn, dan dependencies...`
    Rust: (tidak ada padanan)
  - JS: `> Menginstall tema Nebula (Auto-Confirm)...`
    Rust: (tidak ada padanan)
  - JS: `> _Tema Nebula berhasil diinstall!_`
    Rust: (tidak ada padanan)
  - JS: `Command failed with code ␀`
    Rust: (tidak ada padanan)
  - JS: `OWNERSHIP="www-data:www-data";`
    Rust: (tidak ada padanan)
  - JS: `Output: ␀`
    Rust: (tidak ada padanan)
  - JS: `USERSHELL="/bin/bash";' > .blueprintrc &&`
    Rust: (tidak ada padanan)
  - JS: `[Nebula Install Error]`
    Rust: (tidak ada padanan)
  - JS: `[SSH Error]`
    Rust: (tidak ada padanan)
  - JS: `apt-get install -y curl wget unzip git zip gnupg ca-certificates -qq &&`
    Rust: (tidak ada padanan)
  - JS: `apt-get install -y nodejs -qq &&`
    Rust: (tidak ada padanan)
  - JS: `apt-get update -qq &&`
    Rust: (tidak ada padanan)
  - JS: `bash blueprint.sh`
    Rust: (tidak ada padanan)
  - JS: `cd /var/www/pterodactyl &&`
    Rust: (tidak ada padanan)
  - JS: `cd /var/www/pterodactyl && yarn install`
    Rust: (tidak ada padanan)
  - JS: `chmod +x blueprint.sh &&`
    Rust: (tidak ada padanan)
  - JS: `curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -yes -o /etc/apt/keyrings/nodesource.gpg &&`
    Rust: (tidak ada padanan)
  - JS: `echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_22.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list &&`
    Rust: (tidak ada padanan)
  - JS: `echo 'WEBUSER="www-data";`
    Rust: (tidak ada padanan)
  - JS: `else`
    Rust: (tidak ada padanan)
  - JS: `fi`
    Rust: (tidak ada padanan)
  - JS: `if command -v blueprint &> /dev/null; then`
    Rust: (tidak ada padanan)
  - JS: `mkdir -p /etc/apt/keyrings &&`
    Rust: (tidak ada padanan)
  - JS: `npm i -g yarn &&`
    Rust: (tidak ada padanan)
  - JS: `printf "\n\n" | bash blueprint.sh -install nebula;`
    Rust: (tidak ada padanan)
  - JS: `printf "\n\n" | blueprint -install nebula;`
    Rust: (tidak ada padanan)
  - JS: `rm release.zip &&`
    Rust: (tidak ada padanan)
  - JS: `touch .blueprintrc &&`
    Rust: (tidak ada padanan)
  - JS: `unzip -o release.zip &&`
    Rust: (tidak ada padanan)
  - JS: `wget "$(curl -s https://api.github.com/repos/BlueprintFramework/framework/releases/latest | grep 'browser_download_url' | grep 'release.zip' | cut -d '"' -f 4)" -O release.zip &&`
    Rust: (tidak ada padanan)
  - JS: `wget -q -O nebula.blueprint "https://github.com/FikXzModzDeveloper/Nebula-Theme-pterodactyl/raw/main/nebula.blueprint" &&`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ sᴛᴀᴛᴜs: *Terinstall*`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ ɪᴘ: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ✅ *ᴛᴇᴍᴀ ɴᴇʙᴜʟᴀ* 」`
    Rust: `╭┈┈⬡「 🌌 *ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ ɴᴇʙᴜʟᴀ* 」`
  - JS: `❌ Format salah! Gunakan: `ip|password``
    Rust: (tidak ada padanan)
  - JS: `❌ Koneksi gagal!`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[1/3] ᴘʀᴇᴘᴀʀɪɴɢ ᴇɴᴠɪʀᴏɴᴍᴇɴᴛ...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[2/3] ɪɴsᴛᴀʟʟ ʙʟᴜᴇᴘʀɪɴᴛ...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[3/3] ɪɴsᴛᴀʟʟ ɴᴇʙᴜʟᴀ...*`
    Rust: (tidak ada padanan)

### installtemastellar (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/installtemastellar.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/installtemastellar.rs`

  - JS: `> Compiling panel assets...`
    Rust: (tidak ada padanan)
  - JS: `> IP atau Password tidak valid.`
    Rust: (tidak ada padanan)
  - JS: `> Mendownload & install tema Stellar...`
    Rust: (tidak ada padanan)
  - JS: `> Menginstall Node.js, Yarn, Composer...`
    Rust: (tidak ada padanan)
  - JS: `> _Tema Stellar + dependencies berhasil diinstall!_`
    Rust: (tidak ada padanan)
  - JS: `apt-get update -y && apt-get install -y curl git && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && apt-get install -y nodejs && npm i -g yarn && apt-get install -y composer`
    Rust: (tidak ada padanan)
  - JS: `bash <(curl -s https://raw.githubusercontent.com/AnonGhostID/flavor/main/flavor.sh)`
    Rust: (tidak ada padanan)
  - JS: `cd /var/www/pterodactyl && composer install --no-dev --optimize-autoloader && yarn install && export NODE_OPTIONS=--openssl-legacy-provider && yarn build:production && php artisan view:clear && php artisan config:clear`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ sᴛᴀᴛᴜs: *Terinstall*`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ ɪᴘ: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ✅ *ᴛᴇᴍᴀ sᴛᴇʟʟᴀʀ* 」`
    Rust: `╭┈┈⬡「 🎨 *ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ sᴛᴇʟʟᴀʀ* 」`
  - JS: `❌ Format salah! Gunakan: `ip|password``
    Rust: (tidak ada padanan)
  - JS: `❌ Koneksi gagal!`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[1/3] ɪɴsᴛᴀʟʟ ᴅᴇᴘᴇɴᴅᴇɴᴄɪᴇs...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[2/3] ɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ...*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *[3/3] ʙᴜɪʟᴅ ᴀssᴇᴛs...*`
    Rust: (tidak ada padanan)

### uinstalltema (panel)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/panel/uninstalltema.js`
- Rust: `/workspaces/oktz-rs/plugin/src/panel/uninstalltema.rs`

  - JS: `> IP atau Password tidak valid.`
    Rust: (tidak ada padanan)
  - JS: `> Tunggu 1-10 menit hingga proses selesai`
    Rust: (tidak ada padanan)
  - JS: `> _Tema berhasil diuninstall!_`
    Rust: (tidak ada padanan)
  - JS: `[SSH Error]`
    Rust: (tidak ada padanan)
  - JS: `[UninstallTema STDERR]`
    Rust: (tidak ada padanan)
  - JS: `[UninstallTema]`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ sᴛᴀᴛᴜs: *Berhasil*`
    Rust: (tidak ada padanan)
  - JS: `┃ ㊗ ɪᴘ: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 ✅ *ᴜɴɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ* 」`
    Rust: `╭┈┈⬡「 🗑️ *ᴜɴɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ* 」`
  - JS: `❌ Format salah! Gunakan: `ip|password``
    Rust: (tidak ada padanan)
  - JS: `❌ Koneksi gagal!`
    Rust: (tidak ada padanan)
  - JS: `🕕 *ᴍᴇᴍᴘʀᴏsᴇs ᴜɴɪɴsᴛᴀʟʟ ᴛᴇᴍᴀ...*`
    Rust: (tidak ada padanan)

### zodiak (primbon)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/primbon/zodiak.js`
- Rust: `/workspaces/oktz-rs/plugin/src/primbon/zodiak.rs`

  - JS: `')}`
    Rust: (tidak ada padanan)
  - JS: `␀`).join('`
    Rust: (tidak ada padanan)

### pushkontak (pushkontak)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/pushkontak/pushkontak.js`
- Rust: `/workspaces/oktz-rs/plugin/src/pushkontak/pushkontak.rs`

  - JS: `#␀`
    Rust: (tidak ada padanan)
  - JS: `*CARA AKTIVASI:*`
    Rust: (tidak ada padanan)
  - JS: `*INFO:*`
    Rust: (tidak ada padanan)
  - JS: `*PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `*PENJELASAN ALUR PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `*PENJELASAN:*`
    Rust: (tidak ada padanan)
  - JS: `*Reply pesan ini dengan:*`
    Rust: (tidak ada padanan)
  - JS: `1. Jeda adalah waktu tunggu antar pengiriman pesan ke setiap member`
    Rust: (tidak ada padanan)
  - JS: `1. Pastikan grup dalam mode pushkontak: *␀botmode pushkontak*`
    Rust: (tidak ada padanan)
  - JS: `1. Tekan tombol di bawah untuk mengaktifkan mode pushkontak`
    Rust: (tidak ada padanan)
  - JS: `2. Ketik *␀pushkontak* lalu pilih "Mulai Push" dari menu`
    Rust: (tidak ada padanan)
  - JS: `2. Semakin kecil jeda, semakin cepat push selesai, tapi risiko ban lebih tinggi`
    Rust: (tidak ada padanan)
  - JS: `2. Setelah mode berubah, ulangi perintah push kontak`
    Rust: (tidak ada padanan)
  - JS: `3. Bot akan meminta kamu menginput pesan yang ingin dikirim via reply`
    Rust: (tidak ada padanan)
  - JS: `3. Rekomendasi minimal *3000ms* (3 detik) agar aman`
    Rust: (tidak ada padanan)
  - JS: `4. Nilai maksimal *30000ms* (30 detik)`
    Rust: (tidak ada padanan)
  - JS: `4. Setelah konfirmasi, bot mengirim pesan ke setiap member satu per satu`
    Rust: (tidak ada padanan)
  - JS: `5. Setiap pesan ditambahkan kode unik agar terdeteksi berbeda oleh WhatsApp`
    Rust: (tidak ada padanan)
  - JS: `6. Setelah selesai, bot otomatis mengirimkan file VCF berisi semua kontak member`
    Rust: (tidak ada padanan)
  - JS: `Aman dari ban`
    Rust: (tidak ada padanan)
  - JS: `Auto VCF`
    Rust: (tidak ada padanan)
  - JS: `BEGIN:VCARD`
    Rust: (tidak ada padanan)
  - JS: `Cepat, risiko ban tinggi`
    Rust: (tidak ada padanan)
  - JS: `END:VCARD`
    Rust: (tidak ada padanan)
  - JS: `FN:WA[␀] ␀`
    Rust: (tidak ada padanan)
  - JS: `Hentikan push yang sedang berjalan`
    Rust: (tidak ada padanan)
  - JS: `Input pesan lalu push ke semua member`
    Rust: (tidak ada padanan)
  - JS: `Jeda paling lama`
    Rust: (tidak ada padanan)
  - JS: `Jeda saat ini: ␀ms`
    Rust: (tidak ada padanan)
  - JS: `Kode Unik`
    Rust: (tidak ada padanan)
  - JS: `Kontak_␀_␀.vcf`
    Rust: (tidak ada padanan)
  - JS: `Lewati admin grup saat push`
    Rust: (tidak ada padanan)
  - JS: `Normal, rekomendasi`
    Rust: (tidak ada padanan)
  - JS: `Paling aman dari risiko ban`
    Rust: (tidak ada padanan)
  - JS: `Push Kontak`
    Rust: `.pushkontak`
  - JS: `Push pesan ke semua member grup + auto simpan kontak VCF`
    Rust: `Push pesan ke semua member grup + auto simpan kontak VCF — BLOKIRAN`
  - JS: `Rekomendasi untuk pemakaian umum`
    Rust: (tidak ada padanan)
  - JS: `Risiko ban lebih tinggi`
    Rust: (tidak ada padanan)
  - JS: `Sangat aman`
    Rust: (tidak ada padanan)
  - JS: `Simpan kontak ke VCF otomatis setelah push`
    Rust: (tidak ada padanan)
  - JS: `Skip Admin`
    Rust: (tidak ada padanan)
  - JS: `TEL;type=CELL;type=VOICE;waid=␀:+␀`
    Rust: (tidak ada padanan)
  - JS: `Tambah kode random di akhir pesan`
    Rust: (tidak ada padanan)
  - JS: `Untuk grup besar 500+ member`
    Rust: (tidak ada padanan)
  - JS: `VCF Target`
    Rust: (tidak ada padanan)
  - JS: `VCF dikirim ke chat pribadi owner`
    Rust: (tidak ada padanan)
  - JS: `VCF dikirim ke group chat`
    Rust: (tidak ada padanan)
  - JS: `VCF:␀ | Kode:␀ | Jeda:␀s`
    Rust: (tidak ada padanan)
  - JS: `VERSION:3.0`
    Rust: (tidak ada padanan)
  - JS: `pushkontak_␀.vcf`
    Rust: `.pushkontak`
  - JS: `shop.example.com`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *Jeda baru:* ␀ms (␀ detik)`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *Jeda saat ini:* ␀ms (␀ detik)`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *Jeda:* ␀ms`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *SET JEDA PUSH KONTAK*`
    Rust: (tidak ada padanan)
  - JS: `⏱️ *␀setjedapush <ms>* — Atur jeda antar pengiriman`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Jeda Push`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Jeda: *␀ms (␀s)*`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Pilih Jeda`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Rekomendasi Jeda Push Kontak`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Set Jeda Push`
    Rust: (tidak ada padanan)
  - JS: `⏸️ *Sisa:* ␀`
    Rust: (tidak ada padanan)
  - JS: `⏹️`
    Rust: (tidak ada padanan)
  - JS: `⏹️ *Ketik* ␀stoppush *untuk menghentikan*`
    Rust: (tidak ada padanan)
  - JS: `⏹️ *PUSH DIHENTIKAN*`
    Rust: (tidak ada padanan)
  - JS: `⏹️ *␀stoppush* — Hentikan push yang sedang berjalan`
    Rust: (tidak ada padanan)
  - JS: `⏹️ Stop Push`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀ ␀*`
    Rust: (tidak ada padanan)
  - JS: `␀ Auto VCF: ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ Kirim ke Group Chat`
    Rust: (tidak ada padanan)
  - JS: `␀ Kirim ke Private Chat`
    Rust: (tidak ada padanan)
  - JS: `␀ Kode Unik: ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ Skip Admin: ␀`
    Rust: (tidak ada padanan)
  - JS: `␀botmode pushkontak`
    Rust: (tidak ada padanan)
  - JS: `␀kelolapush`
    Rust: (tidak ada padanan)
  - JS: `␀pushkontak_start`
    Rust: (tidak ada padanan)
  - JS: `␀s`
    Rust: (tidak ada padanan)
  - JS: `␀setjedapush`
    Rust: (tidak ada padanan)
  - JS: `␀setjedapush 10000`
    Rust: (tidak ada padanan)
  - JS: `␀setjedapush 15000`
    Rust: (tidak ada padanan)
  - JS: `␀setjedapush 3000`
    Rust: (tidak ada padanan)
  - JS: `␀setjedapush 30000`
    Rust: (tidak ada padanan)
  - JS: `␀setjedapush 5000`
    Rust: (tidak ada padanan)
  - JS: `␀stoppush`
    Rust: (tidak ada padanan)
  - JS: `␀vcftarget_group`
    Rust: (tidak ada padanan)
  - JS: `␀vcftarget_private`
    Rust: (tidak ada padanan)
  - JS: `␀␀`
    Rust: (tidak ada padanan)
  - JS: `⚙️ *KELOLA PUSH KONTAK*`
    Rust: (tidak ada padanan)
  - JS: `⚙️ *␀:* ␀`
    Rust: (tidak ada padanan)
  - JS: `⚙️ Kelola`
    Rust: (tidak ada padanan)
  - JS: `⚙️ Kelola Cepat`
    Rust: (tidak ada padanan)
  - JS: `⚙️ Kelola Push Kontak`
    Rust: (tidak ada padanan)
  - JS: `⚡ 3 Detik`
    Rust: (tidak ada padanan)
  - JS: `⚡ 3 Detik (Cepat)`
    Rust: (tidak ada padanan)
  - JS: `✅ *Berhasil:* ␀`
    Rust: (tidak ada padanan)
  - JS: `✅ *JEDA DIUBAH*`
    Rust: (tidak ada padanan)
  - JS: `✅ *LANGKAH 2/2 — KONFIRMASI*`
    Rust: (tidak ada padanan)
  - JS: `✅ *PUSH SELESAI*`
    Rust: (tidak ada padanan)
  - JS: `✅ *Proses push kontak akan segera berhenti*`
    Rust: (tidak ada padanan)
  - JS: `✅ *ya* — Mulai push sekarang`
    Rust: (tidak ada padanan)
  - JS: `❌ *Atau reply* `batal` *untuk membatalkan*`
    Rust: (tidak ada padanan)
  - JS: `❌ *GAGAL*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal:* ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *MODE TIDAK SESUAI*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Pesan tidak boleh kosong*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Reply tidak valid*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Reply* `batal` *untuk membatalkan sesi*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Reply* `batal` *untuk membatalkan*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Sesi push kontak dibatalkan*`
    Rust: (tidak ada padanan)
  - JS: `❌ *batal* — Batalkan sesi`
    Rust: (tidak ada padanan)
  - JS: `🏔️ 30 Detik (Maksimal)`
    Rust: (tidak ada padanan)
  - JS: `🐢 15 Detik`
    Rust: (tidak ada padanan)
  - JS: `🐢 15 Detik (Sangat Aman)`
    Rust: (tidak ada padanan)
  - JS: `👑 Skip Admin`
    Rust: (tidak ada padanan)
  - JS: `👑 Skip Admin: *␀*`
    Rust: (tidak ada padanan)
  - JS: `👥 *Grup:* ␀`
    Rust: (tidak ada padanan)
  - JS: `👥 *Target:* ␀ member`
    Rust: (tidak ada padanan)
  - JS: `💾 *AUTO-SAVE KONTAK*`
    Rust: (tidak ada padanan)
  - JS: `💾 *Auto VCF:* ␀ | 🔑 *Kode Unik:* ␀`
    Rust: (tidak ada padanan)
  - JS: `💾 *Kontak:* ␀ disimpan`
    Rust: (tidak ada padanan)
  - JS: `💾 Auto VCF`
    Rust: (tidak ada padanan)
  - JS: `💾 Auto VCF: *␀*`
    Rust: (tidak ada padanan)
  - JS: `📊 *Estimasi:* ␀ menit`
    Rust: (tidak ada padanan)
  - JS: `📊 *Total:* ␀`
    Rust: (tidak ada padanan)
  - JS: `📊 *Total:* ␀ kontak`
    Rust: (tidak ada padanan)
  - JS: `📋 *Kirim pesan ke semua member grup secara otomatis + simpan kontak ke file VCF*`
    Rust: (tidak ada padanan)
  - JS: `📋 *Mengatur interval antar pengiriman pesan*`
    Rust: (tidak ada padanan)
  - JS: `📋 *SETTING SAAT INI*`
    Rust: (tidak ada padanan)
  - JS: `📋 *SETTING*`
    Rust: (tidak ada padanan)
  - JS: `📋 Pilih Fitur`
    Rust: (tidak ada padanan)
  - JS: `📌 *Pilih dari tombol di bawah untuk mengubah setting*`
    Rust: (tidak ada padanan)
  - JS: `📌 *Pilih jeda dari tombol di bawah atau ketik manual*`
    Rust: (tidak ada padanan)
  - JS: `📝 *Langkah 1/2 — Input Pesan*`
    Rust: (tidak ada padanan)
  - JS: `📝 *Pesan:* ␀␀`
    Rust: (tidak ada padanan)
  - JS: `📝 *Sesi push sudah aktif*`
    Rust: (tidak ada padanan)
  - JS: `📝 *UBAH PESAN*`
    Rust: (tidak ada padanan)
  - JS: `📝 *ubah* — Ubah pesan yang ingin dikirim`
    Rust: (tidak ada padanan)
  - JS: `📝 *␀botmode pushkontak*`
    Rust: (tidak ada padanan)
  - JS: `📝 *␀pushkontak <pesan>* — Push langsung dengan pesan`
    Rust: (tidak ada padanan)
  - JS: `📝 *␀setjedapush <milidetik>* — Mengubah jeda push`
    Rust: (tidak ada padanan)
  - JS: `📢 *Ketik* ␀pushkontak *untuk memulai lagi*`
    Rust: (tidak ada padanan)
  - JS: `📢 *PUSH KONTAK DIMULAI*`
    Rust: (tidak ada padanan)
  - JS: `📢 *PUSH KONTAK*`
    Rust: (tidak ada padanan)
  - JS: `📢 *SESI PUSH KONTAK*`
    Rust: (tidak ada padanan)
  - JS: `📢 *␀pushkontak* — Buka menu interaktif`
    Rust: (tidak ada padanan)
  - JS: `📢 Aksi`
    Rust: (tidak ada padanan)
  - JS: `📢 Mulai Push`
    Rust: (tidak ada padanan)
  - JS: `📢 Mulai Push (Sesi Input)`
    Rust: (tidak ada padanan)
  - JS: `📢 Pilih Fitur`
    Rust: (tidak ada padanan)
  - JS: `📢 Push Ulang`
    Rust: (tidak ada padanan)
  - JS: `📩 *Reply dengan:* `ya`, `ubah`, atau `batal``
    Rust: (tidak ada padanan)
  - JS: `📩 *Reply lagi dengan pesan yang valid*`
    Rust: (tidak ada padanan)
  - JS: `📩 *Reply pesan ini dengan pesan baru*`
    Rust: (tidak ada padanan)
  - JS: `📩 *Reply pesan ini dengan pesan yang ingin dikirim*`
    Rust: (tidak ada padanan)
  - JS: `📩 *Reply pesan sebelumnya dengan pesan yang ingin di-push*`
    Rust: (tidak ada padanan)
  - JS: `📱 *File VCF telah dikirim ke chat pribadi*`
    Rust: (tidak ada padanan)
  - JS: `📱 Target VCF`
    Rust: (tidak ada padanan)
  - JS: `📱 VCF Target: *␀*`
    Rust: (tidak ada padanan)
  - JS: `📱 _Import file ini ke HP untuk menyimpan semua kontak_`
    Rust: (tidak ada padanan)
  - JS: `🔄 *Memulai push...*`
    Rust: (tidak ada padanan)
  - JS: `🔄 *Push kontak sedang berjalan*`
    Rust: (tidak ada padanan)
  - JS: `🔄 5 Detik`
    Rust: (tidak ada padanan)
  - JS: `🔄 5 Detik (Normal)`
    Rust: (tidak ada padanan)
  - JS: `🔑 *Akses:* Owner only`
    Rust: (tidak ada padanan)
  - JS: `🔑 Kode Unik`
    Rust: (tidak ada padanan)
  - JS: `🔑 Kode Unik: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🔒 *Aktifkan mode pushkontak terlebih dahulu*`
    Rust: (tidak ada padanan)
  - JS: `🔒 *Grup ini belum dalam mode pushkontak*`
    Rust: (tidak ada padanan)
  - JS: `🔓 Aktifkan Mode Pushkontak`
    Rust: (tidak ada padanan)
  - JS: `🔤 *Kirim pesan baru yang ingin di-push*`
    Rust: (tidak ada padanan)
  - JS: `🔤 *Kirim pesan yang ingin di-push ke semua member*`
    Rust: (tidak ada padanan)
  - JS: `🚫 *Jeda harus antara 1000ms - 30000ms*`
    Rust: (tidak ada padanan)
  - JS: `🚫 *Tidak ada member yang bisa dikirim pesan*`
    Rust: (tidak ada padanan)
  - JS: `🚫 *Tidak ada push kontak yang berjalan saat ini*`
    Rust: (tidak ada padanan)
  - JS: `🛡️ 10 Detik`
    Rust: (tidak ada padanan)
  - JS: `🛡️ 10 Detik (Aman)`
    Rust: (tidak ada padanan)

### asmaulhusna (religi)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/religi/asmaulhusna.js`
- Rust: `/workspaces/oktz-rs/plugin/src/religi/asmaulhusna.rs`

  - JS: `asmaulhusna.json`
    Rust: `.asmaulhusna 1`

### audioquran (religi)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/religi/audioquran.js`
- Rust: `/workspaces/oktz-rs/plugin/src/religi/audioquran.rs`

  - JS: `.audio-quran reciters`
    Rust: `- `.audio-quran reciters` (List qari)`
  - JS: `.audio-quran suwar`
    Rust: `audio-quran`
  - JS: `Field 'reciter' wajib (contoh: 'Sudais', 'Mishary', 'Husary').`
    Rust: (tidak ada padanan)
  - JS: `Field 'surah' harus 1-114.`
    Rust: (tidak ada padanan)
  - JS: `[AudioQuran]`
    Rust: (tidak ada padanan)
  - JS: `]+/).filter(Boolean);`
    Rust: (tidak ada padanan)
  - JS: `break;`
    Rust: (tidak ada padanan)
  - JS: `const d = lev(reciterQuery, tok);`
    Rust: (tidak ada padanan)
  - JS: `const ranked = all.map((r) => {`
    Rust: (tidak ada padanan)
  - JS: `const tokens = (r.name ?? "").toLowerCase().split(/[s-`
    Rust: (tidak ada padanan)
  - JS: `for (const tok of tokens) {`
    Rust: (tidak ada padanan)
  - JS: `fuzzyHits.push({ reciter: r, distance: d });`
    Rust: (tidak ada padanan)
  - JS: `fuzzyHits.sort((a, b) => a.distance - b.distance);`
    Rust: (tidak ada padanan)
  - JS: `if (d <= threshold) {`
    Rust: (tidak ada padanan)
  - JS: `if (matches.length === 0) {`
    Rust: (tidak ada padanan)
  - JS: `matches = fuzzyHits.slice(0, 10).map((h) => h.reciter);`
    Rust: (tidak ada padanan)
  - JS: `mp3quran HTTP ␀`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `␀␀?␀`
    Rust: (tidak ada padanan)
  - JS: `🎙️ ␀ qari tersedia (bahasa: ␀)`
    Rust: (tidak ada padanan)
  - JS: `📖 ␀ surah (1-114)`
    Rust: (tidak ada padanan)
  - JS: `📚 ␀ riwayat bacaan`
    Rust: (tidak ada padanan)
  - JS: `📻 ␀ radio stream live`
    Rust: (tidak ada padanan)

### islami (religi)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/religi/islami.js`
- Rust: `/workspaces/oktz-rs/plugin/src/religi/islami.rs`

  - JS: `Religi Plugin Error:`
    Rust: (tidak ada padanan)
  - JS: `❌ Surah terlalu panjang untuk dikirim via teks. Silakan cari ayat spesifik atau surah yang lebih pendek.`
    Rust: (tidak ada padanan)
  - JS: `❌ Terjadi kesalahan saat mengambil data.`
    Rust: (tidak ada padanan)
  - JS: `🕕 Sedang memuat surah...`
    Rust: (tidak ada padanan)

### arena (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/arena.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/arena.rs`

  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `@␀ menerjang ke arah @␀!`
    Rust: (tidak ada padanan)
  - JS: `Semoga berhasil ya kak! 🔥`
    Rust: (tidak ada padanan)
  - JS: `⚔️`
    Rust: (tidak ada padanan)
  - JS: `⚔️ *PERTARUNGAN DIMULAI!* ⚔️`
    Rust: (tidak ada padanan)
  - JS: `💢 Lawan membalas dengan keras: *-␀ HP*`
    Rust: `> 💢 Lawan membalas dengan keras: *-␀ HP*`
  - JS: `🔥 Kamu melancarkan serangan kuat: *-␀ HP*`
    Rust: `> 🔥 Kamu melancarkan serangan kuat: *-␀ HP*`

### boss (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/boss.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/boss.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `*Penalti Kekalahan:*`
    Rust: `*Penalti Kekurangan:*`
  - JS: `.join("`
    Rust: (tidak ada padanan)
  - JS: `␀`)`
    Rust: (tidak ada padanan)
  - JS: `⚔️`
    Rust: (tidak ada padanan)
  - JS: `⚔️ Kamu menebas boss: *-␀ HP*`
    Rust: `> ⚔️ Kamu menebas boss: *-␀ HP*`
  - JS: `👹 Boss mengamuk dan memukul mundur: *-␀ HP*`
    Rust: `> 👹 Boss mengamuk dan memukul mundur: *-␀ HP*`
  - JS: `💥 *CRITICAL HIT!!* Serangan mematikanmu masuk: *-␀ HP*`
    Rust: `> 💥 *CRITICAL HIT!!* Serangan mematikanmu masuk: *-␀ HP*`

### buykoin (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/buykoin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/buykoin.rs`

  - JS: ``;`
    Rust: (tidak ada padanan)
  - JS: `await m.react("💱");`
    Rust: (tidak ada padanan)
  - JS: `const newExp = (user.exp || 0) - expNeeded;`
    Rust: (tidak ada padanan)
  - JS: `const newKoin = (user.koin || 0) + koinAmount;`
    Rust: (tidak ada padanan)
  - JS: `db.setUser(m.sender, {`
    Rust: (tidak ada padanan)
  - JS: `exp: newExp,`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `koin: newKoin,`
    Rust: (tidak ada padanan)
  - JS: `let txt = `💱 *Tukar Berhasil!*`
    Rust: `💱 *Tukar Berhasil!*`
  - JS: `m.reply(txt);`
    Rust: (tidak ada padanan)
  - JS: `s stick to update logic here`
    Rust: (tidak ada padanan)
  - JS: `txt += `*📊 Saldo Sekarang:*`
    Rust: `*📊 Saldo Sekarang:*`
  - JS: `txt += `*📋 Detail:*`
    Rust: `*📋 Detail:*`
  - JS: `txt += `> 💰 Koin: *+␀*`
    Rust: `> 💰 Koin: *+␀*`
  - JS: `txt += `> 💰 Koin: *␀*`;`
    Rust: `> 💰 Koin: *␀*`
  - JS: `txt += `> 🚄 EXP: *-␀*`
    Rust: `> 🚄 EXP: *-␀*`
  - JS: `txt += `> 🚄 EXP: *␀*`
    Rust: `> 🚄 EXP: *␀*`
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)

### casino (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/casino.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/casino.rs`

  - JS: `*Cara Taruhan:*`
    Rust: (tidak ada padanan)
  - JS: `Uang lu cuma *Rp ␀* tapi sok-sokan taruhan *Rp ␀*.`
    Rust: `Uang lu cuma *Rp ␀* tapi sok-sokan *Rp ␀*.`
  - JS: `🎲 Bandar mengocok dadu dan memutar roda Roulette... Tahan napas lu!`
    Rust: (tidak ada padanan)

### coinflip (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/coinflip.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/coinflip.rs`

  - JS: `*CLING!* Koin emas dilempar tinggi ke udara... berputar-putar... 🪙✨`
    Rust: (tidak ada padanan)
  - JS: `Flipping!`
    Rust: (tidak ada padanan)
  - JS: `Result!`
    Rust: (tidak ada padanan)
  - JS: `🪙 COINFLIP`
    Rust: (tidak ada padanan)

### crime (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/crime.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/crime.rs`

  - JS: `Mesin ATM ngeluarin duit kayak air terjun! Lu langsung kabur bawa koper penuh duit.`
    Rust: `Mesin ATM ngeluarin duit kayak air turun! Lu langsung kabur bawa koper penuh duit.`

### dice (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/dice.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/dice.rs`

  - JS: `Result!`
    Rust: (tidak ada padanan)
  - JS: `Rolling!`
    Rust: (tidak ada padanan)
  - JS: `🎲 Bandar mengocok dadu di dalam mangkok kayu... *krok krok krok*...`
    Rust: (tidak ada padanan)
  - JS: `🎲 DADU JALANAN`
    Rust: (tidak ada padanan)
  - JS: `🎲 HASIL DADU`
    Rust: (tidak ada padanan)

### divorce (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/divorce.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/divorce.rs`

  - JS: `-- PUTUS DENGAN --`
    Rust: `x20        -- PUTUS DENGAN --`

### duel (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/duel.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/duel.rs`

  - JS: `@␀ dengan berani menantang @␀!`
    Rust: (tidak ada padanan)
  - JS: `Bertarung!`
    Rust: (tidak ada padanan)
  - JS: `Hasil Duel!`
    Rust: (tidak ada padanan)
  - JS: `⚔️ *DUEL DIMULAI!* ⚔️`
    Rust: (tidak ada padanan)
  - JS: `⚔️ ARENA DUEL`
    Rust: (tidak ada padanan)
  - JS: `💰 Total Taruhan di Tengah: *Rp ␀*`
    Rust: (tidak ada padanan)

### enchant (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/enchant.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/enchant.rs`

  - JS: `> Level ␀ → ␀`
    Rust: `> 📊 Level: *␀ → ␀*`
  - JS: `> ✨ EXP: *+150*`
    Rust: `✨ EXP: *+150*`
  - JS: `> 📊 Level: ␀/10`
    Rust: `> 📊 Level: ␀/␀`
  - JS: `✨ *ᴇɴᴄʜᴀɴᴛ - ᴜᴘɢʀᴀᴅᴇ ᴇǫᴜɪᴘ*`
    Rust: `✨ *ᴇɴᴄʜᴀɴᴛ - ᴅᴘɢʀᴀᴅᴇ ᴇǫᴜɪᴘ*`
  - JS: `✨ *ᴍᴇɴɢ-ᴇɴᴄʜᴀɴᴛ ␀...*`
    Rust: (tidak ada padanan)
  - JS: `❌ ␀ sudah level MAX (10)!`
    Rust: `❌ ␀ sudah level MAX (␀)!`
  - JS: `🎉 *ᴇɴᴄʜᴀɴᴛ ʙᴇʀʜᴀsɪʟ!*`
    Rust: `🎉 *ᴇɴᴄʜᴀɴᴛ ʙᴇᴄʜᴀsɪʟ!*`

### expedition (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/expedition.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/expedition.rs`

  - JS: `└ Status: ␀`}`
    Rust: `└ Status: ␀`
  - JS: `└ 🚀 Kode Area: `␀``
    Rust: `x20  └ 🚀 Kode Area: `␀``
  - JS: `├ ⏳ Waktu: ␀`
    Rust: `⏳ Waktu: ␀`
  - JS: `├ 🎁 Potensi Loot: ␀`
    Rust: `🎁 Potensi Loot: ␀`
  - JS: `├ 📈 EXP: ␀ (Min Lv. ␀)`
    Rust: `📈 EXP: ␀ (Min Lv. ␀)`

### garden (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/garden.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/garden.rs`

  - JS: `└ Status: ␀`}`
    Rust: `└ Status: ␀`

### heal (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/heal.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/heal.rs`

  - JS: `Tidur dulu ah... Zzz... 🛌💤`
    Rust: `Tidur dulu ah...`

### hunt (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/hunt.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/hunt.rs`

  - JS: `Wah jago banget bidikannya kak! Kamu berhasil menembak:`
    Rust: `Wah jago banget bidangannya kak! Kamu berhasil menembak:`
  - JS: `🏹 _Mengendap-endap di balik semak belukar... Menyiapkan anak panah..._ 🌿🤫`
    Rust: (tidak ada padanan)

### kurir (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/kurir.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/kurir.rs`

  - JS: `📈 EXP Kompensasi Lari: *+␀*`
    Rust: `📈 EXP Kompensasi Lari: *+500*`

### leveluprpg (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/leveluprpg.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/leveluprpg.rs`

  - JS: `> > `.leveluprpg off` - Nonaktifkan`
    Rust: `> > `␀leveluprpg off` - Nonaktifkan`
  - JS: `> > `.leveluprpg on` - Aktifkan`
    Rust: `> > `␀leveluprpg on` - Aktifkan`

### lottery (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/lottery.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/lottery.rs`

  - JS: `✨ Lampu Disko menyala... Tabung Gacha berputar hebat... Menarik *␀x* hadiah! 🎁✨`
    Rust: (tidak ada padanan)

### maling (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/maling.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/maling.rs`

  - JS: `Beraksi!`
    Rust: (tidak ada padanan)
  - JS: `Nyempil di keramaian pasar... Mengincar tas emak-emak... 🦹‍♂️🤏`
    Rust: (tidak ada padanan)
  - JS: `Result!`
    Rust: (tidak ada padanan)
  - JS: `🦹 COPET`
    Rust: (tidak ada padanan)
  - JS: `🦹 HASIL NYOPET`
    Rust: (tidak ada padanan)

### merchant (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/merchant.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/merchant.rs`

  - JS: `> Contoh: `␀merchant buy potion 5``
    Rust: `> Contoh: `.merchant buy potion 5``
  - JS: `> Contoh: `␀merchant sell iron 10``
    Rust: `> Contoh: `␀merchant ␀ potion 5``
  - JS: `✅ *ᴘᴇɴᴊᴜᴀʟᴀɴ ʙᴇʀʜᴀsɪʟ*`
    Rust: `✅ *ᴘᴇɴᴊᴜᴀɴ ʙᴇʀʜᴀsɪʟ*`

### mining (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/mining.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/mining.rs`

  - JS: `Memecah batu keras di kedalaman gua...`
    Rust: (tidak ada padanan)
  - JS: `Trangg! Tranggg! ⛏️💎`
    Rust: (tidak ada padanan)
  - JS: `⛏️`
    Rust: (tidak ada padanan)

### mulung (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/mulung.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/mulung.rs`

  - JS: `Mengorek tempat sampah dengan penuh harapan... 🗑️👀`
    Rust: (tidak ada padanan)
  - JS: `Semoga hari ini dapet barang bagus! ✨`
    Rust: (tidak ada padanan)
  - JS: `🗑️`
    Rust: (tidak ada padanan)

### nambalban (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/nambalban.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/nambalban.rs`

  - JS: `Ketemu paku nancep! 📍`
    Rust: (tidak ada padanan)
  - JS: `Sssshh... ngecek ban bocor pakai air sabun... 🫧`
    Rust: (tidak ada padanan)
  - JS: `🛠️`
    Rust: (tidak ada padanan)

### ngamen (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/ngamen.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/ngamen.rs`

  - JS: `Mulai jreng-jreng gitar di *␀*... 🎶`
    Rust: (tidak ada padanan)
  - JS: `Semoga hari ini banyak yang ngasih receh! 💸`
    Rust: (tidak ada padanan)
  - JS: `⚡ Stamina: *-␀*`
    Rust: `⚡ Stamina: -␀`

### ngemis (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/ngemis.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/ngemis.rs`

  - JS: `Berharap ada dermawan lewat di perempatan ini... 🚶‍♂️`
    Rust: (tidak ada padanan)
  - JS: `Pak, bu, minta sedekahnya sedikit... 🥺`
    Rust: (tidak ada padanan)

### ngojek (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/ngojek.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/ngojek.rs`

  - JS: `Ada orderan *␀* sejauh *␀* nih, Gasss! 🗺️`
    Rust: (tidak ada padanan)
  - JS: `Nyalain motor, tarik gas! Mencari penumpang... 🏍️💨`
    Rust: (tidak ada padanan)
  - JS: `⚡ Stamina: *-␀*`
    Rust: `⚡ Stamina: *-␀`
  - JS: `🏍️`
    Rust: (tidak ada padanan)

### cerpen (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/nulis.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/nulis.rs`

  - JS: `Merangkai kata demi kata penuh makna... ✍️`
    Rust: (tidak ada padanan)
  - JS: `Semoga ada penerbit yang ngelirik! 📚`
    Rust: (tidak ada padanan)

### nyapu (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/nyapu.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/nyapu.rs`

  - JS: `Membersihkan sampah-sampah masyarakat... 🗑️`
    Rust: (tidak ada padanan)
  - JS: `Srak sruk srak sruk... 🧹`
    Rust: (tidak ada padanan)

### parkir (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/parkir.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/parkir.rs`

  - JS: `Mulai narik duit parkir di minimarket... 💰`
    Rust: (tidak ada padanan)
  - JS: `Priiiit! Terus, terus, bales kiri dikit! 🏁`
    Rust: (tidak ada padanan)
  - JS: `🅿️`
    Rust: (tidak ada padanan)

### pet (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/pet.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/pet.rs`

  - JS: `Sipp! Akte kelahirannya udah diubah.`
    Rust: `Sipp! Akte kelahiran sudah diubah.`
  - JS: `Status barunya nge-reset ke awal tapi jadi jauh lebih kuat lho! Cek pake `.pet status` ya! 😎🔥`
    Rust: `Status barunya nge-reset ke awal tapi jadi jauh lebih kuat lho!`
  - JS: `• Perut: *␀/␀* (␀)`
    Rust: `• Perut: *␀/100* (␀)`
  - JS: `✨ Dapet EXP *+␀*`
    Rust: `✨ Dapet EXP *+␀*␀`
  - JS: `🎉 *MANTAP! Si ␀ LEVEL UP jadi Level ␀!* 🎉`
    Rust: `🎉 *␀ Si ␀ LEVEL UP jadi Level ␀!* 🎉`
  - JS: `🎉 *WOHOO! Si ␀ LEVEL UP jadi Level ␀!* 🎉`
    Rust: `🎉 *␀ Si ␀ LEVEL UP jadi Level ␀!* 🎉`

### petshop (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/petshop.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/petshop.rs`

  - JS: `Dia udah gak sabar pengen jalan-jalan sama kamu. Jangan lupa kasih makan dan cek statusnya pakai `␀pet` ya! 🐾✨`
    Rust: (tidak ada padanan)

### rob (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/rob.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/rob.rs`

  - JS: `*Sssstttt...* Bersembunyi di gang gelap nunggu target lewat... 🦹‍♂️🔪`
    Rust: (tidak ada padanan)
  - JS: `Beraksi!`
    Rust: (tidak ada padanan)
  - JS: `Yaelah, target lu miskin parah! Duitnya di bawah Rp 1.000, masa tega dirampok? Cari mangsa yang tajir dong! 😤`
    Rust: `Yaelah, target lu miskin parah! Duitnya di bawah Rp 1.000, masa tega dirampok?`
  - JS: `🦹 BEGAL`
    Rust: (tidak ada padanan)

### slot (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/slot.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/slot.rs`

  - JS: `Spin!`
    Rust: (tidak ada padanan)
  - JS: `🎰 *TREK TREK TREK...* Tuas ditarik! Mesin slot berputar cepat...`
    Rust: (tidak ada padanan)
  - JS: `🎰 MESIN SLOT`
    Rust: `🎰 *HASIL MESIN SLOT*`
  - JS: `🎰 RESULT`
    Rust: (tidak ada padanan)

### stamina (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/stamina.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/stamina.rs`

  - JS: `⚡ STAMINA`
    Rust: (tidak ada padanan)

### steal (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/steal.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/steal.rs`

  - JS: `*Ngendap-ngendap...* Memanjat tembok rumah *␀*... 🥷⚔️`
    Rust: (tidak ada padanan)
  - JS: `✨ EXP Menyusup: *+␀*`
    Rust: `✨ EXP Menyusup: *+␀*␀`

### streamer (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/streamer.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/streamer.rs`

  - JS: `Halo guys, welcome back to my live stream! 🎮`
    Rust: (tidak ada padanan)
  - JS: `Mari kita unjuk skill main game cacing... 😎`
    Rust: (tidak ada padanan)

### training (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/training.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/training.rs`

  - JS: `🏋️`
    Rust: (tidak ada padanan)
  - JS: `🏋️ *ʟᴀᴛɪʜᴀɴ ␀...*`
    Rust: (tidak ada padanan)

### treasure (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/treasure.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/treasure.rs`

  - JS: `Membuka *␀* secara perlahan... ✨`
    Rust: (tidak ada padanan)
  - JS: `🔓 Mengutak-atik kunci...`
    Rust: (tidak ada padanan)

### use (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/use.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/use.rs`

  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ EXP Ninja: +␀`
    Rust: `> ✨ EXP Ninja: +␀`
  - JS: `␀ EXP: +␀`
    Rust: `> ✨ EXP: +␀`
  - JS: `␀ Koin: +␀`
    Rust: `> 💰 Koin: +␀`
  - JS: `␀ Ryo (Koin): +␀`
    Rust: `> 💰 Ryo (Koin): +␀`
  - JS: `␀ ␀: +␀`
    Rust: `␀: ␀`

### woodcut (rpg)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/rpg/woodcut.js`
- Rust: `/workspaces/oktz-rs/plugin/src/rpg/woodcut.rs`

  - JS: `Ctak! Ctak! Tumbangg! 🪓🌳`
    Rust: (tidak ada padanan)
  - JS: `Menebang pohon besar dengan sekuat tenaga...`
    Rust: (tidak ada padanan)

### android1-get (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/android1-get.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/android1-get.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `🌐 Buka di Browser`
    Rust: (tidak ada padanan)

### android1 (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/android1.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/android1.rs`

  - JS: `APK nya`
    Rust: (tidak ada padanan)
  - JS: `an1_␀`
    Rust: (tidak ada padanan)
  - JS: `␀ • ⭐␀`
    Rust: (tidak ada padanan)
  - JS: `␀. ␀`
    Rust: `*␀.* ␀`
  - JS: `␀android1-get ␀`
    Rust: (tidak ada padanan)

### animeapaini (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/animeapaini.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/animeapaini.rs`

  - JS: `[ANIMECHECKER Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `form-data`
    Rust: (tidak ada padanan)
  - JS: `␀`}`
    Rust: (tidak ada padanan)

### apkmod-get (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/apkmod-get.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/apkmod-get.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `Gagal mengambil detail APK`
    Rust: (tidak ada padanan)

### apkmod (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/apkmod.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/apkmod.rs`

  - JS: `Hasil untuk "␀"`
    Rust: (tidak ada padanan)
  - JS: `␀ • ␀`
    Rust: (tidak ada padanan)
  - JS: `␀. ␀`
    Rust: `*␀.* ␀`
  - JS: `␀apkmod-get ␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `📱 Pilih APK MOD`
    Rust: (tidak ada padanan)

### applemusic (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/applemusic.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/applemusic.rs`

  - JS: `resolve()`
    Rust: (tidak ada padanan)

### spekhp (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/carisinyal.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/carisinyal.rs`

  - JS: `- *Audio Jack:* ␀`
    Rust: `Audio Jack`
  - JS: `- *CPU:* ␀`
    Rust: `- *␀:* ␀`
  - JS: `- *Charging:* ␀`
    Rust: `📶 *Jaringan:* ␀`
  - JS: `- *Chipset:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *GPU:* ␀`
    Rust: `- *␀:* ␀`
  - JS: `- *Jumlah:* ␀`
    Rust: `- *Jumlah:* 1 Kamera`
  - JS: `- *Kapasitas:* ␀`
    Rust: `- *Kapasitas:* 4230 mAh`
  - JS: `- *Konfigurasi:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *NFC:* ␀`
    Rust: `- *␀:* ␀`
  - JS: `- *OS Saat Rilis:* ␀`
    Rust: `OS Saat Rilis`
  - JS: `- *RAM:* ␀`
    Rust: `- *␀:* ␀`
  - JS: `- *Refresh Rate:* ␀`
    Rust: `Refresh Rate`
  - JS: `- *Resolusi:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *Sensor:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *Storage:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *Tipe:* ␀`
    Rust: `- *␀:* ␀`
  - JS: `- *Ukuran:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *Video:* ␀`
    Rust: (tidak ada padanan)
  - JS: `.oxy-post`
    Rust: `oxy-post`
  - JS: `.oxy-post-meta`
    Rust: `oxy-post-meta`
  - JS: `.oxy-post-title`
    Rust: `oxy-post-title`
  - JS: `Bypass Charging`
    Rust: (tidak ada padanan)
  - JS: `Fitur Lainnya`
    Rust: (tidak ada padanan)
  - JS: `Jaminan Update`
    Rust: (tidak ada padanan)
  - JS: `Jenis Memori`
    Rust: (tidak ada padanan)
  - JS: `Jenis RAM`
    Rust: (tidak ada padanan)
  - JS: `Memori Eksternal`
    Rust: `Memori Internal`
  - JS: `Reverse Charging`
    Rust: (tidak ada padanan)
  - JS: `Reverse Wireless Charging`
    Rust: (tidak ada padanan)
  - JS: `SIM Card`
    Rust: (tidak ada padanan)
  - JS: `Wireless Charging`
    Rust: (tidak ada padanan)
  - JS: `meta[name="description"]`
    Rust: (tidak ada padanan)
  - JS: `meta[property="og:image"]`
    Rust: (tidak ada padanan)
  - JS: `table.box-info tr.box-baris`
    Rust: (tidak ada padanan)
  - JS: `td.kolom-dua`
    Rust: `kolom-dua`
  - JS: `td.kolom-satu`
    Rust: `kolom-satu`

### advance-comparation (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/comparation.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/comparation.rs`

  - JS: `*--- ␀ ---*`
    Rust: (tidak ada padanan)
  - JS: `*1. ␀*`
    Rust: (tidak ada padanan)
  - JS: `*2. ␀*`
    Rust: (tidak ada padanan)
  - JS: `*␀ vs ␀`}*`
    Rust: (tidak ada padanan)
  - JS: `*␀*`
    Rust: (tidak ada padanan)
  - JS: `- Tidak ada data.`
    Rust: (tidak ada padanan)
  - JS: `- ␀: ␀`
    Rust: (tidak ada padanan)
  - JS: `> ␀: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> ␀: ␀`
    Rust: `> - ␀`
  - JS: `> 💡 *Kirim angka (contoh: 1)* untuk memilih perangkat kedua, atau ketik `batal` untuk membatalkan.`
    Rust: `> 💡 *Kirim angka (contoh: 1)* untuk memilih perangkat pertama, atau ketik `batal` untuk membatalkan.`
  - JS: `> 💡 _Kirim teks nama perangkat kedua untuk dicari (contoh: iphone 17 pro max)_`
    Rust: (tidak ada padanan)
  - JS: `Gagal mendapatkan data perbandingan.`
    Rust: (tidak ada padanan)
  - JS: `Mau di bandingkan sama apa?`
    Rust: (tidak ada padanan)
  - JS: `Memproses perbandingan antara:`
    Rust: (tidak ada padanan)
  - JS: `Oke, kamu memilih *␀* dengan type *␀* untuk device pertama.`
    Rust: (tidak ada padanan)
  - JS: `Perangkat "␀" tidak ditemukan. Silakan ketik nama perangkat lain yang ingin dicari, atau ketik `batal`.`
    Rust: (tidak ada padanan)
  - JS: `Sesi perbandingan perangkat sudah berakhir.`
    Rust: (tidak ada padanan)
  - JS: `Terjadi kesalahan saat memproses data perbandingan.`
    Rust: (tidak ada padanan)
  - JS: `Terjadi kesalahan saat mencari perangkat kedua. Silakan coba lagi nanti atau ketik `batal`.`
    Rust: (tidak ada padanan)
  - JS: `Tunggu sebentar, data perbandingan sedang diambil...`
    Rust: (tidak ada padanan)
  - JS: `[Compare Fetch Error]`
    Rust: (tidak ada padanan)
  - JS: `[Compare Search 2 Error]`
    Rust: (tidak ada padanan)
  - JS: `[Compare Search Error]`
    Rust: (tidak ada padanan)
  - JS: `⏰ *SESI KEDALUWARSA*`
    Rust: (tidak ada padanan)
  - JS: `⚙️ *SPECIFICATIONS*`
    Rust: (tidak ada padanan)
  - JS: `✅ ␀`
    Rust: (tidak ada padanan)
  - JS: `⭐ *REVIEW SCORES*`
    Rust: (tidak ada padanan)
  - JS: `🏆 *OVERALL SCORE*`
    Rust: (tidak ada padanan)
  - JS: `💡 *REASONS TO CONSIDER*`
    Rust: (tidak ada padanan)
  - JS: `📊 *ADVANCE COMPARISON*`
    Rust: `📊 *ADVANCE COMPARATION*`
  - JS: `📊 *PENCARIAN DEVICE 2: ␀*`
    Rust: `📊 *PENCARIAN DEVICE 1: ␀*`
  - JS: `🚪 Sesi perbandingan dibatalkan.`
    Rust: (tidak ada padanan)

### filmget (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/filmget.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/filmget.rs`

  - JS: `┃ ⏱️ Duration: ␀`
    Rust: `⏱️ Duration`
  - JS: `┃ ⭐ Rating: ␀`
    Rust: `┃ ⭐ Rating: -`
  - JS: `┃ 🎬 Director: ␀`
    Rust: `🎬 Director`
  - JS: `┃ 🎭 Genre: ␀`
    Rust: `┃ 🎭 Genre: Action`
  - JS: `┃ 👥 Actors: ␀`
    Rust: `┃ 👥 Actors: A, B`
  - JS: `┃ 📅 Release: ␀`
    Rust: `┃ 📅 Release: 2024`
  - JS: `┃ 📺 Quality: ␀`
    Rust: `┃ 📺 Quality: -`

### google (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/google.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/google.rs`

  - JS: `❌ Gagal mencari di Google, coba lagi nanti`
    Rust: (tidak ada padanan)

### gsmarena (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/gsmarena.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/gsmarena.rs`

  - JS: `gsmarena-api`
    Rust: (tidak ada padanan)

### mangatoon (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/mangatoon.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/mangatoon.rs`

  - JS: `Hasil Mangatoon tidak ditemukan`
    Rust: (tidak ada padanan)

### mcpedl (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/mcpedl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/mcpedl.rs`

  - JS: `Hasil MCPEDL tidak ditemukan`
    Rust: (tidak ada padanan)

### melolo (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/melolo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/melolo.rs`

  - JS: `Hasil Melolo tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `❌ ␀`
    Rust: (tidak ada padanan)

### movieku (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/movieku.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/movieku.rs`

  - JS: `- ␀: ␀`
    Rust: (tidak ada padanan)
  - JS: `...`
    Rust: (tidak ada padanan)
  - JS: `.entry-content p`
    Rust: `entry-content`
  - JS: `.movieku <judul film>`
    Rust: `> `␀movieku <judul film>``
  - JS: `.movieku avengers`
    Rust: `> `␀movieku avengers``
  - JS: `1080p`
    Rust: (tidak ada padanan)
  - JS: `360p`
    Rust: (tidak ada padanan)
  - JS: `480p`
    Rust: (tidak ada padanan)
  - JS: `720p`
    Rust: (tidak ada padanan)
  - JS: `Cari dan tampilkan informasi film lengkap dari Movieku beserta link download dalam berbagai kualitas`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Country:`
    Rust: (tidak ada padanan)
  - JS: `Director:`
    Rust: (tidak ada padanan)
  - JS: `Duration:`
    Rust: (tidak ada padanan)
  - JS: `Genre:`
    Rust: (tidak ada padanan)
  - JS: `Quality:`
    Rust: (tidak ada padanan)
  - JS: `Rating:`
    Rust: (tidak ada padanan)
  - JS: `Release:`
    Rust: (tidak ada padanan)
  - JS: `Score:`
    Rust: (tidak ada padanan)
  - JS: `Stars:`
    Rust: (tidak ada padanan)
  - JS: `X-Requested-With`
    Rust: (tidak ada padanan)
  - JS: `a[href*="abyssplayer"]`
    Rust: (tidak ada padanan)
  - JS: `action=ts_ac_do_search&ts_ac_query=␀`
    Rust: (tidak ada padanan)
  - JS: `img[src*="wp-content/uploads"]`
    Rust: `wp-content/uploads`
  - JS: `ul li`
    Rust: (tidak ada padanan)
  - JS: `⏱️ Durasi: *␀*`
    Rust: (tidak ada padanan)
  - JS: `▶️ *Streaming:* ␀`
    Rust: (tidak ada padanan)
  - JS: `⭐ Rating: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🌍 Negara: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🌟 Pemeran: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🎬 *␀*`
    Rust: `- *␀*`
  - JS: `🎬 Sutradara: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🎭 Genre: *␀*`
    Rust: (tidak ada padanan)
  - JS: `📀 *␀*`
    Rust: `- *␀*`
  - JS: `📅 Rilis: *␀*`
    Rust: (tidak ada padanan)
  - JS: `📊 Score: *␀*`
    Rust: (tidak ada padanan)
  - JS: `📋 *DETAIL FILM*`
    Rust: (tidak ada padanan)
  - JS: `📝 *Sinopsis:*`
    Rust: (tidak ada padanan)
  - JS: `📺 Kualitas: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🔗 ␀`
    Rust: (tidak ada padanan)
  - JS: `🔽 *LINK DOWNLOAD*`
    Rust: (tidak ada padanan)

### nerdfont-ambil (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/nerdfont-ambil.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/nerdfont-ambil.rs`

  - JS: `"]?([^`
    Rust: (tidak ada padanan)
  - JS: `]+)['`
    Rust: (tidak ada padanan)
  - JS: `a.font-preview`
    Rust: (tidak ada padanan)
  - JS: `a.nf-fa-download`
    Rust: (tidak ada padanan)
  - JS: `a.nf-oct-link_external`
    Rust: (tidak ada padanan)
  - JS: `div.item`
    Rust: (tidak ada padanan)
  - JS: `span.nerd-font-invisible-text`
    Rust: (tidak ada padanan)

### nerdfont (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/nerdfont.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/nerdfont.rs`

  - JS: `"]?([^`
    Rust: `]?([^'`
  - JS: `.nerdfont <query>`
    Rust: (tidak ada padanan)
  - JS: `.nerdfont Coolvetica`
    Rust: `.nerdfont-ambil Coolvetica`
  - JS: `Aku harap, font ini dapat membantu kamu`
    Rust: (tidak ada padanan)
  - JS: `Cari font di DaFont`
    Rust: (tidak ada padanan)
  - JS: `]+)['`
    Rust: (tidak ada padanan)
  - JS: `a.font-preview`
    Rust: `font-preview`
  - JS: `a.nf-fa-download`
    Rust: `nf-fa-download`
  - JS: `a.nf-oct-link_external`
    Rust: `nf-oct-link_external`
  - JS: `div.item`
    Rust: (tidak ada padanan)
  - JS: `span.nerd-font-invisible-text`
    Rust: `nerd-font-invisible-text`

### pap (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/pap.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/pap.rs`

  - JS: `Pilih menu pap lainnya di bawah ini 👇`
    Rust: (tidak ada padanan)
  - JS: `Tidak ada hasil`
    Rust: (tidak ada padanan)
  - JS: `[PAP Search]`
    Rust: (tidak ada padanan)
  - JS: `␀pap cewe`
    Rust: `.pap cewe`
  - JS: `␀pap cowo`
    Rust: (tidak ada padanan)
  - JS: `␀pap femboy`
    Rust: (tidak ada padanan)
  - JS: `␀pap ␀`
    Rust: (tidak ada padanan)
  - JS: `⚧ Femboy`
    Rust: (tidak ada padanan)
  - JS: `👦 Cowo`
    Rust: (tidak ada padanan)
  - JS: `👧 Cewe`
    Rust: (tidak ada padanan)
  - JS: `📸 *PAP ␀*`
    Rust: (tidak ada padanan)
  - JS: `🔁 Next`
    Rust: (tidak ada padanan)

### pddikti (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/pddikti.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/pddikti.rs`

  - JS: `*Contoh:* `.pddikti mhs Gibran Rakabuming``
    Rust: `*Contoh:* `␀pddikti mhs Gibran Rakabuming``
  - JS: `- `.pddikti all <query>``
    Rust: `- `␀pddikti all <query>``
  - JS: `- `.pddikti dosen <nama/NIDN>``
    Rust: `- `␀pddikti dosen <nama/NIDN>``
  - JS: `- `.pddikti mhs <nama/NIM>``
    Rust: `- `␀pddikti mhs <nama/NIM>``
  - JS: `- `.pddikti prodi <nama_prodi>``
    Rust: `- `␀pddikti prodi <nama_prodi>``
  - JS: `- `.pddikti pt <nama_kampus>``
    Rust: `- `␀pddikti pt <nama_kampus>``
  - JS: `[PDDIKTI]`
    Rust: (tidak ada padanan)

### pin (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/pin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/pin.rs`

  - JS: `[Pin] Album gagal, kirim satu-satu:`
    Rust: (tidak ada padanan)
  - JS: `[Pin] Error:`
    Rust: (tidak ada padanan)

### pinvid (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/pinvid.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/pinvid.rs`

  - JS: `[PinVid] Album message failed, sending individually:`
    Rust: (tidak ada padanan)
  - JS: `[PinVid] Error:`
    Rust: (tidak ada padanan)
  - JS: `[PinVid] FFmpeg error for video ␀:`
    Rust: (tidak ada padanan)
  - JS: `[PinVid] Failed to process video ␀:`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg -y -i "␀" -c copy -bsf:a aac_adtstoasc "␀"`
    Rust: (tidak ada padanan)
  - JS: `pinvid_␀_␀.mp4`
    Rust: (tidak ada padanan)

### pixiv (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/pixiv.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/pixiv.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `📝 *ᴋᴜᴇʀʏ:* ␀`
    Rust: `📝 *ᴋᴜᴇʀʛ:* ␀`

### play (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/play.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/play.rs`

  - JS: `Gagal mengambil audio dari API`
    Rust: (tidak ada padanan)
  - JS: `Video tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `YouTube Video`
    Rust: (tidak ada padanan)
  - JS: `[Play]`
    Rust: (tidak ada padanan)
  - JS: `yt-search`
    Rust: (tidak ada padanan)
  - JS: `␀.mp3`
    Rust: (tidak ada padanan)

### play2 (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/play2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/play2.rs`

  - JS: `&& s.videoId) ||`
    Rust: (tidak ada padanan)
  - JS: `'use strict';`
    Rust: (tidak ada padanan)
  - JS: `(function() {`
    Rust: (tidak ada padanan)
  - JS: `) +`
    Rust: (tidak ada padanan)
  - JS: `) > MAX_BASE64_AUDIO_BYTES) {`
    Rust: (tidak ada padanan)
  - JS: `) fail(error);`
    Rust: (tidak ada padanan)
  - JS: `) {`
    Rust: (tidak ada padanan)
  - JS: `).map(Number);`
    Rust: (tidak ada padanan)
  - JS: `).toString(`
    Rust: `.to_string()`
  - JS: `)}``
    Rust: (tidak ada padanan)
  - JS: `)}`;`
    Rust: (tidak ada padanan)
  - JS: `* ========================================================= */`
    Rust: (tidak ada padanan)
  - JS: `* DOWNLOAD AUDIO`
    Rust: (tidak ada padanan)
  - JS: `* FFMPEG COMPRESS`
    Rust: (tidak ada padanan)
  - JS: `* HANDLER`
    Rust: (tidak ada padanan)
  - JS: `* HTML MUSIC PLAYER`
    Rust: (tidak ada padanan)
  - JS: `* MESSAGE SENDER (AIRICH inline HTML via relayMessage)`
    Rust: (tidak ada padanan)
  - JS: `* PLUGIN CONFIG`
    Rust: (tidak ada padanan)
  - JS: `* SAVETUBE`
    Rust: (tidak ada padanan)
  - JS: `* THUMBNAIL`
    Rust: (tidak ada padanan)
  - JS: `* { margin: 0; padding: 0; box-sizing: border-box; -webkit-tap-highlight-color: transparent; }`
    Rust: (tidak ada padanan)
  - JS: `+ String(m).padStart(2,`
    Rust: (tidak ada padanan)
  - JS: `+ String(s).padStart(2,`
    Rust: (tidak ada padanan)
  - JS: `, () => {});`
    Rust: (tidak ada padanan)
  - JS: `, (chunk) => {`
    Rust: (tidak ada padanan)
  - JS: `, (code) => {`
    Rust: (tidak ada padanan)
  - JS: `, (error) => {`
    Rust: (tidak ada padanan)
  - JS: `, error);`
    Rust: (tidak ada padanan)
  - JS: `, position:`
    Rust: (tidak ada padanan)
  - JS: `, quality =`
    Rust: (tidak ada padanan)
  - JS: `, {`
    Rust: (tidak ada padanan)
  - JS: `--ink: #ffffff;`
    Rust: (tidak ada padanan)
  - JS: `--muted: #b9b1b6;`
    Rust: (tidak ada padanan)
  - JS: `--sys: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;`
    Rust: (tidak ada padanan)
  - JS: `.bar { position: relative; height: 4px; border-radius: 4px; background: rgba(255,255,255,.22); cursor: pointer; margin-bottom: 6px; touch-action: none; }`
    Rust: (tidak ada padanan)
  - JS: `.bar__dot { position: absolute; top: 50%; left: 0; width: 11px; height: 11px; border-radius: 50%; background: #fff; transform: translate(-50%,-50%); pointer-events: none; }`
    Rust: (tidak ada padanan)
  - JS: `.bar__fill { position: absolute; left: 0; top: 0; bottom: 0; width: 0; border-radius: 4px; background: #fff; pointer-events: none; }`
    Rust: (tidak ada padanan)
  - JS: `.bg { position: absolute; inset: -30%; width: 160%; height: 160%; object-fit: cover; filter: blur(38px) saturate(1.5); opacity: .85; z-index: 0; }`
    Rust: (tidak ada padanan)
  - JS: `.catch(() => null);`
    Rust: (tidak ada padanan)
  - JS: `.content { position: relative; z-index: 2; padding: 16px 18px 18px; }`
    Rust: (tidak ada padanan)
  - JS: `.controls { display: flex; align-items: center; justify-content: space-between; }`
    Rust: (tidak ada padanan)
  - JS: `.ctrl { width: 34px; height: 34px; display: flex; align-items: center; justify-content: center; color: var(--ink); background: none; border: none; cursor: pointer; padding: 0; transition: opacity .2s ease, transform .15s ease; }`
    Rust: (tidak ada padanan)
  - JS: `.ctrl:active { opacity: .6; transform: scale(.92); }`
    Rust: (tidak ada padanan)
  - JS: `.filter(Boolean)`
    Rust: (tidak ada padanan)
  - JS: `.filter(item => item && Number.isFinite(Number(item.time)))`
    Rust: (tidak ada padanan)
  - JS: `.head { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 14px; }`
    Rust: (tidak ada padanan)
  - JS: `.head__album { font-size: 12px; font-weight: 600; margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }`
    Rust: (tidak ada padanan)
  - JS: `.head__from { font-size: 9px; letter-spacing: .14em; text-transform: uppercase; color: var(--muted); }`
    Rust: (tidak ada padanan)
  - JS: `.head__icon { width: 18px; height: 18px; color: var(--ink); opacity: .85; flex: none; }`
    Rust: (tidak ada padanan)
  - JS: `.head__mid { text-align: center; flex: 1; min-width: 0; }`
    Rust: (tidak ada padanan)
  - JS: `.info { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; margin-bottom: 10px; }`
    Rust: (tidak ada padanan)
  - JS: `.info__artist { font-size: 12px; color: var(--muted); margin-top: 3px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }`
    Rust: (tidak ada padanan)
  - JS: `.info__heart { width: 34px; height: 34px; flex: none; display: flex; align-items: center; justify-content: center; background: none; border: none; color: var(--muted); cursor: pointer; padding: 0; }`
    Rust: (tidak ada padanan)
  - JS: `.info__heart.is-on { color: #ff5c8a; }`
    Rust: (tidak ada padanan)
  - JS: `.info__names { min-width: 0; }`
    Rust: (tidak ada padanan)
  - JS: `.info__title { font-size: 17px; font-weight: 600; line-height: 1.3; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }`
    Rust: (tidak ada padanan)
  - JS: `.join(`
    Rust: (tidak ada padanan)
  - JS: `.jpeg({ quality: 50 })`
    Rust: (tidak ada padanan)
  - JS: `.lyric-line { font-size: 10.5px; color: rgba(255,255,255,.4); line-height: 1.4; transition: color .25s ease, font-size .25s ease, opacity .25s ease, transform .25s ease; text-align: left; white-space: normal; word-wrap: break-word; overflow-wrap: anywhere; font-weight: 500; opacity: .75; transform: translateX(0) scale(1); transform-origin: left center; }`
    Rust: (tidak ada padanan)
  - JS: `.lyric-line.is-active { font-size: 12px; color: #fff; font-weight: 700; opacity: 1; transform: translateX(2px) scale(1.01); }`
    Rust: (tidak ada padanan)
  - JS: `.lyrics-empty { font-size: 10.5px; color: var(--muted); text-align: left; padding: 16px 0; }`
    Rust: (tidak ada padanan)
  - JS: `.lyrics-text { width: 100%; padding: 32px 0; display: flex; flex-direction: column; gap: 5px; }`
    Rust: (tidak ada padanan)
  - JS: `.map((item) =>`
    Rust: (tidak ada padanan)
  - JS: `.map(item => ({ time: Number(item.time), text: String(item.text || '♪') }))`
    Rust: (tidak ada padanan)
  - JS: `.mini-lyrics { position: relative; height: 82px; overflow-x: hidden; overflow-y: auto; scrollbar-width: none; -ms-overflow-style: none; margin-bottom: 10px; overscroll-behavior: contain; scroll-behavior: auto; mask-image: linear-gradient(180deg, transparent 0%, black 18%, black 82%, transparent 100%); -webkit-mask-image: linear-gradient(180deg, transparent 0%, black 18%, black 82%, transparent 100%); }`
    Rust: (tidak ada padanan)
  - JS: `.mini-lyrics::-webkit-scrollbar { display: none; }`
    Rust: (tidak ada padanan)
  - JS: `.note { margin-top: 12px; text-align: center; font-size: 10px; color: var(--muted); line-height: 1.6; }`
    Rust: (tidak ada padanan)
  - JS: `.play { width: 52px; height: 52px; border-radius: 50%; background: #fff; color: #12070b; border: none; cursor: pointer; display: flex; align-items: center; justify-content: center; flex: none; padding: 0; box-shadow: 0 6px 16px rgba(0,0,0,.4); transition: transform .15s ease; }`
    Rust: (tidak ada padanan)
  - JS: `.play:active { transform: scale(.93); }`
    Rust: (tidak ada padanan)
  - JS: `.player { position: relative; width: 100%; max-width: 330px; border-radius: 18px; overflow: hidden; background: #1a0d12; box-shadow: 0 18px 40px rgba(0,0,0,.5); }`
    Rust: (tidak ada padanan)
  - JS: `.poster img { width: 100%; height: 100%; object-fit: cover; display: block; }`
    Rust: (tidak ada padanan)
  - JS: `.poster { width: 100%; aspect-ratio: 1; border-radius: 10px; overflow: hidden; background: rgba(255,255,255,.06); box-shadow: 0 12px 26px rgba(0,0,0,.45); margin-bottom: 14px; }`
    Rust: (tidak ada padanan)
  - JS: `.replace(/'/g,`
    Rust: `.replace(''',`
  - JS: `.resize(250, 250, { fit:`
    Rust: (tidak ada padanan)
  - JS: `.sort((a, b) => a.time - b.time);`
    Rust: (tidak ada padanan)
  - JS: `.then((v) => v.json())`
    Rust: (tidak ada padanan)
  - JS: `.time { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); margin-bottom: 12px; font-variant-numeric: tabular-nums; }`
    Rust: (tidak ada padanan)
  - JS: `.toBuffer();`
    Rust: (tidak ada padanan)
  - JS: `.veil { position: absolute; inset: 0; z-index: 1; background: linear-gradient(180deg, rgba(20,8,12,.65) 0%, rgba(20,8,12,.8) 45%, rgba(12,5,8,.96) 100%); }`
    Rust: (tidak ada padanan)
  - JS: `.wrap { display: flex; align-items: center; justify-content: center; padding: 10px; }`
    Rust: (tidak ada padanan)
  - JS: `/(?:v=|shorts/|youtu.be/|embed/)([a-zA-Z0-9_-]{11})/,`
    Rust: (tidak ada padanan)
  - JS: `/(?:youtu.be/|youtube.com/(?:watch?v=|shorts/|embed/))([a-zA-Z0-9_-]{11})/,`
    Rust: (tidak ada padanan)
  - JS: `/* =========================================================`
    Rust: (tidak ada padanan)
  - JS: `// YTMusic kadang balikin ARTIST/PLAYLIST duluan — ambil entri`
    Rust: (tidak ada padanan)
  - JS: `// pertama yang benar-benar punya videoId (SONG/VIDEO).`
    Rust: (tidak ada padanan)
  - JS: `16k`
    Rust: (tidak ada padanan)
  - JS: `24000`
    Rust: (tidak ada padanan)
  - JS: `:`
    Rust: (tidak ada padanan)
  - JS: `: HEADERS[`
    Rust: (tidak ada padanan)
  - JS: `: error,`
    Rust: (tidak ada padanan)
  - JS: `: item?.lyrics || item?.text || item?.content ||`
    Rust: (tidak ada padanan)
  - JS: `: track.artist?.name ||`
    Rust: (tidak ada padanan)
  - JS: `:root {`
    Rust: (tidak ada padanan)
  - JS: `;`
    Rust: (tidak ada padanan)
  - JS: `</button>`
    Rust: (tidak ada padanan)
  - JS: `</div>`
    Rust: (tidak ada padanan)
  - JS: `</script>`
    Rust: (tidak ada padanan)
  - JS: `</style>`
    Rust: (tidak ada padanan)
  - JS: `<audio id=`
    Rust: (tidak ada padanan)
  - JS: `<button class=`
    Rust: (tidak ada padanan)
  - JS: `<div class=`
    Rust: (tidak ada padanan)
  - JS: `<img class=`
    Rust: (tidak ada padanan)
  - JS: `<img src=`
    Rust: (tidak ada padanan)
  - JS: `<script>`
    Rust: (tidak ada padanan)
  - JS: `<span id=`
    Rust: (tidak ada padanan)
  - JS: `<style>`
    Rust: (tidak ada padanan)
  - JS: `<svg class=`
    Rust: (tidak ada padanan)
  - JS: `<svg id=`
    Rust: (tidak ada padanan)
  - JS: `<svg viewBox=`
    Rust: (tidak ada padanan)
  - JS: `>`
    Rust: (tidak ada padanan)
  - JS: `> ${error?.message ||`
    Rust: (tidak ada padanan)
  - JS: `>0:00</span>`
    Rust: (tidak ada padanan)
  - JS: `>10</text></svg>`
    Rust: (tidak ada padanan)
  - JS: `></audio>`
    Rust: (tidak ada padanan)
  - JS: `></div>`
    Rust: (tidak ada padanan)
  - JS: `><circle cx=`
    Rust: (tidak ada padanan)
  - JS: `><path d=`
    Rust: (tidak ada padanan)
  - JS: `><rect x=`
    Rust: (tidak ada padanan)
  - JS: `>Lirik belum tersedia untuk lagu ini.</div>';`
    Rust: (tidak ada padanan)
  - JS: `>YT Music Audio</div>`
    Rust: (tidak ada padanan)
  - JS: `>support terus kami yaaa</div>`
    Rust: (tidak ada padanan)
  - JS: `>␀</div>`
    Rust: (tidak ada padanan)
  - JS: `>␀</span>`
    Rust: (tidak ada padanan)
  - JS: `? `data:image/jpeg;base64,${thumb.toString(`
    Rust: (tidak ada padanan)
  - JS: `? item`
    Rust: (tidak ada padanan)
  - JS: `? new Error(`
    Rust: (tidak ada padanan)
  - JS: `? track.artists.map((a) => a.name).join(`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Contoh: `␀play2 another love tom odell``,`
    Rust: `Contoh: `␀play2 another love tom odell``
  - JS: `FFMPEG_BITRATE,`
    Rust: (tidak ada padanan)
  - JS: `FFMPEG_CHANNELS,`
    Rust: (tidak ada padanan)
  - JS: `FFMPEG_CODEC,`
    Rust: (tidak ada padanan)
  - JS: `FFMPEG_FORMAT,`
    Rust: (tidak ada padanan)
  - JS: `FFMPEG_SAMPLE_RATE,`
    Rust: (tidak ada padanan)
  - JS: `JSON.stringify(Array.isArray(lyrics) ? lyrics : []),`
    Rust: (tidak ada padanan)
  - JS: `JSON.stringify({`
    Rust: (tidak ada padanan)
  - JS: `METADATA_DECRYPTION_KEY,`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Linux; Android 15) AppleWebKit/537.36 Chrome/130 Mobile Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `WA-Bot/1.0 (https://github.com/)`
    Rust: (tidak ada padanan)
  - JS: `] });`
    Rust: `]);`
  - JS: `] },`
    Rust: (tidak ada padanan)
  - JS: `__typename:`
    Rust: (tidak ada padanan)
  - JS: ``;`
    Rust: (tidak ada padanan)
  - JS: ``⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
  - JS: ``❌ Gagal memproses lagu.`
    Rust: `❌ Gagal memproses lagu.`
  - JS: `activeLyricIndex = -1;`
    Rust: (tidak ada padanan)
  - JS: `activeLyricIndex = index;`
    Rust: (tidak ada padanan)
  - JS: `album = track.album?.name || track.album?.title ||`
    Rust: (tidak ada padanan)
  - JS: `album,`
    Rust: (tidak ada padanan)
  - JS: `alias: [`
    Rust: `alias = [`
  - JS: `alt=`
    Rust: (tidak ada padanan)
  - JS: `aria-label=`
    Rust: (tidak ada padanan)
  - JS: `artist = track.artists?.length`
    Rust: (tidak ada padanan)
  - JS: `artist = vid.author?.name ||`
    Rust: (tidak ada padanan)
  - JS: `artist,`
    Rust: (tidak ada padanan)
  - JS: `async function compressAudio(inputBuffer) {`
    Rust: (tidak ada padanan)
  - JS: `async function downloadAudioBuffer(url) {`
    Rust: (tidak ada padanan)
  - JS: `async function getThumb(url) {`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `async function savetube(url, { downloadType =`
    Rust: (tidak ada padanan)
  - JS: `async function savetubeRetry(url, opts, retry = 3) {`
    Rust: (tidak ada padanan)
  - JS: `async function sendMusicPlayer(sock, jid, html, submessageText) {`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('durationchange', () => {`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('ended', () => {`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('error', setPaused);`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('loadedmetadata', () => {`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('pause', () => { if (!audio.ended) setPaused(); });`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('play', setPlaying);`
    Rust: (tidak ada padanan)
  - JS: `audio.addEventListener('timeupdate', updateProgress);`
    Rust: (tidak ada padanan)
  - JS: `audio.currentTime = (x / rect.width) * duration;`
    Rust: (tidak ada padanan)
  - JS: `audio.currentTime = Math.max(0, (Number(audio.currentTime) || 0) - 10);`
    Rust: (tidak ada padanan)
  - JS: `audio.currentTime = Math.min(duration, (Number(audio.currentTime) || 0) + 10);`
    Rust: (tidak ada padanan)
  - JS: `audio.pause();`
    Rust: (tidak ada padanan)
  - JS: `audioSrc,`
    Rust: (tidak ada padanan)
  - JS: `await audio.play();`
    Rust: (tidak ada padanan)
  - JS: `await m.react(`
    Rust: (tidak ada padanan)
  - JS: `await new Promise((resolve) => setTimeout(resolve, 1000));`
    Rust: (tidak ada padanan)
  - JS: `await sendMusicPlayer(sock, m.chat, html, `␀ - ␀`);`
    Rust: (tidak ada padanan)
  - JS: `await sock.relayMessage(`
    Rust: (tidak ada padanan)
  - JS: `bar.addEventListener('pointerdown', event => seekFromPointer(event.clientX));`
    Rust: (tidak ada padanan)
  - JS: `body: JSON.stringify({`
    Rust: (tidak ada padanan)
  - JS: `botForwardedMessage: {`
    Rust: (tidak ada padanan)
  - JS: `botMetadata: {`
    Rust: (tidak ada padanan)
  - JS: `botResponseId: responseId,`
    Rust: (tidak ada padanan)
  - JS: `btnFw.addEventListener('click', () => {`
    Rust: (tidak ada padanan)
  - JS: `btnRw.addEventListener('click', () => {`
    Rust: (tidak ada padanan)
  - JS: `bytes[i] = binary.charCodeAt(i);`
    Rust: (tidak ada padanan)
  - JS: `category:`
    Rust: `category =`
  - JS: `chunks.push(chunk);`
    Rust: (tidak ada padanan)
  - JS: `console.error(`
    Rust: (tidak ada padanan)
  - JS: `const SCROLL_SMOOTHNESS = 0.18;`
    Rust: (tidak ada padanan)
  - JS: `const active = lyricElements[index];`
    Rust: (tidak ada padanan)
  - JS: `const audio = await savetubeRetry(ytUrl, {`
    Rust: (tidak ada padanan)
  - JS: `const audio = document.getElementById('audio');`
    Rust: (tidak ada padanan)
  - JS: `const audioSrc = `data:audio/ogg;base64,${compressedBuffer.toString(`
    Rust: (tidak ada padanan)
  - JS: `const bar = document.getElementById('bar');`
    Rust: (tidak ada padanan)
  - JS: `const binary = atob(encoded);`
    Rust: (tidak ada padanan)
  - JS: `const btnFw = document.getElementById('btn-fw');`
    Rust: (tidak ada padanan)
  - JS: `const btnRw = document.getElementById('btn-rw');`
    Rust: (tidak ada padanan)
  - JS: `const buffer = Buffer.from(await res.arrayBuffer());`
    Rust: (tidak ada padanan)
  - JS: `const bytes = new Uint8Array(binary.length);`
    Rust: (tidak ada padanan)
  - JS: `const cdn = cdnRes.cdn;`
    Rust: (tidak ada padanan)
  - JS: `const cdnRes = await fetch(`
    Rust: (tidak ada padanan)
  - JS: `const chunks = [];`
    Rust: (tidak ada padanan)
  - JS: `const compressedBuffer = await compressAudio(originalBuffer);`
    Rust: (tidak ada padanan)
  - JS: `const containerHeight = lyricsContainer.clientHeight;`
    Rust: (tidak ada padanan)
  - JS: `const cur = document.getElementById('cur');`
    Rust: (tidak ada padanan)
  - JS: `const current = Math.max(0, Math.min(duration, Number(audio.currentTime) || 0));`
    Rust: (tidak ada padanan)
  - JS: `const decipher = createDecipheriv(`
    Rust: (tidak ada padanan)
  - JS: `const decrypted = Buffer.concat([`
    Rust: (tidak ada padanan)
  - JS: `const detail = await yts(ytUrl);`
    Rust: (tidak ada padanan)
  - JS: `const difference = targetScrollTop - currentScrollTop;`
    Rust: (tidak ada padanan)
  - JS: `const dl = await fetch(`https://␀/download`, {`
    Rust: (tidak ada padanan)
  - JS: `const dot = document.getElementById('dot');`
    Rust: (tidak ada padanan)
  - JS: `const dur = document.getElementById('dur');`
    Rust: (tidak ada padanan)
  - JS: `const duration = Number(audio.duration);`
    Rust: (tidak ada padanan)
  - JS: `const el = document.createElement('div');`
    Rust: (tidak ada padanan)
  - JS: `const elementHeight = element.offsetHeight;`
    Rust: (tidak ada padanan)
  - JS: `const elementTop = element.offsetTop;`
    Rust: (tidak ada padanan)
  - JS: `const encoded = '␀';`
    Rust: (tidak ada padanan)
  - JS: `const encrypted = Buffer.from(info.data,`
    Rust: (tidak ada padanan)
  - JS: `const fail = (error) => {`
    Rust: (tidak ada padanan)
  - JS: `const fill = document.getElementById('fill');`
    Rust: (tidak ada padanan)
  - JS: `const fragment = document.createDocumentFragment();`
    Rust: (tidak ada padanan)
  - JS: `const h = Math.floor(sec / 3600);`
    Rust: (tidak ada padanan)
  - JS: `const h = Math.floor(seconds / 3600);`
    Rust: (tidak ada padanan)
  - JS: `const heart = document.getElementById('heart');`
    Rust: (tidak ada padanan)
  - JS: `const html = createMusicPlayer({`
    Rust: (tidak ada padanan)
  - JS: `const iconPause = document.getElementById('icon-pause');`
    Rust: (tidak ada padanan)
  - JS: `const iconPlay = document.getElementById('icon-play');`
    Rust: (tidak ada padanan)
  - JS: `const idMatch = url.match(`
    Rust: (tidak ada padanan)
  - JS: `const imageSrc = thumb?.length`
    Rust: (tidak ada padanan)
  - JS: `const index = findLyricIndex(time);`
    Rust: (tidak ada padanan)
  - JS: `const info = await fetch(`https://␀/v2/info`, {`
    Rust: (tidak ada padanan)
  - JS: `const json = new TextDecoder('utf-8').decode(bytes);`
    Rust: (tidak ada padanan)
  - JS: `const lrclib = await getLRCLyrics({`
    Rust: (tidak ada padanan)
  - JS: `const lyricElements = [];`
    Rust: (tidak ada padanan)
  - JS: `const lyricsContainer = document.getElementById('mini-lyrics');`
    Rust: (tidak ada padanan)
  - JS: `const lyricsData = await ytm.getLyrics(trackIdForLyrics);`
    Rust: (tidak ada padanan)
  - JS: `const lyricsJson = Buffer.from(`
    Rust: (tidak ada padanan)
  - JS: `const lyricsText = document.getElementById('lyrics-text');`
    Rust: (tidak ada padanan)
  - JS: `const m = Math.floor((seconds % 3600) / 60);`
    Rust: (tidak ada padanan)
  - JS: `const m = Math.floor(sec % 3600 / 60);`
    Rust: (tidak ada padanan)
  - JS: `const match = ytUrl.match(`
    Rust: (tidak ada padanan)
  - JS: `const maxScroll = Math.max(0, lyricsContainer.scrollHeight - containerHeight);`
    Rust: (tidak ada padanan)
  - JS: `const mid = Math.floor((low + high) / 2);`
    Rust: (tidak ada padanan)
  - JS: `const originalBuffer = await downloadAudioBuffer(audio.url);`
    Rust: (tidak ada padanan)
  - JS: `const output = Buffer.concat(chunks);`
    Rust: (tidak ada padanan)
  - JS: `const parsed = JSON.parse(json);`
    Rust: (tidak ada padanan)
  - JS: `const parts = String(timestamp).split(`
    Rust: (tidak ada padanan)
  - JS: `const percent = (current / duration) * 100;`
    Rust: (tidak ada padanan)
  - JS: `const play = document.getElementById('play');`
    Rust: (tidak ada padanan)
  - JS: `const pluginConfig = {`
    Rust: (tidak ada padanan)
  - JS: `const raw = Buffer.from(await res.arrayBuffer());`
    Rust: (tidak ada padanan)
  - JS: `const rect = bar.getBoundingClientRect();`
    Rust: (tidak ada padanan)
  - JS: `const res = await fetch(url);`
    Rust: (tidak ada padanan)
  - JS: `const res = await fetch(url, {`
    Rust: (tidak ada padanan)
  - JS: `const responseId = randomUUID();`
    Rust: (tidak ada padanan)
  - JS: `const s = Math.floor(sec % 60);`
    Rust: (tidak ada padanan)
  - JS: `const s = Math.floor(seconds % 60);`
    Rust: (tidak ada padanan)
  - JS: `const safeArtist = escapeHtml(artist);`
    Rust: (tidak ada padanan)
  - JS: `const safeDuration = escapeHtml(duration ||`
    Rust: (tidak ada padanan)
  - JS: `const safeImage =`
    Rust: (tidak ada padanan)
  - JS: `const safeTitle = escapeHtml(title);`
    Rust: (tidak ada padanan)
  - JS: `const songs = await ytm.search(text);`
    Rust: (tidak ada padanan)
  - JS: `const text = m.text?.trim();`
    Rust: `let text = ctx.text.trim();`
  - JS: `const thumb = await getThumb(thumbUrl);`
    Rust: (tidak ada padanan)
  - JS: `const time = Number(currentTime) || 0;`
    Rust: (tidak ada padanan)
  - JS: `const track =`
    Rust: (tidak ada padanan)
  - JS: `const vid = detail?.videos?.[0];`
    Rust: (tidak ada padanan)
  - JS: `const videoId = idMatch[1];`
    Rust: (tidak ada padanan)
  - JS: `const x = Math.max(0, Math.min(clientX - rect.left, rect.width));`
    Rust: (tidak ada padanan)
  - JS: `const ytm = await getYTMusic();`
    Rust: (tidak ada padanan)
  - JS: `const { command, args } = buildFfmpegCommand([`
    Rust: (tidak ada padanan)
  - JS: `contextInfo: {`
    Rust: (tidak ada padanan)
  - JS: `cooldown: 15,`
    Rust: `cooldown = 15,`
  - JS: `cur.textContent = '0:00';`
    Rust: (tidak ada padanan)
  - JS: `cur.textContent = formatTime(current);`
    Rust: (tidak ada padanan)
  - JS: `currentScrollTop += difference * SCROLL_SMOOTHNESS;`
    Rust: (tidak ada padanan)
  - JS: `currentScrollTop = 0;`
    Rust: (tidak ada padanan)
  - JS: `currentScrollTop = targetScrollTop;`
    Rust: (tidak ada padanan)
  - JS: `cy=`
    Rust: (tidak ada padanan)
  - JS: `data: Buffer.from(`
    Rust: (tidak ada padanan)
  - JS: `decipher.final(),`
    Rust: (tidak ada padanan)
  - JS: `decipher.update(encrypted.subarray(16)),`
    Rust: (tidak ada padanan)
  - JS: `description:`
    Rust: `description =`
  - JS: `deviceListMetadata: {},`
    Rust: (tidak ada padanan)
  - JS: `deviceListMetadataVersion: 2,`
    Rust: (tidak ada padanan)
  - JS: `dot.style.left = '0%';`
    Rust: (tidak ada padanan)
  - JS: `dot.style.left = percent + '%';`
    Rust: (tidak ada padanan)
  - JS: `downloadType,`
    Rust: (tidak ada padanan)
  - JS: `downloadType:`
    Rust: (tidak ada padanan)
  - JS: `duration = formatDuration(durationSec);`
    Rust: (tidak ada padanan)
  - JS: `duration = vid.timestamp ||`
    Rust: (tidak ada padanan)
  - JS: `duration,`
    Rust: (tidak ada padanan)
  - JS: `duration: durationSec,`
    Rust: (tidak ada padanan)
  - JS: `duration: metadata.durationLabel,`
    Rust: (tidak ada padanan)
  - JS: `durationSec = Number(track.duration || 0);`
    Rust: (tidak ada padanan)
  - JS: `durationSec = secondsFromTimestamp(duration);`
    Rust: (tidak ada padanan)
  - JS: `el.className = 'lyric-line';`
    Rust: (tidak ada padanan)
  - JS: `el.dataset.index = String(index);`
    Rust: (tidak ada padanan)
  - JS: `el.dataset.time = String(line.time);`
    Rust: (tidak ada padanan)
  - JS: `el.textContent = line.text || '♪';`
    Rust: (tidak ada padanan)
  - JS: `encrypted.subarray(0, 16),`
    Rust: (tidak ada padanan)
  - JS: `energi: 1,`
    Rust: `energi = 1,`
  - JS: `error?.code ===`
    Rust: (tidak ada padanan)
  - JS: `example:`
    Rust: `example =`
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `fail(`
    Rust: (tidak ada padanan)
  - JS: `fallbackLyrics =`
    Rust: (tidak ada padanan)
  - JS: `fallbackLyrics = lyricsData`
    Rust: (tidak ada padanan)
  - JS: `fallbackLyrics = lyricsData;`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg = spawn(command, args, { stdio: [`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg.kill(`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg.on(`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg.stderr.on(`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg.stdin.end(inputBuffer);`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg.stdin.on(`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg.stdout.on(`
    Rust: (tidak ada padanan)
  - JS: `fill.style.width = '0%';`
    Rust: (tidak ada padanan)
  - JS: `fill.style.width = percent + '%';`
    Rust: (tidak ada padanan)
  - JS: `fill=`
    Rust: (tidak ada padanan)
  - JS: `finished = true;`
    Rust: (tidak ada padanan)
  - JS: `font-family=`
    Rust: (tidak ada padanan)
  - JS: `font-size=`
    Rust: (tidak ada padanan)
  - JS: `font-weight=`
    Rust: (tidak ada padanan)
  - JS: `for (let i = 0; i < binary.length; i++) {`
    Rust: (tidak ada padanan)
  - JS: `for (let i = 0; i < retry; i++) {`
    Rust: (tidak ada padanan)
  - JS: `forwardOrigin: 4,`
    Rust: (tidak ada padanan)
  - JS: `forwardedAiBotMessageInfo: { botJid:`
    Rust: (tidak ada padanan)
  - JS: `forwardingScore: 999,`
    Rust: (tidak ada padanan)
  - JS: `fragment.appendChild(el);`
    Rust: (tidak ada padanan)
  - JS: `function animate() {`
    Rust: (tidak ada padanan)
  - JS: `function calculateLyricScroll(element) {`
    Rust: (tidak ada padanan)
  - JS: `function createMusicPlayer({ title, artist, duration, audioSrc, imageSrc, lyrics }) {`
    Rust: (tidak ada padanan)
  - JS: `function escapeAttr(text =`
    Rust: (tidak ada padanan)
  - JS: `function findLyricIndex(time) {`
    Rust: (tidak ada padanan)
  - JS: `function formatDuration(seconds = 0) {`
    Rust: (tidak ada padanan)
  - JS: `function formatTime(sec) {`
    Rust: (tidak ada padanan)
  - JS: `function renderLyrics() {`
    Rust: (tidak ada padanan)
  - JS: `function resetLyrics() {`
    Rust: (tidak ada padanan)
  - JS: `function secondsFromTimestamp(timestamp =`
    Rust: (tidak ada padanan)
  - JS: `function seekFromPointer(clientX) {`
    Rust: (tidak ada padanan)
  - JS: `function setLyricScrollTarget(element, immediate = false) {`
    Rust: (tidak ada padanan)
  - JS: `function setPaused() {`
    Rust: (tidak ada padanan)
  - JS: `function setPlaying() {`
    Rust: (tidak ada padanan)
  - JS: `function startScrollAnimation() {`
    Rust: (tidak ada padanan)
  - JS: `function updateLyrics(currentTime, { immediate = false } = {}) {`
    Rust: (tidak ada padanan)
  - JS: `function updateProgress() {`
    Rust: (tidak ada padanan)
  - JS: `h +`
    Rust: (tidak ada padanan)
  - JS: `headers: HEADERS,`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `heart.addEventListener('click', () => heart.classList.toggle('is-on'));`
    Rust: (tidak ada padanan)
  - JS: `height=`
    Rust: (tidak ada padanan)
  - JS: `high = mid - 1;`
    Rust: (tidak ada padanan)
  - JS: `html, body { background: transparent; color: var(--ink); font-family: var(--sys); -webkit-font-smoothing: antialiased; }`
    Rust: (tidak ada padanan)
  - JS: `iconPause.style.display = 'block';`
    Rust: (tidak ada padanan)
  - JS: `iconPause.style.display = 'none';`
    Rust: (tidak ada padanan)
  - JS: `iconPlay.style.display = 'block';`
    Rust: (tidak ada padanan)
  - JS: `iconPlay.style.display = 'none';`
    Rust: (tidak ada padanan)
  - JS: `id: videoId,`
    Rust: (tidak ada padanan)
  - JS: `id=`
    Rust: (tidak ada padanan)
  - JS: `if (!/youtube.com|youtu.be/i.test(text)) {`
    Rust: (tidak ada padanan)
  - JS: `if (!Array.isArray(lyrics) || !lyrics.length) {`
    Rust: (tidak ada padanan)
  - JS: `if (!Buffer.isBuffer(inputBuffer) || !inputBuffer.length) {`
    Rust: (tidak ada padanan)
  - JS: `if (!Number.isFinite(duration) || duration <= 0) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!Number.isFinite(sec) || sec < 0) return '0:00';`
    Rust: (tidak ada padanan)
  - JS: `if (!Number.isFinite(seconds) || seconds <= 0) return`
    Rust: (tidak ada padanan)
  - JS: `if (!audio?.url) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!buffer.length || buffer.length > MAX_ORIGINAL_AUDIO_SIZE) {`
    Rust: (tidak ada padanan)
  - JS: `if (!cdnRes?.cdn) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!dl?.data?.downloadUrl) {`
    Rust: (tidak ada padanan)
  - JS: `if (!element || !lyricsContainer) return 0;`
    Rust: (tidak ada padanan)
  - JS: `if (!element || !lyricsContainer) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!idMatch) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!info?.data) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!lyrics.length || !lyricElements.length) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!lyrics.length) return -1;`
    Rust: (tidak ada padanan)
  - JS: `if (!metadata?.key) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!output.length)`
    Rust: (tidak ada padanan)
  - JS: `if (!rect.width) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!res.ok) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!res.ok) throw new Error(`Download audio gagal (␀)`);`
    Rust: (tidak ada padanan)
  - JS: `if (!syncedLyrics.length && lrclib?.plainLyrics) {`
    Rust: (tidak ada padanan)
  - JS: `if (!syncedLyrics.length && trackIdForLyrics) {`
    Rust: (tidak ada padanan)
  - JS: `if (!syncedLyrics.length) {`
    Rust: (tidak ada padanan)
  - JS: `if (!text) {`
    Rust: (tidak ada padanan)
  - JS: `if (!timestamp) return 0;`
    Rust: (tidak ada padanan)
  - JS: `if (!track || !track.videoId) {`
    Rust: (tidak ada padanan)
  - JS: `if (!url) return Buffer.alloc(0);`
    Rust: (tidak ada padanan)
  - JS: `if (!url) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!vid) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (Array.isArray(parsed)) {`
    Rust: (tidak ada padanan)
  - JS: `if (Buffer.byteLength(audioSrc,`
    Rust: (tidak ada padanan)
  - JS: `if (Math.abs(difference) < 0.5) {`
    Rust: (tidak ada padanan)
  - JS: `if (Number(lyrics[mid].time) <= time) {`
    Rust: (tidak ada padanan)
  - JS: `if (Number.isFinite(audio.duration)) dur.textContent = formatTime(audio.duration);`
    Rust: (tidak ada padanan)
  - JS: `if (active) setLyricScrollTarget(active, immediate);`
    Rust: (tidak ada padanan)
  - JS: `if (activeLyricIndex !== -1) {`
    Rust: (tidak ada padanan)
  - JS: `if (audio.paused) {`
    Rust: (tidak ada padanan)
  - JS: `if (code !== 0) return fail(new Error(`FFmpeg gagal (␀)`));`
    Rust: (tidak ada padanan)
  - JS: `if (error?.code !==`
    Rust: (tidak ada padanan)
  - JS: `if (fallbackLyrics) {`
    Rust: (tidak ada padanan)
  - JS: `if (finished) return;`
    Rust: (tidak ada padanan)
  - JS: `if (h > 0) return h + ':' + String(m).padStart(2, '0') + ':' + String(s).padStart(2, '0');`
    Rust: (tidak ada padanan)
  - JS: `if (h > 0) {`
    Rust: `if h > 0 {`
  - JS: `if (i < retry - 1) {`
    Rust: (tidak ada padanan)
  - JS: `if (immediate) {`
    Rust: (tidak ada padanan)
  - JS: `if (index < 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (index === activeLyricIndex && !immediate) return;`
    Rust: (tidak ada padanan)
  - JS: `if (lrclib?.syncedLyrics) {`
    Rust: (tidak ada padanan)
  - JS: `if (lyricsContainer) lyricsContainer.scrollTop = 0;`
    Rust: (tidak ada padanan)
  - JS: `if (match) trackIdForLyrics = match[1];`
    Rust: (tidak ada padanan)
  - JS: `if (outputSize > MAX_COMPRESSED_AUDIO_SIZE) {`
    Rust: (tidak ada padanan)
  - JS: `if (parts.length === 2) return parts[0] * 60 + parts[1];`
    Rust: (tidak ada padanan)
  - JS: `if (parts.length === 3) return parts[0] * 3600 + parts[1] * 60 + parts[2];`
    Rust: (tidak ada padanan)
  - JS: `if (parts.some(Number.isNaN)) return 0;`
    Rust: (tidak ada padanan)
  - JS: `if (scrollAnimationFrame !== null) return;`
    Rust: (tidak ada padanan)
  - JS: `if (track.thumbnails?.length) {`
    Rust: (tidak ada padanan)
  - JS: `if (typeof lyricsData ===`
    Rust: (tidak ada padanan)
  - JS: `imageSrc ||`
    Rust: (tidak ada padanan)
  - JS: `imageSrc,`
    Rust: (tidak ada padanan)
  - JS: `isEnabled: true,`
    Rust: `is_enabled = true,`
  - JS: `isForwarded: true,`
    Rust: (tidak ada padanan)
  - JS: `isGroup: false,`
    Rust: `is_group = false,`
  - JS: `isOwner: false,`
    Rust: `is_owner = false,`
  - JS: `isPremium: false,`
    Rust: `is_premium = false,`
  - JS: `isPrivate: false,`
    Rust: `is_private = false,`
  - JS: `jid,`
    Rust: (tidak ada padanan)
  - JS: `key: metadata.key,`
    Rust: (tidak ada padanan)
  - JS: `lastErr = e;`
    Rust: (tidak ada padanan)
  - JS: `let activeLyricIndex = -1;`
    Rust: (tidak ada padanan)
  - JS: `let album =`
    Rust: (tidak ada padanan)
  - JS: `let artist =`
    Rust: (tidak ada padanan)
  - JS: `let duration =`
    Rust: (tidak ada padanan)
  - JS: `let durationSec = 0;`
    Rust: (tidak ada padanan)
  - JS: `let fallbackLyrics =`
    Rust: (tidak ada padanan)
  - JS: `let ffmpeg;`
    Rust: (tidak ada padanan)
  - JS: `let finished = false;`
    Rust: (tidak ada padanan)
  - JS: `let lastErr;`
    Rust: (tidak ada padanan)
  - JS: `let low = 0, high = lyrics.length - 1, result = -1;`
    Rust: (tidak ada padanan)
  - JS: `let lyrics = [];`
    Rust: (tidak ada padanan)
  - JS: `let metadata;`
    Rust: (tidak ada padanan)
  - JS: `let outputSize = 0;`
    Rust: (tidak ada padanan)
  - JS: `let syncedLyrics = [];`
    Rust: (tidak ada padanan)
  - JS: `let target = elementTop - (containerHeight / 2) + (elementHeight / 2);`
    Rust: (tidak ada padanan)
  - JS: `let targetScrollTop = 0, currentScrollTop = 0, scrollAnimationFrame = null;`
    Rust: (tidak ada padanan)
  - JS: `let thumbUrl =`
    Rust: (tidak ada padanan)
  - JS: `let title =`
    Rust: (tidak ada padanan)
  - JS: `let trackIdForLyrics = null;`
    Rust: (tidak ada padanan)
  - JS: `let ytUrl = text;`
    Rust: (tidak ada padanan)
  - JS: `low = mid + 1;`
    Rust: (tidak ada padanan)
  - JS: `lyricElements.forEach((el, i) => el.classList.toggle('is-active', i === index));`
    Rust: (tidak ada padanan)
  - JS: `lyricElements.forEach(el => el.classList.remove('is-active'));`
    Rust: (tidak ada padanan)
  - JS: `lyricElements.length = 0;`
    Rust: (tidak ada padanan)
  - JS: `lyricElements.push(el);`
    Rust: (tidak ada padanan)
  - JS: `lyrics = [];`
    Rust: (tidak ada padanan)
  - JS: `lyrics = parsed`
    Rust: (tidak ada padanan)
  - JS: `lyrics.forEach((line, index) => {`
    Rust: (tidak ada padanan)
  - JS: `lyrics: syncedLyrics,`
    Rust: (tidak ada padanan)
  - JS: `lyricsContainer.scrollTop = currentScrollTop;`
    Rust: (tidak ada padanan)
  - JS: `lyricsContainer.scrollTop = targetScrollTop;`
    Rust: (tidak ada padanan)
  - JS: `lyricsData?.content ||`
    Rust: (tidak ada padanan)
  - JS: `lyricsData?.lyrics ||`
    Rust: (tidak ada padanan)
  - JS: `lyricsData?.text ||`
    Rust: (tidak ada padanan)
  - JS: `lyricsText.appendChild(fragment);`
    Rust: (tidak ada padanan)
  - JS: `lyricsText.innerHTML = '';`
    Rust: (tidak ada padanan)
  - JS: `lyricsText.innerHTML = '<div class=`
    Rust: (tidak ada padanan)
  - JS: `message: {`
    Rust: (tidak ada padanan)
  - JS: `messageContextInfo: {`
    Rust: (tidak ada padanan)
  - JS: `messageDisclaimerText:`
    Rust: (tidak ada padanan)
  - JS: `messageType: 1,`
    Rust: (tidak ada padanan)
  - JS: `metadata = JSON.parse(decrypted.toString(`
    Rust: (tidak ada padanan)
  - JS: `method:`
    Rust: (tidak ada padanan)
  - JS: `name:`
    Rust: `name =`
  - JS: `outputSize += chunk.length;`
    Rust: (tidak ada padanan)
  - JS: `payload: html,`
    Rust: (tidak ada padanan)
  - JS: `play.addEventListener('click', async () => {`
    Rust: (tidak ada padanan)
  - JS: `preload=`
    Rust: (tidak ada padanan)
  - JS: `primitive: {`
    Rust: (tidak ada padanan)
  - JS: `quality,`
    Rust: (tidak ada padanan)
  - JS: `quality:`
    Rust: (tidak ada padanan)
  - JS: `r=`
    Rust: (tidak ada padanan)
  - JS: `reject(error);`
    Rust: (tidak ada padanan)
  - JS: `renderLyrics();`
    Rust: (tidak ada padanan)
  - JS: `resetLyrics();`
    Rust: (tidak ada padanan)
  - JS: `resolve(output);`
    Rust: (tidak ada padanan)
  - JS: `response_id: responseId,`
    Rust: (tidak ada padanan)
  - JS: `result = mid;`
    Rust: (tidak ada padanan)
  - JS: `return (`
    Rust: `return`
  - JS: `return 0;`
    Rust: `return`
  - JS: `return Buffer.alloc(0);`
    Rust: (tidak ada padanan)
  - JS: `return Math.max(0, Math.min(target, maxScroll));`
    Rust: (tidak ada padanan)
  - JS: `return ``
    Rust: `return`
  - JS: `return await savetube(url, opts);`
    Rust: (tidak ada padanan)
  - JS: `return await sharp(raw)`
    Rust: (tidak ada padanan)
  - JS: `return buffer;`
    Rust: (tidak ada padanan)
  - JS: `return escapeHtml(text);`
    Rust: (tidak ada padanan)
  - JS: `return fail(new Error(`
    Rust: (tidak ada padanan)
  - JS: `return m +`
    Rust: `return`
  - JS: `return m + ':' + String(s).padStart(2, '0');`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return new Promise((resolve, reject) => {`
    Rust: (tidak ada padanan)
  - JS: `return reject(error);`
    Rust: (tidak ada padanan)
  - JS: `return result;`
    Rust: (tidak ada padanan)
  - JS: `return {`
    Rust: `return`
  - JS: `return;`
    Rust: `return`
  - JS: `richResponseMessage: {`
    Rust: (tidak ada padanan)
  - JS: `rx=`
    Rust: (tidak ada padanan)
  - JS: `scrollAnimationFrame = null;`
    Rust: (tidak ada padanan)
  - JS: `scrollAnimationFrame = requestAnimationFrame(animate);`
    Rust: (tidak ada padanan)
  - JS: `sec = Number(sec);`
    Rust: (tidak ada padanan)
  - JS: `seconds = Number(seconds);`
    Rust: (tidak ada padanan)
  - JS: `sections: [`
    Rust: (tidak ada padanan)
  - JS: `setPaused();`
    Rust: (tidak ada padanan)
  - JS: `setPlaying();`
    Rust: (tidak ada padanan)
  - JS: `songs.find((s) => s.type ===`
    Rust: (tidak ada padanan)
  - JS: `songs.find((s) => s.videoId);`
    Rust: (tidak ada padanan)
  - JS: `src=`
    Rust: (tidak ada padanan)
  - JS: `startScrollAnimation();`
    Rust: (tidak ada padanan)
  - JS: `stroke-linecap=`
    Rust: (tidak ada padanan)
  - JS: `stroke-linejoin=`
    Rust: (tidak ada padanan)
  - JS: `stroke-width=`
    Rust: (tidak ada padanan)
  - JS: `stroke=`
    Rust: (tidak ada padanan)
  - JS: `style=`
    Rust: (tidak ada padanan)
  - JS: `submessages: [{ messageType: 2, messageText: submessageText }],`
    Rust: (tidak ada padanan)
  - JS: `syncedLyrics = normalizeLyrics(syncedLyrics);`
    Rust: (tidak ada padanan)
  - JS: `syncedLyrics = parseSyncedLyrics(fallbackLyrics);`
    Rust: (tidak ada padanan)
  - JS: `syncedLyrics = parseSyncedLyrics(lrclib.syncedLyrics);`
    Rust: (tidak ada padanan)
  - JS: `syncedLyrics = plainLyricsToSynced(fallbackLyrics, durationSec);`
    Rust: (tidak ada padanan)
  - JS: `syncedLyrics = plainLyricsToSynced(lrclib.plainLyrics, durationSec);`
    Rust: (tidak ada padanan)
  - JS: `targetScrollTop = 0;`
    Rust: (tidak ada padanan)
  - JS: `targetScrollTop = calculateLyricScroll(element);`
    Rust: (tidak ada padanan)
  - JS: `text-anchor=`
    Rust: (tidak ada padanan)
  - JS: `throw lastErr;`
    Rust: (tidak ada padanan)
  - JS: `throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `throw new Error(dl?.message ||`
    Rust: (tidak ada padanan)
  - JS: `thumbUrl = track.thumbnails[track.thumbnails.length - 1].url;`
    Rust: (tidak ada padanan)
  - JS: `thumbUrl = vid.thumbnail;`
    Rust: (tidak ada padanan)
  - JS: `thumbnail: metadata.thumbnail,`
    Rust: (tidak ada padanan)
  - JS: `title = track.name || track.title ||`
    Rust: (tidak ada padanan)
  - JS: `title = vid.title ||`
    Rust: (tidak ada padanan)
  - JS: `title,`
    Rust: (tidak ada padanan)
  - JS: `title: metadata.title,`
    Rust: (tidak ada padanan)
  - JS: `trackIdForLyrics = track.videoId;`
    Rust: (tidak ada padanan)
  - JS: `trusted_sources: [],`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `typeof item ===`
    Rust: (tidak ada padanan)
  - JS: `unifiedResponse: {`
    Rust: (tidak ada padanan)
  - JS: `updateLyrics(audio.currentTime, { immediate: true });`
    Rust: (tidak ada padanan)
  - JS: `updateLyrics(current);`
    Rust: (tidak ada padanan)
  - JS: `updateProgress();`
    Rust: (tidak ada padanan)
  - JS: `url: `https://www.youtube.com/watch?v=␀`,`
    Rust: (tidak ada padanan)
  - JS: `url: dl.data.downloadUrl,`
    Rust: (tidak ada padanan)
  - JS: `usage:`
    Rust: `usage =`
  - JS: `viewBox=`
    Rust: (tidak ada padanan)
  - JS: `view_model: {`
    Rust: (tidak ada padanan)
  - JS: `while (low <= high) {`
    Rust: (tidak ada padanan)
  - JS: `width=`
    Rust: (tidak ada padanan)
  - JS: `y=`
    Rust: (tidak ada padanan)
  - JS: `yt-search`
    Rust: (tidak ada padanan)
  - JS: `ytUrl = `https://www.youtube.com/watch?v=␀`;`
    Rust: (tidak ada padanan)
  - JS: `ytmusic-api`
    Rust: (tidak ada padanan)
  - JS: `{`
    Rust: (tidak ada padanan)
  - JS: `{ messageId: responseId },`
    Rust: (tidak ada padanan)
  - JS: `} = {}) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (error) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {`
    Rust: (tidak ada padanan)
  - JS: `} catch {}`
    Rust: (tidak ada padanan)
  - JS: `} else if (Array.isArray(lyricsData)) {`
    Rust: (tidak ada padanan)
  - JS: `})`
    Rust: (tidak ada padanan)
  - JS: `})();`
    Rust: `));`
  - JS: `}),`
    Rust: `),`
  - JS: `});`
    Rust: `};`
  - JS: `},`
    Rust: (tidak ada padanan)
  - JS: `}`,`
    Rust: (tidak ada padanan)
  - JS: `␀/get?␀`
    Rust: (tidak ada padanan)

### playcall (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/playcall.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/playcall.rs`

  - JS: `.message`
    Rust: (tidak ada padanan)
  - JS: `CapabilityError.tried`
    Rust: (tidak ada padanan)
  - JS: `[PlayCall] gagal — ␀`
    Rust: (tidak ada padanan)
  - JS: `axios.get`
    Rust: (tidak ada padanan)
  - JS: `call_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `yt-search`
    Rust: (tidak ada padanan)
  - JS: `␀: ␀`
    Rust: (tidak ada padanan)

### playch (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/playch.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/playch.rs`

  - JS: `.message`
    Rust: (tidak ada padanan)
  - JS: `CapabilityError.tried`
    Rust: (tidak ada padanan)
  - JS: `Gagal mengambil audio dari API`
    Rust: (tidak ada padanan)
  - JS: `Konversi opus gagal`
    Rust: (tidak ada padanan)
  - JS: `[PlayCh] gagal — ␀`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg -y -i "␀" -vn -map_metadata -1 -ac 1 -ar 48000 -c:a libopus -b:a 96k -vbr on -application audio -f ogg "␀"`
    Rust: (tidak ada padanan)
  - JS: `in_*.mp3`
    Rust: (tidak ada padanan)
  - JS: `in_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `out_␀.ogg`
    Rust: (tidak ada padanan)
  - JS: `yt-search`
    Rust: (tidak ada padanan)
  - JS: `␀: ␀`
    Rust: (tidak ada padanan)

### playsoundcloud (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/playsoundcloud.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/playsoundcloud.rs`

  - JS: `Error: ␀`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mendownload lagu kak! 😭`
    Rust: (tidak ada padanan)

### playvid (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/playvid.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/playvid.rs`

  - JS: `CapabilityError.message`
    Rust: (tidak ada padanan)
  - JS: `[PlayVid] gagal — ␀`
    Rust: (tidak ada padanan)
  - JS: `semua backend gagal`
    Rust: (tidak ada padanan)
  - JS: `yt-search`
    Rust: (tidak ada padanan)
  - JS: `␀: ␀`
    Rust: (tidak ada padanan)

### resep (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/resep.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/resep.rs`

  - JS: `Maaf, gagal mengambil detail resep untuk "␀".`
    Rust: (tidak ada padanan)
  - JS: `Sesi pencarian resep sudah berakhir karena lebih dari 5 menit. Silakan ketik perintah .resep lagi.`
    Rust: (tidak ada padanan)
  - JS: `Terjadi kesalahan saat memuat resep. Silakan coba lagi.`
    Rust: (tidak ada padanan)
  - JS: `[RESEP Detail Error]`
    Rust: (tidak ada padanan)
  - JS: `[RESEP Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `⏰ *SESI KEDALUWARSA*`
    Rust: (tidak ada padanan)
  - JS: `🚪 Pencarian resep dibatalkan.`
    Rust: (tidak ada padanan)

### searchthatsong (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/searchthatsong.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/searchthatsong.rs`

  - JS: `Content-Length`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `[SearchThatSong]`
    Rust: (tidak ada padanan)
  - JS: `request timeout`
    Rust: (tidak ada padanan)
  - JS: `␀/`
    Rust: (tidak ada padanan)

### soundcloud (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/soundcloud.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/soundcloud.rs`

  - JS: `#__NEXT_DATA__`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)

### spotify (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/spotify.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/spotify.rs`

  - JS: `.spdl`
    Rust: `.spdl ␀`
  - JS: `[Spotify Search]`
    Rust: `🚀 Spotify Search`
  - JS: `tekan tombol di bawah ini`
    Rust: (tidak ada padanan)

### spotplay (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/spotplay.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/spotplay.rs`

  - JS: `.spotplay`
    Rust: (tidak ada padanan)
  - JS: `[Spotplay Error]`
    Rust: (tidak ada padanan)
  - JS: `␀ - ␀.mp3`
    Rust: (tidak ada padanan)

### tiktokfoto (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/tiktokfoto.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/tiktokfoto.rs`

  - JS: `user-agent`
    Rust: (tidak ada padanan)
  - JS: `x-api-key`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal memuat foto TikTok`
    Rust: (tidak ada padanan)

### ttsearch (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/ttsearch.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/ttsearch.rs`

  - JS: `㊗ ᴜsᴀɢᴇ: `␀ttsearch <query>``
    Rust: `㊗ ᴅsᴀɢᴇ: `␀ttsearch <query>``

### wikipedia (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/wikipedia.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/wikipedia.rs`

  - JS: `.infobox tr`
    Rust: (tidak ada padanan)
  - JS: `.mw-parser-output > p`
    Rust: `mw-parser-output`
  - JS: `.mw-parser-output > section`
    Rust: `mw-parser-output`
  - JS: `.mw-parser-output img`
    Rust: `mw-parser-output`
  - JS: `.tagline`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Mobile Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `[Wikipedia Search]`
    Rust: (tidak ada padanan)
  - JS: `accept-language`
    Rust: (tidak ada padanan)
  - JS: `daftar isi`
    Rust: (tidak ada padanan)
  - JS: `h2, h3`
    Rust: (tidak ada padanan)
  - JS: `id-ID,id;q=0.9,en-US;q=0.8,en;q=0.7`
    Rust: (tidak ada padanan)
  - JS: `p, ul, ol`
    Rust: (tidak ada padanan)
  - JS: `script, style, noscript, sup.reference, .mw-editsection, .navbox, .metadata, .ambox, .hatnote, .toc, #toc, table.vertical-navbox`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)
  - JS: `␀/w/api.php`
    Rust: (tidak ada padanan)

### ytplay (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/ytplay.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/ytplay.rs`

  - JS: `)`
    Rust: (tidak ada padanan)
  - JS: `)";`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `*{`
    Rust: (tidak ada padanan)
  - JS: `+ video.poster +`
    Rust: (tidak ada padanan)
  - JS: `-webkit-overflow-scrolling:touch`
    Rust: (tidak ada padanan)
  - JS: `-webkit-tap-highlight-color:transparent;`
    Rust: (tidak ada padanan)
  - JS: `-webkit-touch-callout:none;`
    Rust: (tidak ada padanan)
  - JS: `-webkit-user-select:none;`
    Rust: (tidak ada padanan)
  - JS: `.artist{`
    Rust: (tidak ada padanan)
  - JS: `.bg-overlay{`
    Rust: (tidak ada padanan)
  - JS: `.bg{`
    Rust: (tidak ada padanan)
  - JS: `.big-btn svg{`
    Rust: (tidak ada padanan)
  - JS: `.big-btn{`
    Rust: (tidak ada padanan)
  - JS: `.big-play{`
    Rust: (tidak ada padanan)
  - JS: `.bottom-left,`
    Rust: (tidak ada padanan)
  - JS: `.bottom-right{`
    Rust: (tidak ada padanan)
  - JS: `.bottom{`
    Rust: (tidak ada padanan)
  - JS: `.content{`
    Rust: (tidak ada padanan)
  - JS: `.controls{`
    Rust: (tidak ada padanan)
  - JS: `.debug-bar-fill{`
    Rust: (tidak ada padanan)
  - JS: `.debug-bar{`
    Rust: (tidak ada padanan)
  - JS: `.debug-cell b{`
    Rust: (tidak ada padanan)
  - JS: `.debug-cell span{`
    Rust: (tidak ada padanan)
  - JS: `.debug-cell{`
    Rust: (tidak ada padanan)
  - JS: `.debug-grid{`
    Rust: (tidak ada padanan)
  - JS: `.debug-head{`
    Rust: (tidak ada padanan)
  - JS: `.debug-log{`
    Rust: (tidak ada padanan)
  - JS: `.debug-server{`
    Rust: (tidak ada padanan)
  - JS: `.debug-state{`
    Rust: (tidak ada padanan)
  - JS: `.debug-title{`
    Rust: (tidak ada padanan)
  - JS: `.debug-wrap{`
    Rust: (tidak ada padanan)
  - JS: `.frame-playing .big-play{`
    Rust: (tidak ada padanan)
  - JS: `.icon-btn svg{`
    Rust: (tidak ada padanan)
  - JS: `.icon-btn{`
    Rust: (tidak ada padanan)
  - JS: `.info{`
    Rust: (tidak ada padanan)
  - JS: `.log-line{`
    Rust: (tidak ada padanan)
  - JS: `.player-wrap{`
    Rust: (tidak ada padanan)
  - JS: `.player{`
    Rust: (tidak ada padanan)
  - JS: `.progress{`
    Rust: (tidak ada padanan)
  - JS: `.replace(/`/g, "\`")`
    Rust: (tidak ada padanan)
  - JS: `.replace(/␀`
    Rust: (tidak ada padanan)
  - JS: `.side-btn svg{`
    Rust: (tidak ada padanan)
  - JS: `.side-btn{`
    Rust: (tidak ada padanan)
  - JS: `.song-title{`
    Rust: (tidak ada padanan)
  - JS: `.times{`
    Rust: (tidak ada padanan)
  - JS: `.top-sub{`
    Rust: (tidak ada padanan)
  - JS: `.top-title{`
    Rust: (tidak ada padanan)
  - JS: `.top{`
    Rust: (tidak ada padanan)
  - JS: `.video-frame video{`
    Rust: (tidak ada padanan)
  - JS: `.video-frame{`
    Rust: (tidak ada padanan)
  - JS: `.volume{`
    Rust: (tidak ada padanan)
  - JS: `180deg,`
    Rust: (tidak ada padanan)
  - JS: `: "";`
    Rust: (tidak ada padanan)
  - JS: `</button>`
    Rust: (tidak ada padanan)
  - JS: `</div>`
    Rust: (tidak ada padanan)
  - JS: `</script>`;`
    Rust: (tidak ada padanan)
  - JS: `</style>`
    Rust: (tidak ada padanan)
  - JS: `</svg>`
    Rust: (tidak ada padanan)
  - JS: `<b id="dbgChunks">0</b>`
    Rust: (tidak ada padanan)
  - JS: `<b id="dbgDownloaded">0.00 MB</b>`
    Rust: (tidak ada padanan)
  - JS: `<b id="dbgLoop">OFF</b>`
    Rust: (tidak ada padanan)
  - JS: `<b id="dbgMime">-</b>`
    Rust: (tidak ada padanan)
  - JS: `<b id="dbgProgress">0%</b>`
    Rust: (tidak ada padanan)
  - JS: `<b id="dbgTotal">-</b>`
    Rust: (tidak ada padanan)
  - JS: `<body>`
    Rust: (tidak ada padanan)
  - JS: `<button class="big-btn" type="button" onclick="event.stopPropagation();toggleVideo()">`
    Rust: (tidak ada padanan)
  - JS: `<button class="icon-btn" type="button" onclick="toggleMute()" id="muteButton">`
    Rust: (tidak ada padanan)
  - JS: `<button class="icon-btn" type="button" onclick="toggleMute()">`
    Rust: (tidak ada padanan)
  - JS: `<button class="icon-btn" type="button" onclick="toggleRepeat()" id="repeatButton">`
    Rust: (tidak ada padanan)
  - JS: `<button class="side-btn" type="button" onclick="nextVideo()">`
    Rust: (tidak ada padanan)
  - JS: `<button class="side-btn" type="button" onclick="previousVideo()">`
    Rust: (tidak ada padanan)
  - JS: `<button class="side-btn" type="button" onclick="toggleVideo()">`
    Rust: (tidak ada padanan)
  - JS: `<div class="artist" id="artist">Unknown Channel</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="bg" id="background"></div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="bg-overlay"></div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="big-play" id="bigPlay">`
    Rust: (tidak ada padanan)
  - JS: `<div class="bottom">`
    Rust: (tidak ada padanan)
  - JS: `<div class="bottom-left">`
    Rust: (tidak ada padanan)
  - JS: `<div class="bottom-right">`
    Rust: (tidak ada padanan)
  - JS: `<div class="content">`
    Rust: (tidak ada padanan)
  - JS: `<div class="controls">`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-bar">`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-bar-fill" id="dbgBarFill"></div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-cell">`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-grid">`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-head">`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-log" id="debugLog"></div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-server" id="wsServer">-</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-state" id="wsState">CONNECTING</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-title">WEBSOCKET VIDEO</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="debug-wrap">`
    Rust: (tidak ada padanan)
  - JS: `<div class="info">`
    Rust: (tidak ada padanan)
  - JS: `<div class="player">`
    Rust: (tidak ada padanan)
  - JS: `<div class="player-wrap">`
    Rust: (tidak ada padanan)
  - JS: `<div class="progress">`
    Rust: (tidak ada padanan)
  - JS: `<div class="song-title" id="songTitle">Unknown Video</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="times">`
    Rust: (tidak ada padanan)
  - JS: `<div class="top">`
    Rust: (tidak ada padanan)
  - JS: `<div class="top-sub" id="playlistSub">␀</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="top-title" id="playlistTitle">NOW PLAYING VIDEO</div>`
    Rust: (tidak ada padanan)
  - JS: `<div class="video-frame" id="videoFrame" onclick="toggleVideo()">`
    Rust: (tidak ada padanan)
  - JS: `<div>`
    Rust: (tidak ada padanan)
  - JS: `<input class="volume" id="volume" type="range" min="0" max="1" step="0.01" value="1" oninput="changeVolume(this.value)">`
    Rust: (tidak ada padanan)
  - JS: `<input id="progress" type="range" min="0" max="100" value="0" step="0.1" oninput="seekVideo(this.value)">`
    Rust: (tidak ada padanan)
  - JS: `<path d="M11 5 6 9H2v6h4l5 4V5z"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M16 6.5a9 9 0 0 1 0 11"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M17 2l4 4-4 4"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M17 5v14"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M19 20 9 12l10-8v16z"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M19 5v14"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M19 9a5 5 0 0 1 0 6"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M21 13v2a3 3 0 0 1-3 3H3"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M3 11V9a3 3 0 0 1 3-3h15"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M5 19V5"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M7 5v14"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="M8 5v14l11-7z"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="m14 9 5 6"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="m19 9-5 6"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="m5 4 10 8-10 8V4z"></path>`
    Rust: (tidak ada padanan)
  - JS: `<path d="m7 22-4-4 4-4"></path>`
    Rust: (tidak ada padanan)
  - JS: `<script>`
    Rust: (tidak ada padanan)
  - JS: `<span id="currentTime">0:00</span>`
    Rust: (tidak ada padanan)
  - JS: `<span id="duration">0:00</span>`
    Rust: (tidak ada padanan)
  - JS: `<span>Chunks</span>`
    Rust: (tidak ada padanan)
  - JS: `<span>Downloaded</span>`
    Rust: (tidak ada padanan)
  - JS: `<span>Loop</span>`
    Rust: (tidak ada padanan)
  - JS: `<span>MIME</span>`
    Rust: (tidak ada padanan)
  - JS: `<span>Progress</span>`
    Rust: (tidak ada padanan)
  - JS: `<span>Total</span>`
    Rust: (tidak ada padanan)
  - JS: `<svg id="pauseIcon" viewBox="0 0 24 24" style="display:none">`
    Rust: (tidak ada padanan)
  - JS: `<svg id="playIcon" viewBox="0 0 24 24">`
    Rust: (tidak ada padanan)
  - JS: `<svg id="smallPauseIcon" viewBox="0 0 24 24" style="display:none">`
    Rust: (tidak ada padanan)
  - JS: `<svg id="smallPlayIcon" viewBox="0 0 24 24">`
    Rust: (tidak ada padanan)
  - JS: `<svg id="volumeIcon" viewBox="0 0 24 24">`
    Rust: (tidak ada padanan)
  - JS: `<svg viewBox="0 0 24 24">`
    Rust: (tidak ada padanan)
  - JS: `<video id="videoPlayer" playsinline webkit-playsinline preload="none" poster="␀"></video>`
    Rust: (tidak ada padanan)
  - JS: `? `␀:␀``
    Rust: (tidak ada padanan)
  - JS: `Contoh: ␀ytplay melukis senja``
    Rust: `Contoh: ␀ytplay melukis senja`
  - JS: `URL.revokeObjectURL(wsObjectUrl);`
    Rust: (tidak ada padanan)
  - JS: `\\`
    Rust: (tidak ada padanan)
  - JS: `__typename: "FOAHtmlPrimitiveDemoDONOTUSE",`
    Rust: (tidak ada padanan)
  - JS: `__typename: "GenAISingleLayoutViewModel",`
    Rust: (tidak ada padanan)
  - JS: `__typename: "GenAIUnifiedResponseSection",`
    Rust: (tidak ada padanan)
  - JS: ``Mana judul atau link video YouTube yang mau diputar?`
    Rust: `Mana judul atau link video YouTube yang mau diputar?`
  - JS: `accent-color:#fff;`
    Rust: (tidak ada padanan)
  - JS: `aiRich.addSection({`
    Rust: (tidak ada padanan)
  - JS: `align-items:center;`
    Rust: (tidak ada padanan)
  - JS: `artist.textContent = config.channel || "Unknown Channel";`
    Rust: (tidak ada padanan)
  - JS: `aspect-ratio:16/9;`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `autoplay: false,`
    Rust: (tidak ada padanan)
  - JS: `await aiRich.send(m.chat, { includesUnifiedResponse: true });`
    Rust: (tidak ada padanan)
  - JS: `await m.react("☢");`
    Rust: (tidak ada padanan)
  - JS: `await m.react("✅");`
    Rust: (tidak ada padanan)
  - JS: `await m.react("🕕");`
    Rust: (tidak ada padanan)
  - JS: `await m.reply(`Gagal memutar video YouTube.`
    Rust: `Gagal memutar video YouTube.`
  - JS: `background-position:center;`
    Rust: (tidak ada padanan)
  - JS: `background-size:cover;`
    Rust: (tidak ada padanan)
  - JS: `background.style.backgroundImage = "url(`
    Rust: (tidak ada padanan)
  - JS: `background:#000;`
    Rust: (tidak ada padanan)
  - JS: `background:#090a0f;`
    Rust: (tidak ada padanan)
  - JS: `background:#111318;`
    Rust: (tidak ada padanan)
  - JS: `background:#fff;`
    Rust: (tidak ada padanan)
  - JS: `background:linear-gradient(`
    Rust: (tidak ada padanan)
  - JS: `background:linear-gradient(90deg,#4cc9f0,#7bff9f);`
    Rust: (tidak ada padanan)
  - JS: `background:rgba(0,0,0,.18);`
    Rust: (tidak ada padanan)
  - JS: `background:rgba(0,0,0,.35);`
    Rust: (tidak ada padanan)
  - JS: `background:rgba(255,255,255,.05);`
    Rust: (tidak ada padanan)
  - JS: `background:rgba(255,255,255,.08);`
    Rust: (tidak ada padanan)
  - JS: `background:transparent;`
    Rust: (tidak ada padanan)
  - JS: `body{`
    Rust: (tidak ada padanan)
  - JS: `border-radius:15px;`
    Rust: (tidak ada padanan)
  - JS: `border-radius:16px;`
    Rust: (tidak ada padanan)
  - JS: `border-radius:20px;`
    Rust: (tidak ada padanan)
  - JS: `border-radius:4px;`
    Rust: (tidak ada padanan)
  - JS: `border-radius:50%;`
    Rust: (tidak ada padanan)
  - JS: `border-radius:8px;`
    Rust: (tidak ada padanan)
  - JS: `border:0;`
    Rust: (tidak ada padanan)
  - JS: `border:1px solid rgba(255,255,255,.12);`
    Rust: (tidak ada padanan)
  - JS: `box-shadow:0 10px 40px rgba(0,0,0,.55)`
    Rust: (tidak ada padanan)
  - JS: `box-shadow:0 12px 35px rgba(0,0,0,.45);`
    Rust: (tidak ada padanan)
  - JS: `box-shadow:0 6px 18px rgba(0,0,0,.35)`
    Rust: (tidak ada padanan)
  - JS: `box-sizing:border-box`
    Rust: (tidak ada padanan)
  - JS: `box.appendChild(div);`
    Rust: (tidak ada padanan)
  - JS: `box.removeChild(box.firstChild);`
    Rust: (tidak ada padanan)
  - JS: `box.scrollTop = box.scrollHeight;`
    Rust: (tidak ada padanan)
  - JS: `channel: "␀",`
    Rust: `📺 Channel: ␀`
  - JS: `channel: info.channel || yt.channel || "YouTube",`
    Rust: (tidak ada padanan)
  - JS: `color:#08090d;`
    Rust: (tidak ada padanan)
  - JS: `color:#ffd166;`
    Rust: (tidak ada padanan)
  - JS: `color:#fff;`
    Rust: (tidak ada padanan)
  - JS: `color:rgba(255,255,255,.75);`
    Rust: (tidak ada padanan)
  - JS: `const aiRich = new AIRich(sock);`
    Rust: (tidak ada padanan)
  - JS: `const dur = info.duration`
    Rust: (tidak ada padanan)
  - JS: `const htmlPayload = buildPlayerHtml({`
    Rust: (tidak ada padanan)
  - JS: `const info = await getYoutubeDirectUrl(yt.url, "360");`
    Rust: (tidak ada padanan)
  - JS: `const query = m.text?.trim();`
    Rust: (tidak ada padanan)
  - JS: `const safeChannel = escapeHtmlJs(channel);`
    Rust: (tidak ada padanan)
  - JS: `const safeDuration = escapeHtmlJs(duration);`
    Rust: (tidak ada padanan)
  - JS: `const safeTitle = escapeHtmlJs(title);`
    Rust: (tidak ada padanan)
  - JS: `const safeVideoUrl = escapeHtmlJs(videoUrl);`
    Rust: (tidak ada padanan)
  - JS: `const safeWsUrl = escapeHtmlJs(wsUrl);`
    Rust: (tidak ada padanan)
  - JS: `const thumbRes = await axios.get(info.thumb, {`
    Rust: (tidak ada padanan)
  - JS: `const titleHeader = `YOUTUBE PLAY - ␀`;`
    Rust: (tidak ada padanan)
  - JS: `const wsUrl = `wss://stream.kanara.my.id/?url=␀`;`
    Rust: (tidak ada padanan)
  - JS: `const yt = await searchYoutube(query);`
    Rust: (tidak ada padanan)
  - JS: `currentTime.textContent = "0:00";`
    Rust: (tidak ada padanan)
  - JS: `currentTime.textContent = formatTime(video.currentTime);`
    Rust: (tidak ada padanan)
  - JS: `cursor:pointer`
    Rust: (tidak ada padanan)
  - JS: `display:block;`
    Rust: (tidak ada padanan)
  - JS: `display:flex;`
    Rust: (tidak ada padanan)
  - JS: `display:grid;`
    Rust: (tidak ada padanan)
  - JS: `div.className = "log-line";`
    Rust: (tidak ada padanan)
  - JS: `div.textContent = line;`
    Rust: (tidak ada padanan)
  - JS: `duration: "␀",`
    Rust: (tidak ada padanan)
  - JS: `duration: dur,`
    Rust: (tidak ada padanan)
  - JS: `durationLabel.textContent = formatTime(video.duration);`
    Rust: (tidak ada padanan)
  - JS: `el = document.getElementById("dbgBarFill");`
    Rust: (tidak ada padanan)
  - JS: `el = document.getElementById("dbgChunks");`
    Rust: (tidak ada padanan)
  - JS: `el = document.getElementById("dbgProgress");`
    Rust: (tidak ada padanan)
  - JS: `el = document.getElementById("dbgTotal");`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `fill:currentColor;`
    Rust: (tidak ada padanan)
  - JS: `fill:none;`
    Rust: (tidak ada padanan)
  - JS: `filter:blur(22px);`
    Rust: (tidak ada padanan)
  - JS: `finishWsDownload(config);`
    Rust: (tidak ada padanan)
  - JS: `flex-shrink:0`
    Rust: (tidak ada padanan)
  - JS: `font-family:Arial,Helvetica,sans-serif;`
    Rust: (tidak ada padanan)
  - JS: `font-family:monospace`
    Rust: (tidak ada padanan)
  - JS: `font-size:10px;`
    Rust: (tidak ada padanan)
  - JS: `font-size:11px;`
    Rust: (tidak ada padanan)
  - JS: `font-size:13px;`
    Rust: (tidak ada padanan)
  - JS: `font-size:19px;`
    Rust: (tidak ada padanan)
  - JS: `font-size:8px;`
    Rust: (tidak ada padanan)
  - JS: `font-size:9px;`
    Rust: (tidak ada padanan)
  - JS: `font-weight:700;`
    Rust: (tidak ada padanan)
  - JS: `frame.classList.add("frame-playing");`
    Rust: (tidak ada padanan)
  - JS: `frame.classList.remove("frame-playing");`
    Rust: (tidak ada padanan)
  - JS: `function buildPlayerHtml({ title, channel, duration, videoUrl, wsUrl, posterBase64 }) {`
    Rust: (tidak ada padanan)
  - JS: `function changeVolume(value) {`
    Rust: (tidak ada padanan)
  - JS: `function finishWsDownload(cfg) {`
    Rust: (tidak ada padanan)
  - JS: `function fmtSize(bytes) {`
    Rust: (tidak ada padanan)
  - JS: `function formatTime(seconds) {`
    Rust: (tidak ada padanan)
  - JS: `function nextVideo() {`
    Rust: (tidak ada padanan)
  - JS: `function playVideo() {`
    Rust: (tidak ada padanan)
  - JS: `function previousVideo() {`
    Rust: (tidak ada padanan)
  - JS: `function seekVideo(value) {`
    Rust: (tidak ada padanan)
  - JS: `function setWsState(state) {`
    Rust: (tidak ada padanan)
  - JS: `function startVideoStream(cfg) {`
    Rust: (tidak ada padanan)
  - JS: `function toggleMute() {`
    Rust: (tidak ada padanan)
  - JS: `function toggleRepeat() {`
    Rust: (tidak ada padanan)
  - JS: `function toggleVideo() {`
    Rust: (tidak ada padanan)
  - JS: `function updateLoopStatus() {`
    Rust: (tidak ada padanan)
  - JS: `function updateUI() {`
    Rust: (tidak ada padanan)
  - JS: `function updateWsProgress() {`
    Rust: (tidak ada padanan)
  - JS: `function writeLog(line) {`
    Rust: (tidak ada padanan)
  - JS: `gap:12px`
    Rust: (tidak ada padanan)
  - JS: `gap:18px;`
    Rust: (tidak ada padanan)
  - JS: `gap:6px;`
    Rust: (tidak ada padanan)
  - JS: `gap:9px`
    Rust: (tidak ada padanan)
  - JS: `grid-template-columns:repeat(3,1fr);`
    Rust: (tidak ada padanan)
  - JS: `height:100%;`
    Rust: (tidak ada padanan)
  - JS: `height:110px;`
    Rust: (tidak ada padanan)
  - JS: `height:17px;`
    Rust: (tidak ada padanan)
  - JS: `height:19px;`
    Rust: (tidak ada padanan)
  - JS: `height:26px;`
    Rust: (tidak ada padanan)
  - JS: `height:34px;`
    Rust: (tidak ada padanan)
  - JS: `height:38px;`
    Rust: (tidak ada padanan)
  - JS: `height:4px;`
    Rust: (tidak ada padanan)
  - JS: `height:64px;`
    Rust: (tidak ada padanan)
  - JS: `host = new URL(wsUrl).host;`
    Rust: (tidak ada padanan)
  - JS: `if (!box) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!info?.url) {`
    Rust: (tidak ada padanan)
  - JS: `if (!isFinite(bytes)) return "0 B";`
    Rust: (tidak ada padanan)
  - JS: `if (!isFinite(seconds)) return "0:00";`
    Rust: (tidak ada padanan)
  - JS: `if (!isFinite(video.duration)) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!isLoop) {`
    Rust: (tidak ada padanan)
  - JS: `if (!message) return;`
    Rust: (tidak ada padanan)
  - JS: `if (!query) {`
    Rust: (tidak ada padanan)
  - JS: `if (!video.src) userPressed = true;`
    Rust: (tidak ada padanan)
  - JS: `if (!wsUrl && videoUrl) {`
    Rust: (tidak ada padanan)
  - JS: `if (!wsUrl) {`
    Rust: (tidak ada padanan)
  - JS: `if (bytes >= 1024) return (bytes / 1024).toFixed(2) + " KB";`
    Rust: (tidak ada padanan)
  - JS: `if (bytes >= 1048576) return (bytes / 1048576).toFixed(2) + " MB";`
    Rust: (tidak ada padanan)
  - JS: `if (cfg.autoplay || userPressed) {`
    Rust: (tidak ada padanan)
  - JS: `if (config.volume !== undefined) {`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.style.width = pct.toFixed(1) + "%";`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.textContent = String(wsChunks.length);`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.textContent = fmtSize(wsDownloaded);`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.textContent = fmtSize(wsTotal);`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.textContent = isLoop ? "ON" : "OFF";`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.textContent = pct.toFixed(1) + "%";`
    Rust: (tidak ada padanan)
  - JS: `if (el) el.textContent = state;`
    Rust: (tidak ada padanan)
  - JS: `if (info.thumb) {`
    Rust: (tidak ada padanan)
  - JS: `if (message.contentLength !== undefined && message.contentLength !== null) {`
    Rust: (tidak ada padanan)
  - JS: `if (message.type === "end") {`
    Rust: (tidak ada padanan)
  - JS: `if (message.type === "error") {`
    Rust: (tidak ada padanan)
  - JS: `if (message.type === "start") {`
    Rust: (tidak ada padanan)
  - JS: `if (mimeEl) mimeEl.textContent = wsMime;`
    Rust: (tidak ada padanan)
  - JS: `if (now - wsLastLog > 400) {`
    Rust: (tidak ada padanan)
  - JS: `if (result && result.catch) {`
    Rust: (tidak ada padanan)
  - JS: `if (sec < 10) sec = "0" + sec;`
    Rust: (tidak ada padanan)
  - JS: `if (serverEl) {`
    Rust: (tidak ada padanan)
  - JS: `if (typeof WebSocket === "undefined") {`
    Rust: (tidak ada padanan)
  - JS: `if (typeof event.data === "string") {`
    Rust: (tidak ada padanan)
  - JS: `if (video.paused) {`
    Rust: (tidak ada padanan)
  - JS: `if (video.poster && background) {`
    Rust: (tidak ada padanan)
  - JS: `if (video.volume > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (wsDone) return;`
    Rust: (tidak ada padanan)
  - JS: `if (wsDone) {`
    Rust: (tidak ada padanan)
  - JS: `if (wsObjectUrl) {`
    Rust: (tidak ada padanan)
  - JS: `if (wsTotal !== null && wsTotal > 0) {`
    Rust: (tidak ada padanan)
  - JS: `if (wsVideo) {`
    Rust: (tidak ada padanan)
  - JS: `input[type=range]{`
    Rust: (tidak ada padanan)
  - JS: `inset:-25px;`
    Rust: (tidak ada padanan)
  - JS: `inset:0;`
    Rust: (tidak ada padanan)
  - JS: `isLoop = !isLoop;`
    Rust: (tidak ada padanan)
  - JS: `isLoop = config.loop || false;`
    Rust: (tidak ada padanan)
  - JS: `justify-content:center;`
    Rust: (tidak ada padanan)
  - JS: `justify-content:space-between;`
    Rust: (tidak ada padanan)
  - JS: `let posterBase64 = "";`
    Rust: (tidak ada padanan)
  - JS: `letter-spacing:.5px`
    Rust: (tidak ada padanan)
  - JS: `letter-spacing:.8px;`
    Rust: (tidak ada padanan)
  - JS: `line-height:1.4`
    Rust: (tidak ada padanan)
  - JS: `line-height:1.6;`
    Rust: (tidak ada padanan)
  - JS: `loop: false`
    Rust: (tidak ada padanan)
  - JS: `margin-bottom:15px`
    Rust: (tidak ada padanan)
  - JS: `margin-bottom:8px`
    Rust: (tidak ada padanan)
  - JS: `margin-bottom:8px;`
    Rust: (tidak ada padanan)
  - JS: `margin-top:12px;`
    Rust: (tidak ada padanan)
  - JS: `margin-top:13px`
    Rust: (tidak ada padanan)
  - JS: `margin-top:15px;`
    Rust: (tidak ada padanan)
  - JS: `margin-top:18px`
    Rust: (tidak ada padanan)
  - JS: `margin-top:2px;`
    Rust: (tidak ada padanan)
  - JS: `margin-top:3px;`
    Rust: (tidak ada padanan)
  - JS: `margin-top:5px;`
    Rust: (tidak ada padanan)
  - JS: `margin-top:7px`
    Rust: (tidak ada padanan)
  - JS: `margin:0;`
    Rust: (tidak ada padanan)
  - JS: `margin:auto;`
    Rust: (tidak ada padanan)
  - JS: `max-width:260px`
    Rust: (tidak ada padanan)
  - JS: `max-width:440px;`
    Rust: (tidak ada padanan)
  - JS: `message = JSON.parse(event.data);`
    Rust: (tidak ada padanan)
  - JS: `object-fit:contain`
    Rust: (tidak ada padanan)
  - JS: `opacity:.38;`
    Rust: (tidak ada padanan)
  - JS: `opacity:.55;`
    Rust: (tidak ada padanan)
  - JS: `opacity:.5;`
    Rust: (tidak ada padanan)
  - JS: `opacity:.6;`
    Rust: (tidak ada padanan)
  - JS: `opacity:.8`
    Rust: (tidak ada padanan)
  - JS: `opacity:.85`
    Rust: (tidak ada padanan)
  - JS: `opacity:0;`
    Rust: (tidak ada padanan)
  - JS: `overflow-y:auto;`
    Rust: (tidak ada padanan)
  - JS: `overflow:hidden;`
    Rust: (tidak ada padanan)
  - JS: `padding-top:15px`
    Rust: (tidak ada padanan)
  - JS: `padding:0;`
    Rust: (tidak ada padanan)
  - JS: `padding:12px`
    Rust: (tidak ada padanan)
  - JS: `padding:12px;`
    Rust: (tidak ada padanan)
  - JS: `padding:18px;`
    Rust: (tidak ada padanan)
  - JS: `padding:3px 8px;`
    Rust: (tidak ada padanan)
  - JS: `padding:6px 8px`
    Rust: (tidak ada padanan)
  - JS: `padding:6px 8px;`
    Rust: (tidak ada padanan)
  - JS: `payload: htmlPayload.trim()`
    Rust: (tidak ada padanan)
  - JS: `playVideo();`
    Rust: (tidak ada padanan)
  - JS: `playlistSub.textContent = config.title || "Unknown Video";`
    Rust: (tidak ada padanan)
  - JS: `pointer-events:none`
    Rust: (tidak ada padanan)
  - JS: `position:absolute;`
    Rust: (tidak ada padanan)
  - JS: `position:relative;`
    Rust: (tidak ada padanan)
  - JS: `posterBase64`
    Rust: (tidak ada padanan)
  - JS: `posterBase64 = "data:image/jpeg;base64," + Buffer.from(thumbRes.data).toString("base64");`
    Rust: (tidak ada padanan)
  - JS: `primitive: {`
    Rust: (tidak ada padanan)
  - JS: `progress.value = 0;`
    Rust: (tidak ada padanan)
  - JS: `progress.value = value;`
    Rust: (tidak ada padanan)
  - JS: `repeatButton.style.opacity = isLoop ? "1" : ".55";`
    Rust: (tidak ada padanan)
  - JS: `repeatButton.title = isLoop ? "Loop ON" : "Loop OFF";`
    Rust: (tidak ada padanan)
  - JS: `responseType: "arraybuffer",`
    Rust: (tidak ada padanan)
  - JS: `result.catch(function() {});`
    Rust: (tidak ada padanan)
  - JS: `return `<style>`
    Rust: (tidak ada padanan)
  - JS: `return bytes + " B";`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return min + ":" + sec;`
    Rust: (tidak ada padanan)
  - JS: `return;`
    Rust: (tidak ada padanan)
  - JS: `rgba(5,6,10,.35),`
    Rust: (tidak ada padanan)
  - JS: `rgba(5,6,10,.72)`
    Rust: (tidak ada padanan)
  - JS: `serverEl.textContent = host || "-";`
    Rust: (tidak ada padanan)
  - JS: `setTimeout(function() {`
    Rust: (tidak ada padanan)
  - JS: `setWsState("CLOSED");`
    Rust: (tidak ada padanan)
  - JS: `setWsState("COMPLETE");`
    Rust: (tidak ada padanan)
  - JS: `setWsState("CONNECTED");`
    Rust: (tidak ada padanan)
  - JS: `setWsState("CONNECTING");`
    Rust: (tidak ada padanan)
  - JS: `setWsState("DOWNLOADING");`
    Rust: (tidak ada padanan)
  - JS: `setWsState("ERROR");`
    Rust: (tidak ada padanan)
  - JS: `smallPauseIcon.style.display = "block";`
    Rust: (tidak ada padanan)
  - JS: `smallPauseIcon.style.display = "none";`
    Rust: (tidak ada padanan)
  - JS: `smallPlayIcon.style.display = "block";`
    Rust: (tidak ada padanan)
  - JS: `smallPlayIcon.style.display = "none";`
    Rust: (tidak ada padanan)
  - JS: `songTitle.textContent = config.title || "Unknown Video";`
    Rust: (tidak ada padanan)
  - JS: `startVideoStream(config);`
    Rust: (tidak ada padanan)
  - JS: `stroke-linecap:round;`
    Rust: (tidak ada padanan)
  - JS: `stroke-linejoin:round`
    Rust: (tidak ada padanan)
  - JS: `stroke-width:2;`
    Rust: (tidak ada padanan)
  - JS: `stroke:currentColor;`
    Rust: (tidak ada padanan)
  - JS: `text-overflow:ellipsis`
    Rust: (tidak ada padanan)
  - JS: `text-overflow:ellipsis;`
    Rust: (tidak ada padanan)
  - JS: `throw new Error("Gagal mendapatkan direct link video.");`
    Rust: `Gagal mendapatkan direct link video.`
  - JS: `timeout: 10000`
    Rust: (tidak ada padanan)
  - JS: `title: "␀",`
    Rust: (tidak ada padanan)
  - JS: `title: info.title || yt.title || "YouTube Video",`
    Rust: (tidak ada padanan)
  - JS: `title: titleHeader,`
    Rust: (tidak ada padanan)
  - JS: `touch-action:manipulation;`
    Rust: (tidak ada padanan)
  - JS: `transform:scale(1.15);`
    Rust: (tidak ada padanan)
  - JS: `transition:opacity .2s;`
    Rust: (tidak ada padanan)
  - JS: `transition:width .3s`
    Rust: (tidak ada padanan)
  - JS: `trusted_sources: [],`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `type: wsMime || "video/mp4"`
    Rust: (tidak ada padanan)
  - JS: `updateLoopStatus();`
    Rust: (tidak ada padanan)
  - JS: `updateUI();`
    Rust: (tidak ada padanan)
  - JS: `updateWsProgress();`
    Rust: (tidak ada padanan)
  - JS: `user-select:none;`
    Rust: (tidak ada padanan)
  - JS: `userPressed = false;`
    Rust: (tidak ada padanan)
  - JS: `var artist = document.getElementById("artist");`
    Rust: (tidak ada padanan)
  - JS: `var background = document.getElementById("background");`
    Rust: (tidak ada padanan)
  - JS: `var blob = new Blob(wsChunks, {`
    Rust: (tidak ada padanan)
  - JS: `var box = document.getElementById("debugLog");`
    Rust: (tidak ada padanan)
  - JS: `var bytes = event.data.byteLength || 0;`
    Rust: (tidak ada padanan)
  - JS: `var config = window.VIDEO_CONFIG || {};`
    Rust: (tidak ada padanan)
  - JS: `var currentTime = document.getElementById("currentTime");`
    Rust: (tidak ada padanan)
  - JS: `var div = document.createElement("div");`
    Rust: (tidak ada padanan)
  - JS: `var durationLabel = document.getElementById("duration");`
    Rust: (tidak ada padanan)
  - JS: `var el = document.getElementById("dbgDownloaded");`
    Rust: (tidak ada padanan)
  - JS: `var el = document.getElementById("dbgLoop");`
    Rust: (tidak ada padanan)
  - JS: `var el = document.getElementById("wsState");`
    Rust: (tidak ada padanan)
  - JS: `var frame = document.getElementById("videoFrame");`
    Rust: (tidak ada padanan)
  - JS: `var host = "-";`
    Rust: (tidak ada padanan)
  - JS: `var isLoop = false;`
    Rust: (tidak ada padanan)
  - JS: `var message = null;`
    Rust: (tidak ada padanan)
  - JS: `var mimeEl = document.getElementById("dbgMime");`
    Rust: (tidak ada padanan)
  - JS: `var min = Math.floor(seconds / 60);`
    Rust: (tidak ada padanan)
  - JS: `var now = Date.now();`
    Rust: (tidak ada padanan)
  - JS: `var pct = Math.min(100, (wsDownloaded / wsTotal) * 100);`
    Rust: (tidak ada padanan)
  - JS: `var playlistSub = document.getElementById("playlistSub");`
    Rust: (tidak ada padanan)
  - JS: `var progress = document.getElementById("progress");`
    Rust: (tidak ada padanan)
  - JS: `var repeatButton = document.getElementById("repeatButton");`
    Rust: (tidak ada padanan)
  - JS: `var result = video.play();`
    Rust: (tidak ada padanan)
  - JS: `var sec = Math.floor(seconds % 60);`
    Rust: (tidak ada padanan)
  - JS: `var serverEl = document.getElementById("wsServer");`
    Rust: (tidak ada padanan)
  - JS: `var smallPauseIcon = document.getElementById("smallPauseIcon");`
    Rust: (tidak ada padanan)
  - JS: `var smallPlayIcon = document.getElementById("smallPlayIcon");`
    Rust: (tidak ada padanan)
  - JS: `var songTitle = document.getElementById("songTitle");`
    Rust: (tidak ada padanan)
  - JS: `var userPressed = false;`
    Rust: (tidak ada padanan)
  - JS: `var value = (video.currentTime / video.duration) * 100;`
    Rust: (tidak ada padanan)
  - JS: `var video = document.getElementById("videoPlayer");`
    Rust: (tidak ada padanan)
  - JS: `var videoUrl = cfg.videoUrl || "";`
    Rust: (tidak ada padanan)
  - JS: `var volume = document.getElementById("volume");`
    Rust: (tidak ada padanan)
  - JS: `var wsChunks = [];`
    Rust: (tidak ada padanan)
  - JS: `var wsDone = false;`
    Rust: (tidak ada padanan)
  - JS: `var wsDownloaded = 0;`
    Rust: (tidak ada padanan)
  - JS: `var wsLastLog = 0;`
    Rust: (tidak ada padanan)
  - JS: `var wsMime = "video/mp4";`
    Rust: (tidak ada padanan)
  - JS: `var wsObjectUrl = null;`
    Rust: (tidak ada padanan)
  - JS: `var wsTotal = null;`
    Rust: (tidak ada padanan)
  - JS: `var wsUrl = cfg.wsUrl || "";`
    Rust: (tidak ada padanan)
  - JS: `var wsVideo = null;`
    Rust: (tidak ada padanan)
  - JS: `video.addEventListener("ended", function() {`
    Rust: (tidak ada padanan)
  - JS: `video.addEventListener("loadedmetadata", function() {`
    Rust: (tidak ada padanan)
  - JS: `video.addEventListener("pause", function() {`
    Rust: (tidak ada padanan)
  - JS: `video.addEventListener("play", function() {`
    Rust: (tidak ada padanan)
  - JS: `video.addEventListener("timeupdate", function() {`
    Rust: (tidak ada padanan)
  - JS: `video.currentTime = (Number(value) / 100) * video.duration;`
    Rust: (tidak ada padanan)
  - JS: `video.currentTime = 0;`
    Rust: (tidak ada padanan)
  - JS: `video.currentTime = Math.max(0, video.currentTime - 10);`
    Rust: (tidak ada padanan)
  - JS: `video.currentTime = Math.min(video.duration, video.currentTime + 10);`
    Rust: (tidak ada padanan)
  - JS: `video.load();`
    Rust: (tidak ada padanan)
  - JS: `video.loop = isLoop;`
    Rust: (tidak ada padanan)
  - JS: `video.muted = !video.muted;`
    Rust: (tidak ada padanan)
  - JS: `video.muted = false;`
    Rust: (tidak ada padanan)
  - JS: `video.pause();`
    Rust: (tidak ada padanan)
  - JS: `video.src = wsObjectUrl;`
    Rust: (tidak ada padanan)
  - JS: `video.volume = Number(value);`
    Rust: (tidak ada padanan)
  - JS: `video.volume = config.volume;`
    Rust: (tidak ada padanan)
  - JS: `videoUrl: "␀",`
    Rust: (tidak ada padanan)
  - JS: `videoUrl: info.url,`
    Rust: (tidak ada padanan)
  - JS: `view_model: {`
    Rust: (tidak ada padanan)
  - JS: `volume.value = config.volume;`
    Rust: (tidak ada padanan)
  - JS: `volume: 0.8,`
    Rust: (tidak ada padanan)
  - JS: `while (box.childNodes.length > 80) {`
    Rust: (tidak ada padanan)
  - JS: `white-space:nowrap;`
    Rust: (tidak ada padanan)
  - JS: `white-space:pre-wrap`
    Rust: (tidak ada padanan)
  - JS: `width:0;`
    Rust: (tidak ada padanan)
  - JS: `width:100%;`
    Rust: (tidak ada padanan)
  - JS: `width:17px;`
    Rust: (tidak ada padanan)
  - JS: `width:19px;`
    Rust: (tidak ada padanan)
  - JS: `width:26px;`
    Rust: (tidak ada padanan)
  - JS: `width:34px;`
    Rust: (tidak ada padanan)
  - JS: `width:38px;`
    Rust: (tidak ada padanan)
  - JS: `width:64px;`
    Rust: (tidak ada padanan)
  - JS: `width:85px`
    Rust: (tidak ada padanan)
  - JS: `window.VIDEO_CONFIG = {`
    Rust: (tidak ada padanan)
  - JS: `window.addEventListener("beforeunload", function() {`
    Rust: (tidak ada padanan)
  - JS: `word-break:break-all;`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Binary chunk: " + fmtSize(bytes));`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Blob created (" + fmtSize(blob.size) + ")");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Code: " + event.code + " Reason: " + (event.reason || "-"));`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Connecting...");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Download complete");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("END");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("No video URL tersedia");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Player ready");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("START | MIME: " + wsMime);`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Server error: " + (message.message || "unknown"));`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Size: " + fmtSize(wsTotal));`
    Rust: (tidak ada padanan)
  - JS: `writeLog("Video source attached");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("WebSocket API tidak tersedia");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("WebSocket OPEN");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("WebSocket closed unexpectedly");`
    Rust: (tidak ada padanan)
  - JS: `writeLog("WebSocket error");`
    Rust: (tidak ada padanan)
  - JS: `wsChunks = [];`
    Rust: (tidak ada padanan)
  - JS: `wsChunks.push(event.data);`
    Rust: (tidak ada padanan)
  - JS: `wsDone = true;`
    Rust: (tidak ada padanan)
  - JS: `wsDownloaded += bytes;`
    Rust: (tidak ada padanan)
  - JS: `wsLastLog = now;`
    Rust: (tidak ada padanan)
  - JS: `wsMime = message.mime || "video/mp4";`
    Rust: (tidak ada padanan)
  - JS: `wsObjectUrl = URL.createObjectURL(blob);`
    Rust: (tidak ada padanan)
  - JS: `wsTotal = Number(message.contentLength);`
    Rust: (tidak ada padanan)
  - JS: `wsTotal = Number(message.length);`
    Rust: (tidak ada padanan)
  - JS: `wsUrl = "wss://stream.kanara.my.id/?url=" + encodeURIComponent(videoUrl);`
    Rust: (tidak ada padanan)
  - JS: `wsUrl,`
    Rust: (tidak ada padanan)
  - JS: `wsUrl: "␀",`
    Rust: (tidak ada padanan)
  - JS: `wsVideo = new WebSocket(wsUrl);`
    Rust: (tidak ada padanan)
  - JS: `wsVideo.binaryType = "arraybuffer";`
    Rust: (tidak ada padanan)
  - JS: `wsVideo.close();`
    Rust: (tidak ada padanan)
  - JS: `wsVideo.onclose = function(event) {`
    Rust: (tidak ada padanan)
  - JS: `wsVideo.onerror = function() {`
    Rust: (tidak ada padanan)
  - JS: `wsVideo.onmessage = function(event) {`
    Rust: (tidak ada padanan)
  - JS: `wsVideo.onopen = function() {`
    Rust: (tidak ada padanan)
  - JS: `yt-search`
    Rust: (tidak ada padanan)
  - JS: `z-index:2`
    Rust: (tidak ada padanan)
  - JS: `z-index:3`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {}`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} catch {}`
    Rust: (tidak ada padanan)
  - JS: `} else if (message.length !== undefined && message.length !== null) {`
    Rust: (tidak ada padanan)
  - JS: `} else {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `}, 300);`
    Rust: (tidak ada padanan)
  - JS: `};`
    Rust: (tidak ada padanan)
  - JS: `␀`);`
    Rust: (tidak ada padanan)

### yts (search)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/search/yts.js`
- Rust: `/workspaces/oktz-rs/plugin/src/search/yts.rs`

  - JS: `yt-search`
    Rust: (tidak ada padanan)

### countrystalk (stalker  )
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/countrystalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/countrystalk.rs`

  - JS: `[Country Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)

### ffstalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/ffstalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/ffstalk.rs`

  - JS: `[FFStalk]`
    Rust: (tidak ada padanan)

### genshinstalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/genshinstalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/genshinstalk.rs`

  - JS: `Halo Traveler! Ini dia informasi akun untuk UID *␀*:`
    Rust: `Halo Traveler! Ini dia informasi akun untuk UID *␀*:\n\n`
  - JS: `[Genshin Stalk]`
    Rust: (tidak ada padanan)

### igstalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/igstalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/igstalk.rs`

  - JS: `␀igstalk`
    Rust: (tidak ada padanan)

### pintereststalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/pintereststalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/pintereststalk.rs`

  - JS: `[Pinterest Stalk]`
    Rust: (tidak ada padanan)

### robloxplayer (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/robloxplayer.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/robloxplayer.rs`

  - JS: `[RobloxPlayer] Error:`
    Rust: (tidak ada padanan)

### robloxstalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/robloxstalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/robloxstalk.rs`

  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `User tidak ditemukan`
    Rust: `❌ Username *␀* tidak ditemukan`
  - JS: `🎮 *Games* (␀):`
    Rust: `␀🎮 *Games* (␀):`
  - JS: `🏆 *Badges* (␀):`
    Rust: `␀🏆 *Badges* (␀):`
  - JS: `📅 *Created*: ␀) : "-"}`
    Rust: `📅 *Created*: ␀`
  - JS: `🔗 https://roblox.com/users/␀/profile`
    Rust: `␀🔗 https://roblox.com/users/␀/profile`

### stalkdc (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/stalkdc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/discordstalk.rs`

  - JS: `"Android"`
    Rust: (tidak ada padanan)
  - JS: `"Chromium";v="127", "Not)A;Brand";v="99", "Microsoft Edge Simulate";v="127", "Lemur";v="127"`
    Rust: (tidak ada padanan)
  - JS: `&& typeof arr[1] ===`
    Rust: (tidak ada padanan)
  - JS: `) return arr[1];`
    Rust: (tidak ada padanan)
  - JS: `) {`
    Rust: (tidak ada padanan)
  - JS: `) ||`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `, retries = 2 } = {}) {`
    Rust: (tidak ada padanan)
  - JS: `...(action ? { sa: action } : {}),`
    Rust: (tidak ada padanan)
  - JS: `.stalkdc 1280009524941426700`
    Rust: (tidak ada padanan)
  - JS: `.stalkdc <id_discord>`
    Rust: (tidak ada padanan)
  - JS: `443`
    Rust: (tidak ada padanan)
  - JS: `6Lc0ZystAAAAAKESQ1HMsCQujyLgDvSu6FW1CHfP`
    Rust: (tidak ada padanan)
  - JS: `80`
    Rust: (tidak ada padanan)
  - JS: `: UA,`
    Rust: (tidak ada padanan)
  - JS: `: recaptchaToken,`
    Rust: (tidak ada padanan)
  - JS: `;`
    Rust: (tidak ada padanan)
  - JS: `?0`
    Rust: (tidak ada padanan)
  - JS: `Maaf, sistem sedang mengalami gangguan saat melakukan *bypass reCAPTCHA* atau memanggil API. Silakan coba beberapa saat lagi.`
    Rust: (tidak ada padanan)
  - JS: `Mengetahui info profil akun Discord berdasarkan ID`
    Rust: (tidak ada padanan)
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `Pastikan ID Discord yang kamu masukkan hanya berupa angka.`);`
    Rust: (tidak ada padanan)
  - JS: `Sistem gagal menemukan akun dengan ID *␀*. Mungkin ID tersebut salah atau akun telah dihapus.`
    Rust: (tidak ada padanan)
  - JS: `_Error: ␀_`);`
    Rust: (tidak ada padanan)
  - JS: `_Pesan error: ${resp.message ||`
    Rust: (tidak ada padanan)
  - JS: `_Powered by dclookup.id_`;`
    Rust: (tidak ada padanan)
  - JS: `` +`
    Rust: (tidak ada padanan)
  - JS: ``*CARA PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: ``- Contoh: `␀stalkdc 1280009524941426700``
    Rust: (tidak ada padanan)
  - JS: ``- Ketik `␀stalkdc <ID Discord>``
    Rust: (tidak ada padanan)
  - JS: ``;`
    Rust: (tidak ada padanan)
  - JS: ``Fitur ini membantumu untuk melacak profil pengguna Discord hanya dari ID-nya saja!`
    Rust: (tidak ada padanan)
  - JS: ``_Pastikan kamu memasukkan ID berbentuk angka (bukan username ya!)._``
    Rust: (tidak ada padanan)
  - JS: ``␀/api/user/lookup`,`
    Rust: (tidak ada padanan)
  - JS: ``👾 *DISCORD STALKER* 👾`
    Rust: (tidak ada padanan)
  - JS: `accept-language`
    Rust: (tidak ada padanan)
  - JS: `accept:`
    Rust: (tidak ada padanan)
  - JS: `ar:`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { text, sock }) {`
    Rust: (tidak ada padanan)
  - JS: `async function lookupUser(discordId, recaptchaToken) {`
    Rust: (tidak ada padanan)
  - JS: `async function solveRecaptchaV3({ sitekey, url, action, hl =`
    Rust: (tidak ada padanan)
  - JS: `await m.react(`
    Rust: (tidak ada padanan)
  - JS: `await m.reply(caption);`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMessage(m.chat, { image: { url: avaUrl }, caption: caption }, { quoted: m });`
    Rust: (tidak ada padanan)
  - JS: `c: cMatch[1],`
    Rust: (tidak ada padanan)
  - JS: `caption += ``
    Rust: (tidak ada padanan)
  - JS: `caption += `Pencarian berhasil! Berikut adalah informasi profil dari Discord tersebut:`
    Rust: (tidak ada padanan)
  - JS: `caption += `🆔 *User ID:* ␀`
    Rust: (tidak ada padanan)
  - JS: `caption += `🏅 *Badges:* ␀`
    Rust: (tidak ada padanan)
  - JS: `caption += `🏷️ *Global Name:* ${d.global_name ||`
    Rust: (tidak ada padanan)
  - JS: `caption += `👤 *Username:* ␀`
    Rust: (tidak ada padanan)
  - JS: `caption += `🗓️ *Dibuat Sejak:* ${d.creation_date ||`
    Rust: (tidak ada padanan)
  - JS: `caption += `🛡️ *Clan:* ␀ (␀)`
    Rust: (tidak ada padanan)
  - JS: `caption += `🤖 *Apakah Bot?:* ␀`
    Rust: (tidak ada padanan)
  - JS: `cb: randomCb(),`
    Rust: (tidak ada padanan)
  - JS: `co: buildCo(url),`
    Rust: (tidak ada padanan)
  - JS: `co: params.co,`
    Rust: (tidak ada padanan)
  - JS: `console.error("DC Lookup Error:", e);`
    Rust: (tidak ada padanan)
  - JS: `const anchorQs = new URLSearchParams(params).toString();`
    Rust: (tidak ada padanan)
  - JS: `const anchorRes = await gc.get(`␀anchor?␀`, {`
    Rust: (tidak ada padanan)
  - JS: `const arr = JSON.parse(cleaned);`
    Rust: (tidak ada padanan)
  - JS: `const avaUrl = d.avatar_url || d.default_avatar_url || d.avatar;`
    Rust: (tidak ada padanan)
  - JS: `const badges = (d.badges || []).map((b) => b.name).join(`
    Rust: (tidak ada padanan)
  - JS: `const base = `␀api2/`;`
    Rust: (tidak ada padanan)
  - JS: `const cMatch = String(anchorRes.data).match(/id="recaptcha-token"s+value="([^"]+)"/);`
    Rust: (tidak ada padanan)
  - JS: `const d = resp.data || {};`
    Rust: (tidak ada padanan)
  - JS: `const discordId = text.replace(/[^0-9]/g,`
    Rust: (tidak ada padanan)
  - JS: `const isBot = d.is_bot ?`
    Rust: (tidak ada padanan)
  - JS: `const m = cleaned.match(/"rresp","([^"]+)"/);`
    Rust: (tidak ada padanan)
  - JS: `const params = {`
    Rust: (tidak ada padanan)
  - JS: `const postData = new URLSearchParams({`
    Rust: (tidak ada padanan)
  - JS: `const reloadRes = await gc.post(`␀reload?k=␀`, postData, {`
    Rust: (tidak ada padanan)
  - JS: `const res = await axios.post(`
    Rust: (tidak ada padanan)
  - JS: `const resp = await lookupUser(discordId, token);`
    Rust: (tidak ada padanan)
  - JS: `const token = extractRresp(reloadRes.data);`
    Rust: (tidak ada padanan)
  - JS: `const { token } = await solveRecaptchaV3({ sitekey: SITEKEY, url: BASE, action: ACTION });`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `for (let attempt = 1; attempt <= retries + 1; attempt++) {`
    Rust: (tidak ada padanan)
  - JS: `gagal grab versi collector (v) dari api.js`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `headers: { referer: new URL(url).origin +`
    Rust: (tidak ada padanan)
  - JS: `hl,`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `if (!cMatch) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!discordId) {`
    Rust: (tidak ada padanan)
  - JS: `if (!resp.success) {`
    Rust: (tidak ada padanan)
  - JS: `if (!sitekey) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (!text) {`
    Rust: (tidak ada padanan)
  - JS: `if (!token) throw new Error(`
    Rust: (tidak ada padanan)
  - JS: `if (Array.isArray(arr) && arr[0] ===`
    Rust: (tidak ada padanan)
  - JS: `if (anchorRes.status !== 200) throw new Error(`anchor HTTP ␀`);`
    Rust: (tidak ada padanan)
  - JS: `if (attempt <= retries) {`
    Rust: (tidak ada padanan)
  - JS: `if (d.avatar_url || d.default_avatar_url || d.avatar) {`
    Rust: (tidak ada padanan)
  - JS: `if (d.clan) {`
    Rust: (tidak ada padanan)
  - JS: `if (reloadRes.status !== 200) throw new Error(`reload HTTP ␀`);`
    Rust: (tidak ada padanan)
  - JS: `if (typeof res.data !==`
    Rust: (tidak ada padanan)
  - JS: `k: params.k,`
    Rust: (tidak ada padanan)
  - JS: `k: sitekey,`
    Rust: (tidak ada padanan)
  - JS: `lastErr = err;`
    Rust: (tidak ada padanan)
  - JS: `let caption = `👾 *DISCORD STALKER* 👾`
    Rust: (tidak ada padanan)
  - JS: `let lastErr;`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`❌ *GAGAL MELACAK AKUN*`
    Rust: (tidak ada padanan)
  - JS: `new URLSearchParams({ id: discordId }).toString(),`
    Rust: (tidak ada padanan)
  - JS: `origin:`
    Rust: (tidak ada padanan)
  - JS: `origin: BASE,`
    Rust: (tidak ada padanan)
  - JS: `params.cb = randomCb();`
    Rust: (tidak ada padanan)
  - JS: `reason:`
    Rust: (tidak ada padanan)
  - JS: `referer: `␀/`,`
    Rust: (tidak ada padanan)
  - JS: `referer: `␀anchor?␀`,`
    Rust: (tidak ada padanan)
  - JS: `return m ? m[1] : null;`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`❌ *AKUN TIDAK DITEMUKAN*`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`❌ *ID TIDAK VALID*`
    Rust: (tidak ada padanan)
  - JS: `return res.data;`
    Rust: (tidak ada padanan)
  - JS: `return { token, version: params.v };`
    Rust: (tidak ada padanan)
  - JS: `s*/,`
    Rust: (tidak ada padanan)
  - JS: `sec-ch-ua`
    Rust: (tidak ada padanan)
  - JS: `sec-ch-ua-mobile`
    Rust: (tidak ada padanan)
  - JS: `sec-ch-ua-platform`
    Rust: (tidak ada padanan)
  - JS: `size:`
    Rust: (tidak ada padanan)
  - JS: `throw lastErr;`
    Rust: (tidak ada padanan)
  - JS: `throw new Error(`Response bukan JSON (HTTP ␀)`);`
    Rust: (tidak ada padanan)
  - JS: `timeout: 20000,`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)
  - JS: `v: await fetchLatestVersion(),`
    Rust: (tidak ada padanan)
  - JS: `v: params.v,`
    Rust: (tidak ada padanan)
  - JS: `validateStatus: () => true,`
    Rust: (tidak ada padanan)
  - JS: `{`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (e) {}`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} else {`
    Rust: (tidak ada padanan)
  - JS: `}).toString();`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `},`
    Rust: (tidak ada padanan)
  - JS: `};`
    Rust: (tidak ada padanan)
  - JS: `}_`);`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: (tidak ada padanan)
  - JS: `␀api.js`
    Rust: (tidak ada padanan)

### stalkml (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/stalkml.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/stalkml.rs`

  - JS: `User-Agent`
    Rust: (tidak ada padanan)

### tiktokstalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/tiktokstalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/tiktokstalk.rs`

  - JS: `.0`
    Rust: (tidak ada padanan)

### ytstalk (stalker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/stalker/ytstalk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/stalker/ytstalk.rs`

  - JS: `␀ytstalk`
    Rust: (tidak ada padanan)

### attp (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/attp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/attp.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `API gagal, fallback ke local`
    Rust: (tidak ada padanan)
  - JS: `Exif error:`
    Rust: (tidak ada padanan)
  - JS: `bold ␀px sans-serif`
    Rust: (tidak ada padanan)

### attp2 (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/attp2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/attp2.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `bold ␀px sans-serif`
    Rust: (tidak ada padanan)

### brat (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/brat.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/brat.rs`

  - JS: `API returned blank image`
    Rust: `API returned blank`

### bratextra (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/bratextra.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/bratextra.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `Template tidak ditemukan`
    Rust: `Template tidak`
  - JS: `bold ␀px sans-serif`
    Rust: (tidak ada padanan)

### bratvid (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/bratvid.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/bratvid.rs`

  - JS: `brat-␀.webp`
    Rust: (tidak ada padanan)
  - JS: `spawn("ffmpeg")`
    Rust: (tidak ada padanan)

### bratvid2 (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/bratvid2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/bratvid2.rs`

  - JS: `+faststart`
    Rust: (tidak ada padanan)
  - JS: `-FE0F`
    Rust: (tidak ada padanan)
  - JS: `-c:v`
    Rust: (tidak ada padanan)
  - JS: `-crf`
    Rust: (tidak ada padanan)
  - JS: `-f`
    Rust: (tidak ada padanan)
  - JS: `-i`
    Rust: (tidak ada padanan)
  - JS: `-movflags`
    Rust: (tidak ada padanan)
  - JS: `-pix_fmt`
    Rust: (tidak ada padanan)
  - JS: `-preset`
    Rust: (tidak ada padanan)
  - JS: `-r`
    Rust: (tidak ada padanan)
  - JS: `-s`
    Rust: (tidak ada padanan)
  - JS: `-vf scale=512:512`
    Rust: (tidak ada padanan)
  - JS: `-y`
    Rust: (tidak ada padanan)
  - JS: `23`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `Download gagal (␀)`
    Rust: (tidak ada padanan)
  - JS: `FFmpeg exit code ␀: ␀`
    Rust: (tidak ada padanan)
  - JS: `Halo Guys`
    Rust: (tidak ada padanan)
  - JS: `Teks kosong`
    Rust: (tidak ada padanan)
  - JS: `[BratVid2 Error]`
    Rust: (tidak ada padanan)
  - JS: ``Contoh: ␀bratvid2 hello world``
    Rust: ``Contoh: .bratvid2 hello world``
  - JS: `blur(␀px)`
    Rust: (tidak ada padanan)
  - JS: `brat-`
    Rust: (tidak ada padanan)
  - JS: `brat2-␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `impact.ttf`
    Rust: (tidak ada padanan)
  - JS: `output.mp4`
    Rust: (tidak ada padanan)
  - JS: `pipe:0`
    Rust: (tidak ada padanan)
  - JS: `rgba(255, 255, 255, 0)`
    Rust: (tidak ada padanan)
  - JS: `rgba(255, 255, 255, 0.05)`
    Rust: (tidak ada padanan)
  - JS: `rgba(255, 255, 255, 0.35)`
    Rust: (tidak ada padanan)
  - JS: `rgba(255, 255, 255, 0.95)`
    Rust: (tidak ada padanan)
  - JS: `␀-fe0f`
    Rust: (tidak ada padanan)
  - JS: `␀px Impact`
    Rust: (tidak ada padanan)
  - JS: `␀x␀`
    Rust: (tidak ada padanan)

### emojimix (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/emojimix.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/emojimix.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `80px sans-serif`
    Rust: (tidak ada padanan)

### linesticker (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/linesticker.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/linesticker.rs`

  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[LineSticker] Error:`
    Rust: (tidak ada padanan)
  - JS: `[LineSticker] Sticker error:`
    Rust: (tidak ada padanan)

### pinpack (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/pinpack.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/pinpack.rs`

  - JS: `Sticker pack dari Pinterest: ␀`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[PinPack] Error:`
    Rust: (tidak ada padanan)
  - JS: `[PinPack] Pack send failed:`
    Rust: (tidak ada padanan)
  - JS: `── .✦ 𝗣𝗜𝗡 𝗣𝗔𝗖𝗞 ✦. ── 𝜗ৎ`
    Rust: `── .✦ 𝗣𝗜𝗇 𝗣𝗔𝗖𝗞 ✦. ── 𝜗ৎ`

### qc (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/qc.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/qc.rs`

  - JS: `14px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `Foto-AI`
    Rust: (tidak ada padanan)
  - JS: `bold ␀px sans-serif`
    Rust: (tidak ada padanan)
  - JS: `rgba(255,255,255,0.6)`
    Rust: (tidak ada padanan)
  - JS: `~ ␀`
    Rust: (tidak ada padanan)

### smeme (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/smeme.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/smeme.rs`

  - JS: `[SMEME] Error:`
    Rust: (tidak ada padanan)
  - JS: `[SMEME] Image uploaded:`
    Rust: (tidak ada padanan)
  - JS: `[SMEME] Sharp resize failed:`
    Rust: (tidak ada padanan)
  - JS: `[SMEME] Telegraph failed:`
    Rust: (tidak ada padanan)
  - JS: ``Contoh: ␀smeme Ketika|Kamu Lupa``
    Rust: ``Contoh: .smeme Ketika|Kamu Lupa``
  - JS: ``Contoh: ␀smeme Top|Bottom``
    Rust: ``Contoh: .smeme Top|Bottom``
  - JS: `form-data`
    Rust: (tidak ada padanan)

### smemevid (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/smemevid.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/smemevid.rs`

  - JS: `-c:v libx264`
    Rust: (tidak ada padanan)
  - JS: `-crf 26`
    Rust: `-crf`
  - JS: `-map [out]`
    Rust: (tidak ada padanan)
  - JS: `-preset fast`
    Rust: `-preset`
  - JS: `-t 4`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `Foto-AI`
    Rust: (tidak ada padanan)
  - JS: `Stream video tidak ditemukan`
    Rust: (tidak ada padanan)
  - JS: `[0:v]crop='min(iw,ih)':'min(iw,ih)',scale=␀:␀,fps=8[vid]`
    Rust: `crop='min(iw,ih)':'min(iw,ih)',scale=512:512,fps=8`
  - JS: `[vid][1:v]overlay=0:0[out]`
    Rust: (tidak ada padanan)
  - JS: ``Contoh: ␀smemevid Top|Bottom``
    Rust: ``Contoh: .smemevid Top|Bottom``
  - JS: ``Contoh: ␀smemevid WIDTH OR HEIGHT|WHY NOT BOTH?``
    Rust: ``Contoh: .smemevid WIDTH OR HEIGHT|WHY NOT BOTH?``
  - JS: `bold ␀px Impact, Arial`
    Rust: (tidak ada padanan)
  - JS: `fluent-ffmpeg`
    Rust: (tidak ada padanan)
  - JS: `overlay-␀.png`
    Rust: (tidak ada padanan)
  - JS: `vid-out-␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `vid-␀.mp4`
    Rust: (tidak ada padanan)

### sticker (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/sticker.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/sticker.rs`

  - JS: `"␀" -i "␀" -vf "␀" -c:a copy -y "␀"`
    Rust: (tidak ada padanan)
  - JS: `"␀" -i "␀" -vf "␀" -y "␀"`
    Rust: (tidak ada padanan)
  - JS: `"␀" -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "␀"`
    Rust: (tidak ada padanan)
  - JS: `*ᴄᴏɴᴛᴏʜ:*`
    Rust: `*ᴄᴏɴᴛᴏʛ:*`
  - JS: `Sticker album selesai (␀). ␀ media lainnya dilewati (maks 10).`
    Rust: (tidak ada padanan)
  - JS: `[Sticker] Album failed:`
    Rust: (tidak ada padanan)
  - JS: `[Sticker] Album item skipped:`
    Rust: (tidak ada padanan)
  - JS: `[Sticker] FFmpeg processing failed:`
    Rust: (tidak ada padanan)
  - JS: `duration_check_␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `format=rgba`
    Rust: (tidak ada padanan)
  - JS: `geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(gt(pow(X-W/2,2)+pow(Y-H/2,2),pow(min(W,H)/2,2)),0,255)'`
    Rust: `format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(gt(pow(X-W/2,2)+pow(Y-H/2,2),pow(min(W,H)/2,2)),0,255)'`
  - JS: `geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(lt(X,␀)*lt(Y,␀)*gt(pow(␀-X,2)+pow(␀-Y,2),pow(␀,2)),0,if(gt(X,W-␀)*lt(Y,␀)*gt(pow(X-W+␀,2)+pow(␀-Y,2),pow(␀,2)),0,if(lt(X,␀)*gt(Y,H-␀)*gt(pow(␀-X,2)+pow(Y-H+␀,2),pow(␀,2)),0,if(gt(X,W-␀)*gt(Y,H-␀)*gt(pow(X-W+␀,2)+pow(Y-H+␀,2),pow(␀,2)),0,255))))'`
    Rust: (tidak ada padanan)
  - JS: `sticker_in_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `sticker_out_␀.␀`
    Rust: (tidak ada padanan)

### stickerly (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/stickerly.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/stickerly.rs`

  - JS: `> 💡 *Kirim angka (contoh: 1)* untuk mendownload pack, atau ketik `batal` untuk membatalkan pencarian.`
    Rust: `> 💡 *Kirim angka (contoh: 1)* untuk mendownload pack, atau ketik `batal``
  - JS: `Sesi pencarian stickerly sudah berakhir karena lebih dari 5 menit. Silakan ulangi perintah.`
    Rust: (tidak ada padanan)
  - JS: `Sticker pack dari Sticker.ly`
    Rust: `Cari dan download sticker pack dari Sticker.ly`
  - JS: `[Stickerly Pack Fetch Error]`
    Rust: (tidak ada padanan)
  - JS: `[Stickerly Pack Send Error]`
    Rust: (tidak ada padanan)
  - JS: `[Stickerly Search Error]`
    Rust: (tidak ada padanan)

### stickerpack (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/stickerpack.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/stickerpack.rs`

  - JS: `Query kosong`
    Rust: (tidak ada padanan)
  - JS: `Sticker pack: ␀`
    Rust: `Sticker.ly Pack`
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `[StickerPack] Error:`
    Rust: (tidak ada padanan)
  - JS: `[StickerPack] Pack send failed:`
    Rust: (tidak ada padanan)

### swm (sticker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/swm.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/swm.rs`

  - JS: `000000`
    Rust: (tidak ada padanan)
  - JS: `[SWM] Error:`
    Rust: (tidak ada padanan)

### ttp (maker)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/sticker/ttp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/sticker/ttp.rs`

  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `bold ␀px sans-serif`
    Rust: (tidak ada padanan)

### addlist (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/addlist.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/addlist.rs`

  - JS: `> Produk tetap disimpan TANPA thumbnail.`
    Rust: (tidak ada padanan)
  - JS: `L␀`
    Rust: (tidak ada padanan)
  - JS: `[AddList] Upload error:`
    Rust: (tidak ada padanan)
  - JS: `image.jpg`
    Rust: (tidak ada padanan)
  - JS: `video.mp4`
    Rust: (tidak ada padanan)
  - JS: `• Kirim gambar/video terlebih dahulu, lalu reply media tersebut dengan command di atas untuk menambahkan media 📸`
    Rust: (tidak ada padanan)
  - JS: `⏳ _Mengunggah media..._`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *Gagal upload media: ␀*`
    Rust: (tidak ada padanan)
  - JS: `🎬 Media: ✅ Video`
    Rust: (tidak ada padanan)
  - JS: `🖼️ Media: ✅ Gambar`
    Rust: (tidak ada padanan)

### addstok (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/addstok.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/addstok.rs`

  - JS: `.txt`
    Rust: (tidak ada padanan)
  - JS: `Pastikan file tidak kosong dan dapat diunduh 📄`
    Rust: (tidak ada padanan)
  - JS: `_Lihat daftar stok: `␀liststok ␀`_`
    Rust: `Lihat daftar produk: `␀liststok` 📋`
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `⏭️ Duplikat dilewati: *␀*`
    Rust: (tidak ada padanan)
  - JS: `⏳ _Memproses file..._`
    Rust: (tidak ada padanan)
  - JS: `✅ *IMPORT STOK SELESAI*`
    Rust: (tidak ada padanan)
  - JS: `❌ *File kosong.* 📄`
    Rust: (tidak ada padanan)
  - JS: `❌ *File tidak berisi data valid.* 📄`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal membaca file.*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Terlalu banyak item.* Maksimal 1.000 per import 📄`
    Rust: (tidak ada padanan)
  - JS: `➕ Ditambahkan: *␀* akun 🔑`
    Rust: `➕ Ditambahkan: *␀ pcs*`
  - JS: `📊 Total stok: *␀* akun`
    Rust: `🔑 Total stok saat ini: *␀* akun`

### beli (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/beli.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/beli.rs`

  - JS: `.beli 1`
    Rust: (tidak ada padanan)
  - JS: `.beli <nomor_produk>`
    Rust: (tidak ada padanan)
  - JS: `1️⃣ Transfer sebesar *␀* ke nomor admin 💰`
    Rust: (tidak ada padanan)
  - JS: `2️⃣ Setelah transfer, kirim *bukti pembayaran* ke admin 📸`
    Rust: (tidak ada padanan)
  - JS: `3️⃣ Admin akan memverifikasi dan mengirim data produk ke Anda ✅`
    Rust: (tidak ada padanan)
  - JS: `Ketik `␀beli <nomor>` untuk memesan.`
    Rust: (tidak ada padanan)
  - JS: `Ketik `␀listproduk` untuk melihat daftar produk 🛍️`
    Rust: (tidak ada padanan)
  - JS: `Silakan hubungi admin atau cek kembali nanti.`
    Rust: (tidak ada padanan)
  - JS: `TRX-␀`
    Rust: (tidak ada padanan)
  - JS: `_Kami akan segera mengisi ulang stok_ 🙏`
    Rust: (tidak ada padanan)
  - JS: `_Setelah menerima bukti transfer 📸, reply pesan pembeli lalu ketik `␀done ␀`_ ✅`
    Rust: (tidak ada padanan)
  - JS: `_Simpan nomor ini untuk referensi_ 📌`
    Rust: (tidak ada padanan)
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `␀ *␀.* ␀ — ␀ ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ Produk *␀* saat ini sedang tidak tersedia 😔`
    Rust: (tidak ada padanan)
  - JS: `␀ Produk: *␀*`
    Rust: (tidak ada padanan)
  - JS: `␀@s.whatsapp.net`
    Rust: (tidak ada padanan)
  - JS: `❌ *Stok Habis*`
    Rust: (tidak ada padanan)
  - JS: `🏦 ␀: `␀` a.n ␀`
    Rust: (tidak ada padanan)
  - JS: `🏷️ Tipe: *␀*`
    Rust: (tidak ada padanan)
  - JS: `🏷️ ~~␀~~`
    Rust: (tidak ada padanan)
  - JS: `👤 Pembeli: *␀*`
    Rust: (tidak ada padanan)
  - JS: `💰 Harga: *␀*`
    Rust: (tidak ada padanan)
  - JS: `💳 *INSTRUKSI PEMBAYARAN*`
    Rust: (tidak ada padanan)
  - JS: `📝 _␀_`
    Rust: (tidak ada padanan)
  - JS: `📞 Hubungi admin: wa.me/␀`
    Rust: (tidak ada padanan)
  - JS: `📦 *Detail Pesanan:*`
    Rust: (tidak ada padanan)
  - JS: `📭 *Belum ada produk tersedia.*`
    Rust: (tidak ada padanan)
  - JS: `📱 Nomor: `␀``
    Rust: (tidak ada padanan)
  - JS: `📱 QRIS: Tersedia`
    Rust: (tidak ada padanan)
  - JS: `🛒 *PESANAN BARU*`
    Rust: (tidak ada padanan)
  - JS: `🛒 *PESANAN DIBUAT*`
    Rust: (tidak ada padanan)
  - JS: `🛒 *Pilih Produk*`
    Rust: (tidak ada padanan)
  - JS: `🛒 Pesan produk dan dapatkan nomor transaksi`
    Rust: (tidak ada padanan)
  - JS: `🧾 Nomor Transaksi Anda: `␀``
    Rust: (tidak ada padanan)
  - JS: `🧾 Nomor Transaksi: `␀``
    Rust: (tidak ada padanan)
  - JS: `🧾 TRX: `␀``
    Rust: (tidak ada padanan)

### editlist (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/editlist.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/editlist.rs`

  - JS: `Gunakan: nama, isi, deskripsi, gambar, video 📋`
    Rust: (tidak ada padanan)
  - JS: ``␀editlist 1 gambar` (reply gambar 🖼️)`
    Rust: (tidak ada padanan)
  - JS: `image.jpg`
    Rust: (tidak ada padanan)
  - JS: `video.mp4`
    Rust: (tidak ada padanan)
  - JS: `⏳ _Mengunggah gambar..._`
    Rust: (tidak ada padanan)
  - JS: `⏳ _Mengunggah video..._`
    Rust: (tidak ada padanan)
  - JS: `❌ *Field tidak dikenali.*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal mengunggah gambar.* Coba lagi nanti 🖼️`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal mengunggah video.* Coba lagi nanti 🎬`
    Rust: (tidak ada padanan)
  - JS: `🎬 Video: ✅`
    Rust: (tidak ada padanan)
  - JS: `🖼️ Gambar: ✅`
    Rust: (tidak ada padanan)

### editproduk (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/editproduk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/editproduk.rs`

  - JS: `Gunakan: nama, harga, diskon, stok, tipe, deskripsi, detail, gambar, video 📋`
    Rust: (tidak ada padanan)
  - JS: `Hapus semua stock items terlebih dahulu sebelum mengubah tipe ke Fisik.`
    Rust: (tidak ada padanan)
  - JS: `Harga jual saat ini: Rp ␀ 💰`
    Rust: (tidak ada padanan)
  - JS: `Kirim gambar lalu reply dengan command ini.`
    Rust: (tidak ada padanan)
  - JS: `Kirim video lalu reply dengan command ini.`
    Rust: (tidak ada padanan)
  - JS: `Produk ini memiliki *␀* data akun 🔑`
    Rust: (tidak ada padanan)
  - JS: `Rentang: 1-␀ 📋`
    Rust: (tidak ada padanan)
  - JS: `Silakan chat bot secara langsung 📱`
    Rust: (tidak ada padanan)
  - JS: `Tambahkan produk terlebih dahulu: `␀addproduk` ➕`
    Rust: (tidak ada padanan)
  - JS: `Untuk menjaga privasi 🛡️, pengeditan produk hanya dapat dilakukan di *private chat*.`
    Rust: (tidak ada padanan)
  - JS: `image.jpg`
    Rust: (tidak ada padanan)
  - JS: `video.mp4`
    Rust: (tidak ada padanan)
  - JS: `~~Rp ␀~~`
    Rust: (tidak ada padanan)
  - JS: `⏳ _Mengunggah gambar..._`
    Rust: (tidak ada padanan)
  - JS: `⏳ _Mengunggah video..._`
    Rust: (tidak ada padanan)
  - JS: `␀ Tipe: *␀*`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *Tidak bisa mengubah ke Fisik*`
    Rust: (tidak ada padanan)
  - JS: `✅ *PRODUK DIPERBARUI*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Field tidak dikenali.*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal mengunggah gambar.* Coba lagi nanti 🖼️`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal mengunggah video.* Coba lagi nanti 🎬`
    Rust: (tidak ada padanan)
  - JS: `❌ *Harga diskon harus lebih besar dari harga jual.*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Harga tidak valid.* Minimal Rp 1.000 💰`
    Rust: (tidak ada padanan)
  - JS: `❌ *Nama terlalu pendek.* Minimal 2 karakter 🏷️`
    Rust: (tidak ada padanan)
  - JS: `❌ *Nomor produk tidak valid.*`
    Rust: (tidak ada padanan)
  - JS: `❌ *Stok tidak valid.* Gunakan angka ≥ 0 atau `unlimited``
    Rust: (tidak ada padanan)
  - JS: `❌ *Tipe tidak valid.* Gunakan `digital` 🔑 atau `fisik` 📦`
    Rust: (tidak ada padanan)
  - JS: `🎬 *Reply atau kirim video baru.*`
    Rust: (tidak ada padanan)
  - JS: `🎬 Video: ✅`
    Rust: (tidak ada padanan)
  - JS: `🏷️ Nama: *␀*`
    Rust: (tidak ada padanan)
  - JS: `👀 _Lihat perubahan: `␀listproduk`_`
    Rust: (tidak ada padanan)
  - JS: `💰 Harga: *Rp ␀*`
    Rust: (tidak ada padanan)
  - JS: `📊 Stok: *␀*`
    Rust: (tidak ada padanan)
  - JS: `📭 *Belum ada produk.*`
    Rust: (tidak ada padanan)
  - JS: `🖼️ *Reply atau kirim gambar baru.*`
    Rust: (tidak ada padanan)
  - JS: `🖼️ Gambar: ✅`
    Rust: (tidak ada padanan)
  - JS: `🗑️ Hapus semua: `␀editproduk ␀ stok 0``
    Rust: (tidak ada padanan)
  - JS: `🚫 *Akses Ditolak*`
    Rust: (tidak ada padanan)

### hapusproduk (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/hapusproduk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/hapusproduk.rs`

  - JS: `Ketik `␀hapusproduk <nomor>``
    Rust: `Ketik `.hapusproduk <nomor>``
  - JS: `♾️`
    Rust: (tidak ada padanan)

### hapusstok (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/hapusstok.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/hapusstok.rs`

  - JS: `Ubah tipe stok terlebih dahulu: `␀editproduk ␀ stok <jumlah>``
    Rust: `Ubah tipe stok terlebih dahulu: `.editproduk <nomor> stok <jumlah>``
  - JS: ``␀editproduk ␀ stok <jumlah_baru>``
    Rust: ``␀editproduk <nomor> stok <jumlah_baru>``
  - JS: `📋 Lihat daftar: `␀liststok ␀``
    Rust: `📋 Lihat daftar: `.liststok <nomor>``

### list (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/list.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/list.rs`

  - JS: `💡 _Ketik `␀list <nomor>` untuk membaca detail informasi_`
    Rust: `💡 _Ketik `.list <nomor>` untuk membaca detail informasi_`

### listproduk (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/listproduk.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/listproduk.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)

### liststok (store)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/store/liststok.js`
- Rust: `/workspaces/oktz-rs/plugin/src/store/liststok.rs`

  - JS: ``␀.` ␀␀`
    Rust: ``␀.` ␀`
  - JS: `• Edit: `␀editproduk ␀ stok <jumlah>``
    Rust: `• Edit: `␀editproduk <nomor> stok <jumlah>``
  - JS: `• Import: `␀addstok ␀` (reply file .txt 📄)`
    Rust: `• Import: `␀addstok <nomor>` (reply file .txt 📄)`
  - JS: `• Manual: `␀addstok ␀|<detail>``
    Rust: `• Manual: `␀addstok <nomor>|<detail>``
  - JS: `• Tambah: `␀addstok ␀ <jumlah>``
    Rust: `• Tambah: `␀addstok <nomor> <jumlah>``
  - JS: `✏️ Edit: `␀editstok ␀ <nomor_item>|<detail_baru>``
    Rust: `✏️ Edit: `␀editstok <nomor> <nomor_item>|<detail_baru>``
  - JS: `➕ Tambah: `␀addstok ␀|<detail>``
    Rust: `➕ Tambah: `␀addstok <nomor>|<detail>``
  - JS: `🗑️ Hapus: `␀hapusstok ␀ <nomor_item>``
    Rust: `🗑️ Hapus: `␀hapusstok <nomor> <nomor_item>``

### bandingkan-hp (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/bandingkan-hp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/bandingkan_hp.rs`

  - JS: `*CARA PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `- *1:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *2:* ␀`
    Rust: (tidak ada padanan)
  - JS: `- *␀* tidak ditemukan di database.`
    Rust: (tidak ada padanan)
  - JS: `- Contoh: `␀bandingkan-hp iphone 13 | samsung s22``
    Rust: (tidak ada padanan)
  - JS: `- Ketik `␀bandingkan-hp <hp pertama> | <hp kedua>``
    Rust: (tidak ada padanan)
  - JS: `.bandingkan-hp <hp1> | <hp2>`
    Rust: (tidak ada padanan)
  - JS: `.bandingkan-hp oppo a3s | vivo y91`
    Rust: (tidak ada padanan)
  - JS: `.ct-text-block, .ct-new-columns`
    Rust: (tidak ada padanan)
  - JS: `Fitur ini akan membantumu membandingkan spesifikasi dua *smartphone* secara langsung agar kamu bisa memilih mana yang terbaik!`
    Rust: (tidak ada padanan)
  - JS: `Maaf, sistem mengalami gangguan saat mencoba membandingkan HP dari *Carisinyal*. Silakan coba lagi nanti.`
    Rust: (tidak ada padanan)
  - JS: `Membandingkan dua spesifikasi smartphone dari Carisinyal.`
    Rust: (tidak ada padanan)
  - JS: `Pastikan kamu memasukkan dua nama ponsel dengan pemisah tanda vertikal (|).`
    Rust: (tidak ada padanan)
  - JS: `Silakan coba kata kunci yang berbeda.`
    Rust: (tidak ada padanan)
  - JS: `User-Agent`
    Rust: (tidak ada padanan)
  - JS: `_Pastikan kamu menggunakan tanda garis lurus (|) sebagai pemisah antara dua nama HP tersebut ya!_`
    Rust: (tidak ada padanan)
  - JS: `bandingkan-hp`
    Rust: (tidak ada padanan)
  - JS: `select[name="hp_1"] option`
    Rust: (tidak ada padanan)
  - JS: `⚖️ *FITUR PERBANDINGAN HP*`
    Rust: (tidak ada padanan)
  - JS: `⚖️ *PERBANDINGAN SMARTPHONE*`
    Rust: (tidak ada padanan)
  - JS: `✨ *␀*`
    Rust: (tidak ada padanan)
  - JS: `❌ *FORMAT SALAH*`
    Rust: (tidak ada padanan)
  - JS: `❌ *GAGAL MENGAMBIL DATA*`
    Rust: (tidak ada padanan)
  - JS: `❌ *HP TIDAK DITEMUKAN*`
    Rust: (tidak ada padanan)
  - JS: `📱 *HP 1:* ␀`
    Rust: (tidak ada padanan)
  - JS: `📱 *HP 2:* ␀`
    Rust: (tidak ada padanan)
  - JS: `📸 (Cek Album)`
    Rust: (tidak ada padanan)
  - JS: `🔗 *Sumber:* Carisinyal`
    Rust: (tidak ada padanan)
  - JS: `🔹 *␀:*`
    Rust: (tidak ada padanan)

### carbon (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/carbon.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/carbon.rs`

  - JS: `.export-container`
    Rust: (tidak ada padanan)
  - JS: `14px`
    Rust: (tidak ada padanan)
  - JS: `152%`
    Rust: (tidak ada padanan)
  - JS: `20px`
    Rust: (tidak ada padanan)
  - JS: `2x`
    Rust: (tidak ada padanan)
  - JS: `56px`
    Rust: (tidak ada padanan)
  - JS: `68px`
    Rust: (tidak ada padanan)
  - JS: `Anonymous Pro`
    Rust: (tidak ada padanan)
  - JS: `Droid Sans Mono`
    Rust: (tidak ada padanan)
  - JS: `Fira Code`
    Rust: (tidak ada padanan)
  - JS: `Gagal generate carbon gambar`
    Rust: (tidak ada padanan)
  - JS: `JetBrains Mono`
    Rust: (tidak ada padanan)
  - JS: `Source Code Pro`
    Rust: (tidak ada padanan)
  - JS: `dracula-pro`
    Rust: (tidak ada padanan)
  - JS: `night-owl`
    Rust: (tidak ada padanan)
  - JS: `one-dark`
    Rust: (tidak ada padanan)
  - JS: `panda-syntax`
    Rust: (tidak ada padanan)
  - JS: `rgba(226,233,239,1)`
    Rust: (tidak ada padanan)
  - JS: `solarized-dark`
    Rust: (tidak ada padanan)
  - JS: `solarized-light`
    Rust: (tidak ada padanan)

### caribug (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/caribug.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/caribug.rs`

  - JS: `*Tingkat:* ␀ ␀`
    Rust: `*Tingkat:* ␀`
  - JS: `.caribug function test() {}`
    Rust: `.caribug function test() ␀`
  - JS: `[CariBug]`
    Rust: (tidak ada padanan)
  - JS: ``␀caribug function test() {}``
    Rust: ``␀caribug function test() {␀}``
  - JS: `␀caribug`
    Rust: (tidak ada padanan)

### cekidch (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/cekidch.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/cekidch.rs`

  - JS: `*CONTOH PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `*RINCIAN CHANNEL:*`
    Rust: (tidak ada padanan)
  - JS: `.cekidch <link channel>`
    Rust: (tidak ada padanan)
  - JS: `.cekidch https://whatsapp.com/channel/xxxxx`
    Rust: (tidak ada padanan)
  - JS: `Berikut adalah detail informasi lengkap mengenai channel yang Anda cari:`
    Rust: (tidak ada padanan)
  - JS: `Cek ID dan info lengkap channel dari link`
    Rust: (tidak ada padanan)
  - JS: `Maaf, sistem tidak dapat menemukan informasi dari channel tersebut. Mungkin link sudah kedaluwarsa atau channel telah dihapus.`
    Rust: (tidak ada padanan)
  - JS: `Pastikan link yang Anda masukkan adalah link channel WhatsApp yang sah dan benar.`
    Rust: (tidak ada padanan)
  - JS: `Silakan masukkan link channel WhatsApp yang ingin Anda cek informasinya secara detail.`
    Rust: (tidak ada padanan)
  - JS: `[CekIdCh] Error:`
    Rust: (tidak ada padanan)
  - JS: `© ␀`
    Rust: (tidak ada padanan)
  - JS: `• Dibuat Pada: *␀*`
    Rust: `• Dibuat Pada: *—*`
  - JS: `• ID Channel: `␀``
    Rust: `• ID Channel: `0029ABC``
  - JS: `• Nama: *␀*`
    Rust: `• Nama: *OKTZ*`
  - JS: `• Status: *␀*`
    Rust: (tidak ada padanan)
  - JS: `• Subscriber: *␀*`
    Rust: `• Subscriber: *25*`
  - JS: `• `␀cekidch https://whatsapp.com/channel/xxxxx``
    Rust: (tidak ada padanan)
  - JS: `ℹ️ *INFORMASI PENGGUNAAN*`
    Rust: (tidak ada padanan)
  - JS: `✓ Verified`
    Rust: (tidak ada padanan)
  - JS: `❌ *CHANNEL TIDAK DITEMUKAN*`
    Rust: (tidak ada padanan)
  - JS: `❌ *LINK TIDAK VALID*`
    Rust: (tidak ada padanan)
  - JS: `📋 Ambil ID Saluran nya`
    Rust: (tidak ada padanan)
  - JS: `🔗 Buka Channel nya`
    Rust: (tidak ada padanan)

### cekxl (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/cekxl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/cekxl.rs`

  - JS: `❌ Tidak bisa mengecek nomor *␀*, pastikan nomor tersebut merupakan nomor XL atau Axis yang aktif`
    Rust: `❌ Tidak bisa mengecek nomor *␀*, pastikan nomor tersebut merupakan nomor XL atau Axis yang aktif ya`

### cjstoesm (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/cjstoesm.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/cjstoesm.rs`

  - JS: `"]([^`
    Rust: `]([^'`
  - JS: `.cjstoesm`
    Rust: (tidak ada padanan)
  - JS: `.cjstoesm <reply kode>`
    Rust: (tidak ada padanan)
  - JS: `> *Cara pakai:*`
    Rust: (tidak ada padanan)
  - JS: `> *Contoh CJS:*`
    Rust: (tidak ada padanan)
  - JS: `> Convert CommonJS ke ES Modules`
    Rust: (tidak ada padanan)
  - JS: `> Reply kode CJS dengan ␀cjstoesm`
    Rust: (tidak ada padanan)
  - JS: `> `const axios = require('axios')``
    Rust: (tidak ada padanan)
  - JS: `> `module.exports = handler``
    Rust: (tidak ada padanan)
  - JS: `Convert CommonJS ke ESM (ES Modules)`
    Rust: (tidak ada padanan)
  - JS: `]+)['`
    Rust: (tidak ada padanan)
  - JS: `const __dirname = dirname(__filename);`
    Rust: (tidak ada padanan)
  - JS: `const __filename = fileURLToPath(import.meta.url);`
    Rust: (tidak ada padanan)
  - JS: `export * from '␀';`
    Rust: (tidak ada padanan)
  - JS: `export const ␀ = ␀(␀) => {`
    Rust: (tidak ada padanan)
  - JS: `export const ␀ = ␀;`
    Rust: (tidak ada padanan)
  - JS: `export default ␀(␀) =>`
    Rust: (tidak ada padanan)
  - JS: `export default ␀;`
    Rust: (tidak ada padanan)
  - JS: `export default ␀␀ ␀␀ {`
    Rust: (tidak ada padanan)
  - JS: `export default ␀␀␀ {`
    Rust: (tidak ada padanan)
  - JS: `export { ␀ as ␀ };`
    Rust: (tidak ada padanan)
  - JS: `export { ␀ };`
    Rust: (tidak ada padanan)
  - JS: `export ␀function ␀␀ {`
    Rust: (tidak ada padanan)
  - JS: `import '␀';`
    Rust: (tidak ada padanan)
  - JS: `import { dirname } from 'path';`
    Rust: (tidak ada padanan)
  - JS: `import { fileURLToPath } from 'url';`
    Rust: (tidak ada padanan)
  - JS: `import { ␀ as ␀ } from '␀';`
    Rust: (tidak ada padanan)
  - JS: `import { ␀ } from '␀';`
    Rust: (tidak ada padanan)
  - JS: `import ␀ from '␀';`
    Rust: (tidak ada padanan)
  - JS: `␀ as ␀`
    Rust: (tidak ada padanan)
  - JS: `🔄 *ᴄᴊs ᴛᴏ ᴇsᴍ ᴄᴏɴᴠᴇʀᴛᴇʀ*`
    Rust: (tidak ada padanan)

### converter (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/converter.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/converter.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `[Converter] Error:`
    Rust: (tidak ada padanan)
  - JS: `convert_␀.␀`
    Rust: `␀/convert_␀.␀`

### crm (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/crm.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/crm.rs`

  - JS: `"[Circular]"`
    Rust: (tidak ada padanan)
  - JS: `"[MaxDepth]"`
    Rust: (tidak ada padanan)
  - JS: `(?:\.|[^`
    Rust: (tidak ada padanan)
  - JS: `).toLowerCase() ===`
    Rust: (tidak ada padanan)
  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `, codeBlocks: makeCodeBlocks(code) },`
    Rust: (tidak ada padanan)
  - JS: `]`
    Rust: (tidak ada padanan)
  - JS: `],`
    Rust: (tidak ada padanan)
  - JS: ``]/.test(token)) {`
    Rust: (tidak ada padanan)
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `async function sendSnippet(sock, m, rawQuoted) {`
    Rust: (tidak ada padanan)
  - JS: `await sendSnippet(sock, m, unwrapped);`
    Rust: (tidak ada padanan)
  - JS: `await sock.relayMessage(`
    Rust: (tidak ada padanan)
  - JS: `await sock.relayMessage(relayMsg.key.remoteJid, relayMsg.message, {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMessage(`
    Rust: (tidak ada padanan)
  - JS: `blocks.push({`
    Rust: (tidak ada padanan)
  - JS: `blocks.push({ highlightType, codeContent: token });`
    Rust: (tidak ada padanan)
  - JS: `botForwardedMessage: {`
    Rust: (tidak ada padanan)
  - JS: `botMetadata: {`
    Rust: (tidak ada padanan)
  - JS: `botResponseId:`
    Rust: (tidak ada padanan)
  - JS: `caption: `📄 ␀`,`
    Rust: (tidak ada padanan)
  - JS: `certificateChain: [`
    Rust: (tidak ada padanan)
  - JS: `codeContent: code.slice(lastIndex),`
    Rust: (tidak ada padanan)
  - JS: `codeContent: code.slice(lastIndex, match.index),`
    Rust: (tidak ada padanan)
  - JS: `codeMetadata: { codeLanguage:`
    Rust: (tidak ada padanan)
  - JS: `const code = buildReadableSendCode(rawQuoted);`
    Rust: (tidak ada padanan)
  - JS: `const isSnippet = (m.args?.[0] ||`
    Rust: (tidak ada padanan)
  - JS: `const rawQuoted = m.quoted?.message;`
    Rust: (tidak ada padanan)
  - JS: `const relayContent = normalizeForRelay(rawQuoted);`
    Rust: (tidak ada padanan)
  - JS: `const relayMsg = generateWAMessageFromContent(m.chat, relayContent, {`
    Rust: (tidak ada padanan)
  - JS: `const token = match[0];`
    Rust: (tidak ada padanan)
  - JS: `const unwrapped = unwrap(rawQuoted);`
    Rust: (tidak ada padanan)
  - JS: `const { filepath, filename } = buildRelayCodeFile(unwrapped);`
    Rust: (tidak ada padanan)
  - JS: `contextInfo: {`
    Rust: (tidak ada padanan)
  - JS: `deviceListMetadata: {},`
    Rust: (tidak ada padanan)
  - JS: `deviceListMetadataVersion: 2,`
    Rust: (tidak ada padanan)
  - JS: `document: readFileSync(filepath),`
    Rust: (tidak ada padanan)
  - JS: `export { pluginConfig as config, handler };`
    Rust: (tidak ada padanan)
  - JS: `fileName: filename,`
    Rust: (tidak ada padanan)
  - JS: `forwardOrigin: 4,`
    Rust: (tidak ada padanan)
  - JS: `forwardedAiBotMessageInfo: { botJid:`
    Rust: (tidak ada padanan)
  - JS: `forwardingScore: 1,`
    Rust: (tidak ada padanan)
  - JS: `highlightType = 1;`
    Rust: (tidak ada padanan)
  - JS: `highlightType = 2;`
    Rust: (tidak ada padanan)
  - JS: `highlightType = 3;`
    Rust: (tidak ada padanan)
  - JS: `highlightType = 4;`
    Rust: (tidak ada padanan)
  - JS: `highlightType: 0,`
    Rust: (tidak ada padanan)
  - JS: `if (!rawQuoted) {`
    Rust: (tidak ada padanan)
  - JS: `if (!unwrapped || typeof unwrapped !==`
    Rust: (tidak ada padanan)
  - JS: `if (/^(const|let|var|function|return|class|static|new|async|await|if|else|for|while|do|switch|case|break|continue|try|catch|finally|throw|import|export|from|default|extends|typeof|instanceof|in|of|delete|void|yield)$/.test(token)) {`
    Rust: (tidak ada padanan)
  - JS: `if (isSnippet) {`
    Rust: (tidak ada padanan)
  - JS: `if (lastIndex < code.length) {`
    Rust: (tidak ada padanan)
  - JS: `if (match.index > lastIndex) {`
    Rust: (tidak ada padanan)
  - JS: `isForwarded: true,`
    Rust: (tidak ada padanan)
  - JS: `lastIndex = regex.lastIndex;`
    Rust: (tidak ada padanan)
  - JS: `let highlightType = 0;`
    Rust: (tidak ada padanan)
  - JS: `let lastIndex = 0;`
    Rust: (tidak ada padanan)
  - JS: `let match;`
    Rust: (tidak ada padanan)
  - JS: `m.react(`
    Rust: (tidak ada padanan)
  - JS: `m.reply(`⚠️ Gagal kirim snippet: ␀`);`
    Rust: (tidak ada padanan)
  - JS: `m.reply(te(m.prefix, m.command, m.pushName));`
    Rust: (tidak ada padanan)
  - JS: `message: {`
    Rust: (tidak ada padanan)
  - JS: `messageContextInfo: {`
    Rust: (tidak ada padanan)
  - JS: `messageDisclaimerText:`
    Rust: (tidak ada padanan)
  - JS: `messageId: relayMsg.key.id,`
    Rust: (tidak ada padanan)
  - JS: `messageType: 5,`
    Rust: (tidak ada padanan)
  - JS: `mimetype:`
    Rust: (tidak ada padanan)
  - JS: `proofs: [`
    Rust: (tidak ada padanan)
  - JS: `quoted: m,`
    Rust: (tidak ada padanan)
  - JS: `return blocks;`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`❌ Reply pesan yang mau di-relay dulu, lalu ketik *␀␀*`);`
    Rust: `❌ Reply pesan yang mau di-relay dulu, lalu ketik *␀␀*`
  - JS: `return;`
    Rust: (tidak ada padanan)
  - JS: `richResponseMessage: {`
    Rust: (tidak ada padanan)
  - JS: `signature:`
    Rust: (tidak ada padanan)
  - JS: `submessages: [`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `useCase: 1,`
    Rust: (tidak ada padanan)
  - JS: `userJid: sock.user?.id,`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `verificationMetadata: {`
    Rust: (tidak ada padanan)
  - JS: `version: 1,`
    Rust: (tidak ada padanan)
  - JS: `while ((match = regex.exec(code)) !== null) {`
    Rust: (tidak ada padanan)
  - JS: `{ messageType: 2, messageText:`
    Rust: (tidak ada padanan)
  - JS: `{ quoted: m }`
    Rust: (tidak ada padanan)
  - JS: `{}`
    Rust: `{␀}`
  - JS: `|"(?:\.|[^"\])*"|`(?:\.|[^`\])*`|b(?:const|let|var|function|return|class|static|new|async|await|if|else|for|while|do|switch|case|break|continue|try|catch|finally|throw|import|export|from|default|extends|typeof|instanceof|in|of|delete|void|yield)b|b(?:true|false|null|undefined)b|bd+(?:.d+)?b)/g;`
    Rust: (tidak ada padanan)
  - JS: `|| !Object.keys(unwrapped).length) {`
    Rust: (tidak ada padanan)
  - JS: `|| /snipp/i.test(m.command);`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `} catch (error) {`
    Rust: (tidak ada padanan)
  - JS: `} else if (/^(true|false|null|undefined)$/.test(token) || /^d/.test(token)) {`
    Rust: (tidak ada padanan)
  - JS: `} else if (/^///.test(token) || /^/*/.test(token)) {`
    Rust: (tidak ada padanan)
  - JS: `} else if (/^[`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `};`
    Rust: (tidak ada padanan)
  - JS: `␀}`
    Rust: `␀}}`

### dafont (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/dafont.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/dafont.rs`

  - JS: `_Setelah daftar muncul, reply pesan bot dengan nomor font buat download_`
    Rust: `_Setelah daftar muncul, reply pesan bot dengan nomor font buat download file-nya_`
  - JS: `❌ Gagal mencari font, coba lagi nanti`
    Rust: `Gagal mencari font, coba lagi nanti`

### delpp (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/delpp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/delpp.rs`

  - JS: `> Foto profil bot berhasil dihapus!`
    Rust: (tidak ada padanan)
  - JS: `> Tidak dapat menghapus foto bot.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴘᴘ ʙᴏᴛ ᴅɪʜᴀᴘᴜs*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Bot JID tidak ditemukan.`
    Rust: (tidak ada padanan)

### deploy (owner)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/deploy.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/deploy.rs`

  - JS: `*CONTOH PENGGUNAAN:*`
    Rust: (tidak ada padanan)
  - JS: `*Penyebab Error:*`
    Rust: (tidak ada padanan)
  - JS: `*RINCIAN DEPLOY:*`
    Rust: (tidak ada padanan)
  - JS: `.htm`
    Rust: (tidak ada padanan)
  - JS: `.html`
    Rust: (tidak ada padanan)
  - JS: `.vercel.app`
    Rust: (tidak ada padanan)
  - JS: `.zip`
    Rust: (tidak ada padanan)
  - JS: `<!DOCTYPE html>`
    Rust: (tidak ada padanan)
  - JS: `</body>`
    Rust: (tidak ada padanan)
  - JS: `</head>`
    Rust: (tidak ada padanan)
  - JS: `</html>`
    Rust: (tidak ada padanan)
  - JS: `<body>`
    Rust: (tidak ada padanan)
  - JS: `<head>`
    Rust: (tidak ada padanan)
  - JS: `<html lang="id">`
    Rust: (tidak ada padanan)
  - JS: `<meta charset="UTF-8">`
    Rust: (tidak ada padanan)
  - JS: `<meta name="viewport" content="width=device-width, initial-scale=1.0">`
    Rust: (tidak ada padanan)
  - JS: `<title>␀</title>`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `Bearer ␀`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Deploy HTML atau file ZIP ke Vercel`
    Rust: (tidak ada padanan)
  - JS: `File ZIP yang Anda unggah tidak berisi file apapun. Pastikan file ZIP tersebut berisi proyek HTML/Web statis.`
    Rust: (tidak ada padanan)
  - JS: `Fitur ini digunakan untuk melakukan *deploy* (hosting) kode HTML atau file proyek lengkap (ZIP) secara langsung ke Vercel.`
    Rust: (tidak ada padanan)
  - JS: `Proyek Anda telah berhasil diunggah ke Vercel dan saat ini sedang dalam proses penyebaran (building). Anda dapat segera mengaksesnya melalui tautan berikut.`
    Rust: (tidak ada padanan)
  - JS: `Sistem hanya mendukung deploy dari format berikut:`
    Rust: (tidak ada padanan)
  - JS: `Terjadi kesalahan saat mencoba mengunggah proyek ke Vercel.`
    Rust: (tidak ada padanan)
  - JS: `Token Vercel belum dikonfigurasi di pengaturan sistem. Silakan atur `config.vercel.token` terlebih dahulu.`
    Rust: (tidak ada padanan)
  - JS: `adm-zip`
    Rust: (tidak ada padanan)
  - JS: `index.html`
    Rust: (tidak ada padanan)
  - JS: `utf-8`
    Rust: (tidak ada padanan)
  - JS: `• Dokumen `.html``
    Rust: (tidak ada padanan)
  - JS: `• Dokumen arsip `.zip``
    Rust: (tidak ada padanan)
  - JS: `• Nama Proyek: *␀*`
    Rust: (tidak ada padanan)
  - JS: `• Reply file `.html` atau `.zip` dengan perintah: `␀deploy namasitus``
    Rust: (tidak ada padanan)
  - JS: `• Reply teks/kode HTML dengan perintah: `␀deploy namasitus``
    Rust: (tidak ada padanan)
  - JS: `• Tautan: https://␀`
    Rust: (tidak ada padanan)
  - JS: `• Teks kode HTML`
    Rust: (tidak ada padanan)
  - JS: `• Tipe Proyek: *␀*`
    Rust: (tidak ada padanan)
  - JS: `ℹ️ *INFORMASI PENGGUNAAN*`
    Rust: (tidak ada padanan)
  - JS: `␀.vercel.app`
    Rust: (tidak ada padanan)
  - JS: `✅ *DEPLOY BERHASIL*`
    Rust: (tidak ada padanan)
  - JS: `❌ *DEPLOY GAGAL*`
    Rust: (tidak ada padanan)
  - JS: `❌ *FILE ZIP KOSONG*`
    Rust: (tidak ada padanan)
  - JS: `❌ *FORMAT TIDAK DIDUKUNG*`
    Rust: (tidak ada padanan)
  - JS: `❌ *TOKEN BELUM DIATUR*`
    Rust: (tidak ada padanan)

### emojitoimage (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/emojitoimage.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/emojitoimage.rs`

  - JS: `🖼️`
    Rust: (tidak ada padanan)

### esmtocjs (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/esmtocjs.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/esmtocjs.rs`

  - JS: `"]([^`
    Rust: `]([^'`
  - JS: `.esmtocjs`
    Rust: (tidak ada padanan)
  - JS: `.esmtocjs <reply kode>`
    Rust: (tidak ada padanan)
  - JS: `> *Cara pakai:*`
    Rust: (tidak ada padanan)
  - JS: `> *Contoh ESM:*`
    Rust: (tidak ada padanan)
  - JS: `> Convert ES Modules ke CommonJS`
    Rust: (tidak ada padanan)
  - JS: `> Reply kode ESM dengan ␀esmtocjs`
    Rust: (tidak ada padanan)
  - JS: `> `export default function() {}``
    Rust: (tidak ada padanan)
  - JS: `> `import axios from 'axios'``
    Rust: (tidak ada padanan)
  - JS: `Convert ESM (ES Modules) ke CommonJS`
    Rust: (tidak ada padanan)
  - JS: `Object.assign(module.exports, require('␀'))`
    Rust: (tidak ada padanan)
  - JS: `const __default__ = ␀␀ {`
    Rust: (tidak ada padanan)
  - JS: `const { ␀ } = require('␀')`
    Rust: `const {{ ␀ }} = require('␀')`
  - JS: `module.exports = { ...module.exports, ␀ }`
    Rust: (tidak ada padanan)
  - JS: `module.exports = { ␀ }`
    Rust: (tidak ada padanan)
  - JS: `module.exports = ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ ␀␀ {`
    Rust: `␀ ␀ =`
  - JS: `␀: ␀`
    Rust: (tidak ada padanan)
  - JS: `🔄 *ᴇsᴍ ᴛᴏ ᴄᴊs ᴄᴏɴᴠᴇʀᴛᴇʀ*`
    Rust: (tidak ada padanan)

### remini (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/hd.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/hd.rs`

  - JS: `);`
    Rust: (tidak ada padanan)
  - JS: `*/`
    Rust: (tidak ada padanan)
  - JS: `,`
    Rust: (tidak ada padanan)
  - JS: `, String(method));`
    Rust: (tidak ada padanan)
  - JS: `, blob,`
    Rust: (tidak ada padanan)
  - JS: `, {`
    Rust: (tidak ada padanan)
  - JS: `> Gambar telah berhasil di-upscale dan dijernihkan.`,`
    Rust: (tidak ada padanan)
  - JS: ``*🪁 HD IMAGE*`
    Rust: `*🪁 HD IMAGE*`
  - JS: ````␀remini`````
    Rust: ``␀remini``
  - JS: `async function handler(m, { sock }) {`
    Rust: (tidak ada padanan)
  - JS: `async function photoihancer(imageBuffer, method = 1) {`
    Rust: (tidak ada padanan)
  - JS: `await sock.sendMessage(`
    Rust: (tidak ada padanan)
  - JS: `body: form,`
    Rust: (tidak ada padanan)
  - JS: `caption: `✅ *BERHASIL*`
    Rust: (tidak ada padanan)
  - JS: `console.error("[HD Error]", err);`
    Rust: (tidak ada padanan)
  - JS: `const blob = new Blob([imageBuffer], { type:`
    Rust: (tidak ada padanan)
  - JS: `const enhancedBuffer = await photoihancer(b);`
    Rust: (tidak ada padanan)
  - JS: `const form = new FormData();`
    Rust: (tidak ada padanan)
  - JS: `const img = m.isImage || (m.quoted && m.quoted.type === "imageMessage");`
    Rust: (tidak ada padanan)
  - JS: `const res = await fetch(`
    Rust: (tidak ada padanan)
  - JS: `export { config, handler };`
    Rust: (tidak ada padanan)
  - JS: `form.set(`
    Rust: (tidak ada padanan)
  - JS: `headers: {`
    Rust: (tidak ada padanan)
  - JS: `if (!img) {`
    Rust: (tidak ada padanan)
  - JS: `if (!res.ok) throw new Error(`␀ ␀`);`
    Rust: (tidak ada padanan)
  - JS: `image: enhancedBuffer,`
    Rust: (tidak ada padanan)
  - JS: `let b = m.quoted?.isMedia ? await m.quoted.download() : await m.download();`
    Rust: (tidak ada padanan)
  - JS: `m.chat,`
    Rust: (tidak ada padanan)
  - JS: `m.react("✅");`
    Rust: (tidak ada padanan)
  - JS: `m.react("❌");`
    Rust: (tidak ada padanan)
  - JS: `m.react("🕕");`
    Rust: (tidak ada padanan)
  - JS: `m.reply("Maaf, terjadi kesalahan saat memproses gambar dari scraper. Coba lagi nanti ya.");`
    Rust: (tidak ada padanan)
  - JS: `method:`
    Rust: (tidak ada padanan)
  - JS: `return Buffer.from(await res.arrayBuffer());`
    Rust: (tidak ada padanan)
  - JS: `return m.reply(`
    Rust: (tidak ada padanan)
  - JS: `t delete the credit`
    Rust: (tidak ada padanan)
  - JS: `try {`
    Rust: (tidak ada padanan)
  - JS: `{`
    Rust: (tidak ada padanan)
  - JS: `{ quoted: m }`
    Rust: (tidak ada padanan)
  - JS: `}`
    Rust: (tidak ada padanan)
  - JS: `} catch (err) {`
    Rust: (tidak ada padanan)
  - JS: `});`
    Rust: (tidak ada padanan)
  - JS: `},`
    Rust: (tidak ada padanan)

### hd2 (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/hd2.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/hd2.rs`

  - JS: `// memproses`
    Rust: (tidak ada padanan)
  - JS: `[HD2 Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `gagal`
    Rust: (tidak ada padanan)
  - JS: `hd-␀.jpg`
    Rust: `␀/hd-␀.jpg`
  - JS: `m.download()`
    Rust: (tidak ada padanan)
  - JS: `node:crypto`
    Rust: (tidak ada padanan)
  - JS: `node:fs/promises`
    Rust: (tidak ada padanan)
  - JS: `node:os`
    Rust: (tidak ada padanan)
  - JS: `node:path`
    Rust: (tidak ada padanan)

### hd3 (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/hd3.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/hd3.rs`

  - JS: `- Atau balas (reply) gambar yang sudah terkirim dengan perintah *␀hd3*`
    Rust: `- Atau balas (reply) gambar yang sudah terkirim dengan perintah *.hd3*`
  - JS: `- Kirim gambar dan tambahkan pesan *␀hd3*`
    Rust: `- Kirim gambar dan tambahkan pesan *.hd3*`
  - JS: `[HD3 Plugin Error]`
    Rust: (tidak ada padanan)
  - JS: `hd-␀.jpg`
    Rust: `␀/hd-␀.jpg`
  - JS: `node:crypto`
    Rust: (tidak ada padanan)
  - JS: `node:fs/promises`
    Rust: (tidak ada padanan)
  - JS: `node:os`
    Rust: (tidak ada padanan)
  - JS: `node:path`
    Rust: (tidak ada padanan)

### hdvid (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/hdvid.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/hdvid.rs`

  - JS: `-c:a copy`
    Rust: (tidak ada padanan)
  - JS: `-c:v libx264`
    Rust: (tidak ada padanan)
  - JS: `-crf 23`
    Rust: `-crf`
  - JS: `-preset fast`
    Rust: `-preset`
  - JS: `Estimasi waktu tergantung ukuran video, mohon bersabar ya kak!`
    Rust: (tidak ada padanan)
  - JS: `HDVID-␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `Meningkatkan kualitas video menjadi HD dengan pure FFMPEG`
    Rust: `Meningkatkan kualitas video menjadi HD dengan pure FFmpeg`
  - JS: `Video kakak sedang diproses agar menjadi HD! ✨`
    Rust: (tidak ada padanan)
  - JS: `fluent-ffmpeg`
    Rust: (tidak ada padanan)
  - JS: `input-hd-␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `output-hd-␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `scale=iw*2:ih*2:flags=lanczos`
    Rust: (tidak ada padanan)
  - JS: `unsharp=5:5:1.0:5:5:0.0`
    Rust: (tidak ada padanan)
  - JS: `🎞️ *PROSES ENHANCE DIMULAI* 🎞️`
    Rust: (tidak ada padanan)

### hitungwrmlbb (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/hitungwrmlbb.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/hitungwrmlbb.rs`

  - JS: `[WR MLBB Error]`
    Rust: (tidak ada padanan)
  - JS: `😔 Terjadi kesalahan sistem saat menghitung Win Rate. Mohon coba lagi nanti.`
    Rust: (tidak ada padanan)

### imgtoprompt (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/imgtoprompt.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/imgtoprompt.rs`

  - JS: `> Menganalisis gambar untuk menghasilkan prompt`
    Rust: (tidak ada padanan)
  - JS: `[ImgToPrompt Error]`
    Rust: (tidak ada padanan)
  - JS: `img2prompt_␀.webp`
    Rust: (tidak ada padanan)
  - JS: `🕕 *ᴍᴇᴍᴘʀᴏsᴇs ɢᴀᴍʙᴀʀ...*`
    Rust: (tidak ada padanan)

### invoicemaker (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/invoicemaker.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/invoicemaker.rs`

  - JS: `&apikey=␀`
    Rust: (tidak ada padanan)
  - JS: `&date=␀`
    Rust: (tidak ada padanan)
  - JS: `&image=␀`
    Rust: (tidak ada padanan)
  - JS: `&invoice=␀`
    Rust: (tidak ada padanan)
  - JS: `&items=␀`
    Rust: (tidak ada padanan)
  - JS: `&status=␀`
    Rust: (tidak ada padanan)
  - JS: `&total=␀`
    Rust: (tidak ada padanan)
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `API tidak mengembalikan data yang valid`
    Rust: `> API tidak mengembalikan data yang valid`
  - JS: `store=␀`
    Rust: (tidak ada padanan)

### ipwho (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/ipwho.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/ipwho.rs`

  - JS: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴜɪ*`
  - JS: `🕕 *ᴍᴇɴᴄᴀʀɪ ɪɴꜰᴏ ɪᴘ...*`
    Rust: (tidak ada padanan)

### izen (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/izen.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/izen.rs`

  - JS: `Error: ␀`
    Rust: (tidak ada padanan)
  - JS: `👉 `␀izen <link>``
    Rust: `👉 `.izen <link>``
  - JS: `👉 `␀izen https://sfl.gl/xxxxx``
    Rust: `👉 `.izen https://sfl.gl/xxxxx``

### kalkulatormbg (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/kalkulatormbg.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/kalkulatormbg.rs`

  - JS: `Error: ␀`
    Rust: (tidak ada padanan)
  - JS: ``␀kkmbg 1000000000``
    Rust: ``.kkmbg 1000000000``
  - JS: `id-ID`
    Rust: (tidak ada padanan)
  - JS: `❌ Maaf kak, terjadi kesalahan saat menghitung! 😭`
    Rust: (tidak ada padanan)
  - JS: `🏢 UMR DKI Jakarta (Rp 5,4 Jt/bulan): ␀`
    Rust: `🏢 UMR DKI Jakarta (Rp 5,4 Jt/bulan): ␀%`
  - JS: `🏭 UMR Jawa Tengah (Rp 2,04 Jt/bulan): ␀`
    Rust: `🏭 UMR Jawa Tengah (Rp 2,04 Jt/bulan): ␀%`
  - JS: `👨‍🏫 Gaji Guru Honorer (Rp 300rb/bulan): ␀`
    Rust: `👨‍🏫 Gaji Guru Honorer (Rp 300rb/bulan): ␀x`

### lookup (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/lookup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/lookup.rs`

  - JS: `🕕 *ᴍᴇɴᴄᴀʀɪ ɪɴꜰᴏ ᴅᴏᴍᴀɪɴ...*`
    Rust: (tidak ada padanan)

### musikapaini (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/musikapaini.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/musikapaini.rs`

  - JS: `form-data`
    Rust: (tidak ada padanan)

### nulis (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/nulis.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/nulis.rs`

  - JS: `18px Zahraaa, Arial`
    Rust: (tidak ada padanan)
  - JS: `20px Zahraaa, Arial`
    Rust: (tidak ada padanan)
  - JS: `2d`
    Rust: (tidak ada padanan)
  - JS: `> Contoh:`
    Rust: (tidak ada padanan)
  - JS: `> File template kertas tidak ditemukan di config.assets`
    Rust: (tidak ada padanan)
  - JS: `> Hatihati ketahuan! 📖`
    Rust: (tidak ada padanan)
  - JS: `> Maksimal 500 karakter`
    Rust: (tidak ada padanan)
  - JS: `> Membuat tulisan tangan...`
    Rust: (tidak ada padanan)
  - JS: `> `␀nulis <teks>``
    Rust: `.nulis <teks>`
  - JS: `> `␀nulis Aku cinta kamu selamanya``
    Rust: `.nulis Aku cinta kamu selamanya`
  - JS: `Gagal load font:`
    Rust: (tidak ada padanan)
  - JS: `Generate tulisan tangan di kertas`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʟᴜʟɪsᴀɴ ᴛᴀɴɢᴀɴ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴛᴇᴋs ᴛᴇʀʟᴀʟᴜ ᴘᴀɴᴊᴀɴɢ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴛᴇᴍᴘʟᴀᴛᴇ ᴛɪᴅᴀᴋ ᴀᴅᴀ*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *ᴍᴇᴍᴘʀᴏsᴇs...*`
    Rust: (tidak ada padanan)

### ocr (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/ocr.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/ocr.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `.join("`
    Rust: (tidak ada padanan)
  - JS: `␀`)`
    Rust: (tidak ada padanan)

### pastebin (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/pastebin.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/getpaste.rs`

  - JS: `*Cara pakai:*`
    Rust: (tidak ada padanan)
  - JS: `.pastebin <text>`
    Rust: (tidak ada padanan)
  - JS: `.pastebin console.log("Hello World")`
    Rust: (tidak ada padanan)
  - JS: `> Ambil key di: https://pastebin.com/api`
    Rust: (tidak ada padanan)
  - JS: `> Contoh: `␀pastebin console.log("Hello")``
    Rust: (tidak ada padanan)
  - JS: `> Lalu restart bot`
    Rust: (tidak ada padanan)
  - JS: `> Owner harus set `PASTEBIN_DEV_KEY` di file `.env``
    Rust: (tidak ada padanan)
  - JS: `> Paste akan expired sesuai pengaturan Pastebin.`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `Bad API request`
    Rust: (tidak ada padanan)
  - JS: `Content-Type`
    Rust: (tidak ada padanan)
  - JS: `Kirim teks untuk di-upload ke Pastebin.`
    Rust: (tidak ada padanan)
  - JS: `Paste dari ␀ - ␀`
    Rust: (tidak ada padanan)
  - JS: `Pastebin Upload`
    Rust: (tidak ada padanan)
  - JS: `Upload teks ke Pastebin`
    Rust: (tidak ada padanan)
  - JS: `• Reply teks dengan `␀pastebin``
    Rust: (tidak ada padanan)
  - JS: `• `␀pastebin <text>``
    Rust: (tidak ada padanan)
  - JS: `┃ 📊 ᴜᴋᴜʀᴀɴ: *␀ chars*`
    Rust: (tidak ada padanan)
  - JS: `┃ 📝 ᴊᴜᴅᴜʟ: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔗 ʟɪɴᴋ: ␀`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴅᴇᴛᴀɪʟ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴘᴀsᴛᴇʙɪɴ ʙᴇʀʜᴀsɪʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *PASTEBIN API Key belum diset!*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)
  - JS: `📋 *ᴘᴀsᴛᴇʙɪɴ ᴜᴘʟᴏᴀᴅ*`
    Rust: (tidak ada padanan)

### phisingataubukan (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/phisingataubukan.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/phisingataubukan.rs`

  - JS: `- Membawa ke Situs Berbahaya: ␀`
    Rust: `- Membawa ke Situs Berbahaya:`
  - JS: `- Mengandung Malware: ␀`
    Rust: `- Mengandung Malware:`
  - JS: `- Menginstal Software Jahat: ␀`
    Rust: `- Menginstal Software Jahat:`
  - JS: `- Phising: ␀`
    Rust: `- Phising:`
  - JS: `- Unduhan Tidak Wajar: ␀`
    Rust: `- Unduhan Tidak Wajar:`
  - JS: `[Phishing Check Error]`
    Rust: (tidak ada padanan)

### ptv (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/ptv.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/ptv.rs`

  - JS: `❌ Gagal download video dari quoted.`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal download video.`
    Rust: (tidak ada padanan)
  - JS: `🕕 *ᴍᴇᴍʙᴜᴀᴛ ᴘᴛᴠ...*`
    Rust: (tidak ada padanan)

### qrcustom (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/qrcustom.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/txt2qr.rs`

  - JS: `*Contoh:*`
    Rust: (tidak ada padanan)
  - JS: `.qrcustom <url>`
    Rust: (tidak ada padanan)
  - JS: `.qrcustom https://wa.me/628xxx`
    Rust: (tidak ada padanan)
  - JS: `> `␀qrcustom <url/text>``
    Rust: (tidak ada padanan)
  - JS: `> `␀qrcustom https://wa.me/628xxx``
    Rust: (tidak ada padanan)
  - JS: `> ␀␀`
    Rust: `> ␀`
  - JS: `Generate QR code custom dengan logo`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*`
    Rust: (tidak ada padanan)
  - JS: `💡 Reply gambar untuk custom logo di tengah QR`
    Rust: (tidak ada padanan)
  - JS: `📱 *QR Code*`
    Rust: (tidak ada padanan)
  - JS: `🕕 *Generating QR code...*`
    Rust: (tidak ada padanan)

### qwa (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/qwa.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/qwa.rs`

  - JS: `Error QWA:`
    Rust: (tidak ada padanan)
  - JS: `Gagal download/upload media quoted:`
    Rust: (tidak ada padanan)
  - JS: `Gagal download/upload media utama:`
    Rust: (tidak ada padanan)
  - JS: `awesome-phonenumber`
    Rust: (tidak ada padanan)

### removebg (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/removebg.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/removebg.rs`

  - JS: `[RemoveBG Error]`
    Rust: (tidak ada padanan)
  - JS: `rmbg_␀.jpg`
    Rust: (tidak ada padanan)
  - JS: `❌ Buffer gambar tidak valid`
    Rust: (tidak ada padanan)

### rvo (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/rvo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/rvo.rs`

  - JS: `404`
    Rust: (tidak ada padanan)
  - JS: `> ␀`
    Rust: (tidak ada padanan)
  - JS: `Gagal download`
    Rust: (tidak ada padanan)
  - JS: `Gagal download media`
    Rust: (tidak ada padanan)
  - JS: `Media sudah kadaluarsa atau sudah dihapus dari server WhatsApp.`
    Rust: (tidak ada padanan)
  - JS: `_Pesan View Once yang terlalu lama atau sering dibuka biasanya akan otomatis hangus dari sistem WhatsApp dan tidak bisa diunduh lagi._`
    Rust: (tidak ada padanan)
  - JS: ``Pesan :``
    Rust: (tidak ada padanan)
  - JS: `rvo_␀.␀`
    Rust: (tidak ada padanan)
  - JS: `❌ *Gagal Membuka View Once*`
    Rust: (tidak ada padanan)
  - JS: `📎 View once media`
    Rust: (tidak ada padanan)

### sendngl (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/sendngl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/sendngl.rs`

  - JS: `Contoh: `␀sendngl https://ngl.link/xxxx | hai`
    Rust: `Contoh: `␀sendngl https://ngl.link/xxxx | hai``

### setbio (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/setbio.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/setbio.rs`

  - JS: `> Bio bot berhasil dihapus!`
    Rust: `berhasil dihapus`
  - JS: `> Bio bot sekarang:`
    Rust: `Bio bot sekarang`
  - JS: `> Tidak dapat mengubah bio bot.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ *ʙɪᴏ ʙᴏᴛ ᴅɪʜᴀᴘᴜs*`
    Rust: `✅ *ʙɪᴏ ʙᴏᴛ ᴅɪᴜʙᴀʜ*`
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)

### setname (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/setname.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/setname.rs`

  - JS: `> Nama bot sekarang: *␀*`
    Rust: `Nama bot sekarang`
  - JS: `> Tidak dapat mengubah nama bot.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ *ɴᴀᴍᴀ ʙᴏᴛ ᴅɪᴜʙᴀʜ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)

### setpp (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/setpp.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/setpp.rs`

  - JS: `> Foto profil bot berhasil diperbarui!`
    Rust: (tidak ada padanan)
  - JS: `> Tidak dapat mengubah foto bot.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴘᴘ ʙᴏᴛ ᴅɪᴜʙᴀʜ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ɢᴀɢᴀʟ*`
    Rust: (tidak ada padanan)
  - JS: `❌ Bot JID tidak ditemukan.`
    Rust: (tidak ada padanan)
  - JS: `❌ Gagal mengambil gambar.`
    Rust: (tidak ada padanan)

### spamngl (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/spamngl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/spamngl.rs`

  - JS: `Contoh: `␀spamngl https://ngl.link/xxxx | hai | 10`
    Rust: `Contoh: `␀spamngl https://ngl.link/xxxx | hai | 10``

### styleteks (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/styleteks.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/styleteks.rs`

  - JS: `Maaf, sistem sedang mengalami gangguan saat mencoba mengubah gaya teksmu.`
    Rust: (tidak ada padanan)
  - JS: `❌ *GAGAL MENGUBAH TEKS*`
    Rust: (tidak ada padanan)

### tempmail (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/tempmail.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/tempmail.rs`

  - JS: `> *␀tempmail create* — Bikin email baru`
    Rust: `> *.tempmail create* — Bikin email baru`
  - JS: `> *␀tempmail inbox* — Cek pesan masuk`
    Rust: `> *.tempmail inbox* — Cek pesan masuk`
  - JS: `> Belum punya email, ketik *␀tempmail create* dulu`
    Rust: `> Belum punya email, ketik *.tempmail create* dulu`
  - JS: `> Gunakan *␀tempmail create* atau *␀tempmail inbox*`
    Rust: `> Gunakan *.tempmail create* atau *.tempmail inbox*`
  - JS: `Cek pesan masuk dengan *␀tempmail inbox*`
    Rust: `Cek pesan masuk dengan *.tempmail inbox*`
  - JS: `Ketik *␀tempmail create* dulu.`
    Rust: `Ketik *.tempmail create* dulu.`

### toaudio (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/toaudio.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/toaudio.rs`

  - JS: `audio_␀.mp3`
    Rust: `␀/audio_␀.mp3`
  - JS: `ffmpeg -y -i "␀" -vn -ar 44100 -ac 2 -b:a 192k "␀"`
    Rust: (tidak ada padanan)
  - JS: `input_␀.␀`
    Rust: `␀/input_␀.␀`

### toimg (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/toimg.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/toimg.rs`

  - JS: `> Coba kirim ulang stickernya.`
    Rust: (tidak ada padanan)
  - JS: `> File sticker tidak valid atau rusak.`
    Rust: (tidak ada padanan)
  - JS: `> Gunakan `␀tovideo` untuk mengubahnya.`
    Rust: (tidak ada padanan)
  - JS: `> Sticker ini adalah sticker animasi (GIF).`
    Rust: (tidak ada padanan)
  - JS: `> Sticker mungkin sudah tidak tersedia.`
    Rust: (tidak ada padanan)
  - JS: `> Terjadi kesalahan saat memproses.`
    Rust: (tidak ada padanan)
  - JS: `> Tidak dapat mengunduh sticker.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `⚠️ *sᴛɪᴄᴋᴇʀ ᴀɴɪᴍᴀsɪ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ᴇʀʀᴏʀ*`
    Rust: (tidak ada padanan)
  - JS: `❌ *ꜰɪʟᴇ ᴋᴏʀᴜᴘ*`
    Rust: (tidak ada padanan)

### tourl (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/tourl.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/tourl.rs`

  - JS: `*Cara Pakai:*`
    Rust: (tidak ada padanan)
  - JS: `.tourl`
    Rust: (tidak ada padanan)
  - JS: `.tourl (reply/kirim media) | .tourl <teks>`
    Rust: (tidak ada padanan)
  - JS: `// function`
    Rust: (tidak ada padanan)
  - JS: `> Batas: *␀*`
    Rust: (tidak ada padanan)
  - JS: `> Gagal di server: ␀`
    Rust: (tidak ada padanan)
  - JS: `> Kebanyakan host gratisan nggak kuat file segede itu,`
    Rust: (tidak ada padanan)
  - JS: `> Ukuran: *␀*`
    Rust: (tidak ada padanan)
  - JS: `C-Net`
    Rust: (tidak ada padanan)
  - JS: `Halo kak! Butuh link untuk media kamu? Aku bisa bantu uploadin ke berbagai server gratisan loh!`
    Rust: (tidak ada padanan)
  - JS: `Paste.rs`
    Rust: (tidak ada padanan)
  - JS: `T O U R L`
    Rust: (tidak ada padanan)
  - JS: `Teks kamu udah di-paste, yuk ambil linknya! ✨`
    Rust: (tidak ada padanan)
  - JS: `Upload media ke multiple host, atau paste teks dan dapet link`
    Rust: (tidak ada padanan)
  - JS: `Yeay! Media kamu udah berhasil di-upload ke server awan. Silakan pilih linknya dan salin pakai tombol di bawah ya kak! ✨`
    Rust: (tidak ada padanan)
  - JS: `[tourl] gagal download media yang di-reply: ␀`
    Rust: (tidak ada padanan)
  - JS: `[tourl] gagal download media: ␀`
    Rust: (tidak ada padanan)
  - JS: `[tourl] gagal kirim card paste: ␀`
    Rust: (tidak ada padanan)
  - JS: `[tourl] gagal kirim card: ␀`
    Rust: (tidak ada padanan)
  - JS: `[tourl] paste ␀ gagal: ␀`
    Rust: (tidak ada padanan)
  - JS: `content-type`
    Rust: (tidak ada padanan)
  - JS: `copy_␀`
    Rust: (tidak ada padanan)
  - JS: `file.␀`
    Rust: (tidak ada padanan)
  - JS: `jadi percuma dicoba. Compres dulu ya, kak 🙏`
    Rust: (tidak ada padanan)
  - JS: `options.upload is not a`
    Rust: (tidak ada padanan)
  - JS: `∞ Permanen`
    Rust: (tidak ada padanan)
  - JS: `⏳ *Expired :* ∞ Permanen`
    Rust: (tidak ada padanan)
  - JS: `⏳ *Expired :* ␀`
    Rust: (tidak ada padanan)
  - JS: `␀ balasan bukan URL`
    Rust: (tidak ada padanan)
  - JS: `␀ gagal`
    Rust: (tidak ada padanan)
  - JS: `☁️ *Server :* ␀`
    Rust: (tidak ada padanan)
  - JS: `⚠️ Kak, tolong reply ke file (gambar/video/audio/berkas) ya!`
    Rust: (tidak ada padanan)
  - JS: `⚠️ _Fyi kak, ada yang gagal di server: ␀_`
    Rust: (tidak ada padanan)
  - JS: `❌ *Kak, filenya kelewat besar!*`
    Rust: (tidak ada padanan)
  - JS: `❌ Aduh kak, paste-nya gagal semua!`
    Rust: (tidak ada padanan)
  - JS: `❌ Aduh kak, semuanya pada error pas upload!`
    Rust: (tidak ada padanan)
  - JS: `❌ Waduh kak, medianya nggak kebaca. Coba kirim ulang deh!`
    Rust: (tidak ada padanan)
  - JS: `👉 Atau reply media yang udah ada dengan `␀tourl``
    Rust: (tidak ada padanan)
  - JS: `👉 Kirim media dengan caption `␀tourl``
    Rust: (tidak ada padanan)
  - JS: `👉 Kirim teks setelah command (`␀tourl <teks>`) buat dapet link paste`
    Rust: (tidak ada padanan)
  - JS: `📋 *Server :* ␀`
    Rust: (tidak ada padanan)
  - JS: `📋 Salin Link ␀`
    Rust: (tidak ada padanan)
  - JS: `📝 *TEXT UPLOAD BERHASIL!* 📝`
    Rust: (tidak ada padanan)
  - JS: `📤 *MEDIA UPLOADER* 📤`
    Rust: (tidak ada padanan)
  - JS: `🔗 *Link :*`
    Rust: (tidak ada padanan)
  - JS: `🚀 *UPLOAD BERHASIL!* 🚀`
    Rust: (tidak ada padanan)

### tovideo (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/tovideo.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/tovideo.rs`

  - JS: `> 1. Kirim sticker + caption `␀tovideo``
    Rust: `> 1. Kirim sticker + caption `.tovideo``
  - JS: `> 2. Reply sticker dengan `␀tovideo``
    Rust: `> 2. Reply sticker dengan `.tovideo``
  - JS: `> Sticker tidak bisa dikonversi (tidak animated)`
    Rust: `> Sticker tidak bisa dikonversi.`
  - JS: `Conversion timeout`
    Rust: (tidak ada padanan)
  - JS: `FFmpeg:`
    Rust: `> FFmpeg: ␀`
  - JS: `Output file empty`
    Rust: (tidak ada padanan)
  - JS: `[ToVideo] Error:`
    Rust: (tidak ada padanan)
  - JS: `fluent-ffmpeg`
    Rust: (tidak ada padanan)
  - JS: `gif_␀.gif`
    Rust: (tidak ada padanan)
  - JS: `vid_␀.mp4`
    Rust: (tidak ada padanan)

### tovn (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/tovn.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/tovn.rs`

  - JS: `"␀"`
    Rust: (tidak ada padanan)
  - JS: `-ac 1`
    Rust: `-ac`
  - JS: `-application voip`
    Rust: `-application`
  - JS: `-ar 48000`
    Rust: (tidak ada padanan)
  - JS: `-b:a 128k`
    Rust: (tidak ada padanan)
  - JS: `-c:a libopus`
    Rust: (tidak ada padanan)
  - JS: `-i "␀"`
    Rust: (tidak ada padanan)
  - JS: `> 1. Kirim audio/video + caption `␀tovn``
    Rust: `> 1. Kirim audio/video + caption `.tovn``
  - JS: `> 2. Reply audio/video dengan `␀tovn``
    Rust: `> 2. Reply audio/video dengan `.tovn``
  - JS: `> Terjadi kesalahan saat memproses.`
    Rust: (tidak ada padanan)
  - JS: `> _␀_`
    Rust: (tidak ada padanan)
  - JS: `ffmpeg -y`
    Rust: (tidak ada padanan)
  - JS: `input_␀.␀`
    Rust: `␀/input_␀.␀`
  - JS: `vn_␀.ogg`
    Rust: `␀/vn_␀.ogg`

### transkrip (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/transkrip.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/transkrip.rs`

  - JS: `ffmpeg -y -i "␀" -ar 16000 -ac 1 -f wav "␀"`
    Rust: (tidak ada padanan)
  - JS: `form-data`
    Rust: (tidak ada padanan)
  - JS: `stt_␀.ogg`
    Rust: `␀/stt_␀.ogg`
  - JS: `stt_␀.wav`
    Rust: `␀/stt_␀.wav`

### txt2qr (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/txt2qr.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/txt2qr.rs`

  - JS: `> ␀␀`
    Rust: `> ␀`

### videotranscribe (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/videotranscribe.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/videotranscribe.rs`

  - JS: `.video-transcribe https://youtu.be/dQw4w9WgXcQ`
    Rust: `.video-transcribe https://youtu.be/dQw4w9WgXcQ id`
  - JS: `Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Mobile Safari/537.36`
    Rust: (tidak ada padanan)
  - JS: `Response bukan JSON:`
    Rust: `❌ *Gagal:* Response bukan JSON`
  - JS: `URL kosong / tidak valid`
    Rust: `❌ *Gagal:* URL kosong / tidak valid`
  - JS: `[VideoTranscribe]`
    Rust: `video-transcribe`
  - JS: ````␀video-transcribe <url_video> [bahasa]````
    Rust: ``␀video-transcribe <url_video> [bahasa]``
  - JS: `content-type`
    Rust: (tidak ada padanan)
  - JS: `user-agent`
    Rust: (tidak ada padanan)
  - JS: `␀:␀`
    Rust: (tidak ada padanan)

### wink (tools)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tools/wink.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tools/wink.rs`

  - JS: `> Kirim/reply video lalu caption `␀wink``
    Rust: `> Kirim/reply video lalu caption `.wink``
  - JS: `> Maksimal *4 menit*. Kalau prosesnya tidak selesai,command ini`
    Rust: (tidak ada padanan)
  - JS: `> Video lagi diproses AI Wink biar jadi *Ultra HD* ✨`
    Rust: (tidak ada padanan)
  - JS: `Maksimal *50MB*, video kamu *␀MB*.`
    Rust: (tidak ada padanan)
  - JS: `WINK-HD-␀.mp4`
    Rust: (tidak ada padanan)
  - JS: `Wink selesai tanpa URL hasil`
    Rust: (tidak ada padanan)
  - JS: `[wink] gagal: ␀`
    Rust: (tidak ada padanan)
  - JS: `batal — tidak akan dikasih video setengah jalan.`
    Rust: (tidak ada padanan)
  - JS: `proses tidak selesai`
    Rust: `tidak selesai`
  - JS: `prosesnya dibatalkan — bukan dikasih video asal._`
    Rust: (tidak ada padanan)
  - JS: `server error`
    Rust: (tidak ada padanan)
  - JS: `wink-␀.mp4`
    Rust: `wink-video.mp4`
  - JS: `⏱️ *ᴘʀᴏsᴇs ᴋᴇʜᴀʟᴜᴜ ᴛɪᴍᴇ ɴᴏᴜᴛ*`
    Rust: `⏱️ *ᴘʀᴏsᴇs ᴋᴇʜᴀʟᴜ ᴛɪᴍᴇ ɴᴏᴜᴛ*`
  - JS: `⚠️ _Fitur Premium, proses estimasi 1-4 menit. Kalau lewat 4 menit`
    Rust: `⚠️ _Fitur Premium, proses estimasi 1-4 menit._`
  - JS: `🎬 *ᴘʀᴏsᴇs ᴡɪɴᴋ ᴇɴʜᴀɴᴄᴇ ᴅɪᴍᴜʟᴀɪ*`
    Rust: (tidak ada padanan)

### ttselon (tts)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/tts/ttselon.js`
- Rust: `/workspaces/oktz-rs/plugin/src/tts/ttselon.rs`

  - JS: `FFmpeg error`
    Rust: (tidak ada padanan)
  - JS: `audio/ogg; codecs=opus`
    Rust: (tidak ada padanan)
  - JS: `tts_␀.ogg`
    Rust: (tidak ada padanan)
  - JS: `tts_␀.wav`
    Rust: (tidak ada padanan)

### birthday (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/birthday.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/birthday.rs`

  - JS: `> Gunakan: ␀setbirthday DD-MM`
    Rust: `␀> Gunakan: ␀setbirthday DD-MM`

### buyfitur (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/buyfitur.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/buyfitur.rs`

  - JS: `✨ *ᴘʀᴇᴍɪᴜᴍ ᴜsᴇʀ*`
    Rust: `✨ *ᴘʀᴇᴍɪᴜᴍ ᴅsᴇʀ*`

### daftar (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/daftar.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/daftar.rs`

  - JS: `")}`
    Rust: (tidak ada padanan)
  - JS: `*Reply pesan ini dengan:*`
    Rust: (tidak ada padanan)
  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Atau ketik: `␀bataldaftar``
    Rust: (tidak ada padanan)
  - JS: `> Atau: *Cewe* / *Cewek* / *Perempuan* / *P*`
    Rust: (tidak ada padanan)
  - JS: `> Balas dengan: *Cowo* / *Cowok* / *Laki-laki* / *L*`
    Rust: (tidak ada padanan)
  - JS: `> Berapa umurmu?`
    Rust: (tidak ada padanan)
  - JS: `> Bonus daftar pertama sudah pernah kamu klaim`
    Rust: (tidak ada padanan)
  - JS: `> Contoh: `17``
    Rust: (tidak ada padanan)
  - JS: `> Daftar ulang tidak mendapat reward lagi`
    Rust: (tidak ada padanan)
  - JS: `> Kamu cowo atau cewe?`
    Rust: (tidak ada padanan)
  - JS: `> Kirim nama yang benar ya.`
    Rust: (tidak ada padanan)
  - JS: `> Kirim umur yang benar ya.`
    Rust: (tidak ada padanan)
  - JS: `> Masukkan angka umur dari *1 - 100* tahun`
    Rust: (tidak ada padanan)
  - JS: `> Mulai lagi dengan: `␀daftar``
    Rust: (tidak ada padanan)
  - JS: `> Pilih gender yang benar ya.`
    Rust: (tidak ada padanan)
  - JS: `> Reply pesan terakhir bot untuk melanjutkan`
    Rust: (tidak ada padanan)
  - JS: `> Reply: `revisi nama`, `revisi umur`, atau `revisi gender``
    Rust: (tidak ada padanan)
  - JS: `> Reply: `ya`, `revisi nama`, `revisi umur`, `revisi gender`, atau `batal``
    Rust: (tidak ada padanan)
  - JS: `> Siapa nama kamu?`
    Rust: (tidak ada padanan)
  - JS: `> Untuk batal: reply `batal` atau ketik `␀bataldaftar``
    Rust: (tidak ada padanan)
  - JS: `> Untuk unregister: `␀unreg``
    Rust: (tidak ada padanan)
  - JS: `> `batal` untuk batalkan`
    Rust: (tidak ada padanan)
  - JS: `> `revisi gender` untuk ubah gender`
    Rust: (tidak ada padanan)
  - JS: `> `revisi nama` untuk ubah nama`
    Rust: (tidak ada padanan)
  - JS: `> `revisi umur` untuk ubah umur`
    Rust: (tidak ada padanan)
  - JS: `> `ya` untuk simpan`
    Rust: (tidak ada padanan)
  - JS: `> ⚡ +␀ Energi`
    Rust: (tidak ada padanan)
  - JS: `> ⭐ +␀ EXP`
    Rust: (tidak ada padanan)
  - JS: `> 💰 +␀ Koin`
    Rust: (tidak ada padanan)
  - JS: `Apakah data berikut sudah benar?`
    Rust: (tidak ada padanan)
  - JS: `Halo *␀* 👋`
    Rust: (tidak ada padanan)
  - JS: `Laki-laki`
    Rust: (tidak ada padanan)
  - JS: `Selamat datang, *␀*!`
    Rust: (tidak ada padanan)
  - JS: `edit gender`
    Rust: (tidak ada padanan)
  - JS: `edit nama`
    Rust: (tidak ada padanan)
  - JS: `edit umur`
    Rust: (tidak ada padanan)
  - JS: `revisi gender`
    Rust: (tidak ada padanan)
  - JS: `revisi jk`
    Rust: (tidak ada padanan)
  - JS: `revisi nama`
    Rust: (tidak ada padanan)
  - JS: `revisi umur`
    Rust: (tidak ada padanan)
  - JS: `revisi usia`
    Rust: (tidak ada padanan)
  - JS: `ubah gender`
    Rust: (tidak ada padanan)
  - JS: `ubah nama`
    Rust: (tidak ada padanan)
  - JS: `ubah umur`
    Rust: (tidak ada padanan)
  - JS: `␀`).join("`
    Rust: (tidak ada padanan)
  - JS: `┃ Bonus daftar pertama sudah pernah diambil`
    Rust: (tidak ada padanan)
  - JS: `┃ Bonus daftar sudah pernah diklaim`
    Rust: (tidak ada padanan)
  - JS: `┃ Daftar ulang tidak mendapat reward lagi`
    Rust: (tidak ada padanan)
  - JS: `┃ Tidak ada reward tambahan kali ini`
    Rust: (tidak ada padanan)
  - JS: `┃ ⚡ +␀ Energi`
    Rust: (tidak ada padanan)
  - JS: `┃ ⭐ +␀ EXP`
    Rust: (tidak ada padanan)
  - JS: `┃ � *Cewe* / *Cewek* / *Perempuan* / *P*`
    Rust: (tidak ada padanan)
  - JS: `┃ � *Cowo* / *Cowok* / *Laki-laki* / *L*`
    Rust: (tidak ada padanan)
  - JS: `┃ 🎂 Umur: *␀ tahun` : "-"}*`
    Rust: (tidak ada padanan)
  - JS: `┃ 👤 Gender: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 👧 *Cewe* / *Cewek* / *Perempuan* / *P*`
    Rust: (tidak ada padanan)
  - JS: `┃ 💰 +␀ Koin`
    Rust: (tidak ada padanan)
  - JS: `┃ 📛 Nama: *␀*`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 🎁 *ʀᴇᴡᴀʀᴅs* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 🎁 *ʙᴏɴᴜs* 」`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📋 *ᴅᴀᴛᴀ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴘᴇʀᴛᴀɴʏᴀᴀɴ 4/4*`
    Rust: (tidak ada padanan)
  - JS: `✅ Kamu sudah terdaftar!`
    Rust: (tidak ada padanan)
  - JS: `✨ Dengan daftar, data akun kamu jadi lebih aman dan pengalaman pakai bot jadi lebih lengkap.`
    Rust: (tidak ada padanan)
  - JS: `❌ Balasan tidak valid!`
    Rust: (tidak ada padanan)
  - JS: `❌ Gender tidak valid!`
    Rust: (tidak ada padanan)
  - JS: `❌ Nama harus 2-30 karakter!`
    Rust: (tidak ada padanan)
  - JS: `❌ Pendaftaran dibatalkan.`
    Rust: (tidak ada padanan)
  - JS: `❌ Revisi belum spesifik!`
    Rust: (tidak ada padanan)
  - JS: `❌ Umur tidak valid!`
    Rust: (tidak ada padanan)
  - JS: `� *ᴘᴇʀᴛᴀɴʏᴀᴀɴ 3/4*`
    Rust: (tidak ada padanan)
  - JS: `🌟 *Manfaat Daftar*`
    Rust: (tidak ada padanan)
  - JS: `🎁 *Bonus Daftar Pertama*`
    Rust: (tidak ada padanan)
  - JS: `🎁 *Status Bonus*`
    Rust: (tidak ada padanan)
  - JS: `🎂 *ʀᴇᴠɪsɪ ᴜᴍᴜʀ*`
    Rust: (tidak ada padanan)
  - JS: `🎂 *ᴘᴇʀᴛᴀɴʏᴀᴀɴ 2/4*`
    Rust: (tidak ada padanan)
  - JS: `🎉 *ᴘᴇɴᴅᴀꜰᴛᴀʀᴀɴ ʙᴇʀʜᴀsɪʟ!*`
    Rust: (tidak ada padanan)
  - JS: `👋 *sᴇʟᴀᴍᴀᴛ ᴅᴀᴛᴀɴɢ ᴅɪ ᴍᴇɴᴜ ᴅᴀꜰᴛᴀʀ*`
    Rust: (tidak ada padanan)
  - JS: `👤 *ʀᴇᴠɪsɪ ɢᴇɴᴅᴇʀ*`
    Rust: (tidak ada padanan)
  - JS: `📌 *Wajib reply pesan ini ya*`
    Rust: (tidak ada padanan)
  - JS: `📌 Umur hanya boleh *1 - 100* tahun`
    Rust: (tidak ada padanan)
  - JS: `📛 *ʀᴇᴠɪsɪ ɴᴀᴍᴀ*`
    Rust: (tidak ada padanan)
  - JS: `📝 *Pertanyaan 1/4*`
    Rust: (tidak ada padanan)
  - JS: `📝 Masih ada sesi pendaftaran aktif!`
    Rust: (tidak ada padanan)
  - JS: `📩 Reply pesan ini dengan angka umur baru kamu`
    Rust: (tidak ada padanan)
  - JS: `📩 Reply pesan ini dengan angka umur kamu`
    Rust: (tidak ada padanan)
  - JS: `📩 Reply pesan ini dengan jawabanmu`
    Rust: (tidak ada padanan)
  - JS: `📩 Reply pesan ini dengan nama baru kamu`
    Rust: (tidak ada padanan)
  - JS: `🔓 Setelah daftar kamu bisa mengakses semua command`
    Rust: (tidak ada padanan)
  - JS: `🗂️ Data akun kamu tersimpan lebih rapi`
    Rust: (tidak ada padanan)
  - JS: `🚀 Sekarang kamu sudah siap menggunakan bot!`
    Rust: (tidak ada padanan)
  - JS: `🛠️ Kalau ada yang salah, kamu bisa revisi per bagian.`
    Rust: (tidak ada padanan)

### daily (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/daily.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/daily.rs`

  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)

### level (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/level.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/level.rs`

  - JS: `> Next level: *␀ exp* lagi!`
    Rust: `> Next level: *␀* exp* lagi!`
  - JS: `⚔️ Legend`
    Rust: (tidak ada padanan)
  - JS: `⭐ Elite`
    Rust: (tidak ada padanan)
  - JS: `🎖️ Master`
    Rust: (tidak ada padanan)
  - JS: `🐉 Mythic`
    Rust: (tidak ada padanan)
  - JS: `💜 Epic`
    Rust: (tidak ada padanan)
  - JS: `💪 Grandmaster`
    Rust: (tidak ada padanan)
  - JS: `🛡️ Warrior`
    Rust: `┃ 🛡️ Warrior`

### levelup (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/levelup.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/levelup.rs`

  - JS: `┃ > `.levelup off` - Nonaktifkan`
    Rust: `┃ > `␀levelup off` - Nonaktifkan`
  - JS: `┃ > `.levelup on` - Aktifkan`
    Rust: `┃ > `␀levelup on` - Aktifkan`
  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)

### setbirthday (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/setbirthday.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/setbirthday.rs`

  - JS: `╰┈┈┈┈┈┈┈┈⬡`
    Rust: (tidak ada padanan)

### unreg (user)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/user/unreg.js`
- Rust: `/workspaces/oktz-rs/plugin/src/user/unreg.rs`

  - JS: `120363400911374213@newsletter`
    Rust: (tidak ada padanan)
  - JS: `> Daftar dengan `␀daftar``
    Rust: (tidak ada padanan)
  - JS: `> Untuk daftar ulang: `␀daftar``
    Rust: (tidak ada padanan)
  - JS: `Data pendaftaran kamu sudah dihapus.`
    Rust: (tidak ada padanan)
  - JS: `✅ *ᴜɴʀᴇɢɪsᴛᴇʀ ʙᴇʀʜᴀsɪʟ!*`
    Rust: (tidak ada padanan)
  - JS: `❌ Kamu belum terdaftar!`
    Rust: `belum terdaftar`

### inspect (utility)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/utility/inspect.js`
- Rust: `/workspaces/oktz-rs/plugin/src/utility/inspect.rs`

  - JS: `┃ ✅ ᴊᴏɪɴ ᴀᴘᴘʀᴏᴠᴀʟ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ ✅ ᴠᴇʀɪꜰɪᴄᴀᴛɪᴏɴ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ ➕ ᴍᴇᴍʙᴇʀ ᴀᴅᴅ ᴍᴏᴅᴇ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🆔 ɪᴅ: `␀``
    Rust: (tidak ada padanan)
  - JS: `┃ 🏘️ ɪs ᴄᴏᴍᴍᴜɴɪᴛʏ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 👑 ᴄʀᴇᴀᴛᴏʀ: @␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 👥 sᴜʙsᴄʀɪʙᴇʀs: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 👥 ᴘᴀʀᴛɪᴄɪᴘᴀɴᴛs: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📅 ᴄʀᴇᴀᴛᴇᴅ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📌 sᴛᴀᴛᴇ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📝 ɴᴀᴍᴇ: *␀*`
    Rust: (tidak ada padanan)
  - JS: `┃ 📢 ᴀɴɴᴏᴜɴᴄᴇ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 📣 ᴄᴏᴍᴍᴜɴɪᴛʏ ᴀɴɴᴏᴜɴᴄᴇ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔒 ʀᴇsᴛʀɪᴄᴛ: ␀`
    Rust: (tidak ada padanan)
  - JS: `┃ 🔗 ʟɪɴᴋᴇᴅ ᴘᴀʀᴇɴᴛ: ␀`
    Rust: (tidak ada padanan)
  - JS: `├ @␀ [␀]`
    Rust: (tidak ada padanan)
  - JS: `╭┈┈⬡「 📊 *ᴅᴇᴛᴀɪʟ* 」`
    Rust: (tidak ada padanan)
  - JS: `╰┈┈⬡`
    Rust: (tidak ada padanan)
  - JS: `❌ Bot di-kick dari grup tersebut!`
    Rust: (tidak ada padanan)
  - JS: `❌ Grup/Saluran tidak ditemukan!`
    Rust: (tidak ada padanan)
  - JS: `❌ Hanya support URL Grup atau Saluran WhatsApp!`
    Rust: (tidak ada padanan)
  - JS: `❌ URL grup telah di-reset!`
    Rust: (tidak ada padanan)
  - JS: `👑 *ᴀᴅᴍɪɴs:*`
    Rust: (tidak ada padanan)
  - JS: `📋 *ɪɴꜰᴏʀᴍᴀᴛɪᴏɴ ɢʀᴏᴜᴘ*`
    Rust: (tidak ada padanan)
  - JS: `📝 *ᴅᴇsᴄʀɪᴘᴛɪᴏɴ:*`
    Rust: (tidak ada padanan)
  - JS: `📺 *ɪɴꜰᴏʀᴍᴀᴛɪᴏɴ ᴄʜᴀɴɴᴇʟ*`
    Rust: (tidak ada padanan)

### notifmakan (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/utility/notifmakan.js`
- Rust: `/workspaces/oktz-rs/plugin/src/utility/notifmakan.rs`

  - JS: `> Contoh: `08.00,13.00,20.00``
    Rust: `> Contoh: `07.00,12.30,19.00``
  - JS: `> Contoh: `␀notifmakan edit 08.00,13.00,20.00``
    Rust: `> Contoh: `␀notifmakan on 07.00,12.00,19.00``
  - JS: `⏰ *Jadwal:* ␀* WIB`).join(', ')}`
    Rust: (tidak ada padanan)
  - JS: `⏰ Jadwal: ␀*`).join(', ')} WIB`
    Rust: (tidak ada padanan)
  - JS: `❌ *Masukkan jadwal baru!*`
    Rust: `❌ *Masukkan jadwal makan!*`

### notiftidur (group)
- JS: `/workspaces/oktz-rs/oktz-md/plugins/utility/notiftidur.js`
- Rust: `/workspaces/oktz-rs/plugin/src/utility/notiftidur.rs`

  - JS: `> Contoh: `23.00` atau `22.30``
    Rust: `> Contoh: `22.00` atau `23.30``
  - JS: `> Contoh: `␀notiftidur edit 23.00``
    Rust: `> Contoh: `␀notiftidur on 22.00``
  - JS: `⏰ *Jadwal:* ␀* WIB`).join(', ')}`
    Rust: (tidak ada padanan)
  - JS: `⏰ Jadwal: ␀*`).join(', ')} WIB`
    Rust: (tidak ada padanan)
  - JS: `⚪ Belum diatur`
    Rust: (tidak ada padanan)
  - JS: `✅ Aktif`
    Rust: (tidak ada padanan)
  - JS: `❌ *Masukkan jadwal baru!*`
    Rust: `❌ *Masukkan jadwal tidur!*`
  - JS: `❌ Nonaktif`
    Rust: (tidak ada padanan)

