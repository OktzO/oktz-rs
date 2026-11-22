//! `plugins/game/fisch.js` — Fishit, game memancing (`fisht`).
//!
//! Port 1:1 dari handler JS + lib `src/lib/fisch.js` + data
//! `src/lib/fisch-data.js`. State pemain adalah objek `user.fisch` JS, jadi
//! port memakai `serde_json::Value` langsung (bukan struct) supaya pola
//! `f.x = (f.x || 0) + y` dan field opsional terjaga.
//!
//! `ponytail:` TIGA hal JS tidak di-port:
//! - `sock.sendPreview(...)` (link preview berjudul `𝗙𝗜𝗦𝗛 𝗜𝗧 𝗚𝗔𝗠𝗘𝗦`
//!   dengan `jpegThumbnail` aset `fishit`) tidak ada di `Reply` — semua
//!   keluaran dikirim sebagai teks biasa. Prefix
//!   `${config.info.website} ` dari `send()` tetap dipertahankan.
//! - `setTimeout` jeda memancing (`fisch.js:171-176`) — hasil langsung
//!   menyusul pesan pertama dalam satu balasan.
//! - `db.markDirty("users")` → `Mutation::SetUser` di akhir handler; jalur
//!   yang di JS `return` lebih awal setelah refund (gacha duplikat rod)
//!   tidak menulis apa pun, sesuai state JS akhirnya.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};
use serde_json::{Value, json};

use crate::store::done::ms_ke_iso;

/// `FC = 15` — jeda antar pancingan detik (`fisch.js:31`).
const FC: i64 = 15;

// ---------------------------------------------------------------------------
// Data — `src/lib/fisch-data.js` (urutan entry dipertahankan seperti JS)
// ---------------------------------------------------------------------------

/// Ikan: (name, rarity, avgValue, minKg, maxKg).
type Fish = (&'static str, &'static str, f64, f64, f64);

/// Pulau: (key, nama, daftar ikan).
type Island = (&'static str, &'static str, &'static [Fish]);

/// (key, nama, [fish...]) — fish: (name, rarity, avgValue, minKg, maxKg).
const ISLANDS: &[Island] = &[
    ("mousewood", "Mousewood", &[
        ("Red Snapper", "common", 35.0, 1.0, 4.0),
        ("Largemouth Bass", "common", 44.0, 2.75, 5.6),
        ("Trout", "common", 52.0, 2.95, 7.1),
        ("Anchovy", "common", 80.0, 2.0, 8.0),
        ("Bream", "common", 62.0, 1.9, 6.1),
        ("Sockeye Salmon", "uncommon", 210.0, 4.5, 28.9),
        ("Yellowfin Tuna", "uncommon", 180.0, 2.0, 6.9),
        ("Carp", "uncommon", 265.0, 3.0, 12.0),
        ("Goldfish", "uncommon", 310.0, 0.9, 8.0),
        ("Snook", "rare", 600.0, 5.0, 78.6),
        ("Flounder", "rare", 640.0, 3.5, 76.4),
        ("Eel", "rare", 710.0, 2.65, 76.6),
        ("Pike", "epic", 2800.0, 2.35, 154.4),
        ("Whiptail Catfish", "epic", 4200.0, 2.0, 400.0),
        ("Whisker Bill", "mythic", 9500.0, 31.0, 67.5),
        ("Mudskipper", "rare", 780.0, 0.5, 12.0),
        ("Treble Bass", "exotic", 26000.0, 2.75, 2444.4),
        ("Mossy Turtle", "epic", 3600.0, 4.0, 80.0),
        ("Ironback Carp", "mythic", 11000.0, 20.0, 90.0),
        ("Ancient Gudgeon", "legendary", 55000.0, 8.0, 120.0),
        ("Phantom Trout", "godly", 140000.0, 15.0, 200.0),
        ("Spirit Bass", "secret", 420000.0, 30.0, 800.0),
    ]),
    ("roslitbay", "Roslit Bay", &[
        ("Minnow", "common", 200.0, 1.0, 26.3),
        ("Perch", "common", 225.0, 1.0, 40.8),
        ("Chub", "common", 215.0, 2.0, 26.7),
        ("Pearl", "uncommon", 390.0, 1.0, 37.5),
        ("Butterflyfish", "uncommon", 450.0, 0.85, 62.3),
        ("Clownfish", "uncommon", 425.0, 0.35, 52.5),
        ("Blue Tang", "rare", 850.0, 1.0, 54.0),
        ("Rose Pearl", "rare", 1250.0, 1.0, 90.6),
        ("Ribbon Eel", "epic", 2100.0, 7.75, 77.5),
        ("Clam", "epic", 2000.0, 1.0, 12.4),
        ("Squid", "legendary", 6000.0, 1.5, 84.0),
        ("Angelfish", "legendary", 6250.0, 1.25, 75.0),
        ("Alligator Gar", "mythic", 15000.0, 32.5, 158.9),
        ("Arapaima", "godly", 30000.0, 150.0, 187.5),
        ("Dumbo Octopus", "godly", 35000.0, 2.75, 618.8),
        ("Axolotl", "secret", 75000.0, 1.0, 666.7),
        ("Aurora Pearl", "secret", 100000.0, 1.0, 1406.3),
        ("Manta Ray", "secret", 125000.0, 887.5, 2662.5),
        ("Void Eel", "secret", 180000.0, 5.0, 1000.0),
    ]),
    ("mushgroveswamp", "Mushgrove Swamp", &[
        ("Fungal Cluster", "common", 1400.0, 1.0, 4.0),
        ("Swamp Bass", "common", 1680.0, 1.0, 4.0),
        ("White Perch", "uncommon", 1960.0, 1.2, 5.0),
        ("Grey Carp", "uncommon", 2240.0, 1.0, 4.5),
        ("Bowfin", "rare", 4200.0, 1.0, 4.5),
        ("Swamp Scallop", "rare", 5600.0, 1.0, 1.0),
        ("Marsh Gar", "epic", 12600.0, 1.0, 27.5),
        ("Diamond Catfish", "epic", 14000.0, 1.0, 10.0),
        ("Mushgrove Crab", "legendary", 32200.0, 1.0, 1.0),
        ("Alligator", "mythic", 68600.0, 1.0, 225.0),
        ("Handfish", "godly", 156800.0, 1.0, 9.0),
        ("RocketFuel", "secret", 518000.0, 5.0, 1.25),
        ("Resin", "secret", 560000.0, 1.0, 7.0),
        ("Spore Dragon", "godly", 165000.0, 80.0, 600.0),
    ]),
    ("terrapinisland", "Terrapin Island", &[
        ("Largemouth Bass", "common", 3400.0, 2.75, 51.9),
        ("Sea Bass", "uncommon", 3800.0, 4.0, 63.3),
        ("Gudgeon", "rare", 6400.0, 0.2, 26.7),
        ("Smallmouth Bass", "rare", 7800.0, 1.1, 45.3),
        ("Walleye", "epic", 14500.0, 2.9, 65.3),
        ("White Bass", "legendary", 41200.0, 1.4, 61.6),
        ("Redeye Bass", "mythic", 72400.0, 0.9, 69.0),
        ("Chinook Salmon", "mythic", 81200.0, 25.0, 143.8),
        ("Golden Smallmouth Bass", "godly", 182000.0, 3.0, 466.7),
        ("Sea Turtle", "godly", 236000.0, 110.0, 1466.7),
        ("Manatee", "secret", 580000.0, 150.0, 2400.0),
        ("Celestial Turtle", "godly", 250000.0, 150.0, 1200.0),
    ]),
    ("theocean", "The Ocean", &[
        ("Tire", "common", 3600.0, 11.0, 20.0),
        ("Seaweed", "common", 4400.0, 0.2, 6.0),
        ("Mackerel", "uncommon", 6200.0, 2.5, 46.9),
        ("Mullet", "uncommon", 6800.0, 1.25, 56.3),
        ("Sardine", "uncommon", 7800.0, 0.2, 20.0),
        ("Haddock", "rare", 9800.0, 2.75, 34.4),
        ("Salmon", "rare", 10200.0, 7.0, 91.0),
        ("Amberjack", "rare", 13400.0, 30.0, 86.3),
        ("Gold Cod", "epic", 18600.0, 6.0, 54.0),
        ("Barracuda", "epic", 22500.0, 8.25, 112.5),
        ("Nurse Shark", "legendary", 65400.0, 125.0, 166.7),
        ("Diamond Swordfish", "mythic", 98200.0, 175.0, 595.0),
        ("Bluefin Tuna", "mythic", 128000.0, 160.0, 254.5),
        ("Stingray", "mythic", 134000.0, 22.5, 172.5),
        ("Sailfish", "godly", 268000.0, 50.0, 666.7),
        ("Dolphin", "godly", 324000.0, 175.0, 1050.0),
        ("Moonfish", "godly", 394000.0, 375.0, 1350.0),
        ("Sawfish", "secret", 564000.0, 500.0, 1250.0),
        ("Mythic Fish", "secret", 620000.0, 1.0, 1428.6),
        ("Megalodon", "extinct", 3280000.0, 50000.0, 110000.0),
        ("Primordial Whale", "extinct", 5000000.0, 80000.0, 200000.0),
    ]),
    ("atlantis", "Atlantis", &[
        ("Voltfin Carp", "common", 2520.0, 17.5, 126.0),
        ("Aqua Scribe", "common", 818.0, 0.75, 40.9),
        ("Neptune's Nibbler", "common", 600.0, 1.0, 30.0),
        ("Lightning Minnow", "uncommon", 3200.0, 1.0, 80.0),
        ("Poseidon's Perch", "uncommon", 1960.0, 1.75, 49.0),
        ("Sunken Silverscale", "uncommon", 1672.0, 0.9, 41.8),
        ("Tentacled Horror", "rare", 21250.0, 8.5, 212.5),
        ("Mosaic Swimmer", "rare", 11250.0, 3.0, 112.5),
        ("Static Ray", "rare", 21000.0, 22.5, 210.0),
        ("Titan Tuna", "epic", 31420.0, 5.0, 157.1),
        ("Colossal Carp", "epic", 25200.0, 3.5, 126.0),
        ("Oracle's Eye", "legendary", 157500.0, 6.75, 262.5),
        ("Tentacle Eel", "legendary", 202500.0, 15.0, 337.5),
        ("Starlit Weaver", "mythic", 305859.0, 30.0, 600.0),
        ("Deep One", "mythic", 230399.0, 32.5, 650.0),
        ("Abyssal Devourer", "godly", 1440000.0, 1800.0, 6000.0),
        ("Celestial Koi", "godly", 1440000.0, 750.0, 6000.0),
        ("The Kraken", "extinct", 5120000.0, 12777.8, 112000.0),
        ("Ancient Kraken", "special", 10500000.0, 35000.0, 175000.0),
    ]),
    ("volcanicdepths", "Volcanic Depths", &[
        ("Lava Minnow", "common", 8000.0, 0.5, 10.0),
        ("Ember Bass", "common", 9500.0, 1.0, 15.0),
        ("Scorched Perch", "uncommon", 16000.0, 2.0, 30.0),
        ("Magma Eel", "uncommon", 19000.0, 3.0, 50.0),
        ("Flame Snapper", "rare", 35000.0, 5.0, 80.0),
        ("Obsidian Catfish", "rare", 42000.0, 8.0, 120.0),
        ("Inferno Barracuda", "epic", 90000.0, 15.0, 200.0),
        ("Molten Ray", "epic", 105000.0, 20.0, 250.0),
        ("Magma Shark", "legendary", 280000.0, 80.0, 600.0),
        ("Ember Leviathan", "mythic", 750000.0, 200.0, 1500.0),
        ("Volcano God", "godly", 2500000.0, 1000.0, 5000.0),
        ("Phoenix Fish", "godly", 2800000.0, 500.0, 4000.0),
        ("Lava Drake", "secret", 6500000.0, 2000.0, 15000.0),
        ("Primordial Inferno", "extinct", 25000000.0, 100000.0, 300000.0),
    ]),
    ("crystalcaves", "Crystal Caves", &[
        ("Glowfin Minnow", "common", 25000.0, 0.3, 8.0),
        ("Crystal Chub", "common", 28000.0, 0.5, 12.0),
        ("Prism Perch", "uncommon", 55000.0, 1.0, 25.0),
        ("Gem Carp", "uncommon", 65000.0, 2.0, 35.0),
        ("Diamond Trout", "rare", 120000.0, 3.0, 60.0),
        ("Sapphire Bass", "rare", 140000.0, 4.0, 80.0),
        ("Quartz Catfish", "epic", 350000.0, 15.0, 200.0),
        ("Amethyst Ray", "epic", 400000.0, 20.0, 250.0),
        ("Crystal Serpent", "legendary", 900000.0, 50.0, 500.0),
        ("Obsidian Dragon", "mythic", 2800000.0, 200.0, 2000.0),
        ("Crystal God", "godly", 9000000.0, 1000.0, 8000.0),
        ("Void Crystal", "secret", 28000000.0, 5000.0, 30000.0),
        ("Genesis Stone", "extinct", 120000000.0, 200000.0, 1000000.0),
    ]),
];

/// `travelRequirements` — (key, money, fish); mousewood tanpa syarat.
const TRAVEL: &[(&str, f64, i64)] = &[
    ("mousewood", 0.0, 0),
    ("roslitbay", 5000.0, 5),
    ("mushgroveswamp", 25000.0, 20),
    ("terrapinisland", 100000.0, 50),
    ("theocean", 1000000.0, 100),
    ("atlantis", 10000000.0, 200),
    ("volcanicdepths", 100000000.0, 400),
    ("crystalcaves", 1000000000.0, 750),
];

/// Rod: (key, name, luck, speed, comboFish, comboMutations, mutationsLuck,
/// sellMultiplier, price, description, maxLevel).
type Rod = (
    &'static str,
    &'static str,
    f64,
    f64,
    i64,
    i64,
    f64,
    f64,
    f64,
    &'static str,
    i64,
);

const RODS: &[Rod] = &[
    ("basicrod", "Basic Fishing Rod", 0.0, 0.0, 1, 1, 0.0, 0.0, 0.0, "Pancingan standar untuk pemula.", 5),
    ("ironrod", "Iron Rod", 0.02, 0.03, 1, 1, 0.001, 0.05, 25000.0, "Pancingan besi yang lebih kuat.", 10),
    ("goldrod", "Gold Rod", 0.06, 0.07, 1, 1, 0.003, 0.15, 250000.0, "Pancingan emas dengan luck lebih tinggi.", 15),
    ("diamondrod", "Diamond Rod", 0.12, 0.13, 2, 1, 0.007, 0.30, 2500000.0, "Pancingan berlian — combo ikan meningkat.", 20),
    ("ancientrod", "Ancient Rod", 0.20, 0.20, 2, 2, 0.015, 0.50, 25000000.0, "Pancingan kuno — mutasi combo meningkat.", 25),
    ("mythicrod", "Mythic Rod", 0.30, 0.28, 3, 2, 0.025, 0.75, 250000000.0, "Pancingan mythic — memanggil ikan langka.", 30),
    ("legendaryrod", "Legendary Rod", 0.42, 0.38, 3, 3, 0.040, 1.00, 2500000000.0, "Pancingan legenda — combo penuh & sell bonus besar.", 40),
    ("depthseekerrod", "Depthseeker Rod", 0.55, 0.48, 4, 3, 0.060, 1.30, 25000000000.0, "Pancingan penjelajah lautan dalam.", 50),
    ("voidrod", "Void Rod", 0.70, 0.60, 4, 4, 0.085, 1.75, 0.0, "Pancingan void — dari dimensi lain. (Token Store)", 60),
    ("cosmicrod", "Cosmic Rod", 0.85, 0.72, 5, 4, 0.115, 2.20, 0.0, "Pancingan kosmik — kekuatan dari bintang. (Token Store)", 75),
    ("prestigerod", "Prestige Rod", 0.95, 0.80, 5, 5, 0.150, 2.70, 0.0, "Pancingan prestige — hadiah bagi yang melampaui batas.", 99),
    ("omegaRod", "Omega Rod", 1.20, 0.90, 6, 5, 0.200, 3.50, 0.0, "Pancingan omega — milik sang juara season.", 99),
];

/// Enchant: (key, name, rarity, luck, mutationChance, sellMultiplier,
/// fishSize, desc) — nilai 0.0 berarti `effect` TIDAK ada di JS
/// (`enchant?.effect?.luck` → undefined → dilewati).
type Enchant = (
    &'static str,
    &'static str,
    &'static str,
    f64,
    f64,
    f64,
    f64,
    &'static str,
);

const ENCHANTS: &[Enchant] = &[
    ("swift", "Swift", "common", 0.0, 0.0, 0.0, 0.0, "Mempercepat progres menangkap."),
    ("hasty", "Hasty", "common", 0.0, 0.0, 0.0, 0.0, "Meningkatkan kecepatan progres."),
    ("blessedsong", "Blessed Song", "common", 0.0, 0.0, 0.0, 0.0, "+40% Progress Speed"),
    ("patient", "Patient", "common", 1.05, 0.0, 0.0, 0.0, "Kesabaran menghasilkan hasil lebih baik."),
    ("skilled", "Skilled", "common", 0.0, 0.0, 0.0, 0.0, "Tambahan XP setiap tangkapan."),
    ("divine", "Divine", "rare", 1.45, 0.0, 0.0, 0.0, "Keberuntungan tinggi."),
    ("clever", "Clever", "rare", 0.0, 0.0, 0.0, 0.0, "x2.25 XP"),
    ("frostbite", "Frostbite", "rare", 1.35, 0.0, 0.0, 0.0, "Dingin es menenangkan ikan."),
    ("lucky", "Lucky", "epic", 1.2, 0.0, 0.0, 0.0, "Menambah keberuntungan."),
    ("volcanic", "Volcanic", "epic", 1.7, 0.0, 1.6, 0.0, "Panas magma meningkatkan nilai ikan."),
    ("glittered", "Glittered", "epic", 0.0, 0.03, 0.0, 0.0, "Peluang mutasi +3%"),
    ("breezed", "Breezed", "epic", 1.65, 0.009, 0.0, 0.0, "+65% Luck, +20% Progress, +0.9% Mutation"),
    ("royalcrest", "Royal Crest", "legendary", 2.0, 0.0, 2.5, 0.0, "Simbol raja laut, nilai dan luck luar biasa."),
    ("crystalwave", "Crystal Wave", "legendary", 1.85, 0.04, 0.0, 0.0, "Gelombang kristal, hasil langka dan cepat."),
    ("seaoverlord", "Sea Overlord", "legendary", 0.0, 0.0, 4.0, 6.0, "+300% Fish Size, 4x Sell"),
    ("phantom", "Phantom", "legendary", 1.8, 0.05, 0.0, 0.0, "Energi roh laut, luck dan mutasi langka."),
    ("chaotic", "Chaotic", "mythic", 0.0, 0.0, 24.0, 0.0, "Nilai jual ikan drastis."),
    ("wise", "Wise", "mythic", 0.0, 0.0, 0.0, 0.0, "x5 XP"),
    ("mutated", "Mutated", "mythic", 0.0, 0.1, 0.0, 0.0, "+10% Mutation"),
    ("immortal", "Immortal", "mythic", 1.75, 0.0, 16.0, 0.0, "+75% Luck, +30% Progress, 16x Sell"),
    ("abyssborn", "Abyssborn", "mythic", 2.2, 0.12, 4.0, 0.0, "Kekuatan laut dalam."),
    ("astral", "Astral", "mythic", 2.5, 0.0, 0.0, 0.0, "Daya kosmik dari bintang."),
    ("abyssalflare", "Abyssal Flare", "godly", 4.5, 0.25, 9.0, 0.0, "Api dari jurang laut."),
    ("reapersnet", "Reaper's Net", "godly", 4.0, 0.0, 7.0, 0.0, "Jaring pencabut laut."),
    ("timeless", "Timeless", "secret", 5.0, 0.0, 0.0, 0.0, "Energi waktu membimbingmu."),
    ("voidheart", "Voidheart", "secret", 6.0, 0.25, 12.0, 0.0, "Inti kekosongan, mutasi dan nilai ekstrem."),
    ("godslayer", "Godslayer", "secret", 7.5, 0.35, 20.0, 0.0, "Rod legendaris yang menantang dewa laut."),
    ("universe", "Universe", "secret", 8.5, 0.45, 24.0, 0.0, "Rod yang mudah mendapatkan ikan secret."),
    ("apocalypse", "Apocalypse", "secret", 9.0, 0.5, 28.0, 4.0, "Kekuatan akhir zaman."),
];

/// Mutasi: (nama, multiplier, chance) — URUTAN FILE `fisch-data.js:212-297`
/// dipertahankan karena `rollMutations` mengiterasi `Object.entries`.
const MUTATIONS: &[(&str, f64, f64)] = &[
    ("Universe", 24.0, 0.00001), ("Frozen", 21.0, 0.00001), ("Phoenix", 19.0, 0.00001),
    ("Seeker", 17.8, 0.00001), ("Tryhard", 17.0, 0.0005), ("Darkness", 16.8, 0.0005),
    ("Mossy", 16.5, 0.0005), ("Mastered", 16.0, 0.0005), ("Glowy", 15.0, 0.0007),
    ("Umbra", 15.0, 0.0007), ("Evil", 15.0, 0.0007), ("Nocturnal", 14.2, 0.0008),
    ("Serene", 14.0, 0.0008), ("Diurnal", 13.5, 0.0008), ("Atomic", 12.0, 0.001),
    ("Chaotic", 12.0, 0.001), ("Glacial", 12.0, 0.001), ("Puritas", 10.7, 0.0015),
    ("Snowy", 10.0, 0.002), ("Blessed", 10.0, 0.002), ("Infernal", 10.0, 0.002),
    ("Luminescent", 9.0, 0.0025), ("Carrot", 8.0, 0.003), ("Nuclear", 8.0, 0.003),
    ("Rainbow Cluster", 8.0, 0.003), ("Chilled", 8.0, 0.003), ("Prismize", 8.0, 0.003),
    ("Toxic", 8.0, 0.003), ("Nova", 7.5, 0.0035), ("Shrouded", 7.5, 0.0035),
    ("Stardust", 7.5, 0.0035), ("Levitas", 7.0, 0.004), ("Aurora", 6.5, 0.0045),
    ("Wrath", 6.5, 0.0045), ("Astral", 6.0, 0.005), ("Gemstone", 6.0, 0.005),
    ("Heavenly", 6.0, 0.005), ("Crimson", 6.0, 0.005), ("Lost", 5.5, 0.006),
    ("Ashen Fortune", 5.0, 0.007), ("Bloom", 5.0, 0.007), ("Cursed Touch", 5.0, 0.007),
    ("Emberflame", 5.0, 0.007), ("Galactic", 5.0, 0.007), ("Mythical", 4.5, 0.008),
    ("Anomalous", 4.44, 0.008), ("Spirit", 4.2, 0.008), ("Aureolin", 4.0, 0.009),
    ("Greedy", 4.0, 0.009), ("Revitalized", 4.0, 0.009), ("Abyssal", 3.5, 0.01),
    ("Aurulent", 3.5, 0.01), ("Atlantean", 3.0, 0.012), ("Celestial", 3.0, 0.012),
    ("Crystalized", 3.0, 0.012), ("Ember", 3.0, 0.012), ("Aurelian", 2.5, 0.015),
    ("Fossilized", 2.5, 0.015), ("Lunar", 2.5, 0.015), ("Scorched", 2.5, 0.015),
    ("Moon-Kissed", 2.2, 0.02), ("Aurous", 2.0, 0.025), ("Midas", 2.0, 0.025),
    ("Giant", 2.0, 0.03), ("Purified", 2.0, 0.03), ("Sparkling", 1.85, 0.05),
    ("Glossy", 1.6, 0.06), ("Silver", 1.6, 0.06), ("Big", 1.5, 0.08),
    ("Transparent", 2.2, 0.025), ("Metallic", 2.8, 0.018), ("Bioluminescent", 3.2, 0.014),
    ("Ancient", 3.8, 0.012), ("Radioactive", 4.2, 0.009), ("Crystalline", 4.8, 0.007),
    ("Void-Touched", 5.5, 0.006), ("Mythweaver", 6.2, 0.005), ("Starborn", 7.0, 0.004),
    ("Primordial", 8.5, 0.003), ("Dreambreaker", 10.5, 0.002), ("Sovereign", 13.0, 0.001),
    ("Omniscient", 16.0, 0.0008), ("Transcendent", 20.0, 0.00005), ("Absolute", 30.0, 0.00001),
];

/// `RARITY_EMOJI` (`fisch-data.js:299`).
fn rc(rarity: &str) -> &'static str {
    match rarity {
        "common" => "⚪", "uncommon" => "🟢", "rare" => "🔵", "epic" => "🟣",
        "legendary" => "🟡", "mythic" => "🔴", "godly" => "🟠", "exotic" => "🟤",
        "secret" => "⚫", "extinct" => "💀", "special" => "✨",
        _ => "W",
    }
}

