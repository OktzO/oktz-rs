//! `plugins/owner/startschedule.js` — mulai ulang scheduler tertentu atau semua.
//!
//! ## Penyimpangan
//!
//! - `startSchedulerByName` / `initSholatScheduler` (`src/lib/scheduler.js`,
//!   `src/lib/sholat-scheduler.js`) membuat cron job di dalam proses Node.
//!   Di Rust tidak ada penjadwal di API plugin, jadi scheduler yang "aktif"
//!   dicatat di `ctx.sessions` (memori proses, seperti `activeCronJobs` di JS)
//!   dan penandaan `autoSholat` tetap ditulis ke store lewat
//!   `Mutation::SetSetting`.
//! - Nama scheduler untuk "all" adalah `All Schedulers` (`scheduler.js:435`),
//!   sedangkan "sholat" punya blok balasan sendiri.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};

/// Nama scheduler yang dikenal dan label tampilnya (`scheduler.js:389-441`).
const SCHEDULER: [(&str, &str); 5] = [
    ("limitreset", "Daily Limit Reset"),
    ("groupschedule", "Group Schedule"),
    ("sewa", "Sewa Checker"),
    ("messages", "Schedule Planner"),
    ("sholat", "Sholat Scheduler"),
];

/// Teks bantuan `startschedule.js:26-41`.
pub(crate) fn bantuan() -> String {
    String::from(
        "▶️ *sᴛᴀʀᴛ sᴄʜᴇᴅᴜʟᴇʀ*\n\n\
         *Usage:*\n\
         `.startschedule <nama>`\n\n\
         *Available schedulers:*\n\
         • `limitreset` - Daily Limit Reset\n\
         • `groupschedule` - Group Schedule\n\
         • `sewa` - Sewa Checker\n\
         • `messages` - Scheduled Messages\n\
         • `sholat` - Sholat Scheduler\n\
         • `all` - Semua scheduler\n\n\
         *Example:*\n\
         `.startschedule sholat`\n\
         `.startschedule all`",
    )
}

/// Blok "dimulai" `startschedule.js:77-82`.
pub(crate) fn dimulai(nama: &str) -> String {
    format!(
        "▶️ *sᴄʜᴇᴅᴜʟᴇʀ ᴅɪᴍᴜʟᴀɪ*\n\n\
         > Scheduler: *{nama}*\n\
         > Status: ✅ Aktif\n\n\
         _Scheduler telah dimulai kembali_"
    )
}

/// Blok khusus sholat `startschedule.js:59-64`.
pub(crate) fn dimulai_sholat() -> String {
    String::from(
        "▶️ *sᴄʜᴇᴅᴜʟᴇʀ ᴅɪᴍᴜʟᴀɪ*\n\n\
         > Scheduler: *Sholat Scheduler*\n\
         > Status: ✅ Aktif\n\n\
         _Notifikasi waktu sholat akan dikirim ke grup yang mengaktifkan fitur ini_",
    )
}

/// `startschedule.js:84-86`.
pub(crate) const TIDAK_ADA: &str =
    "❌ Scheduler tidak ditemukan atau sudah aktif\n\nGunakan `.startschedule` untuk melihat daftar scheduler";

/// `startschedule.js:52`.
pub(crate) const SHOLAT_AKTIF: &str = "ℹ️ Sholat Scheduler sudah dalam keadaan aktif";

/// Kunci session untuk menandai scheduler yang aktif.
fn kunci(nama: &str) -> String {
    format!("scheduler:{nama}")
}

/// Nama tampilan scheduler dari kuncinya.
fn cari_scheduler(key: &str) -> Option<&'static str> {
    let key = key.to_lowercase().replace([' ', '-'], "");
    SCHEDULER
        .iter()
        .find(|(k, _)| {
            *k == key
                || match *k {
                    "limitreset" => key == "limit" || key == "dailylimitreset",
                    "groupschedule" => key == "group" || key == "groupsched",
                    "sewa" => key == "sewachecker",
                    "messages" => key == "msg" || key == "scheduledmessages",
                    "sholat" => false,
                    _ => false,
                }
        })
        .map(|(_, label)| *label)
}

