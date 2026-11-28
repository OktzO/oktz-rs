//! Akses baca ke snapshot store yang sudah di-parse pipeline.

use serde_json::Value;

/// Snapshot store yang dipakai gate permission, dibagikan ke plugin tanpa
/// menyalin pohon JSON.
///
/// `settings.json` bisa belasan kali lebih besar setelah di-parse; plugin yang
/// mem-parse ulang file itu untuk setiap command (leaderboard, daftar user,
/// profil user lain) menggandakan puncak memori. Implementasi produksi
/// membungkus `Arc` snapshot yang sama dengan pipeline.
pub trait StoreView: Send + Sync + std::fmt::Debug {
    /// `db.getUser(key)` untuk kunci store apa adanya.
    fn user(&self, key: &str) -> Option<Value>;

    /// `Object.keys(db.data.users).length`.
    fn user_count(&self) -> usize;

    /// Iterasi seluruh user tanpa menyalin nilainya.
    fn for_each_user(&self, f: &mut dyn FnMut(&str, &Value));

    /// `Object.keys(db.data.groups).length`.
    fn group_count(&self) -> usize {
        0
    }

    /// Iterasi seluruh grup tanpa menyalin nilainya.
    fn for_each_group(&self, _f: &mut dyn FnMut(&str, &Value)) {}

    /// Cari user lewat nomor; kunci store menyimpan nomor tanpa server.
    fn user_by_phone(&self, phone: &str) -> Option<Value> {
        let mut found = None;
        self.for_each_user(&mut |key, value| {
            if found.is_none() && key.split('@').next().unwrap_or(key) == phone {
                found = Some(value.clone());
            }
        });
        found
    }
}
