//! `plugins/sticker/sticker.js` — ubah gambar/video jadi sticker dengan opsi
//! crop/resize/lingkaran/melengkung.
//!
//! Yang diport apa adanya:
//! - Kartu pakai lengkap (`sticker.js:173-190`).
//! - Parser opsi `--crop/-c`, `--resize/-r WxH`, `--circle`, `--rounded`,
//!   plus dua argumen bebas sebagai packname/author (`sticker.js:53-84`).
//! - Rantai filter ffmpeg untuk gambar dan video (`sticker.js:86-141`),
//!   termasuk filter `geq` yang panjang — disalin apa adanya.
//! - Pesan "video terlalu panjang" (`sticker.js:247`), pesan gagal unduh
//!   (`sticker.js:228`), dan peringatan "efek tidak bisa diterapkan"
//!   (`sticker.js:306-310`).
//! - Reaksi penutup `✅` / `❌` / `☢`.
//!
//! `ponytail:`
//! - JS menulis berkas sementara lalu memanggil `exec`; di sini video/gambar
//!   dialirkan lewat pipe ffmpeg (`-i pipe:0 ... pipe:1`), jadi tidak ada
//!   berkas di disk.
//! - Pemeriksaan durasi video memakai `ffprobe` (`sticker.js:243`). `ffprobe`
//!   tidak dibungkus di `oktz_plugin_api::ffmpeg`, jadi durasi dibaca dari
//!   keluaran stderr ffmpeg. Kalau tidak terbaca, pemeriksaan dilewati —
//!   sama seperti `catch (e) {}` di JS.
//! - `sock.sendStickerPack` untuk album (`sticker.js:205`) tidak ada di
//!   `Reply`, jadi album multi-media tidak dikirim.
//! - `sock.sendImageAsSticker`/`sendVideoAsSticker` diganti `Reply::Sticker`.

use oktz_plugin_api::registry::IncomingKind;
use oktz_plugin_api::{Ctx, MediaSource, Reply, plugin};

/// `sticker.js:173-190` — kartu pakai.
pub(crate) fn pakai(prefix: &str) -> String {
    format!(
        "🖼️ *sᴛɪᴄᴋᴇʀ ᴍᴀᴋᴇʀ*\n\n\
         Kirim/reply gambar atau video dengan caption:\n\
         `{prefix}s`\n\n\
         *ᴏᴘsɪ:*\n\
         > `--crop` - Crop jadi kotak\n\
         > `--resize WxH` - Resize ke ukuran\n\
         > `--circle` - Bentuk lingkaran\n\
         > `--rounded` - Sudut melengkung\n\n\
         *ᴄᴏɴᴛᴏʛ:*\n\
         > `{prefix}s --crop`\n\
         > `{prefix}s --resize 256x256`\n\
         > `{prefix}s --circle`\n\
         > `{prefix}s --rounded`\n\n\
         *ᴀʟʙᴜᴍ:*\n\
         > Reply pesan foto/video yang banyak → semua jadi sticker dalam 1 pesan"
    )
}

/// `sticker.js:228` — media gagal diunduh.
pub(crate) const GAGAL_UNDUH: &str = "❌ Gagal mendownload media!";

/// `sticker.js:247` — video lebih dari 10 detik.
pub(crate) fn terlalu_panjang(durasi: f64) -> String {
    format!(
        "❌ Video terlalu panjang!\n\n> Durasi: {} detik\n> Maksimal: 10 detik",
        format!("{durasi:.1}")
    )
}

/// `sticker.js:306-310` — efek gagal diproses.
pub(crate) const EFEK_GAGAL: &str = "⚠️ *Efek tidak bisa diterapkan*\n\n\
     > Sticker sudah dikirim, tapi efek yang kakak minta gagal diproses.\n\
     > _Sebaiknya cek format/ukuran sumbernya ya._";

/// Opsi hasil `parseOptions` (`sticker.js:53-84`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Opsi {
    pub crop: bool,
    pub resize: Option<String>,
    pub circle: bool,
    pub rounded: bool,
    pub packname: Option<String>,
    pub author: Option<String>,
}

