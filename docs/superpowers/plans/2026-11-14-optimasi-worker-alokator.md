# Isolasi Worker Blocking + Alokator Jemalloc — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Handler plugin berat (HTTP/ffmpeg/image) jalan di thread pool blocking tokio, dan alokator bin diganti jemalloc dengan decay, supaya bot tidak macet saat multi command paralel dan RSS turun kembali ke OS.

**Architecture:** `Router::route`/`answer` jadi async; titik dispatch handler dibungkus `tokio::task::spawn_blocking` dengan `catch_unwind` di dalam closure. Bin `oktz` pasang `#[global_allocator]` jemalloc + `tikv-jemalloc-ctl` untuk `background_thread`/decay 5 dtk.

**Tech Stack:** tokio `spawn_blocking`, tikv-jemallocator 0.6, tikv-jemalloc-ctl 0.6.

**Spec:** `docs/superpowers/specs/2026-11-14-optimasi-worker-alokator-design.md`

## Global Constraints

- Paritas byte-per-byte TIDAK berubah — jangan sentuh render/teks/balasan.
- Jangan sentuh: `ourin-md/`, `vendor/`, `scripts/paritas-audit.py`, `docs/paritas/*`, throttle 300ms (`pipeline.rs:4039`), parameter ffmpeg.
- `cargo update` TERLARANG — tambah dep baru saja (Cargo.lock menambah entri baru, dep lama tidak berubah).
- Komentar kode Bahasa Indonesia, hanya alasan non-obvious + rujukan baris JS / `ponytail:`.
- Urutan aman: `cargo build --release` → `cargo test --release` → `cargo clippy --release --all-targets`.
- `panic = "unwind"` di `[profile.release]` JANGAN diubah (`catch_unwind` plugin mensyaratkannya).

## Review Focus

1. **Panik plugin di thread blocking** → balasan "❌ Plugin error, coba lagi nanti." tetap terkirim, bukan crash/JoinError terbuka. Test: `panicking_handler_is_caught_not_propagated` (Task 1, sudah ada — konversi + jalankan).
2. **`route`/`answer` di runtime current-thread (test)** → `spawn_blocking` tetap sah di `#[tokio::test]`. Test: `handler_dijalankan_di_thread_bukan_tokio_worker` (Task 1).
3. **JoinError pada `answer`** → `None` (paritas `try {} catch {}` JS `handler.js:440-473`), bukan panic. Pin manual saat review Task 1 (kode `unwrap_or(None)`).
4. **Build statis `+crt-static` + jemalloc** → duplicate symbol/link gagal harus terdeteksi di langkah build statis, bukan di panel produksi. Verifikasi: langkah build statis Task 2.
5. **Gagal set opsi ctl jemalloc** → warn log + jalan lanjut, bukan panic. Pin manual saat review Task 2 (kode tidak ada `.unwrap()`).

---

### Task 1: `Router::route`/`answer` async + handler di `spawn_blocking`

**Files:**
- Modify: `lib/oktz-router/Cargo.toml` (deps tokio)
- Modify: `lib/oktz-router/lib.rs:86-174` (`route`), `lib.rs:161-174` (`answer`), `lib.rs:316-435` (tests)
- Modify: `lib/oktz-wa/pipeline.rs:4512` (`.answer(...).await`), `lib/oktz-wa/pipeline.rs:4589` (`.route(...).await`)

**Interfaces:**
- Consumes: `oktz_plugin_api::run_answer_handlers(ctx: Ctx) -> Option<Reply>` (`registry.rs:1527`), `HandlerFn = Arc<dyn Fn(Ctx) -> Reply + Send + Sync>` (`registry.rs:10`), semua trait `Ctx` sudah `Send + Sync`.
- Produces: `Router::route(&self, caller: &mut Caller, body: &str, plugin_lookup: impl Fn(&str) -> Option<&'static PluginEntry>) -> Decision` — kini `async`, tipe & perilaku Decision TIDAK berubah. `Router::answer(&self, caller: &Caller, body: &str) -> Option<Reply>` — kini `async`.

- [ ] **Step 1: Tulis test gagal di `lib/oktz-router/lib.rs` (mod tests)**

```rust
#[tokio::test]
async fn handler_dijalankan_di_thread_bukan_tokio_worker() {
    let s = stub();
    let r = Router::new(&s, ParseOptions::default(), 1000);
    let mut c = caller();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(None));
    let sink = seen.clone();
    let e = Box::leak(Box::new(PluginEntry {
        meta: PluginMeta {
            name: "tnt".into(),
            category: "tools".into(),
            ..Default::default()
        },
        handler: Arc::new(move |_| {
            *sink.lock().unwrap() = Some(std::thread::current().id());
            Reply::None
        }),
    }));
    let _ = r.route(&mut c, ".tnt", |_| Some(e)).await;
    let got = seen.lock().unwrap().clone().expect("handler harus jalan");
    assert_ne!(
        got,
        std::thread::current().id(),
        "handler harus di thread blocking, bukan thread tokio worker"
    );
}
```

