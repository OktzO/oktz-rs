//! Engine umum 22 game quiz — port `src/lib/games.js` (`FotoGames`),
//! `src/lib/game-data.js`, dan bagian reward `src/lib/context.js`.
//!
//! Satu engine, 22 plugin tipis: tiap game cuma memakai konstanta
//! [`QuizGame`] + `#[plugin]`/`#[answer_handler]` wrapper, pola
//! `family100.rs`.
//!
//! `ponytail:` `setSessionTimer` butuh event loop; sesi kedaluwarsa dihapus
//! saat dibaca berikutnya (pola `family100.rs`), bukan timer yang memanggil
//! `endSession`.

use oktz_plugin_api::{Ctx, MediaSource, Mutation, Reply};
use serde_json::{Map, Value, json};

use crate::common::now_ms;
use crate::rpg::heal::{apply_exp, rpg_map};

/// `gameSessions` (`game-data.js:7`) — map bersama semua game, key `chatId`.
pub(crate) const SESSIONS_KEY: &str = "gameSessions";

const FAST_ANSWER_THRESHOLD_MS: i64 = 4_000;

/// `website` (`config.js`) — awalan caption preview seperti `sendGamePreview`.
fn website(ctx: &Ctx) -> String {
    ctx.config().info.website.clone()
}

pub(crate) struct QuizGame {
    pub name: &'static str,
    pub alias: &'static [&'static str],
    pub emoji: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub timeout_ms: i64,
    /// `dataFile` JS — nama berkas JSON di `data/`.
    pub data_file: &'static str,
    /// `None` = `questionField: null` (tebakgambar).
    pub question_field: Option<&'static str>,
    pub answer_field: &'static str,
    pub has_image: bool,
    pub image_field: &'static str,
    pub hint_count: usize,
}

/// `checkAnswerAdvanced` status.
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Correct,
    Close,
    Wrong,
}

pub struct AnswerResult {
    pub status: Status,
    pub similarity: f64,
}

/// `normalizeAnswer` (`game-data.js:57`).
pub fn normalize_answer(answer: &str) -> String {
    let mut out = String::with_capacity(answer.len());
    let mut prev_space = true;
    for c in answer.chars() {
        if c.is_ascii_alphanumeric() || c.is_ascii_whitespace() {
            let c = c.to_ascii_lowercase();
            if c.is_ascii_whitespace() {
                if !prev_space {
                    out.push(' ');
                }
                prev_space = true;
            } else {
                out.push(c);
                prev_space = false;
            }
        }
    }
    out.trim().to_string()
}

/// `levenshteinDistance` + `getSimilarity` (`game-data.js:68-93`).
fn similarity(a: &str, b: &str) -> f64 {
    let x: Vec<char> = a.chars().collect();
    let y: Vec<char> = b.chars().collect();
    let max_len = x.len().max(y.len());
    if max_len == 0 {
        return 1.0;
    }
    let mut dp: Vec<usize> = (0..=y.len()).collect();
    for (i, &xi) in x.iter().enumerate() {
        let mut prev = dp[0];
        dp[0] = i + 1;
        for (j, &yj) in y.iter().enumerate() {
            let tmp = dp[j + 1];
            dp[j + 1] = if xi == yj {
                prev
            } else {
                1 + dp[j].min(dp[j + 1]).min(prev)
            };
            prev = tmp;
        }
    }
    (max_len - dp[y.len()]) as f64 / max_len as f64
}

/// `checkAnswerAdvanced` (`game-data.js:95`).
pub fn check_answer_advanced(correct: &str, user: &str) -> AnswerResult {
    let n1 = normalize_answer(correct);
    let n2 = normalize_answer(user);
    if n1 == n2 {
        return AnswerResult {
            status: Status::Correct,
            similarity: 1.0,
        };
    }
    if (n1.contains(&n2) || n2.contains(&n1)) && n2.len() as f64 >= n1.len() as f64 * 0.8 {
        return AnswerResult {
            status: Status::Correct,
            similarity: 0.95,
        };
    }
    let s = similarity(&n1, &n2);
    if s >= 0.85 {
        AnswerResult {
            status: Status::Correct,
            similarity: s,
        }
    } else if s >= 0.6 {
        AnswerResult {
            status: Status::Close,
            similarity: s,
        }
    } else {
        AnswerResult {
            status: Status::Wrong,
            similarity: s,
        }
    }
}

