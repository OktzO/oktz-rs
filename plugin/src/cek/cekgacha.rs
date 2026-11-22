//! `plugins/cek/cekgacha.js` — Cek hoki gacha kamu.
//!
//! Balasan: kegachaan acak `floor(rand * 101)` + lima tier deskripsi (ambang
//! `>= 90/70/50/30`), lalu dua kalimat "diri sendiri" vs "orang lain".

use crate::common::{cek_reply, percent_101};
use oktz_plugin_api::{Ctx, Reply, plugin};

#[plugin(
    name = "cekgacha",
    alias = ["luck"],
    category = "cek",
    description = "Cek hoki gacha kamu",
    usage = ".cekgacha <nama>",
    example = ".cekgacha Budi",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekgacha(ctx: Ctx) -> Reply {
    cek_reply(
        &ctx,
        percent_101(),
        "kegachaan",
        [
            "HOKI PARAH! SSR GUARANTEED! ✨💎",
            "Lucky! Pasti dapet SR keatas! 🍀",
            "Hoki-hoki dikit 😊",
            "Hmm... pray harder! 🙏",
            "SIAL! Nanti aja gachanya! 💔",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    /// Tier cekgacha.js — urutan dari ambang tertinggi.
    const TIERS: [&str; 5] = [
        "HOKI PARAH! SSR GUARANTEED! ✨💎",
        "Lucky! Pasti dapet SR keatas! 🍀",
        "Hoki-hoki dikit 😊",
        "Hmm... pray harder! 🙏",
        "SIAL! Nanti aja gachanya! 💔",
    ];

    /// Rentang inklusif tiap tier, dibaca dari ambang `percent >= N` di JS.
    const BANDS: [(u32, u32); 5] = [(90, 100), (70, 89), (50, 69), (30, 49), (0, 29)];

    const ME: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn run(c: Ctx) -> Reply {
        (find("cekgacha").unwrap().handler)(c)
    }

    fn ctx_with(mention: Option<&str>) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "cekgacha".into(),
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

    /// Ambil angka di `*<persen>%*` — satu-satunya angka persen di balasan.
    fn percent_of(text: &str) -> u32 {
        let body = text
            .split_once("*")
            .and_then(|(_, rest)| rest.split_once('%'))
            .map(|(head, _)| head.to_string())
            .unwrap_or_else(|| panic!("balasan tanpa persen: {text}"));
        assert!(
            body.chars().all(|c| c.is_ascii_digit()),
            "persen bukan angka bulat: {body:?} di {text}"
        );
        body.parse().expect("persen harus bisa diparse")
    }

    #[test]
    fn plugin_config_disalin_apa_adanya() {
        let p = find("cekgacha").unwrap();
        assert_eq!(p.meta.name, "cekgacha");
        assert_eq!(p.meta.alias, ["luck"]);
        assert_eq!(p.meta.category, "cek");
        assert_eq!(p.meta.description, "Cek hoki gacha kamu");
        assert_eq!(p.meta.usage, ".cekgacha <nama>");
        assert_eq!(p.meta.example, ".cekgacha Budi");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn setiap_alias_resolve_kembali_ke_plugin_ini() {
        for key in std::iter::once("cekgacha").chain(["luck"]) {
            assert_eq!(
                find(key).map(|p| p.meta.name.as_str()),
                Some("cekgacha"),
                "key {key} salah resolve"
            );
        }
    }

    #[test]
    fn tanpa_mention_target_adalah_sender_dan_kalimatnya_kamu() {
        let (text, mentions) = split(&run(ctx_with(None)));
        assert!(text.starts_with("Hai @628111\n"), "{text}");
        assert!(
            text.contains("Tingkat kegachaan kamu *"),
            "{text}"
        );
        assert!(!text.contains("dia sebesar"), "self pakai 'kamu': {text}");
        assert_eq!(mentions, vec![ME.to_string()]);
    }

    #[test]
    fn mention_lain_mengambil_kalimat_dia_dan_mention_jid_penuh() {
        let (text, mentions) = split(&run(ctx_with(Some(OTHER))));
        assert!(
            text.starts_with("Kamu ingin ngecek tingkat kegachaan @628999 yak? "),
            "{text}"
        );
        assert!(
            text.contains("Tingkat kegachaan dia sebesar *"),
            "{text}"
        );
        assert_eq!(mentions, vec![OTHER.to_string()]);
    }

    #[test]
    fn device_ikut_teks_tapi_mention_penuh() {
        let jid = "628999:3@s.whatsapp.net";
        let (text, mentions) = split(&run(ctx_with(Some(jid))));
        // JS `mentioned.split('@')[0]` membuang server saja; `:device` tetap
        // tampil (lihat `jid_number_strips_the_server_but_keeps_the_device`).
        assert!(text.contains("@628999:3 yak?"), "display: {text}");
        assert_eq!(mentions, vec![jid.to_string()]);
    }

    #[test]
    fn deskripsi_selalu_lima_tier_plugin_ini_dan_satu_banded_saja() {
        for _ in 0..2000 {
            let (text, _) = split(&run(ctx_with(Some(OTHER))));
            let hits: Vec<usize> = TIERS
                .iter()
                .enumerate()
                .filter(|(_, t)| text.contains(**t))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(hits.len(), 1, "harus tepat satu tier, dapat {hits:?}: {text}");
            let (lo, hi) = BANDS[hits[0]];
            let percent = percent_of(&text);
            assert!(
                (lo..=hi).contains(&percent),
                "tier {:?} dapat percent {percent}, harusnya {lo}..={hi}: {text}",
                TIERS[hits[0]]
            );
        }
    }

    #[test]
    fn mention_yang_sama_dengan_sender_mengambil_cabang_kamu() {
        // JS: `mentioned === m.sender`, jadi mention milik diri sendiri bukan
        // "orang lain" meski `mentionedJid` terisi.
        let (text, mentions) = split(&run(ctx_with(Some(ME))));
        assert!(text.contains("Tingkat"), "{text}");
        assert!(!text.contains("yak?"), "harus cabang self: {text}");
        assert_eq!(mentions, vec![ME.to_string()]);
    }

    #[test]
    fn plugin_hanya_membaca_tanpa_mutasi() {
        assert!(run(ctx_with(None)).mutations().is_empty());
        assert!(run(ctx_with(Some(OTHER))).mutations().is_empty());
    }

    /// Kunci output: template JS persis (`cekgacha.js:34-40`).
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
            command: "cekgacha".into(),
            sender: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        let lain = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            ..diri.clone()
        };
        for percent in [0u32, 29, 30, 49, 50, 69, 70, 89, 90, 100] {
            let desc = tier_js(percent);
            let Reply::Text { text, mentions } = cek_reply(&diri, percent, "kegachaan", TIERS) else {
                panic!("harus Reply::Text");
            };
            assert_eq!(
                text,
                format!(
                    "Hai @{number}\n    \nTingkat kegachaan kamu *{percent}%*\n```{desc}```",
                    number = "628111",
                    desc = desc
                ),
                "percent {percent}"
            );
            assert_eq!(mentions, vec!["628111@s.whatsapp.net".to_string()]);

            let Reply::Text { text, mentions } = cek_reply(&lain, percent, "kegachaan", TIERS) else {
                panic!("harus Reply::Text");
            };
            assert_eq!(
                text,
                format!(
                    "Kamu ingin ngecek tingkat kegachaan @{number} yak? \n    \nTingkat kegachaan dia sebesar *{percent}%*\n```{desc}```",
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
            command: "cekgacha".into(),
            sender: "628111@s.whatsapp.net".into(),
            ..Default::default()
        };
        let lain = Ctx {
            mentioned: vec!["628999@s.whatsapp.net".into()],
            ..diri.clone()
        };
        for _ in 0..200 {
            let Reply::Text { text, .. } = cekgacha(diri.clone()) else {
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
                    "Hai @{number}\n    \nTingkat kegachaan kamu *{percent}%*\n```{desc}```",
                    number = "628111",
                    desc = desc
                ),
                "{text}"
            );

            let Reply::Text { text, .. } = cekgacha(lain.clone()) else {
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
                    "Kamu ingin ngecek tingkat kegachaan @{number} yak? \n    \nTingkat kegachaan dia sebesar *{percent}%*\n```{desc}```",
                    number = "628999",
                    desc = desc
                ),
                "{text}"
            );
        }
    }

}
