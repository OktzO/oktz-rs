//! Lobi dungeon interaktif.

use oktz_plugin_api::{Ctx, Mutation, Reply, answer_handler, plugin};
use serde_json::{Value, json};

/// `SESSION_TIMEOUT` — 5 menit.
pub(crate) const SESSION_TIMEOUT_MS: i64 = 5 * 60 * 1000;

/// `Date.now() - session.time > SESSION_TIMEOUT` — strict, jadi sesi
/// umurnya persis 5 menit masih hidup. Dipisah dari handler karena batasnya
/// hanya bisa dikunci persis kalau `now` dikontrol test; lewat handler,
/// `now_ms()` di dalam selalu beberapa ms lebih besar dari yang dibaca test.
pub(crate) fn session_still_alive(now: i64, at: i64) -> bool {
    now - at <= SESSION_TIMEOUT_MS
}

/// `DUNGEONS` — id, nama, level minimal. Monster/reward tidak dipakai lobi.
pub(crate) const DUNGEONS: [(i64, &str, i64); 10] = [
    (1, "🌲 Hutan Gelap", 1),
    (2, "🍄 Rawa Beracun", 5),
    (3, "🏰 Kastil Tua", 10),
    (4, "🏜️ Padang Pasir Kematian", 15),
    (5, "🌋 Gunung Api", 20),
    (6, "🧊 Gua Es Abadi", 25),
    (7, "☁️ Reruntuhan Langit", 30),
    (8, "🌊 Lautan Bayangan", 35),
    (9, "🕳️ Jurang Ketiadaan", 40),
    (10, "👹 Neraka Terdalam", 50),
];

pub(crate) const SESSION_FIELD: &str = "rpg.dungeon_session";

/// `if (!user.rpg.X) user.rpg.X = default` — `0` dan `null` ikut memakai default.
fn default_field(rpg: Option<&Value>, key: &str, default: i64) -> i64 {
    let v = rpg.and_then(|r| r.get(key));
    if crate::common::js_truthy(v) {
        v.and_then(Value::as_i64).unwrap_or(default)
    } else {
        default
    }
}

/// Balasan blokir selama ada sesi yang belum kedaluwarsa.
pub(crate) const ACTIVE: &str = "⚔️ *SESI DUNGEON MASIH AKTIF*\n\n\
     Kamu sedang berada di pertengahan eksplorasi!\n\
     > Balas chat terakhir bot untuk membatalkan (ketik `batal`) atau melanjutkan aksi (ketik `serang` / `lari`).";

/// Cabang `available.length === 0`. Tidak terjangkau lewat handler: `levelReq`
/// paling rendah adalah 1 dan `user.level || 1` minimal 1. Tetap dibawa agar
/// ambang level tidak bisa diam-diam berubah tanpa guard.
pub(crate) fn too_low(user_level: i64) -> String {
    format!(
        "❌ *LEVEL TERLALU RENDAH*\n\n> Level kamu saat ini adalah *{user_level}*. \
         Kamu butuh minimal level *1* untuk memasuki dungeon paling mudah."
    )
}

pub(crate) fn lobby(user_level: i64, stamina: i64) -> String {
    let mut txt = format!(
        "🏰 *LOBI DUNGEON*\n\n📊 *Statistik Kamu:*\n> Level: *{user_level}*\n> Stamina: *{stamina}/100*\n\n\
         Pilih lokasi yang ingin kamu jelajahi:\n\n"
    );
    for (id, name, req) in DUNGEONS {
        if user_level >= req {
            txt.push_str(&format!("🔓 *{id}.* {name} (Lv {req}+)\n"));
        } else {
            txt.push_str(&format!("> 🔒 *{id}.* {name} (Butuh Lv {req})\n"));
        }
    }
    txt.push_str(
        "\n> 💡 Balas pesan ini dengan *angka* lokasi yang 🔓 (contoh: `1`) atau ketik `batal` untuk keluar.",
    );
    txt
}

