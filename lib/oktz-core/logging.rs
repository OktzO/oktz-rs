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

/// Pasang subscriber tracing. Level dari `OKTZ_LOG` (`off|error|warn|info|debug|trace`,
/// default argumen `level`); `RUST_LOG` menimpa keduanya. `OKTZ_LOG=off` =
/// tak ada log tracing sama sekali.
pub fn init(level: &str) {
    INIT.call_once(|| {
        let dipilih = std::env::var("OKTZ_LOG")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| level.to_string());
        if dipilih.trim().eq_ignore_ascii_case("off") {
            return;
        }
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&dipilih));
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
    fn init_is_idempotent() {
        init("info");
        init("debug");
        init("trace");
    }

    #[test]
    fn init_off_tidak_panic() {
        // INIT sudah terkunci oleh test lain; panggilan ini hanya memastikan
        // cabang "off" tidak panic saat dipanggil.
        unsafe { std::env::set_var("OKTZ_LOG", "off") };
        init("info");
        unsafe { std::env::remove_var("OKTZ_LOG") };
    }
}
