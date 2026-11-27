//! Game quiz dari engine (`FotoGames`).

use crate::game::quiz::{QuizGame, answer, handler};
use oktz_plugin_api::{Ctx, Reply, answer_handler, plugin};

const CFG: QuizGame = QuizGame {
    name: "tebakkimia",
    alias: &["kimia", "chemistry", "unsur"],
    emoji: "🧪",
    title: "TEBAK KIMIA",
    description: "Tebak unsur kimia",
    timeout_ms: 60000,
    data_file: "tebakkimia.json",
    question_field: Some("unsur"),
    answer_field: "lambang",
    has_image: false,
    image_field: "img",
    hint_count: 2,
};

#[plugin(
    name = "tebakkimia",
    alias = ["kimia", "chemistry", "unsur"],
    category = "game",
    description = "Tebak unsur kimia",
    usage = ".tebakkimia",
    example = ".tebakkimia",
    cooldown = 5,
    energi = 0
)]
pub fn tebakkimia(ctx: Ctx) -> Reply {
    handler(&CFG, ctx)
}

#[answer_handler]
pub fn tebakkimia_answer(ctx: &Ctx) -> Option<Reply> {
    answer(&CFG, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn meta_dan_alias_terdaftar() {
        let p = find("tebakkimia").expect("terdaftar");
        assert_eq!(p.meta.name, "tebakkimia");
        for a in ["kimia", "chemistry", "unsur"] {
            assert_eq!(find(a).unwrap().meta.name, "tebakkimia");
        }
    }

    #[test]
    fn kartu_soal_punya_hint_dan_waktu() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "tebakkimia".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
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
            command: "tebakkimia".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "nyerah".into(),
            ..Default::default()
        };
        handler(&CFG, ctx.clone());
        assert!(ctx.session_get(crate::game::quiz::SESSIONS_KEY).is_some());
        let r = tebakkimia_answer(&ctx).expect("nyerah dijawab");
        let teks = r
            .without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string();
        assert!(teks.contains("Jawaban:"), "{teks}");
    }
}
