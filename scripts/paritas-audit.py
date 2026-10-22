#!/usr/bin/env python3
"""Auditor paritas tampilan: bandingkan template output JS vs Rust per plugin.

Tujuan: memastikan BALASAN yang dilihat user identik byte-per-byte antara bot
JS (oktz-md) dan bot Rust (oktz-rs).

Cara kerja:
  1. Ekstrak semua string literal "user-visible" dari tiap plugin JS.
  2. Ekstrak semua string literal / format! dari plugin Rust.
  3. Normalisasi interpolasi (${x} di JS, {x} di Rust) jadi penanda sama.
  4. Bandingkan: fragmen yang ada di JS tapi hilang di Rust = PARITAS RUSAK.

_NORMALISASI_JS_PLACEHOLDER dan _NORMALISASI_RUST_PLACEHOLDER sengaja sama
supaya perbandingan tidak_rb membandingkan placeholder yang berbeda.
"""
import re, os, sys, json, difflib

AKAR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
JS_ROOT = os.path.join(AKAR, "oktz-md", "plugins")
RS_ROOT = os.path.join(AKAR, "plugin", "src")

PH = "␀"  # penanda interpolasi

# --- ekstraksi JS -----------------------------------------------------------

def js_strings(src):
    """Semua string literal (kutip tunggal/dua + template literal)."""
    out = []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if c in "\"'":
            q = c
            j = i + 1
            buf = []
            while j < n:
                if src[j] == "\\":
                    buf.append(src[j:j+2]); j += 2; continue
                if src[j] == q:
                    break
                buf.append(src[j]); j += 1
            out.append(("".join(buf), i))
            i = j + 1
            continue
        if c == "`":
            j = i + 1
            depth = 0
            buf = []
            while j < n:
                if src[j] == "\\":
                    buf.append(src[j:j+2]); j += 2; continue
                if src[j] == "$" and j + 1 < n and src[j+1] == "{":
                    depth = 1
                    buf.append("${")
                    j += 2
                    while j < n and depth:
                        if src[j] == "{": depth += 1
                        elif src[j] == "}": depth -= 1
                        buf.append(src[j]); j += 1
                    continue
                if src[j] == "`" and depth == 0:
                    break
                buf.append(src[j]); j += 1
            out.append(("".join(buf), i))
            i = j + 1
            continue
        i += 1
    return out

def unescape(s):
    """Balik escape literal Rust/JS supaya '\n' jadi newline sungguhan."""
    out = []
    i = 0
    while i < len(s):
        if s[i] == "\\" and i + 1 < len(s):
            c = s[i+1]
            out.append({"n": "\n", "t": "\t", "r": "\r", '"': '"', "\\": "\\", "0": "\0"}.get(c, c))
            i += 2
            continue
        out.append(s[i]); i += 1
    return "".join(out)

def norm_js(s):
    s = unescape(s)
    s = re.sub(r"\$\{[^}]*\}", PH, s)
    return s

def norm_rust(s):
    s = unescape(s)
    s = re.sub(r"\{[^{}]*\}", PH, s)
    return s

# string yang bukan "user-visible": import path, key config, nama fungsi
SKIP_JS = re.compile(
    r"^(?:@|\.{0,2}/|[a-z0-9_\-]+/[a-z0-9_\-./]+$)", re.I)

HEX_COLOR = re.compile(r"^#[0-9a-fA-F]{3,8}$")
MODULE_NAME = re.compile(r"^[a-z0-9@._\-/]+$")
IMPORTISH = re.compile(r"^(?:node-|\./|\.\./|[a-z@]+/[a-z0-9@._/-]+$)")

def noise(t):
    if HEX_COLOR.match(t):
        return True
    if IMPORTISH.match(t) and " " not in t:
        return True
    if re.fullmatch(r"[0-9a-fA-F]{16,}", t):
        return True
    return False

def js_visible(src):
    res = []
    for raw, pos in js_strings(src):
        t = norm_js(raw)
        t = t.strip()
        if len(t) < 2:
            continue
        if SKIP_JS.match(t):
            continue
        if t.startswith(("http", "application/", "text/", "image/")):
            continue
        # baris yang cuma berisi nama variabel / key
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", t):
            continue
        if noise(t):
            continue
        res.append(t)
    return res

# --- ekstraksi Rust ----------------------------------------------------------

def rust_visible(src):
    res = []
    # format!("...") dan string biasa
    for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', src, re.S):
        raw = m.group(1)
        t = norm_rust(raw).strip()
        if len(t) < 2:
            continue
        if t.startswith(("http", "./", "../", "application/", "text/", "image/")):
            continue
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", t):
            continue
        if noise(t):
            continue
        res.append(t)
    return res

# --- kumpulkan plugin -------------------------------------------------------

def js_plugins():
    res = {}
    for dirpath, _, files in os.walk(JS_ROOT):
        for f in files:
            if not f.endswith(".js"):
                continue
            p = os.path.join(dirpath, f)
            src = open(p, encoding="utf-8", errors="replace").read()
            m = re.search(r"(?:pluginConfig|const\s+config)\s*=\s*\{", src)
            if not m:
                continue
            i = src.index("{", m.start())
            depth = 0
            for j in range(i, len(src)):
                if src[j] == "{":
                    depth += 1
                elif src[j] == "}":
                    depth -= 1
                    if depth == 0:
                        break
            body = src[i:j+1]
            nm = re.search(r"\bname\s*:\s*(\[[^\]]*\]|[\"'][^\"']*[\"'])", body)
            if not nm:
                continue
            raw = nm.group(1)
            names = re.findall(r"[\"']([^\"']+)[\"']", raw) if raw.startswith("[") \
                else [raw.strip("\"'")]
            cat = re.search(r"\bcategory\s*:\s*[\"']([^\"']+)[\"']", body)
            res[p] = {
                "names": names,
                "category": cat.group(1) if cat else "?",
                "texts": js_visible(src),
            }
    return res

