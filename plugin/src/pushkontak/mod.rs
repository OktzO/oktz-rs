//! `plugins/pushkontak/` — kategori baru untuk plugin push kontak.
//!
//! `//! NOTE (mod.rs):` kategori ini belum ada sebelumnya di `src/`,
//! sehingga folder dan `mod.rs` ini dibuat baru (izin pada brief aturan 7).
//! Hanya `pushkontak.rs` yang dimiliki tugas ini; `mod.rs` memuatnya.

// Folder `pushkontak/` meniru layout `plugins/pushkontak/` di JS, jadi nama
// modul sengaja sama dengan nama folder induknya.
#[allow(clippy::module_inception)]
pub mod pushkontak;
