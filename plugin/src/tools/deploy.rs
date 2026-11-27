//! Deploy HTML atau file ZIP ke Vercel.
//!
//! `Keterbatasan:` kalau `config.vercel` ditambahkan ke `Runtime` nanti, tambahkan (guard
//! token, reaksi `🕕`, ekstrak ZIP/HTML, POST ke `https://api.vercel.com/v13/deployments`,
//! GET domains, fallback ke `${name}.vercel.app`).

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "deploy",
    alias = ["vercel"],
    category = "owner",
    description = "Deploy HTML atau file ZIP ke Vercel (DIBLOKIR: config.vercel belum tersedia di Rust)",
    usage = ".deploy <namawebsite>",
    example = ".deploy mysite",
    cooldown = 60,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn deploy(_ctx: Ctx) -> Reply {
    Reply::None
}
