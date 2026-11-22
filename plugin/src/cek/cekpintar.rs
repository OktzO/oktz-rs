//! `plugins/cek/cekpintar.js` — cek seberapa pintar kamu.
//!
//! IQ = `floor(rand * 100) + 70` (`cekpintar.js:20`) sehingga 70..=169,
//! ambang tier `150/130/110/90` (`cekpintar.js:23-33`), kalimat "diri
//! sendiri" vs "orang lain" di `cekpintar.js:35-40`.
//!
//! Perhatikan: penanda bold di JS ditutup — `*${iq}*`.

use crate::common::{num_and_self, roll};
use oktz_plugin_api::{Ctx, Reply, plugin};

/// `floor(rand * 100) + 70` -> 70..=169, yaitu rentang Rust `70..170`.
///
/// Offset +70 inilah yang membuat kelima tier terjangkau: tanpa itu, range
/// 0..=99 membuat tier `>= 150` dan `>= 130` jadi dead branch permanen.
pub(crate) fn pintar_iq() -> u32 {
    roll(100) + 70
}

/// `cekpintar.js:23-33`.
pub(crate) fn pintar_desc(iq: u32) -> &'static str {
    if iq >= 150 {
        "JENIUS! Einstein level! 🧠✨"
    } else if iq >= 130 {
        "Sangat cerdas! 🎓"
    } else if iq >= 110 {
        "Di atas rata-rata! 👍"
    } else if iq >= 90 {
        "Normal, rata-rata 😊"
    } else {
        "Tetap semangat belajar! 📚"
    }
}

pub(crate) fn pintar_text(ctx: &Ctx, iq: u32) -> String {
    let (number, is_self) = num_and_self(ctx);
    let desc = pintar_desc(iq);
    if is_self {
        format!("Hai @{number}\n    \nKepintaran (IQ) kamu *{iq}*\n```{desc}```")
    } else {
        format!(
            "Kamu ingin ngecek kepintaran @{number} yak? \n    \nKepintaran (IQ) dia sebesar *{iq}*\n```{desc}```"
        )
    }
}

#[plugin(
    name = "cekpintar",
    alias = ["pintar", "iq", "smart"],
    category = "cek",
    description = "Cek seberapa pintar kamu",
    usage = ".cekpintar <nama>",
    example = ".cekpintar Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekpintar(ctx: Ctx) -> Reply {
    Reply::Text {
        text: pintar_text(&ctx, pintar_iq()),
        mentions: vec![ctx.mention_or_sender().to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    /// Tier `cekpintar.js:23-33` — urutan dari ambang tertinggi.
    const TIERS: [&str; 5] = [
        "JENIUS! Einstein level! 🧠✨",
        "Sangat cerdas! 🎓",
        "Di atas rata-rata! 👍",
        "Normal, rata-rata 😊",
        "Tetap semangat belajar! 📚",
    ];

    const ME: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn run(c: Ctx) -> Reply {
        (find("cekpintar").unwrap().handler)(c)
    }

    fn ctx_with(mention: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cekpintar".into(),
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

    /// Ambil angka IQ di `*<angka>*` — satu-satunya angka di balasan.
    fn iq_of(text: &str) -> u32 {
        let body = text
            .split_once('*')
            .and_then(|(_, rest)| rest.split_once('*'))
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| panic!("balasan tanpa IQ: {text}"));
        assert!(
            body.chars().all(|c| c.is_ascii_digit()),
            "IQ bukan angka bulat: {body:?} di {text}"
        );
        body.parse().expect("IQ harus bisa diparse")
    }

    #[test]
    fn output_sama_dengan_js() {
        // Template JS (`cekpintar.js:35-40`) — penanda bold harus ditutup.
        for _ in 0..500 {
            let (text, mentions) = split(&run(ctx_with(Some(OTHER))));
            let iq = iq_of(&text);
            let desc = pintar_desc(iq);
            let expect = format!(
                "Kamu ingin ngecek kepintaran @{number} yak? \n    \nKepintaran (IQ) dia sebesar *{iq}*\n```{desc}```",
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
            let iq = iq_of(&text);
            let desc = pintar_desc(iq);
            let expect = format!(
                "Hai @{number}\n    \nKepintaran (IQ) kamu *{iq}*\n```{desc}```",
                number = "628111"
            );
            assert_eq!(text, expect);
            assert_eq!(mentions, vec![ME.to_string()]);
        }
    }

    #[test]
    fn tier_selalu_cocok_dengan_iqnya() {
        for _ in 0..2000 {
            let (text, _) = split(&run(ctx_with(Some(OTHER))));
            let iq = iq_of(&text);
            assert!((70..=169).contains(&iq), "IQ di luar rentang JS: {iq}");
            let hits: Vec<usize> = TIERS
                .iter()
                .enumerate()
                .filter(|(_, t)| text.contains(**t))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(hits.len(), 1, "harus tepat satu tier, dapat {hits:?}: {text}");
            assert_eq!(TIERS[hits[0]], pintar_desc(iq), "tier beda dari ambang: {text}");
        }
    }

    #[test]
    fn plugin_config_disalin_apa_adanya() {
        let p = find("cekpintar").unwrap();
        assert_eq!(p.meta.name, "cekpintar");
        assert_eq!(p.meta.alias, ["pintar", "iq", "smart"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek seberapa pintar kamu");
        assert_eq!(p.meta.usage, ".cekpintar <nama>");
        assert_eq!(p.meta.example, ".cekpintar Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }
}
