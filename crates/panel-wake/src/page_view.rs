//! Which page the frame asks the server for.

use core::fmt;
use core::str::FromStr;

/// The page a frame shows, which its buttons step through.
///
/// The server reads it from the `view` query parameter of `/panel`. A frame that asks for
/// [`Today`](Self::Today) leaves the parameter out, so the server's default for that frame
/// applies.
///
/// ```
/// use panel_wake::PageView;
///
/// assert_eq!(PageView::Today.next(), PageView::Week);
/// assert_eq!(PageView::Today.previous(), PageView::Season);
/// assert_eq!("rarest".parse(), Ok(PageView::Rarest));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PageView {
    /// The birds heard today: the server's default page.
    #[default]
    Today,
    /// The birds heard this week.
    Week,
    /// The rarest bird lately.
    Rarest,
    /// The day's story: the dawn chorus, the first and last singer, and the weather.
    Story,
    /// The season: arrivals, this week last year, and the year's list.
    Season,
}

impl PageView {
    /// Every view, in the order the buttons step through them.
    pub const ALL: [Self; 5] = [
        Self::Today,
        Self::Week,
        Self::Rarest,
        Self::Story,
        Self::Season,
    ];

    /// The value of the `view` query parameter.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Week => "week",
            Self::Rarest => "rarest",
            Self::Story => "story",
            Self::Season => "season",
        }
    }

    /// The view after this one, wrapping around.
    #[must_use]
    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    /// The view before this one, wrapping around.
    #[must_use]
    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    /// The view as a byte, e.g. for RTC memory.
    #[must_use]
    pub fn to_byte(self) -> u8 {
        self.index() as u8
    }

    /// Reads [`to_byte`](Self::to_byte)'s byte; anything else reads as [`Today`](Self::Today).
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        Self::ALL
            .get(usize::from(byte))
            .copied()
            .unwrap_or_default()
    }

    /// The view's place in [`ALL`](Self::ALL).
    fn index(self) -> usize {
        match self {
            Self::Today => 0,
            Self::Week => 1,
            Self::Rarest => 2,
            Self::Story => 3,
            Self::Season => 4,
        }
    }
}

/// Writes the query value, e.g. `week`.
impl fmt::Display for PageView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The error returned when a view's name is not one of [`PageView::ALL`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParsePageViewError;

impl fmt::Display for ParsePageViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the view must be today, week, rarest, story or season")
    }
}

impl core::error::Error for ParsePageViewError {}

impl FromStr for PageView {
    type Err = ParsePageViewError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text: &str = text.trim();
        Self::ALL
            .into_iter()
            .find(|view| view.as_str().eq_ignore_ascii_case(text))
            .ok_or(ParsePageViewError)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn steps_round_both_ways_and_survives_memory() {
        for view in PageView::ALL {
            assert_eq!(view.next().previous(), view);
            assert_eq!(PageView::from_byte(view.to_byte()), view);
            assert_eq!(view.as_str().parse(), Ok(view));
        }
        assert_eq!(PageView::Season.next(), PageView::Today);
        assert_eq!(PageView::from_byte(200), PageView::Today);
        assert_eq!("monthly".parse::<PageView>(), Err(ParsePageViewError));
    }
}
