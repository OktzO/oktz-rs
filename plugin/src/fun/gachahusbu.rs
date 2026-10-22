//! `plugins/fun/gachahusbu.js` — gacha husbu (suami virtual) + sub-command.
//!
//! ## Penyimpangan yang tidak bisa ditiru
//!
//! 1. **Gambar header.** JS mengambil gambar dari Pinterest lewat
//!    `api.nexray.web.id/api/search/pinterest` lalu meng-upload-nya ke WA.
//!    `Reply::CardHeader` hanya bisa menunjuk URL/berkas; upload-nya
//!    dikerjakan pipeline, jadi hasilnya sama — tetapi URL gambarnya memakai
//!    fallback tetap, bukan hasil pencarian Pinterest.
//! 2. **View-once.** JS membungkus kartu dalam `viewOnceMessage`; `Reply`
//!    tidak punya pembungkus itu, jadi kartu dikirim sebagai interactive
//!    message biasa.
//!
//! Sub-command: `husbuku`/`suamiku` (status), `tinggalinhusbu` (putus),
//! `gachahusbu`/`gachasuami` (gacha + panduan), `husbuaction` (aksi).
//! State husbu disimpan di `user.husbu`, statistik di `user.husbuStats`,
//! riwayat di `user.husbuHistory`.

use oktz_plugin_api::registry::{CardHeader, InteractiveButton};
use oktz_plugin_api::{Ctx, MediaSource, Mutation, Reply, plugin};
use serde_json::{Map, Value};

use crate::common::{error_template, js_truthy, now_ms};
use crate::fun::gachawaifu::{
    anger_eff_mood, anger_meter, anger_update, apply_neglect, dowry_of, energi_gain,
    final_gain, fmt_num, get_daily_mood, init_stats, iso_now, iso_to_ms, jnum, js_str,
    koin_gain, map_of, marrage_day, mood_label, mstr, mnum_or, old_exp, record_pull, set_user,
    tier_index, tier_mult, today_str,
};
use crate::rpg::heal::{apply_exp, math_random, rpg_map, thousands_en};
use crate::fun::pool_data::HUSBU_ENTRIES;

/// `TIER_WEIGHTS` (`data/husbu/index.js:37`).
const TIER_WEIGHTS: [(&str, f64); 5] = [
    ("Common", 55.0),
    ("Rare", 25.0),
    ("Epic", 13.0),
    ("Legendary", 5.5),
    ("Mythic", 1.5),
];

/// `VALID_HUSB_PERSONALITIES` (`data/husbu/index.js:38-40`).
const VALID_HUSB_PERSONALITIES: [&str; 11] = [
    "tsundere",
    "kuudere",
    "genki",
    "yandere",
    "dandere",
    "oji-san",
    "playboy",
    "prince",
    "badboy",
    "sunao",
    "femboy",
];

/// `PITY_THRESHOLD` (`src/lib/romance.js:9`).
const PITY_THRESHOLD: i64 = 20;
/// `EVENT_CHANCE` (`src/lib/romance.js:10`).
const EVENT_CHANCE: f64 = 0.18;
/// `MOOD_MULT` (`src/lib/romance.js:20`).
const MOOD_MULT: [(&str, f64); 5] = [
    ("ceria", 1.3),
    ("romantis", 1.2),
    ("biasa", 1.0),
    ("sedih", 0.7),
    ("marah", 0.5),
];

/// `HUSB_PERSONALITIES` (`gachahusbu.js:174`) — dipakai untuk backfill.
const HUSB_PERSONALITIES: [&str; 11] = VALID_HUSB_PERSONALITIES;

/// Gambar fallback saat Pinterest tidak bisa diakses (`gachahusbu.js:40`).
const IMAGE_FALLBACK: &str =
    "https://i.pinimg.com/736x/8f/3e/2a/8f3e2a77ec65cdbcfad4ff3bc17e825f.jpg";

/// Satu aksi `HUSB_ACTIONS` (`src/lib/husbu.js:21-80`).
pub(crate) struct ActionDef {
    key: &'static str,
    phase: &'static str,
    base: (f64, f64),
    exp: i64,
    likes: &'static [&'static str],
    dislikes: &'static [&'static str],
    text: &'static str,
}

