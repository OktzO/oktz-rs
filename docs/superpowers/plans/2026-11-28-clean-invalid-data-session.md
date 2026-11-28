# Clean Invalid Data + Session Prune Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an owner plugin that deletes invalid `users`/`groups` entries and extend `clearsessiondb` to prune expired local SQLite rows and stale Turso session data.

**Architecture:** Deletions go through two new `Mutation` variants applied by the existing pipeline store, so disk and Turso stay in sync. Local session pruning lives in `oktz-store` and uses raw SQL through the already-installed `libsql` local backend. Live Signal state (prekeys, sessions, identities, sender keys) is never deleted.

**Tech Stack:** Rust, serde_json, libsql, tokio, existing plugin/store patterns.

**Spec:** `docs/superpowers/specs/2026-11-28-clean-invalid-data-session-design.md`

## Global Constraints

- No new dependencies.
- No unnecessary comments.
- Keep changes focused; follow existing plugin/store patterns.
- User/grup valid tidak boleh dihapus.
- Prekey/session/identity/sender_key aktif tidak boleh dihapus.
- Verification: `cargo fmt --all`, `cargo clippy`, `cargo check`; full tests in GitHub Actions.

## Review Focus

- Key user yang mengandung `@` atau `:` harus dihapus persis, bukan dinormalisasi lalu menghapus key lain.
- Grup dengan key valid tetapi value bukan object harus ikut terhapus.
- SQLite yang sedang dipakai bot (WAL) tidak boleh menggagalkan command karena `SQLITE_BUSY`.
- `clearsessiondb` tidak boleh menghapus `session_state` scope `main`.
- `msg_secrets` dengan `expires_at = 0` berarti tidak pernah kedaluwarsa dan tidak boleh dihapus.

---

### Task 1: Store view grup + mutasi hapus

**Files:**
- Modify: `lib/oktz-plugin-api/store.rs`
- Modify: `lib/oktz-plugin-api/registry.rs:114`
- Modify: `lib/oktz-wa/pipeline.rs:508` (`apply_mutation`)
- Modify: `lib/oktz-wa/pipeline.rs:1039` (`impl StoreView for StoreSnapshot`)
- Test: `lib/oktz-wa/pipeline.rs` test module

**Interfaces:**
- Produces:
  - `StoreView::group_count(&self) -> usize`
  - `StoreView::for_each_group(&self, f: &mut dyn FnMut(&str, &Value))`
  - `Mutation::DeleteUser { key: String }`
  - `Mutation::DeleteGroup { key: String }`

- [ ] **Step 1: Write failing tests in `lib/oktz-wa/pipeline.rs` tests module**

```rust
#[test]
fn delete_user_menghapus_key_persis() {
    let path = std::env::temp_dir().join(format!(
        "oktz-deluser-{}.json",
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::write(
        &path,
        r#"{"users":{"unknown":{"koin":1},"628111":{"koin":2}}}"#,
    )
    .unwrap();
    let mut store = JsonStore::open(&path).unwrap();
    apply_mutation(
        &mut store,
        &Mutation::DeleteUser {
            key: "unknown".into(),
        },
        0,
        path.clone(),
    )
    .unwrap();
    let users = store.get("users").unwrap();
    assert!(users.get("unknown").is_none(), "key invalid harus hilang");
    assert_eq!(users["628111"]["koin"], 2, "user valid utuh");
    std::fs::remove_file(&path).ok();
}

#[test]
fn delete_group_menghapus_key_persis() {
    let path = std::env::temp_dir().join(format!(
        "oktz-delgroup-{}.json",
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::write(
        &path,
        r#"{"groups":{"bukan-grup":{"rpg":true},"120363@g.us":{"rpg":true}}}"#,
    )
    .unwrap();
    let mut store = JsonStore::open(&path).unwrap();
    apply_mutation(
        &mut store,
        &Mutation::DeleteGroup {
            key: "bukan-grup".into(),
        },
        0,
        path.clone(),
    )
    .unwrap();
    let groups = store.get("groups").unwrap();
    assert!(groups.get("bukan-grup").is_none());
    assert_eq!(groups["120363@g.us"]["rpg"], true);
    std::fs::remove_file(&path).ok();
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p oktz-wa --lib delete_ --no-run`
Expected: compile error because the variants do not exist.

- [ ] **Step 3: Implement**

In `lib/oktz-plugin-api/registry.rs`, add to `Mutation`:

```rust
/// Hapus satu key `users` apa adanya (tanpa normalisasi).
DeleteUser { key: String },
/// Hapus satu key `groups` apa adanya.
DeleteGroup { key: String },
```

