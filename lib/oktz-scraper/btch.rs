//! Klien resmi `btch-downloader@6.3.6` untuk scraper yang dipakai plugin JS.
//!
//! Padanan file JS: paket npm `btch-downloader` dan `btch-http`, lalu
//! `plugins/download/{capcutdl,cocofundl,snackvideodl}.js`. Klien ini
//! sengaja memakai `HttpGet` workspace, bukan membuat koneksi sendiri.

use std::fmt;

use oktz_plugin_api::{HttpCall, HttpError, HttpGet};
use serde_json::{Map, Value, json};

/// Base URL dari `btch-downloader/dist/Defaults/site.js`.
pub const BASE_URL: &str = "https://backend1.tioo.eu.org";

/// Versi paket yang_header dan versi klien yang dikirim.
pub const VERSION: &str = "6.3.6";

/// Watermark developer dari `btch-downloader/dist/Watermark/config.js`.
pub const DEVELOPER: &str = "BOTCAHX";

/// Tujuan laporan masalah dari `btch-downloader/dist/Defaults/site.js`.
pub const ISSUES: &str = "https://github.com/hostinger-bot/btch-downloader/issues";

/// Kegagalan request `btch-http`, setara error yang ditangkap SDK Node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtchError {
    /// Request gagal sebelum body JSON bisa dibaca.
    Transport(String),
    /// Backend menjawab status di luar 2xx.
    Http { status: u16, message: String },
    /// Body 2xx bukan JSON valid.
    InvalidJson(String),
}

impl fmt::Display for BtchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(message) | Self::InvalidJson(message) => f.write_str(message),
            Self::Http { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for BtchError {}

impl From<HttpError> for BtchError {
    fn from(error: HttpError) -> Self {
        Self::Transport(error.message)
    }
}

/// GET mentah ke backend `btch`, setara fungsi `HttpGet` dari `btch-http`.
pub fn get(http: &dyn HttpGet, endpoint: &str, target_url: &str) -> Result<Value, BtchError> {
    let url = format!("{BASE_URL}/{endpoint}?url={}", form_encode(target_url));
    let request = HttpCall::get(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", format!("btch/{VERSION}"))
        .header("X-Client-Version", VERSION)
        .terima_semua_status();
    let response = http.call(&request)?;

    if !(200..300).contains(&response.status) {
        return Err(BtchError::Http {
            status: response.status,
            message: format!("{} Unknown", response.status),
        });
    }

    serde_json::from_slice(response.body.bytes())
        .map_err(|error| BtchError::InvalidJson(error.to_string()))
}

/// `URLSearchParams.append` Node: form-urlencoded, spasi menjadi `+`.
fn form_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                out.push(*byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Envelope kegagalan dari `formatErrorResponse` pada SDK Node.
fn failure(error: &BtchError) -> Value {
    json!({
        "developer": DEVELOPER,
        "status": false,
        "message": error.to_string(),
        "note": format!("Please report issues to {ISSUES}")
    })
}

/// `capcut(url)` dari `btch-downloader`: data CapCut disebar ke envelope utama.
pub fn capcut(http: &dyn HttpGet, target_url: &str) -> Value {
    match get(http, "capcut", target_url) {
        Ok(data) => {
            let mut out = Map::new();
            out.insert(
                "developer".to_string(),
                Value::String(DEVELOPER.to_string()),
            );
            out.insert("status".to_string(), Value::Bool(true));
            if let Some(fields) = data.as_object() {
                for (key, value) in fields {
                    out.insert(key.clone(), value.clone());
                }
            }
            Value::Object(out)
        }
        Err(error) => failure(&error),
    }
}

/// `cocofun(url)` dari `btch-downloader`: data CocoFun berada di `result`.
pub fn cocofun(http: &dyn HttpGet, target_url: &str) -> Value {
    match get(http, "cocofun", target_url) {
        Ok(data) => json!({
            "developer": DEVELOPER,
            "status": true,
            "result": data
        }),
        Err(error) => failure(&error),
    }
}

/// `snackvideo(url)` dari `btch-downloader`: data SnackVideo berada di `result`.
pub fn snackvideo(http: &dyn HttpGet, target_url: &str) -> Value {
    match get(http, "snackvideo", target_url) {
        Ok(data) => json!({
            "developer": DEVELOPER,
            "status": true,
            "result": data
        }),
        Err(error) => failure(&error),
    }
}
