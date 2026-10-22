//! `plugins/tools/bandingkan-hp.js` — bandingkan spesifikasi dua smartphone
//! dari Carisinyal.
//!
//! ## Penyimpangan
//!
//! - JS memakai `cheerio` untuk mem-parsing HTML. Crate `oktz-plugins` tidak
//!   punya parser HTML dan menambah dependensi berarti menyentuh
//!   `Cargo.toml` di luar wilayah yang boleh disentuh, jadi port ini memakai
//!   parser HTML kecil yang ditulis di file ini (tokenizer tag + pohon
//!   sederhana). Bentuk yang dibaca sama seperti di JS: elemen ber-class
//!   `ct-text-block` (judul seksi) dan `ct-new-columns` (baris perbandingan
//!   dengan anak `ct-div-block`).
//! - Album gambar (`albumMessage` + `messageAssociation`) tidak ada di API
//!   `Reply`. Port ini mengirim gambar-gambarnya sebagai pesan terpisah
//!   setelah teks, dengan urutan yang sama.

use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

/// Node HTML hasil parser mini.
#[derive(Debug, Clone)]
struct Node {
    tag: String,
    class: String,
    attrs: Vec<(String, String)>,
    /// Isi teks langsung (bukan termasuk anak).
    text: String,
    children: Vec<Node>,
    /// Posisi tag pembuka di HTML sumber — dipakai untuk mengurutkan node
    /// seperti urutan dokumen.
    pos: usize,
}

/// Tag yang tidak punya pasangan tutup.
const VOID: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

impl Node {
    fn attr(&self, nama: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == nama)
            .map(|(_, v)| v.as_str())
    }

    fn punya_class(&self, class: &str) -> bool {
        self.class.split_whitespace().any(|c| c == class)
    }

    /// `$(el).text()` — seluruh teks di subtree.
    fn teks(&self) -> String {
        let mut out = self.text.clone();
        for c in &self.children {
            out.push_str(&c.teks());
        }
        out
    }

    /// `node.find("img").first()`.
    fn cari_img(&self) -> Option<&Node> {
        for c in &self.children {
            if c.tag == "img" {
                return Some(c);
            }
            if let Some(inner) = c.cari_img() {
                return Some(inner);
            }
        }
        None
    }

    /// `node.find("li")` — semua anak `<li>` di subtree.
    fn cari_li(&self) -> Vec<&Node> {
        let mut out = Vec::new();
        for c in &self.children {
            if c.tag == "li" {
                out.push(c);
            }
            out.extend(c.cari_li());
        }
        out
    }

    /// `node.children(".ct-div-block")` — anak langsung dengan class itu.
    fn anak_dengan_class(&self, class: &str) -> Vec<&Node> {
        self.children
            .iter()
            .filter(|c| c.punya_class(class))
            .collect()
    }
}

/// Bentuk sementara saat pohon dibangun.
struct Raw {
    tag: String,
    class: String,
    attrs: Vec<(String, String)>,
    text: String,
    children: Vec<usize>,
    pos: usize,
}

