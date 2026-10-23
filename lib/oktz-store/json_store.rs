use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use oktz_core::error::{Error, Result};
use serde_json::{Map, Value};

/// Nomor urut nama file temp — dipakai bersama PID supaya dua writer di
/// proses yang sama tidak memakai nama temp yang sama.
static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

pub struct JsonStore {
    path: PathBuf,
    data: Value,
}

/// Isi file top-level bukan object — owner.json/partner.json/premium.json
/// top-level-nya array. Set key pada file seperti itu berarti menghapus seluruh
/// isinya (mis. seluruh roster premium), jadi tolak eksplisit, jangan diam-diam.
const NOT_OBJECT: &str =
    "file JSON top-level bukan object (array/scalar) — set() akan menghapus seluruh isi";

impl JsonStore {
    /// Buka file JSON; kalau tidak ada, mulai dari objek kosong.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            let data: Value = serde_json::from_str(&raw).map_err(Error::from)?;
            Ok(Self { path, data })
        } else {
            Ok(Self {
                path,
                data: Value::Object(Map::new()),
            })
        }
    }

    fn map_mut(&mut self) -> Result<&mut Map<String, Value>> {
        self.data
            .as_object_mut()
            .ok_or_else(|| Error::Config(NOT_OBJECT.into()))
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.as_object().and_then(|m| m.get(key))
    }

    pub fn set(&mut self, key: &str, value: Value) -> Result<()> {
        self.map_mut()?.insert(key.to_string(), value);
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.as_object_mut().and_then(|m| m.remove(key))
    }

    pub fn all(&self) -> &Value {
        &self.data
    }

    /// Ganti SELURUH isi file.
    ///
    /// `set()` hanya bisa mengubah satu key di level atas. Layout multi-file
    /// bot JS menyimpan satu KOLEKSI per file (`users.json` = objek users,
    /// `groups.json` = objek groups — `database.js:127-136`), jadi menulis
    /// koleksi itu butuh mengganti isi file, bukan menambah key.
    pub fn replace(&mut self, value: Value) -> Result<()> {
        self.data = value;
        Ok(())
    }

    /// Tulis atomik: temp lalu rename (sama seperti `database.js:49`).
    ///
    /// Nama temp WAJIB unik per writer. Dua proses yang menulis file yang
    /// sama (mis. dua `oktz boot` paralel, atau bot + `oktz boot`) akan
    /// saling menimpa temp yang sama: rename pertama memindahkan file,
    /// rename kedua gagal `NotFound` dan proses mati.
    pub fn write(&self) -> Result<()> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let tmp = self
            .path
            .with_extension(format!("{}-{seq}.tmp", std::process::id()));
        let body = serde_json::to_string(&self.data).map_err(Error::from)?;
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpfile(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("oktz-store-test-{name}.json"));
        let _ = std::fs::remove_file(&p);
        p
    }

    // Regresi: premium.json/owner.json/partner.json top-level-nya array.
    // Dulu map_mut() mengganti isinya jadi object kosong -> roster hilang.
    #[test]
    fn set_on_array_file_is_rejected_not_destructive() {
        let path = tmpfile("array-file");
        let original = r#"[{"jid":"6285608953677","isPremium":true}]"#;
        std::fs::write(&path, original).unwrap();

        let mut s = JsonStore::open(&path).unwrap();
        assert!(s.set("jid", Value::from("x")).is_err(), "harus ditolak");
        s.write().unwrap();

        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(after, original, "isi file array TIDAK boleh berubah");
    }

    #[test]
    fn write_on_unopened_array_file_preserves_content() {
        let path = tmpfile("array-preserve");
        let original = r#"[1,2,3]"#;
        std::fs::write(&path, original).unwrap();
        let s = JsonStore::open(&path).unwrap();
        s.write().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn missing_file_starts_empty() {
        let s = JsonStore::open(tmpfile("missing")).unwrap();
        assert_eq!(s.all(), &Value::Object(Map::new()));
    }

    #[test]
    fn set_get_remove() {
        let mut s = JsonStore::open(tmpfile("crud")).unwrap();
        s.set("a", Value::from(1)).unwrap();
        assert_eq!(s.get("a"), Some(&Value::from(1)));
        s.remove("a");
        assert_eq!(s.get("a"), None);
    }

    // Review Focus #1: field yang tidak dikenal harus bertahan utuh.
    #[test]
    fn unknown_fields_survive_read_modify_write() {
        let path = tmpfile("unknown-fields");
        std::fs::write(
            &path,
            r#"{"known":1,"fieldBaru":{"nested":[1,2,{"x":"y"}]},"tail":"keep"}"#,
        )
        .unwrap();

        let mut s = JsonStore::open(&path).unwrap();
        s.set("known", Value::from(99)).unwrap();
        s.write().unwrap();

        let after = std::fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&after).unwrap();
        assert_eq!(v["fieldBaru"]["nested"][2]["x"], "y", "field asing hilang");
        assert_eq!(v["tail"], "keep");
        assert_eq!(v["known"], 99);
    }

    // Review Focus #3: angka tidak jadi float.
    #[test]
    fn integers_stay_integers_on_disk() {
        let path = tmpfile("ints");
        let mut s = JsonStore::open(&path).unwrap();
        s.set("count", Value::from(42)).unwrap();
        s.write().unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(after, r#"{"count":42}"#);
        assert!(!after.contains("42.0"));
    }

    #[test]
    fn write_is_byte_identical_to_lowdb_format() {
        let path = tmpfile("bytefmt");
        let mut s = JsonStore::open(&path).unwrap();
        s.set("z", Value::from(1)).unwrap();
        s.set("a", Value::from(2)).unwrap();
        s.write().unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            after, r#"{"z":1,"a":2}"#,
            "urutan + format harus sama lowdb"
        );
    }

    #[test]
    fn tmp_file_is_cleaned_after_write() {
        let path = tmpfile("tmpclean");
        let s = JsonStore::open(&path).unwrap();
        s.write().unwrap();
        // Nama temp unik per writer (PID + urutan), jadi sisa file dicek lewat
        // pola, bukan nama tetap.
        let leftovers: Vec<String> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with("oktz-store-test-tmpclean") && n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "file tmp tidak boleh tertinggal: {leftovers:?}"
        );
    }

    #[test]
    fn parallel_writes_to_one_file_do_not_clash_on_the_tmp_name() {
        // Regresi: nama temp dulu tetap (`settings.json.tmp`) → dua writer
        // menimpa temp yang sama, rename pertama memindahkan file, rename
        // kedua gagal `NotFound` dan proses mati. `oktz boot` dua kali
        // paralel di folder data yang sama memunculkan ini.
        let path = tmpfile("paralel");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut handles = Vec::new();
        for i in 0..8 {
            let path = path.clone();
            handles.push(std::thread::spawn(move || {
                for n in 0..20 {
                    let mut s = JsonStore::open(&path).unwrap();
                    s.set("writer", Value::from(i * 100 + n)).unwrap();
                    s.write().unwrap();
                }
            }));
        }
        for h in handles {
            h.join().expect("writer tidak boleh gagal");
        }
        // Isi akhir harus salah satu nilai writer — bukan file rusak.
        let raw = std::fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert!(v.get("writer").is_some(), "file akhir harus valid: {raw}");
    }
}