def rust_plugins():
    res = {}
    for dirpath, _, files in os.walk(RS_ROOT):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(dirpath, f)
            src = open(p, encoding="utf-8", errors="replace").read()
            for m in re.finditer(r"#\[plugin\((.*?)\)\]\s*(?:pub\s+)?fn\s+(\w+)", src, re.S):
                args = m.group(1)
                nm = re.search(r'name\s*=\s*"([^"]+)"', args)
                if not nm:
                    continue
                cat = re.search(r'category\s*=\s*"([^"]*)"', args)
                res[nm.group(1)] = {
                    "file": p,
                    "category": cat.group(1) if cat else "?",
                    "texts": rust_visible(src),
                }
    return res

def norm_lines(texts):
    """Pecah jadi baris, buang yang cuma placeholder."""
    lines = set()
    for t in texts:
        for ln in t.split("\n"):
            ln = ln.strip()
            if not ln:
                continue
            if ln == PH:
                continue
            lines.add(ln)
    return lines

def main():
    js = js_plugins()
    rs = rust_plugins()
    rs_names = set(rs)
    rs_alias = set()
    for v in rs.values():
        rs_alias.update(v["aliases"] if "aliases" in v else [])

    # alias rust -> nama plugin
    alias_ke_nama = {}
    for dirpath, _, files in os.walk(RS_ROOT):
        for f in files:
            if not f.endswith(".rs"):
                continue
            p = os.path.join(dirpath, f)
            src = open(p, encoding="utf-8", errors="replace").read()
            for m in re.finditer(r"#\[plugin\((.*?)\)\]\s*(?:pub\s+)?fn\s+(\w+)", src, re.S):
                a = re.search(r'alias\s*=\s*\[([^\]]*)\]', m.group(1))
                nm = re.search(r'name\s*=\s*"([^"]+)"', m.group(1))
                if a and nm:
                    for al in re.findall(r'"([^"]*)"', a.group(1)):
                        alias_ke_nama.setdefault(al, nm.group(1))
                        rs_alias.add(al)

    laporan = []
    for p, info in sorted(js.items()):
        if not info["names"]:
            continue
        nama = info["names"][0]
        pasangan = None
        for cand in info["names"]:
            if cand in rs_names:
                pasangan = cand
                break
        if pasangan is None:
            for cand in info["names"]:
                if cand in alias_ke_nama:
                    pasangan = alias_ke_nama[cand]
                    break
        if pasangan is None or pasangan not in rs:
            laporan.append({
                "tipe": "HILANG", "js": p, "nama": nama,
                "kategori": info["category"], "hilang": [], "beda": [],
            })
            continue
        jsl = norm_lines(info["texts"])
        rsl = norm_lines(rs[pasangan]["texts"])
        hilang = sorted(jsl - rsl)
        beda = []
        for ln in hilang:
            cand = difflib.get_close_matches(ln, sorted(rsl), n=1, cutoff=0.72)
            beda.append({"js": ln, "rust_mirip": cand[0] if cand else None})
        laporan.append({
            "tipe": "ADA" if not hilang else "BEDA",
            "js": p, "nama": nama, "rust": rs[pasangan]["file"],
            "kategori": info["category"],
            "hilang": hilang, "beda": beda,
        })

    out = os.path.join(AKAR, "docs", "paritas")
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "paritas.json"), "w") as f:
        json.dump(laporan, f, indent=1, ensure_ascii=False)

    # ringkas per kategori
    per_kat = {}
    for e in laporan:
        k = per_kat.setdefault(e["kategori"], {"HILANG": [], "BEDA": [], "ADA": []})
        k[e["tipe"]].append(e)
    with open(os.path.join(out, "ringkas.md"), "w") as f:
        f.write("# Ringkasan paritas JS vs Rust\n\n")
        f.write(f"Total plugin JS diperiksa: {len(laporan)}\n\n")
        f.write(f"- HILANG (tidak ada di Rust): {sum(len(v['HILANG']) for v in per_kat.values())}\n")
        f.write(f"- BEDA (baris output beda): {sum(len(v['BEDA']) for v in per_kat.values())}\n")
        f.write(f"- ADA (tidak ada baris hilang): {sum(len(v['ADA']) for v in per_kat.values())}\n\n")
        f.write("| kategori | hilang | beda | ada |\n|---|---|---|---|\n")
        for k in sorted(per_kat):
            v = per_kat[k]
            f.write(f"| {k} | {len(v['HILANG'])} | {len(v['BEDA'])} | {len(v['ADA'])} |\n")
        f.write("\n## Plugin dengan baris output berbeda\n\n")
        for e in laporan:
            if e["tipe"] != "BEDA":
                continue
            f.write(f"### {e['nama']} ({e['kategori']})\n")
            f.write(f"- JS: `{e['js']}`\n- Rust: `{e.get('rust','-')}`\n\n")
            for d in e["beda"]:
                f.write(f"  - JS: `{d['js']}`\n")
                if d["rust_mirip"]:
                    f.write(f"    Rust: `{d['rust_mirip']}`\n")
                else:
                    f.write(f"    Rust: (tidak ada padanan)\n")
            f.write("\n")
    print(f"JS: {len(js)} plugin, Rust: {len(rs)} plugin")
    print(f"HILANG: {sum(len(v['HILANG']) for v in per_kat.values())}")
    print(f"BEDA:   {sum(len(v['BEDA']) for v in per_kat.values())}")
    print(f"ADA:    {sum(len(v['ADA']) for v in per_kat.values())}")
    print(f"-> {out}/ringkas.md")

if __name__ == "__main__":
    main()
