//! Klien HTTP dan(assert) kecil khusus uji jaringan downloader.
//!
//! Padananrequest di `plugins/download/*.js` memakai `axios`; helper ini
//! memakai `curl` yang sudah tersedia agar crate plugin tidak menambah
//! dependency jaringan. Semua fungsi di modul ini hanya aktif pada `cfg(test)`.

use oktz_plugin_api::{
    Ctx, HttpBody, HttpCall, HttpError, HttpGet, HttpMethod, HttpResponse, MultipartPart, Reply,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

const SENDER: &str = "628111@s.whatsapp.net";
static TEMP_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub(crate) struct LiveRequest {
    pub status: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct LiveHttp {
    requests: Mutex<Vec<LiveRequest>>,
}

impl LiveHttp {
    pub(crate) fn requests(&self) -> Vec<LiveRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl HttpGet for LiveHttp {
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
        url: &str,
        parts: &[(&str, HttpBody)],
    ) -> Result<HttpResponse, HttpError> {
        let mut converted = Vec::with_capacity(parts.len());
        for (name, body) in parts {
            let part = match body {
                HttpBody::Form(value) => MultipartPart::field(*name, value.clone()),
                HttpBody::Bytes { data, mimetype } => {
                    let mut part = MultipartPart::file(*name, *name, data.to_vec());
                    if let Some(mimetype) = mimetype {
                        part = part.with_mimetype(mimetype.clone());
                    }
                    part
                }
                HttpBody::Multipart(_) => {
                    return Err(HttpError {
                        message: "nested multipart tidak didukung oleh helper live".into(),
                    });
                }
            };
            converted.push(part);
        }
        let parts = converted;
        self.call(&HttpCall::post(url).multipart(parts))
    }

    fn call(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
        let result = self.execute(req);
        let (status, error) = match &result {
            Ok(response) => (Some(response.status), None),
            Err(error) => (None, Some(error.message.clone())),
        };
        self.requests
            .lock()
            .unwrap()
            .push(LiveRequest { status, error });
        result
    }
}

impl LiveHttp {
    fn execute(&self, req: &HttpCall) -> Result<HttpResponse, HttpError> {
        let header_path = temp_path("header", "txt");
        let body_path = temp_path("body", "bin");
        let mut body_files = Vec::new();

        let mut command = Command::new("curl");
        command
            .arg("--silent")
            .arg("--show-error")
            .arg("--location")
            .arg("--compressed")
            .arg("--connect-timeout")
            .arg("15")
            .arg("--max-time")
            .arg("120")
            .arg("--dump-header")
            .arg(&header_path)
            .arg("--output")
            .arg(&body_path)
            .arg("--write-out")
            .arg("%{http_code}");

        if req.method == HttpMethod::Head {
            command.arg("--head");
        } else if req.method != HttpMethod::Get {
            command.arg("--request").arg(req.method.as_str());
        }

        for (name, value) in &req.headers {
            command.arg("--header").arg(format!("{name}: {value}"));
        }

        match req.body.as_ref() {
            Some(HttpBody::Bytes { data, mimetype }) => {
                if let Some(mimetype) = mimetype
                    && !req
                        .headers
                        .iter()
                        .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                {
                    command
                        .arg("--header")
                        .arg(format!("Content-Type: {mimetype}"));
                }
                let path = temp_path("request", "bin");
                fs::write(&path, data).map_err(|error| HttpError {
                    message: format!("gagal menulis body live: {error}"),
                })?;
                body_files.push(path.clone());
                command
                    .arg("--data-binary")
                    .arg(format!("@{}", path.display()));
            }
            Some(HttpBody::Form(form)) => {
                if !req
                    .headers
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                {
                    command
                        .arg("--header")
                        .arg("Content-Type: application/x-www-form-urlencoded");
                }
                command.arg("--data-raw").arg(form);
            }
            Some(HttpBody::Multipart(parts)) => {
                for part in parts {
                    match &part.body {
                        HttpBody::Form(value) => {
                            command.arg("--form").arg(format!("{}={value}", part.name));
                        }
                        HttpBody::Bytes { data, mimetype } => {
                            let path = temp_path("multipart", "bin");
                            fs::write(&path, data).map_err(|error| HttpError {
                                message: format!("gagal menulis multipart live: {error}"),
                            })?;
                            body_files.push(path.clone());
                            let mut spec = format!("{}=@{}", part.name, path.display());
                            if let Some(file_name) = &part.file_name {
                                spec.push_str(&format!(";filename={file_name}"));
                            }
                            if let Some(mimetype) = mimetype {
                                spec.push_str(&format!(";type={mimetype}"));
                            }
                            command.arg("--form").arg(spec);
                        }
                        HttpBody::Multipart(_) => {
                            return Err(HttpError {
                                message: "nested multipart tidak didukung oleh helper live".into(),
                            });
                        }
                    }
                }
            }
            None => {}
        }

        command.arg("--url").arg(&req.url);
        let output = command.output();
        for path in body_files {
            let _ = fs::remove_file(path);
        }
        let output = output.map_err(|error| HttpError {
            message: format!("curl gagal: {error}"),
        })?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(HttpError {
                message: format!("curl transport gagal: {message}"),
            });
        }

        let status = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u16>()
            .map_err(|error| HttpError {
                message: format!("status curl tidak valid: {error}"),
            })?;
        let headers = parse_headers(&header_path);
        let body = fs::read(&body_path).unwrap_or_default();
        let mimetype = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
            .map(|(_, value)| value.clone());
        let _ = fs::remove_file(header_path);
        let _ = fs::remove_file(body_path);

        if status >= 400 && !req.terima_semua_status {
            return Err(HttpError {
                message: format!("HTTP {status}"),
            });
        }
        Ok(HttpResponse {
            status,
            body: HttpBody::Bytes {
                data: Arc::from(body),
                mimetype,
            },
            headers,
        })
    }
}

fn parse_headers(path: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn temp_path(label: &str, extension: &str) -> PathBuf {
    let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "oktz-downloader-{label}-{}-{id}.{extension}",
        std::process::id()
    ))
}

