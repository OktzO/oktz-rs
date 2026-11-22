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
//! Ronde 3 (adendum 9): K/L/M render, tapi `.play2` (0@bot + edit susulan)
//! tetap gagal. O–T semua memakai amplop L yang terbukti (`0@bot`, score 1)
//! dan menguji sisa delta terhadap `.play2` asli:
//!
//! O. amplop L + edit susulan (combo `0@bot`+edit belum pernah diuji)
//! P. payload `player_html` asli (poster placeholder + lirik, kecil)
//! Q. P + edit susulan = cermin persis envelope produksi `.play2`
//! R. payload besar ~30 KB di bawah amplop L (ukuran)
//! S. `<style>` saja tanpa img (isolasi style)
//! T. `img` data-URI JPEG sah (isolasi format gambar)
//!
//! Ronde 4 (adendum 10): `.play2` SUDAH render (envelope v0.2.69). Sisa
//! pertanyaan: batas ukuran payload di klien owner — sumber komunitas
//! bertentangan (Vanta 60000, kotonehara 64 KiB, zaileys 256 KiB). U–Y
//! tangga ukuran 60/128/256/512/768 KB di amplop L: kartu terbesar yang
//! render = langit-langit nyata; itu menentukan panjang audio yang bisa
//! ditanam di kartu player (16 kbps opus ≈ 2 KB/detik mentah).
//! - Z: data URI ogg langsung (tanpa edit) — baseline.
//! - AA: Z + edit susulan — uji hipotesis reload membunuh audio.
//! - AB: data URI → `fetch` → blob URL — uji blokir skema `data:`.
//!
//! Hasil owner (v0.2.73): Z/AA/AB `error.code=4`,
//! "Media load rejected by URL safety check"; AB `fetch REJECT TypeError`.
//! Itu `HTMLMediaElement::IsSafeToLoadURL` → `SecurityOrigin::CanReadContent`
//! GAGAL: dokumen kartu ber-origin OPAQUE, jadi SEMUA URL media ditolak di
//! level origin (bukan codec, bukan CSP media, bukan edit susulan).
//!
//! Ronde 6: dua jalur yang tidak lewat URL media sama sekali / membawa
//! origin dokumen:
//!
//! AC. Web Audio: `atob` → ArrayBuffer → `decodeAudioData` → BufferSource
//!     (tanpa `<audio src>`, tanpa URL → tak tersentuh IsSafeToLoadURL).
//! AD. blob dibuat IN-DOCUMENT dari `atob` (bukan fetch): origin blob =
//!     origin dokumen → kandidat lolos `CanReadContent`.
use oktz_plugin_api::{AiSubMessage, Ctx, Reply, RichAi, plugin};
use serde_json::json;

use crate::search::play2::{Baris, base64_encode, player_html};

const KEPALA: &str = "🧪 *Uji kartu HTML*\n\nTiga puluh kartu: A–Y (payload/amplop/ukuran), Z–AD (uji AUDIO).\nKhusus Z–AD: pencet tombol ▶ PLAY di kartunya, lalu laporkan (1) ada suara atau tidak, (2) tulisan hijau di bawah tombol — itu diagnosis mandiri player-nya. Prioritas: AC dan AD.";

/// Opus 6 detik (sine 440 Hz, 16k/24kHz/mono/ogg — argumen ffmpeg yang
/// sama dengan `.play2`) untuk probe audio Z/AA/AB.
const AUDIO_UJI: &str = "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";

/// JPEG 1×1 sah untuk uji format gambar (kartu T).
const JPEG_UJI: &str = "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";

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

/// Kartu ronde 3 (O–T): amplop L proven (`0@bot`, score 1) + satu variabel
/// uji dari `ubah`.
fn kartu_l(huruf: &str, html: String, ubah: impl FnOnce(&mut RichAi)) -> Reply {
    kartu_varian(huruf, satu_section(html, &[]), |r| {
        r.bot_jid = Some("0@bot".into());
        ubah(r);
    })
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
        kartu_l("O", judul("O"), |r| r.edit_susulan = true),
        kartu_l("P", player_html_uji("P"), |_| {}),
        kartu_l("Q", player_html_uji("Q"), |r| r.edit_susulan = true),
        kartu_l("R", payload_besar("R"), |_| {}),
        kartu_l(
            "S",
            judul("S") + "<style>body{background:#1a0d12;color:#fff;font-family:sans-serif}</style>",
            |_| {},
        ),
        kartu_l(
            "T",
            format!(r#"{}<img src="data:image/jpeg;base64,{JPEG_UJI}" width="80" height="80" alt="">"#, judul("T")),
            |_| {},
        ),
        kartu_l("U", payload_ukuran("U", 60_000), |_| {}),
        kartu_l("V", payload_ukuran("V", 128_000), |_| {}),
        kartu_l("W", payload_ukuran("W", 256_000), |_| {}),
        kartu_l("X", payload_ukuran("X", 512_000), |_| {}),
        kartu_l("Y", payload_ukuran("Y", 768_000), |_| {}),
        kartu_audio("Z", false, false),
        kartu_audio("AA", false, true),
        kartu_audio("AB", true, false),
        kartu_audio_wa("AC"),
        kartu_audio_blob("AD"),
    ])
}

