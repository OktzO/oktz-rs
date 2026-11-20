//! `.cekkartu` — diagnostik kartu AIRich primitif HTML.
//!
//! Klien owner menampilkan "versi WhatsApp Anda tidak mendukungnya" untuk
//! `.play2`/`.ytplay` (primitif HTML), sementara `.ai` (teks) dan `.ping2`
//! (tabel) dari bot yang sama render normal. Satu perintah ini mengirim
//! sembilan kartu yang HANYA berbeda satu variabel, supaya sekali uji bisa
//! dipastikan mana yang ditolak klien:
//!
//! A. baseline: teks HTML minimal, tanpa submessage tambahan
//! B. `img` data-URI di dalam payload
//! C. `<script>` inline
//! D. `trusted_sources` tidak kosong
//! E. dokumen HTML utuh (`<!DOCTYPE html>`)
//! F. karakter non-ASCII (emoji, `▶`, `❚❚`)
//! G. payload besar (~30 KB) tanpa gambar
//! H. section dibungkus `GenAIUnifiedResponseSection` + `title`
//! I. `submessages` KOSONG (hanya section)
//!
//! Ronde 2 (adendum 6–7): A–I semua gagal di klien Android owner
//! (2.26.45.99) sementara teks/tabel render; amplop sudah identik byte
//! dengan JS. J–N menguji sisa kandidat di lapis amplop/jalur:
//!
//! J. section markdown + HTML dalam SATU pesan (markdown tampil tapi HTML
//!    tidak = masalah tingkat section; seluruh pesan gagal = tingkat pesan)
//! K. edit susulan identik `protocolMessage type=14` (zaileys
//!    `bypassDownload` — kartu baru dirender setelah edit-relay)
//! L. `botJid: 0@bot` (nilai capture nyata Vanta 2026-11-18)
//! M. bentuk capture Vanta penuh: `0@bot` + `richResponseSourcesMetadata`
//!    kosong + disclaimer judul
//! N. typename `FOAHtmlPrimitiveDemoDONOTUSE` (satu-satunya kelas impl di
//!    dex; barangkali registrasi Pando berubah)
//!
//! Kalau A muncul tapi B tidak → gambar/data-URI; kalau A muncul tapi G
//! tidak → ukuran; kalau TIDAK ADA yang muncul → klien memang menolak
//! primitif HTML (bukan isi payload, bukan amplop).
use oktz_plugin_api::{AiSubMessage, Ctx, Reply, RichAi, plugin};
use serde_json::json;

use crate::search::play2::base64_encode;

const KEPALA: &str = "🧪 *Uji kartu HTML*\n\nEmpat belas kartu menyusul berurutan: A–I (isi payload), J–N (amplop/jalur).\nKartu yang muncul = variasi itu diterima klien. Laporkan huruf mana saja yang tampil (atau \"tidak ada sama sekali\").";

