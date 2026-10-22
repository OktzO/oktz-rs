//! `plugins/owner/delplugin.js` — hapus plugin berdasarkan nama.
//!
//! Port dari nol. Yang dipindah:
//! - teks bantuan saat nama tidak diberikan (`delplugin.js:48-54`).
//! - balasan "tidak ditemukan" (`:64`).
//! - balasan "berhasil dihapus" (`:75-80`).
//!
//! `ponytail:` registry plugin Rust bersifat STATIS — mendaftarkan plugin lewat
//! `linkme::distributed_slice` saat compile, dan tidak ada `import()` dinamis
//! seperti di JS. Karena itu plugin yang sudah terdaftar tidak bisa dilepas
//! saat runtime. Balasan_success replaced dengan penjelasan bahwa penghapusan
//! butuh build/restart atau versi JS. Pencarian plugin memakai registry
//! (`oktz_plugin_api::find`) — padanan paling dekat dari `findPluginFile` JS
//! yang mencocokkan nama/alias plugin.

use oktz_plugin_api::{Ctx, Reply, plugin};

/// Teks bantuan (`delplugin.js:48-54`) — prefiks `.` di sini ditulis keras di
/// JS, bukan `m.prefix`.
fn bantuan(push_name: &str) -> String {
    format!(
        "Halo *{push_name}*, sepertinya kamu lupa memasukkan nama plugin yang ingin dihapus.\n\n\
         Silakan gunakan format perintah berikut:\n\
         - .delplugin <nama plugin>\n\n\
         Contoh penggunaan:\n\
         - .delplugin bliblidl"
    )
}

/// `m.pushName` — kalau kosong, JS tetap menulis string kosong; di sini
/// dipakai apa adanya supaya teksnya sama.
fn push_name(ctx: &Ctx) -> String {
    ctx.push_name.clone()
}

#[plugin(
    name = "delplugin",
    alias = ["delpl", "hapusplugin", "removeplugin"],
    category = "owner",
    description = "Hapus plugin berdasarkan nama",
    usage = ".delplugin <nama>",
    example = ".delplugin bliblidl",
    cooldown = 5,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn delplugin(ctx: Ctx) -> Reply {
    // `m.fullArgs?.trim() || m.args?.[0]` (`delplugin.js:44`).
    let name = {
        let full = ctx.text.trim();
        if full.is_empty() {
            ctx.args.first().cloned().unwrap_or_default()
        } else {
            full.to_string()
        }
    };

    if name.is_empty() {
        return Reply::text(bantuan(&push_name(&ctx)));
    }

    let found = oktz_plugin_api::find(&name);

    if found.is_none() {
        return Reply::text(format!(
            "Maaf *{}*, plugin dengan nama {name} tidak dapat ditemukan.",
            push_name(&ctx)
        ))
        .with_reaction("❌");
    }

    // `ponytail:` unload + `fs.unlinkSync` tidak bisa dari `Reply` — registry
    // plugin Rust di-link saat compile (linkme), tidak ada pemuatan runtime.
    Reply::text(
        "⚠️ *PENGHAPUSAN PLUGIN TIDAK TERSEDIA*\n\n\
         Registry plugin pada versi Rust bersifat statis (didaftarkan saat \
         compile), jadi plugin tidak bisa dilepas saat runtime.\n\n\
         Hapus modul plugin-nya di `plugin/src/owner/` lalu build ulang bot.\n\n\
         Contoh JS: `.delplugin bliblidl`",
    )
    .with_reaction("☢")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    const ME: &str = "628111@s.whatsapp.net";

    fn run(teks: &str, push_name: &str) -> Reply {
        delplugin(Ctx {
            text: teks.into(),
            args: teks.split_whitespace().map(str::to_string).collect(),
            sender: ME.into(),
            chat: ME.into(),
            push_name: push_name.into(),
            prefix: ".".into(),
            command: "delplugin".into(),
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
        let p = find("delplugin").unwrap();
        assert_eq!(p.meta.name, "delplugin");
        assert_eq!(p.meta.alias, ["delpl", "hapusplugin", "removeplugin"]);
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Hapus plugin berdasarkan nama");
        assert_eq!(p.meta.usage, ".delplugin <nama>");
        assert_eq!(p.meta.example, ".delplugin bliblidl");
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_group && !p.meta.is_private);
    }

    #[test]
    fn alias_menunjuk_ke_plugin_ini() {
        for a in ["delpl", "hapusplugin", "removeplugin"] {
            assert_eq!(find(a).map(|e| e.meta.name.as_str()), Some("delplugin"), "{a}");
        }
    }

    #[test]
    fn tanpa_nama_tampil_bantuan() {
        assert_eq!(
            teks(&run("", "Budi")),
            "Halo *Budi*, sepertinya kamu lupa memasukkan nama plugin yang ingin dihapus.\n\n\
             Silakan gunakan format perintah berikut:\n\
             - .delplugin <nama plugin>\n\n\
             Contoh penggunaan:\n\
             - .delplugin bliblidl"
        );
        assert_eq!(reaksi(&run("", "Budi")), None);
    }

    #[test]
    fn plugin_tidak_ada_dikatakan_tidak_ditemukan() {
        let r = run("plugin-hantu-xyz", "Budi");
        assert_eq!(
            teks(&r),
            "Maaf *Budi*, plugin dengan nama plugin-hantu-xyz tidak dapat ditemukan."
        );
        assert_eq!(reaksi(&r), Some("❌"));
    }

    #[test]
    fn plugin_ada_tapi_tidak_bisa_dihapus_saat_runtime() {
        let r = run("star", "Budi");
        assert!(teks(&r).contains("PENGHAPUSAN PLUGIN TIDAK TERSEDIA"), "{}", teks(&r));
        assert_eq!(reaksi(&r), Some("☢"));
    }
}
