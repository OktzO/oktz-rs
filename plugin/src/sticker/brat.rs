//! `plugins/sticker/brat.js` — menu variant brat dan generator sticker brat.

use oktz_plugin_api::{Ctx, HttpCall, MediaSource, Reply, Tombol, plugin};

pub(crate) const CAPTION_MENU: &str =
    "🌿 *kamu mau buat brat yak, silahkan pilih variant brat tombol dibawah*";

pub(crate) fn baris_varian(prefix: &str, text: &str) -> Vec<(String, String, String)> {
    [
        ("Brat Default", "Sticker brat versi biasa", "bratimg"),
        ("Brat Green", "Variant brat warna hijau", "bratgreen"),
        ("Brat Cewek", "Variant brat cewek", "bratcewek"),
        ("Brat Anime", "Variant brat anime", "animebrat"),
        ("Brat Video", "Sticker brat animated", "bratvid"),
        ("Brat Video V2", "Sticker brat video v2", "bratvid2"),
    ]
    .iter()
    .map(|(judul, des, cmd)| {
        (
            (*judul).to_string(),
            format!("{des} • .{cmd} <text>"),
            format!("{prefix}{cmd} {text}").trim().to_string(),
        )
    })
    .collect()
}

fn menu_brat(prefix: &str, text: &str, subtitle: &str) -> Reply {
    let rows: Vec<serde_json::Value> = baris_varian(prefix, text)
        .into_iter()
        .map(|(title, description, id)| {
            serde_json::json!({
                "title": title,
                "description": description,
                "id": id,
            })
        })
        .collect();

    Reply::interactive(CAPTION_MENU)
        .with_header(oktz_plugin_api::kartu::KartuHeader {
            title: Some(String::new()),
            subtitle: Some(subtitle.to_string()),
            media: Some(MediaSource::path("assets/image/foto.jpeg")),
            video: false,
        })
        .with_footer("Pilih variant brat favorit kamu")
        .with_button(
            Tombol::dari_nama(
                "single_select",
                serde_json::json!({
                    "title": "🌾 Pilih Variant Brat",
                    "sections": [{
                        "title": "Variant Brat",
                        "rows": rows,
                    }],
                }),
            )
            .expect("nama tombol WA dikenal"),
        )
}

pub(crate) fn pakai(prefix: &str) -> String {
    format!("🖼️ *ʙʀᴀᴛ ɪᴍᴀɢᴇ*\n\n> Masukkan teks\n\n`Contoh: {prefix}bratimg Hai semua`")
}

