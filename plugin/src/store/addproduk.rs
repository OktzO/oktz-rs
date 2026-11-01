//! `plugins/store/addproduk.js` — tambah produk baru ke toko (owner, private).
//!
//! `uploadImage` (`src/lib/uploader.js`) dipakai untuk thumbnail; padanannya
//! ada sekarang lewat `ctx.http` — port `tools/tourl.rs::unggah_pertama`
//! mencoba host `UPLOAD_PROVIDERS` berurutan dan berhenti di URL pertama,
//! persis perilaku `uploadImage`. `Mutation::SetSetting` membawa array
//! `storeProducts` yang sudah ditambah produk baru.
//!
//! `ponytail:` pesan `⏳ _Mengunggah media..._` di JS dikirim SEBELUM unggah;
//! `Reply` hanya satu per handler, jadi pesan itu digabung hilang — unggah
//! berjalan langsung. Teks peringatan gagal unggah memakai pesan generik
//! "semua host upload gagal" karena `HttpGet` tidak membawa pesan error JS.

use oktz_plugin_api::{Ctx, Mutation, Reply, plugin};

/// `storeProducts` — array produk.
const PRODUK: &str = "storeProducts";

/// `parseInt` JS — ambil digit awal, `NaN` → None.
pub(crate) fn js_parse_int(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let (negatif, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok().map(|n| if negatif { -n } else { n })
}

/// Kartu pakai saat format salah (`addproduk.js:32-56`).
pub(crate) fn kartu_pakai(prefix: &str) -> String {
    format!(
        "➕ *TAMBAH PRODUK BARU*\n\n\
         📋 Format:\n\
         `{prefix}addproduk <nama>|<harga>|<tipe>|<stok>|<deskripsi>`\n\n\
         📌 *Parameter:*\n\
         • *nama* — Nama produk (min. 2 karakter)\n\
         • *harga* — Harga dalam Rupiah (min. 1.000)\n\
         • *tipe* — `digital` 🔑 atau `fisik` 📦 (opsional, default: digital)\n\
         • *stok* — Jumlah stok atau `unlimited` (opsional, default: 999)\n\
         • *deskripsi* — Deskripsi singkat (opsional)\n\n\
         🔑 *Digital* = Produk berupa akun/key/data unik per item\n\
         📦 *Fisik* = Produk berupa barang, stok berupa jumlah\n\n\
         📝 *Contoh:*\n\
         `{prefix}addproduk Spotify Premium|25000|digital|10|Akun Premium 1 Bulan`\n\
         `{prefix}addproduk Baju Kaos|65000|fisik|8|Kaos Polos Cotton 30s`\n\
         `{prefix}addproduk Netflix|35000|digital|unlimited|Sharing Account`\n\n\
         🖼️ *Tips:*\n\
         • Kirim gambar/video terlebih dahulu, lalu reply media tersebut dengan command di atas untuk menambahkan thumbnail 📸\n\
         • Untuk produk *digital*, gunakan `{prefix}addstok` setelah produk dibuat untuk menambahkan data akun/key 🔑\n\
         • Untuk produk *fisik*, stok otomatis diatur dari angka yang dimasukkan 📦\n\
         • Harga diskon bisa diatur nanti dengan `{prefix}editproduk` 🏷️"
    )
}

#[plugin(
    name = "addproduk",
    alias = ["addproduct"],
    category = "store",
    description = "➕ Tambah produk baru ke toko (hanya di private chat)",
    usage = ".addproduk <nama>|<harga>|<tipe>|<stok>|<deskripsi>",
    example = ".addproduk Spotify Premium|25000|digital|10|Akun Premium 1 Bulan",
    cooldown = 3,
    energi = 0,
    is_owner = true,
    is_premium = false,
    is_group = false,
    is_private = true
)]
pub fn addproduk(ctx: Ctx) -> Reply {
    if ctx.is_group {
        return Reply::text(format!(
            "🚫 *Akses Ditolak*\n\n\
             Untuk menjaga privasi dan keamanan data produk 🛡️, penambahan produk hanya dapat dilakukan di *private chat*.\n\n\
             Silakan chat bot secara langsung 📱, lalu ketik:\n\
             `{p}addproduk <nama>|<harga>|<tipe>|<stok>|<deskripsi>`",
            p = ctx.prefix
        ));
    }

    let text = ctx.text.trim().to_string();
    let parts: Vec<String> = text.split('|').map(|p| p.trim().to_string()).collect();

    if parts.len() < 2 {
        return Reply::text(kartu_pakai(&ctx.prefix));
    }

    let name = parts[0].clone();
    let price = js_parse_int(&parts[1]);
    let type_str = parts.get(2).map(|s| s.to_lowercase()).unwrap_or_default();
    let stock_str = parts.get(3).cloned().unwrap_or_default();
    let description = parts.get(4).map(|s| s.replace(";;", "\n")).unwrap_or_default();

    if name.is_empty() || name.chars().count() < 2 {
        return Reply::text(
            "❌ *Nama produk terlalu pendek.*\n\nMinimal 2 karakter diperlukan agar mudah dikenali pelanggan 📝",
        );
    }
    if price.map_or(true, |p| p < 1000) {
        return Reply::text(
            "❌ *Harga tidak valid.*\n\nHarga minimal *Rp 1.000* 💰 Pastikan Anda memasukkan angka yang benar.",
        );
    }
    let price = price.unwrap();

    let tipe = if type_str == "fisik" || type_str == "physical" {
        "fisik"
    } else {
        "digital"
    };
    let stock = if stock_str.to_lowercase() == "unlimited" {
        -1
    } else {
        js_parse_int(&stock_str).unwrap_or(999)
    };

    // ---- thumbnail: media reply / media langsung (`addproduk.js:67-88`) ----
    let has_quoted_media = ctx.quoted().map_or(false, |q| q.is_media());
    let is_direct_media =
        ctx.media.is_some() && (ctx.is_media_kind(oktz_plugin_api::registry::IncomingKind::Image) || ctx.is_media_kind(oktz_plugin_api::registry::IncomingKind::Video));

    let mut image_url: Option<String> = None;
    let mut video_url: Option<String> = None;
    let mut upload_warning: Option<String> = None;

    if has_quoted_media || is_direct_media {
        let is_video = ctx.quoted().map_or(false, |q| q.kind.is_video())
            || ctx.is_media_kind(oktz_plugin_api::registry::IncomingKind::Video);
        let unduhan = if has_quoted_media {
            ctx.download_quoted_media().ok()
        } else {
            ctx.download_media().ok()
        };
        match unduhan.filter(|b| !b.is_empty()) {
            Some(buffer) => {
                let filename = if is_video { "video.mp4" } else { "image.jpg" };
                match crate::tools::tourl::unggah_pertama(&ctx, &buffer, filename) {
                    Some(url) if is_video => video_url = Some(url),
                    Some(url) => image_url = Some(url),
                    None => {
                        upload_warning = Some(
                            "⚠️ *Gagal upload media: semua host upload gagal*\n> Produk tetap disimpan TANPA thumbnail.".to_string(),
                        )
                    }
                }
            }
            None => {
                upload_warning = Some(
                    "⚠️ *Gagal upload media: media tidak bisa diunduh*\n> Produk tetap disimpan TANPA thumbnail.".to_string(),
                )
            }
        }
    }

    let mut products: Vec<serde_json::Value> = ctx
        .setting(PRODUK)
        .and_then(|v| v.as_array())
        .map(|a| a.to_vec())
        .unwrap_or_default();

    let product_id = format!("P{}", crate::common::now_ms());
    let mut new_product = serde_json::json!({
        "id": product_id,
        "name": name,
        "price": price,
        "originalPrice": null,
        "type": tipe,
        "stock": stock,
        "stockItems": [],
        "description": description,
        "detail": "",
        "image": image_url,
        "video": video_url,
        "createdAt": ms_ke_iso(crate::common::now_ms()),
    });
    if new_product["image"].is_null() && image_url.is_none() {
        // tetap null seperti JS `image: null`
    }
    let _ = &mut new_product;

    products.push(new_product.clone());

    let type_icon = if tipe == "digital" { "🔑" } else { "📦" };
    let type_label = if tipe == "digital" { "Digital" } else { "Fisik" };

    let mut reply = String::from("✅ *PRODUK DITAMBAHKAN*\n\n");
    reply.push_str(&format!("🏷️ Nama: *{}*\n", new_product["name"].as_str().unwrap_or("")));
    reply.push_str(&format!("💰 Harga: *Rp {}*\n", ribuan(price)));
    reply.push_str(&format!("{type_icon} Tipe: *{type_label}*\n"));
    reply.push_str(&format!(
        "📊 Stok: *{}*\n",
        if stock == -1 { "Unlimited ♾️".to_string() } else { stock.to_string() }
    ));
    let desc = new_product["description"].as_str().unwrap_or("");
    if !desc.is_empty() {
        reply.push_str(&format!("📝 Deskripsi: _{desc}_\n"));
    }
    if image_url.is_some() {
        reply.push_str("🖼️ Thumbnail: ✅ Gambar\n");
    }
    if video_url.is_some() {
        reply.push_str("🎬 Thumbnail: ✅ Video\n");
    }
    if let Some(warning) = &upload_warning {
        reply.push_str(&format!("\n{warning}\n"));
    } else if image_url.is_none() && video_url.is_none() {
        reply.push_str("🖼️ Thumbnail: *tidak ada*\n");
    }
    reply.push_str("\n📌 *Langkah Selanjutnya:*\n");
    if tipe == "digital" {
        reply.push_str(&format!(
            "1️⃣ Tambahkan data akun/key: `{p}addstok {}|<detail>`\n",
            products.len(),
            p = ctx.prefix
        ));
        reply.push_str(&format!(
            "2️⃣ Atau import dari file .txt: `{p}addstok {}` (reply file 📄)\n",
            products.len(),
            p = ctx.prefix
        ));
    } else {
        reply.push_str(&format!(
            "1️⃣ Stok sudah diatur otomatis ({stock} pcs) 📦\n"
        ));
        reply.push_str(&format!(
            "2️⃣ Tambah stok: `{p}editproduk {} stok <jumlah>`\n",
            products.len(),
            p = ctx.prefix
        ));
    }
    reply.push_str(&format!(
        "3️⃣ Lihat produk: `{p}listproduk` 🛍️\n\n_Produk ini akan terlihat oleh pelanggan melalui `{p}listproduk`_ 🎉",
        p = ctx.prefix
    ));

    Reply::text(reply)
        .mutating(vec![Mutation::SetSetting {
            key: PRODUK.into(),
            value: serde_json::Value::Array(products),
        }])
        .with_reaction("✅")
}