/// `HUSB_ACTIONS` — 53 aksi.
const HUSB_ACTIONS: &[ActionDef] = &[
    ActionDef { key: "jalan_taman", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["sunao", "dandere", "kuudere", "genki", "femboy"],
        dislikes: &["badboy", "prince"],
        text: "🌳 Kamu ajak *{n}* jalan di taman. Angin sejuk, dia menggenggam tanganmu sambil tersenyum." },
    ActionDef { key: "jalan_mall", phase: "approach", base: (4.0, 10.0), exp: 25,
        likes: &["playboy"],
        dislikes: &["dandere", "kuudere", "genki"],
        text: "🏢 Mall ramai. *{n}* cuma mengekor pelan, terlihat bosan." },
    ActionDef { key: "jalan_pantai", phase: "approach", base: (8.0, 16.0), exp: 35,
        likes: &["genki", "sunao", "kuudere"],
        dislikes: &["prince"],
        text: "🏖️ Ombak pantai. *{n}* berlari-lari di pasir dan mengajakmu main air." },
    ActionDef { key: "jalan_kota", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["genki", "playboy"],
        dislikes: &["dandere", "femboy"],
        text: "🌆 Jalan kaki di tepi kota saat lampu mulai menyala. *{n}* menatapmu kagum." },
    ActionDef { key: "kafe_kopi", phase: "approach", base: (4.0, 8.0), exp: 25,
        likes: &["kuudere"],
        dislikes: &["prince"],
        text: "☕ *{n}* meringis meneguk kopi pahit. Ternyata dia lebih suka manis." },
    ActionDef { key: "kafe_matcha", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["dandere", "playboy", "kuudere"],
        dislikes: &[],
        text: "🍵 Kamu traktir matcha latte. *{n}* tersenyum manis padamu." },
    ActionDef { key: "kafe_kue", phase: "approach", base: (8.0, 15.0), exp: 35,
        likes: &["playboy", "femboy", "genki"],
        dislikes: &[],
        text: "🍰 Berbagi sepotong kue. *{n}* tak mau kalah rebutan minta gigitan." },
    ActionDef { key: "kafe_susu", phase: "approach", base: (5.0, 10.0), exp: 25,
        likes: &["femboy", "sunao"],
        dislikes: &[],
        text: "🥛 Segelas susu hangat. *{n}* menyeruput pelan dan tampak nyaman." },
    ActionDef { key: "bioskop_romantis", phase: "approach", base: (8.0, 16.0), exp: 35,
        likes: &["playboy", "kuudere"],
        dislikes: &["genki"],
        text: "🎬 Film romantis. *{n}* merangkulmu makin erat saat adegan ciuman." },
    ActionDef { key: "bioskop_horor", phase: "approach", base: (10.0, 20.0), exp: 40,
        likes: &["genki", "tsundere"],
        dislikes: &["dandere", "kuudere"],
        text: "👻 JUMPSCARE! *{n}* pura-pura tenang padahal tangannya mencengkerammu." },
    ActionDef { key: "bioskop_animasi", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["genki", "sunao"],
        dislikes: &["prince"],
        text: "🎨 Film animasi. *{n}* tertawa lepas di beberapa adegan." },
    ActionDef { key: "belanja_baju", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["genki", "playboy"],
        dislikes: &["femboy"],
        text: "👕 Kamu pilihkan baju keren buat *{n}*. Dia memamerkannya sambil pipi merona." },
    ActionDef { key: "belanja_perhiasan", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["playboy", "prince"],
        dislikes: &["kuudere"],
        text: "💍 Dia membelikanmu aksesoris manis. Matanya berbinar saat kamu memakainya." },
    ActionDef { key: "belanja_boneka", phase: "approach", base: (5.0, 10.0), exp: 25,
        likes: &["femboy", "genki"],
        dislikes: &["prince"],
        text: "🧸 Boneka kecil mungil. *{n}* malu-malu tapi tetap memeluknya." },
    ActionDef { key: "karaoke_duet", phase: "approach", base: (8.0, 16.0), exp: 35,
        likes: &["genki", "playboy"],
        dislikes: &["kuudere", "tsundere"],
        text: "🎤 Duet lagu favorit. *{n}* semangat sampai fals nada tapi tertawa." },
    ActionDef { key: "karaoke_solo", phase: "approach", base: (4.0, 10.0), exp: 25,
        likes: &["tsundere"],
        dislikes: &["genki"],
        text: "🎵 *{n}* menyanyi sendirian dengan suara merdu. Kamu terpukau." },
    ActionDef { key: "arcade_duo", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["genki", "tsundere"],
        dislikes: &["kuudere"],
        text: "🕹️ Adu skor di arcade. *{n}* menantangmu lagi dan lagi, tak mau kalah." },
    ActionDef { key: "arcade_boneka", phase: "approach", base: (8.0, 14.0), exp: 35,
        likes: &["genki", "playboy"],
        dislikes: &[],
        text: "🎯 *{n}* memenangkan boneka untukmu. Wajahnya berseri-seri." },
    ActionDef { key: "piknik_taman", phase: "approach", base: (8.0, 15.0), exp: 35,
        likes: &["kuudere", "sunao", "femboy"],
        dislikes: &["badboy"],
        text: "🧺 Piknik di atas tikar rumput. *{n}* bersandar santai menatap awan." },
    ActionDef { key: "piknik_pantai", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["genki", "sunao"],
        dislikes: &["prince"],
        text: "🏝️ Piknik pantai dengan bekal buatanmu. *{n}* sangat menikmatinya." },
    ActionDef { key: "masak_bareng", phase: "approach", base: (8.0, 16.0), exp: 35,
        likes: &["sunao", "oji-san", "playboy"],
        dislikes: &[],
        text: "🍳 Masak bareng di dapur. *{n}* antusias mengaduk sausnya." },
    ActionDef { key: "masak_kue", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["sunao", "oji-san", "femboy"],
        dislikes: &[],
        text: "🍰 Memanggang kue bersama. *{n}* bangga dengan hasil kreasimu berdua." },
    ActionDef { key: "restoran_makan", phase: "approach", base: (8.0, 16.0), exp: 35,
        likes: &["oji-san", "playboy", "femboy"],
        dislikes: &["kuudere"],
        text: "🍽️ Makan malam di restoran romantis. *{n}* menatapmu lembut dari seberang meja." },
    ActionDef { key: "restoran_dimsum", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["genki", "playboy"],
        dislikes: &["prince"],
        text: "🥟 Cicip dimsum bareng! *{n}* berebutan denganmu sampai tawa." },
    ActionDef { key: "restoran_bbq", phase: "approach", base: (8.0, 14.0), exp: 35,
        likes: &["genki", "oji-san"],
        dislikes: &["prince"],
        text: "🍖 Bakar-bakar di BBQ. *{n}* memangangkan daging untukmu dengan telaten." },
    ActionDef { key: "olahraga_hiking", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["genki", "kuudere"],
        dislikes: &["playboy", "prince"],
        text: "⛰️ Mendaki bukit. *{n}* terengah-engah tapi semangat dan menjaga langkahmu." },
    ActionDef { key: "olahraga_lari", phase: "approach", base: (8.0, 14.0), exp: 35,
        likes: &["genki", "tsundere"],
        dislikes: &["femboy"],
        text: "🏃 Lari pagi keliling taman. *{n}* menyesuaikan kecepatannya untukmu." },
    ActionDef { key: "olahraga_panjat", phase: "approach", base: (5.0, 10.0), exp: 25,
        likes: &["genki", "badboy"],
        dislikes: &["prince", "femboy"],
        text: "🧗 Panjat tebing indoor. *{n}* merayap pelan demi mengamankanmu." },
    ActionDef { key: "alam_camping", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["kuudere", "sunao", "dandere"],
        dislikes: &["playboy"],
        text: "🏕️ Camping di bawah bintang. *{n}* bersandar santai menatap langit malam." },
    ActionDef { key: "alam_mancing", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["kuudere", "oji-san"],
        dislikes: &["genki"],
        text: "🎣 Memancing di danau tenang. *{n}* senang saat akhirnya dapat ikan." },
    ActionDef { key: "alam_perahu", phase: "approach", base: (8.0, 16.0), exp: 35,
        likes: &["genki", "sunao"],
        dislikes: &["tsundere"],
        text: "⛵ Menyusuri sungai naik perahu. *{n}* menutup mata menikmati angin." },
    ActionDef { key: "seni_museum", phase: "approach", base: (6.0, 12.0), exp: 30,
        likes: &["kuudere", "dandere"],
        dislikes: &["genki"],
        text: "🖼️ Menjelajahi museum. *{n}* takjub di depan lukisan tua." },
    ActionDef { key: "seni_melukis", phase: "approach", base: (8.0, 15.0), exp: 35,
        likes: &["playboy", "sunao", "oji-san"],
        dislikes: &[],
        text: "🎨 Melukis bareng di studio. Karya *{n}* kacau balau tapi lucu." },
    ActionDef { key: "seni_konser", phase: "approach", base: (10.0, 18.0), exp: 40,
        likes: &["genki", "playboy", "sunao"],
        dislikes: &["kuudere"],
        text: "🎸 Nonton konser. *{n}* bernyanyi keras tanpa malu." },
    ActionDef { key: "peluk_belakang", phase: "intim", base: (10.0, 18.0), exp: 45,
        likes: &["yandere", "playboy", "tsundere"],
        dislikes: &[],
        text: "🤗 Kamu memeluk *{n}* dari belakang. Dia terkejut sesaat lalu tersenyum aman." },
    ActionDef { key: "peluk_depan", phase: "intim", base: (12.0, 20.0), exp: 50,
        likes: &["yandere", "playboy", "oji-san"],
        dislikes: &[],
        text: "💑 Kalian berpelukan erat. Detak jantung berdebar seirama." },
    ActionDef { key: "cium_kening", phase: "intim", base: (10.0, 18.0), exp: 45,
        likes: &["dandere", "tsundere", "yandere"],
        dislikes: &[],
        text: "😚 Kamu mengecup keningnya. *{n}* memejamkan mata bahagia." },
    ActionDef { key: "cium_bibir", phase: "intim", base: (14.0, 24.0), exp: 55,
        likes: &["yandere", "playboy"],
        dislikes: &["kuudere"],
        text: "💋 Ciuman lembut dan panjang. *{n}* membalas dengan desahan kecil." },
    ActionDef { key: "tidur_kelon", phase: "intim", base: (10.0, 16.0), exp: 45,
        likes: &["femboy", "oji-san"],
        dislikes: &[],
        text: "🛏️ Kamu merebahkan kepala di dada *{n}* yang bidang. Dia mengusap rambutmu pelan." },
    ActionDef { key: "tidur_serandu", phase: "intim", base: (12.0, 20.0), exp: 50,
        likes: &["oji-san", "femboy", "yandere"],
        dislikes: &[],
        text: "🌙 Kalian berpelukan erat di ranjang. *{n}* berbisik bahwa dia merasa aman." },
    ActionDef { key: "mandi_punggung", phase: "intim", base: (10.0, 16.0), exp: 45,
        likes: &["oji-san", "kuudere"],
        dislikes: &[],
        text: "🛁 Kamu membasuh punggung *{n}* dengan lembut. Dia mendesah rileks." },
    ActionDef { key: "mandi_bahu", phase: "intim", base: (12.0, 20.0), exp: 50,
        likes: &["playboy", "oji-san"],
        dislikes: &[],
        text: "🧼 *{n}* menunduk malu saat kamu mengusap bahunya di air hangat." },
    ActionDef { key: "gendong_putri", phase: "intim", base: (10.0, 18.0), exp: 45,
        likes: &["playboy", "yandere", "tsundere"],
        dislikes: &[],
        text: "👸 *{n}* menggendongmu dengan gaya putri. Kamu memeluk lehernya erat." },
    ActionDef { key: "gendong_punggung", phase: "intim", base: (8.0, 14.0), exp: 40,
        likes: &["genki", "sunao"],
        dislikes: &[],
        text: "🏃 *{n}* menggendongmu di punggung sambil tertawa-tawa." },
    ActionDef { key: "tepuk_kepala", phase: "intim", base: (6.0, 12.0), exp: 35,
        likes: &["dandere", "tsundere"],
        dislikes: &[],
        text: "🖐️ Kamu menepuk kepala *{n}* pelan. Dia mendengus tapi pipinya memerah." },
    ActionDef { key: "pijat_bahu", phase: "intim", base: (10.0, 18.0), exp: 45,
        likes: &["oji-san", "yandere", "playboy"],
        dislikes: &[],
        text: "💆 Kamu memijat bahu *{n}* yang tegang. Dia mendesah rileks." },
    ActionDef { key: "mesra", phase: "married", base: (6.0, 12.0), exp: 35,
        likes: &["playboy", "oji-san"],
        dislikes: &[],
        text: "👩‍❤️‍👨 Kalian menghabiskan malam dengan mesra sebagai pasangan suami istri." },
    ActionDef { key: "rayu", phase: "married", base: (8.0, 14.0), exp: 40,
        likes: &["tsundere", "playboy"],
        dislikes: &[],
        text: "💌 Kata-kata rayuanmu membuat *{n}* tersipu dan memelukmu." },
    ActionDef { key: "hadiah", phase: "married", base: (10.0, 16.0), exp: 45,
        likes: &["playboy", "prince"],
        dislikes: &[],
        text: "🎁 Hadiah kejutan untuk suami tercinta. *{n}* tersenyum bangga." },
    ActionDef { key: "bulanmadu_pantai", phase: "married", base: (12.0, 20.0), exp: 50,
        likes: &["genki", "sunao"],
        dislikes: &[],
        text: "🌴 Bulan madu di pantai tropis. *{n}* menggandengmu menikmati sunset." },
    ActionDef { key: "bulanmadu_hotel", phase: "married", base: (14.0, 24.0), exp: 55,
        likes: &["oji-san", "yandere"],
        dislikes: &[],
        text: "🏨 Suite hotel mewah. Malam penuh kehangatan berdua saja." },
    ActionDef { key: "nontonrumah", phase: "married", base: (8.0, 14.0), exp: 40,
        likes: &["playboy", "dandere", "tsundere"],
        dislikes: &[],
        text: "📺 Nonton maraton film di sofa. *{n}* nyender di bahumu sampai ketiduran." },
    ActionDef { key: "jalanpagi", phase: "married", base: (10.0, 16.0), exp: 45,
        likes: &["genki", "oji-san", "sunao"],
        dislikes: &[],
        text: "🌅 Jalan santai pagi hari sambil beli sarapan. *{n}* menggandengmu." },
];

