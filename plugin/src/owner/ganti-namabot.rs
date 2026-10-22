//! `plugins/owner/ganti-namabot.js` — ganti nama bot di config.js.
//!
//! Port dari nol. Yang dipindah:
//! - bantuan saat nama kosong (`ganti-namabot.js:24-26`).
//! - penulisan `config.js` (`:30-38`).
//! - teks sukses (`:40`).
//!
//! `ponytail:` objek config live (`config.bot.name = newName`) tidak bisa
//! dimutasi di `Ctx` (`config` berupa `Arc`), sama seperti `ganti-namadev`.
//! Yang ditulis hanya berkas `config.js`.
//!
//! `ponytail:` regex JS `/bot:\s*\{[\s\S]*?name:\s*['"]([^'"]*)['"]/`
//! (`ganti-namabot.js:32-36`) digantikan pemindai sederhana — crate `regex`
//! tidak ada di `oktz-plugins`.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Jalur berkas konfigurasi (`ganti-namabot.js:30`).
const CONFIG_PATH: &str = "config.js";

/// Bantuan saat nama kosong (`ganti-namabot.js:24-26`).
pub(crate) fn bantuan(prefix: &str, nama_saat_ini: &str) -> String {
    format!(
        "🤖 *ɢᴀɴᴛɪ ɴᴀᴍᴀ ʙᴏᴛ*\n\n\
         > Nama saat ini: *{nama_saat_ini}*\n\n\
         *Penggunaan:*\n\
         `{prefix}ganti-namabot <nama baru>`"
    )
}

/// Teks sukses (`ganti-namabot.js:40`).
pub(crate) fn sukses(nama_baru: &str) -> String {
    format!("✅ *ʙᴇʀʜᴀsɪʟ*\n\n> Nama bot diganti ke: *{nama_baru}*")
}

/// Ganti nilai `name` di dalam blok `bot: { … }`.
///
/// Padanan `/bot:\s*\{[\s\S]*?name:\s*['"]([^'"]*)['"]/` +
/// penggantian dengan `'nama baru'` (`ganti-namabot.js:32-36`).
fn ganti_nama_bot(content: &str, nama_baru: &str) -> Option<String> {
    let bot_idx = content.find("bot:")?;
    let colon_end = bot_idx + "bot:".len();
    let setelah = &content[colon_end..];
    let ws = setelah.len() - setelah.trim_start().len();
    let brace_rel = setelah[ws..].find('{')?;
    let brace = colon_end + ws + brace_rel;

    let rest = &content[brace..];
    let name_rel = rest.find("name:")?;
    let name_idx = brace + name_rel;

    let after_name = &content[name_idx + "name:".len()..];
    let ws2 = after_name.len() - after_name.trim_start().len();
    let val_start = name_idx + "name:".len() + ws2;

    let quote = content[val_start..].chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let end_rel = content[val_start + 1..].find(quote)?;
    let end = val_start + 1 + end_rel;

    let mut out = String::with_capacity(content.len() + nama_baru.len());
    out.push_str(&content[..val_start]);
    out.push('\'');
    out.push_str(nama_baru);
    out.push('\'');
    out.push_str(&content[end + quote.len_utf8()..]);
    Some(out)
}

