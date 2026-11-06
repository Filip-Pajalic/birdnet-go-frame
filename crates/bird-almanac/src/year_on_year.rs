//! This week against the same week last year.

use alloc::vec::Vec;

use birdnet_model::SpeciesSummary;

/// How the species of one window compare with those of the same window a year earlier.
#[derive(Clone, Debug, PartialEq)]
pub struct YearOnYear {
    /// Species heard now.
    pub now: usize,
    /// Species heard then.
    pub then: usize,
    /// Heard now but not then, most detections first.
    pub gained: Vec<SpeciesSummary>,
    /// Heard then but not now, most detections first.
    pub missing: Vec<SpeciesSummary>,
}

impl YearOnYear {
    /// Compares `now` with `then`, both summaries most detections first.
    #[must_use]
    pub fn new(now: &[SpeciesSummary], then: &[SpeciesSummary]) -> Self {
        let absent = |row: &&SpeciesSummary, other: &[SpeciesSummary]| {
            !other
                .iter()
                .any(|held| held.scientific_name == row.scientific_name)
        };
        Self {
            now: now.len(),
            then: then.len(),
            gained: now
                .iter()
                .filter(|row| absent(row, then))
                .cloned()
                .collect(),
            missing: then
                .iter()
                .filter(|row| absent(row, now))
                .cloned()
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::string::String;

    use birdnet_model::ScientificName;

    use super::*;

    fn bird(name: &str) -> SpeciesSummary {
        SpeciesSummary {
            scientific_name: ScientificName::new(name),
            common_name: String::from(name),
            count: 1,
            first_heard: None,
            last_heard: None,
            max_confidence: None,
        }
    }

    #[test]
    fn sorts_gains_from_losses() {
        let now = [bird("Parus major"), bird("Bombycilla garrulus")];
        let then = [bird("Parus major"), bird("Turdus iliacus"), bird("Pica pica")];
        let compared = YearOnYear::new(&now, &then);
        assert_eq!((compared.now, compared.then), (2, 3));
        assert_eq!(compared.gained[0].common_name, "Bombycilla garrulus");
        assert_eq!(compared.missing.len(), 2);
    }
}
