# Otlp-rs library

## Overview

There is library to enable observability on rust service with following abilities:
 - local and remote tracing (supporting jaeger);
 - local and remote logging (supporting loki).

## Quick Start

1. Need to include this repository into project Cargo.toml manifest file:

   Include using local system path:
   ```toml
   [dependencies.otlp]
   path = "./otlp"
    ```
   
   Include using git:
   ```toml
   [dependencies.otlp]
   git = "https://<user>:<token-or-password>@<git-url>/otlp.git"
   tag = "0.0.1"
   ```

2. Past example code into main function:

    ```rust
    fn main() -> anyhow::Result<()> {
        let otlp_config = otlp::TelemetryConfig::builder()
            .level("info".to_string())
            .enable_direct_loki(false)
            .enable_remote_otlp(false)
            .build()?;

        let _otlp_guard = init_telemetry(SERVICE_NAME, &otlp_config)?;
        
         // ... Another app logic
   
         Ok(())
   }
    ```
