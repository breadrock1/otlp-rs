use axum::routing::get;
use axum::Router;
use otlp::init_telemetry;
use otlp::HeaderAttribute;
use otlp::HttpLogger;
use otlp::PathFilter;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

const SERVICE_NAME: &str = "axum-service";
const SERVICE_ADDRESS: &str = "0.0.0.0:8080";

async fn handler() -> &'static str {
    "Hello"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let attributes = vec![
        HeaderAttribute::new("user_id", "x-user-id"),
        HeaderAttribute::new("organization_id", "x-organization-id"),
    ];

    let otlp_config = otlp::TelemetryConfig::builder()
        .level("info".to_string())
        .enable_direct_loki(false)
        .enable_remote_otlp(false)
        .enable_syslog(false)
        .syslog_address("udp://127.0.0.1:5514".to_string())
        .attributes(attributes.clone())
        .build()?;

    let _otlp_guard = init_telemetry(SERVICE_NAME, &otlp_config)?;

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