/// `getHint` (`game-data.js:127`).
pub fn get_hint(answer: &str, reveal_count: usize) -> String {
    answer
        .chars()
        .enumerate()
        .map(|(i, c)| if c == ' ' || i < reveal_count { c } else { '_' })
        .collect()
}

/// `getProgressiveHint` (`game-data.js:253`) — reveal = 2 + attempts/2,
/// dihitung non-spasi.
pub fn progressive_hint(answer: &str, attempts: i64) -> String {
    let total = answer.chars().filter(|c| *c != ' ').count();
    let reveal = (total).min(2 + (attempts as usize) / 2);
    let mut revealed = 0;
    answer
        .chars()
        .map(|c| {
            if c == ' ' {
                return c;
            }
            if revealed < reveal {
                revealed += 1;
                return c;
            }
            '_'
        })
        .collect()
}

/// `isSurrender` (`game-data.js:138`) — `===` per kata, bukan `includes`.
pub fn is_surrender(text: &str) -> bool {
    const WORDS: &[&str] = &[
        "nyerah",
        "aku nyerah",
        "gw nyerah",
        "gue nyerah",
        "menyerah",
        "aku menyerah",
        "gw menyerah",
        "skip",
        "lewat",
        "ga tau",
        "gatau",
        "gak tau",
        "tidak tau",
        "nggak tau",
        "give up",
    ];
    let normalized = text.to_lowercase().trim().to_string();
    WORDS.contains(&normalized.as_str())
}

/// Bank soal dari `data/` — `include_str!` + `OnceLock`, pola `family100.rs`.
pub fn bank(data_file: &str) -> &'static Vec<Value> {
    static EMPTY: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    static CACHE: std::sync::OnceLock<std::collections::HashMap<&'static str, Vec<Value>>> =
        std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        for (name, raw) in BANKS {
            let v: Vec<Value> = serde_json::from_str(raw).unwrap_or_default();
            m.insert(*name, v);
        }
        m
    });
    cache
        .get(data_file.trim_end_matches(".json"))
        .unwrap_or_else(|| EMPTY.get_or_init(Vec::new))
}

/// `getRandomItem(dataFile)` — `None` kalau bank kosong.
pub(crate) fn random_question(data_file: &str) -> Option<&'static Value> {
    use rand::Rng as _;
    let b = bank(data_file);
    if b.is_empty() {
        return None;
    }
    b.get(rand::rng().random_range(0..b.len()))
}

/// `getRandomReward` (`game-data.js:245`) — (limit, koin, exp).
fn random_reward() -> (i64, i64, i64) {
    use rand::Rng as _;
    let mut rng = rand::rng();
    (
        rng.random_range(3..=8),
        rng.random_range(500..=2000),
        rng.random_range(1000..=3000),
    )
}

const FAST_ANSWER_PRAISES: [&str; 10] = [
    "⚡ Kilat banget! Kamu jenius!",
    "🚀 Super cepat! Otak encer!",
    "🔥 Wuih monster! Jawab secepat kilat!",
    "💫 Luar biasa! Kamu the flash!",
    "🎯 Precision tinggi! Langsung tepat!",
    "⭐ Bintang! Refleks dewa!",
    "🏆 Legend! Kecepatan maximal!",
    "💎 Premium player! Gak ada lawan!",
    "🦅 Tajam seperti elang!",
    "🧠 Big brain! IQ tinggi detected!",
];

const WIN_MESSAGES: [&str; 6] = [
    "🌟 *GG WP! Otakmu encer!*",
    "✨ *KEREN ABIS! Lu emang pinter!*",
    "🎉 *MANTAPPPP! Jawaban sempurna!*",
    "💫 *EPIC! Gak ada lawan lu!*",
    "🏆 *NGERI! Otak lu kayak Google!*",
    "🔥 *LEGEND! Jawab kek gak ada beban!*",
];

const TIMEOUT_MESSAGES: [&str; 3] = [
    "⏱️ *Yah telat, waktu habis!*",
    "⏱️ *WAKTU HABIS!*",
    "⏱️ *Telat bro, waktu dah abis!*",
];

const SURRENDER_MESSAGES: [&str; 3] = [
    "🏳️ *Yahhh nyerah deh...*",
    "🏳️ *MENYERAH!*",
    "🏳️ *Yah sayang banget nyerah...*",
];

fn pick<const N: usize>(arr: [&str; N]) -> usize {
    use rand::Rng as _;
    rand::rng().random_range(0..arr.len())
}

