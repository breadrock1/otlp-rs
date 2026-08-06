use axum::http::Request;
use cached::proc_macro::once;
use regex::Regex;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[once]
fn build_path_filter() -> PathFilter {
    PathFilter::default()
}

pub fn otel_axum_layer_filter_callback(path: &str) -> bool {
    let path_filter = build_path_filter();
    !path_filter.is_path_ignored(path)
}

#[derive(Clone)]
pub struct PathFilter {
    pub paths: Vec<Regex>,
}

impl Default for PathFilter {
    fn default() -> Self {
        PathFilter {
            paths: vec![
                Regex::new("/health").expect("failed to compile health regex"),
                Regex::new("/metrics").expect("failed to compile metrics regex"),
                Regex::new("/favicon.ico").expect("failed to compile favicon.ico regex"),
                Regex::new("/static/.*").expect("failed to compile static regex"),
                Regex::new("/api/metrics").expect("failed to compile api metrics regex"),
                Regex::new("/api/swagger/.*").expect("failed to compile swagger regex"),
                Regex::new("/api-docs/openapi.json").expect("failed to compile swagger regex"),
            ],
        }
    }
}

struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);

impl<'a> opentelemetry::propagation::Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

impl<B> tower_http::trace::MakeSpan<B> for PathFilter {
    fn make_span(&mut self, request: &Request<B>) -> tracing::Span {
        let path = request.uri().path();
        if self.is_path_ignored(path) {
            return tracing::span!(tracing::Level::DEBUG, "http-request-filtered");
        }

        let span = tracing::info_span!(
            "http-request-parent",
            method = %request.method(),
            status_code = tracing::field::Empty,
            uri = %request.uri(),
            version = ?request.version(),
        );

        let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        if let Err(err) = span.set_parent(parent_context) {
            return tracing::span!(tracing::Level::DEBUG, "failed to set span parent", err=?err);
        }

        span
    }
}

impl PathFilter {
    pub fn is_path_ignored(&self, path: &str) -> bool {
        self.paths.iter().any(|it| it.is_match_at(path, 0))
    }
}
