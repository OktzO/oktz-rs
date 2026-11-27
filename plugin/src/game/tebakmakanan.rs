//! Game quiz dari engine (`FotoGames`).

use crate::game::quiz::{QuizGame, answer, handler};
use oktz_plugin_api::{Ctx, Reply, answer_handler, plugin};

const CFG: QuizGame = QuizGame {
    name: "tebakmakanan",
    alias: &["makanan", "food"],
    emoji: "🍲",
    title: "TEBAK MAKANAN",
    description: "Tebak nama makanan",
    timeout_ms: 60000,
    data_file: "tebakmakanan.json",
    question_field: Some("soal"),
    answer_field: "jawaban",
    has_image: true,
    image_field: "img",
    hint_count: 2,
};

#[plugin(
    name = "tebakmakanan",
    alias = ["makanan", "food"],
    category = "game",
    description = "Tebak nama makanan",
    usage = ".tebakmakanan",
    example = ".tebakmakanan",
    cooldown = 5,
    energi = 0
)]
pub fn tebakmakanan(ctx: Ctx) -> Reply {
    handler(&CFG, ctx)
}

#[answer_handler]
pub fn tebakmakanan_answer(ctx: &Ctx) -> Option<Reply> {
    answer(&CFG, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn meta_dan_alias_terdaftar() {
        let p = find("tebakmakanan").expect("terdaftar");
        assert_eq!(p.meta.name, "tebakmakanan");
        for a in ["makanan", "food"] {
            assert_eq!(find(a).unwrap().meta.name, "tebakmakanan");
        }
    }

    #[test]
    fn kartu_soal_punya_hint_dan_waktu() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tebakmakanan".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            http: crate::game::quiz::test_support::http_gambar(),
            ..Default::default()
        };
        let r = handler(&CFG, ctx);
        let teks = r
            .without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string();
        assert!(teks.contains("💡 Hint:"), "{teks}");
        assert!(teks.contains("⏱️ Waktu:"), "{teks}");
        assert!(teks.contains("🎁 Hadiah:"), "{teks}");
    }

    #[test]
    fn nyerah_mengakhiri_sesi() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tebakmakanan".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "nyerah".into(),
            http: crate::game::quiz::test_support::http_gambar(),
            ..Default::default()
        };
        handler(&CFG, ctx.clone());
        assert!(ctx.session_get(crate::game::quiz::SESSIONS_KEY).is_some());
        let r = tebakmakanan_answer(&ctx).expect("nyerah dijawab");
        let teks = r
            .without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string();
        assert!(teks.contains("Jawaban:"), "{teks}");
    }
}
