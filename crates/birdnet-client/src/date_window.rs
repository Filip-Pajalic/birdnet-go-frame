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
    fn applies_the_offset() {
        // 2026-11-05T23:30:00Z is already the 6th at UTC+1.
        let window = DateWindow::hours_back(1_793_921_400, 60, 0);
        assert_eq!(window.end.to_string(), "2026-11-06");
    }
}