#[plugin(
    name = "dungeon",
    alias = ["dg", "labirin"],
    category = "game",
    description = "Jelajahi dungeon dan lawan monster secara interaktif",
    usage = ".dungeon",
    example = ".dungeon",
    cooldown = 60,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn dungeon(ctx: Ctx) -> Reply {
    // `user.level || 1` — `0`, `null`, dan `NaN` jadi 1; level negatif
    // TIDAK, karena angka negatif truthy di versi awal dan harus jatuh ke guard level.
    let raw_level = crate::common::int_field(&ctx, "level");
    let user_level = if raw_level == 0 { 1 } else { raw_level };
    let rpg = ctx.user_field("rpg");

    // `if (!user.rpg.X) user.rpg.X = 10/100` — dievaluasi ulang di bawah saat
    // menulis, karena versi awal menyimpan objeknya in-place lalu `db.save()`.
    let stamina = default_field(rpg, "stamina", 100);

    if let Some(session) = ctx.user_field(SESSION_FIELD)
        && !session.is_null()
    {
        let at = session.get("time").and_then(Value::as_i64).unwrap_or(0);
        if session_still_alive(crate::common::now_ms(), at) {
            return Reply::text(ACTIVE);
        }
        // `delete user.rpg.dungeon_session` lalu jatuh ke lobby — penulisan
        // terjadi di bawah bersama sesi barunya.
    }

    if !DUNGEONS.iter().any(|(_, _, req)| user_level >= *req) {
        return Reply::text(too_low(user_level));
    }

    let mut block = serde_json::Map::new();
    for (key, default) in [
        ("attack", 10),
        ("health", 100),
        ("maxHealth", 100),
        ("stamina", 100),
        ("maxStamina", 100),
    ] {
        block.insert(
            key.into(),
            serde_json::json!(default_field(rpg, key, default)),
        );
    }
    block.insert(
        "dungeon_session".into(),
        serde_json::json!({ "stage": "lobi", "time": crate::common::now_ms() }),
    );

    let mut data = serde_json::Map::new();
    data.insert("rpg".into(), Value::Object(block));
    if !crate::common::js_truthy(ctx.user_field("inventory")) {
        data.insert("inventory".into(), Value::Object(Default::default()));
    }

    Reply::text(lobby(user_level, stamina)).with_mutation(Mutation::SetUser {
        jid: ctx.sender.clone(),
        data: Value::Object(data),
    })
}

/// Monster per dungeon, urut `DUNGEONS`.
const MONSTERS: [&[&str]; 10] = [
    &[
        "Goblin Liar",
        "Slime Raksasa",
        "Serigala Malam",
        "Bandit Hutan",
    ],
    &[
        "Kodok Mutan",
        "Pohon Berjalan",
        "Laba-laba Racun",
        "Viper Rawa",
    ],
    &[
        "Prajurit Tengkorak",
        "Zombie Kelaparan",
        "Hantu Penasaran",
        "Gargoyle Batu",
    ],
    &[
        "Kalajengking Raksasa",
        "Mummy Bangkit",
        "Worm Gurun",
        "Jin Jahat",
    ],
    &["Elemental Api", "Golem Magma", "Naga Kecil", "Hellhound"],
    &["Golem Es", "Raksasa Frost", "Yeti Ganas", "Serigala Salju"],
    &[
        "Harpy Petir",
        "Griffin Liar",
        "Valkyrie Jatuh",
        "Golem Angin",
    ],
    &[
        "Kraken Bayi",
        "Siren Pemikat",
        "Hiu Hantu",
        "Leviathan Merah",
    ],
    &[
        "Malaikat Kematian",
        "Void Walker",
        "Shadow Fiend",
        "Behemoth",
    ],
    &[
        "Iblis Merah",
        "Succubus Mematikan",
        "Cerberus",
        "Raja Iblis",
    ],
];

