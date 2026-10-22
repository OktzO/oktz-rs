//! `plugins/cek/cekkaya.js` — Cek tingkat kekayaan 0-100%, lima ambang `>= 90/70/50/30`.

use crate::common::{cek_reply, percent_101};
use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "cekkaya",
    alias = ["kaya", "rich"],
    category = "cek",
    description = "Cek seberapa kaya kamu",
    usage = ".cekkaya <nama>",
    example = ".cekkaya Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekkaya(ctx: Ctx) -> Reply {
    cek_reply(
        &ctx,
        percent_101(),
        "kekayaan",
        [
            "Sultan! Crazy rich! 💎",
            "Tajir melintir! 💰",
            "Lumayan berada 💵",
            "Cukup lah buat hidup 😊",
            "Semangat nabung! 🙏",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIERS: [&str; 5] = ["Sultan! Crazy rich! 💎", "Tajir melintir! 💰", "Lumayan berada 💵", "Cukup lah buat hidup 😊", "Semangat nabung! 🙏"];
    const LABEL: &str = "kekayaan";
    const SENDER: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    /// Undian sebanyak ini hampir pasti menyentuh kelima tier: pita
    /// teratas hanya 11/101, jadi peluang gagal satu tier ~1e-10.
    const SAMPLES: usize = 2_000;

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("cekkaya").unwrap().handler)(ctx)
    }

    fn base() -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cekkaya".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            ..Default::default()
        }
    }

    fn ctx_with_mention() -> Ctx {
        Ctx {
            mentioned: vec![OTHER.to_string()],
            ..base()
        }
    }

    fn text_of(r: &Reply) -> (String, Vec<String>) {
        match r {
            Reply::Text { text, mentions } => (text.clone(), mentions.clone()),
            other => panic!("harus Text, dapat {other:?}"),
        }
    }

    /// Baca ulang angka di `*NN%*` supaya batas `floor(random * 101)`
    /// diuji dari balasan plugin yang sungguhan, bukan dari tebakan.
    fn percent_in(text: &str) -> u32 {
        let rest = text.split_once('*').expect("harus ada *percent*").1;
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().expect("percent harus angka")
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("cekkaya").unwrap();
        assert_eq!(p.meta.name, "cekkaya");
        assert_eq!(p.meta.alias, ["kaya", "rich"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek seberapa kaya kamu");
        assert_eq!(p.meta.usage, ".cekkaya <nama>");
        assert_eq!(p.meta.example, ".cekkaya Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(p.meta.is_enabled);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for key in ["cekkaya","kaya", "rich"] {
            assert_eq!(
                oktz_plugin_api::find(key).map(|p| p.meta.name.as_str()),
                Some("cekkaya"),
                "key {key} salah resolve"
            );
        }
    }

    #[test]
    fn without_mention_the_reply_targets_the_sender() {
        let (text, mentions) = text_of(&run(base()));
        assert_eq!(mentions, [SENDER.to_string()]);
        assert!(text.starts_with("Hai @628111"), "{text}");
        assert!(
            text.contains(&format!("Tingkat {LABEL} kamu *{}%", percent_in(&text))),
            "{text}"
        );
        assert!(!text.contains("dia sebesar"), "self tidak pakai 'dia': {text}");
    }

    #[test]
    fn with_mention_the_reply_targets_the_mention_and_says_dia() {
        let (text, mentions) = text_of(&run(ctx_with_mention()));
        assert_eq!(mentions, [OTHER.to_string()]);
        assert!(
            text.contains(&format!("tingkat {LABEL} @628999 yak?")),
            "{text}"
        );
        assert!(
            text.contains(&format!("Tingkat {LABEL} dia sebesar *{}%", percent_in(&text))),
            "{text}"
        );
    }

    #[test]
    fn every_percent_value_picks_exactly_one_tier() {
        // `Math.floor(Math.random() * 101)` = 0..=100 inklusif; semua nilai
        // ditutup supaya ambang yang bergeser langsung ketahuan.
        let ctx = base();
        for percent in 0..=100u32 {
            let (text, _) = text_of(&cek_reply(&ctx, percent, LABEL, TIERS));
            assert!(text.contains(&format!("*{percent}%*")), "percent hilang: {text}");
            let hits: Vec<&&str> = TIERS.iter().filter(|t| text.contains(**t)).collect();
            assert_eq!(hits.len(), 1, "percent {percent} memuat {} tier: {text}", hits.len());
        }
    }

    #[test]
    fn tier_boundaries_are_inclusive_at_90_70_50_30() {
        let ctx = base();
        for (percent, expected) in [
            (100u32, TIERS[0]),
            (90, TIERS[0]),
            (89, TIERS[1]),
            (70, TIERS[1]),
            (69, TIERS[2]),
            (50, TIERS[2]),
            (49, TIERS[3]),
            (30, TIERS[3]),
            (29, TIERS[4]),
            (0, TIERS[4]),
        ] {
            let (text, _) = text_of(&cek_reply(&ctx, percent, LABEL, TIERS));
            assert!(
                text.contains(expected),
                "percent {percent} harusnya {expected:?}: {text}"
            );
        }
    }

    #[test]
    fn handler_stays_in_range_and_reaches_every_tier() {
        let ctx = base();
        let mut seen: Vec<&str> = Vec::new();
        for _ in 0..SAMPLES {
            let (text, mentions) = text_of(&run(ctx.clone()));
            assert_eq!(mentions, [SENDER.to_string()]);
            assert!(text.contains(&format!("Tingkat {LABEL} kamu")), "{text}");
            let percent = percent_in(&text);
            assert!(percent <= 100, "percent di luar 0..=100: {percent}");
            let hits: Vec<&&str> = TIERS.iter().filter(|t| text.contains(**t)).collect();
            assert_eq!(hits.len(), 1, "balasan memuat {} tier: {text}", hits.len());
            if !seen.contains(hits[0]) {
                seen.push(hits[0]);
            }
        }
        assert_eq!(seen.len(), 5, "tier yang tidak terjangkau: {seen:?}");
    }

    /// Kunci output: template JS persis (`cekkaya.js:34-40`).
    ///
    /// Persennya dikunci (bukan acak) supaya teks yang dibandingkan
    /// deterministik; jalur handler dicek terpisah di bawah.
    #[test]
    fn output_sama_dengan_js() {
        let tier_js = |percent: u32| -> &'static str {
            match percent {
                90..=100 => TIERS[0],
                70..=89 => TIERS[1],
                50..=69 => TIERS[2],
                30..=49 => TIERS[3],
                _ => TIERS[4],
            }
        };
        let diri = Ctx {
            prefix: ".".into(),
            command: "cekkaya".into(),
            sender: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        let lain = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            ..diri.clone()
        };
        for percent in [0u32, 29, 30, 49, 50, 69, 70, 89, 90, 100] {
            let desc = tier_js(percent);
            let Reply::Text { text, mentions } = cek_reply(&diri, percent, "kekayaan", TIERS) else {
                panic!("harus Reply::Text");
            };
            assert_eq!(
                text,
                format!(
                    "Hai @{number}\n    \nTingkat kekayaan kamu *{percent}%*\n```{desc}```",
                    number = "628111",
                    desc = desc
                ),
                "percent {percent}"
            );
            assert_eq!(mentions, vec!["628111@s.whatsapp.net".to_string()]);

            let Reply::Text { text, mentions } = cek_reply(&lain, percent, "kekayaan", TIERS) else {
                panic!("harus Reply::Text");
            };
            assert_eq!(
                text,
                format!(
                    "Kamu ingin ngecek tingkat kekayaan @{number} yak? \n    \nTingkat kekayaan dia sebesar *{percent}%*\n```{desc}```",
                    number = "628999",
                    desc = desc
                ),
                "percent {percent}"
            );
            assert_eq!(mentions, vec!["628999@s.whatsapp.net".to_string()]);
        }
    }

    /// Jalur handler: persennya acak, tapi kerangka teksnya tetap sama.
    #[test]
    fn output_sama_dengan_js_di_jalur_handler() {
        let diri = Ctx {
            prefix: ".".into(),
            command: "cekkaya".into(),
            sender: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        let lain = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            ..diri.clone()
        };
        for _ in 0..200 {
            let Reply::Text { text, .. } = cekkaya(diri.clone()) else {
                panic!("harus Reply::Text");
            };
            let percent: u32 = text
                .split_once('*')
                .and_then(|(_, rest)| rest.split_once('%'))
                .and_then(|(head, _)| head.parse().ok())
                .expect("persen di balasan");
            assert!(
                (0..=100).contains(&percent),
                "persen di luar rentang JS: {percent}"
            );
            let desc = match percent {
                90..=100 => TIERS[0],
                70..=89 => TIERS[1],
                50..=69 => TIERS[2],
                30..=49 => TIERS[3],
                _ => TIERS[4],
            };
            assert_eq!(
                text,
                format!(
                    "Hai @{number}\n    \nTingkat kekayaan kamu *{percent}%*\n```{desc}```",
                    number = "628111",
                    desc = desc
                ),
                "{text}"
            );

            let Reply::Text { text, .. } = cekkaya(lain.clone()) else {
                panic!("harus Reply::Text");
            };
            let percent: u32 = text
                .split_once('*')
                .and_then(|(_, rest)| rest.split_once('%'))
                .and_then(|(head, _)| head.parse().ok())
                .expect("persen di balasan");
            let desc = match percent {
                90..=100 => TIERS[0],
                70..=89 => TIERS[1],
                50..=69 => TIERS[2],
                30..=49 => TIERS[3],
                _ => TIERS[4],
            };
            assert_eq!(
                text,
                format!(
                    "Kamu ingin ngecek tingkat kekayaan @{number} yak? \n    \nTingkat kekayaan dia sebesar *{percent}%*\n```{desc}```",
                    number = "628999",
                    desc = desc
                ),
                "{text}"
            );
        }
    }

}
