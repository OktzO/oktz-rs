use oktz_core::error::{Error, Result};

/// Koneksi Turso/LibSQL.
///
/// Produksi: `Builder::new_remote(url, token)` — pure HTTP (Hrana), tanpa
/// proses C. Test: `with_conn` supaya `:memory:` bisa dipakai tanpa jaringan.
///
/// Skema mengikuti bot JS (`oktz-md/src/lib/turso.js`) supaya database Turso
/// yang sama bisa dibaca kedua bot:
/// - `stores(key TEXT PRIMARY KEY, data TEXT, updated_at INTEGER)` — blob JSON
///   per store (`users`, `groups`, `settings`, ...).
/// - `session_creds(scope TEXT PRIMARY KEY, creds TEXT, updated_at INTEGER)`.
/// - `session_keys(scope, category, id, data, updated_at, PK(scope,category,id))`.
use libsql::Value;

pub struct TursoStore {
    pub url: String,
    pub auth_token: String,
    /// Koneksi yang sudah jadi (test `:memory:`); produksi `None` —
    /// `connect()` membangun dari `url` + `auth_token`.
    conn: Option<libsql::Connection>,
}

impl TursoStore {
    pub fn from_env() -> Result<Self> {
        let url = oktz_core::env::Env::require("TURSO_URL")?;
        let auth_token = oktz_core::env::Env::require("TURSO_AUTH_TOKEN")?;
        Ok(Self {
            url,
            auth_token,
            conn: None,
        })
    }