In `lib/oktz-wa/pipeline.rs::apply_mutation`, add:

```rust
Mutation::DeleteUser { key } => {
    if let Some(users) = store.get_mut("users").ok().and_then(Value::as_object_mut) {
        users.remove(key);
    }
}
Mutation::DeleteGroup { key } => {
    if let Some(groups) = store.get_mut("groups").ok().and_then(Value::as_object_mut) {
        groups.remove(key);
    }
}
```

Use `JsonStore::get_mut` only if the collection exists; a missing collection is a no-op, not an error.

In `lib/oktz-plugin-api/store.rs`, add default methods:

```rust
fn group_count(&self) -> usize {
    0
}

fn for_each_group(&self, _f: &mut dyn FnMut(&str, &Value)) {}
```

In `impl oktz_plugin_api::StoreView for StoreSnapshot`, implement both by reading `"groups"` the same way `user_count`/`for_each_user` read `"users"`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p oktz-wa --lib delete_`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add lib/oktz-plugin-api/store.rs lib/oktz-plugin-api/registry.rs lib/oktz-wa/pipeline.rs
git commit -m "feat(store): mutasi hapus user/grup dan akses grup di StoreView"
```

---

### Task 2: Plugin `cleanuser`

**Files:**
- Create: `plugin/src/owner/cleanuser.rs`
- Modify: `plugin/src/owner/mod.rs` (dekat `clearsessiondb`)
- Test: `plugin/src/owner/cleanuser.rs` test module