/// `HUSB_PERSONALITY_TAG` (`src/lib/husbu.js:7-19`).
const PERSONALITY_TAG: [(&str, &[&str], &[&str]); 11] = [
    ("tsundere", &["jalan_taman", "piknik_taman"], &["rayu"]),
    ("playboy", &["belanja_perhiasan", "cium_bibir", "mesra"], &["olahraga_panjat"]),
    ("kuudere", &["kafe_kopi", "bioskop_romantis", "piknik_taman"], &["arcade_duo"]),
    ("dandere", &["kafe_matcha", "masak_bareng"], &["jalan_mall"]),
    ("yandere", &["peluk_belakang", "cium_kening", "cium_bibir"], &["jalan_mall"]),
    ("oji-san", &["masak_bareng", "tidur_kelon", "alam_mancing"], &[]),
    ("genki", &["karaoke_duet", "arcade_duo", "piknik_pantai"], &["kafe_kopi"]),
    ("prince", &["belanja_perhiasan", "hadiah"], &["jalan_pantai"]),
    ("badboy", &["olahraga_panjat", "bioskop_horor"], &["piknik_taman"]),
    ("sunao", &["jalan_pantai", "piknik_pantai", "masak_bareng"], &["karaoke_solo"]),
    ("femboy", &["kafe_kue", "kafe_susu", "piknik_taman", "belanja_boneka", "tidur_kelon"], &["olahraga_panjat", "bioskop_horor"]),
];

