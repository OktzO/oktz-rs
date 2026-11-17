// Port `extractIE` — builder.js:40-212.
use serde_json::{Map, Value, json};
use crate::rich::{AiSubMessage, RichAi};

/// Flag jenis inline entity yang diekstrak (default semua aktif).
#[derive(Debug, Clone, Copy)]
pub struct Flag {
    pub extract: bool,
    pub hyperlink: bool,
    pub citation: bool,
    pub latex: bool,
}

impl Default for Flag {
    fn default() -> Self {
        Self { extract: true, hyperlink: true, citation: true, latex: true }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct HasilEkstrak {
    pub teks: String,
    pub entities: Vec<Value>,
}

/// `JSON.stringify` JS: bilangan bulat tanpa `.0`.
fn angka_js(v: f64) -> Value {
    if v.fract() == 0.0 && v.abs() <= 9_007_199_254_740_992.0 {
        Value::from(v as i64)
    } else {
        serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
    }
}

/// `Number(x) || default` JS (builder.js:83-87).
fn angka(s: &str, def: f64) -> f64 {
    s.trim().parse::<f64>().ok().filter(|v| *v != 0.0 && v.is_finite()).unwrap_or(def)
}

pub fn extract_ie(teks: &str, flag: &Flag) -> HasilEkstrak {
    if !flag.extract {
        // builder.js:41-47 — teks apa adanya.
        return HasilEkstrak { teks: teks.to_string(), entities: Vec::new() };
    }

    let chars: Vec<char> = teks.chars().collect();
    let n = chars.len();
    let mut teks_hasil = String::new();
    let mut entities = Vec::new();
    let mut last = 0usize;
    let mut citation_index = 1u64;
    let mut hyperlink_index = 0usize;
    let mut latex_index = 0usize;
    let mut stack: Vec<usize> = Vec::new();
    let mut i = 0usize;

    while i < n {
        if chars[i] == '[' && (i == 0 || chars[i - 1] != '\\') {
            stack.push(i);
        } else if chars[i] == ']' && i + 1 < n && (chars[i + 1] == '(' || chars[i + 1] == '<') {
            // builder.js:107-109
            let Some(start) = stack.pop() else { i += 1; continue };

            let open = chars[i + 1];
            let close = if open == '(' { ')' } else { '>' };
            let mut end = i + 2;
            let mut depth = 1i32;
            while end < n && depth > 0 {
                if chars[end] == open && chars[end - 1] != '\\' {
                    depth += 1;
                } else if chars[end] == close && chars[end - 1] != '\\' {
                    depth -= 1;
                }
                end += 1;
            }
            if depth != 0 {
                i += 1;
                continue;
            }

            // builder.js:125-126
            let raw = potong(&chars, start + 1, i).trim().to_string();
            let mut url = potong(&chars, i + 2, end - 1).trim().to_string();

            let key: String;
            let tag: String;
            let entity: Value;

            if open == '<' {
                // builder.js:132-151
                if !flag.latex {
                    i += 1;
                    continue;
                }
                let bagian: Vec<&str> = raw.split('|').collect();
                let ambil = |k: usize| bagian.get(k).copied().unwrap_or("");
                let txt = ambil(0);
                key = format!("NIXEL_LATEX_{latex_index}");
                latex_index += 1;
                let isi = if txt.is_empty() { "image" } else { txt };
                tag = format!("{{{{{key}}}}}{isi}{{{{/{key}}}}}");
                entity = json!({
                    "key": key,
                    "metadata": {
                        "latex_expression": txt,
                        "latex_image": {
                            "url": url,
                            "width": angka_js(angka(ambil(1), 100.0)),
                            "height": angka_js(angka(ambil(2), 100.0)),
                        },
                        "font_height": angka_js(angka(ambil(3), 83.333333333333)),
                        "padding": angka_js(angka(ambil(4), 15.0)),
                        "__typename": "GenAILatexItem",
                    },
                });
            } else if !raw.is_empty() {
                // builder.js:152-172
                if !flag.hyperlink {
                    i += 1;
                    continue;
                }
                let trusted = !url.starts_with('!');
                if !trusted {
                    url = url[1..].to_string();
                }
                key = format!("NIXEL_HYPERLINK_{hyperlink_index}");
                hyperlink_index += 1;
                tag = format!("{{{{{key}}}}}{url}{{{{/{key}}}}}");
                entity = json!({
                    "key": key,
                    "metadata": {
                        "display_name": raw,
                        "is_trusted": trusted,
                        "url": url,
                        "__typename": "GenAIInlineLinkItem",
                    },
                });
            } else {
                // builder.js:173-188
                if !flag.citation {
                    i += 1;
                    continue;
                }
                key = format!("NIXEL_CITATION_{}", citation_index - 1);
                tag = format!("{{{{{key}}}}}{url}{{{{/{key}}}}}");
                entity = json!({
                    "key": key,
                    "metadata": {
                        "reference_id": citation_index,
                        "reference_url": url,
                        "reference_title": url,
                        "reference_display_name": url,
                        "sources": [],
                        "__typename": "GenAISearchCitationItem",
                    },
                });
                citation_index += 1;
            }

            // builder.js:190-201
            teks_hasil.push_str(&potong(&chars, last, start));
            teks_hasil.push_str(&tag);
            last = end;
            entities.push(entity);
            i = end - 1;
        }
        i += 1;
    }

    teks_hasil.push_str(&potong(&chars, last, chars.len()));
    HasilEkstrak { teks: teks_hasil, entities }
}

fn potong(chars: &[char], a: usize, b: usize) -> String {
    chars[a..b].iter().collect()
}

// Port `tokenizer` — builder.js:1540-2203.

/// Jenis highlight token (padanan `TYPE_MAP` builder.js:2042-2049).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    Default,
    Keyword,
    Method,
    Str,
    Number,
    Comment,
}

impl Highlight {
    pub fn nama(&self) -> &'static str {
        match self {
            Highlight::Default => "DEFAULT",
            Highlight::Keyword => "KEYWORD",
            Highlight::Method => "METHOD",
            Highlight::Str => "STR",
            Highlight::Number => "NUMBER",
            Highlight::Comment => "COMMENT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KodeToken {
    pub isi: String,
    pub jenis: Highlight,
}

/// 12 set kata kunci — disalin persis dari builder.js:1541-2022.
fn kata_kunci(lang_lower: &str) -> &'static [&'static str] {
    match lang_lower {
        "javascript" => &[
            "break", "case", "catch", "continue", "debugger", "delete", "do", "else", "finally",
            "for", "function", "if", "in", "instanceof", "new", "return", "switch", "this",
            "throw", "try", "typeof", "var", "void", "while", "with", "true", "false", "null",
            "undefined", "class", "const", "let", "super", "extends", "export", "import", "yield",
            "static", "constructor", "async", "await", "get", "set",
        ],
        "typescript" => &[
            "abstract", "any", "as", "asserts", "bigint", "boolean", "declare", "enum",
            "implements", "infer", "interface", "is", "keyof", "module", "namespace", "never",
            "readonly", "require", "number", "object", "override", "private", "protected",
            "public", "satisfies", "string", "symbol", "type", "unknown", "using", "from",
            "break", "case", "catch", "continue", "do", "else", "finally", "for", "function",
            "if", "new", "return", "switch", "this", "throw", "try", "var", "void", "while",
            "class", "const", "let", "extends", "import", "export", "async", "await",
        ],
        "python" => &[
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from",
            "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass",
            "raise", "return", "try", "while", "with", "yield",
        ],
        "java" => &[
            "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
            "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
            "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
            "interface", "long", "native", "new", "package", "private", "protected", "public",
            "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
            "throw", "throws", "transient", "try", "void", "volatile", "while",
        ],
        "golang" => &[
            "break", "case", "chan", "const", "continue", "default", "defer", "else",
            "fallthrough", "for", "func", "go", "goto", "if", "import", "interface", "map",
            "package", "range", "return", "select", "struct", "switch", "type", "var",
        ],
        "c" => &[
            "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
            "else", "enum", "extern", "float", "for", "goto", "if", "int", "long", "register",
            "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
            "union", "unsigned", "void", "volatile", "while",
        ],
        "cpp" => &[
            "alignas", "alignof", "and", "auto", "bool", "break", "case", "catch", "class",
            "const", "constexpr", "continue", "delete", "do", "double", "else", "enum",
            "explicit", "export", "extern", "false", "float", "for", "friend", "if", "inline",
            "int", "long", "mutable", "namespace", "new", "noexcept", "nullptr", "operator",
            "private", "protected", "public", "return", "short", "signed", "sizeof", "static",
            "struct", "switch", "template", "this", "throw", "true", "try", "typedef",
            "typename", "union", "unsigned", "using", "virtual", "void", "while",
        ],
        "php" => &[
            "abstract", "and", "array", "as", "break", "callable", "case", "catch", "class",
            "clone", "const", "continue", "declare", "default", "do", "echo", "else", "elseif",
            "empty", "enddeclare", "endfor", "endforeach", "endif", "endswitch", "endwhile",
            "extends", "final", "finally", "fn", "for", "foreach", "function", "global", "goto",
            "if", "implements", "include", "include_once", "instanceof", "interface", "match",
            "namespace", "new", "null", "or", "private", "protected", "public", "require",
            "require_once", "return", "static", "switch", "throw", "trait", "try", "use", "var",
            "while", "yield",
        ],
        "rust" => &[
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
            "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
            "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
            "true", "type", "unsafe", "use", "where", "while",
        ],
        "html" => &[
            "html", "head", "body", "div", "span", "p", "a", "img", "video", "audio", "script",
            "style", "link", "meta", "form", "input", "button", "table", "tr", "td", "th",
            "ul", "ol", "li", "section", "article", "header", "footer", "nav", "main",
        ],
        "bash" => &[
            "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac",
            "function", "in", "select", "until", "break", "continue", "return", "export",
            "readonly", "local", "declare",
        ],
        "markdown" => &["#", "##", "###", "####", "#####", "######"],
        _ => &[],
    }
}

/// `\s` JS persis (bukan `char::is_whitespace` Rust — beda di beberapa codepoint).
fn spasi_js(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}'
            | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

fn dorong(tokens: &mut Vec<KodeToken>, isi: &str, jenis: Highlight) {
    if isi.is_empty() {
        return;
    }
    match tokens.last_mut() {
        Some(t) if t.jenis == jenis => t.isi.push_str(isi),
        _ => tokens.push(KodeToken { isi: isi.to_string(), jenis }),
    }
}

pub fn tokenize_kode(kode: &str, bahasa: &str) -> Vec<KodeToken> {
    // builder.js:2025-2040 — perbandingan `lang` MENTAH (peka huruf besar).
    if bahasa.is_empty() || bahasa == "txt" || bahasa == "text" || bahasa == "plaintext" {
        return vec![KodeToken { isi: kode.into(), jenis: Highlight::Default }];
    }

    // builder.js:2051 — lookup keyword pakai lowercase.
    let lang_lower = bahasa.to_lowercase();
    let kata = kata_kunci(&lang_lower);
    let chars: Vec<char> = kode.chars().collect();
    let n = chars.len();
    let mut tokens: Vec<KodeToken> = Vec::new();
    // builder.js:2071-2082 — kelas identifier per `lang.toLowerCase()`.
    let is_ident = |c: char| match lang_lower.as_str() {
        "css" => c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '-'),
        "html" => c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | ':' | '-'),
        _ => c.is_ascii_alphanumeric() || matches!(c, '_' | '$'),
    };
    let mut i = 0usize;

    while i < n {
        let c = chars[i];

        if spasi_js(c) {
            // builder.js:2087-2096
            let s = i;
            while i < n && spasi_js(chars[i]) {
                i += 1;
            }
            let isi: String = chars[s..i].iter().collect();
            dorong(&mut tokens, &isi, Highlight::Default);
            continue;
        }

        // builder.js:2098-2107 — `#` komentar hanya utk python/bash (lang mentah).
        if (c == '/' && i + 1 < n && chars[i + 1] == '/')
            || (c == '#' && (bahasa == "python" || bahasa == "bash"))
        {
            let s = i;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            let isi: String = chars[s..i].iter().collect();
            dorong(&mut tokens, &isi, Highlight::Comment);
            continue;
        }

        if c == '"' || c == '\'' || c == '`' {
            // builder.js:2109-2128
            let s = i;
            let q = c;
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                } else if chars[i] == q {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            let isi: String = chars[s..i].iter().collect();
            dorong(&mut tokens, &isi, Highlight::Str);
            continue;
        }

        if c.is_ascii_digit() {
            // builder.js:2130-2139
            let s = i;
            while i < n && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | '_')) {
                i += 1;
            }
            let isi: String = chars[s..i].iter().collect();
            dorong(&mut tokens, &isi, Highlight::Number);
            continue;
        }

        if c.is_ascii_alphabetic() || matches!(c, '_' | '$') {
            // builder.js:2141-2190
            let s = i;
            while i < n && is_ident(chars[i]) {
                i += 1;
            }
            let word: String = chars[s..i].iter().collect();
            let mut jenis = Highlight::Default;

            if kata.contains(&word.as_str()) {
                jenis = Highlight::Keyword;
            } else if bahasa == "css" {
                // builder.js:2154-2163 — css branch pakai lang mentah.
                let mut j = i;
                while j < n && spasi_js(chars[j]) {
                    j += 1;
                }
                if j < n && chars[j] == ':' {
                    jenis = Highlight::Keyword;
                }
            } else if bahasa == "html" {
                // builder.js:2164-2174 — html branch pakai lang mentah.
                let mut p = s as isize - 1;
                while p >= 0 && spasi_js(chars[p as usize]) {
                    p -= 1;
                }
                if p >= 0 {
                    let pc = chars[p as usize];
                    if pc == '<' || (pc == '/' && p >= 1 && chars[(p - 1) as usize] == '<') {
                        jenis = Highlight::Keyword;
                    }
                }
            }

            if jenis == Highlight::Default {
                // builder.js:2176-2186
                let mut j = i;
                while j < n && spasi_js(chars[j]) {
                    j += 1;
                }
                if j < n && chars[j] == '(' {
                    jenis = Highlight::Method;
                }
            }

            dorong(&mut tokens, &word, jenis);
            continue;
        }

        // builder.js:2192-2193
        dorong(&mut tokens, &c.to_string(), Highlight::Default);
        i += 1;
    }

    tokens
}