**Interfaces:**
- Consumes: `Ctx::store() -> Option<&dyn StoreView>`, `Mutation::DeleteUser`, `Mutation::DeleteGroup`, `Reply::mutating`.
- Produces: plugin name `cleanuser`, aliases `bersihuser`, `hapususerinvalid`, `cleangroup`.

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    #[test]
    fn kunci_user_valid_hanya_nomor_dan_lid() {
        assert!(kunci_user_valid("628123456789"));
        assert!(kunci_user_valid("lid:123456789012345"));
        assert!(!kunci_user_valid("628123456789@s.whatsapp.net"));
        assert!(!kunci_user_valid("628123456789:5"));
        assert!(!kunci_user_valid("unknown"));
        assert!(!kunci_user_valid("120363123456789"));
        assert!(!kunci_user_valid("62812abc"));
        assert!(kunci_user_valid("62812"));
    }

    #[test]
    fn kunci_grup_valid_hanya_jid_grup() {
        assert!(kunci_grup_valid("120363123456789012@g.us"));
        assert!(!kunci_grup_valid("120363123456789012"));
        assert!(!kunci_grup_valid("bukan-grup"));
        assert!(!kunci_grup_valid("120363123456789012@s.whatsapp.net"));
    }

    #[test]
    fn terdaftar_dengan_meta_owner() {
        let p = find("cleanuser").unwrap();
        assert_eq!(p.meta.category, "owner");
        assert!(p.meta.is_owner);
        assert_eq!(p.meta.alias, ["bersihuser", "hapususerinvalid", "cleangroup"]);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p oktz-plugins cleanuser`
Expected: compile error, module missing.

- [ ] **Step 3: Implement `plugin/src/owner/cleanuser.rs`**

Rules:

```rust
fn kunci_user_valid(key: &str) -> bool {
    if let Some(lid) = key.strip_prefix("lid:") {
        return (10..=20).contains(&lid.len()) && lid.bytes().all(|b| b.is_ascii_digit());
    }
    !key.is_empty()
        && key.len() <= 15
        && key.bytes().all(|b| b.is_ascii_digit())
        && !key.starts_with("120")
}

fn kunci_grup_valid(key: &str) -> bool {
    let Some(id) = key.strip_suffix("@g.us") else {
        return false;
    };
    id.len() >= 10 && id.bytes().all(|b| b.is_ascii_digit())
}
```

Command body:

```rust
pub fn cleanuser(ctx: Ctx) -> Reply {
    let Some(store) = ctx.store() else {
        return Reply::text("❌ Snapshot store tidak tersedia");
    };

    let mut mutations = Vec::new();
    let mut contoh_user = Vec::new();
    let mut contoh_grup = Vec::new();
    let mut user = 0usize;
    let mut grup = 0usize;

    store.for_each_user(&mut |key, value| {
        if !kunci_user_valid(key) || !value.is_object() {
            if contoh_user.len() < 10 {
                contoh_user.push(key.to_string());
            }
            user += 1;
            mutations.push(Mutation::DeleteUser { key: key.to_string() });
        }
    });
    store.for_each_group(&mut |key, value| {
        if !kunci_grup_valid(key) || !value.is_object() {
            if contoh_grup.len() < 10 {
                contoh_grup.push(key.to_string());
            }
            grup += 1;
            mutations.push(Mutation::DeleteGroup { key: key.to_string() });
        }
    });

    let mut teks = format!(
        "╭┈┈⬡「 🧹 *ᴄʟᴇᴀɴ ᴅᴀᴛᴀ* 」\n┃\n┃ 👤 User invalid: *{user}*\n┃ 👥 Grup invalid: *{grup}*"
    );
    if !contoh_user.is_empty() {
        teks.push_str(&format!("\n┃\n┃ User: `{}`", contoh_user.join("`, `")));
    }
    if !contoh_grup.is_empty() {
        teks.push_str(&format!("\n┃\n┃ Grup: `{}`", contoh_grup.join("`, `")));
    }
    teks.push_str("\n┃\n┃ ✅ Data valid tidak disentuh\n╰┈┈⬡");

    Reply::text(teks)
        .with_reaction("🧹")
        .mutating(mutations)
}
```

Plugin metadata: name `cleanuser`, aliases above, category `owner`, description `"Hapus user/grup invalid dari database"`, usage/example `.cleanuser`, cooldown 60, energi 0, owner only, not group/private/admin.

Add `pub mod cleanuser;` in `plugin/src/owner/mod.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p oktz-plugins cleanuser`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add plugin/src/owner/cleanuser.rs plugin/src/owner/mod.rs
git commit -m "feat(plugin): cleanuser hapus entri user/grup invalid"
```

---

### Task 3: Prune SQLite lokal

**Files:**
- Create: `lib/oktz-store/session_db.rs`
- Modify: `lib/oktz-store/lib.rs`
- Test: `lib/oktz-store/session_db.rs` test module

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PruneReport {
    pub msg_secrets: u64,
    pub tc_tokens: u64,
    pub sent_messages: u64,
    pub pending_inbound: u64,
}

pub async fn prune_expired(
    db_path: &str,
    now_secs: i64,
) -> oktz_core::error::Result<Option<PruneReport>>;
```

`Ok(None)` means the file does not exist.

- [ ] **Step 1: Write failing test**

```rust
#[tokio::test]
async fn prune_hanya_baris_kedaluwarsa() {
    let path = std::env::temp_dir().join(format!(
        "oktz-session-prune-{}.db",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
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
        "INSERT INTO tc_tokens VALUES ('a',x'00',100,NULL,0), ('b',x'00',999999,NULL,0)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO sent_messages VALUES ('c','1',x'00',100), ('c','2',x'00',999999)",
        (),
    )
    .await
    .unwrap();
    conn.execute(
        "INSERT INTO pending_inbound_messages VALUES ('c','s','1',x'00',100), ('c','s','2',x'00',999999)",
        (),
    )
    .await
    .unwrap();
    drop(conn);
    drop(db);

    let now = 1_000_000i64;
    let r = prune_expired(path.to_str().unwrap(), now).await.unwrap().unwrap();
    assert_eq!(
        r,
        PruneReport {
            msg_secrets: 1,
            tc_tokens: 1,
            sent_messages: 1,
            pending_inbound: 1,
        }
    );
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn prune_file_tidak_ada_mengembalikan_none() {
    let r = prune_expired("/tmp/oktz-tidak-ada-session.db", 1)
        .await
        .unwrap();
    assert!(r.is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p oktz-store session_db`
Expected: compile error, module missing.

- [ ] **Step 3: Implement `lib/oktz-store/session_db.rs`**

Use `libsql::Builder::new_local(db_path)`, `db.connect()`, then:

```sql
PRAGMA busy_timeout = 3000
DELETE FROM msg_secrets WHERE expires_at != 0 AND expires_at <= ?1
DELETE FROM tc_tokens
  WHERE (token = X'' OR token_timestamp < ?2)
    AND (sender_timestamp IS NULL OR sender_timestamp < ?2)
DELETE FROM sent_messages WHERE created_at < ?3
DELETE FROM pending_inbound_messages WHERE inserted_at < ?3
```

with `?1 = now_secs`, `?2 = now_secs - 30 * 86_400`, `?3 = now_secs - 7 * 86_400`.

Map every libsql error to `oktz_core::error::Error::Config(format!("session_db ...: {e}"))`. Use `conn.execute(sql, [param])` and take the returned affected-row count as `u64`.

Add `pub mod session_db;` to `lib/oktz-store/lib.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p oktz-store session_db`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add lib/oktz-store/session_db.rs lib/oktz-store/lib.rs
git commit -m "feat(store): prune baris session SQLite yang kedaluwarsa"
```

---

### Task 4: Integrasi `clearsessiondb`

**Files:**
- Modify: `plugin/src/owner/clearsessiondb.rs`
- Test: `plugin/src/owner/clearsessiondb.rs` test module

**Interfaces:**
- Consumes: `oktz_store::session_db::prune_expired(db_path: &str, now_secs: i64)`.

- [ ] **Step 1: Write failing test**

Add to the existing tests module:

```rust
#[test]
fn prune_lokal_dipanggil_untuk_db_yang_ada() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let d = scratch("prune-lokal");
    unsafe { std::env::set_var("OKTZ_DB_DIR", &d) };
    let path = d.join("oktz-run.db");
    let _ = std::fs::remove_file(&path);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let db = rt.block_on(libsql::Builder::new_local(&path).build()).unwrap();
    let conn = db.connect().unwrap();
    rt.block_on(async {
        conn.execute(
            "CREATE TABLE msg_secrets (chat TEXT, sender TEXT, msg_id TEXT, secret BLOB, created_at INTEGER, expires_at INTEGER, message_ts INTEGER)",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO msg_secrets VALUES ('c','s','1',x'00',0,1,0)",
            (),
        )
        .await
        .unwrap();
    });
    drop(conn);
    drop(db);

    let (lokal, gagal, prune) = bersih_lokal(&d.to_string_lossy(), "oktz-run.db");
    unsafe { std::env::remove_var("OKTZ_DB_DIR") };
    assert!(gagal.is_empty(), "{gagal:?}");
    assert_eq!(lokal, 0);
    assert_eq!(prune.unwrap().msg_secrets, 1);
    std::fs::remove_dir_all(&d).ok();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p oktz-plugins prune_lokal_dipanggil`
Expected: compile error, `bersih_lokal` missing.

- [ ] **Step 3: Implement**

Refactor the local section into:

```rust
fn bersih_lokal(
    db_dir: &str,
    base: &str,
) -> (usize, Vec<String>, Result<oktz_store::session_db::PruneReport, String>)
```

It calls `bersih_karantina`, then runs `prune_expired` on `<db_dir>/<base>` using a dedicated current-thread runtime in a scoped thread, mirroring `turso_bersih` so `block_in_place` is never used. Missing DB returns `PruneReport::default()`.

In `clearsessiondb`, add report lines:

```text
┃ 🗃️ msg_secrets: *{n}*
┃ 🗃️ tc_tokens: *{n}*
┃ 🗃️ sent_messages: *{n}*
┃ 🗃️ pending_inbound: *{n}*
```

On error, append `┃ ⚠️ Prune lokal: {e}` but still finish. Update `help` text to mention prune tabel kedaluwarsa dan prekey aktif dipertahankan.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p oktz-plugins clearsessiondb`
Expected: existing tests + new test pass.

- [ ] **Step 5: Commit**

```bash
git add plugin/src/owner/clearsessiondb.rs
git commit -m "feat(plugin): clearsessiondb prune session kedaluwarsa lokal"
```

---

### Task 5: Verifikasi dan rilis

**Files:**
- Modify: `launcher.js` (`VERSI_BIN`)

- [ ] **Step 1: Format and lint**

Run:
```bash
cargo fmt --all
git checkout -- plugin/src/common.rs
cargo clippy -p oktz-store -p oktz-wa -p oktz-plugins --all-targets
cargo check -p oktz-wa -p oktz-plugins --all-targets
```
Expected: no new warnings/errors from these changes.

- [ ] **Step 2: Run targeted tests**

Run:
```bash
cargo test -p oktz-store session_db
cargo test -p oktz-wa --lib delete_
cargo test -p oktz-plugins cleanuser
cargo test -p oktz-plugins clearsessiondb
```
Expected: all pass.

- [ ] **Step 3: Bump launcher fallback version**

Set `VERSI_BIN` in `launcher.js` to the next tag.

- [ ] **Step 4: Commit, push, tag, verify release**

```bash
git add -A
git commit -m "chore: rilis versi baru"
git push origin main
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
gh run list --limit 4
gh release view vX.Y.Z --json assets
```

Expected: `ci` and `release` success; assets `oktz-linux-x64`, `oktz-linux-x64-static`, `ffmpeg-linux-x64-static.gz`, `ffmpeg-linux-x64-static.sha256` present.
