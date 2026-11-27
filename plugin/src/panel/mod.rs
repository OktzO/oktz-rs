pub mod cekjeda;
pub mod cpanel;
pub mod installtemabilling;
pub mod installtemaenigma;
pub mod installtemanebula;
pub mod installtemastellar;
pub mod jedacreate;
pub mod rolemanager;
pub mod uninstalltema;

// Ditambahkan untuk perilaku 1:1 dengan bot versi awal (kategori panel):
// file yang sudah ada di disk tapi belum dideklarasikan (orphan)
// port baru dari `plugins/panel/`.
pub mod createserver;
pub mod delpanel;

// Port baru dari `plugins/panel/`.
pub mod addseller;
pub mod linode;