pub(crate) fn env_required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!(
            "uji jaringan butuh {name}; isi link publik valid atau kredensial, lalu jalankan ulang"
        )
    })
}

pub(crate) fn live_ctx(command: &str, text: &str, args: Vec<String>) -> (Arc<LiveHttp>, Ctx) {
    let http = Arc::new(LiveHttp::default());
    let ctx = Ctx {
        prefix: ".".into(),
        command: command.into(),
        sender: SENDER.into(),
        chat: SENDER.into(),
        text: text.into(),
        args,
        http: http.clone(),
        ..Default::default()
    };
    (http, ctx)
}

fn payload(reply: &Reply) -> &Reply {
    match reply.without_mutations() {
        Reply::Reaction { reply, .. } => payload(reply),
        Reply::Sequence(items) => items
            .iter()
            .find(|item| media_kind(item).is_some())
            .unwrap_or_else(|| items.first().unwrap_or(reply)),
        other => other,
    }
}

fn media_kind(reply: &Reply) -> Option<&'static str> {
    match payload(reply) {
        Reply::Image { media: Some(_), .. } => Some("image"),
        Reply::Video { media: Some(_), .. } => Some("video"),
        Reply::Audio { media: Some(_), .. } => Some("audio"),
        Reply::Document { media: Some(_), .. } => Some("document"),
        _ => None,
    }
}

pub(crate) fn assert_media(reply: &Reply, expected: &[&str], http: &LiveHttp) {
    let actual = media_kind(reply);
    if actual.is_some_and(|kind| expected.contains(&kind)) {
        return;
    }

    let requests = http.requests();
    let upstream_hint = requests.iter().any(|request| {
        request.error.is_some()
            || request
                .status
                .is_some_and(|status| !(200..400).contains(&status))
    });
    let klasifikasi = if upstream_hint {
        "UPSTREAM_MATI: transport atau HTTP provider gagal"
    } else if requests.is_empty() {
        "BUG_PORT: handler tidak melakukan request"
    } else {
        "BUG_PORT_OR_NORMALISASI: provider menjawab, tetapi port tidak menghasilkan media"
    };
    panic!(
        "{klasifikasi}; media aktual={actual:?}, harapan={expected:?}, requests={requests:?}, reply={reply:?}"
    );
}