#[plugin(
    name = "ganti-namabot",
    alias = ["setnamabot", "gantibot"],
    category = "owner",
    description = "Ganti nama bot di config.js",
    usage = ".ganti-namabot <nama baru>",
    example = ".ganti-namabot Bot Baru",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ganti_namabot(ctx: Ctx) -> Reply {
    let nama_baru = ctx.args.join(" ");

    if nama_baru.is_empty() {
        let sekarang = ctx.config().bot.name.clone();
        let sekarang = if sekarang.is_empty() {
            "-".to_string()
        } else {
            sekarang
        };
        return Reply::text(bantuan(&ctx.prefix, &sekarang));
    }

    let konten = match ctx.berkas().baca_teks(CONFIG_PATH) {
        Ok(k) => k,
        Err(_) => return Reply::text(crate::common::error_template(&ctx)),
    };

    let baru = ganti_nama_bot(&konten, &nama_baru).unwrap_or(konten);

    if ctx.berkas().tulis_teks(CONFIG_PATH, &baru).is_err() {
        return Reply::text(crate::common::error_template(&ctx));
    }

    Reply::text(sukses(&nama_baru))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::berkas::{Berkas, BerkasPalsu};
    use oktz_plugin_api::config::{BotInfo, Runtime};
    use oktz_plugin_api::find;
    use std::sync::Arc;

    const ME: &str = "628111@s.whatsapp.net";

    fn berkas_dengan(konten: &str) -> Arc<BerkasPalsu> {
        let b = BerkasPalsu::baru();
        b.tulis_teks(CONFIG_PATH, konten).unwrap();
        Arc::new(b)
    }

    fn run(berkas: Arc<BerkasPalsu>, args: &[&str], nama_bot: &str) -> Reply {
        ganti_namabot(Ctx {
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "ganti-namabot".into(),
            berkas,
            config: Arc::new(Runtime {
                bot: BotInfo {
                    name: nama_bot.into(),
                    ..BotInfo::default()
                },
                ..Runtime::default()
            }),
            ..Ctx::default()
        })
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("ganti-namabot").unwrap();
        assert_eq!(p.meta.name, "ganti-namabot");
        assert_eq!(p.meta.alias, ["setnamabot", "gantibot"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Ganti nama bot di config.js");
        assert_eq!(p.meta.usage, ".ganti-namabot <nama baru>");
        assert_eq!(p.meta.example, ".ganti-namabot Bot Baru");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["setnamabot", "gantibot"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("ganti-namabot"), "{a}");
        }
    }

    #[test]
    fn nama_kosong_tampil_bantuan() {
        let b = berkas_dengan("bot: { name: 'Oktz' }");
        let r = run(b, &[], "Oktz");
        assert_eq!(
            text_of(&r),
            "🤖 *ɢᴀɴᴛɪ ɴᴀᴍᴀ ʙᴏᴛ*\n\n\
             > Nama saat ini: *Oktz*\n\n\
             *Penggunaan:*\n\
             `.ganti-namabot <nama baru>`"
        );
    }

    #[test]
    fn nama_bot_kosong_pakai_tanda_hubung() {
        let b = berkas_dengan("bot: { name: 'Oktz' }");
        let r = run(b, &[], "");
        assert!(text_of(&r).contains("> Nama saat ini: *-*"), "{}", text_of(&r));
    }

    #[test]
    fn nama_baru_ditulis_ke_config_js() {
        let b = berkas_dengan("bot: {\n  name: 'Oktz',\n  version: '1.0'\n}");
        let r = run(b.clone(), &["Bot", "Baru"], "Oktz");
        assert_eq!(text_of(&r), "✅ *ʙᴇʀʜᴀsɪʟ*\n\n> Nama bot diganti ke: *Bot Baru*");
        assert_eq!(
            b.baca_teks(CONFIG_PATH).unwrap(),
            "bot: {\n  name: 'Bot Baru',\n  version: '1.0'\n}"
        );
    }

    #[test]
    fn nama_ganda_dipakai_tanda_kutip() {
        let b = berkas_dengan("bot: { name: \"Oktz\" }");
        let r = run(b.clone(), &["Baru"], "Oktz");
        assert_eq!(text_of(&r), sukses("Baru"));
        assert_eq!(b.baca_teks(CONFIG_PATH).unwrap(), "bot: { name: 'Baru' }");
    }

    #[test]
    fn berkas_tidak_ada_menggunakan_error_template() {
        let b = Arc::new(BerkasPalsu::baru());
        let r = run(b, &["Baru"], "Oktz");
        assert_eq!(
            text_of(&r),
            crate::common::error_template_bawaan(".", "ganti-namabot")
        );
    }
}
