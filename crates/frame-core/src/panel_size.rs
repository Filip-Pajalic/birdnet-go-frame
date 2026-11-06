//! The Spectra 6 panels the frame can lay a page out for.

use core::fmt;
use core::str::FromStr;

use crate::resolution::Resolution;

/// A Spectra 6 panel size, by diagonal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PanelSize {
    /// 7.3", 800x480.
    Inches7Point3,
    /// 13.3", 1600x1200, driven as two halves by two T133A01 controllers.
    #[default]
    Inches13Point3,
}

impl PanelSize {
    /// Returns the panel's resolution in its native landscape orientation.
    #[must_use]
    pub fn native_resolution(self) -> Resolution {
        match self {
            Self::Inches7Point3 => Resolution::new(800, 480),
            Self::Inches13Point3 => Resolution::new(1600, 1200),
        }
    }
}

/// The error returned for a panel size other than `7.3` or `13.3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParsePanelSizeError;

impl fmt::Display for ParsePanelSizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("panel size must be 7.3 or 13.3")
    }
}

impl core::error::Error for ParsePanelSizeError {}

impl FromStr for PanelSize {
    type Err = ParsePanelSizeError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.trim() {
            "7.3" => Ok(Self::Inches7Point3),
            "13.3" => Ok(Self::Inches13Point3),
            _ => Err(ParsePanelSizeError),
        }
    }
}
