//! Prune baris session SQLite lokal yang sudah kedaluwarsa.
//!
//! Hanya tabel ber-TTL yang disentuh. `sessions`, `identities`, `sender_keys`,
//! `prekeys`, dan `signed_prekeys` adalah state aktif Signal — menghapusnya
//! membuat pesan berikutnya gagal didekripsi, jadi tidak pernah masuk daftar.

use oktz_core::error::{Error, Result};

const HARI: i64 = 86_400;
const TC_TTL: i64 = 30 * HARI;
const SENT_TTL: i64 = 7 * HARI;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PruneReport {
    pub msg_secrets: u64,
    pub tc_tokens: u64,
    pub sent_messages: u64,
    pub pending_inbound: u64,
}

async fn hapus(
    conn: &libsql::Connection,
    sql: &str,
    param: i64,
    tabel: &str,
) -> Result<u64> {
    match conn.execute(sql, [param]).await {
        Ok(n) => Ok(n),
        Err(e) if e.to_string().contains("no such table") => Ok(0),
        Err(e) => Err(Error::Config(format!("session_db {tabel}: {e}"))),
    }
}

/// `Ok(None)` kalau `db_path` belum ada (bot belum pernah pairing).
pub async fn prune_expired(db_path: &str, now_secs: i64) -> Result<Option<PruneReport>> {
    if !std::path::Path::new(db_path).exists() {
        return Ok(None);
    }

    let db = libsql::Builder::new_local(db_path)
        .build()
        .await
        .map_err(|e| Error::Config(format!("session_db buka: {e}")))?;
    let conn = db
        .connect()
        .map_err(|e| Error::Config(format!("session_db koneksi: {e}")))?;
    conn.execute_batch("PRAGMA busy_timeout = 3000")
        .await
        .map_err(|e| Error::Config(format!("session_db busy_timeout: {e}")))?;

    let tc_cutoff = now_secs.saturating_sub(TC_TTL);
    let sent_cutoff = now_secs.saturating_sub(SENT_TTL);

    let msg_secrets = hapus(
        &conn,
        "DELETE FROM msg_secrets WHERE expires_at != 0 AND expires_at <= ?1",
        now_secs,
        "msg_secrets",
    )
    .await?;
    let tc_tokens = hapus(
        &conn,
        "DELETE FROM tc_tokens \
         WHERE (token = X'' OR token_timestamp < ?1) \
           AND (sender_timestamp IS NULL OR sender_timestamp < ?1)",
        tc_cutoff,
        "tc_tokens",
    )
    .await?;
    let sent_messages = hapus(
        &conn,
        "DELETE FROM sent_messages WHERE created_at < ?1",
        sent_cutoff,
        "sent_messages",
    )
    .await?;
    let pending_inbound = hapus(
        &conn,
        "DELETE FROM pending_inbound_messages WHERE inserted_at < ?1",
        sent_cutoff,
        "pending_inbound_messages",
    )
    .await?;

    Ok(Some(PruneReport {
        msg_secrets,
        tc_tokens,
        sent_messages,
        pending_inbound,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn db_baru() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "oktz-session-prune-{}-{}.db",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[tokio::test]
    async fn prune_hanya_baris_kedaluwarsa() {
        let path = db_baru();
        let db = libsql::Builder::new_local(&path).build().await.unwrap();
        let conn = db.connect().unwrap();
        for sql in [
            "CREATE TABLE msg_secrets (chat TEXT, sender TEXT, msg_id TEXT, secret BLOB, created_at INTEGER, expires_at INTEGER, message_ts INTEGER)",
            "CREATE TABLE tc_tokens (jid TEXT, token BLOB, token_timestamp INTEGER, sender_timestamp INTEGER, updated_at INTEGER)",
            "CREATE TABLE sent_messages (chat_jid TEXT, message_id TEXT, payload BLOB, created_at INTEGER)",
            "CREATE TABLE pending_inbound_messages (chat TEXT, sender TEXT, id TEXT, message BLOB, inserted_at INTEGER)",
        ] {
            conn.execute(sql, ()).await.unwrap();
        }
        conn.execute(
            "INSERT INTO msg_secrets VALUES ('c','s','1',x'00',0,100,0), ('c','s','2',x'00',0,0,0)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO tc_tokens VALUES ('a',x'00',100,NULL,0), ('b',x'00',1700000000,NULL,0)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO sent_messages VALUES ('c','1',x'00',100), ('c','2',x'00',1700000000)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO pending_inbound_messages VALUES ('c','s','1',x'00',100), ('c','s','2',x'00',1700000000)",
            (),
        )
        .await
        .unwrap();
        drop(conn);
        drop(db);

        let now = 1_700_000_000i64;
        let r = prune_expired(path.to_str().unwrap(), now)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            r,
            PruneReport {
                msg_secrets: 1,
                tc_tokens: 1,
                sent_messages: 1,
                pending_inbound: 1,
            }
        );
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }

    #[tokio::test]
    async fn prune_file_tidak_ada_mengembalikan_none() {
        let r = prune_expired("/tmp/oktz-tidak-ada-session.db", 1)
            .await
            .unwrap();
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn db_kosong_tanpa_tabel_dianggap_nol() {
        let path = db_baru();
        std::fs::write(&path, b"").unwrap();
        let r = prune_expired(path.to_str().unwrap(), 1_700_000_000)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(r, PruneReport::default());
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }
}