- [ ] **Step 2: Jalankan test, pastikan gagal**

Run: `cargo test --release -p oktz-router --lib handler_dijalankan_di_thread`
Expected: FAIL — compile error (`.route(...).await` di fn sync, `#[tokio::test]` butuh dev-dep tokio).

- [ ] **Step 3: Tambah tokio di `lib/oktz-router/Cargo.toml`**

```toml
[dependencies]
# spawn_blocking: handler plugin berat jalan di thread pool blocking, bukan
# worker tokio — lihat docs/superpowers/specs/2026-11-14-optimasi-worker-alokator-design.md
tokio = { version = "1.53", features = ["rt"] }

[dev-dependencies]
tokio = { version = "1.53", features = ["rt", "macros"] }
```

(dep `[dev-dependencies]` linkme yang sudah ada tetap)

- [ ] **Step 4: Ubah `route` jadi async di `lib/oktz-router/lib.rs:86-153`**

Tanda tangan: `pub async fn route(&self, caller: &mut Caller, body: &str, plugin_lookup: impl Fn(&str) -> Option<&'static PluginEntry>) -> Decision`.

Semua gate TIDAK berubah. Hanya titik dispatch (baris 145-152) yang diganti:

```rust
let c = self.build_ctx(caller, &parsed);
// Handler jalan di thread pool blocking — HTTP/ffmpeg/image tidak menahan
// worker tokio (spec ronde-1 A; ureq timeout 20s, ffmpeg puluhan detik).
// catch_unwind tetap DI DALAM closure: panik plugin = balasan error,
// bukan JoinError yang telanjang.
let dipanik = tokio::task::spawn_blocking(move || {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (entry.handler)(c)))
})
.await;
match dipanik {
    Ok(Ok(r)) => Decision::Handled(r),
    Ok(Err(_)) => {
        tracing::error!("plugin panik, ditahan di router");
        Decision::Handled(Reply::text("❌ Plugin error, coba lagi nanti."))
    }
    Err(join) => {
        tracing::error!(error = %join, "spawn_blocking handler gagal jalan");
        Decision::Handled(Reply::text("❌ Plugin error, coba lagi nanti."))
    }
}
```

(`entry: &'static PluginEntry` dan `c: Ctx` keduanya `'static` + `Send` — sah dipindah.)

- [ ] **Step 5: Ubah `answer` jadi async di `lib/oktz-router/lib.rs:161-174`**

```rust
pub async fn answer(&self, caller: &Caller, body: &str) -> Option<Reply> {
    if all_answer_handlers().is_empty() {
        return None;
    }
    let parsed = parse_command(body, &self.parse_opts);
    let parsed = ParsedCommand {
        command: String::new(),
        text: body.to_string(),
        ..parsed
    };
    let ctx = self.build_ctx(caller, &parsed);
    // JoinError = handler mati sebelum sempat panik-tertangkap → None,
    // paritas try/catch JS (`handler.js:440-473`) yang menelan galat.
    tokio::task::spawn_blocking(move || oktz_plugin_api::run_answer_handlers(ctx))
        .await
        .unwrap_or(None)
}
```

- [ ] **Step 6: Konversi 10 test lama yang memanggil `route`/`answer` (`lib.rs:317-435`)**

`#[test]` → `#[tokio::test] async fn`, tambah `.await` di setiap `r.route(...)` / `r.answer(...)`. Nama & assertion semua test TIDAK berubah: `answer_menerima_pesan_mentah_bukan_hasil_parse`, `non_command_returns_not_a_command`, `command_without_plugin_is_silent`, `plugin_runs_and_returns_reply`, `mode_denied_before_plugin_lookup`, `permission_denied_returns_message`, `panicking_handler_is_caught_not_propagated`, `ctx_carries_parsed_fields_to_handler`, `answer_test_dua_pertama_dijawab_...` (3 test answer sekitar baris 293-324 — konversi semuanya yang memanggil `.route`/`.answer`).

- [ ] **Step 7: Pipeline — tambah `.await` di dua pemanggilan**

`lib/oktz-wa/pipeline.rs:4512`: `if let Some(reply) = router.answer(&caller, &body).await {`
`lib/oktz-wa/pipeline.rs:4589`: `let decision = router.route(&mut caller, &body, lookup).await;`

- [ ] **Step 8: Build + test crate router**

Run: `cargo build --release && cargo test --release -p oktz-router --lib`
Expected: BUILD OK, semua test router PASS (termasuk 2 test baru/lama panik).

- [ ] **Step 9: Test pipeline + integrasi**

Run: `cargo test --release -p oktz-wa --lib && cargo test --release --test live_route -- --nocapture`
Expected: PASS.

- [ ] **Step 10: Clippy**

Run: `cargo clippy --release --all-targets`
Expected: tanpa warning baru.

- [ ] **Step 11: Commit**

```bash
git add lib/oktz-router lib/oktz-wa/pipeline.rs
git commit -m "perf: handler plugin jalan di thread blocking, bukan worker tokio"
```