// builder.js:2253-2261 — newLayout('Single', primitive).
fn new_layout(primitive: Value) -> Value {
    json!({
        "view_model": {
            "primitive": primitive,
            "__typename": "GenAISingleLayoutViewModel",
        }
    })
}

impl RichAi {
    // builder.js:1037-1064
    pub fn teks(&mut self, teks: &str) -> &mut Self {
        let h = extract_ie(teks, &Flag::default());
        self.submessages.push(AiSubMessage::Teks { teks: h.teks.clone() });
        let mut prim: Map<String, Value> = Map::new();
        prim.insert("text".into(), Value::String(h.teks));
        if !h.entities.is_empty() {
            prim.insert("inline_entities".into(), Value::Array(h.entities));
        }
        prim.insert("__typename".into(), Value::String("GenAIMarkdownTextUXPrimitive".into()));
        self.sections.push(new_layout(Value::Object(prim)));
        self
    }

    // builder.js:1066-1090
    pub fn kode(&mut self, bahasa: &str, kode: &str) -> &mut Self {
        let blok = tokenize_kode(kode, bahasa);
        self.submessages.push(AiSubMessage::Kode { bahasa: bahasa.into(), blok: blok.clone() });
        let code_blocks: Vec<Value> = blok.iter().map(|t| {
            json!({"content": t.isi, "type": t.jenis.nama()})
        }).collect();
        let mut prim: Map<String, Value> = Map::new();
        prim.insert("language".into(), Value::String(bahasa.into()));
        prim.insert("code_blocks".into(), Value::Array(code_blocks));
        prim.insert("__typename".into(), Value::String("GenAICodeUXPrimitive".into()));
        self.sections.push(new_layout(Value::Object(prim)));
        self
    }

