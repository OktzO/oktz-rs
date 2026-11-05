# STUBS — Sisa Penyimpangan Port Rust

> Dihasilkan otomatis dari penanda `ponytail:` di `plugin/src`.
> Total: **616 catatan** di **426 file**. Tanggal: 2026-11-05.

Setiap catatan adalah penyimpangan sadar dari bot JS (`oktz-md/`) — bukan bug.
Bentuknya tiga jenis:
- **Fitur mati** — panggilan JS yang tidak punya padanan (`sock.*`, jadibot, `db.getUser` untuk user lain) → pesan "tidak tersedia" yang jujur.
- **Simplifikasi** — perilaku sama tapi jalur beda (parser HTML, satuan UTF-16, config yang tidak diekspos ke `Ctx`).
- **Pengurangan kosmetik** — reaksi kedua, kartu gambar, konfigurasi yang dilewat.

Teks setelah `ponytail:` di file sumber memuat alasan + upgrade path.


## owner — owner (117 catatan / 86 file)

### `owner/addenergi.rs` (2)

- L71: `Ctx` hanya mengisi `user_data` untuk `sender` dan `participants[i].data` untuk roster grup. JID di luar keduanya dianggap user baru — sama dengan `db.getUser(jid) || db.setUser(jid)`, yang memberi `config.energi.default`. Tambahkan `user_entry(jid)` ke `Ctx` kalau router mau membaca state user mana pun.
- L151: bisa dijawab untuk pengirim — tambahkan `owner_numbers` ke `Ctx` kalau router mau mengecek JID target.

### `owner/addexp.rs` (1)

- L87: `user` di JS adalah row TARGET, sedangkan `Ctx` hanya membawa row pengirim. Exp yang dijumlahkan dan ditulis di sini milik pengirim; tambahkan `user_entry(jid)` ke `Ctx` kalau router mau menghitung dari row target.

### `owner/addkoin.rs` (1)

- L49: `Ctx` hanya membawa row pengirim (lihat `oktz-wa` pipeline); koin user lain tidak terbaca, jadi `db.updateKoin` di sini dijumlahkan dari `0` — sama seperti user yang belum punya koin. Tambahkan `user_entry(jid)` ke `Ctx` kalau router mau membaca row sembarang.

### `owner/addlevel.rs` (1)

- L34: hanya row pengirim yang terbaca di `Ctx`, dan kartu level-up (canvas + `sock.sendMedia`) tidak ada di port ini — yang ditulis di sini persis `db.updateExp` + `db.setUser(targetJid, user)` dari JS.

### `owner/addlimitall.rs` (1)

- L86: `gb?.subject` diambil dari metadata grup yang tidak ada di `Ctx` (lihat `group/intro.rs`), jadi nama grup dari `group_data.subject` — atau teks cadangan kalau tidak ada.

### `owner/addowner.rs` (1)

- L16: saat DB baca-tulis, `config.pterodactyl`, dan resolusi LID tersedia, implementasikan ulang dan tambahkan `pub mod addowner;` di `src/owner/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `owner/addpartner.rs` (1)

- L87: store terpisah tidak terjangkau dari plugin (, lihat modul).

### `owner/addprem.rs` (1)

- L3: JS tulis state lewat `getDatabase()` (`db.data.premium`, `db.getUser`/`db.setUser`, `db.updateExp`, `db.updateKoin`, `db.save()`) serta `jadibotId`/`isJadibot` — tidak ada di `Ctx`, dan tidak ada `Mutation` untuk array `db.data.premium`. Oleh karena itu hanya cabang validasi + teks help yang di-port; operasi tulis (add/del/list jadibot maupun reguler) diakhiri dengan `error_template` dan reaksi `☢`, pola yang sama seperti `swm.rs` / `delpanel.rs`. Saat store premium + jadibot tersedia di `Ctx`, tambahkan `pub mod addprem;` di `src/owner/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `owner/addsewa.rs` (3)

- L9: tabel `db.db.data.sewa` adalah tabel top-level, bukan milik user/grup — dipetakan ke key global `sewa` di `settings`, sama seperti `owner/sewabot.rs` dan `owner/listsewa.rs`. ponytail: `sock.sendText(groupId,...)` (pesan perkenalan ke grup, `addsewa.js:184-191`) tidak punya padanan: `Reply` hanya sampai ke `caller.chat`. Teks balasan tetap utuh karena `sendText` dibungkus `try {} catch {}` yang tidak mengubah teks. ponytail: cek "bot sudah ada di grup" (`tryJoinGroup`, `:96-101`) butuh JID bot sendiri yang tidak ada di `Ctx`; jadi `terima_undangan` selalu dicoba dan hasilnya dilaporkan apa adanya.
- L13: `sock.sendText(groupId,...)` (pesan perkenalan ke grup, `addsewa.js:184-191`) tidak punya padanan: `Reply` hanya sampai ke `caller.chat`. Teks balasan tetap utuh karena `sendText` dibungkus `try {} catch {}` yang tidak mengubah teks. ponytail: cek "bot sudah ada di grup" (`tryJoinGroup`, `:96-101`) butuh JID bot sendiri yang tidak ada di `Ctx`; jadi `terima_undangan` selalu dicoba dan hasilnya dilaporkan apa adanya.
- L18: cek "bot sudah ada di grup" (`tryJoinGroup`, `:96-101`) butuh JID bot sendiri yang tidak ada di `Ctx`; jadi `terima_undangan` selalu dicoba dan hasilnya dilaporkan apa adanya.

### `owner/akses.rs` (1)

- L3: `db.getUser` / `db.setUser` dipindah ke `ctx.user_field` + `Mutation::SetUser`. Library `ms` untuk durasi tidak ada di Rust; parser durasi sederhana (`1h`, `1d`, `30d`, `1y`, `permanent`/`perm`) ditulis sendiri. `sock.parseMention` dan daftar interaktif `delakses` tidak ada di `Ctx`; del tanpa command spesifik menampilkan teks daftar akses aktif. Reaksi JS (`m.reply` tanpa reaksi eksplisit di akses) tidak dibawa. Catatan: baris `pub mod akses;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/anticall.rs` (1)

- L3: status tanpa argumen membaca `config.features.antiCall` (`config.js`, `false`) sebagai fallback terakhir. `Ctx` tidak membawa config, jadi nilainya ikut `config.js` yang di-hardcode — persis seperti `panel/cpanel.rs` melakukan untuk `config.pterodactyl.*`. Begitu `config.js` diisi, baris ini ikut stale sampai router menyediakannya lewat `Ctx`.

### `owner/autobackup.rs` (1)

- L3: modul `src/lib/auto-backup.js` TIDAK di-port. Ia menyimpan state di berkas sendiri, menjadwalkan timer, dan mengirim zip lewat `sock` — tidak ada padanannya di `Ctx`, dan mengarang tempatnya akan membuat `.autobackup status` melaporkan angka palsu. Yang dipindah: parsing aksi, validasi interval (`parseInterval`, murni dan punya batas angka), serta seluruh teks statis. Status/jadwal/win/lokal yang butuh modul itu berhenti di balasan "tidak tersedia".

### `owner/autobio.rs` (1)

- L4: scheduler `startAutoBioChecker` (yang benar-benar memanggil `sock.updateProfileStatus`) tidak ikut — plugin ini hanya menulis tiga setting yang dibaca scheduler itu. Tambahkan scheduler bio ke router kalau capability-nya tersedia.

### `owner/autojoingc.rs` (1)

- L3: hanya `handler` yang di-port. `autoJoinDetector` di JS ikut diekspor dan dipakai middleware untuk memanggil `sock.groupAcceptInvite` — butuh `sock`, jadi tidak ada di sini.

### `owner/autoread.rs` (1)

- L3: status tanpa argumen membaca `config.features.autoRead` (`config.js`, `true`) sebagai fallback terakhir. `Ctx` tidak membawa config, jadi nilainya ikut `config.js` yang di-hardcode — persis seperti `panel/cpanel.rs` melakukan untuk `config.pterodactyl.*`.

### `owner/autosambut.rs` (1)

- L4: flag `--global` tidak bisa diport: cabangnya menulis ke SETIAP grup hasil `sock.groupFetchAllParticipating()`, sedangkan `Ctx` tidak punya daftar grup dan satu `Mutation::SetGroup` hanya bisa satu JID. Perintah yang membawa `--global` di sini berlaku untuk grup ini saja. Tambahkan `all_group_jids: Vec<String>` ke `Ctx` kalau router mau keduanya.

### `owner/autosholat.rs` (1)

- L8: 1. `saluranCtx()` tidak ada di `Reply`; tidak dipakai handler ini. 2. Reaksi hilang sebagian di cabang `kota`: JS bereaksi `🔍` sebelum request lalu `📍` sesudahnya (`autosholat.js:158,168`), sedangkan `Reply::Reaction` hanya bisa membawa satu emoji. Yang dipasang adalah reaksi terakhir — satu-satunya yang masih terlihat setelah request selesai, karena bereaksi dua kali pada pesan yang sama menimpa. 3. Cache `NodeCache` 24 jam di `sholat-api.js` tidak ikut; `ctx.http` tidak punya lapisan cache dan itu tidak mengubah balasannya.

### `owner/autotyping.rs` (1)

- L3: `saluranCtx()` (`autotyping.js:39`) ditempel sebagai `contextInfo` forwarded-newsletter; `Reply` tidak punya field context, jadi yang hilang hanya metadata itu. Teks balasannya utuh.

### `owner/backupdb.rs` (1)

- L3: `sendStoreBackup(sock)` TIDAK di-port: ia membaca `database/`, `db.json`, `storage/database.json` dari disk dan mengirimkannya lewat `sock`, dan tidak satu pun ada di `Ctx`. `Reply` hanya bisa satu pesan, sedangkan JS mengirim dua (progres lalu hasil), jadi yang dipindah adalah pesan progres — satu-satunya bagian yang murni string. Balasan kedua (size/files/schema) butuh hasil `sendStoreBackup` yang tidak ada.

### `owner/ban.rs` (1)

- L3: `config.bannedUsers = bannedList` di JS hanya mengisi objek config in-memory (dipakai `config.isBanned`), bukan store — jadi tidak ada mutasi untuk itu. `config.isOwner(target)` juga tidak ikut: `Ctx` hanya membawa satu flag untuk pengirim, bukan untuk sembarang nomor, sehingga ban owner tidak dicegah di port ini.

### `owner/bcgc.rs` (1)

- L11: kalau `sock`, `db`, dan state global tersedia di `Ctx` nanti, port ulang dari `plugins/owner/bcgc.js` (guard `stopbcgc`/`jedabcgc`/`on`/ `off`, deteksi media quoted, loop grup dengan jeda, dan reaksi `📢`/`❌`/`✅`/`☢`). Fungsi murni di bawah (`parseDelay`, `formatDelay`) disalin dari JS untuk dijaga agar tidak hilang, dan diuji.

### `owner/bcpc.rs` (2)

- L9: JS collects kontak dari `sock.store.chats`, `sock.store.messages`, dan `sock.store.contacts` (`:79-113`). `Ctx` tidak punya daftar kontak global — `ctx.user_data` hanya memuat data pengirim, dan `sock` hanya bisa melihat grup. Karena itu daftar kontak selalu kosong dan handler berakhir di balasan "Tidak ada kontak ditemukan" — persis seperti JS ketika tidak ada kontak yang terdeteksi. Broadcast sungguhan (yang butuh `sock.sendMedia`/`sock.sendText` ke JID lain) juga tidak bisa jalan: `Reply` hanya sampai ke `caller.chat`. ponytail: `jedaBcpc` dibaca dari `db.setting("jedaBcpc") || 5000` (`:42`) — dipetakan ke key setting `jedaBcpc`.
- L18: `jedaBcpc` dibaca dari `db.setting("jedaBcpc") || 5000` (`:42`) — dipetakan ke key setting `jedaBcpc`.

### `owner/block.rs` (1)

- L7: guard JS "jangan blokir bot sendiri" butuh `sock.user?.id` (JID bot) yang tidak ada di `Ctx` — guard tidak di-port, blokir nomor bot sendiri tetap mungkin. Upgrade path: ekspos `bot_id` di `Ctx`. `unblock.rs` memakai `resolve_target` dari file ini.

### `owner/buatsaluran.rs` (1)

- L8: `sock.newsletterCreate(name, description)` tidak ada di `oktz_plugin_api::sock::Sock` — API itu hanya punya operasi grup dan kontak. Jadi pembuatan saluran selalu jatuh ke cabang `catch` JS, dan pesannya menjelaskan alasannya.

### `owner/cekschedule.rs` (1)

- L3: `getFullSchedulerStatus()`, `getTodaySchedule()`, `extractPrayerTimes()`, dan `db.setting('autoSholatKota')` berasal dari `src/lib/scheduler.js` dan `src/lib/sholat-api.js`, yang bukan bagian dari `Ctx` API. Hanya `db.setting('autoSholat')` (lewat `ctx.setting`) yang tersedia. Sisa tidak dapat di-port saat ini. Catatan: baris `pub mod cekschedule;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/chatclear.rs` (1)

