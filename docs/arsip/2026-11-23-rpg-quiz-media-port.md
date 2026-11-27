# Port RPG + Quiz Games + Fix Media/Gacha — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix gacha waifu/husbu media failure + `.ssweb` flaky delivery, port 22 quiz games (tebakkata dkk), port per-message EXP (+15) with level-up notification, and add `.dor` alias for `.kick`.

**Architecture:** (1) Media fix: pipeline (`lib/oktz-wa/pipeline.rs`) gacha kartu — `MediaSource::Url` di header kartu kehilangan `Referer` + tidak bawa timeout tegas; ganti dengan byte pre-fetch + fallback di plugin, persis `getHusbuImage` JS. (2) Quiz games: satu engine umum baru `plugin/src/game/quiz.rs` (port `FotoGames` + `game-data.js`) + 22 file plugin tipis yang memakai `include_str!` data JSON dari `data/`. (3) EXP per-pesan: satu blok di `Pipeline::process` sebelum route (paritas `handler.js:1120`), notifikasi level-up teks. (4) `.dor`: satu string di alias kick.

**Tech Stack:** Rust workspace (`oktz`, `oktz-plugins`, `oktz-wa`, `oktz-plugin-api`), `linkme` registry, `serde_json`, `include_str!`.

**Spec:** hasil investigasi session ini — registry `./target/debug/oktz plugins` (22 quiz game tidak ada), `docs/paritas/paritas.json`, JS referensi `oktz-md/src/lib/games.js`, `game-data.js`, `level.js`, `context.js`, `oktz-md/plugins/fun/gachahusbu.js` (`getHusbuImage`), `oktz-md/plugins/group/kick.js`.

## Global Constraints

- Komentar hanya yang penting (non-obvious, invariant, ponytail). Bahasa Indonesia. Tidak ada komentar TODO/debug.
- `cargo fmt` + `cargo clippy --release --all-targets` + `cargo check` sebelum commit.
- `Result`/`?`, tidak ada `unwrap()`/`expect()` di jalur produksi (test boleh).
- Dependency baru dilarang — `rand`, `serde_json`, `linkme` sudah ada.
- Target runtime: binary statis Linux x86_64 (panel Pterodactyl). Tidak menambah dependency native berat.
- Hemat RAM: jangan clone besar; data quiz pakai `include_str!` + parse sekali ke `OnceLock` (pola `family100.rs:37-41`).
- JS adalah acuan mutlak untuk output string; byte-per-byte bila memungkinkan.
- Nama sesi game: `gameSessions` (pola `family100.rs:26`), map global `Ctx::sessions`.
- `cargo build --release` penuh di GitHub Actions, bukan lokal (4 vCPU/8GB). Lokal cukup `cargo check`/`cargo test -p oktz-plugins <filter>`.

## Review Focus

1. **Fetch gambar header kartu gagal** (Pinterest mati/lambat): user harus tetap dapat kartu/teks, bukan error. Test: task 1.
2. **Sesi quiz di chat berbeda tidak saling ganggu** (map per-chat, satu sesi aktif per chat). Test: task 2.
3. **EXP tidak dobel** pada satu pesan (dedup + sekali per `process`), dan tidak berlaku untuk pesan tanpa body. Test: task 3.
4. **Level-up notification dihormati setting**: `settings.levelupNotif === false` → tidak kirim notifikasi (JS `level.js:146`), tapi stat tetap ditulis. Test: task 3.
5. **Alias `.dor` tidak menimpa command lain** yang namanya `dor` (registry `find`: nama > commands > alias). Test: task 4.

---

### Task 1: Fix gacha media + ssweb delivery

**Files:**
- Modify: `plugin/src/fun/gachahusbu.rs` (fungsi `gacha_husbu`, `kartu_husbu`, konstanta `IMAGE_FALLBACK`)
- Modify: `plugin/src/fun/gachawaifu.rs` (fungsi `gacha_waifu`, `gambar_waifu`)
- Test: mod `#[cfg(test)]` di kedua berkas itu

