//! A calendar date, computed without an OS clock library.

use core::fmt;

/// A proleptic Gregorian calendar date.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate {
    /// Year, e.g. 2026.
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u8,
    /// Day of the month, 1 to 31.
    pub day: u8,
}

impl CivilDate {
    /// Returns the date `days` after 1970-01-01 (negative for earlier dates).
    ///
    /// Uses Howard Hinnant's `civil_from_days` algorithm.
    #[must_use]
    pub fn from_days_since_epoch(days: i64) -> Self {
        let shifted: i64 = days + 719_468;
        let era: i64 = shifted.div_euclid(146_097);
        let day_of_era: i64 = shifted.rem_euclid(146_097);
        let year_of_era: i64 =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year: i64 =
            day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index: i64 = (5 * day_of_year + 2) / 153;
        let day: i64 = day_of_year - (153 * month_index + 2) / 5 + 1;
        let month: i64 = if month_index < 10 {
            month_index + 3
        } else {
            month_index - 9
        };
        let year: i64 = year_of_era + era * 400 + i64::from(month <= 2);
        Self {
            year: i32::try_from(year).unwrap_or(i32::MAX),
            month: u8::try_from(month).unwrap_or(1),
            day: u8::try_from(day).unwrap_or(1),
        }
    }
}

impl CivilDate {
    /// Returns the days from 1970-01-01 to this date (negative for earlier dates).
    ///
    /// Uses Howard Hinnant's `days_from_civil` algorithm, the inverse of
    /// [`from_days_since_epoch`](Self::from_days_since_epoch).
    #[must_use]
    pub fn days_since_epoch(self) -> i64 {
        let month: i64 = i64::from(self.month);
        let year: i64 = i64::from(self.year) - i64::from(month <= 2);
        let era: i64 = year.div_euclid(400);
        let year_of_era: i64 = year.rem_euclid(400);
        let shifted_month: i64 = if month > 2 { month - 3 } else { month + 9 };
        let day_of_year: i64 = (153 * shifted_month + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era: i64 = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// Reads a date written `YYYY-MM-DD`, alone or at the start of a longer timestamp such as
    /// `2026-11-06T05:12:00+01:00`. Returns `None` when it does not start with a valid date.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let date: &str = text.get(..10)?;
        let mut parts = date.split('-');
        let year: i32 = parts.next()?.parse().ok()?;
        let month: u8 = parts.next()?.parse().ok()?;
        let day: u8 = parts.next()?.parse().ok()?;
        let parsed = Self { year, month, day };
        // Round-tripping through the day count rejects days a month does not have.
        (Self::from_days_since_epoch(parsed.days_since_epoch()) == parsed).then_some(parsed)
    }

    /// Returns the date `days` later (earlier when negative).
    #[must_use]
    pub fn add_days(self, days: i64) -> Self {
        Self::from_days_since_epoch(self.days_since_epoch() + days)
    }

    /// Returns the first of January of this date's year.
    #[must_use]
    pub fn start_of_year(self) -> Self {
        Self {
            year: self.year,
            month: 1,
            day: 1,
        }
    }

    /// Returns the same day a year earlier; the 29th of February becomes the 28th.
    #[must_use]
    pub fn year_earlier(self) -> Self {
        let earlier = Self {
            year: self.year - 1,
            ..self
        };
        if self.month == 2 && self.day == 29 {
            Self { day: 28, ..earlier }
        } else {
            earlier
        }
    }
}

impl fmt::Display for CivilDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

#[cfg(test)]
mod test {
    use alloc::string::{String, ToString};

    use super::*;

    #[test]
    fn epoch() {
        assert_eq!(
            CivilDate::from_days_since_epoch(0),
            CivilDate {
                year: 1970,
                month: 1,
                day: 1
            }
        );
    }

    #[test]
    fn counts_days_both_ways() {
        for days in [-800_000, -1, 0, 19_782, 20_398, 400_000] {
            assert_eq!(
                CivilDate::from_days_since_epoch(days).days_since_epoch(),
                days
            );
        }
    }

    #[test]
    fn parses_dates_and_timestamps() {
        let date = CivilDate::parse("2026-11-06T05:12:00+01:00").expect("a date");
        assert_eq!(date.to_string(), "2026-11-06");
        assert_eq!(date.add_days(-6).to_string(), "2026-10-31");
        assert_eq!(date.start_of_year().to_string(), "2026-01-01");
        assert_eq!(
            CivilDate::parse("2024-02-29")
                .map(CivilDate::year_earlier)
                .map(|date| date.to_string()),
            Some(String::from("2023-02-28"))
        );
        for bad in ["2026-02-30", "2026-13-01", "06/11/2026", "2026-11"] {
            assert_eq!(CivilDate::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn leap_day_and_before_epoch() {
        // 2024-02-29 is day 19_782.
        assert_eq!(
            CivilDate::from_days_since_epoch(19_782).to_string(),
            "2024-02-29"
        );
        assert_eq!(
            CivilDate::from_days_since_epoch(-1).to_string(),
            "1969-12-31"
        );
    }
}
