# oktz-rs

Bot WhatsApp berbasis Rust. Implementasi Rust adalah acuan perilaku; tidak ada
lagi kewajiban meniru bot JS lama.

Repo ini TIDAK menyimpan binary. Binary Linux x64 diunduh otomatis dari
GitHub Release saat `launcher.js` pertama kali dijalankan (disimpan ke
`bin/oktz`), jadi clone repo tetap kecil dan panel tidak butuh toolchain Rust.

## Jalankan

```bash
node launcher.js                 # tanpa argumen: creds -> Turso -> pairing, lalu run
node launcher.js pair-code 628xxx  # paksa pairing nomor
node launcher.js pair-qr           # paksa pairing QR
node launcher.js verify-db <path>  # cek db sesi
```

Tanpa argumen launcher mengikuti urutan bot JS lama:

1. `storage/session/creds.json` ada dan sudah paired → `run`.
2. Belum ada sesi, Turso aktif (`TURSO_URL`/`TURSO_AUTH_TOKEN` di `.env`) →
   pulihkan `session_state` scope `main` (snapshot utuh: db SQLite + creds +
   marker), tulis `database/wa/oktz-run.db` + `creds.json` + `.ident`, lalu
   `run`.
3. Belum ada sesi dan tidak ada Turso → pairing code (atau QR bila
   `USE_PAIRING_CODE=false`), simpan `creds.json`, lalu `run`.

Launcher mencari `bin/oktz`, lalu `target/release/oktz`; kalau tidak ada, launcher
mengunduh aset `oktz-linux-x64-static` dari release terbaru (fallback
`oktz-linux-x64`). Bisa dipaksa lewat `OKTZ_BIN=/path/ke/oktz`.

Versi target diambil dari tag rilis terbaru GitHub (redirect `releases/latest`,
bukan API karena IP panel sering kena rate limit 403). Penanda versi disimpan di
`bin/.oktz-versi`; kalau tag berbeda, binary diunduh ulang. Kalau GitHub tidak
bisa dihubungi, binary yang sudah ada tetap dipakai.

Unduh manual (misal mesin tanpa internet): ambil `oktz-linux-x64-static` dari
halaman release, simpan sebagai `bin/oktz`, `chmod +x bin/oktz`.

Setelah pairing, creds tersimpan di `storage/session/creds.json` dan sesi
ratchet di `database/wa/oktz-run.db`. Dua direktori itu harus persisten.

## Panel Pterodactyl (egg Node.js)

1. Upload/clone repo ini.
2. Startup command: `node launcher.js` (atau `npm start`).
3. Node 18+; tidak ada `npm install` (tanpa dependensi).
4. Panel harus Linux x86_64; binary statis dipilih supaya tidak butuh loader glibc.
5. Set `TZ=Asia/Jakarta` supaya tanggal/jam sapaan sama dengan bot JS.
6. Isi `.env` (salin dari `.env.example`). Kalau belum ada sesi, launcher
   pairing sendiri lewat `PAIRING_NUMBER`; kalau Turso diisi, sesi dipulihkan
   dulu.
7. Deteksi "Running" panel: buka *Startup* → **Start Configuration** dan set
   `done` ke `oktz bot online` (baris yang dicetak bot saat tersambung ke
   WhatsApp). Tanpa itu server tetap "Starting" walau bot jalan. Bisa juga
   dikosongkan (`{}`) supaya panel menandai Running begitu proses jalan.

Sinyal stop panel diteruskan launcher ke binary; exit code binary = exit code
launcher.

## .env

Semua config lewat `.env` (salin dari `.env.example`), dibaca binary Rust dan
launcher. Kunci utama: `OWNER_NUMBERS`, `BOT_*`, `PAIRING_NUMBER`,
`USE_PAIRING_CODE`, `DATABASE_PATH`, `TURSO_*`, `STICKER_*`, `SALURAN_*`,
`APIKEY_*`. Config JSON kompleks masih bisa lewat `OKTZ_CONFIG_JSON`.
`OKTZ_LOG` mengatur keriuhan log tracing Rust: `off|error|warn|info|debug|trace`
(default `info`); `off` = senyap total, `RUST_LOG` menimpa.

## Build ulang

```bash
RUSTFLAGS="-C target-feature=+crt-static" \
  cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/oktz bin/oktz
```

## Verifikasi

```bash
cargo test --release
cargo clippy --release --all-targets
node --test launcher.test.js   # logika versi rilis launcher
```

## Batas yang diketahui

- Welcome/goodbye baru tipe teks (1); tipe 2..8 (kartu/album), banner
  promote/demote, dan badge newsletter belum ada.
- Sebagian plugin JS belum diport (AI, stalk, game tertentu, arsip, dll);
  plugin stub tidak didaftarkan.
- Turso menyimpan DB bot (blob JSON per store di tabel `stores`) + snapshot
  sesi utuh (`session_state` scope `main`: db SQLite gzip+base64, creds.json,
  marker `.ident`) — ditulis bot Rust berkala (write-behind), dibaca launcher
  saat lokal hilang. Logout dipercaya menghapus snapshot (backup
  `purged-<ts>` 24 jam). Tabel `session_creds`/`session_keys` lama tidak
  dipakai lagi.
- Pairing QR mencetak payload, belum dirender jadi gambar QR.
