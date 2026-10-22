//! `plugins/cek/cekberat.js` — cek berat badan random.
//!
//! Angka berat = `floor(rand * 60) + 40` (`cekberat.js:22`), ambang tier
//! `90/70/55/45` (`cekberat.js:23-32`), dan dua kalimat "diri sendiri" vs
//! "orang lain" (`cekberat.js:34-40`).
//!
//! Perhatikan: penanda bold di JS ditutup — `*${berat} kg*`.

use crate::common::{num_and_self, roll};
use oktz_plugin_api::{Ctx, Reply, plugin};

/// `floor(rand * 60) + 40` -> 40..=99, yaitu rentang Rust `40..100`.
pub(crate) fn berat_kg() -> u32 {
    roll(60) + 40
}

/// `cekberat.js:22-32`. Ambang 55 dan 45 (bukan kelipatan 10) — inilah yang
/// membedakannya dari `cek.rs`.
pub(crate) fn berat_desc(kg: u32) -> &'static str {
    if kg >= 90 {
        "Big boy/girl! 💪"
    } else if kg >= 70 {
        "Berisi dan sehat! 😊"
    } else if kg >= 55 {
        "Ideal banget! 👍"
    } else if kg >= 45 {
        "Langsing nih~ 🌸"
    } else {
        "Kurus banget, makan yang banyak! 🍔"
    }
}

pub(crate) fn berat_text(ctx: &Ctx, kg: u32) -> String {
    let (number, is_self) = num_and_self(ctx);
    let desc = berat_desc(kg);
    if is_self {
        format!("Hai @{number}\n    \nBerat badan kamu *{kg} kg*\n```{desc}```")
    } else {
        format!(
            "Kamu ingin ngecek berat badan @{number} yak? \n    \nBerat badan dia sebesar *{kg} kg*\n```{desc}```"
        )
    }
}

#[plugin(
    name = "cekberat",
    alias = ["berat", "weight"],
    category = "cek",
    description = "Cek berat badan random",
    usage = ".cekberat <nama>",
    example = ".cekberat Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekberat(ctx: Ctx) -> Reply {
    Reply::Text {
        text: berat_text(&ctx, berat_kg()),
        mentions: vec![ctx.mention_or_sender().to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    /// Tier `cekberat.js:23-32` — urutan dari ambang tertinggi.
    const TIERS: [&str; 5] = [
        "Big boy/girl! 💪",
        "Berisi dan sehat! 😊",
        "Ideal banget! 👍",
        "Langsing nih~ 🌸",
        "Kurus banget, makan yang banyak! 🍔",
    ];

    const ME: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn run(c: Ctx) -> Reply {
        (find("cekberat").unwrap().handler)(c)
    }

    fn ctx_with(mention: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cekberat".into(),
            sender: ME.into(),
            mentioned: mention.map(str::to_string).into_iter().collect(),
            ..Default::default()
        }
    }

    fn split(r: &Reply) -> (String, Vec<String>) {
        match r {
            Reply::Text { text, mentions } => (text.clone(), mentions.clone()),
            other => panic!("harus Reply::Text, dapat {other:?}"),
        }
    }

    /// Ambil angka kilogram di `*<angka> kg*` — satu-satunya angka di balasan.
    fn kg_of(text: &str) -> u32 {
        let body = text
            .split_once('*')
            .and_then(|(_, rest)| rest.split_once(" kg*"))
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| panic!("balasan tanpa bobot: {text}"));
        assert!(
            body.chars().all(|c| c.is_ascii_digit()),
            "bobot bukan angka bulat: {body:?} di {text}"
        );
        body.parse().expect("bobot harus bisa diparse")
    }

    #[test]
    fn output_sama_dengan_js() {
        // Template JS (`cekberat.js:34-40`) — termasuk penanda bold yang
        // harus tertutup dan baris kosong berisi empat spasi.
        for _ in 0..500 {
            let (text, mentions) = split(&run(ctx_with(Some(OTHER))));
            let kg = kg_of(&text);
            let desc = berat_desc(kg);
            let expect = format!(
                "Kamu ingin ngecek berat badan @{number} yak? \n    \nBerat badan dia sebesar *{kg} kg*\n```{desc}```",
                number = "628999"
            );
            assert_eq!(text, expect);
            assert_eq!(mentions, vec![OTHER.to_string()]);
        }
    }

    #[test]
    fn output_sama_dengan_js_untuk_diri_sendiri() {
        for _ in 0..500 {
            let (text, mentions) = split(&run(ctx_with(None)));
            let kg = kg_of(&text);
            let desc = berat_desc(kg);
            let expect = format!(
                "Hai @{number}\n    \nBerat badan kamu *{kg} kg*\n```{desc}```",
                number = "628111"
            );
            assert_eq!(text, expect);
            assert_eq!(mentions, vec![ME.to_string()]);
        }
    }

    #[test]
    fn tier_selalu_cocok_dengan_bobotnya() {
        for _ in 0..2000 {
            let (text, _) = split(&run(ctx_with(Some(OTHER))));
            let kg = kg_of(&text);
            assert!((40..=99).contains(&kg), "bobot di luar rentang JS: {kg}");
            let hits: Vec<usize> = TIERS
                .iter()
                .enumerate()
                .filter(|(_, t)| text.contains(**t))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(hits.len(), 1, "harus tepat satu tier, dapat {hits:?}: {text}");
            assert_eq!(TIERS[hits[0]], berat_desc(kg), "tier beda dari ambang: {text}");
        }
    }

    #[test]
    fn plugin_config_disalin_apa_adanya() {
        let p = find("cekberat").unwrap();
        assert_eq!(p.meta.name, "cekberat");
        assert_eq!(p.meta.alias, ["berat", "weight"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek berat badan random");
        assert_eq!(p.meta.usage, ".cekberat <nama>");
        assert_eq!(p.meta.example, ".cekberat Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }
}