#[plugin(
    name = "startschedule",
    alias = ["startscheduler", "schedstart", "resumeschedule"],
    category = "owner",
    description = "Memulai ulang scheduler tertentu atau semua",
    usage = ".startschedule <nama|all>",
    example = ".startschedule sholat",
    cooldown = 3,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn startschedule(ctx: Ctx) -> Reply {
    let target = ctx
        .args
        .first()
        .map(|a| a.to_lowercase())
        .unwrap_or_default();

    if target.is_empty() {
        return Reply::text(bantuan());
    }

    // Cabang khusus: `target === 'sholat'`.
    if target == "sholat" {
        if crate::common::js_truthy(ctx.setting("autoSholat")) {
            return Reply::text(SHOLAT_AKTIF);
        }
        ctx.session_set(&kunci("sholat"), serde_json::json!(true));
        return Reply::text(dimulai_sholat()).with_mutation(Mutation::SetSetting {
            key: "autoSholat".to_string(),
            value: serde_json::json!(true),
        });
    }

    if target == "all" {
        ctx.session_set(&kunci("all"), serde_json::json!(true));
        for (nama, _) in SCHEDULER {
            ctx.session_set(&kunci(nama), serde_json::json!(true));
        }
        return Reply::text(dimulai("All Schedulers"))
            .with_mutation(Mutation::SetSetting {
                key: "autoSholat".to_string(),
                value: serde_json::json!(true),
            });
    }

    let Some(label) = cari_scheduler(&target) else {
        return Reply::text(TIDAK_ADA);
    };
    if ctx.session_get(&kunci(&target)).is_some() {
        return Reply::text(TIDAK_ADA);
    }
    ctx.session_set(&kunci(&target), serde_json::json!(true));
    Reply::text(dimulai(label))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str], settings: serde_json::Value) -> Reply {
        let ctx = Ctx {
            prefix: ".".into(),
            command: "startschedule".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: args.join(" "),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            settings,
            ..Default::default()
        };
        startschedule(ctx)
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn meta_dan_alias_sama_dengan_js() {
        let p = oktz_plugin_api::find("startschedule").unwrap();
        assert_eq!(p.meta.name, "startschedule");
        assert_eq!(
            p.meta.alias,
            ["startscheduler", "schedstart", "resumeschedule"]
        );
        assert_eq!(p.meta.category, "owner");
        assert_eq!(p.meta.description, "Memulai ulang scheduler tertentu atau semua");
        assert_eq!(p.meta.usage, ".startschedule <nama|all>");
        assert_eq!(p.meta.example, ".startschedule sholat");
        assert_eq!(p.meta.cooldown, 3);
        assert_eq!(p.meta.energi, 0);
        assert!(p.meta.is_owner);
    }

    #[test]
    fn tanpa_nama_menampilkan_daftar_scheduler() {
        let r = run(&[], serde_json::json!({}));
        assert_eq!(teks(&r), bantuan());
        assert!(bantuan().contains("• `limitreset` - Daily Limit Reset"));
        assert!(bantuan().contains("• `all` - Semua scheduler"));
        assert!(bantuan().contains(".startschedule sholat"));
    }

    #[test]
    fn scheduler_tidak_dikenal_ditolak() {
        let r = run(&["ngawur"], serde_json::json!({}));
        assert_eq!(teks(&r), TIDAK_ADA);
    }

    #[test]
    fn notifying_scheduler_dimulai_dengan_label_benar() {
        let r = run(&["sewa"], serde_json::json!({}));
        assert_eq!(
            teks(&r),
            "▶️ *sᴄʜᴇᴅᴜʟᴇʀ ᴅɪᴍᴜʟᴀɪ*\n\n\
             > Scheduler: *Sewa Checker*\n\
             > Status: ✅ Aktif\n\n\
             _Scheduler telah dimulai kembali_"
        );
    }

    #[test]
    fn sholat_yang_sudah_aktif_memberi_informasi() {
        let r = run(&["sholat"], serde_json::json!({ "autoSholat": true }));
        assert_eq!(teks(&r), SHOLAT_AKTIF);
    }

    #[test]
    fn sholat_aktif_menulis_pengaturan_dan_balok_khusus() {
        let r = run(&["sholat"], serde_json::json!({}));
        assert_eq!(teks(&r), dimulai_sholat());
        assert_eq!(r.mutations().len(), 1);
        match &r.mutations()[0] {
            Mutation::SetSetting { key, value } => {
                assert_eq!(key, "autoSholat");
                assert_eq!(value, &serde_json::json!(true));
            }
            other => panic!("harus SetSetting, dapat {other:?}"),
        }
    }

    #[test]
    fn all_menyalakan_semua_scheduler() {
        let r = run(&["all"], serde_json::json!({}));
        assert!(teks(&r).contains("> Scheduler: *All Schedulers*"));
        assert_eq!(r.mutations().len(), 1);
    }
}