impl Opsi {
    /// `sticker.js:196` — apakah ada efek yang perlu diproses.
    pub(crate) fn ada_efek(&self) -> bool {
        self.crop || self.resize.is_some() || self.circle || self.rounded
    }
}

/// `parseOptions` (`sticker.js:53-84`) — tanpa regex crate.
pub(crate) fn parse_opsi(args: &[String]) -> Opsi {
    let mut opsi = Opsi::default();
    let mut i = 0usize;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--crop" || arg == "-c" {
            opsi.crop = true;
        } else if arg == "--resize" || arg == "-r" {
            if let Some(next) = args.get(i + 1)
                && is_ukuran(next)
            {
                opsi.resize = Some(next.clone());
                i += 1;
            }
        } else if arg == "--circle" {
            opsi.circle = true;
        } else if arg == "--rounded" {
            opsi.rounded = true;
        } else if !arg.starts_with('-') && opsi.packname.is_none() {
            opsi.packname = Some(arg.to_string());
        } else if !arg.starts_with('-') && opsi.packname.is_some() && opsi.author.is_none() {
            opsi.author = Some(arg.to_string());
        }
        i += 1;
    }
    opsi
}

/// `/^\d+x\d+$/i` (`sticker.js:68`).
fn is_ukuran(raw: &str) -> bool {
    let (a, b) = match raw.split_once('x').or_else(|| raw.split_once('X')) {
        Some(p) => p,
        None => return false,
    };
    !a.is_empty()
        && !b.is_empty()
        && a.chars().all(|c| c.is_ascii_digit())
        && b.chars().all(|c| c.is_ascii_digit())
}

/// Filter `geq` untuk lingkaran (`sticker.js:101-102`).
const FILTER_LINGKARAN: &str = "format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(gt(pow(X-W/2,2)+pow(Y-H/2,2),pow(min(W,H)/2,2)),0,255)'";

/// Filter `geq` untuk sudut melengkung (`sticker.js:107-108`).
const FILTER_MELENGKUNG: &str = "format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(lt(X,50)*lt(Y,50)*gt(pow(50-X,2)+pow(50-Y,2),pow(50,2)),0,if(gt(X,W-50)*lt(Y,50)*gt(pow(X-W+50,2)+pow(50-Y,2),pow(50,2)),0,if(lt(X,50)*gt(Y,H-50)*gt(pow(50-X,2)+pow(Y-H+50,2),pow(50,2)),0,if(gt(X,W-50)*gt(Y,H-50)*gt(pow(X-W+50,2)+pow(Y-H+50,2),pow(50,2)),0,255))))'";

/// `processImage` (`sticker.js:86-118`) — rantai filter untuk gambar.
fn filter_gambar(opsi: &Opsi) -> Vec<String> {
    let mut filters = Vec::new();
    if let Some(ukuran) = &opsi.resize {
        let (lebar, tinggi) = ukuran.split_once('x').or_else(|| ukuran.split_once('X')).unwrap_or((ukuran, ukuran));
        filters.push(format!(
            "scale={lebar}:{tinggi}:force_original_aspect_ratio=decrease"
        ));
        filters.push(format!(
            "pad={lebar}:{tinggi}:(ow-iw)/2:(oh-ih)/2:color=0x00000000"
        ));
    }
    if opsi.crop {
        filters.push("crop='min(iw,ih)':'min(iw,ih)'".to_string());
        filters.push("scale=512:512".to_string());
    }
    if opsi.circle {
        filters.push(FILTER_LINGKARAN.to_string());
    }
    if opsi.rounded {
        filters.push(FILTER_MELENGKUNG.to_string());
    }
    filters
}

