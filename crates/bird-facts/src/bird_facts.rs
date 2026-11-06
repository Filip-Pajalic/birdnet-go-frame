//! The bundled fact table.

use bird_sizes::{BirdSizes, NameTable};

use crate::fact::{Diet, Fact, Lifestyle, Migration};

/// AVONET's and Wikidata's facts for the species BirdNET v2.4 can detect.
const BUNDLED: &str = include_str!("../data/bird_facts.csv");

/// Latitude from which "as far north (or south) as" is worth saying.
const FAR_LATITUDE: u8 = 55;

/// The table's columns after the name, in order.
const COLUMNS: usize = 9;

/// A table of facts per species, read in place, with AVONET's masses beside it.
///
/// The table's columns are `scientific_name`, `wingspan_cm`, `incubation_days`,
/// `oldest_years`, `range_kkm2` (thousands of km²), `north` and `south` (the range's extreme
/// latitudes, south negative), `migration` (1 resident, 2 partial, 3 migrant), `diet` and
/// `lifestyle` (one-letter codes); any may be empty. Rows are sorted by name ignoring case.
#[derive(Clone, Copy, Debug)]
pub struct BirdFacts<'a> {
    table: NameTable<'a>,
    sizes: BirdSizes<'a>,
}

impl BirdFacts<'static> {
    /// Returns the bundled table.
    #[must_use]
    #[expect(
        clippy::expect_used,
        clippy::missing_panics_doc,
        reason = "the bundled table is checked by a test"
    )]
    pub fn bundled() -> Self {
        Self::new(
            NameTable::parse_csv(BUNDLED).expect("the bundled table is sorted"),
            BirdSizes::bundled(),
        )
    }
}

impl<'a> BirdFacts<'a> {
    /// Creates a table from `table`, with the columns described on [`BirdFacts`], and the
    /// masses in `sizes`.
    #[must_use]
    pub fn new(table: NameTable<'a>, sizes: BirdSizes<'a>) -> Self {
        Self { table, sizes }
    }

    /// Returns how many species have facts besides their mass.
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns whether the table has no species.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns every fact known about the species with this scientific name, compared
    /// ignoring case and spacing, in a fixed order. Empty for a species neither table knows.
    pub fn facts(&self, scientific_name: &str) -> impl Iterator<Item = Fact> + 'a {
        let mut fields: [&'a str; COLUMNS] = [""; COLUMNS];
        if let Some(row) = self.table.row(scientific_name) {
            for (field, value) in fields.iter_mut().zip(row.split(',')) {
                *field = value.trim();
            }
        }
        let weight: Option<Fact> = self
            .sizes
            .find(scientific_name)
            .filter(|grams| *grams > 0.0)
            .map(|grams| Fact::Weight { grams });
        let [
            wingspan,
            incubation,
            oldest,
            range,
            north,
            south,
            migration,
            diet,
            lifestyle,
        ] = fields;
        let north: Option<u8> = north
            .parse::<i16>()
            .ok()
            .and_then(|degrees| u8::try_from(degrees).ok());
        let south: Option<u8> = south
            .parse::<i16>()
            .ok()
            .and_then(|degrees| u8::try_from(-degrees).ok());
        [
            weight,
            wingspan
                .parse()
                .ok()
                .map(|centimetres| Fact::Wingspan { centimetres }),
            incubation
                .parse()
                .ok()
                .map(|days| Fact::Incubation { days }),
            oldest.parse().ok().map(|years| Fact::Oldest { years }),
            range.parse::<u32>().ok().map(|thousands| Fact::Range {
                square_kilometres: thousands.saturating_mul(1_000),
            }),
            north
                .filter(|degrees| *degrees >= FAR_LATITUDE)
                .map(|degrees| Fact::North { degrees }),
            south
                .filter(|degrees| *degrees >= FAR_LATITUDE - 15)
                .map(|degrees| Fact::South { degrees }),
            migration_of(migration).map(Fact::Migration),
            diet_of(diet).map(Fact::Diet),
            lifestyle_of(lifestyle).map(Fact::Lifestyle),
        ]
        .into_iter()
        .flatten()
    }

