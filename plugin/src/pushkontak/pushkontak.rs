//! Push pesan ke semua member grup + auto simpan kontak VCF.
//!
//! `Keterbatasan:` saat state sesi global, resolusi LID, dan interactive buttons tersedia,
//! implementasikan ulang. `mod.rs` sudah memuat modul ini.

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "pushkontak",
    alias = [
        "puskontak", "push", "stoppush", "setjedapush",
        "pushkontak_start", "kelolapush", "autovcf_on", "autovcf_off",
        "kodeunik_on", "kodeunik_off", "vcftarget_private", "vcftarget_group",
        "skipadmin_on", "skipadmin_off",
    ],
    category = "pushkontak",
    description = "Push pesan ke semua member grup + auto simpan kontak VCF — BLOKIRAN",
    usage = ".pushkontak",
    example = ".pushkontak",
    cooldown = 10,
    energi = 0,
    is_group = true,
    is_private = false,
    is_owner = true
)]
pub fn pushkontak(ctx: Ctx) -> Reply {
    let _ = ctx;
    Reply::text(
        "❌ *PUSH KONTAK — TIDAK TERSEDIA*\n\n\
         Port Rust tertunda karena butuh:\n\
         • state sesi global antar pesan (`pushkontakSessions`)\n\
         • resolusi LID (`resolveAnyLidToJid`)\n\
         • `sock.user.id` (nomor bot sendiri)\n\
         • interactive buttons / native flow\n\
         • kirim dokumen VCF per member dengan jeda\n\n\
         Contoh JS: `.pushkontak`",
    )
}
