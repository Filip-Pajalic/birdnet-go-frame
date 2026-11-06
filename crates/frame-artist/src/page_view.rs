//! Which of a frame's pages to draw.

use core::fmt;
use core::str::FromStr;

/// One of the pages a frame can show, chosen by `view=` or a frame's setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PageView {
    /// The birds heard recently, as the layout draws them (`today`, the default).
    #[default]
    Today,
    /// The birds heard in the last seven days (`week`).
    Week,
    /// Today's birds, rarest this year first and biggest (`rarest`).
    Rarest,
    /// The day's story: first and last singers, weather, and when each bird sang (`story`).
    Story,
    /// The year so far: arrivals, this week last year, and the year list (`season`).
    Season,
}

impl PageView {
    /// Every view, in the order a frame's button cycles through them.
    pub const ALL: [Self; 5] = [
        Self::Today,
        Self::Week,
        Self::Rarest,
        Self::Story,
        Self::Season,
    ];

    /// Returns the view's name, as `view=` takes it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Week => "week",
            Self::Rarest => "rarest",
            Self::Story => "story",
            Self::Season => "season",
        }
    }
}

impl fmt::Display for PageView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// The error returned for a view name that is not one of [`PageView::ALL`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParseViewError;

impl fmt::Display for ParseViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected `today`, `week`, `rarest`, `story` or `season`")
    }
}

impl core::error::Error for ParseViewError {}

impl FromStr for PageView {
    type Err = ParseViewError;

    /// Reads a view's name, in any case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|view| view.name().eq_ignore_ascii_case(text.trim()))
            .ok_or(ParseViewError)
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn names_round_trip() {
        for view in PageView::ALL {
            assert_eq!(view.to_string().parse::<PageView>(), Ok(view));
        }
        assert_eq!("Rarest".parse::<PageView>(), Ok(PageView::Rarest));
        assert_eq!("month".parse::<PageView>(), Err(ParseViewError));
    }
}
