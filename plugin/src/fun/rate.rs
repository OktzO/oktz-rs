use crate::common::random_index;
use oktz_plugin_api::{Ctx, Reply, plugin};

pub(crate) const RATE: [(&str, &str); 20] = [
    ("10/10", "Sempurna! Nggak ada duanya!"),
    ("9/10", "Hampir sempurna! Keren banget!"),
    ("8/10", "Bagus banget! Mantap!"),
    ("7/10", "Cukup bagus, di atas rata-rata!"),
    ("6/10", "Lumayan, bisa lebih baik lagi."),
    ("5/10", "Biasa aja sih, standar."),
    ("4/10", "Hmm, kurang sedikit."),
    ("3/10", "Perlu banyak perbaikan."),
    ("2/10", "Aduh, masih jauh dari bagus."),
    ("1/10", "Maaf, tapi ini parah."),
    ("100/10", "LEGEND! Beyond perfect!"),
    ("11/10", "Melebihi ekspektasi!"),
    ("69/100", "Nice..."),
    ("420/10", "BLAZING!"),
    ("∞/10", "Gacor kang"),
    ("7.5/10", "Solid! Good job!"),
    ("8.5/10", "Impressive!"),
    ("9.5/10", "Near perfection!"),
    ("-1/10", "Aku nggak tau harus ngomong apa..."),
    ("???/10", "Error 404: Rating not found."),
];

pub(crate) const HELP_RATE: &str =
    "⭐ *ʀᴀᴛᴇ*\n\n> Masukkan sesuatu untuk dinilai!\n\n*Contoh:*\n> .rate wajahku";

pub(crate) fn render_rate(score: &str, comment: &str) -> String {
    format!("Rating dari aku: *{score}*\n{comment}")
}

#[plugin(
    name = "rate",
    alias = ["nilai", "rating"],
    category = "fun",
    description = "Minta bot memberi rating sesuatu",
    usage = ".rate <sesuatu>",
    example = ".rate wajahku",
    cooldown = 3,
    energi = 0,
    is_private = false
)]
pub fn rate(ctx: Ctx) -> Reply {
    if ctx.text.trim().is_empty() {
        return Reply::text(HELP_RATE);
    }
    let (score, comment) = RATE[random_index(RATE.len())];
    Reply::text(render_rate(score, comment))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const SENDER: &str = "628111@s.whatsapp.net";

    fn run(text: &str) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "rate".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            ..Default::default()
        };
        (find("rate").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("plugin harus membalas teks")
            .to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let m = &find("rate").unwrap().meta;
        assert_eq!(m.name, "rate");
        assert_eq!(m.alias, ["nilai", "rating"]);
        assert_eq!(m.category, "fun");
        assert_eq!(m.description, "Minta bot memberi rating sesuatu");
        assert_eq!(m.usage, ".rate <sesuatu>");
        assert_eq!(m.example, ".rate wajahku");
        assert_eq!(m.cooldown, 3);
        assert_eq!(m.energi, 0);
        assert!(!m.is_private && !m.is_group && !m.is_owner && !m.is_premium);
    }

    /// `rate.js:47-49` — teks bantuan memakai "Dinilai" (bukan "Dinilai").
    #[test]
    fn output_sama_dengan_js() {
        for kosong in ["", "   ", "\t"] {
            assert_eq!(
                text_of(&run(kosong)),
                "⭐ *ʀᴀᴛᴇ*\n\n> Masukkan sesuatu untuk dinilai!\n\n*Contoh:*\n> .rate wajahku",
                "input {kosong:?}"
            );
        }
    }

    /// `rate.js:60-62` — `Rating dari aku: *<score>*` lalu komentar.
    #[test]
    fn rating_dibatasi_ke_daftar_js() {
        for _ in 0..80 {
            let t = text_of(&run("wajahku"));
            let sisa = t.strip_prefix("Rating dari aku: *").expect("prefix rating");
            let (score, comment) = sisa.split_once("*\n").expect("pola <score>*\\n<komentar>");
            assert!(
                RATE.iter()
                    .any(|(sc, cm)| *sc == score && *cm == comment),
                "rating di luar tabel JS: {score} / {comment}"
            );
        }
    }
}
