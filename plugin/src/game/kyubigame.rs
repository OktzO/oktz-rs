//! Lobi misi shinobi.

use oktz_plugin_api::{Ctx, Mutation, Reply, answer_handler, plugin};
use serde_json::{Value, json};

pub(crate) const SESSION_TIMEOUT_MS: i64 = 5 * 60 * 1000;

/// `now - at <= SESSION_TIMEOUT_MS` — `<=`, jadi sesi
/// yang umurnya persis 5 menit masih aktif. Dipisah dari handler karena
/// batasnya hanya bisa dikunci persis kalau `now` dikontrol test; lewat
/// handler, `now_ms()` di dalam selalu beberapa ms lebih besar dari
/// dibaca test.
pub(crate) fn session_still_running(now: i64, at: i64) -> bool {
    now - at <= SESSION_TIMEOUT_MS
}

pub(crate) const SESSION_FIELD: &str = "rpg.kyubigame_session";

/// `LOCATIONS` — id, nama, level minimal.
pub(crate) const LOCATIONS: [(i64, &str, i64); 7] = [
    (1, "🍃 Desa Konoha", 1),
    (2, "🌳 Hutan Kematian", 5),
    (3, "☁️ Padang Petir", 10),
    (4, "🦇 Gua Akatsuki", 15),
    (5, "🌊 Lembah Akhir", 25),
    (6, "💥 Medan Perang Shinobi", 35),
    (7, "🦊 Kurama's Cage", 50),
];

pub(crate) const ACTIVE: &str = "⚔️ *MISI SHINOBI MASIH AKTIF*\n\n\
     Kamu masih berada di medan pertempuran!\n\
     > Balas pesan terakhir bot dengan (`serang` / `lari`) atau batalkan misi (ketik `batal`).";

/// Cabang `available.length === 0`. Tidak terjangkau lewat handler — lokasi
/// termurah butuh level 1 dan `user.level || 1` minimal 1 — tapi tetap dibawa
/// supaya ambang level tidak bisa berubah diam-diam.
pub(crate) fn too_low(user_level: i64) -> String {
    format!(
        "❌ *LEVEL TERLALU RENDAH*\n\n> Level kamu saat ini adalah *{user_level}*. \
         Kamu butuh minimal level *1* untuk memulai petualangan shinobi."
    )
}

pub(crate) fn lobby(user_level: i64, stamina: i64) -> String {
    let mut txt = format!(
        "⛩️ *LOBI SHINOBI*\n\n📊 *Statistik Shinobi:*\n> Level: *{user_level}*\n> Stamina: *{stamina}/100*\n\n\
         Pilih lokasi misi yang ingin kamu jelajahi:\n\n"
    );
    for (id, name, req) in LOCATIONS {
        if user_level >= req {
            txt.push_str(&format!("🔓 *{id}.* {name} (Lv {req}+)\n"));
        } else {
            txt.push_str(&format!("> 🔒 *{id}.* {name} (Butuh Lv {req})\n"));
        }
    }
    txt.push_str(
        "\n> 💡 Balas pesan ini dengan *angka* lokasi misi (contoh: `1`) atau ketik `batal` untuk keluar.",
    );
    txt
}

fn default_field(rpg: Option<&Value>, key: &str, default: i64) -> i64 {
    let v = rpg.and_then(|r| r.get(key));
    if crate::common::js_truthy(v) {
        v.and_then(Value::as_i64).unwrap_or(default)
    } else {
        default
    }
}

