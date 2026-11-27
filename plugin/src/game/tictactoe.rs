//! Buat/join room TicTacToe di grup.
//!
//! `Keterbatasan:` versi awal menghapus room `WAITING` lewat `setTimeout(..., 300000)`.
//! Tanpa timer, room yang sudah lewat 5 menit diabaikan saat handler dipanggil lagi — efek
//! yang sama, tanpa pengusir yang berjalan sendiri. Room `PLAYING` tidak pernah dihapus
//! otomatis di versi awal, jadi tidak juga di sini.
//!
//! `Keterbatasan:` `safeReply`/`safeReact` membungkus `m.reply`/`m.react` dengan retry
//! rate-limit dan `delay()`. `Ctx` sinkron tanpa I/O, jadi retry itu hilang; isi pesannya
//! tidak berubah.

use oktz_plugin_api::{Ctx, Mutation, Reply, answer_handler, plugin};
use serde_json::{Map, Value, json};

use crate::common::{jid_number, now_ms};

/// `global.tictactoeGames` dipetakan ke session store dengan nama yang sama,
/// supaya whoever mem-port router tic-tac-toe tidak perlu tahu.
pub(crate) const GAMES_KEY: &str = "tictactoeGames";

/// Awalan `roomId` dan penentu
/// "masih dalam game".
pub(crate) const ROOM_PREFIX: &str = "ttt_";

/// `setTimeout(..., 300000)` — 5 menit.
pub(crate) const ROOM_TTL_MS: i64 = 300_000;

/// Sender sudah punya room.
pub(crate) const BUSY: &str = "❌ Kamu masih dalam game!\n\n> \
    Selesaikan game kamu atau ketik *nyerah* untuk menyerah.";

/// `boardSymbols` — urutan tidak berpengaruh karena
/// dicari per kunci.
const BOARD_SYMBOLS: [(&str, &str); 11] = [
    ("X", "❌"),
    ("O", "⭕"),
    ("1", "1️⃣"),
    ("2", "2️⃣"),
    ("3", "3️⃣"),
    ("4", "4️⃣"),
    ("5", "5️⃣"),
    ("6", "6️⃣"),
    ("7", "7️⃣"),
    ("8", "8️⃣"),
    ("9", "9️⃣"),
];

/// `boardSymbols[String(cell)] || cell`.
pub(crate) fn board_symbol(cell: &str) -> String {
    BOARD_SYMBOLS
        .iter()
        .find(|(key, _)| *key == cell)
        .map_or_else(|| cell.to_string(), |(_, sym)| (*sym).to_string())
}

/// `TicTacToe.render(0, 0)` — `_x` dan `_o` masih nol di room yang baru dibuat
/// atau baru digabung, jadi semua sel kosong dan bernomor 1-9.
fn fresh_cells() -> [String; 9] {
    std::array::from_fn(|i| (i + 1).to_string())
}

/// `renderBoard(arr)`.
pub(crate) fn render_board(cells: &[String; 9]) -> String {
    let c: Vec<String> = cells.iter().map(|s| board_symbol(s)).collect();
    format!(
        "┌───┬───┬───┐\n\
         │ {0} │ {1} │ {2} │\n\
         ├───┼───┼───┤\n\
         │ {3} │ {4} │ {5} │\n\
         ├───┼───┼───┤\n\
         │ {6} │ {7} │ {8} │\n\
         └───┴───┴───┘",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], c[8]
    )
}

pub(crate) fn fresh_board() -> String {
    render_board(&fresh_cells())
}

/// Room `WAITING` yang lewat TTL — diperlakukan sama seperti sudah dihapus
/// `setTimeout`. Berlaku untuk SEMUA pencarian room, bukan cuma yang mencari
/// room kosong: di versi awal room basi sudah hilang dari map, jadi tidak lagi
/// menahan pemanggil di cabang "masih dalam game" maupun bisa di-join.
fn is_expired(room: &Value, now: i64) -> bool {
    room.get("state").and_then(Value::as_str) == Some("WAITING")
        && now - room.get("createdAt").and_then(Value::as_i64).unwrap_or(0) > ROOM_TTL_MS
}

/// `room.id.startsWith("ttt_") && [playerX, playerO].filter(Boolean).includes(sender)`.
///
/// Room `PLAYING` tidak pernah dihapus `setTimeout`, jadi tidak ada cek TTL
/// di sini — hanya `WAITING` basi yang dianggap sudah hilang.
fn is_active_for(room: &Value, sender: &str, now: i64) -> bool {
    room.get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| id.starts_with(ROOM_PREFIX))
        && [room.get("playerX"), room.get("playerO")]
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|p| p == sender)
        && !is_expired(room, now)
}

