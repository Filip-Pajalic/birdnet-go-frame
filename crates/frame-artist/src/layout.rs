//! Which page to draw.

use core::fmt;
use core::str::FromStr;

/// Which page to draw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// A picture of every bird, heavier birds bigger (`collage`, the default).
    #[default]
    Collage,
    /// Names and counts only (`list`).
    List,
}

/// The error returned for a layout other than `collage` or `list`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParseLayoutError;

impl fmt::Display for ParseLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected `collage` or `list`")
    }
}

impl core::error::Error for ParseLayoutError {}

impl FromStr for Layout {
    type Err = ParseLayoutError;

    /// Reads `collage` or `list`, in any case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.eq_ignore_ascii_case("collage") {
            Ok(Self::Collage)
        } else if text.eq_ignore_ascii_case("list") {
            Ok(Self::List)
        } else {
            Err(ParseLayoutError)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_either_case() {
        assert_eq!("LIST".parse::<Layout>(), Ok(Layout::List));
        assert_eq!("collage".parse::<Layout>(), Ok(Layout::Collage));
        assert_eq!("grid".parse::<Layout>(), Err(ParseLayoutError));
    }
}
