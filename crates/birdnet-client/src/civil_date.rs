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
    use alloc::string::ToString;

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
