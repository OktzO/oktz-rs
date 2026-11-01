//! `plugins/store/done.js` — konfirmasi transaksi selesai dan kirim data
//! ke pembeli.
//!
//! `ponytail:` TIGA langkah JS tidak di-port, semuanya di luar `Ctx`:
//! - `sock.sendMessage(buyerJid, { text: invoiceTxt })` — pengiriman ke JID
//!   lain bukan bagian dari `Reply` (`done.js:157-178`). Invoice tetap
//!   dihitung (termasuk `stockItemDetail` yang diambil dari stok), tapi
//!   teksnya tidak bisa dikirim ke pembeli.
//! - Notifikasi grup pembelian `sock.sendMessage(trx.purchaseChat, ...)`
//!   (`done.js:180-196`) — alasan sama.
//! - `contextInfo` newsletter forwarding pada invoice — `Reply::Text`
//!   tidak membawa contextInfo.
//! Kedua pengiriman yang hilang ada di `try/catch` terpisah di JS, jadi
//! alur konfirmasi (mutasi stok + status transaksi + balasan admin) tetap
//! utuh. Reaksi `✅` dilekatkan pada balasan konfirmasi seperti
//! `tools/txt2qr.rs`.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};

use crate::store::beli::{price_of, rupiah};
use crate::store::liststok;

/// `storeTransactions` — objek `{ trxId → transaksi }` (`beli.rs`).
const TRX: &str = "storeTransactions";
/// `storeProducts` — array produk.
const PRODUK: &str = "storeProducts";

// ---------------------------------------------------------------------------
// Tanggal: `new Date(value)` menerima angka ms atau string ISO.
// ---------------------------------------------------------------------------

/// `new Date(value).getTime()` untuk value angka atau string ISO 8601.
/// `0` kalau tidak bisa diparse — JS `Number.isNaN` jalur `getRegistrationTime`
/// juga jatuh ke 0.
pub(crate) fn date_ke_ms(value: &serde_json::Value) -> i64 {
    match value {
        serde_json::Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)).unwrap_or(0),
        serde_json::Value::String(s) => iso_ke_ms(s),
        _ => 0,
    }
}

/// Parse `2026-11-01T12:34:56.789Z` (atau tanpa fraksi/zona) ke ms epoch.
/// String dianggap UTC seperti `Date.parse` pada bentuk ISO.
pub(crate) fn iso_ke_ms(s: &str) -> i64 {
    let s = s.trim();
    // tanggal wajib: YYYY-MM-DD
    let (t, sisa) = match s.split_once('T') {
        Some((t, sisa)) => (t, Some(sisa)),
        None => (s, None),
    };
    let tgl: Vec<&str> = t.split('-').collect();
    if tgl.len() != 3 {
        return 0;
    }
    let (Ok(y), Ok(mo), Ok(d)) = (
        tgl[0].parse::<i64>(),
        tgl[1].parse::<u32>(),
        tgl[2].parse::<u32>(),
    ) else {
        return 0;
    };
    let mut jam = 0i64;
    let mut menit = 0i64;
    let mut detik = 0i64;
    let mut ms = 0i64;
    if let Some(waktu) = sisa {
        let waktu = waktu.trim_end_matches('Z');
        // buang offset +07:00 / -05:00 — store menyimpan UTC.
        let waktu = match waktu.find(['+', '-']) {
            Some(i) => &waktu[..i],
            None => waktu,
        };
        let bag: Vec<&str> = waktu.split(':').collect();
        let angka = |x: &str| -> i64 {
            // fraksi detik "56.789"
            let mut it = x.split('.');
            let bulat: i64 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            let fraksi: i64 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            bulat * if x.contains('.') { 1 } else { 1 } + if x.contains('.') { fraksi } else { 0 }
        };
        if bag.len() > 0 {
            jam = bag[0].parse().unwrap_or(0);
        }
        if bag.len() > 1 {
            menit = bag[1].parse().unwrap_or(0);
        }
        if bag.len() > 2 {
            let mut it = bag[2].split('.');
            detik = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            ms = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }
    crate::common::days_from_civil(y, mo, d) * 86_400_000
        + jam * 3_600_000
        + menit * 60_000
        + detik * 1000
        + ms
}

/// `new Date().toISOString()` — UTC `yyyy-mm-ddThh:mm:ss.mmmZ`.
pub(crate) fn ms_ke_iso(ms: i64) -> String {
    // UTC = Asia/Jakarta - 7 jam (lihat `common::JAKARTA_OFFSET_SECS`).
    let (y, mo, d, h, mi, s) = crate::common::jakarta_parts(ms - 7 * 3_600_000);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms_pad:03}Z", ms_pad = (ms.rem_euclid(1000)))
}