/// `r.state === "WAITING" && r.chat === chat && (roomName ? r.name === roomName: true)`.
///
/// Tidak ada cek awalan `ttt_` di sini — versi awal juga tidak punya, jadi room
/// waiting milik plugin lain di chat yang sama tetap bisa di-join.
fn is_waiting(room: &Value, chat: &str, room_name: &str, now: i64) -> bool {
    room.get("state").and_then(Value::as_str) == Some("WAITING")
        && room.get("chat").and_then(Value::as_str) == Some(chat)
        && (room_name.is_empty() || room.get("name").and_then(Value::as_str) == Some(room_name))
        && !is_expired(room, now)
}

/// `const roomId = "ttt_" + Date.now()`.
///
/// `Date.now()` Resolution milidetik: di versi awal dua room tidak mungkin dibuat
/// milidetik yang sama (setiap panggilan lewat jaringan), tapi handler di sini
/// sinkon dan dua perintah berurutan bisa jatuh di milideki yang sama — key
/// yang sama menimpa room sebelumnya. Suffix hanya dipakai kalau key-nya sudah
/// diambil supaya room pertama tidak hilang.
fn new_room_id(rooms: &Map<String, Value>, now: i64) -> String {
    let base = format!("{ROOM_PREFIX}{now}");
    let mut id = base.clone();
    let mut suffix = 1;
    while rooms.contains_key(&id) {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    id
}

/// Keputusan handler terhadap room yang ada. Tidak ada yang dikirim ke user
/// pada poin ini — teksnya dirangkai pemanggil.
enum Decision {
    /// Pengirim masih punya room — tidak ada perubahan state.
    Busy,
    /// Pengirim masuk ke room yang menunggu.
    Joined { player_x: String, player_o: String },
    /// Room baru dibuat untuk pengirim.
    Created,
}

fn decide(
    rooms: &mut Map<String, Value>,
    room_name: &str,
    sender: &str,
    chat: &str,
    now: i64,
) -> Decision {
    if rooms.values().any(|r| is_active_for(r, sender, now)) {
        return Decision::Busy;
    }

    let waiting = rooms
        .iter()
        .find(|(_, r)| is_waiting(r, chat, room_name, now))
        .map(|(id, _)| id.clone());

    match waiting {
        Some(id) => {
            let room = rooms
                .get_mut(&id)
                .expect("key baru dibaca dari map yang sama");
            if let Some(obj) = room.as_object_mut() {
                obj.insert("playerO".into(), serde_json::json!(sender));
                obj.insert("state".into(), serde_json::json!("PLAYING"));
                obj.insert("cells".into(), serde_json::json!(fresh_cells()));
                obj.insert("currentTurn".into(), serde_json::json!(sender));
                obj.insert("turns".into(), serde_json::json!(0));
            }
            Decision::Joined {
                player_x: room
                    .get("playerX")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                player_o: sender.to_string(),
            }
        }
        None => {
            let room_id = new_room_id(rooms, now);
            rooms.insert(
                room_id.clone(),
                serde_json::json!({
                    "id": room_id,
                    "chat": chat,
                    "name": if room_name.is_empty() { Value::Null } else { serde_json::json!(room_name) },
                    "playerX": sender,
                    "playerO": Value::Null,
                    "state": "WAITING",
                    "createdAt": now,
                }),
            );
            Decision::Created
        }
    }
}

#[plugin(
    name = "tictactoe",
    alias = ["ttt", "xo"],
    category = "game",
    description = "Main TicTacToe dengan player lain",
    usage = ".tictactoe [room name] atau .ttt",
    example = ".tictactoe",
    cooldown = 5,
    energi = 0,
    is_group = true,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn tictactoe(ctx: Ctx) -> Reply {
    let room_name = ctx.args.join(" ").trim().to_string();
    let now = now_ms();

    let mut decided = None;
    ctx.session_update(GAMES_KEY, |slot| {
        if !slot.is_object() {
            *slot = serde_json::json!({});
        }
        let rooms = slot.as_object_mut().expect("baru diganti objek");
        decided = Some(decide(rooms, &room_name, &ctx.sender, &ctx.chat, now));
    });

    match decided.expect("decide selalu menghasilkan keputusan") {
        Decision::Busy => Reply::text(BUSY),
        Decision::Created => Reply::text(format!(
            "🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*\n\n\
             Room dibuat! Menunggu partner...\n\n\
             > Ketik `.tictactoe{}` untuk join\n\
             > Room akan expire dalam 5 menit",
            if room_name.is_empty() {
                String::new()
            } else {
                format!(" {room_name}")
            }
        ))
        .with_reaction("🕕"),
        Decision::Joined { player_x, player_o } => Reply::Text {
            // `game._currentTurn` masih `false` di room yang baru digabung,
            // jadi `currentTurn` = `playerX`.
            text: format!(
                "🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*\n\n\
                 Partner ditemukan!\n\n\
                 ❌ @{}\n⭕ @{}\n\n{}\n\n\
                 > Giliran: @{}\n\
                 > Reply pesan ini dengan angka 1-9\n\
                 > Ketik *nyerah* untuk menyerah",
                jid_number(&player_x),
                jid_number(&player_o),
                fresh_board(),
                jid_number(&player_x),
            ),
            mentions: vec![player_x, player_o],
        }
        .with_reaction("🎮"),
    }
}

const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

fn room_of(rooms: &Map<String, Value>, sender: &str, chat: &str) -> Option<(String, Value)> {
    rooms
        .iter()
        .find(|(_, r)| {
            r.get("state").and_then(Value::as_str) == Some("PLAYING")
                && r.get("chat").and_then(Value::as_str) == Some(chat)
                && ["playerX", "playerO"]
                    .iter()
                    .filter_map(|k| r.get(*k).and_then(Value::as_str))
                    .any(|p| p == sender)
        })
        .map(|(id, r)| (id.clone(), r.clone()))
}

fn winner_of(cells: &[Value]) -> Option<&'static str> {
    for line in WIN_LINES {
        let sym = cells[line[0]].as_str()?;
        if sym != "X" && sym != "O" {
            continue;
        }
        if line.iter().all(|i| cells[*i].as_str() == Some(sym)) {
            return Some(if sym == "X" { "X" } else { "O" });
        }
    }
    None
}

