//! A species' scientific name, the key the frame matches everything on.

use alloc::string::String;
use core::fmt;

/// A scientific name such as `Parus major`, compared case- and whitespace-insensitively.
#[derive(Clone, Debug)]
pub struct ScientificName(String);

impl ScientificName {
    /// Creates a name from text as BirdNET-Go spells it.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(String::from(name))
    }

    /// Returns the name as given.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns whether the name has the shape of a binomial (`Genus species`). Single-word
    /// labels such as `Engine` do not.
    #[must_use]
    pub fn is_binomial(&self) -> bool {
        let mut words = self.0.split_whitespace();
        let (Some(genus), Some(species)) = (words.next(), words.next()) else {
            return false;
        };
        // Case is not checked: BirdNET-Go has stored names like `parus major`.
        let genus_ok: bool = genus.chars().all(char::is_alphabetic);
        let species_ok: bool = species
            .chars()
            .all(|letter| letter.is_alphabetic() || letter == '-');
        genus_ok && species_ok
    }

    fn normalized_words(&self) -> impl Iterator<Item = char> + '_ {
        self.0
            .split_whitespace()
            .flat_map(|word| word.chars().chain(Some(' ')))
            .flat_map(char::to_lowercase)
    }
}

impl PartialEq for ScientificName {
    fn eq(&self, other: &Self) -> bool {
        self.normalized_words().eq(other.normalized_words())
    }
}

impl Eq for ScientificName {}

impl fmt::Display for ScientificName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compares_without_case_or_spacing() {
        assert_eq!(
            ScientificName::new("Parus major"),
            ScientificName::new(" parus  Major ")
        );
        assert_ne!(
            ScientificName::new("Parus major"),
            ScientificName::new("Parus minor")
        );
    }

    #[test]
    fn spots_binomials() {
        assert!(ScientificName::new("Erithacus rubecula").is_binomial());
        assert!(!ScientificName::new("Engine").is_binomial());
        // "Human vocal" passes this check; SpeciesSummary::is_bird drops it because BirdNET
        // gives it the same text as its common name.
        assert!(ScientificName::new("Human vocal").is_binomial());
    }
}
