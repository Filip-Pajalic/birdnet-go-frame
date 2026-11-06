//! Why an HTTP exchange failed.

use std::{fmt, io};

/// Why a request could not be made or its response read.
#[derive(Debug)]
#[must_use]
#[non_exhaustive]
pub enum HttpError {
    /// The URL is not one this client can fetch; the text says why.
    BadUrl(&'static str),
    /// Connecting, sending or receiving failed, or timed out.
    Io(io::Error),
    /// The other side did not speak HTTP/1.x as expected; the text says how.
    Malformed(&'static str),
    /// The response body is larger than [`crate::MAX_BODY_BYTES`].
    TooLarge,
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadUrl(problem) => write!(formatter, "bad URL: {problem}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Malformed(problem) => write!(formatter, "malformed HTTP response: {problem}"),
            Self::TooLarge => formatter.write_str("response too large"),
        }
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
