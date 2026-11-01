//! `plugins/panel/linode.js` — manajemen VPS Linode (13 command).
//!
//! `pluginConfig.name` JS adalah array 13 nama dengan `alias: []`, jadi elemen
//! pertama (`linode2gb`) jadi `name` dan 12 sisanya masuk `commands` — pola
//! yang sama dengan `panel/uninstalltema.rs`.
//!
//! `ponytail:` JS memakai `fetch` global dengan header `Authorization: Bearer`,
//! jadi semua permintaan lewat [`oktz_plugin_api::HttpCall`] +
//! `ctx.http.call` (bukan `http.get` polos yang membuang header). Semua
//! kegagalan — fetch error, JSON rusak, `!res.ok` — jatuh ke `catch` yang sama:
//! `m.react('☢')` + `te(prefix, command, pushName)` (`linode.js:375-378`).
//!
//! Alur `linode*gb` menunggu 60 detik di tengah handler
//! (`await new Promise(r => setTimeout(r, 60000))`, `linode.js:94`) —
//! diterjemahkan apa adanya dengan `std::thread::sleep`.

use oktz_plugin_api::{Ctx, HttpCall, HttpMethod, Reply, plugin};
use serde_json::Value;
use rand::Rng;

use crate::common::error_template;

/// `linode.js:21-26` — spesifikasi per command create.
struct Spec {
    command: &'static str,
    jenis: &'static str,
    ram: &'static str,
    label: &'static str,
}

const LINODE_TYPES: [Spec; 4] = [
    Spec { command: "linode2gb", jenis: "g6-standard-1", ram: "2GB", label: "2GB" },
    Spec { command: "linode4gb", jenis: "g6-standard-2", ram: "4GB", label: "4GB" },
    Spec { command: "linode8gb", jenis: "g6-standard-4", ram: "8GB", label: "8GB" },
    Spec { command: "linode16gb", jenis: "g6-standard-8", ram: "16GB", label: "16GB" },
];

/// `linode.js:4` — tanpa `I`, `O`, `l` (huruf yang membingungkan).
const KARAKTER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz";
/// `linode.js:13` — tanpa `0`, `1`.
const NOMOR: &[u8] = b"23456789";

/// `randomKarakter(length)` — `Math.floor(Math.random() * chars.length)`,
/// batas atas eksklusif.
fn random_karakter(panjang: usize) -> String {
    (0..panjang)
        .map(|_| KARAKTER[rand::rng().random_range(0..KARAKTER.len())] as char)
        .collect()
}

/// `randomNomor(length)`.
fn random_nomor(panjang: usize) -> String {
    (0..panjang)
        .map(|_| NOMOR[rand::rng().random_range(0..NOMOR.len())] as char)
        .collect()
}

/// Pesan saat token belum diset (`linode.js:47-48`) — string kosong itu
/// falsy di JS, sama seperti `String::is_empty()` di sini.
fn token_kosong() -> String {
    "❌ Linode API Token tidak dikonfigurasi!\n\n\
     Tambahkan di config.js:\n\
     ```\n\
     APIkey: {\n  \
     linode: 'YOUR_LINODE_TOKEN'\n\
     }\n\
     ```"
        .to_string()
}

/// Menu bantuan (`linode.js:358-373`) — tidak pernah tercapai lewat registry
/// (semua 13 nama adalah command), dibawa untuk paritas.
fn menu_bantuan() -> String {
    "☁️ *ʟɪɴᴏᴅᴇ ᴄᴏᴍᴍᴀɴᴅs*\n\n\
     > .linode2gb <label> - Buat VPS 2GB\n\
     > .linode4gb <label> - Buat VPS 4GB\n\
     > .linode8gb <label> - Buat VPS 8GB\n\
     > .linode16gb <label> - Buat VPS 16GB\n\
     > .listlinode - Daftar VPS\n\
     > .onlinode <id> - Hidupkan VPS\n\
     > .offlinode <id> - Matikan VPS\n\
     > .rebootlinode <id> - Restart VPS\n\
     > .rebuildlinode <id> <image> - Rebuild\n\
     > .delinode <id> - Hapus VPS\n\
     > .saldolinode - Cek saldo\n\
     > .sisalinode - Total VPS\n\
     > .cekvpslinode <id> - Detail VPS"
        .to_string()
}

