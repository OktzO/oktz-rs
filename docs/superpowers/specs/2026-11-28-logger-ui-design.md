# Desain: Tampilan logger gaya badge (bukan rail)

## Masalah
Tampilan rail sekarang dinilai buruk. Owner mau gaya badge seperti modul usulannya:
`[jam] [BADGE] nama(id) [REPLY] » pesan`.

## Tujuan
- Badge tipe chat: `PRIVATE`, `GROUP`, `CHANNEL` (+ `STATUS`; kode sudah bedakan `@broadcast`).
- Penanda `[REPLY]` untuk pesan masuk yang membalas.
- Pesan masuk yang BUKAN command → isi ditulis `NoCmd`, bukan teks penuh (user kadang kirim teks sangat panjang).
- AI dihapus total (tidak ada `log_ai`).
- Log tracing bisa disembunyikan/diatur lewat `.env`; format lebih ringkas.
- Penanda polos `oktz bot online` (saat konek) untuk deteksi Running panel Pterodactyl.

## Non-tujuan
- Tidak mengubah paritas pesan WhatsApp.
- Teks balasan bot tidak dicetak (cukup baris status kirim).

## Pilihan yang sudah diputuskan
Owner menolak A (rail + penanda). Memilih **B**: ganti penuh ke gaya badge. Warna
lewat crate `owo-colors` (owner minta), bukan raw SGR.

## Cara (yang dibangun)
1. `lib/oktz-wa/ui.rs` — ganti rail jadi badge:
   - `enum TipeChat { Private, Group, Channel, Status }` (badge = tipe saja, tanpa nama grup).
   - `pesan_masuk(ts, tipe, nama, id, is_reply, isi)` → `[jam] [BADGE] nama(id)[ [REPLY]] » isi`.
   - `status_kirim(ts, ok, tujuan, dur_ms, alasan)` → `[jam] ↳ ✓ Terkirim → X · ms` / `↳ ✕ Gagal → X · alasan`.
   - `format_masuk`/`format_status` (murni, param `aktif: bool`) yang dites; pembungkus publik pakai `warna_aktif()`.
   - Warna badge: Private biru, Group hijau, Channel violet, Status abu; `id`/sisa redup; `[REPLY]` kuning.
   - `pub const PENANDA_ONLINE = "oktz bot online"` (polos, regex-safe untuk panel).
2. `lib/oktz-wa/pipeline.rs`
   - `tipe_chat_untuk(is_group, chat)` → `TipeChat`.
   - `isi_log(is_command, body)` → body atau `"NoCmd"`.
   - Deteksi reply: `context_info_of(&m.message)` → `quoted_message.is_some()`.
   - Titik kirim pakai `ui::status_kirim` (signature sama).
3. `lib/oktz-core/logging.rs` — formatter `[jam] [tag] field=val` (tag = segmen target terakhir); `OKTZ_LOG` (`off|error|warn|info|debug|trace`, default `info`); `off` = tracing mati; `RUST_LOG` menang atas `OKTZ_LOG`.
4. `src/main.rs` — `Env::load_dotenv()` sebelum `logging::init`; cetak `ui::PENANDA_ONLINE` saat konek.
5. `.env.example` + `README.md` — dokumentasi `OKTZ_LOG` + set panel Start Configuration `done` = `oktz bot online`.

## Keputusan akhir
- Badge grup: tipe saja `[GROUP]` (tanpa nama grup).
- Baris sukses kirim: `↳ ✓ Terkirim → X · ms` (bukan `└─ BOT ke X » teks`).
- Penanda panel: `oktz bot online` (user set field startup/running-detection ke string ini).
