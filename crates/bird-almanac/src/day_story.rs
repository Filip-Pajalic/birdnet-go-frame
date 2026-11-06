//! The day so far: who sang first and last, and when the chorus peaked.

use alloc::string::String;
use alloc::vec::Vec;

use birdnet_client::CivilDate;
use birdnet_model::{SpeciesHours, SpeciesSummary};

use crate::stamp;

/// A species and the local time it sang, `HH:MM`.
#[derive(Clone, Debug, PartialEq)]
pub struct Singer {
    /// The species.
    pub species: SpeciesSummary,
    /// When, `HH:MM`.
    pub time: String,
}

/// The story of one day.
#[derive(Clone, Debug, PartialEq)]
pub struct DayStory {
    /// The first bird heard today.
    pub first: Option<Singer>,
    /// The latest bird heard so far.
    pub last: Option<Singer>,
    /// Detections in each local hour, every species together.
    pub chorus: [u32; 24],
    /// The busiest hour so far, if anything was heard.
    pub peak_hour: Option<u32>,
}

impl DayStory {
    /// Tells `date`'s story from its summary (`DateWindow::days_back(now, offset, 1)`) and
    /// the hourly counts of its species. Times on other days are ignored, so a summary that
    /// reaches back into yesterday still tells today's story.
    #[must_use]
    pub fn new(date: CivilDate, species: &[SpeciesSummary], hours: &[SpeciesHours]) -> Self {
        let singer = |row: &SpeciesSummary, stamp: Option<&String>| -> Option<Singer> {
            let stamp: &str = stamp?;
            if stamp::date(Some(stamp)) != Some(date) {
                return None;
            }
            Some(Singer {
                species: row.clone(),
                time: String::from(stamp::clock(stamp)?),
            })
        };
        let firsts: Vec<Singer> = species
            .iter()
            .filter_map(|row| singer(row, row.first_heard.as_ref()))
            .collect();
        let lasts: Vec<Singer> = species
            .iter()
            .filter_map(|row| singer(row, row.last_heard.as_ref()))
            .collect();
        let mut chorus: [u32; 24] = [0; 24];
        for row in hours {
            for (total, &count) in chorus.iter_mut().zip(&row.hours) {
                *total = total.saturating_add(count);
            }
        }
        let peak_hour: Option<u32> = chorus
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            // The earliest of equally busy hours.
            .max_by(|(left_hour, left), (right_hour, right)| {
                left.cmp(right).then_with(|| right_hour.cmp(left_hour))
            })
            .and_then(|(hour, _)| u32::try_from(hour).ok());
        Self {
            first: firsts.into_iter().min_by(|left, right| left.time.cmp(&right.time)),
            last: lasts.into_iter().max_by(|left, right| left.time.cmp(&right.time)),
            chorus,
            peak_hour,
        }
    }
}

#[cfg(test)]
mod test {
    use birdnet_model::ScientificName;

    use super::*;

    fn bird(name: &str, first: &str, last: &str) -> SpeciesSummary {
        SpeciesSummary {
            scientific_name: ScientificName::new(name),
            common_name: String::from(name),
            count: 1,
            first_heard: Some(String::from(first)),
            last_heard: Some(String::from(last)),
            max_confidence: None,
        }
    }

    #[test]
    fn finds_first_and_last_singers_of_the_day() {
        let date: CivilDate = CivilDate::parse("2026-11-06").expect("a date");
        let species = [
            bird("Turdus merula", "2026-11-06T05:05:00+01:00", "2026-11-06T16:55:00+01:00"),
            // First heard yesterday, so it cannot be today's first.
            bird("Strix aluco", "2026-11-05T23:05:00+01:00", "2026-11-06T04:55:00+01:00"),
            bird("Parus major", "2026-11-06T06:05:00+01:00", "2026-11-06T17:55:00+01:00"),
        ];
        let mut hours: [u32; 24] = [0; 24];
        hours[6] = 9;
        hours[7] = 9;
        let counts = [SpeciesHours {
            scientific_name: ScientificName::new("Parus major"),
            hours,
        }];
        let story = DayStory::new(date, &species, &counts);
        let first: Singer = story.first.expect("a first singer");
        assert_eq!((first.species.common_name.as_str(), first.time.as_str()), ("Turdus merula", "05:05"));
        assert_eq!(story.last.map(|singer| singer.time), Some(String::from("17:55")));
        assert_eq!(story.peak_hour, Some(6));
    }
}
