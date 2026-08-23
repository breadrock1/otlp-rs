use crate::config::TelemetryConfig;
use std::fmt::Debug;
use std::sync::Mutex;
use syslog::{Formatter3164, Logger, LoggerBackend, Severity};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// A `tracing_subscriber` layer that forwards every log event to a syslog
/// server (or the local `/dev/log` Unix socket), mapping the tracing level to
/// the matching syslog severity.
pub struct SyslogLayer {
    logger: Mutex<Logger<LoggerBackend, Formatter3164>>,
}

impl SyslogLayer {
    fn new(logger: Logger<LoggerBackend, Formatter3164>) -> Self {
        Self {
            logger: Mutex::new(logger),
        }
    }
}

impl<S: Subscriber> Layer<S> for SyslogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut fields = EventFields::default();
        event.record(&mut fields);

        let message = format!("{}: {}", event.metadata().target(), fields.to_message());
        let severity = severity_of(event.metadata().level());

        // Writes through `&mut self`; guard the logger behind a mutex because
        // `on_event` only hands us `&self`.
        let mut logger = match self.logger.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        let result = match severity {
            Severity::LOG_ERR => logger.err(&message),
            Severity::LOG_WARNING => logger.warning(&message),
            Severity::LOG_NOTICE => logger.notice(&message),
            Severity::LOG_INFO => logger.info(&message),
            Severity::LOG_DEBUG => logger.debug(&message),
            _ => logger.info(&message),
        };

        if let Err(err) = result {
            // Print to stderr instead of re-entering the subscriber.
            eprintln!("failed to write to syslog: {err}");
        }
    }
}

fn severity_of(level: &Level) -> Severity {
    match *level {
        Level::ERROR => Severity::LOG_ERR,
        Level::WARN => Severity::LOG_WARNING,
        Level::INFO => Severity::LOG_INFO,
        Level::DEBUG | Level::TRACE => Severity::LOG_DEBUG,
    }
}

/// Builds a `SyslogLayer` from the config, or `None` when syslog is disabled.
///
/// When `syslog_address` is omitted/empty the local Unix socket (`/dev/log`) is
/// used; otherwise the address must look like `udp://host:port` or
/// `tcp://host:port`.
pub fn build_syslog_layer(config: &TelemetryConfig) -> anyhow::Result<Option<SyslogLayer>> {
    if !config.enable_syslog() {
        return Ok(None);
    }

    let formatter = Formatter3164::default();
    let logger = match config.syslog_address().as_deref() {
        None | Some("") => syslog::unix(formatter)?,
        Some(addr) => build_remote_logger(formatter, addr)?,
    };

    Ok(Some(SyslogLayer::new(logger)))
}

fn build_remote_logger(
    formatter: Formatter3164,
    addr: &str,
) -> anyhow::Result<Logger<LoggerBackend, Formatter3164>> {
    let server = addr
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(addr);

    match addr {
        a if a.starts_with("udp://") => Ok(syslog::udp(formatter, ("0.0.0.0", 0), server)?),
        a if a.starts_with("tcp://") => Ok(syslog::tcp(formatter, server)?),
        _ => Err(anyhow::anyhow!(
            "unsupported syslog address, expected udp:// or tcp://: {addr}"
        )),
    }
}

#[derive(Default)]
struct EventFields {
    message: Option<String>,
    entries: Vec<String>,
}

impl EventFields {
    fn record(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
            self.message = Some(value);
        } else {
            self.entries.push(format!("{}={}", field.name(), value));
        }
    }

    fn to_message(&self) -> String {
        let body = self.message.clone().unwrap_or_default();
        if self.entries.is_empty() {
            body
        } else {
            format!("{body} [{}]", self.entries.join(", "))
        }
    }
}

impl Visit for EventFields {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.record(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record(field, value.to_string());
    }
}
