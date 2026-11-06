//! The error for a response body that is not what the API documents.

use core::fmt;

/// A response body could not be read as the expected JSON.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParseError {
    problem: &'static str,
    offset: usize,
}

impl ParseError {
    pub(crate) fn new(problem: &'static str, offset: usize) -> Self {
        Self { problem, offset }
    }

    /// Returns what was wrong.
    #[must_use]
    pub fn problem(&self) -> &'static str {
        self.problem
    }

    /// Returns the byte offset in the body where the problem was found.
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unreadable BirdNET-Go response: {} at byte {}",
            self.problem, self.offset
        )
    }
}

impl core::error::Error for ParseError {}
