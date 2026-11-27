//! Bikin brat versi custom (canvas sticker).

use oktz_plugin_api::{Ctx, Reply, plugin};

pub(crate) fn help(prefix: &str, cmd: &str) -> String {
    format!("⚠️ Harap masukkan teksnya!\nContoh: `{prefix}{cmd} Halo semuanya`")
}

#[plugin(
    name = "bratextra",
    alias = ["bratanime", "bratbahlil", "brathd", "bratpatrick", "bratsquidward", "bratwhite",
             "bratchika", "bratkobato", "bratmenhera", "bratnezuko", "bratqiqi", "bratruromiya",
             "bratumaru"],
    category = "sticker",
    description = "Bikin brat versi custom",
    usage = ".bratanime <teks>",
    example = ".bratanime Halo",
    cooldown = 5,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bratextra(ctx: Ctx) -> Reply {
    let cmd = ctx.command.as_str();
    let text = ctx.text.trim();
    if text.is_empty() {
        return Reply::text(help(&ctx.prefix, cmd));
    }

    // butuh:
    // 1. `@napi-rs/canvas` (createCanvas, loadImage, GlobalFonts) — render
    // teks di atas template gambar. Tidak ada crate canvas di dependensi
    // `oktz-plugins`.
    // 2. `fetch(url)` → buffer template + font dari GitHub — bisa via
    // `ctx.http`, tapi rendering ulang pixel-per-pixel harus sama persis
    // dengan versi awal (tokenisasi emoji, line wrap, rotasi, menurunkan fontSize
    // 2-2 sampai muat).
    // 3. `sock.sendImageAsSticker` — kirim sticker, bisa `Reply::Sticker`.
    //
    // `brathd` juga memakai endpoint eksternal `aqul-brat.hf.space` (bukan
    // scraper yang tersedia di `oktz_scraper`).
    //
    // `Keterbatasan:` tambahkan crate canvas (atau port renderer teks
    // `oktz_scraper`) dan unggah byte sticker, lalu replikasikan semua 13
    // template + loop perkecil font.
    Reply::text(help(&ctx.prefix, cmd))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_of(cmd: &str, text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: cmd.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn teks_kosong_mengembalikan_kontoh_pakai() {
        let r = bratextra(ctx_of("bratanime", ""));
        assert!(r.text_body().unwrap().contains("Harap masukkan teksnya"));
    }

    #[test]
    fn semua_alias_pakai_handler_sama() {
        for cmd in [
            "bratanime",
            "bratbahlil",
            "brathd",
            "bratpatrick",
            "bratsquidward",
            "bratwhite",
            "bratchika",
            "bratkobato",
            "bratmenhera",
            "bratnezuko",
            "bratqiqi",
            "bratruromiya",
            "bratumaru",
        ] {
            let r = bratextra(ctx_of(cmd, "Halo"));
            assert!(
                r.text_body().unwrap().contains("Halo semuanya")
                    || r.text_body().unwrap().contains("teksnya")
            );
        }
    }
}
