//! Kelola owner bot (mode-aware).
//!
//! `Keterbatasan:` saat DB baca-tulis, `config.pterodactyl`, dan resolusi LID tersedia,
//! implementasikan ulang dan tambahkan `pub mod addowner;` di `src/owner/mod.rs` secara
//! manual (mod.rs milik plugin lain — dilarang menyentuh per brief).

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "addowner",
    alias = ["addown", "setowner", "delowner", "dedown", "ownerlist", "listowner"],
    category = "owner",
    description = "Kelola owner bot (mode-aware) — BLOKIRAN",
    usage = ".addowner <nomor/@tag/reply>",
    example = ".addowner 6281234567890",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true
)]
pub fn addowner(_ctx: Ctx) -> Reply {
    Reply::text(
        "❌ *ADD OWNER — TIDAK TERSEDIA*\n\n\
         Port Rust tertunda karena butuh:\n\
         • baca-tulis `db.data.owner` dan `db.setting(\"ownerNames\")`\n\
         • `config.pterodactyl` (ownerPanels, sellers)\n\
         • jadibot-database + resolusi LID\n\
         • `getGroupMode` untuk mode cpanel",
    )
}
