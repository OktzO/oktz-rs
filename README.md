# oktz-rs

Bot WhatsApp versi Rust dari `ourin-md`. Perilaku mengikuti bot JS produksi.

Repo ini TIDAK menyimpan binary. Binary Linux x64 diunduh otomatis dari
GitHub Release saat `launcher.js` pertama kali dijalankan (disimpan ke
`bin/oktz`), jadi clone repo tetap kecil dan panel tidak butuh toolchain Rust.

## Jalankan

```bash
node launcher.js                 # = oktz run (daemon sampai SIGINT/SIGTERM)
node launcher.js pair-code 628xxx  # pairing nomor (tanpa argumen: session.pairingNumber)
node launcher.js pair-qr           # pairing QR
node launcher.js verify-db <path>  # cek db sesi
```

Launcher mencari `bin/oktz`, lalu `target/release/oktz`; kalau tidak ada, launcher
mengunduh aset `oktz-linux-x64` dari release terbaru. Bisa dipaksa lewat
`OKTZ_BIN=/path/ke/oktz`.

Unduh manual (misal mesin tanpa internet): ambil `oktz-linux-x64` dari halaman
release, simpan sebagai `bin/oktz`, `chmod +x bin/oktz`.

Setelah pairing, creds tersimpan di `storage/session/creds.json` dan sesi
ratchet di `database/wa/`. Dua direktori itu harus persisten.

## Panel Pterodactyl (egg Node.js)

1. Upload/clone repo ini.
2. Startup command: `node launcher.js` (atau `npm start`).
3. Node 18+; tidak ada `npm install` (tanpa dependensi).
4. Panel harus Linux x86_64 dengan glibc >= 2.34 (Debian 12 / Ubuntu 22.04+).
5. Set `TZ=Asia/Jakarta` supaya tanggal/jam sapaan sama dengan bot JS.
6. Isi `config.json` (lihat bawah), lalu pairing lewat console panel.

Sinyal stop panel diteruskan launcher ke binary; exit code binary = exit code
launcher.

## config.json

Sepadan `config.js`. Nilai yang **diisi** di file menang atas `.env`; string
kosong berarti "jangan sentuh" (API key di `.env` tetap dipakai). Lokasi file
bisa dipindah lewat `OKTZ_CONFIG`.

Bagian penting: `owner`, `bot`, `session` (pairing/QR), `database`, `turso`,
`sticker`, `saluran`, `apiKeys`.

## Build ulang

```bash
cargo build --release
cp target/release/oktz bin/oktz
```

## Verifikasi

```bash
cargo test --release
cargo clippy --release --all-targets
```

## Batas yang diketahui

- Welcome/goodbye baru tipe teks (1); tipe 2..8 (kartu/album), banner
  promote/demote, dan badge newsletter belum ada.
- Sebagian plugin JS belum diport (AI, stalk, game tertentu, arsip, dll);
  plugin stub tidak didaftarkan.
- Turso baru validasi konfigurasi, belum baca/tulis.