    /// Test: pakai koneksi yang sudah dibangun (`Builder::new_local(":memory:")`).
    pub fn with_conn(conn: libsql::Connection) -> Self {
        Self {
            url: String::new(),
            auth_token: String::new(),
            conn: Some(conn),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.url.is_empty() && !self.auth_token.is_empty()
    }

    pub fn validate(&self) -> Result<()> {
        if self.conn.is_some() {
            return Ok(());
        }
        if self.url.is_empty() {
            return Err(Error::Config("TURSO_URL kosong".into()));
        }
        if self.url.starts_with("libsql://") || self.url.starts_with("https://") {
            Ok(())
        } else {
            Err(Error::Config(format!(
                "TURSO_URL skema tak dikenal: {}",
                self.url
            )))
        }
    }

    /// Koneksi baru. Produksi: remote. Test: koneksi yang disuntik.
    pub async fn connect(&self) -> Result<libsql::Connection> {
        match &self.conn {
            Some(conn) => Ok(conn.clone()),
            None => {
                let db = libsql::Builder::new_remote(self.url.clone(), self.auth_token.clone())
                    .build()
                    .await
                    .map_err(|e| Error::Config(format!("koneksi Turso gagal: {e}")))?;
                db.connect()
                    .map_err(|e| Error::Config(format!("buka koneksi Turso gagal: {e}")))
            }
        }
    }

    /// `initTursoTables` (`oktz-md/src/lib/turso.js:20-37`) — idempoten.
    pub async fn ensure_tables(&self, conn: &libsql::Connection) -> Result<()> {
        for sql in [
            "CREATE TABLE IF NOT EXISTS stores (key TEXT PRIMARY KEY, data TEXT, updated_at INTEGER)",
            "CREATE TABLE IF NOT EXISTS session_creds (scope TEXT PRIMARY KEY, creds TEXT, updated_at INTEGER)",
            "CREATE TABLE IF NOT EXISTS session_keys (scope TEXT, category TEXT, id TEXT, data TEXT, updated_at INTEGER, PRIMARY KEY (scope, category, id))",
        ] {
            conn.execute(sql, ())
                .await
                .map_err(turso_err("ensure_tables"))?;
        }
        Ok(())
    }

    /// Baca satu row `stores`; `None` kalau key belum ada.
    pub async fn load_store(&self, conn: &libsql::Connection, key: &str) -> Result<Option<serde_json::Value>> {
        let mut rows = conn
            .query("SELECT data FROM stores WHERE key = ?1", [key])
            .await
            .map_err(turso_err("load_store"))?;
        let Some(row) = rows.next().await.map_err(turso_err("load_store next"))? else {
            return Ok(None);
        };
        let data: String = row
            .get(0)
            .map_err(turso_err("load_store get"))?;
        serde_json::from_str(&data)
            .map(Some)
            .map_err(Error::from)
    }

    /// UPSERT satu row `stores` — `INSERT ... ON CONFLICT DO UPDATE`, sama
    /// dengan `writeToTurso` (`oktz-md/src/lib/database.js:401-411`).
    pub async fn put_store(&self, conn: &libsql::Connection, key: &str, v: &serde_json::Value) -> Result<()> {
        let data = crate::lowdb::stringify(v)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        conn.execute(
            "INSERT INTO stores (key, data, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            [key, data.as_str(), &now.to_string()],
        )
        .await
        .map_err(turso_err("put_store"))?;
        Ok(())
    }

    /// `flushAllToTurso` seed guard (`oktz-md/src/lib/database.js:164-166`):
    /// apakah tabel `stores` sudah berisi sesuatu.
    pub async fn has_any_store(&self, conn: &libsql::Connection) -> Result<bool> {
        let mut rows = conn
            .query("SELECT 1 FROM stores LIMIT 1", ())
            .await
            .map_err(turso_err("has_any_store"))?;
        Ok(rows.next().await.map_err(turso_err("has_any_store next"))?.is_some())
    }

    /// `sessionKeyStats` (`plugins/owner/sessionstats.js`) — per kategori.
    pub async fn session_key_stats(&self, conn: &libsql::Connection, scope: &str) -> Result<Vec<(String, i64, i64)>> {
        let mut rows = conn
            .query(
                "SELECT category, COUNT(*), SUM(LENGTH(data)) FROM session_keys WHERE scope = ?1 GROUP BY category",
                [scope],
            )
            .await
            .map_err(turso_err("session_key_stats"))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await.map_err(turso_err("session_key_stats next"))? {
            out.push((
                row.get::<String>(0).map_err(turso_err("stats cat"))?,
                row.get::<i64>(1).map_err(turso_err("stats cnt"))?,
                row.get::<i64>(2).map_err(turso_err("stats bytes"))?,
            ));
        }
        Ok(out)
    }

    /// Mirror tabel SQLite lokal → Turso, JSON-per-row (lihat plan Task 3).
    ///
    /// Skema Turso generik: `CREATE TABLE IF NOT EXISTS <t> (k INTEGER PRIMARY KEY AUTOINCREMENT, v TEXT)`
    /// — `v` = seluruh row sebagai JSON (BLOB → hex). Sederhana & selaras;
    /// upgrade ke kolom asli kalau ukuran jadi masalah.
    pub async fn mirror_sqlite_table(
        &self,
        conn: &libsql::Connection,
        local_path: &std::path::Path,
        table: &str,
    ) -> Result<usize> {
        let local = libsql::Builder::new_local(local_path)
            .build()
            .await
            .map_err(turso_err("buka sqlite lokal"))?;
        let lconn = local
            .connect()
            .map_err(turso_err("koneksi sqlite lokal"))?;
        let mut rows = lconn
            .query(&format!("SELECT * FROM {table}"), ())
            .await
            .map_err(turso_err(&format!("baca {table}")))?;
        let columns: Vec<String> = (0..rows.column_count())
            .filter_map(|i| rows.column_name(i).map(str::to_string))
            .collect();
        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {table} (k INTEGER PRIMARY KEY AUTOINCREMENT, v TEXT)"
            ),
            (),
        )
        .await
        .map_err(turso_err(&format!("buat tabel {table}")))?;
        let mut n = 0usize;
        while let Some(row) = rows.next().await.map_err(turso_err(&format!("iter {table}")))? {
            let mut obj = serde_json::Map::new();
            for (i, col) in columns.iter().enumerate() {
                let v: Value = row
                    .get(i as i32)
                    .map_err(turso_err(&format!("baca {col}")))?;
                obj.insert(col.clone(), libsql_to_json(&v));
            }
            let v = serde_json::Value::Object(obj);
            let data = serde_json::to_string(&v).map_err(Error::from)?;
            conn.execute(
                &format!(
                    "INSERT INTO {table} (k, v) VALUES (?1, ?2)
                     ON CONFLICT(k) DO UPDATE SET v = excluded.v"
                ),
                [n.to_string().as_str(), data.as_str()],
            )
            .await
            .map_err(turso_err(&format!("upsert {table}")))?;
            n += 1;
        }
        Ok(n)
    }
}

