//! `.duck` — chat duck.ai (GPT-6 Luna / GPT-5.4 mini / Claude Haiku 5.5).
//!
//! `Keterbatasan:` tabel `__tpl` di shim memuat hasil serialisasi `innerHTML` versi Chrome
//! untuk 4 template yang pernah muncul. Template baru dari DDG bikin angka challenge salah
//! (server balas 418, bukan crash); tambahkan satu baris di `__tpl` saat itu terjadi.

use oktz_plugin_api::{plugin, Ctx, HttpCall, HttpGet, Reply};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const STATUS: &str = "https://duckduckgo.com/duckchat/v1/status";
const CHAT: &str = "https://duck.ai/duckchat/v1/chat";
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36";
const STACK: &str = "Error\n    at https://duck.ai/dist/duckai-dist/entry.duckai.js:2:1";
const MAX_LEN: usize = 4096;
const PIKIR_PREVIEW: usize = 800;
const SUMBER_MAKS: usize = 3;
const SESI: &str = "duckai:model:";

const SHIM: &str = r##"
function __serialize(html) {
  var known = {
    "<div><div></div><div></div": ["<div><div></div><div></div></div>", 3],
    "<p><div></p><p></div": ["<p></p><div><p></p><p></p></div>", 4],
    "<br><div></br><br></div": ["<br><div><br><br></div>", 4],
    "<li><div></li><li></div": ["<li><div></div></li><li></li>", 3]
  };
  if (known[html]) return { html: known[html][0], count: known[html][1] };
  var partial = html.match(/<\/[a-zA-Z][\w-]*$/);
  if (partial) html = html.slice(0, partial.index);
  var re = /<(\/?)([a-zA-Z][\w-]*)[^>]*>?/g, stack = [], m, count = 0;
  while ((m = re.exec(html)) !== null) {
    if (m[1] === "/") { if (stack.length) stack.pop(); }
    else { stack.push(m[2].toLowerCase()); count++; }
  }
  var out = html;
  for (var i = stack.length - 1; i >= 0; i--) out += "</" + stack[i] + ">";
  return { html: out, count: count };
}

function HTMLElement() {}
function HTMLDivElement() {}
function HTMLIFrameElement() {}
function Node() {}
function Element() {}
function NodeList() {}
Element.prototype = Object.create(Node.prototype);
HTMLElement.prototype = Object.create(Element.prototype);
HTMLDivElement.prototype = Object.create(HTMLElement.prototype);
HTMLIFrameElement.prototype = Object.create(HTMLElement.prototype);
NodeList.prototype = Object.create(Array.prototype);
NodeList.prototype.constructor = NodeList;

function __el(ctor, tag) {
  var e = Object.create(ctor.prototype);
  e.tagName = String(tag).toUpperCase();
  e.children = [];
  e.childNodes = [];
  e.__html = "";
  e.__nodes = 0;
  Object.defineProperty(e, "innerHTML", {
    get: function () { return e.__html; },
    set: function (v) { var r = __serialize(String(v)); e.__html = r.html; e.__nodes = r.count; }
  });
  e.srcdoc = "";
  e.__attached = false;
  e.style = { cssText: "", setProperty: function () {} };
  e.contentWindow = null;
  e.contentDocument = null;
  e.getAttribute = function () { return null; };
  e.setAttribute = function () {};
  e.matches = function () { return false; };
  e.closest = function () { return null; };
  e.getBoundingClientRect = function () {
    return e.__attached
      ? { width: 100, height: 20, top: 0, left: 0, right: 100, bottom: 20, x: 0, y: 0 }
      : { width: 0, height: 0, top: 0, left: 0, right: 0, bottom: 0, x: 0, y: 0 };
  };
  Object.defineProperty(e, "offsetWidth", { get: function () { return e.__attached ? 100 : 0; } });
  Object.defineProperty(e, "offsetHeight", { get: function () { return e.__attached ? 20 : 0; } });
  Object.defineProperty(e, "clientWidth", { get: function () { return e.__attached ? 100 : 0; } });
  Object.defineProperty(e, "clientHeight", { get: function () { return e.__attached ? 20 : 0; } });
  Object.defineProperty(e, "scrollWidth", { get: function () { return e.__attached ? 100 : 0; } });
  Object.defineProperty(e, "scrollHeight", { get: function () { return e.__attached ? 20 : 0; } });
  Object.defineProperty(e, "offsetParent", { get: function () { return e.__attached ? __body : null; } });
  Object.defineProperty(e, "isConnected", { get: function () { return e.__attached; } });
  Object.defineProperty(e, "parentNode", { get: function () { return e.__parent || null; } });
  Object.defineProperty(e, "ownerDocument", { get: function () { return __doc; } });
  Object.defineProperty(e, "textContent", {
    get: function () { return e.__text || ""; },
    set: function (v) { e.__text = String(v); }
  });
  e.hidden = false;
  e.clientTop = 0;
  e.classList = { add: function () {}, remove: function () {}, contains: function () { return false; } };
  e.dataset = {};
  e.checkVisibility = function () { return e.__attached; };
  e.getClientRects = function () { return []; };
  e.querySelector = function () { return null; };
  e.querySelectorAll = function () { var l = new NodeList(); l.length = e.__nodes; return l; };
  e.appendChild = function (c) { c.__attached = true; c.__parent = e; e.children.push(c); e.childNodes.push(c); return c; };
  e.removeChild = function (c) {
    var i = e.children.indexOf(c);
    if (i >= 0) { e.children.splice(i, 1); e.childNodes.splice(i, 1); }
    return c;
  };
  return e;
}

