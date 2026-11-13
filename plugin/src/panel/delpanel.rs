//! `plugins/panel/delpanel.js` — hapus server (+ user) Pterodactyl.
//!
//! `ponytail:` `config.pterodactyl.server1..3` (`config.js:258-299`) TIDAK
//! ada di `oktz_plugin_api::config::Runtime`, dan `Ctx::config()` read-only.
//! Mengikuti pola `cpanel.rs`, nilai `domain`/`apikey` di-hardcode dari
//! `config.js` yang memang kosong untuk ketiga server — jadi cabang
//! `validateServerConfig` (`delpanel.js:29-34`) selalu menemukan
//! `['domain', 'apikey (PTLA)']` dan handler berakhir di kartu
//! "sᴇʀᴠᴇʀ sX ʙᴇʟᴜᴍ ᴋᴏɴꜰɪɢ", persis perilaku produksi JS. Sisa alur
//! (GET server → GET user → DELETE → opsi `full`) tetap diport lewat
//! `ctx.http.call` supaya port penuh tinggal mengganti sumber konfigurasi;
//! lewat konfigurasi kosong jalur itu tidak terjangkau, dan test menjangkau
//! lewat `run_dengan_konfigurasi` dengan config palsu.

use oktz_plugin_api::{Ctx, HttpCall, Reply, plugin};

use crate::common::error_template;

/// `config.pterodactyl.serverN` — `domain`/`apikey` kosong di `config.js`.
struct KonfigServer {
    domain: &'static str,
    apikey: &'static str,
}

const SERVER1: KonfigServer = KonfigServer { domain: "", apikey: "" };
const SERVER2: KonfigServer = KonfigServer { domain: "", apikey: "" };
const SERVER3: KonfigServer = KonfigServer { domain: "", apikey: "" };

/// `validateServerConfig` (`delpanel.js:29-34`).
fn field_hilang(cfg: &KonfigServer) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if cfg.domain.is_empty() {
        missing.push("domain");
    }
    if cfg.apikey.is_empty() {
        missing.push("apikey (PTLA)");
    }
    missing
}

/// Header standar Pterodactyl (`delpanel.js:88-92`) — varian user GET hanya
/// membawa Authorization (`delpanel.js:102`).
fn headers(req: HttpCall, apikey: &str, penuh: bool) -> HttpCall {
    let req = req.header("Authorization", format!("Bearer {apikey}"));
    if penuh {
        req.header("Content-Type", "application/json")
            .header("Accept", "Application/vnd.pterodactyl.v1+json")
    } else {
        req
    }
}

#[plugin(
    name = "delpanel",
    alias = ["hapuspanel", "deletepanel"],
    category = "panel",
    description = "Hapus panel (server + user)",
    usage = ".delpanel [s1/s2/s3] serverid [full]",
    example = ".delpanel 5 atau .delpanel s2 5 full",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn delpanel(ctx: Ctx) -> Reply {
    kerjakan(&ctx, &SERVER1, &SERVER2, &SERVER3)
}

/// Badan handler, diparameterisasi supaya test bisa memakai konfigurasi palsu.
fn kerjakan(
    ctx: &Ctx,
    s1: &KonfigServer,
    s2: &KonfigServer,
    s3: &KonfigServer,
) -> Reply {
    // `m.text?.trim().split(' ')` — `ctx.args` sudah token split-whitespace.
    let mut server_key = "s1";
    let mut rest: &[String] = &ctx.args;
    if let Some(first) = ctx.args.first() {
        let kunci = first.to_ascii_lowercase();
        if kunci == "s1" || kunci == "s2" || kunci == "s3" {
            server_key = match kunci.as_str() {
                "s2" => "s2",
                "s3" => "s3",
                _ => "s1",
            };
            rest = &ctx.args[1..];
        }
    }
    let server_cfg = konfig_server_dari(server_key, s1, s2, s3);
    let missing = field_hilang(server_cfg);

    // `validateServerConfig` gagal → kartu "belum konfig" (`delpanel.js:59-66`).
    if !missing.is_empty() {
        let available = server_tersedia_dari(s1, s2, s3);
        let mut txt = format!("⚠️ *sᴇʀᴠᴇʀ {} ʙᴇʟᴜᴍ ᴋᴏɴꜰɪɢ*\n\n", server_key.to_uppercase());
        if !available.is_empty() {
            txt.push_str(&format!("> Server tersedia: *{}*", available.join(", ")));
        }
        return Reply::text(txt);
    }

    let Some(server_id) = rest.first() else {
        return Reply::text(format!(
            "⚠️ *ᴄᴀʀᴀ ᴘᴀᴋᴀɪ*\n\n\
             > `{prefix}delpanel ID` - Hapus server saja\n\
             > `{prefix}delpanel ID full` - Hapus server + user\n\
             > `{prefix}delpanel s2 ID` - Dari server 2\n\n\
             > Lihat ID dengan `{prefix}listserver`",
            prefix = ctx.prefix
        ));
    };
    let server_id = server_id.as_str();
    let option = rest.get(1).map(|s| s.to_ascii_lowercase());
    let server_label = server_key.to_uppercase();

    // `if (isNaN(serverId))` — string yang gagal dipaksa jadi angka.
    if server_id.trim().parse::<f64>().is_err() {
        return Reply::text("❌ Server ID harus berupa angka.");
    }

    match alur_hapus(ctx, server_cfg, server_id, option.as_deref(), &server_label) {
        Ok(reply) => reply,
        Err(()) => Reply::text(error_template(ctx)),
    }
}