/// `n.toLocaleString("id-ID")` — ribuan bertitik.
fn ribuan(n: i64) -> String {
    crate::tools::kalkulatormbg::thousands_id(n as f64)
}

/// `new Date().toISOString()` (sama dengan `store/done.rs::ms_ke_iso`).
fn ms_ke_iso(ms: i64) -> String {
    crate::store::done::ms_ke_iso(ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx_of(text: &str) -> Ctx {
        Ctx {
            text: text.to_string(),
            args: text.split('|').map(str::to_string).collect(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            prefix: ".".into(),
            command: "addproduk".into(),
            settings: json!({ "storeProducts": [] }),
            ..Ctx::default()
        }
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("addproduk").unwrap();
        assert_eq!(p.meta.name, "addproduk");
        assert_eq!(p.meta.alias, ["addproduct"]);
        assert_eq!(p.meta.category, "store");
        assert_eq!(
            p.meta.description,
            "➕ Tambah produk baru ke toko (hanya di private chat)"
        );
        assert_eq!(p.meta.usage, ".addproduk <nama>|<harga>|<tipe>|<stok>|<deskripsi>");
        assert_eq!(p.meta.example, ".addproduk Spotify Premium|25000|digital|10|Akun Premium 1 Bulan");
        assert_eq!(p.meta.cooldown, 3);
        assert!(p.meta.is_owner);
        assert!(p.meta.is_private, "isPrivate: true di JS");
    }

    #[test]
    fn js_parse_int_mengikuti_semantik_js() {
        assert_eq!(js_parse_int("25000"), Some(25000));
        assert_eq!(js_parse_int("25.000"), Some(25), "parseInt berhenti di titik");
        assert_eq!(js_parse_int("10 pcs"), Some(10));
        assert_eq!(js_parse_int("abc"), None);
        assert_eq!(js_parse_int(""), None);
        assert_eq!(js_parse_int("-5"), Some(-5));
    }

    #[test]
    fn di_grup_ditolak() {
        let mut c = ctx_of("Spotify|25000");
        c.is_group = true;
        let t = addproduk(c).text_body().unwrap().to_string();
        assert!(t.starts_with("🚫 *Akses Ditolak*"));
        assert!(t.contains("`.addproduk <nama>|<harga>|<tipe>|<stok>|<deskripsi>`"));
    }

    #[test]
    fn tanpa_pemisah_kartu_pakai() {
        let t = addproduk(ctx_of("hanya-nama")).text_body().unwrap().to_string();
        assert!(t.starts_with("➕ *TAMBAH PRODUK BARU*"));
        assert!(t.contains("`{p}addproduk <nama>".replace("{p}", ".").as_str()));
        assert!(t.contains("`{p}addstok`".replace("{p}", ".").as_str()));
    }

    #[test]
    fn nama_pendek_dan_harga_invalid() {
        let t = addproduk(ctx_of("A|25000")).text_body().unwrap().to_string();
        assert!(t.starts_with("❌ *Nama produk terlalu pendek.*"));
        let t = addproduk(ctx_of("Spotify|500")).text_body().unwrap().to_string();
        assert!(t.starts_with("❌ *Harga tidak valid.*"));
        let t = addproduk(ctx_of("Spotify|abc")).text_body().unwrap().to_string();
        assert!(t.starts_with("❌ *Harga tidak valid.*"));
    }

    #[test]
    fn produk_digital_ditambahkan_dengan_stok_default() {
        let r = addproduk(ctx_of("Spotify Premium|25000|digital|10|Akun Premium;;1 Bulan"));
        let t = r.text_body().unwrap().to_string();
        assert!(t.starts_with("✅ *PRODUK DITAMBAHKAN*"), "{t}");
        assert!(t.contains("🏷️ Nama: *Spotify Premium*"));
        assert!(t.contains("💰 Harga: *Rp 25.000*"));
        assert!(t.contains("🔑 Tipe: *Digital*"));
        assert!(t.contains("📊 Stok: *10*"));
        assert!(t.contains("📝 Deskripsi: _Akun Premium\n1 Bulan_"), ";; jadi newline");
        assert!(t.contains("🖼️ Thumbnail: *tidak ada*"));
        assert!(t.contains("1️⃣ Tambahkan data akun/key: `.addstok 1|<detail>`"));
        for m in r.mutations() {
            if let Mutation::SetSetting { key, value } = m {
                assert_eq!(key, "storeProducts");
                assert_eq!(value.as_array().unwrap().len(), 1);
                let p = &value[0];
                assert!(p["id"].as_str().unwrap_or("").starts_with('P'));
                assert_eq!(p["type"], "digital");
                assert_eq!(p["stock"], 10);
                assert_eq!(p["originalPrice"], serde_json::Value::Null);
                assert_eq!(p["stockItems"], json!([]));
                assert!(p["createdAt"].as_str().is_some_and(|s| s.ends_with('Z')));
                return;
            }
        }
        panic!("harus ada SetSetting storeProducts");
    }

    #[test]
    fn unlimited_dan_tipe_default_digital() {
        let r = addproduk(ctx_of("Netflix|35000|digital|unlimited|Sharing Account"));
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("📊 Stok: *Unlimited ♾️*"));
        for m in r.mutations() {
            if let Mutation::SetSetting { value, .. } = m {
                assert_eq!(value[0]["stock"], -1);
            }
        }
        let r = addproduk(ctx_of("Produk|5000"));
        for m in r.mutations() {
            if let Mutation::SetSetting { value, .. } = m {
                assert_eq!(value[0]["type"], "digital");
                assert_eq!(value[0]["stock"], 999, "stok kosong → 999");
            }
        }
    }

    #[test]
    fn produk_fisik_langkah_berikutnya_beda() {
        let r = addproduk(ctx_of("Baju Kaos|65000|fisik|8|Kaos Polos Cotton 30s"));
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("📦 Tipe: *Fisik*"));
        assert!(t.contains("1️⃣ Stok sudah diatur otomatis (8 pcs) 📦"));
        assert!(t.contains("2️⃣ Tambah stok: `.editproduk 1 stok <jumlah>`"));
        for m in r.mutations() {
            if let Mutation::SetSetting { value, .. } = m {
                assert_eq!(value[0]["type"], "fisik");
            }
        }
    }

    #[test]
    fn reaksi_sukses_dan_produk_kedua_dapat_indeks_dua() {
        let mut c = ctx_of("Kedua|2000");
        c.settings["storeProducts"] = json!([{ "id": "P1", "name": "Pertama" }]);
        let r = addproduk(c);
        assert!(matches!(r.without_mutations(), Reply::Reaction { emoji, .. } if emoji == "✅"));
        for m in r.mutations() {
            if let Mutation::SetSetting { value, .. } = m {
                assert_eq!(value.as_array().unwrap().len(), 2);
                assert_eq!(value[1]["name"], "Kedua");
            }
        }
    }

    #[test]
    fn thumbnail_gagal_unggah_saat_http_mati_dan_media_direply() {
        let mut c = ctx_of("Spotify|25000|digital|10|desc");
        // media langsung tanpa http (Ctx::default() → NoHttp).
        c.media = Some(oktz_plugin_api::registry::IncomingMedia {
            kind: oktz_plugin_api::registry::IncomingKind::Image,
            mimetype: Some("image/jpeg".into()),
            file_name: None,
            caption: String::new(),
        });
        let r = addproduk(c);
        let t = r.text_body().unwrap().to_string();
        assert!(t.contains("⚠️ *Gagal upload media:"), "{t}");
        assert!(t.contains("> Produk tetap disimpan TANPA thumbnail."));
        assert!(t.contains("🖼️ Thumbnail: ✅") == false);
    }
}