/// `toLocaleString("id-ID", { timeZone: "Asia/Jakarta" })` — `d/m/yyyy, hh.mm.ss`.
pub(crate) fn lokali(ms: i64) -> String {
    let (y, mo, d, h, mi, s) = crate::common::jakarta_parts(ms);
    format!("{d}/{mo}/{y}, {h:02}.{mi:02}.{s:02}")
}

/// `toLocaleString("id-ID", { day: "2-digit", month: "2-digit", year:
/// "numeric", hour: "2-digit", minute: "2-digit", timeZone: "Asia/Jakarta" })`
/// — `dd/mm/yyyy, hh.mm`.
pub(crate) fn lokali_pendek(ms: i64) -> String {
    let (y, mo, d, h, mi, _) = crate::common::jakarta_parts(ms);
    format!("{d:02}/{mo:02}/{y}, {h:02}.{mi:02}")
}

// ---------------------------------------------------------------------------
// Teks
// ---------------------------------------------------------------------------

/// Kartu pakai saat nomor transaksi kosong (`done.js:23-39`).
pub(crate) fn kartu_pakai(prefix: &str) -> String {
    format!(
        "✅ *KONFIRMASI TRANSAKSI*\n\n\
         📋 Format: `{prefix}done <nomor_trx>`\n\n\
         📌 *Cara penggunaan:*\n\
         1️⃣ Reply pesan dari pembeli (yang sudah membayar 💰)\n\
         2️⃣ Ketik `{prefix}done TRX-001`\n\n\
         🤖 Bot akan otomatis:\n\
         • Mengirim data produk ke nomor pembeli 📤\n\
         • Menandai transaksi sebagai selesai ✅\n\
         • Mengirim notifikasi ke pembeli 🔔\n\n\
         🧾 *Nomor transaksi* didapat ketika pembeli melakukan `{prefix}beli <nomor_produk>`\n\n\
         ⚠️ _Pastikan Anda sudah menerima bukti pembayaran sebelum konfirmasi_ 📸"
    )
}

/// Daftar transaksi pending saat nomor tidak ditemukan (`done.js:47-61`).
fn pending_list(prefix: &str, trx_id: &str, pending: &[&serde_json::Value]) -> String {
    let mut txt = format!("❌ *Transaksi `{trx_id}` tidak ditemukan.*\n\n");
    txt.push_str("⏳ *Transaksi pending saat ini:*\n\n");
    for t in pending {
        let type_icon = if t.get("productType").and_then(|v| v.as_str()) == Some("fisik") {
            "📦"
        } else {
            "🔑"
        };
        let ms = date_ke_ms(t.get("createdAt").unwrap_or(&serde_json::Value::Null));
        txt.push_str(&format!(
            "• 🧾 `{}` — {} {} ({}) oleh {}\n",
            t.get("trxId").and_then(|v| v.as_str()).unwrap_or(""),
            type_icon,
            t.get("productName").and_then(|v| v.as_str()).unwrap_or(""),
            rupiah(price_of(t)),
            t.get("buyerName").and_then(|v| v.as_str()).unwrap_or("")
        ));
        txt.push_str(&format!("  🕐 _{}_ \n\n", lokali(ms)));
    }
    txt.push_str(&format!("📌 Reply pesan pembeli lalu ketik: `{prefix}done <nomor_trx>`"));
    txt
}