/// `encCost` (`fisch.js:33-42`).
fn enc_cost(rarity: &str) -> f64 {
    match rarity {
        "rare" => 500000.0, "epic" => 5e6, "legendary" => 5e7, "mythic" => 5e8,
        "godly" => 5e9, "secret" => 5e10,
        _ => 50000.0,
    }
}

/// `UPGRADES` — (key, name, desc, maxLevel, effect per level, baseCost,
/// costMultiplier); `effect(lv) = lv*per_level`, `getCost(lv) =
/// floor(baseCost * mult^lv)`.
const UPGRADES: &[(&str, &str, &str, i64, f64, f64, f64)] = &[
    ("luck", "🍀 Luck Upgrade", "Luck +2% per level", 50, 0.02, 5000000.0, 2.5),
    ("speed", "⚡ Speed Upgrade", "Speed -1% per level", 40, 0.01, 3000000.0, 2.3),
    ("sell", "💰 Sell Upgrade", "Sell +5% per level", 60, 0.05, 8000000.0, 2.8),
];

fn upgrade_of(key: &str) -> Option<&(&'static str, &'static str, &'static str, i64, f64, f64, f64)> {
    UPGRADES.iter().find(|(k, ..)| *k == key)
}

/// `DAILY_REWARDS` (`fisch-data.js:308-314`) — (streak, money, tickets).
const DAILY_REWARDS: &[(i64, f64, i64)] = &[
    (1, 50000.0, 0), (2, 100000.0, 0), (3, 250000.0, 1), (4, 500000.0, 0),
    (5, 1000000.0, 2), (6, 2000000.0, 0), (7, 10000000.0, 5),
    (14, 100000000.0, 10), (30, 1000000000.0, 20),
];

/// `TOKEN_SHOP` — (id, name, cost, type, value_str, value_i64).
/// Rod memakai `value_str`, sisanya `value_i64` (lihat switch JS).
const TOKEN_SHOP: &[(&str, &str, i64, &str, &str, i64)] = &[
    ("voidrod", "🌑 Void Rod", 300, "rod", "voidrod", 0),
    ("cosmicrod", "🌌 Cosmic Rod", 800, "rod", "cosmicrod", 0),
    ("tickets10", "🎟️ 10 Tiket Gacha", 50, "tickets", "", 10),
    ("tickets50", "🎟️ 50 Tiket Gacha", 200, "tickets", "", 50),
    ("money", "💰 100B Coins", 100, "coins", "", 100000000000),
    ("bigmoney", "💰 10T Coins", 500, "coins", "", 10000000000000),
];

const GACHA_COST_COINS: f64 = 5000000.0;
const GACHA_PITY_LIMIT: i64 = 80;