var __meta = { getAttribute: function (n) {
  if (n === "content") return "default-src 'none'; script-src 'unsafe-inline';";
  if (n === "sandbox") return "allow-scripts allow-same-origin";
  return null;
} };
var __jsa = __el(HTMLIFrameElement, "iframe");
__jsa.getAttribute = function (n) {
  if (n === "sandbox") return "allow-scripts allow-same-origin";
  if (n === "srcdoc") return "DuckDuckGo Fraud & Abuse";
  return null;
};
__jsa.contentDocument = {
  querySelector: function (s) { return s === 'meta[http-equiv="Content-Security-Policy"]' ? __meta : null; },
  querySelectorAll: function () { return new NodeList(); }
};
__jsa.contentWindow = { frames: undefined, document: __jsa.contentDocument, get: function () { return function () {}; } };
__jsa.contentWindow.self = __jsa.contentWindow;

var __body = __el(HTMLElement, "body");
var __doc = {
  body: __body,
  childNodes: [__body],
  children: [__body],
  documentElement: __el(HTMLElement, "html"),
  querySelector: function (s) { return s === "#jsa" ? __jsa : null; },
  querySelectorAll: function () { return new NodeList(); },
  getElementById: function () { return null; },
  createElement: function (tag) {
    var t = String(tag).toLowerCase();
    var e = t === "div" ? __el(HTMLDivElement, tag) : (t === "iframe" ? __el(HTMLIFrameElement, tag) : __el(HTMLElement, tag));
    if (t === "iframe") {
      e.contentWindow = { frames: undefined };
      e.contentWindow.self = e.contentWindow;
      e.contentWindow.document = { body: __el(HTMLElement, "body"), children: [], querySelector: function () { return null; }, querySelectorAll: function () { return new NodeList(); } };
      e.contentDocument = e.contentWindow.document;
    }
    return e;
  }
};

var __win = globalThis;
var __origToString = Object.prototype.toString;
Object.prototype.toString = function () {
  if (this === __win) return "[object Window]";
  return __origToString.call(this);
};
__win.document = __doc;
__win.self = __win;
__win.top = __win;
__win.parent = __win;
__win.name = "";
__win.hasOwnProperty = Object.prototype.hasOwnProperty.bind(__win);
__win.__DDG_BE_VERSION__ = "serp";
__win.__DDG_FE_CHAT_HASH__ = "x";
__win.parseInt = parseInt;
__win.String = String;
__win.Object = Object;
__win.Array = Array;
__win.Promise = Promise;
__win.JSON = JSON;
__win.Math = Math;
__win.Error = Error;
__win.Symbol = Symbol;
__win.Proxy = Proxy;
__win.Window = function Window() {};
__win.console = { log: function () {}, warn: function () {} };
__win.getComputedStyle = function (el) {
  var tampil = el && el.__attached ? "block" : "";
  return { getPropertyValue: function (n) { return n === "display" ? tampil : ""; } };
};
__win.matchMedia = function () { return { matches: false }; };
__win.setTimeout = function (f) { if (typeof f === "function") f(); return 0; };
__win.location = { origin: "https://duck.ai", href: "https://duck.ai/" };

