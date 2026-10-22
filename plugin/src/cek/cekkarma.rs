//! `plugins/cek/cekkarma.js` — cek tingkat karma kamu.
//!
//! Persen = `floor(rand * 101)` (`cekkarma.js:18`), ambang tier
//! `80/60/40/20` (`cekkarma.js:22-26`), kalimat "diri sendiri" vs "orang
//! lain" di `cekkarma.js:28-34`.
//!
//! Perhatikan: penanda bold di JS ditutup — `*${percent}%*`.

use crate::common::{num_and_self, roll};
use oktz_plugin_api::{Ctx, Reply, plugin};

/// `floor(rand * 101)` -> 0..=100.
pub(crate) fn karma_percent() -> u32 {
    roll(101)
}

/// Ambang persis `cekkarma.js:22-26` — semua `>=`, semua kelipatan 20.
pub(crate) fn karma_desc(percent: u32) -> &'static str {
    if percent >= 80 {
        "Karma baik! Surga menantimu~ ✨"
    } else if percent >= 60 {
        "Cukup baik, terus tingkatkan! 🙏"
    } else if percent >= 40 {
        "Netral, perbanyak kebaikan~ ⚖️"
    } else if percent >= 20 {
        "Hati-hati dengan karma buruk! ⚠️"
    } else {
        "Wah perlu banyak tobat nih... 😱"
    }
}

pub(crate) fn karma_text(ctx: &Ctx, percent: u32) -> String {
    let (number, is_self) = num_and_self(ctx);
    let desc = karma_desc(percent);
    if is_self {
        format!("Hai @{number}\n    \nTingkat kekarmaan kamu *{percent}%*\n```{desc}```")
    } else {
        format!(
            "Kamu ingin ngecek tingkat kekarmaan @{number} yak? \n    \nTingkat kekarmaan dia sebesar *{percent}%*\n```{desc}```"
        )
    }
}

#[plugin(
    name = "cekkarma",
    alias = ["karma"],
    category = "cek",
    description = "Cek tingkat karma kamu",
    usage = ".cekkarma <nama>",
    example = ".cekkarma Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekkarma(ctx: Ctx) -> Reply {
    Reply::Text {
        text: karma_text(&ctx, karma_percent()),
        mentions: vec![ctx.mention_or_sender().to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    /// Tier `cekkarma.js:22-26` — urutan dari ambang tertinggi.
    const TIERS: [&str; 5] = [
        "Karma baik! Surga menantimu~ ✨",
        "Cukup baik, terus tingkatkan! 🙏",
        "Netral, perbanyak kebaikan~ ⚖️",
        "Hati-hati dengan karma buruk! ⚠️",
        "Wah perlu banyak tobat nih... 😱",
    ];

    const ME: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn run(c: Ctx) -> Reply {
        (find("cekkarma").unwrap().handler)(c)
    }

    fn ctx_with(mention: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cekkarma".into(),
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

    /// Ambil angka persen di `*<angka>%*` — satu-satunya angka di balasan.
    fn percent_of(text: &str) -> u32 {
        let body = text
            .split_once('*')
            .and_then(|(_, rest)| rest.split_once("%*"))
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| panic!("balasan tanpa persen: {text}"));
        assert!(
            body.chars().all(|c| c.is_ascii_digit()),
            "persen bukan angka bulat: {body:?} di {text}"
        );
        body.parse().expect("persen harus bisa diparse")
    }

    #[test]
    fn output_sama_dengan_js() {
        // Template JS (`cekkarma.js:28-34`) — penanda bold harus ditutup.
        for _ in 0..500 {
            let (text, mentions) = split(&run(ctx_with(Some(OTHER))));
            let percent = percent_of(&text);
            let desc = karma_desc(percent);
            let expect = format!(
                "Kamu ingin ngecek tingkat kekarmaan @{number} yak? \n    \nTingkat kekarmaan dia sebesar *{percent}%*\n```{desc}```",
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
            let percent = percent_of(&text);
            let desc = karma_desc(percent);
            let expect = format!(
                "Hai @{number}\n    \nTingkat kekarmaan kamu *{percent}%*\n```{desc}```",
                number = "628111"
            );
            assert_eq!(text, expect);
            assert_eq!(mentions, vec![ME.to_string()]);
        }
    }

    #[test]
    fn tier_selalu_cocok_dengan_persennya() {
        for _ in 0..2000 {
            let (text, _) = split(&run(ctx_with(Some(OTHER))));
            let percent = percent_of(&text);
            assert!((0..=100).contains(&percent), "persen di luar rentang JS: {percent}");
            let hits: Vec<usize> = TIERS
                .iter()
                .enumerate()
                .filter(|(_, t)| text.contains(**t))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(hits.len(), 1, "harus tepat satu tier, dapat {hits:?}: {text}");
            assert_eq!(TIERS[hits[0]], karma_desc(percent), "tier beda dari ambang: {text}");
        }
    }

    #[test]
    fn plugin_config_disalin_apa_adanya() {
        let p = find("cekkarma").unwrap();
        assert_eq!(p.meta.name, "cekkarma");
        assert_eq!(p.meta.alias, ["karma"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek tingkat karma kamu");
        assert_eq!(p.meta.usage, ".cekkarma <nama>");
        assert_eq!(p.meta.example, ".cekkarma Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }
}
