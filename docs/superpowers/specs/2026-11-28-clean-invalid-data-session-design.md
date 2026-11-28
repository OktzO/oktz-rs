# Desain: bersihkan data invalid + session basi

Tanggal: 2026-11-28

## Tujuan

1. Plugin owner untuk menghapus entri `users`/`groups` yang tidak mungkin dipakai bot.
2. `clearsessiondb` menghapus artefak session yang sudah kedaluwarsa di lokal dan Turso tanpa menyentuh session aktif.

## Batasan

- Hanya key rusak/tidak terjangkau yang dihapus. User/grup valid tidak disentuh.
- Hapus langsung tanpa preview (pilihan user), tetapi hanya owner.
- `prekeys`, `signed_prekeys`, `sessions`, `identities`, `sender_keys` tidak dihapus paksa.
- Tidak menambah dependency baru.
- Perubahan lewat `Mutation` + jalur `apply` pipeline supaya disk dan Turso tetap sinkron.

## Aturan validitas

User valid bila:

- `lid:<digit>` dengan 10–20 digit, atau
- 1–15 digit ASCII, tidak diawali `120` (pola ID grup), dan
- value berupa object JSON.

Grup valid bila:

- `<digit>@g.us` dengan minimal 10 digit sebelum `@`, dan
- value berupa object JSON.

Selain itu dianggap invalid.

## Session

Lokal:

- `.broken-*` di `OKTZ_DB_DIR`/`database/wa` (perilaku lama).
- SQLite `oktz-run.db`: hapus `msg_secrets` dengan `expires_at != 0 AND expires_at <= now`, `sent_messages`/`pending_inbound_messages` lebih tua dari 7 hari, `tc_tokens` lebih tua dari 30 hari pada kedua timestamp.

Turso:

- `session_state` selain scope `main`/`main:*`.
- Tabel legacy `session_creds` dan `session_keys`.

## Alasan prekey tidak dihapus

X3DH: prekey sekali pakai baru boleh dihapus setelah dipakai. Library sudah menghapusnya saat konsumsi. Menghapus prekey yang belum dipakai membuat sesi baru gagal didekripsi. Session/identity/sender key lama juga dipertahankan karena pembersihan agresif menyebabkan `No session`/`Bad MAC` pada pesan tertunda.