- L11: kalau `chat-cleaner` diekspose lewat `Ctx` nanti, port ulang dari `plugins/owner/chatclear.js` (guard `on`/`off`/`status`, reaksi `✅`, dan teks respons yang sama). NOTE: baris `pub mod chatclear;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs dilarang disentuh karena sudah milik plugin lain).

### `owner/checkban.rs` (1)

- L4: `config.isOwner(target)` hanya bisa dijawab untuk pengirim di `Ctx`, jadi baris `isOwner()` pada balasannya dicetak sebagai `?`. Selain itu `config.bannedUsers` di JS adalah cache in-memory yang diisi oleh `config.isBanned()`; di sini hanya sumber `db.setting('bannedUsers')` yang dibaca, karena itu satu-satunya yang benar-benar persisten.

### `owner/cpuprofile.rs` (1)

- L3: `src/lib/profiler.js` TIDAK di-port. `startCpuProfile`/ `stopCpuProfile` butuh `node:inspector` dan menulis berkas `.cpuprofile`; `process.memoryUsage()` juga tidak punya padanan di `Ctx`. Jadi tidak ada profil yang benar-benar dibuat. Yang dipindah: parsing sub-perintah, penghitungan durasi (termasuk batas atasnya), nama profil, dan teks help — semuanya murni aritmetika dan string.

### `owner/delenergi.rs` (2)

- L49: `Ctx` hanya mengisi `user_data` untuk `sender` dan `participants[i].data` untuk roster grup. JID di luar keduanya dianggap "tidak ada di database" — sama dengan `db.getUser(jid)` yang `null`.
- L258: target mention di luar roster tidak punya row di `Ctx`, jadi guard "user tidak ditemukan" dulu yang kena — sama seperti `db.getUser(targetJid)` yang `null` untuk user yang tak pernah terdaftar. Barisnya ditulis ke JID target, bukan ke sender.

### `owner/delexp.rs` (2)

- L45: hanya `sender` (dan anggota roster grup) yang row-nya terbaca lewat `Ctx`; lihat catatan modul `addenergi`.
- L247: row target di luar sender+roster tidak terbaca, jadi guard "tidak ditemukan" dulu yang kena — bukan menulis ke sender.

### `owner/delkoin.rs` (2)

- L50: hanya `sender` (dan anggota roster grup) yang row-nya terbaca lewat `Ctx`; lihat catatan modul `addenergi`.
- L255: row target di luar sender+roster tidak terbaca, jadi guard "tidak ditemukan" dulu yang kena — bukan menulis ke sender.

### `owner/dellevel.rs` (1)

- L3: JS baca `db.getUser(targetJid)` untuk JID yang bukan sender. `Ctx` hanyabringakan data sender, jadi user lain diambil dari roster (`ctx.roster.participants[].data`) — entry yang tidak ada di sana dianggap user baru (`exp = 0`), sama seperti `db.getUser(x) || db.setUser(x)`.

### `owner/delplugin.rs` (2)

- L8: registry plugin Rust bersifat STATIS — mendaftarkan plugin lewat `linkme::distributed_slice` saat compile, dan tidak ada `import()` dinamis seperti di JS. Karena itu plugin yang sudah terdaftar tidak bisa dilepas saat runtime. Balasan_success replaced dengan penjelasan bahwa penghapusan butuh build/restart atau versi JS. Pencarian plugin memakai registry (`oktz_plugin_api::find`) — padanan paling dekat dari `findPluginFile` JS yang mencocokkan nama/alias plugin.
- L77: unload + `fs.unlinkSync` tidak bisa dari `Reply` — registry plugin Rust di-link saat compile (linkme), tidak ada pemuatan runtime.

### `owner/disableplugin.rs` (3)

- L8: cabang terakhir JS menulis `isEnabled: true` → `false` di berkas plugin sumber, lalu meminta restart/hot reload. Di Rust registry plugin bersifat statis (linkme) — plugin didaftarkan saat compile dan `PluginMeta::is_enabled` tidak dibaca router saat dispatch. Jadi plugin tidak bisa dimatikan saat runtime; balasannya menjelaskan itu. ponytail: `ctx.berkas()` berakar ke direktori kerja, bukan folder `plugins/` — jadi penulisan berkas plugin sumber juga tidak tersedia di sini.
- L14: `ctx.berkas()` berakar ke direktori kerja, bukan folder `plugins/` — jadi penulisan berkas plugin sumber juga tidak tersedia di sini.
- L65: registry statis — plugin tidak bisa dimatikan saat runtime.

### `owner/enableplugin.rs` (1)

- L10: cabang terakhir JS menulis `isEnabled: false` → `true` di berkas plugin lalu meminta restart. Di Rust registry bersifat statis (linkme) dan semua plugin bawaan `is_enabled = true`, jadi cabang itu tidak punya padanan yang berguna — plugin yang ada di registry memang sudah aktif, dan hasilnya sama persis dengan balasan "sudah aktif".

### `owner/exec.rs` (1)

- L3: `eval()` kode JavaScript dan `config.isOwner(m.sender)` tidak dapat di-replikasi di Rust (tidak ada mesin JS, dan `config.isOwner` bukan bagian dari `Ctx`). Yang dipindah: deteksi kode dari quoted / teks, dan teks penjelasan. Eksekusi kode sendiri tidak dapat dilakukan. Catatan: baris `pub mod exec;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/ganti-asset.rs` (1)

- L43: sediakan akses tulis config + filesystem, dan daftarkan `answer_handler` untuk nomor balasan asset.

### `owner/ganti-namabot.rs` (2)

- L8: objek config live (`config.bot.name = newName`) tidak bisa dimutasi di `Ctx` (`config` berupa `Arc`), sama seperti `ganti-namadev`. Yang ditulis hanya berkas `config.js`. ponytail: regex JS `/bot:\s*\{[\s\S]*?name:\s*['"]([^'"]*)['"]/` (`ganti-namabot.js:32-36`) digantikan pemindai sederhana — crate `regex` tidak ada di `oktz-plugins`.
- L12: regex JS `/bot:\s*\{[\s\S]*?name:\s*['"]([^'"]*)['"]/` (`ganti-namabot.js:32-36`) digantikan pemindai sederhana — crate `regex` tidak ada di `oktz-plugins`.

### `owner/ganti-namaowner.rs` (2)

- L9: cabang `main <nama>` di JS menulis berkas `path.join(process.cwd(), "config.js")` lalu memutasi objek config live. Di produksi `ctx.berkas` masih `BerkasPalsu` (memori) dan `ctx.config` bertipe `Arc` yang tidak bisa dimutasi, jadi penulisan itu tidak punya efek nyata; handler berhenti di jalur `catch` JS sendiri (`te(m.prefix, m.command, m.pushName)`, `ganti-namaowner.js:68-70`). Saat `ctx.berkas` jadi `BerkasNyata` dan ada mutasi config, cabang regex `owner: {...name: '...'}` bisa diisi penuh. Catatan: berkas ini bernama `ganti-namaowner.rs` (pakai dash), jadi mod.rs butuh `#[path = "ganti-namaowner.rs"] pub mod ganti_namaowner;`.
- L144: penulisan config.js tidak punya efek di produksi (`ctx.berkas` = `BerkasPalsu`, `ctx.config` Arc tak termutasi). JS berakhir di `catch` saat penulisan gagal, dan itu yang dikirim.

### `owner/ganti-rpg.rs` (1)

- L36: tambahkan akses tulis asset/file ke `Ctx`, lalu unduh byte gambar dan simpan ke `assets/image/rpg.jpeg`.

### `owner/ganti_demote.rs` (2)

- L3: `updateAssetUrl` dari `src/lib/uploader.js` (unggah ke asset / config realtime) tidak diekspos ke plugin di registry Rust. Unduhan media (`ctx.download_quoted_media()` / `ctx.download_media()`) tersedia, tetapi unggahan hasilnya tidak dapat dieksekusi.
- L62: `updateAssetUrl('demote', buffer, 'demote.jpeg')` dan pembaruan config realtime tidak tersedia di registry Rust.

### `owner/ganti_foto.rs` (1)

- L3: `updateAssetUrl` (`src/lib/uploader.js`) untuk unggah dan mendapatkan URL baru tidak tersedia di `Ctx`/`Reply`; setelah unduh media berhasil, handler diakhiri `error_template` + `☢` (pola `swm.rs`). Unduh media itu sendiri (`m.download` / `m.quoted.download`) di-port melalui `Ctx::download_media` / `download_quoted_media`. Saat `updateAssetUrl` tersedia, tambahkan `pub mod ganti_foto;` di `src/owner/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `owner/ganti_gambar.rs` (1)

- L14: `fit: inside` murni belum ada di `gambar` — `ubah_ukuran` memotong (cover) lebih dulu, lalu `pas_di_dalam()` mengecilkan ke dalam 1280x720. Hasil gambarnya sedikit berbeda dari JS, teks balasannya tidak.

### `owner/ganti_goodbye.rs` (1)

- L3: `m.download()` / `m.quoted.download()` dan `updateAssetUrl('goodbye', buffer,...)` dari `src/lib/uploader.js` tidak ada di `Ctx` API, jadi unduh + upload tidak dapat dijalankan. Yang dipindah: deteksi gambar (trigger atau quoted) dan teks balasan. Catatan: baris `pub mod ganti_goodbye;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/ganti_namadev.rs` (1)

- L3: objek config live (`config.bot.developer = newName`) tidak dapat dimutasi di `Ctx` — hanya berkas `config.js` yang ditulis, sama seperti JS yang menimpa berkas lalu memuat ulang. `fs.readFileSync`/`writeFileSync` → `ctx.berkas().baca_teks()` / `tulis_teks()` jalur `"config.js"` (relatif ke akar aset = CWD). Regex tanpa crate `regex` (tidak ada di Cargo.toml): cari `developer:`, lewati spasi, ambil antiko cutip, ganti isinya. `te(...)` → `crate::common::error_template(&ctx)`.

### `owner/ganti_rules.rs` (2)

- L3: `updateAssetUrl` dari `src/lib/uploader.js` (unggah ke asset / config realtime) tidak diekspos ke plugin di registry Rust. Unduhan media tersedia, tetapi unggahan hasilnya tidak dapat dieksekusi.
- L61: `updateAssetUrl('rules', buffer, 'rules.jpeg')` dan pembaruan config realtime tidak tersedia di registry Rust.

### `owner/ganti_store.rs` (1)

- L3: `updateAssetUrl` (`src/lib/uploader.js`) untuk unggah dan mendapatkan URL baru tidak tersedia di `Ctx`/`Reply`; setelah unduh media berhasil, handler diakhiri `error_template` + `☢` (pola `swm.rs`). Unduh media itu sendiri di-port melalui `Ctx::download_media` / `download_quoted_media`. Saat `updateAssetUrl` tersedia, tambahkan `pub mod ganti_store;` di `src/owner/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `owner/gantiscraper.rs` (2)

- L12: hot reload (`import()` dinamis + `hotReloadPlugin`, `gantiscraper.js:82-101`) MUSTAHIL di Rust: registry plugin/scraper didaftarkan saat compile (linkme), tidak ada evaluasi JS saat runtime. Jadi berkas scraper tetap ditulis dan di-backup, tapi bagian "Hot Reload" diganti pemberitahuan bahwa bot perlu restart. ponytail: `ctx.berkas()` berakar ke direktori kerja (cwd) — sama dengan `process.cwd()` di JS, jadi `./src/scraper` dan `./backup/scraper` punya padanan persis.
- L18: `ctx.berkas()` berakar ke direktori kerja (cwd) — sama dengan `process.cwd()` di JS, jadi `./src/scraper` dan `./backup/scraper` punya padanan persis.

### `owner/goodbyeall.rs` (1)

- L3: `sock.groupFetchAllParticipating()` TIDAK di-port. Daftarnya grup hanya ada di Baileys, dan `Ctx` tidak menyediakannya: `ctx.group_data` adalah baris store milik SATU grup, bukan peta semua grup. Tanpa daftar itu `db.setGroup(.., { leave })` tidak punya target, jadi plugin berhenti di balasan "tidak tersedia" dan TIDAK menulis store apa pun. Yang dipindah: parsing aksi dan teks usage.

### `owner/hapusdata.rs` (2)

- L10: `db.resetToDefaults()` (`hapusdata.js:34`) menulis ulang seluruh berkas database (`users.json`, `groups.json`, `settings.json`, …) dengan default. Store Rust adalah satu basis data SQLite/JSON dengan tabel terpisah, dan `Mutation` hanya punya `SetUser`/`SetGroup`/ `SetSetting` — tidak ada "hapus semua tabel". Jadi: - statistik hanya bisa dihitung untuk tabel yang terlihat lewat `Ctx` (`settings` dan `sewa`); `users`/`groups`/`stats`/`premium`/ `owner`/`partner` tidak bisa dienumerasi dari API plugin. - reset yang bisa dilakukan hanya menyetel ulang key `settings` dan `sewa` ke default JS (`defaultSettings`, `defaultSewa`). ponytail: backup folder (`database/backups/pre-reset-<ts>`) tidak bisa dibuat karena lokasi berkas store Rust tidak diekspos ke plugin; path-nya tetap ditulis supaya teksnya sama dengan JS.
- L21: backup folder (`database/backups/pre-reset-<ts>`) tidak bisa dibuat karena lokasi berkas store Rust tidak diekspos ke plugin; path-nya tetap ditulis supaya teksnya sama dengan JS.

### `owner/heapsnap.rs` (1)

- L3: `takeHeapSnapshot` (v8 `writeHeapSnapshot`) dan `heapSnapshotEligibility` (yang membaca `process.memoryUsage()`) tidak di-port: keduanya butuh V8/heap dan tidak ada di `Ctx`. Yang dipindah: konstanta ceiling, `formatMB`, dan parsing label — semua murni. Catatan meta: alias JS memuat `"heapsnap"` — sama dengan `name`. Dibiarkan apa adanya (lihat laporan batch), bukan dirapikan diam-diam.

### `owner/leave.rs` (3)

- L3: Tidak ada API grup (`sock.groupLeave`, `sock.groupGetInviteInfo`, `m.groupMetadata`) yang diekspos ke plugin di registry Rust. Bagian grup tidak dapat dicerminkan; handler di bawah hanya menangani parsing argumen dan kasus "tidak ada grup yang ditargetkan", lalu jatuh ke error saat operasi grup harus dieksekusi.
- L66: `m.groupMetadata` dan `sock.groupLeave` tidak tersedia di registry; grup aktif tidak dapat ditargetkan.
- L77: `sock.groupGetInviteInfo` dan `sock.groupLeave` tidak tersedia di registry, jadi operasi grup tidak dapat dieksekusi.

### `owner/listban.rs` (1)

- L3: JS membaca `config.bannedUsers` dulu dan baru jatuh ke `db.setting('bannedUsers')` kalau cache itu kosong. `config.bannedUsers` adalah cache in-memory milik `config.js` (`config.js:521`), tidak ada di `Ctx`, jadi yang dipakai selalu setting.

### `owner/listjadibot.rs` (1)

- L3: `getAllJadibotSessions()` / `getActiveJadibots()` berasal dari `src/lib/jadibot-manager.js`, dan `sock.sendMessage` dengan `interactiveButtons` tidak ada di `Ctx` API. Tidak dapat di-port saat ini. Yang dipindah: teks penjelasan ketersediaan. Catatan: baris `pub mod listjadibot;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/listjadibotaktif.rs` (1)

- L10: kalau entry point kedua jadibot dan `sock` tersedia di `Ctx` nanti, port ulang dari `plugins/owner/listjadibotaktif.js` (render `@id`, uptime, owner, daftar mention, dan tombol interaktif). `format_uptime` disalin dari JS untuk dijaga dan diuji.

### `owner/listsewa.rs` (1)

- L3: JS menyimpan data sewa di `db.db.data.sewa`, tabel top-level yang bukan milik user maupun grup. `Mutation` hanya punya `SetUser` / `SetGroup` / `SetSetting`, jadi tabel itu dipetakan ke key global `sewa` di `settings` — persis yang dilakukan `group/checksewa.rs:18` untuk plugin yang sama. Bentuk isinya sama dengan JS, hanya lokasinya yang berbeda. Semantik yang disalin dari JS: - `db.db.data.sewa` yang falsy dibuat `{ enabled: false, groups: {} }` (`listsewa.js:43-46`); tabel yang sudah ada tidak pernah ditulis ulang. - Urutan `Object.keys` — `serde_json` di workspace memakai `preserve_order`, jadi urutan key sama dengan urutan di file. - Sort lifetime di belakang lalu `expiredAt` menaik (`:60-66`); `Array#sort` stabil, `sort_by` Rust juga. - `timeHelper.fromTimestamp(addedAt, 'DD/MM/YYYY')` → `common::date_dmy` (zona Asia/Jakarta); `addedAt` falsy jadi `'-'` (`:80`). - Ambang ⚠️ `diff <= 259200000` (`:37`), pembagian integer `Math.floor` (`:24-26`). - `new Date(non-angka)` = Invalid Date → `formatToParts` melempar `RangeError` sebelum teks terkirim; `Reply::None` + mutasi init adalah padanan terdekat, sama seperti `group/checksewa.rs:185`.

### `owner/mutechat.rs` (1)

- L3: `sock.chatModify({ mute: 1 | null }, targetJid)` dan `m.react('✅')` tidak ada di `Ctx` API, jadi tidak dapat meniru perilaku JS apa adanya. Yang dipindah: resolusi target (mention / quoted / nomor argumen / chat saat private) dan teks balasan. Aksi chatModify sendiri tidak dapat dilakukan di sini. Catatan: baris `pub mod mutechat;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/pinchat.rs` (1)

- L34: sediakan operasi chatModify lewat `Ctx` (atau router), lalu replikasikan logika targetJid (mention → quoted → args → chat).

### `owner/prefix.rs` (1)

- L9: `m.command?.toLowerCase()` di JS sudah lowercase lewat router, jadi `ctx.command` dipakai langsung.

### `owner/ptvch.rs` (1)

- L11: kalau `download_media` dan variant video PTV tersedia di `Ctx`/`Reply` nanti, port ulang dari `plugins/owner/ptvch.js` (reaksi `🕕` lalu `✅`, pesan konfirmasi ke pemilik).

### `owner/public.rs` (1)

- L3: dua sumber runtime tidak ada di `Ctx`: 1. `config.mode` — variabel mutable di `config.js`. Yang dipakai adalah `db.setting('botMode')`, cermin durable-nya yang juga dibaca router (`mode_settings_from`, `pipeline.rs:250`). 2. `m.fromMe` — tidak ada di `Ctx`, jadi jalur `if (m.fromMe) return true` hilang; owner tetap harus lolos cek nomor di bawah. `OWNER_NUMBERS` ikut dipakai `self.rs`.

### `owner/renewsewa.rs` (2)

- L3: Tidak ada API database (`db.db.data.sewa`, `db.db.write()`) dan API grup (`sock.groupGetInviteInfo`, `sock.sendText`) yang diekspos ke plugin di registry Rust. Bagian penyimpanan dan pengiriman pesan ke grup tidak dapat dicerminkan. Handler di bawah hanya mem-port parsing argumen, validasi durasi, dan pemformatan, lalu jatuh ke error saat operasi DB/grup harus dieksekusi.
- L110: `resolveGroupId` (butuh `sock.groupGetInviteInfo`) dan operasi `db.db.data.sewa` / `db.db.write()` tidak tersedia di registry, jadi penyimpanan sewa dan pengiriman notifikasi ke grup tidak dapat dieksekusi.

### `owner/resetdb.rs` (2)

- L9: JS menyalin `database/db.json` ke `database/db_backup_<ts>.json` lalu mengosongkan tabel `users`, `groups`, dan `clans` (`resetdb.js:57-86`). Store Rust adalah satu basis data (SQLite/JSON) dengan tabel terpisah dan `Mutation` hanya punya `SetUser`/`SetGroup`/ `SetSetting` — tidak ada "kosongkan tabel" maupun "hapus semua user". Karena itu jumlah user/group/clan yang dilaporkan selalu `0` dan hanya key `settings`/`sewa` yang bisa disetel ulang ke default JS.
- L99: tabel `users`/`groups`/`clans` tidak bisa dikosongkan dari API plugin; hanya key settings/sewa yang bisa disetel ulang.

### `owner/resetlimitdefault.rs` (1)

- L3: `config.limits` tidak ada di `config.js`, jadi `config.limits?.default || 25` selalu `25` — angka itu hasil runtime JS, bukan tebakan.

### `owner/savekontak.rs` (3)

- L7: daftar grup diambil dari `sock.groupFetchAllParticipating()` (`savekontak.js:22`) yang tersedia sebagai `Sock::semua_grup()`. ponytail: nomor bot sendiri tidak bisa disaring — `sock.user.id` tidak ada di `Ctx` — jadi semua peserta grup ikut masuk ke VCF. ponytail: pesan kedua JS (`{ contacts: { displayName, contacts } }`, `savekontak.js:72-77`) adalah `contactsMessage` WA; `Reply` tidak punya varian itu, jadi hanya dokumen VCF yang dikirim.
- L10: nomor bot sendiri tidak bisa disaring — `sock.user.id` tidak ada di `Ctx` — jadi semua peserta grup ikut masuk ke VCF. ponytail: pesan kedua JS (`{ contacts: { displayName, contacts } }`, `savekontak.js:72-77`) adalah `contactsMessage` WA; `Reply` tidak punya varian itu, jadi hanya dokumen VCF yang dikirim.
- L13: pesan kedua JS (`{ contacts: { displayName, contacts } }`, `savekontak.js:72-77`) adalah `contactsMessage` WA; `Reply` tidak punya varian itu, jadi hanya dokumen VCF yang dikirim.

### `owner/schedule.rs` (1)

- L11: kalau `scheduler` diekspose lewat `Ctx` nanti, port ulang seluruh logika `add`/`edit`/`list`/`kategori`/`preset`/`detail`/`del`/`status` dari `plugins/owner/schedule.js`. NOTE: baris `pub mod schedule;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs dilarang disentuh karena sudah milik plugin lain).

### `owner/self.rs` (1)

- L3: sama seperti `public.rs`: `config.mode` dibaca lewat `db.setting('botMode')` dan `m.fromMe` tidak ada di `Ctx`. Cek owner dan pembacaan mode dipinjam dari `public.rs` supaya kedua plugin tidak menyimpang diam-diam.

### `owner/sessionstats.rs` (1)

- L9: `sessionKeyStats()` (`src/lib/turso-session.js`) membaca tabel `session_keys` Turso — tabel auth state Baileys. `Ctx` hanya membawa `sessions` (peta in-memory untuk state antarm plugin), bukan auth state, dan tidak ada API Turso di `oktz-plugin-api`. Jadi cabang "ada key" tidak punya padanan dan yang bisa ditampilkan adalah cabang "tidak ada key".

### `owner/setmenu.rs` (1)

- L52: sediakan akses store + asset buffer + kirim tombol ke `Ctx`, lalu replikasikan loop variant dan tombol single_select.

### `owner/setmenucat.rs` (2)

- L3: `db.setting(key, value)` + `db.save()` (penulisan) dan `sock.sendButton` tidak diekspos ke plugin di registry Rust. Hanya baca `ctx.setting("menucatVariant")` yang tersedia. Cabang tulis (dengan argumen) dan kirim tombol interaktif (tanpa argumen) jatuh ke error / placeholder.
- L102: `db.setting("menucatVariant", selected.id)` + `db.save()` (penulisan database) tidak tersedia di registry Rust, jadi perubahan tidak dapat disimpan.

### `owner/setownertype.rs` (1)

- L3: `db.setting`/`db.save` (`src/lib/database.js`) dan `sock.sendMessage` dengan `interactiveButtons` tidak tersedia di `Ctx`/`Reply`; handler diakhiri `error_template` + `☢` (pola `swm.rs`). Tabel varian, parsing opsi, dan teks pilihan di-port agar bisa diuji. Saat store + tombol interaktif tersedia, tambahkan `pub mod setownertype;` di `src/owner/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `owner/setreply.rs` (1)

- L3: tombol `single_select` + `sock.sendButton` tidak punya padanan di `Ctx`/`Reply`. Baris tombol JS (`setreply.js:110-127`) dirender jadi daftar teks di dalam body (`Reply::Interactive`), pola yang sama dengan `owner/setgoodbyetype.rs`; penanda ` ✓` dan urutan varian dipertahankan. `config.ui?.replyVariant` tidak ada di `Runtime`, jadi nilai awal variant hanya `db.setting("replyVariant")` (`:108`). Quirk yang disalin apa adanya: pencocokan variant **case-sensitive** (`VARIANTS[m.text]`, `:92`) — `V5` bukan `v5`; penanda ` ✓` hanya muncul kalau nilai tersimpan benar-benar angka yang sama (`val.id === current`, `:112`), jadi `"5"` (string) tidak menandai apa pun. Catatan: `mod.rs` kategori `owner` dimiliki plugin lain (tidak boleh disentuh), jadi baris `pub mod setreply;` perlu ditambahkan manual.

### `owner/setwelcometype.rs` (1)

- L3: tombol `single_select` + `sock.sendButton` tidak punya padanan di `Ctx`/`Reply`. Body JS dikirim apa adanya (`setwelcometype.js:72-85`, termasuk blok `*PENJELASAN TIPE:*` yang sudah memuat semua keterangan), lalu baris tombol (`:54-61`) dirender jadi daftar teks — pola yang sama dengan `owner/setgoodbyetype.rs`. Quirk yang disalin: argumen di-lowercase (`:41`) dan wajib cocok `/^v?[1-8]$/` (`:43`); `"v3 extra"` tetap sukses karena hanya argumen pertama yang dibaca. `VARIANTS[current].name` melempar `TypeError` kalau `welcomeType` tersimpan bukan 1–8 (`:75`), jadi cabang itu berakhir di `error_template` seperti jalur catch framework JS. Catatan: `mod.rs` kategori `owner` dimiliki plugin lain (tidak boleh disentuh), jadi baris `pub mod setwelcometype;` perlu ditambahkan manual.

### `owner/sewabot.rs` (5)

- L3: JS menyimpan data sewa di `db.db.data.sewa`, tabel top-level yang bukan milik user maupun grup. `Mutation` hanya punya `SetUser` / `SetGroup` / `SetSetting`, jadi tabel itu dipetakan ke key global `sewa` di `settings` — persis `group/checksewa.rs:18` dan `owner/listsewa.rs`. ponytail: cabang `confirm` dan `leave` di JS menyapu semua grup (`sock.groupFetchAllParticipating`, `sock.sendText` per grup, lalu `sock.groupLeave`). Pengiriman ke JID lain tidak punya padanan: `Reply` hanya sampai ke `caller.chat` (`pipeline.rs:3058`). Karena itu jalur `try` diperlakukan seperti gagal di JS dan handler mengirim jalur `catch`-nya: `confirm` → reaksi `✅` + `te(...)` (`sewabot.js:142-145`), `leave` → reaksi `☢` + `te(...)` (`:199-203`). Cabang `confirm` tetap menulis `enabled: true` karena JS menulis SEBELUM `try` (`:98-99`). ponytail: reaksi pembuka JS (`🕕`, `:100` dan `:150`) hilang — `Reply` hanya mengirim reaksi sebagai pembungkus level atas (`pipeline.rs:2999`), tidak ada dua reaksi berurutan. `pendingConfirmations` (`:19`) dipetakan ke `ctx.sessions` key `sewabot:pending:<sender>`. TTL 60000 ms yang di JS memakai `setTimeout` (`:77-80`) diperiksa saat dibaca: entri kedaluwarsa dibuang, jadi `confirm` dan `cancel` melihat kondisi "sudah dihapus timer" yang sama. Catatan: `mod.rs` kategori `owner` dimiliki plugin lain (tidak boleh disentuh), jadi baris `pub mod sewabot;` perlu ditambahkan manual.
- L8: cabang `confirm` dan `leave` di JS menyapu semua grup (`sock.groupFetchAllParticipating`, `sock.sendText` per grup, lalu `sock.groupLeave`). Pengiriman ke JID lain tidak punya padanan: `Reply` hanya sampai ke `caller.chat` (`pipeline.rs:3058`). Karena itu jalur `try` diperlakukan seperti gagal di JS dan handler mengirim jalur `catch`-nya: `confirm` → reaksi `✅` + `te(...)` (`sewabot.js:142-145`), `leave` → reaksi `☢` + `te(...)` (`:199-203`). Cabang `confirm` tetap menulis `enabled: true` karena JS menulis SEBELUM `try` (`:98-99`). ponytail: reaksi pembuka JS (`🕕`, `:100` dan `:150`) hilang — `Reply` hanya mengirim reaksi sebagai pembungkus level atas (`pipeline.rs:2999`), tidak ada dua reaksi berurutan. `pendingConfirmations` (`:19`) dipetakan ke `ctx.sessions` key `sewabot:pending:<sender>`. TTL 60000 ms yang di JS memakai `setTimeout` (`:77-80`) diperiksa saat dibaca: entri kedaluwarsa dibuang, jadi `confirm` dan `cancel` melihat kondisi "sudah dihapus timer" yang sama. Catatan: `mod.rs` kategori `owner` dimiliki plugin lain (tidak boleh disentuh), jadi baris `pub mod sewabot;` perlu ditambahkan manual.
- L16: reaksi pembuka JS (`🕕`, `:100` dan `:150`) hilang — `Reply` hanya mengirim reaksi sebagai pembungkus level atas (`pipeline.rs:2999`), tidak ada dua reaksi berurutan. `pendingConfirmations` (`:19`) dipetakan ke `ctx.sessions` key `sewabot:pending:<sender>`. TTL 60000 ms yang di JS memakai `setTimeout` (`:77-80`) diperiksa saat dibaca: entri kedaluwarsa dibuang, jadi `confirm` dan `cancel` melihat kondisi "sudah dihapus timer" yang sama. Catatan: `mod.rs` kategori `owner` dimiliki plugin lain (tidak boleh disentuh), jadi baris `pub mod sewabot;` perlu ditambahkan manual.
- L234: sapu grup (`sendText` + `groupLeave` per grup non-whitelist) butuh pengiriman ke JID lain — `Reply` berhenti di `caller.chat` (`pipeline.rs:3058`). Jalur `try` dianggap gagal, jadi hasilnya `catch` JS: reaksi ✅ + `te(...)` (`sewabot.js:142-145`). Tambahkan begitu pipeline punya API kirim/keluar grup.
- L254: sama seperti `confirm` — sapu grup tidak bisa dari `Reply`; hasilnya `catch` JS: reaksi ☢ + `te(...)` (`sewabot.js:199-203`).

### `owner/sistemdaftar.rs` (2)

- L12: statistik pendaftaran (`getRegistrationStats`, `sistemdaftar.js:29-44`) menghitung SELURUH user yang sudah daftar — `db.getAllUsers()` tidak tersedia di `Ctx` (hanya data pengirim yang diekspos). Jadi jumlahnya selalu `0`. ponytail: `sock.sendMessage(..., { contextInfo: getRegistrationContextInfo() })` menandai pesan sebagai forwarded dari surat berita. `Reply::Text` tidak punya slot `contextInfo`, jadi penandaan itu tidak ikut.
- L17: `sock.sendMessage(..., { contextInfo: getRegistrationContextInfo() })` menandai pesan sebagai forwarded dari surat berita. `Reply::Text` tidak punya slot `contextInfo`, jadi penandaan itu tidak ikut.

### `owner/srt.rs` (3)

- L12: MD5 (`crypto.createHash("md5")`) tidak punya crate yang tersedia di `oktz-plugins`, jadi implementasinya diletakkan di berkas ini (lihat [`md5_hex`]). Hasilnya identik dengan MD5 standar, sehingga nama berkas `srt_<hash>.jpeg` sama dengan yang dipakai JS. ponytail: album gambar (`albumMessage`, `srt.js:86-125`) tidak ada di `Reply`; jadi gambarnya dikirim berurutan sebagai pesan terpisah lewat `Reply::sequence`. ponytail: `global.srtSession` dipetakan ke `ctx.sessions` dengan key `srt:session:<chat>`; `srtEnabled` dipetakan ke setting global `srtEnabled`.
- L17: album gambar (`albumMessage`, `srt.js:86-125`) tidak ada di `Reply`; jadi gambarnya dikirim berurutan sebagai pesan terpisah lewat `Reply::sequence`. ponytail: `global.srtSession` dipetakan ke `ctx.sessions` dengan key `srt:session:<chat>`; `srtEnabled` dipetakan ke setting global `srtEnabled`.
- L21: `global.srtSession` dipetakan ke `ctx.sessions` dengan key `srt:session:<chat>`; `srtEnabled` dipetakan ke setting global `srtEnabled`.

### `owner/star.rs` (1)

- L3: `sock.chatModify` dengan field `star` tidak ada di `Ctx` API, dan `m.react('⭐')` tidak bisa digabung dengan teks balasan karena `Reply::with_reaction` hanya satu emoji. Yang dipindah: deteksi quoted, penanganan argumen `hapus`/`unstar`, dan teks balasan. Perubahan state bintang tidak dapat dilakukan di sini. Catatan: baris `pub mod star;` perlu ditambah ke `owner/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `owner/stop.rs` (1)

- L3: langkah kedua TIDAK di-port. `process.exit` butuh menghentikan proses dari dalam handler, dan `Ctx` sengaja tidak punya akses ke itu — plugin pure tidak boleh mematikan host. Yang dipindah adalah gate owner dan teks balasan. Kalau shutdown dari command memang dibutuhkan, itu keputusan router/pipeline, bukan plugin.

### `owner/stopbcpc.rs` (1)

- L3: JS memakai global mutable `global.statusBcpc` / `global.stopBcpc`. `Ctx` hanya membawa `sessions`, jadi keduanya dipindah ke sana DENGAN KEY YANG SAMA — `bcpc.js` yang di-port harus memakai `session_set("statusBcpc",..)` dan `session_take("stopBcpc")`, kalau tidak kedua plugin tidak pernah melihat satu sama lain.

### `owner/stopdandeletejadibot.rs` (1)

- L36: sediakan antarmuka stop/hapus session jadibot, lalu replikasikan daftar sesi + pilihan target via mention/reply.

### `owner/stopschedule.rs` (1)

- L3: `stopSchedulerByName`, `stopSholatScheduler` (`src/lib/scheduler.js`, `src/lib/sholat-scheduler.js`) dan `getDatabase()`/`db.setting` tidak tersedia di `Ctx`; handler diakhiri `error_template` + `☢` (pola `swm.rs`). Teks bantuan dan daftar scheduler di-port agar bisa diuji. Saat scheduler + store tersedia, tambahkan `pub mod stopscheduler;` di `src/owner/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `owner/swgc.rs` (2)

- L11: `sock.sendMessage(jid, { groupStatusMessage })` — pesan status grup dengan border hijau — tidak punya padanan di `Reply` (tidak ada varian `groupStatusMessage`). Karena itu branch `--confirm` berakhir di bentuk `catch` JS: `❌ *ᴇʀʀᴏʀ*` dengan pesan gagalnya. ponytail: `pendingSwgc` (Map di memori) dipetakan ke `ctx.sessions` dengan key `swgc:pending:<sender>`; media disimpan ke `temp/swgc_<ts>.<ext>` — padanan persis dari `tempFile` JS (`swgc.js:205`).
- L16: `pendingSwgc` (Map di memori) dipetakan ke `ctx.sessions` dengan key `swgc:pending:<sender>`; media disimpan ke `temp/swgc_<ts>.<ext>` — padanan persis dari `tempFile` JS (`swgc.js:205`).

### `owner/swgcall.rs` (1)

- L17: saat `Sock` mendukung group status + `semua_grup()`, dan `Reply` mendukung interactive buttons, implementasikan ulang dan tambahkan `pub mod swgcall;` di `src/owner/mod.rs` secara manual.

### `owner/swgcv2all.rs` (1)

- L10: pesan `groupStatusMessageV2` (ring pink) tidak ada padanannya di `Reply` — tidak ada varian yang bisa mengirim status grup ke JID lain. Karena itu tidak ada grup yang berhasil dikirimi; jumlahnya dilaporkan apa adanya (Sukses 0, Gagal = jumlah grup), dan alasannya dijelaskan di doc-comment modul ini.

### `owner/unban.rs` (1)

- L3: dua hal tidak ada di `Ctx` dan sengaja tidak ditebak: 1. `isLid`/`lidToJid` (`src/lib/lid.js`) butuh cache di disk. Tanpa cache, `lidToJid` tinggal mengganti sufiks `@lid` — dan karena `resolveTarget` membuang semua non-digit sesudahnya, digit hasilnya tetap sama. 2. `config.bannedUsers = bannedList` menulis cache in-memory `config.js`; sumber yang dibaca `config.isBanned` adalah `db.setting('bannedUsers')`, jadi mutasi setting sudah cukup.

### `owner/welcomeall.rs` (1)

- L3: sama seperti `goodbyeall.js`: `sock.groupFetchAllParticipating()` tidak di-port, jadi daftar grup tidak bisa diambil dan `db.setGroup(.., { welcome })` tidak punya target. Lihat catatan di `goodbyeall.rs`.


## search — pencarian (87 catatan / 39 file)

### `search/android1-get.rs` (4)

- L3: JS mengirim dokumen lewat `sock.sendMessage` dengan `contextInfo: { forwardingScore: 99, isForwarded: true }`. `Reply::Document` tidak punya context info, jadi dokumen terkirim tanpa penanda forwarded — nama berkas, mimetype, dan URL tetap sama. ponytail: cabang "URL tidak tersedia" mengirim `forwardedNewsletterMessageInfo` (butuh `config.saluran`/`config.bot.name`) plus `interactiveButtons` cta_url. `Reply::Interactive` hanya membawa teksnya, jadi yang terkirim cuma `> ⚠️ Download URL tidak tersedia`. ponytail: reaksi pertama (`🕕`) tidak ikut: `Reply::Reaction` hanya satu emoji, dan reaksi terakhir yang menandai hasil akhir — sama seperti `search/android1.rs`.
- L8: cabang "URL tidak tersedia" mengirim `forwardedNewsletterMessageInfo` (butuh `config.saluran`/`config.bot.name`) plus `interactiveButtons` cta_url. `Reply::Interactive` hanya membawa teksnya, jadi yang terkirim cuma `> ⚠️ Download URL tidak tersedia`. ponytail: reaksi pertama (`🕕`) tidak ikut: `Reply::Reaction` hanya satu emoji, dan reaksi terakhir yang menandai hasil akhir — sama seperti `search/android1.rs`.
- L13: reaksi pertama (`🕕`) tidak ikut: `Reply::Reaction` hanya satu emoji, dan reaksi terakhir yang menandai hasil akhir — sama seperti `search/android1.rs`.
- L25: `Ctx` tidak membawa config, jadi kunci ikut `config.js` yang di-hardcode — sama seperti `search/android1.rs`.

### `search/android1.rs` (4)

- L4: JS mengirim kartu `sock.sendButton` dengan picker `single_select` (judul "Pilih APK", footer "📱 Android1 Search", dan thumbnail `getAssetBuffer("foto")`). `Reply::Interactive` hanya membawa teksnya, jadi daftar yang sama terkirim tanpa tombol/thumbnail — persis teks `android1.js:83-105`. ponytail: JS menyimpan hasil ke `db.db.data.sessions` (map sessions tingkat-atas, di luar `Mutation` yang tersedia) supaya tombol `android1-get` bisa membacanya. Tidak ada plugin di batch ini yang membaca map itu, jadi tulisannya dilewati. ponytail: reaksi pertama (`🔍`) tidak ikut: `Reply::Reaction` hanya membawa satu emoji, dan reaksi terakhir (`✅`/`❌`/`☢`) yang menandai hasil akhir — sama seperti `info/jadwalbola.rs`.
- L10: JS menyimpan hasil ke `db.db.data.sessions` (map sessions tingkat-atas, di luar `Mutation` yang tersedia) supaya tombol `android1-get` bisa membacanya. Tidak ada plugin di batch ini yang membaca map itu, jadi tulisannya dilewati. ponytail: reaksi pertama (`🔍`) tidak ikut: `Reply::Reaction` hanya membawa satu emoji, dan reaksi terakhir (`✅`/`❌`/`☢`) yang menandai hasil akhir — sama seperti `info/jadwalbola.rs`.
- L15: reaksi pertama (`🔍`) tidak ikut: `Reply::Reaction` hanya membawa satu emoji, dan reaksi terakhir (`✅`/`❌`/`☢`) yang menandai hasil akhir — sama seperti `info/jadwalbola.rs`.
- L27: `Ctx` tidak membawa config, jadi kunci ikut `config.js` yang di-hardcode — persis `info/jadwalbola.rs`. Begitu `.env` diisi, baris ini ikut stale sampai router menyediakannya lewat `Ctx`.

### `search/animeapaini.rs` (3)

- L8: `sock.sendMessage({ image: { url }, caption })` dipetakan ke `Reply::image(MediaSource::Url, caption)`. `Reply::Image` tidak menyimpan status kutipan seperti `{ quoted: m }` di JS. ponytail: `full_matches` kosong tetap lolos guard (`![]` falsy di JS), lalu `full_matches[0].anilist` melempar TypeError dan masuk `catch` — hasilnya template error, bukan "tidak ditemukan".
- L12: `full_matches` kosong tetap lolos guard (`![]` falsy di JS), lalu `full_matches[0].anilist` melempar TypeError dan masuk `catch` — hasilnya template error, bukan "tidak ditemukan".
- L116: `toFixed(2)` JS membulatkan setengah ke arah luar sedangkan `{:.2}` Rust memakai pembulatan ke genap; angka tepat di tengah bisa berbeda satu digit terakhir. Teks asal (`"92.5%"`) tidak pernah terpengaruh.

### `search/apkmod-get.rs` (3)

- L3: JS mengirim dokumen lewat `sock.sendMessage` dengan `contextInfo: { forwardingScore: 99, isForwarded: true }`; `Reply::Document` tidak punya context info, jadi penanda forwarded tidak ikut. ponytail: cabang "URL tidak tersedia" mengirim `forwardedNewsletterMessageInfo` (`config.saluran?.id`, `config.saluran?.name || config.bot?.name`) — `Ctx` tidak membawa config, dan `Reply::Interactive` tidak punya context info, jadi yang terkirim hanya teksnya.
- L7: cabang "URL tidak tersedia" mengirim `forwardedNewsletterMessageInfo` (`config.saluran?.id`, `config.saluran?.name || config.bot?.name`) — `Ctx` tidak membawa config, dan `Reply::Interactive` tidak punya context info, jadi yang terkirim hanya teksnya.
- L19: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.

### `search/apkmod.rs` (4)

- L3: JS mengirim daftar lewat `sock.sendButton` dengan picker `single_select` plus thumbnail `getAssetBuffer("foto")`. `Reply::Interactive` hanya membawa teksnya, jadi tombol (judul, deskripsi, `id` untuk `apkmod-get`) dan thumbnail tidak ikut — persis teks `apkmod.js:52-58`. ponytail: label tombol `${app.name.substring(0, 24)}` bisa melempar TypeError kalau `name` bukan string. Karena tombol tidak bisa dikirim, pemotongan 24 karakter itu tidak ikut, dan `name` yang hilang dirender `undefined` lewat [`interp`] — bukan error. ponytail: `ctx.sessions` tidak punya padanan map session tingkat-atas yang dipakai tombol, jadi tidak ada state yang ditulis di sini.
- L8: label tombol `${app.name.substring(0, 24)}` bisa melempar TypeError kalau `name` bukan string. Karena tombol tidak bisa dikirim, pemotongan 24 karakter itu tidak ikut, dan `name` yang hilang dirender `undefined` lewat [`interp`] — bukan error. ponytail: `ctx.sessions` tidak punya padanan map session tingkat-atas yang dipakai tombol, jadi tidak ada state yang ditulis di sini.
- L13: `ctx.sessions` tidak punya padanan map session tingkat-atas yang dipakai tombol, jadi tidak ada state yang ditulis di sini.
- L22: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.

### `search/applemusic.rs` (1)

- L3: JS memakai `resolver.resolve('ytmusic', …)` — lapisan backend berlapis: scraper lokal `ytmusic-api` dulu, baru agregator. Backend lokal itu modul npm dengan state internal, dan `src/lib/resolve.js` belum di-port, jadi yang dipakai di sini hanya backend agregator yang sudah menjadi cadangan di JS (`api.nexray.eu.cc/search/applemusic`, path tanpa prefix `/api` — menambahkannya membuatnya 404). Bentuk field sudah dinormalkan di `normalize` (`src/capabilities/ytmusic.js`), jadi yang dibaca di sini bentuk yang sama dengan yang dibaca plugin JS.

### `search/bingimage.rs` (4)

- L3: hasil pencarian dikirim sebagai `albumMessage` (`sock.sendMessage`), dan `Reply` hanya punya satu sumber media per balasan — tidak ada padanan album. Yang dipindah: guard kata kunci, daftar URL hasil, unduhan tiap gambar (termasuk kegagalan unduhan yang di JS masuk `catch`), dan reaksi. Balasan tanpa media yang berhasil dikirim apa adanya — tidak ada teks lain di JS untuk album itu. ponytail: `fotoApi.apiFaa.get("/faa/google-image", …)` adalah `src/lib/apimanager.js` yang menambah `User-Agent` bawaan dan parameter query; URL-nya ditulis langsung di sini dengan bentuk yang sama.
- L10: `fotoApi.apiFaa.get("/faa/google-image", …)` adalah `src/lib/apimanager.js` yang menambah `User-Agent` bawaan dan parameter query; URL-nya ditulis langsung di sini dengan bentuk yang sama.
- L19: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.
- L99: Album tidak bisa dikirim; lihat catatan di kepala file.

### `search/carigrup.rs` (2)

- L3: kunci API ada di `config.APIkey?.cuki`, dan `Ctx` tidak membawa config, jadi nilainya ikut `config.js` yang di-hardcode — persis `search/android1.rs`. Effect-nya cabang "API key belum dikonfigurasi" jadi jalur yang benar-benar terpakai sampai router menyediakan config.
- L56: batas JS dihitung per kode unit UTF-16; di sini per `char`, jadi emoji di dekat batas bisa berbeda satu karakter dari JS.

### `search/carisinyal.rs` (3)

- L4: JS memakai `cheerio` (paket npm). `oktz-plugins` tidak punya parser HTML dan menambah dependency baru dilarang, jadi markup yang dibaca plugin — `.oxy-post`, `.oxy-post-title`, `.oxy-post-meta`, `table.box-info tr.box-baris`, `td.kolom-satu`, `td.kolom-dua`, `h1`, dan dua `<meta>` — diparse manual, sama seperti `search/gsmarena.rs`. ponytail: `.text()` di cheerio membaca teks turunan dan membuang markup; `text_content` di bawah melakukan hal yang sama. Entitas HTML yang lazim (`&amp;`, `&nbsp;`, dst) diterjemahkan, sisanya dibiarkan apa adanya. ponytail: `sock.sendMessage({ image: { url }, caption })` tidak punya masalah apa pun di sini — `Reply::image` membawa URL sebagai `MediaSource::Url`, sama seperti wire-nya.
- L10: `.text()` di cheerio membaca teks turunan dan membuang markup; `text_content` di bawah melakukan hal yang sama. Entitas HTML yang lazim (`&amp;`, `&nbsp;`, dst) diterjemahkan, sisanya dibiarkan apa adanya. ponytail: `sock.sendMessage({ image: { url }, caption })` tidak punya masalah apa pun di sini — `Reply::image` membawa URL sebagai `MediaSource::Url`, sama seperti wire-nya.
- L14: `sock.sendMessage({ image: { url }, caption })` tidak punya masalah apa pun di sini — `Reply::image` membawa URL sebagai `MediaSource::Url`, sama seperti wire-nya.

### `search/chords.rs` (2)

- L3: kunci API ada di `config.APIkey?.neoxr`, dan `Ctx` tidak membawa config, jadi nilainya ikut `config.js` yang di-hardcode — persis `search/android1.rs`. Selama router belum menyediakan config, cabang "API key belum diset" (`explainFailure` + `MissingApiKeyError` dari `src/lib/stalker-fallback.js`) yang jadi jalur utama; itu alasan jalur HTTP ditutup dengan satu fungsi murni [`chord_text`] yang bisa diuji tanpa jaringan dan tanpa key.
- L17: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.

### `search/comparation.rs` (3)

- L4: file JS punya DUA handler. `handler` (yang ada di sini) memulai pencarian dan menulis sesi; `comparationAnswerHandler` adalah hook pesan biasa (`if (!m.body || m.isCommand) return false`) yang membaca angka pilihan, kelanjutannya, lalu hasil perbandingan. Hook pesan biasa tidak ada di `oktz-plugin-api` — satu plugin satu command — jadi handler kedua itu tidak ikut, dan sesi yang ditulis di sini tidak punya tahap lanjutan. ponytail: `db.getUser(m.sender)` lalu `user.compare_session = {…}` + `db.save()` dipetakan ke `Mutation::SetUser` yang menimpa satu field.
- L11: `db.getUser(m.sender)` lalu `user.compare_session = {…}` + `db.save()` dipetakan ke `Mutation::SetUser` yang menimpa satu field.
- L20: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.

### `search/film.rs` (2)

- L3: JS mengirim daftar lewat `sock.sendButton` dengan picker `single_select` dan thumbnail `getAssetBuffer("foto")`. `Reply::Interactive` hanya membawa teksnya, jadi tombol (judul, deskripsi, `id` untuk `filmget`) dan thumbnail tidak ikut — persis teks `film.js:49-57`.
- L15: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.

### `search/filmget.rs` (3)

- L3: JS mengirim `sock.sendMessage` dengan `footer` + `contextInfo: saluranCtx()` + `interactiveButtons` cta_url. Saluran ctx membaca `config.saluran`/`config.bot.name` dan `Reply` tidak punya context info, jadi `Reply::Interactive` hanya membawa teksnya. ponytail: `axios.get(film.thumbnail, { responseType: "arraybuffer" })` dibungkus `try {} catch {}` — hasilnya TIDAK dipakai apa pun, cuma memaksa thumbnail terunduh. Request itu tetap dilakukan di sini supaya urutan dan jumlahnya sama.
- L8: `axios.get(film.thumbnail, { responseType: "arraybuffer" })` dibungkus `try {} catch {}` — hasilnya TIDAK dipakai apa pun, cuma memaksa thumbnail terunduh. Request itu tetap dilakukan di sini supaya urutan dan jumlahnya sama.
- L20: `config.APIkey?.neoxr || ''` — lihat catatan di `search/android1.rs`.

### `search/google.rs` (1)

- L3: JS memakai cheerio untuk mem-parse RSS. Di sini dokumen XML dibaca dengan pencarian tag sederhana — RSS Google News punya bentuk yang tetap (`<item>` dengan `<title>`, `<link>`, `<pubDate>`, `<source>`), jadi parser HTML tidak perlu dan menambah dependency baru dilarang. Dua header JS (`User-Agent` dan `X-Engine-Context`) belum bisa dikirim: `HttpGet` tidak punya setter header per-request. Keduanya dicatat di sini supaya tidak hilang saat mekanismenya ditambahkan. Host tidak ditulis langsung: `ENGINE_CONFIG.TARGET_HOST` di `src/scraper/google.js` disimpan sebagai base64 lalu di-decode sekali di konstruktor. Hasilnya sama, dan QString base64 ikut dipakai untuk `compileBinaryPayload`, jadi tidak ada kode base64 yang bisa insulting.

### `search/gsmarena.rs` (2)

- L3: JS memakai paket npm `gsmarena-api` yang mem-parse HTML dengan cheerio. Yang dipindah di sini bentuk respons yang dibaca plugin (hasil pencarian + detail perangkat) dengan selector yang sama persis, ditulis manual karena crate ini tidak punya parser HTML dan menambah dependency baru dilarang. ponytail: `www.gsmarena.com` menyajikan halaman "Turnstile check" untuk klien tanpa browser. Kalau itu yang masuk, `parse_search` mengembalikan daftar kosong dan plugin menjawab "HP tidak ditemukan" — persis apa yang terjadi di JS ketika cheerio tidak menemukan satu pun `<li>`.
- L9: `www.gsmarena.com` menyajikan halaman "Turnstile check" untuk klien tanpa browser. Kalau itu yang masuk, `parse_search` mengembalikan daftar kosong dan plugin menjawab "HP tidak ditemukan" — persis apa yang terjadi di JS ketika cheerio tidak menemukan satu pun `<li>`.

### `search/lyrics.rs` (2)

- L4: `fetchLyrics` menelan semua error axios dan mengembalikan `null`, jadi kegagalan jaringan berakhir di cabang "tidak ditemukan", bukan di `catch` handler. Cabang `catch` handler karena itu tidak bisa dicapai dari `Ctx` mana pun — sama seperti di JS — dan tetap ditulis supaya tidak hilang tanpa jejak.
- L156: tidak bisa dicapai: satu-satunya pemanggilan jaringan dibungkus `try {} catch { return null }` di `fetchLyrics`, dan setelah itu tidak ada lagi yang bisa melempar. Ditulis supaya teks aslinya tidak hilang.

### `search/mangatoon.rs` (2)

- L7: `sock.sendMedia(m.chat, cover, caption.trim(), m, {type:'image'})` ada padanannya: `Reply::Image` dengan `MediaSource::url`. ponytail: `trimText` memakai `value.length` dan `value.slice`, yaitu satuan UTF-16 V8. Di sini dipakai `.chars()` — beda hanya untuk emoji dan karakter astral di luar BMP.
- L10: `trimText` memakai `value.length` dan `value.slice`, yaitu satuan UTF-16 V8. Di sini dipakai `.chars()` — beda hanya untuk emoji dan karakter astral di luar BMP.

### `search/mcpedl.rs` (1)

- L7: `trimText` memakai satuan UTF-16 V8 (`length`/`slice`); di sini `.chars()`. Sama seperti `search::mangatoon`.

### `search/melolo.rs` (2)

- L4: `config.APIkey?.covenant` tidak ada di `Ctx`, jadi guard `melolo.js:74` selalu benar dan `explainFailure(MissingApiKeyError(..))` yang jadi balasan — sama seperti `download::sfiledl`. ponytail: `fotoApi.covenant.meloloCategory` (`src/lib/apimanager.js`) tidak di-port; jalur HTTP setelahnya tidak terjangkau selama kunci kosong, jadi hanya logika plugin yang dipindah: normalisasi hasil, pemotongan teks, dan susunan caption.
- L8: `fotoApi.covenant.meloloCategory` (`src/lib/apimanager.js`) tidak di-port; jalur HTTP setelahnya tidak terjangkau selama kunci kosong, jadi hanya logika plugin yang dipindah: normalisasi hasil, pemotongan teks, dan susunan caption.

### `search/movieku.rs` (2)

- L3: JS memakai cheerio untuk mem-parse halaman detail. Crate ini tidak punya parser HTML dan menambah dependency baru dilarang, jadi yang dipakai di sini hanya operasi yang sama dengan selector yang dibaca plugin: `h1`, `.entry-content p`, `img[src*="wp-content/uploads"]`, `ul li`, `a` di dalam `li`, `a[href*="abyssplayer"]`, `strong`, dan `a` di dalam `parent()`. ponytail: JS mengirim DUA pesan saat ada lebih dari satu hasil: kartu poster dulu, lalu daftar "HASIL LAINNYA". `Reply` hanya satu pesan, jadi yang terkirim kartu pertamanya; teks daftar tetap ditulis sebagai fungsi murni.
- L9: JS mengirim DUA pesan saat ada lebih dari satu hasil: kartu poster dulu, lalu daftar "HASIL LAINNYA". `Reply` hanya satu pesan, jadi yang terkirim kartu pertamanya; teks daftar tetap ditulis sebagai fungsi murni.

### `search/nerdfont-ambil.rs` (2)

- L6: `sharp` tidak ada di API plugin: `jpegThumbnail` hasil `.resize(50, 50)` di `nerdfont-ambil.js:78-86` tidak ikut. Sisanya (dokumen zip + caption) ada padanannya di `Reply::Document`. ponytail: `sock.sendMessage` di `:74` tidak di-`await`, jadi handler JS selalu selesai tanpa balasan apa pun; di sini dokumennya dikembalikan sebagai balasan karena itu satu-satunya jalanpipeline untuk mengirim media.
- L10: `sock.sendMessage` di `:74` tidak di-`await`, jadi handler JS selalu selesai tanpa balasan apa pun; di sini dokumennya dikembalikan sebagai balasan karena itu satu-satunya jalanpipeline untuk mengirim media.

### `search/nerdfont.rs` (1)

- L3: JS memakai cheerio. Crate ini tidak punya parser HTML dan menambah dependency baru dilarang, jadi yang dipakai di sini hanya operasi yang sama dengan selector yang dibaca plugin: `div.item`, `span.nerd-font-invisible-text`, `div` pertama, `a.font-preview`, `a.nf-oct-link_external`, dan `a.nf-fa-download`. Dua anomali JS dipertahankan apa adanya: 1. `textContent.match(/Version:…/)[1]` melempar kalau pola tidak cocok — satu item tanpa `Version:`/`Info:`/gambar membuat SELURUH permintaan gagal. `parse_fonts` mengembalikan `Err` untuk itu, persis `throw`. 2. `sock.sendMessage` dengan `interactiveButtons` + `buttonParamsJson` diteruskan apa adanya: `single_select` + `sections[0].rows` jadi `InteractiveButton`.

### `search/npm.rs` (1)

- L8: `data.objects.slice(0, 8).forEach` tidak menambah baris baru di akhir tiap paket (`> ⭐ Score: …` tanpa `\n`), jadi paket berikutnya menempel ke baris yang sama. Anomali itu ikut dipertahankan.

### `search/pap.rs` (2)

- L3: `resolver.resolve("pinterest", { q })` (`src/lib/resolve.js`) belum di-port, jadi `fetch_urls` selalu `Err` dan yang menyala adalah cabang `catch` biasa — pola yang sama dengan `download::pindl`. Logika plugin sendiri dipindah semua: daftar query per tipe, validasi magic number, batas ukuran, dan susunan pesan. ponytail: `prepareWAMessageMedia`, `generateWAMessageFromContent`, `viewOnceMessage`, `nativeFlowMessage`, `sock.waUploadToServer`, dan `sock.relayMessage` tidak punya padanan di `Reply`, jadi kartu interaktif yang dikirim JS tidak ikut.
- L9: `prepareWAMessageMedia`, `generateWAMessageFromContent`, `viewOnceMessage`, `nativeFlowMessage`, `sock.waUploadToServer`, dan `sock.relayMessage` tidak punya padanan di `Reply`, jadi kartu interaktif yang dikirim JS tidak ikut.

### `search/pin.rs` (2)

- L3: `resolver.resolve("pinterest", { q })` (`src/lib/resolve.js`) belum di-port, jadi `resolve_pinterest` selalu `Err` dan yang menyala adalah cabang `catch` biasa — pola `download::pindl`. Guard argumen dan seluruh susunan pesan tetap dipor supaya utuh. ponytail: album `albumMessage` + `messageContextInfo.messageSecret` + `sock.relayMessage` + `sock.waUploadToServer` tidak punya padanan di `Reply`, dan `Reply` hanya bisa satu media. Fallback `sock.sendMessage` satu per satu juga tidak ada padanannya.
- L8: album `albumMessage` + `messageContextInfo.messageSecret` + `sock.relayMessage` + `sock.waUploadToServer` tidak punya padanan di `Reply`, dan `Reply` hanya bisa satu media. Fallback `sock.sendMessage` satu per satu juga tidak ada padanannya.

### `search/pinvid.rs` (2)

- L3: `config.APIkey?.firefly` tidak ada di `Ctx`, jadi kunci ikut nilai bawaan `"default"` persis seperti `|| "default"` di JS saat `.env` kosong (`pinvid.js:53`). ponytail: `fs`, `child_process` (`ffmpeg`), `crypto`, album WA (`sock.relayMessage`), dan `config.saluran` tidak ada di API plugin: konversi `.m3u8`, berkas sementara, thumbnail newsletter, dan pengiriman album tidak ikut. Yang dipindah adalah guard, pemotongan daftar, ambang ukuran, dan seluruh susunan pesan. Catatan dari audit: `firefly.maiku.my.id` — tunnel Argo-nya mati (Cloudflare 530 / 1033), jadi endpoint ini praktis selalu gagal.
- L7: `fs`, `child_process` (`ffmpeg`), `crypto`, album WA (`sock.relayMessage`), dan `config.saluran` tidak ada di API plugin: konversi `.m3u8`, berkas sementara, thumbnail newsletter, dan pengiriman album tidak ikut. Yang dipindah adalah guard, pemotongan daftar, ambang ukuran, dan seluruh susunan pesan. Catatan dari audit: `firefly.maiku.my.id` — tunnel Argo-nya mati (Cloudflare 530 / 1033), jadi endpoint ini praktis selalu gagal.

### `search/pixiv.rs` (3)

- L3: `config.APIkey?.neoxr` tidak ada di `Ctx`, jadi kunci dikunci `""` — pola `search::android1` dan `search::mangatoon`. Query `apikey=` tetap dikirim apa adanya, sama seperti `|| ''` di JS. ponytail: `contextInfo.forwardedNewsletterMessageInfo` + `sock.sendMessage` tidak ada padanannya di `Reply`; yang dikirim hanya teks caption-nya. `config.saluran` / `config.bot.name` juga tidak ada di `Ctx`, dan tanpa newsletter info nilainya juga tidak terpakai. ponytail: `error.response?.status === 403` tidak bisa dibedakan: `HttpError` hanya membawa `message`, bukan status. Cabang 403 karena itu tidak terjangkau; teksnya tetap ditulis sebagai fungsi murni.
- L7: `contextInfo.forwardedNewsletterMessageInfo` + `sock.sendMessage` tidak ada padanannya di `Reply`; yang dikirim hanya teks caption-nya. `config.saluran` / `config.bot.name` juga tidak ada di `Ctx`, dan tanpa newsletter info nilainya juga tidak terpakai. ponytail: `error.response?.status === 403` tidak bisa dibedakan: `HttpError` hanya membawa `message`, bukan status. Cabang 403 karena itu tidak terjangkau; teksnya tetap ditulis sebagai fungsi murni.
- L12: `error.response?.status === 403` tidak bisa dibedakan: `HttpError` hanya membawa `message`, bukan status. Cabang 403 karena itu tidak terjangkau; teksnya tetap ditulis sebagai fungsi murni.

### `search/playcall.rs` (1)

- L4: `global.voipClient` (`playcall.js:48`), `fs`, `crypto`, `child_process`/`extensionFor`, `yt-search`, dan resolver `resolver.resolve("youtube",..)` tidak ada di API plugin. Guard `if (!global.voipClient)` jadi selalu benar, jadi itu yang diport — persis `download::sfiledl` dengan kapabilitas yang belum di-port. `pluginConfig.isEnabled` di JS `false`; itu ikut disalin lewat `is_enabled`.

### `search/playch.rs` (1)

- L3: JS memakai paket npm `yt-search`; padanannya ada di `search/ytsearch.rs`. Unduhan audio memakai kapabilitas `youtube` format `mp3` → `oktz_scraper::youtube::get_youtube_direct_url`. PENYIMPANGAN (dijelaskan di laporan akhir): - `sock.sendMessage(chId, { audio, ptt, waveform })` mengirim voice note ke SALURAN. `Reply` di `oktz-plugin-api` tidak punya "kirim ke chat lain", jadi voice note-nya dikirim ke chat tempat perintah dipanggil. - `waveform` kustom tidak ada di `Reply::Audio`. - `extensionFor(format)` tidak dipakai: input ffmpeg dibaca lewat `pipe:0` (ffmpeg mendeteksi format dari isi), bukan berkas sementara berekstensi.

### `search/playvid.rs` (3)

- L3: JS memakai paket npm `yt-search`; padanannya ada di `search/ytsearch.rs`. ponytail: unduhan video memakai kapabilitas `youtube` format `mp4`, yang di JS memakai `getYoutubeDirectUrl(url, "360")` (SaveTube). Padanannya `oktz_scraper::youtube::get_youtube_direct_url`. ponytail: `sock.sendPreview` (pratinjau tautan) menjadi `Reply::Preview`.
- L6: unduhan video memakai kapabilitas `youtube` format `mp4`, yang di JS memakai `getYoutubeDirectUrl(url, "360")` (SaveTube). Padanannya `oktz_scraper::youtube::get_youtube_direct_url`. ponytail: `sock.sendPreview` (pratinjau tautan) menjadi `Reply::Preview`.
- L10: `sock.sendPreview` (pratinjau tautan) menjadi `Reply::Preview`.

### `search/ptvsearch.rs` (1)

- L3: JS mengirim `video: { url }` dengan `mimetype: video/mp4` dan `ptv: true`, tanpa caption. `Reply::video` tidak punya bidang `ptv`, jadi dikirim sebagai video biasa. Satu reaksi akhir (`✅`/`❌`/`☢`) dipasang.

### `search/resep.rs` (1)

- L3: `getDatabase()` (`db.getUser`/`db.save`) dan `resepAnswerHandler` (`answer_handler`) butuh store + pendaftaran answer handler yang tidak tersedia di `Ctx`; handler pencarian untuk teks tidak kosong diakhiri dengan `error_template` + `☢`. Fungsi URL dan rendering daftar/detail di-port agar bisa diuji. Saat store + answer handler tersedia, tambahkan `pub mod resep;` di `src/search/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `search/searchthatsong.rs` (1)

- L3: dua permintaan HTTP berturut-turut dengan cookie sesi (`route-preview` lalu full search) dan pengiriman image + teks lirik + audio berurutan. `Reply::sequence` digunakan, tetapi `quoted: m` pada tiap pesan tidak tersedia di `Ctx`, jadi pesan tidak mengutip pemicu. Header kustom (Cookie, User-Agent) dikirim lewat `HttpCall::call`. Catatan: baris `pub mod searchthatsong;` perlu ditambah ke `search/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `search/soundcloud.rs` (4)

- L3: JS memakai `cheerio` untuk ambil teks `#__NEXT_DATA__` lalu `JSON.parse`. `oktz-plugins` tidak punya parser HTML, jadi elemen `#__NEXT_DATA__` dicari manual dan isi teksnya di-parse sebagai JSON. ponytail: `new URL(image, url).href` untuk resolve gambar — di sini URL relatif diselesaikan terhadap base. ponytail: `sock.sendMedia(..., { type: 'image' })` menjadi `Reply::image(MediaSource::Url)` dengan caption. ponytail: `m.react` dua kali (`🕕` lalu `✅`) — hanya reaksi akhir. Catatan: baris `pub mod soundcloud;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L7: `new URL(image, url).href` untuk resolve gambar — di sini URL relatif diselesaikan terhadap base. ponytail: `sock.sendMedia(..., { type: 'image' })` menjadi `Reply::image(MediaSource::Url)` dengan caption. ponytail: `m.react` dua kali (`🕕` lalu `✅`) — hanya reaksi akhir. Catatan: baris `pub mod soundcloud;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L10: `sock.sendMedia(..., { type: 'image' })` menjadi `Reply::image(MediaSource::Url)` dengan caption. ponytail: `m.react` dua kali (`🕕` lalu `✅`) — hanya reaksi akhir. Catatan: baris `pub mod soundcloud;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L13: `m.react` dua kali (`🕕` lalu `✅`) — hanya reaksi akhir. Catatan: baris `pub mod soundcloud;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).

### `search/spotify.rs` (2)

- L3: JS memakai `resolver.resolve("spotify", { q })`, yang hanya dilayani backend agregator `nexray` (`src/capabilities/spotify.js:149-160`): `GET https://api.nexray.eu.cc/search/spotify?q=…`. Path-nya TANPA prefix `/api` — menambahkannya membuat host menjawab 404 (sama seperti `search/applemusic.rs`). Bentuk respons dinormalkan di `normalize` (`src/capabilities/spotify.js:148`): amplop `{ status, result }` dibuka, `result` berupa array dibaca sebagai daftar track, dan tiap track disaring `trackCari` (`:110-129`) — entri tanpa judul atau tanpa tautan dibuang, `duration` yang bukan string menjadi `"0:00"`. ponytail: tombol "Unduh Lagu Pertama" hanya dikirim kalaucover pertama bisa diambil (dikecilkan ke 300×170 lewat `sharp`) DAN tautannya cocok `open.spotify.com/track/`. Kalau salah satu tidak terpenuhi, plugin mengirim teks polos — persis `spotify.js:62-104`.
- L15: tombol "Unduh Lagu Pertama" hanya dikirim kalaucover pertama bisa diambil (dikecilkan ke 300×170 lewat `sharp`) DAN tautannya cocok `open.spotify.com/track/`. Kalau salah satu tidak terpenuhi, plugin mengirim teks polos — persis `spotify.js:62-104`.

### `search/spotplay.rs` (2)

- L3: JS memakai dua resolutions berurutan: 1. `resolver.resolve("spotify", { q })` — cari track, ambil yang tautannya `open.spotify.com/track/` (`spotplay.js:30`). 2. `resolver.resolve("spotify", { url })` — ambil link unduhan. Backend lokal (spotyloader) sudah ada di crate scraper: `oktz_scraper::spotify::download_spotify` (`src/scraper/spotify.js`). Backend pencarian tidak punya padanan lokal — sama seperti `search/spotify.rs`, pencarian memakai agregator `nexray`. ponytail: `sock.sendMedia(..., { type: "audio", mimetype: "audio/mpeg", ptt: false, fileName })` — `Reply::Audio` tidak punya field nama berkas, jadi nama berkas `Artis - Judul.mp3` tidak ikut (dicatat di laporan).
- L12: `sock.sendMedia(..., { type: "audio", mimetype: "audio/mpeg", ptt: false, fileName })` — `Reply::Audio` tidak punya field nama berkas, jadi nama berkas `Artis - Judul.mp3` tidak ikut (dicatat di laporan).

### `search/ttsearch.rs` (2)

- L3: JS memakai scraper `tiktokSearchVideo` (`src/scraper/tiktoksearch.js`); padanannya sudah ada di crate scraper: `oktz_scraper::tiktoksearch::tiktok_search_video`. ponytail: JS mengirim album berisi maksimal 5 video (`maxShow = Math.min(videos.length, 5)`) dengan `contextInfo` forwarded. `Reply::Video` tidak punya `contextInfo`, jadi penandaan forwarded tidak ikut — sisanya (video + caption) sama.
- L7: JS mengirim album berisi maksimal 5 video (`maxShow = Math.min(videos.length, 5)`) dengan `contextInfo` forwarded. `Reply::Video` tidak punya `contextInfo`, jadi penandaan forwarded tidak ikut — sisanya (video + caption) sama.

### `search/wikipedia.rs` (3)

- L3: JS memakai `cheerio` untuk parse halaman artikel Wikipedia (`#firstHeading`, `.tagline`, `.mw-parser-output > section`, `.infobox tr`, `.mw-parser-output img`). `oktz-plugins` tidak punya parser HTML, jadi semua dipakai manual di bawah. Elemen yang di-remove di JS (`script`, `style`, `.infobox` dll) dilewati saat ekstraksi teks. ponytail: `fixUrl` untuk URL protokol-relatif (`//`) dan relativ (`/`). `uniqueBy` untuk dedup gambar berdasarkan URL. ponytail: `sock.sendMessage({ image, caption })` menjadi `Reply::image(MediaSource::Url)` dengan caption. Catatan: baris `pub mod wikipedia;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L9: `fixUrl` untuk URL protokol-relatif (`//`) dan relativ (`/`). `uniqueBy` untuk dedup gambar berdasarkan URL. ponytail: `sock.sendMessage({ image, caption })` menjadi `Reply::image(MediaSource::Url)` dengan caption. Catatan: baris `pub mod wikipedia;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L12: `sock.sendMessage({ image, caption })` menjadi `Reply::image(MediaSource::Url)` dengan caption. Catatan: baris `pub mod wikipedia;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/search/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).

### `search/yts.rs` (3)

- L3: JS memakai paket npm `yt-search`; padanannya ada di `search/ytsearch.rs` (data sumbernya sama: `ytInitialData`). ponytail: kartu yang dikirim adalah `buttonsMessage` (headerType 6) dengan `locationMessage` berisi thumbnail 300×170, judul, dan alamat `📺 Channel: … | ⏱️ Durasi: …`. Rust memakai `CardHeader::location`. ponytail: `config.bot.name` jadi footer kartu.
- L6: kartu yang dikirim adalah `buttonsMessage` (headerType 6) dengan `locationMessage` berisi thumbnail 300×170, judul, dan alamat `📺 Channel: … | ⏱️ Durasi: …`. Rust memakai `CardHeader::location`. ponytail: `config.bot.name` jadi footer kartu.
- L10: `config.bot.name` jadi footer kartu.


## group — grup (62 catatan / 50 file)

### `group/absen.rs` (1)

- L19: milidetik dan offset diabaikan — satu-satunya penulisnya `new Date().toISOString()` (`mulaiabsen.js:41`), jadi bentuknya selalu `YYYY-MM-DDTHH:mm:ss.sssZ`. Naik ke parser penuh kalau ada sumber lain.

### `group/add.rs` (1)

- L6: `sock.groupParticipantsUpdate` di Rust mengembalikan `()` tanpa detail status per-nomor (JS: `res.status 200/408/lain`). Akibatnya cabang sukses/undang/gagal per-nomor dari JS disatukan: pada `Ok` semua nomor valid dilaporkan berhasil, pada `Err` dilaporkan gagal. Reaksi `🕕` dari JS dihilangkan karena `Reply::with_reaction` hanya membawa satu emoji — hanya reaksi penutup (`✅`/`❌`) yang dikirim.

### `group/addcmdsticker.rs` (3)

- L3: dua hal tidak ikut. 1. `getQuotedStickerHash` membaca `fileSha256`, lalu `mediaKey`, dari `quoted.message.stickerMessage` (`sticker-command.js:80-101`). `QuotedMessage` tidak punya keduanya, jadi tidak ada kunci yang bisa ditulis — dan karena pemeriksaan itu mendahului `getPlugin`, semua cabang setelah "reply sticker" mati. Plugin diam, bukanAddition yang menipu. 2. `src/lib/sticker-command.js` menulis ke `database/stickerCommands.json`, bukan ke store plugin. Pemetaannya ke store global: key `setting("stickerCommands")` dengan bentuk yang sama persis (hash → `{command, addedBy, addedAt}`) — sama yang dipakai `delstickercmd`.
- L26: Cabang JS yang memakainya sudah mati (lihat modul), jadi hanya tes yang memakai helper ini.
- L96: `getQuotedStickerHash` tidak bisa dihitung (lihat modul). Karena pemeriksaan itu mendahului `getPlugin(cleanCmd)` di JS, semua cabang setelahnya mati — plugin diam, bukan melapor sukses tanpa kunci yang benar-benar ditulis.

### `group/addtoxic.rs` (1)

- L56: 1 di Rust — perbedaan itu tidak materiel untuk daftar kata, tapi batas 30EMU bukan 30 byte.

### `group/afk.rs` (1)

- L3: hanya `handler` yang dipindah. `checkAfk` (`afk.js:76-126`) adalah hook middleware yang dipanggil sebelum plugin lain — ia perlu `db.getUser` untuk JID peserta yang di-mention (yang tidak ada di `Ctx`, hanya ada untuk `sender`) dan `notifyThrottle` lintas-pesan, jadi tidak ikut. Lihat laporan batch. `sweepAfkStorage` juga tidak ikut: penyapuan based-on-ukuran hanya memangkas cache in-memory, dan `Ctx` tidak punya cache AFK sendiri.

### `group/antidocument.rs` (1)

- L3: hanya handler `.antidocument` yang dipindah. `checkAntidocument` (pemeriksaan per-pesan) butuh `sock.sendMessage` untuk menghapus pesan — `Ctx` tidak punya itu. `gpMsg` yang membaca `config.groupProtection` ikut tidak dipindah karena hanya dipakai `checkAntidocument`.

### `group/antijudol.rs` (1)

- L3: `checkJudol` (deteksi per-pesan) tidak ikut — butuh `sock.sendMessage` untuk menghapus pesan dan melempar kicked, dan `Ctx` tidak punya keduanya. Yang dipindah di sini hanya handler `.antijudol`.

### `group/antilinkall.rs` (1)

- L3: hanya handler `.antilinkall` yang dipindah. Deteksi per-pesan ada di middleware dan butuh `sock.sendMessage` untuk menghapus pesan — `Ctx` tidak punya itu.

### `group/antilinkgc.rs` (1)

- L3: hanya handler `.antilinkgc` yang dipindah; filter per-pesan ada di middleware dan butuh `sock.sendMessage` untuk menghapus pesan.

### `group/antimedia.rs` (1)

- L3: hanya handler `.antimedia` yang dipindah. `checkAntimedia` (pemeriksaan per-pesan) butuh `sock.sendMessage` untuk menghapus pesan — `Ctx` tidak punya itu. `gpMsg` yang membaca `config.groupProtection` ikut tidak dipindah karena hanya dipakai `checkAntimedia`.

### `group/antiphising.rs` (1)

- L3: hanya handler `.antiphising` yang dipindah; deteksi per-pesan ada di middleware dan butuh `sock.sendMessage` untuk menghapus pesan.

### `group/antisticker.rs` (1)

- L3: hanya handler `.antisticker` yang dipindah. `checkAntisticker` (pemeriksaan per-pesan) butuh `sock.sendMessage` untuk menghapus pesan — `Ctx` tidak punya itu. `gpMsg` yang membaca `config.groupProtection` ikut tidak dipindah karena hanya dipakai `checkAntisticker`.

### `group/autoai.rs` (1)

- L3: tiga tabel yang dipakai JS (`db.db.data.autoai`, `db.db.data.autoai_personas`, `db.db.data.autoai_global`) adalah tabel top-level, jadi dipetakan ke key global dengan nama yang sama di `settings` — persis seperti `clan/clancreate.rs` untuk `clans` dan `rpg/guild.rs` untuk `guilds`. Bentuk isinya sama dengan JS, hanya lokasinya berbeda. Yang tidak dipindah: - `characters.bell409.instruction` di JS dibaca dari `config.autoaiPersonas.Bell409` (`config.js:313`). `Ctx` tidak membawa config runtime, jadi literalnya ikut disalin ke bawah — nilainya persis sama, bukan `""`. - `generateVoiceResponse` (`:466`) + `convertToOggOpus` (`:59`) butuh `axios`, `child_process` (ffmpeg), `fs`, dan `path`. - `saluranCtx()` tidak dipakai di handler ini, jadi tidak ada penyempitan context di sini. Regex `m.fullArgs.match(/--…/)` diganti pemindaian karakter biasa: crate ini tidak punya dependency regex. `--logic=` memakai pola malas dengan lookahead `\s+--(?:persona|type|logic|mode)|$` dan `.` yang tidak melompati pemisah baris — keduanya ditiru di [`flag_logic`].

### `group/autodl.rs` (1)

- L3: `pluginConfig.name` di JS adalah array `['autodl', 'autodownload']`. `PluginMeta.name` hanya `String`, jadi elemen pertama jadi `name` dan sisanya jadi command tambahan (`PluginMeta::commands`) — registry `find()` mengeceknya sebelum alias. Penegakan `m.text` dipindah tidak: filtering link dan unduhan berjalan di middleware/handler lain, bukan di `.autodl`.

### `group/autoforward.rs` (1)

- L3: penerusan pesan nyata ada di middleware dan butuh `sock.sendMessage`/`sock.sendMessage` grup lain, yang tidak ada di `Ctx`. Yang dipindah hanya sakelar `.autoforward`.

### `group/automedia.rs` (1)

- L3: `autoMediaHandler` yang diekspor ke `handler.js` tidak ikut. Fungsi itu berjalan per-pesan dan butuh `m.download()` plus `sock.sendMedia` — keduanya tidak ada di `Ctx`. Yang dipindah hanya perintahnya.

### `group/banchat.rs` (1)

- L7: `db.setGroup(m.chat, {...groupData, isBanned })` menyalin seluruh objek grup. `Mutation::SetGroup` sudah merge, jadi hanya field yang berubah yang perlu dikirim — hasil di store sama.

### `group/botmode.rs` (1)

- L6: hanya `handler` yang dipindah. `getGroupMode`, `getModeCategories`, dan `filterCategoriesByMode` (`botmode.js:117-137`) dipakai middleware untuk menyembunyikan command, bukan oleh handler ini, jadi tidak ikut — lihat laporan batch.

### `group/checksewa.rs` (1)

- L3: JS menyimpan data sewa di `db.db.data.sewa`, yaitu tabel top-level yang bukan milik user maupun grup. `Mutation` hanya punya `SetUser`/`SetGroup`/`SetSetting`, jadi tabel itu dipetakan ke key global `sewa` di `settings` — persis yang dilakukan `clan/clancreate.rs` untuk `db.db.data.clans` dan `rpg/guild.rs` untuk `db.db.data.guilds`. Bentuk isinya sama dengan JS, hanya lokasinya yang berbeda. `formatCountdown` memakai `Math.floor`, jadi pecahan milidetik dipangkas ke bawah — sama dengan pembagian integer yang dipakai di sini.

### `group/clearchat.rs` (1)

- L3: seluruh badan plugin adalah operasi Baileys: `sock.chatModify({ delete, lastMessages })` (`clearchat.js:26-32`) dan, kalau itu gagal, `sock.chatModify({ clear, messages })` (`:38-46`). Keduanya tidak ada di API plugin, jadi tidak ada pesan yang dihapus dan tidak ada balasan sukses yang dikirim — plugin diam, bukan melapor chat sudah bersih. `m.react('🗑️')`/`'☢'` ikut hilang karena menempel pada percakapan yang sama.

### `group/delete.rs` (2)

- L9: `sock.sendMessage(m.chat, { delete: key })` tidak ada di API plugin, jadi `m.react('✅')` setelahnya juga tidak terjadi — plugin diam setelah gerbang lolos, bukan melapor penghapusan yang tidak terjadi. `botJids` dari `sock.user` juga tidak terbaca, jadi `isBotMessage` hanya bisa `true` lewat `fromMe`.
- L95: `botJids.includes(...)` butuh `sock.user` — lihat modul.

### `group/delstickercmd.rs` (2)

- L3: dua hal TIDAK di-port. Pertama, `getQuotedStickerHash(m)` membaca `fileSha256`/`mediaKey` dari `quoted.message.stickerMessage`, dan `QuotedMessage` tidak punya keduanya — jadi cabang "reply sticker" selalu jatuh ke pesan "Gagal menghapus!". Kedua, `src/lib/sticker-command.js` menyimpan ke `database/stickerCommands.json`; satu-satunya operasi tulis yang tersedia di `Mutation` adalah `SetSetting`, jadi pemetaannya disimpan di `setting("stickerCommands")` dengan bentuk yang sama (hash → `{command, addedBy, addedAt}`). Plugin yang menambah entri (`addcmdsticker`) harus menulis ke key yang sama.
- L101: Cabang quoted tidak bisa di-port (lihat modul) — hanya jalur nama yang bisa menghapus apa pun.

### `group/getpp.rs` (1)

- L6: pengambilan avatar `sock.profilePictureUrl(target, 'image')` (`getpp.js:34`) dan pengirimannya `sock.sendMedia(...)` (`:39`) tidak ada di API plugin, jadi tidak ada gambar yang dikirim — termasuk fallback `https://files.catbox.moe/ejy4ky.jpg` yang di JS hanya dipakai kalau avatar target gagal diambil.

### `group/hidetag2.rs` (2)

- L7: dua hal di JS tidak bisa di-port. 1. Cabang media (gambar/video/sticker/audio/dokumen) butuh `q.download()` (`downloadContentMessage`) yang tidak ada di `Ctx`, jadi berhenti di `Reply::None`. 2. `fakeQuoted` (remoteJid `status@broadcast`, participant `0@s.whatsapp.net`, conversation = `config.bot.name`) tidak punya padanan di `Reply`; teks dikirim polos. Upgrade path: tambah varian quoted-message di `oktz-plugin-api` kalau `fakeQuoted` benar-benar dipakai klien. JS membakar dua reaction: `📢` sebelum kirim lalu `✅` setelahnya pada mode teks saja. `Reply` hanya punya satu reaction, jadi mode teks memakai `✅`.
- L54: Cabang media tidak bisa di-port (lihat modul).

### `group/ht.rs` (2)

- L8: cabang media (gambar/video/sticker/audio/dokumen) butuh `quoted.download()` (`downloadContentMessage`) yang tidak ada di `Ctx`, jadi hanya mode teks yang hidup. Reply-mode teks memakai `sock.sendMessage` tanpa `{ quoted: m }` (polos), sedangkan mode teks murni memakai `{ quoted: m }` — perbedaan itu dipertahankan lewat `Reply::Text` vs `Reply::TextQuoted`.
- L47: Cabang media tidak bisa di-port (lihat modul).

### `group/htpremium.rs` (3)

- L9: 1. Cabang media di reply-mode butuh `quoted.download()` (`downloadContentMessage`) yang tidak ada di `Ctx`, jadi berhenti di `Reply::None`. 2. `contextInfo.groupMentions: [{ groupJid, groupSubject: cmd }]` di mode teks tidak punya padanan di `Reply::TextQuoted` — teks tetap `@chat` + body dan mention per-jid, tapi tag `@everyone`/custom subject hilang. Upgrade path: tambah field group-mention ke `oktz-plugin-api` kalau klien lokal benar-benar relies pada `groupSubject`.
- L52: `cmd` hanya jadi `groupSubject` yang tidak bisa di-port — lihat.
- L58: Cabang media tidak bisa di-port (lihat modul).

### `group/intro.rs` (1)

- L3: `@group` dan `@desc` diambil dari `m.groupMetadata.subject` / `.desc` — metadata WA, bukan `groups[chat]`. `Ctx` tidak membawa keduanya, jadi kedua placeholder itu selalu jatuh ke nilai cadangan JS. Tambahkan `group_subject` / `group_desc` ke `Ctx` kalau router mau keduanya; sisanya (`@user`, `@count`, `@date`, `@time`, `@botname`) sudah tercermin.

### `group/jadwalgroup.rs` (1)

- L6: `delete group.scheduleOpen` jadi `scheduleOpen = null`, karena `Mutation::SetGroup` merge (`{...existing,...data}`) dan tidak bisa menghapus key. Pembaca di JS semuanya `group.scheduleOpen || null`, jadi `null` dan key yang hilang tidak bisa dibedakan.

### `group/kick.rs` (1)

- L9: dua gerbang "tidak bisa kick bot sendiri" (`kick.js:46` dan `:71-77`) membandingkan target dengan `sock.user?.id` dan `sock.user?.lid`; `Ctx` tidak punya JID bot, jadi keduanya dilewati. Karena mengeluarkan member juga tidak di-port, tidak ada siapa pun yang bisa ikut terpental.

### `group/listwarn.rs` (1)

- L54: JS memilih `m.quoted` sebelum `m.mentionedJid[0]`; `Ctx` tidak membawa pesan yang di-reply, jadi reply-mode tidak terjangkau. Tambahkan `quoted_sender: Option<String>` ke `Ctx` kalau router menyediakannya.

### `group/mulaiabsen.rs` (1)

- L8: `pruneAbsensi()` membaca `createdAt` yang di JS berupa string ISO dari `new Date().toISOString()`. `Ctx` tidak punya parser tanggal, jadi string itu diurai dengan fungsi lokal di bawah (bentuknya tetap `Date#toISOString`, persis seperti yang ditulis JS).

### `group/mute.rs` (1)

- L6: `m.groupMetadata.subject` diambil dari `groups[chat].name`, yang `handler.js:327` isi dari `meta.subject`. Kalau store belum pernah menyimpan nama, teksnya jatuh ke nilai cadangan.

### `group/mutegc.rs` (1)

- L6: `m.groupMetadata.subject` diambil dari `groups[chat].name` (pola yang sama dengan `mute.rs`). `Ctx` tidak membawa subject langsung.

### `group/mutemember.rs` (1)

- L6: `lidToJid`/`findParticipantByNumber` di JS membaca `lidCache` (pemetaan LID → nomor) yang tidak ada di `Ctx`. Yang dipakai di sini cukup penggantian sufiks `@lid` dan pencocokan `jid` roster, yang selalu berisi nomor telepon asli.

### `group/notifclosegroup.rs` (1)

- L7: `m.isAdmin` tidak ada di `Ctx`; guard memakainya `ctx.roster.is_admin(sender)`, roster grup untuk pesan yang sama.

### `group/notifgantitag.rs` (1)

- L6: cabang `on all`/`off all` butuh `sock.groupFetchAllParticipating()` (`notifgantitag.js:34` dan `:57`) untuk tahu daftar grup. `Ctx` tidak punya daftar itu, jadi kedua cabang berhenti di balasan "tidak di-port" — penting: tanpa cabang ini `on all` akan jatuh ke `on` lokal dan diam-diam hanya mengubah satu grup. Ekspor kedua file ini, `handleLabelChange` (`:120`), adalah hook event `protocolMessage` type 30 yang mengirim lewat `sock.sendText` + `saluranCtx()`; keduanya tidak ada di API plugin dan tidak ikut di-port.

### `group/notifsholat.rs` (1)

- L12: `m.isAdmin` tidak ada di `Ctx`; guard memakainya `ctx.roster.is_admin(sender)`, roster grup untuk pesan yang sama. `group.autoSholat ?? setting('autoSholat') ?? false` — `??` hanya null/undefined, jadi `false` milik grup tidak ditimpa global.

### `group/openvo.rs` (1)

- L6: pemeriksaan `content.viewOnce` (`openvo.js:50`) tidak bisa dilakukan karena `QuotedMessage` tidak membawa flag itu, dan pengunduhannya `downloadContentFromMessage` (`:78`) tidak ada di API plugin. Jadi media yang dikenali (image/video/audio) berakhir tanpa balasan apa pun — bukan video yang dikirim.

### `group/pinchat.rs` (1)

- L57: padanannya. `key.rawParticipant`/`key.participant` juga tidak terbawa — tidak masalah karena pin-nya sendiri tidak di-port.

### `group/poll.rs` (2)

- L3: `sock.sendMessage(m.chat, { poll: {...} })` tidak ada di API plugin (poll native WA), jadi validasi dipindah penuh tetapi pembuatan poll-nya tidak. Plugin diam setelah validasi lolos — bukan menulis "✅ Success membuat poll" untuk poll yang tidak pernah dikirim. `question.length` di JS menghitung unit UTF-16; di sini `chars().count()`, yang sama untuk semua teks BMP dan hanya beda untuk emoji di luar BMP.
- L102: `sock.sendMessage({ poll })` tidak ada di `Ctx` (lihat ).

### `group/rulesgrup.rs` (1)

- L3: JS mengirim gambar asset `rules` + caption kalau `getAssetBuffer('rules')` ada, dan teks polos kalau tidak. `Ctx` tidak punya jalur asset, jadi port ini selalu cabang teks. Naik ke `Reply::Image` + `MediaSource::path` begitu router punya asset store.

### `group/setdeskgc.rs` (1)

- L59: JS menghitung `.length` sebagai unit UTF-16, Rust menghitung scalar; emoji di luar BMP terhitung 1 di sini dan 2 di JS. Ambang 2048 jauh dari batas itu untuk deskripsi biasa.

### `group/setnamegc.rs` (1)

- L46: JS menghitung `.length` sebagai unit UTF-16, Rust menghitung scalar; emoji di luar BMP terhitung 1 di sini dan 2 di JS.

### `group/setppgc.rs` (2)

- L3: dua cabang bisa dipindah, dua tidak. Yang dipindah: guard "tidak ada gambar" (`m.quoted?.isImage` → `QuotedKind::is_image`) dan teks errors. Yang tidak: `m.quoted.download()`/`m.download()` (butuh `downloadContentMessage`) dan `sock.updateProfilePicture`. `Ctx` juga tidak membawa flag "pesan yang dipicu adalah gambar", jadi cabang `m.isImage` tidak punya sumber.
- L36: Unduh + unggah tidak ada di `Ctx` (lihat modul).

### `group/setrulesgrup.rs` (1)

- L3: sumber teks di JS adalah `m.text?.trim() || (m.quoted?.body || m.quoted?.text || '')`. `Ctx` tidak membawa pesan yang di-reply, jadi cabang kedua tidak bisa dipindah; plugin hanya menerima teks yang ikut diketik. Kalau quotes di Instrumentasikan nanti, tambahkan `quoted: Option<Quoted>` ke `Ctx` dan pulihkan cabang itu.

### `group/slowmode.rs` (1)

- L6: `Mutation::SetGroup` merge, jadi `slowmode: { enabled: false }` menimpa objek lama secara utuh — persis seperti `{...groupData, slowmode}` di JS, bukan patch di dalam objek.

### `group/totag.rs` (2)

- L3: inti plugin adalah `sock.sendMessage(chat, { forward: m.quoted.fakeObj || m.quoted })`. `Reply` tidak punya varian forward dan `QuotedMessage` tidak menyimpan `fakeObj`/`rawMessage`, jadi tidak ada payload yang bisa dikirim ulang. Yang bisa di-port adalah guard-nya: konteks reply, reaction 📢/✅, dan pemeriksaan roster kosong. Upgrade path: tambah `Reply::Forward` + `QuotedMessage::raw` kalau forward pesan jadi kebutuhan nyata.
- L46: Forward tidak portable; reaction saja yang tersisa (lihat ).

### `group/unmute.rs` (1)

- L4: `groups[chat].name`, lihat catatan di sana.

### `group/unmutegc.rs` (1)

- L3: `m.groupMetadata.subject` diambil dari `groups[chat].name` (pola yang sama dengan `mute.rs`). `Ctx` tidak membawa subject langsung.

### `group/warn.rs` (2)

- L11: gerbang "jangan warn bot sendiri" (`warn.js:84-88`) membandingkan target dengan `sock.user?.id`; `Ctx` tidak punya JID bot, jadi gerbang itu dilewati — pola yang sama dengan `kick.rs`/`delete.rs`.
- L182: Gerbang bot sendiri (`warn.js:84-88`) dilewati — lihat di header modul.


## ai — AI (50 catatan / 38 file)

### `ai/ai.rs` (3)

- L7: `AIRich` mengirim SATU pesan `botForwardedMessage > richResponseMessage` yang berisi beberapa submessage. API plugin Rust tidak punya varian "rich response", jadi tiap blok dikirim sebagai pesan sendiri lewat `Reply::sequence` — teks tetap sama, hanya bentuknya berbeda. Tabel memakai `Reply::table` (`sock.sendTable` di Rust), blok kode memakai teks ber-fence ``` (AIRich merender kartu kode; teks fence adalah padanan terdekat yang tersedia). ponytail: `extractIE` (`builder.js`) memecah inline entity (link, sitasi, LaTeX) dari teks sebelum dikirim. Rust tidak punya parser itu, jadi teks mentah dipakai apa adanya. ponytail: `sessions` di JS adalah `Map` modul-local yang di-evict kalau lewat 500 entri. `Ctx::sessions` adalah `HashMap` global tanpa urutan, jadi eviction tidak bisa disalin; kunci dipakai berawalan `ai:gemini:` supaya tidak bentrok dengan plugin lain.
- L15: `extractIE` (`builder.js`) memecah inline entity (link, sitasi, LaTeX) dari teks sebelum dikirim. Rust tidak punya parser itu, jadi teks mentah dipakai apa adanya. ponytail: `sessions` di JS adalah `Map` modul-local yang di-evict kalau lewat 500 entri. `Ctx::sessions` adalah `HashMap` global tanpa urutan, jadi eviction tidak bisa disalin; kunci dipakai berawalan `ai:gemini:` supaya tidak bentrok dengan plugin lain.
- L19: `sessions` di JS adalah `Map` modul-local yang di-evict kalau lewat 500 entri. `Ctx::sessions` adalah `HashMap` global tanpa urutan, jadi eviction tidak bisa disalin; kunci dipakai berawalan `ai:gemini:` supaya tidak bentrok dengan plugin lain.

### `ai/aiassist.rs` (3)

- L8: `HttpGet` tidak punya setter header per-request, jadi header `cookie`, `sec-ch-ua*`, `origin`, dan `referer` yang dikirim JS tidak ikut. akibatnya `!response.ok` (403/500) tidak pernah terlihat sebagai `status:false` seperti di JS — `UreqHttp` mengubahnya jadi `Err` lebih dulu, jadi yang tampil template error `catch`, bukan `❌ *AI Assistant Error*`. Isi `chatId`, `deviceId`, `selectedChatModel`, dan `locale` tetap sama.
- L40: `slice` JS menghitung UTF-16 code unit, `chars()` menghitung code point. Bedanya cuma terlihat kalau 4096 batasnya jatuh di tengah emoji.
- L158: karena `UreqHttp` sudah `Err` untuk status >= 400 — lihat.

### `ai/anime-gen.rs` (1)

- L3: dua hal tidak ditulis di sini: 1. Header `User-Agent` per-request (`http.js:30`) — `HttpGet` tidak punya setter header. `UreqHttp` sudah mengirim UA sendiri. 2. Cabang `catch` (`anime-gen.js:57-64`). `f()` (`http.js`) menelan semua error dan mengembalikan `null`, jadi `data` selalu `null` saat gagal dan plugin masuk cabang "ɢᴀɢᴀʟ" di `:47`, bukan `catch`. Menulis `catch` berarti menambah jalur yang tidak pernah tercapai di JS.

### `ai/bananagen.rs` (1)

- L3: JS bereaksi `🕕` lalu `✅`; `Reply::with_reaction` hanya satu emoji, jadi hanya `✅` dibawa pada sukses. `m.download()` / `m.quoted.download()` dipindah ke `ctx.download_media()` / `download_quoted_media()`. `live3d` dipanggil lewat `oktz_scraper::seaart::live3d`. Hasil dikirim sebagai `Reply::image` byte (MediaSource::bytes) tanpa quote. Catatan: baris `pub mod bananagen;` perlu ditambah ke `ai/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `ai/bananagen2.rs` (1)

- L7: rantai `error.response.data.message || error.response.data.error` di `bananagen2.js:44-45` tidak bisa terjadi: `UreqHttp` mengubah status `>= 400` jadi `Err` tanpa body. Yang tersisa `error.message`, persis seperti `src/lib/axios`/`UreqHttp` sudah dipakai plugin lain di folder ini.

### `ai/claudehaiku.rs` (1)

- L7: tiga hal tidak bisa di-port dan tidak ada padanannya di `Ctx`/ `Reply`, jadi tidak ditulis (bukan di shortcut): 1. Header per-request (`x-device-uuid`, `sec-ch-ua*`, `origin`, `referer`). 2. `saluranCtx()` (`src/lib/context.js:151`) yang ditempel sebagai `contextInfo` forwarded-newsletter — `Reply` tidak punya field context. 3. `crypto.randomUUID()` diganti generator UUID v4 di [`super::aiassist`] (pola hex yang sama, sumber acaknya `crate::common::roll`).

### `ai/deepseek.rs` (2)

- L3: header `Cookie` (analytics `_ga`/`_gid`/`sbox-guid` + `anonymous_user_id`) tidak bisa dikirim, jadi `makeCookieHeader`, `makeSboxGuid`, dan `randomNumber` (`src/scraper/deepseek.js:13-37`) sengaja tidak ditulis — kodenya hanya dipakai untuk header itu. `history` juga selalu kosong: satu-satunya pemanggil tidak pernah mengirimnya. ponytail: JS memakai `validateStatus: () => true`, jadi respons 4xx/5xx tetap dibaca dan biasanya berakhir sebagai `success:false`. `UreqHttp` membalik itu jadi `Err`, sehingga jalur 403 yang paling mungkin terjadi jatuh ke template error `catch` — bukan `❌ *DeepSeek Gagal*`.
- L9: JS memakai `validateStatus: () => true`, jadi respons 4xx/5xx tetap dibaca dan biasanya berakhir sebagai `success:false`. `UreqHttp` membalik itu jadi `Err`, sehingga jalur 403 yang paling mungkin terjadi jatuh ke template error `catch` — bukan `❌ *DeepSeek Gagal*`.

### `ai/dolphin.rs` (1)

- L6: header `origin`, `referer`, dan `user-agent` khusus tidak bisa dikirim (`HttpGet` tidak punya setter header per-request).

### `ai/efek.rs` (2)

- L9: `withTimeout(..., 20000)` per provider tidak bisa disalin ke mesin sinkron ini (tidak ada timer). `UreqHttp` sudah memasang timeout global 20 detik (`ureq_http.rs:24`), kebetulan sama dengan `PROVIDER_TIMEOUT_MS` di JS.
- L423: `withTimeout(..., 45000)` dan loop provider (`effect-api.js:106-120`) tidak bisa disalin tanpa timer + config; ekspos `effectApi` di `Runtime` kalau host cadangan mau dipakai.

### `ai/feelbetter.rs` (1)

- L3: header `cookie: feelbet-memory=…` tidak bisa dikirim (`HttpGet` tidak punya setter header per-request), jadi `makeMemoryId()` (`src/scraper/feeb.js:11-18`) sengaja tidak ditulis — id-nya cuma dipakai untuk header itu. `saluranCtx()` juga tidak ada padanannya di `Reply`.

### `ai/gita.rs` (1)

- L9: reacted `🕕` (`gita.js:25`) dilewati karena `Reply::Reaction` cuma menampung satu emoji; reaksi akhir (`✅`/`☢`) yang dikirim.

### `ai/gpt5.rs` (1)

- L7: header per-request dan `salunanCtx()` tidak punya padanan di `Ctx`/`Reply` — lihat catatan di `claudehaiku.rs`.

### `ai/jokowi-ai.rs` (1)

- L7: header `cookie`/`sec-ch-ua*`/`origin`/`referer` tidak bisa dikirim (`HttpGet` tidak punya setter header), dan `UreqHttp` mengubah status >= 400 jadi `Err` — jadi cabang `❌ *Jokowi AI Error*` hanya terjangkau lewat implementasi `HttpGet` lain, bukan lewat `UreqHttp`.

### `ai/kobo-ai.rs` (1)

- L6: header `cookie`/`sec-ch-ua*`/`origin`/`referer` tidak bisa dikirim, dan status >= 400 jadi `Err` di `UreqHttp` — lihat `aiassist.rs`.

### `ai/matematika.rs` (1)

- L3: header `User-Agent` per-request (`matematika.js:35`) tidak bisa dikirim — `HttpGet` tidak punya setter header. reacted `🕕` (`matematika.js:27`) dilewati karena `Reply::Reaction` cuma menampung satu emoji; reaksi akhir (`❌`/`✅`/`☢`) yang dikirim.

### `ai/musicmaker.rs` (1)

- L3: JS memanggil `sock.sendMessage` dua kali (audio lalu image) + `m.react("🕕")` lalu `m.react("✅")`. `Reply` hanya menampung satu reaksi, jadi yang dipegang adalah yang terakhir (`✅`), dan kedua pesan jadi satu `Sequence` (audio dulu, image dengan caption diikuti). `example` diisi persis `pluginConfig` JS.

### `ai/muslimai.rs` (1)

- L7: reacted `🕕` (`muslimai.js:67`) dilewati karena `Reply::Reaction` cuma menampung satu emoji; reaksi akhir (`✅`/`☢`) yang dikirim.

### `ai/prabowo-ai.rs` (1)

- L6: header `cookie`/`sec-ch-ua*`/`origin`/`referer` tidak bisa dikirim, dan status >= 400 jadi `Err` di `UreqHttp` — lihat `aiassist.rs`.

### `ai/quillbot.rs` (1)

- L7: header `User-Agent` per-request (`quillbot.js:32`) tidak bisa dikirim — `HttpGet` tidak punya setter header. reacted `🕕` (`quillbot.js:26`) dilewati karena `Reply::Reaction` cuma menampung satu emoji; reaksi akhir (`❌`/`✅`/`☢`) yang dikirim.

### `ai/qwen3.rs` (1)

- L8: tidak ditulis: 1. Header per-request (`x-device-uuid`, `sec-ch-ua*`, `origin`, `referer`) — `HttpGet` tidak punya setter header. 2. `saluranCtx()` (`qwen3.js:52`) yang ditempel sebagai `contextInfo` forwarded-newsletter — `Reply` tidak punya field context. 3. Cabang `status:false` (`qwen3.js:41-46`) praktis tidak terjangkau: `UreqHttp` sudah mengubah status >= 400 jadi `Err`, jadi yang tampil template error `catch`, bukan `❌ *Qwen3 Gagal*`.

### `ai/simi.rs` (1)

- L3: header `User-Agent` per-request (`simi.js:33`) tidak bisa dikirim — `HttpGet` tidak punya setter header. reacted `🕕` (`simi.js:26`) dilewati karena `Reply::Reaction` hanya menampung satu emoji; reaksi terakhir (`❌`/`✅`/`☢`) yang ikut.

### `ai/sologo.rs` (1)

- L3: header `User-Agent` per-request (`sologo.js:32`) tidak bisa dikirim — `HttpGet` tidak punya setter header. reacted `🕕` (`sologo.js:26`) dilewati karena `Reply::Reaction` hanya menampung satu emoji; reaksi terakhir (`❌`/`✅`/`☢`) yang ikut.

### `ai/text2img2.rs` (1)

- L3: header `User-Agent` per-request (`text2img2.js:30`) tidak bisa dikirim — `HttpGet` tidak punya setter header. reacted `🕕` (`text2img2.js:23`) dilewati karena `Reply::Reaction` hanya menampung satu emoji; reaksi terakhir (`❌`/`✅`) yang ikut. `m.fullArgs` dipakai sebagai sumber prompt. `serialize.js:408` mengisinya dengan `result.text`, jadi sama dengan `ctx.text`. Jalur noprefix (`serialize.js:430`) menghitung ulang dari `body`, tapi hasilnya juga `trim()` dari sisa command — tidak ada selisih yang terlihat di user.

### `ai/to3d.rs` (2)

- L10: reaksi `🕕` (`to3d.js:40`) tidak ikut — `Reply::with_reaction` hanya bisa membawa satu emoji, jadi yang dikirim reaksi penutup. ponytail: `m.quoted.isMedia` (bukan `isImage`) yang dipakai untuk menentukan sumber unduhan; `Ctx::quoted().is_some_and(|q| q.is_media())` adalah padanannya.
- L13: `m.quoted.isMedia` (bukan `isImage`) yang dipakai untuk menentukan sumber unduhan; `Ctx::quoted().is_some_and(|q| q.is_media())` adalah padanannya.

### `ai/tocartoon.rs` (1)

- L3: reaksi ganda JS (`🕕` lalu `✅`) disatukan jadi satu reaksi akhir (`✅`) karena `Reply::with_reaction` hanya satu emoji. `live3d` berasal dari `oktz_scraper::seaart` (bukan disalin ke sini).

### `ai/toemotebatu.rs` (2)

- L436: `withTimeout(..., 45000)` dan loop provider (`effect-api.js:106-120`) tidak bisa disalin tanpa timer + config; ekspos `effectApi` di `Runtime` kalau host cadangan mau dipakai.
- L494: reaksi 🕕 hilang (`toemotebatu.js:27`) — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup ❌/✅/☢ yang dikirim.

### `ai/tofigure.rs` (1)

- L11: reaksi `🕕` (`tofigure.js:26`) tidak ikut — `Reply` hanya membawa satu reaksi, jadi yang dikirim reaksi penutup.

### `ai/tofigurev2.rs` (2)

- L437: `withTimeout(..., 45000)` dan loop provider (`effect-api.js:106-120`) tidak bisa disalin tanpa timer + config; ekspos `effectApi` di `Runtime` kalau host cadangan mau dipakai.
- L495: reaksi 🕕 hilang (`tofigurev2.js:27`) — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup ❌/✅/☢ yang dikirim.

### `ai/tofigurine.rs` (1)

- L4: `live3d` di Rust mengembalikan `Vec<u8>` (byte gambar mentah), bukan `{ image }`, karena `serde_json::Value` tidak bisa membawa buffer. Satu reaksi akhir (`✅`/`❌`/`☢`) dipasang.

### `ai/toghibli.rs` (2)

- L435: `withTimeout(..., 45000)` dan loop provider (`effect-api.js:106-120`) tidak bisa disalin tanpa timer + config; ekspos `effectApi` di `Runtime` kalau host cadangan mau dipakai.
- L492: reaksi 🕕 hilang (`toghibli.js:27`) — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup ❌/✅/☢ yang dikirim.

### `ai/tohijab.rs` (1)

- L11: reaksi `🕕` (`tohijab.js:26`) tidak ikut — `Reply` hanya membawa satu reaksi, jadi yang dikirim reaksi penutup.

### `ai/tojapanese.rs` (1)

- L3: `uploadImage` (`src/lib/uploader.js`) dan `applyImageEffect` (`src/lib/effect-api.js`) tidak tersedia di `Ctx`/`Reply`; handler untuk gambar diakhiri `error_template` + `☢` (pola `swm.rs`). Pemeriksaan gambar dan teks pakai di-port agar bisa diuji. Saat unggah + efek tersedia, tambahkan `pub mod tojapanese;` di `src/ai/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `ai/tomekah.rs` (2)

- L435: `withTimeout(..., 45000)` dan loop provider (`effect-api.js:106-120`) tidak bisa disalin tanpa timer + config; ekspos `effectApi` di `Runtime` kalau host cadangan mau dipakai.
- L492: reaksi 🕕 hilang (`tomekah.js:27`) — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup ❌/✅/☢ yang dikirim.

### `ai/tooilpainting.rs` (1)

- L10: reaksi `🕕` (`tooilpainting.js:35`) tidak ikut — `Reply` hanya bisa membawa satu emoji, jadi yang dikirim reaksi penutup.

### `ai/txt2img.rs` (1)

- L4: JS memakai `f()` dari `src/lib/http.js` yang tidak pernah melempar (status >= 400 kembali `null`); di sini `ctx.http.get` mengembalikan `Err` untuk status >= 400, jadi cabang gagal ditangani lewat `Err`/null. Satu reaksi akhir (`❌`/`✅`/`☢`) dipasang.

### `ai/txt2img2.rs` (2)

- L7: `HttpGet` tidak punya POST dengan `Content-Type: text/plain`, jadi `checkPrompt` memakai `post_json` — body `["<prompt>"]` sama persis, hanya header isian yang beda. ponytail: `getResult` di JS membaca stream sambil jalan (bisa 180 detik) dan berhenti begitu `process_completed` muncul. `HttpGet` cuma bisa GET penuh, jadi blok SSE yang belum lengkap ikut ikut terbaca — hasil yang dipakai tetap sama, hanya waktunya tidak bisa dipotong.
- L11: `getResult` di JS membaca stream sambil jalan (bisa 180 detik) dan berhenti begitu `process_completed` muncul. `HttpGet` cuma bisa GET penuh, jadi blok SSE yang belum lengkap ikut ikut terbaca — hasil yang dipakai tetap sama, hanya waktunya tidak bisa dipotong.

### `ai/waguri-ai.rs` (1)

- L6: header `cookie`/`sec-ch-ua*`/`origin`/`referer` tidak bisa dikirim, dan status >= 400 jadi `Err` di `UreqHttp` — lihat `aiassist.rs`.

### `ai/wormgpt.rs` (1)

- L9: reacted `🕕` (`wormgpt.js:29`) dilewati karena `Reply::Reaction` cuma menampung satu emoji; reaksi akhir (`✅`/`☢`) yang dikirim.


## canvas — canvas/gambar (39 catatan / 27 file)

### `canvas/applemusic.rs` (1)

- L3: kunci upload `TERMAI_UPLOAD_KEY` dibaca dari `process.env` di JS, yang tidak ada di `Ctx`; port menggunakan `""` (mirip `|| ""` di JS). Tambahkan kolom kunci ke `Runtime` kalau router mengeksposnya nanti. Catatan: baris `pub mod applemusic;` perlu ditambah ke `src/canvas/mod.rs` secara manual (mod.rs milik plugin lain).

### `canvas/balogo.rs` (1)

- L6: `catch` di JS (`balogo.js:39-42`) hanya menempel pada `sock.sendMedia`, satu-satunya operasi yang bisa melempar. Di sini media cuma URL dan upload-nya dikerjakan pipeline, jadi cabang `☢` + `src/lib/error.js` tidak punya sumber kegagalan dan tidak ditulis.

### `canvas/buatquotes.rs` (1)

- L3: `@napi-rs/canvas`, `node:fs`, dan font dari GitHub tidak tersedia di `Ctx`/`Reply`; gambar quote tidak dapat dihasilkan. Data background (8 background) dan teks pakai di-port agar bisa diuji. Handler untuk teks lengkap diakhiri `error_template` + `☢` (pola `swm.rs`). Saat canvas tersedia, tambahkan `pub mod buatquotes;` di `src/canvas/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `canvas/facepalm.rs` (1)

- L11: kalau API canvas dan `download_media` tersedia di `Ctx` nanti, port ulang dari `plugins/canvas/facepalm.js` (guard tipe imageMessage, reaksi `🕕`, gambar avatar di (199,112) 235x235, overlay di (0,0), encode PNG).

### `canvas/fakebankjago.rs` (1)

- L6: `Number(saldoClean).toLocaleString("id-ID")` memakai pemisah ribuan titik. Saldo sepanjang puluhan digit meleset: `Number` jadi `1e+20` (dan V8 menulis `100.000.000.000.000.000.000`), sedangkan `thousands_id` memformat digit apa adanya.

### `canvas/fakecall.rs` (1)

- L3: `m.react('🕕')` lalu reaksi `📞`/`❌`/`☢` — `Reply` hanya menyimpan satu reaksi, jadi yang dipegang adalah yang terakhir (`📞` saat berhasil, `❌` saat gagal foto profil/API, `☢` saat error template). `m.download`/`m.quoted.download` → `ctx.download_media()` / `download_quoted_media()`. `sock.profilePictureUrl` → `ctx.sock().foto_profil(sender, "image")`. `form-data` multipart → `HttpCall::post(...).multipart(...)`. Header `form.getHeaders()` tidak ditiru (`HttpGet` tidak punya setter header per-request). `te(...)` → `crate::common::error_template(&ctx)`.

### `canvas/fakedana.rs` (3)

- L4: `nominal` dibersihkan jadi digit saja sebelum masuk URL, jadi `encodeURIComponent` di JS tidak pernah mengubah apa pun. Di sini pemanggilan itu dilewati, bukan ditiru. ponytail: `te()` memakai `m.pushName`, yang tidak ada di `Ctx` — teks balasan memakai fallback `'User'`, sama seperti `error.js` saat `pushName` kosong.
- L8: `te()` memakai `m.pushName`, yang tidak ada di `Ctx` — teks balasan memakai fallback `'User'`, sama seperti `error.js` saat `pushName` kosong.
- L54: di atas `1e21` V8 beralih ke notasi eksponensial (`1e+30`); `thousands_id` tidak, jadi angka sepanjang itu akan tampil berbeda. Praktis tidak terjadi untuk nominal.

### `canvas/fakedev7.rs` (2)

- L3: inti plugin JS adalah render pixel `@napi-rs/canvas` 1080x1080 (background kode Python acak, gradien radial, avatar lingkaran dari gambar yang diunduh, teks melingkar, badge crown/leaf/blue tick). `Kanvas` (`oktz_plugin_api::kanvas`) tidak punya compositing gambar/gradien radial terhadap gambar/font ganda, dan `Gambar` tidak membuka akses piksel mentah — render 1:1 mustahil tanpa dependensi baru (`Cargo.toml` di luar jangkauan port ini). Yang diport: seluruh cabang sebelum render persis JS (help, sumber avatar, pp-kosong); langkah render jatuh ke cabang `catch` JS (`fakedev7.js:171-174`). ponytail: reaksi `🕕` (`fakedev7.js:43`) hilang — `Reply` hanya membawa satu reaksi; yang dipegang reaksi catch (`❌`).
- L13: reaksi `🕕` (`fakedev7.js:43`) hilang — `Reply` hanya membawa satu reaksi; yang dipegang reaksi catch (`❌`).

### `canvas/fakegopay.rs` (2)

- L4: axios di JS mengirim dua header (`User-Agent` dan `Content-Type`). `HttpGet` tidak punya setter header per-request, jadi dua header itu tidak ikut. Dicatat supaya tidak hilang saat mekanismenya ditambahkan. ponytail: `console.error("[FakeGopay Error]", e)` di cabang `catch` tidak punya padanan — `Reply` tidak membawa log.
- L9: `console.error("[FakeGopay Error]", e)` di cabang `catch` tidak punya padanan — `Reply` tidak membawa log.

### `canvas/fakeml.rs` (2)

- L27: blok SALINAN sama persis dengan `ai/tofigurev2.rs`; angkat ke `crate::common` kalau plugin di luar ai/canvas turut memakainya.
- L525: reaksi 🕕 (`fakeml.js:62`) hilang — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup di bawah yang dikirim.

### `canvas/fakeovo.rs` (2)

- L4: axios di JS mengirim dua header (`User-Agent` dan `Content-Type`). `HttpGet` tidak punya setter header per-request, jadi dua header itu tidak ikut. ponytail: `console.error("[FakeOvo Error]", e)` di cabang `catch` tidak punya padanan — `Reply` tidak membawa log.
- L8: `console.error("[FakeOvo Error]", e)` di cabang `catch` tidak punya padanan — `Reply` tidak membawa log.

### `canvas/fakestory.rs` (2)

- L3: inti plugin JS adalah render pixel `@napi-rs/canvas` 720x1150 (background blur `ctx.filter`, kartu rounded-rect + clip, avatar lingkaran, ikon `Path2D` SVG, teks dengan shadow). `Kanvas` tidak punya compositing gambar, blur, clip, maupun `Path2D` — render 1:1 mustahil tanpa dependensi baru (`Cargo.toml` di luar jangkauan port ini). Yang diport: seluruh alur sebelum render persis JS (username, avatar, cek gambar, unduhan); langkah render jatuh ke cabang `catch` JS (`fakestory.js:241-244`). ponytail: reaksi `🕕` (`fakestory.js:204`) hilang — `Reply` hanya membawa satu reaksi; yang dipegang reaksi cabang terakhir (`❌`/`☢`). Helper `aset_pp_kosong`, `get_avatar`, dan `gagal` dipakai bersama `fakestory2/3/4` (satu nama = satu berkas tetap terjaga).
- L11: reaksi `🕕` (`fakestory.js:204`) hilang — `Reply` hanya membawa satu reaksi; yang dipegang reaksi cabang terakhir (`❌`/`☢`). Helper `aset_pp_kosong`, `get_avatar`, dan `gagal` dipakai bersama `fakestory2/3/4` (satu nama = satu berkas tetap terjaga).

### `canvas/gura.rs` (1)

- L13: blok SALINAN `uploadImage` sama persis dengan `canvas/profileig.rs` dan empat berkas `ai/*`; angkat ke `crate::common` kalau plugin lain turut memakainya.

### `canvas/igstory.rs` (1)

- L13: kalau API canvas penuh tersedia di `Ctx` nanti, port ulang `generateIgStory` dari `plugins/canvas/igstory.js` (latar storyig.png, foto utama blur 28px cover+blur, avatar bundar, ikon recolor `source-in`, crop 1400px bawah).

### `canvas/igstoryimg.rs` (1)

- L13: kalau API canvas penuh tersedia di `Ctx` nanti, port ulang `generateIgimg` dari `plugins/canvas/igstoryimg.js` (bg igimg.png 898x1600, foto zona (38,136)-(860,912) radius 20, PP bundar 80px, nama auto-fit 25→16px, username 17px).

### `canvas/iqc.rs` (1)

- L6: cabang `catch` di JS (`iqc.js:38-42`) menangkap kegagalan `sock.sendMessage`. Pengiriman sekarang dikerjakan pipeline, jadi cabang itu tidak lagi punya pemicu di dalam handler. `pluginConfig` tidak punya key `example`, jadi `example` kosong — bukan diisi tebakan.

### `canvas/iqc2.rs` (2)

- L7: `m.pushName` tidak ada di `Ctx`. `serialize.js:760` memakai `pushName || "Unknown"`, jadi teks help memakai `"Unknown"` sebagai fallback — bukan `'User'` seperti `error.js`. ponytail: cabang `catch` di JS (`iqc2.js:45-49`) menangkap kegagalan `sock.sendMessage`. Pengiriman sekarang dikerjakan pipeline, jadi cabang itu tidak lagi punya pemicu di dalam handler. `pluginConfig` tidak punya key `example`, jadi `example` kosong — bukan diisi tebakan.
- L11: cabang `catch` di JS (`iqc2.js:45-49`) menangkap kegagalan `sock.sendMessage`. Pengiriman sekarang dikerjakan pipeline, jadi cabang itu tidak lagi punya pemicu di dalam handler. `pluginConfig` tidak punya key `example`, jadi `example` kosong — bukan diisi tebakan.

### `canvas/iqcpink.rs` (2)

- L3: inti plugin JS adalah render pixel `@napi-rs/canvas` (background catbox 906x1736, font Inter yang diunduh, emoji Apple dari JSON GitHub, bubble rounded-rect + shadow, wrapping dengan `measureText`). `Kanvas` (`oktz_plugin_api::kanvas`) tidak punya compositing gambar, font ganda, shadow, maupun `measureText`, dan `Gambar` tidak membuka akses piksel mentah — render 1:1 mustahil tanpa dependensi baru (`Cargo.toml` di luar jangkauan port ini). Yang diport: cabang validasi persis JS; langkah render jatuh ke cabang `catch` JS (`iqcpink.js:490-498`). ponytail: reaksi `🕕` (`iqcpink.js:468`) hilang — `Reply` hanya membawa satu reaksi, jadi yang dipegang reaksi penutup (`❌` pada cabang catch; `✅` tidak terjangkau selama render belum ada).
- L12: reaksi `🕕` (`iqcpink.js:468`) hilang — `Reply` hanya membawa satu reaksi, jadi yang dipegang reaksi penutup (`❌` pada cabang catch; `✅` tidak terjangkau selama render belum ada).

### `canvas/kalender.rs` (1)

- L12: kalau API canvas penuh tersedia di `Ctx` nanti, port ulang `renderTahun`/`renderBulan` dari `plugins/canvas/kalender.js` (3 kolom, skala kartu 1.6/2.1, statistik 17/8/25).

### `canvas/musiccard.rs` (1)

- L3: `ImageUploadService` (pixhost.to) dan `api.nexray.eu.cc/canvas/musiccard` untuk menghasilkan gambar kartu tidak tersedia di `Ctx`/`Reply`; handler untuk gambar+teks lengkap diakhiri `error_template` + `☢` (pola `swm.rs`). Unduh gambar (tersedia via `Ctx::download_media`) dan pemisahan judul/artis di-port agar bisa diuji. Saat unggah + API kartu tersedia, tambahkan `pub mod musiccard;` di `src/canvas/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `canvas/pakustad.rs` (1)

- L9: `config.APIkey?.cuki` tidak ada di `Ctx`, jadi dikunci `""` — pola yang sama dengan `crate::ai::anime_gen` dan `crate::info::jadwalbola`. Efeknya cabang "API Key belum diset" (`pakustad.js:34`) selalu benar dan jalur HTTP setelahnya tidak terjangkau sampai router menyediakan config.

### `canvas/profileig.rs` (2)

- L22: blok SALINAN sama persis dengan `ai/tofigurev2.rs`; angkat ke `crate::common` kalau plugin di luar ai/canvas turut memakainya.
- L529: reaksi 🕕 (`profileig.js:30`) hilang — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup di bawah yang dikirim.

### `canvas/sroast.rs` (2)

- L4: Tidak ada API canvas (gambar, font, teks) yang diekspos ke plugin di registry Rust, dan tidak ada klien HTTP untuk memuat font dari URL eksternal. Sebagian gambar dari plugin ini tidak dapat dicerminkan. Handler di bawah hanya menangani guard input dan jatuh ke balasan error saat bagian canvas harus dieksekusi — ini penyimpangan disengaja yang perlu diimplementasikan ulang saat API canvas tersedia.
- L20: Implementasi canvas asli (createCanvas, wrapText, drawText, encode PNG, GlobalFonts.register dari catbox.moe) tidak tersedia di registry Rust. Fungsi ini adalah placeholder yang selalu gagal.

### `canvas/starboy.rs` (1)

- L11: kalau media intake dan uploader tersedia di `Ctx` nanti, port ulang dari `plugins/canvas/starboy.js` (reaksi `🕕` → `✅`/`☢`).

### `canvas/topixel.rs` (2)

- L3: inti plugin JS adalah loop piksel mentah sharp (`image.raw().toBuffer()` + rata-rata blok per `block` piksel, `topixel.js:26-81`). `Gambar` (`oktz_plugin_api::gambar`) tidak membuka akses piksel mentah dan crate `image` bukan dependensi langsung `oktz-plugins` (`Cargo.toml` di luar jangkauan port ini), jadi render 1:1 mustahil. Yang diport: seluruh cabang validasi + `getBlock` persis JS; langkah pixelate jatuh ke cabang `catch` JS (`topixel.js:118-121`). ponytail: reaksi `🕕` (`topixel.js:103`) hilang — `Reply` hanya membawa satu reaksi; yang dipegang reaksi catch (`☢`).
- L11: reaksi `🕕` (`topixel.js:103`) hilang — `Reply` hanya membawa satu reaksi; yang dipegang reaksi catch (`☢`).

### `canvas/ttqc.rs` (1)

- L12: kalau API canvas penuh tersedia di `Ctx` nanti, port ulang `render` dari `plugins/canvas/ttqc.js` (canvas 1080x2280 @2x, PP bundar, bubble putih wrap 520px, menu 6 item Font Awesome).

### `canvas/watercolortext.rs` (1)

- L7: `config.APIkey.cuki` tidak ada di `Ctx`, jadi dikunci `""` — pola yang sama dengan `crate::ai::anime_gen` dan `crate::info::jadwalbola`. Efeknya cabang "API key belum diset" (`watercolortext.js:35`) selalu benar dan jalur HTTP setelahnya tidak terjangkau sampai router menyediakan config.


## tools — tools (34 catatan / 28 file)

### `tools/am_data.rs` (2)

- L4: JS mengirim header `Authorization: Bearer ${KEY}` dan `Content-Type: application/json`; `UreqHttp::call` mempertahankan header kustom melalui `HttpCall::call`. Stub bawaan (`NoHttp`) tidak pernah jaringan, jadi jalur gagal diuji lewat `Err`.
- L44: tanpa `icu4x` tidak bisa meniru locale id-ID byte-per-byte; format yang dihasilkan adalah representasi yang sama secara struktur (`DD Month YYYY`). Bulan dalam Bahasa Indonesia.

### `tools/carbon.rs` (1)

- L3: pengiriman tidak bisa di-port. `buildCarbonUrl` + `THEMES`/`FONTS` ikut hilang bersamanya: satu-satunya pemanggilnya adalah `@microlink/mql`, yaitu layanan screenshot berbasis browser yang tidak ada di `HttpGet` (yang cuma bisa GET/POST biasa). Reaksi 🕕 dan kartu usage — dua hal yang murni teks — tetap dipertahankan. Teks sumber diambil `m.text || m.quoted?.text` (`quoted` menang kalau `m.text` kosong).

### `tools/caribug.rs` (1)

- L6: JS bereaksi dua kali (`🕕` awal, lalu `✅`/`❌` akhir). `Reply::with_reaction` cuma satu emoji, jadi cuma reaksi akhir yang ikut.

### `tools/cekxl.rs` (1)

- L3: JS memakai `axios.get(..., { timeout: 30000 })`; `UreqHttp` punya timeout global sendiri, jadi timeout per-permintaan tidak ditiru. Balasan tidak dikutip (JS `m.reply(txt.trim())` polos).

### `tools/cjstoesm.rs` (1)

- L13: `sock.sendCodeBlock` tidak ada di API port, jadi hasil konversi dikirim sebagai teks biasa. Cabang `catch` JS tidak diport — konversi string di Rust tidak bisa gagal di tengah jalan seperti `replace` JS.

### `tools/deploy.rs` (1)

- L12: kalau `config.vercel` ditambahkan ke `Runtime` nanti, port ulang dari `plugins/tools/deploy.js` (guard token, reaksi `🕕`, ekstrak ZIP/HTML, POST ke `https://api.vercel.com/v13/deployments`, GET domains, fallback ke `${name}.vercel.app`). NOTE: baris `pub mod deploy;` perlu ditambah ke `tools/mod.rs` secara manual (mod.rs dilarang disentuh karena sudah milik plugin lain).

### `tools/emojitoanimasi.rs` (2)

- L3: `contextInfo: saluranCtx()` (`src/lib/context.js`) tidak tersedia di `Reply::Sticker`, dan reaksi ganda JS (`🎭` lalu `✅`) disatukan jadi satu reaksi akhir (`✅`) seperti `phisingataubukan` (`with_reaction` hanya satu emoji). Saat `contextInfo` tersedia di `Reply`, pasang ulang.
- L87: dicatat di di atas.

### `tools/esmtocjs.rs` (1)

- L14: `sock.sendCodeBlock` tidak ada di API port, jadi hasil konversi dikirim sebagai teks biasa. Cabang `catch` JS tidak diport — konversi string di Rust tidak bisa gagal di tengah jalan.

### `tools/hd.rs` (1)

- L44: setelah handler bisa async / `Reply` menunda render, pasangkan alur: unduh media → POST multipart ke ihancer → kirim byte hasil sebagai image.

### `tools/hdvid.rs` (1)

- L10: JS mengirim kartu `🎞️ *PROSES ENHANCE DIMULAI*` sebelum ffmpeg; di sini tidak dikirim supaya satu pemanggilan handler tetap satu balasan (pakai `Reply::Sequence` kalau balasan progres dibutuhkan). `fileName: HDVID-<ms>.mp4` tidak ada di `Reply::video`; `mimetype` sudah dipasang.

### `tools/imgtoprompt.rs` (1)

- L3: scraping sebenarnya ada di crate `oktz_scraper` (modul `img2prompt`), jadi plugin ini cuma panggil fungsi itu — tidak menyalin logika scraper ke sini. Unduh media: `m.download()`/`m.quoted.download()` → `ctx.download_media()` / `download_quoted_media()`. `Buffer.isBuffer` / validasi kosong → cek `bytes.len() > 0`. `result.status === 'eror' ||result.prompt` → teks gagal. `te(...)` → `crate::common::error_template(&ctx)`.

### `tools/invoicemaker.rs` (1)

- L6: konteks pengiriman (newsletter `contextInfo`) dari JS tidak ada di `Reply`, jadi dikirim sebagai gambar biasa dengan caption. Kunci API `neoxr` dibaca dari `ctx.config().api_keys.neoxr`.

### `tools/izen.rs` (1)

- L3: `m.react("⏳")` dan `m.react("✅")` terjadi pada satu pesan, sementara `Reply::with_reaction` hanya satu emoji — hanya `✅` yang dibawa. `encode_uri_component` disalin ke sini (duplikasi dari `phisingataubukan.js`) karena rule port tidak menyentuh file plugin lain. Catatan: baris `pub mod izen;` perlu ditambah ke `tools/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `tools/nikparser.rs` (1)

- L321: JS `new Date(y, m, d)` akan rollover untuk bulan > 12 atau hari di luar jumlah hari bulan itu (mis. "99/99/9999"), sedangkan di sini nilai tak masuk akal dibuang supaya tidak melahirkan tanggal karangan. Naikkan ke normalisasi penuh kalau data nyata pernah keluar dari rentang wajar.

### `tools/nulis.rs` (1)

- L11: kalau API canvas dan aset tersedia di `Ctx` nanti, port ulang dari `plugins/tools/nulis.js` (wrap teks lewat `measureText`, tanggal + hari di kanan atas kertas, buffer JPEG).

### `tools/pastebin.rs` (1)

- L35: jalur POST-nya sendiri sudah bisa direplikasi sekarang lewat `HttpCall::post(url).form(body)` (`registry.rs:1202-1247`); yang menahan port adalah kunci dev + preview di atas. Begitu kunci bisa dibaca dari `Ctx::config()`/env dan `Reply` punya varian preview, implementasi bisa ditulis tanpa menyentuh berkas ini lagi.

### `tools/phisingataubukan.rs` (3)

- L6: dua hal tidak bisa di-cerminkan persis: - `m.pushName` tidak ada di `Ctx`, jadi cabang "halo" memakai fallback `'User'` yang sama seperti `te()` di `src/lib/error.js`. - JS bereaksi dua kali pada satu pesan (`m.react("🕕")` di awal, lalu `✅`/`❌` di akhir). `Reply::with_reaction` hanya membawa satu reaksi, jadi reaksi penutup yang dikirim dan `🕕` hilang — sama seperti batas "satu balasan" yang sudah dicatat di HANDOFF §3.5 butir 6.
- L112: hasil `toLocaleString` bergantung zona waktu runtime Node. Port ini mengunci `Asia/Jakarta` lewat `crate::common::jakarta_parts`. Kalau bot dijalankan di zona lain, ini ikut bergeser.
- L118: V8 juga mengoercive array jadi `Date` dengan aturan aneh (`new Date([1,2])` -> 2001-02-01). Array dan object tidak direproduksi karena tidak mungkin keluar dari API yang dipanggil plugin ini; keduanya menghasilkan `"Invalid Date"`.

### `tools/ptv.rs` (1)

- L7: WA `ptv: true` (bentuk video-note bulat) tidak ada padanannya di `Reply::Video` (hanya `gif_playback`) — dikirim sebagai video loop tanpa suara (efek visual sama seperti JS yang juga memakai `gifPlayback: true`). Upgrade path: tambah flag `ptv` di `Reply::Video` + wire `videoNote`.

### `tools/qrcustom.rs` (1)

- L9: DUA hal JS tidak di-port: - Komposit logo (`sharp(logo).resize(120,120)` + `composite centre`, `qrcustom.js:38-41`): `oktz_plugin_api::gambar` tidak mengekspos piksel mentah (hanya resize/encode), jadi logo dari `m.download()` / `m.quoted.download()` tidak bisa ditempel ke matriks QR. QR dikirim tanpa logo. - Reaksi `📱` (`qrcustom.js:51`) dibuang: membungkus gambar dengan `with_reaction` membuat gambar tidak pernah di-upload (lihat `random/cecanindo.rs`). Pesan `🕕` tetap dikirim lebih dulu seperti JS. Input di-encode sebagai SATU segmen BYTE. JS mengoptimasi segmen (numeric/alphanumeric) lewat `Segments.fromString`; hasil scan tetap sama, hanya susunan bit internalnya yang berbeda.

### `tools/removebg.rs` (1)

- L4: `pixa()` JS menormalkan gambar ke JPEG dengan `sharp` sebelum dikirim (dan menyimpannya ke berkas sementara). `sharp` tidak ada di crate scraper, jadi byte dari WhatsApp dikirim apa adanya — gambar WA hampir selalu JPEG. Tambahkan normalisasi kalau suatu saat butuh webp/png/heic.

### `tools/sendngl.rs` (1)

- L3: `sendNgl` di JS (`src/lib/stalker-fallback.js`) menjalankan fallback cuki → nexray, dan reaksi `🎴` sebelum pemakaian dihapus karena `Reply` hanya mampu satu reaksi (yang penutup). Semua jaringan lewat `ctx.http`; tidak menyalin logika `stalker-fallback.js` ke sini, melainkan meniru alurnya (cuki dulu, nexray cadangan) lewat dua permintaan HTTP.

### `tools/setpp.rs` (1)

- L6: `sock.user?.id` (JID bot) tidak ada di `Ctx` — tapi `SockKlien::set_foto_profil` hanya mendukung foto profil bot sendiri untuk jid non-grup, jadi argumen jid dilewati apa adanya (paritas tetap: foto bot yang berubah). Upgrade path: ekspos `bot_id` di `Ctx` untuk validasi jid.

### `tools/ssweb.rs` (1)

- L3: opsi `--mobile`/`--hp` dan penambahan `https://` di-port apa adanya. Reaksi ganda JS (`🕕` lalu `✅`) disatukan jadi satu reaksi akhir `✅`, karena `Reply::with_reaction` hanya membawa satu emoji.

### `tools/styleteks.rs` (2)

- L3: tombol `cta_copy` tidak punya padanan di `Reply` (hanya teks yang bisa dikirim), jadi hasil dikirim sebagai balasan teks biasa — kutipan `quoted: m` tetap ikut karena itu bawaan `Reply::Text`. Satu `Reply` juga hanya membawa satu reaksi; reaksi penutup `✅`/`❌` yang dipertahankan, reaksi pembuka `🕕` dilewatkan.
- L792: JS memecah per code unit UTF-16 (emoji jadi surrogate tunggal yang hanya bisa dikirim sebagai CESU-8); di sini per `char` supaya hasilnya tetap UTF-8 valid.

### `tools/tempmail.rs` (1)

- L3: `TempMailCreate()` / `TempMailInbox()` dipanggil lewat crate `oktz_scraper::tempmail` (bukan menyalin logika scraper). Simpan email ke store user lewat `Mutation::SetUser`. `m.react("🕕")` / `✅` / `❌` / `☢` / `📭` — satu reaksi saja yang dibawa (`✅` atau error). Catatan: baris `pub mod tempmail;` perlu ditambah ke `tools/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `tools/toimg.rs` (1)

- L6: tidak terekspos) — sticker animasi tetap diproses sebagai gambar; upgrade path: ekspos `isAnimated` di `IncomingMedia` untuk menyarankan `.tovideo` seperti JS.

### `tools/txt2qr.rs` (1)

- L34: `Ctx` tidak bawa kan QR renderer (qrcode JS). Pembuatan QR membutuhkan crate gambar yang belum ada di dependensi crate ini. Tambahkan renderer QR di sini (atau panggil lewat `oktz_scraper`) bila tersedia. Sementara, balas pesan teks yang meniru output JS.

### `tools/videotranscribe.rs` (2)

- L6: `crypto.randomUUID()` di JS diganti hex acak via `rand`; format track_id tetap `{hex}_{ms}`.
- L17: `HEADERS` JS (termasuk user-agent) tidak bisa dipasang lewat `HttpGet::post_json`; endpoint tetap menerima tanpa header itu. Naik kelas kalau server mulai menolak: tambah varian `post_json_headers`.


## fun — fun (27 catatan / 16 file)

### `fun/cekpacar.rs` (1)

- L3: `Ctx` hanya membawa store untuk `sender` (`user_data`) dan untuk member grup (`Participant::data`). Target dari quote/mention/angka yang TIDAK ada di roster diperlakukan seperti `db.getUser(...) || {}` — sama dengan user yang belum pernah ada, jadi jawabannya "tidak punya pasangan". Akibatnya `isMutual` hanya bisa `true` untuk pasangan yang ada di roster.

### `fun/fuckmylife.rs` (3)

- L3: `Reply::Reaction` hanya bisa membawa satu emoji, sedangkan JS bereaksi `🕕` lebih dulu lalu menggantinya dengan `❌`/`✅`/`☢`. Yang ikut hanya reaksi terakhir. ponytail: `config.APIkey?.neoxr` (`config.js:327` = `process.env.APIKEY_NEOXR || ""`) tidak dibawa `Ctx`, jadi kuncinya dikunci kosong — pola yang sama dipakai `info/jadwalbola.rs` dan `ai/anime-gen.rs`. ponytail: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User` seperti `pushName || 'User'` di JS saat `pushName` kosong.
- L7: `config.APIkey?.neoxr` (`config.js:327` = `process.env.APIKEY_NEOXR || ""`) tidak dibawa `Ctx`, jadi kuncinya dikunci kosong — pola yang sama dipakai `info/jadwalbola.rs` dan `ai/anime-gen.rs`. ponytail: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User` seperti `pushName || 'User'` di JS saat `pushName` kosong.
- L11: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User` seperti `pushName || 'User'` di JS saat `pushName` kosong.

### `fun/gachawaifu.rs` (5)

- L9: Yang tidak bisa dicerminkan persis (semua ditandai di tempat pemakaiannya): 1. Pool `data/waifu/*.js` belum ter-port → `WAIFU_ENTRIES` kosong, jadi cabang gacha selalu jatuh ke `te()` + reaksi ☢ seperti `waifupool.rs`. 2. Kartu `viewOnceMessage` + `nativeFlowMessage` + gambar Pinterest (`getWaifuImage`/`getBuffer`) tidak ada di `Reply` → balasan teks polos dengan isi `body` yang sama; opsi tombol dibuang, `imageUrl` di store memakai konstanta fallback `gachawaifu.js:40`. 3. Kartu level-up canvas di `checkAndNotifyLevelUp` tidak ikut — hanya `exp`/`rpg` yang tersimpan (`apply_exp`). 4. `delete user.waifu` → `waifu: null` (SetUser merge, bukan delete). 5. Satu `Reply` hanya bisa satu reaksi — urutan JS (`🕕` lalu `✅`) dipadatkan ke reaksi TERAKHIR. `pub mod gachawaifu;` perlu ditambah manual ke `src/fun/mod.rs`.
- L867: tanggal yang tidak bisa diparse di JS jadi `NaN` dan hasilnya `NaN` juga; di sini jatuh ke `0`.
- L1065: `repeat` JS melempar untuk n negatif (anger negatif yang tersimpan rusak); di sini n dijepit 0..=10.
- L1513: `initStats` sudah menulis `user.waifuStats` di memori JS yang ikut ter-flush nanti; di sini tidak ada mutasi.
- L1667: tier tak dikenal bikin JS melempar (`DOWRY[tier]` undefined); di sini dowry-nya dilewati. Tier selalu valid dari pool.

### `fun/husbualbum.rs` (1)

- L9: JS memanggil `db.getUser(m.sender)` dan `if (!user) return;` tanpa balasan sama sekali. `Ctx::user_data` = `Value::Null` untuk user yang belum pernah ada, jadi user null diperlakukan sebagai "album kosong" — pesan yang diberikan JS, bukan diam. Bedanya nyata (user baru dapat cara mulai gacha) dan disengaja: diam tidak pernah bisa dit touted.

### `fun/husbupool.rs` (1)

- L41: pindahkan data husbu ke Rust (atau sertakan sebagai embedded data di crate ini), lalu implementasikan pencarian + paginasi dalam `Reply::Text` / `Reply::Interactive`.

### `fun/jodoh.rs` (1)

- L3: dua hal tidak bisa dicerminkan persis: 1. `sock.user.id` tidak ada di `Ctx`, jadi bot sendiri TIDAK bisa dibuang dari kandidat. Kalau bot adalah anggota grup, dia bisa jadi hasil jodoh. Tambahkan `bot_jid: String` ke `Ctx` kalau router mau membuangnya. 2. `sort(() => Math.random() - 0.5)` di JS adalah shuffle bias. Di sini Fisher-Yates — tujuannya "ambil dua orang berbeda", dan keduanya memenuhi itu.

### `fun/mimpi.rs` (1)

- L14: `m.pushName` tidak ada di `Ctx`, jadi nama bawaannya hanya nomor pengirim (`m.sender.split('@')[0]`), sama seperti cabang yang tidak punya args di JS (`mimpi.js:101`).

### `fun/puisi.rs` (3)

- L3: `Reply::Reaction` hanya bisa membawa satu emoji, sedangkan JS bereaksi `🕕` lebih dulu lalu menggantinya dengan `❌`/`✅`/`☢`. Yang ikut hanya reaksi terakhir. ponytail: `config.APIkey?.neoxr` (`config.js:327` = `process.env.APIKEY_NEOXR || ""`) tidak dibawa `Ctx`, jadi kuncinya dikunci kosong. ponytail: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User`.
- L7: `config.APIkey?.neoxr` (`config.js:327` = `process.env.APIKEY_NEOXR || ""`) tidak dibawa `Ctx`, jadi kuncinya dikunci kosong. ponytail: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User`.
- L10: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User`.

### `fun/putus.rs` (2)

- L4: dua hal tidak bisa dicerminkan persis: 1. `delete senderData.fun.pasangan` tidak ada padanannya di `Mutation::SetUser` (itu merge `??`), jadi `pasangan` ditulis `null`. Pembaca JS `userData.fun?.pasangan` memperlakukan `null` sama dengan tidak ada — tapi byte-nya berbeda. 2. `db.getUser(exPartner)` hanya bisa dijawab kalau pasangannya masih ada di roster grup (`Participant::data`). Untuk pasangan yang tidak ada di grup, `isMutual` selalu false dan pencatatan di sisi lama ikut hilang — sama seperti `db.getUser` untuk user yang belum pernah ada.
- L21: `Mutation::SetUser` yang mengosongkan pasangan — lihat catatan.

### `fun/senja.rs` (3)

- L3: `Reply::Reaction` hanya bisa membawa satu emoji, sedangkan JS bereaksi `🕕` lebih dulu lalu menggantinya dengan `❌`/`✅`/`☢`. Yang ikut hanya reaksi terakhir. ponytail: `config.APIkey?.neoxr` (`config.js:327` = `process.env.APIKEY_NEOXR || ""`) tidak dibawa `Ctx`, jadi kuncinya dikunci kosong. ponytail: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User`.
- L7: `config.APIkey?.neoxr` (`config.js:327` = `process.env.APIKEY_NEOXR || ""`) tidak dibawa `Ctx`, jadi kuncinya dikunci kosong. ponytail: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User`.
- L10: `Ctx` tidak membawa `m.pushName`, jadi `{pushName}` pada template error selalu jatuh ke `User`.

### `fun/siapa.rs` (1)

- L8: dua langkah JS TIDAK di-port: penyaringan `id== sock.user?.id?.split(':')[0] + '@s.whatsapp.net'` (butuh `sock`, bot sendiri bisa ikut terundi) dan cabang `catch` yang memanggil `te(m.prefix, m.command, m.pushName)` — handler Rust tidak bisa melempar, jadi tidak ada error yang perlu ditangkap.

### `fun/terima.rs` (1)

- L10: kalau store grup dan `sock` tersedia di `Ctx` nanti, port ulang dari `plugins/fun/terima.js` (guard target tembakan, tulis `pasangan`/`jadiPacar`/`terimaCount` ke dua user, hapus session, reaksi `💕`).

### `fun/tod.rs` (1)

- L4: JS memanggil `sock.groupMetadata(m.chat)` dan, kalau roster kosong, `randomMember.jid` melempar `TypeError` yang ditangkap `catch`. Di sini roster kosong langsung menghasilkan balasan template error yang sama — tidak ada jalur yang bisa "diam-diam" membalas tanpa target.

### `fun/top.rs` (1)

- L15: JS memfilter JID bot sendiri dari kandidat (`id== sock.user?.id`, `top.js:38`) dan `GroupRoster` tidak menyimpan JID bot, jadi bot bisa ikut masuk daftar. Tambahkan `bot_jid` ke `GroupRoster` kalau ini terasa mengganggu.

### `fun/waifualbum.rs` (1)

- L9: JS memanggil `db.getUser(m.sender)` dan `if (!user) return;` tanpa balasan sama sekali. `Ctx::user_data` = `Value::Null` untuk user yang belum pernah ada, jadi user null diperlakukan sebagai "album kosong" — pesan yang diberikan JS, bukan diam. Bedanya nyata (user baru dapat cara mulai gacha) dan disengaja: diam tidak pernah bisa dit touted.

### `fun/waifupool.rs` (1)

- L3: `searchPool` (`data/waifu/index.js`) dan `sock.sendButton` (tombol interaktif) tidak tersedia di `Ctx`/`Reply`; handler diakhiri `error_template` + `☢` (pola `swm.rs`). Emoji tier, aritmatika paginasi, parsing nav/query, dan rendering satu entri di-port agar bisa diuji. Saat data pool + tombol tersedia, tambahkan `pub mod waifupool;` di `src/fun/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).


