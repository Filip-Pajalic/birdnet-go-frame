//! What a station's detections say beyond today's list: which birds are new for the year,
//! which have just arrived, which of today's birds are the rarest, how this week compares with
//! the same week last year, and the day's story (who sang first and last, and when the chorus
//! peaked).
//!
//! Everything is worked out from BirdNET-Go's species summaries over longer windows and its
//! hourly counts, so it needs no database of its own. `no_std` with `alloc`.
//!
//! ```
//! use bird_almanac::Almanac;
//! use birdnet_client::{BirdnetClient, DateWindow};
//! use birdnet_mock::MockTransport;
//!
//! let now: i64 = 1_793_955_600; // 2026-11-06T09:00:00Z
//! let mut client = BirdnetClient::new("mock", MockTransport::new(move || now));
//! let year = client.species_summary(DateWindow::year_to_date(now, 0)).expect("mock");
//! let almanac = Almanac::new(DateWindow::local_date(now, 0), year);
//! assert!(almanac.year_list().len() > 20);
//! ```

#![no_std]

extern crate alloc;

mod almanac;
mod arrival;
mod day_story;
mod stamp;
mod year_on_year;

pub use almanac::{Almanac, QUIET_START_DAYS};
pub use arrival::Arrival;
pub use day_story::{DayStory, Singer};
pub use year_on_year::YearOnYear;
