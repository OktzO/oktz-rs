//! Command untuk mengelola scheduled messages.
//!
//! `Keterbatasan:` kalau `scheduler` diekspose lewat `Ctx` nanti, tambahkan seluruh logika
//! `add`/`edit`/`list`/`kategori`/`preset`/`detail`/`del`/`status`.

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "schedule",
    alias = ["sched", "jadwal", "timer"],
    category = "owner",
    description = "Bikin reminder atau jadwal bebas dengan text custom (DIBLOKIR: scheduler.js belum tersedia di Rust)",
    usage = ".schedule <add/edit/list/kategori/preset/detail/del/status> [options]",
    example = ".schedule preset sekolah 06:30",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn schedule(_ctx: Ctx) -> Reply {
    Reply::None
}
