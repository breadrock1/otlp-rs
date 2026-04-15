use axum::body::HttpBody;
use axum::extract::Request;
use axum::http::header;
use axum::response::Response;
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tower::Service;
use tower::layer::Layer;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct HttpLogger;

impl HttpLogger {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> Layer<S> for HttpLogger {
    type Service = LoggerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggerMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct LoggerMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for LoggerMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    // 2026-04-14 23:55:13.583 INFO http-request
    // request_id="53bb0163-ee16-410f-85f6-e57493f1ef67"
    // method="GET"
    // uri="/api/v1/cloud/buckets"
    // status="200"
    // message="Ok"
    // bytes_received="0"
    // bytes_sent="148"
    // latency="7.986792ms"
    // referer="http://localhost:2893/api/swagger/index.html"
    // client_ip="127.0.0.1"
    // user_agent="Mozilla/5.0 Firefox/149.0"

    fn call(&mut self, request: Request) -> Self::Future {
        let request_id = Uuid::new_v4();
        let method = request.method().clone();
        let uri = request.uri().clone();

        let bytes_received = request.body().size_hint().lower();

        let referer = request
            .headers()
            .get(header::REFERER)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string();

        let user_agent = request
            .headers()
            .get(header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let client_ip = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let instant = std::time::Instant::now();
        let future = self.inner.call(request);

        Box::pin(async move {
            let response: Response = future.await?;

            let latency = format!("{}ms", instant.elapsed().as_millis());
            let status = response.status().as_u16();
            let msg = response.status().canonical_reason().unwrap_or("unknown");

            let bytes_sent = response
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            match response.status().is_success() {
                true => tracing::info!(
                    %request_id,
                    %method,
                    %uri,
                    status,
                    msg,
                    bytes_received,
                    bytes_sent,
                    latency,
                    %referer,
                    %client_ip,
                    %user_agent,
                    "http-request"
                ),
                false => tracing::error!(
                    %request_id,
                    %method,
                    %uri,
                    status,
                    error = msg,
                    bytes_received,
                    bytes_sent,
                    latency,
                    %referer,
                    %client_ip,
                    %user_agent,
                    "http-request"
                ),
            };

            Ok(response)
        })
    }
}
