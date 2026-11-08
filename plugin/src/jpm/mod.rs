//! Kategori `jpm` — sistem broadcast massal (JPM).
//!
//! Catatan: `pub mod jpm;` ditambahkan di sini karena kategori `jpm` belum
//! ada sama sekali di `rust/crates/oktz-plugins/src/` (rule 7 PORT-BRIEF).
//! Plugin lain di kategori ini, kalau ada, harusaftarkan `pub mod`-nya sendiri.

#[allow(clippy::module_inception)] // struktur folder mengikuti plugin JS
pub mod jpm;
