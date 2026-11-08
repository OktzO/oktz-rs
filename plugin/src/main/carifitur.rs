//! `plugins/main/carifitur.js` — mencari fitur berdasarkan keyword dengan
//! detail lengkap.
//!
//! Sumber daftar fitur: JS membaca seluruh folder `plugins/` secara dinamis
//! (`fs.readdirSync` + `import()`); di Rust registry (`all_plugins()`) sudah
//! memuat semua plugin yang ter-port, jadi iterasi dilakukan ke sana.
//! `getCaseCommands()` dari `case/foto.js` tidak di-port (bukan bagian
//! registry plugin) — entri `cping`/`listallcase`/`listallplugin` tidak
//! muncul.
//!
//! `ponytail:` `sock.sendButton` dengan header gambar `getAssetBuffer("foto")`
//! dipetakan ke `Reply::interactive` + `CardHeader` image dari
//! `config.assets["foto"]` (idiom `main/menu.rs`). Reaksi `🕕`/`✅`/`❌`
//! dibuang: membungkus kartu dengan `with_reaction` membuat header media
//! tidak pernah di-upload.

use oktz_plugin_api::registry::{CardHeader, CardList, CardRow, CardSection, InteractiveButton};
use oktz_plugin_api::{Ctx, MediaSource, Reply, all_plugins, plugin};

/// Aset `foto` — `getAssetBuffer("foto")` (idiom `main/menu.rs`).
fn aset_foto(ctx: &Ctx) -> MediaSource {
    MediaSource::path(
        ctx.config()
            .assets
            .get("foto")
            .unwrap_or("./assets/image/foto.jpeg"),
    )
}

/// `levenshteinDistance` (`carifitur.js:23-42`) — DP penuh, apa adanya.
pub(crate) fn levenshtein(s1: &str, s2: &str) -> usize {
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1]
            } else {
                1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1])
            };
        }
    }
    dp[m][n]
}

/// `getSimilarity` (`carifitur.js:44-51`).
pub(crate) fn similarity(s1: &str, s2: &str) -> f64 {
    let max_len = s1.chars().count().max(s2.chars().count());
    if max_len == 0 {
        return 1.0;
    }
    let (a, b) = (s1.to_lowercase(), s2.to_lowercase());
    (max_len as f64 - levenshtein(&a, &b) as f64) / max_len as f64
}

/// `matchesKeyword` (`carifitur.js:53-70`).
pub(crate) fn cocok_keyword(teks: &str, keyword: &str) -> bool {
    if teks.is_empty() || keyword.is_empty() {
        return false;
    }
    let teks_lower = teks.to_lowercase();
    let keyword_lower = keyword.to_lowercase();
    if teks_lower.contains(&keyword_lower) || keyword_lower.contains(&teks_lower) {
        return true;
    }
    for word in teks_lower.split_whitespace() {
        if word.contains(&keyword_lower) || keyword_lower.contains(word) {
            return true;
        }
    }
    similarity(&teks_lower, &keyword_lower) >= 0.6
}

/// Satu fitur dari registry — cerminan objek plugin JS.
struct Fitur {
    name: String,
    alias: Vec<String>,
    category: String,
    description: String,
    usage: String,
    is_enabled: bool,
    is_premium: bool,
    is_owner: bool,
    cooldown: u32,
}

fn semua_fitur() -> Vec<Fitur> {
    all_plugins()
        .iter()
        .map(|p| Fitur {
            name: p.meta.name.clone(),
            alias: p.meta.alias.clone(),
            category: p.meta.category.clone(),
            description: p.meta.description.clone(),
            usage: p.meta.usage.clone(),
            is_enabled: p.meta.is_enabled,
            is_premium: p.meta.is_premium,
            is_owner: p.meta.is_owner,
            cooldown: p.meta.cooldown,
        })
        .collect()
}

/// Kartu pakai saat keyword kosong (`carifitur.js:110-120`).
pub(crate) fn kartu_pakai(prefix: &str) -> String {
    format!(
        "🔍 *ᴄᴀʀɪ ꜰɪᴛᴜʀ*\n\n\
         ╭┈┈⬡「 📋 *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ* 」\n\
         ┃ `{prefix}carifitur <keyword>`\n\
         ╰┈┈⬡\n\n\
         > Contoh:\n\
         `{prefix}carifitur sticker`\n\
         `{prefix}carifitur download`\n\
         `{prefix}carifitur game`"
    )
}