/// Parser mini: token `<tag attr>`, `</tag>`, `<tag/>`, teks.
fn parse_html(html: &str) -> Node {
    let mut raw: Vec<Raw> = vec![Raw {
        tag: "#root".into(),
        class: String::new(),
        attrs: Vec::new(),
        text: String::new(),
        children: Vec::new(),
        pos: 0,
    }];
    let mut current = 0usize;
    let mut induk: Vec<usize> = Vec::new();
    let mut teks_buf = String::new();

    let bytes: Vec<char> = html.chars().collect();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == '<' {
            // Komentar / doctype.
            if bytes[i..].starts_with(&['<', '!', '-', '-']) {
                let mut j = i + 4;
                while j + 2 < bytes.len()
                    && !(bytes[j] == '-' && bytes[j + 1] == '-' && bytes[j + 2] == '>')
                {
                    j += 1;
                }
                i = (j + 3).min(bytes.len());
                continue;
            }
            if bytes[i..].starts_with(&['<', '!']) {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j] != '>' {
                    j += 1;
                }
                i = (j + 1).min(bytes.len());
                continue;
            }
            // Tagutup.
            if bytes.get(i + 1) == Some(&'/') {
                let mut j = i + 2;
                while j < bytes.len() && bytes[j] != '>' {
                    j += 1;
                }
                if !teks_buf.is_empty() {
                    let t = std::mem::take(&mut teks_buf);
                    raw[current].text.push_str(&t);
                }
                if let Some(p) = induk.pop() {
                    current = p;
                }
                i = (j + 1).min(bytes.len());
                continue;
            }
            // Tag buka.
            let tag_pos = i;
            let mut j = i + 1;
            let mut dalam_kutip: Option<char> = None;
            while j < bytes.len() {
                let c = bytes[j];
                match dalam_kutip {
                    Some(q) => {
                        if c == q {
                            dalam_kutip = None;
                        }
                    }
                    None => {
                        if c == '"' || c == '\'' {
                            dalam_kutip = Some(c);
                        } else if c == '>' {
                            break;
                        }
                    }
                }
                j += 1;
            }
            let isi: String = bytes[i + 1..j.min(bytes.len())].iter().collect();
            if !teks_buf.is_empty() {
                let t = std::mem::take(&mut teks_buf);
                raw[current].text.push_str(&t);
            }
            let self_close = isi.trim_end().ends_with('/');
            let isi_bersih = isi.trim_end().trim_end_matches('/');
            let mut parts = isi_bersih.splitn(2, |c: char| c.is_whitespace());
            let tag = parts.next().unwrap_or("").to_ascii_lowercase();
            let attrs_raw = parts.next().unwrap_or("");
            let attrs = parse_attrs(attrs_raw);
            let class = attrs
                .iter()
                .find(|(k, _)| k == "class")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            if !tag.is_empty() {
                raw.push(Raw {
                    tag: tag.clone(),
                    class,
                    attrs,
                    text: String::new(),
                    children: Vec::new(),
                    pos: tag_pos,
                });
                let idx = raw.len() - 1;
                raw[current].children.push(idx);
                if !VOID.contains(&tag.as_str()) && !self_close {
                    induk.push(current);
                    current = idx;
                }
            }
            i = (j + 1).min(bytes.len());
            continue;
        }
        teks_buf.push(bytes[i]);
        i += 1;
    }
    if !teks_buf.is_empty() {
        raw[current].text.push_str(&teks_buf);
    }

    bangun(&raw, 0)
}

/// Bentuk pohon dari arena.
fn bangun(raw: &[Raw], idx: usize) -> Node {
    let r = &raw[idx];
    Node {
        tag: r.tag.clone(),
        class: r.class.clone(),
        attrs: r.attrs.clone(),
        text: r.text.clone(),
        children: r.children.iter().map(|c| bangun(raw, *c)).collect(),
        pos: r.pos,
    }
}

fn parse_attrs(raw: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '/') {
            i += 1;
        }
        let start = i;
        while i < chars.len() && chars[i] != '=' && !chars[i].is_whitespace() {
            i += 1;
        }
        if start == i {
            break;
        }
        let nama: String = chars[start..i].iter().collect::<String>().to_ascii_lowercase();
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        let mut nilai = String::new();
        if i < chars.len() && chars[i] == '=' {
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                let q = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != q {
                    nilai.push(chars[i]);
                    i += 1;
                }
                i += 1;
            } else {
                while i < chars.len() && !chars[i].is_whitespace() {
                    nilai.push(chars[i]);
                    i += 1;
                }
            }
        }
        out.push((nama, nilai));
    }
    out
}

/// `text.replace(/\s+/g, " ").trim()`.
fn rapatkan(s: &str) -> String {
    let mut out = String::new();
    let mut spasi = false;
    for c in s.chars() {
        if c.is_whitespace() {
            spasi = true;
            continue;
        }
        if spasi && !out.is_empty() {
            out.push(' ');
        }
        spasi = false;
        out.push(c);
    }
    out
}

/// Kumpulkan node ber-class tertentu dalam urutan dokumen.
fn kumpulkan_class<'a>(node: &'a Node, class: &str, keluar: &mut Vec<&'a Node>) {
    for c in &node.children {
        if c.punya_class(class) {
            keluar.push(c);
        }
        kumpulkan_class(c, class, keluar);
    }
}

/// `node.closest(".ct-new-columns")` — ada ancestor dengan class itu?
fn punya_ancestor(node: &Node, class: &str, ancestors: &mut Vec<String>) -> bool {
    if ancestors
        .iter()
        .any(|c| c.split_whitespace().any(|x| x == class))
    {
        return true;
    }
    for c in &node.children {
        ancestors.push(c.class.clone());
        let found = punya_ancestor(c, class, ancestors);
        ancestors.pop();
        if found {
            return true;
        }
    }
    false
}

