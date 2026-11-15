# Perbaikan provider mati (JS→Rust port) — 2026-11-15

## Masalah (diverifikasi live)

| Upstream | Status | Plugin kena |
|---|---|---|
| `tikwm.com/api` | 403 | `.tt`, `.ttmp3` |
| `savett.cc` | halaman hidup, POST tidak diandalkan | fallback `.tt` |
| `kyzznekoo.zone.id/api/img/v2/nanobanana` | 404 | `.bananagen` |
| `api.yuulabs.web.id/api/ai/flux-img` | 404 | `.bananagen2` |
| `ihancer.com/api/enhance` | 500 | `.remini`/`.hd` |
| `api.nexray.eu.cc/random/anime` | 500 | `.waifu`, `.neko`, dll |
| `api.nexray.web.id/api/search/pinterest` | 404 | `.pap`, `.gachawaifu` (gambar) |
| `kyzznekoo.zone.id/api/asupan/paptt` | 404 | `.paptt` (dimatikan) |
| `api.waifu.im` | DNS mati | `.mywaifu` (di luar scope) |
| `i.pinimg.com/736x/8f3e2a...` | 403 | gambar kartu `.gachawaifu` |
| pool `src/tiktok/*.json` | 359 URL, sebagian mati | `.asupan`, `.bocil`, `.ukhty` |

## Provider pengganti

`apiii-xrina.my.id` (openapi.json, 365 route) — semua route dites live:
`/download/tiktok2`, `/search/pinterest?q=&limit=`, `/ai-image/nanobanana?image=&prompt=`,
`/ai-image/hd|hd2|unblur`, `/upload/uguu?url=`.
`nekos.best/api/v2/<tipe>` (wajib UA browser) + `api.otakugifs.xyz` untuk reaksi.
Upload: `uguu.se` (litterbox mati — `Invalid uploader`).

## Perubahan

1. `lib/oktz-scraper/rinn.rs` + `unggah.rs` (baru) — satu pintu provider.
2. `lib/oktz-scraper/tiktok.rs` — rinn jadi provider pertama di `ttdown`.
3. `plugin/src/download/ttmp3.rs`, `tiktok.rs` — tipe media dari `medias[].type`, bukan `durations`.
4. `plugin/src/ai/bananagen.rs` → rinn; `plugin/src/ai/nanobanana.rs` (baru).
5. `plugin/src/tools/hd.rs` — dari stub jadi implementasi.
6. `plugin/src/random/anime.rs` — nekos.best/otakugifs + peta 7 tipe mati.
7. `plugin/src/fun/gachawaifu.rs` — `imageUrl` segar dari rinn.
8. `plugin/src/search/pap.rs` — `fetch_urls` lewat rinn.
9. `plugin/src/asupan/{asupan,bocil,ukhty}.rs` — retry maks 5 URL.
10. `plugin/src/nsfw/paptt.rs` — `is_enabled = false`.

## Verifikasi

`cargo test -p oktz-plugins --lib` (per kategori) → `cargo clippy` → CI `cargo test --workspace` saat push.
Disk: `/home` cuma ~880MB; toolchain gcc diambil dari `/nix/store/...-gcc-wrapper-14.3.0/bin`.

## Rilis

Commit per grup → bump `launcher.js` `VERSI_BIN` → push `main` → tag `v0.2.52` (`.github/workflows/release.yml` bangun `oktz-linux-x64-static` + ffmpeg).
