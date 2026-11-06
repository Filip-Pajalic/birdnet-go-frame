//! The error for a response body that is not what the API documents.

use core::fmt;

/// A response body could not be read as the expected JSON.
#[derive(Debug)]
#[must_use]
pub struct ParseError {
    source: serde_json::Error,
}

impl ParseError {
    pub(crate) fn new(source: serde_json::Error) -> Self {
        Self { source }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unreadable BirdNET-Go response: {}", self.source)
    }
}

impl core::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.source)
    }
}