**Interfaces:**
- Consumes: `oktz_scraper::rinn::pinterest(&dyn HttpGet, q, limit) -> Result<Vec<String>, HttpError>` (sudah ada, `rinn.rs:171`), `ctx.http` (`Arc<dyn HttpGet>`), `MediaSource::bytes`.
- Produces: `fn gambar_bytes(ctx: &Ctx, keyword: &str) -> Vec<u8>` di masing-masing berkas (pre-fetch byte gambar; kosong = gagal) — kartu header hanya diberi media kalau byte > 100.

Root cause: kartu hasil gacha menempel URL pinterest mentah ke `CardHeader::image(MediaSource::url(url))`; pipeline `load_media` fetch URL itu tanpa Referer dan gagal (Pinterest menolak hotlink tanpa Referer/UA) → `PipelineError::Media` → seluruh balasan dibatalkan. JS membandingkan: `getHusbuImage` ambil URL dari API, lalu `getBuffer(url)` dengan `Referer: https://www.pinterest.com/` — kalau gagal, JANGAN buat image header (`gachahusbu.js:80-90` komentar "Buffer kosong => JANGAN buat imageMessage").

- [ ] **Step 1: Write failing test** — di `gachahusbu.rs` tests: kartu status (`status_husbu` dengan user punya `husbu.imageUrl` = URL mati) TETAP menghasilkan `Reply::Interactive` (bukan error), dan header TANPA media ketika byte fetch gagal. Stub `HttpGet` baru `HttpBytesStub` yang gagal untuk URL gambar (pola stub ssweb.rs:108).
- [ ] **Step 2: Run** `cargo test -p oktz-plugins gachahusbu` → FAIL.
- [ ] **Step 3: Implement** — di `gacha_husbu` dan `status_husbu`: sebelum membangun kartu, pre-fetch byte: `let img = gambar_bytes(&ctx, keyword)` (fetch URL dengan header `Referer: https://www.pinterest.com/` via `HttpCall` + `with_timeout(15_000)`, fallback `IMAGE_FALLBACK` URL, fallback kedua = kosong). Header hanya `.image(MediaSource::bytes(img))` kalau `img.len() > 100`; kalau kosong, header tanpa media (JS `sendHusbuMessage:83` `media = null`). `imageUrl` di store tetap simpan URL (paritas DB JS).
- [ ] **Step 4: Run** `cargo test -p oktz-plugins gachahusbu` → PASS.
- [ ] **Step 5: Sama untuk `gachawaifu.rs`** — fungsi `gambar_waifu` sudah ada tapi mengembalikan URL; tambah `gambar_bytes` (pre-fetch dengan Referer), ubah `kartu_waifu` memakai byte. Test: kartu hasil gacha dengan fetch gagal → tetap `Reply::Interactive` tanpa header media.
- [ ] **Step 6: `.ssweb` hardening** — `plugin/src/tools/ssweb.rs:75-79`: `Reply::image(MediaSource::bytes(bytes))` sudah byte; yang perlu: cek `res.status` (thum.io bisa balas 403/500 → body HTML bukan gambar) dan mime: kalau body bukan gambar (cek bukan PNG/JPEG magic bytes atau status >= 400), balas `error_template` + ☢. Test baru: status 403 → teks error, bukan `Reply::image`.
- [ ] **Step 7: fmt + clippy + check**: `cargo fmt && cargo clippy --release -p oktz-plugins 2>&1 | tail -5 && cargo check -p oktz-plugins 2>&1 | tail -3`.
- [ ] **Step 8: Commit** `fix(plugin): gacha kartu header pre-fetch byte + Referer, ssweb cek status`

### Task 2: Port 22 quiz games

