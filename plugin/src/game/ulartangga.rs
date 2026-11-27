//! Ular tangga bersama player lain dengan visual board.
//!
//! `Keterbatasan:` saat state game + canvas board tersedia, implementasikan ulang dan
//! tambahkan `pub mod ulartangga;` di `src/game/mod.rs` secara manual.

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "ulartangga",
    alias = ["ut", "snakeladder", "sl"],
    category = "game",
    description = "Main ular tangga bersama player lain dengan visual board — BLOKIRAN",
    usage = ".ulartangga <create|join|start|info|exit|delete>",
    example = ".ulartangga create",
    cooldown = 5,
    energi = 0,
    is_group = true,
    is_private = false
)]
pub fn ulartangga(ctx: Ctx) -> Reply {
    let _ = ctx;
    Reply::text(
        "❌ *ULAR TANGGA — TIDAK TERSEDIA*\n\n\
         Port Rust tertunda karena butuh:\n\
         • render board kustom (`drawBoard`)\n\
         • `sock.sendPreview`\n\
         • state game global antar pesan\n\
         • DB koin/energi/user\n\n\
         Contoh JS: `.ulartangga create`",
    )
}
