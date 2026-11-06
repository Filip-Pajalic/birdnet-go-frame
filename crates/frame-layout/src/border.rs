//! Whether a frame's pages wear a border, and which.

use core::fmt;
use core::str::FromStr;

use crate::border_theme::{BorderTheme, ParseBorderThemeError};

/// A frame's border setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Border {
    /// The season's or holiday's border, changing with the date (`season`, the default).
    #[default]
    Season,
    /// No border (`off`).
    Off,
    /// One theme all year, e.g. `frost`.
    Always(BorderTheme),
}

impl Border {
    /// Returns the theme to draw on `year`-`month`-`day`, or `None` for no border. See
    /// [`BorderTheme::for_date`] for `southern`.
    #[must_use]
    pub fn theme(self, year: i32, month: u8, day: u8, southern: bool) -> Option<BorderTheme> {
        match self {
            Self::Season => Some(BorderTheme::for_date(year, month, day, southern)),
            Self::Off => None,
            Self::Always(theme) => Some(theme),
        }
    }
}

impl fmt::Display for Border {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Season => formatter.write_str("season"),
            Self::Off => formatter.write_str("off"),
            Self::Always(theme) => write!(formatter, "{theme}"),
        }
    }
}

/// The error returned for a border that is not `season`, `off` or a theme's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParseBorderError;

impl fmt::Display for ParseBorderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "expected `season`, `off`, or one of `frost`, `blossom`, `meadow`, `autumn`, \
             `christmas`, `easter` and `midsummer`",
        )
    }
}

impl core::error::Error for ParseBorderError {}

impl From<ParseBorderThemeError> for ParseBorderError {
    fn from(_: ParseBorderThemeError) -> Self {
        Self
    }
}

impl FromStr for Border {
    type Err = ParseBorderError;

    /// Reads `season` (or `on`), `off` (or `none`), or a theme's name, in any case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text: &str = text.trim();
        if ["season", "on", "true"]
            .iter()
            .any(|word| word.eq_ignore_ascii_case(text))
        {
            Ok(Self::Season)
        } else if ["off", "none", "false"]
            .iter()
            .any(|word| word.eq_ignore_ascii_case(text))
        {
            Ok(Self::Off)
        } else {
            let theme: BorderTheme = text.parse()?;
            Ok(Self::Always(theme))
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn reads_and_writes_every_setting() {
        for border in [
            Border::Season,
            Border::Off,
            Border::Always(BorderTheme::Easter),
        ] {
            assert_eq!(border.to_string().parse(), Ok(border));
        }
        assert_eq!("NONE".parse(), Ok(Border::Off));
        assert_eq!("tinsel".parse::<Border>(), Err(ParseBorderError));
        assert_eq!(Border::Off.theme(2026, 12, 24, false), None);
        assert_eq!(
            Border::Season.theme(2026, 12, 24, false),
            Some(BorderTheme::Christmas)
        );
    }
}