**Files:**
- Create: `plugin/src/game/quiz.rs` — engine umum (port `FotoGames.createHandler` + `answerHandler` + helper `game-data.js`)
- Create: 22 file `plugin/src/game/<nama>.rs` (tebakkata, tebaknegara, asahotak, caklontong, kataacak, riddle, siapakahaku, susunkata, tebakbendera, tebakdrakor, tebakepep, tebakfilm, tebakgambar, tebakhewan, tebakjkt48, tebakkalimat, tebakkimia, tebaklagu, tebaklirik, tebakmakanan, tebakprofesi, tebaktebakan, tekateki)
- Copy data: `oktz-md/src/data/<nama>.json` → `data/<nama>.json` (22 file, ~800K total — tebakgambar 232K terbesar; family100 sudah ada)
- Modify: `plugin/src/game/mod.rs` (tambah 23 `pub mod`)

**Interfaces:**
- Consumes: `Ctx.session_get/set/take("gameSessions")` (pola `family100.rs`), `crate::rpg::heal::{apply_exp, rpg_map, math_random}` untuk reward EXP, `Reply::Preview` untuk kartu soal (JS `sendGamePreview` — `Reply::preview(text, url, title, desc, None)` ada di registry.rs:375), `Reply::image` untuk game has_image, `Mutation::SetUser` untuk reward koin/energi.
- Produces: `pub struct QuizGame { pub name, pub alias, pub emoji, pub title, pub description, pub timeout_ms, pub cooldown, pub has_image, pub image_field, pub question_field, pub answer_field, pub hint_count }`; `fn bank(cfg) -> &'static Vec<Value>` (OnceLock parse `include_str!`); `fn handler(cfg: &'static QuizGame, ctx: Ctx) -> Reply`; `fn answer(cfg: &'static QuizGame, ctx: Ctx) -> Option<Reply>`. Tiap file plugin tipis: konstanta cfg + `#[plugin]` wrapper + `#[answer_handler]` wrapper + tests.

Data JS: `dataFile` default `<game>.json`; tebakbendera override `tebakbendera2.json` + `answerField: name`; tebakkimia `questionField: unsur, answerField: lambang`; tebakgambar `questionField: null, hasImage: true, timeout: 90000, hintCount: 3`; tebakdrakor/tebakmakanan punya field `deskripsi` sebagai soal tambahan (JS render pakai `questionField` saja — cek JS `handler`: caption `> ${question[cfg.questionField]}` hanya kalau `questionField` truthy). Alias per game dari daftar register di atas.

