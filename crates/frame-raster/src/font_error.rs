//! The error for a font file that cannot be read.

use core::fmt;

/// The font data could not be parsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct FontError {
    message: &'static str,
}

impl FontError {
    pub(crate) fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unreadable font: {}", self.message)
    }
}

impl core::error::Error for FontError {}
