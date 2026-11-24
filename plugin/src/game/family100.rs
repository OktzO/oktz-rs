//! `plugins/game/family100.js` — tebak jawaban teratas survei.
//!
//! `ponytail:` `family100AnswerHandler` bukan command; router global yang
//! memanggilnya (sama seperti `kyubigameAnswerHandler` di `kyubigame.js`).
//! Yang dipindah di sini `handler`.
//!
//! Bank soal `src/data/family100.json` di-`include_str!` apa adanya, bukan
//! disalin jadi konstanta Rust: 7152 soal / 1,6 MB jauh lebih besar dari file
//! plugin mana pun, dan menyalinnya jadi array `const` berarti data punya dua
//! sumber yang bisa melenceng diam-diam.
//!
//! `ponytail:` `setSessionTimer` butuh event loop, sedangkan `Reply` tidak
//! punya timer. Sesi yang lewat 120 detik dianggap habis saat dibaca
//! berikutnya — efek yang sama dengan sweeper 5-menit di `game-data.js`,
//! tanpa timer yang benar-benar memanggil `endSession`.

use oktz_plugin_api::{Ctx, Mutation, Reply, answer_handler, plugin};
use serde_json::{Value, json};

/// `createSession(..., 120000)` (`family100.js:84`).
pub(crate) const SESSION_TIMEOUT_MS: i64 = 120_000;

/// `gameSessions` di `game-data.js:7` — satu map bersama untuk semua game,
/// di-key `chatId`. Nama key dipertahankan supaya plugin game lain yang juga
/// memakainya tidak membuat sesi kedua untuk chat yang sama.
pub(crate) const SESSIONS_KEY: &str = "gameSessions";

const BANK: &str = include_str!("../../../data/family100.json");

/// Satu baris `src/data/family100.json`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Question {
    pub soal: String,
    pub jawaban: Vec<String>,
}

