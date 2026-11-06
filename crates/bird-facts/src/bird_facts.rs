//! The bundled fact tables.

use bird_sizes::{BirdSizes, NameTable};

use crate::fact::Fact;

/// Wikidata's numbers for the species BirdNET v2.4 can detect.
const BUNDLED: &str = include_str!("../data/bird_facts.csv");
/// Notes on birds common in Sweden, by language.
const BUNDLED_NOTES: [(&str, &str); 2] = [
    ("en", include_str!("../data/notes_en.csv")),
    ("sv", include_str!("../data/notes_sv.csv")),
];

/// The table's columns after the name, in order.
const COLUMNS: usize = 3;
/// How many languages' notes a table can hold.
const NOTE_LANGUAGES: usize = 4;

/// Facts per species: notes in a few languages, and numbers for every species.
///
/// The numbers table's columns are `scientific_name`, `wingspan_cm`, `incubation_days` and
/// `oldest_years`; any may be empty. A notes table's are `scientific_name` and `notes`, the
/// notes separated by `|`. Rows are sorted by name ignoring case.
#[derive(Clone, Copy, Debug)]
pub struct BirdFacts<'a> {
    table: NameTable<'a>,
    sizes: BirdSizes<'a>,
    notes: [Option<(&'a str, NameTable<'a>)>; NOTE_LANGUAGES],
}

impl BirdFacts<'static> {
    /// Returns the bundled tables, with English and Swedish notes.
    #[must_use]
    #[expect(
        clippy::expect_used,
        clippy::missing_panics_doc,
        reason = "the bundled tables are checked by a test"
    )]
    pub fn bundled() -> Self {
        let mut facts = Self::new(
            NameTable::parse_csv(BUNDLED).expect("the bundled table is sorted"),
            BirdSizes::bundled(),
        );
        for (language, text) in BUNDLED_NOTES {
            facts = facts.with_notes(
                language,
                NameTable::parse_csv(text).expect("the bundled notes are sorted"),
            );
        }
        facts
    }
}

impl<'a> BirdFacts<'a> {
    /// Creates a table from `table`, with the columns described on [`BirdFacts`], and the
    /// masses in `sizes`, with no notes.
    #[must_use]
    pub fn new(table: NameTable<'a>, sizes: BirdSizes<'a>) -> Self {
        Self {
            table,
            sizes,
            notes: [None; NOTE_LANGUAGES],
        }
    }

