//! Auto-hapus semua chat grup & private tiap
//!
//! `Keterbatasan:` kalau `chat-cleaner` diekspose lewat `Ctx` nanti, tambahkan (guard
//! `on`/`off`/`status`, reaksi `✅`, dan teks respons yang sama).

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "autoclear",
    alias = [],
    category = "owner",
    description = "Auto-hapus semua chat grup & private tiap tengah malam (DIBLOKIR: chat-cleaner.js belum tersedia di Rust)",
    usage = ".autoclear on|off|status",
    example = ".autoclear on",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn chatclear(_ctx: Ctx) -> Reply {
    Reply::None
}