/// `extractCell` (`bandingkan-hp.js:72-87`).
fn extract_cell(cell: &Node) -> String {
    if let Some(img) = cell.cari_img() {
        if let Some(src) = img.attr("src") {
            return src.to_string();
        }
        if let Some(src) = img.attr("data-src") {
            return src.to_string();
        }
        return String::new();
    }
    let items = cell.cari_li();
    if !items.is_empty() {
        return items
            .iter()
            .map(|li| rapatkan(&li.teks()))
            .collect::<Vec<_>>()
            .join("; ");
    }
    rapatkan(&cell.teks())
}

/// Satu baris perbandingan.
struct Baris {
    label: Option<String>,
    value1: String,
    value2: String,
}

/// Satu seksi hasil scraping.
struct Seksi {
    section: String,
    rows: Vec<Baris>,
}

/// `fetchCompare` (`bandingkan-hp.js:89-123`).
fn fetch_compare(html: &str) -> Vec<Seksi> {
    let root = parse_html(html);
    let mut blok: Vec<&Node> = Vec::new();
    kumpulkan_class(&root, "ct-text-block", &mut blok);
    kumpulkan_class(&root, "ct-new-columns", &mut blok);
    // Urutan dokumen.
    blok.sort_by_key(|n| n.pos);

    let mut sections: Vec<Seksi> = Vec::new();
    let mut current = Seksi {
        section: "UMUM".to_string(),
        rows: Vec::new(),
    };

    for node in blok {
        if node.punya_class("ct-text-block") {
            let mut ancestors = Vec::new();
            if punya_ancestor(node, "ct-new-columns", &mut ancestors) {
                continue;
            }
            let title = node.teks().trim().to_string();
            if title.is_empty() {
                continue;
            }
            if !current.rows.is_empty() {
                sections.push(current);
            }
            current = Seksi {
                section: title,
                rows: Vec::new(),
            };
            continue;
        }

        let cells = node.anak_dengan_class("ct-div-block");
        if cells.len() < 3 {
            continue;
        }
        let label = rapatkan(&cells[0].teks());
        let value1 = extract_cell(cells[1]);
        let value2 = extract_cell(cells[2]);
        if value1.is_empty() && value2.is_empty() {
            continue;
        }
        current.rows.push(Baris {
            label: if label.is_empty() { None } else { Some(label) },
            value1,
            value2,
        });
    }

    if !current.rows.is_empty() {
        sections.push(current);
    }
    sections
}

/// `normalize` (`bandingkan-hp.js:29-31`).
fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect()
}

/// `score` (`bandingkan-hp.js:33-43`).
fn score(title: &str, query: &str) -> i64 {
    let t = normalize(title);
    let q = normalize(query);
    let mut s = 0i64;
    if t == q {
        s += 100;
    }
    if t.contains(&q) {
        s += 80;
    }
    if q.contains(&t) {
        s += 60;
    }
    let mut kata = String::new();
    let mut digit = false;
    for c in q.chars() {
        let d = c.is_ascii_digit();
        if !kata.is_empty() && d != digit {
            if t.contains(&kata) {
                s += 10;
            }
            kata.clear();
        }
        digit = d;
        kata.push(c);
    }
    if !kata.is_empty() && t.contains(&kata) {
        s += 10;
    }
    s
}

/// Satu entri daftar HP.
#[derive(Clone)]
struct Hp {
    id: String,
    title: String,
}

/// `getPhoneList` (`bandingkan-hp.js:45-57`).
fn get_phone_list(html: &str) -> Vec<Hp> {
    let mut list = Vec::new();
    let Some(mulai) = html.find("<select") else {
        return list;
    };
    let Some(selesai) = html[mulai..].find("</select>").map(|i| mulai + i) else {
        return list;
    };
    let blok = &html[mulai..selesai];
    let Some(pos_nama) = blok.find("hp_1") else {
        return list;
    };
    let isi = &blok[pos_nama..];
    let mut idx = 0usize;
    while let Some(pos) = isi[idx..].find("<option") {
        let tag_mulai = idx + pos;
        let Some(tag_akhir) = isi[tag_mulai..].find('>').map(|i| tag_mulai + i) else {
            break;
        };
        let tag = &isi[tag_mulai..tag_akhir];
        let value = ambil_atribut(tag, "value");
        let after = &isi[tag_akhir + 1..];
        let Some(close) = after.find("</option>") else {
            break;
        };
        let title = rapatkan(&after[..close]);
        if let Some(v) = value
            && !v.is_empty()
            && !title.is_empty()
        {
            list.push(Hp { id: v, title });
        }
        idx = tag_akhir + 1 + close + "</option>".len();
    }
    list
}

