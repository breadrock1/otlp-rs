use derive_builder::Builder;
use gset::Getset;
use serde_derive::Deserialize;

/// Mapping of a well-known request attribute to the HTTP header its value is
/// read from. The extracted value is attached to the http-request log record
/// as a separate top-level field and to the OTel span as an attribute.
#[derive(Clone, Debug, Default, Deserialize, Getset)]
pub struct HeaderAttribute {
    /// The attribute name as it appears in tracing/logs
    /// (e.g. `"user_id"`, `"organization_id"`).
    #[getset(get, vis = "pub")]
    name: String,
    /// The HTTP header name the value is read from (e.g. `"x-user-id"`).
    #[getset(get, vis = "pub")]
    header: String,
}

impl HeaderAttribute {
    pub fn new(name: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            header: header.into(),
        }
    }
}

#[derive(Clone, Builder, Deserialize, Getset)]
pub struct TelemetryConfig {
    #[getset(get, vis = "pub")]
    #[builder(default = "info".to_string())]
    level: String,
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable_remote_otlp: bool,
    #[getset(get, vis = "pub")]
    #[builder(default, setter(strip_option))]
    otlp_address: Option<String>,
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable_direct_loki: bool,
    #[getset(get, vis = "pub")]
    #[builder(default, setter(strip_option))]
    loki_address: Option<String>,
    /// Whether to forward log messages to a syslog server.
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable_syslog: bool,
    /// Syslog server address: `udp://host:port` or `tcp://host:port`. When
    /// omitted/empty, a local Unix socket (`/dev/log`) is used.
    #[getset(get, vis = "pub")]
    #[builder(default, setter(strip_option))]
    syslog_address: Option<String>,
    /// Request attributes to extract from HTTP headers and attach to
    /// http-request logs and OTel spans. Empty by default.
    #[getset(get, vis = "pub")]
    #[builder(default)]
    attributes: Vec<HeaderAttribute>,
}

impl TelemetryConfig {
    pub fn builder() -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::default()
    }
}