var navigator = { userAgent: "__UA__", webdriver: false, hardwareConcurrency: 8, platform: "Linux x86_64" };
var window = __win;
var document = __doc;
var self = __win;
var top = __win;
var getComputedStyle = __win.getComputedStyle;
var matchMedia = __win.matchMedia;
var performance = { now: function () { return 0; } };
var localStorage = { getItem: function () { return null; }, setItem: function () {} };
"##;

/// Model duck.ai yang gratis (`entityHasAccess: true`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Model {
    Gpt6,
    Gpt54,
    Haiku,
}

impl Model {
    const SEMUA: [Model; 3] = [Model::Gpt6, Model::Gpt54, Model::Haiku];

    fn id(self) -> &'static str {
        match self {
            Self::Gpt6 => "gpt-6-luna",
            Self::Gpt54 => "gpt-5.4-mini",
            Self::Haiku => "claude-haiku-5-5",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Gpt6 => "GPT-6 Luna",
            Self::Gpt54 => "GPT-5.4 mini",
            Self::Haiku => "Claude Haiku 5.5",
        }
    }

    fn effort(self) -> &'static str {
        match self {
            Self::Gpt54 => "medium",
            _ => "low",
        }
    }

    fn dari(teks: &str) -> Option<Self> {
        match teks {
            "gpt-6" | "gpt6" | "luna" => Some(Self::Gpt6),
            "gpt-5.4" | "gpt54" | "mini" => Some(Self::Gpt54),
            "haiku-5.5" | "haiku5.5" | "haiku55" | "haiku" => Some(Self::Haiku),
            _ => None,
        }
    }

    fn tersimpan(ctx: &Ctx) -> Self {
        let kunci = format!("{SESI}{}", ctx.chat);
        let nama = ctx
            .sessions
            .lock()
            .ok()
            .and_then(|s| s.get(&kunci).and_then(Value::as_str).map(str::to_string));
        nama.as_deref().and_then(Self::dari).unwrap_or(Self::Gpt6)
    }

    fn simpan(self, ctx: &Ctx) {
        let kunci = format!("{SESI}{}", ctx.chat);
        if let Ok(mut s) = ctx.sessions.lock() {
            s.insert(kunci, Value::String(self.id().to_string()));
        }
    }
}

/// Jawaban atau kegagalan satu panggilan duck.ai.
#[derive(Debug, PartialEq, Eq)]
enum Hasil {
    Jawab {
        teks: String,
        pikir: String,
        sumber: Vec<String>,
    },
    Ditolak(u16),
    Rusak,
}

fn sha_b64(teks: &str) -> String {
    b64_encode(&Sha256::digest(teks.as_bytes()))
}