/// `(minReward, maxReward, dropChance)` per dungeon.
const REWARDS: [(i64, i64, i64); 10] = [
    (100, 300, 40),
    (250, 500, 45),
    (400, 800, 50),
    (600, 1200, 55),
    (900, 1700, 60),
    (1300, 2400, 65),
    (1800, 3300, 70),
    (2500, 4500, 75),
    (3500, 6000, 80),
    (5000, 10000, 90),
];

/// `LOOT_TABLE`.
const LOOT: [(&str, &str, i64, i64, i64); 7] = [
    ("iron", "⛏️", 40, 1, 5),
    ("gold", "🪙", 20, 1, 3),
    ("diamond", "💎", 5, 1, 2),
    ("potion", "🧪", 30, 1, 3),
    ("herb", "🌿", 25, 2, 6),
    ("leather", "👞", 35, 2, 5),
    ("mysterybox", "📦", 3, 1, 1),
];

struct Data {
    nama: &'static str,
    level: i64,
    monsters: &'static [&'static str],
    min_reward: i64,
    max_reward: i64,
    drop_chance: i64,
}

fn data(id: i64) -> Option<Data> {
    let idx = DUNGEONS.iter().position(|(i, _, _)| *i == id)?;
    let (_, nama, level) = DUNGEONS[idx];
    let (min_reward, max_reward, drop_chance) = REWARDS[idx];
    Some(Data {
        nama,
        level,
        monsters: MONSTERS[idx],
        min_reward,
        max_reward,
        drop_chance,
    })
}

fn clear_session(ctx: &Ctx) -> Mutation {
    Mutation::SetUser {
        jid: ctx.sender.clone(),
        data: json!({ "rpg": { "dungeon_session": null } }),
    }
}

fn exp_after(ctx: &Ctx, gain: i64) -> i64 {
    let old = ctx.user_field("exp").and_then(Value::as_i64).unwrap_or(0);
    if old == -1 {
        -1
    } else {
        old.saturating_add(gain).clamp(0, 9_000_000_000)
    }
}