## sticker — sticker (25 catatan / 14 file)

### `sticker/attp.rs` (1)

- L4: JS memiliki dua fallback yang tidak tersedia di registry Rust: (1) canvas `@napi-rs/canvas` untuk teks animasi lokal, dan (2) `addExifToWebp` dari `src/lib/exif.js`. Keduanya tidak dapat dicerminkan, sehingga saat API gagal plugin jatuh ke balasan error (mirip cabang `catch` JS) dan frame animasi lokal tidak dikirim.

### `sticker/attp2.rs` (1)

- L3: `@napi-rs/canvas`, `addExifToWebp` (encoder WebP + EXIF), dan `sock.sendMessage({sticker})` tidak tersedia di `Ctx`/`Reply`; sticker animasi tidak dapat dihasilkan. Teks pakai dan jumlah frame di-port agar bisa diuji. Handler untuk teks diakhiri `error_template` + `☢` (pola `swm.rs`). Saat canvas + encoder WebP tersedia, tambahkan `pub mod attp2;` di `src/sticker/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `sticker/bratextra.rs` (1)

- L54: tambahkan crate canvas (atau port renderer teks ke `oktz_scraper`) dan unggah byte sticker, lalu replikasikan semua 13 template + loop perkecil font.

### `sticker/emojimix.rs` (1)

- L3: JS memiliki fallback canvas (`@napi-rs/canvas`) saat API gagal, tetapi tidak ada API canvas yang diekspos ke plugin di registry Rust, dan crate `regex` tidak tersedia (tidak boleh mengubah Cargo.toml). Deteksi emoji menggunakan pendekatan non-ASCII sederhana; fallback canvas tidak dapat dicerminkan dan jatuh ke balasan error (mirip cabang `catch` JS).

### `sticker/linesticker.rs` (3)

- L7: `sock.sendImageAsSticker` / `sendVideoAsSticker` dengan `packname`/`author` tidak tersedia di `Reply::Sticker`; metadata itu tidak punya kanal sama sekali di registry Rust. Sticker dikirim apa adanya lewat `Reply::Sticker`, dan video animasi juga dikirim sebagai sticker (tidak ada varian "video sticker"). ponytail: `Reply::Sequence` tidak boleh dibungkus `Reaction` — pipeline merender `Sequence` lewat loop, bukan `render_reply`, jadi reaksi ✅/☢ pada jalur multi-pesan tidak ikut (pola yang sama dipakai `download/tiktokdl2.rs`).
- L13: `Reply::Sequence` tidak boleh dibungkus `Reaction` — pipeline merender `Sequence` lewat loop, bukan `render_reply`, jadi reaksi ✅/☢ pada jalur multi-pesan tidak ikut (pola yang sama dipakai `download/tiktokdl2.rs`).
- L186: `Reply::Sequence` tidak boleh dibungkus `Reaction` (pipeline merender `Sequence` lewat loop, bukan `render_reply`), jadi reaksi ✅/☢ tidak ikut di jalur ini — sama seperti `download/tiktokdl2.rs`.

### `sticker/pinpack.rs` (3)

- L11: - `sock.sendStickerPack` tidak ada di `Reply`, jadi sticker dikirim satu per satu lewat `Reply::sequence`; percabangan "Pack gagal, mengirim satu per satu..." (`:115-119`) tidak pernah tersentuh. - `sharp(...).resize(512, 512, { fit: "contain", background: transparent })` tidak punya padanan persis di `oktz_plugin_api::gambar`; gambar dikonversi ke WebP apa adanya. - `addExifToWebp` tidak ada, jadi metadata packname/author tidak ikut.
- L141: `Reply::Sequence` tidak boleh dibungkus `Reaction` (pipeline merender `Sequence` lewat loop, bukan `render_reply`), jadi reaksi ✘ tidak ikut — sama seperti `download/tiktokdl2.rs`.
- L155: reaksi ✓ tidak bisa menempel pada `Reply::Sequence` — lihat catatan di atas.

### `sticker/qc.rs` (2)

- L12: render quote-nya (`qc.js:114-155`) memakai `@napi-rs/canvas`: kanvas 512x512 diisi warna latar, teks putih dibungkus baris (lebar maksimal 460px, font 36px), lalu `~ <nama>` di pojok kanan bawah. Tidak ada renderer teks di Rust dan dependensi repo tidak boleh diubah, jadi jalur setelah guard berakhir di cabang `catch` JS: `error_template` + `☢` (`qc.js:165-167`).
- L170: Render quote butuh canvas — lihat modul.

### `sticker/smeme-animated.rs` (1)

- L15: - `uploadImage` (`src/lib/uploader.js:14-40`) mencoba 9 host keyless berurutan. Di sini hanya host pertama yang dipakai — Litterbox (`src/lib/upload-providers.js:21-37`) — karena bentuk multipart-nya persis sama dan host lain butuh parsing HTML/cookie khusus. - `sock.sendImageAsSticker` tidak ada; yang dipakai `Reply::Sticker`. Metadata `packname`/`author` (`config.sticker`) tidak punya kanal di `Reply::Sticker`, sama seperti plugin sticker lain di repo ini.

### `sticker/smeme.rs` (1)

- L16: - JS mengubah gambar ke 512x512 `fit: "contain"` dengan latar transparan sebelum diunggah (`smeme.js:58-67`). `oktz_plugin_api::gambar` tidak punya mode "contain + pad transparan" (hanya `cover` dan `inside`), jadi gambar diunggah apa adanya (dikonversi ke PNG kalau bisa). Memegen.link sendiri menyesuaikan ukuran latar, jadi hasil akhirnya hampir sama. - `addExifToWebp` tidak ada di Rust, jadi metadata packname/author pada sticker tidak ikut — sama seperti plugin sticker lain di repo ini.

### `sticker/smemevid.rs` (1)

- L14: render meme-nya butuh dua hal yang tidak ada di Rust: (1) teks digambar di kanvas 512x512 (`smemevid.js:87-117`) — tidak ada renderer teks, dan (2) video di-overlay dengan ffmpeg lalu dikirim sebagai video-sticker (`smemevid.js:119-144`) — `Reply` tidak punya varian "video sticker", jadi yang dikirim `Reply::video`. Kalau ffmpeg tidak ada atau gagal, video asli dikirim apa adanya.

### `sticker/stickerly.rs` (2)

- L7: empat hal JS yang tidak bisa dicerminkan apa adanya: - `sock.sendStickerPack(m.chat, urls, m, {...})` (`stickerly.js:147-154`) tidak ada padanannya; sticker dikirim per URL lewat `Reply::Sequence` (maks 20, sama seperti `slice(0, 20)` di `stickerly.js:144`). Metadata pack (`name`, `packname`, `publisher`, `author`, `description`, `emojis`) beserta fallback `"Sticker.ly Pack"`/`config.sticker.author`/`"Bot"` (`stickerly.js:142-143`) hilang karena `Reply::Sticker` tidak membawa metadata pack. - Reaksi berurutan (`🕕` di `stickerly.js:22,115` lalu `❌`/`✅`/`☢`) tidak mungkin: satu `Reply` membawa satu reaksi. Jalur teks mengirim reaksi TERAKHIR (konvensi repo, mis. `rpg/jualan.rs`); jalur sticker bahkan itu tidak bisa — butir berikut. - Reaksi tidak bisa dibarengkan dengan `Reply::Sequence`: jalur reaksi pipeline merender isinya lewat `render_reply`, dan `Sequence` ditolak di sana (`oktz-wa/src/pipeline.rs:2999-3008`, penolakan di `:1647-1649`) — seluruh balasan jadi error. Sticker pack dikirim sebagai `Sequence` TANPA reaksi `✅`; mengirim sticker lebih penting daripada reaksi. - `stickerlyAnswerHandler` (`stickerly.js:74-112`) ikut diport, tapi BELUM BISA JALAN di produksi: `Router::answer` membangun `Ctx` lewat `oktz-router/src/lib.rs:170-196` tanpa pembungkus `PluginCtxSource::fill` (`oktz-wa/src/pipeline.rs:2151-2170`), jadi `user_data` Null dan `http`/`sessions` default; selain itu pipeline jalur answer (`oktz-wa/src/pipeline.rs:2881-2889`) tidak menulis mutasi balasan. Perbaikannya ada di crate `lib/` — di luar berkas ini. `delete user.stickerly_session` (`stickerly.js:86,95,107`) ditulis sebagai `null` karena `Mutation::SetUser` menggabung, bukan menghapus key. Untuk pembacaan JS semantiknya sama: `!user.stickerly_session` juga falsy untuk `null`.
- L69: objek/array dicetak sebagai JSON, sedangkan JS mencetak `[object Object]` — bentuk yang tidak muncul di data API ini.

### `sticker/stickerpack.rs` (3)

- L14: - `sock.sendStickerPack` tidak ada di `Reply`, jadi sticker dikirim satu per satu lewat `Reply::sequence`. Konsekuensinya pesan "Pack gagal, mengirim satu per satu..." (percabangan `catch` JS) tidak pernah muncul — paket selalu dikirim per satu. - `sharp(...).resize(512, 512, { fit: "contain", background: transparent })` tidak punya padanan persis di `oktz_plugin_api::gambar` (hanya `cover` dan `inside`); gambar dikonversi ke WebP apa adanya. - `addExifToWebp` tidak ada, jadi metadata packname/author tidak ikut.
- L198: `Reply::Sequence` tidak boleh dibungkus `Reaction` (pipeline merender `Sequence` lewat loop, bukan `render_reply`), jadi reaksi ✘ tidak ikut — sama seperti `download/tiktokdl2.rs`.
- L218: reaksi ✓ tidak bisa menempel pada `Reply::Sequence` — lihat catatan di atas.

### `sticker/swm.rs` (3)

- L10: tambah `Ctx::download_quoted()` + `Reply::Sticker` di `oktz-plugin-api` untuk menghidupkan lagi langkah `swm.js:57-79`.
- L88: `swm.js:53-79` butuh buffer sticker; lihat catatan modul.
- L238: modul) — cabang `catch` JS yang dipakai.

### `sticker/ttp.rs` (2)

- L5: render teks (`ttp.js:30-70`: `createCanvas`, `measureText`, `fillText`, `canvas.encode("png")`) butuh rasterizer teks + font; `oktz_plugin_api::gambar` hanya bisa resize/grayscale/encode. Jadi jalur teks berakhir di teks error JS (`ttp.js:82`) + `☢`, sama seperti `sticker/attp2.rs`. Reaksi `🕕` (`ttp.js:27`) tidak dikirim: `with_reaction` hanya membawa satu reaksi dan JS menimpanya dengan `☢` (`ttp.js:81`). `console.error("[TTP Maker]", err.message)` (`ttp.js:80`) tidak punya padanan log di kontrak plugin. Saat rasterizer teks ada: kanvas 512x512 → `Reply::sticker(...)` + `✅`. `packname`/`author` (`config.sticker`) tidak ada di `Reply::Sticker`. Kategori plugin ini `maker` (`ttp.js:7`), tapi berkasnya ada di `plugin/src/sticker/` mengikuti `plugins/sticker/ttp.js`. `NOTE (mod.rs):` tambahkan `pub mod ttp;` ke `sticker/mod.rs` secara manual kalau berkas ini didaftarkan (mod.rs bukan milik tugas ini).
- L66: Rasterizer teks belum ada di Rust — lihat di atas.


## main — menu utama (19 catatan / 13 file)

### `main/allmenu.rs` (2)

- L25: yang berbeda dari JS, beserta alasannya: - "case" dari `case/foto.js` tidak ada di registry Rust, jadi hanya plugin yang bisa dihitung. Angka "fitur" = jumlah plugin aktif saja — persis seperti JS kalau `casesByCategory` kosong. - Urutan command di dalam satu kategori: JS memakai urutan registrasi plugin, Rust memakai urutan alfabetis. `all_plugins()` tidak menjamin urutan registrasi, dan hasil yang bisa diulang lebih berguna. - `createBracketBox` di JS menerima `emoji` tapi TIDAK memakainya (`allmenu.js:71-75`) — judul kotak cuma `${title} MENU `. Parameternya tetap dibawa di sini supaya tanda tangannya sama, tapi ditandai `_emoji` supaya tidak ada peringatan compiler. - Varian 2/5/6 butuh aset header (`foto` / `mp4`). Kalau asetnya hilang, kartu tidak bisa dibangun → jatuh ke balasan cadangan, sama seperti `catch` JS (`allmenu.js:592-606`). - Audio menu: JS mengubah MP3 jadi Opus (voice note) lewat ffmpeg untuk gaya 1 & 2 (`allmenu.js:428-465`). Di sini MP3 dikirim apa adanya (`Reply::audio`), yaitu cabang cadangan JS (`allmenu.js:458-464`). - `Reply::Image` di API Rust tidak punya `contextInfo`, jadi forwarded + surat berita dari `getContextInfo` tidak ikut pada balasan cadangan gambar; mention pemicunya tetap dibawa.
- L706: di modul).

### `main/benefitpremium.rs` (1)

- L10: `config.owner.number` di-hardcode dari `config.js`.

### `main/carifitur.rs` (1)

- L11: `sock.sendButton` dengan header gambar `getAssetBuffer("foto")` dipetakan ke `Reply::interactive` + `CardHeader` image dari `config.assets["foto"]` (idiom `main/menu.rs`). Reaksi `🕕`/`✅`/`❌` dibuang: membungkus kartu dengan `with_reaction` membuat header media tidak pernah di-upload.

### `main/donasi.rs` (1)

- L4: JS mengirim tombol interaktif (`sendButton`/`interactiveButtons`) yang tidak ada dalam enum `Reply`; kirimannya disederhanakan jadi teks, atau gambar QRIS dengan teks sebagai caption kalau `qrisUrl` terisi. Footer dan `contextInfo` (newsletter) tidak dapat dicerminkan di API plugin — `saluran.id`/`saluran.name` (default id `120363400911374213@newsletter`) hanya dipakai untuk `contextInfo` itu, jadi tidak dibaca di sini.

### `main/gacha.rs` (1)

- L12: kartu level-up `addExpWithLevelCheck` tidak ikut (butuh canvas + `sock.sendMedia`).

### `main/jadibot.rs` (1)

- L16: saat soket/entry point kedua tersedia, tambahkan `pub mod jadibot;` di `src/main/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

