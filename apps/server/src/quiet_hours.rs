//! Hours when the panel need not wake: it sleeps through them in one go.

use std::fmt;
use std::str::FromStr;

const DAY: u32 = 24 * 60 * 60;

/// A daily span of local hours, e.g. 22 to 6, that may wrap past midnight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuietHours {
    /// The hour the quiet starts, 0 to 23.
    start: u8,
    /// The hour it ends, 0 to 23.
    end: u8,
}

impl QuietHours {
    /// Seconds a panel asking at `local_seconds` past local midnight should sleep: until the
    /// quiet ends, if it has begun, and never less than `poll_seconds`.
    #[must_use]
    pub fn sleep_seconds(self, local_seconds: u32, poll_seconds: u32) -> u32 {
        let now: u32 = local_seconds % DAY;
        let (start, end): (u32, u32) = (u32::from(self.start) * 3600, u32::from(self.end) * 3600);
        let quiet: bool = if start <= end {
            (start..end).contains(&now)
        } else {
            now >= start || now < end
        };
        if !quiet {
            return poll_seconds;
        }
        let until_end: u32 = (end + DAY - now) % DAY;
        until_end.max(poll_seconds)
    }
}

/// Why `QUIET_HOURS` could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseQuietHoursError;

impl fmt::Display for ParseQuietHoursError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("quiet hours must be two different hours, e.g. 22-6")
    }
}

impl std::error::Error for ParseQuietHoursError {}

impl FromStr for QuietHours {
    type Err = ParseQuietHoursError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (start, end) = text.trim().split_once('-').ok_or(ParseQuietHoursError)?;
        let hour = |value: &str| -> Result<u8, ParseQuietHoursError> {
            value
                .trim()
                .parse::<u8>()
                .ok()
                .filter(|hour| *hour < 24)
                .ok_or(ParseQuietHoursError)
        };
        let hours = Self {
            start: hour(start)?,
            end: hour(end)?,
        };
        if hours.start == hours.end {
            return Err(ParseQuietHoursError);
        }
        Ok(hours)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sleeps_through_the_night_in_one_go() {
        let night: QuietHours = "22-6".parse().expect("valid");
        let poll: u32 = 900;
        assert_eq!(
            night.sleep_seconds(21 * 3600, poll),
            poll,
            "evening: as usual"
        );
        assert_eq!(night.sleep_seconds(22 * 3600, poll), 8 * 3600);
        assert_eq!(night.sleep_seconds(3 * 3600 + 1800, poll), 2 * 3600 + 1800);
        assert_eq!(
            night.sleep_seconds(6 * 3600 - 60, poll),
            poll,
            "never under the poll"
        );
        assert_eq!(
            night.sleep_seconds(6 * 3600, poll),
            poll,
            "morning: as usual"
        );
        assert_eq!(night.sleep_seconds(DAY + 23 * 3600, poll), 7 * 3600);
    }

    #[test]
    fn spans_within_a_day_work_too() {
        let lunch: QuietHours = "12-14".parse().expect("valid");
        assert_eq!(lunch.sleep_seconds(13 * 3600, 600), 3600);
        assert_eq!(lunch.sleep_seconds(15 * 3600, 600), 600);
    }

    #[test]
    fn rejects_nonsense() {
        for text in ["", "22", "22-22", "25-6", "a-b", "22-6-1"] {
            assert!(text.parse::<QuietHours>().is_err(), "{text}");
        }
    }
}