fn koin_of(ctx: &Ctx, jid: &str) -> i64 {
    ctx.roster
        .participants
        .iter()
        .find(|p| p.jid == jid)
        .and_then(|p| p.data.get("koin"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

/// `answerHandler` — balasan angka 1-9
/// `nyerah` untuk room yang sedang berjalan.
#[answer_handler]
pub fn tictactoe_answer(ctx: &Ctx) -> Option<Reply> {
    let text = ctx.text.trim().to_lowercase();
    if text.is_empty() {
        return None;
    }
    let (room_id, room) = {
        let rooms = ctx.session_get(GAMES_KEY)?;
        room_of(rooms.as_object()?, &ctx.sender, &ctx.chat)?
    };

    let player_x = room.get("playerX").and_then(Value::as_str)?.to_string();
    let player_o = room.get("playerO").and_then(Value::as_str)?.to_string();
    let remove_room = {
        let rooms = ctx.sessions.clone();
        let id = room_id.clone();
        move || {
            if let Some(map) = rooms
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get_mut(GAMES_KEY)
                .and_then(Value::as_object_mut)
            {
                map.remove(&id);
            }
        }
    };

    if matches!(text.as_str(), "nyerah" | "surrender" | "give up") {
        let winner = if ctx.sender == player_x {
            player_o.clone()
        } else {
            player_x.clone()
        };
        remove_room();
        let text = format!(
            "🏳️ *MENYERAH!*\n\n@{} menyerah!\n@{} menang! +Rp 500",
            jid_number(&ctx.sender),
            jid_number(&winner)
        );
        return Some(
            Reply::Text {
                text,
                mentions: vec![winner.clone(), ctx.sender.clone()],
            }
            .mutating(vec![Mutation::SetUser {
                jid: winner.clone(),
                data: json!({ "koin": koin_of(ctx, &winner) + 500 }),
            }])
            .with_reaction("🏳️"),
        );
    }

    let mv = crate::common::parse_int(&text)?;
    if !(1..=9).contains(&mv) {
        return None;
    }

    let current_turn = room
        .get("currentTurn")
        .and_then(Value::as_str)
        .unwrap_or(&player_x);
    if current_turn != ctx.sender {
        return Some(Reply::text("❌ Bukan giliranmu!"));
    }

    let mut cells: Vec<Value> = room
        .get("cells")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| fresh_cells().into_iter().map(Value::String).collect());
    let idx = (mv - 1) as usize;
    if matches!(cells[idx].as_str(), Some("X" | "O")) {
        return Some(Reply::text("❌ Posisi sudah terisi!"));
    }

    let symbol = if ctx.sender == player_x { "X" } else { "O" };
    cells[idx] = json!(symbol);
    let board_cells: [String; 9] = std::array::from_fn(|i| {
        cells[i]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| (i + 1).to_string())
    });
    let board = render_board(&board_cells);

    let winner = winner_of(&cells);
    let full = cells.iter().all(|c| matches!(c.as_str(), Some("X" | "O")));

    if let Some(sym) = winner {
        let winner_jid = if sym == "X" { &player_x } else { &player_o };
        let loser = if winner_jid == &player_x {
            &player_o
        } else {
            &player_x
        };
        remove_room();
        let text = format!(
            "🎉 *GAME OVER!*\n\n{board}\n\n🏆 @{} menang! +Rp 1.000",
            jid_number(winner_jid)
        );
        return Some(
            Reply::Text {
                text,
                mentions: vec![winner_jid.clone(), loser.clone()],
            }
            .mutating(vec![Mutation::SetUser {
                jid: winner_jid.clone(),
                data: json!({ "koin": koin_of(ctx, winner_jid) + 1000 }),
            }])
            .with_reaction("🎉"),
        );
    }

    if full {
        remove_room();
        return Some(
            Reply::Text {
                text: format!("🤝 *SERI!*\n\n{board}\n\n> Tidak ada pemenang!"),
                mentions: vec![player_x, player_o],
            }
            .with_reaction("🤝"),
        );
    }

    let next = if ctx.sender == player_x {
        player_o.clone()
    } else {
        player_x.clone()
    };
    let id = room_id.clone();
    ctx.session_update(GAMES_KEY, |slot| {
        if let Some(room) = slot.as_object_mut().and_then(|m| m.get_mut(&id)) {
            room["cells"] = Value::Array(cells.clone());
            room["currentTurn"] = json!(next);
            room["turns"] = json!(room.get("turns").and_then(Value::as_i64).unwrap_or(0) + 1);
        }
    });

    Some(
        Reply::Text {
            text: format!(
                "🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*\n\n{board}\n\n> Giliran: @{}",
                jid_number(&next)
            ),
            mentions: vec![next],
        }
        .with_reaction("🎮"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use oktz_plugin_api::{SessionStore, new_session_store};

    const CHAT: &str = "120363@g.us";
    const SENDER: &str = "628111@s.whatsapp.net";
    const OTHER: &str = "628999@s.whatsapp.net";

    fn ctx(sender: &str, args: &str, sessions: &SessionStore) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "tictactoe".into(),
            text: args.into(),
            args: args.split_whitespace().map(str::to_string).collect(),
            sender: sender.into(),
            chat: CHAT.into(),
            is_group: true,
            // `Arc` di-clone supaya test boleh inspect store setelah handler.
            sessions: sessions.clone(),
            ..Default::default()
        }
    }

    fn run(sender: &str, args: &str, sessions: &SessionStore) -> Reply {
        let c = ctx(sender, args, sessions);
        (oktz_plugin_api::find("tictactoe").unwrap().handler)(c)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    fn reaction_of(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    fn rooms(sessions: &SessionStore) -> Vec<(String, Value)> {
        sessions
            .lock()
            .unwrap()
            .get(GAMES_KEY)
            .and_then(Value::as_object)
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    fn waiting_room(id: &str, player_x: &str, name: Option<&str>, created: i64) -> (String, Value) {
        (
            id.into(),
            serde_json::json!({
                "id": id,
                "chat": CHAT,
                "name": name.map_or(Value::Null, |n| serde_json::json!(n)),
                "playerX": player_x,
                "playerO": Value::Null,
                "state": "WAITING",
                "createdAt": created,
            }),
        )
    }

    fn seed(sessions: &SessionStore, room: (String, Value)) {
        let mut map = Map::new();
        map.insert(room.0, room.1);
        sessions
            .lock()
            .unwrap()
            .insert(GAMES_KEY.into(), Value::Object(map));
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("tictactoe").unwrap();
        assert_eq!(p.meta.name, "tictactoe");
        assert_eq!(p.meta.alias, ["ttt", "xo"]);
        assert_eq!(p.meta.category, "game");
        assert_eq!(p.meta.description, "Main TicTacToe dengan player lain");
        assert_eq!(p.meta.usage, ".tictactoe [room name] atau .ttt");
        assert_eq!(p.meta.example, ".tictactoe");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_admin);
        assert!(!p.meta.is_bot_admin);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["tictactoe", "ttt", "xo"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "tictactoe");
        }
    }

    #[test]
    fn the_first_command_creates_a_waiting_room() {
        let s = new_session_store();
        let r = run(SENDER, "", &s);
        assert_eq!(
            text_of(&r),
            "🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*\n\n\
             Room dibuat! Menunggu partner...\n\n\
             > Ketik `.tictactoe` untuk join\n\
             > Room akan expire dalam 5 menit"
        );
        assert_eq!(reaction_of(&r), Some("🕕"));

        let list = rooms(&s);
        assert_eq!(list.len(), 1);
        let (id, room) = &list[0];
        assert!(id.starts_with(ROOM_PREFIX), "{id}");
        assert_eq!(room["id"], serde_json::json!(id.as_str()));
        assert_eq!(room["chat"], serde_json::json!(CHAT));
        assert_eq!(room["name"], Value::Null, "room tanpa nama disimpan null");
        assert_eq!(room["playerX"], serde_json::json!(SENDER));
        assert_eq!(room["playerO"], Value::Null);
        assert_eq!(room["state"], serde_json::json!("WAITING"));
        assert!(room["createdAt"].is_i64());
    }

    #[test]
    fn a_named_room_echoes_the_name_in_the_join_hint() {
        let s = new_session_store();
        let r = run(SENDER, " opardu   ultimate ", &s);
        assert!(
            text_of(&r).contains("> Ketik `.tictactoe opardu ultimate` untuk join"),
            "{}",
            text_of(&r)
        );
        assert_eq!(rooms(&s)[0].1["name"], serde_json::json!("opardu ultimate"));
    }

    #[test]
    fn the_second_player_joins_the_waiting_room_of_the_same_chat() {
        let s = new_session_store();
        run(SENDER, "ruang 1", &s);
        let r = run(OTHER, "ruang 1", &s);

        assert!(
            text_of(&r).starts_with("🎮 *ᴛɪᴄ ᴛᴀᴄ ᴛᴏᴇ*\n\nPartner ditemukan!\n\n"),
            "{}",
            text_of(&r)
        );
        assert_eq!(reaction_of(&r), Some("🎮"));
        // `safeReact` lalu `safeReply` dipetakan ke `Reply::Reaction`
        // membungkus balasan teksnya — mention ada di balasan dalam.
        match r.without_mutations() {
            Reply::Reaction { reply, .. } => match reply.as_ref() {
                Reply::Text { mentions, .. } => {
                    assert_eq!(mentions, &[SENDER.to_string(), OTHER.to_string()])
                }
                other => panic!("isi Reaction harus Text, dapat {other:?}"),
            },
            other => panic!("harus Reaction, dapat {other:?}"),
        }

        let list = rooms(&s);
        assert_eq!(list.len(), 1, "tidak boleh membuat room kedua");
        assert_eq!(list[0].1["state"], serde_json::json!("PLAYING"));
        assert_eq!(list[0].1["playerX"], serde_json::json!(SENDER));
        assert_eq!(list[0].1["playerO"], serde_json::json!(OTHER));
    }

    #[test]
    fn the_joined_message_shows_the_empty_board_and_the_first_players_turn() {
        let s = new_session_store();
        run(SENDER, "", &s);
        let r = run(OTHER, "", &s);
        assert!(
            text_of(&r).contains("❌ @628111\n⭕ @628999"),
            "{}",
            text_of(&r)
        );
        assert!(
            text_of(&r).contains("> Giliran: @628111"),
            "giliran pertama = playerX: {}",
            text_of(&r)
        );
        assert!(text_of(&r).contains(&fresh_board()), "{}", text_of(&r));
    }

    #[test]
    fn a_player_who_is_already_in_a_game_is_turned_away() {
        let s = new_session_store();
        run(SENDER, "", &s);
        let before = rooms(&s);
        let r = run(SENDER, "nama lain", &s);
        assert_eq!(text_of(&r), BUSY);
        assert_eq!(reaction_of(&r), None, "JS tidak bereaksi di cabang ini");
        assert_eq!(rooms(&s), before, "tidak boleh ada perubahan state");
    }

    #[test]
    fn the_player_o_is_also_locked_out_of_creating_a_new_room() {
        let s = new_session_store();
        run(SENDER, "", &s);
        run(OTHER, "", &s);
        let r = run(OTHER, "", &s);
        assert_eq!(text_of(&r), BUSY);
        assert_eq!(rooms(&s).len(), 1);
    }

    #[test]
    fn a_different_room_name_does_not_join_the_waiting_one() {
        let s = new_session_store();
        run(SENDER, "ruang 1", &s);
        run(OTHER, "ruang 2", &s);
        let list = rooms(&s);
        assert_eq!(list.len(), 2, "nama room berbeda → dua room terpisah");
        assert!(
            list.iter()
                .all(|(_, r)| r["state"] == serde_json::json!("WAITING"))
        );
        // Dua perintah berurutan bisa jatuh di milidetik yang sama; kalau key
        // room tidak dibedakan, room pertama tertimpa dan hanya tersisa satu.
        for (id, room) in &list {
            assert!(id.starts_with(ROOM_PREFIX), "{id}");
            assert_eq!(room["id"], serde_json::json!(id.as_str()));
        }
        let names: Vec<&Value> = list.iter().map(|(_, r)| &r["name"]).collect();
        assert!(names.contains(&&serde_json::json!("ruang 1")), "{names:?}");
        assert!(names.contains(&&serde_json::json!("ruang 2")), "{names:?}");
    }

    #[test]
    fn an_empty_name_matches_any_waiting_room_but_named_join_does_not() {
        let s = new_session_store();
        run(SENDER, "ruang 1", &s);
        // Tanpa nama, filter dilewati (`roomName ?...: true`).
        let r = run(OTHER, "", &s);
        assert!(
            text_of(&r).contains("Partner ditemukan!"),
            "{}",
            text_of(&r)
        );
        // Dengan nama yang tidak cocok, tidak ada yang di-join.
        let s2 = new_session_store();
        run(SENDER, "ruang 1", &s2);
        let r = run(OTHER, "ruang 9", &s2);
        assert!(text_of(&r).contains("Room dibuat!"), "{}", text_of(&r));
    }

    #[test]
    fn an_expired_waiting_room_is_treated_as_deleted() {
        let s = new_session_store();
        // `createdAt` 0 berarti sudah lewat 5 menit untuk `now_ms()` sekarang.
        seed(&s, waiting_room("ttt_1", SENDER, None, 0));
        let r = run(OTHER, "", &s);
        assert!(text_of(&r).contains("Room dibuat!"), "{}", text_of(&r));
        assert_eq!(
            rooms(&s).len(),
            2,
            "room basi tidak dihapus, room baru ditambahkan"
        );
    }

    #[test]
    fn the_owner_of_an_expired_waiting_room_is_no_longer_locked_out() {
        let s = new_session_store();
        seed(&s, waiting_room("ttt_1", SENDER, None, 0));
        let r = run(SENDER, "", &s);
        assert!(text_of(&r).contains("Room dibuat!"), "{}", text_of(&r));
    }

    #[test]
    fn a_waiting_room_just_inside_the_ttl_can_still_be_joined() {
        let s = new_session_store();
        // Buffer 30 detik (pola `7ec234b`): `createdAt` tepat `now - TTL`
        // akan gugur saat handler membaca `now_ms()` 1 ms lebih lambat,
        // karena ceknya strict `>`.
        seed(
            &s,
            waiting_room("ttt_1", SENDER, None, now_ms() - ROOM_TTL_MS + 30_000),
        );
        let r = run(OTHER, "", &s);
        assert!(
            text_of(&r).contains("Partner ditemukan!"),
            "tepat di batas: {}",
            text_of(&r)
        );
    }

    #[test]
    fn a_playing_room_never_expires_on_its_own() {
        let s = new_session_store();
        run(SENDER, "", &s);
        run(OTHER, "", &s);
        // Setelah join, `createdAt` dipaksa tua tapi state PLAYING — versi awal tidak
        // pernah menghapus room ini lewat setTimeout.
        let id = rooms(&s)[0].0.clone();
        s.lock()
            .unwrap()
            .get_mut(GAMES_KEY)
            .and_then(Value::as_object_mut)
            .and_then(|o| o.get_mut(&id))
            .and_then(Value::as_object_mut)
            .expect("room hasil join")
            .insert("createdAt".into(), serde_json::json!(0));
        let r = run(SENDER, "", &s);
        assert_eq!(text_of(&r), BUSY, "PLAYING tidak boleh kadaluarsa");
        assert_eq!(rooms(&s).len(), 1);
    }

    /// Awalan `ttt_` hanya mengecek "masih dalam game".
    /// Pencarian room yang menunggu tidak memfilter
    /// awalan sama sekali, jadi room waiting milik plugin lain di chat yang
    /// tetap di-join.
    #[test]
    fn a_waiting_room_without_the_prefix_is_still_joinable() {
        let s = new_session_store();
        s.lock().unwrap().insert(
            GAMES_KEY.into(),
            serde_json::json!({"lain": {"id": "lain", "chat": CHAT, "playerX": OTHER, "playerO": Value::Null, "state": "WAITING", "createdAt": now_ms()}}),
        );
        let r = run(SENDER, "", &s);
        assert!(
            text_of(&r).contains("Partner ditemukan!"),
            "id tanpa `ttt_` tetap boleh di-join: {}",
            text_of(&r)
        );
        assert_eq!(rooms(&s).len(), 1, "tidak boleh membuat room kedua");
        assert_eq!(rooms(&s)[0].1["playerO"], serde_json::json!(SENDER));
    }

    #[test]
    fn a_room_without_the_prefix_never_counts_as_an_active_game() {
        let s = new_session_store();
        s.lock().unwrap().insert(
            GAMES_KEY.into(),
            serde_json::json!({"lain": {"id": "lain", "chat": "lain@g.us", "playerX": SENDER, "playerO": Value::Null, "state": "WAITING", "createdAt": now_ms()}}),
        );
        let r = run(SENDER, "", &s);
        assert!(
            text_of(&r).contains("Room dibuat!"),
            "id tanpa `ttt_` bukan milik plugin ini: {}",
            text_of(&r)
        );
        assert_eq!(rooms(&s).len(), 2, "room lama milik chat lain tetap ada");
    }

    #[test]
    fn the_board_maps_every_symbol_and_passes_unknowns_through() {
        assert_eq!(board_symbol("X"), "❌");
        assert_eq!(board_symbol("O"), "⭕");
        for i in 1..=9 {
            assert_eq!(board_symbol(&i.to_string()), format!("{i}️⃣"), "sel {i}");
        }
        assert_eq!(board_symbol("10"), "10", "di luar tabel → apa adanya");
        assert_eq!(board_symbol(""), "");
    }

    #[test]
    fn the_board_frame_has_nine_cells_and_four_rules() {
        let board = fresh_board();
        assert_eq!(board.lines().count(), 7, "3 baris sel + 4 garis: {board}");
        for (i, sym) in BOARD_SYMBOLS.iter().skip(2) {
            assert!(
                board.contains(&format!("│ {sym} │")),
                "sel {i} hilang: {board}"
            );
        }
        // `│ c0 │ c1 │ c2 │` = 4 garis tegak per baris sel, 3 baris sel.
        assert_eq!(
            board.matches('│').count(),
            12,
            "4 garis tegak per baris × 3 baris"
        );
    }

    #[test]
    fn the_board_of_a_marked_up_game_uses_the_symbols_not_the_numbers() {
        let cells = ["X", "O", "1", "2", "3", "4", "5", "6", "7"].map(str::to_string);
        let board = render_board(&cells);
        assert!(board.contains("│ ❌ │ ⭕ │ 1️⃣ │"), "{board}");
        assert!(!board.contains("│ X │"), "X harus jadi ❌: {board}");
    }

    #[test]
    fn a_session_slot_holding_junk_is_replaced_by_an_empty_map() {
        let s = new_session_store();
        s.lock()
            .unwrap()
            .insert(GAMES_KEY.into(), serde_json::json!("bukan objek"));
        let r = run(SENDER, "", &s);
        assert!(text_of(&r).contains("Room dibuat!"), "{}", text_of(&r));
        assert_eq!(rooms(&s).len(), 1);
    }

    #[test]
    fn a_very_long_room_name_is_echoed_whole_without_being_truncated() {
        let s = new_session_store();
        let name = "a".repeat(500);
        let r = run(SENDER, &name, &s);
        assert!(
            text_of(&r).contains(&format!(".tictactoe {name}` untuk join")),
            "nama panjang harus utuh"
        );
        assert_eq!(rooms(&s)[0].1["name"], serde_json::json!(name));
    }

    fn answer_ctx(sender: &str, text: &str, sessions: &SessionStore) -> Ctx {
        Ctx {
            text: text.into(),
            ..ctx(sender, text, sessions)
        }
    }

    fn seeding(sessions: &SessionStore, cells: Value, turn: &str) {
        sessions.lock().unwrap().insert(
            GAMES_KEY.into(),
            serde_json::json!({
                "ttt_1": {
                    "id": "ttt_1",
                    "chat": CHAT,
                    "playerX": SENDER,
                    "playerO": OTHER,
                    "state": "PLAYING",
                    "cells": cells,
                    "currentTurn": turn,
                    "turns": 0,
                    "createdAt": now_ms(),
                }
            }),
        );
    }

    #[test]
    fn a_valid_move_marks_the_cell_and_hands_the_turn_over() {
        let s = new_session_store();
        seeding(
            &s,
            serde_json::json!(["1", "2", "3", "4", "5", "6", "7", "8", "9"]),
            SENDER,
        );
        let r = tictactoe_answer(&answer_ctx(SENDER, "1", &s)).expect("langkah ditangani");
        let t = text_of(&r);
        assert!(t.contains("│ ❌ │ 2️⃣ │ 3️⃣ │"), "{t}");
        assert!(t.contains("> Giliran: @628999"), "{t}");
        let room = &rooms(&s)[0].1;
        assert_eq!(room["cells"][0], serde_json::json!("X"));
        assert_eq!(room["currentTurn"], serde_json::json!(OTHER));
    }

    #[test]
    fn a_move_outside_your_turn_is_rejected() {
        let s = new_session_store();
        seeding(
            &s,
            serde_json::json!(["1", "2", "3", "4", "5", "6", "7", "8", "9"]),
            SENDER,
        );
        let r = tictactoe_answer(&answer_ctx(OTHER, "1", &s)).expect("ditolak");
        assert!(text_of(&r).contains("Bukan giliranmu"));
    }

    #[test]
    fn an_occupied_cell_is_rejected() {
        let s = new_session_store();
        seeding(
            &s,
            serde_json::json!(["X", "2", "3", "4", "5", "6", "7", "8", "9"]),
            SENDER,
        );
        let r = tictactoe_answer(&answer_ctx(SENDER, "1", &s)).expect("ditolak");
        assert!(text_of(&r).contains("Posisi sudah terisi"));
    }

    #[test]
    fn a_winning_move_ends_the_game_and_rewards_the_winner() {
        let s = new_session_store();
        seeding(
            &s,
            serde_json::json!(["X", "X", "3", "O", "O", "6", "7", "8", "9"]),
            SENDER,
        );
        let r = tictactoe_answer(&answer_ctx(SENDER, "3", &s)).expect("menang");
        let t = text_of(&r);
        assert!(t.contains("GAME OVER!"), "{t}");
        assert!(t.contains("@628111 menang"), "{t}");
        assert!(rooms(&s).is_empty(), "room selesai harus dibuang");
        let m = r.mutations();
        assert_eq!(m.len(), 1);
        match &m[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, SENDER);
                assert_eq!(
                    data["koin"],
                    serde_json::json!(1000),
                    "roster kosong → 0+1000"
                );
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn a_full_board_without_a_line_is_a_tie() {
        let s = new_session_store();
        seeding(
            &s,
            // X menang? tidak: barisan penuh tanpa tiga simbol sama.
            serde_json::json!(["X", "O", "X", "X", "O", "O", "O", "X", "9"]),
            SENDER,
        );
        let r = tictactoe_answer(&answer_ctx(SENDER, "9", &s)).expect("seri");
        assert!(text_of(&r).contains("SERI!"), "{}", text_of(&r));
        assert!(rooms(&s).is_empty());
    }

    #[test]
    fn surrender_ends_the_room_and_pays_the_other_player() {
        let s = new_session_store();
        seeding(
            &s,
            serde_json::json!(["1", "2", "3", "4", "5", "6", "7", "8", "9"]),
            SENDER,
        );
        let r = tictactoe_answer(&answer_ctx(SENDER, "nyerah", &s)).expect("menyerah");
        let t = text_of(&r);
        assert!(t.contains("MENYERAH!"), "{t}");
        assert!(t.contains("@628999 menang!"), "{t}");
        assert!(rooms(&s).is_empty());
        match &r.mutations()[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, OTHER);
                assert_eq!(data["koin"], serde_json::json!(500));
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn a_non_numeric_text_in_a_live_room_is_ignored() {
        let s = new_session_store();
        seeding(
            &s,
            serde_json::json!(["1", "2", "3", "4", "5", "6", "7", "8", "9"]),
            SENDER,
        );
        assert!(tictactoe_answer(&answer_ctx(SENDER, "halo", &s)).is_none());
    }
}