    // builder.js:1092-1115, 2205-2251
    pub fn tabel(&mut self, baris: Vec<Vec<String>>) -> &mut Self {
        if baris.is_empty() {
            return self;
        }
        let (header, sisa) = baris.split_first().unwrap();
        let max_len = std::iter::once(header.len())
            .chain(sisa.iter().map(|r| r.len()))
            .max()
            .unwrap_or(0);
        let pad = |r: &Vec<String>| -> Vec<String> {
            let mut v = r.clone();
            v.resize(max_len, String::new());
            v
        };
        let header_pad = pad(header);
        let sisa_pad: Vec<Vec<String>> = sisa.iter().map(pad).collect();

        // baris padded: header dulu, lalu sisa
        let mut semua_padded = vec![header_pad.clone()];
        semua_padded.extend(sisa_pad.clone());

        self.submessages.push(AiSubMessage::Tabel {
            judul: String::new(),
            baris: semua_padded,
            heading: true,
        });

        // bangun unified_rows — is_header → cells → markdown_cells (bila ada entity)
        let mut unified_rows: Vec<Value> = Vec::new();
        let buat_row = |is_header: bool, cells: &Vec<String>| -> Value {
            let markdown_cells: Vec<Value> = cells.iter().map(|sel| {
                let h = extract_ie(sel, &Flag::default());
                let mut m: Map<String, Value> = Map::new();
                m.insert("text".into(), Value::String(h.teks));
                if !h.entities.is_empty() {
                    m.insert("inline_entities".into(), Value::Array(h.entities));
                }
                Value::Object(m)
            }).collect();
            let ada_entity = markdown_cells.iter().any(|c| {
                c.get("inline_entities").map_or(false, |v| !v.as_array().unwrap().is_empty())
            });
            let mut row: Map<String, Value> = Map::new();
            row.insert("is_header".into(), Value::Bool(is_header));
            row.insert("cells".into(), Value::Array(cells.iter().map(|s| Value::String(s.clone())).collect()));
            if ada_entity {
                row.insert("markdown_cells".into(), Value::Array(markdown_cells));
            }
            Value::Object(row)
        };
        unified_rows.push(buat_row(true, &header_pad));
        for r in &sisa_pad {
            unified_rows.push(buat_row(false, r));
        }

        let mut prim: Map<String, Value> = Map::new();
        prim.insert("rows".into(), Value::Array(unified_rows));
        prim.insert("__typename".into(), Value::String("GenATableUXPrimitive".into()));
        self.sections.push(new_layout(Value::Object(prim)));
        self
    }

