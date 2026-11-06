//! One thing worth knowing about a species.

/// One fact about a species: a note in the page's language, or a number whose words are the
/// page's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fact<'a> {
    /// Something special about it, written out: a habit, a song, a record or folklore.
    Note(&'a str),
    /// Its body mass.
    Weight {
        /// Grams.
        grams: f32,
    },
    /// Its wingspan, tip to tip.
    Wingspan {
        /// Centimetres.
        centimetres: u16,
    },
    /// How long its eggs take to hatch.
    Incubation {
        /// Days.
        days: u16,
    },
    /// The oldest known wild bird of the species.
    Oldest {
        /// Years.
        years: u16,
    },
}