fn b64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for potong in data.chunks(3) {
        let b = [
            potong[0],
            *potong.get(1).unwrap_or(&0),
            *potong.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        if potong.len() > 1 {
            out.push(T[(n >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }
        if potong.len() > 2 {
            out.push(T[n as usize & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn b64_nilai(c: u8) -> Option<u8> {
    Some(match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    })
}

fn b64_decode(teks: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(teks.len() / 4 * 3);
    let mut buf: u32 = 0;
    let mut bit = 0u32;
    for c in teks.bytes() {
        if c == b'=' || c.is_ascii_whitespace() {
            continue;
        }
        let v = b64_nilai(c)?;
        buf = (buf << 6) | u32::from(v);
        bit += 6;
        if bit >= 8 {
            bit -= 8;
            out.push((buf >> bit) as u8);
            buf &= (1 << bit) - 1;
        }
    }
    Some(out)
}

fn header_vqd(blob_b64: &str) -> Option<String> {
    let js = String::from_utf8(b64_decode(blob_b64)?).ok()?;
    let mut nilai = jalankan_challenge(&js)?;
    let mentah = nilai.get_mut("client_hashes")?.as_array_mut()?;
    for sel in mentah.iter_mut() {
        let teks = sel.as_str()?;
        *sel = Value::String(sha_b64(teks));
    }
    if let Some(meta) = nilai.get_mut("meta").and_then(Value::as_object_mut) {
        meta.insert("origin".into(), json!("https://duck.ai"));
        meta.insert("stack".into(), json!(STACK));
        meta.insert("duration".into(), json!("11"));
    }
    Some(b64_encode(serde_json::to_vec(&nilai).ok()?.as_slice()))
}

fn sloppy() -> rquickjs::context::EvalOptions {
    let mut o = rquickjs::context::EvalOptions::default();
    o.strict = false;
    o
}

fn jalankan_challenge(js: &str) -> Option<Value> {
    use rquickjs::{Context, Runtime};
    let isi = js
        .trim_start_matches("(async function(){")
        .trim_end_matches("})()");
    let rt = Runtime::new().ok()?;
    let ctx = Context::full(&rt).ok()?;
    let siap: Option<()> = ctx.with(|c| -> Option<()> {
        c.eval_with_options::<(), _>(SHIM.replace("__UA__", UA), sloppy())
            .ok()?;
        c.eval_with_options::<(), _>(
            format!(
                "(async function(){{{isi}}})().then(function(r){{ globalThis.__out = JSON.stringify(r); }}, function(e){{ globalThis.__out = 'ERR ' + (e && e.message ? e.message : e); }});"
            ),
            sloppy(),
        )
        .ok()?;
        Some(())
    });
    siap?;
    while rt.execute_pending_job().unwrap_or(false) {}
    let teks: String = ctx.with(|c| {
        c.eval("typeof __out === 'undefined' ? '' : __out")
            .unwrap_or_default()
    });
    serde_json::from_str(&teks).ok()
}

fn ambil_blob(http: &dyn HttpGet) -> Option<String> {
    let res = http
        .call(
            &HttpCall::get(STATUS)
                .header("x-vqd-accept", "1")
                .header("user-agent", UA)
                .header("accept", "*/*"),
        )
        .ok()?;
    if res.status >= 400 {
        return None;
    }
    res.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("x-vqd-hash-1"))
        .map(|(_, v)| v.clone())
}

fn body_chat(model: Model, tanya: &str) -> Value {
    json!({
        "model": model.id(),
        "messages": [
            { "role": "user", "content": [{ "type": "text", "text": tanya }] },
            { "role": "assistant", "content": "", "parts": [{ "type": "text", "text": "" }] },
        ],
        "canUseTools": true,
        "reasoningEffort": model.effort(),
        "metadata": { "toolChoice": { "WebSearch": true } },
        "canUseApproxLocation": null,
        "canDelegateImageGeneration": null,
        "canShowGreeting": false,
    })
}

fn tanya(http: &dyn HttpGet, model: Model, tanya: &str) -> Hasil {
    let Some(blob) = ambil_blob(http) else {
        return Hasil::Rusak;
    };
    let Some(vqd) = header_vqd(&blob) else {
        return Hasil::Rusak;
    };
    let body = serde_json::to_vec(&body_chat(model, tanya)).unwrap_or_default();
    let res = match http.call(
        &HttpCall::post(CHAT)
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header("user-agent", UA)
            .header("x-vqd-accept", "1")
            .header("x-vqd-hash-1", vqd)
            .header("x-duckai-tool-approval", "web-search")
            .header("referer", "https://duck.ai/")
            .header("origin", "https://duck.ai")
            .raw(body, "application/json")
            .terima_semua_status(),
    ) {
        Ok(r) => r,
        Err(_) => return Hasil::Rusak,
    };
    if res.status >= 400 {
        return Hasil::Ditolak(res.status);
    }
    let (teks, pikir, sumber) = parse_sse(&res.text());
    if teks.is_empty() {
        return Hasil::Rusak;
    }
    Hasil::Jawab {
        teks,
        pikir,
        sumber,
    }
}

fn parse_sse(raw: &str) -> (String, String, Vec<String>) {
    let mut jawab = String::new();
    let mut pikir = String::new();
    let mut sumber: Vec<String> = Vec::new();
    for baris in raw.lines() {
        let Some(data) = baris.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" || data.starts_with('[') {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        match v.get("role").and_then(Value::as_str) {
            Some("assistant") => {
                if let Some(m) = v.get("message").and_then(Value::as_str) {
                    jawab.push_str(m);
                }
            }
            Some("reasoning") => {
                if let Some(m) = v.get("message").and_then(Value::as_str) {
                    pikir.push_str(m);
                }
            }
            Some("tool-invocation") if v.get("state").and_then(Value::as_str) == Some("result") => {
                if let Some(urls) = v
                    .get("data")
                    .and_then(|d| d.get("urls"))
                    .and_then(Value::as_array)
                {
                    for u in urls.iter().filter_map(Value::as_str) {
                        if sumber.len() >= SUMBER_MAKS {
                            break;
                        }
                        if !sumber.iter().any(|x| x == u) {
                            sumber.push(u.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    (jawab, pikir, sumber)
}

fn potong(teks: &str, batas: usize) -> String {
    if teks.chars().count() > batas {
        let head: String = teks.chars().take(batas).collect();
        format!("{head}\n\n... (dipotong)")
    } else {
        teks.to_string()
    }
}

fn kartu_bantuan(prefix: &str, sekarang: Model) -> String {
    let daftar = Model::SEMUA
        .iter()
        .map(|m| format!("> `{prefix}duck {} <pertanyaan>` — {}", m.label(), m.id()))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "🦆 *Duck.ai*\n\n\
         > Chat gratis lewat duck.ai, WebSearch + reasoning selalu aktif.\n\n\
         *MODEL AKTIF:* {}\n\n\
         *PENGGUNAAN:*\n{daftar}\n\n\
         *CONTOH:*\n\
         > `{prefix}duck gpt-6 jelaskan black hole`\n\
         > `{prefix}duck apa itu rust?` (pakai model terakhir)",
        sekarang.label()
    )
}

fn kelihatan_model(teks: &str) -> bool {
    let t = teks.to_ascii_lowercase();
    t.starts_with("gpt")
        || t.starts_with("claude")
        || t.starts_with("haiku")
        || t.starts_with("mistral")
}

#[plugin(
    name = "duck",
    alias = ["duckai", "duck-ai"],
    category = "ai",
    description = "Chat duck.ai — GPT-6 Luna / GPT-5.4 mini / Claude Haiku 5.5",
    usage = ".duck [model] <pertanyaan>",
    example = ".duck gpt-6 jelaskan black hole",
    cooldown = 10,
    energi = 2,
    is_enabled = true,
    is_owner = false,
    is_premium = false,
    is_group = false,
    is_private = false,
    is_admin = false,
    is_bot_admin = false
)]
pub fn duck(ctx: Ctx) -> Reply {
    if ctx.args.is_empty() {
        return Reply::text(kartu_bantuan(&ctx.prefix, Model::tersimpan(&ctx)));
    }

    let (model, pertanyaan) = match Model::dari(&ctx.args[0].to_ascii_lowercase()) {
        Some(m) => {
            m.simpan(&ctx);
            (m, ctx.args[1..].join(" "))
        }
        None if kelihatan_model(&ctx.args[0]) => {
            return Reply::text(format!(
                "❌ *Model tidak dikenal*\n\n> `{}`\n\nCoba: {}",
                ctx.args[0],
                Model::SEMUA
                    .iter()
                    .map(|m| format!("`{}`", m.label()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .with_reaction("☢");
        }
        None => (Model::tersimpan(&ctx), ctx.args.join(" ")),
    };

    if pertanyaan.trim().is_empty() {
        return Reply::text(kartu_bantuan(&ctx.prefix, model));
    }

    match tanya(&*ctx.http, model, pertanyaan.trim()) {
        Hasil::Jawab {
            teks,
            pikir,
            sumber,
        } => {
            let mut out = String::new();
            if !pikir.trim().is_empty() {
                let preview: String = if pikir.chars().count() > PIKIR_PREVIEW {
                    let head: String = pikir.chars().take(PIKIR_PREVIEW).collect();
                    format!("{head}...")
                } else {
                    pikir.clone()
                };
                out.push_str(&format!(
                    "💭 *Proses Berpikir:*\n{}\n\n",
                    preview.trim().replace('\n', "\n> ")
                ));
            }
            out.push_str(teks.trim());
            if !sumber.is_empty() {
                out.push_str("\n\n🌐 *Sumber:*");
                for u in &sumber {
                    out.push_str(&format!("\n> {u}"));
                }
            }
            Reply::text(potong(&out, MAX_LEN)).with_reaction("✅")
        }
        Hasil::Ditolak(status) => Reply::text(format!(
            "❌ *Duck.ai menolak* (HTTP {status})\n\n> Challenge kedaluwarsa atau kena limit. Coba lagi nanti."
        ))
        .with_reaction("☢"),
        Hasil::Rusak => Reply::text(crate::common::error_template(&ctx)).with_reaction("☢"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oktz_plugin_api::{HttpBody, HttpError, HttpResponse};
    use std::sync::{Arc, Mutex};

    const FIXTURE: &str = include_str!("fixtures/duckai-vqd.js");

    #[derive(Debug)]
    struct Stub {
        blob: String,
        sse: String,
        status: u16,
        offline: bool,
        seen: Mutex<Vec<(String, String)>>,
    }

    impl Stub {
        fn baru(sse: &str, status: u16) -> Arc<Stub> {
            Arc::new(Stub {
                blob: b64_encode(FIXTURE.as_bytes()),
                sse: sse.to_string(),
                status,
                offline: false,
                seen: Mutex::new(Vec::new()),
            })
        }

        fn offline() -> Arc<Stub> {
            Arc::new(Stub {
                blob: String::new(),
                sse: String::new(),
                status: 200,
                offline: true,
                seen: Mutex::new(Vec::new()),
            })
        }
    }

    impl HttpGet for Stub {
        fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::get(url))
        }

        fn post_json(&self, url: &str, body: &Value) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::post(url).json(body))
        }

        fn post_form(&self, url: &str, form: &str) -> Result<HttpResponse, HttpError> {
            self.call(&HttpCall::post(url).form(form))
        }

        fn post_multipart(
            &self,
            _: &str,
            _: &[(&str, HttpBody)],
        ) -> Result<HttpResponse, HttpError> {
            unimplemented!("duck hanya JSON")
        }

        fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
            let body = req
                .body
                .as_ref()
                .map(|b| String::from_utf8_lossy(b.bytes()).to_string())
                .unwrap_or_default();
            self.seen.lock().unwrap().push((req.url.clone(), body));
            if self.offline {
                return Err(HttpError {
                    message: "offline".into(),
                });
            }
            if req.url == STATUS {
                return Ok(HttpResponse {
                    status: 200,
                    body: HttpBody::Bytes {
                        data: Arc::from(Vec::new()),
                        mimetype: None,
                    },
                    headers: vec![("x-vqd-hash-1".into(), self.blob.clone())],
                });
            }
            Ok(HttpResponse {
                status: self.status,
                body: HttpBody::Bytes {
                    data: Arc::from(self.sse.clone().into_bytes()),
                    mimetype: Some("text/event-stream".into()),
                },
                headers: vec![],
            })
        }
    }

    fn ctx(http: Arc<Stub>, args: &str) -> Ctx {
        Ctx {
            prefix: ".".into(),
            command: "duck".into(),
            sender: "628111@s.whatsapp.net".into(),
            chat: "628111@s.whatsapp.net".into(),
            text: args.into(),
            args: args.split_whitespace().map(str::to_string).collect(),
            http,
            ..Default::default()
        }
    }

    fn jalankan(http: Arc<Stub>, args: &str) -> Reply {
        (oktz_plugin_api::find("duck").unwrap().handler)(ctx(http, args))
    }

    fn teks(r: &Reply) -> &str {
        r.text_body().expect("harus ada teks")
    }

    #[test]
    fn challenge_fixture_menghasilkan_raw_yang_sama_dengan_browser() {
        let js = include_str!("fixtures/duckai-vqd.js");
        let v = jalankan_challenge(js).expect("challenge harus bisa dijalankan");
        let raw: Vec<&str> = v["client_hashes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(raw, [UA, "9687", "5246"]);
        assert_eq!(
            v["server_hashes"].as_array().unwrap().len(),
            3,
            "3 server hash"
        );
        assert!(v["meta"]["challenge_id"].is_string());
    }

    #[test]
    fn header_vqd_berisi_hash_dan_meta_tambahan() {
        let js = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/ai/fixtures/duckai-vqd.js"
        ))
        .unwrap();
        let blob = b64_encode(js.as_bytes());
        let header = header_vqd(&blob).expect("header harus terbentuk");
        let v: Value = serde_json::from_slice(&b64_decode(&header).unwrap()).unwrap();
        assert_eq!(v["client_hashes"][0], sha_b64(UA));
        assert_eq!(v["meta"]["origin"], "https://duck.ai");
        assert_eq!(v["meta"]["duration"], "11");
        assert!(v["meta"]["stack"].as_str().unwrap().starts_with("Error"));
    }

    #[test]
    fn b64_bolak_balik() {
        assert_eq!(b64_encode(b"hi"), "aGk=");
        assert_eq!(b64_decode("aGk=").unwrap(), b"hi");
        assert!(b64_decode("!!!").is_none());
        assert_eq!(
            sha_b64("abc"),
            "ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0="
        );
    }

    #[test]
    fn alias_model_dikenali() {
        for (k, m) in [
            ("gpt-6", Model::Gpt6),
            ("gpt54", Model::Gpt54),
            ("mini", Model::Gpt54),
            ("haiku", Model::Haiku),
        ] {
            assert_eq!(Model::dari(k), Some(m), "{k}");
        }
        assert_eq!(Model::dari("gpt-7"), None);
    }

    #[test]
    fn registrasi_plugin_sesuai() {
        let p = oktz_plugin_api::find("duck").unwrap();
        assert_eq!(p.meta.name, "duck");
        assert_eq!(p.meta.alias, ["duckai", "duck-ai"]);
        assert_eq!(p.meta.category, "ai");
        assert!(oktz_plugin_api::find("duckai").is_some());
    }

    #[test]
    fn tanpa_argumen_menampilkan_kartu_model() {
        let http = Stub::baru("", 200);
        let r = jalankan(http.clone(), "");
        assert!(teks(&r).contains("Duck.ai"));
        assert!(teks(&r).contains("GPT-5.4 mini"));
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn model_asing_tidak_ditelan_jadi_pertanyaan() {
        let http = Stub::baru("", 200);
        let r = jalankan(http.clone(), "gpt-7 halo");
        assert!(teks(&r).contains("Model tidak dikenal"));
        assert!(http.seen.lock().unwrap().is_empty());
    }

    #[test]
    fn sse_menggabung_jawaban_dan_menyimpan_sumber() {
        let raw = concat!(
            "data: [CHAT_TITLE:awal]\n\n",
            "data: {\"role\":\"tool-invocation\",\"state\":\"result\",\"data\":{\"urls\":[\"https://a.id\",\"https://b.id\"]}}\n\n",
            "data: {\"role\":\"reasoning\",\"message\":\"mikir \"}\n\n",
            "data: {\"role\":\"assistant\",\"message\":\"PONG\"}\n\n",
            "data: {\"role\":\"assistant\",\"message\":\"!\"}\n\n",
            "data: [DONE]\n\n"
        );
        let (jawab, pikir, sumber) = parse_sse(raw);
        assert_eq!(jawab, "PONG!");
        assert_eq!(pikir, "mikir ");
        assert_eq!(sumber, ["https://a.id", "https://b.id"]);
    }

    #[test]
    fn jawaban_duck_dipakai_model_dan_reasoning_benar() {
        let http = Stub::baru(
            "data: {\"role\":\"assistant\",\"message\":\"hai\"}\n\ndata: [DONE]\n",
            200,
        );
        let r = jalankan(http.clone(), "gpt-5.4 hai");
        assert_eq!(teks(&r), "hai");
        let terlihat = http.seen.lock().unwrap().clone();
        let (_, body) = terlihat
            .iter()
            .find(|(u, _)| u == CHAT)
            .expect("chat dipanggil");
        let v: Value = serde_json::from_str(body).unwrap();
        assert_eq!(v["model"], "gpt-5.4-mini");
        assert_eq!(v["reasoningEffort"], "medium");
        assert_eq!(v["canUseTools"], true);
        assert_eq!(v["metadata"]["toolChoice"]["WebSearch"], true);
    }

    #[test]
    fn status_418_jadi_kartu_ditolak() {
        let http = Stub::baru("{\"status\":418}", 418);
        let r = jalankan(http, "hai");
        assert!(teks(&r).contains("menolak"));
        assert!(matches!(
            r.without_mutations(),
            Reply::Reaction { emoji, .. } if emoji == "☢"
        ));
    }

    #[test]
    #[ignore = "butuh jaringan ke duck.ai"]
    fn live_chat_gpt6() {
        let http = oktz_plugin_api::UreqHttp::new();
        match tanya(&http, Model::Gpt6, "Reply with the single word PONG") {
            Hasil::Jawab { teks, .. } => assert!(!teks.trim().is_empty()),
            lain => panic!("{lain:?}"),
        }
    }

    #[test]
    fn offline_pakai_template_error_bawaan() {
        let r = jalankan(Stub::offline(), "hai");
        assert_eq!(teks(&r), crate::common::error_template_bawaan(".", "duck"));
    }
}