/// Player mini berbasis **Web Audio** — tanpa URL media sama sekali
/// (`decodeAudioData` dari ArrayBuffer atob). Melewati
/// `IsSafeToLoadURL` yang menolak origin opaque.
fn kartu_audio_wa(huruf: &str) -> Reply {
    let html = AUDIO_PROBE_WA
        .replace("__HURUF__", huruf)
        .replace("__B64__", AUDIO_UJI);
    kartu_l(huruf, html, |_| {})
}

/// Player mini `<audio>` dengan blob URL dibuat **in-document** dari atob
/// (bukan `fetch`): origin blob = origin dokumen, kandidat lolos
/// `CanReadContent`.
fn kartu_audio_blob(huruf: &str) -> Reply {
    let html = AUDIO_PROBE_BLOB
        .replace("__HURUF__", huruf)
        .replace("__B64__", AUDIO_UJI);
    kartu_l(huruf, html, |_| {})
}

const AUDIO_PROBE_WA: &str = r#"<div style="font:700 18px sans-serif;padding:10px">KARTU __HURUF__ — uji WebAudio</div>
<button id="p" style="font-size:20px;padding:16px 32px;margin:10px">▶ PLAY</button>
<div id="s" style="font:12px monospace;padding:10px;background:#000;color:#0f0;white-space:pre-wrap">init</div>
<script>
(function(){
var s=document.getElementById('s'),log=[],ctx=null,buf=null,src=null,t0=0,tick=null;
function t(x){log.push(x);if(log.length>8)log.shift();s.textContent=log.join('\n');}
var b64='__B64__';
function bytes(){var bin=atob(b64),u=new Uint8Array(bin.length);for(var i=0;i<bin.length;i++)u[i]=bin.charCodeAt(i);return u;}
var AC=window.AudioContext||window.webkitAudioContext;
t('AudioContext='+(AC?'ada':'TIDAK ADA'));
document.getElementById('p').addEventListener('click',function(){
try{
if(!AC){t('berhenti: tanpa AudioContext');return;}
if(!ctx){ctx=new AC();t('ctx='+ctx.state);}
if(ctx.state==='suspended'){ctx.resume().then(function(){t('resume='+ctx.state);});}
if(!buf){ctx.decodeAudioData(bytes().buffer,function(b){buf=b;t('decode OK dur='+b.duration.toFixed(3));mulai();},function(e){t('decode GAGAL '+(e&&e.message||e));});}
else{mulai();}
}catch(e){t('EXC '+e.name+': '+e.message);}
});
function mulai(){
if(!ctx||!buf){return;}
try{
if(src){src.stop();}
src=ctx.createBufferSource();src.buffer=buf;src.connect(ctx.destination);
var mulaiAt=ctx.currentTime+0.05;src.start(mulaiAt);t0=mulaiAt;
t('start ctx='+ctx.state+' sampleRate='+ctx.sampleRate);
if(tick){clearInterval(tick);}
tick=setInterval(function(){t('wb='+(ctx.currentTime-t0).toFixed(2));},500);
}catch(e){t('EXC start '+e.name+': '+e.message);}
}
})();
</script>
"#;

