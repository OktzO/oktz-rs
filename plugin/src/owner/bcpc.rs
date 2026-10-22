//! `plugins/owner/bcpc.js` — broadcast pesan ke semua kontak private chat.
//!
//! Port dari nol. Yang dipindah:
//! - teks panduan (`bcpc.js:44-51`).
//! - guard `global.statusBcpc` (`:54-58`) memakai key session yang sama
//!   dengan `owner/stopbcpc.rs` (`statusBcpc` / `stopBcpc`).
//! - jalur "tidak ada kontak" (`:115-120`).
//!
//! `ponytail:` JS collects kontak dari `sock.store.chats`,
//! `sock.store.messages`, dan `sock.store.contacts` (`:79-113`). `Ctx` tidak
//! punya daftar kontak global — `ctx.user_data` hanya memuat data pengirim,
//! dan `sock` hanya bisa melihat grup. Karena itu daftar kontak selalu kosong
//! dan handler berakhir di balasan "Tidak ada kontak ditemukan" — persis
//! seperti JS ketika tidak ada kontak yang terdeteksi. Broadcast sungguhan
//! (yang butuh `sock.sendMedia`/`sock.sendText` ke JID lain) juga tidak bisa
//! jalan: `Reply` hanya sampai ke `caller.chat`.
//!
//! `ponytail:` `jedaBcpc` dibaca dari `db.setting("jedaBcpc") || 5000`
//! (`:42`) — dipetakan ke key setting `jedaBcpc`.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `global.statusBcpc` — key yang sama dipakai `owner/stopbcpc.rs`.
const STATUS_KEY: &str = "statusBcpc";

/// `db.setting("jedaBcpc") || 5000` (`bcpc.js:42`).
fn jeda(ctx: &Ctx) -> i64 {
    ctx.setting_i64("jedaBcpc")
        .filter(|v| *v != 0)
        .unwrap_or(5000)
}

/// Panduan (`bcpc.js:44-51`).
fn panduan(ctx: &Ctx) -> String {
    let jeda = jeda(ctx);
    format!(
        "📱 *BROADCAST PRIVATE CHAT*\n\n\
         Jeda: {jeda}ms ({}s)\n\n\
         *PENGGUNAAN:*\n\
         • `{}bcpc <pesan>` — Kirim ke semua kontak\n\
         • `{}bcpc (reply media)` — Kirim dengan media\n\n\
         ⚠️ *Peringatan:* Bot akan mengirim pesan ke semua kontak yang tersimpan!\n\n\
         ℹ️ *Note:* Kontak hanya terdeteksi jika mereka sudah pernah mengirim pesan ke bot. Kontak yang hanya disimpan tapi belum pernah chat tidak akan muncul.",
        format_jeda_detik(jeda),
        ctx.prefix,
        ctx.prefix
    )
}

/// `(jeda / 1000).toFixed(1)` (`bcpc.js:43`).
fn format_jeda_detik(jeda: i64) -> String {
    let detik = jeda as f64 / 1000.0;
    format!("{detik:.1}")
}

#[plugin(
    name = "bcpc",
    alias = ["broadcastpc", "bcprivate"],
    category = "owner",
    description = "Broadcast pesan ke semua kontak private chat",
    usage = ".bcpc <pesan>",
    example = ".bcpc <pesan>",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bcpc(ctx: Ctx) -> Reply {
    let input = ctx.text.trim();

    if input.is_empty() {
        return Reply::text(panduan(&ctx));
    }

    if ctx
        .session_get(STATUS_KEY)
        .is_some_and(|v| crate::common::js_truthy_value(&v))
    {
        return Reply::text(format!(
            "❌ Broadcast private sedang berjalan.\nKetik `{}stopbcpc` untuk menghentikan.",
            ctx.prefix
        ));
    }

    // `m.react("📱")` (`:60`) — reaksi jadi satu-satunya reaksi balasan.
    // Tidak ada kontak yang bisa dikumpulkan (lihat catatan modul), jadi
    // jalurnya sama dengan JS saat daftar kontak kosong.
    Reply::text(
        "❌ Tidak ada kontak ditemukan.\n\n\
         Pastikan bot sudah pernah menerima pesan dari kontak tersebut.",
    )
    .with_reaction("📱")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(teks: &str, settings: serde_json::Value, sessions: &oktz_plugin_api::SessionStore) -> Reply {
        bcpc(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "bcpc".into(),
            settings,
            sessions: sessions.clone(),
            ..Ctx::default()
        })
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or_default().to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji),
            _ => None,
        }
    }

    #[test]
    fn terdaftar_dengan_meta_js() {
        let p = find("bcpc").unwrap();
        assert_eq!(p.meta.name, "bcpc");
        assert_eq!(p.meta.alias, ["broadcastpc", "bcprivate"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Broadcast pesan ke semua kontak private chat");
        assert_eq!(p.meta.usage, ".bcpc <pesan>");
        assert_eq!(p.meta.cooldown, 10);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["broadcastpc", "bcprivate"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("bcpc"), "{a}");
        }
    }

    #[test]
    fn tanpa_pesan_tampil_panduan_dengan_jeda_bawaan() {
        let s = oktz_plugin_api::new_session_store();
        let r = run("", serde_json::Value::Null, &s);
        let t = teks(&r);
        assert!(t.starts_with("📱 *BROADCAST PRIVATE CHAT*"), "{t}");
        assert!(t.contains("Jeda: 5000ms (5.0s)"), "{t}");
        assert!(t.contains("• `.bcpc <pesan>` — Kirim ke semua kontak"), "{t}");
        assert!(t.contains("⚠️ *Peringatan:* Bot akan mengirim pesan ke semua kontak yang tersimpan!"), "{t}");
    }

    #[test]
    fn jeda_dari_setting_dipakai() {
        let s = oktz_plugin_api::new_session_store();
        let r = run(
            "",
            serde_json::json!({ "jedaBcpc": 2500 }),
            &s,
        );
        assert!(teks(&r).contains("Jeda: 2500ms (2.5s)"), "{}", teks(&r));
    }

    #[test]
    fn broadcast_sedang_berjalan_ditegah() {
        let s = oktz_plugin_api::new_session_store();
        s.lock()
            .unwrap()
            .insert("statusBcpc".to_string(), serde_json::json!(true));
        let r = run("halo", serde_json::Value::Null, &s);
        assert_eq!(
            teks(&r),
            "❌ Broadcast private sedang berjalan.\nKetik `.stopbcpc` untuk menghentikan."
        );
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn tanpa_kontak_balasan_gagal_dan_reaksi_telepon() {
        let s = oktz_plugin_api::new_session_store();
        let r = run("halo dunia", serde_json::Value::Null, &s);
        assert_eq!(
            teks(&r),
            "❌ Tidak ada kontak ditemukan.\n\n\
             Pastikan bot sudah pernah menerima pesan dari kontak tersebut."
        );
        assert_eq!(reaksi(&r), Some("📱"));
    }
}
