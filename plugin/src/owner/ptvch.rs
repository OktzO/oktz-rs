//! Kirim video sebagai PTV ke channel.
//!
//! `Keterbatasan:` kalau `download_media` dan variant video PTV tersedia di `Ctx`/`Reply`
//! nanti, tambahkan (reaksi `🕕` lalu `✅`, pesan konfirmasi ke pemilik).

use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "ptvch",
    alias = ["ptvchanel", "ptvstory"],
    category = "owner",
    description = "Kirim video sebagai PTV ke channel (DIBLOKIR: butuh download media + kirim video PTV via sock)",
    usage = ".ptvch (reply video)",
    example = ".ptvch",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ptvch(_ctx: Ctx) -> Reply {
    Reply::None
}

#[cfg(test)]
mod tests {
    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("ptvch").unwrap();
        assert_eq!(p.meta.name, "ptvch");
        assert_eq!(p.meta.alias, ["ptvchanel", "ptvstory"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }
}