fn pick_str<const N: usize>(arr: [&'static str; N]) -> &'static str {
    arr[pick(arr)]
}

/// `formatRemainingTime` (`game-data.js:219`).
fn format_remaining_time(seconds: i64) -> String {
    if seconds <= 0 {
        return String::from("0 detik");
    }
    if seconds < 60 {
        return format!("{seconds} detik");
    }
    format!("{}m {}s", seconds / 60, seconds % 60)
}

/// `getRemainingTime` — detik, ceil, tidak negatif.
fn remaining_time(end_time: i64, now: i64) -> i64 {
    (((end_time - now) as f64) / 1000.0).ceil().max(0.0) as i64
}

fn session_of(ctx: &Ctx) -> Option<Value> {
    ctx.session_get(SESSIONS_KEY)?.get(&ctx.chat).cloned()
}

fn question_str<'a>(q: &'a Value, field: Option<&str>) -> Option<&'a str> {
    field.and_then(|f| q.get(f)).and_then(Value::as_str)
}

/// `handler` (`games.js:80-181`) — entry command.
pub(crate) fn handler(cfg: &QuizGame, ctx: Ctx) -> Reply {
    let now = now_ms();

    // Cabang "ada game jalan" (`games.js:88-115`).
    if let Some(session) = session_of(&ctx) {
        if session.get("gameType").and_then(Value::as_str) == Some(cfg.name) {
            let end_time = session.get("endTime").and_then(Value::as_i64).unwrap_or(0);
            if remaining_time(end_time, now) > 0 {
                let question = session.get("question").cloned().unwrap_or(Value::Null);
                let answer = question
                    .get(cfg.answer_field)
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let mut text = String::from("⚠️ *Eh ada game jalan nih, jawab dulu!*\n\n");
                if let Some(soal) = question_str(&question, cfg.question_field) {
                    text += &format!("```\n{soal}\n```\n\n");
                }
                text += &format!("💡 Hint: *{}*\n", get_hint(answer, cfg.hint_count));
                text += &format!(
                    "⏱️ Sisa: *{}*\n\n",
                    format_remaining_time(remaining_time(end_time, now))
                );
                text += "_Jawab langsung atau ketik \"nyerah\"\nSetiap salah, hint akan bertambah_";
                let web = website(&ctx);
                return Reply::preview(
                    format!("{web}\n\n{web}\n{text}"),
                    web,
                    cfg.title,
                    cfg.description,
                    None,
                );
            }
            ctx.session_take(SESSIONS_KEY);
        }
    }

    let Some(question) = random_question(cfg.data_file) else {
        return Reply::text("❌ *ᴅᴀᴛᴀ ᴛɪᴅᴀᴋ ᴛᴇʀsᴇᴅɪᴀ*\n\n> Data game tidak tersedia!");
    };
    let answer = question
        .get(cfg.answer_field)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let mut caption = format!("{} *{}*\n\n", cfg.emoji, cfg.title);
    if let Some(soal) = question_str(question, cfg.question_field) {
        caption += &format!("> {soal}\n");
    }
    caption += &format!("💡 Hint: *{}*\n", get_hint(&answer, cfg.hint_count));
    caption += &format!("⏱️ Waktu: *{} detik*\n", cfg.timeout_ms / 1000);
    caption += "🎁 Hadiah: *Limit, Koin, EXP (random)*\n\n";
    caption += "_Jawab langsung atau ketik \"nyerah\"\nSetiap salah, hint akan bertambah_";

    let session = json!({
        "chatId": ctx.chat,
        "gameType": cfg.name,
        "question": question,
        "startTime": now,
        "timeout": cfg.timeout_ms,
        "endTime": now + cfg.timeout_ms,
        "attempts": 0,
    });
    let mut sessions = ctx.session_get(SESSIONS_KEY).unwrap_or(json!({}));
    sessions[&ctx.chat] = session;
    ctx.session_set(SESSIONS_KEY, sessions);

    if cfg.has_image {
        let img = question
            .get(cfg.image_field)
            .and_then(Value::as_str)
            .unwrap_or("");
        if img.is_empty() {
            return Reply::text("❌ *ɢᴀɢᴀʟ ᴍᴇᴍᴜᴀᴛ ɢᴀᴍʙᴀʀ*\n\n> Coba lagi nanti!");
        }
        // `fetchBuffer(question[imageField])` JS — byte gambar di pipeline.
        return Reply::image(MediaSource::url(img), caption).with_reaction("🎲");
    }
    let web = website(&ctx);
    Reply::preview(
        format!("{web}\n\n{caption}"),
        web,
        format!("{} {}", cfg.emoji, cfg.title),
        "Jawab pertanyaan!",
        None,
    )
}

