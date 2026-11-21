//! Jembatan future async → thread sinkron.
//!
//! Trait `Sock`/`MediaFetch` sinkron (mengikuti Baileys), klien WhatsApp
//! asinkron. Implementasinya memakai [`blokir`] untuk menunggu.
//!
//! Dua flavor runtime diperlakukan beda:
//! - Multi-thread: `block_in_place` mengembalikan worker ke pool selagi
//!   menunggu — aman dipanggil dari worker maupun thread blocking.
//! - Current-thread (1 vCPU): `block_in_place` PANIC. Dipakai `Handle::block_on`
//!   yang hanya sah dari LUAR worker — pemanggil wajib sudah berada di thread
//!   blocking (`spawn_blocking`) atau thread non-runtime.

use std::future::Future;

/// Blokir sampai `fut` selesai. `Err` hanya kalau tidak ada runtime tokio.
///
/// Lihat catatan modul soal kewajiban pemanggil di runtime current-thread.
pub fn blokir<F: Future>(fut: F) -> Result<F::Output, &'static str> {
    let handle =
        tokio::runtime::Handle::try_current().map_err(|_| "dipanggil tanpa runtime tokio")?;
    Ok(match handle.runtime_flavor() {
        tokio::runtime::RuntimeFlavor::CurrentThread => handle.block_on(fut),
        _ => tokio::task::block_in_place(|| handle.block_on(fut)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Current-thread: `blokir` wajib jalan dari thread blocking, bukan worker.
    // Future-nya menyentuh timer driver supaya terbukti wake-up lintas-thread
    // benar (bukan cuma future instan).
    #[tokio::test(flavor = "current_thread")]
    async fn current_thread_dari_thread_blocking() {
        let hasil = tokio::task::spawn_blocking(|| {
            blokir(async {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                7u8
            })
        })
        .await
        .expect("thread blocking");
        assert_eq!(hasil, Ok(7));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn multi_thread_dari_worker() {
        assert_eq!(blokir(async { 9u8 }), Ok(9));
    }
}