fn judul(huruf: &str) -> String {
    format!(r#"<div style="font:700 24px sans-serif;padding:12px">KARTU {huruf}</div>"#)
}

fn primitive(html: String, sumber: &[&str]) -> serde_json::Value {
    json!({
        "__typename": "GenAIaeacdsnwHtmlPrimitive",
        "payload": html,
        "trusted_sources": sumber,
    })
}

fn satu_section(html: String, sumber: &[&str]) -> serde_json::Value {
    json!({
        "view_model": {
            "primitive": primitive(html, sumber),
            "__typename": "GenAISingleLayoutViewModel",
        }
    })
}

fn kartu(huruf: &str, section: serde_json::Value, dengan_teks: bool) -> Reply {
    let submessages = if dengan_teks {
        vec![AiSubMessage::Teks {
            teks: format!("Kartu {huruf}"),
        }]
    } else {
        Vec::new()
    };
    Reply::TabelAi(Box::new(RichAi {
        submessages,
        sections: vec![section],
        disclaimer: Some(String::new()),
        quoted: false,
        skor: Some(1),
        ..Default::default()
    }))
}

/// Kartu varian J–N: satu section HTML + knob amplop/jalur diubah `ubah`.
fn kartu_varian(huruf: &str, section: serde_json::Value, ubah: impl FnOnce(&mut RichAi)) -> Reply {
    let mut rich = RichAi {
        submessages: vec![AiSubMessage::Teks {
            teks: format!("Kartu {huruf}"),
        }],
        sections: vec![section],
        disclaimer: Some(String::new()),
        quoted: false,
        skor: Some(1),
        ..Default::default()
    };
    ubah(&mut rich);
    Reply::TabelAi(Box::new(rich))
}

pub(crate) fn semua_kartu() -> Reply {
    // B: gambar data-URI (SVG kecil yang sah).
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><rect width="80" height="40" fill="#c33"/></svg>"##;
    let gambar = format!(
        r#"{}<img src="data:image/svg+xml;base64,{}" width="80" height="40" alt="">"#,
        judul("B"),
        base64_encode(svg.as_bytes())
    );

    // C: skrip inline.
    let skrip = format!(
        r#"{}<script>document.body.style.background='#123456'</script>"#,
        judul("C")
    );

    // E: dokumen utuh.
    let dokumen = format!("<!DOCTYPE html><html><body>{}</body></html>", judul("E"));

    // F: non-ASCII.
    let unicode = format!(r#"{}<div>▶ ❚❚ 🎧 ✓</div>"#, judul("F"));

    // G: muatan besar tanpa gambar (~30 KB teks).
    let mut besar = judul("G");
    for i in 0..600 {
        besar.push_str(&format!(
            r#"<div style="font:13px sans-serif">baris uji {i} — 0123456789 abcdefghijklmnopqrstuvwxyz</div>"#
        ));
    }

    // H: section dibungkus tipe section + judul (bentuk `.ytplay`).
    let h = json!({
        "__typename": "GenAIUnifiedResponseSection",
        "title": "Uji H",
        "view_model": {
            "__typename": "GenAISingleLayoutViewModel",
            "title": "Uji H",
            "primitive": primitive(judul("H"), &[]),
        }
    });

    // J: markdown + HTML dalam satu pesan — diskriminasi tingkat section
    // vs tingkat pesan. Bentuk markdown sama dengan builder `.ai`.
    let markdown = json!({
        "view_model": {
            "primitive": {
                "text": "Kartu J: bagian markdown ini harus tampil.",
                "__typename": "GenAIMarkdownTextUXPrimitive",
            },
            "__typename": "GenAISingleLayoutViewModel",
        }
    });

    // N: typename demo — satu-satunya kelas `FOAHtmlPrimitive` impl di dex.
    let n = json!({
        "view_model": {
            "primitive": {
                "__typename": "FOAHtmlPrimitiveDemoDONOTUSE",
                "payload": judul("N"),
                "trusted_sources": [],
            },
            "__typename": "GenAISingleLayoutViewModel",
        }
    });

    Reply::sequence(vec![
        Reply::text(KEPALA),
        kartu("A", satu_section(judul("A"), &[]), true),
        kartu("B", satu_section(gambar, &[]), true),
        kartu("C", satu_section(skrip, &[]), true),
        kartu("D", satu_section(judul("D"), &["contoh"]), true),
        kartu("E", satu_section(dokumen, &[]), true),
        kartu("F", satu_section(unicode, &[]), true),
        kartu("G", satu_section(besar, &[]), true),
        kartu("H", h, true),
        kartu("I", satu_section(judul("I"), &[]), false),
        kartu_varian("J", satu_section(judul("J"), &[]), |r| {
            r.sections.insert(0, markdown);
        }),
        kartu_varian("K", satu_section(judul("K"), &[]), |r| {
            r.edit_susulan = true;
        }),
        kartu_varian("L", satu_section(judul("L"), &[]), |r| {
            r.bot_jid = Some("0@bot".into());
        }),
        kartu_varian("M", satu_section(judul("M"), &[]), |r| {
            r.bot_jid = Some("0@bot".into());
            r.sources_metadata = true;
            r.disclaimer = Some("Kartu M".into());
        }),
        kartu_varian("N", n, |_| {}),
    ])
}

#[plugin(
    name = "cekkartu",
    alias = ["ujikartu", "airichtest"],
    category = "tools",
    description = "Diagnostik kartu HTML AIRich (uji 9 variasi payload)",
    usage = ".cekkartu",
    example = ".cekkartu",
    cooldown = 5,
    energi = 0,
    // `false`/`false` = tidak dibatasi chat (sama seperti plugin owner lain);
    // `true` berarti "khusus", jadi dua-duanya true mustahil dipenuhi.
    is_group = false,
    is_private = false,
    is_owner = true,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn cekkartu(_ctx: Ctx) -> Reply {
    semua_kartu()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards(r: &Reply) -> Vec<&RichAi> {
        let Reply::Sequence(list) = r.without_mutations().tanpa_reaksi() else {
            panic!("harus sequence");
        };
        list.iter()
            .filter_map(|x| match x.without_mutations().tanpa_reaksi() {
                Reply::TabelAi(ai) => Some(&**ai),
                _ => None,
            })
            .collect()
    }

    fn payload(ai: &RichAi) -> String {
        ai.sections[0]["view_model"]["primitive"]["payload"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn diagnostik_bisa_dipakai_di_chat_privat_dan_grup() {
        let meta = oktz_plugin_api::find("cekkartu").unwrap().meta.clone();
        assert!(!meta.is_group && !meta.is_private, "tidak dibatasi chat");
        assert!(meta.is_owner, "hanya owner");
    }

    #[test]
    fn empat_belas_kartu_menguji_satu_variabel_per_kartu() {
        let r = semua_kartu();
        let list = cards(&r);
        assert_eq!(list.len(), 14, "A..N");
        assert!(payload(list[0]).contains("KARTU A"));
        assert!(payload(list[1]).contains("data:image/svg+xml;base64,"), "B gambar");
        assert!(payload(list[2]).contains("<script>"), "C skrip");
        assert_eq!(
            list[3].sections[0]["view_model"]["primitive"]["trusted_sources"],
            json!(["contoh"]),
            "D sumber"
        );
        assert!(payload(list[4]).starts_with("<!DOCTYPE html>"), "E dokumen");
        assert!(payload(list[5]).contains('▶'), "F non-ASCII");
        assert!(payload(list[6]).len() > 20_000, "G besar: {}", payload(list[6]).len());
        assert_eq!(
            list[7].sections[0]["__typename"],
            json!("GenAIUnifiedResponseSection"),
            "H bungkus section"
        );
        assert_eq!(list[7].sections[0]["view_model"]["title"], json!("Uji H"));
        assert!(list[8].submessages.is_empty(), "I tanpa submessage");
        // J: markdown dulu, baru HTML — satu pesan dua section.
        assert_eq!(list[9].sections.len(), 2, "J dua section");
        assert_eq!(
            list[9].sections[0]["view_model"]["primitive"]["__typename"],
            json!("GenAIMarkdownTextUXPrimitive"),
            "J section markdown"
        );
        assert!(
            list[9].sections[1]["view_model"]["primitive"]["payload"]
                .as_str()
                .unwrap()
                .contains("KARTU J")
        );
        // K: edit susulan identik (zaileys bypassDownload).
        assert!(list[10].edit_susulan, "K edit susulan");
        // L: botJid capture Vanta.
        assert_eq!(list[11].bot_jid.as_deref(), Some("0@bot"), "L botJid");
        // M: bentuk capture Vanta penuh.
        assert_eq!(list[12].bot_jid.as_deref(), Some("0@bot"), "M botJid");
        assert!(list[12].sources_metadata, "M sources metadata");
        assert_eq!(list[12].disclaimer.as_deref(), Some("Kartu M"), "M disclaimer");
        // N: typename demo.
        assert_eq!(
            list[13].sections[0]["view_model"]["primitive"]["__typename"],
            json!("FOAHtmlPrimitiveDemoDONOTUSE"),
            "N typename demo"
        );
        for (i, ai) in list.iter().enumerate() {
            if i == 13 {
                continue; // N typename demo
            }
            let idx = if i == 9 { 1 } else { 0 };
            assert_eq!(
                ai.sections[idx]["view_model"]["primitive"]["__typename"],
                json!("GenAIaeacdsnwHtmlPrimitive"),
                "kartu {i}"
            );
        }
    }
}