    /// Adds the notes in `table` for pages in `language` (`sv`), unless the table already holds
    /// as many languages as it can.
    #[must_use]
    pub fn with_notes(mut self, language: &'a str, table: NameTable<'a>) -> Self {
        if let Some(slot) = self.notes.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some((language, table));
        }
        self
    }

    /// Returns how many species have numbers besides their mass.
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Returns whether the table has no species.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns what is known about the species with this scientific name, compared ignoring
    /// case and spacing, for a page in `language` (`sv`, `sv-SE`): its notes in that language
    /// if it has any, else its numbers, in a fixed order. Empty for a species nothing is known
    /// about.
    pub fn facts(&self, scientific_name: &str, language: &str) -> impl Iterator<Item = Fact<'a>> {
        let main: &str = language.split(['-', '_']).next().unwrap_or("");
        let notes: Option<&'a str> = self
            .notes
            .iter()
            .flatten()
            .find(|(known, _)| known.eq_ignore_ascii_case(main))
            .and_then(|(_, table)| table.row(scientific_name));
        let numbers: [Option<Fact<'a>>; 4] = if notes.is_some() {
            [None; 4]
        } else {
            self.numbers(scientific_name)
        };
        notes
            .into_iter()
            .flat_map(|row| row.split('|'))
            .map(str::trim)
            .filter(|note| !note.is_empty())
            .map(Fact::Note)
            .chain(numbers.into_iter().flatten())
    }

    /// Returns one of the species' facts for day `day` (days since 1970), as [`facts`]
    /// gives them: another each day, in an order of the species' own, so birds on the same
    /// page tell different things.
    ///
    /// [`facts`]: Self::facts
    #[must_use]
    pub fn fact_of_the_day(
        &self,
        scientific_name: &str,
        language: &str,
        day: i64,
    ) -> Option<Fact<'a>> {
        let count: usize = self.facts(scientific_name, language).count();
        if count == 0 {
            return None;
        }
        let count_i64: i64 = i64::try_from(count).unwrap_or(1);
        let start: i64 = i64::from(name_hash(scientific_name) % 1024);
        let index: usize = usize::try_from((day + start).rem_euclid(count_i64)).unwrap_or(0);
        self.facts(scientific_name, language).nth(index)
    }

    /// The species' mass, wingspan, incubation and greatest age, where known.
    fn numbers(&self, scientific_name: &str) -> [Option<Fact<'a>>; 4] {
        let mut fields: [&'a str; COLUMNS] = [""; COLUMNS];
        if let Some(row) = self.table.row(scientific_name) {
            for (field, value) in fields.iter_mut().zip(row.split(',')) {
                *field = value.trim();
            }
        }
        let [wingspan, incubation, oldest] = fields;
        [
            self.sizes
                .find(scientific_name)
                .filter(|grams| *grams > 0.0)
                .map(|grams| Fact::Weight { grams }),
            wingspan
                .parse()
                .ok()
                .map(|centimetres| Fact::Wingspan { centimetres }),
            incubation
                .parse()
                .ok()
                .map(|days| Fact::Incubation { days }),
            oldest.parse().ok().map(|years| Fact::Oldest { years }),
        ]
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
    fn bundled_tables_are_sorted_and_read() {
        let facts = BirdFacts::bundled();
        assert!(facts.len() > 2_000);
        let blackbird: Vec<Fact> = facts.facts("turdus  MERULA", "fi").collect();
        assert_eq!(
            blackbird,
            [
                Fact::Weight { grams: 103.0 },
                Fact::Wingspan { centimetres: 36 },
                Fact::Incubation { days: 13 },
            ]
        );
        let swedish: Vec<Fact> = facts.facts("Turdus merula", "sv-SE").collect();
        assert_eq!(swedish.len(), 3);
        assert_eq!(
            swedish.first(),
            Some(&Fact::Note(
                "var en skygg skogsfågel tills den flyttade in i städerna på 1800-talet"
            ))
        );
        assert_eq!(facts.facts("Turdus merula", "en").count(), 3);
    }

    #[test]
    fn every_note_is_short_and_in_both_languages() {
        let facts = BirdFacts::bundled();
        for (language, text) in BUNDLED_NOTES {
            let table = NameTable::parse_csv(text).expect("sorted");
            assert!(table.len() > 100, "{language}");
            for (name, _) in table.rows() {
                let notes: Vec<Fact> = facts.facts(name, language).collect();
                assert_eq!(
                    notes.len(),
                    facts.facts(name, "en").count(),
                    "{name} has as many notes in {language} as in English"
                );
                for note in notes {
                    let Fact::Note(note) = note else {
                        panic!("{name} has no notes in {language}");
                    };
                    assert!(note.chars().count() <= 72, "{name}: {note}");
                }
            }
        }
    }

    #[test]
    fn a_fact_a_day_in_turn() {
        let facts = BirdFacts::bundled();
        for language in ["sv", "de"] {
            let all: Vec<Fact> = facts.facts("Parus major", language).collect();
            let days: Vec<Fact> = (0..i64::try_from(all.len()).expect("small"))
                .filter_map(|day| facts.fact_of_the_day("Parus major", language, 20_000 + day))
                .collect();
            assert_eq!(days.len(), all.len());
            for fact in &all {
                assert!(days.contains(fact), "{fact:?} is never told");
            }
        }
        assert_eq!(
            facts.fact_of_the_day("Parus major", "sv", 20_000),
            facts.fact_of_the_day("parus major", "sv", 20_000)
        );
    }

    #[test]
    fn reads_any_table_and_skips_empty_fields() {
        let table =
            NameTable::parse_csv("h\nAptenodytes forsteri,,64,\nZz zz,,,\n").expect("sorted");
        let notes =
            NameTable::parse_csv("h\nAptenodytes forsteri,dives 500 m| |breeds in winter\n")
                .expect("sorted");
        let sizes = BirdSizes::parse_csv("h\n").expect("empty");
        let facts = BirdFacts::new(table, sizes).with_notes("en", notes);
        let penguin: Vec<Fact> = facts.facts("Aptenodytes forsteri", "pt").collect();
        assert_eq!(penguin, [Fact::Incubation { days: 64 }]);
        let penguin: Vec<Fact> = facts.facts("Aptenodytes forsteri", "en-GB").collect();
        assert_eq!(
            penguin,
            [Fact::Note("dives 500 m"), Fact::Note("breeds in winter")]
        );
        assert_eq!(facts.facts("Zz zz", "en").count(), 0);
        assert_eq!(facts.fact_of_the_day("Zz zz", "en", 1), None);
    }
}
