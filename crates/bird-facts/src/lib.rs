//! Something worth knowing about each bird species, for one line about the page's main bird.
//!
//! [`BirdFacts::bundled`] embeds notes written for this project on about 140 birds common in
//! Sweden, in English and Swedish: habits, songs, records and folklore. For other birds and
//! languages it falls back to numbers from Wikidata (CC0) and AVONET (CC BY 4.0) for the
//! species BirdNET can detect; see `data/ATTRIBUTION.md`. Nothing is looked up online.
//! [`BirdFacts::fact_of_the_day`] picks one [`Fact`] per species and day, so the line changes
//! daily but not between wakes. `no_std`, no allocation.
//!
//! ```
//! use bird_facts::{BirdFacts, Fact};
//!
//! let facts = BirdFacts::bundled();
//! let great_tit: Vec<Fact> = facts.facts("Parus major", "de").collect();
//! assert!(great_tit.contains(&Fact::Wingspan { centimetres: 23 }));
//! assert!(matches!(
//!     facts.fact_of_the_day("Parus major", "sv", 20_400),
//!     Some(Fact::Note(_))
//! ));
//! assert_eq!(facts.fact_of_the_day("Not a bird", "sv", 20_400), None);
//! ```

#![no_std]

mod bird_facts;
mod fact;

pub use bird_facts::BirdFacts;
pub use fact::Fact;
