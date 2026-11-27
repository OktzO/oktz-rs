//! Game quiz dari engine (`FotoGames`).

use crate::game::quiz::{QuizGame, answer, handler};
use oktz_plugin_api::{Ctx, Reply, answer_handler, plugin};

const CFG: QuizGame = QuizGame {
    name: "tebakgambar",
    alias: &["tg", "guessimage"],
    emoji: "🖼️",
    title: "TEBAK GAMBAR",
    description: "Tebak kata dari gambar",
    timeout_ms: 90000,
    data_file: "tebakgambar.json",
    question_field: None,
    answer_field: "jawaban",
    has_image: true,
    image_field: "img",
    hint_count: 3,
};

#[plugin(
    name = "tebakgambar",
    alias = ["tg", "guessimage"],
    category = "game",
    description = "Tebak kata dari gambar",
    usage = ".tebakgambar",
    example = ".tebakgambar",
    cooldown = 5,
    energi = 0
)]
pub fn tebakgambar(ctx: Ctx) -> Reply {
    handler(&CFG, ctx)
}

#[answer_handler]
pub fn tebakgambar_answer(ctx: &Ctx) -> Option<Reply> {
    answer(&CFG, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn meta_dan_alias_terdaftar() {
        let p = find("tebakgambar").expect("terdaftar");
        assert_eq!(p.meta.name, "tebakgambar");
        for a in ["tg", "guessimage"] {
            assert_eq!(find(a).unwrap().meta.name, "tebakgambar");
        }
    }

    #[test]
    fn kartu_soal_punya_hint_dan_waktu() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tebakgambar".into(),
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
            command: "tebakgambar".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "nyerah".into(),
            http: crate::game::quiz::test_support::http_gambar(),
            ..Default::default()
        };
        handler(&CFG, ctx.clone());
        assert!(ctx.session_get(crate::game::quiz::SESSIONS_KEY).is_some());
        let r = tebakgambar_answer(&ctx).expect("nyerah dijawab");
        let teks = r
            .without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string();
        assert!(teks.contains("Jawaban:"), "{teks}");
    }
}