/// `processVideo` (`sticker.js:120-141`) — rantai filter untuk video.
fn filter_video(opsi: &Opsi) -> Vec<String> {
    let mut filters = Vec::new();
    if let Some(ukuran) = &opsi.resize {
        let (lebar, tinggi) = ukuran.split_once('x').or_else(|| ukuran.split_once('X')).unwrap_or((ukuran, ukuran));
        filters.push(format!(
            "scale={lebar}:{tinggi}:force_original_aspect_ratio=decrease"
        ));
        filters.push(format!(
            "pad={lebar}:{tinggi}:(ow-iw)/2:(oh-ih)/2:color=black"
        ));
    }
    if opsi.crop {
        filters.push("crop='min(iw,ih)':'min(iw,ih)'".to_string());
        filters.push("scale=512:512".to_string());
    }
    filters
}

/// Jalankan ffmpeg untuk gambar → PNG.
fn proses_gambar(input: &[u8], opsi: &Opsi) -> Option<Vec<u8>> {
    let filters = filter_gambar(opsi);
    if filters.is_empty() {
        return None;
    }
    let argumen = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        "pipe:0".to_string(),
        "-vf".to_string(),
        filters.join(","),
        "-f".to_string(),
        "image2".to_string(),
        "-vcodec".to_string(),
        "png".to_string(),
        "pipe:1".to_string(),
    ];
    let refs: Vec<&str> = argumen.iter().map(String::as_str).collect();
    oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&refs, input).ok()
}

/// Jalankan ffmpeg untuk video → MP4.
fn proses_video(input: &[u8], opsi: &Opsi) -> Option<Vec<u8>> {
    let filters = filter_video(opsi);
    if filters.is_empty() {
        return None;
    }
    let argumen = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "error".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        "pipe:0".to_string(),
        "-vf".to_string(),
        filters.join(","),
        "-c:a".to_string(),
        "copy".to_string(),
        "-movflags".to_string(),
        "frag_keyframe+empty_moov".to_string(),
        "-f".to_string(),
        "mp4".to_string(),
        "pipe:1".to_string(),
    ];
    let refs: Vec<&str> = argumen.iter().map(String::as_str).collect();
    oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&refs, input).ok()
}

/// Durasi video dalam detik — dibaca dari keluaran ffmpeg (`Duration: HH:MM:SS.ss`).
///
/// `None` kalau tidak terbaca; pemanggil lalu melewati pemeriksaandurasi,
/// sama seperti `catch (e) {}` pada `ffprobe` di JS (`sticker.js:252`).
pub(crate) fn durasi_video(video: &[u8]) -> Option<f64> {
    let argumen = [
        "-hide_banner",
        "-i",
        "pipe:0",
        "-f",
        "null",
        "-",
    ];
    let keluaran = oktz_plugin_api::ffmpeg::jalankan_dengan_masukan(&argumen, video);
    let teks = match keluaran {
        Ok(_) => return None,
        Err(e) => e,
    };
    let posisi = teks.find("Duration:")?;
    let sisanya = &teks[posisi + "Duration:".len()..];
    let akhir = sisanya.find(',')?;
    let jam = sisanya[..akhir].trim();
    let mut total = 0.0f64;
    for (indeks, bagian) in jam.split(':').enumerate() {
        let nilai: f64 = bagian.trim().parse().ok()?;
        let pengali = match indeks {
            0 => 3600.0,
            1 => 60.0,
            _ => 1.0,
        };
        total += nilai * pengali;
    }
    Some(total)
}

