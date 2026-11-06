//! A species heard for the first time in a while.

use alloc::vec::Vec;

use birdnet_client::CivilDate;
use birdnet_model::SpeciesSummary;

use crate::stamp;

/// A species and the day it arrived: first heard this year, or first heard after an absence.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrival {
    /// The species, with its count so far this year.
    pub species: SpeciesSummary,
    /// The day it was first heard.
    pub date: CivilDate,
}

impl Arrival {
    /// Returns the species of `recent` (a summary of the last few weeks) that are not in
    /// `before` (a summary of the weeks before those), dated by when they were first heard in
    /// `recent`, newest first: birds back from migration, or new altogether.
    #[must_use]
    pub fn returned(recent: &[SpeciesSummary], before: &[SpeciesSummary]) -> Vec<Self> {
        let mut arrivals: Vec<Self> = recent
            .iter()
            .filter(|row| {
                !before
                    .iter()
                    .any(|held| held.scientific_name == row.scientific_name)
            })
            .filter_map(|row| {
                Some(Self {
                    species: row.clone(),
                    date: stamp::date(row.first_heard.as_deref())?,
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
}

#[cfg(test)]
mod test {
    use alloc::string::String;

    use birdnet_model::ScientificName;

    use super::*;

    fn bird(name: &str, first: &str) -> SpeciesSummary {
        SpeciesSummary {
            scientific_name: ScientificName::new(name),
            common_name: String::from(name),
            count: 1,
            first_heard: Some(String::from(first)),
            last_heard: None,
            max_confidence: None,
        }
    }

    #[test]
    fn finds_birds_back_after_an_absence() {
        let recent = [
            bird("Parus major", "2026-10-20T07:00:00+02:00"),
            bird("Turdus pilaris", "2026-10-25T07:00:00+02:00"),
            bird("Turdus iliacus", "2026-10-30T07:00:00+01:00"),
        ];
        let before = [bird("Parus major", "2026-08-20T07:00:00+02:00")];
        let names: Vec<String> = Arrival::returned(&recent, &before)
            .into_iter()
            .map(|arrival| arrival.species.common_name)
            .collect();
        assert_eq!(names, ["Turdus iliacus", "Turdus pilaris"]);
    }
}
