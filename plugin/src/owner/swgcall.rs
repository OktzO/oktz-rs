//! Post Group Status/Story ke SEMUA grup.
//!
//! `Keterbatasan:` saat `Sock` mendukung group status + `semua_grup()`, dan `Reply`
//! mendukung interactive buttons, implementasikan ulang dan tambahkan `pub mod swgcall;` di
//! `src/owner/mod.rs` secara manual.

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "swgcall",
    alias = ["swgcsemua", "swgcbroadcast", "swgcbc", "groupstoryall"],
    category = "owner",
    description = "Post Group Status/Story ke SEMUA grup (border hijau) — BLOKIRAN",
    usage = ".swgcall <teks> atau reply media",
    example = ".swgcall Pengumuman penting!",
    cooldown = 30,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true
)]
pub fn swgcall(_ctx: Ctx) -> Reply {
    Reply::text(
        "❌ *SWGCALL — TIDAK TERSEDIA*\n\n\
         Port Rust tertunda karena butuh:\n\
         • `sock.sendMessage(..., { groupStatusMessage })`\n\
         • `sock.groupFetchAllParticipating()`\n\
         • deteksi tipe media (file-type)\n\
         • state global antar pesan (`_swgcallPending`)\n\
         • interactive buttons / quick reply\n\n\
         Contoh JS: `.swgcall Pengumuman penting!`",
    )
}