    pub fn section(&mut self, section: Value) -> &mut Self {
        self.sections.push(section);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nama_highlight() {
        assert_eq!(Highlight::Default.nama(), "DEFAULT");
        assert_eq!(Highlight::Keyword.nama(), "KEYWORD");
        assert_eq!(Highlight::Method.nama(), "METHOD");
        assert_eq!(Highlight::Str.nama(), "STR");
        assert_eq!(Highlight::Number.nama(), "NUMBER");
        assert_eq!(Highlight::Comment.nama(), "COMMENT");
    }

    // Vektor verbatim dari brief task-2 (oracle builder.js:1540-2203).
    #[test]
    fn tokenizer_sama_dengan_js() {
        let cek = |kode: &str, bahasa: &str, harap: &[(&str, Highlight)]| {
            let got = tokenize_kode(kode, bahasa);
            let got: Vec<(&str, Highlight)> =
                got.iter().map(|t| (t.isi.as_str(), t.jenis)).collect();
            assert_eq!(got, harap, "kode={kode:?} bahasa={bahasa:?}");
        };
        cek(
            "const app = createApp({});\napp.mount(\"#app\"); // hi",
            "javascript",
            &[
                ("const", Highlight::Keyword),
                (" app = ", Highlight::Default),
                ("createApp", Highlight::Method),
                ("({});\napp.", Highlight::Default),
                ("mount", Highlight::Method),
                ("(", Highlight::Default),
                ("\"#app\"", Highlight::Str),
                ("); ", Highlight::Default),
                ("// hi", Highlight::Comment),
            ],
        );
        cek(
            "def f(x):\n  # kom\n  return 1",
            "python",
            &[
                ("def", Highlight::Keyword),
                (" ", Highlight::Default),
                ("f", Highlight::Method),
                ("(x):\n  ", Highlight::Default),
                ("# kom", Highlight::Comment),
                ("\n  ", Highlight::Default),
                ("return", Highlight::Keyword),
                (" ", Highlight::Default),
                ("1", Highlight::Number),
            ],
        );
        cek(
            ".a { color: red; }",
            "css",
            &[(".a { ", Highlight::Default), ("color", Highlight::Keyword), (": red; }", Highlight::Default)],
        );
        cek(
            "<div class=\"x\">hi</div>",
            "html",
            &[
                ("<", Highlight::Default),
                ("div", Highlight::Keyword),
                (" class=", Highlight::Default),
                ("\"x\"", Highlight::Str),
                (">hi</", Highlight::Default),
                ("div", Highlight::Keyword),
                (">", Highlight::Default),
            ],
        );
        cek("halo", "text", &[("halo", Highlight::Default)]);
        cek("", "text", &[("", Highlight::Default)]);
        cek("let x = \"1\"", "zzz", &[("let x = ", Highlight::Default), ("\"1\"", Highlight::Str)]);
        cek("a() color: x", "js", &[("a", Highlight::Method), ("() color: x", Highlight::Default)]);
        cek("a() color: x", "Text", &[("a", Highlight::Method), ("() color: x", Highlight::Default)]);
        cek("a() color: x", "text", &[("a() color: x", Highlight::Default)]);
        cek("a() color: x", "CSS", &[("a", Highlight::Method), ("() color: x", Highlight::Default)]);
        cek(
            "a() color: x",
            "css",
            &[
                ("a", Highlight::Method),
                ("() ", Highlight::Default),
                ("color", Highlight::Keyword),
                (": x", Highlight::Default),
            ],
        );
        cek("a  b", "javascript", &[("a  b", Highlight::Default)]);
    }


    fn cek(input: &str, flag: Flag, teks: &str, entities: &str) {
        let h = extract_ie(input, &flag);
        assert_eq!(h.teks, teks);
        assert_eq!(
            serde_json::to_string(&h.entities).unwrap(),
            entities
        );
    }

    #[test]
    fn output_sama_dengan_js() {
        cek(
            "[Vue](https://vuejs.org)",
            Flag::default(),
            "{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[x](!https://a.b)",
            Flag::default(),
            "{{NIXEL_HYPERLINK_0}}https://a.b{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"x","is_trusted":false,"url":"https://a.b","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[](https://a.b)",
            Flag::default(),
            "{{NIXEL_CITATION_0}}https://a.b{{/NIXEL_CITATION_0}}",
            r#"[{"key":"NIXEL_CITATION_0","metadata":{"reference_id":1,"reference_url":"https://a.b","reference_title":"https://a.b","reference_display_name":"https://a.b","sources":[],"__typename":"GenAISearchCitationItem"}}]"#,
        );
        cek(
            "[x|10|20|30|40]<https://l>",
            Flag::default(),
            "{{NIXEL_LATEX_0}}x{{/NIXEL_LATEX_0}}",
            r#"[{"key":"NIXEL_LATEX_0","metadata":{"latex_expression":"x","latex_image":{"url":"https://l","width":10,"height":20},"font_height":30,"padding":40,"__typename":"GenAILatexItem"}}]"#,
        );
        cek(
            "[x]<https://l>",
            Flag::default(),
            "{{NIXEL_LATEX_0}}x{{/NIXEL_LATEX_0}}",
            r#"[{"key":"NIXEL_LATEX_0","metadata":{"latex_expression":"x","latex_image":{"url":"https://l","width":100,"height":100},"font_height":83.333333333333,"padding":15,"__typename":"GenAILatexItem"}}]"#,
        );
        cek(
            "[x|0|0|0|0]<https://l>",
            Flag::default(),
            "{{NIXEL_LATEX_0}}x{{/NIXEL_LATEX_0}}",
            r#"[{"key":"NIXEL_LATEX_0","metadata":{"latex_expression":"x","latex_image":{"url":"https://l","width":100,"height":100},"font_height":83.333333333333,"padding":15,"__typename":"GenAILatexItem"}}]"#,
        );
        cek(
            "[x|1.5|2.25]<https://l>",
            Flag::default(),
            "{{NIXEL_LATEX_0}}x{{/NIXEL_LATEX_0}}",
            r#"[{"key":"NIXEL_LATEX_0","metadata":{"latex_expression":"x","latex_image":{"url":"https://l","width":1.5,"height":2.25},"font_height":83.333333333333,"padding":15,"__typename":"GenAILatexItem"}}]"#,
        );
        cek(
            "\\[bukan](https://x) [ok](https://y)",
            Flag::default(),
            "\\[bukan](https://x) {{NIXEL_HYPERLINK_0}}https://y{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"ok","is_trusted":true,"url":"https://y","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[a [b] c](https://x)",
            Flag::default(),
            "[a {{NIXEL_HYPERLINK_0}}https://x{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"b] c","is_trusted":true,"url":"https://x","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[a](https://x [b](https://y)",
            Flag::default(),
            "[a](https://x {{NIXEL_HYPERLINK_0}}https://y{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"b","is_trusted":true,"url":"https://y","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "🚀 [Vue](https://vuejs.org)",
            Flag::default(),
            "🚀 {{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}]"#,
        );
    }