#[plugin(
    name = "kyubigame",
    alias = ["kyubi", "naruto", "shinobi"],
    category = "game",
    description = "Jelajahi dunia shinobi dan hadapi musuh Ninja terkuat",
    usage = ".kyubigame",
    example = ".kyubigame",
    cooldown = 60,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn kyubigame(ctx: Ctx) -> Reply {
    // `user.level || 1` — `0`/`null` jadi 1, negatif tetap negatif (truthy).
    let raw_level = crate::common::int_field(&ctx, "level");
    let user_level = if raw_level == 0 { 1 } else { raw_level };
    let rpg = ctx.user_field("rpg");
    let stamina = default_field(rpg, "stamina", 100);

    if let Some(session) = ctx.user_field(SESSION_FIELD)
        && !session.is_null()
    {
        let at = session.get("time").and_then(Value::as_i64).unwrap_or(0);
        if session_still_running(crate::common::now_ms(), at) {
            return Reply::text(ACTIVE);
        }
    }

    if !LOCATIONS.iter().any(|(_, _, req)| user_level >= *req) {
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
        "kyubigame_session".into(),
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

struct Lokasi {
    id: i64,
    nama: &'static str,
    level: i64,
    monsters: &'static [&'static str],
    min_reward: i64,
    max_reward: i64,
    drop_chance: i64,
}

const LOKASI: [Lokasi; 7] = [
    Lokasi {
        id: 1,
        nama: "🍃 Desa Konoha",
        level: 1,
        monsters: &[
            "Genin Pemula",
            "Bandit Liar",
            "Anjing Hutan",
            "Ninja Pengintai",
        ],
        min_reward: 100,
        max_reward: 300,
        drop_chance: 40,
    },
    Lokasi {
        id: 2,
        nama: "🌳 Hutan Kematian",
        level: 5,
        monsters: &[
            "Ninja Otogakure",
            "Harimau Raksasa",
            "Lipan Racun",
            "Ular Orochimaru",
        ],
        min_reward: 250,
        max_reward: 500,
        drop_chance: 45,
    },
    Lokasi {
        id: 3,
        nama: "☁️ Padang Petir",
        level: 10,
        monsters: &[
            "Ninja Kumo",
            "Samurai Besi",
            "Burung Hantu Petir",
            "Serigala Listrik",
        ],
        min_reward: 400,
        max_reward: 800,
        drop_chance: 50,
    },
    Lokasi {
        id: 4,
        nama: "🦇 Gua Akatsuki",
        level: 15,
        monsters: &[
            "Klon Zetsu Putih",
            "Kelelawar Beracun",
            "Boneka Sasori",
            "Ninja Pelarian",
        ],
        min_reward: 600,
        max_reward: 1200,
        drop_chance: 55,
    },
    Lokasi {
        id: 5,
        nama: "🌊 Lembah Akhir",
        level: 25,
        monsters: &[
            "Ninja Pembunuh",
            "Mizukage Klon",
            "Uchiha Hantu",
            "Patung Golem",
        ],
        min_reward: 900,
        max_reward: 1700,
        drop_chance: 60,
    },
    Lokasi {
        id: 6,
        nama: "💥 Medan Perang Shinobi",
        level: 35,
        monsters: &[
            "Zetsu Raksasa",
            "Edo Tensei Kage",
            "Shinobi Undead",
            "Pasukan Klon",
        ],
        min_reward: 1300,
        max_reward: 2400,
        drop_chance: 65,
    },
    Lokasi {
        id: 7,
        nama: "🦊 Kurama's Cage",
        level: 50,
        monsters: &[
            "Chakra Ekor Sembilan",
            "Kyubi Liar",
            "Kurama Kegelapan",
            "Roh Bijuu",
        ],
        min_reward: 2500,
        max_reward: 4500,
        drop_chance: 75,
    },
];

/// `LOOT_TABLE`.
const LOOT: [(&str, &str, i64, i64, i64); 5] = [
    ("kunai", "🗡️", 40, 2, 5),
    ("shuriken", "⚔️", 35, 3, 6),
    ("chakra", "🌀", 30, 1, 3),
    ("scroll", "📜", 15, 1, 2),
    ("bowlramen", "🍜", 20, 1, 2),
];

fn lokasi(id: i64) -> Option<&'static Lokasi> {
    LOKASI.iter().find(|l| l.id == id)
}

