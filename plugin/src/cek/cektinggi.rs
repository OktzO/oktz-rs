//! `plugins/cek/cektinggi.js` — cek tinggi badan random.
//!
//! Tinggi = `floor(rand * 50) + 150` (`cektinggi.js:21`) sehingga 150..=199,
//! ambang tier `190/175/165/155` (`cektinggi.js:23-33`), kalimat "diri
//! sendiri" vs "orang lain" di `cektinggi.js:35-40`.
//!
//! Perhatikan: penanda bold di JS ditutup — `*${tinggi} cm*`.

use crate::common::{num_and_self, roll};
use oktz_plugin_api::{Ctx, Reply, plugin};

/// `floor(rand * 50) + 150` -> 150..=199, yaitu rentang Rust `150..200`.
pub(crate) fn tinggi_cm() -> u32 {
    roll(50) + 150
}

/// `cektinggi.js:23-33`.
pub(crate) fn tinggi_desc(cm: u32) -> &'static str {
    if cm >= 190 {
        "TINGGI BANGET! Model basketball! 🏀"
    } else if cm >= 175 {
        "Tinggi ideal! 😎"
    } else if cm >= 165 {
        "Lumayan tinggi 👍"
    } else if cm >= 155 {
        "Standard kok 🙂"
    } else {
        "Imut dan mungil! 🥺"
    }
}

/// Kalimat "orang lain" di JS memang berbeda dari kalimat self: `tingkat
/// ketinggian` vs `Tinggi badan`. Ketidakkonsistenan itu dibiarkan — itu isi
/// file aslinya, bukan bug yang boleh dirapikan diam-diam.
pub(crate) fn tinggi_text(ctx: &Ctx, cm: u32) -> String {
    let (number, is_self) = num_and_self(ctx);
    let desc = tinggi_desc(cm);
    if is_self {
        format!("Hai @{number}\n    \nTinggi badan kamu *{cm} cm*\n```{desc}```")
    } else {
        format!(
            "Kamu ingin ngecek tingkat ketinggian @{number} yak? \n    \nTinggi badan dia sebesar *{cm} cm*\n```{desc}```"
        )
    }
}

#[plugin(
    name = "cektinggi",
    alias = ["tinggi", "tall"],
    category = "cek",
    description = "Cek tinggi badan random",
    usage = ".cektinggi <nama>",
    example = ".cektinggi Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cektinggi(ctx: Ctx) -> Reply {
    Reply::Text {
        text: tinggi_text(&ctx, tinggi_cm()),
        mentions: vec![ctx.mention_or_sender().to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    /// Tier `cektinggi.js:23-33` — urutan dari ambang tertinggi.
    const TIERS: [&str; 5] = [
        "TINGGI BANGET! Model basketball! 🏀",
        "Tinggi ideal! 😎",
        "Lumayan tinggi 👍",
        "Standard kok 🙂",
        "Imut dan mungil! 🥺",
    ];

    const ME: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn run(c: Ctx) -> Reply {
        (find("cektinggi").unwrap().handler)(c)
    }

    fn ctx_with(mention: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cektinggi".into(),
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

    /// Ambil angka cm di `*<angka> cm*` — satu-satunya angka di balasan.
    fn cm_of(text: &str) -> u32 {
        let body = text
            .split_once('*')
            .and_then(|(_, rest)| rest.split_once(" cm*"))
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| panic!("balasan tanpa tinggi: {text}"));
        assert!(
            body.chars().all(|c| c.is_ascii_digit()),
            "tinggi bukan angka bulat: {body:?} di {text}"
        );
        body.parse().expect("tinggi harus bisa diparse")
    }

    #[test]
    fn output_sama_dengan_js() {
        // Template JS (`cektinggi.js:35-40`) — penanda bold harus ditutup.
        for _ in 0..500 {
            let (text, mentions) = split(&run(ctx_with(Some(OTHER))));
            let cm = cm_of(&text);
            let desc = tinggi_desc(cm);
            let expect = format!(
                "Kamu ingin ngecek tingkat ketinggian @{number} yak? \n    \nTinggi badan dia sebesar *{cm} cm*\n```{desc}```",
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
            let cm = cm_of(&text);
            let desc = tinggi_desc(cm);
            let expect = format!(
                "Hai @{number}\n    \nTinggi badan kamu *{cm} cm*\n```{desc}```",
                number = "628111"
            );
            assert_eq!(text, expect);
            assert_eq!(mentions, vec![ME.to_string()]);
        }
    }

    #[test]
    fn tier_selalu_cocok_dengan_tingginya() {
        for _ in 0..2000 {
            let (text, _) = split(&run(ctx_with(Some(OTHER))));
            let cm = cm_of(&text);
            assert!((150..=199).contains(&cm), "tinggi di luar rentang JS: {cm}");
            let hits: Vec<usize> = TIERS
                .iter()
                .enumerate()
                .filter(|(_, t)| text.contains(**t))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(hits.len(), 1, "harus tepat satu tier, dapat {hits:?}: {text}");
            assert_eq!(TIERS[hits[0]], tinggi_desc(cm), "tier beda dari ambang: {text}");
        }
    }

    #[test]
    fn plugin_config_disalin_apa_adanya() {
        let p = find("cektinggi").unwrap();
        assert_eq!(p.meta.name, "cektinggi");
        assert_eq!(p.meta.alias, ["tinggi", "tall"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek tinggi badan random");
        assert_eq!(p.meta.usage, ".cektinggi <nama>");
        assert_eq!(p.meta.example, ".cektinggi Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }
}