/// `answerHandler` (`games.js:183-296`).
pub(crate) fn answer(cfg: &QuizGame, ctx: &Ctx) -> Option<Reply> {
    let user_answer = ctx.text.trim().to_string();
    if user_answer.is_empty() || user_answer.starts_with('.') {
        return None;
    }

    let session = session_of(ctx)?;
    if session.get("gameType").and_then(Value::as_str) != Some(cfg.name) {
        return None;
    }

    let now = now_ms();
    let end_time = session.get("endTime").and_then(Value::as_i64).unwrap_or(0);
    if remaining_time(end_time, now) <= 0 {
        ctx.session_take(SESSIONS_KEY);
        return None;
    }

    let question = session.get("question").cloned().unwrap_or(Value::Null);
    let answer = question
        .get(cfg.answer_field)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    // `isSurrender` (`games.js:210-221`) — TIDAK butuh reply pesan game.
    if is_surrender(&user_answer) {
        ctx.session_take(SESSIONS_KEY);
        let text = format!(
            "{}\n\nJawaban: *{answer}*\n\n_@{} menyerah_",
            pick_str(SURRENDER_MESSAGES),
            crate::common::jid_number(&ctx.sender)
        );
        return Some(Reply::Text {
            text,
            mentions: vec![ctx.sender.clone()],
        });
    }

    // `isReplyToGame` (`game-data.js:227`).
    let replied = ctx.quoted().is_some_and(|q| {
        q.from_me || {
            let message_key = session.get("messageKey");
            message_key
                .and_then(|k| k.get("id"))
                .and_then(Value::as_str)
                == Some(q.id.as_str())
        }
    });
    if !replied {
        return None;
    }

    let attempts = session.get("attempts").and_then(Value::as_i64).unwrap_or(0) + 1;
    let result = check_answer_advanced(&answer, &user_answer);

    match result.status {
        Status::Correct => {
            ctx.session_take(SESSIONS_KEY);

            let (mut limit, mut koin, mut exp) = random_reward();
            let mut bonus_text = String::new();

            // `checkFastAnswer` (`context.js:155`).
            let start_time = session
                .get("startTime")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let elapsed = now - start_time;
            if elapsed <= FAST_ANSWER_THRESHOLD_MS {
                let praise = pick_str(FAST_ANSWER_PRAISES);
                bonus_text = format!(
                    "\n\n{praise}\n⚡ *BONUS KILAT:* +1 Limit, +500 Koin\n⏱️ Waktu: *{:.1}s*",
                    elapsed as f64 / 1000.0
                );
                limit += 1;
                koin += 500;
                exp += 50;
            }

            let mut rpg = rpg_map(ctx);
            let old_exp = ctx.user_field("exp").and_then(Value::as_i64).unwrap_or(0);
            exp = apply_exp(&mut rpg, old_exp, exp);

            let old_koin = ctx.user_field("koin").and_then(Value::as_i64).unwrap_or(0);
            let new_koin = if old_koin == -1 {
                -1
            } else {
                (old_koin + koin).clamp(0, crate::rpg::heal::MAX_KOIN)
            };
            let old_energi = ctx
                .user_field("energi")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let new_energi = if old_energi == -1 {
                -1
            } else {
                (old_energi + limit).max(0)
            };

            let mut text = format!("{}\n\n", pick_str(WIN_MESSAGES));
            text += &format!("Jawaban: *{answer}*\n");
            text += &format!("Pemenang: *@{}*\n", crate::common::jid_number(&ctx.sender));
            text += &format!("Percobaan: *{attempts}x*\n\n");
            let mut parts: Vec<String> = Vec::new();
            if limit > 0 {
                parts.push(format!("+{limit} Limit"));
            }
            if koin > 0 {
                parts.push(format!("+{koin} Koin"));
            }
            if exp > 0 {
                parts.push(format!("+{exp} EXP"));
            }
            if !parts.is_empty() {
                text += &format!("🎁 {}", parts.join(", "));
            }
            text += &bonus_text;

            let mut data = Map::new();
            data.insert("exp".into(), json!(exp));
            data.insert("koin".into(), json!(new_koin));
            data.insert("energi".into(), json!(new_energi));
            data.insert("rpg".into(), Value::Object(rpg));

            Some(
                Reply::Text {
                    text,
                    mentions: vec![ctx.sender.clone()],
                }
                .mutating(vec![Mutation::SetUser {
                    jid: ctx.sender.clone(),
                    data: Value::Object(data),
                }])
                .with_reaction("✅"),
            )
        }
        Status::Close => {
            let percent = (result.similarity * 100.0).round() as i64;
            Some(
                Reply::text(format!(
                    "🔥 *Hampir!* Jawabanmu *{percent}%* mirip!\n_Sisa waktu: *{}*_",
                    format_remaining_time(remaining_time(end_time, now))
                ))
                .with_reaction("🔥"),
            )
        }
        Status::Wrong => {
            if remaining_time(end_time, now) > 0 && attempts < 10 {
                let hint = progressive_hint(&answer, attempts);
                let mut sessions = ctx.session_get(SESSIONS_KEY).unwrap_or(json!({}));
                sessions[&ctx.chat]["attempts"] = json!(attempts);
                ctx.session_set(SESSIONS_KEY, sessions);
                return Some(
                    Reply::text(format!(
                        "❌ Belum bener! Hint: *{hint}*\n_Sisa: *{}*_",
                        format_remaining_time(remaining_time(end_time, now))
                    ))
                    .with_reaction("❌"),
                );
            }
            None
        }
    }
}