### `main/menu.rs` (1)

- L17: yang berbeda dari JS, beserta alasannya: - "Riwayat command teratas" (`menu.js:990-1007`) dibaca dari `database/main/stats.json`, yang tidak ada di Rust — blok "History" pada varian 6 tidak ditulis. - Mode global `config.mode` tidak ada di `Ctx`; baris "Mode" memakai `PUBLIC` seperti config produksi. - Varian 7 mengirim video kedua yang menempel pada kartu lewat `messageAssociation` (`menu.js:1215-1229`). Di Rust video itu dikirim sebagai pesan biasa setelah kartu, jadi tidak menempel. - Varian 8 memakai `prepareWAMessageMedia` + `mediaTypeOverride: "thumbnail-link"` untuk thumbnail preview; di Rust thumbnail dikirim sebagai gambar biasa tanpa field CDN preview. Fallback ke balasan teks polos mengikuti `try/catch` JS (`menu.js:1379-1406`): kalau aset header tidak ada, kartu tidak bisa dibangun lalu yang dikirim teksnya.

### `main/menucat.rs` (1)

- L22: yang berbeda dari JS, beserta alasannya: - "case" dari `case/foto.js` tidak ada di registry Rust, jadi hanya plugin yang bisa dihitung. Angka `(X plugin + Y case)` (`menucat.js:403-405`) karena itu tidak pernah ditulis — persis seperti JS kalau `casesByCategory` kosong. - Urutan command di dalam satu kategori: JS memakai urutan registrasi plugin, Rust memakai urutan alfabetis. `all_plugins()` tidak menjamin urutan registrasi, dan hasil yang bisa diulang lebih berguna. - Cuaca gagal diambil (open-meteo tidak bisa dihubungi) → teks `"Cuaca tidak tersedia"` (`menucat.js:307-309`) dan kartu tetap terkirim, sama seperti JS. Command tidak pernah gagal karena jaringan. - Varian 6 di JS hanya terpasang di cabang "pakai kategori"; di sini kedua cabangnya memakai kartu yang sama, supaya satu setting `menucatVariant` berperilaku sama di mana pun. Fallback ke balasan teks polos mengikuti `try/catch` JS (`menucat.js:362-364`): kalau aset header tidak ada, kartu tidak bisa dibangun lalu yang dikirim teksnya.