/// Pesan bantuan per-command saat argumen id/label kosong.
fn minta_id(prefix: &str, cmd: &str, contoh_id: &str) -> String {
    format!("❌ Masukkan ID Linode!\n\nContoh: {prefix}{cmd} {contoh_id}")
}

/// `${nilai}` di dalam template literal JS — `undefined` untuk null/hilang,
/// angka dan bool diratakan, array digabung koma (dipakai `ipv4`).
fn js_str(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::from("undefined"),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| match x {
                Value::Null => String::new(),
                other => js_str(Some(other)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => String::from("[object Object]"),
    }
}

/// `${l.ipv4?.[0] || '-'}` — string kosong itu falsy, jadi jatuh ke `'-'`.
fn ipv4_pertama(info: &Value) -> String {
    let s = info
        .get("ipv4")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
        .unwrap_or("");
    if s.is_empty() { "-".to_string() } else { s.to_string() }
}

/// Header standar semua permintaan Linode (`linode.js:78-81` dll).
fn headers(req: HttpCall, token: &str) -> HttpCall {
    req.header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
}

/// `res.ok` JS — status 200-299.
fn res_ok(status: u16) -> bool {
    (200..300).contains(&status)
}

/// `catch` bersama: `m.react('☢')` + `te(m.prefix, m.command, m.pushName)`.
fn gagal(ctx: &Ctx) -> Reply {
    Reply::text(error_template(ctx)).with_reaction("☢")
}

#[plugin(
    name = "linode2gb",
    commands = [
        "linode4gb", "linode8gb", "linode16gb",
        "listlinode", "onlinode", "offlinode", "rebootlinode", "rebuildlinode",
        "delinode", "saldolinode", "sisalinode", "cekvpslinode",
    ],
    alias = [],
    category = "linode",
    description = "Linode VPS Management",
    usage = ".linode2gb <label> | .listlinode | .onlinode <id> | dst",
    example = ".linode2gb myserver",
    cooldown = 10,
    energi = 0,
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn linode(ctx: Ctx) -> Reply {
    let token = ctx.config().api_keys.linode.clone();
    if token.is_empty() {
        return Reply::text(token_kosong());
    }

    match kerjakan(&ctx, &token) {
        Ok(reply) => reply,
        Err(()) => gagal(&ctx),
    }
}

/// Dispatch satu command; `Err` = jalur `catch` JS.
fn kerjakan(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let cmd = ctx.command.as_str();

    if let Some(spec) = LINODE_TYPES.iter().find(|s| s.command == cmd) {
        return buat_linode(ctx, token, spec);
    }

    match cmd {
        "listlinode" => listlinode(ctx, token),
        "onlinode" => aksi_id(ctx, token, "boot", "🔌", "berhasil dihidupkan!"),
        "offlinode" => aksi_id(ctx, token, "shutdown", "🔌", "berhasil dimatikan!"),
        "rebootlinode" => aksi_id(ctx, token, "reboot", "🔄", "berhasil di-reboot!"),
        "rebuildlinode" => rebuild(ctx, token),
        "delinode" => delinode(ctx, token),
        "saldolinode" => saldo(ctx, token),
        "sisalinode" => sisalinode(ctx, token),
        "cekvpslinode" => cekvpslinode(ctx, token),
        // Tidak terjangkau lewat registry (semua nama terdaftar), dibawa
        // untuk paritas dengan `else` terakhir di JS.
        _ => Ok(Reply::text(menu_bantuan())),
    }
}

/// Cabang create `linode2gb`..`linode16gb` (`linode.js:54-118`).
fn buat_linode(ctx: &Ctx, token: &str, spec: &Spec) -> Result<Reply, ()> {
    let Some(label) = ctx.args.first() else {
        return Ok(Reply::text(format!(
            "❌ Masukkan label untuk VPS!\n\nContoh: {}{} myserver",
            ctx.prefix, spec.command
        )));
    };
    let label = label.clone();

    // `randomKarakter(5) + randomNomor(3)` (`linode.js:61`).
    let root_pass = format!("{}{}", random_karakter(5), random_nomor(3));

    let body = serde_json::json!({
        "label": label,
        "region": "ap-south",
        "type": spec.jenis,
        "image": "linode/ubuntu20.04",
        "root_pass": root_pass,
        "stackscript_id": Value::Null,
        "authorized_keys": Value::Null,
        "backups_enabled": false,
    });

    let req = headers(
        HttpCall::post("https://api.linode.com/v4/linode/instances"),
        token,
    )
    .json(&body);
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let data = res.json().ok_or(())?;
    if !res_ok(res.status) {
        // `throw new Error(createData.errors?.[0]?.reason || '...')` — semua
        // error lewat catch yang sama.
        return Err(());
    }
    let linode_id = js_str(data.get("id"));

    // `await m.reply('🕕 Linode sedang dibuat... Tunggu 60 detik.')`
    // (`linode.js:92`), lalu `setTimeout` 60000 ms (`linode.js:94`).
    let tunggu = Reply::text("🕕 Linode sedang dibuat... Tunggu 60 detik.");
    std::thread::sleep(std::time::Duration::from_secs(60));

    let req = headers(
        HttpCall::get(format!("https://api.linode.com/v4/linode/instances/{linode_id}")),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let info = res.json().ok_or(())?;
    // `linodeInfo.ipv4?.[0] || 'Pending'`.
    let ip = {
        let s = js_str(info.get("ipv4").and_then(Value::as_array).and_then(|a| a.first()));
        if s.is_empty() { "Pending".to_string() } else { s }
    };

    let msg = format!(
        "✅ *ʟɪɴᴏᴅᴇ {} ʙᴇʀʜᴀsɪʟ ᴅɪʙᴜᴀᴛ*\n\n\
         > 🆔 ID: `{linode_id}`\n\
         > 🏷️ Label: `{label}`\n\
         > 🌐 IP: `{ip}`\n\
         > 🔑 Password: `{root_pass}`\n\
         > 💾 RAM: {}\n\
         > 📍 Region: ap-south",
        spec.label,
        spec.ram,
    );

    // Urutan JS: react 🚀 → reply 🕕 → (tunggu) → reply final → react ✅.
    Ok(Reply::sequence(vec![
        Reply::reaction("🚀"),
        tunggu,
        Reply::text(msg),
        Reply::reaction("✅"),
    ]))
}

/// `listlinode` (`linode.js:120-150`).
fn listlinode(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let req = headers(
        HttpCall::get("https://api.linode.com/v4/linode/instances"),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let data = res.json().ok_or(())?;
    if !res_ok(res.status) {
        return Err(());
    }

    let kosong = data
        .get("data")
        .and_then(Value::as_array)
        .map(|a| a.is_empty())
        .unwrap_or(true);
    if kosong {
        return Ok(Reply::text("📋 *ᴅᴀғᴛᴀʀ ʟɪɴᴏᴅᴇ*\n\n> Tidak ada VPS aktif."));
    }

    let mut msg = String::from("📋 *ᴅᴀғᴛᴀʀ ʟɪɴᴏᴅᴇ ᴠᴘs*\n\n");
    for (i, l) in data
        .get("data")
        .and_then(Value::as_array)
        .map(|a| a.as_slice())
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        msg.push_str(&format!("*{}. {}*\n", i + 1, js_str(l.get("label"))));
        msg.push_str(&format!("> ID: `{}`\n", js_str(l.get("id"))));
        msg.push_str(&format!("> IP: `{}`\n", ipv4_pertama(l)));
        msg.push_str(&format!("> Status: {}\n\n", js_str(l.get("status"))));
    }
    // `m.reply(msg.trim())` — trim sebelum kirim.
    Ok(Reply::sequence(vec![
        Reply::text(msg.trim().to_string()),
        Reply::reaction("✅"),
    ]))
}

/// `onlinode` / `offlinode` / `rebootlinode` — POST ke sub-resource aksi
/// (`linode.js:152-253`). `m.react(emoji)` mendahului request; urutan kirim
/// tetap dijaga lewat `Sequence`.
fn aksi_id(ctx: &Ctx, token: &str, aksi: &str, emoji: &str, sukses: &str) -> Result<Reply, ()> {
    let Some(id) = ctx.args.first() else {
        return Ok(Reply::text(minta_id(&ctx.prefix, &ctx.command, "12345")));
    };
    let id = id.clone();

    let req = headers(
        HttpCall::post(format!(
            "https://api.linode.com/v4/linode/instances/{id}/{aksi}"
        )),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    if !res_ok(res.status) {
        return Err(());
    }
    Ok(Reply::sequence(vec![
        Reply::reaction(emoji),
        Reply::text(format!("✅ Linode ID `{id}` {sukses}")),
        Reply::reaction("✅"),
    ]))
}

/// `rebuildlinode` (`linode.js:224-253`).
fn rebuild(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let Some(id) = ctx.args.first() else {
        return Ok(Reply::text(minta_id(
            &ctx.prefix,
            "rebuildlinode",
            "12345 linode/ubuntu20.04",
        )));
    };
    let id = id.clone();
    // `const image = args[1] || 'linode/ubuntu20.04'`.
    let image = ctx
        .args
        .get(1)
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_else(|| "linode/ubuntu20.04".to_string());
    let root_pass = format!("{}{}", random_karakter(4), random_nomor(3));

    let body = serde_json::json!({ "image": image, "root_pass": root_pass });
    let req = headers(
        HttpCall::post(format!(
            "https://api.linode.com/v4/linode/instances/{id}/rebuild"
        )),
        token,
    )
    .json(&body);
    let res = ctx.http.call(&req).map_err(|_| ())?;
    if !res_ok(res.status) {
        return Err(());
    }
    Ok(Reply::sequence(vec![
        Reply::reaction("🔧"),
        Reply::text(format!(
            "✅ Linode ID `{id}` berhasil di-rebuild!\n\n> 🔑 Password baru: `{root_pass}`\n> 🖼️ Image: {image}"
        )),
        Reply::reaction("✅"),
    ]))
}

/// `delinode` — DELETE (`linode.js:255-277`).
fn delinode(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let Some(id) = ctx.args.first() else {
        return Ok(Reply::text(minta_id(&ctx.prefix, "delinode", "12345")));
    };
    let id = id.clone();

    let req = headers(
        HttpCall::new(
            HttpMethod::Delete,
            format!("https://api.linode.com/v4/linode/instances/{id}"),
        ),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    if !res_ok(res.status) {
        return Err(());
    }
    Ok(Reply::sequence(vec![
        Reply::reaction("🗑️"),
        Reply::text(format!("✅ Linode ID `{id}` berhasil dihapus!")),
        Reply::reaction("✅"),
    ]))
}

/// `saldolinode` (`linode.js:279-304`) — `data.koin` dan
/// `data.credit_remaining` dibagi 100, `toFixed(2)`.
fn saldo(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let req = headers(
        HttpCall::get("https://api.linode.com/v4/account"),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let data = res.json().ok_or(())?;
    if !res_ok(res.status) {
        return Err(());
    }

    let koin = data.get("koin").and_then(Value::as_f64).unwrap_or(0.0);
    let kredit = data
        .get("credit_remaining")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let msg = format!(
        "💰 *sᴀʟᴅᴏ ᴀᴋᴜɴ ʟɪɴᴏᴅᴇ*\n\n> 💵 Balance: ${:.2}\n> 🎁 Credit: ${:.2}",
        koin / 100.0,
        kredit / 100.0
    );
    Ok(Reply::sequence(vec![
        Reply::reaction("💰"),
        Reply::text(msg),
        Reply::reaction("✅"),
    ]))
}

/// `sisalinode` (`linode.js:306-325`).
fn sisalinode(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let req = headers(
        HttpCall::get("https://api.linode.com/v4/linode/instances"),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let data = res.json().ok_or(())?;
    if !res_ok(res.status) {
        return Err(());
    }
    let total = data
        .get("data")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    Ok(Reply::sequence(vec![
        Reply::reaction("📊"),
        Reply::text(format!("📊 *ᴛᴏᴛᴀʟ ʟɪɴᴏᴅᴇ ᴀᴋᴛɪғ*\n\n> {total} VPS")),
        Reply::reaction("✅"),
    ]))
}

/// `cekvpslinode` (`linode.js:327-356`).
fn cekvpslinode(ctx: &Ctx, token: &str) -> Result<Reply, ()> {
    let Some(id) = ctx.args.first() else {
        return Ok(Reply::text(minta_id(&ctx.prefix, "cekvpslinode", "12345")));
    };
    let id = id.clone();

    let req = headers(
        HttpCall::get(format!("https://api.linode.com/v4/linode/instances/{id}")),
        token,
    );
    let res = ctx.http.call(&req).map_err(|_| ())?;
    let l = res.json().ok_or(())?;
    if !res_ok(res.status) {
        return Err(());
    }

    // `${l.ipv4?.join(', ') || '-'}` — array kosong ikut jatuh ke `'-'`.
    let ip = {
        let s = js_str(l.get("ipv4"));
        if s.is_empty() { "-".to_string() } else { s }
    };
    let msg = format!(
        "🔍 *ᴅᴇᴛᴀɪʟ ʟɪɴᴏᴅᴇ*\n\n\
         > 🆔 ID: `{}`\n\
         > 🏷️ Label: `{}`\n\
         > 📊 Status: {}\n\
         > 📍 Region: {}\n\
         > 💾 Type: {}\n\
         > 🌐 IP: `{}`",
        js_str(l.get("id")),
        js_str(l.get("label")),
        js_str(l.get("status")),
        js_str(l.get("region")),
        js_str(l.get("type")),
        ip,
    );
    Ok(Reply::sequence(vec![
        Reply::reaction("🔍"),
        Reply::text(msg),
        Reply::reaction("✅"),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::find;
    use std::sync::Arc;

    fn ctx(command: &str, args: &[&str], token: &str) -> Ctx {
        let config = oktz_plugin_api::config::Runtime {
            api_keys: oktz_plugin_api::config::ApiKeys {
                linode: token.to_string(),
                ..oktz_plugin_api::config::ApiKeys::default()
            },
            ..oktz_plugin_api::config::Runtime::default()
        };
        Ctx {
            prefix: ".".into(),
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            config: Arc::new(config),
            ..Default::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations()
            .text_body()
            .expect("harus ada teks")
            .to_string()
    }

    #[test]
    fn semua_13_kunci_js_terdaftar() {
        for kunci in [
            "linode2gb", "linode4gb", "linode8gb", "linode16gb", "listlinode",
            "onlinode", "offlinode", "rebootlinode", "rebuildlinode", "delinode",
            "saldolinode", "sisalinode", "cekvpslinode",
        ] {
            assert_eq!(
                find(kunci).unwrap_or_else(|| panic!("{kunci} tidak terdaftar")).meta.name,
                "linode2gb",
                "kunci {kunci} harus resolve ke plugin linode"
            );
        }
        let m = &find("linode2gb").unwrap().meta;
        assert_eq!(m.category, "linode", "kategori JS: 'linode'");
        assert!(m.alias.is_empty(), "alias JS: []");
        assert_eq!(m.cooldown, 10);
        assert_eq!(m.energi, 0);
        assert!(m.is_owner);
        assert!(!m.is_group && !m.is_private);
    }

    #[test]
    fn random_mengikuti_alphabet_js() {
        // `randomKarakter(5) + randomNomor(3)` — 8 karakter, tanpa 0/1/I/O/l.
        for _ in 0..50 {
            let pass = format!("{}{}", random_karakter(5), random_nomor(3));
            assert_eq!(pass.len(), 8, "{pass}");
            assert!(
                pass.chars().all(|c| KARAKTER.contains(&(c as u32 as u8))
                    || NOMOR.contains(&(c as u32 as u8))),
                "{pass} memuat karakter di luar alphabet JS"
            );
        }
        // randomNomor hanya angka '2'..'9'.
        for _ in 0..50 {
            let n = random_nomor(3);
            assert!(n.chars().all(|c| NOMOR.contains(&(c as u32 as u8))), "{n}");
        }
    }

    #[test]
    fn token_kosong_mengembalikan_pesan_konfigurasi() {
        let r = linode(ctx("listlinode", &[], ""));
        let t = teks(&r);
        assert!(t.starts_with("❌ Linode API Token tidak dikonfigurasi!"), "{t}");
        assert!(t.contains("linode: 'YOUR_LINODE_TOKEN'"), "{t}");
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn http_gagal_jatuh_ke_catch_reaksi_radioaktif() {
        // NoHttp (default) selalu gagal — padanan `fetch` melempar di JS.
        let c = ctx("listlinode", &[], "token-test");
        let r = linode(c);
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "☢"
        ));
        let t = teks(&r);
        assert!(
            t.contains("Kayaknya command"),
            "harus pakai te() template: {t}"
        );
    }

    #[test]
    fn create_tanpa_label_minta_label() {
        let c = ctx("linode2gb", &[], "token-test");
        let r = linode(c);
        assert_eq!(
            teks(&r),
            "❌ Masukkan label untuk VPS!\n\nContoh: .linode2gb myserver"
        );
    }

    #[test]
    fn aksi_tanpa_id_minta_id_dengan_contoh_per_command() {
        for (cmd, contoh) in [
            ("onlinode", ".onlinode 12345"),
            ("offlinode", ".offlinode 12345"),
            ("rebootlinode", ".rebootlinode 12345"),
            ("delinode", ".delinode 12345"),
            ("cekvpslinode", ".cekvpslinode 12345"),
            (
                "rebuildlinode",
                ".rebuildlinode 12345 linode/ubuntu20.04",
            ),
        ] {
            let c = ctx(cmd, &[], "token-test");
            let r = linode(c);
            assert_eq!(
                teks(&r),
                format!("❌ Masukkan ID Linode!\n\nContoh: {contoh}"),
                "{cmd}"
            );
        }
    }

    #[test]
    fn ipv4_kosong_jatuh_ke_strip_atau_pending() {
        assert_eq!(ipv4_pertama(&serde_json::json!({})), "-");
        assert_eq!(ipv4_pertama(&serde_json::json!({ "ipv4": [] })), "-");
        assert_eq!(
            ipv4_pertama(&serde_json::json!({ "ipv4": ["1.2.3.4", "5.6.7.8"] })),
            "1.2.3.4"
        );
    }

    #[test]
    fn js_str_mengikuti_semantik_template_literal() {
        assert_eq!(js_str(None), "undefined");
        assert_eq!(js_str(Some(&Value::Null)), "undefined");
        assert_eq!(js_str(Some(&serde_json::json!(12345))), "12345");
        assert_eq!(js_str(Some(&serde_json::json!(["a", "b"]))), "a,b");
        assert_eq!(js_str(Some(&serde_json::json!({}))), "[object Object]");
    }

    #[test]
    fn command_tak_dikenal_menampilkan_menu_bantuan() {
        // Jalur `else` terakhir JS — tidak terjangkau lewat registry (semua
        // nama terdaftar sebagai command), diuji langsung lewat dispatch.
        let c = ctx("bukanlinode", &[], "token-test");
        match kerjakan(&c, "token-test") {
            Ok(r) => {
                let t = teks(&r);
                assert!(t.starts_with("☁️ *ʟɪɴᴏᴅᴇ ᴄᴏᴍᴍᴀɴᴅs*"), "{t}");
                assert!(t.contains(".linode16gb <label> - Buat VPS 16GB"), "{t}");
            }
            other => panic!("harus Ok, dapat {other:?}"),
        }
        assert!(menu_bantuan().contains(".linode16gb <label> - Buat VPS 16GB"));
    }
}
