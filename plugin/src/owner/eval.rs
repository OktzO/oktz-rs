//! `plugins/owner/eval.js` — REPL JavaScript milik owner.
//!
//! ## PENYIMPANGAN (tidak bisa ditiru)
//!
//! JS menjalankan `eval("(async () => { ... })()")` — artinya kode
//! JavaScript sembarang dieksekusi di dalam proses Node, dengan akses
//! penuh ke `sock`, `db`, `fs`, `axios`, dan `config`. Di Rust tidak ada
//! interpreter JavaScript, dan menambahkannya berarti memasukkan mesin
//! eksekusi skrip ke dalam bot WhatsApp — tidak Equivalent, dan tidak
//! aman. Karena itu perintah ini TIDAK menjalankan kode apa pun; ia
//! hanya menjelaskan bahwa fiturnya tidak tersedia di versi Rust.
//!
//! Semua cabang lain (`Owner Only!`, teks bantuan `⚙️ *ᴇᴠᴀʟ*`) tetap
//! disalin persis supaya perilakunya tidak berubah di luar batas yang
//! memang mustahil.
//!
//! `noPrefix: ['=>']` / `customTrigger` (`eval.js:23-24`) tidak punya
//! padanan di router Rust — perintah tetap dipanggil dengan prefix.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// `config.isOwner(m.sender)` gagal → `eval.js:32`.
pub(crate) const BUKAN_OWNER: &str = "❌ *Owner Only!*";

/// Teks bantuan `eval.js:37-45`.
pub(crate) fn bantuan() -> String {
    String::from(
        "⚙️ *ᴇᴠᴀʟ*\n\n\
         > Masukkan kode JavaScript!\n\n\
         *Contoh:*\n\
         > .$ 1 + 1\n\
         > .$ m.chat\n\
         > .$ db.getUser(m.sender)",
    )
}

/// Balasan untuk Versi Rust — `eval` tidak ada di sini.
pub(crate) fn tidak_tersedia() -> String {
    String::from(
        "⚙️ *ᴇᴠᴀʟ ʀᴇsᴜʟᴛ*\n\n\
         ╭┈┈⬡「 📋 *ɪɴғᴏ* 」\n\
         ┃ ❌ Error\n\
         ┃ Type: Error\n\
         ╰┈┈┈┈┈┈┈┈⬡\n\n\
         ```Fitur eval tidak tersedia di versi Rust.```\n\n\
         Bot Rust tidak menjalankan kode JavaScript. Perintah ini hanya\
         menampilkan informasi, bukan mengeksekusi kode.",
    )
}

#[plugin(
    name = "eval",
    alias = ["$", "ev", "evaluate", "=>"],
    category = "owner",
    description = "Jalankan kode JavaScript (Owner Only)",
    usage = "=> <code> atau .$ <code>",
    example = "=> m.chat",
    cooldown = 0,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn eval(ctx: Ctx) -> Reply {
    if !ctx.is_owner {
        return Reply::text(BUKAN_OWNER);
    }

    // `m.fullArgs?.trim() || m.text?.trim()` (`eval.js:35`).
    let code = ctx.text.trim();
    if code.is_empty() {
        return Reply::text(bantuan());
    }

    // `eval` tidak bisa ditiru — lihat catatan penyimpangan di doc-comment.
    Reply::text(tidak_tersedia())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, is_owner: bool) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "eval".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: text.into(),
            args: text.split_whitespace().map(str::to_string).collect(),
            is_owner,
            ..Default::default()
        };
        eval(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("eval").unwrap();
        assert_eq!(p.meta.name, "eval");
        assert_eq!(p.meta.alias, ["$", "ev", "evaluate", "=>"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Jalankan kode JavaScript (Owner Only)");
        assert_eq!(p.meta.usage, "=> <code> atau .$ <code>");
        assert_eq!(p.meta.example, "=> m.chat");
        assert_eq!(p.meta.cooldown, 0);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private && !p.meta.is_premium);
    }

    #[test]
    fn bukan_owner_menolak_dengan_teks_js() {
        let r = run("1+1", false);
        assert_eq!(teks(&r), "❌ *Owner Only!*");
    }

    #[test]
    fn tanpa_kode_menampilkan_bantuan_js() {
        for text in ["", "   "] {
            let r = run(text, true);
            assert_eq!(teks(&r), bantuan());
            assert!(r.mutations().is_empty());
        }
        assert!(bantuan().starts_with("⚙️ *ᴇᴠᴀʟ*"));
        assert!(bantuan().contains("> .$ 1 + 1"));
    }

    #[test]
    fn kode_tidak_dijalankan_melainkan_dijelaskan() {
        let r = run("m.chat", true);
        let t = teks(&r);
        assert!(t.contains("Fitur eval tidak tersedia di versi Rust"), "{t}");
        assert!(t.contains("❌ Error"), "{t}");
        assert!(t.contains("Type: Error"), "{t}");
    }
}
