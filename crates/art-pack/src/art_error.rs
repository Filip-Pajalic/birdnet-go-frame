//! Why `.artp` bytes could not be read.

use core::fmt;

/// Why `.artp` bytes could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum ArtError {
    /// The magic is not `ARTP` or the data ends inside the header.
    NotArt,
    /// The version is not one this crate reads.
    UnsupportedVersion(u16),
    /// The runs do not add up to the declared size.
    Corrupt,
}

impl fmt::Display for ArtError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotArt => formatter.write_str("not an .artp picture"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported .artp version {version}")
            }
            Self::Corrupt => formatter.write_str("corrupt .artp picture"),
        }
    }
}

impl core::error::Error for ArtError {}
