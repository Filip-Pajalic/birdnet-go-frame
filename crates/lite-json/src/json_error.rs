//! Why a document could not be read.

use core::fmt;

/// A document is not valid JSON.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct JsonError {
    problem: &'static str,
    offset: usize,
}

impl JsonError {
    pub(crate) fn new(problem: &'static str, offset: usize) -> Self {
        Self { problem, offset }
    }

    /// Returns what was wrong.
    #[must_use]
    pub fn problem(&self) -> &'static str {
        self.problem
    }

    /// Returns the byte offset where the problem was found.
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.problem, self.offset)
    }
}

impl core::error::Error for JsonError {}
