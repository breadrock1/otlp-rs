use derive_builder::Builder;
use gset::Getset;
use serde_derive::Deserialize;

const DEFAULT_LOKI_ADDRESS: &str = "localhost:3100";

#[derive(Clone, Builder, Deserialize, Getset)]
pub struct TelemetryConfig {
    #[getset(get, vis = "pub")]
    logger: LoggerConfig,
    #[getset(get, vis = "pub")]
    tracing: TracingConfig,
}

impl TelemetryConfig {
    pub fn builder() -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::default()
    }
}

#[derive(Clone, Builder, Deserialize, Getset)]
pub struct TracingConfig {
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable: bool,

    #[getset(get, vis = "pub")]
    #[builder(default = "info".to_string())]
    level: String,

    #[getset(get, vis = "pub")]
    #[builder(default, setter(strip_option))]
    address: Option<String>,
}

impl TracingConfig {
    pub fn builder() -> TracingConfigBuilder {
        TracingConfigBuilder::default()
    }

    pub fn is_remote_enabled(&self) -> bool {
        self.enable
    }
}

#[derive(Clone, Builder, Deserialize, Getset)]
pub struct LoggerConfig {
    #[getset(get, vis = "pub")]
    #[builder(default = "info".to_string())]
    level: String,

    /// Request attributes to extract from HTTP headers and attach to
    /// http-request logs and OTel spans. Empty by default.
    #[getset(get, vis = "pub")]
    #[builder(default)]
    attributes: Vec<HeaderAttribute>,

    #[getset(get, vis = "pub")]
    #[builder(default)]
    loki: Option<LokiConfig>,
    #[getset(get, vis = "pub")]
    #[builder(default)]
    syslog: Option<SyslogConfig>,
}

impl LoggerConfig {
    pub fn builder() -> LoggerConfigBuilder {
        LoggerConfigBuilder::default()
    }

    pub fn is_loki_enabled(&self) -> bool {
        self.loki.as_ref().map(|it| it.enable()).unwrap_or_default()
    }

    pub fn get_loki_config(&self) -> &LokiConfig {
        self.loki.as_ref().unwrap()
    }

    pub fn is_syslog_enabled(&self) -> bool {
        self.syslog
            .as_ref()
            .map(|it| it.enable())
            .unwrap_or_default()
    }

    pub fn get_syslog_config(&self) -> &SyslogConfig {
        self.syslog.as_ref().unwrap()
    }
}

#[derive(Clone, Builder, Deserialize, Getset)]
pub struct LokiConfig {
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable: bool,

    #[getset(get, vis = "pub")]
    #[builder(default, setter(strip_option))]
    address: String,
}

impl Default for LokiConfig {
    fn default() -> Self {
        LokiConfig {
            enable: false,
            address: DEFAULT_LOKI_ADDRESS.to_string(),
        }
    }
}

impl LokiConfig {
    pub fn builder() -> LokiConfigBuilder {
        LokiConfigBuilder::default()
    }
}

#[derive(Clone, Builder, Default, Deserialize, Getset)]
pub struct SyslogConfig {
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable: bool,

    #[getset(get, vis = "pub")]
    #[builder(default, setter(strip_option))]
    address: Option<String>,
}

impl SyslogConfig {
    pub fn builder() -> SyslogConfigBuilder {
        SyslogConfigBuilder::default()
    }
}

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