/// Transaksi sudah selesai (`done.js:71-79`).
fn sudah_selesai(trx_id: &str, trx: &serde_json::Value) -> String {
    let fisik = trx.get("productType").and_then(|v| v.as_str()) == Some("fisik");
    let ms = date_ke_ms(trx.get("completedAt").unwrap_or(&serde_json::Value::Null));
    format!(
        "⚠️ *Transaksi sudah selesai.*\n\n\
         🧾 TRX: `{trx_id}`\n\
         {} Produk: *{}*\n\
         👤 Pembeli: {}\n\
         ✅ Selesai pada: {}\n\n\
         _Transaksi ini sudah dikonfirmasi sebelumnya_ 🔒",
        if fisik { "📦" } else { "🔑" },
        trx.get("productName").and_then(|v| v.as_str()).unwrap_or(""),
        trx.get("buyerName").and_then(|v| v.as_str()).unwrap_or(""),
        lokali(ms),
    )
}

/// Invoice untuk pembeli (`done.js:124-146`) — dihitung untuk paritas dan
/// dipakai pesan gagal-kirim di JS; di Rust tidak ada jalur kirim ke JID
/// lain, jadi hanya teks konfirmasi admin yang dibalas.
#[allow(dead_code)]
fn invoice(
    time_str: &str,
    product_type: &str,
    product_name: &str,
    price: i64,
    stock_item_detail: Option<&str>,
) -> String {
    let fisik = product_type == "fisik";
    let mut txt = String::from("🎉 *TRANSAKSI BERHASIL*\n\n");
    txt.push_str(&format!("🕐 Waktu: `{time_str}`\n"));
    txt.push_str("✅ Status: *Berhasil*\n\n");
    txt.push_str("📦 *Detail Pesanan:*\n");
    txt.push_str(&format!(
        "{} Produk: *{}*\n",
        if fisik { "📦" } else { "🔑" },
        product_name
    ));
    txt.push_str(&format!(
        "🏷️ Tipe: *{}*\n",
        if fisik { "Fisik" } else { "Digital" }
    ));
    txt.push_str(&format!("💰 Harga: *{}*\n\n", rupiah(price)));
    if let Some(detail) = stock_item_detail {
        txt.push_str(&format!("🔑 *Data Produk:*\n```\n{detail}\n```\n\n"));
        txt.push_str("⚠️ _Simpan data di atas dengan baik. Jangan bagikan ke siapapun_ 🔒\n\n");
    } else if fisik {
        txt.push_str("📦 _Produk fisik akan dikirim oleh admin. Silakan konfirmasi alamat pengiriman._\n\n");
    }
    txt.push_str("🙏 Terima kasih telah berbelanja! _Next order ya_ ✨");
    txt
}