fn bank() -> &'static Vec<Question> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<Question>> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str::<Vec<Value>>(BANK)
            .map(|rows| {
                rows.iter()
                    .map(|row| Question {
                        soal: row
                            .get("soal")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        jawaban: row
                            .get("jawaban")
                            .and_then(Value::as_array)
                            .map(|a| {
                                a.iter()
                                    .filter_map(Value::as_str)
                                    .map(str::to_string)
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// `getRandomItem("family100.json")` — `None` kalau bank kosong
/// (`family100.js:59-63`).
pub(crate) fn random_question() -> Option<&'static Question> {
    use rand::Rng as _;
    let b = bank();
    if b.is_empty() {
        return None;
    }
    b.get(rand::rng().random_range(0..b.len()))
}

/// `formatRemainingTime` (`game-data.js:189`).
pub(crate) fn format_remaining_time(seconds: i64) -> String {
    if seconds <= 0 {
        return String::from("0 detik");
    }
    if seconds < 60 {
        return format!("{seconds} detik");
    }
    format!("{}m {}s", seconds / 60, seconds % 60)
}

/// `getRemainingTime` (`game-data.js:181`) — detik, `Math.ceil`, tidak negatif.
pub(crate) fn remaining_time(end_time: i64, now: i64) -> i64 {
    (((end_time - now) as f64) / 1000.0).ceil().max(0.0) as i64
}

fn session_of(ctx: &Ctx) -> Option<Value> {
    ctx.session_get(SESSIONS_KEY)?.get(&ctx.chat).cloned()
}

/// `family100.js:43-53` — kartu "sesi masih jalan".
pub(crate) fn in_progress(session: &Value, remaining_secs: i64) -> String {
    let question = session.get("question");
    let soal = question
        .and_then(|q| q.get("soal"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let total = question
        .and_then(|q| q.get("jawaban"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let answered: Vec<String> = session
        .get("answered")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let mut text = format!("Wah, sesi Family 100 masih jalan nih kak! 😱✨\n\n*{soal}*\n\n");
    text.push_str(&format!("Terjawab: *{} dari {total}*\n", answered.len()));
    for (i, ans) in answered.iter().enumerate() {
        text.push_str(&format!("{}. ✅ {ans}\n", i + 1));
    }
    for i in answered.len()..total {
        text.push_str(&format!("{}. ❓ ???\n", i + 1));
    }
    text.push_str(&format!(
        "\nSisa waktu: *{}* ⏳\nBuruan di-reply pesannya buat jawab! 🔥",
        format_remaining_time(remaining_secs)
    ));
    text
}

/// `family100.js:67-75` — kartu soal baru.
pub(crate) fn question_card(question: &Question) -> String {
    let mut text = format!(
        "Waktunya main *FAMILY 100*! 🎉✨\n\n*Pertanyaan:* {}\n\nTotal Jawaban: *{}* 📝\n",
        question.soal,
        question.jawaban.len()
    );
    for i in 1..=question.jawaban.len() {
        text.push_str(&format!("{i}. ❓ ???\n"));
    }
    text.push_str(
        "\nWaktu kamu cuman *120 detik* aja ya! ⏱️\n\
         Hadiahnya? Random *EXP* & *Koin* buat setiap jawaban bener! 🎁💸\n\n\
         Cara main: langsung *reply pesan ini* dengan jawabanmu, atau reply dengan kata *nyerah* kalau udah pusing banget 🏳️😂",
    );
    text
}

/// `family100.js:61` — bank soal kosong.
pub(crate) const NO_QUESTION: &str = "Yah maaf banget kak, soal gamenya lagi kosong nih 😭💔";

#[plugin(
    name = "family100",
    alias = ["f100"],
    category = "game",
    description = "Survey says! Tebak jawaban teratas survei",
    usage = ".family100",
    example = ".family100",
    cooldown = 5,
    energi = 0,
    is_group = true,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn family100(ctx: Ctx) -> Reply {
    let now = crate::common::now_ms();
    if let Some(session) = session_of(&ctx)
        && session.get("gameType").and_then(Value::as_str) == Some("family100")
    {
        let end_time = session.get("endTime").and_then(Value::as_i64).unwrap_or(0);
        let remaining = remaining_time(end_time, now);
        if remaining > 0 {
            return Reply::text(in_progress(&session, remaining));
        }
        // Lewat waktu. Di JS `setSessionTimer` yang memanggil `endSession`;
        // tanpa timer, sesi dibuang di sini.
        ctx.session_take(SESSIONS_KEY);
    }

    let Some(question) = random_question() else {
        return Reply::text(NO_QUESTION);
    };

    let chat = ctx.chat.clone();
    ctx.session_update(SESSIONS_KEY, |sessions| {
        // `session_update` memberi `Value::Null` untuk key baru, sedangkan JS
        // menulis ke objek `gameSessions` yang sudah ada.
        if !sessions.is_object() {
            *sessions = serde_json::json!({});
        }
        let entry = sessions
            .as_object_mut()
            .expect("baru saja diganti objek")
            .entry(chat.clone())
            .or_insert(Value::Null);
        *entry = serde_json::json!({
            "chatId": chat,
            "gameType": "family100",
            "question": { "soal": question.soal, "jawaban": question.jawaban },
            "answered": [],
            "answeredBy": {},
            "startTime": now,
            "endTime": now + SESSION_TIMEOUT_MS,
        });
    });

    Reply::text(question_card(question))
}

/// `isSurrender` (`game-data.js:138-147`).
const SURRENDER_WORDS: [&str; 15] = [
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

pub(crate) fn is_surrender(text: &str) -> bool {
    SURRENDER_WORDS.contains(&text)
}

/// `getSimilarity` (`family100.js:219-243`) — Levenshtein yang dinormalkan
/// panjang kata lebih panjang.
fn similarity(a: &str, b: &str) -> f64 {
    let (longer, shorter): (Vec<char>, Vec<char>) = if a.chars().count() > b.chars().count() {
        (a.chars().collect(), b.chars().collect())
    } else {
        (b.chars().collect(), a.chars().collect())
    };
    if longer.is_empty() {
        return 1.0;
    }
    let mut costs: Vec<usize> = (0..=shorter.len()).collect();
    for i in 1..=longer.len() {
        let mut last = i;
        for j in 1..=shorter.len() {
            let new_value = if longer[i - 1] == shorter[j - 1] {
                costs[j - 1]
            } else {
                costs[j - 1].min(last).min(costs[j]) + 1
            };
            costs[j - 1] = last;
            last = new_value;
        }
        costs[shorter.len()] = last;
    }
    (longer.len() - costs[shorter.len()]) as f64 / longer.len() as f64
}

/// `getRandomReward` (`game-data.js:245-251`).
fn random_reward() -> (i64, i64) {
    use rand::Rng as _;
    let mut rng = rand::rng();
    let exp = rng.random_range(1000..=3000);
    let koin = rng.random_range(500..=2000);
    (exp, koin)
}

fn jawaban_of(session: &Value) -> Vec<String> {
    session
        .get("question")
        .and_then(|q| q.get("jawaban"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn answered_of(session: &Value) -> Vec<String> {
    session
        .get("answered")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn soal_of(session: &Value) -> String {
    session
        .get("question")
        .and_then(|q| q.get("soal"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// Kartu daftar jawaban setelah satu jawaban benar (`family100.js:196-210`).
fn correct_card(soal: &str, jawaban: &[String], answered: &[String]) -> String {
    let mut text = format!("*Pertanyaan:* {soal}\n\n");
    for (i, ans) in jawaban.iter().enumerate() {
        if answered.iter().any(|a| a.eq_ignore_ascii_case(ans)) {
            text.push_str(&format!("{}. ✅ {ans}\n", i + 1));
        } else {
            text.push_str(&format!("{}. ❓ ???\n", i + 1));
        }
    }
    text
}

/// `family100AnswerHandler` (`family100.js:111-217`) — jawaban main, bukan
/// command. `None` = `return false` (biarkan plugin lain menangani).
#[answer_handler]
pub fn family100_answer(ctx: &Ctx) -> Option<Reply> {
    let user_answer = ctx.text.trim().to_lowercase();
    if user_answer.is_empty() {
        return None;
    }

    let sesi = session_of(ctx)?;
    if sesi.get("gameType").and_then(Value::as_str) != Some("family100") {
        return None;
    }

    let now = crate::common::now_ms();
    let end_time = sesi.get("endTime").and_then(Value::as_i64).unwrap_or(0);
    if remaining_time(end_time, now) <= 0 {
        // JS `setSessionTimer` mengirim kartu waktu habis; tanpa timer, sesi
        // basi dibuang di sini dan jawaban tidak dianggap milik game.
        ctx.session_take(SESSIONS_KEY);
        return None;
    }

    let jawaban = jawaban_of(&sesi);
    let answered = answered_of(&sesi);
    // `isReplyToGame` (`game-data.js:227-233`) membandingkan id pesan game.
    // Id kiriman bot tidak tersedia di `Ctx`, jadi `fromMe` yang dipakai.
    let quoting = ctx.quoted().is_some_and(|q| q.from_me);

    if is_surrender(&user_answer) {
        ctx.session_take(SESSIONS_KEY);
        let sisa: Vec<String> = jawaban
            .iter()
            .filter(|j| !answered.iter().any(|a| a.eq_ignore_ascii_case(j)))
            .cloned()
            .collect();
        let mut text = "Walahh pada nyerah nih ceritanya? 🥺🏳️\n\n".to_string();
        text.push_str(&format!(
            "Padahal udah nebak *{} dari {}* lho! 👏\n\n",
            answered.len(),
            jawaban.len()
        ));
        if !sisa.is_empty() {
            text.push_str("Nih aku kasih tau jawaban sisanya:\n");
            for ans in &sisa {
                text.push_str(&format!("• {ans}\n"));
            }
        }
        text.push_str("\nGapapa, next time pasti bisa full senyum! 💖✨");
        return Some(Reply::text(text));
    }

    if answered.iter().any(|a| a == &user_answer) {
        if quoting {
            return Some(
                Reply::text(format!(
                    "Hayo lho, jawaban *{user_answer}* udah ada yang jawab tadi kak! Cari yang lain dong 😂✨"
                ))
                .with_reaction("⚠️"),
            );
        }
        return None;
    }

    let Some(idx) = jawaban.iter().position(|ans| {
        let a = ans.to_lowercase();
        similarity(&a, &user_answer) >= 0.8 || a.contains(&user_answer) || user_answer.contains(&a)
    }) else {
        if quoting {
            return Some(
                Reply::text("Tettt! ❌ Salah kak! Coba dipikir-pikir lagi deh 😂🧠")
                    .with_reaction("❌"),
            );
        }
        return None;
    };

    let original = jawaban[idx].clone();
    let orig_lower = original.to_lowercase();
    let sender = ctx.sender.clone();
    let chat = ctx.chat.clone();
    let (exp_reward, koin_reward) = random_reward();

    ctx.session_update(SESSIONS_KEY, |sessions| {
        let Some(s) = sessions.get_mut(&chat) else {
            return;
        };
        if let Some(arr) = s.get_mut("answered").and_then(Value::as_array_mut)
            && !arr.iter().any(|v| v.as_str() == Some(orig_lower.as_str()))
        {
            arr.push(json!(orig_lower));
        }
        if let Some(by) = s.get_mut("answeredBy").and_then(Value::as_object_mut) {
            by.insert(orig_lower.clone(), json!(sender));
        }
    });

    let sesi = session_of(ctx)?;
    let answered = answered_of(&sesi);
    let soal = soal_of(&sesi);
    let total = jawaban.len();

    let old_exp = ctx.user_field("exp").and_then(Value::as_i64).unwrap_or(0);
    let old_koin = ctx.user_field("koin").and_then(Value::as_i64).unwrap_or(0);
    let new_exp = if old_exp == -1 {
        -1
    } else {
        old_exp.saturating_add(exp_reward).clamp(0, 9_000_000_000)
    };
    let new_koin = old_koin.saturating_add(koin_reward).max(0);
    let mutasi = Mutation::SetUser {
        jid: sender.clone(),
        data: json!({ "exp": new_exp, "koin": new_koin }),
    };

    if answered.len() == total {
        ctx.session_take(SESSIONS_KEY);
        let mut by: Vec<(String, String)> = sesi
            .get("answeredBy")
            .and_then(Value::as_object)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        by.sort_by(|a, b| a.0.cmp(&b.0));
        let mut mentions: Vec<String> = Vec::new();
        for (_, who) in &by {
            if !mentions.contains(who) {
                mentions.push(who.clone());
            }
        }
        let mut text = "WOAAHH KEREN BANGET! Semua jawaban ketebak dong! 🎉🔥✨\n\n".to_string();
        text.push_str(&format!("*Pertanyaan:* {soal}\n\n"));
        for (i, ans) in jawaban.iter().enumerate() {
            let who = by
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(ans))
                .map(|(_, v)| v.split('@').next().unwrap_or("?").to_string())
                .unwrap_or_else(|| "?".to_string());
            text.push_str(&format!("{}. ✅ {ans} - @{who}\n", i + 1));
        }
        text.push_str(
            "\n🎊 Selamat buat kalian semua yang udah ikutan mikir! Gacor banget otaknya! 🧠💯",
        );
        return Some(
            Reply::Text { text, mentions }
                .mutating(vec![mutasi])
                .with_reaction("🎉"),
        );
    }

    let sisa = total - answered.len();
    let mut text = format!(
        "Benerrr banget! ✅🎉\n@{} dapet *+{exp_reward} EXP* & *+{koin_reward} Koin* nih! 💸✨\n\n",
        crate::common::jid_number(&sender)
    );
    text.push_str(&correct_card(&soal, &jawaban, &answered));
    text.push_str(&format!("\nAyo gas sisa *{sisa}* jawaban lagi kak! 🔥⏱️"));
    Some(
        Reply::Text {
            text,
            mentions: vec![sender],
        }
        .mutating(vec![mutasi])
        .with_reaction("✅"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAT: &str = "120363@g.us";
    const SENDER: &str = "628111@s.whatsapp.net";

    fn base_ctx() -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "family100".into(),
            sender: SENDER.into(),
            chat: CHAT.into(),
            is_group: true,
            ..Default::default()
        }
    }

    /// `Ctx` dengan sesi yang sudah ada, untuk menguji cabang "masih jalan".
    fn ctx_with(sessions: Value) -> Ctx {
        let ctx = base_ctx();
        ctx.session_set(SESSIONS_KEY, sessions);
        ctx
    }

    fn run(ctx: Ctx) -> Reply {
        (oktz_plugin_api::find("family100").unwrap().handler)(ctx)
    }

    /// Dimiliki (bukan `&str`) supaya bisa dipakai langsung pada balasan
    /// temporer tanpa pinjaman yang lebih panjang dari statement.
    fn text_of(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("family100").unwrap();
        assert_eq!(p.meta.name, "family100");
        assert_eq!(p.meta.alias, ["f100"]);
        assert_eq!(p.meta.category, "game");
        assert_eq!(
            p.meta.description,
            "Survey says! Tebak jawaban teratas survei"
        );
        assert_eq!(p.meta.usage, ".family100");
        assert_eq!(p.meta.example, ".family100");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
        assert!(!p.meta.is_admin && !p.meta.is_bot_admin);
    }

    #[test]
    fn the_alias_f100_resolves_to_this_plugin() {
        assert_eq!(
            oktz_plugin_api::find("f100").unwrap().meta.name,
            "family100"
        );
    }

    #[test]
    fn the_bank_is_the_original_json_not_a_trimmed_copy() {
        assert!(bank().len() > 1000, "bank soal terpotong: {}", bank().len());
        let q = &bank()[0];
        assert_eq!(q.soal, "Hotel Bintang 5 Jakarta");
        assert_eq!(
            q.jawaban,
            [
                "sheraton",
                "the westin",
                "kempinski",
                "the langham",
                "borobudur",
                "hilton"
            ]
        );
    }

    #[test]
    fn a_fresh_game_posts_the_question_and_writes_a_120_second_session() {
        let ctx = base_ctx();
        let before = crate::common::now_ms();
        let r = run(ctx.clone());
        let t = text_of(&r);
        assert!(t.starts_with("Waktunya main *FAMILY 100*"), "{t}");
        assert!(t.contains("120 detik"), "{t}");
        assert!(
            r.mutations().is_empty(),
            "state ditulis lewat session, bukan mutasi"
        );

        let s = ctx.session_get(SESSIONS_KEY).unwrap()[CHAT].clone();
        assert_eq!(s["gameType"], serde_json::json!("family100"));
        assert_eq!(s["chatId"], serde_json::json!(CHAT));
        assert_eq!(s["answered"], serde_json::json!([]));
        assert_eq!(s["answeredBy"], serde_json::json!({}));
        let end = s["endTime"].as_i64().unwrap();
        let start = s["startTime"].as_i64().unwrap();
        assert!(
            start >= before && start <= crate::common::now_ms(),
            "startTime {start} bukan sekarang"
        );
        assert_eq!(end - start, SESSION_TIMEOUT_MS, "deadline harus 120 detik");
        // Kartu yang dikirim harus soal yang benar-benar tersimpan.
        assert!(
            t.contains(s["question"]["soal"].as_str().unwrap()),
            "kartu dan sesi harus soal yang sama: {t}"
        );
        assert_eq!(
            s["question"]["jawaban"].as_array().unwrap().len(),
            t.matches("❓ ???").count(),
            "jumlah baris kartu harus sama dengan jumlah jawaban"
        );
    }

    #[test]
    fn a_second_call_shows_the_running_session_instead_of_a_new_question() {
        let now = crate::common::now_ms();
        let t = text_of(&run(ctx_with(serde_json::json!({
            CHAT: {
                "gameType": "family100",
                "question": { "soal": "Hotel Bintang 5 Jakarta", "jawaban": ["sheraton", "hilton"] },
                "answered": ["sheraton"],
                "answeredBy": { "sheraton": SENDER },
                "startTime": now,
                "endTime": now + SESSION_TIMEOUT_MS,
            }
        }))));
        assert!(t.starts_with("Wah, sesi Family 100 masih jalan"), "{t}");
        assert!(t.contains("Hotel Bintang 5 Jakarta"), "{t}");
        assert!(t.contains("Terjawab: *1 dari 2*"), "{t}");
        assert!(t.contains("1. ✅ sheraton"), "{t}");
        assert!(t.contains("2. ❓ ???"), "{t}");
        assert!(
            t.contains("Sisa waktu: *1m 59s*") || t.contains("Sisa waktu: *2m 0s*"),
            "sisa waktu harus dihitung dari deadline: {t}"
        );
    }

    #[test]
    fn a_session_from_another_game_does_not_block_a_new_family100() {
        let now = crate::common::now_ms();
        let t = text_of(&run(ctx_with(serde_json::json!({
            CHAT: {
                "gameType": "tictactoe",
                "question": { "soal": "?", "jawaban": [] },
                "startTime": now,
                "endTime": now + SESSION_TIMEOUT_MS,
            }
        }))));
        assert!(t.starts_with("Waktunya main *FAMILY 100*"), "{t}");
    }

    #[test]
    fn an_expired_session_is_replaced_by_a_new_one() {
        let now = crate::common::now_ms();
        let ctx = ctx_with(serde_json::json!({
            CHAT: {
                "gameType": "family100",
                "question": { "soal": "Sesi Lama", "jawaban": ["x"] },
                "answered": ["x"],
                "startTime": now - SESSION_TIMEOUT_MS - 1_000,
                "endTime": now - 1_000,
            }
        }));
        let __balasan = run(ctx.clone());
        let t = text_of(&__balasan);
        assert!(t.starts_with("Waktunya main *FAMILY 100*"), "{t}");
        assert!(
            !t.contains("Sesi Lama"),
            "sesi kedaluwarsa tidak boleh tampil"
        );
        assert_ne!(
            ctx.session_get(SESSIONS_KEY).unwrap()[CHAT]["question"]["soal"],
            serde_json::json!("Sesi Lama")
        );
    }

    // `createSession` (game-data.js:149) hanya membersihkan timer sesi lama lalu
    // `gameSessions.set(chatId, session)` tanpa mengecek `gameType` — sesi game
    // lain memang DITIMPA, bukan dipertahankan.
    #[test]
    fn a_running_session_of_another_game_is_replaced_by_the_new_one() {
        let now = crate::common::now_ms();
        let ctx = ctx_with(serde_json::json!({
            CHAT: {
                "gameType": "tictactoe",
                "question": { "soal": "?", "jawaban": [] },
                "startTime": now,
                "endTime": now + SESSION_TIMEOUT_MS,
            }
        }));
        let __balasan = run(ctx.clone());
        let t = text_of(&__balasan);
        assert!(t.starts_with("Waktunya main *FAMILY 100*"), "{t}");
        let s = ctx.session_get(SESSIONS_KEY).unwrap()[CHAT].clone();
        assert_eq!(
            s["gameType"],
            serde_json::json!("family100"),
            "sesi lama harus tertimpa"
        );
        assert_ne!(
            s["question"]["soal"],
            serde_json::json!("?"),
            "soal baru harus tersimpan"
        );
    }

    #[test]
    fn remaining_time_rounds_up_and_never_goes_negative() {
        assert_eq!(remaining_time(1_000, 0), 1, "999ms harus 1 detik");
        assert_eq!(remaining_time(120_000, 0), 120);
        assert_eq!(remaining_time(0, 0), 0);
        assert_eq!(
            remaining_time(0, 5_000),
            0,
            "kedaluwarsa tidak boleh negatif"
        );
    }

    #[test]
    fn the_remaining_time_labels_follow_the_js_bands() {
        assert_eq!(format_remaining_time(0), "0 detik");
        assert_eq!(format_remaining_time(1), "1 detik");
        assert_eq!(format_remaining_time(59), "59 detik");
        assert_eq!(format_remaining_time(60), "1m 0s");
        assert_eq!(format_remaining_time(120), "2m 0s");
        assert_eq!(format_remaining_time(125), "2m 5s");
    }

    #[test]
    fn the_question_card_lists_one_row_per_answer() {
        let t = question_card(&Question {
            soal: "Uji".into(),
            jawaban: vec!["a".into(), "b".into(), "c".into()],
        });
        assert!(t.contains("Total Jawaban: *3* 📝"), "{t}");
        assert!(t.contains("1. ❓ ???\n2. ❓ ???\n3. ❓ ???\n"), "{t}");
        assert_eq!(t.matches("❓ ???").count(), 3, "{t}");
    }

    /// `Ctx` untuk answer handler: user punya exp/koin awal, sesi aktif.
    fn answer_ctx(sessions: Value, text: &str) -> Ctx {
        let ctx = ctx_with(sessions);
        Ctx {
            text: text.into(),
            user_data: serde_json::json!({ "exp": 0, "koin": 0 }),
            ..ctx
        }
    }

    fn quoted_from_bot() -> oktz_plugin_api::QuotedMessage {
        oktz_plugin_api::QuotedMessage {
            id: "AAA".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: CHAT.into(),
            text: "Waktunya main FAMILY 100".into(),
            kind: oktz_plugin_api::QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: true,
            timestamp: 0,
        }
    }

    fn active_sessions() -> Value {
        let now = crate::common::now_ms();
        serde_json::json!({
            CHAT: {
                "gameType": "family100",
                "question": { "soal": "Uji", "jawaban": ["sheraton", "hilton"] },
                "answered": [],
                "answeredBy": {},
                "startTime": now,
                "endTime": now + SESSION_TIMEOUT_MS,
            }
        })
    }

    #[test]
    fn a_correct_answer_rewards_and_is_recorded() {
        let ctx = answer_ctx(active_sessions(), "sheraton");
        let r = family100_answer(&ctx).expect("jawaban benar ditangani");
        let t = text_of(&r);
        assert!(t.contains("Benerrr banget!"), "{t}");
        assert!(t.contains("1. ✅ sheraton"), "{t}");
        assert!(t.contains("2. ❓ ???"), "{t}");
        assert_eq!(r.mutations().len(), 1, "hadiah ditulis sebagai mutasi");
        let s = ctx.session_get(SESSIONS_KEY).unwrap()[CHAT].clone();
        assert_eq!(s["answered"], serde_json::json!(["sheraton"]));
        assert_eq!(s["answeredBy"]["sheraton"], serde_json::json!(SENDER));
    }

    #[test]
    fn a_wrong_answer_without_quoting_is_left_to_other_handlers() {
        let ctx = answer_ctx(active_sessions(), "kucing");
        assert!(family100_answer(&ctx).is_none());
        assert_eq!(
            ctx.session_get(SESSIONS_KEY).unwrap()[CHAT]["answered"],
            serde_json::json!([])
        );
    }

    #[test]
    fn a_wrong_answer_quoting_the_bot_replies_tett() {
        let mut ctx = answer_ctx(active_sessions(), "kucing");
        ctx.quoted = Some(quoted_from_bot());
        let t = text_of(&family100_answer(&ctx).expect("quoted salah ditangani"));
        assert!(t.contains("Tettt! ❌"), "{t}");
    }

    #[test]
    fn surrender_reveals_the_remaining_answers_and_ends_the_session() {
        let ctx = answer_ctx(active_sessions(), "nyerah");
        let t = text_of(&family100_answer(&ctx).expect("nyerah ditangani"));
        assert!(t.contains("pada nyerah"), "{t}");
        assert!(t.contains("• sheraton"), "{t}");
        assert!(t.contains("• hilton"), "{t}");
        assert!(
            ctx.session_get(SESSIONS_KEY)
                .and_then(|s| s.get(CHAT).cloned())
                .is_none()
        );
    }

    #[test]
    fn the_last_correct_answer_ends_the_session_and_mentions_winners() {
        let mut ctx = answer_ctx(active_sessions(), "hilton");
        ctx.user_data = serde_json::json!({ "exp": 0, "koin": 0 });
        // Jawaban pertama sudah tercatat; ini yang menutup sesi.
        {
            let chat = ctx.chat.clone();
            ctx.session_update(SESSIONS_KEY, |sessions| {
                sessions[&chat]["answered"] = serde_json::json!(["sheraton"]);
                sessions[&chat]["answeredBy"] = serde_json::json!({ "sheraton": SENDER });
            });
        }
        let r = family100_answer(&ctx).expect("jawaban terakhir ditangani");
        let t = text_of(&r);
        assert!(t.contains("Semua jawaban ketebak"), "{t}");
        assert!(t.contains("@628111"), "{t}");
        assert!(
            ctx.session_get(SESSIONS_KEY)
                .and_then(|s| s.get(CHAT).cloned())
                .is_none()
        );
    }

    #[test]
    fn a_correct_answer_is_matched_when_the_answer_contains_it() {
        let ctx = answer_ctx(active_sessions(), "hotel sheraton");
        assert!(
            family100_answer(&ctx).is_some(),
            "`ans.includes(user)` harus cocok"
        );
    }

    #[test]
    fn similarity_is_one_for_identical_and_low_for_unrelated() {
        assert_eq!(similarity("sheraton", "sheraton"), 1.0);
        assert!(similarity("sheraton", "xyz") < 0.5);
    }
}
