//! The error for a page that is not the size its panel needs.

use core::fmt;

/// The page is not the size [`panel_bytes::page_resolution`](crate::panel_bytes::page_resolution)
/// gives for its panel and rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct PageSizeError;

impl fmt::Display for PageSizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("page is not the size its panel or screen needs")
    }
}

impl core::error::Error for PageSizeError {}