### `main/ping2.rs` (3)

- L9: data sistem dibaca dari `/proc` + `std`; `df` tetap dipanggil lewat `std::process::Command` karena `statvfs` tidak ada di `std`. ponytail: `process.memoryUsage()`, `process.version`, dan `m.messageTimestamp` tidak ada; Heap = VmRSS/VmSize, Node = `N/A`, WA Roundtrip = 1 ms (cabang `Date.now()` di JS). Urutan kirim meniru handler JS: tabel dulu, BARU reaksi `✅` (`ping2.js:129-141`). Reaksi pembuka `🕕` (`ping2.js:60`) dikirim sebelum tabel dan dibawa `Reply::Sequence`.
- L11: `process.memoryUsage()`, `process.version`, dan `m.messageTimestamp` tidak ada; Heap = VmRSS/VmSize, Node = `N/A`, WA Roundtrip = 1 ms (cabang `Date.now()` di JS). Urutan kirim meniru handler JS: tabel dulu, BARU reaksi `✅` (`ping2.js:129-141`). Reaksi pembuka `🕕` (`ping2.js:60`) dikirim sebelum tabel dan dibawa `Reply::Sequence`.
- L148: `process.memoryUsage().heapUsed/.heapTotal` — padanan terdekat adalah VmRSS/VmSize proses ini (byte), bukan heap V8.

