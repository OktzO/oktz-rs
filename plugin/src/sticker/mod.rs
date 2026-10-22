// Modul plugin kategori `sticker` — cerminan `oktz-md/plugins/sticker/*.js`.
//
// Catatan: `ttp.rs` sengaja TIDAK didaftarkan di sini. Metadata plugin-nya
// memakai kategori `"maker"` (`oktz-md/plugins/sticker/ttp.js:8`), jadi
// pendaftarannya milik kategori `maker`, bukan `sticker`.
//
// Ditambahkan scripts/pasang-modul.mjs.
pub mod animebrat;
pub mod attp;
pub mod attp2;
pub mod brat;
pub mod bratcewek;
pub mod bratextra;
pub mod bratgreen;
pub mod bratvid;
pub mod bratvid2;
pub mod emojimix;
pub mod linesticker;
pub mod pinpack;
pub mod qc;
pub mod smeme;
pub mod smemevid;
pub mod sticker;
pub mod stickerly;
pub mod stickerpack;
pub mod swm;

// Nama berkasnya memakai tanda hubung (`smeme-animated.js` → `smeme-animated.rs`),
// jadi modul perlu `#[path]` agar Rust mencarinya dengan nama itu.
#[path = "smeme-animated.rs"]
pub mod smeme_animated;