    /// Returns one of the species' facts for day `day` (days since 1970): another each day,
    /// in an order of the species' own, so birds on the same page tell different things.
    #[must_use]
    pub fn fact_of_the_day(&self, scientific_name: &str, day: i64) -> Option<Fact> {
        let count: usize = self.facts(scientific_name).count();
        if count == 0 {
            return None;
        }
        let count_i64: i64 = i64::try_from(count).unwrap_or(1);
        let start: i64 = i64::from(name_hash(scientific_name) % 1024);
        let index: usize = usize::try_from((day + start).rem_euclid(count_i64)).unwrap_or(0);
        self.facts(scientific_name).nth(index)
    }
}

fn migration_of(code: &str) -> Option<Migration> {
    match code {
        "1" => Some(Migration::Resident),
        "2" => Some(Migration::Partial),
        "3" => Some(Migration::Migrant),
        _ => None,
    }
}

fn diet_of(code: &str) -> Option<Diet> {
    match code {
        "I" => Some(Diet::Invertebrates),
        "G" => Some(Diet::Seeds),
        "F" => Some(Diet::Fruit),
        "N" => Some(Diet::Nectar),
        "H" => Some(Diet::Plants),
        "W" => Some(Diet::WaterPlants),
        "A" => Some(Diet::WaterAnimals),
        "V" => Some(Diet::Vertebrates),
        "S" => Some(Diet::Carrion),
        "O" => Some(Diet::Anything),
        _ => None,
    }
}

fn lifestyle_of(code: &str) -> Option<Lifestyle> {
    match code {
        "A" => Some(Lifestyle::Aerial),
        "Q" => Some(Lifestyle::Aquatic),
        "T" => Some(Lifestyle::Terrestrial),
        "I" => Some(Lifestyle::Perching),
        _ => None,
    }
}

/// FNV-1a of the name in lower case, stable across runs and platforms.
fn name_hash(scientific_name: &str) -> u32 {
    scientific_name
        .split_ascii_whitespace()
        .flat_map(str::bytes)
        .fold(0x811c_9dc5, |hash, byte| {
            (hash ^ u32::from(byte.to_ascii_lowercase())).wrapping_mul(0x0100_0193)
        })
}

#[cfg(test)]
mod test {
    use super::*;

    extern crate alloc;

    use alloc::vec::Vec;

    #[test]
    fn bundled_table_is_sorted_and_reads() {
        let facts = BirdFacts::bundled();
        assert!(facts.len() > 6_000);
        let blackbird: Vec<Fact> = facts.facts("turdus  MERULA").collect();
        assert_eq!(
            blackbird,
            [
                Fact::Weight { grams: 103.0 },
                Fact::Wingspan { centimetres: 36 },
                Fact::Incubation { days: 13 },
                Fact::Range {
                    square_kilometres: 10_572_000
                },
                Fact::North { degrees: 71 },
                Fact::Migration(Migration::Resident),
                Fact::Diet(Diet::Anything),
            ]
        );
    }

    #[test]
    fn a_fact_a_day_in_turn() {
        let facts = BirdFacts::bundled();
        let all: Vec<Fact> = facts.facts("Parus major").collect();
        let week: Vec<Fact> = (0..i64::try_from(all.len()).expect("small"))
            .filter_map(|day| facts.fact_of_the_day("Parus major", 20_000 + day))
            .collect();
        assert_eq!(week.len(), all.len());
        for fact in &all {
            assert!(week.contains(fact), "{fact:?} is never told");
        }
        assert_eq!(
            facts.fact_of_the_day("Parus major", 20_000),
            facts.fact_of_the_day("parus major", 20_000)
        );
    }

    #[test]
    fn reads_any_table_and_skips_empty_fields() {
        let table =
            NameTable::parse_csv("h\nAptenodytes forsteri,,64,,,-60,-78,1,A,Q\nZz zz,,,,,,,,,\n")
                .expect("sorted");
        let sizes = BirdSizes::parse_csv("h\n").expect("empty");
        let facts = BirdFacts::new(table, sizes);
        let penguin: Vec<Fact> = facts.facts("Aptenodytes forsteri").collect();
        assert_eq!(
            penguin,
            [
                Fact::Incubation { days: 64 },
                Fact::South { degrees: 78 },
                Fact::Migration(Migration::Resident),
                Fact::Diet(Diet::WaterAnimals),
                Fact::Lifestyle(Lifestyle::Aquatic),
            ]
        );
        assert_eq!(facts.facts("Zz zz").count(), 0);
        assert_eq!(facts.fact_of_the_day("Zz zz", 1), None);
    }
}