### `main/rules.rs` (1)

- L3: `getAssetBuffer("rules")` membaca gambar dari disk lalu mengirimnya sebagai `image` + caption. Plugin tidak punya `fs`, dan `MediaSource::Path` butuh path yang sudah diketahui router — sedangkan `getAssetBuffer` tidak pernah mengembalikan path, hanya buffer. Jadi gambar tidak bisa dikirim dan balasan selalu teks.

### `main/stats.rs` (3)

- L13: `process.memoryUsage()` tidak ada; Heap memakai VmRSS/VmSize dari `/proc/self/status` — padanan terdekat. ponytail: `process.version` tidak punya padanan di Rust; kolom Node diisi `N/A`.
- L15: `process.version` tidak punya padanan di Rust; kolom Node diisi `N/A`.
- L132: `process.memoryUsage().heapUsed/.heapTotal` — padanan terdekat adalah VmRSS/VmSize proses ini (byte), bukan heap V8.

### `main/stopjadibot.rs` (2)

- L3: `stopJadibot`, `isJadibotActive`, dan `getJadibotStatus` dari `src/lib/jadibot-manager.js` tidak diekspos ke plugin di registry Rust. Tidak ada state jadibot yang dapat dibaca atau diubah.
- L26: Semua operasi jadibot butuh jadibot-manager yang tidak tersedia di registry Rust.

### `main/tqto.rs` (1)

- L3: `config.bot.name` tidak ada di `Ctx`, jadi nama bot di-hardcode dari `config.js:39` — sama seperti `info/benefitpartner.rs`.


## rpg — rpg (16 catatan / 14 file)

### `rpg/alchemy.rs` (1)

- L3: `user.inventory[material] -= needed` lalu `delete` kalau `<= 0`, jadi bahan habis HILANG dari tas — bukan disimpan sebagai 0 seperti `sellall`. Itu yang bikin `sellall` dan `alchemy` tidak bisa saling menimpa. `alchemy.js:132` mengirim pesan "Blubuk blubuk..." lebih dulu, lalu `setTimeout(3000)` (`alchemy.js:133`), baru pesan hasil. Dua pesan itu dikirim berurutan lewat [`Reply::sequence_text`]. Reaksi `🧪`/`✅` (`alchemy.js:131`, `:145`) tidak ikut: reaksi tidak bisa dibarengkan `Reply::sequence` (jalur reaksi pipeline hanya bisa merender satu pesan), jadi teks dua pesan lebih dipilih.

### `rpg/arena.rs` (1)

- L3: `db.getUser(mentioned)` tidak ada di `Ctx` untuk private chat dan roster yang tidak memuat target — `other_user` mengembalikan `None` dan plugin membalas "belum terdaftar" untuk row yang memang belum ada (`db.getUser` juga `null` di sana). Di private chat (`isGroup: false`) roster selalu kosong, jadi duel hanya jalan di grup.

### `rpg/breeding.rs` (1)

- L3: `db.getUser(mentioned)` hanya terbaca lewat roster grup (`other_user`), dan `isGroup: true` di JS — jadi di private chat plugin ini selalu menjawab "nggak punya peliharaan" karena pasangan tidak ada di store yang bisa dibaca. `PET_NAMES[type]` juga bisa `undefined` untuk tipe tak dikenal; `?.split(" ")[1] || "Baby"` menutupnya di `nama` bayi.

### `rpg/dice.rs` (1)

