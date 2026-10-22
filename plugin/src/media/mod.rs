//! `plugins/media/music.js` — koleksi musik 1-65.
//!
//! Alias `music1`..`music52` diambil apa adanya dari `MUSIC_LIST`
//! (`music.js:1-4`): JS membangunnya dengan loop `for (let i = 1; i <= 52; i++)`,
//! jadi urutannya naik dan tidak ada `music53`..`music65` — padahal
//! `handler` menerima angka sampai 65 (`music.js:26`). Perbedaan itu ikut
//! ditiru: `.music60` tidak pernah sampai ke plugin karena tidak ada aliasnya.

pub mod music;

/// Semua alias `music1`..`music52` (`music.js:1-4`).
pub(crate) const ALIAS: [&str; 52] = [
    "music1", "music2", "music3", "music4", "music5", "music6", "music7", "music8",
    "music9", "music10", "music11", "music12", "music13", "music14", "music15", "music16",
    "music17", "music18", "music19", "music20", "music21", "music22", "music23", "music24",
    "music25", "music26", "music27", "music28", "music29", "music30", "music31", "music32",
    "music33", "music34", "music35", "music36", "music37", "music38", "music39", "music40",
    "music41", "music42", "music43", "music44", "music45", "music46", "music47", "music48",
    "music49", "music50", "music51", "music52",
];
