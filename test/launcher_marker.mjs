#!/usr/bin/env node
// Self-check: path marker `.ident` launcher HARUS sama dengan yang dibaca
// binary Rust (`PathBuf::with_extension("ident")` → `oktz-run.ident`, BUKAN
// `oktz-run.db.ident`). Kalau beda, `prepare_store` tidak melihat marker,
// db hasil restore dibuang, dan sesi ratchet hilang (Bad Mac).
//
// Jalankan: node test/launcher_marker.mjs
import assert from "node:assert/strict";
import path from "node:path";
import {
  pathMarker,
  pathDbSession,
  pathLogoutMarker,
  restoreTursoDiizinkan,
} from "../launcher.js";

// Semantik Rust: `with_extension("ident")` mengganti ekstensi `db`.
assert.equal(
  pathMarker("/app/database/wa/oktz-run.db"),
  "/app/database/wa/oktz-run.ident",
  "marker harus oktz-run.ident (bukan oktz-run.db.ident)"
);
assert.ok(
  !pathMarker("/app/database/wa/oktz-run.db").endsWith(".db.ident"),
  "marker tidak boleh oktz-run.db.ident"
);
// Nama tanpa `.db` tetap dapat sufiks `.ident`.
assert.equal(pathMarker("/app/x/session"), "/app/x/session.ident");

// Lokasi db session mengikuti OKTZ_DB_DIR.
const prev = process.env.OKTZ_DB_DIR;
process.env.OKTZ_DB_DIR = "/data/db";
assert.equal(pathDbSession("/app"), path.join("/data/db", "oktz-run.db"));
delete process.env.OKTZ_DB_DIR;
assert.equal(pathDbSession("/app"), path.join("/app", "database/wa", "oktz-run.db"));
if (prev !== undefined) process.env.OKTZ_DB_DIR = prev;

// Penanda logout: sesi yang baru logout TIDAK boleh dibangkitkan dari Turso.
assert.equal(
  pathLogoutMarker("/app/storage/session/creds.json"),
  "/app/storage/session/creds.json.loggedout"
);
assert.equal(
  restoreTursoDiizinkan({ credsSiap: false, markerLogout: false, tursoAktif: true }),
  true,
  "lokal kosong + belum logout + Turso aktif → restore"
);
assert.equal(
  restoreTursoDiizinkan({ credsSiap: false, markerLogout: true, tursoAktif: true }),
  false,
  "sudah logout → jangan restore (anti-zombie)"
);
assert.equal(
  restoreTursoDiizinkan({ credsSiap: true, markerLogout: false, tursoAktif: true }),
  false,
  "sesi lokal sehat → lokal menang"
);
assert.equal(
  restoreTursoDiizinkan({ credsSiap: false, markerLogout: false, tursoAktif: false }),
  false,
  "Turso tidak aktif → tidak ada restore"
);

console.log("launcher_marker: OK");
