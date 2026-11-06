//! The mass table.

use crate::name_table::NameTable;

/// AVONET masses for the species BirdNET v2.4 can detect.
const BUNDLED: &str = include_str!("../data/bird_sizes.csv");

/// A `scientific_name,mass_g` table, read in place.
///
/// Lookups are a binary search over the text's lines, so the table needs no allocation. The
/// text must be sorted by name, ignoring case; [`BirdSizes::parse_csv`] checks this.
#[derive(Clone, Copy, Debug)]
pub struct BirdSizes<'a> {
    table: NameTable<'a>,
    median_grams: f32,
}

impl BirdSizes<'static> {
    /// Returns the bundled AVONET table.
    #[must_use]
    #[expect(
        clippy::expect_used,
        clippy::missing_panics_doc,
        reason = "the bundled table is checked by a test"
    )]
    pub fn bundled() -> Self {
        Self::parse_csv(BUNDLED).expect("the bundled table is sorted")
    }
}

impl<'a> BirdSizes<'a> {
    /// Reads a table whose first line is a header and whose other lines are
    /// `scientific name,mass in grams`, sorted by name ignoring case. Lines that do not parse
    /// are ignored by lookups.
    ///
    /// Returns `None` if the lines are not sorted.
    #[must_use]
    pub fn parse_csv(text: &'a str) -> Option<Self> {
        let mut sizes = Self {
            table: NameTable::parse_csv(text)?,
            median_grams: 1.0,
        };
        sizes.median_grams = sizes.median();
        Some(sizes)
    }

    /// Returns how many species the table has.
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns whether the table has no species.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the median mass, used for species the table does not know.
    #[must_use]
    pub fn median_grams(&self) -> f32 {
        self.median_grams
    }

    /// Returns the mass of the species with this scientific name, compared ignoring case and
    /// spacing, or [`Self::median_grams`] when the table does not know it.
    #[must_use]
    pub fn mass_grams(&self, scientific_name: &str) -> f32 {
        self.find(scientific_name).unwrap_or(self.median_grams)
    }

    /// Returns the mass of the species, or `None` when the table does not know it.
    #[must_use]
    pub fn find(&self, scientific_name: &str) -> Option<f32> {
        self.table.row(scientific_name)?.trim().parse().ok()
    }

    fn median(&self) -> f32 {
        // Masses are positive, so their bit patterns sort like the numbers; count how many lie
        // under each candidate to find the middle one without a buffer.
        let masses = || {
            self.table
                .rows()
                .filter_map(|(_name, mass)| mass.trim().parse::<f32>().ok())
                .filter(|mass| *mass > 0.0)
        };
        let count: usize = masses().count();
        if count == 0 {
            return 1.0;
        }
        let middle: usize = count / 2;
        let (mut low, mut high): (u32, u32) = (0, f32::MAX.to_bits());
        while low < high {
            let candidate: u32 = low + (high - low) / 2;
            let at_or_below: usize = masses().filter(|mass| mass.to_bits() <= candidate).count();
            if at_or_below > middle {
                high = candidate;
            } else {
                low = candidate + 1;
            }
        }
        f32::from_bits(low)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SMALL: &str =
        "scientific_name,mass_g\nAegithalos caudatus,8.6\nParus major,16.3\nPica pica,217\n";

    #[test]
    fn bundled_table_is_sorted_and_complete() {
        let sizes = BirdSizes::bundled();
        assert_eq!(sizes.len(), 6421);
        // Every row can be found again, which also proves the search reaches the ends.
        for line in BUNDLED.lines().skip(1) {
            let (name, mass) = line.split_once(',').expect("two fields");
            assert_eq!(sizes.find(name), mass.parse().ok(), "{name}");
        }
    }

    #[test]
    fn finds_rows_and_falls_back_to_the_median() {
        let sizes = BirdSizes::parse_csv(SMALL).expect("sorted");
        assert_eq!(sizes.find("aegithalos CAUDATUS"), Some(8.6));
        assert_eq!(sizes.find("Pica pica"), Some(217.0));
        assert_eq!(sizes.find("Aaa"), None);
        assert_eq!(sizes.find("Zzz"), None);
        assert_eq!(sizes.median_grams(), 16.3);
        assert_eq!(sizes.mass_grams("Unknown bird"), 16.3);
    }

    #[test]
    fn rejects_unsorted_tables_and_tolerates_empty_ones() {
        assert!(BirdSizes::parse_csv("h\nPica pica,1\nParus major,2\n").is_none());
        let empty = BirdSizes::parse_csv("").expect("empty is sorted");
        assert!(empty.is_empty());
        assert_eq!(empty.mass_grams("Parus major"), 1.0);
    }
}