fn clear_session(ctx: &Ctx) -> Mutation {
    Mutation::SetUser {
        jid: ctx.sender.clone(),
        data: json!({ "rpg": { "kyubigame_session": null } }),
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

/// `kyubigameAnswerHandler` — balasan lokasi misi
/// atau aksi tempur, bukan command.
#[answer_handler]
pub fn kyubigame_answer(ctx: &Ctx) -> Option<Reply> {
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

    if !session_still_running(now, at) {
        return Some(
            Reply::text(
                "⏰ *MISI KEDALUWARSA*\n\n> Sesi misi shinobi kamu sudah hangus karena \
                 tidak aktif selama 5 menit.",
            )
            .with_mutation(clear_session(ctx)),
        );
    }

    let raw_level = crate::common::int_field(ctx, "level");
    let level = if raw_level == 0 { 1 } else { raw_level };

    if matches!(text.as_str(), "batal" | "cancel" | "keluar") {
        return Some(
            Reply::text("🚪 Kamu berhasil membatalkan misi dan kembali ke desa dengan selamat.")
                .with_mutation(clear_session(ctx)),
        );
    }

    match sesi.get("stage").and_then(Value::as_str) {
        Some("lobi") => {
            use rand::Rng as _;
            let choice = crate::common::parse_int(&text)?;
            let Some(loc) = lokasi(choice) else {
                return Some(Reply::text(format!(
                    "❌ *MISI TIDAK VALID*\n\n> Lokasi nomor {choice} tidak ada di peta shinobi."
                )));
            };
            if level < loc.level {
                return Some(Reply::text(format!(
                    "🔒 *MISI TERKUNCI*\n\n> Level kamu (*Lv {level}*) belum cukup untuk \
                     memasuki *{}*.\n> Kamu butuh minimal *Lv {}*.",
                    loc.nama, loc.level
                )));
            }

            let stamina = ctx
                .user_field("rpg.stamina")
                .and_then(Value::as_i64)
                .unwrap_or(100);
            if stamina < 30 {
                return Some(Reply::text(format!(
                    "⚡ *CHAKRA/STAMINA TIDAK CUKUP*\n\nKamu butuh setidaknya *30 stamina* \
                     untuk masuk.\nSisa stamina kamu saat ini hanya *{stamina}*.\n\n\
                     > 💡 *Tips:* Gunakan perintah `.rest` atau batalkan dulu (ketik `batal`)."
                )));
            }

            let monster = loc.monsters[rand::rng().random_range(0..loc.monsters.len())];
            let monster_power = loc.level * 10 + rand::rng().random_range(0..30);
            let sesi_baru = json!({
                "stage": "encounter",
                "locationId": loc.id,
                "locationName": loc.nama,
                "levelReq": loc.level,
                "monster": monster,
                "monsterPower": monster_power,
                "maxReward": loc.max_reward,
                "minReward": loc.min_reward,
                "dropChance": loc.drop_chance,
                "time": now,
            });
            let text = format!(
                "⛩️ *MEMASUKI AREA MISI*\n\nKamu melompat perlahan menyusuri *{}*...\n\
                 > ⚡ Stamina berkurang *30*\n\n\
                 Tiba-tiba, seorang *👹 {monster}* melesat dari kegelapan dan menghadang jalanmu!\n\n\
                 *⚔️ APA YANG INGIN KAMU LAKUKAN?*\n\
                 > Balas pesan ini dengan `serang` untuk melawan\n\
                 > Balas pesan ini dengan `lari` untuk mundur (berisiko)",
                loc.nama
            );
            Some(Reply::text(text).with_mutation(Mutation::SetUser {
                jid: ctx.sender.clone(),
                data: json!({
                    "rpg": { "stamina": stamina - 30, "kyubigame_session": sesi_baru }
                }),
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
                    let ryo = if max_reward > min_reward {
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

                    let mut report = "🎉 *MISI BERHASIL!*\n\n".to_string();
                    report.push_str(&format!(
                        "Dengan jutsu mematikan, kamu berhasil mengalahkan *{monster}*!\n\n\
                         *🎁 HADIAH PENYELESAIAN MISI:*\n\
                         > ✨ EXP: *+{exp_reward}*\n\
                         > 💰 Ryo (Koin): *+{ryo}*\n"
                    ));
                    if !loot_lines.is_empty() {
                        report.push_str("\n*📦 BARANG JARAHAN SHINOBI:*\n");
                        for line in &loot_lines {
                            report.push_str(&format!("> {line}\n"));
                        }
                    }

                    let mut data = serde_json::Map::new();
                    data.insert("exp".into(), json!(exp_after(ctx, exp_reward)));
                    data.insert("koin".into(), json!(koin.saturating_add(ryo).max(0)));
                    data.insert("rpg".into(), json!({ "kyubigame_session": null }));
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
                    let ryo_loss = (koin as f64 * 0.15).floor() as i64;
                    let new_health = (health - 40).max(1);
                    let report = format!(
                        "💀 *MISI GAGAL!*\n\n\
                         Kekuatanmu belum sebanding! *{monster}* memukul mundur dirimu dengan telak.\n\
                         Kamu berhasil menggunakan jutsu substitusi dan merangkak keluar dengan tubuh penuh luka.\n\n\
                         *💔 KERUGIAN:*\n\
                         > 💸 Uang jatuh: *-{ryo_loss} Ryo*\n\
                         > ❤️ Darah berkurang: *-40 HP*\n\n\
                         > 💡 *Tips:* Naikan levelmu, makan ramen, atau perkuat jutsumu!"
                    );
                    Some(
                        Reply::text(report)
                            .with_reaction("💀")
                            .with_mutation(Mutation::SetUser {
                                jid: ctx.sender.clone(),
                                data: json!({
                                    "koin": (koin - ryo_loss).max(0),
                                    "rpg": { "health": new_health, "kyubigame_session": null },
                                }),
                            }),
                    )
                }
            } else if matches!(text.as_str(), "lari" | "kabur" | "run") {
                let lolos = rand::rng().random::<f64>() > 0.5;
                if lolos {
                    return Some(
                        Reply::text(format!(
                            "🏃‍♂️ *BERHASIL KABUR!*\n\nKamu melemparkan bom asap dan berlari \
                             sekuat tenaga. *{monster}* kehilangan jejakmu!\n\
                             Kamu selamat tanpa cedera, tapi petualangan ini sia-sia."
                        ))
                        .with_reaction("💨")
                        .with_mutation(clear_session(ctx)),
                    );
                }
                let new_health = (health - 25).max(1);
                Some(
                    Reply::text(format!(
                        "💥 *GAGAL KABUR!*\n\nKakimu tersandung jebakan ninja! *{monster}* \
                         mengejarmu dan mendaratkan serangannya di tubuhmu!\n\n\
                         *💔 KERUGIAN:*\n> ❤️ Darah berkurang: *-25 HP*"
                    ))
                    .with_reaction("🩸")
                    .with_mutation(Mutation::SetUser {
                        jid: ctx.sender.clone(),
                        data: json!({
                            "rpg": { "health": new_health, "kyubigame_session": null },
                        }),
                    }),
                )
            } else {
                Some(Reply::text(
                    "❓ *PERINTAH TIDAK DIKENAL*\n\n\
                     > Balas dengan `serang` untuk melawan musuh.\n\
                     > Balas dengan `lari` untuk kabur.\n\
                     > Balas dengan `batal` jika ingin membatalkan misi.",
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
            command: "kyubigame".into(),
            sender: SENDER.into(),
            chat: SENDER.into(),
            text: String::new(),
            args: Vec::new(),
            user_data: user,
            ..Default::default()
        };
        (oktz_plugin_api::find("kyubigame").unwrap().handler)(ctx)
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
        let p = oktz_plugin_api::find("kyubigame").unwrap();
        assert_eq!(p.meta.name, "kyubigame");
        assert_eq!(p.meta.alias, ["kyubi", "naruto", "shinobi"]);
        assert_eq!(p.meta.category, "game");
        assert_eq!(
            p.meta.description,
            "Jelajahi dunia shinobi dan hadapi musuh Ninja terkuat"
        );
        assert_eq!(p.meta.usage, ".kyubigame");
        assert_eq!(p.meta.example, ".kyubigame");
        assert_eq!(p.meta.cooldown, 60);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn every_alias_resolves_to_this_plugin() {
        for a in ["kyubi", "naruto", "shinobi"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "kyubigame");
        }
    }

    #[test]
    fn a_fresh_user_gets_the_lobby_and_a_lobi_session() {
        let r = run(serde_json::json!({}));
        let t = text_of(&r);
        assert!(t.starts_with("⛩️ *LOBI SHINOBI*"), "{t}");
        assert!(t.contains("> Level: *1*"), "{t}");
        assert!(t.contains("> Stamina: *100/100*"), "{t}");
        assert!(t.contains("🔓 *1.* 🍃 Desa Konoha (Lv 1+)"), "{t}");
        assert!(t.contains("🔒 *3.* ☁️ Padang Petir (Butuh Lv 10)"), "{t}");

        let data = written(&r);
        assert_eq!(
            data["rpg"]["kyubigame_session"]["stage"],
            serde_json::json!("lobi")
        );
        assert!(data["rpg"]["kyubigame_session"]["time"].as_i64().unwrap() > 0);
        assert_eq!(data["rpg"]["attack"], serde_json::json!(10));
        assert_eq!(data["inventory"], serde_json::json!({}));
    }

    #[test]
    fn existing_rpg_values_survive_and_zero_falls_back_to_the_default() {
        let r = run(serde_json::json!({"rpg": {"attack": 77, "stamina": 0}}));
        let data = written(&r);
        assert_eq!(data["rpg"]["attack"], serde_json::json!(77));
        assert_eq!(data["rpg"]["stamina"], serde_json::json!(100));
    }

    #[test]
    fn a_live_mission_blocks_a_second_run() {
        let now = crate::common::now_ms();
        let r = run(
            serde_json::json!({"rpg": {"kyubigame_session": {"stage": "encounter", "time": now}}}),
        );
        assert_eq!(text_of(&r), ACTIVE);
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn an_expired_mission_is_replaced_by_a_fresh_lobi_session() {
        let stale = crate::common::now_ms() - SESSION_TIMEOUT_MS - 1;
        let r = run(
            serde_json::json!({"rpg": {"kyubigame_session": {"stage": "encounter", "time": stale}}}),
        );
        assert!(
            text_of(&r).starts_with("⛩️ *LOBI SHINOBI*"),
            "{}",
            text_of(&r)
        );
        assert_eq!(
            written(&r)["rpg"]["kyubigame_session"]["stage"],
            serde_json::json!("lobi")
        );
    }

    #[test]
    fn a_mission_exactly_at_the_timeout_is_still_running() {
        let t0 = 1_000_000_000_000;
        assert!(
            session_still_running(t0 + SESSION_TIMEOUT_MS, t0),
            "tepat 5 menit masih aktif"
        );
        assert!(
            !session_still_running(t0 + SESSION_TIMEOUT_MS + 1, t0),
            "1 ms lewat timeout sudah expired"
        );
        assert!(session_still_running(t0 + 1, t0), "sesi baru aktif");
    }

    #[test]
    fn a_just_short_of_the_timeout_is_dispatched_by_the_handler() {
        // Batasnya dikunci di `session_still_running`; lewat handler cukup
        // membuktikan cabang "masih aktif" benar-benar dipilih,
        // margin 1 menit supaya jam tidak bisa ikut menentukan hasilnya.
        let fresh = crate::common::now_ms() - SESSION_TIMEOUT_MS + 60_000;
        let r = run(
            serde_json::json!({"rpg": {"kyubigame_session": {"stage": "lobi", "time": fresh}}}),
        );
        assert_eq!(text_of(&r), ACTIVE);
    }

    #[test]
    fn a_session_without_a_timestamp_is_treated_as_expired() {
        let r = run(serde_json::json!({"rpg": {"kyubigame_session": {}}}));
        assert!(
            text_of(&r).starts_with("⛩️ *LOBI SHINOBI*"),
            "{}",
            text_of(&r)
        );
    }

    #[test]
    fn the_lock_line_follows_the_level_threshold_per_location() {
        let t = owned(run(serde_json::json!({"level": 5})));
        assert!(t.contains("🔓 *2.* 🌳 Hutan Kematian (Lv 5+)"), "{t}");
        assert!(t.contains("🔒 *3.* ☁️ Padang Petir"), "{t}");

        let t = owned(run(serde_json::json!({"level": 50})));
        assert!(t.contains("🔓 *7.* 🦊 Kurama's Cage (Lv 50+)"), "{t}");
    }

    #[test]
    fn level_zero_becomes_one_and_a_negative_level_hits_the_guard() {
        let t = owned(run(serde_json::json!({"level": 0})));
        assert!(t.contains("> Level: *1*"), "{t}");

        let r = run(serde_json::json!({"level": -3}));
        assert_eq!(text_of(&r), too_low(-3));
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn the_level_guard_survives_as_dead_branches_are_forbidden() {
        assert!(too_low(-1).starts_with("❌ *LEVEL TERLALU RENDAH*"));
        assert!(LOCATIONS.iter().all(|(_, _, req)| *req >= 1));
    }

    #[test]
    fn a_null_rpg_object_is_handled_without_panicking() {
        for user in [
            serde_json::json!(null),
            serde_json::json!({}),
            serde_json::json!({"rpg": null}),
            serde_json::json!({"rpg": 7}),
        ] {
            let r = run(user.clone());
            assert!(text_of(&r).starts_with("⛩️ *LOBI SHINOBI*"), "input {user}");
            assert_eq!(
                written(&r)["rpg"]["health"],
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
                "kyubigame_session": {"stage": "lobi", "time": crate::common::now_ms()},
            },
        })
    }

    #[test]
    fn picking_a_location_spends_stamina_and_starts_the_encounter() {
        let ctx = answer_ctx(lobi_user(1), "1");
        let r = kyubigame_answer(&ctx).expect("pilihan lokasi ditangani");
        let t = text_of(&r);
        assert!(t.contains("MEMASUKI AREA MISI"), "{t}");
        let data = answer_mutation(&r);
        assert_eq!(data["rpg"]["stamina"], serde_json::json!(70));
        assert_eq!(
            data["rpg"]["kyubigame_session"]["stage"],
            serde_json::json!("encounter")
        );
    }

    #[test]
    fn a_locked_location_is_rejected_without_mutation() {
        let r = kyubigame_answer(&answer_ctx(lobi_user(1), "7")).expect("ditolak");
        assert!(text_of(&r).contains("MISI TERKUNCI"));
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn batal_clears_the_session() {
        let r = kyubigame_answer(&answer_ctx(lobi_user(1), "batal")).expect("dibatalkan");
        assert!(text_of(&r).contains("membatalkan misi"));
        assert_eq!(answer_mutation(&r)["rpg"]["kyubigame_session"], Value::Null);
    }

    #[test]
    fn an_expired_session_is_reported_and_cleared() {
        let stale = crate::common::now_ms() - SESSION_TIMEOUT_MS - 1;
        let user = serde_json::json!({
            "rpg": {"kyubigame_session": {"stage": "lobi", "time": stale}},
        });
        let r = kyubigame_answer(&answer_ctx(user, "1")).expect("kedaluwarsa");
        assert!(text_of(&r).contains("MISI KEDALUWARSA"));
        assert_eq!(answer_mutation(&r)["rpg"]["kyubigame_session"], Value::Null);
    }

    #[test]
    fn attacking_ends_the_encounter_and_writes_koin_or_health() {
        let user = serde_json::json!({
            "level": 1,
            "koin": 1000,
            "rpg": {
                "attack": 1000,
                "health": 100,
                "kyubigame_session": {
                    "stage": "encounter",
                    "monster": "Bandit Liar",
                    "monsterPower": 1,
                    "levelReq": 1,
                    "minReward": 100,
                    "maxReward": 300,
                    "dropChance": 40,
                    "time": crate::common::now_ms(),
                },
            },
        });
        let r = kyubigame_answer(&answer_ctx(user, "serang")).expect("serang ditangani");
        let data = answer_mutation(&r);
        assert_eq!(data["rpg"]["kyubigame_session"], Value::Null);
        assert!(
            data.get("koin").is_some(),
            "hadiah koin harus ditulis: {data}"
        );
    }

    #[test]
    fn an_unknown_text_in_the_encounter_keeps_the_session() {
        let user = serde_json::json!({
            "rpg": {"kyubigame_session": {"stage": "encounter", "time": crate::common::now_ms()}},
        });
        let r = kyubigame_answer(&answer_ctx(user, "halo")).expect("panduan");
        assert!(text_of(&r).contains("PERINTAH TIDAK DIKENAL"));
        assert!(r.mutations().is_empty());
    }
}
