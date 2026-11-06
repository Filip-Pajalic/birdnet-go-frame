//! The path of BirdNET-Go's hourly counts.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::birdnet_client::query_escape;
use crate::civil_date::CivilDate;

/// The most species BirdNET-Go answers in one hourly batch request.
pub const HOURLY_BATCH: usize = 10;

/// Returns the path under `/api/v2` that asks for `species`' detections in each local hour of
/// `date`, at most [`HOURLY_BATCH`] of them, for clients that make their
/// own requests.
///
/// ```
/// use birdnet_client::CivilDate;
///
/// let date = CivilDate { year: 2026, month: 11, day: 6 };
/// assert_eq!(
///     birdnet_client::hourly_path(date, &["Parus major"]),
///     "/analytics/time/hourly/batch?date=2026-11-06&species=Parus%20major"
/// );
/// ```
#[must_use]
pub fn hourly_path(date: CivilDate, species: &[&str]) -> String {
    let names: Vec<String> = species
        .iter()
        .map(|name| format!("&species={}", query_escape(name)))
        .collect();
    format!("/analytics/time/hourly/batch?date={date}{}", names.concat())
}
