use std::fmt;

/// Cache errors.  These are always non-fatal: the cache layer logs and
/// continues (fail-open) unless the user explicitly configures fail-closed.
#[derive(Debug)]
pub enum CacheError {
    /// Backend store error (e.g. Redis unreachable).
    Backend(String),
    /// Serialization / deserialization failure.
    Codec(String),
    /// Invalid configuration or key.
    Invalid(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::Backend(msg) => write!(f, "cache backend error: {msg}"),
            CacheError::Codec(msg) => write!(f, "cache codec error: {msg}"),
            CacheError::Invalid(msg) => write!(f, "cache invalid: {msg}"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<std::io::Error> for CacheError {
    fn from(e: std::io::Error) -> Self {
        CacheError::Backend(e.to_string())
    }
}