fn konfig_server_dari<'a>(
    kunci: &str,
    s1: &'a KonfigServer,
    s2: &'a KonfigServer,
    s3: &'a KonfigServer,
) -> &'a KonfigServer {
    match kunci {
        "s2" => s2,
        "s3" => s3,
        _ => s1,
    }
}

fn server_tersedia_dari(s1: &KonfigServer, s2: &KonfigServer, s3: &KonfigServer) -> Vec<&'static str> {
    let mut out = Vec::new();
    for (nama, cfg) in [("s1", s1), ("s2", s2), ("s3", s3)] {
        if !cfg.domain.is_empty() && !cfg.apikey.is_empty() {
            out.push(nama);
        }
    }
    out
}

/// Alur setelah validasi (`delpanel.js:86-141`). `Err` = `catch` luar.
fn alur_hapus(
    ctx: &Ctx,
    server_cfg: &KonfigServer,
    server_id: &str,
    option: Option<&str>,
    server_label: &str,
) -> Result<Reply, ()> {
    let domain = server_cfg.domain;
    let apikey = server_cfg.apikey;

    // GET server → attributes.
    let req = headers(
        HttpCall::get(format!("{domain}/api/application/servers/{server_id}")),
        apikey,
        true,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let data = res.json().ok_or(())?;
    let server = data.get("attributes").ok_or(())?;
    let nama_server = server
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("undefined")
        .to_string();
    let user_id = server
        .get("user")
        .map(|v| v.to_string().trim_matches('"').to_string())
        .unwrap_or_else(|| "undefined".to_string());

    // GET user — kegagalan ditelan (`catch (e) {}`).
    let user_info = ctx
        .http
        .call(&headers(
            HttpCall::get(format!("{domain}/api/application/users/{user_id}")),
            apikey,
            false,
        ))
        .ok()
        .and_then(|r| r.json())
        .and_then(|d| d.get("attributes").cloned());
    let is_user_admin = user_info
        .as_ref()
        .and_then(|u| u.get("root_admin"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let username = user_info
        .as_ref()
        .and_then(|u| u.get("username"))
        .map(|v| v.as_str().unwrap_or("undefined").to_string());

    let mode = if option == Some("full") { "Server + User" } else { "Server saja" };

    // `await m.reply('🗑️ *ᴍᴇɴɢʜᴀᴘᴜs ᴘᴀɴᴇʟ...*...')` (`delpanel.js:108`).
    let tahap_satu = format!(
        "🗑️ *ᴍᴇɴɢʜᴀᴘᴜs ᴘᴀɴᴇʟ...*\n\n> Server: *{server_label}*\n> Panel: `{nama_server}`\n> Mode: *{mode}*"
    );

    let req = headers(
        HttpCall::new(
            oktz_plugin_api::HttpMethod::Delete,
            format!("{domain}/api/application/servers/{server_id}"),
        ),
        apikey,
        true,
    );
    ctx.http.call(&req).map_err(|_| ())?;

    let mut hasil = format!(
        "✅ *sᴇʀᴠᴇʀ ᴅɪʜᴀᴘᴜs [{server_label}]*\n\n> Nama: `{nama_server}`\n> ID: `{server_id}`"
    );

    if option == Some("full") {
        if let Some(username) = username.filter(|_| !is_user_admin) {
            let req = headers(
                HttpCall::new(
                    oktz_plugin_api::HttpMethod::Delete,
                    format!("{domain}/api/application/users/{user_id}"),
                ),
                apikey,
                true,
            );
            if ctx.http.call(&req).is_ok() {
                hasil.push_str("\n✅ *ᴜsᴇʀ ᴅɪʜᴀᴘᴜs*\n");
                hasil.push_str(&format!("> Username: `{username}`\n"));
                hasil.push_str(&format!("> ID: `{user_id}`"));
            } else {
                hasil.push_str("\n⚠️ User gagal dihapus (mungkin masih punya server lain)");
            }
        } else if is_user_admin {
            hasil.push_str("\n⚠️ User adalah Admin, tidak dihapus");
        }
    }

    Ok(Reply::sequence(vec![
        Reply::text(tahap_satu),
        Reply::text(hasil),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;

    fn ctx(command: &str, args: &[&str]) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    #[test]
    fn registrasi_meta_js() {
        let m = &find("delpanel").unwrap().meta;
        assert_eq!(m.name, "delpanel");
        assert_eq!(m.alias, ["hapuspanel", "deletepanel"]);
        assert_eq!(m.category, "panel");
        assert_eq!(m.description, "Hapus panel (server + user)");
        assert_eq!(m.usage, ".delpanel [s1/s2/s3] serverid [full]");
        assert_eq!(m.example, ".delpanel 5 atau .delpanel s2 5 full");
        assert_eq!(m.cooldown, 10);
        assert_eq!(m.energi, 0);
        assert!(m.is_owner);
        assert!(!m.is_group && !m.is_private && !m.is_premium);
        for a in ["hapuspanel", "deletepanel"] {
            assert_eq!(find(a).unwrap().meta.name, "delpanel", "alias {a}");
        }
    }

    #[test]
    fn konfigurasi_kosong_config_js_selalu_belum_konfig() {
        // `config.js` pterodactyl server1..3 domain/apikey kosong → JS produksi
        // selalu berhenti di kartu ini, apa pun argumennya.
        for args in [
            vec![],
            vec!["5"],
            vec!["s2", "5", "full"],
            vec!["s3", "7"],
            vec!["abc"],
        ] {
            let r = delpanel(ctx("delpanel", &args));
            let label = match args.first().map(|s| s.to_ascii_lowercase()) {
                Some(k) if k == "s2" || k == "s3" => k.to_uppercase(),
                _ => "S1".to_string(),
            };
            let t = teks(&r);
            assert_eq!(
                t,
                format!("⚠️ *sᴇʀᴠᴇʀ {label} ʙᴇʟᴜᴍ ᴋᴏɴꜰɪɢ*\n\n"),
                "args {args:?}"
            );
        }
    }

    #[test]
    fn field_hilang_meniru_validate_server_config() {
        assert_eq!(
            field_hilang(&KonfigServer { domain: "", apikey: "" }),
            vec!["domain", "apikey (PTLA)"]
        );
        assert!(field_hilang(&KonfigServer { domain: "http://x", apikey: "k" }).is_empty());
        assert_eq!(
            field_hilang(&KonfigServer { domain: "http://x", apikey: "" }),
            vec!["apikey (PTLA)"]
        );
    }

    #[test]
    fn konfigurasi_palsu_tanpa_http_jatuh_ke_template_error() {
        // `fetch`/`axios` gagal (NoHttp) → catch luar → `te(...)` tanpa reaksi.
        let cfg_ok = KonfigServer { domain: "http://panel.test", apikey: "k" };
        let r = kerjakan(&ctx("delpanel", &["5"]), &cfg_ok, &cfg_ok, &cfg_ok);
        assert!(
            teks(&r).contains("Ehhh~ command `.delpanel`"),
            "{}",
            teks(&r)
        );
        assert!(r.without_mutations().text_body().is_some());
    }

    #[test]
    fn server_id_bukan_angka_ditolak_sebelum_http() {
        let cfg_ok = KonfigServer { domain: "http://panel.test", apikey: "k" };
        let r = kerjakan(
            &ctx("delpanel", &["abc"]),
            &cfg_ok,
            &cfg_ok,
            &cfg_ok,
        );
        assert_eq!(teks(&r), "❌ Server ID harus berupa angka.");
    }

    #[test]
    fn server_tersedia_dari_konfigurasi_terisi() {
        let kosong = KonfigServer { domain: "", apikey: "" };
        let isi = KonfigServer { domain: "http://x", apikey: "k" };
        assert!(server_tersedia_dari(&kosong, &kosong, &kosong).is_empty());
        assert_eq!(server_tersedia_dari(&isi, &kosong, &isi), vec!["s1", "s3"]);
    }
}