- [ ] **Step 1: Copy data** `cp oktz-md/src/data/{22 nama}.json data/` (verifikasi: `ls data/*.json | wc -l` = 23).
- [ ] **Step 2: Write failing test** di `quiz.rs` — `bank` parse & hitung entri (tebakkata = 30), `get_hint` (`getHint` JS: reveal 2 char pertama, sisanya `_`, spasi tetap), `normalize_answer` (`[^a-z0-9 ]` dibuang, trim), `check_answer_advanced` (correct exact / substring ≥80% → correct 0.95 / similarity ≥0.85 correct / ≥0.6 close / else wrong — port `getSimilarity` Levenshtein JS), `is_surrender` (nyerah/skip/lewat/gak tau dkk), `progressive_hint` (reveal = 2 + floor(attempts/2), hitung non-space).
- [ ] **Step 3: Run** `cargo test -p oktz-plugins quiz` → FAIL.
- [ ] **Step 4: Implement engine** `quiz.rs`: `bank()` (OnceLock + serde_json), helper murni di atas, `handler()` — cek sesi aktif (sama game → kartu "ada game jalan" pakai `Reply::preview`, JS games.js:93-115), ambil soal acak, bangun teks kartu (JS games.js:158-170: emoji + title + soal di code block + hint + waktu + hadiah + "Jawab langsung atau ketik nyerah"), simpan sesi `ctx.session_set("gameSessions", ...)` bentuk `{chatId, gameType, question, startTime, endTime, attempts}` (paritas JS `createSession`), kembalikan `Reply::preview(...)` (teks) atau `Reply::image` (has_image; thumbnail dari `MediaSource::url(question.img)`, JS fetch buffer — ponytail: URL langsung, pipeline sudah resolve).
- [ ] **Step 5: Implement `answer()`** — port `games.js:186-296`: `None` kalau bukan sesi game ini; `isSurrender` → endSession + teks menyerah + mention; bukan reply pesan game (JS `isReplyToGame`: `m.quoted.id === session.messageKey.id || fromMe` — Rust: `ctx.quoted().is_some_and(|q| q.from_me || id match)`; ponytail: id pesan bot terkirim tidak terekam di Ctx — pakai `fromMe` saja, sama dengan `family100.rs`); jawaban benar → reward `getRandomReward()` (limit 3-8, koin 500-2000, exp 1000-3000) + bonus kilat (≤4s: +1 limit +500 koin +50 exp + praise acak dari 10 FAST_ANSWER_PRAISES) → `Mutation::SetUser` (energi = lama+limit, koin, exp via `apply_exp`) + teks win; close (≥60%) → reaksi 🔥 + teks "Hampir! X% mirip"; salah → reaksi ❌ + progressive hint (maks 10 attempts).
- [ ] **Step 6: Run** `cargo test -p oktz-plugins quiz` → PASS.
- [ ] **Step 7: 22 file plugin tipis** — pola per file: `const CFG: QuizGame = QuizGame { ... }; #[plugin(name=..., alias=..., category="game", cooldown=5, energi=0)] pub fn tebakkata(ctx: Ctx) -> Reply { quiz::handler(&CFG, ctx) }` + `#[answer_handler] pub fn tebakkata_answer(ctx: Ctx) -> Option<Reply> { quiz::answer(&CFG, ctx) }` + test register (pola `family100.rs` tests: `find("tebakkata")` meta cocok, alias resolve).
- [ ] **Step 8: `plugin/src/game/mod.rs`** — tambah 23 `pub mod` (quiz + 22 game).
- [ ] **Step 9: fmt + clippy + check + test penuh** `cargo test -p oktz-plugins --lib game::quiz 2>&1 | tail -3`.
- [ ] **Step 10: Commit** `feat(game): port 22 quiz game (tebakkata, tebaknegara, asahotak, ...) dari engine FotoGames JS`

### Task 3: EXP per-pesan + notifikasi level-up

**Files:**
- Modify: `lib/oktz-wa/pipeline.rs` — `Pipeline::process` (setelah dedup/UI, sebelum gate grup ~line 4700) + helper baru `fn exp_pesan(...)`
- Test: mod tests pipeline.rs

**Interfaces:**
- Consumes: `self.ctx.user_entry(&caller.sender)` (baca exp), `self.ctx.apply(&[Mutation::SetUser{..}], now_ms)` (tulis), `settings.levelupNotif` via `self.ctx.settings_entry()`, `crate::user` tidak tersedia di oktz-wa → konstanta `EXP_PER_LEVEL=10_000` + `calculate_level` lokal kecil (pola `games/ekonomi.rs:36`).
- Produces: `fn hitung_exp(old_exp: i64) -> (i64, Option<i64>)` → `(new_exp, Some(new_level) kalau naik)`; tidak mengembalikan Reply — notifikasi dikirim inline via `self.deliver`.

Paritas JS `handler.js:1120-1124`: SEMUA pesan dengan body (termasuk non-command) dapat +15 EXP, SEBELUM gate apa pun. `level.js:217-242` `addExpWithLevelCheck`: exp `-1` = unlimited (tidak berubah); clamp `0..=9_000_000_000`; `db.setUser(sender, {exp, rpg})` — Rust `apply_set_user` merge, jadi cukup `{ "exp": new }`; kalau level naik: tulis `rpg.level/maxHealth/maxMana/maxStamina/health/mana/stamina` (JS level.js:140-147, rumus 100+(lv-1)*10 / *5 / *5); notifikasi dilewati kalau `user.settings.levelupNotif === false` (JS level.js:146).

