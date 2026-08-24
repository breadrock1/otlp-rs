use axum::Router;
use axum::routing::get;
use otlp::HeaderAttribute;
use otlp::HttpLogger;
use otlp::PathFilter;
use otlp::init_telemetry;
use otlp::{LoggerConfig, LokiConfig, SyslogConfig, TelemetryConfig, TracingConfig};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

const SERVICE_NAME: &str = "axum-service";
const SERVICE_ADDRESS: &str = "0.0.0.0:8080";
const LEVEL: &str = "info";

async fn handler() -> &'static str {
    "Hello"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let attributes = vec![
        HeaderAttribute::new("user_id", "x-user-id"),
        HeaderAttribute::new("organization_id", "x-organization-id"),
    ];

    let telemetry_config = build_app_config(attributes.clone());
    let _otlp_guard = init_telemetry(SERVICE_NAME, &telemetry_config)?;

    // ... Another necessary initialization of application

    let app = Router::new()
        .route("/", get(handler))
        .layer(TraceLayer::new_for_http().make_span_with(PathFilter::new(attributes.clone())))
        .layer(HttpLogger::new(attributes));

    let listener = TcpListener::bind(SERVICE_ADDRESS).await?;
    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!(err=?err, "failed to stop http server");
    };

    Ok(())
}

fn build_app_config(attributes: Vec<HeaderAttribute>) -> TelemetryConfig {
    let tracing_config = TracingConfig::builder()
        .enable(true)
        .level(LEVEL.to_string())
        .address("localhost:4317".to_string())
        .build()
        .expect("failed to init tracing config");

    let loki_config = LokiConfig::builder()
        .enable(true)
        .address("http://localhost:3100".to_string())
        .build()
        .expect("failed to init loki config");

    let syslog_config = SyslogConfig::builder()
        .enable(true)
        .address("udp://localhost:514".to_string())
        .build()
        .expect("failed to init syslog config");

    let logger_config = LoggerConfig::builder()
        .level(LEVEL.to_string())
        .loki(Some(loki_config))
        .syslog(Some(syslog_config))
        .attributes(attributes)
        .build()
        .expect("failed to init common logger config");

    TelemetryConfig::builder()
        .logger(logger_config)
        .tracing(tracing_config)
        .build()
        .expect("failed to init common telemetry config")
}
