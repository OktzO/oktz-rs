use owo_colors::OwoColorize;
use std::io::IsTerminal;
use std::sync::Once;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, format};
use tracing_subscriber::registry::LookupSpan;

static INIT: Once = Once::new();

/// Tag ringkas untuk log tracing: segmen terakhir target
/// (`oktz_core::config` → `config`; `Client/OfflineResume` apa adanya).
fn ringkas_target(target: &str) -> &str {
    target.rsplit("::").next().unwrap_or(target)
}

/// Formatter ringkas `[jam] [tag] pesan` + field `key=value`. Tag diwarnai per
/// level (INFO cyan, WARN kuning, ERROR merah), jam redup.
struct FormatRingkas;

impl<S, N> FormatEvent<S, N> for FormatRingkas
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut w: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();
        let aktif = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        let jam = format!("[{}]", chrono::Local::now().format("%H:%M:%S"));
        let tag = format!("[{}]", ringkas_target(meta.target()));
        let (jam, tag) = if aktif {
            let tag = match *meta.level() {
                tracing::Level::ERROR => tag.red().bold().to_string(),
                tracing::Level::WARN => tag.yellow().bold().to_string(),
                tracing::Level::INFO => tag.cyan().to_string(),
                _ => tag.dimmed().to_string(),
            };
            (jam.bright_black().to_string(), tag)
        } else {
            (jam, tag)
        };
        write!(w, "{jam} {tag} ")?;
        ctx.field_format().format_fields(w.by_ref(), event)?;
        writeln!(w)
    }
}

/// Pilih level dari env. `None` = jangan pasang subscriber (OKTZ_LOG=off).
/// `RUST_LOG` menang atas `OKTZ_LOG` (escape hatch tetap hidup).
fn pilih_level(oktz: Option<&str>, rust: Option<&str>, def: &str) -> Option<String> {
    let bersih = |v: Option<&str>| {
        v.map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    let rust = bersih(rust);
    if rust.is_some() {
        return rust;
    }
    match bersih(oktz) {
        Some(v) if v.eq_ignore_ascii_case("off") => None,
        Some(v) => Some(v),
        None => Some(def.to_string()),
    }
}

/// Pasang subscriber tracing. Level dari `OKTZ_LOG` (`off|error|warn|info|debug|trace`,
/// default argumen `level`); `RUST_LOG` menimpa keduanya. `OKTZ_LOG=off` =
/// tak ada log tracing sama sekali (kecuali `RUST_LOG` diisi).
pub fn init(level: &str) {
    INIT.call_once(|| {
        let Some(spec) = pilih_level(
            std::env::var("OKTZ_LOG").ok().as_deref(),
            std::env::var("RUST_LOG").ok().as_deref(),
            level,
        ) else {
            return;
        };
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&spec));
        let _ = tracing_subscriber::fmt()
            .event_format(FormatRingkas)
            .with_env_filter(filter)
            .try_init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ringkas_target_ambil_segmen_terakhir() {
        assert_eq!(ringkas_target("oktz_core::config"), "config");
        assert_eq!(ringkas_target("whatsapp_rust::client::node_io"), "node_io");
        assert_eq!(
            ringkas_target("Client/OfflineResume"),
            "Client/OfflineResume"
        );
    }

    #[test]
    fn pilih_level_off_dan_prioritas_rust_log() {
        // off tanpa RUST_LOG → senyap total
        assert_eq!(pilih_level(Some("off"), None, "info"), None);
        // RUST_LOG menang: off diabaikan
        assert_eq!(
            pilih_level(Some("off"), Some("debug"), "info").as_deref(),
            Some("debug")
        );
        assert_eq!(
            pilih_level(Some("warn"), None, "info").as_deref(),
            Some("warn")
        );
        assert_eq!(pilih_level(Some(""), None, "info").as_deref(), Some("info"));
        assert_eq!(pilih_level(None, None, "info").as_deref(), Some("info"));
    }

    #[test]
    fn init_is_idempotent() {
        init("info");
        init("debug");
        init("trace");
    }
}