/// `dungeonAnswerHandler` — balasan lokasi atau aksi
/// tempur, bukan command.
#[answer_handler]
pub fn dungeon_answer(ctx: &Ctx) -> Option<Reply> {
    let text = ctx.text.trim().to_lowercase();
    if text.is_empty() {
        return None;
    }

    let sesi = ctx
        .user_field(SESSION_FIELD)
        .filter(|v| !v.is_null())?
        .clone();
    let now = crate::common::now_ms();
    let at = sesi.get("time").and_then(Value::as_i64).unwrap_or(0);

    if !session_still_alive(now, at) {
        return Some(
            Reply::text(
                "⏰ *SESI DUNGEON KEDALUWARSA*\n\n> Sesi dungeon kamu sudah hangus \
                 karena tidak aktif selama 5 menit.",
            )
            .with_mutation(clear_session(ctx)),
        );
    }

    let raw_level = crate::common::int_field(ctx, "level");
    let level = if raw_level == 0 { 1 } else { raw_level };

    if matches!(text.as_str(), "batal" | "cancel" | "keluar") {
        return Some(
            Reply::text("🚪 Kamu berhasil keluar dari Lobi Dungeon dengan selamat.")
                .with_mutation(clear_session(ctx)),
        );
    }

    match sesi.get("stage").and_then(Value::as_str) {
        Some("lobi") => {
            use rand::Rng as _;
            let choice = crate::common::parse_int(&text)?;
            let Some(d) = data(choice) else {
                return Some(Reply::text(format!(
                    "❌ *PILIHAN TIDAK VALID*\n\n> Dungeon nomor {choice} tidak ditemukan."
                )));
            };
            if level < d.level {
                return Some(Reply::text(format!(
                    "🔒 *DUNGEON TERKUNCI*\n\n> Level kamu (*Lv {level}*) belum cukup \
                     untuk memasuki *{}*.\n> Kamu butuh minimal *Lv {}*.",
                    d.nama, d.level
                )));
            }

            let stamina = ctx
                .user_field("rpg.stamina")
                .and_then(Value::as_i64)
                .unwrap_or(100);
            if stamina < 30 {
                return Some(Reply::text(format!(
                    "⚡ *STAMINA TIDAK CUKUP*\n\nKamu butuh setidaknya *30 stamina* \
                     untuk masuk.\nSisa stamina kamu saat ini hanya *{stamina}*.\n\n\
                     > 💡 *Tips:* Gunakan perintah `.rest` atau batalkan dulu (ketik `batal`)."
                )));
            }

            let monster = d.monsters[rand::rng().random_range(0..d.monsters.len())];
            let monster_power = d.level * 10 + rand::rng().random_range(0..30);
            let sesi_baru = json!({
                "stage": "encounter",
                "dungeonId": choice,
                "dungeonName": d.nama,
                "levelReq": d.level,
                "monster": monster,
                "monsterPower": monster_power,
                "maxReward": d.max_reward,
                "minReward": d.min_reward,
                "dropChance": d.drop_chance,
                "time": now,
            });
            let text = format!(
                "🚪 *MEMASUKI DUNGEON*\n\nKamu melangkah perlahan ke dalam *{}*...\n\
                 > ⚡ Stamina berkurang *30*\n\n\
                 Tiba-tiba, seekor *👹 {monster}* muncul dari kegelapan dan menghadang jalanmu!\n\n\
                 *⚔️ APA YANG INGIN KAMU LAKUKAN?*\n\
                 > Balas pesan ini dengan `serang` untuk melawan\n\
                 > Balas pesan ini dengan `lari` untuk kabur (berisiko)",
                d.nama
            );
            Some(Reply::text(text).with_mutation(Mutation::SetUser {
                jid: ctx.sender.clone(),
                data: json!({ "rpg": { "stamina": stamina - 30, "dungeon_session": sesi_baru } }),
            }))
        }
        Some("encounter") => {
            use rand::Rng as _;
            let monster = sesi.get("monster").and_then(Value::as_str).unwrap_or("");
            let monster_power = sesi
                .get("monsterPower")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let level_req = sesi.get("levelReq").and_then(Value::as_i64).unwrap_or(1);
            let min_reward = sesi.get("minReward").and_then(Value::as_i64).unwrap_or(0);
            let max_reward = sesi.get("maxReward").and_then(Value::as_i64).unwrap_or(0);
            let drop_chance = sesi.get("dropChance").and_then(Value::as_i64).unwrap_or(0) as f64;
            let health = ctx
                .user_field("rpg.health")
                .and_then(Value::as_i64)
                .unwrap_or(100);

            if matches!(text.as_str(), "serang" | "attack" | "lawan") {
                let attack = ctx
                    .user_field("rpg.attack")
                    .and_then(Value::as_i64)
                    .unwrap_or(10);
                let koin = ctx.user_field("koin").and_then(Value::as_i64).unwrap_or(0);
                let user_power = attack + level * 4 + rand::rng().random_range(0..20);
                let win = user_power >= monster_power || rand::rng().random::<f64>() > 0.4;

                if win {
                    let exp_reward = (150.0 * (level_req as f64 / 2.0)
                        + (rand::rng().random::<f64>() * 200.0).floor())
                        as i64;
                    let gold = if max_reward > min_reward {
                        rand::rng().random_range(min_reward..max_reward)
                    } else {
                        min_reward
                    };

                    let mut inventory = ctx
                        .user_field("inventory")
                        .and_then(Value::as_object)
                        .cloned()
                        .unwrap_or_default();
                    let mut loot_lines = Vec::new();
                    for (item, icon, chance, qty_min, qty_max) in LOOT {
                        if rand::rng().random::<f64>() * 100.0
                            < chance as f64 * (drop_chance / 50.0)
                        {
                            let qty = rand::rng().random_range(qty_min..=qty_max);
                            let lama = inventory.get(item).and_then(Value::as_i64).unwrap_or(0);
                            inventory.insert(item.into(), json!(lama + qty));
                            loot_lines.push(format!("{icon} {item} (x{qty})"));
                        }
                    }

                    let mut report = "🎉 *KEMENANGAN GEMILANG!*\n\n".to_string();
                    report.push_str(&format!(
                        "Dengan serangan mematikan, kamu berhasil menebas *{monster}*!\n\n\
                         *🎁 HADIAH YANG DIDAPAT:*\n\
                         > ✨ EXP: *+{exp_reward}*\n\
                         > 💰 Koin: *+{gold}*\n"
                    ));
                    if !loot_lines.is_empty() {
                        report.push_str("\n*📦 BARANG JARAHAN (LOOT):*\n");
                        for line in &loot_lines {
                            report.push_str(&format!("> {line}\n"));
                        }
                    }

                    let mut data = serde_json::Map::new();
                    data.insert("exp".into(), json!(exp_after(ctx, exp_reward)));
                    data.insert("koin".into(), json!(koin.saturating_add(gold).max(0)));
                    data.insert("rpg".into(), json!({ "dungeon_session": null }));
                    if !inventory.is_empty() {
                        data.insert("inventory".into(), Value::Object(inventory));
                    }
                    Some(
                        Reply::text(report)
                            .with_reaction("🏆")
                            .with_mutation(Mutation::SetUser {
                                jid: ctx.sender.clone(),
                                data: Value::Object(data),
                            }),
                    )
                } else {
                    let gold_loss = (koin as f64 * 0.15).floor() as i64;
                    let new_health = (health - 40).max(1);
                    let report = format!(
                        "💀 *KEKALAHAN TRAGIS!*\n\n\
                         Kekuatanmu belum sebanding! *{monster}* memukul mundur dirimu dengan telak.\n\
                         Kamu berhasil merangkak keluar dengan tubuh penuh luka.\n\n\
                         *💔 KERUGIAN:*\n\
                         > 💸 Uang jatuh: *-{gold_loss} Koin*\n\
                         > ❤️ Darah berkurang: *-40 HP*\n\n\
                         > 💡 *Tips:* Naikan levelmu, makan potion, atau perkuat senjata!"
                    );
                    Some(
                        Reply::text(report)
                            .with_reaction("💀")
                            .with_mutation(Mutation::SetUser {
                                jid: ctx.sender.clone(),
                                data: json!({
                                    "koin": (koin - gold_loss).max(0),
                                    "rpg": { "health": new_health, "dungeon_session": null },
                                }),
                            }),
                    )
                }
            } else if matches!(text.as_str(), "lari" | "kabur" | "run") {
                let lolos = rand::rng().random::<f64>() > 0.5;
                if lolos {
                    return Some(
                        Reply::text(format!(
                            "🏃‍♂️ *BERHASIL KABUR!*\n\nKamu berbalik dan berlari sekuat \
                             tenaga. *{monster}* kehilangan jejakmu!\n\
                             Kamu selamat tanpa cedera, tapi petualangan ini sia-sia."
                        ))
                        .with_reaction("💨")
                        .with_mutation(clear_session(ctx)),
                    );
                }
                let new_health = (health - 25).max(1);
                Some(
                    Reply::text(format!(
                        "💥 *GAGAL KABUR!*\n\nKakimu tersandung bebatuan! *{monster}* \
                         mengejarmu dan mendaratkan cakarnya di tubuhmu!\n\n\
                         *💔 KERUGIAN:*\n> ❤️ Darah berkurang: *-25 HP*"
                    ))
                    .with_reaction("🩸")
                    .with_mutation(Mutation::SetUser {
                        jid: ctx.sender.clone(),
                        data: json!({
                            "rpg": { "health": new_health, "dungeon_session": null },
                        }),
                    }),
                )
            } else {
                Some(Reply::text(
                    "❓ *PILIHAN TIDAK DIKENAL*\n\n\
                     > Balas dengan `serang` untuk melawan monster.\n\
                     > Balas dengan `lari` untuk kabur.\n\
                     > Balas dengan `batal` jika benar-benar menyerah.",
                ))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENDER: &str = "628111@s.whatsapp.net";

    fn run(user: Value) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "dungeon".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: String::new(),
            args: Vec::new(),
            user_data: user,
            ..Default::default()
        };
        (oktz_plugin_api::find("dungeon").unwrap().handler)(ctx)
    }

    fn text_of(r: &Reply) -> &str {
        r.without_mutations().text_body().expect("harus ada teks")
    }

    /// Varian dimiliki: untuk `owned(run(..))` di dalam assert.
    fn owned(r: Reply) -> String {
        r.text_body().map(str::to_string).unwrap_or_default()
    }

    fn written(r: &Reply) -> &Value {
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetUser { jid, data } => {
                assert_eq!(jid, SENDER);
                data
            }
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    #[test]
    fn registers_with_the_js_plugin_config() {
        let p = oktz_plugin_api::find("dungeon").unwrap();
        assert_eq!(p.meta.name, "dungeon");
        assert_eq!(p.meta.alias, ["dg", "labirin"]);
        assert_eq!(p.meta.category, "game");
        assert_eq!(
            p.meta.description,
            "Jelajahi dungeon dan lawan monster secara interaktif"
        );
        assert_eq!(p.meta.usage, ".dungeon");
        assert_eq!(p.meta.example, ".dungeon");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["dg", "labirin"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "dungeon");
        }
    }

    #[test]
    fn a_fresh_user_gets_the_lobby_and_a_lobi_session() {
        let r = run(serde_json::json!({}));
        let t = text_of(&r);
        assert!(t.starts_with("🏰 *LOBI DUNGEON*"), "{t}");
        assert!(t.contains("> Level: *1*"), "{t}");
        assert!(t.contains("> Stamina: *100/100*"), "{t}");
        assert!(t.contains("🔓 *1.* 🌲 Hutan Gelap (Lv 1+)"), "{t}");
        assert!(t.contains("🔒 *3.* 🏰 Kastil Tua (Butuh Lv 10)"), "{t}");

        let data = written(&r);
        assert_eq!(
            data["rpg"]["dungeon_session"]["stage"],
            serde_json::json!("lobi")
        );
        assert!(data["rpg"]["dungeon_session"]["time"].as_i64().unwrap() > 0);
        assert_eq!(data["rpg"]["attack"], serde_json::json!(10));
        assert_eq!(data["rpg"]["maxStamina"], serde_json::json!(100));
        assert_eq!(
            data["inventory"],
            serde_json::json!({}),
            "`if (!user.inventory)`"
        );
    }

    #[test]
    fn existing_rpg_values_are_preserved_and_zero_falls_back_to_the_default() {
        let r = run(serde_json::json!({"rpg": {"attack": 55, "stamina": 0, "health": 30}}));
        let data = written(&r);
        assert_eq!(
            data["rpg"]["attack"],
            serde_json::json!(55),
            "nilai ada tidak ditimpa"
        );
        assert_eq!(data["rpg"]["health"], serde_json::json!(30));
        assert_eq!(
            data["rpg"]["stamina"],
            serde_json::json!(100),
            "`!0` → default"
        );
    }

    #[test]
    fn an_existing_inventory_is_not_reset() {
        let r = run(serde_json::json!({"inventory": {"iron": 3}}));
        let data = written(&r);
        assert!(
            data.get("inventory").is_none(),
            "tidak ada key inventory di patch: {data}"
        );
    }

    #[test]
    fn a_live_session_blocks_a_second_run_and_writes_nothing() {
        let now = crate::common::now_ms();
        let r =
            run(serde_json::json!({"rpg": {"dungeon_session": {"stage": "lobi", "time": now}}}));
        assert_eq!(text_of(&r), ACTIVE);
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn an_expired_session_is_replaced_by_a_new_one() {
        let stale = crate::common::now_ms() - SESSION_TIMEOUT_MS - 1;
        let r = run(
            serde_json::json!({"rpg": {"dungeon_session": {"stage": "encounter", "time": stale}}}),
        );
        assert!(
            text_of(&r).starts_with("🏰 *LOBI DUNGEON*"),
            "{}",
            text_of(&r)
        );
        let data = written(&r);
        assert_eq!(
            data["rpg"]["dungeon_session"]["stage"],
            serde_json::json!("lobi")
        );
    }

    #[test]
    fn a_session_exactly_at_the_timeout_is_still_alive() {
        let t0 = 1_000_000_000_000;
        assert!(
            session_still_alive(t0 + SESSION_TIMEOUT_MS, t0),
            "tepat 5 menit masih hidup"
        );
        assert!(
            !session_still_alive(t0 + SESSION_TIMEOUT_MS + 1, t0),
            "1 ms lewat timeout sudah mati"
        );
        assert!(session_still_alive(t0 + 1, t0), "sesi baru hidup");
    }

    #[test]
    fn a_session_a_minute_short_of_the_timeout_keeps_the_lobby_reply() {
        // Batasnya dikunci di `session_still_alive`; lewat handler cukup
        // membuktikan cabang "masih hidup" yang dipilih, dengan margin 1 menit
        // supaya pencatatan jam di test tidak ikut menentukan hasilnya.
        let fresh = crate::common::now_ms() - SESSION_TIMEOUT_MS + 60_000;
        let r =
            run(serde_json::json!({"rpg": {"dungeon_session": {"stage": "lobi", "time": fresh}}}));
        assert_eq!(text_of(&r), ACTIVE);
    }

    #[test]
    fn a_session_without_a_timestamp_is_treated_as_expired() {
        let r = run(serde_json::json!({"rpg": {"dungeon_session": {"stage": "lobi"}}}));
        assert!(
            text_of(&r).starts_with("🏰 *LOBI DUNGEON*"),
            "{}",
            text_of(&r)
        );
    }

    #[test]
    fn the_unlock_line_follows_the_level_threshold_per_dungeon() {
        let t = owned(run(serde_json::json!({"level": 5})));
        assert!(t.contains("🔓 *1.* 🌲 Hutan Gelap"), "{t}");
        assert!(t.contains("🔓 *2.* 🍄 Rawa Beracun (Lv 5+)"), "{t}");
        assert!(t.contains("🔒 *3.* 🏰 Kastil Tua"), "{t}");
        assert!(t.contains("> Level: *5*"), "{t}");

        let t = owned(run(serde_json::json!({"level": 50})));
        for (id, _, _) in DUNGEONS {
            assert!(
                t.contains(&format!("🔓 *{id}.*")),
                "dungeon {id} belum terbuka: {t}"
            );
        }
    }

    #[test]
    fn level_zero_is_read_as_one_because_of_the_js_or_fallback() {
        let t = owned(run(serde_json::json!({"level": 0})));
        assert!(t.contains("> Level: *1*"), "{t}");
        assert!(t.contains("🔓 *1.*"), "{t}");
    }

    #[test]
    fn a_negative_level_falls_through_to_the_level_guard() {
        // `-5 || 1` = -5 di versi awal, jadi level negatif bukan 1 dan guard level
        // yang menangkapnya.
        let r = run(serde_json::json!({"level": -5}));
        assert_eq!(text_of(&r), too_low(-5));
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn the_level_guard_exists_even_though_the_table_covers_level_one() {
        // Dungeon termurah butuh level 1, jadi cabang ini tidak bisa dipicu
        // lewat handler — diuji langsung supaya tetap hidup.
        assert_eq!(
            too_low(0),
            "❌ *LEVEL TERLALU RENDAH*\n\n> Level kamu saat ini adalah *0*. Kamu butuh minimal level *1* untuk memasuki dungeon paling mudah."
        );
        assert!(DUNGEONS.iter().all(|(_, _, req)| *req >= 1));
    }

    #[test]
    fn a_null_rpg_object_is_handled_without_panicking() {
        for user in [
            serde_json::json!(null),
            serde_json::json!({}),
            serde_json::json!({"rpg": null}),
            serde_json::json!({"rpg": "bukan object"}),
        ] {
            let r = run(user.clone());
            assert!(text_of(&r).starts_with("🏰 *LOBI DUNGEON*"), "input {user}");
            assert_eq!(
                written(&r)["rpg"]["stamina"],
                serde_json::json!(100),
                "input {user}"
            );
        }
    }

    fn answer_ctx(user: Value, text: &str) -> Ctx {
        Ctx {
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: text.into(),
            user_data: user,
            ..Default::default()
        }
    }

    fn answer_mutation(r: &Reply) -> &Value {
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetUser { data, .. } => data,
            other => panic!("harus SetUser, dapat {other:?}"),
        }
    }

    fn lobi_user(level: i64) -> Value {
        serde_json::json!({
            "level": level,
            "rpg": {
                "stamina": 100,
                "attack": 10,
                "health": 100,
                "dungeon_session": {"stage": "lobi", "time": crate::common::now_ms()},
            },
        })
    }

    #[test]
    fn picking_a_dungeon_spends_stamina_and_starts_the_encounter() {
        let r = dungeon_answer(&answer_ctx(lobi_user(1), "1")).expect("pilihan ditangani");
        assert!(text_of(&r).contains("MEMASUKI DUNGEON"));
        let data = answer_mutation(&r);
        assert_eq!(data["rpg"]["stamina"], serde_json::json!(70));
        assert_eq!(
            data["rpg"]["dungeon_session"]["stage"],
            serde_json::json!("encounter")
        );
    }

    #[test]
    fn a_locked_dungeon_is_rejected_without_mutation() {
        let r = dungeon_answer(&answer_ctx(lobi_user(1), "10")).expect("ditolak");
        assert!(text_of(&r).contains("DUNGEON TERKUNCI"));
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn batal_clears_the_dungeon_session() {
        let r = dungeon_answer(&answer_ctx(lobi_user(1), "batal")).expect("dibatalkan");
        assert!(text_of(&r).contains("keluar dari Lobi Dungeon"));
        assert_eq!(answer_mutation(&r)["rpg"]["dungeon_session"], Value::Null);
    }

    #[test]
    fn an_expired_dungeon_session_is_reported_and_cleared() {
        let stale = crate::common::now_ms() - SESSION_TIMEOUT_MS - 1;
        let user = serde_json::json!({
            "rpg": {"dungeon_session": {"stage": "lobi", "time": stale}},
        });
        let r = dungeon_answer(&answer_ctx(user, "1")).expect("kedaluwarsa");
        assert!(text_of(&r).contains("SESI DUNGEON KEDALUWARSA"));
        assert_eq!(answer_mutation(&r)["rpg"]["dungeon_session"], Value::Null);
    }

    #[test]
    fn attacking_ends_the_encounter_and_writes_rewards() {
        let user = serde_json::json!({
            "level": 1,
            "koin": 1000,
            "rpg": {
                "attack": 1000,
                "health": 100,
                "dungeon_session": {
                    "stage": "encounter",
                    "monster": "Goblin Liar",
                    "monsterPower": 1,
                    "levelReq": 1,
                    "minReward": 100,
                    "maxReward": 300,
                    "dropChance": 40,
                    "time": crate::common::now_ms(),
                },
            },
        });
        let r = dungeon_answer(&answer_ctx(user, "serang")).expect("serang ditangani");
        let data = answer_mutation(&r);
        assert_eq!(data["rpg"]["dungeon_session"], Value::Null);
        assert!(
            data.get("koin").is_some(),
            "hadiah koin harus ditulis: {data}"
        );
    }

    #[test]
    fn an_unknown_text_in_the_encounter_keeps_the_session() {
        let user = serde_json::json!({
            "rpg": {"dungeon_session": {"stage": "encounter", "time": crate::common::now_ms()}},
        });
        let r = dungeon_answer(&answer_ctx(user, "halo")).expect("panduan");
        assert!(text_of(&r).contains("PILIHAN TIDAK DIKENAL"));
        assert!(r.mutations().is_empty());
    }
}
