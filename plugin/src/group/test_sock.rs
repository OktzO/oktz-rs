//! Stub `Sock` untuk test plugin grup — merekam `ubah_peserta`, sisanya gagal.
//! Satu salinan untuk promote/demote (kick punya stub sendiri yang lebih tua).

use oktz_plugin_api::{AdaWhatsapp, AksiPeserta, Sock, SockError};
use serde_json::Value;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub(crate) struct CatatSock {
    pub gagal: bool,
    pub seen: Mutex<Vec<(String, Vec<String>, AksiPeserta)>>,
}

impl CatatSock {
    fn gagal() -> SockError {
        SockError::baru("gagal")
    }
}

impl Sock for CatatSock {
    fn ada_whatsapp(&self, _: &[String]) -> Result<Vec<AdaWhatsapp>, SockError> {
        Err(Self::gagal())
    }
    fn foto_profil(&self, _: &str, _: &str) -> Result<Option<String>, SockError> {
        Err(Self::gagal())
    }
    fn metadata_grup(&self, _: &str) -> Result<Value, SockError> {
        Err(Self::gagal())
    }
    fn semua_grup(&self) -> Result<Value, SockError> {
        Err(Self::gagal())
    }
    fn ubah_peserta(&self, jid: &str, peserta: &[String], aksi: AksiPeserta) -> Result<(), SockError> {
        self.seen
            .lock()
            .unwrap()
            .push((jid.to_string(), peserta.to_vec(), aksi));
        if self.gagal {
            Err(Self::gagal())
        } else {
            Ok(())
        }
    }
    fn set_foto_profil(&self, _: &str, _: &[u8]) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn keluar_grup(&self, _: &str) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn kode_undangan(&self, _: &str) -> Result<String, SockError> {
        Err(Self::gagal())
    }
    fn info_undangan(&self, _: &str) -> Result<Value, SockError> {
        Err(Self::gagal())
    }
    fn buat_grup(&self, _: &str, _: &[String]) -> Result<Value, SockError> {
        Err(Self::gagal())
    }
    fn terima_undangan(&self, _: &str) -> Result<String, SockError> {
        Err(Self::gagal())
    }
    fn tandai_dibaca(&self, _: &[Value]) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn presence(&self, _: &str, _: &str) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn status_teks(&self, _: &str) -> Result<Value, SockError> {
        Err(Self::gagal())
    }
    fn nama_kontak(&self, _: &str) -> Result<String, SockError> {
        Err(Self::gagal())
    }
    fn profil_bisnis(&self, _: &str) -> Result<Option<Value>, SockError> {
        Err(Self::gagal())
    }
    fn set_bio(&self, _: &str) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn set_nama(&self, _: &str) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn hapus_foto_bot(&self) -> Result<(), SockError> {
        Err(Self::gagal())
    }
    fn blokir(&self, _: &str, _: bool) -> Result<(), SockError> {
        Err(Self::gagal())
    }
}
