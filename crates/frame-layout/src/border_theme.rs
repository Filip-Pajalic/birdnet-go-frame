//! Which plants the page's border wears.

use core::fmt;
use core::str::FromStr;

/// A seasonal or holiday border: which plants' [`crate::CornerSprays`] the page wears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BorderTheme {
    /// Winter: mistletoe and Christmas rose.
    Frost,
    /// Spring: bird cherry in flower and wood anemone.
    Blossom,
    /// Summer: cornflower and ox-eye daisy.
    Meadow,
    /// Autumn: rowan berries, oak and acorns.
    Autumn,
    /// From Lucia (13 December) to Epiphany (6 January): holly and mistletoe.
    Christmas,
    /// From Palm Sunday to Easter Monday: pussy willow and wood anemone.
    Easter,
    /// Midsummer, 19 to 25 June: ox-eye daisy and cornflower.
    Midsummer,
}

impl BorderTheme {
    /// Every theme.
    pub const ALL: [Self; 7] = [
        Self::Frost,
        Self::Blossom,
        Self::Meadow,
        Self::Autumn,
        Self::Christmas,
        Self::Easter,
        Self::Midsummer,
    ];

    /// Returns the theme's name, as settings take it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Frost => "frost",
            Self::Blossom => "blossom",
            Self::Meadow => "meadow",
            Self::Autumn => "autumn",
            Self::Christmas => "christmas",
            Self::Easter => "easter",
            Self::Midsummer => "midsummer",
        }
    }

    /// Returns the theme for the date `year`-`month`-`day`: a holiday's when one is on, else
    /// the season's. `southern` turns the seasons round for a garden south of the equator,
    /// where Midsummer is not kept in June.
    ///
    /// ```
    /// use frame_layout::BorderTheme;
    ///
    /// assert_eq!(BorderTheme::for_date(2026, 11, 6, false), BorderTheme::Autumn);
    /// assert_eq!(BorderTheme::for_date(2026, 12, 24, false), BorderTheme::Christmas);
    /// assert_eq!(BorderTheme::for_date(2027, 3, 28, false), BorderTheme::Easter);
    /// assert_eq!(BorderTheme::for_date(2026, 7, 1, true), BorderTheme::Frost);
    /// ```
    #[must_use]
    pub fn for_date(year: i32, month: u8, day: u8, southern: bool) -> Self {
        if (month == 12 && day >= 13) || (month == 1 && day <= 6) {
            return Self::Christmas;
        }
        let day_of_year: i32 = day_of_year(year, month, day);
        let easter: i32 = easter_day_of_year(year);
        if (easter - 7..=easter + 1).contains(&day_of_year) {
            return Self::Easter;
        }
        if !southern && month == 6 && (19..=25).contains(&day) {
            return Self::Midsummer;
        }
        let month: u8 = if southern {
            (month + 5) % 12 + 1
        } else {
            month
        };
        match month {
            3..=5 => Self::Blossom,
            6..=8 => Self::Meadow,
            9..=11 => Self::Autumn,
            _ => Self::Frost,
        }
    }
}

impl fmt::Display for BorderTheme {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// The error returned for a name that is not one of [`BorderTheme::ALL`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParseBorderThemeError;

impl fmt::Display for ParseBorderThemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "expected `frost`, `blossom`, `meadow`, `autumn`, `christmas`, `easter` or \
             `midsummer`",
        )
    }
}

impl core::error::Error for ParseBorderThemeError {}

impl FromStr for BorderTheme {
    type Err = ParseBorderThemeError;

    /// Reads a theme's name, in any case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|theme| theme.name().eq_ignore_ascii_case(text.trim()))
            .ok_or(ParseBorderThemeError)
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// 1 for the first of January.
fn day_of_year(year: i32, month: u8, day: u8) -> i32 {
    const BEFORE: [i32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let index: usize = usize::from(month.clamp(1, 12)) - 1;
    let leap: i32 = i32::from(month > 2 && is_leap(year));
    BEFORE[index] + leap + i32::from(day)
}

/// Western Easter Sunday's day of the year, by the anonymous Gregorian algorithm.
fn easter_day_of_year(year: i32) -> i32 {
    let a: i32 = year.rem_euclid(19);
    let b: i32 = year.div_euclid(100);
    let c: i32 = year.rem_euclid(100);
    let d: i32 = b / 4;
    let e: i32 = b % 4;
    let f: i32 = (b + 8) / 25;
    let g: i32 = (b - f + 1) / 3;
    let h: i32 = (19 * a + b - d - g + 15) % 30;
    let i: i32 = c / 4;
    let k: i32 = c % 4;
    let l: i32 = (32 + 2 * e + 2 * i - h - k) % 7;
    let m: i32 = (a + 11 * h + 22 * l) / 451;
    let month: i32 = (h + l - 7 * m + 114) / 31;
    let day: i32 = (h + l - 7 * m + 114) % 31 + 1;
    day_of_year(
        year,
        u8::try_from(month).unwrap_or(4),
        u8::try_from(day).unwrap_or(1),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_easter() {
        // 2024-03-31, 2025-04-20, 2026-04-05, 2027-03-28 (day 87).
        assert_eq!(easter_day_of_year(2024), day_of_year(2024, 3, 31));
        assert_eq!(easter_day_of_year(2025), day_of_year(2025, 4, 20));
        assert_eq!(easter_day_of_year(2026), day_of_year(2026, 4, 5));
        assert_eq!(easter_day_of_year(2027), 87);
    }

    #[test]
    fn holidays_then_seasons() {
        let on = |month: u8, day: u8| BorderTheme::for_date(2026, month, day, false);
        assert_eq!(on(1, 6), BorderTheme::Christmas);
        assert_eq!(on(1, 7), BorderTheme::Frost);
        assert_eq!(on(3, 28), BorderTheme::Blossom);
        assert_eq!(on(3, 29), BorderTheme::Easter);
        assert_eq!(on(4, 6), BorderTheme::Easter);
        assert_eq!(on(4, 7), BorderTheme::Blossom);
        assert_eq!(on(6, 18), BorderTheme::Meadow);
        assert_eq!(on(6, 19), BorderTheme::Midsummer);
        assert_eq!(on(6, 26), BorderTheme::Meadow);
        assert_eq!(on(9, 1), BorderTheme::Autumn);
        assert_eq!(on(12, 12), BorderTheme::Frost);
        assert_eq!(BorderTheme::for_date(2026, 6, 21, true), BorderTheme::Frost);
        assert_eq!(
            BorderTheme::for_date(2026, 10, 1, true),
            BorderTheme::Blossom
        );
    }

    #[test]
    fn names_round_trip() {
        for theme in BorderTheme::ALL {
            assert_eq!(theme.name().parse(), Ok(theme));
        }
        assert_eq!("Frost".parse(), Ok(BorderTheme::Frost));
        assert_eq!(
            "halloween".parse::<BorderTheme>(),
            Err(ParseBorderThemeError)
        );
    }
}