    #[test]
    fn flag_mematikan_jenis() {
        cek(
            "[Vue](https://vuejs.org) sisa",
            Flag { hyperlink: false, ..Flag::default() },
            "[Vue](https://vuejs.org) sisa",
            "[]",
        );
        cek(
            "[x|10]<https://l>",
            Flag { latex: false, ..Flag::default() },
            "[x|10]<https://l>",
            "[]",
        );
        cek(
            "[](https://a.b)",
            Flag { citation: false, ..Flag::default() },
            "[](https://a.b)",
            "[]",
        );
        cek(
            "[a](https://x)",
            Flag { extract: false, ..Flag::default() },
            "[a](https://x)",
            "[]",
        );
    }

    #[test]
    fn indeks_urut_dan_edge() {
        cek(
            "[a](https://x) [b](https://y)",
            Flag::default(),
            "{{NIXEL_HYPERLINK_0}}https://x{{/NIXEL_HYPERLINK_0}} {{NIXEL_HYPERLINK_1}}https://y{{/NIXEL_HYPERLINK_1}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"a","is_trusted":true,"url":"https://x","__typename":"GenAIInlineLinkItem"}},{"key":"NIXEL_HYPERLINK_1","metadata":{"display_name":"b","is_trusted":true,"url":"https://y","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[a](x) [](y) [c](z)",
            Flag::default(),
            "{{NIXEL_HYPERLINK_0}}x{{/NIXEL_HYPERLINK_0}} {{NIXEL_CITATION_0}}y{{/NIXEL_CITATION_0}} {{NIXEL_HYPERLINK_1}}z{{/NIXEL_HYPERLINK_1}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"a","is_trusted":true,"url":"x","__typename":"GenAIInlineLinkItem"}},{"key":"NIXEL_CITATION_0","metadata":{"reference_id":1,"reference_url":"y","reference_title":"y","reference_display_name":"y","sources":[],"__typename":"GenAISearchCitationItem"}},{"key":"NIXEL_HYPERLINK_1","metadata":{"display_name":"c","is_trusted":true,"url":"z","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[]()",
            Flag::default(),
            "{{NIXEL_CITATION_0}}{{/NIXEL_CITATION_0}}",
            r#"[{"key":"NIXEL_CITATION_0","metadata":{"reference_id":1,"reference_url":"","reference_title":"","reference_display_name":"","sources":[],"__typename":"GenAISearchCitationItem"}}]"#,
        );
        cek("[tanpa tutup", Flag::default(), "[tanpa tutup", "[]");
        cek(
            "[nested (kurung)](https://x)",
            Flag::default(),
            "{{NIXEL_HYPERLINK_0}}https://x{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"nested (kurung)","is_trusted":true,"url":"https://x","__typename":"GenAIInlineLinkItem"}}]"#,
        );
        cek(
            "[esc \\(x\\)](https://x)",
            Flag::default(),
            "{{NIXEL_HYPERLINK_0}}https://x{{/NIXEL_HYPERLINK_0}}",
            r#"[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"esc \\(x\\)","is_trusted":true,"url":"https://x","__typename":"GenAIInlineLinkItem"}}]"#,
        );
    }

