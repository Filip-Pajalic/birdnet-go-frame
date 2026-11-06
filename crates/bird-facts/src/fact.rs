//! One thing worth knowing about a species.

/// Whether a species migrates, as AVONET records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Migration {
    /// It stays in one place all year.
    Resident,
    /// Some birds migrate and some stay.
    Partial,
    /// It migrates.
    Migrant,
}

/// What a species mostly eats: AVONET's trophic niche.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Diet {
    /// Insects and other invertebrates.
    Invertebrates,
    /// Seeds.
    Seeds,
    /// Fruit.
    Fruit,
    /// Nectar.
    Nectar,
    /// Land plants.
    Plants,
    /// Water plants.
    WaterPlants,
    /// Fish and other animals caught in water.
    WaterAnimals,
    /// Land vertebrates: mice, birds, lizards.
    Vertebrates,
    /// Carrion.
    Carrion,
    /// Almost anything.
    Anything,
}

/// Where a species spends its time: AVONET's primary lifestyle. Generalists have none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lifestyle {
    /// Mostly in the air.
    Aerial,
    /// Mostly on or in water.
    Aquatic,
    /// Mostly on the ground.
    Terrestrial,
    /// Mostly perched in trees and bushes.
    Perching,
}

/// One fact about a species, as numbers and kinds; the words are the page's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fact {
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
    /// The area of its range.
    Range {
        /// Square kilometres.
        square_kilometres: u32,
    },
    /// How far north it is found, when that is far.
    North {
        /// Degrees of latitude north.
        degrees: u8,
    },
    /// How far south it is found, when that is far.
    South {
        /// Degrees of latitude south.
        degrees: u8,
    },
    /// Whether it migrates.
    Migration(Migration),
    /// What it mostly eats.
    Diet(Diet),
    /// Where it spends its time.
    Lifestyle(Lifestyle),
}