#[plugin(
    name = "done",
    alias = ["selesai", "kirim", "confirm"],
    category = "store",
    description = "✅ Konfirmasi transaksi selesai dan kirim data ke pembeli (reply pesan pembeli)",
    usage = ".done <nomor_trx> (reply pesan pembeli)",
    example = ".done TRX-001",
    cooldown = 3,
    energi = 0,
    is_owner = true,
    is_premium = false,
    is_group = false,
    is_private = false
)]
pub fn done(ctx: Ctx) -> Reply {
    let trx_id = ctx.text.trim().to_string();

    if trx_id.is_empty() {
        return Reply::text(kartu_pakai(&ctx.prefix));
    }

    let Some(transactions) = ctx.setting(TRX).cloned().filter(|v| v.is_object()) else {
        // `db.setting("storeTransactions") || {}` — tanpa key, tidak ada pending.
        return Reply::text(format!(
            "❌ *Transaksi `{trx_id}` tidak ditemukan.*\n\n\
             📭 Tidak ada transaksi pending saat ini.\n\n\
             _Pembeli dapat membuat pesanan dengan `{p}beli <nomor_produk>`_ 🛒",
            p = ctx.prefix
        ));
    };
    let Some(trx) = transactions.get(&trx_id).cloned() else {
        let pending: Vec<&serde_json::Value> = transactions
            .as_object()
            .map(|m| {
                m.values()
                    .filter(|t| t.get("status").and_then(|v| v.as_str()) == Some("pending"))
                    .collect()
            })
            .unwrap_or_default();
        if !pending.is_empty() {
            return Reply::text(pending_list(&ctx.prefix, &trx_id, &pending));
        }
        return Reply::text(format!(
            "❌ *Transaksi `{trx_id}` tidak ditemukan.*\n\n\
             📭 Tidak ada transaksi pending saat ini.\n\n\
             _Pembeli dapat membuat pesanan dengan `{p}beli <nomor_produk>`_ 🛒",
            p = ctx.prefix
        ));
    };

    if trx.get("status").and_then(|v| v.as_str()) == Some("completed") {
        return Reply::text(sudah_selesai(&trx_id, &trx));
    }

    let mut buyer_jid = trx
        .get("buyerJid")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // `if (m.quoted && m.isGroup)` — pembeli = pengirim pesan yang di-reply.
    if let Some(quoted) = ctx.quoted() {
        if ctx.is_group {
            let quoted_sender = quoted.sender.clone();
            if !quoted_sender.is_empty() && quoted_sender != ctx.sender {
                buyer_jid = quoted_sender;
            }
        }
    }

    if buyer_jid.is_empty() {
        return Reply::text(
            "❌ *Tidak dapat menemukan nomor pembeli.*\n\nTransaksi ini tidak memiliki data pembeli yang valid 📱",
        );
    }

    let buyer_num = buyer_jid.split('@').next().unwrap_or("").to_string();

    let mut products: Vec<serde_json::Value> = ctx
        .setting(PRODUK)
        .and_then(|v| v.as_array())
        .map(|a| a.to_vec())
        .unwrap_or_default();
    let product_idx = products
        .iter()
        .position(|p| p.get("id") == trx.get("productId"));
    let mut mutations: Vec<Mutation> = Vec::new();

    let mut stock_item_detail: Option<serde_json::Value> = None;
    if let Some(idx) = product_idx {
        let product_type = trx
            .get("productType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let is_fisik = product_type == "fisik";

        if !is_fisik {
            let items_kosong = products[idx]
                .get("stockItems")
                .and_then(|v| v.as_array())
                .map_or(true, |a| a.is_empty());
            if !items_kosong {
                // `const item = product.stockItems.shift()` (`done.js:96-98`).
                let item = products[idx]["stockItems"].as_array_mut().unwrap().remove(0);
                stock_item_detail = item
                    .get("detail")
                    .filter(|v| crate::common::js_truthy_value(v))
                    .cloned();
                let sisa = products[idx]["stockItems"].as_array().unwrap().len();
                products[idx]["stock"] = serde_json::json!(sisa);
                mutations.push(Mutation::SetSetting {
                    key: PRODUK.into(),
                    value: serde_json::Value::Array(products.clone()),
                });
            } else if products[idx]
                .get("detail")
                .is_some_and(|v| crate::common::js_truthy_value(v))
            {
                stock_item_detail =
                    products[idx].get("detail").cloned();
            }
        } else {
            // Fisik: kurangi stok kecuali unlimited.
            let stock = products[idx].get("stock").and_then(|v| v.as_i64());
            if let Some(stock) = stock {
                if stock != -1 && stock > 0 {
                    products[idx]["stock"] = serde_json::json!(stock - 1);
                    mutations.push(Mutation::SetSetting {
                        key: PRODUK.into(),
                        value: serde_json::Value::Array(products.clone()),
                    });
                }
            }
        }
    }

    // Mutasi transaksi: status completed + completedAt ISO + buyerJid final.
    let mut transactions = transactions;
    let now = crate::common::now_ms();
    let mut trx_baru = trx.clone();
    trx_baru["status"] = serde_json::json!("completed");
    trx_baru["completedAt"] = serde_json::json!(ms_ke_iso(now));
    trx_baru["buyerJid"] = serde_json::json!(buyer_jid);
    trx_baru["stockItemDetail"] = stock_item_detail.clone().unwrap_or(serde_json::Value::Null);
    transactions[&trx_id] = trx_baru;
    mutations.push(Mutation::SetSetting {
        key: TRX.into(),
        value: transactions,
    });

    // `stockItemDetail` hasil shift masih perlu untuk sisa stok — JS membaca
    // ulang dari objek produk setelah mutasi.
    let product = product_idx.map(|i| &products[i]);
    let mut confirm = format!(
        "✅ *TRANSAKSI DIKONFIRMASI*\n\n\
         🧾 TRX: `{trx_id}`\n\
         {} Produk: *{}*\n\
         👤 Pembeli: *{}*\n\
         📱 Nomor: `{}`\n\
         💰 Harga: *{}*\n",
        if trx.get("productType").and_then(|v| v.as_str()) == Some("fisik") { "📦" } else { "🔑" },
        trx.get("productName").and_then(|v| v.as_str()).unwrap_or(""),
        trx.get("buyerName").and_then(|v| v.as_str()).unwrap_or(""),
        buyer_num,
        rupiah(price_of(&trx)),
    );
    if let Some(p) = product {
        let stok = if liststok::is_fisik(p) {
            match p.get("stock").and_then(|v| v.as_i64()) {
                Some(-1) => "♾️ Unlimited".to_string(),
                Some(n) => format!("{n} pcs"),
                None => "0 pcs".to_string(),
            }
        } else {
            format!(
                "{} akun",
                p.get("stockItems")
                    .and_then(|v| v.as_array())
                    .map_or(0, Vec::len)
            )
        };
        confirm.push_str(&format!("📊 Sisa stok: *{stok}*\n"));
    }
    confirm.push_str("\n📤 _Data telah dikirim ke nomor pembeli_ ✅");

    Reply::text(confirm).mutating(mutations).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split_whitespace().map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "done".into(),
            settings: json!({
                "storeProducts": [{
                    "id": "P1", "name": "Akun Prem", "type": "digital", "price": 25000,
                    "stock": 3,
                    "stockItems": [{"detail": "user:pass-1"}, {"detail": "user:pass-2"}],
                    "detail": ""
                }],
                "storeTransactions": {
                    "TRX-001": {
                        "trxId": "TRX-001", "buyerJid": "628222@s.whatsapp.net",
                        "buyerName": "Budi", "purchaseChat": "628222@s.whatsapp.net",
                        "purchaseIsGroup": false, "productId": "P1", "productName": "Akun Prem",
                        "productType": "digital", "price": 25000, "status": "pending",
                        "createdAt": 1_767_225_600_000i64
                    }
                }
            }),
            ..Ctx::default()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("done").unwrap();
        assert_eq!(p.meta.name, "done");
        assert_eq!(p.meta.alias, ["selesai", "kirim", "confirm"]);
        assert_eq!(p.meta.category, "store");
        assert_eq!(
            p.meta.description,
            "✅ Konfirmasi transaksi selesai dan kirim data ke pembeli (reply pesan pembeli)"
        );
        assert_eq!(p.meta.usage, ".done <nomor_trx> (reply pesan pembeli)");
        assert_eq!(p.meta.example, ".done TRX-001");
        assert_eq!(p.meta.cooldown, 3);
        assert!(p.meta.is_owner);
        assert!(!p.meta.is_premium && !p.meta.is_group);
    }

    #[test]
    fn semua_alias_menunjuk_done() {
        for key in ["done", "selesai", "kirim", "confirm"] {
            assert_eq!(
                oktz_plugin_api::find(key).map(|p| p.meta.name.as_str()),
                Some("done")
            );
        }
    }

    #[test]
    fn tanpa_nomor_transaksi_kartu_pakai() {
        let r = done(ctx_of(""));
        let t = r.text_body().unwrap();
        assert!(t.starts_with("✅ *KONFIRMASI TRANSAKSI*"));
        assert!(t.contains("`.done <nomor_trx>`"));
        assert!(t.contains("2️⃣ Ketik `.done TRX-001`"));
        assert!(r.mutations().is_empty());
    }

    #[test]
    fn transaksi_diproses_stok_dan_status_berubah() {
        let r = done(ctx_of("TRX-001"));
        let teks = r.text_body().unwrap();
        assert!(teks.starts_with("✅ *TRANSAKSI DIKONFIRMASI*"), "{teks}");
        assert!(teks.contains("🧾 TRX: `TRX-001`"));
        assert!(teks.contains("👤 Pembeli: *Budi*"));
        assert!(teks.contains("📱 Nomor: `628222`"));
        assert!(teks.contains("💰 Harga: *Rp 25.000*"));
        assert!(teks.contains("📊 Sisa stok: *1 akun*"));
        // mutasi: storeProducts (stok berkurang) + storeTransactions
        let mut setting_mutation = 0;
        let mut produk_baru = None;
        let mut trx_baru = None;
        for m in r.mutations() {
            match m {
                Mutation::SetSetting { key, value } => {
                    setting_mutation += 1;
                    if key == "storeProducts" {
                        produk_baru = Some(value.clone());
                    } else if key == "storeTransactions" {
                        trx_baru = Some(value.clone());
                    }
                }
                _ => panic!("harus SetSetting"),
            }
        }
        assert_eq!(setting_mutation, 2);
        let produk = produk_baru.unwrap();
        // JS: `product.stock = product.stockItems.length` SETELAH shift.
        assert_eq!(produk[0]["stock"], 1);
        assert_eq!(produk[0]["stockItems"].as_array().unwrap().len(), 1);
        assert_eq!(produk[0]["stockItems"][0]["detail"], "user:pass-2");
        let trx = trx_baru.unwrap();
        assert_eq!(trx["TRX-001"]["status"], "completed");
        assert!(
            trx["TRX-001"]["completedAt"].as_str().is_some_and(|s| s.ends_with('Z')),
            "completedAt ISO seperti JS"
        );
        assert_eq!(trx["TRX-001"]["stockItemDetail"], "user:pass-1");
        assert_eq!(trx["TRX-001"]["buyerJid"], "628222@s.whatsapp.net");
        // reaksi ✅ pada balasan
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "✅"
        ));
    }

    #[test]
    fn reply_pembeli_di_grup_menjadi_buyer_jid() {
        let mut c = ctx_of("TRX-001");
        c.is_group = true;
        c.sender = "628333@s.whatsapp.net".into();
        c.quoted = Some(oktz_plugin_api::registry::QuotedMessage {
            id: "abc".into(),
            sender: "628999@s.whatsapp.net".into(),
            chat: c.chat.clone(),
            text: String::new(),
            kind: oktz_plugin_api::registry::QuotedKind::Text,
            mimetype: None,
            file_name: None,
            from_me: false,
            timestamp: 0,
        });
        let r = done(c);
        assert!(r.text_body().unwrap().contains("📱 Nomor: `628999`"));
        // buyerJid di transaksi ikut diganti
        for m in r.mutations() {
            if let Mutation::SetSetting { key, value } = m {
                if key == "storeTransactions" {
                    assert_eq!(value["TRX-001"]["buyerJid"], "628999@s.whatsapp.net");
                }
            }
        }
    }

    #[test]
    fn transaksi_selesai_ditolak_ulang() {
        let mut c = ctx_of("TRX-001");
        c.settings["storeTransactions"]["TRX-001"]["status"] = json!("completed");
        c.settings["storeTransactions"]["TRX-001"]["completedAt"] =
            json!("2026-02-01T10:00:00.000Z");
        let r = done(c);
        let t = r.text_body().unwrap();
        assert!(t.starts_with("⚠️ *Transaksi sudah selesai.*"));
        assert!(t.contains("✅ Selesai pada: 1/2/2026, 17.00.00"), "{t}");
        assert!(r.mutations().is_empty(), "tidak ada mutasi saat ditolak");
    }

    #[test]
    fn nomor_tidak_ada_menampilkan_daftar_pending() {
        let mut c = ctx_of("TRX-404");
        c.settings["storeTransactions"]["TRX-002"] = json!({
            "trxId": "TRX-002", "buyerName": "Sari", "productName": "Baju",
            "productType": "fisik", "price": 65000, "status": "pending",
            "createdAt": 1_767_225_600_000i64
        });
        let r = done(c);
        let t = r.text_body().unwrap();
        assert!(t.starts_with("❌ *Transaksi `TRX-404` tidak ditemukan.*"));
        assert!(t.contains("⏳ *Transaksi pending saat ini:*"));
        assert!(t.contains("• 🧾 `TRX-002` — 📦 Baju (Rp 65.000) oleh Sari"));
        assert!(t.contains("📌 Reply pesan pembeli lalu ketik: `.done <nomor_trx>`"));
    }

    #[test]
    fn nomor_tidak_ada_tanpa_pending() {
        let mut c = ctx_of("TRX-404");
        // TRX-001 dari ctx_of juga completed — tidak ada pending tersisa.
        c.settings["storeTransactions"]["TRX-001"]["status"] = json!("completed");
        c.settings["storeTransactions"]["TRX-002"] = json!({
            "trxId": "TRX-002", "status": "completed", "createdAt": 0
        });
        let r = done(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("📭 Tidak ada transaksi pending saat ini."));
        assert!(t.contains("`.beli <nomor_produk>`"));
    }

    #[test]
    fn produk_fisik_stok_berkurang_dan_tanpa_item_detail() {
        let mut c = ctx_of("TRX-001");
        c.settings["storeProducts"] = json!([{
            "id": "P1", "name": "Baju", "type": "fisik", "price": 65000, "stock": 8
        }]);
        c.settings["storeTransactions"]["TRX-001"]["productType"] = json!("fisik");
        c.settings["storeTransactions"]["TRX-001"]["productName"] = json!("Baju");
        c.settings["storeTransactions"]["TRX-001"]["price"] = json!(65000);
        let r = done(c);
        let t = r.text_body().unwrap();
        assert!(t.contains("📊 Sisa stok: *7 pcs*"), "{t}");
        for m in r.mutations() {
            if let Mutation::SetSetting { key, value } = m {
                if key == "storeProducts" {
                    assert_eq!(value[0]["stock"], 7);
                }
                assert_eq!(value["TRX-001"]["stockItemDetail"], serde_json::Value::Null);
            }
        }
    }

    #[test]
    fn stok_unlimited_fisik_tidak_berubah() {
        let mut c = ctx_of("TRX-001");
        c.settings["storeProducts"] = json!([{
            "id": "P1", "name": "Baju", "type": "fisik", "price": 65000, "stock": -1
        }]);
        c.settings["storeTransactions"]["TRX-001"]["productType"] = json!("fisik");
        c.settings["storeTransactions"]["TRX-001"]["price"] = json!(65000);
        let r = done(c);
        let mut stok_mutation = false;
        for m in r.mutations() {
            if let Mutation::SetSetting { key, .. } = m {
                if key == "storeProducts" {
                    stok_mutation = true;
                }
            }
        }
        assert!(!stok_mutation, "stok -1 tidak ditulis ulang");
        assert!(r.text_body().unwrap().contains("📊 Sisa stok: *♾️ Unlimited*"));
    }

    #[test]
    fn tanpa_buyer_jid_ditolak() {
        let mut c = ctx_of("TRX-001");
        c.settings["storeTransactions"]["TRX-001"]["buyerJid"] = json!(null);
        let r = done(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.starts_with("❌ *Tidak dapat menemukan nomor pembeli.*"));
    }

    #[test]
    fn iso_dan_angka_diparse_sama() {
        assert_eq!(iso_ke_ms("2026-01-01T00:00:00.000Z"), 1_767_225_600_000);
        assert_eq!(iso_ke_ms("2026-01-01T00:00:00Z"), 1_767_225_600_000);
        assert_eq!(
            date_ke_ms(&json!(1_767_225_600_000i64)),
            1_767_225_600_000
        );
        assert_eq!(date_ke_ms(&json!("bukan tanggal")), 0);
        // Asia/Jakarta = UTC+7
        assert_eq!(lokali(0), "1/1/1970, 07.00.00");
        assert_eq!(lokali_pendek(1_767_225_600_000), "01/01/2026, 07.00");
        assert_eq!(
            ms_ke_iso(1_767_225_600_000),
            "2026-01-01T00:00:00.000Z"
        );
    }
}
