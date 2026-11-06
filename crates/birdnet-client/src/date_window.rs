//! The date range a species summary is asked for.

use alloc::format;
use alloc::string::String;

use crate::civil_date::CivilDate;

const SECONDS_PER_DAY: i64 = 86_400;

/// An inclusive range of local dates, as BirdNET-Go's summary endpoint takes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateWindow {
    /// First day, inclusive.
    pub start: CivilDate,
    /// Last day, inclusive.
    pub end: CivilDate,
}

impl DateWindow {
    /// Returns the dates from `start` to `end`, both included.
    #[must_use]
    pub fn new(start: CivilDate, end: CivilDate) -> Self {
        Self { start, end }
    }

    /// Returns the local date at `now_unix_seconds` in a time zone `utc_offset_minutes` east of
    /// UTC.
    #[must_use]
    pub fn local_date(now_unix_seconds: i64, utc_offset_minutes: i32) -> CivilDate {
        let local_now: i64 = now_unix_seconds + i64::from(utc_offset_minutes) * 60;
        CivilDate::from_days_since_epoch(local_now.div_euclid(SECONDS_PER_DAY))
    }

    /// Returns the `days` local dates ending today: 1 for today alone, 7 for the last week.
    /// 0 counts as 1.
    #[must_use]
    pub fn days_back(now_unix_seconds: i64, utc_offset_minutes: i32, days: u32) -> Self {
        let today: CivilDate = Self::local_date(now_unix_seconds, utc_offset_minutes);
        Self {
            start: today.add_days(1 - i64::from(days.max(1))),
            end: today,
        }
    }

    /// Returns the local dates from the first of January to today.
    #[must_use]
    pub fn year_to_date(now_unix_seconds: i64, utc_offset_minutes: i32) -> Self {
        let today: CivilDate = Self::local_date(now_unix_seconds, utc_offset_minutes);
        Self {
            start: today.start_of_year(),
            end: today,
        }
    }

    /// Returns the whole calendar year before the one at `now_unix_seconds`, from the first of
    /// January to the last of December.
    #[must_use]
    pub fn previous_year(now_unix_seconds: i64, utc_offset_minutes: i32) -> Self {
        let this_year: CivilDate =
            Self::local_date(now_unix_seconds, utc_offset_minutes).start_of_year();
        Self {
            start: this_year.year_earlier(),
            end: this_year.add_days(-1),
        }
    }

    /// Returns the same dates a year earlier.
    #[must_use]
    pub fn year_earlier(self) -> Self {
        Self {
            start: self.start.year_earlier(),
            end: self.end.year_earlier(),
        }
    }

    /// Returns whether `date` is in the window.
    #[must_use]
    pub fn contains(self, date: CivilDate) -> bool {
        self.start <= date && date <= self.end
    }

    /// Returns the local dates covering the last `hours` before `now_unix_seconds`, in a time
    /// zone `utc_offset_minutes` east of UTC. The range rounds outwards to whole days, so it may
    /// count a little more than `hours`.
    #[must_use]
    pub fn hours_back(now_unix_seconds: i64, utc_offset_minutes: i32, hours: u32) -> Self {
        let local_now: i64 = now_unix_seconds + i64::from(utc_offset_minutes) * 60;
        let local_start: i64 = local_now - i64::from(hours) * 3_600;
        Self {
            start: CivilDate::from_days_since_epoch(local_start.div_euclid(SECONDS_PER_DAY)),
            end: CivilDate::from_days_since_epoch(local_now.div_euclid(SECONDS_PER_DAY)),
        }
    }

    /// Returns the path and query of BirdNET-Go's species summary for this window, as
    /// [`crate::BirdnetClient::species_summary`] asks for it, for a device that sends its own
    /// requests: `/api/v2/analytics/species/summary?start_date=2026-11-05&end_date=2026-11-06`.
    #[must_use]
    pub fn species_summary_path(self) -> String {
        format!(
            "/api/v2/analytics/species/summary?start_date={}&end_date={}",
            self.start, self.end
        )
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn names_the_summary_path() {
        // 2026-11-06T07:00:00Z, 6 hours back in UTC.
        assert_eq!(
            DateWindow::hours_back(1_793_948_400, 0, 6).species_summary_path(),
            "/api/v2/analytics/species/summary?start_date=2026-11-06&end_date=2026-11-06"
        );
    }

    #[test]
    fn crosses_midnight() {
        // 2026-11-06T01:00:00Z, 6 hours back reaches 2026-11-05.
        let window = DateWindow::hours_back(1_793_926_800, 0, 6);
        assert_eq!(window.start.to_string(), "2026-11-05");
        assert_eq!(window.end.to_string(), "2026-11-06");
    }

    #[test]
    fn whole_days_back() {
        // 2026-11-06T09:00:00Z.
        let week = DateWindow::days_back(1_793_955_600, 0, 7);
        assert_eq!(week.start.to_string(), "2026-10-31");
        assert_eq!(week.end.to_string(), "2026-11-06");
        let year = DateWindow::year_to_date(1_793_955_600, 0);
        assert_eq!(year.start.to_string(), "2026-01-01");
        assert_eq!(week.year_earlier().start.to_string(), "2025-10-31");
        assert!(week.contains(year.end) && !week.contains(year.start));
        let last = DateWindow::previous_year(1_793_955_600, 0);
        assert_eq!(last.start.to_string(), "2025-01-01");
        assert_eq!(last.end.to_string(), "2025-12-31");
    }

    #[test]
    fn applies_the_offset() {
        // 2026-11-05T23:30:00Z is already the 6th at UTC+1.
        let window = DateWindow::hours_back(1_793_921_400, 60, 0);
        assert_eq!(window.end.to_string(), "2026-11-06");
    }
}