- [ ] **Step 1: Write failing test** — pipeline test: inbound "halo" (non-command) → store user exp naik 15; exp kedua pesan menabrak batas 10_000 → `rpg.level=2` + maxHealth 110; `settings.levelupNotif=false` → tidak ada pesan level-up tapi stat tetap ditulis; exp `-1` tetap `-1`.
- [ ] **Step 2: Run** `cargo test -p oktz-wa exp_pesan` → FAIL.
- [ ] **Step 3: Implement** — di `process`, tepat setelah `ui::pesan_masuk` (paritas JS: exp ditambah sebelum gate): baca `user_entry(sender)`, `exp` lama (`as_i64` default 0), `hitung_exp`, `self.ctx.apply(&[SetUser{sender, {"exp": new, "rpg": rpg_patch}}])`; kalau level naik dan `settings.levelupNotif !== false` → bangun teks (JS level.js:172-190: `🎊 *SELAMAT @<num>!*\n\nLevel kamu bertambah N\n🥗 Level kamu sekarang *X*\n\nSekarang kamu berada di rank *role*\n\nMau cek detail level? ketik _.level_\n\nSering seringlah berinteraksi dengan bot agar level kamu bertambah!`) dengan mention sender, deliver via `self.deliver_reply`. Role: port `getRole` (Mythic/Legend/Epic/Grandmaster/Master/Elite/Warrior). ponytail: kartu canvas `generateLevelUpCard` tidak di-port — notifikasi teks; tambah kartu kalau diminta.
- [ ] **Step 4: Run** `cargo test -p oktz-wa exp_pesan` → PASS.
- [ ] **Step 5: fmt + clippy + check**: `cargo fmt && cargo clippy --release -p oktz-wa 2>&1 | tail -3 && cargo check -p oktz-wa`.
- [ ] **Step 6: Commit** `feat(rpg): exp per-pesan +15 & notifikasi level-up (port handler.js + level.js)`

### Task 4: Alias `.dor` untuk kick

**Files:**
- Modify: `plugin/src/group/kick.rs:52` — `alias = ["remove", "tendang", "dor"]`
- Test: mod tests di kick.rs

**Interfaces:**
- Consumes: registry `find` (nama > commands > alias, `registry.rs:1480`).
- Produces: `.dor @user`/`.dor` (reply) → kick.

- [ ] **Step 1: Write failing test** — di kick.rs tests: `find("dor")` resolve ke `kick`.
- [ ] **Step 2: Run** `cargo test -p oktz-plugins kick` → FAIL.
- [ ] **Step 3: Implement** — tambah `"dor"` ke alias.
- [ ] **Step 4: Run** `cargo test -p oktz-plugins kick` → PASS.
- [ ] **Step 5: Commit** `feat(group): alias .dor untuk .kick`

### Task 5: Verifikasi penuh + rilis

- [ ] **Step 1:** `cargo fmt --all`
- [ ] **Step 2:** `cargo clippy --release --all-targets 2>&1 | tail -5` → tidak ada warning baru.
- [ ] **Step 3:** `cargo test -p oktz-plugins --lib 2>&1 | tail -3` + `cargo test -p oktz-wa 2>&1 | tail -3` → semua pass (lokal: debug, CARGO_INCREMENTAL=0).
- [ ] **Step 4:** commit + push ke main.
- [ ] **Step 5:** pantau GitHub Actions (ci.yml): build release sukses, binary `oktz-linux-x64-static` ter-update di Release.
- [ ] **Step 6:** verifikasi `./oktz plugins` di CI/lokal: tebakkata dkk terdaftar (target: +23 plugin dari 792 → 815).
