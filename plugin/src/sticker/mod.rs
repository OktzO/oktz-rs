// Modul plugin kategori `sticker` — cerminan `oktz-md/plugins/sticker/*.js`.
//
// `ttp.rs` ikut di sini: berkasnya ada di `plugins/sticker/ttp.js` dan
// kategori `maker` tidak punya folder modul sendiri, jadi paling aman
// mendeklarasikan modulnya di sini sementara `PluginMeta::category`
// tetap `"maker"` mengikuti JS (`ttp.js:8`).
//
// `bratgreen`/`bratanime`/`bratcewek` hidup di `bratstatic.rs` dan
// `bratvid2`/`bratlocal` di `bratvid.rs` — jangan daftarkan berkas lama
// versi HTTP (bratgreen.rs, animebrat.rs, bratcewek.rs, bratvid2.rs);
// mendeklarasikan keduanya membuat key registry dobel.
pub mod attp;
pub mod attp2;
pub mod brat;
pub mod bratextra;
pub mod bratstatic;
pub mod bratvid;
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
pub mod ttp;

// Nama berkasnya memakai tanda hubung (`smeme-animated.js` → `smeme-animated.rs`),
// jadi modul perlu `#[path]` agar Rust mencarinya dengan nama itu.
#[path = "smeme-animated.rs"]
pub mod smeme_animated;
