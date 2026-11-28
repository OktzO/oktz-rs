# Desain: Tampilan logger gaya badge (bukan rail)

## Masalah
Tampilan rail sekarang dinilai buruk. Owner mau gaya badge seperti modul usulannya:
`[jam] [BADGE] nama(id) [REPLY] » pesan` + baris balasan bot `└─ BOT ke X » pesan`.

## Tujuan
- Badge tipe chat: `PRIVATE`, `GROUP`, `CHANNEL` (+ `STATUS`; kode sudah bedakan `@broadcast`).
- Penanda `[REPLY]` untuk pesan masuk yang membalas.
- Pesan masuk yang BUKAN command → isi ditulis `NoCmd`, bukan teks penuh (user kadang kirim teks sangat panjang).
- Baris balasan bot `└─ BOT ke <tujuan> » <teks>`.
- AI dihapus total (tidak ada `log_ai`).
- Log tracing bisa disembunyikan/diatur lewat `.env`; format lebih ringkas.

## Non-tujuan
- Tidak pakai crate `owo_colors`; tetap raw SGR seperti `ui.rs` (dep tetap ramping).
- Tidak mengubah paritas pesan WhatsApp.

## Pilihan yang sudah diputuskan
Owner menolak A (rail + penanda). Memilih **B**: ganti penuh ke gaya badge.

## Cara
1. `lib/oktz-wa/ui.rs` — ganti rail jadi badge:
   - `enum TipeChat { Private, Group { nama: String }, Channel, Status }`.
   - `pesan_masuk(ts, tipe, nama, id, is_reply, isi)` → `[jam] [BADGE] nama(id) [REPLY] » isi`
   - `balasan_bot(ts, tujuan, isi)` → `[jam] └─ BOT ke tujuan » isi`
   - Warna badge: Private biru, Group hijau, Channel violet, Status abu.
2. `lib/oktz-wa/pipeline.rs`
   - `label_chat_untuk` balikin `TipeChat` (+ nama grup).
   - Deteksi reply: `context_info_of(&m.message)` → `quoted_message.is_some()`.
   - Deteksi command: `parse_command(&body, &self.parse_opts).is_command`; non-command → isi `"NoCmd"`.
   - Ganti `ui::status_kirim` di titik kirim jadi baris `↳ ✓ Terkirim → X · ms` / `↳ ✕ Gagal → X · alasan` (signature sama; teks balasan bot tidak dicetak).
3. `lib/oktz-core/logging.rs` — `init` baca `OKTZ_LOG` (`off|error|warn|info|debug|trace`, default `info`); `off` = tracing mati. `RUST_LOG` tetap menang. Format `HH:MM:SS LEVEL: pesan` + `with_target(false)`.
4. `src/main.rs` — `Env::load_dotenv()` sebelum `logging::init`.
5. `.env.example` + `README.md` — dokumentasi `OKTZ_LOG`.

## Perlu konfirmasi
- Badge grup: ikut nama grup (`[GROUP · Keluarga Semur]`) atau tipe saja (`[GROUP]`)?
- `✓ terkirim → X · ms` diganti `└─ BOT ke X » teks`, atau dua-duanya?
