//! Upload teks ke Pastebin.
//!
//! `Keterbatasan:` jalur POST-nya sendiri sudah bisa direplikasi sekarang lewat
//! `HttpCall::post(url).form(body)` (`registry.rs:1202-1247`); yang menahan port adalah
//! kunci dev + preview di atas. Begitu kunci bisa dibaca dari `Ctx::config()`/env dan
//! `Reply` punya varian preview, implementasi bisa ditulis tanpa menyentuh berkas ini lagi.
