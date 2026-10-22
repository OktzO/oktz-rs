//! `plugins/media/music.js` — kirim lagu dari koleksi `Rez4-3yz/Music-rd`.
//!
//! Perilaku yang ikut diport apa adanya:
//! - Guard angka: `m.command.replace('music','')` lalu `parseInt`; di luar
//!   1..65 plugin membalas kartu "Tersedia" (`music.js:23-28`).
//! - URL audio `https://raw.githubusercontent.com/Rez4-3yz/Music-rd/master/music/music<N>.mp3`
//!   (`music.js:32`) dikirim sebagai audio (bukan voice note).
//!
//! `ponytail:` `sock.sendMedia(..., { type: 'audio' })` di JS menerima URL dan
//! mengunduhnya sendiri. Di Rust `Reply::audio(MediaSource::url(..))` juga
//! menampilkan URL, jadi bentuknya sama — bedanya hanya file_name: JS tidak
//! menentukan nama berkas untuk audio ini, dan `Reply::Audio` juga tidak punya
//! kolom itu.

use super::ALIAS;
use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};


/// `music.js:27` — kartu saat nomor lagu di luar 1..65.
pub(crate) const DAFTAR: &str =
    "🎵 *ᴍᴜsɪᴄ ᴄᴏʟʟᴇᴄᴛɪᴏɴ*\n\n> Tersedia: .music1 - .music65";

/// `music.js:44` — gagal mengambil lagu.
pub(crate) const GAGAL: &str = "❌ *ᴇʀʀᴏʀ*\n\n> Musik tidak ditemukan atau gagal diambil.";

/// `music.js:32` — URL lagu ke-`n`.
pub(crate) fn url_lagu(nomor: i64) -> String {
    format!("https://raw.githubusercontent.com/Rez4-3yz/Music-rd/master/music/music{nomor}.mp3")
}

/// `music.js:23-26` — `m.command.replace('music','')` lalu `parseInt`.
///
/// `replace` JS hanya mengganti kemunculan PERTAMA, jadi command
/// `music1music2` jadi `1music2` — `parseInt` membaca `1` saja. Nomor di luar
/// 1..65 ditolak.
pub(crate) fn nomor_lagu(command: &str) -> Option<i64> {
    let sisa = match command.find("music") {
        Some(i) => &command[i + "music".len()..],
        None => command,
    };
    let angka = crate::common::parse_int(sisa)?;
    if (1..=65).contains(&angka) {
        Some(angka)
    } else {
        None
    }
}

#[plugin(
    name = "music",
    alias = [
        "music1", "music2", "music3", "music4", "music5", "music6", "music7", "music8",
        "music9", "music10", "music11", "music12", "music13", "music14", "music15", "music16",
        "music17", "music18", "music19", "music20", "music21", "music22", "music23", "music24",
        "music25", "music26", "music27", "music28", "music29", "music30", "music31", "music32",
        "music33", "music34", "music35", "music36", "music37", "music38", "music39", "music40",
        "music41", "music42", "music43", "music44", "music45", "music46", "music47", "music48",
        "music49", "music50", "music51", "music52",
    ],
    category = "media",
    description = "Koleksi musik 1-65",
    usage = ".music1 sampai .music65",
    example = ".music1",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn music(ctx: Ctx) -> Reply {
    let Some(nomor) = nomor_lagu(&ctx.command) else {
        return Reply::text(DAFTAR);
    };

    // `m.react('🕕')` (`music.js:30`) lalu media lalu `m.react('✅')`
    // (`music.js:40`) — `Reply::Reaction` hanya membawa satu reaksi, jadi yang
    // ikut adalah reaksi penutup (konvensi repo).
    Reply::audio(MediaSource::url(url_lagu(nomor))).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_of(command: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: command.into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    #[test]
    fn output_sama_dengan_js() {
        // `music.js:27` — kartu "Tersedia" persis seperti JS.
        assert_eq!(
            DAFTAR,
            "🎵 *ᴍᴜsɪᴄ ᴄᴏʟʟᴇᴄᴛɪᴏɴ*\n\n> Tersedia: .music1 - .music65"
        );
        // `music.js:44` — pesan gagal persis seperti JS.
        assert_eq!(
            GAGAL,
            "❌ *ᴇʀʀᴏʀ*\n\n> Musik tidak ditemukan atau gagal diambil."
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("music").unwrap();
        assert_eq!(p.meta.name, "music");
        assert_eq!(p.meta.category, "media");
        assert_eq!(p.meta.description, "Koleksi musik 1-65");
        assert_eq!(p.meta.usage, ".music1 sampai .music65");
        assert_eq!(p.meta.example, ".music1");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn alias_mencakup_music1_sampai_music52() {
        assert_eq!(ALIAS.len(), 52);
        assert_eq!(ALIAS[0], "music1");
        assert_eq!(ALIAS[51], "music52");
        for a in ALIAS {
            assert_eq!(
                oktz_plugin_api::find(a).unwrap().meta.name,
                "music",
                "alias {a}"
            );
        }
    }

    #[test]
    fn nomor_diambil_dari_nama_command() {
        assert_eq!(nomor_lagu("music1"), Some(1));
        assert_eq!(nomor_lagu("music52"), Some(52));
        assert_eq!(nomor_lagu("music65"), Some(65));
        // Di luar rentang → kartu "Tersedia".
        assert_eq!(nomor_lagu("music0"), None);
        assert_eq!(nomor_lagu("music66"), None);
        assert_eq!(nomor_lagu("music"), None);
        assert_eq!(nomor_lagu("musik1"), None);
    }

    #[test]
    fn nomor_di_luar_rentang_menampilkan_kartu_tersedia() {
        let r = music(ctx_of("music0"));
        assert_eq!(teks(&r), DAFTAR);
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn nomor_valid_mengirim_audio_dengan_url_lagu() {
        let r = music(ctx_of("music7"));
        assert_eq!(reaksi(&r), Some("✅"));
        let Reply::Reaction { reply, .. } = r.without_mutations() else {
            panic!("harus dibungkus reaksi");
        };
        match reply.as_ref() {
            Reply::Audio { media, ptt } => {
                assert!(!ptt, "bukan voice note");
                assert_eq!(
                    media,
                    &Some(MediaSource::Url {
                        url: "https://raw.githubusercontent.com/Rez4-3yz/Music-rd/master/music/music7.mp3".into(),
                        mimetype: None,
                    })
                );
            }
            lain => panic!("harus audio, dapat {lain:?}"),
        }
    }

    #[test]
    fn url_lagu_mengikuti_pola_js() {
        assert_eq!(
            url_lagu(1),
            "https://raw.githubusercontent.com/Rez4-3yz/Music-rd/master/music/music1.mp3"
        );
    }
}