    // --- builder RichAi ---

    fn sj(v: &Value) -> String { serde_json::to_string(v).unwrap() }

    #[test]
    fn builder_teks_polos() {
        let mut r = RichAi::default();
        r.teks("Halo dunia");
        assert_eq!(r.submessages, vec![AiSubMessage::Teks { teks: "Halo dunia".into() }]);
        let want = r#"{"view_model":{"primitive":{"text":"Halo dunia","__typename":"GenAIMarkdownTextUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}"#;
        assert_eq!(sj(&r.sections[0]), want);
    }

    #[test]
    fn builder_teks_link() {
        let mut r = RichAi::default();
        r.teks("[Vue](https://vuejs.org) bagus");
        let want = r#"{"view_model":{"primitive":{"text":"{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}} bagus","inline_entities":[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}],"__typename":"GenAIMarkdownTextUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}"#;
        assert_eq!(sj(&r.sections[0]), want);
        assert!(matches!(&r.submessages[0], AiSubMessage::Teks { teks } if teks.contains("NIXEL_HYPERLINK_0")));
    }

    #[test]
    fn builder_kode() {
        let mut r = RichAi::default();
        r.kode("js", "const app = createApp({});");
        let AiSubMessage::Kode { bahasa, blok } = &r.submessages[0] else { panic!() };
        assert_eq!(bahasa, "js");
        assert!(blok.iter().any(|t| t.isi == "createApp" && t.jenis == Highlight::Method));
        let want = r#"{"view_model":{"primitive":{"language":"js","code_blocks":[{"content":"const app = ","type":"DEFAULT"},{"content":"createApp","type":"METHOD"},{"content":"({});","type":"DEFAULT"}],"__typename":"GenAICodeUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}"#;
        assert_eq!(sj(&r.sections[0]), want);
    }