/// Pesan tanpa hasil (`carifitur.js:169-172`).
fn tanpa_hasil(keyword: &str) -> String {
    format!("🔍 *ʜᴀsɪʟ ᴘᴇɴᴄᴀʀɪᴀɴ*\n\n> Tidak ditemukan fitur dengan keyword `{keyword}`")
}

/// Potongan `slice(0, n)` JS dengan elipsis.
fn potong(teks: &str, n: usize) -> String {
    let jumlah = teks.chars().count();
    if jumlah > n {
        format!("{}...", teks.chars().take(n).collect::<String>())
    } else {
        teks.to_string()
    }
}

#[plugin(
    name = "carifitur",
    alias = ["searchcmd", "findcmd", "cari", "search"],
    category = "main",
    description = "Mencari fitur berdasarkan keyword dengan detail lengkap",
    usage = ".carifitur <keyword>",
    example = ".carifitur sticker",
    cooldown = 3,
    energi = 0,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false
)]
pub fn carifitur(ctx: Ctx) -> Reply {
    let keyword = ctx.text.trim().to_string();
    if keyword.is_empty() {
        return Reply::text(kartu_pakai(&ctx.prefix));
    }

    let fitur = semua_fitur();
    let matches: Vec<(f64, String, &Fitur)> = fitur
        .iter()
        .filter(|p| p.is_enabled)
        .filter_map(|p| {
            let mut score = 0.0f64;
            let mut reason = String::new();
            if cocok_keyword(&p.name, &keyword) {
                score = similarity(&p.name, &keyword) * 1.2;
                reason = "nama".into();
            }
            for alias in &p.alias {
                if cocok_keyword(alias, &keyword) {
                    score = score.max(similarity(alias, &keyword) * 1.1);
                    if reason.is_empty() {
                        reason = "alias".into();
                    }
                }
            }
            if cocok_keyword(&p.description, &keyword) {
                score = score.max(similarity(&p.description, &keyword) * 0.8);
                if reason.is_empty() {
                    reason = "deskripsi".into();
                }
            }
            if cocok_keyword(&p.category, &keyword) {
                score = score.max(similarity(&p.category, &keyword) * 0.7);
                if reason.is_empty() {
                    reason = "kategori".into();
                }
            }
            if reason.is_empty() {
                None
            } else {
                Some((score, reason, p))
            }
        })
        .collect();

    if matches.is_empty() {
        return Reply::text(tanpa_hasil(&keyword));
    }

    let mut sorted = matches;
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut text = format!("🔍 *ʜᴀsɪʟ ᴘᴇɴᴄᴀʀɪᴀɴ: \"{keyword}\"*\n");
    text.push_str(&format!("> Ditemukan *{}* fitur\n", sorted.len()));
    text.push_str("> Pilih salah satu command di bawah:\n\n");

    let top_matches = sorted.len().min(15);
    let detail = sorted.len().min(5);
    for (i, (_, _, p)) in sorted[..detail].iter().enumerate() {
        let mut badges = String::new();
        if p.is_premium {
            badges.push('💎');
        }
        if p.is_owner {
            badges.push('👑');
        }
        text.push_str(&format!(
            "*{}. {}{}* {}\n",
            i + 1,
            ctx.prefix,
            p.name,
            badges
        ));
        text.push_str(&format!("📁 Kategori: `{}`\n", p.category));
        text.push_str(&format!("📝 {}\n", potong(&p.description, 50)));
        if !p.usage.is_empty() {
            text.push_str(&format!("💡 Usage: `{}`\n", p.usage));
        }
        if p.cooldown > 0 {
            text.push_str(&format!("⏱️ Cooldown: {}s\n", p.cooldown));
        }
        text.push('\n');
    }
    if top_matches > 5 {
        text.push_str(&format!("_+{} hasil lainnya tersedia_", top_matches - 5));
    }

    // Kartu single_select: 10 hasil teratas (`carifitur.js:184-201`).
    let rows: Vec<CardRow> = sorted[..sorted.len().min(10)]
        .iter()
        .map(|(_, _, p)| {
            CardRow::new(
                format!("{}{}", ctx.prefix, p.name),
                format!("{}{}", ctx.prefix, p.name),
            )
            .description(format!("{} • {}", p.category, potong(&p.description, 40)))
        })
        .collect();
    let section = CardSection::new(format!("Hasil untuk \"{keyword}\"")).rows(rows);

    Reply::interactive(text)
        .with_header(CardHeader::new().image(aset_foto(&ctx)))
        .with_button_card(InteractiveButton::select(&CardList::new(
            "📋 Pilih Command",
            [section],
        )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "carifitur".into(),
            ..Ctx::default()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("carifitur").unwrap();
        assert_eq!(p.meta.name, "carifitur");
        assert_eq!(p.meta.alias, ["searchcmd", "findcmd", "cari", "search"]);
        assert_eq!(p.meta.category, "main");
        assert_eq!(
            p.meta.description,
            "Mencari fitur berdasarkan keyword dengan detail lengkap"
        );
        assert_eq!(p.meta.usage, ".carifitur <keyword>");
        assert_eq!(p.meta.example, ".carifitur sticker");
        assert_eq!(p.meta.cooldown, 3);
        assert!(!p.meta.is_owner && !p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn levenshtein_dan_similarity_mengikuti_js() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert!((similarity("abc", "abc") - 1.0).abs() < f64::EPSILON);
        assert_eq!(similarity("", ""), 1.0, "maxLen 0 → 1");
        // (7 - 3) / 7
        assert!((similarity("kitten", "sitting") - 4.0 / 7.0).abs() < 1e-9);
        assert!((similarity("ABC", "abc") - 1.0).abs() < 1e-9, "lowercase");
    }

    #[test]
    fn cocok_keyword_mengikuti_js() {
        assert!(cocok_keyword("sticker", "stic"));
        assert!(cocok_keyword("stic", "sticker"));
        assert!(cocok_keyword("cek khodam", "khodam"));
        assert!(!cocok_keyword("kitten", "sitting"), "similarity 0.4 < 0.6");
        assert!(!cocok_keyword("", "abc"));
    }

    #[test]
    fn keyword_kosong_kartu_pakai() {
        let t = carifitur(ctx_of("")).text_body().unwrap().to_string();
        assert!(t.starts_with("🔍 *ᴄᴀʀɪ ꜰɪᴛᴜʀ*"));
        assert!(t.contains("┃ `.carifitur <keyword>`"));
        assert!(t.contains("`.carifitur sticker`"));
    }

    #[test]
    fn cari_sticker_menemukan_plugin_sticker() {
        let r = carifitur(ctx_of("sticker"));
        match r.without_mutations() {
            Reply::Interactive { kartu } => {
                let body = &kartu.body;
                assert!(
                    body.starts_with("🔍 *ʜᴀsɪʟ ᴘᴇɴᴄᴀʀɪᴀɴ: \"sticker\"*"),
                    "{body}"
                );
                assert!(body.contains("> Ditemukan *"), "{body}");
                assert!(body.contains(".sticker*"), "{body}");
                assert!(!kartu.buttons.is_empty());
                assert_eq!(kartu.buttons[0].name, "single_select");
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }

    #[test]
    fn cari_kategori_game_penuh_kartu() {
        let r = carifitur(ctx_of("game"));
        match r.without_mutations() {
            Reply::Interactive { kartu } => {
                assert!(kartu.body.contains("> Ditemukan *"));
                assert!(!kartu.buttons.is_empty());
            }
            other => panic!("harus Interactive, dapat {other:?}"),
        }
    }

    #[test]
    fn tidak_ada_hasil_pesannya_tepat() {
        // `zzzzqqqq` justru cocok dengan plugin `q` (inclusion dua arah
        // `matchesKeyword`) — persis perilaku JS. Keyword simbol harusnya
        // tidak menemukan apa pun.
        let r = carifitur(ctx_of("!!!"));
        assert_eq!(r.text_body().unwrap(), tanpa_hasil("!!!"));
        // ...dan inclusion itu memang memproduksi hasil di JS juga.
        let q = carifitur(ctx_of("zzzzqqqq"));
        let body = match q.without_mutations() {
            Reply::Interactive { kartu } => kartu.body.clone(),
            other => panic!("harus Interactive, dapat {other:?}"),
        };
        assert!(body.contains(".q*"), "{body}");
    }
}
