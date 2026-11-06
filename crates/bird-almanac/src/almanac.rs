//! The year so far.

use alloc::vec::Vec;

use birdnet_client::CivilDate;
use birdnet_model::SpeciesSummary;

use crate::arrival::Arrival;
use crate::stamp;

/// Days at the start of a year in which nothing counts as new or as an arrival: on the first
/// of January every bird is new for the year, and the residents take a while to all be heard.
pub const QUIET_START_DAYS: i64 = 14;

/// The species heard this year, and what they say about today.
#[derive(Clone, Debug, PartialEq)]
pub struct Almanac {
    today: CivilDate,
    year: Vec<SpeciesSummary>,
}

impl Almanac {
    /// Creates the almanac for `today` from the year-to-date summary `year` (as
    /// `DateWindow::year_to_date` asks for it), whose `first_heard` says when each species
    /// was first heard this year.
    #[must_use]
    pub fn new(today: CivilDate, year: Vec<SpeciesSummary>) -> Self {
        Self { today, year }
    }

    /// Returns the day the almanac is for.
    #[must_use]
    pub fn today(&self) -> CivilDate {
        self.today
    }

    /// Returns every species heard this year, most detections first.
    #[must_use]
    pub fn year_list(&self) -> &[SpeciesSummary] {
        &self.year
    }

    /// Returns whether `date` is late enough in the year for a first sighting to mean much.
    fn counts(&self, date: CivilDate) -> bool {
        date.days_since_epoch() - date.start_of_year().days_since_epoch() >= QUIET_START_DAYS
    }

    /// Returns the species first heard this year in the last `days` days (today included),
    /// newest first. Empty in the first [`QUIET_START_DAYS`] of the year.
    #[must_use]
    pub fn arrivals(&self, days: u32) -> Vec<Arrival> {
        let since: CivilDate = self.today.add_days(1 - i64::from(days.max(1)));
        let mut arrivals: Vec<Arrival> = self
            .year
            .iter()
            .filter_map(|row| {
                let date: CivilDate = stamp::date(row.first_heard.as_deref())?;
                (date >= since && date <= self.today && self.counts(date)).then(|| Arrival {
                    species: row.clone(),
                    date,
                })
            })
            .collect();
        arrivals.sort_by(|left, right| {
            right
                .date
                .cmp(&left.date)
                .then_with(|| left.species.common_name.cmp(&right.species.common_name))
        });
        arrivals
    }

    /// Returns the species heard today for the first time this year: the rare-bird alert.
    #[must_use]
    pub fn new_today(&self) -> Vec<SpeciesSummary> {
        self.arrivals(1)
            .into_iter()
            .map(|arrival| arrival.species)
            .collect()
    }

    /// Returns how many times `species` was heard this year, 0 if never.
    #[must_use]
    pub fn year_count(&self, species: &SpeciesSummary) -> u32 {
        self.year
            .iter()
            .find(|row| row.scientific_name == species.scientific_name)
            .map_or(0, |row| row.count)
    }

    /// Returns `heard` (e.g. today's species) rarest first: fewest detections this year, then
    /// fewest in `heard`.
    #[must_use]
    pub fn rarest(&self, heard: &[SpeciesSummary]) -> Vec<SpeciesSummary> {
        let mut ranked: Vec<(u32, SpeciesSummary)> = heard
            .iter()
            .map(|row| (self.year_count(row).max(row.count), row.clone()))
            .collect();
        ranked.sort_by(|(left_year, left), (right_year, right)| {
            left_year
                .cmp(right_year)
                .then_with(|| left.count.cmp(&right.count))
                .then_with(|| left.common_name.cmp(&right.common_name))
        });
        ranked.into_iter().map(|(_, row)| row).collect()
    }
}

#[cfg(test)]
mod test {
    use alloc::string::String;

    use birdnet_model::ScientificName;

    use super::*;

    fn bird(name: &str, count: u32, first: &str) -> SpeciesSummary {
        SpeciesSummary {
            scientific_name: ScientificName::new(name),
            common_name: String::from(name),
            count,
            first_heard: Some(String::from(first)),
            last_heard: None,
            max_confidence: None,
        }
    }

    fn date(text: &str) -> CivilDate {
        CivilDate::parse(text).expect("a date")
    }

    fn year() -> Vec<SpeciesSummary> {
        alloc::vec![
            bird("Parus major", 900, "2026-01-01T07:05:00+01:00"),
            bird("Turdus pilaris", 40, "2026-10-30T08:05:00+01:00"),
            bird("Bombycilla garrulus", 3, "2026-11-06T09:05:00+01:00"),
            bird("Turdus iliacus", 25, "2026-11-02T08:05:00+01:00"),
        ]
    }

    #[test]
    fn finds_new_birds_and_arrivals() {
        let almanac = Almanac::new(date("2026-11-06"), year());
        let new: Vec<SpeciesSummary> = almanac.new_today();
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].scientific_name.as_str(), "Bombycilla garrulus");
        let arrivals: Vec<Arrival> = almanac.arrivals(14);
        let names: Vec<&str> = arrivals
            .iter()
            .map(|arrival| arrival.species.scientific_name.as_str())
            .collect();
        assert_eq!(
            names,
            ["Bombycilla garrulus", "Turdus iliacus", "Turdus pilaris"]
        );
    }

    #[test]
    fn early_january_is_quiet() {
        let almanac = Almanac::new(
            date("2026-01-01"),
            alloc::vec![bird("Parus major", 9, "2026-01-01T07:05:00+01:00")],
        );
        assert!(almanac.new_today().is_empty());
    }

    #[test]
    fn ranks_the_rarest_first() {
        let almanac = Almanac::new(date("2026-11-06"), year());
        let today: Vec<SpeciesSummary> = alloc::vec![
            bird("Parus major", 30, ""),
            bird("Bombycilla garrulus", 3, ""),
            bird("Pica pica", 1, ""),
        ];
        let rarest: Vec<SpeciesSummary> = almanac.rarest(&today);
        let names: Vec<&str> = rarest
            .iter()
            .map(|row| row.scientific_name.as_str())
            .collect();
        assert_eq!(names, ["Pica pica", "Bombycilla garrulus", "Parus major"]);
    }
}