- L3: `sendRpgPreview` + `setTimeout(2500)` butuh `sock` dan tidur async, yang tidak ada di `Ctx` API (sinkron). Animasi "mengocok dadu" dilewati; validasi, undian, perhitungan kemenang/kehilangan, dan mutasi koin lewat `Mutation::SetUser` dipindah. Catatan: baris `pub mod dice;` perlu ditambah ke `rpg/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `rpg/divorce.rs` (1)

- L3: `db.getUser(spouseJid)` hanya terbaca lewat roster grup, dan plugin ini `isGroup: false` — jadi di private chat pasangan tidak pernah ditemukan dan hanya sisi pemanggil yang dibersihkan. `spouse` dan `marriedAt` ditulis `null`, bukan `delete`, jadi tetap terbaca sebagai "tidak punya pasangan" (`!user.rpg.spouse`).

### `rpg/gift.rs` (1)

- L3: `db.getUser(spouseJid)` hanya terbaca lewat roster grup, dan plugin ini `isGroup: false` — jadi di private chat pasangan tidak pernah ditemukan dan baris partner tidak pernah ditulis. Capacitas hilang satu-directional: item keluar dari tas pengirim tapi tidak masuk ke pasangan.

### `rpg/guild.rs` (1)

- L3: tabel guild ada di `db.db.data.guilds` — key TOP-LEVEL store, bukan `settings` dan bukan milik user. `Mutation` hanya punya SetUser / SetGroup / SetSetting, jadi di sini guild disimpan sebagai setting `guilds` (`ctx.setting("guilds")` untuk baca). Perilaku plugin tetap sama karena hanya satu writer.

### `rpg/hourly.rs` (1)

- L3: `config.isPremium(m.sender)` adalah fungsi server; `Ctx` hanya membawa satu flag owner dan field `isPremium` milik pengirim — partner tier tidak terlihat, jadi premium yang berasal dari luar store user tidak terdeteksi di sini.

### `rpg/jualan.rs` (1)

- L120: `Ctx` tidak punya pembacaan store untuk JID sembarang, jadi di private chat (atau mention ke non-anggota) bar selalu `None` walau row-nya ada. Tambahkan `user_entry(jid)` ke `Ctx` kalau router mau menutup celah ini; sementara ini plugin yang butuh row target (`marry`, `transfer`) treating `None` sebagai "belum pernah ada", sama seperti `db.getUser` yang mengembalikan objek kosong lewat `db.setUser(target)`.

### `rpg/marry.rs` (1)

- L6: bar pasangan dibaca lewat `crate::rpg::jualan::roster_row`, yang hanya melihat anggota grup. Di private chat pasangan yang sudah punya `rpg.spouse` bisa terlewat — lihat catatan helper itu.

### `rpg/sellall.rs` (1)

- L3: `SELL_PRICES` di JS hanya 22 kunci dan TIDAK memuat `potion`/`mpotion`/`stamina`/`kunai`/`shuriken`/`chakra`/`scroll` (meski ada di `inventory.js`). Item yang tidak punya harga jual di sini tidak terjual dan TIDAK dikosongkan — persis seperti di JS.

### `rpg/slot.rs` (2)

- L4: JS mengirim DUA pesan: preview "TREK TREK TREK..." lewat `sock.sendPreview`, `setTimeout(2500)`, lalu preview hasil lewat `sock.sendPreview` yang sama. `Ctx` tidak punya `sock` dan `Reply` hanya membawa satu payload, jadi hanya pesan hasil yang terkirim — teksnya sama persis dengan `slot.js:79-88`.
- L112: jadi negatif — nilai di atas i64::MAX tidak bisa mewakili perilaku f64 JS, naik kelas kalau saldo sebesar itu benar-benar dipakai.

### `rpg/stamina.rs` (2)

- L3: JS mengirim lewat `sendRpgPreview` (`sock.sendPreview`) yang menempelkan thumbnail RPG + `config.info.website` ke caption dan menyetel judul/deskripsi kartu. `Ctx` tidak punya `sock`, jadi teksnya sama persis tapi terkirim sebagai balasan biasa tanpa kartu preview.
- L19: `Math.round(Infinity)` / `NaN` di JS membuat `repeat` melempar `RangeError` (tidak ada `try/catch` di handler), sedangkan `repeat` Rust panik. Hasilnya dijepit ke `0..=10` supaya data rusak tidak menjatuhkan proses — `maxStamina` sendiri selalu terisi `|| 100` sebelum ini dipanggil.

### `rpg/transfer.rs` (1)

- L4: lewat `crate::rpg::jualan::roster_inventory` — lihat catatan di sana: di luar grup bar itu tidak terbaca dan dianggap kosong.


## info — info (16 catatan / 11 file)

### `info/benefitpartner.rs` (1)

- L3: `config.bot.name` / `config.owner.name` di-hardcode dari `config.js` karena `Ctx` tidak membawa config. Ganti dengan config runtime kalau router someday menyediakannya.

### `info/berita.rs` (2)

- L3: JS memakai `cheerio` dengan `{ xmlMode: true }` untuk parsing RSS. `oktz-plugins` tidak punya parser HTML/XML dan menambah dependency baru dilarang, jadi markup RSS — `<item>`, `<title>`, `<link>`, `<pubDate>`, `<description>` — dipakai manual di bawah. ponytail: `formatDate` di JS pakai `new Date(...).toLocaleString('id-ID')`. Tanpa crate tanggal, pubDate yang tidak bisa di-parse dikembalikan apa adanya (cabang `catch` di JS). Format yang dihasilkan hanya pendekatan.
- L8: `formatDate` di JS pakai `new Date(...).toLocaleString('id-ID')`. Tanpa crate tanggal, pubDate yang tidak bisa di-parse dikembalikan apa adanya (cabang `catch` di JS). Format yang dihasilkan hanya pendekatan.

### `info/bluearchive-char.rs` (1)

- L3: 1. `contextInfo` `{forwardingScore, isForwarded, forwardedNewsletter…}` yang ditempel pada `sock.sendMessage` (`bluearchive-char.js:143-150`) tidak punya field di `Reply` — badge forwarded-newsletter hilang. `config.saluran?.id` / `config.saluran?.name` (`config.js:124-127`) hanya dipakai di `contextInfo` itu, jadi ikut tidak terjangkau. 2. `char.bio.substring(0, 200)` dihitung per unit UTF-16 di JS; di sini per `char`, jadi batasnya bergeser untuk emoji. 3. Reaksi `🕕` (`:91`) dan `✅` (`:159`) tidak bisa dua-duanya: satu `Reply` hanya membawa satu `Reaction`. Yang dipasang adalah reaksi cabang akhir. `list()` (`bluearchive-char.js:33-46`) juga menyatukan ulang `imgSmall` dan `img` ke prefiks CDN, tapi `char()` hanya membaca `.url` dari hasil itu — penyatuan di `list()` tidak pernah dipakai, jadi tidak ditulis.

### `info/fiturpremium.rs` (1)

- L3: `db.setting('capprem')` adalah override global di store dan `Ctx` tidak membawanya, jadi syaratnya selalu `PluginMeta::is_premium`.

### `info/gag.rs` (1)

- L3: reaksi hilang sebagian. JS bereaksi `🕕` di awal lalu `✅`, `❌`, atau `☢`; `Reply::Reaction` hanya bisa membawa satu emoji, jadi hanya reaksi terakhir yang ikut. Reaksi `🕕` dibuang.

### `info/gagwatch.rs` (1)

- L3: reaksi hilang sebagian. JS bereaksi `🕕` di awal lalu `✅`, `❌`, atau `☢`; `Reply::Reaction` hanya bisa membawa satu emoji, jadi hanya reaksi terakhir yang ikut.

### `info/gempa.rs` (1)

- L3: 1. `sock.sendMedia(m.chat, Buffer.from(imgRes.data), text, m, {type:'image'})` (`gempa.js:56`) dijawab `Reply::image(MediaSource::url(..), text)`. `MediaSource::Url` baru di-fetch saat pipeline mengirim, sedangkan JS men-download dulu di `:52-55`; akibatnya `catch` di `:59` yang turun ke `m.reply(text,..)` tidak bisa terjadi di sini. 2. `contextInfo: saluranCtx()` (`gempa.js:60` dan `:63`) tidak punya field di `Reply` — badge forwarded-newsletter hilang. 3. `m.react('🕕')` di `:21` tidak ikut: `Reply::Reaction` hanya bisa membawa satu emoji, jadi yang dipasang adalah reaksi cabang akhir (`✅` atau `☢`). Sama seperti `crate::info::jadwalbola`. `gempa.js` tidak pernah memanggil `te(...)`, jadi `error_template` tidak dipakai di file ini.

### `info/harilibur.rs` (1)

- L3: reaksi hilang sebagian. JS bereaksi `🕕` di awal lalu `✅`, `❌`, atau `☢`; `Reply::Reaction` hanya bisa membawa satu emoji, jadi hanya reaksi terakhir yang ikut.

### `info/infotourney.rs` (4)

- L3: JS memakai `cheerio` untuk parse `.items-row.item`, `h2[itemprop="name"] a`, `p img`, `time[itemprop="datePublished"]`, dan `p[style="text-align: center;"]`. `oktz-plugins` tidak punya parser HTML, jadi semua dipakai manual di bawah. ponytail: `timeHelper.fromTimestamp(datePublished, "DD/MM/YYYY HH:mm")` — tanpa crate tanggal, string datetime dikembalikan apa adanya kalau tidak bisa di-parse. ponytail: `sock.sendMedia(..., { type: 'image' })` menjadi `Reply::image(MediaSource::Url)` dengan caption. URL di-fetch pipeline saat kirim. ponytail: `m.react` dua kali (`🕕` lalu `✅`/`❌`) — hanya reaksi akhir yang terkirim. Catatan: baris `pub mod infotourney;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/info/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L7: `timeHelper.fromTimestamp(datePublished, "DD/MM/YYYY HH:mm")` — tanpa crate tanggal, string datetime dikembalikan apa adanya kalau tidak bisa di-parse. ponytail: `sock.sendMedia(..., { type: 'image' })` menjadi `Reply::image(MediaSource::Url)` dengan caption. URL di-fetch pipeline saat kirim. ponytail: `m.react` dua kali (`🕕` lalu `✅`/`❌`) — hanya reaksi akhir yang terkirim. Catatan: baris `pub mod infotourney;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/info/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L10: `sock.sendMedia(..., { type: 'image' })` menjadi `Reply::image(MediaSource::Url)` dengan caption. URL di-fetch pipeline saat kirim. ponytail: `m.react` dua kali (`🕕` lalu `✅`/`❌`) — hanya reaksi akhir yang terkirim. Catatan: baris `pub mod infotourney;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/info/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).
- L13: `m.react` dua kali (`🕕` lalu `✅`/`❌`) — hanya reaksi akhir yang terkirim. Catatan: baris `pub mod infotourney;` perlu ditambahkan ke `rust/crates/oktz-plugins/src/info/mod.rs` oleh orchestrator (tidak boleh disentuh agent porting per aturan PORT-BRIEF #2).

### `info/jadwalbola.rs` (2)

- L3: reaksi hilang sebagian. JS bereaksi `🕕`, lalu `❌`, `✅`, atau `☢`; `Reply::Reaction` hanya bisa membawa satu emoji, jadi hanya reaksi terakhir yang ikut. ponytail: `ctx.http` tidak bisa mengikuti `sendWithRetry`/delay dari plugin lain, dan `jadwalbola.js` memang tidak punya retry — satu `GET` lalu selesai.
- L7: `ctx.http` tidak bisa mengikuti `sendWithRetry`/delay dari plugin lain, dan `jadwalbola.js` memang tidak punya retry — satu `GET` lalu selesai.

### `info/livescore.rs` (1)

- L10: reaksi hilang sebagian. JS bereaksi `🕕`, lalu `✅` atau `❌`; `Reply::Reaction` hanya bisa membawa satu emoji, jadi hanya reaksi terakhir yang ikut.


## download — downloader (15 catatan / 8 file)

### `download/alightmotion.rs` (1)

- L3: JS mengirim `document: { url }` (sumber `Url`, bukan byte), dan nama berkas memakai `config.bot.name`. Di `Ctx::default()` nama bot kosong, jadi tes di bawah menghasilkan `AlightMotion_.zip`; produksi (lebih `Runtime::from_config`) menghasilkan `AlightMotion_Oktz Bot.zip` seperti JS.

### `download/githubdl.rs` (2)

- L3: pemeriksaan kedua di JS memakai `fetch(..., { method: 'HEAD' })`, tapi `HttpGet` (`oktz-plugin-api`) tidak punya HEAD — hanya GET. Konsekuensinya ZIP yang dicek ikut terunduh utuh, dan `maxContentLength` yang di JS membatasi sisanya tidak ada di sini. Tambahkan metode HEAD ke trait `HttpGet` kalau repo besar mulai jadi masalah. ponytail: `Reply` tidak punya variant document, jadi hasil `sendMedia` hanya terkirim sebagai nama file teks.
- L9: `Reply` tidak punya variant document, jadi hasil `sendMedia` hanya terkirim sebagai nama file teks.

### `download/rednotedl.rs` (1)

- L158: hanya foto pertama yang ikut — `Reply` cuma bisa satu media.

### `download/shopeedl.rs` (1)

- L3: `Reply` tidak punya variant video, jadi hasil `sendMessage` hanya terkirim sebagai caption teks; URL videonya tidak ikut.

### `download/threaddl.rs` (3)

- L3: `he.decode` dari paket npm `he` punya tabel ~2000 entitas. Yang dipindah di sini subset yang lazim muncul pada teks Threads plus semua entitas numerik (`&#123;` / `&#x7B;`); entitas di luar subset itu dibiarkan apa adanya — sama seperti `he.decode` pada entity yang tidak dikenal. ponytail: axios di sini memakai `validateStatus: () => true`, jadi status 4xx/5xx tidak melempar. `HttpGet` justru mengembalikan `Err` di atas 399, sehingga 4xx/5xx sekarang masuk ke cabang `catch` (`☢`), bukan ke cabang "gagal mengambil data" (`❌`). Perbaiki di `HttpGet` kalau bedanya penting. ponytail: `Reply` tidak punya variant album maupun video. Postinggan banyak media hanya mengirim teks caption-nya; satu video juga hanya teks caption; satu foto masih terkirim sebagai gambar.
- L9: axios di sini memakai `validateStatus: () => true`, jadi status 4xx/5xx tidak melempar. `HttpGet` justru mengembalikan `Err` di atas 399, sehingga 4xx/5xx sekarang masuk ke cabang `catch` (`☢`), bukan ke cabang "gagal mengambil data" (`❌`). Perbaiki di `HttpGet` kalau bedanya penting. ponytail: `Reply` tidak punya variant album maupun video. Postinggan banyak media hanya mengirim teks caption-nya; satu video juga hanya teks caption; satu foto masih terkirim sebagai gambar.
- L14: `Reply` tidak punya variant album maupun video. Postinggan banyak media hanya mengirim teks caption-nya; satu video juga hanya teks caption; satu foto masih terkirim sebagai gambar.

### `download/tiktok.rs` (2)

- L17: yang berbeda dari JS, beserta alasannya: - `resolveTikTokUrl` (`tiktokdl.js:3-24`) mengikuti redirect短 link `vt.tiktok.com` dan mengambil URL akhirnya. Klien HTTP Rust mengikuti redirect secara internal tapi tidak参加会议 URL akhir, jadi short link diteruskan apa adanya — Savett dan TikWM sama-sama menerimanya. - Urutan provider: JS mencoba TikWM lalu Savett (`tiktokdl.js:352-375`). `oktz_scraper::tiktok::ttdown` sudah merantai TikWM → Savett → YuuLabs → MusicalDown, jadi rantai yang lebih panjang itu yang dipakai di sini. - `formatDate` (`tiktokdl.js:35-46`) memakai `toLocaleDateString` dengan locale Inggris; tanpa ICU di Rust, scraper memakai format ISO-like. - Album slide butuh `albumMessage` + `relayMessage` per slide (`tiktokdl.js:470+`); kontrak `Reply` cuma satu media per pesan, jadi cabang foto mengirim foto pertama saja.
- L295: `Reply::Sequence` tidak boleh dibungkus `Reaction` (pipeline merender `Sequence` lewat loop, bukan `render_reply`), jadi reaksi ✅ pada jalur ini tidak ikut — sama seperti `download/tiktokdl2.rs`.

### `download/tiktokdl2.rs` (4)

- L149: reaksi ⏱️ (`tiktokdl2.js:134`) hilang — `Reply` hanya membawa satu reaksi, jadi hanya reaksi penutup di bawah yang dikirim.
- L155: JS cuma memakai savett; `ttdown` ditambahkan sebagai cadangan (lihat header). Hapus `or_else` kalau ingin persis seperti JS.
- L194: album slide butuh `albumMessage` + `relayMessage` (`tiktokdl2.js:196-234`), tidak ada di kontrak `Reply`; slide dikirim satu per satu sebagai gambar. Reaksi ✅ juga tidak bisa menempel di `Sequence` (pipeline melewati varian `Reaction`).
- L239: album slide butuh `albumMessage` + `relayMessage` (`tiktokdl2.js:196-234`), tidak ada di kontrak `Reply`; cabang foto diperlakukan seperti album kosong sehingga mendarat di note + audio ini. Reaksi ✅ (`tiktokdl2.js:261`) juga tidak bisa menempel di `Sequence` (pipeline melewati varian `Reaction`).

### `download/ttmp3.rs` (1)

- L13: yang berbeda dari JS, beserta alasannya: - JS menulis berkas sementara di `tmp/` lalu menghapusnya; di sini video dan audio mengalir lewat pipe ffmpeg, jadi tidak ada berkas yang perlu dibersihkan. - `fileName: TikTok_Audio_${Date.now()}.mp3` (`ttmp3.js:147`) tidak bisa ditulis: kontrak `Reply::Audio` tidak punya nama berkas. - `Referer` saat mengunduh MP3 memakai origin URL (`ttmp3.js:127`); di sini memakai origin yang sama persis.


## user — user (15 catatan / 11 file)

### `user/bataldaftar.rs` (2)

- L7: `getRegistrationSessionKey` di JS menormalkan `@lid` lebih dulu lewat `lidToJid` (`src/lib/lid.js` — butuh cache file + `onigis`), jadi kunci di sini cukup digit dari `m.sender` penuh (suffix `:device` ikut terhitung, sama seperti `replace(/[^0-9]/g,"")`). Plugin `daftar` yang menulis sesi harus memakai kunci yang sama; lihat ponytail: di modul itu.
- L11: menulis sesi harus memakai kunci yang sama; lihat di modul itu.

### `user/birthday.rs` (1)

- L4: `Ctx::user_data` hanya diisi untuk `sender`, sedangkan JS membaca `db.getUser(target)` setelah mention. Port ini karena itu selalu membaca dan menampilkan birthday PENGIRIM — nama dan mention juga mengikuti pengirim supaya tidak ada baris yang mencampur nama orang lain dengan tanggal orang ini. Jalankan tanpa mention (`.birthday`) kalau butuh hasil yang persis sama dengan JS. Perbaiki di `Ctx` lebih dulu kalau mention harus didukung.

### `user/birthdaylist.rs` (1)

- L8: `ctx.roster.participants[i].data` dipakai, bukan `db.getUser`: pipeline mengisi `data` hanya untuk baris yang ada di store, jadi `Value::Null` = "user belum pernah ada" seperti `db.getUser` mengembalikan `undefined`. Pengurutan bukan "belum ulang tahun dulu" sederhana: `aNext`/`bNext` di JS menandai ulang tahun yang sudahLewat tahun ini, jadi yang lewat masih ikut — hanya tertinggal di bawah yang belum. Comparing dua-duanya menghasilkan urutan yang sama seperti pengurutan JS, dan itu yang dipakai di sini.

### `user/buyenergi.rs` (2)

- L3: `config.energi.premium` tidak dibaca. Default-nya `99999999`, bukan `-1`, jadi `db.updateEnergi` tidak pernah mengembalikan unlimited ke premium; hanya owner (`config.energi.owner = -1`) yang dapat, dan itu dicerminkan di `unlimited_energi()`.
- L65: teks JS menjanjikan "Koin dikembalikan" tapi tidak ada pengembalian — `db.updateKoin` sudah dijalankan baris sebelumnya. Bug itu ikut dip-port apa adanya; perbaikannya berarti mengubah store, bukan cuma teks, jadi harus lewat keputusan-porting terpisah.

### `user/buyfitur.rs` (1)

- L3: `config.isPremium(m.sender)` TIDAK di-port: itu memanggil `config.js`, bukan store, dan `Ctx` hanya membawa `is_owner` (kepemilikan bot). Jadi hanya pemeriksaan `user.isPremium` yang tersisa — user premium yang secondhand diabaikan, dan angka koinnya tidak terpakai. `db.updateKoin(jid, -3000)` tidak ada di `Mutation`; `SetUser` menulis objek partial dengan operator `??` di JS, jadi `koin` yang dikirim di sini harus berupa ANGKA LENGAP, bukan `{"$dec": 3000}` — kalau tidak, field itu tidak akan pernah berkurang.

### `user/daftar.rs` (2)

- L4: `getDatabase()`, `db.setUser()`, `db.updateKoin/Exp/Energi()`, `db.save()`, dan `sock.sendMessage`/`m.react` tidak diekspos ke plugin di registry Rust. Tidak ada store, tidak ada sesi, dan tidak ada pengiriman pesan. Seluruh handler di bawah jatuh ke error template — penyimpangan disengaja yang perlu diimplementasikan ulang saat API database/sock tersedia.
- L29: Semua logika daftar (cek isRegistered, sesi registrasi, buat sesi, kirim prompt dengan gambar, m.react) butuh database dan sock yang tidak tersedia di registry Rust.

### `user/energi.rs` (1)

- L3: `db.setting('energi')` adalah toggle GLOBAL di store, dan `Ctx` hanya membawa `users[sender]` + `groups[chat]`. Handler ini karena itu selalu memakai cabang "energi aktif" (`config.energi.enabled === true`). Cabang "OFF" tetap diuji lewat [`render`], yang menerima flag-nya sebagai argumen — begitu router expose settings, cukup teruskan nilainya.

### `user/koin.rs` (1)

- L3: `Ctx::user_data` hanya diisi untuk `sender` (lihat `oktz-wa/src/pipeline.rs`), sedangkan JS membaca `db.getUser(targetJid)` setelah mention. Untuk target selain sender angka yang tampil adalah milik pengirim. Tambahkan `user_data_for(target)` ke `Ctx` kalau router mau membaca state target.

### `user/listdaftar.rs` (1)

- L12: `contextInfo.forwardedNewsletterMessageInfo` (saluran) pada pengiriman JS tidak ada di `Reply::Text` — mentions tetap dikirim.

### `user/profile.rs` (1)

- L3: Sembilan panggilan `sock`/lib JS dibungkus `safe()` di JS — gagal berarti `null` dan barisnya tetap dicetak dengan nilai kosong. Di `Ctx::sock` hanya dua yang tersedia, sisanya dianggap gagal seperti `safe()` yang error (baris tampil `-` / nilai default): - `sock.onWhatsApp` → `sock().ada_whatsapp` (Eksistensi WA + JID kanonik). - `sock.profilePictureUrl` → `sock().foto_profil` (avatar + `ppMedia`). - `fetchStatus`, `getBusinessProfile`, `getCatalog`, `getCollections`, `getLidFromJid`, `getContact`, `getDevice` → tidak ada di `Sock` (Bio, Business Info, LID, Device ID, "Bot WhatsApp" selalu `null`). - `m.key?.id` (Device ID) dan `sock.user?.id` / `platform` / Node `process.version` (Bot Viewpoint) tidak diekspos `Ctx`. - `user.rpg` yang JS tulis-ulang (level/health/mana/stamina) dikirim lewat `Mutation::SetUser` dengan objek `rpg` utuh dari snapshot store. - `config.isOwner`/`config.isPremium` untuk TARGET ditiru dari `config().owner.number` + `db.data.owner` + flag `isPremium` user + `settings.premiumUsers` (salinan `config.js:364-446` yang terjangkau `Ctx`).

### `user/unreg.rs` (2)

- L3: `getDatabase()`, `db.setUser()`, dan `db.save()` tidak diekspos ke plugin di registry Rust. Tidak ada store user yang dapat dibaca atau ditulis. Handler di bawah hanya melaporkan "belum terdaftar" (karena store selalu kosong) dan jatuh ke error untuk cabang sukses yang seharusnya menyimpan.
- L28: `db.getUser(m.sender)` tidak tersedia; tanpa store, user selalu dianggap belum terdaftar.


## stalker — stalker (14 catatan / 12 file)

### `stalker/countrystalk.rs` (1)

- L8: `c.population.toLocaleString('id-ID')` memakai pemisah ribuan titik dan pemisah desimal koma (dikonfirmasi dengan `(1234567).toLocaleString('id-ID')` di Node). `toLocaleString` juga membulatkan ke tiga angka desimal untuk angka pecahan; itu ikut.

### `stalker/discordstalk.rs` (1)

- L3: `m.react("🔍")` terjadi sebelum pemanggilan API, sementara `Reply::with_reaction` hanya mampu membawa satu reaksi — yang ikut ke balasan hanyalah reaksi penutup (`✅`/`❌`/`☢`), `🔍` hilang. Begitu pula `sock.sendMessage(..., { quoted: m })` untuk avatar tidak tersedia di `Ctx`; kirim avatar jadi `Reply::image` tanpa quoted. `d.created_at` di JS di-parse `new Date(...)` (terima string ISO). Di sini hanya angka epoch ms (atau string numerik) yang diformat; nilai lain jadi `-`. Tambahkan pemroses ISO ke `crate::common` kalau butuh ketepatan penuh.

### `stalker/ffstalk.rs` (1)

- L3: JS bereaksi `🕕` sebelum request, lalu `✅`/`❌` di akhir. `Reply::Reaction` hanya membawa satu emoji, jadi reaksi akhir (yang masih terlihat setelah request selesai) yang dipasang. Gambar banner dikirim tanpa `quoted` karena `Reply::Image` tidak punya bidang quoted.

### `stalker/genshinstalk.rs` (1)

- L3: JS bereaksi dua kali (`🕕` lalu `✅`/`❌`), sementara `Reply::with_reaction` hanya satu emoji — hanya reaksi akhir (`✅` / `❌`/`☢`) yang dibawa. Header `User-Agent` dikirim lewat `HttpCall::call`. `sock.sendMessage` dengan `quoted: m` tidak tersedia; image dikirim tanpa quote. Catatan: baris `pub mod genshinstalk;` perlu ditambah ke `stalker/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).

### `stalker/githubstalk.rs` (1)

- L9: `m.react('🔍')` terjadi sebelum permintaan, jadi pada cabang gagal reaksi terakhir yang ikut adalah `☢`.

### `stalker/npmstalk.rs` (1)

- L8: `m.react('🔍')` terjadi sebelum permintaan, jadi pada cabang gagal reaksi terakhir yang ikut adalah `☢`. `Reply::Reaction` hanya bisa membawa satu emoji.

### `stalker/pintereststalk.rs` (1)

- L7: reaksi `🕕` terjadi sebelum permintaan. Karena `Reply::Reaction` hanya bisa membawa satu emoji, yang ikut adalah reaksi terakhir: `❌` saat profil tidak ditemukan, `✅` setelah caption terkirim, `☢` saat gagal.

### `stalker/robloxplayer.rs` (1)

- L7: JS bereaksi dua kali (`🔍` di awal, `✅`/`❌` di akhir). `Reply::with_reaction` hanya boleh satu emoji, jadi hanya reaksi akhir yang dibawa — pola yang sama sudah dicatat di `ytstalk.rs`.

### `stalker/robloxstalk.rs` (1)

- L6: bawaan ( tambahkan field tanggal ke `Ctx` kalau diperlukan).

### `stalker/stalkdc.rs` (3)

- L10: `randomCb()` memakai `rand` (JS `Math.random`) dan versi collector TIDAK di-cache antar panggilan seperti `_cachedV` — antrean plugin Rust tidak punya state modul. Biaya satu request kecil per panggilan. ponytail: JS bereaksi `🕕` sebelum permintaan lalu `✅`/`❌` sesudahnya (`stalkdc.js:179,185,215,218`), sedangkan `Reply::with_reaction` hanya bisa membawa satu reaksi — jadi `🕕` hilang dan yang ikut ke balasan hanya reaksi penutup. Reaksi `✅` juga dikirim sebelum media avatar diunggah, jadi kegagalan upload tidak menambah balasan error. ponytail: JS mengirim avatar dengan `{ quoted: m }` (`stalkdc.js:210`); `Reply::image` tidak bisa mengutip pesan pemicu, jadi avatar dikirim tanpa quote.
- L14: JS bereaksi `🕕` sebelum permintaan lalu `✅`/`❌` sesudahnya (`stalkdc.js:179,185,215,218`), sedangkan `Reply::with_reaction` hanya bisa membawa satu reaksi — jadi `🕕` hilang dan yang ikut ke balasan hanya reaksi penutup. Reaksi `✅` juga dikirim sebelum media avatar diunggah, jadi kegagalan upload tidak menambah balasan error. ponytail: JS mengirim avatar dengan `{ quoted: m }` (`stalkdc.js:210`); `Reply::image` tidak bisa mengutip pesan pemicu, jadi avatar dikirim tanpa quote.
- L20: JS mengirim avatar dengan `{ quoted: m }` (`stalkdc.js:210`); `Reply::image` tidak bisa mengutip pesan pemicu, jadi avatar dikirim tanpa quote.

### `stalker/tiktokstalk.rs` (1)

- L11: `m.react('🔍')` terjadi sebelum permintaan, jadi pada cabang gagal reaksi terakhir yang ikut adalah `☢`.

### `stalker/ytstalk.rs` (1)

- L7: `m.react('🔍')` terjadi sebelum guard kunci, jadi pada cabang "kunci kosong" reaksi terakhir yang ikut adalah `❌`. `Reply::Reaction` hanya bisa membawa satu emoji, jadi hanya yang terakhir.


## game — game (13 catatan / 9 file)

### `game/dungeon.rs` (1)

- L3: `dungeon.js` mengekspor `handler` DAN `dungeonAnswerHandler`. Yang kedua dipanggil router global saat user me-reply (bukan sebuah command), dan `oktz-plugin-api` hanya punya satu titik masuk: `fn(Ctx)`. Yang dipindah di sini `handler` — lobi plus pembuatan sesi. Tahap `encounter` (pilih lokasi, serang, lari) butuh `dungeonAnswerHandler` dan harus dipasang router sebelum alur ini bisa dilanjutkan.

### `game/family100.rs` (2)

- L3: `family100AnswerHandler` bukan command; router global yang memanggilnya (sama seperti `kyubigameAnswerHandler` di `kyubigame.js`). Yang dipindah di sini `handler`. Bank soal `src/data/family100.json` di-`include_str!` apa adanya, bukan disalin jadi konstanta Rust: 7152 soal / 1,6 MB jauh lebih besar dari file plugin mana pun, dan menyalinnya jadi array `const` berarti data punya dua sumber yang bisa melenceng diam-diam. ponytail: `setSessionTimer` butuh event loop, sedangkan `Reply` tidak punya timer. Sesi yang lewat 120 detik dianggap habis saat dibaca berikutnya — efek yang sama dengan sweeper 5-menit di `game-data.js`, tanpa timer yang benar-benar memanggil `endSession`.
- L12: `setSessionTimer` butuh event loop, sedangkan `Reply` tidak punya timer. Sesi yang lewat 120 detik dianggap habis saat dibaca berikutnya — efek yang sama dengan sweeper 5-menit di `game-data.js`, tanpa timer yang benar-benar memanggil `endSession`.

### `game/fisch.rs` (1)

- L8: TIGA hal JS tidak di-port: - `sock.sendPreview(...)` (link preview berjudul `𝗙𝗜𝗦𝗛 𝗜𝗧 𝗚𝗔𝗠𝗘𝗦` dengan `jpegThumbnail` aset `fishit`) tidak ada di `Reply` — semua keluaran dikirim sebagai teks biasa. Prefix `${config.info.website} ` dari `send()` tetap dipertahankan. - `setTimeout` jeda memancing (`fisch.js:171-176`) — hasil langsung menyusul pesan pertama dalam satu balasan. - `db.markDirty("users")` → `Mutation::SetUser` di akhir handler; jalur yang di JS `return` lebih awal setelah refund (gacha duplikat rod) tidak menulis apa pun, sesuai state JS akhirnya.

### `game/kyubigame.rs` (1)

- L3: sama seperti `dungeon.rs`: `kyubigame.js` mengekspor `handler` dan `kyubigameAnswerHandler`, dan yang kedua bukan command — router global yang memanggilnya. Yang dipindah di sini `handler`.

### `game/minecraft.rs` (1)

- L46: sediakan akses store user/grup + asset buffer + kirim preview ke `Ctx`, lalu port seluruh logika game dari `minecraft.js` dan `minecraft-data.js` ke fungsi-fungsi di berkas ini (atau modul pembantu yang di-copy, karena dilarang menyentuh berkas lain).

### `game/suitpvp.rs` (2)

- L3: `answerHandler` bukan command; router global yang memanggilnya. Yang dipindah di sini `handler` saja — sama seperti `kyubigame.rs`. ponytail: JS memakai `setTimeout` untuk mengakhiri ruang duel. Tanpa timer, ruang yang sudah lewat `TIMEOUT` diabaikan saat handler dipanggil berikutnya — efek yang sama, tanpa pengusir yang berjalan sendiri.
- L6: JS memakai `setTimeout` untuk mengakhiri ruang duel. Tanpa timer, ruang yang sudah lewat `TIMEOUT` diabaikan saat handler dipanggil berikutnya — efek yang sama, tanpa pengusir yang berjalan sendiri.

### `game/tictactoe.rs` (3)

- L3: hanya `handler` yang dipindah. `answerHandler` (balasan angka 1-9, menyerah, deteksi pemenang, hadiah koin) dipanggil router global lewat `plugin.answerHandler` (`src/handler.js:453`), sedangkan `PluginEntry` hanya punya satu `handler` — tidak ada tempat untuk fungsi kedua itu. Akibatnya room yang dibuat di sini belum bisa dimainan; sama seperti `suitpvp.rs` dan `kyubigame.rs`. ponytail: JS menghapus room `WAITING` lewat `setTimeout(..., 300000)`. Tanpa timer, room yang sudah lewat 5 menit diabaikan saat handler dipanggil lagi — efek yang sama, tanpa pengusir yang berjalan sendiri. Room `PLAYING` tidak pernah dihapus otomatis di JS, jadi tidak juga di sini. ponytail: `safeReply`/`safeReact` membungkus `m.reply`/`m.react` dengan retry rate-limit dan `delay()`. `Ctx` sinkron tanpa I/O, jadi retry itu hilang; isi pesannya tidak berubah.
- L10: JS menghapus room `WAITING` lewat `setTimeout(..., 300000)`. Tanpa timer, room yang sudah lewat 5 menit diabaikan saat handler dipanggil lagi — efek yang sama, tanpa pengusir yang berjalan sendiri. Room `PLAYING` tidak pernah dihapus otomatis di JS, jadi tidak juga di sini. ponytail: `safeReply`/`safeReact` membungkus `m.reply`/`m.react` dengan retry rate-limit dan `delay()`. `Ctx` sinkron tanpa I/O, jadi retry itu hilang; isi pesannya tidak berubah.
- L16: `safeReply`/`safeReact` membungkus `m.reply`/`m.react` dengan retry rate-limit dan `delay()`. `Ctx` sinkron tanpa I/O, jadi retry itu hilang; isi pesannya tidak berubah.

### `game/ulartangga.rs` (1)

- L18: saat state game + canvas board tersedia, implementasikan ulang dan tambahkan `pub mod ulartangga;` di `src/game/mod.rs` secara manual.

### `game/wwkill.rs` (1)

- L9: router hanya mendaftarkan handler plugin, bukan "answer handler" terpisah. Yang dipindah di sini murni jalur aksi malam; fase siang, vote, dan pengaturan ulang malam tetap milik `werewolf.js`.


## panel — panel (13 catatan / 11 file)

### `panel/addseller.rs` (1)

- L12: `isLid`/`lidToJid` (`addseller.js:4`) tidak ada di `Ctx`; JID dipakai apa adanya seperti `cleanJid` tanpa cabang LID. Sinkronisasi `config.pterodactyl` dari legacy data (`addseller.js:65-68`) tidak ada padanannya karena `Ctx::config()` read-only dan tanpa field pterodactyl — tidak berpengaruh pada balasan.

### `panel/cekjeda.rs` (1)

- L6: `formatTime` di sini BUKAN milik `jedacreate.rs` — JS punya dua fungsi dengan nama sama dan bentuk berbeda: yang di `cekjeda.js` menyertakan detik pada level jam dan menulis `"0 detik"` untuk nilai non-positif.

### `panel/cpanel.rs` (2)

- L3: status server, token DigitalOcean, dan website diambil dari `config.js` (`config.pterodactyl.server1..5`, `config.digitalocean.token`, `config.info.website`). `Ctx` tidak membawa config, jadi nilainya ikut `config.js` yang di-hardcode — persis seperti `info/benefitpartner.rs` melakukan untuk `config.bot.name`. Saat port ini semua `domain`/`apikey` masih kosong, jadi kelima status `❌` dan token DO `❌`; begitu `config.js` diisi, baris ini ikut stale sampai router menyediakannya lewat `Ctx`. ponytail: `getAccessibleServers` (roles-cpanel.js) membaca file `database/cpanel/*.json` lewat `fs` — tidak ada padanannya di `Ctx`, jadi role user non-owner selalu tampil "Tidak ada". Lihat laporan batch.
- L11: `getAccessibleServers` (roles-cpanel.js) membaca file `database/cpanel/*.json` lewat `fs` — tidak ada padanannya di `Ctx`, jadi role user non-owner selalu tampil "Tidak ada". Lihat laporan batch.

### `panel/delpanel.rs` (1)

- L3: `config.pterodactyl.server1..3` (`config.js:258-299`) TIDAK ada di `oktz_plugin_api::config::Runtime`, dan `Ctx::config()` read-only. Mengikuti pola `cpanel.rs`, nilai `domain`/`apikey` di-hardcode dari `config.js` yang memang kosong untuk ketiga server — jadi cabang `validateServerConfig` (`delpanel.js:29-34`) selalu menemukan `['domain', 'apikey (PTLA)']` dan handler berakhir di kartu "sᴇʀᴠᴇʀ sX ʙᴇʟᴜᴍ ᴋᴏɴꜰɪɢ", persis perilaku produksi JS. Sisa alur (GET server → GET user → DELETE → opsi `full`) tetap diport lewat `ctx.http.call` supaya port penuh tinggal mengganti sumber konfigurasi; lewat konfigurasi kosong jalur itu tidak terjangkau, dan test menjangkau lewat `run_dengan_konfigurasi` dengan config palsu.

### `panel/installtemabilling.rs` (1)

- L4: hanya guard + validasinya yang dipindah. Langkah JS setelah `m.react('🕕')` — `new ssh2.Client()`, `.connect()`, tiga `execSSH` — butuh soket TCP yang tidak ada di `Ctx`, jadi tidak bisa di-port; balasannya gantinya memberi tahu host mana yang tidak bisa disentuh. `parse_input` dipakai juga oleh `uninstalltema.rs` (lihat laporan batch).

### `panel/installtemaenigma.rs` (1)

- L3: sama seperti dua installer tema lain: guard + validasi dipindah, `execSSHInteractive` (yang menulis `AKSES TOKEN`, pilihan tema, lalu link WA/group/channel) butuh klien SSH yang tidak ada di `Ctx`.

### `panel/installtemanebula.rs` (1)

- L4: sama seperti `installtemabilling.rs`: guard + validasi dipindah, tiga `execSSH` (Node 22, Blueprint, `blueprint -install nebula`) tidak bisa jalan tanpa klien SSH yang tidak ada di `Ctx`.

### `panel/installtemastellar.rs` (1)

- L3: guard + validasi dipindah; tiga `execSSH` (deps, `flavor.sh`, build assets) butuh klien SSH yang tidak ada di `Ctx`.

### `panel/linode.rs` (1)

- L7: JS memakai `fetch` global dengan header `Authorization: Bearer`, jadi semua permintaan lewat [`oktz_plugin_api::HttpCall`] + `ctx.http.call` (bukan `http.get` polos yang membuang header). Semua kegagalan — fetch error, JSON rusak, `!res.ok` — jatuh ke `catch` yang sama: `m.react('☢')` + `te(prefix, command, pushName)` (`linode.js:375-378`). Alur `linode*gb` menunggu 60 detik di tengah handler (`await new Promise(r => setTimeout(r, 60000))`, `linode.js:94`) — diterjemahkan apa adanya dengan `std::thread::sleep`.

### `panel/rolemanager.rs` (2)

- L8: `src/lib/roles-cpanel.js` menyimpan tiap pasangan role/server di file sendiri (`database/cpanel/<role>_<server>.json`). `Mutation` hanya punya `SetSetting`, jadi semua daftar dipindah ke satu setting `cpanelRoles` dengan kunci `"<role>_<server>"` → array nomor. Pemetaan 1:1, hanya tempatnya yang berbeda. ponytail: `isLid(jid)` → `lidToJid(jid)` tidak di-port; `src/lib/lid.js` butuh cache file `database/lid-cache.json` plus `jidDecode` dari `onigis`. JID `@lid` karena itu diproses apa adanya: angka device-nya yang dipakai.
- L14: `isLid(jid)` → `lidToJid(jid)` tidak di-port; `src/lib/lid.js` butuh cache file `database/lid-cache.json` plus `jidDecode` dari `onigis`. JID `@lid` karena itu diproses apa adanya: angka device-nya yang dipakai.

### `panel/uninstalltema.rs` (1)

- L3: `pluginConfig.name` di JS adalah array `['uinstalltema','uninstalltema','removetema','hapustema']` dengan `alias: []`, jadi elemen pertama jadi `name` dan sisanya jadi command tambahan (`PluginMeta::commands`) — pola yang sama dengan `group/autodl.rs`. Yang dipindah: kedua guard, pembacaan `ip|password`, dan penulisan `global.installtema = { vps, pwvps }` (saya di sini `Ctx::session_set`). `ress.connect()` + `exec` + tulis `skyzodev/2/y/x` ke stream butuh klien SSH yang tidak ada di `Ctx`. `parse_input` dipakai dari `installtemabilling.rs`.


## store — store (10 catatan / 7 file)

### `store/addlist.rs` (2)

- L3: cabang upload media (`m.quoted?.isMedia` / `m.isMedia` + `uploadImage`) TIDAK dipindah — `Ctx` tidak membawa media terkirim dan tidak ada API upload. Tanpa media, `imageUrl`/`videoUrl` tetap `null` dan plugin tetap menyimpan produk/informasi TEKS dengan benar; yang hilang hanya thumbnail. Reply yang menyebut media pun tidak muncul karena keduanya `null`.
- L77: `image`/`video` tetap null — lihat catatan di modul.

### `store/addproduk.rs` (1)

- L9: pesan `⏳ _Mengunggah media..._` di JS dikirim SEBELUM unggah; `Reply` hanya satu per handler, jadi pesan itu digabung hilang — unggah berjalan langsung. Teks peringatan gagal unggah memakai pesan generik "semua host upload gagal" karena `HttpGet` tidak membawa pesan error JS.

### `store/addstok.rs` (2)

- L7: cabang import dari file `.txt` (reply `documentMessage` `.txt`) TIDAK di-port: `m.quoted.download()` butuh `downloadContentMessage`, yang tidak ada di `QuotedMessage`. Input `.addstok <nomor>` dengan reply file karena itu jatuh ke kartu bantuan, sama seperti file yang bukan `.txt`.
- L223: (jalur import `.txt` tidak di-port — lihat modul).

### `store/beli.rs` (1)

- L6: TIGA langkah JS TIDAK di-port, semuanya di luar `Ctx`: `config.owner.number`, `config.store.payment`, dan `config.store.qris` (isi `config.js`, bukan store); pengiriman gambar/video produk lewat `sock.sendMessage`; dan notifikasi ke admin lewat `sock.sendMessage(ownerJid,...)`. Reply hanya boleh satu, jadi blok instruksi pembayaran digabung ke balasan pesanan — nomor transaksi, yang justru bagian penting, tetap utuh.

### `store/done.rs` (1)

- L4: TIGA langkah JS tidak di-port, semuanya di luar `Ctx`: - `sock.sendMessage(buyerJid, { text: invoiceTxt })` — pengiriman ke JID lain bukan bagian dari `Reply` (`done.js:157-178`). Invoice tetap dihitung (termasuk `stockItemDetail` yang diambil dari stok), tapi teksnya tidak bisa dikirim ke pembeli. - Notifikasi grup pembelian `sock.sendMessage(trx.purchaseChat,...)` (`done.js:180-196`) — alasan sama. - `contextInfo` newsletter forwarding pada invoice — `Reply::Text` tidak membawa contextInfo. Kedua pengiriman yang hilang ada di `try/catch` terpisah di JS, jadi alur konfirmasi (mutasi stok + status transaksi + balasan admin) tetap utuh. Reaksi `✅` dilekatkan pada balasan konfirmasi seperti `tools/txt2qr.rs`.

### `store/editlist.rs` (2)

- L3: field `gambar` dan `video` tidak bisa dipindah — keduanya butuh reply/download media lalu upload, dan `Ctx` tidak membawa media. Percabangan itu dikembalikan sebagai pesan eksplisit (branch yang sama seperti di JS ketika tidak ada media), bukan diam-diam menyatakan berhasil. Kalau `Ctx` nanti membawa `quoted`, tinggal isi dua cabang itu.
- L82: Dua cabang ini di JS butuh media; lihat modul.

### `store/editproduk.rs` (1)

- L43: tambahkan akses store (baca `storeProducts`, tulis via `Mutation::SetSetting`) dan unggah media ke `Ctx`, lalu replikasikan seluruh switch field dari JS.


## random — random (9 catatan / 9 file)

### `random/anime.rs` (1)

- L9: pengiriman view-once interaktif (`generateWAMessageFromContent` + `relayMessage`) dan tombol `quick_reply` "Lanjut Cari Lagi?" tidak ada di `Reply` — gambar dikirim biasa. Reaksi `🖼️` (`anime.js:53`) dibuang: membungkus media dengan `with_reaction` membuat gambar tidak pernah di-upload (lihat `random/cecanindo.rs`). Deteksi GIF tetap jalan: byte diambil lewat `ctx.http` lalu dicek magic `GIF` seperti `anime.js:56-57`, dan GIF dikirim sebagai video `gifPlayback`.

### `random/barandom.rs` (1)

- L3: cabang `catch` JS (reaksi `☢` + `te(...)`) tidak bisa terjadi di sini: plugin ini tidak melakukan HTTP apa pun, pipeline yang mengunduh URL dan melaporkan kegagalan sebagai `PipelineError`, bukan sebagai reply. Reaksi `🕕` di JS juga hilang — hanya reaksi terakhir (`✅`) yang dibawa.

### `random/cecanchina.rs` (1)

- L3: reaksi hilang. JS bereaksi dua kali (bendera lalu `✅`) lewat `m.react`. Di sini `Reply::with_reaction` membungkus reply dalam `Reply::Reaction`, sedangkan `Reply::has_media`/`media_source` hanya menembus `Reply::Mutate` — jadi gambar yang dibungkus reaksi tidak akan pernah di-upload. Gambar lebih penting daripada reaksi, jadi reaksi dibuang demi media yang benar-benar terkirim.

### `random/cecanindo.rs` (1)

- L3: reaksi hilang. JS bereaksi dua kali (bendera lalu `✅`) lewat `m.react`. Di sini `Reply::with_reaction` membungkus reply dalam `Reply::Reaction`, sedangkan `Reply::has_media`/`media_source` hanya menembus `Reply::Mutate` — jadi gambar yang dibungkus reaksi tidak akan pernah di-upload. Gambar lebih penting daripada reaksi, jadi reaksi dibuang demi media yang benar-benar terkirim.

### `random/cecanjepang.rs` (1)

- L3: reaksi hilang. JS bereaksi dua kali (bendera lalu `✅`) lewat `m.react`. Di sini `Reply::with_reaction` membungkus reply dalam `Reply::Reaction`, sedangkan `Reply::has_media`/`media_source` hanya menembus `Reply::Mutate` — jadi gambar yang dibungkus reaksi tidak akan pernah di-upload. Gambar lebih penting daripada reaksi, jadi reaksi dibuang demi media yang benar-benar terkirim.

### `random/cecankorea.rs` (1)

- L3: reaksi hilang. JS bereaksi dua kali (bendera lalu `✅`) lewat `m.react`. Di sini `Reply::with_reaction` membungkus reply dalam `Reply::Reaction`, sedangkan `Reply::has_media`/`media_source` hanya menembus `Reply::Mutate` — jadi gambar yang dibungkus reaksi tidak akan pernah di-upload. Gambar lebih penting daripada reaksi, jadi reaksi dibuang demi media yang benar-benar terkirim.

### `random/cecanthai.rs` (1)

- L3: reaksi hilang. JS bereaksi dua kali (bendera lalu `✅`) lewat `m.react`. Di sini `Reply::with_reaction` membungkus reply dalam `Reply::Reaction`, sedangkan `Reply::has_media`/`media_source` hanya menembus `Reply::Mutate` — jadi gambar yang dibungkus reaksi tidak akan pernah di-upload. Gambar lebih penting daripada reaksi, jadi reaksi dibuang demi media yang benar-benar terkirim.

### `random/cecanvietnam.rs` (1)

- L3: reaksi hilang. JS bereaksi dua kali (bendera lalu `✅`) lewat `m.react`. Di sini `Reply::with_reaction` membungkus reply dalam `Reply::Reaction`, sedangkan `Reply::has_media`/`media_source` hanya menembus `Reply::Mutate` — jadi gambar yang dibungkus reaksi tidak akan pernah di-upload. Gambar lebih penting daripada reaksi, jadi reaksi dibuang demi media yang benar-benar terkirim.

### `random/couple.rs` (1)

- L3: JS mengirim `albumMessage` dengan `quoted: m`. `Reply::sequence` tidak membawa `quoted`, dan tiap elemen sequence adalah satu `Reply` tersendiri. Catatan: baris `pub mod couple;` perlu ditambah ke `random/mod.rs` secara manual (mod.rs sudah milik plugin lain, dilarang disentuh).


## clan — clan (7 catatan / 4 file)

### `clan/clancreate.rs` (3)

- L3: JS menyimpan seluruh clan di `db.db.data.clans`, satu map global yang bukan milik user maupun grup. `Ctx` hanya membuka tiga wearer state (user, group, settings), jadi map itu dipindah ke key global `clans` di `settings` — satu-satunya map global yang bisa dibaca plugin. Bentuk isinya sama dengan JS; hanya lokasinya yang berbeda.
- L15: Key global tempat peta `clans` disimpan — lihat catatan di atas.
- L267: `clanName.length` di JS menghitung UTF-16 code unit. untuk nama clan Latin keduanya sama; nama astral (emoji) dihitung 2 di JS dan 1 di Rust, jadi batas 20 EMU di sini longgar 20 char.

### `clan/claninvite.rs` (2)

- L6: JS menolak target yang `getUser(target).clanId` sudah terisi — baris store milik user lain tidak bisa dibaca dari `Ctx`. Yang di sini dijequivalentkan dengan "target sudah jadi anggota clan mana pun" dari peta `clans`. Selisihnya satu: user dengan `clanId` menunjuk clan yang sudah dihapus lolos di Rust, sedangkan JS menolaknya.
- L90: `if (targetUser?.clanId) return …` — lihat catatan di atas.

### `clan/clanmembers.rs` (1)

- L3: JS memanggil `db.getUser(jid)` untuk SETIAP member. Baris store milik orang lain tidak bisa dibaca dari `Ctx` — yang tersedia cuma baris sender (`ctx.user_data`) dan baris peserta grup yang sudah dimuat (`ctx.roster.participants[].data`). Tiga sumber itu dipakai di sini. Member yang tidak ada di ketiganya memakai default yang sama dengan JS: `level` 1 dan `koin` 0.

### `clan/clanwar.rs` (1)

- L3: JS memanggil `db.getUser(jid)` untuk SETIAP member kedua clan (power, lalu `updateKoin`/`updateExp`/`updateEnergi`). Baris store milik orang lain tidak bisa dibaca dari `Ctx` — yang tersedia cuma baris sender (`ctx.user_data`) dan baris peserta grup yang sudah dimuat (`ctx.roster.participants[].data`), sama seperti yang dicatat di `clanmembers.rs`. Member di luar dua sumber itu dihitung dengan default JS (level 1, exp 0) untuk power, dan TIDAK bisa diberi hadiah karena nilai lama `koin`/`exp`/`energi`-nya tidak diketahui. Jumlah yang terlewat disebut di balasan supaya hadiah hilang tidak senyap.


## islamic — islamic (5 catatan / 2 file)

### `islamic/murrotal.rs` (3)

- L3: JS memakai `cheerio` untuk parse `.surah-item` dan atribut `data-title`/`data-audio`. `oktz-plugins` tidak punya parser HTML, jadi markup — `.surah-item`, `h5`, `p`, atribut — dipakai manual di bawah. ponytail: `sock.sendMedia(..., { type: 'audio' })` menjadi `Reply::audio(MediaSource::Url)`; URL di-fetch oleh pipeline pada saat kirim, tidak di-download plugin terlebih dahulu. ponytail: `m.react` dua kali (`🔍` lalu `✅`/`❌`) — `Reply::with_reaction` hanya membawa satu emoji, jadi reaksi akhir saja yang terkirim.
- L7: `sock.sendMedia(..., { type: 'audio' })` menjadi `Reply::audio(MediaSource::Url)`; URL di-fetch oleh pipeline pada saat kirim, tidak di-download plugin terlebih dahulu. ponytail: `m.react` dua kali (`🔍` lalu `✅`/`❌`) — `Reply::with_reaction` hanya membawa satu emoji, jadi reaksi akhir saja yang terkirim.
- L11: `m.react` dua kali (`🔍` lalu `✅`/`❌`) — `Reply::with_reaction` hanya membawa satu emoji, jadi reaksi akhir saja yang terkirim.

### `islamic/quran.rs` (2)

- L3: JS memakai `cheerio` untuk parse `h1`, `h1 + span`, dan `div[id]` dengan id numerik, lalu ambil `[dir="rtl"]`, `.text-primary-500`, `.text-neutral-700`. `oktz-plugins` tidak punya parser HTML, jadi semua dipakai manual di bawah. ponytail: Balasan di-chunk jika lebih dari 60000 karakter (JS). `Reply::sequence` digunakan untuk beberapa pesan berurutan.
- L8: Balasan di-chunk jika lebih dari 60000 karakter (JS). `Reply::sequence` digunakan untuk beberapa pesan berurutan.


## (akar) — akar (3 catatan / 1 file)

### `common.rs` (3)

- L213: `Ctx` tidak membawa `body` mentah, jadi spasi ganda di tengah pesan akan dinormalkan jadi satu spasi. Tambahkan `body: String` ke `Ctx` kalau router nanti mau kesetiaan byte-per-byte.
- L255: JS pakai `m.quoted` lebih dulu dan `m.pushName` untuk nama. `Ctx` tidak membawa keduanya, jadi reply-mode dan nama push tidak bisa dicerminkan. Tambahkan `quoted: Option<...>` + `push_name: String` ke `Ctx` kalau router mau keduanya.
- L485: beberapa plugin JS memfilter JID bot sendiri (`id== sock.user?.id`), dan `GroupRoster` tidak punya JID bot — hanya flag `is_bot_admin`. Pemanggil yang butuh filter itu harus menambahkan `bot_jid` ke `GroupRoster` lebih dulu.


## games — games (3 catatan / 3 file)

### `games/ekonomi.rs` (1)

- L3: `db.updateKoin` / `db.updateEnergi` / `addExpWithLevelCheck` di `games.js` butuh `sock` + `db` (I/O), yang belum ada di `Ctx`. Hanya fungsi hitungan polos yang dipindah ke sini.

### `games/lib.rs` (1)

- L163: JS memiliki satu instance module-level `games`. Di Rust ini adalah konstruktor; pemanggil yang ingin singleton sama harus membagikan `FotoGames` melalui `Arc<Mutex<_>>` atau state router.

### `games/mod.rs` (1)

- L5: orchestrator. Lihat di tiap fungsi yang butuh I/O WA/store yang belum ada di `Ctx` class A.


## cek — cek (3 catatan / 3 file)

### `cek/cekowner.rs` (1)

- L3: `config.isOwner/isPartner/isPremium` membaca `config.js`, yang tidak ada di `Ctx`. Yang dipakai sebagai gantinya adalah daftar `owner`/`partner`/`premium` dari settings — satu-satunya sumber yang memang membaca store, dan itu juga yang ditulis `owner/*` plugin. Owner list di `config.js` sendiri tidak terlihat di sini.

### `cek/cekpartner.rs` (1)

- L3: `db.data.partner` adalah store top-level, bukan `db.data.settings`. `Ctx` hanya membawa settings, jadi daftarnya dibaca dari `ctx.setting_list("partner")` — sama seperti `owner/delpartner.rs`.

### `cek/cekprem.rs` (1)

- L3: `config.isPremium/isOwner` membaca `config.js`, yang tidak ada di `Ctx`. Gantinya: daftar `owner`/`partner`/`premium` dari settings. Sifatnya dipertahankan: di `config.js`, owner selalu dianggap premium dan partner selalu dianggap premium, jadi daftar yang-shortcut itu ikut ditiru agar plug-in ini tidak memberi jawaban lebih longgar dari JS.


## anime — anime (3 catatan / 2 file)

### `anime/mywaifu.rs` (1)

- L3: `crypto.randomBytes(32)` (messageSecret) dan `sock.waUploadToServer`/`relayMessage` diabaikan — `Reply` tidak bawa contextInfo/messageSecret, dan pipeline yang meng-upload media Url. `m.react('🕕')`/`'✅'`/`'❌'` → satu reaksi terakhir. `te(...)` → `crate::common::error_template(&ctx)`.

### `anime/topanime.rs` (2)

- L6: JS mengirim gambar dengan `{ quoted: m }`. `Reply::Image` tidak punya penanda kutipan, jadi pipeline yang menentukan.
- L37: hanya array dan string yang punya `.length`; objek, angka, dan boolean mengembalikan `undefined`, jadi `Math.min(undefined, 10)` = `NaN` dan loop `i < NaN` tidak pernah jalan. `None` di sini berarti `NaN` itu.


## nsfw — nsfw (2 catatan / 1 file)

### `nsfw/paptt.rs` (2)

- L3: axios di JS mengirim dua header per-request (`User-Agent` Chrome/150 dan `Content-Type: application/json`). `HttpGet` tidak punya setter header, jadi keduanya tidak ikut — `UreqHttp` sudah memasang UA-nya sendiri untuk GET. ponytail: `console.error("[PAPTT Error]", e)` di cabang `catch` tidak punya padanan — `Reply` tidak membawa log. `pluginConfig` JS tidak punya field `example`, jadi `example` kosong di sini — bukan diisi tebakan.
- L8: `console.error("[PAPTT Error]", e)` di cabang `catch` tidak punya padanan — `Reply` tidak membawa log. `pluginConfig` JS tidak punya field `example`, jadi `example` kosong di sini — bukan diisi tebakan.


## ephoto — ephoto (2 catatan / 1 file)

### `ephoto/ephoto.rs` (2)

- L23: JS melempar di langkah 3 kalau `set-cookie` kosong (`initialResponse.headers['set-cookie'].join`), dan `sendMedia` di dalam `try` gagal kalau URL hasilnya tidak bisa diunduh — keduanya jatuh ke template error yang sama, jadi di sini keduanya `Err`.
- L369: elemen tanpa atribut `value` menghasilkan string kosong (`cheerio` `.val()`), sedangkan `input` tidak ditemukan sama sekali menghasilkan `undefined` di JS — di sini jadi `Err` agar jatuh ke template error, bukan mengirim `"undefined"` ke server.


## utility — utility (2 catatan / 1 file)

### `utility/inspect.rs` (2)

- L3: `sock.groupGetInviteInfo` dan `sock.newsletterMsg` butuh I/O Baileys yang tidak ada di `Ctx`; handler untuk teks tidak kosong diakhiri dengan `error_template` + `☢`, pola `swm.rs`. Fungsi pemola pola (regex) dan teks pakai di-port agar bisa diuji. Saat `sock` tersedia, tambahkan `pub mod inspect;` di `src/utility/mod.rs` secara manual (mod.rs milik plugin lain — dilarang menyentuh per brief).
- L13: dicatat di di atas. Kartu pakai (`inspect.js:22-28`).


## asupan — asupan (1 catatan / 1 file)

### `asupan/asupan.rs` (1)

- L6: JS membaca 8 berkas JSON lalu mengambil byte video lewat HTTP. Di Rust berkas dibaca lewat `ctx.berkas()` dan byte video lewat `ctx.http.get()`. Reaksi `🕕` / `❌` dari JS diruntuhkan jadi satu reaksi terakhir karena `Reply::with_reaction` hanya boleh satu emoji.


## tts — tts (1 catatan / 1 file)

### `tts/ttselon.rs` (1)

- L3: `m.react('🚀')` lalu `✅`/`❌`/`☢` — satu reaksi terakhir. `ctx.config()` tidak dibaca (API tidak butuh kunci). `sock.sendMessage({ audio, contextInfo: saluranCtx() })` → `Reply::voice_note` (contextInfo/saluran diabaikan, `Reply` tidak bawa contextInfo). `fs` temp + ffmpeg → `ctx.berkas()` + `oktz_plugin_api::ffmpeg`. `te(...)` → `crate::common::error_template(&ctx)`.


## media — media (1 catatan / 1 file)

### `media/music.rs` (1)

- L9: `sock.sendMedia(..., { type: 'audio' })` di JS menerima URL dan mengunduhnya sendiri. Di Rust `Reply::audio(MediaSource::url(..))` juga menampilkan URL, jadi bentuknya sama — bedanya hanya file_name: JS tidak menentukan nama berkas untuk audio ini, dan `Reply::Audio` juga tidak punya kolom itu.


## religi — religi (1 catatan / 1 file)

### `religi/jadwalsholat.rs` (1)

- L3: `saluranCtx()` dari `src/lib/context.js` tidak tersedia di API plugin, jadi `contextInfo` (newsletter) diabaikan. Audio adzan dan teks dikirim berurutan lewat `Reply::sequence` (audio dulu, teks sesudahnya, mirip `await` berurutan di JS). Satu reaksi akhir (`✅`/`❌`/`☢`).


## pushkontak — pushkontak (1 catatan / 1 file)

### `pushkontak/pushkontak.rs` (1)

- L21: saat state sesi global, resolusi LID, dan interactive buttons tersedia, implementasikan ulang. `mod.rs` sudah memuat modul ini.