/// Nilai entri gacha: string (rod/rarity) atau angka (tiket/token/boost).
#[derive(Clone, Copy)]
enum GachaNilai {
    Str(&'static str),
    Int(i64),
    Float(f64),
}

impl GachaNilai {
    fn ke_value(self) -> Value {
        match self {
            Self::Str(s) => json!(s),
            Self::Int(n) => json!(n),
            Self::Float(f) => json!(f),
        }
    }
}

/// `GACHA_POOL` — (type, value, label, rarity, weight).
const GACHA_POOL: &[(&str, GachaNilai, &str, &str, f64)] = &[
    ("tickets", GachaNilai::Int(2), "🎟️ 2 Tiket Gacha", "common", 22.0),
    ("xp_boost", GachaNilai::Float(1.5), "⚡ XP Boost x1.5", "common", 18.0),
    ("rod", GachaNilai::Str("ironrod"), "🎣 Iron Rod", "rare", 10.0),
    ("enchant_scroll", GachaNilai::Str("common"), "📜 Scroll (Common)", "common", 15.0),
    ("enchant_scroll", GachaNilai::Str("rare"), "📜 Scroll (Rare)", "rare", 8.0),
    ("tickets", GachaNilai::Int(5), "🎟️ 5 Tiket Gacha", "rare", 4.0),
    ("rod", GachaNilai::Str("goldrod"), "🎣 Gold Rod", "epic", 5.0),
    ("enchant_scroll", GachaNilai::Str("epic"), "📜 Scroll (Epic)", "epic", 4.0),
    ("tokens", GachaNilai::Int(25), "🪙 25 Tokens", "epic", 3.0),
    ("rod", GachaNilai::Str("diamondrod"), "🎣 Diamond Rod", "legendary", 2.5),
    ("tokens", GachaNilai::Int(75), "🪙 75 Tokens", "legendary", 2.0),
    ("rod", GachaNilai::Str("voidrod"), "🌑 Void Rod", "ssr", 0.5),
    ("rod", GachaNilai::Str("cosmicrod"), "🌌 Cosmic Rod", "ssr", 0.3),
    ("tokens", GachaNilai::Int(200), "🪙 200 Tokens", "ssr", 0.2),
];

/// Hadiah jackpot: (type, min, max, weight).
type JackpotReward = (&'static str, f64, f64, f64);

/// Pool jackpot: (id, name, cost, weight pool, rewards).
type JackpotPool = (&'static str, &'static str, f64, f64, &'static [JackpotReward]);

/// `JACKPOT_POOLS` (`lib/fisch.js:246-306`) — (id, name, cost, weight pool,
/// rewards); reward: (type, min, max, weight).
const JACKPOT_POOLS: &[JackpotPool] = &[
    ("mini", "Mini Jackpot", 500000.0, 60.0, &[
        ("coins", 1e6, 5e6, 40.0), ("energi", 5.0, 20.0, 25.0),
        ("limit", 5.0, 15.0, 20.0), ("tickets", 1.0, 3.0, 10.0),
        ("exp_boost", 1.0, 3.0, 5.0),
    ]),
    ("mega", "Mega Jackpot", 5000000.0, 30.0, &[
        ("coins", 1e8, 5e8, 30.0), ("energi", 20.0, 100.0, 20.0),
        ("limit", 15.0, 50.0, 20.0), ("tickets", 3.0, 10.0, 15.0),
        ("premium_7d", 1.0, 1.0, 10.0), ("exp_boost", 3.0, 10.0, 5.0),
    ]),
    ("ultra", "Ultra Jackpot", 50000000.0, 9.0, &[
        ("coins", 1e9, 9e9, 25.0), ("energi", 100.0, 500.0, 15.0),
        ("limit", 50.0, 200.0, 15.0), ("premium_30d", 1.0, 1.0, 10.0),
        ("partner_7d", 1.0, 1.0, 5.0), ("tokens", 50.0, 200.0, 20.0),
        ("tickets", 10.0, 50.0, 10.0),
    ]),
    ("legend", "Legendary Jackpot", 500000000.0, 1.0, &[
        ("coins", 1e9, 9e9, 20.0), ("limit", 200.0, 999.0, 15.0),
        ("energi", 500.0, 9999.0, 10.0), ("premium_30d", 1.0, 1.0, 15.0),
        ("partner_30d", 1.0, 1.0, 5.0), ("tokens", 200.0, 1000.0, 20.0),
        ("unlimited_energi", 1.0, 1.0, 5.0), ("unlimited_limit", 1.0, 1.0, 5.0),
        ("tickets", 50.0, 200.0, 5.0),
    ]),
];

/// Label reward jackpot (`fisch.js:785-797`).
fn label_jackpot(tipe: &str) -> &str {
    match tipe {
        "coins" => "Koin", "energi" => "Energi", "limit" => "Limit",
        "tickets" => "Gacha Tickets", "tokens" => "Prestige Tokens",
        "exp_boost" => "EXP Boost", "premium_7d" => "Premium 7 Hari",
        "premium_30d" => "Premium 30 Hari", "partner_7d" => "Partner 7 Hari",
        "partner_30d" => "Partner 30 Hari", "unlimited_energi" => "UNLIMITED Energi",
        "unlimited_limit" => "UNLIMITED Limit",
        _ => tipe,
    }
}

// ---------------------------------------------------------------------------
// Lib — `src/lib/fisch.js`
// ---------------------------------------------------------------------------

/// `formatMoney` (`lib/fisch.js:16-26`).
pub(crate) fn format_money(n: f64) -> String {
    if !n.is_finite() || n == 0.0 {
        return "0".to_string();
    }
    const S: [&str; 12] = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc"];
    let mut t = (n.abs().log10() / 3.0).floor() as i64;
    if t >= S.len() as i64 {
        t = S.len() as i64 - 1;
    }
    if t < 0 {
        return format!("{n:.2}");
    }
    let scale = 10f64.powi((t * 3) as i32);
    let v = (n / scale * 100.0).round() / 100.0;
    let teks = if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
    };
    format!("{teks}{}", S[t as usize])
}

/// `fishingRod[key]` sebagai objek JS baru (`{ ...fishingRod.basicrod }`).
pub(crate) fn rod_def(key: &str) -> Value {
    let (_, name, luck, speed, combo, combo_mut, mut_luck, sell, price, desc, max_level) =
        RODS.iter().find(|(k, ..)| *k == key).expect("rod dikenal");
    json!({
        "name": name, "luck": luck, "speed": speed,
        "comboFish": combo, "comboMutations": combo_mut,
        "mutationsLuck": mut_luck, "sellMultiplier": sell,
        "price": price, "description": desc,
        "level": 1, "maxLevel": max_level, "exp": 0, "expToNextLevel": 100,
    })
}

fn enchant_of(key: &str) -> Option<&Enchant> {
    ENCHANTS.iter().find(|(k, ..)| *k == key)
}

/// `getOrCreateFischUser` (`lib/fisch.js:39-82`).
pub(crate) fn default_fisch() -> Value {
    json!({
        "money": 200.0,
        "level": 1,
        "exp": 0,
        "expToNextLevel": 100,
        "usedFishingRod": "basicrod",
        "fishingRods": { "basicrod": rod_def("basicrod") },
        "currentIsland": "mousewood",
        "inventory": [],
        "fishFound": [],
        "mutationFound": [],
        "fishCaught": 0,
        "totalEarned": 0,
        "gachaTickets": 0,
        "gachaPity": 0,
        "prestigeTokens": 0,
        "prestige": 0,
        "luckUpgrade": 0,
        "speedUpgrade": 0,
        "sellUpgrade": 0,
        "lastDaily": null,
        "dailyStreak": 0,
        "fishingPending": [],
        "travelFound": ["mousewood"],
        "achievements": [],
        "achievementPoints": 0,
        "streak": 0,
        "lastFishTime": 0,
    })
}

/// Normalisasi `if (!f.fishingRods)` dst (`lib/fisch.js:75-81`).
fn normalisasi(f: &mut Value) {
    if f.get("fishingRods").and_then(Value::as_object).is_none_or(|m| m.is_empty()) {
        f["fishingRods"] = json!({ "basicrod": rod_def("basicrod") });
    } else if f["fishingRods"].get("basicrod").is_none() {
        f["fishingRods"]["basicrod"] = rod_def("basicrod");
    }
    if f.get("travelFound").and_then(Value::as_array).is_none_or(|a| a.is_empty()) {
        f["travelFound"] = json!(["mousewood"]);
    }
    if f.get("achievements").and_then(Value::as_array).is_none() {
        f["achievements"] = json!([]);
    }
}

/// `getRandomFish` (`lib/fisch.js:84-159`).
pub(crate) fn random_fish(rod: &Value, island_key: &str) -> Option<Value> {
    use rand::Rng as _;
    let (_, _, list) = ISLANDS.iter().find(|(k, ..)| *k == island_key)?;
    let enchant = rod.get("enchant").and_then(Value::as_str).and_then(enchant_of);
    let mut luck_bonus = rod.get("luck").and_then(Value::as_f64).unwrap_or(0.0);
    if let Some(e) = enchant
        && e.3 != 0.0 {
            // `luckBonus *= enchant.effect.luck`
            luck_bonus *= e.3;
        }

    let peluang = |rarity: &str| -> f64 {
        match rarity {
            "common" => 1000.0,
            "uncommon" => 400.0 + luck_bonus * 200.0,
            "rare" => 100.0 + luck_bonus * 80.0,
            "epic" => 20.0 + luck_bonus * 30.0,
            "legendary" => 5.0 + luck_bonus * 15.0,
            "mythic" => 1.0 + luck_bonus * 8.0,
            "godly" => 0.5 + luck_bonus * 5.0,
            "exotic" => 0.3 + luck_bonus * 4.0,
            "secret" => 1.0 + luck_bonus * 10.0,
            "extinct" => 0.1 + luck_bonus * 2.0,
            "special" => 0.05 + luck_bonus * 1.0,
            _ => 1.0,
        }
    };

    let total: f64 = list.iter().map(|f| peluang(f.1)).sum();
    let mut roll = rand::rng().random::<f64>() * total;
    let mut chosen = &list[0];
    for fish in list.iter() {
        roll -= peluang(fish.1);
        if roll <= 0.0 {
            chosen = fish;
            break;
        }
    }

    let mut weight = chosen.3 + rand::rng().random::<f64>() * (chosen.4 - chosen.3);
    if let Some(e) = enchant
        && e.6 != 0.0 {
            weight *= e.6; // effect.fishSize
        }
    let weight = (weight * 100.0).round() / 100.0; // toFixed(2)

    let mut total_price = (chosen.2 * weight).round();
    if let Some(sm) = rod.get("sellMultiplier").and_then(Value::as_f64)
        && sm != 0.0 {
            total_price = (total_price * (1.0 + sm)).round();
        }
    if let Some(e) = enchant
        && e.5 != 0.0 {
            total_price = (total_price * e.5).round();
        }

    let mut_max = rod.get("comboMutations").and_then(Value::as_i64).unwrap_or(1);
    let mut_bonus = rod.get("mutationsLuck").and_then(Value::as_f64).unwrap_or(0.0)
        + enchant.map_or(0.0, |e| e.4);
    let mutasi = roll_mutations(mut_max, mut_bonus);
    let is_mutated = mutasi.iter().any(|m| m != "Normal");
    let mut nama = chosen.0.to_string();
    if is_mutated {
        for m in &mutasi {
            if m != "Normal"
                && let Some((_, mult, _)) = MUTATIONS.iter().find(|(n, ..)| n == m) {
                    total_price = (total_price * mult).round();
                }
        }
        nama = format!("🧬 {nama}");
    }

    Some(json!({
        "name": nama,
        "rarity": chosen.1,
        "type": "fish",
        "kg": weight,
        "pricePerKg": chosen.2,
        "price": total_price,
        "mutations": mutasi,
        "isMutated": is_mutated,
    }))
}

/// `rollMutations` (`lib/fisch.js:161-169`).
fn roll_mutations(max_count: i64, bonus: f64) -> Vec<String> {
    use rand::Rng as _;
    let mut found: Vec<String> = Vec::new();
    for (name, _, chance) in MUTATIONS {
        if found.len() as i64 >= max_count {
            break;
        }
        let final_chance = chance.powf(0.5) + bonus;
        if rand::rng().random::<f64>() < final_chance {
            found.push(name.to_string());
        }
    }
    if found.is_empty() {
        vec!["Normal".to_string()]
    } else {
        found
    }
}

/// `addRodExp` (`lib/fisch.js:171-189`) — None = tidak naik level.
pub(crate) fn add_rod_exp(f: &mut Value, rod_key: &str, amount: i64) -> Option<String> {
    let rod = f.get_mut("fishingRods").and_then(|r| r.get_mut(rod_key))?;
    if rod["level"].as_i64().unwrap_or(1) >= rod["maxLevel"].as_i64().unwrap_or(5) {
        return None;
    }
    rod["exp"] = json!(rod["exp"].as_i64().unwrap_or(0) + amount);
    let mut naik = false;
    while rod["exp"].as_i64().unwrap_or(0) >= rod["expToNextLevel"].as_i64().unwrap_or(100)
        && rod["level"].as_i64().unwrap_or(1) < rod["maxLevel"].as_i64().unwrap_or(5)
    {
        rod["exp"] = json!(rod["exp"].as_i64().unwrap_or(0) - rod["expToNextLevel"].as_i64().unwrap_or(100));
        rod["level"] = json!(rod["level"].as_i64().unwrap_or(1) + 1);
        rod["luck"] = json!(((rod["luck"].as_f64().unwrap_or(0.0) + 0.01) * 1000.0).round() / 1000.0);
        rod["speed"] = json!(((rod["speed"].as_f64().unwrap_or(0.0) + 0.01) * 1000.0).round() / 1000.0);
        rod["expToNextLevel"] = json!((rod["expToNextLevel"].as_f64().unwrap_or(100.0) * 1.5).floor() as i64);
        naik = true;
    }
    if naik {
        Some(format!(
            "🎣 Rod *{}* naik ke level {}!\n✨ Luck +1%, Speed +1%",
            rod["name"].as_str().unwrap_or(""),
            rod["level"]
        ))
    } else {
        None
    }
}

/// `addPlayerExp` (`lib/fisch.js:191-201`) — true = level up.
pub(crate) fn add_player_exp(f: &mut Value, amount: i64) -> bool {
    f["exp"] = json!(f["exp"].as_i64().unwrap_or(0) + amount);
    let mut naik = false;
    while f["exp"].as_i64().unwrap_or(0) >= f["expToNextLevel"].as_i64().unwrap_or(100)
        && f["level"].as_i64().unwrap_or(1) < 9999
    {
        f["exp"] = json!(f["exp"].as_i64().unwrap_or(0) - f["expToNextLevel"].as_i64().unwrap_or(100));
        f["level"] = json!(f["level"].as_i64().unwrap_or(1) + 1);
        f["expToNextLevel"] = json!((f["expToNextLevel"].as_f64().unwrap_or(100.0) * 1.3).floor() as i64);
        naik = true;
    }
    naik
}

/// `getUpgradedStats` (`lib/fisch.js:203-213`) — (luck, speed, sellMultiplier).
pub(crate) fn upgraded_stats(f: &Value, rod: &Value) -> (f64, f64, f64) {
    let luck_bonus = UPGRADES[0].4 * f["luckUpgrade"].as_f64().unwrap_or(0.0);
    let speed_bonus = UPGRADES[1].4 * f["speedUpgrade"].as_f64().unwrap_or(0.0);
    let sell_bonus = UPGRADES[2].4 * f["sellUpgrade"].as_f64().unwrap_or(0.0);
    let prestige_bonus = f["prestige"].as_f64().unwrap_or(0.0) * 0.05;
    (
        rod["luck"].as_f64().unwrap_or(0.0) + luck_bonus + prestige_bonus,
        (rod["speed"].as_f64().unwrap_or(0.0) + speed_bonus).min(0.98),
        rod["sellMultiplier"].as_f64().unwrap_or(0.0) + sell_bonus,
    )
}

/// `doGachaPull` (`lib/fisch.js:215-234`) — (item, isSSR, isPity).
pub(crate) fn gacha_pull(f: &mut Value) -> (Value, bool, bool) {
    use rand::Rng as _;
    let is_pity = f["gachaPity"].as_i64().unwrap_or(0) >= GACHA_PITY_LIMIT;
    let pool: Vec<&(&str, GachaNilai, &str, &str, f64)> = GACHA_POOL
        .iter()
        .filter(|p| !is_pity || p.3 == "ssr")
        .collect();
    let total: f64 = pool.iter().map(|p| p.4).sum();
    let mut roll = rand::rng().random::<f64>() * total;
    let mut item = pool[0];
    for p in &pool {
        roll -= p.4;
        if roll <= 0.0 {
            item = p;
            break;
        }
    }
    let is_ssr = item.3 == "ssr";
    f["gachaPity"] = if is_ssr { json!(0) } else { json!(f["gachaPity"].as_i64().unwrap_or(0) + 1) };
    (json!({ "type": item.0, "value": item.1.ke_value(), "label": item.2 }), is_ssr, is_pity)
}

/// `getStreakBonus` (`lib/fisch.js:236-244`) — (mult, luckAdd).
pub(crate) fn streak_bonus(streak: i64) -> (f64, f64) {
    match streak {
        s if s >= 100 => (3.0, 0.15),
        s if s >= 50 => (2.0, 0.05),
        s if s >= 20 => (1.5, 0.0),
        s if s >= 10 => (1.35, 0.05),
        s if s >= 5 => (1.2, 0.0),
        s if s >= 3 => (1.1, 0.0),
        _ => (1.0, 0.0),
    }
}

/// `doJackpotPull` (`lib/fisch.js:308-326`) — (reward type, value).
pub(crate) fn jackpot_pull(pool_id: &str) -> Option<(String, f64)> {
    use rand::Rng as _;
    let (_, _, _, _, rewards) = JACKPOT_POOLS.iter().find(|(id, ..)| *id == pool_id)?;
    let total: f64 = rewards.iter().map(|r| r.3).sum();
    let mut roll = rand::rng().random::<f64>() * total;
    let mut chosen = &rewards[0];
    for r in rewards.iter() {
        roll -= r.3;
        if roll <= 0.0 {
            chosen = r;
            break;
        }
    }
    let value = (chosen.1 + rand::rng().random::<f64>() * (chosen.2 - chosen.1)).round();
    Some((chosen.0.to_string(), value))
}

/// `applyJackpotReward` (`lib/fisch.js:328-432`) — mutasi `f` dan field user
/// di luar fisch (energi/limit/premium/partner) → `(desc, fieldUser)`.
fn apply_jackpot(f: &mut Value, user_luar: &mut Value, tipe: &str, val: f64) -> String {
    match tipe {
        "coins" => {
            f["money"] = json!(money_of(f) + val);
            format!("+{} Coins", format_money(val))
        }
        "energi" => {
            user_luar["energi"] = json!(user_luar["energi"].as_f64().unwrap_or(0.0) + val);
            format!("+{} Energi", val)
        }
        "limit" => {
            user_luar["limit"] = json!(user_luar["limit"].as_f64().unwrap_or(0.0) + val);
            format!("+{} Limit", val)
        }
        "tickets" => {
            f["gachaTickets"] = json!(f["gachaTickets"].as_f64().unwrap_or(0.0) + val);
            format!("+{} Gacha Tickets", val)
        }
        "tokens" => {
            f["prestigeTokens"] = json!(f["prestigeTokens"].as_f64().unwrap_or(0.0) + val);
            format!("+{} Prestige Tokens", val)
        }
        "exp_boost" => {
            let boost = (f["expToNextLevel"].as_f64().unwrap_or(0.0) * val * 0.5).floor();
            f["exp"] = json!(f["exp"].as_f64().unwrap_or(0.0) + boost);
            format!("+{} EXP (x{} boost)", format_money(boost), val)
        }
        t if t == "premium_7d" || t == "premium_30d" => {
            let hari = if t == "premium_7d" { 7.0 } else { 30.0 };
            let now = crate::common::now_ms() as f64;
            let lama = date_ms(user_luar["premiumExpiry"].as_str().unwrap_or(""));
            let mulai = if lama > now { lama } else { now };
            user_luar["premiumExpiry"] = json!(ms_ke_iso((mulai + hari * 86_400_000.0) as i64));
            user_luar["isPremium"] = json!(true);
            format!("Premium {} Hari!", hari as i64)
        }
        t if t == "partner_7d" || t == "partner_30d" => {
            let hari = if t == "partner_7d" { 7.0 } else { 30.0 };
            let now = crate::common::now_ms() as f64;
            let lama = date_ms(user_luar["partnerExpiry"].as_str().unwrap_or(""));
            let mulai = if lama > now { lama } else { now };
            user_luar["partnerExpiry"] = json!(ms_ke_iso((mulai + hari * 86_400_000.0) as i64));
            user_luar["isPartner"] = json!(true);
            format!("Partner {} Hari!", hari as i64)
        }
        "unlimited_energi" => {
            user_luar["energi"] = json!(-1);
            "UNLIMITED Energi!".to_string()
        }
        "unlimited_limit" => {
            user_luar["limit"] = json!(-1);
            "UNLIMITED Limit!".to_string()
        }
        _ => "Mystery Reward!".to_string(),
    }
}

/// `new Date(value).getTime()` untuk string ISO / angka (lihat
/// `store/done.rs::date_ke_ms`).
fn date_ms(v: &str) -> f64 {
    crate::store::done::date_ke_ms(&Value::String(v.to_string())) as f64
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// `send()` (`fisch.js:46-60`) — link preview tidak ada, teksnya saja:
/// `${config.info.website} ${text}`.
fn kirim(ctx: &Ctx, text: &str) -> Reply {
    Reply::text(format!("{} {}", ctx.config().info.website, text))
}

/// Bantuan (`fisch.js:114-127`).
fn bantuan(ctx: &Ctx) -> String {
    let isi = "*FISHIT GAME*\n_Sistem permainan memancing terlengkap_\n\n\
*FISHING*\n`.fisht mancing` _Mulai memancing_\n`.fisht view` _Ambil tangkapan_\n`.fisht sell` _Jual ikan_\n`.fisht fishbook` _Koleksi ikan_\n`.fisht mutbook` _Koleksi mutasi_\n`.fisht top` _Leaderboard_\n\n\
*PROFILE*\n`.fisht me` _Profil kamu_\n`.fisht stats` _Stats detail_\n`.fisht daily` _Daily reward_\n\n\
*PULAU & ROD*\n`.fisht travel` _Daftar pulau_\n`.fisht travel <pulau>` _Pindah pulau_\n`.fisht shop` _Toko rod_\n`.fisht buy <rod>` _Beli rod_\n`.fisht equip <rod>` _Pasang rod_\n`.fisht rods` _Koleksi rod_\n`.fisht enchant <key>` _Enchant rod_\n`.fisht enchants` _Daftar enchant_\n`.fisht rodup` _Upgrade rod_\n\n\
*PRESTIGE*\n`.fisht prestige` _Info prestige_\n`.fisht tokens` _Token store_\n`.fisht upgrade` _Upgrade stats_\n`.fisht gacha` _Gacha_\n`.fisht gacha ticket` _Gacha pakai tiket_\n\n\
*JACKPOT*\n`.fisht jackpot` _Daftar jackpot_\n`.fisht jackpot <tier>` _Main jackpot_\n_Jackpot bisa kasih _Premium_, _Partner_, _Energi_, _Limit_, bahkan _UNLIMITED_!_";
    format!("{} {isi}", ctx.config().info.website)
}

fn money_of(f: &Value) -> f64 {
    f["money"].as_f64().unwrap_or(0.0)
}

fn i64_of(v: &Value) -> i64 {
    v.as_i64().unwrap_or(0)
}

/// Rod aktif (`f.usedFishingRod || 'basicrod'`).
fn rod_aktif(f: &Value) -> Option<&Value> {
    let key = f["usedFishingRod"].as_str().unwrap_or("basicrod");
    f["fishingRods"].get(key)
}

#[plugin(
    name = "fisht",
    alias = ["fishit"],
    category = "game",
    description = "Fishit - Fishing Rod Game",
    usage = ".fisht <command>",
    example = ".fisht help",
    cooldown = 3,
    energi = 0,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false
)]
pub fn fisht(ctx: Ctx) -> Reply {
    let cmd = ctx.command.to_lowercase();
    let sub = ctx.args.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let sa: Vec<String> = ctx.args.iter().skip(1).cloned().collect();

    // ---- toggle `.fishit on/off` (`fisch.js:85-110`) ----
    if cmd == "fishit" && (sub.is_empty() || sub == "on" || sub == "off") {
        if !ctx.is_group {
            return Reply::text("_Toggle hanya di grup_");
        }
        if !ctx.is_owner && !ctx.roster.is_admin(&ctx.sender) {
            return Reply::text("_Hanya admin/owner_");
        }
        let mut gd = if ctx.group_data.is_object() { ctx.group_data.clone() } else { json!({}) };
        if sub == "on" {
            gd["fishitEnabled"] = json!(true);
            return kirim(&ctx, "*FISHIT ENABLED*\n\nSemua member wajib main Fishit!\nKetik `.fisht help` untuk mulai")
                .mutating(vec![Mutation::SetGroup { jid: ctx.chat.clone(), data: gd }]);
        }
        if sub == "off" {
            gd["fishitEnabled"] = json!(false);
            return kirim(&ctx, "*FISHIT DISABLED*")
                .mutating(vec![Mutation::SetGroup { jid: ctx.chat.clone(), data: gd }]);
        }
        let status = if crate::common::js_truthy_value(gd.get("fishitEnabled").unwrap_or(&Value::Null)) {
            "ON"
        } else {
            "OFF"
        };
        return Reply::text(format!("*Fishit:* {status}\n`.fishit on/off`"));
    }

    if cmd != "fisht" && cmd != "fishit" {
        return Reply::None;
    }

    if sub.is_empty() || sub == "help" || sub == "menu" {
        return Reply::text(bantuan(&ctx));
    }

    // ---- state (`getOrCreateFischUser`) ----
    let mut user = if ctx.user_data.is_object() { ctx.user_data.clone() } else { json!({}) };
    if user.get("fisch").is_none_or(Value::is_null) {
        user["fisch"] = default_fisch();
    }
    normalisasi(&mut user["fisch"]);
    let mut f = user["fisch"].clone();
    let mut user_luar: Value = json!({});
    let balasan: String;
    let mut balasan_seq: Option<Reply> = None;

    match sub.as_str() {
        // ---- mancing (`fisch.js:128-188`) ----
        "mancing" | "fish" => {
            if f["fishingPending"].as_array().is_some_and(|a| !a.is_empty()) {
                return Reply::text("_Masih ada tangkapan!_ `.fisht view` _dulu._");
            }
            let now = crate::common::now_ms();
            let last = f["lastFishTime"].as_f64().unwrap_or(0.0) as i64;
            if last != 0 && now - last < FC * 1000 {
                let tunggu = (FC * 1000 - (now - last) + 999) / 1000;
                return Reply::text(format!("_Tunggu *{tunggu}* detik_"));
            }
            let rk = f["usedFishingRod"].as_str().unwrap_or("basicrod").to_string();
            let Some(rod) = f["fishingRods"].get(&rk).cloned() else {
                return Reply::text("_Rod aktif tidak ada!_ `.fisht rods`");
            };
            let ik = f["currentIsland"].as_str().unwrap_or("mousewood").to_string();
            let (st_luck, _st_speed, _st_sell) = upgraded_stats(&f, &rod);
            let mut catches: Vec<Value> = Vec::new();
            let mut tv = 0.0f64;
            for _ in 0..rod["comboFish"].as_i64().unwrap_or(1) {
                if let Some(mut fish) = random_fish(&rod, &ik) {
                    let (mult, _) = streak_bonus(f["streak"].as_i64().unwrap_or(0));
                    let price = (fish["price"].as_f64().unwrap_or(0.0) * mult).round();
                    fish["price"] = json!(price);
                    tv += price;
                    catches.push(fish);
                }
            }
            f["fishingPending"] = Value::Array(catches.clone());
            f["lastFishTime"] = json!(now);
            f["streak"] = json!(f["streak"].as_i64().unwrap_or(0) + 1);

            let nama_pulau = ISLANDS.iter().find(|(k, ..)| *k == ik).map(|(_, n, _)| *n).unwrap_or(&ik);
            let memancing = format!(
                "*_Memancing di {}..._*\n_Rod: {} | Luck: {:.1}%_",
                nama_pulau,
                rod["name"].as_str().unwrap_or(""),
                st_luck * 100.0
            );
            let mut txt = String::from("*HASIL MANCING!*\n\n");
            for c in &catches {
                txt.push_str(&format!(
                    "{} *{}*\n   _{} | {}kg_\n",
                    rc(c["rarity"].as_str().unwrap_or("")),
                    c["name"].as_str().unwrap_or(""),
                    format_money(c["price"].as_f64().unwrap_or(0.0)),
                    c["kg"]
                ));
                if c["isMutated"].as_bool().unwrap_or(false) {
                    let muts: Vec<String> = c["mutations"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|m| m.as_str())
                                .filter(|m| *m != "Normal")
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    txt.push_str(&format!("   _Mutasi: {}_\n", muts.join(", ")));
                }
            }
            txt.push_str(&format!("\n*Total: {}*", format_money(tv)));
            let streak = f["streak"].as_i64().unwrap_or(0);
            if streak >= 3 {
                txt.push_str(&format!("\n_Streak: {streak}x_"));
            }
            txt.push_str("\n\n`.fisht view` untuk mengambil!");
            // JS: send("Memancing...") → setTimeout → send(hasil) — dua pesan.
            balasan_seq = Some(kirim(&ctx, &memancing));
            balasan = txt;
        }
        // ---- view (`fisch.js:190-230`) ----
        "view" => {
            let Some(catches) = f["fishingPending"].as_array().cloned() else {
                return Reply::text("_Tidak ada tangkapan._ `.fisht mancing` _dulu!_");
            };
            if catches.is_empty() {
                return Reply::text("_Tidak ada tangkapan._ `.fisht mancing` _dulu!_");
            }
            let jumlah_tangkapan = catches.len();
            let mut tv = 0.0f64;
            let mut te = 0i64;
            let mut nf: Vec<String> = Vec::new();
            let mut nm: Vec<String> = Vec::new();
            for fish in &catches {
                let harga = fish["price"].as_f64().unwrap_or(0.0);
                tv += harga;
                // `te += Math.max(10, Math.floor(fish.price / 50))`
                te += 10.max((harga / 50.0).floor() as i64);
                let cn = strip_emoji(fish["name"].as_str().unwrap_or(""));
                let sudah = f["fishFound"].as_array().is_some_and(|a| a.iter().any(|v| v.as_str() == Some(&cn)));
                if !sudah {
                    f["fishFound"].as_array_mut().unwrap().push(json!(cn));
                    nf.push(cn);
                }
                if fish["isMutated"].as_bool().unwrap_or(false)
                    && let Some(muts) = fish["mutations"].as_array() {
                        for m in muts {
                            if let Some(m) = m.as_str()
                                && m != "Normal" {
                                    let sudah = f["mutationFound"].as_array().is_some_and(|a| a.iter().any(|v| v.as_str() == Some(m)));
                                    if !sudah {
                                        f["mutationFound"].as_array_mut().unwrap().push(json!(m));
                                        nm.push(m.to_string());
                                    }
                                }
                        }
                    }
            }
            f["money"] = json!(money_of(&f) + tv);
            f["fishCaught"] = json!(i64_of(&f["fishCaught"]) + catches.len() as i64);
            f["totalEarned"] = json!(f["totalEarned"].as_f64().unwrap_or(0.0) + tv);
            let mut inv = f["inventory"].as_array().cloned().unwrap_or_default();
            inv.extend(catches.iter().cloned());
            f["inventory"] = Value::Array(inv);
            let rk_view = f["usedFishingRod"].as_str().unwrap_or("basicrod").to_string();
            let rlu = add_rod_exp(&mut f, &rk_view, te);
            let plu = add_player_exp(&mut f, te);
            f["fishingPending"] = json!([]);
            let mut txt = format!(
                "*TANGKAPAN DIAMBIL!*\n\n+{}\n+{} EXP\n+{} ikan\n",
                format_money(tv),
                te,
                jumlah_tangkapan
            );
            if !nf.is_empty() {
                txt.push_str(&format!("\n*Ikan Baru:* {}", nf.join(", ")));
            }
            if !nm.is_empty() {
                txt.push_str(&format!("\n*Mutasi Baru:* {}", nm.join(", ")));
            }
            if let Some(rlu) = rlu {
                txt.push_str(&format!("\n\n{rlu}"));
            }
            if plu {
                txt.push_str(&format!("\n*LEVEL UP! Level {}*", f["level"]));
            }
            balasan = txt;
        }
        // ---- sell (`fisch.js:232-255`) ----
        "sell" => {
            let Some(inventory) = f["inventory"].as_array().cloned() else {
                return Reply::text("_Inventory kosong!_ `.fisht mancing` _dulu._");
            };
            if inventory.is_empty() {
                return Reply::text("_Inventory kosong!_ `.fisht mancing` _dulu._");
            }
            let tv: f64 = inventory.iter().map(|i| i["price"].as_f64().unwrap_or(0.0)).sum();
            let sb = UPGRADES[2].4 * f["sellUpgrade"].as_f64().unwrap_or(0.0);
            let rsm = rod_aktif(&f).and_then(|r| r["sellMultiplier"].as_f64()).unwrap_or(0.0);
            let fv = (tv * (1.0 + sb + rsm)).round();
            let jumlah = inventory.len();
            f["money"] = json!(money_of(&f) + fv);
            f["totalEarned"] = json!(f["totalEarned"].as_f64().unwrap_or(0.0) + fv);
            f["inventory"] = json!([]);
            balasan = format!(
                "*IKAN TERJUAL!*\n\nJumlah: {jumlah}\nTotal: {}\nSaldo: {}",
                format_money(fv),
                format_money(money_of(&f))
            );
        }
        // ---- me (`fisch.js:257-268`) ----
        "me" => {
            let rod = rod_aktif(&f);
            let pulau = match f["currentIsland"].as_str() {
                Some(k) => {
                    let n = ISLANDS.iter().find(|(ik, ..)| *ik == k).map(|(_, n, _)| *n);
                    n.unwrap_or(k)
                }
                None => f["currentIsland"].as_str().unwrap_or(""),
            };
            balasan = format!(
                "*PROFIL PEMANCING*\n\n*Level:* {} _({}/{}) EXP)_\n*Uang:* {}\n*Ikan:* {}\n*Rod:* {} _(Lv.{})_\n*Pulau:* {}\n*Streak:* {}\n*Prestige:* {}\n*Tokens:* {}\n*Tickets:* {}\n*FishBook:* {}\n*Mutasi:* {}\n*Upgrades:*\n  _Luck: Lv.{}_\n  _Speed: Lv.{}_\n  _Sell: Lv.{}_",
                f["level"],
                f["exp"],
                f["expToNextLevel"],
                format_money(money_of(&f)),
                f["fishCaught"],
                rod.map(|r| r["name"].as_str().unwrap_or("Basic")).unwrap_or("Basic"),
                rod.map(|r| r["level"].as_i64().unwrap_or(1)).unwrap_or(1),
                pulau,
                f["streak"].as_i64().unwrap_or(0),
                f["prestige"].as_i64().unwrap_or(0),
                f["prestigeTokens"].as_i64().unwrap_or(0),
                f["gachaTickets"].as_i64().unwrap_or(0),
                f["fishFound"].as_array().map_or(0, Vec::len),
                f["mutationFound"].as_array().map_or(0, Vec::len),
                f["luckUpgrade"].as_i64().unwrap_or(0),
                f["speedUpgrade"].as_i64().unwrap_or(0),
                f["sellUpgrade"].as_i64().unwrap_or(0),
            );
        }
        // ---- stats (`fisch.js:270-282`) ----
        "stats" => {
            let rod = rod_aktif(&f);
            let (st_luck, st_speed, st_sell) = rod
                .map(|r| upgraded_stats(&f, r))
                .unwrap_or((0.0, 0.0, 0.0));
            let mut txt = String::new();
            match rod {
                Some(r) => {
                    let max_level = r["maxLevel"].as_i64().unwrap_or(5);
                    let exp = r["exp"].as_i64().unwrap_or(0);
                    let next = r["expToNextLevel"].as_i64().unwrap_or(100);
                    let combo = r["comboFish"].as_i64().unwrap_or(1);
                    txt.push_str(&format!(
                        "*STATS DETAIL*\n\n*Rod: {}*\n  _Lv.{}/{max_level} | EXP {exp}/{next}_\n  _Luck: {:.1}% | Speed: {:.1}%_\n  _Sell: +{:.1}% | Combo: {combo}_\n",
                        r["name"].as_str().unwrap_or("None"),
                        r["level"].as_i64().unwrap_or(1),
                        st_luck * 100.0,
                        st_speed * 100.0,
                        st_sell * 100.0,
                    ));
                    if let Some(ek) = r["enchant"].as_str() {
                        let e = enchant_of(ek);
                        txt.push_str(&format!(
                            "  _Enchant: {} ({})_\n",
                            e.map(|e| e.1).unwrap_or(ek),
                            e.map(|e| e.2).unwrap_or("?"),
                        ));
                    }
                }
                None => txt.push_str("*STATS DETAIL*\n\n*Rod: None*\n"),
            }
            txt.push_str(&format!(
                "\n*Upgrades*\n  _Luck: Lv.{} (+{:.1}%)_\n  _Speed: Lv.{} (+{:.1}%)_\n  _Sell: Lv.{} (+{:.1}%)_",
                f["luckUpgrade"].as_i64().unwrap_or(0),
                UPGRADES[0].4 * f["luckUpgrade"].as_f64().unwrap_or(0.0) * 100.0,
                f["speedUpgrade"].as_i64().unwrap_or(0),
                UPGRADES[1].4 * f["speedUpgrade"].as_f64().unwrap_or(0.0) * 100.0,
                f["sellUpgrade"].as_i64().unwrap_or(0),
                UPGRADES[2].4 * f["sellUpgrade"].as_f64().unwrap_or(0.0) * 100.0,
            ));
            balasan = txt;
        }
        // ---- fishbook (`fisch.js:283-298`) ----
        "fishbook" => {
            let kosong = f["fishFound"].as_array().is_none_or(|a| a.is_empty());
            if kosong {
                return Reply::text("_Fish Book kosong!_ `.fisht mancing` _dulu_");
            }
            let found: Vec<String> = f["fishFound"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
            let mut txt = format!("*FISH BOOK* _({} spesies)_\n\n", found.len());
            for (_, nama_isle, list) in ISLANDS {
                let dapat: Vec<&(&str, &str, f64, f64, f64)> =
                    list.iter().filter(|fish| found.iter().any(|c| c == fish.0)).collect();
                if !dapat.is_empty() {
                    txt.push_str(&format!("*{nama_isle}*\n"));
                    for fish in dapat {
                        txt.push_str(&format!("  {} {}\n", rc(fish.1), fish.0));
                    }
                    txt.push('\n');
                }
            }
            balasan = txt.trim_end().to_string();
        }
        // ---- mutbook (`fisch.js:300-310`) ----
        "mutbook" => {
            let kosong = f["mutationFound"].as_array().is_none_or(|a| a.is_empty());
            if kosong {
                return Reply::text("_Mutation Book kosong!_");
            }
            let found: Vec<String> = f["mutationFound"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
            let mut txt = format!("*MUTATION BOOK* _({})_\n\n", found.len());
            for m in &found {
                if let Some((_, mult, _)) = MUTATIONS.iter().find(|(n, ..)| n == m) {
                    txt.push_str(&format!("*{m}* _x{mult}_\n"));
                }
            }
            balasan = txt.trim_end().to_string();
        }
        // ---- travel (`fisch.js:312-356`) ----
        "travel" => {
            if sa.is_empty() {
                let mut txt = String::from("*DAFTAR PULAU*\n\n");
                for (k, nama_isle, _) in ISLANDS {
                    let req = TRAVEL.iter().find(|(rk, ..)| rk == k);
                    let ok = f["travelFound"].as_array().is_some_and(|a| a.iter().any(|v| v.as_str() == Some(k)));
                    let kini = f["currentIsland"].as_str() == Some(*k);
                    txt.push_str(&format!(
                        "{} *{nama_isle}*{}\n",
                        if ok { "[OK]" } else { "[LOCK]" },
                        if kini { " _< Now_" } else { "" }
                    ));
                    match req {
                        Some((_, money, fish)) if *money > 0.0 => {
                            txt.push_str(&format!("   _{} | {fish} fish_\n", format_money(*money)));
                        }
                        _ => txt.push_str("   _Free_\n"),
                    }
                }
                txt.push_str("\n`.fisht travel <pulau>`");
                balasan = txt;
            } else {
                let tk = sa[0].to_lowercase();
                if !ISLANDS.iter().any(|(k, ..)| *k == tk) {
                    return Reply::text("_Pulau tidak ada!_ `.fisht travel`");
                }
                if f["currentIsland"].as_str() == Some(tk.as_str()) {
                    let nama = ISLANDS.iter().find(|(k, ..)| *k == tk).map(|(_, n, _)| *n).unwrap_or("");
                    return Reply::text(format!("_Sudah di {nama}!_"));
                }
                let req = TRAVEL.iter().find(|(rk, ..)| *rk == tk);
                if let Some((_, money, fish)) = req
                    && *money > 0.0 {
                        if money_of(&f) < *money {
                            return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(*money)));
                        }
                        if i64_of(&f["fishCaught"]) < *fish {
                            return Reply::text(format!("_Ikan kurang! Butuh {fish}_"));
                        }
                        f["money"] = json!(money_of(&f) - money);
                    }
                let sudah = f["travelFound"].as_array().is_some_and(|a| a.iter().any(|v| v.as_str() == Some(tk.as_str())));
                if !sudah {
                    let mut tf = f["travelFound"].as_array().cloned().unwrap_or_default();
                    tf.push(json!(tk));
                    f["travelFound"] = Value::Array(tf);
                }
                f["currentIsland"] = json!(tk);
                let (nama, jumlah_ikan) = ISLANDS
                    .iter()
                    .find(|(k, ..)| *k == tk)
                    .map(|(_, n, l)| (*n, l.len()))
                    .unwrap_or(("", 0));
                balasan = format!(
                    "*PINDAH PULAU!*\n\nSekarang di *{nama}*\n{jumlah_ikan} jenis ikan tersedia"
                );
            }
        }
        // ---- shop (`fisch.js:358-371`) ----
        "shop" => {
            let mut txt = String::from("*TOKO FISHING ROD*\n\n");
            for (_, name, luck, speed, combo, _, _, _, price, desc, _) in RODS {
                if *price > 0.0 {
                    txt.push_str(&format!(
                        "*{name}*\n   _{}_\n   _Luck +{:.0}% | Speed +{:.0}% | Combo: {combo}_\n   _{desc}_\n\n",
                        format_money(*price),
                        luck * 100.0,
                        speed * 100.0,
                    ));
                }
            }
            txt.push_str("`.fisht buy <rod>`");
            balasan = txt;
        }
        // ---- buy (`fisch.js:373-397`) ----
        "buy" => {
            let rk = sa.first().map(|s| s.to_lowercase()).unwrap_or_default();
            if rk.is_empty() {
                return Reply::text("_Tentukan rod!_ `.fisht shop`");
            }
            let Some(def) = RODS.iter().find(|(k, ..)| *k == rk) else {
                return Reply::text("_Rod tidak ada!_ `.fisht shop`");
            };
            if f["fishingRods"].get(&rk).is_some() {
                let nama = def.1;
                return Reply::text(format!("_Sudah punya {nama}!_"));
            }
            if def.8 == 0.0 {
                return Reply::text("_Rod ini dari Token/Prestige!_");
            }
            if money_of(&f) < def.8 {
                return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(def.8)));
            }
            f["money"] = json!(money_of(&f) - def.8);
            f["fishingRods"][&rk] = rod_def(&rk);
            balasan = format!(
                "*ROD DIBELI!*\n\n*{}*\n_Ketik_ `.fisht equip {rk}` _untuk memasang_",
                def.1
            );
        }
        // ---- equip (`fisch.js:399-415`) ----
        "equip" => {
            let rk = sa.first().map(|s| s.to_lowercase()).unwrap_or_default();
            if rk.is_empty() {
                return Reply::text("_Tentukan rod!_ `.fisht rods`");
            }
            let Some(rod) = f["fishingRods"].get(&rk).cloned() else {
                return Reply::text("_Tidak punya rod ini!_ `.fisht rods`");
            };
            f["usedFishingRod"] = json!(rk);
            balasan = format!(
                "*ROD DIPASANG!*\n\n*{}* _sekarang aktif_",
                rod["name"].as_str().unwrap_or("")
            );
        }
        // ---- rods (`fisch.js:417-435`) ----
        "rods" => {
            let kosong = f["fishingRods"].as_object().is_none_or(|m| m.is_empty());
            if kosong {
                return Reply::text("_Tidak punya rod!_");
            }
            let mut txt = String::from("*KOLEKSI ROD*\n\n");
            let rods = f["fishingRods"].as_object().unwrap();
            for (k, rod) in rods {
                let aktif = f["usedFishingRod"].as_str() == Some(k.as_str());
                txt.push_str(&format!(
                    "*{}*{}\n  _Lv.{}/{} | Luck {:.0}% | Speed {:.0}%_\n",
                    rod["name"].as_str().unwrap_or(""),
                    if aktif { " _AKTIF_" } else { "" },
                    rod["level"].as_i64().unwrap_or(1),
                    rod["maxLevel"].as_i64().unwrap_or(5),
                    rod["luck"].as_f64().unwrap_or(0.0) * 100.0,
                    rod["speed"].as_f64().unwrap_or(0.0) * 100.0,
                ));
                if let Some(ek) = rod["enchant"].as_str() {
                    txt.push_str(&format!(
                        "  _Enchant: {}_\n",
                        enchant_of(ek).map(|e| e.1).unwrap_or(ek)
                    ));
                }
            }
            txt.push_str("\n`.fisht equip <rod>`");
            let _ = rods.len();
            balasan = txt;
        }
        // ---- enchant (`fisch.js:436-468`) ----
        "enchant" => {
            let rk = f["usedFishingRod"].as_str().unwrap_or("basicrod").to_string();
            let Some(rod) = f["fishingRods"].get(&rk).cloned() else {
                return Reply::text("_Rod aktif tidak ada!_");
            };
            let ek = sa.first().map(|s| s.to_lowercase()).unwrap_or_default();
            if ek.is_empty() {
                if let Some(current) = rod["enchant"].as_str() {
                    let e = enchant_of(current);
                    return Reply::text(format!(
                        "_Enchant saat ini: *{}* ({})_\n`.fisht enchant <key>` untuk ganti",
                        e.map(|e| e.1).unwrap_or(current),
                        e.map(|e| e.2).unwrap_or("?"),
                    ));
                }
                return Reply::text("_Tentukan enchant!_ `.fisht enchants`");
            }
            let Some(ench) = enchant_of(&ek) else {
                return Reply::text("_Enchant tidak ada!_ `.fisht enchants`");
            };
            let cost = enc_cost(ench.2);
            if money_of(&f) < cost {
                return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(cost)));
            }
            f["money"] = json!(money_of(&f) - cost);
            f["fishingRods"][&rk]["enchant"] = json!(ek);
            balasan = format!(
                "*ENCHANT DIPASANG!*\n\n*{}* _({})_ ke *{}*\n_{}_\n_Biaya: {}_",
                ench.1,
                ench.2,
                rod["name"].as_str().unwrap_or(""),
                ench.7,
                format_money(cost),
            );
        }
        // ---- enchants (`fisch.js:470-499`) ----
        "enchants" => {
            let mut txt = String::from("*DAFTAR ENCHANTMENT*\n\n");
            for rarity in ["common", "rare", "epic", "legendary", "mythic", "godly", "secret"] {
                let list: Vec<&Enchant> =
                    ENCHANTS.iter().filter(|e| e.2 == rarity).collect();
                if list.is_empty() {
                    continue;
                }
                txt.push_str(&format!(
                    "{} *{}* _{}_\n",
                    rc(rarity),
                    rarity.to_uppercase(),
                    format_money(enc_cost(rarity))
                ));
                for e in list {
                    txt.push_str(&format!("  `{}`: {} _{}_\n", e.0, e.1, e.7));
                }
                txt.push('\n');
            }
            balasan = format!("{}\n`.fisht enchant <key>`", txt.trim_end());
        }
        // ---- rodup (`fisch.js:501-523`) ----
        "rodup" => {
            let rk = f["usedFishingRod"].as_str().unwrap_or("basicrod").to_string();
            let Some(rod) = f["fishingRods"].get(&rk).cloned() else {
                return Reply::text("_Rod aktif tidak ada!_");
            };
            if rod["level"].as_i64().unwrap_or(1) >= rod["maxLevel"].as_i64().unwrap_or(5) {
                return Reply::text("_Rod sudah max level!_");
            }
            let dasar = (rod["price"].as_f64().unwrap_or(0.0) * 0.1 * rod["level"].as_f64().unwrap_or(1.0)).floor() as i64;
            // `... || 10000 * rod.level` — 0 jatuh ke fallback.
            let cost = if dasar == 0 { 10000 * rod["level"].as_i64().unwrap_or(1) } else { dasar };
            if money_of(&f) < cost as f64 {
                return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(cost as f64)));
            }
            f["money"] = json!(money_of(&f) - cost as f64);
            let xp = (rod["expToNextLevel"].as_f64().unwrap_or(100.0) * 0.5).floor() as i64;
            let res = add_rod_exp(&mut f, &rk, xp);
            balasan = match res {
                Some(res) => format!("*ROD UPGRADED!*\n\n{res}\n_Biaya: {}_", format_money(cost as f64)),
                None => format!("*ROD EXP UP!*\n\n+{xp} EXP\n_Biaya: {}_", format_money(cost as f64)),
            };
        }
        // ---- daily (`fisch.js:525-556`) ----
        "daily" => {
            let now = crate::common::now_ms();
            if let Some(last) = f["lastDaily"].as_str() {
                let lama = date_ms(last);
                if now as f64 - lama < 86_400_000.0 {
                    let tunggu = ((86_400_000.0 - (now as f64 - lama)) / 3_600_000.0).ceil() as i64;
                    return Reply::text(format!("_Daily sudah diambil! Tunggu *{tunggu}* jam._"));
                }
            }
            let lama = f["lastDaily"].as_str().map(date_ms).unwrap_or(0.0);
            let berlanjut = lama != 0.0 && (now as f64 - lama) < 172_800_000.0;
            f["dailyStreak"] = json!(if berlanjut {
                f["dailyStreak"].as_i64().unwrap_or(0) + 1
            } else {
                1
            });
            let streak = f["dailyStreak"].as_i64().unwrap_or(1);
            let mut rw = &DAILY_REWARDS[0];
            for r in DAILY_REWARDS {
                if streak >= r.0 {
                    rw = r;
                }
            }
            f["money"] = json!(money_of(&f) + rw.1);
            f["gachaTickets"] = json!(f["gachaTickets"].as_i64().unwrap_or(0) + rw.2);
            f["lastDaily"] = json!(ms_ke_iso(now));
            balasan = format!(
                "*DAILY REWARD!*\n\n*Streak:* {streak} hari\n+{}\n+{} Tiket Gacha\n*Saldo:* {}",
                format_money(rw.1),
                rw.2,
                format_money(money_of(&f))
            );
        }
        // ---- gacha (`fisch.js:558-627`) ----
        "gacha" => {
            let pakai_tiket = sa.first().map(|s| s.to_lowercase()).as_deref() == Some("ticket");
            if pakai_tiket {
                if f["gachaTickets"].as_i64().unwrap_or(0) < 1 {
                    return Reply::text(format!(
                        "_Tiket habis! Punya {}_",
                        f["gachaTickets"].as_i64().unwrap_or(0)
                    ));
                }
                f["gachaTickets"] = json!(f["gachaTickets"].as_i64().unwrap_or(0) - 1);
            } else {
                if money_of(&f) < GACHA_COST_COINS {
                    return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(GACHA_COST_COINS)));
                }
                f["money"] = json!(money_of(&f) - GACHA_COST_COINS);
            }
            let (item, is_ssr, pity) = gacha_pull(&mut f);
            let mut txt = String::from("*GACHA RESULT!*\n\n");
            match item["type"].as_str().unwrap_or("") {
                "rod" => {
                    let value = item["value"].as_str().unwrap_or("");
                    let def = RODS.iter().find(|(k, ..)| *k == value);
                    if def.is_some() && f["fishingRods"].get(value).is_none() {
                        f["fishingRods"][value] = rod_def(value);
                        txt.push_str(&format!("DAPAT ROD: *{}*\n", item["label"].as_str().unwrap_or("")));
                    } else if f["fishingRods"].get(value).is_some() {
                        let harga = def.map(|d| d.8).unwrap_or(0.0);
                        let refund = ((harga * 0.3).floor() as i64).max(0);
                        let refund = if refund == 0 { 100000 } else { refund };
                        f["money"] = json!(money_of(&f) + refund as f64);
                        txt.push_str(&format!(
                            "Duplikat: *{}* _+{}_",
                            item["label"].as_str().unwrap_or(""),
                            format_money(refund as f64)
                        ));
                        txt.push('\n');
                    }
                }
                "tickets" => {
                    let v = i64_of(&item["value"]);
                    f["gachaTickets"] = json!(f["gachaTickets"].as_i64().unwrap_or(0) + v);
                    txt.push_str(&format!("+{v} Tiket\n"));
                }
                "tokens" => {
                    let v = i64_of(&item["value"]);
                    f["prestigeTokens"] = json!(f["prestigeTokens"].as_i64().unwrap_or(0) + v);
                    txt.push_str(&format!("+{v} Tokens\n"));
                }
                "coins" => {
                    let v = item["value"].as_f64().unwrap_or(0.0);
                    f["money"] = json!(money_of(&f) + v);
                    txt.push_str(&format!("+{}\n", format_money(v)));
                }
                "enchant_scroll" => {
                    let rarity = item["value"].as_str().unwrap_or("");
                    let avail: Vec<&Enchant> =
                        ENCHANTS.iter().filter(|e| e.2 == rarity).collect();
                    if !avail.is_empty() {
                        use rand::Rng as _;
                        let idx = if avail.len() > 1 {
                            rand::rng().random_range(0..avail.len())
                        } else {
                            0
                        };
                        let ed = avail[idx];
                        let rk = f["usedFishingRod"].as_str().unwrap_or("basicrod").to_string();
                        if f["fishingRods"].get(&rk).is_some() {
                            f["fishingRods"][&rk]["enchant"] = json!(ed.0);
                            txt.push_str(&format!(
                                "Enchant: *{}* _({})_ ke rod!\n",
                                ed.1,
                                rarity
                            ));
                        }
                    }
                }
                "xp_boost" => {
                    let boost = (f["expToNextLevel"].as_f64().unwrap_or(0.0) * 0.5).floor();
                    f["exp"] = json!(f["exp"].as_f64().unwrap_or(0.0) + boost);
                    txt.push_str(&format!("XP Boost x{}!\n", item["value"]));
                }
                _ => txt.push_str(&format!("{}\n", item["label"].as_str().unwrap_or(""))),
            }
            if is_ssr {
                txt.push_str("\n*SSR PULL!*");
            }
            if pity {
                txt.push_str("\n*Pity Activated!*");
            }
            txt.push_str(&format!(
                "\n\n_Pity: {}/{}_\n_Saldo: {} | Tiket: {}_",
                f["gachaPity"].as_i64().unwrap_or(0),
                GACHA_PITY_LIMIT,
                format_money(money_of(&f)),
                f["gachaTickets"].as_i64().unwrap_or(0),
            ));
            balasan = txt;
        }
        // ---- upgrade (`fisch.js:628-662`) ----
        "upgrade" => {
            let tipe = sa.first().map(|s| s.to_lowercase()).unwrap_or_default();
            let Some(upg) = upgrade_of(&tipe) else {
                let mut txt = String::from("*UPGRADE SHOP*\n\n");
                for (k, name, desc, max_level, _, base_cost, mult) in UPGRADES {
                    let lv = f[format!("{k}Upgrade").as_str()].as_i64().unwrap_or(0);
                    let next = (base_cost * mult.powi(lv as i32)).floor();
                    txt.push_str(&format!(
                        "*{name}* _(Lv.{lv}/{max_level})_\n  _{desc}_\n  {}\n\n",
                        if lv >= *max_level {
                            "_MAXED_".to_string()
                        } else {
                            format!("_Next: {}_", format_money(next))
                        }
                    ));
                }
                txt.push_str("`.fisht upgrade <luck/speed/sell>`");
                return kirim(&ctx, &txt);
            };
            let lv = f[format!("{tipe}Upgrade").as_str()].as_i64().unwrap_or(0);
            if lv >= upg.3 {
                return Reply::text("_Sudah max!_");
            }
            let cost = (upg.5 * upg.6.powi(lv as i32)).floor();
            if money_of(&f) < cost {
                return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(cost)));
            }
            f["money"] = json!(money_of(&f) - cost);
            f[format!("{tipe}Upgrade").as_str()] = json!(lv + 1);
            balasan = format!(
                "*{} LEVEL UP!*\n\n_Level {}_\n_Biaya: {}_\n_{}_",
                upg.1,
                lv + 1,
                format_money(cost),
                upg.2
            );
        }
        // ---- prestige (`fisch.js:664-726`) ----
        "prestige" => {
            let cp = f["prestige"].as_i64().unwrap_or(0);
            let konfirm = sa.first().map(|s| s.to_lowercase()).as_deref() == Some("confirm");
            if konfirm {
                let reqs: [(i64, f64); 5] = [
                    (500, 1e10),
                    (1500, 1e12),
                    (4000, 1e14),
                    (10000, 1e19),
                    (25000, 1e22),
                ];
                let Some((fish, money)) = reqs.get(cp as usize).copied() else {
                    return Reply::text("_Sudah max prestige!_");
                };
                if i64_of(&f["fishCaught"]) < fish {
                    return Reply::text(format!("_Ikan kurang! Butuh {fish}_"));
                }
                if money_of(&f) < money {
                    return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(money)));
                }
                f["prestige"] = json!(cp + 1);
                f["money"] = json!((money_of(&f) * 0.1).floor());
                f["fishCaught"] = json!(0);
                f["streak"] = json!(0);
                let tokens = [50, 150, 500, 1000, 5000][cp.min(4) as usize];
                f["prestigeTokens"] = json!(f["prestigeTokens"].as_i64().unwrap_or(0) + tokens);
                let titles = [
                    "Pemancing Baru",
                    "Veteran",
                    "Master Angler",
                    "Legend",
                    "Transcendent",
                    "God of Fishing",
                ];
                let prestige_baru = f["prestige"].as_i64().unwrap_or(0);
                if cp == 0 && f["fishingRods"].get("prestigerod").is_none() {
                    f["fishingRods"]["prestigerod"] = rod_def("prestigerod");
                }
                if cp == 2 && f["fishingRods"].get("cosmicrod").is_none() {
                    f["fishingRods"]["cosmicrod"] = rod_def("cosmicrod");
                }
                balasan = format!(
                    "*PRESTIGE UP!*\n\n*Title:* {}\n*Tokens:* {}\n\n_Uang -90%, fish count reset_",
                    titles.get(prestige_baru as usize).unwrap_or(&titles[5]),
                    f["prestigeTokens"]
                );
            } else {
                let mut txt = format!(
                    "*PRESTIGE SYSTEM*\n\n*Prestige:* {cp}\n*Tokens:* {}\n\n",
                    f["prestigeTokens"].as_i64().unwrap_or(0)
                );
                let all: [(i64, i64, f64, &str); 5] = [
                    (1, 500, 1e10, "Prestige Rod + 50 tokens"),
                    (2, 1500, 1e12, "Luck +20% + 150 tokens"),
                    (3, 4000, 1e14, "Cosmic Rod + 500 tokens"),
                    (4, 10000, 1e19, "2x EXP + 1000 tokens"),
                    (5, 25000, 1e22, "Eternity Rod + 5000 tokens"),
                ];
                for (lv, fish, money, rw) in all {
                    txt.push_str(&format!(
                        "{} *P{lv}*: _{fish} fish | {}_\n  _{rw}_\n\n",
                        if cp >= lv { "[OK]" } else { "[LOCK]" },
                        format_money(money)
                    ));
                }
                txt.push_str("`.fisht prestige confirm` _(hati-hati!)_");
                balasan = txt;
            }
        }
        // ---- tokens (`fisch.js:728-773`) ----
        "tokens" => {
            if sa.is_empty() {
                let mut txt = format!(
                    "*TOKEN STORE*\n\n*Tokens:* {}\n\n",
                    f["prestigeTokens"].as_i64().unwrap_or(0)
                );
                for (_, name, cost, _, _, _) in TOKEN_SHOP {
                    txt.push_str(&format!("*{name}* _{cost} tokens_\n"));
                }
                txt.push_str("\n`.fisht tokens <id>`");
                balasan = txt;
            } else {
                let iid = sa[0].to_lowercase();
                let Some(item) = TOKEN_SHOP.iter().find(|(id, ..)| *id == iid) else {
                    return Reply::text("_Item tidak ada!_ `.fisht tokens`");
                };
                if f["prestigeTokens"].as_i64().unwrap_or(0) < item.2 {
                    return Reply::text(format!("_Tokens kurang! Butuh {}_", item.2));
                }
                f["prestigeTokens"] = json!(f["prestigeTokens"].as_i64().unwrap_or(0) - item.2);
                match item.3 {
                    "rod" => {
                        let value = item.4;
                        if RODS.iter().any(|(k, ..)| *k == value)
                            && f["fishingRods"].get(value).is_none()
                        {
                            f["fishingRods"][value] = rod_def(value);
                        } else {
                            f["prestigeTokens"] =
                                json!(f["prestigeTokens"].as_i64().unwrap_or(0) + item.2);
                            return Reply::text("_Sudah punya rod ini!_");
                        }
                    }
                    "tickets" => {
                        f["gachaTickets"] =
                            json!(f["gachaTickets"].as_i64().unwrap_or(0) + item.5);
                    }
                    "coins" => {
                        f["money"] = json!(money_of(&f) + item.5 as f64);
                    }
                    _ => {}
                }
                balasan = format!(
                    "*ITEM DIBELI!*\n\n*{}* _seharga {} tokens_",
                    item.1, item.2
                );
            }
        }
        // ---- jackpot (`fisch.js:775-838`) ----
        "jackpot" => {
            if sa.is_empty() {
                let mut txt = String::from(
                    "*JACKPOT SYSTEM*\n\n_Sistem jackpot dengan hadiah besar!_\n_Bisa dapat _Premium_, _Partner_, _Energi_, _Limit_, bahkan _UNLIMITED_!_\n\n",
                );
                for (_, name, cost, weight, rewards) in JACKPOT_POOLS {
                    txt.push_str(&format!(
                        "*{name}*\n  _Biaya: {}_\n  _Rate: {weight}%_\n  _Rewards:_\n",
                        format_money(*cost)
                    ));
                    for rw in *rewards {
                        let nilai = if rw.1 == rw.2 {
                            format!("{}", rw.1)
                        } else {
                            format!("{}-{}", rw.1, rw.2)
                        };
                        txt.push_str(&format!(
                            "    _{}: {} ({}%)_\n",
                            label_jackpot(rw.0),
                            nilai,
                            rw.3
                        ));
                    }
                    txt.push('\n');
                }
                txt.push_str("`.fisht jackpot <mini/mega/ultra/legend>`");
                balasan = txt;
            } else {
                let pool_id = sa[0].to_lowercase();
                let Some((_, nama, cost, _, _)) = JACKPOT_POOLS.iter().find(|(id, ..)| *id == pool_id) else {
                    return Reply::text("_Tier tidak ada!_ `.fisht jackpot`");
                };
                if money_of(&f) < *cost {
                    return Reply::text(format!("_Uang kurang! Butuh {}_", format_money(*cost)));
                }
                f["money"] = json!(money_of(&f) - cost);
                let Some((tipe, val)) = jackpot_pull(&pool_id) else {
                    return Reply::text("_Gagal! Coba lagi._");
                };
                let desc = apply_jackpot(&mut f, &mut user_luar, &tipe, val);
                let besar = matches!(
                    tipe.as_str(),
                    "premium_7d" | "premium_30d" | "partner_7d" | "partner_30d"
                        | "unlimited_energi" | "unlimited_limit"
                );
                let mut txt = format!("*{}!*\n\n", nama.to_uppercase());
                if besar {
                    txt = format!("*{}!*\n\n*JACKPOT BESAR!*\n\n", nama.to_uppercase());
                }
                txt.push_str(&format!(
                    "{desc}\n\n_Biaya: {}_\n_Saldo: {}_",
                    format_money(*cost),
                    format_money(money_of(&f))
                ));
                balasan = txt;
            }
        }
        // ---- top (`fisch.js:840-868`) ----
        "top" => {
            let mut rankings: Vec<(String, i64, f64, i64, i64)> = Vec::new();
            if let Ok(teks) = ctx.berkas().baca_teks("database/main/settings.json")
                && let Ok(root) = serde_json::from_str::<Value>(&teks)
                    && let Some(users) = root.get("users").and_then(Value::as_object) {
                        for (jid, ud) in users {
                            if let Some(fs) = ud.get("fisch").filter(|v| !v.is_null()) {
                                rankings.push((
                                    jid.clone(),
                                    fs["fishCaught"].as_i64().unwrap_or(0),
                                    fs["money"].as_f64().unwrap_or(0.0),
                                    fs["level"].as_i64().unwrap_or(1),
                                    fs["prestige"].as_i64().unwrap_or(0),
                                ));
                            }
                        }
                    }
            if rankings.is_empty() {
                return Reply::text("_Belum ada pemain!_");
            }
            rankings.sort_by(|a, b| {
                b.4.cmp(&a.4)
                    .then(b.1.cmp(&a.1))
                    .then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
            });
            let mut txt = String::from("*LEADERBOARD FISCHIT*\n\n");
            for (i, (jid, fish, money, _, prestige)) in rankings.iter().take(10).enumerate() {
                let medal: String = match i {
                    0 => "1.".into(),
                    1 => "2.".into(),
                    2 => "3.".into(),
                    n => format!("{}.", n + 1),
                };
                txt.push_str(&format!(
                    "{medal} @{jid}\n  _P{prestige} | {fish} fish | {}_\n",
                    format_money(*money)
                ));
            }
            balasan = txt.trim_end().to_string();
        }
        _ => return Reply::text("_Command fisht tidak dikenal._"),
    }

    // ---- tulis state (`db.markDirty("users")`) ----
    user["fisch"] = f;
    let mut data = json!({ "fisch": user["fisch"] });
    if user_luar.as_object().is_some_and(|m| !m.is_empty()) {
        for (k, v) in user_luar.as_object().unwrap() {
            data[k.clone()] = v.clone();
        }
    }
    let akhir = match balasan_seq {
        Some(pertama) => Reply::sequence(vec![pertama, kirim(&ctx, &balasan)]),
        None => kirim(&ctx, &balasan),
    };
    akhir.mutating(vec![Mutation::SetUser { jid: ctx.sender.clone(), data }])
}

