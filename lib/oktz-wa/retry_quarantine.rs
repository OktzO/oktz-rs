//! Kuarantina retry receipt grup agar bot tidak banjir resend.
//!
//! WhatsApp Web memproses SETIAP retry receipt; di grup besar, anggota yang
//! sesinya tidak pernah jadi bisa mengirim retry untuk tiap pesan dan memicu
//! badai `Resending message` berturut-turut. whatsapp-rust menyediakan hook
//! [`RetryAdmission`] untuk bot: token bucket per (grup, pengirim). Contoh
//! resmi di `whatsapp-rust-0.7.0/examples/retry_quarantine.rs`; berkas ini
//! versi produksinya dengan anggaran longgar supaya pemulihan wajar tetap
//! jalan.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use whatsapp_rust::prelude::Client;
use whatsapp_rust::{Jid, RetryAdmission};

/// Jatah awal per (grup, pengirim) sebelum kuarantina aktif.
const BURST: f64 = 2.0;
/// Isi ulang token per jam per pasangan; pemulihan tetap mungkin.
const ISI_PER_JAM: f64 = 6.0;
/// Batas jumlah pasangan yang dilacak; kelebihan pasangan baru diloloskan.
const MAKS_PASANGAN: usize = 8192;

struct Bucket {
    token: f64,
    terakhir: Instant,
}

impl Bucket {
    fn ambil(&mut self, sekarang: Instant, burst: f64, isi_per_detik: f64) -> bool {
        let lewat = sekarang
            .saturating_duration_since(self.terakhir)
            .as_secs_f64();
        self.token = (self.token + lewat * isi_per_detik).min(burst);
        self.terakhir = sekarang;
        if self.token >= 1.0 {
            self.token -= 1.0;
            true
        } else {
            false
        }
    }
}

struct Kuarantina {
    buckets: Mutex<HashMap<(String, String), Bucket>>,
    ditolak: AtomicU64,
}

impl RetryAdmission for Kuarantina {
    fn admit(&self, chat: &Jid, requester: &Jid, _retry_count: u8) -> bool {
        let sekarang = Instant::now();
        let key = (chat.user.to_string(), requester.user.to_string());
        let mut buckets = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        if !buckets.contains_key(&key) && buckets.len() >= MAKS_PASANGAN {
            return true;
        }
        let isi_per_detik = ISI_PER_JAM / 3600.0;
        let allowed = buckets
            .entry(key)
            .or_insert_with(|| Bucket {
                token: BURST,
                terakhir: sekarang,
            })
            .ambil(sekarang, BURST, isi_per_detik);
        drop(buckets);
        if !allowed {
            self.ditolak.fetch_add(1, Ordering::Relaxed);
        }
        allowed
    }
}

/// Pasang kuarantina pada klien produksi. `false` = policy sudah pernah dipasang.
pub fn pasang(client: &Arc<Client>) -> bool {
    client.set_retry_admission(Arc::new(Kuarantina {
        buckets: Mutex::new(HashMap::new()),
        ditolak: AtomicU64::new(0),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grup() -> Jid {
        "123-456@g.us".parse().unwrap()
    }

    fn orang(id: &str) -> Jid {
        format!("{id}@lid").parse().unwrap()
    }

    #[test]
    fn pasangan_yang_sama_dibatasi_setelah_burst() {
        let k = Kuarantina {
            buckets: Mutex::new(HashMap::new()),
            ditolak: AtomicU64::new(0),
        };
        assert!(k.admit(&grup(), &orang("111"), 1));
        assert!(k.admit(&grup(), &orang("111"), 1));
        assert!(!k.admit(&grup(), &orang("111"), 1));
        assert_eq!(k.ditolak.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn pengirim_lain_punya_jatah_sendiri() {
        let k = Kuarantina {
            buckets: Mutex::new(HashMap::new()),
            ditolak: AtomicU64::new(0),
        };
        assert!(k.admit(&grup(), &orang("111"), 1));
        assert!(k.admit(&grup(), &orang("222"), 1));
    }
}