### Task 2: Alokator jemalloc di bin `oktz`

**Files:**
- Modify: `Cargo.toml` (bagian `[dependencies]` bin, setelah baris tokio `:44`)
- Modify: `src/main.rs` (bagian atas file + awal `main`)

**Interfaces:**
- Consumes: `oktz_core::logging::init` (sudah ada, `src/main.rs:107`).
- Produces: `#[global_allocator]` jemalloc aktif untuk target bin `oktz` saja; fungsi `atur_jemalloc()` private di `src/main.rs`. Tidak ada tipe publik baru.

- [ ] **Step 1: Tambah dua dep di `Cargo.toml` bin**

```toml
# Alokator jemalloc: memori bebas dikembalikan ke OS (decay), bukan ditahan
# glibc selamanya — RAM bot naik merangkak dulu karena ini. Setelan decay di
# src/main.rs::atur_jemalloc.
tikv-jemallocator = { version = "0.6", features = ["unprefixed_malloc_on_supported_platforms"] }
tikv-jemalloc-ctl = { version = "0.6", features = ["background_thread", "stats"] }
```

Versi terbaru yang cocok dicek lewat `cargo add tikv-jemallocator --dry-run` dulu (rustc 1.94); pilih versi stabil 0.6.x tanpa meng-upgrade dep lain. Kalau perlu, validasi API ctl via context7 docs (`/tikv/jemallocator`).

- [ ] **Step 2: Pasang allocator + setting di `src/main.rs`**

Di atas `fn main` (bagian atas file):

```rust
/// Alokator jemalloc — memori bebas kembali ke OS lewat decay, bukan ditahan
/// glibc (akar RSS naik-turun tidak turun; spec optimasi ronde 1-B).
#[global_allocator]
static ALOKATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// `background_thread` = purge otomatis; decay 5 dtk = halaman kotor bebas
/// dikembalikan ~5 detik setelah free. Gagal set = warn, bukan panic —
/// default jemalloc (decay 10 dtk) tetap lebih baik daripada glibc.
fn atur_jemalloc() {
    use tikv_jemalloc_ctl as ctl;
    let _ = ctl::background_thread::set(true)
        .inspect_err(|e| tracing::warn!(error = %e, "background_thread jemalloc gagal diset"));
    let _ = ctl::dirty_decay_ms::set(5000.0)
        .inspect_err(|e| tracing::warn!(error = %e, "dirty_decay_ms jemalloc gagal diset"));
    let _ = ctl::muzzy_decay_ms::set(5000.0)
        .inspect_err(|e| tracing::warn!(error = %e, "muzzy_decay_ms jemalloc gagal diset"));
}
```

(Apel API ctl dicek pada Step 1; jika signature beda — mis. `set` menerima `&Jemalloc` atau tipe lain — ikuti docs, prinsipnya tetap: set sekali, gagal = warn.)

Panggil `atur_jemalloc();` sebagai baris pertama `async fn main` (sebelum `oktz_core::logging::init` sekalipun boleh; warn ctl tetap terbaca karena subscriber sudah init saat itu — kalau belum, pindah panggilan ke SETELAH `logging::init`).

- [ ] **Step 3: Build + smoke bin**

Run: `cargo build --release && ./target/release/oktz plugins`
Expected: JSON `{"plugins": N, ...}` dengan N > 0, exit 0.

- [ ] **Step 4: Build statis seperti CI**

Run: `RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu && readelf -l target/x86_64-unknown-linux-gnu/release/oktz | grep INTERP; echo exit=$?`
Expected: build sukses; `grep INTERP` TIDAK menemukan apa pun (exit grep 1 = benar untuk aset statis). Kalau link gagal duplicate symbol → STOP, laporkan ke owner, JANGAN matikan fitur diam-diam.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "perf: alokator jemalloc + decay agar RSS kembali ke OS"
```

### Task 3: Verifikasi penuh

**Files:** tidak ada perubahan — hanya verifikasi.

- [ ] **Step 1: Test semua**

Run: `cargo test --release`
Expected: PASS semua (root, lib/*, plugin).

- [ ] **Step 2: Clippy penuh**

Run: `cargo clippy --release --all-targets`
Expected: bersih.

- [ ] **Step 3: Paritas**

Run: `python scripts/paritas-audit.py`
Expected: lulus (tidak ada file JS yang tersentuh, jadi wajib tetap hijau).

- [ ] **Step 4: Lint launcher + smoke plugin count**

Run: `node --test launcher.test.js && ./target/release/oktz plugins`
Expected: PASS; JSON plugins > 0.

- [ ] **Step 5: Catat ukuran RSS nyata (opsional, produksi)**

Setelah deploy: bandingkan `rss_kb` log CONNECTED vs setelah burst command berat — RSS harus naik lalu TURUN kembali dalam ~10 dtk (decay 5 dtk × 2). Kalau masih merangkak, itu sinyal lanjut ke ronde 2 (C+D churn per-pesan).