    #[test]
    fn builder_tabel_padding() {
        let mut r = RichAi::default();
        r.tabel(vec![
            vec!["A".into(), "B".into()],
            vec!["1".into()],
        ]);
        assert_eq!(r.submessages, vec![AiSubMessage::Tabel {
            judul: "".into(),
            baris: vec![vec!["A".into(), "B".into()], vec!["1".into(), "".into()]],
            heading: true,
        }]);
        let want = r#"{"view_model":{"primitive":{"rows":[{"is_header":true,"cells":["A","B"]},{"is_header":false,"cells":["1",""]}],"__typename":"GenATableUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}"#;
        assert_eq!(sj(&r.sections[0]), want);
    }

    #[test]
    fn builder_tabel_link_oracle() {
        let mut r = RichAi::default();
        r.tabel(vec![
            vec!["Nama".into(), "Link".into()],
            vec!["Vue".into(), "[Vue](https://vuejs.org)".into()],
            vec!["React".into(), "https://react.dev".into()],
        ]);
        let want = r#"{"view_model":{"primitive":{"rows":[{"is_header":true,"cells":["Nama","Link"]},{"is_header":false,"cells":["Vue","[Vue](https://vuejs.org)"],"markdown_cells":[{"text":"Vue"},{"text":"{{NIXEL_HYPERLINK_0}}https://vuejs.org{{/NIXEL_HYPERLINK_0}}","inline_entities":[{"key":"NIXEL_HYPERLINK_0","metadata":{"display_name":"Vue","is_trusted":true,"url":"https://vuejs.org","__typename":"GenAIInlineLinkItem"}}]}]},{"is_header":false,"cells":["React","https://react.dev"]}],"__typename":"GenATableUXPrimitive"},"__typename":"GenAISingleLayoutViewModel"}}"#;
        assert_eq!(sj(&r.sections[0]), want);
    }

    #[test]
    fn builder_tabel_kosong_noop() {
        let mut r = RichAi::default();
        r.tabel(Vec::new());
        assert!(r.submessages.is_empty());
        assert!(r.sections.is_empty());
    }

    #[test]
    fn builder_section_passthrough() {
        let mut r = RichAi::default();
        r.section(json!({"a": 1}));
        assert_eq!(r.sections[0], json!({"a": 1}));
    }
}