include!("quiz_banks.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use Status::{Close, Correct, Wrong};

    // ------------------------------------------------ unit engine (port game-data.js)

    fn sim(c: &str) -> String {
        normalize_answer(c)
    }

    #[test]
    fn normalize_membuang_non_alnum() {
        assert_eq!(sim("  Halo   Dunia! "), "halo dunia");
        assert_eq!(sim("KUCUNG"), "kucung");
        assert_eq!(sim(""), "");
    }

    #[test]
    fn check_answer_advanced_empat_status() {
        assert_eq!(check_answer_advanced("BIOSKOP", "bioskop").status, Correct);
        // substring >= 80% panjang
        assert_eq!(
            check_answer_advanced("Langkah Kaki", "langkah").status,
            Wrong
        );
        assert_eq!(
            check_answer_advanced("Langkah Kaki", "Langkah Kaki Banget").status,
            Correct
        );
        // similarity >= 0.85
        assert_eq!(
            check_answer_advanced("sheraton", "sheratun").status,
            Correct
        );
        // 0.6..0.85 = close
        assert_eq!(check_answer_advanced("kucung", "kucing").status, Close);
        assert_eq!(check_answer_advanced("kucung", "xyz").status, Wrong);
    }

    #[test]
    fn get_hint_menampilkani_n_karakter_pertama() {
        assert_eq!(get_hint("BIOSKOP", 2), "BI_____");
        assert_eq!(get_hint("Langkah Kaki", 2), "La_____ ____");
        assert_eq!(get_hint("", 2), "");
    }

    #[test]
    fn progressive_hint_menghitung_non_spasi() {
        assert_eq!(progressive_hint("BIOSKOP", 0), "BI_____");
        assert_eq!(progressive_hint("BIOSKOP", 1), "BI_____");
        assert_eq!(progressive_hint("BIOSKOP", 2), "BIO____");
        assert_eq!(progressive_hint("Langkah Kaki", 4), "Lang___ ____");
        assert_eq!(progressive_hint("Langkah Kaki", 20), "Langkah Kaki");
    }

    #[test]
    fn is_surrender_daftar_kata_js() {
        for w in [
            "nyerah",
            "aku nyerah",
            "skip",
            "gak tau",
            "give up",
            "MENYERAH",
        ] {
            assert!(is_surrender(w), "{w}");
        }
        assert!(!is_surrender("kucing"));
        assert!(!is_surrender(""));
    }

    #[test]
    fn bank_tebakkata_302_entri() {
        assert_eq!(bank("tebakkata").len(), 302);
        assert_eq!(bank("tebakbendera2").len(), 254);
    }

    #[test]
    fn reward_dalam_rentang_js() {
        for _ in 0..20 {
            let r = random_reward();
            assert!((3..=8).contains(&r.0), "limit {r:?}");
            assert!((500..=2000).contains(&r.1), "koin {r:?}");
            assert!((1000..=3000).contains(&r.2), "exp {r:?}");
        }
    }
}