fn turso_err(konteks: impl Into<String>) -> impl FnOnce(libsql::Error) -> Error {
    move |e| Error::Config(format!("turso {}: {e}", konteks.into()))
}

/// `libsql::Value` → `serde_json::Value`. BLOB disalin base64 supaya aman di
/// JSON; integer i64 yang kecil tetap angka.
fn libsql_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Integer(i) => serde_json::Value::from(*i),
        Value::Real(f) => serde_json::Value::from(*f),
        Value::Text(s) => serde_json::Value::from(s.clone()),
        Value::Blob(b) => {
            use std::fmt::Write as _;
            let mut s = String::with_capacity(b.len() * 2);
            for byte in b.iter() {
                let _ = write!(s, "{byte:02x}");
            }
            serde_json::Value::from(s)
        }
    }
}

impl Clone for TursoStore {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            auth_token: self.auth_token.clone(),
            conn: self.conn.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn mem() -> (TursoStore, libsql::Connection) {
        let db = libsql::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        let store = TursoStore::with_conn(conn.clone());
        store.ensure_tables(&conn).await.unwrap();
        (store, conn)
    }

    #[tokio::test]
    async fn validate_rejects_empty_url() {
        let s = TursoStore {
            url: String::new(),
            auth_token: "t".into(),
            conn: None,
        };
        assert!(s.validate().is_err());
        assert!(!s.is_configured());
    }

    #[tokio::test]
    async fn validate_rejects_unknown_scheme() {
        let s = TursoStore {
            url: "postgres://x".into(),
            auth_token: "t".into(),
            conn: None,
        };
        assert!(s.validate().is_err());
    }

    #[tokio::test]
    async fn validate_accepts_libsql() {
        let s = TursoStore {
            url: "libsql://db.example.turso.io".into(),
            auth_token: "t".into(),
            conn: None,
        };
        assert!(s.validate().is_ok());
        assert!(s.is_configured());
    }

    #[tokio::test]
    async fn put_then_load_store_roundtrips() {
        let (store, conn) = mem().await;
        let v = serde_json::json!({ "628111": { "koin": 5, "exp": 10 } });
        store.put_store(&conn, "users", &v).await.unwrap();
        assert_eq!(store.load_store(&conn, "users").await.unwrap(), Some(v));
    }

    #[tokio::test]
    async fn load_missing_store_is_none() {
        let (store, conn) = mem().await;
        assert_eq!(store.load_store(&conn, "ngawur").await.unwrap(), None);
        assert!(!store.has_any_store(&conn).await.unwrap());
    }

    #[tokio::test]
    async fn has_any_store_after_put() {
        let (store, conn) = mem().await;
        store
            .put_store(&conn, "groups", &serde_json::json!({}))
            .await
            .unwrap();
        assert!(store.has_any_store(&conn).await.unwrap());
    }

    #[tokio::test]
    async fn mirror_sqlite_table_copies_rows_as_json() {
        let (store, conn) = mem().await;
        let dir = std::env::temp_dir().join(format!("oktz-turso-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("lokal.db");
        let _ = std::fs::remove_file(&path);
        let local = libsql::Builder::new_local(&path).build().await.unwrap();
        let l = local.connect().unwrap();
        l.execute("CREATE TABLE identities (a TEXT, b BLOB)", ())
            .await
            .unwrap();
        l.execute("INSERT INTO identities VALUES ('satu', x'0102')", ())
            .await
            .unwrap();
        l.execute("INSERT INTO identities VALUES ('dua', x'0304')", ())
            .await
            .unwrap();
        let n = store.mirror_sqlite_table(&conn, &path, "identities").await.unwrap();
        assert_eq!(n, 2);
        let mut rows = conn
            .query("SELECT v FROM identities ORDER BY k", ())
            .await
            .unwrap();
        let r1: String = rows.next().await.unwrap().unwrap().get(0).unwrap();
        let v1: serde_json::Value = serde_json::from_str(&r1).unwrap();
        assert_eq!(v1["a"], "satu");
        assert_eq!(v1["b"], "0102", "BLOB harus base64/hex di JSON");
    }

    #[tokio::test]
    async fn session_key_stats_empty() {
        let (store, conn) = mem().await;
        let s = store.session_key_stats(&conn, "main").await.unwrap();
        assert!(s.is_empty());
    }
}