const AUDIO_PROBE_BLOB: &str = r#"<div style="font:700 18px sans-serif;padding:10px">KARTU __HURUF__ — uji blob atob</div>
<button id="p" style="font-size:20px;padding:16px 32px;margin:10px">▶ PLAY</button>
<div id="s" style="font:12px monospace;padding:10px;background:#000;color:#0f0;white-space:pre-wrap">init</div>
<audio id="a" preload="metadata"></audio>
<script>
(function(){
var a=document.getElementById('a'),s=document.getElementById('s'),log=[];
function t(x){log.push(x);if(log.length>8)log.shift();s.textContent=log.join('\n');}
var b64='__B64__';
function bytes(){var bin=atob(b64),u=new Uint8Array(bin.length);for(var i=0;i<bin.length;i++)u[i]=bin.charCodeAt(i);return u;}
t('rs='+a.readyState+' ns='+a.networkState);
a.addEventListener('loadedmetadata',function(){t('dur='+a.duration);});
a.addEventListener('error',function(){t('ERR code='+(a.error?a.error.code:'?')+' '+(a.error?a.error.message:''));});
a.addEventListener('play',function(){t('event play');});
a.addEventListener('pause',function(){t('event pause');});
var tickCount=0;a.addEventListener('timeupdate',function(){if(tickCount++<5)t('t='+a.currentTime.toFixed(2));});
document.getElementById('p').addEventListener('click',function(){
try{
var u=bytes();t('atob len='+u.length);
var b=new Blob([u],{type:'audio/ogg'});
var url=URL.createObjectURL(b);t('blob url dibikin');
a.src=url;
a.play().then(function(){t('OK paused='+a.paused+' muted='+a.muted+' vol='+a.volume);}).catch(function(e){t('REJECT '+e.name+': '+e.message);});
}catch(e){t('EXC '+e.name+': '+e.message);}
});
})();
</script>
"#;

/// Player mini pelapor-status: tombol PLAY + kotak hijau berisi event
/// audio (error code, hasil `play()`, tick waktu). `blob` = data URI
/// diconvert ke blob URL dulu; `edit` = edit susulan identik.
fn kartu_audio(huruf: &str, blob: bool, edit: bool) -> Reply {
    let blob_js = if blob {
        "fetch(a.src).then(function(r){return r.blob();}).then(function(b){a.src=URL.createObjectURL(b);t('blob dipasang');}).catch(function(e){t('fetch REJECT '+e.name);});"
    } else {
        ""
    };
    let html = AUDIO_PROBE
        .replace("__HURUF__", huruf)
        .replace("__B64__", AUDIO_UJI)
        .replace("__BLOB__", blob_js);
    kartu_l(huruf, html, |r| r.edit_susulan = edit)
}

const AUDIO_PROBE: &str = r#"<div style="font:700 18px sans-serif;padding:10px">KARTU __HURUF__ — uji audio</div>
<button id="p" style="font-size:20px;padding:16px 32px;margin:10px">▶ PLAY</button>
<div id="s" style="font:12px monospace;padding:10px;background:#000;color:#0f0;white-space:pre-wrap">init</div>
<audio id="a" preload="metadata" src="data:audio/ogg;base64,__B64__"></audio>
<script>
(function(){
var a=document.getElementById('a'),s=document.getElementById('s'),log=[];
function t(x){log.push(x);if(log.length>8)log.shift();s.textContent=log.join('\n');}
t('rs='+a.readyState+' ns='+a.networkState);
if(isFinite(a.duration))t('dur='+a.duration);
a.addEventListener('loadedmetadata',function(){t('dur='+a.duration);});
a.addEventListener('error',function(){t('ERR code='+(a.error?a.error.code:'?')+' '+(a.error?a.error.message:''));});
a.addEventListener('play',function(){t('event play');});
a.addEventListener('pause',function(){t('event pause');});
var tick=0;a.addEventListener('timeupdate',function(){if(tick++<5)t('t='+a.currentTime.toFixed(2));});
document.getElementById('p').addEventListener('click',function(){
t('klik');
a.play().then(function(){t('OK paused='+a.paused+' muted='+a.muted+' vol='+a.volume);}).catch(function(e){t('REJECT '+e.name+': '+e.message);});
});
__BLOB__
})();
</script>
"#;

/// Payload `player_html` produksi dengan data sintetis (poster placeholder,
/// lirik pendek) — generator yang sama persis dengan `.play2`.
fn player_html_uji(huruf: &str) -> String {
    let lirik = vec![
        Baris { time: 0.0, text: "Baris lirik pertama uji".into() },
        Baris { time: 12.5, text: "Baris lirik kedua uji".into() },
        Baris { time: 30.0, text: "Baris lirik ketiga uji".into() },
    ];
    player_html(&format!("Kartu {huruf}"), "Uji Coba", "3:45", "", &lirik, "")
}

/// Muatan ~30 KB teks (bentuk kartu G) untuk uji ukuran di amplop proven.
fn payload_besar(huruf: &str) -> String {
    let mut besar = judul(huruf);
    for i in 0..600 {
        besar.push_str(&format!(
            r#"<div style="font:13px sans-serif">baris uji {i} — 0123456789 abcdefghijklmnopqrstuvwxyz</div>"#
        ));
    }
    besar
}