/// Ambil nilai atribut dari potongan tag.
fn ambil_atribut(tag: &str, nama: &str) -> Option<String> {
    let mut cursor = 0usize;
    while let Some(pos) = tag[cursor..].find(nama) {
        let start = cursor + pos;
        // Pastikan bukan bagian dari nama atribut lain ("data-value").
        let sebelum = tag[..start].chars().next_back();
        if sebelum.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            cursor = start + nama.len();
            continue;
        }
        let mut i = start + nama.len();
        while i < tag.len() && tag.as_bytes()[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= tag.len() || tag.as_bytes()[i] != b'=' {
            cursor = start + nama.len();
            continue;
        }
        i += 1;
        while i < tag.len() && tag.as_bytes()[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= tag.len() {
            return None;
        }
        let q = tag.as_bytes()[i] as char;
        if q == '"' || q == '\'' {
            let rest = &tag[i + 1..];
            let end = rest.find(q)?;
            return Some(rest[..end].to_string());
        }
        let end = tag[i..]
            .find(|c: char| c.is_whitespace())
            .unwrap_or(tag.len() - i);
        return Some(tag[i..i + end].to_string());
    }
    None
}

/// `bestMatch` (`bandingkan-hp.js:59-70`).
fn best_match(list: &[Hp], query: &str) -> Option<Hp> {
    let mut best: Option<Hp> = None;
    let mut best_score = -1i64;
    for item in list {
        let s = score(&item.title, query);
        if s > best_score {
            best_score = s;
            best = Some(item.clone());
        }
    }
    best
}

fn cara_pakai(prefix: &str) -> String {
    format!(
        "⚖️ *FITUR PERBANDINGAN HP*\n\n\
         Fitur ini akan membantumu membandingkan spesifikasi dua *smartphone* secara langsung agar kamu bisa memilih mana yang terbaik!\n\n\
         *CARA PENGGUNAAN:*\n\
         - Ketik `{prefix}bandingkan-hp <hp pertama> | <hp kedua>`\n\
         - Contoh: `{prefix}bandingkan-hp iphone 13 | samsung s22`\n\n\
         _Pastikan kamu menggunakan tanda garis lurus (|) sebagai pemisah antara dua nama HP tersebut ya!_"
    )
}

fn format_salah() -> &'static str {
    "❌ *FORMAT SALAH*\n\nPastikan kamu memasukkan dua nama ponsel dengan pemisah tanda vertikal (|)."
}

fn gagal() -> &'static str {
    "❌ *GAGAL MENGAMBIL DATA*\n\nMaaf, sistem mengalami gangguan saat mencoba membandingkan HP dari *Carisinyal*. Silakan coba lagi nanti."
}