/// `${name.replace(/[\u{1F300}-\u{1F9FF}]/gu, '').trim()}` (`fisch.js:203`).
fn strip_emoji(nama: &str) -> String {
    nama.chars()
        .filter(|c| !('\u{1F300}'..='\u{1F9FF}').contains(c))
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx_of(text: &str, command: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: command.into(),
            ..Ctx::default()
        }
    }

    fn ctx_dengan_fisch(fisch: Value) -> Ctx {
        let mut c = ctx_of("fisht", "fisht");
        c.user_data = json!({ "fisch": fisch });
        c
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("fisht").unwrap();
        assert_eq!(p.meta.name, "fisht");
        assert_eq!(p.meta.alias, ["fishit"]);
        assert_eq!(p.meta.category, "game");
        assert_eq!(p.meta.description, "Fishit - Fishing Rod Game");
        assert_eq!(p.meta.usage, ".fisht <command>");
        assert_eq!(p.meta.example, ".fisht help");
        assert_eq!(p.meta.cooldown, 3);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn alias_fishit_menunjuk_plugin_ini() {
        assert_eq!(
            oktz_plugin_api::find("fishit").map(|p| p.meta.name.as_str()),
            Some("fisht")
        );
    }

    #[test]
    fn format_money_mengikuti_js() {
        assert_eq!(format_money(0.0), "0");
        assert_eq!(format_money(999.0), "999");
        assert_eq!(format_money(1500.0), "1.5K");
        assert_eq!(format_money(1234.0), "1.23K");
        assert_eq!(format_money(25000.0), "25K");
        assert_eq!(format_money(1e6), "1M");
        assert_eq!(format_money(2.5e6), "2.5M");
        assert_eq!(format_money(1e10), "10B");
        assert_eq!(format_money(1e12), "1T");
    }

    #[test]
    fn bantuan_mencantumkan_semua_subcommand() {
        let r = fisht(ctx_of("", "fisht"));
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("*FISHIT GAME*"), "{t}");
        assert!(t.contains("`.fisht mancing`"));
        assert!(t.contains("`.fisht jackpot <tier>`"));
    }

    #[test]
    fn toggle_fishit_hanya_di_grup() {
        let r = fisht(ctx_of("on", "fishit"));
        assert_eq!(r.text_body().unwrap(), "_Toggle hanya di grup_");
    }

    #[test]
    fn toggle_fishit_on_di_grup_admin() {
        use oktz_plugin_api::registry::{GroupRoster, Participant};
        let mut c = ctx_of("on", "fishit");
        c.is_group = true;
        c.roster = GroupRoster {
            participants: vec![Participant {
                jid: c.sender.clone(),
                is_admin: true,
                is_owner: false,
                ..Default::default()
            }],
            ..Default::default()
        };
        c.group_data = json!({});
        let r = fisht(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("*FISHIT ENABLED*"), "{t}");
        // mutasi group data
        let mut ada = false;
        for m in r.mutations() {
            if let Mutation::SetGroup { data, .. } = m {
                assert_eq!(data["fishitEnabled"], true);
                ada = true;
            }
        }
        assert!(ada);
    }

    #[test]
    fn toggle_fishit_ditolak_untuk_member() {
        use oktz_plugin_api::registry::{GroupRoster, Participant};
        let mut c = ctx_of("on", "fishit");
        c.is_group = true;
        c.roster = GroupRoster {
            participants: vec![Participant {
                jid: c.sender.clone(),
                is_admin: false,
                is_owner: false,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(fisht(c).text_body().unwrap(), "_Hanya admin/owner_");
    }

    #[test]
    fn mancing_pertama_menghasilkan_tangkapan_dan_state_tersimpan() {
        let c = ctx_dengan_fisch(default_fisch());
        let mut c = c;
        c.text = "fisht".into();
        c.args = vec!["mancing".into()];
        let r = fisht(c);
        match r.without_mutations() {
            Reply::Sequence(v) => {
                assert_eq!(v.len(), 2, "Memancing... lalu HASIL");
                assert!(v[0].text_body().unwrap().contains("*_Memancing di Mousewood..._*"));
                let hasil = v[1].text_body().unwrap().to_string();
                assert!(hasil.contains("*HASIL MANCING!*"), "{hasil}");
                assert!(hasil.contains("`.fisht view` untuk mengambil!"));
            }
            other => panic!("harus Sequence, dapat {other:?}"),
        }
        // state: pending terisi, streak 1
        let mut ada_state = false;
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                let f = &data["fisch"];
                assert_eq!(f["streak"], 1);
                assert_eq!(f["fishingPending"].as_array().unwrap().len(), 1);
                assert!(f["lastFishTime"].as_f64().unwrap_or(0.0) > 0.0);
                ada_state = true;
            }
        }
        assert!(ada_state);
    }

    #[test]
    fn mancing_dengan_pending_tunggu_view() {
        let mut fisch = default_fisch();
        fisch["fishingPending"] = json!([{"name": "Trout", "price": 100, "kg": 2, "rarity": "common", "mutations": ["Normal"], "isMutated": false}]);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["mancing".into()];
        assert_eq!(
            fisht(c).text_body().unwrap(),
            "_Masih ada tangkapan!_ `.fisht view` _dulu._"
        );
    }

    #[test]
    fn mancing_cooldown_menghitung_detik() {
        let mut fisch = default_fisch();
        fisch["lastFishTime"] = json!(crate::common::now_ms() - 5000);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["mancing".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("_Tunggu *10* detik_"), "{t}");
    }

    #[test]
    fn view_mengambil_tangkapan_dan_exp_minimum_sepuluh() {
        let mut fisch = default_fisch();
        fisch["fishingPending"] = json!([
            {"name": "Trout", "price": 100, "kg": 2.5, "rarity": "common", "mutations": ["Normal"], "isMutated": false}
        ]);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["view".into()];
        let r = fisht(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.trim_start().starts_with("*TANGKAPAN DIAMBIL!*"), "{t}");
        assert!(t.contains("+100\n"), "{t}");
        assert!(t.contains("+10 EXP"), "exp minimal 10");
        assert!(t.contains("+1 ikan"));
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                let f = &data["fisch"];
                assert_eq!(f["money"], 300.0, "200 awal + 100");
                assert_eq!(f["fishCaught"], 1);
                assert_eq!(f["exp"], 10);
                assert!(f["fishingPending"].as_array().unwrap().is_empty());
                assert_eq!(f["inventory"].as_array().unwrap().len(), 1);
                assert_eq!(f["fishFound"].as_array().unwrap()[0], "Trout");
                return;
            }
        }
        panic!("harus SetUser");
    }

    #[test]
    fn view_tanpa_tangkapan() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["view".into()];
        assert_eq!(
            fisht(c).text_body().unwrap(),
            "_Tidak ada tangkapan._ `.fisht mancing` _dulu!_"
        );
    }

    #[test]
    fn sell_kosong_dan_dengan_isi() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["sell".into()];
        assert_eq!(
            fisht(c).text_body().unwrap(),
            "_Inventory kosong!_ `.fisht mancing` _dulu._"
        );
        let mut fisch = default_fisch();
        fisch["inventory"] = json!([
            {"name": "Trout", "price": 1000},
            {"name": "Carp", "price": 2000}
        ]);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["sell".into()];
        let r = fisht(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("*IKAN TERJUAL!*"), "{t}");
        assert!(t.contains("Jumlah: 2"));
        assert!(t.contains("Total: 3K"), "1.0 + 2.0 sell bonus 0 → 3000");
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                assert_eq!(data["fisch"]["money"], 3200.0);
                assert!(data["fisch"]["inventory"].as_array().unwrap().is_empty());
                return;
            }
        }
        panic!("harus SetUser");
    }

    #[test]
    fn daily_pertama_memberi_reward_streak_satu() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["daily".into()];
        let r = fisht(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.trim_start().starts_with("*DAILY REWARD!*"), "{t}");
        assert!(t.contains("*Streak:* 1 hari"));
        assert!(t.contains("+50K\n"), "reward streak 1 = 50000");
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                assert_eq!(data["fisch"]["dailyStreak"], 1);
                assert!(
                    data["fisch"]["lastDaily"].as_str().is_some_and(|s| s.ends_with('Z')),
                    "lastDaily ISO"
                );
                return;
            }
        }
        panic!("harus SetUser");
    }

    #[test]
    fn daily_dua_kali_ditolak() {
        let mut fisch = default_fisch();
        fisch["lastDaily"] = json!(ms_ke_iso(crate::common::now_ms()));
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["daily".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("_Daily sudah diambil! Tunggu *"), "{t}");
    }

    #[test]
    fn travel_daftar_pulau_dan_pindah() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["travel".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("*DAFTAR PULAU*"), "{t}");
        assert!(t.contains("[OK] *Mousewood* _< Now_"), "{t}");
        assert!(t.contains("[LOCK] *Roslit Bay*"));

        // pindah tanpa uang
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["travel".into(), "roslitbay".into()];
        assert_eq!(
            fisht(c).text_body().unwrap(),
            "_Uang kurang! Butuh 5K_"
        );
        // pindah dengan uang cukup
        let mut fisch = default_fisch();
        fisch["money"] = json!(20000.0);
        fisch["fishCaught"] = json!(10);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["travel".into(), "roslitbay".into()];
        let r = fisht(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("*PINDAH PULAU!*"), "{t}");
        assert!(t.contains("Sekarang di *Roslit Bay*"));
        assert!(t.contains("19 jenis ikan tersedia"));
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                assert_eq!(data["fisch"]["currentIsland"], "roslitbay");
                assert_eq!(data["fisch"]["money"], 15000.0);
                return;
            }
        }
        panic!("harus SetUser");
    }

    #[test]
    fn buy_dan_equip_rod() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["buy".into(), "ironrod".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("_Uang kurang! Butuh 25K_"), "{t}");

        let mut fisch = default_fisch();
        fisch["money"] = json!(50000.0);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["buy".into(), "ironrod".into()];
        let r = fisht(c);
        assert!(r.text_body().unwrap().contains("*ROD DIBELI!*"));
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                assert_eq!(data["fisch"]["money"], 25000.0);
                assert_eq!(data["fisch"]["fishingRods"]["ironrod"]["name"], "Iron Rod");
                return;
            }
        }
        panic!("harus SetUser");
    }

    #[test]
    fn upgrade_shop_dan_upgrade_luck() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["upgrade".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("*UPGRADE SHOP*"), "{t}");
        assert!(t.contains("_Next: 5M_"), "{t}");

        let mut fisch = default_fisch();
        fisch["money"] = json!(6e6);
        let mut c = ctx_dengan_fisch(fisch);
        c.args = vec!["upgrade".into(), "luck".into()];
        let r = fisht(c);
        assert!(r.text_body().unwrap().contains("*🍀 Luck Upgrade LEVEL UP!*"));
        for m in r.mutations() {
            if let Mutation::SetUser { data, .. } = m {
                assert_eq!(data["fisch"]["luckUpgrade"], 1);
                assert_eq!(data["fisch"]["money"], 1e6);
                return;
            }
        }
        panic!("harus SetUser");
    }

    #[test]
    fn jackpot_daftar_dengan_rate_pool() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["jackpot".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("*JACKPOT SYSTEM*"), "{t}");
        assert!(t.contains("_Rate: 60%_"), "{t}");
        assert!(t.contains("_Rate: 1%_"), "{t}");
        assert!(t.contains("UNLIMITED Energi"));
    }

    #[test]
    fn gacha_tanpa_uang_dan_dengan_tiket_kosong() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["gacha".into()];
        assert_eq!(fisht(c).text_body().unwrap(), "_Uang kurang! Butuh 5M_");
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["gacha".into(), "ticket".into()];
        assert_eq!(fisht(c).text_body().unwrap(), "_Tiket habis! Punya 0_");
    }

    #[test]
    fn top_tanpa_pemain() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["top".into()];
        assert_eq!(fisht(c).text_body().unwrap(), "_Belum ada pemain!_");
    }

    #[test]
    fn fishbook_kosong_dan_stats() {
        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["fishbook".into()];
        assert_eq!(fisht(c).text_body().unwrap(), "_Fish Book kosong!_ `.fisht mancing` _dulu_");

        let mut c = ctx_dengan_fisch(default_fisch());
        c.args = vec!["stats".into()];
        let t = fisht(c).text_body().unwrap().to_string();
        assert!(t.contains("*STATS DETAIL*"), "{t}");
        assert!(t.contains("*Rod: Basic Fishing Rod*"), "{t}");
        assert!(t.contains("_Luck: 0.0% | Speed: 0.0%_"), "{t}");
    }

    #[test]
    fn random_fish_dari_island_mousewood_berbentuk_benar() {
        let rod = rod_def("basicrod");
        for _ in 0..50 {
            let fish = random_fish(&rod, "mousewood").unwrap();
            assert!(fish["price"].as_f64().unwrap_or(0.0) > 0.0);
            assert!(fish["kg"].as_f64().unwrap_or(0.0) > 0.0);
            assert_eq!(fish["type"], "fish");
            let muts = fish["mutations"].as_array().unwrap();
            assert!(!muts.is_empty());
            assert!(muts[0] == "Normal" || fish["isMutated"].as_bool().unwrap());
        }
        assert!(random_fish(&rod, "bukanisland").is_none());
    }

    #[test]
    fn roll_mutations_normal_saat_tidak_kena() {
        let m = roll_mutations(1, -1.0); // chance pasti di bawah nol
        assert_eq!(m, ["Normal"]);
    }

    #[test]
    fn add_rod_exp_dan_add_player_exp_level_up() {
        let mut f = default_fisch();
        assert!(add_rod_exp(&mut f, "basicrod", 100).is_some(), "100/100 → naik");
        assert_eq!(f["fishingRods"]["basicrod"]["level"], 2);
        assert_eq!(f["fishingRods"]["basicrod"]["expToNextLevel"], 150);
        assert!(add_player_exp(&mut f, 100));
        assert_eq!(f["level"], 2);
        assert_eq!(f["expToNextLevel"], 130);
        // max level tidak naik
        let mut f2 = default_fisch();
        f2["fishingRods"]["basicrod"]["level"] = json!(5);
        assert!(add_rod_exp(&mut f2, "basicrod", 1000).is_none());
    }

    #[test]
    fn streak_bonus_mengikuti_js() {
        assert_eq!(streak_bonus(0), (1.0, 0.0));
        assert_eq!(streak_bonus(3), (1.1, 0.0));
        assert_eq!(streak_bonus(5), (1.2, 0.0));
        assert_eq!(streak_bonus(10), (1.35, 0.05));
        assert_eq!(streak_bonus(20), (1.5, 0.0));
        assert_eq!(streak_bonus(50), (2.0, 0.05));
        assert_eq!(streak_bonus(100), (3.0, 0.15));
    }

    #[test]
    fn gacha_pull_pity_mengisi_ulang_dan_ssr_reset() {
        let mut f = default_fisch();
        f["gachaPity"] = json!(GACHA_PITY_LIMIT);
        let (_, is_ssr, is_pity) = gacha_pull(&mut f);
        assert!(is_pity);
        assert!(is_ssr, "pity memaksa pool SSR");
        assert_eq!(f["gachaPity"], 0);
        // non-SSR menaikkan pity
        for _ in 0..500 {
            let mut f2 = default_fisch();
            let (_, ssr, _) = gacha_pull(&mut f2);
            if !ssr {
                assert_eq!(f2["gachaPity"], 1);
                break;
            }
        }
    }

    #[test]
    fn jackpot_pull_memberi_reward_dari_pool() {
        let (tipe, val) = jackpot_pull("mini").unwrap();
        assert!((1e6..=5e6).contains(&val) || tipe != "coins");
        assert!(jackpot_pull("bukan").is_none());
    }

    #[test]
    fn strip_emoji_mengikuti_regex_js() {
        assert_eq!(strip_emoji("🧬 Trout"), "Trout");
        assert_eq!(strip_emoji("Red Snapper"), "Red Snapper");
        assert_eq!(strip_emoji("  Salmon "), "Salmon");
    }
}
