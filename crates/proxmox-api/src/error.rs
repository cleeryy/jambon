use std::fmt;

/// Errors that can occur when interacting with the Proxmox VE API.
#[derive(Debug)]
pub enum Error {
    /// HTTP transport-level error (connectivity, TLS, etc.).
    Http(reqwest::Error),
    /// API returned an error status code with a message.
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
    /// Authentication failed (401 / 403).
    Unauthorized(String),
    /// The requested resource was not found.
    NotFound(String),
    /// Configuration error (missing/invalid settings).
    Config(String),
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Api { status, body } => {
                write!(f, "API error ({status}): {body}")
            }
            Self::Unauthorized(msg) => write!(f, "Authentication failed: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