/// Muatan teks yang dipadatkan sampai `target` byte — tangga ukuran U–Y.
fn payload_ukuran(huruf: &str, target: usize) -> String {
    let mut besar = judul(huruf);
    let mut i = 0;
    while besar.len() < target {
        besar.push_str(&format!(
            r#"<div style="font:13px sans-serif">baris uji {i} — 0123456789 abcdefghijklmnopqrstuvwxyz</div>"#
        ));
        i += 1;
    }
    besar
}

#[plugin(
    name = "cekkartu",
    alias = ["ujikartu", "airichtest"],
    category = "tools",
    description = "Diagnostik kartu HTML AIRich (30 variasi payload/amplop/ukuran/audio)",
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
    fn tiga_puluh_kartu_menguji_satu_variabel_per_kartu() {
        let r = semua_kartu();
        let list = cards(&r);
        assert_eq!(list.len(), 30, "A..AD");
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
        // O–T: semua amplop L proven (botJid 0@bot).
        for ai in &list[14..] {
            assert_eq!(ai.bot_jid.as_deref(), Some("0@bot"));
        }
        // O: combo 0@bot + edit susulan.
        assert!(list[14].edit_susulan, "O edit susulan");
        assert!(payload(list[14]).contains("KARTU O"));
        // P: payload player_html produksi (poster placeholder + lirik JSON
        // base64 untuk script player).
        let p = payload(list[15]);
        assert!(p.contains("<style>"), "P style");
        assert!(p.contains("data:image/svg+xml;base64,"), "P poster placeholder");
        let json_lirik = serde_json::to_string(&vec![
            json!({"time": 0.0, "text": "Baris lirik pertama uji"}),
            json!({"time": 12.5, "text": "Baris lirik kedua uji"}),
            json!({"time": 30.0, "text": "Baris lirik ketiga uji"}),
        ])
        .unwrap();
        assert!(p.contains(&base64_encode(json_lirik.as_bytes())), "P lirik base64");
        assert!(!list[15].edit_susulan, "P tanpa edit");
        // Q: cermin produksi = P + edit.
        assert_eq!(payload(list[16]), p.replace("Kartu P", "Kartu Q"), "Q payload sama dengan P");
        assert!(list[16].edit_susulan, "Q edit susulan");
        // R: ukuran ~30 KB.
        assert!(payload(list[17]).len() > 20_000, "R besar");
        // S: style tanpa img.
        let s = payload(list[18]);
        assert!(s.contains("<style>") && !s.contains("<img"), "S style saja");
        // T: img JPEG.
        assert!(payload(list[19]).contains("data:image/jpeg;base64,"), "T jpeg");
        // U–Y: tangga ukuran 60/128/256/512/768 KB.
        for (idx, target) in [
            (20, 60_000),
            (21, 128_000),
            (22, 256_000),
            (23, 512_000),
            (24, 768_000),
        ] {
            let len = payload(list[idx]).len();
            assert!(len >= target && len < target + 200, "kartu {idx}: {len} vs {target}");
        }
        // Z/AA/AB: probe audio pelapor-status.
        for idx in 25..28 {
            let p = payload(list[idx]);
            assert!(p.contains("data:audio/ogg;base64,"), "audio tertanam {idx}");
            assert!(p.contains("id=\"p\""), "tombol play {idx}");
            assert!(p.contains("REJECT"), "pelapor status {idx}");
        }
        assert!(!list[25].edit_susulan, "Z tanpa edit");
        assert!(list[26].edit_susulan, "AA edit susulan");
        assert!(payload(list[27]).contains("createObjectURL"), "AB blob");
        assert!(!payload(list[25]).contains("createObjectURL"), "Z data URI");
        // AC: WebAudio — tak ada URL media, decode dari atob.
        let ac = payload(list[28]);
        assert!(ac.contains("decodeAudioData"), "AC decode");
        assert!(ac.contains("AudioContext"), "AC context");
        assert!(!ac.contains("<audio"), "AC tanpa elemen audio");
        // AD: blob in-document dari atob (bukan fetch).
        let ad = payload(list[29]);
        assert!(ad.contains("new Blob"), "AD blob");
        assert!(ad.contains("createObjectURL"), "AD object URL");
        assert!(!ad.contains("fetch("), "AD tanpa fetch");
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