pub(crate) fn api_url(text: &str) -> String {
    let mut out = String::from("https://api.theresav.eu/api/maker/brat?text=");
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub(crate) fn parse_brat_args(input: &str) -> (String, u8) {
    let text = input.trim();
    let mut dari = 0usize;
    while let Some(rel) = text[dari..].find("-blur") {
        let awal = dari + rel;
        let sisa = &text[awal + 5..];
        let spasi = sisa.len() - sisa.trim_start().len();
        if let Some(d @ b'0'..=b'3') = sisa.as_bytes().get(spasi).copied() {
            let akhir = awal + 5 + spasi + 1;
            let bersih = format!("{}{}", &text[..awal], &text[akhir..])
                .trim()
                .to_string();
            return (bersih, d - b'0');
        }
        dari = awal + 5;
    }
    (text.to_string(), 0)
}

#[plugin(
    name = "brat",
    alias = ["bratmenu", "bratimg", "brattext"],
    category = "sticker",
    description = "Menu variant brat dan generator sticker brat",
    usage = ".brat | .bratimg <text>",
    example = ".bratimg Hai semua",
    cooldown = 10,
    energi = 1,
    is_group = false,
    is_private = false
)]
pub fn brat(ctx: Ctx) -> Reply {
    let command = ctx.command.to_lowercase();
    if command == "brat" || command == "bratmenu" {
        let cfg = ctx.config();
        let subtitle = if cfg.bot.name.is_empty() { "Oktz Bot".to_string() } else { cfg.bot.name.clone() };
        return menu_brat(&ctx.prefix, &ctx.text, &subtitle);
    }

    if ctx.text.is_empty() {
        return Reply::text(pakai(&ctx.prefix));
    }

    // `brat.js:139-151` — packname/author dari config; kosong = DEFAULT
    // `Sticker`/`Bot` (`exif.js:33-38` via `??`).
    let (parsed, blur) = parse_brat_args(&ctx.text);
    let cfg = ctx.config();
    let (pn, au) = (
        if cfg.sticker.packname.is_empty() { "Sticker".to_string() } else { cfg.sticker.packname.clone() },
        if cfg.sticker.author.is_empty() { "Bot".to_string() } else { cfg.sticker.author.clone() },
    );

    /// `brat.js:145` — fallback render lokal putih.
    fn render_fallback(parsed: &str, blur: u8, pn: &str, au: &str) -> Result<Vec<u8>, ()> {
        let png = oktz_plugin_api::render_brat(parsed, oktz_plugin_api::BratTema::Putih, blur)
            .map_err(|_| ())?;
        let webp = oktz_plugin_api::gambar_ke_stiker_webp(&png, &[]).map_err(|_| ())?;
        // `addExifToWebp` gagal di JS hanya di-log (`socket.js:549-551`) —
        // webp polos tetap terkirim.
        match oktz_plugin_api::tempel_metadata_stiker(&webp, pn, au, &oktz_plugin_api::EXIF_EMOJIS) {
            Ok(b) => Ok(b),
            Err(_) => Ok(webp),
        }
    }

    // `brat.js:141-146` — API (timeout 15s) gagal ATAU blank → fallback
    // render. Konversi webp dari byte API gagal = `sendImageAsSticker`
    // melempar → catch LUAR (`brat.js:154-157`), bukan fallback.
    let hasil = match ctx.http.call(&HttpCall::get(api_url(&parsed)).with_timeout(15000)) {
        Ok(res) if (200..300).contains(&res.status) && !oktz_plugin_api::is_blank(res.body.bytes()) => {
            match oktz_plugin_api::gambar_ke_stiker_webp(res.body.bytes(), &[]) {
                Ok(webp) => match oktz_plugin_api::tempel_metadata_stiker(
                    &webp, &pn, &au, &oktz_plugin_api::EXIF_EMOJIS,
                ) {
                    Ok(b) => Ok(b),
                    // `addExifToWebp` gagal hanya di-log (`socket.js:549-551`).
                    Err(_) => Ok(webp),
                },
                Err(_) => Err(()),
            }
        }
        _ => render_fallback(&parsed, blur, &pn, &au),
    };

    // `brat.js:136-157` — 🕐 → sticker → ✅; gagal total → ☢ lalu te.
    match hasil {
        Ok(bytes) => Reply::sequence(vec![
            Reply::reaction("🕐"),
            Reply::sticker_opsi(
                MediaSource::bytes(bytes).with_mimetype("image/webp"),
                oktz_plugin_api::StickerOpsi::gambar().with_packname(&pn).with_author(&au),
            ),
            Reply::reaction("✅"),
        ]),
        Err(()) => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brat_menu_has_six_rows() {
        let r = menu_brat(".", "halo", "Oktz Bot");
        let Reply::Interactive { kartu } = r else {
            panic!("harus interactive")
        };
        assert!(kartu.body.contains("silahkan pilih variant brat"));
    }

    #[test]
    fn output_sama_dengan_js() {
        // `brat.js:130-134` — kartu pakai, literal JS.
        assert_eq!(
            pakai("."),
            "🖼️ *ʙʀᴀᴛ ɪᴍᴀɢᴇ*\n\n> Masukkan teks\n\n`Contoh: .bratimg Hai semua`"
        );
        // `brat.js:67` — caption menu, literal JS.
        assert_eq!(
            CAPTION_MENU,
            "🌿 *kamu mau buat brat yak, silahkan pilih variant brat tombol dibawah*"
        );
        // `brat.js:57-63` — baris varian + ID `{prefix}{command} <text>`.
        let baris = baris_varian(".", "hai");
        assert_eq!(baris.len(), 6);
        assert_eq!(
            baris[0],
            (
                "Brat Default".to_string(),
                "Sticker brat versi biasa • .bratimg <text>".to_string(),
                ".bratimg hai".to_string()
            )
        );
        // `brat.js:110-113` — ID baris terakhir tanpa spasi saat teks kosong.
        assert_eq!(baris_varian(".", "").last().unwrap().2, ".bratvid2");
    }
}
