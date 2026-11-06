//! How the page is turned relative to the panel's native landscape.

use core::fmt;
use core::str::FromStr;

/// How far the page, as the viewer sees it, is turned counter-clockwise from the panel's
/// native landscape. Matches fugleramme's **Rotation** setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Rotation {
    /// Native landscape.
    #[default]
    Degrees0,
    /// A quarter turn counter-clockwise (portrait).
    Degrees90,
    /// Upside down.
    Degrees180,
    /// Three quarter turns counter-clockwise (portrait).
    Degrees270,
}

impl Rotation {
    /// Returns the number of counter-clockwise quarter turns, from 0 to 3.
    #[must_use]
    pub fn quarter_turns(self) -> u8 {
        match self {
            Self::Degrees0 => 0,
            Self::Degrees90 => 1,
            Self::Degrees180 => 2,
            Self::Degrees270 => 3,
        }
    }

    /// Returns the turn in degrees: 0, 90, 180 or 270.
    #[must_use]
    pub fn degrees(self) -> u16 {
        u16::from(self.quarter_turns()) * 90
    }

    /// Returns whether the page is portrait on a landscape panel.
    #[must_use]
    pub fn is_portrait(self) -> bool {
        self.quarter_turns() % 2 == 1
    }
}

/// The error returned when a rotation is not 0, 90, 180 or 270.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseRotationError;

impl fmt::Display for ParseRotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("rotation must be 0, 90, 180 or 270")
    }
}

impl core::error::Error for ParseRotationError {}

impl FromStr for Rotation {
    type Err = ParseRotationError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.trim() {
            "0" => Ok(Self::Degrees0),
            "90" => Ok(Self::Degrees90),
            "180" => Ok(Self::Degrees180),
            "270" => Ok(Self::Degrees270),
            _ => Err(ParseRotationError),
        }
    }
}