#[plugin(
    name = "sticker",
    alias = ["s", "stiker", "stickergif"],
    category = "sticker",
    description = "Membuat sticker dari gambar/video dengan opsi crop/resize",
    usage = ".s [--crop] [--resize WxH] [--circle] [--rounded]",
    example = ".s --crop\n.s --resize 256x256\n.s --circle\n.s --rounded",
    cooldown = 5,
    energi = 1,
    is_group = false,
    is_private = false,
    is_owner = false,
    is_premium = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn sticker(ctx: Ctx) -> Reply {
    // `sticker.js:169-170` — gambar/video di pesan pemicu atau yang di-reply.
    let quoted = ctx.quoted();
    let is_image = ctx.is_media_kind(IncomingKind::Image)
        || quoted.is_some_and(|q| q.kind.is_image() || q.kind.wire_type() == "imageMessage");
    let is_video = ctx.is_media_kind(IncomingKind::Video)
        || quoted.is_some_and(|q| q.kind.is_video() || q.kind.wire_type() == "videoMessage");
    if !is_image && !is_video {
        return Reply::text(pakai(&ctx.prefix));
    }

    // `sticker.js:221-225` — media yang di-reply didahulukan kalau media.
    let media = if quoted.is_some_and(|q| q.is_media()) {
        ctx.download_quoted_media().ok()
    } else if ctx.is_media() {
        ctx.download_media().ok()
    } else {
        None
    };
    let Some(mut buffer) = media else {
        return Reply::text(GAGAL_UNDUH).with_reaction("❌");
    };

    // `sticker.js:233-254` — cek durasi video.
    if is_video {
        if let Some(durasi) = durasi_video(&buffer)
            && durasi > 10.0
        {
            return Reply::text(terlalu_panjang(durasi)).with_reaction("☢");
        }
    }

    // `sticker.js:195` — opsi efek.
    let opsi = parse_opsi(&ctx.args);
    let mut diproses = true;
    if opsi.ada_efek() {
        let hasil = if is_image {
            proses_gambar(&buffer, &opsi)
        } else {
            proses_video(&buffer, &opsi)
        };
        match hasil {
            Some(hasil) => buffer = hasil,
            // `sticker.js:284-289` — ffmpeg gagal: buffer asli tetap dikirim.
            None => diproses = false,
        }
    }

    // `sticker.js:296-302` — kirim sticker, lalu centang hijau.
    let balasan = Reply::sticker(MediaSource::bytes(buffer.clone()));

    // `sticker.js:304-311` — efek gagal: tetap kirim sticker + peringatan.
    // `ponytail:` `Reply::Sequence` tidak boleh berisi `Reaction` (pipeline
    // merender tiap elemen_sequence lewat `render_reply`, yang menolak
    // `Reaction`), jadi pada jalur ini reaksi ✅ tidak ikut.
    if opsi.ada_efek() && !diproses {
        return Reply::sequence(vec![balasan, Reply::text(EFEK_GAGAL)]);
    }
    balasan.with_reaction("✅")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::IncomingMedia;

    const ME: &str = "628111@s.whatsapp.net";

    fn ctx_of(gambar: bool, args: &[&str]) -> Ctx {
        Ctx {
            text: args.join(" "),
            args: args.iter().map(|a| (*a).to_string()).collect(),
            sender: ME.into(),
            chat: ME.into(),
            prefix: ".".into(),
            command: "s".into(),
            media: gambar.then(|| IncomingMedia {
                kind: IncomingKind::Image,
                mimetype: Some("image/png".into()),
                file_name: None,
                caption: String::new(),
            }),
            ..Ctx::default()
        }
    }

    fn teks(r: &Reply) -> String {
        r.without_mutations().text_body().unwrap_or("").to_string()
    }

    fn reaksi(r: &Reply) -> Option<&str> {
        match r.without_mutations() {
            Reply::Reaction { emoji, .. } => Some(emoji.as_str()),
            _ => None,
        }
    }

    fn args_dari(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_string()).collect()
    }

    #[test]
    fn output_sama_dengan_js() {
        assert_eq!(GAGAL_UNDUH, "❌ Gagal mendownload media!");
        assert_eq!(
            terlalu_panjang(12.34),
            "❌ Video terlalu panjang!\n\n> Durasi: 12.3 detik\n> Maksimal: 10 detik"
        );
        assert_eq!(
            EFEK_GAGAL,
            "⚠️ *Efek tidak bisa diterapkan*\n\n> Sticker sudah dikirim, tapi efek yang kakak minta gagal diproses.\n> _Sebaiknya cek format/ukuran sumbernya ya._"
        );
    }

    #[test]
    fn meta_sama_dengan_js() {
        let p = oktz_plugin_api::find("sticker").unwrap();
        assert_eq!(p.meta.name, "sticker");
        assert_eq!(p.meta.alias, ["s", "stiker", "stickergif"]);
        assert_eq!(p.meta.category, "sticker");
        assert_eq!(
            p.meta.description,
            "Membuat sticker dari gambar/video dengan opsi crop/resize"
        );
        assert_eq!(p.meta.usage, ".s [--crop] [--resize WxH] [--circle] [--rounded]");
        assert_eq!(
            p.meta.example,
            ".s --crop\n.s --resize 256x256\n.s --circle\n.s --rounded"
        );
        assert_eq!(p.meta.cooldown, 5);
        assert_eq!(p.meta.energi, 1);
        assert!(!p.meta.is_group && !p.meta.is_private);
        assert!(!p.meta.is_owner && !p.meta.is_premium);
    }

    #[test]
    fn setiap_alias_menunjuk_plugin_ini() {
        for a in ["s", "stiker", "stickergif"] {
            assert_eq!(oktz_plugin_api::find(a).unwrap().meta.name, "sticker");
        }
    }

    #[test]
    fn parse_opsi_menurut_js() {
        let o = parse_opsi(&args_dari(&["--crop"]));
        assert!(o.crop && !o.circle && !o.rounded && o.resize.is_none());

        let o = parse_opsi(&args_dari(&["-c"]));
        assert!(o.crop);

        let o = parse_opsi(&args_dari(&["--resize", "256x256"]));
        assert_eq!(o.resize.as_deref(), Some("256x256"));
        assert!(o.ada_efek());

        let o = parse_opsi(&args_dari(&["-r", "300x200"]));
        assert_eq!(o.resize.as_deref(), Some("300x200"));

        // Ukuran tidak valid → opsi diabaikan, argumen berikutnya tetap diproses.
        let o = parse_opsi(&args_dari(&["--resize", "abc"]));
        assert!(o.resize.is_none());

        let o = parse_opsi(&args_dari(&["--circle", "--rounded"]));
        assert!(o.circle && o.rounded);

        // Dua argumen bebas pertama = packname, kedua = author.
        let o = parse_opsi(&args_dari(&["Foto-AI", "Lucky", "--crop"]));
        assert_eq!(o.packname.as_deref(), Some("Foto-AI"));
        assert_eq!(o.author.as_deref(), Some("Lucky"));
        assert!(o.crop);
    }

    #[test]
    fn filter_gambar_sesuai_js() {
        let o = parse_opsi(&args_dari(&["--resize", "256x256"]));
        let f = filter_gambar(&o);
        assert_eq!(
            f,
            vec![
                "scale=256:256:force_original_aspect_ratio=decrease".to_string(),
                "pad=256:256:(ow-iw)/2:(oh-ih)/2:color=0x00000000".to_string(),
            ]
        );

        let o = parse_opsi(&args_dari(&["--crop"]));
        assert_eq!(
            filter_gambar(&o),
            vec![
                "crop='min(iw,ih)':'min(iw,ih)'".to_string(),
                "scale=512:512".to_string(),
            ]
        );

        let o = parse_opsi(&args_dari(&["--circle"]));
        assert!(filter_gambar(&o)[0].starts_with("format=rgba,geq=r='r(X,Y)'"));
    }

    #[test]
    fn filter_video_menggunakan_pad_hitam() {
        let o = parse_opsi(&args_dari(&["--resize", "256x256"]));
        let f = filter_video(&o);
        assert_eq!(
            f[1],
            "pad=256:256:(ow-iw)/2:(oh-ih)/2:color=black".to_string()
        );
        // Lingkaran/melengkung hanya berlaku untuk gambar.
        let o = parse_opsi(&args_dari(&["--circle"]));
        assert!(filter_video(&o).is_empty());
    }

    #[test]
    fn tanpa_media_menampilkan_kartu_pakai() {
        let r = sticker(ctx_of(false, &[]));
        assert_eq!(teks(&r), pakai("."));
        assert_eq!(reaksi(&r), None);
    }

    #[test]
    fn media_gagal_diunduh_menampilkan_pesan_gagal() {
        let r = sticker(ctx_of(true, &[]));
        assert_eq!(teks(&r), GAGAL_UNDUH);
        assert_eq!(reaksi(&r), Some("❌"));
    }
}
