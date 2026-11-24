//! `plugins/game/siapakahaku.js` — game quiz dari engine `games.js` (`FotoGames`).
//!
//! Bank soal `src/data/siapakahaku.json` di-`include_str!` apa adanya (pola
//! `family100.rs`); handler dan answer handler di `game::quiz`.

use crate::game::quiz::{QuizGame, answer, handler};
use oktz_plugin_api::{Ctx, Reply, answer_handler, plugin};

const CFG: QuizGame = QuizGame {
    name: "siapakahaku",
    alias: &["siapa", "whoami"],
    emoji: "🎭",
    title: "SIAPAKAH AKU",
    description: "Tebak dari deskripsi",
    timeout_ms: 60000,
    data_file: "siapakahaku.json",
    question_field: Some("soal"),
    answer_field: "jawaban",
    has_image: false,
    image_field: "img",
    hint_count: 2,
};

#[plugin(
    name = "siapakahaku",
    alias = ["siapa", "whoami"],
    category = "game",
    description = "Tebak dari deskripsi",
    usage = ".siapakahaku",
    example = ".siapakahaku",
    cooldown = 5,
    energi = 0
)]
pub fn siapakahaku(ctx: Ctx) -> Reply {
    handler(&CFG, ctx)
}

#[answer_handler]
pub fn siapakahaku_answer(ctx: &Ctx) -> Option<Reply> {
    answer(&CFG, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn meta_dan_alias_terdaftar() {
        let p = find("siapakahaku").expect("terdaftar");
        assert_eq!(p.meta.name, "siapakahaku");
        // alias `siapa` = command plugin `bego` (`siapa.js:10`) — alias hanya
        // dipakai kalau nama tidak ada, jadi `.siapa` selalu ke `bego`.
        for a in ["whoami"] {
            assert_eq!(find(a).unwrap().meta.name, "siapakahaku");
        }
    }

    #[test]
    fn kartu_soal_punya_hint_dan_waktu() {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "siapakahaku".into(),
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
            command: "siapakahaku".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: "nyerah".into(),
            ..Default::default()
        };
        handler(&CFG, ctx.clone());
        assert!(ctx.session_get(crate::game::quiz::SESSIONS_KEY).is_some());
        let r = siapakahaku_answer(&ctx).expect("nyerah dijawab");
        let teks = r
            .without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string();
        assert!(teks.contains("Jawaban:"), "{teks}");
    }
}
