//! `plugins/cek/ceksisaumur.js` — cek sisa umur kamu.
//!
//! Tiga undian terpisah (`ceksisaumur.js:22-24`): `tahun` 20..=99, `bulan`
//! 0..=11, `hari` 0..=29. Tier memakai `>` (bukan `>=`) pada `tahun`
//! (`ceksisaumur.js:27-34`).
//!
//! Perhatikan: penanda bold di JS ditutup —
//! `*${tahun} Tahun ${bulan} Bulan ${hari} Hari*`.

use crate::common::{num_and_self, roll};
use oktz_plugin_api::{Ctx, Reply, plugin};

/// Tiga undian terpisah — `ceksisaumur.js:22-24`. Semuanya 0..=n-1; hanya
/// tahun yang diberi offset.
pub(crate) fn sisaumur_tahun() -> u32 {
    roll(80) + 20
}

pub(crate) fn sisaumur_bulan() -> u32 {
    roll(12)
}

pub(crate) fn sisaumur_hari() -> u32 {
    roll(30)
}

/// SATU-SATUNYA plugin di batch ini yang memakai `>` bukan `>=`
/// (`ceksisaumur.js:27-34`).
///
/// Konsekuensi: `tahun == 80` -> tier kedua, `tahun == 60` -> tier ketiga,
/// `tahun == 40` -> tier keempat. Kalau dibikin `>=`, tiga nilai itu lompat satu
/// tingkat — dan bukan probabilitas kecil (masing-masing 1 dari 80 nilai).
pub(crate) fn sisaumur_desc(tahun: u32) -> &'static str {
    if tahun > 80 {
        "Panjang umur banget! 🎉"
    } else if tahun > 60 {
        "Lumayan panjang~ ✨"
    } else if tahun > 40 {
        "Cukup lah ya 😊"
    } else {
        "Jaga kesehatan ya! 🙏"
    }
}

pub(crate) fn sisaumur_text(ctx: &Ctx, tahun: u32, bulan: u32, hari: u32) -> String {
    let (number, is_self) = num_and_self(ctx);
    let desc = sisaumur_desc(tahun);
    if is_self {
        format!(
            "Hai @{number}\n    \nSisa umur kamu *{tahun} Tahun {bulan} Bulan {hari} Hari*\n```{desc}```"
        )
    } else {
        format!(
            "Kamu ingin ngecek tingkat kesisaumuran @{number} yak? \n    \nSisa umur dia sebesar *{tahun} Tahun {bulan} Bulan {hari} Hari*\n```{desc}```"
        )
    }
}

#[plugin(
    name = "ceksisaumur",
    alias = ["sisaumur"],
    category = "cek",
    description = "Cek sisa umur kamu",
    usage = ".ceksisaumur <nama>",
    example = ".ceksisaumur Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn ceksisaumur(ctx: Ctx) -> Reply {
    Reply::Text {
        text: sisaumur_text(
            &ctx,
            sisaumur_tahun(),
            sisaumur_bulan(),
            sisaumur_hari(),
        ),
        mentions: vec![ctx.mention_or_sender().to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    /// Tier `ceksisaumur.js:27-34` — urutan dari ambang tertinggi.
    const TIERS: [&str; 4] = [
        "Panjang umur banget! 🎉",
        "Lumayan panjang~ ✨",
        "Cukup lah ya 😊",
        "Jaga kesehatan ya! 🙏",
    ];

    const ME: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn run(c: Ctx) -> Reply {
        (find("ceksisaumur").unwrap().handler)(c)
    }

    fn ctx_with(mention: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "ceksisaumur".into(),
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

    /// Ambil tiga angka di `*<tahun> Tahun <bulan> Bulan <hari> Hari*`.
    fn durasi_of(text: &str) -> (u32, u32, u32) {
        let body = text
            .split_once('*')
            .and_then(|(_, rest)| rest.split_once('*'))
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| panic!("balasan tanpa durasi: {text}"));
        let angka: Vec<u32> = body
            .split_whitespace()
            .step_by(2)
            .map(|s| {
                assert!(
                    s.chars().all(|c| c.is_ascii_digit()),
                    "angka durasi bukan angka bulat: {s:?} di {text}"
                );
                s.parse().expect("angka durasi harus bisa diparse")
            })
            .collect();
        assert_eq!(angka.len(), 3, "harus ada tahun/bulan/hari: {text}");
        (angka[0], angka[1], angka[2])
    }

    #[test]
    fn output_sama_dengan_js() {
        // Template JS (`ceksisaumur.js:36-41`) — penanda bold harus ditutup.
        for _ in 0..500 {
            let (text, mentions) = split(&run(ctx_with(Some(OTHER))));
            let (tahun, bulan, hari) = durasi_of(&text);
            let desc = sisaumur_desc(tahun);
            let expect = format!(
                "Kamu ingin ngecek tingkat kesisaumuran @{number} yak? \n    \nSisa umur dia sebesar *{tahun} Tahun {bulan} Bulan {hari} Hari*\n```{desc}```",
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
            let (tahun, bulan, hari) = durasi_of(&text);
            let desc = sisaumur_desc(tahun);
            let expect = format!(
                "Hai @{number}\n    \nSisa umur kamu *{tahun} Tahun {bulan} Bulan {hari} Hari*\n```{desc}```",
                number = "628111"
            );
            assert_eq!(text, expect);
            assert_eq!(mentions, vec![ME.to_string()]);
        }
    }

    #[test]
    fn rentang_undian_sama_dengan_js() {
        for _ in 0..2000 {
            let (text, _) = split(&run(ctx_with(Some(OTHER))));
            let (tahun, bulan, hari) = durasi_of(&text);
            assert!((20..=99).contains(&tahun), "tahun di luar rentang JS: {tahun}");
            assert!((0..=11).contains(&bulan), "bulan di luar rentang JS: {bulan}");
            assert!((0..=29).contains(&hari), "hari di luar rentang JS: {hari}");
            let hits: Vec<usize> = TIERS
                .iter()
                .enumerate()
                .filter(|(_, t)| text.contains(**t))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(hits.len(), 1, "harus tepat satu tier, dapat {hits:?}: {text}");
            assert_eq!(TIERS[hits[0]], sisaumur_desc(tahun), "tier beda dari ambang: {text}");
        }
    }

    #[test]
    fn ambang_pakai_tanda_lebih_besar_bukan_lebih_besar_sama() {
        // `tahun > 80` -> tier pertama, jadi 80 masih tier kedua (JS:27-34).
        assert_eq!(sisaumur_desc(99), "Panjang umur banget! 🎉");
        assert_eq!(sisaumur_desc(81), "Panjang umur banget! 🎉");
        assert_eq!(sisaumur_desc(80), "Lumayan panjang~ ✨");
        assert_eq!(sisaumur_desc(61), "Lumayan panjang~ ✨");
        assert_eq!(sisaumur_desc(60), "Cukup lah ya 😊");
        assert_eq!(sisaumur_desc(41), "Cukup lah ya 😊");
        assert_eq!(sisaumur_desc(40), "Jaga kesehatan ya! 🙏");
    }

    #[test]
    fn plugin_config_disalin_apa_adanya() {
        let p = find("ceksisaumur").unwrap();
        assert_eq!(p.meta.name, "ceksisaumur");
        assert_eq!(p.meta.alias, ["sisaumur"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek sisa umur kamu");
        assert_eq!(p.meta.usage, ".ceksisaumur <nama>");
        assert_eq!(p.meta.example, ".ceksisaumur Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }
}