#[plugin(
    name = "bandingkan-hp",
    alias = ["bandingkanhp", "comparehp"],
    category = "tools",
    description = "Membandingkan dua spesifikasi smartphone dari Carisinyal.",
    usage = ".bandingkan-hp <hp1> | <hp2>",
    example = ".bandingkan-hp oppo a3s | vivo y91",
    cooldown = 5,
    energi = 2,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn bandingkan_hp(ctx: Ctx) -> Reply {
    let text = ctx.text.clone();
    if text.is_empty() || !text.contains('|') {
        return Reply::text(cara_pakai(&ctx.prefix));
    }

    let mut bagian = text.split('|');
    let query1 = bagian.next().unwrap_or("").trim().to_string();
    let query2 = bagian.next().unwrap_or("").trim().to_string();
    if query1.is_empty() || query2.is_empty() {
        return Reply::text(format_salah()).with_reaction("❌");
    }

    let Ok(res) = ctx.http.get("https://carisinyal.com/compare/") else {
        return Reply::text(gagal()).with_reaction("❌");
    };
    let list = get_phone_list(&res.text());
    let phone1 = best_match(&list, &query1);
    let phone2 = best_match(&list, &query2);

    let (phone1, phone2) = (phone1, phone2);
    let (Some(phone1), Some(phone2)) = (phone1.clone(), phone2.clone()) else {
        let mut err = String::from("❌ *HP TIDAK DITEMUKAN*\n\n");
        if phone1.is_none() {
            err.push_str(&format!("- *{query1}* tidak ditemukan di database.\n"));
        }
        if phone2.is_none() {
            err.push_str(&format!("- *{query2}* tidak ditemukan di database.\n"));
        }
        err.push_str("\nSilakan coba kata kunci yang berbeda.");
        return Reply::text(err).with_reaction("❌");
    };

    let url = format!(
        "https://carisinyal.com/compare/?hp_1={}&hp_2={}",
        phone1.id, phone2.id
    );
    let Ok(res) = ctx.http.get(&url) else {
        return Reply::text(gagal()).with_reaction("❌");
    };
    let mut sections = fetch_compare(&res.text());

    // Kumpulkan gambar dan ganti nilainya dengan penanda (`bandingkan-hp.js:161-173`).
    let mut images: Vec<String> = Vec::new();
    for sec in sections.iter_mut() {
        for row in sec.rows.iter_mut() {
            if row.value1.starts_with("http") {
                if !images.contains(&row.value1) {
                    images.push(row.value1.clone());
                }
                row.value1 = "📸 (Cek Album)".to_string();
            }
            if row.value2.starts_with("http") {
                if !images.contains(&row.value2) {
                    images.push(row.value2.clone());
                }
                row.value2 = "📸 (Cek Album)".to_string();
            }
        }
    }

    let mut caption = String::from("⚖️ *PERBANDINGAN SMARTPHONE*\n\n");
    caption.push_str(&format!("📱 *HP 1:* {}\n", phone1.title));
    caption.push_str(&format!("📱 *HP 2:* {}\n\n", phone2.title));
    for sec in &sections {
        caption.push_str(&format!("✨ *{}*\n", sec.section.to_uppercase()));
        for row in &sec.rows {
            let Some(label) = &row.label else {
                continue;
            };
            caption.push_str(&format!("🔹 *{label}:*\n"));
            caption.push_str(&format!("  - *1:* {}\n", row.value1));
            caption.push_str(&format!("  - *2:* {}\n", row.value2));
        }
        caption.push('\n');
    }
    caption.push_str("🔗 *Sumber:* Carisinyal");

    let mut balasan = vec![Reply::text(caption)];
    for img in images {
        balasan.push(Reply::image(MediaSource::url(img), ""));
    }
    Reply::sequence(balasan).with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{Ctx, Reply as R};

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_kosong() -> Ctx {
        Ctx {
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "bandingkan-hp".into(),
            ..Ctx::default()
        }
    }

    fn body(r: &R) -> String {
        r.text_body().expect("harus ada teks").to_string()
    }

    #[test]
    fn output_sama_dengan_js_tanpa_pemisah() {
        let r = bandingkan_hp(ctx_kosong());
        assert_eq!(
            body(&r),
            "⚖️ *FITUR PERBANDINGAN HP*\n\n\
             Fitur ini akan membantumu membandingkan spesifikasi dua *smartphone* secara langsung agar kamu bisa memilih mana yang terbaik!\n\n\
             *CARA PENGGUNAAN:*\n\
             - Ketik `.bandingkan-hp <hp pertama> | <hp kedua>`\n\
             - Contoh: `.bandingkan-hp iphone 13 | samsung s22`\n\n\
             _Pastikan kamu menggunakan tanda garis lurus (|) sebagai pemisah antara dua nama HP tersebut ya!_"
        );
    }

    #[test]
    fn output_sama_dengan_js_format_salah() {
        let ctx = Ctx {
            text: "oppo |".into(),
            ..ctx_kosong()
        };
        let r = bandingkan_hp(ctx);
        assert_eq!(
            body(&r),
            "❌ *FORMAT SALAH*\n\nPastikan kamu memasukkan dua nama ponsel dengan pemisah tanda vertikal (|)."
        );
    }

    #[test]
    fn score_sama_dengan_js() {
        assert_eq!(score("iPhone 13", "iphone 13"), 100 + 80 + 60 + 20);
        assert!(score("OPPO A3S", "oppo a3s") > 0);
        assert_eq!(score("OPPO A3S", "vivo y91"), 0);
    }

    #[test]
    fn daftar_hp_dibaca_dari_select() {
        let html = "<html><body><select name=\"hp_1\"><option value=\"1\">OPPO A3S</option><option value=\"2\">VIVO Y91</option></select></body></html>";
        let list = get_phone_list(html);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "1");
        assert_eq!(list[0].title, "OPPO A3S");
        assert_eq!(list[1].title, "VIVO Y91");
    }

    #[test]
    fn seksi_dibaca_dari_html() {
        let html = "<div class=\"ct-text-block\">LAYAR</div>\
                    <div class=\"ct-new-columns\"><div class=\"ct-div-block\">Ukuran</div>\
                    <div class=\"ct-div-block\">6.5 inci</div><div class=\"ct-div-block\">6.4 inci</div></div>";
        let sections = fetch_compare(html);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].section, "LAYAR");
        assert_eq!(sections[0].rows.len(), 1);
        assert_eq!(sections[0].rows[0].label.as_deref(), Some("Ukuran"));
        assert_eq!(sections[0].rows[0].value1, "6.5 inci");
    }
}
