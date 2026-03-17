use derive_builder::Builder;
use gset::Getset;
use serde_derive::Deserialize;

#[derive(Clone, Builder, Deserialize, Getset)]
pub struct TelemetryConfig {
    #[getset(get, vis = "pub")]
    #[builder(default = "info".to_string())]
    level: String,
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable_remote_otlp: bool,
    #[getset(get, vis = "pub")]
    otlp_address: Option<String>,
    #[getset(get_copy, vis = "pub")]
    #[builder(default = "false")]
    enable_direct_loki: bool,
    #[getset(get, vis = "pub")]
    loki_address: Option<String>,
}

impl TelemetryConfig {
    pub fn builder() -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::default()
    }
}