/// `HUSB_EVENTS` (`src/lib/husbu.js:82-103`) — 20 event.
const EVENTS: &[EventDef] = &[
    EventDef { id: "rain", phase: "any", married_only: false,
        aff: EventAff::Fixed(5.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🌧️ Hujan deras turun! *{n}* berbagi jaket denganmu, bahu menempel bahu." },
    EventDef { id: "wallet", phase: "any", married_only: false,
        aff: EventAff::Fixed(0.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: Some(EventKoin::Wallet),
        text: "💸 Kamu menemukan dompet di jalan dan mengembalikannya. Rezeki mengalir!" },
    EventDef { id: "rival", phase: "any", married_only: false,
        aff: EventAff::Fixed(-4.0), yandere_aff: 5.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "😠 Seseorang mendekatimu! *{n}* gelisah dan mood-nya turun." },
    EventDef { id: "idol", phase: "any", married_only: false,
        aff: EventAff::Fixed(6.0), yandere_aff: 0.0, mood: Some("ceria"), next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🎮 Toko game favoritnya buka! *{n}* semangat dan mood-nya naik." },
    EventDef { id: "cat", phase: "any", married_only: false,
        aff: EventAff::Cat, yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🐱 Seekor kucing lucu tersangkut di pohon. Kalian menyelamatkannya!" },
    EventDef { id: "lottery", phase: "any", married_only: false,
        aff: EventAff::Fixed(0.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: Some(EventKoin::Lottery),
        text: "🎰 Tiket lotre jatuh dari langit! Kamu coba keberuntunganmu..." },
    EventDef { id: "late", phase: "any", married_only: false,
        aff: EventAff::Fixed(0.0), yandere_aff: 0.0, mood: Some("sedih"), next_mult: Some(0.8), anger: 0.0,
        koin: None,
        text: "📉 Kamu hampir telat janji temu! *{n}* cemberut sepanjang hari." },
    EventDef { id: "anniv", phase: "any", married_only: true,
        aff: EventAff::Fixed(6.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "💍 Kenangan hari pernikahan kalian teringat kembali. Kalian terharu bersama." },
    EventDef { id: "intimate", phase: "intim", married_only: false,
        aff: EventAff::Fixed(10.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🔥 Momen kalian berlanjut. *{n}* berbisik pelan, \"jangan berhenti...\"" },
    EventDef { id: "rainbow", phase: "approach", married_only: false,
        aff: EventAff::Fixed(5.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🌈 Pelangi muncul setelah hujan reda. Kalian berhenti untuk mengabadikannya." },
    EventDef { id: "confess", phase: "intim", married_only: false,
        aff: EventAff::Fixed(8.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "💞 *{n}* mengaku bahwa dia mulai benar-benar mencintaimu." },
    EventDef { id: "gift", phase: "married", married_only: true,
        aff: EventAff::Fixed(0.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: Some(EventKoin::Gift),
        text: "🎁 Hadiah kejutan kecil untukmu sebagai pasangan." },
    EventDef { id: "foto", phase: "any", married_only: false,
        aff: EventAff::Fixed(4.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "📸 Foto bareng! *{n}* melotot saat kejepret momen memalukan." },
    EventDef { id: "lomba", phase: "any", married_only: false,
        aff: EventAff::Fixed(0.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: Some(EventKoin::Lomba),
        text: "🏅 Kamu ikut lomba jalan santai dan menang hadiah receh!" },
    EventDef { id: "salahpaham", phase: "any", married_only: false,
        aff: EventAff::Fixed(-5.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 12.0,
        koin: None,
        text: "📵 *{n}* melihat notif dari akun lain di ponselmu dan salah paham!" },
    EventDef { id: "pujian", phase: "any", married_only: false,
        aff: EventAff::Fixed(5.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🗣️ Seseorang memuji kalian: \"cocok banget!\" *{n}* tersipu." },
    EventDef { id: "sunset", phase: "any", married_only: false,
        aff: EventAff::Fixed(6.0), yandere_aff: 0.0, mood: Some("romantis"), next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "🌇 Kalian berhenti menikmati matahari terbenam. Momen tak terlupakan." },
    EventDef { id: "kenangan", phase: "any", married_only: true,
        aff: EventAff::Fixed(7.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "📿 Kamu menemukan foto lama kalian. *{n}* terharu dan memelukmu erat." },
    EventDef { id: "badai", phase: "any", married_only: false,
        aff: EventAff::Fixed(-4.0), yandere_aff: 0.0, mood: Some("marah"), next_mult: Some(1.0), anger: 0.0,
        koin: None,
        text: "⛈️ Hujan badai datang! *{n}* kehujanan dan mood-nya rusak." },
    EventDef { id: "rezeki", phase: "any", married_only: false,
        aff: EventAff::Fixed(0.0), yandere_aff: 0.0, mood: None, next_mult: Some(1.0), anger: 0.0,
        koin: Some(EventKoin::Rezeki),
        text: "🍀 Rezeki nomplok! Kamu dapat uang tak terduga." },
];

/// `MENUS` (`gachahusbu.js:298-322`).
const MENUS: [(&str, &str, &[(&str, &str)]); 19] = [
    (
        "menu_jalanjalan",
        "🚶 Jalan-jalan ke...",
        &[
            ("🌳 Taman", "jalan_taman"),
            ("🏢 Mall", "jalan_mall"),
            ("🏖️ Pantai", "jalan_pantai"),
            ("🌆 Kota", "jalan_kota"),
        ],
    ),
    (
        "menu_kafe",
        "☕ Kamu traktir dia...",
        &[
            ("☕ Kopi Pahit", "kafe_kopi"),
            ("🍵 Matcha Latte", "kafe_matcha"),
            ("🍰 Kue", "kafe_kue"),
            ("🥛 Susu Hangat", "kafe_susu"),
        ],
    ),
    (
        "menu_bioskop",
        "🎬 Pilih genre film...",
        &[
            ("💥 Action", "bioskop_romantis"),
            ("👻 Horor", "bioskop_horor"),
            ("🎨 Animasi", "bioskop_animasi"),
        ],
    ),
    (
        "menu_belanja",
        "🛍️ Minta dibeliin apa?..",
        &[
            ("👕 Baju Keren", "belanja_baju"),
            ("💍 Aksesoris", "belanja_perhiasan"),
            ("🧸 Boneka", "belanja_boneka"),
        ],
    ),
    (
        "menu_kuliner",
        "🍽️ Makan di...",
        &[
            ("🍽️ Restoran", "restoran_makan"),
            ("🥟 Dimsum", "restoran_dimsum"),
            ("🍖 BBQ", "restoran_bbq"),
        ],
    ),
    (
        "menu_olahraga",
        "⚽ Olahraga...",
        &[
            ("⛰️ Hiking", "olahraga_hiking"),
            ("🏃 Lari Pagi", "olahraga_lari"),
            ("🧗 Panjat", "olahraga_panjat"),
        ],
    ),
    (
        "menu_alam",
        "🏕️ Petualangan alam...",
        &[
            ("🏕️ Camping", "alam_camping"),
            ("🎣 Mancing", "alam_mancing"),
            ("⛵ Perahu", "alam_perahu"),
        ],
    ),
    (
        "menu_seni",
        "🎨 Seni & budaya...",
        &[
            ("🖼️ Museum", "seni_museum"),
            ("🎨 Melukis", "seni_melukis"),
            ("🎸 Konser", "seni_konser"),
        ],
    ),
    (
        "menu_karaoke",
        "🎤 Karaoke...",
        &[("🎶 Duet", "karaoke_duet"), ("🎵 Solo", "karaoke_solo")],
    ),
    (
        "menu_arcade",
        "🕹️ Di arcade...",
        &[
            ("🎮 Adu Skor", "arcade_duo"),
            ("🎯 Main Boneka", "arcade_boneka"),
        ],
    ),
    (
        "menu_piknik",
        "🧺 Piknik...",
        &[
            ("🌳 Di Taman", "piknik_taman"),
            ("🏝️ Di Pantai", "piknik_pantai"),
        ],
    ),
    (
        "menu_masak",
        "🍳 Masak bareng...",
        &[("🍳 Masakan Rumah", "masak_bareng"), ("🍰 Kue", "masak_kue")],
    ),
    ("menu_lainnya", "🎮 Aksi lainnya...", &[]),
    (
        "menu_peluk",
        "🫂 Cara memeluk...",
        &[
            ("🤗 Dari Belakang", "peluk_belakang"),
            ("💑 Berhadapan", "peluk_depan"),
        ],
    ),
    (
        "menu_cium",
        "💋 Cium di...",
        &[("😚 Kening", "cium_kening"), ("💋 Bibir", "cium_bibir")],
    ),
    (
        "menu_tidur",
        "🛏️ Tidur bareng...",
        &[("🫂 Pulas", "tidur_kelon"), ("🌙 Serandu", "tidur_serandu")],
    ),
    (
        "menu_mandi",
        "🛁 Mandi bareng...",
        &[
            ("🛁 Gosok Punggung", "mandi_punggung"),
            ("🧼 Usap Bahu", "mandi_bahu"),
        ],
    ),
    (
        "menu_gendong",
        "🤗 Gendong...",
        &[
            ("👸 Gaya Putri", "gendong_putri"),
            ("🏃 Punggung", "gendong_punggung"),
        ],
    ),
    (
        "menu_bulanmadu",
        "🌴 Bulan madu...",
        &[
            ("🏝️ Pantai", "bulanmadu_pantai"),
            ("🏨 Hotel", "bulanmadu_hotel"),
        ],
    ),
];

/// Daftar pemicu tamparan (`gachahusbu.js:329`).
const MESUM_ACTIONS: [&str; 15] = [
    "peluk",
    "cium",
    "tidur",
    "mandi",
    "gendong_putri",
    "gendong_punggung",
    "tepuk_kepala",
    "intim_belakang",
    "intim_depan",
    "intim_kening",
    "intim_bibir",
    "intim_kelon",
    "intim_panas",
    "intim_punggung",
    "intim_bahu",
];

/// Footer kartu husbu (`gachahusbu.js:123-125`).
const FOOTER_DEFAULT: &str = "❤️ Buat dia luluh dan jatuh cinta padamu!";
const FOOTER_MENU: &str = "💭 Dia menunggu jawabanmu...";
const FOOTER_MARRIED: &str = "❤️ Kamu adalah istrinya yang sah!";
const FOOTER_PANDUAN: &str = "Tekan tombol untuk mulai!";

/// Hasil satu aksi (`applyAction`).
struct ActionResult {
    phase: &'static str,
    change: i64,
    exp: i64,
    text: String,
    like: bool,
    dislike: bool,
}

/// `EventAff` — nilai affection per event.
enum EventAff {
    Fixed(f64),
    /// `aff: (p) => (p === "playboy" || p === "genki" ? 9 : 4)`.
    Cat,
}

/// Sumber koin acak per event (`koin: () => ...`).
enum EventKoin {
    Wallet,
    Lottery,
    Gift,
    Lomba,
    Rezeki,
}

/// Satu event `HUSB_EVENTS`.
struct EventDef {
    id: &'static str,
    phase: &'static str,
    married_only: bool,
    aff: EventAff,
    yandere_aff: f64,
    mood: Option<&'static str>,
    next_mult: Option<f64>,
    anger: f64,
    koin: Option<EventKoin>,
    text: &'static str,
}

/// Hasil satu undian event (`rollEvent`).
struct EventRoll {
    text: String,
    aff: f64,
    koin: i64,
    mood: Option<&'static str>,
    next_mult: f64,
    anger: f64,
}

/// `husbu_' + nama tanpa spasi + '@s.whatsapp.net'` (`gachahusbu.js:214`).
fn husbu_jid(name: &str) -> String {
    let clean: String = name.chars().filter(|c| !c.is_whitespace()).collect();
    format!("husbu_{clean}@s.whatsapp.net")
}

/// `buildPool` (`data/husbu/index.js:47-77`) — validasi, dedupe, trim, bobot.
fn build_pool() -> Vec<crate::fun::gachawaifu::Waifu> {
    let mut seen: Vec<String> = Vec::new();
    let mut all: Vec<crate::fun::gachawaifu::Waifu> = Vec::new();
    for w in HUSBU_ENTRIES {
        if w.name.is_empty()
            || w.series.is_empty()
            || w.age == 0
            || w.height.is_empty()
            || w.weight.is_empty()
            || w.tier.is_empty()
            || w.personality.is_empty()
        {
            continue;
        }
        if !TIER_WEIGHTS.iter().any(|(t, _)| *t == w.tier) {
            continue;
        }
        if !VALID_HUSB_PERSONALITIES.contains(&w.personality) {
            continue;
        }
        let name = w.name.trim().to_string();
        let key = name.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let series = w.series.trim().to_string();
        let keyword = w
            .keyword
            .filter(|k| !k.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("{name} {series} anime"));
        all.push(crate::fun::gachawaifu::Waifu {
            name,
            series,
            tier: w.tier.to_string(),
            personality: w.personality.to_string(),
            keyword,
            age: w.age,
            height: w.height.to_string(),
            weight: w.weight.to_string(),
            roll_weight: 0.0,
        });
    }
    let mut tier_count: Vec<(String, f64)> = Vec::new();
    for w in &all {
        match tier_count.iter_mut().find(|(t, _)| *t == w.tier) {
            Some((_, n)) => *n += 1.0,
            None => tier_count.push((w.tier.clone(), 1.0)),
        }
    }
    for w in &mut all {
        let weight = TIER_WEIGHTS
            .iter()
            .find(|(t, _)| *t == w.tier)
            .map_or(0.0, |(_, w0)| *w0);
        let count = tier_count
            .iter()
            .find(|(t, _)| *t == w.tier)
            .map_or(1.0, |(_, c)| *c);
        w.roll_weight = weight / count;
    }
    all
}

/// `rollHusbu` (`src/lib/husbu.js:105-115`).
fn roll_husbu<'a>(
    pool: &'a [crate::fun::gachawaifu::Waifu],
    pity_counter: i64,
    rng: &mut dyn FnMut() -> f64,
) -> Option<&'a crate::fun::gachawaifu::Waifu> {
    let eligible: Vec<&crate::fun::gachawaifu::Waifu> = if pity_counter >= PITY_THRESHOLD {
        pool.iter()
            .filter(|w| tier_index(&w.tier) >= 2)
            .collect()
    } else {
        pool.iter().collect()
    };
    let total: f64 = eligible.iter().map(|w| w.roll_weight).sum();
    let mut r = rng() * total;
    for w in &eligible {
        r -= w.roll_weight;
        if r <= 0.0 {
            return Some(w);
        }
    }
    eligible.last().copied()
}

/// `applyAction` (`src/lib/husbu.js:117-131`).
fn apply_action(
    key: &str,
    husbu: &Map<String, Value>,
    mood: &str,
    mult_override: f64,
    rng: &mut dyn FnMut() -> f64,
) -> Option<ActionResult> {
    let action = HUSB_ACTIONS.iter().find(|a| a.key == key)?;
    if !js_truthy(husbu.get("personality")) {
        return None;
    }
    let personality = mstr(husbu.get("personality")).unwrap_or_default();
    let name = mstr(husbu.get("name")).unwrap_or_default();

    let (min, max) = action.base;
    let base = min + rng() * (max - min);
    let mut mult = MOOD_MULT
        .iter()
        .find(|(m, _)| *m == mood)
        .map_or(1.0, |(_, v)| *v);
    mult *= mult_override;

    let tag = PERSONALITY_TAG.iter().find(|(p, _, _)| *p == personality);
    let (tag_like, tag_dislike) = tag.map_or((&[][..], &[][..]), |(_, l, d)| (&l[..], &d[..]));
    let liked = action.likes.contains(&personality.as_str()) || tag_like.contains(&key);
    let disliked =
        action.dislikes.contains(&personality.as_str()) || tag_dislike.contains(&key);
    if liked {
        mult *= 1.2;
    } else if disliked {
        mult *= 0.7;
    }

    Some(ActionResult {
        phase: action.phase,
        change: (crate::fun::gachawaifu::js_round(base * mult) as i64).max(1),
        exp: action.exp,
        text: action.text.replace("{n}", &name),
        like: liked,
        dislike: disliked,
    })
}

/// `rollEvent` (`src/lib/husbu.js:133-145`).
fn roll_event(
    married: bool,
    phase: &str,
    personality: &str,
    name: &str,
    rng: &mut dyn FnMut() -> f64,
) -> Option<EventRoll> {
    if rng() > EVENT_CHANCE {
        return None;
    }
    let pool: Vec<&EventDef> = EVENTS
        .iter()
        .filter(|e| {
            if e.married_only && !married {
                return false;
            }
            e.phase == "any" || e.phase == phase
        })
        .collect();
    if pool.is_empty() {
        return None;
    }
    let e = pool[(rng() * pool.len() as f64).floor() as usize % pool.len()];
    let mut aff = match e.aff {
        EventAff::Fixed(v) => v,
        EventAff::Cat => {
            if personality == "playboy" || personality == "genki" {
                9.0
            } else {
                4.0
            }
        }
    };
    if e.id == "rival" && personality == "yandere" {
        aff = e.yandere_aff;
    }
    let koin = match e.koin {
        None => 0,
        Some(EventKoin::Wallet) => 1000 + (rng() * 24001.0).floor() as i64,
        Some(EventKoin::Lottery) => {
            if rng() < 0.5 {
                20000 + (rng() * 40001.0).floor() as i64
            } else {
                0
            }
        }
        Some(EventKoin::Gift) => 2000 + (rng() * 9001.0).floor() as i64,
        Some(EventKoin::Lomba) => 500 + (rng() * 5001.0).floor() as i64,
        Some(EventKoin::Rezeki) => 3000 + (rng() * 8001.0).floor() as i64,
    };
    Some(EventRoll {
        text: e.text.replace("{n}", name),
        aff,
        koin,
        mood: e.mood,
        next_mult: e.next_mult.unwrap_or(1.0),
        anger: e.anger,
    })
}

/// Opsi `menu_lainnya` — bergantung pada `affection` (`gachahusbu.js:311-315`).
fn menu_lainnya_options(affection: f64) -> &'static [(&'static str, &'static str)] {
    if affection < 80.0 {
        &[
            ("🎤 Karaoke", "menu_karaoke"),
            ("🕹️ Arcade", "menu_arcade"),
            ("🧺 Piknik", "menu_piknik"),
            ("🍳 Masak", "menu_masak"),
            ("🍽️ Kuliner", "menu_kuliner"),
            ("⚽ Olahraga", "menu_olahraga"),
            ("🏕️ Alam", "menu_alam"),
            ("🎨 Seni", "menu_seni"),
        ]
    } else if affection < 100.0 {
        &[
            ("🤗 Gendong", "menu_gendong"),
            ("🖐️ Tepuk Kepala", "tepuk_kepala"),
        ]
    } else {
        &[
            ("🌴 Bulan Madu", "menu_bulanmadu"),
            ("💝 Hadiah", "hadiah"),
        ]
    }
}

/// Tombol aksi utama per fase affection (`gachahusbu.js:93-121`).
fn tombol_utama(affection: f64, married: bool) -> &'static [(&'static str, &'static str)] {
    if affection < 80.0 {
        &[
            ("🚶 Jalan-jalan", "menu_jalanjalan"),
            ("☕ Kafe", "menu_kafe"),
            ("🎬 Bioskop", "menu_bioskop"),
            ("🛍️ Belanja", "menu_belanja"),
            ("🎮 Lainnya", "menu_lainnya"),
        ]
    } else if affection < 100.0 {
        &[
            ("🫂 Peluk", "menu_peluk"),
            ("💋 Cium", "menu_cium"),
            ("🛏️ Tidur", "menu_tidur"),
            ("🛁 Mandi", "menu_mandi"),
            ("🎁 Lainnya", "menu_lainnya"),
        ]
    } else if !married {
        &[
            ("💍 Terima Lamaran", "nikah"),
            ("💝 Hadiah", "hadiah"),
        ]
    } else {
        &[
            ("👩‍❤️‍👨 Mesra", "mesra"),
            ("💌 Rayu", "rayu"),
            ("🌴 Bulan Madu", "menu_bulanmadu"),
            ("🎁 Klaim Harian", "klaim"),
        ]
    }
}

/// Rakit kartu interaktif husbu — padanan `sendHusbuMessage`
/// (`gachahusbu.js:68-147`).
fn kartu_husbu(
    ctx: &Ctx,
    h: &Map<String, Value>,
    body: &str,
    custom: Option<&[(&str, &str)]>,
) -> Reply {
    let tier = js_str(h.get("tier"));
    let nama = js_str(h.get("name"));
    let married = js_truthy(h.get("married"));
    let affection = mnum_or(h.get("affection"), 50.0);

    let (footer, opsi): (&str, Vec<(&str, &str)>) = match custom {
        Some(list) => (FOOTER_MENU, list.to_vec()),
        None => (
            if married {
                FOOTER_MARRIED
            } else {
                FOOTER_DEFAULT
            },
            tombol_utama(affection, married).to_vec(),
        ),
    };

    let buttons = opsi
        .into_iter()
        .map(|(label, id)| {
            InteractiveButton::quick_reply(label, format!("{}husbuaction {id}", ctx.prefix))
        })
        .collect();

    let header = CardHeader::new()
        .title(format!("🌟 *{} TIER HUSBANDO* 🌟", tier.to_uppercase()))
        .subtitle(nama.clone());
    let header = match h.get("imageUrl").and_then(Value::as_str) {
        Some(url) if !url.is_empty() => header.image(MediaSource::url(url)),
        _ => header,
    };

    Reply::interactive(body)
        .with_footer(footer)
        .with_header(header)
        .with_buttons(buttons)
}

/// `marrageDay` (`gachahusbu.js:149-154`).
fn hari_pernikahan(h: &Map<String, Value>, now: i64) -> i64 {
    marrage_day(mstr(h.get("marriedDate")).as_deref(), now)
}

/// `moodState` (`gachahusbu.js:56-60`).
fn mood_state_husbu(_user: &Map<String, Value>, h: &Map<String, Value>, sender: &str, now: i64) -> String {
    if let Some(until) = mstr(h.get("moodUntil")).and_then(|s| iso_to_ms(&s))
        && now < until
        && let Some(mood) = map_of(h.get("mood")).get("type").and_then(Value::as_str)
    {
        return mood.to_string();
    }
    get_daily_mood(sender, &today_str(now)).to_string()
}

/// Patch `SetUser` minimal yang menulis balik objek `husbu`.
fn husbu_patch(h: &Map<String, Value>) -> Map<String, Value> {
    let mut patch = Map::new();
    patch.insert("husbu".into(), Value::Object(h.clone()));
    patch
}

/// `balas` — teks + mutasi + reaksi.
fn balas(jid: &str, text: String, patch: Option<Map<String, Value>>, reaction: Option<&str>) -> Reply {
    let reply = match patch {
        Some(p) => Reply::text(text).with_mutation(set_user(jid, p)),
        None => Reply::text(text),
    };
    match reaction {
        Some(emoji) => reply.with_reaction(emoji),
        None => reply,
    }
}

/// `gachahusbu.js:233-241` — panduan.
fn guide(prefix: &str) -> String {
    format!(
        "💕 *SISTEM GACHA HUSBU* 💕\n\n\
         Simulasi kencan virtual interaktif untuk mendapatkan laki-laki anime idamanmu! \
         Tarik perhatiannya, buat dia jatuh cinta, dan nikahi dia!\n\n\
         *PENGGUNAAN:*\n\
         • *{prefix}gachahusbu* — Panduan ini\n\
         • *{prefix}husbuku* — Panel interaksi\n\
         • *{prefix}husbualbum* — Riwayat & statistik luck\n\
         • *{prefix}husbupool* — Jelajahi pool husbu\n\
         • *{prefix}tinggalinhusbu* — Putuskan hubungan\n\n\
         *ALUR:*\n1. Tekan tombol **Panggil Husbando**.\n\
         2. 3 Fase: Pendekatan (<80) → Intim (80–99) → Menikah (100).\n\
         3. Mood, personality & anger memengaruhi poin affection.\n\
         4. Pity: 20 roll tanpa Epic+ dijamin dapat Epic+.\n\
         5. Affection makin sulit ditambah makin tinggi + interaksi berulang menyusut."
    )
}

/// Cabang `husbuku`/`suamiku` (`gachahusbu.js:183-209`).
fn status_husbu(ctx: &Ctx, user: &Map<String, Value>) -> Reply {
    let Some(h_value) = user.get("husbu").filter(|v| js_truthy(Some(*v))) else {
        return Reply::text(format!(
            "⚠️ *Kamu belum memiliki husbu!*\nSilakan ketik *{}gachahusbu* untuk memulainya!",
            ctx.prefix
        ));
    };
    let h = map_of(Some(h_value));
    let now = now_ms();
    let mood = mood_state_husbu(user, &h, &ctx.sender, now);
    let decay = apply_neglect(&mut h.clone(), now);
    let status = if js_truthy(h.get("married")) {
        "Telah Menikah 💍"
    } else {
        "Pendekatan 💖"
    };
    let day = hari_pernikahan(&h, now);
    let mood_line = if js_truthy(h.get("married")) {
        format!(
            "\n🗓️ *Hari ke-{day}*{}",
            if day >= 7 {
                if day >= 100 {
                    " 🏆"
                } else if day >= 30 {
                    " 🎖️"
                } else {
                    " 🎉"
                }
            } else {
                ""
            }
        )
    } else {
        String::new()
    };
    let mut text = format!(
        "📸 *STATUS HUSBU KAMU* 📸\n\n\
         💖 *Nama:* {}\n\
         💎 *Tier:* {}\n\
         🎭 *Personality:* {}\n\
         🌤️ *Mood hari ini:* {}\n\
         😡 *Anger:* {}\n\
         💞 *Affection:* {}/100\n\
         💍 *Status:* {}{mood_line}\n",
        js_str(h.get("name")),
        js_str(h.get("tier")),
        js_str(h.get("personality")),
        mood_label(&mood),
        anger_meter(&h),
        js_str(h.get("affection")),
        status
    );
    if decay > 0 {
        text.push_str(&format!(
            "📉 *Neglect:* Affection turun -{decay} karena kamu jarang interaksi!\n"
        ));
    }
    if mnum_or(h.get("anger"), 0.0) >= 50.0 {
        text.push_str("⚠️ *Dia sedang MARAH!* Perbaiki hubungan sebelum affection habis!\n");
    }
    text.push_str("\nLanjutkan interaksi dengan memilih aksi di bawah!");

    let mut patch = Map::new();
    patch.insert("husbu".into(), Value::Object(h.clone()));
    kartu_husbu(ctx, &h, &text, None)
        .with_mutation(set_user(&ctx.sender, patch))
        .with_reaction("✅")
}

/// Cabang `tinggalinhusbu` (`gachahusbu.js:211-223`).
fn tinggalin_husbu(ctx: &Ctx, user: &Map<String, Value>) -> Reply {
    let Some(h_value) = user.get("husbu").filter(|v| js_truthy(Some(*v))) else {
        return Reply::text("⚠️ *Kamu bahkan belum punya husbu!* Cari dulu gih!");
    };
    let h = map_of(Some(h_value));
    let nama = js_str(h.get("name"));
    let jid = husbu_jid(&nama);
    let mut mutations: Vec<Mutation> = Vec::new();
    let mut patch = Map::new();
    if js_truthy(h.get("married")) {
        if map_of(user.get("fun")).get("pasangan").and_then(Value::as_str) == Some(jid.as_str()) {
            let mut fun = map_of(user.get("fun"));
            fun.insert("pasangan".into(), Value::from(""));
            patch.insert("fun".into(), Value::Object(fun));
        }
        let mut fun = Map::new();
        fun.insert("pasangan".into(), Value::from(""));
        let mut data = Map::new();
        data.insert("fun".into(), Value::Object(fun));
        mutations.push(set_user(&jid, data));
    }
    patch.insert("husbu".into(), Value::Null);
    mutations.push(set_user(&ctx.sender, patch));

    let text = format!(
        "💔 *KAMU MENCAMPAKKAN {}!*\n\n\
         Kamu mengembalikan barang-barangnya dan memintanya untuk pergi. \
         Dia menatapmu dengan mata kecewa yang mendalam, berbalik tanpa sepatah kata pun, \
         lalu menghilang di tengah hujan.\n\n\
         Kalian kini resmi berpisah.",
        nama.to_uppercase()
    );
    Reply::text(text).mutating(mutations).with_reaction("💔")
}

/// Cabang `gachahusbu`/`gachasuami` (`gachahusbu.js:225-282`).
fn gacha_husbu(ctx: &Ctx, user: &Map<String, Value>, rng: &mut dyn FnMut() -> f64) -> Reply {
    if let Some(h_value) = user.get("husbu").filter(|v| js_truthy(Some(*v))) {
        let h = map_of(Some(h_value));
        let pesan_status = if js_truthy(h.get("married")) {
            "Dia sudah menjadi suamimu!"
        } else {
            "Dia sedang berusaha melulukkan hatimu!"
        };
        return balas(
            &ctx.sender,
            format!(
                "⚠️ *Kamu sudah memiliki Husbu!*\n\nNama: *{}*\nTier: *{}*\nAffection: *{}/100*\n\n\
                 Jangan serakah! Jaga husbu yang kamu miliki sekarang. {} \
                 Ketik *{}husbuku* untuk berinteraksi dengannya.",
                js_str(h.get("name")),
                js_str(h.get("tier")),
                js_str(h.get("affection")),
                pesan_status,
                ctx.prefix
            ),
            None,
            Some("😡"),
        );
    }
    let sub = ctx.args.first().map(|a| a.to_lowercase()).unwrap_or_default();
    if sub != "start" {
        return Reply::interactive(guide(&ctx.prefix))
            .with_footer(FOOTER_PANDUAN)
            .with_button(InteractiveButton::quick_reply(
                "🎲 Panggil Husbando!",
                format!("{}gachahusbu start", ctx.prefix),
            ));
    }

    let mut user = user.clone();
    let stats = init_stats(&mut user);
    let pool = build_pool();
    let pity = mnum_or(stats.get("pityCounter"), 0.0) as i64;
    let Some(picked) = roll_husbu(&pool, pity, rng) else {
        // JS: `picked.keyword` melempar → `catch` → `te()` + reaksi ☢.
        return Reply::text(error_template(ctx)).with_reaction("☢");
    };
    let now = now_ms();
    let mut h = Map::new();
    h.insert("name".into(), Value::from(picked.name.clone()));
    h.insert("series".into(), Value::from(picked.series.clone()));
    h.insert("tier".into(), Value::from(picked.tier.clone()));
    h.insert("personality".into(), Value::from(picked.personality.clone()));
    h.insert("keyword".into(), Value::from(picked.keyword.clone()));
    h.insert("age".into(), jnum(picked.age as f64));
    h.insert("height".into(), Value::from(picked.height.clone()));
    h.insert("weight".into(), Value::from(picked.weight.clone()));
    h.insert("affection".into(), jnum(50.0));
    h.insert("married".into(), Value::Bool(false));
    h.insert("imageUrl".into(), Value::from(IMAGE_FALLBACK));
    h.insert("rollWeight".into(), Value::from(picked.roll_weight));

    let mut stats = stats;
    record_pull(
        &mut stats,
        &mut user,
        &picked.name,
        &picked.series,
        &picked.tier,
        &iso_now(now),
    );

    let mut patch = Map::new();
    patch.insert("husbu".into(), Value::Object(h.clone()));
    patch.insert("husbuStats".into(), Value::Object(stats.clone()));
    if let Some(history) = user.get("husbuHistory") {
        patch.insert("husbuHistory".into(), history.clone());
    }

    let text = format!(
        "🎉 *KYAAA! KAMU MENDAPATKAN HUSBANDO BARU!* 🎉\n\n\
         💖 *Nama:* {}\n🎂 *Usia:* {} tahun\n📏 *Tinggi:* {}\n⚖️ *Berat:* {}\n\
         💎 *Tier:* {}\n🎭 *Personality:* {}\n💞 *Affection:* {}/100\n\n\
         Pilih interaksi untuk mulai PDKT. Hati-hati jangan sampai affection habis!",
        js_str(h.get("name")),
        js_str(h.get("age")),
        js_str(h.get("height")),
        js_str(h.get("weight")),
        js_str(h.get("tier")),
        js_str(h.get("personality")),
        js_str(h.get("affection")),
    );
    kartu_husbu(ctx, &h, &text, None)
        .with_mutation(set_user(&ctx.sender, patch))
        .with_reaction("✅")
}

/// Cabang `husbuaction` (`gachahusbu.js:284-467`).
fn husbu_action(ctx: &Ctx, user: &Map<String, Value>, rng: &mut dyn FnMut() -> f64) -> Reply {
    let Some(h_value) = user.get("husbu").filter(|v| js_truthy(Some(*v))) else {
        return balas(
            &ctx.sender,
            format!(
                "Kamu belum memiliki husbu! Ketik *{}gachahusbu* untuk memulai.",
                ctx.prefix
            ),
            None,
            Some("❌"),
        );
    };
    let action = ctx.args.first().map(|a| a.to_lowercase()).unwrap_or_default();
    let mut h = map_of(Some(h_value));
    let now = now_ms();

    // Backfill personality (`gachahusbu.js:289-295`).
    let mut dirty = false;
    if !js_truthy(h.get("personality")) {
        let name = mstr(h.get("name")).unwrap_or_default();
        let mut hash: u32 = 0;
        for c in name.chars() {
            hash = hash.wrapping_mul(31).wrapping_add(c as u32);
        }
        let personality = HUSB_PERSONALITIES[(hash as usize) % HUSB_PERSONALITIES.len()];
        h.insert("personality".into(), Value::from(personality));
        dirty = true;
    }
    let dirty_patch = |h: &Map<String, Value>| -> Option<Map<String, Value>> {
        if dirty {
            Some(husbu_patch(h))
        } else {
            None
        }
    };

    if let Some((_, title, opsi)) = MENUS.iter().find(|(k, _, _)| *k == action) {
        let opsi: Vec<(&str, &str)> = if opsi.is_empty() {
            menu_lainnya_options(mnum_or(h.get("affection"), 50.0)).to_vec()
        } else {
            opsi.to_vec()
        };
        let reply = kartu_husbu(ctx, &h, title, Some(&opsi));
        return match dirty_patch(&h) {
            Some(patch) => reply.with_mutation(set_user(&ctx.sender, patch)),
            None => reply,
        };
    }

    if MESUM_ACTIONS.contains(&action.as_str()) && mnum_or(h.get("affection"), 50.0) < 80.0 {
        let drop = (rng() * 30.0).floor() as i64 + 30;
        let affection = mnum_or(h.get("affection"), 50.0);
        h.insert("affection".into(), jnum((affection - drop as f64).max(0.0)));
        let text = format!(
            "💢 *HENTIKAN!* Kamu bertindak terlalu agresif saat perasaannya masih samar! *{}* \
             menepis tanganmu dengan wajah marah dan kecewa.\n💞 *Affection -{drop}*",
            js_str(h.get("name"))
        );
        return balas(&ctx.sender, text, Some(husbu_patch(&h)), Some("💢"));
    }

    if action == "nikah" {
        let affection = mnum_or(h.get("affection"), 50.0);
        if affection < 100.0 {
            return balas(
                &ctx.sender,
                "⚠️ Affection belum 100! Jangan terburu-buru menerima lamarannya!".to_string(),
                dirty_patch(&h),
                None,
            );
        }
        if js_truthy(h.get("married")) {
            return balas(
                &ctx.sender,
                "⚠️ Kalian kan sudah menikah!".to_string(),
                dirty_patch(&h),
                None,
            );
        }
        let tier = js_str(h.get("tier"));
        let Some(dowry) = dowry_of(&tier) else {
            return Reply::text(error_template(ctx)).with_reaction("☢");
        };
        h.insert("married".into(), Value::Bool(true));
        h.insert("marriedDate".into(), Value::from(iso_now(now)));

        let nama = js_str(h.get("name"));
        let jid = husbu_jid(&nama);
        let mut patch = husbu_patch(&h);
        let mut fun = map_of(user.get("fun"));
        fun.insert("pasangan".into(), Value::from(jid.clone()));
        patch.insert("fun".into(), Value::Object(fun));

        let mut stats = map_of(user.get("husbuStats"));
        let married_count = mnum_or(stats.get("marriedCount"), 0.0) + 1.0;
        stats.insert("marriedCount".into(), jnum(married_count));
        patch.insert("husbuStats".into(), Value::Object(stats));

        if let Some(v) = energi_gain(user, dowry.limit, ctx.is_owner) {
            patch.insert("energi".into(), jnum(v as f64));
        }
        if let Some(v) = koin_gain(user, dowry.koin) {
            patch.insert("koin".into(), jnum(v as f64));
        }
        let mut rpg = rpg_map(ctx);
        let new_exp = apply_exp(&mut rpg, old_exp(user), dowry.exp);
        patch.insert("exp".into(), jnum(new_exp as f64));
        patch.insert("rpg".into(), Value::Object(rpg));

        let mut mutations = vec![set_user(&ctx.sender, patch)];
        let mut other = Map::new();
        let mut other_fun = Map::new();
        other_fun.insert("pasangan".into(), Value::from(ctx.sender.clone()));
        other.insert("fun".into(), Value::Object(other_fun));
        other.insert("name".into(), Value::from(nama.clone()));
        mutations.push(set_user(&jid, other));

        let text = format!(
            "💍 *KAMU RESMI MENERIMA LAMARAN {}!* 💍\n\n\
             Sebagai nafkah:\n- ⚡ {} Limit/Energi\n- 💰 {} Koin\n- ✨ {} EXP\n\n\
             Status `.cekpacar` kini resmi berpasangan!",
            nama.to_uppercase(),
            thousands_en(dowry.limit),
            thousands_en(dowry.koin),
            thousands_en(dowry.exp),
        );
        return Reply::text(text).mutating(mutations).with_reaction("💍");
    }

    if action == "hadiah" {
        if mnum_or(h.get("affection"), 50.0) < 100.0 {
            return balas(
                &ctx.sender,
                "⚠️ Dia belum cukup mencintaimu untuk memberi hadiah!".to_string(),
                dirty_patch(&h),
                None,
            );
        }
        let g = 300 + (rng() * 401.0).floor() as i64;
        let mut patch = dirty_patch(&h).unwrap_or_default();
        if let Some(v) = energi_gain(user, g, ctx.is_owner) {
            patch.insert("energi".into(), jnum(v as f64));
        }
        let nama = js_str(h.get("name"));
        let text = format!("💝 *{nama}* memberimu hadiah!\nKamu mendapat ⚡ {g} Limit/Energi!");
        return balas(&ctx.sender, text, Some(patch), Some("💝"));
    }

    if action == "klaim" {
        if !js_truthy(h.get("married")) {
            return balas(
                &ctx.sender,
                "⚠️ Hanya suami sah yang bisa klaim hadiah harian!".to_string(),
                dirty_patch(&h),
                None,
            );
        }
        let today = today_str(now);
        if mstr(h.get("lastClaimDate")).as_deref() == Some(today.as_str()) {
            return balas(
                &ctx.sender,
                "⚠️ Kamu sudah klaim hari ini! Coba lagi besok.".to_string(),
                dirty_patch(&h),
                None,
            );
        }
        let tier = js_str(h.get("tier"));
        let tier_mult = tier_mult(&tier);
        let koin = ((5000.0 + rng() * 15001.0) * tier_mult).floor() as i64;
        let exp = ((200.0 + rng() * 601.0) * tier_mult).floor() as i64;
        let day = hari_pernikahan(&h, now);
        let mut bonus = String::new();
        let mut bonus_koin = 0i64;
        if day == 7 || day == 30 || day == 100 {
            bonus_koin = (day as f64 * 1000.0 * tier_mult).floor() as i64;
            bonus = format!(
                "\n🎉 *MILESTONE HARI KE-{day}!* Bonus +{} Koin!",
                thousands_en(bonus_koin)
            );
        }
        h.insert("lastClaimDate".into(), Value::from(today));

        let mut patch = husbu_patch(&h);
        if bonus_koin > 0
            && let Some(v) = koin_gain(user, bonus_koin)
        {
            patch.insert("koin".into(), jnum(v as f64));
        }
        if let Some(v) = koin_gain(user, koin) {
            patch.insert("koin".into(), jnum(v as f64));
        }
        let mut rpg = rpg_map(ctx);
        let new_exp = apply_exp(&mut rpg, old_exp(user), exp);
        patch.insert("exp".into(), jnum(new_exp as f64));
        patch.insert("rpg".into(), Value::Object(rpg));

        let nama = js_str(h.get("name"));
        let text = format!(
            "🎁 *{nama}* memberi hadiah harian!\n💰 +{} Koin\n✨ +{exp} EXP{bonus}",
            thousands_en(koin)
        );
        return balas(&ctx.sender, text, Some(patch), Some("🎁"));
    }

    action_engine(ctx, user, &mut h, &action, rng)
}

/// Mesin aksi umum (`gachahusbu.js:390-466`).
fn action_engine(
    ctx: &Ctx,
    user: &Map<String, Value>,
    h: &mut Map<String, Value>,
    action: &str,
    rng: &mut dyn FnMut() -> f64,
) -> Reply {
    let now = now_ms();
    let neg_decay = apply_neglect(h, now);
    let mult = if mstr(h.get("nextMultUntil"))
        .and_then(|s| iso_to_ms(&s))
        .is_some_and(|t| now < t)
    {
        0.8
    } else {
        1.0
    };
    let anger = mnum_or(h.get("anger"), 0.0);
    let mood = anger_eff_mood(&mood_state_husbu(user, h, &ctx.sender, now), anger);
    let Some(result) = apply_action(action, h, &mood, mult, rng) else {
        return balas(
            &ctx.sender,
            "Aksi tidak dikenali. Gunakan tombol husbu.".to_string(),
            Some(husbu_patch(h)),
            Some("❓"),
        );
    };
    let married = js_truthy(h.get("married"));
    if result.phase == "married" && !married {
        return balas(
            &ctx.sender,
            "Aksi ini hanya untuk pasangan suami istri!".to_string(),
            Some(husbu_patch(h)),
            Some("⛔"),
        );
    }
    let affection = mnum_or(h.get("affection"), 50.0);
    if result.phase == "intim" && !married && affection < 80.0 {
        return Reply::None.with_mutation(set_user(&ctx.sender, husbu_patch(h)));
    }

    if result.phase == "married" && action != "nikah" {
        let eff = final_gain(result.change, affection, 0, mnum_or(h.get("anger"), 0.0));
        h.insert(
            "anger".into(),
            jnum(anger_update(
                mnum_or(h.get("anger"), 0.0),
                result.like,
                result.dislike,
            )),
        );
        let new_aff = (affection + eff.change as f64).clamp(0.0, 100.0);
        h.insert("affection".into(), jnum(new_aff));
        let koin = eff.change.max(0) * 100;

        let mut patch = husbu_patch(h);
        if koin > 0
            && let Some(v) = koin_gain(user, koin)
        {
            patch.insert("koin".into(), jnum(v as f64));
        }
        let mut rpg = rpg_map(ctx);
        let new_exp = apply_exp(&mut rpg, old_exp(user), result.exp);
        patch.insert("exp".into(), jnum(new_exp as f64));
        patch.insert("rpg".into(), Value::Object(rpg));

        let aff_line = if eff.change >= 0 {
            format!("💞 *Affection +{}*", eff.change)
        } else {
            format!("💞 *Affection {}*", eff.change)
        };
        let text = format!(
            "{}\n\n{} (Total: {}/100)\n💰 +{koin} Koin\n✨ +{} EXP",
            result.text,
            aff_line,
            fmt_num(new_aff),
            result.exp
        );
        return balas(&ctx.sender, text, Some(patch), Some("❤️"));
    }

    let today = today_str(now);
    if mstr(h.get("lastActionDate")).as_deref() != Some(today.as_str()) {
        h.insert("actionsToday".into(), jnum(0.0));
        h.insert("lastActionDate".into(), Value::from(today));
    }
    let actions_today = mnum_or(h.get("actionsToday"), 0.0) as i64 + 1;
    h.insert("actionsToday".into(), jnum(actions_today as f64));

    let eff = final_gain(result.change, affection, actions_today, anger);
    h.insert(
        "anger".into(),
        jnum(anger_update(anger, result.like, result.dislike)),
    );

    let mut new_aff = affection + eff.change as f64;
    let mut event_koin = 0i64;
    let mut event_exp = false;
    let mut event_block = String::new();
    let personality = mstr(h.get("personality")).unwrap_or_default();
    let name = mstr(h.get("name")).unwrap_or_default();
    if let Some(ev) = roll_event(married, result.phase, &personality, &name, rng) {
        new_aff += ev.aff;
        event_koin = ev.koin;
        if ev.anger > 0.0 {
            let a = (mnum_or(h.get("anger"), 0.0) + ev.anger).min(100.0);
            h.insert("anger".into(), jnum(a));
        }
        if let Some(ev_mood) = ev.mood {
            h.insert(
                "mood".into(),
                serde_json::json!({ "type": ev_mood, "since": iso_now(now) }),
            );
            h.insert(
                "moodUntil".into(),
                Value::from(iso_now(now + 12 * 3_600_000)),
            );
        }
        if ev.next_mult < 1.0 {
            h.insert(
                "nextMultUntil".into(),
                Value::from(iso_now(now + 24 * 3_600_000)),
            );
        }
        let aff_part = if ev.aff == 0.0 {
            String::new()
        } else {
            format!(
                " ({}{} aff)",
                if ev.aff > 0.0 { "+" } else { "" },
                fmt_num(ev.aff)
            )
        };
        let koin_part = if event_koin == 0 {
            String::new()
        } else {
            format!(" (+{} koin)", thousands_en(event_koin))
        };
        event_block = format!("\n\n✨ *EVENT:* {}{aff_part}{koin_part}", ev.text);
        event_exp = true;
    }
    let new_aff = new_aff.clamp(0.0, 100.0);
    h.insert("affection".into(), jnum(new_aff));

    let mut patch = husbu_patch(h);
    if event_koin != 0
        && let Some(v) = koin_gain(user, event_koin)
    {
        patch.insert("koin".into(), jnum(v as f64));
    }
    let mut rpg = rpg_map(ctx);
    let gain_exp = result.exp + if event_exp { 15 } else { 0 };
    let new_exp = apply_exp(&mut rpg, old_exp(user), gain_exp);
    patch.insert("exp".into(), jnum(new_exp as f64));
    patch.insert("rpg".into(), Value::Object(rpg));

    let mut aff_text = format!("💞 *Affection:* {}/100", fmt_num(new_aff));
    if eff.change != 0 {
        aff_text = format!(
            "💞 *Affection berubah:* {}{} (Total: {}/100)",
            if eff.change > 0 { "+" } else { "" },
            eff.change,
            fmt_num(new_aff)
        );
    }
    if new_aff == 100.0 {
        aff_text = "💞 *Affection MAKSIMAL! (100/100)* 🎉\n💍 *Dia akan melamarmu!*".to_string();
    }
    if neg_decay > 0 {
        aff_text.push_str(&format!("\n📉 *Neglect: -{neg_decay} aff* (jarang interaksi)"));
    }
    if eff.angry {
        aff_text.push_str(&format!("\n😡 *Dia MARAH!* Drain -{} aff", eff.drain));
    }
    if !eff.extra.is_empty() {
        aff_text.push('\n');
        aff_text.push_str(&eff.extra.join("\n"));
    }

    if new_aff <= 0.0 {
        let mut mutations = Vec::new();
        if married {
            let jid = husbu_jid(&name);
            if map_of(user.get("fun")).get("pasangan").and_then(Value::as_str) == Some(jid.as_str())
            {
                let mut fun = map_of(user.get("fun"));
                fun.insert("pasangan".into(), Value::from(""));
                patch.insert("fun".into(), Value::Object(fun));
            }
            let mut fun = Map::new();
            fun.insert("pasangan".into(), Value::from(""));
            let mut data = Map::new();
            data.insert("fun".into(), Value::Object(fun));
            mutations.push(set_user(&jid, data));
        }
        patch.insert("husbu".into(), Value::Null);
        mutations.push(set_user(&ctx.sender, patch));
        let text = format!(
            "💔 *{} MENINGGALKANMU!* 💔\n\n\
             Karena kasih sayangnya habis (0), dia mengemasi seluruh barang-barangnya dan pergi. \
             Kamu kehilangan husbumu! *(Ketik {}gachahusbu untuk memulai ulang)*",
            name.to_uppercase(),
            ctx.prefix
        );
        return Reply::text(text).mutating(mutations).with_reaction("💔");
    }

    let koin_line = if event_koin != 0 {
        format!("\n💰 +{} Koin", thousands_en(event_koin))
    } else {
        String::new()
    };
    let text = format!(
        "{}{}\n\n{}\n✨ +{} EXP{koin_line}",
        result.text, event_block, aff_text, result.exp
    );
    let emoji = if new_aff == 100.0 { "💍" } else { "✨" };
    balas(&ctx.sender, text, Some(patch), Some(emoji))
}

/// `handler` (`gachahusbu.js:176-468`) — urutan cabang persis JS.
fn dispatch(ctx: &Ctx, rng: &mut dyn FnMut() -> f64) -> Reply {
    let Some(root) = ctx.user_data.as_object() else {
        // JS: `if (!user) return;` — diam.
        return Reply::None;
    };
    let user = root.clone();
    let cmd = ctx.command.to_lowercase();

    match cmd.as_str() {
        "husbuku" | "suamiku" => status_husbu(ctx, &user),
        "tinggalinhusbu" => tinggalin_husbu(ctx, &user),
        "gachahusbu" | "gachasuami" => gacha_husbu(ctx, &user, rng),
        "husbuaction" => husbu_action(ctx, &user, rng),
        _ => Reply::None,
    }
}

#[plugin(
    name = "gachahusbu",
    alias = ["husbuaction", "tinggalinhusbu", "husbuku", "suamiku", "gachasuami"],
    category = "fun",
    description = "Gacha husbu impianmu, rebut hatinya, dan jadikan dia pasanganmu!",
    usage = ".gachahusbu | .husbuku | .tinggalinhusbu",
    example = ".gachahusbu",
    cooldown = 5,
    energi = 2,
    is_group = false,
    is_private = false
)]
pub fn gachahusbu(ctx: Ctx) -> Reply {
    let mut rng = math_random;
    dispatch(&ctx, &mut rng)
}
