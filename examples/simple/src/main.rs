use otlp::init_telemetry;
use tokio::net::TcpListener;

const SERVICE_NAME: &str = "axum-service";
const SERVICE_ADDRESS: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let otlp_config = otlp::TelemetryConfig::builder()
        .level("info".to_string())
        .enable_direct_loki(false)
        .enable_remote_otlp(false)
        .build()?;

    let _otlp_guard = init_telemetry(SERVICE_NAME, &otlp_config)?;

    // ... Another necessary initialization of application

    let app = axum::Router::new();
    let listener = TcpListener::bind(SERVICE_ADDRESS).await?;
    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!(err=?err, "failed to stop http server");
    };

    Ok(())
}

