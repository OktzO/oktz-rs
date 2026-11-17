// Port `extractIE` — builder.js:40-212.
use serde_json::{Value, json};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
