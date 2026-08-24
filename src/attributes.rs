use crate::config::HeaderAttribute;
use axum::http::HeaderMap;

/// The fixed set of request attribute names the crate knows how to render as
/// separate top-level fields. `tracing` requires statically-declared field
/// names, so arbitrary attribute names cannot be emitted as individual fields.
/// Config entries whose `name` is not in this set are ignored (with a warning).
pub const SUPPORTED_ATTRIBUTES: &[&str] = &["user_id", "organization_id"];

/// Reads the configured request attributes from the request headers.
///
/// For every configured `{name, header}` pair a single value is produced only
/// when the header is present and contains valid UTF-8. Missing headers and
/// non-UTF-8 values are silently skipped.
pub fn extract_attributes(
    headers: &HeaderMap,
    attributes: &[HeaderAttribute],
) -> Vec<(String, String)> {
    attributes
        .iter()
        .filter_map(|attr| {
            let value = headers
                .get(attr.header())
                .and_then(|h| h.to_str().ok())
                .map(str::to_owned)?;
            Some((attr.name().to_owned(), value))
        })
        .collect()
}

/// Logs a warning for every configured attribute whose `name` is not part of
/// [`SUPPORTED_ATTRIBUTES`] and therefore cannot be rendered as a field.
pub fn validate_attributes(attributes: &[HeaderAttribute]) {
    for attr in attributes {
        if !SUPPORTED_ATTRIBUTES.contains(&attr.name().as_str()) {
            tracing::warn!(
                name = %attr.name(),
                "unsupported request attribute, it will be ignored"
            );
        }
    }
}
